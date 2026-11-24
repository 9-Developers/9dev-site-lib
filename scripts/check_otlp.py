"""Exercise the built server using a local OTLP receiver; no cloud credentials."""

import argparse
import http.client
import http.server
import json
import os
import signal
import socket
import subprocess
import tempfile
import threading
import time
from collections import Counter
from contextlib import closing
from pathlib import Path

from opentelemetry.proto.collector.logs.v1.logs_service_pb2 import (
    ExportLogsServiceRequest,
)
from opentelemetry.proto.collector.metrics.v1.metrics_service_pb2 import (
    ExportMetricsServiceRequest,
)
from opentelemetry.proto.metrics.v1.metrics_pb2 import (
    AGGREGATION_TEMPORALITY_CUMULATIVE,
)

ROOT = Path(__file__).resolve().parents[1]
RECEIVED = []
ARRIVAL = threading.Condition()


class Receiver(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        body = self.rfile.read(int(self.headers["Content-Length"]))
        with ARRIVAL:
            RECEIVED.append((self.path, dict(self.headers), body))
            ARRIVAL.notify_all()
        self.send_response(200)
        self.send_header("Content-Type", "application/x-protobuf")
        self.send_header("Content-Length", "0")
        self.end_headers()

    def log_message(self, *_):
        pass


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def fetch(port, path="/health"):
    # Connect directly to loopback without proxy or redirect handling.
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=1)
    with closing(connection):
        connection.request(
            "GET",
            path,
            headers={
                "Authorization": "Bearer private-input",
                "Cookie": "private-input",
            },
        )
        with connection.getresponse() as response:
            return response.status


def attributes(items):
    return {
        item.key: getattr(item.value, item.value.WhichOneof("value")) for item in items
    }


def check_resource(resource, service):
    labels = attributes(resource.attributes)
    assert labels["service.name"] == service, labels["service.name"]
    assert labels["deployment.environment.name"] == "staging"


def read_metrics(body, service, counts, durations):
    for group in ExportMetricsServiceRequest.FromString(body).resource_metrics:
        check_resource(group.resource, service)
        for scope in group.scope_metrics:
            for metric in scope.metrics:
                if metric.name == "http.server.requests":
                    data = metric.sum
                    assert data.is_monotonic, "expected a monotonic request counter"
                elif metric.name == "http.server.request.duration":
                    data = metric.histogram
                    assert metric.unit == "s", metric.unit
                else:
                    continue
                assert (
                    data.aggregation_temporality == AGGREGATION_TEMPORALITY_CUMULATIVE
                )
                for point in data.data_points:
                    labels = attributes(point.attributes)
                    assert labels.keys() == {
                        "http.route",
                        "http.request.method",
                        "http.response.status_code",
                    }, labels.keys()
                    key = (
                        labels["http.route"],
                        labels["http.request.method"],
                        labels["http.response.status_code"],
                    )
                    if metric.name == "http.server.requests":
                        assert point.WhichOneof("value") == "as_int"
                        counts[key] = point.as_int
                    else:
                        assert (
                            point.explicit_bounds
                            and sum(point.bucket_counts) == point.count
                        )
                        assert not point.count or point.sum > 0, (
                            f"no measured latency for {key}"
                        )
                        durations[key] = point.count


def expect_metrics(service, expected):
    # Periodic exports are cumulative. Use the latest value per series, never
    # add successive exports together (a stuck counter would then look healthy).
    counts, durations = {}, {}
    offset = 0

    def matches():
        nonlocal offset
        for path, _, body in RECEIVED[offset:]:
            if path == "/otlp/v1/metrics":
                read_metrics(body, service, counts, durations)
        offset = len(RECEIVED)
        return counts == expected and durations == expected

    with ARRIVAL:
        if not ARRIVAL.wait_for(matches, timeout=10):
            raise AssertionError(
                f"{service}: expected {dict(expected)}; got counters {counts}, histogram counts {durations}"
            )


def request_logs(body, service):
    for group in ExportLogsServiceRequest.FromString(body).resource_logs:
        check_resource(group.resource, service)
        for scope in group.scope_logs:
            for record in scope.log_records:
                if record.body.string_value == "HTTP request completed":
                    fields = attributes(record.attributes)
                    yield fields["route"], fields["method"], fields["status"]


def exercise(
    binary, traffic, stop_signal, settings, *, live_service=None, invalid=False
):
    # Do not inherit any telemetry settings or load the real checkout's .env.
    env = {
        k: v
        for k, v in os.environ.items()
        if not k.startswith(
            ("OTEL_", "RUST_LOG", "DEPLOYMENT_ENVIRONMENT", "HOST", "PORT")
        )
    }
    port = free_port()
    env.update(
        HOST="127.0.0.1",
        PORT=str(port),
        RUST_LOG="info",
        NO_PROXY="127.0.0.1",
        no_proxy="127.0.0.1",
    )
    env.update(settings)
    with tempfile.TemporaryDirectory() as cwd:
        process = subprocess.Popen(
            [str(binary)],
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            if invalid:
                out, err = process.communicate(timeout=10)
                assert process.returncode != 0
                assert "OTEL_EXPORTER_OTLP_ENDPOINT" in err
            else:
                deadline = time.monotonic() + 15
                while time.monotonic() < deadline:
                    if process.poll() is not None:
                        _, err = process.communicate()
                        raise AssertionError(
                            f"{binary.name} exited before readiness: {err.strip()}"
                        )
                    try:
                        if fetch(port) == 200:
                            break
                    except OSError:
                        pass
                    time.sleep(0.05)
                else:
                    raise AssertionError("server never became ready")
                expected = Counter()

                def send(cases):
                    for path, route, status in cases:
                        actual = fetch(port, path)
                        assert actual == status, (path, actual, status)
                        expected[route, "GET", status] += 1

                if live_service:
                    for _ in range(2):
                        send(traffic)
                        expect_metrics(live_service, expected)
                    # Leave one more observation for graceful shutdown to flush.
                    send(traffic[:1])
                else:
                    send(traffic)
                process.send_signal(stop_signal)
                out, err = process.communicate(timeout=20)
                assert process.returncode == 0, err
                logs = [json.loads(line) for line in out.splitlines()]
                requests = [
                    log["fields"]
                    for log in logs
                    if log["fields"].get("message") == "HTTP request completed"
                ]
                observed = Counter(
                    (row["route"], row["method"], row["status"]) for row in requests
                )
                assert observed == expected, (observed, expected)
                if live_service:
                    expect_metrics(live_service, expected)
            assert "private-input" not in out + err
            assert "dummy-credential" not in out + err
            assert "secret-marker" not in out + err
            if not invalid:
                return expected
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    target = parser.add_mutually_exclusive_group()
    target.add_argument(
        "--fulla",
        type=Path,
        metavar="BINARY",
        help="exercise Fulla using a migrated DATABASE_URL",
    )
    target.add_argument(
        "--server", type=Path, metavar="BINARY", help="exercise a site's own GET cases"
    )
    parser.add_argument("--service", help="expected default service.name for --server")
    parser.add_argument(
        "--get", nargs=3, action="append", metavar=("PATH", "ROUTE_LABEL", "STATUS")
    )
    args = parser.parse_args()
    if args.server:
        if not args.service or not args.get:
            parser.error(
                "--server requires --service and at least one --get PATH ROUTE_LABEL STATUS"
            )
        service = args.service
        traffic = [(path, label, int(status)) for path, label, status in args.get]
    else:
        if args.get or args.service:
            parser.error("--get and --service require --server")
        service = "fulla.fyi" if args.fulla else "telemetry-smoke"
        path, route = (
            ("/licenses", "/licenses")
            if args.fulla
            else ("/items/private-input", "/items/{id}")
        )
        traffic = [
            (path + "?private-input=private-input", route, 200),
            ("/private-input", "unmatched", 404),
        ]
        if args.fulla:
            traffic.append(("/site.webmanifest", "static", 200))
    example = not (args.server or args.fulla)
    binary = (
        args.server or args.fulla or ROOT / "target/debug/examples/telemetry_smoke"
    ).resolve()
    stop_signal = signal.SIGINT if example else signal.SIGTERM
    receiver = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Receiver)
    worker = threading.Thread(target=receiver.serve_forever, daemon=True)
    worker.start()
    try:
        exercise(binary, traffic, stop_signal, {})
        assert not RECEIVED, "export should be disabled by default"
        expected = exercise(
            binary,
            traffic,
            stop_signal,
            {
                "OTEL_EXPORTER_OTLP_ENDPOINT": f"http://127.0.0.1:{receiver.server_port}/otlp",
                "OTEL_EXPORTER_OTLP_HEADERS": "Authorization=Basic dummy-credential",
                "DEPLOYMENT_ENVIRONMENT": "staging",
                "OTEL_METRIC_EXPORT_INTERVAL": "250",
                "OTEL_BLRP_SCHEDULE_DELAY": "60000",
            },
            live_service=service,
        )
        assert {path for path, _, _ in RECEIVED} == {
            "/otlp/v1/logs",
            "/otlp/v1/metrics",
        }
        exported_requests = Counter()
        for path, headers, body in RECEIVED:
            lowered = {key.lower(): value for key, value in headers.items()}
            assert lowered["authorization"] == "Basic dummy-credential"
            assert lowered["content-type"] == "application/x-protobuf"
            assert body and body[0] == 10, "expected an OTLP protobuf resource envelope"
            if path == "/otlp/v1/logs":
                exported_requests.update(request_logs(body, service))
            if example:
                assert b"smoke-version" in body
                assert b"local-only event" not in body
            assert b"private-input" not in body and b"dummy-credential" not in body
            assert b"/health" not in body
        assert exported_requests == expected, (exported_requests, expected)
        if example:
            logs = b"".join(
                body for path, _, body in RECEIVED if path.endswith("/logs")
            )
            assert b"allowed application event" in logs
        exercise(
            binary,
            traffic,
            stop_signal,
            {
                "OTEL_EXPORTER_OTLP_ENDPOINT": "https://secret-marker:password@example.com"
            },
            invalid=True,
        )
        exercise(
            binary,
            traffic,
            stop_signal,
            {"OTEL_EXPORTER_OTLP_ENDPOINT": f"http://127.0.0.1:{free_port()}/otlp"},
        )
        print(
            f"{service}: OTLP smoke checks passed (live counter and latency growth, request logs, probe exclusion, "
            "auth, privacy, shutdown, disabled export, invalid config, receiver outage)"
        )
    finally:
        receiver.shutdown()
        receiver.server_close()
