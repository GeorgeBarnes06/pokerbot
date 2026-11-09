import { isRedSuit, rankLabel, suitSymbol } from "@/lib/format";
import type { Card } from "@/types/log";

interface Props {
  /** Null draws an empty dashed slot. */
  card: Card | null;
  size?: "seat" | "board";
  /** Greyed out, for folded hands. */
  muted?: boolean;
}

/** A card with the rank above the suit, outlined with a small offset shadow. */
export function PlayingCard({ card, size = "seat", muted = false }: Props) {
  const box =
    size === "board"
      ? // Seat-sized on phones so the board fits between the seats.
        "h-9 w-[26px] gap-px rounded-[3px] text-[11px] shadow-[1px_1px_0_var(--ink)] sm:h-[58px] sm:w-[42px] sm:gap-[3px] sm:rounded sm:text-[17px] sm:shadow-[2px_2px_0_var(--ink)]"
      : "h-9 w-[26px] gap-px rounded-[3px] text-[11px] shadow-[1px_1px_0_var(--ink)]";
  const suitSize = size === "board" ? "text-xs sm:text-[21px]" : "text-xs";

  if (!card) {
    return (
      <span
        aria-hidden
        className={`${box} inline-flex border border-dashed border-faint shadow-none`}
      />
    );
  }

  const color = muted ? "text-muted" : isRedSuit(card.suit) ? "text-red" : "text-ink";
  const border = muted ? "border-faint shadow-none" : "border-ink";
  return (
    <span
      aria-label={`${rankLabel(card.rank)} of ${card.suit}`}
      className={`${box} ${border} ${color} inline-flex flex-col items-center justify-center border bg-paper leading-none`}
    >
      <span>{rankLabel(card.rank)}</span>
      <span className={suitSize}>{suitSymbol(card.suit)}</span>
    </span>
  );
}
