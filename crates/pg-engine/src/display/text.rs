//! `BCFontDef` and `BCText`: bitmap text from atlas glyphs (`docs/re/rendering.md` §5.7, §6).

use bevy_ecs::prelude::*;

use crate::display::{self, BC_DISPLAY_OBJECT, DisplayObject, model};
use crate::gl::GlContext;
use crate::library::Library;
use crate::object::{Class, ObjectWorldExt};
use crate::texture::{TextureDefId, Textures};

/// `mTextBuffer` holds at most this many bytes before its terminator.
const TEXT_BUFFER_CAPACITY: usize = 1023;

/// `BCFontDef`: glyph texture definitions by byte value (`"<fontId>:FullSet:<i>"`).
#[derive(Clone, Debug)]
pub struct FontDef {
    pub id: String,
    pub char_set: String,
    pub glyphs: [Option<TextureDefId>; 256],
}

impl FontDef {
    /// `-[BCFontDef initWithId:]` @ 0x100066780: an empty glyph map.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            char_set: String::new(),
            glyphs: [None; 256],
        }
    }

    /// `-[BCFontDef setCharSet:]` @ 0x100066a34: for `i` below the string's (UTF-16) length,
    /// the glyph of UTF-8 byte `i` is the texture def `"<id>:FullSet:<i>"` (NULL if unknown).
    pub fn set_char_set(&mut self, char_set: &str, library: &Library) {
        self.char_set = char_set.to_owned();
        let length = char_set.encode_utf16().count();
        let bytes = char_set.as_bytes().iter().take(length);
        for (index, byte) in bytes.enumerate() {
            let glyph_id = format!("{}:FullSet:{}", self.id, index);
            self.glyphs[*byte as usize] = library.texture_def_id(&glyph_id);
        }
    }

    fn glyph(&self, byte: u8) -> Option<TextureDefId> {
        self.glyphs[byte as usize]
    }
}

/// `FontJustification`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Justification {
    #[default]
    Left = 0,
    Right = 1,
    Center = 2,
}

impl Justification {
    /// The justification of a raw `FontJustification` value. Values other than 1 and 2 lay
    /// text out exactly like 0 (they only measure lines), so they map to `Left`.
    pub fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Justification::Right,
            2 => Justification::Center,
            _ => Justification::Left,
        }
    }
}

/// `BCText` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct Text {
    pub modified: bool,
    pub text: Option<String>,

    /// `mTextBuffer` (≤ 1023 bytes, trimmed).
    pub buffer: Vec<u8>,
    pub font: Option<String>,
    pub justification: Justification,
    pub wrap_width: f32,
    pub line_count: i32,
    pub text_length: i32,
    pub char_count: i32,
}

/// One entry of the shared glyph array (`FontCharTexture` at `DAT_1007d1f60`): the glyph's
/// texture def (its page and UVs) and its quad moved to the pen position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontCharTexture {
    pub texture: TextureDefId,
    pub vertices: [f32; 12],
}

pub const BC_TEXT: Class = Class {
    name: "BCText",
    superclass: Some(&BC_DISPLAY_OBJECT),
    draw: draw_text,
    draw_in_multi_model: draw_text_in_multi_model,
    ..BC_DISPLAY_OBJECT
};

/// `-[BCText initWithId:fontDef:]` @ 0x1001a75b0.
pub fn spawn(world: &mut World, id: Option<&str>, font: Option<&str>) -> Entity {
    let entity = display::spawn(world, &BC_TEXT, id);
    let text = Text {
        font: font.map(str::to_owned),
        line_count: 100,
        ..Default::default()
    };
    world.entity_mut(entity).insert(text);
    entity
}

/// The C string in `bytes`: everything before the first NUL (`strlen`).
fn c_string(bytes: &[u8]) -> &[u8] {
    match bytes.iter().position(|byte| *byte == 0) {
        Some(end) => &bytes[..end],
        None => bytes,
    }
}

/// `' '`, `'\t'`, `'\n'` and `'\r'` (the mask `0x100002600`).
fn is_trimmed_white_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

/// The UTF-8 bytes `getBytes:maxLength:1023…` copies: the longest prefix of whole characters
/// that fits in the buffer.
fn utf8_prefix(text: &str) -> &[u8] {
    let mut end = text.len().min(TEXT_BUFFER_CAPACITY);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text.as_bytes()[..end]
}

/// `-setText:` @ 0x1001a781c (trims leading/trailing whitespace; nil → empty).
pub fn set_text(world: &mut World, entity: Entity, text: Option<&str>) {
    let mut buffer = Vec::new();
    let mut length = 0;
    if let Some(text) = text {
        let source = utf8_prefix(text);
        let mut leading = true;
        for byte in source.iter().copied() {
            if leading && is_trimmed_white_space(byte) {
                continue;
            }
            leading = false;
            buffer.push(byte);
            if !is_trimmed_white_space(byte) {
                length = buffer.len();
            }
        }
        buffer.truncate(length);
    }

    let mut state = world.obj_mut::<Text>(entity);
    state.buffer = buffer;
    state.text_length = length as i32;
    state.text = text.map(str::to_owned);
    state.modified = true;
}

/// `-setFontJustification:` @ 0x1001a79c0.
pub fn set_font_justification(world: &mut World, entity: Entity, justification: Justification) {
    let mut state = world.obj_mut::<Text>(entity);
    state.justification = justification;
    state.modified = true;
}

/// `-setWrapWidth:` @ 0x1001a79f0 (0 = no wrapping).
pub fn set_wrap_width(world: &mut World, entity: Entity, wrap_width: f32) {
    let mut state = world.obj_mut::<Text>(entity);
    state.wrap_width = wrap_width;
    state.modified = true;
}

/// `-setLineCount:` @ 0x1001a7a20 (stored only; drawing ignores it).
pub fn set_line_count(world: &mut World, entity: Entity, line_count: i32) {
    let mut state = world.obj_mut::<Text>(entity);
    state.line_count = line_count;
    state.modified = true;
}

/// `-setFontDef:` @ 0x1001a7a50 (the font by id; `None` is a nil font).
pub fn set_font_def(world: &mut World, entity: Entity, font: Option<&str>) {
    world.obj_mut::<Text>(entity).font = font.map(str::to_owned);
}

/// `-useTextBuffer` @ 0x1001a779c: releases `mText`; the caller then writes the C string
/// into `buffer` directly and calls [`show_text_buffer`].
pub fn use_text_buffer(world: &mut World, entity: Entity) {
    world.obj_mut::<Text>(entity).text = None;
}

/// `-showTextBuffer` @ 0x1001a77e4: `mTextLength = strlen(mTextBuffer)`.
pub fn show_text_buffer(world: &mut World, entity: Entity) {
    let mut state = world.obj_mut::<Text>(entity);
    let length = c_string(&state.buffer).len();
    state.text_length = length as i32;
}

/// The text's font, or `None` for a nil `mFontDef` (unregistered id).
fn font_of<'w>(world: &'w World, text: &Text) -> Option<&'w FontDef> {
    let font_id = text.font.as_deref()?;
    world.resource::<Library>().font_def(font_id)
}

/// `-textWidth` @ 0x1001a7760.
pub fn text_width(world: &World, entity: Entity) -> f32 {
    let text = world.obj::<Text>(entity);
    match font_of(world, text) {
        Some(font) => font_text_width(world, font, &text.buffer),
        None => 0.0,
    }
}

/// The glyph advance (frame width, `+0x28`).
fn advance(textures: &Textures, glyph: TextureDefId) -> f32 {
    textures.def(glyph).frame_width
}

/// `-[BCFontDef textWidth:]` @ 0x100066818.
pub fn font_text_width(world: &World, font: &FontDef, text: &[u8]) -> f32 {
    let textures = world.resource::<Textures>();
    let mut line_width = 0.0;
    let mut widest = 0.0;
    for byte in c_string(text).iter().copied() {
        if byte == b'\n' {
            widest = if line_width > widest { line_width } else { widest };
            line_width = 0.0;
        } else if let Some(glyph) = font.glyph(byte) {
            line_width += advance(textures, glyph);
        }
    }
    if line_width > widest { line_width } else { widest }
}

/// `-[BCFontDef copyWordWrappedText:destinationBuffer:textLength:width:]` @ 0x100066970.
///
/// `text` is the source of `textLength` bytes. A line breaks at its last space once its
/// width reaches `width`; the break is written at the space's *source* index into the
/// destination (an original bug that shows once characters were dropped), and the last
/// space is not forgotten at a newline.
pub fn copy_word_wrapped_text(world: &World, font: &FontDef, text: &[u8], width: f32) -> Vec<u8> {
    let textures = world.resource::<Textures>();
    let mut destination = vec![0u8; text.len() + 1];
    let mut out = 0;
    let mut last_space: Option<usize> = None;
    let mut line_width = 0.0;
    let mut width_at_space = 0.0;

    for (index, byte) in text.iter().copied().enumerate() {
        if byte == b'\n' {
            destination[out] = b'\n';
            out += 1;
            line_width = 0.0;
            continue;
        }
        // Signed `char`: bytes from 0x80 up are dropped like unmapped characters.
        if (byte as i8) < 0 {
            continue;
        }
        let Some(glyph) = font.glyph(byte) else {
            continue;
        };
        line_width += advance(textures, glyph);
        destination[out] = byte;
        if byte == b' ' {
            last_space = Some(index);
            width_at_space = line_width;
        }
        if let Some(space) = last_space
            && line_width >= width
        {
            destination[space] = b'\n';
            line_width -= width_at_space;
            last_space = None;
        }
        out += 1;
    }

    destination.truncate(out);
    destination
}

/// `-[BCFontDef lengthOfWordsThatFit:width:]` @ 0x1000668a8.
///
/// `text` is the string's UTF-8; the scan is bounded by its UTF-16 length (`-length`).
pub fn length_of_words_that_fit(world: &World, font: &FontDef, text: &[u8], width: f32) -> i32 {
    let textures = world.resource::<Textures>();
    let length = match std::str::from_utf8(text) {
        Ok(string) => string.encode_utf16().count(),
        Err(_) => text.len(),
    };

    let mut last_space = 0;
    let mut line_width = 0.0;
    let mut index = 0;
    while index < length {
        let byte = text[index];
        if byte == b'\n' {
            break;
        }
        if byte == b' ' {
            last_space = index;
        }
        if let Some(glyph) = font.glyph(byte) {
            line_width += advance(textures, glyph);
            if line_width >= width {
                return last_space as i32;
            }
        }
        index += 1;
    }
    index as i32
}

/// `-[BCFontDef fillFontCharTextureArray:textBuffer:fontJustification:]` @ 0x100066c08:
/// lays out the C string `text` from the pen at (0, 0); lines go down by the frame height
/// of the last glyph drawn. Right and centred lines start at `-floor(width)` and
/// `-floor(width / 2)`.
pub fn fill_font_char_texture_array(world: &World, font: &FontDef, text: &[u8], justification: Justification) -> Vec<FontCharTexture> {
    let textures = world.resource::<Textures>();
    let text = c_string(text);
    let mut glyphs = Vec::new();
    let mut pen_x: f32 = 0.0;
    let mut pen_y: f32 = 0.0;
    let mut line_height = 0.0;
    let mut line_width = 0.0;

    for index in 0..text.len() {
        if justification != Justification::Left && line_width == 0.0 {
            for byte in text[index..].iter().copied() {
                if byte == b'\n' {
                    break;
                }
                if let Some(glyph) = font.glyph(byte) {
                    line_width += advance(textures, glyph);
                }
            }
            if justification == Justification::Right {
                pen_x = -line_width.floor();
            } else {
                let half_width: f32 = line_width * 0.5;
                pen_x = -half_width.floor();
            }
        }

        let byte = text[index];
        if byte == b'\n' {
            pen_y -= line_height;
            line_width = 0.0;
            pen_x = 0.0;
            continue;
        }
        let Some(glyph) = font.glyph(byte) else {
            continue;
        };
        let def = textures.def(glyph);
        let mut vertices = def.vertices;
        for corner in 0..4 {
            vertices[corner * 3] += pen_x;
            vertices[corner * 3 + 1] += pen_y;
        }
        let entry = FontCharTexture { texture: glyph, vertices };
        glyphs.push(entry);
        line_height = def.frame_height;
        pen_x += def.frame_width;
    }
    glyphs
}

/// `-updateDisplayText` @ 0x1001a7670: the display string (the buffer, or its word-wrapped
/// copy) laid out into glyphs; `mCharCount` = their number.
fn update_display_text(world: &mut World, entity: Entity) -> Vec<FontCharTexture> {
    let glyphs = {
        let text = world.obj::<Text>(entity);
        match font_of(world, text) {
            None => Vec::new(),
            Some(font) => {
                let display_text = display_text(world, font, text);
                fill_font_char_texture_array(world, font, &display_text, text.justification)
            }
        }
    };
    world.obj_mut::<Text>(entity).char_count = glyphs.len() as i32;
    glyphs
}

/// The string `updateDisplayText` lays out: `strncpy` of the buffer cut at `mTextLength`, or
/// the word-wrapped copy of its first `mTextLength` bytes.
fn display_text(world: &World, font: &FontDef, text: &Text) -> Vec<u8> {
    let text_length = text.text_length.max(0) as usize;
    if text.wrap_width == 0.0 {
        let copied = c_string(&text.buffer);
        let end = copied.len().min(TEXT_BUFFER_CAPACITY).min(text_length);
        return copied[..end].to_vec();
    }
    let mut source = Vec::with_capacity(text_length);
    for index in 0..text_length {
        let byte = text.buffer.get(index).copied().unwrap_or(0);
        source.push(byte);
    }
    copy_word_wrapped_text(world, font, &source, text.wrap_width)
}

/// The glyph quads, blended, with the current colour and transform.
fn draw_glyphs(world: &World, glyphs: &[FontCharTexture], gl: &mut GlContext) {
    for glyph in glyphs {
        model::draw_quad(world, glyph.texture, &glyph.vertices, true, gl);
    }
}

/// The colour channels of the direct parent (`[mDisplayGroup red]` …); nil answers zeros.
fn parent_color(world: &World, parent: Option<Entity>) -> [f32; 4] {
    match parent.and_then(|group| world.get::<DisplayObject>(group)) {
        Some(group) => [group.red, group.green, group.blue, group.alpha],
        None => [0.0; 4],
    }
}

/// `BCText::drawOpenGL:` @ 0x1001a7b28.
fn draw_text(world: &mut World, entity: Entity, gl: &mut GlContext) {
    if display::object(world, entity).hidden {
        return;
    }
    let glyphs = update_display_text(world, entity);

    let display = display::object(world, entity);
    let [parent_red, parent_green, parent_blue, parent_alpha] = parent_color(world, display.display_group);
    let red = parent_red * display.red;
    let green = parent_green * display.green;
    let blue = parent_blue * display.blue;
    let alpha = parent_alpha * display.alpha;
    gl.set_color(red * alpha, green * alpha, blue * alpha, alpha);

    gl.push_matrix();
    model::apply_sprite_transform(display, gl);
    draw_glyphs(world, &glyphs, gl);
    gl.pop_matrix();
}

/// `BCText::drawInMultiModelOpenGL:…` @ 0x1001a7d70.
fn draw_text_in_multi_model(world: &mut World, entity: Entity, gl: &mut GlContext, _parent: Entity, _rgba: [f32; 4]) {
    let glyphs = update_display_text(world, entity);
    draw_glyphs(world, &glyphs, gl);
}
