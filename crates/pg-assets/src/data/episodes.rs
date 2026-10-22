//! `assets/data/episodes.json`: the 49 episodes / god idols of list 0x1007306f8 in list
//! order (Episode1 … Episode31, Episode31B, Episode32 … Episode48) with their enemy idols,
//! realm powers, challenges and level-ups (`docs/re/struct-xml.md` §8, §13).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/episodes.json";

/// Deserializes the embedded `episodes.json`.
pub fn load() -> Result<EpisodesFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct EpisodesFile {
    pub episodes: Vec<Episode>,
}

/// `realmName:` values (episode +0x70); the discriminant is the stored int.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Realm {
    Oceanus = 0,
    Aeris = 1,
    Infernis = 2,
    Animus = 3,
    Pax = 4,
}

/// `attackName:` values (episode +0x74).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Attack {
    Strong = 0,
    Counter = 1,
    Fast = 2,
}

/// Episode +0x78: picks the `PygmyBattleAttack*` or `PygmyBattleEnergyAttack*` behaviours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackStyle {
    Physical = 0,
    Energy = 1,
}

/// `realmPowerName:` values (realm power +0x04).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealmPower {
    RealmDamage = 0,
    TieDamage = 1,
    TotalBoostDamage = 2,
    AttackBoostDamage = 3,
    AttackReflectDamage = 4,
    AttackDrainHealth = 5,
    AttackDefense = 6,
    TotalDefense = 7,
    IgnoreAttackDefense = 8,
    RealmHeal = 9,
    FreezeIdol = 10,
    ResurrectAfterDeath = 11,
    PowerUpAllIdols = 12,
    DrainLevelPoints = 13,
    CancelEnemyPowerups = 14,
    UnfreezeIdols = 15,
}

/// `BCStructXmlEpisodeNodeDef` (type 32), without its runtime members.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Episode {
    /// `Episode1`…; key of the episode's preferences and of `ActiveEpisode`.
    pub id: String,

    /// `"1"` … `"31B"`.
    pub episode_number: String,

    pub idol_id: String,
    pub idol_name: String,
    pub idol_title: String,

    /// Summary text with `[PrimaryValue]`, `[SecondaryValue]` and `[TurnCount]` markers.
    pub power_description: String,

    pub power_short_description: String,

    /// Raw image loaded into an icon bits slot.
    pub icon_texture: String,

    /// Raw image of the idol.
    pub idol_texture: String,

    pub realm: Realm,
    pub attack: Attack,
    pub attack_style: AttackStyle,
    pub episode_title: String,

    /// Runtime selection slot 0–3; −1 (not selected) for every episode.
    pub initial_selection_slot: i32,

    pub damage_per_level: f32,
    pub health_per_level: f32,
    pub power_turn_count: i32,
    pub enemy_idols: Vec<EnemyIdol>,
    pub realm_powers: Vec<RealmPowerDef>,
    pub challenges: Vec<Challenge>,
    pub level_ups: Vec<LevelUp>,
}

/// Enemy idol node (type 20).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct EnemyIdol {
    /// Episode whose idol the opponent uses.
    pub episode_id: String,

    /// That idol's level in this battle (0–19).
    pub level: i32,
}

/// Realm power node (type 21): value = base + per level × idol level.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct RealmPowerDef {
    pub power: RealmPower,
    pub base_value: f32,
    pub value_per_level: f32,
}

/// Challenge node (type 22), without its runtime members.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Challenge {
    pub event_id: String,
    pub sacrifice_id: Option<String>,
    pub gift_id: Option<String>,

    /// Apocalypse gem 1–7 or 0.
    pub gem_id: i32,

    pub description: String,
    pub hint: String,
}

/// Level-up node (type 23), without its runtime members.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct LevelUp {
    pub event_id: String,
    pub sacrifice_id: Option<String>,
    pub gift_id: Option<String>,
    pub description: String,

    /// Points for level 19 (the cap).
    pub max_points: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_in_episode_order() {
        let file = load().expect("episodes.json matches its schema");
        assert_eq!(file.episodes.len(), 49);
        assert_eq!(file.episodes[0].id, "Episode1");
        assert_eq!(file.episodes[31].id, "Episode31B");
        assert_eq!(file.episodes[48].id, "Episode48");

        for episode in &file.episodes {
            assert_eq!(episode.enemy_idols.len(), 4, "{}", episode.id);
            assert_eq!(episode.initial_selection_slot, -1, "{}", episode.id);
        }
    }

    #[test]
    fn enum_discriminants_are_the_stored_values() {
        assert_eq!(Realm::Pax as i32, 4);
        assert_eq!(Attack::Fast as i32, 2);
        assert_eq!(AttackStyle::Energy as i32, 1);
        assert_eq!(RealmPower::UnfreezeIdols as i32, 15);
    }
}
