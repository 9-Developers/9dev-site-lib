//! Optional OTLP export. HTTP handlers never wait for the network exporter.
use std::{
    net::IpAddr,
    time::{Duration, Instant},
};

use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
};
use opentelemetry::{
    KeyValue,
    metrics::{Counter, Histogram, Meter, MeterProvider},
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider, metrics::SdkMeterProvider};
use tracing_subscriber::{EnvFilter, filter::filter_fn, prelude::*};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn optional_config(name: &str) -> Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(_) => Err(format!("{name} must contain valid Unicode").into()),
    }
}

/// Stable log target shared by the middleware across all applications.
const REQUEST_TARGET: &str = "nine_dev_site::http";

/// Owns the log and metric exporters for the lifetime of one server process.
pub struct Telemetry {
    metrics: Option<SdkMeterProvider>,
    logs: Option<SdkLoggerProvider>,
}

impl Telemetry {
    /// Install the process subscriber once, **before** entering Tokio.
    /// Pass the application version (e.g. `env!("CARGO_PKG_VERSION")` in the binary).
    /// Only request events and the listed application targets/submodules are exported.
    /// With no OTLP endpoint this installs only JSON stdout logging.
    pub fn init(
        default_service: &'static str,
        version: &'static str,
        app_targets: &'static [&'static str],
    ) -> Result<Self> {
        let endpoint = optional_config("OTEL_EXPORTER_OTLP_ENDPOINT")?;
        let mut telemetry = Self {
            metrics: None,
            logs: None,
        };
        if let Some(endpoint) = endpoint {
            validate_endpoint(&endpoint)?;
            // Keep one explicit destination and protocol for this small service.
            for signal in ["METRICS", "LOGS"] {
                for setting in ["ENDPOINT", "HEADERS", "PROTOCOL"] {
                    let name = format!("OTEL_EXPORTER_OTLP_{signal}_{setting}");
                    if optional_config(&name)?.is_some() {
                        return Err(
                            format!("{name} is unsupported; use the common OTLP settings").into(),
                        );
                    }
                }
            }
            if optional_config("OTEL_EXPORTER_OTLP_PROTOCOL")?.is_some_and(|p| p != "http/protobuf")
            {
                return Err("OTEL_EXPORTER_OTLP_PROTOCOL must be http/protobuf".into());
            }
            if let Some(headers) = optional_config("OTEL_EXPORTER_OTLP_HEADERS")? {
                validate_headers(&headers)?;
            }
            let service = label_config("OTEL_SERVICE_NAME", default_service)?;
            let environment = label_config("DEPLOYMENT_ENVIRONMENT", "development")?;
            // An allowlist prevents accidental export of arbitrary environment/resource data.
            let resource = Resource::builder_empty()
                .with_attributes([
                    KeyValue::new("service.name", service),
                    KeyValue::new("service.version", version),
                    // Distinguish cumulative streams from concurrent replicas and restarts.
                    KeyValue::new(
                        "service.instance.id",
                        format!("{:032x}", rand::random::<u128>()),
                    ),
                    KeyValue::new("deployment.environment.name", environment),
                ])
                .build();
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(5))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|_| "could not initialize OTLP HTTP client")?;
            let log_exporter = opentelemetry_otlp::LogExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
                .with_http_client(client.clone())
                .with_timeout(Duration::from_secs(5))
                .build()
                .map_err(|_| "could not initialize OTLP log exporter")?;
            let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
                .with_http_client(client)
                .with_timeout(Duration::from_secs(5))
                .build()
                .map_err(|_| "could not initialize OTLP metric exporter")?;
            telemetry.logs = Some(
                SdkLoggerProvider::builder()
                    .with_resource(resource.clone())
                    .with_batch_exporter(log_exporter)
                    .build(),
            );
            telemetry.metrics = Some(
                SdkMeterProvider::builder()
                    .with_resource(resource)
                    .with_periodic_exporter(metric_exporter)
                    .build(),
            );
        }
        let filter = match optional_config("RUST_LOG")? {
            Some(value) => {
                EnvFilter::try_new(value).map_err(|_| "RUST_LOG must be a valid tracing filter")?
            }
            None => EnvFilter::new("info"),
        };
        let bridge = telemetry.logs.as_ref().map(|provider| {
            OpenTelemetryTracingBridge::new(provider).with_filter(filter_fn(move |meta| {
                // Export only our application events. In particular never feed exporter
                // diagnostics or HTTP-client events back into the exporter itself.
                let target = meta.target();
                target == REQUEST_TARGET
                    || app_targets.iter().any(|prefix| {
                        target == *prefix
                            || target
                                .strip_prefix(prefix)
                                .is_some_and(|tail| tail.starts_with("::"))
                    })
            }))
        });
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_current_span(false)
                    .with_span_list(false),
            )
            .with(bridge)
            .try_init()
            .map_err(|_| "could not initialize tracing")?;
        tracing::info!(
            otlp_enabled = telemetry.metrics.is_some(),
            "observability initialized"
        );
        Ok(telemetry)
    }

    /// Middleware state to install on the finished Axum router (including its fallback).
    pub fn requests(&self) -> RequestMetrics {
        RequestMetrics::new(
            self.metrics
                .as_ref()
                .map(|provider| provider.meter("nine_dev_site_lib::http")),
        )
    }

    /// Flush and stop SDK workers after HTTP has drained, outside Tokio.
    pub fn shutdown(self) {
        // Shutdown flushes pending data on SDK workers; the server has already drained.
        if let Some(metrics) = self.metrics
            && metrics.shutdown().is_err()
        {
            tracing::warn!("OTLP metric shutdown failed");
        }
        if let Some(logs) = self.logs
            && logs.shutdown().is_err()
        {
            tracing::warn!("OTLP log shutdown failed");
        }
    }
}

fn label_config(name: &str, default: &str) -> Result<String> {
    let value = optional_config(name)?.unwrap_or_else(|| default.to_owned());
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(format!(
            "{name} must be 1–64 ASCII letters, digits, dots, underscores or hyphens"
        )
        .into());
    }
    Ok(value)
}

fn validate_endpoint(endpoint: &str) -> Result<()> {
    let url = reqwest::Url::parse(endpoint)
        .map_err(|_| "OTEL_EXPORTER_OTLP_ENDPOINT must be a valid URL")?;
    let loopback = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host
                .trim_matches(['[', ']'])
                .parse::<IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("OTEL_EXPORTER_OTLP_ENDPOINT must use HTTPS (HTTP only on loopback), without credentials, query or fragment".into());
    }
    if url.path().trim_end_matches('/').ends_with("/v1/metrics")
        || url.path().trim_end_matches('/').ends_with("/v1/logs")
    {
        return Err(
            "OTEL_EXPORTER_OTLP_ENDPOINT must be the base URL, without a signal suffix".into(),
        );
    }
    Ok(())
}

fn validate_headers(headers: &str) -> Result<()> {
    use axum::http::{HeaderName, HeaderValue};
    if headers.is_empty()
        || headers.split(',').any(|pair| {
            let Some((name, value)) = pair.trim().split_once('=') else {
                return true;
            };
            // The OTLP SDK URL-decodes header values before constructing HTTP headers.
            let Ok(value) = percent_encoding::percent_decode_str(value.trim()).decode_utf8() else {
                return true;
            };
            name.trim().parse::<HeaderName>().is_err()
                || value.is_empty()
                || value.parse::<HeaderValue>().is_err()
        })
    {
        return Err(
            "OTEL_EXPORTER_OTLP_HEADERS must contain comma-separated name=value pairs".into(),
        );
    }
    Ok(())
}

/// Cloneable request middleware state. The default logs locally without recording metrics.
#[derive(Clone, Default)]
pub struct RequestMetrics {
    count: Option<Counter<u64>>,
    duration: Option<Histogram<f64>>,
    classify: Option<RouteOverride>,
}

/// Map site-specific paths (e.g. a catch-all or a stable favicon) to a finite
/// static label. Return `None` to use the router's pattern; never return raw input.
pub type RouteOverride = fn(matched: Option<&str>, path: &str) -> Option<&'static str>;

impl RequestMetrics {
    #[must_use]
    pub fn with_route_override(mut self, classify: RouteOverride) -> Self {
        self.classify = Some(classify);
        self
    }

    fn new(meter: Option<Meter>) -> Self {
        let Some(meter) = meter else {
            return Self::default();
        };
        Self {
            classify: None,
            count: Some(
                meter
                    .u64_counter("http.server.requests")
                    .with_description("Completed HTTP requests")
                    .build(),
            ),
            duration: Some(
                meter
                    .f64_histogram("http.server.request.duration")
                    .with_unit("s")
                    .with_description(
                        "Time to produce response headers, excluding response body transfer",
                    )
                    .with_boundaries(vec![
                        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                    ])
                    .build(),
            ),
        }
    }
}

/// Attach with `Router::layer` after registering all routes and the fallback.
pub async fn observe(
    State(metrics): State<RequestMetrics>,
    request: Request,
    next: Next,
) -> Response {
    let matched = request.extensions().get::<MatchedPath>().cloned();
    let pattern = matched.as_ref().map(MatchedPath::as_str);
    // Deployment probes do not represent user traffic.
    if pattern == Some("/health") {
        return next.run(request).await;
    }
    let route = metrics
        .classify
        .and_then(|classify| classify(pattern, request.uri().path()))
        .unwrap_or_else(|| match pattern {
            Some(path) if path.starts_with("/static/") || path.starts_with("/.well-known/") => {
                "static"
            }
            Some(path) => path,
            None => "unmatched",
        });
    let method = match request.method().as_str() {
        "GET" => "GET",
        "POST" => "POST",
        "HEAD" => "HEAD",
        "PUT" => "PUT",
        "DELETE" => "DELETE",
        "PATCH" => "PATCH",
        "OPTIONS" => "OPTIONS",
        _ => "OTHER",
    };
    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();
    if let (Some(count), Some(histogram)) = (&metrics.count, &metrics.duration) {
        let attrs = [
            KeyValue::new("http.route", route.to_owned()),
            KeyValue::new("http.request.method", method),
            KeyValue::new("http.response.status_code", i64::from(status)),
        ];
        count.add(1, &attrs);
        histogram.record(duration, &attrs);
    }
    if status >= 500 {
        tracing::error!(
            target: REQUEST_TARGET,
            route,
            method,
            status,
            duration_seconds = duration,
            "HTTP request completed"
        );
    } else if status >= 400 {
        tracing::warn!(
            target: REQUEST_TARGET,
            route,
            method,
            status,
            duration_seconds = duration,
            "HTTP request completed"
        );
    } else {
        tracing::info!(
            target: REQUEST_TARGET,
            route,
            method,
            status,
            duration_seconds = duration,
            "HTTP request completed"
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request as HttpRequest, StatusCode},
        middleware,
        routing::get,
    };
    use opentelemetry_sdk::metrics::{
        InMemoryMetricExporter,
        data::{AggregatedMetrics, MetricData},
    };
    use tower::ServiceExt;

    #[test]
    fn endpoint_and_headers_validation_does_not_echo_secrets() {
        for endpoint in [
            "https://example.grafana.net/otlp",
            "http://127.0.0.1:4318",
            "http://[::1]:4318",
        ] {
            assert!(validate_endpoint(endpoint).is_ok());
        }
        for endpoint in [
            "",
            "garbage",
            "http://example.com",
            "https://user:secret@example.com",
            "https://example.com?secret=foo",
            "https://example.com#secret",
            "https://example.com/otlp/v1/logs",
        ] {
            let error = validate_endpoint(endpoint).err().map(|e| e.to_string());
            assert!(error.is_some());
            assert!(!error.unwrap_or_default().contains("secret"));
        }
        assert!(validate_headers("Authorization=Basic abc==").is_ok());
        assert!(validate_headers("Authorization=Basic%20abc==").is_ok());
        for headers in [
            "",
            "Authorization",
            "Authorization=",
            "Authorization=secret\nvalue",
            "Authorization=secret%0Avalue",
            "bad name=secret",
        ] {
            let error = validate_headers(headers).err().map(|e| e.to_string());
            assert!(error.is_some());
            assert!(!error.unwrap_or_default().contains("secret"));
        }
    }

    #[tokio::test]
    async fn records_statuses_and_latency_with_bounded_labels_and_excludes_probes() -> Result<()> {
        let exporter = InMemoryMetricExporter::default();
        let provider = SdkMeterProvider::builder()
            .with_periodic_exporter(exporter.clone())
            .build();
        let metrics = RequestMetrics::new(Some(provider.meter("test")));
        let app = Router::new()
            .route("/items/{id}", get(|| async { "ok" }))
            .route(
                "/form",
                axum::routing::post(|| async { StatusCode::UNPROCESSABLE_ENTITY }),
            )
            .route("/health", get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(metrics.clone(), observe));
        for (method, path, body, status) in [
            ("GET", "/items/private-id?secret=query", "", 200),
            ("POST", "/form", "field=private-invalid", 422),
            ("PUT", "/items/private-id", "", 405),
            ("GET", "/private-missing-one", "", 404),
            ("GET", "/private-missing-two", "", 404),
            ("PRIVATE-METHOD", "/items/private-id", "", 405),
            ("GET", "/health", "", 200),
        ] {
            let response = app
                .clone()
                .oneshot(
                    HttpRequest::builder()
                        .method(method)
                        .uri(path)
                        .header("authorization", "Bearer private-token")
                        .header("content-type", "application/x-www-form-urlencoded")
                        .body(Body::from(body))?,
                )
                .await?;
            assert_eq!(response.status().as_u16(), status);
        }
        let failure = Router::new()
            .route(
                "/items/{id}",
                get(|| async { StatusCode::INTERNAL_SERVER_ERROR }),
            )
            .layer(middleware::from_fn_with_state(metrics, observe));
        failure
            .oneshot(
                HttpRequest::builder()
                    .uri("/items/private-id")
                    .body(Body::empty())?,
            )
            .await?;
        provider.force_flush()?;
        let batches = exporter.get_finished_metrics()?;
        let mut count = 0;
        let mut durations = 0;
        let mut statuses = std::collections::BTreeSet::new();
        for metric in batches
            .iter()
            .flat_map(|batch| batch.scope_metrics())
            .flat_map(|scope| scope.metrics())
        {
            match metric.data() {
                AggregatedMetrics::U64(MetricData::Sum(sum)) => {
                    assert_eq!(metric.name(), "http.server.requests");
                    for point in sum.data_points() {
                        count += point.value();
                        let attrs: Vec<_> = point.attributes().collect();
                        assert_eq!(attrs.len(), 3);
                        for attr in attrs {
                            assert!(!attr.value.to_string().contains("private"));
                            assert!(!attr.value.to_string().contains("secret"));
                            assert_ne!(attr.value.to_string(), "/health");
                            if attr.key.as_str() == "http.response.status_code" {
                                statuses.insert(attr.value.to_string());
                            }
                        }
                    }
                }
                AggregatedMetrics::F64(MetricData::Histogram(histogram)) => {
                    assert_eq!(metric.unit(), "s");
                    for point in histogram.data_points() {
                        durations += point.count();
                        assert!(point.sum() >= 0.0);
                        assert_eq!(point.bounds().count(), 11);
                    }
                }
                _ => return Err("unexpected metric type".into()),
            }
        }
        assert_eq!(count, 7);
        assert_eq!(durations, 7);
        for status in ["200", "404", "405", "422", "500"] {
            assert!(statuses.contains(status));
        }
        provider.shutdown()?;
        Ok(())
    }
}
