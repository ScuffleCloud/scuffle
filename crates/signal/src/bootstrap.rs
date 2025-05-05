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
    fn signals(&self) -> Vec<crate::SignalKind> {
        vec![crate::SignalKind::Terminate, crate::SignalKind::Interrupt]
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
        signal: Option<crate::SignalKind>,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        let err = if let Some(signal) = signal {
            anyhow::anyhow!("received signal, shutting down immediately: {:?}", signal)
        } else {
            anyhow::anyhow!("timeout reached, shutting down immediately")
        };

        std::future::ready(Err(err))
    }

    /// Waits for the global shutdown to complete after a signal cancels the local context.
    /// Defaults to the global context’s shutdown ([`scuffle_context::Handler::global().shutdown()`]).
    /// Override to use a custom context or condition for shutdown completion.
    fn block_global_shutdown(&self) -> impl std::future::Future<Output = ()> + Send {
        scuffle_context::Handler::global().shutdown()
    }
}

impl<Global: SignalConfig> Service<Global> for SignalSvc {
    fn enabled(&self, global: &Arc<Global>) -> impl std::future::Future<Output = anyhow::Result<bool>> + Send {
        std::future::ready(Ok(!global.signals().is_empty()))
    }

    async fn run(self, global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
        let timeout = global.timeout();

        let signals = global.signals();
        anyhow::ensure!(!signals.is_empty(), "no signals to listen for");

        let mut handler = crate::SignalHandler::with_signals(signals);

        // Wait for a signal, or for the context to be done.
        handler.recv().with_context(&ctx).await;
        global.on_shutdown().await?;
        drop(ctx);

        tokio::select! {
            signal = handler.recv() => {
                global.on_force_shutdown(Some(signal)).await?;
            },
            _ = global.block_global_shutdown() => {}
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

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod test {
    use std::sync::Arc;

    use scuffle_bootstrap::{GlobalWithoutConfig, Service};
    use scuffle_future_ext::FutureExt;

    use super::SignalConfig;
    use crate::test::raise_signal;
    use crate::{SignalKind, SignalSvc};

    async fn force_shutdown_two_signals<Global: GlobalWithoutConfig + SignalConfig>() {
        let (ctx, handler) = scuffle_context::Context::new();

        let _global_ctx = scuffle_context::Context::global();

        let svc = SignalSvc;
        let global = <Global as GlobalWithoutConfig>::init().await.unwrap();

        assert!(svc.enabled(&global).await.unwrap());
        let result = tokio::spawn(svc.run(global, ctx));

        // Wait for the service to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        raise_signal(SignalKind::Interrupt).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        raise_signal(SignalKind::Interrupt).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        match result.with_timeout(tokio::time::Duration::from_millis(1000)).await {
            Ok(Ok(Err(e))) => {
                assert_eq!(e.to_string(), "received signal, shutting down immediately: Interrupt");
            }
            r => panic!("unexpected result: {r:?}"),
        }

        assert!(
            handler
                .shutdown()
                .with_timeout(tokio::time::Duration::from_millis(1000))
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

    impl SignalConfig for TestGlobal {
        async fn block_global_shutdown(&self) {
            std::future::pending().await
        }
    }

    #[tokio::test]
    #[cfg(not(valgrind))]
    async fn default_bootstrap_service() {
        force_shutdown_two_signals::<TestGlobal>().await;
    }

    struct NoTimeoutTestGlobal(tokio::sync::Notify);

    impl GlobalWithoutConfig for NoTimeoutTestGlobal {
        fn init() -> impl std::future::Future<Output = anyhow::Result<Arc<Self>>> + Send {
            std::future::ready(Ok(Arc::new(Self(tokio::sync::Notify::new()))))
        }
    }

    impl SignalConfig for NoTimeoutTestGlobal {
        fn timeout(&self) -> Option<std::time::Duration> {
            None
        }

        // We dont want to block the global shutdown
        async fn block_global_shutdown(&self) {
            self.0.notified().await;
        }
    }

    #[tokio::test]
    #[cfg(not(valgrind))]
    async fn bootstrap_service_no_timeout() {
        let (ctx, handler) = scuffle_context::Context::new();
        let svc = SignalSvc;
        let global = <NoTimeoutTestGlobal as GlobalWithoutConfig>::init().await.unwrap();

        assert!(svc.enabled(&global).await.unwrap());
        let mut result = tokio::spawn(svc.run(global.clone(), ctx));

        // Wait for the service to start
        println!("waiting for service to start");
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        raise_signal(SignalKind::Interrupt).await;
        // no timeout so it should block indefinitely
        assert!(
            (&mut result)
                .with_timeout(tokio::time::Duration::from_millis(100))
                .await
                .is_err()
        );

        global.0.notify_one();

        assert!(result.with_timeout(tokio::time::Duration::from_millis(100)).await.is_ok());

        assert!(
            handler
                .shutdown()
                .with_timeout(tokio::time::Duration::from_millis(1000))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    #[cfg(not(valgrind))]
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
        fn signals(&self) -> Vec<crate::SignalKind> {
            vec![]
        }

        fn timeout(&self) -> Option<std::time::Duration> {
            None
        }

        async fn block_global_shutdown(&self) {
            std::future::pending().await
        }
    }

    #[tokio::test]
    async fn bootstrap_service_no_signals() {
        let (ctx, handler) = scuffle_context::Context::new();
        let svc = SignalSvc;
        let global = <NoSignalsTestGlobal as GlobalWithoutConfig>::init().await.unwrap();

        assert!(!svc.enabled(&global).await.unwrap());
        let result = svc.run(global, ctx).await.unwrap_err();

        assert_eq!(result.to_string(), "no signals to listen for");

        assert!(
            handler
                .shutdown()
                .with_timeout(tokio::time::Duration::from_millis(1000))
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
            Some(std::time::Duration::from_millis(50))
        }

        async fn block_global_shutdown(&self) {
            std::future::pending().await
        }
    }

    #[tokio::test]
    #[cfg(not(valgrind))]
    async fn bootstrap_service_timeout_force_shutdown() {
        let (ctx, handler) = scuffle_context::Context::new();

        // Block the global context
        let _global_ctx = scuffle_context::Context::global();

        let svc = SignalSvc;
        let global = <SmallTimeoutTestGlobal as GlobalWithoutConfig>::init().await.unwrap();

        assert!(svc.enabled(&global).await.unwrap());
        let result = tokio::spawn(svc.run(global, ctx));

        // Wait for the service to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        raise_signal(crate::SignalKind::Interrupt).await;

        match result.with_timeout(tokio::time::Duration::from_millis(1000)).await {
            Ok(Ok(Err(e))) => {
                assert_eq!(e.to_string(), "timeout reached, shutting down immediately");
            }
            _ => panic!("unexpected result"),
        }

        assert!(
            handler
                .shutdown()
                .with_timeout(tokio::time::Duration::from_millis(1000))
                .await
                .is_ok()
        );
    }
}
