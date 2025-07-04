use std::sync::Arc;

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use diesel::{
    CombineDsl, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
};
use diesel_async::RunQueryDsl;
use pkcs8::DecodePublicKey;
use sha2::Digest;
use tonic_types::{ErrorDetails, StatusExt};

use crate::CoreConfig;
use crate::chrono_ext::ChronoDateTimeExt;
use crate::google_api::GoogleIdToken;
use crate::id::Id;
use crate::middleware::IpAddressInfo;
use crate::models::{User, UserEmail, UserId, UserSession};
use crate::schema::{mfa_totps, mfa_webauthn_pks, user_emails, user_sessions, users};
use crate::std_ext::ResultExt;
use crate::utils::generate_random_bytes;

pub(crate) fn encrypt_token(
    algorithm: pb::scufflecloud::core::v1::DeviceAlgorithm,
    token: &[u8],
    pk_der_data: &[u8],
) -> Result<Vec<u8>, tonic::Status> {
    match algorithm {
        pb::scufflecloud::core::v1::DeviceAlgorithm::RsaOaepSha256 => {
            let pk = rsa::RsaPublicKey::from_public_key_der(pk_der_data)
                .into_tonic_err_with_field_violation("public_key_data", "failed to parse public key")?;
            let padding = rsa::Oaep::new::<sha2::Sha256>();
            let enc_data = pk
                .encrypt(&mut rsa::rand_core::OsRng, padding, token)
                .into_tonic_internal_err("failed to encrypt token")?;
            Ok(enc_data)
        }
    }
}

pub(crate) async fn get_user_by_email(db: &mut diesel_async::AsyncPgConnection, email: &str) -> Result<User, tonic::Status> {
    let Some((user, _)) = users::dsl::users
        .inner_join(user_emails::dsl::user_emails.on(users::dsl::primary_email.eq(user_emails::dsl::email.nullable())))
        .filter(user_emails::dsl::email.eq(&email))
        .select((User::as_select(), user_emails::dsl::email))
        .first::<(User, String)>(&mut *db)
        .await
        .optional()
        .into_tonic_internal_err("failed to query user by email")?
    else {
        return Err(tonic::Status::with_error_details(
            tonic::Code::NotFound,
            "user not found",
            ErrorDetails::new(),
        ));
    };

    Ok(user)
}

pub(crate) fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

pub(crate) struct NewUserData<'a> {
    pub email: Option<String>,
    pub preferred_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password: Option<&'a str>,
}

impl From<GoogleIdToken> for NewUserData<'_> {
    fn from(value: GoogleIdToken) -> Self {
        Self {
            email: value.email_verified.then_some(value.email),
            preferred_name: value.name,
            first_name: value.given_name,
            last_name: value.family_name,
            password: None,
        }
    }
}

pub(crate) async fn create_new_user_and_session<G: CoreConfig>(
    global: &Arc<G>,
    db: &mut diesel_async::AsyncPgConnection,
    new_user_data: NewUserData<'_>,
    device: pb::scufflecloud::core::v1::Device,
    ip_info: &IpAddressInfo,
) -> Result<(User, pb::scufflecloud::core::v1::NewUserSessionToken), tonic::Status> {
    let email = new_user_data.email.as_ref().map(|e| normalize_email(e));

    let password_hash = if let Some(password) = new_user_data.password {
        // Create user with given password
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .into_tonic_internal_err("failed to hash password")?
            .to_string();
        Some(hash)
    } else {
        None
    };

    let user = User {
        id: Id::new(),
        preferred_name: new_user_data.preferred_name,
        first_name: new_user_data.first_name,
        last_name: new_user_data.last_name,
        password_hash,
        primary_email: email.clone(),
    };
    diesel::insert_into(users::dsl::users)
        .values(&user)
        .execute(db)
        .await
        .into_tonic_internal_err("failed to insert user")?;

    if let Some(email) = email {
        let user_email = UserEmail {
            email: email.clone(),
            user_id: user.id,
            created_at: chrono::Utc::now(),
        };
        diesel::insert_into(user_emails::dsl::user_emails)
            .values(&user_email)
            .execute(db)
            .await
            .into_tonic_internal_err("failed to insert user email")?;
    }

    let new_token = create_session(global, db, user.id, device, ip_info, false).await?;

    Ok((user, new_token))
}

pub(crate) async fn create_session<G: CoreConfig>(
    global: &Arc<G>,
    db: &mut diesel_async::AsyncPgConnection,
    user_id: UserId,
    device: pb::scufflecloud::core::v1::Device,
    ip_info: &IpAddressInfo,
    check_mfa: bool,
) -> Result<pb::scufflecloud::core::v1::NewUserSessionToken, tonic::Status> {
    let mfa_pending = check_mfa
        && !mfa_totps::dsl::mfa_totps
            .filter(mfa_totps::dsl::user_id.eq(user_id))
            .select(mfa_totps::dsl::user_id)
            .union(
                mfa_webauthn_pks::dsl::mfa_webauthn_pks
                    .filter(mfa_webauthn_pks::dsl::user_id.eq(user_id))
                    .select(mfa_webauthn_pks::dsl::user_id),
            )
            .load::<UserId>(db)
            .await
            .into_tonic_internal_err("failed to query mfa factors")?
            .is_empty();

    // Create user session, device and token
    let device_fingerprint = sha2::Sha256::digest(&device.public_key_data).to_vec();

    let session_expires_at = if mfa_pending {
        chrono::Utc::now() + global.mfa_timeout()
    } else {
        chrono::Utc::now() + global.user_session_timeout()
    };
    let token_id = Id::new();
    let token_expires_at = chrono::Utc::now() + global.user_session_token_timeout();

    let token = generate_random_bytes().into_tonic_internal_err("failed to generate token")?;
    let encrypted_token = encrypt_token(device.algorithm(), &token, &device.public_key_data)?;

    let user_session = UserSession {
        user_id,
        device_fingerprint,
        device_algorithm: device.algorithm().into(),
        device_pk_data: device.public_key_data,
        last_used_at: chrono::Utc::now(),
        last_ip: ip_info.to_network(),
        token_id: Some(token_id),
        token: Some(token.to_vec()),
        token_expires_at: Some(token_expires_at),
        expires_at: session_expires_at,
        mfa_pending,
    };
    diesel::insert_into(user_sessions::dsl::user_sessions)
        .values(&user_session)
        .execute(db)
        .await
        .into_tonic_internal_err("failed to insert user session")?;

    let new_token = pb::scufflecloud::core::v1::NewUserSessionToken {
        id: token_id.to_string(),
        encrypted_token,
        expires_at: Some(token_expires_at.to_prost_timestamp_utc()),
        session_mfa_pending: mfa_pending,
    };

    Ok(new_token)
}
