# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Poker bankroll Monte Carlo simulator (`br`). Runs N parallel simulations of a bankroll moving through a multi-limit poker ladder with cashout rules, then reports a full percentile table (`final_bankroll`, `cashout`) and probability query results.

## Build & Run

```bash
cargo build                    # debug build
cargo build --release          # release build (thin LTO, codegen-units=1)
cargo run                      # run CLI with default config.toml
cargo run -- -H 50000         # quick smoke test (NUM_SIMULATIONS from .env)
cargo run -- validate          # validate config only
cargo run -- --json            # JSON output
cargo run --bin server         # start HTTP server on :3000
```

No tests exist yet. No linter or formatter config — use `cargo fmt` and `cargo clippy`.

## Architecture

Rust edition 2024, package name `br`. Two binaries share the library crate:

- **`src/bin/cli.rs`** — CLI entry point (`br`). Contains Clap arg definitions (`Cli`, `Command`) and `main`. Reads `.env`, TOML config, runs simulations, prints text or JSON.
- **`src/bin/server.rs`** — Axum HTTP server. `POST /simulate` accepts JSON config body, returns JSON results. Uses `spawn_blocking` to offload CPU work.

Library modules:

- **`.env`** — `NUM_SIMULATIONS=N` (not committed). Read at startup by both binaries via `dotenvy`.
- **`src/core/config.rs`** — `Config`, `LimitConfig`, `CashoutRule`, `CashoutKind`, `ProbabilityQuery` structs + `validate()` + `starting_limit_index()`. Serde-driven (deserializes from both TOML and JSON). Does not contain `num_simulations`.
- **`src/core/simulation.rs`** — `run_simulation()` — single-run hot loop: 100-hand steps using precomputed Normal distributions, limit movement, cashout logic, bust detection. Returns `SimResult` with `final_bankroll`, `total_cashouts`, `net_profit`, `went_bust`, `query_hits`.
- **`src/core/stats.rs`** — `AggregateStats` — stores `runs: Vec<(final_bankroll, total_cashouts)>` sorted by `net_profit`. `report()` returns `SimReport` with 17 coherent percentile pairs. `query_probability()` for hit rates.
- **`src/runner.rs`** — `run_simulations(config, seed, num_simulations)` — orchestrates parallel execution via Rayon `par_iter`. Each sim gets its own `SmallRng` (seeded from `seed XOR i` or OS entropy).
- **`src/output/json.rs`** — Serializes `AggregateStats` into `{simulations, percentiles: {worst…best: {final_bankroll, cashout}}, probability_queries, elapsed_seconds}`.
- **`src/output/text.rs`** — Pretty-prints a condensed percentile table (Worst/P10/P30/P50/P70/P90/Best) and probability queries to stdout.

## Key Design Points

- Simulation granularity is 100-hand steps (not per-hand). `total_hands` and `interval_hands` must be multiples of 100.
- `PrecomputedLimits` pre-builds `Normal<f64>` distributions outside the hot loop to avoid repeated construction.
- Config validation is separate from deserialization — `config::validate()` is called explicitly after parsing.
- The same `Config` struct serves both TOML (CLI) and JSON (server) via Serde.
- Cashout withdrawal is clamped so bankroll never drops below `bust_bankroll`.
