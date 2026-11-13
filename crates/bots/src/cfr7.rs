//! CFR-7.
//!
//! The name suggests counterfactual regret minimisation: a strategy trained offline by
//! self-play, usually on a simplified ("abstracted") version of the game, then
//! looked up at the table.
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
pub struct Cfr7 {
    rng: StdRng,       // seeded per seat by the runner; use it for all randomness
    samples: u32,      // run-outs per equity estimate
    bet_fraction: f64, // bet size, as a fraction of the pot (the site's CFR-7 bets 66%)
    assumed_fold: f64, // how often a bet is assumed to make the opponent fold (site: 30%)
}

impl Cfr7 {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            samples: 500,
            bet_fraction: 0.66,
            assumed_fold: 0.30,
        }
    }
}

#[allow(dead_code, unused_variables, clippy::ptr_arg)] // remove once decide calls these
impl Cfr7 {
    fn range_equity(&mut self, view: &PlayerView) -> f64 {
        // Your share of the pot against the hands opponents are *likely* to hold, not
        // any two cards (that's MONTE).
        //
        // 1. Start from kit::equity_vs_random(view, opponents, self.samples, &mut self.rng).
        // 2. Narrow it to a range: someone who raised preflop rarely holds 72o. One way is
        //    to deal random hands but skip ones outside a range list (like FOLDBOT's
        //    PREMIUM/PLAYABLE) and re-deal.
        //
        // Placeholder until written:
        0.5
    }

    fn action_values(&self, view: &PlayerView, equity: f64) -> [f64; 3] {
        // Expected chips won by each choice: [fold, call or check, bet or raise].
        //
        // - Fold: 0 (you lose nothing more).
        // - Call: equity × (pot + to_call) − to_call.
        // - Bet: assumed_fold × pot
        //        + (1 − assumed_fold) × (equity × (pot + 2 × bet) − bet),
        //   with bet = bet_fraction × pot.
        //
        // Placeholder until written:
        [0.0, 0.0, 0.0]
    }

    fn choose_mixed(&mut self, values: [f64; 3]) -> usize {
        // Mixed play, the CFR idea: don't always take the best choice; pick each with
        // probability in proportion to how much better it is than the average
        // ("regret matching").
        //
        // 1. average = mean of the three values.
        // 2. weight of each = max(value − average, 0).
        // 3. All weights 0: return the best one. Otherwise pick at random by weight
        //    with self.rng (rand::Rng's gen_range(0.0..total)).
        //
        // Placeholder until written:
        1
    }
}

impl Bot for Cfr7 {
    fn decide(&mut self, view: &PlayerView, _notes: &mut Vec<String>) -> Action {
        // 1. equity = self.range_equity(view)
        // 2. values = self.action_values(view, equity)
        // 3. choice = self.choose_mixed(values)
        // 4. 0 → kit::check_or_fold, 1 → kit::check_or_call,
        //    2 → kit::raise_pot_fraction(view, self.bet_fraction)
        // 5. Note: equity, the three values and the choice, then kit::describe(action).
        //
        // A full CFR bot trains a strategy offline by self-play and looks it up here.
        // That's a separate program; this equity-and-mixing version is what the site's
        // duel trace shows.
        //
        // Placeholder until written:
        kit::check_or_call(view)
    }
}
