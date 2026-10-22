//! `BattleEnergyBallLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `BattleEnergyBallLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BattleEnergyBallLogic {
    /// `mBattleEnergyBallLogicListener`: `@"<BattleEnergyBallLogicListener>"`
    pub battle_energy_ball_logic_listener: Option<Entity>,

    /// `mTargetPygmyLogic`: `@"PygmyLogic"`
    pub target_pygmy_logic: Option<Entity>,

    /// `mbLastBall`: `B`
    pub last_ball: bool,

    /// `mbNoDamage`: `B`
    pub no_damage: bool,
}

pub const BATTLE_ENERGY_BALL_LOGIC: Class = Class {
    name: "BattleEnergyBallLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The ball aims this far right of the target per unit of the target's x scale (its facing).
const TARGET_X_OFFSET_PER_SCALE: f32 = 15.0;

/// The ball aims this far above the target's position.
const TARGET_Y_OFFSET: f32 = 32.0;

/// The share of the remaining distance the ball covers each frame.
const HOMING_RATE: f32 = 0.5;

/// The ball hits once it is this close to the target height.
const HIT_DISTANCE: f32 = 1.0;

/// `[[BattleEnergyBallLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&BATTLE_ENERGY_BALL_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[BattleEnergyBallLogic initWithDisplayObject:]` @ 0x1001a9250
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(BattleEnergyBallLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);
}

/// `-[BattleEnergyBallLogic setBattleEnergyBallLogicListener:]` @ 0x1001a92f0
pub fn set_battle_energy_ball_logic_listener(world: &mut World, this: Entity, battle_energy_ball_logic_listener: Entity) {
    world.obj_mut::<BattleEnergyBallLogic>(this).battle_energy_ball_logic_listener = Some(battle_energy_ball_logic_listener);
}

/// `-[BattleEnergyBallLogic setLastBall:]` @ 0x1001a934c
pub fn set_last_ball(world: &mut World, this: Entity, last_ball: bool) {
    world.obj_mut::<BattleEnergyBallLogic>(this).last_ball = last_ball;
}

/// `-[BattleEnergyBallLogic setNoDamage:]` @ 0x1001a935c
pub fn set_no_damage(world: &mut World, this: Entity, no_damage: bool) {
    world.obj_mut::<BattleEnergyBallLogic>(this).no_damage = no_damage;
}

/// `-[BattleEnergyBallLogic onGameFrame:]` @ 0x1001a936c
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    // The ball only runs while it is in the display tree, where `activateWithStartPos:…` puts it
    // after setting the target; the original reads `[[mTargetPygmyLogic displayObject] pos]`
    // through a NULL pointer without one.
    let target = world.obj::<BattleEnergyBallLogic>(this).target_pygmy_logic;
    let target = target.expect("BattleEnergyBallLogic: an active energy ball always has a target pygmy");

    let target_display_object = logic::display_object(world, target);
    let target_display = display::object(world, target_display_object);
    let target_pos = target_display.pos;
    let target_scale_x = target_display.scale.x;

    let pos = display::object(world, display_object).pos;
    let aim_x = target_pos.x + target_scale_x * TARGET_X_OFFSET_PER_SCALE;
    let delta_y = (target_pos.y + TARGET_Y_OFFSET) - pos.y;
    let delta_x = aim_x - pos.x;

    let next_x = pos.x + delta_x * HOMING_RATE;
    let next_y = pos.y + delta_y * HOMING_RATE;
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = next_x;
        display.pos.y = next_y;
    }

    let arrived = delta_y.abs() <= HIT_DISTANCE;
    if !arrived {
        return;
    }

    display::remove_from_display_group(world, display_object);

    let ball = world.obj::<BattleEnergyBallLogic>(this);
    if ball.no_damage {
        return;
    }

    let last_ball = ball.last_ball;
    if last_ball {
        crate::pygmy::killed_by_battle_energy_ball(world, target);
    } else {
        crate::pygmy::hit_by_battle_energy_ball(world, target);
    }
}

/// `-[BattleEnergyBallLogic activateWithStartPos:targetPygmyLogic:]` @ 0x1001a94e0
pub fn activate_with_start_pos(world: &mut World, this: Entity, start_pos: Vec4, target_pygmy_logic: Option<Entity>) {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos = start_pos;
    world.obj_mut::<BattleEnergyBallLogic>(this).target_pygmy_logic = target_pygmy_logic;

    // `[[nil displayObject] insertAfter:mDisplayObject]` does nothing.
    let Some(target_pygmy_logic) = target_pygmy_logic else {
        return;
    };
    let target_display_object = logic::display_object(world, target_pygmy_logic);
    display::insert_after(world, target_display_object, Some(display_object));
}
