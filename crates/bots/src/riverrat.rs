//! RIVERRAT.
//!
//! The name suggests a loose player who calls a lot and chases draws all the way to the river.
//!
//! # Design
//!
//! - **Idea:** A loose, passive player (a "calling station"). Preflop it calls with a wide
//!   `PLAYABLE` range (and completes the small blind with anything) and never raises.
//!   Postflop it bets only two pair or better; otherwise it checks when it can, calls with
//!   any pair of its own or better, and chases flush and straight draws until the river;
//!   with nothing, it folds.
//! - **Reads from `PlayerView`:** hole cards, board, street, pot and amount to call.
//! - **Remembers between hands (`observe`):** nothing; every hand is played on its own.
//! - **Parameters to tune:** the `PLAYABLE` list, `max_call_fraction` (0.5),
//!   `draw_call_fraction` (0.35), `bet_fraction` (0.5).
//! - **Expected result:** beats `check_call` (+65.4 BB/100 heads-up) by betting its strong
//!   hands into it, and finishes 2nd at the 8-bot table behind FOLDBOT (while the others are
//!   still check/call stubs). Heads-up against FOLDBOT it's about even (+2.2 BB/100 over
//!   5,000 hands, within noise): FOLDBOT folds most small blinds.

use engine::evaluator::HandCategory;
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

pub struct RiverRat {
    #[allow(dead_code)] // unused until the strategy needs randomness
    rng: StdRng, // seeded per seat by the runner; use it for all randomness
    max_call_fraction: f64, // largest call with a made hand, as a share of the pot
    draw_call_fraction: f64, // largest call while chasing a draw
    bet_fraction: f64,      // bet size with two pair or better, as a share of the pot
}

impl RiverRat {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            max_call_fraction: 0.5,
            draw_call_fraction: 0.35,
            bet_fraction: 0.5,
        }
    }
}

impl RiverRat {
    fn preflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // Loose and passive: see lots of flops cheaply, never raise.
        let hand_name = kit::preflop_key(view.hole_cards); // e.g. "87s"
        let raises = kit::raises_this_street(view); // 0 = nobody has raised yet

        // Small blind, nobody raised: completing costs half a big blind, so see a flop with
        // any two cards. Folding here gives away a blind every time.
        if raises == 0 && view.to_call > 0 && view.to_call <= view.blinds.small {
            let action = kit::check_or_call(view);
            let did = kit::describe(action);
            notes.push(format!("{hand_name}, cheap to complete: {did}"));
            return action;
        }

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
        // Mostly passive: bets only two pair or better, otherwise checks when it can. Facing
        // a bet, it calls with any piece of the board, and with a draw until the river.
        let strong = kit::made_hand(view).is_some_and(|h| h.category >= HandCategory::TwoPair);
        let hand = has_own_pair_or_better(view);
        let drawing =
            view.board.len() < 5 && (kit::has_flush_draw(view) || kit::has_straight_draw(view));
        let pot_odds = kit::pot_odds(view);

        let (label, action) = if view.to_call == 0 && strong {
            (
                "two pair or better",
                kit::raise_pot_fraction(view, self.bet_fraction),
            )
        } else if view.to_call == 0 {
            ("free", kit::check_or_call(view))
        } else if hand && pot_odds <= self.max_call_fraction {
            ("made hand", kit::check_or_call(view))
        } else if drawing && pot_odds <= self.draw_call_fraction {
            ("drawing", kit::check_or_call(view))
        } else {
            ("nothing worth the price", kit::check_or_fold(view))
        };

        let did = kit::describe(action);
        notes.push(format!("{label}, pot odds {pot_odds:.2}: {did}"));
        action
    }
}

/// A pair that uses one of your hole cards, or anything better (two pair and up).
/// A pair that's only on the board doesn't count: everyone has it.
fn has_own_pair_or_better(view: &PlayerView) -> bool {
    let hand = kit::made_hand(view).expect("postflop always has a board");
    match hand.category {
        HandCategory::HighCard => false,
        HandCategory::OnePair => view
            .hole_cards
            .iter()
            .any(|card| card.rank.get_rank() == hand.kickers[0]),
        _ => true,
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
