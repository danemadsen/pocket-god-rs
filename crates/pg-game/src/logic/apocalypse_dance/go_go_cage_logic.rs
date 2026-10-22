//! `GoGoCageLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::animation;
use pg_engine::logic;

/// `GoGoCageLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct GoGoCageLogic {
    /// `mGoGoCageLogicListener`: `@"<GoGoCageLogicListener>"`
    pub go_go_cage_logic_listener: Option<Entity>,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const GO_GO_CAGE_LOGIC: Class = Class {
    name: "GoGoCageLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The cage animation's channel that carries the dancing pygmy.
const PYGMY_CHANNEL: usize = 1;

/// Sequence-item handlers implemented by `GoGoCageLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "pygmyFallInsideComplete" => seq_pygmy_fall_inside_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[GoGoCageLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&GO_GO_CAGE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[GoGoCageLogic initWithDisplayObject:]` @ 0x10008f6c4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(GoGoCageLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);
}

/// `-[GoGoCageLogic setGoGoCageLogicListener:]` @ 0x10008f728
pub fn set_go_go_cage_logic_listener(world: &mut World, this: Entity, go_go_cage_logic_listener: Entity) {
    world.obj_mut::<GoGoCageLogic>(this).go_go_cage_logic_listener = Some(go_go_cage_logic_listener);
}

/// `-[GoGoCageLogic isAvailable]` @ 0x10008f784
pub fn is_available(world: &mut World, this: Entity) -> bool {
    world.obj::<GoGoCageLogic>(this).pygmy_logic.is_none()
}

/// `-[GoGoCageLogic pygmyFallIntoGoGoCage:]` @ 0x10008f79c
pub fn pygmy_fall_into_go_go_cage(world: &mut World, this: Entity, pygmy_fall_into_go_go_cage: Entity) {
    let pygmy = pygmy_fall_into_go_go_cage;
    if world.obj::<GoGoCageLogic>(this).pygmy_logic.is_some() {
        return;
    }
    world.obj_mut::<GoGoCageLogic>(this).pygmy_logic = Some(pygmy);

    let cage_animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy);
    animation::set_tweenable(world, cage_animation, Some(pygmy_display_object), PYGMY_CHANNEL);

    logic::set_behavior(world, this, Some("GoGoCagePygmyFallInside"));
}

/// `-[GoGoCageLogic enterIsland:]` @ 0x10008f830
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    logic::set_behavior(world, this, Some("GoGoCageEnterIsland"));
}

/// `-[GoGoCageLogic removePygmyFromGoGoCage:]` @ 0x10008f848
pub fn remove_pygmy_from_go_go_cage(world: &mut World, this: Entity, pygmy_from_go_go_cage: Entity) {
    let pygmy_logic = world.obj::<GoGoCageLogic>(this).pygmy_logic;
    if pygmy_logic != Some(pygmy_from_go_go_cage) {
        return;
    }
    world.obj_mut::<GoGoCageLogic>(this).pygmy_logic = None;

    let cage_animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, cage_animation, PYGMY_CHANNEL);

    logic::set_behavior(world, this, Some("GoGoCageEnterIsland"));
}

/// `-[GoGoCageLogic exitIsland:]` @ 0x10008f8c4
///
/// A pygmy still dancing in the cage is thrown out.
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    let pygmy_logic = world.obj::<GoGoCageLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy_logic {
        let cage_animation = logic::animation(world, this);
        animation::clear_tweenable_channel_id_now(world, cage_animation, PYGMY_CHANNEL);

        crate::pygmy::toss_from_go_go_cage(world, pygmy);
        world.obj_mut::<GoGoCageLogic>(this).pygmy_logic = None;
    }

    logic::set_behavior(world, this, Some("GoGoCageExitIsland"));
}

/// `-[GoGoCageLogic onGameFrame:]` @ 0x10008f9a8
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[GoGoCageLogic pygmyFallInsideComplete:::]` @ 0x10008f9ac
///
/// Only logs `GOGO CAGE FALL INSIDE COMPLETE:` (debug output, dropped).
pub fn seq_pygmy_fall_inside_complete(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}
