//! Calls from the engine into the game, installed by the game at start-up.
//!
//! The engine never names game classes. Where a BoltLib class calls game code (the view
//! delegate, `[[IdolManager instance] triggerEpisodeEvent:]`, preference storage), it goes
//! through these function pointers.

use bevy_ecs::prelude::*;

/// Game callbacks; defaults do nothing.
#[derive(Resource, Clone, Copy)]
pub struct EngineHooks {
    /// `BCViewDelegate -touchBegan:allTouches:` (the touch entity).
    pub touch_began: fn(&mut World, Entity),

    /// `BCViewDelegate -touchMoved:allTouches:`.
    pub touch_moved: fn(&mut World, Entity),

    /// `BCViewDelegate -touchEnded:allTouches:`.
    pub touch_ended: fn(&mut World, Entity),

    /// `BCViewDelegate -touchCancelled:allTouches:`.
    pub touch_cancelled: fn(&mut World, Entity),

    /// `BCViewDelegate -preStartTouchBeganXPos:yPos:` (touch input not yet enabled).
    pub pre_start_touch_began: fn(&mut World, f32, f32),

    /// `[[IdolManager instance] triggerEpisodeEvent:]`.
    pub trigger_episode_event: fn(&mut World, &str),
}

fn touch_noop(_world: &mut World, _touch: Entity) {}

fn pre_start_noop(_world: &mut World, _x: f32, _y: f32) {}

fn episode_event_noop(_world: &mut World, _event: &str) {}

impl Default for EngineHooks {
    fn default() -> Self {
        Self {
            touch_began: touch_noop,
            touch_moved: touch_noop,
            touch_ended: touch_noop,
            touch_cancelled: touch_noop,
            pre_start_touch_began: pre_start_noop,
            trigger_episode_event: episode_event_noop,
        }
    }
}

pub fn hooks(world: &World) -> EngineHooks {
    *world.resource::<EngineHooks>()
}
