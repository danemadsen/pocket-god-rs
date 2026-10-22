//! `assets/data/logic-objects.json`: the 632 entries of the `getLogic:` table built by
//! `BCLibrary(Logic) -createLogicClasses` and wired by `-initLogicListeners`, ordered by
//! index (`docs/re/logic.md` §13).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/logic-objects.json";

/// Deserializes the embedded `logic-objects.json`.
pub fn load() -> Result<Vec<LogicObjectDef>, DataError> {
    super::load(PATH)
}

/// One table entry.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct LogicObjectDef {
    /// `getLogic:` index = creation order.
    pub index: usize,

    /// Slot address, `0x1007e3030 + 8 · index`.
    pub address: String,

    /// ObjC class of the logic object.
    pub class: String,

    /// `"initWithDisplayObject:"` or `"initWithDisplayObject:displayMarkerId:"`.
    pub init_selector: String,

    /// Second argument of the latter (`"MidOceanMarker"`).
    pub display_marker_id: Option<String>,

    /// The display object given to the initializer.
    pub binding: DisplayRef,

    /// Steps run right after creation, in machine-code order.
    pub init_steps: Vec<InitStep>,

    /// `set…Listener:` calls of `initLogicListeners`, in order.
    pub listeners: Vec<ListenerDef>,

    /// `setTweenable:channelId:` calls on this entry's animation, in order.
    pub tweenables: Vec<TweenableDef>,

    /// Informational: functions calling `getLogic:` with this constant index.
    pub references: Vec<CodeReference>,
}

/// Where a display object comes from.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DisplayRef {
    /// `[[class alloc] initWithId:id]` (always `BCMultiModel`).
    NewModel { class: String, id: String },

    /// `[library getDisplayGroupById:group]`.
    DisplayGroup { group: String },

    /// `[[library getDisplayGroupById:group] getDisplayObject:object]`.
    GroupObject { group: String, object: String },

    /// The `displayObject` of table entry `index` (tweenables only).
    LogicDisplayObject { index: usize },
}

/// One initialization step; `None` coordinates are not written.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum InitStep {
    SetPos { x: Option<f32>, y: Option<f32>, z: Option<f32> },
    SetScale { x: Option<f32>, y: Option<f32>, z: Option<f32> },
    PlayAnimation { animation_id: String },
    SetBehavior { sequence_id: String },
    SetIslandLocationState { value: i32 },
    SetApocalypseDoorType { value: i32 },
    SetAlwaysOpen { value: bool },
    SetApocalypseGemType { value: i32 },
    SetApocalypseDiseaseType { value: i32 },
    SetApocalypseCloudStyleId { value: String },
}

/// One `set<Name>LogicListener:` call.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct ListenerDef {
    pub selector: String,
    pub target: ListenerTarget,
}

/// The listener object.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ListenerTarget {
    /// `[class instance]`; `index` is its table entry, `None` for non-logic singletons.
    Singleton { class: String, index: Option<usize> },
}

/// One `setTweenable:channelId:` call.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct TweenableDef {
    pub channel_id: i32,
    pub target: DisplayRef,
}

/// A function that calls `getLogic:` with this entry's index.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct CodeReference {
    pub function: String,
    pub address: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_in_index_order() {
        let entries = load().expect("logic-objects.json matches its schema");
        assert_eq!(entries.len(), 632);
        for (position, entry) in entries.iter().enumerate() {
            assert_eq!(entry.index, position);
        }

        let listener_count: usize = entries.iter().map(|entry| entry.listeners.len()).sum();
        assert_eq!(listener_count, 304);

        let tweenable_count: usize = entries.iter().map(|entry| entry.tweenables.len()).sum();
        assert_eq!(tweenable_count, 261);
    }

    #[test]
    fn island_object_logic_example() {
        let entries = load().expect("logic-objects.json matches its schema");
        let entry = &entries[49];
        assert_eq!(entry.class, "IslandObjectLogic");
        assert_eq!(entry.display_marker_id.as_deref(), Some("MidOceanMarker"));

        let DisplayRef::NewModel { class, id } = &entry.binding else {
            panic!("entry 49 binds a new model");
        };
        assert_eq!(class, "BCMultiModel");
        assert_eq!(id, "MisfitOceanRock");

        let InitStep::SetPos { x, y, z } = &entry.init_steps[1] else {
            panic!("second step of entry 49 is set_pos");
        };
        assert_eq!((*x, *y, *z), (Some(400.0), Some(98.0), Some(-1200.0)));
    }
}
