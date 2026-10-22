//! Parser for the original `Timelines.bin` (`docs/re/timelines.md` §2): a port of the reader
//! of `tools/re/timelines.py` for the 64-bit layout of v1.48.2, including the structural
//! checks of its `validate` command that fail a file (the "explained anomalies" it only warns
//! about — irregular frame indices, exporter invariants, NaN colours, dangling next
//! animations — are accepted).
//!
//! The file is six sections, each a little-endian `u64` size followed by that many bytes:
//! strings, animations, texts, frames, state pointers and states. String references are
//! byte offsets from the start of the string data (`0xFFFFFFFF` = none); record offsets are
//! relative to the data of their section. The records borrow their strings from the input.

use std::collections::HashSet;
use std::fmt;

/// Path of the embedded file, relative to `assets/`.
pub const PATH: &str = "timelines/Timelines.bin";

/// Word size of the arm64 build (the 32-bit v1.45.3 file is not supported).
pub const WORD_SIZE: usize = 8;

/// A string reference meaning "no string" (compared as the full 64-bit word).
pub const NULL_STRING: u64 = 0xFFFF_FFFF;

/// `BinaryAnimationDef`.
pub const ANIMATION_RECORD_SIZE: usize = 72;

/// Dynamic text record.
pub const TEXT_RECORD_SIZE: usize = 40;

/// `BinaryFrameDef`.
pub const FRAME_RECORD_SIZE: usize = 48;

/// Frame state: 140 bytes of fields plus 4 bytes of alignment padding.
pub const STATE_RECORD_SIZE: usize = 144;

/// Tweenable channels of a `BCAnimation` (valid `channel` values are below this).
pub const CHANNEL_COUNT: i64 = 30;

/// Frame-state flag bits (§2.8).
pub mod flags {
    pub const X: u64 = 0x0001;
    pub const Y: u64 = 0x0002;
    pub const ROTATION: u64 = 0x0020;
    pub const SKEW_X: u64 = 0x0040;
    pub const SKEW_Y: u64 = 0x0080;
    pub const SCALE_X: u64 = 0x0100;
    pub const SCALE_Y: u64 = 0x0200;
    pub const ALPHA: u64 = 0x0800;
    pub const COLOR: u64 = 0x1000;
    pub const TEXTURE: u64 = 0x2000;
    pub const DYNAMIC_TEXT: u64 = 0x4000;

    /// Every bit that may occur in the file.
    pub const KNOWN: u64 =
        X | Y | ROTATION | SKEW_X | SKEW_Y | SCALE_X | SCALE_Y | ALPHA | COLOR | TEXTURE | DYNAMIC_TEXT;

    /// Set by `parseBinAnim:ext:` on the states it has fixed up; never in the file.
    pub const LOADED: u64 = 0x8000_0000;
}

/// Number of example problems quoted in an error message.
const EXAMPLE_LIMIT: usize = 8;

const SECTION_NAMES: [&str; 6] = ["strings", "animations", "texts", "frames", "state_pointers", "states"];

/// Why a file was rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimelineError {
    pub message: String,
}

impl TimelineError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for TimelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Timelines.bin: {}", self.message)
    }
}

impl std::error::Error for TimelineError {}

/// A dynamic text record (§2.5): the loader creates a `BCText` from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRecord<'a> {
    /// Dynamic-text id (unique); also the string that states reference.
    pub id: &'a str,

    /// Initial text (`setText:`).
    pub text: &'a str,

    /// `BCFontDef` id.
    pub font: &'a str,

    /// 0 left, 1 right, 2 centre (the loader uses the low 32 bits).
    pub justification: i64,
}

/// `BinaryAnimationDef` (§2.4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationRecord<'a> {
    pub name: &'a str,

    /// Frames per second (`BCBinaryAnimationDef::fps` reads the low 32 bits).
    pub fps: i64,

    /// Number of frames (`length` reads the low 32 bits).
    pub length: i64,

    /// 0 or 1; `loop` returns `value != 0`.
    pub looping: i64,

    /// 0 or 1; `isMoveEnabled` returns `value != 0`, `applyMovement` requires `value == 1`.
    pub move_enabled: i64,

    /// Displacement per full cycle.
    pub move_x: f32,
    pub move_y: f32,

    /// Played when a non-looping animation completes.
    pub next_animation: Option<&'a str>,

    /// Index of the first frame record in [`TimelineFile::frames`].
    pub first_frame: usize,

    pub frame_count: usize,
}

/// `BinaryFrameDef` (§2.6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameRecord<'a> {
    /// Frame number the record shows (compared as its low 32 bits).
    pub index: i64,

    /// `BCSound` id played when the frame is reached.
    pub sound: Option<&'a str>,

    /// Selector sent to the animation listener.
    pub action: Option<&'a str>,

    /// Cumulative displacement within the cycle at this frame.
    pub move_x: f32,
    pub move_y: f32,

    /// Index of the frame's first slot in [`TimelineFile::state_pointers`].
    pub first_pointer: usize,

    /// Number of states (draw order).
    pub state_count: usize,
}

/// A frame state record (§2.8).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateRecord<'a> {
    /// Flash instance / layer name.
    pub name: Option<&'a str>,

    /// Flag bits ([`flags`]).
    pub flags: u64,

    /// Texture id ([`flags::TEXTURE`]) or dynamic-text id ([`flags::DYNAMIC_TEXT`]).
    pub texture: Option<&'a str>,

    /// Tweenable channel 0–29 (0 = none).
    pub channel: i64,

    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub skew_x: f32,
    pub skew_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub alpha: f32,
    pub red: f32,
    pub green: f32,
    pub blue: f32,

    /// Quad corners v0..v3 as (x, y, z).
    pub vertices: [f32; 12],
}

/// The parsed file.
#[derive(Clone, Debug)]
pub struct TimelineFile<'a> {
    /// Size of the string table, NUL padding included.
    pub string_table_size: usize,

    /// Strings in the table, including the empty strings of the padding.
    pub string_count: usize,

    pub texts: Vec<TextRecord<'a>>,
    pub animations: Vec<AnimationRecord<'a>>,
    pub frames: Vec<FrameRecord<'a>>,

    /// The state index of every pointer slot, in file order (draw order within a frame).
    pub state_pointers: Vec<u32>,

    pub states: Vec<StateRecord<'a>>,
}

impl<'a> TimelineFile<'a> {
    /// The frame records of an animation.
    pub fn frames_of(&self, animation: &AnimationRecord) -> &[FrameRecord<'a>] {
        let end = animation.first_frame + animation.frame_count;
        &self.frames[animation.first_frame..end]
    }

    /// The state indices of a frame, in draw order.
    pub fn states_of(&self, frame: &FrameRecord) -> &[u32] {
        let end = frame.first_pointer + frame.state_count;
        &self.state_pointers[frame.first_pointer..end]
    }

    /// The animation named `name`.
    pub fn animation(&self, name: &str) -> Option<&AnimationRecord<'a>> {
        self.animations.iter().find(|animation| animation.name == name)
    }
}

/// Parses the embedded `Timelines.bin`.
pub fn load() -> Result<TimelineFile<'static>, TimelineError> {
    let Some(bytes) = crate::file(PATH) else {
        return Err(TimelineError::new("not embedded"));
    };
    parse(bytes)
}

/// Parses and validates a 64-bit `Timelines.bin`.
pub fn parse(bytes: &[u8]) -> Result<TimelineFile<'_>, TimelineError> {
    let sections = read_sections(bytes)?;
    let [string_section, animation_section, text_section, frame_section, pointer_section, state_section] = sections;

    let string_data = section_bytes(bytes, &string_section);
    let mut strings = StringTable::new(string_data)?;

    let texts = parse_texts(bytes, &text_section, &mut strings)?;
    let states = parse_states(bytes, &state_section, &mut strings)?;
    let state_count = states.records.len();
    let state_pointers = parse_pointers(bytes, &pointer_section, state_count)?;
    let pointer_count = state_pointers.len();
    let frames = parse_frames(bytes, &frame_section, &mut strings, pointer_count)?;
    let frame_count = frames.records.len();
    let animations = parse_animations(bytes, &animation_section, &mut strings, frame_count)?;

    let file = TimelineFile {
        string_table_size: string_data.len(),
        string_count: strings.starts.len(),
        texts: texts.records,
        animations: animations.records,
        frames: frames.records,
        state_pointers,
        states: states.records,
    };

    let header_bytes = (SECTION_NAMES.len() + 1) * WORD_SIZE;
    let record_bytes = file.animations.len() * ANIMATION_RECORD_SIZE
        + file.texts.len() * TEXT_RECORD_SIZE
        + file.frames.len() * FRAME_RECORD_SIZE
        + file.state_pointers.len() * WORD_SIZE
        + file.states.len() * STATE_RECORD_SIZE;
    let accounted = header_bytes + file.string_table_size + record_bytes;
    if accounted != bytes.len() {
        let message = format!("byte accounting: {accounted} of {} bytes explained", bytes.len());
        return Err(TimelineError::new(message));
    }

    check_strings(&strings)?;
    check_unique_names(&file)?;
    check_animation_fields(&file)?;
    check_frame_coverage(&file, &animations.frames_offsets)?;
    check_pointer_coverage(&file, &frames.states_offsets)?;
    check_state_coverage(&file)?;
    check_load_slots(&texts.font_def_slots, &states.slots)?;
    check_state_flags(&file)?;
    check_floats(&file)?;
    check_dynamic_text_references(&file)?;
    check_justifications(&file)?;

    Ok(file)
}

#[derive(Clone, Copy, Debug)]
struct Section {
    name: &'static str,
    data_offset: usize,
    size: usize,
}

fn section_bytes<'a>(bytes: &'a [u8], section: &Section) -> &'a [u8] {
    let end = section.data_offset + section.size;
    &bytes[section.data_offset..end]
}

fn read_word_checked(bytes: &[u8], offset: usize) -> Result<u64, TimelineError> {
    let end = offset.checked_add(WORD_SIZE);
    let Some(word) = end.and_then(|end| bytes.get(offset..end)) else {
        let message = format!("word at {offset:#x} runs past the end of the file");
        return Err(TimelineError::new(message));
    };
    Ok(read_u64(word, 0))
}

/// Little-endian `u64` at `offset` (the caller has checked the bounds).
fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    let mut word = [0u8; 8];
    word.copy_from_slice(&bytes[offset..offset + 8]);
    u64::from_le_bytes(word)
}

fn read_i64(bytes: &[u8], offset: usize) -> i64 {
    let mut word = [0u8; 8];
    word.copy_from_slice(&bytes[offset..offset + 8]);
    i64::from_le_bytes(word)
}

fn read_f32(bytes: &[u8], offset: usize) -> f32 {
    let mut word = [0u8; 4];
    word.copy_from_slice(&bytes[offset..offset + 4]);
    f32::from_le_bytes(word)
}

fn to_usize(value: u64, what: &str) -> Result<usize, TimelineError> {
    match usize::try_from(value) {
        Ok(converted) => Ok(converted),
        Err(_) => {
            let message = format!("{what} {value:#x} is out of range");
            Err(TimelineError::new(message))
        }
    }
}

/// Six sections, each a size word followed by that many bytes, filling the file exactly.
fn read_sections(bytes: &[u8]) -> Result<[Section; 6], TimelineError> {
    let mut sections = [Section {
        name: "",
        data_offset: 0,
        size: 0,
    }; 6];
    let mut offset = 0usize;

    for (slot, name) in sections.iter_mut().zip(SECTION_NAMES) {
        let size_word = read_word_checked(bytes, offset)?;
        let size = to_usize(size_word, "section size")?;
        let data_offset = offset + WORD_SIZE;
        let end = data_offset.checked_add(size);
        let Some(end) = end.filter(|end| *end <= bytes.len()) else {
            let message = format!("section {name} ({size} bytes at {data_offset:#x}) runs past the end of the file");
            return Err(TimelineError::new(message));
        };
        *slot = Section { name, data_offset, size };
        offset = end;
    }

    if offset != bytes.len() {
        let unexplained = bytes.len() - offset;
        let message = format!("{unexplained} unexplained bytes after the last section");
        return Err(TimelineError::new(message));
    }
    Ok(sections)
}

fn record_count(section: &Section, record_size: usize) -> Result<usize, TimelineError> {
    if !section.size.is_multiple_of(record_size) {
        let message = format!("{} section size {} is not a multiple of {record_size}", section.name, section.size);
        return Err(TimelineError::new(message));
    }
    Ok(section.size / record_size)
}

/// NUL-terminated ASCII strings, referenced by byte offset from the start of the table.
struct StringTable<'a> {
    text: &'a str,
    starts: Vec<u32>,

    /// For every byte offset of the table: 1 + the index of the string starting there, or 0.
    index_by_offset: Vec<u32>,

    referenced: Vec<bool>,
}

impl<'a> StringTable<'a> {
    fn new(data: &'a [u8]) -> Result<Self, TimelineError> {
        if let Some(&last) = data.last()
            && last != 0
        {
            return Err(TimelineError::new("string table does not end with a NUL byte"));
        }
        if let Some(position) = data.iter().position(|byte| !byte.is_ascii()) {
            let message = format!("byte {position:#x} of the string table is not ASCII");
            return Err(TimelineError::new(message));
        }
        let Ok(text) = std::str::from_utf8(data) else {
            return Err(TimelineError::new("the string table is not ASCII"));
        };
        if u32::try_from(data.len()).is_err() {
            return Err(TimelineError::new("the string table is larger than 4 GB"));
        }

        let mut starts = Vec::new();
        let mut index_by_offset = vec![0u32; data.len()];
        let mut offset = 0usize;
        while offset < data.len() {
            starts.push(offset as u32);
            index_by_offset[offset] = starts.len() as u32;
            let rest = &data[offset..];
            let length = rest.iter().position(|byte| *byte == 0).unwrap_or(rest.len());
            offset += length + 1;
        }

        let referenced = vec![false; starts.len()];
        let table = Self {
            text,
            starts,
            index_by_offset,
            referenced,
        };
        Ok(table)
    }

    /// The text of string `index` (without its NUL).
    fn text_of(&self, index: usize) -> &'a str {
        let start = self.starts[index] as usize;
        let end = match self.starts.get(index + 1) {
            Some(next) => *next as usize - 1,
            None => self.text.len() - 1,
        };
        &self.text[start..end]
    }

    fn lookup(&mut self, reference: u64) -> Result<Option<&'a str>, TimelineError> {
        if reference == NULL_STRING {
            return Ok(None);
        }

        let slot = usize::try_from(reference).ok().and_then(|offset| self.index_by_offset.get(offset));
        let Some(&slot) = slot.filter(|slot| **slot != 0) else {
            let message = format!("string offset {reference:#x} is not the start of a string");
            return Err(TimelineError::new(message));
        };

        let index = slot as usize - 1;
        self.referenced[index] = true;
        Ok(Some(self.text_of(index)))
    }
}

struct ParsedTexts<'a> {
    records: Vec<TextRecord<'a>>,
    font_def_slots: Vec<u64>,
}

fn parse_texts<'a>(
    bytes: &'a [u8],
    section: &Section,
    strings: &mut StringTable<'a>,
) -> Result<ParsedTexts<'a>, TimelineError> {
    let count = record_count(section, TEXT_RECORD_SIZE)?;
    let mut records = Vec::with_capacity(count);
    let mut font_def_slots = Vec::with_capacity(count);
    let mut problems = Vec::new();

    let data = section_bytes(bytes, section);
    let (record_arrays, _) = data.as_chunks::<TEXT_RECORD_SIZE>();
    for (record_index, record) in record_arrays.iter().enumerate() {
        let id_reference = read_u64(record, 0);
        let text_reference = read_u64(record, 8);
        let font_reference = read_u64(record, 16);
        let font_def_slot = read_u64(record, 24);
        let justification = read_i64(record, 32);

        let id = strings.lookup(id_reference)?;
        let text = strings.lookup(text_reference)?;
        let font = strings.lookup(font_reference)?;
        font_def_slots.push(font_def_slot);

        let (Some(id), Some(text), Some(font)) = (id, text, font) else {
            problems.push(format!("text record {record_index}: id={id:?} font={font:?} text={text:?}"));
            continue;
        };
        let record = TextRecord {
            id,
            text,
            font,
            justification,
        };
        records.push(record);
    }

    fail_on_problems("text records", &problems)?;
    Ok(ParsedTexts { records, font_def_slots })
}

struct ParsedStates<'a> {
    records: Vec<StateRecord<'a>>,

    /// `(texture def slot, dynamic text slot)` of every state.
    slots: Vec<(u64, u64)>,
}

fn parse_states<'a>(
    bytes: &'a [u8],
    section: &Section,
    strings: &mut StringTable<'a>,
) -> Result<ParsedStates<'a>, TimelineError> {
    let count = record_count(section, STATE_RECORD_SIZE)?;
    let mut records = Vec::with_capacity(count);
    let mut slots = Vec::with_capacity(count);

    let data = section_bytes(bytes, section);
    let (record_arrays, _) = data.as_chunks::<STATE_RECORD_SIZE>();
    for record in record_arrays {
        let name_reference = read_u64(record, 0);
        let flag_bits = read_u64(record, 8);
        let texture_reference = read_u64(record, 16);
        let texture_def_slot = read_u64(record, 24);
        let channel = read_i64(record, 32);
        let dynamic_text_slot = read_u64(record, 40);

        let name = strings.lookup(name_reference)?;
        let texture = strings.lookup(texture_reference)?;

        let floats_offset = 6 * WORD_SIZE;
        let mut values = [0f32; 11];
        for (index, value) in values.iter_mut().enumerate() {
            *value = read_f32(record, floats_offset + index * 4);
        }
        let vertices_offset = floats_offset + 11 * 4;
        let mut vertices = [0f32; 12];
        for (index, vertex) in vertices.iter_mut().enumerate() {
            *vertex = read_f32(record, vertices_offset + index * 4);
        }

        let [x, y, rotation, skew_x, skew_y, scale_x, scale_y, alpha, red, green, blue] = values;
        let record = StateRecord {
            name,
            flags: flag_bits,
            texture,
            channel,
            x,
            y,
            rotation,
            skew_x,
            skew_y,
            scale_x,
            scale_y,
            alpha,
            red,
            green,
            blue,
            vertices,
        };
        records.push(record);
        slots.push((texture_def_slot, dynamic_text_slot));
    }

    Ok(ParsedStates { records, slots })
}

/// Converts every pointer slot (a byte offset into the states section) to a state index.
fn parse_pointers(bytes: &[u8], section: &Section, state_count: usize) -> Result<Vec<u32>, TimelineError> {
    let count = record_count(section, WORD_SIZE)?;
    let mut pointers = Vec::with_capacity(count);

    let data = section_bytes(bytes, section);
    let (slot_arrays, _) = data.as_chunks::<WORD_SIZE>();
    for (pointer_index, slot) in slot_arrays.iter().enumerate() {
        let pointer = u64::from_le_bytes(*slot);
        if !pointer.is_multiple_of(STATE_RECORD_SIZE as u64) {
            let message = format!("state pointer {pointer_index}: misaligned state offset {pointer:#x}");
            return Err(TimelineError::new(message));
        }
        let state_index = pointer / STATE_RECORD_SIZE as u64;
        if state_index >= state_count as u64 {
            let message = format!("state pointer {pointer_index}: state offset {pointer:#x} out of bounds");
            return Err(TimelineError::new(message));
        }
        pointers.push(state_index as u32);
    }

    Ok(pointers)
}

struct ParsedFrames<'a> {
    records: Vec<FrameRecord<'a>>,

    /// Raw `states_offset` of every frame (for the coverage check).
    states_offsets: Vec<u64>,
}

fn parse_frames<'a>(
    bytes: &'a [u8],
    section: &Section,
    strings: &mut StringTable<'a>,
    pointer_count: usize,
) -> Result<ParsedFrames<'a>, TimelineError> {
    let count = record_count(section, FRAME_RECORD_SIZE)?;
    let mut records = Vec::with_capacity(count);
    let mut states_offsets = Vec::with_capacity(count);

    let data = section_bytes(bytes, section);
    let (record_arrays, _) = data.as_chunks::<FRAME_RECORD_SIZE>();
    for (record_index, record) in record_arrays.iter().enumerate() {
        let index = read_i64(record, 0);
        let sound_reference = read_u64(record, 8);
        let action_reference = read_u64(record, 16);
        let move_x = read_f32(record, 24);
        let move_y = read_f32(record, 28);
        let state_count = read_i64(record, 32);
        let states_offset = read_u64(record, 40);

        let sound = strings.lookup(sound_reference)?;
        let action = strings.lookup(action_reference)?;

        if !states_offset.is_multiple_of(WORD_SIZE as u64) {
            let message = format!("frame record {record_index}: misaligned pointer offset {states_offset:#x}");
            return Err(TimelineError::new(message));
        }
        let first_pointer = to_usize(states_offset / WORD_SIZE as u64, "pointer index")?;
        let out_of_bounds = || {
            let message = format!("frame record {record_index}: pointer range out of bounds");
            TimelineError::new(message)
        };
        let Ok(state_count) = usize::try_from(state_count) else {
            return Err(out_of_bounds());
        };
        let last_pointer = first_pointer.checked_add(state_count);
        if last_pointer.is_none_or(|last| last > pointer_count) {
            return Err(out_of_bounds());
        }

        let record = FrameRecord {
            index,
            sound,
            action,
            move_x,
            move_y,
            first_pointer,
            state_count,
        };
        records.push(record);
        states_offsets.push(states_offset);
    }

    Ok(ParsedFrames { records, states_offsets })
}

struct ParsedAnimations<'a> {
    records: Vec<AnimationRecord<'a>>,

    /// Raw `frames_offset` of every animation (for the coverage check).
    frames_offsets: Vec<u64>,
}

fn parse_animations<'a>(
    bytes: &'a [u8],
    section: &Section,
    strings: &mut StringTable<'a>,
    frame_count_total: usize,
) -> Result<ParsedAnimations<'a>, TimelineError> {
    let count_word = read_word_checked(bytes, section.data_offset)?;
    let count = to_usize(count_word, "animation count")?;
    let expected_size = count
        .checked_mul(ANIMATION_RECORD_SIZE)
        .and_then(|records| records.checked_add(WORD_SIZE));
    if expected_size != Some(section.size) {
        let size = section.size;
        let message = format!("animation section is {size} bytes, expected {expected_size:?} for {count} records");
        return Err(TimelineError::new(message));
    }

    let data = section_bytes(bytes, section);
    let record_data = &data[WORD_SIZE..];
    let (record_arrays, _) = record_data.as_chunks::<ANIMATION_RECORD_SIZE>();
    let mut records = Vec::with_capacity(count);
    let mut frames_offsets = Vec::with_capacity(count);

    for (record_index, record) in record_arrays.iter().enumerate() {
        let name_reference = read_u64(record, 0);
        let fps = read_i64(record, 8);
        let length = read_i64(record, 16);
        let looping = read_i64(record, 24);
        let move_enabled = read_i64(record, 32);
        let move_x = read_f32(record, 40);
        let move_y = read_f32(record, 44);
        let next_reference = read_u64(record, 48);
        let frame_count = read_i64(record, 56);
        let frames_offset = read_u64(record, 64);

        let name = strings.lookup(name_reference)?;
        let next_animation = strings.lookup(next_reference)?;

        let Some(name) = name else {
            let message = format!("animation record {record_index} has no name");
            return Err(TimelineError::new(message));
        };
        if !frames_offset.is_multiple_of(FRAME_RECORD_SIZE as u64) {
            let message = format!("animation {name}: misaligned frames offset {frames_offset:#x}");
            return Err(TimelineError::new(message));
        }
        let first_frame = to_usize(frames_offset / FRAME_RECORD_SIZE as u64, "frame index")?;
        let out_of_bounds = || {
            let message = format!("animation {name}: frame range out of bounds");
            TimelineError::new(message)
        };
        let Ok(frame_count) = usize::try_from(frame_count) else {
            return Err(out_of_bounds());
        };
        let last_frame = first_frame.checked_add(frame_count);
        if last_frame.is_none_or(|last| last > frame_count_total) {
            return Err(out_of_bounds());
        }

        let record = AnimationRecord {
            name,
            fps,
            length,
            looping,
            move_enabled,
            move_x,
            move_y,
            next_animation,
            first_frame,
            frame_count,
        };
        records.push(record);
        frames_offsets.push(frames_offset);
    }

    Ok(ParsedAnimations { records, frames_offsets })
}

fn fail_on_problems(check: &str, problems: &[String]) -> Result<(), TimelineError> {
    if problems.is_empty() {
        return Ok(());
    }

    let mut examples: Vec<String> = problems.iter().take(EXAMPLE_LIMIT).cloned().collect();
    if problems.len() > EXAMPLE_LIMIT {
        examples.push(format!("... {} more", problems.len() - EXAMPLE_LIMIT));
    }
    let message = format!("{check}: {} problem(s): {}", problems.len(), examples.join("; "));
    Err(TimelineError::new(message))
}

/// Every string is referenced except the empty NUL padding at the end, which keeps the table
/// size a multiple of 4.
fn check_strings(strings: &StringTable) -> Result<(), TimelineError> {
    let mut last_referenced = None;
    for (index, start) in strings.starts.iter().enumerate() {
        if strings.referenced[index] {
            last_referenced = Some(*start);
        }
    }

    let mut problems = Vec::new();
    let mut unreferenced_count = 0;
    for (index, start) in strings.starts.iter().enumerate() {
        if strings.referenced[index] {
            continue;
        }
        unreferenced_count += 1;

        let text = strings.text_of(index);
        let before_last = last_referenced.is_some_and(|last| *start < last);
        if !text.is_empty() || before_last {
            problems.push(format!("unreferenced string at {start:#x}: {text:?}"));
        }
    }
    if !strings.text.len().is_multiple_of(4) {
        problems.push(format!("table size {} is not a multiple of 4", strings.text.len()));
    }
    if unreferenced_count >= 8 {
        problems.push(format!("{unreferenced_count} trailing padding bytes"));
    }

    fail_on_problems("string table", &problems)
}

fn check_unique_names(file: &TimelineFile) -> Result<(), TimelineError> {
    let mut problems = Vec::new();

    let mut animation_names = HashSet::new();
    for animation in &file.animations {
        if !animation_names.insert(animation.name) {
            problems.push(format!("animation {} repeated", animation.name));
        }
    }
    let mut text_ids = HashSet::new();
    for text in &file.texts {
        if !text_ids.insert(text.id) {
            problems.push(format!("text {} repeated", text.id));
        }
    }

    fail_on_problems("unique ids", &problems)
}

fn check_animation_fields(file: &TimelineFile) -> Result<(), TimelineError> {
    let mut problems = Vec::new();
    for animation in &file.animations {
        let name = animation.name;
        if animation.looping != 0 && animation.looping != 1 {
            problems.push(format!("{name}: loop={}", animation.looping));
        }
        if animation.move_enabled != 0 && animation.move_enabled != 1 {
            problems.push(format!("{name}: move_enabled={}", animation.move_enabled));
        }
        if animation.length < 0 {
            problems.push(format!("{name}: length={}", animation.length));
        }
        if animation.length > 0 && animation.fps <= 0 {
            problems.push(format!("{name}: fps={}", animation.fps));
        }
        let has_move = animation.move_x != 0.0 || animation.move_y != 0.0;
        if animation.move_enabled == 0 && has_move {
            problems.push(format!("{name}: move offsets without move_enabled"));
        }
    }
    fail_on_problems("animation fields", &problems)
}

/// Animations own consecutive frame ranges in file order, and every frame record belongs to
/// exactly one animation.
fn check_frame_coverage(file: &TimelineFile, frames_offsets: &[u64]) -> Result<(), TimelineError> {
    let mut owners = vec![0u32; file.frames.len()];
    let mut problems = Vec::new();
    let mut expected_offset = 0u64;

    for (animation, frames_offset) in file.animations.iter().zip(frames_offsets) {
        if animation.frame_count != 0 && *frames_offset != expected_offset {
            problems.push(format!("{}: frames at {frames_offset:#x}, expected {expected_offset:#x}", animation.name));
        }
        let frame_bytes = (animation.frame_count * FRAME_RECORD_SIZE) as u64;
        expected_offset = frames_offset + frame_bytes;

        let end = animation.first_frame + animation.frame_count;
        for owner in &mut owners[animation.first_frame..end] {
            *owner += 1;
        }
    }
    for (record_index, owner_count) in owners.iter().enumerate() {
        if *owner_count != 1 {
            problems.push(format!("frame record {record_index} used by {owner_count} animations"));
        }
    }

    fail_on_problems("frame coverage", &problems)
}

/// Frames own consecutive pointer ranges in file order, and every pointer slot belongs to
/// exactly one frame.
fn check_pointer_coverage(file: &TimelineFile, states_offsets: &[u64]) -> Result<(), TimelineError> {
    let mut owners = vec![0u32; file.state_pointers.len()];
    let mut problems = Vec::new();
    let mut expected_offset = 0u64;

    for (record_index, (frame, states_offset)) in file.frames.iter().zip(states_offsets).enumerate() {
        if frame.state_count != 0 && *states_offset != expected_offset {
            let expected = format!("expected {expected_offset:#x}");
            let problem = format!("frame record {record_index}: pointers at {states_offset:#x}, {expected}");
            problems.push(problem);
        }
        let pointer_bytes = (frame.state_count * WORD_SIZE) as u64;
        expected_offset = states_offset + pointer_bytes;

        let end = frame.first_pointer + frame.state_count;
        for owner in &mut owners[frame.first_pointer..end] {
            *owner += 1;
        }
    }
    for (pointer_index, owner_count) in owners.iter().enumerate() {
        if *owner_count != 1 {
            problems.push(format!("pointer {pointer_index} used by {owner_count} frames"));
        }
    }

    fail_on_problems("state pointer coverage", &problems)
}

/// Every state is referenced by at least one frame.
fn check_state_coverage(file: &TimelineFile) -> Result<(), TimelineError> {
    let mut referenced = vec![false; file.states.len()];
    for frame in &file.frames {
        for state_index in file.states_of(frame) {
            referenced[*state_index as usize] = true;
        }
    }

    let mut problems = Vec::new();
    for (state_index, is_referenced) in referenced.iter().enumerate() {
        if !is_referenced {
            problems.push(format!("state {state_index} unreferenced"));
        }
    }
    fail_on_problems("state coverage", &problems)
}

/// The slots the loader fills (font def, texture def, dynamic text) are zero in the file.
fn check_load_slots(font_def_slots: &[u64], state_slots: &[(u64, u64)]) -> Result<(), TimelineError> {
    let mut problems = Vec::new();
    for (record_index, slot) in font_def_slots.iter().enumerate() {
        if *slot != 0 {
            problems.push(format!("text record {record_index}: font def slot {slot:#x}"));
        }
    }
    for (state_index, (texture_def_slot, dynamic_text_slot)) in state_slots.iter().enumerate() {
        if *texture_def_slot != 0 || *dynamic_text_slot != 0 {
            problems.push(format!("state {state_index}: pointer slots not zero"));
        }
    }
    fail_on_problems("load-time slots", &problems)
}

fn check_state_flags(file: &TimelineFile) -> Result<(), TimelineError> {
    let mut problems = Vec::new();
    for (state_index, state) in file.states.iter().enumerate() {
        let unknown = state.flags & !flags::KNOWN;
        let has_texture = state.flags & flags::TEXTURE != 0;
        let has_dynamic_text = state.flags & flags::DYNAMIC_TEXT != 0;
        let has_texture_string = state.texture.is_some();

        if unknown != 0 {
            problems.push(format!("state {state_index}: unknown flag bits {unknown:#x}"));
        }
        if has_texture && has_dynamic_text {
            problems.push(format!("state {state_index}: both texture and dynamic text flags"));
        }
        if (has_texture || has_dynamic_text) != has_texture_string {
            problems.push(format!("state {state_index}: texture string does not match flags {:#x}", state.flags));
        }
        if !(0..CHANNEL_COUNT).contains(&state.channel) {
            problems.push(format!("state {state_index}: channel {}", state.channel));
        }
    }
    fail_on_problems("state flags", &problems)
}

/// Move, transform and vertex floats are finite (colours may be NaN, §2.10).
fn check_floats(file: &TimelineFile) -> Result<(), TimelineError> {
    let mut problems = Vec::new();
    for animation in &file.animations {
        if !animation.move_x.is_finite() || !animation.move_y.is_finite() {
            problems.push(format!("animation {}: non-finite move", animation.name));
        }
    }
    for (record_index, frame) in file.frames.iter().enumerate() {
        if !frame.move_x.is_finite() || !frame.move_y.is_finite() {
            problems.push(format!("frame record {record_index}: non-finite move"));
        }
    }
    for (state_index, state) in file.states.iter().enumerate() {
        let transform = [state.x, state.y, state.rotation, state.skew_x, state.skew_y, state.scale_x, state.scale_y];
        let transform_finite = transform.iter().all(|value| value.is_finite());
        let vertices_finite = state.vertices.iter().all(|value| value.is_finite());
        if !transform_finite || !vertices_finite {
            problems.push(format!("state {state_index}: non-finite geometry"));
        }
    }
    fail_on_problems("finite geometry", &problems)
}

/// Every dynamic-text state names a text record.
fn check_dynamic_text_references(file: &TimelineFile) -> Result<(), TimelineError> {
    let text_ids: HashSet<&str> = file.texts.iter().map(|text| text.id).collect();
    let mut problems = Vec::new();
    for (state_index, state) in file.states.iter().enumerate() {
        if state.flags & flags::DYNAMIC_TEXT == 0 {
            continue;
        }
        let known = state.texture.is_some_and(|id| text_ids.contains(id));
        if !known {
            problems.push(format!("state {state_index}: {:?}", state.texture));
        }
    }
    fail_on_problems("dynamic text references", &problems)
}

fn check_justifications(file: &TimelineFile) -> Result<(), TimelineError> {
    let mut problems = Vec::new();
    for text in &file.texts {
        if !(0..=2).contains(&text.justification) {
            problems.push(format!("text {}: justification {}", text.id, text.justification));
        }
    }
    fail_on_problems("text records", &problems)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn embedded() -> &'static [u8] {
        crate::file(PATH).expect("Timelines.bin is embedded")
    }

    #[test]
    fn parses_the_shipped_file_with_documented_counts() {
        let file = load().expect("the shipped file validates");
        assert_eq!(file.string_table_size, 203_696);
        assert_eq!(file.string_count, 10_531);
        assert_eq!(file.animations.len(), 2354);
        assert_eq!(file.texts.len(), 176);
        assert_eq!(file.frames.len(), 56_175);
        assert_eq!(file.state_pointers.len(), 304_286);
        assert_eq!(file.states.len(), 81_294);

        let looping_count = file.animations.iter().filter(|animation| animation.looping != 0).count();
        assert_eq!(looping_count, 568);

        let move_count = file.animations.iter().filter(|animation| animation.move_enabled != 0).count();
        assert_eq!(move_count, 56);

        let next_count = file.animations.iter().filter(|animation| animation.next_animation.is_some()).count();
        assert_eq!(next_count, 168);

        let total_length: i64 = file.animations.iter().map(|animation| animation.length).sum();
        assert_eq!(total_length, 56_165);

        let max_states = file.frames.iter().map(|frame| frame.state_count).max();
        assert_eq!(max_states, Some(70));
    }

    #[test]
    fn first_records_match_the_reference_parser() {
        let file = load().expect("the shipped file validates");

        let text = file.texts[0];
        let text_fields = (text.id, text.text, text.font, text.justification);
        assert_eq!(text_fields, ("WhatsnewLine12", "can collect!", "Arial12", 0));

        let animation = file.animations[0];
        assert_eq!(animation.name, "AchievementGenericAnim");
        assert_eq!((animation.fps, animation.length, animation.frame_count, animation.first_frame), (20, 104, 104, 0));

        let frame = file.frames[0];
        assert_eq!((frame.index, frame.sound, frame.action), (0, Some("AchievementBell"), None));
        assert_eq!((frame.state_count, frame.first_pointer), (13, 0));
        assert_eq!(&file.states_of(&frame)[..5], &[0, 1, 2, 3, 4]);

        let state = file.states[0];
        assert_eq!(state.name, Some("AchievementBG"));
        assert_eq!(state.flags, 0x2323);
        assert_eq!(state.texture, Some("AchievementBG"));
        assert_eq!(state.channel, 0);
        assert_eq!((state.x, state.y), (165.0, 152.85));
        assert_eq!((state.scale_x, state.scale_y), (0.42, 0.42));
        assert_eq!(state.vertices[0..3], [173.82, 200.73, 0.0]);
        assert_eq!(state.vertices[9..12], [316.2, 155.37, 0.0]);
    }

    #[test]
    fn keeps_the_explained_anomalies() {
        let file = load().expect("the shipped file validates");

        let wind = file.animation("PygmyWindBlown2Anim").expect("animation exists");
        let indices: Vec<i64> = file.frames_of(wind).iter().map(|frame| frame.index).collect();
        assert_eq!(indices, [0, 1, 2, 2]);

        let cage = file.animation("PygmyGoGoCageDropAnim").expect("animation exists");
        assert_eq!(cage.next_animation, Some("PygmyDanceAnim"));
        assert!(file.animation("PygmyDanceAnim").is_none());

        let null_anim = file.animation("NullAnim").expect("animation exists");
        assert_eq!((null_anim.length, null_anim.frame_count), (0, 0));

        for state_index in [11_388, 70_085, 75_878] {
            let state = file.states[state_index];
            assert!(state.alpha.is_nan() && state.red.is_nan(), "state {state_index}");
        }
    }

    fn string_sum(text: Option<&str>) -> u64 {
        let Some(text) = text else {
            return 0;
        };
        let byte_sum: u64 = text.bytes().map(u64::from).sum();
        byte_sum + 1000 * text.len() as u64
    }

    fn weighted(sum: u64, position: usize, value: u64) -> u64 {
        let weight = position as u64 + 1;
        sum.wrapping_add(weight.wrapping_mul(value))
    }

    /// Whole-file digests, computed with the same formulas over `tools/re/timelines.py`'s
    /// parse of the shipped file (scratch script run when this parser was written).
    #[test]
    fn whole_file_digests_match_the_reference_parser() {
        let file = load().expect("the shipped file validates");

        let mut animation_digest = 0u64;
        for (position, animation) in file.animations.iter().enumerate() {
            let mut value = animation.fps as u64 * 7 + animation.length as u64 * 11;
            value += animation.looping as u64 * 13 + animation.move_enabled as u64 * 17;
            value += animation.frame_count as u64 * 19 + animation.first_frame as u64 * 23;
            value += u64::from(animation.move_x.to_bits()) + u64::from(animation.move_y.to_bits()) * 3;
            value += string_sum(Some(animation.name)) * 29 + string_sum(animation.next_animation) * 31;
            animation_digest = weighted(animation_digest, position, value);
        }

        let mut frame_digest = 0u64;
        for (position, frame) in file.frames.iter().enumerate() {
            let mut value = frame.index as u64 * 7;
            value += u64::from(frame.move_x.to_bits()) + u64::from(frame.move_y.to_bits()) * 3;
            value += frame.state_count as u64 * 11 + frame.first_pointer as u64 * 13;
            value += string_sum(frame.sound) * 17 + string_sum(frame.action) * 19;
            frame_digest = weighted(frame_digest, position, value);
        }

        let mut state_digest = 0u64;
        for (position, state) in file.states.iter().enumerate() {
            let mut value = state.flags * 7 + state.channel as u64 * 11;
            value += string_sum(state.name) * 13 + string_sum(state.texture) * 17;
            let values = [
                state.x,
                state.y,
                state.rotation,
                state.skew_x,
                state.skew_y,
                state.scale_x,
                state.scale_y,
                state.alpha,
                state.red,
                state.green,
                state.blue,
            ];
            for (index, component) in values.iter().enumerate() {
                value = value.wrapping_add(u64::from(component.to_bits()) * (index as u64 + 1));
            }
            for (index, coordinate) in state.vertices.iter().enumerate() {
                value = value.wrapping_add(u64::from(coordinate.to_bits()) * (index as u64 + 20));
            }
            state_digest = weighted(state_digest, position, value);
        }

        let mut pointer_digest = 0u64;
        for (position, state_index) in file.state_pointers.iter().enumerate() {
            pointer_digest = weighted(pointer_digest, position, u64::from(*state_index));
        }

        let mut text_digest = 0u64;
        for (position, text) in file.texts.iter().enumerate() {
            let mut value = string_sum(Some(text.id)) + string_sum(Some(text.text)) * 3;
            value += string_sum(Some(text.font)) * 5 + text.justification as u64 * 7;
            text_digest = weighted(text_digest, position, value);
        }

        assert_eq!(animation_digest, 126_146_561_885_824);
        assert_eq!(frame_digest, 203_103_118_590_311_843);
        assert_eq!(state_digest, 6_200_314_394_781_833_715);
        assert_eq!(pointer_digest, 2_589_361_148_429_684);
        assert_eq!(text_digest, 2_509_887_435);
    }

    #[test]
    fn every_dynamic_text_state_resolves() {
        let file = load().expect("the shipped file validates");
        let dynamic_count = file.states.iter().filter(|state| state.flags & flags::DYNAMIC_TEXT != 0).count();
        assert_eq!(dynamic_count, 508);
    }

    #[test]
    fn rejects_truncated_files() {
        let bytes = embedded();
        let truncated = &bytes[..bytes.len() - 1];
        let error = parse(truncated).expect_err("a truncated file is rejected");
        assert!(error.message.contains("states"), "{error}");

        let error = parse(&bytes[..4]).expect_err("a stub is rejected");
        assert!(error.message.contains("runs past the end"), "{error}");
    }

    #[test]
    fn rejects_trailing_bytes() {
        let mut bytes = embedded().to_vec();
        bytes.push(0);
        let error = parse(&bytes).expect_err("trailing bytes are rejected");
        assert!(error.message.contains("unexplained"), "{error}");
    }

    #[test]
    fn rejects_a_string_reference_into_the_middle_of_a_string() {
        let mut bytes = embedded().to_vec();
        let string_table_size = read_u64(&bytes, 0) as usize;
        let animation_data = string_table_size + 2 * WORD_SIZE;
        let first_record = animation_data + WORD_SIZE;
        let name_reference = read_u64(&bytes, first_record);
        let shifted = (name_reference + 1).to_le_bytes();
        bytes[first_record..first_record + 8].copy_from_slice(&shifted);

        let error = parse(&bytes).expect_err("a bad reference is rejected");
        assert!(error.message.contains("not the start of a string"), "{error}");
    }

    #[test]
    fn rejects_unknown_flag_bits() {
        let reference = load().expect("the shipped file validates");
        let mut bytes = embedded().to_vec();
        let states_size_offset = bytes.len() - reference.states.len() * STATE_RECORD_SIZE - WORD_SIZE;
        let flags_offset = states_size_offset + WORD_SIZE + 8;
        let flag_bits = read_u64(&bytes, flags_offset) | 0x10_0000;
        bytes[flags_offset..flags_offset + 8].copy_from_slice(&flag_bits.to_le_bytes());

        let error = parse(&bytes).expect_err("unknown flags are rejected");
        assert!(error.message.contains("unknown flag bits"), "{error}");
    }
}
