import Link from "next/link";
import { notFound } from "next/navigation";
import { Suspense } from "react";

import { HandReplay } from "@/components/HandReplay";
import { chips } from "@/lib/format";
import { loadTournament } from "@/lib/logs";

export default function HandPage({ params }: PageProps<"/tournaments/[id]/hands/[hand]">) {
  return (
    <Suspense fallback={<p className="text-zinc-500">Loading hand…</p>}>
      {params.then(({ id, hand }) => (
        <Hand id={id} handNumber={Number(hand)} />
      ))}
    </Suspense>
  );
}

async function Hand({ id, handNumber }: { id: string; handNumber: number }) {
  const log = await loadTournament(id);
  const hand = log?.hands.find((h) => h.hand_number === handNumber);
  if (!log || !hand) notFound();

  const hasPrevious = log.hands.some((h) => h.hand_number === handNumber - 1);
  const hasNext = log.hands.some((h) => h.hand_number === handNumber + 1);
  const link = "text-emerald-700 hover:underline dark:text-emerald-400";

  return (
    <div className="flex flex-col gap-4">
      <nav className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm">
        <Link href={`/tournaments/${id}`} className={link}>
          ← {id}
        </Link>
        <span className="ml-auto flex gap-4">
          {hasPrevious && (
            <Link href={`/tournaments/${id}/hands/${handNumber - 1}`} className={link}>
              ← Hand {handNumber - 1}
            </Link>
          )}
          {hasNext && (
            <Link href={`/tournaments/${id}/hands/${handNumber + 1}`} className={link}>
              Hand {handNumber + 1} →
            </Link>
          )}
        </span>
      </nav>

      <div>
        <h1 className="text-2xl font-semibold">Hand {handNumber}</h1>
        <p className="mt-1 text-sm text-zinc-500">
          Blinds {chips(hand.blinds.small)}/{chips(hand.blinds.big)}
          {hand.blinds.ante > 0 && `, ante ${chips(hand.blinds.ante)}`} · {hand.seats.length}{" "}
          players dealt in · seed {hand.seed}
        </p>
      </div>

      {/* Keyed so the replay restarts from step 1 when moving between hands. */}
      <HandReplay key={`${id}-${handNumber}`} hand={hand} />
    </div>
  );
}
