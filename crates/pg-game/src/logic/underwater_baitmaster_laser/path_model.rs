//! `PathModel` (superclass `BCDisplayObject`).
//!
//! The ribbon drawn along a piranha's finger-drawn path (`docs/re/rendering.md` §8.1, §8.6).
//! The original shares the piranha's `std::vector<VECTOR4>` by pointer; the port keeps a copy
//! that the piranha refreshes through `setPathVector:` wherever the original rebuilds the
//! model, which is the only time the shared vector is read here.

use crate::prelude::*;

use pg_engine::display;

/// `PathModel` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct PathModel {
    /// `mpPathVector`: `^{vector<VECTOR4, std::__1::allocator<VECTOR4> >=^{?}^{?}{__compressed_pair<VECTOR4 *, std::__1::allocator<VECTOR4> >=^{?}}}`
    pub path_vector: Vec<Vec4>,

    /// `mModelVertexCount`: `i`
    pub model_vertex_count: i32,

    /// `mModelIndexCount`: `i`
    pub model_index_count: i32,

    /// `mpModelVertices`: `^f`
    pub model_vertices: Vec<f32>,

    /// `mpModelIndices`: `^s`
    pub model_indices: Vec<i16>,
}

pub const PATH_MODEL: Class = Class {
    name: "PathModel",
    superclass: Some(&pg_engine::display::BC_DISPLAY_OBJECT),
    draw: draw_open_gl,
    ..pg_engine::display::BC_DISPLAY_OBJECT
};

/// The ribbon's half-width (`createModel` passes 1.5 as `lineWidth:`).
const LINE_WIDTH: f32 = 1.5;

/// `-[PathModel initWithId:pathVector:]` @ 0x100124fa4
pub fn init_with_id(world: &mut World, this: Entity, id: &str, path_vector: Vec<Vec4>) {
    world.entity_mut(this).insert(PathModel::default());

    // `[super initWithId:]`.
    let display_object = DisplayObject::new(Some(id));
    world.entity_mut(this).insert(display_object);

    world.obj_mut::<PathModel>(this).path_vector = path_vector;
    create_model(world, this);
}

/// `-[PathModel setPathVector:]` @ 0x100125018
pub fn set_path_vector(world: &mut World, this: Entity, path_vector: Vec<Vec4>) {
    world.obj_mut::<PathModel>(this).path_vector = path_vector;
    create_model(world, this);
}

/// `-[PathModel pathVector]` @ 0x100125034
pub fn path_vector(world: &mut World, this: Entity) -> Vec<Vec4> {
    world.obj::<PathModel>(this).path_vector.clone()
}

/// `-[PathModel freeModel]` @ 0x100125044
///
/// The counts are left as they are, as in the original.
pub fn free_model(world: &mut World, this: Entity) {
    let mut model = world.obj_mut::<PathModel>(this);
    model.model_vertices = Vec::new();
    model.model_indices = Vec::new();
}

/// `-[PathModel calcModelVertices:modelIndices:lineWidth:]` @ 0x1001250e8
///
/// The original writes through the two buffer pointers, which `createModel` has just stored in
/// `mpModelVertices` / `mpModelIndices`; the port fills the buffers it is handed and stores them
/// there. It also writes each point's heading into the point's `w`, as the original does to
/// the shared path. Needs at least two points (`createModel` checks).
pub fn calc_model_vertices(world: &mut World, this: Entity, calc_model_vertices: Vec<f32>, model_indices: Vec<i16>, line_width: f32) {
    let mut vertices = calc_model_vertices;
    let mut indices = model_indices;
    let mut path = std::mem::take(&mut world.obj_mut::<PathModel>(this).path_vector);
    let point_count = path.len();

    let mut vertex_index = 0usize;
    let mut index_index = 0usize;
    let mut previous: Option<usize> = None;
    let mut current = 0usize;
    loop {
        let point = path[current];
        let next = path[current + 1];

        let delta_x = next.x - point.x;
        let delta_y = next.y - point.y;
        let inverse_length = 1.0 / (delta_x * delta_x + delta_y * delta_y).sqrt();
        let direction_x = delta_x * inverse_length;
        let direction_y = delta_y * inverse_length;

        let mut angle = direction_y.atan2(direction_x);
        let (sin, cos) = angle.sin_cos();
        let sin_offset = sin * line_width;
        let cos_offset = cos * line_width;
        let mut right_x = point.x + sin_offset;
        let mut right_y = point.y - cos_offset;
        let mut left_x = point.x - sin_offset;
        let mut left_y = point.y + cos_offset;

        // Joints use the bisector of the two segments, with the same half-width.
        if let Some(previous) = previous {
            let back = path[previous];
            let back_x = back.x - point.x;
            let back_y = back.y - point.y;
            let inverse_back_length = 1.0 / (back_x * back_x + back_y * back_y).sqrt();
            let bisector_x = direction_x + back_x * inverse_back_length;
            if bisector_x != 0.0 {
                let bisector_y = direction_y + back_y * inverse_back_length;
                if bisector_y != 0.0 {
                    angle = bisector_y.atan2(bisector_x);
                    let (sin, cos) = angle.sin_cos();
                    let cos_offset = cos * line_width;
                    let sin_offset = sin * line_width;
                    let cross = direction_x * bisector_y - direction_y * bisector_x;
                    if cross <= 0.0 {
                        right_x = point.x + cos_offset;
                        right_y = point.y + sin_offset;
                        left_x = point.x - cos_offset;
                        left_y = point.y - sin_offset;
                    } else {
                        right_x = point.x - cos_offset;
                        right_y = point.y - sin_offset;
                        left_x = point.x + cos_offset;
                        left_y = point.y + sin_offset;
                    }
                }
            }
        }

        path[current].w = angle;
        vertices[vertex_index] = right_x;
        vertices[vertex_index + 1] = right_y;
        vertices[vertex_index + 2] = 0.0;
        indices[index_index] = index_index as i16;
        vertices[vertex_index + 3] = left_x;
        vertices[vertex_index + 4] = left_y;
        vertices[vertex_index + 5] = 0.0;
        indices[index_index + 1] = (index_index + 1) as i16;
        index_index += 2;
        vertex_index += 6;

        let reached_last_segment = current + 2 == point_count;
        if reached_last_segment {
            break;
        }
        previous = Some(current);
        current += 1;
    }

    // The last point takes the heading of the last segment.
    let last = path[current + 1];
    let before_last = path[current];
    let delta_x = last.x - before_last.x;
    let delta_y = last.y - before_last.y;
    let inverse_length = 1.0 / (delta_x * delta_x + delta_y * delta_y).sqrt();
    let direction_x = delta_x * inverse_length;
    let direction_y = delta_y * inverse_length;
    let angle = direction_y.atan2(direction_x);
    let (sin, cos) = angle.sin_cos();
    let cos_offset = cos * line_width;
    let sin_offset = sin * line_width;

    path[current + 1].w = angle;
    vertices[vertex_index] = last.x + sin_offset;
    vertices[vertex_index + 1] = last.y - cos_offset;
    vertices[vertex_index + 2] = 0.0;
    indices[index_index] = index_index as i16;
    vertices[vertex_index + 3] = last.x - sin_offset;
    vertices[vertex_index + 4] = last.y + cos_offset;
    vertices[vertex_index + 5] = 0.0;
    indices[index_index + 1] = (index_index + 1) as i16;

    let mut model = world.obj_mut::<PathModel>(this);
    model.path_vector = path;
    model.model_vertices = vertices;
    model.model_indices = indices;
}

/// `-[PathModel createModel]` @ 0x100125374
pub fn create_model(world: &mut World, this: Entity) {
    free_model(world, this);

    let point_count = world.obj::<PathModel>(this).path_vector.len();
    if point_count < 2 {
        let mut model = world.obj_mut::<PathModel>(this);
        model.model_vertex_count = 0;
        model.model_index_count = 0;
        return;
    }

    // Two vertices (a left and a right edge) per point, drawn as one strip.
    let vertex_count = (point_count * 2) as i32;
    let index_count = vertex_count & !1;
    {
        let mut model = world.obj_mut::<PathModel>(this);
        model.model_vertex_count = vertex_count;
        model.model_index_count = index_count;
    }

    let indices = vec![0i16; index_count as usize];
    let vertices = vec![0.0f32; vertex_count as usize * 3];
    calc_model_vertices(world, this, vertices, indices, LINE_WIDTH);
}

/// `-[PathModel drawOpenGL:]` @ 0x100125470
///
/// Untextured, in `alpha / 2` grey, at the object's position only; blending is whatever the
/// previous draw left (`docs/re/rendering.md` §3.1).
pub fn draw_open_gl(world: &mut World, this: Entity, gl: &mut GlContext) {
    let (pos, alpha) = {
        let object = display::object(world, this);
        (object.pos, object.alpha)
    };

    let model = world.obj::<PathModel>(this);
    let index_count = model.model_index_count.max(0) as usize;
    let mut strip: Vec<[f32; 3]> = Vec::with_capacity(index_count);
    for index in model.model_indices.iter().take(index_count) {
        let vertex = *index as u16 as usize * 3;
        let position = [model.model_vertices[vertex], model.model_vertices[vertex + 1], model.model_vertices[vertex + 2]];
        strip.push(position);
    }

    gl.push_matrix();
    gl.translate(pos.x, pos.y, pos.z);
    gl.set_texture_2d(false);

    let grey = alpha * 0.5;
    gl.set_color(grey, grey, grey, grey);
    gl.draw_strip(&strip, &[]);

    gl.set_texture_2d(true);
    gl.pop_matrix();
}
