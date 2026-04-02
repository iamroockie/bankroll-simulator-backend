use crate::core::config::Config;
use crate::core::stats::AggregateStats;

pub fn print_results(
    config: &Config,
    stats: &AggregateStats,
    config_path: &str,
    elapsed_secs: f64,
) {
    let pr = stats.percentile_report();
    let sim_per_sec = stats.total_simulations as f64 / elapsed_secs;

    println!(
        "\nPoker Bankroll Simulator  v{}\n",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "Config: {}  |  Simulations: {}  |  Hands: {}",
        config_path,
        format_count(stats.total_simulations),
        format_count(config.total_hands as usize)
    );

    println!("\n--- Confidence Intervals (Net Profit) -------------------------");
    println!(
        "  70% CI  (p15 – p85):     {:>10}  –  {:<10}",
        format_dollars(pr.p15),
        format_dollars(pr.p85)
    );
    println!(
        "  95% CI  (p2.5 – p97.5):  {:>10}  –  {:<10}",
        format_dollars(pr.p2_5),
        format_dollars(pr.p97_5)
    );

    println!("\n--- Extremes ---------------------------------------------------");
    println!("  Best result:   {}", format_dollars(pr.best));
    println!("  Worst result:  {}", format_dollars(pr.worst));

    println!("\n--- Percentile Distribution ------------------------------------");
    println!(
        "  {:<12} {:<12} {:<12} {:<12} {:<12}",
        "P10", "P25", "P50", "P75", "P90"
    );
    println!(
        "  {:<12} {:<12} {:<12} {:<12} {:<12}",
        format_dollars(pr.p10),
        format_dollars(pr.p25),
        format_dollars(pr.p50),
        format_dollars(pr.p75),
        format_dollars(pr.p90),
    );

    if !config.probability_queries.is_empty() {
        println!("\n--- Probability Queries ----------------------------------------");
        for (qi, query) in config.probability_queries.iter().enumerate() {
            let prob = stats.query_probability(qi);
            println!("  {:<44} {:.4}%", query.description(), prob * 100.0);
        }
    }

    println!(
        "\nCompleted in {:.1}s  ({:.0} sim/s)",
        elapsed_secs, sim_per_sec
    );
}

pub fn print_validate_summary(config: &Config) {
    println!("Config is valid.\n");
    println!(
        "  Starting bankroll: {}",
        format_dollars(config.starting_bankroll)
    );
    println!(
        "  Simulations:       {}",
        format_count(config.num_simulations)
    );
    println!(
        "  Total hands:       {}",
        format_count(config.total_hands as usize)
    );

    println!(
        "  Bust threshold:    {}",
        format_dollars(config.bust_bankroll)
    );

    println!("\n  Limits ({}):", config.limits.len());
    for limit in &config.limits {
        let up = limit
            .move_up_at
            .map(|v| format_dollars(v))
            .unwrap_or_else(|| "—".to_string());
        let down = limit
            .move_down_at
            .map(|v| format_dollars(v))
            .unwrap_or_else(|| "—".to_string());
        println!(
            "    {:8}  EV={:+.1} BB/100  std={:.1} BB/100  up>{}  down<{}",
            limit.name, limit.ev_per_100, limit.std_dev_per_100, up, down,
        );
    }

    if let Some(rule) = &config.cashout_rule {
        println!("\n  Cashout rule:");
        println!(
            "    Every {} hands: {:?}",
            format_count(rule.interval_hands as usize),
            rule.kind,
        );
    }

    if !config.probability_queries.is_empty() {
        println!(
            "\n  Probability queries ({}):",
            config.probability_queries.len()
        );
        for query in &config.probability_queries {
            println!("    {}", query.description());
        }
    }
}

fn format_dollars(value: f64) -> String {
    if value < 0.0 {
        format!("-${:.0}", value.abs())
    } else {
        format!("${:.0}", value)
    }
}

fn format_count(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}
