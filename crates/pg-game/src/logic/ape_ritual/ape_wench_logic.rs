//! `ApeWenchLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ape_ritual::ape_altar_logic;
use crate::logic::island_core::sprite_logic;
use crate::logic::island_core::sprite_logic::SpriteLogic;
use pg_engine::display;
use pg_engine::logic;

/// `ApeWenchLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApeWenchLogic {
    /// `mApeWenchLogicListener`: `@"<ApeWenchLogicListener>"`
    pub ape_wench_logic_listener: Option<Entity>,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbCanCrankWench`: `B`
    pub can_crank_wench: bool,

    /// `mbTurning`: `B`
    pub turning: bool,

    /// `mbWenchSoundOn`: `B`
    pub wench_sound_on: bool,

    /// `mbTurnToIdlePoint`: `B`
    pub turn_to_idle_point: bool,

    /// `mbStruggling`: `B`
    pub struggling: bool,

    /// `mbBeingSmashed`: `B`
    pub being_smashed: bool,

    /// `mWenchSoundFrames`: `i`
    pub wench_sound_frames: i32,

    /// `mApeWenchRot`: `f`
    pub ape_wench_rot: f32,

    /// `mLastApeWenchDeltaRot`: `f`
    pub last_ape_wench_delta_rot: f32,
}

pub const APE_WENCH_LOGIC: Class = Class {
    name: "ApeWenchLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The tweenable channel of the wench's animation that carries the hanging pygmy.
const PYGMY_CHANNEL: usize = 1;

/// Frames the crank sound keeps playing after the last turn.
const WENCH_SOUND_FRAMES: i32 = 5;

/// Sequence-item handlers implemented by `ApeWenchLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "pygmyFallInsideComplete" => seq_pygmy_fall_inside_complete(world, this, call),
        "retractComplete" => seq_retract_complete(world, this, call),
        "struggleComplete" => seq_struggle_complete(world, this, call),
        "apeHitComplete" => seq_ape_hit_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ApeWenchLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APE_WENCH_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mApeWenchLogicListener apeAltarLogic:self]` (nil for a nil listener).
fn listener_ape_altar_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<ApeWenchLogic>(this).ape_wench_logic_listener;
    let listener = listener?;
    crate::controller::ape_altar_logic(world, listener, this)
}

/// The hanging pygmy struggles instead of the crank turning: the crank stops and goes back to
/// zero (both struggle paths of `setApeWenchDeltaRot:`).
fn start_struggling(world: &mut World, this: Entity) {
    {
        let mut wench = world.obj_mut::<ApeWenchLogic>(this);
        wench.turning = false;
        wench.struggling = true;
        wench.wench_sound_on = false;
        wench.ape_wench_rot = 0.0;
    }
    logic::set_behavior(world, this, Some("ApeWenchStruggle"));
}

/// `-[ApeWenchLogic initWithDisplayObject:]` @ 0x100089de4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApeWenchLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);
    world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_style_id = Some(String::from("ApeWench"));
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("ApeWenchExitIsland"));
}

/// `-[ApeWenchLogic setApeWenchLogicListener:]` @ 0x100089ea0
pub fn set_ape_wench_logic_listener(world: &mut World, this: Entity, ape_wench_logic_listener: Entity) {
    world.obj_mut::<ApeWenchLogic>(this).ape_wench_logic_listener = Some(ape_wench_logic_listener);
}

/// `-[ApeWenchLogic isAvailable]` @ 0x100089efc
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let wench = world.obj::<ApeWenchLogic>(this);
    wench.pygmy_logic.is_none() && !wench.being_smashed
}

/// `-[ApeWenchLogic canCrankWench]` @ 0x100089f2c
pub fn can_crank_wench(world: &mut World, this: Entity) -> bool {
    let wench = world.obj::<ApeWenchLogic>(this);
    wench.pygmy_logic.is_some() && wench.can_crank_wench && !wench.being_smashed
}

/// `-[ApeWenchLogic apeHitApeWench]` @ 0x100089f6c
pub fn ape_hit_ape_wench(world: &mut World, this: Entity) {
    let active = world.obj::<ApeWenchLogic>(this).active;
    if !active {
        return;
    }
    world.obj_mut::<ApeWenchLogic>(this).being_smashed = true;

    let pygmy = world.obj::<ApeWenchLogic>(this).pygmy_logic;
    if pygmy.is_none() {
        logic::set_behavior(world, this, Some("ApeWenchApeHit"));
        return;
    }

    // With the crank turned, the altar winds back.
    let ape_wench_rot = world.obj::<ApeWenchLogic>(this).ape_wench_rot;
    if ape_wench_rot > 0.0 {
        world.obj_mut::<ApeWenchLogic>(this).turning = false;

        let ape_altar = listener_ape_altar_logic(world, this);
        if let Some(ape_altar) = ape_altar {
            ape_altar_logic::retract_ape_altar(world, ape_altar);
        }
    }

    {
        let mut wench = world.obj_mut::<ApeWenchLogic>(this);
        wench.wench_sound_on = false;
        wench.can_crank_wench = false;
    }

    let pygmy = world.obj::<ApeWenchLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::ape_owns_pygmy(world, pygmy);
    }
    logic::set_behavior(world, this, Some("ApeWenchApeHitPygmy"));
}

/// `-[ApeWenchLogic retractApeWench]` @ 0x10008a074
pub fn retract_ape_wench(world: &mut World, this: Entity) {
    {
        let mut wench = world.obj_mut::<ApeWenchLogic>(this);
        wench.turning = false;
        wench.wench_sound_on = false;
        wench.can_crank_wench = false;
    }
    logic::set_behavior(world, this, Some("ApeWenchRetract"));
}

/// `-[ApeWenchLogic setApeWenchDeltaRot:]` @ 0x10008a0b0
///
/// One step of the player's circular crank motion. The crank turns by `−0.165 · delta` and
/// stretches the altar pygmy by the same delta. With nothing on the altar, or with the altar
/// fully retracted when a forward turn completes a revolution, the hanging pygmy struggles
/// instead.
pub fn set_ape_wench_delta_rot(world: &mut World, this: Entity, ape_wench_delta_rot: f32) {
    let wench = world.obj::<ApeWenchLogic>(this);
    if !wench.can_crank_wench || wench.struggling {
        return;
    }

    world.obj_mut::<ApeWenchLogic>(this).turn_to_idle_point = false;
    let ape_altar = listener_ape_altar_logic(world, this);
    world.obj_mut::<ApeWenchLogic>(this).last_ape_wench_delta_rot = ape_wench_delta_rot;

    let can_stretch = ape_altar.is_some_and(|ape_altar| ape_altar_logic::can_stretch(world, ape_altar));
    if !can_stretch {
        let struggling = world.obj::<ApeWenchLogic>(this).struggling;
        if !struggling {
            start_struggling(world, this);
        }
        return;
    }

    let last_rot = world.obj::<ApeWenchLogic>(this).ape_wench_rot;
    let rot = last_rot + ape_wench_delta_rot * -0.165;
    world.obj_mut::<ApeWenchLogic>(this).ape_wench_rot = rot;

    let torn_apart = ape_altar.is_some_and(|ape_altar| {
        ape_altar_logic::set_ape_altar_delta_stretch(world, ape_altar, ape_wench_delta_rot)
    });
    if torn_apart {
        return;
    }

    // A forward step completes a revolution when `floorf` of the rotation changes (the binary
    // rounds with `frintm`, not by truncating to int).
    if ape_wench_delta_rot > 0.0 {
        let rot = world.obj::<ApeWenchLogic>(this).ape_wench_rot;
        let crossed_whole_turn = last_rot.floor() != rot.floor();
        if crossed_whole_turn {
            let altar_retracted =
                ape_altar.is_some_and(|ape_altar| ape_altar_logic::is_atlar_fully_retracted(world, ape_altar));
            let struggling = world.obj::<ApeWenchLogic>(this).struggling;
            if altar_retracted && !struggling {
                start_struggling(world, this);
                return;
            }
        }
    }

    world.obj_mut::<ApeWenchLogic>(this).wench_sound_frames = WENCH_SOUND_FRAMES;

    let turning = world.obj::<ApeWenchLogic>(this).turning;
    if !turning {
        {
            let mut wench = world.obj_mut::<ApeWenchLogic>(this);
            wench.turning = true;
            wench.wench_sound_on = true;
        }
        logic::set_behavior(world, this, Some("ApeWenchTurn"));
    } else {
        let wench_sound_on = world.obj::<ApeWenchLogic>(this).wench_sound_on;
        if !wench_sound_on {
            world.obj_mut::<ApeWenchLogic>(this).wench_sound_on = true;
            logic::set_behavior(world, this, Some("ApeWenchSoundOn"));
        }
    }

    let rot = world.obj::<ApeWenchLogic>(this).ape_wench_rot;
    let animation = logic::animation(world, this);
    pg_engine::animation::set_frame_percent(world, animation, rot);
}

/// `-[ApeWenchLogic stopTurningApeWench]` @ 0x10008a30c
///
/// The finger let go: `onGameFrame:` eases the crank to a whole turn.
pub fn stop_turning_ape_wench(world: &mut World, this: Entity) {
    let turning = world.obj::<ApeWenchLogic>(this).turning;
    if !turning {
        return;
    }
    world.obj_mut::<ApeWenchLogic>(this).turn_to_idle_point = true;

    let wench_sound_on = world.obj::<ApeWenchLogic>(this).wench_sound_on;
    if !wench_sound_on {
        world.obj_mut::<ApeWenchLogic>(this).wench_sound_on = true;
        logic::set_behavior(world, this, Some("ApeWenchSoundOn"));
    }

    let ape_altar = listener_ape_altar_logic(world, this);
    if let Some(ape_altar) = ape_altar {
        ape_altar_logic::stop_stretching_ape_altar(world, ape_altar);
    }
}

/// `-[ApeWenchLogic pygmyFallIntoApeWench:]` @ 0x10008a3b0
pub fn pygmy_fall_into_ape_wench(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let current_pygmy = world.obj::<ApeWenchLogic>(this).pygmy_logic;
    if current_pygmy.is_some() {
        return;
    }

    {
        let mut wench = world.obj_mut::<ApeWenchLogic>(this);
        wench.turning = false;
        wench.ape_wench_rot = 0.0;
        wench.pygmy_logic = Some(pygmy_logic);
    }

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("ApeWenchPygmyFallInside"));
}

/// `-[ApeWenchLogic enterIsland:]` @ 0x10008a45c
///
/// The startup flag is unused.
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    {
        let mut wench = world.obj_mut::<ApeWenchLogic>(this);
        wench.active = true;
        wench.being_smashed = false;
        wench.can_crank_wench = false;
    }

    logic::start_game_frame(world, this);
    logic::set_behavior(world, this, Some("ApeWenchEnterIsland"));
    sprite_logic::show_apocalypse_cloud(world, this);
}

/// `-[ApeWenchLogic removePygmyFromApeWench:]` @ 0x10008a4dc
///
/// The crank is locked whichever pygmy leaves. If the wench's own pygmy leaves with the crank
/// turned, the altar winds back.
pub fn remove_pygmy_from_ape_wench(world: &mut World, this: Entity, pygmy_logic: Entity) {
    world.obj_mut::<ApeWenchLogic>(this).can_crank_wench = false;

    let current_pygmy = world.obj::<ApeWenchLogic>(this).pygmy_logic;
    if current_pygmy != Some(pygmy_logic) {
        return;
    }

    world.obj_mut::<ApeWenchLogic>(this).pygmy_logic = None;
    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("ApeWenchEnterIsland"));

    let ape_wench_rot = world.obj::<ApeWenchLogic>(this).ape_wench_rot;
    if ape_wench_rot > 0.0 {
        {
            let mut wench = world.obj_mut::<ApeWenchLogic>(this);
            wench.turning = false;
            wench.wench_sound_on = false;
        }

        let ape_altar = listener_ape_altar_logic(world, this);
        if let Some(ape_altar) = ape_altar {
            ape_altar_logic::retract_ape_altar(world, ape_altar);
        }
        logic::set_behavior(world, this, Some("ApeWenchRetractNoPygmy"));
    }
}

/// `-[ApeWenchLogic exitIsland:]` @ 0x10008a5d8
///
/// `now` also stops the game frame.
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    {
        let mut wench = world.obj_mut::<ApeWenchLogic>(this);
        wench.active = false;
        wench.being_smashed = false;
        wench.wench_sound_on = false;
        wench.can_crank_wench = false;
    }

    let pygmy = world.obj::<ApeWenchLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::toss_from_ape_wench(world, pygmy);
        world.obj_mut::<ApeWenchLogic>(this).pygmy_logic = None;
    }

    if now {
        logic::stop_game_frame(world, this);
    }

    logic::set_behavior(world, this, Some("ApeWenchExitIsland"));
    sprite_logic::hide_apocalypse_cloud(world, this);
}

/// `-[ApeWenchLogic onGameFrame:]` @ 0x10008a724
///
/// After the finger lets go, the crank eases a fifth of the way per frame up to the next whole
/// turn (`ceilf`) and then idles. Otherwise the crank sound stops a few frames after the last
/// turn. `SpriteLogic`'s physics does not run.
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let wench = world.obj::<ApeWenchLogic>(this);
    let turn_to_idle_point = wench.turn_to_idle_point;
    let turning = wench.turning;
    let wench_sound_on = wench.wench_sound_on;

    if turn_to_idle_point && turning {
        let rot = world.obj::<ApeWenchLogic>(this).ape_wench_rot;
        let rot_to_whole_turn = rot.ceil() - rot;
        let scaled_rot_to_whole_turn = rot_to_whole_turn * 100.0;
        if scaled_rot_to_whole_turn.floor() == 0.0 {
            {
                let mut wench = world.obj_mut::<ApeWenchLogic>(this);
                wench.turning = false;
                wench.wench_sound_on = false;
                wench.turn_to_idle_point = false;
            }
            logic::set_behavior(world, this, Some("ApeWenchIdle"));
            return;
        }

        world.obj_mut::<ApeWenchLogic>(this).wench_sound_frames = WENCH_SOUND_FRAMES;
        let rot = world.obj::<ApeWenchLogic>(this).ape_wench_rot;
        let eased_rot = rot_to_whole_turn * 0.2 + rot;
        world.obj_mut::<ApeWenchLogic>(this).ape_wench_rot = eased_rot;

        let animation = logic::animation(world, this);
        pg_engine::animation::set_frame_percent(world, animation, eased_rot);
        return;
    }

    if !wench_sound_on {
        return;
    }

    let wench_sound_frames = world.obj::<ApeWenchLogic>(this).wench_sound_frames;
    world.obj_mut::<ApeWenchLogic>(this).wench_sound_frames = wench_sound_frames.wrapping_sub(1);
    if wench_sound_frames > 1 {
        return;
    }

    world.obj_mut::<ApeWenchLogic>(this).wench_sound_on = false;
    logic::set_behavior(world, this, Some("ApeWenchSoundOff"));
}

/// `-[ApeWenchLogic pygmyFallInsideComplete:::]` @ 0x10008a848
pub fn seq_pygmy_fall_inside_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApeWenchLogic>(this).can_crank_wench = true;
    ItemResult::Done
}

/// `-[ApeWenchLogic retractComplete:::]` @ 0x10008a860
pub fn seq_retract_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut wench = world.obj_mut::<ApeWenchLogic>(this);
    wench.can_crank_wench = true;
    wench.ape_wench_rot = 0.0;
    ItemResult::Done
}

/// `-[ApeWenchLogic struggleComplete:::]` @ 0x10008a884
pub fn seq_struggle_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApeWenchLogic>(this).struggling = false;
    ItemResult::Done
}

/// `-[ApeWenchLogic apeHitComplete:::]` @ 0x10008a898
pub fn seq_ape_hit_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApeWenchLogic>(this).being_smashed = false;

    let pygmy = world.obj::<ApeWenchLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::ape_smash_on_ape_wench(world, pygmy);

        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id(world, animation, PYGMY_CHANNEL);
        world.obj_mut::<ApeWenchLogic>(this).pygmy_logic = None;
    }
    ItemResult::Done
}
