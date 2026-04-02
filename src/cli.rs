use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "br", version, about = "Poker Bankroll Simulator")]
pub struct Cli {
    /// Path to TOML config file
    #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
    pub config: String,

    /// Override num_simulations from config
    #[arg(short, long, value_name = "N")]
    pub simulations: Option<usize>,

    /// Override total_hands from config
    #[arg(short = 'H', long, value_name = "N")]
    pub hands: Option<u64>,

    /// Seed for reproducible runs (each sim i uses seed XOR i)
    #[arg(long)]
    pub seed: Option<u64>,

    /// Suppress progress output, print only final results
    #[arg(short, long)]
    pub quiet: bool,

    /// Output results as JSON
    #[arg(short, long)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Validate config without running simulations
    Validate,
}
