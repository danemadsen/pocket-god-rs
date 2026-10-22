//! `LightningModel` (superclass `BCDisplayObject`).

use crate::prelude::*;

use pg_engine::display;

/// `LightningModel` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct LightningModel {
    /// `mpPathList`: `^{list<VECTOR4, std::__1::allocator<VECTOR4> >={__list_node_base<VECTOR4, void *>=^{__list_node<VECTOR4, void *>}^{__list_node<VECTOR4, void *>}}{__compressed_pair<unsigned long, std::__1::allocator<std::__1::__list_node<VECTOR4, void *> > >=Q}}`
    pub path_list: Vec<Vec4>,

    /// `mModelVertexCount`: `i`
    pub model_vertex_count: i32,

    /// `mModelIndexCount`: `i`
    pub model_index_count: i32,

    /// `mpInnerModelVertices`: `^f`
    pub inner_model_vertices: Vec<f32>,

    /// `mpMiddleModelVertices`: `^f`
    pub middle_model_vertices: Vec<f32>,

    /// `mpOuterModelVertices`: `^f`
    pub outer_model_vertices: Vec<f32>,

    /// `mpModelIndices`: `^s`
    pub model_indices: Vec<i16>,
}

pub const LIGHTNING_MODEL: Class = Class {
    name: "LightningModel",
    superclass: Some(&pg_engine::display::BC_DISPLAY_OBJECT),
    draw: draw_open_gl,
    ..pg_engine::display::BC_DISPLAY_OBJECT
};

/// Half-width of the bright core strip.
const INNER_LINE_WIDTH: f32 = 2.0;

/// Half-width of the middle glow strip.
const MIDDLE_LINE_WIDTH: f32 = 6.0;

/// Half-width of the outer glow strip.
const OUTER_LINE_WIDTH: f32 = 9.0;

/// The glow strips' share of the model's alpha.
const GLOW_ALPHA_SCALE: f32 = 0.2;

/// One vertex pair of the path strip that `LightningModel`, `RainbowModel` and `PathModel`
/// build along a point list (`docs/re/rendering.md` §8.1).
pub(crate) struct PathStripPair {
    /// `A`: the vertex on the right-hand side of the path.
    pub right: Vec2,

    /// `B`: the vertex on the left-hand side of the path.
    pub left: Vec2,

    /// The angle the generator writes back into the point's `w`.
    pub angle: f32,
}

/// `(x, y)` scaled by `1 / sqrtf(x² + y²)`, as the strip generator normalises.
fn normalize_xy(x: f32, y: f32) -> (f32, f32) {
    let inverse_length = 1.0 / (x * x + y * y).sqrt();
    (x * inverse_length, y * inverse_length)
}

/// The vertex pair of point `index` of the shared path-strip generator (rendering.md §8.1):
/// `line_width` either side of the path, perpendicular to the segment towards the next point
/// (the last point uses the segment from its predecessor); inner joints use the angle bisector
/// with the previous segment when both of its components are non-zero. Needs two points.
pub(crate) fn path_strip_pair(points: &[Vec4], index: usize, line_width: f32) -> PathStripPair {
    let point = points[index];

    if index == points.len() - 1 {
        let previous = points[index - 1];
        let (direction_x, direction_y) = normalize_xy(point.x - previous.x, point.y - previous.y);
        let angle = direction_y.atan2(direction_x);
        let sin_width = angle.sin() * line_width;
        let cos_width = angle.cos() * line_width;
        let right = Vec2::new(point.x + sin_width, point.y - cos_width);
        let left = Vec2::new(point.x - sin_width, point.y + cos_width);
        return PathStripPair { right, left, angle };
    }

    let next = points[index + 1];
    let (direction_x, direction_y) = normalize_xy(next.x - point.x, next.y - point.y);
    let mut angle = direction_y.atan2(direction_x);
    let sin_width = angle.sin() * line_width;
    let cos_width = angle.cos() * line_width;
    let mut right = Vec2::new(point.x + sin_width, point.y - cos_width);
    let mut left = Vec2::new(point.x - sin_width, point.y + cos_width);

    if index > 0 {
        let previous = points[index - 1];
        let (back_x, back_y) = normalize_xy(previous.x - point.x, previous.y - point.y);
        let bisector_x = direction_x + back_x;
        if bisector_x != 0.0 {
            let bisector_y = direction_y + back_y;
            if bisector_y != 0.0 {
                angle = bisector_y.atan2(bisector_x);
                let cos_offset = angle.cos() * line_width;
                let sin_offset = angle.sin() * line_width;
                let turn = direction_x * bisector_y - direction_y * bisector_x;
                if turn <= 0.0 {
                    right = Vec2::new(point.x + cos_offset, point.y + sin_offset);
                    left = Vec2::new(point.x - cos_offset, point.y - sin_offset);
                } else {
                    right = Vec2::new(point.x - cos_offset, point.y - sin_offset);
                    left = Vec2::new(point.x + cos_offset, point.y + sin_offset);
                }
            }
        }
    }

    PathStripPair { right, left, angle }
}

/// The positions `glDrawElements(GL_TRIANGLE_STRIP, count, GL_UNSIGNED_SHORT, indices)` reads
/// from a `glVertexPointer(3, GL_FLOAT, 0, vertices)` array, in strip order.
pub(crate) fn strip_vertices(vertices: &[f32], indices: &[i16], count: i32) -> Vec<[f32; 3]> {
    let used_count = count.max(0) as usize;
    let mut strip = Vec::with_capacity(used_count);
    for &index in &indices[..used_count] {
        let base = index as u16 as usize * 3;
        strip.push([vertices[base], vertices[base + 1], vertices[base + 2]]);
    }
    strip
}

/// `-[LightningModel initWithId:pathList:]` @ 0x1000bf768
///
/// `path_list` is the model's own copy of the drawn points (`FUN_1000bfe3c` copies the
/// caller's list).
pub fn init_with_id(world: &mut World, this: Entity, id: &str, path_list: Vec<Vec4>) {
    world.entity_mut(this).insert(LightningModel::default());

    // `[super initWithId:]`
    let display_object = DisplayObject::new(Some(id));
    world.entity_mut(this).insert(display_object);

    world.obj_mut::<LightningModel>(this).path_list = path_list;
    create_model(world, this);
}

/// `-[LightningModel pathList]` @ 0x1000bf80c
///
/// The original hands out the list itself; its readers only read it, so the port returns the
/// points (with the segment angles `createModel` wrote into `w`).
pub fn path_list(world: &mut World, this: Entity) -> Vec<Vec4> {
    world.obj::<LightningModel>(this).path_list.clone()
}

/// `-[LightningModel freeModel]` @ 0x1000bf81c
pub fn free_model(world: &mut World, this: Entity) {
    let mut model = world.obj_mut::<LightningModel>(this);
    model.inner_model_vertices = Vec::new();
    model.middle_model_vertices = Vec::new();
    model.outer_model_vertices = Vec::new();
    model.model_indices = Vec::new();
}

/// `-[LightningModel calcModelVertices:modelIndices:lineWidth:]` @ 0x1000bf914
///
/// Fills one strip of two vertices per point and writes each point's pair angle into its `w`.
pub fn calc_model_vertices(world: &mut World, this: Entity, vertices: &mut [f32], model_indices: &mut [i16], line_width: f32) {
    let mut points = {
        let mut model = world.obj_mut::<LightningModel>(this);
        std::mem::take(&mut model.path_list)
    };

    // PORT: with an empty list the original walks the list's sentinel node (undefined
    // behaviour); `createModel` already leaves a one-point list empty. Nothing is built here.
    if points.len() >= 2 {
        for index in 0..points.len() {
            let pair = path_strip_pair(&points, index, line_width);
            points[index].w = pair.angle;

            let vertex_base = index * 6;
            vertices[vertex_base] = pair.right.x;
            vertices[vertex_base + 1] = pair.right.y;
            vertices[vertex_base + 2] = 0.0;
            vertices[vertex_base + 3] = pair.left.x;
            vertices[vertex_base + 4] = pair.left.y;
            vertices[vertex_base + 5] = 0.0;

            let index_base = index * 2;
            model_indices[index_base] = index_base as i16;
            model_indices[index_base + 1] = (index_base + 1) as i16;
        }
    }

    world.obj_mut::<LightningModel>(this).path_list = points;
}

/// `-[LightningModel createModel]` @ 0x1000bfbb0
///
/// Three strips over the same points: inner (half-width 2), middle (6) and outer (9).
pub fn create_model(world: &mut World, this: Entity) {
    free_model(world, this);

    let point_count = world.obj::<LightningModel>(this).path_list.len();
    if point_count == 1 {
        let mut model = world.obj_mut::<LightningModel>(this);
        model.model_vertex_count = 0;
        model.model_index_count = 0;
        return;
    }

    let model_vertex_count = (point_count as i32) << 1;
    let model_index_count = ((point_count as u32) << 1) as i32;
    {
        let mut model = world.obj_mut::<LightningModel>(this);
        model.model_vertex_count = model_vertex_count;
        model.model_index_count = model_index_count;
    }

    let vertex_float_count = model_vertex_count as usize * 3;
    let mut model_indices = vec![0; model_index_count as usize];

    let mut inner_model_vertices = vec![0.0; vertex_float_count];
    calc_model_vertices(world, this, &mut inner_model_vertices, &mut model_indices, INNER_LINE_WIDTH);

    let mut middle_model_vertices = vec![0.0; vertex_float_count];
    calc_model_vertices(world, this, &mut middle_model_vertices, &mut model_indices, MIDDLE_LINE_WIDTH);

    let mut outer_model_vertices = vec![0.0; vertex_float_count];
    calc_model_vertices(world, this, &mut outer_model_vertices, &mut model_indices, OUTER_LINE_WIDTH);

    let mut model = world.obj_mut::<LightningModel>(this);
    model.model_indices = model_indices;
    model.inner_model_vertices = inner_model_vertices;
    model.middle_model_vertices = middle_model_vertices;
    model.outer_model_vertices = outer_model_vertices;
}

/// `-[LightningModel drawOpenGL:]` @ 0x1000bfd14
///
/// Untextured; the blending state is whatever the previous draw left.
pub fn draw_open_gl(world: &mut World, this: Entity, gl: &mut GlContext) {
    let display = display::object(world, this);
    let pos = display.pos;
    let alpha = display.alpha;

    let model = world.obj::<LightningModel>(this);
    let index_count = model.model_index_count;
    let outer_strip = strip_vertices(&model.outer_model_vertices, &model.model_indices, index_count);
    let middle_strip = strip_vertices(&model.middle_model_vertices, &model.model_indices, index_count);
    let inner_strip = strip_vertices(&model.inner_model_vertices, &model.model_indices, index_count);

    gl.push_matrix();
    gl.translate(pos.x, pos.y, pos.z);
    gl.set_texture_2d(false);

    let glow_alpha = alpha * GLOW_ALPHA_SCALE;
    gl.set_color(glow_alpha, glow_alpha, glow_alpha, glow_alpha);
    gl.draw_strip(&outer_strip, &[]);

    gl.set_color(glow_alpha, glow_alpha, glow_alpha, glow_alpha);
    gl.draw_strip(&middle_strip, &[]);

    gl.set_color(alpha, alpha, alpha, alpha);
    gl.draw_strip(&inner_strip, &[]);

    gl.set_texture_2d(true);
    gl.pop_matrix();
}
