use std::sync::Arc;

use scuffle_bootstrap::global::Global;
use scuffle_bootstrap::service::Service;
use scuffle_context::ContextFutExt;

/// A [`Service`] that listens for signals and cancels the context when a signal is received.
#[derive(Default, Debug, Clone, Copy)]
pub struct SignalSvc;

/// Configuration for the signal service.
pub trait SignalConfig: Global {
    /// The signals to listen for.
    ///
    /// By default, listens for `SIGTERM` and `SIGINT`.
    fn signals(&self) -> Vec<tokio::signal::unix::SignalKind> {
        vec![
            tokio::signal::unix::SignalKind::terminate(),
            tokio::signal::unix::SignalKind::interrupt(),
        ]
    }

    /// The timeout before forcing a shutdown.
    fn timeout(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs(30))
    }

    /// Called when the service is shutting down.
    fn on_shutdown(self: &Arc<Self>) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        std::future::ready(Ok(()))
    }

    /// Called when the service is force shutting down.
    fn on_force_shutdown(
        &self,
        signal: Option<tokio::signal::unix::SignalKind>,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        let err = if let Some(signal) = signal {
            anyhow::anyhow!("received signal, shutting down immediately: {:?}", signal)
        } else {
            anyhow::anyhow!("timeout reached, shutting down immediately")
        };

        std::future::ready(Err(err))
    }
}

impl<Global: SignalConfig> Service<Global> for SignalSvc {
    fn enabled(&self, global: &Arc<Global>) -> impl std::future::Future<Output = anyhow::Result<bool>> + Send {
        std::future::ready(Ok(!global.signals().is_empty()))
    }

    async fn run(self, global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        let timeout = global.timeout();

        let signals = global.signals();
        let mut handler = crate::SignalHandler::with_signals(signals);

        // Wait for a signal, or for the context to be done.
        handler.recv().with_context(&ctx).await;
        global.on_shutdown().await?;
        drop(ctx);

        tokio::select! {
            signal = handler.recv() => {
                global.on_force_shutdown(Some(signal)).await?;
            },
            _ = scuffle_context::Handler::global().shutdown() => {}
            Some(()) = async {
                if let Some(timeout) = timeout {
                    tokio::time::sleep(timeout).await;
                    Some(())
                } else {
                    None
                }
            } => {
                global.on_force_shutdown(None).await?;
            },
        };

        Ok(())
    }
}

#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use scuffle_bootstrap::Service;
    use scuffle_bootstrap::global::GlobalWithoutConfig;
    use scuffle_future_ext::FutureExt;
    use tokio::signal::unix::SignalKind;

    use super::{SignalConfig, SignalSvc};
    use crate::SignalHandler;
    use crate::tests::raise_signal;

    async fn force_shutdown_two_signals<Global: GlobalWithoutConfig + SignalConfig>() {
        let (ctx, handler) = scuffle_context::Context::new();

        // Block the global context
        let _global_ctx = scuffle_context::Context::global();

        let svc = SignalSvc;
        let global = <Global as GlobalWithoutConfig>::init().await.unwrap();

        assert!(svc.enabled(&global).await.unwrap());
        let result = tokio::spawn(svc.run(global, ctx));

        // Wait for the service to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        raise_signal(tokio::signal::unix::SignalKind::interrupt());
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        raise_signal(tokio::signal::unix::SignalKind::interrupt());

        match result.with_timeout(tokio::time::Duration::from_millis(100)).await {
            Ok(Ok(Err(e))) => {
                assert_eq!(e.to_string(), "received signal, shutting down immediately: SignalKind(2)");
            }
            _ => panic!("unexpected result"),
        }

        assert!(
            handler
                .shutdown()
                .with_timeout(tokio::time::Duration::from_millis(100))
                .await
                .is_ok()
        );
    }

    struct TestGlobal;

    impl GlobalWithoutConfig for TestGlobal {
        fn init() -> impl std::future::Future<Output = anyhow::Result<Arc<Self>>> + Send {
            std::future::ready(Ok(Arc::new(Self)))
        }
    }

    impl SignalConfig for TestGlobal {}

    #[tokio::test]
    async fn default_bootstrap_service() {
        force_shutdown_two_signals::<TestGlobal>().await;
    }
    struct NoTimeoutTestGlobal;

    impl GlobalWithoutConfig for NoTimeoutTestGlobal {
        fn init() -> impl std::future::Future<Output = anyhow::Result<Arc<Self>>> + Send {
            std::future::ready(Ok(Arc::new(Self)))
        }
    }

    impl SignalConfig for NoTimeoutTestGlobal {
        fn timeout(&self) -> Option<std::time::Duration> {
            None
        }
    }

    #[tokio::test]
    async fn bootstrap_service_no_timeout() {
        let (ctx, handler) = scuffle_context::Context::new();
        let svc = SignalSvc;
        let global = NoTimeoutTestGlobal::init().await.unwrap();

        assert!(svc.enabled(&global).await.unwrap());
        let result = tokio::spawn(svc.run(global, ctx));

        // Wait for the service to start
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;

        raise_signal(tokio::signal::unix::SignalKind::interrupt());
        assert!(result.await.is_ok());

        assert!(
            handler
                .shutdown()
                .with_timeout(tokio::time::Duration::from_millis(100))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn bootstrap_service_force_shutdown() {
        force_shutdown_two_signals::<NoTimeoutTestGlobal>().await;
    }

    struct NoSignalsTestGlobal;

    impl GlobalWithoutConfig for NoSignalsTestGlobal {
        fn init() -> impl std::future::Future<Output = anyhow::Result<Arc<Self>>> + Send {
            std::future::ready(Ok(Arc::new(Self)))
        }
    }

    impl SignalConfig for NoSignalsTestGlobal {
        fn signals(&self) -> Vec<tokio::signal::unix::SignalKind> {
            vec![]
        }

        fn timeout(&self) -> Option<std::time::Duration> {
            None
        }
    }

    #[tokio::test]
    async fn bootstrap_service_no_signals() {
        let (ctx, handler) = scuffle_context::Context::new();
        let svc = SignalSvc;
        let global = NoSignalsTestGlobal::init().await.unwrap();

        assert!(!svc.enabled(&global).await.unwrap());
        let result = tokio::spawn(svc.run(global, ctx));

        // Wait for the service to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Make a new handler to catch the raised signal as it is expected to not be
        // caught by the service
        let mut signal_handler = SignalHandler::new().with_signal(SignalKind::terminate());

        raise_signal(tokio::signal::unix::SignalKind::terminate());

        // Wait for a signal to be received
        assert_eq!(signal_handler.recv().await, SignalKind::terminate());

        // Expected to timeout
        assert!(result.with_timeout(tokio::time::Duration::from_millis(100)).await.is_err());

        assert!(
            handler
                .shutdown()
                .with_timeout(tokio::time::Duration::from_millis(100))
                .await
                .is_ok()
        );
    }

    struct SmallTimeoutTestGlobal;

    impl GlobalWithoutConfig for SmallTimeoutTestGlobal {
        fn init() -> impl std::future::Future<Output = anyhow::Result<Arc<Self>>> + Send {
            std::future::ready(Ok(Arc::new(Self)))
        }
    }

    impl SignalConfig for SmallTimeoutTestGlobal {
        fn timeout(&self) -> Option<std::time::Duration> {
            Some(std::time::Duration::from_millis(5))
        }
    }

    #[tokio::test]
    async fn bootstrap_service_timeout_force_shutdown() {
        let (ctx, handler) = scuffle_context::Context::new();

        // Block the global context
        let _global_ctx = scuffle_context::Context::global();

        let svc = SignalSvc;
        let global = SmallTimeoutTestGlobal::init().await.unwrap();

        assert!(svc.enabled(&global).await.unwrap());
        let result = tokio::spawn(svc.run(global, ctx));

        // Wait for the service to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        raise_signal(tokio::signal::unix::SignalKind::terminate());

        match result.with_timeout(tokio::time::Duration::from_millis(100)).await {
            Ok(Ok(Err(e))) => {
                assert_eq!(e.to_string(), "timeout reached, shutting down immediately");
            }
            _ => panic!("unexpected result"),
        }

        assert!(
            handler
                .shutdown()
                .with_timeout(tokio::time::Duration::from_millis(100))
                .await
                .is_ok()
        );
    }
}
