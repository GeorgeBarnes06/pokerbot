//! GTO-LITE.
//!
//! The name suggests a simplified game-theory-optimal strategy: fixed preflop range charts,
//! a few standard bet sizes, and mixed (randomised) choices in close spots.
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

use engine::state::Action;
use engine::view::PlayerView;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::{kit, Bot};

/// Hands to open when nobody has raised.
#[allow(dead_code)] // remove once preflop uses them
const OPEN_RANGE: &[&str] = &[];
/// Hands to call a raise with.
#[allow(dead_code)]
const CALL_RANGE: &[&str] = &[];
/// Hands to re-raise (3-bet) with.
#[allow(dead_code)]
const THREE_BET_RANGE: &[&str] = &[];

#[allow(dead_code)] // remove once the strategy uses every field
pub struct GtoLite {
    rng: StdRng,          // seeded per seat by the runner; use it for all randomness
    open_size_bb: f64,    // preflop open size, in big blinds
    small_bet: f64,       // small postflop bet, as a fraction of the pot
    big_bet: f64,         // big postflop bet, as a fraction of the pot
    bluff_frequency: f64, // share of weak hands that bet anyway
}

impl GtoLite {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            open_size_bb: 2.5,
            small_bet: 0.33,
            big_bet: 0.75,
            bluff_frequency: 0.3,
        }
    }
}

#[allow(dead_code, unused_variables, clippy::ptr_arg)] // remove once decide calls these and notes is used
impl GtoLite {
    /// True with the given probability, using the bot's seeded rng. For mixed
    /// choices: `if self.mix(0.3) { bluff }`.
    fn mix(&mut self, probability: f64) -> bool {
        self.rng.gen_bool(probability.clamp(0.0, 1.0))
    }

    fn preflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // Chart-based: look the hand up in the three lists.
        //
        // 1. hand = kit::preflop_key(view.hole_cards), raises = kit::raises_this_street(view)
        // 2. Nobody raised: OPEN_RANGE → raise to open_size_bb big blinds; else check or fold.
        // 3. Facing a raise: THREE_BET_RANGE → re-raise; CALL_RANGE → call; else fold.
        // 4. Mix the edges: for hands on the border of a list, use self.mix(0.5).
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }

    fn postflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // Two sizes, picked by hand type:
        //
        // 1. Rate the hand with kit::made_hand(view).
        // 2. Very strong: bet big_bet. Medium: bet small_bet or check.
        // 3. Weak: bet small_bet when self.mix(self.bluff_frequency), else check or fold.
        // 4. Facing a bet: call if kit::pot_odds(view) is low enough for the hand.
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }
}

impl Bot for GtoLite {
    fn decide(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        if view.board.is_empty() {
            self.preflop(view, notes)
        } else {
            self.postflop(view, notes)
        }
    }
}
