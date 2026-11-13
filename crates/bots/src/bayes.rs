//! BAYES.
//!
//! The name suggests opponent modelling: update beliefs about each opponent's tendencies from
//! every hand seen (`observe`), and adjust to exploit them.
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

use std::collections::HashMap;

use engine::log::HandLog;
use engine::state::Action;
use engine::view::PlayerView;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::{kit, Bot};

/// What BAYES has seen one opponent do, counted in `observe`.
#[allow(dead_code)] // remove once observe fills these in
#[derive(Clone, Debug, Default)]
struct OpponentStats {
    hands: u32,          // hands this opponent was dealt in
    played_preflop: u32, // hands they put chips in voluntarily (called or raised)
    raised_preflop: u32, // hands they raised preflop
    faced_bet: u32,      // times they faced a bet after the flop
    folded_to_bet: u32,  // ...and folded
}

#[allow(dead_code)] // remove once the strategy uses every field
pub struct Bayes {
    rng: StdRng,                               // seeded per seat by the runner
    seat: Option<usize>,                       // own seat, learned in decide
    opponents: HashMap<String, OpponentStats>, // by player name (stable across hands)
    prior_strength: f64,                       // pretend hands seen before any data
    prior_fold_rate: f64,                      // assumed fold-to-bet rate with no data
    open_size_bb: f64,                         // preflop open size, in big blinds
    bet_fraction: f64,                         // postflop bet size, as a fraction of the pot
}

impl Bayes {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            seat: None,
            opponents: HashMap::new(),
            prior_strength: 10.0,
            prior_fold_rate: 0.4,
            open_size_bb: 2.5,
            bet_fraction: 0.66,
        }
    }
}

#[allow(dead_code, unused_variables, clippy::ptr_arg)] // remove once decide calls these and notes is used
impl Bayes {
    fn fold_rate(&self, name: &str) -> f64 {
        // Belief about how often this opponent folds to a bet, starting from the prior
        // and moving towards what's been seen as data comes in:
        //   (folded_to_bet + prior_fold_rate × prior_strength) / (faced_bet + prior_strength)
        // Unknown name: self.prior_fold_rate.
        //
        // Placeholder until written:
        self.prior_fold_rate
    }

    fn main_opponent<'a>(&self, view: &'a PlayerView) -> Option<&'a str> {
        // The opponent to adjust to: the last player to bet or raise this hand
        // (view.history), or the only other player left in the pot. Return their name
        // from view.seats.
        //
        // Placeholder until written:
        None
    }

    fn preflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // A solid default range (like FOLDBOT's lists), adjusted per opponent:
        // - against someone who plays lots of hands (played_preflop / hands high), play
        //   more and raise for value;
        // - against someone who raises rarely, respect their raises and fold more.
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }

    fn postflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // 1. opponent = self.main_opponent(view); belief = self.fold_rate(name).
        // 2. Bluff more against opponents who fold a lot, never against ones who don't.
        // 3. Bet strong hands for value; bigger against opponents who call too much.
        // 4. Note the belief: e.g. "MONTE folds 62% to bets, bluff".
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }
}

impl Bot for Bayes {
    fn decide(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        self.seat = Some(view.seat);
        if view.board.is_empty() {
            self.preflop(view, notes)
        } else {
            self.postflop(view, notes)
        }
    }

    fn observe(&mut self, log: &HandLog) {
        // Count what each opponent did this hand.
        //
        // For each seat in log.seats except your own (self.seat):
        //   stats = self.opponents.entry(name.clone()).or_default()
        //   stats.hands += 1
        //   Look through log.actions for that seat:
        //   - preflop Call or Raise → played_preflop (once per hand)
        //   - preflop Raise         → raised_preflop (once per hand)
        //   - after the flop, facing a bet: faced_bet += 1; if they Fold, folded_to_bet += 1
        //
        // Blinds are posted, not actions, so they don't count as playing a hand.
        let _ = log;
    }
}
