# Bots

Eight models, one file each, all playing the format from the reference site:
8-handed No-Limit Hold'em, blinds 50/100, 10,000-chip stacks, rebuys, 10,000 hands.

| File | Name | What the name suggests |
|---|---|---|
| `src/cfr7.rs` | CFR-7 | Counterfactual regret minimisation, trained offline by self-play |
| `src/nash.rs` | NASH | Unexploitable equilibrium ranges, push/fold when short |
| `src/monte.rs` | MONTE | Monte Carlo hand-strength estimates against pot odds |
| `src/tilt.rs` | TILT | Plays worse (looser, more aggressive) after losing |
| `src/gto_lite.rs` | GTO-LITE | Range charts, a few bet sizes, randomised close spots |
| `src/riverrat.rs` | RIVERRAT | Loose caller who chases draws to the river |
| `src/foldbot.rs` | FOLDBOT | Very tight: folds almost everything, bets strong hands |
| `src/bayes.rs` | BAYES | Learns each opponent's tendencies and exploits them |

These are hints from the names only. Every stub currently checks or calls.

## Designing a model

1. Fill in the **Design** block at the top of the model's file: idea, inputs,
   memory, parameters, expected result.
2. Replace the placeholder body of `decide`. Return an `Action`; push short
   lines to `notes` to explain the choice (they show in replays, never to
   other bots).
3. If the model learns or remembers, implement `observe`. It is called after
   every hand with the log as this seat may see it.
4. Run the checks and a match (below).

### What a bot can see: `PlayerView`

| Field | Meaning |
|---|---|
| `hole_cards` | Your two cards |
| `board` | Community cards so far |
| `street` | `Preflop`, `Flop`, `Turn`, `River` |
| `pot`, `current_bet`, `to_call` | Chips in the middle, the bet to match, what you owe |
| `legal` | `can_check`, `call_amount`, `raise_range: Option<(min, max)>` |
| `seats` | Every seat's name, stack, status, bets (no hole cards) |
| `button`, `blinds`, `hand_number` | Table position and stakes |
| `history` | Every action this hand, with think times (other bots' notes removed) |

### Helpers: `bots::kit`

They read the table and build legal actions; they don't decide anything.

- `check_or_call`, `check_or_fold`, `all_in`
- `raise_to(view, to)`, `raise_pot_fraction(view, 0.66)`: clamped into the legal range
- `pot_odds`, `stack_in_big_blinds`, `players_in_hand`, `seats_after_button`
- `actions_this_street`, `raises_this_street`
- `preflop_key(cards)`: `"AKs"`, `"T9o"`, `"QQ"`
- `engine::evaluator::evaluate(seven_cards)` ranks a made hand

`engine::analysis` computes equity with **every** hand known. It is for figures
only: a bot calling it would be cheating, and it never sees other players' cards anyway.

### Rules every model must follow

- **Legal moves only.** An illegal action is replaced by check/fold and counts
  against you. Use `view.legal` or the `kit` raise helpers.
- **Repeatable.** Use the bot's own `rng` (seeded per seat) for every random
  choice, never `rand::thread_rng()`, so a seed replays the same match.
- **Fast.** Each decision within `DECISION_BUDGET` (100 ms) in a release build.
- **No peeking.** Only `PlayerView` and the redacted logs from `observe`.

## Checking and running

```bash
cargo test -p bots                      # legality at a full table, heads-up, and with short stacks
cargo test --release -p bots            # same, plus the 100 ms time budget
cargo run --release -p runner           # 8-bot table, 10,000 hands -> logs/default-1/
cargo run --release -p runner -- configs/duel.json   # heads-up duel -> logs/duel-1/
cd viewer && npm run dev                # browse the results
```

To test one model in isolation, copy `configs/duel.json` and set the other seat
to `"bot": "check_call"`. Change `seed` to get a different deal.

## Figures and the data behind them

The runner now records everything the reference site's exhibits show.

| Figure | Data | Where |
|---|---|---|
| Live tables: hands played, hands/sec | `hands_per_second`, `elapsed_ms` | `summary.json` |
| Leaderboard: net chips, BB/100 | `standings[].net_chips`, `bb_per_100`, `rebuys` | `summary.json` |
| Net-chips line over hands | `hands[].net` per seat, per hand | `summary.json` |
| Equity bar / analysis of one hand | `equity[]` per street (set `equity_samples`) | `hands/<n>.json` |
| Message trace: bot thinking | `actions[].think_us`, `actions[].notes` | `hands/<n>.json` |
| Opening book: lines played | `actions[]` sequences across hands | `hands/<n>.json` |
| Heads-up duel, stacks reset every hand | `reset_stacks_each_hand: true` | `configs/duel.json` |

The replay page already shows think times and notes. Viewer pages for the duel
chart, the equity bar, the message trace and the opening book are still to build.

## A suggested order

Simplest first, so each one gives the next something to beat:

1. **FOLDBOT** and **RIVERRAT**: a starting-hand rule and a calling rule.
2. **TILT**: one of the above plus a mood that `observe` updates.
3. **MONTE**: simulate random opponent hands to estimate your equity, compare with `pot_odds`.
4. **GTO-LITE**: range charts from `preflop_key`, two or three bet sizes, `rng` for mixing.
5. **BAYES**: count each opponent's folds, calls and raises in `observe`; adjust.
6. **NASH**: push/fold charts for short stacks; balanced ranges otherwise.
7. **CFR-7**: needs offline training (a separate program that writes a strategy
   file the bot loads), so it is the biggest piece of work.
