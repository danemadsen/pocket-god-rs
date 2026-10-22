//! `LocustModel` (superclass `BCDisplayObject`).

use crate::prelude::*;

use pg_engine::display;
use pg_engine::library;
use pg_engine::texture::Textures;

/// `LocustModel` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct LocustModel {
    /// `mpLocustPointArray`: `^f`
    ///
    /// The original keeps a pointer to `LocustLogic`'s static point array, which the logic
    /// rewrites and hands over again every frame; the port keeps the copy it is given. Empty
    /// stands for the NULL pointer of a model that has not been given points yet.
    pub locust_point_array: Vec<f32>,

    /// `mLocustPointCount`: `i`
    pub locust_point_count: i32,
}

pub const LOCUST_MODEL: Class = Class {
    name: "LocustModel",
    superclass: Some(&pg_engine::display::BC_DISPLAY_OBJECT),
    draw: draw_open_gl,
    ..pg_engine::display::BC_DISPLAY_OBJECT
};

/// The texture definition of the locust atlas page: its width is the point size
/// (`DAT_1007b8fc0`) and its page the bound texture (`DAT_1007b8f98`).
const LOCUST_ATLAS_ID: &str = "LocustAtlas0";

/// `-[LocustModel initWithId:]` @ 0x100069b88
pub fn init_with_id(world: &mut World, this: Entity, id: &str) {
    world.entity_mut(this).insert(LocustModel::default());

    let display_object = DisplayObject::new(Some(id));
    world.entity_mut(this).insert(display_object);
}

/// `-[LocustModel setLocustPointArray:count:]` @ 0x100069c00
pub fn set_locust_point_array(world: &mut World, this: Entity, locust_point_array: Vec<f32>, count: i32) {
    let mut model = world.obj_mut::<LocustModel>(this);
    model.locust_point_array = locust_point_array;
    model.locust_point_count = count;
}

/// `-[LocustModel drawOpenGL:]` @ 0x100069c1c
///
/// Point sprites (`docs/re/rendering.md` §8.3): every locust is the whole 16×16 locust atlas
/// as a screen-aligned square of `width` pixels, in the current colour (never set here).
pub fn draw_open_gl(world: &mut World, this: Entity, gl: &mut GlContext) {
    let (points, count) = {
        let model = world.obj::<LocustModel>(this);
        if model.locust_point_array.is_empty() {
            return;
        }
        (model.locust_point_array.clone(), model.locust_point_count)
    };

    let pos = display::object(world, this).pos;

    let texture_def = library::texture_def_id(world, LOCUST_ATLAS_ID);
    let (point_size, page) = match texture_def {
        Some(texture_def) => {
            let textures = world.resource::<Textures>();
            (textures.def(texture_def).width, textures.page_of(texture_def))
        }
        None => (0.0, None),
    };

    // A negative count draws nothing (`glDrawArrays` rejects it); the array always holds
    // the whole swarm, so the count never runs past it.
    let point_count = usize::try_from(count).unwrap_or(0);
    let mut sprite_points = Vec::with_capacity(point_count);
    for pair in points.chunks_exact(2).take(point_count) {
        sprite_points.push([pair[0], pair[1]]);
    }

    gl.push_matrix();
    gl.translate(pos.x, pos.y, pos.z);
    gl.set_blend(true);
    gl.bind_texture(page);
    gl.draw_point_sprites(&sprite_points, point_size);
    gl.pop_matrix();
}
