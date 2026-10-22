//! `assets/data/konkey-dong-levels.json`: the Konkey Dong levels of list 0x100649c68 in list
//! order (= `mKonkeyDongLevelArray` index) (`docs/re/struct-xml.md` §10, §13).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/konkey-dong-levels.json";

/// Deserializes the embedded `konkey-dong-levels.json`.
pub fn load() -> Result<KonkeyDongLevelsFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct KonkeyDongLevelsFile {
    pub levels: Vec<KonkeyDongLevel>,
}

/// Level node (type 13) with the nodes (types 14–19) that follow it.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct KonkeyDongLevel {
    /// Not read by the game (levels are indexed by list order).
    pub id: String,

    pub display_group: String,

    /// Object of the group wrapped in a `MultiModelLogic`.
    pub vegetation_object: String,

    pub vegetation_behavior: String,
    pub level_width: f32,
    pub level_height: f32,

    /// Kong pos node (type 14); the last one wins.
    pub kong_pos: Option<[f32; 2]>,

    /// Pygmy start node (type 15); the last one wins.
    pub start_pygmy_pos: Option<[f32; 2]>,

    pub platforms: Vec<Platform>,
    pub ladders: Vec<Ladder>,
    pub picks: Vec<Pick>,
    pub beetles: Vec<Beetle>,
}

/// Platform node (type 16): a `KonkeyDongPlatformLogic`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Platform {
    pub object: String,
    pub behavior: String,
    pub start_platform: bool,
    pub final_platform: bool,
    pub boulder_x_pos_vel: f32,
    pub flip_boulder_x_pos_vel: bool,
}

/// Ladder node (type 17): a `KonkeyDongLadderLogic`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Ladder {
    pub object: String,
    pub behavior: String,
    pub force_boulder_drop: bool,
    pub do_not_pass: bool,
}

/// Pick node (type 18): a `KonkeyDongPickLogic`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Pick {
    pub object: String,
    pub loop_lock: i32,
}

/// Beetle node (type 19): a `KonkeyDongBeetleLogic`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Beetle {
    pub object: String,
    pub loop_unlock: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_counts() {
        let file = load().expect("konkey-dong-levels.json matches its schema");
        let ids: Vec<&str> = file.levels.iter().map(|level| level.id.as_str()).collect();
        assert_eq!(ids, ["Level0", "Level1", "Level4", "Level2", "Level3"]);

        let mut platform_count = 0;
        let mut ladder_count = 0;
        let mut pick_count = 0;
        let mut beetle_count = 0;
        for level in &file.levels {
            platform_count += level.platforms.len();
            ladder_count += level.ladders.len();
            pick_count += level.picks.len();
            beetle_count += level.beetles.len();
        }
        assert_eq!((platform_count, ladder_count, pick_count, beetle_count), (48, 30, 9, 21));
    }
}
