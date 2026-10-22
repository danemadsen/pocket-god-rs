//! `LaserGunLogic` (superclass `SpriteLogic`).
//!
//! The laser gun on the laser shark's head with its beam and blast models. Its listener is the
//! `LaserSharkLogic`.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::underwater_baitmaster_laser::laser_shark_logic;
use pg_engine::display;
use pg_engine::logic;

/// `LaserGunLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct LaserGunLogic {
    /// `mLaserGunLogicListener`: `@"<LaserGunLogicListener>"`
    pub laser_gun_logic_listener: Option<Entity>,

    /// `mbRetractingBeam`: `B`
    pub retracting_beam: bool,

    /// `mLaserBeamDisplayObject`: `@"BCDisplayObject"`
    pub laser_beam_display_object: Option<Entity>,

    /// `mLaserBeamLogic`: `@"BCLogic"`
    pub laser_beam_logic: Option<Entity>,

    /// `mLaserBlastDisplayObject`: `@"BCDisplayObject"`
    pub laser_blast_display_object: Option<Entity>,

    /// `mLaserBlastLogic`: `@"BCLogic"`
    pub laser_blast_logic: Option<Entity>,
}

pub const LASER_GUN_LOGIC: Class = Class {
    name: "LaserGunLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The animation channel of the beam model.
const LASER_BEAM_CHANNEL: usize = 1;

/// The animation channel of the blast model.
const LASER_BLAST_CHANNEL: usize = 2;

/// Sequence-item handlers implemented by `LaserGunLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "onShootLaserComplete" => seq_on_shoot_laser_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `LaserGunLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animShowBeam" => anim_show_beam(world, this),
        "animHalfBeam" => anim_half_beam(world, this),
        "animHideBeam" => anim_hide_beam(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[LaserGunLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&LASER_GUN_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[LaserGunLogic initWithDisplayObject:]` @ 0x1001735e0
///
/// Builds the beam and blast models, each with its own `MultiModelLogic`, on animation
/// channels 1 and 2. The gun's own game frame does not run.
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(LaserGunLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let animation = logic::animation(world, this);

    let beam_display_object = display::multi_model::spawn(world, Some("LaserBeam"));
    world.obj_mut::<LaserGunLogic>(this).laser_beam_display_object = Some(beam_display_object);
    let beam_logic = pg_engine::logic::multi_model::spawn(world, beam_display_object);
    world.obj_mut::<LaserGunLogic>(this).laser_beam_logic = Some(beam_logic);
    logic::set_behavior(world, beam_logic, Some("LaserBeam"));
    pg_engine::animation::set_tweenable(world, animation, Some(beam_display_object), LASER_BEAM_CHANNEL);

    let blast_display_object = display::multi_model::spawn(world, Some("LaserBlast"));
    world.obj_mut::<LaserGunLogic>(this).laser_blast_display_object = Some(blast_display_object);
    let blast_logic = pg_engine::logic::multi_model::spawn(world, blast_display_object);
    world.obj_mut::<LaserGunLogic>(this).laser_blast_logic = Some(blast_logic);
    logic::set_behavior(world, blast_logic, Some("LaserBlast"));
    pg_engine::animation::set_tweenable(world, animation, Some(blast_display_object), LASER_BLAST_CHANNEL);

    logic::stop_game_frame(world, this);

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
    logic::set_behavior(world, this, Some("LaserGun"));
}

/// `-[LaserGunLogic setLaserGunLogicListener:]` @ 0x1001737c0
pub fn set_laser_gun_logic_listener(world: &mut World, this: Entity, laser_gun_logic_listener: Entity) {
    world.obj_mut::<LaserGunLogic>(this).laser_gun_logic_listener = Some(laser_gun_logic_listener);
}

/// `-[LaserGunLogic enable]` @ 0x100173900
pub fn enable(_world: &mut World, _this: Entity) {}

/// `-[LaserGunLogic disable]` @ 0x100173904
pub fn disable(_world: &mut World, _this: Entity) {}

/// `-[LaserGunLogic shootLaser]` @ 0x100173908
pub fn shoot_laser(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("LaserGunShoot"));
}

/// `-[LaserGunLogic updateLaser:]` @ 0x100173920
///
/// Stretches the beam over `update_laser` (the distance to the shot) and puts the blast at its
/// end; a retracting beam reaches 60 % as far.
pub fn update_laser(world: &mut World, this: Entity, update_laser: f32) {
    let (retracting_beam, beam_display_object, blast_display_object) = {
        let gun = world.obj::<LaserGunLogic>(this);
        (gun.retracting_beam, gun.laser_beam_display_object, gun.laser_blast_display_object)
    };

    let mut beam_x_scale = (update_laser - 19.0) / 100.0;
    if retracting_beam {
        beam_x_scale = beam_x_scale * 0.6;
    }
    if let Some(beam_display_object) = beam_display_object {
        display::object_mut(world, beam_display_object).scale.x = beam_x_scale;
    }
    if let Some(blast_display_object) = blast_display_object {
        display::object_mut(world, blast_display_object).pos.x = update_laser;
    }
}

/// `-[LaserGunLogic animShowBeam]` @ 0x1001739b4
pub fn anim_show_beam(world: &mut World, this: Entity) {
    let blast_logic = {
        let mut gun = world.obj_mut::<LaserGunLogic>(this);
        gun.retracting_beam = false;
        gun.laser_blast_logic
    };
    if let Some(blast_logic) = blast_logic {
        logic::set_behavior(world, blast_logic, Some("LaserBlast"));
    }
}

/// `-[LaserGunLogic animHalfBeam]` @ 0x1001739e4
pub fn anim_half_beam(world: &mut World, this: Entity) {
    world.obj_mut::<LaserGunLogic>(this).retracting_beam = true;
}

/// `-[LaserGunLogic animHideBeam]` @ 0x1001739f8
pub fn anim_hide_beam(_world: &mut World, _this: Entity) {}

/// `-[LaserGunLogic onShootLaserComplete:::]` @ 0x1001739fc
pub fn seq_on_shoot_laser_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<LaserGunLogic>(this).laser_gun_logic_listener;
    if let Some(listener) = listener {
        laser_shark_logic::on_shoot_laser_complete(world, listener, this);
    }
    ItemResult::Done
}
