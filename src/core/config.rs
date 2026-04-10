use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub starting_bankroll: f64,
    /// Bankroll below which the simulation counts as bust
    pub bust_bankroll: f64,
    pub total_hands: u64,
    pub limits: Vec<LimitConfig>,
    pub cashout_rule: Option<CashoutRule>,
    #[serde(default)]
    pub probability_queries: Vec<ProbabilityQuery>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LimitConfig {
    pub name: String,
    /// Size of 1 big blind in dollars (e.g. 0.50 for NL50)
    pub bb_size: f64,
    /// Expected value in BB/100
    pub ev_per_100: f64,
    /// Standard deviation in BB/100
    pub std_dev_per_100: f64,
    /// Move up to next limit when bankroll >= this value (None for top limit)
    pub move_up_at: Option<f64>,
    /// Move down to previous limit when bankroll < this value (None for bottom limit)
    pub move_down_at: Option<f64>,
}

impl LimitConfig {
    /// Dollar EV per 100-hand step
    pub fn ev_dollars(&self) -> f64 {
        self.ev_per_100 * self.bb_size
    }

    /// Dollar std_dev per 100-hand step
    pub fn std_dev_dollars(&self) -> f64 {
        self.std_dev_per_100 * self.bb_size
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CashoutRule {
    /// Must be divisible by 100 (simulation step size)
    pub interval_hands: u64,
    pub kind: CashoutKind,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CashoutKind {
    /// Dollar amount
    Fixed { amount: f64 },
    /// Fraction of profit since last cashout (0.0 – 1.0)
    ProfitPercentage { percentage: f64 },
    /// Fraction of current bankroll (0.0 – 1.0)
    BankrollPercentage { percentage: f64 },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProbabilityQuery {
    Bust,
    ReachProfit { target: f64 },
    ReachBankroll { target: f64 },
    AtLimitOrAbove { limit: String },
}

impl ProbabilityQuery {
    pub fn description(&self) -> String {
        match self {
            ProbabilityQuery::Bust => "P(bust)".to_string(),
            ProbabilityQuery::ReachProfit { target } => {
                format!("P(net profit >= ${:.0})", target)
            }
            ProbabilityQuery::ReachBankroll { target } => {
                format!("P(bankroll >= ${:.0} at end)", target)
            }
            ProbabilityQuery::AtLimitOrAbove { limit } => {
                format!("P(at {} or above at end)", limit)
            }
        }
    }
}

/// Validate config and return error string if invalid
pub fn validate(config: &Config) -> Result<(), String> {
    if config.limits.is_empty() {
        return Err("At least one limit must be defined".to_string());
    }

    // Limits must be sorted ascending by bb_size
    for i in 1..config.limits.len() {
        if config.limits[i].bb_size <= config.limits[i - 1].bb_size {
            return Err(format!(
                "Limits must be in ascending order by bb_size: {} ({}) is not greater than {} ({})",
                config.limits[i].name,
                config.limits[i].bb_size,
                config.limits[i - 1].name,
                config.limits[i - 1].bb_size,
            ));
        }
    }

    for (i, limit) in config.limits.iter().enumerate() {
        if limit.std_dev_per_100 <= 0.0 {
            return Err(format!(
                "Limit {}: std_dev_per_100 must be positive",
                limit.name
            ));
        }
        if i + 1 < config.limits.len() && limit.move_up_at.is_none() {
            return Err(format!(
                "Limit {}: move_up_at is required (not the top limit)",
                limit.name
            ));
        }
        if i > 0 && limit.move_down_at.is_none() {
            return Err(format!(
                "Limit {}: move_down_at is required (not the bottom limit)",
                limit.name
            ));
        }
        if let (Some(down), Some(up)) = (limit.move_down_at, limit.move_up_at) {
            if down >= up {
                return Err(format!(
                    "Limit {}: move_down_at ({}) must be less than move_up_at ({})",
                    limit.name, down, up
                ));
            }
        }
        // move_up_at monotonically increasing
        if i + 1 < config.limits.len() {
            if let (Some(up_i), Some(up_next)) = (limit.move_up_at, config.limits[i + 1].move_up_at)
            {
                if up_i >= up_next {
                    return Err(format!(
                        "move_up_at of {} ({}) must be less than move_up_at of {} ({})",
                        limit.name,
                        up_i,
                        config.limits[i + 1].name,
                        up_next
                    ));
                }
            }
        }
    }

    if let Some(rule) = &config.cashout_rule {
        if rule.interval_hands == 0 || rule.interval_hands % 100 != 0 {
            return Err(format!(
                "Cashout interval_hands ({}) must be a positive multiple of 100",
                rule.interval_hands
            ));
        }
        match rule.kind {
            CashoutKind::Fixed { amount } => {
                if amount < 0.0 {
                    return Err("Cashout amount must be non-negative".to_string());
                }
            }
            CashoutKind::ProfitPercentage { percentage }
            | CashoutKind::BankrollPercentage { percentage } => {
                if percentage < 0.0 || percentage > 1.0 {
                    return Err("Cashout percentage must be between 0.0 and 1.0".to_string());
                }
            }
        }
    }

    // Validate AtLimitOrAbove queries reference real limits
    for query in &config.probability_queries {
        if let ProbabilityQuery::AtLimitOrAbove { limit } = query {
            if !config.limits.iter().any(|l| &l.name == limit) {
                return Err(format!(
                    "probability_query references unknown limit: {}",
                    limit
                ));
            }
        }
    }

    if config.bust_bankroll < 0.0 {
        return Err("bust_bankroll must be positive".to_string());
    }
    if config.bust_bankroll >= config.starting_bankroll {
        return Err("bust_bankroll must be less than starting_bankroll".to_string());
    }
    if config.starting_bankroll <= 0.0 {
        return Err("starting_bankroll must be positive".to_string());
    }
    if config.total_hands == 0 || config.total_hands % 100 != 0 {
        return Err("total_hands must be a positive multiple of 100".to_string());
    }

    Ok(())
}

/// Find the index of the highest limit playable with the given bankroll
pub fn starting_limit_index(config: &Config) -> usize {
    let mut idx = 0;
    for (i, limit) in config.limits.iter().enumerate() {
        let min_bankroll = limit.move_down_at.unwrap_or(config.bust_bankroll);
        if config.starting_bankroll >= min_bankroll {
            idx = i;
        }
    }
    idx
}
