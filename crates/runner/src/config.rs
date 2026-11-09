//! Tournament configuration, loaded from JSON.

use bots::{Bot, CheckCall};
use engine::state::{Blinds, Chips};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub name: String,
    /// Which strategy plays this seat, e.g. `"check_call"`.
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
        for player in &self.players {
            make_bot(&player.bot)?;
        }
        Ok(())
    }

    /// The blinds in force for hand number `hand_number` (1-based).
    pub fn blinds_for_hand(&self, hand_number: u64) -> Blinds {
        let level = ((hand_number - 1) / self.hands_per_level) as usize;
        self.blind_levels[level.min(self.blind_levels.len() - 1)]
    }
}

/// Builds the bot named in a config file.
pub fn make_bot(kind: &str) -> Result<Box<dyn Bot>, String> {
    match kind {
        "check_call" => Ok(Box::new(CheckCall)),
        other => Err(format!("unknown bot {other:?}")),
    }
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
