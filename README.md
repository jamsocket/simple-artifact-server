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

## Using it

Generally, you would use this program as an `ENTRYPOINT` of a Docker container. See `examples/streamlit/README.md` for an example that uses Streamlit.

The only required argument is `--command`, specifies a command that is run as the "artifact server". We expect this command to run on port 9090.

The `simple-artifact-server` program listens on 8080 and proxies requests to 9090, unless they have the special `/_frag/*` path prefix, in which case it
handles them itself.
