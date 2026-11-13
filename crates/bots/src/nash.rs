//! NASH.
//!
//! The name suggests Nash-equilibrium play: ranges and frequencies chosen so no opponent can
//! exploit them, e.g. push/fold charts once stacks get short.
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
use rand::SeedableRng;

use crate::{kit, Bot};

/// Hands to go all-in with when short-stacked. Fill in, e.g. from a push/fold chart.
#[allow(dead_code)] // remove once push_or_fold uses it
const PUSH_RANGE: &[&str] = &[];
/// Hands to call someone else's all-in with when short-stacked.
#[allow(dead_code)]
const CALL_PUSH_RANGE: &[&str] = &[];

#[allow(dead_code)] // remove once the strategy uses every field
pub struct Nash {
    rng: StdRng,             // seeded per seat by the runner; use it for all randomness
    push_fold_below_bb: f64, // at or below this many big blinds, play push/fold only
    open_size_bb: f64,       // normal preflop open size, in big blinds
    bet_fraction: f64,       // postflop bet size, as a fraction of the pot
}

impl Nash {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            push_fold_below_bb: 15.0,
            open_size_bb: 2.5,
            bet_fraction: 0.5,
        }
    }
}

#[allow(dead_code, unused_variables, clippy::ptr_arg)] // remove once decide calls these and notes is used
impl Nash {
    fn preflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // 1. Short stack (kit::stack_in_big_blinds(view) <= self.push_fold_below_bb):
        //    return self.push_or_fold(view, notes).
        // 2. Otherwise play balanced ranges: open, call or 3-bet by hand list, and use
        //    self.rng so the same hand isn't always played the same way.
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }

    fn push_or_fold(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // 1. hand = kit::preflop_key(view.hole_cards)
        // 2. Nobody all-in yet and hand in PUSH_RANGE: kit::all_in(view).
        // 3. Facing an all-in and hand in CALL_PUSH_RANGE: kit::check_or_call(view).
        // 4. Otherwise kit::check_or_fold(view).
        //
        // Real push/fold charts depend on seat and stack size; one list is a start.
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }

    fn postflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // Balanced betting: bet strong hands *and* some weak ones (bluffs) at the same
        // size, so the bet gives nothing away.
        //
        // 1. Rate the hand with kit::made_hand(view).
        // 2. Strong: bet self.bet_fraction of the pot.
        // 3. Weak: bluff a set share of the time (self.rng), otherwise check or fold.
        // 4. Facing a bet: call when kit::pot_odds(view) is below your estimated equity.
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }
}

impl Bot for Nash {
    fn decide(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        if view.board.is_empty() {
            self.preflop(view, notes)
        } else {
            self.postflop(view, notes)
        }
    }
}
