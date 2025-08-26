# Web-status

A tiny Rust web app that shows the latest block number from any EVM JSON‑RPC endpoint. Built with Axum 0.7, Reqwest, Tera, and HTMX.

## Features
- `/` : HTML page with a button that fetches the latest block via HTMX (no full page reload).
- `/api/latest-block` : JSON endpoint that calls eth_blockNumber on your RPC and returns the block in hex and integer.
- Minimal, readable code you can extend: latency probes, health checks, simple dashboards.

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
```
ETH_RPC=https://YOUR-RPC-ENDPOINT
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
# {
#   "blockNumberHex": "0x14c8e327",
#   "blockNumber": 348154151
# }
```

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