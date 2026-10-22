//! `TheRunsTurdLogic` (superclass `SpriteLogic`).
//!
//! A turd on the course of The Runs. The crack hole owns four of them, lays one on a turd
//! event's platforms and recycles it when the event has scrolled away; a pygmy that runs into
//! it skids.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;

/// `TheRunsTurdLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct TheRunsTurdLogic {
    /// `mTheRunsTurdLogicListener`: `@"<TheRunsTurdLogicListener>"`
    pub the_runs_turd_logic_listener: Option<Entity>,

    /// `mbCheckCollision`: `B`
    pub check_collision: bool,
}

pub const THE_RUNS_TURD_LOGIC: Class = Class {
    name: "TheRunsTurdLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"TheRunsPygmyDisplayMarker"`: the turd goes into the course just before it.
const PYGMY_DISPLAY_MARKER: &str = "TheRunsPygmyDisplayMarker";

/// `@"TheRunsTurdInit"`
const INIT_BEHAVIOR: &str = "TheRunsTurdInit";

/// `@"TheRunsTurdSkid"`
const SKID_BEHAVIOR: &str = "TheRunsTurdSkid";

/// `@"BlankAnim"`
const BLANK_BEHAVIOR: &str = "BlankAnim";

/// The squared distance (30²) within which the pygmy hits the turd.
const COLLISION_DISTANCE_SQUARED: f32 = 900.0;

/// `[[TheRunsTurdLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&THE_RUNS_TURD_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[TheRunsTurdLogic initWithDisplayObject:]` @ 0x100067fd0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(TheRunsTurdLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::stop_game_frame(world, this);
}

/// `-[TheRunsTurdLogic setTheRunsTurdLogicListener:]` @ 0x100068034
pub fn set_the_runs_turd_logic_listener(world: &mut World, this: Entity, the_runs_turd_logic_listener: Entity) {
    world.obj_mut::<TheRunsTurdLogic>(this).the_runs_turd_logic_listener = Some(the_runs_turd_logic_listener);
}

/// `-[TheRunsTurdLogic activateAtXPos:yPos:]` @ 0x100068090
pub fn activate_at_x_pos(world: &mut World, this: Entity, x_pos: f32, y_pos: f32) {
    world.obj_mut::<TheRunsTurdLogic>(this).check_collision = true;

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

    pg_engine::logic::set_behavior(world, this, Some(INIT_BEHAVIOR));
}

/// `-[TheRunsTurdLogic deactivate]` @ 0x10006815c
pub fn deactivate(world: &mut World, this: Entity) {
    world.obj_mut::<TheRunsTurdLogic>(this).check_collision = false;

    let display_object = pg_engine::logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    pg_engine::logic::set_behavior(world, this, Some(BLANK_BEHAVIOR));
}

/// `-[TheRunsTurdLogic checkCollision:lastPos:]` @ 0x1000681bc
///
/// Whether the pygmy (`pygmy_logic`) has run into the turd, which then skids away and stops
/// checking. `last_pos` (the pygmy's position on the previous frame) is not used.
pub fn check_collision(world: &mut World, this: Entity, pygmy_logic: Entity, _last_pos: Vec4) -> bool {
    let check_collision = world.obj::<TheRunsTurdLogic>(this).check_collision;
    if !check_collision {
        return false;
    }

    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy_logic);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    let display_object = pg_engine::logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    let delta_x = pos.x - pygmy_pos.x;
    let delta_y = pos.y - pygmy_pos.y;
    let distance_squared = delta_x * delta_x + delta_y * delta_y;
    let hit = distance_squared < COLLISION_DISTANCE_SQUARED;
    if !hit {
        return false;
    }

    world.obj_mut::<TheRunsTurdLogic>(this).check_collision = false;
    pg_engine::logic::set_behavior(world, this, Some(SKID_BEHAVIOR));
    true
}

/// `-[TheRunsTurdLogic onGameFrame:]` @ 0x100068284
///
/// Empty: the turd has none of `SpriteLogic`'s physics.
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}
