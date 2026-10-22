//! Draws of the drawable display classes through the GL state machine
//! (`docs/re/rendering.md` §5.3–§5.8, §7).

use bevy_ecs::prelude::*;
use bevy_math::{Mat4, Vec3, Vec4};
use pg_engine::animation::{BcAnimation, FrameRef, TimelineFrame, TimelineState, Timelines, state_flags};
use pg_engine::display::model::SPRITE_Z;
use pg_engine::display::{self, gradient, model, multi_model, rect, tile_model};
use pg_engine::draw::{BlendMode, DrawVertex, TextureId};
use pg_engine::gl::GlContext;
use pg_engine::library::{GradientColor, GradientDef, Library};
use pg_engine::texture::{AtlasDef, AtlasId, TextureDef, TextureDefId, Textures};

const QUAD: [f32; 12] = [0.0, 60.0, 0.0, 68.0, 60.0, 0.0, 0.0, 6.0, 0.0, 68.0, 6.0, 0.0];

fn new_world() -> World {
    let mut world = World::new();
    world.insert_resource(Library::default());
    world.insert_resource(Textures::default());
    world.insert_resource(Timelines::default());
    world
}

fn add_texture(world: &mut World, id: &str, blend: bool, vertices: [f32; 12]) -> TextureDefId {
    let mut textures = world.resource_mut::<Textures>();
    if textures.atlases.is_empty() {
        let atlas = AtlasDef {
            id: "atlas0".to_owned(),
            width: 64,
            height: 64,
            image: Some("TextureAtlas0".to_owned()),
            ..Default::default()
        };
        textures.add_atlas(atlas);
    }
    let def = TextureDef {
        id: id.to_owned(),
        atlas: Some(AtlasId(0)),
        blend,
        width: 68.0,
        height: 54.0,
        frame_width: 80.0,
        frame_height: 60.0,
        uvs: [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        vertices,
    };
    let texture = TextureDefId(textures.defs.len() as u32);
    textures.defs.push(def);
    texture
}

/// Clip x/y = eye x/y; eye z (`[-1000, 0]` in the tests) is mapped inside the clip volume.
fn test_gl() -> GlContext {
    let mut gl = GlContext::default();
    let depth = Mat4::from_scale(Vec3::new(1.0, 1.0, 0.001));
    let shift = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.5));
    gl.begin_frame(shift * depth, [0.0; 4]);
    gl
}

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1e-3, "{actual} != {expected}");
}

fn assert_at(vertex: &DrawVertex, x: f32, y: f32) {
    assert_close(vertex.position[0], x);
    assert_close(vertex.position[1], y);
}

fn place(world: &mut World, entity: Entity, x: f32, y: f32, z: f32) {
    display::object_mut(world, entity).pos = Vec4::new(x, y, z, 1.0);
}

fn state(texture: Option<TextureDefId>, channel: u8) -> TimelineState {
    TimelineState {
        name: "Root".to_owned(),
        flags: 0,
        texture,
        dynamic_text: None,
        channel,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        alpha: 1.0,
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        vertices: QUAD,
    }
}

/// Appends one frame made of `states` (in draw order) and returns it.
fn add_frame(world: &mut World, states: Vec<TimelineState>) -> FrameRef {
    let mut timelines = world.resource_mut::<Timelines>();
    let first_state = timelines.state_lists.len() as u32;
    let state_count = states.len() as u32;
    for state in states {
        let index = timelines.states.len() as u32;
        timelines.states.push(state);
        timelines.state_lists.push(index);
    }
    let frame = TimelineFrame {
        index: 0,
        sound: None,
        action: None,
        move_x: 0.0,
        move_y: 0.0,
        first_state,
        state_count,
    };
    let reference = FrameRef(timelines.frames.len() as u32);
    timelines.frames.push(frame);
    reference
}

#[test]
fn sprite_depth_constant_is_the_original_bit_pattern() {
    assert_eq!(SPRITE_Z.to_bits(), 0xc38a8f5c);
    assert_eq!(model::depth_factor(SPRITE_Z), 1.0);
}

#[test]
fn model_premultiplies_its_tint_and_blends_below_full_alpha() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "HalloweenCobweb", false, QUAD);
    let entity = model::spawn(&mut world, Some("Cobweb"), Some(texture));
    place(&mut world, entity, 10.0, 160.0, SPRITE_Z);
    display::object_mut(&mut world, entity).set_rgba(1.0, 0.5, 0.25, 0.5);

    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);

    let batch = &gl.list.batches[0];
    assert_eq!(batch.blend, BlendMode::Premultiplied);
    assert_eq!(batch.texture, Some(TextureId(1)));
    assert_eq!(batch.vertices.len(), 6);
    assert_eq!(batch.vertices[0].color, [0.5, 0.25, 0.125, 0.5]);
    assert_at(&batch.vertices[0], 10.0, 220.0);
    assert_at(&batch.vertices[1], 78.0, 220.0);
    assert_at(&batch.vertices[2], 10.0, 166.0);
    assert_at(&batch.vertices[5], 78.0, 166.0);
}

#[test]
fn opaque_texture_at_full_alpha_disables_blending() {
    let mut world = new_world();
    let opaque = add_texture(&mut world, "MenuBG", false, QUAD);
    let translucent = add_texture(&mut world, "Sun", true, QUAD);
    let first = model::spawn(&mut world, None, Some(opaque));
    let second = model::spawn(&mut world, None, Some(translucent));
    place(&mut world, first, 0.0, 160.0, SPRITE_Z);
    place(&mut world, second, 0.0, 160.0, SPRITE_Z);

    let mut gl = test_gl();
    display::draw(&mut world, first, &mut gl);
    display::draw(&mut world, second, &mut gl);

    assert_eq!(gl.list.batches[0].blend, BlendMode::Opaque);
    assert_eq!(gl.list.batches[1].blend, BlendMode::Premultiplied);
    assert!(gl.blend());
}

#[test]
fn deeper_models_shrink_toward_the_horizon() {
    let mut world = new_world();
    let quad = [10.0, 20.0, 0.0, 10.0, 20.0, 0.0, 10.0, 20.0, 0.0, 10.0, 20.0, 0.0];
    let texture = add_texture(&mut world, "Glacier", true, quad);
    let entity = model::spawn(&mut world, None, Some(texture));
    let depth = SPRITE_Z * 2.0;
    place(&mut world, entity, 100.0, 200.0, depth);

    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);

    // kz = 0.5: y = 0.5 · (200 − 160) + 160 = 180, the quad is scaled by 0.5.
    let vertex = &gl.list.batches[0].vertices[0];
    assert_at(vertex, 105.0, 190.0);
}

#[test]
fn pivot_is_the_fixed_point_of_rotation() {
    let mut world = new_world();
    let quad = [10.0, 0.0, 0.0, 20.0, 0.0, 0.0, 10.0, 0.0, 0.0, 20.0, 0.0, 0.0];
    let texture = add_texture(&mut world, "Arrow", true, quad);
    let entity = model::spawn(&mut world, None, Some(texture));
    place(&mut world, entity, 0.0, 160.0, SPRITE_Z);
    {
        let mut object = display::object_mut(&mut world, entity);
        object.pivot_pos = Vec4::new(10.0, 0.0, 0.0, 1.0);
        object.rot = Vec4::new(0.0, 0.0, 90.0, 1.0);
    }

    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);

    let vertices = &gl.list.batches[0].vertices;
    assert_at(&vertices[0], 10.0, 160.0);
    assert_at(&vertices[1], 10.0, 170.0);
}

#[test]
fn hidden_and_multi_model_mode_models_draw_nothing() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "Sun", true, QUAD);
    let hidden = model::spawn(&mut world, None, Some(texture));
    let bound = model::spawn(&mut world, None, Some(texture));
    place(&mut world, hidden, 0.0, 160.0, SPRITE_Z);
    place(&mut world, bound, 0.0, 160.0, SPRITE_Z);
    display::object_mut(&mut world, hidden).hidden = true;
    display::enable_multi_model_mode(&mut world, bound);
    display::object_mut(&mut world, hidden).set_rgba(0.0, 0.0, 0.0, 0.0);

    let mut gl = test_gl();
    display::draw(&mut world, hidden, &mut gl);
    display::draw(&mut world, bound, &mut gl);

    assert!(gl.list.batches.is_empty());
    assert_eq!(gl.color(), [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn nested_model_ignores_hidden_and_the_callers_colour() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "Sword", true, QUAD);
    let entity = model::spawn(&mut world, None, Some(texture));
    let parent = multi_model::spawn(&mut world, None);
    {
        let mut object = display::object_mut(&mut world, entity);
        object.hidden = true;
        object.origin_pos = Vec4::new(-40.0, -30.0, 0.0, 1.0);
        object.set_rgba(1.0, 1.0, 1.0, 0.5);
    }

    let mut gl = test_gl();
    display::draw_in_multi_model(&mut world, entity, &mut gl, parent, [0.1, 0.2, 0.3, 0.4]);

    let vertex = &gl.list.batches[0].vertices[0];
    assert_eq!(vertex.color, [0.5, 0.5, 0.5, 0.5]);
    assert_at(vertex, -40.0, 30.0);
}

#[test]
fn multi_model_draws_states_in_order_with_state_and_model_tint() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "Ant_01", false, QUAD);
    let mut tinted = state(Some(texture), 0);
    tinted.alpha = 0.5;
    tinted.red = 0.5;
    let invisible = state(None, 0);
    let opaque = state(Some(texture), 0);
    let frame = add_frame(&mut world, vec![tinted, invisible, opaque]);

    let entity = multi_model::spawn(&mut world, Some("Ant"));
    place(&mut world, entity, 0.0, 160.0, SPRITE_Z);
    display::object_mut(&mut world, entity).set_rgba(0.5, 1.0, 1.0, 1.0);
    multi_model::set_binary_frame_def(&mut world, entity, Some(frame), None);

    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);

    let batches = &gl.list.batches;
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].blend, BlendMode::Premultiplied);
    assert_eq!(batches[0].vertices[0].color, [0.125, 0.5, 0.5, 0.5]);
    assert_eq!(batches[1].blend, BlendMode::Opaque);
    assert_eq!(batches[1].vertices[0].color, [0.5, 1.0, 1.0, 1.0]);
}

#[test]
fn multi_model_draws_a_bound_channel_where_the_object_stands() {
    let mut world = new_world();
    let body = add_texture(&mut world, "Pygmy_Body", true, QUAD);
    let sword = add_texture(&mut world, "Sword", true, QUAD);
    let mut channel_state = state(Some(sword), 7);
    channel_state.alpha = 0.5;
    let frame = add_frame(&mut world, vec![channel_state]);

    let child = multi_model::spawn(&mut world, Some("Sword"));
    place(&mut world, child, 140.0, 70.0, SPRITE_Z);
    {
        let mut object = display::object_mut(&mut world, child);
        object.rot = Vec4::new(0.0, 0.0, 30.0, 1.0);
    }
    display::set_texture_def(&mut world, child, Some(body));

    let mut player = BcAnimation::default();
    player.channels[7].tweenable = Some(child);
    let animation = world.spawn(player).id();

    let entity = multi_model::spawn(&mut world, Some("Pygmy"));
    place(&mut world, entity, 100.0, 50.0, SPRITE_Z);
    {
        let mut object = display::object_mut(&mut world, entity);
        object.scale = Vec4::new(2.0, 1.0, 1.0, 1.0);
        object.origin_pos = Vec4::new(5.0, 0.0, 0.0, 1.0);
    }
    multi_model::set_binary_frame_def(&mut world, entity, Some(frame), Some(animation));

    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);

    // The state's texture was handed to the child (texture mode, frame cleared).
    assert_eq!(display::texture_def(&world, child), Some(sword));
    assert_eq!(multi_model::binary_frame_def(&world, child), None);

    // x = (140 − 100) / 2 − 5 = 15, y = 20, rz = 30, sx = 2, sy = 1.
    let model_position = Mat4::from_translation(Vec3::new(100.0, 50.0, SPRITE_Z));
    let model_scale = Mat4::from_scale(Vec3::new(2.0, 1.0, 1.0));
    let model_origin = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
    let child_position = Mat4::from_translation(Vec3::new(15.0, 20.0, 0.0));
    let child_rotation = Mat4::from_rotation_z(30.0_f32.to_radians());
    let child_scale = Mat4::from_scale(Vec3::new(2.0, 1.0, 1.0));
    let model_matrix = model_position * model_scale * model_origin;
    let child_matrix = child_position * child_rotation * child_scale;
    let corner = Vec4::new(0.0, 60.0, 0.0, 1.0);
    let expected = model_matrix * child_matrix * corner;

    let vertex = &gl.list.batches[0].vertices[0];
    assert_at(vertex, expected.x, expected.y);
    assert_eq!(vertex.color, [0.5, 0.5, 0.5, 0.5]);
}

#[test]
fn unbound_channel_falls_back_to_the_state_quad() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "Shield", true, QUAD);
    let frame = add_frame(&mut world, vec![state(Some(texture), 8), state(None, 9)]);
    let animation = world.spawn(BcAnimation::default()).id();

    let entity = multi_model::spawn(&mut world, None);
    place(&mut world, entity, 0.0, 160.0, SPRITE_Z);
    multi_model::set_binary_frame_def(&mut world, entity, Some(frame), Some(animation));

    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);

    assert_eq!(gl.list.batches.len(), 1);
    assert_eq!(gl.list.batches[0].vertices.len(), 6);
}

#[test]
fn nested_multi_model_draws_textured_states_itself_and_multiplies_colours() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "Bubble", false, QUAD);
    let other = add_texture(&mut world, "Wing", false, QUAD);
    let mut bubble = state(Some(texture), 3);
    bubble.alpha = 0.5;
    bubble.green = 0.5;
    let frame = add_frame(&mut world, vec![bubble]);

    let bound = model::spawn(&mut world, None, Some(other));
    let mut player = BcAnimation::default();
    player.channels[3].tweenable = Some(bound);
    let animation = world.spawn(player).id();

    let entity = multi_model::spawn(&mut world, None);
    multi_model::set_binary_frame_def(&mut world, entity, Some(frame), Some(animation));
    display::object_mut(&mut world, entity).origin_pos = Vec4::new(1.0, 2.0, 0.0, 1.0);
    let parent = multi_model::spawn(&mut world, None);

    let mut gl = test_gl();
    display::draw_in_multi_model(&mut world, entity, &mut gl, parent, [1.0, 1.0, 0.5, 0.5]);

    // Texture first: the channel object is neither drawn nor given the texture.
    assert_eq!(display::texture_def(&world, bound), Some(other));
    let batch = &gl.list.batches[0];
    assert_eq!(batch.vertices.len(), 6);
    assert_eq!(batch.blend, BlendMode::Premultiplied);
    assert_eq!(batch.vertices[0].color, [0.25, 0.125, 0.125, 0.25]);
    assert_at(&batch.vertices[0], 1.0, 62.0);
}

#[test]
fn nested_channel_state_places_the_object_at_the_state_and_passes_the_tint_on() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "Wing", true, QUAD);
    let mut wing = state(None, 2);
    wing.flags = state_flags::X | state_flags::Y;
    wing.x = 10.0;
    wing.y = 20.0;
    let frame = add_frame(&mut world, vec![wing]);

    let child = multi_model::spawn(&mut world, Some("Wing"));
    place(&mut world, child, 999.0, 999.0, SPRITE_Z);
    display::set_texture_def(&mut world, child, Some(texture));
    let mut player = BcAnimation::default();
    player.channels[2].tweenable = Some(child);
    let animation = world.spawn(player).id();

    let entity = multi_model::spawn(&mut world, Some("Bee"));
    multi_model::set_binary_frame_def(&mut world, entity, Some(frame), Some(animation));
    let parent = multi_model::spawn(&mut world, None);

    let mut gl = test_gl();
    display::draw_in_multi_model(&mut world, entity, &mut gl, parent, [1.0, 0.5, 1.0, 0.5]);

    let vertex = &gl.list.batches[0].vertices[0];
    assert_eq!(vertex.color, [0.5, 0.25, 0.5, 0.5]);
    assert_at(vertex, 10.0, 80.0);
}

#[test]
fn horizontal_tile_wraps_by_the_ancestor_offset_and_culls_tiles() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "Cloud", true, QUAD);
    let mut tiles = Vec::new();
    for left in [0.0, 200.0, 400.0] {
        let mut tile = state(Some(texture), 0);
        let right = left + 200.0;
        tile.vertices = [left, 100.0, 0.0, right, 100.0, 0.0, left, 0.0, 0.0, right, 0.0, 0.0];
        tiles.push(tile);
    }
    let frame = add_frame(&mut world, tiles);

    let root = display::spawn(&mut world, &display::BC_DISPLAY_OBJECT, Some("Root"));
    let group = display::spawn(&mut world, &display::BC_DISPLAY_OBJECT, Some("Sky"));
    place(&mut world, root, -160.0, 240.0, 0.0);
    place(&mut world, group, 30.0, 0.0, 0.0);
    display::object_mut(&mut world, group).display_group = Some(root);

    let entity = tile_model::spawn(&mut world, Some("Clouds"), 600.0, 0.0);
    place(&mut world, entity, 50.0, 7.0, SPRITE_Z);
    display::object_mut(&mut world, entity).display_group = Some(group);
    multi_model::set_binary_frame_def(&mut world, entity, Some(frame), None);

    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);

    // offset 30, X = 80, m = fmod(80, 600) − 600 = −520; visible x ∈ [520, 1000]: only the
    // third tile, translated by m − offset = −550.
    let batches = &gl.list.batches;
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].vertices.len(), 6);
    assert_at(&batches[0].vertices[0], -150.0, 107.0);
}

#[test]
fn vertical_tile_uses_the_parent_y_and_the_bottom_and_top_edges() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "Bubbles", true, QUAD);
    let mut tiles = Vec::new();
    for bottom in [0.0, 300.0] {
        let mut tile = state(Some(texture), 0);
        let top = bottom + 300.0;
        tile.vertices = [0.0, top, 0.0, 50.0, top, 0.0, 0.0, bottom, 0.0, 50.0, bottom, 0.0];
        tile.alpha = 0.5;
        tiles.push(tile);
    }
    let frame = add_frame(&mut world, tiles);

    let parent = display::spawn(&mut world, &display::BC_DISPLAY_OBJECT, Some("Water"));
    place(&mut world, parent, 0.0, 20.0, 0.0);
    let entity = tile_model::spawn(&mut world, None, 0.0, 600.0);
    place(&mut world, entity, 5.0, 10.0, SPRITE_Z);
    display::object_mut(&mut world, entity).display_group = Some(parent);
    multi_model::set_binary_frame_def(&mut world, entity, Some(frame), None);

    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);

    // Y = 30 → m = 30 − 600 = −570; visible y ∈ [570, 890]: only the upper tile, moved by
    // m − parent y = −590.
    let batches = &gl.list.batches;
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].vertices.len(), 6);
    assert_at(&batches[0].vertices[0], 5.0, 10.0);
    assert_eq!(batches[0].vertices[0].color, [0.5, 0.5, 0.5, 0.5]);
}

#[test]
fn tile_transitions_follow_the_original_limits() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "StormCloud", true, QUAD);
    let mut tiles = Vec::new();
    for left in [100.0, 300.0, 520.0] {
        let mut tile = state(Some(texture), 0);
        let right = left + 150.0;
        tile.vertices = [left, 100.0, 0.0, right, 100.0, 0.0, left, 0.0, 0.0, right, 0.0, 0.0];
        tiles.push(tile);
    }
    let frame = add_frame(&mut world, tiles);
    let entity = tile_model::spawn(&mut world, None, 600.0, 0.0);
    place(&mut world, entity, -20.0, 0.0, SPRITE_Z);
    multi_model::set_binary_frame_def(&mut world, entity, Some(frame), None);

    // m = −20: the limit starts at 500; the first tile reaches past x = 20 and starts left of
    // the limit, lowering it to 100, so the window opens at x = −20 + 100 = 80.
    tile_model::start_transition_in_left(&mut world, entity);
    let tile = world.get::<tile_model::TileModel>(entity).unwrap().clone();
    assert_eq!(tile.transition_max_x_pos, 80.0);
    assert_eq!(tile.transition_min_x_pos, 0.0);
    assert!(!tile_model::transition_in_left(&mut world, entity, 400.0));
    assert!(tile_model::transition_in_left(&mut world, entity, 1.0));

    // The right edge: the limit starts at 20; only the third tile starts beyond x = 500.
    tile_model::start_transition_in_right(&mut world, entity);
    let tile = world.get::<tile_model::TileModel>(entity).unwrap().clone();
    assert_eq!(tile.transition_min_x_pos, 500.0);
    assert_eq!(tile.transition_max_x_pos, 480.0);
    assert!(!tile_model::transition_in_right(&mut world, entity, -500.0));
    assert!(tile_model::transition_in_right(&mut world, entity, -1.0));

    tile_model::start_transition_out_bottom(&mut world, entity);
    let tile = world.get::<tile_model::TileModel>(entity).unwrap().clone();
    assert_eq!(tile.transition_max_y_pos, 480.0);
    assert_eq!(tile.transition_min_y_pos, 0.0);

    tile_model::start_transition_vertical(&mut world, entity, "startTransitionOutTop");
    let tile = world.get::<tile_model::TileModel>(entity).unwrap().clone();
    assert_eq!(tile.transition_max_y_pos, 320.0);
    assert!(tile_model::transition_vertical(&mut world, entity, "transitionOutTop:", -321.0));

    tile_model::reset_transition_invisible(&mut world, entity);
    let tile = world.get::<tile_model::TileModel>(entity).unwrap().clone();
    assert_eq!(tile.transition_max_x_pos, -1000.0);
    assert_eq!(tile.transition_min_x_pos, 1000.0);
}

#[test]
fn rect_draws_untextured_without_a_hidden_check() {
    let mut world = new_world();
    let texture = add_texture(&mut world, "Label", true, QUAD);
    let background = rect::spawn(&mut world, Some("NameBackground"), -50.0, 0.0, 50.0, 14.0);
    place(&mut world, background, 100.0, 160.0, SPRITE_Z);
    {
        let mut object = display::object_mut(&mut world, background);
        object.set_rgba(0.0, 0.0, 0.0, 0.5);
        object.hidden = true;
    }
    rect::set_max_x_pos(&mut world, background, 60.0);
    let label = model::spawn(&mut world, None, Some(texture));
    place(&mut world, label, 0.0, 160.0, SPRITE_Z);

    let mut gl = test_gl();
    display::draw(&mut world, background, &mut gl);
    display::draw(&mut world, label, &mut gl);

    let batches = &gl.list.batches;
    assert_eq!(batches[0].texture, None);
    assert_eq!(batches[0].blend, BlendMode::Premultiplied);
    assert_eq!(batches[0].vertices[0].color, [0.0, 0.0, 0.0, 0.5]);
    assert_at(&batches[0].vertices[0], 50.0, 174.0);
    assert_at(&batches[0].vertices[1], 160.0, 174.0);
    assert_eq!(batches[1].texture, Some(TextureId(1)));
}

fn ramp(stops: &[(f32, f32)]) -> GradientDef {
    let mut colors = Vec::new();
    for (pos, value) in stops {
        let color = GradientColor {
            pos: *pos,
            r: *value,
            g: *value,
            b: *value,
            a: 1.0,
        };
        colors.push(color);
    }
    GradientDef { colors }
}

#[test]
fn gradient_definitions_parse_like_sscanf() {
    let def = gradient::create_gradient_def("1.0:0xff1F1221,0.05:0xf5FDEFE,junk");
    assert_eq!(def.colors.len(), 3);
    assert_eq!(def.colors[0].pos, 1.0);
    assert_eq!(def.colors[0].r, 31.0 / 255.0);
    assert_eq!(def.colors[0].a, 1.0);
    assert_eq!(def.colors[1].pos, 0.05);
    assert_eq!(def.colors[1].a, 15.0 / 255.0);
    assert_eq!(def.colors[1].r, 95.0 / 255.0);
    assert_eq!(def.colors[1].b, 254.0 / 255.0);
    assert_eq!(def.colors[2].pos, 0.0);
    assert_eq!(def.colors[2].a, 0.0);
}

#[test]
fn gradient_set_and_fade_rewrite_the_shared_def_and_the_mesh() {
    let mut world = new_world();
    let island = ramp(&[(1.0, 0.0), (0.0, 0.0)]);
    let day = ramp(&[(1.0, 0.2), (0.5, 0.4)]);
    let night = ramp(&[(0.0, 1.0), (0.0, 0.0)]);
    {
        let mut library = world.resource_mut::<Library>();
        library.gradient_defs.insert("IslandGradient".to_owned(), island);
        library.gradient_defs.insert("DayGradient".to_owned(), day);
        library.gradient_defs.insert("NightGradient".to_owned(), night);
    }
    let entity = gradient::spawn(&mut world, Some("Gradient"), 480.0, 100.0, Some("IslandGradient"));
    let created = world.get::<gradient::Gradient>(entity).unwrap().clone();
    let expected_vertices = vec![[0.0, 100.0, 0.0], [480.0, 100.0, 0.0], [0.0, 0.0, 0.0], [480.0, 0.0, 0.0]];
    assert_eq!(created.vertices, expected_vertices);

    gradient::set_gradient(&mut world, entity, "DayGradient");
    gradient::fade_gradient(&mut world, entity, "NightGradient", 0.5);

    let shared = world.resource::<Library>().gradient_def("IslandGradient").unwrap().clone();
    assert_eq!(shared.colors[0].pos, 0.5);
    assert_eq!(shared.colors[0].r, 0.6);
    assert_eq!(shared.colors[1].pos, 0.25);
    assert_eq!(shared.colors[1].r, 0.2);
    let gradient = world.get::<gradient::Gradient>(entity).unwrap();
    assert_eq!(gradient.def, shared);
    assert_eq!(gradient.vertices[2], [0.0, 25.0, 0.0]);
    assert_eq!(gradient.colors[3], [0.2, 0.2, 0.2, 1.0]);
    let color = gradient::gradient_color_at_index(&world, entity, 0);
    assert_eq!(color.r, 0.6);
}

#[test]
fn gradient_draws_vertex_colours_with_the_inherited_blend_state() {
    let mut world = new_world();
    let space = ramp(&[(1.0, 0.25), (0.0, 0.75)]);
    world
        .resource_mut::<Library>()
        .gradient_defs
        .insert("SpaceGradient".to_owned(), space);
    let entity = gradient::spawn(&mut world, None, 480.0, 187.0, Some("SpaceGradient"));
    place(&mut world, entity, 0.0, 149.0, SPRITE_Z);
    display::object_mut(&mut world, entity).set_alpha(0.0);

    let mut gl = test_gl();
    gl.set_blend(false);
    display::draw(&mut world, entity, &mut gl);

    let batch = &gl.list.batches[0];
    assert_eq!(batch.texture, None);
    assert_eq!(batch.blend, BlendMode::Opaque);
    assert_eq!(batch.vertices[0].color, [0.25, 0.25, 0.25, 1.0]);
    assert_at(&batch.vertices[0], 0.0, 336.0);
    assert_at(&batch.vertices[2], 0.0, 149.0);

    display::object_mut(&mut world, entity).hidden = true;
    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);
    assert!(gl.list.batches.is_empty());
    let parent = multi_model::spawn(&mut world, None);
    display::draw_in_multi_model(&mut world, entity, &mut gl, parent, [1.0; 4]);
    assert_eq!(gl.list.batches[0].blend, BlendMode::Premultiplied);
}

#[test]
fn unknown_gradient_def_has_no_geometry() {
    let mut world = new_world();
    let entity = gradient::spawn(&mut world, None, 480.0, 100.0, Some("MissingGradient"));
    let gradient = world.get::<gradient::Gradient>(entity).unwrap();
    assert_eq!(gradient.def_id, None);
    assert!(gradient.vertices.is_empty());
}
