"use client";

import { useEffect, useMemo, useState } from "react";

import { PlayingCard } from "@/components/PlayingCard";
import { chips } from "@/lib/format";
import { buildFrames, streetName, type SeatFrame } from "@/lib/replay";
import type { HandLog } from "@/types/log";

export function HandReplay({ hand }: { hand: HandLog }) {
  const frames = useMemo(() => buildFrames(hand), [hand]);
  const [step, setStep] = useState(0);
  const last = frames.length - 1;
  const frame = frames[Math.min(step, last)];

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "ArrowRight") setStep((s) => Math.min(s + 1, last));
      if (event.key === "ArrowLeft") setStep((s) => Math.max(s - 1, 0));
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [last]);

  return (
    <div className="grid gap-6 lg:grid-cols-[1fr_18rem]">
      <div className="flex flex-col gap-4">
        <section className="rounded-3xl border-4 border-amber-900/60 bg-emerald-800 p-4 text-white shadow-inner sm:p-6">
          <div className="flex flex-col items-center gap-3">
            <div className="text-xs font-medium tracking-wider text-emerald-200 uppercase">
              {streetName(frame.street)}
            </div>
            <div className="flex min-h-14 gap-1.5">
              {frame.board.length === 0 ? (
                <span className="self-center text-sm text-emerald-200/70">No board cards yet</span>
              ) : (
                frame.board.map((card, i) => <PlayingCard key={i} card={card} />)
              )}
            </div>
            <div className="rounded-full bg-black/25 px-3 py-1 text-sm font-semibold">
              Pot {chips(frame.pot)}
            </div>
            <p className="min-h-10 text-center text-sm text-emerald-50">{frame.description}</p>
          </div>

          <div className="mt-4 grid grid-cols-2 gap-2 sm:grid-cols-4">
            {frame.seats.map((seat) => (
              <Seat key={seat.seat} seat={seat} acting={seat.seat === frame.actor} />
            ))}
          </div>
        </section>

        <div className="flex flex-wrap items-center gap-2">
          <ControlButton onClick={() => setStep(0)} disabled={step === 0}>
            ⏮ Start
          </ControlButton>
          <ControlButton onClick={() => setStep((s) => Math.max(s - 1, 0))} disabled={step === 0}>
            ◀ Back
          </ControlButton>
          <ControlButton
            onClick={() => setStep((s) => Math.min(s + 1, last))}
            disabled={step === last}
          >
            Next ▶
          </ControlButton>
          <ControlButton onClick={() => setStep(last)} disabled={step === last}>
            End ⏭
          </ControlButton>
          <span className="ml-auto text-sm text-zinc-500">
            Step {step + 1} of {frames.length} · arrow keys work too
          </span>
        </div>
      </div>

      <aside className="rounded-xl border border-zinc-200 dark:border-zinc-800">
        <h2 className="border-b border-zinc-200 px-3 py-2 text-sm font-semibold dark:border-zinc-800">
          Action log
        </h2>
        <ol className="max-h-[32rem] overflow-y-auto text-sm">
          {frames.map((f, i) => (
            <li key={i}>
              <button
                type="button"
                onClick={() => setStep(i)}
                className={`w-full px-3 py-1.5 text-left hover:bg-zinc-100 dark:hover:bg-zinc-900 ${
                  i === step ? "bg-emerald-50 font-medium dark:bg-emerald-950" : ""
                } ${f.actionIndex === null ? "text-zinc-500" : ""}`}
              >
                {f.description}
              </button>
            </li>
          ))}
        </ol>
      </aside>
    </div>
  );
}

function Seat({ seat, acting }: { seat: SeatFrame; acting: boolean }) {
  const ring = seat.isWinner
    ? "ring-2 ring-amber-300"
    : acting
      ? "ring-2 ring-white"
      : "";
  return (
    <div
      className={`flex flex-col gap-1.5 rounded-xl bg-black/30 p-2 ${ring} ${seat.folded ? "opacity-50" : ""}`}
    >
      <div className="flex items-center justify-between gap-1">
        <span className="truncate text-sm font-semibold">{seat.name}</span>
        {seat.isButton && (
          <span
            title="Dealer button"
            className="rounded-full bg-white px-1.5 text-[10px] font-bold text-zinc-900"
          >
            D
          </span>
        )}
      </div>
      <div className="flex gap-1">
        <PlayingCard card={seat.holeCards?.[0] ?? null} size="sm" dimmed={seat.folded} />
        <PlayingCard card={seat.holeCards?.[1] ?? null} size="sm" dimmed={seat.folded} />
      </div>
      <div className="text-xs text-emerald-100">
        {chips(seat.stack)} chips
        {seat.bet > 0 && <span className="text-amber-200"> · bet {chips(seat.bet)}</span>}
      </div>
      <div className="min-h-4 text-xs font-medium">
        {seat.shownHand ?? seat.lastAction ?? (seat.allIn ? "All-in" : "")}
      </div>
    </div>
  );
}

function ControlButton({
  children,
  onClick,
  disabled,
}: {
  children: React.ReactNode;
  onClick: () => void;
  disabled: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className="rounded-lg border border-zinc-300 px-3 py-1.5 text-sm font-medium hover:bg-zinc-100 disabled:opacity-40 disabled:hover:bg-transparent dark:border-zinc-700 dark:hover:bg-zinc-900"
    >
      {children}
    </button>
  );
}
