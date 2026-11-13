//! RIVERRAT.
//!
//! The name suggests a loose player who calls a lot and chases draws all the way to the river.
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

/// Starting hands it plays: wide. Fill in, e.g. any pair, any ace, suited connectors.
#[allow(dead_code)] // remove once preflop uses it
const PLAYABLE: &[&str] = &[];

#[allow(dead_code)] // remove once the strategy uses every field
pub struct RiverRat {
    rng: StdRng,             // seeded per seat by the runner; use it for all randomness
    max_call_fraction: f64,  // largest call with a made hand, as a share of the pot
    draw_call_fraction: f64, // largest call while chasing a draw
    open_size_bb: f64,       // preflop raise size, when it raises at all
}

impl RiverRat {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            max_call_fraction: 0.5,
            draw_call_fraction: 0.35,
            open_size_bb: 2.0,
        }
    }
}

#[allow(dead_code, unused_variables, clippy::ptr_arg)] // remove once decide calls these and notes is used
impl RiverRat {
    fn preflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // Loose and passive: see lots of flops cheaply.
        //
        // 1. hand in PLAYABLE: call (kit::check_or_call), unless the raise is huge.
        // 2. Rarely raise; that's not its style.
        // 3. Otherwise kit::check_or_fold(view).
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }

    fn postflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // Calls with any piece of the board, and chases draws to the river.
        //
        // 1. Made hand (a pair or better, kit::made_hand): call up to max_call_fraction.
        // 2. Drawing (has_flush_draw or has_straight_draw), not on the river: call up to
        //    draw_call_fraction.
        // 3. On the river with a missed draw: check or fold (or bluff with self.rng).
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }

    fn has_flush_draw(view: &PlayerView) -> bool {
        // Four cards of one suit among hole cards + board, using at least one hole card.
        // Count suits with card.suit.
        //
        // Placeholder until written:
        false
    }

    fn has_straight_draw(view: &PlayerView) -> bool {
        // Four ranks in a row (an open-ended draw) among hole cards + board. Remember an
        // ace also counts as 1 (A-2-3-4).
        //
        // Placeholder until written:
        false
    }
}

impl Bot for RiverRat {
    fn decide(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        if view.board.is_empty() {
            self.preflop(view, notes)
        } else {
            self.postflop(view, notes)
        }
    }
}
