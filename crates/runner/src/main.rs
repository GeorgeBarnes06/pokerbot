//! Runs a tournament from a config file and writes its log to `logs/`.
//!
//! Usage: `cargo run -p runner -- [config.json]` (defaults to `configs/default.json`).

mod config;
mod tournament;

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use config::{make_bot, TournamentConfig};

fn main() -> ExitCode {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "configs/default.json".into());

    match run(Path::new(&path)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(config_path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(config_path)
        .map_err(|e| format!("reading {}: {e}", config_path.display()))?;
    let config = TournamentConfig::from_json(&text)?;

    let mut bots = config
        .players
        .iter()
        .map(|p| make_bot(&p.bot))
        .collect::<Result<Vec<_>, _>>()?;
    let log = tournament::run(&config, &mut bots);

    fs::create_dir_all("logs").map_err(|e| format!("creating logs/: {e}"))?;
    let out = format!("logs/tournament-{}.json", config.seed);
    let json = serde_json::to_string_pretty(&log).map_err(|e| format!("encoding log: {e}"))?;
    fs::write(&out, json).map_err(|e| format!("writing {out}: {e}"))?;

    let status = if log.finished { "finished" } else { "stopped at max_hands" };
    println!("{} hands played ({status})", log.hands.len());
    for standing in &log.standings {
        let busted = standing
            .busted_on_hand
            .map(|hand| format!("out on hand {hand}"))
            .unwrap_or_else(|| format!("{} chips", standing.final_stack));
        println!(
            "{:>2}. {} ({}) - {busted}",
            standing.place, standing.name, standing.bot
        );
    }
    println!("log written to {out}");
    Ok(())
}
