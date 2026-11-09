use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Rank {
    pub fn get_rank(self) -> u8 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 12,
            Rank::King => 13,
            Rank::Ace => 14,
        }
    }
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }
}

impl Suit {
    pub fn get_suit(self) -> u8 {
        match self {
            Suit::Clubs => 0,
            Suit::Diamonds => 1,
            Suit::Hearts => 2,
            Suit::Spades => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseCardError;

impl FromStr for Card {
    type Err = ParseCardError;

    /// Parses short notation like "As" (ace of spades) or "Td" (ten of diamonds).
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut chars = text.chars();
        let rank = match chars.next().ok_or(ParseCardError)? {
            '2' => Rank::Two,
            '3' => Rank::Three,
            '4' => Rank::Four,
            '5' => Rank::Five,
            '6' => Rank::Six,
            '7' => Rank::Seven,
            '8' => Rank::Eight,
            '9' => Rank::Nine,
            'T' => Rank::Ten,
            'J' => Rank::Jack,
            'Q' => Rank::Queen,
            'K' => Rank::King,
            'A' => Rank::Ace,
            _ => return Err(ParseCardError),
        };
        let suit = match chars.next().ok_or(ParseCardError)? {
            'c' => Suit::Clubs,
            'd' => Suit::Diamonds,
            'h' => Suit::Hearts,
            's' => Suit::Spades,
            _ => return Err(ParseCardError),
        };
        if chars.next().is_some() {
            return Err(ParseCardError);
        }
        Ok(Card::new(rank, suit))
    }
}

#[cfg(test)]
mod tests {
    use super::{Card, ParseCardError, Rank, Suit};

    #[test]
    fn parses_short_card_notation() {
        assert_eq!("As".parse(), Ok(Card::new(Rank::Ace, Suit::Spades)));
        assert_eq!("Td".parse(), Ok(Card::new(Rank::Ten, Suit::Diamonds)));
        assert_eq!("2c".parse(), Ok(Card::new(Rank::Two, Suit::Clubs)));
    }

    #[test]
    fn rejects_malformed_card_notation() {
        for text in ["", "A", "1s", "Ax", "as", "Ass"] {
            assert_eq!(text.parse::<Card>(), Err(ParseCardError), "{text:?}");
        }
    }

    #[test]
    fn rank_values_are_in_ascending_order() {
        let ranks = [
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
        ];

        for (index, rank) in ranks.iter().enumerate() {
            assert_eq!(rank.get_rank(), index as u8 + 2);
        }
    }

    #[test]
    fn card_constructor_sets_rank_and_suit() {
        let card = Card::new(Rank::Ace, Suit::Spades);

        assert_eq!(card.rank, Rank::Ace);
        assert_eq!(card.suit, Suit::Spades);
    }

    #[test]
    fn suit_values_are_distinct_and_stable() {
        assert_eq!(Suit::Clubs.get_suit(), 0);
        assert_eq!(Suit::Diamonds.get_suit(), 1);
        assert_eq!(Suit::Hearts.get_suit(), 2);
        assert_eq!(Suit::Spades.get_suit(), 3);
    }
}