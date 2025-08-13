//! Service types.

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, ready};

/// A service that can be run.
///
/// This trait is used to define a service that can be run in parallel to other
/// services.
///
/// # See Also
///
/// - [`Global`](crate::Global)
/// - [`GlobalWithoutConfig`](crate::GlobalWithoutConfig)
/// - [`main`](crate::main)
pub trait Service<Global>: Send + Sync + 'static + Sized {
    /// Returns the name of the service, if any.
    fn name(&self) -> Option<&'static str> {
        None
    }

    /// Initialize the service and return `Ok(true)` if the service should be
    /// run.
    fn enabled(&self, global: &Arc<Global>) -> impl std::future::Future<Output = anyhow::Result<bool>> + Send {
        let _ = global;
        std::future::ready(Ok(true))
    }

    /// Run the service.
    /// This function should return a future that is pending as long as the
    /// service is running. When the service finishes without any errors,
    /// the future should resolve to `Ok(())`. As a best practice, the
    /// service should stop as soon as the provided context is done.
    ///
    /// # See Also
    /// - [`scuffle_context::Context`]
    fn run(
        self,
        global: Arc<Global>,
        ctx: scuffle_context::Context,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send + 'static {
        let _ = global;
        async move {
            ctx.done().await;
            Ok(())
        }
    }
}

impl<G, F, Fut> Service<G> for F
where
    F: FnOnce(Arc<G>, scuffle_context::Context) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    fn run(
        self,
        global: Arc<G>,
        ctx: scuffle_context::Context,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send + 'static {
        self(global, ctx)
    }
}

pin_project_lite::pin_project! {
    /// A future that can be named yielding the name and the result of the inner future.
    #[must_use = "futures do nothing unless polled"]
    pub struct NamedFuture<T> {
        name: &'static str,
        #[pin]
        fut: T,
    }
}

impl<T> NamedFuture<T> {
    /// Create a new named future from the given name and future.
    pub fn new(name: &'static str, fut: T) -> Self {
        Self { name, fut }
    }
}

impl<T> std::future::Future for NamedFuture<T>
where
    T: std::future::Future,
{
    type Output = (&'static str, T::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let res = ready!(this.fut.poll(cx));
        Poll::Ready((this.name, res))
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::sync::Arc;

    use scuffle_future_ext::FutureExt;

    use super::{NamedFuture, Service};

    struct DefaultService;

    impl Service<()> for DefaultService {}

    #[tokio::test]
    async fn defaukt_service() {
        let svc = DefaultService;
        let global = Arc::new(());
        let (ctx, handler) = scuffle_context::Context::new();

        assert_eq!(svc.name(), None);
        assert!(svc.enabled(&global).await.unwrap());

        handler.cancel();

        assert!(matches!(svc.run(global, ctx).await, Ok(())));

        assert!(
            handler
                .shutdown()
                .with_timeout(tokio::time::Duration::from_millis(200))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn future_service() {
        let (ctx, handler) = scuffle_context::Context::new();
        let global = Arc::new(());

        let fut_fn = |_global: Arc<()>, _ctx: scuffle_context::Context| async { anyhow::Result::<()>::Ok(()) };
        assert!(fut_fn.run(global, ctx).await.is_ok());

        handler.cancel();
        assert!(
            handler
                .shutdown()
                .with_timeout(tokio::time::Duration::from_millis(200))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn named_future() {
        let named_fut = NamedFuture::new("test", async { 42 });
        assert_eq!(named_fut.await, ("test", 42));
    }
}
