//! `assets/data/links.json`: the store link nodes (types 38 and 37) at the start of
//! `parseStructXml:` call 4. Dead data in 1.48.2: nothing calls `openCountryURL:`
//! (`docs/re/struct-xml.md` §7, §13).

use serde::Deserialize;

use super::DataError;

pub const PATH: &str = "data/links.json";

/// Deserializes the embedded `links.json`.
pub fn load() -> Result<LinksFile, DataError> {
    super::load(PATH)
}

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct LinksFile {
    pub links: Vec<Link>,
}

/// `BCStructXmlLinkNodeDef`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Link {
    pub id: String,
    pub country_links: Vec<CountryLink>,
}

/// `BCStructXmlCountryLinkNodeDef`.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct CountryLink {
    /// ISO country code (`US`, `AE`, …).
    pub country: String,

    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_with_documented_counts() {
        let file = load().expect("links.json matches its schema");
        let ids: Vec<&str> = file.links.iter().map(|link| link.id.as_str()).collect();
        assert_eq!(ids, ["PocketGodFigures", "PocketGodOogaJump", "PocketGodUranus", "PocketGodComic", "PocketGod"]);

        let country_count: usize = file.links.iter().map(|link| link.country_links.len()).sum();
        assert_eq!(country_count, 169);
    }
}
