//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::island_core::sprite_logic::SpriteLogic;
use crate::logic::nopee::no_pee_logic;
use crate::logic::sky_powers::hurricane_logic;
use crate::logic::sky_powers::whirl_pool_logic;
use crate::logic::underwater_baitmaster_laser::chum_logic;
use crate::logic::underwater_baitmaster_laser::piranha_logic;
use crate::logic::underwater_paindrain::pain_drain_platform_logic;
use crate::logic::underwater_paindrain::tile_drain_logic::{self, TileDrainLogic};
use crate::logic::underwater_stage::jellyfish_logic;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::touch::BcTouch;

/// Tweenable channel of the pygmy's animation that carries the No Pee embed bubbles.
const NO_PEE_EMBED_CHANNEL: usize = 17;

/// Jellyfish attacks reach a pygmy within this distance (`checkJellyfishCollision`).
const JELLYFISH_REACH: f32 = 30.0;

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyUnderwaterNopee {
    /// `mEatingPiranhaLogic`: `@"PiranhaLogic"`
    pub eating_piranha_logic: Option<Entity>,

    /// `mbBaitMaster`: `B`
    pub bait_master: bool,

    /// `mHoldingChumLogic`: `@"ChumLogic"`
    pub holding_chum_logic: Option<Entity>,

    /// `mbHoldingChum`: `B`
    pub holding_chum: bool,

    /// `mbPainDrainPlatformCollision`: `B`
    pub pain_drain_platform_collision: bool,

    /// `mbPainDrainCanTurn`: `B`
    pub pain_drain_can_turn: bool,

    /// `mbPainDrainTurnRight`: `B`
    pub pain_drain_turn_right: bool,

    /// `mbNoPeeIntroComplete`: `B`
    pub no_pee_intro_complete: bool,

    /// `mbNoPeeStarted`: `B`
    pub no_pee_started: bool,

    /// `mbNoPeeKilled`: `B`
    pub no_pee_killed: bool,

    /// `mbNoPeeThrustOn`: `B`
    pub no_pee_thrust_on: bool,

    /// `mbNoPeeChangeDirection`: `B`
    pub no_pee_change_direction: bool,

    /// `mbNoPeeTurnToStone`: `B`
    pub no_pee_turn_to_stone: bool,

    /// `mNoPeeThrustVel`: `f`
    pub no_pee_thrust_vel: f32,

    /// `mNoPeeTurnAroundTime`: `f`
    pub no_pee_turn_around_time: f32,

    /// `mNoPeeSwimXVel`: `f`
    pub no_pee_swim_x_vel: f32,

    /// `mNoPeeSwimTargetXVel`: `f`
    pub no_pee_swim_target_x_vel: f32,

    /// `mbNoPeeSwimmingLeft`: `B`
    pub no_pee_swimming_left: bool,

    /// `mbNoPeeChangingDirectionDownward`: `B`
    pub no_pee_changing_direction_downward: bool,

    /// `mNoPeePosVel`: `{?="x"f"y"f"z"f"w"f}`
    pub no_pee_pos_vel: Vec4,

    /// `mNoPeeEmbedLogic`: `@"MultiModelLogic"`
    pub no_pee_embed_logic: Option<Entity>,

    /// `mJellyfishLogic`: `@"JellyfishLogic"`
    pub jellyfish_logic: Option<Entity>,

    /// `mbAddedUnderwater`: `B`
    pub added_underwater: bool,

    /// `mbActiveUnderwater`: `B`
    pub active_underwater: bool,

    /// `mbSkeweredRight`: `B`
    pub skewered_right: bool,

    /// `mbHitBySharkLaser`: `B`
    pub hit_by_shark_laser: bool,

    /// `mbEatenByLaserShark`: `B`
    pub eaten_by_laser_shark: bool,

    /// `mbHitByJellyfish`: `B`
    pub hit_by_jellyfish: bool,

    /// `mbHitByUnderwaterPower`: `B`
    pub hit_by_underwater_power: bool,
}

/// `mPygmyLogicListener`: always the view controller (docs/re/pygmy.md §23).
fn listener_of(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener
}

/// `mPosVel.x = mPosVel.y = 0` (z and w keep their values).
fn stop_moving(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;
}

/// `[mDisplayObject setZRot:0]` and `[mDisplayObject setIsSelectable:NO]`.
fn hold_still_unselectable(world: &mut World, this: Entity) {
    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.is_selectable = false;
}

/// `[mDisplayObject setIsSelectable:]`.
fn set_selectable(world: &mut World, this: Entity, selectable: bool) {
    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = selectable;
}

/// `mSacrificeId = @"…"`: what the pygmy's death will be recorded as.
fn set_sacrifice_id(world: &mut World, this: Entity, sacrifice_id: &str) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(sacrifice_id.to_owned());
}

/// `[mPickedUpByWhirlPoolLogic dropSpriteLogic:self]`, then the reference is cleared. The
/// whirlpool forgets the pygmy without calling back `droppedByWhirlPool:`, and
/// `mbPickedUpByWhirlPool` keeps its value.
fn drop_from_whirl_pool(world: &mut World, this: Entity) {
    let whirl_pool = world.obj::<SpriteLogic>(this).picked_up_by_whirl_pool_logic;
    if let Some(whirl_pool) = whirl_pool {
        whirl_pool_logic::drop_sprite_logic(world, whirl_pool, this);
        world.obj_mut::<SpriteLogic>(this).picked_up_by_whirl_pool_logic = None;
    }
}

/// `[listener pygmyUnderwaterPygmyDisplayMarker]` (nil for a nil listener).
fn underwater_pygmy_display_marker(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = listener_of(world, this)?;
    crate::controller::pygmy_underwater_pygmy_display_marker(world, listener)
}

/// `[marker insertAfter:mDisplayObject]`: the pygmy moves into the marker's display group,
/// right after the marker (a nil marker does nothing).
fn insert_after_marker(world: &mut World, this: Entity, marker: Option<Entity>) {
    let Some(marker) = marker else {
        return;
    };
    let display_object = pg_engine::logic::display_object(world, this);
    display::insert_after(world, marker, Some(display_object));
}

/// `[listener noPeeLogic:self]` (nil for a nil listener).
fn no_pee_logic_of(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = listener_of(world, this)?;
    crate::controller::no_pee_logic(world, listener, this)
}

/// `[listener tileDrainLogic:self]` (nil for a nil listener).
fn tile_drain_logic_of(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = listener_of(world, this)?;
    crate::controller::tile_drain_logic(world, listener, this)
}

/// The drag-end block of the underwater transitions (docs/re/pygmy.md §1.3): a dragged pygmy
/// is let go, its drag label offsets reset and the listener told.
fn end_drag(world: &mut World, this: Entity) {
    let dragging = world.obj::<PygmyPygmyCoreFrame>(this).dragging;
    if !dragging {
        return;
    }

    world.obj_mut::<PygmyPygmyTouchDrag>(this).over_fire = false;
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.dragging = false;
        core.apply_spring = false;
        core.drag_label_x_offset = 0.0;
        core.drag_label_y_offset = 0.0;
    }

    let listener = listener_of(world, this);
    if let Some(listener) = listener {
        crate::controller::on_pygmy_drag_complete(world, listener, this);
    }
}

/// The guard of `startUnderwater`: true when the pygmy is in none of these 58 states.
fn can_start_underwater(world: &World, this: Entity) -> bool {
    let picked_up_by_hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let tom_in_closet = core.persona_personality == 2 && core.in_closet_temple_closet_door;

    let busy = tom_in_closet
        || core.touch_began_inside
        || core.remove_from_story_mode
        || core.tilt_state != 0
        || core.cut_in_half_by_charlie
        || core.time_temple_pygmy_type != 0
        || picked_up_by_hurricane
        || core.dropped_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.eaten_by_locust_queen
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.trapped_in_spider_web
        || core.zombie_explode
        || core.is_zombie
        || core.is_ghost
        || core.touched_by_ghost
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.catch_shark
        || core.shark_on_head
        || core.swarmed
        || core.caught_by_bird
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.in_crack_hole
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.beheaded_by_squid
        || core.cooked
        || core.killed_by_plague
        || core.drowning;

    !busy
}

/// The early returns of `waitUnderwater` after its Tom, Xenu and brawl cases: true when the
/// pygmy is in none of these 57 states.
fn can_wait_underwater(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    let busy = core.touch_began_inside
        || core.remove_from_story_mode
        || core.tilt_state != 0
        || core.cut_in_half_by_charlie
        || core.time_temple_pygmy_type != 0
        || core.dropped_by_hurricane
        || core.hit_by_lightning
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.eaten_by_locust_queen
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.slamming_closet_temple_closet_door
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.trapped_in_spider_web
        || core.zombie_explode
        || core.is_zombie
        || core.is_ghost
        || core.touched_by_ghost
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.catch_shark
        || core.shark_on_head
        || core.swarmed
        || core.caught_by_bird
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.in_crack_hole
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.beheaded_by_squid
        || core.cooked
        || core.killed_by_plague
        || core.drowning
        || core.underwater;

    !busy
}

/// The guard of `suckedIntoDrain:`: true when the pygmy is in none of these 59 states.
fn can_be_sucked_into_drain(world: &World, this: Entity) -> bool {
    let sprite = world.obj::<SpriteLogic>(this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    let busy = core.touch_began_inside
        || core.remove_from_story_mode
        || core.tilt_state != 0
        || core.cut_in_half_by_charlie
        || core.time_temple_pygmy_type != 0
        || sprite.picked_up_by_hurricane
        || core.dropped_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.trapped_in_spider_web
        || core.zombie_explode
        || core.is_zombie
        || core.is_ghost
        || core.touched_by_ghost
        || core.possessed_by_thetan
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.catch_shark
        || core.shark_on_head
        || core.swarmed
        || core.caught_by_bird
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.beheaded_by_squid
        || core.dragging
        || core.cooked
        || core.killed_by_plague
        || sprite.picked_up_by_whirl_pool
        || core.drowning;

    !busy
}

/// The guard of `startPainDrain`: true when the pygmy is in none of these 56 states.
fn can_start_pain_drain(world: &World, this: Entity) -> bool {
    let picked_up_by_hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    let busy = core.touch_began_inside
        || core.remove_from_story_mode
        || core.tilt_state != 0
        || core.cut_in_half_by_charlie
        || core.time_temple_pygmy_type != 0
        || picked_up_by_hurricane
        || core.dropped_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.eaten_by_piranha
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.trapped_in_spider_web
        || core.zombie_explode
        || core.is_zombie
        || core.is_ghost
        || core.touched_by_ghost
        || core.possessed_by_thetan
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.catch_shark
        || core.shark_on_head
        || core.swarmed
        || core.caught_by_bird
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.beheaded_by_squid
        || core.cooked
        || core.killed_by_plague
        || core.drowning;

    !busy
}

/// `-[PygmyLogic (Island) startSinking:::]` @ 0x1000bac8c
pub fn seq_start_sinking(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyUnderwaterNopee>(this).active_underwater = true;
    set_selectable(world, this, true);

    let bait_master = world.obj::<PygmyUnderwaterNopee>(this).bait_master;
    if bait_master {
        start_bait_master(world, this);
    }
    ItemResult::Done
}

/// `-[PygmyLogic (Underwater) stopBeingEatenByPiranha]` @ 0x10011bc64
pub fn stop_being_eaten_by_piranha(world: &mut World, this: Entity) {
    let piranha = world.obj::<PygmyUnderwaterNopee>(this).eating_piranha_logic;
    let Some(piranha) = piranha else {
        return;
    };

    piranha_logic::stop_eating_pygmy(world, piranha, this);
    world.obj_mut::<PygmyUnderwaterNopee>(this).eating_piranha_logic = None;
    set_selectable(world, this, true);
}

/// `-[PygmyLogic (Underwater) stopHoldingChum]` @ 0x10011bcd8
pub fn stop_holding_chum(world: &mut World, this: Entity) {
    let chum = world.obj::<PygmyUnderwaterNopee>(this).holding_chum_logic;
    let Some(chum) = chum else {
        return;
    };

    chum_logic::drop_chum(world, chum, this);

    let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
    state.holding_chum_logic = None;
    state.holding_chum = false;
}

/// `-[PygmyLogic (Underwater) checkPainDrainCollision:]` @ 0x10011bd30
///
/// The pain drain mini-game, each frame: lands on and falls off the platforms, turns, and is
/// spiked on leaving `0 <= y <= 350`.
pub fn check_pain_drain_collision(world: &mut World, this: Entity, last_pos: Vec4) {
    let (pain_drain, pain_drain_spiked) = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        (core.pain_drain, core.pain_drain_spiked)
    };
    if !pain_drain || pain_drain_spiked {
        return;
    }

    let tile_drain = tile_drain_logic_of(world, this);

    // `painDrainPlatformLogicArray` only returns the ivar.
    let platforms = match tile_drain {
        Some(tile_drain) => {
            let drain = world.obj::<TileDrainLogic>(tile_drain);
            drain.pain_drain_platform_logic_array.clone()
        }
        None => Vec::new(),
    };

    let mut platform_collision = false;
    for platform in platforms {
        let hit = pain_drain_platform_logic::check_collision(world, platform, this, last_pos);
        if hit {
            platform_collision = true;
            break;
        }
    }

    let previous_collision = world.obj::<PygmyUnderwaterNopee>(this).pain_drain_platform_collision;
    if previous_collision != platform_collision {
        world.obj_mut::<PygmyUnderwaterNopee>(this).pain_drain_platform_collision = platform_collision;

        let behavior = if platform_collision {
            "PygmyPainDrainLand"
        } else {
            world.obj_mut::<PygmyUnderwaterNopee>(this).pain_drain_can_turn = false;
            "PygmyPainDrainFall"
        };
        pg_engine::logic::set_behavior(world, this, Some(behavior));
    }

    let display_object = pg_engine::logic::display_object(world, this);

    let (on_platform, can_turn) = {
        let state = world.obj::<PygmyUnderwaterNopee>(this);
        (state.pain_drain_platform_collision, state.pain_drain_can_turn)
    };
    if on_platform && can_turn {
        let x_vel = pg_engine::logic::logic(world, this).pos_vel.x;
        let x_scale = display::object(world, display_object).scale.x;
        if x_vel * x_scale < 0.0 {
            world.obj_mut::<PygmyUnderwaterNopee>(this).pain_drain_can_turn = false;
            pg_engine::logic::set_behavior(world, this, Some("PygmyPainDrainTurn"));
        }
    }

    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);
    let out_of_bounds = min_pos.y < 0.0 || max_pos.y > 350.0;
    if !out_of_bounds {
        return;
    }

    {
        let mut object = display::object_mut(world, display_object);
        if min_pos.y < 0.0 {
            object.pos.y = (object.pos.y - min_pos.y) + 0.0;
        }
        if max_pos.y > 350.0 {
            object.pos.y = 350.0 - (max_pos.y - object.pos.y);
        }
    }

    stop_moving(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).pain_drain_spiked = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    if let Some(tile_drain) = tile_drain {
        tile_drain_logic::stop_pain_drain(world, tile_drain);
    }
    pg_engine::logic::set_behavior(world, this, Some("PygmyPainDrainSpiked"));
}

/// `-[PygmyLogic (Underwater) pickedUpByWhirlPool:]` @ 0x10011c0b0
pub fn picked_up_by_whirl_pool(world: &mut World, this: Entity, picked_up_by_whirl_pool: Entity) -> bool {
    let display_object = pg_engine::logic::display_object(world, this);

    let can_be_picked_up = {
        let selectable = display::object(world, display_object).is_selectable;
        let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_whirl_pool;
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let state = world.obj::<PygmyUnderwaterNopee>(this);

        !core.touch_began_inside
            && !core.remove_from_story_mode
            && selectable
            && core.underwater
            && !already_picked_up
            && !core.eaten_by_piranha
            && !core.sucked_into_drain
            && !core.pain_drain
            && !state.hit_by_shark_laser
            && !state.hit_by_jellyfish
            && !state.hit_by_underwater_power
            && !state.eaten_by_laser_shark
            && !core.skewered
            && !state.bait_master
            && !core.dragging
    };
    if !can_be_picked_up {
        return false;
    }

    world.obj_mut::<SpriteLogic>(this).picked_up_by_whirl_pool = true;
    stop_holding_chum(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    stop_moving(world, this);
    world.obj_mut::<SpriteLogic>(this).picked_up_by_whirl_pool_logic = Some(picked_up_by_whirl_pool);
    hold_still_unselectable(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyPickedUpByWhirlPool"));
    true
}

/// `-[PygmyLogic (Underwater) droppedByWhirlPool:]` @ 0x10011c2bc
pub fn dropped_by_whirl_pool(world: &mut World, this: Entity, _dropped_by_whirl_pool: Entity) {
    let picked_up = world.obj::<SpriteLogic>(this).picked_up_by_whirl_pool;
    if !picked_up {
        return;
    }

    stop_moving(world, this);
    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_whirl_pool = false;
        sprite.picked_up_by_whirl_pool_logic = None;
    }
    set_selectable(world, this, true);
    pg_engine::logic::set_behavior(world, this, Some("PygmyDroppedByWhirlPool"));
}

/// `-[PygmyLogic (Underwater) canBeHitByChainSpear:]` @ 0x10011c35c
pub fn can_be_hit_by_chain_spear(world: &mut World, this: Entity, _chain_spear: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let state = world.obj::<PygmyUnderwaterNopee>(this);
    core.underwater && !core.skewered && !state.eaten_by_laser_shark && !state.hit_by_shark_laser
}

/// `-[PygmyLogic (Underwater) hitByChainSpear:]` @ 0x10011c3bc
pub fn hit_by_chain_spear(world: &mut World, this: Entity, _chain_spear: Entity) {
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.skewered = true;
        core.tilt_state = 0;
    }
    stop_moving(world, this);
    hold_still_unselectable(world, this);
    stop_being_eaten_by_piranha(world, this);
    stop_holding_chum(world, this);
    drop_from_whirl_pool(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    let x_scale = display::object(world, display_object).scale.x;
    let behavior = if x_scale > 0.0 {
        world.obj_mut::<PygmyUnderwaterNopee>(this).skewered_right = true;
        "PygmyHitByChainSpearRight"
    } else {
        display::object_mut(world, display_object).scale.x = 1.0;
        "PygmyHitByChainSpearLeft"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Underwater) hitByChainSpearRetract:]` @ 0x10011c520
pub fn hit_by_chain_spear_retract(world: &mut World, this: Entity, _chain_spear: Entity) {
    let skewered_right = world.obj::<PygmyUnderwaterNopee>(this).skewered_right;
    let behavior = if skewered_right {
        "PygmyHitByChainSpearRightRetract"
    } else {
        "PygmyHitByChainSpearLeftRetract"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Underwater) hitByChainSpearComplete:]` @ 0x10011c558
pub fn hit_by_chain_spear_complete(world: &mut World, this: Entity, _chain_spear: Entity) {
    set_sacrifice_id(world, this, "Skewered");
    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Underwater) canBeHitBySharkLaser]` @ 0x10011c588
pub fn can_be_hit_by_shark_laser(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let state = world.obj::<PygmyUnderwaterNopee>(this);
    core.underwater && !core.skewered && !state.eaten_by_laser_shark && !state.hit_by_shark_laser
}

/// `-[PygmyLogic (Underwater) hitBySharkLaser]` @ 0x10011c5e8
pub fn hit_by_shark_laser(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyUnderwaterNopee>(this).hit_by_shark_laser = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    stop_being_eaten_by_piranha(world, this);
    stop_holding_chum(world, this);
    stop_moving(world, this);
    hold_still_unselectable(world, this);
    set_sacrifice_id(world, this, "ZappedByLaserShark");
    drop_from_whirl_pool(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyHitBySharkLaser"));
}

/// `-[PygmyLogic (Underwater) stopEatenByLaserShark:]` @ 0x10011c6f8
pub fn stop_eaten_by_laser_shark(world: &mut World, this: Entity, _laser_shark: Entity) {
    world.obj_mut::<PygmyUnderwaterNopee>(this).eaten_by_laser_shark = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    set_selectable(world, this, true);
    pg_engine::logic::set_behavior(world, this, Some("PygmyDropUnderwater"));
}

/// `-[PygmyLogic (Underwater) startEatenByLaserShark:]` @ 0x10011c774
pub fn start_eaten_by_laser_shark(world: &mut World, this: Entity, _laser_shark: Entity) {
    world.obj_mut::<PygmyUnderwaterNopee>(this).eaten_by_laser_shark = true;
    set_selectable(world, this, false);
}

/// `-[PygmyLogic (Underwater) eatenByLaserShark:]` @ 0x10011c7ac
pub fn eaten_by_laser_shark(world: &mut World, this: Entity, _laser_shark: Entity) {
    set_sacrifice_id(world, this, "FedToLaserShark");
    pg_engine::logic::set_behavior(world, this, Some("PygmyEatenByLaserShark"));
}

/// `-[PygmyLogic (Underwater) swallowedByLaserShark:]` @ 0x10011c7dc
pub fn swallowed_by_laser_shark(world: &mut World, this: Entity, _laser_shark: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Underwater) canBeHitByJellyfish:]` @ 0x10011c7f4
pub fn can_be_hit_by_jellyfish(world: &mut World, this: Entity, _jellyfish: Entity) -> bool {
    let underwater = world.obj::<PygmyPygmyCoreFrame>(this).underwater;
    let state = world.obj::<PygmyUnderwaterNopee>(this);
    underwater && !state.hit_by_jellyfish && !state.hit_by_underwater_power
}

/// The common part of `suffocatedByJellyfish:` and `stungByJellyfish:`: the jellyfish holds the
/// pygmy still for its attack.
fn attacked_by_jellyfish(
    world: &mut World,
    this: Entity,
    jellyfish: Entity,
    sacrifice_id: &str,
    behavior: &str,
) {
    drop_from_whirl_pool(world, this);
    {
        let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
        state.hit_by_jellyfish = true;
        state.jellyfish_logic = Some(jellyfish);
    }
    stop_moving(world, this);
    hold_still_unselectable(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    set_sacrifice_id(world, this, sacrifice_id);
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Underwater) suffocatedByJellyfish:]` @ 0x10011c840
pub fn suffocated_by_jellyfish(world: &mut World, this: Entity, jellyfish: Entity) {
    attacked_by_jellyfish(world, this, jellyfish, "JellyfishScubaMask", "PygmySuffocatedByJellyfish");
}

/// `-[PygmyLogic (Underwater) stungByJellyfish:]` @ 0x10011c948
pub fn stung_by_jellyfish(world: &mut World, this: Entity, jellyfish: Entity) {
    attacked_by_jellyfish(world, this, jellyfish, "StungByJellyfish", "PygmyStungByJellyfish");
}

/// `-[PygmyLogic (Underwater) jellyfishAttackComplete:]` @ 0x10011ca50
pub fn jellyfish_attack_complete(world: &mut World, this: Entity, _jellyfish: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
    world.obj_mut::<PygmyUnderwaterNopee>(this).jellyfish_logic = None;
}

/// `-[PygmyLogic (Underwater) animJellyfishAttackComplete]` @ 0x10011ca94
pub fn anim_jellyfish_attack_complete(world: &mut World, this: Entity) {
    let jellyfish = world.obj::<PygmyUnderwaterNopee>(this).jellyfish_logic;
    if let Some(jellyfish) = jellyfish {
        jellyfish_logic::on_jellyfish_attack_complete(world, jellyfish, this);
    }
    world.obj_mut::<PygmyUnderwaterNopee>(this).jellyfish_logic = None;
}

/// `-[PygmyLogic (Underwater) onJellyfishSuffocateAttackComplete:::]` @ 0x10011cad8
pub fn seq_on_jellyfish_suffocate_attack_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let jellyfish = world.obj::<PygmyUnderwaterNopee>(this).jellyfish_logic;
    if let Some(jellyfish) = jellyfish {
        jellyfish_logic::on_jellyfish_suffocate_attack_complete(world, jellyfish, this);
    }
    world.obj_mut::<PygmyUnderwaterNopee>(this).jellyfish_logic = None;
    ItemResult::Done
}

/// What `checkJellyfishCollision` does to a dropped pygmy that a jellyfish catches: the touch
/// is released and the pygmy held still for the attack.
fn caught_by_jellyfish(world: &mut World, this: Entity, sacrifice_id: &str, behavior: &str) {
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.touch_began_inside = false;
        core.active_touch = None;
    }
    world.obj_mut::<PygmyUnderwaterNopee>(this).hit_by_jellyfish = true;
    stop_moving(world, this);
    hold_still_unselectable(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    set_sacrifice_id(world, this, sacrifice_id);
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// Distance in the xy plane between `(x, y)` and `point`.
fn distance_xy(x: f32, y: f32, point_x: f32, point_y: f32) -> f32 {
    let dx = x - point_x;
    let dy = y - point_y;
    (dx * dx + dy * dy).sqrt()
}

/// `-[PygmyLogic (Underwater) checkJellyfishCollision]` @ 0x10011cb20
///
/// A pygmy dropped onto a jellyfish: near its body it is eaten, near the point 60 below it the
/// jellyfish drinks it, and near the centre of a circle of jellyfish it is shocked.
pub fn check_jellyfish_collision(world: &mut World, this: Entity) -> bool {
    let Some(listener) = listener_of(world, this) else {
        return false;
    };
    let jellyfish_enabled = crate::controller::is_jellyfish_enabled(world, listener);
    if !jellyfish_enabled {
        return false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let (pos, origin_y_pos) = {
        let object = display::object(world, display_object);
        (object.pos, object.origin_pos.y)
    };
    let probe_x = pos.x;
    let probe_y = pos.y + (origin_y_pos + 20.0);

    let jellyfish_array = crate::controller::jellyfish_logic_array(world, listener, this);
    for jellyfish in jellyfish_array {
        let circle_leader = jellyfish_logic::is_circle_jerk_leader(world, jellyfish);
        if circle_leader {
            let circle_jerk_pos = jellyfish_logic::circle_jerk_pos(world, jellyfish);
            let circle_distance = distance_xy(probe_x, probe_y, circle_jerk_pos.x, circle_jerk_pos.y);
            if circle_distance < JELLYFISH_REACH {
                caught_by_jellyfish(world, this, "ElectrocuteByJellyfish", "PygmyShockedByJellyfish");
                jellyfish_logic::shock_pygmy_logic(world, jellyfish, this);
                return true;
            }
            continue;
        }

        let can_interact = jellyfish_logic::can_interact_with_pygmy(world, jellyfish);
        if !can_interact {
            continue;
        }

        let jellyfish_display_object = pg_engine::logic::display_object(world, jellyfish);
        let jellyfish_pos = display::object(world, jellyfish_display_object).pos;

        let body_distance = distance_xy(probe_x, probe_y, jellyfish_pos.x, jellyfish_pos.y);
        if body_distance < JELLYFISH_REACH {
            world.obj_mut::<PygmyUnderwaterNopee>(this).jellyfish_logic = Some(jellyfish);
            caught_by_jellyfish(world, this, "EatenByJellyfish", "PygmyHitByJellyfish");
            jellyfish_logic::eat_pygmy_logic(world, jellyfish, this);
            return true;
        }

        let mouth_y_pos = jellyfish_pos.y + -60.0;
        let mouth_distance = distance_xy(probe_x, probe_y, jellyfish_pos.x, mouth_y_pos);
        if mouth_distance < JELLYFISH_REACH {
            world.obj_mut::<PygmyUnderwaterNopee>(this).jellyfish_logic = Some(jellyfish);
            caught_by_jellyfish(world, this, "DrankByJellyfish", "PygmyDrankByJellyfish");
            jellyfish_logic::drink_pygmy_logic(world, jellyfish, this);
            return true;
        }
    }
    false
}

/// `-[PygmyLogic (Underwater) canBeTouchedByUnderwaterPower]` @ 0x10011d088
pub fn can_be_touched_by_underwater_power(world: &mut World, this: Entity) -> bool {
    let underwater = world.obj::<PygmyPygmyCoreFrame>(this).underwater;
    let state = world.obj::<PygmyUnderwaterNopee>(this);
    underwater && !state.hit_by_jellyfish && !state.hit_by_underwater_power
}

/// `-[PygmyLogic (Underwater) canBeTouchedByUnderwaterPower:]` @ 0x10011d0d4
///
/// Despite the name, the hit itself: power types 1 to 5 are cold, heat, oil, acid and air;
/// any other type only holds the pygmy still.
pub fn can_be_touched_by_underwater_power_with_arg(world: &mut World, this: Entity, power_type: i32) {
    let whirl_pool = world.obj::<SpriteLogic>(this).picked_up_by_whirl_pool_logic;
    if let Some(whirl_pool) = whirl_pool {
        whirl_pool_logic::drop_sprite_logic(world, whirl_pool, this);
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_whirl_pool_logic = None;
        sprite.picked_up_by_whirl_pool = false;
    }

    world.obj_mut::<PygmyUnderwaterNopee>(this).hit_by_underwater_power = true;
    stop_moving(world, this);
    hold_still_unselectable(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let (sacrifice_id, behavior) = match power_type {
        1 => ("HitByUnderwaterPowerCold", "PygmyHitByUnderwaterPowerCold"),
        2 => ("HitByUnderwaterPowerHeat", "PygmyHitByUnderwaterPowerHeat"),
        3 => ("HitByUnderwaterPowerOil", "PygmyHitByUnderwaterPowerOil"),
        4 => ("HitByUnderwaterPowerAcid", "PygmyHitByUnderwaterPowerAcid"),
        5 => ("HitByUnderwaterPowerAir", "PygmyHitByUnderwaterPowerAir"),
        _ => return,
    };
    set_sacrifice_id(world, this, sacrifice_id);
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Underwater) dropChum:]` @ 0x10011d2e0
pub fn drop_chum(world: &mut World, this: Entity, _chum: Entity) {
    {
        let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
        if state.holding_chum_logic.is_some() {
            state.holding_chum_logic = None;
            state.holding_chum = false;
        }
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    pg_engine::logic::set_behavior(world, this, Some("PygmyUnderwaterPanicIdle"));
}

/// `-[PygmyLogic (Underwater) canHoldChum:]` @ 0x10011d330
pub fn can_hold_chum(world: &mut World, this: Entity, _chum: Entity) -> bool {
    let picked_up_by_whirl_pool = world.obj::<SpriteLogic>(this).picked_up_by_whirl_pool;
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let state = world.obj::<PygmyUnderwaterNopee>(this);

    core.underwater
        && !core.remove_from_story_mode
        && !core.skewered
        && !state.holding_chum
        && !state.bait_master
        && !core.eaten_by_piranha
        && !core.sucked_into_drain
        && !core.pain_drain
        && !picked_up_by_whirl_pool
        && !state.hit_by_shark_laser
        && !state.eaten_by_laser_shark
}

/// `-[PygmyLogic (Underwater) holdChum:]` @ 0x10011d41c
pub fn hold_chum(world: &mut World, this: Entity, chum: Entity) {
    {
        let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
        state.holding_chum = true;
        state.holding_chum_logic = Some(chum);
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    stop_moving(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    pg_engine::logic::set_behavior(world, this, Some("PygmyHoldChum"));
}

/// `-[PygmyLogic (Underwater) startBaitMaster]` @ 0x10011d4bc
///
/// Bait master is remembered by every underwater pygmy that is not skewered; the swimming ones
/// hold still at once.
pub fn start_bait_master(world: &mut World, this: Entity) {
    let (underwater, skewered) = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        (core.underwater, core.skewered)
    };
    if !underwater || skewered {
        return;
    }

    world.obj_mut::<PygmyUnderwaterNopee>(this).bait_master = true;
    let active_underwater = world.obj::<PygmyUnderwaterNopee>(this).active_underwater;
    if !active_underwater {
        return;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    stop_moving(world, this);
    drop_from_whirl_pool(world, this);

    let dragging = world.obj::<PygmyPygmyCoreFrame>(this).dragging;
    if dragging {
        {
            let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
            core.dragging = false;
            core.apply_spring = false;
        }

        let display_object = pg_engine::logic::display_object(world, this);
        display::object_mut(world, display_object).rot.z = 0.0;

        let listener = listener_of(world, this);
        if let Some(listener) = listener {
            crate::controller::on_pygmy_drag_complete(world, listener, this);
        }
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyStartBaitMaster"));
}

/// `-[PygmyLogic (Underwater) stopBaitMaster]` @ 0x10011d60c
pub fn stop_bait_master(world: &mut World, this: Entity) {
    let eaten_by_piranha = world.obj::<PygmyPygmyCoreFrame>(this).eaten_by_piranha;
    let (bait_master, active_underwater) = {
        let state = world.obj::<PygmyUnderwaterNopee>(this);
        (state.bait_master, state.active_underwater)
    };
    if bait_master && !eaten_by_piranha && active_underwater {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        pg_engine::logic::set_behavior(world, this, Some("PygmyStopBaitMaster"));
    }

    stop_being_eaten_by_piranha(world, this);
    world.obj_mut::<PygmyUnderwaterNopee>(this).bait_master = false;
}

/// `-[PygmyLogic (Underwater) eatingPiranhaLogic]` @ 0x10011d6a8
pub fn eating_piranha_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyUnderwaterNopee>(this).eating_piranha_logic
}

/// `-[PygmyLogic (Underwater) canBeEatenByPiranha:]` @ 0x10011d6bc
pub fn can_be_eaten_by_piranha(world: &mut World, this: Entity, _piranha: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let state = world.obj::<PygmyUnderwaterNopee>(this);

    !core.dragging
        && !core.remove_from_story_mode
        && !core.touch_began_inside
        && state.bait_master
        && !core.eaten_by_piranha
        && !core.sucked_into_drain
        && !core.pain_drain
        && state.active_underwater
}

/// `-[PygmyLogic (Underwater) eatenByPiranha:]` @ 0x10011d76c
pub fn eaten_by_piranha(world: &mut World, this: Entity, piranha: Entity) {
    let eaten_by_piranha = world.obj::<PygmyPygmyCoreFrame>(this).eaten_by_piranha;
    let bait_master = world.obj::<PygmyUnderwaterNopee>(this).bait_master;
    if !bait_master || eaten_by_piranha {
        return;
    }

    world.obj_mut::<PygmyUnderwaterNopee>(this).eating_piranha_logic = Some(piranha);
    set_selectable(world, this, false);
    pg_engine::logic::set_behavior(world, this, Some("PygmyPiranhaAttack"));
}

/// `-[PygmyLogic (Underwater) eatenByPiranhaComplete:]` @ 0x10011d808
pub fn eaten_by_piranha_complete(world: &mut World, this: Entity, _piranha: Entity) {
    let (bait_master, eating_piranha_logic) = {
        let state = world.obj::<PygmyUnderwaterNopee>(this);
        (state.bait_master, state.eating_piranha_logic)
    };
    if !bait_master || eating_piranha_logic.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).eaten_by_piranha = true;
    world.obj_mut::<PygmyUnderwaterNopee>(this).eating_piranha_logic = None;
    set_sacrifice_id(world, this, "FedToPiranha");
    pg_engine::logic::set_behavior(world, this, Some("PygmyPiranhaAttackComplete"));
}

/// `-[PygmyLogic (Underwater) stopEatenByPiranha:]` @ 0x10011d87c
pub fn stop_eaten_by_piranha(world: &mut World, this: Entity, _piranha: Entity) {
    let (bait_master, eating_piranha_logic) = {
        let state = world.obj::<PygmyUnderwaterNopee>(this);
        (state.bait_master, state.eating_piranha_logic)
    };
    if !bait_master || eating_piranha_logic.is_none() {
        return;
    }

    world.obj_mut::<PygmyUnderwaterNopee>(this).eating_piranha_logic = None;
    set_selectable(world, this, true);
    pg_engine::logic::set_behavior(world, this, Some("PygmyStopPiranhaAttack"));
}

/// `-[PygmyLogic (Underwater) addedUnderwater]` @ 0x10011d90c
///
/// A pygmy created while the island is underwater.
pub fn added_underwater(world: &mut World, this: Entity) {
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.tilt_state = 0;
        core.falling = false;
        core.underwater = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyUnderwaterNopee>(this).added_underwater = true;
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_side_collision = false;
        core.enable_island_floor_collision = false;
        core.invalid_sacrifice = false;
    }
    stop_moving(world, this);
    hold_still_unselectable(world, this);

    let marker = underwater_pygmy_display_marker(world, this);
    insert_after_marker(world, this, marker);
    update_label_layer(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyAddedUnderwater"));
}

/// `-[PygmyLogic (Underwater) startUnderwater]` @ 0x10011da5c
///
/// An existing pygmy when the island goes underwater.
pub fn start_underwater(world: &mut World, this: Entity) {
    if !can_start_underwater(world, this) {
        return;
    }

    wait_underwater(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;

    let marker = underwater_pygmy_display_marker(world, this);
    insert_after_marker(world, this, marker);
    update_label_layer(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyStartUnderwater"));
}

/// `-[PygmyLogic (Underwater) waitUnderwater]` @ 0x10011dfb8
///
/// Takes the pygmy out of whatever it was doing and hides it at a random spot underwater.
pub fn wait_underwater(world: &mut World, this: Entity) {
    let (tom_in_closet, held_by_xenu, died_from_thetan_brawl) = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let tom_in_closet = core.persona_personality == 2 && core.in_closet_temple_closet_door;
        (tom_in_closet, core.held_by_xenu, core.died_from_thetan_brawl)
    };
    if tom_in_closet {
        return;
    }
    if held_by_xenu {
        stop_being_held_by_xenu(world, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
        return;
    }
    if died_from_thetan_brawl {
        stop_thetan_brawl(world, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
        return;
    }
    if !can_wait_underwater(world, this) {
        return;
    }

    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    stop_being_trapped_in_zombie_grave(world, this);
    stop_being_trapped_in_idol_pedestal(world, this);
    stop_being_inside_temple_door(world, this);
    stop_being_inside_apocalypse_temple_door(world, this);
    stop_being_on_top_apocalypse_time_temple_platform(world, this);
    stop_being_in_apocalypse_temple_fountain(world, this);
    stop_thetan_brawl(world, this);
    stop_being_on_closet_temple_thetan_meter(world, this);
    stop_being_on_closet_temple_couch(world, this);
    stop_being_in_closet_temple_closet_door(world, this);
    stop_being_held_in_toms_arms(world, this);
    stop_being_possessed_by_thetan(world, this);
    stop_holding_baby_pygmy(world, this);
    stop_being_thrown_by_locusts(world, this);
    stop_playing_catch_with_monkey(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);

    set_selectable(world, this, false);

    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        hurricane_logic::drop_sprite_logic(world, hurricane, this);
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = None;
        sprite.picked_up_by_hurricane = false;
    }

    end_drag(world, this);

    let doodle_jumping = world.obj::<PygmyPygmyCoreFrame>(this).doodle_jumping;
    if doodle_jumping {
        world.obj_mut::<PygmyMinigames>(this).doodle_on_ice = false;
        {
            let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
            core.doodle_jumping = false;
            core.doodle_knocked_down = false;
        }

        let listener = listener_of(world, this);
        if let Some(listener) = listener {
            crate::controller::on_pygmy_stop_doodle_jump(world, listener, this);
        }
    }

    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.tilt_state = 0;
        core.falling = false;
        core.underwater = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_side_collision = false;
        core.enable_island_floor_collision = false;
    }

    let display_object = pg_engine::logic::display_object(world, this);

    let x_random = world.resource_mut::<Random>().arc4random_mod(32001);
    display::object_mut(world, display_object).pos.x = (x_random + 8000) as f32 / 100.0;
    let y_random = world.resource_mut::<Random>().arc4random_mod(12001);
    display::object_mut(world, display_object).pos.y = (y_random + 16000) as f32 / 100.0;

    stop_moving(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.z = SPRITE_Z;
        object.rot.z = 0.0;
        object.set_alpha(0.0);
    }
    pg_engine::logic::set_behavior(world, this, Some("PygmyWaitUnderwater"));
}

/// `-[PygmyLogic (Underwater) stopUnderwater]` @ 0x10011e9c4
///
/// Back on land: an underwater pygmy returns to the island, unless a power or the piranhas
/// already got it.
pub fn stop_underwater(world: &mut World, this: Entity) {
    let (underwater, eaten_by_piranha) = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        (core.underwater, core.eaten_by_piranha)
    };
    if !underwater {
        return;
    }

    let hit_by_underwater_power = world.obj::<PygmyUnderwaterNopee>(this).hit_by_underwater_power;
    if hit_by_underwater_power || eaten_by_piranha {
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
        return;
    }

    end_drag(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).underwater = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyUnderwaterNopee>(this).active_underwater = false;
    stop_holding_chum(world, this);
    stop_being_eaten_by_piranha(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.is_selectable = true;
        object.set_alpha(1.0);
    }
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_side_collision = true;
        core.enable_island_floor_collision = true;
    }

    let listener = listener_of(world, this);
    let marker = match listener {
        Some(listener) => crate::controller::pygmy_display_marker(world, listener),
        None => None,
    };
    insert_after_marker(world, this, marker);
    update_label_layer(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyStopUnderwater"));
}

/// `-[PygmyLogic (Underwater) suckedIntoDrain:]` @ 0x10011ebd8
pub fn sucked_into_drain(world: &mut World, this: Entity, _tile_drain: Entity) -> bool {
    if !can_be_sucked_into_drain(world, this) {
        return false;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sucked_into_drain = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    set_selectable(world, this, false);
    stop_moving(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.rot.z = 0.0;
        object.set_alpha(1.0);
    }
    pg_engine::logic::set_behavior(world, this, Some("PygmySuckedIntoDrain"));
    true
}

/// `-[PygmyLogic (Underwater) escapeFromDrain:]` @ 0x10011f138
pub fn escape_from_drain(world: &mut World, this: Entity, _tile_drain: Entity) {
    let sucked_into_drain = world.obj::<PygmyPygmyCoreFrame>(this).sucked_into_drain;
    if !sucked_into_drain {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sucked_into_drain = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    set_selectable(world, this, true);
    stop_moving(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.rot.z = 0.0;
        object.set_alpha(1.0);
    }
    pg_engine::logic::set_behavior(world, this, Some("PygmyEscapeFromDrain"));
}

/// `-[PygmyLogic (Underwater) startPainDrain]` @ 0x10011f208
pub fn start_pain_drain(world: &mut World, this: Entity) {
    if !can_start_pain_drain(world, this) {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).pain_drain = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    set_selectable(world, this, false);
    stop_moving(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = 240.0;
        object.pos.y = 250.0;
        object.pos.z = SPRITE_Z;
        object.rot.z = 0.0;
        object.set_alpha(1.0);
    }
    pg_engine::logic::set_behavior(world, this, Some("PygmyPainDrainStart"));
}

/// `-[PygmyLogic (Underwater) dieFromPainDrain]` @ 0x10011f758
pub fn die_from_pain_drain(world: &mut World, this: Entity) {
    set_sacrifice_id(world, this, "PainDrainSpikes");

    let marker = underwater_pygmy_display_marker(world, this);
    insert_after_marker(world, this, marker);
    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Underwater) checkForLaserSharkEnabled:::]` @ 0x10011f7e4
pub fn seq_check_for_laser_shark_enabled(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let laser_shark_enabled = match listener {
        Some(listener) => crate::controller::is_laser_shark_enabled(world, listener),
        None => false,
    };
    if laser_shark_enabled {
        pg_engine::logic::set_behavior(world, this, Some("PygmyUnderwaterPanicIdle"));
    }
    ItemResult::Done
}

/// `-[PygmyLogic (Underwater) checkForLaserSharkDisabled:::]` @ 0x10011f844
pub fn seq_check_for_laser_shark_disabled(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let laser_shark_enabled = match listener {
        Some(listener) => crate::controller::is_laser_shark_enabled(world, listener),
        None => false,
    };
    if !laser_shark_enabled {
        pg_engine::logic::set_behavior(world, this, Some("PygmyUnderwaterIdle"));
    }
    ItemResult::Done
}

/// `-[PygmyLogic (Underwater) addUnderwaterComplete:::]` @ 0x10011f8a4
pub fn seq_add_underwater_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
        state.added_underwater = false;
        state.active_underwater = true;
    }
    set_selectable(world, this, true);

    let bait_master = world.obj::<PygmyUnderwaterNopee>(this).bait_master;
    if bait_master {
        start_bait_master(world, this);
    }
    ItemResult::Done
}

/// `swim:::` and `panicSwim:::`, which differ only in their speeds: on the first call a random
/// duration in `[minTime, maxTime]`, an upward speed and a random direction (inward at the
/// island's ends); the item completes, stopping, when the time is up.
fn swim_for_a_while(
    world: &mut World,
    this: Entity,
    call: &SequenceCall,
    y_vel: f32,
    x_speed: f32,
) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);

    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;

        let coin = world.resource_mut::<Random>().arc4random();
        let listener = listener_of(world, this);
        let (island_min_x_pos, island_max_x_pos) = match listener {
            Some(listener) => {
                let island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
                let island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
                (island_min_x_pos, island_max_x_pos)
            }
            None => (0.0, 0.0),
        };

        pg_engine::logic::logic_mut(world, this).pos_vel.y = y_vel;

        let x_pos = display::object(world, display_object).pos.x;
        let goes_left = x_pos == island_max_x_pos || (coin & 1 == 0 && x_pos != island_min_x_pos);
        let (x_scale, x_vel) = if goes_left { (-1.0, -x_speed) } else { (1.0, x_speed) };
        display::object_mut(world, display_object).scale.x = x_scale;
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    let result = if stall_time <= 0.0 {
        stop_moving(world, this);
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    result
}

/// `-[PygmyLogic (Underwater) swim:::]` @ 0x10011f944
pub fn seq_swim(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    swim_for_a_while(world, this, call, 50.0, 10.0)
}

/// `-[PygmyLogic (Underwater) panicSwim:::]` @ 0x10011fb3c
pub fn seq_panic_swim(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    swim_for_a_while(world, this, call, 30.0, 30.0)
}

/// `-[PygmyLogic (Underwater) painDrainCanTurn:::]` @ 0x10011fd34
pub fn seq_pain_drain_can_turn(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyUnderwaterNopee>(this).pain_drain_can_turn = true;
    ItemResult::Done
}

/// `-[PygmyLogic (Underwater) fallIntoDrain:::]` @ 0x10011fd50
///
/// Moves to 16 above the drain in one second, then tells the drain. Only the y axis ends the
/// move, so a pygmy already level with that point waits forever.
pub fn seq_fall_into_drain(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let Some(tile_drain) = tile_drain_logic_of(world, this) else {
        // The original reads `[[nil displayObject] pos]` here and would crash.
        return ItemResult::Pending;
    };
    let drain_display_object = pg_engine::logic::display_object(world, tile_drain);
    let drain_pos = display::object(world, drain_display_object).pos;
    let target_y_pos = drain_pos.y + 16.0;

    if call.first_time {
        let pos = display::object(world, display_object).pos;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = (drain_pos.x + 0.0) - pos.x;
        logic.pos_vel.y = target_y_pos - pos.y;
    }

    let y_vel = pg_engine::logic::logic(world, this).pos_vel.y;
    let y_pos = display::object(world, display_object).pos.y;
    let passed_going_down = y_vel < 0.0 && y_pos < target_y_pos;
    let passed_going_up = y_vel > 0.0 && y_pos > target_y_pos;
    if !passed_going_down && !passed_going_up {
        return ItemResult::Pending;
    }

    stop_moving(world, this);
    tile_drain_logic::pygmy_fall_into_drain_complete(world, tile_drain, this);
    ItemResult::Done
}

/// `-[PygmyLogic (NoPee) canBeHitByNoPeeObject]` @ 0x10018a5a8
pub fn can_be_hit_by_no_pee_object(world: &mut World, this: Entity) -> bool {
    let state = world.obj::<PygmyUnderwaterNopee>(this);
    !state.no_pee_killed && state.no_pee_intro_complete
}

/// `-[PygmyLogic (NoPee) collideAndDie]` @ 0x10018a5e0
pub fn collide_and_die(world: &mut World, this: Entity) {
    let killed = world.obj::<PygmyUnderwaterNopee>(this).no_pee_killed;
    if killed {
        return;
    }

    world.obj_mut::<PygmyUnderwaterNopee>(this).no_pee_killed = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
        state.no_pee_pos_vel.x = 0.0;
        state.no_pee_pos_vel.y = 0.0;
    }

    let no_pee = no_pee_logic_of(world, this);
    if let Some(no_pee) = no_pee {
        no_pee_logic::on_pygmy_killed(world, no_pee);
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.scale.y = 0.5;
}

/// `-[PygmyLogic (NoPee) animNoPeeEmbedDie]` @ 0x10018a6bc
pub fn anim_no_pee_embed_die(world: &mut World, this: Entity) {
    let embed = world.obj::<PygmyUnderwaterNopee>(this).no_pee_embed_logic;
    if let Some(embed) = embed {
        pg_engine::logic::set_behavior(world, embed, Some("NoPeeEmbedDie"));
    }
}

/// `-[PygmyLogic (NoPee) animNoPeeFloatUpward]` @ 0x10018a6e4
pub fn anim_no_pee_float_upward(world: &mut World, this: Entity) {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
}

/// `-[PygmyLogic (NoPee) animNoPeeSinkDownward]` @ 0x10018a6f8
pub fn anim_no_pee_sink_downward(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyUnderwaterNopee>(this).no_pee_turn_to_stone = true;
}

/// `-[PygmyLogic (NoPee) noPeeSwimmingLeft]` @ 0x10018a710
pub fn no_pee_swimming_left(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyUnderwaterNopee>(this).no_pee_swimming_left
}

/// `-[PygmyLogic (NoPee) hitByNoPeeSeaBlob:]` @ 0x10018a724
pub fn hit_by_no_pee_sea_blob(world: &mut World, this: Entity, _sea_blob: Entity) {
    collide_and_die(world, this);
    set_sacrifice_id(world, this, "ImpregnatedByBlob");
    pg_engine::logic::set_behavior(world, this, Some("PygmyNoPeeHitSeaBlob"));
}

/// `-[PygmyLogic (NoPee) hitByNoPeeMine:]` @ 0x10018a780
pub fn hit_by_no_pee_mine(world: &mut World, this: Entity, _mine: Entity) {
    collide_and_die(world, this);
    set_sacrifice_id(world, this, "Stoned");
    pg_engine::logic::set_behavior(world, this, Some("PygmyNoPeeHitMine"));
}

/// `-[PygmyLogic (NoPee) hitByNoPeeClam:]` @ 0x10018a7dc
///
/// The pygmy snaps into the clam.
pub fn hit_by_no_pee_clam(world: &mut World, this: Entity, clam: Entity) {
    let clam_display_object = pg_engine::logic::display_object(world, clam);
    let clam_pos = display::object(world, clam_display_object).pos;
    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).pos = clam_pos;

    collide_and_die(world, this);
    set_sacrifice_id(world, this, "EatenByClam");
    pg_engine::logic::set_behavior(world, this, Some("PygmyNoPeeHitClam"));
}

/// `-[PygmyLogic (NoPee) noPeeRunOutOfBreath]` @ 0x10018a894
pub fn no_pee_run_out_of_breath(world: &mut World, this: Entity) {
    let killed = world.obj::<PygmyUnderwaterNopee>(this).no_pee_killed;
    if killed {
        return;
    }

    collide_and_die(world, this);
    set_sacrifice_id(world, this, "RunOutOfBreath");
    pg_engine::logic::set_behavior(world, this, Some("PygmyNoPeeHitPillar"));
}

/// `-[PygmyLogic (NoPee) checkNoPeeCollision:]` @ 0x10018a914
///
/// Drowns the pygmy above `y = 1000` and keeps it from sinking below 0. The original receives
/// `&mLastPos` and ignores it.
pub fn check_no_pee_collision(world: &mut World, this: Entity, _last_pos: Vec4) {
    let (started, killed) = {
        let state = world.obj::<PygmyUnderwaterNopee>(this);
        (state.no_pee_started, state.no_pee_killed)
    };
    if !started || killed {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    if y_pos > 1000.0 {
        collide_and_die(world, this);
        set_sacrifice_id(world, this, "WaterWingsDrowned");
        pg_engine::logic::set_behavior(world, this, Some("PygmyNoPeeHitPillar"));
        return;
    }
    if y_pos < 0.0 {
        display::object_mut(world, display_object).pos.y = 0.0;
    }
}

/// `-[PygmyLogic (NoPee) startNoPee]` @ 0x10018a9e8
pub fn start_no_pee(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyTouchDrag>(this).no_pee = true;
    {
        let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
        state.no_pee_started = false;
        state.no_pee_intro_complete = false;
        state.no_pee_killed = false;
        state.no_pee_turn_to_stone = false;
        state.no_pee_thrust_on = false;
        state.no_pee_change_direction = false;
        state.no_pee_changing_direction_downward = false;
        state.no_pee_swimming_left = false;
        state.no_pee_swim_x_vel = 100.0;
        state.no_pee_swim_target_x_vel = 100.0;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_side_collision = false;
        core.enable_island_floor_collision = false;
    }
    set_selectable(world, this, true);
    {
        let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
        state.no_pee_thrust_vel = 0.0;
        state.no_pee_turn_around_time = 0.0;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = 250.0;
        object.pos.y = 12.0;
        object.scale.x = 0.5;
        object.scale.y = 0.5;
        object.rot.z = 0.0;
    }
    {
        let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
        state.no_pee_pos_vel.x = 0.0;
        state.no_pee_pos_vel.y = 0.0;
    }

    let embed = pg_engine::library::logic(world, crate::logic_table::NO_PEE_EMBED);
    world.obj_mut::<PygmyUnderwaterNopee>(this).no_pee_embed_logic = Some(embed);
    pg_engine::logic::set_behavior(world, embed, Some("NoPeeEmbedBubbles"));

    display::object_mut(world, display_object).set_alpha(1.0);
    pg_engine::logic::set_behavior(world, this, Some("PygmyNoPeeStart"));
    world.obj_mut::<PygmyPygmyCoreFrame>(this).drag_label_y_offset = -20.0;
}

/// `-[PygmyLogic (NoPee) onNoPeeIntroComplete]` @ 0x10018ac38
pub fn on_no_pee_intro_complete(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyUnderwaterNopee>(this).no_pee_intro_complete = true;

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.scale.x = 0.5;
        object.scale.y = 0.5;
    }
    world.obj_mut::<PygmyUnderwaterNopee>(this).no_pee_pos_vel.x = 100.0;

    let animation = pg_engine::logic::animation(world, this);
    let embed = world.obj::<PygmyUnderwaterNopee>(this).no_pee_embed_logic;
    let embed_display_object = match embed {
        Some(embed) => Some(pg_engine::logic::display_object(world, embed)),
        None => None,
    };
    pg_engine::animation::set_tweenable(world, animation, embed_display_object, NO_PEE_EMBED_CHANNEL);

    if let Some(embed) = embed {
        pg_engine::logic::set_behavior(world, embed, Some("NoPeeEmbedSwimBubbles"));
    }
    pg_engine::logic::set_behavior(world, this, Some("PygmyNoPeeGlide"));
}

/// `-[PygmyLogic (NoPee) handleNoPeeMovement:]` @ 0x10018ad28
///
/// The pygmy's frame while `mbNoPee` (it replaces the normal `onGameFrame:` physics). Before
/// the first touch and after death the body only drifts; while swimming, a thrust along the
/// heading, a constant forward swim speed and a righting torque steer it (docs/re/pygmy.md §15).
pub fn handle_no_pee_movement(world: &mut World, this: Entity, dt: f32) {
    let display_object = pg_engine::logic::display_object(world, this);
    let no_pee = world.obj::<PygmyPygmyTouchDrag>(this).no_pee;
    let (started, killed) = {
        let state = world.obj::<PygmyUnderwaterNopee>(this);
        (state.no_pee_started, state.no_pee_killed)
    };

    if !no_pee || !started || killed {
        let ignore_gravity = world.obj::<SpriteLogic>(this).ignore_gravity;
        let drift_vel = {
            let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
            if !ignore_gravity {
                state.no_pee_pos_vel.y += dt * 50.0;
            }
            if state.no_pee_turn_to_stone {
                state.no_pee_pos_vel.y += dt * -50.0;
            }
            state.no_pee_pos_vel
        };
        pg_engine::logic::logic_mut(world, this).pos_vel = drift_vel;
        {
            let mut object = display::object_mut(world, display_object);
            object.pos.x += drift_vel.x * dt;
            object.pos.y += drift_vel.y * dt;
            object.pos.z += drift_vel.z * dt;
        }
        move_name_label(world, this);
        return;
    }

    let rot_z = display::object(world, display_object).rot.z;
    let (swimming_left, changing_direction_downward, thrust_on) = {
        let state = world.obj::<PygmyUnderwaterNopee>(this);
        let swimming_left = state.no_pee_swimming_left;
        let changing_direction_downward = state.no_pee_changing_direction_downward;
        (swimming_left, changing_direction_downward, state.no_pee_thrust_on)
    };

    let direction: f32 = if swimming_left { -1.0 } else { 1.0 };
    let tilt = direction * 20.0;
    let heading = (rot_z + tilt + -90.0 + 360.0 + 180.0) % 360.0 + -180.0;

    // While a roll that started pointing down lasts, the pygmy turns the other way round.
    let turn_clockwise = if changing_direction_downward { swimming_left } else { !swimming_left };

    let mut thrust_force = 0.0;
    let mut turn_force = 0.0;
    if thrust_on || changing_direction_downward {
        turn_force = if turn_clockwise { -600.0 } else { 600.0 };

        let rot_vel_z = pg_engine::logic::logic(world, this).rot_vel.z;
        let spinning_backwards = if turn_clockwise { rot_vel_z > 0.0 } else { rot_vel_z < 0.0 };
        if spinning_backwards {
            pg_engine::logic::logic_mut(world, this).rot_vel.z = 0.0;
        }
        thrust_force = 500.0;
    }

    let heading_in_range = if turn_clockwise {
        heading <= 0.0 && -160.0 < heading
    } else {
        0.0 <= heading && heading < 160.0
    };
    let righting_force;
    if heading_in_range {
        righting_force = direction * 300.0;
    } else {
        righting_force = direction * -300.0;
        if !changing_direction_downward {
            turn_force = 0.0;
        }
    }

    let (thrust_vel, no_pee_pos_vel, swim_x_vel) = {
        let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);

        let old_thrust_vel = state.no_pee_thrust_vel;
        let accelerated = old_thrust_vel + (thrust_force + old_thrust_vel * -0.9) * dt;
        let thrust_vel = if accelerated > 200.0 { 200.0 } else { accelerated };
        state.no_pee_thrust_vel = thrust_vel;

        let mut no_pee_pos_vel = state.no_pee_pos_vel;
        no_pee_pos_vel.x = no_pee_pos_vel.x - no_pee_pos_vel.x * 0.5 * dt;
        no_pee_pos_vel.y = no_pee_pos_vel.y + (50.0 - no_pee_pos_vel.y * 0.5) * dt;
        state.no_pee_pos_vel = no_pee_pos_vel;

        let mut swim_x_vel = state.no_pee_swim_x_vel;
        let swim_target_x_vel = state.no_pee_swim_target_x_vel;
        if swim_x_vel != swim_target_x_vel {
            let turn_around_time = state.no_pee_turn_around_time + dt;
            state.no_pee_turn_around_time = turn_around_time;

            let mut blend = turn_around_time * 0.5;
            if blend >= 1.0 {
                blend = 1.0;
            }
            swim_x_vel = swim_x_vel + blend * (swim_target_x_vel - swim_x_vel);
            state.no_pee_swim_x_vel = swim_x_vel;
        }

        (thrust_vel, no_pee_pos_vel, swim_x_vel)
    };

    let thrust_angle = (heading - tilt + 90.0 + 360.0) % 360.0;
    let thrust_radians = pg_engine::math::deg_to_rad(thrust_angle);
    let (thrust_sin, thrust_cos) = thrust_radians.sin_cos();

    let mut pos_vel = no_pee_pos_vel;
    pos_vel.x = thrust_vel * thrust_cos + pos_vel.x;
    pos_vel.y = thrust_vel * (thrust_sin + -0.6) + pos_vel.y;
    pos_vel.z = pos_vel.z + 0.0;
    pos_vel.x = swim_x_vel + pos_vel.x;
    pg_engine::logic::logic_mut(world, this).pos_vel = pos_vel;

    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x += pos_vel.x * dt;
        object.pos.y += pos_vel.y * dt;
        object.pos.z += pos_vel.z * dt;
    }

    let rot_vel_z = {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        let old_rot_vel_z = logic.rot_vel.z;
        let rot_vel_z = old_rot_vel_z + (turn_force + righting_force + old_rot_vel_z * -0.9) * dt;
        logic.rot_vel.z = rot_vel_z;
        rot_vel_z
    };

    let new_rot_z = heading + rot_vel_z * dt - tilt + 90.0;
    display::object_mut(world, display_object).rot.z = new_rot_z;

    if changing_direction_downward {
        let pointing_up = 0.0 < new_rot_z && new_rot_z < 180.0;
        let pointing_up_one_turn_back = -360.0 < new_rot_z && new_rot_z < -180.0;
        if pointing_up || pointing_up_one_turn_back {
            world.obj_mut::<PygmyUnderwaterNopee>(this).no_pee_changing_direction_downward = false;
        }
    }

    let last_pos = world.obj::<PygmyPygmyCoreFrame>(this).last_pos;
    check_no_pee_collision(world, this, last_pos);
    move_name_label(world, this);

    let pos = display::object(world, display_object).pos;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).last_pos = pos;
}

/// Starts a No Pee roll toward the new swimming direction (`touchBeganNoPee:allTouches:`).
fn start_no_pee_roll(
    world: &mut World,
    this: Entity,
    swimming_left: bool,
    pointing_down: bool,
    swim_target_x_vel: f32,
) {
    {
        let mut state = world.obj_mut::<PygmyUnderwaterNopee>(this);
        if pointing_down {
            state.no_pee_changing_direction_downward = true;
        }
        state.no_pee_swimming_left = swimming_left;
        state.no_pee_turn_around_time = 0.0;
        state.no_pee_change_direction = true;
        state.no_pee_swim_target_x_vel = swim_target_x_vel;
    }
    pg_engine::logic::set_behavior(world, this, Some("PygmyNoPeeRoll"));
}

/// `-[PygmyLogic (NoPee) touchBeganNoPee:allTouches:]` @ 0x10018b318
///
/// A finger down thrusts; one on the other half of the screen than the pygmy swims toward
/// rolls it round. The first touch starts the game.
pub fn touch_began_no_pee(world: &mut World, this: Entity, touch: Entity) {
    let no_pee = world.obj::<PygmyPygmyTouchDrag>(this).no_pee;
    if !no_pee {
        return;
    }
    let (intro_complete, killed) = {
        let state = world.obj::<PygmyUnderwaterNopee>(this);
        (state.no_pee_intro_complete, state.no_pee_killed)
    };
    if !intro_complete || killed {
        return;
    }
    let active_touch = world.obj::<PygmyPygmyCoreFrame>(this).active_touch;
    if active_touch.is_some() {
        return;
    }
    if pg_engine::logic::is_hibernating(world, this) {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).active_touch = Some(touch);

    let started = world.obj::<PygmyUnderwaterNopee>(this).no_pee_started;
    if !started {
        world.obj_mut::<PygmyUnderwaterNopee>(this).no_pee_started = true;
        let no_pee_logic = no_pee_logic_of(world, this);
        if let Some(no_pee_logic) = no_pee_logic {
            no_pee_logic::on_pygmy_start_swimming(world, no_pee_logic);
        }
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyUnderwaterNopee>(this).no_pee_thrust_on = true;

    let display_object = pg_engine::logic::display_object(world, this);
    let rot_z = display::object(world, display_object).rot.z;
    let pocket_god = pg_engine::library::display_group(world, "PocketGod");
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let touch_x_pos = match pocket_god {
        Some(pocket_god) => display::view_to_display_object(world, pocket_god, touch_pos).x,
        None => 0.0,
    };

    let swimming_left = world.obj::<PygmyUnderwaterNopee>(this).no_pee_swimming_left;
    if touch_x_pos <= 240.0 {
        if !swimming_left {
            let pointing_down = (rot_z < 0.0 && -90.0 < rot_z) || (270.0 < rot_z && rot_z < 360.0);
            start_no_pee_roll(world, this, true, pointing_down, -100.0);
        }
    } else if swimming_left {
        let pointing_down = (rot_z < -90.0 && -180.0 < rot_z) || (180.0 < rot_z && rot_z < 270.0);
        start_no_pee_roll(world, this, false, pointing_down, 100.0);
    }

    let change_direction = world.obj::<PygmyUnderwaterNopee>(this).no_pee_change_direction;
    if !change_direction {
        pg_engine::logic::set_behavior(world, this, Some("PygmyNoPeeSwimFlap"));
    }
}

/// `-[PygmyLogic (NoPee) touchEndedInsideNoPee:allTouches:]` @ 0x10018b650
pub fn touch_ended_inside_no_pee(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<PygmyPygmyCoreFrame>(this).active_touch;
    if active_touch != Some(touch) {
        return;
    }

    world.obj_mut::<PygmyUnderwaterNopee>(this).no_pee_thrust_on = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).active_touch = None;
}

/// `-[PygmyLogic (NoPee) onPygmyNoPeeRollComplete:::]` @ 0x10018b680
pub fn seq_on_pygmy_no_pee_roll_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyUnderwaterNopee>(this).no_pee_change_direction = false;
    ItemResult::Done
}

/// `-[PygmyLogic (NoPee) waitUntilOffScreen:::]` @ 0x10018b698
pub fn seq_wait_until_off_screen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    if y_pos > 1000.0 || y_pos < 0.0 {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[PygmyLogic (NoPee) onNoPeeHitComplete:::]` @ 0x10018b6f0
pub fn seq_on_no_pee_hit_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, NO_PEE_EMBED_CHANNEL);

    let no_pee = no_pee_logic_of(world, this);
    if let Some(no_pee) = no_pee {
        no_pee_logic::stop_no_pee(world, no_pee);
    }
    ItemResult::Done
}
