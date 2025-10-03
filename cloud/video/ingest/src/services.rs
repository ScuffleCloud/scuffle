use std::sync::Arc;

#[derive(Debug)]
pub struct IngestSvc<G> {
    _phantom: std::marker::PhantomData<G>,
}

impl<G> Default for IngestSvc<G> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<G: ingest_traits::Global> scuffle_bootstrap::Service<G> for IngestSvc<G> {
    async fn run(self, _global: Arc<G>, _ctx: scuffle_context::Context) -> anyhow::Result<()> {
        Ok(())
    }
}
