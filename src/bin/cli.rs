use br::{
    cli::{Cli, Command},
    core::config,
    output, runner,
};
use clap::Parser;

fn main() {
    dotenvy::dotenv().ok();

    let num_simulations: usize = std::env::var("NUM_SIMULATIONS")
        .expect("NUM_SIMULATIONS must be set in .env")
        .parse()
        .expect("NUM_SIMULATIONS must be a valid integer");

    let cli = Cli::parse();

    let toml_str = match std::fs::read_to_string(&cli.config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading config file '{}': {}", cli.config, e);
            std::process::exit(1);
        }
    };

    let mut cfg: config::Config = match toml::from_str(&toml_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing config file: {}", e);
            std::process::exit(1);
        }
    };

    if let Some(h) = cli.hands {
        cfg.total_hands = h;
    }

    if let Err(e) = config::validate(&cfg) {
        eprintln!("Config validation error: {}", e);
        std::process::exit(1);
    }

    if let Some(Command::Validate) = cli.command {
        output::text::print_validate_summary(&cfg, num_simulations);
        return;
    }

    if !cli.quiet {
        eprintln!(
            "Running {} simulations ({} hands each)...",
            num_simulations, cfg.total_hands
        );
    }

    let result = runner::run_simulations(&cfg, cli.seed, num_simulations);
    let elapsed = result.elapsed.as_secs_f64();

    if cli.json {
        println!(
            "{}",
            output::json::to_json_string(&cfg, &result.stats, elapsed)
        );
    } else {
        output::text::print_results(&cfg, &result.stats, &cli.config, elapsed);
    }
}
