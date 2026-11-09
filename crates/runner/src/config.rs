//! Tournament configuration, loaded from JSON.

use bots::Bot;
use engine::state::{Blinds, Chips};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub name: String,
    /// Which model plays this seat: one of `bots::MODELS` (e.g. `"cfr7"`) or `"check_call"`.
    pub bot: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TournamentConfig {
    /// Seeds every shuffle, so the same config replays the same tournament.
    pub seed: u64,
    pub starting_stack: Chips,
    /// Hands played at each blind level before moving to the next.
    pub hands_per_level: u64,
    /// Blind levels in order; the last one repeats once reached.
    pub blind_levels: Vec<Blinds>,
    /// Stop after this many hands. With rebuys this is the match length; without,
    /// a safety limit in case nobody wins.
    pub max_hands: u64,
    /// Cash-game format: a player with less than one big blind at the start of a
    /// hand is topped back up to `starting_stack`, so nobody is ever eliminated.
    /// Without rebuys, players bust out until one has every chip.
    #[serde(default)]
    pub rebuy: bool,
    /// Heads-up duel format: every stack goes back to `starting_stack` before each
    /// hand, so each hand is scored on its own.
    #[serde(default)]
    pub reset_stacks_each_hand: bool,
    /// Random run-outs per street for the equity figures (0 = don't record).
    /// Costs time: a few hundred is plenty for heads-up; leave at 0 for long 8-bot runs.
    #[serde(default)]
    pub equity_samples: u32,
    /// One entry per seat, in seat order.
    pub players: Vec<PlayerConfig>,
}

impl TournamentConfig {
    pub fn from_json(text: &str) -> Result<Self, String> {
        let config: Self = serde_json::from_str(text).map_err(|e| format!("invalid config: {e}"))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(2..=8).contains(&self.players.len()) {
            return Err(format!("need 2 to 8 players, got {}", self.players.len()));
        }
        if self.blind_levels.is_empty() {
            return Err("need at least one blind level".into());
        }
        if self.blind_levels.iter().any(|b| b.big == 0 || b.small > b.big) {
            return Err("each blind level needs 0 < small <= big".into());
        }
        if self.hands_per_level == 0 || self.max_hands == 0 || self.starting_stack == 0 {
            return Err("hands_per_level, max_hands and starting_stack must be positive".into());
        }
        for (seat, player) in self.players.iter().enumerate() {
            make_bot(&player.bot, self.bot_seed(seat))?;
        }
        Ok(())
    }

    /// Scored by chips won rather than finishing place.
    pub fn is_cash(&self) -> bool {
        self.rebuy || self.reset_stacks_each_hand
    }

    /// Each seat's bot gets its own seed, derived from the match seed.
    pub fn bot_seed(&self, seat: usize) -> u64 {
        self.seed
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(seat as u64 + 1)
    }

    /// The blinds in force for hand number `hand_number` (1-based).
    pub fn blinds_for_hand(&self, hand_number: u64) -> Blinds {
        let level = ((hand_number - 1) / self.hands_per_level) as usize;
        self.blind_levels[level.min(self.blind_levels.len() - 1)]
    }
}

/// Builds the bot named in a config file.
pub fn make_bot(kind: &str, seed: u64) -> Result<Box<dyn Bot>, String> {
    bots::create(kind, seed).ok_or_else(|| {
        let known: Vec<&str> = bots::MODELS.iter().map(|(name, _)| *name).collect();
        format!("unknown bot {kind:?} (known: {}, check_call)", known.join(", "))
    })
}

#[cfg(test)]
mod tests {
    use super::TournamentConfig;

    const CONFIG: &str = r#"{
        "seed": 1,
        "starting_stack": 1000,
        "hands_per_level": 10,
        "blind_levels": [
            { "small": 5, "big": 10, "ante": 0 },
            { "small": 10, "big": 20, "ante": 2 }
        ],
        "max_hands": 500,
        "players": [
            { "name": "A", "bot": "check_call" },
            { "name": "B", "bot": "check_call" }
        ]
    }"#;

    #[test]
    fn parses_valid_config() {
        let config = TournamentConfig::from_json(CONFIG).unwrap();

        assert_eq!(config.players.len(), 2);
        assert_eq!(config.blind_levels[1].ante, 2);
    }

    #[test]
    fn blinds_rise_each_level_then_stay_at_the_last() {
        let config = TournamentConfig::from_json(CONFIG).unwrap();

        assert_eq!(config.blinds_for_hand(1).big, 10);
        assert_eq!(config.blinds_for_hand(10).big, 10);
        assert_eq!(config.blinds_for_hand(11).big, 20);
        assert_eq!(config.blinds_for_hand(400).big, 20);
    }

    #[test]
    fn rejects_unknown_bot() {
        let text = CONFIG.replacen("check_call", "genius", 1);
        let error = TournamentConfig::from_json(&text).unwrap_err();

        assert!(error.contains("genius"), "{error}");
    }

    #[test]
    fn rejects_too_few_players() {
        let mut config = TournamentConfig::from_json(CONFIG).unwrap();
        config.players.pop();

        assert!(config.validate().is_err());
    }
}
