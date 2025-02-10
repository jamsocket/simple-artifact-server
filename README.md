# Jamsocket simple artifact server

This is a simple web server that wraps another web server and proxies traffic to it. This server also
provides endpoints that allow files to be uploaded, and to allow the “wrapped” web server process to
be interrupted or restarted.

This server is designed to run on [Jamsocket](https://jamsocket.com) or [Plane](https://plane.dev).

This provides a generalized way to build “fragment” or “artifact” functionality, where an LLM generates
some code that is run as input to a process that runs in a sandboxed environment.

## API

Get server status:

```
GET /_frag/status
```

Upload a file:

```
POST /_frag/upload/:filename?restart={true|false}&interrupt={true|false}
```

Restart the wrapped server:

```
POST /_frag/restart
```

Interrupt the wrapped server:

```
POST /_frag/interrupt
```

All requests that do not have the `/_frag/` path prefix will be proxied to the wrapped server.

## Auth

If the `x-verified-user-data` header is provided (e.g. by passing `auth` to the connect endpoint in Jamsocket),
it is parsed as JSON. If the JSON contains a field `readOnly` with value `true`, access is allowed to the
wrapped server but not the `/_frag/` endpoints.

## Running locally

To run the server:

```bash
cargo build
mkdir scratch
cd scratch
../target/debug/simple-artifact-server --command "python3 -m http.server 9090"
```

Then, to send a file:

```bash
curl -X POST --data-binary "@README.md" http://localhost:8080/_frag/upload/README.md
```

Then, access the file:

```bash
curl http://localhost:8080/README.md
```
