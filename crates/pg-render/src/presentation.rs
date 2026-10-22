//! Whether the [`SplashScreen`](crate::SplashScreen) image has reached the screen.
//!
//! Render pipelines compile asynchronously, so the first frames after start-up can go out
//! without the splash although it is up: on a desktop it typically appears in the third frame,
//! and later on slower GPUs. Work that blocks updates while the launch image is meant to show
//! (the game's synchronous `loadGame`) waits for [`SplashPresented`], which the render world
//! sets once a frame that drew the splash was rendered.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::prelude::*;
use bevy::render::render_phase::ViewSortedRenderPhases;
use bevy::render::render_resource::PipelineCache;
use bevy::render::renderer::render_system;
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp, RenderSystems as RenderStages};

use crate::batches::BatchPool;

/// Whether a frame showing the current splash image went to the GPU. Shared by the main and
/// the render world; reset whenever the splash image changes or is taken down.
#[derive(Resource, Clone, Default)]
pub struct SplashPresented(Arc<AtomicBool>);

impl SplashPresented {
    /// Whether the splash image has been rendered.
    pub fn get(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// Records whether the splash image has been rendered (the renderer's job; public for
    /// tests and setups without a render world).
    pub fn set(&self, presented: bool) {
        self.0.store(presented, Ordering::Release);
    }
}

/// The main-world batch entity that draws the splash image this frame (render world).
#[derive(Resource, Default)]
struct ExtractedSplashSlot(Option<Entity>);

pub(crate) fn build(app: &mut App) {
    let presented = SplashPresented::default();
    app.insert_resource(presented.clone());

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app.insert_resource(presented);
    render_app.init_resource::<ExtractedSplashSlot>();
    render_app.add_systems(ExtractSchedule, extract_splash_slot);

    // Once the frame was rendered with the pipelines that were ready for it.
    let detect = detect_splash_presented.in_set(RenderStages::Render).after(render_system);
    render_app.add_systems(Render, detect);
}

fn extract_splash_slot(pool: Extract<Res<BatchPool>>, mut slot: ResMut<ExtractedSplashSlot>) {
    slot.0 = pool.splash_slot;
}

/// Sets [`SplashPresented`] when this frame's 2D phase drew the splash batch: it was queued and
/// its pipeline was ready.
fn detect_splash_presented(
    slot: Res<ExtractedSplashSlot>,
    phases: Res<ViewSortedRenderPhases<Transparent2d>>,
    pipeline_cache: Res<PipelineCache>,
    presented: Res<SplashPresented>,
) {
    let Some(splash_entity) = slot.0 else {
        return;
    };
    if presented.get() {
        return;
    }

    for phase in phases.values() {
        for ((_, main_entity), item) in &phase.items {
            if main_entity.id() != splash_entity {
                continue;
            }
            let pipeline = pipeline_cache.get_render_pipeline(item.pipeline);
            if pipeline.is_some() {
                presented.set(true);
                return;
            }
        }
    }
}
