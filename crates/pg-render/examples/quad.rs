//! Smoke test for the renderer without the engine: draws a background quad, a textured atlas
//! quad (an embedded atlas page) and an additive quad. `PG_SCREENSHOT=<path>` saves the window
//! after a few frames and exits.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use pg_engine::draw::{BlendMode, DrawList, DrawVertex, TextureId};
use pg_render::{FrameDrawList, LogicalScreen, PgRenderPlugin, TexturePages, decode_texture_image, page_image};

const ATLAS_IMAGE: &str = "ApeIslandTextureAtlas0";
const ATLAS_PAGE: TextureId = TextureId(1);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(PgRenderPlugin);
    app.insert_resource(LogicalScreen::default());
    app.add_systems(Startup, load_atlas);
    app.add_systems(Update, build_frame);
    app.add_systems(Update, auto_screenshot);
    app.run();
}

fn load_atlas(mut images: ResMut<Assets<Image>>, mut pages: ResMut<TexturePages>) {
    let decoded = decode_texture_image(ATLAS_IMAGE).expect("the atlas is embedded");
    let image = page_image(decoded.pixels, decoded.width, decoded.height, false);
    let handle = images.add(image);
    pages.insert(ATLAS_PAGE, handle);
}

fn vertex(x: f32, y: f32, u: f32, v: f32, color: [f32; 4]) -> DrawVertex {
    DrawVertex {
        position: [x, y, 0.0, 1.0],
        uv: [u, v],
        color,
    }
}

fn build_frame(time: Res<Time>, mut frame: ResMut<FrameDrawList>) {
    let mut list = DrawList::default();
    let sky = [0.4, 0.7, 1.0, 1.0];
    let white = [1.0, 1.0, 1.0, 1.0];

    let background = [
        vertex(-1.0, 1.0, 0.0, 0.0, sky),
        vertex(1.0, 1.0, 1.0, 0.0, sky),
        vertex(-1.0, -1.0, 0.0, 1.0, sky),
        vertex(1.0, -1.0, 1.0, 1.0, sky),
    ];
    list.push_quad(None, BlendMode::Alpha, background);

    let wobble = time.elapsed_secs().sin() * 0.1;
    let atlas = [
        vertex(-0.6 + wobble, 0.9, 0.0, 0.0, white),
        vertex(0.6 + wobble, 0.9, 1.0, 0.0, white),
        vertex(-0.6, -0.9, 0.0, 1.0, white),
        vertex(0.6, -0.9, 1.0, 1.0, white),
    ];
    list.push_quad(Some(ATLAS_PAGE), BlendMode::Premultiplied, atlas);

    let glow = [1.0, 0.5, 0.1, 0.5];
    let additive = [
        vertex(0.5, 0.2, 0.0, 0.0, glow),
        vertex(0.9, 0.2, 0.0, 0.0, glow),
        vertex(0.5, -0.2, 0.0, 0.0, glow),
        vertex(0.9, -0.2, 0.0, 0.0, glow),
    ];
    list.push_quad(None, BlendMode::Additive, additive);

    frame.0 = list;
}

/// With `PG_SCREENSHOT=<path>`, captures the window after a few frames and exits.
fn auto_screenshot(mut commands: Commands, mut frames: Local<u32>, mut exit: MessageWriter<AppExit>) {
    let Ok(path) = std::env::var("PG_SCREENSHOT") else {
        return;
    };

    *frames += 1;
    if *frames == 30 {
        let mut screenshot = commands.spawn(Screenshot::primary_window());
        screenshot.observe(save_to_disk(path));
    }
    if *frames == 45 {
        exit.write(AppExit::Success);
    }
}
