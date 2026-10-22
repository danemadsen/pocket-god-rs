//! `MoonLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `MoonLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MoonLogic {
    /// `mMoonLogicListener`: `@"<MoonLogicListener>"`
    pub moon_logic_listener: Option<Entity>,

    /// `mStarsLogic`: `@"MultiModelLogic"`
    pub stars_logic: Option<Entity>,

    /// `mAlpha`: `f`
    pub alpha: f32,

    /// `mTargetAlpha`: `f`
    pub target_alpha: f32,

    /// `mTargetYPos`: `f`
    pub target_y_pos: f32,

    /// `mbTransitionIn`: `B`
    pub transition_in: bool,

    /// `mbTransitionOut`: `B`
    pub transition_out: bool,
}

pub const MOON_LOGIC: Class = Class {
    name: "MoonLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `[[MoonLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&MOON_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[MoonLogic initWithDisplayObject:]` @ 0x1000ca64c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(MoonLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<MoonLogic>(this).transition_in = false;
    display::remove_from_display_group(world, display_object);
}

/// `-[MoonLogic setMoonLogicListener:]` @ 0x1000ca6c8
pub fn set_moon_logic_listener(world: &mut World, this: Entity, moon_logic_listener: Entity) {
    world.obj_mut::<MoonLogic>(this).moon_logic_listener = Some(moon_logic_listener);
}

/// `-[MoonLogic onGameFrame:]` @ 0x1000ca788
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let moon = world.obj::<MoonLogic>(this);
    if !moon.transition_in && !moon.transition_out {
        return;
    }
    let target_y_pos = moon.target_y_pos;

    let y_pos = {
        let mut display = display::object_mut(world, display_object);
        display.pos.y = display.pos.y + (target_y_pos - display.pos.y) * 0.5;
        display.pos.y
    };

    let (alpha, stars_logic, transition_out) = {
        let mut moon = world.obj_mut::<MoonLogic>(this);
        moon.alpha = moon.alpha + (moon.target_alpha - moon.alpha) * 0.5;
        (moon.alpha, moon.stars_logic, moon.transition_out)
    };

    if y_pos != target_y_pos {
        if let Some(stars_logic) = stars_logic {
            let stars_display_object = logic::display_object(world, stars_logic);
            display::object_mut(world, stars_display_object).set_alpha(alpha);
        }
        return;
    }

    if transition_out {
        display::remove_from_display_group(world, display_object);
        if let Some(stars_logic) = stars_logic {
            let stars_display_object = logic::display_object(world, stars_logic);
            display::remove_from_display_group(world, stars_display_object);
        }
        world.obj_mut::<MoonLogic>(this).stars_logic = None;
    }

    let mut moon = world.obj_mut::<MoonLogic>(this);
    moon.transition_in = false;
    moon.transition_out = false;
}

/// `-[MoonLogic transitionIn:]` @ 0x1000ca900
///
/// Brings the moon and the star field in (after `StarDisplayMarker`), the stars fading in.
pub fn transition_in(world: &mut World, this: Entity, transition_in: f32) {
    let listener = {
        let mut moon = world.obj_mut::<MoonLogic>(this);
        moon.transition_in = true;
        moon.target_y_pos = transition_in;
        moon.alpha = 0.0;
        moon.target_alpha = 1.0;
        moon.moon_logic_listener
    };

    let stars_logic = match listener {
        Some(listener) => crate::controller::stars_logic(world, listener, this),
        None => None,
    };
    world.obj_mut::<MoonLogic>(this).stars_logic = stars_logic;

    let star_marker = pg_engine::library::display_marker(world, "StarDisplayMarker");
    let display_object = logic::display_object(world, this);
    let stars_display_object = match stars_logic {
        Some(stars_logic) => Some(logic::display_object(world, stars_logic)),
        None => None,
    };

    // A nil stars object still reaches `insertAfter:`.
    if let Some(star_marker) = star_marker {
        display::insert_after(world, star_marker, Some(display_object));
        display::insert_after(world, star_marker, stars_display_object);
    }

    let alpha = world.obj::<MoonLogic>(this).alpha;
    if let Some(stars_display_object) = stars_display_object {
        display::object_mut(world, stars_display_object).set_alpha(alpha);
    }
    if let Some(stars_logic) = stars_logic {
        logic::set_behavior(world, stars_logic, Some("StarsStartLoop"));
    }
}

/// `-[MoonLogic transitionOut:]` @ 0x1000caa48
///
/// Only while the stars are held: fades them out and stops their loop.
pub fn transition_out(world: &mut World, this: Entity, transition_out: f32) {
    let Some(stars_logic) = world.obj::<MoonLogic>(this).stars_logic else {
        return;
    };

    {
        let mut moon = world.obj_mut::<MoonLogic>(this);
        moon.transition_out = true;
        moon.target_y_pos = transition_out;
        moon.target_alpha = 0.0;
    }
    logic::set_behavior(world, stars_logic, Some("StarsStopLoop"));
}
