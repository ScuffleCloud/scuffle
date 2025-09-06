use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use rand::Rng;
use tonic::Code;
use tonic_types::{ErrorDetails, StatusExt};

use crate::cedar::{Action, Unauthenticated};
use crate::http_ext::RequestExt;
use crate::models::{User, UserSessionRequest, UserSessionRequestId};
use crate::operations::{NoopOperationDriver, Operation, TransactionOperationDriver};
use crate::schema::user_session_requests;
use crate::std_ext::{OptionExt, ResultExt};
use crate::{CoreConfig, common};

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CreateUserSessionRequestRequest> {
    type Driver = NoopOperationDriver;
    type Principal = Unauthenticated;
    type Resource = UserSessionRequest;
    type Response = pb::scufflecloud::core::v1::UserSessionRequest;

    const ACTION: Action = Action::CreateUserSessionRequest;

    async fn load_principal(&mut self, _driver: &mut Self::Driver) -> Result<Self::Principal, tonic::Status> {
        Ok(Unauthenticated)
    }

    async fn load_resource(&mut self, _driver: &mut Self::Driver) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let ip_info = self.ip_address_info()?;
        let code = format!("{:06}", rand::rngs::OsRng.gen_range(0..=999999));

        Ok(UserSessionRequest {
            id: UserSessionRequestId::new(),
            device_name: self.get_ref().name.clone(),
            device_ip: ip_info.to_network(),
            code,
            approved_by: None,
            expires_at: chrono::Utc::now() + global.user_session_request_timeout(),
        })
    }

    async fn execute(
        self,
        _driver: &mut Self::Driver,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        diesel::insert_into(user_session_requests::dsl::user_session_requests)
            .values(&resource)
            .execute(&mut db)
            .await
            .into_tonic_internal_err("failed to insert user session request")?;

        Ok(resource.into())
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestRequest> {
    type Driver = NoopOperationDriver;
    type Principal = Unauthenticated;
    type Resource = UserSessionRequest;
    type Response = pb::scufflecloud::core::v1::UserSessionRequest;

    const ACTION: Action = Action::GetUserSessionRequest;

    async fn load_principal(&mut self, _driver: &mut Self::Driver) -> Result<Self::Principal, tonic::Status> {
        Ok(Unauthenticated)
    }

    async fn load_resource(&mut self, _driver: &mut Self::Driver) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let id: UserSessionRequestId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

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

        Ok(session_request)
    }

    async fn execute(
        self,
        _driver: &mut Self::Driver,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        Ok(resource.into())
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::GetUserSessionRequestByCodeRequest> {
    type Driver = NoopOperationDriver;
    type Principal = Unauthenticated;
    type Resource = UserSessionRequest;
    type Response = pb::scufflecloud::core::v1::UserSessionRequest;

    const ACTION: Action = Action::GetUserSessionRequest;

    async fn load_principal(&mut self, _driver: &mut Self::Driver) -> Result<Self::Principal, tonic::Status> {
        Ok(Unauthenticated)
    }

    async fn load_resource(&mut self, _driver: &mut Self::Driver) -> Result<Self::Resource, tonic::Status> {
        let global = &self.global::<G>()?;
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let Some(session_request) = user_session_requests::dsl::user_session_requests
            .filter(
                user_session_requests::dsl::code
                    .eq(&self.get_ref().code)
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

        Ok(session_request)
    }

    async fn execute(
        self,
        _driver: &mut Self::Driver,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        Ok(resource.into())
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::ApproveUserSessionRequestByCodeRequest> {
    type Driver = TransactionOperationDriver;
    type Principal = User;
    type Resource = UserSessionRequest;
    type Response = pb::scufflecloud::core::v1::UserSessionRequest;

    const ACTION: Action = Action::ApproveUserSessionRequest;

    async fn load_principal(&mut self, _driver: &mut Self::Driver) -> Result<Self::Principal, tonic::Status> {
        let global = &self.global::<G>()?;
        let session = self.session_or_err()?;
        common::get_user_by_id(global, session.user_id).await
    }

    async fn load_resource(&mut self, driver: &mut Self::Driver) -> Result<Self::Resource, tonic::Status> {
        let Some(session_request) = user_session_requests::dsl::user_session_requests
            .filter(
                user_session_requests::dsl::code
                    .eq(&self.get_ref().code)
                    .and(user_session_requests::dsl::approved_by.is_null())
                    .and(user_session_requests::dsl::expires_at.gt(chrono::Utc::now())),
            )
            .select(UserSessionRequest::as_select())
            .first::<UserSessionRequest>(&mut driver.conn)
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

        Ok(session_request)
    }

    async fn execute(
        self,
        driver: &mut Self::Driver,
        principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let session_request = diesel::update(user_session_requests::dsl::user_session_requests)
            .filter(user_session_requests::dsl::id.eq(resource.id))
            .set(user_session_requests::dsl::approved_by.eq(&principal.id))
            .returning(UserSessionRequest::as_select())
            .get_result::<UserSessionRequest>(&mut driver.conn)
            .await
            .into_tonic_internal_err("failed to update user session request")?;

        Ok(session_request.into())
    }
}

impl<G: CoreConfig> Operation<G> for tonic::Request<pb::scufflecloud::core::v1::CompleteUserSessionRequestRequest> {
    type Driver = TransactionOperationDriver;
    type Principal = Unauthenticated;
    type Resource = UserSessionRequest;
    type Response = pb::scufflecloud::core::v1::NewUserSessionToken;

    const ACTION: Action = Action::CompleteUserSessionRequest;

    async fn load_principal(&mut self, _driver: &mut Self::Driver) -> Result<Self::Principal, tonic::Status> {
        Ok(Unauthenticated)
    }

    async fn load_resource(&mut self, driver: &mut Self::Driver) -> Result<Self::Resource, tonic::Status> {
        let id: UserSessionRequestId = self
            .get_ref()
            .id
            .parse()
            .into_tonic_err_with_field_violation("id", "invalid ID")?;

        // Delete the session request
        let Some(session_request) = diesel::delete(user_session_requests::dsl::user_session_requests)
            .filter(user_session_requests::dsl::id.eq(id))
            .returning(UserSessionRequest::as_select())
            .get_result::<UserSessionRequest>(&mut driver.conn)
            .await
            .optional()
            .into_tonic_internal_err("failed to delete user session request")?
        else {
            return Err(tonic::Status::with_error_details(
                Code::NotFound,
                "unknown id",
                ErrorDetails::new(),
            ));
        };

        Ok(session_request)
    }

    async fn execute(
        self,
        driver: &mut Self::Driver,
        _principal: Self::Principal,
        resource: Self::Resource,
    ) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;
        let ip_info = self.ip_address_info()?;
        let payload = self.into_inner();

        let device = payload.device.require("device")?;

        let Some(approved_by) = resource.approved_by else {
            return Err(tonic::Status::with_error_details(
                tonic::Code::FailedPrecondition,
                "user session request is not approved yet",
                ErrorDetails::new(),
            ));
        };

        let new_token = common::create_session(global, &mut driver.conn, approved_by, device, &ip_info, false).await?;
        Ok(new_token)
    }
}
