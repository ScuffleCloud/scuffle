use std::sync::Arc;

use core_cedar::CedarEntity;
use core_traits::ResultExt;
use diesel_async::TransactionManager;

use crate::cedar::{self, Action};
use crate::http_ext::RequestExt;

pub(crate) mod login;
pub(crate) mod organization_invitations;
pub(crate) mod organizations;
pub(crate) mod user_session_requests;
pub(crate) mod user_sessions;
pub(crate) mod users;

pub(crate) struct OperationDriver<'a, G: core_traits::Global> {
    global: &'a Arc<G>,
    conn: Option<G::Connection<'a>>,
}

impl<'a, G: core_traits::Global> OperationDriver<'a, G> {
    const fn new(global: &'a Arc<G>) -> Self {
        OperationDriver { global, conn: None }
    }

    pub(crate) async fn conn(&mut self) -> Result<&mut G::Connection<'a>, tonic::Status> {
        let conn = &mut self.conn;
        if let Some(conn) = conn {
            return Ok(conn);
        }

        let mut db = self
            .global
            .db()
            .await
            .into_tonic_internal_err("failed to connect to database")?;
        <G::Connection<'a> as diesel_async::AsyncConnection>::TransactionManager::begin_transaction(&mut db)
            .await
            .into_tonic_internal_err("failed to begin transaction")?;
        Ok(conn.insert(db))
    }

    async fn abort(self) -> Result<(), tonic::Status> {
        let Some(mut conn) = self.conn else {
            return Ok(());
        };

        <G::Connection<'a> as diesel_async::AsyncConnection>::TransactionManager::rollback_transaction(&mut conn)
            .await
            .into_tonic_internal_err("failed to rollback transaction")
    }

    async fn commit(self) -> Result<(), tonic::Status> {
        let Some(mut conn) = self.conn else {
            return Ok(());
        };

        <G::Connection<'a> as diesel_async::AsyncConnection>::TransactionManager::commit_transaction(&mut conn)
            .await
            .into_tonic_internal_err("failed to commit transaction")
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
pub(crate) trait Operation<G: core_traits::Global>: RequestExt + Sized + Send {
    /// The cedar principal type for the operation.
    type Principal: CedarEntity + Send + Sync;
    /// The cedar resource type for the operation.
    type Resource: CedarEntity + Send + Sync;
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
        driver: &mut OperationDriver<'_, G>,
    ) -> impl Future<Output = Result<Self::Principal, tonic::Status>> + Send;

    fn load_resource(
        &mut self,
        driver: &mut OperationDriver<'_, G>,
    ) -> impl Future<Output = Result<Self::Resource, tonic::Status>> + Send;

    fn execute(
        self,
        driver: &mut OperationDriver<'_, G>,
        principal: Self::Principal,
        resource: Self::Resource,
    ) -> impl Future<Output = Result<Self::Response, tonic::Status>> + Send;

    async fn run(mut self) -> Result<Self::Response, tonic::Status> {
        self.validate().await?;

        let global = self.global::<G>()?;
        let mut driver = OperationDriver::new(&global);

        let fut = async {
            let principal = self.load_principal(&mut driver).await?;
            let resource = self.load_resource(&mut driver).await?;

            cedar::is_authorized(&global, self.session(), &principal, &Self::ACTION, &resource).await?;

            self.execute(&mut driver, principal, resource).await
        };

        match fut.await {
            Ok(resp) => {
                driver.commit().await?;
                Ok(resp)
            }
            Err(e) => {
                driver.abort().await?;
                Err(e)
            }
        }
    }
}
