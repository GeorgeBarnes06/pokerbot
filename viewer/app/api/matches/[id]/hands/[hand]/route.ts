import { loadHand } from "@/lib/logs";

/** One hand's full log, so the live table can move on without reloading the page. */
export async function GET(_request: Request, ctx: RouteContext<"/api/matches/[id]/hands/[hand]">) {
  const { id, hand } = await ctx.params;
  const log = await loadHand(id, Number(hand));
  if (!log) {
    return Response.json({ error: "hand not found" }, { status: 404 });
  }
  return Response.json(log);
}
