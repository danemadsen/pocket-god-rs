//! `Persona` (superclass `NSObject`).

use std::collections::BTreeMap;

use pg_engine::prefs;

use crate::prelude::*;

/// `Persona` ivars.
///
/// The two dictionaries map an id to its count (`NSNumber`).
///
/// PORT: they are kept in key order. The original's `NSMutableDictionary` enumerates in its
/// hash order (which depends on the iOS version), and that order shows in
/// `sortedSacrificeIdArray` / `sortedGiftIdArray` among ids with the same count.
#[derive(Component, Clone, Debug, Default)]
pub struct Persona {
    /// `mPreferences`: `@"BCPreferences"`
    pub preferences: Option<Entity>,

    /// `mId`: `@"NSString"`
    pub id: Option<String>,

    /// `mName`: `@"NSString"`
    pub name: Option<String>,

    /// `mDescription`: `@"NSString"`
    pub description: Option<String>,

    /// `mMood`: `i`
    pub mood: i32,

    /// `mTotalSacrifices`: `i`
    pub total_sacrifices: i32,

    /// `mSacrificeDictionary`: `@"NSMutableDictionary"` (sacrifice id → count)
    pub sacrifice_dictionary: BTreeMap<String, i32>,

    /// `mTotalGifts`: `i`
    pub total_gifts: i32,

    /// `mGiftDictionary`: `@"NSMutableDictionary"` (gift id → count)
    pub gift_dictionary: BTreeMap<String, i32>,
}

pub const PERSONA: Class = Class {
    name: "Persona",
    superclass: None,
    ..Class::NS_OBJECT
};

/// `[NSString stringWithFormat:@"%@:<name>", mId]`: the key of one of this persona's values
/// in the `Persona` document.
pub(crate) fn preference_key(world: &World, this: Entity, name: &str) -> String {
    let persona = world.obj::<Persona>(this);
    let id = persona.id.as_deref().unwrap_or("(null)");
    format!("{id}:{name}")
}

/// `[mPreferences setPreference:<key> asInt:value]` for the key `%@:<name>`.
pub(crate) fn write_int_preference(world: &mut World, this: Entity, name: &str, value: i32) {
    let key = preference_key(world, this, name);
    let preferences = world.obj::<Persona>(this).preferences;
    if let Some(preferences) = preferences {
        prefs::set_preference_as_int(world, preferences, &key, value);
    }
}

/// `[mPreferences setPreference:<key> asString:value]` for the key `%@:<name>`.
fn write_string_preference(world: &mut World, this: Entity, name: &str, value: &str) {
    let key = preference_key(world, this, name);
    let preferences = world.obj::<Persona>(this).preferences;
    if let Some(preferences) = preferences {
        prefs::set_preference_as_string(world, preferences, &key, value);
    }
}

/// `[mPreferences setPreference:<key> asDictionary:dictionary]` for the key `%@:<name>`.
pub(crate) fn write_dictionary_preference(world: &mut World, this: Entity, name: &str, dictionary: &BTreeMap<String, i32>) {
    let key = preference_key(world, this, name);
    let preferences = world.obj::<Persona>(this).preferences;
    if let Some(preferences) = preferences {
        prefs::set_preference_as_dictionary(world, preferences, &key, Some(dictionary));
    }
}

/// `-[Persona initWithId:defaultName:defaultDescription:preferences:]` @ 0x1001a44d8
pub fn init_with_id(world: &mut World, this: Entity, id: &str, default_name: &str, default_description: &str, preferences: Entity) {
    world.entity_mut(this).insert(Persona::default());

    {
        let mut persona = world.obj_mut::<Persona>(this);
        persona.preferences = Some(preferences);
        persona.id = Some(id.to_owned());
    }

    let name_key = preference_key(world, this, "Name");
    let name = prefs::set_default_preference_as_string(world, preferences, &name_key, default_name);
    world.obj_mut::<Persona>(this).name = name;

    let description_key = preference_key(world, this, "Description");
    let description = prefs::set_default_preference_as_string(world, preferences, &description_key, default_description);
    world.obj_mut::<Persona>(this).description = description;

    let mood_key = preference_key(world, this, "PersonaMood");
    let mood = prefs::set_default_preference_as_int(world, preferences, &mood_key, 2);
    world.obj_mut::<Persona>(this).mood = mood;

    let total_sacrifices_key = preference_key(world, this, "TotalSacrifices");
    let total_sacrifices = prefs::set_default_preference_as_int(world, preferences, &total_sacrifices_key, 0);
    world.obj_mut::<Persona>(this).total_sacrifices = total_sacrifices;

    let sacrifice_dictionary_key = preference_key(world, this, "SacrificeDictionary");
    let stored_sacrifices = prefs::set_default_preference_as_dictionary(world, preferences, &sacrifice_dictionary_key, None);
    let mut sacrifice_dictionary = BTreeMap::new();
    if let Some(stored_sacrifices) = stored_sacrifices {
        sacrifice_dictionary.extend(stored_sacrifices);
    }
    world.obj_mut::<Persona>(this).sacrifice_dictionary = sacrifice_dictionary;

    let total_gifts_key = preference_key(world, this, "TotalGifts");
    let total_gifts = prefs::set_default_preference_as_int(world, preferences, &total_gifts_key, 0);
    world.obj_mut::<Persona>(this).total_gifts = total_gifts;

    let gift_dictionary_key = preference_key(world, this, "GiftDictionary");
    let stored_gifts = prefs::set_default_preference_as_dictionary(world, preferences, &gift_dictionary_key, None);
    let mut gift_dictionary = BTreeMap::new();
    if let Some(stored_gifts) = stored_gifts {
        gift_dictionary.extend(stored_gifts);
    }
    world.obj_mut::<Persona>(this).gift_dictionary = gift_dictionary;

    update_mood(world, this);
}

/// `-[Persona dealloc]` @ 0x1001a4838
///
/// For the persona's last owner. The id, name, description, dictionaries and the document
/// reference go with the entity.
pub fn dealloc(world: &mut World, this: Entity) {
    // `[super dealloc]`: `NSObject` frees the object.
    world.despawn(this);
}

/// `-[Persona refreshFromPreferences]` @ 0x1001a4904
pub fn refresh_from_preferences(world: &mut World, this: Entity) {
    // `mPreferences` is set by the initialiser and never cleared.
    let Some(preferences) = world.obj::<Persona>(this).preferences else {
        return;
    };

    let name_key = preference_key(world, this, "Name");
    let name = prefs::get_string_preference(world, preferences, &name_key);
    world.obj_mut::<Persona>(this).name = name;

    let description_key = preference_key(world, this, "Description");
    let description = prefs::get_string_preference(world, preferences, &description_key);
    world.obj_mut::<Persona>(this).description = description;

    let mood_key = preference_key(world, this, "PersonaMood");
    let mood = prefs::get_int_preference(world, preferences, &mood_key);
    world.obj_mut::<Persona>(this).mood = mood;

    let total_sacrifices_key = preference_key(world, this, "TotalSacrifices");
    let total_sacrifices = prefs::get_int_preference(world, preferences, &total_sacrifices_key);
    world.obj_mut::<Persona>(this).total_sacrifices = total_sacrifices;

    let sacrifice_dictionary_key = preference_key(world, this, "SacrificeDictionary");
    let stored_sacrifices = prefs::get_dictionary_preference(world, preferences, &sacrifice_dictionary_key);
    if let Some(stored_sacrifices) = stored_sacrifices {
        let mut persona = world.obj_mut::<Persona>(this);
        persona.sacrifice_dictionary.clear();
        persona.sacrifice_dictionary.extend(stored_sacrifices);
    }

    let mut sacrifice_count_sum: i32 = 0;
    for count in world.obj::<Persona>(this).sacrifice_dictionary.values() {
        sacrifice_count_sum = count.wrapping_add(sacrifice_count_sum);
    }
    let total_sacrifices = world.obj::<Persona>(this).total_sacrifices;
    if total_sacrifices < sacrifice_count_sum {
        set_total_sacrifices(world, this, sacrifice_count_sum);
    }

    let total_gifts_key = preference_key(world, this, "TotalGifts");
    let total_gifts = prefs::get_int_preference(world, preferences, &total_gifts_key);
    world.obj_mut::<Persona>(this).total_gifts = total_gifts;

    // The gift dictionary is re-read with `setDefaultPreference:asDictionary:nil`, not
    // `getDictionaryPreference:`.
    let gift_dictionary_key = preference_key(world, this, "GiftDictionary");
    let stored_gifts = prefs::set_default_preference_as_dictionary(world, preferences, &gift_dictionary_key, None);
    if let Some(stored_gifts) = stored_gifts {
        let mut persona = world.obj_mut::<Persona>(this);
        persona.gift_dictionary.clear();
        persona.gift_dictionary.extend(stored_gifts);
    }

    let mut gift_count_sum: i32 = 0;
    for count in world.obj::<Persona>(this).gift_dictionary.values() {
        gift_count_sum = count.wrapping_add(gift_count_sum);
    }
    let total_gifts = world.obj::<Persona>(this).total_gifts;
    if total_gifts < gift_count_sum {
        set_total_gifts(world, this, gift_count_sum);
    }

    update_mood(world, this);
}

/// `-[Persona name]` @ 0x1001a4e9c
pub fn name(world: &mut World, this: Entity) -> Option<String> {
    world.obj::<Persona>(this).name.clone()
}

/// `-[Persona setName:]` @ 0x1001a4eac
pub fn set_name(world: &mut World, this: Entity, name: &str) {
    world.obj_mut::<Persona>(this).name = Some(name.to_owned());

    write_string_preference(world, this, "Name", name);
}

/// `-[Persona description]` @ 0x1001a4f64
pub fn description(world: &mut World, this: Entity) -> Option<String> {
    world.obj::<Persona>(this).description.clone()
}

/// `-[Persona setDescription:]` @ 0x1001a4f74
pub fn set_description(world: &mut World, this: Entity, description: &str) {
    world.obj_mut::<Persona>(this).description = Some(description.to_owned());

    write_string_preference(world, this, "Description", description);
}

/// `-[Persona updateMood]` @ 0x1001a502c
///
/// Moods: 0 Happy, 1 Content, 2 Normal, 3 Sad, 4 Annoyed, 5 Angry, 6 FreakedOut.
pub fn update_mood(world: &mut World, this: Entity) {
    let (total_sacrifices, total_gifts) = {
        let persona = world.obj::<Persona>(this);
        (persona.total_sacrifices, persona.total_gifts)
    };

    let mut sacrifices_per_gift = total_sacrifices as f32;
    let gifts = total_gifts as f32;
    if 0.0 < gifts {
        sacrifices_per_gift = sacrifices_per_gift / gifts;
    }

    let mood = if sacrifices_per_gift < 4.0 && total_sacrifices >= 21 {
        0
    } else if sacrifices_per_gift < 7.0 && total_sacrifices >= 11 {
        1
    } else if sacrifices_per_gift < 10.0 {
        2
    } else if sacrifices_per_gift < 20.0 {
        3
    } else if sacrifices_per_gift < 30.0 {
        4
    } else if sacrifices_per_gift < 40.0 {
        5
    } else {
        6
    };
    world.obj_mut::<Persona>(this).mood = mood;

    write_int_preference(world, this, "PersonaMood", mood);
}

/// `-[Persona mood]` @ 0x1001a51a0
pub fn mood(world: &mut World, this: Entity) -> i32 {
    world.obj::<Persona>(this).mood
}

/// `-[Persona totalSacrifices]` @ 0x1001a51b0
pub fn total_sacrifices(world: &mut World, this: Entity) -> i32 {
    world.obj::<Persona>(this).total_sacrifices
}

/// `-[Persona setTotalSacrifices:]` @ 0x1001a51c0
pub fn set_total_sacrifices(world: &mut World, this: Entity, total_sacrifices: i32) {
    world.obj_mut::<Persona>(this).total_sacrifices = total_sacrifices;

    write_int_preference(world, this, "TotalSacrifices", total_sacrifices);
    update_mood(world, this);
}

/// `-[Persona incrementTotalSacrifices]` @ 0x1001a5260
pub fn increment_total_sacrifices(world: &mut World, this: Entity) {
    let total_sacrifices = {
        let mut persona = world.obj_mut::<Persona>(this);
        persona.total_sacrifices = persona.total_sacrifices.wrapping_add(1);
        persona.total_sacrifices
    };

    write_int_preference(world, this, "TotalSacrifices", total_sacrifices);
    update_mood(world, this);
}

/// `-[Persona sacrificeCount:]` @ 0x1001a5308
pub fn sacrifice_count(world: &mut World, this: Entity, sacrifice_id: &str) -> i32 {
    let persona = world.obj::<Persona>(this);
    match persona.sacrifice_dictionary.get(sacrifice_id) {
        Some(count) => *count,
        None => 0,
    }
}

/// `-[Persona addSacrifice:]` @ 0x1001a5350
pub fn add_sacrifice(world: &mut World, this: Entity, sacrifice_id: &str) {
    let sacrifice_dictionary = {
        let mut persona = world.obj_mut::<Persona>(this);
        let count = persona.sacrifice_dictionary.get(sacrifice_id).copied().unwrap_or(0);
        persona.sacrifice_dictionary.insert(sacrifice_id.to_owned(), count.wrapping_add(1));
        persona.sacrifice_dictionary.clone()
    };

    write_dictionary_preference(world, this, "SacrificeDictionary", &sacrifice_dictionary);
}

/// `-[Persona sortedSacrificeIdArray]` @ 0x1001a5458
pub fn sorted_sacrifice_id_array(world: &mut World, this: Entity) -> Vec<String> {
    let persona = world.obj::<Persona>(this);
    ids_by_descending_count(&persona.sacrifice_dictionary)
}

/// `-[Persona totalGifts]` @ 0x1001a5698
pub fn total_gifts(world: &mut World, this: Entity) -> i32 {
    world.obj::<Persona>(this).total_gifts
}

/// `-[Persona setTotalGifts:]` @ 0x1001a56a8
pub fn set_total_gifts(world: &mut World, this: Entity, total_gifts: i32) {
    world.obj_mut::<Persona>(this).total_gifts = total_gifts;

    write_int_preference(world, this, "TotalGifts", total_gifts);
    update_mood(world, this);
}

/// `-[Persona incrementTotalGifts]` @ 0x1001a5748
pub fn increment_total_gifts(world: &mut World, this: Entity) {
    let total_gifts = {
        let mut persona = world.obj_mut::<Persona>(this);
        persona.total_gifts = persona.total_gifts.wrapping_add(1);
        persona.total_gifts
    };

    write_int_preference(world, this, "TotalGifts", total_gifts);
    update_mood(world, this);
}

/// `-[Persona giftCount:]` @ 0x1001a57f0
pub fn gift_count(world: &mut World, this: Entity, gift_id: &str) -> i32 {
    let persona = world.obj::<Persona>(this);
    match persona.gift_dictionary.get(gift_id) {
        Some(count) => *count,
        None => 0,
    }
}

/// `-[Persona addGift:]` @ 0x1001a5838
///
/// Unlike `addSacrifice:` there is no nil check: a nil id would raise in `setObject:forKey:`.
pub fn add_gift(world: &mut World, this: Entity, gift_id: &str) {
    let gift_dictionary = {
        let mut persona = world.obj_mut::<Persona>(this);
        let count = persona.gift_dictionary.get(gift_id).copied().unwrap_or(0);
        persona.gift_dictionary.insert(gift_id.to_owned(), count.wrapping_add(1));
        persona.gift_dictionary.clone()
    };

    write_dictionary_preference(world, this, "GiftDictionary", &gift_dictionary);
}

/// `-[Persona sortedGiftIdArray]` @ 0x1001a5924
pub fn sorted_gift_id_array(world: &mut World, this: Entity) -> Vec<String> {
    let persona = world.obj::<Persona>(this);
    ids_by_descending_count(&persona.gift_dictionary)
}

/// The body of `sortedSacrificeIdArray` and `sortedGiftIdArray`: the values sorted by
/// `intValue` descending; at each new count, `allKeysForObject:` of that count is appended.
fn ids_by_descending_count(dictionary: &BTreeMap<String, i32>) -> Vec<String> {
    let mut counts: Vec<i32> = dictionary.values().copied().collect();
    counts.sort_by(|first, second| second.cmp(first));

    let mut ids = Vec::with_capacity(5);
    let mut last_count: i32 = -1;
    for count in counts {
        if last_count == count {
            continue;
        }

        for (id, id_count) in dictionary {
            if *id_count == count {
                ids.push(id.clone());
            }
        }
        last_count = count;
    }
    ids
}
