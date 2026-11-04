# Deposits – account service (baseline)

The Deposits domain's account service for the agentic SDLC demo (Rust, axum). It stands in for the Deposits
integration and core systems: current accounts of the (fictional) demo customers.

- `GET /accounts/{account_id}`: account with balance and transactions (demo data in `api/src/accounts.rs`).
- Run: `cd api && cargo run` (port 8080, or `PORT=…`). Test: `cargo test`.
- Contracts are owned by the architecture repository of the run (`interfaces/`); low-level designs go to `design/`.
