//! Poker bots: the `Bot` interface, the registry of models, and shared helpers.
//!
//! See `crates/bots/README.md` for the design guide.

use std::time::Duration;

use engine::log::HandLog;
use engine::state::Action;
use engine::view::PlayerView;

pub mod kit;

pub mod bayes;
pub mod cfr7;
pub mod foldbot;
pub mod gto_lite;
pub mod monte;
pub mod nash;
pub mod riverrat;
pub mod tilt;

/// A poker strategy. Each seat at the table gets its own instance.
pub trait Bot {
    /// Chooses an action for the spot described by `view`.
    ///
    /// `view.legal` lists what is allowed; an illegal choice is replaced with a
    /// check if possible, otherwise a fold. Lines pushed to `notes` explain the
    /// choice: they appear in replays but are never shown to other bots.
    fn decide(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action;

    /// Called after every hand at the table. Only this bot's own hole cards,
    /// hands shown at showdown, and its own notes are visible.
    fn observe(&mut self, _log: &HandLog) {}
}

/// Time a single decision should take in a release build. Checked by
/// `cargo test --release -p bots`.
pub const DECISION_BUDGET: Duration = Duration::from_millis(100);

/// The eight models, as `(config name, display name)`, in default seat order.
pub const MODELS: [(&str, &str); 8] = [
    ("cfr7", "CFR-7"),
    ("nash", "NASH"),
    ("monte", "MONTE"),
    ("tilt", "TILT"),
    ("gto_lite", "GTO-LITE"),
    ("riverrat", "RIVERRAT"),
    ("foldbot", "FOLDBOT"),
    ("bayes", "BAYES"),
];

/// Builds the bot a config names. `seed` makes any randomness in the bot
/// repeatable, so a match replays exactly.
pub fn create(name: &str, seed: u64) -> Option<Box<dyn Bot>> {
    let bot: Box<dyn Bot> = match name {
        "cfr7" => Box::new(cfr7::Cfr7::new(seed)),
        "nash" => Box::new(nash::Nash::new(seed)),
        "monte" => Box::new(monte::Monte::new(seed)),
        "tilt" => Box::new(tilt::Tilt::new(seed)),
        "gto_lite" => Box::new(gto_lite::GtoLite::new(seed)),
        "riverrat" => Box::new(riverrat::RiverRat::new(seed)),
        "foldbot" => Box::new(foldbot::FoldBot::new(seed)),
        "bayes" => Box::new(bayes::Bayes::new(seed)),
        "check_call" => Box::new(CheckCall),
        _ => return None,
    };
    Some(bot)
}

/// Baseline that always checks or calls. Useful as an opponent when testing a model.
#[derive(Clone, Copy, Debug, Default)]
pub struct CheckCall;

impl Bot for CheckCall {
    fn decide(&mut self, view: &PlayerView, _notes: &mut Vec<String>) -> Action {
        kit::check_or_call(view)
    }
}
