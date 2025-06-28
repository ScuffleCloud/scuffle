use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use diesel_async::RunQueryDsl;
use sha2::Digest;

use crate::chrono_datetime_ext::ChronoDateTimeExt;
use crate::id::Id;
use crate::middleware::IpAddressInfo;
use crate::models::{User, UserEmail, UserSession};
use crate::result_ext::ResultExt;
use crate::schema::{user_emails, user_sessions, users};
use crate::services::sessions::crypto;

pub(crate) async fn create_new_user_and_session(
    db: &mut diesel_async::AsyncPgConnection,
    email: String,
    password: Option<&str>,
    device: pb::scufflecloud::core::v1::Device,
    ip_info: &IpAddressInfo,
) -> Result<(User, pb::scufflecloud::core::v1::NewUserSessionToken), tonic::Status> {
    let password_hash = if let Some(password) = password {
        // Create user with given password
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let argon2 = Argon2::default();

        Some(
            argon2
                .hash_password(password.as_bytes(), &salt)
                .into_tonic_internal("failed to hash password")?
                .to_string(),
        )
    } else {
        None
    };

    let user = User {
        id: Id::new(),
        preferred_name: None,
        first_name: None,
        last_name: None,
        password_hash,
        primary_email: email.clone(),
    };
    diesel::insert_into(users::dsl::users)
        .values(&user)
        .execute(db)
        .await
        .into_tonic_internal("failed to insert user")?;

    let user_email = UserEmail {
        email,
        user_id: user.id,
        created_at: chrono::Utc::now(),
    };
    diesel::insert_into(user_emails::dsl::user_emails)
        .values(&user_email)
        .execute(db)
        .await
        .into_tonic_internal("failed to insert user email")?;

    // TODO: Process existing invites

    // Create user session, device and token
    let device_fingerprint = sha2::Sha256::digest(&device.public_key_data).to_vec();

    let token_id = Id::new();
    let token_expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    let session_expires_at = chrono::Utc::now() + chrono::Duration::days(30);

    let token = crypto::generate_token().into_tonic_internal("failed to generate token")?;

    let encrypted_token = crypto::encrypt_token(device.algorithm(), &token, &device.public_key_data)?;

    let user_session = UserSession {
        user_id: user.id,
        device_fingerprint,
        device_algorithm: device.algorithm().into(),
        device_pk_data: device.public_key_data,
        last_used_at: chrono::Utc::now(),
        last_ip: ip_info.to_network(),
        token_id: Some(token_id),
        token: Some(token.to_vec()),
        token_expires_at: Some(token_expires_at),
        expires_at: session_expires_at,
    };
    diesel::insert_into(user_sessions::dsl::user_sessions)
        .values(&user_session)
        .execute(db)
        .await
        .into_tonic_internal("failed to insert user session")?;

    let new_token = pb::scufflecloud::core::v1::NewUserSessionToken {
        id: token_id.to_string(),
        encrypted_token,
        expires_at: Some(token_expires_at.to_prost_timestamp_utc()),
        session_expires_at: Some(session_expires_at.to_prost_timestamp_utc()),
    };

    Ok((user, new_token))
}
