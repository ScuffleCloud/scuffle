use base64::Engine;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use sha2::Digest;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::{self, Action, CoreApplication};
use crate::chrono_ext::ChronoDateTimeExt;
use crate::common::{self, NewUserData, TxError};
use crate::http_ext::RequestExt;
use crate::models::{
    EmailRegistrationRequest, EmailRegistrationRequestId, MagicLinkUserSessionRequest, MagicLinkUserSessionRequestId,
    Organization, OrganizationMember, UserGoogleAccount, UserSession, UserSessionRequest, UserSessionRequestId,
    UserSessionTokenId,
};
use crate::schema::{
    email_registration_requests, magic_link_user_session_requests, organization_members, organizations, user_emails,
    user_google_accounts, user_session_requests, user_sessions,
};
use crate::services::CoreSvc;
use crate::std_ext::{OptionExt, ResultExt};
use crate::{CoreConfig, captcha, google_api, totp, webauthn};

#[async_trait::async_trait]
impl<G: CoreConfig> pb::scufflecloud::core::v1::sessions_service_server::SessionsService for CoreSvc<G> {
    async fn register_with_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::RegisterWithEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let global = &req.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;
        let payload = req.into_inner();

        // Check captcha
        let captcha = payload.captcha.require("captcha")?;
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        let email = common::normalize_email(&payload.email);

        // Generate random code
        let code = common::generate_random_bytes().into_tonic_internal_err("failed to generate registration code")?;
        // let code_base64 = base64::prelude::BASE64_URL_SAFE.encode(&code);

        db.transaction::<_, TxError, _>(move |conn| {
            async move {
                // Check if email is already registered
                if user_emails::dsl::user_emails
                    .find(&email)
                    .select(user_emails::dsl::email)
                    .first::<String>(conn)
                    .await
                    .optional()
                    .into_tonic_internal_err("failed to query database")?
                    .is_some()
                {
                    return Err(tonic::Status::with_error_details(
                        Code::AlreadyExists,
                        "email is already registered",
                        ErrorDetails::new(),
                    )
                    .into());
                }

                // Create email registration request
                let registration_request = EmailRegistrationRequest {
                    id: EmailRegistrationRequestId::new(),
                    user_id: None,
                    email,
                    code: code.to_vec(),
                    expires_at: chrono::Utc::now() + global.email_registration_request_timeout(),
                };

                diesel::insert_into(email_registration_requests::dsl::email_registration_requests)
                    .values(registration_request)
                    .execute(conn)
                    .await
                    .into_tonic_internal_err("failed to insert email registration request")?;

                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        // TODO: Send email with registration code link code_base64

        Ok(tonic::Response::new(()))
    }

    async fn complete_register_with_email(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteRegisterWithEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let global = &req.global::<G>()?;
        let ip_info = req.ip_address_info()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let device = payload.device.require("device")?;

        let new_token = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    // Delete email registration request
                    let Some(registration_request) =
                        diesel::delete(email_registration_requests::dsl::email_registration_requests)
                            .filter(
                                email_registration_requests::dsl::code
                                    .eq(&payload.code)
                                    .and(email_registration_requests::dsl::user_id.is_null())
                                    .and(email_registration_requests::dsl::expires_at.gt(chrono::Utc::now())),
                            )
                            .returning(EmailRegistrationRequest::as_select())
                            .get_result::<EmailRegistrationRequest>(conn)
                            .await
                            .optional()
                            .into_tonic_internal_err("failed to delete email registration request")?
                    else {
                        return Err(
                            tonic::Status::with_error_details(Code::NotFound, "unknown code", ErrorDetails::new()).into(),
                        );
                    };

                    let (user, new_token) = common::create_new_user_and_session(
                        global,
                        conn,
                        NewUserData {
                            email: Some(registration_request.email.clone()),
                            preferred_name: None,
                            first_name: None,
                            last_name: None,
                        },
                        device,
                        &ip_info,
                    )
                    .await?;

                    // Check if this transaction was allowed
                    cedar::is_authorized(global, None, &user, Action::RegisterWithEmailPassword, CoreApplication)?;

                    Ok(new_token)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_token))
    }

    async fn login_with_email_options(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailOptionsRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithEmailOptionsResponse>, tonic::Status> {
        let global = &req.global::<G>()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        // Check captcha
        let captcha = payload.captcha.require("captcha")?;
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        let user = common::get_user_by_email(&mut db, &payload.email).await?;

        let mut options = vec![];

        if user.password_hash.is_some()
            && cedar::is_authorized(global, None, &user, Action::LoginWithEmailPassword, CoreApplication).is_ok()
        {
            options.push(pb::scufflecloud::core::v1::LoginWithEmailOptions::Password as i32);
        }

        if cedar::is_authorized(global, None, &user, Action::LoginWithMagicLink, CoreApplication).is_ok() {
            options.push(pb::scufflecloud::core::v1::LoginWithEmailOptions::MagicLink as i32);
        }

        Ok(tonic::Response::new(
            pb::scufflecloud::core::v1::LoginWithEmailOptionsResponse { options },
        ))
    }

    async fn login_with_email_and_password(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithEmailAndPasswordRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let global = &req.global::<G>()?;
        let ip_info = req.ip_address_info()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let device = payload.device.require("device")?;
        let captcha = payload.captcha.require("captcha")?;

        // Check captcha
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        let new_token = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    let user = common::get_user_by_email(conn, &payload.email).await?;

                    cedar::is_authorized(global, None, &user, Action::LoginWithEmailPassword, CoreApplication)?;

                    // Verify password
                    let Some(password_hash) = &user.password_hash else {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::FailedPrecondition,
                            "user does not have a password set",
                            ErrorDetails::new(),
                        )
                        .into());
                    };

                    common::verify_password(password_hash, &payload.password)?;

                    Ok(common::create_session(global, conn, user.id, device, &ip_info, true).await?)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_token))
    }

    async fn login_with_magic_link(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithMagicLinkRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let global = &req.global::<G>()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let captcha = payload.captcha.require("captcha")?;

        // Check captcha
        match captcha.provider() {
            pb::scufflecloud::core::v1::CaptchaProvider::Turnstile => {
                captcha::turnstile::verify_in_tonic(global, &captcha.token).await?;
            }
        }

        db.transaction::<_, TxError, _>(|conn| {
            async move {
                let user = common::get_user_by_email(conn, &payload.email).await?;

                cedar::is_authorized(global, None, &user, Action::RequestMagicLink, CoreApplication)?;

                let code = common::generate_random_bytes().into_tonic_internal_err("failed to generate magic link code")?;

                // Insert email link user session request
                let session_request = MagicLinkUserSessionRequest {
                    id: MagicLinkUserSessionRequestId::new(),
                    user_id: user.id,
                    code: code.to_vec(),
                    expires_at: chrono::Utc::now() + global.magic_link_user_session_request_timeout(),
                };
                diesel::insert_into(magic_link_user_session_requests::dsl::magic_link_user_session_requests)
                    .values(session_request)
                    .execute(conn)
                    .await
                    .into_tonic_internal_err("failed to insert magic link user session request")?;

                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        // TODO: Send email with code

        Ok(tonic::Response::new(()))
    }

    async fn complete_login_with_magic_link(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithMagicLinkRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let global = &req.global::<G>()?;
        let ip_info = req.ip_address_info()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let device = payload.device.require("device")?;

        let new_token = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    // Find and delete magic link user session request
                    let Some(session_request) =
                        diesel::delete(magic_link_user_session_requests::dsl::magic_link_user_session_requests)
                            .filter(
                                magic_link_user_session_requests::dsl::code
                                    .eq(&payload.code)
                                    .and(magic_link_user_session_requests::dsl::expires_at.gt(chrono::Utc::now())),
                            )
                            .returning(MagicLinkUserSessionRequest::as_select())
                            .get_result::<MagicLinkUserSessionRequest>(conn)
                            .await
                            .optional()
                            .into_tonic_internal_err("failed to delete magic link user session request")?
                    else {
                        return Err(
                            tonic::Status::with_error_details(Code::NotFound, "unknown code", ErrorDetails::new()).into(),
                        );
                    };

                    cedar::is_authorized(
                        global,
                        None,
                        session_request.user_id,
                        Action::LoginWithMagicLink,
                        CoreApplication,
                    )?;

                    // Create a new session for the user
                    let new_token =
                        common::create_session(global, conn, session_request.user_id, device, &ip_info, true).await?;

                    Ok(new_token)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_token))
    }

    async fn login_with_google(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithGoogleRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::LoginWithGoogleResponse>, tonic::Status> {
        let global = &req.global::<G>()?;
        let payload = req.into_inner();

        let device = payload.device.require("device")?;
        let device_fingerprint = sha2::Sha256::digest(&device.public_key_data);
        let state = base64::prelude::BASE64_URL_SAFE.encode(device_fingerprint);

        let authorization_url = google_api::authorization_url(global, &state);

        Ok(tonic::Response::new(pb::scufflecloud::core::v1::LoginWithGoogleResponse {
            authorization_url,
        }))
    }

    async fn complete_login_with_google(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteLoginWithGoogleRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::CompleteLoginWithGoogleResponse>, tonic::Status> {
        let global = &req.global::<G>()?;
        let ip_info = req.ip_address_info()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let device = payload.device.require("device")?;
        let device_fingerprint = sha2::Sha256::digest(&device.public_key_data);
        let state = base64::prelude::BASE64_URL_SAFE
            .decode(&payload.state)
            .into_tonic_internal_err("failed to decode state")?;

        if *device_fingerprint != state {
            return Err(tonic::Status::with_error_details(
                tonic::Code::FailedPrecondition,
                "device fingerprint does not match state",
                ErrorDetails::new(),
            ));
        }

        let google_token = google_api::request_tokens(global, &payload.code)
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

        let response = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    let mut google_workspace = None;

                    // Update the organization if the user is an admin of a Google Workspace
                    if let Some((workspace_user, hd)) = workspace_user {
                        if workspace_user.is_admin {
                            let n = diesel::update(organizations::dsl::organizations)
                                .filter(organizations::dsl::google_customer_id.eq(&workspace_user.customer_id))
                                .set(organizations::dsl::google_hosted_domain.eq(&google_token.id_token.hd))
                                .execute(conn)
                                .await
                                .into_tonic_internal_err("failed to update organization")?;

                            if n == 0 {
                                google_workspace = Some(pb::scufflecloud::core::v1::complete_login_with_google_response::GoogleWorkspace::UnassociatedGoogleHostedDomain(hd));
                            }
                        }
                    }

                    let google_account = user_google_accounts::dsl::user_google_accounts
                        .find(&google_token.id_token.sub)
                        .first::<UserGoogleAccount>(conn)
                        .await
                        .optional()?;

                    match google_account {
                        Some(google_account) => {
                            cedar::is_authorized(global, None, google_account.user_id, Action::LoginWithGoogle, CoreApplication)?;

                            // Create a new session for the user
                            let new_token =
                                common::create_session(global, conn, google_account.user_id, device, &ip_info, false).await?;
                            Ok(pb::scufflecloud::core::v1::CompleteLoginWithGoogleResponse {
                                new_user_session_token: Some(new_token),
                                first_login: false,
                                google_workspace,
                            })
                        }
                        None => {
                            let (user, new_token) = common::create_new_user_and_session(
                                global,
                                conn,
                                google_token.id_token.clone().into(),
                                device,
                                &ip_info,
                            )
                            .await?;

                            cedar::is_authorized(global, None, &user, Action::RegisterWithGoogle, CoreApplication)?;

                            let google_account = UserGoogleAccount {
                                sub: google_token.id_token.sub,
                                access_token: google_token.access_token,
                                access_token_expires_at: chrono::Utc::now()
                                    + chrono::Duration::seconds(google_token.expires_in as i64),
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

                                    google_workspace = Some(
                                        pb::scufflecloud::core::v1::complete_login_with_google_response::GoogleWorkspace::Joined(org.into())
                                    );
                                }
                            }

                            Ok(pb::scufflecloud::core::v1::CompleteLoginWithGoogleResponse {
                                new_user_session_token: Some(new_token),
                                first_login: true,
                                google_workspace,
                            })
                        }
                    }
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(response))
    }

    async fn login_with_webauthn_public_key(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::LoginWithWebauthnPublicKeyRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let global = &req.global::<G>()?;
        let ip_info = req.ip_address_info()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let assertion_response = payload.response.require("response")?;
        let device = payload.device.require("device")?;

        let new_token = db
            .transaction::<_, TxError, _>(|conn| {
                async move {
                    let webauthn_pk =
                        webauthn::process_challenge(global, conn, &payload.credential_id, &assertion_response).await?;

                    cedar::is_authorized(global, None, webauthn_pk.user_id, Action::LoginWithWebauthn, CoreApplication)?;

                    // Create a new session for the user
                    let new_token =
                        common::create_session(global, conn, webauthn_pk.user_id, device, &ip_info, false).await?;

                    Ok(new_token)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_token))
    }

    async fn create_user_session_request(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CreateUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        let global = &req.global::<G>()?;
        let ip_info = req.ip_address_info()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let code = format!("{:06}", rand::random_range(0..=999999));

        let session_request = UserSessionRequest {
            id: UserSessionRequestId::new(),
            device_name: payload.name,
            device_ip: ip_info.to_network(),
            code,
            approved_by: None,
            expires_at: chrono::Utc::now() + global.user_session_request_timeout(),
        };

        diesel::insert_into(user_session_requests::dsl::user_session_requests)
            .values(&session_request)
            .execute(&mut db)
            .await
            .into_tonic_internal_err("failed to insert user session request")?;

        Ok(tonic::Response::new(session_request.into()))
    }

    async fn get_user_session_request(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        let global = &req.global::<G>()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let id: UserSessionRequestId = payload.id.parse().into_tonic_internal_err("failed to parse id")?;

        let Some(session_request) = user_session_requests::dsl::user_session_requests
            .find(&id)
            .filter(user_session_requests::dsl::expires_at.gt(chrono::Utc::now()))
            .select(UserSessionRequest::as_select())
            .first::<UserSessionRequest>(&mut db)
            .await
            .optional()
            .into_tonic_internal_err("failed to query user session request")?
        else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::NotFound,
                "user session request not found",
                ErrorDetails::new(),
            ));
        };

        Ok(tonic::Response::new(session_request.into()))
    }

    async fn get_user_session_request_by_code(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        let global = &req.global::<G>()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let Some(session_request) = user_session_requests::dsl::user_session_requests
            .filter(
                user_session_requests::dsl::code
                    .eq(&payload.code)
                    .and(user_session_requests::dsl::expires_at.gt(chrono::Utc::now())),
            )
            .select(UserSessionRequest::as_select())
            .first::<UserSessionRequest>(&mut db)
            .await
            .optional()
            .into_tonic_internal_err("failed to query user session request")?
        else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::NotFound,
                "user session request not found",
                ErrorDetails::new(),
            ));
        };

        Ok(tonic::Response::new(session_request.into()))
    }

    async fn approve_user_session_request_by_code(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ApproveUserSessionRequestByCodeRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSessionRequest>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let session_request = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    let Some(session_request) = diesel::update(user_session_requests::dsl::user_session_requests)
                        .filter(
                            user_session_requests::dsl::code
                                .eq(&payload.code)
                                .and(user_session_requests::dsl::approved_by.is_null())
                                .and(user_session_requests::dsl::expires_at.gt(chrono::Utc::now())),
                        )
                        .set(user_session_requests::dsl::approved_by.eq(&session.user_id))
                        .returning(UserSessionRequest::as_select())
                        .get_result::<UserSessionRequest>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to update user session request")?
                    else {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::NotFound,
                            "user session request not found",
                            ErrorDetails::new(),
                        )
                        .into());
                    };

                    session.is_authorized(global, session.user_id, Action::ApproveUserSessionRequest, &session_request)?;

                    Ok(session_request)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(session_request.into()))
    }

    async fn complete_user_session_request(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::CompleteUserSessionRequestRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let global = &req.global::<G>()?;
        let ip_info = req.ip_address_info()?;
        let payload = req.into_inner();

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let id: UserSessionRequestId = payload.id.parse().into_tonic_internal_err("failed to parse id")?;
        let device = payload.device.require("device")?;

        let new_token = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    // Delete user session request
                    let Some(session_request) = diesel::delete(user_session_requests::dsl::user_session_requests)
                        .filter(user_session_requests::dsl::id.eq(id))
                        .returning(UserSessionRequest::as_select())
                        .get_result::<UserSessionRequest>(conn)
                        .await
                        .optional()
                        .into_tonic_internal_err("failed to delete user session request")?
                    else {
                        return Err(
                            tonic::Status::with_error_details(Code::NotFound, "unknown id", ErrorDetails::new()).into(),
                        );
                    };

                    let Some(approved_by) = session_request.approved_by else {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::FailedPrecondition,
                            "user session request is not approved yet",
                            ErrorDetails::new(),
                        )
                        .into());
                    };

                    let new_token = common::create_session(global, conn, approved_by, device, &ip_info, false).await?;

                    Ok(new_token)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(new_token))
    }

    async fn validate_mfa_for_user_session(
        &self,
        req: tonic::Request<pb::scufflecloud::core::v1::ValidateMfaForUserSessionRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserSession>, tonic::Status> {
        let global = &req.global::<G>()?;
        let (_, ext, payload) = req.into_parts();
        let session = ext.session_or_err()?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let session = db
            .transaction::<_, TxError, _>(move |conn| {
                async move {
                    // Verify MFA challenge response
                    match payload.response.require("response")? {
                        pb::scufflecloud::core::v1::validate_mfa_for_user_session_request::Response::Totp(
                            pb::scufflecloud::core::v1::ValidateMfaForUserSessionTotp { code },
                        ) => {
                            totp::process_token(conn, session.user_id, &code).await?;
                        }
                        pb::scufflecloud::core::v1::validate_mfa_for_user_session_request::Response::Webauthn(
                            pb::scufflecloud::core::v1::ValidateMfaForUserSessionWebauthn { credential_id, response },
                        ) => {
                            let assertion_response = response.require("response.response")?;
                            webauthn::process_challenge(global, conn, &credential_id, &assertion_response).await?;
                        }
                    }

                    // Set mfa_pending=false and reset session expiry
                    let session = diesel::update(user_sessions::dsl::user_sessions)
                        .filter(
                            user_sessions::dsl::user_id
                                .eq(&session.user_id)
                                .and(user_sessions::dsl::device_fingerprint.eq(&session.device_fingerprint)),
                        )
                        .set((
                            user_sessions::dsl::mfa_pending.eq(false),
                            user_sessions::dsl::expires_at.eq(chrono::Utc::now() + global.user_session_timeout()),
                        ))
                        .returning(UserSession::as_select())
                        .get_result::<UserSession>(conn)
                        .await
                        .into_tonic_internal_err("failed to update user session")?;

                    Ok(session)
                }
                .scope_boxed()
            })
            .await?;

        Ok(tonic::Response::new(session.into()))
    }

    async fn refresh_user_session(
        &self,
        req: tonic::Request<()>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::NewUserSessionToken>, tonic::Status> {
        let global = &req.global::<G>()?;
        let session = req.session_or_err()?;
        session.is_authorized(global, session.user_id, Action::RefreshUserSession, session)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let token_id = UserSessionTokenId::new();
        let token = common::generate_random_bytes().into_tonic_internal_err("failed to generate token")?;
        let encrypted_token = common::encrypt_token(session.device_algorithm.into(), &token, &session.device_pk_data)?;

        let session = diesel::update(user_sessions::dsl::user_sessions)
            .filter(
                user_sessions::dsl::user_id
                    .eq(&session.user_id)
                    .and(user_sessions::dsl::device_fingerprint.eq(&session.device_fingerprint)),
            )
            .set((
                user_sessions::dsl::token_id.eq(token_id),
                user_sessions::dsl::token.eq(token),
                user_sessions::dsl::token_expires_at.eq(chrono::Utc::now() + global.user_session_token_timeout()),
            ))
            .returning(UserSession::as_select())
            .get_result::<UserSession>(&mut db)
            .await
            .into_tonic_internal_err("failed to update user session")?;

        let (Some(token_id), Some(token_expires_at)) = (session.token_id, session.token_expires_at) else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::Internal,
                "user session does not have a token",
                ErrorDetails::new(),
            ));
        };

        let new_token = pb::scufflecloud::core::v1::NewUserSessionToken {
            id: token_id.to_string(),
            encrypted_token,
            expires_at: Some(token_expires_at.to_prost_timestamp_utc()),
            session_mfa_pending: session.mfa_pending,
        };

        Ok(tonic::Response::new(new_token))
    }

    async fn invalidate_user_session(&self, req: tonic::Request<()>) -> Result<tonic::Response<()>, tonic::Status> {
        let global = &req.global::<G>()?;
        let session = req.session_or_err()?;
        session.is_authorized(global, session.user_id, Action::InvalidateUserSession, session)?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        diesel::delete(user_sessions::dsl::user_sessions)
            .filter(
                user_sessions::dsl::user_id
                    .eq(&session.user_id)
                    .and(user_sessions::dsl::device_fingerprint.eq(&session.device_fingerprint)),
            )
            .execute(&mut db)
            .await
            .into_tonic_internal_err("failed to update user session")?;

        Ok(tonic::Response::new(()))
    }
}
