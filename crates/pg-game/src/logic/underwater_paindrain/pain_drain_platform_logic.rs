//! `PainDrainPlatformLogic` (superclass `SpriteLogic`).
//!
//! One of the four rising floors of the Pain Drain mini-game. Each floor spans the screen except
//! for a gap 80 wide around its x, which the pygmy falls through. A floor that rises past the top
//! wraps to the bottom, below the floor placed before it.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;

/// `PainDrainPlatformLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct PainDrainPlatformLogic {
    /// `mPainDrainPlatformLogicListener`: `@"<PainDrainPlatformLogicListener>"`
    pub pain_drain_platform_logic_listener: Option<Entity>,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

/// The file-scope placement state the platforms share: `moveToBottom` places each platform
/// below the one placed before it.
#[derive(Resource, Clone, Debug, Default)]
pub struct PainDrainPlatformLogicStatics {
    /// `DAT_1007e7c08`: set once a placement's random x offset is under 60; from then on the
    /// gap spacing is a fixed 60. Never cleared, so it carries over to later games.
    pub fixed_gap_spacing: bool,

    /// `DAT_1007e7c0c`: the least horizontal distance between a new platform's gap and the gap
    /// of the platform placed before it: 120 for a game's first platform, one less after each
    /// placement, never below 0.
    pub gap_spacing: f32,

    /// `DAT_1007e7c18`: the platform placed last (not retained). `DAT_1007e7c10`, the pointer to
    /// its display object's `pos` that `moveToBottom` reads, always goes with it.
    pub previous_platform: Option<Entity>,
}

pub const PAIN_DRAIN_PLATFORM_LOGIC: Class = Class {
    name: "PainDrainPlatformLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Where `moveToBottom` puts a platform when no platform was placed before it.
const FIRST_PLATFORM_X_POS: f32 = 100.0;
const FIRST_PLATFORM_Y_POS: f32 = 60.0;

/// The gap spacing of a game's first platform.
const FIRST_GAP_SPACING: f32 = 120.0;

/// The gap spacing once `DAT_1007e7c08` is set.
const FIXED_GAP_SPACING: f32 = 60.0;

/// The gaps stay between these x positions.
const GAP_MIN_X_POS: f32 = 60.0;
const GAP_MAX_X_POS: f32 = 420.0;

/// Half the width of a platform's gap.
const GAP_HALF_WIDTH: f32 = 40.0;

/// A platform rising above this wraps to the bottom.
const WRAP_Y_POS: f32 = 356.0;

/// The placement state (all zero before the first platform).
fn statics(world: &World) -> PainDrainPlatformLogicStatics {
    match world.get_resource::<PainDrainPlatformLogicStatics>() {
        Some(statics) => statics.clone(),
        None => PainDrainPlatformLogicStatics::default(),
    }
}

/// The placement state, for writing.
fn statics_mut(world: &mut World) -> Mut<'_, PainDrainPlatformLogicStatics> {
    world.get_resource_or_insert_with(PainDrainPlatformLogicStatics::default)
}

/// `[[PainDrainPlatformLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&PAIN_DRAIN_PLATFORM_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[PainDrainPlatformLogic initWithDisplayObject:]` @ 0x1000cdc74
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(PainDrainPlatformLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    // The first platform since the last ones were released starts a new game's spacing.
    let previous_platform = statics(world).previous_platform;
    if previous_platform.is_none() {
        statics_mut(world).gap_spacing = FIRST_GAP_SPACING;
    }

    move_to_bottom(world, this);
}

/// `-[PainDrainPlatformLogic setPainDrainPlatformLogicListener:]` @ 0x1000cdcf0
pub fn set_pain_drain_platform_logic_listener(world: &mut World, this: Entity, pain_drain_platform_logic_listener: Entity) {
    world.obj_mut::<PainDrainPlatformLogic>(this).pain_drain_platform_logic_listener = Some(pain_drain_platform_logic_listener);
}

/// `-[PainDrainPlatformLogic dealloc]` @ 0x1000ce154
///
/// When the drain's platform array lets go of the platform (`removePlatforms`): the placement
/// forgets it if it was placed last, so the next game starts afresh. `BCLogic` then releases
/// the `Platform` multi-model, whose only other owner (the display tree) the drain already
/// removed.
pub fn dealloc(world: &mut World, this: Entity) {
    let previous_platform = statics(world).previous_platform;
    if previous_platform == Some(this) {
        statics_mut(world).previous_platform = None;
    }

    let display_object = logic::display_object(world, this);

    // `[mPainDrainPlatformLogicListener release]` needs no port; `[super dealloc]`.
    logic::dealloc(world, this);

    let out_of_display_tree = match world.get::<DisplayObject>(display_object) {
        Some(object) => object.display_group.is_none(),
        None => false,
    };
    if out_of_display_tree {
        world.despawn(display_object);
    }
}

/// `-[PainDrainPlatformLogic moveToBottom]` @ 0x1000cdd4c
///
/// Puts the first platform at (100, 60). Any later one goes 100 to 150 below the platform placed
/// before it, its gap at a random x in 60…420 at least the gap spacing to the left or right of
/// that platform's gap.
pub fn move_to_bottom(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = FIRST_PLATFORM_X_POS;
        object.pos.y = FIRST_PLATFORM_Y_POS;
        object.pos.z = SPRITE_Z;
    }

    let placement = statics(world);
    if let Some(previous_platform) = placement.previous_platform {
        // `DAT_1007e7c10` points at the previous platform's live `pos`. For this very platform
        // that is the position just reset above.
        let previous_display_object = logic::display_object(world, previous_platform);

        let y_roll = world.resource_mut::<Random>().arc4random_mod(5001);
        let y_step = y_roll as i32 - 15000;
        let y_offset = y_step as f32 / 100.0;
        let previous_y_pos = display::object(world, previous_display_object).pos.y;
        display::object_mut(world, display_object).pos.y = y_offset + previous_y_pos;

        let spacing = if placement.fixed_gap_spacing { FIXED_GAP_SPACING } else { placement.gap_spacing };
        let previous_x_pos = display::object(world, previous_display_object).pos.x;
        let min_x_offset = spacing + (GAP_MIN_X_POS - previous_x_pos);
        let max_x_offset = (GAP_MAX_X_POS - previous_x_pos) - spacing;

        // The offset is drawn in hundredths from `min_x_step` up to `max_x_step` (exclusive).
        let min_x_step = if min_x_offset > 0.0 { 0 } else { (min_x_offset * 100.0) as i32 };
        let max_x_step = if max_x_offset < 0.0 {
            1
        } else {
            let truncated_max_x_step = (max_x_offset * 100.0) as i32;
            truncated_max_x_step.wrapping_add(1)
        };
        let x_step_count = max_x_step.wrapping_sub(min_x_step) as u32;
        let x_roll = world.resource_mut::<Random>().arc4random_mod(x_step_count);
        let x_step = min_x_step.wrapping_add(x_roll as i32);
        let x_offset = x_step as f32 / 100.0;

        // A positive offset puts the gap to the right of the previous one, otherwise to the left.
        let shifted_x_pos = x_offset + previous_x_pos;
        let x_pos = if x_offset > 0.0 { spacing + shifted_x_pos } else { shifted_x_pos - spacing };
        display::object_mut(world, display_object).pos.x = x_pos;

        let mut statics = statics_mut(world);
        let reduced_gap_spacing = statics.gap_spacing - 1.0;
        statics.gap_spacing = if reduced_gap_spacing < 0.0 { 0.0 } else { reduced_gap_spacing };

        // The first offset drawn under 60 switches to the fixed spacing for good.
        if x_offset.abs() < 60.0 {
            statics.fixed_gap_spacing = true;
        }
    }

    logic::set_behavior(world, this, Some("PainDrainPlatformInit"));

    statics_mut(world).previous_platform = Some(this);
}

/// `-[PainDrainPlatformLogic checkCollision:lastPos:]` @ 0x1000cdf20
///
/// A pygmy outside the gap that was not below the platform last frame and is now at or under its
/// top lands on it (`vel.y = 0`, `y = platform.y`). The platform keeps carrying its rider until
/// the rider is over the gap; any other pygmy that does not land clears the rider.
pub fn check_collision(world: &mut World, this: Entity, pygmy_logic: Entity, last_pos: Vec4) -> bool {
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    let (own_last_pos, rider) = {
        let platform = world.obj::<PainDrainPlatformLogic>(this);
        (platform.last_pos, platform.pygmy_logic)
    };

    let was_below = last_pos.y < own_last_pos.y;
    if was_below && rider.is_none() {
        return false;
    }

    let x_offset = pygmy_pos.x - pos.x;
    let outside_gap = x_offset < -GAP_HALF_WIDTH || x_offset > GAP_HALF_WIDTH;
    if outside_gap {
        let crossed_top = pygmy_pos.y <= pos.y && last_pos.y >= own_last_pos.y;
        if crossed_top || rider == Some(pygmy_logic) {
            logic::logic_mut(world, pygmy_logic).pos_vel.y = 0.0;
            display::object_mut(world, pygmy_display_object).pos.y = pos.y;
            world.obj_mut::<PainDrainPlatformLogic>(this).pygmy_logic = Some(pygmy_logic);
            return true;
        }
    }

    world.obj_mut::<PainDrainPlatformLogic>(this).pygmy_logic = None;
    false
}

/// `-[PainDrainPlatformLogic onGameFrame:]` @ 0x1000ce06c
///
/// Remembers the position, moves by the velocity the drain sets and wraps to the bottom above
/// y = 356 (no `[super onGameFrame:]`).
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;
    let pos_vel = state.pos_vel;

    let pos = display::object(world, display_object).pos;
    world.obj_mut::<PainDrainPlatformLogic>(this).last_pos = pos;

    let y_pos = {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = object.pos.x + pos_vel.x * dt;
        object.pos.y = object.pos.y + pos_vel.y * dt;
        object.pos.z = object.pos.z + pos_vel.z * dt;
        object.pos.y
    };

    if y_pos > WRAP_Y_POS {
        move_to_bottom(world, this);
    }
}
