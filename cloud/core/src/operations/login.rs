use base64::Engine;
use core_db_types::models::{
    MagicLinkRequest, MagicLinkRequestId, Organization, OrganizationMember, User, UserGoogleAccount, UserId,
};
use core_db_types::schema::{magic_link_requests, organization_members, organizations, user_google_accounts, users};
use core_traits::EmailServiceClient;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{OptionExt, RequestExt, ResultExt};
use geo_ip::GeoIpRequestExt;
use pb::scufflecloud::core::v1::CaptchaProvider;
use sha2::Digest;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::{Action, CoreApplication, Unauthenticated};
use crate::common::normalize_email;
use crate::http_ext::CoreRequestExt;
use crate::operations::{Operation, OperationDriver};
use crate::{captcha, common, google_api};

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::LoginWithMagicLinkRequest> {
    type Principal = Unauthenticated;
    type Resource = CoreApplication;
    type Response = ();

    const ACTION: Action = Action::RequestMagicLink;

    async fn validate(&mut self) -> Result<(), tonic::Status> {
        let global = &self.global::<G>()?;
        let captcha = self.get_ref().captcha.clone().require("captcha")?;

        // Check captcha
        match captcha.provider() {
            CaptchaProvider::Unspecified => {
                return Err(tonic::Status::with_error_details(
                    Code::InvalidArgument,
                    "captcha provider must be set",
                    ErrorDetails::new(),
                ));
            }
            CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, self.ip_address_info()?.ip_address, &captcha.token).await?;
            }
        }

        Ok(())
    }

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        Ok(Unauthenticated)
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        _principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;

        let email = normalize_email(&self.get_ref().email);

        let conn = driver.conn().await?;

        let user = common::get_user_by_email(conn, &email).await?;
        let user_id = user.as_ref().map(|u| u.id);

        let code = common::generate_random_bytes().into_tonic_internal_err("failed to generate magic link code")?;
        let code_base64 = base64::prelude::BASE64_URL_SAFE.encode(code);

        let timeout = global.timeout_config().magic_link_request;

        // Insert email link user session request
        let session_request = MagicLinkRequest {
            id: MagicLinkRequestId::new(),
            user_id,
            email: email.clone(),
            code: code.to_vec(),
            expires_at: chrono::Utc::now() + timeout,
        };
        diesel::insert_into(magic_link_requests::dsl::magic_link_requests)
            .values(session_request)
            .execute(conn)
            .await
            .into_tonic_internal_err("failed to insert magic link user session request")?;

        // Send email
        let email_msg = if user_id.is_none() {
            core_emails::register_with_email_email(&self.dashboard_origin::<G>()?, code_base64, timeout)
                .into_tonic_internal_err("failed to render registration email")?
        } else {
            core_emails::magic_link_email(&self.dashboard_origin::<G>()?, code_base64, timeout)
                .into_tonic_internal_err("failed to render magic link email")?
        };

        let email_msg = common::email_to_pb(global, email, user.and_then(|u| u.preferred_name), email_msg);

        global
            .email_service()
            .send_email(email_msg)
            .await
            .into_tonic_internal_err("failed to send magic link email")?;

        Ok(())
    }
}

#[derive(Clone)]
struct CompleteLoginWithMagicLinkState {
    create_user: bool,
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithMagicLinkRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::LoginWithMagicLink;

    async fn load_principal(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let conn = driver.conn().await?;

        // Find and delete magic link request
        let Some(magic_link_request) = diesel::delete(magic_link_requests::dsl::magic_link_requests)
            .filter(
                magic_link_requests::dsl::code
                    .eq(&self.get_ref().code)
                    .and(magic_link_requests::dsl::expires_at.gt(chrono::Utc::now())),
            )
            .returning(MagicLinkRequest::as_select())
            .get_result::<MagicLinkRequest>(conn)
            .await
            .optional()
            .into_tonic_internal_err("failed to delete magic link request")?
        else {
            return Err(tonic::Status::with_error_details(
                Code::NotFound,
                "unknown code",
                ErrorDetails::new(),
            ));
        };

        let mut state = CompleteLoginWithMagicLinkState { create_user: false };

        // Load user
        let user = if let Some(user_id) = magic_link_request.user_id {
            users::dsl::users
                .find(user_id)
                .first::<User>(conn)
                .await
                .into_tonic_internal_err("failed to query user")?
        } else {
            state.create_user = true;

            let hash = sha2::Sha256::digest(&magic_link_request.email);
            let avatar_url = format!("https://gravatar.com/avatar/{:x}?s=80&d=identicon", hash);

            User {
                id: UserId::new(),
                preferred_name: None,
                first_name: None,
                last_name: None,
                password_hash: None,
                primary_email: Some(magic_link_request.email),
                avatar_url: Some(avatar_url),
            }
        };

        self.extensions_mut().insert(state);

        Ok(user)
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        mut self,
        driver: &mut OperationDriver<'_, G>,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let metadata = common::CreateSessionMetadata::from_req::<G, _>(&self)?;
        let state: CompleteLoginWithMagicLinkState = self
            .extensions_mut()
            .remove()
            .into_tonic_internal_err("missing CompleteLoginWithMagicLinkState state")?;

        let device = self.into_inner().device.require("device")?;

        let conn = driver.conn().await?;

        if state.create_user {
            common::create_user(conn, &principal).await?;
        }

        let new_token = common::create_session(global, conn, &principal, device, metadata, !state.create_user).await?;
        Ok(new_token)
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailAndPasswordRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::LoginWithEmailPassword;

    async fn validate(&mut self) -> Result<(), tonic::Status> {
        let global = &self.global::<G>()?;
        let captcha = self.get_ref().captcha.clone().require("captcha")?;

        // Check captcha
        match captcha.provider() {
            CaptchaProvider::Unspecified => {
                return Err(tonic::Status::with_error_details(
                    Code::InvalidArgument,
                    "captcha provider must be set",
                    ErrorDetails::new(),
                ));
            }
            CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, self.ip_address_info()?.ip_address, &captcha.token).await?;
            }
        }

        Ok(())
    }

    async fn load_principal(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let conn = driver.conn().await?;
        let Some(user) = common::get_user_by_email(conn, &self.get_ref().email).await? else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::NotFound,
                "user not found",
                ErrorDetails::new(),
            ));
        };

        Ok(user)
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let metadata = common::CreateSessionMetadata::from_req::<G, _>(&self)?;
        let payload = self.into_inner();

        let conn = driver.conn().await?;

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

        common::create_session(global, conn, &principal, device, metadata, true).await
    }
}

#[derive(Clone, Default)]
struct CompleteLoginWithGoogleState {
    first_login: bool,
    google_workspace: Option<pb::scufflecloud::core::v1::complete_login_with_google_response::GoogleWorkspace>,
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithGoogleRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::CompleteLoginWithGoogleResponse;

    const ACTION: Action = Action::LoginWithGoogle;

    async fn validate(&mut self) -> Result<(), tonic::Status> {
        let device = self.get_ref().device.clone().require("device")?;
        let device_fingerprint = sha2::Sha256::digest(&device.public_key_data);
        let state = urlencoding::decode(&self.get_ref().state).into_tonic_internal_err("failed to decode state")?;
        let state = base64::prelude::BASE64_URL_SAFE
            .decode(state.as_ref())
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

    async fn load_principal(&mut self, driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;

        let google_token = google_api::request_tokens(global, &self.dashboard_origin::<G>()?, &self.get_ref().code)
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

        let conn = driver.conn().await?;

        // Update the organization if the user is an admin of a Google Workspace
        if let Some((workspace_user, hd)) = workspace_user
            && workspace_user.is_admin
        {
            let n = diesel::update(organizations::dsl::organizations)
                .filter(organizations::dsl::google_customer_id.eq(&workspace_user.customer_id))
                .set(organizations::dsl::google_hosted_domain.eq(&google_token.id_token.hd))
                .execute(conn)
                .await
                .into_tonic_internal_err("failed to update organization")?;

            if n == 0 {
                state.google_workspace = Some(pb::scufflecloud::core::v1::complete_login_with_google_response::GoogleWorkspace::UnassociatedGoogleHostedDomain(hd));
            }
        }

        let google_account = user_google_accounts::dsl::user_google_accounts
            .find(&google_token.id_token.sub)
            .first::<UserGoogleAccount>(conn)
            .await
            .optional()
            .into_tonic_internal_err("failed to query google account")?;

        match google_account {
            Some(google_account) => {
                // Load existing user
                let user = diesel::update(users::dsl::users)
                    .filter(users::dsl::id.eq(google_account.user_id))
                    .set(users::dsl::avatar_url.eq(google_token.id_token.picture))
                    .returning(User::as_select())
                    .get_result::<User>(conn)
                    .await
                    .into_tonic_internal_err("failed to update user")?;

                self.extensions_mut().insert(state);

                Ok(user)
            }
            None => {
                // This Google account is not associated with a Scuffle user yet

                // Check if email is already registered
                let registered_user = if google_token.id_token.email_verified {
                    common::get_user_by_email(conn, &google_token.id_token.email).await?
                } else {
                    None
                };

                let user = match registered_user {
                    Some(user) => user, // Use existing user
                    None => {
                        // Create new user
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
                            avatar_url: google_token.id_token.picture,
                        };

                        common::create_user(conn, &user).await?;

                        user
                    }
                };

                let google_account = UserGoogleAccount {
                    sub: google_token.id_token.sub,
                    access_token: google_token.access_token,
                    access_token_expires_at: chrono::Utc::now() + chrono::Duration::seconds(google_token.expires_in as i64),
                    user_id: user.id,
                    created_at: chrono::Utc::now(),
                };

                diesel::insert_into(user_google_accounts::dsl::user_google_accounts)
                    .values(google_account)
                    .execute(conn)
                    .await
                    .into_tonic_internal_err("failed to insert user google account")?;

                if let Some(hd) = google_token.id_token.hd {
                    // Check if the organization exists for the hosted domain
                    let organization = organizations::dsl::organizations
                        .filter(organizations::dsl::google_hosted_domain.eq(hd))
                        .first::<Organization>(conn)
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
                            .execute(conn)
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

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        mut self,
        driver: &mut OperationDriver<'_, G>,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let metadata = common::CreateSessionMetadata::from_req::<G, _>(&self)?;

        let state = self
            .extensions_mut()
            .remove::<CompleteLoginWithGoogleState>()
            .into_tonic_internal_err("missing CompleteLoginWithGoogleState state")?;

        let device = self.into_inner().device.require("device")?;

        let conn = driver.conn().await?;

        // Create session
        let token = common::create_session(global, conn, &principal, device, metadata, false).await?;

        Ok(pb::scufflecloud::core::v1::CompleteLoginWithGoogleResponse {
            new_user_session_token: Some(token),
            first_login: state.first_login,
            google_workspace: state.google_workspace,
        })
    }
}

impl<G: core_traits::Global> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::LoginWithWebauthnRequest> {
    type Principal = User;
    type Resource = CoreApplication;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::LoginWithWebauthn;

    async fn load_principal(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let user_id: UserId = self
            .get_ref()
            .user_id
            .parse()
            .into_tonic_err_with_field_violation("user_id", "invalid ID")?;

        common::get_user_by_id(global, user_id).await
    }

    async fn load_resource(&mut self, _driver: &mut OperationDriver<'_, G>) -> Result<Self::Resource, tonic::Status> {
        Ok(CoreApplication)
    }

    async fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        principal: Self::Principal,
        _resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let metadata = common::CreateSessionMetadata::from_req::<G, _>(&self)?;
        let payload = self.into_inner();

        let pk_cred: webauthn_rs::prelude::PublicKeyCredential = serde_json::from_str(&payload.response_json)
            .into_tonic_err_with_field_violation("response_json", "invalid public key credential")?;
        let device = payload.device.require("device")?;

        let conn = driver.conn().await?;

        common::finish_webauthn_authentication(global, conn, principal.id, &pk_cred).await?;

        // Create a new session for the user
        let new_token = common::create_session(global, conn, &principal, device, metadata, false).await?;
        Ok(new_token)
    }
}
