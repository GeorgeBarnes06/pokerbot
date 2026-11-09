//! Public module declarations for poker bots.

use engine::log::HandLog;
use engine::state::Action;
use engine::view::PlayerView;

pub mod random;
pub mod rule_based;
pub mod monte_carlo;
pub mod adaptive;

/// A poker strategy. Each seat at the table gets its own instance.
pub trait Bot {
    /// Chooses an action for the spot described by `view`.
    ///
    /// `view.legal` lists what is allowed. An illegal choice is replaced with a
    /// check if possible, otherwise a fold.
    fn decide(&mut self, view: &PlayerView) -> Action;

    /// Called after every hand at the table. Only this bot's own hole cards and
    /// hands shown at showdown are visible.
    fn observe(&mut self, _log: &HandLog) {}
}

/// Placeholder that always checks or calls. Used to run tournaments until the
/// real models exist.
#[derive(Clone, Copy, Debug, Default)]
pub struct CheckCall;

impl Bot for CheckCall {
    fn decide(&mut self, view: &PlayerView) -> Action {
        if view.legal.can_check {
            Action::Check
        } else {
            Action::Call
        }
    }
}
