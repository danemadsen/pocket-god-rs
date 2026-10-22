//! `ApeAltarLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ape_ritual::ape_wench_logic;
use crate::logic::island_core::sprite_logic;
use crate::logic::island_core::sprite_logic::SpriteLogic;
use pg_engine::display;
use pg_engine::logic;

/// `ApeAltarLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApeAltarLogic {
    /// `mApeAltarLogicListener`: `@"<ApeAltarLogicListener>"`
    pub ape_altar_logic_listener: Option<Entity>,

    /// `mPygmyHeadDisplayObject`: `@"BCMultiModel"`
    pub pygmy_head_display_object: Option<Entity>,

    /// `mPygmyRightHandDisplayObject`: `@"BCMultiModel"`
    pub pygmy_right_hand_display_object: Option<Entity>,

    /// `mPygmyLeftHandDisplayObject`: `@"BCMultiModel"`
    pub pygmy_left_hand_display_object: Option<Entity>,

    /// `mPygmyRightLegDisplayObject`: `@"BCMultiModel"`
    pub pygmy_right_leg_display_object: Option<Entity>,

    /// `mPygmyLeftLegDisplayObject`: `@"BCMultiModel"`
    pub pygmy_left_leg_display_object: Option<Entity>,

    /// `mPygmyHeadLogic`: `@"MultiModelLogic"`
    pub pygmy_head_logic: Option<Entity>,

    /// `mPygmyRightHandLogic`: `@"MultiModelLogic"`
    pub pygmy_right_hand_logic: Option<Entity>,

    /// `mPygmyLeftHandLogic`: `@"MultiModelLogic"`
    pub pygmy_left_hand_logic: Option<Entity>,

    /// `mPygmyRightLegLogic`: `@"MultiModelLogic"`
    pub pygmy_right_leg_logic: Option<Entity>,

    /// `mPygmyLeftLegLogic`: `@"MultiModelLogic"`
    pub pygmy_left_leg_logic: Option<Entity>,

    /// `mPygmyRightHandPosVel`: `{?="x"f"y"f"z"f"w"f}`
    pub pygmy_right_hand_pos_vel: Vec4,

    /// `mPygmyLeftHandPosVel`: `{?="x"f"y"f"z"f"w"f}`
    pub pygmy_left_hand_pos_vel: Vec4,

    /// `mPygmyRightLegPosVel`: `{?="x"f"y"f"z"f"w"f}`
    pub pygmy_right_leg_pos_vel: Vec4,

    /// `mPygmyLeftLegPosVel`: `{?="x"f"y"f"z"f"w"f}`
    pub pygmy_left_leg_pos_vel: Vec4,

    /// `mbHeadPanic`: `B`
    pub head_panic: bool,

    /// `mbRetracting`: `B`
    pub retracting: bool,

    /// `mbStretching`: `B`
    pub stretching: bool,

    /// `mbBreakPygmy`: `B`
    pub break_pygmy: bool,

    /// `mbMovePygmyLimbs`: `B`
    pub move_pygmy_limbs: bool,

    /// `mApeAltarStretch`: `f`
    pub ape_altar_stretch: f32,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const APE_ALTAR_LOGIC: Class = Class {
    name: "ApeAltarLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The tweenable channel of the altar's animation that carries the stretched pygmy.
const PYGMY_CHANNEL: usize = 1;

/// The tweenable channel that carries the pygmy head multimodel.
const PYGMY_HEAD_CHANNEL: usize = 2;

/// Where the torn-off limbs start, relative to the altar: right hand, left hand, right leg,
/// left leg.
const LIMB_START_OFFSETS: [(f32, f32); 4] = [(52.0, 107.0), (-52.0, 107.0), (21.0, 42.0), (-21.0, 42.0)];

/// Sequence-item handlers implemented by `ApeAltarLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "breakPygmyComplete" => seq_break_pygmy_complete(world, this, call),
        "pygmyFallInsideComplete" => seq_pygmy_fall_inside_complete(world, this, call),
        "apeGrabPygmyComplete" => seq_ape_grab_pygmy_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ApeAltarLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APE_ALTAR_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[[BCMultiModel alloc] initWithId:id]` and a `MultiModelLogic` on it: one part of the altar
/// pygmy. Returns the display object and its logic.
fn spawn_pygmy_part(world: &mut World, id: &str) -> (Entity, Entity) {
    let part_display_object = pg_engine::display::multi_model::spawn(world, Some(id));
    let part_logic = pg_engine::logic::multi_model::spawn(world, part_display_object);
    (part_display_object, part_logic)
}

/// `[mApeAltarLogicListener apeWenchLogic:self]` (nil for a nil listener).
fn listener_ape_wench_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<ApeAltarLogic>(this).ape_altar_logic_listener;
    let listener = listener?;
    crate::controller::ape_wench_logic(world, listener, this)
}

/// The four limb logics in the original's order: right hand, left hand, right leg, left leg.
fn limb_logics(world: &World, this: Entity) -> [Option<Entity>; 4] {
    let altar = world.obj::<ApeAltarLogic>(this);
    [
        altar.pygmy_right_hand_logic,
        altar.pygmy_left_hand_logic,
        altar.pygmy_right_leg_logic,
        altar.pygmy_left_leg_logic,
    ]
}

/// The four limb velocities, in the order of [`limb_logics`].
fn limb_pos_vels(world: &World, this: Entity) -> [Vec4; 4] {
    let altar = world.obj::<ApeAltarLogic>(this);
    [
        altar.pygmy_right_hand_pos_vel,
        altar.pygmy_left_hand_pos_vel,
        altar.pygmy_right_leg_pos_vel,
        altar.pygmy_left_leg_pos_vel,
    ]
}

/// Stores the four limb velocities, given in the order of [`limb_logics`].
fn set_limb_pos_vels(world: &mut World, this: Entity, pos_vels: [Vec4; 4]) {
    let [right_hand_pos_vel, left_hand_pos_vel, right_leg_pos_vel, left_leg_pos_vel] = pos_vels;

    let mut altar = world.obj_mut::<ApeAltarLogic>(this);
    altar.pygmy_right_hand_pos_vel = right_hand_pos_vel;
    altar.pygmy_left_hand_pos_vel = left_hand_pos_vel;
    altar.pygmy_right_leg_pos_vel = right_leg_pos_vel;
    altar.pygmy_left_leg_pos_vel = left_leg_pos_vel;
}

/// A torn-off limb's launch velocity: `((int)(arc4random() % 4001 - 2000) / 10, 500, 0, 1)`.
fn random_limb_pos_vel(world: &mut World) -> Vec4 {
    let roll = world.resource_mut::<Random>().arc4random_mod(4001);

    // Unsigned wrap-around, then read as signed.
    let spread = roll.wrapping_sub(2000) as i32;
    let x_vel = spread as f32 / 10.0;
    Vec4::new(x_vel, 500.0, 0.0, 1.0)
}

/// Takes the four limbs out of the display tree and blanks the head and limb animations
/// (`exitIsland:` and `onGameFrame:` end the tear-apart the same way).
fn remove_pygmy_parts(world: &mut World, this: Entity) {
    let limb_logics = limb_logics(world, this);
    for limb_logic in limb_logics.into_iter().flatten() {
        let limb_display_object = logic::display_object(world, limb_logic);
        display::remove_from_display_group(world, limb_display_object);
    }

    let head_logic = world.obj::<ApeAltarLogic>(this).pygmy_head_logic;
    if let Some(head_logic) = head_logic {
        logic::set_behavior(world, head_logic, Some("BlankAnim"));
    }
    for limb_logic in limb_logics.into_iter().flatten() {
        logic::set_behavior(world, limb_logic, Some("BlankAnim"));
    }
}

/// `-[ApeAltarLogic initWithDisplayObject:]` @ 0x10017cfc4
///
/// Also builds the altar pygmy's head (carried on the altar's channel 2) and the four limbs
/// that fly off when the pygmy is torn apart.
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApeAltarLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_style_id = Some(String::from("ApeAltar"));

    let (head_display_object, head_logic) = spawn_pygmy_part(world, "ApeAltarPygmyHead");
    {
        let mut altar = world.obj_mut::<ApeAltarLogic>(this);
        altar.pygmy_head_display_object = Some(head_display_object);
        altar.pygmy_head_logic = Some(head_logic);
    }
    let animation = logic::animation(world, this);
    pg_engine::animation::set_tweenable(world, animation, Some(head_display_object), PYGMY_HEAD_CHANNEL);

    let (right_hand_display_object, right_hand_logic) = spawn_pygmy_part(world, "ApeAltarPygmyRightHand");
    let (left_hand_display_object, left_hand_logic) = spawn_pygmy_part(world, "ApeAltarPygmyLeftHand");
    let (right_leg_display_object, right_leg_logic) = spawn_pygmy_part(world, "ApeAltarPygmyRightLeg");
    let (left_leg_display_object, left_leg_logic) = spawn_pygmy_part(world, "ApeAltarPygmyLeftLeg");
    {
        let mut altar = world.obj_mut::<ApeAltarLogic>(this);
        altar.pygmy_right_hand_display_object = Some(right_hand_display_object);
        altar.pygmy_right_hand_logic = Some(right_hand_logic);
        altar.pygmy_left_hand_display_object = Some(left_hand_display_object);
        altar.pygmy_left_hand_logic = Some(left_hand_logic);
        altar.pygmy_right_leg_display_object = Some(right_leg_display_object);
        altar.pygmy_right_leg_logic = Some(right_leg_logic);
        altar.pygmy_left_leg_display_object = Some(left_leg_display_object);
        altar.pygmy_left_leg_logic = Some(left_leg_logic);
    }

    logic::set_behavior(world, this, Some("ApeAltarExitIsland"));
}

/// `-[ApeAltarLogic setApeAltarLogicListener:]` @ 0x10017d260
pub fn set_ape_altar_logic_listener(world: &mut World, this: Entity, ape_altar_logic_listener: Entity) {
    world.obj_mut::<ApeAltarLogic>(this).ape_altar_logic_listener = Some(ape_altar_logic_listener);
}

/// `-[ApeAltarLogic isAvailable]` @ 0x10017d2bc
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let altar = world.obj::<ApeAltarLogic>(this);
    altar.pygmy_logic.is_none() && !altar.retracting && !altar.break_pygmy
}

/// `-[ApeAltarLogic canStretch]` @ 0x10017d2fc
pub fn can_stretch(world: &mut World, this: Entity) -> bool {
    world.obj::<ApeAltarLogic>(this).pygmy_logic.is_some()
}

/// `-[ApeAltarLogic altarPygmyLogic]` @ 0x10017d314
pub fn altar_pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ApeAltarLogic>(this).pygmy_logic
}

/// `-[ApeAltarLogic isPygmyAvailableForApe]` @ 0x10017d324
pub fn is_pygmy_available_for_ape(world: &mut World, this: Entity) -> bool {
    let altar = world.obj::<ApeAltarLogic>(this);
    altar.ape_altar_stretch >= 1.0 && altar.pygmy_logic.is_some()
}

/// `-[ApeAltarLogic isAtlarFullyRetracted]` @ 0x10017d35c
pub fn is_atlar_fully_retracted(world: &mut World, this: Entity) -> bool {
    world.obj::<ApeAltarLogic>(this).ape_altar_stretch == 0.0
}

/// `-[ApeAltarLogic apeGrabPygmy]` @ 0x10017d374
pub fn ape_grab_pygmy(world: &mut World, this: Entity) {
    {
        let mut altar = world.obj_mut::<ApeAltarLogic>(this);
        altar.head_panic = false;
        altar.retracting = false;
        altar.stretching = false;
        altar.break_pygmy = true;
    }

    let pygmy = world.obj::<ApeAltarLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::ape_owns_pygmy(world, pygmy);
    }
    logic::set_behavior(world, this, Some("ApeAltarApeGrabPygmy"));
}

/// `-[ApeAltarLogic retractApeAltar]` @ 0x10017d3f8
pub fn retract_ape_altar(world: &mut World, this: Entity) {
    {
        let mut altar = world.obj_mut::<ApeAltarLogic>(this);
        altar.head_panic = false;
        altar.stretching = false;
        altar.retracting = true;
    }
    logic::set_behavior(world, this, Some("ApeAltarRetract"));
}

/// `-[ApeAltarLogic setApeAltarDeltaStretch:]` @ 0x10017d438
///
/// The wench's crank stretches the altar pygmy (`stretch -= 0.03 · delta`, never below 0, and
/// the animation follows the stretch). Past 1.2 the pygmy is torn apart and its four limbs fly
/// off; that returns YES.
pub fn set_ape_altar_delta_stretch(world: &mut World, this: Entity, ape_altar_delta_stretch: f32) -> bool {
    let break_pygmy = world.obj::<ApeAltarLogic>(this).break_pygmy;
    if break_pygmy {
        return false;
    }

    let mut stretch = world.obj::<ApeAltarLogic>(this).ape_altar_stretch;
    if stretch > 1.0 {
        let idol_manager = crate::managers::idol_manager::instance(world);
        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::trigger_episode_event(world, idol_manager, "WindPygmyOnAltar");
        }
        stretch = world.obj::<ApeAltarLogic>(this).ape_altar_stretch;
    }

    // `fcsel`: anything not above zero, NaN included, becomes 0.
    let stretched = stretch + ape_altar_delta_stretch * -0.03;
    let stretch = if stretched > 0.0 { stretched } else { 0.0 };
    world.obj_mut::<ApeAltarLogic>(this).ape_altar_stretch = stretch;

    let stretching = world.obj::<ApeAltarLogic>(this).stretching;
    if !stretching {
        world.obj_mut::<ApeAltarLogic>(this).stretching = true;
        logic::set_behavior(world, this, Some("ApeAltarStretch"));
    }

    let altar = world.obj::<ApeAltarLogic>(this);
    let head_panic = altar.head_panic;
    let stretch = altar.ape_altar_stretch;
    if !head_panic && stretch > 0.0 {
        world.obj_mut::<ApeAltarLogic>(this).head_panic = true;
        let head_logic = world.obj::<ApeAltarLogic>(this).pygmy_head_logic;
        if let Some(head_logic) = head_logic {
            logic::set_behavior(world, head_logic, Some("ApeAltarPygmyHeadPanic"));
        }
    }

    let stretch = world.obj::<ApeAltarLogic>(this).ape_altar_stretch;
    if stretch > 1.2 {
        tear_pygmy_apart(world, this);
        return true;
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::set_frame_percent(world, animation, stretch);
    false
}

/// The `stretch > 1.2` branch of `setApeAltarDeltaStretch:`: the wench lets go, the pygmy is
/// torn apart, and its four limbs fly off from around the altar, after the drag marker.
fn tear_pygmy_apart(world: &mut World, this: Entity) {
    {
        let mut altar = world.obj_mut::<ApeAltarLogic>(this);
        altar.head_panic = false;
        altar.retracting = false;
        altar.stretching = false;
        altar.break_pygmy = true;
        altar.ape_altar_stretch = 0.0;
    }

    let ape_wench = listener_ape_wench_logic(world, this);
    if let Some(ape_wench) = ape_wench {
        ape_wench_logic::retract_ape_wench(world, ape_wench);
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

    let pygmy = world.obj::<ApeAltarLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::torn_apart_from_ape_altar(world, pygmy);
    }
    {
        let mut altar = world.obj_mut::<ApeAltarLogic>(this);
        altar.pygmy_logic = None;
        altar.move_pygmy_limbs = true;
    }

    let head_logic = world.obj::<ApeAltarLogic>(this).pygmy_head_logic;
    if let Some(head_logic) = head_logic {
        logic::set_behavior(world, head_logic, Some("BlankAnim"));
    }

    let limb_logics = limb_logics(world, this);
    let drag_display_marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    for limb_logic in limb_logics.into_iter().flatten() {
        let limb_display_object = logic::display_object(world, limb_logic);
        if let Some(drag_display_marker) = drag_display_marker {
            display::insert_after(world, drag_display_marker, Some(limb_display_object));
        }
    }

    let display_object = logic::display_object(world, this);
    let altar_pos = display::object(world, display_object).pos;
    for (limb_logic, (x_offset, y_offset)) in limb_logics.into_iter().zip(LIMB_START_OFFSETS) {
        let Some(limb_logic) = limb_logic else {
            continue;
        };
        let limb_display_object = logic::display_object(world, limb_logic);
        let mut limb_display = display::object_mut(world, limb_display_object);
        limb_display.pos.x = altar_pos.x + x_offset;
        limb_display.pos.y = altar_pos.y + y_offset;
        limb_display.pos.z = altar_pos.z;
    }

    let right_hand_pos_vel = random_limb_pos_vel(world);
    let left_hand_pos_vel = random_limb_pos_vel(world);
    let right_leg_pos_vel = random_limb_pos_vel(world);
    let left_leg_pos_vel = random_limb_pos_vel(world);
    let limb_pos_vels = [right_hand_pos_vel, left_hand_pos_vel, right_leg_pos_vel, left_leg_pos_vel];
    set_limb_pos_vels(world, this, limb_pos_vels);

    for limb_logic in limb_logics.into_iter().flatten() {
        logic::set_behavior(world, limb_logic, Some("ApeAltarPygmySpinningArm"));
    }
    logic::set_behavior(world, this, Some("ApeAltarBreakPygmy"));
}

/// `-[ApeAltarLogic stopStretchingApeAltar]` @ 0x10017da40
pub fn stop_stretching_ape_altar(world: &mut World, this: Entity) {
    world.obj_mut::<ApeAltarLogic>(this).head_panic = false;

    let head_logic = world.obj::<ApeAltarLogic>(this).pygmy_head_logic;
    if let Some(head_logic) = head_logic {
        logic::set_behavior(world, head_logic, Some("ApeAltarPygmyHeadIdle"));
    }
}

/// `-[ApeAltarLogic pygmyFallIntoApeAltar:]` @ 0x10017da70
pub fn pygmy_fall_into_ape_altar(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let current_pygmy = world.obj::<ApeAltarLogic>(this).pygmy_logic;
    if current_pygmy.is_some() {
        return;
    }

    {
        let mut altar = world.obj_mut::<ApeAltarLogic>(this);
        altar.head_panic = false;
        altar.stretching = false;
        altar.ape_altar_stretch = 0.0;
        altar.pygmy_logic = Some(pygmy_logic);
    }

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("ApeAltarPygmyFallInside"));
}

/// `-[ApeAltarLogic enterIsland:]` @ 0x10017db28
///
/// The startup flag is unused.
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    {
        let mut altar = world.obj_mut::<ApeAltarLogic>(this);
        altar.head_panic = false;
        altar.retracting = false;
        altar.stretching = false;
        altar.break_pygmy = false;
    }

    logic::start_game_frame(world, this);
    logic::set_behavior(world, this, Some("ApeAltarEnterIsland"));
    sprite_logic::show_apocalypse_cloud(world, this);
}

/// `-[ApeAltarLogic removePygmyFromApeAltar:]` @ 0x10017dbb0
///
/// A pygmy leaving mid-stretch lets the wench go; the altar springs back (`…Snap` when it was
/// stretched past 0.3).
pub fn remove_pygmy_from_ape_altar(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let current_pygmy = world.obj::<ApeAltarLogic>(this).pygmy_logic;
    if current_pygmy != Some(pygmy_logic) {
        return;
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    world.obj_mut::<ApeAltarLogic>(this).pygmy_logic = None;
    logic::set_behavior(world, this, Some("ApeAltarEnterIsland"));

    let stretching = world.obj::<ApeAltarLogic>(this).stretching;
    if !stretching {
        return;
    }

    {
        let mut altar = world.obj_mut::<ApeAltarLogic>(this);
        altar.head_panic = false;
        altar.stretching = false;
    }

    let ape_wench = listener_ape_wench_logic(world, this);
    if let Some(ape_wench) = ape_wench {
        ape_wench_logic::retract_ape_wench(world, ape_wench);
    }

    let stretch = world.obj::<ApeAltarLogic>(this).ape_altar_stretch;
    let behavior = if stretch > 0.3 {
        "ApeAltarRetractNoPygmySnap"
    } else {
        world.obj_mut::<ApeAltarLogic>(this).retracting = true;
        "ApeAltarRetractNoPygmy"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[ApeAltarLogic exitIsland:]` @ 0x10017dccc
///
/// `now` also stops the game frame. Limbs still in flight keep `mbMovePygmyLimbs` set.
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let pygmy = world.obj::<ApeAltarLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::toss_from_ape_altar(world, pygmy);
        world.obj_mut::<ApeAltarLogic>(this).pygmy_logic = None;
    }

    if now {
        logic::stop_game_frame(world, this);
    }

    logic::set_behavior(world, this, Some("ApeAltarExitIsland"));
    sprite_logic::hide_apocalypse_cloud(world, this);
    remove_pygmy_parts(world, this);
}

/// `-[ApeAltarLogic onGameFrame:]` @ 0x10017e078
///
/// Winds a retracting altar back (0.06 per frame) and lets the torn-off limbs fall until all
/// four are below y = −100. `SpriteLogic`'s physics does not run.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let retracting = world.obj::<ApeAltarLogic>(this).retracting;
    if retracting {
        let stretch = world.obj::<ApeAltarLogic>(this).ape_altar_stretch - 0.06;
        world.obj_mut::<ApeAltarLogic>(this).ape_altar_stretch = stretch;

        if stretch < 0.0 {
            world.obj_mut::<ApeAltarLogic>(this).retracting = false;

            let pygmy = world.obj::<ApeAltarLogic>(this).pygmy_logic;
            if pygmy.is_none() {
                logic::set_behavior(world, this, Some("ApeAltarEnterIsland"));
            }
        } else {
            let animation = logic::animation(world, this);
            pg_engine::animation::set_frame_percent(world, animation, stretch);
        }
    }

    let move_pygmy_limbs = world.obj::<ApeAltarLogic>(this).move_pygmy_limbs;
    if !move_pygmy_limbs {
        return;
    }

    // Gravity (0, −3000, 0), applied to the velocity first.
    let x_vel_step = dt * 0.0;
    let y_vel_step = dt * -3000.0;
    let mut limb_pos_vels = limb_pos_vels(world, this);
    for limb_pos_vel in &mut limb_pos_vels {
        limb_pos_vel.x = x_vel_step + limb_pos_vel.x;
        limb_pos_vel.y = y_vel_step + limb_pos_vel.y;
        limb_pos_vel.z = x_vel_step + limb_pos_vel.z;
    }
    set_limb_pos_vels(world, this, limb_pos_vels);

    let limb_logics = limb_logics(world, this);
    let mut limbs_below_island = true;
    for (limb_logic, limb_pos_vel) in limb_logics.into_iter().zip(limb_pos_vels) {
        let Some(limb_logic) = limb_logic else {
            continue;
        };
        let limb_display_object = logic::display_object(world, limb_logic);
        let mut limb_display = display::object_mut(world, limb_display_object);
        limb_display.pos.x = limb_display.pos.x + limb_pos_vel.x * dt;
        limb_display.pos.y = limb_display.pos.y + limb_pos_vel.y * dt;
        limb_display.pos.z = limb_display.pos.z + limb_pos_vel.z * dt;
        limbs_below_island &= limb_display.pos.y < -100.0;
    }
    if !limbs_below_island {
        return;
    }

    world.obj_mut::<ApeAltarLogic>(this).move_pygmy_limbs = false;
    remove_pygmy_parts(world, this);
}

/// `-[ApeAltarLogic breakPygmyComplete:::]` @ 0x10017e4f4
pub fn seq_break_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApeAltarLogic>(this).break_pygmy = false;
    ItemResult::Done
}

/// `-[ApeAltarLogic pygmyFallInsideComplete:::]` @ 0x10017e508
pub fn seq_pygmy_fall_inside_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let head_logic = world.obj::<ApeAltarLogic>(this).pygmy_head_logic;
    if let Some(head_logic) = head_logic {
        logic::set_behavior(world, head_logic, Some("ApeAltarPygmyHeadIdle"));
    }
    ItemResult::Done
}

/// `-[ApeAltarLogic apeGrabPygmyComplete:::]` @ 0x10017e540
pub fn seq_ape_grab_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let ape_wench = listener_ape_wench_logic(world, this);
    if let Some(ape_wench) = ape_wench {
        ape_wench_logic::retract_ape_wench(world, ape_wench);
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, PYGMY_CHANNEL);

    let pygmy = world.obj::<ApeAltarLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::grabbed_from_ape_altar(world, pygmy);
    }
    world.obj_mut::<ApeAltarLogic>(this).pygmy_logic = None;
    ItemResult::Done
}
