//! `assets/data/sequence-methods.json`: per sequence-item method, the positional parameter
//! names and every handler implementation, sorted by method (`docs/re/sequences.md` §2.2).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/sequence-methods.json";

/// Deserializes the embedded `sequence-methods.json`.
pub fn load() -> Result<Vec<SequenceMethod>, DataError> {
    super::load(PATH)
}

/// One method used by the data or implemented by a handler.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct SequenceMethod {
    pub method: String,

    /// `method + ":::"`.
    pub selector: String,

    /// Parameters in positional order (empty for methods without parameters).
    pub params: Vec<MethodParam>,

    /// Length of the generated parameter arrays (1 for methods without parameters); `None`
    /// for selectors no sequence uses.
    pub param_slots: Option<u32>,

    pub implementations: Vec<Implementation>,

    /// Number of items using the method.
    pub uses: u32,

    /// Number of sequences using it.
    pub sequence_count: u32,
}

/// How a positional parameter's name was recovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParamSource {
    /// Matched against the values of the 1.45.3 `sequence.xml`.
    Xml,

    /// The 1.45.3 attribute exists; the position comes from the handler code.
    XmlName,

    /// New in 1.48.2; named from what the handler does.
    Inferred,
}

/// One positional parameter.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct MethodParam {
    pub index: usize,
    pub name: String,
    pub source: ParamSource,
}

/// One `<method>:::` implementation.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Implementation {
    pub class: String,
    pub category: Option<String>,
    pub symbol: String,
    pub address: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_counts() {
        let methods = load().expect("sequence-methods.json matches its schema");
        assert_eq!(methods.len(), 533);

        for method in &methods {
            let expected_selector = format!("{}:::", method.method);
            assert_eq!(method.selector, expected_selector);
            for (position, param) in method.params.iter().enumerate() {
                assert_eq!(param.index, position, "{}", method.method);
            }
        }
    }
}
