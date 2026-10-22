//! `StoryManager`: methods in this part of the class.
//!
//! This part also owns the story data that the original keeps outside Objective-C objects:
//! the C++ `BCStoryDef` / `BCStoryElementDef` objects ([`StoryDefStore`]), the
//! `BCStoryStateDef` tables (`story_state_*`), the story XML (`toXml` and the `NSXMLParser`
//! delegate) and the zlib story files (`compressBlob` / `uncompressBlob`).

use quick_xml::Reader;
use quick_xml::XmlVersion;
use quick_xml::events::{BytesRef, BytesStart, Event};

use pg_engine::sequence::ns_float_value;

use crate::persona::persona::Persona;
use crate::persona::tribe_persona::TribePersona;
use crate::prelude::*;

use super::*;

// ---- Story objects ------------------------------------------------------------------------

/// `BCStoryElementDef::type` (+0x08) of an environment.
pub const STORY_ENVIRONMENT_ELEMENT: i32 = 0;

/// `BCStoryElementDef::type` of a caption.
pub const STORY_CAPTION_ELEMENT: i32 = 1;

/// `BCStoryElementDef::type` of a character.
pub const STORY_CHARACTER_ELEMENT: i32 = 2;

/// `BCStoryElementDef::type` of an event.
pub const STORY_EVENT_ELEMENT: i32 = 3;

/// `0x40a00000`: the delay of a new element.
pub const DEFAULT_STORY_ELEMENT_DELAY: f32 = 5.0;

/// `@"ENTER TITLE"`: the title of a new story.
pub const ENTER_TITLE: &str = "ENTER TITLE";

/// `@"ENTER CAPTION"`: the text of a new caption.
pub const ENTER_CAPTION: &str = "ENTER CAPTION";

/// `@"ENTER NAME"`: the name of a new character.
pub const ENTER_NAME: &str = "ENTER NAME";

/// `@"ENTER DIALOG"`: the dialog of a new character.
pub const ENTER_DIALOG: &str = "ENTER DIALOG";

/// `@"Idle"`: the state of a new character.
pub const IDLE_STATE: &str = "Idle";

/// `@"Dance"`: the state of a new event.
pub const DANCE_STATE: &str = "Dance";

/// `@"[NEW NAME]"`: the first entry of the name menu.
pub const NEW_NAME: &str = "[NEW NAME]";

/// `@"StoryMap"`: the file listing the My List stories.
const STORY_MAP_FILE: &str = "StoryMap";

/// `kCFAbsoluteTimeIntervalSince1970`: seconds from 1970-01-01 to 2001-01-01 (UTC).
const ABSOLUTE_TIME_INTERVAL_SINCE_1970: i64 = 978_307_200;

const SECONDS_PER_DAY: i64 = 86_400;

/// The C++ class of a `BCStoryElementDef` (its vtable), which picks `clone`, `isEqual` and
/// `toXml`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryElementClass {
    /// `BCStoryEnvironmentElementDef` (vtable `0x10037d460`, 0x20 bytes).
    Environment,

    /// `BCStoryCaptionElementDef` (vtable `0x10037d4a0`, 0x18 bytes).
    Caption,

    /// `BCStoryCharacterElementDef` (vtable `0x10037d4e0`, 0x28 bytes).
    Character,

    /// `BCStoryEventElementDef` (vtable `0x10037d520`, 0x18 bytes).
    Event,
}

/// A `BCStoryElementDef`: the base members and those of its four subclasses. In the original
/// the subclasses overlay their members from +0x10 on (the location, the caption, the
/// character's name or the event's state); here each has its own field and the fields of the
/// other classes stay at their defaults.
#[derive(Clone, Debug)]
pub struct StoryElementDef {
    /// The C++ class (vtable).
    pub class: StoryElementClass,

    /// +0x08 `int`: [`STORY_ENVIRONMENT_ELEMENT`] … [`STORY_EVENT_ELEMENT`].
    pub element_type: i32,

    /// +0x0c `float`: seconds the element plays (`storyStall:::`).
    pub delay: f32,

    /// Environment +0x10 `int`: index into [`STORY_LOCATION_STATE_ARRAY`].
    pub location: i32,

    /// Environment +0x14 `int`: index into [`STORY_WEATHER_STATE_ARRAY`].
    pub weather: i32,

    /// Environment +0x18 `int`: index into [`STORY_TIME_OF_DAY_STATE_ARRAY`].
    pub time_of_day: i32,

    /// Caption +0x10 `NSString *`.
    pub caption: Option<String>,

    /// Character +0x10 `NSString *`: the pygmy's name.
    pub name: Option<String>,

    /// Character +0x18 `NSString *`: the word balloon's text.
    pub dialog: Option<String>,

    /// Character +0x20 / event +0x10 `NSString *`: a character or event state id.
    pub state: Option<String>,
}

impl StoryElementDef {
    fn blank(class: StoryElementClass, element_type: i32) -> Self {
        Self {
            class,
            element_type,
            delay: DEFAULT_STORY_ELEMENT_DELAY,
            location: 0,
            weather: 0,
            time_of_day: 0,
            caption: None,
            name: None,
            dialog: None,
            state: None,
        }
    }

    /// `new BCStoryEnvironmentElementDef` as the parser and the editor build it: Sand Island,
    /// clear, day, a 5 s delay.
    pub fn new_environment() -> Self {
        Self::blank(StoryElementClass::Environment, STORY_ENVIRONMENT_ELEMENT)
    }

    /// `new BCStoryCaptionElementDef`: `"ENTER CAPTION"`, a 5 s delay.
    pub fn new_caption() -> Self {
        let mut element_def = Self::blank(StoryElementClass::Caption, STORY_CAPTION_ELEMENT);
        element_def.caption = Some(ENTER_CAPTION.to_owned());
        element_def
    }

    /// `new BCStoryCharacterElementDef`: `"ENTER NAME"`, `"ENTER DIALOG"`, `Idle`, a 5 s delay.
    pub fn new_character() -> Self {
        let mut element_def = Self::blank(StoryElementClass::Character, STORY_CHARACTER_ELEMENT);
        element_def.name = Some(ENTER_NAME.to_owned());
        element_def.dialog = Some(ENTER_DIALOG.to_owned());
        element_def.state = Some(IDLE_STATE.to_owned());
        element_def
    }

    /// `new BCStoryEventElementDef`: `Dance`, a 5 s delay.
    pub fn new_event() -> Self {
        let mut element_def = Self::blank(StoryElementClass::Event, STORY_EVENT_ELEMENT);
        element_def.state = Some(DANCE_STATE.to_owned());
        element_def
    }

    /// `clone` (vtable +0x10: `FUN_1000531b4`, `FUN_1000534f4`, `FUN_1000536dc`,
    /// `FUN_100053934`): a new element of the same class with the class's type, this delay
    /// and this element's own members.
    pub fn clone_element_def(&self) -> Self {
        let mut element_def = match self.class {
            StoryElementClass::Environment => Self::new_environment(),
            StoryElementClass::Caption => Self::blank(StoryElementClass::Caption, STORY_CAPTION_ELEMENT),
            StoryElementClass::Character => Self::blank(StoryElementClass::Character, STORY_CHARACTER_ELEMENT),
            StoryElementClass::Event => Self::blank(StoryElementClass::Event, STORY_EVENT_ELEMENT),
        };
        element_def.delay = self.delay;

        match self.class {
            StoryElementClass::Environment => {
                element_def.location = self.location;
                element_def.weather = self.weather;
                element_def.time_of_day = self.time_of_day;
            }
            StoryElementClass::Caption => {
                element_def.caption = self.caption.clone();
            }
            StoryElementClass::Character => {
                element_def.name = self.name.clone();
                element_def.dialog = self.dialog.clone();
                element_def.state = self.state.clone();
            }
            StoryElementClass::Event => {
                element_def.state = self.state.clone();
            }
        }
        element_def
    }

    /// `isEqual` (vtable +0x18: `FUN_10005320c`, `FUN_100053574`, `FUN_1000537bc`,
    /// `FUN_1000539b4`) with `self` as the receiver: same type and delay, then this class's
    /// members, the strings compared with `[other.x isEqualToString:self.x]`.
    pub fn is_equal(&self, other: &StoryElementDef) -> bool {
        if other.element_type != self.element_type {
            return false;
        }
        if other.delay != self.delay {
            return false;
        }

        match self.class {
            StoryElementClass::Environment => {
                other.location == self.location
                    && other.weather == self.weather
                    && other.time_of_day == self.time_of_day
            }
            StoryElementClass::Caption => ns_string_equal(other.caption.as_deref(), self.caption.as_deref()),
            StoryElementClass::Character => {
                ns_string_equal(other.name.as_deref(), self.name.as_deref())
                    && ns_string_equal(other.dialog.as_deref(), self.dialog.as_deref())
                    && ns_string_equal(other.state.as_deref(), self.state.as_deref())
            }
            StoryElementClass::Event => ns_string_equal(other.state.as_deref(), self.state.as_deref()),
        }
    }

    /// `toXml` (vtable +0x20: `FUN_100053268`, `FUN_1000535c8`, `FUN_100053858`,
    /// `FUN_100053a08`). Texts are not escaped: a `]]>` in user text breaks the file.
    pub fn to_xml(&self) -> String {
        let delay = f64::from(self.delay);
        match self.class {
            StoryElementClass::Environment => {
                let location = location_id(self.location);
                let weather = if self.weather == 1 { "Cloudy" } else { "Clear" };
                let time_of_day = match self.time_of_day {
                    1 => "Night",
                    2 => "Sunset",
                    _ => "Day",
                };
                format!(
                    "    <storyEnvironment delay=\"{delay:.1}\" location=\"{location}\" weather=\"{weather}\" timeOfDay=\"{time_of_day}\"/>\n"
                )
            }
            StoryElementClass::Caption => {
                let caption = ns_description(self.caption.as_deref());
                format!("    <storyCaption delay=\"{delay:.1}\"><![CDATA[{caption}]]></storyCaption>\n")
            }
            StoryElementClass::Character => {
                let state = ns_description(self.state.as_deref());
                let name = ns_description(self.name.as_deref());
                let dialog = ns_description(self.dialog.as_deref());

                let mut xml = String::with_capacity(100);
                xml.push_str(&format!("    <storyCharacter delay=\"{delay:.1}\" state=\"{state}\">\n"));
                xml.push_str(&format!("        <storyCharacterName><![CDATA[{name}]]></storyCharacterName>\n"));
                xml.push_str(&format!("        <storyCharacterDialog><![CDATA[{dialog}]]></storyCharacterDialog>\n"));
                xml.push_str("    </storyCharacter>\n");
                xml
            }
            StoryElementClass::Event => {
                let state = ns_description(self.state.as_deref());
                format!("    <storyEvent delay=\"{delay:.1}\" state=\"{state}\"/>\n")
            }
        }
    }
}

/// A `BCStoryDef` (vtable `0x10037d440`, 0x28 bytes). Its +0x18 `int`, always 0 and never
/// read, is left out.
#[derive(Clone, Debug)]
pub struct StoryDef {
    /// +0x08 `NSString *`
    pub title: Option<String>,

    /// +0x10 `NSDate *` as `CFAbsoluteTime` seconds: nil for the built-in stories, set to now
    /// when a story without one is saved or named.
    pub date: Option<f64>,

    /// +0x20 `std::vector<BCStoryElementDef *> *`: element ids in [`StoryDefStore`].
    pub element_defs: Vec<usize>,
}

impl StoryDef {
    /// `new BCStoryDef` as the parser, the Create tab and `clone` build it: `"ENTER TITLE"`, no
    /// date, no elements.
    pub fn new() -> Self {
        Self {
            title: Some(ENTER_TITLE.to_owned()),
            date: None,
            element_defs: Vec::new(),
        }
    }
}

impl Default for StoryDef {
    fn default() -> Self {
        Self::new()
    }
}

/// The C++ heap of story objects. A `BCStoryDef *` or a `BCStoryElementDef *` is `Some(id)` of
/// an object here (`None` is NULL); `delete` removes the object. Ids are never reused, so a
/// dangling pointer finds nothing instead of another object.
#[derive(Resource, Debug, Default)]
pub struct StoryDefStore {
    next_id: usize,
    story_defs: HashMap<usize, StoryDef>,
    element_defs: HashMap<usize, StoryElementDef>,
}

impl StoryDefStore {
    fn allocate_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }

    /// `new BCStoryDef`: stores `story_def` and returns its id.
    pub fn insert_story_def(&mut self, story_def: StoryDef) -> usize {
        let id = self.allocate_id();
        self.story_defs.insert(id, story_def);
        id
    }

    /// `new BCStory…ElementDef`: stores `element_def` and returns its id.
    pub fn insert_element_def(&mut self, element_def: StoryElementDef) -> usize {
        let id = self.allocate_id();
        self.element_defs.insert(id, element_def);
        id
    }

    /// `*story_def`. Panics on a deleted story (the original reads freed memory).
    pub fn story_def(&self, id: usize) -> &StoryDef {
        let story_def = self.story_defs.get(&id);
        story_def.expect("a live BCStoryDef")
    }

    pub fn story_def_mut(&mut self, id: usize) -> &mut StoryDef {
        let story_def = self.story_defs.get_mut(&id);
        story_def.expect("a live BCStoryDef")
    }

    /// `*element_def`. Panics on a deleted element (the original reads freed memory).
    pub fn element_def(&self, id: usize) -> &StoryElementDef {
        let element_def = self.element_defs.get(&id);
        element_def.expect("a live BCStoryElementDef")
    }

    pub fn element_def_mut(&mut self, id: usize) -> &mut StoryElementDef {
        let element_def = self.element_defs.get_mut(&id);
        element_def.expect("a live BCStoryElementDef")
    }

    /// The element `id` if it still exists.
    pub fn get_element_def(&self, id: usize) -> Option<&StoryElementDef> {
        self.element_defs.get(&id)
    }

    /// `(int)story_def->elements->size()`.
    pub fn element_count(&self, story_def: usize) -> i32 {
        let element_defs = &self.story_def(story_def).element_defs;
        element_defs.len() as i32
    }

    /// `(*story_def->elements)[index]`.
    pub fn element_id(&self, story_def: usize, index: i32) -> usize {
        let element_defs = &self.story_def(story_def).element_defs;
        element_defs[index as usize]
    }

    /// `delete story_def` (`FUN_100053188` → `FUN_1000530b8`): its elements are deleted too.
    pub fn delete_story_def(&mut self, id: usize) {
        let Some(story_def) = self.story_defs.remove(&id) else {
            return;
        };
        for element_def in story_def.element_defs {
            self.delete_element_def(element_def);
        }
    }

    /// `delete element_def` (the deleting destructor at vtable +0x08).
    pub fn delete_element_def(&mut self, id: usize) {
        self.element_defs.remove(&id);
    }
}

/// The story objects (created by `init`).
pub fn story_def_store(world: &World) -> &StoryDefStore {
    world.resource::<StoryDefStore>()
}

/// The story objects, mutably (created on first use).
pub fn story_def_store_mut(world: &mut World) -> Mut<'_, StoryDefStore> {
    world.get_resource_or_insert_with(StoryDefStore::default)
}

/// `FUN_10004fd3c(story_def, index)`: erases element `index` and deletes it.
pub fn erase_story_element_def(world: &mut World, story_def: usize, index: i32) {
    let mut store = story_def_store_mut(world);
    let element_defs = &mut store.story_def_mut(story_def).element_defs;
    // PORT: an index past the end is undefined behaviour in the original; nothing happens here.
    if index < 0 || index as usize >= element_defs.len() {
        return;
    }
    let element_def = element_defs.remove(index as usize);
    store.delete_element_def(element_def);
}

/// `FUN_10004fdd8(story_def, element_def)`: appends `element_def`.
pub fn append_story_element_def(world: &mut World, story_def: usize, element_def: usize) {
    let mut store = story_def_store_mut(world);
    store.story_def_mut(story_def).element_defs.push(element_def);
}

/// `FUN_100053a7c` (`std::vector::insert` of one element) on a story's elements: inserts
/// `element_def` before element `index`.
pub fn insert_story_element_def(world: &mut World, story_def: usize, index: i32, element_def: usize) {
    let mut store = story_def_store_mut(world);
    let element_defs = &mut store.story_def_mut(story_def).element_defs;
    // PORT: a position past the end is undefined behaviour in the original; it appends here.
    let position = (index.max(0) as usize).min(element_defs.len());
    element_defs.insert(position, element_def);
}

/// `FUN_10004fe2c(story_def, index)`: moves element `index` one place later (erase, then
/// insert at `index + 1`).
pub fn move_story_element_def_later(world: &mut World, story_def: usize, index: i32) {
    let element_def = {
        let mut store = story_def_store_mut(world);
        let element_defs = &mut store.story_def_mut(story_def).element_defs;
        if index < 0 || index as usize >= element_defs.len() {
            return;
        }
        element_defs.remove(index as usize)
    };
    insert_story_element_def(world, story_def, index + 1, element_def);
}

/// `FUN_10004fee0(story_def, index)`: moves element `index` one place earlier (erase, then
/// insert at `index - 1`).
pub fn move_story_element_def_earlier(world: &mut World, story_def: usize, index: i32) {
    let element_def = {
        let mut store = story_def_store_mut(world);
        let element_defs = &mut store.story_def_mut(story_def).element_defs;
        if index < 0 || index as usize >= element_defs.len() {
            return;
        }
        element_defs.remove(index as usize)
    };
    insert_story_element_def(world, story_def, index - 1, element_def);
}

/// `FUN_10004ff94(story_def)`: a new story with this title (no date) and clones of its
/// elements.
pub fn clone_story_def(world: &mut World, story_def: usize) -> usize {
    let mut store = story_def_store_mut(world);
    let mut clone = StoryDef::new();
    let source = store.story_def(story_def).clone();
    clone.title = source.title;
    let clone_id = store.insert_story_def(clone);

    for element_def in source.element_defs {
        let element_clone = store.element_def(element_def).clone_element_def();
        let element_clone_id = store.insert_element_def(element_clone);
        store.story_def_mut(clone_id).element_defs.push(element_clone_id);
    }
    clone_id
}

/// `FUN_10004dce8(template_def, story_def)`: equal titles (`[story.title isEqualToString:
/// template.title]`), equal element counts and every template element `isEqual` to its
/// counterpart.
fn is_story_def_equal(world: &World, template_def: usize, story_def: usize) -> bool {
    let store = story_def_store(world);
    let template = store.story_def(template_def);
    let story = store.story_def(story_def);

    let same_title = ns_string_equal(story.title.as_deref(), template.title.as_deref());
    if !same_title {
        return false;
    }
    if story.element_defs.len() as i32 != template.element_defs.len() as i32 {
        return false;
    }

    for (template_element, story_element) in template.element_defs.iter().zip(&story.element_defs) {
        let template_element_def = store.element_def(*template_element);
        let story_element_def = store.element_def(*story_element);
        if !template_element_def.is_equal(story_element_def) {
            return false;
        }
    }
    true
}

/// The story's date, set to now first when it has none (`[[NSDate alloc] init]`).
fn story_def_date(world: &mut World, story_def: usize) -> f64 {
    let now = crate::controller::cf_absolute_time_get_current(world);
    let mut store = story_def_store_mut(world);
    let story = store.story_def_mut(story_def);
    *story.date.get_or_insert(now)
}

/// `FUN_100048a44(story_def)`: the story file's XML.
fn story_def_xml(world: &mut World, story_def: usize) -> String {
    let date = story_def_date(world, story_def);
    let date_text = format_story_date(date);

    let store = story_def_store(world);
    let story = store.story_def(story_def);
    let title = ns_description(story.title.as_deref());

    let mut xml = String::with_capacity(1000);
    xml.push_str(&format!("<story date=\"{date_text}\">\n"));
    xml.push_str(&format!("    <storyTitle><![CDATA[{title}]]></storyTitle>\n"));
    for element_def in &story.element_defs {
        let element_xml = store.element_def(*element_def).to_xml();
        xml.push_str(&element_xml);
    }
    xml.push_str("</story>\n");
    xml
}

/// `FUN_100048be0(story_def)`: the story's file name, `story-yyyyMMddHHmmss` from its date.
pub fn story_def_file_name(world: &mut World, story_def: usize) -> String {
    let date = story_def_date(world, story_def);
    let date_text = format_story_date(date);
    let without_colons = date_text.replace(':', "");
    let without_dashes = without_colons.replace('-', "");
    let compact = without_dashes.replace(' ', "");
    format!("story-{compact}")
}

/// `FUN_10005299c(story_def, text)`: a date attribute (`dateFromString:`, nil when it does not
/// parse); a nil attribute changes nothing.
fn set_story_def_date_from_string(world: &mut World, story_def: usize, text: Option<&str>) {
    let Some(text) = text else {
        return;
    };
    let date = parse_story_date(text);
    story_def_store_mut(world).story_def_mut(story_def).date = date;
}

/// `FUN_100053334`: a location index's id for the story files; out of range is Sand Island.
fn location_id(location: i32) -> &'static str {
    let in_range = location >= 1 && (location as usize) < LOCATION_STATES.len();
    if !in_range {
        return LOCATION_STATES[0].0;
    }
    LOCATION_STATES[location as usize].0
}

/// `FUN_100052a5c(element_def, text)`: the exact location id sets the index; anything else
/// (and nil) leaves it.
fn set_location_from_string(world: &mut World, element_def: usize, text: Option<&str>) {
    let index = LOCATION_STATES.iter().position(|(id, _, _)| Some(*id) == text);
    let Some(index) = index else {
        return;
    };
    story_def_store_mut(world).element_def_mut(element_def).location = index as i32;
}

/// `FUN_100052cb0(element_def, text)`: `Day` 0, `Night` 1, `Sunset` 2; anything else leaves
/// the time.
fn set_time_of_day_from_string(world: &mut World, element_def: usize, text: Option<&str>) {
    let mut store = story_def_store_mut(world);
    let element = store.element_def_mut(element_def);
    if text == Some("Day") {
        element.time_of_day = 0;
    }
    if text == Some("Night") {
        element.time_of_day = 1;
    }
    if text == Some("Sunset") {
        element.time_of_day = 2;
    }
}

/// `[a isEqualToString:b]`: NO when either is nil.
pub(crate) fn ns_string_equal(receiver: Option<&str>, other: Option<&str>) -> bool {
    match (receiver, other) {
        (Some(receiver), Some(other)) => receiver == other,
        _ => false,
    }
}

/// `%@` of an `NSString *`: `(null)` for nil.
pub(crate) fn ns_description(text: Option<&str>) -> &str {
    text.unwrap_or("(null)")
}

// ---- Dates (`NSDateFormatter` with `yyyy-MM-dd HH:mm:ss`) -----------------------------------

/// Days from 1970-01-01 to the given proleptic Gregorian date.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let shifted_year = if month <= 2 { year - 1 } else { year };
    let era = shifted_year.div_euclid(400);
    let year_of_era = shifted_year - era * 400;
    let shifted_month = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The proleptic Gregorian date `days` after 1970-01-01.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted_days = days + 719_468;
    let era = shifted_days.div_euclid(146_097);
    let day_of_era = shifted_days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    let first_of_month = days_from_civil(year, month, 1);
    let next_month_year = if month == 12 { year + 1 } else { year };
    let next_month = if month == 12 { 1 } else { month + 1 };
    let first_of_next_month = days_from_civil(next_month_year, next_month, 1);
    first_of_next_month - first_of_month
}

/// `[formatter stringFromDate:date]` with `yyyy-MM-dd HH:mm:ss`.
// PORT: the original formats in the device's time zone; there is none here, so UTC is used
// (both directions, so story files still round-trip).
fn format_story_date(date: f64) -> String {
    let seconds_since_1970 = date.floor() as i64 + ABSOLUTE_TIME_INTERVAL_SINCE_1970;
    let days = seconds_since_1970.div_euclid(SECONDS_PER_DAY);
    let second_of_day = seconds_since_1970.rem_euclid(SECONDS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    let hour = second_of_day / 3600;
    let minute = second_of_day % 3600 / 60;
    let second = second_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

/// One all-digit field of a date.
fn date_field(text: &str) -> Option<i64> {
    let all_digits = !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
    if !all_digits {
        return None;
    }
    text.parse().ok()
}

/// `[formatter dateFromString:text]` with `yyyy-MM-dd HH:mm:ss` (UTC, see
/// [`format_story_date`]); nil when the text does not match.
fn parse_story_date(text: &str) -> Option<f64> {
    let (date_text, time_text) = text.split_once(' ')?;
    let date_fields: Vec<&str> = date_text.split('-').collect();
    let time_fields: Vec<&str> = time_text.split(':').collect();
    if date_fields.len() != 3 || time_fields.len() != 3 {
        return None;
    }

    let year = date_field(date_fields[0])?;
    let month = date_field(date_fields[1])?;
    let day = date_field(date_fields[2])?;
    let hour = date_field(time_fields[0])?;
    let minute = date_field(time_fields[1])?;
    let second = date_field(time_fields[2])?;

    let valid_month = (1..=12).contains(&month);
    if !valid_month {
        return None;
    }
    let valid_day = day >= 1 && day <= days_in_month(year, month);
    let valid_time = hour <= 23 && minute <= 59 && second <= 59;
    if !valid_day || !valid_time {
        return None;
    }

    let days = days_from_civil(year, month, day);
    let seconds_since_1970 = days * SECONDS_PER_DAY + hour * 3600 + minute * 60 + second;
    let absolute_seconds = seconds_since_1970 - ABSOLUTE_TIME_INTERVAL_SINCE_1970;
    Some(absolute_seconds as f64)
}

// ---- BCStoryStateDef tables ------------------------------------------------------------------

/// A `BCStoryStateDef` record: `{NSString *id; NSString *displayName; u32 flags}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryStateDef {
    /// +0x00: the id the story files store.
    pub id: Option<String>,

    /// +0x08: the option menus' text.
    pub display_name: Option<String>,

    /// +0x10: what a state needs, or what a location, weather or time offers.
    pub flags: u32,
}

/// `PTR_PTR_1007cc1b8` (57 character states). A `BCStoryStateDef **` list is `Some` of one of
/// the `STORY_*_ARRAY` handles; a `BCStoryStateDef *` is
/// `Some(array * STORY_STATE_ARRAY_STRIDE + index)`, entry `index` of that list (see
/// [`story_state_array_entry`] and [`story_state_def`]).
pub const STORY_CHARACTER_STATE_ARRAY: usize = 1;

/// `PTR_PTR_1007cc418` (6 event states).
pub const STORY_EVENT_STATE_ARRAY: usize = 2;

/// `PTR_PTR_1007cc5e8` (17 locations).
pub const STORY_LOCATION_STATE_ARRAY: usize = 3;

/// `PTR_PTR_1007cc6a8` (2 weathers).
pub const STORY_WEATHER_STATE_ARRAY: usize = 4;

/// `PTR_PTR_1007cc708` (3 times of day).
pub const STORY_TIME_OF_DAY_STATE_ARRAY: usize = 5;

/// `DAT_1007e4400`: the name list `createStoryCharacterNameArray` rebuilds (records at
/// `DAT_1007e4720`).
pub const STORY_CHARACTER_NAME_ARRAY: usize = 6;

/// See [`STORY_CHARACTER_STATE_ARRAY`].
pub const STORY_STATE_ARRAY_STRIDE: usize = 0x100;

/// `(id, displayName, flags)` of a static table's records, in list order.
type StaticStoryStates = [(&'static str, &'static str, u32)];

/// The records of `PTR_PTR_1007cc1b8` (`0x1007cbc60` …).
const CHARACTER_STATES: &StaticStoryStates = &[
    ("Idle", "Idle", 0x1),
    ("Happy", "Happy", 0x1),
    ("VeryHappy", "Very Happy", 0x1),
    ("Sad", "Sad", 0x1),
    ("Crying", "Crying", 0x1),
    ("Angry", "Angry", 0x1),
    ("VeryAngry", "Very Angry", 0x1),
    ("Confused", "Confused", 0x1),
    ("VeryConfused", "Very Confused", 0x1),
    ("Scared", "Scared", 0x1),
    ("FreakedOut", "Freaked Out", 0x1),
    ("Prophecy", "Prophecy", 0x1),
    ("Dance", "Dance", 0x1),
    ("Fart", "Gassy", 0x1),
    ("Sneeze", "Sneeze", 0x1),
    ("Wave", "Wave", 0x1),
    ("SkirtFall1", "Skirt Fall 1", 0x1),
    ("SkirtFall2", "Skirt Fall 2", 0x1),
    ("NosePick", "Pick Nose", 0x1),
    ("Scratch", "Scratch Butt", 0x1),
    ("Yawn", "Yawn", 0x1),
    ("ChestBeat", "Beat Chest", 0x1),
    ("FixHair", "Fix Hair", 0x1),
    ("Cold", "Cold", 0x1),
    ("Hot", "Hot", 0x1),
    ("Explode", "Explode", 0x1),
    ("Lasered", "Lasered", 0x1),
    ("KnockOffIsland", "Knock Off Island", 0x1),
    ("Flick", "Flick", 0x2000),
    ("FallInCrack", "Fall In Crack", 0x1),
    ("LocustMutate", "Locust Mutate", 0x1),
    ("JumpToDeath", "Jump Off Cliff", 0x20_0000),
    ("Waterwings", "Waterwings", 0x2_2000),
    ("DoubleRainbow1", "Double Rainbow OMG", 0x2000),
    ("DoubleRainbow2", "So Intense", 0x2000),
    ("DoubleRainbow3", "Whooa OMG OMG", 0x2000),
    ("DoubleRainbow4", "Full On Double RB", 0x2000),
    ("DoubleRainbow5", "Whaduzdismean?", 0x2000),
    ("DoubleRainbow6", "Triple RB", 0x2000),
    ("CharlieIdle", "Charlie Stare", 0x1),
    ("CharlieWinning", "Charlie Winning", 0x1),
    ("CharlieDoubleRainbow", "Charlie Rainbow", 0x1),
    ("CharlieImATotalFrickinRockStar", "Charlie RockStar", 0x1),
    ("CharlieItsOnBringIt", "Charlie Bring It", 0x1),
    ("CharlieOneSpeedOneGear", "Charlie One Speed", 0x1),
    ("CharlieTigerBlood", "Charlie Tiger Blood", 0x1),
    ("CharlieBiPolar", "Charlie BiPolar", 0x1),
    ("CharlieVaticanAssasinWarlock", "Charlie Warlock", 0x1),
    ("Sick", "Sick with Plague", 0x1),
    ("Barf", "Barf", 0x1),
    ("Cough", "Cough", 0x1),
    ("Sneeze2", "Sneeze with phlegm", 0x1),
    ("DieFromPlague", "Die From Plague", 0x1),
    ("Suspicious", "Suspicious", 0x1),
    ("CleanHands", "Clean hands.", 0x1),
    ("SprayPurell", "Spray PureHell.", 0x1),
    ("PurificationDeath", "Purification Death.", 0x1),
];

/// The records of `PTR_PTR_1007cc418` (`0x1007cc388` …).
const EVENT_STATES: &StaticStoryStates = &[
    ("Dance", "Dance", 0x1),
    ("Earthquake", "Earthquake", 0x1),
    ("SharkSnap", "Shark Snap", 0x2_2000),
    ("ActivateRainbow", "Activate Rainbow", 0x2000),
    ("ActivateMyRainbow", "Activate My Rainbow", 0x2000),
    ("DeactivateRainbow", "Deactivate Rainbow", 0x2000),
];

/// The records of `PTR_PTR_1007cc5e8` (`0x1007cc450` …), also the location ids of
/// `FUN_100052a5c` and `FUN_100053334`.
const LOCATION_STATES: &StaticStoryStates = &[
    ("SandIsland", "Sand Island", 0x2_3000),
    ("RockIsland", "Rock Island", 0x2_2004),
    ("GraveyardIsland", "Graveyard Island", 0x2_2040),
    ("IceIsland", "Ice Island", 0x202_2000),
    ("ApeIsland", "Ape Mountain w/Altar", 0xa0_2000),
    ("ApeIslandTree", "Ape Mountain w/Tree", 0xa0_2000),
    ("MisfitIsland", "Misfit Island", 0x42_2000),
    ("GodIsland", "God Island", 0x2_2002),
    ("FailBoat", "Fail Boat", 0x2_2100),
    ("ApocalypseIsland", "Apocalypse Island", 0x2_2008),
    ("TimeTemple", "Time Chamber", 0x20_0a00),
    ("LocustTemple", "Locust Chamber", 0x20_0880),
    ("DanceTemple", "Dance Chamber", 0x20_0820),
    ("PlagueTemple", "Plague Chamber", 0x30_0800),
    ("PurificationTemple", "Purification Chamber", 0x120_0800),
    ("ClosetTemple", "Closet Chamber", 0x20_0810),
    ("XenuTemple", "Xenu Chamber", 0x20_8800),
];

/// The records of `PTR_PTR_1007cc6a8` (`0x1007cc678` …).
const WEATHER_STATES: &StaticStoryStates = &[("Clear", "Clear", 0x4000), ("Cloudy", "Cloudy", 0x1_0000)];

/// The records of `PTR_PTR_1007cc708` (`0x1007cc6c0` …).
const TIME_OF_DAY_STATES: &StaticStoryStates = &[
    ("Day", "Day", 0x400),
    ("Night", "Night", 0x8_0000),
    ("Sunset", "Sunset", 0x4_0000),
];

/// `isStoryStateDef:compatibleWithEnvironmentElementDef:`'s masks of the flags each part of
/// the environment can refuse.
const LOCATION_STATE_MASK: u32 = 0x3f2_bbfe;
const WEATHER_STATE_MASK: u32 = 0x1_4000;
const TIME_OF_DAY_STATE_MASK: u32 = 0xc_0400;

/// `StoryManager`'s file-scope globals.
#[derive(Resource, Debug, Default)]
pub struct StoryManagerStatics {
    /// `DAT_1007e4720`: the name records of `createStoryCharacterNameArray` (id and display
    /// name are both the name; the flags stay 0). Records past the current list keep the
    /// names an earlier, longer list wrote.
    pub character_name_records: Vec<String>,

    /// How many entries `DAT_1007e4400` lists before its NULL.
    pub character_name_count: usize,
}

fn static_story_states(array: usize) -> Option<&'static StaticStoryStates> {
    match array {
        STORY_CHARACTER_STATE_ARRAY => Some(CHARACTER_STATES),
        STORY_EVENT_STATE_ARRAY => Some(EVENT_STATES),
        STORY_LOCATION_STATE_ARRAY => Some(LOCATION_STATES),
        STORY_WEATHER_STATE_ARRAY => Some(WEATHER_STATES),
        STORY_TIME_OF_DAY_STATE_ARRAY => Some(TIME_OF_DAY_STATES),
        _ => None,
    }
}

/// The number of entries before the NULL of a `BCStoryStateDef **` list.
fn story_state_array_len(world: &World, array: usize) -> usize {
    if let Some(states) = static_story_states(array) {
        return states.len();
    }
    if array != STORY_CHARACTER_NAME_ARRAY {
        return 0;
    }
    match world.get_resource::<StoryManagerStatics>() {
        Some(statics) => statics.character_name_count,
        None => 0,
    }
}

/// `array[index]` of a `BCStoryStateDef **` list: the entry's `BCStoryStateDef *`, nil at the
/// terminating NULL. The original does not check bounds; past the NULL (and for a nil list)
/// this also gives nil.
pub fn story_state_array_entry(world: &World, array: Option<usize>, index: i32) -> Option<usize> {
    let array = array?;
    let len = story_state_array_len(world, array);
    let in_range = index >= 0 && (index as usize) < len;
    if !in_range {
        return None;
    }
    Some(array * STORY_STATE_ARRAY_STRIDE + index as usize)
}

/// `*state`: the record behind a `BCStoryStateDef *` from [`story_state_array_entry`].
pub fn story_state_def(world: &World, state: usize) -> StoryStateDef {
    let array = state / STORY_STATE_ARRAY_STRIDE;
    let index = state % STORY_STATE_ARRAY_STRIDE;

    if let Some(states) = static_story_states(array) {
        let (id, display_name, flags) = states[index];
        return StoryStateDef {
            id: Some(id.to_owned()),
            display_name: Some(display_name.to_owned()),
            flags,
        };
    }

    // A name record: both strings are the name. A record no list has written yet is zero.
    let statics = world.get_resource::<StoryManagerStatics>();
    let name = match statics {
        Some(statics) => statics.character_name_records.get(index).cloned(),
        None => None,
    };
    StoryStateDef {
        id: name.clone(),
        display_name: name,
        flags: 0,
    }
}

/// `state->flags` (0 for nil).
fn story_state_flags(world: &World, state: Option<usize>) -> u32 {
    match state {
        Some(state) => story_state_def(world, state).flags,
        None => 0,
    }
}

/// The entry of a static list whose id is `[record.id isEqualToString:id]`.
fn find_story_state(world: &World, array: usize, id: Option<&str>) -> Option<usize> {
    let len = story_state_array_len(world, array);
    for index in 0..len {
        let state = array * STORY_STATE_ARRAY_STRIDE + index;
        let record = story_state_def(world, state);
        if ns_string_equal(record.id.as_deref(), id) {
            return Some(state);
        }
    }
    None
}

/// The element `element_def` (nil and deleted elements give `None`).
fn live_element_def(world: &World, element_def: Option<usize>) -> Option<&StoryElementDef> {
    let element_def = element_def?;
    story_def_store(world).get_element_def(element_def)
}

// ---- Lifetime and listener ------------------------------------------------------------------

/// `-[StoryManager init]` @ 0x100048238
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(StoryManager::default());
    world.init_resource::<StoryDefStore>();

    let sequence = pg_engine::sequence::spawn(world, None, Some(this));
    let story_sequence = pg_engine::sequence::spawn(world, None, Some(this));

    let mut manager = world.obj_mut::<StoryManager>(this);
    manager.sequence = Some(sequence);
    manager.story_sequence = Some(story_sequence);
    manager.main_story_def_vector = Vec::new();
    manager.my_list_story_def_vector = Vec::new();
    manager.logic_array = Vec::with_capacity(5);
    manager.story_ui_logic_array = Vec::with_capacity(5);
    manager.story_watch_ui_logic_array = Vec::with_capacity(5);
    manager.story_my_list_ui_logic_array = Vec::with_capacity(5);
    manager.story_create_ui_logic_array = Vec::with_capacity(5);
    manager.story_create_title_ui_logic_array = Vec::with_capacity(5);
    manager.story_create_environment_ui_logic_array = Vec::with_capacity(5);
    manager.story_create_caption_ui_logic_array = Vec::with_capacity(5);
    manager.story_create_character_ui_logic_array = Vec::with_capacity(5);
    manager.story_create_event_ui_logic_array = Vec::with_capacity(5);
    manager.story_confirm_ui_logic_array = Vec::with_capacity(5);
    manager.story_add_menu_ui_logic_array = Vec::with_capacity(5);
    manager.story_overlay_ui_logic_array = Vec::with_capacity(5);
    manager.story_option_menu_ui_logic_array = Vec::with_capacity(5);
    manager.story_option_button_logic_array = Vec::with_capacity(5);
    manager.story_wide_option_menu_ui_logic_array = Vec::with_capacity(5);
    manager.story_wide_option_button_logic_array = Vec::with_capacity(5);
    manager.missing_story_id_array = Vec::with_capacity(5);
    manager.parse_story_xml_array = Vec::with_capacity(5);
}

/// `-[StoryManager id]` @ 0x1000485f0
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some("StoryManager".to_owned())
}

/// `-[StoryManager setStoryManagerListener:]` @ 0x1000485fc
pub fn set_story_manager_listener(world: &mut World, this: Entity, story_manager_listener: Entity) {
    world.obj_mut::<StoryManager>(this).story_manager_listener = Some(story_manager_listener);
}

/// `-[StoryManager dealloc]` @ 0x100048658
pub fn dealloc(world: &mut World, this: Entity) {
    let (main_story_defs, my_list_story_defs, sequence, story_sequence) = {
        let mut manager = world.obj_mut::<StoryManager>(this);
        let main_story_defs = std::mem::take(&mut manager.main_story_def_vector);
        let my_list_story_defs = std::mem::take(&mut manager.my_list_story_def_vector);
        (main_story_defs, my_list_story_defs, manager.sequence, manager.story_sequence)
    };

    {
        let mut store = story_def_store_mut(world);
        for story_def in main_story_defs {
            store.delete_story_def(story_def);
        }
        for story_def in my_list_story_defs {
            store.delete_story_def(story_def);
        }
    }

    if let Some(sequence) = sequence {
        pg_engine::sequence::stop(world, sequence);
        world.despawn(sequence);
    }
    if let Some(story_sequence) = story_sequence {
        pg_engine::sequence::stop(world, story_sequence);
        world.despawn(story_sequence);
    }

    world.despawn(this);
}

/// `-[StoryManager canReceiveCallbacksNow]` @ 0x100048938
pub fn can_receive_callbacks_now(_world: &mut World, _this: Entity) -> bool {
    true
}

// ---- Story files ----------------------------------------------------------------------------

/// `[mStoryManagerListener systemPreferences]` (nil for a nil listener).
fn system_preferences(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<StoryManager>(this).story_manager_listener?;
    crate::controller::system_preferences(world, listener)
}

/// `[[listener systemPreferences] applicationDataFromFile:file_name]`.
fn application_data_from_file(world: &mut World, this: Entity, file_name: &str) -> Option<Vec<u8>> {
    system_preferences(world, this)?;
    pg_engine::prefs::application_data_from_file(world, file_name)
}

/// `[[listener systemPreferences] writeApplicationData:blob toFile:file_name]` once the
/// preferences were asked for (a nil blob writes nothing).
fn write_application_data(world: &mut World, system_preferences: Option<Entity>, blob: Option<Vec<u8>>, file_name: &str) {
    let (Some(_), Some(blob)) = (system_preferences, blob) else {
        return;
    };
    pg_engine::prefs::write_application_data_to_file(world, &blob, file_name);
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

/// `compressBlob` (`FUN_100047ee4`): `compress()` at the default level; nil for empty data.
pub(crate) fn compress_blob(data: &[u8]) -> Option<Vec<u8>> {
    if data.is_empty() {
        return None;
    }
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(data, 6);
    Some(compressed)
}

/// `uncompressBlob` (`FUN_100048000`): inflates a zlib stream; nil for empty data or a stream
/// that fails or does not end.
pub(crate) fn uncompress_blob(data: &[u8]) -> Option<Vec<u8>> {
    if data.is_empty() {
        return None;
    }
    miniz_oxide::inflate::decompress_to_vec_zlib(data).ok()
}

/// `-[StoryManager loadStoryDefs]` @ 0x100048940
pub fn load_story_defs(world: &mut World, this: Entity) {
    let path = pg_engine::prefs::resource_asset_path("story", "xml");
    parse_xml_from_file(world, this, Some(&path));
    load_story_map_xml(world, this);
}

/// `-[StoryManager saveStoryToFile:]` @ 0x1000489bc
pub fn save_story_to_file(world: &mut World, this: Entity, save_story_to_file: Option<usize>) {
    let system_preferences = system_preferences(world, this);
    let Some(story_def) = save_story_to_file else {
        return;
    };

    let xml = story_def_xml(world, story_def);
    let data = unicode_data(&xml);
    let blob = compress_blob(&data);
    let file_name = story_def_file_name(world, story_def);
    write_application_data(world, system_preferences, blob, &file_name);
}

/// `-[StoryManager loadStoryFromFile:]` @ 0x100048d28
pub fn load_story_from_file(world: &mut World, this: Entity, load_story_from_file: &str) -> Option<usize> {
    let data = application_data_from_file(world, this, load_story_from_file);
    if let Some(data) = data {
        let xml_data = uncompress_blob(&data);
        parse_xml_from_data(world, this, xml_data.as_deref());
    }
    world.obj::<StoryManager>(this).last_parsed_story_def
}

/// `-[StoryManager createStoryMapXml]` @ 0x100048da4
pub fn create_story_map_xml(world: &mut World, this: Entity) -> String {
    let mut xml = String::with_capacity(1000);
    xml.push_str("<storyMap>\n");

    let mut index = 0;
    loop {
        let story_def = {
            let manager = world.obj::<StoryManager>(this);
            manager.my_list_story_def_vector.get(index).copied()
        };
        let Some(story_def) = story_def else {
            break;
        };
        let file_name = story_def_file_name(world, story_def);
        xml.push_str(&format!("    <storyMapEntry id=\"{file_name}\"/>\n"));
        index += 1;
    }

    xml.push_str("</storyMap>\n");
    xml
}

/// `-[StoryManager saveStoryMapXml]` @ 0x100048e8c
pub fn save_story_map_xml(world: &mut World, this: Entity) {
    let system_preferences = system_preferences(world, this);
    let xml = create_story_map_xml(world, this);
    let data = unicode_data(&xml);
    let blob = compress_blob(&data);
    write_application_data(world, system_preferences, blob, STORY_MAP_FILE);
}

/// `-[StoryManager loadStoryMapXml]` @ 0x100048f10
pub fn load_story_map_xml(world: &mut World, this: Entity) {
    let Some(data) = application_data_from_file(world, this, STORY_MAP_FILE) else {
        return;
    };
    let xml_data = uncompress_blob(&data);
    parse_xml_from_data(world, this, xml_data.as_deref());

    // The original enumerates the live array (a story file adding to it would raise).
    let story_ids = world.obj::<StoryManager>(this).parse_story_xml_array.clone();
    for story_id in &story_ids {
        load_story_from_file(world, this, story_id);
    }

    world.obj_mut::<StoryManager>(this).parse_story_xml_array.clear();
}

// ---- State lookups ------------------------------------------------------------------------

/// `-[StoryManager countStoryStateArray:]` @ 0x10004ac58
pub fn count_story_state_array(world: &mut World, _this: Entity, count_story_state_array: Option<usize>) -> i32 {
    // PORT: the original dereferences a nil list (no caller passes one).
    let Some(array) = count_story_state_array else {
        return 0;
    };
    story_state_array_len(world, array) as i32
}

/// `-[StoryManager getStoryEnvironmentLocationStateDef:]` @ 0x10004ac6c
pub fn get_story_environment_location_state_def(world: &mut World, _this: Entity, story_environment_location_state_def: Option<usize>) -> Option<usize> {
    let location = live_element_def(world, story_environment_location_state_def)?.location;
    story_state_array_entry(world, Some(STORY_LOCATION_STATE_ARRAY), location)
}

/// `-[StoryManager getStoryEnvironmentWeatherStateDef:]` @ 0x10004ac80
pub fn get_story_environment_weather_state_def(world: &mut World, _this: Entity, story_environment_weather_state_def: Option<usize>) -> Option<usize> {
    let weather = live_element_def(world, story_environment_weather_state_def)?.weather;
    story_state_array_entry(world, Some(STORY_WEATHER_STATE_ARRAY), weather)
}

/// `-[StoryManager getStoryEnvironmentTimeOfDayStateDef:]` @ 0x10004ac94
pub fn get_story_environment_time_of_day_state_def(world: &mut World, _this: Entity, story_environment_time_of_day_state_def: Option<usize>) -> Option<usize> {
    let time_of_day = live_element_def(world, story_environment_time_of_day_state_def)?.time_of_day;
    story_state_array_entry(world, Some(STORY_TIME_OF_DAY_STATE_ARRAY), time_of_day)
}

/// `-[StoryManager getStoryEventStateDef:]` @ 0x10004aca8
pub fn get_story_event_state_def(world: &mut World, _this: Entity, story_event_state_def: Option<usize>) -> Option<usize> {
    let state = live_element_def(world, story_event_state_def)?.state.clone();
    find_story_state(world, STORY_EVENT_STATE_ARRAY, state.as_deref())
}

/// `-[StoryManager getStoryCharacterStateDef:]` @ 0x10004ad10
pub fn get_story_character_state_def(world: &mut World, _this: Entity, story_character_state_def: Option<usize>) -> Option<usize> {
    let state = live_element_def(world, story_character_state_def)?.state.clone();
    find_story_state(world, STORY_CHARACTER_STATE_ARRAY, state.as_deref())
}

/// `-[StoryManager isStoryStateDef:compatibleWithEnvironmentElementDef:]` @ 0x10004ad78
pub fn is_story_state_def(world: &mut World, this: Entity, story_state_def: Option<usize>, compatible_with_environment_element_def: Option<usize>) -> i32 {
    let environment = compatible_with_environment_element_def;
    let location = get_story_environment_location_state_def(world, this, environment);
    let weather = get_story_environment_weather_state_def(world, this, environment);
    let time_of_day = get_story_environment_time_of_day_state_def(world, this, environment);

    // PORT: the original dereferences a nil state or environment (an unknown state id crashes
    // it); nil reads as no flags here.
    let state_flags = story_state_flags(world, story_state_def);
    let location_flags = story_state_flags(world, location);
    let weather_flags = story_state_flags(world, weather);
    let time_of_day_flags = story_state_flags(world, time_of_day);

    let location_needs = state_flags & LOCATION_STATE_MASK;
    if location_needs & location_flags != location_needs {
        return 1;
    }
    let weather_needs = state_flags & WEATHER_STATE_MASK;
    if weather_flags & weather_needs != weather_needs {
        return 2;
    }
    let time_of_day_needs = state_flags & TIME_OF_DAY_STATE_MASK;
    if time_of_day_flags & time_of_day_needs != time_of_day_needs {
        return 3;
    }
    0
}

/// `-[StoryManager createStoryCharacterNameArray]` @ 0x10004cf50
pub fn create_story_character_name_array(world: &mut World, this: Entity) -> Option<usize> {
    // PORT: the names go through an `NSMutableDictionary`, so the original lists them in its
    // hash order; this keeps the order they were added in (docs/re/managers-platform.md §20).
    let mut names: Vec<String> = Vec::with_capacity(5);

    let listener = world.obj::<StoryManager>(this).story_manager_listener;
    let tribe_persona = match listener {
        Some(listener) => crate::controller::tribe_persona(world, listener),
        None => None,
    };
    // `[tribePersona pygmyPersonaArray]` and `[persona name]` return their ivars.
    let tribe = match tribe_persona {
        Some(tribe_persona) => world.get::<TribePersona>(tribe_persona),
        None => None,
    };
    let pygmy_personas = match tribe {
        Some(tribe) => tribe.pygmy_persona_array.clone(),
        None => Vec::new(),
    };
    for pygmy_persona in pygmy_personas {
        // A nil name raises in the original (`setObject:forKey:`); every pygmy has one.
        let persona = world.get::<Persona>(pygmy_persona);
        let name = persona.and_then(|persona| persona.name.clone());
        if let Some(name) = name {
            add_dictionary_key(&mut names, name);
        }
    }

    for fixed_name in ["OOGA", "BOOGA", "KLIK", "KLAK", "DOOBY", "NOOBY"] {
        add_dictionary_key(&mut names, fixed_name.to_owned());
    }

    let active_story_def = world.obj::<StoryManager>(this).active_story_def;
    if let Some(active_story_def) = active_story_def {
        let store = story_def_store(world);
        let element_defs = store.story_def(active_story_def).element_defs.clone();
        for element_def in element_defs {
            let element = store.element_def(element_def);
            if element.element_type != STORY_CHARACTER_ELEMENT {
                continue;
            }
            let Some(name) = element.name.as_deref() else {
                continue;
            };
            if name.is_empty() {
                continue;
            }
            add_dictionary_key(&mut names, name.to_owned());
        }
    }

    // Entry 0 is "[NEW NAME]", then the names while the index stays below 100.
    let mut statics = world.get_resource_or_insert_with(StoryManagerStatics::default);
    write_character_name_record(&mut statics, 0, NEW_NAME);
    let mut count = 1;
    for name in &names {
        if count > 99 {
            break;
        }
        write_character_name_record(&mut statics, count, name);
        count += 1;
    }
    statics.character_name_count = count;

    Some(STORY_CHARACTER_NAME_ARRAY)
}

/// `[dictionary setObject:key forKey:key]`: a new key is added, an existing one stays put.
fn add_dictionary_key(keys: &mut Vec<String>, key: String) {
    if keys.contains(&key) {
        return;
    }
    keys.push(key);
}

/// Record `index` of `DAT_1007e4720` (both strings = `name`).
fn write_character_name_record(statics: &mut StoryManagerStatics, index: usize, name: &str) {
    let records = &mut statics.character_name_records;
    if records.len() <= index {
        records.resize(index + 1, String::new());
    }
    records[index] = name.to_owned();
}

// ---- Cleanup ------------------------------------------------------------------------------

/// `-[StoryManager deleteActiveStoryDef]` @ 0x10004dae8
pub fn delete_active_story_def(world: &mut World, this: Entity) {
    let (active_story_def, position) = {
        let manager = world.obj::<StoryManager>(this);
        let active_story_def = manager.active_story_def;
        let position = manager
            .my_list_story_def_vector
            .iter()
            .position(|story_def| Some(*story_def) == active_story_def);
        (active_story_def, position)
    };

    if let (Some(active_story_def), Some(position)) = (active_story_def, position) {
        world.obj_mut::<StoryManager>(this).my_list_story_def_vector.remove(position);

        let system_preferences = system_preferences(world, this);
        let file_name = story_def_file_name(world, active_story_def);
        if system_preferences.is_some() {
            pg_engine::prefs::remove_file(world, &file_name);
        }

        save_story_map_xml(world, this);
        story_def_store_mut(world).delete_story_def(active_story_def);
    }

    world.obj_mut::<StoryManager>(this).active_story_def = None;
}

/// `-[StoryManager checkForUnmodifiedStoryDef]` @ 0x10004dc18
pub fn check_for_unmodified_story_def(world: &mut World, this: Entity) {
    let (active_story_def, template_story_def) = {
        let manager = world.obj::<StoryManager>(this);
        (manager.active_story_def, manager.template_story_def)
    };
    let Some(active_story_def) = active_story_def else {
        return;
    };

    let unmodified_clone = match template_story_def {
        Some(template_story_def) => is_story_def_equal(world, template_story_def, active_story_def),
        None => false,
    };
    world.obj_mut::<StoryManager>(this).template_story_def = None;

    let untouched_new_story = {
        let store = story_def_store(world);
        let story = store.story_def(active_story_def);
        let default_title = ns_string_equal(story.title.as_deref(), Some(ENTER_TITLE));
        default_title && (story.element_defs.len() as i32) < 2
    };

    if unmodified_clone || untouched_new_story {
        delete_active_story_def(world, this);
    }
}

// ---- Story XML parser ---------------------------------------------------------------------

/// `NSXMLParser`'s encoding detection for the story files: UTF-16 with a byte-order mark (as
/// `saveStoryToFile:` writes them), otherwise UTF-8.
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
fn parse_story_xml(world: &mut World, this: Entity, data: &[u8]) {
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
                // An empty `<![CDATA[]]>` is reported too: the built-in stories use it for a
                // silent character (docs/re/managers-platform.md §20).
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

/// `-[StoryManager parseXmlFromData:]` @ 0x10005213c
pub fn parse_xml_from_data(world: &mut World, this: Entity, parse_xml_from_data: Option<&[u8]>) {
    // A parser made from nil data parses nothing.
    let Some(data) = parse_xml_from_data else {
        return;
    };
    parse_story_xml(world, this, data);
}

/// `-[StoryManager parseXmlFromFile:]` @ 0x1000521dc
pub fn parse_xml_from_file(world: &mut World, this: Entity, parse_xml_from_file: Option<&str>) {
    // The bundle is the embedded asset set; a missing file parses nothing.
    let Some(path) = parse_xml_from_file else {
        return;
    };
    let Some(data) = pg_assets::file(path) else {
        return;
    };
    parse_story_xml(world, this, data);
}

/// `-[StoryManager parserDidStartDocument:]` @ 0x100052294
pub fn parser_did_start_document(world: &mut World, this: Entity) {
    world.obj_mut::<StoryManager>(this).last_parsed_story_def = None;
}

/// `-[StoryManager parserDidEndDocument:]` @ 0x1000522a4
pub fn parser_did_end_document(_world: &mut World, _this: Entity) {}

/// `[[attributes objectForKey:key] floatValue]` (0 for a missing attribute).
fn attribute_float_value(attributes: &HashMap<String, String>, key: &str) -> f32 {
    match attributes.get(key) {
        Some(value) => ns_float_value(value),
        None => 0.0,
    }
}

/// `FUN_10004fdd8(mpParsedStoryDef, element_def)`. Outside a `story` element the original
/// dereferences NULL; the element is left out here.
fn append_to_parsed_story_def(world: &mut World, this: Entity, element_def: Option<usize>) {
    let parsed_story_def = world.obj::<StoryManager>(this).parsed_story_def;
    let (Some(parsed_story_def), Some(element_def)) = (parsed_story_def, element_def) else {
        return;
    };
    append_story_element_def(world, parsed_story_def, element_def);
}

/// `story`: a new story; built-ins (no `date`) go to the Watch list, the others replace the
/// My List story of the same file name or are appended to My List.
fn parse_story_element(world: &mut World, this: Entity, attributes: &HashMap<String, String>) {
    let story_def = story_def_store_mut(world).insert_story_def(StoryDef::new());
    world.obj_mut::<StoryManager>(this).parsed_story_def = Some(story_def);

    let Some(date) = attributes.get("date") else {
        let mut manager = world.obj_mut::<StoryManager>(this);
        manager.main_story_def_vector.push(story_def);
        manager.last_parsed_story_def = Some(story_def);
        return;
    };
    set_story_def_date_from_string(world, story_def, Some(date.as_str()));

    let mut index = 0;
    loop {
        let existing_story_def = {
            let manager = world.obj::<StoryManager>(this);
            manager.my_list_story_def_vector.get(index).copied()
        };
        let Some(existing_story_def) = existing_story_def else {
            break;
        };

        let file_name = story_def_file_name(world, story_def);
        let existing_file_name = story_def_file_name(world, existing_story_def);
        if file_name == existing_file_name {
            {
                let mut manager = world.obj_mut::<StoryManager>(this);
                manager.my_list_story_def_vector[index] = story_def;
                manager.last_parsed_story_def = Some(story_def);
                if manager.active_story_def == Some(existing_story_def) {
                    manager.active_story_def = Some(story_def);
                }
            }
            story_def_store_mut(world).delete_story_def(existing_story_def);
            return;
        }
        index += 1;
    }

    let mut manager = world.obj_mut::<StoryManager>(this);
    manager.my_list_story_def_vector.push(story_def);
    manager.last_parsed_story_def = Some(story_def);
}

/// `storyEnvironment`: a new environment with the `delay`, `location`, `weather` and
/// `timeOfDay` attributes, appended to the story.
fn parse_story_environment_element(world: &mut World, this: Entity, attributes: &HashMap<String, String>) {
    let new_element = StoryElementDef::new_environment();
    let element_def = story_def_store_mut(world).insert_element_def(new_element);
    world.obj_mut::<StoryManager>(this).parsed_story_environment_element_def = Some(element_def);

    let delay = attribute_float_value(attributes, "delay");
    story_def_store_mut(world).element_def_mut(element_def).delay = delay;

    let location = attributes.get("location").map(String::as_str);
    set_location_from_string(world, element_def, location);

    let weather = attributes.get("weather").map(String::as_str);
    if weather == Some("Clear") {
        story_def_store_mut(world).element_def_mut(element_def).weather = 0;
    } else if weather == Some("Cloudy") {
        story_def_store_mut(world).element_def_mut(element_def).weather = 1;
    }

    let time_of_day = attributes.get("timeOfDay").map(String::as_str);
    set_time_of_day_from_string(world, element_def, time_of_day);

    let parsed_element_def = world.obj::<StoryManager>(this).parsed_story_environment_element_def;
    append_to_parsed_story_def(world, this, parsed_element_def);
}

/// `storyCaption`: a new caption with the `delay` attribute (its text follows), appended to
/// the story.
fn parse_story_caption_element(world: &mut World, this: Entity, attributes: &HashMap<String, String>) {
    world.obj_mut::<StoryManager>(this).parse_story_caption = true;

    let new_element = StoryElementDef::new_caption();
    let element_def = story_def_store_mut(world).insert_element_def(new_element);
    world.obj_mut::<StoryManager>(this).parsed_story_caption_element_def = Some(element_def);

    let delay = attribute_float_value(attributes, "delay");
    story_def_store_mut(world).element_def_mut(element_def).delay = delay;

    let parsed_element_def = world.obj::<StoryManager>(this).parsed_story_caption_element_def;
    append_to_parsed_story_def(world, this, parsed_element_def);
}

/// `storyCharacter`: a new character with the `delay` and `state` attributes (name and
/// dialog follow), appended to the story.
fn parse_story_character_element(world: &mut World, this: Entity, attributes: &HashMap<String, String>) {
    let new_element = StoryElementDef::new_character();
    let element_def = story_def_store_mut(world).insert_element_def(new_element);
    world.obj_mut::<StoryManager>(this).parsed_story_character_element_def = Some(element_def);

    let delay = attribute_float_value(attributes, "delay");
    let state = attributes.get("state").cloned();
    {
        let mut store = story_def_store_mut(world);
        let element = store.element_def_mut(element_def);
        element.delay = delay;
        element.state = state;
    }

    let parsed_element_def = world.obj::<StoryManager>(this).parsed_story_character_element_def;
    append_to_parsed_story_def(world, this, parsed_element_def);
}

/// `storyEvent`: a new event with the `delay` and `state` attributes, appended to the story.
fn parse_story_event_element(world: &mut World, this: Entity, attributes: &HashMap<String, String>) {
    let new_element = StoryElementDef::new_event();
    let element_def = story_def_store_mut(world).insert_element_def(new_element);
    world.obj_mut::<StoryManager>(this).parsed_story_event_element_def = Some(element_def);

    let delay = attribute_float_value(attributes, "delay");
    let state = attributes.get("state").cloned();
    {
        let mut store = story_def_store_mut(world);
        let element = store.element_def_mut(element_def);
        element.delay = delay;
        element.state = state;
    }

    let parsed_element_def = world.obj::<StoryManager>(this).parsed_story_event_element_def;
    append_to_parsed_story_def(world, this, parsed_element_def);
}

/// `-[StoryManager parser:didStartElement:namespaceURI:qualifiedName:attributes:]` @ 0x1000522a8
pub fn parser_did_start_element_namespace_uri_qualified_name_attributes(world: &mut World, this: Entity, did_start_element: &str, attributes: &HashMap<String, String>) {
    match did_start_element {
        "storyMapEntry" => {
            // `addObject:nil` raises in the original; an entry without an id is skipped.
            if let Some(story_id) = attributes.get("id") {
                let story_id = story_id.clone();
                world.obj_mut::<StoryManager>(this).parse_story_xml_array.push(story_id);
            }
        }
        "story" => parse_story_element(world, this, attributes),
        "storyTitle" => world.obj_mut::<StoryManager>(this).parse_story_title = true,
        "storyEnvironment" => parse_story_environment_element(world, this, attributes),
        "storyCaption" => parse_story_caption_element(world, this, attributes),
        "storyCharacter" => parse_story_character_element(world, this, attributes),
        "storyCharacterName" => world.obj_mut::<StoryManager>(this).parse_story_character_name = true,
        "storyCharacterDialog" => world.obj_mut::<StoryManager>(this).parse_story_character_dialog = true,
        "storyEvent" => parse_story_event_element(world, this, attributes),
        _ => {}
    }
}

/// `-[StoryManager parser:foundCharacters:]` @ 0x100052d40
pub fn parser_found_characters(world: &mut World, this: Entity, found_characters: &str) {
    // The chunk replaces the field (it is not appended); a literal `\n` becomes a newline.
    let text = found_characters.replace("\\n", "\n");

    let manager = world.obj::<StoryManager>(this);
    let parse_story_title = manager.parse_story_title;
    let parse_story_caption = manager.parse_story_caption;
    let parse_story_character_name = manager.parse_story_character_name;
    let parse_story_character_dialog = manager.parse_story_character_dialog;
    let parsed_story_def = manager.parsed_story_def;
    let parsed_caption = manager.parsed_story_caption_element_def;
    let parsed_character = manager.parsed_story_character_element_def;

    // The original writes through the pointer even when it is NULL (a text outside its
    // element); nothing is written here then.
    let mut store = story_def_store_mut(world);
    if parse_story_title {
        if let Some(parsed_story_def) = parsed_story_def {
            store.story_def_mut(parsed_story_def).title = Some(text);
        }
    } else if parse_story_caption {
        if let Some(parsed_caption) = parsed_caption {
            store.element_def_mut(parsed_caption).caption = Some(text);
        }
    } else if parse_story_character_name {
        if let Some(parsed_character) = parsed_character {
            store.element_def_mut(parsed_character).name = Some(text);
        }
    } else if parse_story_character_dialog {
        if let Some(parsed_character) = parsed_character {
            store.element_def_mut(parsed_character).dialog = Some(text);
        }
    }
}

/// `-[StoryManager parser:didEndElement:namespaceURI:qualifiedName:]` @ 0x100052e84
pub fn parser_did_end_element_namespace_uri_qualified_name(world: &mut World, this: Entity, did_end_element: &str) {
    let mut manager = world.obj_mut::<StoryManager>(this);
    match did_end_element {
        "story" => manager.parsed_story_def = None,
        "storyTitle" => manager.parse_story_title = false,
        "storyEnvironment" => manager.parsed_story_environment_element_def = None,
        "storyCaption" => {
            manager.parse_story_caption = false;
            manager.parsed_story_caption_element_def = None;
        }
        "storyCharacter" => manager.parsed_story_character_element_def = None,
        "storyCharacterName" => manager.parse_story_character_name = false,
        "storyCharacterDialog" => manager.parse_story_character_dialog = false,
        "storyEvent" => manager.parsed_story_event_element_def = None,
        _ => {}
    }
}

/// `-[StoryManager parser:parseErrorOccurred:]` @ 0x100052ff0
pub fn parser_parse_error_occurred(_world: &mut World, _this: Entity) {
    // The original only logs the error, its line and its column.
}
