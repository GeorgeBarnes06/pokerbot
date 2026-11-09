//! Running a match: a fixed number of hands with rebuys, or an elimination
//! tournament until one player has every chip.

use std::cmp::Reverse;
use std::time::{Duration, Instant};

use bots::Bot;
use engine::analysis::street_equities;
use engine::hand::Decision;
use engine::log::HandLog;
use engine::state::{Blinds, Chips, GameState, Player, PlayerStatus};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::config::TournamentConfig;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Standing {
    /// 1 for the best result.
    pub place: usize,
    pub seat: usize,
    pub name: String,
    pub bot: String,
    pub final_stack: Chips,
    /// Times this player was topped back up to the starting stack.
    pub rebuys: u64,
    /// Chips won or lost overall, summed hand by hand (top-ups and resets don't count).
    pub net_chips: i64,
    /// Net chips in big blinds per 100 hands, using the first level's big blind.
    pub bb_per_100: f64,
    /// The hand on which this player lost their last chip (elimination format only).
    pub busted_on_hand: Option<u64>,
}

/// One line per hand, for listing hands without loading each one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HandSummary {
    pub hand_number: u64,
    pub blinds: Blinds,
    pub pot: Chips,
    pub winners: Vec<usize>,
    pub showdown: bool,
    /// Chips each seat won (+) or lost (−) this hand, indexed by seat.
    pub net: Vec<i64>,
    /// Seats that never folded, i.e. still in at the end.
    pub still_in: Vec<usize>,
    /// Community cards dealt (0, 3, 4 or 5).
    pub board_cards: usize,
}

impl HandSummary {
    fn of(hand: &HandLog) -> Self {
        let mut winners: Vec<usize> = hand
            .result
            .awards
            .iter()
            .flat_map(|award| award.winners.iter().copied())
            .collect();
        winners.sort_unstable();
        winners.dedup();
        HandSummary {
            hand_number: hand.hand_number,
            blinds: hand.blinds,
            pot: hand.result.awards.iter().map(|award| award.amount).sum(),
            winners,
            showdown: !hand.result.shown.is_empty(),
            net: hand_net(hand),
            still_in: hand
                .seats
                .iter()
                .map(|s| s.seat)
                .filter(|&seat| {
                    !hand
                        .actions
                        .iter()
                        .any(|a| a.seat == seat && a.action == engine::state::Action::Fold)
                })
                .collect(),
            board_cards: hand.board.len(),
        }
    }
}

/// Chips each seat won or lost in one hand, indexed by seat (0 if not dealt in).
fn hand_net(hand: &HandLog) -> Vec<i64> {
    let mut net = vec![0; hand.final_stacks.len()];
    for start in &hand.seats {
        net[start.seat] = hand.final_stacks[start.seat] as i64 - start.stack as i64;
    }
    net
}

/// Everything about a match except the individual hands.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchSummary {
    pub config: TournamentConfig,
    /// False if an elimination tournament hit `max_hands` with several players left.
    pub finished: bool,
    pub standings: Vec<Standing>,
    /// Time spent playing the hands (engine and bots), excluding equity analysis.
    pub elapsed_ms: u64,
    pub hands_per_second: f64,
    pub hands: Vec<HandSummary>,
}

/// A finished match with every hand.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TournamentLog {
    pub config: TournamentConfig,
    pub finished: bool,
    pub standings: Vec<Standing>,
    pub elapsed_ms: u64,
    pub hands: Vec<HandLog>,
}

impl TournamentLog {
    pub fn summary(&self) -> MatchSummary {
        MatchSummary {
            config: self.config.clone(),
            finished: self.finished,
            standings: self.standings.clone(),
            elapsed_ms: self.elapsed_ms,
            hands_per_second: self.hands.len() as f64 / (self.elapsed_ms.max(1) as f64 / 1000.0),
            hands: self.hands.iter().map(HandSummary::of).collect(),
        }
    }
}

/// Plays the match described by `config`. `bots[seat]` decides for each seat.
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
    // Time spent playing (engine and bots), not recording equity for the figures.
    let mut playing = Duration::ZERO;

    let mut hands = Vec::new();
    let mut rebuys = vec![0u64; config.players.len()];
    // Chips won or lost so far, summed hand by hand, so top-ups and resets don't count.
    let mut net = vec![0i64; config.players.len()];
    // Busted seats in the order they went out, with the hand number.
    let mut eliminated: Vec<(usize, u64)> = Vec::new();

    while state.hand_number < config.max_hands && (config.is_cash() || live_players(&state) >= 2) {
        state.blinds = config.blinds_for_hand(state.hand_number + 1);
        for player in &mut state.players {
            if config.reset_stacks_each_hand {
                player.stack = config.starting_stack;
                player.status = PlayerStatus::Active;
            } else if config.rebuy && player.stack < state.blinds.big {
                player.stack = config.starting_stack;
                player.status = PlayerStatus::Active;
                rebuys[player.seat] += 1;
            }
        }

        // 53-bit seeds stay exact when the viewer parses the log as JavaScript numbers.
        let seed = rng.gen::<u64>() >> 11;
        let hand_started = Instant::now();
        let mut log = state.play_hand(seed, |view| {
            let mut notes = Vec::new();
            let action = bots[view.seat].decide(view, &mut notes);
            Decision { action, notes }
        });
        playing += hand_started.elapsed();
        if config.equity_samples > 0 {
            log.equity = street_equities(&log, config.equity_samples, seed);
        }
        for (seat, chips) in hand_net(&log).into_iter().enumerate() {
            net[seat] += chips;
        }

        for (seat, bot) in bots.iter_mut().enumerate() {
            bot.observe(&log.redacted_for(seat));
        }

        if !config.is_cash() {
            // Players busting on the same hand: the one who started with fewer chips places lower.
            let mut busted: Vec<_> = log
                .seats
                .iter()
                .filter(|s| log.final_stacks[s.seat] == 0)
                .collect();
            busted.sort_by_key(|s| s.stack);
            eliminated.extend(busted.iter().map(|s| (s.seat, log.hand_number)));
        }

        hands.push(log);
    }

    let finished = config.is_cash() || live_players(&state) < 2;
    TournamentLog {
        config: config.clone(),
        finished,
        standings: standings(config, &state, &rebuys, &net, &eliminated),
        elapsed_ms: playing.as_millis() as u64,
        hands,
    }
}

fn live_players(state: &GameState) -> usize {
    state.players.iter().filter(|p| p.stack > 0).count()
}

fn standings(
    config: &TournamentConfig,
    state: &GameState,
    rebuys: &[u64],
    net: &[i64],
    eliminated: &[(usize, u64)],
) -> Vec<Standing> {
    let order: Vec<(usize, Option<u64>)> = if config.is_cash() {
        // Cash format: rank by chips won.
        let mut seats: Vec<usize> = (0..state.players.len()).collect();
        seats.sort_by_key(|&seat| Reverse(net[seat]));
        seats.into_iter().map(|seat| (seat, None)).collect()
    } else {
        // Elimination: survivors by chips, then busted players, last out first.
        let mut survivors: Vec<&Player> = state.players.iter().filter(|p| p.stack > 0).collect();
        survivors.sort_by_key(|p| Reverse(p.stack));
        survivors
            .iter()
            .map(|p| (p.seat, None))
            .chain(eliminated.iter().rev().map(|&(seat, hand)| (seat, Some(hand))))
            .collect()
    };

    let hands = state.hand_number.max(1) as f64;
    let big_blind = config.blind_levels[0].big as f64;
    order
        .into_iter()
        .enumerate()
        .map(|(i, (seat, busted_on_hand))| Standing {
            place: i + 1,
            seat,
            name: config.players[seat].name.clone(),
            bot: config.players[seat].bot.clone(),
            final_stack: state.players[seat].stack,
            rebuys: rebuys[seat],
            net_chips: net[seat],
            bb_per_100: net[seat] as f64 / big_blind / hands * 100.0,
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
        fn decide(&mut self, view: &PlayerView, notes: &mut Vec<String>) -> Action {
            notes.push("shove".into());
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
            rebuy: false,
            reset_stacks_each_hand: false,
            equity_samples: 0,
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
    fn rebuy_match_plays_every_hand_and_nobody_busts() {
        let mut config = config(6, 5);
        config.rebuy = true;
        config.max_hands = 60;
        let log = run(&config, &mut shovers(6));

        assert!(log.finished);
        assert_eq!(log.hands.len(), 60);
        assert!(log.standings.iter().all(|s| s.busted_on_hand.is_none()));
        assert!(log.standings.iter().any(|s| s.rebuys > 0));
        // Every hand is dealt to all six players.
        assert!(log.hands.iter().all(|h| h.seats.len() == 6));
    }

    #[test]
    fn rebuy_standings_rank_by_net_chips_which_sum_to_zero() {
        let mut config = config(6, 9);
        config.rebuy = true;
        config.max_hands = 60;
        let log = run(&config, &mut shovers(6));

        let nets: Vec<i64> = log.standings.iter().map(|s| s.net_chips).collect();
        assert_eq!(nets.iter().sum::<i64>(), 0);
        assert!(nets.windows(2).all(|w| w[0] >= w[1]), "{nets:?}");
        for s in log.standings.iter().filter(|s| s.rebuys == 0) {
            assert_eq!(s.net_chips, s.final_stack as i64 - 500);
        }
    }

    #[test]
    fn rebuy_tops_up_short_stacks_below_one_big_blind() {
        let mut config = config(2, 3);
        config.rebuy = true;
        config.max_hands = 40;
        let log = run(&config, &mut shovers(2));

        for hand in &log.hands {
            assert!(hand.seats.iter().all(|s| s.stack >= hand.blinds.big), "hand {}", hand.hand_number);
        }
    }

    #[test]
    fn rebuy_net_chips_sum_to_zero_when_short_stacks_are_topped_up() {
        // Calling stations leave players with a few chips below the big blind,
        // so top-ups (not just full reloads) must be counted.
        let mut config = config(8, 1);
        config.rebuy = true;
        config.max_hands = 400;
        config.blind_levels = vec![Blinds { small: 50, big: 100, ante: 0 }];
        let mut bots: Vec<Box<dyn Bot>> = (0..8).map(|_| Box::new(CheckCall) as Box<dyn Bot>).collect();
        let log = run(&config, &mut bots);

        assert!(log.standings.iter().any(|s| s.rebuys > 0));
        assert_eq!(log.standings.iter().map(|s| s.net_chips).sum::<i64>(), 0);
    }

    #[test]
    fn summary_lists_every_hand_with_pot_and_winners() {
        let log = run(&config(4, 2), &mut shovers(4));
        let summary = log.summary();

        assert_eq!(summary.hands.len(), log.hands.len());
        let first = &summary.hands[0];
        assert_eq!(first.hand_number, 1);
        assert!(first.pot > 0);
        assert!(!first.winners.is_empty());
        for (hand, full) in summary.hands.iter().zip(&log.hands) {
            assert_eq!(hand.board_cards, full.board.len());
            assert!(hand.winners.iter().all(|w| hand.still_in.contains(w)));
        }
    }

    #[test]
    fn same_seed_replays_the_same_tournament() {
        let first = run(&config(6, 42), &mut shovers(6));
        let second = run(&config(6, 42), &mut shovers(6));

        // Everything but timing must match.
        assert_eq!(first.standings, second.standings);
        assert_eq!(first.hands.len(), second.hands.len());
        for (a, b) in first.hands.iter().zip(&second.hands) {
            assert_eq!(a.seats, b.seats);
            assert_eq!(a.board, b.board);
            assert_eq!(a.final_stacks, b.final_stacks);
            let actions = |h: &engine::log::HandLog| h.actions.iter().map(|r| (r.seat, r.action)).collect::<Vec<_>>();
            assert_eq!(actions(a), actions(b));
        }
    }

    #[test]
    fn duel_resets_stacks_every_hand_and_net_chips_balance() {
        let mut config = config(2, 8);
        config.reset_stacks_each_hand = true;
        config.max_hands = 50;
        let log = run(&config, &mut shovers(2));

        assert_eq!(log.hands.len(), 50);
        assert!(log.hands.iter().all(|h| h.seats.iter().all(|s| s.stack == 500)));
        assert_eq!(log.standings.iter().map(|s| s.net_chips).sum::<i64>(), 0);
        assert!(log.standings.iter().all(|s| s.rebuys == 0));
    }

    #[test]
    fn hand_summaries_record_each_seats_net_chips() {
        let log = run(&config(4, 2), &mut shovers(4));
        let summary = log.summary();

        let mut totals = [0i64; 4];
        for hand in &summary.hands {
            assert_eq!(hand.net.iter().sum::<i64>(), 0, "hand {}", hand.hand_number);
            for (seat, chips) in hand.net.iter().enumerate() {
                totals[seat] += chips;
            }
        }
        for standing in &summary.standings {
            assert_eq!(totals[standing.seat], standing.net_chips);
        }
        assert!(summary.hands_per_second > 0.0);
    }

    #[test]
    fn equity_and_notes_are_recorded_when_asked() {
        let mut config = config(2, 4);
        config.reset_stacks_each_hand = true;
        config.max_hands = 5;
        config.equity_samples = 100;
        let log = run(&config, &mut shovers(2));

        assert!(log.hands.iter().all(|h| !h.equity.is_empty()));
        assert!(log.hands[0].actions.iter().all(|a| a.notes == ["shove"]));
        // Bots never see each other's notes or the equity.
        let redacted = log.hands[0].redacted_for(0);
        assert!(redacted.equity.is_empty());
        assert!(redacted.actions.iter().filter(|a| a.seat == 1).all(|a| a.notes.is_empty()));
    }
}
