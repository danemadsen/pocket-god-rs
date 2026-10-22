//! `assets/data/gradient-defs.json`: the `<gradientDef>` nodes of `parseStructXml:` call 2
//! (root 0x10064a270) (`docs/re/struct-xml.md` §4.3, §13).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/gradient-defs.json";

/// Deserializes the embedded `gradient-defs.json`.
pub fn load() -> Result<GradientDefsFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct GradientDefsFile {
    pub gradients: Vec<GradientDef>,
}

/// One gradient def node (type 10).
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct GradientDef {
    pub id: String,

    /// The `gradient` attribute: comma-separated `"<pos>:<0xAARRGGBB>"` stops, parsed at load
    /// by `+[BCGradient createGradientDef:]`.
    pub spec: String,

    /// The stops as the extractor's emulation of `createGradientDef:` computed them.
    pub stops: Vec<GradientStop>,
}

/// One `BCGradientColor`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct GradientStop {
    pub position: f32,
    pub argb: u32,

    /// Red, green, blue, alpha.
    pub rgba: [f32; 4],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_counts() {
        let file = load().expect("gradient-defs.json matches its schema");
        assert_eq!(file.gradients.len(), 15);

        for gradient in &file.gradients {
            let component_count = gradient.spec.split(',').count();
            assert_eq!(gradient.stops.len(), component_count, "{}", gradient.id);
        }
    }
}
