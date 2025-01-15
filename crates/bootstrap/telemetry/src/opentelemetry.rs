pub use ::opentelemetry::*;

#[derive(Debug, thiserror::Error)]
pub enum OpenTelemetryError {
    #[error("metrics: {0}")]
    Metrics(#[from] opentelemetry_sdk::metrics::MetricError),
    #[error("traces: {0}")]
    Traces(#[from] opentelemetry::trace::TraceError),
    #[error("logs: {0}")]
    Logs(#[from] opentelemetry_sdk::logs::LogError),
}

#[derive(Debug, Default, Clone)]
pub struct OpenTelemetry {
    #[cfg(feature = "opentelemetry-metrics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-metrics")))]
    metrics: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
    #[cfg(feature = "opentelemetry-traces")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-traces")))]
    traces: Option<opentelemetry_sdk::trace::TracerProvider>,
    #[cfg(feature = "opentelemetry-logs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-logs")))]
    logs: Option<opentelemetry_sdk::logs::LoggerProvider>,
}

impl OpenTelemetry {
    pub fn new() -> Self {
        Self::default()
    }

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

    #[cfg(feature = "opentelemetry-traces")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-traces")))]
    pub fn with_traces(self, traces: impl Into<Option<opentelemetry_sdk::trace::TracerProvider>>) -> Self {
        Self {
            traces: traces.into(),
            #[cfg(feature = "opentelemetry-metrics")]
            metrics: self.metrics,
            #[cfg(feature = "opentelemetry-logs")]
            logs: self.logs,
        }
    }

    #[cfg(feature = "opentelemetry-logs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry-logs")))]
    pub fn with_logs(self, logs: impl Into<Option<opentelemetry_sdk::logs::LoggerProvider>>) -> Self {
        Self {
            logs: logs.into(),
            #[cfg(feature = "opentelemetry-traces")]
            traces: self.traces,
            #[cfg(feature = "opentelemetry-metrics")]
            metrics: self.metrics,
        }
    }

    /// Flushes all metrics, traces, and logs, warning; this blocks the
    /// current thread.
    pub fn flush(&self) -> Result<(), OpenTelemetryError> {
        #[cfg(feature = "opentelemetry-metrics")]
        if let Some(metrics) = &self.metrics {
            metrics.force_flush()?;
        }

        #[cfg(feature = "opentelemetry-traces")]
        if let Some(traces) = &self.traces {
            for r in traces.force_flush() {
                r?;
            }
        }

        #[cfg(feature = "opentelemetry-logs")]
        if let Some(logs) = &self.logs {
            for r in logs.force_flush() {
                r?;
            }
        }

        Ok(())
    }

    /// Shuts down all metrics, traces, and logs.
    pub fn shutdown(&self) -> Result<(), OpenTelemetryError> {
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