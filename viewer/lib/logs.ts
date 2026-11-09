// Server-side access to match logs written by the Rust runner.
//
// Each match is a folder: logs/<id>/summary.json plus logs/<id>/hands/<n>.json.

import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

import type { HandLog, MatchSummary } from "@/types/log";

/**
 * Defaults to the repo's logs/ folder, next to viewer/. Logs are read at request
 * time, so the file accesses below are excluded from build tracing.
 */
const LOGS_DIR =
  process.env.POKERBOT_LOGS_DIR ?? path.join(process.cwd(), "..", "logs");

/** Match ids are folder names; anything else could escape LOGS_DIR. */
const ID_PATTERN = /^[\w-]+$/;

export interface MatchListing {
  id: string;
  seed: number;
  players: number;
  hands: number;
  rebuy: boolean;
  finished: boolean;
  leader: string | null;
}

async function readJson<T>(...parts: string[]): Promise<T | null> {
  try {
    const text = await readFile(/*turbopackIgnore: true*/ path.join(LOGS_DIR, ...parts), "utf8");
    return JSON.parse(text) as T;
  } catch {
    return null;
  }
}

export async function loadSummary(id: string): Promise<MatchSummary | null> {
  if (!ID_PATTERN.test(id)) return null;
  return readJson<MatchSummary>(id, "summary.json");
}

export async function loadHand(id: string, handNumber: number): Promise<HandLog | null> {
  if (!ID_PATTERN.test(id) || !Number.isSafeInteger(handNumber) || handNumber < 1) return null;
  return readJson<HandLog>(id, "hands", `${handNumber}.json`);
}

export async function listMatches(): Promise<MatchListing[]> {
  let entries: string[];
  try {
    entries = await readdir(/*turbopackIgnore: true*/ LOGS_DIR);
  } catch {
    return [];
  }

  const listings = await Promise.all(
    entries
      .filter((id) => ID_PATTERN.test(id))
      .sort()
      .map(async (id): Promise<MatchListing | null> => {
        const summary = await loadSummary(id);
        if (!summary) return null;
        return {
          id,
          seed: summary.config.seed,
          players: summary.config.players.length,
          hands: summary.hands.length,
          rebuy: summary.config.rebuy,
          finished: summary.finished,
          leader: summary.standings[0]?.name ?? null,
        };
      }),
  );
  return listings.filter((listing) => listing !== null);
}
