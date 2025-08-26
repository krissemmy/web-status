# Web-status

A tiny Rust web app that shows the latest block number and node latency status from any EVM JSON‑RPC endpoint. Built with Axum 0.7, Reqwest, Tera, and HTMX.

## Features
- `/` : HTML page with live status cards.
    - Block Number Card: fetches the latest block (auto-refreshes every 5s).
    - Node Latency Card: probes the RPC multiple times, computes p50/p95, and displays a green/yellow/red badge based on thresholds.
- `/api/latest-block` : JSON endpoint that calls eth_blockNumber on your RPC and returns the block in hex and integer.
- `/api/node-latency` – JSON endpoint that times several eth_blockNumber calls and reports percentiles + status.

## Stack
- Rust 1.79+ (stable)
- Axum 0.7 (web framework)
- Reqwest (HTTP client)
- Tera (server‑side templates)
- HTMX (progressive interactivity)
- Tokio (async runtime)

## Quick Start
1) Prerequisites
- Install Rust toolchain: https://rustup.rs

cargo install cargo-edit
2) Clone & Enter
```
git clone https://github.com/krissemmy/web-status.git
cd web-status
```

3) Configure Environment
- Create a .env file in the project root:
```bash
ETH_RPC=https://YOUR-RPC-ENDPOINT
CHAIN_NAME=<BLOCKCHAIN_NAME> 
#e.g CHAIN_NAME=ethereum
# ETH_RPC=http://localhost:8545 
```
**Notes:**
- Use a Base/Ethereum/any EVM RPC (local or hosted). No quotes around the URL.
- Example (local): ETH_RPC=http://127.0.0.1:8545

4) Run
```bash
cargo run
```
Open: http://127.0.0.1:3000

5) Verify via cURL

```bash
curl -s http://127.0.0.1:3000/api/latest-block | jq
curl -s http://127.0.0.1:3000/api/node-latency | jq
```

## UI Preview

Block Card auto-refreshes every 5s and shows both hex + int.00
Latency Card auto-refreshes every 7s and shows p50, p95, and a badge:

- ✅ Green (OK): p95 ≤ 300ms
- ⚠️ Yellow (WARN): 300–800ms
- ❌ Red (DOWN): p95 > 800ms or many failures

## Project Structure

```
web-status/
├─ Cargo.toml
├─ .env                   # contains ETH_RPC
└─ src/
   └─ main.rs             # routes, template, RPC call
```

## Common Errors & Fixes

1) Template string compile errors (e.g., expected ';', found '"')
- Ensure the HTML template is a raw string and the delimiters match: r##"..."##;

2) .env not loading
- Ensure dotenvy is included and dotenvy::dotenv().ok(); is called before reading env vars.
- No quotes in .env values.