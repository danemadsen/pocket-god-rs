//! `MagnifyingBeamModel` (superclass `BCDisplayObject`).
//!
//! The magnifying glass's light cone (`docs/re/rendering.md` §8.7): an hourglass of two
//! triangles whose waist (the focus) is the only lit part.

use crate::prelude::*;

use pg_engine::display;

/// `MagnifyingBeamModel` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MagnifyingBeamModel {
    /// `mTopWidth`: `f`
    pub top_width: f32,

    /// `mBottomWidth`: `f`
    pub bottom_width: f32,

    /// `mFocusHeight`: `f`
    pub focus_height: f32,

    /// `mModelVertexCount`: `i`
    pub model_vertex_count: i32,

    /// `mModelIndexCount`: `i`
    pub model_index_count: i32,

    /// `mpModelColors`: `^f`
    pub model_colors: Vec<f32>,

    /// `mpModelVertices`: `^f`
    pub model_vertices: Vec<f32>,

    /// `mpModelIndices`: `^s`
    pub model_indices: Vec<i16>,
}

pub const MAGNIFYING_BEAM_MODEL: Class = Class {
    name: "MagnifyingBeamModel",
    superclass: Some(&pg_engine::display::BC_DISPLAY_OBJECT),
    draw: draw_open_gl,
    ..pg_engine::display::BC_DISPLAY_OBJECT
};

/// The model's vertex and index counts (`createModel`).
const MODEL_VERTEX_COUNT: i32 = 6;
const MODEL_INDEX_COUNT: i32 = 6;

/// The colour channels of the two focus vertices (v2, v3), the only coloured ones.
const FOCUS_COLOR_CHANNELS: std::ops::Range<usize> = 8..16;

/// The colour `createModel` starts the focus vertices with.
const FOCUS_COLOR_SEED: f32 = 0.5;

/// The focus vertices' share of the object's alpha, in every channel.
const FOCUS_COLOR_SCALE: f32 = 0.7;

/// `-[MagnifyingBeamModel initWithId:]` @ 0x100034dfc
pub fn init_with_id(world: &mut World, this: Entity, id: &str) {
    world.entity_mut(this).insert(MagnifyingBeamModel::default());

    // `[super initWithId:]`
    let display_object = DisplayObject::new(Some(id));
    world.entity_mut(this).insert(display_object);

    create_model(world, this);
}

/// `-[MagnifyingBeamModel freeModel]` @ 0x100034e60
///
/// The colour array is left alone, as in the original (which never frees it).
pub fn free_model(world: &mut World, this: Entity) {
    let mut model = world.obj_mut::<MagnifyingBeamModel>(this);
    model.model_vertices = Vec::new();
    model.model_indices = Vec::new();
}

/// `-[MagnifyingBeamModel dealloc]` @ 0x100034edc
///
/// For the magnifying glass's `touchEndedInside:allTouches:`, which takes the beam out of the
/// display tree, its last owner.
pub fn dealloc(world: &mut World, this: Entity) {
    free_model(world, this);

    // `[super dealloc]`
    world.despawn(this);
}

/// `-[MagnifyingBeamModel setTopWidth:]` @ 0x100034eac
pub fn set_top_width(world: &mut World, this: Entity, top_width: f32) {
    world.obj_mut::<MagnifyingBeamModel>(this).top_width = top_width;
}

/// `-[MagnifyingBeamModel setBottomWidth:]` @ 0x100034ebc
pub fn set_bottom_width(world: &mut World, this: Entity, bottom_width: f32) {
    world.obj_mut::<MagnifyingBeamModel>(this).bottom_width = bottom_width;
}

/// `-[MagnifyingBeamModel setFocusHeight:]` @ 0x100034ecc
pub fn set_focus_height(world: &mut World, this: Entity, focus_height: f32) {
    world.obj_mut::<MagnifyingBeamModel>(this).focus_height = focus_height;
}

/// `-[MagnifyingBeamModel calcModelVertices]` @ 0x100034f34
///
/// The vertices are absolute (the position is baked in): the top edge at `y`, the focus one
/// focus height below, the bottom edge twice as far.
pub fn calc_model_vertices(world: &mut World, this: Entity) {
    let (pos, alpha) = {
        let object = display::object(world, this);
        (object.pos, object.alpha)
    };

    let mut model = world.obj_mut::<MagnifyingBeamModel>(this);
    let half_top_width = model.top_width * 0.5;
    let half_bottom_width = model.bottom_width * 0.5;
    let focus_y_pos = pos.y - model.focus_height;
    let bottom_y_pos = pos.y + model.focus_height * -2.0;

    let vertices = &mut model.model_vertices;
    vertices[0] = pos.x - half_top_width;
    vertices[1] = pos.y;
    vertices[2] = pos.z;
    vertices[3] = pos.x + half_top_width;
    vertices[4] = pos.y;
    vertices[5] = pos.z;
    vertices[6] = pos.x;
    vertices[7] = focus_y_pos;
    vertices[8] = pos.z;
    vertices[9] = pos.x;
    vertices[10] = focus_y_pos;
    vertices[11] = pos.z;
    vertices[12] = pos.x - half_bottom_width;
    vertices[13] = bottom_y_pos;
    vertices[14] = pos.z;
    vertices[15] = pos.x + half_bottom_width;
    vertices[16] = bottom_y_pos;
    vertices[17] = pos.z;

    let focus_color = alpha * FOCUS_COLOR_SCALE;
    for channel in &mut model.model_colors[FOCUS_COLOR_CHANNELS] {
        *channel = focus_color;
    }
}

/// `-[MagnifyingBeamModel createModel]` @ 0x100035078
pub fn create_model(world: &mut World, this: Entity) {
    free_model(world, this);

    let mut model = world.obj_mut::<MagnifyingBeamModel>(this);
    model.model_vertex_count = MODEL_VERTEX_COUNT;
    model.model_index_count = MODEL_INDEX_COUNT;

    let vertex_count = model.model_vertex_count as usize;
    model.model_vertices = vec![0.0; vertex_count * 3];

    let mut colors = vec![0.0; vertex_count * 4];
    for channel in &mut colors[FOCUS_COLOR_CHANNELS] {
        *channel = FOCUS_COLOR_SEED;
    }
    model.model_colors = colors;

    // A single strip through the six vertices in order.
    model.model_indices = vec![0, 1, 2, 3, 4, 5];
}

/// `-[MagnifyingBeamModel drawOpenGL:]` @ 0x100035168
///
/// An untextured strip with a colour per vertex and no transform of its own; the blending
/// state is whatever the previous draw left (`docs/re/rendering.md` §3.1).
pub fn draw_open_gl(world: &mut World, this: Entity, gl: &mut GlContext) {
    calc_model_vertices(world, this);

    // `glDrawElements` over the vertex and colour arrays.
    let model = world.obj::<MagnifyingBeamModel>(this);
    let used_count = model.model_index_count.max(0) as usize;
    let mut strip_vertices = Vec::with_capacity(used_count);
    let mut strip_colors = Vec::with_capacity(used_count);
    for &index in &model.model_indices[..used_count] {
        let vertex = index as u16 as usize;

        let vertex_base = vertex * 3;
        let position = [
            model.model_vertices[vertex_base],
            model.model_vertices[vertex_base + 1],
            model.model_vertices[vertex_base + 2],
        ];
        strip_vertices.push(position);

        let color_base = vertex * 4;
        let color = [
            model.model_colors[color_base],
            model.model_colors[color_base + 1],
            model.model_colors[color_base + 2],
            model.model_colors[color_base + 3],
        ];
        strip_colors.push(color);
    }

    gl.set_texture_2d(false);
    gl.draw_colored_strip(&strip_vertices, &[], &strip_colors);
    gl.set_texture_2d(true);
}
