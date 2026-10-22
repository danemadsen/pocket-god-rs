//! `BCTileModel`: a wrapping, scrolling strip (`docs/re/rendering.md` §5.6).

use bevy_ecs::prelude::*;

use crate::animation::FrameRef;
use crate::display::multi_model::{FrameState, MultiModel, frame_state, frame_state_indices};
use crate::display::{self, DisplayObject, model, multi_model};
use crate::gl::GlContext;
use crate::object::{Class, ObjectWorldExt};

/// The right edge of the 3:2 game area the horizontal transitions work against.
const SCREEN_WIDTH: f32 = 480.0;

/// The top edge of the 3:2 game area the vertical transitions work against.
const SCREEN_HEIGHT: f32 = 320.0;

/// The transition window of a fully visible strip (`0x447a0000`).
const TRANSITION_LIMIT: f32 = 1000.0;

/// `BCTileModel` ivars.
#[derive(Component, Clone, Debug)]
pub struct TileModel {
    pub wrap_width: f32,
    pub wrap_height: f32,
    pub vertical: bool,
    pub transition_min_x_pos: f32,
    pub transition_max_x_pos: f32,
    pub transition_min_y_pos: f32,
    pub transition_max_y_pos: f32,
}

pub const BC_TILE_MODEL: Class = Class {
    name: "BCTileModel",
    superclass: Some(&multi_model::BC_MULTI_MODEL),
    draw: draw_tile_model,
    ..multi_model::BC_MULTI_MODEL
};

/// `-[BCTileModel initWithId:wrapWidth:wrapHeight:]` @ 0x100021ec0.
pub fn spawn(world: &mut World, id: Option<&str>, wrap_width: f32, wrap_height: f32) -> Entity {
    let entity = display::spawn(world, &BC_TILE_MODEL, id);
    let tile = TileModel {
        wrap_width,
        wrap_height,
        vertical: wrap_width == 0.0,
        transition_min_x_pos: -TRANSITION_LIMIT,
        transition_max_x_pos: TRANSITION_LIMIT,
        transition_min_y_pos: -TRANSITION_LIMIT,
        transition_max_y_pos: TRANSITION_LIMIT,
    };
    let components = (model::Model::default(), multi_model::MultiModel::default(), tile);
    world.entity_mut(entity).insert(components);
    entity
}

/// `-setBinaryFrameDef:` @ 0x100021ff8: stores the frame and clears the texture def; the
/// channel array is left as it is.
pub fn set_binary_frame_def(world: &mut World, entity: Entity, frame: Option<FrameRef>) {
    world.obj_mut::<MultiModel>(entity).frame = frame;
    world.obj_mut::<model::Model>(entity).texture_def = None;
}

/// `value` wrapped into `[-period, 0]`: `fmod` in double, then shifted down a period when
/// `value` is positive (the shared start of the draws and the transition helpers).
fn wrap(value: f32, period: f32) -> f32 {
    let remainder = (f64::from(value) % f64::from(period)) as f32;
    if value > 0.0 { remainder - period } else { remainder }
}

/// The states of the model's current frame (none without a frame; the original would crash).
fn current_states(world: &World, entity: Entity) -> Vec<FrameState> {
    let Some(frame) = world.obj::<MultiModel>(entity).frame else {
        return Vec::new();
    };
    let indices = frame_state_indices(world, frame);
    let mut states = Vec::with_capacity(indices.len());
    for index in indices {
        states.push(frame_state(world, index));
    }
    states
}

/// `-xPos` and `-displayGroup` of an object; nil (or a despawned entity) answers zeros.
fn x_pos_and_parent(world: &World, object: Option<Entity>) -> (f32, Option<Entity>) {
    match object.and_then(|entity| world.get::<DisplayObject>(entity)) {
        Some(display) => (display.pos.x, display.display_group),
        None => (0.0, None),
    }
}

/// Σ xPos of the ancestor groups, excluding the top-most one.
fn ancestor_x_offset(world: &World, entity: Entity) -> f32 {
    let parent = display::object(world, entity).display_group;
    let (mut x_pos, mut next) = x_pos_and_parent(world, parent);
    let mut offset = 0.0;
    while let Some(group) = next {
        offset += x_pos;
        (x_pos, next) = x_pos_and_parent(world, Some(group));
    }
    offset
}

/// One tile: the state quad with colour `(a·R, a·G, a·B, a)`, `a = state alpha · A`.
fn draw_tile(world: &World, entity: Entity, state: &FrameState, gl: &mut GlContext) {
    let display = display::object(world, entity);
    let alpha = state.alpha * display.alpha;
    let red = alpha * display.red;
    let green = alpha * display.green;
    let blue = alpha * display.blue;
    gl.set_color(red, green, blue, alpha);

    // Every tile has a texture in the data; the original dereferences it unchecked.
    let Some(texture) = state.texture else {
        return;
    };
    let blend = model::quad_blend(world, texture, alpha);
    model::draw_quad(world, texture, &state.vertices, blend, gl);
}

/// `BCTileModel::drawOpenGL:` @ 0x100022b64.
fn draw_tile_model(world: &mut World, entity: Entity, gl: &mut GlContext) {
    let vertical = world.obj::<TileModel>(entity).vertical;
    if vertical {
        draw_vertical(world, entity, gl);
    } else {
        draw_horizontal(world, entity, gl);
    }
}

/// `BCTileModel::drawOpenGLHorizontal:` @ 0x100022618.
fn draw_horizontal(world: &World, entity: Entity, gl: &mut GlContext) {
    let display = display::object(world, entity);
    let has_frame = world.obj::<MultiModel>(entity).frame.is_some();
    if display.hidden || !has_frame {
        return;
    }
    let pos = display.pos;
    let tile = world.obj::<TileModel>(entity);

    let offset = ancestor_x_offset(world, entity);
    let wrapped = wrap(offset + pos.x, tile.wrap_width);
    gl.push_matrix();
    gl.translate(wrapped - offset, pos.y, pos.z);

    let limit_max = tile.transition_max_x_pos - wrapped;
    let screen_max = SCREEN_WIDTH - wrapped;
    let max_x = if limit_max < screen_max { limit_max } else { screen_max };
    let limit_min = tile.transition_min_x_pos - wrapped;
    let screen_min = -wrapped;
    let min_x = if limit_min > screen_min { limit_min } else { screen_min };

    for state in current_states(world, entity) {
        let v = &state.vertices;
        let mut low = if v[3] < v[0] { v[3] } else { v[0] };
        let mut high = if v[3] > v[0] { v[3] } else { v[0] };
        low = if v[6] < low { v[6] } else { low };
        high = if v[6] > high { v[6] } else { high };
        low = if v[9] < low { v[9] } else { low };
        high = if v[9] > high { v[9] } else { high };

        if low <= max_x && high >= min_x {
            draw_tile(world, entity, &state, gl);
        }
    }
    gl.pop_matrix();
}

/// `BCTileModel::drawOpenGLVertical:` @ 0x100022908.
fn draw_vertical(world: &World, entity: Entity, gl: &mut GlContext) {
    let display = display::object(world, entity);
    let has_frame = world.obj::<MultiModel>(entity).frame.is_some();
    if display.hidden || !has_frame {
        return;
    }
    let pos = display.pos;
    let tile = world.obj::<TileModel>(entity);

    let parent = display.display_group;
    let parent_y = match parent.and_then(|group| world.get::<DisplayObject>(group)) {
        Some(group) => group.pos.y,
        None => 0.0,
    };
    let wrapped = wrap(parent_y + pos.y, tile.wrap_height);
    gl.push_matrix();
    gl.translate(pos.x, wrapped - parent_y, pos.z);

    let limit_max = tile.transition_max_y_pos - wrapped;
    let screen_max = SCREEN_HEIGHT - wrapped;
    let max_y = if limit_max < screen_max { limit_max } else { screen_max };
    let limit_min = tile.transition_min_y_pos - wrapped;
    let screen_min = -wrapped;
    let min_y = if limit_min > screen_min { limit_min } else { screen_min };

    for state in current_states(world, entity) {
        let bottom = state.vertices[7];
        let top = state.vertices[1];
        if bottom <= max_y && top >= min_y {
            draw_tile(world, entity, &state, gl);
        }
    }
    gl.pop_matrix();
}

/// `-resetTransitionVisible` @ 0x10002205c.
pub fn reset_transition_visible(world: &mut World, entity: Entity) {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_x_pos = TRANSITION_LIMIT;
    tile.transition_min_x_pos = -TRANSITION_LIMIT;
}

/// `-resetTransitionInvisible` @ 0x100022080.
pub fn reset_transition_invisible(world: &mut World, entity: Entity) {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_x_pos = -TRANSITION_LIMIT;
    tile.transition_min_x_pos = TRANSITION_LIMIT;
}

/// `-startTransitionInLeft` @ 0x1000220a4: the strip appears from the left edge.
pub fn start_transition_in_left(world: &mut World, entity: Entity) {
    let pos_x = display::object(world, entity).pos.x;
    let wrap_width = world.obj::<TileModel>(entity).wrap_width;
    let wrapped = wrap(pos_x, wrap_width);

    let mut limit = SCREEN_WIDTH - wrapped;
    let screen_left = -wrapped;
    for state in current_states(world, entity) {
        let left = state.vertices[0];
        let right = state.vertices[3];
        if right > screen_left && limit > left {
            limit = left;
        }
    }

    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_x_pos = wrapped + limit;
    tile.transition_min_x_pos = 0.0;
}

/// `-startTransitionInRight` @ 0x100022198: the strip appears from the right edge.
pub fn start_transition_in_right(world: &mut World, entity: Entity) {
    let pos_x = display::object(world, entity).pos.x;
    let wrap_width = world.obj::<TileModel>(entity).wrap_width;
    let wrapped = wrap(pos_x, wrap_width);

    let mut limit = -wrapped;
    let screen_right = SCREEN_WIDTH - wrapped;
    for state in current_states(world, entity) {
        let left = state.vertices[0];
        if left > screen_right && limit < left {
            limit = left;
        }
    }

    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_min_x_pos = wrapped + limit;
    tile.transition_max_x_pos = SCREEN_WIDTH;
}

/// `-startTransitionOutLeft` @ 0x100022178.
pub fn start_transition_out_left(world: &mut World, entity: Entity) {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_x_pos = SCREEN_WIDTH;
    tile.transition_min_x_pos = 0.0;
}

/// `-startTransitionOutRight` @ 0x10002226c.
pub fn start_transition_out_right(world: &mut World, entity: Entity) {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_x_pos = SCREEN_WIDTH;
    tile.transition_min_x_pos = 0.0;
}

/// `-transitionInLeft:` @ 0x10002228c.
pub fn transition_in_left(world: &mut World, entity: Entity, delta: f32) -> bool {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_x_pos += delta;
    tile.transition_max_x_pos > SCREEN_WIDTH
}

/// `-transitionOutLeft:` @ 0x1000222b4.
pub fn transition_out_left(world: &mut World, entity: Entity, delta: f32) -> bool {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_min_x_pos += delta;
    tile.transition_min_x_pos > SCREEN_WIDTH
}

/// `-transitionInRight:` @ 0x1000222dc.
pub fn transition_in_right(world: &mut World, entity: Entity, delta: f32) -> bool {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_min_x_pos += delta;
    tile.transition_min_x_pos < 0.0
}

/// `-transitionOutRight:` @ 0x1000222fc.
pub fn transition_out_right(world: &mut World, entity: Entity, delta: f32) -> bool {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_x_pos += delta;
    tile.transition_max_x_pos < 0.0
}

/// `-startTransitionInBottom` @ 0x10002235c: the strip appears from the bottom edge.
pub fn start_transition_in_bottom(world: &mut World, entity: Entity) {
    let pos_y = display::object(world, entity).pos.y;
    let wrap_height = world.obj::<TileModel>(entity).wrap_height;
    let wrapped = wrap(pos_y, wrap_height);

    let mut limit = SCREEN_HEIGHT - wrapped;
    let screen_bottom = -wrapped;
    for state in current_states(world, entity) {
        let top = state.vertices[1];
        let bottom = state.vertices[7];
        if bottom > screen_bottom && limit > top {
            limit = top;
        }
    }

    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_y_pos = wrapped + limit;
    tile.transition_min_y_pos = 0.0;
}

/// `-startTransitionInTop` @ 0x100022454: the strip appears from the top edge.
pub fn start_transition_in_top(world: &mut World, entity: Entity) {
    let pos_y = display::object(world, entity).pos.y;
    let wrap_height = world.obj::<TileModel>(entity).wrap_height;
    let wrapped = wrap(pos_y, wrap_height);

    let mut limit = -wrapped;
    let screen_top = SCREEN_HEIGHT - wrapped;
    for state in current_states(world, entity) {
        let top = state.vertices[1];
        if top > screen_top && limit < top {
            limit = top;
        }
    }

    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_min_y_pos = wrapped + limit;
    tile.transition_max_y_pos = SCREEN_HEIGHT;
}

/// `-startTransitionOutBottom` @ 0x100022434 (sets the maximum to 480, an original bug).
pub fn start_transition_out_bottom(world: &mut World, entity: Entity) {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_y_pos = SCREEN_WIDTH;
    tile.transition_min_y_pos = 0.0;
}

/// `-startTransitionOutTop` @ 0x10002252c.
pub fn start_transition_out_top(world: &mut World, entity: Entity) {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_y_pos = SCREEN_HEIGHT;
    tile.transition_min_y_pos = 0.0;
}

/// `-transitionInBottom:` @ 0x10002254c.
pub fn transition_in_bottom(world: &mut World, entity: Entity, delta: f32) -> bool {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_y_pos += delta;
    tile.transition_max_y_pos > SCREEN_HEIGHT
}

/// `-transitionOutBottom:` @ 0x100022574.
pub fn transition_out_bottom(world: &mut World, entity: Entity, delta: f32) -> bool {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_min_y_pos += delta;
    tile.transition_min_y_pos > SCREEN_HEIGHT
}

/// `-transitionInTop:` @ 0x10002259c.
pub fn transition_in_top(world: &mut World, entity: Entity, delta: f32) -> bool {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_min_y_pos += delta;
    tile.transition_min_y_pos < 0.0
}

/// `-transitionOutTop:` @ 0x1000225bc.
pub fn transition_out_top(world: &mut World, entity: Entity, delta: f32) -> bool {
    let mut tile = world.obj_mut::<TileModel>(entity);
    tile.transition_max_y_pos += delta;
    tile.transition_max_y_pos < 0.0
}

/// `-startTransitionInTop`/`Bottom`, `-startTransitionOutTop`/`Bottom`, `-transitionIn/OutTop/Bottom:`
/// @ 0x10002235c … 0x1000225bc: the vertical analogues.
///
/// Sends one of the four vertical `startTransition…` selectors by name; any other selector is
/// unrecognised (an exception in the original).
pub fn start_transition_vertical(world: &mut World, entity: Entity, selector: &str) {
    match selector {
        "startTransitionInBottom" => start_transition_in_bottom(world, entity),
        "startTransitionOutBottom" => start_transition_out_bottom(world, entity),
        "startTransitionInTop" => start_transition_in_top(world, entity),
        "startTransitionOutTop" => start_transition_out_top(world, entity),
        _ => panic!("BCTileModel does not respond to `{selector}`"),
    }
}

/// Vertical `transition…:` steps (see [`start_transition_vertical`]); the trailing colon of
/// the selector is optional.
pub fn transition_vertical(world: &mut World, entity: Entity, selector: &str, delta: f32) -> bool {
    let name = selector.strip_suffix(':').unwrap_or(selector);
    match name {
        "transitionInBottom" => transition_in_bottom(world, entity, delta),
        "transitionOutBottom" => transition_out_bottom(world, entity, delta),
        "transitionInTop" => transition_in_top(world, entity, delta),
        "transitionOutTop" => transition_out_top(world, entity, delta),
        _ => panic!("BCTileModel does not respond to `{selector}`"),
    }
}

#[cfg(test)]
mod tests {
    use super::wrap;

    #[test]
    fn wrap_shifts_positive_positions_down_a_period() {
        assert_eq!(wrap(80.0, 600.0), -520.0);
        assert_eq!(wrap(600.0, 600.0), -600.0);
        assert_eq!(wrap(0.0, 600.0), 0.0);
        assert_eq!(wrap(-20.0, 600.0), -20.0);
        assert_eq!(wrap(-620.0, 600.0), -20.0);
    }

    #[test]
    fn wrap_by_zero_is_nan_like_fmod() {
        assert!(wrap(10.0, 0.0).is_nan());
        assert!(wrap(f32::NAN, 600.0).is_nan());
    }
}
