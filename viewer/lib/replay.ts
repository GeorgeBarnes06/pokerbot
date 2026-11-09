// Rebuilds the table step by step from a hand log, for the live table and replays.

import { cardText, categoryLabel, chips } from "@/lib/format";
import type { Card, HandLog, Street } from "@/types/log";

export type FrameType = "start" | "blind" | "deal" | "act" | "street" | "refund" | "show";

export interface SeatFrame {
  seat: number;
  name: string;
  /** "S1 CFR-7": seat number (1-based) and name. */
  label: string;
  /** Null until the cards are dealt. */
  holeCards: [Card, Card] | null;
  stack: number;
  folded: boolean;
  allIn: boolean;
  /** Share of the pot this hand wins on average, 0–1, every hand known.
   *  Null if folded, not dealt yet, or not recorded for this match. */
  equity: number | null;
  /** The seat's latest action, e.g. "CALL 100", "FOLD", "+1,200". */
  act: string;
  isButton: boolean;
  isWinner: boolean;
}

export interface Frame {
  type: FrameType;
  street: Street;
  board: Card[];
  pot: number;
  seats: SeatFrame[];
  /** Seat that acted in this step, highlighted on the table. */
  actor: number | null;
  /** One-letter code for the actor's action strip (b X C B R A F). */
  code: string | null;
  /** Street name for the timeline header, on the step a street begins. */
  tag: string | null;
  /** Seats that won chips at the end. */
  winners: number[];
  description: string;
  /** How long the actor took to decide, in microseconds. */
  thinkUs: number | null;
  /** The actor's own notes on this action. */
  notes: string[];
}

/** Pause after each kind of step, in units of the speed setting (from the reference site). */
export const WAIT: Record<FrameType, number> = {
  start: 2,
  blind: 1,
  deal: 2,
  act: 1,
  street: 3,
  refund: 2,
  show: 10,
};

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

const BOARD_SIZE: Record<Street, number> = {
  preflop: 0,
  flop: 3,
  turn: 4,
  river: 5,
  showdown: 5,
};

export const streetName = (street: Street) => street.toUpperCase();

export function buildFrames(hand: HandLog): Frame[] {
  const equityAt = new Map(
    (hand.equity ?? []).map((e) => [e.street, new Map(e.seats.map((s) => [s.seat, s.equity]))]),
  );

  const seats: SeatFrame[] = hand.seats.map((start) => ({
    seat: start.seat,
    name: start.name,
    label: `S${start.seat + 1} ${start.name}`,
    holeCards: null,
    stack: start.stack,
    folded: false,
    allIn: false,
    equity: null,
    act: "",
    isButton: start.seat === hand.button,
    isWinner: false,
  }));
  const bySeat = new Map(seats.map((s) => [s.seat, s]));
  const nameOf = (seat: number) => bySeat.get(seat)?.name ?? `S${seat + 1}`;

  let street: Street = "preflop";
  let boardShown = 0;
  let pot = 0;
  let currentBet = 0;
  const frames: Frame[] = [];

  const setEquity = () => {
    const shares = equityAt.get(street);
    for (const seat of seats) {
      seat.equity = !seat.folded && seat.holeCards && shares ? (shares.get(seat.seat) ?? null) : null;
    }
  };

  const push = (type: FrameType, description: string, extra: Partial<Frame> = {}) => {
    frames.push({
      type,
      street,
      board: hand.board.slice(0, boardShown),
      pot,
      seats: seats.map((s) => ({ ...s })),
      actor: null,
      code: null,
      tag: null,
      winners: [],
      description,
      thinkUs: null,
      notes: [],
      ...extra,
    });
  };

  push("start", `HAND ${String(hand.hand_number).padStart(6, "0")}`);

  // Antes go in silently; blinds are steps of their own, small blind first.
  const antes = new Map(hand.seats.map((s) => [s.seat, Math.min(hand.blinds.ante, s.stack)]));
  for (const seat of seats) {
    const ante = antes.get(seat.seat) ?? 0;
    seat.stack -= ante;
    pot += ante;
  }
  const blinds = hand.seats
    .map((s) => ({ seat: s.seat, amount: s.posted - (antes.get(s.seat) ?? 0) }))
    .filter((b) => b.amount > 0)
    .sort((a, b) => a.amount - b.amount);
  blinds.forEach((blind, i) => {
    const seat = bySeat.get(blind.seat)!;
    const kind = blinds.length > 1 && i === 0 ? "SB" : "BB";
    seat.stack -= blind.amount;
    seat.allIn = seat.stack === 0;
    seat.act = `${kind} ${chips(blind.amount)}`;
    pot += blind.amount;
    currentBet = Math.max(currentBet, blind.amount);
    push("blind", `${seat.name} POSTS ${kind} ${chips(blind.amount)}`, { actor: seat.seat, code: "b" });
  });

  for (const start of hand.seats) {
    bySeat.get(start.seat)!.holeCards = start.hole_cards;
  }
  setEquity();
  push("deal", "CARDS DEALT", { tag: "PRE" });

  const dealStreet = (next: Street, note = "") => {
    street = next;
    boardShown = BOARD_SIZE[next];
    currentBet = 0;
    for (const seat of seats) seat.act = seat.folded ? "FOLD" : "";
    setEquity();
    const cards = hand.board.slice(0, boardShown).map(cardText).join(" ");
    push("street", `${streetName(next)}: ${cards}${note}`, { tag: streetName(next) });
  };

  hand.actions.forEach((record) => {
    if (record.street !== street) dealStreet(record.street);

    const seat = bySeat.get(record.seat);
    if (!seat) return;
    seat.stack -= record.amount;
    pot += record.amount;

    const action = record.action;
    let act: string;
    let code: string;
    let said: string;
    if (action === "fold") {
      seat.folded = true;
      seat.equity = null;
      [act, code, said] = ["FOLD", "F", "FOLDS"];
    } else if (action === "check") {
      [act, code, said] = ["CHECK", "X", "CHECKS"];
    } else if (action === "call") {
      [act, code, said] = [`CALL ${chips(record.amount)}`, "C", `CALLS ${chips(record.amount)}`];
    } else {
      const to = chips(action.raise.to);
      [act, code, said] =
        currentBet === 0 ? [`BET ${to}`, "B", `BETS ${to}`] : [`RAISE TO ${to}`, "R", `RAISES TO ${to}`];
      currentBet = Math.max(currentBet, action.raise.to);
    }
    if (seat.stack === 0 && record.amount > 0) {
      seat.allIn = true;
      const total = typeof action === "object" ? action.raise.to : record.amount;
      act = `ALL-IN ${chips(total)}`;
      code = "A";
      said = `GOES ALL-IN ${chips(total)}`;
    }
    seat.act = act;
    const replaced = record.rejected ? " (ILLEGAL MOVE REPLACED)" : "";
    push("act", `${seat.name} ${said}${replaced}`, {
      actor: seat.seat,
      code,
      thinkUs: record.think_us ?? null,
      notes: record.notes ?? [],
    });
  });

  const result = hand.result;
  if (result.shown.length > 0) {
    for (const next of ["flop", "turn", "river"] as const) {
      if (BOARD_SIZE[next] > BOARD_SIZE[street] && hand.board.length >= BOARD_SIZE[next]) {
        dealStreet(next, " (ALL-IN, NO MORE BETTING)");
      }
    }
  }

  if (result.uncalled_bet) {
    const { seat, amount } = result.uncalled_bet;
    const s = bySeat.get(seat);
    if (s) {
      s.stack += amount;
      s.act = `RETURNED ${chips(amount)}`;
    }
    pot -= amount;
    push("refund", `${chips(amount)} UNCALLED, RETURNED TO ${nameOf(seat)}`, { actor: seat });
  }

  // Winnings: what each seat's stack gained over its stack before the payout.
  const won = new Map<number, number>();
  for (const seat of seats) {
    const after = hand.final_stacks[seat.seat] ?? seat.stack;
    if (after > seat.stack) won.set(seat.seat, after - seat.stack);
  }
  const winners = [...won.keys()];
  for (const seat of seats) {
    seat.stack = hand.final_stacks[seat.seat] ?? seat.stack;
    seat.isWinner = won.has(seat.seat);
    if (seat.isWinner) seat.act = `+${chips(won.get(seat.seat)!)}`;
  }
  const handName = (seat: number) => {
    const shown = result.shown.find((h) => h.seat === seat);
    return shown ? categoryLabel(shown.value.category).toUpperCase() : null;
  };
  const description = winners
    .map((seat) => {
      const name = handName(seat);
      return `${nameOf(seat)} WINS ${chips(won.get(seat)!)}${name ? ` WITH ${name}` : ", UNCONTESTED"}`;
    })
    .join("; ");
  push("show", description || "POT AWARDED", { tag: "SHOW", winners });

  return frames;
}
