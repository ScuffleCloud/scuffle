//! Opentelemetry support

pub use ::opentelemetry::*;

/// OpenTelemetry configuration.
///
/// This struct contains different OpenTelemetry providers for metrics, traces, and logs.
/// If set, these providers will be used to collect and export telemetry data.
#[derive(Debug, Default, Clone)]
pub struct OpenTelemetry {
    #[cfg(feature = "opentelemetry-metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-metrics")))]
    metrics: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
    #[cfg(feature = "opentelemetry-traces")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-traces")))]
    traces: Option<opentelemetry_sdk::trace::SdkTracerProvider>,
    #[cfg(feature = "opentelemetry-logs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-logs")))]
    logs: Option<opentelemetry_sdk::logs::SdkLoggerProvider>,
}

impl OpenTelemetry {
    /// Creates a new empty OpenTelemetry configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if any of the providers are enabled.
    pub fn is_enabled(&self) -> bool {
        #[cfg_attr(
            not(any(
                feature = "opentelemetry-metrics",
                feature = "opentelemetry-traces",
                feature = "opentelemetry-logs"
            )),
            allow(unused_mut)
        )]
        let mut enabled = false;
        #[cfg(feature = "opentelemetry-metrics")]
        {
            enabled |= self.metrics.is_some();
        }
        #[cfg(feature = "opentelemetry-traces")]
        {
            enabled |= self.traces.is_some();
        }
        #[cfg(feature = "opentelemetry-logs")]
        {
            enabled |= self.logs.is_some();
        }
        enabled
    }

    /// Sets the metrics provider.
    #[cfg(feature = "opentelemetry-metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-metrics")))]
    pub fn with_metrics(self, metrics: impl Into<Option<opentelemetry_sdk::metrics::SdkMeterProvider>>) -> Self {
        Self {
            metrics: metrics.into(),
            #[cfg(feature = "opentelemetry-traces")]
            traces: self.traces,
            #[cfg(feature = "opentelemetry-logs")]
            logs: self.logs,
        }
    }

    /// Sets the traces provider.
    #[cfg(feature = "opentelemetry-traces")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-traces")))]
    pub fn with_traces(self, traces: impl Into<Option<opentelemetry_sdk::trace::SdkTracerProvider>>) -> Self {
        Self {
            traces: traces.into(),
            #[cfg(feature = "opentelemetry-metrics")]
            metrics: self.metrics,
            #[cfg(feature = "opentelemetry-logs")]
            logs: self.logs,
        }
    }

    /// Sets the logs provider.
    #[cfg(feature = "opentelemetry-logs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-logs")))]
    pub fn with_logs(self, logs: impl Into<Option<opentelemetry_sdk::logs::SdkLoggerProvider>>) -> Self {
        Self {
            logs: logs.into(),
            #[cfg(feature = "opentelemetry-traces")]
            traces: self.traces,
            #[cfg(feature = "opentelemetry-metrics")]
            metrics: self.metrics,
        }
    }

    /// Flushes all metrics, traces, and logs.
    ///
    /// <div class="warning">Warning: This blocks the current thread.</div>
    pub fn flush(&self) -> Result<(), opentelemetry_sdk::error::OTelSdkError> {
        #[cfg(feature = "opentelemetry-metrics")]
        if let Some(metrics) = &self.metrics {
            metrics.force_flush()?;
        }

        #[cfg(feature = "opentelemetry-traces")]
        if let Some(traces) = &self.traces {
            traces.force_flush()?;
        }

        #[cfg(feature = "opentelemetry-logs")]
        if let Some(logs) = &self.logs {
            logs.force_flush()?;
        }

        Ok(())
    }

    /// Shuts down all metrics, traces, and logs.
    pub fn shutdown(&self) -> Result<(), opentelemetry_sdk::error::OTelSdkError> {
        #[cfg(feature = "opentelemetry-metrics")]
        if let Some(metrics) = &self.metrics {
            metrics.shutdown()?;
        }

        #[cfg(feature = "opentelemetry-traces")]
        if let Some(traces) = &self.traces {
            traces.shutdown()?;
        }

        #[cfg(feature = "opentelemetry-logs")]
        if let Some(logs) = &self.logs {
            logs.shutdown()?;
        }

        Ok(())
    }
}
