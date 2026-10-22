//! `NoPeePillarLogic` (superclass `SpriteLogic`).
//!
//! Dead code in 1.48.2: nothing creates a pillar. Ported for completeness.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `NoPeePillarLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeePillarLogic {
    /// `mNoPeePillarLogicListener`: `@"<NoPeePillarLogicListener>"`
    pub no_pee_pillar_logic_listener: Option<Entity>,

    /// `mNoPeePillarType`: `i`
    pub no_pee_pillar_type: i32,

    /// `mNoPeePathMarkerCollisionMaxXPos`: `f`
    pub no_pee_path_marker_collision_max_x_pos: f32,

    /// `mNoPeePathMarkerCollisionMinXPos`: `f`
    pub no_pee_path_marker_collision_min_x_pos: f32,
}

pub const NO_PEE_PILLAR_LOGIC: Class = Class {
    name: "NoPeePillarLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The obstacle box of the "flappy" and collapsing tests: 80 wide from the pillar's x, with a
/// gap of ±40 around its y.
const OBSTACLE_WIDTH: f32 = 80.0;
const OBSTACLE_GAP_HALF_HEIGHT: f32 = 40.0;

/// The tunnel's half width, measured along the pillar's normal.
const TUNNEL_HALF_WIDTH: f32 = 30.0;

/// `[[NoPeePillarLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_PILLAR_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeePillarLogic initWithDisplayObject:]` @ 0x1000d5454
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeePillarLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);
}

/// `-[NoPeePillarLogic setNoPeePillarLogicListener:]` @ 0x1000d54b8
pub fn set_no_pee_pillar_logic_listener(world: &mut World, this: Entity, no_pee_pillar_logic_listener: Entity) {
    world.obj_mut::<NoPeePillarLogic>(this).no_pee_pillar_logic_listener = Some(no_pee_pillar_logic_listener);
}

/// `-[NoPeePillarLogic setPathMarkerCollisionMinXPos:maxXPos:]` @ 0x1000d5514
pub fn set_path_marker_collision_min_x_pos(world: &mut World, this: Entity, min_x_pos: f32, max_x_pos: f32) {
    let mut pillar = world.obj_mut::<NoPeePillarLogic>(this);
    pillar.no_pee_path_marker_collision_min_x_pos = min_x_pos;
    pillar.no_pee_path_marker_collision_max_x_pos = max_x_pos;
}

/// `-[NoPeePillarLogic checkCollision:lastPos:]` @ 0x1000d5530
pub fn check_collision(world: &mut World, this: Entity, check_collision: Entity, last_pos: Option<usize>) -> bool {
    check_tunnel_collision(world, this, check_collision, last_pos)
}

/// `-[NoPeePillarLogic checkFlappyCollision:lastPos:]` @ 0x1000d5540
///
/// Within the obstacle's 80 points of x, anything outside the gap around its y collides.
pub fn check_flappy_collision(world: &mut World, this: Entity, check_flappy_collision: Entity, _last_pos: Option<usize>) -> bool {
    is_outside_obstacle_gap(world, this, check_flappy_collision)
}

/// `-[NoPeePillarLogic checkTunnelCollision:lastPos:]` @ 0x1000d5600
///
/// Between the configured x range, anything outside the slanted tunnel (along the pillar's
/// rotation, `TUNNEL_HALF_WIDTH` on each side) collides.
pub fn check_tunnel_collision(world: &mut World, this: Entity, check_tunnel_collision: Entity, _last_pos: Option<usize>) -> bool {
    let other_display_object = logic::display_object(world, check_tunnel_collision);
    let other_pos = display::object(world, other_display_object).pos;
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let z_rot = display::object(world, display_object).rot.z;

    let pillar = world.obj::<NoPeePillarLogic>(this);
    let min_x_pos = pillar.no_pee_path_marker_collision_min_x_pos;
    let max_x_pos = pillar.no_pee_path_marker_collision_max_x_pos;
    if other_pos.x < min_x_pos || other_pos.x > max_x_pos {
        return false;
    }

    let angle = ((z_rot + -90.0) * std::f32::consts::PI) / 180.0;
    let half_width = angle.sin() * TUNNEL_HALF_WIDTH;
    let x_distance = pos.x - other_pos.x;
    let tunnel_top = half_width + x_distance / angle.tan() + pos.y;
    if other_pos.y > tunnel_top {
        return true;
    }

    let tunnel_bottom = tunnel_top + half_width * -2.0;
    other_pos.y < tunnel_bottom
}

/// `-[NoPeePillarLogic checkCollapsingCollision:lastPos:]` @ 0x1000d5740
///
/// The same test as `checkFlappyCollision:lastPos:`.
pub fn check_collapsing_collision(world: &mut World, this: Entity, check_collapsing_collision: Entity, _last_pos: Option<usize>) -> bool {
    is_outside_obstacle_gap(world, this, check_collapsing_collision)
}

/// The body of `checkFlappyCollision:lastPos:` and `checkCollapsingCollision:lastPos:`:
/// `x < other.x < x + 80` and `other.y` outside `y ± 40`.
fn is_outside_obstacle_gap(world: &World, this: Entity, other: Entity) -> bool {
    let other_display_object = logic::display_object(world, other);
    let other_pos = display::object(world, other_display_object).pos;
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    let within_x = other_pos.x > pos.x && other_pos.x < pos.x + OBSTACLE_WIDTH;
    if !within_x {
        return false;
    }

    let gap_top = pos.y + OBSTACLE_GAP_HALF_HEIGHT;
    let gap_bottom = pos.y + -OBSTACLE_GAP_HALF_HEIGHT;
    other_pos.y > gap_top || other_pos.y < gap_bottom
}

/// `-[NoPeePillarLogic setNoPeePillarType:]` @ 0x1000d5800
pub fn set_no_pee_pillar_type(world: &mut World, this: Entity, no_pee_pillar_type: i32) {
    world.obj_mut::<NoPeePillarLogic>(this).no_pee_pillar_type = no_pee_pillar_type;

    if no_pee_pillar_type == 1 {
        logic::set_behavior(world, this, Some("NoPeePillarLargeObstacle"));
        return;
    }
    if no_pee_pillar_type == 0 {
        logic::set_behavior(world, this, Some("NoPeePillarPathMarker"));
    }
}

/// `-[NoPeePillarLogic onGameFrame:]` @ 0x1000d584c
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}
