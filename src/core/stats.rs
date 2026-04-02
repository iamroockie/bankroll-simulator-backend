use crate::core::simulation::SimResult;

pub struct AggregateStats {
    /// Sorted net profits across all simulations
    pub net_profits: Vec<f64>,
    pub total_simulations: usize,
    /// Hit counts for each ProbabilityQuery
    pub query_hit_counts: Vec<usize>,
}

pub struct PercentileReport {
    pub p2_5: f64,
    pub p10: f64,
    pub p15: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p85: f64,
    pub p90: f64,
    pub p97_5: f64,
    pub best: f64,
    pub worst: f64,
}

impl AggregateStats {
    pub fn from_results(results: Vec<SimResult>, num_queries: usize) -> Self {
        let total_simulations = results.len();
        let mut net_profits = Vec::with_capacity(total_simulations);
        let mut query_hit_counts = vec![0usize; num_queries];

        for result in &results {
            net_profits.push(result.net_profit);
            for (qi, &hit) in result.query_hits.iter().enumerate() {
                if hit {
                    query_hit_counts[qi] += 1;
                }
            }
        }

        net_profits.sort_unstable_by(f64::total_cmp);

        Self {
            net_profits,
            total_simulations,
            query_hit_counts,
        }
    }

    pub fn percentile_report(&self) -> PercentileReport {
        PercentileReport {
            p2_5: self.percentile(0.025),
            p10: self.percentile(0.10),
            p15: self.percentile(0.15),
            p25: self.percentile(0.25),
            p50: self.percentile(0.50),
            p75: self.percentile(0.75),
            p85: self.percentile(0.85),
            p90: self.percentile(0.90),
            p97_5: self.percentile(0.975),
            best: *self.net_profits.last().unwrap_or(&0.0),
            worst: *self.net_profits.first().unwrap_or(&0.0),
        }
    }

    pub fn query_probability(&self, query_idx: usize) -> f64 {
        if self.total_simulations == 0 {
            return 0.0;
        }
        self.query_hit_counts[query_idx] as f64 / self.total_simulations as f64
    }

    fn percentile(&self, p: f64) -> f64 {
        if self.net_profits.is_empty() {
            return 0.0;
        }
        let idx = (p * (self.net_profits.len() - 1) as f64).round() as usize;
        self.net_profits[idx]
    }
}
