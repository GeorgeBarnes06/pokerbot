//! Runs a match from a config file and writes its log to `logs/<config>-<seed>/`.
//!
//! Usage: `cargo run -p runner -- [config.json]` (defaults to `configs/default.json`).
//!
//! The log folder holds `summary.json` (config, standings, one line per hand) and
//! `hands/<n>.json` with every hand in full, so the viewer never loads the whole match.

mod config;
mod tournament;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use config::{make_bot, TournamentConfig};
use serde::Serialize;
use tournament::TournamentLog;

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

    let stem = config_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("match");
    let out = PathBuf::from("logs").join(format!("{stem}-{}", config.seed));
    write_log(&out, &log)?;

    let status = if log.finished { "finished" } else { "stopped at max_hands" };
    println!("{} hands played ({status})", log.hands.len());
    for standing in &log.standings {
        let result = if config.rebuy {
            format!(
                "{:+} chips ({:+.1} BB/100, {} rebuys)",
                standing.net_chips, standing.bb_per_100, standing.rebuys
            )
        } else {
            standing
                .busted_on_hand
                .map(|hand| format!("out on hand {hand}"))
                .unwrap_or_else(|| format!("{} chips", standing.final_stack))
        };
        println!("{:>2}. {} ({}) - {result}", standing.place, standing.name, standing.bot);
    }
    println!("log written to {}/", out.display());
    Ok(())
}

/// Replaces any earlier log for the same config and seed.
fn write_log(dir: &Path, log: &TournamentLog) -> Result<(), String> {
    if dir.exists() {
        fs::remove_dir_all(dir).map_err(|e| format!("clearing {}: {e}", dir.display()))?;
    }
    let hands_dir = dir.join("hands");
    fs::create_dir_all(&hands_dir).map_err(|e| format!("creating {}: {e}", hands_dir.display()))?;

    write_json(&dir.join("summary.json"), &log.summary())?;
    for hand in &log.hands {
        write_json(&hands_dir.join(format!("{}.json", hand.hand_number)), hand)?;
    }
    Ok(())
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let json = serde_json::to_string(value).map_err(|e| format!("encoding {}: {e}", path.display()))?;
    fs::write(path, json).map_err(|e| format!("writing {}: {e}", path.display()))
}
