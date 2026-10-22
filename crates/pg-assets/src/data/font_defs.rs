//! `assets/data/font-defs.json`: the `<addFontToAtlas>` nodes of `parseStructXml:` call 2
//! (root 0x10064a270) (`docs/re/struct-xml.md` §4.2, §13).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/font-defs.json";

/// Deserializes the embedded `font-defs.json`.
pub fn load() -> Result<FontDefsFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct FontDefsFile {
    pub fonts: Vec<FontDef>,
}

/// One font node (type 12). Glyph `i` of `char_set` is the texture def
/// `"<id>:FullSet:<i>"`; folder, file name and extension only fed the offline atlas builder.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct FontDef {
    pub id: String,
    pub folder: String,
    pub file_name: String,
    pub ext: String,
    pub char_set: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_fonts() {
        let file = load().expect("font-defs.json matches its schema");
        let ids: Vec<&str> = file.fonts.iter().map(|font| font.id.as_str()).collect();
        assert_eq!(ids, ["Arial12Black", "Arial12", "AnimatedGothicLite20", "Doop24Green"]);
        assert_eq!(file.fonts[0].char_set.len(), 95);
        assert_eq!(file.fonts[3].char_set, " L0123456789");
    }
}
