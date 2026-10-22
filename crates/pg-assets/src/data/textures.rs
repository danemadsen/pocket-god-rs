//! `assets/data/textures.json`: the 26 atlas slots, the 8337 texture definitions, the 530
//! images and the 19 bits slots (`docs/re/textures.md` §9.1). Lists keep the binary order:
//! the atlases and textures of the two `processAtlasTextureDefs:textureDefs:` lists of
//! `loadGame` are concatenated (16 slots and 4674 defs from the first list, then 10 slots
//! and 3663 defs from the second).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/textures.json";

/// Deserializes the embedded `textures.json`.
pub fn load() -> Result<TexturesFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct TexturesFile {
    pub format_version: u32,

    /// Always true: every image holds premultiplied alpha.
    pub premultiplied_alpha: bool,

    pub atlases: Vec<Atlas>,
    pub textures: Vec<TextureDef>,
    pub images: Vec<RawImage>,
    pub bits_slots: Vec<BitsSlot>,
}

/// `BCAtlasTextureDef.format` (+0x28) by its `docs/re/textures.md` §2.1 name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextureFormat {
    Png,
    Rgb888,
    Rgb565,
    Rgba8888,
    Rgba4444,
    Rgba5551,
    L8,
    La88,
    Pvrtc4,
    Pvrtc2,
}

/// `BCAtlasTextureDef`: one GL texture slot.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Atlas {
    /// The id of the slot's own texture def (+0x48), the registry key.
    pub id: String,

    /// Address of the def in the original executable.
    pub address: String,

    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub format_code: i32,

    /// The BOOL of the slot's own texture def.
    pub has_alpha: bool,

    /// Build-time channel (`MainAtlas`, `MonsterAtlas`, …); `None` for single slots.
    pub channel: Option<String>,

    /// Page number of the slot within its channel.
    pub page: u32,

    /// `+0x00`: image uploaded when the slot is created; `None` = created empty.
    pub init_image: Option<String>,

    /// `+0x20`: resource extension (`raw`).
    pub ext: Option<String>,

    /// `+0x10`: alternative image (never used).
    pub alt_image: Option<String>,

    /// `+0x2c`: format of the alternative image.
    pub alt_format_code: i32,

    /// `+0x30 == 1`: the file is zlib-compressed.
    pub zlib: bool,

    /// `+0x34`: read from `Library/Caches` instead of the bundle.
    pub from_caches_dir: bool,

    /// `+0x35`: skip slot creation.
    pub no_init: bool,

    /// `+0x36`: mipmapped minification.
    pub mipmap: bool,

    /// Every atlas-kind image that can be loaded into the slot.
    pub images: Vec<String>,
}

/// `BCTextureDef`: one sprite.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct TextureDef {
    pub id: String,

    /// Id of the slot the def samples.
    pub atlas: String,

    /// Image the def samples by default (derived by the tool, §9.3).
    pub image: String,

    /// `+0x08`: blend even at alpha 1.
    pub has_alpha: bool,

    /// Left of the texel rect (top-left origin).
    pub x: i32,

    /// Top of the texel rect.
    pub y: i32,

    /// `+0x20`: trimmed width (whole texels).
    pub width: i32,

    /// `+0x24`: trimmed height.
    pub height: i32,

    /// `+0x28`: untrimmed frame width (also the glyph advance).
    pub original_width: i32,

    /// `+0x2c`: untrimmed frame height.
    pub original_height: i32,

    /// Zero-area def from a fully transparent source frame.
    pub empty: bool,

    /// `+0x30`: (u, v) of top-left, top-right, bottom-left, bottom-right.
    pub uvs: [f32; 8],

    /// `+0x50`: (x, y, z) of the same corners, pixels, y up.
    pub vertices: [f32; 12],
}

/// How an image file is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageKind {
    /// A whole slot image (`glTexImage2D`).
    Atlas,

    /// Concatenated sub-images patched into slots (`glTexSubImage2D`).
    Bits,
}

/// One converted `.raw` file (`assets/textures/<name>.png`).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct RawImage {
    pub name: String,

    /// Path of the PNG relative to `assets/`.
    pub png: String,

    pub raw_size: u64,
    pub kind: ImageKind,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,

    /// Atlas images: the slots they are loaded into; bits images: the slots they patch.
    pub atlases: Vec<String>,

    /// Sub-images of a bits image in file order (empty for atlas images).
    pub regions: Vec<ImageRegion>,

    /// Human-readable call sites, sequences and skin values that load the image.
    pub loaded_by: Vec<String>,

    /// False for the files 1.48.2 never loads (§2.4).
    pub referenced: bool,
}

/// A sub-image of a bits image (a rect in the PNG) and where it goes.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct ImageRegion {
    pub src_x: u32,
    pub src_y: u32,
    pub width: u32,
    pub height: u32,

    /// Destination def; `None` = chosen at run time (episode icons and idols).
    pub texture: Option<String>,

    /// Destination slot; `None` = chosen at run time.
    pub atlas: Option<String>,

    /// Destination texel x; `None` = chosen at run time.
    pub x: Option<i32>,

    /// Destination texel y; `None` = chosen at run time.
    pub y: Option<i32>,
}

/// A static `BCSkinTextureBitsDef` array filled at run time (episode icon / idol slots).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct BitsSlot {
    pub name: String,
    pub address: String,
    pub used_by: String,
    pub regions: Vec<BitsSlotRegion>,
}

/// `BCSkinTextureBitsDef`: a destination rect (texels) in a def's slot.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct BitsSlotRegion {
    pub texture: String,
    pub atlas: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_counts() {
        let file = load().expect("textures.json matches its schema");
        assert_eq!(file.format_version, 1);
        assert!(file.premultiplied_alpha);
        assert_eq!(file.atlases.len(), 26);
        assert_eq!(file.textures.len(), 8337);
        assert_eq!(file.images.len(), 530);
        assert_eq!(file.bits_slots.len(), 19);
    }

    #[test]
    fn every_slot_has_its_own_def_and_every_def_a_known_slot() {
        let file = load().expect("textures.json matches its schema");
        for atlas in &file.atlases {
            let own_def = file.textures.iter().find(|texture| texture.id == atlas.id);
            assert!(own_def.is_some(), "slot {} has no own def", atlas.id);
        }
        for texture in &file.textures {
            let known = file.atlases.iter().any(|atlas| atlas.id == texture.atlas);
            assert!(known, "{} samples unknown slot {}", texture.id, texture.atlas);
        }
    }

    #[test]
    fn format_names_match_codes() {
        let file = load().expect("textures.json matches its schema");
        for atlas in &file.atlases {
            let expected = match atlas.format_code {
                5 => TextureFormat::Rgba4444,
                9 => TextureFormat::Pvrtc4,
                other => panic!("{}: unexpected format code {other}", atlas.id),
            };
            assert_eq!(atlas.format, expected, "{}", atlas.id);
        }
    }

    #[test]
    fn every_png_is_embedded() {
        let file = load().expect("textures.json matches its schema");
        for image in &file.images {
            assert!(crate::file(&image.png).is_some(), "{} is not embedded", image.png);
        }
    }
}
