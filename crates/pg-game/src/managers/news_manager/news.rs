//! `NewsManager`: methods in this part of the class.

use quick_xml::Reader;
use quick_xml::XmlVersion;
use quick_xml::events::{BytesRef, BytesStart, Event};

use super::*;
use crate::controller;
use crate::logic::idol_ui_widgets::map_paper_logic;
use crate::managers::idol_manager;
use crate::managers::skin_pack_manager;
use crate::managers::story_manager;

use pg_engine::display;
use pg_engine::display::text;
use pg_engine::library;
use pg_engine::library::Library;
use pg_engine::platform::Locale;
use pg_engine::prefs;
use pg_engine::prefs::ServerLoadListener;
use pg_engine::sequence;
use pg_engine::sequence::{ns_bool_value, ns_int_value};

/// `-[NewsManager id]` (`0x10038efd0`).
const CLASS_ID: &str = "NewsManager";

/// `PTR_cf_http___appdata_boltcreative_com_s3_amazonaws_com_news_news_xml_1004d1a40`: the
/// feed was `http://appdata.boltcreative.com.s3.amazonaws.com/news/news.xml`. The server is
/// gone and anyone could re-register the bucket, so the port has no feed and downloads no
/// icons (`docs/re/managers-platform.md` §9.4); a URL here opts in to both.
const NEWS_FEED_URL: Option<&str> = None;

/// The news cache in `Library/Caches` (the last 10 items, UTF-16, zlib).
const NEWS_CACHE_FILE_NAME: &str = "news.xml";

/// Icon downloads of this many bytes or fewer are ignored ("not big enough").
const MIN_ICON_DATA_LENGTH: usize = 1024;

/// The cache keeps this many items, the last ones of the vector.
const CACHED_NEWS_ITEM_COUNT: usize = 10;

const LOOK_AT_NEWS_EVENT: &str = "LookAtNews";

const NEWS_CONTENT_DISPLAY_GROUP: &str = "NewsContentDisplayGroup";
const NEWS_POPUP_DISPLAY_GROUP: &str = "NewsPopupDisplayGroup";

const OPEN_NEWS_SCREEN_SEQUENCE: &str = "OpenNewsScreen";
const OPEN_NEWS_TEASER_SEQUENCE: &str = "OpenNewsTeaser";
const CLOSE_NEWS_SCREEN_SEQUENCE: &str = "CloseNewsScreen";
const CLOSE_NEWS_TEASER_SEQUENCE: &str = "CloseNewsTeaser";
const CLICK_NEWS_SCREEN_BUTTON_SEQUENCE: &str = "ClickNewsScreenButton";
const UPDATE_NEWS_SCREEN_SEQUENCE: &str = "UpdateNewsScreen";

/// The map paper's atlas, loaded into the screen slot when the paper opens.
const MAP_TEXTURE_ATLAS: &str = "MapTextureAtlas0";

/// The atlas slot `FUN_100038d68` loads `MapTextureAtlas0` into (the def at 0x10077b790).
const SCREEN_ATLAS_ID: &str = "screenAtlas0";

/// The icon shown while a news item's own icon is missing (also the bits file the stand-alone
/// news icon value starts with).
const DEFAULT_NEWS_ICON: &str = "DefaultNewsIcon";

/// `setWrapWidth:` of the title, the article and the teaser blurb (`0x43a00000`).
const NEWS_TEXT_WRAP_WIDTH: f32 = 320.0;

const NEWS_TITLE_LINE_COUNT: i32 = 1;
const NEWS_ARTICLE_LINE_COUNT: i32 = 17;
const NEWS_BLURB_LINE_COUNT: i32 = 3;

/// The page the news screen's MoreInfo button opens.
const BOLT_CREATIVE_URL: &str = "http://www.boltcreative.com";

/// `BCNewsDef` (a C++ object of 0x50 bytes, vtable 0x10037d600).
#[derive(Clone, Debug, Default)]
pub struct NewsDef {
    /// +0x08: `newsItem@id` (`""` when absent).
    pub id: String,

    /// +0x10: `newsItem@order`.
    pub order: i32,

    /// +0x14: `newsItem@wasRead`, set only when present.
    pub was_read: bool,

    /// +0x18: the `newsTitle` text (nil until one is parsed).
    pub title: Option<String>,

    /// +0x20: the `newsTease` text.
    pub tease: Option<String>,

    /// +0x28: the `newsArticle` text.
    pub article: Option<String>,

    /// +0x30: `newsItem@imageUrl`, the icon's server directory.
    pub image_url: String,

    /// +0x38: `newsItem@imageFileName`, the icon's name.
    pub image_file_name: String,

    /// +0x40: `newsItem@imageExt`.
    pub image_ext: String,

    /// +0x48: `newsItem@linkUrl`, or the text of a `countryLink` for this device's country.
    pub link_url: String,
}

/// The `BCNewsDef` objects (`new BCNewsDef` … `delete`) by id: `mpNewsDefVector`,
/// `mpActiveNewsDef` and `mpParsedNewsDef` hold these ids.
#[derive(Resource, Default)]
pub struct NewsDefStore {
    next_id: usize,
    news_defs: HashMap<usize, NewsDef>,
}

impl NewsDefStore {
    /// `new BCNewsDef`: stores `news_def` and returns its id.
    pub fn insert(&mut self, news_def: NewsDef) -> usize {
        self.next_id += 1;
        let id = self.next_id;
        self.news_defs.insert(id, news_def);
        id
    }

    /// `delete news_def`.
    pub fn remove(&mut self, id: usize) {
        self.news_defs.remove(&id);
    }

    /// The def with this id, if it is alive.
    pub fn get(&self, id: usize) -> Option<&NewsDef> {
        self.news_defs.get(&id)
    }

    /// The def with this id.
    pub fn news_def(&self, id: usize) -> &NewsDef {
        self.news_defs.get(&id).expect("a live BCNewsDef")
    }

    /// The def with this id, mutably.
    pub fn news_def_mut(&mut self, id: usize) -> &mut NewsDef {
        self.news_defs.get_mut(&id).expect("a live BCNewsDef")
    }
}

/// `NewsManager`'s file-scope state.
#[derive(Resource, Default)]
pub struct NewsManagerStatics {
    /// The bits file of the stand-alone news icon value (`0x1005f9710`; nil here stands for its
    /// initial `DefaultNewsIcon`): the image `FUN_1000384b0` and `FUN_100038800` upload into
    /// the value's one record, the 64 × 64 `NewsIcon` rect of `NewsAtlas0`.
    pub news_icon_bits_image: Option<String>,
}

/// The ivar a button logic of `setLogic:` is stored in.
type LogicSlot = fn(&mut NewsManager) -> &mut Option<Entity>;

/// The UI array a button logic of `setLogic:` joins.
type UiArraySlot = fn(&mut NewsManager) -> &mut Vec<Entity>;

/// The ivar and the UI array of each `BCButtonLogic` that `setLogic:` creates.
fn button_logic_slot(logic_id: &str) -> Option<(LogicSlot, UiArraySlot)> {
    let slot: (LogicSlot, UiArraySlot) = match logic_id {
        "NewsScreenCloseButtonLogic" => (
            |manager| &mut manager.news_screen_close_button_logic,
            |manager| &mut manager.news_screen_ui_logic_array,
        ),
        "NewsScreenMoreInfoButtonLogic" => (
            |manager| &mut manager.news_screen_more_info_button_logic,
            |manager| &mut manager.news_screen_ui_logic_array,
        ),
        "NewsScreenNextButtonLogic" => (
            |manager| &mut manager.news_screen_next_button_logic,
            |manager| &mut manager.news_screen_ui_logic_array,
        ),
        "NewsScreenPrevButtonLogic" => (
            |manager| &mut manager.news_screen_prev_button_logic,
            |manager| &mut manager.news_screen_ui_logic_array,
        ),
        "NewsScreenLinkButtonLogic" => (
            |manager| &mut manager.news_screen_link_button_logic,
            |manager| &mut manager.news_screen_ui_logic_array,
        ),
        "NewsTeaserCloseButtonLogic" => (
            |manager| &mut manager.news_teaser_close_button_logic,
            |manager| &mut manager.news_teaser_ui_logic_array,
        ),
        "NewsTeaserMoreInfoButtonLogic" => (
            |manager| &mut manager.news_teaser_more_info_button_logic,
            |manager| &mut manager.news_teaser_ui_logic_array,
        ),
        "NewsTeaserLinkButtonLogic" => (
            |manager| &mut manager.news_teaser_link_button_logic,
            |manager| &mut manager.news_teaser_ui_logic_array,
        ),
        _ => return None,
    };
    Some(slot)
}

/// The news objects (created by `init`).
fn news_def_store(world: &World) -> &NewsDefStore {
    world.resource::<NewsDefStore>()
}

/// The news objects, mutably (created on first use).
fn news_def_store_mut(world: &mut World) -> Mut<'_, NewsDefStore> {
    world.get_resource_or_insert_with(NewsDefStore::default)
}

/// `%@` of an object that may be nil.
fn ns_description(value: Option<&str>) -> &str {
    value.unwrap_or("(null)")
}

/// `mNewsManagerListener` (the view controller: `loadGame` sets it right after `init`).
fn listener(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<NewsManager>(this).news_manager_listener
}

/// `[mNewsManagerListener systemPreferences]`.
fn listener_system_preferences(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = listener(world, this)?;
    controller::system_preferences(world, listener)
}

/// `[mNewsManagerListener mapPaperLogic]`.
fn listener_map_paper_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = listener(world, this)?;
    controller::map_paper_logic(world, listener)
}

/// `[mNewsManagerListener openUrl:url]`.
fn listener_open_url(world: &mut World, this: Entity, url: &str) {
    let listener = listener(world, this);
    if let Some(listener) = listener {
        controller::open_url(world, listener, url);
    }
}

/// `[mNewsManagerListener onNewsScreenClosed]`.
fn notify_news_screen_closed(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    if let Some(listener) = listener {
        controller::on_news_screen_closed(world, listener);
    }
}

/// `[mNewsManagerListener onNewsTeaserClosed]`.
fn notify_news_teaser_closed(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    if let Some(listener) = listener {
        controller::on_news_teaser_closed(world, listener);
    }
}

/// `[mNewsManagerListener onLoadLocationTextureAtlas]`.
fn notify_load_location_texture_atlas(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    if let Some(listener) = listener {
        controller::on_load_location_texture_atlas(world, listener);
    }
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:id]]`.
fn set_sequence(world: &mut World, this: Entity, id: &str) {
    let manager_sequence = world.obj::<NewsManager>(this).sequence;
    let sequence_def = library::sequence_def(world, id);
    if let Some(manager_sequence) = manager_sequence {
        sequence::set_sequence_def(world, manager_sequence, sequence_def);
    }
}

/// `[group getDisplayObject:id]` (nil for a nil group).
fn group_object(world: &World, display_group: Option<Entity>, id: &str) -> Option<Entity> {
    let display_group = display_group?;
    display::group::get_display_object(world, display_group, id)
}

/// `[object show]`.
fn show(world: &mut World, object: Option<Entity>) {
    if let Some(object) = object {
        display::object_mut(world, object).hidden = false;
    }
}

/// `[object hide]`.
fn hide(world: &mut World, object: Option<Entity>) {
    if let Some(object) = object {
        display::object_mut(world, object).hidden = true;
    }
}

/// `[object setIsSelectable:selectable]`.
fn set_selectable(world: &mut World, object: Option<Entity>, selectable: bool) {
    if let Some(object) = object {
        display::object_mut(world, object).is_selectable = selectable;
    }
}

/// `[object setText:value]`.
fn set_text(world: &mut World, object: Option<Entity>, value: Option<&str>) {
    if let Some(object) = object {
        text::set_text(world, object, value);
    }
}

/// `[object setWrapWidth:wrap_width]` and `[object setLineCount:line_count]`.
fn set_text_layout(world: &mut World, object: Option<Entity>, wrap_width: f32, line_count: i32) {
    if let Some(object) = object {
        text::set_wrap_width(world, object, wrap_width);
        text::set_line_count(world, object, line_count);
    }
}

/// `[[BCButtonLogic alloc] initWithDisplayObject:]` with this object as its listener.
fn spawn_button_logic(world: &mut World, this: Entity, display_object: Entity) -> Entity {
    let button = pg_engine::logic::button::spawn(world, display_object);
    pg_engine::logic::button::set_button_logic_listener(world, button, Some(this));
    button
}

/// This object as the listener of `loadDataFromServer:listenerData:url:`.
fn server_load_listener(this: Entity) -> ServerLoadListener {
    ServerLoadListener {
        object: this,
        on_complete: on_load_data_from_server_complete,
        on_error: on_load_data_from_server_error,
    }
}

/// `FUN_100038d68(file, @"raw", screenAtlas0)`.
fn load_screen_atlas(world: &mut World, file: &str) {
    let screen_atlas = world.resource::<Library>().atlas_id(SCREEN_ATLAS_ID);
    if let Some(screen_atlas) = screen_atlas {
        pg_engine::texture::load_atlas(world, screen_atlas, file);
    }
}

/// Writes the stand-alone news icon value's bits file (`0x1005f9710`).
fn set_news_icon_bits_image(world: &mut World, image: &str) {
    let mut statics = world.get_resource_or_insert_with(NewsManagerStatics::default);
    statics.news_icon_bits_image = Some(image.to_owned());
}

/// The stand-alone news icon value's bits file (`0x1005f9710`).
fn news_icon_bits_image(world: &World) -> String {
    let image = match world.get_resource::<NewsManagerStatics>() {
        Some(statics) => statics.news_icon_bits_image.clone(),
        None => None,
    };
    image.unwrap_or_else(|| DEFAULT_NEWS_ICON.to_owned())
}

/// `FUN_1000384b0(&newsIconValue)` (`activateSkinTextureBitsBatchDef`): uploads
/// `<bits file>.raw` from the bundle into the `NewsIcon` rect. Every bundled `*NewsIcon` image
/// lists that rect as its one region, which is the value's one record.
fn activate_news_icon_bits(world: &mut World) {
    let image = news_icon_bits_image(world);
    skin_pack_manager::activate_skin_texture_bits_batch_def(world, &image);
}

/// `FUN_100038800(&newsIconValue)` (`activateSkinTextureBitsBatchDefLibraryCache`): the same
/// upload from `Library/Caches/<bits file>.raw`.
fn activate_news_icon_bits_library_cache(_world: &mut World) {
    // PORT: the renderer uploads only the embedded images, so a cached icon cannot be shown.
    // Only an icon download writes one, and the port downloads none (NEWS_FEED_URL), so this
    // is never reached.
}

/// `[text dataUsingEncoding:NSUnicodeStringEncoding]`: a byte-order mark, then UTF-16 in the
/// device's (little-endian) byte order.
fn unicode_data(text: &str) -> Vec<u8> {
    let mut data = Vec::with_capacity(2 + text.len() * 2);
    data.extend_from_slice(&[0xff, 0xfe]);
    for unit in text.encode_utf16() {
        data.extend_from_slice(&unit.to_le_bytes());
    }
    data
}

/// `FUN_100055864`: one cached `newsItem` (the values are not escaped).
fn news_item_xml(news_def: &NewsDef) -> String {
    let mut xml = String::with_capacity(1000);

    let was_read = i32::from(news_def.was_read);
    let item = format!(
        "<newsItem order=\"{}\" wasRead=\"{}\" id=\"{}\" imageUrl=\"{}\" imageFileName=\"{}\" imageExt=\"{}\" linkUrl=\"{}\">\n",
        news_def.order, was_read, news_def.id, news_def.image_url, news_def.image_file_name, news_def.image_ext, news_def.link_url,
    );
    xml.push_str(&item);

    let title = ns_description(news_def.title.as_deref());
    let title_xml = format!("    <newsTitle><![CDATA[{title}]]></newsTitle>\n");
    xml.push_str(&title_xml);

    let tease = ns_description(news_def.tease.as_deref());
    let tease_xml = format!("    <newsTease><![CDATA[{tease}]]></newsTease>\n");
    xml.push_str(&tease_xml);

    let article = ns_description(news_def.article.as_deref());
    let article_xml = format!("    <newsArticle><![CDATA[{article}]]></newsArticle>\n");
    xml.push_str(&article_xml);

    xml.push_str("</newsItem>\n");
    xml
}

/// `-[NewsManager init]` @ 0x100055444
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(NewsManager::default());

    // The news vector, the icon dictionary and the three logic arrays start empty.
    let manager_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<NewsManager>(this).sequence = Some(manager_sequence);
    world.init_resource::<NewsDefStore>();
}

/// `-[NewsManager id]` @ 0x10005559c
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some(CLASS_ID.to_owned())
}

/// `-[NewsManager setNewsManagerListener:]` @ 0x1000555a8
pub fn set_news_manager_listener(world: &mut World, this: Entity, news_manager_listener: Entity) {
    world.obj_mut::<NewsManager>(this).news_manager_listener = Some(news_manager_listener);
}

/// `-[NewsManager dealloc]` @ 0x100055604
pub fn dealloc(world: &mut World, this: Entity) {
    let news_defs = std::mem::take(&mut world.obj_mut::<NewsManager>(this).news_def_vector);
    {
        let mut store = news_def_store_mut(world);
        for news_def in news_defs {
            store.remove(news_def);
        }
    }

    // The arrays are the logics' only owners, but their display objects still point at them,
    // so the logics are kept alive.
    let manager_sequence = world.obj::<NewsManager>(this).sequence;
    if let Some(manager_sequence) = manager_sequence {
        sequence::stop(world, manager_sequence);
        world.despawn(manager_sequence);
    }

    world.despawn(this);
}

/// `-[NewsManager saveNewsToFile]` @ 0x100055740
pub fn save_news_to_file(world: &mut World, this: Entity) {
    let mut xml = String::with_capacity(1000);
    xml.push_str("<news>\n");

    let news_defs = world.obj::<NewsManager>(this).news_def_vector.clone();
    let first_cached = news_defs.len().saturating_sub(CACHED_NEWS_ITEM_COUNT);
    let store = news_def_store(world);
    for news_def in &news_defs[first_cached..] {
        let def = store.news_def(*news_def);
        let item_xml = news_item_xml(def);
        xml.push_str(&item_xml);
    }
    xml.push_str("</news>\n");

    let data = unicode_data(&xml);
    let blob = story_manager::compress_blob(&data);

    let system_preferences = listener_system_preferences(world, this);
    let (Some(_), Some(blob)) = (system_preferences, blob) else {
        return;
    };
    prefs::write_application_data_to_library_cache_file(world, &blob, NEWS_CACHE_FILE_NAME);
}

/// `-[NewsManager loadNewsFromFile]` @ 0x100055964
pub fn load_news_from_file(world: &mut World, this: Entity) {
    let system_preferences = listener_system_preferences(world, this);
    if system_preferences.is_none() {
        return;
    }

    let data = prefs::application_data_from_library_cache_file(world, NEWS_CACHE_FILE_NAME);
    let Some(data) = data else {
        return;
    };
    let xml_data = story_manager::uncompress_blob(&data);
    parse_xml_from_data(world, this, xml_data.as_deref());
}

/// `-[NewsManager hasUnreadNews]` @ 0x1000559dc
pub fn has_unread_news(world: &mut World, this: Entity) -> bool {
    // Only the last item counts.
    let last_news_def = world.obj::<NewsManager>(this).news_def_vector.last().copied();
    let Some(last_news_def) = last_news_def else {
        return false;
    };
    !news_def_store(world).news_def(last_news_def).was_read
}

/// `-[NewsManager markAsRead]` @ 0x100055a18
pub fn mark_as_read(world: &mut World, this: Entity) {
    let news_defs = world.obj::<NewsManager>(this).news_def_vector.clone();
    {
        let mut store = news_def_store_mut(world);
        for news_def in news_defs {
            store.news_def_mut(news_def).was_read = true;
        }
    }
    save_news_to_file(world, this);
}

/// `-[NewsManager openNewsTeaserOnLoadComplete]` @ 0x100055a54
pub fn open_news_teaser_on_load_complete(world: &mut World, this: Entity) {
    world.obj_mut::<NewsManager>(this).open_news_teaser_on_load_complete = true;
}

/// `-[NewsManager loadNewsDefs]` @ 0x100055a68
pub fn load_news_defs(world: &mut World, this: Entity) {
    load_news_from_file(world, this);

    let news_xml_path = prefs::resource_asset_path("news", "xml");
    parse_xml_from_file(world, this, Some(&news_xml_path));

    // PORT: the feed request is made only when a feed URL is configured (NEWS_FEED_URL).
    let system_preferences = listener_system_preferences(world, this);
    let (Some(system_preferences), Some(feed_url)) = (system_preferences, NEWS_FEED_URL) else {
        return;
    };
    let server_listener = server_load_listener(this);
    prefs::load_data_from_server(world, system_preferences, server_listener, None, feed_url);
}

/// `-[NewsManager openNewsScreen]` @ 0x100055b1c
pub fn open_news_screen(world: &mut World, this: Entity) {
    let idol_manager = idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        idol_manager::delay_trigger_episode_event(world, idol_manager, LOOK_AT_NEWS_EVENT);
    }

    let news_teaser_active = world.obj::<NewsManager>(this).news_teaser_active;
    if news_teaser_active {
        world.obj_mut::<NewsManager>(this).news_teaser_active = false;
        notify_news_teaser_closed(world, this);
    }

    {
        let mut manager = world.obj_mut::<NewsManager>(this);
        let news_def_count = manager.news_def_vector.len();
        manager.news_def_index = 0;
        if news_def_count != 0 {
            manager.news_def_index = news_def_count as i32 - 1;
        }
        manager.news_screen_active = true;
    }

    let news_content = library::display_group(world, NEWS_CONTENT_DISPLAY_GROUP);
    set_sequence(world, this, OPEN_NEWS_SCREEN_SEQUENCE);

    let map_paper = listener_map_paper_logic(world, this);
    if let Some(map_paper) = map_paper {
        map_paper_logic::set_map_paper_logic_listener(world, map_paper, this);
        map_paper_logic::activate(world, map_paper);
    }
    hide(world, news_content);
}

/// `-[NewsManager openNewsTeaser]` @ 0x100055cb8
pub fn open_news_teaser(world: &mut World, this: Entity) {
    world.obj_mut::<NewsManager>(this).news_teaser_active = true;
    set_sequence(world, this, OPEN_NEWS_TEASER_SEQUENCE);
    mark_as_read(world, this);
}

/// `-[NewsManager closeNewsScreen]` @ 0x100055d48
pub fn close_news_screen(world: &mut World, this: Entity) {
    let news_screen_active = world.obj::<NewsManager>(this).news_screen_active;
    if news_screen_active {
        world.obj_mut::<NewsManager>(this).news_screen_active = false;
        notify_news_screen_closed(world, this);
        set_sequence(world, this, CLOSE_NEWS_SCREEN_SEQUENCE);
    }

    let news_teaser_active = world.obj::<NewsManager>(this).news_teaser_active;
    if news_teaser_active {
        world.obj_mut::<NewsManager>(this).news_teaser_active = false;
        notify_news_teaser_closed(world, this);
        set_sequence(world, this, CLOSE_NEWS_TEASER_SEQUENCE);
    }
}

/// `-[NewsManager onMapPaperLogicShowMap]` @ 0x100055e74
pub fn on_map_paper_logic_show_map(world: &mut World, _this: Entity) {
    load_screen_atlas(world, MAP_TEXTURE_ATLAS);
    let news_content = library::display_group(world, NEWS_CONTENT_DISPLAY_GROUP);
    show(world, news_content);
}

/// `-[NewsManager onMapPaperLogicHideMap]` @ 0x100055edc
pub fn on_map_paper_logic_hide_map(world: &mut World, this: Entity) {
    notify_news_screen_closed(world, this);
    notify_load_location_texture_atlas(world, this);
    let news_content = library::display_group(world, NEWS_CONTENT_DISPLAY_GROUP);
    hide(world, news_content);
}

/// `-[NewsManager onMapPaperLogicAnimComplete]` @ 0x100055f64
pub fn on_map_paper_logic_anim_complete(world: &mut World, this: Entity) {
    set_sequence(world, this, CLOSE_NEWS_SCREEN_SEQUENCE);
}

/// `-[NewsManager updateNewsIcon:]` @ 0x100055fcc
pub fn update_news_icon(world: &mut World, this: Entity, news_def: Option<usize>) {
    let system_preferences = listener_system_preferences(world, this);

    // Callers pass the active def, which is never nil here (a nil one would crash the
    // original).
    let Some(news_def) = news_def else {
        return;
    };
    let def = news_def_store(world).news_def(news_def).clone();

    let in_resources = system_preferences.is_some() && prefs::does_file_exist_in_resources(&def.image_file_name, "raw");
    if in_resources {
        set_news_icon_bits_image(world, &def.image_file_name);
        activate_news_icon_bits(world);
        return;
    }

    let in_library_cache = match system_preferences {
        Some(_) => prefs::does_file_exist_in_library_cache(world, &def.image_file_name, "raw"),
        None => false,
    };
    if in_library_cache {
        set_news_icon_bits_image(world, &def.image_file_name);
        activate_news_icon_bits_library_cache(world);
        return;
    }

    set_news_icon_bits_image(world, DEFAULT_NEWS_ICON);
    activate_news_icon_bits(world);

    let already_loading = world.obj::<NewsManager>(this).loading_icon_dictionary.contains_key(&def.image_file_name);
    if already_loading {
        return;
    }
    {
        let mut manager = world.obj_mut::<NewsManager>(this);
        let image_file_name = def.image_file_name.clone();
        manager.loading_icon_dictionary.insert(image_file_name.clone(), image_file_name);
    }

    // PORT: icons are downloaded only when a feed URL is configured (NEWS_FEED_URL).
    let icon_url = format!("{}{}.{}", def.image_url, def.image_file_name, def.image_ext);
    let (Some(system_preferences), Some(_)) = (system_preferences, NEWS_FEED_URL) else {
        return;
    };
    let server_listener = server_load_listener(this);
    let listener_data = Some(news_def as u64);
    prefs::load_data_from_server(world, system_preferences, server_listener, listener_data, &icon_url);
}

/// `-[NewsManager updateNewsScreen]` @ 0x10005615c
pub fn update_news_screen(world: &mut World, this: Entity) {
    let news_content = library::display_group(world, NEWS_CONTENT_DISPLAY_GROUP);
    let title = group_object(world, news_content, "Title");
    let article = group_object(world, news_content, "Article");
    let prev_button = group_object(world, news_content, "PrevButton");
    let next_button = group_object(world, news_content, "NextButton");
    let click_me = group_object(world, news_content, "ClickMe");
    let news_image = group_object(world, news_content, "NewsImage");

    hide(world, prev_button);
    set_selectable(world, prev_button, false);
    hide(world, next_button);
    set_selectable(world, next_button, false);
    hide(world, click_me);
    set_selectable(world, news_image, false);
    set_text(world, title, Some(""));
    set_text(world, article, Some(""));

    let news_def_index = world.obj::<NewsManager>(this).news_def_index;
    if news_def_index < 0 {
        return;
    }

    // PORT: with no news at all the original reads past the end of the empty vector and
    // crashes; the screen stays blank instead.
    let active_news_def = world.obj::<NewsManager>(this).news_def_vector.get(news_def_index as usize).copied();
    let Some(active_news_def) = active_news_def else {
        return;
    };
    world.obj_mut::<NewsManager>(this).active_news_def = Some(active_news_def);

    if news_def_index > 0 {
        show(world, prev_button);
        set_selectable(world, prev_button, true);
    }

    let news_def_count = world.obj::<NewsManager>(this).news_def_vector.len() as u64;
    let last_index = news_def_count.wrapping_sub(1);
    let has_next = (i64::from(news_def_index) as u64) < last_index && news_def_count != 0;
    if has_next {
        show(world, next_button);
        set_selectable(world, next_button, true);
    }

    let def = news_def_store(world).news_def(active_news_def).clone();
    if !def.link_url.is_empty() {
        show(world, click_me);
        set_selectable(world, news_image, true);
    }

    set_text(world, title, def.title.as_deref());
    set_text_layout(world, title, NEWS_TEXT_WRAP_WIDTH, NEWS_TITLE_LINE_COUNT);

    let tease = ns_description(def.tease.as_deref());
    let article_body = ns_description(def.article.as_deref());
    let article_text = format!("{tease}\n\n{article_body}");
    set_text(world, article, Some(&article_text));
    set_text_layout(world, article, NEWS_TEXT_WRAP_WIDTH, NEWS_ARTICLE_LINE_COUNT);

    update_news_icon(world, this, Some(active_news_def));
}

/// `-[NewsManager updateNewsTeaser]` @ 0x100056514
pub fn update_news_teaser(world: &mut World, this: Entity) {
    let news_popup = library::display_group(world, NEWS_POPUP_DISPLAY_GROUP);
    let news_blurb = group_object(world, news_popup, "NewsBlurb");
    let click_me = group_object(world, news_popup, "ClickMe");
    let news_image = group_object(world, news_popup, "NewsImage");

    hide(world, click_me);
    set_selectable(world, news_image, false);
    set_text(world, news_blurb, Some(""));

    // The teaser shows the last item.
    let last_news_def = world.obj::<NewsManager>(this).news_def_vector.last().copied();
    let Some(last_news_def) = last_news_def else {
        return;
    };
    world.obj_mut::<NewsManager>(this).active_news_def = Some(last_news_def);

    let def = news_def_store(world).news_def(last_news_def).clone();
    if !def.link_url.is_empty() {
        show(world, click_me);
        set_selectable(world, news_image, true);
    }

    set_text(world, news_blurb, def.tease.as_deref());
    set_text_layout(world, news_blurb, NEWS_TEXT_WRAP_WIDTH, NEWS_BLURB_LINE_COUNT);

    update_news_icon(world, this, Some(last_news_def));
}

/// `-[NewsManager updateNewsScreen:::]` @ 0x100056704
pub fn seq_update_news_screen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_news_screen(world, this);
    ItemResult::Done
}

/// `-[NewsManager updateNewsTeaser:::]` @ 0x100056728
pub fn seq_update_news_teaser(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_news_teaser(world, this);
    ItemResult::Done
}

/// `-[NewsManager loadLocationTextureAtlas:::]` @ 0x10005674c
pub fn seq_load_location_texture_atlas(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    notify_load_location_texture_atlas(world, this);
    ItemResult::Done
}

/// `-[NewsManager openNewsLink:]` @ 0x10005677c
pub fn open_news_link(world: &mut World, this: Entity, url: &str) {
    match url {
        "launchStoreScreen" => {
            close_news_screen(world, this);
            let listener = listener(world, this);
            if let Some(listener) = listener {
                controller::on_news_launch_store_screen(world, listener);
            }
        }
        "launchChallengeScreen" => {
            close_news_screen(world, this);
            let listener = listener(world, this);
            if let Some(listener) = listener {
                controller::on_news_launch_challenge_screen(world, listener);
            }
        }
        "launchStoryScreen" => {
            close_news_screen(world, this);
            let listener = listener(world, this);
            if let Some(listener) = listener {
                controller::on_news_launch_story_screen(world, listener);
            }
        }
        // Any other link also logs "NEWS OPEN URL: %@".
        _ => listener_open_url(world, this, url),
    }
}

/// `-[NewsManager onButtonPress:touch:]` @ 0x1000568e0
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let manager = world.obj::<NewsManager>(this).clone();
    let pressed = Some(button);

    if manager.news_screen_close_button_logic == pressed {
        world.obj_mut::<NewsManager>(this).news_screen_active = false;
        let map_paper = listener_map_paper_logic(world, this);
        if let Some(map_paper) = map_paper {
            map_paper_logic::deactivate(world, map_paper);
        }
        set_sequence(world, this, CLICK_NEWS_SCREEN_BUTTON_SEQUENCE);
        return;
    }

    if manager.news_screen_more_info_button_logic == pressed {
        listener_open_url(world, this, BOLT_CREATIVE_URL);
        return;
    }

    if manager.news_screen_next_button_logic == pressed {
        {
            let mut news_manager = world.obj_mut::<NewsManager>(this);
            let news_def_index = news_manager.news_def_index + 1;
            news_manager.news_def_index = news_def_index;

            let news_def_count = news_manager.news_def_vector.len();
            let last_index = (news_def_count as u64).wrapping_sub(1);
            if last_index < i64::from(news_def_index) as u64 {
                news_manager.news_def_index = news_def_count as i32 - 1;
            }
        }
        set_sequence(world, this, UPDATE_NEWS_SCREEN_SEQUENCE);
        return;
    }

    if manager.news_screen_prev_button_logic == pressed {
        {
            let mut news_manager = world.obj_mut::<NewsManager>(this);
            let news_def_index = news_manager.news_def_index;
            news_manager.news_def_index = if news_def_index > 0 { news_def_index - 1 } else { 0 };
        }
        set_sequence(world, this, UPDATE_NEWS_SCREEN_SEQUENCE);
        return;
    }

    if manager.news_teaser_close_button_logic == pressed {
        world.obj_mut::<NewsManager>(this).news_teaser_active = false;
        notify_news_teaser_closed(world, this);
        set_sequence(world, this, CLOSE_NEWS_TEASER_SEQUENCE);
        return;
    }

    if manager.news_teaser_more_info_button_logic == pressed {
        world.obj_mut::<NewsManager>(this).news_teaser_active = false;
        let listener = listener(world, this);
        if let Some(listener) = listener {
            controller::on_news_teaser_launch_news_screen(world, listener);
        }
        return;
    }

    let link_pressed = manager.news_screen_link_button_logic == pressed || manager.news_teaser_link_button_logic == pressed;
    if !link_pressed {
        return;
    }

    // The link buttons are selectable only while a def is shown (a nil one would crash the
    // original).
    let active_news_def = world.obj::<NewsManager>(this).active_news_def;
    let Some(active_news_def) = active_news_def else {
        return;
    };
    let link_url = news_def_store(world).news_def(active_news_def).link_url.clone();
    open_news_link(world, this, &link_url);
}

/// `-[NewsManager onButtonRelease:touch:]` @ 0x100056ba0
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[NewsManager onButtonReleaseOutside:touch:]` @ 0x100056ba4
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[NewsManager setLogic:]` @ 0x100056ba8: the boot script's `setLogic` items for this
/// manager (`call` carries the `BCSequenceItemDef`: `displayGroupId`, `logicId`,
/// `displayObjectId`).
pub fn set_logic(world: &mut World, this: Entity, call: &SequenceCall) {
    let display_group_id = call.param("displayGroupId");
    let logic_id = call.param("logicId");
    let display_object_id = call.param("displayObjectId");

    let display_group = match display_group_id {
        Some(display_group_id) => library::display_group(world, display_group_id),
        None => None,
    };
    let display_object = match display_object_id {
        Some(display_object_id) => group_object(world, display_group, display_object_id),
        None => None,
    };

    // A nil id matches nothing.
    let Some(logic_id) = logic_id else {
        return;
    };
    let Some((slot, ui_array)) = button_logic_slot(logic_id) else {
        return;
    };

    // PORT: the original also creates the logic on a nil display object; the engine's logics
    // need one, so nothing is created. Every `setLogic` item of the boot script names an
    // existing object.
    let Some(display_object) = display_object else {
        return;
    };

    let button = spawn_button_logic(world, this, display_object);
    let mut manager = world.obj_mut::<NewsManager>(this);
    *slot(&mut manager) = Some(button);
    ui_array(&mut manager).push(button);
    manager.logic_array.push(button);
}

// ---- News XML parser ------------------------------------------------------------------------

/// `NSXMLParser`'s encoding detection: UTF-16 with a byte-order mark (the cache, as
/// `saveNewsToFile` writes it), otherwise UTF-8 (the bundled file).
fn decode_xml_text(data: &[u8]) -> Option<String> {
    let utf16_units = |bytes: &[u8], little_endian: bool| -> Vec<u16> {
        let mut units = Vec::with_capacity(bytes.len() / 2);
        for pair in bytes.chunks_exact(2) {
            let pair = [pair[0], pair[1]];
            let unit = if little_endian { u16::from_le_bytes(pair) } else { u16::from_be_bytes(pair) };
            units.push(unit);
        }
        units
    };

    if let Some(body) = data.strip_prefix(&[0xff, 0xfe]) {
        let units = utf16_units(body, true);
        return String::from_utf16(&units).ok();
    }
    if let Some(body) = data.strip_prefix(&[0xfe, 0xff]) {
        let units = utf16_units(body, false);
        return String::from_utf16(&units).ok();
    }
    let body = data.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(data);
    let text = std::str::from_utf8(body).ok()?;
    Some(text.to_owned())
}

/// An element's attributes (`NSDictionary` of `attributes:`); nil for malformed ones, which
/// end the parse with an error.
fn element_attributes(start: &BytesStart) -> Option<HashMap<String, String>> {
    let mut attributes = HashMap::new();
    for attribute in start.attributes() {
        let attribute = attribute.ok()?;
        let key: &str = attribute.key.as_ref();
        let value = attribute.normalized_value(XmlVersion::Implicit1_0).ok()?;
        attributes.insert(key.to_owned(), value.into_owned());
    }
    Some(attributes)
}

/// The characters of an entity or character reference in text (reported on their own, like
/// `NSXMLParser` does); nil for an undeclared entity, a fatal error.
fn resolve_reference(reference: &BytesRef) -> Option<String> {
    if reference.is_char_ref() {
        let character = reference.resolve_char_ref().ok()??;
        return Some(character.to_string());
    }
    let name: &str = reference;
    let resolved = quick_xml::escape::resolve_predefined_entity(name)?;
    Some(resolved.to_owned())
}

/// `-[NSXMLParser parse]` with this manager as the delegate: the delegate methods in document
/// order. An error ends the parse after `parser:parseErrorOccurred:`.
fn parse_news_xml(world: &mut World, this: Entity, data: &[u8]) {
    let Some(text) = decode_xml_text(data) else {
        parser_parse_error_occurred(world, this);
        return;
    };

    let mut reader = Reader::from_str(&text);
    reader.config_mut().expand_empty_elements = true;

    parser_did_start_document(world, this);
    loop {
        let event = reader.read_event();
        match event {
            Ok(Event::Start(start)) => {
                let name = start.name();
                let element_name: &str = name.as_ref();
                let Some(attributes) = element_attributes(&start) else {
                    parser_parse_error_occurred(world, this);
                    return;
                };
                parser_did_start_element_namespace_uri_qualified_name_attributes(world, this, element_name, &attributes);
            }
            Ok(Event::End(end)) => {
                let name = end.name();
                let element_name: &str = name.as_ref();
                parser_did_end_element_namespace_uri_qualified_name(world, this, element_name);
            }
            Ok(Event::Text(text)) => {
                // `NSXMLParser` never reports empty character data.
                let characters = text.xml10_content();
                if !characters.is_empty() {
                    parser_found_characters(world, this, &characters);
                }
            }
            Ok(Event::CData(cdata)) => {
                // The delegate has no `parser:foundCDATA:`, so CDATA arrives as characters.
                let characters = cdata.xml10_content();
                parser_found_characters(world, this, &characters);
            }
            Ok(Event::GeneralRef(reference)) => {
                let Some(characters) = resolve_reference(&reference) else {
                    parser_parse_error_occurred(world, this);
                    return;
                };
                parser_found_characters(world, this, &characters);
            }
            Ok(Event::Eof) => {
                parser_did_end_document(world, this);
                return;
            }
            Ok(_) => {}
            Err(_) => {
                parser_parse_error_occurred(world, this);
                return;
            }
        }
    }
}

/// `-[NewsManager parseXmlFromData:]` @ 0x1000570c4
pub fn parse_xml_from_data(world: &mut World, this: Entity, parse_xml_from_data: Option<&[u8]>) {
    // A parser made from nil data parses nothing.
    let Some(data) = parse_xml_from_data else {
        return;
    };
    parse_news_xml(world, this, data);
}

/// `-[NewsManager parseXmlFromFile:]` @ 0x100057164
pub fn parse_xml_from_file(world: &mut World, this: Entity, parse_xml_from_file: Option<&str>) {
    // The bundle is the embedded asset set; a missing file parses nothing.
    let Some(path) = parse_xml_from_file else {
        return;
    };
    let Some(data) = pg_assets::file(path) else {
        return;
    };
    parse_news_xml(world, this, data);
}

/// `-[NewsManager parserDidStartDocument:]` @ 0x10005721c
pub fn parser_did_start_document(world: &mut World, this: Entity) {
    world.obj_mut::<NewsManager>(this).parse_country_link = false;
}

/// `-[NewsManager parserDidEndDocument:]` @ 0x10005722c
pub fn parser_did_end_document(_world: &mut World, _this: Entity) {}

/// `newsItem`: reuses the def with the same id (taken out of the vector, its texts kept) or
/// makes a new one, sets its attributes and inserts it after the last item of a lower order.
fn parse_news_item_element(world: &mut World, this: Entity, attributes: &HashMap<String, String>) {
    let item_id = attributes.get("id").cloned();
    let order = match attributes.get("order") {
        Some(order) => ns_int_value(order),
        None => 0,
    };
    let was_read = attributes.get("wasRead").cloned();
    let image_url = attributes.get("imageUrl").cloned();
    let image_file_name = attributes.get("imageFileName").cloned();
    let image_ext = attributes.get("imageExt").cloned();
    let link_url = attributes.get("linkUrl").cloned();

    world.obj_mut::<NewsManager>(this).parsed_news_def = None;

    // `[itemId isEqualToString:def.id]`: an item without an id never matches.
    if let Some(item_id) = &item_id {
        let position = {
            let manager = world.obj::<NewsManager>(this);
            let store = news_def_store(world);
            let news_defs = &manager.news_def_vector;
            news_defs.iter().position(|news_def| store.news_def(*news_def).id == *item_id)
        };
        if let Some(position) = position {
            let mut manager = world.obj_mut::<NewsManager>(this);
            let reused_news_def = manager.news_def_vector.remove(position);
            manager.parsed_news_def = Some(reused_news_def);
        }
    }

    // After the last item whose order is lower (the first such search result counts as 0),
    // so an item that is not newer than any other lands at index 1; never past the end.
    let insert_index = {
        let manager = world.obj::<NewsManager>(this);
        let store = news_def_store(world);
        let news_defs = &manager.news_def_vector;
        let mut index = 1;
        if !news_defs.is_empty() {
            let mut lower_index = 0;
            for (position, news_def) in news_defs.iter().enumerate() {
                if store.news_def(*news_def).order < order {
                    lower_index = position as i32;
                }
            }
            index = lower_index + 1;
        }

        let news_def_count = news_defs.len() as u64;
        let mut clamped_index = news_def_count as i32;
        if i64::from(index) as u64 <= news_def_count {
            clamped_index = index;
        }
        clamped_index.max(0) as usize
    };

    let parsed_news_def = world.obj::<NewsManager>(this).parsed_news_def;
    let parsed_news_def = match parsed_news_def {
        Some(parsed_news_def) => parsed_news_def,
        None => {
            let new_news_def = NewsDef {
                id: item_id.unwrap_or_default(),
                ..NewsDef::default()
            };
            let new_news_def = news_def_store_mut(world).insert(new_news_def);
            world.obj_mut::<NewsManager>(this).parsed_news_def = Some(new_news_def);
            new_news_def
        }
    };

    {
        let mut store = news_def_store_mut(world);
        let news_def = store.news_def_mut(parsed_news_def);
        if let Some(was_read) = was_read {
            news_def.was_read = ns_bool_value(&was_read);
        }
        news_def.order = order;
        news_def.image_url = image_url.unwrap_or_default();
        news_def.image_file_name = image_file_name.unwrap_or_default();
        news_def.image_ext = image_ext.unwrap_or_default();
        news_def.link_url = link_url.unwrap_or_default();
    }

    let mut manager = world.obj_mut::<NewsManager>(this);
    manager.news_def_vector.insert(insert_index, parsed_news_def);
}

/// `countryLink`: its text becomes the link when its `id` is this device's country.
fn parse_country_link_element(world: &mut World, this: Entity, attributes: &HashMap<String, String>) {
    let country_code = match world.get_resource::<Locale>() {
        Some(locale) => locale.country_code.clone(),
        None => None,
    };
    let link_country_code = attributes.get("id");

    // `[countryCode isEqualToString:linkId]` is NO when either is nil.
    let is_this_country = match (country_code, link_country_code) {
        (Some(country_code), Some(link_country_code)) => country_code == *link_country_code,
        _ => false,
    };
    if !is_this_country {
        return;
    }

    // The original also logs "FOUND COUTNRY LINK ID:".
    world.obj_mut::<NewsManager>(this).parse_country_link = true;
}

/// `-[NewsManager parser:didStartElement:namespaceURI:qualifiedName:attributes:]` @ 0x100057230
pub fn parser_did_start_element_namespace_uri_qualified_name_attributes(world: &mut World, this: Entity, did_start_element: &str, attributes: &HashMap<String, String>) {
    match did_start_element {
        "newsItem" => parse_news_item_element(world, this, attributes),
        "newsTitle" => world.obj_mut::<NewsManager>(this).parse_news_title = true,
        "newsTease" => world.obj_mut::<NewsManager>(this).parse_news_tease = true,
        "newsArticle" => world.obj_mut::<NewsManager>(this).parse_news_article = true,
        "countryLink" => parse_country_link_element(world, this, attributes),
        _ => {}
    }
}

/// `-[NewsManager parser:foundCharacters:]` @ 0x10005773c
pub fn parser_found_characters(world: &mut World, this: Entity, found_characters: &str) {
    // The chunk replaces the field (it is not appended); a literal `\n` becomes a newline.
    let text = found_characters.replace("\\n", "\n");

    let manager = world.obj::<NewsManager>(this);
    let parse_news_title = manager.parse_news_title;
    let parse_news_tease = manager.parse_news_tease;
    let parse_news_article = manager.parse_news_article;
    let parse_country_link = manager.parse_country_link;
    let parsed_news_def = manager.parsed_news_def;

    if parse_country_link {
        // The original also logs "NEWS COUNTRY LINK: %@".
        world.obj_mut::<NewsManager>(this).parse_country_link = false;
    }

    // The original writes through the pointer even when it is NULL (a text outside a
    // `newsItem`); nothing is written then.
    let Some(parsed_news_def) = parsed_news_def else {
        return;
    };

    let mut store = news_def_store_mut(world);
    let news_def = store.news_def_mut(parsed_news_def);
    if parse_news_title {
        news_def.title = Some(text.clone());
    }
    if parse_news_tease {
        news_def.tease = Some(text.clone());
    }
    if parse_news_article {
        news_def.article = Some(text.clone());
    }
    if parse_country_link {
        news_def.link_url = text;
    }
}

/// `-[NewsManager parser:didEndElement:namespaceURI:qualifiedName:]` @ 0x100057924
pub fn parser_did_end_element_namespace_uri_qualified_name(world: &mut World, this: Entity, did_end_element: &str) {
    let mut manager = world.obj_mut::<NewsManager>(this);
    match did_end_element {
        "newsItem" => manager.parsed_news_def = None,
        "newsTitle" => manager.parse_news_title = false,
        "newsTease" => manager.parse_news_tease = false,
        "newsArticle" => manager.parse_news_article = false,
        _ => {}
    }
}

/// `-[NewsManager parser:parseErrorOccurred:]` @ 0x1000579ec
pub fn parser_parse_error_occurred(_world: &mut World, _this: Entity) {
    // The original only logs the error, its line and its column.
}

/// `-[NewsManager onLoadDataFromServerComplete:url:data:]` @ 0x100057ab4
pub fn on_load_data_from_server_complete(world: &mut World, this: Entity, listener_data: Option<u64>, _url: &str, data: &[u8]) {
    // No listener data: the feed.
    let Some(icon_news_def) = listener_data else {
        parse_xml_from_data(world, this, Some(data));
        save_news_to_file(world, this);

        let open_news_teaser_on_load_complete = world.obj::<NewsManager>(this).open_news_teaser_on_load_complete;
        if open_news_teaser_on_load_complete && has_unread_news(world, this) {
            open_news_teaser(world, this);
        }
        return;
    };

    // Otherwise the def whose icon this is. Small answers (error pages) only log.
    if data.len() <= MIN_ICON_DATA_LENGTH {
        return;
    }

    let icon_news_def = icon_news_def as usize;
    let image_file_name = match news_def_store(world).get(icon_news_def) {
        Some(news_def) => news_def.image_file_name.clone(),
        None => return,
    };

    let system_preferences = listener_system_preferences(world, this);
    if system_preferences.is_some() {
        let icon_file_name = format!("{image_file_name}.raw");
        prefs::write_application_data_to_library_cache_file(world, data, &icon_file_name);
    }

    // A nil active def would crash the original; the icon is shown only by the screens.
    let active_news_def = world.obj::<NewsManager>(this).active_news_def;
    let Some(active_news_def) = active_news_def else {
        return;
    };
    let shows_this_icon = news_def_store(world).news_def(active_news_def).image_file_name == image_file_name;
    if shows_this_icon {
        set_news_icon_bits_image(world, &image_file_name);
        activate_news_icon_bits_library_cache(world);
    }
}

/// `-[NewsManager onLoadDataFromServerError:url:error:]` @ 0x100057c68
pub fn on_load_data_from_server_error(_world: &mut World, _this: Entity, _listener_data: Option<u64>, _url: &str, _error: &str) {
    // The original only logs the error.
}
