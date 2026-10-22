//! `FireworkLogic` (superclass `SpriteLogic`).

use std::f32::consts::TAU;

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::display::model;
use pg_engine::logic;

/// `FireworkLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct FireworkLogic {
    /// `mFireworkLogicListener`: `@"<FireworkLogicListener>"`
    pub firework_logic_listener: Option<Entity>,

    /// `mSparkDistance`: `f`
    pub spark_distance: f32,

    /// `mTargetDistance`: `f`
    pub target_distance: f32,

    /// `mSparkAlpha`: `f`
    pub spark_alpha: f32,

    /// `mTargetAlpha`: `f`
    pub target_alpha: f32,

    /// `mSparkArray`: `@"NSMutableArray"`
    pub spark_array: Vec<Entity>,

    /// `mStartTime`: `f`
    pub start_time: f32,
}

pub const FIREWORK_LOGIC: Class = Class {
    name: "FireworkLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The sparks of one burst.
const SPARK_COUNT: usize = 10;

/// `[[FireworkLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&FIREWORK_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[FireworkLogic initWithDisplayObject:]` @ 0x1000d4290
///
/// Ten `Firework` sparks copy the burst's colour, position and origin and go right after it;
/// the burst itself stops drawing its texture.
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(FireworkLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<FireworkLogic>(this).spark_array = Vec::with_capacity(5);

    let spark_texture = pg_engine::library::texture_def_id(world, "FireworkSpark");
    for _ in 0..SPARK_COUNT {
        let spark = model::spawn(world, Some("Firework"), spark_texture);

        let burst = display::object(world, display_object);
        let (red, green, blue) = (burst.red, burst.green, burst.blue);
        let burst_pos = burst.pos;
        let burst_origin = burst.origin_pos;

        {
            let mut spark_display = display::object_mut(world, spark);

            // `setRed:green:blue:`
            spark_display.red = red;
            spark_display.green = green;
            spark_display.blue = blue;
            spark_display.color = true;

            spark_display.pos.x = burst_pos.x;
            spark_display.pos.y = burst_pos.y;
            spark_display.pos.z = burst_pos.z;
            spark_display.origin_pos.x = burst_origin.x;
            spark_display.origin_pos.y = burst_origin.y;
        }

        display::insert_after(world, display_object, Some(spark));
        world.obj_mut::<FireworkLogic>(this).spark_array.push(spark);
    }

    display::set_texture_def(world, display_object, None);

    let distance_random = world.resource_mut::<Random>().arc4random();
    {
        let mut firework = world.obj_mut::<FireworkLogic>(this);
        firework.spark_distance = 0.0;
        firework.target_distance = (distance_random % 501 + 500) as f32 / 10.0;
        firework.spark_alpha = 1.0;
        firework.target_alpha = 1.0;
    }
    logic::set_behavior(world, this, Some("FireworkPop"));
}

/// `-[FireworkLogic setFireworkLogicListener:]` @ 0x1000d4644
pub fn set_firework_logic_listener(world: &mut World, this: Entity, firework_logic_listener: Entity) {
    world.obj_mut::<FireworkLogic>(this).firework_logic_listener = Some(firework_logic_listener);
}

/// `-[FireworkLogic onGameFrame:]` @ 0x1000d46a0
///
/// The sparks spread on a ring around the burst and fade out after 0.7 s; the burst dies after
/// 1.3 s.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let start_time = {
        let mut firework = world.obj_mut::<FireworkLogic>(this);
        firework.start_time = firework.start_time + dt;
        if firework.start_time > 0.7 {
            firework.target_alpha = 0.0;
        }
        firework.start_time
    };
    if start_time > 1.3 {
        die(world, this);
        return;
    }

    let (spark_distance, spark_alpha, sparks) = {
        let mut firework = world.obj_mut::<FireworkLogic>(this);
        firework.spark_distance = firework.spark_distance + (firework.target_distance - firework.spark_distance) * 0.1;
        firework.spark_alpha = firework.spark_alpha + (firework.target_alpha - firework.spark_alpha) * 0.1;
        (firework.spark_distance, firework.spark_alpha, firework.spark_array.clone())
    };

    let burst_pos = display::object(world, display_object).pos;
    let angle_step = TAU / sparks.len() as f32;
    let mut angle = 0.0_f32;
    for spark in sparks {
        let (sin, cos) = angle.sin_cos();

        let mut spark_display = display::object_mut(world, spark);
        spark_display.pos.x = burst_pos.x + cos * spark_distance;
        spark_display.pos.y = burst_pos.y + sin * spark_distance;
        spark_display.set_alpha(spark_alpha);

        angle = angle_step + angle;
    }
}

/// `-[FireworkLogic die]` @ 0x1000d4a20
///
/// Takes the sparks out of the display tree, releases them and tells the listener.
pub fn die(world: &mut World, this: Entity) {
    let sparks = world.obj::<FireworkLogic>(this).spark_array.clone();
    for &spark in &sparks {
        display::remove_from_display_group(world, spark);
    }

    // `removeAllObjects` released the sparks' last reference.
    world.obj_mut::<FireworkLogic>(this).spark_array.clear();
    for spark in sparks {
        world.despawn(spark);
    }

    let listener = world.obj::<FireworkLogic>(this).firework_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_firework_die(world, listener, this);
    }
}
