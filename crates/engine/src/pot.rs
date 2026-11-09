//! Main-pot and side-pot representation.

use serde::{Deserialize, Serialize};

use crate::card::Card;
use crate::evaluator::{evaluate, HandValue};
use crate::state::{Chips, GameState, Player, PlayerStatus};

/// One pot and the seats that can win it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pot {
    pub amount: Chips,
    /// Seats still in the hand that put in enough to contest this pot.
    pub eligible: Vec<usize>,
}

/// A bet nobody matched, handed back before the pots are built.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UncalledBet {
    pub seat: usize,
    pub amount: Chips,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShownHand {
    pub seat: usize,
    pub value: HandValue,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PotAward {
    pub amount: Chips,
    /// Seats that split this pot, starting left of the button.
    pub winners: Vec<usize>,
}

/// How a finished hand was paid out.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HandResult {
    pub uncalled_bet: Option<UncalledBet>,
    /// Hands revealed at showdown; empty if everyone else folded.
    pub shown: Vec<ShownHand>,
    /// Main pot first, then each side pot.
    pub awards: Vec<PotAward>,
}

/// Splits everyone's hand contributions into a main pot and side pots.
///
/// Each all-in amount caps a pot: a player can only win, from each opponent,
/// as much as they put in themselves. Folded players' chips go into the pots
/// but they are never eligible.
pub fn build_pots(players: &[Player]) -> Vec<Pot> {
    let in_hand = |p: &Player| matches!(p.status, PlayerStatus::Active | PlayerStatus::AllIn);

    let mut levels: Vec<Chips> = players
        .iter()
        .filter(|p| in_hand(p) && p.hand_contribution > 0)
        .map(|p| p.hand_contribution)
        .collect();
    levels.sort_unstable();
    levels.dedup();

    let mut pots = Vec::new();
    let mut previous = 0;
    for level in levels {
        let amount = players
            .iter()
            .map(|p| p.hand_contribution.min(level) - p.hand_contribution.min(previous))
            .sum();
        let eligible = players
            .iter()
            .filter(|p| in_hand(p) && p.hand_contribution >= level)
            .map(|p| p.seat)
            .collect();
        pots.push(Pot { amount, eligible });
        previous = level;
    }

    // Chips a folded player put in above every live contribution still belong in the pot.
    let leftover: Chips = players
        .iter()
        .map(|p| p.hand_contribution.saturating_sub(previous))
        .sum();
    if let Some(last) = pots.last_mut() {
        last.amount += leftover;
    }

    pots
}

impl GameState {
    /// Total chips committed to the pot this hand.
    pub fn pot_total(&self) -> Chips {
        self.players.iter().map(|p| p.hand_contribution).sum()
    }

    /// Pays out the pot once betting is over and reports what happened.
    ///
    /// Returns any uncalled bet, awards each pot to the best hand among its eligible
    /// players (ties split, odd chips to the first winner left of the button), and
    /// marks players left with no chips as busted.
    ///
    /// # Panics
    ///
    /// If betting is not finished.
    pub fn settle_hand(&mut self) -> HandResult {
        assert!(
            self.is_hand_over(),
            "settle_hand called before betting finished"
        );

        let uncalled_bet = self.return_uncalled_bet();
        let pots = build_pots(&self.players);

        let shown: Vec<ShownHand> = if self.players_in_hand() > 1 {
            self.players
                .iter()
                .filter(|p| matches!(p.status, PlayerStatus::Active | PlayerStatus::AllIn))
                .map(|p| ShownHand {
                    seat: p.seat,
                    value: self.hand_value(p.seat),
                })
                .collect()
        } else {
            Vec::new()
        };

        let mut awards = Vec::new();
        for pot in pots {
            let mut winners = if shown.is_empty() {
                pot.eligible.clone()
            } else {
                let value_of = |seat: usize| {
                    shown
                        .iter()
                        .find(|hand| hand.seat == seat)
                        .map(|hand| hand.value)
                        .expect("eligible seats are shown")
                };
                let best = pot.eligible.iter().map(|&seat| value_of(seat)).max();
                pot.eligible
                    .iter()
                    .copied()
                    .filter(|&seat| Some(value_of(seat)) == best)
                    .collect()
            };
            self.sort_from_left_of_button(&mut winners);

            let count = winners.len() as Chips;
            let share = pot.amount / count;
            let odd_chips = pot.amount % count;
            for (i, &seat) in winners.iter().enumerate() {
                let extra = if (i as Chips) < odd_chips { 1 } else { 0 };
                self.players[seat].stack += share + extra;
            }
            awards.push(PotAward {
                amount: pot.amount,
                winners,
            });
        }

        // The chips have moved to the winners' stacks.
        for player in &mut self.players {
            player.hand_contribution = 0;
            player.street_bet = 0;
            if player.stack == 0 {
                player.status = PlayerStatus::Busted;
            }
        }

        HandResult {
            uncalled_bet,
            shown,
            awards,
        }
    }

    /// Gives back the part of the largest contribution that nobody else matched.
    fn return_uncalled_bet(&mut self) -> Option<UncalledBet> {
        let top = self
            .players
            .iter()
            .max_by_key(|p| p.hand_contribution)?
            .seat;
        let second = self
            .players
            .iter()
            .filter(|p| p.seat != top)
            .map(|p| p.hand_contribution)
            .max()
            .unwrap_or(0);

        let player = &mut self.players[top];
        let amount = player
            .hand_contribution
            .checked_sub(second)
            .filter(|&a| a > 0)?;
        player.hand_contribution -= amount;
        player.stack += amount;
        Some(UncalledBet { seat: top, amount })
    }

    fn hand_value(&self, seat: usize) -> HandValue {
        let [first, second] = self.players[seat]
            .hole_cards
            .expect("players at showdown have hole cards");
        let board: [Card; 5] = self.board[..]
            .try_into()
            .expect("showdown needs a full board");
        evaluate([
            first, second, board[0], board[1], board[2], board[3], board[4],
        ])
    }

    fn sort_from_left_of_button(&self, seats: &mut [usize]) {
        let n = self.players.len();
        let button = self.button;
        seats.sort_by_key(|&seat| (seat + n - button - 1) % n);
    }
}

#[cfg(test)]
mod tests {
    use super::{build_pots, Pot, PotAward, UncalledBet};
    use crate::card::Card;
    use crate::evaluator::HandCategory;
    use crate::state::{Action, Blinds, Chips, GameState, Player, PlayerStatus, Street};

    fn cards(text: &str) -> Vec<Card> {
        text.split_whitespace()
            .map(|card| card.parse().expect("valid card"))
            .collect()
    }

    fn players_with(contributions: &[(Chips, PlayerStatus)]) -> Vec<Player> {
        contributions
            .iter()
            .enumerate()
            .map(|(seat, &(contribution, status))| {
                let mut player = Player::new(seat, format!("P{seat}"), 0);
                player.hand_contribution = contribution;
                player.status = status;
                player
            })
            .collect()
    }

    /// A finished hand at showdown with the given hole cards, board, and contributions.
    /// The button is on the last seat, so seat 0 is first left of the button.
    fn showdown(hands: &[(&str, Chips)], board: &str) -> GameState {
        let players = hands
            .iter()
            .enumerate()
            .map(|(seat, &(hole, contribution))| {
                let mut player = Player::new(seat, format!("P{seat}"), 0);
                let hole = cards(hole);
                player.hole_cards = Some([hole[0], hole[1]]);
                player.hand_contribution = contribution;
                player.status = PlayerStatus::AllIn;
                player
            })
            .collect();
        let mut state = GameState::new(
            players,
            hands.len() - 1,
            Blinds {
                small: 5,
                big: 10,
                ante: 0,
            },
        );
        state.board = cards(board);
        state.street = Street::Showdown;
        state
    }

    fn total_chips(state: &GameState) -> Chips {
        state
            .players
            .iter()
            .map(|p| p.stack + p.hand_contribution)
            .sum()
    }

    #[test]
    fn equal_contributions_make_one_pot() {
        use PlayerStatus::Active;
        let pots = build_pots(&players_with(&[
            (100, Active),
            (100, Active),
            (100, Active),
        ]));

        assert_eq!(
            pots,
            vec![Pot {
                amount: 300,
                eligible: vec![0, 1, 2]
            }]
        );
    }

    #[test]
    fn short_all_in_creates_side_pot() {
        use PlayerStatus::{Active, AllIn};
        let pots = build_pots(&players_with(&[(50, AllIn), (100, Active), (100, Active)]));

        assert_eq!(
            pots,
            vec![
                Pot {
                    amount: 150,
                    eligible: vec![0, 1, 2]
                },
                Pot {
                    amount: 100,
                    eligible: vec![1, 2]
                },
            ]
        );
    }

    #[test]
    fn several_all_ins_create_layered_side_pots() {
        use PlayerStatus::{Active, AllIn};
        let pots = build_pots(&players_with(&[
            (30, AllIn),
            (80, AllIn),
            (200, Active),
            (200, Active),
        ]));

        assert_eq!(
            pots,
            vec![
                Pot {
                    amount: 120,
                    eligible: vec![0, 1, 2, 3]
                },
                Pot {
                    amount: 150,
                    eligible: vec![1, 2, 3]
                },
                Pot {
                    amount: 240,
                    eligible: vec![2, 3]
                },
            ]
        );
    }

    #[test]
    fn folded_chips_count_but_folded_players_cannot_win() {
        use PlayerStatus::{Active, AllIn, Folded};
        let pots = build_pots(&players_with(&[
            (80, Folded),
            (50, AllIn),
            (100, Active),
            (100, Active),
        ]));

        assert_eq!(
            pots,
            vec![
                Pot {
                    amount: 200,
                    eligible: vec![1, 2, 3]
                },
                Pot {
                    amount: 130,
                    eligible: vec![2, 3]
                },
            ]
        );
    }

    #[test]
    fn best_hand_wins_the_pot() {
        let mut state = showdown(&[("Ah Ad", 100), ("Kh Kd", 100)], "2c 7s 9h Jd 3c");
        let result = state.settle_hand();

        assert_eq!(
            result.awards,
            vec![PotAward {
                amount: 200,
                winners: vec![0]
            }]
        );
        assert_eq!(result.shown.len(), 2);
        assert_eq!(result.shown[0].value.category, HandCategory::OnePair);
        assert_eq!(state.players[0].stack, 200);
        assert_eq!(state.players[1].status, PlayerStatus::Busted);
    }

    #[test]
    fn tied_hands_split_and_odd_chip_goes_left_of_button() {
        // Both players play the board's straight; the folded player's 1 chip makes it odd.
        let mut state = showdown(
            &[("2c 3d", 50), ("4c 5d", 50), ("6c 7d", 1)],
            "Th Js Qd Kc Ah",
        );
        state.players[2].status = PlayerStatus::Folded;
        let result = state.settle_hand();

        assert_eq!(
            result.awards,
            vec![PotAward {
                amount: 101,
                winners: vec![0, 1]
            }]
        );
        assert_eq!(state.players[0].stack, 51);
        assert_eq!(state.players[1].stack, 50);
    }

    #[test]
    fn short_stack_wins_main_pot_and_side_pot_goes_to_next_best() {
        let mut state = showdown(
            &[("Ah Ad", 50), ("Kh Kd", 100), ("Qh Qd", 100)],
            "2c 7s 9h Jd 3c",
        );
        let result = state.settle_hand();

        assert_eq!(
            result.awards,
            vec![
                PotAward {
                    amount: 150,
                    winners: vec![0]
                },
                PotAward {
                    amount: 100,
                    winners: vec![1]
                },
            ]
        );
        assert_eq!(state.players[0].stack, 150);
        assert_eq!(state.players[1].stack, 100);
        assert_eq!(state.players[2].status, PlayerStatus::Busted);
    }

    #[test]
    fn uncalled_all_in_excess_is_returned() {
        let mut state = showdown(&[("Kh Kd", 300), ("Ah Ad", 100)], "2c 7s 9h Jd 3c");
        let result = state.settle_hand();

        assert_eq!(
            result.uncalled_bet,
            Some(UncalledBet {
                seat: 0,
                amount: 200
            })
        );
        assert_eq!(
            result.awards,
            vec![PotAward {
                amount: 200,
                winners: vec![1]
            }]
        );
        assert_eq!(state.players[0].stack, 200);
        assert_ne!(state.players[0].status, PlayerStatus::Busted);
    }

    fn table(count: usize) -> GameState {
        let players = (0..count)
            .map(|seat| Player::new(seat, format!("P{seat}"), 1000))
            .collect();
        GameState::new(
            players,
            0,
            Blinds {
                small: 5,
                big: 10,
                ante: 0,
            },
        )
    }

    #[test]
    fn everyone_folding_to_a_raise_wins_blinds_and_gets_raise_back() {
        let mut state = table(8);
        state.start_hand(1);
        state.apply_action(Action::Raise { to: 30 }).unwrap(); // seat 4
        for _ in 0..7 {
            state.apply_action(Action::Fold).unwrap();
        }
        let result = state.settle_hand();

        assert_eq!(
            result.uncalled_bet,
            Some(UncalledBet {
                seat: 4,
                amount: 20
            })
        );
        assert!(result.shown.is_empty());
        assert_eq!(
            result.awards,
            vec![PotAward {
                amount: 25,
                winners: vec![4]
            }]
        );
        assert_eq!(state.players[4].stack, 1015);
        assert_eq!(total_chips(&state), 8000);
    }

    #[test]
    fn played_hand_conserves_chips() {
        let mut state = table(3);
        state.start_hand(9);
        state.apply_action(Action::Raise { to: 1000 }).unwrap();
        state.apply_action(Action::Call).unwrap();
        state.apply_action(Action::Fold).unwrap();
        assert!(state.is_hand_over());

        state.settle_hand();

        assert_eq!(total_chips(&state), 3000);
        assert_eq!(state.pot_total(), 0);
    }

    #[test]
    #[should_panic(expected = "before betting finished")]
    fn settling_mid_hand_panics() {
        let mut state = table(3);
        state.start_hand(1);
        state.settle_hand();
    }
}
