//! `LaserSharkLogic` (superclass `SpriteLogic`).
//!
//! The laser shark: drawn in by the chum, it swims after it (or the pygmy holding it) and eats
//! it; the player fires its head laser at the tilt-steered target. Its listener is the
//! `ChumBucketLogic`.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::underwater_baitmaster_laser::{chum_bucket_logic, chum_logic, laser_gun_logic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::rad_to_deg;

/// `LaserSharkLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct LaserSharkLogic {
    /// `mLaserSharkLogicListener`: `@"<LaserSharkLogicListener>"`
    pub laser_shark_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbStartShootLaser`: `B`
    pub start_shoot_laser: bool,

    /// `mbShootLaser`: `B`
    pub shoot_laser: bool,

    /// `mShootPos`: `{?="x"f"y"f"z"f"w"f}`
    pub shoot_pos: Vec4,

    /// `mbShotPygmy`: `B`
    pub shot_pygmy: bool,

    /// `mbChumOnScreen`: `B`
    pub chum_on_screen: bool,

    /// `mLaserGunDisplayObject`: `@"BCDisplayObject"`
    pub laser_gun_display_object: Option<Entity>,

    /// `mLaserGunLogic`: `@"LaserGunLogic"`
    pub laser_gun_logic: Option<Entity>,

    /// `mEatingPygmyLogic`: `@"PygmyLogic"`
    pub eating_pygmy_logic: Option<Entity>,
}

pub const LASER_SHARK_LOGIC: Class = Class {
    name: "LaserSharkLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Swimming speed.
const SWIM_SPEED: f32 = 150.0;

/// The animation channel the laser gun rides on.
const LASER_GUN_CHANNEL: usize = 1;

/// Sequence-item handlers implemented by `LaserSharkLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "eatPygmyComplete" => seq_eat_pygmy_complete(world, this, call),
        "initPos" => seq_init_pos(world, this, call),
        "swim" => seq_swim(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `LaserSharkLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animHideChum" => anim_hide_chum(world, this),
        "animKillPygmy" => anim_kill_pygmy(world, this),
        "animStopShark" => anim_stop_shark(world, this),
        "animStartShark" => anim_start_shark(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[LaserSharkLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&LASER_SHARK_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[LaserSharkLogic initWithDisplayObject:]` @ 0x1000c4d54
///
/// Mounts the laser gun on animation channel 1.
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(LaserSharkLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let laser_gun_display_object = display::multi_model::spawn(world, Some("LaserGun"));
    world.obj_mut::<LaserSharkLogic>(this).laser_gun_display_object = Some(laser_gun_display_object);
    let laser_gun = laser_gun_logic::spawn(world, laser_gun_display_object);
    world.obj_mut::<LaserSharkLogic>(this).laser_gun_logic = Some(laser_gun);
    laser_gun_logic::set_laser_gun_logic_listener(world, laser_gun, this);

    let animation = logic::animation(world, this);
    pg_engine::animation::set_tweenable(world, animation, Some(laser_gun_display_object), LASER_GUN_CHANNEL);

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
}

/// `-[LaserSharkLogic setLaserSharkLogicListener:]` @ 0x1000c4e74
pub fn set_laser_shark_logic_listener(world: &mut World, this: Entity, laser_shark_logic_listener: Entity) {
    world.obj_mut::<LaserSharkLogic>(this).laser_shark_logic_listener = Some(laser_shark_logic_listener);
}

/// `-[LaserSharkLogic isEnabled]` @ 0x1000c4ed0
pub fn is_enabled(world: &mut World, this: Entity) -> bool {
    world.obj::<LaserSharkLogic>(this).active
}

/// `-[LaserSharkLogic enterIsland]` @ 0x1000c4ee0
pub fn enter_island(world: &mut World, this: Entity) {
    let active = world.obj::<LaserSharkLogic>(this).active;
    if active {
        return;
    }
    world.obj_mut::<LaserSharkLogic>(this).active = true;

    let laser_gun = world.obj::<LaserSharkLogic>(this).laser_gun_logic;
    if let Some(laser_gun) = laser_gun {
        laser_gun_logic::enable(world, laser_gun);
    }

    let display_object = logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "TsunamiDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
    logic::set_behavior(world, this, Some("LaserSharkEnter"));
}

/// `-[LaserSharkLogic exitIsland]` @ 0x1000c4fa4
pub fn exit_island(world: &mut World, this: Entity) {
    let active = world.obj::<LaserSharkLogic>(this).active;
    if !active {
        return;
    }
    world.obj_mut::<LaserSharkLogic>(this).active = false;

    let eating_pygmy = world.obj::<LaserSharkLogic>(this).eating_pygmy_logic;
    if let Some(eating_pygmy) = eating_pygmy {
        crate::pygmy::stop_eaten_by_laser_shark(world, eating_pygmy, this);
        world.obj_mut::<LaserSharkLogic>(this).eating_pygmy_logic = None;
    }

    let laser_gun = world.obj::<LaserSharkLogic>(this).laser_gun_logic;
    if let Some(laser_gun) = laser_gun {
        laser_gun_logic::disable(world, laser_gun);
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    logic::set_behavior(world, this, Some("LaserSharkExit"));
}

/// `-[LaserSharkLogic onGameFrame:]` @ 0x1000c505c
///
/// Integrates without any force and aims the gun at the target (at the shot position while
/// shooting). The first frame of a shot fixes the shot position and zaps the first pygmy within
/// 30 of it.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let pos_vel = {
        let mut state = logic::logic_mut(world, this);
        state.pos_force = pos_force;
        state.pos_vel.x = state.pos_vel.x + pos_force.x * dt;
        state.pos_vel.y = state.pos_vel.y + pos_force.y * dt;
        state.pos_vel.z = state.pos_vel.z + pos_force.z * dt;
        state.pos_vel
    };
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = object.pos.x + pos_vel.x * dt;
        object.pos.y = object.pos.y + pos_vel.y * dt;
        object.pos.z = object.pos.z + pos_vel.z * dt;
    }

    let (laser_gun_display_object, laser_gun, listener) = {
        let shark = world.obj::<LaserSharkLogic>(this);
        (shark.laser_gun_display_object, shark.laser_gun_logic, shark.laser_shark_logic_listener)
    };
    let laser_target = match listener {
        Some(listener) => chum_bucket_logic::laser_target_logic(world, listener, this),
        None => None,
    };

    // The gun and the target always exist once the bucket's boot items have run (the original
    // reads through nil pointers otherwise).
    let (Some(laser_gun_display_object), Some(laser_target)) = (laser_gun_display_object, laser_target) else {
        return;
    };
    let target_display_object = logic::display_object(world, laser_target);
    let target_pos = display::object(world, target_display_object).pos;

    let start_shoot_laser = world.obj::<LaserSharkLogic>(this).start_shoot_laser;
    if start_shoot_laser {
        {
            let mut shark = world.obj_mut::<LaserSharkLogic>(this);
            shark.start_shoot_laser = false;
            shark.shoot_pos = target_pos;
        }
        zap_pygmy(world, this, target_pos, listener);
    }

    let (shoot_laser, shoot_pos) = {
        let shark = world.obj::<LaserSharkLogic>(this);
        (shark.shoot_laser, shark.shoot_pos)
    };
    let aim_pos = if shoot_laser { shoot_pos } else { target_pos };

    let gun_pos = display::object(world, laser_gun_display_object).pos;
    let delta_x = aim_pos.x - gun_pos.x;
    let delta_y = aim_pos.y - gun_pos.y;
    let heading = delta_y.atan2(delta_x);
    display::object_mut(world, laser_gun_display_object).rot.z = rad_to_deg(heading);

    if shoot_laser {
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
        if let Some(laser_gun) = laser_gun {
            laser_gun_logic::update_laser(world, laser_gun, distance);
        }
    }
}

/// The hit test of a shot: the first pygmy that can be hit with its middle (20 above its
/// position) within 30 of the shot position is zapped.
fn zap_pygmy(world: &mut World, this: Entity, shoot_pos: Vec4, listener: Option<Entity>) {
    let pygmies = match listener {
        Some(listener) => chum_bucket_logic::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };

    for pygmy in pygmies {
        if !crate::pygmy::can_be_hit_by_shark_laser(world, pygmy) {
            continue;
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let offset_x = shoot_pos.x - (pygmy_pos.x + 0.0);
        let offset_y = shoot_pos.y - (pygmy_pos.y + 20.0);
        if offset_x * offset_x + offset_y * offset_y < 900.0 {
            world.obj_mut::<LaserSharkLogic>(this).shot_pygmy = true;
            crate::pygmy::hit_by_shark_laser(world, pygmy);
            if let Some(listener) = listener {
                chum_bucket_logic::on_laser_shark_zap_pygmy(world, listener, this);
            }
            return;
        }
    }
}

/// `-[LaserSharkLogic shootLaser]` @ 0x1000c5430
pub fn shoot_laser(world: &mut World, this: Entity) {
    let laser_gun = {
        let mut shark = world.obj_mut::<LaserSharkLogic>(this);
        shark.shoot_laser = true;
        shark.start_shoot_laser = true;
        shark.laser_gun_logic
    };
    if let Some(laser_gun) = laser_gun {
        laser_gun_logic::shoot_laser(world, laser_gun);
    }
}

/// `-[LaserSharkLogic onShootLaserComplete:]` @ 0x1000c5468
pub fn on_shoot_laser_complete(world: &mut World, this: Entity, _shoot_laser_complete: Entity) {
    world.obj_mut::<LaserSharkLogic>(this).shoot_laser = false;
}

/// `[mLaserSharkLogicListener chumLogic:self]`.
fn listener_chum_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<LaserSharkLogic>(this).laser_shark_logic_listener;
    match listener {
        Some(listener) => chum_bucket_logic::chum_logic(world, listener, this),
        None => None,
    }
}

/// `-[LaserSharkLogic animHideChum]` @ 0x1000c5478
pub fn anim_hide_chum(world: &mut World, this: Entity) {
    let chum = listener_chum_logic(world, this);
    if let Some(chum) = chum {
        chum_logic::eaten_by_shark(world, chum);
    }
}

/// `-[LaserSharkLogic animKillPygmy]` @ 0x1000c54b8
pub fn anim_kill_pygmy(world: &mut World, this: Entity) {
    let chum = listener_chum_logic(world, this);
    if let Some(chum) = chum {
        chum_logic::eaten_by_shark(world, chum);
    }

    let eating_pygmy = world.obj::<LaserSharkLogic>(this).eating_pygmy_logic;
    if let Some(eating_pygmy) = eating_pygmy {
        crate::pygmy::eaten_by_laser_shark(world, eating_pygmy, this);
    }
}

/// `-[LaserSharkLogic animStopShark]` @ 0x1000c5520
pub fn anim_stop_shark(world: &mut World, this: Entity) {
    let mut state = logic::logic_mut(world, this);
    state.pos_vel.x = 0.0;
    state.pos_vel.y = 0.0;
}

/// `-[LaserSharkLogic animStartShark]` @ 0x1000c5538
///
/// Swims off along its heading (mirrored by `xScale`).
pub fn anim_start_shark(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let (z_rot, x_scale) = {
        let object = display::object(world, display_object);
        (object.rot.z, object.scale.x)
    };

    let heading = x_scale * std::f32::consts::PI * z_rot / 180.0;
    let (sin, cos) = heading.sin_cos();

    let mut state = logic::logic_mut(world, this);
    state.pos_vel.x = x_scale * (cos * SWIM_SPEED);
    state.pos_vel.y = sin * SWIM_SPEED;
}

/// `-[LaserSharkLogic eatPygmyComplete:::]` @ 0x1000c55ec
pub fn seq_eat_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let eating_pygmy = world.obj::<LaserSharkLogic>(this).eating_pygmy_logic;
    if let Some(eating_pygmy) = eating_pygmy {
        crate::pygmy::swallowed_by_laser_shark(world, eating_pygmy, this);
    }
    world.obj_mut::<LaserSharkLogic>(this).eating_pygmy_logic = None;
    ItemResult::Done
}

/// `-[LaserSharkLogic initPos:::]` @ 0x1000c5630
///
/// Enters level from a random side at a random height between 70 and 250.
pub fn seq_init_pos(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    let side_random = world.resource_mut::<Random>().arc4random();
    {
        let mut object = display::object_mut(world, display_object);
        object.rot.z = 0.0;
        object.scale.x = 1.0;
        object.pos.x = -50.0;
        if side_random & 1 != 0 {
            object.scale.x = -1.0;
            object.pos.x = 530.0;
        }
    }

    let height_random = world.resource_mut::<Random>().arc4random();
    let y_pos = ((height_random % 1801 + 700) as i32) as f32 / 10.0;
    display::object_mut(world, display_object).pos.y = y_pos;

    ItemResult::Done
}

/// `-[LaserSharkLogic swim:::]` @ 0x1000c5710
///
/// Steers toward the chum (or the screen's middle height without one), its heading held within
/// ±30°, and bites the chum (or the pygmy holding it) in reach of its jaws. It turns back at the
/// screen edge while there is something to chase and reports leaving the screen otherwise.
pub fn seq_swim(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);
    let (pos, z_rot, x_scale) = {
        let object = display::object(world, display_object);
        (object.pos, object.rot.z, object.scale.x)
    };

    let mut delta_x = 160.0;
    let mut delta_y = 160.0 - pos.y;

    let chum = listener_chum_logic(world, this);
    if let Some(chum) = chum {
        world.obj_mut::<LaserSharkLogic>(this).chum_on_screen = true;

        let chum_display_object = logic::display_object(world, chum);
        let chum_pos = display::object(world, chum_display_object).pos;
        let chase_chum = chum_pos.y > 100.0 || chum_logic::is_pygmy_holding_chum(world, chum);
        if chase_chum {
            delta_x = (chum_pos.x - pos.x).abs();
            delta_y = chum_pos.y - pos.y;
        } else {
            delta_x = 160.0;
        }

        if chum_logic::can_be_fed_to_shark(world, chum) {
            let jaws_x = x_scale * 80.0 + pos.x;
            let jaws_y = pos.y + -1.0;
            let bite_x = jaws_x - chum_pos.x;
            let bite_y = jaws_y - chum_pos.y;
            let bite_distance = (bite_x * bite_x + bite_y * bite_y).sqrt();
            if bite_distance < 20.0 {
                let behavior = if chum_logic::is_pygmy_holding_chum(world, chum) {
                    let holding_pygmy = chum_logic::holding_pygmy_logic(world, chum);
                    world.obj_mut::<LaserSharkLogic>(this).eating_pygmy_logic = holding_pygmy;
                    if let Some(holding_pygmy) = holding_pygmy {
                        crate::pygmy::start_eaten_by_laser_shark(world, holding_pygmy, this);
                    }
                    "LaserSharkEatPygmy"
                } else {
                    "LaserSharkEatChum"
                };
                logic::set_behavior(world, this, Some(behavior));
                return ItemResult::Replaced;
            }
        }
    }

    let course = rad_to_deg(delta_y.atan2(delta_x));
    let course = if course > 30.0 { 30.0 } else { course };
    let course = if course < -30.0 { -30.0 } else { course };
    let turned_z_rot = z_rot + (x_scale * course - z_rot) * 0.5;
    display::object_mut(world, display_object).rot.z = turned_z_rot;

    let heading = x_scale * std::f32::consts::PI * turned_z_rot / 180.0;
    let (sin, cos) = heading.sin_cos();
    let vel_x = x_scale * (cos * SWIM_SPEED);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = vel_x;
        state.pos_vel.y = sin * SWIM_SPEED;
    }

    let (chum_on_screen, shot_pygmy, listener) = {
        let shark = world.obj::<LaserSharkLogic>(this);
        (shark.chum_on_screen, shark.shot_pygmy, shark.laser_shark_logic_listener)
    };
    let at_edge = (x_scale < 0.0 && max_pos.x < 80.0) || (x_scale > 0.0 && min_pos.x > 400.0);
    if (chum_on_screen || shot_pygmy) && at_edge {
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        display::object_mut(world, display_object).rot.z = 0.0;
        {
            let mut shark = world.obj_mut::<LaserSharkLogic>(this);
            shark.shot_pygmy = false;
            shark.chum_on_screen = false;
        }
        logic::set_behavior(world, this, Some("LaserSharkTurn"));
        return ItemResult::Replaced;
    }

    let off_screen = (vel_x < 0.0 && max_pos.x < -50.0) || (vel_x > 0.0 && min_pos.x > 530.0);
    if !off_screen {
        return ItemResult::Pending;
    }
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    if let Some(listener) = listener {
        chum_bucket_logic::on_laser_shark_exit_screen(world, listener, this);
    }
    ItemResult::Done
}

/// `-[LaserSharkLogic die:::]` @ 0x1000c5b00
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    exit_island(world, this);
    ItemResult::Done
}
