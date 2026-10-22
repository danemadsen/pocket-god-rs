//! `MoronTurtleLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::moronpests_chase::moron_snake_logic;
use pg_engine::display;
use pg_engine::logic;

/// `MoronTurtleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MoronTurtleLogic {
    /// `mMoronTurtleLogicListener`: `@"<MoronTurtleLogicListener>"`
    pub moron_turtle_logic_listener: Option<Entity>,

    /// `mbStartedChasingPygmy`: `B`
    pub started_chasing_pygmy: bool,

    /// `mbCaughtPygmy`: `B`
    pub caught_pygmy: bool,

    /// `mbCatchingUp`: `B`
    pub catching_up: bool,

    /// `mMoronPestsDisplayGroup`: `@"BCDisplayGroup"`
    pub moron_pests_display_group: Option<Entity>,

    /// `mMoronPestsPygmyLogic`: `@"PygmyLogic"`
    pub moron_pests_pygmy_logic: Option<Entity>,
}

pub const MORON_TURTLE_LOGIC: Class = Class {
    name: "MoronTurtleLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The animation channel that carries the pygmy while it rides or is eaten.
const PYGMY_CHANNEL: usize = 1;

/// Where `startMoronPests:` puts the turtle (x and y; the rest of `pos` is kept).
const START_X_POS: f32 = 47.0;
const START_Y_POS: f32 = 130.0;

/// The running speed the turtle accelerates to after the pygmy jumps off its back.
const CATCH_UP_SPEED: f32 = 500.0;

/// The fastest the turtle runs while chasing.
const MAX_CHASE_SPEED: f32 = 525.0;

/// The turtle never falls further than this behind the left edge of the screen.
const MIN_SCREEN_X_POS: f32 = -50.0;

/// The pygmy is caught once it is less than this far ahead of the turtle.
const CATCH_DISTANCE: f32 = 90.0;

/// Sequence-item handlers implemented by `MoronTurtleLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        "onPygmyJumpOnBackComplete" => seq_on_pygmy_jump_on_back_complete(world, this, call),
        "onEatPygmyComplete" => seq_on_eat_pygmy_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `MoronTurtleLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animStopMoving" => anim_stop_moving(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[MoronTurtleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&MORON_TURTLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[MoronTurtleLogic initWithDisplayObject:]` @ 0x100172ec8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(MoronTurtleLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let display_group = pg_engine::library::display_group(world, "MoronPestsDisplayGroup");
    world.obj_mut::<MoronTurtleLogic>(this).moron_pests_display_group = display_group;
}

/// `-[MoronTurtleLogic setMoronTurtleLogicListener:]` @ 0x100172f54
pub fn set_moron_turtle_logic_listener(world: &mut World, this: Entity, moron_turtle_logic_listener: Entity) {
    world.obj_mut::<MoronTurtleLogic>(this).moron_turtle_logic_listener = Some(moron_turtle_logic_listener);
}

/// `-[MoronTurtleLogic checkPygmyCollision]` @ 0x100173014
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let turtle = world.obj::<MoronTurtleLogic>(this);
    let caught_pygmy = turtle.caught_pygmy;
    let pygmy_logic = turtle.moron_pests_pygmy_logic;
    if caught_pygmy {
        return;
    }

    // The chase only runs after `startMoronPests:` set the pygmy (the original would read
    // through a NULL position otherwise).
    let Some(pygmy_logic) = pygmy_logic else {
        return;
    };

    let display_object = logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
    let caught = x_pos + CATCH_DISTANCE > pygmy_x_pos;
    if !caught {
        return;
    }

    world.obj_mut::<MoronTurtleLogic>(this).caught_pygmy = true;

    let listener = world.obj::<MoronTurtleLogic>(this).moron_turtle_logic_listener;
    if let Some(listener) = listener {
        moron_snake_logic::on_moron_turtle_catch_pygmy(world, listener);
    }

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);

    crate::pygmy::eaten_by_moron_turtle(world, pygmy_logic, this);
    logic::set_behavior(world, this, Some("MoronTurtleEatPygmy"));
}

/// `-[MoronTurtleLogic onGameFrame:]` @ 0x10017315c
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    let turtle = world.obj::<MoronTurtleLogic>(this);
    let started_chasing_pygmy = turtle.started_chasing_pygmy;
    let catching_up = turtle.catching_up;
    let caught_pygmy = turtle.caught_pygmy;
    let display_group = turtle.moron_pests_display_group;
    if shutdown || !started_chasing_pygmy {
        return;
    }

    if catching_up {
        let x_vel = logic::logic(world, this).pos_vel.x;
        let new_x_vel = x_vel + (CATCH_UP_SPEED - x_vel) * 0.2 * 60.0 * dt;
        logic::logic_mut(world, this).pos_vel.x = new_x_vel;

        if CATCH_UP_SPEED - new_x_vel < 1.0 {
            world.obj_mut::<MoronTurtleLogic>(this).catching_up = false;
        }
    } else if !caught_pygmy {
        let x_vel = logic::logic(world, this).pos_vel.x;
        let new_x_vel = dt * 0.2 + x_vel;
        logic::logic_mut(world, this).pos_vel.x = new_x_vel;

        if new_x_vel > MAX_CHASE_SPEED {
            logic::logic_mut(world, this).pos_vel.x = MAX_CHASE_SPEED;
        }
    }

    let display_object = logic::display_object(world, this);
    let pos_vel = logic::logic(world, this).pos_vel;
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = display.pos.x + pos_vel.x * dt;
        display.pos.y = display.pos.y + pos_vel.y * dt;
        display.pos.z = display.pos.z + pos_vel.z * dt;
    }

    // `[nil pos]` would be read through NULL; the group always exists.
    if let Some(display_group) = display_group {
        let group_x_pos = display::object(world, display_group).pos.x;
        let x_pos = display::object(world, display_object).pos.x;
        if x_pos + group_x_pos < MIN_SCREEN_X_POS {
            display::object_mut(world, display_object).pos.x = MIN_SCREEN_X_POS - group_x_pos;
        }
    }

    check_pygmy_collision(world, this);
}

/// `-[MoronTurtleLogic die:::]` @ 0x100173338
pub fn seq_die(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[MoronTurtleLogic startMoronPests:]` @ 0x100173340
pub fn start_moron_pests(world: &mut World, this: Entity, pygmy_logic: Entity) {
    world.obj_mut::<MoronTurtleLogic>(this).moron_pests_pygmy_logic = Some(pygmy_logic);

    let display_object = logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "MoronPestsPygmyDisplayMarker");
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }

    {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = START_X_POS;
        display.pos.y = START_Y_POS;
    }

    {
        let mut turtle = world.obj_mut::<MoronTurtleLogic>(this);
        turtle.started_chasing_pygmy = false;
        turtle.caught_pygmy = false;
    }

    logic::set_behavior(world, this, Some("MoronTurtleWaitForPygmy"));
}

/// `-[MoronTurtleLogic pygmyJumpOnBack:]` @ 0x100173408
pub fn pygmy_jump_on_back(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);

    logic::set_behavior(world, this, Some("MoronTurtlePygmyJumpOnBack"));
}

/// `-[MoronTurtleLogic onPygmyJumpOnBackComplete:::]` @ 0x10017347c
pub fn seq_on_pygmy_jump_on_back_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pygmy_logic = world.obj::<MoronTurtleLogic>(this).moron_pests_pygmy_logic;
    if let Some(pygmy_logic) = pygmy_logic {
        crate::pygmy::jump_on_moron_turtle_back_complete(world, pygmy_logic, this);
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

    {
        let mut turtle = world.obj_mut::<MoronTurtleLogic>(this);
        turtle.catching_up = true;
        turtle.started_chasing_pygmy = true;
    }

    logic::set_behavior(world, this, Some("MoronTurtleStartRunning"));
    ItemResult::Done
}

/// `-[MoronTurtleLogic animStopMoving]` @ 0x100173518
pub fn anim_stop_moving(world: &mut World, this: Entity) {
    logic::logic_mut(world, this).pos_vel.x = 0.0;

    let pygmy_logic = world.obj::<MoronTurtleLogic>(this).moron_pests_pygmy_logic;
    if let Some(pygmy_logic) = pygmy_logic {
        crate::pygmy::eaten_by_moron_turtle_stop_moving(world, pygmy_logic, this);
    }

    let listener = world.obj::<MoronTurtleLogic>(this).moron_turtle_logic_listener;
    if let Some(listener) = listener {
        moron_snake_logic::on_moron_pests_stop_moving_camera(world, listener);
    }
}

/// `-[MoronTurtleLogic onEatPygmyComplete:::]` @ 0x10017357c
pub fn seq_on_eat_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pygmy_logic = world.obj::<MoronTurtleLogic>(this).moron_pests_pygmy_logic;
    if let Some(pygmy_logic) = pygmy_logic {
        crate::pygmy::eaten_by_moron_turtle_complete(world, pygmy_logic, this);
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    ItemResult::Done
}
