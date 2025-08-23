use diesel::{BoolExpressionMethods, ExpressionMethods, SelectableHelper};
use diesel_async::RunQueryDsl;
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::Action;
use crate::chrono_ext::ChronoDateTimeExt;
use crate::http_ext::RequestExt;
use crate::models::{User, UserSession, UserSessionTokenId};
use crate::operations::Operation;
use crate::schema::user_sessions;
use crate::std_ext::{OptionExt, ResultExt};
use crate::{CoreConfig, common, totp};

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ValidateMfaForUserSessionRequest> {
    type Principal = User;
    type Resource = UserSession;
    type Response = pb::scufflecloud::core::v1::UserSession;

    const ACTION: Action = Action::ValidateMfaForUserSession;

    async fn load_principal(
        &mut self,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> Result<Self::Principal, tonic::Status> {
        let session = self.session_or_err()?;
        common::get_user_by_id(conn, session.user_id).await
    }

    async fn load_resource(&mut self, _conn: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        let session = self.session_or_err()?;
        Ok(session.clone())
    }

    async fn execute(
        self,
        conn: &mut diesel_async::AsyncPgConnection,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let payload = self.into_inner();

        // Verify MFA challenge response
        match payload.response.require("response")? {
            pb::scufflecloud::core::v1::validate_mfa_for_user_session_request::Response::Totp(
                pb::scufflecloud::core::v1::ValidateMfaForUserSessionTotp { code },
            ) => {
                totp::process_token(conn, resource.user_id, &code).await?;
            }
            pb::scufflecloud::core::v1::validate_mfa_for_user_session_request::Response::Webauthn(
                pb::scufflecloud::core::v1::ValidateMfaForUserSessionWebauthn { response_json },
            ) => {
                let pk_cred: webauthn_rs::prelude::PublicKeyCredential = serde_json::from_str(&response_json)
                    .into_tonic_err_with_field_violation("response_json", "invalid public key credential")?;
                common::finish_webauthn_authentication(global, conn, resource.user_id, &pk_cred).await?;
            }
            pb::scufflecloud::core::v1::validate_mfa_for_user_session_request::Response::RecoveryCode(
                pb::scufflecloud::core::v1::ValidateMfaForUserSessionRecoveryCode { code },
            ) => {
                common::process_recovery_code(conn, resource.user_id, &code).await?;
            }
        }

        // Set mfa_pending=false and reset session expiry
        let session = diesel::update(user_sessions::dsl::user_sessions)
            .filter(
                user_sessions::dsl::user_id
                    .eq(&resource.user_id)
                    .and(user_sessions::dsl::device_fingerprint.eq(&resource.device_fingerprint)),
            )
            .set((
                user_sessions::dsl::mfa_pending.eq(false),
                user_sessions::dsl::expires_at.eq(chrono::Utc::now() + global.user_session_timeout()),
            ))
            .returning(UserSession::as_select())
            .get_result::<UserSession>(conn)
            .await
            .into_tonic_internal_err("failed to update user session")?;

        Ok(session.into())
    }
}

pub(crate) struct RefreshUserSessionRequest;

impl<G: CoreConfig> Operation<G> for tonic::Request<RefreshUserSessionRequest> {
    type Principal = User;
    type Resource = UserSession;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::RefreshUserSession;

    async fn load_principal(
        &mut self,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> Result<Self::Principal, tonic::Status> {
        let session = self.expired_session_or_err()?;
        common::get_user_by_id(conn, session.user_id).await
    }

    async fn load_resource(&mut self, _conn: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        let session = self.expired_session_or_err()?;
        Ok(session.clone())
    }

    async fn execute(
        self,
        conn: &mut diesel_async::AsyncPgConnection,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;

        let token_id = UserSessionTokenId::new();
        let token = common::generate_random_bytes().into_tonic_internal_err("failed to generate token")?;
        let encrypted_token = common::encrypt_token(resource.device_algorithm.into(), &token, &resource.device_pk_data)?;

        let session = diesel::update(user_sessions::dsl::user_sessions)
            .filter(
                user_sessions::dsl::user_id
                    .eq(&resource.user_id)
                    .and(user_sessions::dsl::device_fingerprint.eq(&resource.device_fingerprint)),
            )
            .set((
                user_sessions::dsl::token_id.eq(token_id),
                user_sessions::dsl::token.eq(token),
                user_sessions::dsl::token_expires_at.eq(chrono::Utc::now() + global.user_session_token_timeout()),
            ))
            .returning(UserSession::as_select())
            .get_result::<UserSession>(conn)
            .await
            .into_tonic_internal_err("failed to update user session")?;

        let (Some(token_id), Some(token_expires_at)) = (session.token_id, session.token_expires_at) else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::Internal,
                "user session does not have a token",
                ErrorDetails::new(),
            ));
        };

        let mfa_options = if session.mfa_pending {
            common::mfa_options(conn, session.user_id).await?
        } else {
            vec![]
        };

        let new_token = pb::scufflecloud::core::v1::NewUserSessionToken {
            id: token_id.to_string(),
            encrypted_token,
            expires_at: Some(token_expires_at.to_prost_timestamp_utc()),
            session_mfa_pending: session.mfa_pending,
            mfa_options: mfa_options.into_iter().map(|o| o as i32).collect(),
        };

        Ok(new_token)
    }
}

pub(crate) struct InvalidateUserSessionRequest;

impl<G: CoreConfig> Operation<G> for tonic::Request<InvalidateUserSessionRequest> {
    type Principal = User;
    type Resource = UserSession;
    type Response = ();

    const ACTION: Action = Action::InvalidateUserSession;
    const TRANSACTION: bool = false;

    async fn load_principal(
        &mut self,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> Result<Self::Principal, tonic::Status> {
        let session = self.session_or_err()?;
        common::get_user_by_id(conn, session.user_id).await
    }

    async fn load_resource(&mut self, _conn: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        let session = self.session_or_err()?;
        Ok(session.clone())
    }

    async fn execute(
        self,
        conn: &mut diesel_async::AsyncPgConnection,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        diesel::delete(user_sessions::dsl::user_sessions)
            .filter(
                user_sessions::dsl::user_id
                    .eq(&resource.user_id)
                    .and(user_sessions::dsl::device_fingerprint.eq(&resource.device_fingerprint)),
            )
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to delete user session")?;

        Ok(())
    }
}
