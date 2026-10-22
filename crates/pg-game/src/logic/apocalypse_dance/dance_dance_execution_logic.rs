//! `DanceDanceExecutionLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::apocalypse_dance::dance_dance_execution_screen_logic;
use crate::logic::apocalypse_dance::disco_ball_logic;
use crate::logic::island_core::sprite_logic;
use crate::logic_table;
use pg_engine::animation;
use pg_engine::display;
use pg_engine::library;
use pg_engine::logic;

/// `DanceDanceExecutionLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct DanceDanceExecutionLogic {
    /// `mDanceDanceExecutionLogicListener`: `@"<DanceDanceExecutionLogicListener>"`
    pub dance_dance_execution_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mDanceFlags`: `i`
    pub dance_flags: i32,

    /// `mDanceFace`: `i`
    pub dance_face: i32,

    /// `mTriggeredDanceFlag`: `i`
    pub triggered_dance_flag: i32,

    /// `mDanceFaceLogic`: `@"MultiModelLogic"`
    pub dance_face_logic: Option<Entity>,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const DANCE_DANCE_EXECUTION_LOGIC: Class = Class {
    name: "DanceDanceExecutionLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `mDanceFlags` bits of the held arrow buttons.
const LEFT_ARROW: i32 = 1;
const DOWN_ARROW: i32 = 2;
const UP_ARROW: i32 = 4;
const RIGHT_ARROW: i32 = 8;

/// The stage animation's channel that carries the dancing pygmy.
const PYGMY_CHANNEL: usize = 1;

/// One record of `updateDanceMove`'s table at `0x1007516c0`: the stage behaviour played while
/// all of `arrows` are held.
struct DanceMove {
    behavior: &'static str,
    arrows: u32,
}

/// The table at `0x1007516c0` (count `DAT_100751760` = 10): two-arrow moves first, so they win
/// over the single arrows they contain.
const DANCE_MOVES: [DanceMove; 10] = [
    DanceMove { behavior: "DanceDanceExecutionLeftDown", arrows: 3 },
    DanceMove { behavior: "DanceDanceExecutionLeftUp", arrows: 5 },
    DanceMove { behavior: "DanceDanceExecutionDownUp", arrows: 6 },
    DanceMove { behavior: "DanceDanceExecutionRightLeft", arrows: 9 },
    DanceMove { behavior: "DanceDanceExecutionRightDown", arrows: 10 },
    DanceMove { behavior: "DanceDanceExecutionRightUp", arrows: 12 },
    DanceMove { behavior: "DanceDanceExecutionLeft", arrows: 1 },
    DanceMove { behavior: "DanceDanceExecutionDown", arrows: 2 },
    DanceMove { behavior: "DanceDanceExecutionUp", arrows: 4 },
    DanceMove { behavior: "DanceDanceExecutionRight", arrows: 8 },
];

/// Sequence-item handlers implemented by `DanceDanceExecutionLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "stopBackgroundLoop" => seq_stop_background_loop(world, this, call),
        "playBackgroundLoop" => seq_play_background_loop(world, this, call),
        "smashPygmyComplete" => seq_smash_pygmy_complete(world, this, call),
        "pygmyFallInsideComplete" => seq_pygmy_fall_inside_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `DanceDanceExecutionLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animStartDanceIntro" => anim_start_dance_intro(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[DanceDanceExecutionLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&DANCE_DANCE_EXECUTION_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[DanceDanceExecutionLogic initWithDisplayObject:]` @ 0x10017c834
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(DanceDanceExecutionLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;

    let dance_face_logic = library::logic(world, logic_table::DANCE_DANCE_PYGMY_FACE);
    world.obj_mut::<DanceDanceExecutionLogic>(this).dance_face_logic = Some(dance_face_logic);
}

/// `-[DanceDanceExecutionLogic setDanceDanceExecutionLogicListener:]` @ 0x10017c8f4
pub fn set_dance_dance_execution_logic_listener(world: &mut World, this: Entity, dance_dance_execution_logic_listener: Entity) {
    let listener = Some(dance_dance_execution_logic_listener);
    world.obj_mut::<DanceDanceExecutionLogic>(this).dance_dance_execution_logic_listener = listener;
}

/// `-[DanceDanceExecutionLogic setDanceFace:]` @ 0x10017c950
///
/// The pygmy's face on the screen: 0 good, 1 bad, 2 cocky.
pub fn set_dance_face(world: &mut World, this: Entity, dance_face: i32) {
    let stage = world.obj::<DanceDanceExecutionLogic>(this);
    if stage.dance_face == dance_face {
        return;
    }
    let dance_face_logic = stage.dance_face_logic;
    world.obj_mut::<DanceDanceExecutionLogic>(this).dance_face = dance_face;

    let behavior = match dance_face {
        2 => "DanceFaceCocky",
        1 => "DanceFaceBad",
        0 => "DanceFaceGood",
        _ => return,
    };
    if let Some(dance_face_logic) = dance_face_logic {
        logic::set_behavior(world, dance_face_logic, Some(behavior));
    }
}

/// `-[DanceDanceExecutionLogic updateDanceMove]` @ 0x10017c9ec
///
/// Plays the first move whose arrows are all held.
pub fn update_dance_move(world: &mut World, this: Entity) {
    let dance_flags = world.obj::<DanceDanceExecutionLogic>(this).dance_flags as u32;

    for dance_move in &DANCE_MOVES {
        let held_arrows = dance_flags & dance_move.arrows;
        if held_arrows != dance_move.arrows {
            continue;
        }

        world.obj_mut::<DanceDanceExecutionLogic>(this).triggered_dance_flag = dance_move.arrows as i32;
        logic::set_behavior(world, this, Some(dance_move.behavior));
        return;
    }
}

/// `-[DanceDanceExecutionLogic pressDanceButton:]` @ 0x10017ca5c
pub fn press_dance_button(world: &mut World, this: Entity, press_dance_button: i32) {
    world.obj_mut::<DanceDanceExecutionLogic>(this).dance_flags |= press_dance_button;
    update_dance_move(world, this);
}

/// `-[DanceDanceExecutionLogic releaseDanceButton:]` @ 0x10017ca80
pub fn release_dance_button(world: &mut World, this: Entity, release_dance_button: i32) {
    world.obj_mut::<DanceDanceExecutionLogic>(this).dance_flags &= !release_dance_button;
}

/// `-[DanceDanceExecutionLogic onPressLeft]` @ 0x10017ca98
pub fn on_press_left(world: &mut World, this: Entity) {
    press_dance_button(world, this, LEFT_ARROW);
}

/// `-[DanceDanceExecutionLogic onReleaseLeft]` @ 0x10017caac
pub fn on_release_left(world: &mut World, this: Entity) {
    release_dance_button(world, this, LEFT_ARROW);
}

/// `-[DanceDanceExecutionLogic onPressDown]` @ 0x10017cac0
pub fn on_press_down(world: &mut World, this: Entity) {
    press_dance_button(world, this, DOWN_ARROW);
}

/// `-[DanceDanceExecutionLogic onReleaseDown]` @ 0x10017cad4
pub fn on_release_down(world: &mut World, this: Entity) {
    release_dance_button(world, this, DOWN_ARROW);
}

/// `-[DanceDanceExecutionLogic onPressUp]` @ 0x10017cae8
pub fn on_press_up(world: &mut World, this: Entity) {
    press_dance_button(world, this, UP_ARROW);
}

/// `-[DanceDanceExecutionLogic onReleaseUp]` @ 0x10017cafc
pub fn on_release_up(world: &mut World, this: Entity) {
    release_dance_button(world, this, UP_ARROW);
}

/// `-[DanceDanceExecutionLogic onPressRight]` @ 0x10017cb10
pub fn on_press_right(world: &mut World, this: Entity) {
    press_dance_button(world, this, RIGHT_ARROW);
}

/// `-[DanceDanceExecutionLogic onReleaseRight]` @ 0x10017cb24
pub fn on_release_right(world: &mut World, this: Entity) {
    release_dance_button(world, this, RIGHT_ARROW);
}

/// `-[DanceDanceExecutionLogic isAvailable]` @ 0x10017cb38
pub fn is_available(world: &mut World, this: Entity) -> bool {
    world.obj::<DanceDanceExecutionLogic>(this).pygmy_logic.is_none()
}

/// `-[DanceDanceExecutionLogic pygmyFallIntoDanceDanceExecution:]` @ 0x10017cb50
///
/// An empty stage takes the pygmy and opens the rhythm game.
pub fn pygmy_fall_into_dance_dance_execution(world: &mut World, this: Entity, pygmy_fall_into_dance_dance_execution: Entity) {
    let pygmy = pygmy_fall_into_dance_dance_execution;
    world.obj_mut::<DanceDanceExecutionLogic>(this).dance_flags = 0;
    if world.obj::<DanceDanceExecutionLogic>(this).pygmy_logic.is_some() {
        return;
    }
    world.obj_mut::<DanceDanceExecutionLogic>(this).pygmy_logic = Some(pygmy);

    let stage_animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy);
    animation::set_tweenable(world, stage_animation, Some(pygmy_display_object), PYGMY_CHANNEL);

    logic::set_behavior(world, this, Some("DanceDanceExecutionPygmyFallInside"));

    let screen = library::logic(world, logic_table::DANCE_DANCE_EXECUTION_SCREEN);
    dance_dance_execution_screen_logic::open_dance_dance_execution(world, screen);
}

/// `-[DanceDanceExecutionLogic enterIsland:]` @ 0x10017cc2c
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    world.obj_mut::<DanceDanceExecutionLogic>(this).active = true;
    logic::set_behavior(world, this, Some("DanceDanceExecutionEnterIsland"));
}

/// `-[DanceDanceExecutionLogic removePygmyFromDanceDanceExecution:]` @ 0x10017cc54
///
/// Quits the rhythm game whether or not this pygmy was the dancer.
pub fn remove_pygmy_from_dance_dance_execution(world: &mut World, this: Entity, pygmy_from_dance_dance_execution: Entity) {
    let pygmy_logic = world.obj::<DanceDanceExecutionLogic>(this).pygmy_logic;
    if pygmy_logic == Some(pygmy_from_dance_dance_execution) {
        world.obj_mut::<DanceDanceExecutionLogic>(this).pygmy_logic = None;

        let stage_animation = logic::animation(world, this);
        animation::clear_tweenable_channel_id_now(world, stage_animation, PYGMY_CHANNEL);

        logic::set_behavior(world, this, Some("DanceDanceExecutionRemovePygmy"));
    }

    let screen = library::logic(world, logic_table::DANCE_DANCE_EXECUTION_SCREEN);
    dance_dance_execution_screen_logic::quit_screen(world, screen);
}

/// `-[DanceDanceExecutionLogic exitIsland:]` @ 0x10017cd00
///
/// A pygmy still on the stage is thrown off.
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    world.obj_mut::<DanceDanceExecutionLogic>(this).active = false;

    let pygmy_logic = world.obj::<DanceDanceExecutionLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy_logic {
        let stage_animation = logic::animation(world, this);
        animation::clear_tweenable_channel_id_now(world, stage_animation, PYGMY_CHANNEL);

        crate::pygmy::toss_from_dance_dance_execution(world, pygmy);
        world.obj_mut::<DanceDanceExecutionLogic>(this).pygmy_logic = None;
    }

    logic::set_behavior(world, this, Some("DanceDanceExecutionExitIsland"));
}

/// `-[DanceDanceExecutionLogic gameOver]` @ 0x10017cdf0
///
/// The doom meter is full: the pygmy is executed and the disco ball falls on it.
pub fn game_over(world: &mut World, this: Entity) {
    let pygmy_logic = world.obj::<DanceDanceExecutionLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy_logic {
        crate::pygmy::dance_dance_executed(world, pygmy, this);
    }

    let disco_ball = library::logic(world, logic_table::DISCO_BALL);
    disco_ball_logic::smash_pygmy(world, disco_ball);
}

/// `-[DanceDanceExecutionLogic smashPygmy]` @ 0x10017ce5c
pub fn smash_pygmy(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("DanceDanceExecutionSmashPygmy"));
}

/// `-[DanceDanceExecutionLogic animStartDanceIntro]` @ 0x10017ce74
pub fn anim_start_dance_intro(world: &mut World, _this: Entity) {
    let screen = library::logic(world, logic_table::DANCE_DANCE_EXECUTION_SCREEN);
    dance_dance_execution_screen_logic::on_dance_intro_started(world, screen);
}

/// `-[DanceDanceExecutionLogic stopBackgroundLoop:::]` @ 0x10017cebc
pub fn seq_stop_background_loop(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<DanceDanceExecutionLogic>(this).dance_dance_execution_logic_listener;
    if let Some(listener) = listener {
        crate::controller::stop_background_loop(world, listener);
    }
    ItemResult::Done
}

/// `-[DanceDanceExecutionLogic playBackgroundLoop:::]` @ 0x10017ceec
pub fn seq_play_background_loop(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<DanceDanceExecutionLogic>(this).dance_dance_execution_logic_listener;
    if let Some(listener) = listener {
        crate::controller::play_background_loop(world, listener, None);
    }
    ItemResult::Done
}

/// `-[DanceDanceExecutionLogic smashPygmyComplete:::]` @ 0x10017cf20
pub fn seq_smash_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let stage_animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, stage_animation, PYGMY_CHANNEL);

    let pygmy_logic = world.obj::<DanceDanceExecutionLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy_logic {
        crate::pygmy::dance_dance_executed_complete(world, pygmy, this);
    }
    world.obj_mut::<DanceDanceExecutionLogic>(this).pygmy_logic = None;
    ItemResult::Done
}

/// `-[DanceDanceExecutionLogic onGameFrame:]` @ 0x10017cf88
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[DanceDanceExecutionLogic pygmyFallInsideComplete:::]` @ 0x10017cf8c
pub fn seq_pygmy_fall_inside_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<DanceDanceExecutionLogic>(this).dance_dance_execution_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_dance_dance_execution_start(world, listener, this);
    }
    ItemResult::Done
}
