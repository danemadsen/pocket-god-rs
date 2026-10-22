//! Building display groups from the struct-XML data (`docs/re/struct-xml.md` §3–§6).
//!
//! `BCLibrary::parseStructXml:` walks a flat node list; `display-groups.json` lists the display
//! nodes of each call nested under the top-level group (node type 2) they follow. Every node
//! after a group attaches to it (an empty group never becomes the current group), empty
//! groups, groups and markers are registered in the library as they are created, and marker
//! links are resolved after the whole call, last marker first, against the library as filled
//! so far. A rect node (none in 1.48.2) also starts a new group in the original; the JSON lists
//! that group as the next entry, so it is built like any other group. Every object starts
//! hibernating.

use bevy_ecs::prelude::*;
use pg_assets::data::display_groups::{
    self, DisplayAttributes, GroupDef, NodeDef, NodeKind, ParseCall, TextJustification,
};

use crate::display::text::Justification;
use crate::display::{self, BC_DISPLAY_OBJECT, gradient, group, model, multi_model, rect, text, tile_model};
use crate::library::Library;

/// `BCLibrary::parseStructXml:` @ 0x100026674 for the display nodes of every `parse_calls`
/// entry of `display-groups.json`, in call order (loadGame's five calls).
pub fn build_display_groups(world: &mut World) {
    let file = display_groups::load().expect("assets/data/display-groups.json matches its schema");
    for call in &file.parse_calls {
        build_parse_call(world, call);
    }
}

/// The display nodes of one `parseStructXml:` call: nodes before the first group (no current
/// group), then each group and its children, then the deferred marker links in reverse order.
pub fn build_parse_call(world: &mut World, call: &ParseCall) {
    let mut pending_links: Vec<(Entity, String)> = Vec::new();

    for node in &call.orphans {
        build_node(world, None, node, &mut pending_links);
    }

    for group_def in &call.groups {
        let current = build_group(world, group_def);
        for node in &group_def.children {
            build_node(world, Some(current), node, &mut pending_links);
        }
    }

    for (marker, link_id) in pending_links.iter().rev() {
        let linked_group = world.resource::<Library>().display_group(link_id);
        group::add_display_object(world, *marker, linked_group);
    }
}

/// Node type 2: a new `BCDisplayGroup` that is registered and becomes the current group.
fn build_group(world: &mut World, group_def: &GroupDef) -> Entity {
    let id = group_def.attributes.id.as_str();
    let created = group::spawn_group(world, Some(id));
    parse_display_object_attributes(world, created, &group_def.attributes);
    add_display_group(world, id, created);
    created
}

/// One node of the `switch` in `parseStructXml:` (types 1, 3–9 and 11).
fn build_node(
    world: &mut World,
    current: Option<Entity>,
    node: &NodeDef,
    pending_links: &mut Vec<(Entity, String)>,
) {
    let id = node.attributes.id.as_str();

    match &node.kind {
        NodeKind::DisplayObject => {
            let created = display::spawn(world, &BC_DISPLAY_OBJECT, Some(id));
            parse_display_object_attributes(world, created, &node.attributes);
            add_child_and_release(world, current, created);
        }
        NodeKind::EmptyGroup => {
            let created = group::spawn_group(world, Some(id));
            parse_display_object_attributes(world, created, &node.attributes);
            if let Some(current) = current {
                group::add_display_object(world, current, Some(created));
            }
            add_display_group(world, id, created);
        }
        NodeKind::Marker { link_id } => {
            let created = group::spawn_marker(world, Some(id));
            parse_display_object_attributes(world, created, &node.attributes);
            if let Some(link_id) = link_id {
                pending_links.push((created, link_id.clone()));
            }
            if let Some(current) = current {
                group::add_display_object(world, current, Some(created));
            }
            add_display_marker(world, id, created);
        }
        NodeKind::Rect { rect: bounds } => {
            let [min_x, min_y, max_x, max_y] = *bounds;
            let created = rect::spawn(world, Some(id), min_x, min_y, max_x, max_y);
            parse_display_object_attributes(world, created, &node.attributes);
            add_child_and_release(world, current, created);
        }
        NodeKind::Model { texture } => {
            let texture_def = match texture {
                Some(texture_id) => world.resource::<Library>().texture_def_id(texture_id),
                None => None,
            };
            let created = model::spawn(world, Some(id), texture_def);
            parse_display_object_attributes(world, created, &node.attributes);
            add_child_and_release(world, current, created);
        }
        NodeKind::MultiModel => {
            let created = multi_model::spawn(world, Some(id));
            parse_display_object_attributes(world, created, &node.attributes);
            add_child_and_release(world, current, created);
        }
        NodeKind::TileModel { wrap_width, wrap_height } => {
            let created = tile_model::spawn(world, Some(id), *wrap_width, *wrap_height);
            parse_display_object_attributes(world, created, &node.attributes);
            add_child_and_release(world, current, created);
        }
        NodeKind::Gradient { gradient: gradient_id, width, height } => {
            let gradient_def = known_gradient_def(world, gradient_id.as_deref());
            let created = gradient::spawn(world, Some(id), *width, *height, gradient_def);
            parse_display_object_attributes(world, created, &node.attributes);
            add_child_and_release(world, current, created);
        }
        NodeKind::Text { font, text: initial_text, justification } => {
            let font_def = known_font_def(world, font.as_deref());
            let created = text::spawn(world, Some(id), font_def);
            parse_display_object_attributes(world, created, &node.attributes);

            // The node's center flag sets justification 2, then its right flag 1; the JSON
            // holds the result, and the setter only stores the value and marks the text dirty.
            match justification {
                TextJustification::Left => {}
                TextJustification::Center => {
                    text::set_font_justification(world, created, Justification::Center);
                }
                TextJustification::Right => {
                    text::set_font_justification(world, created, Justification::Right);
                }
            }

            text::set_text(world, created, initial_text.as_deref());
            add_child_and_release(world, current, created);
        }
    }
}

/// `[current addDisplayObject:object]; [object release]`: without a current group (a message
/// to nil) nothing else holds the object, so it is destroyed.
fn add_child_and_release(world: &mut World, current: Option<Entity>, object: Entity) {
    match current {
        Some(current) => group::add_display_object(world, current, Some(object)),
        None => {
            world.despawn(object);
        }
    }
}

/// `BCLibrary::addDisplayGroup:` @ 0x100023bd0 (a later group with the same id replaces it).
fn add_display_group(world: &mut World, id: &str, display_group: Entity) {
    let mut library = world.resource_mut::<Library>();
    library.display_groups.insert(id.to_owned(), display_group);
}

/// `BCLibrary::addDisplayMarker:` @ 0x100023c5c (a later marker with the same id replaces it).
fn add_display_marker(world: &mut World, id: &str, marker: Entity) {
    let mut library = world.resource_mut::<Library>();
    library.display_markers.insert(id.to_owned(), marker);
}

/// `getGradientDefById:` for the `BCGradient` initializer: unknown or missing ids give NULL.
fn known_gradient_def<'a>(world: &World, gradient_id: Option<&'a str>) -> Option<&'a str> {
    let gradient_id = gradient_id?;
    let library = world.resource::<Library>();
    library.gradient_def(gradient_id)?;
    Some(gradient_id)
}

/// `getFontDefById:` for the `BCText` initializer: unknown or missing ids give nil.
fn known_font_def<'a>(world: &World, font_id: Option<&'a str>) -> Option<&'a str> {
    let font_id = font_id?;
    let library = world.resource::<Library>();
    library.font_def(font_id)?;
    Some(font_id)
}

/// The packed `0xAARRGGBB` colour as passed to `setRed:green:blue:alpha:` (f32 divisions by
/// 255.0, the constant at 0x10029d43c).
fn unpack_color(packed: u32) -> [f32; 4] {
    let red = ((packed >> 16) & 0xff) as f32 / 255.0;
    let green = ((packed >> 8) & 0xff) as f32 / 255.0;
    let blue = (packed & 0xff) as f32 / 255.0;
    let alpha = (packed >> 24) as f32 / 255.0;
    [red, green, blue, alpha]
}

/// `BCLibrary::parseDisplayObjectAttributes:displayObject:` @ 0x10002626c: each member is
/// applied through its plain `BCDisplayObject` setter only when it differs from the default
/// (0, or 1 for scales and colour channels; a packed colour only when non-zero), in this order.
fn parse_display_object_attributes(world: &mut World, entity: Entity, attributes: &DisplayAttributes) {
    let [origin_x, origin_y, origin_z] = attributes.origin;
    let [x, y, z] = attributes.pos;
    let [scale_x, scale_y, scale_z] = attributes.scale;
    let [rot_x, rot_y, rot_z] = attributes.rot;
    let [skew_x, skew_y, skew_z] = attributes.skew;
    let [red, green, blue, alpha] = attributes.rgba;
    let [col_x_min, col_y_min, col_z_min] = attributes.col_min;
    let [col_x_max, col_y_max, col_z_max] = attributes.col_max;
    let packed_color = attributes.packed_color.unwrap_or(0);

    let mut object = display::object_mut(world, entity);

    if let Some(data) = &attributes.data {
        object.data = Some(data.clone());
    }

    if origin_x != 0.0 {
        object.origin_pos.x = origin_x;
    }
    if origin_y != 0.0 {
        object.origin_pos.y = origin_y;
    }
    if origin_z != 0.0 {
        object.origin_pos.z = origin_z;
    }

    if x != 0.0 {
        object.pos.x = x;
    }
    if y != 0.0 {
        object.pos.y = y;
    }
    if z != 0.0 {
        object.pos.z = z;
    }

    if scale_x != 1.0 {
        object.scale.x = scale_x;
    }
    if scale_y != 1.0 {
        object.scale.y = scale_y;
    }
    if scale_z != 1.0 {
        object.scale.z = scale_z;
    }

    if rot_x != 0.0 {
        object.rot.x = rot_x;
    }
    if rot_y != 0.0 {
        object.rot.y = rot_y;
    }
    if rot_z != 0.0 {
        object.rot.z = rot_z;
    }

    if skew_x != 0.0 {
        object.skew.x = skew_x;
    }
    if skew_y != 0.0 {
        object.skew.y = skew_y;
    }
    if skew_z != 0.0 {
        object.skew.z = skew_z;
    }

    if packed_color != 0 {
        let [packed_red, packed_green, packed_blue, packed_alpha] = unpack_color(packed_color);
        object.set_rgba(packed_red, packed_green, packed_blue, packed_alpha);
    }

    // setRed: / setGreen: / setBlue: / setAlpha: also set mbColor.
    if red != 1.0 {
        object.red = red;
        object.color = true;
    }
    if green != 1.0 {
        object.green = green;
        object.color = true;
    }
    if blue != 1.0 {
        object.blue = blue;
        object.color = true;
    }
    if alpha != 1.0 {
        object.set_alpha(alpha);
    }

    if attributes.hidden {
        object.hidden = true;
    }
    if attributes.selectable {
        object.is_selectable = true;
    }

    if col_x_min != 0.0 {
        object.col_x_min = col_x_min;
    }
    if col_x_max != 0.0 {
        object.col_x_max = col_x_max;
    }
    if col_y_min != 0.0 {
        object.col_y_min = col_y_min;
    }
    if col_y_max != 0.0 {
        object.col_y_max = col_y_max;
    }
    if col_z_min != 0.0 {
        object.col_z_min = col_z_min;
    }
    if col_z_max != 0.0 {
        object.col_z_max = col_z_max;
    }
}

#[cfg(test)]
mod tests {
    use bevy_math::Vec4;

    use super::*;
    use crate::display::DisplayObject;
    use crate::math::{VEC4_ONE, VEC4_ORIGIN};

    fn attributes(json: &str) -> DisplayAttributes {
        serde_json::from_str(json).expect("valid attributes")
    }

    const DEFAULTS: &str = r#"{"id": "Object", "data": null, "origin": [0.0, 0.0, 0.0],
        "pos": [0.0, 0.0, 0.0], "scale": [1.0, 1.0, 1.0], "rot": [0.0, 0.0, 0.0],
        "skew": [0.0, 0.0, 0.0], "packed_color": null, "rgba": [1.0, 1.0, 1.0, 1.0],
        "hidden": false, "selectable": false, "col_min": [0.0, 0.0, 0.0],
        "col_max": [0.0, 0.0, 0.0]}"#;

    fn spawn_plain(world: &mut World) -> Entity {
        display::spawn(world, &BC_DISPLAY_OBJECT, Some("Object"))
    }

    #[test]
    fn default_attributes_leave_the_initializer_state() {
        let mut world = World::new();
        let entity = spawn_plain(&mut world);
        parse_display_object_attributes(&mut world, entity, &attributes(DEFAULTS));

        let object = world.get::<DisplayObject>(entity).expect("display object");
        assert_eq!(object.pos, VEC4_ORIGIN);
        assert_eq!(object.scale, VEC4_ONE);
        assert_eq!(object.origin_pos, VEC4_ORIGIN);
        assert!(!object.color, "no colour setter ran");
        assert!(object.hibernate, "new objects hibernate");
        assert!(!object.hidden && !object.is_selectable);
        assert_eq!(object.data, None);
    }

    #[test]
    fn non_default_members_are_applied() {
        let json = r#"{"id": "Star", "data": "Meteor", "origin": [1.0, 2.0, 0.0],
            "pos": [291.5, 64.6, -277.12], "scale": [0.65, 0.65, 1.0], "rot": [0.0, 0.0, 9.03],
            "skew": [3.0, 0.0, 0.0], "packed_color": null, "rgba": [0.41, 1.0, -1.0, 0.5],
            "hidden": true, "selectable": true, "col_min": [-10.0, -20.0, 0.0],
            "col_max": [10.0, 20.0, 0.0]}"#;
        let mut world = World::new();
        let entity = spawn_plain(&mut world);
        parse_display_object_attributes(&mut world, entity, &attributes(json));

        let object = world.get::<DisplayObject>(entity).expect("display object");
        assert_eq!(object.data.as_deref(), Some("Meteor"));
        assert_eq!(object.origin_pos, Vec4::new(1.0, 2.0, 0.0, 1.0));
        assert_eq!(object.pos, Vec4::new(291.5, 64.6, -277.12, 1.0));
        assert_eq!(object.scale, Vec4::new(0.65, 0.65, 1.0, 1.0));
        assert_eq!(object.rot, Vec4::new(0.0, 0.0, 9.03, 1.0));
        assert_eq!(object.skew, Vec4::new(3.0, 0.0, 0.0, 1.0));
        assert_eq!([object.red, object.green, object.blue, object.alpha], [0.41, 1.0, -1.0, 0.5]);
        assert!(object.color);
        assert!(object.hidden && object.is_selectable);
        assert_eq!((object.col_x_min, object.col_x_max), (-10.0, 10.0));
        assert_eq!((object.col_y_min, object.col_y_max), (-20.0, 20.0));
    }

    /// `SunReflection`: the packed colour is applied, then channels that are not 1 override it.
    #[test]
    fn packed_colour_then_channel_overrides() {
        let packed_only = DEFAULTS.replace(r#""packed_color": null"#, r#""packed_color": 16777215"#);
        let mut world = World::new();
        let entity = spawn_plain(&mut world);
        parse_display_object_attributes(&mut world, entity, &attributes(&packed_only));
        let object = world.get::<DisplayObject>(entity).expect("display object");
        assert_eq!([object.red, object.green, object.blue, object.alpha], [1.0, 1.0, 1.0, 0.0]);
        assert!(object.color);

        let overridden = packed_only.replace(r#""rgba": [1.0, 1.0, 1.0, 1.0]"#, r#""rgba": [1.0, 0.5, 1.0, 0.25]"#);
        let entity = spawn_plain(&mut world);
        parse_display_object_attributes(&mut world, entity, &attributes(&overridden));
        let object = world.get::<DisplayObject>(entity).expect("display object");
        assert_eq!([object.red, object.green, object.blue, object.alpha], [1.0, 0.5, 1.0, 0.25]);
    }

    #[test]
    fn packed_colour_uses_f32_divisions() {
        let [red, green, blue, alpha] = unpack_color(0x80ff_4000);
        assert_eq!(red, 255.0f32 / 255.0);
        assert_eq!(green, 64.0f32 / 255.0);
        assert_eq!(blue, 0.0);
        assert_eq!(alpha, 128.0f32 / 255.0);
    }
}
