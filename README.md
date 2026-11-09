# Pokerbot

A No-Limit Texas Hold'em engine in Rust for matches between 8 bots, with a
Next.js viewer that replays every hand.

The format follows the [No Limit bot tournament](https://pokerbot.sequoiacap.com/):
8 bots per table, blinds 50/100, 10,000-chip stacks with rebuys, 10,000 hands,
ranked by net chips and big blinds per 100 hands.

## Quick start

```bash
cargo run --release -p runner                         # 8-bot match -> logs/default-1/
cargo run --release -p runner -- configs/duel.json    # heads-up duel -> logs/duel-1/
cd viewer && npm install && npm run dev               # http://localhost:3000
```

## Layout

```text
crates/
  engine/   rules: cards, hand evaluator, betting, side pots, hand logs, equity analysis
  bots/     the Bot interface, 8 model stubs, helpers, conformance tests
  runner/   plays a match from a config and writes its logs
configs/    match configs (default.json: 8-bot table, duel.json: heads-up)
logs/       match output, one folder per match (git-ignored)
viewer/     Next.js app: live tables, leaderboard, hand replays
```

## Status

| Part | State |
|---|---|
| Engine | Complete: dealing, blinds/antes, betting with min-raise and all-in rules, side and split pots, showdown |
| Runner | Cash format with rebuys, heads-up duel (stacks reset every hand), elimination tournament |
| Viewer | Live tables (hand grid, 8-seat table, action strips), leaderboard, paged hand list, replays |
| Bots | **To design.** All 8 are stubs that check or call; see [crates/bots/README.md](crates/bots/README.md) |

## The bots

| Config name | Bot |
|---|---|
| `cfr7` | CFR-7 |
| `nash` | NASH |
| `monte` | MONTE |
| `tilt` | TILT |
| `gto_lite` | GTO-LITE |
| `riverrat` | RIVERRAT |
| `foldbot` | FOLDBOT |
| `bayes` | BAYES |
| `check_call` | Baseline that always checks or calls |

The design guide, rules every bot must follow (legal moves, seeded randomness,
100 ms per decision) and the figure-to-data map are in
[crates/bots/README.md](crates/bots/README.md).

## Configs

| Field | Meaning |
|---|---|
| `seed` | Seeds every shuffle and each bot; the same config replays the same match |
| `starting_stack`, `blind_levels`, `hands_per_level` | Stakes; one level means fixed blinds |
| `max_hands` | Match length (or a safety stop for elimination) |
| `rebuy` | Top a stack back up when it falls below one big blind |
| `reset_stacks_each_hand` | Duel format: every stack resets before each hand |
| `equity_samples` | Run-outs per street for the equity figures (0 = off; about 20 s per 10,000 8-bot hands at 100) |
| `players` | One `{ "name", "bot" }` per seat |

## Checks

```bash
cargo test --workspace          # engine, runner, and bot conformance
cargo test --release -p bots    # adds the 100 ms decision budget
cargo clippy --workspace --all-targets
cd viewer && npx tsc --noEmit && npm run lint
```
