//! `TheRunsStalactiteLogic` (superclass `SpriteLogic`).
//!
//! One of the four stalactites of The Runs (`TheRunsDisplayGroup/Stalactite1…4`). The crack
//! hole hangs one over each platform of a stalactite event, makes them fall once the first is
//! near and takes them down again when they leave the screen; a pygmy under one is hit.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;

/// `TheRunsStalactiteLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct TheRunsStalactiteLogic {
    /// `mTheRunsStalactiteLogicListener`: `@"<TheRunsStalactiteLogicListener>"`
    pub the_runs_stalactite_logic_listener: Option<Entity>,

    /// `mbCheckCollision`: `B`
    pub check_collision: bool,
}

pub const THE_RUNS_STALACTITE_LOGIC: Class = Class {
    name: "TheRunsStalactiteLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"TheRunsPygmyDisplayMarker"`: the stalactite goes into the course just before it.
const PYGMY_DISPLAY_MARKER: &str = "TheRunsPygmyDisplayMarker";

/// `@"TheRunsStalactiteInit"`
const INIT_BEHAVIOR: &str = "TheRunsStalactiteInit";

/// `[[TheRunsStalactiteLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&THE_RUNS_STALACTITE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[TheRunsStalactiteLogic initWithDisplayObject:]` @ 0x100113960
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(TheRunsStalactiteLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::set_behavior(world, this, Some(INIT_BEHAVIOR));
    pg_engine::logic::stop_game_frame(world, this);
}

/// `-[TheRunsStalactiteLogic setTheRunsStalactiteLogicListener:]` @ 0x1001139e0
pub fn set_the_runs_stalactite_logic_listener(world: &mut World, this: Entity, the_runs_stalactite_logic_listener: Entity) {
    world.obj_mut::<TheRunsStalactiteLogic>(this).the_runs_stalactite_logic_listener = Some(the_runs_stalactite_logic_listener);
}

/// `-[TheRunsStalactiteLogic isActive]` @ 0x100113a3c
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<TheRunsStalactiteLogic>(this).check_collision
}

/// `-[TheRunsStalactiteLogic activateAtXPos:yPos:]` @ 0x100113a4c
pub fn activate_at_x_pos(world: &mut World, this: Entity, x_pos: f32, y_pos: f32) {
    world.obj_mut::<TheRunsStalactiteLogic>(this).check_collision = true;

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = x_pos;
        object.pos.y = y_pos;
        object.pos.z = SPRITE_Z;
    }

    let marker = pg_engine::library::display_marker(world, PYGMY_DISPLAY_MARKER);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[TheRunsStalactiteLogic deactivate]` @ 0x100113af8
pub fn deactivate(world: &mut World, this: Entity) {
    world.obj_mut::<TheRunsStalactiteLogic>(this).check_collision = false;

    let display_object = pg_engine::logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[TheRunsStalactiteLogic setCheckCollision:]` @ 0x100113b24
pub fn set_check_collision(world: &mut World, this: Entity, check_collision: bool) {
    world.obj_mut::<TheRunsStalactiteLogic>(this).check_collision = check_collision;
}

/// `-[TheRunsStalactiteLogic checkCollision:lastPos:]` @ 0x100113b34
///
/// 1 when the pygmy (`pygmy_logic`) is above the stalactite's tip and within its width
/// (x + 10 to x + 209), otherwise 0. `last_pos` (the pygmy's position on the previous frame)
/// is not used.
pub fn check_collision(world: &mut World, this: Entity, pygmy_logic: Entity, _last_pos: Vec4) -> i32 {
    let check_collision = world.obj::<TheRunsStalactiteLogic>(this).check_collision;
    if !check_collision {
        return 0;
    }

    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy_logic);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    let display_object = pg_engine::logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    let tip_y_pos = pos.y + 0.0;
    let above_tip = pygmy_pos.y > tip_y_pos;
    if !above_tip {
        return 0;
    }

    let under_stalactite = pygmy_pos.x > pos.x + 10.0 && pygmy_pos.x < pos.x + 209.0;
    if under_stalactite {
        return 1;
    }
    0
}

/// `-[TheRunsStalactiteLogic onGameFrame:]` @ 0x100113bf8
///
/// Empty: the stalactite has none of `SpriteLogic`'s physics.
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}
