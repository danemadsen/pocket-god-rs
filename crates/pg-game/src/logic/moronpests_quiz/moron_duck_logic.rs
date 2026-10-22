//! `MoronDuckLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::moronpests_quiz::moron_pests_question_screen_logic;
use pg_engine::display;
use pg_engine::library;
use pg_engine::logic;

/// `MoronDuckLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MoronDuckLogic {
    /// `mMoronDuckLogicListener`: `@"<MoronDuckLogicListener>"`
    pub moron_duck_logic_listener: Option<Entity>,

    /// `mHeadLogic`: `@"MultiModelLogic"`
    pub head_logic: Option<Entity>,

    /// `mMoronPestsDisplayGroup`: `@"BCDisplayGroup"`
    pub moron_pests_display_group: Option<Entity>,

    /// `mMoronPestsPygmyLogic`: `@"PygmyLogic"`
    pub moron_pests_pygmy_logic: Option<Entity>,

    /// `mMoronDuckEyeScale`: `i`
    pub moron_duck_eye_scale: i32,

    /// `mScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub screen_pos: Vec4,

    /// `mbActive`: `B`
    pub active: bool,
}

pub const MORON_DUCK_LOGIC: Class = Class {
    name: "MoronDuckLogic",
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

/// The marker a duck is put in front of when it comes in (literal 0x100392330).
const MORON_PESTS_PYGMY_DISPLAY_MARKER_ID: &str = "MoronPestsPygmyDisplayMarker";

/// How far right of its screen position a duck starts (`DAT_10029d2c0`).
const ENTRY_X_OFFSET: f32 = 480.0;

/// Sequence-item handlers implemented by `MoronDuckLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        "exitScreenComplete" => seq_exit_screen_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[MoronDuckLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&MORON_DUCK_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mMoronPestsDisplayGroup pos]`. The group always exists; the original dereferences the
/// result of `[nil pos]` (a crash), the port panics.
fn display_group_pos(world: &World, group: Option<Entity>) -> Vec4 {
    let Some(group) = group else {
        panic!("MoronDuckLogic: MoronPestsDisplayGroup is nil (the original dereferences [nil pos])");
    };
    display::object(world, group).pos
}

/// `-[MoronDuckLogic initWithDisplayObject:]` @ 0x1001a7e24
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(MoronDuckLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let own_display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, own_display_object);
        object.is_selectable = true;
        object.col_x_min = -40.0;
        object.col_x_max = 40.0;
        object.col_y_min = 0.0;
        object.col_y_max = 80.0;
    }

    let moron_pests_display_group = library::display_group(world, MORON_PESTS_DISPLAY_GROUP_ID);
    world.obj_mut::<MoronDuckLogic>(this).moron_pests_display_group = moron_pests_display_group;
}

/// `-[MoronDuckLogic setMoronDuckLogicListener:]` @ 0x1001a7f40
pub fn set_moron_duck_logic_listener(world: &mut World, this: Entity, moron_duck_logic_listener: Entity) {
    world.obj_mut::<MoronDuckLogic>(this).moron_duck_logic_listener = Some(moron_duck_logic_listener);
}

/// `-[MoronDuckLogic setHeadLogic:]` @ 0x1001a8000
pub fn set_head_logic(world: &mut World, this: Entity, head_logic: Entity) {
    world.obj_mut::<MoronDuckLogic>(this).head_logic = Some(head_logic);
}

/// `-[MoronDuckLogic screenPos]` @ 0x1001a8010
///
/// The original returns a pointer to `mScreenPos`; nothing reads a duck's through it.
pub fn screen_pos(world: &mut World, this: Entity) -> Vec4 {
    world.obj::<MoronDuckLogic>(this).screen_pos
}

/// `-[MoronDuckLogic moronDuckEyeScale]` @ 0x1001a8020
pub fn moron_duck_eye_scale(world: &mut World, this: Entity) -> i32 {
    world.obj::<MoronDuckLogic>(this).moron_duck_eye_scale
}

/// `-[MoronDuckLogic onGameFrame:]` @ 0x1001a8038
///
/// Keeps the duck from scrolling left of its screen position while the pests' group moves.
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let (moron_pests_display_group, screen_pos) = {
        let duck = world.obj::<MoronDuckLogic>(this);
        (duck.moron_pests_display_group, duck.screen_pos)
    };
    let group_pos = display_group_pos(world, moron_pests_display_group);
    let min_x_pos = screen_pos.x - group_pos.x;

    let mut object = display::object_mut(world, display_object);
    if object.pos.x < min_x_pos {
        object.pos.x = min_x_pos;
    }
}

/// `-[MoronDuckLogic die:::]` @ 0x1001a80d0
pub fn seq_die(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[MoronDuckLogic surroundPygmy:screenPos:scale:moronDuckEyeScale:]` @ 0x1001a80d8
///
/// The original takes `screenPos` by pointer and copies all four fields. The head keeps its
/// size on screen (`1 / scale²` inside a body of `scale`); the eye-scale rank picks its eyes.
pub fn surround_pygmy(world: &mut World, this: Entity, surround_pygmy: Option<Entity>, screen_pos: Vec4, scale: f32, moron_duck_eye_scale: i32) {
    {
        let mut duck = world.obj_mut::<MoronDuckLogic>(this);
        duck.active = true;
        duck.moron_pests_pygmy_logic = surround_pygmy;
        duck.screen_pos = screen_pos;
        duck.moron_duck_eye_scale = moron_duck_eye_scale;
    }

    let marker = library::display_marker(world, MORON_PESTS_PYGMY_DISPLAY_MARKER_ID);
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }

    {
        let mut object = display::object_mut(world, display_object);
        object.scale.x = scale;
        object.scale.y = scale;
    }

    let head_logic = world.obj::<MoronDuckLogic>(this).head_logic;
    let Some(head_logic) = head_logic else {
        panic!("MoronDuckLogic: mHeadLogic is nil (the original writes through [[nil displayObject] scale])");
    };
    let head_display_object = logic::display_object(world, head_logic);
    let head_scale = 1.0 / (scale * scale);
    {
        let mut head_object = display::object_mut(world, head_display_object);
        head_object.scale.x = head_scale;
        head_object.scale.y = head_scale;
    }

    let eye_scale = world.obj::<MoronDuckLogic>(this).moron_duck_eye_scale;
    let head_animation_id = match eye_scale {
        0 => Some("RubberDuckHead4"),
        1 => Some("RubberDuckHead3"),
        2 => Some("RubberDuckHead2"),
        3 => Some("RubberDuckHead1"),
        _ => None,
    };
    if let Some(head_animation_id) = head_animation_id {
        logic::play_animation_id(world, head_logic, Some(head_animation_id));
    }

    let moron_pests_display_group = world.obj::<MoronDuckLogic>(this).moron_pests_display_group;
    let group_pos = display_group_pos(world, moron_pests_display_group);
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = (screen_pos.x + ENTRY_X_OFFSET) - group_pos.x;
        object.pos.y = screen_pos.y - group_pos.y;
    }

    logic::set_behavior(world, this, Some("MoronDuckSurroundPygmy"));
}

/// `-[MoronDuckLogic exitScreen]` @ 0x1001a8300
pub fn exit_screen(world: &mut World, this: Entity) {
    let active = world.obj::<MoronDuckLogic>(this).active;
    if !active {
        return;
    }

    {
        let mut duck = world.obj_mut::<MoronDuckLogic>(this);
        duck.active = false;
        duck.moron_pests_pygmy_logic = None;
    }
    logic::set_behavior(world, this, Some("MoronDuckExitScreen"));
}

/// `-[MoronDuckLogic exitScreenComplete:::]` @ 0x1001a833c
pub fn seq_exit_screen_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[MoronDuckLogic touchBeganInside:allTouches:]` @ 0x1001a8370
///
/// The listener is always the `MoronPestsQuestionScreenLogic` (`initLogicListeners`).
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let (active, listener) = {
        let duck = world.obj::<MoronDuckLogic>(this);
        (duck.active, duck.moron_duck_logic_listener)
    };
    if !active {
        return;
    }

    if let Some(listener) = listener {
        moron_pests_question_screen_logic::on_moron_duck_logic_selected(world, listener, this);
    }
}

/// `-[MoronDuckLogic touchBeganOutside:allTouches:]` @ 0x1001a83a8
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[MoronDuckLogic touchMovedEntered:allTouches:]` @ 0x1001a83ac
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[MoronDuckLogic touchMovedExited:allTouches:]` @ 0x1001a83b0
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[MoronDuckLogic touchMovedInside:allTouches:]` @ 0x1001a83b4
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[MoronDuckLogic touchMovedOutside:allTouches:]` @ 0x1001a83c4
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[MoronDuckLogic touchEndedInside:allTouches:]` @ 0x1001a83c8
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[MoronDuckLogic touchEndedOutside:allTouches:]` @ 0x1001a83cc
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
