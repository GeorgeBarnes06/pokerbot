import Link from "next/link";
import { Suspense } from "react";

import { listTournaments } from "@/lib/logs";

export default function Home() {
  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-2xl font-semibold">Tournaments</h1>
      <Suspense fallback={<p className="text-zinc-500">Loading logs…</p>}>
        <TournamentList />
      </Suspense>
    </div>
  );
}

async function TournamentList() {
  const tournaments = await listTournaments();

  if (tournaments.length === 0) {
    return (
      <p className="text-zinc-500">
        No logs yet. Run <code className="font-mono">cargo run -p runner</code> from the repo
        root to create one in <code className="font-mono">logs/</code>.
      </p>
    );
  }

  return (
    <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
      {tournaments.map((t) => (
        <li key={t.id}>
          <Link
            href={`/tournaments/${t.id}`}
            className="block rounded-xl border border-zinc-200 p-4 hover:border-emerald-600 dark:border-zinc-800"
          >
            <div className="font-semibold">{t.id}</div>
            <div className="mt-1 text-sm text-zinc-500">
              Seed {t.seed} · {t.players} players · {t.hands} hands
            </div>
            <div className="mt-2 text-sm">
              {t.finished ? `Winner: ${t.winner}` : "Stopped at hand limit"}
            </div>
          </Link>
        </li>
      ))}
    </ul>
  );
}
