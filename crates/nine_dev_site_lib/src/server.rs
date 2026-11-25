//! Server-only startup, graceful shutdown, and telemetry lifetime.
use std::{future::Future, io, net::SocketAddr, pin::Pin};

use crate::{
    startup,
    telemetry::{RequestMetrics, Telemetry},
};

/// Inputs to the site's async server, after configuration and signal registration succeed.
pub struct Server {
    /// Bind this address after completing the site's readiness checks.
    pub address: SocketAddr,
    /// Request logging and metrics for the site's finished router.
    pub requests: RequestMetrics,
    /// Pass to Axum's `with_graceful_shutdown` to drain in-flight HTTP requests.
    pub shutdown: Pin<Box<dyn Future<Output = ()> + Send>>,
}

/// Run from synchronous `main` after CLI dispatch and `Telemetry::init`.
/// The runtime stops and exporters flush after `serve` returns, including on error.
///
/// # Errors
/// Propagates configuration, runtime, signal registration, and application errors.
pub fn run<F, E>(telemetry: Telemetry, serve: impl FnOnce(Server) -> F) -> Result<(), E>
where
    F: Future<Output = Result<(), E>>,
    E: From<io::Error>,
{
    let result = (|| {
        let address = startup::listen_addr()?;
        tokio::runtime::Runtime::new()?.block_on(async {
            serve(Server {
                address,
                requests: telemetry.requests(),
                shutdown: Box::pin(shutdown_signal()?),
            })
            .await
        })
    })();
    telemetry.shutdown();
    result
}

// Register synchronously inside Tokio, before the application can bind its listener.
fn shutdown_signal() -> io::Result<impl Future<Output = ()>> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut interrupt = signal(SignalKind::interrupt())?;
        let mut terminate = signal(SignalKind::terminate())?;
        Ok(async move {
            tokio::select! {
                _ = interrupt.recv() => {}
                _ = terminate.recv() => {}
            }
        })
    }
    #[cfg(windows)]
    {
        let mut interrupt = tokio::signal::windows::ctrl_c()?;
        Ok(async move {
            interrupt.recv().await;
        })
    }
}
