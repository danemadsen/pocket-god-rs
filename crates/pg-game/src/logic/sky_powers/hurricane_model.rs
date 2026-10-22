//! `HurricaneModel` (superclass `BCDisplayObject`).

use crate::prelude::*;

use crate::logic::sky_powers::lightning_model::strip_vertices;
use pg_engine::display;

/// `HurricaneModel` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct HurricaneModel {
    /// `mHurricaneWindPosCount`: `i`
    pub hurricane_wind_pos_count: i32,

    /// `mHurricaneWindPosArray`: `[5{HurricaneWindPos="mPos"{?="x"f"y"f"z"f"w"f}"mPosVel"{?="x"f"y"f"z"f"w"f}"mWindRot"f"mWindRotVel"f"mRadius"f"mRadiusVel"f"mDrawPos"f"mDrawPosVel"f"mLineWidth"f"mSegmentCount"i}]`
    pub hurricane_wind_pos_array: [HurricaneWindPos; 5],

    /// `mModelVertexCount`: `i`
    pub model_vertex_count: i32,

    /// `mModelIndexCount`: `i`
    pub model_index_count: i32,

    /// `mpModelVertices`: `^f`
    pub model_vertices: Vec<f32>,

    /// `mpModelIndices`: `^s`
    pub model_indices: Vec<i16>,
}

pub const HURRICANE_MODEL: Class = Class {
    name: "HurricaneModel",
    superclass: Some(&pg_engine::display::BC_DISPLAY_OBJECT),
    draw: draw_open_gl,
    ..pg_engine::display::BC_DISPLAY_OBJECT
};

/// Indices `createModel` allocates (`malloc(0x50)`): 20 segments of two vertices.
const MODEL_INDEX_CAPACITY: usize = 40;

/// Vertex floats `createModel` allocates (`malloc(0x1e0)`): 40 vertices of three floats.
const MODEL_VERTEX_FLOAT_CAPACITY: usize = 120;

/// How far the lower vertex of each pair sits below the upper one.
const RIBBON_HEIGHT: f32 = 2.0;

/// `-[HurricaneModel initWithId:]` @ 0x10018b768
pub fn init_with_id(world: &mut World, this: Entity, id: &str) {
    // The zeroed component also stands for the `bzero` of the wind records.
    world.entity_mut(this).insert(HurricaneModel::default());

    // `[super initWithId:]`
    let display_object = DisplayObject::new(Some(id));
    world.entity_mut(this).insert(display_object);

    create_model(world, this);
}

/// `-[HurricaneModel freeModel]` @ 0x10018b7e0
pub fn free_model(world: &mut World, this: Entity) {
    let mut model = world.obj_mut::<HurricaneModel>(this);
    model.model_vertices = Vec::new();
    model.model_indices = Vec::new();
}

/// `-[HurricaneModel calcModelVertices:modelIndices:hurricaneWindPos:]` @ 0x10018b884
///
/// One wind ribbon (rendering.md §8.5): `mSegmentCount` vertex pairs around a horizontal
/// circle of growing radius, rising by `mPosVel.y` per segment, starting `mDrawPos` segments
/// along. Also sets the model's vertex and index counts.
pub fn calc_model_vertices(world: &mut World, this: Entity, vertices: &mut [f32], model_indices: &mut [i16], hurricane_wind_pos: &HurricaneWindPos) {
    let wind_pos = hurricane_wind_pos;
    let segment_count = wind_pos.segment_count;

    if segment_count > 0 {
        let pos_vel_y = wind_pos.pos_vel.y;
        let radius_vel = wind_pos.radius_vel;
        let draw_pos = wind_pos.draw_pos;
        let wind_rot_vel = wind_pos.wind_rot_vel;
        let mut angle = wind_pos.wind_rot + draw_pos * wind_rot_vel;
        let mut radius = wind_pos.radius + draw_pos * radius_vel;
        let mut y_pos = wind_pos.pos.y + pos_vel_y * draw_pos;

        for segment in 0..segment_count as usize {
            let x_offset = radius * angle.cos();
            let z_offset = radius * angle.sin();
            radius = radius_vel + radius;
            angle = wind_rot_vel + angle;

            let vertex_base = segment * 6;
            vertices[vertex_base] = x_offset + wind_pos.pos.x;
            vertices[vertex_base + 1] = y_pos;
            vertices[vertex_base + 2] = z_offset + wind_pos.pos.z;
            vertices[vertex_base + 3] = x_offset + wind_pos.pos.x;
            vertices[vertex_base + 4] = y_pos - RIBBON_HEIGHT;
            vertices[vertex_base + 5] = z_offset + wind_pos.pos.z;

            let index_base = segment * 2;
            model_indices[index_base] = index_base as i16;
            model_indices[index_base + 1] = (index_base + 1) as i16;

            y_pos = pos_vel_y + y_pos;
        }
    }

    let mut model = world.obj_mut::<HurricaneModel>(this);
    model.model_vertex_count = segment_count << 1;
    model.model_index_count = wind_pos.segment_count << 1;
}

/// `-[HurricaneModel setHurricaneWindPosCount:]` @ 0x10018b9e0
pub fn set_hurricane_wind_pos_count(world: &mut World, this: Entity, hurricane_wind_pos_count: i32) {
    world.obj_mut::<HurricaneModel>(this).hurricane_wind_pos_count = hurricane_wind_pos_count;
}

/// `-[HurricaneModel hurricaneWindPosArray]` @ 0x10018b9f0
///
/// The original returns a pointer to the records, which `HurricaneLogic` seeds and animates
/// in place; the port hands out the ivar itself.
pub fn hurricane_wind_pos_array(world: &mut World, this: Entity) -> Mut<'_, [HurricaneWindPos; 5]> {
    let model = world.obj_mut::<HurricaneModel>(this);
    model.map_unchanged(|model| &mut model.hurricane_wind_pos_array)
}

/// `-[HurricaneModel createModel]` @ 0x10018ba00
pub fn create_model(world: &mut World, this: Entity) {
    free_model(world, this);

    let mut model = world.obj_mut::<HurricaneModel>(this);
    model.model_indices = vec![0; MODEL_INDEX_CAPACITY];
    model.model_vertices = vec![0.0; MODEL_VERTEX_FLOAT_CAPACITY];
}

/// `-[HurricaneModel drawOpenGL:]` @ 0x10018ba54
///
/// The five ribbons in black at the model's alpha, untextured; the blending state is whatever
/// the previous draw left.
pub fn draw_open_gl(world: &mut World, this: Entity, gl: &mut GlContext) {
    let display = display::object(world, this);
    let pos = display.pos;
    let alpha = display.alpha;

    let wind_pos_array = world.obj::<HurricaneModel>(this).hurricane_wind_pos_array.clone();
    let (mut model_vertices, mut model_indices) = {
        let mut model = world.obj_mut::<HurricaneModel>(this);
        let vertices = std::mem::take(&mut model.model_vertices);
        let indices = std::mem::take(&mut model.model_indices);
        (vertices, indices)
    };

    gl.push_matrix();
    gl.translate(pos.x, pos.y, pos.z);
    gl.set_texture_2d(false);
    gl.set_color(0.0, 0.0, 0.0, alpha);

    for wind_pos in &wind_pos_array {
        calc_model_vertices(world, this, &mut model_vertices, &mut model_indices, wind_pos);
        let index_count = world.obj::<HurricaneModel>(this).model_index_count;
        let strip = strip_vertices(&model_vertices, &model_indices, index_count);
        gl.draw_strip(&strip, &[]);
    }

    gl.set_texture_2d(true);
    gl.pop_matrix();

    let mut model = world.obj_mut::<HurricaneModel>(this);
    model.model_vertices = model_vertices;
    model.model_indices = model_indices;
}
