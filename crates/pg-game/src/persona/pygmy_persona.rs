//! `PygmyPersona` (superclass `Persona`).

use crate::persona::persona::{self as persona_class, Persona};
use crate::prelude::*;

/// `PygmyPersona` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyPersona {
    /// `mbActive`: `B`
    pub active: bool,

    /// `mPersonaPersonality`: `i`
    pub persona_personality: i32,

    /// `mPygmyLogic`: `@"PygmyLogic"` (retained: the pygmy must outlive the binding, which
    /// `-[TribePersona sacrifice:pygmyLogic:]` undoes)
    pub pygmy_logic: Option<Entity>,
}

pub const PYGMY_PERSONA: Class = Class {
    name: "PygmyPersona",
    superclass: Some(&crate::persona::persona::PERSONA),
    ..crate::persona::persona::PERSONA
};

/// `-[PygmyPersona initWithId:defaultName:defaultDescription:preferences:]` @ 0x1001a6278
pub fn init_with_id(world: &mut World, this: Entity, id: &str, default_name: &str, default_description: &str, preferences: Entity) {
    world.entity_mut(this).insert(PygmyPersona::default());
    persona_class::init_with_id(world, this, id, default_name, default_description, preferences);

    let mut pygmy_persona = world.obj_mut::<PygmyPersona>(this);
    pygmy_persona.pygmy_logic = None;
    pygmy_persona.persona_personality = 0;
}

/// `-[PygmyPersona dealloc]` @ 0x1001a62d0
///
/// For the persona's last owner. Releasing `mPygmyLogic` leaves the pygmy to its other owners.
pub fn dealloc(world: &mut World, this: Entity) {
    persona_class::dealloc(world, this);
}

/// `-[PygmyPersona setActive:]` @ 0x1001a6334
pub fn set_active(world: &mut World, this: Entity, active: bool) {
    world.obj_mut::<PygmyPersona>(this).active = active;
}

/// `-[PygmyPersona isActive]` @ 0x1001a6344
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyPersona>(this).active
}

/// `-[PygmyPersona setPersonaPersonality:]` @ 0x1001a6354
pub fn set_persona_personality(world: &mut World, this: Entity, persona_personality: i32) {
    world.obj_mut::<PygmyPersona>(this).persona_personality = persona_personality;
}

/// `-[PygmyPersona personaPersonality]` @ 0x1001a6364
pub fn persona_personality(world: &mut World, this: Entity) -> i32 {
    world.obj::<PygmyPersona>(this).persona_personality
}

/// `-[PygmyPersona setPygmyLogic:]` @ 0x1001a6374
pub fn set_pygmy_logic(world: &mut World, this: Entity, pygmy_logic: Option<Entity>) {
    world.obj_mut::<PygmyPersona>(this).pygmy_logic = pygmy_logic;

    let name = world.obj::<Persona>(this).name.clone();
    if let Some(pygmy_logic) = pygmy_logic {
        crate::pygmy::set_pygmy_name_label(world, pygmy_logic, name.as_deref());
    }
}

/// `-[PygmyPersona pygmyLogic]` @ 0x1001a63ec
pub fn pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyPersona>(this).pygmy_logic
}

/// `-[PygmyPersona setName:]` @ 0x1001a63fc
pub fn set_name(world: &mut World, this: Entity, name: &str) {
    persona_class::set_name(world, this, name);

    let pygmy_logic = world.obj::<PygmyPersona>(this).pygmy_logic;
    let name = world.obj::<Persona>(this).name.clone();
    if let Some(pygmy_logic) = pygmy_logic {
        crate::pygmy::set_pygmy_name_label(world, pygmy_logic, name.as_deref());
    }
}
