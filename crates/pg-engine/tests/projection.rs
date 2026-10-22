//! The projection and root-group transform map game space to the screen as in
//! `docs/re/rendering.md` §1.4.

use bevy_math::{Vec3, Vec4};
use pg_engine::draw::DrawList;
use pg_engine::gl::GlContext;

fn project(screen_width: f32, screen_height: f32, game_x: f32, game_y: f32) -> (f32, f32) {
    let mut gl = GlContext::default();
    let projection = GlContext::view_projection(screen_width, screen_height);
    gl.begin_frame(projection, [0.0; 4]);

    // Root display group: xPos = -160, yPos = 240, zRot = -90.
    gl.translate(-160.0, 240.0, 0.0);
    gl.rotate(0.0, 0.0, -90.0);

    let quad = [[game_x, game_y, -1.0], [game_x + 0.001, game_y, -1.0], [game_x, game_y + 0.001, -1.0]];
    gl.draw_strip(&quad, &[]);
    let list: &DrawList = &gl.list;
    let first = list.batches[0].vertices[0].position;
    let ndc = Vec4::from_array(first);
    (ndc.x / ndc.w, ndc.y / ndc.w)
}

#[test]
fn screen_centre_is_game_240_160() {
    let (x, y) = project(480.0, 320.0, 240.0, 160.0);
    assert!(x.abs() < 1e-5 && y.abs() < 1e-5, "({x}, {y})");
}

#[test]
fn edges_follow_the_logical_width() {
    let (right, _) = project(568.0, 320.0, 240.0 + 284.0, 160.0);
    assert!((right - 1.0).abs() < 1e-4, "{right}");
    let (_, top) = project(568.0, 320.0, 240.0, 320.0);
    assert!((top - 1.0).abs() < 1e-4, "{top}");
    let _ = Vec3::ZERO;
}
