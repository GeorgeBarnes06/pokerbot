//! Game state, players, and seat status.

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
    /// Whether the player has acted since the street began or since the last full raise.
    pub acted_this_street: bool,
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
    /// `from` itself is checked last, so it is returned only if no other seat matches.
    pub fn next_seat(&self, from: usize, pred: impl Fn(&Player) -> bool) -> Option<usize> {
        let n = self.players.len();
        (1..=n)
            .map(|offset| (from + offset) % n)
            .find(|&seat| pred(&self.players[seat]))
    }

    pub fn start_hand(&mut self, seed: u64) {
        self.hand_number += 1;
        for player in &mut self.players {
            player.hole_cards = None;
            player.street_bet = 0;
            player.hand_contribution = 0;
            player.acted_this_street = false;
            if player.status != PlayerStatus::Busted {
                player.status = PlayerStatus::Active;
            }
        }
        self.board.clear();
        self.street = Street::Preflop;
        self.last_aggressor = None;

        let not_busted = |p: &Player| p.status != PlayerStatus::Busted;
        self.button = self
            .next_seat(self.button, not_busted)
            .expect("button must be on a non-busted player");

        self.deck = Deck::new();
        self.deck.shuffle(seed);

        // Deal like a real dealer: one card to each live player starting left of
        // the button, then a second round.
        let n = self.players.len();
        let order: Vec<usize> = (1..=n)
            .map(|offset| (self.button + offset) % n)
            .filter(|&seat| not_busted(&self.players[seat]))
            .collect();
        let first_cards: Vec<Card> = order
            .iter()
            .map(|_| self.deck.draw().expect("deck has enough cards"))
            .collect();
        for (&seat, first_card) in order.iter().zip(first_cards) {
            let second_card = self.deck.draw().expect("deck has enough cards");
            self.players[seat].hole_cards = Some([first_card, second_card]);
        }

        if self.blinds.ante > 0 {
            for seat in 0..n {
                if not_busted(&self.players[seat]) {
                    self.post_ante(seat, self.blinds.ante);
                }
            }
        }

        let live_count = self.players.iter().filter(|p| not_busted(p)).count();
        let (small_blind_seat, big_blind_seat) = if live_count == 2 {
            // Heads-up: the button posts the small blind.
            let big = self.next_seat(self.button, not_busted).expect("two players");
            (self.button, big)
        } else {
            let small = self.next_seat(self.button, not_busted).expect("at least one player");
            let big = self.next_seat(small, not_busted).expect("at least two players");
            (small, big)
        };
        self.post(small_blind_seat, self.blinds.small);
        self.post(big_blind_seat, self.blinds.big);

        self.current_bet = self.blinds.big;
        self.min_raise = self.blinds.big;
        // First to act is left of the big blind; heads-up this lands back on the button.
        // If blinds put everyone all-in, this runs the board out instead.
        self.pass_turn(big_blind_seat);
    }

    /// Moves up to `amount` from `seat`'s stack into the pot; returns what was actually posted.
    pub(crate) fn post(&mut self, seat: usize, amount: Chips) -> Chips {
        let posted = self.commit(seat, amount);
        self.players[seat].street_bet += posted;
        posted
    }

    /// Moves up to `amount` into the pot without counting toward the street's bet,
    /// as an ante does; returns what was actually posted.
    fn post_ante(&mut self, seat: usize, amount: Chips) -> Chips {
        self.commit(seat, amount)
    }

    fn commit(&mut self, seat: usize, amount: Chips) -> Chips {
        let player = &mut self.players[seat];
        let posted = amount.min(player.stack);
        player.stack -= posted;
        player.hand_contribution += posted;
        if player.stack == 0 {
            player.status = PlayerStatus::AllIn;
        }
        posted
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
            acted_this_street: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{Blinds, GameState, Player, PlayerStatus};
    use crate::card::Card;
    use crate::deck::Deck;

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

    #[test]
    fn start_hand_moves_button_and_posts_blinds() {
        let mut state = table(8);
        state.start_hand(1);

        assert_eq!(state.hand_number, 1);
        assert_eq!(state.button, 1);
        assert_eq!(state.players[2].street_bet, 5);
        assert_eq!(state.players[2].stack, 995);
        assert_eq!(state.players[3].street_bet, 10);
        assert_eq!(state.players[3].hand_contribution, 10);
        assert_eq!(state.players[3].stack, 990);
        assert_eq!(state.current_bet, 10);
        assert_eq!(state.min_raise, 10);
        assert_eq!(state.to_act, Some(4));
    }

    #[test]
    fn start_hand_skips_busted_seats() {
        let mut state = table(8);
        state.players[1].status = PlayerStatus::Busted;
        state.players[3].status = PlayerStatus::Busted;
        state.start_hand(1);

        assert_eq!(state.button, 2);
        assert_eq!(state.players[4].street_bet, 5);
        assert_eq!(state.players[5].street_bet, 10);
        assert_eq!(state.to_act, Some(6));
    }

    #[test]
    fn start_hand_heads_up_button_posts_small_blind_and_acts_first() {
        let mut state = table(2);
        state.start_hand(1);

        assert_eq!(state.button, 1);
        assert_eq!(state.players[1].street_bet, 5);
        assert_eq!(state.players[0].street_bet, 10);
        assert_eq!(state.to_act, Some(1));
    }

    #[test]
    fn short_stack_goes_all_in_on_big_blind() {
        let mut state = table(8);
        state.players[3].stack = 6;
        state.start_hand(1);

        assert_eq!(state.players[3].street_bet, 6);
        assert_eq!(state.players[3].stack, 0);
        assert_eq!(state.players[3].status, PlayerStatus::AllIn);
    }

    #[test]
    fn start_hand_deals_two_unique_cards_to_live_players() {
        let mut state = table(8);
        state.players[5].status = PlayerStatus::Busted;
        state.start_hand(1);

        let mut seen = HashSet::new();
        for player in &state.players {
            match player.hole_cards {
                Some([a, b]) => {
                    assert!(seen.insert(a) && seen.insert(b), "duplicate card");
                }
                None => assert_eq!(player.status, PlayerStatus::Busted),
            }
        }
        assert_eq!(seen.len(), 14);
        assert_eq!(state.deck.cards.len(), 52 - 14);
    }

    #[test]
    fn start_hand_deals_one_card_per_round_from_left_of_button() {
        let mut state = table(3);
        state.start_hand(7);

        // Recreate the same shuffled deck; draw() takes from the end.
        let mut deck = Deck::new();
        deck.shuffle(7);
        let draws: Vec<Card> = (0..6).map(|_| deck.draw().unwrap()).collect();

        // Button moves to seat 1, so dealing order is seats 2, 0, 1.
        assert_eq!(state.players[2].hole_cards, Some([draws[0], draws[3]]));
        assert_eq!(state.players[0].hole_cards, Some([draws[1], draws[4]]));
        assert_eq!(state.players[1].hole_cards, Some([draws[2], draws[5]]));
    }

    #[test]
    fn same_seed_deals_same_hands() {
        let mut first = table(8);
        let mut second = table(8);
        first.start_hand(42);
        second.start_hand(42);

        for (a, b) in first.players.iter().zip(&second.players) {
            assert_eq!(a.hole_cards, b.hole_cards);
        }
    }

    #[test]
    fn busted_player_gets_no_cards() {
        let mut state = table(8);
        state.start_hand(1);
        state.players[6].status = PlayerStatus::Busted;
        state.start_hand(2);

        assert_eq!(state.players[6].hole_cards, None);
    }

    #[test]
    fn antes_go_into_the_pot_but_not_the_bet_to_call() {
        let mut state = table(8);
        state.blinds.ante = 2;
        state.start_hand(1);

        assert_eq!(state.players[4].stack, 998);
        assert_eq!(state.players[4].street_bet, 0);
        assert_eq!(state.players[4].hand_contribution, 2);
        assert_eq!(state.players[3].street_bet, 10);
        assert_eq!(state.players[3].hand_contribution, 12);
        assert_eq!(state.current_bet, 10);
    }

    #[test]
    fn start_hand_clears_previous_hand() {
        let mut state = table(8);
        state.start_hand(1);
        state.players[4].status = PlayerStatus::Folded;
        state.start_hand(2);

        assert_eq!(state.hand_number, 2);
        assert_eq!(state.players[4].status, PlayerStatus::Active);
        assert_eq!(state.players[2].street_bet, 0);
        assert_eq!(state.players[2].hand_contribution, 0);
    }
}
