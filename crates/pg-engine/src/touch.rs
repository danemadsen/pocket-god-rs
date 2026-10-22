//! `BCTouch` and the touch event queue (`docs/re/rendering.md` §9.1–§9.2, `docs/re/logic.md` §9).
//!
//! A `BCTouch` entity is created for each finger the first time it reaches the queue and is
//! never despawned: gameplay keeps weak references to touches (`mActiveTouch`) and compares
//! them after the finger lifted, which a despawned entity would turn into a panic.

use bevy_ecs::prelude::*;
use bevy_math::Vec4;

use crate::hooks;
use crate::math::VEC4_ORIGIN;
use crate::object::{Class, Isa, ObjectWorldExt};
use crate::view::BcView;

/// `UITouchPhase`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TouchPhase {
    #[default]
    Began = 0,
    Moved = 1,
    Stationary = 2,
    Ended = 3,
    Cancelled = 4,
}

/// `BCTouch` ivars. One per finger for the finger's whole life.
#[derive(Component, Clone, Debug)]
pub struct BcTouch {
    /// Platform identifier of the finger (`mUITouch`).
    pub ui_touch: u64,
    pub touch_phase: TouchPhase,
    pub time_stamp: f32,
    pub game_time: f32,
    pub tap_count: i32,

    /// View points (origin top-left), z = 0, w = 1.
    pub pos: Vec4,

    /// The logic currently holding the touch.
    pub active_object: Option<Entity>,
}

impl Default for BcTouch {
    /// `-[BCTouch init]` @ 0x100177de0: everything zero, `pos = (0, 0, 0, 1)`.
    fn default() -> Self {
        Self {
            ui_touch: 0,
            touch_phase: TouchPhase::Began,
            time_stamp: 0.0,
            game_time: 0.0,
            tap_count: 0,
            pos: VEC4_ORIGIN,
            active_object: None,
        }
    }
}

pub const BC_TOUCH: Class = Class {
    name: "BCTouch",
    superclass: None,
    ..Class::NS_OBJECT
};

/// A raw platform touch, in view points (`touchWidth × touchHeight`, origin top-left).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlatformTouch {
    pub id: u64,
    pub phase: TouchPhase,
    pub time_stamp: f64,
    pub tap_count: i32,
    pub x: f32,
    pub y: f32,
}

/// Touches delivered by the platform since the last game frame.
#[derive(Resource, Default)]
pub struct PlatformTouches(pub Vec<PlatformTouch>);

/// Number of slots in `BCView`'s touch event ring.
pub const TOUCH_EVENT_RING_SIZE: usize = 1000;

/// One slot of the ring (0x30-byte records at `DAT_1008162d0`).
#[derive(Clone, Copy, Debug, Default)]
pub struct TouchEvent {
    /// The `BCTouch` (nil in a slot that was never written).
    pub touch: Option<Entity>,

    /// Set when the slot is written, cleared once it was dispatched.
    pub valid: bool,
    pub phase: TouchPhase,
    pub time_stamp: f32,
    pub tap_count: i32,
    pub x: f32,
    pub y: f32,
}

/// `BCView`'s touch event ring: 1000 slots, a read index (`DAT_1008162c8`) and a write index
/// (`DAT_1008162cc`), with no overflow check.
#[derive(Clone, Debug)]
pub struct TouchEventRing {
    pub events: Vec<TouchEvent>,
    pub read_index: i32,
    pub write_index: i32,
}

impl Default for TouchEventRing {
    fn default() -> Self {
        Self {
            events: vec![TouchEvent::default(); TOUCH_EVENT_RING_SIZE],
            read_index: 0,
            write_index: 0,
        }
    }
}

/// The ring index after `index` (999 wraps to 0).
fn next_ring_index(index: i32) -> i32 {
    if index < 999 { index + 1 } else { 0 }
}

/// `[[BCTouch alloc] init]` followed by `setUITouch:`.
pub fn spawn_touch(world: &mut World, ui_touch: u64) -> Entity {
    let touch = BcTouch {
        ui_touch,
        ..BcTouch::default()
    };
    world.spawn((Isa(&BC_TOUCH), touch)).id()
}

/// `BCView::getTouch:` @ 0x1001770b0: the `BCTouch` of this finger in `mUITouchArray`, or a
/// new one appended to it.
pub fn get_touch(world: &mut World, ui_touch: u64) -> Entity {
    let ui_touches = world.resource::<BcView>().ui_touches.clone();
    for touch in ui_touches {
        if world.obj::<BcTouch>(touch).ui_touch == ui_touch {
            return touch;
        }
    }

    let touch = spawn_touch(world, ui_touch);
    world.resource_mut::<BcView>().ui_touches.push(touch);
    touch
}

/// `BCView::createTouchEvents:withEvent:` @ 0x100177264 for one platform touch.
pub fn create_touch_event(world: &mut World, touch: PlatformTouch) {
    let touch_active = world.resource::<BcView>().touch_active;
    if !touch_active {
        if touch.phase == TouchPhase::Began {
            let pre_start_touch_began = hooks::hooks(world).pre_start_touch_began;
            pre_start_touch_began(world, touch.x, touch.y);
        }
        return;
    }

    let bc_touch = get_touch(world, touch.id);

    let mut view = world.resource_mut::<BcView>();
    let finished = touch.phase == TouchPhase::Ended || touch.phase == TouchPhase::Cancelled;
    if finished {
        view.ui_touches.retain(|entry| *entry != bc_touch);
    }

    let ring = &mut view.touch_events;
    let slot = ring.write_index;
    ring.write_index = next_ring_index(slot);
    ring.events[slot as usize] = TouchEvent {
        touch: Some(bc_touch),
        valid: true,
        phase: touch.phase,
        time_stamp: touch.time_stamp as f32,
        tap_count: touch.tap_count,
        x: touch.x,
        y: touch.y,
    };
}

/// `-createTouchEvents:withEvent:` over every touch the platform delivered since the last
/// tick, in delivery order.
pub fn create_touch_events(world: &mut World) {
    let Some(mut platform_touches) = world.get_resource_mut::<PlatformTouches>() else {
        return;
    };
    let touches = std::mem::take(&mut platform_touches.0);
    for touch in touches {
        create_touch_event(world, touch);
    }
}

/// `BCView::processTouchEvents:` @ 0x100176c54: drain the queue into the view delegate.
pub fn process_touch_events(world: &mut World, game_time: f32) {
    loop {
        let (slot, event) = {
            let ring = &world.resource::<BcView>().touch_events;
            if ring.read_index == ring.write_index {
                return;
            }
            let slot = ring.read_index as usize;
            (slot, ring.events[slot])
        };
        if !event.valid {
            return;
        }
        {
            let ring = &mut world.resource_mut::<BcView>().touch_events;
            ring.read_index = next_ring_index(ring.read_index);
        }
        let Some(touch) = event.touch else {
            panic!("touch event slot {slot} is valid without a BCTouch");
        };

        {
            let mut bc_touch = world.obj_mut::<BcTouch>(touch);
            bc_touch.touch_phase = event.phase;
            bc_touch.time_stamp = event.time_stamp;
            bc_touch.game_time = game_time;
            bc_touch.tap_count = event.tap_count;
            bc_touch.pos.x = event.x;
            bc_touch.pos.y = event.y;
        }
        {
            let mut view = world.resource_mut::<BcView>();
            if !view.game_touches.contains(&touch) {
                view.game_touches.push(touch);
            }
        }

        let delegate = hooks::hooks(world);
        match event.phase {
            TouchPhase::Began => (delegate.touch_began)(world, touch),
            TouchPhase::Moved => (delegate.touch_moved)(world, touch),
            TouchPhase::Stationary => {}
            TouchPhase::Ended => {
                (delegate.touch_ended)(world, touch);
                remove_game_touch(world, touch);
            }
            TouchPhase::Cancelled => {
                (delegate.touch_cancelled)(world, touch);
                remove_game_touch(world, touch);
            }
        }

        world.resource_mut::<BcView>().touch_events.events[slot].valid = false;
    }
}

/// `[mGameTouchArray removeObject:touch]`.
fn remove_game_touch(world: &mut World, touch: Entity) {
    let mut view = world.resource_mut::<BcView>();
    view.game_touches.retain(|entry| *entry != touch);
}

/// The fingers currently down (`mGameTouchArray`, the `allTouches` argument).
pub fn all_touches(world: &World) -> Vec<Entity> {
    world.resource::<BcView>().game_touches.clone()
}

/// `BCView::clearTouchArrayForModalViewController` @ 0x100176c38.
pub fn clear_touch_array_for_modal_view_controller(world: &mut World) {
    world.resource_mut::<BcView>().game_touches.clear();
}
