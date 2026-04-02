use serde::Serialize;

use crate::core::config::{Config, ProbabilityQuery};
use crate::core::stats::AggregateStats;

#[derive(Serialize)]
struct Interval {
    low: f64,
    high: f64,
}

#[derive(Serialize)]
struct Percentiles {
    p10: f64,
    p25: f64,
    p50: f64,
    p75: f64,
    p90: f64,
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
    ci_70: Interval,
    ci_95: Interval,
    best: f64,
    worst: f64,
    percentiles: Percentiles,
    probability_queries: Vec<QueryResult>,
    elapsed_seconds: f64,
}

/// Serialize simulation results to a JSON string.
/// Returns a String so it can be used by both the CLI and an HTTP handler.
pub fn to_json_string(config: &Config, stats: &AggregateStats, elapsed_secs: f64) -> String {
    let pr = stats.percentile_report();

    let probability_queries = config
        .probability_queries
        .iter()
        .enumerate()
        .map(|(qi, query)| {
            let probability = stats.query_probability(qi);
            match query {
                ProbabilityQuery::Bust => QueryResult::Bust { probability },
                ProbabilityQuery::ReachProfit { target } => {
                    QueryResult::ReachProfit { target: *target, probability }
                }
                ProbabilityQuery::ReachBankroll { target } => {
                    QueryResult::ReachBankroll { target: *target, probability }
                }
                ProbabilityQuery::AtLimitOrAbove { limit } => {
                    QueryResult::AtLimitOrAbove { limit: limit.clone(), probability }
                }
            }
        })
        .collect();

    let response = SimResponse {
        ci_70: Interval { low: pr.p15, high: pr.p85 },
        ci_95: Interval { low: pr.p2_5, high: pr.p97_5 },
        best: pr.best,
        worst: pr.worst,
        percentiles: Percentiles {
            p10: pr.p10,
            p25: pr.p25,
            p50: pr.p50,
            p75: pr.p75,
            p90: pr.p90,
        },
        probability_queries,
        elapsed_seconds: elapsed_secs,
    };

    serde_json::to_string(&response).expect("SimResponse serialization cannot fail")
}
