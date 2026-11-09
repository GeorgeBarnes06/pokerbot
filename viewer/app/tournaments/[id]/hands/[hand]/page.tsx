import { notFound } from "next/navigation";
import { Suspense } from "react";

import { LiveTables } from "@/components/LiveTables";
import { BracketLink, Divider, Exhibit } from "@/components/Sheet";
import { gridData } from "@/lib/grid";
import { loadHand, loadSummary } from "@/lib/logs";

export default function HandPage({ params, searchParams }: PageProps<"/tournaments/[id]/hands/[hand]">) {
  return (
    <Suspense fallback={<p className="text-muted">LOADING HAND…</p>}>
      {Promise.all([params, searchParams]).then(([{ id, hand }, query]) => (
        <Hand id={id} handNumber={Number(hand)} autoplay={query.autoplay !== undefined} />
      ))}
    </Suspense>
  );
}

async function Hand({ id, handNumber, autoplay }: { id: string; handNumber: number; autoplay: boolean }) {
  const [summary, hand] = await Promise.all([loadSummary(id), loadHand(id, handNumber)]);
  if (!summary || !hand) notFound();

  return (
    <>
      <nav className="flex flex-wrap justify-between gap-x-4">
        <BracketLink href={`/tournaments/${id}`}>&lt; {id}</BracketLink>
      </nav>

      <Divider />

      <Exhibit label="A" title="Live tables">
        {/* Keyed so a fresh link to another hand starts a fresh replay. */}
        <LiveTables
          key={`${id}-${handNumber}`}
          matchId={id}
          totalHands={summary.hands.length}
          tableSize={summary.config.players.length}
          playerCount={summary.config.players.length}
          handsPerSecond={summary.hands_per_second}
          grid={gridData(summary)}
          initialHand={hand}
          autoplay={autoplay}
          syncUrl
        />
      </Exhibit>
    </>
  );
}
