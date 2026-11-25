//! Minimal server for the loopback OTLP wire test.
use axum::{Router, middleware, routing::get};
use nine_dev_site_lib::{
    server::{self, Server},
    telemetry::{self, Telemetry},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let telemetry = Telemetry::init("telemetry-smoke", "smoke-version", &["smoke_app"])?;
    Ok(server::run(telemetry, serve)?)
}

async fn serve(server: Server) -> std::io::Result<()> {
    tracing::info!(target: "smoke_app::startup", "allowed application event");
    tracing::info!(target: "smoke_application", "local-only event");
    let app = Router::new()
        .route("/items/{id}", get(|| async { "ok" }))
        .route("/health", get(|| async { "ok" }))
        .route(
            "/{*path}",
            get(|| async { axum::http::StatusCode::NOT_FOUND }),
        )
        .layer(middleware::from_fn_with_state(
            server.requests.with_route_override(|pattern, _| {
                (pattern == Some("/{*path}")).then_some("unmatched")
            }),
            telemetry::observe,
        ));
    let listener = tokio::net::TcpListener::bind(server.address).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(server.shutdown)
        .await
}
