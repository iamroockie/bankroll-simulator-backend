# Poker Bankroll Simulator

Monte Carlo bankroll simulator with multi-limit movement, cashouts, and custom probability queries.

## Features

- Multi-limit ladder with automatic move-up/move-down thresholds
- Per-limit EV and standard deviation settings
- Cashout rules: fixed amount, % of profit since last cashout, % of current bankroll
- Probability queries: bust chance, reaching a profit target, reaching a bankroll target, finishing at a given limit or above
- CLI binary (`br`) + HTTP server binary (`server`)

## CLI Usage

```
br [OPTIONS] [SUBCOMMAND]

OPTIONS:
  -c, --config <FILE>      Path to TOML config [default: config.toml]
  -s, --simulations <N>    Override num_simulations from config
  -H, --hands <N>          Override total_hands from config
      --seed <N>           Fixed seed for reproducible runs
  -q, --quiet              Suppress progress line
  -j, --json               Output results as JSON

SUBCOMMANDS:
  validate                 Validate config without running simulations
```

### Examples

```bash
# Run with default config
br

# Quick smoke test
br -s 1000 -H 50000

# Validate config
br validate -c config.toml

# JSON output
br --json | jq .
```

## Config Format

```toml
starting_bankroll = 3_000
bust_bankroll     = 1_000           # bust if bankroll drops below this
num_simulations   = 10_000          # between 1 and 100_000
total_hands       = 1_000_000

[[limits]]                          # list in ascending bb_size order
name             = "NL25"
bb_size          = 0.25             # big blind size in dollars
ev_per_100       = 8                # expected value in BB/100
std_dev_per_100  = 90               # standard deviation in BB/100
move_up_at       = 2_500            # move up when bankroll >= this

[[limits]]
name             = "NL50"
bb_size          = 0.50
ev_per_100       = 7
std_dev_per_100  = 95
move_up_at       = 5_000
move_down_at     = 2_000            # move down when bankroll < this

# Cashout rule (optional)
[cashout_rule]
interval_hands = 50_000             # must be a multiple of 100
[cashout_rule.kind]
type        = "profit_percentage"   # fixed | profit_percentage | bankroll_percentage
percentage  = 0.50                  # fraction (0–1) for profit_percentage | bankroll_percentage
#amount     = 500                   # dollar amount for fixed

# Probability queries (optional)
[[probability_queries]]
type = "bust"

[[probability_queries]]
type   = "reach_profit"
target = 100_000.0

[[probability_queries]]
type   = "reach_bankroll"
target = 50_000.0

[[probability_queries]]
type  = "at_limit_or_above"
limit = "NL500"
```

### Validation rules

- `limits` must be in ascending `bb_size` order
- All limits except the top must have `move_up_at`; all except the bottom must have `move_down_at`
- `move_down_at < move_up_at` for each limit
- `bust_bankroll` must be positive and less than `starting_bankroll`
- `interval_hands` must be a positive multiple of 100
- `num_simulations` must be between 1 and 100_000

## HTTP Server

```bash
server              # listens on 0.0.0.0:3000 (default)
server --port 8080  # custom port
server -p 8080
```

`POST /simulate[?seed=N]` — accepts the config as JSON in the request body, returns results as JSON.

**Request body** — same structure as `config.toml` but in JSON:

```json
{
  "starting_bankroll": 3000,
  "bust_bankroll": 1000,
  "num_simulations": 10000,
  "total_hands": 1000000,
  "limits": [
    {
      "name": "NL25",
      "bb_size": 0.25,
      "ev_per_100": 8,
      "std_dev_per_100": 90,
      "move_up_at": 2500
    }
  ]
}
```

**Response:**

```json
{
  "ci_70":  { "low": 10234.00, "high": 39847.00 },
  "ci_95":  { "low":  1102.00, "high": 51293.00 },
  "best":   61450.00,
  "worst":   -980.00,
  "percentiles": { "p10": 6200, "p25": 14100, "p50": 24500, "p75": 33800, "p90": 44200 },
  "probability_queries": [
    { "type": "bust",         "probability": 0.032100 },
    { "type": "reach_profit", "target": 100000.00, "probability": 0.417000 }
  ],
  "elapsed_seconds": 4.20
}
```

Errors are returned as `400 Bad Request` with body `{"error": "..."}`.
