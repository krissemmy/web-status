use axum::{routing::{get}, Router, extract::State, Json};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc, time::Instant};
use tera::{Tera, Context};
use axum::response::Html;
use reqwest::Client;
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    tera: Arc<Tera>,
    http: Client,
    rpc_url: String,
    chain_name: String,
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

#[derive(Serialize)]
struct LatencyResp {
    p50_ms: f64,
    p95_ms: f64,
    samples: usize,
    status: &'static str, // "ok" | "warn" | "down"
}

async fn rpc_block_number(client: &reqwest::Client, rpc_url: &str) -> Result<(), reqwest::Error> {
    let body = JsonRpcReq {
        jsonrpc: "2.0",
        method: "eth_blockNumber",
        params: vec![],
        id: 1,
    };
    client.post(rpc_url).json(&body).send().await?.error_for_status()?;
    Ok(())
}

fn percentile(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if v.is_empty() { return f64::NAN; }
    let idx = ((p * (v.len() as f64 - 1.0)).round() as usize).min(v.len() - 1);
    v[idx]
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
    let chain_name = std::env::var("CHAIN_NAME").unwrap_or_else(|_| "unknown".into());

    let state = AppState {
        tera: Arc::new(tera),
        http: Client::new(),
        rpc_url,
        chain_name,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/latest-block", get(latest_block))
        .route("/api/node-latency", get(node_latency))
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

async fn node_latency(State(state): State<AppState>) -> Json<LatencyResp> {
    // do N serial calls; simple & stable
    let n = 7usize;
    let mut samples = Vec::with_capacity(n);
    for _ in 0..n {
        let t0 = Instant::now();
        // ignore individual errors; treat as slow/down
        let ok = rpc_block_number(&state.http, &state.rpc_url).await.is_ok();
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        // if failed, record a large sentinel (3s)
        samples.push(if ok { ms } else { 3000.0 });
    }

    let mut s = samples.clone();
    let p50 = percentile(&mut s, 0.50);
    let p95 = percentile(&mut s, 0.95);

    // thresholds (tune as you like)
    // ok:   p95 <= 300ms
    // warn: 300ms < p95 <= 800ms
    // down: p95  > 800ms (or many failures)
    let status = if p95.is_nan() {
        "down"
    } else if p95 <= 300.0 {
        "ok"
    } else if p95 <= 800.0 {
        "warn"
    } else {
        "down"
    };

    Json(LatencyResp { p50_ms: p50, p95_ms: p95, samples: n, status })
}


async fn index(State(state): State<AppState>) -> Html<String> {
    let mut ctx = Context::new();
    ctx.insert("title", "Web3 Node Current Block Status");
    ctx.insert("chain_name", &state.chain_name);
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
    "blockNumberHex": block_str,
    "blockNumber": block_num,
    "chain": state.chain_name,
    }))
}


const INDEX_HTML: &str = r##"<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <title>Web3 Node Current Block Status</title>
  <script src="https://unpkg.com/htmx.org@1.9.12"></script>
  <style>
    body { font-family: system-ui, -apple-system, Segoe UI, Roboto, Arial; margin: 2rem; }
    .card { max-width: 720px; padding: 1rem 1.5rem; border: 1px solid #e5e7eb; border-radius: 12px; background: #fff; margin-bottom: 1rem; }
    .btn { padding: .6rem 1rem; border-radius: 8px; border: 1px solid #d1d5db; cursor: pointer; background: #f9fafb; }
    .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
    .row { display: flex; align-items: center; gap: .75rem; flex-wrap: wrap; }
    .badge { display: inline-block; padding: .25rem .6rem; border-radius: 999px; font-size: .85rem; border: 1px solid transparent; }
    .ok { background:#ecfdf5; color:#065f46; border-color:#a7f3d0; }     /* green */
    .warn { background:#fffbeb; color:#92400e; border-color:#fde68a; }   /* yellow */
    .down { background:#fef2f2; color:#991b1b; border-color:#fecaca; }   /* red */
    small { color:#6b7280; }
  </style>
</head>
<body>
  <h1>{{ title }} ({{ chain_name }})</h1>

  <!-- Block Number Card -->
  <div class="card">
    <p>Click or wait — auto-refreshes every 15s from your ETH_RPC.</p>

    <!-- auto poller -->
    <div
      hx-get="/api/latest-block"
      hx-trigger="load, every 15s"
      hx-target="#out"
      hx-swap="innerHTML">
    </div>

    <button class="btn"
      hx-get="/api/latest-block"
      hx-target="#out"
      hx-swap="innerHTML">
      Get Latest Block
    </button>

    <pre id="out" class="mono" style="margin-top: 1rem;">(loading…)</pre>
    <small id="ts"></small>
  </div>

  <!-- Latency Card -->
  <div class="card">
    <div class="row" style="justify-content: space-between;">
      <div class="row">
        <strong>Node Latency</strong>
        <span id="status-badge" class="badge down">checking…</span>
      </div>
      <button class="btn"
        hx-get="/api/node-latency"
        hx-target="#latency-json"
        hx-swap="innerHTML">
        Probe Now
      </button>
    </div>

    <!-- auto poll latency every 7s -->
    <div
      hx-get="/api/node-latency"
      hx-trigger="load, every 7s"
      hx-target="#latency-json"
      hx-swap="innerHTML">
    </div>

    <!-- raw JSON lands here (hidden); script parses and renders pretty text -->
    <pre id="latency-json" class="mono" style="display:none;"></pre>

    <div class="mono" style="margin-top: .75rem;">
      p50: <span id="lat-p50">—</span> ms,
      p95: <span id="lat-p95">—</span> ms
    </div>
    <small id="lat-ts"></small>
  </div>

  <script>
    // Update timestamp when latest-block swaps
    document.body.addEventListener('htmx:afterSwap', function (evt) {
      if (evt.target && evt.target.id === 'out') {
        document.getElementById('ts').textContent =
          'Last updated: ' + new Date().toLocaleTimeString();
      }
    });

    // Parse latency JSON and update UI
    document.body.addEventListener('htmx:afterOnLoad', function (evt) {
      try {
        const url = evt.detail.xhr.responseURL || '';
        if (!url.includes('/api/node-latency')) return;

        const data = JSON.parse(evt.detail.xhr.responseText);
        const p50 = (data.p50_ms ?? NaN).toFixed(1);
        const p95 = (data.p95_ms ?? NaN).toFixed(1);
        const status = data.status || 'down';

        document.getElementById('lat-p50').textContent = p50;
        document.getElementById('lat-p95').textContent = p95;

        const badge = document.getElementById('status-badge');
        badge.classList.remove('ok', 'warn', 'down');
        badge.classList.add(status);
        badge.textContent = (status === 'ok' ? 'OK' : status === 'warn' ? 'WARN' : 'DOWN');

        document.getElementById('lat-ts').textContent =
          'Last probe: ' + new Date().toLocaleTimeString();
      } catch (e) {
        // ignore parse errors
      }
    });
  </script>
</body>
</html>
"##;
