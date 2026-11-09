import { notFound } from "next/navigation";
import { Suspense } from "react";

import { HandReplay } from "@/components/HandReplay";
import { BracketLink, Divider, Exhibit } from "@/components/Sheet";
import { loadHand, loadSummary } from "@/lib/logs";

export default function HandPage({ params }: PageProps<"/tournaments/[id]/hands/[hand]">) {
  return (
    <Suspense fallback={<p className="text-muted">LOADING HAND…</p>}>
      {params.then(({ id, hand }) => (
        <Hand id={id} handNumber={Number(hand)} />
      ))}
    </Suspense>
  );
}

async function Hand({ id, handNumber }: { id: string; handNumber: number }) {
  const [summary, hand] = await Promise.all([loadSummary(id), loadHand(id, handNumber)]);
  if (!summary || !hand) notFound();

  const hasPrevious = handNumber > 1;
  const hasNext = handNumber < summary.hands.length;
  const nextHref = hasNext ? `/tournaments/${id}/hands/${handNumber + 1}` : null;

  return (
    <>
      <nav className="flex flex-wrap justify-between gap-x-4">
        <BracketLink href={`/tournaments/${id}`}>&lt; {id}</BracketLink>
        <span className="flex gap-4">
          {hasPrevious && (
            <BracketLink href={`/tournaments/${id}/hands/${handNumber - 1}`}>&lt; Hand {handNumber - 1}</BracketLink>
          )}
          {nextHref && <BracketLink href={nextHref}>Hand {handNumber + 1} &gt;</BracketLink>}
        </span>
      </nav>

      <Divider />

      <Exhibit
        label="C"
        title="Hand replay"
        aside={`${hand.seats.length} dealt in · seed ${hand.seed}`}
      >
        {/* Keyed so the replay restarts from step 1 when moving between hands. */}
        <HandReplay
          key={`${id}-${handNumber}`}
          hand={hand}
          tableSize={summary.config.players.length}
          nextHref={nextHref}
        />
      </Exhibit>
    </>
  );
}
