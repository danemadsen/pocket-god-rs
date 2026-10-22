//! `ApeBongoLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::island_core::sprite_logic::SpriteLogic;
use pg_engine::display;
use pg_engine::logic;

/// `ApeBongoLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApeBongoLogic {
    /// `mApeBongoLogicListener`: `@"<ApeBongoLogicListener>"`
    pub ape_bongo_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbPlaying`: `B`
    pub playing: bool,

    /// `mbCanDrum`: `B`
    pub can_drum: bool,

    /// `mbBeingSmashed`: `B`
    pub being_smashed: bool,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const APE_BONGO_LOGIC: Class = Class {
    name: "ApeBongoLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The tweenable channel of the bongo's animation that carries the drumming pygmy.
const PYGMY_CHANNEL: usize = 1;

/// Sequence-item handlers implemented by `ApeBongoLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "pygmyFallInsideComplete" => seq_pygmy_fall_inside_complete(world, this, call),
        "apeHitComplete" => seq_ape_hit_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ApeBongoLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APE_BONGO_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[ApeBongoLogic initWithDisplayObject:]` @ 0x100136fdc
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApeBongoLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_style_id = Some(String::from("ApeBongo"));
    logic::set_behavior(world, this, Some("ApeBongoExitIsland"));
}

/// `-[ApeBongoLogic setApeBongoLogicListener:]` @ 0x100137098
pub fn set_ape_bongo_logic_listener(world: &mut World, this: Entity, ape_bongo_logic_listener: Entity) {
    world.obj_mut::<ApeBongoLogic>(this).ape_bongo_logic_listener = Some(ape_bongo_logic_listener);
}

/// `-[ApeBongoLogic isAvailable]` @ 0x1001370f4
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let bongo = world.obj::<ApeBongoLogic>(this);
    bongo.pygmy_logic.is_none() && !bongo.being_smashed
}

/// `-[ApeBongoLogic apeHitApeBongo]` @ 0x100137124
pub fn ape_hit_ape_bongo(world: &mut World, this: Entity) {
    let active = world.obj::<ApeBongoLogic>(this).active;
    if !active {
        return;
    }
    world.obj_mut::<ApeBongoLogic>(this).being_smashed = true;

    let pygmy = world.obj::<ApeBongoLogic>(this).pygmy_logic;
    let behavior = match pygmy {
        Some(pygmy) => {
            crate::pygmy::ape_owns_pygmy(world, pygmy);
            "ApeBongoApeHitPygmy"
        }
        None => "ApeBongoApeHit",
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[ApeBongoLogic pygmyFallIntoApeBongo:]` @ 0x1001371bc
pub fn pygmy_fall_into_ape_bongo(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let current_pygmy = world.obj::<ApeBongoLogic>(this).pygmy_logic;
    if current_pygmy.is_some() {
        return;
    }

    {
        let mut bongo = world.obj_mut::<ApeBongoLogic>(this);
        bongo.can_drum = false;
        bongo.pygmy_logic = Some(pygmy_logic);
    }

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("ApeBongoPygmyFallInside"));
}

/// `-[ApeBongoLogic isPlaying]` @ 0x10013725c
pub fn is_playing(world: &mut World, this: Entity) -> bool {
    let bongo = world.obj::<ApeBongoLogic>(this);
    bongo.playing && bongo.can_drum
}

/// `-[ApeBongoLogic startPlaying]` @ 0x10013728c
///
/// A pygmy that is still falling in starts drumming in `pygmyFallInsideComplete:::`.
pub fn start_playing(world: &mut World, this: Entity) {
    let pygmy = world.obj::<ApeBongoLogic>(this).pygmy_logic;
    if pygmy.is_none() {
        return;
    }
    world.obj_mut::<ApeBongoLogic>(this).playing = true;

    let can_drum = world.obj::<ApeBongoLogic>(this).can_drum;
    if can_drum {
        logic::set_behavior(world, this, Some("ApeBongoPygmyStart"));
    }
}

/// `-[ApeBongoLogic stopPlaying]` @ 0x1001372d8
pub fn stop_playing(world: &mut World, this: Entity) {
    let playing = world.obj::<ApeBongoLogic>(this).playing;
    if !playing {
        return;
    }
    world.obj_mut::<ApeBongoLogic>(this).playing = false;
    logic::set_behavior(world, this, Some("ApeBongoPygmyStop"));
}

/// `-[ApeBongoLogic enterIsland:]` @ 0x100137308
///
/// The startup flag is unused.
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    {
        let mut bongo = world.obj_mut::<ApeBongoLogic>(this);
        bongo.active = true;
        bongo.playing = false;
        bongo.can_drum = false;
        bongo.being_smashed = false;
    }

    logic::set_behavior(world, this, Some("ApeBongoEnterIsland"));
    sprite_logic::show_apocalypse_cloud(world, this);
}

/// `-[ApeBongoLogic removePygmyFromApeBongo:]` @ 0x100137380
pub fn remove_pygmy_from_ape_bongo(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let current_pygmy = world.obj::<ApeBongoLogic>(this).pygmy_logic;
    if current_pygmy != Some(pygmy_logic) {
        return;
    }

    {
        let mut bongo = world.obj_mut::<ApeBongoLogic>(this);
        bongo.playing = false;
        bongo.can_drum = false;
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    world.obj_mut::<ApeBongoLogic>(this).pygmy_logic = None;
    logic::set_behavior(world, this, Some("ApeBongoEnterIsland"));
}

/// `-[ApeBongoLogic exitIsland:]` @ 0x100137414
///
/// The flag is unused.
pub fn exit_island(world: &mut World, this: Entity, _now: bool) {
    world.obj_mut::<ApeBongoLogic>(this).active = false;

    let pygmy = world.obj::<ApeBongoLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::toss_from_ape_bongo(world, pygmy);
        world.obj_mut::<ApeBongoLogic>(this).pygmy_logic = None;
    }

    {
        let mut bongo = world.obj_mut::<ApeBongoLogic>(this);
        bongo.playing = false;
        bongo.can_drum = false;
        bongo.being_smashed = false;
    }

    logic::set_behavior(world, this, Some("ApeBongoExitIsland"));
    sprite_logic::hide_apocalypse_cloud(world, this);
}

/// `-[ApeBongoLogic onGameFrame:]` @ 0x10013753c
///
/// Empty: the bongo does not run `SpriteLogic`'s physics.
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[ApeBongoLogic pygmyFallInsideComplete:::]` @ 0x100137540
pub fn seq_pygmy_fall_inside_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApeBongoLogic>(this).can_drum = true;

    let playing = world.obj::<ApeBongoLogic>(this).playing;
    if playing {
        logic::set_behavior(world, this, Some("ApeBongoPygmyStart"));
    }
    ItemResult::Done
}

/// `-[ApeBongoLogic apeHitComplete:::]` @ 0x10013758c
pub fn seq_ape_hit_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApeBongoLogic>(this).being_smashed = false;

    let pygmy = world.obj::<ApeBongoLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::ape_smash_on_ape_bongo(world, pygmy);

        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id(world, animation, PYGMY_CHANNEL);
        world.obj_mut::<ApeBongoLogic>(this).pygmy_logic = None;
    }
    ItemResult::Done
}
