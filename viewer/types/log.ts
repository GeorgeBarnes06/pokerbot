// Mirrors the JSON the Rust runner writes to logs/<match>/ (see crates/engine/src/log.rs
// and crates/runner/src/tournament.rs). Keep in sync when those structs change.

export type Rank =
  | "two"
  | "three"
  | "four"
  | "five"
  | "six"
  | "seven"
  | "eight"
  | "nine"
  | "ten"
  | "jack"
  | "queen"
  | "king"
  | "ace";

export type Suit = "clubs" | "diamonds" | "hearts" | "spades";

export interface Card {
  rank: Rank;
  suit: Suit;
}

export type Street = "preflop" | "flop" | "turn" | "river" | "showdown";

/** Unit variants serialize as strings; `Raise { to }` as `{ raise: { to } }`. */
export type Action = "fold" | "check" | "call" | { raise: { to: number } };

export interface Blinds {
  small: number;
  big: number;
  ante: number;
}

export interface ActionRecord {
  street: Street;
  seat: number;
  action: Action;
  /** Chips this action put into the pot. */
  amount: number;
  /** What the bot asked for, when it was illegal and replaced. */
  rejected?: Action;
}

export interface SeatStart {
  seat: number;
  name: string;
  /** Stack before antes and blinds. */
  stack: number;
  hole_cards: [Card, Card] | null;
  /** Antes and blinds posted before any action. */
  posted: number;
}

export type HandCategory =
  | "high_card"
  | "one_pair"
  | "two_pair"
  | "three_of_a_kind"
  | "straight"
  | "flush"
  | "full_house"
  | "four_of_a_kind"
  | "straight_flush";

export interface HandValue {
  category: HandCategory;
  kickers: number[];
}

export interface ShownHand {
  seat: number;
  value: HandValue;
}

export interface PotAward {
  amount: number;
  winners: number[];
}

export interface UncalledBet {
  seat: number;
  amount: number;
}

export interface HandResult {
  uncalled_bet: UncalledBet | null;
  shown: ShownHand[];
  awards: PotAward[];
}

export interface HandLog {
  hand_number: number;
  seed: number;
  button: number;
  blinds: Blinds;
  seats: SeatStart[];
  actions: ActionRecord[];
  board: Card[];
  result: HandResult;
  /** Every seat's stack after the payout, indexed by seat. */
  final_stacks: number[];
}

export interface PlayerConfig {
  name: string;
  bot: string;
}

export interface TournamentConfig {
  seed: number;
  starting_stack: number;
  hands_per_level: number;
  blind_levels: Blinds[];
  max_hands: number;
  /** Cash format: short stacks are topped back up and nobody is eliminated. */
  rebuy: boolean;
  players: PlayerConfig[];
}

export interface Standing {
  place: number;
  seat: number;
  name: string;
  bot: string;
  final_stack: number;
  rebuys: number;
  /** Final stack minus the starting stack and every top-up. */
  net_chips: number;
  bb_per_100: number;
  busted_on_hand: number | null;
}

/** One line per hand in `summary.json`. */
export interface HandSummary {
  hand_number: number;
  blinds: Blinds;
  pot: number;
  winners: number[];
  showdown: boolean;
}

/** `logs/<match>/summary.json`; each hand is in `hands/<n>.json`. */
export interface MatchSummary {
  config: TournamentConfig;
  finished: boolean;
  standings: Standing[];
  hands: HandSummary[];
}
