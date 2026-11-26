//! TILT.
//!
//! The name suggests a player whose mood changes with results: steady when winning, looser and
//! more aggressive after losing big pots (state kept in `observe`).
//!
//! # Design
//!
//! - **Idea:** FOLDBOT's tight, aggressive style, loosened by a mood. Losing chips raises
//!   the mood; every hand it fades. The more tilted, the bigger it opens, the bigger the
//!   bets it calls, and above half tilt it plays `TILT_EXTRA` hands and bluffs weak ones.
//! - **Reads from `PlayerView`:** hole cards, board, pot, amount to call, raises this
//!   street, and its own seat (saved for `observe`).
//! - **Remembers between hands (`observe`):** its mood, `tilt` from 0.0 (calm) to 1.0. Each
//!   big blind lost adds `tilt_per_bb_lost`; every hand multiplies it by `cool_down`.
//! - **Parameters to tune:** `tilt_per_bb_lost` (0.02), `cool_down` (0.9), the calm/tilted
//!   pairs for open size (2.5/5 BB) and call limit (0.25/0.5 of the pot),
//!   `wide_range_above` (0.5), `reraise_multiple` (3), `bet_fraction` (0.75),
//!   `max_bluff_rate` (0.3), and the three hand lists.
//! - **Expected result:** with the others still check/call stubs, 1st at the 8-bot table
//!   (tilted on 27% of decisions; bigger tilted bets get paid off). Heads-up it's level with
//!   FOLDBOT (+4.1 BB/100) and RIVERRAT (-1.8), within noise over 5,000 hands.

use engine::log::HandLog;
use engine::state::Action;
use engine::view::PlayerView;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::{kit, Bot};

/// Hands it always raises, calm or not (the same as FOLDBOT's).
const PREMIUM: &[&str] = &["AA", "KK", "QQ", "JJ", "AKs", "AKo"];
/// Hands it plays when calm (the same as FOLDBOT's).
const PLAYABLE: &[&str] = &["TT", "99", "88", "AQs", "AQo", "AJs", "KQs"];
/// Extra hands it also plays once tilt is above `wide_range_above`.
#[rustfmt::skip]
const TILT_EXTRA: &[&str] = &[
    "77", "66", "55", "44", "33", "22",
    "ATs", "A9s", "A8s", "A7s", "A6s", "A5s", "A4s", "A3s", "A2s", "AJo", "ATo",
    "KJs", "KTs", "QJs", "QTs", "JTs", "KQo", "KJo",
    "T9s", "98s", "87s",
];

pub struct Tilt {
    rng: StdRng,               // seeded per seat by the runner; use it for all randomness
    seat: Option<usize>,       // own seat, learned in decide so observe can find itself
    tilt: f64,                 // mood: 0.0 calm .. 1.0 fully tilted
    tilt_per_bb_lost: f64,     // how much tilt each big blind lost in a hand adds
    cool_down: f64,            // tilt is multiplied by this after every hand (calms down)
    calm_call_fraction: f64,   // largest call, as a share of the pot, when calm
    tilted_call_fraction: f64, // largest call when fully tilted
    calm_open_size_bb: f64,    // preflop open size when calm
    tilted_open_size_bb: f64,  // preflop open size when fully tilted
    reraise_multiple: f64,     // re-raise size, as a multiple of the raise faced
    wide_range_above: f64,     // tilt above which it also plays TILT_EXTRA
    bet_fraction: f64,         // postflop bet size, as a fraction of the pot
    max_bluff_rate: f64,       // share of weak hands it bluffs when fully tilted
}

impl Tilt {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            seat: None,
            tilt: 0.0,
            tilt_per_bb_lost: 0.02,
            cool_down: 0.9,
            calm_call_fraction: 0.25,
            tilted_call_fraction: 0.5,
            calm_open_size_bb: 2.5,
            tilted_open_size_bb: 5.0,
            reraise_multiple: 3.0,
            wide_range_above: 0.5,
            bet_fraction: 0.75,
            max_bluff_rate: 0.3,
        }
    }
}

impl Tilt {
    /// A setting between its calm and tilted values, by mood: `calm` at tilt 0.0,
    /// `tilted` at 1.0, and in proportion between.
    fn blend(&self, calm: f64, tilted: f64) -> f64 {
        calm + (tilted - calm) * self.tilt
    }

    fn preflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // FOLDBOT's preflop, loosened by mood: bigger opens, bigger calls, and more hands
        // once tilted.
        let hand_name = kit::preflop_key(view.hole_cards); // e.g. "AKs"
        let raises = kit::raises_this_street(view); // 0 = nobody has raised yet
        let open_bb = self.blend(self.calm_open_size_bb, self.tilted_open_size_bb);
        let call_limit = self.blend(self.calm_call_fraction, self.tilted_call_fraction);
        let open_to = (open_bb * view.blinds.big as f64) as u64;
        let reraise_to = (self.reraise_multiple * view.current_bet as f64) as u64;

        let hand = hand_name.as_str();
        let wide = self.tilt > self.wide_range_above;
        let playable = PLAYABLE.contains(&hand) || (wide && TILT_EXTRA.contains(&hand));

        let (label, action) = if PREMIUM.contains(&hand) {
            let to = if raises == 0 { open_to } else { reraise_to };
            ("premium", kit::raise_to(view, to))
        } else if playable && raises == 0 {
            ("playable", kit::raise_to(view, open_to))
        } else if playable && raises == 1 && kit::pot_odds(view) <= call_limit {
            ("playable, cheap raise", kit::check_or_call(view))
        } else if playable {
            ("playable, too expensive", kit::check_or_fold(view))
        } else {
            ("not in range", kit::check_or_fold(view))
        };

        let did = kit::describe(action);
        let tilt = self.tilt;
        notes.push(format!(
            "tilt {tilt:.2}, {hand_name} {label}, {raises} raises: {did}"
        ));
        action
    }

    fn postflop(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        // FOLDBOT's postflop, loosened by mood: calls bigger bets, and bluffs weak hands
        // when tilted.
        let call_limit = self.blend(self.calm_call_fraction, self.tilted_call_fraction);
        let bluff_rate = self.tilt * self.max_bluff_rate; // 0 when calm
        let nobody_bet = view.to_call == 0;

        let (label, action) = match kit::strength(view) {
            kit::Strength::Strong if nobody_bet => {
                ("strong", kit::raise_pot_fraction(view, self.bet_fraction))
            }
            kit::Strength::Strong => ("strong", kit::check_or_call(view)),
            kit::Strength::Medium if nobody_bet => ("medium", kit::check_or_call(view)),
            kit::Strength::Medium if kit::pot_odds(view) <= call_limit => {
                ("medium, cheap bet", kit::check_or_call(view))
            }
            kit::Strength::Medium => ("medium, too expensive", kit::check_or_fold(view)),
            kit::Strength::Weak if nobody_bet && self.rng.gen_bool(bluff_rate) => (
                "weak, bluff",
                kit::raise_pot_fraction(view, self.bet_fraction),
            ),
            kit::Strength::Weak => ("weak", kit::check_or_fold(view)),
        };

        let did = kit::describe(action);
        let tilt = self.tilt;
        notes.push(format!("tilt {tilt:.2}, {label}: {did}"));
        action
    }
}

impl Bot for Tilt {
    fn decide(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
        self.seat = Some(view.seat);
        if view.board.is_empty() {
            self.preflop(view, notes)
        } else {
            self.postflop(view, notes)
        }
    }

    fn observe(&mut self, log: &HandLog) {
        // Update the mood after every hand.

        let Some(seat) = self.seat else {
            return;
        };
        let Some(me) = log.seats.iter().find(|s| s.seat == seat) else {
            return;
        };
        let old_stack = me.stack; // before the hand
        let new_stack = log.final_stacks[seat]; // after the hand
        let chip_change = new_stack as i64 - old_stack as i64;

        if chip_change < 0 {
            let bb_lost = -chip_change as f64 / log.blinds.big as f64;
            self.tilt += bb_lost * self.tilt_per_bb_lost;
        }
        self.tilt *= self.cool_down;
        self.tilt = self.tilt.clamp(0.0, 1.0);
    }
}
