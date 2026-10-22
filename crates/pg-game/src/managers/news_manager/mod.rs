//! `NewsManager` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod news;
pub use news::*;

/// `NewsManager` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NewsManager {
    /// `mNewsManagerListener`: `@"<NewsManagerListener>"`
    pub news_manager_listener: Option<Entity>,

    /// `mSequence`: `@"BCSequence"`
    pub sequence: Option<Entity>,

    /// `mbNewsScreenActive`: `B`
    pub news_screen_active: bool,

    /// `mbNewsTeaserActive`: `B`
    pub news_teaser_active: bool,

    /// `mNewsDefIndex`: `i`
    pub news_def_index: i32,

    /// `mpActiveNewsDef`: `^{BCNewsDef=^^?@iB@@@@@@@}`
    pub active_news_def: Option<usize>,

    /// `mpNewsDefVector`: `^{vector<BCNewsDef *, std::__1::allocator<BCNewsDef *> >=^^{BCNewsDef}^^{BCNewsDef}{__compressed_pair<BCNewsDef **, std::__1::allocator<BCNewsDef *> >=^^{BCNewsDef}}}`
    pub news_def_vector: Vec<usize>,

    /// `mLoadingIconDictionary`: `@"NSMutableDictionary"`
    pub loading_icon_dictionary: HashMap<String, String>,

    /// `mpParsedNewsDef`: `^{BCNewsDef=^^?@iB@@@@@@@}`
    pub parsed_news_def: Option<usize>,

    /// `mbParseNewsTitle`: `B`
    pub parse_news_title: bool,

    /// `mbParseNewsTease`: `B`
    pub parse_news_tease: bool,

    /// `mbParseNewsArticle`: `B`
    pub parse_news_article: bool,

    /// `mbParseCountryLink`: `B`
    pub parse_country_link: bool,

    /// `mbOpenNewsTeaserOnLoadComplete`: `B`
    pub open_news_teaser_on_load_complete: bool,

    /// `mLogicArray`: `@"NSMutableArray"`
    pub logic_array: Vec<Entity>,

    /// `mNewsScreenUiLogicArray`: `@"NSMutableArray"`
    pub news_screen_ui_logic_array: Vec<Entity>,

    /// `mNewsTeaserUiLogicArray`: `@"NSMutableArray"`
    pub news_teaser_ui_logic_array: Vec<Entity>,

    /// `mNewsScreenCloseButtonLogic`: `@"BCButtonLogic"`
    pub news_screen_close_button_logic: Option<Entity>,

    /// `mNewsScreenMoreInfoButtonLogic`: `@"BCButtonLogic"`
    pub news_screen_more_info_button_logic: Option<Entity>,

    /// `mNewsScreenNextButtonLogic`: `@"BCButtonLogic"`
    pub news_screen_next_button_logic: Option<Entity>,

    /// `mNewsScreenPrevButtonLogic`: `@"BCButtonLogic"`
    pub news_screen_prev_button_logic: Option<Entity>,

    /// `mNewsScreenLinkButtonLogic`: `@"BCButtonLogic"`
    pub news_screen_link_button_logic: Option<Entity>,

    /// `mNewsTeaserCloseButtonLogic`: `@"BCButtonLogic"`
    pub news_teaser_close_button_logic: Option<Entity>,

    /// `mNewsTeaserMoreInfoButtonLogic`: `@"BCButtonLogic"`
    pub news_teaser_more_info_button_logic: Option<Entity>,

    /// `mNewsTeaserLinkButtonLogic`: `@"BCButtonLogic"`
    pub news_teaser_link_button_logic: Option<Entity>,
}

pub const NEWS_MANAGER: Class = Class {
    name: "NewsManager",
    superclass: None,
    sequence_item,
    id,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..Class::NS_OBJECT
};

/// Sequence-item handlers implemented by `NewsManager` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "updateNewsScreen" => seq_update_news_screen(world, this, call),
        "updateNewsTeaser" => seq_update_news_teaser(world, this, call),
        "loadLocationTextureAtlas" => seq_load_location_texture_atlas(world, this, call),
        _ => return (Class::NS_OBJECT.sequence_item)(world, this, call),
    };
    Some(result)
}
