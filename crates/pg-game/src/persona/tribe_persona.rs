//! `TribePersona` (superclass `Persona`).

use crate::persona::persona as persona_class;
use crate::persona::pygmy_persona;
use crate::prelude::*;

/// `TribePersona` ivars.
///
/// Every pygmy persona starts in the dead list. `resurrectPygmyLogic:` binds the first dead
/// one to a new pygmy and moves it to the living list; `sacrifice:pygmyLogic:` unbinds it and,
/// if still active, appends it to the end of the dead list, so the personas rotate.
#[derive(Component, Clone, Debug, Default)]
pub struct TribePersona {
    /// `mPygmyPersonaArray`: `@"NSMutableArray"`
    pub pygmy_persona_array: Vec<Entity>,

    /// `mDeadPygmyPersonaArray`: `@"NSMutableArray"`
    pub dead_pygmy_persona_array: Vec<Entity>,

    /// `mLivingPygmyPersonaArray`: `@"NSMutableArray"`
    pub living_pygmy_persona_array: Vec<Entity>,

    /// `mGuestPygmyPersonaArray`: `@"NSMutableArray"` (never allocated in the original: it
    /// stays nil and empty)
    pub guest_pygmy_persona_array: Vec<Entity>,
}

pub const TRIBE_PERSONA: Class = Class {
    name: "TribePersona",
    superclass: Some(&crate::persona::persona::PERSONA),
    ..crate::persona::persona::PERSONA
};

/// `-[TribePersona initWithId:defaultName:defaultDescription:preferences:]` @ 0x1001a5b64
pub fn init_with_id(world: &mut World, this: Entity, id: &str, default_name: &str, default_description: &str, preferences: Entity) {
    world.entity_mut(this).insert(TribePersona::default());
    persona_class::init_with_id(world, this, id, default_name, default_description, preferences);

    let mut tribe_persona = world.obj_mut::<TribePersona>(this);
    tribe_persona.pygmy_persona_array = Vec::with_capacity(5);
    tribe_persona.dead_pygmy_persona_array = Vec::with_capacity(5);
    tribe_persona.living_pygmy_persona_array = Vec::with_capacity(5);
}

/// `-[TribePersona dealloc]` @ 0x1001a5c44
///
/// For the tribe persona's last owner (the view controller). Releasing the three lists does
/// not free the pygmy personas here: a pygmy bound to one (`mPygmyPersona`) or the view
/// controller (Charlie, Tom) may still hold it.
pub fn dealloc(world: &mut World, this: Entity) {
    persona_class::dealloc(world, this);
}

/// `-[TribePersona refreshFromPreferences]` @ 0x1001a5cd4
///
/// The tribe's own values, then every pygmy persona's (guests are not in the list).
pub fn refresh_from_preferences(world: &mut World, this: Entity) {
    persona_class::refresh_from_preferences(world, this);

    let pygmy_persona_array = world.obj::<TribePersona>(this).pygmy_persona_array.clone();
    for pygmy_persona in pygmy_persona_array {
        crate::send::refresh_from_preferences(world, Some(pygmy_persona));
    }
}

/// `-[TribePersona addPygmyPersona:]` @ 0x1001a5e00
pub fn add_pygmy_persona(world: &mut World, this: Entity, pygmy_persona: Entity) {
    pygmy_persona::set_active(world, pygmy_persona, true);

    let mut tribe_persona = world.obj_mut::<TribePersona>(this);
    tribe_persona.pygmy_persona_array.push(pygmy_persona);
    tribe_persona.dead_pygmy_persona_array.push(pygmy_persona);
}

/// `-[TribePersona pygmyPersonaArray]` @ 0x1001a5e78
pub fn pygmy_persona_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<TribePersona>(this).pygmy_persona_array.clone()
}

/// `-[TribePersona addGuestPygmyPersona:]` @ 0x1001a5e88
///
/// Original bug, kept: `mGuestPygmyPersonaArray` is never allocated, so the `addObject:` goes
/// to nil and nothing is added.
pub fn add_guest_pygmy_persona(_world: &mut World, _this: Entity, _guest_pygmy_persona: Entity) {}

/// `-[TribePersona activateGuestPygmyPersona:]` @ 0x1001a5ea4
///
/// An inactive guest becomes active and, unless it is bound to a living pygmy, goes to the
/// front of the dead list: the next resurrection picks it.
pub fn activate_guest_pygmy_persona(world: &mut World, this: Entity, guest_pygmy_persona: Entity) {
    if pygmy_persona::is_active(world, guest_pygmy_persona) {
        return;
    }
    pygmy_persona::set_active(world, guest_pygmy_persona, true);

    let mut tribe_persona = world.obj_mut::<TribePersona>(this);
    if !tribe_persona.living_pygmy_persona_array.contains(&guest_pygmy_persona) {
        tribe_persona.dead_pygmy_persona_array.insert(0, guest_pygmy_persona);
    }
}

/// `-[TribePersona deactivateGuestPygmyPersona:]` @ 0x1001a5f44
pub fn deactivate_guest_pygmy_persona(world: &mut World, this: Entity, guest_pygmy_persona: Entity) {
    if !pygmy_persona::is_active(world, guest_pygmy_persona) {
        return;
    }
    pygmy_persona::set_active(world, guest_pygmy_persona, false);

    // `removeObject:` drops every occurrence.
    let mut tribe_persona = world.obj_mut::<TribePersona>(this);
    if tribe_persona.dead_pygmy_persona_array.contains(&guest_pygmy_persona) {
        tribe_persona.dead_pygmy_persona_array.retain(|dead| *dead != guest_pygmy_persona);
    }
}

/// `-[TribePersona addGift:pygmyLogic:]` @ 0x1001a5fe4
///
/// Counted only for a pygmy that has a persona: on the tribe and on that persona.
pub fn add_gift(world: &mut World, this: Entity, gift_id: &str, pygmy_logic: Entity) {
    let Some(pygmy_persona) = crate::pygmy::pygmy_persona(world, pygmy_logic) else {
        return;
    };

    persona_class::add_gift(world, this, gift_id);
    persona_class::increment_total_gifts(world, this);
    persona_class::add_gift(world, pygmy_persona, gift_id);
    persona_class::increment_total_gifts(world, pygmy_persona);
}

/// `-[TribePersona sacrifice:pygmyLogic:]` @ 0x1001a6090
///
/// A nil sacrifice id counts nothing; the pygmy's persona is unbound either way.
pub fn sacrifice(world: &mut World, this: Entity, sacrifice_id: Option<&str>, pygmy_logic: Entity) {
    if let Some(sacrifice_id) = sacrifice_id {
        persona_class::increment_total_sacrifices(world, this);
        persona_class::add_sacrifice(world, this, sacrifice_id);
    }

    let Some(pygmy_persona) = crate::pygmy::pygmy_persona(world, pygmy_logic) else {
        return;
    };

    if let Some(sacrifice_id) = sacrifice_id {
        persona_class::increment_total_sacrifices(world, pygmy_persona);
        persona_class::add_sacrifice(world, pygmy_persona, sacrifice_id);
    }
    pygmy_persona::set_pygmy_logic(world, pygmy_persona, None);

    // `removeObject:` drops every occurrence.
    world.obj_mut::<TribePersona>(this).living_pygmy_persona_array.retain(|living| *living != pygmy_persona);

    if pygmy_persona::is_active(world, pygmy_persona) {
        world.obj_mut::<TribePersona>(this).dead_pygmy_persona_array.push(pygmy_persona);
    }
}

/// `-[TribePersona resurrectPygmyLogic:]` @ 0x1001a61b4
///
/// Binds the first dead persona to the new pygmy, both ways, and moves it to the living list.
pub fn resurrect_pygmy_logic(world: &mut World, this: Entity, pygmy_logic: Entity) {
    // `objectAtIndex:0` raises on an empty list, as in the original.
    let pygmy_persona = world.obj::<TribePersona>(this).dead_pygmy_persona_array[0];

    pygmy_persona::set_pygmy_logic(world, pygmy_persona, Some(pygmy_logic));
    crate::pygmy::set_pygmy_persona(world, pygmy_logic, pygmy_persona);

    let mut tribe_persona = world.obj_mut::<TribePersona>(this);
    tribe_persona.living_pygmy_persona_array.push(pygmy_persona);
    // `removeObject:` drops every occurrence.
    tribe_persona.dead_pygmy_persona_array.retain(|dead| *dead != pygmy_persona);
}
