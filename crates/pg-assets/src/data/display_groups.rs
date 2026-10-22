//! `assets/data/display-groups.json`: the display nodes of the five `BCLibrary
//! parseStructXml:` calls of `loadGame`, in call order (`docs/re/struct-xml.md` §3–§6, §13).
//!
//! Each call is the list of its top-level groups (node type 2) with the nodes that follow
//! each group as its children. A rect node (type 5, none in 1.48.2) is listed as a child and
//! also starts the next group, exactly like the original's missing `break`.

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/display-groups.json";

/// Deserializes the embedded `display-groups.json`.
pub fn load() -> Result<DisplayGroupsFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct DisplayGroupsFile {
    /// The five `parseStructXml:` calls, in order.
    pub parse_calls: Vec<ParseCall>,
}

/// One `parseStructXml:` call.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct ParseCall {
    /// Address of the node list (`"0x1006fe1d0"`).
    pub root: String,

    pub groups: Vec<GroupDef>,

    /// Display nodes met before the first group of the call (none in 1.48.2).
    pub orphans: Vec<NodeDef>,
}

/// The `BCStructXmlDisplayObjectNodeDef` members `parseDisplayObjectAttributes:` applies
/// (stored values; see `struct-xml.md` §5 for which ones are applied).
#[derive(Clone, Debug, Deserialize)]
pub struct DisplayAttributes {
    pub id: String,
    pub data: Option<String>,
    pub origin: [f32; 3],
    pub pos: [f32; 3],
    pub scale: [f32; 3],
    pub rot: [f32; 3],
    pub skew: [f32; 3],

    /// `0xAARRGGBB`; `None` when 0 (not applied).
    pub packed_color: Option<u32>,

    /// Stored red, green, blue, alpha.
    pub rgba: [f32; 4],

    pub hidden: bool,
    pub selectable: bool,

    /// `[colXMin, colYMin, colZMin]`.
    pub col_min: [f32; 3],

    /// `[colXMax, colYMax, colZMax]`.
    pub col_max: [f32; 3],
}

/// A top-level display group (node type 2) and the nodes attached to it.
#[derive(Clone, Debug, Deserialize)]
pub struct GroupDef {
    #[serde(flatten)]
    pub attributes: DisplayAttributes,

    /// Element text of the XML (never read by the code).
    pub element_text: Option<String>,

    pub children: Vec<NodeDef>,
}

/// A display node that is not a top-level group.
#[derive(Clone, Debug, Deserialize)]
pub struct NodeDef {
    #[serde(flatten)]
    pub attributes: DisplayAttributes,

    #[serde(flatten)]
    pub kind: NodeKind,
}

/// The node type and its own fields.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeKind {
    /// Type 1: a plain `BCDisplayObject`.
    DisplayObject,

    /// Type 3: `<emptyDisplayGroup>`, a registered group that never becomes current.
    EmptyGroup,

    /// Type 4: `<displayMarker linkId>`.
    Marker { link_id: Option<String> },

    /// Type 5: `[min_x, min_y, max_x, max_y]`.
    Rect { rect: [f32; 4] },

    /// Type 6: `<model texture>`.
    Model { texture: Option<String> },

    /// Type 7: `<multiModel>`.
    MultiModel,

    /// Type 8: `<tileModel wrapWidth wrapHeight>`.
    TileModel { wrap_width: f32, wrap_height: f32 },

    /// Type 9: `<gradient gradient width height>`.
    Gradient { gradient: Option<String>, width: f32, height: f32 },

    /// Type 11: `<text font center>` with its element text.
    Text {
        font: Option<String>,
        text: Option<String>,
        justification: TextJustification,
    },
}

/// The justification `parseStructXml:` leaves on a text (the right flag wins over center).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextJustification {
    Left,
    Right,
    Center,
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use serde_json::Value;

    use super::*;

    const ATTRIBUTE_KEYS: [&str; 13] = [
        "id",
        "data",
        "origin",
        "pos",
        "scale",
        "rot",
        "skew",
        "packed_color",
        "rgba",
        "hidden",
        "selectable",
        "col_min",
        "col_max",
    ];

    fn kind_keys(kind: &str) -> &'static [&'static str] {
        match kind {
            "display_object" | "empty_group" | "multi_model" => &[],
            "marker" => &["link_id"],
            "rect" => &["rect"],
            "model" => &["texture"],
            "tile_model" => &["wrap_width", "wrap_height"],
            "gradient" => &["gradient", "width", "height"],
            "text" => &["font", "text", "justification"],
            other => panic!("unknown node kind {other}"),
        }
    }

    fn key_set(object: &Value) -> BTreeSet<String> {
        let Value::Object(map) = object else {
            panic!("expected an object, got {object}");
        };
        map.keys().cloned().collect()
    }

    fn check_node_keys(node: &Value) {
        let Some(Value::String(kind)) = node.get("kind") else {
            panic!("node without kind: {node}");
        };
        let mut expected: BTreeSet<String> = ATTRIBUTE_KEYS.iter().map(|key| key.to_string()).collect();
        expected.insert("kind".to_owned());
        for key in kind_keys(kind) {
            expected.insert(key.to_string());
        }
        assert_eq!(key_set(node), expected, "keys of {node}");
    }

    /// The flattened types cannot deny unknown fields, so check the key sets directly.
    #[test]
    fn every_key_is_modelled() {
        let bytes = crate::file(PATH).expect("display-groups.json is embedded");
        let file: Value = serde_json::from_slice(bytes).expect("valid JSON");
        let Some(Value::Array(calls)) = file.get("parse_calls") else {
            panic!("no parse_calls");
        };

        let mut group_keys: BTreeSet<String> = ATTRIBUTE_KEYS.iter().map(|key| key.to_string()).collect();
        group_keys.insert("element_text".to_owned());
        group_keys.insert("children".to_owned());

        for call in calls {
            let Some(Value::Array(groups)) = call.get("groups") else {
                panic!("call without groups");
            };
            let Some(Value::Array(orphans)) = call.get("orphans") else {
                panic!("call without orphans");
            };
            for orphan in orphans {
                check_node_keys(orphan);
            }
            for group in groups {
                assert_eq!(key_set(group), group_keys);
                let Some(Value::Array(children)) = group.get("children") else {
                    panic!("group without children");
                };
                for child in children {
                    check_node_keys(child);
                }
            }
        }
    }

    #[test]
    fn deserializes_with_documented_counts() {
        let file = load().expect("display-groups.json matches its schema");
        let roots: Vec<&str> = file.parse_calls.iter().map(|call| call.root.as_str()).collect();
        assert_eq!(roots, ["0x10071ab98", "0x10064a270", "0x1006fe1d0", "0x1007162f0", "0x1008162c0"]);

        let group_counts: Vec<usize> = file.parse_calls.iter().map(|call| call.groups.len()).collect();
        assert_eq!(group_counts, [0, 0, 229, 37, 0]);

        let mut child_count = 0;
        let mut marker_count = 0;
        let mut linked_marker_count = 0;
        let mut group_ids = HashSet::new();
        for call in &file.parse_calls {
            assert!(call.orphans.is_empty());
            for group in &call.groups {
                group_ids.insert(group.attributes.id.as_str());
                child_count += group.children.len();
                for child in &group.children {
                    if let NodeKind::Marker { link_id } = &child.kind {
                        marker_count += 1;
                        if link_id.is_some() {
                            linked_marker_count += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(child_count, 4842 + 318);
        assert_eq!(marker_count, 148);
        assert_eq!(linked_marker_count, 89);
        assert_eq!(group_ids.len(), 266, "group ids are unique");
        assert!(group_ids.contains("PocketGod"));
        assert!(group_ids.contains("IslandDisplayGroup"));
    }

    #[test]
    fn node_kinds_and_fields_deserialize() {
        let json = r#"{"kind": "text", "id": "Label", "data": null, "origin": [0.0, 0.0, 0.0],
            "pos": [1.5, 2.0, -277.12], "scale": [1.0, 1.0, 1.0], "rot": [0.0, 0.0, 0.0],
            "skew": [0.0, 0.0, 0.0], "packed_color": 16777215, "rgba": [1.0, 0.5, 1.0, 1.0],
            "hidden": true, "selectable": false, "col_min": [0.0, 0.0, 0.0],
            "col_max": [0.0, 0.0, 0.0], "font": "Arial12", "text": "Hi", "justification": "center"}"#;
        let node: NodeDef = serde_json::from_str(json).expect("valid node");
        assert_eq!(node.attributes.id, "Label");
        assert_eq!(node.attributes.pos, [1.5, 2.0, -277.12]);
        assert_eq!(node.attributes.packed_color, Some(0x00ff_ffff));
        assert!(node.attributes.hidden);
        let NodeKind::Text { font, text, justification } = node.kind else {
            panic!("expected a text node");
        };
        assert_eq!(font.as_deref(), Some("Arial12"));
        assert_eq!(text.as_deref(), Some("Hi"));
        assert_eq!(justification, TextJustification::Center);
    }
}
