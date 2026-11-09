// Display helpers for cards, hands, and actions.

import type { Action, Card, HandCategory, Rank, Suit } from "@/types/log";

const RANK_LABELS: Record<Rank, string> = {
  two: "2",
  three: "3",
  four: "4",
  five: "5",
  six: "6",
  seven: "7",
  eight: "8",
  nine: "9",
  ten: "10",
  jack: "J",
  queen: "Q",
  king: "K",
  ace: "A",
};

const SUIT_SYMBOLS: Record<Suit, string> = {
  clubs: "♣",
  diamonds: "♦",
  hearts: "♥",
  spades: "♠",
};

const CATEGORY_LABELS: Record<HandCategory, string> = {
  high_card: "High card",
  one_pair: "One pair",
  two_pair: "Two pair",
  three_of_a_kind: "Three of a kind",
  straight: "Straight",
  flush: "Flush",
  full_house: "Full house",
  four_of_a_kind: "Four of a kind",
  straight_flush: "Straight flush",
};

export const rankLabel = (rank: Rank) => RANK_LABELS[rank];
export const suitSymbol = (suit: Suit) => SUIT_SYMBOLS[suit];
export const isRedSuit = (suit: Suit) => suit === "hearts" || suit === "diamonds";
export const categoryLabel = (category: HandCategory) => CATEGORY_LABELS[category];

export const cardText = (card: Card) => `${rankLabel(card.rank)}${suitSymbol(card.suit)}`;

export function actionText(action: Action): string {
  if (typeof action === "string") {
    return { fold: "folds", check: "checks", call: "calls" }[action];
  }
  return `raises to ${chips(action.raise.to)}`;
}

export const chips = (amount: number) => amount.toLocaleString("en-US");
