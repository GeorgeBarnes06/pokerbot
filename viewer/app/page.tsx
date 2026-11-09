import Link from "next/link";
import { Suspense } from "react";

import { Exhibit } from "@/components/Sheet";
import { chips } from "@/lib/format";
import { listMatches } from "@/lib/logs";

export default function Home() {
  return (
    <Exhibit label="A" title="Match logs">
      <Suspense fallback={<p className="text-muted">LOADING LOGS…</p>}>
        <MatchList />
      </Suspense>
    </Exhibit>
  );
}

async function MatchList() {
  const matches = await listMatches();

  if (matches.length === 0) {
    return (
      <p className="text-muted">
        NO LOGS YET. RUN <span className="text-ink">cargo run --release -p runner</span> FROM THE
        REPO ROOT TO WRITE ONE TO <span className="text-ink">logs/</span>.
      </p>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left">
        <thead className="text-muted">
          <tr>
            <th className="pr-6 font-normal">LOG</th>
            <th className="pr-6 font-normal">FORMAT</th>
            <th className="pr-6 font-normal">SEED</th>
            <th className="pr-6 font-normal">BOTS</th>
            <th className="pr-6 text-right font-normal">HANDS</th>
            <th className="font-normal">LEADER</th>
          </tr>
        </thead>
        <tbody>
          {matches.map((m) => (
            <tr key={m.id}>
              <td className="pr-6">
                <Link href={`/tournaments/${m.id}`} className="hover:bg-warn hover:text-[#111]">
                  [{m.id.toUpperCase()}]
                </Link>
              </td>
              <td className="pr-6">{m.rebuy ? "CASH" : "ELIMINATION"}</td>
              <td className="pr-6 tabular-nums">{m.seed}</td>
              <td className="pr-6 tabular-nums">{m.players}</td>
              <td className="pr-6 text-right tabular-nums">{chips(m.hands)}</td>
              <td className="uppercase">
                {m.finished || m.rebuy ? (
                  <>
                    <span className="text-green">W</span> {m.leader}
                  </>
                ) : (
                  <span className="text-amber">STOPPED AT HAND LIMIT</span>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
