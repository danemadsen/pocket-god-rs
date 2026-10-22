//! `BCDisplayObject` and its subclasses (`docs/re/rendering.md` §4–§5, `docs/re/struct-xml.md` §6).
//!
//! Display objects are entities carrying [`DisplayObject`] (the base ivars) plus one component
//! per subclass. The display hierarchy is the original's: a group's `children` array and a
//! weak `display_group` parent pointer. It is not a Bevy hierarchy: one group can sit in
//! several markers' child arrays at once, exactly as in the original.
//!
//! Plain accessors of the original (`pos`, `setXPos:`, `alpha`, `setIsHidden:` …) are field
//! accesses on [`DisplayObject`]; the functions here are the methods with behaviour.

pub mod build;
pub mod gradient;
pub mod group;
pub mod model;
pub mod multi_model;
pub mod rect;
pub mod text;
pub mod tile_model;

use std::f32::consts::{FRAC_PI_2, PI};

use bevy_ecs::prelude::*;
use bevy_math::Vec4;

use crate::gl::GlContext;
use crate::math::{VEC4_ONE, VEC4_ORIGIN, deg_to_rad};
use crate::object::{Class, Isa, ObjectWorldExt, isa, try_isa};
use crate::view::{BcView, frustum_center_x_pos, frustum_center_y_pos};

/// `BCDisplayObject` ivars (defaults from `initWithId:` @ 0x10010dfc0).
#[derive(Component, Clone, Debug)]
pub struct DisplayObject {
    pub id: Option<String>,
    pub data: Option<String>,

    /// `mDisplayGroup`: the parent (weak).
    pub display_group: Option<Entity>,

    /// `mLogic`: the attached logic object (weak).
    pub logic: Option<Entity>,
    pub debug: bool,
    pub hidden: bool,
    pub is_selectable: bool,
    pub hibernate: bool,
    pub hibernate_ref_count: i32,
    pub pos: Vec4,
    pub rot: Vec4,
    pub skew: Vec4,
    pub scale: Vec4,
    pub pivot_pos: Vec4,
    pub origin_pos: Vec4,

    /// `mbColor`: set by every colour setter, never read.
    pub color: bool,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,

    /// `setMatrixA:b:c:d:` (never read).
    pub matrix: [f32; 4],
    pub min_pos: Vec4,
    pub max_pos: Vec4,
    pub col_x_min: f32,
    pub col_x_max: f32,
    pub col_y_min: f32,
    pub col_y_max: f32,
    pub col_z_min: f32,
    pub col_z_max: f32,

    /// `mbMultiModelMode`: drawn by a parent `BCMultiModel` instead of in its own place.
    pub multi_model_mode: bool,
}

impl DisplayObject {
    pub fn new(id: Option<&str>) -> Self {
        Self {
            id: id.map(str::to_owned),
            data: None,
            display_group: None,
            logic: None,
            debug: false,
            hidden: false,
            is_selectable: false,
            hibernate: true,
            hibernate_ref_count: 0,
            pos: VEC4_ORIGIN,
            rot: VEC4_ORIGIN,
            skew: VEC4_ORIGIN,
            scale: VEC4_ONE,
            pivot_pos: VEC4_ORIGIN,
            origin_pos: VEC4_ORIGIN,
            color: false,
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
            matrix: [1.0, 0.0, 0.0, 1.0],
            min_pos: VEC4_ORIGIN,
            max_pos: VEC4_ORIGIN,
            col_x_min: 0.0,
            col_x_max: 0.0,
            col_y_min: 0.0,
            col_y_max: 0.0,
            col_z_min: 0.0,
            col_z_max: 0.0,
            multi_model_mode: false,
        }
    }

    /// `-setRed:green:blue:alpha:` (sets `mbColor`).
    pub fn set_rgba(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.red = red;
        self.green = green;
        self.blue = blue;
        self.alpha = alpha;
        self.color = true;
    }

    /// `-setAlpha:` (sets `mbColor`).
    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
        self.color = true;
    }
}

pub const BC_DISPLAY_OBJECT: Class = Class {
    name: "BCDisplayObject",
    superclass: None,
    hibernate: hibernate_object,
    unhibernate: unhibernate_object,
    add_selectable_logics: add_selectable_logics_object,
    ..Class::NS_OBJECT
};

/// `-initWithId:` @ 0x10010dfc0 on a fresh entity of the given class.
pub fn spawn(world: &mut World, class: &'static Class, id: Option<&str>) -> Entity {
    let object = DisplayObject::new(id);
    world.spawn((Isa(class), object)).id()
}

/// The display object component of an entity.
pub fn object(world: &World, entity: Entity) -> &DisplayObject {
    world.obj::<DisplayObject>(entity)
}

/// The display object component of an entity, mutably.
pub fn object_mut(world: &mut World, entity: Entity) -> Mut<'_, DisplayObject> {
    world.obj_mut::<DisplayObject>(entity)
}

/// `-id`.
pub fn id(world: &World, entity: Entity) -> Option<String> {
    object(world, entity).id.clone()
}

/// `-setLogic:` @ 0x10010e520: store (weak) and immediately hibernate / unhibernate the logic.
pub fn set_logic(world: &mut World, entity: Entity, logic: Option<Entity>) {
    let hibernating = {
        let mut display = object_mut(world, entity);
        display.logic = logic;
        display.hibernate
    };
    let Some(logic) = logic else {
        return;
    };
    if hibernating {
        crate::logic::hibernate(world, logic);
    } else {
        crate::logic::unhibernate(world, logic);
    }
}

/// `-hibernate` (dispatched: groups recurse into children).
pub fn hibernate(world: &mut World, entity: Entity) {
    let Some(class) = try_isa(world, entity) else {
        return;
    };
    (class.hibernate)(world, entity);
}

/// `-unhibernate` (dispatched: groups recurse into children).
pub fn unhibernate(world: &mut World, entity: Entity) {
    let Some(class) = try_isa(world, entity) else {
        return;
    };
    (class.unhibernate)(world, entity);
}

/// `BCDisplayObject::hibernate` @ 0x10010e454 (ref-counted, no clamping). Returns whether
/// the object went to sleep on this call (groups then recurse).
pub fn hibernate_object_counted(world: &mut World, entity: Entity) -> bool {
    let (went_to_sleep, logic) = {
        let mut display = object_mut(world, entity);
        display.hibernate_ref_count -= 1;
        let went_to_sleep = display.hibernate_ref_count == 0;
        if went_to_sleep {
            display.hibernate = true;
        }
        (went_to_sleep, display.logic)
    };
    if went_to_sleep && let Some(logic) = logic {
        crate::logic::hibernate(world, logic);
    }
    went_to_sleep
}

/// `BCDisplayObject::unhibernate` @ 0x10010e49c. Returns whether the object woke up on this
/// call. The count is incremented after the logic woke up, from its value at that point.
pub fn unhibernate_object_counted(world: &mut World, entity: Entity) -> bool {
    let (woke_up, logic) = {
        let mut display = object_mut(world, entity);
        let woke_up = display.hibernate_ref_count == 0;
        if woke_up {
            display.hibernate = false;
        }
        (woke_up, display.logic)
    };
    if woke_up && let Some(logic) = logic {
        crate::logic::unhibernate(world, logic);
    }
    object_mut(world, entity).hibernate_ref_count += 1;
    woke_up
}

fn hibernate_object(world: &mut World, entity: Entity) {
    hibernate_object_counted(world, entity);
}

fn unhibernate_object(world: &mut World, entity: Entity) {
    unhibernate_object_counted(world, entity);
}

/// `BCDisplayObject::addSelectableLogicsToArray:` @ 0x10010e3dc.
fn add_selectable_logics_object(world: &World, entity: Entity, logics: &mut Vec<Entity>) {
    let display = object(world, entity);
    if !display.is_selectable {
        return;
    }
    if let Some(logic) = display.logic {
        logics.push(logic);
    }
}

/// `-addSelectableLogicsToArray:` (dispatched).
pub fn add_selectable_logics(world: &World, entity: Entity, logics: &mut Vec<Entity>) {
    let class = isa(world, entity);
    (class.add_selectable_logics)(world, entity, logics);
}

/// `BCDisplayObject::addLogicsToArray:` @ 0x10010e438: the logic, selectable or not.
pub fn add_logics_object(world: &World, entity: Entity, logics: &mut Vec<Entity>) {
    if let Some(logic) = object(world, entity).logic {
        logics.push(logic);
    }
}

/// `-drawOpenGL:` (dispatched).
pub fn draw(world: &mut World, entity: Entity, gl: &mut GlContext) {
    let Some(class) = try_isa(world, entity) else {
        return;
    };
    (class.draw)(world, entity, gl);
}

/// `-drawInMultiModelOpenGL:red:green:blue:alpha:` (dispatched).
pub fn draw_in_multi_model(world: &mut World, entity: Entity, gl: &mut GlContext, parent: Entity, rgba: [f32; 4]) {
    let Some(class) = try_isa(world, entity) else {
        return;
    };
    (class.draw_in_multi_model)(world, entity, gl, parent, rgba);
}

/// `-setTextureDef:` (dispatched; a no-op on the base class).
pub fn set_texture_def(world: &mut World, entity: Entity, texture: Option<crate::texture::TextureDefId>) {
    let Some(class) = try_isa(world, entity) else {
        return;
    };
    (class.set_texture_def)(world, entity, texture);
}

/// `-textureDef` (dispatched; NULL on the base class).
pub fn texture_def(world: &World, entity: Entity) -> Option<crate::texture::TextureDefId> {
    let class = try_isa(world, entity)?;
    (class.texture_def)(world, entity)
}

/// `-enableMultiModelMode` (dispatched; only `BCModel` and subclasses react).
pub fn enable_multi_model_mode(world: &mut World, entity: Entity) {
    let Some(class) = try_isa(world, entity) else {
        return;
    };
    (class.enable_multi_model_mode)(world, entity);
}

/// `-disableMultiModelMode` (dispatched).
pub fn disable_multi_model_mode(world: &mut World, entity: Entity) {
    let Some(class) = try_isa(world, entity) else {
        return;
    };
    (class.disable_multi_model_mode)(world, entity);
}

/// `-removeFromDisplayGroup` @ 0x10010e574.
pub fn remove_from_display_group(world: &mut World, entity: Entity) {
    let parent = object(world, entity).display_group;
    if let Some(parent) = parent {
        group::remove_display_object(world, parent, Some(entity));
    }
}

/// `-moveToDisplayGroup:` @ 0x10010e684: remove, then `addDisplayObject:`.
pub fn move_to_display_group(world: &mut World, entity: Entity, group: Option<Entity>) {
    remove_from_display_group(world, entity);
    if let Some(group) = group {
        group::add_display_object(world, group, Some(entity));
    }
}

/// `-moveToDisplayGroup:atIndex:` @ 0x10010e6c8.
pub fn move_to_display_group_at_index(world: &mut World, entity: Entity, group: Option<Entity>, index: i32) {
    remove_from_display_group(world, entity);
    if let Some(group) = group {
        group::add_display_object_at_index(world, group, Some(entity), index);
    }
}

/// `-insertBefore:` @ 0x10010e598: put `other` into this object's group just before it. A nil
/// `other` still reaches `addDisplayObject:atIndex:` (a marker parent then empties its slot).
pub fn insert_before(world: &mut World, entity: Entity, other: Option<Entity>) {
    if let Some(other) = other {
        remove_from_display_group(world, other);
    }
    let Some(parent) = object(world, entity).display_group else {
        return;
    };
    let index = group::index_of_display_object(world, parent, entity);
    group::add_display_object_at_index(world, parent, other, index);
}

/// `-insertAfter:` @ 0x10010e60c: put `other` into this object's group just after it. A nil
/// `other` still reaches `addDisplayObject:atIndex:` (a marker parent then empties its slot).
pub fn insert_after(world: &mut World, entity: Entity, other: Option<Entity>) {
    if let Some(other) = other {
        remove_from_display_group(world, other);
    }
    let Some(parent) = object(world, entity).display_group else {
        return;
    };
    let index = group::index_of_display_object(world, parent, entity);
    group::add_display_object_at_index(world, parent, other, index + 1);
}

/// `-setOriginPosInPlace:` @ 0x10010eb10: moves `pos` so the geometry stays put (each axis
/// mirrored by the sign of its own scale), then stores all four origin components.
pub fn set_origin_pos_in_place(world: &mut World, entity: Entity, origin: Vec4) {
    let mut display = object_mut(world, entity);
    let mut delta_x = origin.x - display.origin_pos.x;
    let mut delta_y = origin.y - display.origin_pos.y;
    let mut delta_z = origin.z - display.origin_pos.z;
    if display.scale.x < 0.0 {
        delta_x = -delta_x;
    }
    if display.scale.y < 0.0 {
        delta_y = -delta_y;
    }
    if display.scale.z < 0.0 {
        delta_z = -delta_z;
    }

    display.pos.x -= delta_x;
    display.pos.y -= delta_y;
    display.pos.z -= delta_z;
    display.origin_pos = origin;
}

/// `-setOriginXPosInPlace:` @ 0x10010ec08.
pub fn set_origin_x_pos_in_place(world: &mut World, entity: Entity, value: f32) {
    let mut display = object_mut(world, entity);
    let delta = value - display.origin_pos.x;
    let pos = display.pos.x;
    display.pos.x = if 0.0 < display.scale.x { pos - delta } else { delta + pos };
    display.origin_pos.x = value;
}

/// `-setOriginYPosInPlace:` @ 0x10010ec74.
pub fn set_origin_y_pos_in_place(world: &mut World, entity: Entity, value: f32) {
    let mut display = object_mut(world, entity);
    let delta = value - display.origin_pos.y;
    let pos = display.pos.y;
    display.pos.y = if 0.0 < display.scale.y { pos - delta } else { delta + pos };
    display.origin_pos.y = value;
}

/// `-setOriginZPosInPlace:` @ 0x10010ecec (tests scale.y — original bug, keep).
pub fn set_origin_z_pos_in_place(world: &mut World, entity: Entity, value: f32) {
    let mut display = object_mut(world, entity);
    let delta = value - display.origin_pos.z;
    let pos = display.pos.z;
    display.pos.z = if 0.0 < display.scale.y { pos - delta } else { delta + pos };
    display.origin_pos.z = value;
}

/// `-minPos` @ 0x10010ee78 (also stored in `min_pos`; z and w are copied from `pos`).
pub fn min_pos(world: &mut World, entity: Entity) -> Vec4 {
    let mut display = object_mut(world, entity);
    let mut min = display.pos;
    min.x = display.origin_pos.x * display.scale.x + display.pos.x;
    min.y += display.origin_pos.y * display.scale.y;

    let col_x = if 0.0 < display.scale.x { display.col_x_min } else { display.col_x_max };
    min.x += display.scale.x * col_x;
    let col_y = if 0.0 < display.scale.y { display.col_y_min } else { display.col_y_max };
    min.y += display.scale.y * col_y;

    display.min_pos = min;
    min
}

/// `-maxPos` @ 0x10010ef58 (also stored in `max_pos`; z and w are copied from `pos`).
pub fn max_pos(world: &mut World, entity: Entity) -> Vec4 {
    let mut display = object_mut(world, entity);
    let mut max = display.pos;
    max.x = display.origin_pos.x * display.scale.x + display.pos.x;
    max.y += display.origin_pos.y * display.scale.y;

    let col_x = if 0.0 < display.scale.x { display.col_x_max } else { display.col_x_min };
    max.x += display.scale.x * col_x;
    let col_y = if 0.0 < display.scale.y { display.col_y_max } else { display.col_y_min };
    max.y += display.scale.y * col_y;

    display.max_pos = max;
    max
}

/// `-rotatedMinPos` @ 0x10010f038: the min corner rotated by `rot.z` about `pos` (stored in
/// `min_pos` with z = 0; w keeps whatever `min_pos` held).
pub fn rotated_min_pos(world: &mut World, entity: Entity) -> Vec4 {
    let mut display = object_mut(world, entity);
    let mut min = display.min_pos;
    min.x = display.origin_pos.x * display.scale.x;
    min.y = display.origin_pos.y * display.scale.y;
    min.z = 0.0;

    let col_x = if 0.0 < display.scale.x { display.col_x_min } else { display.col_x_max };
    min.x += display.scale.x * col_x;
    let col_y = if 0.0 < display.scale.y { display.col_y_min } else { display.col_y_max };
    min.y += display.scale.y * col_y;

    let angle = deg_to_rad(display.rot.z);
    let (sin, cos) = angle.sin_cos();
    let rotated_x = cos * min.x - sin * min.y + display.pos.x;
    let rotated_y = sin * min.x + cos * min.y + display.pos.y;
    min.x = rotated_x;
    min.y = rotated_y;

    display.min_pos = min;
    min
}

/// `-rotatedMaxPos` @ 0x10010f17c: the max corner rotated by `rot.z` about `pos` (stored in
/// `max_pos` with z = 0; w keeps whatever `max_pos` held).
pub fn rotated_max_pos(world: &mut World, entity: Entity) -> Vec4 {
    let mut display = object_mut(world, entity);
    let mut max = display.max_pos;
    max.x = display.origin_pos.x * display.scale.x;
    max.y = display.origin_pos.y * display.scale.y;
    max.z = 0.0;

    let col_x = if 0.0 < display.scale.x { display.col_x_max } else { display.col_x_min };
    max.x += display.scale.x * col_x;
    let col_y = if 0.0 < display.scale.y { display.col_y_max } else { display.col_y_min };
    max.y += display.scale.y * col_y;

    let angle = deg_to_rad(display.rot.z);
    let (sin, cos) = angle.sin_cos();
    let rotated_x = cos * max.x - sin * max.y + display.pos.x;
    let rotated_y = sin * max.x + cos * max.y + display.pos.y;
    max.x = rotated_x;
    max.y = rotated_y;

    display.max_pos = max;
    max
}

/// The original's `MATRIX`: 16 floats, column-major (element `column * 4 + row`).
type Matrix = [f32; 16];

/// `FUN_1001a1184`: the identity matrix.
fn matrix_identity() -> Matrix {
    let mut matrix = [0.0; 16];
    matrix[0] = 1.0;
    matrix[5] = 1.0;
    matrix[10] = 1.0;
    matrix[15] = 1.0;
    matrix
}

/// `FUN_1001a1414`: a translation matrix.
fn matrix_translation(x: f32, y: f32, z: f32) -> Matrix {
    let mut matrix = matrix_identity();
    matrix[12] = x;
    matrix[13] = y;
    matrix[14] = z;
    matrix
}

/// `FUN_1001a1454`: the rotation about x by `-angle` radians (`Rx(angle)` transposed).
fn matrix_rotation_x(angle: f32) -> Matrix {
    let (sin, cos) = angle.sin_cos();
    let mut matrix = matrix_identity();
    matrix[5] = cos;
    matrix[6] = -sin;
    matrix[9] = sin;
    matrix[10] = cos;
    matrix
}

/// `FUN_1001a14b4`: the rotation about y by `-angle` radians (`Ry(angle)` transposed).
fn matrix_rotation_y(angle: f32) -> Matrix {
    let (sin, cos) = angle.sin_cos();
    let mut matrix = matrix_identity();
    matrix[0] = cos;
    matrix[2] = sin;
    matrix[8] = -sin;
    matrix[10] = cos;
    matrix
}

/// `FUN_1001a151c`: the rotation about z by `-angle` radians (`Rz(angle)` transposed).
fn matrix_rotation_z(angle: f32) -> Matrix {
    let (sin, cos) = angle.sin_cos();
    let mut matrix = matrix_identity();
    matrix[0] = cos;
    matrix[1] = -sin;
    matrix[4] = sin;
    matrix[5] = cos;
    matrix
}

/// `FUN_1001a11bc(out, a, b)`: `out = b · a`, each element summed left to right.
fn matrix_multiply(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            let first = a[column * 4] * b[row];
            let second = a[column * 4 + 1] * b[4 + row];
            let third = a[column * 4 + 2] * b[8 + row];
            let fourth = a[column * 4 + 3] * b[12 + row];
            out[column * 4 + row] = first + second + third + fourth;
        }
    }
    out
}

/// `FUN_100007b20(out, v, m)`: `out = m · v`, each component summed left to right.
fn matrix_transform(matrix: &Matrix, vector: [f32; 4]) -> [f32; 4] {
    let mut out = [0.0; 4];
    for row in 0..4 {
        let first = matrix[row] * vector[0];
        let second = matrix[4 + row] * vector[1];
        let third = matrix[8 + row] * vector[2];
        let fourth = matrix[12 + row] * vector[3];
        out[row] = first + second + third + fourth;
    }
    out
}

/// `-createViewToDisplayObjectMatrix:` @ 0x10010f400: the parent's matrix (identity at the
/// root) followed by `Rz(-r.z)·Ry(-r.y)·Rx(-r.x)·T(-p)`, the inverse of the GL transform
/// without scale.
fn create_view_to_display_object_matrix(world: &World, entity: Entity) -> Matrix {
    let display = object(world, entity);
    let parent_matrix = match display.display_group {
        Some(parent) => create_view_to_display_object_matrix(world, parent),
        None => matrix_identity(),
    };

    let angle_x = deg_to_rad(display.rot.x);
    let angle_y = deg_to_rad(display.rot.y);
    let angle_z = deg_to_rad(display.rot.z);
    let rotation_x = matrix_rotation_x(angle_x);
    let rotation_y = matrix_rotation_y(angle_y);
    let rotation_z = matrix_rotation_z(angle_z);
    let translation = matrix_translation(-display.pos.x, -display.pos.y, -display.pos.z);

    let translated = matrix_multiply(&parent_matrix, &translation);
    let rotated_x = matrix_multiply(&translated, &rotation_x);
    let rotated_y = matrix_multiply(&rotated_x, &rotation_y);
    matrix_multiply(&rotated_y, &rotation_z)
}

/// `-createDisplayObjectToViewMatrix:` @ 0x10010f528: the parent's matrix followed by
/// `T(p)·Rz(r.z)·Ry(r.y)·Rx(r.x)` (scale ignored).
fn create_display_object_to_view_matrix(world: &World, entity: Entity) -> Matrix {
    let display = object(world, entity);
    let parent_matrix = match display.display_group {
        Some(parent) => create_display_object_to_view_matrix(world, parent),
        None => matrix_identity(),
    };

    // The angles are negated here (`rot · -π / 180`) and negated again by the builders.
    let angle_x = display.rot.x * -PI / 180.0;
    let angle_y = display.rot.y * -PI / 180.0;
    let angle_z = display.rot.z * -PI / 180.0;
    let rotation_x = matrix_rotation_x(angle_x);
    let rotation_y = matrix_rotation_y(angle_y);
    let rotation_z = matrix_rotation_z(angle_z);
    let translation = matrix_translation(display.pos.x, display.pos.y, display.pos.z);

    let rotation_yx = matrix_multiply(&rotation_x, &rotation_y);
    let rotation_zyx = matrix_multiply(&rotation_yx, &rotation_z);
    let local = matrix_multiply(&rotation_zyx, &translation);
    matrix_multiply(&local, &parent_matrix)
}

/// `-viewToDisplayObject:` @ 0x10010f664: a view-space (touch) point into this object's space.
pub fn view_to_display_object(world: &World, entity: Entity, view_point: Vec4) -> Vec4 {
    let view = world.resource::<BcView>();
    let frustum_center_x = frustum_center_x_pos(view);
    let frustum_center_y = frustum_center_y_pos(view);
    let screen_width = view.screen_width;
    let screen_height = view.screen_height;
    let touch_width = view.touch_width;
    let touch_height = view.touch_height;

    let screen_x = (screen_width / touch_width) * view_point.x - frustum_center_x;
    let screen_y = (screen_height / touch_height) * (touch_height - view_point.y) - frustum_center_y;
    let screen_point = [screen_x, screen_y, 0.0, 1.0];

    let screen_rotation = matrix_rotation_z(FRAC_PI_2);
    let object_matrix = create_view_to_display_object_matrix(world, entity);
    let matrix = matrix_multiply(&screen_rotation, &object_matrix);

    let local = matrix_transform(&matrix, screen_point);
    Vec4::from_array(local)
}

/// `-displayObjectToView:` @ 0x10010f800: a point of this object's space into view points
/// (only x and y of `point` are used).
pub fn display_object_to_view(world: &World, entity: Entity, point: Vec4) -> Vec4 {
    let view = world.resource::<BcView>();
    let frustum_center_x = frustum_center_x_pos(view);
    let frustum_center_y = frustum_center_y_pos(view);
    let screen_width = view.screen_width;
    let screen_height = view.screen_height;
    let touch_width = view.touch_width;
    let touch_height = view.touch_height;

    let object_point = [point.x, point.y, 0.0, 1.0];
    let screen_rotation = matrix_rotation_z(-FRAC_PI_2);
    let object_matrix = create_display_object_to_view_matrix(world, entity);
    let matrix = matrix_multiply(&object_matrix, &screen_rotation);
    let screen = matrix_transform(&matrix, object_point);

    let view_x = (touch_width / screen_width) * (frustum_center_x + screen[0]);
    let view_y = touch_height - (touch_height / screen_height) * (frustum_center_y + screen[1]);
    Vec4::new(view_x, view_y, screen[2], screen[3])
}

/// `-doesPointIntersect:checkSelect:debug:` @ 0x10010f9f0 (strict box test in unscaled units).
pub fn does_point_intersect(world: &World, entity: Entity, view_point: Vec4, check_select: bool) -> bool {
    let display = object(world, entity);
    if !display.is_selectable && check_select {
        return false;
    }

    let local = view_to_display_object(world, entity, view_point);

    let origin = display.origin_pos;
    let (x_min, x_max) = if display.scale.x < 0.0 {
        (-origin.x - display.col_x_max, -origin.x - display.col_x_min)
    } else {
        (origin.x + display.col_x_min, origin.x + display.col_x_max)
    };
    let (y_min, y_max) = if display.scale.y < 0.0 {
        (-origin.y - display.col_y_max, -origin.y - display.col_y_min)
    } else {
        (origin.y + display.col_y_min, origin.y + display.col_y_max)
    };

    let inside_x = x_min < local.x && local.x < x_max;
    let inside_y = y_min < local.y && local.y < y_max;
    inside_x && inside_y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix_from_glam(matrix: bevy_math::Mat4) -> Matrix {
        matrix.to_cols_array()
    }

    #[test]
    fn rotation_helpers_build_the_inverse_rotations() {
        let angle = 0.7_f32;
        let expected_x = matrix_from_glam(bevy_math::Mat4::from_rotation_x(-angle));
        let expected_y = matrix_from_glam(bevy_math::Mat4::from_rotation_y(-angle));
        let expected_z = matrix_from_glam(bevy_math::Mat4::from_rotation_z(-angle));
        let pairs = [
            (matrix_rotation_x(angle), expected_x),
            (matrix_rotation_y(angle), expected_y),
            (matrix_rotation_z(angle), expected_z),
        ];
        for (actual, expected) in pairs {
            for index in 0..16 {
                let difference = (actual[index] - expected[index]).abs();
                assert!(difference < 1e-6, "element {index}: {} vs {}", actual[index], expected[index]);
            }
        }
    }

    #[test]
    fn multiply_puts_the_second_operand_on_the_left() {
        let translation = matrix_translation(1.0, 2.0, 3.0);
        let rotation = matrix_rotation_z(-FRAC_PI_2);
        let product = matrix_multiply(&translation, &rotation);
        let expected = bevy_math::Mat4::from_cols_array(&rotation) * bevy_math::Mat4::from_cols_array(&translation);
        let expected = expected.to_cols_array();
        for index in 0..16 {
            assert!((product[index] - expected[index]).abs() < 1e-6, "element {index}");
        }

        let point = matrix_transform(&product, [0.0, 0.0, 0.0, 1.0]);
        assert!((point[0] + 2.0).abs() < 1e-5 && (point[1] - 1.0).abs() < 1e-5, "{point:?}");
        assert_eq!(point[2], 3.0);
    }
}
