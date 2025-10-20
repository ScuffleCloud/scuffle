use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono::Utc;
use core_db_types::models::{
    MfaPasskeyCredential, MfaPasskeyCredentialId, MfaTotpCredential, MfaTotpCredentialId, NewMfaPasskeyCredential,
    NewMfaRecoveryCode, NewMfaTotpCredential,
};
use core_db_types::schema::{mfa_passkey_credentials, mfa_recovery_codes, mfa_totp_credentials, users};
use core_pb::v1::user_mfa_service_server::UserMfaService;
use core_pb::v1::{
    UserMfaGetRequest, UserMfaGetResponse, UserMfaPasskeyCredentialFinalizeRequest,
    UserMfaPasskeyCredentialFinalizeResponse, UserMfaPasskeyCredentialSetupRequest, UserMfaPasskeyCredentialSetupResponse,
    UserMfaPasskeyCredentialUpdateRequest, UserMfaPasskeyCredentialUpdateResponse, UserMfaRegenerateRecoveryCodesRequest,
    UserMfaRegenerateRecoveryCodesResponse, UserMfaTotpCredentialFinalizeRequest, UserMfaTotpCredentialFinalizeResponse,
    UserMfaTotpCredentialSetupRequest, UserMfaTotpCredentialSetupResponse, UserMfaTotpCredentialUpdateRequest,
    UserMfaTotpCredentialUpdateResponse, user,
};
use core_traits::DatabaseExt;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{ChronoDateTimeExt, OptionExt, RequestExt, ResultExt};
use fred::interfaces::KeysInterface;
use fred::types::Expiration;
use rand::RngCore;
use tonic::async_trait;
use tonic_types::{ErrorDetails, StatusExt};
use webauthn_rs::prelude::{CredentialID, PasskeyRegistration, RegisterPublicKeyCredential};

use crate::auth_session::{AuthSessionExt, AuthSessionResultExt};

const TOTP_SETUP_EXPIRY: i64 = 300; // 5 minutes
const PASSKEY_SETUP_EXPIRY: i64 = webauthn_rs::DEFAULT_AUTHENTICATOR_TIMEOUT.as_secs() as i64;

fn totp_credential_to_proto(credential: MfaTotpCredential) -> user::mfa::TotpCredential {
    user::mfa::TotpCredential {
        id: credential.id.to_string(),
        name: credential.name.unwrap_or_default(),
        created_at: Some(credential.id.datetime().into()),
        last_used_at: credential.last_used_at.map(|d| d.to_prost_timestamp_utc()),
    }
}

fn passkey_credential_to_proto(credential: MfaPasskeyCredential) -> user::mfa::PasskeyCredential {
    user::mfa::PasskeyCredential {
        id: credential.id.to_string(),
        name: credential.name.unwrap_or_default(),
        credential_id: credential.credential_id,
        created_at: Some(credential.id.datetime().into()),
        last_used_at: credential.last_used_at.map(|d| d.to_prost_timestamp_utc()),
    }
}

fn generate_recovery_code() -> String {
    todo!("not implemented")
}

#[async_trait]
impl<G: core_traits::Global> UserMfaService for crate::services::CoreSvc<G> {
    async fn totp_credential_setup(
        &self,
        request: tonic::Request<UserMfaTotpCredentialSetupRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaTotpCredentialSetupResponse>> {
        let global = request.global::<G>()?;

        // Get authenticated user session
        let auth_session = request.auth_user::<G>().await?.required()?.with_mfa_required()?;

        let user = global
            .user_loader()
            .load(auth_session.id())
            .await
            .unwrap_or_default()
            .into_tonic_internal_err("failed to query user")?;

        let mut secret = vec![0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret);

        // Generate TOTP secret
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret,
            Some("scuffle.cloud".into()),
            user.primary_email.clone(),
        )
        .into_tonic_internal_err("failed to generate TOTP")?;

        let url = totp.get_url();

        let totp_credential_id = MfaTotpCredentialId::new();

        // Store setup data in Redis
        let redis_key = format!("mfa:totp:setup:{}:{}", auth_session.id(), totp_credential_id);

        let _: () = global
            .redis()
            .set(&redis_key, &url, Some(Expiration::EX(TOTP_SETUP_EXPIRY)), None, false)
            .await
            .into_tonic_internal_err("failed to store TOTP setup data")?;

        Ok(tonic::Response::new(UserMfaTotpCredentialSetupResponse {
            totp_credential_id: totp_credential_id.to_string(),
            totp_credential_url: url,
        }))
    }

    async fn totp_credential_finalize(
        &self,
        request: tonic::Request<UserMfaTotpCredentialFinalizeRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaTotpCredentialFinalizeResponse>> {
        let global = request.global::<G>()?;

        // Get authenticated user session
        let auth_session = request.auth_user::<G>().await?.required()?.with_mfa_required()?;

        let req = request.into_inner();

        // Parse credential ID
        let totp_credential_id: MfaTotpCredentialId = req
            .totp_credential_id
            .parse()
            .into_tonic_err_with_field_violation("totp_credential_id", "invalid credential ID")?;

        // Retrieve setup data from Redis
        let redis_key = format!("mfa:totp:setup:{}:{}", auth_session.id(), totp_credential_id);
        let url = global
            .redis()
            .get::<Option<String>, _>(&redis_key)
            .await
            .into_tonic_internal_err("failed to retrieve TOTP setup data")?
            .into_tonic_not_found("TOTP setup not found or expired")?;

        let totp = totp_rs::TOTP::from_url(&url).into_tonic_internal_err("failed to parse TOTP URL")?;

        let is_valid = totp
            .check_current(&req.totp_generated_token)
            .into_tonic_internal_err("failed to check TOTP generated token")?;
        if !is_valid {
            return Err(tonic::Status::with_error_details(
                tonic::Code::InvalidArgument,
                "invalid TOTP generated token",
                ErrorDetails::with_bad_request_violation("totp_generated_token", "invalid TOTP generated token"),
            ));
        }

        // Insert into database
        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let credential = db
            .tx(async |tx| {
                // Delete existing TOTP credential if any (only one per user)
                diesel::delete(mfa_totp_credentials::table.filter(mfa_totp_credentials::user_id.eq(auth_session.id())))
                    .execute(tx)
                    .await
                    .into_tonic_internal_err("failed to delete existing TOTP credential")?;

                // Insert new credential
                let new_credential = NewMfaTotpCredential::builder()
                    .id(totp_credential_id)
                    .user_id(auth_session.id())
                    .url(url)
                    .build();

                let credential = diesel::insert_into(mfa_totp_credentials::table)
                    .values(&new_credential)
                    .returning(MfaTotpCredential::as_select())
                    .get_result::<MfaTotpCredential>(tx)
                    .await
                    .into_tonic_internal_err("failed to insert TOTP credential")?;

                Ok(credential)
            })
            .await?;

        Ok(tonic::Response::new(UserMfaTotpCredentialFinalizeResponse {
            totp_credential: Some(totp_credential_to_proto(credential)),
        }))
    }

    async fn totp_credential_update(
        &self,
        request: tonic::Request<UserMfaTotpCredentialUpdateRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaTotpCredentialUpdateResponse>> {
        let global = request.global::<G>()?;

        // Get authenticated user session
        let auth_session = request.auth_user::<G>().await?.required()?.with_mfa_required()?;

        let req = request.into_inner();

        // Parse credential ID
        let totp_credential_id: MfaTotpCredentialId = req
            .totp_credential_id
            .parse()
            .into_tonic_err_with_field_violation("totp_credential_id", "invalid credential ID")?;

        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let credential = diesel::update(
            mfa_totp_credentials::table
                .filter(mfa_totp_credentials::id.eq(totp_credential_id))
                .filter(mfa_totp_credentials::user_id.eq(auth_session.id())),
        )
        .set(mfa_totp_credentials::name.eq(req.name.as_deref()))
        .returning(MfaTotpCredential::as_select())
        .get_result::<MfaTotpCredential>(&mut db)
        .await
        .optional()
        .into_tonic_internal_err("failed to update TOTP credential")?
        .into_tonic_not_found("TOTP credential not found")?;

        Ok(tonic::Response::new(UserMfaTotpCredentialUpdateResponse {
            totp_credential: Some(totp_credential_to_proto(credential)),
        }))
    }

    async fn passkey_credential_setup(
        &self,
        request: tonic::Request<UserMfaPasskeyCredentialSetupRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaPasskeyCredentialSetupResponse>> {
        let global = request.global::<G>()?;
        let user_session = request.auth_user::<G>().await?.required()?.with_mfa_required()?;

        let user = global
            .user_loader()
            .load(user_session.id())
            .await
            .unwrap_or_default()
            .into_tonic_internal_err("failed to query user")?;

        let user_display_name = user.preferred_name.or_else(|| {
            if let (Some(first_name), Some(last_name)) = (user.first_name, user.last_name) {
                Some(format!("{} {}", first_name, last_name))
            } else {
                None
            }
        });

        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let passkey_credentials = mfa_passkey_credentials::table
            .filter(mfa_passkey_credentials::user_id.eq(user_session.id()))
            .select(mfa_passkey_credentials::credential_id)
            .load::<Vec<u8>>(&mut db)
            .await
            .into_tonic_internal_err("failed to query webauthn credentials")?;

        let (options, state) = global
            .webauthn()
            .start_passkey_registration(
                user_session.id().ulid().into(),
                &user.primary_email,
                user_display_name.as_ref().unwrap_or(&user.primary_email),
                Some(passkey_credentials.into_iter().map(CredentialID::from).collect()),
            )
            .into_tonic_internal_err("failed to start WebAuthn credential registration")?;

        let options = serde_json::to_string(&options).into_tonic_internal_err("failed to serialize WebAuthn options")?;
        let state = serde_json::to_string(&state).into_tonic_internal_err("failed to serialize WebAuthn state")?;

        let passkey_credential_id = MfaPasskeyCredentialId::new();

        let _: () = global
            .redis()
            .set(
                &format!("mfa:passkey:setup:{}:{}", user_session.id(), passkey_credential_id),
                &state,
                Some(Expiration::EX(PASSKEY_SETUP_EXPIRY)),
                None,
                false,
            )
            .await
            .into_tonic_internal_err("failed to store WebAuthn setup data")?;

        Ok(tonic::Response::new(UserMfaPasskeyCredentialSetupResponse {
            passkey_credential_id: passkey_credential_id.to_string(),
            passkey_options: options,
        }))
    }

    async fn passkey_credential_finalize(
        &self,
        request: tonic::Request<UserMfaPasskeyCredentialFinalizeRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaPasskeyCredentialFinalizeResponse>> {
        let global = request.global::<G>()?;
        let user_session = request.auth_user::<G>().await?.required()?.with_mfa_required()?;

        let req = request.into_inner();

        let passkey_credential_id: MfaPasskeyCredentialId = req
            .passkey_credential_id
            .parse()
            .into_tonic_err_with_field_violation("passkey_credential_id", "invalid credential ID")?;

        let state = global
            .redis()
            .getdel::<Option<String>, _>(&format!("mfa:passkey:setup:{}:{}", user_session.id(), passkey_credential_id))
            .await
            .into_tonic_internal_err("failed to retrieve Passkey setup data")?
            .into_tonic_not_found("Passkey setup not found or expired")?;

        let response: RegisterPublicKeyCredential =
            serde_json::from_str(&req.passkey_response).into_tonic_internal_err("failed to parse Passkey response")?;
        let state: PasskeyRegistration =
            serde_json::from_str(&state).into_tonic_internal_err("failed to parse Passkey setup data")?;
        let passkey = global
            .webauthn()
            .finish_passkey_registration(&response, &state)
            .into_tonic_internal_err("failed to finish Passkey credential registration")?;

        let passkey_json = serde_json::to_value(&passkey).into_tonic_internal_err("failed to serialize Passkey passkey")?;

        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let new_credential = NewMfaPasskeyCredential::builder()
            .id(passkey_credential_id)
            .user_id(user_session.id())
            .credential_id(passkey.cred_id().to_vec())
            .credential(passkey_json)
            .counter(0)
            .build();

        let credential = diesel::insert_into(mfa_passkey_credentials::table)
            .values(&new_credential)
            .returning(MfaPasskeyCredential::as_select())
            .get_result::<MfaPasskeyCredential>(&mut db)
            .await
            .into_tonic_internal_err("failed to insert Passkey credential")?;

        Ok(tonic::Response::new(UserMfaPasskeyCredentialFinalizeResponse {
            passkey_credential: Some(passkey_credential_to_proto(credential)),
        }))
    }

    async fn passkey_credential_update(
        &self,
        request: tonic::Request<UserMfaPasskeyCredentialUpdateRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaPasskeyCredentialUpdateResponse>> {
        let global = request.global::<G>()?;
        let user_session = request.auth_user::<G>().await?.required()?.with_mfa_required()?;

        let req = request.into_inner();

        let passkey_credential_id: MfaPasskeyCredentialId = req
            .passkey_credential_id
            .parse()
            .into_tonic_err_with_field_violation("passkey_credential_id", "invalid credential ID")?;

        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let credential = diesel::update(
            mfa_passkey_credentials::table
                .filter(mfa_passkey_credentials::id.eq(passkey_credential_id))
                .filter(mfa_passkey_credentials::user_id.eq(user_session.id())),
        )
        .set(mfa_passkey_credentials::name.eq(req.name.as_deref()))
        .returning(MfaPasskeyCredential::as_select())
        .get_result::<MfaPasskeyCredential>(&mut db)
        .await
        .optional()
        .into_tonic_internal_err("failed to update Passkey credential")?
        .into_tonic_not_found("Passkey credential not found")?;

        Ok(tonic::Response::new(UserMfaPasskeyCredentialUpdateResponse {
            passkey_credential: Some(passkey_credential_to_proto(credential)),
        }))
    }

    async fn regenerate_recovery_codes(
        &self,
        request: tonic::Request<UserMfaRegenerateRecoveryCodesRequest>,
    ) -> tonic::Result<tonic::Response<UserMfaRegenerateRecoveryCodesResponse>> {
        let global = request.global::<G>()?;
        let user_session = request.auth_user::<G>().await?.required()?.with_mfa_required()?;

        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let recovery_codes = (0..10).map(|_| generate_recovery_code()).collect::<Vec<_>>();

        db.tx(async |tx| {
            diesel::delete(mfa_recovery_codes::table.filter(mfa_recovery_codes::user_id.eq(user_session.id())))
                .execute(tx)
                .await
                .into_tonic_internal_err("failed to delete existing recovery codes")?;

            diesel::update(users::table.filter(users::id.eq(user_session.id())))
                .set(users::mfa_recovery_codes_regenerated_at.eq(Utc::now()))
                .execute(tx)
                .await
                .into_tonic_internal_err("failed to update user recovery codes regenerated at")?;

            let mut new_recovery_codes = Vec::new();
            for code in &recovery_codes {
                let mut hasher = Argon2::default();
                let salt = SaltString::generate(&mut rand::rngs::OsRng);
                let hash = hasher
                    .hash_password(code.as_bytes(), &salt)
                    .into_tonic_internal_err("failed to hash recovery code")?;
                new_recovery_codes.push(
                    NewMfaRecoveryCode::builder()
                        .user_id(user_session.id())
                        .code_hash(hash.to_string().into())
                        .build(),
                );
            }

            diesel::insert_into(mfa_recovery_codes::table)
                .values(new_recovery_codes)
                .execute(tx)
                .await
                .into_tonic_internal_err("failed to insert recovery codes")?;

            Ok(())
        })
        .await?;

        Ok(tonic::Response::new(UserMfaRegenerateRecoveryCodesResponse {
            recovery_codes,
        }))
    }

    async fn get(&self, request: tonic::Request<UserMfaGetRequest>) -> tonic::Result<tonic::Response<UserMfaGetResponse>> {
        let global = request.global::<G>()?;
        let user_session = request.auth_user::<G>().await?.required()?.with_mfa_required()?;

        let user = global
            .user_loader()
            .load(user_session.id())
            .await
            .unwrap_or_default()
            .into_tonic_internal_err("failed to query user")?;

        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let totp_credential = mfa_totp_credentials::table
            .filter(mfa_totp_credentials::user_id.eq(user_session.id()))
            .select(MfaTotpCredential::as_select())
            .first::<MfaTotpCredential>(&mut db)
            .await
            .optional()
            .into_tonic_internal_err("failed to query TOTP credentials")?;

        let passkey_credentials = mfa_passkey_credentials::table
            .filter(mfa_passkey_credentials::user_id.eq(user_session.id()))
            .select(MfaPasskeyCredential::as_select())
            .load::<MfaPasskeyCredential>(&mut db)
            .await
            .into_tonic_internal_err("failed to query Passkey credentials")?;

        let recovery_codes = mfa_recovery_codes::table
            .filter(mfa_recovery_codes::user_id.eq(user_session.id()))
            .count()
            .get_result::<i64>(&mut db)
            .await
            .into_tonic_internal_err("failed to query recovery codes")?;

        Ok(tonic::Response::new(UserMfaGetResponse {
            mfa: Some(user::Mfa {
                totp_credentials: totp_credential.map(totp_credential_to_proto),
                passkey_credentials: passkey_credentials.into_iter().map(passkey_credential_to_proto).collect(),
                recovery_codes: Some(user::mfa::RecoveryCodes {
                    remaining: recovery_codes as u32,
                    created_at: user.mfa_recovery_codes_regenerated_at.map(|d| d.to_prost_timestamp_utc()),
                    last_used_at: user.mfa_recovery_codes_last_used_at.map(|d| d.to_prost_timestamp_utc()),
                }),
            }),
        }))
    }
}
