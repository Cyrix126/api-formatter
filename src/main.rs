use std::{error::Error, net::Ipv4Addr};

use axum::{
    body::to_bytes,
    extract::{Request, State},
    response::IntoResponse,
    Router,
};
use reqwest::{header::CONTENT_LENGTH, Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tracing::{debug, info, warn};

use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature="reduce-big-numbers")] {
const BIG_NUMBER_DIGIT: f64 = 1000000000000.0;
    }
}
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
    let mut headers = rep.headers().to_owned();
    headers.remove(CONTENT_LENGTH);
    let status = rep.status();
    if let Ok(body) = rep.json::<Value>().await {
        debug!("pretty formatting the response: {}", body);
        let pretty_json = format_json_fields_into_readable_output(body);
        (status, headers, pretty_json.to_string()).into_response()
    } else {
        (axum::http::StatusCode::BAD_REQUEST, "API does not have a json output. This proxy formatter supports only API with json output.").into_response()
    }
}

fn format_json_fields_into_readable_output(json: Value) -> Value {
    match json {
        Value::Object(object) => {
            let mut new_object = Map::new();
            for (key, value) in object.into_iter() {
                new_object.insert(key, format_json_fields_into_readable_output(value));
            }
            Value::Object(new_object)
        }
        Value::Array(array) => {
            let mut new_array = Vec::new();
            for item in array.into_iter() {
                new_array.push(format_json_fields_into_readable_output(item));
            }
            Value::Array(new_array)
        }
        Value::Number(n) => {
            if n.is_f64() {
                cfg_if! {
                    if #[cfg(feature="reduce-big-numbers")] {
                let mut nb = n.as_f64().unwrap();
                debug!(" f64 nb was: {}", nb);
                if nb.abs() >= BIG_NUMBER_DIGIT {
                    nb = nb / BIG_NUMBER_DIGIT;
                debug!(" f64 nb is now: {}", nb);
                }
                } else {
                    let nb = n.as_f64().unwrap();
                }
                }
                return Value::String(readable::num::Float::from(nb).to_string());
            }
            if n.is_i64() {
                cfg_if! {
                    if #[cfg(feature="reduce-big-numbers")] {
                let mut nb = n.as_i64().unwrap() as f64;
                debug!(" i64 nb was: {}", nb);
                if nb.abs() as f64 >= BIG_NUMBER_DIGIT {
                    nb = nb / BIG_NUMBER_DIGIT;
                debug!(" i64 nb is now: {}", nb);
                return Value::String(readable::num::Float::from(nb).to_string());
                }
                } else {
                    let nb = n.as_i64().unwrap();
                return Value::String(readable::num::Int::from(nb).to_string());
                }
                }
            }
            if n.is_u64() {
                cfg_if! {
                    if #[cfg(feature="reduce-big-numbers")] {
                let mut nb = n.as_u64().unwrap() as f64;
                debug!("f64 nb was: {}", nb);
                if nb >= BIG_NUMBER_DIGIT {
                    nb = nb / BIG_NUMBER_DIGIT;
                debug!("f64 nb is now: {}", nb);
                return Value::String(readable::num::Float::from(nb).to_string());
                }
                } else {
                    let nb = n.as_u64().unwrap();
                return Value::String(readable::num::Unsigned::from(nb).to_string());
                }
                }
            }
            // all case of Number should be handled but because it's not en enum, need to return here.
            Value::Number(n)
        }
        _ => {
            // no action on other values
            json
        }
    }
}
