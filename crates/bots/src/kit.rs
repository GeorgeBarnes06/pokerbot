//! Helpers any model can use. They read the table and build legal actions;
//! they don't decide anything.

use engine::card::{Card, Rank};
use engine::log::ActionRecord;
use engine::state::{Action, Chips, PlayerStatus};
use engine::view::PlayerView;

/// Check if free, otherwise call.
pub fn check_or_call(view: &PlayerView) -> Action {
    if view.legal.can_check {
        Action::Check
    } else {
        Action::Call
    }
}

/// Check if free, otherwise fold.
pub fn check_or_fold(view: &PlayerView) -> Action {
    if view.legal.can_check {
        Action::Check
    } else {
        Action::Fold
    }
}

/// Bet or raise to a total of `to` chips this street, clamped into the legal
/// range. If raising isn't allowed, checks or calls instead.
pub fn raise_to(view: &PlayerView, to: Chips) -> Action {
    match view.legal.raise_range {
        Some((min, max)) => Action::Raise { to: to.clamp(min, max) },
        None => check_or_call(view),
    }
}

/// Bet or raise by `fraction` of the pot after calling, e.g. 0.5 for half pot,
/// 1.0 for pot. Clamped into the legal range like [`raise_to`].
pub fn raise_pot_fraction(view: &PlayerView, fraction: f64) -> Action {
    let pot_after_call = view.pot + view.to_call;
    let size = (pot_after_call as f64 * fraction).round() as Chips;
    raise_to(view, view.current_bet + size)
}

/// Go all-in: raise to the maximum, or call if raising isn't allowed.
pub fn all_in(view: &PlayerView) -> Action {
    match view.legal.raise_range {
        Some((_, max)) => Action::Raise { to: max },
        None => check_or_call(view),
    }
}

/// Share of the final pot a call costs: call if your equity beats this.
/// Zero when there is nothing to call.
pub fn pot_odds(view: &PlayerView) -> f64 {
    if view.to_call == 0 {
        return 0.0;
    }
    view.to_call as f64 / (view.pot + view.to_call) as f64
}

/// This player's stack, in big blinds.
pub fn stack_in_big_blinds(view: &PlayerView) -> f64 {
    view.seats[view.seat].stack as f64 / view.blinds.big as f64
}

/// Players still contesting the pot, including this one.
pub fn players_in_hand(view: &PlayerView) -> usize {
    view.seats
        .iter()
        .filter(|s| matches!(s.status, PlayerStatus::Active | PlayerStatus::AllIn))
        .count()
}

/// Seats clockwise from the button to this player, counting only players dealt
/// in: 0 is the button, 1 the small blind, 2 the big blind (heads-up, the
/// button is the small blind and the other player is 1).
pub fn seats_after_button(view: &PlayerView) -> usize {
    let n = view.seats.len();
    (0..n)
        .map(|offset| (view.button + offset) % n)
        .filter(|&seat| view.seats[seat].status != PlayerStatus::Busted)
        .position(|seat| seat == view.seat)
        .unwrap_or(0)
}

/// Actions already taken on the current street.
pub fn actions_this_street(view: &PlayerView) -> impl Iterator<Item = &ActionRecord> {
    view.history.iter().filter(move |a| a.street == view.street)
}

/// Bets and raises so far on the current street (blinds don't count).
pub fn raises_this_street(view: &PlayerView) -> usize {
    actions_this_street(view)
        .filter(|a| matches!(a.action, Action::Raise { .. }))
        .count()
}

/// Standard short name for a starting hand: "AA", "AKs" (suited), "T9o" (offsuit).
pub fn preflop_key(cards: [Card; 2]) -> String {
    let [a, b] = if cards[0].rank >= cards[1].rank {
        cards
    } else {
        [cards[1], cards[0]]
    };
    let (high, low) = (rank_char(a.rank), rank_char(b.rank));
    if a.rank == b.rank {
        format!("{high}{low}")
    } else if a.suit == b.suit {
        format!("{high}{low}s")
    } else {
        format!("{high}{low}o")
    }
}

fn rank_char(rank: Rank) -> char {
    b"23456789TJQKA"[rank.get_rank() as usize - 2] as char
}

#[cfg(test)]
mod tests {
    use engine::state::{Action, Blinds, GameState, Player};
    use engine::view::PlayerView;

    use super::*;

    fn first_view(players: usize) -> PlayerView {
        let players = (0..players)
            .map(|seat| Player::new(seat, format!("P{seat}"), 1000))
            .collect();
        let mut state = GameState::new(players, 0, Blinds { small: 5, big: 10, ante: 0 });
        state.start_hand(1);
        state.current_view(&[]).unwrap()
    }

    fn cards(text: &str) -> [Card; 2] {
        let c: Vec<Card> = text.split_whitespace().map(|c| c.parse().unwrap()).collect();
        [c[0], c[1]]
    }

    #[test]
    fn preflop_keys_use_standard_notation() {
        assert_eq!(preflop_key(cards("Ah Ad")), "AA");
        assert_eq!(preflop_key(cards("Kd As")), "AKo");
        assert_eq!(preflop_key(cards("9s Ts")), "T9s");
        assert_eq!(preflop_key(cards("2c 7h")), "72o");
    }

    #[test]
    fn raise_helpers_stay_legal() {
        let view = first_view(4);
        assert_eq!(raise_to(&view, 1), Action::Raise { to: 20 });
        assert_eq!(raise_to(&view, 50_000), Action::Raise { to: 1000 });
        assert_eq!(all_in(&view), Action::Raise { to: 1000 });
        // Pot 15 + 10 to call = 25; a pot-sized raise goes to 10 + 25.
        assert_eq!(raise_pot_fraction(&view, 1.0), Action::Raise { to: 35 });
    }

    #[test]
    fn reads_the_spot() {
        let view = first_view(4);
        // Seat 0 acts first: button is seat 1, blinds 2 and 3.
        assert_eq!(seats_after_button(&view), 3);
        assert_eq!(players_in_hand(&view), 4);
        assert!((pot_odds(&view) - 10.0 / 25.0).abs() < 1e-9);
        assert_eq!(stack_in_big_blinds(&view), 100.0);
        assert_eq!(check_or_fold(&view), Action::Fold);
        assert_eq!(check_or_call(&view), Action::Call);
    }
}
