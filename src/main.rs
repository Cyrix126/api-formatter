use std::{error::Error, net::Ipv4Addr};

use axum::{
    body::to_bytes,
    extract::{Request, State},
    response::IntoResponse,
    Router,
};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, warn};
#[derive(Serialize, Deserialize)]
struct Config {
    listen_address: Ipv4Addr,
    listen_port: u16,
    api_url: Url,
}

#[derive(Clone)]
struct AppState {
    client: Client,
    base_uri: Url,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_address: Ipv4Addr::new(127, 0, 0, 1),
            listen_port: 8080,
            api_url: Url::parse("https://127.0.0.1:8081").expect(
                "the url provided for api_url in the configuration file does not sem to be valid.",
            ),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    info!("reading configuration file...");
    let config: Config = confy::load_path("/etc/api-formatter/config.toml").expect("check that the directory /etc/api-formatter exist and that the program is ran with sufficient permission");
    info!("creating client");
    let state = AppState {
        client: Client::new(),
        base_uri: config.api_url.to_owned(),
    };
    let route = Router::new().fallback(handler).with_state(state);
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.listen_address, config.listen_port))
            .await?;
    info!(
        "Listening to {}:{}",
        config.listen_address, config.listen_port
    );
    axum::serve(listener, route.into_make_service()).await?;
    Ok(())
}

async fn handler(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    info!("new request ! ");
    debug!("The request received {:?}", request);
    let uri = format!("{}{}", state.base_uri, request.uri()).replace("//", "/");
    debug!("Url to request: {uri}");
    let req = state
        .client
        .request(request.method().to_owned(), uri)
        .headers(request.headers().to_owned())
        .body(to_bytes(request.into_body(), usize::MAX).await.unwrap())
        .send()
        .await;
    debug!("The request that will be make by the proxy {:?}", req);
    match req {
        Ok(rep) => format_resp(rep).await.into_response(),
        Err(err) => {
            warn!("request had an error: {}", err);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                err.to_string(),
            )
                .into_response()
        }
    }
}

async fn format_resp(rep: reqwest::Response) -> impl IntoResponse {
    let headers = rep.headers().to_owned();
    let status = rep.status();
    if let Ok(mut body) = rep.json::<Value>().await {
        debug!("pretty formatting the response: {}", body);
        format_json_fields_into_readable_output(&mut body);
        (status, headers, body.to_string()).into_response()
    } else {
        (axum::http::StatusCode::BAD_REQUEST, "API does not have a json output. This proxy formatter supports only API with json output.").into_response()
    }
}

fn format_json_fields_into_readable_output(json: &mut Value) {
    match json {
        Value::Object(object) => {
            for (_key, value) in object {
                format_json_fields_into_readable_output(value); // Recursive call to handle nested objects
            }
        }
        Value::Array(array) => {
            for item in array {
                format_json_fields_into_readable_output(item); // Recursive call to handle nested objects
            }
        }
        Value::Number(n) => {
            if n.is_i64() {
                *n = readable::num::Int::from(n.as_i64().unwrap()).inner().into();
            }
            if n.is_u64() {
                *n = readable::num::Unsigned::from(n.as_u64().unwrap())
                    .inner()
                    .into();
            }
            if n.is_f64() {
                *n = serde_json::Number::from_f64(
                    readable::num::Float::from(n.as_f64().unwrap()).inner(),
                )
                .expect("NAN or INFINITY should not be present in json Number");
            }
        }
        _ => {
            // no action on other values
        }
    }
}
