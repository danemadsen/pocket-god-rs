//! `NoPeeTreasureLogic` (superclass `NoPeeObjectLogic`).

use crate::prelude::*;

use crate::logic::nopee::no_pee_logic;
use crate::logic::nopee::no_pee_object_logic::{self, NoPeeObjectLogic};
use pg_engine::logic;

/// `NoPeeTreasureLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeTreasureLogic {
    /// `mNoPeeTreasureLogicListener`: `@"<NoPeeTreasureLogicListener>"`
    pub no_pee_treasure_logic_listener: Option<Entity>,

    /// `mNoPeeTreasureType`: `i`
    pub no_pee_treasure_type: i32,

    /// `mbTreasureCollected`: `B`
    pub treasure_collected: bool,

    /// `mTreasureLevel`: `i`
    pub treasure_level: i32,
}

pub const NO_PEE_TREASURE_LOGIC: Class = Class {
    name: "NoPeeTreasureLogic",
    superclass: Some(&crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC
};

/// The pygmy collects the treasure within this distance.
const COLLECT_RADIUS: f32 = 50.0;

/// The treasure respawns at a y in this range.
const SPAWN_MIN_Y_POS: f32 = 100.0;
const SPAWN_MAX_Y_POS: f32 = 800.0;

/// Sequence-item handlers implemented by `NoPeeTreasureLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "setAnimationBasedOnLevel" => seq_set_animation_based_on_level(world, this, call),
        "onNoPeeTreasureCollectComplete" => seq_on_no_pee_treasure_collect_complete(world, this, call),
        _ => return (crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[NoPeeTreasureLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_TREASURE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeTreasureLogic initWithDisplayObject:]` @ 0x10008ddb8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeTreasureLogic::default());
    no_pee_object_logic::init_with_display_object(world, this, display_object);
}

/// `-[NoPeeTreasureLogic setNoPeeTreasureLogicListener:]` @ 0x10008ddf4
pub fn set_no_pee_treasure_logic_listener(world: &mut World, this: Entity, no_pee_treasure_logic_listener: Entity) {
    world.obj_mut::<NoPeeTreasureLogic>(this).no_pee_treasure_logic_listener = Some(no_pee_treasure_logic_listener);
    no_pee_object_logic::set_no_pee_object_logic_listener(world, this, no_pee_treasure_logic_listener);
}

/// `-[NoPeeTreasureLogic setNoPeeTreasureType:]` @ 0x10008de64
pub fn set_no_pee_treasure_type(world: &mut World, this: Entity, no_pee_treasure_type: i32) {
    world.obj_mut::<NoPeeTreasureLogic>(this).no_pee_treasure_type = no_pee_treasure_type;
}

/// `-[NoPeeTreasureLogic activate]` @ 0x10008de74
pub fn activate(world: &mut World, this: Entity) {
    {
        let mut treasure = world.obj_mut::<NoPeeTreasureLogic>(this);
        treasure.treasure_level = 0;
        treasure.treasure_collected = false;
    }
    logic::set_behavior(world, this, Some("NoPeeTreasureInit"));

    no_pee_object_logic::activate(world, this);
    no_pee_object_logic::random_spawn_position_no_overlap_with_min_y_pos(world, this, SPAWN_MIN_Y_POS, SPAWN_MAX_Y_POS);
}

/// `-[NoPeeTreasureLogic onGameFrame:]` @ 0x10008df10
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let object_listener = world.obj::<NoPeeObjectLogic>(this).no_pee_object_logic_listener;
    let pygmy = match object_listener {
        Some(object_listener) => no_pee_logic::no_pee_pygmy_logic(world, object_listener),
        None => None,
    };

    if let Some(pygmy) = pygmy {
        let collected = world.obj::<NoPeeTreasureLogic>(this).treasure_collected;
        if !collected && no_pee_object_logic::check_collision(world, this, pygmy, COLLECT_RADIUS) {
            world.obj_mut::<NoPeeTreasureLogic>(this).treasure_collected = true;

            let listener = world.obj::<NoPeeTreasureLogic>(this).no_pee_treasure_logic_listener;
            if let Some(listener) = listener {
                no_pee_logic::on_collect_no_pee_treasure(world, listener);
            }
            logic::set_behavior(world, this, Some("NoPeeTreasureCollect"));
        }
    }

    no_pee_object_logic::on_game_frame(world, this, dt);
}

/// `-[NoPeeTreasureLogic shouldHideNoPeeMapItem]` @ 0x10008e004
pub fn should_hide_no_pee_map_item(world: &mut World, this: Entity) -> bool {
    world.obj::<NoPeeTreasureLogic>(this).treasure_collected
}

/// `-[NoPeeTreasureLogic setAnimationBasedOnLevel:::]` @ 0x10008e014
///
/// Plays entry `mTreasureLevel` of the comma-separated `list`, first clamping the level to the
/// last entry.
pub fn seq_set_animation_based_on_level(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let animation_ids = call.list("list");

    // `count - 1 < (unsigned long)level`: an absent list (count 0) never clamps.
    let count = animation_ids.len() as u64;
    let last_index = count.wrapping_sub(1);
    let level = world.obj::<NoPeeTreasureLogic>(this).treasure_level;
    if last_index < level as i64 as u64 {
        let clamped_level = (count as i32).wrapping_sub(1);
        world.obj_mut::<NoPeeTreasureLogic>(this).treasure_level = clamped_level;
    }

    let level = world.obj::<NoPeeTreasureLogic>(this).treasure_level;
    let animation_id = animation_ids.get(level as usize).copied();
    let animation = logic::animation(world, this);
    pg_engine::animation::play_animation_id(world, animation, animation_id);
    ItemResult::Done
}

/// `-[NoPeeTreasureLogic onNoPeeTreasureCollectComplete:::]` @ 0x10008e0e8
pub fn seq_on_no_pee_treasure_collect_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut treasure = world.obj_mut::<NoPeeTreasureLogic>(this);
        treasure.treasure_level = treasure.treasure_level + 1;
        treasure.treasure_collected = false;
    }
    no_pee_object_logic::random_spawn_position_no_overlap_with_min_y_pos(world, this, SPAWN_MIN_Y_POS, SPAWN_MAX_Y_POS);
    logic::set_behavior(world, this, Some("NoPeeTreasureInit"));
    ItemResult::Done
}
