//! `RainbowModel` (superclass `BCDisplayObject`).

use crate::prelude::*;

use crate::logic::sky_powers::lightning_model::path_strip_pair;
use pg_engine::display;
use pg_engine::library;
use pg_engine::texture::Textures;

/// `RainbowModel` ivars.
///
/// `path_list` is the original's shared `mpRainbowList` itself: the view controller's
/// `(Game)` rainbow gesture appends the drawn points to it and `RainbowLogic` clears, copies
/// and replays it, all through this field.
#[derive(Component, Clone, Debug, Default)]
pub struct RainbowModel {
    /// `mpPathList`: `^{list<VECTOR4, std::__1::allocator<VECTOR4> >={__list_node_base<VECTOR4, void *>=^{__list_node<VECTOR4, void *>}^{__list_node<VECTOR4, void *>}}{__compressed_pair<unsigned long, std::__1::allocator<std::__1::__list_node<VECTOR4, void *> > >=Q}}`
    pub path_list: Vec<Vec4>,

    /// `mModelVertexCount`: `i`
    pub model_vertex_count: i32,

    /// `mModelIndexCount`: `i`
    pub model_index_count: i32,

    /// `mpTextureCoords`: `^f`
    pub texture_coords: Vec<f32>,

    /// `mpModelColors`: `^f`
    pub model_colors: Vec<f32>,

    /// `mpModelVertices`: `^f`
    pub model_vertices: Vec<f32>,

    /// `mpModelIndices`: `^s`
    pub model_indices: Vec<i16>,
}

pub const RAINBOW_MODEL: Class = Class {
    name: "RainbowModel",
    superclass: Some(&pg_engine::display::BC_DISPLAY_OBJECT),
    draw: draw_open_gl,
    ..pg_engine::display::BC_DISPLAY_OBJECT
};

/// The ribbon's half-width.
const RAINBOW_LINE_WIDTH: f32 = 50.0;

/// The texture whose left texel column the whole ribbon samples (the static def at
/// `0x10052fab0`, atlas `TextureAtlasTest0`).
const RAINBOW_TEXTURE_ID: &str = "Rainbow";

/// Paths of at least this many points fade in and out over `FADE_POINT_COUNT` points.
const LONG_PATH_POINT_COUNT: f32 = 10.0;

/// Points over which a long path fades in (and out).
const FADE_POINT_COUNT: i32 = 5;

/// Brightness gained (and lost) per point while a long path fades.
const FADE_STEP: f32 = 0.2;

/// `-[RainbowModel initWithId:pathList:]` @ 0x10008ef14
///
/// The original keeps a pointer to the caller's list; here the model owns the points from now
/// on (see [`RainbowModel`]).
pub fn init_with_id(world: &mut World, this: Entity, id: &str, path_list: Vec<Vec4>) {
    world.entity_mut(this).insert(RainbowModel::default());

    // `[super initWithId:]`
    let display_object = DisplayObject::new(Some(id));
    world.entity_mut(this).insert(display_object);

    world.obj_mut::<RainbowModel>(this).path_list = path_list;
}

/// `-[RainbowModel pathList]` @ 0x10008ef6c
///
/// The original hands out the shared list; the port returns its points for reading. Writers
/// change [`RainbowModel::path_list`] in place.
pub fn path_list(world: &mut World, this: Entity) -> Vec<Vec4> {
    world.obj::<RainbowModel>(this).path_list.clone()
}

/// `-[RainbowModel freeModel]` @ 0x10008ef7c
pub fn free_model(world: &mut World, this: Entity) {
    let mut model = world.obj_mut::<RainbowModel>(this);
    model.model_colors = Vec::new();
    model.texture_coords = Vec::new();
    model.model_vertices = Vec::new();
    model.model_indices = Vec::new();
}

/// The `Rainbow` texture's `(u left, v top, v bottom)` (`uvs[0]`, `uvs[1]`, `uvs[5]`).
fn rainbow_texture_uvs(world: &World) -> (f32, f32, f32) {
    let Some(texture) = library::texture_def_id(world, RAINBOW_TEXTURE_ID) else {
        return (0.0, 0.0, 0.0);
    };
    let uvs = world.resource::<Textures>().def(texture).uvs;
    (uvs[0], uvs[1], uvs[5])
}

/// `-[RainbowModel calcModelVertices:modelIndices:modelColors:textureCoords:lineWidth:]` @ 0x10008f050
///
/// The path strip of rendering.md §8.2: premultiplied white whose brightness ramps up over the
/// first points and down over the last ones (the last pair is transparent), every vertex on
/// the left texel column of the `Rainbow` sprite (right-hand vertices at its top, left-hand
/// ones at its bottom). Each point's `w` receives its pair angle. Needs two points.
pub fn calc_model_vertices(world: &mut World, this: Entity, vertices: &mut [f32], model_indices: &mut [i16], model_colors: &mut [f32], texture_coords: &mut [f32], line_width: f32) {
    let alpha = display::object(world, this).alpha;
    let (texture_u, texture_top_v, texture_bottom_v) = rainbow_texture_uvs(world);
    let mut points = {
        let mut model = world.obj_mut::<RainbowModel>(this);
        std::mem::take(&mut model.path_list)
    };

    let point_count = points.len() as i32;
    let half_point_count = point_count / 2;
    let (fade_point_count, fade_step) = if (point_count as f32) < LONG_PATH_POINT_COUNT {
        (half_point_count, 1.0 / half_point_count as f32)
    } else {
        (FADE_POINT_COUNT, FADE_STEP)
    };
    let fade_out_start = (point_count - fade_point_count) * 2;

    let last = points.len() - 1;
    let mut brightness = 0.0_f32;
    for index in 0..=last {
        let pair = path_strip_pair(&points, index, line_width);
        points[index].w = pair.angle;

        let color = if index == last { 0.0 } else { brightness * alpha };

        let vertex_base = index * 6;
        vertices[vertex_base] = pair.right.x;
        vertices[vertex_base + 1] = pair.right.y;
        vertices[vertex_base + 2] = 0.0;
        vertices[vertex_base + 3] = pair.left.x;
        vertices[vertex_base + 4] = pair.left.y;
        vertices[vertex_base + 5] = 0.0;

        let texture_base = index * 4;
        texture_coords[texture_base] = texture_u;
        texture_coords[texture_base + 1] = texture_top_v;
        texture_coords[texture_base + 2] = texture_u;
        texture_coords[texture_base + 3] = texture_bottom_v;

        let color_base = index * 8;
        model_colors[color_base..color_base + 8].fill(color);

        let index_base = index * 2;
        model_indices[index_base] = index_base as i16;
        model_indices[index_base + 1] = (index_base + 1) as i16;

        if index == last {
            break;
        }

        let next_index_base = (index_base + 2) as i32;
        if next_index_base < fade_out_start {
            brightness = fade_step + brightness;
            if brightness > 1.0 {
                brightness = 1.0;
            }
        } else {
            brightness = brightness - fade_step;
            if brightness < 0.0 {
                brightness = 0.0;
            }
        }
    }

    world.obj_mut::<RainbowModel>(this).path_list = points;
}

/// `-[RainbowModel createModel]` @ 0x10008f4a4
pub fn create_model(world: &mut World, this: Entity) {
    free_model(world, this);

    let point_count = world.obj::<RainbowModel>(this).path_list.len();
    if point_count < 2 {
        let mut model = world.obj_mut::<RainbowModel>(this);
        model.model_vertex_count = 0;
        model.model_index_count = 0;
        return;
    }

    let model_vertex_count = (point_count as i32) << 1;
    let model_index_count = ((point_count as u32) << 1) as i32;
    {
        let mut model = world.obj_mut::<RainbowModel>(this);
        model.model_vertex_count = model_vertex_count;
        model.model_index_count = model_index_count;
    }

    let vertex_count = model_vertex_count as usize;
    let mut model_indices = vec![0; model_index_count as usize];
    let mut model_colors = vec![0.0; vertex_count * 4];
    let mut texture_coords = vec![0.0; vertex_count * 2];
    let mut model_vertices = vec![0.0; vertex_count * 3];
    calc_model_vertices(world, this, &mut model_vertices, &mut model_indices, &mut model_colors, &mut texture_coords, RAINBOW_LINE_WIDTH);

    let mut model = world.obj_mut::<RainbowModel>(this);
    model.model_indices = model_indices;
    model.model_colors = model_colors;
    model.texture_coords = texture_coords;
    model.model_vertices = model_vertices;
}

/// `-[RainbowModel drawOpenGL:]` @ 0x10008f5d8
///
/// Rebuilds the ribbon every frame from the current points, then draws it textured, with a
/// colour array and blending on.
pub fn draw_open_gl(world: &mut World, this: Entity, gl: &mut GlContext) {
    if world.obj::<RainbowModel>(this).path_list.len() < 2 {
        return;
    }
    create_model(world, this);

    let pos = display::object(world, this).pos;
    let texture_page = match library::texture_def_id(world, RAINBOW_TEXTURE_ID) {
        Some(texture) => world.resource::<Textures>().page_of(texture),
        None => None,
    };

    // `glDrawElements` over the vertex, texture-coordinate and colour arrays.
    let model = world.obj::<RainbowModel>(this);
    let used_count = model.model_index_count.max(0) as usize;
    let mut strip_vertices = Vec::with_capacity(used_count);
    let mut strip_uvs = Vec::with_capacity(used_count);
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

        let texture_base = vertex * 2;
        let uv = [model.texture_coords[texture_base], model.texture_coords[texture_base + 1]];
        strip_uvs.push(uv);

        let color_base = vertex * 4;
        let color = [
            model.model_colors[color_base],
            model.model_colors[color_base + 1],
            model.model_colors[color_base + 2],
            model.model_colors[color_base + 3],
        ];
        strip_colors.push(color);
    }

    gl.push_matrix();
    gl.translate(pos.x, pos.y, pos.z);
    gl.bind_texture(texture_page);
    gl.set_blend(true);
    gl.draw_colored_strip(&strip_vertices, &strip_uvs, &strip_colors);
    gl.pop_matrix();
}
