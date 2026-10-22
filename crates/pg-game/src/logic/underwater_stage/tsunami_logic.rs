//! `TsunamiLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::underwater_baitmaster_laser::{anchor_logic, chum_bucket_logic};
use crate::logic::underwater_paindrain::{tile_drain_logic, tile_plug_logic};
use crate::logic::underwater_stage::{spear_statue_logic, underwater_scenery_logic, underwater_temple_logic};
use crate::logic_table;
use pg_engine::{animation, display, library, logic};

/// `TsunamiLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct TsunamiLogic {
    /// `mTsunamiLogicListener`: `@"<TsunamiLogicListener>"`
    pub tsunami_logic_listener: Option<Entity>,

    /// `mbFrontLayer`: `B`
    pub front_layer: bool,
}

pub const TSUNAMI_LOGIC: Class = Class {
    name: "TsunamiLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Island location state of the underwater ruins.
const LOCATION_UNDERWATER: i32 = 2;

/// Island location state of Ape island.
const LOCATION_APE_ISLAND: i32 = 5;

/// Island location state of the underwater coral reef.
const LOCATION_UNDERWATER_CORAL_REEF: i32 = 9;

/// Island location state of the first apocalypse temple (the time temple).
const LOCATION_TIME_TEMPLE: i32 = 11;

/// Island location state of the last apocalypse temple (the Xenu temple).
const LOCATION_XENU_TEMPLE: i32 = 17;

/// Animation channels the front tsunami carries the ruins' props on.
const SPEAR_STATUE_CHANNEL: usize = 1;
const ANCHOR_CHANNEL: usize = 2;
const CHUM_BUCKET_CHANNEL: usize = 3;
const TILE_DRAIN_CHANNEL: usize = 4;

/// Animation channels `setFrontLayer:` binds the coral reef foreground and the temple to.
const CORAL_REEF_FOREGROUND_CHANNEL: usize = 5;
const UNDERWATER_TEMPLE_CHANNEL: usize = 6;

/// Sequence-item handlers implemented by `TsunamiLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "activateUnderwaterTemple" => seq_activate_underwater_temple(world, this, call),
        "tsunamiActivateComplete" => seq_tsunami_activate_complete(world, this, call),
        "tsunamiDeactivateComplete" => seq_tsunami_deactivate_complete(world, this, call),
        "setAnimationBasedOnUnderwaterLocation" => seq_set_animation_based_on_underwater_location(world, this, call),
        "setAnimationBasedOnLastUnderwaterLocation" => seq_set_animation_based_on_last_underwater_location(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `TsunamiLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animHideIslandAssets" => anim_hide_island_assets(world, this),
        "animShowIslandAssets" => anim_show_island_assets(world, this),
        "animShowScenery" => anim_show_scenery(world, this),
        "animHideScenery" => anim_hide_scenery(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[TsunamiLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&TSUNAMI_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[TsunamiLogic initWithDisplayObject:]` @ 0x10000bd30
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(TsunamiLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
}

/// `-[TsunamiLogic setTsunamiLogicListener:]` @ 0x10000bdb4
pub fn set_tsunami_logic_listener(world: &mut World, this: Entity, tsunami_logic_listener: Entity) {
    world.obj_mut::<TsunamiLogic>(this).tsunami_logic_listener = Some(tsunami_logic_listener);
}

/// `-[TsunamiLogic setFrontLayer:]` @ 0x10000be10
///
/// The front layer carries the coral reef foreground (channel 5) and the temple (channel 6).
pub fn set_front_layer(world: &mut World, this: Entity, front_layer: bool) {
    world.obj_mut::<TsunamiLogic>(this).front_layer = front_layer;
    if !front_layer {
        return;
    }

    let own_animation = logic::animation(world, this);

    let foreground = library::logic(world, logic_table::UNDERWATER_CORAL_REEF_FOREGROUND);
    let foreground_display_object = logic::display_object(world, foreground);
    animation::set_tweenable(world, own_animation, Some(foreground_display_object), CORAL_REEF_FOREGROUND_CHANNEL);

    let temple = library::logic(world, logic_table::UNDERWATER_TEMPLE);
    let temple_display_object = logic::display_object(world, temple);
    animation::set_tweenable(world, own_animation, Some(temple_display_object), UNDERWATER_TEMPLE_CHANNEL);
}

/// `-[TsunamiLogic activate:]` @ 0x10000bf18
///
/// The wave rises: from the left (`activate` NO) or the right (YES), or without water when
/// leaving a location that has no sea.
pub fn activate(world: &mut World, this: Entity, activate: bool) {
    let front_layer = world.obj::<TsunamiLogic>(this).front_layer;
    let own_display_object = logic::display_object(world, this);

    let behavior = if !front_layer {
        let marker = library::display_marker(world, "CloudBehindDisplayMarker");
        if let Some(marker) = marker {
            display::insert_after(world, marker, Some(own_display_object));
        }

        let last_island_location_state = listener_last_island_location_state(world, this);
        if has_no_sea(last_island_location_state) {
            "BehindTsunamiStartNoWater"
        } else if !activate {
            "BehindTsunamiStartLeft"
        } else {
            "BehindTsunamiStartRight"
        }
    } else {
        let marker = library::display_marker(world, "TsunamiDisplayMarker");
        if let Some(marker) = marker {
            display::insert_before(world, marker, Some(own_display_object));
        }

        let last_island_location_state = listener_last_island_location_state(world, this);
        if has_no_sea(last_island_location_state) {
            let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener;
            if let Some(listener) = listener {
                crate::controller::on_tsunami_hide_island_assets(world, listener, this);
            }
            "FrontTsunamiStartNoWater"
        } else if !activate {
            "FrontTsunamiStartLeft"
        } else {
            "FrontTsunamiStartRight"
        }
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[TsunamiLogic deactivate]` @ 0x10000c1cc
///
/// The front wave recedes, taking the spear, the ruins' props and the temple buttons with it.
pub fn deactivate(world: &mut World, this: Entity) {
    if !world.obj::<TsunamiLogic>(this).front_layer {
        return;
    }

    let last_island_location_state = listener_last_island_location_state(world, this);
    let behavior = if has_no_sea(last_island_location_state) {
        "FrontTsunamiEndNoWater"
    } else {
        "FrontTsunamiEnd"
    };
    logic::set_behavior(world, this, Some(behavior));

    let spear_statue = listener_spear_statue_logic(world, this);
    if let Some(spear_statue) = spear_statue {
        spear_statue_logic::hide_spear(world, spear_statue);
    }

    let anchor = listener_anchor_logic(world, this);
    if let Some(anchor) = anchor {
        anchor_logic::transition_out(world, anchor);
    }

    let tile_drain = listener_tile_drain_logic(world, this);
    if let Some(tile_drain) = tile_drain {
        tile_drain_logic::transition_out(world, tile_drain);
    }

    let tile_plug = listener_tile_plug_logic(world, this);
    if let Some(tile_plug) = tile_plug {
        tile_plug_logic::exit_island(world, tile_plug);
    }

    let chum_bucket = listener_chum_bucket_logic(world, this);
    if let Some(chum_bucket) = chum_bucket {
        chum_bucket_logic::transition_out(world, chum_bucket);
    }

    let temple = library::logic(world, logic_table::UNDERWATER_TEMPLE);
    underwater_temple_logic::deactivate(world, temple);
}

/// `-[TsunamiLogic moveBetweenUnderwaterLocations]` @ 0x10000c3ec
///
/// Swapping the ruins and the coral reef: arriving at the ruins brings their props back in.
pub fn move_between_underwater_locations(world: &mut World, this: Entity) {
    let island_location_state = listener_island_location_state(world, this);
    let spear_statue = listener_spear_statue_logic(world, this);

    if island_location_state == LOCATION_UNDERWATER {
        bring_in_spear_statue(world, this, spear_statue);
        if let Some(spear_statue) = spear_statue {
            spear_statue_logic::show_spear(world, spear_statue);
        }
        bring_in_ruins_props(world, this);

        let temple = library::logic(world, logic_table::UNDERWATER_TEMPLE);
        underwater_temple_logic::deactivate(world, temple);
    } else if let Some(spear_statue) = spear_statue {
        spear_statue_logic::transition_out(world, spear_statue);
    }

    logic::set_behavior(world, this, Some("FrontTsunamiChangeUnderwaterLocation"));
}

/// `-[TsunamiLogic animHideIslandAssets]` @ 0x10000c618
pub fn anim_hide_island_assets(world: &mut World, this: Entity) {
    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_tsunami_hide_island_assets(world, listener, this);
    }
}

/// `-[TsunamiLogic animShowIslandAssets]` @ 0x10000c63c
pub fn anim_show_island_assets(world: &mut World, this: Entity) {
    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_tsunami_show_island_assets(world, listener, this);
    }
}

/// `-[TsunamiLogic animShowScenery]` @ 0x10000c660
pub fn anim_show_scenery(world: &mut World, this: Entity) {
    let scenery_logics = listener_underwater_scenery_logic_array(world, this);
    for scenery in scenery_logics {
        underwater_scenery_logic::transition_in(world, scenery);
    }

    let island_location_state = listener_island_location_state(world, this);
    if island_location_state != LOCATION_UNDERWATER {
        return;
    }

    let spear_statue = listener_spear_statue_logic(world, this);
    bring_in_spear_statue(world, this, spear_statue);
    bring_in_ruins_props(world, this);
}

/// `-[TsunamiLogic animHideScenery]` @ 0x10000c8f8
pub fn anim_hide_scenery(world: &mut World, this: Entity) {
    let scenery_logics = listener_underwater_scenery_logic_array(world, this);
    for scenery in scenery_logics {
        underwater_scenery_logic::transition_out(world, scenery);
    }

    let last_island_location_state = listener_last_island_location_state(world, this);
    if last_island_location_state != LOCATION_UNDERWATER {
        return;
    }

    let spear_statue = listener_spear_statue_logic(world, this);
    if let Some(spear_statue) = spear_statue {
        spear_statue_logic::transition_out(world, spear_statue);
    }
}

/// `-[TsunamiLogic onGameFrame:]` @ 0x10000cacc
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[TsunamiLogic activateUnderwaterTemple:::]` @ 0x10000cad0
pub fn seq_activate_underwater_temple(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let island_location_state = listener_island_location_state(world, this);
    if island_location_state == LOCATION_UNDERWATER_CORAL_REEF {
        let temple = library::logic(world, logic_table::UNDERWATER_TEMPLE);
        underwater_temple_logic::activate(world, temple);
    }
    ItemResult::Done
}

/// `-[TsunamiLogic tsunamiActivateComplete:::]` @ 0x10000cb44
pub fn seq_tsunami_activate_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let spear_statue = listener_spear_statue_logic(world, this);
    if let Some(spear_statue) = spear_statue {
        spear_statue_logic::show_spear(world, spear_statue);
    }

    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_tsunami_activate_complete(world, listener, this);
    }
    ItemResult::Done
}

/// `-[TsunamiLogic tsunamiDeactivateComplete:::]` @ 0x10000cbac
///
/// Releases the statue, the anchor and the drain; the chum bucket stays bound to channel 3.
pub fn seq_tsunami_deactivate_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let own_animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, own_animation, SPEAR_STATUE_CHANNEL);
    animation::clear_tweenable_channel_id_now(world, own_animation, ANCHOR_CHANNEL);
    animation::clear_tweenable_channel_id_now(world, own_animation, TILE_DRAIN_CHANNEL);

    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_tsunami_deactivate_complete(world, listener, this);
    }
    ItemResult::Done
}

/// `-[TsunamiLogic setAnimationBasedOnUnderwaterLocation:::]` @ 0x10000cc3c
pub fn seq_set_animation_based_on_underwater_location(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let island_location_state = listener_island_location_state(world, this);
    play_location_animation(world, this, call, island_location_state);
    ItemResult::Done
}

/// `-[TsunamiLogic setAnimationBasedOnLastUnderwaterLocation:::]` @ 0x10000ccc8
pub fn seq_set_animation_based_on_last_underwater_location(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let last_island_location_state = listener_last_island_location_state(world, this);
    play_location_animation(world, this, call, last_island_location_state);
    ItemResult::Done
}

/// The animation of the two `setAnimationBased…` items: `[0] coralReef` at the coral reef when
/// present, otherwise `[1] ruins`, played on the own animation player.
fn play_location_animation(world: &mut World, this: Entity, call: &SequenceCall, location_state: i32) {
    let mut animation_id = call.param("ruins").map(str::to_owned);
    let coral_reef_animation_id = call.param("coralReef");
    if location_state == LOCATION_UNDERWATER_CORAL_REEF {
        if let Some(coral_reef_animation_id) = coral_reef_animation_id {
            animation_id = Some(coral_reef_animation_id.to_owned());
        }
    }

    let own_animation = logic::animation(world, this);
    animation::play_animation_id(world, own_animation, animation_id.as_deref());
}

/// The statue part shared by `moveBetweenUnderwaterLocations` and `animShowScenery`: bound to
/// channel 1, then `transitionIn` (both are messages to nil without a statue).
fn bring_in_spear_statue(world: &mut World, this: Entity, spear_statue: Option<Entity>) {
    let own_animation = logic::animation(world, this);
    let statue_display_object = spear_statue.map(|statue| logic::display_object(world, statue));
    animation::set_tweenable(world, own_animation, statue_display_object, SPEAR_STATUE_CHANNEL);

    if let Some(spear_statue) = spear_statue {
        spear_statue_logic::transition_in(world, spear_statue);
    }
}

/// The ruins' props that follow the statue in `moveBetweenUnderwaterLocations` and
/// `animShowScenery`: anchor (channel 2), drain (channel 4) and chum bucket (channel 3), each
/// bound and sent `transitionIn`.
fn bring_in_ruins_props(world: &mut World, this: Entity) {
    let own_animation = logic::animation(world, this);

    let anchor = listener_anchor_logic(world, this);
    let anchor_display_object = anchor.map(|anchor| logic::display_object(world, anchor));
    animation::set_tweenable(world, own_animation, anchor_display_object, ANCHOR_CHANNEL);
    if let Some(anchor) = anchor {
        anchor_logic::transition_in(world, anchor);
    }

    let tile_drain = listener_tile_drain_logic(world, this);
    let tile_drain_display_object = tile_drain.map(|tile_drain| logic::display_object(world, tile_drain));
    animation::set_tweenable(world, own_animation, tile_drain_display_object, TILE_DRAIN_CHANNEL);
    if let Some(tile_drain) = tile_drain {
        tile_drain_logic::transition_in(world, tile_drain);
    }

    let chum_bucket = listener_chum_bucket_logic(world, this);
    let chum_bucket_display_object = chum_bucket.map(|chum_bucket| logic::display_object(world, chum_bucket));
    animation::set_tweenable(world, own_animation, chum_bucket_display_object, CHUM_BUCKET_CHANNEL);
    if let Some(chum_bucket) = chum_bucket {
        chum_bucket_logic::transition_in(world, chum_bucket);
    }
}

/// Ape island (5) and the apocalypse temples (11–17) have no sea: the wave comes without water.
fn has_no_sea(island_location_state: i32) -> bool {
    let is_temple = (LOCATION_TIME_TEMPLE..=LOCATION_XENU_TEMPLE).contains(&island_location_state);
    island_location_state == LOCATION_APE_ISLAND || is_temple
}

/// `[mTsunamiLogicListener islandLocationState]` (0 from a nil listener).
fn listener_island_location_state(world: &mut World, this: Entity) -> i32 {
    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener;
    match listener {
        Some(listener) => crate::controller::island_location_state(world, listener),
        None => 0,
    }
}

/// `[mTsunamiLogicListener lastIslandLocationState]` (0 from a nil listener).
fn listener_last_island_location_state(world: &mut World, this: Entity) -> i32 {
    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener;
    match listener {
        Some(listener) => crate::controller::last_island_location_state(world, listener),
        None => 0,
    }
}

/// `[mTsunamiLogicListener spearStatueLogic:self]`.
fn listener_spear_statue_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener?;
    crate::controller::spear_statue_logic(world, listener, this)
}

/// `[mTsunamiLogicListener anchorLogic:self]`.
fn listener_anchor_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener?;
    crate::controller::anchor_logic(world, listener, this)
}

/// `[mTsunamiLogicListener tileDrainLogic:self]`.
fn listener_tile_drain_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener?;
    crate::controller::tile_drain_logic(world, listener, this)
}

/// `[mTsunamiLogicListener tilePlugLogic:self]`.
fn listener_tile_plug_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener?;
    crate::controller::tile_plug_logic(world, listener, this)
}

/// `[mTsunamiLogicListener chumBucketLogic:self]`.
fn listener_chum_bucket_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener?;
    crate::controller::chum_bucket_logic(world, listener, this)
}

/// `[mTsunamiLogicListener underwaterSceneryLogicArray:self]` (empty from a nil listener).
fn listener_underwater_scenery_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = world.obj::<TsunamiLogic>(this).tsunami_logic_listener;
    match listener {
        Some(listener) => crate::controller::underwater_scenery_logic_array(world, listener, this),
        None => Vec::new(),
    }
}
