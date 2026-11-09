//! TILT.
//!
//! The name suggests a player whose mood changes with results: steady when winning, looser and
//! more aggressive after losing big pots (state kept in `observe`).
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

pub struct Tilt {
    /// Seeded per seat by the runner, so matches replay exactly. Use it for all randomness.
    #[allow(dead_code)] // used once the strategy is written
    rng: StdRng,
}

impl Tilt {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl Bot for Tilt {
    fn decide(&mut self, view: &PlayerView, _notes: &mut Vec<String>) -> Action {
        // TODO(design): TILT's strategy goes here. Placeholder: check or call.
        kit::check_or_call(view)
    }
}
