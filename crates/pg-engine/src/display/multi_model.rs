//! `BCMultiModel`: a model showing the current frame of a timeline animation
//! (`docs/re/rendering.md` §5.5, `docs/re/timelines.md` §5).

use bevy_ecs::prelude::*;

use crate::animation::{BcAnimation, FrameRef, Timelines, state_flags};
use crate::display::{self, DisplayObject, model};
use crate::gl::GlContext;
use crate::object::{Class, ObjectWorldExt};
use crate::texture::TextureDefId;

/// `BCMultiModel` ivars. `channels_of` is the animation player whose 30 tweenable channels
/// `mpTweenableChannelArray` points into.
#[derive(Component, Clone, Debug)]
pub struct MultiModel {
    pub frame: Option<FrameRef>,
    pub channels_of: Option<Entity>,
    pub model_indices: [u16; 4],
}

impl Default for MultiModel {
    fn default() -> Self {
        Self {
            frame: None,
            channels_of: None,
            model_indices: [0, 1, 2, 3],
        }
    }
}

pub const BC_MULTI_MODEL: Class = Class {
    name: "BCMultiModel",
    superclass: Some(&model::BC_MODEL),
    draw: draw_multi_model,
    draw_in_multi_model: draw_multi_model_in_multi_model,
    set_texture_def: set_multi_model_texture_def,
    ..model::BC_MODEL
};

/// `-[BCMultiModel initWithId:]` @ 0x100181694.
pub fn spawn(world: &mut World, id: Option<&str>) -> Entity {
    let entity = display::spawn(world, &BC_MULTI_MODEL, id);
    world.entity_mut(entity).insert((model::Model::default(), MultiModel::default()));
    entity
}

/// `-setBinaryFrameDef:tweenableChannelArray:` @ 0x100181950: animation mode.
pub fn set_binary_frame_def(world: &mut World, entity: Entity, frame: Option<FrameRef>, channels_of: Option<Entity>) {
    {
        let mut multi_model = world.obj_mut::<MultiModel>(entity);
        multi_model.frame = frame;
        multi_model.channels_of = channels_of;
    }
    world.obj_mut::<model::Model>(entity).texture_def = None;
}

/// `-binaryFrameDef` @ 0x10018197c.
pub fn binary_frame_def(world: &World, entity: Entity) -> Option<FrameRef> {
    world.obj::<MultiModel>(entity).frame
}

/// `-setTextureDef:` @ 0x100181924: static single-texture mode.
fn set_multi_model_texture_def(world: &mut World, entity: Entity, texture: Option<TextureDefId>) {
    world.obj_mut::<model::Model>(entity).texture_def = texture;
    let mut multi_model = world.obj_mut::<MultiModel>(entity);
    multi_model.frame = None;
    multi_model.channels_of = None;
}

/// The fields of a frame state record that the draws read (`BinaryFrameState`, timelines.md §2.8).
#[derive(Clone, Copy, Debug)]
pub(crate) struct FrameState {
    pub flags: u32,
    pub texture: Option<TextureDefId>,
    pub dynamic_text: Option<Entity>,
    pub channel: u8,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub alpha: f32,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub vertices: [f32; 12],
}

impl FrameState {
    fn has(&self, flag: u32) -> bool {
        self.flags & flag != 0
    }
}

/// The draw-ordered state indices of a frame (its `BinaryFrameStatePointer` list), copied so
/// that sub-object draws can take the world.
pub(crate) fn frame_state_indices(world: &World, frame: FrameRef) -> Vec<u32> {
    let timelines = world.resource::<Timelines>();
    timelines.frame_states(frame).to_vec()
}

/// A copy of the drawn fields of state record `index`.
pub(crate) fn frame_state(world: &World, index: u32) -> FrameState {
    let state = world.resource::<Timelines>().state(index);
    FrameState {
        flags: state.flags,
        texture: state.texture,
        dynamic_text: state.dynamic_text,
        channel: state.channel,
        x: state.x,
        y: state.y,
        rotation: state.rotation,
        scale_x: state.scale_x,
        scale_y: state.scale_y,
        alpha: state.alpha,
        red: state.red,
        green: state.green,
        blue: state.blue,
        vertices: state.vertices,
    }
}

/// `channels[channel].mTweenable` of the player `channels_of` (nil without a channel array).
fn channel_tweenable(world: &World, channels_of: Option<Entity>, channel: u8) -> Option<Entity> {
    let player = world.get::<BcAnimation>(channels_of?)?;
    let slot = player.channels.get(channel as usize)?;
    slot.tweenable
}

/// The object a state draws through: its dynamic text (`+0x28`), else its channel's tweenable.
/// A despawned entity is messaged like nil.
fn state_tweenable(world: &World, entity: Entity, state: &FrameState) -> Option<Entity> {
    let candidate = match state.dynamic_text {
        Some(text) => Some(text),
        None => {
            let channels_of = world.obj::<MultiModel>(entity).channels_of;
            channel_tweenable(world, channels_of, state.channel)
        }
    };
    candidate.filter(|object| world.get::<DisplayObject>(*object).is_some())
}

/// `BCMultiModel::drawOpenGL:` @ 0x10018198c.
fn draw_multi_model(world: &mut World, entity: Entity, gl: &mut GlContext) {
    let display = display::object(world, entity);
    if display.hidden || display.multi_model_mode {
        return;
    }

    gl.push_matrix();
    model::apply_model_transform(world, entity, gl);

    let texture = world.obj::<model::Model>(entity).texture_def;
    if texture.is_some() {
        let display = display::object(world, entity);
        let alpha = display.alpha;
        model::set_premultiplied_color(display, gl);
        model::draw_texture_def(world, texture, alpha, gl);
    } else if let Some(frame) = world.obj::<MultiModel>(entity).frame {
        let states = frame_state_indices(world, frame);
        for index in states {
            let state = frame_state(world, index);
            match state_tweenable(world, entity, &state) {
                None => draw_state_quad(world, entity, &state, gl),
                Some(child) => draw_state_child(world, entity, child, &state, gl),
            }
        }
    }

    gl.pop_matrix();
}

/// A top-level state without a sub-object: its own quad, tinted by the state and the model.
fn draw_state_quad(world: &World, entity: Entity, state: &FrameState, gl: &mut GlContext) {
    let Some(texture) = state.texture else {
        return;
    };
    let display = display::object(world, entity);
    let alpha = state.alpha * display.alpha;
    let red = state.red * display.red;
    let green = state.green * display.green;
    let blue = state.blue * display.blue;

    gl.set_color(alpha * red, alpha * green, alpha * blue, alpha);
    let blend = model::quad_blend(world, texture, alpha);
    model::draw_quad(world, texture, &state.vertices, blend, gl);
}

/// Where a state puts its sub-object: `translate(x, y, 0)`, `rotate(0, 0, z_rotation)`,
/// `scale(scale_x, scale_y, 1)`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ChildTransform {
    x: f32,
    y: f32,
    z_rotation: f32,
    scale_x: f32,
    scale_y: f32,
}

impl ChildTransform {
    /// The top-level rule: the state's values where its flags say so, else the inverse of
    /// the channel transform (timelines.md §4.4), so a sub-object is drawn where it stands.
    fn top_level(state: &FrameState, model: &DisplayObject, child: &DisplayObject) -> Self {
        let x = if state.has(state_flags::X) {
            state.x
        } else {
            (child.pos.x - model.pos.x) / model.scale.x - model.origin_pos.x
        };
        let y = if state.has(state_flags::Y) {
            state.y
        } else {
            (child.pos.y - model.pos.y) / model.scale.y - model.origin_pos.y
        };
        let z_rotation = if state.has(state_flags::ROTATION) {
            state.rotation
        } else if model.scale.x > 0.0 {
            child.rot.z - model.rot.z
        } else if state.has(state_flags::SCALE_X) {
            model.rot.z + (180.0 - child.rot.z)
        } else {
            model.rot.z - child.rot.z
        };
        let scale_x = if state.has(state_flags::SCALE_X) {
            state.scale_x
        } else {
            child.scale.x * model.scale.x
        };
        let scale_y = if state.has(state_flags::SCALE_Y) {
            state.scale_y
        } else {
            child.scale.y * model.scale.y
        };
        Self {
            x,
            y,
            z_rotation,
            scale_x,
            scale_y,
        }
    }

    /// The nested rule: the state's values, else the sub-object's own (no conversion).
    fn nested(state: &FrameState, child: &DisplayObject) -> Self {
        let x = if state.has(state_flags::X) { state.x } else { child.pos.x };
        let y = if state.has(state_flags::Y) { state.y } else { child.pos.y };
        let z_rotation = if state.has(state_flags::ROTATION) {
            state.rotation
        } else {
            child.rot.z
        };
        let scale_x = if state.has(state_flags::SCALE_X) {
            state.scale_x
        } else {
            child.scale.x
        };
        let scale_y = if state.has(state_flags::SCALE_Y) {
            state.scale_y
        } else {
            child.scale.y
        };
        Self {
            x,
            y,
            z_rotation,
            scale_x,
            scale_y,
        }
    }

    fn apply(&self, gl: &mut GlContext) {
        gl.translate(self.x, self.y, 0.0);
        gl.rotate(0.0, 0.0, self.z_rotation);
        gl.scale(self.scale_x, self.scale_y, 1.0);
    }
}

/// A top-level state drawn through a sub-object; the state's texture is handed to it first.
fn draw_state_child(world: &mut World, entity: Entity, child: Entity, state: &FrameState, gl: &mut GlContext) {
    gl.push_matrix();

    let model = display::object(world, entity);
    let child_display = display::object(world, child);
    let transform = ChildTransform::top_level(state, model, child_display);
    let alpha = state.alpha * model.alpha;
    let red = state.red * model.red;
    let green = state.green * model.green;
    let blue = state.blue * model.blue;

    if let Some(texture) = state.texture {
        display::set_texture_def(world, child, Some(texture));
    }
    gl.set_color(alpha * red, alpha * green, alpha * blue, alpha);
    transform.apply(gl);

    let rgba = [red, green, blue, alpha];
    display::draw_in_multi_model(world, child, gl, entity, rgba);
    gl.pop_matrix();
}

/// `BCMultiModel::drawInMultiModelOpenGL:…` @ 0x100181f60.
fn draw_multi_model_in_multi_model(world: &mut World, entity: Entity, gl: &mut GlContext, _parent: Entity, rgba: [f32; 4]) {
    let display = display::object(world, entity);
    if display.hidden {
        return;
    }
    let alpha = display.alpha;
    let origin = display.origin_pos;
    gl.translate(origin.x, origin.y, origin.z);

    let texture = world.obj::<model::Model>(entity).texture_def;
    if texture.is_some() {
        model::draw_texture_def(world, texture, alpha, gl);
        return;
    }
    let Some(frame) = world.obj::<MultiModel>(entity).frame else {
        return;
    };

    let states = frame_state_indices(world, frame);
    for index in states {
        let state = frame_state(world, index);
        if state.texture.is_some() {
            draw_nested_state_quad(world, entity, &state, rgba, gl);
            continue;
        }
        if let Some(child) = state_tweenable(world, entity, &state) {
            draw_nested_state_child(world, entity, child, &state, rgba, gl);
        }
    }
}

/// A nested state with a texture: its quad, tinted by the state, the model and the caller.
fn draw_nested_state_quad(world: &World, entity: Entity, state: &FrameState, rgba: [f32; 4], gl: &mut GlContext) {
    let Some(texture) = state.texture else {
        return;
    };
    let [red_in, green_in, blue_in, alpha_in] = rgba;
    let display = display::object(world, entity);
    let alpha = state.alpha * display.alpha * alpha_in;
    let red = state.red * display.red * red_in;
    let green = state.green * display.green * green_in;
    let blue = state.blue * display.blue * blue_in;

    gl.set_color(alpha * red, alpha * green, alpha * blue, alpha);
    let blend = model::quad_blend(world, texture, alpha);
    model::draw_quad(world, texture, &state.vertices, blend, gl);
}

/// A nested state drawn through a sub-object (no `setTextureDef:` on this path).
fn draw_nested_state_child(world: &mut World, entity: Entity, child: Entity, state: &FrameState, rgba: [f32; 4], gl: &mut GlContext) {
    gl.push_matrix();

    let child_display = display::object(world, child);
    let transform = ChildTransform::nested(state, child_display);
    let [red_in, green_in, blue_in, alpha_in] = rgba;
    let model = display::object(world, entity);
    let alpha = state.alpha * model.alpha * alpha_in;
    let red = state.red * model.red * red_in;
    let green = state.green * model.green * green_in;
    let blue = state.blue * model.blue * blue_in;

    gl.set_color(alpha * red, alpha * green, alpha * blue, alpha);
    transform.apply(gl);

    let child_rgba = [red, green, blue, alpha];
    display::draw_in_multi_model(world, child, gl, entity, child_rgba);
    gl.pop_matrix();
}

#[cfg(test)]
mod tests {
    use bevy_math::Vec4;

    use super::{ChildTransform, FrameState};
    use crate::animation::state_flags;
    use crate::display::DisplayObject;

    fn state(flags: u32) -> FrameState {
        FrameState {
            flags,
            texture: None,
            dynamic_text: None,
            channel: 1,
            x: 3.0,
            y: 4.0,
            rotation: 45.0,
            scale_x: 0.5,
            scale_y: 0.25,
            alpha: 1.0,
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            vertices: [0.0; 12],
        }
    }

    fn object(pos: Vec4, rotation: f32, scale: Vec4) -> DisplayObject {
        let mut display = DisplayObject::new(None);
        display.pos = pos;
        display.rot = Vec4::new(0.0, 0.0, rotation, 1.0);
        display.scale = scale;
        display
    }

    #[test]
    fn flagged_parts_come_from_the_state() {
        let flags = state_flags::X | state_flags::Y | state_flags::ROTATION | state_flags::SCALE_X | state_flags::SCALE_Y;
        let model = object(Vec4::new(1.0, 2.0, 0.0, 1.0), 10.0, Vec4::ONE);
        let child = object(Vec4::new(50.0, 60.0, 0.0, 1.0), 30.0, Vec4::new(2.0, 2.0, 1.0, 1.0));
        let expected = ChildTransform {
            x: 3.0,
            y: 4.0,
            z_rotation: 45.0,
            scale_x: 0.5,
            scale_y: 0.25,
        };
        assert_eq!(ChildTransform::top_level(&state(flags), &model, &child), expected);
        assert_eq!(ChildTransform::nested(&state(flags), &child), expected);
    }

    #[test]
    fn unflagged_parts_invert_the_channel_transform() {
        let mut model = object(Vec4::new(100.0, 50.0, 0.0, 1.0), 10.0, Vec4::new(2.0, 0.5, 1.0, 1.0));
        model.origin_pos = Vec4::new(5.0, -2.0, 0.0, 1.0);
        let child = object(Vec4::new(140.0, 70.0, 0.0, 1.0), 30.0, Vec4::new(3.0, 4.0, 1.0, 1.0));
        let transform = ChildTransform::top_level(&state(0), &model, &child);
        assert_eq!(transform.x, 15.0);
        assert_eq!(transform.y, 42.0);
        assert_eq!(transform.z_rotation, 20.0);
        assert_eq!(transform.scale_x, 6.0);
        assert_eq!(transform.scale_y, 2.0);
    }

    #[test]
    fn mirrored_models_pick_the_rotation_rule_by_the_scale_x_flag() {
        let model = object(Vec4::new(0.0, 0.0, 0.0, 1.0), 10.0, Vec4::new(-1.0, 1.0, 1.0, 1.0));
        let child = object(Vec4::new(0.0, 0.0, 0.0, 1.0), 30.0, Vec4::ONE);
        let with_flag = ChildTransform::top_level(&state(state_flags::SCALE_X), &model, &child);
        let without_flag = ChildTransform::top_level(&state(0), &model, &child);
        assert_eq!(with_flag.z_rotation, 10.0 + (180.0 - 30.0));
        assert_eq!(without_flag.z_rotation, 10.0 - 30.0);

        let mut unscaled = model.clone();
        unscaled.scale.x = f32::NAN;
        let nan_scale = ChildTransform::top_level(&state(0), &unscaled, &child);
        assert_eq!(nan_scale.z_rotation, 10.0 - 30.0);
    }

    #[test]
    fn nested_parts_use_the_objects_own_values() {
        let child = object(Vec4::new(7.0, 8.0, 0.0, 1.0), 30.0, Vec4::new(3.0, 4.0, 1.0, 1.0));
        let transform = ChildTransform::nested(&state(0), &child);
        let expected = ChildTransform {
            x: 7.0,
            y: 8.0,
            z_rotation: 30.0,
            scale_x: 3.0,
            scale_y: 4.0,
        };
        assert_eq!(transform, expected);
    }
}
