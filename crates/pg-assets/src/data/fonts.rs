//! `assets/data/fonts.json`: the four bitmap fonts with the metrics of their 297 glyph
//! texture defs (`docs/re/audio-prefs-text.md` §3.2). Same fonts as `font-defs.json`; the
//! glyph values are those of `textures.json` under the glyph texture ids.

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/fonts.json";

/// Deserializes the embedded `fonts.json`.
pub fn load() -> Result<FontTable, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct FontTable {
    pub format_version: u32,
    pub fonts: Vec<Font>,
}

/// One `BCFontDef`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Font {
    pub id: String,
    pub folder: String,
    pub file_name: String,
    pub ext: String,

    /// Exactly as stored; the index of a character is its glyph number.
    pub char_set: String,

    pub glyphs: Vec<Glyph>,
}

/// One glyph: the texture def `"<font>:FullSet:<index>"`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Glyph {
    pub index: u32,
    pub character: String,
    pub byte: u8,
    pub texture_id: String,

    /// Slot id (`atlasTest0`…).
    pub atlas: String,

    /// Image of the slot (`TextureAtlasTest0`…).
    pub atlas_image: String,

    /// `BCTextureDef` +0x28.
    pub advance: f32,

    /// `BCTextureDef` +0x2c (line advance).
    pub height: f32,

    /// `BCTextureDef` +0x20 (0 for space).
    pub quad_width: f32,

    /// `BCTextureDef` +0x24.
    pub quad_height: f32,

    /// 4 × (u, v) in the vertex order.
    pub uvs: [f32; 8],

    /// 4 × (x, y, z): top-left, top-right, bottom-left, bottom-right; y up.
    pub vertices: [f32; 12],
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::data::{font_defs, textures};

    #[test]
    fn glyphs_match_the_texture_defs() {
        let table = load().expect("fonts.json matches its schema");
        let defs = font_defs::load().expect("font-defs.json matches its schema");
        let texture_file = textures::load().expect("textures.json matches its schema");

        let mut texture_defs = HashMap::new();
        for texture in &texture_file.textures {
            texture_defs.insert(texture.id.as_str(), texture);
        }

        let mut glyph_count = 0;
        for (font, def) in table.fonts.iter().zip(&defs.fonts) {
            assert_eq!(font.id, def.id);
            assert_eq!(font.char_set, def.char_set);
            assert_eq!(font.glyphs.len(), font.char_set.len());
            for glyph in &font.glyphs {
                let texture = texture_defs.get(glyph.texture_id.as_str()).expect("glyph def exists");
                assert_eq!(glyph.advance, texture.original_width as f32, "{}", glyph.texture_id);
                assert_eq!(glyph.uvs, texture.uvs, "{}", glyph.texture_id);
                assert_eq!(glyph.vertices, texture.vertices, "{}", glyph.texture_id);
                glyph_count += 1;
            }
        }
        assert_eq!(glyph_count, 297);
    }
}
