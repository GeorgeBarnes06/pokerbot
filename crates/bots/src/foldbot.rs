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

use engine::evaluator::HandCategory;
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

pub struct FoldBot {
    #[allow(dead_code)] // unused until the strategy needs randomness
    rng: StdRng, // seeded per seat by the runner; use it for all randomness
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
        let action = match strength(view) {
            Strength::Strong => {
                if view.to_call == 0 {
                    //if nobody has bet
                    kit::raise_pot_fraction(view, self.bet_fraction) // bet 75% of the pot
                } else {
                    kit::check_or_call(view)
                }
            }
            Strength::Medium => {
                if view.to_call == 0 {
                    //if nobody has bet
                    kit::check_or_call(view)
                } else {
                    // if somebody has bet
                    let pot_odds = kit::pot_odds(view);
                    if pot_odds <= self.max_call_fraction {
                        kit::check_or_call(view)
                    } else {
                        kit::check_or_fold(view)
                    }
                }
            }
            Strength::Weak => {
                if view.to_call == 0 {
                    kit::check_or_call(view)
                } else {
                    kit::check_or_fold(view)
                }
            }
        };
        let did = kit::describe(action);
        notes.push(format!("postflop: {did}"));
        action
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

fn strength(view: &PlayerView) -> Strength {
    let hand = kit::made_hand(view).unwrap();

    if hand.category >= HandCategory::TwoPair {
        return Strength::Strong;
    }
    if hand.category == HandCategory::HighCard {
        return Strength::Weak;
    }
    let pair = hand.kickers[0];
    let top = view
        .board
        .iter()
        .map(|card| card.rank.get_rank())
        .max()
        .unwrap();
    let c1 = view.hole_cards[0].rank.get_rank();
    let c2 = view.hole_cards[1].rank.get_rank();
    if c1 != pair && c2 != pair {
        return Strength::Weak;
    } else if pair >= top {
        return Strength::Strong;
    }
    Strength::Medium
}
