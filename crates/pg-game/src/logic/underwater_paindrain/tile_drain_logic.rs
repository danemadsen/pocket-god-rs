//! `TileDrainLogic` (superclass `SpriteLogic`).
//!
//! The drain in the floor of the underwater ruins and the host of the Pain Drain mini-game.
//! Dragging up from the drain lifts its plug; the open drain then sucks in a pygmy standing over
//! it, who has to fall through the gaps of four rising platforms to keep away from the spikes.
//! Its listener is the view controller.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::underwater_paindrain::{pain_drain_platform_logic, tile_plug_logic};
use pg_engine::animation;
use pg_engine::display;
use pg_engine::display::multi_model;
use pg_engine::logic;
use pg_engine::touch::BcTouch;

/// `TileDrainLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct TileDrainLogic {
    /// `mTileDrainLogicListener`: `@"<TileDrainLogicListener>"`
    pub tile_drain_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbTransitionIn`: `B`
    pub transition_in: bool,

    /// `mbUnplugged`: `B`
    pub unplugged: bool,

    /// `mbLifting`: `B`
    pub lifting: bool,

    /// `mTouchIslandStartPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_island_start_pos: Vec4,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mbPainDrainActive`: `B`
    pub pain_drain_active: bool,

    /// `mPainDrainPygmyLogic`: `@"PygmyLogic"`
    pub pain_drain_pygmy_logic: Option<Entity>,

    /// `mPlatformYPosVel`: `f`
    pub platform_y_pos_vel: f32,

    /// `mPainDrainPlatformLogicArray`: `@"NSMutableArray"`
    pub pain_drain_platform_logic_array: Vec<Entity>,

    /// `mPainDrainBackgroundLogic`: `@"MultiModelLogic"`
    pub pain_drain_background_logic: Option<Entity>,

    /// `mPainDrainLeftChainLogic`: `@"MultiModelLogic"`
    pub pain_drain_left_chain_logic: Option<Entity>,

    /// `mPainDrainRightChainLogic`: `@"MultiModelLogic"`
    pub pain_drain_right_chain_logic: Option<Entity>,

    /// `mPainDrainSpikesTopLogic`: `@"MultiModelLogic"`
    pub pain_drain_spikes_top_logic: Option<Entity>,

    /// `mPainDrainSpikesBottomLogic`: `@"MultiModelLogic"`
    pub pain_drain_spikes_bottom_logic: Option<Entity>,
}

pub const TILE_DRAIN_LOGIC: Class = Class {
    name: "TileDrainLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The animation channel the caught pygmy is bound to.
const PYGMY_CHANNEL: usize = 1;

/// The number of platforms `createPlatforms` makes.
const PLATFORM_COUNT: usize = 4;

/// The platforms' rising speed when Pain Drain starts; it grows by 1 per second up to the max.
const START_PLATFORM_Y_POS_VEL: f32 = 50.0;
const MAX_PLATFORM_Y_POS_VEL: f32 = 210.0;

/// A touch dragged above this height (in the drain's group) lifts the plug.
const LIFT_Y_POS: f32 = 150.0;

/// `catchPygmy:::` takes a pygmy within this horizontal distance of the drain and this height
/// above it.
const CATCH_X_DISTANCE: f32 = 50.0;
const CATCH_Y_DISTANCE: f32 = 130.0;

/// Sequence-item handlers implemented by `TileDrainLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "tileDrainSpikePygmyComplete" => seq_tile_drain_spike_pygmy_complete(world, this, call),
        "unplugTilePlug" => seq_unplug_tile_plug(world, this, call),
        "catchPygmyComplete" => seq_catch_pygmy_complete(world, this, call),
        "catchPygmy" => seq_catch_pygmy(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[logic setBehavior:behavior]` for a Pain Drain scenery logic, which may be nil.
fn set_scenery_behavior(world: &mut World, scenery_logic: Option<Entity>, behavior: &str) {
    if let Some(scenery_logic) = scenery_logic {
        logic::set_behavior(world, scenery_logic, Some(behavior));
    }
}

/// `[logic displayObject]` for a Pain Drain scenery logic, which may be nil.
fn scenery_display_object(world: &World, scenery_logic: Option<Entity>) -> Option<Entity> {
    let scenery_logic = scenery_logic?;
    let display_object = logic::display_object(world, scenery_logic);
    Some(display_object)
}

/// `[[group getDisplayObject:id] logic]` (nil for a missing group or object).
fn group_object_logic(world: &World, group: Option<Entity>, id: &str) -> Option<Entity> {
    let group = group?;
    let display_object = display::group::get_display_object(world, group, id)?;
    display::object(world, display_object).logic
}

/// `[listener tilePlugLogic:self]` (nil for a nil listener).
fn listener_tile_plug_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<TileDrainLogic>(this).tile_drain_logic_listener?;
    crate::controller::tile_plug_logic(world, listener, this)
}

/// `[[TileDrainLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&TILE_DRAIN_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[TileDrainLogic initWithDisplayObject:]` @ 0x100117114
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(TileDrainLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<TileDrainLogic>(this).pain_drain_platform_logic_array = Vec::with_capacity(5);

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
}

/// `-[TileDrainLogic setTileDrainLogicListener:]` @ 0x1001171bc
pub fn set_tile_drain_logic_listener(world: &mut World, this: Entity, tile_drain_logic_listener: Entity) {
    world.obj_mut::<TileDrainLogic>(this).tile_drain_logic_listener = Some(tile_drain_logic_listener);
}

/// `-[TileDrainLogic pygmyLogicArray:]` @ 0x100117218
///
/// Forwards to the listener with `self` as the argument (the caller's is ignored).
pub fn pygmy_logic_array(world: &mut World, this: Entity, _pygmy_logic_array: Entity) -> Vec<Entity> {
    let listener = world.obj::<TileDrainLogic>(this).tile_drain_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::pygmy_logic_array(world, listener, this)
}

/// `-[TileDrainLogic painDrainPlatformLogicArray]` @ 0x10011723c
pub fn pain_drain_platform_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<TileDrainLogic>(this).pain_drain_platform_logic_array.clone()
}

/// `-[TileDrainLogic dealloc]` @ 0x100118434
///
/// For the drain's only owner, the view controller, which releases it in its own `dealloc`.
pub fn dealloc(world: &mut World, this: Entity) {
    remove_display_objects(world, this);

    // `[mPainDrainPlatformLogicArray release]` and `[mTileDrainLogicListener release]` need no
    // port; `[super dealloc]`.
    logic::dealloc(world, this);
}

/// `-[TileDrainLogic removePlatforms]` @ 0x10011724c
///
/// Clears the Pain Drain scenery and takes the platforms away; the array was their last owner.
pub fn remove_platforms(world: &mut World, this: Entity) {
    let (background, left_chain, right_chain, spikes_top, spikes_bottom) = {
        let drain = world.obj::<TileDrainLogic>(this);
        (
            drain.pain_drain_background_logic,
            drain.pain_drain_left_chain_logic,
            drain.pain_drain_right_chain_logic,
            drain.pain_drain_spikes_top_logic,
            drain.pain_drain_spikes_bottom_logic,
        )
    };
    set_scenery_behavior(world, background, "PainDrainBackgroundClear");
    set_scenery_behavior(world, left_chain, "PainDrainChainClear");
    set_scenery_behavior(world, right_chain, "PainDrainChainClear");
    set_scenery_behavior(world, spikes_top, "PainDrainSpikesTopClear");
    set_scenery_behavior(world, spikes_bottom, "PainDrainSpikesBottomClear");

    let platforms = world.obj::<TileDrainLogic>(this).pain_drain_platform_logic_array.clone();
    for &platform in &platforms {
        let platform_display_object = logic::display_object(world, platform);
        display::remove_from_display_group(world, platform_display_object);
        logic::shutdown(world, platform);
    }

    world.obj_mut::<TileDrainLogic>(this).pain_drain_platform_logic_array.clear();

    // `removeAllObjects` released the platforms' last owner.
    for platform in platforms {
        pain_drain_platform_logic::dealloc(world, platform);
    }
}

/// `-[TileDrainLogic createPlatforms]` @ 0x100117454
///
/// Starts the Pain Drain scenery (found in `PainDrainDisplayGroup` the first time) from y = 0
/// and puts four new platforms after `PainDrainPlatformDisplayMarker`.
pub fn create_platforms(world: &mut World, this: Entity) {
    let platform_marker = pg_engine::library::display_marker(world, "PainDrainPlatformDisplayMarker");

    let background = world.obj::<TileDrainLogic>(this).pain_drain_background_logic;
    if background.is_none() {
        let group = pg_engine::library::display_group(world, "PainDrainDisplayGroup");
        let background = group_object_logic(world, group, "Background");
        let left_chain = group_object_logic(world, group, "LeftChain");
        let right_chain = group_object_logic(world, group, "RightChain");
        let spikes_top = group_object_logic(world, group, "SpikesTop");
        let spikes_bottom = group_object_logic(world, group, "SpikesBottom");

        let mut drain = world.obj_mut::<TileDrainLogic>(this);
        drain.pain_drain_background_logic = background;
        drain.pain_drain_left_chain_logic = left_chain;
        drain.pain_drain_right_chain_logic = right_chain;
        drain.pain_drain_spikes_top_logic = spikes_top;
        drain.pain_drain_spikes_bottom_logic = spikes_bottom;
    }

    let (background, left_chain, right_chain, spikes_top, spikes_bottom) = {
        let drain = world.obj::<TileDrainLogic>(this);
        (
            drain.pain_drain_background_logic,
            drain.pain_drain_left_chain_logic,
            drain.pain_drain_right_chain_logic,
            drain.pain_drain_spikes_top_logic,
            drain.pain_drain_spikes_bottom_logic,
        )
    };
    set_scenery_behavior(world, background, "PainDrainBackgroundInit");
    set_scenery_behavior(world, left_chain, "PainDrainChainInit");
    set_scenery_behavior(world, right_chain, "PainDrainChainInit");
    set_scenery_behavior(world, spikes_top, "PainDrainSpikesTopInit");
    set_scenery_behavior(world, spikes_bottom, "PainDrainSpikesBottomInit");

    let scrolling_scenery = [background, left_chain, right_chain];
    for scenery_logic in scrolling_scenery {
        if let Some(scenery_object) = scenery_display_object(world, scenery_logic) {
            display::object_mut(world, scenery_object).pos.y = 0.0;
        }
    }

    for _ in 0..PLATFORM_COUNT {
        let platform_model = multi_model::spawn(world, Some("Platform"));
        if let Some(platform_marker) = platform_marker {
            display::insert_after(world, platform_marker, Some(platform_model));
        }

        let platform = pain_drain_platform_logic::spawn(world, platform_model);
        pain_drain_platform_logic::set_pain_drain_platform_logic_listener(world, platform, this);
        world.obj_mut::<TileDrainLogic>(this).pain_drain_platform_logic_array.push(platform);
    }
}

/// `-[TileDrainLogic removeDisplayObjects]` @ 0x1001177e8
pub fn remove_display_objects(world: &mut World, this: Entity) {
    remove_platforms(world, this);
}

/// `-[TileDrainLogic onGameFrame:]` @ 0x1001177f8
///
/// Every platform rises at `mPlatformYPosVel`. During Pain Drain the speed grows by 1 per second
/// up to 210, the background scrolls at half of it and the chains at full speed, and the
/// background's height is the score (no `[super onGameFrame:]`).
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let platforms = world.obj::<TileDrainLogic>(this).pain_drain_platform_logic_array.clone();
    for platform in platforms {
        let platform_y_pos_vel = world.obj::<TileDrainLogic>(this).platform_y_pos_vel;
        logic::logic_mut(world, platform).pos_vel.y = platform_y_pos_vel;
    }

    let pain_drain_active = world.obj::<TileDrainLogic>(this).pain_drain_active;
    if !pain_drain_active {
        return;
    }

    let platform_y_pos_vel = {
        let mut drain = world.obj_mut::<TileDrainLogic>(this);
        let faster_y_pos_vel = drain.platform_y_pos_vel + dt;
        drain.platform_y_pos_vel = if faster_y_pos_vel > MAX_PLATFORM_Y_POS_VEL { MAX_PLATFORM_Y_POS_VEL } else { faster_y_pos_vel };
        drain.platform_y_pos_vel
    };

    let (spikes_top, background, left_chain, right_chain, listener) = {
        let drain = world.obj::<TileDrainLogic>(this);
        (
            drain.pain_drain_spikes_top_logic,
            drain.pain_drain_background_logic,
            drain.pain_drain_left_chain_logic,
            drain.pain_drain_right_chain_logic,
            drain.tile_drain_logic_listener,
        )
    };
    if spikes_top.is_none() {
        return;
    }

    let background_object = scenery_display_object(world, background);
    let left_chain_object = scenery_display_object(world, left_chain);
    let right_chain_object = scenery_display_object(world, right_chain);

    let scroll = platform_y_pos_vel * dt;
    if let Some(background_object) = background_object {
        let mut object = display::object_mut(world, background_object);
        object.pos.y = object.pos.y + scroll * 0.5;
    }
    for chain_object in [left_chain_object, right_chain_object] {
        if let Some(chain_object) = chain_object {
            let mut object = display::object_mut(world, chain_object);
            object.pos.y = object.pos.y + scroll;
        }
    }

    // The original reads the background unchecked; all five scenery logics come from one group.
    let Some(background_object) = background_object else {
        return;
    };
    let background_y_pos = display::object(world, background_object).pos.y;
    let score = (background_y_pos * 0.3125) as i32;
    if let Some(listener) = listener {
        crate::controller::on_pain_drain_score(world, listener, score);
    }
}

/// `-[TileDrainLogic stopPainDrain]` @ 0x100117a64
///
/// The pygmy hit the spikes: the platforms stop and the drain finishes the pygmy off after a
/// pause (`TileDrainPygmySpiked`).
pub fn stop_pain_drain(world: &mut World, this: Entity) {
    let pain_drain_active = world.obj::<TileDrainLogic>(this).pain_drain_active;
    if !pain_drain_active {
        return;
    }

    {
        let mut drain = world.obj_mut::<TileDrainLogic>(this);
        drain.pain_drain_active = false;
        drain.platform_y_pos_vel = 0.0;
    }
    logic::set_behavior(world, this, Some("TileDrainPygmySpiked"));
}

/// `-[TileDrainLogic tileDrainSpikePygmyComplete:::]` @ 0x100117b04
///
/// Ends Pain Drain: the platforms go, the drain returns to the ruins and the pygmy dies.
pub fn seq_tile_drain_spike_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    remove_platforms(world, this);

    let display_object = logic::display_object(world, this);
    let tsunami_marker = pg_engine::library::display_marker(world, "TsunamiDisplayMarker");
    if let Some(tsunami_marker) = tsunami_marker {
        display::insert_after(world, tsunami_marker, Some(display_object));
    }

    let pygmy = world.obj::<TileDrainLogic>(this).pain_drain_pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::die_from_pain_drain(world, pygmy);
    }
    world.obj_mut::<TileDrainLogic>(this).pain_drain_pygmy_logic = None;

    let own_animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, own_animation, PYGMY_CHANNEL);

    let listener = world.obj::<TileDrainLogic>(this).tile_drain_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_pain_drain_stop(world, listener, this);
    }
    ItemResult::Done
}

/// `-[TileDrainLogic activate]` @ 0x100117be8
pub fn activate(world: &mut World, this: Entity) {
    let tsunami_marker = pg_engine::library::display_marker(world, "TsunamiDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(tsunami_marker) = tsunami_marker {
        display::insert_after(world, tsunami_marker, Some(display_object));
    }
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("TileDrainTransitionIn"));
}

/// `-[TileDrainLogic deactivate]` @ 0x100117c84
///
/// Takes the drain out and lets a caught pygmy go. A running Pain Drain ends as if the pygmy
/// were spiked (`stopPainDrain` replaces `TileDrainTransitionOut`).
pub fn deactivate(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("TileDrainTransitionOut"));

    {
        let mut drain = world.obj_mut::<TileDrainLogic>(this);
        drain.lifting = false;
        drain.unplugged = false;
        drain.active_touch = None;
    }
    remove_display_objects(world, this);
    stop_pain_drain(world, this);

    let pygmy = world.obj::<TileDrainLogic>(this).pain_drain_pygmy_logic;
    if let Some(pygmy) = pygmy {
        let own_animation = logic::animation(world, this);
        animation::clear_tweenable_channel_id_now(world, own_animation, PYGMY_CHANNEL);
        crate::pygmy::escape_from_drain(world, pygmy, this);
        world.obj_mut::<TileDrainLogic>(this).pain_drain_pygmy_logic = None;
    }

    // Original bug, kept: `mActiveTouch` was already cleared above, so this never runs.
    let active_touch = world.obj::<TileDrainLogic>(this).active_touch;
    if active_touch.is_some() {
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        world.obj_mut::<TileDrainLogic>(this).active_touch = None;
    }
}

/// `-[TileDrainLogic enterIsland:]` @ 0x100117da4
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let (active, transition_in) = {
        let drain = world.obj::<TileDrainLogic>(this);
        (drain.active, drain.transition_in)
    };
    if active {
        return;
    }
    world.obj_mut::<TileDrainLogic>(this).active = true;
    if transition_in {
        activate(world, this);
    }
}

/// `-[TileDrainLogic exitIsland]` @ 0x100117de0
pub fn exit_island(world: &mut World, this: Entity) {
    let (active, transition_in) = {
        let drain = world.obj::<TileDrainLogic>(this);
        (drain.active, drain.transition_in)
    };
    if !active {
        return;
    }
    world.obj_mut::<TileDrainLogic>(this).active = false;
    if transition_in {
        deactivate(world, this);
    }
}

/// `-[TileDrainLogic transitionIn]` @ 0x100117e18
pub fn transition_in(world: &mut World, this: Entity) {
    let (active, transition_in) = {
        let drain = world.obj::<TileDrainLogic>(this);
        (drain.active, drain.transition_in)
    };
    if transition_in {
        return;
    }
    world.obj_mut::<TileDrainLogic>(this).transition_in = true;
    if active {
        activate(world, this);
    }
}

/// `-[TileDrainLogic transitionOut]` @ 0x100117e54
pub fn transition_out(world: &mut World, this: Entity) {
    let (active, transition_in) = {
        let drain = world.obj::<TileDrainLogic>(this);
        (drain.active, drain.transition_in)
    };
    if !transition_in {
        return;
    }
    world.obj_mut::<TileDrainLogic>(this).transition_in = false;
    if active {
        deactivate(world, this);
    }
}

/// `-[TileDrainLogic canPlugDrain]` @ 0x100117e8c
pub fn can_plug_drain(world: &mut World, this: Entity) -> bool {
    !world.obj::<TileDrainLogic>(this).pain_drain_active
}

/// `-[TileDrainLogic escapeFromDrain:]` @ 0x100117ea4
///
/// The caught pygmy got away (it died): it leaves the drain's animation.
pub fn escape_from_drain(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let caught_pygmy = world.obj::<TileDrainLogic>(this).pain_drain_pygmy_logic;
    if caught_pygmy != Some(pygmy_logic) {
        return;
    }

    let own_animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, own_animation, PYGMY_CHANNEL);
    world.obj_mut::<TileDrainLogic>(this).pain_drain_pygmy_logic = None;
}

/// `-[TileDrainLogic plugDrain:]` @ 0x100117efc
///
/// The plug fell back in: a caught pygmy is let go.
pub fn plug_drain(world: &mut World, this: Entity, _tile_plug: Entity) {
    {
        let mut drain = world.obj_mut::<TileDrainLogic>(this);
        drain.lifting = false;
        drain.unplugged = false;
    }

    let pygmy = world.obj::<TileDrainLogic>(this).pain_drain_pygmy_logic;
    if let Some(pygmy) = pygmy {
        let own_animation = logic::animation(world, this);
        animation::clear_tweenable_channel_id_now(world, own_animation, PYGMY_CHANNEL);
        crate::pygmy::escape_from_drain(world, pygmy, this);
        world.obj_mut::<TileDrainLogic>(this).pain_drain_pygmy_logic = None;
    }
    logic::set_behavior(world, this, Some("TileDrainPlugDrain"));
}

/// `-[TileDrainLogic unplugTilePlug:::]` @ 0x100117f98
///
/// The lift animation finished: the plug comes out on the lifting finger.
pub fn seq_unplug_tile_plug(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let lifting = world.obj::<TileDrainLogic>(this).lifting;
    if !lifting {
        return ItemResult::Done;
    }

    world.obj_mut::<TileDrainLogic>(this).unplugged = true;
    let tile_plug = listener_tile_plug_logic(world, this);
    let active_touch = world.obj::<TileDrainLogic>(this).active_touch;

    // `mbLifting` is only ever set while a touch is active.
    if let (Some(tile_plug), Some(active_touch)) = (tile_plug, active_touch) {
        tile_plug_logic::unplug(world, tile_plug, this, active_touch);
    }
    ItemResult::Done
}

/// `-[TileDrainLogic catchPygmyComplete:::]` @ 0x100118018
///
/// The pygmy went down the drain: Pain Drain starts, and the plug goes back in.
pub fn seq_catch_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<TileDrainLogic>(this).pain_drain_active = true;
    create_platforms(world, this);
    world.obj_mut::<TileDrainLogic>(this).platform_y_pos_vel = START_PLATFORM_Y_POS_VEL;

    let own_animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, own_animation, PYGMY_CHANNEL);

    let (listener, pygmy) = {
        let drain = world.obj::<TileDrainLogic>(this);
        (drain.tile_drain_logic_listener, drain.pain_drain_pygmy_logic)
    };
    // PORT: the original also sends `onPainDrainStart:pygmyLogic:` with a nil pygmy (one that
    // died during the fall into the drain); the controller's port takes a non-nil pygmy.
    if let (Some(listener), Some(pygmy)) = (listener, pygmy) {
        crate::controller::on_pain_drain_start(world, listener, this, pygmy);
    }

    let pygmy = world.obj::<TileDrainLogic>(this).pain_drain_pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::start_pain_drain(world, pygmy);
    }

    let tile_plug = listener_tile_plug_logic(world, this);
    if let Some(tile_plug) = tile_plug {
        tile_plug_logic::plug(world, tile_plug, this);
    }

    let underwater_marker = pg_engine::library::display_marker(world, "UnderwaterDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(underwater_marker) = underwater_marker {
        display::insert_before(world, underwater_marker, Some(display_object));
    }

    {
        let mut drain = world.obj_mut::<TileDrainLogic>(this);
        drain.lifting = false;
        drain.unplugged = false;
    }
    logic::set_behavior(world, this, Some("TileDrainTransitionIn"));
    ItemResult::Done
}

/// `-[TileDrainLogic catchPygmy:::]` @ 0x100118184
///
/// Each frame while the drain is open: sucks in the first pygmy over it (`|dx| < 50`,
/// `0 < dy < 130`) that lets itself be caught. Never completes; the pygmy's fall replaces the
/// behaviour.
pub fn seq_catch_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    let caught_pygmy = world.obj::<TileDrainLogic>(this).pain_drain_pygmy_logic;
    if caught_pygmy.is_some() {
        return ItemResult::Pending;
    }

    let listener = world.obj::<TileDrainLogic>(this).tile_drain_logic_listener;
    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let pos = display::object(world, display_object).pos;

        let x_distance = (pygmy_pos.x - pos.x).abs();
        let y_offset = pygmy_pos.y - pos.y;
        let over_drain = x_distance < CATCH_X_DISTANCE && y_offset > 0.0 && y_offset < CATCH_Y_DISTANCE;
        if !over_drain {
            continue;
        }

        let sucked_in = crate::pygmy::sucked_into_drain(world, pygmy, this);
        if sucked_in {
            world.obj_mut::<TileDrainLogic>(this).pain_drain_pygmy_logic = Some(pygmy);
            break;
        }
    }
    ItemResult::Pending
}

/// `-[TileDrainLogic pygmyFallIntoDrainComplete:]` @ 0x10011838c
///
/// The sucked-in pygmy reached the drain: the drain's animation takes it over on channel 1.
pub fn pygmy_fall_into_drain_complete(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let own_animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    animation::set_tweenable(world, own_animation, Some(pygmy_display_object), PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("TileDrainPygmyFallIntoDrain"));
}

/// `-[TileDrainLogic die:::]` @ 0x100118400
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[TileDrainLogic touchBeganInside:allTouches:]` @ 0x1001184c0
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<TileDrainLogic>(this).active_touch = Some(touch);

    let unplugged = world.obj::<TileDrainLogic>(this).unplugged;
    if unplugged {
        return;
    }

    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let touch_island_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_pos),
        None => Vec4::ZERO,
    };
    world.obj_mut::<TileDrainLogic>(this).touch_island_start_pos = touch_island_pos;
    logic::set_behavior(world, this, Some("TileDrainLiftChain"));
}

/// `-[TileDrainLogic touchBeganOutside:allTouches:]` @ 0x1001185ac
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[TileDrainLogic touchMovedEntered:allTouches:]` @ 0x1001185b0
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[TileDrainLogic touchMovedExited:allTouches:]` @ 0x1001185b4
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[TileDrainLogic touchMovedInside:allTouches:]` @ 0x1001185b8
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[TileDrainLogic touchMovedOutside:allTouches:]` @ 0x1001185c8
///
/// Dragging the drain's touch above y = 150 lifts the plug.
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let (active_touch, lifting, unplugged) = {
        let drain = world.obj::<TileDrainLogic>(this);
        (drain.active_touch, drain.lifting, drain.unplugged)
    };
    if active_touch != Some(touch) || lifting || unplugged {
        return;
    }

    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let touch_island_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_pos),
        None => Vec4::ZERO,
    };
    if touch_island_pos.y > LIFT_Y_POS {
        world.obj_mut::<TileDrainLogic>(this).lifting = true;
        logic::set_behavior(world, this, Some("TileDrainUnplug"));
    }
}

/// `-[TileDrainLogic touchEndedInside:allTouches:]` @ 0x1001186b8
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<TileDrainLogic>(this).active_touch;
    if active_touch != Some(touch) {
        return;
    }

    let unplugged = {
        let mut drain = world.obj_mut::<TileDrainLogic>(this);
        drain.lifting = false;
        drain.active_touch = None;
        drain.unplugged
    };
    if !unplugged {
        logic::set_behavior(world, this, Some("TileDrainDropChain"));
    }
}

/// `-[TileDrainLogic touchEndedOutside:allTouches:]` @ 0x10011870c
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
