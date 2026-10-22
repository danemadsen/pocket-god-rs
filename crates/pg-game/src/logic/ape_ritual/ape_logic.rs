//! `ApeLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ape_ritual::ape_altar_logic;
use crate::logic::ape_ritual::ape_bongo_logic;
use crate::logic::ape_ritual::ape_torch_logic;
use crate::logic::ape_ritual::ape_wench_logic;
use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `ApeLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApeLogic {
    /// `mApeLogicListener`: `@"<ApeLogicListener>"`
    pub ape_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbVisible`: `B`
    pub visible: bool,

    /// `mbEnteringIsland`: `B`
    pub entering_island: bool,

    /// `mbExitIslandAfterEnter`: `B`
    pub exit_island_after_enter: bool,

    /// `mbDancing`: `B`
    pub dancing: bool,

    /// `mbPygmyHasPlague`: `B`
    pub pygmy_has_plague: bool,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const APE_LOGIC: Class = Class {
    name: "ApeLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `apocalypseDiseaseType` of a plague-infected pygmy.
const PLAGUE_DISEASE_TYPE: i32 = 1;

/// Sequence-item handlers implemented by `ApeLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "enterIslandComplete" => seq_enter_island_complete(world, this, call),
        "checkForPlaguePygmy" => seq_check_for_plague_pygmy(world, this, call),
        "checkEatPygmy" => seq_check_eat_pygmy(world, this, call),
        "apeExitAfterIdle" => seq_ape_exit_after_idle(world, this, call),
        "checkForLickChops" => seq_check_for_lick_chops(world, this, call),
        "lickChopsWaitForPygmy" => seq_lick_chops_wait_for_pygmy(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `ApeLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animGrabPygmy" => anim_grab_pygmy(world, this),
        "animHitWench" => anim_hit_wench(world, this),
        "animHitBongo" => anim_hit_bongo(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[ApeLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mApeLogicListener apeAltarLogic:self]` (nil for a nil listener).
fn listener_ape_altar_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<ApeLogic>(this).ape_logic_listener;
    let listener = listener?;
    crate::controller::ape_altar_logic(world, listener, this)
}

/// `[mApeLogicListener apeBongoLogic:self]` (nil for a nil listener).
fn listener_ape_bongo_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<ApeLogic>(this).ape_logic_listener;
    let listener = listener?;
    crate::controller::ape_bongo_logic(world, listener, this)
}

/// `[mApeLogicListener apeWenchLogic:self]` (nil for a nil listener).
fn listener_ape_wench_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<ApeLogic>(this).ape_logic_listener;
    let listener = listener?;
    crate::controller::ape_wench_logic(world, listener, this)
}

/// `[mApeLogicListener apeTorchLogicArray:self]` (none for a nil listener).
fn listener_ape_torch_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = world.obj::<ApeLogic>(this).ape_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::ape_torch_logic_array(world, listener, this)
}

/// `[[mApeLogicListener apeAltarLogic:self] isPygmyAvailableForApe]` (NO for a nil altar).
fn is_altar_pygmy_available_for_ape(world: &mut World, this: Entity) -> bool {
    let ape_altar = listener_ape_altar_logic(world, this);
    ape_altar.is_some_and(|ape_altar| ape_altar_logic::is_pygmy_available_for_ape(world, ape_altar))
}

/// `[[mApeLogicListener apeAltarLogic:self] isAvailable]` (NO for a nil altar).
fn is_altar_available(world: &mut World, this: Entity) -> bool {
    let ape_altar = listener_ape_altar_logic(world, this);
    ape_altar.is_some_and(|ape_altar| ape_altar_logic::is_available(world, ape_altar))
}

/// `-[ApeLogic initWithDisplayObject:]` @ 0x10008886c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApeLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
}

/// `-[ApeLogic setApeLogicListener:]` @ 0x1000888f4
pub fn set_ape_logic_listener(world: &mut World, this: Entity, ape_logic_listener: Entity) {
    world.obj_mut::<ApeLogic>(this).ape_logic_listener = Some(ape_logic_listener);
}

/// `-[ApeLogic startTapping]` @ 0x100088950
///
/// Sent when a celebration starts. The ape only comes when every torch burns, the altar pygmy is
/// stretched far enough and the bongo is being drummed; once it is in, it dances along.
pub fn start_tapping(world: &mut World, this: Entity) {
    let visible = world.obj::<ApeLogic>(this).visible;
    if !visible {
        let ape_torch_logic_array = listener_ape_torch_logic_array(world, this);
        for ape_torch in ape_torch_logic_array {
            if !ape_torch_logic::is_lit(world, ape_torch) {
                return;
            }
        }

        if !is_altar_pygmy_available_for_ape(world, this) {
            return;
        }

        let ape_bongo = listener_ape_bongo_logic(world, this);
        let bongo_playing = ape_bongo.is_some_and(|ape_bongo| ape_bongo_logic::is_playing(world, ape_bongo));
        if !bongo_playing {
            return;
        }

        {
            let mut ape = world.obj_mut::<ApeLogic>(this);
            ape.visible = true;
            ape.entering_island = true;
        }
        logic::set_behavior(world, this, Some("ApeEnterIsland"));

        let visible = world.obj::<ApeLogic>(this).visible;
        if !visible {
            return;
        }
    }

    let entering_island = world.obj::<ApeLogic>(this).entering_island;
    if entering_island {
        return;
    }

    world.obj_mut::<ApeLogic>(this).dancing = true;
    logic::set_behavior(world, this, Some("ApeDanceStart"));
}

/// `-[ApeLogic stopTapping]` @ 0x100088b64
pub fn stop_tapping(world: &mut World, this: Entity) {
    let dancing = world.obj::<ApeLogic>(this).dancing;
    if !dancing {
        return;
    }
    world.obj_mut::<ApeLogic>(this).dancing = false;

    // The original asks the altar for its pygmy twice before falling back to `isAvailable`.
    let behavior = if is_altar_pygmy_available_for_ape(world, this) {
        "ApeEatPygmy"
    } else if is_altar_pygmy_available_for_ape(world, this) || is_altar_available(world, this) {
        "ApeIdle"
    } else {
        "ApeLickChops"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[ApeLogic onDouseTorch]` @ 0x100088c70
pub fn on_douse_torch(world: &mut World, this: Entity) {
    let visible = world.obj::<ApeLogic>(this).visible;
    if !visible {
        return;
    }
    world.obj_mut::<ApeLogic>(this).dancing = false;

    // While the ape is still coming in, it leaves when its entrance completes.
    let entering_island = world.obj::<ApeLogic>(this).entering_island;
    if entering_island {
        world.obj_mut::<ApeLogic>(this).exit_island_after_enter = true;
        return;
    }

    world.obj_mut::<ApeLogic>(this).visible = false;
    logic::set_behavior(world, this, Some("ApeExitIsland"));
}

/// `-[ApeLogic enterIsland:]` @ 0x100088ccc
///
/// The startup flag is unused.
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    {
        let mut ape = world.obj_mut::<ApeLogic>(this);
        ape.active = false;
        ape.visible = false;
        ape.entering_island = false;
        ape.dancing = false;
    }

    let listener = world.obj::<ApeLogic>(this).ape_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_ape_visible(world, listener, this);
    }
}

/// `-[ApeLogic exitIsland:]` @ 0x100088d20
///
/// The flag is unused.
pub fn exit_island(world: &mut World, this: Entity, _now: bool) {
    {
        let mut ape = world.obj_mut::<ApeLogic>(this);
        ape.active = false;
        ape.entering_island = false;
        ape.dancing = false;
    }

    let visible = world.obj::<ApeLogic>(this).visible;
    if !visible {
        return;
    }

    // Original quirk, kept: mbEnteringIsland was cleared just above, so the deferred exit is
    // never taken here.
    let entering_island = world.obj::<ApeLogic>(this).entering_island;
    if entering_island {
        world.obj_mut::<ApeLogic>(this).exit_island_after_enter = true;
        return;
    }

    world.obj_mut::<ApeLogic>(this).visible = false;
    logic::set_behavior(world, this, Some("ApeExitIsland"));
}

/// `-[ApeLogic onGameFrame:]` @ 0x100088df0
///
/// Empty: the ape does not run `SpriteLogic`'s physics.
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[ApeLogic animGrabPygmy]` @ 0x100088df4
pub fn anim_grab_pygmy(world: &mut World, this: Entity) {
    if is_altar_pygmy_available_for_ape(world, this) {
        let ape_altar = listener_ape_altar_logic(world, this);
        if let Some(ape_altar) = ape_altar {
            ape_altar_logic::ape_grab_pygmy(world, ape_altar);
        }
        return;
    }

    logic::set_behavior(world, this, Some("ApeCrushBongoAndWenchPygmies"));
}

/// `-[ApeLogic animHitWench]` @ 0x100088e94
pub fn anim_hit_wench(world: &mut World, this: Entity) {
    let ape_wench = listener_ape_wench_logic(world, this);
    if let Some(ape_wench) = ape_wench {
        ape_wench_logic::ape_hit_ape_wench(world, ape_wench);
    }
}

/// `-[ApeLogic animHitBongo]` @ 0x100088ed4
pub fn anim_hit_bongo(world: &mut World, this: Entity) {
    let ape_bongo = listener_ape_bongo_logic(world, this);
    if let Some(ape_bongo) = ape_bongo {
        ape_bongo_logic::ape_hit_ape_bongo(world, ape_bongo);
    }
}

/// `-[ApeLogic enterIslandComplete:::]` @ 0x100088f14
pub fn seq_enter_island_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApeLogic>(this).entering_island = false;

    let exit_island_after_enter = world.obj::<ApeLogic>(this).exit_island_after_enter;
    if exit_island_after_enter {
        {
            let mut ape = world.obj_mut::<ApeLogic>(this);
            ape.visible = false;
            ape.exit_island_after_enter = false;
        }
        logic::set_behavior(world, this, Some("ApeExitIsland"));
    }
    ItemResult::Done
}

/// `-[ApeLogic checkForPlaguePygmy:::]` @ 0x100088f6c
///
/// After eating a plague pygmy the ape throws up: the `ApeVomit` multimodel (logic slot 1) is
/// inserted before the drag marker, at the ape's position.
pub fn seq_check_for_plague_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pygmy_has_plague = world.obj::<ApeLogic>(this).pygmy_has_plague;
    if !pygmy_has_plague {
        return ItemResult::Done;
    }

    world.obj_mut::<ApeLogic>(this).visible = false;

    let ape_vomit = pg_engine::library::logic(world, crate::logic_table::APE_VOMIT);
    logic::set_behavior(world, ape_vomit, Some("ApeVomitInit"));

    let drag_display_marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    let ape_vomit_display_object = logic::display_object(world, ape_vomit);
    if let Some(drag_display_marker) = drag_display_marker {
        display::insert_before(world, drag_display_marker, Some(ape_vomit_display_object));
    }

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    display::object_mut(world, ape_vomit_display_object).pos = pos;

    logic::set_behavior(world, this, Some("ApeEatPlaguePygmy"));
    ItemResult::Replaced
}

/// `-[ApeLogic checkEatPygmy:::]` @ 0x1000890c0
pub fn seq_check_eat_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let behavior = if is_altar_pygmy_available_for_ape(world, this) {
        let ape_altar = listener_ape_altar_logic(world, this);
        let altar_pygmy = ape_altar.and_then(|ape_altar| ape_altar_logic::altar_pygmy_logic(world, ape_altar));
        let disease_type = altar_pygmy.map_or(0, |pygmy| sprite_logic::apocalypse_disease_type(world, pygmy));
        world.obj_mut::<ApeLogic>(this).pygmy_has_plague = disease_type == PLAGUE_DISEASE_TYPE;
        "ApeEatPygmy"
    } else {
        "ApeCrushBongoAndWenchPygmies"
    };
    logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[ApeLogic apeExitAfterIdle:::]` @ 0x100089198
pub fn seq_ape_exit_after_idle(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApeLogic>(this).visible = false;
    logic::set_behavior(world, this, Some("ApeExitIsland"));
    ItemResult::Replaced
}

/// `-[ApeLogic checkForLickChops:::]` @ 0x1000891d0
pub fn seq_check_for_lick_chops(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if !is_altar_pygmy_available_for_ape(world, this) && !is_altar_available(world, this) {
        logic::set_behavior(world, this, Some("ApeLickChops"));
        return ItemResult::Replaced;
    }
    ItemResult::Done
}

/// `-[ApeLogic lickChopsWaitForPygmy:::]` @ 0x100089278
///
/// Waits while the altar is busy without a pygmy ready for the ape.
pub fn seq_lick_chops_wait_for_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if is_altar_available(world, this) {
        logic::set_behavior(world, this, Some("ApeCrushBongoAndWenchPygmies"));
        return ItemResult::Replaced;
    }

    if is_altar_pygmy_available_for_ape(world, this) {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}
