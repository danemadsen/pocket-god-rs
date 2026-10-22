//! `SunLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::display::group;
use pg_engine::display::model;
use pg_engine::logic;

/// `SunLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct SunLogic {
    /// `mSunLogicListener`: `@"<SunLogicListener>"`
    pub sun_logic_listener: Option<Entity>,

    /// `mbEclipseOn`: `B`
    pub eclipse_on: bool,

    /// `mEclipse1DisplayObject`: `@"BCDisplayObject"`
    pub eclipse1_display_object: Option<Entity>,

    /// `mEclipse2DisplayObject`: `@"BCDisplayObject"`
    pub eclipse2_display_object: Option<Entity>,

    /// `mEclipseRayDisplayObject`: `@"BCDisplayObject"`
    pub eclipse_ray_display_object: Option<Entity>,

    /// `mObstructionLogic`: `@"BCLogic"`
    pub obstruction_logic: Option<Entity>,

    /// `mTargetYPos`: `f`
    pub target_y_pos: f32,

    /// `mbTransitionIn`: `B`
    pub transition_in: bool,

    /// `mbTransitionOut`: `B`
    pub transition_out: bool,

    /// `mEclipseTargetPercent`: `f`
    pub eclipse_target_percent: f32,

    /// `mEclipsePercent`: `f`
    pub eclipse_percent: f32,

    /// `mTargetScale`: `f`
    pub target_scale: f32,
}

pub const SUN_LOGIC: Class = Class {
    name: "SunLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `[[SunLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&SUN_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[SunLogic initWithDisplayObject:]` @ 0x1000c7498
///
/// Builds the two `Eclipse` models that overlay the sun while something passes in front of it.
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(SunLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut sun = world.obj_mut::<SunLogic>(this);
        sun.transition_in = false;
        sun.target_scale = 1.0;
    }

    let eclipse_texture = pg_engine::library::texture_def_id(world, "Eclipse");
    let sun_origin = display::object(world, display_object).origin_pos;

    let eclipse1 = model::spawn(world, Some("Eclipse"), eclipse_texture);
    world.obj_mut::<SunLogic>(this).eclipse1_display_object = Some(eclipse1);
    {
        let mut eclipse1_display = display::object_mut(world, eclipse1);
        eclipse1_display.origin_pos.x = sun_origin.x;
        eclipse1_display.origin_pos.y = sun_origin.y;
    }

    let eclipse2 = model::spawn(world, Some("Eclipse"), eclipse_texture);
    world.obj_mut::<SunLogic>(this).eclipse2_display_object = Some(eclipse2);
    let mut eclipse2_display = display::object_mut(world, eclipse2);
    eclipse2_display.origin_pos.x = sun_origin.x;
    eclipse2_display.origin_pos.y = sun_origin.y;
}

/// `-[SunLogic setSunLogicListener:]` @ 0x1000c767c
pub fn set_sun_logic_listener(world: &mut World, this: Entity, sun_logic_listener: Entity) {
    world.obj_mut::<SunLogic>(this).sun_logic_listener = Some(sun_logic_listener);
}

/// `-[SunLogic onGameFrame:]` @ 0x1000c7768
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let sun = world.obj::<SunLogic>(this);
    let transitioning = sun.transition_in || sun.transition_out;
    let target_y_pos = sun.target_y_pos;
    if transitioning {
        let y_pos = {
            let mut display = display::object_mut(world, display_object);
            display.pos.y = display.pos.y + (target_y_pos - display.pos.y) * 0.5;
            display.pos.y
        };

        if y_pos == target_y_pos {
            let transition_out = world.obj::<SunLogic>(this).transition_out;
            if transition_out {
                display::remove_from_display_group(world, display_object);
            }

            let mut sun = world.obj_mut::<SunLogic>(this);
            sun.transition_in = false;
            sun.transition_out = false;
        }
    }

    let (eclipse_percent, eclipse1, eclipse2, target_scale) = {
        let mut sun = world.obj_mut::<SunLogic>(this);
        sun.eclipse_percent = sun.eclipse_percent + (sun.eclipse_target_percent - sun.eclipse_percent) * 0.13;
        (sun.eclipse_percent, sun.eclipse1_display_object, sun.eclipse2_display_object, sun.target_scale)
    };

    // The two eclipse layers spin in opposite directions; the steps are added in double.
    if eclipse_percent > 0.0 {
        if let Some(eclipse1) = eclipse1 {
            let mut eclipse1_display = display::object_mut(world, eclipse1);
            eclipse1_display.rot.z = (f64::from(eclipse1_display.rot.z) + 0.1) as f32;
        }
        if let Some(eclipse2) = eclipse2 {
            let mut eclipse2_display = display::object_mut(world, eclipse2);
            eclipse2_display.rot.z = (f64::from(eclipse2_display.rot.z) - 0.13) as f32;
        }
    }

    let mut display = display::object_mut(world, display_object);
    let scale = display.scale.x + (target_scale - display.scale.x) * 0.2;
    display.scale.x = scale;
    display.scale.y = scale;
}

/// `-[SunLogic setSunState:]` @ 0x1000c7930
pub fn set_sun_state(world: &mut World, this: Entity, sun_state: i32) {
    let target_scale = match sun_state {
        2 => 0.5,
        1 => 1.5,
        0 => 1.0,
        _ => return,
    };
    world.obj_mut::<SunLogic>(this).target_scale = target_scale;
}

/// `-[SunLogic setEclipsePercent:obstructionLogic:]` @ 0x1000c7978
///
/// `checkEclipse` passes nil with a percentage of 0.
pub fn set_eclipse_percent(world: &mut World, this: Entity, eclipse_percent: f32, obstruction_logic: Option<Entity>) {
    {
        let mut sun = world.obj_mut::<SunLogic>(this);
        sun.obstruction_logic = obstruction_logic;
        sun.eclipse_target_percent = eclipse_percent;
    }

    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;

    let sun = world.obj::<SunLogic>(this);
    let eclipse_on = sun.eclipse_on;
    let eclipse1 = sun.eclipse1_display_object;
    let eclipse2 = sun.eclipse2_display_object;
    let eclipses = [eclipse1, eclipse2];

    if eclipse_percent == 0.0 {
        if eclipse_on {
            world.obj_mut::<SunLogic>(this).eclipse_on = false;
            for eclipse in eclipses.into_iter().flatten() {
                display::remove_from_display_group(world, eclipse);
            }
        }
    } else if !eclipse_on {
        world.obj_mut::<SunLogic>(this).eclipse_on = true;

        let sun_pos = display::object(world, display_object).pos;
        let sun_index = match display_group {
            Some(display_group) => group::index_of_display_object(world, display_group, display_object),
            None => 0,
        };
        let eclipse_index = sun_index + 1;

        for eclipse in eclipses.into_iter().flatten() {
            {
                let mut eclipse_display = display::object_mut(world, eclipse);
                eclipse_display.pos.x = sun_pos.x;
                eclipse_display.pos.y = sun_pos.y;
                eclipse_display.pos.z = sun_pos.z;
                eclipse_display.set_alpha(0.0);
            }
            display::move_to_display_group_at_index(world, eclipse, display_group, eclipse_index);
        }
    }

    // The original then reads the sun's and the obstruction's positions without using them.
    let eclipse_target_percent = world.obj::<SunLogic>(this).eclipse_target_percent;
    for eclipse in eclipses.into_iter().flatten() {
        display::object_mut(world, eclipse).set_alpha(eclipse_target_percent);
    }
}

/// `-[SunLogic transitionIn:]` @ 0x1000c7c08
pub fn transition_in(world: &mut World, this: Entity, transition_in: f32) {
    {
        let mut sun = world.obj_mut::<SunLogic>(this);
        sun.transition_in = true;
        sun.target_y_pos = transition_in;
    }

    let star_marker = pg_engine::library::display_marker(world, "StarDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(star_marker) = star_marker {
        display::insert_after(world, star_marker, Some(display_object));
    }
}

/// `-[SunLogic transitionOut:]` @ 0x1000c7c8c
pub fn transition_out(world: &mut World, this: Entity, transition_out: f32) {
    let mut sun = world.obj_mut::<SunLogic>(this);
    sun.transition_out = true;
    sun.target_y_pos = transition_out;
}
