use serde::{Deserialize, Serialize};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::card::{Card, Rank, Suit};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::new();
        for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            for rank in [
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
                Rank::Ace,
            ] {
                cards.push(Card { rank, suit });
            }
        }
        Self { cards }
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn shuffle(&mut self, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        self.cards.shuffle(&mut rng);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::Deck;

    #[test]
    fn new_deck_has_52_unique_cards() {
        let deck = Deck::new();
        let unique_cards: HashSet<_> = deck.cards.iter().collect();

        assert_eq!(deck.cards.len(), 52);
        assert_eq!(unique_cards.len(), 52);
    }

    #[test]
    fn draw_returns_cards_until_deck_is_empty() {
        let mut deck = Deck::new();

        for _ in 0..52 {
            assert!(deck.draw().is_some());
        }

        assert!(deck.draw().is_none());
    }

    #[test]
    fn same_seed_produces_same_shuffle() {
        let mut first_deck = Deck::new();
        let mut second_deck = Deck::new();

        first_deck.shuffle(42);
        second_deck.shuffle(42);

        assert_eq!(first_deck.cards, second_deck.cards);
    }

    #[test]
    fn shuffle_preserves_all_52_unique_cards() {
        let mut deck = Deck::new();
        deck.shuffle(42);
        let unique_cards: HashSet<_> = deck.cards.iter().collect();

        assert_eq!(deck.cards.len(), 52);
        assert_eq!(unique_cards.len(), 52);
    }
}