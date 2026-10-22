//! `KonkeyDongLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::konkeydong::kong_statue_logic;

/// `KonkeyDongLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct KonkeyDongLogic {
    /// `mKonkeyDongLogicListener`: `@"<KonkeyDongLogicListener>"`
    pub konkey_dong_logic_listener: Option<Entity>,

    /// `mKidnapPygmyLogic`: `@"PygmyLogic"`
    pub kidnap_pygmy_logic: Option<Entity>,

    /// `mbHoldingKidnapedPygmy`: `B`
    pub holding_kidnaped_pygmy: bool,

    /// `mDifficulty`: `f`
    pub difficulty: f32,

    /// `mThrowBoulderTime`: `f`
    pub throw_boulder_time: f32,

    /// `mbEndLevel`: `B`
    pub end_level: bool,
}

pub const KONKEY_DONG_LOGIC: Class = Class {
    name: "KonkeyDongLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"KonkeyDongInit"`
const INIT_BEHAVIOR: &str = "KonkeyDongInit";

/// `@"KonkeyDongKidnapPygmy"`
const KIDNAP_PYGMY_BEHAVIOR: &str = "KonkeyDongKidnapPygmy";

/// `@"KonkeyDongCancelKidnapPygmy"`
const CANCEL_KIDNAP_PYGMY_BEHAVIOR: &str = "KonkeyDongCancelKidnapPygmy";

/// `@"KonkeyDongEndLevel"`
const END_LEVEL_BEHAVIOR: &str = "KonkeyDongEndLevel";

/// `@"KonkeyDongThrowBoulder"`
const THROW_BOULDER_BEHAVIOR: &str = "KonkeyDongThrowBoulder";

/// The pygmy's animation channel while Konkey Dong carries it.
const KIDNAP_PYGMY_CHANNEL: usize = 1;

/// Sequence-item handlers implemented by `KonkeyDongLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "kidnapPygmyComplete" => seq_kidnap_pygmy_complete(world, this, call),
        "cancelKidnapPygmyComplete" => seq_cancel_kidnap_pygmy_complete(world, this, call),
        "endLevelComplete" => seq_end_level_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `KonkeyDongLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animPygmyReaction" => anim_pygmy_reaction(world, this),
        "animResetIdol" => anim_reset_idol(world, this),
        "animThrowBoulder" => anim_throw_boulder(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[KonkeyDongLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&KONKEY_DONG_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[KonkeyDongLogic initWithDisplayObject:]` @ 0x10008fe28
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(KonkeyDongLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::set_behavior(world, this, Some(INIT_BEHAVIOR));
}

/// `-[KonkeyDongLogic setKonkeyDongLogicListener:]` @ 0x10008fe94
pub fn set_konkey_dong_logic_listener(world: &mut World, this: Entity, konkey_dong_logic_listener: Entity) {
    world.obj_mut::<KonkeyDongLogic>(this).konkey_dong_logic_listener = Some(konkey_dong_logic_listener);
}

/// `-[KonkeyDongLogic kidnapPygmy:]` @ 0x10008fef0
///
/// `kidnap_pygmy` is the statue's `mKidnapPygmyLogic`, which can be nil by then.
pub fn kidnap_pygmy(world: &mut World, this: Entity, kidnap_pygmy: Option<Entity>) {
    world.obj_mut::<KonkeyDongLogic>(this).kidnap_pygmy_logic = kidnap_pygmy;
    if let Some(pygmy) = kidnap_pygmy {
        crate::pygmy::kidnapped_by_konkey_dong(world, pygmy);
    }

    pg_engine::logic::set_behavior(world, this, Some(KIDNAP_PYGMY_BEHAVIOR));
    pg_engine::logic::stop_game_frame(world, this);
}

/// `-[KonkeyDongLogic cancelKidnapPygmy]` @ 0x10008ff58
pub fn cancel_kidnap_pygmy(world: &mut World, this: Entity) {
    let animation = pg_engine::logic::animation(world, this);
    let kidnap_pygmy_logic = world.obj::<KonkeyDongLogic>(this).kidnap_pygmy_logic;
    let pygmy_display_object = kidnap_pygmy_logic.map(|pygmy| pg_engine::logic::display_object(world, pygmy));
    pg_engine::animation::set_tweenable(world, animation, pygmy_display_object, KIDNAP_PYGMY_CHANNEL);

    if world.obj::<KonkeyDongLogic>(this).holding_kidnaped_pygmy {
        world.obj_mut::<KonkeyDongLogic>(this).holding_kidnaped_pygmy = false;
        if let Some(pygmy) = kidnap_pygmy_logic {
            crate::pygmy::kidnapped_by_konkey_dong_complete(world, pygmy);
        }
    }

    world.obj_mut::<KonkeyDongLogic>(this).kidnap_pygmy_logic = None;
    pg_engine::logic::set_behavior(world, this, Some(CANCEL_KIDNAP_PYGMY_BEHAVIOR));
}

/// `-[KonkeyDongLogic animPygmyReaction]` @ 0x100090008
pub fn anim_pygmy_reaction(world: &mut World, this: Entity) {
    let listener = world.obj::<KonkeyDongLogic>(this).konkey_dong_logic_listener;
    if let Some(listener) = listener {
        kong_statue_logic::start_pygmy_kidnap_reaction(world, listener);
    }
}

/// `-[KonkeyDongLogic animResetIdol]` @ 0x100090024
///
/// Konkey Dong takes the pygmy out of the statue: the pygmy moves to its own animation channel
/// and the statue resets.
pub fn anim_reset_idol(world: &mut World, this: Entity) {
    world.obj_mut::<KonkeyDongLogic>(this).holding_kidnaped_pygmy = true;

    let animation = pg_engine::logic::animation(world, this);
    let kidnap_pygmy_logic = world.obj::<KonkeyDongLogic>(this).kidnap_pygmy_logic;
    let pygmy_display_object = kidnap_pygmy_logic.map(|pygmy| pg_engine::logic::display_object(world, pygmy));
    pg_engine::animation::set_tweenable(world, animation, pygmy_display_object, KIDNAP_PYGMY_CHANNEL);

    let listener = world.obj::<KonkeyDongLogic>(this).konkey_dong_logic_listener;
    if let Some(listener) = listener {
        kong_statue_logic::kidnap_pygmy(world, listener);
    }
}

/// `-[KonkeyDongLogic kidnapPygmyComplete:::]` @ 0x1000900b0
pub fn seq_kidnap_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<KonkeyDongLogic>(this).holding_kidnaped_pygmy = false;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, KIDNAP_PYGMY_CHANNEL);

    let listener = world.obj::<KonkeyDongLogic>(this).konkey_dong_logic_listener;
    if let Some(listener) = listener {
        kong_statue_logic::kidnap_pygmy_complete(world, listener);
    }

    let kidnap_pygmy_logic = world.obj::<KonkeyDongLogic>(this).kidnap_pygmy_logic;
    if let Some(pygmy) = kidnap_pygmy_logic {
        crate::pygmy::kidnapped_by_konkey_dong_complete(world, pygmy);
    }
    world.obj_mut::<KonkeyDongLogic>(this).kidnap_pygmy_logic = None;
    ItemResult::Done
}

/// `-[KonkeyDongLogic cancelKidnapPygmyComplete:::]` @ 0x10009013c
pub fn seq_cancel_kidnap_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[KonkeyDongLogic startKonkeyDong]` @ 0x100090170
pub fn start_konkey_dong(world: &mut World, this: Entity) {
    let mut konkey_dong = world.obj_mut::<KonkeyDongLogic>(this);
    konkey_dong.difficulty = 1.0;
    konkey_dong.throw_boulder_time = 0.0;
}

/// `-[KonkeyDongLogic startNextLevel:]` @ 0x100090190
///
/// Each completed loop of the levels shortens the time between boulders by 15%, down to 30%.
pub fn start_next_level(world: &mut World, this: Entity, next_level: i32) {
    let mut difficulty = next_level as f32 * -0.15 + 1.0;
    if difficulty < 0.3 {
        difficulty = 0.3;
    }

    {
        let mut konkey_dong = world.obj_mut::<KonkeyDongLogic>(this);
        konkey_dong.end_level = false;
        konkey_dong.throw_boulder_time = 0.0;
        konkey_dong.difficulty = difficulty;
    }
    pg_engine::logic::set_behavior(world, this, Some(INIT_BEHAVIOR));
}

/// `-[KonkeyDongLogic endLevel]` @ 0x1000901f4
pub fn end_level(world: &mut World, this: Entity) {
    world.obj_mut::<KonkeyDongLogic>(this).end_level = true;
    pg_engine::logic::set_behavior(world, this, Some(END_LEVEL_BEHAVIOR));
}

/// `-[KonkeyDongLogic onGameFrame:]` @ 0x10009021c
///
/// Throws a boulder whenever the countdown runs out, then waits `difficulty · 14` seconds.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }
    let konkey_dong = world.obj::<KonkeyDongLogic>(this);
    if konkey_dong.end_level {
        return;
    }

    let throw_boulder_time = konkey_dong.throw_boulder_time - dt;
    world.obj_mut::<KonkeyDongLogic>(this).throw_boulder_time = throw_boulder_time;
    if throw_boulder_time > 0.0 {
        return;
    }

    pg_engine::logic::set_behavior(world, this, Some(THROW_BOULDER_BEHAVIOR));
    let mut konkey_dong = world.obj_mut::<KonkeyDongLogic>(this);
    konkey_dong.throw_boulder_time = konkey_dong.difficulty * 14.0;
}

/// `-[KonkeyDongLogic animThrowBoulder]` @ 0x1000902ac
pub fn anim_throw_boulder(world: &mut World, this: Entity) {
    let display_object = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display_object).pos;

    let listener = world.obj::<KonkeyDongLogic>(this).konkey_dong_logic_listener;
    if let Some(listener) = listener {
        let x_pos = pos.x + 94.0;
        let y_pos = pos.y + 28.0;
        kong_statue_logic::throw_boulder_at_x_pos(world, listener, x_pos, y_pos);
    }
}

/// `-[KonkeyDongLogic endLevelComplete:::]` @ 0x10009031c
pub fn seq_end_level_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<KonkeyDongLogic>(this).konkey_dong_logic_listener;
    if let Some(listener) = listener {
        kong_statue_logic::end_level_complete(world, listener);
    }
    ItemResult::Done
}
