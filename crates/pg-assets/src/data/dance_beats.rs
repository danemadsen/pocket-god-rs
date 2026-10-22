//! `assets/data/dance-beats.json`: the Dance Dance Execution charts of list 0x10074f818
//! (types 35, 34, 33) (`docs/re/struct-xml.md` §9, §13).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/dance-beats.json";

/// Deserializes the embedded `dance-beats.json`.
pub fn load() -> Result<DanceBeatsFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct DanceBeatsFile {
    pub groups: Vec<DanceGroup>,
}

/// Group node (type 35).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct DanceGroup {
    /// Pick a random segment each time instead of the first (and keep the group index).
    pub random_segment: bool,

    pub segments: Vec<DanceSegment>,
}

/// Segment node (type 34).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct DanceSegment {
    pub beats: Vec<DanceBeat>,
}

/// `BCStructXmlDanceDanceBeatNodeDef` (type 33), without its runtime members.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct DanceBeat {
    /// Duration in beats (1–4).
    pub duration: i32,

    pub left: bool,
    pub down: bool,
    pub up: bool,
    pub right: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_counts() {
        let file = load().expect("dance-beats.json matches its schema");
        assert_eq!(file.groups.len(), 80);

        let mut segment_count = 0;
        let mut beat_count = 0;
        for group in &file.groups {
            segment_count += group.segments.len();
            for segment in &group.segments {
                beat_count += segment.beats.len();
            }
        }
        assert_eq!(segment_count, 87);
        assert_eq!(beat_count, 1684);
        assert!(file.groups[79].random_segment);
        assert_eq!(file.groups[79].segments.len(), 8);
    }
}
