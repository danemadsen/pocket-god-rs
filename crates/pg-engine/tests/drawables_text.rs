//! `BCFontDef` layout and `BCText` drawing (`docs/re/rendering.md` §5.7, §6).

use bevy_ecs::prelude::*;
use bevy_math::{Mat4, Vec3, Vec4};
use pg_engine::display::model::SPRITE_Z;
use pg_engine::display::text::{self, FontDef, Justification, Text};
use pg_engine::display::{self, multi_model};
use pg_engine::draw::BlendMode;
use pg_engine::gl::GlContext;
use pg_engine::library::Library;
use pg_engine::texture::{AtlasDef, AtlasId, TextureDef, TextureDefId, Textures};

const FONT: &str = "Test";
const LINE_HEIGHT: f32 = 12.0;

/// A world whose font `Test` maps `' '`, `a`, `b` (advance 10) and `c` (advance 10.5).
fn new_world() -> World {
    let mut world = World::new();
    world.insert_resource(Library::default());
    world.insert_resource(Textures::default());

    let advances = [10.0, 10.0, 10.0, 10.5];
    for (index, advance) in advances.iter().enumerate() {
        let glyph_id = format!("{FONT}:FullSet:{index}");
        let texture = add_glyph(&mut world, &glyph_id, *advance);
        world.resource_mut::<Library>().texture_def_ids.insert(glyph_id, texture);
    }
    let mut font = FontDef::new(FONT);
    font.set_char_set(" abc", world.resource::<Library>());
    world.resource_mut::<Library>().font_defs.insert(FONT.to_owned(), font);
    world
}

fn add_glyph(world: &mut World, id: &str, advance: f32) -> TextureDefId {
    let mut textures = world.resource_mut::<Textures>();
    if textures.atlases.is_empty() {
        let atlas = AtlasDef {
            id: "atlasTest0".to_owned(),
            width: 512,
            height: 512,
            image: Some("TextureAtlasTest0".to_owned()),
            ..Default::default()
        };
        textures.add_atlas(atlas);
    }
    let def = TextureDef {
        id: id.to_owned(),
        atlas: Some(AtlasId(0)),
        blend: false,
        width: advance,
        height: LINE_HEIGHT,
        frame_width: advance,
        frame_height: LINE_HEIGHT,
        uvs: [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        vertices: [0.0, LINE_HEIGHT, 0.0, advance, LINE_HEIGHT, 0.0, 0.0, 0.0, 0.0, advance, 0.0, 0.0],
    };
    let texture = TextureDefId(textures.defs.len() as u32);
    textures.defs.push(def);
    texture
}

fn font(world: &World) -> FontDef {
    world.resource::<Library>().font_def(FONT).unwrap().clone()
}

fn test_gl() -> GlContext {
    let mut gl = GlContext::default();
    let depth = Mat4::from_scale(Vec3::new(1.0, 1.0, 0.001));
    let shift = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.5));
    gl.begin_frame(shift * depth, [0.0; 4]);
    gl
}

fn spawn_text(world: &mut World, content: &str) -> Entity {
    let entity = text::spawn(world, Some("Label"), Some(FONT));
    text::set_text(world, entity, Some(content));
    display::object_mut(world, entity).pos = Vec4::new(100.0, 160.0, SPRITE_Z, 1.0);
    entity
}

#[test]
fn char_set_maps_bytes_to_full_set_glyphs() {
    let world = new_world();
    let font = font(&world);
    let library = world.resource::<Library>();
    assert_eq!(font.glyphs[b' ' as usize], library.texture_def_id("Test:FullSet:0"));
    assert_eq!(font.glyphs[b'c' as usize], library.texture_def_id("Test:FullSet:3"));
    assert_eq!(font.glyphs[b'x' as usize], None);
}

#[test]
fn set_text_trims_only_the_outer_white_space() {
    let mut world = new_world();
    let entity = text::spawn(&mut world, None, Some(FONT));
    text::set_text(&mut world, entity, Some(" \t Hello  world \n\r"));
    let state = world.get::<Text>(entity).unwrap();
    assert_eq!(state.buffer, b"Hello  world".to_vec());
    assert_eq!(state.text_length, 12);
    assert_eq!(state.text.as_deref(), Some(" \t Hello  world \n\r"));
    assert!(state.modified);

    text::set_text(&mut world, entity, None);
    let state = world.get::<Text>(entity).unwrap();
    assert!(state.buffer.is_empty());
    assert_eq!(state.text_length, 0);
    assert_eq!(state.text, None);
}

#[test]
fn set_text_keeps_whole_characters_within_1023_bytes() {
    let mut world = new_world();
    let entity = text::spawn(&mut world, None, Some(FONT));

    let too_long = format!("{}é", "a".repeat(1022));
    text::set_text(&mut world, entity, Some(&too_long));
    assert_eq!(world.get::<Text>(entity).unwrap().text_length, 1022);

    let fitting = format!("{}é", "a".repeat(1021));
    text::set_text(&mut world, entity, Some(&fitting));
    assert_eq!(world.get::<Text>(entity).unwrap().text_length, 1023);
}

#[test]
fn text_width_is_the_widest_line_of_mapped_glyphs() {
    let world = new_world();
    let font = font(&world);
    assert_eq!(text::font_text_width(&world, &font, b"ab\nabc"), 30.5);
    assert_eq!(text::font_text_width(&world, &font, b"abc\nab"), 30.5);
    assert_eq!(text::font_text_width(&world, &font, b"xax"), 10.0);
    assert_eq!(text::font_text_width(&world, &font, b""), 0.0);
}

#[test]
fn word_wrap_breaks_at_the_last_space() {
    let world = new_world();
    let font = font(&world);
    let wrapped = text::copy_word_wrapped_text(&world, &font, b"aa bb", 25.0);
    assert_eq!(wrapped, b"aa\nbb".to_vec());
}

#[test]
fn word_wrap_writes_breaks_at_source_indices() {
    let world = new_world();
    let font = font(&world);
    // The unmapped `x` shifts the destination by one: the break meant for the space lands on
    // the first `a`, and the next space's break is overwritten by `b`.
    let wrapped = text::copy_word_wrapped_text(&world, &font, b"x aa bb", 25.0);
    assert_eq!(wrapped, b" \na bb".to_vec());
}

#[test]
fn word_wrap_remembers_the_last_space_across_newlines() {
    let world = new_world();
    let font = font(&world);
    let wrapped = text::copy_word_wrapped_text(&world, &font, b"ab \ncccc", 35.0);
    assert_eq!(wrapped, b"ab\n\ncccc".to_vec());
}

#[test]
fn word_wrap_drops_high_bytes_and_unmapped_characters() {
    let world = new_world();
    let font = font(&world);
    let source = [b'a', 0xc3, 0xa9, b'x', b'b'];
    let wrapped = text::copy_word_wrapped_text(&world, &font, &source, 1000.0);
    assert_eq!(wrapped, b"ab".to_vec());
}

#[test]
fn length_of_words_that_fit_returns_the_last_space() {
    let world = new_world();
    let font = font(&world);
    assert_eq!(text::length_of_words_that_fit(&world, &font, b"aa bb cc", 45.0), 2);
    assert_eq!(text::length_of_words_that_fit(&world, &font, b"aa bb cc", 1000.0), 8);
    assert_eq!(text::length_of_words_that_fit(&world, &font, b"aa\nbb", 1000.0), 2);
    assert_eq!(text::length_of_words_that_fit(&world, &font, b"bbb", 15.0), 0);
    // The scan is bounded by the UTF-16 length (3), not the 4 UTF-8 bytes.
    let accented = "é a".as_bytes();
    assert_eq!(text::length_of_words_that_fit(&world, &font, accented, 1000.0), 3);
}

#[test]
fn layout_justifies_each_line_by_its_floored_width() {
    let world = new_world();
    let font = font(&world);

    let left = text::fill_font_char_texture_array(&world, &font, b"ab\nc", Justification::Left);
    assert_eq!(left.len(), 3);
    assert_eq!(left[1].vertices[0], 10.0);
    assert_eq!(left[2].vertices[0], 0.0);
    assert_eq!(left[2].vertices[1], 0.0);
    assert_eq!(left[2].vertices[7], -LINE_HEIGHT);

    let right = text::fill_font_char_texture_array(&world, &font, b"abc", Justification::Right);
    assert_eq!(right[0].vertices[0], -30.0);
    assert_eq!(right[2].vertices[3], 0.5);

    let centred = text::fill_font_char_texture_array(&world, &font, b"ab\nabc", Justification::Center);
    assert_eq!(centred.len(), 5);
    assert_eq!(centred[0].vertices[0], -10.0);
    assert_eq!(centred[2].vertices[0], -15.0);
    assert_eq!(centred[2].vertices[7], -LINE_HEIGHT);
}

#[test]
fn raw_justification_values_lay_out_like_the_original() {
    assert_eq!(Justification::from_raw(1), Justification::Right);
    assert_eq!(Justification::from_raw(2), Justification::Center);
    assert_eq!(Justification::from_raw(0), Justification::Left);
    assert_eq!(Justification::from_raw(7), Justification::Left);
}

#[test]
fn text_draw_tints_by_the_direct_parent() {
    let mut world = new_world();
    let parent = display::spawn(&mut world, &display::BC_DISPLAY_OBJECT, Some("Group"));
    display::object_mut(&mut world, parent).set_rgba(0.5, 1.0, 1.0, 0.5);
    let entity = spawn_text(&mut world, "ab");
    {
        let mut object = display::object_mut(&mut world, entity);
        object.display_group = Some(parent);
        object.set_rgba(1.0, 0.5, 1.0, 1.0);
    }

    let mut gl = test_gl();
    gl.set_blend(false);
    display::draw(&mut world, entity, &mut gl);

    assert_eq!(world.get::<Text>(entity).unwrap().char_count, 2);
    let batch = &gl.list.batches[0];
    assert_eq!(batch.blend, BlendMode::Premultiplied);
    assert_eq!(batch.vertices.len(), 12);
    assert_eq!(batch.vertices[0].color, [0.25, 0.25, 0.5, 0.5]);
    assert_eq!(batch.vertices[0].position[0], 100.0);
    assert_eq!(batch.vertices[0].position[1], 172.0);
    assert_eq!(batch.vertices[6].position[0], 110.0);
}

#[test]
fn text_without_a_parent_is_drawn_with_a_zero_colour() {
    let mut world = new_world();
    let entity = spawn_text(&mut world, "a");

    let mut gl = test_gl();
    // A fresh context's colour cache is (0,0,0,0) (FUN_10013f980), which would skip the zero
    // colour; draw after another colour, as in a running frame.
    gl.set_color(1.0, 1.0, 1.0, 1.0);
    display::draw(&mut world, entity, &mut gl);

    assert_eq!(gl.color(), [0.0, 0.0, 0.0, 0.0]);
    assert_eq!(gl.list.batches[0].vertices[0].color, [0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn a_fresh_context_skips_an_initial_zero_colour() {
    let mut gl = test_gl();
    gl.set_color(0.0, 0.0, 0.0, 0.0);
    assert_eq!(gl.color(), [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn hidden_text_is_not_laid_out_but_nested_draws_ignore_hidden() {
    let mut world = new_world();
    let entity = spawn_text(&mut world, "abc");
    display::object_mut(&mut world, entity).hidden = true;

    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);
    assert!(gl.list.batches.is_empty());
    assert_eq!(world.get::<Text>(entity).unwrap().char_count, 0);

    let parent = multi_model::spawn(&mut world, None);
    gl.set_color(0.1, 0.2, 0.3, 0.4);
    display::draw_in_multi_model(&mut world, entity, &mut gl, parent, [1.0; 4]);
    assert_eq!(world.get::<Text>(entity).unwrap().char_count, 3);
    let batch = &gl.list.batches[0];
    assert_eq!(batch.vertices[0].color, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(batch.vertices[0].position[0], 0.0);
    assert_eq!(batch.vertices[0].position[1], LINE_HEIGHT);
}

#[test]
fn wrapped_text_draws_its_lines_downward() {
    let mut world = new_world();
    let entity = spawn_text(&mut world, "aa bb");
    text::set_wrap_width(&mut world, entity, 25.0);
    let parent = multi_model::spawn(&mut world, None);

    let mut gl = test_gl();
    display::draw_in_multi_model(&mut world, entity, &mut gl, parent, [1.0; 4]);

    assert_eq!(world.get::<Text>(entity).unwrap().char_count, 4);
    let vertices = &gl.list.batches[0].vertices;
    assert_eq!(vertices.len(), 24);
    assert_eq!(vertices[12].position[0], 0.0);
    assert_eq!(vertices[12].position[1], 0.0);
}

#[test]
fn unknown_font_lays_out_nothing() {
    let mut world = new_world();
    let entity = text::spawn(&mut world, None, Some("Missing"));
    text::set_text(&mut world, entity, Some("abc"));
    display::object_mut(&mut world, entity).pos = Vec4::new(0.0, 160.0, SPRITE_Z, 1.0);

    let mut gl = test_gl();
    display::draw(&mut world, entity, &mut gl);

    assert!(gl.list.batches.is_empty());
    assert_eq!(text::text_width(&world, entity), 0.0);
}

#[test]
fn text_buffer_writes_are_measured_as_a_c_string() {
    let mut world = new_world();
    let entity = spawn_text(&mut world, "old");
    text::use_text_buffer(&mut world, entity);
    world.get_mut::<Text>(entity).unwrap().buffer = b"cab\0junk".to_vec();
    text::show_text_buffer(&mut world, entity);

    let state = world.get::<Text>(entity).unwrap();
    assert_eq!(state.text, None);
    assert_eq!(state.text_length, 3);
    assert_eq!(text::text_width(&world, entity), 30.5);
}
