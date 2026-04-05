use rand::Rng;
use rand_distr::Normal;

use crate::core::config::{CashoutKind, Config, ProbabilityQuery, starting_limit_index};

/// Result of one complete simulation run
#[derive(Debug)]
pub struct SimResult {
    pub net_profit: f64,
    pub total_cashouts: f64,
    pub went_bust: bool,
    /// Index into config.limits of the final limit being played
    pub final_limit_index: usize,
    pub final_bankroll: f64,
    /// True/false for each ProbabilityQuery (in order)
    pub query_hits: Vec<bool>,
}

/// Precomputed per-limit Normal distributions to avoid constructing them in the hot loop
pub struct PrecomputedLimits {
    pub distributions: Vec<Normal<f64>>,
}

impl PrecomputedLimits {
    pub fn new(config: &Config) -> Self {
        let distributions = config
            .limits
            .iter()
            .map(|l| Normal::new(l.ev_dollars(), l.std_dev_dollars()).unwrap())
            .collect();
        Self { distributions }
    }
}

pub fn run_simulation<R: Rng>(
    config: &Config,
    precomputed: &PrecomputedLimits,
    rng: &mut R,
) -> SimResult {
    let starting_bankroll = config.starting_bankroll;
    let mut bankroll = starting_bankroll;
    let mut hands_played: u64 = 0;
    let mut current_limit_idx = starting_limit_index(config);
    let mut total_cashouts = 0.0;
    let mut went_bust = false;

    // Bankroll at the time of the last cashout (for ProfitPercentage kind)
    let mut last_cashout_bankroll = starting_bankroll;

    // For probability queries: track if target was ever reached during the run
    let num_queries = config.probability_queries.len();
    let mut query_hits = vec![false; num_queries];

    let lowest_bust_threshold = config.bust_bankroll;
    let num_limits = config.limits.len();

    while hands_played < config.total_hands && !went_bust {
        // 1. PLAY ONE STEP (100 hands)
        let sample: f64 = rng.sample(precomputed.distributions[current_limit_idx]);
        bankroll += sample;
        hands_played += 100;
        // 2. CASHOUT RULE
        if let Some(rule) = &config.cashout_rule {
            if hands_played % rule.interval_hands == 0 {
                let withdrawal = compute_withdrawal(
                    rule,
                    bankroll,
                    last_cashout_bankroll,
                    lowest_bust_threshold,
                );
                if withdrawal > 0.0 {
                    bankroll -= withdrawal;
                    total_cashouts += withdrawal;
                    last_cashout_bankroll = bankroll;
                }
            }
        }

        // 3. BUST CHECK
        if bankroll < lowest_bust_threshold {
            went_bust = true;
            break;
        }

        // 4. LIMIT ADJUSTMENT — scan upward, then downward
        // Move up as far as possible
        while current_limit_idx + 1 < num_limits {
            let up_threshold = config.limits[current_limit_idx].move_up_at;
            match up_threshold {
                Some(t) if bankroll >= t => current_limit_idx += 1,
                _ => break,
            }
        }
        // Move down as far as necessary
        while current_limit_idx > 0 {
            let down_threshold = config.limits[current_limit_idx].move_down_at;
            match down_threshold {
                Some(t) if bankroll < t => current_limit_idx -= 1,
                _ => break,
            }
        }
    }

    // Evaluate probability queries at end of run
    let net_profit = (bankroll - starting_bankroll) + total_cashouts;
    for (qi, query) in config.probability_queries.iter().enumerate() {
        query_hits[qi] = match query {
            ProbabilityQuery::Bust => went_bust,
            ProbabilityQuery::ReachProfit { target } => net_profit >= *target,
            ProbabilityQuery::ReachBankroll { target } => bankroll >= *target,
            ProbabilityQuery::AtLimitOrAbove { limit } => {
                let limit_idx = config
                    .limits
                    .iter()
                    .position(|l| &l.name == limit)
                    .unwrap_or(0);
                current_limit_idx >= limit_idx
            }
        };
    }

    SimResult {
        net_profit,
        total_cashouts,
        went_bust,
        final_limit_index: current_limit_idx,
        final_bankroll: bankroll,
        query_hits,
    }
}

fn compute_withdrawal(
    rule: &crate::core::config::CashoutRule,
    bankroll: f64,
    last_cashout_bankroll: f64,
    lowest_bust_threshold: f64,
) -> f64 {
    let raw = match rule.kind {
        CashoutKind::Fixed { amount } => amount,
        CashoutKind::ProfitPercentage { percentage } => {
            let profit = bankroll - last_cashout_bankroll;
            if profit <= 0.0 {
                return 0.0;
            }
            profit * percentage
        }
        CashoutKind::BankrollPercentage { percentage } => bankroll * percentage,
    };
    // Clamp: never leave bankroll below the bust threshold
    let max_withdrawal = (bankroll - lowest_bust_threshold).max(0.0);
    raw.min(max_withdrawal).max(0.0)
}
