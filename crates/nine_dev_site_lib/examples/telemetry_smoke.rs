//! Minimal server for the loopback OTLP wire test.
use axum::{Router, middleware, routing::get};
use nine_dev_site_lib::telemetry::{self, Telemetry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let telemetry = Telemetry::init("telemetry-smoke", "smoke-version", &["smoke_app"])?;
    let result = serve(telemetry.requests());
    telemetry.shutdown();
    result
}

#[tokio::main]
async fn serve(requests: telemetry::RequestMetrics) -> Result<(), Box<dyn std::error::Error>> {
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
            requests.with_route_override(|pattern, _| {
                (pattern == Some("/{*path}")).then_some("unmatched")
            }),
            telemetry::observe,
        ));
    let listener =
        tokio::net::TcpListener::bind(("127.0.0.1", std::env::var("PORT")?.parse::<u16>()?))
            .await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
