"use client";

import { useRouter, useSearchParams } from "next/navigation";
import { useEffect, useMemo, useState } from "react";

import { PlayingCard } from "@/components/PlayingCard";
import { chips } from "@/lib/format";
import { ACTION_CODES, buildFrames, streetName, type SeatFrame } from "@/lib/replay";
import type { Card, HandLog } from "@/types/log";

const SPEEDS = [1, 4, 8, 16] as const;
type Speed = (typeof SPEEDS)[number];

/** Milliseconds per step at 1x. */
const BASE_DELAY = 1000;

interface Props {
  hand: HandLog;
  /** Seats at the table, so positions stay put as players bust. */
  tableSize: number;
  /** Next hand's URL; autoplay continues there when this hand ends. */
  nextHref: string | null;
}

export function HandReplay({ hand, tableSize, nextHref }: Props) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const requested = Number(searchParams.get("autoplay"));
  const initialSpeed = SPEEDS.find((s) => s === requested);

  const frames = useMemo(() => buildFrames(hand), [hand]);
  const last = frames.length - 1;
  const [step, setStep] = useState(0);
  const [playing, setPlaying] = useState(initialSpeed !== undefined);
  const [speed, setSpeed] = useState<Speed>(initialSpeed ?? 4);
  const frame = frames[Math.min(step, last)];

  useEffect(() => {
    if (!playing) return;
    const delay = BASE_DELAY / speed;
    if (step < last) {
      const timer = setTimeout(() => setStep((s) => Math.min(s + 1, last)), delay);
      return () => clearTimeout(timer);
    }
    if (nextHref) {
      const timer = setTimeout(() => router.push(`${nextHref}?autoplay=${speed}`), delay * 3);
      return () => clearTimeout(timer);
    }
  }, [playing, speed, step, last, nextHref, router]);

  const goTo = (target: number) => {
    setPlaying(false);
    setStep(Math.max(0, Math.min(target, last)));
  };

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "ArrowRight") {
        setPlaying(false);
        setStep((s) => Math.min(s + 1, last));
      } else if (event.key === "ArrowLeft") {
        setPlaying(false);
        setStep((s) => Math.max(s - 1, 0));
      } else if (event.key === " ") {
        event.preventDefault();
        setPlaying((p) => !p);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [last]);

  const isPlaying = playing && (step < last || nextHref !== null);

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-1">
        <span className="flex gap-1">
          <TextButton label="First step" onClick={() => goTo(0)} disabled={step === 0}>
            |&lt;
          </TextButton>
          <TextButton label="Previous step" onClick={() => goTo(step - 1)} disabled={step === 0}>
            &lt;
          </TextButton>
          <TextButton
            label={isPlaying ? "Pause" : "Play"}
            onClick={() => {
              if (!isPlaying && step === last && !nextHref) setStep(0);
              setPlaying(!isPlaying);
            }}
            wide
          >
            {isPlaying ? "PAUSE" : "PLAY"}
          </TextButton>
          <TextButton label="Next step" onClick={() => goTo(step + 1)} disabled={step === last}>
            &gt;
          </TextButton>
          <TextButton label="Last step" onClick={() => goTo(last)} disabled={step === last}>
            &gt;|
          </TextButton>
        </span>
        <span className="flex items-center gap-1">
          <span className="text-muted">SPEED</span>
          {SPEEDS.map((s) => (
            <button
              key={s}
              type="button"
              aria-pressed={speed === s}
              onClick={() => setSpeed(s)}
              className={`px-1 ${speed === s ? "bg-ink text-paper" : "hover:bg-warn hover:text-[#111]"}`}
            >
              {s}x
            </button>
          ))}
        </span>
        <span className="ml-auto text-muted">
          STEP {step + 1}/{frames.length} · ←/→ STEP · SPACE PLAY
        </span>
      </div>

      <div className="flex flex-wrap justify-between gap-x-4 uppercase">
        <span>
          HAND {String(hand.hand_number).padStart(6, "0")}, BLINDS {chips(hand.blinds.small)}/
          {chips(hand.blinds.big)}
          {hand.blinds.ante > 0 && `, ANTE ${chips(hand.blinds.ante)}`}
        </span>
        <span>{streetName(frame.street)}</span>
      </div>

      {/* Oval table on wider screens; a plain list of seats on phones. */}
      <figure
        aria-label="Table"
        className="relative mx-auto my-6 hidden aspect-[2/1] w-full max-w-4xl sm:block"
      >
        <div className="absolute inset-[15%_10%] rounded-[50%] border border-ink shadow-[0_0_0_5px_var(--paper),0_0_0_6px_var(--ink)]" />
        <TableCenter board={frame.board} pot={frame.pot} />
        {frame.seats.map((seat) => {
          const angle = Math.PI / 2 + (seat.seat / tableSize) * 2 * Math.PI;
          return (
            <div
              key={seat.seat}
              className="absolute w-[15ch] -translate-x-1/2 -translate-y-1/2"
              style={{
                left: `${50 + 45 * Math.cos(angle)}%`,
                top: `${50 + 43 * Math.sin(angle)}%`,
              }}
            >
              <SeatBox seat={seat} acting={seat.seat === frame.actor} />
            </div>
          );
        })}
      </figure>
      <div className="flex flex-col gap-3 sm:hidden">
        <TableCenter board={frame.board} pot={frame.pot} inline />
        <div className="grid grid-cols-2 gap-2">
          {frame.seats.map((seat) => (
            <SeatBox key={seat.seat} seat={seat} acting={seat.seat === frame.actor} />
          ))}
        </div>
      </div>

      <div className="min-h-[1.8em] border-y border-faint py-1">
        <p>
          <span className="text-muted">&gt; </span>
          {frame.description}
          {frame.thinkUs !== null && (
            <span className="text-muted"> · {formatThink(frame.thinkUs)}</span>
          )}
        </p>
        {frame.notes.map((note, i) => (
          <p key={i} className="pl-[2ch] text-muted">
            {note}
          </p>
        ))}
      </div>

      <ActionStrips seats={frame.seats} />

      <p className="flex flex-wrap gap-x-3 text-muted">
        {ACTION_CODES.map(([code, name]) => (
          <span key={code} className="whitespace-nowrap">
            <span className={codeColor(code)}>{code}</span> {name}
          </span>
        ))}
      </p>

      <details className="border-t border-faint pt-2">
        <summary className="cursor-pointer uppercase text-muted">Full log ({frames.length} steps)</summary>
        <ol className="mt-2">
          {frames.map((f, i) => (
            <li key={i}>
              <button
                type="button"
                onClick={() => goTo(i)}
                className={`w-full text-left ${i === step ? "bg-warn text-[#111]" : "hover:bg-faint/40"} ${
                  f.actionIndex === null && i !== step ? "text-muted" : ""
                }`}
              >
                {String(i + 1).padStart(3, " ")} {f.description}
              </button>
            </li>
          ))}
        </ol>
      </details>
    </div>
  );
}

function TableCenter({ board, pot, inline = false }: { board: Card[]; pot: number; inline?: boolean }) {
  return (
    <div
      className={
        inline
          ? "flex flex-col items-center gap-1"
          : "absolute inset-0 flex flex-col items-center justify-center gap-1"
      }
    >
      <div className="flex min-h-[2.2em] gap-1">
        {board.length === 0 ? (
          <span className="self-center text-muted">[ NO BOARD ]</span>
        ) : (
          board.map((card, i) => <PlayingCard key={i} card={card} />)
        )}
      </div>
      <div>POT {chips(pot)}</div>
    </div>
  );
}

function SeatBox({ seat, acting }: { seat: SeatFrame; acting: boolean }) {
  const frame = seat.isWinner
    ? "border-ink bg-warn text-[#111]"
    : acting
      ? "border-ink bg-paper"
      : "border-faint bg-paper";
  const status = seat.shownHand ?? seat.lastAction ?? (seat.allIn ? "All-in" : "");
  return (
    <div
      className={`flex flex-col items-center border px-1 py-0.5 text-[11px] leading-[1.35] ${frame} ${
        seat.folded && !seat.isWinner ? "text-muted" : ""
      }`}
    >
      <span className="w-full truncate text-center uppercase">
        {seat.name}
        {seat.isButton && <span title="Dealer button"> (D)</span>}
      </span>
      <span className="my-0.5 flex gap-0.5">
        <PlayingCard card={seat.holeCards?.[0] ?? null} dimmed={seat.folded} />
        <PlayingCard card={seat.holeCards?.[1] ?? null} dimmed={seat.folded} />
      </span>
      <span>
        {chips(seat.stack)}
        {seat.bet > 0 && <span className={seat.isWinner ? "" : "text-amber"}> +{chips(seat.bet)}</span>}
      </span>
      <span className="min-h-[1.35em] w-full truncate text-center uppercase">{status}</span>
    </div>
  );
}

function ActionStrips({ seats }: { seats: SeatFrame[] }) {
  const columns = ["PREFLOP", "FLOP", "TURN", "RIVER"];
  return (
    <div className="overflow-x-auto">
      <table className="w-full">
        <thead>
          <tr className="text-left text-muted">
            <th className="pr-4 font-normal">ACTION</th>
            {columns.map((c) => (
              <th key={c} className="pr-4 font-normal">
                {c}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {seats.map((seat) => (
            <tr key={seat.seat}>
              <td className="pr-4 whitespace-nowrap uppercase">{seat.name}</td>
              {seat.strip.map((codes, i) => (
                <td key={i} className="pr-4 tracking-[0.3em]">
                  {codes
                    ? [...codes].map((code, j) => (
                        <span key={j} className={codeColor(code)}>
                          {code}
                        </span>
                      ))
                    : <span className="text-faint">·</span>}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function formatThink(us: number): string {
  return us >= 1000 ? `${(us / 1000).toFixed(1)} MS` : `${us} µS`;
}

function codeColor(code: string): string {
  switch (code) {
    case "W":
      return "text-green";
    case "A":
      return "text-red";
    case "B":
    case "R":
      return "text-amber";
    case "F":
      return "text-muted";
    default:
      return "text-ink";
  }
}

function TextButton({
  children,
  label,
  onClick,
  disabled = false,
  wide = false,
}: {
  children: React.ReactNode;
  label: string;
  onClick: () => void;
  disabled?: boolean;
  wide?: boolean;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      disabled={disabled}
      className={`${wide ? "min-w-[7ch]" : ""} hover:bg-warn hover:text-[#111] disabled:text-faint disabled:hover:bg-transparent`}
    >
      [{children}]
    </button>
  );
}
