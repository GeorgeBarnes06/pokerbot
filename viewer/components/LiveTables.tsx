"use client";

import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";

import { PlayingCard } from "@/components/PlayingCard";
import { TableGrid, type GridData, type LiveCell } from "@/components/TableGrid";
import { chips } from "@/lib/format";
import { ACTION_CODES, buildFrames, streetName, WAIT, type Frame, type SeatFrame } from "@/lib/replay";
import { seatPosition } from "@/lib/seats";
import type { HandLog } from "@/types/log";

/** Milliseconds per wait unit at each speed (from the reference site). */
const SPEED_MS = { 1: 320, 4: 80, 8: 40, 16: 20 } as const;
type Speed = keyof typeof SPEED_MS;
const SPEEDS = [1, 4, 8, 16] as const;

interface Props {
  matchId: string;
  totalHands: number;
  tableSize: number;
  playerCount: number;
  handsPerSecond: number;
  grid: GridData;
  initialHand: HandLog;
  autoplay: boolean;
  /** Keep the address bar on the featured hand (for the hand page). */
  syncUrl: boolean;
}

export function LiveTables({
  matchId,
  totalHands,
  tableSize,
  playerCount,
  handsPerSecond,
  grid,
  initialHand,
  autoplay,
  syncUrl,
}: Props) {
  const [hand, setHand] = useState(initialHand);
  const [step, setStep] = useState(0);
  const [playing, setPlaying] = useState(autoplay);
  const [touched, setTouched] = useState(false);
  const [speed, setSpeed] = useState<Speed>(8);
  const [inView, setInView] = useState(true);
  const [shown, setShown] = useState(0);
  const rootRef = useRef<HTMLDivElement>(null);
  const cache = useRef(new Map<number, HandLog>([[initialHand.hand_number, initialHand]]));
  const requested = useRef(initialHand.hand_number);

  const frames = useMemo(() => buildFrames(hand), [hand]);
  const last = frames.length - 1;
  const frame = frames[Math.min(step, last)];

  const reducedMotion = useSyncExternalStore(subscribeReducedMotion, prefersReducedMotion, () => false);
  const pageVisible = useSyncExternalStore(subscribeVisibility, () => !document.hidden, () => true);
  // Autoplay respects reduced motion until the viewer presses play themselves.
  const running = playing && inView && pageVisible && (touched || !reducedMotion);

  const fetchHand = useCallback(
    async (number: number): Promise<HandLog | null> => {
      const cached = cache.current.get(number);
      if (cached) return cached;
      const response = await fetch(`/api/matches/${matchId}/hands/${number}`);
      if (!response.ok) return null;
      const log = (await response.json()) as HandLog;
      cache.current.set(number, log);
      return log;
    },
    [matchId],
  );

  const goToHand = useCallback(
    async (number: number) => {
      if (number < 1 || number > totalHands) return;
      requested.current = number;
      const log = await fetchHand(number);
      if (!log || requested.current !== number) return;
      setHand(log);
      setStep(0);
    },
    [fetchHand, totalHands],
  );

  // Advance one step after the pause that step deserves; then on to the next hand.
  useEffect(() => {
    if (!running) return;
    const timer = setTimeout(
      () => {
        if (step < last) setStep(step + 1);
        else if (hand.hand_number < totalHands) void goToHand(hand.hand_number + 1);
        else setPlaying(false);
      },
      SPEED_MS[speed] * WAIT[frame.type],
    );
    return () => clearTimeout(timer);
  }, [running, step, last, speed, frame.type, hand.hand_number, totalHands, goToHand]);

  // Fetch the next hand early so playback never waits on the network.
  useEffect(() => {
    if (hand.hand_number < totalHands) void fetchHand(hand.hand_number + 1);
  }, [hand.hand_number, totalHands, fetchHand]);

  useEffect(() => {
    if (syncUrl) window.history.replaceState(null, "", `/tournaments/${matchId}/hands/${hand.hand_number}`);
  }, [syncUrl, matchId, hand.hand_number]);

  // Pause while scrolled out of view.
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    const observer = new IntersectionObserver(([entry]) => setInView(entry.isIntersecting));
    observer.observe(root);
    return () => observer.disconnect();
  }, []);

  const control = (fn: () => void) => () => {
    setTouched(true);
    fn();
  };
  const stepTo = (target: number) => {
    setTouched(true);
    setPlaying(false);
    setStep(Math.max(0, Math.min(target, last)));
  };
  const pickHand = (number: number) => {
    setTouched(true);
    void goToHand(number);
  };

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      if (target?.closest("input, textarea, select, a, button")) return;
      if (event.key === "ArrowRight") {
        setTouched(true);
        setPlaying(false);
        setStep((s) => Math.min(s + 1, last));
      } else if (event.key === "ArrowLeft") {
        setTouched(true);
        setPlaying(false);
        setStep((s) => Math.max(s - 1, 0));
      } else if (event.key === " ") {
        event.preventDefault();
        setTouched(true);
        setPlaying((p) => !p);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [last]);

  const live: LiveCell = useMemo(() => {
    let stillIn = 0;
    let winners = 0;
    for (const seat of frame.seats) {
      if (seat.holeCards && !seat.folded) stillIn |= 1 << seat.seat;
      if (seat.isWinner && frame.type === "show") winners |= 1 << seat.seat;
    }
    return { stillIn, winners, boardCards: frame.board.length };
  }, [frame]);

  const played = hand.hand_number - 1 + (step === last ? 1 : 0);
  // What the play button shows: playing, unless reduced motion is holding autoplay back.
  const isPlaying = playing && (touched || !reducedMotion);
  const handLabel = String(hand.hand_number).padStart(6, "0");

  return (
    <div ref={rootRef} className="flex flex-col">
      <Line>
        <span>
          {chips(totalHands)} HANDS, {playerCount} BOTS, NO-LIMIT HOLD&apos;EM
        </span>
        <span>RECORDED PLAY</span>
      </Line>
      <Line>
        <span>HANDS PLAYED: {chips(played)}</span>
        <span>HANDS/SEC: {chips(Math.round(handsPerSecond))}</span>
      </Line>

      <TableGrid
        totalHands={totalHands}
        featured={hand.hand_number}
        tableSize={tableSize}
        data={grid}
        live={live}
        onPick={pickHand}
        onShownChange={setShown}
      />
      <Line>
        <span>
          SHOWING {chips(shown)} OF {chips(totalHands)} HANDS
        </span>
        <span>▲ HAND {chips(hand.hand_number)}, FEATURED BELOW</span>
      </Line>

      <div className="mt-6 flex items-center justify-center gap-3">
        <TextButton label="Previous hand" onClick={() => pickHand(hand.hand_number - 1)} disabled={hand.hand_number <= 1}>
          &lt;
        </TextButton>
        <span>HAND {handLabel}</span>
        <TextButton
          label="Next hand"
          onClick={() => pickHand(hand.hand_number + 1)}
          disabled={hand.hand_number >= totalHands}
        >
          &gt;
        </TextButton>
      </div>
      <Line>
        <span>REPLAY</span>
        <span className="flex flex-wrap items-center justify-end gap-x-1">
          <TextButton label="Back to start of hand" onClick={() => stepTo(0)}>
            |&lt;
          </TextButton>
          <TextButton label="Previous action" onClick={() => stepTo(step - 1)}>
            &lt;
          </TextButton>
          <TextButton label={isPlaying ? "Pause" : "Play"} onClick={control(() => setPlaying(!isPlaying))}>
            {isPlaying ? "PAUSE" : "PLAY"}
          </TextButton>
          <TextButton label="Next action" onClick={() => stepTo(step + 1)}>
            &gt;
          </TextButton>
          <span className="ml-2">SPEED</span>
          {SPEEDS.map((s) => (
            <button
              key={s}
              type="button"
              aria-pressed={speed === s}
              aria-label={`Speed ${s}x`}
              onClick={control(() => setSpeed(s))}
              className={speed === s ? "bg-ink px-[.3ch] text-paper" : "px-[.3ch] hover:bg-warn hover:text-[#111]"}
            >
              {s}x
            </button>
          ))}
        </span>
      </Line>
      <Line>
        <span>
          HAND {handLabel}, BLINDS {chips(hand.blinds.small)}/{chips(hand.blinds.big)}
          {hand.blinds.ante > 0 && `, ANTE ${chips(hand.blinds.ante)}`}
        </span>
        <span>{frame.type === "show" ? "SHOWDOWN" : streetName(frame.street)}</span>
      </Line>

      <Table frame={frame} tableSize={tableSize} />

      <div className="mt-4 text-center">ACTION</div>
      <Strips frames={frames} step={step} onStep={stepTo} />
      <p className="flex flex-wrap gap-x-[1ch] text-muted">
        {ACTION_CODES.map(([code, name]) => (
          <span key={code} className="whitespace-nowrap">
            {code} {name}
          </span>
        ))}
      </p>

      <Readout frame={frame} />
    </div>
  );
}

function Table({ frame, tableSize }: { frame: Frame; tableSize: number }) {
  return (
    <figure aria-label="Featured table" className="relative mx-auto my-6 aspect-[2/3] w-full sm:aspect-[2/1]">
      <div className="absolute inset-[15%_10%] rounded-[50%] border border-ink shadow-[0_0_0_5px_var(--paper),0_0_0_6px_var(--ink)]" />
      <div className="absolute inset-0 flex flex-col items-center justify-center gap-2">
        <div className="flex gap-1.5">
          {Array.from({ length: 5 }, (_, i) => (
            <PlayingCard key={i} card={frame.board[i] ?? null} size="board" />
          ))}
        </div>
        <div>POT {chips(frame.pot)}</div>
      </div>
      {frame.seats.map((seat) => {
        // Wide tables use a flatter ellipse; the portrait (phone) table a taller one.
        const wide = seatPosition(seat.seat, tableSize, 43, 39);
        const tall = seatPosition(seat.seat, tableSize, 33, 41);
        const style = {
          "--xw": `${wide.left}%`,
          "--yw": `${wide.top}%`,
          "--xt": `${tall.left}%`,
          "--yt": `${tall.top}%`,
        } as React.CSSProperties;
        return (
          <div
            key={seat.seat}
            style={style}
            className="absolute top-[var(--yt)] left-[var(--xt)] -translate-x-1/2 -translate-y-1/2 sm:top-[var(--yw)] sm:left-[var(--xw)]"
          >
            <Seat seat={seat} turn={frame.actor === seat.seat && frame.type !== "show"} won={frame.type === "show" && seat.isWinner} />
          </div>
        );
      })}
    </figure>
  );
}

function Seat({ seat, turn, won }: { seat: SeatFrame; turn: boolean; won: boolean }) {
  const equity = seat.equity === null ? "--" : `${Math.round(seat.equity * 100)}%`;
  return (
    <div
      className={`flex w-[14ch] flex-col items-center bg-paper py-[3px] text-[11px] leading-[1.35] ${
        seat.folded ? "text-muted" : ""
      }`}
    >
      <div className="mb-1 flex min-h-[37px] gap-1">
        {seat.holeCards?.map((card, i) => <PlayingCard key={i} card={card} muted={seat.folded} />)}
      </div>
      <div className={`px-[.6ch] whitespace-nowrap ${turn || won ? "bg-ink text-paper" : ""}`}>
        {seat.label}
        {seat.isButton ? " (D)" : ""}
      </div>
      <div className="whitespace-nowrap">
        {chips(seat.stack)} {equity}
      </div>
      <div className={`my-0.5 h-1 w-[10ch] border ${seat.folded ? "border-faint" : "border-ink"}`}>
        <i className="block h-full bg-ink" style={{ width: `${(seat.equity ?? 0) * 100}%` }} />
      </div>
      <div className="min-h-[1.35em] whitespace-nowrap">{seat.act || " "}</div>
    </div>
  );
}

/** Centre labels on their step, but keep the first and last inside the track. */
function anchor(percent: number): string {
  if (percent <= 2) return "translate-x-0";
  if (percent >= 98) return "-translate-x-full";
  return "-translate-x-1/2";
}

/** One row per seat: action letters laid out along the hand's timeline. */
function Strips({ frames, step, onStep }: { frames: Frame[]; step: number; onStep: (step: number) => void }) {
  const last = Math.max(1, frames.length - 1);
  const pos = (j: number) => (j / last) * 100;
  const current = frames[Math.min(step, frames.length - 1)];
  const tags = frames.flatMap((f, j) => (f.tag ? [{ j, tag: f.tag }] : []));

  return (
    <div className="flex flex-col" style={{ "--progress": `${pos(step)}%` } as React.CSSProperties}>
      <StripRow name="STREET" short="">
        {tags.map(({ j, tag }, i) => {
          // Streets dealt back to back (an all-in run-out) shrink to their first letter.
          const crowded = i > 0 && pos(j) - pos(tags[i - 1].j) < 8;
          return (
            <button
              key={j}
              type="button"
              onClick={() => onStep(j)}
              style={{ left: `${pos(j)}%` }}
              className={`absolute top-0 ${anchor(pos(j))} bg-paper px-[.2ch] hover:bg-ink hover:text-paper ${
                j > step ? "text-muted" : ""
              }`}
            >
              {crowded ? tag[0] : tag}
            </button>
          );
        })}
      </StripRow>
      {current.seats.map((seat) => (
        <StripRow key={seat.seat} name={seat.label} short={`S${seat.seat + 1}`} out={seat.folded}>
          {frames.flatMap((f, j) => {
            const code = f.actor === seat.seat ? f.code : f.type === "show" && f.winners.includes(seat.seat) ? "W" : null;
            if (!code) return [];
            const future = j > step;
            const win = code === "W";
            return [
              <b
                key={j}
                style={{ left: `${pos(j)}%` }}
                className={`absolute top-0 -translate-x-1/2 px-[.2ch] font-normal ${
                  win && !future ? "bg-ink text-paper" : "bg-paper"
                } ${future ? "text-muted" : ""}`}
              >
                {code}
              </b>,
            ];
          })}
        </StripRow>
      ))}
    </div>
  );
}

function StripRow({
  name,
  short,
  out = false,
  children,
}: {
  name: string;
  short: string;
  out?: boolean;
  children: React.ReactNode;
}) {
  return (
    <div className="flex gap-x-[1ch]">
      <span className={`w-[4ch] flex-none overflow-hidden whitespace-nowrap sm:w-[14ch] ${out ? "text-muted" : ""}`}>
        <span className="hidden sm:inline">{name}</span>
        <span className="sm:hidden">{short}</span>
      </span>
      <span className="relative h-[1.8em] min-w-0 flex-1">
        <span className="absolute inset-y-0 right-[1ch] left-[1ch]">
          <span className="absolute top-[53%] left-0 h-px w-[var(--progress)] bg-ink" />
          <span className="absolute top-[53%] right-0 left-[var(--progress)] h-px bg-faint" />
          {children}
        </span>
      </span>
    </div>
  );
}

function Readout({ frame }: { frame: Frame }) {
  return (
    <div className="mt-2 min-h-[1.8em] border-y border-faint py-1">
      <p>
        <span className="text-muted">&gt; </span>
        {frame.description}
        {frame.thinkUs !== null && <span className="text-muted"> · {formatThink(frame.thinkUs)}</span>}
      </p>
      {frame.notes.map((note, i) => (
        <p key={i} className="pl-[2ch] text-muted">
          {note}
        </p>
      ))}
    </div>
  );
}

function Line({ children }: { children: React.ReactNode }) {
  return <div className="flex flex-wrap justify-between gap-x-4 uppercase">{children}</div>;
}

function TextButton({
  children,
  label,
  onClick,
  disabled = false,
}: {
  children: React.ReactNode;
  label: string;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      disabled={disabled}
      className="hover:bg-warn hover:text-[#111] disabled:text-faint disabled:hover:bg-transparent"
    >
      [{children}]
    </button>
  );
}

function formatThink(us: number): string {
  return us >= 1000 ? `${(us / 1000).toFixed(1)} MS` : `${us} µS`;
}

function subscribeReducedMotion(onChange: () => void) {
  const media = window.matchMedia("(prefers-reduced-motion: reduce)");
  media.addEventListener("change", onChange);
  return () => media.removeEventListener("change", onChange);
}

function prefersReducedMotion() {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function subscribeVisibility(onChange: () => void) {
  document.addEventListener("visibilitychange", onChange);
  return () => document.removeEventListener("visibilitychange", onChange);
}
