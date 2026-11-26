//! TILT.
//!
//! The name suggests a player whose mood changes with results: steady when winning, looser and
//! more aggressive after losing big pots (state kept in `observe`).
//!
//! # Design
//!
//! Fill this in before writing the strategy:
//!
//! - **Idea:**
//! - **Reads from `PlayerView`:**
//! - **Remembers between hands (`observe`):**
//! - **Parameters to tune:**
//! - **Expected result:** (net chips / BB per 100 against `check_call`, and in the 8-bot table)

use engine::log::HandLog;
use engine::state::Action;
use engine::view::PlayerView;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::{kit, Bot};

#[allow(dead_code)] // remove once the strategy uses every field
pub struct Tilt {
    rng: StdRng,               // seeded per seat by the runner; use it for all randomness
    seat: Option<usize>,       // own seat, learned in decide so observe can find itself
    tilt: f64,                 // mood: 0.0 calm .. 1.0 fully tilted
    tilt_per_bb_lost: f64,     // how much tilt each big blind lost in a hand adds
    cool_down: f64,            // tilt is multiplied by this after every hand (calms down)
    calm_call_fraction: f64,   // largest call, as a share of the pot, when calm
    tilted_call_fraction: f64, // largest call when fully tilted
    calm_open_size_bb: f64,    // preflop open size when calm
    tilted_open_size_bb: f64,  // preflop open size when fully tilted
}

impl Tilt {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            seat: None,
            tilt: 0.0,
            tilt_per_bb_lost: 0.02,
            cool_down: 0.9,
            calm_call_fraction: 0.25,
            tilted_call_fraction: 0.5,
            calm_open_size_bb: 2.5,
            tilted_open_size_bb: 5.0,
        }
    }
}

#[allow(dead_code, unused_variables, clippy::ptr_arg)] // remove once decide calls these and notes is used
impl Tilt {
    /// A setting between its calm and tilted values, by mood: `calm` at tilt 0.0,
    /// `tilted` at 1.0, and in proportion between.
    fn blend(&self, calm: f64, tilted: f64) -> f64 {
        calm + (tilted - calm) * self.tilt
    }

    fn preflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // Like FOLDBOT's preflop, but the more tilted, the more hands it plays and the
        // bigger it raises:
        // - open size: self.blend(self.calm_open_size_bb, self.tilted_open_size_bb)
        // - a wider hand list when tilted (e.g. calm list + extra hands if tilt > 0.5)
        // Note the mood: e.g. "tilt 0.62, AJo, open to 450".
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }

    fn postflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // 1. Rate the hand with kit::made_hand(view).
        // 2. Call limit: self.blend(self.calm_call_fraction, self.tilted_call_fraction).
        //    Compare it with kit::pot_odds(view).
        // 3. When tilted, bet and bluff more often (self.rng).
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }
}

impl Bot for Tilt {
    fn decide(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        self.seat = Some(view.seat);
        if view.board.is_empty() {
            self.preflop(view, notes)
        } else {
            self.postflop(view, notes)
        }
    }

    fn observe(&mut self, log: &HandLog) {
        // Update the mood after every hand.

        let Some(seat) = self.seat else {
            return;
        };
        let Some(me) = log.seats.iter().find(|s| s.seat == seat) else {
            return;
        };
        let old_stack = me.stack; // before the hand
        let new_stack = log.final_stacks[seat]; // after the hand
        let chip_change = new_stack as i64 - old_stack as i64;

        if chip_change < 0 {
            let bb_lost = -chip_change as f64 / log.blinds.big as f64;
            self.tilt += bb_lost * self.tilt_per_bb_lost;
        }
        self.tilt *= self.cool_down;
        self.tilt = self.tilt.clamp(0.0, 1.0);
    }
}
