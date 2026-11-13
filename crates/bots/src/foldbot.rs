//! FOLDBOT.
//!
//! The name suggests a very tight player: folds everything except strong hands, and bets those.
//!
//! # Design
//!
//! - **Idea:** A tight, aggressive player. Preflop it plays only `PREMIUM` and `PLAYABLE`
//!   hands and folds the rest. Premiums raise and re-raise; playables open, or call a single
//!   cheap raise. Postflop it sorts its hand into strong, medium or weak: strong hands bet,
//!   medium hands check and call small bets, weak hands check or fold.
//! - **Reads from `PlayerView`:** hole cards, board, pot, amount to call, own stack, and the
//!   raises made this street.
//! - **Remembers between hands (`observe`):** nothing; every hand is played on its own.
//! - **Parameters to tune:** the `PREMIUM` and `PLAYABLE` lists, `open_size_bb` (3.0),
//!   `reraise_multiple` (3.0), `bet_fraction` (0.75), `max_call_fraction` (0.33).
//! - **Expected result:** clearly positive BB/100 against `check_call`, which pays off its
//!   strong hands. Small but steady wins in the 8-bot table, folding most hands and losing
//!   little apart from blinds.

#[allow(unused_imports)] // remove once strength uses them
use engine::evaluator::{evaluate_best, HandCategory};
use engine::state::Action;
use engine::view::PlayerView;

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::{kit, Bot};

const PREMIUM: &[&str] = &["AA", "KK", "QQ", "JJ", "AKs", "AKo"];
const PLAYABLE: &[&str] = &["TT", "99", "88", "AQs", "AQo", "AJs", "KQs"];

enum Strength {
    Strong,
    Medium,
    Weak,
}

#[allow(dead_code)] // rng and bet_fraction are used once postflop is written
pub struct FoldBot {
    rng: StdRng,            // seeded per seat by the runner; use it for all randomness
    open_size_bb: f64,      // preflop open size, in big blinds
    reraise_multiple: f64,  // re-raise size, as a multiple of the raise faced
    bet_fraction: f64,      // postflop bet size, as a fraction of the pot
    max_call_fraction: f64, // largest call allowed, as a fraction of the pot
}

impl FoldBot {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            open_size_bb: 3.0,
            reraise_multiple: 3.0,
            bet_fraction: 0.75,
            max_call_fraction: 0.33,
        }
    }
}

#[allow(unused_variables, clippy::ptr_arg)] // remove once postflop uses notes
impl FoldBot {
    fn preflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        let hand_name = kit::preflop_key(view.hole_cards); // e.g. "AKs"
        let raises = kit::raises_this_street(view); // 0 = nobody has raised yet
        let open_to = (self.open_size_bb * view.blinds.big as f64) as u64; // 3.0 × 100 = 300
        let reraise_to = (self.reraise_multiple * view.current_bet as f64) as u64; // 3.0 × their raise

        // Situations 1 and 2: premium hand
        if PREMIUM.contains(&hand_name.as_str()) {
            let action = if raises == 0 {
                kit::raise_to(view, open_to)
            } else {
                kit::raise_to(view, reraise_to)
            };
            let did = kit::describe(action);
            notes.push(format!("premium {hand_name}, {raises} raises: {did}"));
            return action;
        }

        // Situations 3 and 4: playable hand
        if PLAYABLE.contains(&hand_name.as_str()) {
            let action = if raises == 0 {
                kit::raise_to(view, open_to)
            } else if raises == 1 && kit::pot_odds(view) <= self.max_call_fraction {
                kit::check_or_call(view)
            } else {
                kit::check_or_fold(view)
            };
            let did = kit::describe(action);
            notes.push(format!("playable {hand_name}, {raises} raises: {did}"));
            return action;
        }

        // Situation 5: everything else
        let action = kit::check_or_fold(view);
        let did = kit::describe(action);
        notes.push(format!("{hand_name} not in range: {did}"));
        action
    }

    fn postflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // 1. Work out how strong the hand is: hole cards + view.board through the
        //    evaluator, sorted into Strong / Medium / Weak (a small `strength`
        //    helper returning an enum keeps this tidy).
        //
        // 2. Strong (two pair or better, or top pair with a good kicker):
        //    - Nothing to call (view.to_call == 0): bet bet_fraction of the pot
        //      (kit::raise_pot_fraction(view, bet_fraction)).
        //    - Facing a bet: call (kit::check_or_call). Optionally raise with
        //      very strong hands (a set or better).
        //
        // 3. Medium (any other pair):
        //    - Nothing to call: check.
        //    - Facing a bet: call only if kit::pot_odds(view) is at most
        //      max_call_fraction; otherwise fold.
        //
        // 4. Weak (no pair): kit::check_or_fold(view).
        //
        // 5. Before returning, push one note, e.g. "medium pair, call 1/4 pot".
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }
}

impl Bot for FoldBot {
    fn decide(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        if view.board.is_empty() {
            self.preflop(view, notes)
        } else {
            self.postflop(view, notes)
        }
    }
}

#[allow(dead_code)] // remove once postflop calls it
fn strength(view: &PlayerView) -> Strength {
    todo!("FOLDBOT: rate the hand with kit::made_hand(view)") // placeholder so the crate builds
}
