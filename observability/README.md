# Site HTTP telemetry

Each site can send application logs and request metrics directly to the same
Grafana Cloud stack over OTLP HTTP/protobuf. There is no public metrics endpoint,
collector, or distributed trace exporter. Without an OTLP endpoint, the server
still writes JSON logs to stdout.

## Connect a service

In Grafana Cloud, open **OpenTelemetry → Configure** on the destination stack.
Use the supplied base endpoint and authorization header. Give the token
`logs:write` and `metrics:write`; keep it in the deployment's service variables.
Use a separate token per service/environment so they can be rotated independently.

| Service variable | Value |
| --- | --- |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Stack base URL, e.g. `https://<stack-host>/otlp`. Leave unset for stdout only. Do not append `/v1/logs` or `/v1/metrics`. |
| `OTEL_EXPORTER_OTLP_HEADERS` | Exact generated `Authorization=Basic …` value; optional for a loopback receiver. |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `http/protobuf` (default). Both signals use the same endpoint and headers. |
| `OTEL_SERVICE_NAME` | Optional override of the name supplied by the binary. Keep it stable across releases and environments. |
| `DEPLOYMENT_ENVIRONMENT` | `staging` or `production` on Railway; defaults to `development`. |
| `RUST_LOG` | Defaults to `info`. `warn` suppresses successful request logs but does not disable metrics. |

Export requires HTTPS except on loopback. Resources contain the service name,
application version, random per-process instance ID, and environment. Requests
do not wait for network export. Allow time on shutdown for the server to drain
and the SDK to flush; delivery is best effort during an outage.

## Dashboard

Import [grafana-dashboard.json](grafana-dashboard.json) once and select the
stack's Prometheus/Mimir and application Loki data sources. Select **Service**
from the dropdown; it lists jobs that have emitted `http_server_requests_total`.
**Environment** defaults to `.*`. Generate a non-health request and allow two
metric export intervals (normally about two minutes) for the rate panels.

The dashboard uses the same `http.server.requests` counter and
`http.server.request.duration` histogram (seconds, to response headers) as
Valorant Randomizer. The only request dimensions are route pattern, method,
and status. Probe traffic is excluded; assets and errors are included. For a
site with catch-all or stable asset routes, use `RequestMetrics::with_route_override`
to return a fixed category such as `static` or `unmatched`. Never use slugs,
share tokens, query strings, or other client input as labels. Additional
application log targets are selected by the caller; request middleware never
exports bodies, headers, or client addresses.

If data is missing, check `http_server_requests_total{job="fulla.fyi"}` in
Metrics Explore and `{service_name="fulla.fyi"}` in Logs Explore (substitute
your service). Keep Environment at `.*` initially. A stack that does not
promote `deployment.environment.name` on metrics may need a `target_info` join
for exact environment filtering. In Loki the environment can be structured
metadata rather than an indexed label.

## Verify locally

Install the OTLP protobuf bindings in a Python virtual environment
(`python3 -m pip install opentelemetry-proto`), or use the FreeBSD package
`devel/py-opentelemetry-proto`. From the library checkout:

```sh
cargo build --locked -p nine_dev_site_lib --features telemetry --example telemetry_smoke
python3 scripts/check_otlp.py
```

The smoke test sends two rounds of GETs and decodes what reaches a loopback OTLP
receiver. The request counter and latency histogram count must both increase
while the server is running. It also checks the exported request logs, route
labels, health-probe exclusion, and final flush on shutdown. It uses dummy
headers and does not need a Grafana account.

To check Fulla, build the binary, export `DATABASE_URL` for a disposable
PostgreSQL database, apply migrations with `fulla_fyi migrate`, and run
`python3 scripts/check_otlp.py --fulla /path/to/fulla_fyi`.
For another server using the shared telemetry, supply its default service name
and one `--get PATH ROUTE_LABEL STATUS` per request. For example, against an
Ixirsii binary and an empty, migrated database:

```sh
python3 scripts/check_otlp.py --server /path/to/ixirsii-tech --service ixirsii.tech \
  --get /blog /blog 200 --get /blog/private-input '/blog/{slug}' 404 \
  --get /site.webmanifest static 200
```

The dashboard and smoke test are adapted from Valorant Randomizer, copyright
2025 Ryan Porterfield, under the [BSD-3-Clause license](../crates/nine_dev_site_lib/LICENSE-telemetry).
