//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::island_core::sprite_logic::SpriteLogic;

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyIslandHazards {
    /// `mSafeIslandMinXPos`: `f`
    pub safe_island_min_x_pos: f32,

    /// `mSafeIslandMaxXPos`: `f`
    pub safe_island_max_x_pos: f32,

    /// `mSafeRunIslandMinXPos`: `f`
    pub safe_run_island_min_x_pos: f32,

    /// `mSafeRunIslandMaxXPos`: `f`
    pub safe_run_island_max_x_pos: f32,

    /// `mbGingerSunBurn`: `B`
    pub ginger_sun_burn: bool,

    /// `mbWarlockSkipNextExplosion`: `B`
    pub warlock_skip_next_explosion: bool,

    /// `mbIceShatter`: `B`
    pub ice_shatter: bool,

    /// `mSunBurnFreezeCycleCount`: `i`
    pub sun_burn_freeze_cycle_count: i32,

    /// `mbGrabbedByTentacle`: `B`
    pub grabbed_by_tentacle: bool,

    /// `mTentacleStrengthRot`: `f`
    pub tentacle_strength_rot: f32,

    /// `mTentacleStrengthRotVel`: `f`
    pub tentacle_strength_rot_vel: f32,

    /// `mIceHoleLogic`: `@"IceHoleLogic"`
    pub ice_hole_logic: Option<Entity>,

    /// `mCarryingBirdLogic`: `@"BirdLogic"`
    pub carrying_bird_logic: Option<Entity>,

    /// `mbCrushedByBoulder`: `B`
    pub crushed_by_boulder: bool,
}

/// The members of the busy set FREE (`docs/re/pygmy.md` §1.3) that every guard of this part
/// tests. The rest of FREE (`mbTouchBeganInside`, `mTimeTemplePygmyType`, `mbDragging`, the
/// hurricane flags, the containers and `mbIsZombie`) is left out by one guard or another.
fn is_busy_base(core: &PygmyPygmyCoreFrame) -> bool {
    core.remove_from_story_mode
        || core.cut_in_half_by_charlie
        || core.dropped_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.catch_shark
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.resting_in_igloo
        || core.crapping
        || core.trapped_in_spider_web
        || core.trapped_in_zombie_grave
        || core.trapped_in_idol_pedestal
        || core.inside_temple_door
        || core.inside_apocalypse_temple_door
        || core.on_top_apocalypse_time_temple_platform
        || core.eaten_by_locust_queen
        || core.in_apocalypse_temple_fountain
        || core.thetan_brawl
        || core.on_closet_temple_thetan_meter
        || core.on_closet_temple_couch
        || core.in_closet_temple_closet_door
        || core.slamming_closet_temple_closet_door
        || core.held_by_xenu
        || core.held_in_toms_arms
        || core.holding_baby_pygmy
        || core.thrown_by_locusts
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.battle_of_the_gods
        || core.zombie_explode
        || core.is_ghost
        || core.touched_by_ghost
        || core.possessed_by_thetan
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.skewered
        || core.underwater
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.crushed
        || core.caught_by_bird
        || core.falling
        || core.in_crack_hole
        || core.doodle_jumping
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.cooked
        || core.killed_by_plague
        || core.drowning
}

/// The containers of FREE: the Kong statue, the ape wench, bongo and altar, the go-go cage
/// and the Dance Dance Execution.
fn is_in_container(core: &PygmyPygmyCoreFrame) -> bool {
    core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
}

/// `mbBlownByHurricane || mbPickedUpByHurricane`.
fn is_held_by_hurricane(sprite: &SpriteLogic) -> bool {
    sprite.blown_by_hurricane || sprite.picked_up_by_hurricane
}

/// The lightning guard: FREE without the hurricane flags, the containers and `mbIsZombie`.
fn is_busy_for_lightning(core: &PygmyPygmyCoreFrame) -> bool {
    is_busy_base(core) || core.touch_began_inside || core.time_temple_pygmy_type != 0 || core.dragging
}

/// The whole busy set FREE.
fn is_busy(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);

    is_busy_for_lightning(core) || is_held_by_hurricane(sprite) || is_in_container(core) || core.is_zombie
}

/// The stop list S14 (`docs/re/pygmy.md` §1.3).
fn stop_activities(world: &mut World, this: Entity) {
    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_being_eaten(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);
}

/// The stop list S12 (`docs/re/pygmy.md` §1.3): S14 without `stopSleeping` and
/// `stopBeingEaten`, so a sleeping pygmy can be hit in its sleep.
fn stop_activities_keep_sleeping(world: &mut World, this: Entity) {
    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);
}

/// `vel.xy = 0`: only x and y of `mPosVel` are cleared.
fn clear_xy_vel(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;
}

/// `[mDisplayObject setZRot:0]`.
fn clear_z_rot(world: &mut World, this: Entity) {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).rot.z = 0.0;
}

/// `mSacrificeId = @"…"`.
fn set_sacrifice_id(world: &mut World, this: Entity, sacrifice_id: &str) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(sacrifice_id.to_owned());
}

/// The body `hitByBirdPoo:`, `hitByFish:`, `hitBySwordFish:`, `hitByBanana:`,
/// `hitByCookedBird:` and `hitByCookedPygmy:` share: guard FREE +{mbSharkOnHead} while not
/// tilted, the stop list S12, then `behavior`, or `stopMonkeyHumping` and `asleep_behavior`
/// while asleep.
fn hit_by_projectile(world: &mut World, this: Entity, behavior: &str, asleep_behavior: &str) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let blocked = core.tilt_state != 0 || core.shark_on_head;

    if blocked || is_busy(world, this) {
        return false;
    }

    stop_activities_keep_sleeping(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    clear_xy_vel(world, this);
    clear_z_rot(world, this);

    let sleeping = world.obj::<PygmyPygmyCoreFrame>(this).sleeping;
    let behavior = if sleeping {
        stop_monkey_humping(world, this);
        asleep_behavior
    } else {
        behavior
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[PygmyLogic (Island) throwFromExplosion:]` @ 0x10009731c
///
/// `explosion_pos` is the exploding pygmy's position (the original passes a pointer to it).
pub fn throw_from_explosion(world: &mut World, this: Entity, explosion_pos: Vec4) {
    let skip_explosion = world.obj::<PygmyIslandHazards>(this).warlock_skip_next_explosion;

    if !skip_explosion && !is_busy(world, this) {
        stop_activities(world, this);

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.falling = true;
        core.tilt_state = 0;

        clear_z_rot(world, this);

        let display = pg_engine::logic::display_object(world, this);
        let pos = pg_engine::display::object(world, display).pos;
        let x_distance = explosion_pos.x - pos.x;

        // The ratio is taken in double precision and narrowed before `expf`.
        let ratio = (f64::from(x_distance) / 200.0).abs() as f32;
        let strength = (-ratio).exp();

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = strength * 100.0;
        logic.pos_vel.y = strength * 1000.0;

        // `b.le` also takes the NaN case, so only a positive distance faces right.
        if x_distance > 0.0 {
            pg_engine::display::object_mut(world, display).scale.x = 1.0;

            let mut logic = pg_engine::logic::logic_mut(world, this);
            logic.pos_vel.x = -logic.pos_vel.x;
        } else {
            pg_engine::display::object_mut(world, display).scale.x = -1.0;
        }

        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = false;
        set_sacrifice_id(world, this, "ThrownExplosionDrown");

        pg_engine::logic::set_behavior(world, this, Some("PygmyThrowFromExplosion"));
    }

    world.obj_mut::<PygmyIslandHazards>(this).warlock_skip_next_explosion = false;
}

/// `-[PygmyLogic (Island) hitByLightning]` @ 0x100097c2c
pub fn hit_by_lightning(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let is_live_zombie = core.is_zombie && !core.is_ghost;
    let busy = is_busy_for_lightning(core);

    if is_live_zombie {
        zombie_explode(world, this);
        return true;
    }

    if busy {
        return false;
    }

    stop_activities(world, this);

    let earthquake = world.obj::<PygmyPygmyCoreFrame>(this).earthquake;
    if earthquake {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).earthquake = false;

        let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_pygmy_earthquake_stop(world, listener, this);
        }
    }

    // The hurricane forgets the pygmy without calling back `droppedByHurricane:`, so
    // `mbPickedUpByHurricane` stays set.
    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        crate::logic::sky_powers::hurricane_logic::drop_sprite_logic(world, hurricane, this);
        world.obj_mut::<SpriteLogic>(this).picked_up_by_hurricane_logic = None;

        let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_pygmy_hit_by_lightning_in_hurricane(world, listener, this);
        }
    }

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.hit_by_lightning = true;
    core.tilt_state = 0;

    clear_xy_vel(world, this);
    clear_z_rot(world, this);
    set_sacrifice_id(world, this, "Lightning");

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let in_kong_statue = core.in_kong_statue;
    let in_ape_container = core.in_ape_wench || core.in_ape_bongo || core.in_ape_altar;

    let behavior = if in_kong_statue {
        let kong_statue = world.obj::<PygmyPygmyCoreFrame>(this).kong_statue_logic;
        if let Some(kong_statue) = kong_statue {
            crate::logic::konkeydong::kong_statue_logic::remove_pygmy_from_kong_statue(world, kong_statue, this);
        }
        world.obj_mut::<PygmyPygmyCoreFrame>(this).kong_statue_logic = None;

        "PygmyHitByLightningInKongStatue"
    } else if in_ape_container {
        // Bongo, wench, altar: each flag is read again after the previous removal.
        let in_ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_bongo;
        if in_ape_bongo {
            let ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).ape_bongo_logic;
            if let Some(ape_bongo) = ape_bongo {
                crate::logic::ape_ritual::ape_bongo_logic::remove_pygmy_from_ape_bongo(world, ape_bongo, this);
            }
            world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_bongo_logic = None;
        }

        let in_ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_wench;
        if in_ape_wench {
            let ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).ape_wench_logic;
            if let Some(ape_wench) = ape_wench {
                crate::logic::ape_ritual::ape_wench_logic::remove_pygmy_from_ape_wench(world, ape_wench, this);
            }
            world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_wench_logic = None;
        }

        let in_ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_altar;
        if in_ape_altar {
            let ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).ape_altar_logic;
            if let Some(ape_altar) = ape_altar {
                crate::logic::ape_ritual::ape_altar_logic::remove_pygmy_from_ape_altar(world, ape_altar, this);
            }
            world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_altar_logic = None;
        }

        "PygmyHitByLightningInApeSomething"
    } else {
        let picked_up_by_hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;

        if picked_up_by_hurricane {
            set_sacrifice_id(world, this, "HurricaneLightning");
            "PygmyHitByLightningInHurricane"
        } else {
            "PygmyHitByLightning"
        }
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[PygmyLogic (Island) canBeHitByLightning]` @ 0x100098630
pub fn can_be_hit_by_lightning(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let is_live_zombie = core.is_zombie && !core.is_ghost;

    is_live_zombie || !is_busy_for_lightning(core)
}

/// `-[PygmyLogic (Island) explodeFromLovingPocketGod]` @ 0x100098c30
pub fn explode_from_loving_pocket_god(world: &mut World, this: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let busy = is_busy_for_lightning(core) || is_in_container(core) || core.is_zombie;

    if busy {
        return;
    }

    stop_staring_at_rainbow(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).explode_from_loving_pocket_god = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyExplodeFromLovingPocketGod"));
}

/// `-[PygmyLogic (Island) hitByLava:]` @ 0x1000992dc
pub fn hit_by_lava(world: &mut World, this: Entity, _lava: Entity) {
    if is_busy(world, this) {
        return;
    }

    // S14 without `closeWordBalloon`.
    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_being_eaten(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    stop_charlie_mischief(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.on_fire_from_lava = true;
    core.tilt_state = 0;

    clear_z_rot(world, this);
    set_sacrifice_id(world, this, "Lava");

    pg_engine::logic::set_behavior(world, this, Some("PygmyHitByLava"));
}

/// `-[PygmyLogic (Island) hitByIcicle:]` @ 0x100099aec
pub fn hit_by_icicle(world: &mut World, this: Entity, _icicle: Entity) -> bool {
    if is_busy(world, this) {
        return false;
    }

    stop_activities(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.hit_by_icicle = true;
    core.tilt_state = 0;

    clear_xy_vel(world, this);
    clear_z_rot(world, this);
    set_sacrifice_id(world, this, "IciclePinCushion");

    pg_engine::logic::set_behavior(world, this, Some("PygmyHitByIcicle"));
    true
}

/// `-[PygmyLogic (Island) hitByBonFire:]` @ 0x10009a328
pub fn hit_by_bon_fire(world: &mut World, this: Entity, _bon_fire: Entity) {
    if is_busy(world, this) {
        return;
    }

    stop_activities(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.on_fire_from_bon_fire = true;
    core.tilt_state = 0;

    clear_z_rot(world, this);
    set_sacrifice_id(world, this, "BonFire");

    pg_engine::logic::set_behavior(world, this, Some("PygmyHitByBonFire"));
}

/// `-[PygmyLogic (Island) hitByCoconut:]` @ 0x10009ab4c
///
/// A knock-out, not a death: no sacrifice id. The temple pygmy types are not excluded.
pub fn hit_by_coconut(world: &mut World, this: Entity, _coconut: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);

    let blocked = core.tilt_state != 0 || core.shark_on_head;
    let busy = is_busy_base(core)
        || core.touch_began_inside
        || core.dragging
        || is_held_by_hurricane(sprite)
        || is_in_container(core)
        || core.is_zombie;

    if blocked || busy {
        return false;
    }

    stop_activities_keep_sleeping(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    clear_xy_vel(world, this);
    clear_z_rot(world, this);

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let pygmy_type = core.time_temple_pygmy_type;
    let sleeping = core.sleeping;

    let behavior = if pygmy_type != 0 {
        "TemplePygmyHitByCoconut"
    } else if sleeping {
        stop_monkey_humping(world, this);
        "PygmyHitByCoconutAsleep"
    } else {
        "PygmyHitByCoconut"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[PygmyLogic (Island) hitByBirdPoo:]` @ 0x10009b3b8
pub fn hit_by_bird_poo(world: &mut World, this: Entity, _bird_poo: Entity) -> bool {
    hit_by_projectile(world, this, "PygmyHitByBirdPoo", "PygmyHitByBirdPooAsleep")
}

/// `-[PygmyLogic (Island) hitBySnowBall:]` @ 0x10009bc0c
///
/// Also hits a pygmy that is being dragged: the drag ends first.
pub fn hit_by_snow_ball(world: &mut World, this: Entity, _snow_ball: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);

    let blocked = core.tilt_state != 0 || core.shark_on_head;
    let busy = is_busy_base(core)
        || core.time_temple_pygmy_type != 0
        || is_held_by_hurricane(sprite)
        || is_in_container(core)
        || core.is_zombie;

    if blocked || busy {
        return false;
    }

    let dragging = world.obj::<PygmyPygmyCoreFrame>(this).dragging;
    if dragging {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.dragging = false;
        core.apply_spring = false;
        core.drag_label_x_offset = 0.0;
        core.drag_label_y_offset = 0.0;

        clear_z_rot(world, this);

        let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_pygmy_drag_complete(world, listener, this);
        }

        let display = pg_engine::logic::display_object(world, this);
        let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
        let marker = listener.and_then(|listener| crate::controller::pygmy_display_marker(world, listener));
        if let Some(marker) = marker {
            pg_engine::display::insert_after(world, marker, Some(display));
        }

        update_label_layer(world, this);

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.enable_island_floor_collision = true;
        let island_y_pos = core.island_y_pos;

        pg_engine::display::object_mut(world, display).pos.y = island_y_pos;
    }

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.touch_began_inside = false;
    core.active_touch = None;

    stop_activities_keep_sleeping(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.hit_by_snow_ball = true;
    core.tilt_state = 0;

    clear_xy_vel(world, this);
    set_sacrifice_id(world, this, "HitBySnowBall");
    clear_z_rot(world, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyHitBySnowBall"));
    true
}

/// `-[PygmyLogic (Island) hitByFish:]` @ 0x10009c55c
pub fn hit_by_fish(world: &mut World, this: Entity, _fish: Entity) -> bool {
    hit_by_projectile(world, this, "PygmyHitByFish", "PygmyHitByFishAsleep")
}

/// `-[PygmyLogic (Island) hitBySwordFish:]` @ 0x10009cdb0
pub fn hit_by_sword_fish(world: &mut World, this: Entity, _sword_fish: Entity) -> bool {
    hit_by_projectile(world, this, "PygmyHitBySwordFish", "PygmyHitBySwordFishAsleep")
}

/// `-[PygmyLogic (Island) hitByBanana:]` @ 0x10009d604
pub fn hit_by_banana(world: &mut World, this: Entity, _banana: Entity) -> bool {
    hit_by_projectile(world, this, "PygmyHitByBanana", "PygmyHitByBananaAsleep")
}

/// `-[PygmyLogic (Island) hitByCookedBird:]` @ 0x10009de58
pub fn hit_by_cooked_bird(world: &mut World, this: Entity, _cooked_bird: Entity) -> bool {
    hit_by_projectile(world, this, "PygmyHitByCookedBird", "PygmyHitByCookedBirdAsleep")
}

/// `-[PygmyLogic (Island) hitByCookedPygmy:]` @ 0x10009e6ac
pub fn hit_by_cooked_pygmy(world: &mut World, this: Entity, _cooked_pygmy: Entity) -> bool {
    hit_by_projectile(world, this, "PygmyHitByCookedPygmy", "PygmyHitByCookedPygmyAsleep")
}
