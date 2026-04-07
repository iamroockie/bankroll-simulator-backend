use crate::core::config::Config;
use crate::core::stats::AggregateStats;

pub fn print_results(
    config: &Config,
    stats: &AggregateStats,
    config_path: &str,
    elapsed_secs: f64,
) {
    let r = stats.report();
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

    println!("\n--- Results --------------------------------------------------------");
    println!("  {:<10} {:<18} {:<18}", "", "Final Bankroll", "Cashout");
    println!("{}", metric_row("Worst:", r.worst));
    println!("{}", metric_row("P10:", r.p10));
    println!("{}", metric_row("P30:", r.p30));
    println!("{}", metric_row("P50:", r.p50));
    println!("{}", metric_row("P70:", r.p70));
    println!("{}", metric_row("P90:", r.p90));
    println!("{}", metric_row("Best:", r.best));

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

pub fn print_validate_summary(config: &Config, num_simulations: usize) {
    println!("Config is valid.\n");
    println!(
        "  Starting bankroll: {}",
        format_dollars(config.starting_bankroll)
    );
    println!("  Simulations:       {}", format_count(num_simulations));
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

fn metric_row(label: &str, pair: (f64, f64)) -> String {
    format!(
        "  {:<10} {:<18} {:<18}",
        label,
        format_dollars(pair.0),
        format_dollars(pair.1),
    )
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
