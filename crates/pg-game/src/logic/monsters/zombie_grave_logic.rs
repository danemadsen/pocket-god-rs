//! `ZombieGraveLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::touch::BcTouch;

/// `ZombieGraveLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ZombieGraveLogic {
    /// `mZombieGraveLogicListener`: `@"<ZombieGraveLogicListener>"`
    pub zombie_grave_logic_listener: Option<Entity>,

    /// `mTrappedPygmyLogic`: `@"PygmyLogic"`
    pub trapped_pygmy_logic: Option<Entity>,

    /// `mbPygmyBuried`: `B`
    pub pygmy_buried: bool,

    /// `mDigCount`: `i`
    pub dig_count: i32,
}

pub const ZOMBIE_GRAVE_LOGIC: Class = Class {
    name: "ZombieGraveLogic",
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
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The animation channel that carries the pygmy lying in the grave.
const PYGMY_CHANNEL: usize = 1;

/// The mound of dug-out earth beside the hole, relative to the grave's position.
const MOUND_X_OFFSET: f32 = -35.0;
const MOUND_Y_OFFSET: f32 = 20.0;

/// A tap closer than this to the mound shovels earth back in; any other tap digs.
const MOUND_RADIUS: f32 = 30.0;

/// Sequence-item handlers implemented by `ZombieGraveLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "startZombieIdle" => seq_start_zombie_idle(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ZombieGraveLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&ZOMBIE_GRAVE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[ZombieGraveLogic initWithDisplayObject:]` @ 0x1000c42d4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ZombieGraveLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::stop_game_frame(world, this);

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, own_display_object).is_selectable = false;

    pg_engine::logic::set_behavior(world, this, Some("ZombieGraveExitIsland"));
}

/// `-[ZombieGraveLogic setZombieGraveLogicListener:]` @ 0x1000c4378
pub fn set_zombie_grave_logic_listener(world: &mut World, this: Entity, zombie_grave_logic_listener: Entity) {
    world.obj_mut::<ZombieGraveLogic>(this).zombie_grave_logic_listener = Some(zombie_grave_logic_listener);
}

/// `-[ZombieGraveLogic isOpenAndUnoccupied]` @ 0x1000c43d4
pub fn is_open_and_unoccupied(world: &mut World, this: Entity) -> bool {
    let grave = world.obj::<ZombieGraveLogic>(this);
    grave.dig_count == 2 && grave.trapped_pygmy_logic.is_none()
}

/// `-[ZombieGraveLogic isPygmyAboutToBecomeZombie]` @ 0x1000c4408
pub fn is_pygmy_about_to_become_zombie(world: &mut World, this: Entity) -> bool {
    let grave = world.obj::<ZombieGraveLogic>(this);
    grave.dig_count == 0 && grave.trapped_pygmy_logic.is_some()
}

/// `-[ZombieGraveLogic loseBalanceByGrave:]` @ 0x1000c4438
///
/// Binds the stumbling pygmy; the behaviour comes with `tripIntoGrave:fallForward:`.
pub fn lose_balance_by_grave(world: &mut World, this: Entity, lose_balance_by_grave: Entity) {
    world.obj_mut::<ZombieGraveLogic>(this).trapped_pygmy_logic = Some(lose_balance_by_grave);

    let animation = pg_engine::logic::animation(world, this);
    let pygmy_display_object = pg_engine::logic::display_object(world, lose_balance_by_grave);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);
}

/// `-[ZombieGraveLogic tripIntoGrave:fallForward:]` @ 0x1000c4498
pub fn trip_into_grave(world: &mut World, this: Entity, _trip_into_grave: Entity, fall_forward: bool) {
    if fall_forward {
        pg_engine::logic::set_behavior(world, this, Some("ZombieGravePygmyFallForward"));
        return;
    }
    pg_engine::logic::set_behavior(world, this, Some("ZombieGravePygmyFallBackward"));
}

/// `-[ZombieGraveLogic fallIntoGrave:]` @ 0x1000c44c0
pub fn fall_into_grave(world: &mut World, this: Entity, fall_into_grave: Entity) {
    world.obj_mut::<ZombieGraveLogic>(this).trapped_pygmy_logic = Some(fall_into_grave);

    let animation = pg_engine::logic::animation(world, this);
    let pygmy_display_object = pg_engine::logic::display_object(world, fall_into_grave);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);

    pg_engine::logic::set_behavior(world, this, Some("ZombieGravePygmyFall"));
}

/// `-[ZombieGraveLogic ejectFromGrave:]` @ 0x1000c4540
///
/// The pygmy climbs out; the grave stays fully dug.
pub fn eject_from_grave(world: &mut World, this: Entity, _eject_from_grave: Entity) {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

    pg_engine::logic::set_behavior(world, this, Some("ZombieGraveEjectPygmy"));

    let mut grave = world.obj_mut::<ZombieGraveLogic>(this);
    grave.trapped_pygmy_logic = None;
    grave.pygmy_buried = false;
    grave.dig_count = 2;
}

/// `-[ZombieGraveLogic enterIsland:]` @ 0x1000c45c4
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    world.obj_mut::<ZombieGraveLogic>(this).dig_count = 0;
    pg_engine::logic::start_game_frame(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = true;

    pg_engine::logic::set_behavior(world, this, Some("ZombieGraveEnterIsland"));
}

/// `-[ZombieGraveLogic exitIsland:]` @ 0x1000c4638
///
/// Lets the trapped pygmy go; leaving for good (`now`) also blows up every zombie.
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    let trapped = world.obj::<ZombieGraveLogic>(this).trapped_pygmy_logic;
    if let Some(trapped) = trapped {
        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

        crate::pygmy::eject_from_zombie_grave(world, trapped);

        let mut grave = world.obj_mut::<ZombieGraveLogic>(this);
        grave.trapped_pygmy_logic = None;
        grave.pygmy_buried = false;
    }

    if exit_island {
        let listener = world.obj::<ZombieGraveLogic>(this).zombie_grave_logic_listener;
        let pygmies = match listener {
            Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
            None => Vec::new(),
        };
        for pygmy in pygmies {
            crate::pygmy::zombie_explode(world, pygmy);
        }

        pg_engine::logic::stop_game_frame(world, this);
    }

    world.obj_mut::<ZombieGraveLogic>(this).dig_count = 0;

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    pg_engine::logic::set_behavior(world, this, Some("ZombieGraveExitIsland"));
}

/// `-[ZombieGraveLogic onGameFrame:]` @ 0x1000c4888
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[ZombieGraveLogic startZombieIdle:::]` @ 0x1000c488c
///
/// The buried pygmy rises as a zombie and the grave lets go of it.
pub fn seq_start_zombie_idle(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ZombieGraveLogic>(this).pygmy_buried = false;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, PYGMY_CHANNEL);

    let trapped = world.obj::<ZombieGraveLogic>(this).trapped_pygmy_logic;
    if let Some(trapped) = trapped {
        crate::pygmy::start_zombie_idle(world, trapped);
    }
    world.obj_mut::<ZombieGraveLogic>(this).trapped_pygmy_logic = None;
    ItemResult::Done
}

/// Whether the touch lands within `MOUND_RADIUS` of the mound beside the hole, measured in
/// the grave's display group.
fn is_touch_on_mound(world: &World, this: Entity, touch: Entity) -> bool {
    let display_object = pg_engine::logic::display_object(world, this);
    let (pos, display_group) = {
        let object = pg_engine::display::object(world, display_object);
        (object.pos, object.display_group)
    };

    let touch_screen_pos = world.obj::<BcTouch>(touch).pos;
    let touch_pos = match display_group {
        Some(display_group) => pg_engine::display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    let delta_x = (pos.x + MOUND_X_OFFSET) - touch_pos.x;
    let delta_y = (pos.y + MOUND_Y_OFFSET) - touch_pos.y;
    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
    distance < MOUND_RADIUS
}

/// `-[ZombieGraveLogic touchBeganInside:allTouches:]` @ 0x1000c48fc
///
/// A tap on the mound shovels earth back in (one dig step back), any other tap digs (one step
/// on). A pygmy that fell in and is not yet buried is buried by a shovel, or pulled out by a
/// dig; a buried one is dug up again at the second step.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let (dig_count, trapped, pygmy_buried) = {
        let grave = world.obj::<ZombieGraveLogic>(this);
        (grave.dig_count, grave.trapped_pygmy_logic, grave.pygmy_buried)
    };

    let dig = if dig_count == 0 {
        if trapped.is_some() {
            return;
        }
        true
    } else {
        !is_touch_on_mound(world, this, touch)
    };

    if let Some(trapped) = trapped
        && !pygmy_buried
    {
        if !dig {
            {
                let mut grave = world.obj_mut::<ZombieGraveLogic>(this);
                grave.pygmy_buried = true;
                grave.dig_count -= 1;
            }
            pg_engine::logic::set_behavior(world, this, Some("ZombieGraveBuryPygmy"));
            return;
        }

        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

        crate::pygmy::eject_from_zombie_grave(world, trapped);
        pg_engine::logic::set_behavior(world, this, Some("ZombieGraveEjectPygmy"));
        world.obj_mut::<ZombieGraveLogic>(this).trapped_pygmy_logic = None;
        return;
    }

    if dig {
        let behavior = match dig_count {
            0 => "ZombieGraveDig1",
            1 if trapped.is_none() => "ZombieGraveDig2",
            1 => {
                world.obj_mut::<ZombieGraveLogic>(this).pygmy_buried = false;
                "ZombieGraveUnburyPygmy"
            }
            _ => return,
        };
        pg_engine::logic::set_behavior(world, this, Some(behavior));
        world.obj_mut::<ZombieGraveLogic>(this).dig_count += 1;
        return;
    }

    let behavior = match dig_count {
        2 => "ZombieGraveBury2",
        1 if trapped.is_none() => "ZombieGraveBury1",
        1 => "ZombieGravePygmyEmerge",
        _ => return,
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    world.obj_mut::<ZombieGraveLogic>(this).dig_count -= 1;
}

/// `-[ZombieGraveLogic touchBeganOutside:allTouches:]` @ 0x1000c4ba4
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ZombieGraveLogic touchMovedEntered:allTouches:]` @ 0x1000c4ba8
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ZombieGraveLogic touchMovedExited:allTouches:]` @ 0x1000c4bac
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ZombieGraveLogic touchMovedInside:allTouches:]` @ 0x1000c4bb0
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[ZombieGraveLogic touchMovedOutside:allTouches:]` @ 0x1000c4bc0
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ZombieGraveLogic touchEndedInside:allTouches:]` @ 0x1000c4bc4
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ZombieGraveLogic touchEndedOutside:allTouches:]` @ 0x1000c4bc8
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
