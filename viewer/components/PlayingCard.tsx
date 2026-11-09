import { isRedSuit, rankLabel, suitSymbol } from "@/lib/format";
import type { Card } from "@/types/log";

interface Props {
  card: Card | null;
  dimmed?: boolean;
}

/** A card as a small outlined box, e.g. `A♠`; a hatched back when `card` is null. */
export function PlayingCard({ card, dimmed = false }: Props) {
  const box = "inline-flex h-[2.2em] w-[2.6ch] items-center justify-center border leading-none";
  const fade = dimmed ? "opacity-35" : "";

  if (!card) {
    return (
      <span
        aria-label="Face-down card"
        className={`${box} ${fade} border-faint text-faint`}
      >
        ░
      </span>
    );
  }

  const color = isRedSuit(card.suit) ? "text-red" : "text-ink";
  return (
    <span
      aria-label={`${rankLabel(card.rank)} of ${card.suit}`}
      className={`${box} ${fade} ${color} w-[3.2ch] border-ink bg-paper`}
    >
      {rankLabel(card.rank)}
      {suitSymbol(card.suit)}
    </span>
  );
}
