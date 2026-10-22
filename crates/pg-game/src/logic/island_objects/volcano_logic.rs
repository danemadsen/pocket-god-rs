//! `VolcanoLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::island_objects::lava_logic;

/// `VolcanoLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct VolcanoLogic {
    /// `mVolcanoLogicListener`: `@"<VolcanoLogicListener>"`
    pub volcano_logic_listener: Option<Entity>,

    /// `mLavaStallTime`: `f`
    pub lava_stall_time: f32,

    /// `mLavaCount`: `i`
    pub lava_count: i32,

    /// `mLavaLogicArray`: `@"NSMutableArray"`
    pub lava_logic_array: Vec<Entity>,
}

pub const VOLCANO_LOGIC: Class = Class {
    name: "VolcanoLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Sequence-item handlers implemented by `VolcanoLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "lavaRain" => seq_lava_rain(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[VolcanoLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&VOLCANO_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[VolcanoLogic initWithDisplayObject:]` @ 0x1000c6d98
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(VolcanoLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<VolcanoLogic>(this).lava_logic_array = Vec::with_capacity(5);
    pg_engine::logic::stop_game_frame(world, this);

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, own_display_object);

    pg_engine::logic::set_behavior(world, this, Some("VolcanoInit"));
}

/// `-[VolcanoLogic setVolcanoLogicListener:]` @ 0x1000c6e70
pub fn set_volcano_logic_listener(world: &mut World, this: Entity, volcano_logic_listener: Entity) {
    world.obj_mut::<VolcanoLogic>(this).volcano_logic_listener = Some(volcano_logic_listener);
}

/// `-[VolcanoLogic enterIsland:]` @ 0x1000c6ecc
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let display_object = pg_engine::logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "VolcanoDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[VolcanoLogic exitIsland]` @ 0x1000c6f34
pub fn exit_island(world: &mut World, this: Entity) {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, display_object);
}

/// `-[VolcanoLogic islandMinXPos]` @ 0x1000c6fd0
pub fn island_min_x_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<VolcanoLogic>(this).volcano_logic_listener;
    let Some(listener) = listener else {
        return 0.0;
    };
    crate::controller::island_min_x_pos(world, listener)
}

/// `-[VolcanoLogic islandMaxXPos]` @ 0x1000c6fec
pub fn island_max_x_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<VolcanoLogic>(this).volcano_logic_listener;
    let Some(listener) = listener else {
        return 0.0;
    };
    crate::controller::island_max_x_pos(world, listener)
}

/// `-[VolcanoLogic pygmyLogicArray:]` @ 0x1000c7008
///
/// Forwards to the listener with `self` as the argument (the caller's is ignored).
pub fn pygmy_logic_array(world: &mut World, this: Entity, _pygmy_logic_array: Entity) -> Vec<Entity> {
    let listener = world.obj::<VolcanoLogic>(this).volcano_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::pygmy_logic_array(world, listener, this)
}

/// `-[VolcanoLogic onLavaDie:]` @ 0x1000c702c
pub fn on_lava_die(world: &mut World, this: Entity, lava_die: Entity) {
    let lava_display_object = pg_engine::logic::display_object(world, lava_die);
    pg_engine::display::remove_from_display_group(world, lava_display_object);

    world.obj_mut::<VolcanoLogic>(this).lava_logic_array.retain(|lava| *lava != lava_die);
    pg_engine::logic::shutdown(world, lava_die);

    // The array held the blob's last owning reference besides the frame-listener list and its
    // animation player, both of which `shutdown` drops: the blob is deallocated here.
    pg_engine::logic::dealloc(world, lava_die);
}

/// `-[VolcanoLogic hit1]` @ 0x1000c70a0
pub fn hit1(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("VolcanoHit1"));
}

/// `-[VolcanoLogic hit2]` @ 0x1000c70b8
pub fn hit2(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("VolcanoHit2"));
}

/// `-[VolcanoLogic hit3]` @ 0x1000c70d0
pub fn hit3(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("VolcanoHit3"));
}

/// `-[VolcanoLogic erupt1]` @ 0x1000c70e8
pub fn erupt1(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("VolcanoErupt1"));
}

/// `-[VolcanoLogic erupt2]` @ 0x1000c7100
pub fn erupt2(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("VolcanoErupt2"));
}

/// `-[VolcanoLogic erupt3]` @ 0x1000c7118
pub fn erupt3(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("VolcanoErupt3"));
}

/// `-[VolcanoLogic lavaRain:::]` @ 0x1000c7130
///
/// Spawns `[1] minCount`…`[3] maxCount` lava blobs, one every `[0] minTime`…`[2] maxTime`
/// seconds (whole milliseconds), and completes with the last one.
pub fn seq_lava_rain(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_count = call.int("minCount");
        let max_count = call.int("maxCount");
        let count_range = 1u32.wrapping_sub(min_count as u32).wrapping_add(max_count as u32);
        let count_offset = world.resource_mut::<Random>().arc4random_mod(count_range);
        let lava_count = count_offset.wrapping_add(min_count as u32) as i32;

        let mut volcano = world.obj_mut::<VolcanoLogic>(this);
        volcano.lava_count = lava_count;
        volcano.lava_stall_time = 0.0;
    }

    let mut stall_time = world.obj::<VolcanoLogic>(this).lava_stall_time;
    if stall_time == 0.0 {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 1000.0, random);
        world.obj_mut::<VolcanoLogic>(this).lava_stall_time = stall_time;
    }

    let stall_elapsed = stall_time <= 0.0;
    if !stall_elapsed {
        let mut volcano = world.obj_mut::<VolcanoLogic>(this);
        volcano.lava_stall_time -= call.delta_time;
        return ItemResult::Pending;
    }

    let lava_count = {
        let mut volcano = world.obj_mut::<VolcanoLogic>(this);
        volcano.lava_stall_time = 0.0;
        volcano.lava_count = volcano.lava_count.wrapping_sub(1);
        volcano.lava_count
    };
    let result = if lava_count == 0 {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    let lava_model = pg_engine::display::multi_model::spawn(world, Some("NewLava"));
    let listener = world.obj::<VolcanoLogic>(this).volcano_logic_listener;
    let mut island_min_x = 0.0;
    let mut island_max_x = 0.0;
    if let Some(listener) = listener {
        island_min_x = crate::controller::island_min_x_pos(world, listener);
        island_max_x = crate::controller::island_max_x_pos(world, listener);
    }

    // The same draw as a stall time, in tenths of a unit across the island.
    let random = world.resource_mut::<Random>().arc4random();
    let lava_x = pg_engine::logic::random_scaled_time(island_min_x, island_max_x, 10.0, random);
    {
        let mut object = pg_engine::display::object_mut(world, lava_model);
        object.pos.x = lava_x;
        object.pos.y = 400.0;
        object.pos.z = -277.12;
    }
    let marker = pg_engine::library::display_marker(world, "LavaDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(lava_model));
    }

    let lava = lava_logic::spawn(world, lava_model);
    lava_logic::set_lava_logic_listener(world, lava, this);
    world.obj_mut::<VolcanoLogic>(this).lava_logic_array.push(lava);

    if let Some(listener) = listener {
        crate::controller::on_eruption_complete(world, listener, this);
    }
    result
}
