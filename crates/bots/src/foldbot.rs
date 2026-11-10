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

use engine::state::Action;
use engine::view::PlayerView;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::{kit, Bot};

#[allow(dead_code)] // used once the strategy is written
const PREMIUM: &[&str] = &["AA", "KK", "QQ", "JJ", "AKs", "AKo"];
#[allow(dead_code)]
const PLAYABLE: &[&str] = &["TT", "99", "88", "AQs", "AQo", "AJs", "KQs"];

#[allow(dead_code)] // fields used once the strategy is written
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

#[allow(dead_code, unused_variables)] // remove once decide calls these and notes is used
impl FoldBot {
    fn preflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // 1. Get the hand name: kit::preflop_key(view.hole_cards), e.g. "AKs".
        // 2. Count raises so far: kit::raises_this_street(view).
        //    Blinds don't count, so 0 means nobody has raised yet.
        //
        // 3. PREMIUM:
        //    - Nobody has raised: raise to open_size_bb big blinds
        //      (kit::raise_to(view, open_size_bb * view.blinds.big)).
        //    - Someone raised: re-raise to reraise_multiple × view.current_bet.
        //
        // 4. PLAYABLE:
        //    - Nobody has raised: open the same way as a premium.
        //    - One raise, and the call is cheap (kit::pot_odds(view) at most
        //      max_call_fraction): call (kit::check_or_call).
        //    - Otherwise: fold (kit::check_or_fold).
        //
        // 5. Anything else: kit::check_or_fold(view). A free check in the big blind
        //    is taken; everything else folds.
        //
        // 6. Before returning, push one note, e.g. "premium AKs, raise to 300".
        //
        // Placeholder until written:
        kit::check_or_call(view)
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
    fn decide(&mut self, view: &PlayerView, _notes: &mut Vec<String>) -> Action {
        // TODO(design): FOLDBOT's strategy goes here. Placeholder: check or call.
        kit::check_or_call(view)
    }
}
