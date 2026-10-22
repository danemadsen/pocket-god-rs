//! `MoronBeeLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::moronpests_quiz::moron_pests_question_screen_logic;
use pg_engine::display;
use pg_engine::library;
use pg_engine::logic;

/// `MoronBeeLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MoronBeeLogic {
    /// `mMoronBeeLogicListener`: `@"<MoronBeeLogicListener>"`
    pub moron_bee_logic_listener: Option<Entity>,

    /// `mMoronPestsDisplayGroup`: `@"BCDisplayGroup"`
    pub moron_pests_display_group: Option<Entity>,

    /// `mMoronPestsPygmyLogic`: `@"PygmyLogic"`
    pub moron_pests_pygmy_logic: Option<Entity>,

    /// `mScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub screen_pos: Vec4,

    /// `mbActive`: `B`
    pub active: bool,
}

pub const MORON_BEE_LOGIC: Class = Class {
    name: "MoronBeeLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    game_frame_order: 60,
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

/// The group the Moron Pests pests live in (literal 0x100391e10).
const MORON_PESTS_DISPLAY_GROUP_ID: &str = "MoronPestsDisplayGroup";

/// The marker a bee is put in front of when it flies in (literal 0x100392330).
const MORON_PESTS_PYGMY_DISPLAY_MARKER_ID: &str = "MoronPestsPygmyDisplayMarker";

/// How far right of its screen position a bee starts (`DAT_10029d2c0`).
const ENTRY_X_OFFSET: f32 = 480.0;

/// Sequence-item handlers implemented by `MoronBeeLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        "exitScreenComplete" => seq_exit_screen_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[MoronBeeLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&MORON_BEE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mMoronPestsDisplayGroup pos]`. The group always exists; the original dereferences the
/// result of `[nil pos]` (a crash), the port panics.
fn display_group_pos(world: &World, group: Option<Entity>) -> Vec4 {
    let Some(group) = group else {
        panic!("MoronBeeLogic: MoronPestsDisplayGroup is nil (the original dereferences [nil pos])");
    };
    display::object(world, group).pos
}

/// `-[MoronBeeLogic initWithDisplayObject:]` @ 0x100174ed0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(MoronBeeLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let own_display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, own_display_object);
        object.is_selectable = true;
        object.col_x_min = -20.0;
        object.col_x_max = 20.0;
        object.col_y_min = -20.0;
        object.col_y_max = 20.0;
    }

    let moron_pests_display_group = library::display_group(world, MORON_PESTS_DISPLAY_GROUP_ID);
    world.obj_mut::<MoronBeeLogic>(this).moron_pests_display_group = moron_pests_display_group;
}

/// `-[MoronBeeLogic setMoronBeeLogicListener:]` @ 0x100174ff0
pub fn set_moron_bee_logic_listener(world: &mut World, this: Entity, moron_bee_logic_listener: Entity) {
    world.obj_mut::<MoronBeeLogic>(this).moron_bee_logic_listener = Some(moron_bee_logic_listener);
}

/// `-[MoronBeeLogic screenPos]` @ 0x1001750b0
///
/// The original returns a pointer to `mScreenPos`; its only readers (the question screen's sort
/// functions) copy the value.
pub fn screen_pos(world: &mut World, this: Entity) -> Vec4 {
    world.obj::<MoronBeeLogic>(this).screen_pos
}

/// `-[MoronBeeLogic onGameFrame:]` @ 0x1001750c8
///
/// Keeps the bee from scrolling left of its screen position while the pests' group moves.
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let (moron_pests_display_group, screen_pos) = {
        let bee = world.obj::<MoronBeeLogic>(this);
        (bee.moron_pests_display_group, bee.screen_pos)
    };
    let group_pos = display_group_pos(world, moron_pests_display_group);
    let min_x_pos = screen_pos.x - group_pos.x;

    let mut object = display::object_mut(world, display_object);
    if object.pos.x < min_x_pos {
        object.pos.x = min_x_pos;
    }
}

/// `-[MoronBeeLogic die:::]` @ 0x100175160
pub fn seq_die(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[MoronBeeLogic surroundPygmy:screenPos:]` @ 0x100175168
///
/// The original takes `screenPos` by pointer and copies all four fields.
pub fn surround_pygmy(world: &mut World, this: Entity, surround_pygmy: Option<Entity>, screen_pos: Vec4) {
    {
        let mut bee = world.obj_mut::<MoronBeeLogic>(this);
        bee.active = true;
        bee.moron_pests_pygmy_logic = surround_pygmy;
        bee.screen_pos = screen_pos;
    }

    let marker = library::display_marker(world, MORON_PESTS_PYGMY_DISPLAY_MARKER_ID);
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }

    let moron_pests_display_group = world.obj::<MoronBeeLogic>(this).moron_pests_display_group;
    let group_pos = display_group_pos(world, moron_pests_display_group);
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = (screen_pos.x + ENTRY_X_OFFSET) - group_pos.x;
        object.pos.y = screen_pos.y - group_pos.y;
    }

    logic::set_behavior(world, this, Some("MoronBeeSurroundPygmy"));
}

/// `-[MoronBeeLogic exitScreen]` @ 0x100175280
pub fn exit_screen(world: &mut World, this: Entity) {
    let active = world.obj::<MoronBeeLogic>(this).active;
    if !active {
        return;
    }

    {
        let mut bee = world.obj_mut::<MoronBeeLogic>(this);
        bee.active = false;
        bee.moron_pests_pygmy_logic = None;
    }
    logic::set_behavior(world, this, Some("MoronBeeExitScreen"));
}

/// `-[MoronBeeLogic exitScreenComplete:::]` @ 0x1001752bc
pub fn seq_exit_screen_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[MoronBeeLogic touchBeganInside:allTouches:]` @ 0x1001752f0
///
/// The listener is always the `MoronPestsQuestionScreenLogic` (`initLogicListeners`).
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let (active, listener) = {
        let bee = world.obj::<MoronBeeLogic>(this);
        (bee.active, bee.moron_bee_logic_listener)
    };
    if !active {
        return;
    }

    if let Some(listener) = listener {
        moron_pests_question_screen_logic::on_moron_bee_logic_selected(world, listener, this);
    }
}

/// `-[MoronBeeLogic touchBeganOutside:allTouches:]` @ 0x100175328
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[MoronBeeLogic touchMovedEntered:allTouches:]` @ 0x10017532c
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[MoronBeeLogic touchMovedExited:allTouches:]` @ 0x100175330
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[MoronBeeLogic touchMovedInside:allTouches:]` @ 0x100175334
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[MoronBeeLogic touchMovedOutside:allTouches:]` @ 0x100175344
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[MoronBeeLogic touchEndedInside:allTouches:]` @ 0x100175348
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[MoronBeeLogic touchEndedOutside:allTouches:]` @ 0x10017534c
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
