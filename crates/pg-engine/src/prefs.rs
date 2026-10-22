//! `BCPreferences` and `BCKeyChain`: the persistent key-value documents, the keychain, the file
//! helpers other managers use, and server downloads (`docs/re/audio-prefs-text.md` §2).
//!
//! A document is an object (entity) holding a property-list dictionary. Every change marks it
//! dirty. Once loaded it is a game-frame listener (order 40) that writes a dirty dictionary at
//! most every 2 s of game time: to `Documents/<Id>.json`, or for a secure document to the
//! keychain item `PocketGod` / `Preferences`. The keychain is a separate document,
//! `Documents/KeyChain.json`, holding every generic-password item the game uses.
//!
//! Storage goes through [`PreferencesHooks`] (bytes in, bytes out), which the platform layer
//! installs; without them files live in memory ([`MemoryStorage`]). Documents are JSON instead
//! of the original's binary / XML plists, so iOS saves are not read (the legacy-file fallback
//! keeps its place but only finds files in this format). iCloud (`UIDocument`, metadata
//! queries, conflict versions) does not exist here: `isICloudAvailable` is NO, which is the
//! original's own path whenever iCloud is off.
//!
//! In-app purchases are free in this port: the keychain accounts of the skin-pack products
//! always read as bought.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

use bevy_ecs::prelude::*;
use serde_json::{Map, Value};

use crate::object::{Class, Isa, ObjectWorldExt};
use crate::sequence::{ns_float_value, ns_int_value};
use crate::view;

/// `-[BCPreferences id]` @ 0x10016b3e0.
pub const PREFERENCES_CLASS_ID: &str = "BCPreferences";

/// `-[BCPreferences gameFrameOrder]` @ 0x10016e7a4.
pub const PREFERENCES_GAME_FRAME_ORDER: i32 = 40;

/// Game time between two saves of a dirty document (`onGameFrameManageSavingPreferences:`).
const SAVE_INTERVAL: f32 = 2.0;

/// The entry `initPreferences` puts in a fresh dictionary (`@"TEST"` → `@"TEST"`).
const INIT_PREFERENCES_ENTRY: &str = "TEST";

/// Extension of a document's file (`"%@.plist"` in the original, which wrote binary plists).
const PREFERENCE_FILE_EXTENSION: &str = "json";

/// Keychain service of every item the game uses (`@"PocketGod"`).
pub const KEY_CHAIN_APP_NAME: &str = "PocketGod";

/// Keychain account of the secure document (`loadSecurePreferences` @ 0x10016cfe0).
pub const SECURE_PREFERENCES_KEY: &str = "Preferences";

/// `kSecAttrGeneric` of a keychain item nobody has set (`@"THISISTHEUNSETSECRET"`).
pub const UNSET_SECRET: &str = "THISISTHEUNSETSECRET";

/// The secret the skin-pack store writes for a bought pack (`@"BabaBooey"`).
pub const PURCHASED_SECRET: &str = "BabaBooey";

/// Accounts of the skin-pack purchases: `<Pack>SkinPack` (table at 0x10037d620).
const SKIN_PACK_ACCOUNT_SUFFIX: &str = "SkinPack";

/// The keychain document.
const KEY_CHAIN_FILE_NAME: &str = "KeyChain.json";

/// `errSecSuccess`.
const ERR_SEC_SUCCESS: i32 = 0;

/// `errSecItemNotFound`.
const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;

/// Version written into the JSON documents.
const STORAGE_FORMAT_VERSION: u64 = 1;

/// A stored value. Documents hold only what the game stores: `NSNumber`s made with
/// `numberWithInt:` or `numberWithFloat:`, `NSString`s, and dictionaries of string → int.
#[derive(Clone, Debug, PartialEq)]
pub enum PrefValue {
    Int(i32),
    Float(f32),
    String(String),
    Dict(BTreeMap<String, i32>),
}

impl PrefValue {
    /// `-intValue`. A dictionary does not respond to it (the original raises); 0 here.
    pub fn int_value(&self) -> i32 {
        match self {
            PrefValue::Int(value) => *value,
            PrefValue::Float(value) => *value as i32,
            PrefValue::String(text) => ns_int_value(text),
            PrefValue::Dict(_) => 0,
        }
    }

    /// `-floatValue`. A dictionary does not respond to it (the original raises); 0 here.
    pub fn float_value(&self) -> f32 {
        match self {
            PrefValue::Int(value) => *value as f32,
            PrefValue::Float(value) => *value,
            PrefValue::String(text) => ns_float_value(text),
            PrefValue::Dict(_) => 0.0,
        }
    }

    /// The text, when the object is an `NSString`.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PrefValue::String(text) => Some(text),
            _ => None,
        }
    }

    /// The entries, when the object is an `NSDictionary`.
    pub fn as_dictionary(&self) -> Option<&BTreeMap<String, i32>> {
        match self {
            PrefValue::Dict(entries) => Some(entries),
            _ => None,
        }
    }
}

/// The listener of `loadDataFromServer:listenerData:url:` with its two callbacks,
/// `onLoadDataFromServerComplete:url:data:` and `onLoadDataFromServerError:url:error:`, each
/// called with the listener object, its listener data, the url and the data / error text.
#[derive(Clone, Copy, Debug)]
pub struct ServerLoadListener {
    pub object: Entity,
    pub on_complete: fn(&mut World, Entity, Option<u64>, &str, &[u8]),
    pub on_error: fn(&mut World, Entity, Option<u64>, &str, &str),
}

/// `BCLoadInstance`: one queued download.
#[derive(Clone, Debug)]
pub struct LoadInstance {
    pub listener: ServerLoadListener,

    /// The listener's `void *` (`NewsManager` passes its `BCNewsDef`, NULL for the feed).
    pub listener_data: Option<u64>,
    pub url: String,
}

/// `BCPreferences` ivars. iCloud's `mPreferenceICloudFileName`, `mbICloudEnabled`, `mQuery`
/// and `mPreferencesDocument` have no counterpart: iCloud is never available here.
#[derive(Component, Clone, Debug, Default)]
pub struct BcPreferences {
    /// `BCPreferencesListener`: only iCloud updates ever notify it.
    pub preferences_listener: Option<Entity>,
    pub preference_id: String,

    /// `Documents/<Id>.json`; empty for a document made with plain `init`.
    pub preference_file_name: String,

    /// Read when the document's own file does not exist.
    pub legacy_preferences_file_name: Option<String>,

    /// The data; `None` (nil) until loaded.
    pub preference_dictionary: Option<BTreeMap<String, PrefValue>>,

    /// The snapshot a secure save writes (the original hands it to a background thread).
    pub preference_dictionary_for_thread: Option<BTreeMap<String, PrefValue>>,

    /// The keychain item of a secure document, set by `loadSecurePreferences`.
    pub key_chain: Option<Entity>,
    pub preferences_load_complete: bool,
    pub icloud_preferences: bool,
    pub secure_preferences: bool,

    /// Dirty: written at the next save tick.
    pub save_preferences: bool,
    pub waiting_to_save: bool,
    pub last_save_time: f32,

    /// `mpLoadInstanceVector`: queued downloads, the first one in flight.
    pub load_instances: Vec<LoadInstance>,

    /// `mUrlConnection != nil`.
    pub url_connection: bool,
    pub received_data: Option<Vec<u8>>,

    /// The download in flight finished; delivered on the next game frame.
    pub loaded: bool,
    pub debug: bool,
}

pub const BC_PREFERENCES: Class = Class {
    name: "BCPreferences",
    superclass: None,
    game_frame_order: PREFERENCES_GAME_FRAME_ORDER,
    on_game_frame,
    id: class_id,
    ..Class::NS_OBJECT
};

fn class_id(_world: &World, _this: Entity) -> Option<String> {
    Some(PREFERENCES_CLASS_ID.to_owned())
}

/// `BCKeyChain` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BcKeyChain {
    /// `kSecAttrService`.
    pub app_name: String,

    /// `kSecAttrAccount`.
    pub key: String,

    /// `kSecAttrGeneric` as read at creation or last set; `reset` leaves it alone.
    pub secret: String,

    /// The item could not be stored.
    pub error: bool,
}

pub const BC_KEY_CHAIN: Class = Class {
    name: "BCKeyChain",
    superclass: None,
    ..Class::NS_OBJECT
};

/// The keychain: `BCKeyChain`'s object cache (`DAT_1007e3028`) and the stored items.
#[derive(Resource, Default)]
pub struct KeyChains {
    /// `"<app><key>"` → the cached object.
    pub objects: HashMap<String, Entity>,

    /// Stored generic-password items: service → account → `kSecAttrGeneric`.
    pub items: BTreeMap<String, BTreeMap<String, String>>,

    /// `items` were read from storage.
    pub items_loaded: bool,
}

/// The original's storage directories.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StorageDir {
    /// `NSDocumentDirectory`: preference documents, the keychain, saved stories.
    Documents,

    /// `NSCachesDirectory`: the news feed and its icons, excluded from backups.
    Caches,
}

/// Platform services behind `BCPreferences`, installed by the platform layer (files on desktop,
/// internal storage on Android, `localStorage` on the web). Calls are synchronous, like the
/// original's `NSData` / `NSFileManager` calls. The default keeps files in [`MemoryStorage`]
/// and has no network.
#[derive(Resource, Clone, Copy)]
pub struct PreferencesHooks {
    /// The whole file, `None` when it does not exist (`-[NSData initWithContentsOfFile:]`).
    pub read_file: fn(&mut World, StorageDir, &str) -> Option<Vec<u8>>,

    /// Replaces the whole file atomically; false on failure (`writeToFile:atomically:YES`).
    pub write_file: fn(&mut World, StorageDir, &str, &[u8]) -> bool,

    /// Deletes the file if it exists (`removeItemAtPath:error:`).
    pub remove_file: fn(&mut World, StorageDir, &str),

    /// `fileExistsAtPath:`.
    pub file_exists: fn(&mut World, StorageDir, &str) -> bool,

    /// Starts downloading the url for the document (`initWithRequest:delegate:`); false when
    /// no connection could be made. The platform reports progress later (never from inside
    /// this call) with [`connection_did_receive_response`], [`connection_did_receive_data`],
    /// [`connection_did_finish_loading`] and [`connection_did_fail_with_error`].
    pub start_download: fn(&mut World, Entity, &str) -> bool,
}

/// Files of the default [`PreferencesHooks`]: nothing survives a restart.
#[derive(Resource, Default)]
pub struct MemoryStorage {
    pub files: BTreeMap<(StorageDir, String), Vec<u8>>,
}

fn memory_read_file(world: &mut World, dir: StorageDir, name: &str) -> Option<Vec<u8>> {
    let storage = world.get_resource_or_init::<MemoryStorage>();
    let key = (dir, name.to_owned());
    storage.files.get(&key).cloned()
}

fn memory_write_file(world: &mut World, dir: StorageDir, name: &str, data: &[u8]) -> bool {
    let mut storage = world.get_resource_or_init::<MemoryStorage>();
    let key = (dir, name.to_owned());
    storage.files.insert(key, data.to_vec());
    true
}

fn memory_remove_file(world: &mut World, dir: StorageDir, name: &str) {
    let mut storage = world.get_resource_or_init::<MemoryStorage>();
    let key = (dir, name.to_owned());
    storage.files.remove(&key);
}

fn memory_file_exists(world: &mut World, dir: StorageDir, name: &str) -> bool {
    let storage = world.get_resource_or_init::<MemoryStorage>();
    let key = (dir, name.to_owned());
    storage.files.contains_key(&key)
}

fn no_download(_world: &mut World, _prefs: Entity, _url: &str) -> bool {
    false
}

impl Default for PreferencesHooks {
    fn default() -> Self {
        Self {
            read_file: memory_read_file,
            write_file: memory_write_file,
            remove_file: memory_remove_file,
            file_exists: memory_file_exists,
            start_download: no_download,
        }
    }
}

fn preferences_hooks(world: &World) -> PreferencesHooks {
    match world.get_resource::<PreferencesHooks>() {
        Some(hooks) => *hooks,
        None => PreferencesHooks::default(),
    }
}

// ---- Serialization -------------------------------------------------------------------------

/// `dataFromPropertyList:format:errorDescription:` of a dictionary: JSON,
/// `{"version": 1, "values": {key: {"int" | "float" | "string" | "dict": …}}}`.
pub fn data_from_dictionary(dictionary: &BTreeMap<String, PrefValue>) -> Vec<u8> {
    let mut values = Map::new();
    for (key, value) in dictionary {
        let encoded = value_to_json(value);
        values.insert(key.clone(), encoded);
    }

    let mut root = Map::new();
    root.insert("version".to_owned(), Value::from(STORAGE_FORMAT_VERSION));
    root.insert("values".to_owned(), Value::Object(values));
    let document = Value::Object(root);
    document.to_string().into_bytes()
}

/// `propertyListFromData:mutabilityOption:format:errorDescription:` of a dictionary; `None`
/// when the data is not a document.
pub fn dictionary_from_data(data: &[u8]) -> Option<BTreeMap<String, PrefValue>> {
    let document: Value = serde_json::from_slice(data).ok()?;
    let values = document.get("values")?.as_object()?;

    let mut dictionary = BTreeMap::new();
    for (key, encoded) in values {
        let value = value_from_json(encoded)?;
        dictionary.insert(key.clone(), value);
    }
    Some(dictionary)
}

fn value_to_json(value: &PrefValue) -> Value {
    let (tag, content) = match value {
        PrefValue::Int(int) => ("int", Value::from(*int)),
        PrefValue::Float(float) => ("float", float_to_json(*float)),
        PrefValue::String(text) => ("string", Value::from(text.as_str())),
        PrefValue::Dict(entries) => ("dict", counts_to_json(entries)),
    };
    let mut tagged = Map::new();
    tagged.insert(tag.to_owned(), content);
    Value::Object(tagged)
}

fn value_from_json(encoded: &Value) -> Option<PrefValue> {
    let tagged = encoded.as_object()?;
    if tagged.len() != 1 {
        return None;
    }
    let (tag, content) = tagged.iter().next()?;
    match tag.as_str() {
        "int" => {
            let int = int_from_json(content)?;
            Some(PrefValue::Int(int))
        }
        "float" => {
            let float = float_from_json(content)?;
            Some(PrefValue::Float(float))
        }
        "string" => {
            let text = content.as_str()?;
            Some(PrefValue::String(text.to_owned()))
        }
        "dict" => {
            let entries = counts_from_json(content)?;
            Some(PrefValue::Dict(entries))
        }
        _ => None,
    }
}

fn int_from_json(content: &Value) -> Option<i32> {
    let wide = content.as_i64()?;
    i32::try_from(wide).ok()
}

/// Finite floats are JSON numbers (the f32 widened exactly); the others are named.
fn float_to_json(float: f32) -> Value {
    if float.is_nan() {
        return Value::from("nan");
    }
    if float == f32::INFINITY {
        return Value::from("inf");
    }
    if float == f32::NEG_INFINITY {
        return Value::from("-inf");
    }
    let widened = f64::from(float);
    Value::from(widened)
}

fn float_from_json(content: &Value) -> Option<f32> {
    if let Some(name) = content.as_str() {
        return match name {
            "nan" => Some(f32::NAN),
            "inf" => Some(f32::INFINITY),
            "-inf" => Some(f32::NEG_INFINITY),
            _ => None,
        };
    }
    let wide = content.as_f64()?;
    Some(wide as f32)
}

fn counts_to_json(entries: &BTreeMap<String, i32>) -> Value {
    let mut object = Map::new();
    for (key, count) in entries {
        object.insert(key.clone(), Value::from(*count));
    }
    Value::Object(object)
}

fn counts_from_json(content: &Value) -> Option<BTreeMap<String, i32>> {
    let object = content.as_object()?;
    let mut entries = BTreeMap::new();
    for (key, encoded) in object {
        let count = int_from_json(encoded)?;
        entries.insert(key.clone(), count);
    }
    Some(entries)
}

/// The document as ASCII text, the form the original stores in the keychain (an XML plist read
/// with `NSASCIIStringEncoding`): characters beyond ASCII become JSON `\u` escapes.
fn ascii_document_text(dictionary: &BTreeMap<String, PrefValue>) -> String {
    let data = data_from_dictionary(dictionary);
    let text = String::from_utf8_lossy(&data);
    let mut ascii = String::with_capacity(text.len());
    for character in text.chars() {
        if character.is_ascii() {
            ascii.push(character);
            continue;
        }
        let mut units = [0u16; 2];
        for unit in character.encode_utf16(&mut units) {
            let _ = write!(ascii, "\\u{unit:04x}");
        }
    }
    ascii
}

fn data_from_key_chain_items(items: &BTreeMap<String, BTreeMap<String, String>>) -> Vec<u8> {
    let mut services = Map::new();
    for (service, accounts) in items {
        let mut secrets = Map::new();
        for (account, secret) in accounts {
            secrets.insert(account.clone(), Value::from(secret.as_str()));
        }
        services.insert(service.clone(), Value::Object(secrets));
    }

    let mut root = Map::new();
    root.insert("version".to_owned(), Value::from(STORAGE_FORMAT_VERSION));
    root.insert("items".to_owned(), Value::Object(services));
    let document = Value::Object(root);
    document.to_string().into_bytes()
}

fn key_chain_items_from_data(data: &[u8]) -> Option<BTreeMap<String, BTreeMap<String, String>>> {
    let document: Value = serde_json::from_slice(data).ok()?;
    let services = document.get("items")?.as_object()?;

    let mut items = BTreeMap::new();
    for (service, encoded_accounts) in services {
        let accounts = encoded_accounts.as_object()?;
        let mut secrets = BTreeMap::new();
        for (account, secret) in accounts {
            let text = secret.as_str()?;
            secrets.insert(account.clone(), text.to_owned());
        }
        items.insert(service.clone(), secrets);
    }
    Some(items)
}

/// `-[BCPreferences mergeDictionary:fromDictionary:]` @ 0x10016b850 (iCloud conflict
/// versions): keys missing locally are copied, strings keep the local value, numbers keep the
/// larger (`compare:` is `NSOrderedDescending`) and dictionaries merge the same way.
pub fn merge_dictionary(local: &mut BTreeMap<String, PrefValue>, other: &BTreeMap<String, PrefValue>) {
    for (key, other_value) in other {
        let Some(local_value) = local.get_mut(key) else {
            local.insert(key.clone(), other_value.clone());
            continue;
        };
        match other_value {
            PrefValue::String(_) => {}
            PrefValue::Int(_) | PrefValue::Float(_) => {
                if number_is_greater(other_value, local_value) {
                    *local_value = other_value.clone();
                }
            }
            PrefValue::Dict(other_entries) => {
                // The original replaces the local value by a mutable copy of it and merges
                // into that; anything but a dictionary there raises.
                if let PrefValue::Dict(local_entries) = local_value {
                    merge_counts(local_entries, other_entries);
                }
            }
        }
    }
}

fn merge_counts(local: &mut BTreeMap<String, i32>, other: &BTreeMap<String, i32>) {
    for (key, other_count) in other {
        match local.get_mut(key) {
            Some(local_count) => {
                if *other_count > *local_count {
                    *local_count = *other_count;
                }
            }
            None => {
                local.insert(key.clone(), *other_count);
            }
        }
    }
}

/// `[other compare:local] == NSOrderedDescending` between two numbers (a non-number raises in
/// the original; it never wins here).
fn number_is_greater(other: &PrefValue, local: &PrefValue) -> bool {
    let other_number = number_value(other);
    let local_number = number_value(local);
    match (other_number, local_number) {
        (Some(other_number), Some(local_number)) => other_number > local_number,
        _ => false,
    }
}

fn number_value(value: &PrefValue) -> Option<f64> {
    match value {
        PrefValue::Int(int) => Some(f64::from(*int)),
        PrefValue::Float(float) => Some(f64::from(*float)),
        _ => None,
    }
}

// ---- BCPreferences --------------------------------------------------------------------------

/// `-[BCPreferences initWithId:secure:iCloud:]` @ 0x10016b2a8.
pub fn spawn(world: &mut World, id: &str, secure: bool, icloud: bool) -> Entity {
    let preferences = BcPreferences {
        preference_id: id.to_owned(),
        preference_file_name: format!("{id}.{PREFERENCE_FILE_EXTENSION}"),
        secure_preferences: secure,
        icloud_preferences: icloud,
        ..Default::default()
    };
    world.spawn((Isa(&BC_PREFERENCES), preferences)).id()
}

/// `[[BCPreferences alloc] init]`: a document with no id or file, which
/// `ChallengeManager::parseMatchData:` @ 0x1000158e4 and `IdolManager::parseMatchData:`
/// @ 0x100032d0c fill with `loadPreferencesFromData:` (turn-based match data).
pub fn spawn_empty(world: &mut World) -> Entity {
    let preferences = BcPreferences::default();
    world.spawn((Isa(&BC_PREFERENCES), preferences)).id()
}

/// `-[BCPreferences setDebug:]` @ 0x10016b3ec.
pub fn set_debug(world: &mut World, prefs: Entity, debug: bool) {
    world.obj_mut::<BcPreferences>(prefs).debug = debug;
}

/// `-[BCPreferences setPreferencesListener:]` @ 0x10016b3fc.
pub fn set_preferences_listener(world: &mut World, prefs: Entity, listener: Option<Entity>) {
    world.obj_mut::<BcPreferences>(prefs).preferences_listener = listener;
}

/// `-[BCPreferences setLegacyPreferenceFileName:]` @ 0x10016b458.
pub fn set_legacy_preference_file_name(world: &mut World, prefs: Entity, file_name: Option<&str>) {
    let legacy = file_name.map(str::to_owned);
    world.obj_mut::<BcPreferences>(prefs).legacy_preferences_file_name = legacy;
}

/// `-[BCPreferences tracePreferences]` @ 0x10016b594: logs the dictionary.
pub fn trace_preferences(world: &World, prefs: Entity) {
    let document = world.obj::<BcPreferences>(prefs);
    let id = &document.preference_id;
    let dictionary = &document.preference_dictionary;
    eprintln!("pg-engine: BCPreferences {id}: {dictionary:?}");
}

/// `-[BCPreferences isICloudSupported]` @ 0x10016b68c: there is no `UIDocument`.
pub fn is_icloud_supported() -> bool {
    false
}

/// `-[BCPreferences isICloudAvailable]` @ 0x10016b70c.
pub fn is_icloud_available() -> bool {
    is_icloud_supported()
}

/// `-[BCPreferences loadPreferences]` @ 0x10016c920. The iCloud branch (a metadata query for
/// `iCloud<Id>.plist`) needs `isICloudAvailable`, which is NO here, so the load completes at
/// once, as in the original whenever iCloud is off.
pub fn load_preferences(world: &mut World, prefs: Entity) {
    on_preferences_load_complete(world, prefs);
}

/// `-[BCPreferences isPreferencesLoadComplete]` @ 0x10016cc48.
pub fn is_preferences_load_complete(world: &World, prefs: Entity) -> bool {
    world.obj::<BcPreferences>(prefs).preferences_load_complete
}

/// `-[BCPreferences onPreferencesLoadComplete]` @ 0x10016cc58: loads the local data unless a
/// dictionary exists, then starts the save timer (a game-frame listener).
pub fn on_preferences_load_complete(world: &mut World, prefs: Entity) {
    let has_dictionary = {
        let mut document = world.obj_mut::<BcPreferences>(prefs);
        document.preferences_load_complete = true;
        document.preference_dictionary.is_some()
    };
    if !has_dictionary {
        load_local_preferences(world, prefs);
    }
    if world.obj::<BcPreferences>(prefs).debug {
        trace_preferences(world, prefs);
    }
    view::add_game_frame_listener(world, prefs);
}

/// `-[BCPreferences initPreferences]` @ 0x10016cd50: a fresh dictionary, marked dirty.
pub fn init_preferences(world: &mut World, prefs: Entity) {
    let mut dictionary = BTreeMap::new();
    let marker = PrefValue::String(INIT_PREFERENCES_ENTRY.to_owned());
    dictionary.insert(INIT_PREFERENCES_ENTRY.to_owned(), marker);

    let mut document = world.obj_mut::<BcPreferences>(prefs);
    document.preference_dictionary = Some(dictionary);
    document.save_preferences = true;
}

/// `-[BCPreferences loadLocalPreferences]` @ 0x10016ce10.
pub fn load_local_preferences(world: &mut World, prefs: Entity) {
    if world.obj::<BcPreferences>(prefs).secure_preferences {
        load_secure_preferences(world, prefs);
    } else {
        load_file_preferences(world, prefs);
    }
}

/// `-[BCPreferences loadFilePreferences]` @ 0x10016ce40: `Documents/<Id>.json`, else the legacy
/// file; neither, or data that does not parse, starts a fresh dictionary.
pub fn load_file_preferences(world: &mut World, prefs: Entity) {
    let (file_name, legacy_file_name) = {
        let document = world.obj::<BcPreferences>(prefs);
        (
            document.preference_file_name.clone(),
            document.legacy_preferences_file_name.clone(),
        )
    };

    let mut data = application_data_from_file(world, &file_name);
    if data.is_none() {
        if let Some(legacy_file_name) = legacy_file_name {
            data = application_data_from_file(world, &legacy_file_name);
        }
    }
    let Some(data) = data else {
        init_preferences(world, prefs);
        return;
    };

    world.obj_mut::<BcPreferences>(prefs).preference_dictionary = None;
    match dictionary_from_data(&data) {
        Some(dictionary) => {
            world.obj_mut::<BcPreferences>(prefs).preference_dictionary = Some(dictionary);
        }
        None => init_preferences(world, prefs),
    }
}

/// `-[BCPreferences loadSecurePreferences]` @ 0x10016cfe0: the document is the secret of the
/// keychain item `PocketGod` / `Preferences`.
pub fn load_secure_preferences(world: &mut World, prefs: Entity) {
    let key_chain = key_chain_with_key_using_app_name(world, KEY_CHAIN_APP_NAME, SECURE_PREFERENCES_KEY);
    world.obj_mut::<BcPreferences>(prefs).key_chain = Some(key_chain);

    if key_chain_has_been_set(world, key_chain) {
        let secret = key_chain_secret(world, key_chain);
        // `dataUsingEncoding:NSASCIIStringEncoding` is nil for other text, which fails to parse.
        let dictionary = if secret.is_ascii() {
            dictionary_from_data(secret.as_bytes())
        } else {
            None
        };
        if let Some(dictionary) = dictionary {
            world.obj_mut::<BcPreferences>(prefs).preference_dictionary = Some(dictionary);
            return;
        }
    }
    init_preferences(world, prefs);
}

/// `-[BCPreferences saveFilePreferences]` @ 0x10016d11c.
pub fn save_file_preferences(world: &mut World, prefs: Entity) {
    let (data, file_name) = {
        let document = world.obj::<BcPreferences>(prefs);
        let data = document.preference_dictionary.as_ref().map(data_from_dictionary);
        (data, document.preference_file_name.clone())
    };
    let Some(data) = data else {
        eprintln!("pg-engine: BCPreferences {file_name}: SavePreferencesError (no dictionary)");
        return;
    };
    write_application_data_to_file(world, &data, &file_name);
}

/// `-[BCPreferences saveSecurePreferences]` @ 0x10016d1d0: writes the snapshot into the
/// keychain item and releases it. The original runs this on a background thread; here it runs
/// at once, so a secure save never waits for an earlier one.
pub fn save_secure_preferences(world: &mut World, prefs: Entity) {
    let (snapshot, key_chain) = {
        let mut document = world.obj_mut::<BcPreferences>(prefs);
        document.waiting_to_save = false;
        let snapshot = document.preference_dictionary_for_thread.take();
        (snapshot, document.key_chain)
    };
    let Some(snapshot) = snapshot else {
        eprintln!("pg-engine: BCPreferences: SaveSecurePreferencesError (no snapshot)");
        return;
    };
    let text = ascii_document_text(&snapshot);
    if let Some(key_chain) = key_chain {
        key_chain_set_secret(world, key_chain, &text);
    }
}

/// `-[BCPreferences flush]` @ 0x10016d344: the next game frame saves a dirty document.
pub fn flush(world: &mut World, prefs: Entity) {
    world.obj_mut::<BcPreferences>(prefs).last_save_time = SAVE_INTERVAL;
}

/// `-[BCPreferences waitingToSave]` @ 0x10016d358 (there is no iCloud document with unsaved
/// changes here).
pub fn waiting_to_save(world: &World, prefs: Entity) -> bool {
    world.obj::<BcPreferences>(prefs).waiting_to_save
}

/// `-[BCPreferences onPreferencesChanged]` @ 0x10016d3b4.
pub fn on_preferences_changed(world: &mut World, prefs: Entity) {
    let mut document = world.obj_mut::<BcPreferences>(prefs);
    document.waiting_to_save = true;
    document.save_preferences = true;
}

/// How a due save writes the document.
enum SaveKind {
    File,
    Secure,
}

/// `-[BCPreferences onGameFrameManageSavingPreferences:]` @ 0x10016d3f8: a dirty document is
/// written once more than 2 s of game time have passed since the last check.
pub fn on_game_frame_manage_saving_preferences(world: &mut World, prefs: Entity, dt: f32) {
    let save = {
        let mut document = world.obj_mut::<BcPreferences>(prefs);
        document.last_save_time += dt;
        let due = SAVE_INTERVAL < document.last_save_time;
        if !due {
            return;
        }
        document.last_save_time = 0.0;
        if !document.save_preferences {
            return;
        }

        if !document.secure_preferences {
            document.waiting_to_save = false;
            document.save_preferences = false;
            SaveKind::File
        } else if document.preference_dictionary_for_thread.is_none() {
            document.save_preferences = false;
            // `[[NSDictionary alloc] initWithDictionary:]`: a shallow copy (empty for nil).
            let snapshot = document.preference_dictionary.clone().unwrap_or_default();
            document.preference_dictionary_for_thread = Some(snapshot);
            SaveKind::Secure
        } else {
            return;
        }
    };

    match save {
        SaveKind::File => save_file_preferences(world, prefs),
        SaveKind::Secure => save_secure_preferences(world, prefs),
    }
}

/// `-[BCPreferences onGameFrame:]` @ 0x10016e7ac: the save timer, then the completion of a
/// finished download.
pub fn on_game_frame(world: &mut World, prefs: Entity, dt: f32) {
    on_game_frame_manage_saving_preferences(world, prefs, dt);

    let (instance, data) = {
        let mut document = world.obj_mut::<BcPreferences>(prefs);
        if !document.loaded {
            return;
        }
        document.loaded = false;
        let instance = document.load_instances.first().cloned();
        let data = document.received_data.clone().unwrap_or_default();
        (instance, data)
    };
    let Some(instance) = instance else {
        return;
    };

    let listener = instance.listener;
    (listener.on_complete)(world, listener.object, instance.listener_data, &instance.url, &data);

    let start_next = {
        let mut document = world.obj_mut::<BcPreferences>(prefs);
        if !document.load_instances.is_empty() {
            document.load_instances.remove(0);
        }
        document.received_data = None;
        !document.load_instances.is_empty()
    };
    if start_next {
        start_next_load_instance(world, prefs);
    }
}

fn object_for_key<'w>(world: &'w World, prefs: Entity, key: &str) -> Option<&'w PrefValue> {
    let dictionary = world.obj::<BcPreferences>(prefs).preference_dictionary.as_ref()?;
    dictionary.get(key)
}

/// `[mPreferenceDictionary setObject:forKey:]` (nothing while the dictionary is nil).
fn set_object_for_key(world: &mut World, prefs: Entity, key: &str, value: PrefValue) {
    let mut document = world.obj_mut::<BcPreferences>(prefs);
    if let Some(dictionary) = document.preference_dictionary.as_mut() {
        dictionary.insert(key.to_owned(), value);
    }
}

/// `[mPreferenceDictionary objectForKey:]`: the stored object, whatever its type.
pub fn get_preference(world: &World, prefs: Entity, key: &str) -> Option<PrefValue> {
    object_for_key(world, prefs, key).cloned()
}

/// `-[BCPreferences removePreference:]` @ 0x10016d504.
pub fn remove_preference(world: &mut World, prefs: Entity, key: &str) {
    {
        let mut document = world.obj_mut::<BcPreferences>(prefs);
        if let Some(dictionary) = document.preference_dictionary.as_mut() {
            dictionary.remove(key);
        }
    }
    on_preferences_changed(world, prefs);
}

/// `-[BCPreferences doesPreferenceExist:]` @ 0x10016d54c.
pub fn does_preference_exist(world: &World, prefs: Entity, key: &str) -> bool {
    object_for_key(world, prefs, key).is_some()
}

/// `-[BCPreferences getIntPreference:]` @ 0x10016d580: `intValue` of the object, 0 when absent.
pub fn get_int_preference(world: &World, prefs: Entity, key: &str) -> i32 {
    match object_for_key(world, prefs, key) {
        Some(value) => value.int_value(),
        None => 0,
    }
}

/// `-[BCPreferences setPreference:asInt:]` @ 0x10016d5c8.
pub fn set_preference_as_int(world: &mut World, prefs: Entity, key: &str, value: i32) {
    set_object_for_key(world, prefs, key, PrefValue::Int(value));
    on_preferences_changed(world, prefs);
}

/// `-[BCPreferences setDefaultPreference:asInt:]` @ 0x10016d644: the stored value's `intValue`
/// when the key exists (nothing is written), else stores and returns the default.
pub fn set_default_preference_as_int(world: &mut World, prefs: Entity, key: &str, value: i32) -> i32 {
    if let Some(existing) = object_for_key(world, prefs, key) {
        return existing.int_value();
    }
    set_preference_as_int(world, prefs, key, value);
    value
}

/// `-[BCPreferences getFloatPreference:]` @ 0x10016d6cc: `floatValue` of the object, 0 when
/// absent.
pub fn get_float_preference(world: &World, prefs: Entity, key: &str) -> f32 {
    match object_for_key(world, prefs, key) {
        Some(value) => value.float_value(),
        None => 0.0,
    }
}

/// `-[BCPreferences setPreference:asFloat:]` @ 0x10016d714.
pub fn set_preference_as_float(world: &mut World, prefs: Entity, key: &str, value: f32) {
    set_object_for_key(world, prefs, key, PrefValue::Float(value));
    on_preferences_changed(world, prefs);
}

/// `-[BCPreferences setDefaultPreference:asFloat:]` @ 0x10016d78c.
pub fn set_default_preference_as_float(world: &mut World, prefs: Entity, key: &str, value: f32) -> f32 {
    if let Some(existing) = object_for_key(world, prefs, key) {
        return existing.float_value();
    }
    set_preference_as_float(world, prefs, key, value);
    value
}

/// `-[BCPreferences getStringPreference:]` @ 0x10016d814: the object when it is a string.
pub fn get_string_preference(world: &World, prefs: Entity, key: &str) -> Option<String> {
    let value = object_for_key(world, prefs, key)?;
    let text = value.as_string()?;
    Some(text.to_owned())
}

/// `-[BCPreferences setPreference:asString:]` @ 0x10016d830.
pub fn set_preference_as_string(world: &mut World, prefs: Entity, key: &str, value: &str) {
    let text = PrefValue::String(value.to_owned());
    set_object_for_key(world, prefs, key, text);
    on_preferences_changed(world, prefs);
}

/// `-[BCPreferences setDefaultPreference:asString:]` @ 0x10016d884: the stored object when the
/// key exists (`None` unless it is a string), else stores and returns the default.
pub fn set_default_preference_as_string(world: &mut World, prefs: Entity, key: &str, value: &str) -> Option<String> {
    if let Some(existing) = object_for_key(world, prefs, key) {
        let text = existing.as_string()?;
        return Some(text.to_owned());
    }
    set_preference_as_string(world, prefs, key, value);
    Some(value.to_owned())
}

/// `-[BCPreferences getDictionaryPreference:]` @ 0x10016d8f8: the object when it is a
/// dictionary.
pub fn get_dictionary_preference(world: &World, prefs: Entity, key: &str) -> Option<BTreeMap<String, i32>> {
    let value = object_for_key(world, prefs, key)?;
    let entries = value.as_dictionary()?;
    Some(entries.clone())
}

/// `-[BCPreferences setPreference:asDictionary:]` @ 0x10016d914: stores a copy; nil is ignored
/// (not even marked changed).
pub fn set_preference_as_dictionary(
    world: &mut World,
    prefs: Entity,
    key: &str,
    value: Option<&BTreeMap<String, i32>>,
) {
    let Some(entries) = value else {
        return;
    };
    let copy = PrefValue::Dict(entries.clone());
    set_object_for_key(world, prefs, key, copy);
    on_preferences_changed(world, prefs);
}

/// `-[BCPreferences setDefaultPreference:asDictionary:]` @ 0x10016d9e0: the stored object when
/// the key exists (`None` unless it is a dictionary), else stores and returns the default (a
/// nil default stores nothing).
pub fn set_default_preference_as_dictionary(
    world: &mut World,
    prefs: Entity,
    key: &str,
    value: Option<&BTreeMap<String, i32>>,
) -> Option<BTreeMap<String, i32>> {
    if let Some(existing) = object_for_key(world, prefs, key) {
        let entries = existing.as_dictionary()?;
        return Some(entries.clone());
    }
    set_preference_as_dictionary(world, prefs, key, value);
    value.cloned()
}

/// `-[BCPreferences loadPreferencesFromData:]` @ 0x10016da54: replaces the dictionary with the
/// document in `data` (empty or missing data gives an empty dictionary, data that does not
/// parse a fresh one) and marks the load complete. No save timer starts.
pub fn load_preferences_from_data(world: &mut World, prefs: Entity, data: Option<&[u8]>) {
    {
        let mut document = world.obj_mut::<BcPreferences>(prefs);
        document.preferences_load_complete = true;
        document.preference_dictionary = Some(BTreeMap::new());
    }
    let Some(data) = data else {
        return;
    };
    if data.is_empty() {
        return;
    }
    let Some(entries) = dictionary_from_data(data) else {
        init_preferences(world, prefs);
        return;
    };
    let mut document = world.obj_mut::<BcPreferences>(prefs);
    if let Some(dictionary) = document.preference_dictionary.as_mut() {
        dictionary.extend(entries);
    }
}

/// `-[BCPreferences dataFromPreferences]` @ 0x10016db90 (`None` while there is no dictionary).
pub fn data_from_preferences(world: &World, prefs: Entity) -> Option<Vec<u8>> {
    let dictionary = world.obj::<BcPreferences>(prefs).preference_dictionary.as_ref()?;
    Some(data_from_dictionary(dictionary))
}

// ---- File helpers ----------------------------------------------------------------------------

/// Where `<name>.<ext>` of the original bundle lives among the converted assets: `.raw` texture
/// pages are `textures/<name>.png`, `.caf` samples `sounds/<name>.flac`, `.xml` files
/// `xml/<name>.xml` and `.bin` files `timelines/<name>.bin`.
pub fn resource_asset_path(name: &str, ext: &str) -> String {
    match ext {
        "raw" => format!("textures/{name}.png"),
        "caf" => {
            let base = name.strip_prefix("sound/").unwrap_or(name);
            format!("sounds/{base}.flac")
        }
        "xml" => format!("xml/{name}.xml"),
        "bin" => format!("timelines/{name}.bin"),
        _ => format!("{name}.{ext}"),
    }
}

/// `-[BCPreferences doesFileExistInResources:ext:]` @ 0x10016dc14: the bundle is the embedded
/// asset set.
pub fn does_file_exist_in_resources(name: &str, ext: &str) -> bool {
    let path = resource_asset_path(name, ext);
    pg_assets::file(&path).is_some()
}

/// `-[BCPreferences applicationDataFromResourceFile:ext:]` @ 0x10016e004: the converted asset
/// (a PNG for a `.raw` page, FLAC for a `.caf` sample). No 1.48.2 code calls it.
pub fn application_data_from_resource_file(name: &str, ext: &str) -> Option<&'static [u8]> {
    let path = resource_asset_path(name, ext);
    pg_assets::file(&path)
}

/// `-[BCPreferences doesFileExistInDocuments:ext:]` @ 0x10016dc90: `Documents/<name>.<ext>`.
pub fn does_file_exist_in_documents(world: &mut World, name: &str, ext: &str) -> bool {
    let file_name = format!("{name}.{ext}");
    let hooks = preferences_hooks(world);
    (hooks.file_exists)(world, StorageDir::Documents, &file_name)
}

/// `-[BCPreferences doesFileExistInLibraryCache:ext:]` @ 0x10016dd88: `Caches/<name>.<ext>`.
pub fn does_file_exist_in_library_cache(world: &mut World, name: &str, ext: &str) -> bool {
    let file_name = format!("{name}.{ext}");
    let hooks = preferences_hooks(world);
    (hooks.file_exists)(world, StorageDir::Caches, &file_name)
}

/// `-[BCPreferences removeFile:]` @ 0x10016de80: `Documents/<file_name>`.
pub fn remove_file(world: &mut World, file_name: &str) {
    let hooks = preferences_hooks(world);
    (hooks.remove_file)(world, StorageDir::Documents, file_name);
}

/// `-[BCPreferences applicationDataFromFile:]` @ 0x10016df3c: `Documents/<file_name>`.
pub fn application_data_from_file(world: &mut World, file_name: &str) -> Option<Vec<u8>> {
    let hooks = preferences_hooks(world);
    (hooks.read_file)(world, StorageDir::Documents, file_name)
}

/// `-[BCPreferences writeApplicationData:toFile:]` @ 0x10016e090: `Documents/<file_name>`,
/// atomically.
pub fn write_application_data_to_file(world: &mut World, data: &[u8], file_name: &str) -> bool {
    let hooks = preferences_hooks(world);
    (hooks.write_file)(world, StorageDir::Documents, file_name, data)
}

/// `-[BCPreferences applicationDataFromLibraryCacheFile:]` @ 0x10016e138: `Caches/<file_name>`.
pub fn application_data_from_library_cache_file(world: &mut World, file_name: &str) -> Option<Vec<u8>> {
    let hooks = preferences_hooks(world);
    (hooks.read_file)(world, StorageDir::Caches, file_name)
}

/// `-[BCPreferences writeApplicationData:toLibraryCacheFile:]` @ 0x10016e200:
/// `Caches/<file_name>`, atomically (the original also excludes it from backups).
pub fn write_application_data_to_library_cache_file(world: &mut World, data: &[u8], file_name: &str) -> bool {
    let hooks = preferences_hooks(world);
    (hooks.write_file)(world, StorageDir::Caches, file_name, data)
}

// ---- Server downloads ------------------------------------------------------------------------

/// `-[BCPreferences loadDataFromServer:listenerData:url:]` @ 0x10016e3dc: queues a download,
/// starting it when the queue was empty. Every call registers the document as a game-frame
/// listener once more, as the original does.
pub fn load_data_from_server(
    world: &mut World,
    prefs: Entity,
    listener: ServerLoadListener,
    listener_data: Option<u64>,
    url: &str,
) {
    view::add_game_frame_listener(world, prefs);

    let instance = LoadInstance {
        listener,
        listener_data,
        url: url.to_owned(),
    };
    let start = {
        let mut document = world.obj_mut::<BcPreferences>(prefs);
        document.load_instances.push(instance);
        let only_one = document.load_instances.len() == 1;
        if only_one {
            document.loaded = false;
        }
        only_one
    };
    if start {
        start_next_load_instance(world, prefs);
    }
}

/// `-[BCPreferences startNextLoadInstance]` @ 0x10016e2fc.
fn start_next_load_instance(world: &mut World, prefs: Entity) {
    let url = {
        let document = world.obj::<BcPreferences>(prefs);
        let Some(instance) = document.load_instances.first() else {
            return;
        };
        instance.url.clone()
    };
    let hooks = preferences_hooks(world);
    let connected = (hooks.start_download)(world, prefs, &url);

    let mut document = world.obj_mut::<BcPreferences>(prefs);
    document.url_connection = connected;
    if connected {
        document.received_data = Some(Vec::new());
    }
}

/// `-[BCPreferences connection:didReceiveResponse:]` @ 0x10016e574: drops data received so far.
pub fn connection_did_receive_response(world: &mut World, prefs: Entity) {
    let mut document = world.obj_mut::<BcPreferences>(prefs);
    if let Some(data) = document.received_data.as_mut() {
        data.clear();
    }
}

/// `-[BCPreferences connection:didReceiveData:]` @ 0x10016e594.
pub fn connection_did_receive_data(world: &mut World, prefs: Entity, data: &[u8]) {
    let mut document = world.obj_mut::<BcPreferences>(prefs);
    if let Some(received) = document.received_data.as_mut() {
        received.extend_from_slice(data);
    }
}

/// `-[BCPreferences connection:didFailWithError:]` @ 0x10016e5b4: tells the listener and drops
/// the download. The next queued download is not started (the queue stalls, as in the
/// original).
pub fn connection_did_fail_with_error(world: &mut World, prefs: Entity, description: &str, failing_url: Option<&str>) {
    let instance = {
        let mut document = world.obj_mut::<BcPreferences>(prefs);
        document.url_connection = false;
        document.received_data = None;
        document.load_instances.first().cloned()
    };
    let Some(instance) = instance else {
        return;
    };

    let failing_url = failing_url.unwrap_or("(null)");
    let error = format!("Connection failed! Error - {description} {failing_url}");
    let listener = instance.listener;
    (listener.on_error)(world, listener.object, instance.listener_data, &instance.url, &error);

    let mut document = world.obj_mut::<BcPreferences>(prefs);
    if !document.load_instances.is_empty() {
        document.load_instances.remove(0);
    }
}

/// `-[BCPreferences connectionDidFinishLoading:]` @ 0x10016e758: the listener hears about it on
/// the next game frame.
pub fn connection_did_finish_loading(world: &mut World, prefs: Entity) {
    let mut document = world.obj_mut::<BcPreferences>(prefs);
    document.url_connection = false;
    document.loaded = true;
}

// ---- BCKeyChain ------------------------------------------------------------------------------

/// In-app purchases are free in this port (`docs/ARCHITECTURE.md`): the skin-pack store's
/// purchase accounts read as bought.
fn is_free_purchase(app_name: &str, key: &str) -> bool {
    app_name == KEY_CHAIN_APP_NAME && key.ends_with(SKIN_PACK_ACCOUNT_SUFFIX)
}

/// Reads the keychain document the first time any item is needed.
fn load_key_chain_items(world: &mut World) {
    if world.get_resource_or_init::<KeyChains>().items_loaded {
        return;
    }
    let hooks = preferences_hooks(world);
    let data = (hooks.read_file)(world, StorageDir::Documents, KEY_CHAIN_FILE_NAME);
    let items = match data {
        Some(data) => key_chain_items_from_data(&data).unwrap_or_default(),
        None => BTreeMap::new(),
    };

    let mut key_chains = world.resource_mut::<KeyChains>();
    key_chains.items = items;
    key_chains.items_loaded = true;
}

/// Writes the keychain document; false when storage failed.
fn save_key_chain_items(world: &mut World) -> bool {
    let data = {
        let key_chains = world.resource::<KeyChains>();
        data_from_key_chain_items(&key_chains.items)
    };
    let hooks = preferences_hooks(world);
    (hooks.write_file)(world, StorageDir::Documents, KEY_CHAIN_FILE_NAME, &data)
}

fn stored_secret(world: &World, app_name: &str, key: &str) -> Option<String> {
    let key_chains = world.resource::<KeyChains>();
    let accounts = key_chains.items.get(app_name)?;
    accounts.get(key).cloned()
}

/// `+[BCKeyChain keyChainWithKeyUsingAppName:key:]` @ 0x10001ef04: one cached object per
/// `"<app><key>"`.
pub fn key_chain_with_key_using_app_name(world: &mut World, app_name: &str, key: &str) -> Entity {
    let cache_key = format!("{app_name}{key}");
    let cached = {
        let key_chains = world.get_resource_or_init::<KeyChains>();
        key_chains.objects.get(&cache_key).copied()
    };
    if let Some(key_chain) = cached {
        return key_chain;
    }
    let key_chain = init_with_key_using_app_name(world, app_name, key);
    world.resource_mut::<KeyChains>().objects.insert(cache_key, key_chain);
    key_chain
}

/// `+[BCKeyChain keyChainWithKeyUsingAppName:key:defaultSecret:]` @ 0x10001f028: sets the
/// default only when the secret is `""`, never the case for a new item (it holds
/// [`UNSET_SECRET`]).
pub fn key_chain_with_key_using_app_name_default_secret(
    world: &mut World,
    app_name: &str,
    key: &str,
    default_secret: &str,
) -> Entity {
    let key_chain = key_chain_with_key_using_app_name(world, app_name, key);
    let secret = key_chain_secret(world, key_chain);
    if secret.is_empty() {
        key_chain_set_secret(world, key_chain, default_secret);
    }
    key_chain
}

/// `-[BCKeyChain initWithKeyUsingAppName:key:]` @ 0x10001f0a8: reads the generic-password item
/// (service `app_name`, account `key`), adding it with the unset secret when it does not exist.
fn init_with_key_using_app_name(world: &mut World, app_name: &str, key: &str) -> Entity {
    load_key_chain_items(world);

    let mut error = false;
    let secret = match stored_secret(world, app_name, key) {
        Some(secret) => secret,
        None => {
            {
                let mut key_chains = world.resource_mut::<KeyChains>();
                let accounts = key_chains.items.entry(app_name.to_owned()).or_default();
                accounts.insert(key.to_owned(), UNSET_SECRET.to_owned());
            }
            // A failed `SecItemAdd` sets `mbError` (the original then returns nil).
            error = !save_key_chain_items(world);
            UNSET_SECRET.to_owned()
        }
    };

    let object = BcKeyChain {
        app_name: app_name.to_owned(),
        key: key.to_owned(),
        secret,
        error,
    };
    world.spawn((Isa(&BC_KEY_CHAIN), object)).id()
}

/// `-[BCKeyChain setSecret:]` @ 0x10001f39c: replaces the secret, then updates the stored item.
/// After `reset` there is no item: the update fails and only this object's secret changes.
pub fn key_chain_set_secret(world: &mut World, key_chain: Entity, secret: &str) -> i32 {
    let (app_name, key) = {
        let mut object = world.obj_mut::<BcKeyChain>(key_chain);
        object.secret = secret.to_owned();
        (object.app_name.clone(), object.key.clone())
    };
    load_key_chain_items(world);

    let updated = {
        let mut key_chains = world.resource_mut::<KeyChains>();
        let stored = match key_chains.items.get_mut(&app_name) {
            Some(accounts) => accounts.get_mut(&key),
            None => None,
        };
        match stored {
            Some(stored) => {
                *stored = secret.to_owned();
                true
            }
            None => false,
        }
    };
    if updated {
        save_key_chain_items(world);
    } else {
        eprintln!("pg-engine: BCKeyChain {app_name}/{key}: SecItemUpdate error {ERR_SEC_ITEM_NOT_FOUND}");
    }
    0
}

/// `-[BCKeyChain reset]` @ 0x10001f528: deletes the stored item and returns the status; this
/// object keeps its secret.
pub fn key_chain_reset(world: &mut World, key_chain: Entity) -> i32 {
    let (app_name, key) = {
        let object = world.obj::<BcKeyChain>(key_chain);
        (object.app_name.clone(), object.key.clone())
    };
    load_key_chain_items(world);

    let removed = {
        let mut key_chains = world.resource_mut::<KeyChains>();
        match key_chains.items.get_mut(&app_name) {
            Some(accounts) => accounts.remove(&key).is_some(),
            None => false,
        }
    };
    if !removed {
        return ERR_SEC_ITEM_NOT_FOUND;
    }
    save_key_chain_items(world);
    ERR_SEC_SUCCESS
}

/// `-[BCKeyChain hasError]` @ 0x10001f650.
pub fn key_chain_has_error(world: &World, key_chain: Entity) -> bool {
    world.obj::<BcKeyChain>(key_chain).error
}

/// `-[BCKeyChain hasBeenSet]` @ 0x10001f660.
pub fn key_chain_has_been_set(world: &World, key_chain: Entity) -> bool {
    key_chain_secret(world, key_chain) != UNSET_SECRET
}

/// `-[BCKeyChain secret]` @ 0x10001f698. Skin-pack purchase accounts read as bought.
pub fn key_chain_secret(world: &World, key_chain: Entity) -> String {
    let object = world.obj::<BcKeyChain>(key_chain);
    if is_free_purchase(&object.app_name, &object.key) {
        return PURCHASED_SECRET.to_owned();
    }
    object.secret.clone()
}

/// `-[BCKeyChain key]` @ 0x10001f6a8.
pub fn key_chain_key(world: &World, key_chain: Entity) -> String {
    world.obj::<BcKeyChain>(key_chain).key.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::BcView;

    fn test_world() -> World {
        let mut world = World::new();
        world.init_resource::<BcView>();
        world
    }

    /// Moves the stored files into a new world, like a restart.
    fn restart(world: &mut World) -> World {
        let mut storage = world.get_resource_or_init::<MemoryStorage>();
        let files = std::mem::take(&mut storage.files);
        let mut restarted = test_world();
        restarted.insert_resource(MemoryStorage { files });
        restarted
    }

    fn stored_file(world: &mut World, name: &str) -> Option<Vec<u8>> {
        let storage = world.get_resource_or_init::<MemoryStorage>();
        let key = (StorageDir::Documents, name.to_owned());
        storage.files.get(&key).cloned()
    }

    fn loaded_document(world: &mut World, id: &str, secure: bool) -> Entity {
        let prefs = spawn(world, id, secure, secure);
        load_preferences(world, prefs);
        prefs
    }

    #[test]
    fn absent_keys_read_as_zero_or_nil() {
        let mut world = test_world();
        let prefs = loaded_document(&mut world, "Options", false);
        assert_eq!(get_int_preference(&world, prefs, "Mute"), 0);
        assert_eq!(get_float_preference(&world, prefs, "Mute"), 0.0);
        assert_eq!(get_string_preference(&world, prefs, "Mute"), None);
        assert_eq!(get_dictionary_preference(&world, prefs, "Mute"), None);
        assert!(!does_preference_exist(&world, prefs, "Mute"));
        assert_eq!(get_string_preference(&world, prefs, "TEST").as_deref(), Some("TEST"));
    }

    #[test]
    fn defaults_never_overwrite_and_values_convert_like_nsnumber() {
        let mut world = test_world();
        let prefs = loaded_document(&mut world, "Options", false);
        assert_eq!(set_default_preference_as_int(&mut world, prefs, "IslandLocation", 5), 5);
        set_preference_as_int(&mut world, prefs, "IslandLocation", 7);
        assert_eq!(set_default_preference_as_int(&mut world, prefs, "IslandLocation", 5), 7);
        assert_eq!(get_float_preference(&world, prefs, "IslandLocation"), 7.0);

        set_preference_as_float(&mut world, prefs, "Health", -2.75);
        assert_eq!(get_int_preference(&world, prefs, "Health"), -2);
        assert_eq!(set_default_preference_as_float(&mut world, prefs, "Health", 1.0), -2.75);

        set_preference_as_string(&mut world, prefs, "Version", " 12abc");
        assert_eq!(get_int_preference(&world, prefs, "Version"), 12);
        let kept = set_default_preference_as_string(&mut world, prefs, "Version", "7");
        assert_eq!(kept.as_deref(), Some(" 12abc"));

        let name = set_default_preference_as_string(&mut world, prefs, "God:Name", "GENE SIMIANS");
        assert_eq!(name.as_deref(), Some("GENE SIMIANS"));
        let stored_name = get_string_preference(&world, prefs, "God:Name");
        assert_eq!(stored_name.as_deref(), Some("GENE SIMIANS"));
    }

    #[test]
    fn a_nil_default_dictionary_stores_nothing() {
        let mut world = test_world();
        let prefs = loaded_document(&mut world, "Persona", false);
        world.obj_mut::<BcPreferences>(prefs).save_preferences = false;

        let missing = set_default_preference_as_dictionary(&mut world, prefs, "God:GiftDictionary", None);
        assert_eq!(missing, None);
        assert!(!does_preference_exist(&world, prefs, "God:GiftDictionary"));
        assert!(!world.obj::<BcPreferences>(prefs).save_preferences);

        let mut gifts = BTreeMap::new();
        gifts.insert("Coconut".to_owned(), 3);
        set_preference_as_dictionary(&mut world, prefs, "God:GiftDictionary", Some(&gifts));
        let stored = set_default_preference_as_dictionary(&mut world, prefs, "God:GiftDictionary", None);
        assert_eq!(stored, Some(gifts));
        assert!(world.obj::<BcPreferences>(prefs).save_preferences);
    }

    #[test]
    fn a_fresh_document_holds_the_test_entry_and_is_dirty() {
        let mut world = test_world();
        let prefs = loaded_document(&mut world, "System", false);
        let document = world.obj::<BcPreferences>(prefs);
        assert!(document.preferences_load_complete);
        assert!(document.save_preferences);
        assert!(!document.waiting_to_save);
        let listeners = &world.resource::<BcView>().game_frame_listeners;
        assert_eq!(listeners, &vec![prefs]);
        assert_eq!(isa_order(&world, prefs), PREFERENCES_GAME_FRAME_ORDER);
    }

    fn isa_order(world: &World, prefs: Entity) -> i32 {
        crate::object::isa(world, prefs).game_frame_order
    }

    #[test]
    fn dirty_documents_save_after_more_than_two_seconds_and_load_back() {
        let mut world = test_world();
        let prefs = loaded_document(&mut world, "Options", false);
        set_preference_as_int(&mut world, prefs, "Mute", 1);
        assert!(waiting_to_save(&world, prefs));

        on_game_frame(&mut world, prefs, 1.0);
        on_game_frame(&mut world, prefs, 1.0);
        assert_eq!(stored_file(&mut world, "Options.json"), None);
        on_game_frame(&mut world, prefs, 0.5);
        assert!(stored_file(&mut world, "Options.json").is_some());
        assert!(!waiting_to_save(&world, prefs));
        assert_eq!(world.obj::<BcPreferences>(prefs).last_save_time, 0.0);

        let mut restarted = restart(&mut world);
        let reloaded = loaded_document(&mut restarted, "Options", false);
        assert_eq!(get_int_preference(&restarted, reloaded, "Mute"), 1);
        assert!(!restarted.obj::<BcPreferences>(reloaded).save_preferences);
    }

    #[test]
    fn flush_saves_on_the_next_frame_with_time() {
        let mut world = test_world();
        let prefs = loaded_document(&mut world, "Challenge2", false);
        on_game_frame(&mut world, prefs, 0.1);
        flush(&mut world, prefs);
        on_game_frame(&mut world, prefs, 0.0);
        assert_eq!(stored_file(&mut world, "Challenge2.json"), None);
        on_game_frame(&mut world, prefs, 0.016);
        assert!(stored_file(&mut world, "Challenge2.json").is_some());
    }

    #[test]
    fn the_legacy_file_is_read_when_the_document_has_none() {
        let mut world = test_world();
        let mut legacy = BTreeMap::new();
        legacy.insert("Mute".to_owned(), PrefValue::Int(1));
        let data = data_from_dictionary(&legacy);
        write_application_data_to_file(&mut world, &data, "Preferences.plist");

        let prefs = spawn(&mut world, "Options", false, false);
        set_legacy_preference_file_name(&mut world, prefs, Some("Preferences.plist"));
        load_preferences(&mut world, prefs);
        assert_eq!(get_int_preference(&world, prefs, "Mute"), 1);
        assert!(!world.obj::<BcPreferences>(prefs).save_preferences);
    }

    #[test]
    fn unreadable_files_start_a_fresh_document() {
        let mut world = test_world();
        write_application_data_to_file(&mut world, b"bplist00", "Persona.json");
        let prefs = loaded_document(&mut world, "Persona", false);
        assert_eq!(get_string_preference(&world, prefs, "TEST").as_deref(), Some("TEST"));
        assert!(world.obj::<BcPreferences>(prefs).save_preferences);
    }

    #[test]
    fn secure_documents_live_in_the_keychain() {
        let mut world = test_world();
        let prefs = loaded_document(&mut world, "Challenge", true);
        set_preference_as_int(&mut world, prefs, "Episode1BattleUnlock", 1);
        set_preference_as_string(&mut world, prefs, "Name", "Ünïcode");
        flush(&mut world, prefs);
        on_game_frame(&mut world, prefs, 0.016);

        let document = world.obj::<BcPreferences>(prefs);
        assert!(document.preference_dictionary_for_thread.is_none());
        assert!(!document.waiting_to_save);
        assert!(stored_file(&mut world, "Challenge.json").is_none());
        let key_chain = world.obj::<BcPreferences>(prefs).key_chain.expect("keychain");
        assert!(key_chain_secret(&world, key_chain).is_ascii());

        let mut restarted = restart(&mut world);
        let reloaded = loaded_document(&mut restarted, "Challenge", true);
        assert_eq!(get_int_preference(&restarted, reloaded, "Episode1BattleUnlock"), 1);
        let name = get_string_preference(&restarted, reloaded, "Name");
        assert_eq!(name.as_deref(), Some("Ünïcode"));
    }

    #[test]
    fn keychain_items_start_unset_and_reset_keeps_the_cached_secret() {
        let mut world = test_world();
        let option = key_chain_with_key_using_app_name_default_secret(&mut world, KEY_CHAIN_APP_NAME, "TRex Body", "0");
        assert_eq!(key_chain_secret(&world, option), UNSET_SECRET);
        assert!(!key_chain_has_been_set(&world, option));
        assert_eq!(
            key_chain_with_key_using_app_name(&mut world, KEY_CHAIN_APP_NAME, "TRex Body"),
            option
        );

        key_chain_set_secret(&mut world, option, "3");
        assert_eq!(key_chain_reset(&mut world, option), ERR_SEC_SUCCESS);
        assert_eq!(key_chain_reset(&mut world, option), ERR_SEC_ITEM_NOT_FOUND);
        assert_eq!(key_chain_secret(&world, option), "3");
        key_chain_set_secret(&mut world, option, "4");

        let mut restarted = restart(&mut world);
        let reread = key_chain_with_key_using_app_name(&mut restarted, KEY_CHAIN_APP_NAME, "TRex Body");
        assert_eq!(key_chain_secret(&restarted, reread), UNSET_SECRET);
        assert!(!key_chain_has_error(&restarted, reread));
        assert_eq!(key_chain_key(&restarted, reread), "TRex Body");
    }

    #[test]
    fn set_secrets_survive_a_restart() {
        let mut world = test_world();
        let option = key_chain_with_key_using_app_name(&mut world, KEY_CHAIN_APP_NAME, "Shark Skins");
        key_chain_set_secret(&mut world, option, "2");
        let mut restarted = restart(&mut world);
        let reread = key_chain_with_key_using_app_name(&mut restarted, KEY_CHAIN_APP_NAME, "Shark Skins");
        assert_eq!(key_chain_secret(&restarted, reread), "2");
        assert!(key_chain_has_been_set(&restarted, reread));
    }

    #[test]
    fn skin_pack_purchases_read_as_bought() {
        let mut world = test_world();
        let pack = key_chain_with_key_using_app_name(&mut world, KEY_CHAIN_APP_NAME, "AdultSwimSkinPack");
        assert_eq!(key_chain_secret(&world, pack), PURCHASED_SECRET);
        assert!(key_chain_has_been_set(&world, pack));

        let bonus = key_chain_with_key_using_app_name(&mut world, KEY_CHAIN_APP_NAME, "ComicPreOrder");
        assert_eq!(key_chain_secret(&world, bonus), UNSET_SECRET);

        let file = pg_assets::data::skin_packs::load().expect("skin-packs.json matches its schema");
        for pack in &file.packs {
            let key_chain = key_chain_with_key_using_app_name(&mut world, KEY_CHAIN_APP_NAME, &pack.keychain_key);
            assert_eq!(key_chain_secret(&world, key_chain), PURCHASED_SECRET, "{}", pack.id);
        }
    }

    #[test]
    fn match_data_round_trips_and_bad_data_gives_a_fresh_dictionary() {
        let mut world = test_world();
        let sender = spawn_empty(&mut world);
        load_preferences_from_data(&mut world, sender, None);
        assert_eq!(
            world.obj::<BcPreferences>(sender).preference_dictionary,
            Some(BTreeMap::new())
        );
        set_preference_as_string(&mut world, sender, "GameCategory", "BattleOfTheGods");
        set_preference_as_float(&mut world, sender, "ChallengerHealth", 0.1);
        set_preference_as_float(&mut world, sender, "RecipientHealth", f32::NAN);
        let data = data_from_preferences(&world, sender).expect("data");

        let receiver = spawn_empty(&mut world);
        load_preferences_from_data(&mut world, receiver, Some(&data));
        assert_eq!(
            get_string_preference(&world, receiver, "GameCategory").as_deref(),
            Some("BattleOfTheGods")
        );
        assert_eq!(get_float_preference(&world, receiver, "ChallengerHealth"), 0.1);
        assert!(get_float_preference(&world, receiver, "RecipientHealth").is_nan());
        assert!(!world.obj::<BcPreferences>(receiver).save_preferences);
        assert!(world.resource::<BcView>().game_frame_listeners.is_empty());

        load_preferences_from_data(&mut world, receiver, Some(b"{"));
        assert_eq!(get_string_preference(&world, receiver, "TEST").as_deref(), Some("TEST"));
        assert!(world.obj::<BcPreferences>(receiver).save_preferences);
    }

    #[test]
    fn merging_keeps_local_strings_and_larger_numbers() {
        let mut local = BTreeMap::new();
        local.insert("Name".to_owned(), PrefValue::String("Mine".to_owned()));
        local.insert("Unlock".to_owned(), PrefValue::Int(3));
        local.insert("Health".to_owned(), PrefValue::Float(2.5));
        let mut local_counts = BTreeMap::new();
        local_counts.insert("Coconut".to_owned(), 4);
        local.insert("Gifts".to_owned(), PrefValue::Dict(local_counts));

        let mut other = BTreeMap::new();
        other.insert("Name".to_owned(), PrefValue::String("Theirs".to_owned()));
        other.insert("Unlock".to_owned(), PrefValue::Int(2));
        other.insert("Health".to_owned(), PrefValue::Int(3));
        other.insert("New".to_owned(), PrefValue::Int(1));
        let mut other_counts = BTreeMap::new();
        other_counts.insert("Coconut".to_owned(), 2);
        other_counts.insert("Fish".to_owned(), 1);
        other.insert("Gifts".to_owned(), PrefValue::Dict(other_counts));

        merge_dictionary(&mut local, &other);
        assert_eq!(local["Name"], PrefValue::String("Mine".to_owned()));
        assert_eq!(local["Unlock"], PrefValue::Int(3));
        assert_eq!(local["Health"], PrefValue::Int(3));
        assert_eq!(local["New"], PrefValue::Int(1));
        let gifts = local["Gifts"].as_dictionary().expect("dictionary");
        assert_eq!(gifts["Coconut"], 4);
        assert_eq!(gifts["Fish"], 1);
    }

    fn record_complete(world: &mut World, _listener: Entity, data: Option<u64>, url: &str, bytes: &[u8]) {
        let text = String::from_utf8_lossy(bytes);
        let entry = format!("complete {data:?} {url} {text}");
        world.get_resource_or_init::<DownloadLog>().0.push(entry);
    }

    fn record_error(world: &mut World, _listener: Entity, data: Option<u64>, url: &str, error: &str) {
        let entry = format!("error {data:?} {url} {error}");
        world.get_resource_or_init::<DownloadLog>().0.push(entry);
    }

    fn connect(world: &mut World, _prefs: Entity, url: &str) -> bool {
        let entry = format!("start {url}");
        world.get_resource_or_init::<DownloadLog>().0.push(entry);
        true
    }

    #[derive(Resource, Default)]
    struct DownloadLog(Vec<String>);

    #[test]
    fn downloads_complete_on_the_next_frame_and_failures_stall_the_queue() {
        let mut world = test_world();
        let hooks = PreferencesHooks {
            start_download: connect,
            ..Default::default()
        };
        world.insert_resource(hooks);
        let prefs = loaded_document(&mut world, "System", false);
        let listener_object = world.spawn_empty().id();
        let listener = ServerLoadListener {
            object: listener_object,
            on_complete: record_complete,
            on_error: record_error,
        };

        load_data_from_server(&mut world, prefs, listener, None, "feed");
        load_data_from_server(&mut world, prefs, listener, Some(7), "icon");
        assert_eq!(world.resource::<BcView>().game_frame_listeners.len(), 3);

        connection_did_receive_data(&mut world, prefs, b"stale");
        connection_did_receive_response(&mut world, prefs);
        connection_did_receive_data(&mut world, prefs, b"news");
        connection_did_finish_loading(&mut world, prefs);
        on_game_frame(&mut world, prefs, 0.0);

        connection_did_fail_with_error(&mut world, prefs, "offline", None);
        load_data_from_server(&mut world, prefs, listener, None, "feed");

        let log = &world.resource::<DownloadLog>().0;
        let expected = vec![
            "start feed".to_owned(),
            "complete None feed news".to_owned(),
            "start icon".to_owned(),
            "error Some(7) icon Connection failed! Error - offline (null)".to_owned(),
            "start feed".to_owned(),
        ];
        assert_eq!(log, &expected);

        connection_did_fail_with_error(&mut world, prefs, "offline", Some("feed"));
        load_data_from_server(&mut world, prefs, listener, None, "a");
        load_data_from_server(&mut world, prefs, listener, None, "b");
        connection_did_fail_with_error(&mut world, prefs, "offline", None);
        let log = &world.resource::<DownloadLog>().0;
        assert_eq!(
            log.last().map(String::as_str),
            Some("error None a Connection failed! Error - offline (null)")
        );
        assert_eq!(world.obj::<BcPreferences>(prefs).load_instances.len(), 1);
    }

    #[test]
    fn without_a_network_downloads_never_start() {
        let mut world = test_world();
        let prefs = loaded_document(&mut world, "System", false);
        let listener_object = world.spawn_empty().id();
        let listener = ServerLoadListener {
            object: listener_object,
            on_complete: record_complete,
            on_error: record_error,
        };
        load_data_from_server(&mut world, prefs, listener, None, "feed");
        on_game_frame(&mut world, prefs, 0.0);
        let document = world.obj::<BcPreferences>(prefs);
        assert!(!document.url_connection);
        assert_eq!(document.received_data, None);
        assert_eq!(document.load_instances.len(), 1);
        assert!(world.get_resource::<DownloadLog>().is_none());
    }

    #[test]
    fn resource_paths_map_to_converted_assets() {
        assert_eq!(
            resource_asset_path("DefaultNewsIcon", "raw"),
            "textures/DefaultNewsIcon.png"
        );
        assert_eq!(resource_asset_path("sound/fart1", "caf"), "sounds/fart1.flac");
        assert!(does_file_exist_in_resources("DefaultNewsIcon", "raw"));
        assert!(does_file_exist_in_resources("news", "xml"));
        assert!(!does_file_exist_in_resources("MonkeyPromoNewsIcon", "raw"));
    }

    #[test]
    fn cache_and_document_files_are_separate() {
        let mut world = test_world();
        assert!(write_application_data_to_library_cache_file(
            &mut world, b"<news/>", "news.xml"
        ));
        assert!(does_file_exist_in_library_cache(&mut world, "news", "xml"));
        assert!(!does_file_exist_in_documents(&mut world, "news", "xml"));
        assert_eq!(
            application_data_from_library_cache_file(&mut world, "news.xml"),
            Some(b"<news/>".to_vec())
        );

        write_application_data_to_file(&mut world, b"story", "StoryMap");
        assert_eq!(
            application_data_from_file(&mut world, "StoryMap"),
            Some(b"story".to_vec())
        );
        remove_file(&mut world, "StoryMap");
        assert_eq!(application_data_from_file(&mut world, "StoryMap"), None);
    }
}
