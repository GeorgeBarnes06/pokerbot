// Rebuilds the table step by step from a hand log, for the replay view.

import { categoryLabel, chips } from "@/lib/format";
import type { Card, HandLog, Street } from "@/types/log";

export interface SeatFrame {
  seat: number;
  name: string;
  holeCards: [Card, Card] | null;
  stack: number;
  /** Chips in front of the player on the current street. */
  bet: number;
  folded: boolean;
  allIn: boolean;
  /** Short label for the player's latest action, e.g. "Call" or "Raise 60". */
  lastAction: string | null;
  isButton: boolean;
  isWinner: boolean;
  /** Hand shown at showdown, e.g. "Full house". */
  shownHand: string | null;
  /** Action codes per betting street (preflop, flop, turn, river), e.g. "bC". */
  strip: [string, string, string, string];
}

export interface Frame {
  street: Street;
  board: Card[];
  pot: number;
  seats: SeatFrame[];
  /** Seat whose action produced this frame, highlighted on the table. */
  actor: number | null;
  description: string;
  /** Index into `hand.actions` this frame is after, if it is an action frame. */
  actionIndex: number | null;
}

const BOARD_SIZE: Record<Street, number> = {
  preflop: 0,
  flop: 3,
  turn: 4,
  river: 5,
  showdown: 5,
};

const STREET_NAMES: Record<Street, string> = {
  preflop: "Preflop",
  flop: "Flop",
  turn: "Turn",
  river: "River",
  showdown: "Showdown",
};

export const streetName = (street: Street) => STREET_NAMES[street];

/** One-letter action codes used in the action strips. */
export const ACTION_CODES = [
  ["b", "BLIND"],
  ["X", "CHECK"],
  ["C", "CALL"],
  ["B", "BET"],
  ["R", "RAISE"],
  ["A", "ALL-IN"],
  ["F", "FOLD"],
  ["W", "WIN"],
] as const;

const STRIP_INDEX: Record<Street, number> = {
  preflop: 0,
  flop: 1,
  turn: 2,
  river: 3,
  showdown: 3,
};

export function buildFrames(hand: HandLog): Frame[] {
  const nameOf = new Map(hand.seats.map((s) => [s.seat, s.name]));
  const seats: SeatFrame[] = hand.seats.map((start) => {
    const ante = Math.min(hand.blinds.ante, start.stack);
    const stack = start.stack - start.posted;
    return {
      seat: start.seat,
      name: start.name,
      holeCards: start.hole_cards,
      stack,
      bet: start.posted - ante,
      folded: false,
      allIn: stack === 0,
      lastAction: null,
      isButton: start.seat === hand.button,
      isWinner: false,
      shownHand: null,
      strip: [start.posted - ante > 0 ? "b" : "", "", "", ""],
    };
  });
  const bySeat = new Map(seats.map((s) => [s.seat, s]));

  let street: Street = "preflop";
  let pot = hand.seats.reduce((sum, s) => sum + s.posted, 0);
  let currentBet = Math.max(0, ...seats.map((s) => s.bet));

  const frames: Frame[] = [];
  const snapshot = (
    description: string,
    actor: number | null,
    actionIndex: number | null,
    board = hand.board.slice(0, BOARD_SIZE[street]),
  ) => {
    frames.push({
      street,
      board,
      pot,
      seats: seats.map((s) => ({ ...s, strip: [...s.strip] as SeatFrame["strip"] })),
      actor,
      description,
      actionIndex,
    });
  };

  snapshot(`Blinds ${chips(hand.blinds.small)}/${chips(hand.blinds.big)} posted`, null, null);

  const dealStreet = (next: Street) => {
    street = next;
    currentBet = 0;
    for (const seat of seats) {
      seat.bet = 0;
      seat.lastAction = null;
    }
    snapshot(`${streetName(next)} dealt`, null, null);
  };

  hand.actions.forEach((record, index) => {
    if (record.street !== street) dealStreet(record.street);

    const seat = bySeat.get(record.seat);
    if (!seat) return;
    seat.stack -= record.amount;
    seat.bet += record.amount;
    pot += record.amount;

    let label: string;
    let code: string;
    const action = record.action;
    if (action === "fold") {
      seat.folded = true;
      label = "Fold";
      code = "F";
    } else if (action === "check") {
      label = "Check";
      code = "X";
    } else if (action === "call") {
      label = `Call ${chips(record.amount)}`;
      code = "C";
    } else {
      const isBet = currentBet === 0;
      label = `${isBet ? "Bet" : "Raise to"} ${chips(action.raise.to)}`;
      code = isBet ? "B" : "R";
      currentBet = Math.max(currentBet, action.raise.to);
    }
    if (seat.stack === 0 && record.amount > 0) {
      seat.allIn = true;
      label += " (all-in)";
      code = "A";
    }
    seat.lastAction = label;
    seat.strip[STRIP_INDEX[street]] += code;

    const rejected = record.rejected ? " (illegal move replaced)" : "";
    snapshot(`${seat.name}: ${label}${rejected}`, seat.seat, index);
  });

  // Showdown and payout.
  const result = hand.result;
  if (result.shown.length > 0) {
    if (hand.board.length > BOARD_SIZE[street]) {
      for (const next of ["flop", "turn", "river"] as const) {
        if (BOARD_SIZE[next] > BOARD_SIZE[street]) {
          street = next;
          snapshot(`${streetName(next)} dealt (all-in, no more betting)`, null, null);
        }
      }
    }
    street = "showdown";
    for (const shown of result.shown) {
      const seat = bySeat.get(shown.seat);
      if (seat) seat.shownHand = categoryLabel(shown.value.category);
    }
  }

  for (const seat of seats) {
    seat.stack = hand.final_stacks[seat.seat] ?? seat.stack;
    seat.bet = 0;
    seat.isWinner = result.awards.some((award) => award.winners.includes(seat.seat));
    if (seat.isWinner) seat.strip[STRIP_INDEX[street]] += "W";
  }
  const potTotal = result.awards.reduce((sum, award) => sum + award.amount, 0);
  pot = 0;

  const parts = result.awards.map((award, i) => {
    const names = award.winners.map((seat) => nameOf.get(seat) ?? `Seat ${seat}`).join(" & ");
    const potName = result.awards.length > 1 ? (i === 0 ? "main pot" : `side pot ${i}`) : "the pot";
    const verb = award.winners.length > 1 ? "split" : "wins";
    return `${names} ${verb} ${potName} (${chips(award.amount)})`;
  });
  if (result.uncalled_bet) {
    const name = nameOf.get(result.uncalled_bet.seat);
    parts.push(`${chips(result.uncalled_bet.amount)} uncalled returned to ${name}`);
  }
  snapshot(
    parts.join("; ") || `Pot of ${chips(potTotal)} awarded`,
    null,
    null,
    result.shown.length > 0 ? hand.board : hand.board.slice(0, BOARD_SIZE[street]),
  );

  return frames;
}
