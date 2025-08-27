use base64::Engine;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use sha2::Digest;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::{self, Action, CoreApplication, Unauthenticated};
use crate::common::normalize_email;
use crate::http_ext::RequestExt;
use crate::models::{
    MagicLinkUserSessionRequest, MagicLinkUserSessionRequestId, Organization, OrganizationMember, User, UserGoogleAccount,
    UserId,
};
use crate::operations::Operation;
use crate::schema::{magic_link_user_session_requests, organization_members, organizations, user_google_accounts, users};
use crate::std_ext::{OptionExt, ResultExt};
use crate::{CoreConfig, captcha, common, emails, google_api};

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailOptionsRequest> {
    type Principal = Unauthenticated;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::LoginWithEmailOptionsResponse;

    const ACTION: Action = Action::GetLoginWithEmailOptions;
    const TRANSACTION: bool = false;

    async fn validate(&mut self) -> Result<(), tonic::Status> {
        let global = &self.global::<G>()?;
        let captcha = self.get_ref().captcha.clone().require("captcha")?;

        // Check captcha
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        Ok(())
    }

    async fn load_principal(
        &mut self,
        _conn: &mut diesel_async::AsyncPgConnection,
    ) -> Result<Self::Principal, tonic::Status> {
        Ok(Unauthenticated)
    }

    async fn load_resource(&mut self, _conn: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        conn: &mut diesel_async::AsyncPgConnection,
        _principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let payload = self.into_inner();
        let user = common::get_user_by_email(conn, &payload.email).await?;

        let mut options = vec![];

        if user.password_hash.is_some()
            && cedar::is_authorized(global, None, &user, Action::LoginWithEmailPassword, CoreApplication)
                .await
                .is_ok()
        {
            options.push(pb::scufflecloud::core::v1::LoginWithEmailOptions::Password as i32);
        }

        if cedar::is_authorized(global, None, &user, Action::LoginWithMagicLink, CoreApplication)
            .await
            .is_ok()
        {
            options.push(pb::scufflecloud::core::v1::LoginWithEmailOptions::MagicLink as i32);
        }

        Ok(pb::scufflecloud::core::v1::LoginWithEmailOptionsResponse { options })
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailAndPasswordRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::LoginWithEmailPassword;

    async fn validate(&mut self) -> Result<(), tonic::Status> {
        let global = &self.global::<G>()?;
        let captcha = self.get_ref().captcha.clone().require("captcha")?;

        // Check captcha
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        Ok(())
    }

    async fn load_principal(&mut self, tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Principal, tonic::Status> {
        common::get_user_by_email(tx, &self.get_ref().email).await
    }

    async fn load_resource(&mut self, _tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        tx: &mut diesel_async::AsyncPgConnection,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let ip_info = self.ip_address_info()?;
        let payload = self.into_inner();

        let device = payload.device.require("device")?;

        // Verify password
        let Some(password_hash) = &principal.password_hash else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::FailedPrecondition,
                "user does not have a password set",
                ErrorDetails::new(),
            ));
        };

        common::verify_password(password_hash, &payload.password)?;

        common::create_session(global, tx, principal.id, device, &ip_info, true).await
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::LoginWithMagicLinkRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = ();

    const ACTION: Action = Action::RequestMagicLink;

    async fn validate(&mut self) -> Result<(), tonic::Status> {
        let global = &self.global::<G>()?;
        let captcha = self.get_ref().captcha.clone().require("captcha")?;

        // Check captcha
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        Ok(())
    }

    async fn load_principal(&mut self, tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Principal, tonic::Status> {
        let user = common::get_user_by_email(tx, &self.get_ref().email).await?;
        Ok(user)
    }

    async fn load_resource(&mut self, _tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        tx: &mut diesel_async::AsyncPgConnection,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;

        let to_address = principal.primary_email.into_tonic_err(
            Code::FailedPrecondition,
            "user does not have a primary email address",
            ErrorDetails::new(),
        )?;

        let code = common::generate_random_bytes().into_tonic_internal_err("failed to generate magic link code")?;
        let code_base64 = base64::prelude::BASE64_URL_SAFE.encode(code);

        // Insert email link user session request
        let session_request = MagicLinkUserSessionRequest {
            id: MagicLinkUserSessionRequestId::new(),
            user_id: principal.id,
            code: code.to_vec(),
            expires_at: chrono::Utc::now() + global.magic_link_user_session_request_timeout(),
        };
        diesel::insert_into(magic_link_user_session_requests::dsl::magic_link_user_session_requests)
            .values(session_request)
            .execute(tx)
            .await
            .into_tonic_internal_err("failed to insert magic link user session request")?;

        let email = emails::magic_link_email(global, to_address, code_base64)
            .await
            .into_tonic_internal_err("failed to render magic link email")?;
        global
            .email_service()
            .send_email(email)
            .await
            .into_tonic_internal_err("failed to send magic link email")?;

        Ok(())
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithMagicLinkRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::LoginWithMagicLink;

    async fn load_principal(&mut self, tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Principal, tonic::Status> {
        // Find and delete magic link user session request
        let Some(session_request) = diesel::delete(magic_link_user_session_requests::dsl::magic_link_user_session_requests)
            .filter(
                magic_link_user_session_requests::dsl::code
                    .eq(&self.get_ref().code)
                    .and(magic_link_user_session_requests::dsl::expires_at.gt(chrono::Utc::now())),
            )
            .returning(MagicLinkUserSessionRequest::as_select())
            .get_result::<MagicLinkUserSessionRequest>(tx)
            .await
            .optional()
            .into_tonic_internal_err("failed to delete magic link user session request")?
        else {
            return Err(tonic::Status::with_error_details(
                Code::NotFound,
                "unknown code",
                ErrorDetails::new(),
            ));
        };

        // Load user
        let user = users::dsl::users
            .find(session_request.user_id)
            .first::<User>(tx)
            .await
            .into_tonic_internal_err("failed to query user")?;

        Ok(user)
    }

    async fn load_resource(&mut self, _tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        tx: &mut diesel_async::AsyncPgConnection,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let ip_info = self.ip_address_info()?;
        let device = self.into_inner().device.require("device")?;

        let new_token = common::create_session(global, tx, principal.id, device, &ip_info, true).await?;
        Ok(new_token)
    }
}

#[derive(Clone, Default)]
struct CompleteLoginWithGoogleState {
    first_login: bool,
    google_workspace: Option<pb::scufflecloud::core::v1::complete_login_with_google_response::GoogleWorkspace>,
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithGoogleRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::CompleteLoginWithGoogleResponse;

    const ACTION: Action = Action::LoginWithGoogle;

    async fn validate(&mut self) -> Result<(), tonic::Status> {
        let device = self.get_ref().device.clone().require("device")?;
        let device_fingerprint = sha2::Sha256::digest(&device.public_key_data);
        let state = base64::prelude::BASE64_URL_SAFE
            .decode(&self.get_ref().state)
            .into_tonic_internal_err("failed to decode state")?;

        if *device_fingerprint != state {
            return Err(tonic::Status::with_error_details(
                tonic::Code::FailedPrecondition,
                "device fingerprint does not match state",
                ErrorDetails::new(),
            ));
        }

        Ok(())
    }

    async fn load_principal(&mut self, tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;

        let google_token = google_api::request_tokens(global, &self.get_ref().code)
            .await
            .into_tonic_err_with_field_violation("code", "failed to request google token")?;

        // If user is part of a Google Workspace
        let workspace_user = if google_token.scope.contains(google_api::ADMIN_DIRECTORY_API_USER_SCOPE) {
            if let Some(hd) = google_token.id_token.hd.clone() {
                google_api::request_google_workspace_user(global, &google_token.access_token, &google_token.id_token.sub)
                    .await
                    .into_tonic_internal_err("failed to request Google Workspace user")?
                    .map(|u| (u, hd))
            } else {
                None
            }
        } else {
            None
        };

        let mut state = CompleteLoginWithGoogleState {
            first_login: false,
            google_workspace: None,
        };

        // Update the organization if the user is an admin of a Google Workspace
        if let Some((workspace_user, hd)) = workspace_user
            && workspace_user.is_admin
        {
            let n = diesel::update(organizations::dsl::organizations)
                .filter(organizations::dsl::google_customer_id.eq(&workspace_user.customer_id))
                .set(organizations::dsl::google_hosted_domain.eq(&google_token.id_token.hd))
                .execute(tx)
                .await
                .into_tonic_internal_err("failed to update organization")?;

            if n == 0 {
                state.google_workspace = Some(pb::scufflecloud::core::v1::complete_login_with_google_response::GoogleWorkspace::UnassociatedGoogleHostedDomain(hd));
            }
        }

        let google_account = user_google_accounts::dsl::user_google_accounts
            .find(&google_token.id_token.sub)
            .first::<UserGoogleAccount>(tx)
            .await
            .optional()
            .into_tonic_internal_err("failed to query google account")?;

        match google_account {
            Some(google_account) => {
                // Load existing user
                let user = users::dsl::users
                    .find(google_account.user_id)
                    .first::<User>(tx)
                    .await
                    .into_tonic_internal_err("failed to query user")?;

                self.extensions_mut().insert(state);

                Ok(user)
            }
            None => {
                let user = User {
                    id: UserId::new(),
                    preferred_name: google_token.id_token.name,
                    first_name: google_token.id_token.given_name,
                    last_name: google_token.id_token.family_name,
                    password_hash: None,
                    primary_email: google_token
                        .id_token
                        .email_verified
                        .then(|| normalize_email(&google_token.id_token.email)),
                };

                common::create_user(tx, &user).await?;

                let google_account = UserGoogleAccount {
                    sub: google_token.id_token.sub,
                    access_token: google_token.access_token,
                    access_token_expires_at: chrono::Utc::now() + chrono::Duration::seconds(google_token.expires_in as i64),
                    user_id: user.id,
                    created_at: chrono::Utc::now(),
                };

                diesel::insert_into(user_google_accounts::dsl::user_google_accounts)
                    .values(google_account)
                    .execute(tx)
                    .await
                    .into_tonic_internal_err("failed to insert user google account")?;

                if let Some(hd) = google_token.id_token.hd {
                    // Check if the organization exists for the hosted domain
                    let organization = organizations::dsl::organizations
                        .filter(organizations::dsl::google_hosted_domain.eq(hd))
                        .first::<Organization>(tx)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to query organization")?;

                    if let Some(org) = organization {
                        // Associate user with the organization
                        let membership = OrganizationMember {
                            organization_id: org.id,
                            user_id: user.id,
                            invited_by_id: None,
                            inline_policy: None,
                            created_at: chrono::Utc::now(),
                        };

                        diesel::insert_into(organization_members::dsl::organization_members)
                            .values(membership)
                            .execute(tx)
                            .await
                            .into_tonic_internal_err("failed to insert organization membership")?;

                        state.google_workspace = Some(
                            pb::scufflecloud::core::v1::complete_login_with_google_response::GoogleWorkspace::Joined(
                                org.into(),
                            ),
                        );
                    }
                }

                state.first_login = true;
                self.extensions_mut().insert(state);

                Ok(user)
            }
        }
    }

    async fn load_resource(&mut self, _tx: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        mut self,
        tx: &mut diesel_async::AsyncPgConnection,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let ip_info = self.ip_address_info()?;

        let state = self
            .extensions_mut()
            .remove::<CompleteLoginWithGoogleState>()
            .into_tonic_internal_err("missing CompleteLoginWithGoogleState state")?;

        let device = self.into_inner().device.require("device")?;

        // Create session
        let token = common::create_session(global, tx, principal.id, device, &ip_info, false).await?;

        Ok(pb::scufflecloud::core::v1::CompleteLoginWithGoogleResponse {
            new_user_session_token: Some(token),
            first_login: state.first_login,
            google_workspace: state.google_workspace,
        })
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::LoginWithWebauthnRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::LoginWithWebauthn;

    async fn load_principal(
        &mut self,
        _conn: &mut diesel_async::AsyncPgConnection,
    ) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let user_id: UserId = self
            .get_ref()
            .user_id
            .parse()
            .into_tonic_err_with_field_violation("user_id", "invalid ID")?;

        common::get_user_by_id(global, user_id).await
    }

    async fn load_resource(&mut self, _conn: &mut diesel_async::AsyncPgConnection) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        conn: &mut diesel_async::AsyncPgConnection,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let ip_info = self.ip_address_info()?;
        let payload = self.into_inner();

        let pk_cred: webauthn_rs::prelude::PublicKeyCredential = serde_json::from_str(&payload.response_json)
            .into_tonic_err_with_field_violation("response_json", "invalid public key credential")?;
        let device = payload.device.require("device")?;

        common::finish_webauthn_authentication(global, conn, principal.id, &pk_cred).await?;

        // Create a new session for the user
        let new_token = common::create_session(global, conn, principal.id, device, &ip_info, false).await?;
        Ok(new_token)
    }
}
