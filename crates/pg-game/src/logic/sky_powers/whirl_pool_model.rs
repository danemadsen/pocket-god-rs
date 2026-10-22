//! `WhirlPoolModel` (superclass `BCDisplayObject`).

use crate::prelude::*;

use crate::logic::sky_powers::lightning_model::strip_vertices;
use pg_engine::display;

/// `WhirlPoolModel` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct WhirlPoolModel {
    /// `mWhirlPoolWindPosCount`: `i`
    pub whirl_pool_wind_pos_count: i32,

    /// `mWhirlPoolWindPosArray`: `[5{WhirlPoolWindPos="mPos"{?="x"f"y"f"z"f"w"f}"mPosVel"{?="x"f"y"f"z"f"w"f}"mWindRot"f"mWindRotVel"f"mRadius"f"mRadiusVel"f"mDrawPos"f"mDrawPosVel"f"mLineWidth"f"mSegmentCount"i}]`
    pub whirl_pool_wind_pos_array: [WhirlPoolWindPos; 5],

    /// `mModelVertexCount`: `i`
    pub model_vertex_count: i32,

    /// `mModelIndexCount`: `i`
    pub model_index_count: i32,

    /// `mpModelVertices`: `^f`
    pub model_vertices: Vec<f32>,

    /// `mpModelIndices`: `^s`
    pub model_indices: Vec<i16>,
}

pub const WHIRL_POOL_MODEL: Class = Class {
    name: "WhirlPoolModel",
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

/// The green and blue share of the model's alpha (the ribbons are cyan).
const RIBBON_COLOR_SCALE: f32 = 0.8;

/// `-[WhirlPoolModel initWithId:]` @ 0x10011ab88
pub fn init_with_id(world: &mut World, this: Entity, id: &str) {
    // The zeroed component also stands for the `bzero` of the wind records.
    world.entity_mut(this).insert(WhirlPoolModel::default());

    // `[super initWithId:]`
    let display_object = DisplayObject::new(Some(id));
    world.entity_mut(this).insert(display_object);

    create_model(world, this);
}

/// `-[WhirlPoolModel freeModel]` @ 0x10011ac00
pub fn free_model(world: &mut World, this: Entity) {
    let mut model = world.obj_mut::<WhirlPoolModel>(this);
    model.model_vertices = Vec::new();
    model.model_indices = Vec::new();
}

/// `-[WhirlPoolModel calcModelVertices:modelIndices:whirlPoolWindPos:]` @ 0x10011aca4
///
/// One wind ribbon (rendering.md §8.5): `mSegmentCount` vertex pairs along a flat spiral
/// around `mPos`, starting `mDrawPos` segments along. Also sets the model's vertex and index
/// counts.
pub fn calc_model_vertices(world: &mut World, this: Entity, vertices: &mut [f32], model_indices: &mut [i16], whirl_pool_wind_pos: &WhirlPoolWindPos) {
    let wind_pos = whirl_pool_wind_pos;
    let segment_count = wind_pos.segment_count;

    if segment_count > 0 {
        let radius_vel = wind_pos.radius_vel;
        let wind_rot_vel = wind_pos.wind_rot_vel;
        let draw_pos = wind_pos.draw_pos;
        let mut angle = wind_pos.wind_rot + draw_pos * wind_rot_vel;
        let mut radius = wind_pos.radius + radius_vel * draw_pos;

        for segment in 0..segment_count as usize {
            let x_offset = radius * angle.cos();
            let y_offset = radius * angle.sin();

            let vertex_base = segment * 6;
            vertices[vertex_base] = x_offset + wind_pos.pos.x;
            vertices[vertex_base + 1] = y_offset + wind_pos.pos.y;
            vertices[vertex_base + 2] = 0.0;
            vertices[vertex_base + 3] = x_offset + wind_pos.pos.x;
            vertices[vertex_base + 4] = (y_offset + wind_pos.pos.y) - RIBBON_HEIGHT;
            vertices[vertex_base + 5] = 0.0;

            let index_base = segment * 2;
            model_indices[index_base] = index_base as i16;
            model_indices[index_base + 1] = (index_base + 1) as i16;

            radius = radius_vel + radius;
            angle = wind_rot_vel + angle;
        }
    }

    let mut model = world.obj_mut::<WhirlPoolModel>(this);
    model.model_vertex_count = segment_count << 1;
    model.model_index_count = wind_pos.segment_count << 1;
}

/// `-[WhirlPoolModel setWhirlPoolWindPosCount:]` @ 0x10011ade4
pub fn set_whirl_pool_wind_pos_count(world: &mut World, this: Entity, whirl_pool_wind_pos_count: i32) {
    world.obj_mut::<WhirlPoolModel>(this).whirl_pool_wind_pos_count = whirl_pool_wind_pos_count;
}

/// `-[WhirlPoolModel whirlPoolWindPosArray]` @ 0x10011adf4
///
/// The original returns a pointer to the records, which `WhirlPoolLogic` seeds and animates
/// in place; the port hands out the ivar itself.
pub fn whirl_pool_wind_pos_array(world: &mut World, this: Entity) -> Mut<'_, [WhirlPoolWindPos; 5]> {
    let model = world.obj_mut::<WhirlPoolModel>(this);
    model.map_unchanged(|model| &mut model.whirl_pool_wind_pos_array)
}

/// `-[WhirlPoolModel createModel]` @ 0x10011ae04
pub fn create_model(world: &mut World, this: Entity) {
    free_model(world, this);

    let mut model = world.obj_mut::<WhirlPoolModel>(this);
    model.model_indices = vec![0; MODEL_INDEX_CAPACITY];
    model.model_vertices = vec![0.0; MODEL_VERTEX_FLOAT_CAPACITY];
}

/// `-[WhirlPoolModel drawOpenGL:]` @ 0x10011ae58
///
/// The five ribbons in cyan `(0, 0.8a, 0.8a, a)`, untextured; the blending state is whatever
/// the previous draw left.
pub fn draw_open_gl(world: &mut World, this: Entity, gl: &mut GlContext) {
    let display = display::object(world, this);
    let pos = display.pos;
    let alpha = display.alpha;

    let wind_pos_array = world.obj::<WhirlPoolModel>(this).whirl_pool_wind_pos_array.clone();
    let (mut model_vertices, mut model_indices) = {
        let mut model = world.obj_mut::<WhirlPoolModel>(this);
        let vertices = std::mem::take(&mut model.model_vertices);
        let indices = std::mem::take(&mut model.model_indices);
        (vertices, indices)
    };

    gl.push_matrix();
    gl.translate(pos.x, pos.y, pos.z);
    gl.set_texture_2d(false);
    let color = alpha * RIBBON_COLOR_SCALE;
    gl.set_color(0.0, color, color, alpha);

    for wind_pos in &wind_pos_array {
        calc_model_vertices(world, this, &mut model_vertices, &mut model_indices, wind_pos);
        let index_count = world.obj::<WhirlPoolModel>(this).model_index_count;
        let strip = strip_vertices(&model_vertices, &model_indices, index_count);
        gl.draw_strip(&strip, &[]);
    }

    gl.set_texture_2d(true);
    gl.pop_matrix();

    let mut model = world.obj_mut::<WhirlPoolModel>(this);
    model.model_vertices = model_vertices;
    model.model_indices = model_indices;
}
