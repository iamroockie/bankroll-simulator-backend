use crate::core::config::Config;
use crate::core::simulation::{run_simulation, PrecomputedLimits};
use crate::core::stats::AggregateStats;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rayon::prelude::*;
use std::time::{Duration, Instant};

pub struct RunResult {
    pub stats: AggregateStats,
    pub elapsed: Duration,
}

pub fn run_simulations(config: &Config, seed: Option<u64>, num_simulations: usize) -> RunResult {
    let precomputed = PrecomputedLimits::new(config);
    let num_queries = config.probability_queries.len();
    let start = Instant::now();

    let results = (0..num_simulations)
        .into_par_iter()
        .map(|i| {
            let mut rng = match seed {
                Some(s) => SmallRng::seed_from_u64(s ^ (i as u64)),
                None => SmallRng::from_os_rng(),
            };
            run_simulation(config, &precomputed, &mut rng)
        })
        .collect();

    RunResult {
        stats: AggregateStats::from_results(results, num_queries),
        elapsed: start.elapsed(),
    }
}
