//! `KonkeyDongLadderLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::konkeydong::kong_statue_logic;

/// `KonkeyDongLadderLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct KonkeyDongLadderLogic {
    /// `mKonkeyDongLadderLogicListener`: `@"<KonkeyDongLadderLogicListener>"`
    pub konkey_dong_ladder_logic_listener: Option<Entity>,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mLadderHeight`: `f`
    pub ladder_height: f32,

    /// `mbForceBoulderDrop`: `B`
    pub force_boulder_drop: bool,

    /// `mbDoNotPass`: `B`
    pub do_not_pass: bool,
}

pub const KONKEY_DONG_LADDER_LOGIC: Class = Class {
    name: "KonkeyDongLadderLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The ladder's `initWithDisplayObject:` behaviour (`@"KonkeyDongLadder1Init"`).
const LADDER_INIT_BEHAVIOR: &str = "KonkeyDongLadder1Init";

/// Sequence-item handlers implemented by `KonkeyDongLadderLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "setLadderParams" => seq_set_ladder_params(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[KonkeyDongLadderLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&KONKEY_DONG_LADDER_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[KonkeyDongLadderLogic initWithDisplayObject:]` @ 0x10000cd54
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(KonkeyDongLadderLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::set_behavior(world, this, Some(LADDER_INIT_BEHAVIOR));
    pg_engine::logic::stop_game_frame(world, this);
}

/// `-[KonkeyDongLadderLogic setKonkeyDongLadderLogicListener:]` @ 0x10000cdd4
pub fn set_konkey_dong_ladder_logic_listener(world: &mut World, this: Entity, konkey_dong_ladder_logic_listener: Entity) {
    world.obj_mut::<KonkeyDongLadderLogic>(this).konkey_dong_ladder_logic_listener = Some(konkey_dong_ladder_logic_listener);
}

/// `-[KonkeyDongLadderLogic setForceBoulderDrop:]` @ 0x10000ce30
pub fn set_force_boulder_drop(world: &mut World, this: Entity, force_boulder_drop: bool) {
    world.obj_mut::<KonkeyDongLadderLogic>(this).force_boulder_drop = force_boulder_drop;
}

/// `-[KonkeyDongLadderLogic forceBoulderDrop]` @ 0x10000ce40
pub fn force_boulder_drop(world: &mut World, this: Entity) -> bool {
    world.obj::<KonkeyDongLadderLogic>(this).force_boulder_drop
}

/// `-[KonkeyDongLadderLogic setDoNotPass:]` @ 0x10000ce50
pub fn set_do_not_pass(world: &mut World, this: Entity, do_not_pass: bool) {
    world.obj_mut::<KonkeyDongLadderLogic>(this).do_not_pass = do_not_pass;
}

/// `-[KonkeyDongLadderLogic doNotPass]` @ 0x10000ce60
pub fn do_not_pass(world: &mut World, this: Entity) -> bool {
    world.obj::<KonkeyDongLadderLogic>(this).do_not_pass
}

/// `-[KonkeyDongLadderLogic ladderHeight]` @ 0x10000ce70
pub fn ladder_height(world: &mut World, this: Entity) -> f32 {
    world.obj::<KonkeyDongLadderLogic>(this).ladder_height
}

/// `-[KonkeyDongLadderLogic activate]` @ 0x10000ce80
pub fn activate(_world: &mut World, _this: Entity) {}

/// `-[KonkeyDongLadderLogic deactivate]` @ 0x10000ce84
pub fn deactivate(_world: &mut World, _this: Entity) {}

/// `-[KonkeyDongLadderLogic checkTopCollision:lastPos:]` @ 0x10000ce88
///
/// Whether `logic` stands at the top of the ladder: within 10 of its x and within 20 of the top
/// rung (the level's y + the ladder's y + its height).
pub fn check_top_collision(world: &mut World, this: Entity, check_top_collision: Entity, _last_pos: Vec4) -> bool {
    let (ladder_pos, level_pos, logic_pos) = collision_positions(world, this, check_top_collision);
    let ladder_height = world.obj::<KonkeyDongLadderLogic>(this).ladder_height;

    let x_distance = (ladder_pos.x - logic_pos.x).abs();
    let top_y_pos = level_pos.y + ladder_pos.y + ladder_height;
    let y_distance = (top_y_pos - logic_pos.y).abs();
    x_distance < 10.0 && y_distance < 20.0
}

/// `-[KonkeyDongLadderLogic checkBottomCollision:lastPos:]` @ 0x10000cf6c
///
/// Whether `logic` stands at the foot of the ladder: within 10 of its x and within 20 of the
/// level's y + the ladder's y.
pub fn check_bottom_collision(world: &mut World, this: Entity, check_bottom_collision: Entity, _last_pos: Vec4) -> bool {
    let (ladder_pos, level_pos, logic_pos) = collision_positions(world, this, check_bottom_collision);

    let x_distance = (ladder_pos.x - logic_pos.x).abs();
    let bottom_y_pos = level_pos.y + ladder_pos.y;
    let y_distance = (bottom_y_pos - logic_pos.y).abs();
    x_distance < 10.0 && y_distance < 20.0
}

/// The positions both ladder checks read: the ladder's, the active level group's
/// (`[mKonkeyDongLadderLogicListener konkeyDongLevelDisplayObject]`) and the checked logic's.
fn collision_positions(world: &mut World, this: Entity, logic: Entity) -> (Vec4, Vec4, Vec4) {
    let logic_display_object = pg_engine::logic::display_object(world, logic);
    let listener = world.obj::<KonkeyDongLadderLogic>(this).konkey_dong_ladder_logic_listener;
    let level_display_object = listener.and_then(|listener| kong_statue_logic::konkey_dong_level_display_object(world, listener));
    let level_display_object = level_display_object.expect("KonkeyDongLadderLogic: no active level group (the original dereferences [nil pos])");

    let level_pos = pg_engine::display::object(world, level_display_object).pos;
    let logic_pos = pg_engine::display::object(world, logic_display_object).pos;
    let ladder_display_object = pg_engine::logic::display_object(world, this);
    let ladder_pos = pg_engine::display::object(world, ladder_display_object).pos;
    (ladder_pos, level_pos, logic_pos)
}

/// `-[KonkeyDongLadderLogic onGameFrame:]` @ 0x10000d040
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let state = pg_engine::logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let pos = pg_engine::display::object(world, display_object).pos;
    world.obj_mut::<KonkeyDongLadderLogic>(this).last_pos = pos;
}

/// `-[KonkeyDongLadderLogic setLadderParams:::]` @ 0x10000d104
pub fn seq_set_ladder_params(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let height = call.float("height");
    world.obj_mut::<KonkeyDongLadderLogic>(this).ladder_height = height;
    ItemResult::Done
}
