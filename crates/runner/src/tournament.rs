//! Running a full tournament: hands until one player has every chip.

use std::cmp::Reverse;

use bots::Bot;
use engine::log::HandLog;
use engine::state::{Chips, GameState, Player};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::config::TournamentConfig;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Standing {
    /// 1 for the winner.
    pub place: usize,
    pub seat: usize,
    pub name: String,
    pub bot: String,
    pub final_stack: Chips,
    /// The hand on which this player lost their last chip, if they did.
    pub busted_on_hand: Option<u64>,
}

/// Everything needed to replay a tournament.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TournamentLog {
    pub config: TournamentConfig,
    /// False if `max_hands` was reached with more than one player left.
    pub finished: bool,
    pub standings: Vec<Standing>,
    pub hands: Vec<HandLog>,
}

/// Plays hands until one player is left (or `max_hands` is reached).
///
/// `bots[seat]` decides for each seat.
pub fn run(config: &TournamentConfig, bots: &mut [Box<dyn Bot>]) -> TournamentLog {
    assert_eq!(bots.len(), config.players.len(), "one bot per seat");

    let players = config
        .players
        .iter()
        .enumerate()
        .map(|(seat, p)| Player::new(seat, p.name.clone(), config.starting_stack))
        .collect();
    // start_hand moves the button before dealing, so the first button lands on seat 0.
    let last_seat = config.players.len() - 1;
    let mut state = GameState::new(players, last_seat, config.blind_levels[0]);
    let mut rng = StdRng::seed_from_u64(config.seed);

    let mut hands = Vec::new();
    // Busted seats in the order they went out, with the hand number.
    let mut eliminated: Vec<(usize, u64)> = Vec::new();

    while live_players(&state) >= 2 && state.hand_number < config.max_hands {
        state.blinds = config.blinds_for_hand(state.hand_number + 1);
        let log = state.play_hand(rng.gen(), |view| bots[view.seat].decide(view));

        for (seat, bot) in bots.iter_mut().enumerate() {
            bot.observe(&log.redacted_for(seat));
        }

        // Players busting on the same hand: the one who started with fewer chips places lower.
        let mut busted: Vec<_> = log
            .seats
            .iter()
            .filter(|s| log.final_stacks[s.seat] == 0)
            .collect();
        busted.sort_by_key(|s| s.stack);
        eliminated.extend(busted.iter().map(|s| (s.seat, log.hand_number)));

        hands.push(log);
    }

    let finished = live_players(&state) < 2;
    TournamentLog {
        config: config.clone(),
        finished,
        standings: standings(config, &state, &eliminated),
        hands,
    }
}

fn live_players(state: &GameState) -> usize {
    state.players.iter().filter(|p| p.stack > 0).count()
}

fn standings(
    config: &TournamentConfig,
    state: &GameState,
    eliminated: &[(usize, u64)],
) -> Vec<Standing> {
    // Survivors rank by chips; then busted players, last out first.
    let mut survivors: Vec<&Player> = state.players.iter().filter(|p| p.stack > 0).collect();
    survivors.sort_by_key(|p| Reverse(p.stack));

    let order = survivors
        .iter()
        .map(|p| (p.seat, None))
        .chain(eliminated.iter().rev().map(|&(seat, hand)| (seat, Some(hand))));

    order
        .enumerate()
        .map(|(i, (seat, busted_on_hand))| Standing {
            place: i + 1,
            seat,
            name: config.players[seat].name.clone(),
            bot: config.players[seat].bot.clone(),
            final_stack: state.players[seat].stack,
            busted_on_hand,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use bots::{Bot, CheckCall};
    use engine::state::{Action, Blinds};
    use engine::view::PlayerView;

    use super::run;
    use crate::config::{PlayerConfig, TournamentConfig};

    /// Goes all-in whenever it can: makes tournaments end quickly.
    struct Shove;

    impl Bot for Shove {
        fn decide(&mut self, view: &PlayerView) -> Action {
            match view.legal.raise_range {
                Some((_, max)) => Action::Raise { to: max },
                None => Action::Call,
            }
        }
    }

    fn config(players: usize, seed: u64) -> TournamentConfig {
        TournamentConfig {
            seed,
            starting_stack: 500,
            hands_per_level: 5,
            blind_levels: vec![
                Blinds { small: 5, big: 10, ante: 0 },
                Blinds { small: 25, big: 50, ante: 5 },
            ],
            max_hands: 2000,
            players: (0..players)
                .map(|i| PlayerConfig {
                    name: format!("P{i}"),
                    bot: "check_call".into(),
                })
                .collect(),
        }
    }

    fn shovers(count: usize) -> Vec<Box<dyn Bot>> {
        (0..count).map(|_| Box::new(Shove) as Box<dyn Bot>).collect()
    }

    #[test]
    fn tournament_runs_until_one_player_has_every_chip() {
        let config = config(6, 7);
        let log = run(&config, &mut shovers(6));

        assert!(log.finished);
        assert_eq!(log.standings.len(), 6);
        assert_eq!(log.standings[0].final_stack, 3000);
        assert!(log.standings[0].busted_on_hand.is_none());
        assert!(log.standings[1..].iter().all(|s| s.busted_on_hand.is_some()));

        let mut places: Vec<usize> = log.standings.iter().map(|s| s.place).collect();
        places.sort();
        assert_eq!(places, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn chips_are_conserved_every_hand() {
        let config = config(8, 3);
        let log = run(&config, &mut shovers(8));

        for hand in &log.hands {
            assert_eq!(hand.final_stacks.iter().sum::<u64>(), 4000, "hand {}", hand.hand_number);
        }
    }

    #[test]
    fn later_busts_place_higher() {
        let log = run(&config(5, 11), &mut shovers(5));

        let busts: Vec<u64> = log.standings[1..]
            .iter()
            .map(|s| s.busted_on_hand.unwrap())
            .collect();
        assert!(busts.windows(2).all(|w| w[0] >= w[1]), "{busts:?}");
    }

    #[test]
    fn blinds_follow_the_schedule() {
        let mut config = config(4, 1);
        config.max_hands = 8;
        let mut bots: Vec<Box<dyn Bot>> = (0..4).map(|_| Box::new(CheckCall) as Box<dyn Bot>).collect();
        let log = run(&config, &mut bots);

        assert_eq!(log.hands[0].blinds.big, 10);
        assert_eq!(log.hands[4].blinds.big, 10);
        assert_eq!(log.hands[5].blinds.big, 50);
        assert_eq!(log.hands[5].blinds.ante, 5);
    }

    #[test]
    fn max_hands_stops_an_unfinished_tournament() {
        let mut config = config(4, 1);
        config.max_hands = 3;
        let mut bots: Vec<Box<dyn Bot>> = (0..4).map(|_| Box::new(CheckCall) as Box<dyn Bot>).collect();
        let log = run(&config, &mut bots);

        assert_eq!(log.hands.len(), 3);
        assert!(!log.finished);
        assert!(log.standings.iter().filter(|s| s.busted_on_hand.is_none()).count() >= 2);
    }

    #[test]
    fn same_seed_replays_the_same_tournament() {
        let first = run(&config(6, 42), &mut shovers(6));
        let second = run(&config(6, 42), &mut shovers(6));

        assert_eq!(first, second);
    }
}
