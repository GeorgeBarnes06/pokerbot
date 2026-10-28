# Pokerbot

A Rust No-Limit Texas Hold'em tournament simulator for 8-player tables, with a Next.js viewer for replaying hands from JSON logs.

```text
.
├── Cargo.toml
├── crates/
│   ├── engine/
│   ├── bots/
│   └── runner/
├── configs/
├── logs/
└── viewer/
```

Build the Rust workspace with `cargo build`. Run the viewer with `cd viewer && npm run dev`.
