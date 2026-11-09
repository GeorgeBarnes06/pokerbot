//! Hand-log structures for JSON output.

use serde::{Deserialize, Serialize};

use crate::card::Card;
use crate::pot::HandResult;
use crate::state::{Action, Blinds, Chips, Street};

/// One action taken during a hand.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionRecord {
    pub street: Street,
    pub seat: usize,
    pub action: Action,
    /// Chips this action put into the pot.
    pub amount: Chips,
    /// What the player asked for, when that was illegal and replaced with a check or fold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejected: Option<Action>,
    /// How long the player took to decide, in microseconds.
    #[serde(default)]
    pub think_us: u64,
    /// The bot's own notes on why it chose this action. Hidden from other players.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// Each live player's chance of winning at the start of a street, with every
/// hand known. For figures and analysis only; bots never see it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StreetEquity {
    pub street: Street,
    pub seats: Vec<SeatEquity>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeatEquity {
    pub seat: usize,
    /// Share of the pot this hand wins on average, from 0 to 1 (ties split).
    pub equity: f64,
}

/// A player dealt into the hand, as they started it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SeatStart {
    pub seat: usize,
    pub name: String,
    /// Stack before antes and blinds.
    pub stack: Chips,
    /// Hole cards; `None` in a redacted log for cards that were never shown.
    pub hole_cards: Option<[Card; 2]>,
    /// Antes and blinds posted before any action.
    pub posted: Chips,
}

/// Everything that happened in one hand.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HandLog {
    pub hand_number: u64,
    pub seed: u64,
    pub button: usize,
    pub blinds: Blinds,
    pub seats: Vec<SeatStart>,
    pub actions: Vec<ActionRecord>,
    pub board: Vec<Card>,
    pub result: HandResult,
    /// Every seat's stack after the payout, indexed by seat.
    pub final_stacks: Vec<Chips>,
    /// Equity at the start of each street reached; empty unless the runner records it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equity: Vec<StreetEquity>,
}

impl HandLog {
    /// The log as `seat` is entitled to see it: their own hole cards plus any
    /// shown at showdown, and only their own notes. Equity is removed, since it
    /// is computed from every hand.
    pub fn redacted_for(&self, seat: usize) -> HandLog {
        let mut log = self.clone();
        for start in &mut log.seats {
            let shown = self.result.shown.iter().any(|hand| hand.seat == start.seat);
            if start.seat != seat && !shown {
                start.hole_cards = None;
            }
        }
        for record in &mut log.actions {
            if record.seat != seat {
                record.notes.clear();
            }
        }
        log.equity.clear();
        log
    }
}
