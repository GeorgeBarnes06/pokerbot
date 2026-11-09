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

use engine::state::Action;
use engine::view::PlayerView;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::{kit, Bot};

pub struct Bayes {
    /// Seeded per seat by the runner, so matches replay exactly. Use it for all randomness.
    #[allow(dead_code)] // used once the strategy is written
    rng: StdRng,
}

impl Bayes {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl Bot for Bayes {
    fn decide(&mut self, view: &PlayerView, _notes: &mut Vec<String>) -> Action {
        // TODO(design): BAYES's strategy goes here. Placeholder: check or call.
        kit::check_or_call(view)
    }
}
