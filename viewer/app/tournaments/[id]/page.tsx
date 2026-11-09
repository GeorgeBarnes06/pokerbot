import Link from "next/link";
import { notFound } from "next/navigation";
import { Suspense } from "react";

import { BracketLink, Divider, Exhibit, Stat } from "@/components/Sheet";
import { chips, signedChips } from "@/lib/format";
import { loadSummary } from "@/lib/logs";
import type { MatchSummary } from "@/types/log";

const HANDS_PER_PAGE = 100;

export default function MatchPage({ params, searchParams }: PageProps<"/tournaments/[id]">) {
  return (
    <Suspense fallback={<p className="text-muted">LOADING MATCH…</p>}>
      {Promise.all([params, searchParams]).then(([{ id }, query]) => (
        <Match id={id} page={Number(query.page ?? 1)} />
      ))}
    </Suspense>
  );
}

async function Match({ id, page }: { id: string; page: number }) {
  const summary = await loadSummary(id);
  if (!summary) notFound();

  const { config } = summary;
  const blinds = config.blind_levels[0];
  const cash = config.rebuy || config.reset_stacks_each_hand;
  const fixedBlinds = config.blind_levels.length === 1;
  const showdowns = summary.hands.filter((h) => h.showdown).length;

  return (
    <>
      <div className="flex flex-col gap-1">
        <div className="flex flex-wrap justify-between gap-x-4">
          <h1 className="font-normal uppercase">{id}</h1>
          <BracketLink href={`/tournaments/${id}/hands/1?autoplay=8`}>Watch from hand 1</BracketLink>
        </div>
        <p className="uppercase">
          {config.players.length} bots, no-limit hold&apos;em, blinds {chips(blinds.small)}/
          {chips(blinds.big)}
          {fixedBlinds ? "" : " rising"},{" "}
          {config.reset_stacks_each_hand
            ? `stacks reset to ${chips(config.starting_stack)} every hand`
            : config.rebuy
              ? "rebuys"
              : "elimination"}
        </p>
        <div className="flex flex-wrap gap-x-6">
          <Stat label="SEED" value={config.seed} />
          <Stat label="STACKS" value={chips(config.starting_stack)} />
          <Stat label="HANDS" value={chips(summary.hands.length)} />
          <Stat label="SHOWDOWNS" value={chips(showdowns)} />
          <Stat label="HANDS/SEC" value={chips(Math.round(summary.hands_per_second))} />
          {!summary.finished && <span className="text-amber">STOPPED AT HAND LIMIT</span>}
        </div>
      </div>

      <Divider />

      <Exhibit
        label="A"
        title={cash ? "Leaderboard" : "Final standings"}
        aside={cash ? "ranked by net chips" : "ranked by finish"}
      >
        <Standings summary={summary} id={id} />
      </Exhibit>

      <Divider />

      <Exhibit label="B" title="Every hand" aside={`${chips(summary.hands.length)} hands`}>
        <HandTable summary={summary} id={id} page={page} />
      </Exhibit>
    </>
  );
}

function Standings({ summary, id }: { summary: MatchSummary; id: string }) {
  const rebuy = summary.config.rebuy || summary.config.reset_stacks_each_hand;
  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left">
        <thead className="text-muted">
          <tr>
            <th className="pr-6 font-normal">#</th>
            <th className="pr-6 font-normal">PLAYER</th>
            <th className="pr-6 font-normal">BOT</th>
            {rebuy ? (
              <>
                <th className="pr-6 text-right font-normal">NET CHIPS</th>
                <th className="pr-6 text-right font-normal">BB/100</th>
                <th className="text-right font-normal">REBUYS</th>
              </>
            ) : (
              <>
                <th className="pr-6 text-right font-normal">CHIPS</th>
                <th className="text-right font-normal">OUT ON HAND</th>
              </>
            )}
          </tr>
        </thead>
        <tbody>
          {summary.standings.map((s) => (
            <tr key={s.seat} className={s.place === 1 ? "bg-warn text-[#111]" : ""}>
              <td className="pr-6 tabular-nums">{s.place}</td>
              <td className="pr-6 uppercase">{s.name}</td>
              <td className="pr-6">{s.bot}</td>
              {rebuy ? (
                <>
                  <td
                    className={`pr-6 text-right tabular-nums ${
                      s.place === 1 ? "" : s.net_chips < 0 ? "text-red" : "text-green"
                    }`}
                  >
                    {signedChips(s.net_chips)}
                  </td>
                  <td className="pr-6 text-right tabular-nums">
                    {s.bb_per_100 >= 0 ? "+" : "−"}
                    {Math.abs(s.bb_per_100).toFixed(1)}
                  </td>
                  <td className="text-right tabular-nums">{s.rebuys}</td>
                </>
              ) : (
                <>
                  <td className="pr-6 text-right tabular-nums">{chips(s.final_stack)}</td>
                  <td className="text-right tabular-nums">
                    {s.busted_on_hand === null ? (
                      "—"
                    ) : (
                      <Link
                        href={`/tournaments/${id}/hands/${s.busted_on_hand}`}
                        className="hover:bg-warn hover:text-[#111]"
                      >
                        [{s.busted_on_hand}]
                      </Link>
                    )}
                  </td>
                </>
              )}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function HandTable({ summary, id, page }: { summary: MatchSummary; id: string; page: number }) {
  const pages = Math.max(1, Math.ceil(summary.hands.length / HANDS_PER_PAGE));
  const current = Number.isInteger(page) ? Math.min(Math.max(page, 1), pages) : 1;
  const start = (current - 1) * HANDS_PER_PAGE;
  const rows = summary.hands.slice(start, start + HANDS_PER_PAGE);
  const nameOf = new Map(summary.config.players.map((p, seat) => [seat, p.name]));
  const pageHref = (n: number) => `/tournaments/${id}?page=${n}`;

  const pager = (
    <div className="flex flex-wrap items-center justify-between gap-x-4">
      <span className="flex gap-3">
        {current > 1 ? <BracketLink href={pageHref(1)}>|&lt;</BracketLink> : <span className="text-faint">[|&lt;]</span>}
        {current > 1 ? <BracketLink href={pageHref(current - 1)}>&lt;</BracketLink> : <span className="text-faint">[&lt;]</span>}
        {current < pages ? <BracketLink href={pageHref(current + 1)}>&gt;</BracketLink> : <span className="text-faint">[&gt;]</span>}
        {current < pages ? <BracketLink href={pageHref(pages)}>&gt;|</BracketLink> : <span className="text-faint">[&gt;|]</span>}
      </span>
      <span className="text-muted">
        HANDS {chips(start + 1)}–{chips(start + rows.length)} · PAGE {current}/{pages}
      </span>
    </div>
  );

  return (
    <div className="flex flex-col gap-2">
      {pager}
      <div className="overflow-x-auto">
        <table className="w-full text-left">
          <thead className="text-muted">
            <tr>
              <th className="pr-6 font-normal">HAND</th>
              <th className="pr-6 font-normal">BLINDS</th>
              <th className="pr-6 text-right font-normal">POT</th>
              <th className="pr-6 font-normal">WON BY</th>
              <th className="font-normal">ENDED</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((hand) => (
              <tr key={hand.hand_number}>
                <td className="pr-6 tabular-nums">
                  <Link
                    href={`/tournaments/${id}/hands/${hand.hand_number}`}
                    className="hover:bg-warn hover:text-[#111]"
                  >
                    [{String(hand.hand_number).padStart(5, "0")}]
                  </Link>
                </td>
                <td className="pr-6 whitespace-nowrap tabular-nums">
                  {chips(hand.blinds.small)}/{chips(hand.blinds.big)}
                  {hand.blinds.ante > 0 && <span className="text-muted"> +{chips(hand.blinds.ante)}</span>}
                </td>
                <td className="pr-6 text-right tabular-nums">{chips(hand.pot)}</td>
                <td className="pr-6 uppercase">
                  <span className="text-green">W</span>{" "}
                  {hand.winners.map((seat) => nameOf.get(seat)).join(", ")}
                </td>
                <td className={hand.showdown ? "" : "text-muted"}>
                  {hand.showdown ? "SHOWDOWN" : "FOLDS"}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {pager}
    </div>
  );
}
