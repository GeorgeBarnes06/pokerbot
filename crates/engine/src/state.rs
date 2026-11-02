use serde::{Deserialize, Serialize};

use crate::card::Card;
use crate::deck::Deck;

pub type Chips = u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatus {
    /// In the hand and able to act.
    Active,
    /// Gave up this hand.
    Folded,
    /// In the hand, but has no chips left to bet.
    AllIn,
    /// Eliminated from the tournament.
    Busted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub seat: usize,
    pub name: String,
    pub stack: Chips,
    pub hole_cards: Option<[Card; 2]>,
    pub status: PlayerStatus,
    /// Chips put in on the current street; reset each street.
    pub street_bet: Chips,
    /// Chips put in over the whole hand; used to build side pots.
    pub hand_contribution: Chips,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Street {
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Fold,
    Check,
    Call,
    Raise { to: Chips },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Blinds {
    pub small: Chips,
    pub big: Chips,
    pub ante: Chips,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub hand_number: u64,
    pub players: Vec<Player>,
    pub button: usize,
    pub blinds: Blinds,
    pub deck: Deck,
    pub board: Vec<Card>,
    pub street: Street,
    /// Seat whose turn it is, or `None` between betting rounds.
    pub to_act: Option<usize>,
    /// Highest `street_bet` on this street (the amount others must match).
    pub current_bet: Chips,
    /// Minimum size of the next raise: the size of the last raise, or the big blind.
    pub min_raise: Chips,
    /// Last player to bet or raise this street; the round ends when action returns to them.
    pub last_aggressor: Option<usize>,
}

impl GameState {
    pub fn new(players: Vec<Player>, button: usize, blinds: Blinds) -> Self {
        Self {
            hand_number: 0,
            players,
            button,
            blinds,
            deck: Deck::new(),
            board: Vec::new(),
            street: Street::Preflop,
            to_act: None,
            current_bet: 0,
            min_raise: blinds.big,
            last_aggressor: None,
        }
    }

    /// First seat clockwise after `from` whose player matches `pred`.
    ///
    /// `from` itself is checked last, so it is returned only if no other seat matches.
    pub fn next_seat(&self, from: usize, pred: impl Fn(&Player) -> bool) -> Option<usize> {
        let n = self.players.len();
        (1..=n)
            .map(|offset| (from + offset) % n)
            .find(|&seat| pred(&self.players[seat]))
    }
}

impl Player {
    pub fn new(seat: usize, name: impl Into<String>, stack: Chips) -> Self {
        Self {
            seat,
            name: name.into(),
            stack,
            hole_cards: None,
            status: PlayerStatus::Active,
            street_bet: 0,
            hand_contribution: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Blinds, GameState, Player, PlayerStatus};

    fn table(count: usize) -> GameState {
        let players = (0..count)
            .map(|seat| Player::new(seat, format!("P{seat}"), 1000))
            .collect();
        GameState::new(players, 0, Blinds { small: 5, big: 10, ante: 0 })
    }

    fn not_busted(player: &Player) -> bool {
        player.status != PlayerStatus::Busted
    }

    #[test]
    fn new_table_seats_players_with_starting_stack() {
        let state = table(8);

        assert_eq!(state.players.len(), 8);
        assert!(state.players.iter().all(|p| p.stack == 1000));
        assert!(state.players.iter().enumerate().all(|(i, p)| p.seat == i));
    }

    #[test]
    fn next_seat_returns_following_seat() {
        let state = table(8);

        assert_eq!(state.next_seat(3, |_| true), Some(4));
    }

    #[test]
    fn next_seat_wraps_from_last_seat_to_first() {
        let state = table(8);

        assert_eq!(state.next_seat(7, |_| true), Some(0));
    }

    #[test]
    fn next_seat_skips_busted_players() {
        let mut state = table(8);
        state.players[1].status = PlayerStatus::Busted;
        state.players[2].status = PlayerStatus::Busted;

        assert_eq!(state.next_seat(0, not_busted), Some(3));
    }

    #[test]
    fn next_seat_skips_busted_players_across_wrap() {
        let mut state = table(8);
        state.players[7].status = PlayerStatus::Busted;
        state.players[0].status = PlayerStatus::Busted;

        assert_eq!(state.next_seat(6, not_busted), Some(1));
    }

    #[test]
    fn next_seat_returns_from_seat_when_it_is_the_only_match() {
        let mut state = table(2);
        state.players[1].status = PlayerStatus::Busted;

        assert_eq!(state.next_seat(0, not_busted), Some(0));
    }

    #[test]
    fn next_seat_heads_up_alternates_between_two_seats() {
        let state = table(2);

        assert_eq!(state.next_seat(0, not_busted), Some(1));
        assert_eq!(state.next_seat(1, not_busted), Some(0));
    }

    #[test]
    fn next_seat_returns_none_when_nobody_matches() {
        let state = table(8);

        assert_eq!(state.next_seat(0, |_| false), None);
    }
}
