use crate::core::simulation::SimResult;

pub struct AggregateStats {
    /// Sorted ascending by net_profit; each element is (final_bankroll, total_cashouts)
    pub runs: Vec<(f64, f64)>,
    pub total_simulations: usize,
    /// Hit counts for each ProbabilityQuery
    pub query_hit_counts: Vec<usize>,
}

pub struct SimReport {
    pub worst: (f64, f64),
    pub p2_5: (f64, f64),
    pub p5: (f64, f64),
    pub p10: (f64, f64),
    pub p15: (f64, f64),
    pub p20: (f64, f64),
    pub p30: (f64, f64),
    pub p40: (f64, f64),
    pub p50: (f64, f64),
    pub p60: (f64, f64),
    pub p70: (f64, f64),
    pub p80: (f64, f64),
    pub p85: (f64, f64),
    pub p90: (f64, f64),
    pub p95: (f64, f64),
    pub p97_5: (f64, f64),
    pub best: (f64, f64),
}
// Each pair: .0 = final_bankroll, .1 = total_cashouts — from the same simulation run.

impl AggregateStats {
    pub fn from_results(results: Vec<SimResult>, num_queries: usize) -> Self {
        let total_simulations = results.len();
        let mut query_hit_counts = vec![0usize; num_queries];

        let mut triples: Vec<(f64, f64, f64)> = results
            .iter()
            .map(|r| (r.net_profit, r.final_bankroll, r.total_cashouts))
            .collect();

        for result in &results {
            for (qi, &hit) in result.query_hits.iter().enumerate() {
                if hit {
                    query_hit_counts[qi] += 1;
                }
            }
        }

        triples.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        let runs = triples.into_iter().map(|(_, fb, co)| (fb, co)).collect();

        Self {
            runs,
            total_simulations,
            query_hit_counts,
        }
    }

    pub fn report(&self) -> SimReport {
        SimReport {
            worst: self.runs.first().copied().unwrap_or((0.0, 0.0)),
            p2_5: pair_at(&self.runs, 0.025),
            p5: pair_at(&self.runs, 0.05),
            p10: pair_at(&self.runs, 0.10),
            p15: pair_at(&self.runs, 0.15),
            p20: pair_at(&self.runs, 0.20),
            p30: pair_at(&self.runs, 0.30),
            p40: pair_at(&self.runs, 0.40),
            p50: pair_at(&self.runs, 0.50),
            p60: pair_at(&self.runs, 0.60),
            p70: pair_at(&self.runs, 0.70),
            p80: pair_at(&self.runs, 0.80),
            p85: pair_at(&self.runs, 0.85),
            p90: pair_at(&self.runs, 0.90),
            p95: pair_at(&self.runs, 0.95),
            p97_5: pair_at(&self.runs, 0.975),
            best: self.runs.last().copied().unwrap_or((0.0, 0.0)),
        }
    }

    pub fn query_probability(&self, query_idx: usize) -> f64 {
        if self.total_simulations == 0 {
            return 0.0;
        }
        self.query_hit_counts[query_idx] as f64 / self.total_simulations as f64
    }
}

fn pair_at(runs: &[(f64, f64)], p: f64) -> (f64, f64) {
    if runs.is_empty() {
        return (0.0, 0.0);
    }
    let idx = (p * (runs.len() - 1) as f64).round() as usize;
    runs[idx]
}
