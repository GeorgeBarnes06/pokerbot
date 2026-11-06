//! Betting-round state machine.

use crate::state::{Action, Chips, GameState, PlayerStatus, Street};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionError {
    /// Nobody is due to act (between hands, or the hand is over).
    NoPlayerToAct,
    /// Tried to check while facing a bet.
    CannotCheck,
    /// Tried to call with nothing to call; check instead.
    NothingToCall,
    /// Raise total is below the minimum and is not an all-in.
    RaiseTooSmall,
    /// Raise needs more chips than the player has.
    RaiseTooLarge,
    /// Raising is not open to this player: they only face an incomplete all-in
    /// raise, can't put in more than the current bet, or nobody is left to call.
    RaiseNotAllowed,
}

/// What the player to act may do.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegalActions {
    pub can_check: bool,
    /// Chips a call would put in, capped at the player's stack; `None` if there is nothing to call.
    pub call_amount: Option<Chips>,
    /// Smallest and largest `Raise { to }` totals, if raising is allowed.
    pub raise_range: Option<(Chips, Chips)>,
}

impl GameState {
    /// Chips `seat` must add to match the current bet.
    pub fn to_call(&self, seat: usize) -> Chips {
        self.current_bet.saturating_sub(self.players[seat].street_bet)
    }

    /// Players still contesting the pot (not folded or busted).
    pub fn players_in_hand(&self) -> usize {
        self.players
            .iter()
            .filter(|p| matches!(p.status, PlayerStatus::Active | PlayerStatus::AllIn))
            .count()
    }

    /// True once betting is finished: everyone else folded, or the board is complete.
    pub fn is_hand_over(&self) -> bool {
        self.players_in_hand() <= 1 || self.street == Street::Showdown
    }

    /// The options open to the player whose turn it is, or `None` if nobody is to act.
    pub fn legal_actions(&self) -> Option<LegalActions> {
        let seat = self.to_act?;
        let player = &self.players[seat];
        let to_call = self.to_call(seat);

        let all_in_total = player.street_bet + player.stack;
        let others_can_respond = self
            .players
            .iter()
            .any(|p| p.seat != seat && p.status == PlayerStatus::Active);
        let raise_range = (!player.acted_this_street
            && others_can_respond
            && all_in_total > self.current_bet)
            .then(|| {
                let min_to = (self.current_bet + self.min_raise).min(all_in_total);
                (min_to, all_in_total)
            });

        Some(LegalActions {
            can_check: to_call == 0,
            call_amount: (to_call > 0).then(|| to_call.min(player.stack)),
            raise_range,
        })
    }

    /// Applies `action` for the player whose turn it is, then passes the turn on,
    /// dealing the next street when the betting round is complete.
    ///
    /// Illegal actions are rejected without changing the state.
    pub fn apply_action(&mut self, action: Action) -> Result<(), ActionError> {
        let seat = self.to_act.ok_or(ActionError::NoPlayerToAct)?;
        let legal = self.legal_actions().ok_or(ActionError::NoPlayerToAct)?;

        match action {
            Action::Fold => {
                self.players[seat].status = PlayerStatus::Folded;
            }
            Action::Check => {
                if !legal.can_check {
                    return Err(ActionError::CannotCheck);
                }
            }
            Action::Call => {
                let amount = legal.call_amount.ok_or(ActionError::NothingToCall)?;
                self.post(seat, amount);
            }
            Action::Raise { to } => {
                let (min_to, max_to) = legal.raise_range.ok_or(ActionError::RaiseNotAllowed)?;
                if to > max_to {
                    return Err(ActionError::RaiseTooLarge);
                }
                if to < min_to {
                    return Err(ActionError::RaiseTooSmall);
                }

                let raise_size = to - self.current_bet;
                if raise_size >= self.min_raise {
                    // A full raise reopens betting: everyone must respond again.
                    self.min_raise = raise_size;
                    for player in &mut self.players {
                        player.acted_this_street = false;
                    }
                }
                self.current_bet = to;
                self.last_aggressor = Some(seat);
                let already_in = self.players[seat].street_bet;
                self.post(seat, to - already_in);
            }
        }

        self.players[seat].acted_this_street = true;
        self.pass_turn(seat);
        Ok(())
    }

    /// Gives the turn to the next player after `from` who still needs to act.
    /// If nobody does, the betting round is over and the next street is dealt.
    pub(crate) fn pass_turn(&mut self, from: usize) {
        if self.players_in_hand() <= 1 {
            self.to_act = None;
            return;
        }

        let current_bet = self.current_bet;
        let active_count = self
            .players
            .iter()
            .filter(|p| p.status == PlayerStatus::Active)
            .count();
        let needs_to_act = |p: &crate::state::Player| {
            p.status == PlayerStatus::Active
                && (p.street_bet < current_bet || (!p.acted_this_street && active_count >= 2))
        };

        self.to_act = self.next_seat(from, needs_to_act);
        if self.to_act.is_none() {
            self.finish_street();
        }
    }

    /// Deals the following street. If fewer than two players can still bet,
    /// keeps dealing until the board is complete.
    fn finish_street(&mut self) {
        loop {
            self.deal_next_street();
            if self.street == Street::Showdown {
                self.to_act = None;
                return;
            }

            let active_count = self
                .players
                .iter()
                .filter(|p| p.status == PlayerStatus::Active)
                .count();
            if active_count >= 2 {
                // After the flop, the first active player left of the button opens.
                self.to_act = self.next_seat(self.button, |p| p.status == PlayerStatus::Active);
                return;
            }
        }
    }

    fn deal_next_street(&mut self) {
        for player in &mut self.players {
            player.street_bet = 0;
            player.acted_this_street = false;
        }
        self.current_bet = 0;
        self.min_raise = self.blinds.big;
        self.last_aggressor = None;

        let (next, cards) = match self.street {
            Street::Preflop => (Street::Flop, 3),
            Street::Flop => (Street::Turn, 1),
            Street::Turn => (Street::River, 1),
            Street::River | Street::Showdown => (Street::Showdown, 0),
        };
        if cards > 0 {
            // Burn one card before each street, as a real dealer does.
            self.deck.draw();
            for _ in 0..cards {
                let card = self.deck.draw().expect("deck has enough cards");
                self.board.push(card);
            }
        }
        self.street = next;
    }
}

#[cfg(test)]
mod tests {
    use super::{ActionError, LegalActions};
    use crate::state::{Action, Blinds, GameState, Player, PlayerStatus, Street};

    /// A table with blinds 5/10 and 1000-chip stacks, with the first hand started.
    ///
    /// The button moves to seat 1, so with 3+ players seat 2 is the small blind,
    /// seat 3 the big blind, and seat 4 is first to act.
    fn hand(count: usize) -> GameState {
        let players = (0..count)
            .map(|seat| Player::new(seat, format!("P{seat}"), 1000))
            .collect();
        let mut state = GameState::new(players, 0, Blinds { small: 5, big: 10, ante: 0 });
        state.start_hand(1);
        state
    }

    fn act(state: &mut GameState, action: Action) {
        state.apply_action(action).expect("legal action");
    }

    #[test]
    fn first_player_preflop_must_call_or_raise() {
        let state = hand(8);

        assert_eq!(
            state.legal_actions(),
            Some(LegalActions {
                can_check: false,
                call_amount: Some(10),
                raise_range: Some((20, 1000)),
            })
        );
    }

    #[test]
    fn fold_marks_player_and_passes_turn() {
        let mut state = hand(8);
        act(&mut state, Action::Fold);

        assert_eq!(state.players[4].status, PlayerStatus::Folded);
        assert_eq!(state.to_act, Some(5));
    }

    #[test]
    fn call_matches_current_bet_and_passes_turn() {
        let mut state = hand(8);
        act(&mut state, Action::Call);

        assert_eq!(state.players[4].stack, 990);
        assert_eq!(state.players[4].street_bet, 10);
        assert_eq!(state.to_act, Some(5));
    }

    #[test]
    fn check_facing_a_bet_is_rejected() {
        let mut state = hand(8);

        assert_eq!(state.apply_action(Action::Check), Err(ActionError::CannotCheck));
        assert_eq!(state.to_act, Some(4));
        assert_eq!(state.players[4].stack, 1000);
    }

    #[test]
    fn call_with_nothing_to_call_is_rejected() {
        let mut state = hand(2);
        act(&mut state, Action::Call); // small blind completes
        assert_eq!(state.apply_action(Action::Call), Err(ActionError::NothingToCall));
    }

    #[test]
    fn short_call_goes_all_in() {
        let mut state = hand(8);
        state.players[4].stack = 4;
        act(&mut state, Action::Call);

        assert_eq!(state.players[4].street_bet, 4);
        assert_eq!(state.players[4].status, PlayerStatus::AllIn);
        assert_eq!(state.to_act, Some(5));
    }

    #[test]
    fn big_blind_gets_option_then_flop_is_dealt() {
        let mut state = hand(8);
        for _ in 0..7 {
            act(&mut state, Action::Call);
        }

        assert_eq!(state.to_act, Some(3));
        assert!(state.legal_actions().unwrap().can_check);

        act(&mut state, Action::Check);

        assert_eq!(state.street, Street::Flop);
        assert_eq!(state.board.len(), 3);
        assert_eq!(state.deck.cards.len(), 52 - 16 - 4);
        assert_eq!(state.current_bet, 0);
        assert!(state.players.iter().all(|p| p.street_bet == 0));
        assert!(state.players.iter().all(|p| p.hand_contribution == 10));
        assert_eq!(state.to_act, Some(2));
    }

    #[test]
    fn raise_below_minimum_is_rejected() {
        let mut state = hand(8);

        assert_eq!(
            state.apply_action(Action::Raise { to: 15 }),
            Err(ActionError::RaiseTooSmall)
        );
        assert_eq!(
            state.apply_action(Action::Raise { to: 1001 }),
            Err(ActionError::RaiseTooLarge)
        );
        assert_eq!(state.to_act, Some(4));
    }

    #[test]
    fn raise_sets_new_bet_and_minimum() {
        let mut state = hand(8);
        act(&mut state, Action::Raise { to: 30 });

        assert_eq!(state.current_bet, 30);
        assert_eq!(state.min_raise, 20);
        assert_eq!(state.last_aggressor, Some(4));
        assert_eq!(state.players[4].stack, 970);
        assert_eq!(state.legal_actions().unwrap().raise_range, Some((50, 1000)));
    }

    #[test]
    fn raise_reopens_action_for_earlier_callers() {
        let mut state = hand(3);
        // Seat 1 (button) calls, seat 2 (small blind) raises.
        act(&mut state, Action::Call);
        act(&mut state, Action::Raise { to: 40 });
        act(&mut state, Action::Call); // big blind

        assert_eq!(state.to_act, Some(1));
        assert_eq!(state.legal_actions().unwrap().call_amount, Some(30));
        assert!(state.legal_actions().unwrap().raise_range.is_some());
    }

    #[test]
    fn incomplete_all_in_raise_does_not_reopen_betting() {
        let mut state = hand(3);
        // Seats: button 1, small blind 2, big blind 0.
        state.players[2].stack = 115; // 120 total including the posted blind

        act(&mut state, Action::Raise { to: 100 }); // button: full raise
        act(&mut state, Action::Raise { to: 120 }); // small blind: all-in, only 20 more
        act(&mut state, Action::Call); // big blind

        assert_eq!(state.to_act, Some(1));
        let legal = state.legal_actions().unwrap();
        assert_eq!(legal.call_amount, Some(20));
        assert_eq!(legal.raise_range, None);
        assert_eq!(
            state.apply_action(Action::Raise { to: 300 }),
            Err(ActionError::RaiseNotAllowed)
        );
    }

    #[test]
    fn everyone_folding_ends_the_hand() {
        let mut state = hand(8);
        for _ in 0..7 {
            act(&mut state, Action::Fold);
        }

        assert!(state.is_hand_over());
        assert_eq!(state.to_act, None);
        assert_eq!(state.apply_action(Action::Check), Err(ActionError::NoPlayerToAct));
    }

    #[test]
    fn heads_up_big_blind_acts_first_after_the_flop() {
        let mut state = hand(2);
        // Button (seat 1) is the small blind and acts first preflop.
        assert_eq!(state.to_act, Some(1));
        act(&mut state, Action::Call);
        act(&mut state, Action::Check);

        assert_eq!(state.street, Street::Flop);
        assert_eq!(state.to_act, Some(0));
    }

    #[test]
    fn checking_every_street_reaches_showdown() {
        let mut state = hand(2);
        act(&mut state, Action::Call);
        act(&mut state, Action::Check);
        for _ in 0..3 {
            act(&mut state, Action::Check);
            act(&mut state, Action::Check);
        }

        assert_eq!(state.street, Street::Showdown);
        assert_eq!(state.board.len(), 5);
        assert!(state.is_hand_over());
        assert_eq!(state.to_act, None);
    }

    #[test]
    fn all_in_and_call_runs_out_the_board() {
        let mut state = hand(2);
        act(&mut state, Action::Raise { to: 1000 });

        let legal = state.legal_actions().unwrap();
        assert_eq!(legal.call_amount, Some(990));
        assert_eq!(legal.raise_range, None);

        act(&mut state, Action::Call);

        assert_eq!(state.street, Street::Showdown);
        assert_eq!(state.board.len(), 5);
        assert_eq!(state.to_act, None);
        assert!(state.players.iter().all(|p| p.hand_contribution == 1000));
    }

    #[test]
    fn lone_active_player_is_not_asked_to_act_once_matched() {
        let mut state = hand(3);
        state.players[0].stack = 1990; // big blind has chips left after calling
        // Button and small blind go all-in; big blind calls, nobody is left to bet against.
        act(&mut state, Action::Raise { to: 1000 });
        act(&mut state, Action::Call);
        act(&mut state, Action::Call);

        assert_eq!(state.players[0].status, PlayerStatus::Active);
        assert_eq!(state.street, Street::Showdown);
        assert_eq!(state.board.len(), 5);
        assert_eq!(state.to_act, None);
    }

    #[test]
    fn blinds_putting_everyone_all_in_runs_out_the_board() {
        // Both stacks are smaller than the blinds they post.
        let players = vec![Player::new(0, "A", 4), Player::new(1, "B", 3)];
        let mut state = GameState::new(players, 0, Blinds { small: 5, big: 10, ante: 0 });
        state.start_hand(1);

        assert_eq!(state.street, Street::Showdown);
        assert_eq!(state.board.len(), 5);
        assert_eq!(state.to_act, None);
    }
}
