//! What one player is allowed to see when it is their turn.

use serde::{Deserialize, Serialize};

use crate::betting::LegalActions;
use crate::card::Card;
use crate::log::ActionRecord;
use crate::state::{Blinds, Chips, GameState, PlayerStatus, Street};

/// Public information about one seat: everything except hole cards.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SeatView {
    pub seat: usize,
    pub name: String,
    pub stack: Chips,
    pub status: PlayerStatus,
    pub street_bet: Chips,
    pub hand_contribution: Chips,
}

/// The table from the point of view of the player to act.
///
/// Contains the player's own hole cards but never anyone else's, and nothing
/// about the deck.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlayerView {
    pub seat: usize,
    pub hand_number: u64,
    pub hole_cards: [Card; 2],
    pub board: Vec<Card>,
    pub street: Street,
    pub button: usize,
    pub blinds: Blinds,
    pub pot: Chips,
    pub current_bet: Chips,
    pub to_call: Chips,
    pub legal: LegalActions,
    pub seats: Vec<SeatView>,
    /// Every action taken so far this hand, in order. Other players' notes are removed.
    pub history: Vec<ActionRecord>,
}

impl GameState {
    /// The view for the player whose turn it is, or `None` if nobody is to act.
    pub fn current_view(&self, history: &[ActionRecord]) -> Option<PlayerView> {
        let seat = self.to_act?;
        let legal = self.legal_actions()?;
        let hole_cards = self.players[seat]
            .hole_cards
            .expect("player to act has hole cards");

        Some(PlayerView {
            seat,
            hand_number: self.hand_number,
            hole_cards,
            board: self.board.clone(),
            street: self.street,
            button: self.button,
            blinds: self.blinds,
            pot: self.pot_total(),
            current_bet: self.current_bet,
            to_call: self.to_call(seat),
            legal,
            seats: self
                .players
                .iter()
                .map(|p| SeatView {
                    seat: p.seat,
                    name: p.name.clone(),
                    stack: p.stack,
                    status: p.status,
                    street_bet: p.street_bet,
                    hand_contribution: p.hand_contribution,
                })
                .collect(),
            history: history
                .iter()
                .map(|record| ActionRecord {
                    notes: if record.seat == seat { record.notes.clone() } else { Vec::new() },
                    ..record.clone()
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{Blinds, GameState, Player};

    #[test]
    fn view_shows_own_cards_and_betting_spot() {
        let players = (0..4)
            .map(|seat| Player::new(seat, format!("P{seat}"), 1000))
            .collect();
        let mut state = GameState::new(players, 0, Blinds { small: 5, big: 10, ante: 0 });
        state.start_hand(3);

        let view = state.current_view(&[]).expect("someone to act");

        assert_eq!(view.seat, 0);
        assert_eq!(Some(view.hole_cards), state.players[0].hole_cards);
        assert_eq!(view.pot, 15);
        assert_eq!(view.to_call, 10);
        assert_eq!(view.legal, state.legal_actions().unwrap());
        assert_eq!(view.seats.len(), 4);
        assert_eq!(view.seats[3].street_bet, 10);
    }

    #[test]
    fn no_view_when_nobody_is_to_act() {
        let players = (0..2)
            .map(|seat| Player::new(seat, format!("P{seat}"), 1000))
            .collect();
        let state = GameState::new(players, 0, Blinds { small: 5, big: 10, ante: 0 });

        assert_eq!(state.current_view(&[]), None);
    }
}
