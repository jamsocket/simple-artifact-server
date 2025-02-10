use crate::ServerState;
use axum::{
    body::Body,
    extract::{Request, State},
    response::Response,
};
use handlebars::Handlebars;
use http::{uri::Authority, Uri};
use plane_dynamic_proxy::{
    body::{to_simple_body, SimpleBody},
    proxy::ProxyClient,
};
use std::sync::Arc;

const ERROR_TEMPLATE: &str = include_str!("template/error.html");

fn translate_request(mut req: Request<Body>, port: u16) -> http::Request<SimpleBody> {
    let uri = req.uri().clone();

    let mut uri_parts = uri.into_parts();
    uri_parts.scheme = Some("http".parse().unwrap());
    uri_parts.authority =
        Some(Authority::from_maybe_shared(format!("{}:{}", "127.0.0.1", port)).unwrap());
    *req.uri_mut() = Uri::from_parts(uri_parts).unwrap();

    let (parts, body) = req.into_parts();
    let body = to_simple_body(body);

    Request::from_parts(parts, body)
}

pub async fn proxy_request(
    State(server_state): State<Arc<ServerState>>,
    req: Request<Body>,
) -> Response<SimpleBody> {
    if !server_state.wrapped_server.running() {
        let stdout = server_state.wrapped_server.stdout();
        let handlebars = Handlebars::new();
        let rendered = handlebars
            .render_template(ERROR_TEMPLATE, &serde_json::json!({ "stdout": stdout }))
            .unwrap();

        let body = to_simple_body(rendered);

        return Response::builder().status(503).body(body).unwrap();
    }

    let client = ProxyClient::new();
    let req = translate_request(req, server_state.subprocess_port);

    let (response, handler) = client.request(req).await.expect("Infallable");

    if let Some(handler) = handler {
        // proxy websocket connection
        tracing::info!("Proxying websocket connection");
        tokio::spawn(handler.run());
    }

    response
}
