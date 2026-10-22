//! `BCRect`: an untextured rectangle (`docs/re/rendering.md` §5.8).

use bevy_ecs::prelude::*;

use crate::display::{self, BC_DISPLAY_OBJECT, model};
use crate::gl::GlContext;
use crate::object::{Class, ObjectWorldExt};

/// `BCRect` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct Rect {
    pub min_x_pos: f32,
    pub min_y_pos: f32,
    pub max_x_pos: f32,
    pub max_y_pos: f32,

    /// `mpModelVertices` (4 × xyz), recomputed by every bound setter.
    pub vertices: [f32; 12],
}

impl Rect {
    /// `-width` @ 0x100021b90.
    pub fn width(&self) -> f32 {
        self.max_x_pos - self.min_x_pos
    }

    /// `-height` @ 0x100021bb0.
    pub fn height(&self) -> f32 {
        self.max_y_pos - self.min_y_pos
    }
}

pub const BC_RECT: Class = Class {
    name: "BCRect",
    superclass: Some(&BC_DISPLAY_OBJECT),
    draw: draw_rect,
    ..BC_DISPLAY_OBJECT
};

/// `-[BCRect initWithId:minXPos:minYPos:maxXPos:maxYPos:]` @ 0x1000219e0.
pub fn spawn(world: &mut World, id: Option<&str>, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Entity {
    let entity = display::spawn(world, &BC_RECT, id);
    let mut rect = Rect {
        min_x_pos: min_x,
        min_y_pos: min_y,
        max_x_pos: max_x,
        max_y_pos: max_y,
        vertices: [0.0; 12],
    };
    calc_model_vertices(&mut rect);
    world.entity_mut(entity).insert(rect);
    entity
}

/// `-calcModelVertices` @ 0x100021c28.
pub fn calc_model_vertices(rect: &mut Rect) {
    rect.vertices = [
        rect.min_x_pos, rect.max_y_pos, 0.0,
        rect.max_x_pos, rect.max_y_pos, 0.0,
        rect.min_x_pos, rect.min_y_pos, 0.0,
        rect.max_x_pos, rect.min_y_pos, 0.0,
    ];
}

/// `-setMinXPos:` @ 0x100021af0.
pub fn set_min_x_pos(world: &mut World, entity: Entity, value: f32) {
    let mut rect = world.obj_mut::<Rect>(entity);
    rect.min_x_pos = value;
    calc_model_vertices(&mut rect);
}

/// `-setMinYPos:` @ 0x100021b1c.
pub fn set_min_y_pos(world: &mut World, entity: Entity, value: f32) {
    let mut rect = world.obj_mut::<Rect>(entity);
    rect.min_y_pos = value;
    calc_model_vertices(&mut rect);
}

/// `-setMaxXPos:` @ 0x100021b48.
pub fn set_max_x_pos(world: &mut World, entity: Entity, value: f32) {
    let mut rect = world.obj_mut::<Rect>(entity);
    rect.max_x_pos = value;
    calc_model_vertices(&mut rect);
}

/// `-setMaxYPos:` @ 0x100021b74.
pub fn set_max_y_pos(world: &mut World, entity: Entity, value: f32) {
    let mut rect = world.obj_mut::<Rect>(entity);
    rect.max_y_pos = value;
    calc_model_vertices(&mut rect);
}

/// `BCRect::drawOpenGL:` @ 0x100021d58 (no hidden check).
fn draw_rect(world: &mut World, entity: Entity, gl: &mut GlContext) {
    let display = display::object(world, entity);
    let translucent = display.alpha < 1.0;
    gl.set_blend(translucent);
    model::set_premultiplied_color(display, gl);

    gl.push_matrix();
    model::apply_sprite_transform(display, gl);
    let vertices = world.obj::<Rect>(entity).vertices;
    let corners = model::quad_corners(&vertices);
    gl.set_texture_2d(false);
    gl.draw_strip(&corners, &[]);
    gl.set_texture_2d(true);
    gl.pop_matrix();
}
