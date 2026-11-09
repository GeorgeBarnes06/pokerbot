// Compact per-hand data for the grid of tiny tables (bitmasks of seats).

import type { GridData } from "@/components/TableGrid";
import type { MatchSummary } from "@/types/log";

const mask = (seats: number[]) => seats.reduce((bits, seat) => bits | (1 << seat), 0);

export function gridData(summary: MatchSummary): GridData {
  return {
    stillIn: summary.hands.map((h) => mask(h.still_in ?? [])),
    winners: summary.hands.map((h) => mask(h.winners)),
    boardCards: summary.hands.map((h) => h.board_cards ?? 0),
  };
}
