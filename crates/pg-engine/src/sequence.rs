//! `BCSequence`: the data-driven scripts (`docs/re/sequences.md`).
//!
//! A sequence is an object (entity) with a 10-entry call stack of sequence definitions and a
//! delegate. Each frame it steps items; each item names a handler method (`<method>:::`)
//! that is looked up on the delegate's class first and on `BCLibrary` otherwise.

use std::sync::Arc;

use bevy_ecs::prelude::*;

use crate::object::{Class, Isa, ObjectWorldExt, try_isa};
use crate::view;

/// One script (`BCSequenceDef`).
#[derive(Debug)]
pub struct SequenceDef {
    pub id: String,
    pub items: Vec<SequenceItemDef>,
}

pub type SequenceDefRef = Arc<SequenceDef>;

/// One item (`BCSequenceItemDef`): the handler method and the parameters present, in
/// positional order (absent parameters are omitted; `nil` in the original).
#[derive(Debug)]
pub struct SequenceItemDef {
    pub method: String,
    pub params: Vec<(String, String)>,
}

/// What a handler returns to the runner (`docs/re/sequences.md` §3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemResult {
    /// 0: not finished; call the same item next frame with `firstTime = NO`.
    Pending,

    /// 1: finished; advance but stop for this frame.
    Yield,

    /// 2: finished; advance and run the next item in the same frame.
    Done,

    /// 3: the handler replaced the sequence (`setBehavior:`); behaves like `Pending`.
    Replaced,
}

/// The arguments of a handler call: `(BCSequence *sequence, BOOL firstTime, BCSequenceItemDef *def)`.
pub struct SequenceCall<'a> {
    /// The running `BCSequence` object.
    pub sequence: Entity,
    pub first_time: bool,

    /// `[sequence deltaTime]`: the frame's dt.
    pub delta_time: f32,
    pub item: &'a SequenceItemDef,
}

impl SequenceCall<'_> {
    pub fn method(&self) -> &str {
        &self.item.method
    }

    /// The parameter as an `NSString`, `None` when absent (`nil`).
    pub fn param(&self, name: &str) -> Option<&str> {
        self.item
            .params
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn has(&self, name: &str) -> bool {
        self.param(name).is_some()
    }

    /// `[param floatValue]` (0 when absent).
    pub fn float(&self, name: &str) -> f32 {
        match self.param(name) {
            Some(value) => ns_float_value(value),
            None => 0.0,
        }
    }

    /// `[param intValue]` (0 when absent).
    pub fn int(&self, name: &str) -> i32 {
        match self.param(name) {
            Some(value) => ns_int_value(value),
            None => 0,
        }
    }

    /// `[param boolValue]` (NO when absent).
    pub fn bool(&self, name: &str) -> bool {
        match self.param(name) {
            Some(value) => ns_bool_value(value),
            None => false,
        }
    }

    /// `[param componentsSeparatedByString:@","]` (no trimming); empty when absent.
    pub fn list(&self, name: &str) -> Vec<&str> {
        match self.param(name) {
            Some(value) => value.split(',').collect(),
            None => Vec::new(),
        }
    }
}

/// `-[NSString floatValue]`: leading whitespace, then the longest decimal prefix; 0 if none.
pub fn ns_float_value(text: &str) -> f32 {
    let trimmed = text.trim_start();
    let mut end = 0;
    let bytes = trimmed.as_bytes();
    let mut seen_digit = false;
    let mut seen_dot = false;
    let mut seen_exponent = false;
    while end < bytes.len() {
        let byte = bytes[end];
        let is_sign = (byte == b'-' || byte == b'+') && (end == 0 || matches!(bytes[end - 1], b'e' | b'E'));
        if byte.is_ascii_digit() {
            seen_digit = true;
        } else if byte == b'.' && !seen_dot && !seen_exponent {
            seen_dot = true;
        } else if (byte == b'e' || byte == b'E') && seen_digit && !seen_exponent {
            seen_exponent = true;
        } else if !is_sign {
            break;
        }
        end += 1;
    }
    let mut candidate = &trimmed[..end];
    while !candidate.is_empty() {
        if let Ok(value) = candidate.parse::<f32>() {
            return value;
        }
        candidate = &candidate[..candidate.len() - 1];
    }
    0.0
}

/// `-[NSString intValue]`: leading whitespace, optional sign, decimal digits; 0 if none;
/// saturates like the original.
pub fn ns_int_value(text: &str) -> i32 {
    let trimmed = text.trim_start();
    let mut chars = trimmed.chars().peekable();
    let mut negative = false;
    if let Some(&sign) = chars.peek() {
        if sign == '-' || sign == '+' {
            negative = sign == '-';
            chars.next();
        }
    }
    let mut value: i64 = 0;
    for ch in chars {
        let Some(digit) = ch.to_digit(10) else {
            break;
        };
        value = (value * 10 + digit as i64).min(i64::from(i32::MAX) + 1);
    }
    let signed = if negative { -value } else { value };
    signed.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

/// `-[NSString boolValue]`: YES when the first non-blank character (after an optional sign
/// and leading zeros) is `Y`, `y`, `T`, `t` or a digit `1`–`9`.
pub fn ns_bool_value(text: &str) -> bool {
    let trimmed = text.trim_start();
    let unsigned = trimmed.strip_prefix(['+', '-']).unwrap_or(trimmed);
    let significant = unsigned.trim_start_matches('0');
    match significant.chars().next() {
        Some(first) => matches!(first, 'Y' | 'y' | 'T' | 't' | '1'..='9'),
        None => false,
    }
}

/// `BCSequenceStackItem`.
#[derive(Clone, Debug, Default)]
pub struct StackItem {
    pub def: Option<SequenceDefRef>,
    pub item_index: i32,
}

pub const STACK_SIZE: usize = 10;

/// `BCSequence` ivars.
#[derive(Component, Debug, Default)]
pub struct BcSequence {
    pub stack: [StackItem; STACK_SIZE],
    pub stack_index: i32,
    pub delegate: Option<Entity>,
    pub delta_time: f32,
    pub debug: bool,
    pub first_time: bool,
    pub is_running: bool,
    pub hibernate: bool,
    pub hibernate_game_frame_running: bool,
}

pub const BC_SEQUENCE: Class = Class {
    name: "BCSequence",
    superclass: None,
    game_frame_order: 10,
    on_game_frame,
    ..Class::NS_OBJECT
};

fn same_def(a: &Option<SequenceDefRef>, b: &Option<SequenceDefRef>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

/// `-initWithSequenceDef:delegate:` @ 0x1000bff20. Plays when `def` is non-NULL.
pub fn spawn(world: &mut World, def: Option<SequenceDefRef>, delegate: Option<Entity>) -> Entity {
    let has_def = def.is_some();
    let mut sequence = BcSequence {
        delegate,
        first_time: true,
        ..Default::default()
    };
    sequence.stack[0] = StackItem { def, item_index: 0 };

    let entity = world.spawn((Isa(&BC_SEQUENCE), sequence)).id();
    if has_def {
        play(world, entity);
    }
    entity
}

/// `-deltaTime` @ 0x1000c0018.
pub fn delta_time(world: &World, sequence: Entity) -> f32 {
    world.obj::<BcSequence>(sequence).delta_time
}

/// `-hibernate` @ 0x1000c0028.
pub fn hibernate(world: &mut World, sequence: Entity) {
    let mut state = world.obj_mut::<BcSequence>(sequence);
    if state.hibernate {
        return;
    }
    state.hibernate = true;
    state.hibernate_game_frame_running = state.is_running;
    let was_running = state.is_running;
    state.is_running = false;
    if was_running {
        view::remove_game_frame_listener(world, sequence);
    }
}

/// `-unhibernate` @ 0x1000c00b4.
pub fn unhibernate(world: &mut World, sequence: Entity) {
    let mut state = world.obj_mut::<BcSequence>(sequence);
    if !state.hibernate {
        return;
    }
    state.hibernate = false;
    let resume = state.hibernate_game_frame_running;
    if resume {
        play(world, sequence);
    }
}

/// `-setSequenceDef:` @ 0x1000c00ec: replace the whole stack and restart at item 0.
pub fn set_sequence_def(world: &mut World, sequence: Entity, def: Option<SequenceDefRef>) {
    let has_def = def.is_some();
    {
        let mut state = world.obj_mut::<BcSequence>(sequence);
        state.first_time = true;
        state.stack_index = 0;
        state.stack[0] = StackItem { def, item_index: 0 };
    }
    if has_def {
        play(world, sequence);
    } else {
        stop(world, sequence);
    }
}

/// `-pushSequenceDef:` @ 0x1000c0148: call `def`, returning to the next item afterwards.
pub fn push_sequence_def(world: &mut World, sequence: Entity, def: Option<SequenceDefRef>) {
    let Some(def) = def else {
        return;
    };
    {
        let mut state = world.obj_mut::<BcSequence>(sequence);
        if state.stack_index != 0 || state.stack[0].def.is_some() {
            state.stack_index += 1;
        }
        // The original has no bound check; real scripts never exceed depth 3.
        let index = state.stack_index.clamp(0, STACK_SIZE as i32 - 1) as usize;
        state.stack_index = index as i32;
        state.first_time = true;
        state.stack[index] = StackItem {
            def: Some(def),
            item_index: 0,
        };
    }
    play(world, sequence);
}

/// `-play` @ 0x1000c01c4.
pub fn play(world: &mut World, sequence: Entity) {
    let mut state = world.obj_mut::<BcSequence>(sequence);
    if state.hibernate {
        state.hibernate_game_frame_running = true;
        return;
    }
    let index = state.stack_index as usize;
    let has_def = state.stack[index].def.is_some();
    if state.is_running || !has_def {
        return;
    }
    state.is_running = true;
    view::add_game_frame_listener(world, sequence);
}

/// `-stop` @ 0x1000c0274.
pub fn stop(world: &mut World, sequence: Entity) {
    let mut state = world.obj_mut::<BcSequence>(sequence);
    if state.hibernate {
        state.hibernate_game_frame_running = false;
        return;
    }
    if !state.is_running {
        return;
    }
    state.is_running = false;
    view::remove_game_frame_listener(world, sequence);
}

fn top_def(world: &World, sequence: Entity) -> Option<SequenceDefRef> {
    let state = world.obj::<BcSequence>(sequence);
    let index = state.stack_index as usize;
    state.stack[index].def.clone()
}

/// `-onGameFrame:` @ 0x1000c0520: the stack driver (§3.4).
pub fn on_game_frame(world: &mut World, sequence: Entity, dt: f32) {
    let Some(state) = world.get::<BcSequence>(sequence) else {
        return;
    };
    if state.hibernate || !state.is_running {
        return;
    }

    loop {
        let mut before;
        let mut after;
        loop {
            before = top_def(world, sequence);
            run_sequence(world, sequence, dt);
            after = top_def(world, sequence);
            let switched = !same_def(&before, &after) && after.is_some();
            if !switched {
                break;
            }
        }

        let (stack_index, item_index) = {
            let state = world.obj::<BcSequence>(sequence);
            let index = state.stack_index as usize;
            (state.stack_index, state.stack[index].item_index)
        };
        let before_count = before.as_ref().map_or(0, |def| def.items.len() as i32);
        let done = after.is_none() || item_index >= before_count;

        if stack_index >= 1 {
            if !done {
                return;
            }
            world.obj_mut::<BcSequence>(sequence).stack_index -= 1;
            continue;
        }

        if done {
            world.obj_mut::<BcSequence>(sequence).stack[0] = StackItem::default();
            stop(world, sequence);
        }
        return;
    }
}

/// `-runSequence:` @ 0x1000c02f8: step items of the top entry (§3.5).
fn run_sequence(world: &mut World, sequence: Entity, dt: f32) {
    world.obj_mut::<BcSequence>(sequence).delta_time = dt;

    loop {
        let (index, entry, first_time, delegate) = {
            let state = world.obj::<BcSequence>(sequence);
            let index = state.stack_index;
            let entry = state.stack[index as usize].clone();
            (index, entry, state.first_time, state.delegate)
        };
        let Some(def) = entry.def.clone() else {
            return;
        };
        if entry.item_index < 0 || entry.item_index as usize >= def.items.len() {
            return;
        }

        let item = &def.items[entry.item_index as usize];
        let mut call = SequenceCall {
            sequence,
            first_time,
            delta_time: dt,
            item,
        };
        let result = dispatch(world, delegate, &mut call);

        let mut state = world.obj_mut::<BcSequence>(sequence);
        if state.stack_index != index {
            state.stack[index as usize].item_index += 1;
            state.first_time = true;
            return;
        }
        let current = &state.stack[index as usize];
        if !same_def(&current.def, &entry.def) {
            return;
        }
        if current.item_index != entry.item_index {
            return;
        }
        state.first_time = false;

        let top = state.stack_index as usize;
        match result {
            ItemResult::Pending | ItemResult::Replaced => return,
            ItemResult::Yield => {
                state.stack[top].item_index += 1;
                state.first_time = true;
                return;
            }
            ItemResult::Done => {
                state.stack[top].item_index += 1;
                state.first_time = true;
            }
        }
    }
}

/// §3.7: the delegate's class (respondsToSelector:) first, then `BCLibrary`.
fn dispatch(world: &mut World, delegate: Option<Entity>, call: &mut SequenceCall) -> ItemResult {
    if let Some(delegate) = delegate {
        if let Some(class) = try_isa(world, delegate) {
            if let Some(result) = (class.sequence_item)(world, delegate, call) {
                return result;
            }
        }
    }

    match crate::library::handlers::sequence_item(world, call) {
        Some(result) => result,
        None => {
            // The original throws "unrecognized selector"; shipped data never reaches this.
            bevy_log_error(&call.item.method);
            ItemResult::Done
        }
    }
}

fn bevy_log_error(method: &str) {
    eprintln!("pg-engine: no handler for sequence item `{method}:::`");
}
