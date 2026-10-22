//! `BCDisplayGroup` and `BCDisplayMarker` (`docs/re/rendering.md` §4.3–§4.4, §5.2).

use bevy_ecs::prelude::*;

use crate::display::{self, BC_DISPLAY_OBJECT};
use crate::gl::GlContext;
use crate::object::{Class, ObjectWorldExt};

/// `BCDisplayGroup` ivars: `mDisplayObjectArray` (order = draw order = touch priority).
#[derive(Component, Clone, Debug, Default)]
pub struct DisplayGroup {
    pub children: Vec<Entity>,
}

/// Marks a `BCDisplayMarker` (a group holding at most one child).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct DisplayMarker;

pub const BC_DISPLAY_GROUP: Class = Class {
    name: "BCDisplayGroup",
    superclass: Some(&BC_DISPLAY_OBJECT),
    draw: draw_group,
    draw_in_multi_model: draw_group_in_multi_model,
    hibernate: hibernate_group,
    unhibernate: unhibernate_group,
    add_selectable_logics: add_selectable_logics_group,
    ..BC_DISPLAY_OBJECT
};

pub const BC_DISPLAY_MARKER: Class = Class {
    name: "BCDisplayMarker",
    superclass: Some(&BC_DISPLAY_GROUP),
    draw: draw_marker,
    ..BC_DISPLAY_GROUP
};

/// `-[BCDisplayGroup initWithId:]` @ 0x100066f6c.
pub fn spawn_group(world: &mut World, id: Option<&str>) -> Entity {
    let entity = display::spawn(world, &BC_DISPLAY_GROUP, id);
    world.entity_mut(entity).insert(DisplayGroup::default());
    entity
}

/// `-[BCDisplayMarker initWithId:]` @ 0x10018bb48.
pub fn spawn_marker(world: &mut World, id: Option<&str>) -> Entity {
    let entity = display::spawn(world, &BC_DISPLAY_MARKER, id);
    world.entity_mut(entity).insert((DisplayGroup::default(), DisplayMarker));
    entity
}

pub fn children(world: &World, group: Entity) -> &[Entity] {
    &world.obj::<DisplayGroup>(group).children
}

fn children_mut(world: &mut World, group: Entity) -> Mut<'_, DisplayGroup> {
    world.obj_mut::<DisplayGroup>(group)
}

fn is_marker(world: &World, group: Entity) -> bool {
    world.get::<DisplayMarker>(group).is_some()
}

fn is_hibernating(world: &World, group: Entity) -> bool {
    display::object(world, group).hibernate
}

/// `-setDisplayGroup:` @ 0x10010e500 (messaging nil does nothing).
fn set_display_group(world: &mut World, child: Option<Entity>, group: Option<Entity>) {
    if let Some(child) = child {
        display::object_mut(world, child).display_group = group;
    }
}

/// `-addDisplayObject:` @ 0x100067544 (markers: @ 0x10018bd68). Does not detach the child
/// from a previous parent.
pub fn add_display_object(world: &mut World, group: Entity, child: Option<Entity>) {
    if is_marker(world, group) {
        add_display_object_to_marker(world, group, child);
        return;
    }

    let Some(child) = child else {
        return;
    };
    set_display_group(world, Some(child), Some(group));
    children_mut(world, group).children.push(child);
    if !is_hibernating(world, group) {
        display::unhibernate(world, child);
    }
}

/// `BCDisplayMarker::addDisplayObject:` @ 0x10018bd68: the current child (if any) is removed
/// first; nil just clears the slot.
fn add_display_object_to_marker(world: &mut World, marker: Entity, child: Option<Entity>) {
    let current = children(world, marker).first().copied();
    if let Some(current) = current {
        remove_display_object(world, marker, Some(current));
    }

    set_display_group(world, child, Some(marker));
    if let Some(child) = child {
        children_mut(world, marker).children.push(child);
    }
    if is_hibernating(world, marker) {
        return;
    }
    if let Some(child) = child {
        display::unhibernate(world, child);
    }
}

/// `-addDisplayObject:atIndex:` @ 0x1000675d0 (markers ignore the index, @ 0x10018be4c).
pub fn add_display_object_at_index(world: &mut World, group: Entity, child: Option<Entity>, index: i32) {
    if is_marker(world, group) {
        add_display_object_to_marker(world, group, child);
        return;
    }

    let Some(child) = child else {
        return;
    };
    set_display_group(world, Some(child), Some(group));

    let count = children(world, group).len() as i32;
    if index < count && count != 0 {
        // `insertObject:atIndex:` raises NSRangeException for a negative index.
        let Ok(position) = usize::try_from(index) else {
            panic!("-[NSMutableArray insertObject:atIndex:]: index {index} beyond bounds [0 .. {count}]");
        };
        children_mut(world, group).children.insert(position, child);
    } else {
        children_mut(world, group).children.push(child);
    }

    if !is_hibernating(world, group) {
        display::unhibernate(world, child);
    }
}

/// `-removeDisplayObject:` @ 0x1000676ac (`removeObject:` drops every occurrence).
pub fn remove_display_object(world: &mut World, group: Entity, child: Option<Entity>) {
    let Some(child) = child else {
        return;
    };
    if !is_hibernating(world, group) {
        display::hibernate(world, child);
    }
    set_display_group(world, Some(child), None);
    children_mut(world, group).children.retain(|entry| *entry != child);
}

/// `-removeDisplayObjectAtIndex:` @ 0x100067738: an index past the end removes the last child.
pub fn remove_display_object_at_index(world: &mut World, group: Entity, index: i32) {
    let child = {
        let children = children(world, group);
        let count = children.len() as i32;
        if index < count {
            // `objectAtIndex:` raises NSRangeException for a negative index.
            let Ok(position) = usize::try_from(index) else {
                panic!("-[NSArray objectAtIndex:]: index {index} beyond bounds [0 .. {count}]");
            };
            Some(children[position])
        } else if count < 1 {
            None
        } else {
            children.last().copied()
        }
    };

    if !is_hibernating(world, group) && let Some(child) = child {
        display::hibernate(world, child);
    }
    set_display_group(world, child, None);
    if let Some(child) = child {
        children_mut(world, group).children.retain(|entry| *entry != child);
    }
}

/// `-getDisplayObject:` @ 0x10006781c: first direct child with that id.
pub fn get_display_object(world: &World, group: Entity, id: &str) -> Option<Entity> {
    for child in children(world, group) {
        let child_id = display::object(world, *child).id.as_deref();
        if child_id == Some(id) {
            return Some(*child);
        }
    }
    None
}

/// `-getDisplayObjectAtIndex:` @ 0x100067964.
pub fn get_display_object_at_index(world: &World, group: Entity, index: i32) -> Option<Entity> {
    let position = usize::try_from(index).ok()?;
    children(world, group).get(position).copied()
}

/// `-getDisplayObjectIndex:` @ 0x1000679d0 (−1 if none).
pub fn get_display_object_index(world: &World, group: Entity, id: &str) -> i32 {
    for (position, child) in children(world, group).iter().enumerate() {
        let child_id = display::object(world, *child).id.as_deref();
        if child_id == Some(id) {
            return position as i32;
        }
    }
    -1
}

/// `-getDisplayObjectIndexByDisplayObject:` @ 0x100067b1c (−1 if none: NSNotFound truncated).
pub fn index_of_display_object(world: &World, group: Entity, child: Entity) -> i32 {
    let position = children(world, group).iter().position(|entry| *entry == child);
    match position {
        Some(position) => position as i32,
        None => -1,
    }
}

/// `-displayObjectCount` @ 0x100067518.
pub fn display_object_count(world: &World, group: Entity) -> i32 {
    children(world, group).len() as i32
}

/// `-addLogicsToArray:` (dispatched): `BCDisplayGroup` @ 0x1000672c8 adds its own logic only
/// when selectable (a copy/paste of `addSelectableLogicsToArray:`), then recurses into every
/// child; a plain object (@ 0x10010e438) adds its logic whether selectable or not.
pub fn add_logics_to_array(world: &World, group: Entity, logics: &mut Vec<Entity>) {
    if world.get::<DisplayGroup>(group).is_none() {
        display::add_logics_object(world, group, logics);
        return;
    }

    let display = display::object(world, group);
    if display.is_selectable && let Some(logic) = display.logic {
        logics.push(logic);
    }
    for child in children(world, group) {
        add_logics_to_array(world, *child, logics);
    }
}

/// `BCDisplayGroup::hibernate` @ 0x10006738c: counted; on the 1 → 0 transition the logic and
/// then every child hibernate.
fn hibernate_group(world: &mut World, group: Entity) {
    let went_to_sleep = display::hibernate_object_counted(world, group);
    if !went_to_sleep {
        return;
    }

    let children = children(world, group).to_vec();
    for child in children {
        display::hibernate(world, child);
    }
}

/// `BCDisplayGroup::unhibernate` @ 0x10006744c: on the 0 → 1 transition the logic and then
/// every child wake up; the count is incremented last.
fn unhibernate_group(world: &mut World, group: Entity) {
    let (woke_up, logic) = {
        let mut display = display::object_mut(world, group);
        let woke_up = display.hibernate_ref_count == 0;
        if woke_up {
            display.hibernate = false;
        }
        (woke_up, display.logic)
    };

    if woke_up {
        if let Some(logic) = logic {
            crate::logic::unhibernate(world, logic);
        }
        let children = children(world, group).to_vec();
        for child in children {
            display::unhibernate(world, child);
        }
    }

    display::object_mut(world, group).hibernate_ref_count += 1;
}

/// `BCDisplayGroup::addSelectableLogicsToArray:` @ 0x100067204: own logic if selectable, then
/// the children in order (pre-order; hidden objects included).
fn add_selectable_logics_group(world: &World, group: Entity, logics: &mut Vec<Entity>) {
    let display = display::object(world, group);
    if display.is_selectable && let Some(logic) = display.logic {
        logics.push(logic);
    }
    for child in children(world, group) {
        display::add_selectable_logics(world, *child, logics);
    }
}

/// `BCDisplayGroup::drawOpenGL:` @ 0x100067cf8.
fn draw_group(world: &mut World, group: Entity, gl: &mut GlContext) {
    let (hidden, parent, pos, rot, scale) = {
        let display = display::object(world, group);
        (display.hidden, display.display_group, display.pos, display.rot, display.scale)
    };
    if hidden {
        return;
    }
    let children = children(world, group).to_vec();
    if children.is_empty() {
        return;
    }

    if parent.is_none() {
        gl.load_identity();
    }
    gl.push_matrix();
    gl.translate(pos.x, pos.y, pos.z);
    gl.rotate(rot.x, rot.y, rot.z);
    gl.scale(scale.x, scale.y, scale.z);

    for child in children {
        display::draw(world, child, gl);
    }

    gl.pop_matrix();
}

/// `BCDisplayGroup::drawInMultiModelOpenGL:…` @ 0x100067eb4: every child's `drawOpenGL:`,
/// with no transform, hidden check or colour.
fn draw_group_in_multi_model(world: &mut World, group: Entity, gl: &mut GlContext, _parent: Entity, _rgba: [f32; 4]) {
    let children = children(world, group).to_vec();
    for child in children {
        display::draw(world, child, gl);
    }
}

/// `BCDisplayMarker::drawOpenGL:` @ 0x10018be9c: the child only (the marker's own transform,
/// colour and hidden flag are not applied).
fn draw_marker(world: &mut World, marker: Entity, gl: &mut GlContext) {
    let child = children(world, marker).first().copied();
    if let Some(child) = child {
        display::draw(world, child, gl);
    }
}
