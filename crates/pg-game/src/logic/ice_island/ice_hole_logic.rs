//! `IceHoleLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ice_island::icicle_logic;
use crate::logic::island_core::sprite_logic;
use crate::logic::ocean_creatures::sword_fish_logic;

/// `IceHoleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IceHoleLogic {
    /// `mIceHoleLogicListener`: `@"<IceHoleLogicListener>"`
    pub ice_hole_logic_listener: Option<Entity>,

    /// `mMeltingPygmyLogic`: `@"PygmyLogic"`
    pub melting_pygmy_logic: Option<Entity>,

    /// `mbIsMelting`: `B`
    pub is_melting: bool,

    /// `mbIsMelted`: `B`
    pub is_melted: bool,

    /// `mbIsIceSquidActive`: `B`
    pub is_ice_squid_active: bool,

    /// `mExitStallTime`: `f`
    pub exit_stall_time: f32,

    /// `mGrabStallTime`: `f`
    pub grab_stall_time: f32,

    /// `mbAimTentacle`: `B`
    pub aim_tentacle: bool,

    /// `mTentacleDisplayObject`: `@"BCDisplayObject"`
    pub tentacle_display_object: Option<Entity>,

    /// `mTentacleLogic`: `@"MultiModelLogic"`
    pub tentacle_logic: Option<Entity>,

    /// `mGrabbedPygmyLogic`: `@"PygmyLogic"`
    pub grabbed_pygmy_logic: Option<Entity>,

    /// `mFishingPygmyLogic`: `@"PygmyLogic"`
    pub fishing_pygmy_logic: Option<Entity>,

    /// `mSearchingPygmyLogic`: `@"PygmyLogic"`
    pub searching_pygmy_logic: Option<Entity>,

    /// `mSplashingPygmyLogic`: `@"PygmyLogic"`
    pub splashing_pygmy_logic: Option<Entity>,

    /// `mbIsEruptionActive`: `B`
    pub is_eruption_active: bool,

    /// `mEruptionStallTime`: `f`
    pub eruption_stall_time: f32,

    /// `mIcicleCount`: `i`
    pub icicle_count: i32,

    /// `mIcicleLogicArray`: `@"NSMutableArray"`
    pub icicle_logic_array: Vec<Entity>,
}

pub const ICE_HOLE_LOGIC: Class = Class {
    name: "IceHoleLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Tweenable channel of the pygmy in the hole (melting, splashing or fishing).
const PYGMY_CHANNEL: usize = 1;

/// Tweenable channel of the fishing pole.
const FISHING_POLE_CHANNEL: usize = 2;

/// Tweenable channel of the squid's tentacle.
const TENTACLE_CHANNEL: usize = 3;

/// How long a dragged pygmy must stay out of reach before the squid gives up, and hang right
/// above the hole before the tentacle grabs it.
const SQUID_STALL_TIME: f32 = 0.5;

/// Sequence-item handlers implemented by `IceHoleLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "iceHoleActivateComplete" => seq_ice_hole_activate_complete(world, this, call),
        "iceHoleWaitForSwordFish" => seq_ice_hole_wait_for_sword_fish(world, this, call),
        "waitToShowIceSquid" => seq_wait_to_show_ice_squid(world, this, call),
        "waitToGrabPygmy" => seq_wait_to_grab_pygmy(world, this, call),
        "tugOfWar" => seq_tug_of_war(world, this, call),
        "beheadPygmyComplete" => seq_behead_pygmy_complete(world, this, call),
        "eatPygmyComplete" => seq_eat_pygmy_complete(world, this, call),
        "squidExitComplete" => seq_squid_exit_complete(world, this, call),
        "icicleRain" => seq_icicle_rain(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `IceHoleLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animPygmySplashComplete" => anim_pygmy_splash_complete(world, this),
        "animCatchFish" => anim_catch_fish(world, this),
        "animGrabPygmy" => anim_grab_pygmy(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[IceHoleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&ICE_HOLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[IceHoleLogic initWithDisplayObject:]` @ 0x100184920
///
/// Also builds the squid's tentacle: a `Tentacle` multimodel driven by a `MultiModelLogic` and
/// bound to the hole's animation on channel 3.
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(IceHoleLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<IceHoleLogic>(this).icicle_logic_array = Vec::with_capacity(5);

    let tentacle_display_object = pg_engine::display::multi_model::spawn(world, Some("Tentacle"));
    world.obj_mut::<IceHoleLogic>(this).tentacle_display_object = Some(tentacle_display_object);

    let tentacle_logic = pg_engine::logic::multi_model::spawn(world, tentacle_display_object);
    world.obj_mut::<IceHoleLogic>(this).tentacle_logic = Some(tentacle_logic);

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::set_tweenable(world, animation, Some(tentacle_display_object), TENTACLE_CHANNEL);

    pg_engine::logic::set_behavior(world, tentacle_logic, Some("IceHoleIceSquidTentacle"));
    pg_engine::logic::stop_game_frame(world, this);
}

/// `-[IceHoleLogic setIceHoleLogicListener:]` @ 0x100184a6c
pub fn set_ice_hole_logic_listener(world: &mut World, this: Entity, ice_hole_logic_listener: Entity) {
    world.obj_mut::<IceHoleLogic>(this).ice_hole_logic_listener = Some(ice_hole_logic_listener);
}

/// `-[IceHoleLogic tentaclePos]` @ 0x100184ac8
///
/// The original returns a pointer to the tentacle's position, which its only caller reads.
pub fn tentacle_pos(world: &mut World, this: Entity) -> Vec4 {
    let tentacle_display_object = world.obj::<IceHoleLogic>(this).tentacle_display_object;
    match tentacle_display_object {
        Some(tentacle_display_object) => pg_engine::display::object(world, tentacle_display_object).pos,
        // The tentacle is created in `initWithDisplayObject:` and never released.
        None => Vec4::ZERO,
    }
}

/// `-[IceHoleLogic islandMinXPos]` @ 0x100184ae4
pub fn island_min_x_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<IceHoleLogic>(this).ice_hole_logic_listener;
    let Some(listener) = listener else {
        return 0.0;
    };
    crate::controller::island_min_x_pos(world, listener)
}

/// `-[IceHoleLogic islandMaxXPos]` @ 0x100184b00
pub fn island_max_x_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<IceHoleLogic>(this).ice_hole_logic_listener;
    let Some(listener) = listener else {
        return 0.0;
    };
    crate::controller::island_max_x_pos(world, listener)
}

/// `-[IceHoleLogic pygmyLogicArray:]` @ 0x100184b1c
///
/// Forwards to the listener with `self` as the argument (the caller's is ignored).
pub fn pygmy_logic_array(world: &mut World, this: Entity, _pygmy_logic_array: Entity) -> Vec<Entity> {
    let listener = world.obj::<IceHoleLogic>(this).ice_hole_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::pygmy_logic_array(world, listener, this)
}

/// `-[IceHoleLogic removePygmyFromIceHole:]` @ 0x100184b40
pub fn remove_pygmy_from_ice_hole(world: &mut World, this: Entity, pygmy: Entity) {
    let animation = pg_engine::logic::animation(world, this);

    let melting_pygmy = world.obj::<IceHoleLogic>(this).melting_pygmy_logic;
    if melting_pygmy == Some(pygmy) {
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        world.obj_mut::<IceHoleLogic>(this).melting_pygmy_logic = None;
    }

    let splashing_pygmy = world.obj::<IceHoleLogic>(this).splashing_pygmy_logic;
    if splashing_pygmy == Some(pygmy) {
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        world.obj_mut::<IceHoleLogic>(this).splashing_pygmy_logic = None;
    }
}

/// `-[IceHoleLogic onIcicleDie:]` @ 0x100184be0
///
/// The array held the icicle's last reference besides its animation player, which `shutdown`
/// drops: the icicle is deallocated here.
pub fn on_icicle_die(world: &mut World, this: Entity, icicle: Entity) {
    let icicle_display_object = pg_engine::logic::display_object(world, icicle);
    pg_engine::display::remove_from_display_group(world, icicle_display_object);

    world.obj_mut::<IceHoleLogic>(this).icicle_logic_array.retain(|&falling| falling != icicle);
    pg_engine::logic::shutdown(world, icicle);
    pg_engine::logic::dealloc(world, icicle);
}

/// `-[IceHoleLogic shouldPygmyWalkTowardIceHole:]` @ 0x100184c54
///
/// Calls one pygmy at a time: a different pygmy is refused while the one called before is
/// still in the pygmy array and answers YES to `canMeltIceHole`.
pub fn should_pygmy_walk_toward_ice_hole(world: &mut World, this: Entity, pygmy: Entity) -> bool {
    let searching_pygmy = world.obj::<IceHoleLogic>(this).searching_pygmy_logic;
    let other_pygmy_searching = searching_pygmy.is_some() && searching_pygmy != Some(pygmy);
    if other_pygmy_searching {
        let listener = world.obj::<IceHoleLogic>(this).ice_hole_logic_listener;
        let pygmies = match listener {
            Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
            None => Vec::new(),
        };
        for candidate in pygmies {
            let searching_pygmy = world.obj::<IceHoleLogic>(this).searching_pygmy_logic;
            if Some(candidate) == searching_pygmy && crate::pygmy::can_melt_ice_hole(world, candidate) {
                return false;
            }
        }
    }

    world.obj_mut::<IceHoleLogic>(this).searching_pygmy_logic = Some(pygmy);
    true
}

/// `-[IceHoleLogic meltIceHole:]` @ 0x100184dbc
///
/// The burning pygmy melts the hole where it stands.
pub fn melt_ice_hole(world: &mut World, this: Entity, pygmy: Entity) {
    {
        let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
        ice_hole.is_melting = true;
        ice_hole.melting_pygmy_logic = Some(pygmy);
    }

    let animation = pg_engine::logic::animation(world, this);
    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);

    // 240 is the original's position for a nil pygmy, which no caller passes.
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).pos.x = 240.0;
    let pygmy_x_pos = pg_engine::display::object(world, pygmy_display_object).pos.x;
    pg_engine::display::object_mut(world, display_object).pos.x = pygmy_x_pos;

    pg_engine::logic::set_behavior(world, this, Some("IceHoleActivate"));
}

/// `-[IceHoleLogic isFrozen]` @ 0x100184eb4
pub fn is_frozen(world: &mut World, this: Entity) -> bool {
    let ice_hole = world.obj::<IceHoleLogic>(this);
    if ice_hole.is_melted {
        return false;
    }
    !ice_hole.is_melting
}

/// `-[IceHoleLogic isAvailableForFishing]` @ 0x100184ee4
pub fn is_available_for_fishing(world: &mut World, this: Entity) -> bool {
    is_open_and_idle(world, this)
}

/// `-[IceHoleLogic isAvailableForFalling]` @ 0x100184f44
pub fn is_available_for_falling(world: &mut World, this: Entity) -> bool {
    is_open_and_idle(world, this)
}

/// The shared test of `isAvailableForFishing` and `isAvailableForFalling`: melted and done
/// melting, nobody fishing, no eruption and no squid.
fn is_open_and_idle(world: &World, this: Entity) -> bool {
    let ice_hole = world.obj::<IceHoleLogic>(this);
    ice_hole.is_melted
        && !ice_hole.is_melting
        && ice_hole.fishing_pygmy_logic.is_none()
        && !ice_hole.is_eruption_active
        && !ice_hole.is_ice_squid_active
}

/// `-[IceHoleLogic loseBalanceByIceHole:]` @ 0x100184fa4
pub fn lose_balance_by_ice_hole(_world: &mut World, _this: Entity, _lose_balance_by_ice_hole: Entity) {}

/// `-[IceHoleLogic tripIntoIceHole:fallForward:]` @ 0x100184fa8
pub fn trip_into_ice_hole(world: &mut World, this: Entity, _pygmy: Entity, fall_forward: bool) {
    let behavior = if fall_forward {
        "IceHolePygmyFallForward"
    } else {
        "IceHolePygmyFallBackward"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[IceHoleLogic startFishing:]` @ 0x100184fd0
pub fn start_fishing(world: &mut World, this: Entity, pygmy: Entity) {
    world.obj_mut::<IceHoleLogic>(this).fishing_pygmy_logic = Some(pygmy);
}

/// `-[IceHoleLogic startFishingSitDown:]` @ 0x100184fe0
///
/// Binds the pygmy registered by `startFishing:` (not the argument) and the fishing pole.
pub fn start_fishing_sit_down(world: &mut World, this: Entity, _pygmy: Entity) {
    let listener = world.obj::<IceHoleLogic>(this).ice_hole_logic_listener;
    let fishing_pole = match listener {
        Some(listener) => crate::controller::fishing_pole_logic(world, listener, this),
        None => None,
    };

    let animation = pg_engine::logic::animation(world, this);
    let fishing_pygmy = world.obj::<IceHoleLogic>(this).fishing_pygmy_logic;
    let fishing_pygmy_display_object = fishing_pygmy.map(|pygmy| pg_engine::logic::display_object(world, pygmy));
    pg_engine::animation::set_tweenable(world, animation, fishing_pygmy_display_object, PYGMY_CHANNEL);

    let fishing_pole_display_object = fishing_pole.map(|pole| pg_engine::logic::display_object(world, pole));
    pg_engine::animation::set_tweenable(world, animation, fishing_pole_display_object, FISHING_POLE_CHANNEL);

    pg_engine::logic::set_behavior(world, this, Some("IceHoleFishingInit"));
}

/// `-[IceHoleLogic stopFishing:]` @ 0x1001850bc
pub fn stop_fishing(world: &mut World, this: Entity, _pygmy: Entity) {
    world.obj_mut::<IceHoleLogic>(this).fishing_pygmy_logic = None;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FISHING_POLE_CHANNEL);

    pg_engine::logic::set_behavior(world, this, Some("IceHoleInit"));
}

/// `-[IceHoleLogic fishFallIntoIceHole:]` @ 0x10018513c
pub fn fish_fall_into_ice_hole(world: &mut World, this: Entity, _fish: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("IceHoleFishFall"));
}

/// `-[IceHoleLogic burntPygmyFallIntoIceHole:]` @ 0x100185154
pub fn burnt_pygmy_fall_into_ice_hole(world: &mut World, this: Entity, pygmy: Entity) {
    world.obj_mut::<IceHoleLogic>(this).splashing_pygmy_logic = Some(pygmy);

    let animation = pg_engine::logic::animation(world, this);
    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);

    pg_engine::logic::set_behavior(world, this, Some("IceHolePygmyFall"));
}

/// `-[IceHoleLogic animPygmySplashComplete]` @ 0x1001851d4
pub fn anim_pygmy_splash_complete(world: &mut World, this: Entity) {
    let splashing_pygmy = world.obj::<IceHoleLogic>(this).splashing_pygmy_logic;
    let Some(splashing_pygmy) = splashing_pygmy else {
        return;
    };

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, PYGMY_CHANNEL);
    crate::pygmy::ice_hole_splash_complete(world, splashing_pygmy);
    world.obj_mut::<IceHoleLogic>(this).splashing_pygmy_logic = None;
}

/// `-[IceHoleLogic beheadPygmy:]` @ 0x10018523c
pub fn behead_pygmy(world: &mut World, this: Entity, _pygmy: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("IceHoleBeheadPygmy"));
}

/// `-[IceHoleLogic enterIsland:]` @ 0x100185254
///
/// The hole is frozen again on every visit.
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    {
        let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
        ice_hole.is_melted = false;
        ice_hole.is_melting = false;
        ice_hole.aim_tentacle = false;
    }
    pg_engine::logic::start_game_frame(world, this);
    pg_engine::logic::set_behavior(world, this, Some("IceHoleEnterIsland"));
}

/// `-[IceHoleLogic exitIsland:]` @ 0x1001852bc
///
/// Lets go of the melting, fishing and splashing pygmies and freezes the hole; `exitIsland`
/// set also stops its game frame.
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    if exit_island {
        pg_engine::logic::stop_game_frame(world, this);
    }

    let melting_pygmy = world.obj::<IceHoleLogic>(this).melting_pygmy_logic;
    if let Some(melting_pygmy) = melting_pygmy {
        crate::pygmy::melt_ice_hole_complete(world, melting_pygmy, this);
        world.obj_mut::<IceHoleLogic>(this).melting_pygmy_logic = None;
    }

    let animation = pg_engine::logic::animation(world, this);
    let fishing_pygmy = world.obj::<IceHoleLogic>(this).fishing_pygmy_logic;
    if let Some(fishing_pygmy) = fishing_pygmy {
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FISHING_POLE_CHANNEL);
        crate::pygmy::stop_fishing_from_ice_hole(world, fishing_pygmy, this);
        world.obj_mut::<IceHoleLogic>(this).fishing_pygmy_logic = None;
    }

    let splashing_pygmy = world.obj::<IceHoleLogic>(this).splashing_pygmy_logic;
    if let Some(splashing_pygmy) = splashing_pygmy {
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::ice_hole_splash_complete(world, splashing_pygmy);
        world.obj_mut::<IceHoleLogic>(this).splashing_pygmy_logic = None;
    }

    {
        let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
        ice_hole.is_melted = false;
        ice_hole.is_melting = false;
        ice_hole.aim_tentacle = false;
    }
    pg_engine::logic::set_behavior(world, this, Some("IceHoleExitIsland"));
}

/// `-[IceHoleLogic enableIceSquid]` @ 0x10018540c
pub fn enable_ice_squid(_world: &mut World, _this: Entity) {}

/// `-[IceHoleLogic disableIceSquid]` @ 0x100185410
pub fn disable_ice_squid(_world: &mut World, _this: Entity) {}

/// `-[IceHoleLogic onGameFrame:]` @ 0x1001854f8
///
/// While the squid holds a pygmy, the tentacle points at the dragged pygmy and stretches to it
/// (the tentacle model is 56 units long).
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }
    if !world.obj::<IceHoleLogic>(this).aim_tentacle {
        return;
    }
    let Some(dragged_pygmy) = listener_dragged_pygmy_logic(world, this) else {
        return;
    };
    let Some(tentacle_display_object) = world.obj::<IceHoleLogic>(this).tentacle_display_object else {
        return;
    };

    let tentacle_pos = pg_engine::display::object(world, tentacle_display_object).pos;
    let dragged_display_object = pg_engine::logic::display_object(world, dragged_pygmy);
    let dragged_pos = pg_engine::display::object(world, dragged_display_object).pos;

    let delta_x = dragged_pos.x - tentacle_pos.x;
    let delta_y = dragged_pos.y - tentacle_pos.y;
    let angle = delta_y.atan2(delta_x);
    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

    let mut tentacle = pg_engine::display::object_mut(world, tentacle_display_object);
    tentacle.rot.z = pg_engine::math::rad_to_deg(angle);
    tentacle.scale.x = distance / 56.0;
}

/// `[mIceHoleLogicListener draggedPygmyLogic:self]`.
fn listener_dragged_pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<IceHoleLogic>(this).ice_hole_logic_listener?;
    crate::controller::dragged_pygmy_logic(world, listener, this)
}

/// `[mIceHoleLogicListener swordFishLogic:self]`.
fn listener_sword_fish_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<IceHoleLogic>(this).ice_hole_logic_listener?;
    crate::controller::sword_fish_logic(world, listener, this)
}

/// `-[IceHoleLogic animCatchFish]` @ 0x100185640
///
/// The swordfish is pulled to the hole and caught; the fishing pygmy gets the credit.
pub fn anim_catch_fish(world: &mut World, this: Entity) {
    let sword_fish = listener_sword_fish_logic(world, this);
    let display_object = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display_object).pos;

    // The original writes through the NULL position of a nil swordfish; there is always one
    // while ice fishing.
    if let Some(sword_fish) = sword_fish {
        let sword_fish_display_object = pg_engine::logic::display_object(world, sword_fish);
        {
            let mut object = pg_engine::display::object_mut(world, sword_fish_display_object);
            object.pos.x = pos.x;
            object.pos.y = pos.y;
            object.pos.z = pos.z;
        }
        sword_fish_logic::catch(world, sword_fish);
    }

    let fishing_pygmy = world.obj::<IceHoleLogic>(this).fishing_pygmy_logic;
    if let Some(fishing_pygmy) = fishing_pygmy {
        crate::pygmy::catch_sword_fish(world, fishing_pygmy);
    }
}

/// `-[IceHoleLogic iceHoleActivateComplete:::]` @ 0x100185710
pub fn seq_ice_hole_activate_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

    let melting_pygmy = world.obj::<IceHoleLogic>(this).melting_pygmy_logic;
    if let Some(melting_pygmy) = melting_pygmy {
        crate::pygmy::melt_ice_hole_complete(world, melting_pygmy, this);
        world.obj_mut::<IceHoleLogic>(this).melting_pygmy_logic = None;
    }

    let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
    ice_hole.is_melted = true;
    ice_hole.is_melting = false;
    ItemResult::Done
}

/// `-[IceHoleLogic iceHoleWaitForSwordFish:::]` @ 0x1001857a0
pub fn seq_ice_hole_wait_for_sword_fish(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let sword_fish = listener_sword_fish_logic(world, this);
    let can_be_caught = match sword_fish {
        Some(sword_fish) => sword_fish_logic::can_be_caught(world, sword_fish),
        None => false,
    };
    if !can_be_caught {
        return ItemResult::Pending;
    }

    pg_engine::logic::set_behavior(world, this, Some("IceHoleFishingCatchFish"));
    ItemResult::Done
}

/// `-[IceHoleLogic waitToShowIceSquid:::]` @ 0x100185818
///
/// The squid comes up when the island has it and a pygmy that it can grab is dragged right
/// above the hole.
pub fn seq_wait_to_show_ice_squid(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<IceHoleLogic>(this).ice_hole_logic_listener;
    let ice_squid_enabled = match listener {
        Some(listener) => crate::controller::is_ice_squid_enabled(world, listener),
        None => false,
    };
    if !ice_squid_enabled {
        return ItemResult::Pending;
    }

    let Some(dragged_pygmy) = listener_dragged_pygmy_logic(world, this) else {
        return ItemResult::Pending;
    };
    if !crate::pygmy::can_be_grabbed_by_tentacle(world, dragged_pygmy, this) {
        return ItemResult::Pending;
    }

    let dragged_display_object = pg_engine::logic::display_object(world, dragged_pygmy);
    let dragged_pos = pg_engine::display::object(world, dragged_display_object).pos;
    let display_object = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display_object).pos;

    let height_above_hole = dragged_pos.y - pos.y;
    let above_hole = (dragged_pos.x - pos.x).abs() < 50.0 && height_above_hole < 200.0 && 0.0 < height_above_hole;
    if !above_hole {
        return ItemResult::Pending;
    }

    world.obj_mut::<IceHoleLogic>(this).is_ice_squid_active = true;
    pg_engine::logic::set_behavior(world, this, Some("IceHoleIceSquidEnter"));
    ItemResult::Replaced
}

/// `-[IceHoleLogic waitToGrabPygmy:::]` @ 0x100185960
///
/// The squid grabs a dragged pygmy held close above the hole for half a second, and dives
/// again once no pygmy has been above the hole for half a second.
pub fn seq_wait_to_grab_pygmy(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
        ice_hole.grab_stall_time = SQUID_STALL_TIME;
        ice_hole.exit_stall_time = SQUID_STALL_TIME;
    }

    let dragged_pygmy = listener_dragged_pygmy_logic(world, this);
    if let Some(dragged_pygmy) = dragged_pygmy {
        let dragged_display_object = pg_engine::logic::display_object(world, dragged_pygmy);
        let dragged_pos = pg_engine::display::object(world, dragged_display_object).pos;
        let display_object = pg_engine::logic::display_object(world, this);
        let pos = pg_engine::display::object(world, display_object).pos;

        let distance_x = (dragged_pos.x - pos.x).abs();
        let height_above_hole = dragged_pos.y - pos.y;

        let above_hole = distance_x < 50.0 && height_above_hole < 200.0 && 0.0 < height_above_hole;
        if above_hole {
            world.obj_mut::<IceHoleLogic>(this).exit_stall_time = SQUID_STALL_TIME;
        }

        let within_reach = distance_x < 35.0 && height_above_hole < 150.0 && 100.0 < height_above_hole;
        if !within_reach {
            world.obj_mut::<IceHoleLogic>(this).grab_stall_time = SQUID_STALL_TIME;
        } else {
            let grab_stall_time = world.obj::<IceHoleLogic>(this).grab_stall_time;
            if grab_stall_time <= 0.0 {
                {
                    let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
                    ice_hole.aim_tentacle = true;
                    ice_hole.grabbed_pygmy_logic = Some(dragged_pygmy);
                }
                pg_engine::logic::set_behavior(world, this, Some("IceHoleIceSquidGrabPygmy"));
                return ItemResult::Replaced;
            }
        }
    }

    let exit_stall_time = world.obj::<IceHoleLogic>(this).exit_stall_time;
    if exit_stall_time <= 0.0 {
        world.obj_mut::<IceHoleLogic>(this).is_ice_squid_active = false;
        pg_engine::logic::set_behavior(world, this, Some("IceHoleIceSquidExit"));
        return ItemResult::Replaced;
    }

    let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
    ice_hole.exit_stall_time = ice_hole.exit_stall_time - call.delta_time;
    ice_hole.grab_stall_time = ice_hole.grab_stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[IceHoleLogic tugOfWar:::]` @ 0x100185b74
///
/// Once the finger lets go of the grabbed pygmy, the squid eats it.
pub fn seq_tug_of_war(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if listener_dragged_pygmy_logic(world, this).is_some() {
        return ItemResult::Pending;
    }

    let grabbed_pygmy = world.obj::<IceHoleLogic>(this).grabbed_pygmy_logic;
    if let Some(grabbed_pygmy) = grabbed_pygmy {
        crate::pygmy::eaten_by_ice_squid(world, grabbed_pygmy, this);
    }
    pg_engine::logic::set_behavior(world, this, Some("IceHoleEatPygmy"));
    ItemResult::Replaced
}

/// `-[IceHoleLogic beheadPygmyComplete:::]` @ 0x100185bfc
pub fn seq_behead_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
    ice_hole.aim_tentacle = false;
    ice_hole.is_ice_squid_active = false;
    ice_hole.grabbed_pygmy_logic = None;
    ItemResult::Done
}

/// `-[IceHoleLogic eatPygmyComplete:::]` @ 0x100185c28
pub fn seq_eat_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
    ice_hole.aim_tentacle = false;
    ice_hole.is_ice_squid_active = false;
    ice_hole.grabbed_pygmy_logic = None;
    ItemResult::Done
}

/// `-[IceHoleLogic squidExitComplete:::]` @ 0x100185c54
pub fn seq_squid_exit_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
    ice_hole.aim_tentacle = false;
    ice_hole.is_ice_squid_active = false;
    ItemResult::Done
}

/// `-[IceHoleLogic animGrabPygmy]` @ 0x100185c74
pub fn anim_grab_pygmy(world: &mut World, this: Entity) {
    let dragged_pygmy = listener_dragged_pygmy_logic(world, this);
    if let Some(dragged_pygmy) = dragged_pygmy {
        crate::pygmy::grabbed_by_tentacle(world, dragged_pygmy, this);
    }
}

/// `-[IceHoleLogic icicleRain:::]` @ 0x100185cd0
///
/// Spawns `[1] minCount`…`[3] maxCount` icicles, one every `[0] minTime`…`[2] maxTime`
/// seconds (0.01 s steps), each at a random island x (0.1 steps); completes with the last one.
pub fn seq_icicle_rain(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_count = call.int("minCount");
        let max_count = call.int("maxCount");
        let count_range = 1u32.wrapping_sub(min_count as u32).wrapping_add(max_count as u32);
        let count_offset = world.resource_mut::<Random>().arc4random_mod(count_range);
        let icicle_count = count_offset.wrapping_add(min_count as u32) as i32;

        let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
        ice_hole.icicle_count = icicle_count;
        ice_hole.eruption_stall_time = 0.0;
    }

    let mut stall_time = world.obj::<IceHoleLogic>(this).eruption_stall_time;
    if stall_time == 0.0 {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        world.obj_mut::<IceHoleLogic>(this).eruption_stall_time = stall_time;
    }

    if !(stall_time <= 0.0) {
        let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
        ice_hole.eruption_stall_time = ice_hole.eruption_stall_time - call.delta_time;
        return ItemResult::Pending;
    }

    let icicle_count = {
        let mut ice_hole = world.obj_mut::<IceHoleLogic>(this);
        ice_hole.eruption_stall_time = 0.0;
        ice_hole.icicle_count = ice_hole.icicle_count.wrapping_sub(1);
        ice_hole.icicle_count
    };
    let result = if icicle_count == 0 {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    let icicle_model = pg_engine::display::multi_model::spawn(world, Some("Icicle"));
    let listener = world.obj::<IceHoleLogic>(this).ice_hole_logic_listener;
    let mut island_min_x = 0.0;
    let mut island_max_x = 0.0;
    if let Some(listener) = listener {
        island_min_x = crate::controller::island_min_x_pos(world, listener);
        island_max_x = crate::controller::island_max_x_pos(world, listener);
    }

    // The same draw as a stall time, in tenths of a unit across the island.
    let random = world.resource_mut::<Random>().arc4random();
    let icicle_x_pos = pg_engine::logic::random_scaled_time(island_min_x, island_max_x, 10.0, random);
    {
        let mut object = pg_engine::display::object_mut(world, icicle_model);
        object.pos.x = icicle_x_pos;
        object.pos.y = 400.0;
        object.pos.z = -277.12;
    }
    let marker = pg_engine::library::display_marker(world, "LavaDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(icicle_model));
    }

    let icicle = icicle_logic::spawn(world, icicle_model);
    icicle_logic::set_icicle_logic_listener(world, icicle, this);
    world.obj_mut::<IceHoleLogic>(this).icicle_logic_array.push(icicle);
    result
}
