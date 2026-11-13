//! MONTE.
//!
//! The name suggests Monte Carlo simulation: estimate hand strength by dealing out many random
//! opponent hands and boards, then compare it with the pot odds.
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

#[allow(dead_code)] // remove once the strategy uses every field
pub struct Monte {
    rng: StdRng,       // seeded per seat by the runner; use it for all randomness
    samples: u32,      // run-outs per equity estimate (more = steadier, slower)
    bet_fraction: f64, // bet size, as a fraction of the pot (the site's MONTE bets 80%)
    assumed_fold: f64, // how often a bet is assumed to make the opponent fold (site: 20%)
    margin: f64,       // equity needed above the pot odds before calling
}

impl Monte {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            samples: 1000,
            bet_fraction: 0.80,
            assumed_fold: 0.20,
            margin: 0.05,
        }
    }
}

#[allow(dead_code, unused_variables, clippy::ptr_arg)] // remove once decide calls these
impl Monte {
    fn bet_value(&self, view: &PlayerView, equity: f64) -> f64 {
        // Expected chips won by betting bet_fraction of the pot:
        //   bet = bet_fraction × pot
        //   assumed_fold × pot + (1 − assumed_fold) × (equity × (pot + 2 × bet) − bet)
        //
        // Placeholder until written:
        0.0
    }

    fn call_value(&self, view: &PlayerView, equity: f64) -> f64 {
        // Expected chips won by calling: equity × (pot + to_call) − to_call.
        //
        // Placeholder until written:
        0.0
    }
}

impl Bot for Monte {
    fn decide(&mut self, view: &PlayerView, _notes: &mut Vec<String>) -> Action {
        // The same rule on every street, so no preflop/postflop split.
        //
        // 1. opponents = kit::players_in_hand(view) − 1
        // 2. equity = kit::equity_vs_random(view, opponents, self.samples, &mut self.rng)
        //    (against any two cards: that's what makes it MONTE).
        // 3. Work out self.bet_value and self.call_value; folding is worth 0.
        // 4. Take the biggest:
        //    - betting  → kit::raise_pot_fraction(view, self.bet_fraction)
        //    - calling  → kit::check_or_call(view), only if
        //                 equity > kit::pot_odds(view) + self.margin
        //    - otherwise → kit::check_or_fold(view)
        // 5. Note: equity, pot odds and the values, then kit::describe(action).
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }
}
