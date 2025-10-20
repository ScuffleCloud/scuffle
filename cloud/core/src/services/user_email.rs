use std::borrow::Cow;

use core_db_types::models::{NewUserEmail, PendingUserEmail, UserEmail, UserId};
use core_db_types::schema::{pending_user_emails, user_emails};
use core_pb::v1::user::{Email, email};
use core_pb::v1::user_email_service_server::UserEmailService;
use core_pb::v1::{
    UserEmailAddRequest, UserEmailAddResponse, UserEmailListRequest, UserEmailListResponse, UserEmailVerifyRequest,
    UserEmailVerifyResponse,
};
use core_traits::{DatabaseExt, EmailServiceClient};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ext_traits::{ChronoDateTimeExt, OptionExt, RequestExt, ResultExt};
use fred::interfaces::KeysInterface;
use fred::types::Expiration;
use tonic::async_trait;
use tonic_types::{ErrorDetails, StatusExt};

use crate::auth_session::{AuthSession, AuthSessionExt, AuthSessionResultExt};

const EMAIL_VERIFICATION_EXPIRY: i64 = 15 * 60; // 15 minutes
const RATE_LIMIT_WINDOW: i64 = 60; // 1 minute between emails

fn generate_verification_code() -> tonic::Result<String> {
    todo!("not implemented")
}

async fn send_verification_email<G: core_traits::Global>(global: &G, user_id: UserId, email: &str) -> tonic::Result<()> {
    let code = generate_verification_code()?;
    let redis_key = format!("email:verify:{user_id}:{email}");

    let _: () = global
        .redis()
        .set(
            &redis_key,
            &code,
            Some(Expiration::EX(EMAIL_VERIFICATION_EXPIRY)),
            None,
            false,
        )
        .await
        .into_tonic_internal_err("failed to store verification code")?;

    let email_content = core_emails::add_new_email_email(
        global.dashboard_origin(),
        code,
        std::time::Duration::from_secs(EMAIL_VERIFICATION_EXPIRY as u64),
    )
    .into_tonic_internal_err("failed to render email")?;

    global
        .email_service()
        .send_email(email_pb::v1::SendEmailRequest {
            from: Some(email_pb::v1::EmailAddress {
                name: Some(global.email_from_name().to_string()),
                address: global.email_from_address().to_string(),
            }),
            to: Some(email_pb::v1::EmailAddress {
                name: None,
                address: email.to_string(),
            }),
            subject: email_content.subject,
            text: email_content.text,
            html: email_content.html,
        })
        .await
        .into_tonic_internal_err("failed to send verification email")?;

    Ok(())
}

fn email_to_proto(email: UserEmail) -> Email {
    Email {
        address: email.email,
        status: Some(email::Status::Verified(email::Verified {
            verified_at: Some(email.created_at.to_prost_timestamp_utc()),
        })),
    }
}

fn pending_email_to_proto(pending: PendingUserEmail) -> Email {
    Email {
        address: pending.email,
        status: Some(email::Status::NotVerified(email::NotVerified {
            last_sent_code_at: Some(pending.last_sent_code_at.to_prost_timestamp_utc()),
            can_resend_code_at: Some(
                (pending.last_sent_code_at + chrono::Duration::seconds(RATE_LIMIT_WINDOW)).to_prost_timestamp_utc(),
            ),
        })),
    }
}

#[async_trait]
impl<G: core_traits::Global> UserEmailService for crate::services::CoreSvc<G> {
    async fn list(
        &self,
        request: tonic::Request<UserEmailListRequest>,
    ) -> tonic::Result<tonic::Response<UserEmailListResponse>> {
        let global = request.global::<G>()?;

        // Get authenticated user session
        let user_session = request.auth_user::<G>().await?.required()?;

        // Get user emails from database
        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let verified_emails = user_emails::table
            .filter(user_emails::user_id.eq(user_session.id()))
            .select(UserEmail::as_select())
            .load::<UserEmail>(&mut db)
            .await
            .into_tonic_internal_err("failed to query user emails")?;

        // Create a map of pending emails for fast lookup
        let pending_emails = pending_user_emails::table
            .filter(pending_user_emails::user_id.eq(user_session.id()))
            .select(PendingUserEmail::as_select())
            .load::<PendingUserEmail>(&mut db)
            .await
            .into_tonic_internal_err("failed to query pending user emails")?;

        // Add pending emails that are not verified yet
        let mut emails: Vec<_> = verified_emails
            .into_iter()
            .map(email_to_proto)
            .chain(pending_emails.into_iter().map(pending_email_to_proto))
            .collect();

        emails.sort_by(|a, b| a.address.cmp(&b.address));

        Ok(tonic::Response::new(UserEmailListResponse { emails }))
    }

    async fn add(
        &self,
        request: tonic::Request<UserEmailAddRequest>,
    ) -> tonic::Result<tonic::Response<UserEmailAddResponse>> {
        let global = request.global::<G>()?;

        // Get authenticated user session
        let auth_session = request.auth_user::<G>().await?.required()?.with_mfa_required()?;

        let req = request.into_inner();

        let email = req.email.to_lowercase().trim().to_string();
        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;
        let new_pending = db
            .tx(async |tx| {
                // Check if email is already verified
                let existing_verified = user_emails::table
                    .find(&email)
                    .first::<UserEmail>(tx)
                    .await
                    .optional()
                    .into_tonic_internal_err("failed to query user email")?;

                if let Some(existing) = existing_verified {
                    if existing.user_id == auth_session.id() {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::AlreadyExists,
                            "email has already been added to your account",
                            ErrorDetails::with_bad_request_violation(
                                "email",
                                "email has already been added to your account",
                            ),
                        ));
                    } else {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::AlreadyExists,
                            "email is already in use by another account",
                            ErrorDetails::with_bad_request_violation("email", "email is already in use by another account"),
                        ));
                    }
                }

                // Check for existing pending email and rate limiting
                let existing_pending = pending_user_emails::table
                    .filter(pending_user_emails::email.eq(&email))
                    .filter(pending_user_emails::user_id.eq(auth_session.id()))
                    .first::<PendingUserEmail>(tx)
                    .await
                    .optional()
                    .into_tonic_internal_err("failed to query pending user email")?;

                if let Some(pending) = &existing_pending {
                    let now = chrono::Utc::now();
                    let next_retry = pending.last_sent_code_at + chrono::Duration::seconds(RATE_LIMIT_WINDOW);
                    if now < next_retry && pending.codes_sent >= 3 {
                        return Err(tonic::Status::with_error_details(
                            tonic::Code::ResourceExhausted,
                            "an email code has been sent too recently, please wait a moment and try again",
                            ErrorDetails::with_retry_info(Some(
                                (next_retry - now).to_std().expect("failed to convert duration to std"),
                            )),
                        ));
                    }
                }

                // Upsert pending email
                let new_pending = core_db_types::models::NewPendingUserEmail::builder()
                    .email(Cow::Borrowed(&email))
                    .user_id(auth_session.id())
                    .build();

                let new_pending = diesel::insert_into(pending_user_emails::table)
                    .values(&new_pending)
                    .on_conflict((pending_user_emails::email, pending_user_emails::user_id))
                    .do_update()
                    .set((
                        pending_user_emails::codes_sent.eq(pending_user_emails::codes_sent + 1),
                        pending_user_emails::last_sent_code_at.eq(chrono::Utc::now()),
                    ))
                    .returning(PendingUserEmail::as_select())
                    .get_result::<PendingUserEmail>(tx)
                    .await
                    .into_tonic_internal_err("failed to insert pending user email")?;

                Ok(new_pending)
            })
            .await?;

        send_verification_email(global.as_ref(), auth_session.id(), &new_pending.email).await?;

        Ok(tonic::Response::new(UserEmailAddResponse {
            email: Some(pending_email_to_proto(new_pending)),
        }))
    }

    async fn verify(
        &self,
        request: tonic::Request<UserEmailVerifyRequest>,
    ) -> tonic::Result<tonic::Response<UserEmailVerifyResponse>> {
        let global = request.global::<G>()?;

        // Get user_id from request or from session
        let user_id = if let Some(user_id) = request.get_ref().user_id.as_ref() {
            user_id
                .parse::<UserId>()
                .into_tonic_err_with_field_violation("user_id", "invalid user ID")?
        } else {
            // Get from session
            let auth_session = request.auth_user::<G>().await?.required()?;
            auth_session.id()
        };

        let req = request.into_inner();

        let email = req.email.to_lowercase().trim().to_string();

        // Check code in Redis
        let redis_key = format!("email:verify:{}:{}", user_id, email);
        let Some(stored_code) = global
            .redis()
            .getdel::<Option<String>, _>(&redis_key)
            .await
            .into_tonic_internal_err("failed to retrieve verification code")?
        else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::NotFound,
                "verification code expired or not found",
                ErrorDetails::new(),
            ));
        };

        if stored_code != req.code {
            return Err(tonic::Status::with_error_details(
                tonic::Code::NotFound,
                "verification code expired or not found",
                ErrorDetails::new(),
            ));
        }

        let mut db = global
            .db()
            .await
            .into_tonic_internal_err("failed to get database connection")?;

        let email_record = db
            .tx(async |tx| {
                // Verify email doesn't already exist for another user
                let count = user_emails::table
                    .find(&email)
                    .count()
                    .get_result::<i64>(tx)
                    .await
                    .into_tonic_internal_err("failed to query user email")?;

                if count > 0 {
                    return Err(tonic::Status::with_error_details(
                        tonic::Code::AlreadyExists,
                        "email is already in use by another account",
                        ErrorDetails::with_bad_request_violation("email", "email is already in use by another account"),
                    ));
                }

                let pending_email = diesel::delete(
                    pending_user_emails::table
                        .filter(pending_user_emails::email.eq(&email))
                        .filter(pending_user_emails::user_id.eq(user_id)),
                )
                .execute(tx)
                .await
                .into_tonic_internal_err("failed to delete pending user email")?;

                if pending_email != 1 {
                    return Err(tonic::Status::with_error_details(
                        tonic::Code::NotFound,
                        "verification code expired or not found",
                        ErrorDetails::with_bad_request_violation("code", "verification code expired or not found"),
                    ));
                };

                let new_email = NewUserEmail::builder().email(Cow::Borrowed(&email)).user_id(user_id).build();

                let new_email = diesel::insert_into(user_emails::table)
                    .values(&new_email)
                    .returning(UserEmail::as_select())
                    .get_result::<UserEmail>(tx)
                    .await
                    .into_tonic_internal_err("failed to insert user email")?;

                Ok(new_email)
            })
            .await?;

        Ok(tonic::Response::new(UserEmailVerifyResponse {
            email: Some(email_to_proto(email_record)),
        }))
    }
}
