import Link from "next/link";
import { notFound } from "next/navigation";
import { Suspense } from "react";

import { chips } from "@/lib/format";
import { loadTournament } from "@/lib/logs";

export default function TournamentPage({ params }: PageProps<"/tournaments/[id]">) {
  return (
    <Suspense fallback={<p className="text-zinc-500">Loading tournament…</p>}>
      {params.then(({ id }) => (
        <Tournament id={id} />
      ))}
    </Suspense>
  );
}

async function Tournament({ id }: { id: string }) {
  const log = await loadTournament(id);
  if (!log) notFound();

  const nameOf = new Map(log.config.players.map((p, seat) => [seat, p.name]));

  return (
    <div className="flex flex-col gap-8">
      <div>
        <h1 className="text-2xl font-semibold">{id}</h1>
        <p className="mt-1 text-sm text-zinc-500">
          Seed {log.config.seed} · {log.config.players.length} players ·{" "}
          {chips(log.config.starting_stack)} starting chips · {log.hands.length} hands ·{" "}
          {log.finished ? "finished" : "stopped at hand limit"}
        </p>
      </div>

      <section>
        <h2 className="mb-2 text-lg font-semibold">Standings</h2>
        <div className="overflow-x-auto rounded-xl border border-zinc-200 dark:border-zinc-800">
          <table className="w-full text-sm">
            <thead className="bg-zinc-50 text-left text-zinc-500 dark:bg-zinc-900">
              <tr>
                <th className="px-3 py-2">Place</th>
                <th className="px-3 py-2">Player</th>
                <th className="px-3 py-2">Bot</th>
                <th className="px-3 py-2 text-right">Chips</th>
                <th className="px-3 py-2 text-right">Out on hand</th>
              </tr>
            </thead>
            <tbody>
              {log.standings.map((s) => (
                <tr key={s.seat} className="border-t border-zinc-200 dark:border-zinc-800">
                  <td className="px-3 py-2">{s.place}</td>
                  <td className="px-3 py-2 font-medium">{s.name}</td>
                  <td className="px-3 py-2 font-mono text-xs">{s.bot}</td>
                  <td className="px-3 py-2 text-right tabular-nums">{chips(s.final_stack)}</td>
                  <td className="px-3 py-2 text-right tabular-nums">
                    {s.busted_on_hand === null ? (
                      "—"
                    ) : (
                      <Link
                        href={`/tournaments/${id}/hands/${s.busted_on_hand}`}
                        className="text-emerald-700 hover:underline dark:text-emerald-400"
                      >
                        {s.busted_on_hand}
                      </Link>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section>
        <h2 className="mb-2 text-lg font-semibold">Hands</h2>
        <div className="overflow-x-auto rounded-xl border border-zinc-200 dark:border-zinc-800">
          <table className="w-full text-sm">
            <thead className="bg-zinc-50 text-left text-zinc-500 dark:bg-zinc-900">
              <tr>
                <th className="px-3 py-2">#</th>
                <th className="px-3 py-2">Blinds</th>
                <th className="px-3 py-2 text-right">Pot</th>
                <th className="px-3 py-2">Won by</th>
                <th className="px-3 py-2">Ended</th>
              </tr>
            </thead>
            <tbody>
              {log.hands.map((hand) => {
                const pot = hand.result.awards.reduce((sum, a) => sum + a.amount, 0);
                const winners = [...new Set(hand.result.awards.flatMap((a) => a.winners))]
                  .map((seat) => nameOf.get(seat))
                  .join(", ");
                return (
                  <tr key={hand.hand_number} className="border-t border-zinc-200 dark:border-zinc-800">
                    <td className="px-3 py-1.5">
                      <Link
                        href={`/tournaments/${id}/hands/${hand.hand_number}`}
                        className="text-emerald-700 hover:underline dark:text-emerald-400"
                      >
                        {hand.hand_number}
                      </Link>
                    </td>
                    <td className="px-3 py-1.5 tabular-nums">
                      {chips(hand.blinds.small)}/{chips(hand.blinds.big)}
                      {hand.blinds.ante > 0 && ` (ante ${chips(hand.blinds.ante)})`}
                    </td>
                    <td className="px-3 py-1.5 text-right tabular-nums">{chips(pot)}</td>
                    <td className="px-3 py-1.5">{winners}</td>
                    <td className="px-3 py-1.5 text-zinc-500">
                      {hand.result.shown.length > 0 ? "Showdown" : "Everyone folded"}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
