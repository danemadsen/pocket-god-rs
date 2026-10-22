//! `ChumBucketLogic` (superclass `SpriteLogic`).
//!
//! The chum bucket of the underwater ruins: tapping it spills a chum, which brings the laser
//! shark, and shows the laser button. It owns the laser shark, the laser target and the
//! button (built by its own `setLogic:`). Its listener is the view controller.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::underwater_baitmaster_laser::{chum_logic, laser_shark_logic, laser_target_logic};
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;
use pg_engine::touch::BcTouch;

/// `ChumBucketLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ChumBucketLogic {
    /// `mChumBucketLogicListener`: `@"<ChumBucketLogicListener>"`
    pub chum_bucket_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbTransitionIn`: `B`
    pub transition_in: bool,

    /// `mLaserSharkLogic`: `@"LaserSharkLogic"`
    pub laser_shark_logic: Option<Entity>,

    /// `mLaserTargetLogic`: `@"LaserTargetLogic"`
    pub laser_target_logic: Option<Entity>,

    /// `mChumLogic`: `@"ChumLogic"`
    pub chum_logic: Option<Entity>,

    /// `mLogicArray`: `@"NSMutableArray"`
    pub logic_array: Vec<Entity>,

    /// `mLaserButtonLogic`: `@"BCButtonLogic"`
    pub laser_button_logic: Option<Entity>,
}

pub const CHUM_BUCKET_LOGIC: Class = Class {
    name: "ChumBucketLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Sequence-item handlers implemented by `ChumBucketLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "shootLaser" => seq_shoot_laser(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ChumBucketLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&CHUM_BUCKET_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[ChumBucketLogic initWithDisplayObject:]` @ 0x10011af54
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ChumBucketLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<ChumBucketLogic>(this).logic_array = Vec::with_capacity(5);

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
}

/// `-[ChumBucketLogic setChumBucketLogicListener:]` @ 0x10011affc
pub fn set_chum_bucket_logic_listener(world: &mut World, this: Entity, chum_bucket_logic_listener: Entity) {
    world.obj_mut::<ChumBucketLogic>(this).chum_bucket_logic_listener = Some(chum_bucket_logic_listener);
}

/// `-[ChumBucketLogic pygmyLogicArray:]` @ 0x10011b058
///
/// Forwards to the listener with `self` as the argument (the caller's is ignored).
pub fn pygmy_logic_array(world: &mut World, this: Entity, _pygmy_logic_array: Entity) -> Vec<Entity> {
    let listener = world.obj::<ChumBucketLogic>(this).chum_bucket_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::pygmy_logic_array(world, listener, this)
}

/// `-[ChumBucketLogic laserSharkLogic]` @ 0x10011b07c
pub fn laser_shark_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ChumBucketLogic>(this).laser_shark_logic
}

/// `-[ChumBucketLogic activate]` @ 0x10011b08c
pub fn activate(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "TsunamiDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("ChumBucketTransitionIn"));
}

/// `-[ChumBucketLogic deactivate]` @ 0x10011b128
///
/// Ends any zap and takes the button, shark, target and chum away.
pub fn deactivate(world: &mut World, this: Entity) {
    let (laser_button_logic, laser_shark_logic, laser_target_logic, listener) = {
        let bucket = world.obj::<ChumBucketLogic>(this);
        (bucket.laser_button_logic, bucket.laser_shark_logic, bucket.laser_target_logic, bucket.chum_bucket_logic_listener)
    };

    if let Some(laser_button_logic) = laser_button_logic {
        let button_display_object = logic::display_object(world, laser_button_logic);
        display::remove_from_display_group(world, button_display_object);
    }
    if let Some(listener) = listener {
        crate::controller::on_laser_shark_zap_complete(world, listener, this);
    }
    if let Some(laser_shark_logic) = laser_shark_logic {
        laser_shark_logic::exit_island(world, laser_shark_logic);
    }
    if let Some(laser_target_logic) = laser_target_logic {
        laser_target_logic::exit_island(world, laser_target_logic);
    }

    let chum = world.obj::<ChumBucketLogic>(this).chum_logic;
    if let Some(chum) = chum {
        chum_logic::exit_island(world, chum);
        release_chum(world, chum);
    }
    world.obj_mut::<ChumBucketLogic>(this).chum_logic = None;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("ChumBucketTransitionOut"));
}

/// `[mChumLogic shutdown]; [mChumLogic release]`: the bucket owns the chum alone, and the chum
/// holds the last reference to its model once the model is off the display.
fn release_chum(world: &mut World, chum: Entity) {
    let chum_display_object = logic::display_object(world, chum);
    logic::shutdown(world, chum);
    logic::dealloc(world, chum);

    let in_display_group = display::object(world, chum_display_object).display_group.is_some();
    if !in_display_group {
        world.despawn(chum_display_object);
    }
}

/// `-[ChumBucketLogic enterIsland:]` @ 0x10011b248
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let (active, transition_in) = {
        let bucket = world.obj::<ChumBucketLogic>(this);
        (bucket.active, bucket.transition_in)
    };
    if active {
        return;
    }
    world.obj_mut::<ChumBucketLogic>(this).active = true;
    if transition_in {
        activate(world, this);
    }
}

/// `-[ChumBucketLogic exitIsland]` @ 0x10011b284
pub fn exit_island(world: &mut World, this: Entity) {
    let (active, transition_in) = {
        let bucket = world.obj::<ChumBucketLogic>(this);
        (bucket.active, bucket.transition_in)
    };
    if !active {
        return;
    }
    world.obj_mut::<ChumBucketLogic>(this).active = false;
    if transition_in {
        deactivate(world, this);
    }
}

/// `-[ChumBucketLogic transitionIn]` @ 0x10011b2bc
pub fn transition_in(world: &mut World, this: Entity) {
    let (active, transition_in) = {
        let bucket = world.obj::<ChumBucketLogic>(this);
        (bucket.active, bucket.transition_in)
    };
    if transition_in {
        return;
    }
    world.obj_mut::<ChumBucketLogic>(this).transition_in = true;
    if active {
        activate(world, this);
    }
}

/// `-[ChumBucketLogic transitionOut]` @ 0x10011b2f8
pub fn transition_out(world: &mut World, this: Entity) {
    let (active, transition_in) = {
        let bucket = world.obj::<ChumBucketLogic>(this);
        (bucket.active, bucket.transition_in)
    };
    if !transition_in {
        return;
    }
    world.obj_mut::<ChumBucketLogic>(this).transition_in = false;
    if active {
        deactivate(world, this);
    }
}

/// `-[ChumBucketLogic laserTargetLogic:]` @ 0x10011b330
pub fn laser_target_logic(world: &mut World, this: Entity, _laser_target_logic: Entity) -> Option<Entity> {
    world.obj::<ChumBucketLogic>(this).laser_target_logic
}

/// `-[ChumBucketLogic chumLogic:]` @ 0x10011b340
pub fn chum_logic(world: &mut World, this: Entity, _chum_logic: Entity) -> Option<Entity> {
    world.obj::<ChumBucketLogic>(this).chum_logic
}

/// `-[ChumBucketLogic onLaserSharkZapPygmy:]` @ 0x10011b350
pub fn on_laser_shark_zap_pygmy(world: &mut World, this: Entity, _laser_shark_zap_pygmy: Entity) {
    let listener = world.obj::<ChumBucketLogic>(this).chum_bucket_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_laser_shark_zap_pygmy(world, listener, this);
    }
}

/// `-[ChumBucketLogic onLaserSharkExitScreen:]` @ 0x10011b374
pub fn on_laser_shark_exit_screen(world: &mut World, this: Entity, _laser_shark_exit_screen: Entity) {
    let (laser_target_logic, laser_button_logic, listener) = {
        let bucket = world.obj::<ChumBucketLogic>(this);
        (bucket.laser_target_logic, bucket.laser_button_logic, bucket.chum_bucket_logic_listener)
    };

    if let Some(laser_target_logic) = laser_target_logic {
        laser_target_logic::exit_island(world, laser_target_logic);
    }
    if let Some(laser_button_logic) = laser_button_logic {
        let button_display_object = logic::display_object(world, laser_button_logic);
        display::remove_from_display_group(world, button_display_object);
    }
    if let Some(listener) = listener {
        crate::controller::on_laser_shark_zap_complete(world, listener, this);
    }
}

/// `-[ChumBucketLogic shootLaser:::]` @ 0x10011b3f4
pub fn seq_shoot_laser(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let laser_shark_logic = world.obj::<ChumBucketLogic>(this).laser_shark_logic;
    if let Some(laser_shark_logic) = laser_shark_logic {
        laser_shark_logic::shoot_laser(world, laser_shark_logic);
    }
    ItemResult::Done
}

/// `-[ChumBucketLogic die:::]` @ 0x10011b424
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[ChumBucketLogic onGameFrame:]` @ 0x10011b458
///
/// Empty: the bucket does not move.
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[ChumBucketLogic onChumDie:]` @ 0x10011b528
pub fn on_chum_die(world: &mut World, this: Entity, _chum_die: Entity) {
    let chum = world.obj_mut::<ChumBucketLogic>(this).chum_logic.take();
    if let Some(chum) = chum {
        release_chum(world, chum);
    }
}

/// `-[ChumBucketLogic touchBeganInside:allTouches:]` @ 0x10011b578
///
/// Brings in the laser target and the shark; unless a chum is already out, shows the laser
/// button and spills a new chum under the finger, which takes the touch.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let (laser_target_logic, laser_shark_logic) = {
        let bucket = world.obj::<ChumBucketLogic>(this);
        (bucket.laser_target_logic, bucket.laser_shark_logic)
    };
    if let Some(laser_target_logic) = laser_target_logic {
        laser_target_logic::enter_island(world, laser_target_logic);
    }
    if let Some(laser_shark_logic) = laser_shark_logic {
        laser_shark_logic::enter_island(world, laser_shark_logic);
    }

    let (chum, laser_button_logic) = {
        let bucket = world.obj::<ChumBucketLogic>(this);
        (bucket.chum_logic, bucket.laser_button_logic)
    };
    if chum.is_some() {
        return;
    }

    let button_display_object = match laser_button_logic {
        Some(laser_button_logic) => Some(logic::display_object(world, laser_button_logic)),
        None => None,
    };
    let marker = pg_engine::library::display_marker(world, "MenuDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, button_display_object);
    }

    let chum_model = display::multi_model::spawn(world, Some("Chum"));
    let chum = chum_logic::spawn(world, chum_model);
    world.obj_mut::<ChumBucketLogic>(this).chum_logic = Some(chum);
    chum_logic::set_chum_logic_listener(world, chum, this);

    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let chum_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_pos),
        None => Vec4::ZERO,
    };
    {
        let mut object = display::object_mut(world, chum_model);
        object.pos.x = chum_pos.x;
        object.pos.y = chum_pos.y;
        object.pos.z = SPRITE_Z;
    }

    chum_logic::enter_island(world, chum);
    let chum_class = isa(world, chum);
    (chum_class.touch_began_inside)(world, chum, touch);
}

/// `-[ChumBucketLogic touchBeganOutside:allTouches:]` @ 0x10011b7a8
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ChumBucketLogic touchMovedEntered:allTouches:]` @ 0x10011b7ac
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ChumBucketLogic touchMovedExited:allTouches:]` @ 0x10011b7b0
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ChumBucketLogic touchMovedInside:allTouches:]` @ 0x10011b7b4
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[ChumBucketLogic touchMovedOutside:allTouches:]` @ 0x10011b7c4
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ChumBucketLogic touchEndedInside:allTouches:]` @ 0x10011b7c8
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ChumBucketLogic touchEndedOutside:allTouches:]` @ 0x10011b7cc
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}

/// `-[ChumBucketLogic setLogic:]` @ 0x10011b7dc
///
/// A boot `setLogic` item with `manager=ChumBucket` (`[0] displayGroupId`, `[1] logicId`,
/// `[2] displayObjectId`): builds the laser button (kept off the display until needed), the
/// laser shark or the laser target, listened to by the bucket.
pub fn set_logic(world: &mut World, this: Entity, item: &SequenceCall) {
    let logic_id = item.param("logicId");
    let display_object_id = item.param("displayObjectId");
    let display_group = match item.param("displayGroupId") {
        Some(display_group_id) => pg_engine::library::display_group(world, display_group_id),
        None => None,
    };
    let display_object = match (display_group, display_object_id) {
        (Some(display_group), Some(display_object_id)) => display::group::get_display_object(world, display_group, display_object_id),
        _ => None,
    };

    // PORT: the original also creates the logic on a nil display object; the engine's logics
    // need one. The bundled `ChumBucket` items all name existing objects.
    let Some(display_object) = display_object else {
        return;
    };

    let mut created_logic = None;
    if logic_id == Some("LaserButtonLogic") {
        let button = pg_engine::logic::button::spawn(world, display_object);
        world.obj_mut::<ChumBucketLogic>(this).laser_button_logic = Some(button);
        pg_engine::logic::button::set_button_logic_listener(world, button, Some(this));

        let button_display_object = logic::display_object(world, button);
        display::remove_from_display_group(world, button_display_object);
        created_logic = Some(button);
    }
    if logic_id == Some("LaserSharkLogic") {
        let shark = laser_shark_logic::spawn(world, display_object);
        world.obj_mut::<ChumBucketLogic>(this).laser_shark_logic = Some(shark);
        laser_shark_logic::set_laser_shark_logic_listener(world, shark, this);
        created_logic = Some(shark);
    }
    if logic_id == Some("LaserTargetLogic") {
        let target = laser_target_logic::spawn(world, display_object);
        world.obj_mut::<ChumBucketLogic>(this).laser_target_logic = Some(target);
        laser_target_logic::set_laser_target_logic_listener(world, target, this);
        created_logic = Some(target);
    }

    if let Some(created_logic) = created_logic {
        world.obj_mut::<ChumBucketLogic>(this).logic_array.push(created_logic);
    }
}

/// `-[ChumBucketLogic onButtonPress:touch:]` @ 0x10011ba14
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let laser_button_logic = world.obj::<ChumBucketLogic>(this).laser_button_logic;
    if laser_button_logic != Some(button) {
        return;
    }
    logic::set_behavior(world, this, Some("ShootLaser"));
}

/// `-[ChumBucketLogic onButtonRelease:touch:]` @ 0x10011ba44
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[ChumBucketLogic onButtonReleaseOutside:touch:]` @ 0x10011ba48
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}
