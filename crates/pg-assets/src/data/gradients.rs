//! `assets/data/gradients.json`: the 15 gradient defs parsed like
//! `+[BCGradient createGradientDef:]` (`docs/re/audio-prefs-text.md` §4). Same content as
//! `gradient-defs.json`, with a format version.

use serde::Deserialize;

use super::DataError;
use super::gradient_defs::GradientDef;

pub const PATH: &str = "data/gradients.json";

/// Deserializes the embedded `gradients.json`.
pub fn load() -> Result<GradientTable, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct GradientTable {
    pub format_version: u32,

    /// Stops in spec order.
    pub gradients: Vec<GradientDef>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::gradient_defs;

    #[test]
    fn agrees_with_the_struct_xml_gradient_defs() {
        let table = load().expect("gradients.json matches its schema");
        let defs = gradient_defs::load().expect("gradient-defs.json matches its schema");
        assert_eq!(table.gradients.len(), 15);

        for (gradient, def) in table.gradients.iter().zip(&defs.gradients) {
            assert_eq!(gradient.id, def.id);
            assert_eq!(gradient.spec, def.spec);
            for (stop, def_stop) in gradient.stops.iter().zip(&def.stops) {
                assert_eq!(stop.position.to_bits(), def_stop.position.to_bits(), "{}", gradient.id);
                assert_eq!(stop.argb, def_stop.argb, "{}", gradient.id);
                assert_eq!(stop.rgba, def_stop.rgba, "{}", gradient.id);
            }
        }
    }
}
