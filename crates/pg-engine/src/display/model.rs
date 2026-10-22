//! `BCModel`: a textured quad (`docs/re/rendering.md` §5.3–§5.4).

use bevy_ecs::prelude::*;

use crate::display::{self, BC_DISPLAY_OBJECT, DisplayObject};
use crate::gl::GlContext;
use crate::object::{Class, ObjectWorldExt};
use crate::texture::{TextureDefId, Textures};

/// The depth at which a sprite keeps its natural size (`0xc38a8f5c`, rendering.md §1.8).
pub const SPRITE_Z: f32 = -277.12;

/// The horizon line that the fake perspective pulls deeper sprites toward.
const HORIZON_Y: f32 = 160.0;

/// `BCModel` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct Model {
    pub texture_def: Option<TextureDefId>,
}

pub const BC_MODEL: Class = Class {
    name: "BCModel",
    superclass: Some(&BC_DISPLAY_OBJECT),
    draw: draw_model,
    draw_in_multi_model: draw_model_in_multi_model,
    texture_def: model_texture_def,
    set_texture_def: set_model_texture_def,
    enable_multi_model_mode,
    disable_multi_model_mode,
    ..BC_DISPLAY_OBJECT
};

/// `-[BCModel initWithId:textureDef:]` @ 0x100118720.
pub fn spawn(world: &mut World, id: Option<&str>, texture_def: Option<TextureDefId>) -> Entity {
    let entity = display::spawn(world, &BC_MODEL, id);
    world.entity_mut(entity).insert(Model { texture_def });
    entity
}

/// `BCModel::textureDef` @ 0x100118788.
fn model_texture_def(world: &World, entity: Entity) -> Option<TextureDefId> {
    world.obj::<Model>(entity).texture_def
}

/// `BCModel::setTextureDef:` @ 0x100118798 (a plain store).
fn set_model_texture_def(world: &mut World, entity: Entity, texture: Option<TextureDefId>) {
    world.obj_mut::<Model>(entity).texture_def = texture;
}

/// `BCModel::enableMultiModelMode` @ 0x100118a6c.
pub fn enable_multi_model_mode(world: &mut World, entity: Entity) {
    display::object_mut(world, entity).multi_model_mode = true;
}

/// `BCModel::disableMultiModelMode` @ 0x100118a84.
pub fn disable_multi_model_mode(world: &mut World, entity: Entity) {
    display::object_mut(world, entity).multi_model_mode = false;
}

/// The fake perspective factor `kz = -277.12 / pos.z` of the sprite classes (rendering.md §1.8).
pub fn depth_factor(z: f32) -> f32 {
    SPRITE_Z / z
}

/// The shared model transform of `BCModel`/`BCMultiModel` (rendering.md §5.3), applied inside a push.
pub fn apply_model_transform(world: &World, entity: Entity, gl: &mut GlContext) {
    let display = display::object(world, entity);
    let pos = display.pos;
    let rot = display.rot;
    let skew = display.skew;
    let scale = display.scale;
    let pivot = display.pivot_pos;
    let origin = display.origin_pos;

    let depth = depth_factor(pos.z);
    let perspective_y = depth * (pos.y - HORIZON_Y) + HORIZON_Y;
    let scale_x = depth * scale.x;
    let scale_y = depth * scale.y;
    let pivot_x = scale_x * pivot.x;
    let pivot_y = scale_y * pivot.y;
    let pivot_z = scale.z * pivot.z;

    gl.translate(pos.x, perspective_y, pos.z);
    gl.translate(pivot_x, pivot_y, pivot_z);
    gl.rotate(rot.x, rot.y, rot.z);
    gl.skew(skew.x, skew.y);
    gl.scale(scale_x, scale_y, scale.z);
    gl.translate(-pivot.x, -pivot.y, -pivot.z);
    gl.translate(origin.x, origin.y, origin.z);
}

/// The reduced sprite transform of `BCRect` and `BCText`: perspective position, rotation and
/// perspective scale, without pivot, skew or origin (rendering.md §5.7, §5.8).
pub(crate) fn apply_sprite_transform(display: &DisplayObject, gl: &mut GlContext) {
    let pos = display.pos;
    let rot = display.rot;
    let scale = display.scale;

    let depth = depth_factor(pos.z);
    let perspective_y = depth * (pos.y - HORIZON_Y) + HORIZON_Y;
    let scale_x = depth * scale.x;
    let scale_y = depth * scale.y;

    gl.translate(pos.x, perspective_y, pos.z);
    gl.rotate(rot.x, rot.y, rot.z);
    gl.scale(scale_x, scale_y, scale.z);
}

/// `setColor(R·A, G·A, B·A, A)`: the premultiplied own tint of the model classes.
pub(crate) fn set_premultiplied_color(display: &DisplayObject, gl: &mut GlContext) {
    let alpha = display.alpha;
    let red = display.red * alpha;
    let green = display.green * alpha;
    let blue = display.blue * alpha;
    gl.set_color(red, green, blue, alpha);
}

/// The four corners of a flat `float[12]` quad, in strip order.
pub(crate) fn quad_corners(flat: &[f32; 12]) -> [[f32; 3]; 4] {
    [
        [flat[0], flat[1], flat[2]],
        [flat[3], flat[4], flat[5]],
        [flat[6], flat[7], flat[8]],
        [flat[9], flat[10], flat[11]],
    ]
}

/// The four UVs of a flat `float[8]` array, in strip order.
pub(crate) fn quad_uvs(flat: &[f32; 8]) -> [[f32; 2]; 4] {
    [[flat[0], flat[1]], [flat[2], flat[3]], [flat[4], flat[5]], [flat[6], flat[7]]]
}

/// The blend rule of every textured quad draw: `texDef.blend || alpha < 1`.
pub(crate) fn quad_blend(world: &World, texture: TextureDefId, alpha: f32) -> bool {
    let textures = world.resource::<Textures>();
    textures.def(texture).blend || alpha < 1.0
}

/// Binds the page of `texture`, switches blending and draws `vertices` as a 4-vertex strip
/// with the definition's current UVs (indices `{0, 1, 2, 3}`).
pub(crate) fn draw_quad(world: &World, texture: TextureDefId, vertices: &[f32; 12], blend: bool, gl: &mut GlContext) {
    let textures = world.resource::<Textures>();
    let page = textures.page_of(texture);
    let uvs = textures.def(texture).uvs;
    let corners = quad_corners(vertices);
    let coords = quad_uvs(&uvs);

    gl.bind_texture(page);
    gl.set_blend(blend);
    gl.draw_strip(&corners, &coords);
}

/// `FUN_10003841c`: draw a texture definition's own quad with the current colour.
pub fn draw_texture_def(world: &World, texture: Option<TextureDefId>, object_alpha: f32, gl: &mut GlContext) {
    let Some(texture) = texture else {
        return;
    };
    let vertices = world.resource::<Textures>().def(texture).vertices;
    let blend = quad_blend(world, texture, object_alpha);
    draw_quad(world, texture, &vertices, blend, gl);
}

/// `BCModel::drawOpenGL:` @ 0x1001187e8.
fn draw_model(world: &mut World, entity: Entity, gl: &mut GlContext) {
    let display = display::object(world, entity);
    if display.hidden || display.multi_model_mode {
        return;
    }
    let alpha = display.alpha;
    set_premultiplied_color(display, gl);

    gl.push_matrix();
    apply_model_transform(world, entity, gl);
    let texture = world.obj::<Model>(entity).texture_def;
    draw_texture_def(world, texture, alpha, gl);
    gl.pop_matrix();
}

/// `BCModel::drawInMultiModelOpenGL:…` @ 0x1001189d4.
fn draw_model_in_multi_model(world: &mut World, entity: Entity, gl: &mut GlContext, _parent: Entity, _rgba: [f32; 4]) {
    let display = display::object(world, entity);
    let alpha = display.alpha;
    let origin = display.origin_pos;
    set_premultiplied_color(display, gl);

    gl.translate(origin.x, origin.y, origin.z);
    let texture = world.obj::<Model>(entity).texture_def;
    draw_texture_def(world, texture, alpha, gl);
}
