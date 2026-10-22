//! `SpearStatueLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::underwater_stage::chain_spear_logic;
use pg_engine::{animation, display, library, logic};

/// `SpearStatueLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct SpearStatueLogic {
    /// `mSpearStatueLogicListener`: `@"<SpearStatueLogicListener>"`
    pub spear_statue_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mChainSpearDisplayObject`: `@"BCDisplayObject"`
    pub chain_spear_display_object: Option<Entity>,

    /// `mChainSpearLogic`: `@"ChainSpearLogic"`
    pub chain_spear_logic: Option<Entity>,
}

pub const SPEAR_STATUE_LOGIC: Class = Class {
    name: "SpearStatueLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
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

/// The statue's animation channel the chain spear hangs on.
const CHAIN_SPEAR_CHANNEL: usize = 1;

/// `[[SpearStatueLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&SPEAR_STATUE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[SpearStatueLogic initWithDisplayObject:]` @ 0x1000c3dc0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(SpearStatueLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
    logic::set_behavior(world, this, Some("SpearStatueInit"));
}

/// `-[SpearStatueLogic setSpearStatueLogicListener:]` @ 0x1000c3e4c
pub fn set_spear_statue_logic_listener(world: &mut World, this: Entity, spear_statue_logic_listener: Entity) {
    world.obj_mut::<SpearStatueLogic>(this).spear_statue_logic_listener = Some(spear_statue_logic_listener);
}

/// `-[SpearStatueLogic pygmyLogicArray:]` @ 0x1000c3ea8
///
/// Forwards to the listener with `self` as the argument (the caller's is ignored).
pub fn pygmy_logic_array(world: &mut World, this: Entity, _pygmy_logic_array: Entity) -> Vec<Entity> {
    let listener = world.obj::<SpearStatueLogic>(this).spear_statue_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::pygmy_logic_array(world, listener, this)
}

/// `-[SpearStatueLogic onPygmiesSkewered:]` @ 0x1000c3ecc
pub fn on_pygmies_skewered(world: &mut World, this: Entity, pygmies_skewered: i32) {
    let listener = world.obj::<SpearStatueLogic>(this).spear_statue_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_pygmies_skewered(world, listener, pygmies_skewered);
    }
}

/// `-[SpearStatueLogic removeDisplayObjects]` @ 0x1000c3ee8
///
/// Takes the chain spear down. The statue holds the spear's last reference, so the spear logic
/// is deallocated here, and the `ChainSpear` multimodel with it (its group let go of it first).
pub fn remove_display_objects(world: &mut World, this: Entity) {
    let own_animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, own_animation, CHAIN_SPEAR_CHANNEL);

    let chain_spear_display_object = world.obj::<SpearStatueLogic>(this).chain_spear_display_object;
    if let Some(chain_spear_display_object) = chain_spear_display_object {
        display::remove_from_display_group(world, chain_spear_display_object);
    }
    world.obj_mut::<SpearStatueLogic>(this).chain_spear_display_object = None;

    let chain_spear = world.obj::<SpearStatueLogic>(this).chain_spear_logic;
    if let Some(chain_spear) = chain_spear {
        logic::shutdown(world, chain_spear);
        chain_spear_logic::dealloc(world, chain_spear);
        world.obj_mut::<SpearStatueLogic>(this).chain_spear_logic = None;
    }

    if let Some(chain_spear_display_object) = chain_spear_display_object {
        world.despawn(chain_spear_display_object);
    }
}

/// `-[SpearStatueLogic onGameFrame:]` @ 0x1000c3f98
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[SpearStatueLogic transitionIn]` @ 0x1000c3f9c
///
/// The statue comes in with a new chain spear at (18, 19) on its own depth, tilted 40°, drawn
/// just before it and carried on channel 1.
pub fn transition_in(world: &mut World, this: Entity) {
    let own_display_object = logic::display_object(world, this);
    let marker = library::display_marker(world, "TsunamiDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(own_display_object));
    }

    let chain_spear_display_object = display::multi_model::spawn(world, Some("ChainSpear"));
    let statue_z_pos = display::object(world, own_display_object).pos.z;
    {
        let mut object = display::object_mut(world, chain_spear_display_object);
        object.pos.x = 18.0;
        object.pos.y = 19.0;
        object.pos.z = statue_z_pos;
        object.rot.z = 40.0;
    }
    display::insert_before(world, own_display_object, Some(chain_spear_display_object));
    world.obj_mut::<SpearStatueLogic>(this).chain_spear_display_object = Some(chain_spear_display_object);

    let own_animation = logic::animation(world, this);
    animation::set_tweenable(world, own_animation, Some(chain_spear_display_object), CHAIN_SPEAR_CHANNEL);

    let chain_spear = chain_spear_logic::spawn(world, chain_spear_display_object);
    world.obj_mut::<SpearStatueLogic>(this).chain_spear_logic = Some(chain_spear);
    chain_spear_logic::set_chain_spear_logic_listener(world, chain_spear, this);
}

/// `-[SpearStatueLogic transitionOut]` @ 0x1000c4130
pub fn transition_out(world: &mut World, this: Entity) {
    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
    remove_display_objects(world, this);
}

/// `-[SpearStatueLogic showSpear]` @ 0x1000c417c
pub fn show_spear(world: &mut World, this: Entity) {
    world.obj_mut::<SpearStatueLogic>(this).active = true;

    let chain_spear = world.obj::<SpearStatueLogic>(this).chain_spear_logic;
    if let Some(chain_spear) = chain_spear {
        chain_spear_logic::activate(world, chain_spear);
    }
}

/// `-[SpearStatueLogic hideSpear]` @ 0x1000c41a8
pub fn hide_spear(world: &mut World, this: Entity) {
    world.obj_mut::<SpearStatueLogic>(this).active = false;

    let chain_spear = world.obj::<SpearStatueLogic>(this).chain_spear_logic;
    if let Some(chain_spear) = chain_spear {
        chain_spear_logic::clear_skewered_pygmies(world, chain_spear);
    }
    remove_display_objects(world, this);
}

/// `-[SpearStatueLogic touchBeganInside:allTouches:]` @ 0x1000c4270
///
/// A tap on the armed statue fires the spear.
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let statue = world.obj::<SpearStatueLogic>(this);
    if !statue.active {
        return;
    }

    let chain_spear = statue.chain_spear_logic;
    if let Some(chain_spear) = chain_spear {
        chain_spear_logic::shoot_spear(world, chain_spear);
    }
}

/// `-[SpearStatueLogic touchBeganOutside:allTouches:]` @ 0x1000c42a0
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpearStatueLogic touchMovedEntered:allTouches:]` @ 0x1000c42a4
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpearStatueLogic touchMovedExited:allTouches:]` @ 0x1000c42a8
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpearStatueLogic touchMovedInside:allTouches:]` @ 0x1000c42ac
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[SpearStatueLogic touchMovedOutside:allTouches:]` @ 0x1000c42bc
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpearStatueLogic touchEndedInside:allTouches:]` @ 0x1000c42c0
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpearStatueLogic touchEndedOutside:allTouches:]` @ 0x1000c42c4
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
