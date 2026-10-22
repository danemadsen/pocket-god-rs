//! egui, drawn by its own camera over the game.
//!
//! pg-render's game camera blends in gamma space (`CompositingSpace::Srgb`: an `Rgba8Unorm`
//! main texture, or `Rgba16Float` on WebGL2), but bevy_egui 0.43 always builds its pipeline
//! for `Rgba8UnormSrgb` and fails validation when it draws into such a texture. egui
//! therefore gets a camera of its own that draws nothing else and is composited over the game.
//!
//! That camera renders only while egui shows something (the game's only egui piece is the
//! text field of `pg-ui`): with a second camera compositing onto the window every frame,
//! Android presented a fully black frame every second or so.

use bevy::camera::CameraOutputMode;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext};
use pg_engine::platform::TextField;

/// Above pg-render's game camera (order 0).
const EGUI_CAMERA_ORDER: isize = 1;

pub struct EguiOverlayPlugin;

impl Plugin for EguiOverlayPlugin {
    fn build(&self, app: &mut App) {
        // The primary context goes on the overlay camera, never on the game camera.
        let settings = EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        };
        app.insert_resource(settings);

        app.add_plugins(EguiPlugin::default());
        app.add_systems(Startup, spawn_egui_camera);
        app.add_systems(PostUpdate, activate_while_shown);
    }
}

/// Renders the overlay only while the text field is open.
fn activate_while_shown(field: Res<TextField>, mut cameras: Query<&mut Camera, With<PrimaryEguiContext>>) {
    let shown = field.first_responder;
    for mut camera in &mut cameras {
        if camera.is_active != shown {
            camera.is_active = shown;
        }
    }
}

fn spawn_egui_camera(mut commands: Commands) {
    // egui leaves premultiplied colour in the cleared texture; blend that over the game.
    let output_mode = CameraOutputMode::Write {
        blend_state: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        clear_color: ClearColorConfig::None,
    };

    let camera = Camera {
        order: EGUI_CAMERA_ORDER,
        output_mode,
        clear_color: ClearColorConfig::Custom(Color::NONE),
        is_active: false,
        ..default()
    };

    // No render layers: the game's meshes stay out of this camera. Keep the default `Msaa`:
    // Bevy 0.20 shares one 2D depth texture between all cameras on a window, so this camera
    // must use the same sample count as pg-render's game camera (pg-render also keeps the two
    // cameras' target sizes in step when the window is resized).
    commands.spawn((Camera2d, camera, RenderLayers::none(), PrimaryEguiContext));
}
