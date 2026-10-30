//! Seven-card hand evaluation.

use serde::{Deserialize, Serialize};

use crate::card::Card;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum HandCategory {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

/// Comparable strength of a five-card hand.
///
/// `kickers` holds rank values (2..=14) in order of significance, padded with
/// zeros: e.g. a full house is `[trips, pair, 0, 0, 0]` and a straight is
/// `[high card, 0, 0, 0, 0]`. The derived `Ord` compares category first, then
/// kickers left to right.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct HandValue {
    pub category: HandCategory,
    pub kickers: [u8; 5],
}

/// Returns the best five-card hand that can be made from seven cards.
pub fn evaluate(cards: [Card; 7]) -> HandValue {
    let mut best: Option<HandValue> = None;

    for a in 0..3 {
        for b in a + 1..4 {
            for c in b + 1..5 {
                for d in c + 1..6 {
                    for e in d + 1..7 {
                        let five_cards = [cards[a], cards[b], cards[c], cards[d], cards[e]];
                        best = best.max(Some(classify_five(five_cards)));
                    }
                }
            }
        }
    }

    best.expect("seven cards always contain a five-card hand")
}

fn classify_five(cards: [Card; 5]) -> HandValue {
    let mut counts = [0u8; 15];
    for card in cards {
        counts[card.rank.get_rank() as usize] += 1;
    }

    // Distinct ranks as (count, rank), sorted so the biggest group comes first
    // and ties within a group size go to the higher rank.
    let mut groups = [(0u8, 0u8); 5];
    let mut len = 0;
    for rank in 2..=14u8 {
        let count = counts[rank as usize];
        if count > 0 {
            groups[len] = (count, rank);
            len += 1;
        }
    }
    let groups = &mut groups[..len];
    groups.sort_unstable_by(|x, y| y.cmp(x));

    let mut kickers = [0u8; 5];
    for (slot, &(_, rank)) in kickers.iter_mut().zip(groups.iter()) {
        *slot = rank;
    }

    let is_flush = cards.iter().all(|card| card.suit == cards[0].suit);

    let straight_high = if len < 5 {
        None
    } else if kickers[0] - kickers[4] == 4 {
        Some(kickers[0])
    } else if kickers == [14, 5, 4, 3, 2] {
        // The wheel: the ace plays low, so the straight is five-high.
        Some(5)
    } else {
        None
    };

    let (top, second) = (groups[0].0, groups[1].0);

    let category = if straight_high.is_some() && is_flush {
        HandCategory::StraightFlush
    } else if top == 4 {
        HandCategory::FourOfAKind
    } else if top == 3 && second == 2 {
        HandCategory::FullHouse
    } else if is_flush {
        HandCategory::Flush
    } else if straight_high.is_some() {
        HandCategory::Straight
    } else if top == 3 {
        HandCategory::ThreeOfAKind
    } else if top == 2 && second == 2 {
        HandCategory::TwoPair
    } else if top == 2 {
        HandCategory::OnePair
    } else {
        HandCategory::HighCard
    };

    if let Some(high) = straight_high {
        kickers = [high, 0, 0, 0, 0];
    }

    HandValue { category, kickers }
}

#[cfg(test)]
mod tests {
    use super::{classify_five, evaluate, HandCategory, HandValue};
    use crate::card::{Card, Rank, Suit};

    /// Parses short notation like "As" or "Td" into a card.
    fn card(text: &str) -> Card {
        let mut chars = text.chars();
        let rank = match chars.next().unwrap() {
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
            other => panic!("unknown rank {other}"),
        };
        let suit = match chars.next().unwrap() {
            'c' => Suit::Clubs,
            'd' => Suit::Diamonds,
            'h' => Suit::Hearts,
            's' => Suit::Spades,
            other => panic!("unknown suit {other}"),
        };
        Card::new(rank, suit)
    }

    fn five(text: &str) -> HandValue {
        let cards: Vec<Card> = text.split_whitespace().map(card).collect();
        classify_five(cards.try_into().unwrap())
    }

    fn seven(text: &str) -> HandValue {
        let cards: Vec<Card> = text.split_whitespace().map(card).collect();
        evaluate(cards.try_into().unwrap())
    }

    #[test]
    fn stronger_category_beats_weaker_category() {
        let pair = HandValue {
            category: HandCategory::OnePair,
            kickers: [14, 12, 10, 8, 0],
        };
        let straight = HandValue {
            category: HandCategory::Straight,
            kickers: [6, 0, 0, 0, 0],
        };

        assert!(straight > pair);
    }

    #[test]
    fn higher_kickers_win_within_same_category() {
        let ace_kicker = HandValue {
            category: HandCategory::OnePair,
            kickers: [13, 14, 10, 8, 0],
        };
        let queen_kicker = HandValue {
            category: HandCategory::OnePair,
            kickers: [13, 12, 10, 8, 0],
        };

        assert!(ace_kicker > queen_kicker);
    }

    #[test]
    fn identical_category_and_kickers_are_equal() {
        let first = HandValue {
            category: HandCategory::Flush,
            kickers: [14, 11, 9, 6, 2],
        };
        let second = first;

        assert_eq!(first, second);
    }

    #[test]
    fn classifies_every_category() {
        let cases = [
            ("Ah Kd 9c 7s 3h", HandCategory::HighCard, [14, 13, 9, 7, 3]),
            ("Kh Kd 9c 7s 3h", HandCategory::OnePair, [13, 9, 7, 3, 0]),
            ("Kh Kd 3c 3s 9h", HandCategory::TwoPair, [13, 3, 9, 0, 0]),
            ("7h 7d 7c As 2h", HandCategory::ThreeOfAKind, [7, 14, 2, 0, 0]),
            ("9h 8d 7c 6s 5h", HandCategory::Straight, [9, 0, 0, 0, 0]),
            ("Ah Jh 9h 6h 2h", HandCategory::Flush, [14, 11, 9, 6, 2]),
            ("3h 3d 3c Ks Kh", HandCategory::FullHouse, [3, 13, 0, 0, 0]),
            ("Qh Qd Qc Qs 4h", HandCategory::FourOfAKind, [12, 4, 0, 0, 0]),
            ("Th Jh Qh Kh Ah", HandCategory::StraightFlush, [14, 0, 0, 0, 0]),
        ];

        for (hand, category, kickers) in cases {
            assert_eq!(five(hand), HandValue { category, kickers }, "{hand}");
        }
    }

    #[test]
    fn wheel_is_five_high_straight() {
        let wheel = five("Ah 2d 3c 4s 5h");
        assert_eq!(wheel.category, HandCategory::Straight);
        assert_eq!(wheel.kickers, [5, 0, 0, 0, 0]);

        assert!(five("2h 3d 4c 5s 6h") > wheel);
    }

    #[test]
    fn steel_wheel_is_straight_flush() {
        let hand = five("Ah 2h 3h 4h 5h");
        assert_eq!(hand.category, HandCategory::StraightFlush);
        assert_eq!(hand.kickers, [5, 0, 0, 0, 0]);
    }

    #[test]
    fn wraparound_is_not_a_straight() {
        assert_eq!(five("Qh Kd Ac 2s 3h").category, HandCategory::HighCard);
    }

    #[test]
    fn evaluate_picks_best_five_of_seven() {
        // Five through nine makes a straight, but the five hearts make a flush.
        let hand = seven("2h 7h 5h 6c 8s 9h Qh");
        assert_eq!(hand.category, HandCategory::Flush);
        assert_eq!(hand.kickers, [12, 9, 7, 5, 2]);
    }

    #[test]
    fn evaluate_uses_board_pair_for_full_house() {
        let hand = seven("9s 9d 9c Kh Kd 2s 4c");
        assert_eq!(
            hand,
            HandValue {
                category: HandCategory::FullHouse,
                kickers: [9, 13, 0, 0, 0],
            }
        );
    }

    #[test]
    fn evaluate_picks_higher_full_house_from_two_trips() {
        let hand = seven("9s 9d 9c Kh Kd Ks 4c");
        assert_eq!(hand.category, HandCategory::FullHouse);
        assert_eq!(hand.kickers, [13, 9, 0, 0, 0]);
    }

    #[test]
    fn kicker_decides_two_pair_on_paired_board() {
        // Board: Jh Jd 4c 4s 2h. Both players play two pair, jacks and fours.
        let ace_kicker = seven("Ac 8d Jh Jd 4c 4s 2h");
        let king_kicker = seven("Kc 8s Jh Jd 4c 4s 2h");

        assert_eq!(ace_kicker.category, HandCategory::TwoPair);
        assert_eq!(ace_kicker.kickers, [11, 4, 14, 0, 0]);
        assert!(ace_kicker > king_kicker);
    }

    #[test]
    fn third_pair_does_not_count_beyond_kicker() {
        // Three pairs: only the top two count, the best remaining card is the kicker.
        let hand = seven("Ah Ad Kc Ks 3h 3d 2c");
        assert_eq!(hand.kickers, [14, 13, 3, 0, 0]);
    }

    #[test]
    fn split_pot_when_board_plays() {
        let first = seven("2c 3d Ah Kh Qh Jh Th");
        let second = seven("4c 5d Ah Kh Qh Jh Th");
        assert_eq!(first, second);
    }
}
