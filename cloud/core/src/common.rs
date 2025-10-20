use std::sync::Arc;

use argon2::{Argon2, PasswordVerifier};
use core_db_types::models::{
    MfaRecoveryCode, MfaWebauthnCredential, MfaWebauthnCredential, Organization, Organization, OrganizationId, User, User,
    UserEmail, UserId, UserSession, UserSession, UserSessionTokenId,
};
use core_db_types::schema::{
    mfa_recovery_codes, mfa_totp_credentials, mfa_totp_credentials, mfa_webauthn_auth_sessions, mfa_webauthn_credentials,
    mfa_webauthn_credentials, organizations, organizations, user_emails, user_sessions, user_sessions, users, users,
};
use core_traits::EmailServiceClient;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{ChronoDateTimeExt, DisplayExt, OptionExt, ResultExt};
use geo_ip::maxminddb;
use geo_ip::middleware::IpAddressInfo;
use pkcs8::DecodePublicKey;
use rand::RngCore;
use sha2::Digest;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

pub(crate) fn email_to_pb<G: core_traits::ConfigInterface>(
    global: &Arc<G>,
    to_address: String,
    to_name: Option<String>,
    email: core_emails::Email,
) -> pb::scufflecloud::email::v1::SendEmailRequest {
    pb::scufflecloud::email::v1::SendEmailRequest {
        from: Some(pb::scufflecloud::email::v1::EmailAddress {
            name: Some(global.email_from_name().to_string()),
            address: global.email_from_address().to_string(),
        }),
        to: Some(pb::scufflecloud::email::v1::EmailAddress {
            name: to_name,
            address: to_address,
        }),
        subject: email.subject,
        text: email.text,
        html: email.html,
    }
}

pub(crate) fn generate_random_bytes() -> Result<[u8; 32], rand::Error> {
    let mut token = [0u8; 32];
    rand::rngs::OsRng.try_fill_bytes(&mut token)?;
    Ok(token)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TxError {
    #[error("diesel transaction error: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("tonic status error: {0}")]
    Status(#[from] tonic::Status),
}

impl From<TxError> for tonic::Status {
    fn from(err: TxError) -> Self {
        match err {
            TxError::Diesel(e) => e.into_tonic_internal_err("transaction error"),
            TxError::Status(s) => s,
        }
    }
}

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

pub(crate) async fn get_user_by_id<G: core_traits::Global>(global: &Arc<G>, user_id: UserId) -> Result<User, tonic::Status> {
    global
        .user_loader()
        .load(user_id)
        .await
        .ok()
        .into_tonic_internal_err("failed to query user")?
        .into_tonic_not_found("user not found")
}

pub(crate) async fn get_user_by_id_in_tx(
    db: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    user_id: UserId,
) -> Result<User, tonic::Status> {
    let user = users::dsl::users
        .find(user_id)
        .select(User::as_select())
        .first::<User>(db)
        .await
        .optional()
        .into_tonic_internal_err("failed to query user")?
        .into_tonic_not_found("user not found")?;

    Ok(user)
}

pub(crate) async fn get_user_by_email(
    db: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    email: &str,
) -> Result<Option<User>, tonic::Status> {
    let user = users::dsl::users
        .inner_join(user_emails::dsl::user_emails.on(users::dsl::id.eq(user_emails::dsl::user_id)))
        .filter(user_emails::dsl::email.eq(&email))
        .select(User::as_select())
        .first::<User>(db)
        .await
        .optional()
        .into_tonic_internal_err("failed to query user by email")?;

    Ok(user)
}

pub(crate) async fn get_organization_by_id<G: core_traits::Global>(
    global: &Arc<G>,
    organization_id: OrganizationId,
) -> Result<Organization, tonic::Status> {
    let organization = global
        .organization_loader()
        .load(organization_id)
        .await
        .ok()
        .into_tonic_internal_err("failed to query organization")?
        .into_tonic_not_found("organization not found")?;

    Ok(organization)
}

pub(crate) async fn get_organization_by_id_in_tx(
    db: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    organization_id: OrganizationId,
) -> Result<Organization, tonic::Status> {
    let organization = organizations::dsl::organizations
        .find(organization_id)
        .first::<Organization>(db)
        .await
        .optional()
        .into_tonic_internal_err("failed to load organization")?
        .ok_or_else(|| {
            tonic::Status::with_error_details(tonic::Code::NotFound, "organization not found", ErrorDetails::new())
        })?;

    Ok(organization)
}

pub(crate) fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

pub(crate) async fn create_user(
    tx: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    new_user: &User,
) -> Result<(), tonic::Status> {
    diesel::insert_into(users::dsl::users)
        .values(new_user)
        .execute(tx)
        .await
        .into_tonic_internal_err("failed to insert user")?;

    if let Some(email) = new_user.primary_email.as_ref() {
        // Check if email is already registered
        if user_emails::dsl::user_emails
            .find(email)
            .select(user_emails::dsl::email)
            .first::<String>(tx)
            .await
            .optional()
            .into_tonic_internal_err("failed to query user emails")?
            .is_some()
        {
            return Err(tonic::Status::with_error_details(
                Code::AlreadyExists,
                "email is already registered",
                ErrorDetails::new(),
            ));
        }

        let user_email = UserEmail {
            email: email.clone(),
            user_id: new_user.id,
            created_at: chrono::Utc::now(),
        };

        diesel::insert_into(user_emails::dsl::user_emails)
            .values(&user_email)
            .execute(tx)
            .await
            .into_tonic_internal_err("failed to insert user email")?;
    }

    Ok(())
}

pub(crate) async fn mfa_options(
    tx: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    user_id: UserId,
) -> Result<Vec<pb::scufflecloud::core::v1::MfaOption>, tonic::Status> {
    let mut mfa_options = vec![];

    if mfa_totp_credentials::dsl::mfa_totp_credentials
        .filter(mfa_totp_credentials::dsl::user_id.eq(user_id))
        .count()
        .get_result::<i64>(tx)
        .await
        .into_tonic_internal_err("failed to query mfa factors")?
        > 0
    {
        mfa_options.push(pb::scufflecloud::core::v1::MfaOption::Totp);
    }

    if mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
        .filter(mfa_webauthn_credentials::dsl::user_id.eq(user_id))
        .count()
        .get_result::<i64>(tx)
        .await
        .into_tonic_internal_err("failed to query mfa factors")?
        > 0
    {
        mfa_options.push(pb::scufflecloud::core::v1::MfaOption::WebAuthn);
    }

    if mfa_recovery_codes::dsl::mfa_recovery_codes
        .filter(mfa_recovery_codes::dsl::user_id.eq(user_id))
        .count()
        .get_result::<i64>(tx)
        .await
        .into_tonic_internal_err("failed to query mfa factors")?
        > 0
    {
        mfa_options.push(pb::scufflecloud::core::v1::MfaOption::RecoveryCodes);
    }

    Ok(mfa_options)
}

pub(crate) async fn create_session<G: core_traits::Global>(
    global: &Arc<G>,
    tx: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    dashboard_origin: &url::Url,
    user: &User,
    device: pb::scufflecloud::core::v1::Device,
    ip_info: &IpAddressInfo,
    check_mfa: bool,
) -> Result<pb::scufflecloud::core::v1::NewUserSessionToken, tonic::Status> {
    let mfa_options = if check_mfa { mfa_options(tx, user.id).await? } else { vec![] };

    // Create user session, device and token
    let device_fingerprint = sha2::Sha256::digest(&device.public_key_data).to_vec();

    let session_expires_at = if !mfa_options.is_empty() {
        chrono::Utc::now() + global.timeout_config().mfa
    } else {
        chrono::Utc::now() + global.timeout_config().user_session
    };
    let token_id = UserSessionTokenId::new();
    let token_expires_at = chrono::Utc::now() + global.timeout_config().user_session_token;

    let token = generate_random_bytes().into_tonic_internal_err("failed to generate token")?;
    let encrypted_token = encrypt_token(device.algorithm(), &token, &device.public_key_data)?;

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
        mfa_pending: !mfa_options.is_empty(),
    };

    // Upsert session
    // This is an upsert because the user might have already had a session for this device at some point
    diesel::insert_into(user_sessions::dsl::user_sessions)
        .values(&user_session)
        .on_conflict((user_sessions::dsl::user_id, user_sessions::dsl::device_fingerprint))
        .do_update()
        .set((
            user_sessions::dsl::last_used_at.eq(user_session.last_used_at),
            user_sessions::dsl::last_ip.eq(user_session.last_ip),
            user_sessions::dsl::token_id.eq(user_session.token_id),
            user_sessions::dsl::token.eq(token.to_vec()),
            user_sessions::dsl::token_expires_at.eq(user_session.token_expires_at),
            user_sessions::dsl::expires_at.eq(user_session.expires_at),
            user_sessions::dsl::mfa_pending.eq(user_session.mfa_pending),
        ))
        .execute(tx)
        .await
        .into_tonic_internal_err("failed to insert user session")?;

    let new_token = pb::scufflecloud::core::v1::NewUserSessionToken {
        id: token_id.to_string(),
        encrypted_token,
        user_id: user.id.to_string(),
        expires_at: Some(token_expires_at.to_prost_timestamp_utc()),
        session_expires_at: Some(session_expires_at.to_prost_timestamp_utc()),
        session_mfa_pending: user_session.mfa_pending,
        mfa_options: mfa_options.into_iter().map(|o| o as i32).collect(),
    };

    if let Some(primary_email) = user.primary_email.as_ref() {
        let geo_info = ip_info
            .lookup_geoip_info::<maxminddb::geoip2::City>(&**global)
            .into_tonic_internal_err("failed to lookup geoip info")?
            .map(Into::into)
            .unwrap_or_default();
        let email = core_emails::new_device_email(dashboard_origin, ip_info.ip_address, geo_info)
            .into_tonic_internal_err("failed to render email")?;
        let email = email_to_pb(global, primary_email.clone(), user.preferred_name.clone(), email);

        global
            .email_service()
            .send_email(email)
            .await
            .into_tonic_internal_err("failed to send new device email")?;
    }

    Ok(new_token)
}

pub(crate) fn verify_password(password_hash: &str, password: &str) -> Result<(), tonic::Status> {
    let password_hash = argon2::PasswordHash::new(password_hash).into_tonic_internal_err("failed to parse password hash")?;

    match Argon2::default().verify_password(password.as_bytes(), &password_hash) {
        Ok(_) => Ok(()),
        Err(argon2::password_hash::Error::Password) => Err(tonic::Status::with_error_details(
            tonic::Code::PermissionDenied,
            "invalid password",
            ErrorDetails::with_bad_request_violation("password", "invalid password"),
        )),
        Err(_) => Err(tonic::Status::with_error_details(
            tonic::Code::Internal,
            "failed to verify password",
            ErrorDetails::new(),
        )),
    }
}

pub(crate) async fn finish_webauthn_authentication<G: core_traits::Global>(
    global: &Arc<G>,
    tx: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    user_id: UserId,
    reg: &webauthn_rs::prelude::PublicKeyCredential,
) -> Result<(), tonic::Status> {
    let state = diesel::delete(mfa_webauthn_auth_sessions::dsl::mfa_webauthn_auth_sessions)
        .filter(
            mfa_webauthn_auth_sessions::dsl::user_id
                .eq(user_id)
                .and(mfa_webauthn_auth_sessions::dsl::expires_at.gt(chrono::Utc::now())),
        )
        .returning(mfa_webauthn_auth_sessions::dsl::state)
        .get_result::<serde_json::Value>(tx)
        .await
        .optional()
        .into_tonic_internal_err("failed to query webauthn authentication session")?
        .into_tonic_err(
            tonic::Code::FailedPrecondition,
            "no webauthn authentication session found",
            ErrorDetails::new(),
        )?;

    let state: webauthn_rs::prelude::PasskeyAuthentication =
        serde_json::from_value(state).into_tonic_internal_err("failed to deserialize webauthn state")?;

    let result = global
        .webauthn()
        .finish_passkey_authentication(reg, &state)
        .into_tonic_internal_err("failed to finish webauthn authentication")?;

    let counter = result.counter() as i64;

    let credential = mfa_webauthn_credentials::dsl::mfa_webauthn_credentials
        .filter(mfa_webauthn_credentials::dsl::credential_id.eq(result.cred_id().as_ref()))
        .select(MfaWebauthnCredential::as_select())
        .first::<MfaWebauthnCredential>(tx)
        .await
        .into_tonic_internal_err("failed to find webauthn credential")?;

    if counter == 0 || credential.counter.is_none_or(|c| c < counter) {
        diesel::update(mfa_webauthn_credentials::dsl::mfa_webauthn_credentials)
            .filter(mfa_webauthn_credentials::dsl::credential_id.eq(result.cred_id().as_ref()))
            .set((
                mfa_webauthn_credentials::dsl::counter.eq(counter),
                mfa_webauthn_credentials::dsl::last_used_at.eq(chrono::Utc::now()),
            ))
            .execute(tx)
            .await
            .into_tonic_internal_err("failed to update webauthn credential")?;
    } else {
        // Invalid credential
        diesel::delete(mfa_webauthn_credentials::dsl::mfa_webauthn_credentials)
            .filter(mfa_webauthn_credentials::dsl::credential_id.eq(result.cred_id().as_ref()))
            .execute(tx)
            .await
            .into_tonic_internal_err("failed to delete webauthn credential")?;

        return Err(tonic::Status::with_error_details(
            tonic::Code::FailedPrecondition,
            "invalid webauthn credential",
            ErrorDetails::new(),
        ));
    }

    Ok(())
}

pub(crate) async fn process_recovery_code(
    tx: &mut impl diesel_async::AsyncConnection<Backend = diesel::pg::Pg>,
    user_id: UserId,
    code: &str,
) -> Result<(), tonic::Status> {
    let codes = mfa_recovery_codes::dsl::mfa_recovery_codes
        .filter(mfa_recovery_codes::dsl::user_id.eq(user_id))
        .limit(20)
        .load::<MfaRecoveryCode>(tx)
        .await
        .into_tonic_internal_err("failed to load MFA recovery codes")?;

    let argon2 = Argon2::default();

    for recovery_code in codes {
        let hash = argon2::PasswordHash::new(&recovery_code.code_hash)
            .into_tonic_internal_err("failed to parse recovery code hash")?;
        match argon2.verify_password(code.as_bytes(), &hash) {
            Ok(()) => {
                diesel::delete(mfa_recovery_codes::dsl::mfa_recovery_codes)
                    .filter(mfa_recovery_codes::dsl::id.eq(recovery_code.id))
                    .execute(tx)
                    .await
                    .into_tonic_internal_err("failed to delete recovery code")?;

                break;
            }
            Err(argon2::password_hash::Error::Password) => continue,
            Err(e) => {
                return Err(e.into_tonic_internal_err("failed to verify recovery code"));
            }
        }
    }

    Ok(())
}
