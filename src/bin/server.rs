use axum::{
    Router,
    extract::{Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use br::{core::config, output::json, runner};
use clap::Parser;
use serde::Deserialize;
use serde_json::json;
use tower_http::cors::CorsLayer;

#[derive(Parser)]
#[command(name = "br-server", about = "Poker Bankroll Simulator HTTP server")]
struct Cli {
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[derive(Deserialize)]
struct SimulateQuery {
    seed: Option<u64>,
}

fn error_json(msg: String) -> Response {
    (
        StatusCode::BAD_REQUEST,
        [(header::CONTENT_TYPE, "application/json")],
        json!({"error": &msg}).to_string(),
    )
        .into_response()
}

async fn simulate(
    State(num_simulations): State<usize>,
    Query(query): Query<SimulateQuery>,
    body: String,
) -> Response {
    let cfg: config::Config = match serde_json::from_str(&body) {
        Ok(c) => c,
        Err(e) => {
            let msg = e.to_string();
            let clean = msg.split(" at line ").next().unwrap_or(&msg);
            return error_json(clean.to_string());
        }
    };

    if let Err(e) = config::validate(&cfg) {
        return error_json(e);
    }

    let cfg_for_run = cfg.clone();
    let result = tokio::task::spawn_blocking(move || {
        runner::run_simulations(&cfg_for_run, query.seed, num_simulations)
    })
    .await
    .unwrap();

    let body = json::to_json_string(&cfg, &result.stats, result.elapsed.as_secs_f64());
    ([(header::CONTENT_TYPE, "application/json")], body).into_response()
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let num_simulations: usize = std::env::var("NUM_SIMULATIONS")
        .expect("NUM_SIMULATIONS must be set in .env")
        .parse()
        .expect("NUM_SIMULATIONS must be a valid integer");

    let cli = Cli::parse();

    let app = Router::new()
        .route("/simulate", post(simulate))
        .with_state(num_simulations)
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", cli.port);
    println!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
