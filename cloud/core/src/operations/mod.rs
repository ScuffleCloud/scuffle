use diesel_async::AsyncConnection;
use diesel_async::scoped_futures::ScopedFutureExt;

use crate::CoreConfig;
use crate::cedar::{self, Action, CedarEntity};
use crate::common::TxError;
use crate::http_ext::RequestExt;
use crate::std_ext::ResultExt;

mod login;
mod register;

pub(crate) trait Operation<G: CoreConfig>: RequestExt + Sized + Send {
    type Principal: CedarEntity + Send;
    const ACTION: Action;
    type Resource: CedarEntity + Send;
    type Response: Send;

    async fn validate(&self) -> Result<(), tonic::Status> {
        Ok(())
    }

    fn load_principal(
        &self,
        tx: &mut diesel_async::AsyncPgConnection,
    ) -> impl Future<Output = Result<Self::Principal, tonic::Status>> + Send;

    fn load_resource(
        &self,
        tx: &mut diesel_async::AsyncPgConnection,
    ) -> impl Future<Output = Result<Self::Resource, tonic::Status>> + Send;

    fn execute(
        self,
        tx: &mut diesel_async::AsyncPgConnection,
        principal: Self::Principal,
        resource: Self::Resource,
    ) -> impl Future<Output = Result<Self::Response, tonic::Status>> + Send;

    async fn run(self) -> Result<Self::Response, tonic::Status> {
        let global = &self.global::<G>()?;

        self.validate().await?;

        let mut db = global.db().await.into_tonic_internal_err("failed to connect to database")?;

        let resp = db
            .transaction::<_, TxError, _>(move |tx| {
                async move {
                    let principal = self.load_principal(tx).await?;
                    let resource = self.load_resource(tx).await?;

                    cedar::is_authorized(global, self.session(), &principal, Self::ACTION, &resource)?;

                    self.execute(tx, principal, resource).await.map_err(Into::into)
                }
                .scope_boxed()
            })
            .await?;

        Ok(resp)
    }
}
