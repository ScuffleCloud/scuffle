use std::sync::Arc;

#[derive(Debug)]
pub struct VideoApiSvc<G> {
    _phantom: std::marker::PhantomData<G>,
}

impl<G> Default for VideoApiSvc<G> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<G: video_api_traits::Global> scuffle_bootstrap::Service<G> for VideoApiSvc<G> {
    async fn run(self, _global: Arc<G>, _ctx: scuffle_context::Context) -> anyhow::Result<()> {
        Ok(())
    }
}
