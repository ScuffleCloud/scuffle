use std::sync::Arc;

use diesel::{OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use crate::CoreGlobal;
use crate::models::{User, UserId, UserSession};
use crate::schema::users;
use crate::services::CoreSvc;

#[async_trait::async_trait]
impl<G: CoreGlobal> pb::scufflecloud::core::v1::users_service_server::UsersService for CoreSvc<G> {
    async fn get_user(
        &self,
        request: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        if request.extensions().get::<UserSession>().is_none() {
            return Err(tonic::Status::unauthenticated("authentication required"));
        }

        let mut db = request
            .extensions()
            .get::<Arc<G>>()
            .expect("global not found")
            .db()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let user_id: UserId = request
            .get_ref()
            .id
            .parse()
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid user ID format: {e}")))?;

        let user = users::dsl::users
            .find(user_id)
            .select(User::as_select())
            .first::<User>(&mut *db)
            .await
            .optional()
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::not_found("user not found"))?;

        Ok(tonic::Response::new(user.into()))
    }

    async fn update_user_password(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UpdateUserPasswordRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn update_user_names(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UpdateUserNamesRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn update_user_primary_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UpdateUserPrimaryEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::User>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn get_user_emails(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::UserByIdRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::GetUserEmailsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn create_user_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CreateUserEmailRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn complete_create_user_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::CompleteCreateUserEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmail>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }

    async fn delete_user_email(
        &self,
        _request: tonic::Request<pb::scufflecloud::core::v1::DeleteUserEmailRequest>,
    ) -> Result<tonic::Response<pb::scufflecloud::core::v1::UserEmail>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented yet"))
    }
}
