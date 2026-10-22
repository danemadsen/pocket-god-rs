//! `assets/data/skin-packs.json`: the 24 skin packs with their objects, options and values,
//! the stand-alone news-icon value and the shared remap / animation-alias lists
//! (`docs/re/textures.md` §7, §9.2).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/skin-packs.json";

/// Deserializes the embedded `skin-packs.json`.
pub fn load() -> Result<SkinPacksFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SkinPacksFile {
    pub format_version: u32,

    /// Table order of `SkinPackManager::initSkinPackDefs` (0x10037d620).
    pub packs: Vec<SkinPack>,

    pub standalone_values: Vec<StandaloneValue>,

    /// `{target, source}` lists referenced by [`SkinValue::remap_set`].
    pub remap_sets: Vec<Vec<Remap>>,

    /// Alias lists referenced by [`SkinValue::anim_alias_set`].
    pub anim_alias_sets: Vec<Vec<AnimAlias>>,
}

/// `BCSkinPackDef`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SkinPack {
    pub id: String,
    pub address: String,

    /// Display name (+0x08).
    pub name: String,

    /// Purchase panel lines 0–3 (+0x10–0x28).
    pub buy_text: [String; 4],

    /// Success panel line (+0x30).
    pub success_text: String,

    /// App Store product id (+0x38).
    pub product_id: String,

    /// `product_id == "FreeBee"`.
    pub free: bool,

    /// Keychain key of the pack's purchase secret.
    pub keychain_key: String,

    pub objects: Vec<SkinObject>,
}

/// `BCSkinObjectDef`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SkinObject {
    /// Id and display name (`"Tyrannosaurus Rex"`).
    pub id: String,

    /// Behavior set on the customize preview logic.
    pub behavior: String,

    pub options: Vec<SkinOption>,
}

/// `BCSkinOptionDef`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SkinOption {
    /// Id, also the keychain key holding the selected value index.
    pub id: String,

    pub activate_at_init: bool,

    /// `MonsterAtlas`, `SharkAtlas`, `ExtrasAtlas` or `None`.
    pub channel: Option<String>,

    pub values: Vec<SkinValue>,
}

/// `BCSkinOptionValueDef` (applied by `FUN_100038f24`).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SkinValue {
    /// Id and display name.
    pub id: String,

    /// Pack that must be purchased (`None` = default value or the pack's own item).
    pub required_pack: Option<String>,

    /// Behavior for the customize preview.
    pub behavior: Option<String>,

    /// `+0x18`: `.raw` loaded into [`SkinValue::atlas`].
    pub atlas_image: Option<String>,

    /// `+0x20`: the slot [`SkinValue::atlas_image`] goes into.
    pub atlas: Option<String>,

    /// `+0x28`: index into [`SkinPacksFile::remap_sets`].
    pub remap_set: Option<usize>,

    /// `+0x30`: bits image; its regions are in `textures.json`.
    pub bits_image: Option<String>,

    /// `+0x48`: whole images loaded into slots.
    pub atlas_images: Vec<AtlasImage>,

    /// `+0x50`: index into [`SkinPacksFile::anim_alias_sets`].
    pub anim_alias_set: Option<usize>,
}

/// One `{file, format, slotDef}` record of [`SkinValue::atlas_images`].
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct AtlasImage {
    pub image: String,
    pub format_code: i32,
    pub atlas: String,
}

/// One `{target, source}` texture-def pair of a remap list.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Remap {
    pub target: String,
    pub source: String,
}

/// One `{alias, animation, displayObject, behavior, displayGroup}` record; `""` = none.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct AnimAlias {
    pub alias: String,
    pub animation: String,
    pub display_object: String,
    pub behavior: String,
    pub display_group: String,
}

/// A value used outside the packs (the news icon).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct StandaloneValue {
    pub address: String,
    pub used_by: String,
    pub value: SkinValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_counts() {
        let file = load().expect("skin-packs.json matches its schema");
        assert_eq!(file.packs.len(), 24);

        let mut object_count = 0;
        let mut option_count = 0;
        let mut value_count = 0;
        for pack in &file.packs {
            object_count += pack.objects.len();
            for object in &pack.objects {
                option_count += object.options.len();
                for option in &object.options {
                    value_count += option.values.len();
                }
            }
        }
        assert_eq!(object_count, 290);
        assert_eq!(option_count, 294);
        assert_eq!(value_count, 5149);
        assert_eq!(file.remap_sets.len(), 42);
        assert_eq!(file.anim_alias_sets.len(), 132);
        assert_eq!(file.standalone_values.len(), 1);
    }

    #[test]
    fn set_indices_are_in_range() {
        let file = load().expect("skin-packs.json matches its schema");
        for pack in &file.packs {
            for object in &pack.objects {
                for option in &object.options {
                    for value in &option.values {
                        if let Some(index) = value.remap_set {
                            assert!(index < file.remap_sets.len(), "{}", value.id);
                        }
                        if let Some(index) = value.anim_alias_set {
                            assert!(index < file.anim_alias_sets.len(), "{}", value.id);
                        }
                    }
                }
            }
        }
    }
}
