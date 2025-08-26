use axum::{routing::{get}, Router, extract::State, Json};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tera::{Tera, Context};
use axum::response::Html;
use reqwest::Client;
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    tera: Arc<Tera>,
    http: Client,
    rpc_url: String,
}

#[derive(Serialize, Deserialize)]
struct JsonRpcReq<'a> {
    jsonrpc: &'a str,
    method: &'a str,
    params: Vec<serde_json::Value>,
    id: u32,
}

#[derive(Serialize, Deserialize)]
struct JsonRpcResp {
    jsonrpc: String,
    id: u32,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    dotenvy::dotenv().ok();
    let rpc_url = std::env::var("ETH_RPC").unwrap_or_else(|_| "http://127.0.0.1:8545".into());

    let mut tera = Tera::default();
    tera.add_raw_template("index.html", INDEX_HTML).expect("template");
    let state = AppState {
        tera: Arc::new(tera),
        http: Client::new(),
        rpc_url,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/latest-block", get(latest_block))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

fn hex_to_u64(hex: &str) -> Result<u64, std::num::ParseIntError> {
    let trimmed = hex.trim_start_matches("0x");
    u64::from_str_radix(trimmed, 16)
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let mut ctx = Context::new();
    ctx.insert("title", "Web3 Node Current Block Status");
    let html = state.tera.render("index.html", &ctx).unwrap();
    Html(html)
}

async fn latest_block(State(state): State<AppState>) -> Json<serde_json::Value> {
    let body = JsonRpcReq {
        jsonrpc: "2.0",
        method: "eth_blockNumber",
        params: vec![],
        id: 1,
    };

    let resp: JsonRpcResp = state.http.post(&state.rpc_url)
        .json(&body)
        .send().await.unwrap()
        .json().await.unwrap();

    let hex = resp.result.unwrap_or(serde_json::Value::String("0x0".into()));
    let block_str = hex.as_str().unwrap_or("0x0");
    let block_num = hex_to_u64(block_str).unwrap_or(0);

    Json(serde_json::json!({
        "blockNumber": block_num
    }))
}


const INDEX_HTML: &str = r##"<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <title>{{ title }}</title>
  <script src="https://unpkg.com/htmx.org@1.9.12"></script>
  <style>
    body { font-family: system-ui, -apple-system, Segoe UI, Roboto, Arial; margin: 2rem; }
    .card { max-width: 640px; padding: 1rem 1.5rem; border: 1px solid #ddd; border-radius: 12px; }
    .btn { padding: .5rem 1rem; border-radius: 8px; border: 1px solid #ccc; cursor: pointer; background: #f8f8f8; }
    .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
  </style>
</head>
<body>
  <h1>{{ title }}</h1>
  <div class="card">
    <p>Click to fetch the latest Base block from your ETH_RPC.</p>
    <button class="btn"
      hx-get="/api/latest-block"
      hx-trigger="click"
      hx-target="#out"
      hx-swap="innerHTML">Get Latest Block</button>
    <pre id="out" class="mono" style="margin-top: 1rem;">(waiting)</pre>
  </div>
</body>
</html>
"##;
