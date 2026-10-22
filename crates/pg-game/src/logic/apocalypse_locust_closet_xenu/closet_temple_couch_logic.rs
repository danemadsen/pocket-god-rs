//! `ClosetTempleCouchLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use pg_engine::logic;

use crate::logic::island_core::sprite_logic;

/// `ClosetTempleCouchLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ClosetTempleCouchLogic {
    /// `mClosetTempleCouchLogicListener`: `@"<ClosetTempleCouchLogicListener>"`
    pub closet_temple_couch_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const CLOSET_TEMPLE_COUCH_LOGIC: Class = Class {
    name: "ClosetTempleCouchLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `[[ClosetTempleCouchLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&CLOSET_TEMPLE_COUCH_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[ClosetTempleCouchLogic initWithDisplayObject:]` @ 0x10017f4b4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ClosetTempleCouchLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().closet_temple_couch_logic = Some(this);
    logic::stop_game_frame(world, this);
}

/// `-[ClosetTempleCouchLogic setClosetTempleCouchLogicListener:]` @ 0x10017f520
pub fn set_closet_temple_couch_logic_listener(world: &mut World, this: Entity, closet_temple_couch_logic_listener: Entity) {
    world.obj_mut::<ClosetTempleCouchLogic>(this).closet_temple_couch_logic_listener = Some(closet_temple_couch_logic_listener);
}

/// `-[ClosetTempleCouchLogic isAvailable]` @ 0x10017f57c
pub fn is_available(world: &mut World, this: Entity) -> bool {
    world.obj::<ClosetTempleCouchLogic>(this).pygmy_logic.is_none()
}

/// `-[ClosetTempleCouchLogic jumpOntoClosetTempleCouch:]` @ 0x10017f594
pub fn jump_onto_closet_temple_couch(world: &mut World, this: Entity, jump_onto_closet_temple_couch: Entity) {
    world.obj_mut::<ClosetTempleCouchLogic>(this).pygmy_logic = Some(jump_onto_closet_temple_couch);
}

/// `-[ClosetTempleCouchLogic fallOntoClosetTempleCouch:]` @ 0x10017f5a4
pub fn fall_onto_closet_temple_couch(world: &mut World, this: Entity, fall_onto_closet_temple_couch: Entity) {
    world.obj_mut::<ClosetTempleCouchLogic>(this).pygmy_logic = Some(fall_onto_closet_temple_couch);
}

/// `-[ClosetTempleCouchLogic ejectFromClosetTempleCouch:]` @ 0x10017f5b4
pub fn eject_from_closet_temple_couch(world: &mut World, this: Entity, eject_from_closet_temple_couch: Entity) {
    let mut couch = world.obj_mut::<ClosetTempleCouchLogic>(this);
    if couch.pygmy_logic == Some(eject_from_closet_temple_couch) {
        couch.pygmy_logic = None;
    }
}

/// `-[ClosetTempleCouchLogic enterIsland:]` @ 0x10017f5d0
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    if world.obj::<ClosetTempleCouchLogic>(this).active {
        return;
    }
    world.obj_mut::<ClosetTempleCouchLogic>(this).active = true;
    logic::set_behavior(world, this, Some("ClosetTempleCouchEnterIsland"));
}

/// `-[ClosetTempleCouchLogic exitIsland:]` @ 0x10017f604
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    if !world.obj::<ClosetTempleCouchLogic>(this).active {
        return;
    }
    world.obj_mut::<ClosetTempleCouchLogic>(this).active = false;

    // Tom (personality 2) stays on the couch.
    let pygmy = world.obj::<ClosetTempleCouchLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        if crate::pygmy::persona_personality(world, pygmy) != 2 {
            crate::pygmy::eject_from_closet_temple_couch(world, pygmy);
            world.obj_mut::<ClosetTempleCouchLogic>(this).pygmy_logic = None;
        }
    }
    logic::set_behavior(world, this, Some("ClosetTempleCouchExitIsland"));
}

/// `-[ClosetTempleCouchLogic exitIslandNow]` @ 0x10017f698
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[ClosetTempleCouchLogic onGameFrame:]` @ 0x10017f710
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `+[ClosetTempleCouchLogic instance]` @ 0x10017f4a8
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().closet_temple_couch_logic
}
