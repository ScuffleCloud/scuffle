use std::sync::Arc;

use diesel_async::pooled_connection::bb8::PooledConnection;
use diesel_async::{AnsiTransactionManager, AsyncPgConnection, TransactionManager};

use crate::CoreConfig;
use crate::cedar::{self, Action, CedarEntity};
use crate::http_ext::RequestExt;
use crate::std_ext::ResultExt;

pub(crate) mod login;
pub(crate) mod organization_invitations;
pub(crate) mod organizations;
pub(crate) mod register;
pub(crate) mod user_session_requests;
pub(crate) mod user_sessions;
pub(crate) mod users;

pub(crate) trait OperationDriver<G: CoreConfig>: Sized {
    async fn start(global: &Arc<G>) -> Result<Self, tonic::Status>;
    async fn abort(self) -> Result<(), tonic::Status>;
    async fn finish(self) -> Result<(), tonic::Status>;
}

pub(crate) struct TransactionOperationDriver {
    conn: PooledConnection<'static, AsyncPgConnection>,
}

impl<G: CoreConfig> OperationDriver<G> for TransactionOperationDriver {
    async fn start(global: &Arc<G>) -> Result<Self, tonic::Status> {
        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;
        AnsiTransactionManager::begin_transaction(&mut *db)
            .await
            .into_tonic_internal_err("failed to begin transaction")?;

        Ok(Self { conn: db })
    }

    async fn abort(mut self) -> Result<(), tonic::Status> {
        AnsiTransactionManager::rollback_transaction(&mut *self.conn)
            .await
            .into_tonic_internal_err("failed to rollback transaction")
    }

    async fn finish(mut self) -> Result<(), tonic::Status> {
        AnsiTransactionManager::commit_transaction(&mut *self.conn)
            .await
            .into_tonic_internal_err("failed to commit transaction")
    }
}

pub(crate) struct NoopOperationDriver;

impl<G: CoreConfig> OperationDriver<G> for NoopOperationDriver {
    async fn start(global: &Arc<G>) -> Result<Self, tonic::Status> {
        let _ = global;
        Ok(Self)
    }

    async fn abort(self) -> Result<(), tonic::Status> {
        let _ = self;
        Ok(())
    }

    async fn finish(self) -> Result<(), tonic::Status> {
        let _ = self;
        Ok(())
    }
}

/// This trait defines a framework for operations.
///
/// The process of running an operation (calling [`run`](Self::run)) is as follows:
/// 1. Validate the operation with [`validate`](Self::validate).
/// 2. Start the operation driver with [`OperationDriver::start`].
/// 3. Load the principal with [`load_principal`](Self::load_principal).
/// 4. Load the resource with [`load_resource`](Self::load_resource).
/// 5. Check if the principal is authorized to perform the action on the resource with Cedar.
/// 6. Execute the operation with [`execute`](Self::execute).
/// 7. Commit the operation with [`OperationDriver::finish`] if successful,
///    or abort with [`OperationDriver::abort`] if an error occurred.
pub(crate) trait Operation<G: CoreConfig>: RequestExt + Sized + Send {
    type Driver: OperationDriver<G> + Send + Sync;
    /// The cedar principal type for the operation.
    type Principal: CedarEntity<G> + Send + Sync;
    /// The cedar resource type for the operation.
    type Resource: CedarEntity<G> + Send + Sync;
    /// The response type for the operation.
    type Response: Send;

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
        driver: &mut Self::Driver,
    ) -> impl Future<Output = Result<Self::Principal, tonic::Status>> + Send;

    fn load_resource(
        &mut self,
        driver: &mut Self::Driver,
    ) -> impl Future<Output = Result<Self::Resource, tonic::Status>> + Send;

    fn execute(
        self,
        driver: &mut Self::Driver,
        principal: Self::Principal,
        resource: Self::Resource,
    ) -> impl Future<Output = Result<Self::Response, tonic::Status>> + Send;

    async fn run(mut self) -> Result<Self::Response, tonic::Status> {
        self.validate().await?;

        let global = self.global::<G>()?;
        let mut driver = Self::Driver::start(&global).await?;

        let fut = async {
            let principal = self.load_principal(&mut driver).await?;
            let resource = self.load_resource(&mut driver).await?;

            cedar::is_authorized(&global, self.session(), &principal, Self::ACTION, &resource).await?;

            self.execute(&mut driver, principal, resource).await
        };

        match fut.await {
            Ok(resp) => {
                driver.finish().await?;
                Ok(resp)
            }
            Err(e) => {
                driver.abort().await?;
                Err(e)
            }
        }
    }
}
