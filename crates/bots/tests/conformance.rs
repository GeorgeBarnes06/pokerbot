//! Rules every model must follow, whatever its strategy:
//! - only legal actions (an illegal one is silently turned into a check or fold),
//! - no panics, at a full table or heads-up,
//! - decisions within `DECISION_BUDGET` (checked in release builds only:
//!   `cargo test --release -p bots`).

use bots::{create, Bot, DECISION_BUDGET, MODELS};
use engine::hand::Decision;
use engine::log::HandLog;
use engine::state::{Blinds, GameState, Player, PlayerStatus};

const BLINDS: Blinds = Blinds { small: 50, big: 100, ante: 0 };
const STACK: u64 = 10_000;

/// Plays `hands` hands with rebuys, like the default match, and returns every log.
fn play(names: &[&str], hands: u64) -> Vec<HandLog> {
    let mut bots: Vec<Box<dyn Bot>> = names
        .iter()
        .enumerate()
        .map(|(seat, name)| create(name, seat as u64).unwrap_or_else(|| panic!("unknown bot {name}")))
        .collect();
    let players = names
        .iter()
        .enumerate()
        .map(|(seat, name)| Player::new(seat, *name, STACK))
        .collect();
    let mut state = GameState::new(players, names.len() - 1, BLINDS);

    let mut logs = Vec::new();
    for hand in 0..hands {
        for player in &mut state.players {
            if player.stack < BLINDS.big {
                player.stack = STACK;
                player.status = PlayerStatus::Active;
            }
        }
        let log = state.play_hand(hand, |view| {
            let mut notes = Vec::new();
            let action = bots[view.seat].decide(view, &mut notes);
            Decision { action, notes }
        });
        for (seat, bot) in bots.iter_mut().enumerate() {
            bot.observe(&log.redacted_for(seat));
        }
        logs.push(log);
    }
    logs
}

fn assert_legal(logs: &[HandLog], names: &[&str]) {
    for log in logs {
        for action in &log.actions {
            assert!(
                action.rejected.is_none(),
                "{} made an illegal move on hand {}: asked for {:?}",
                names[action.seat],
                log.hand_number,
                action.rejected
            );
        }
    }
}

fn assert_within_budget(logs: &[HandLog], names: &[&str]) {
    if cfg!(debug_assertions) {
        return; // Debug builds are too slow to judge; run with --release.
    }
    let budget = DECISION_BUDGET.as_micros() as u64;
    for log in logs {
        for action in &log.actions {
            assert!(
                action.think_us <= budget,
                "{} took {}ms on hand {} (budget {}ms)",
                names[action.seat],
                action.think_us / 1000,
                log.hand_number,
                budget / 1000
            );
        }
    }
}

#[test]
fn all_eight_models_play_a_full_table_legally() {
    let names: Vec<&str> = MODELS.iter().map(|(name, _)| *name).collect();
    let logs = play(&names, 300);

    assert_legal(&logs, &names);
    assert_within_budget(&logs, &names);
}

#[test]
fn each_model_plays_heads_up_legally() {
    for (name, _) in MODELS {
        let names = [name, "check_call"];
        let logs = play(&names, 200);

        assert_legal(&logs, &names);
        assert_within_budget(&logs, &names);
    }
}

#[test]
fn each_model_handles_short_stacks_and_all_ins() {
    // Tiny stacks force all-ins, side pots, and players with no legal raise.
    for (name, _) in MODELS {
        let names = [name, "check_call", name];
        let mut bots: Vec<Box<dyn Bot>> = names.iter().map(|n| create(n, 1).unwrap()).collect();
        let players = names
            .iter()
            .enumerate()
            .map(|(seat, n)| Player::new(seat, *n, 150 + 100 * seat as u64))
            .collect();
        let mut state = GameState::new(players, 2, BLINDS);
        let log = state.play_hand(9, |view| {
            let mut notes = Vec::new();
            let action = bots[view.seat].decide(view, &mut notes);
            Decision { action, notes }
        });
        assert_legal(&[log], &names);
    }
}

#[test]
fn every_model_name_is_registered() {
    for (name, _) in MODELS {
        assert!(create(name, 0).is_some(), "{name} missing from bots::create");
    }
    assert!(create("check_call", 0).is_some());
    assert!(create("nobody", 0).is_none());
}
