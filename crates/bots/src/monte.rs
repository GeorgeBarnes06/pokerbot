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

impl Monte {
    /// Expected chips won by betting `bet_fraction` of the pot, when nobody has bet.
    fn bet_value(&self, view: &PlayerView, equity: f64) -> f64 {
        // Expected chips won by betting bet_fraction of the pot: they fold assumed_fold of
        // the time (you win the pot), otherwise they call and you win equity of a bigger pot.
        let pot = view.pot as f64;
        let bet = self.bet_fraction * pot;
        let if_they_fold = self.assumed_fold * pot;
        let if_they_call = (1.0 - self.assumed_fold) * (equity * (pot + 2.0 * bet) - bet);
        if_they_fold + if_they_call
    }

    /// Expected chips won by calling a bet, compared with folding (worth 0): your share
    /// of the bigger pot, minus what the call costs.
    fn call_value(&self, view: &PlayerView, equity: f64) -> f64 {
        equity * (view.pot as f64 + view.to_call as f64) - view.to_call as f64
    }
}

impl Bot for Monte {
    fn decide(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // The same rule on every street, preflop included: estimate equity, value each
        // option in chips, take the best.

        // Equity against random hands: what share of the pot it wins on average.
        let opponents = kit::players_in_hand(view) - 1;
        let equity = kit::equity_vs_random(view, opponents, self.samples, &mut self.rng);

        // What each option is worth, in chips.
        let check = equity * view.pot as f64; // nobody has bet: see the next card free
        let bet = self.bet_value(view, equity); // nobody has bet: bet 80% of the pot
        let call = self.call_value(view, equity); // facing a bet: call (folding is worth 0)
        let pot_odds = kit::pot_odds(view);

        let action = if view.to_call == 0 {
            // Nobody has bet: bet when that's worth more than checking.
            if bet > check {
                kit::raise_pot_fraction(view, self.bet_fraction)
            } else {
                kit::check_or_call(view)
            }
        } else if call > 0.0 && equity > pot_odds + self.margin {
            // Facing a bet: call when it makes money, with a margin because random
            // opponents make its hand look better than it is.
            kit::check_or_call(view)
        } else {
            kit::check_or_fold(view)
        };

        let did = kit::describe(action);
        notes.push(format!("equity {equity:.2}, pot odds {pot_odds:.2}: {did}"));
        action
    }
}
