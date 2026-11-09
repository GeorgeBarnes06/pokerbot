// Server-side access to tournament logs written by the Rust runner.

import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

import type { TournamentLog } from "@/types/log";

/**
 * Defaults to the repo's logs/ folder, next to viewer/. Logs are read at request
 * time, so the file accesses below are excluded from build tracing.
 */
const LOGS_DIR =
  process.env.POKERBOT_LOGS_DIR ?? path.join(process.cwd(), "..", "logs");

/** Log ids are file names without `.json`; anything else could escape LOGS_DIR. */
const ID_PATTERN = /^[\w-]+$/;

export interface TournamentSummary {
  id: string;
  seed: number;
  players: number;
  hands: number;
  finished: boolean;
  winner: string | null;
}

export async function loadTournament(id: string): Promise<TournamentLog | null> {
  if (!ID_PATTERN.test(id)) return null;
  try {
    const text = await readFile(
      /*turbopackIgnore: true*/ path.join(LOGS_DIR, `${id}.json`),
      "utf8",
    );
    return JSON.parse(text) as TournamentLog;
  } catch {
    return null;
  }
}

export async function listTournaments(): Promise<TournamentSummary[]> {
  let files: string[];
  try {
    files = await readdir(/*turbopackIgnore: true*/ LOGS_DIR);
  } catch {
    return [];
  }

  const ids = files
    .filter((file) => file.endsWith(".json"))
    .map((file) => file.slice(0, -".json".length))
    .filter((id) => ID_PATTERN.test(id))
    .sort();

  const summaries = await Promise.all(
    ids.map(async (id): Promise<TournamentSummary | null> => {
      const log = await loadTournament(id);
      if (!log) return null;
      return {
        id,
        seed: log.config.seed,
        players: log.config.players.length,
        hands: log.hands.length,
        finished: log.finished,
        winner: log.finished ? (log.standings[0]?.name ?? null) : null,
      };
    }),
  );
  return summaries.filter((summary) => summary !== null);
}
