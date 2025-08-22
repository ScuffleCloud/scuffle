use diesel_async::AsyncConnection;
use diesel_async::scoped_futures::ScopedFutureExt;

use crate::CoreConfig;
use crate::cedar::{self, Action, CedarEntity};
use crate::common::TxError;
use crate::http_ext::RequestExt;
use crate::std_ext::ResultExt;

pub(crate) mod login;
pub(crate) mod organization_invitations;
pub(crate) mod organizations;
pub(crate) mod register;
pub(crate) mod user_session_requests;
pub(crate) mod user_sessions;
pub(crate) mod users;

/// This trait defines a framework for operations.
///
/// The process of running an operation (calling [`run`](Self::run)) is as follows:
/// 1. Validate the operation with [`validate`](Self::validate).
/// 2. Start a new database transaction. (If `Self::TRANSACTION` is `true`)
/// 3. Load the principal with [`load_principal`](Self::load_principal).
/// 4. Load the resource with [`load_resource`](Self::load_resource).
/// 5. Check if the principal is authorized to perform the action on the resource with Cedar.
/// 6. Execute the operation with [`execute`](Self::execute).
/// 7. Commit the database transaction. (If `Self::TRANSACTION` is `true`)
pub(crate) trait Operation<G: CoreConfig>: RequestExt + Sized + Send {
    /// The cedar principal type for the operation.
    type Principal: CedarEntity<G> + Send + Sync;
    /// The cedar resource type for the operation.
    type Resource: CedarEntity<G> + Send + Sync;
    /// The response type for the operation.
    type Response: Send;

    const TRANSACTION: bool = true;
    /// The action that this operation represents.
    const ACTION: Action;

    /// Validates the operation request.
    /// This is called before loading principal and resource and executing the operation.
    /// If this returns an error, the operation will not be executed.
    async fn validate(&mut self) -> Result<(), tonic::Status> {
        Ok(())
    }

    fn load_principal(
        &mut self,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> impl Future<Output = Result<Self::Principal, tonic::Status>> + Send;

    fn load_resource(
        &mut self,
        conn: &mut diesel_async::AsyncPgConnection,
    ) -> impl Future<Output = Result<Self::Resource, tonic::Status>> + Send;

    fn execute(
        self,
        conn: &mut diesel_async::AsyncPgConnection,
        principal: Self::Principal,
        resource: Self::Resource,
    ) -> impl Future<Output = Result<Self::Response, tonic::Status>> + Send;

    async fn run(mut self) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;

        self.validate().await?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        if Self::TRANSACTION {
            let resp = db
                .transaction::<_, TxError, _>(move |tx| {
                    async move {
                        let principal = self.load_principal(tx).await?;
                        let resource = self.load_resource(tx).await?;

                        cedar::is_authorized(global, self.session(), &principal, Self::ACTION, &resource).await?;

                        self.execute(tx, principal, resource).await.map_err(Into::into)
                    }
                    .scope_boxed()
                })
                .await?;

            Ok(resp)
        } else {
            let principal = self.load_principal(&mut db).await?;
            let resource = self.load_resource(&mut db).await?;

            cedar::is_authorized(global, self.session(), &principal, Self::ACTION, &resource).await?;

            self.execute(&mut db, principal, resource).await
        }
    }
}
