//! Playing a complete hand from the deal to the payout.

use crate::log::{ActionRecord, HandLog, SeatStart};
use crate::state::{Action, GameState, PlayerStatus};
use crate::view::PlayerView;

impl GameState {
    /// Plays one full hand, asking `decide` for every action, and returns its log.
    ///
    /// An illegal decision is replaced with a check if one is allowed, otherwise a
    /// fold, and recorded as rejected.
    ///
    /// # Panics
    ///
    /// If fewer than two players have chips.
    pub fn play_hand(&mut self, seed: u64, mut decide: impl FnMut(&PlayerView) -> Action) -> HandLog {
        self.start_hand(seed);

        let seats = self
            .players
            .iter()
            .filter(|p| p.status != PlayerStatus::Busted)
            .map(|p| SeatStart {
                seat: p.seat,
                name: p.name.clone(),
                stack: p.stack + p.hand_contribution,
                hole_cards: p.hole_cards,
                posted: p.hand_contribution,
            })
            .collect();
        let (hand_number, button, blinds) = (self.hand_number, self.button, self.blinds);

        let mut actions = Vec::new();
        while let Some(view) = self.current_view(&actions) {
            let seat = view.seat;
            let street = self.street;
            let stack_before = self.players[seat].stack;

            let requested = decide(&view);
            let (action, rejected) = match self.apply_action(requested) {
                Ok(()) => (requested, None),
                Err(_) => {
                    let fallback = if view.legal.can_check {
                        Action::Check
                    } else {
                        Action::Fold
                    };
                    self.apply_action(fallback)
                        .expect("check or fold is always legal");
                    (fallback, Some(requested))
                }
            };

            actions.push(ActionRecord {
                street,
                seat,
                action,
                amount: stack_before - self.players[seat].stack,
                rejected,
            });
        }

        let board = self.board.clone();
        let result = self.settle_hand();

        HandLog {
            hand_number,
            seed,
            button,
            blinds,
            seats,
            actions,
            board,
            result,
            final_stacks: self.players.iter().map(|p| p.stack).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{Action, Blinds, Chips, GameState, Player, PlayerStatus};
    use crate::view::PlayerView;

    fn table(count: usize) -> GameState {
        let players = (0..count)
            .map(|seat| Player::new(seat, format!("P{seat}"), 1000))
            .collect();
        GameState::new(players, 0, Blinds { small: 5, big: 10, ante: 0 })
    }

    fn check_or_call(view: &PlayerView) -> Action {
        if view.legal.can_check {
            Action::Check
        } else {
            Action::Call
        }
    }

    fn shove(view: &PlayerView) -> Action {
        match view.legal.raise_range {
            Some((_, max)) => Action::Raise { to: max },
            None => Action::Call,
        }
    }

    fn total(stacks: &[Chips]) -> Chips {
        stacks.iter().sum()
    }

    #[test]
    fn calling_down_reaches_showdown_and_conserves_chips() {
        let mut state = table(4);
        let log = state.play_hand(5, check_or_call);

        assert_eq!(log.hand_number, 1);
        assert_eq!(log.seats.len(), 4);
        assert_eq!(log.board.len(), 5);
        assert_eq!(log.result.shown.len(), 4);
        assert_eq!(total(&log.final_stacks), 4000);
        assert!(log.actions.iter().all(|a| a.rejected.is_none()));
    }

    #[test]
    fn log_records_blinds_and_action_amounts() {
        let mut state = table(3);
        let log = state.play_hand(5, check_or_call);

        // Button on seat 1, blinds on seats 2 and 0.
        assert_eq!(log.seats[2].posted, 5);
        assert_eq!(log.seats[0].posted, 10);
        assert!(log.seats.iter().all(|s| s.stack == 1000));
        assert_eq!(log.actions[0].seat, 1);
        assert_eq!(log.actions[0].action, Action::Call);
        assert_eq!(log.actions[0].amount, 10);
        assert_eq!(log.actions[1].amount, 5);
    }

    #[test]
    fn illegal_decision_becomes_check_or_fold() {
        let mut state = table(3);
        let log = state.play_hand(5, |_| Action::Raise { to: 1 });

        let first = log.actions[0];
        assert_eq!(first.action, Action::Fold);
        assert_eq!(first.rejected, Some(Action::Raise { to: 1 }));
        // Everyone folds to the big blind, who wins without showing.
        assert!(log.result.shown.is_empty());
        assert_eq!(total(&log.final_stacks), 3000);
    }

    #[test]
    fn all_in_hands_bust_players_and_conserve_chips() {
        let mut state = table(4);
        state.players[2].stack = 300;
        let log = state.play_hand(11, shove);

        assert_eq!(total(&log.final_stacks), 3300);
        assert!(state
            .players
            .iter()
            .filter(|p| p.stack == 0)
            .all(|p| p.status == PlayerStatus::Busted));
    }

    #[test]
    fn players_only_see_their_own_cards_in_redacted_log() {
        let mut state = table(3);
        let log = state.play_hand(5, |_| Action::Fold);
        let redacted = log.redacted_for(1);

        assert!(redacted.seats[1].hole_cards.is_some());
        assert!(redacted.seats[0].hole_cards.is_none());
        assert!(redacted.seats[2].hole_cards.is_none());
    }

    #[test]
    fn shown_hands_stay_visible_in_redacted_log() {
        let mut state = table(3);
        let log = state.play_hand(5, check_or_call);
        let redacted = log.redacted_for(0);

        assert!(redacted.seats.iter().all(|s| s.hole_cards.is_some()));
    }
}
