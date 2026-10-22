//! `GodPersona` (superclass `Persona`).

use std::collections::BTreeMap;

use pg_engine::prefs;

use crate::persona::persona::{self as persona_class, Persona};
use crate::prelude::*;

/// `GodPersona` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct GodPersona {
    /// `mGodType`: `i`
    pub god_type: i32,

    /// `mAchievementDictionary`: `@"NSMutableDictionary"` (achievement id → count).
    ///
    /// PORT: kept in key order. The original's `NSMutableDictionary` hash order shows in
    /// `achievementIdArray`, which the god stats page lists (its first six ids).
    pub achievement_dictionary: BTreeMap<String, i32>,
}

pub const GOD_PERSONA: Class = Class {
    name: "GodPersona",
    superclass: Some(&crate::persona::persona::PERSONA),
    ..crate::persona::persona::PERSONA
};

/// `-[GodPersona initWithId:defaultName:defaultDescription:preferences:]` @ 0x1001a646c
pub fn init_with_id(world: &mut World, this: Entity, id: &str, default_name: &str, default_description: &str, preferences: Entity) {
    world.entity_mut(this).insert(GodPersona::default());
    persona_class::init_with_id(world, this, id, default_name, default_description, preferences);

    let god_type_key = persona_class::preference_key(world, this, "PersonaGodType");
    let god_type = prefs::set_default_preference_as_int(world, preferences, &god_type_key, 1);
    world.obj_mut::<GodPersona>(this).god_type = god_type;

    let achievement_dictionary_key = persona_class::preference_key(world, this, "AchievementDictionary");
    let stored_achievements = prefs::set_default_preference_as_dictionary(world, preferences, &achievement_dictionary_key, None);
    let mut achievement_dictionary = BTreeMap::new();
    if let Some(stored_achievements) = stored_achievements {
        achievement_dictionary.extend(stored_achievements);
    }
    world.obj_mut::<GodPersona>(this).achievement_dictionary = achievement_dictionary;
}

/// `-[GodPersona dealloc]` @ 0x1001a65c4
///
/// For the god persona's last owner (the view controller). The achievement dictionary goes
/// with the entity.
pub fn dealloc(world: &mut World, this: Entity) {
    persona_class::dealloc(world, this);
}

/// `-[GodPersona refreshFromPreferences]` @ 0x1001a6628
pub fn refresh_from_preferences(world: &mut World, this: Entity) {
    persona_class::refresh_from_preferences(world, this);

    // `mPreferences` is set by the initialiser and never cleared.
    let Some(preferences) = world.obj::<Persona>(this).preferences else {
        return;
    };

    let god_type_key = persona_class::preference_key(world, this, "PersonaGodType");
    let god_type = prefs::get_int_preference(world, preferences, &god_type_key);
    world.obj_mut::<GodPersona>(this).god_type = god_type;

    // Re-read with `setDefaultPreference:asDictionary:nil`, like the gift dictionary.
    let achievement_dictionary_key = persona_class::preference_key(world, this, "AchievementDictionary");
    let stored_achievements = prefs::set_default_preference_as_dictionary(world, preferences, &achievement_dictionary_key, None);
    if let Some(stored_achievements) = stored_achievements {
        let mut god_persona = world.obj_mut::<GodPersona>(this);
        god_persona.achievement_dictionary.clear();
        god_persona.achievement_dictionary.extend(stored_achievements);
    }
}

/// `-[GodPersona setGodType:]` @ 0x1001a675c
pub fn set_god_type(world: &mut World, this: Entity, god_type: i32) {
    world.obj_mut::<GodPersona>(this).god_type = god_type;

    persona_class::write_int_preference(world, this, "PersonaGodType", god_type);
}

/// `-[GodPersona godType]` @ 0x1001a67e8
pub fn god_type(world: &mut World, this: Entity) -> i32 {
    world.obj::<GodPersona>(this).god_type
}

/// `-[GodPersona achievementCount:]` @ 0x1001a67f8
pub fn achievement_count(world: &mut World, this: Entity, achievement_id: &str) -> i32 {
    let god_persona = world.obj::<GodPersona>(this);
    match god_persona.achievement_dictionary.get(achievement_id) {
        Some(count) => *count,
        None => 0,
    }
}

/// `-[GodPersona setAchievement:count:highScoreWins:]` @ 0x1001a6840
///
/// Stores `count` when it beats the stored one: higher when `high_score_wins`, otherwise lower
/// (or anything while the stored count is 0 or missing).
pub fn set_achievement(world: &mut World, this: Entity, achievement_id: &str, count: i32, high_score_wins: bool) {
    let stored_count = {
        let god_persona = world.obj::<GodPersona>(this);
        god_persona.achievement_dictionary.get(achievement_id).copied().unwrap_or(0)
    };

    let higher_wins = stored_count < count && high_score_wins;
    let lower_wins = !high_score_wins && (count < stored_count || stored_count == 0);
    if !higher_wins && !lower_wins {
        return;
    }

    let achievement_dictionary = {
        let mut god_persona = world.obj_mut::<GodPersona>(this);
        god_persona.achievement_dictionary.insert(achievement_id.to_owned(), count);
        god_persona.achievement_dictionary.clone()
    };
    persona_class::write_dictionary_preference(world, this, "AchievementDictionary", &achievement_dictionary);
}

/// `-[GodPersona addIncrementalAchievement:]` @ 0x1001a6978
pub fn add_incremental_achievement(world: &mut World, this: Entity, achievement_id: &str) {
    let achievement_dictionary = {
        let mut god_persona = world.obj_mut::<GodPersona>(this);
        let count = god_persona.achievement_dictionary.get(achievement_id).copied().unwrap_or(0);
        god_persona.achievement_dictionary.insert(achievement_id.to_owned(), count.wrapping_add(1));
        god_persona.achievement_dictionary.clone()
    };

    persona_class::write_dictionary_preference(world, this, "AchievementDictionary", &achievement_dictionary);
}

/// `-[GodPersona achievementIdArray]` @ 0x1001a6a80
///
/// `[mAchievementDictionary allKeys]`, in the dictionary's order (key order, see
/// `achievement_dictionary`).
pub fn achievement_id_array(world: &mut World, this: Entity) -> Vec<String> {
    let god_persona = world.obj::<GodPersona>(this);
    let achievement_ids = god_persona.achievement_dictionary.keys();
    achievement_ids.cloned().collect()
}
