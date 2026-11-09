//! Hand analysis with every card known, for logs and figures.
//!
//! This uses information no player has (everyone's hole cards), so bots must
//! never call it during a hand.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::card::Card;
use crate::deck::Deck;
use crate::evaluator::evaluate;
use crate::log::{HandLog, SeatEquity, StreetEquity};
use crate::state::{Action, Street};

/// Each hand's share of the pot, on average, over random completions of the board.
///
/// `dead` cards (folded hands) can't come on the board. With a full board the
/// answer is exact; otherwise it is estimated from `samples` random run-outs.
pub fn equity(hands: &[[Card; 2]], board: &[Card], dead: &[Card], samples: u32, rng: &mut StdRng) -> Vec<f64> {
    let mut shares = vec![0.0; hands.len()];
    if hands.len() == 1 {
        shares[0] = 1.0;
        return shares;
    }

    let used: Vec<Card> = hands.iter().flatten().chain(board).chain(dead).copied().collect();
    let mut rest: Vec<Card> = Deck::new().cards.into_iter().filter(|c| !used.contains(c)).collect();
    let missing = 5 - board.len();
    let runs = if missing == 0 { 1 } else { samples.max(1) };

    // Placeholder fill; every slot past the known board is overwritten each run.
    let mut full_board = [rest[0]; 5];
    full_board[..board.len()].copy_from_slice(board);
    let mut values = Vec::with_capacity(hands.len());

    for _ in 0..runs {
        let (drawn, _) = rest.partial_shuffle(rng, missing);
        full_board[board.len()..].copy_from_slice(drawn);

        values.clear();
        values.extend(hands.iter().map(|[a, b]| {
            let b5 = full_board;
            evaluate([*a, *b, b5[0], b5[1], b5[2], b5[3], b5[4]])
        }));
        let best = *values.iter().max().expect("at least two hands");
        let winners = values.iter().filter(|&&v| v == best).count() as f64;
        for (share, value) in shares.iter_mut().zip(&values) {
            if *value == best {
                *share += 1.0 / winners;
            }
        }
    }

    for share in &mut shares {
        *share /= runs as f64;
    }
    shares
}

/// Equity of every live player at the start of each street the hand reached.
///
/// Players who folded on an earlier street are out, and their cards are dead.
/// Deterministic for a given `seed`.
pub fn street_equities(log: &HandLog, samples: u32, seed: u64) -> Vec<StreetEquity> {
    let mut rng = StdRng::seed_from_u64(seed);
    let streets = [
        (Street::Preflop, 0),
        (Street::Flop, 3),
        (Street::Turn, 4),
        (Street::River, 5),
    ];

    let mut result = Vec::new();
    for (street, board_len) in streets {
        if log.board.len() < board_len {
            break;
        }
        let folded_before = |seat: usize| {
            log.actions
                .iter()
                .any(|a| a.seat == seat && a.street < street && a.action == Action::Fold)
        };

        let mut live = Vec::new();
        let mut dead = Vec::new();
        for start in &log.seats {
            let Some(cards) = start.hole_cards else { continue };
            if folded_before(start.seat) {
                dead.extend(cards);
            } else {
                live.push((start.seat, cards));
            }
        }
        if live.len() < 2 {
            break;
        }

        let hands: Vec<[Card; 2]> = live.iter().map(|&(_, cards)| cards).collect();
        let shares = equity(&hands, &log.board[..board_len], &dead, samples, &mut rng);
        result.push(StreetEquity {
            street,
            seats: live
                .iter()
                .zip(shares)
                .map(|(&(seat, _), equity)| SeatEquity { seat, equity })
                .collect(),
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::{equity, street_equities};
    use crate::card::Card;
    use crate::state::{Action, Blinds, GameState, Player, Street};

    fn cards(text: &str) -> Vec<Card> {
        text.split_whitespace().map(|c| c.parse().unwrap()).collect()
    }

    fn hand(text: &str) -> [Card; 2] {
        let c = cards(text);
        [c[0], c[1]]
    }

    #[test]
    fn full_board_is_exact() {
        let mut rng = StdRng::seed_from_u64(1);
        let shares = equity(
            &[hand("Ah Ad"), hand("Kh Kd")],
            &cards("2c 7s 9h Jd 3c"),
            &[],
            100,
            &mut rng,
        );
        assert_eq!(shares, vec![1.0, 0.0]);
    }

    #[test]
    fn identical_hands_split() {
        let mut rng = StdRng::seed_from_u64(1);
        let shares = equity(
            &[hand("Ah Kd"), hand("As Kc")],
            &cards("2c 7s 9h Jd 3d"),
            &[],
            100,
            &mut rng,
        );
        assert_eq!(shares, vec![0.5, 0.5]);
    }

    #[test]
    fn aces_are_big_favourites_preflop_and_shares_sum_to_one() {
        let mut rng = StdRng::seed_from_u64(7);
        let shares = equity(&[hand("Ah Ad"), hand("7c 2d")], &[], &[], 4000, &mut rng);

        assert!(shares[0] > 0.8 && shares[0] < 0.92, "{shares:?}");
        assert!((shares.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn dead_cards_never_reach_the_board() {
        // Every remaining heart is dead, so the flush draw can't come in.
        let mut rng = StdRng::seed_from_u64(3);
        let dead = cards("2h 3h 4h 5h 6h");
        let shares = equity(
            &[hand("Ah Kh"), hand("Qs Qc")],
            &cards("7h 8h 2s"),
            &[dead, cards("9h Th Jh Qh")].concat(),
            2000,
            &mut rng,
        );
        assert!(shares[1] > 0.7, "{shares:?}");
    }

    #[test]
    fn street_equities_follow_the_hand_and_drop_folded_players() {
        let players = (0..3)
            .map(|seat| Player::new(seat, format!("P{seat}"), 1000))
            .collect();
        let mut state = GameState::new(players, 0, Blinds { small: 5, big: 10, ante: 0 });
        // Button folds preflop; the blinds check it down.
        let mut log = state.play_hand(4, |view| {
            if view.history.is_empty() {
                Action::Fold
            } else if view.legal.can_check {
                Action::Check
            } else {
                Action::Call
            }
        });
        log.equity = street_equities(&log, 200, 1);

        let streets: Vec<Street> = log.equity.iter().map(|e| e.street).collect();
        assert_eq!(streets, vec![Street::Preflop, Street::Flop, Street::Turn, Street::River]);
        // Everyone starts preflop; the button's fold removes them from the flop on.
        assert_eq!(log.equity[0].seats.len(), 3);
        assert_eq!(log.equity[1].seats.len(), 2);
        for street in &log.equity {
            let total: f64 = street.seats.iter().map(|s| s.equity).sum();
            assert!((total - 1.0).abs() < 1e-9);
        }
        assert_eq!(street_equities(&log, 200, 1), log.equity);
    }
}
