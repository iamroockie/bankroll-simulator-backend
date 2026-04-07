use serde::Serialize;

use crate::core::config::{Config, ProbabilityQuery};
use crate::core::stats::AggregateStats;

#[derive(Serialize)]
struct Metrics {
    final_bankroll: f64,
    cashout: f64,
}

#[derive(Serialize)]
struct PercentilesBlock {
    worst: Metrics,
    p2_5: Metrics,
    p5: Metrics,
    p10: Metrics,
    p15: Metrics,
    p20: Metrics,
    p30: Metrics,
    p40: Metrics,
    p50: Metrics,
    p60: Metrics,
    p70: Metrics,
    p80: Metrics,
    p85: Metrics,
    p90: Metrics,
    p95: Metrics,
    p97_5: Metrics,
    best: Metrics,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum QueryResult {
    Bust { probability: f64 },
    ReachProfit { target: f64, probability: f64 },
    ReachBankroll { target: f64, probability: f64 },
    AtLimitOrAbove { limit: String, probability: f64 },
}

#[derive(Serialize)]
struct SimResponse {
    simulations: usize,
    percentiles: PercentilesBlock,
    probability_queries: Vec<QueryResult>,
    elapsed_seconds: f64,
}

fn mk(pair: (f64, f64)) -> Metrics {
    Metrics {
        final_bankroll: pair.0,
        cashout: pair.1,
    }
}

/// Serialize simulation results to a JSON string.
pub fn to_json_string(config: &Config, stats: &AggregateStats, elapsed_secs: f64) -> String {
    let r = stats.report();

    let probability_queries = config
        .probability_queries
        .iter()
        .enumerate()
        .map(|(qi, query)| {
            let probability = stats.query_probability(qi);
            match query {
                ProbabilityQuery::Bust => QueryResult::Bust { probability },
                ProbabilityQuery::ReachProfit { target } => QueryResult::ReachProfit {
                    target: *target,
                    probability,
                },
                ProbabilityQuery::ReachBankroll { target } => QueryResult::ReachBankroll {
                    target: *target,
                    probability,
                },
                ProbabilityQuery::AtLimitOrAbove { limit } => QueryResult::AtLimitOrAbove {
                    limit: limit.clone(),
                    probability,
                },
            }
        })
        .collect();

    let response = SimResponse {
        simulations: stats.total_simulations,
        percentiles: PercentilesBlock {
            worst: mk(r.worst),
            p2_5: mk(r.p2_5),
            p5: mk(r.p5),
            p10: mk(r.p10),
            p15: mk(r.p15),
            p20: mk(r.p20),
            p30: mk(r.p30),
            p40: mk(r.p40),
            p50: mk(r.p50),
            p60: mk(r.p60),
            p70: mk(r.p70),
            p80: mk(r.p80),
            p85: mk(r.p85),
            p90: mk(r.p90),
            p95: mk(r.p95),
            p97_5: mk(r.p97_5),
            best: mk(r.best),
        },
        probability_queries,
        elapsed_seconds: elapsed_secs,
    };

    serde_json::to_string(&response).expect("SimResponse serialization cannot fail")
}
