//! `ApeTorchLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ape_monkeys::monkey_logic;
use crate::logic::ape_ritual::ape_logic;
use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `ApeTorchLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApeTorchLogic {
    /// `mApeTorchLogicListener`: `@"<ApeTorchLogicListener>"`
    pub ape_torch_logic_listener: Option<Entity>,

    /// `mbLit`: `B`
    pub lit: bool,

    /// `mMonkeyLogic`: `@"MonkeyLogic"`
    pub monkey_logic: Option<Entity>,
}

pub const APE_TORCH_LOGIC: Class = Class {
    name: "ApeTorchLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    perform,
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

/// The tweenable channel of the torch's animation that carries a climbing or perched monkey.
const MONKEY_CHANNEL: usize = 2;

/// Zero-argument selectors sent by name (animation frame actions) that `ApeTorchLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animMonkeyClimbComplete" => anim_monkey_climb_complete(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[ApeTorchLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APE_TORCH_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mApeTorchLogicListener islandCenterXPos]` (0 for a nil listener).
fn listener_island_center_x_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<ApeTorchLogic>(this).ape_torch_logic_listener;
    let Some(listener) = listener else {
        return 0.0;
    };
    crate::controller::island_center_x_pos(world, listener)
}

/// Binds the monkey to the torch's monkey channel and plays the climb or landing animation,
/// mirrored for the torch's side of the island (`startMonkeyClimb:` and `monkeyLand:`).
fn take_monkey(world: &mut World, this: Entity, monkey: Entity, left_behavior: &str, right_behavior: &str) {
    world.obj_mut::<ApeTorchLogic>(this).monkey_logic = Some(monkey);

    let animation = logic::animation(world, this);
    let monkey_display_object = logic::display_object(world, monkey);
    pg_engine::animation::set_tweenable(world, animation, Some(monkey_display_object), MONKEY_CHANNEL);

    let island_center_x_pos = listener_island_center_x_pos(world, this);
    let display_object = logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let behavior = if x_pos > island_center_x_pos { left_behavior } else { right_behavior };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[ApeTorchLogic initWithDisplayObject:]` @ 0x10001f748
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApeTorchLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::stop_game_frame(world, this);
    logic::set_behavior(world, this, Some("ApeTorchEnterIsland"));
}

/// `-[ApeTorchLogic setApeTorchLogicListener:]` @ 0x10001f7ec
pub fn set_ape_torch_logic_listener(world: &mut World, this: Entity, ape_torch_logic_listener: Entity) {
    world.obj_mut::<ApeTorchLogic>(this).ape_torch_logic_listener = Some(ape_torch_logic_listener);
}

/// `-[ApeTorchLogic enterIsland:]` @ 0x10001f848
///
/// The startup flag is unused.
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    world.obj_mut::<ApeTorchLogic>(this).lit = false;
    logic::set_behavior(world, this, Some("ApeTorchEnterIsland"));
}

/// `-[ApeTorchLogic exitIsland]` @ 0x10001f86c
pub fn exit_island(world: &mut World, this: Entity) {
    let monkey = world.obj::<ApeTorchLogic>(this).monkey_logic;
    if let Some(monkey) = monkey {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, MONKEY_CHANNEL);
        monkey_logic::booted_from_ape_torch(world, monkey);
        world.obj_mut::<ApeTorchLogic>(this).monkey_logic = None;
    }

    logic::set_behavior(world, this, Some("ApeTorchExitIsland"));
}

/// `-[ApeTorchLogic isLit]` @ 0x10001f8ec
pub fn is_lit(world: &mut World, this: Entity) -> bool {
    world.obj::<ApeTorchLogic>(this).lit
}

/// `-[ApeTorchLogic hitByLightning]` @ 0x10001f8fc
///
/// Lights an unlit torch (which can then be tapped to douse it) and burns a perched monkey
/// off. Returns whether the torch was lit by this strike.
pub fn hit_by_lightning(world: &mut World, this: Entity) -> bool {
    let lit = world.obj::<ApeTorchLogic>(this).lit;
    if lit {
        return false;
    }

    world.obj_mut::<ApeTorchLogic>(this).lit = true;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("ApeTorchLit"));

    let monkey = world.obj::<ApeTorchLogic>(this).monkey_logic;
    if let Some(monkey) = monkey {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, MONKEY_CHANNEL);
        monkey_logic::booted_from_ape_torch_by_fire(world, monkey);
        world.obj_mut::<ApeTorchLogic>(this).monkey_logic = None;
    }
    true
}

/// `-[ApeTorchLogic isAvailableForMonkey]` @ 0x10001f9d4
pub fn is_available_for_monkey(world: &mut World, this: Entity) -> bool {
    let torch = world.obj::<ApeTorchLogic>(this);
    torch.monkey_logic.is_none() && !torch.lit
}

/// `-[ApeTorchLogic startMonkeyClimb:]` @ 0x10001fa04
pub fn start_monkey_climb(world: &mut World, this: Entity, monkey_logic: Entity) {
    take_monkey(world, this, monkey_logic, "ApeTorchMonkeyClimbLeft", "ApeTorchMonkeyClimbRight");
}

/// `-[ApeTorchLogic monkeyLand:]` @ 0x10001fadc
pub fn monkey_land(world: &mut World, this: Entity, monkey_logic: Entity) {
    take_monkey(world, this, monkey_logic, "ApeTorchMonkeyLandLeft", "ApeTorchMonkeyLandRight");
}

/// `-[ApeTorchLogic monkeyJumpOffTorch:]` @ 0x10001fbb4
pub fn monkey_jump_off_torch(world: &mut World, this: Entity, _monkey_logic: Entity) {
    world.obj_mut::<ApeTorchLogic>(this).monkey_logic = None;

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, MONKEY_CHANNEL);

    let lit = world.obj::<ApeTorchLogic>(this).lit;
    if lit {
        return;
    }
    logic::set_behavior(world, this, Some("ApeTorchEnterIsland"));
}

/// `-[ApeTorchLogic animMonkeyClimbComplete]` @ 0x10001fc34
pub fn anim_monkey_climb_complete(world: &mut World, this: Entity) {
    let monkey = world.obj::<ApeTorchLogic>(this).monkey_logic;
    if let Some(monkey) = monkey {
        monkey_logic::ape_torch_climb_complete(world, monkey);
    }
}

/// `-[ApeTorchLogic onGameFrame:]` @ 0x10001fcb4
///
/// Empty: the torch does not run `SpriteLogic`'s physics.
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[ApeTorchLogic touchBeganInside:allTouches:]` @ 0x10001fcb8
///
/// A tap douses a lit torch and sends the ape away.
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let lit = world.obj::<ApeTorchLogic>(this).lit;
    if !lit {
        return;
    }
    world.obj_mut::<ApeTorchLogic>(this).lit = false;

    let listener = world.obj::<ApeTorchLogic>(this).ape_torch_logic_listener;
    let ape = listener.and_then(|listener| crate::controller::ape_logic(world, listener, this));
    if let Some(ape) = ape {
        ape_logic::on_douse_torch(world, ape);
    }

    logic::set_behavior(world, this, Some("ApeTorchDouse"));
}

/// `-[ApeTorchLogic touchBeganOutside:allTouches:]` @ 0x10001fd3c
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApeTorchLogic touchMovedEntered:allTouches:]` @ 0x10001fd40
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApeTorchLogic touchMovedExited:allTouches:]` @ 0x10001fd44
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApeTorchLogic touchMovedInside:allTouches:]` @ 0x10001fd48
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[ApeTorchLogic touchMovedOutside:allTouches:]` @ 0x10001fd58
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApeTorchLogic touchEndedInside:allTouches:]` @ 0x10001fd5c
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApeTorchLogic touchEndedOutside:allTouches:]` @ 0x10001fd60
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
