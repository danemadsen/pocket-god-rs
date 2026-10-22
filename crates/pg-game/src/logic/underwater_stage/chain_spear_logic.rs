//! `ChainSpearLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::underwater_stage::spear_statue_logic;
use pg_engine::animation::Timelines;
use pg_engine::{animation, display, logic};

/// `ChainSpearLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ChainSpearLogic {
    /// `mChainSpearLogicListener`: `@"<ChainSpearLogicListener>"`
    pub chain_spear_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbInAir`: `B`
    pub in_air: bool,

    /// `mbExtending`: `B`
    pub extending: bool,

    /// `mbShooting`: `B`
    pub shooting: bool,

    /// `mSpearBaseXPos`: `f`
    pub spear_base_x_pos: f32,

    /// `mSpearBaseYPos`: `f`
    pub spear_base_y_pos: f32,

    /// `mLastSpearTipXPos`: `f`
    pub last_spear_tip_x_pos: f32,

    /// `mLastSpearTipYPos`: `f`
    pub last_spear_tip_y_pos: f32,

    /// `mSpearLineAB`: `{?="x"f"y"f"z"f"w"f}`
    pub spear_line_ab: Vec4,

    /// `mSpearLineBA`: `{?="x"f"y"f"z"f"w"f}`
    pub spear_line_ba: Vec4,

    /// `mLengthAB`: `f`
    pub length_ab: f32,

    /// `mSkeweredPygmyLogicArray`: `@"NSMutableArray"`
    pub skewered_pygmy_logic_array: Vec<Entity>,
}

pub const CHAIN_SPEAR_LOGIC: Class = Class {
    name: "ChainSpearLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The channel each skewered pygmy rides on, by how many were skewered before it
/// (`DAT_10029d620`); the seventh and later share channel 1.
const SKEWER_CHANNELS: [usize; 6] = [1, 2, 3, 4, 5, 6];

/// The spear tip's state in every frame of the chain spear's animations: the seventh from the
/// top of the draw order (`Spear` in `ChainSpearShoot`).
const SPEAR_TIP_STATE_FROM_TOP: usize = 7;

/// Added to the tip state's x to reach the spear's point.
const SPEAR_TIP_OFFSET: f32 = 120.0;

/// A pygmy's display position sits this far below the point the spear hits.
const PYGMY_HIT_Y_OFFSET: f32 = 32.0;

/// How close to the spear line a pygmy must be to be skewered.
const SKEWER_DISTANCE: f32 = 20.0;

/// Sequence-item handlers implemented by `ChainSpearLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "onShootComplete" => seq_on_shoot_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `ChainSpearLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animExtended" => anim_extended(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[ChainSpearLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&CHAIN_SPEAR_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[ChainSpearLogic initWithDisplayObject:]` @ 0x100061cf4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ChainSpearLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<ChainSpearLogic>(this).skewered_pygmy_logic_array = Vec::with_capacity(5);
}

/// `-[ChainSpearLogic setChainSpearLogicListener:]` @ 0x100061d7c
pub fn set_chain_spear_logic_listener(world: &mut World, this: Entity, chain_spear_logic_listener: Entity) {
    world.obj_mut::<ChainSpearLogic>(this).chain_spear_logic_listener = Some(chain_spear_logic_listener);
}

/// `-[ChainSpearLogic dealloc]` @ 0x10006265c
///
/// Called by the statue, which holds the last reference. The listener and the skewered array
/// go away with the component; `[super dealloc]`: `SpriteLogic` only releases its cloud style
/// string.
pub fn dealloc(world: &mut World, this: Entity) {
    logic::dealloc(world, this);
}

/// `-[ChainSpearLogic onGameFrame:]` @ 0x100061dd8
///
/// While the spear extends, the segment between the last two tip positions skewers every pygmy
/// it passes through (no `[super onGameFrame:]`: the spear itself never moves).
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let own_display_object = logic::display_object(world, this);
    let frame = display::multi_model::binary_frame_def(world, own_display_object);
    let Some(frame) = frame else {
        return;
    };
    if !world.obj::<ChainSpearLogic>(this).extending {
        return;
    }

    let tip_state_x = {
        let timelines = world.resource::<Timelines>();
        let states = timelines.frame_states(frame);
        let tip_state = states[states.len() - SPEAR_TIP_STATE_FROM_TOP];
        timelines.state(tip_state).x
    };

    let (pos, z_rot) = {
        let object = display::object(world, own_display_object);
        (object.pos, object.rot.z)
    };
    let angle = pg_engine::math::deg_to_rad(z_rot);
    let (sin, cos) = angle.sin_cos();
    let reach = tip_state_x + SPEAR_TIP_OFFSET;
    let tip_x_pos = pos.x + cos * reach;
    let tip_y_pos = pos.y + sin * reach;

    {
        let mut spear = world.obj_mut::<ChainSpearLogic>(this);
        if spear.last_spear_tip_x_pos != tip_x_pos && spear.last_spear_tip_y_pos != tip_y_pos {
            spear.spear_base_x_pos = spear.last_spear_tip_x_pos;
            spear.spear_base_y_pos = spear.last_spear_tip_y_pos;

            let line_ab_x = tip_x_pos - spear.spear_base_x_pos;
            let line_ab_y = tip_y_pos - spear.spear_base_y_pos;
            spear.spear_line_ab = Vec4::new(line_ab_x, line_ab_y, 0.0, 1.0);

            let line_ba_x = spear.spear_base_x_pos - tip_x_pos;
            let line_ba_y = spear.spear_base_y_pos - tip_y_pos;
            spear.spear_line_ba = Vec4::new(line_ba_x, line_ba_y, 0.0, 1.0);

            let line_ab = spear.spear_line_ab;
            spear.length_ab = (line_ab.x * line_ab.x + line_ab.y * line_ab.y).sqrt();
        }
        spear.last_spear_tip_x_pos = tip_x_pos;
        spear.last_spear_tip_y_pos = tip_y_pos;
    }

    let listener = world.obj::<ChainSpearLogic>(this).chain_spear_logic_listener;
    let pygmies = match listener {
        Some(listener) => spear_statue_logic::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };

    for pygmy in pygmies {
        let pygmy_display_object = logic::display_object(world, pygmy);
        if !crate::pygmy::can_be_hit_by_chain_spear(world, pygmy, this) {
            continue;
        }

        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let spear = world.obj::<ChainSpearLogic>(this);
        let line_ab = spear.spear_line_ab;
        let line_ba = spear.spear_line_ba;

        // The hit point must lie between the two ends (both projections ≤ 0) and within 20 of
        // the line through them.
        let from_tip_x = pygmy_pos.x - tip_x_pos;
        let from_tip_y = (pygmy_pos.y + PYGMY_HIT_Y_OFFSET) - tip_y_pos;
        let tip_projection = from_tip_x * line_ab.x + from_tip_y * line_ab.y;
        if !(tip_projection <= 0.0) {
            continue;
        }

        let from_base_x = pygmy_pos.x - spear.spear_base_x_pos;
        let from_base_y = (pygmy_pos.y + PYGMY_HIT_Y_OFFSET) - spear.spear_base_y_pos;
        let base_projection = from_base_x * line_ba.x + from_base_y * line_ba.y;
        if !(base_projection <= 0.0) {
            continue;
        }

        let cross = line_ab.x * from_tip_y - from_tip_x * line_ab.y;
        let line_distance = (cross / spear.length_ab).abs();
        if !(line_distance < SKEWER_DISTANCE) {
            continue;
        }

        let skewered_count = spear.skewered_pygmy_logic_array.len();
        let channel = match SKEWER_CHANNELS.get(skewered_count) {
            Some(channel) => *channel,
            None => 1,
        };

        let own_animation = logic::animation(world, this);
        let hit_display_object = logic::display_object(world, pygmy);
        animation::set_tweenable(world, own_animation, Some(hit_display_object), channel);
        world.obj_mut::<ChainSpearLogic>(this).skewered_pygmy_logic_array.push(pygmy);
        crate::pygmy::hit_by_chain_spear(world, pygmy, this);
    }
}

/// `-[ChainSpearLogic animExtended]` @ 0x1000622d8
///
/// Fully extended: the spear pulls back with its catch.
pub fn anim_extended(world: &mut World, this: Entity) {
    world.obj_mut::<ChainSpearLogic>(this).extending = false;

    let skewered_pygmies = world.obj::<ChainSpearLogic>(this).skewered_pygmy_logic_array.clone();
    for pygmy in skewered_pygmies {
        crate::pygmy::hit_by_chain_spear_retract(world, pygmy, this);
    }
}

/// `-[ChainSpearLogic activate]` @ 0x100062400
pub fn activate(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("ChainSpearInit"));
}

/// `-[ChainSpearLogic clearSkeweredPygmies]` @ 0x100062418
///
/// Lets go of every skewered pygmy (channels 1–6), reports the catch and kills them.
pub fn clear_skewered_pygmies(world: &mut World, this: Entity) {
    world.obj_mut::<ChainSpearLogic>(this).shooting = false;

    let own_animation = logic::animation(world, this);
    for channel in SKEWER_CHANNELS {
        animation::clear_tweenable_channel_id_now(world, own_animation, channel);
    }

    let (listener, skewered_count) = {
        let spear = world.obj::<ChainSpearLogic>(this);
        (spear.chain_spear_logic_listener, spear.skewered_pygmy_logic_array.len())
    };
    if let Some(listener) = listener {
        spear_statue_logic::on_pygmies_skewered(world, listener, skewered_count as i32);
    }

    let skewered_pygmies = world.obj::<ChainSpearLogic>(this).skewered_pygmy_logic_array.clone();
    for pygmy in skewered_pygmies {
        crate::pygmy::hit_by_chain_spear_complete(world, pygmy, this);
    }
    world.obj_mut::<ChainSpearLogic>(this).skewered_pygmy_logic_array.clear();
}

/// `-[ChainSpearLogic shootSpear]` @ 0x100062604
pub fn shoot_spear(world: &mut World, this: Entity) {
    {
        let mut spear = world.obj_mut::<ChainSpearLogic>(this);
        if spear.shooting {
            return;
        }
        spear.shooting = true;
        spear.extending = true;
        spear.last_spear_tip_x_pos = 0.0;
        spear.last_spear_tip_y_pos = 0.0;
    }
    logic::set_behavior(world, this, Some("ChainSpearShoot"));
}

/// `-[ChainSpearLogic onShootComplete:::]` @ 0x1000626d8
pub fn seq_on_shoot_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    clear_skewered_pygmies(world, this);
    ItemResult::Done
}
