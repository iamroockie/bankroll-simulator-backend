# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Poker bankroll Monte Carlo simulator (`br`). Runs N parallel simulations of a bankroll moving through a multi-limit poker ladder with cashout rules, then reports confidence intervals, percentiles, and probability query results.

## Build & Run

```bash
cargo build                    # debug build
cargo build --release          # release build (thin LTO, codegen-units=1)
cargo run                      # run CLI with default config.toml
cargo run -- -s 1000 -H 50000 # quick smoke test
cargo run -- validate          # validate config only
cargo run -- --json            # JSON output
cargo run --bin server         # start HTTP server on :3000
```

No tests exist yet. No linter or formatter config — use `cargo fmt` and `cargo clippy`.

## Architecture

Rust edition 2024, package name `br`. Two binaries share the library crate:

- **`src/bin/cli.rs`** — CLI entry point (`br`). Reads TOML config, runs simulations, prints text or JSON.
- **`src/bin/server.rs`** — Axum HTTP server. `POST /simulate` accepts JSON config body, returns JSON results. Uses `spawn_blocking` to offload CPU work.

Library modules:

- **`src/cli.rs`** — Clap argument definitions (`Cli`, `Command`).
- **`src/core/config.rs`** — `Config`, `LimitConfig`, `CashoutRule`, `CashoutKind`, `ProbabilityQuery` structs + `validate()` + `starting_limit_index()`. Serde-driven (deserializes from both TOML and JSON).
- **`src/core/simulation.rs`** — `run_simulation()` — single-run hot loop: 100-hand steps using precomputed Normal distributions, limit movement, cashout logic, bust detection. Returns `SimResult`.
- **`src/core/stats.rs`** — `AggregateStats` — collects and sorts net profits from all runs, computes percentiles and query hit rates.
- **`src/runner.rs`** — `run_simulations()` — orchestrates parallel execution via Rayon `par_iter`. Each sim gets its own `SmallRng` (seeded from `seed XOR i` or OS entropy).
- **`src/output/json.rs`** — Serializes `AggregateStats` into the JSON response shape (CIs, percentiles, query results).
- **`src/output/text.rs`** — Pretty-prints results to stdout.

## Key Design Points

- Simulation granularity is 100-hand steps (not per-hand). `total_hands` and `interval_hands` must be multiples of 100.
- `PrecomputedLimits` pre-builds `Normal<f64>` distributions outside the hot loop to avoid repeated construction.
- Config validation is separate from deserialization — `config::validate()` is called explicitly after parsing.
- The same `Config` struct serves both TOML (CLI) and JSON (server) via Serde.
- Cashout withdrawal is clamped so bankroll never drops below `bust_bankroll`.
