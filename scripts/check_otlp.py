#!/usr/bin/env python3
"""Exercise the built server using a local OTLP receiver; no cloud credentials."""

import argparse
from contextlib import closing
import http.client
import http.server
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
RECEIVED = []


class Receiver(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        body = self.rfile.read(int(self.headers["Content-Length"]))
        RECEIVED.append((self.path, dict(self.headers), body))
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


def exercise(binary, fulla, settings, invalid=False):
    # Do not inherit any telemetry settings or load the real checkout's .env.
    env = {
        k: v
        for k, v in os.environ.items()
        if not k.startswith(
            ("OTEL_", "RUST_LOG", "DEPLOYMENT_ENVIRONMENT", "HOST", "PORT")
        )
    }
    port = free_port()
    env.update(HOST="127.0.0.1", PORT=str(port), RUST_LOG="info")
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
                for _ in range(100):
                    assert process.poll() is None, "server exited before readiness"
                    try:
                        if fetch(port) == 200:
                            break
                    except OSError:
                        pass
                    time.sleep(0.05)
                else:
                    raise AssertionError("server never became ready")
                route = "/licenses" if fulla else "/items/{id}"
                path = "/licenses" if fulla else "/items/private-input"
                assert fetch(port, path + "?private-input=private-input") == 200
                assert fetch(port, "/private-input") == 404
                if fulla:
                    assert fetch(port, "/site.webmanifest") == 200
                # No interval wait: both signals must flush on graceful shutdown.
                process.send_signal(signal.SIGTERM if fulla else signal.SIGINT)
                out, err = process.communicate(timeout=20)
                assert process.returncode == 0, err
                logs = [json.loads(line) for line in out.splitlines()]
                requests = [
                    log["fields"]
                    for log in logs
                    if log["fields"].get("message") == "HTTP request completed"
                ]
                observed = sorted((row["route"], row["status"]) for row in requests)
                expected = [(route, 200), ("unmatched", 404)]
                if fulla:
                    expected.append(("static", 200))
                assert observed == sorted(expected), requests
            assert "private-input" not in out + err
            assert "dummy-credential" not in out + err
            assert "secret-marker" not in out + err
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fulla",
        type=Path,
        metavar="BINARY",
        help="exercise Fulla using a migrated DATABASE_URL",
    )
    args = parser.parse_args()
    fulla = args.fulla is not None
    binary = (args.fulla or ROOT / "target/debug/examples/telemetry_smoke").resolve()
    service = b"fulla.fyi" if fulla else b"telemetry-smoke"
    receiver = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Receiver)
    worker = threading.Thread(target=receiver.serve_forever, daemon=True)
    worker.start()
    try:
        exercise(binary, fulla, {})
        assert not RECEIVED, "export should be disabled by default"
        exercise(
            binary,
            fulla,
            {
                "OTEL_EXPORTER_OTLP_ENDPOINT": f"http://127.0.0.1:{receiver.server_port}/otlp",
                "OTEL_EXPORTER_OTLP_HEADERS": "Authorization=Basic dummy-credential",
                "DEPLOYMENT_ENVIRONMENT": "staging",
            },
        )
        assert {path for path, _, _ in RECEIVED} == {
            "/otlp/v1/logs",
            "/otlp/v1/metrics",
        }
        for _path, headers, body in RECEIVED:
            lowered = {key.lower(): value for key, value in headers.items()}
            assert lowered["authorization"] == "Basic dummy-credential"
            assert lowered["content-type"] == "application/x-protobuf"
            assert body and body[0] == 10, "expected an OTLP protobuf resource envelope"
            assert service in body and b"staging" in body
            if not fulla:
                assert b"smoke-version" in body
                assert b"local-only event" not in body
            assert b"private-input" not in body and b"dummy-credential" not in body
            assert b"/health" not in body
        metrics = b"".join(
            body for path, _, body in RECEIVED if path.endswith("/metrics")
        )
        logs = b"".join(body for path, _, body in RECEIVED if path.endswith("/logs"))
        assert (
            b"http.server.requests" in metrics
            and b"http.server.request.duration" in metrics
        )
        assert b"HTTP request completed" in logs
        if not fulla:
            assert b"allowed application event" in logs
        exercise(
            binary,
            fulla,
            {
                "OTEL_EXPORTER_OTLP_ENDPOINT": "https://secret-marker:password@example.com"
            },
            invalid=True,
        )
        exercise(
            binary,
            fulla,
            {"OTEL_EXPORTER_OTLP_ENDPOINT": f"http://127.0.0.1:{free_port()}/otlp"},
        )
        print(
            "OTLP smoke checks passed: disabled, logs/metrics wire export, auth, privacy, shutdown, invalid config, receiver outage"
        )
    finally:
        receiver.shutdown()
        receiver.server_close()
