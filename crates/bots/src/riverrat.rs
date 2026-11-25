//! RIVERRAT.
//!
//! The name suggests a loose player who calls a lot and chases draws all the way to the river.
//!
//! # Design
//!
//! - **Idea:** A loose, passive player (a "calling station"). Preflop it calls with a wide
//!   `PLAYABLE` range and rarely raises. Postflop it calls with any pair or better, and
//!   chases flush and straight draws until the river; with nothing, it checks or folds.
//! - **Reads from `PlayerView`:** hole cards, board, street, pot and amount to call.
//! - **Remembers between hands (`observe`):** nothing; every hand is played on its own.
//! - **Parameters to tune:** the `PLAYABLE` list, `max_call_fraction` (0.5),
//!   `draw_call_fraction` (0.35), `open_size_bb` (2.0).
//! - **Expected result:** slightly better than `check_call`, because it folds its worst hands
//!   and missed draws, but losing to tight bots like FOLDBOT that bet their strong hands
//!   into it.

use engine::state::Action;
use engine::view::PlayerView;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::{kit, Bot};

/// Starting hands it plays: wide (46 hands, about 20% of all deals; FOLDBOT plays about 6%).
#[rustfmt::skip]
const PLAYABLE: &[&str] = &[
    // every pair
    "AA", "KK", "QQ", "JJ", "TT", "99", "88", "77", "66", "55", "44", "33", "22",
    // any suited ace
    "AKs", "AQs", "AJs", "ATs", "A9s", "A8s", "A7s", "A6s", "A5s", "A4s", "A3s", "A2s",
    // big offsuit aces
    "AKo", "AQo", "AJo", "ATo", "A9o",
    // two picture cards (or a ten)
    "KQs", "KJs", "KTs", "QJs", "QTs", "JTs", "KQo", "KJo", "QJo",
    // suited connectors
    "T9s", "98s", "87s", "76s", "65s", "54s",
];

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
        // Loose and passive: see lots of flops cheaply, never raise.
        let hand_name = kit::preflop_key(view.hole_cards); // e.g. "87s"
        let raises = kit::raises_this_street(view); // 0 = nobody has raised yet

        // Playable: call the big blind or a single raise; fold to a re-raise.
        if PLAYABLE.contains(&hand_name.as_str()) {
            let action = if raises <= 1 {
                kit::check_or_call(view)
            } else {
                kit::check_or_fold(view)
            };
            let did = kit::describe(action);
            notes.push(format!("playable {hand_name}, {raises} raises: {did}"));
            return action;
        }

        // Everything else: check if free, otherwise fold.
        let action = kit::check_or_fold(view);
        let did = kit::describe(action);
        notes.push(format!("{hand_name} not in range: {did}"));
        action
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
