import { isRedSuit, rankLabel, suitSymbol } from "@/lib/format";
import type { Card } from "@/types/log";

interface Props {
  card: Card | null;
  size?: "sm" | "md";
  dimmed?: boolean;
}

/** A face-up card, or a face-down back when `card` is null. */
export function PlayingCard({ card, size = "md", dimmed = false }: Props) {
  const box =
    size === "sm"
      ? "h-9 w-7 text-xs rounded"
      : "h-14 w-10 text-base rounded-md";
  const fade = dimmed ? "opacity-40" : "";

  if (!card) {
    return (
      <div
        aria-label="Face-down card"
        className={`${box} ${fade} border border-white/20 bg-gradient-to-br from-sky-800 to-sky-950`}
      />
    );
  }

  const color = isRedSuit(card.suit) ? "text-red-600" : "text-zinc-900";
  return (
    <div
      aria-label={`${rankLabel(card.rank)} of ${card.suit}`}
      className={`${box} ${fade} ${color} flex flex-col items-center justify-center border border-zinc-300 bg-white font-semibold leading-none shadow-sm`}
    >
      <span>{rankLabel(card.rank)}</span>
      <span>{suitSymbol(card.suit)}</span>
    </div>
  );
}
