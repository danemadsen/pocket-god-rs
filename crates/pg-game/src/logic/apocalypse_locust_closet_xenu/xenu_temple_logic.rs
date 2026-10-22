//! `XenuTempleLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use pg_engine::logic;

use crate::logic::island_core::sprite_logic;

/// `XenuTempleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct XenuTempleLogic {
    /// `mXenuTempleLogicListener`: `@"<XenuTempleLogicListener>"`
    pub xenu_temple_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbWallTornDown`: `B`
    pub wall_torn_down: bool,
}

pub const XENU_TEMPLE_LOGIC: Class = Class {
    name: "XenuTempleLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `[[XenuTempleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&XENU_TEMPLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[[IdolManager instance] triggerEpisodeEvent:event]`.
fn trigger_episode_event(world: &mut World, event: &str) {
    let idol_manager = crate::managers::idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event(world, idol_manager, event);
    }
}

/// `-[XenuTempleLogic initWithDisplayObject:]` @ 0x10008db00
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(XenuTempleLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().xenu_temple_logic = Some(this);
    logic::stop_game_frame(world, this);
}

/// `-[XenuTempleLogic setXenuTempleLogicListener:]` @ 0x10008db6c
pub fn set_xenu_temple_logic_listener(world: &mut World, this: Entity, xenu_temple_logic_listener: Entity) {
    world.obj_mut::<XenuTempleLogic>(this).xenu_temple_logic_listener = Some(xenu_temple_logic_listener);
}

/// `-[XenuTempleLogic tearDownWall]` @ 0x10008dbc8
pub fn tear_down_wall(world: &mut World, this: Entity) {
    world.obj_mut::<XenuTempleLogic>(this).wall_torn_down = true;
    logic::set_behavior(world, this, Some("XenuTempleTearDownWall"));
}

/// `-[XenuTempleLogic buildWallBackUp]` @ 0x10008dbf0
pub fn build_wall_back_up(world: &mut World, this: Entity) {
    world.obj_mut::<XenuTempleLogic>(this).wall_torn_down = false;
}

/// `-[XenuTempleLogic enterIsland:]` @ 0x10008dc00
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    if world.obj::<XenuTempleLogic>(this).active {
        return;
    }

    let (wall_torn_down, listener) = {
        let mut xenu_temple = world.obj_mut::<XenuTempleLogic>(this);
        xenu_temple.active = true;
        (xenu_temple.wall_torn_down, xenu_temple.xenu_temple_logic_listener)
    };
    let behavior = if wall_torn_down {
        "XenuTempleWallTornDown"
    } else {
        "XenuTempleEnterIsland"
    };
    logic::set_behavior(world, this, Some(behavior));

    let Some(listener) = listener else {
        return;
    };
    if crate::controller::is_tom_pygmy_active(world, listener) {
        trigger_episode_event(world, "BringTomToXenuRoom");
    }
    if crate::controller::is_baby_pygmy_active(world, listener) {
        trigger_episode_event(world, "BringBabyToXenuRoom");
    }
}

/// `-[XenuTempleLogic exitIsland:]` @ 0x10008dd0c
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    if !world.obj::<XenuTempleLogic>(this).active {
        return;
    }
    world.obj_mut::<XenuTempleLogic>(this).active = false;
    logic::set_behavior(world, this, Some("XenuTempleExitIsland"));
}

/// `-[XenuTempleLogic exitIslandNow]` @ 0x10008dd3c
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[XenuTempleLogic onGameFrame:]` @ 0x10008ddb4
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `+[XenuTempleLogic instance]` @ 0x10008daf4
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().xenu_temple_logic
}
