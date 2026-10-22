//! `BCAnimation` (the player) and the parsed `Timelines.bin` (`docs/re/timelines.md`).
//!
//! A player is an object (entity) carrying [`BcAnimation`]. Its definition is a record of
//! [`Timelines`]; the playback methods of `BCBinaryAnimationDef` (`applyFrame…`,
//! `applySoundAndAction…`, `applyMovement…`) are ported here as private functions over those
//! records. A nil `mAnimationDef` is `None` and answers 0 / NO / nil, like a message to nil.
//! Listener calls (`BCAnimationListener`) go to the listener logic through `crate::logic`.

use bevy_ecs::prelude::*;
use bevy_math::Vec4;

use crate::display::{self, DisplayObject};
use crate::library::Library;
use crate::math::trunc_i32;
use crate::object::{Class, Isa, ObjectWorldExt};
use crate::texture::TextureDefId;
use crate::view;

/// Index of an animation in [`Timelines::animations`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationDefId(pub u32);

/// Index of a frame record in [`Timelines::frames`] (a `BinaryFrameDef*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameRef(pub u32);

/// `BinaryAnimationDef`.
#[derive(Clone, Debug)]
pub struct TimelineAnimation {
    pub name: String,
    pub fps: i32,
    pub length: i32,
    pub looping: bool,
    pub move_enabled: bool,
    pub move_x: f32,
    pub move_y: f32,
    pub next_animation: Option<String>,

    /// Frame records `first_frame .. first_frame + frame_count` in [`Timelines::frames`].
    pub first_frame: u32,
    pub frame_count: u32,
}

/// `BinaryFrameDef`.
#[derive(Clone, Debug)]
pub struct TimelineFrame {
    pub index: i32,
    pub sound: Option<String>,
    pub action: Option<String>,
    pub move_x: f32,
    pub move_y: f32,

    /// Draw-ordered states: `state_lists[first_state .. first_state + state_count]`.
    pub first_state: u32,
    pub state_count: u32,
}

/// Frame-state flag bits (timelines.md §2.8).
pub mod state_flags {
    pub const X: u32 = 0x0001;
    pub const Y: u32 = 0x0002;
    pub const ROTATION: u32 = 0x0020;
    pub const SKEW_X: u32 = 0x0040;
    pub const SKEW_Y: u32 = 0x0080;
    pub const SCALE_X: u32 = 0x0100;
    pub const SCALE_Y: u32 = 0x0200;
    pub const ALPHA: u32 = 0x0800;
    pub const COLOR: u32 = 0x1000;
    pub const TEXTURE: u32 = 0x2000;
    pub const DYNAMIC_TEXT: u32 = 0x4000;
}

/// One frame-state record, with its texture / dynamic text resolved at load.
#[derive(Clone, Debug)]
pub struct TimelineState {
    pub name: String,
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

/// Every timeline animation (`BCLibrary::parseBinAnim:ext:`).
#[derive(Resource, Default)]
pub struct Timelines {
    pub animations: Vec<TimelineAnimation>,
    pub frames: Vec<TimelineFrame>,
    pub state_lists: Vec<u32>,
    pub states: Vec<TimelineState>,
}

impl Timelines {
    pub fn animation(&self, id: AnimationDefId) -> &TimelineAnimation {
        &self.animations[id.0 as usize]
    }

    pub fn frame(&self, frame: FrameRef) -> &TimelineFrame {
        &self.frames[frame.0 as usize]
    }

    /// The draw-ordered state indices of a frame.
    pub fn frame_states(&self, frame: FrameRef) -> &[u32] {
        let record = self.frame(frame);
        let start = record.first_state as usize;
        let end = start + record.state_count as usize;
        &self.state_lists[start..end]
    }

    pub fn state(&self, index: u32) -> &TimelineState {
        &self.states[index as usize]
    }
}

/// `TweenableChannel`.
#[derive(Clone, Copy, Debug, Default)]
pub struct TweenableChannel {
    pub multi_model_mode: bool,
    pub enable_multi_model_mode: bool,
    pub release_next_frame: bool,
    pub tweenable: Option<Entity>,
}

pub const CHANNEL_COUNT: usize = 30;

/// `BCAnimation` ivars.
#[derive(Component, Clone, Debug)]
pub struct BcAnimation {
    pub listener: Option<Entity>,
    pub def: Option<AnimationDefId>,
    pub debug: bool,
    pub frame_index: i32,
    pub anim_time: f32,
    pub use_frame_percent: bool,
    pub frame_percent: f32,
    pub is_complete: bool,
    pub is_running: bool,
    pub hibernate: bool,
    pub hibernate_game_frame_running: bool,
    pub channels: [TweenableChannel; CHANNEL_COUNT],
}

impl Default for BcAnimation {
    fn default() -> Self {
        Self {
            listener: None,
            def: None,
            debug: false,
            frame_index: -1,
            anim_time: 0.0,
            use_frame_percent: false,
            frame_percent: 0.0,
            is_complete: false,
            is_running: false,
            hibernate: false,
            hibernate_game_frame_running: false,
            channels: [TweenableChannel::default(); CHANNEL_COUNT],
        }
    }
}

pub const BC_ANIMATION: Class = Class {
    name: "BCAnimation",
    superclass: None,
    game_frame_order: 20,
    on_game_frame,
    id,
    ..Class::NS_OBJECT
};

/// The state flags that make `applyFrame…` move a channel's tweenable.
const TRANSFORM_FLAGS: u32 =
    state_flags::X | state_flags::Y | state_flags::ROTATION | state_flags::SCALE_X | state_flags::SCALE_Y;

/// `-[BCAnimation init]` @ 0x10019fc8c.
pub fn spawn(world: &mut World) -> Entity {
    world.spawn((Isa(&BC_ANIMATION), BcAnimation::default())).id()
}

/// `-setAnimationListener:` @ 0x10019fd40.
pub fn set_animation_listener(world: &mut World, animation: Entity, listener: Option<Entity>) {
    world.obj_mut::<BcAnimation>(animation).listener = listener;
}

/// `-setDebug:` @ 0x10019fd9c.
pub fn set_debug(world: &mut World, animation: Entity, debug: bool) {
    world.obj_mut::<BcAnimation>(animation).debug = debug;
}

/// `-id` @ 0x1001a047c: the listener's id (debug output only).
pub fn id(world: &World, animation: Entity) -> Option<String> {
    let listener = world.obj::<BcAnimation>(animation).listener?;
    crate::logic::id(world, listener)
}

/// `-setAnimationDef:` @ 0x10019ff54 (keeps the frame index; unused by game code).
pub fn set_animation_def(world: &mut World, animation: Entity, def: Option<AnimationDefId>) {
    let mut player = world.obj_mut::<BcAnimation>(animation);
    player.anim_time = 0.0;
    player.def = def;
}

/// `-playAnimationId:` @ 0x1001a02f8.
pub fn play_animation_id(world: &mut World, animation: Entity, id: Option<&str>) {
    let library = world.resource::<Library>();
    let def = match id {
        Some(id) => library.animation_def(id),
        None => None,
    };
    {
        let mut player = world.obj_mut::<BcAnimation>(animation);
        player.use_frame_percent = false;
        player.def = def;
    }
    reset(world, animation);

    let def = world.obj::<BcAnimation>(animation).def;
    let length = def_length(world, def);
    if length == 0 {
        world.obj_mut::<BcAnimation>(animation).is_complete = true;
        stop(world, animation);
        return;
    }

    play(world, animation);
    let def = world.obj::<BcAnimation>(animation).def;
    if !def_is_move_enabled(world, def) {
        return;
    }
    let listener = world.obj::<BcAnimation>(animation).listener;
    if let Some(listener) = listener {
        crate::logic::on_animation_move_enabled(world, listener);
    }
}

/// `-setFramePercent:` @ 0x1001a0430.
pub fn set_frame_percent(world: &mut World, animation: Entity, percent: f32) {
    let was_running = {
        let mut player = world.obj_mut::<BcAnimation>(animation);
        player.use_frame_percent = true;
        player.frame_percent = percent;
        player.is_running
    };
    if was_running {
        return;
    }
    world.obj_mut::<BcAnimation>(animation).is_complete = false;
    play(world, animation);
}

/// `-play` @ 0x1001a01e8.
pub fn play(world: &mut World, animation: Entity) {
    let mut player = world.obj_mut::<BcAnimation>(animation);
    if player.hibernate {
        player.hibernate_game_frame_running = true;
        return;
    }
    if player.is_running {
        return;
    }
    player.is_running = true;
    view::add_game_frame_listener(world, animation);
}

/// `-stop` @ 0x1001a0274.
pub fn stop(world: &mut World, animation: Entity) {
    let mut player = world.obj_mut::<BcAnimation>(animation);
    if player.hibernate {
        player.hibernate_game_frame_running = false;
        return;
    }
    if !player.is_running {
        return;
    }
    player.is_running = false;
    view::remove_game_frame_listener(world, animation);
}

/// `-reset` @ 0x1001a0144.
pub fn reset(world: &mut World, animation: Entity) {
    let mut player = world.obj_mut::<BcAnimation>(animation);
    player.anim_time = 0.0;
    player.frame_index = -1;
    player.is_complete = false;
}

/// `-replayCurrentFrame` @ 0x1001a0170.
pub fn replay_current_frame(world: &mut World, animation: Entity) {
    let (is_complete, def) = {
        let player = world.obj::<BcAnimation>(animation);
        (player.is_complete, player.def)
    };
    if !is_complete {
        return;
    }
    let length = def_length(world, def);
    {
        let mut player = world.obj_mut::<BcAnimation>(animation);
        player.frame_index = length.wrapping_sub(2);
        player.is_complete = false;
    }
    play(world, animation);
}

/// `-isHibernating` @ 0x10019ffbc.
pub fn is_hibernating(world: &World, animation: Entity) -> bool {
    world.get::<BcAnimation>(animation).is_some_and(|player| player.hibernate)
}

/// `-hibernate` @ 0x10019ffcc.
pub fn hibernate(world: &mut World, animation: Entity) {
    let was_running = {
        let mut player = world.obj_mut::<BcAnimation>(animation);
        if player.hibernate {
            return;
        }
        player.hibernate = true;
        let was_running = player.is_running;
        player.hibernate_game_frame_running = was_running;
        player.is_running = false;
        was_running
    };
    if was_running {
        view::remove_game_frame_listener(world, animation);
    }

    for index in 0..CHANNEL_COUNT {
        let tweenable = world.obj::<BcAnimation>(animation).channels[index].tweenable;
        if let Some(tweenable) = tweenable {
            display::hibernate(world, tweenable);
        }
    }
}

/// `-unhibernate` @ 0x1001a0090.
pub fn unhibernate(world: &mut World, animation: Entity) {
    let resume = {
        let mut player = world.obj_mut::<BcAnimation>(animation);
        if !player.hibernate {
            return;
        }
        player.hibernate = false;
        player.hibernate_game_frame_running
    };
    if resume {
        play(world, animation);
    }

    for index in 0..CHANNEL_COUNT {
        let tweenable = world.obj::<BcAnimation>(animation).channels[index].tweenable;
        if let Some(tweenable) = tweenable {
            display::unhibernate(world, tweenable);
        }
    }
}

/// `-setTweenable:channelId:` @ 0x10019fdac.
pub fn set_tweenable(world: &mut World, animation: Entity, tweenable: Option<Entity>, channel: usize) {
    let current = world.obj::<BcAnimation>(animation).channels[channel];
    if current.multi_model_mode {
        world.obj_mut::<BcAnimation>(animation).channels[channel].multi_model_mode = false;
        if let Some(previous) = current.tweenable {
            display::disable_multi_model_mode(world, previous);
        }
    }

    // The new tweenable gets a wake reference; the replaced one keeps its own (it is not
    // sent `hibernate`).
    let hibernating = world.obj::<BcAnimation>(animation).hibernate;
    if !hibernating && let Some(tweenable) = tweenable {
        display::unhibernate(world, tweenable);
    }

    {
        let mut player = world.obj_mut::<BcAnimation>(animation);
        let entry = &mut player.channels[channel];
        entry.tweenable = tweenable;
        entry.enable_multi_model_mode = false;
        entry.release_next_frame = false;
    }
    replay_current_frame(world, animation);
}

/// `-clearTweenableChannelId:` @ 0x10019fe74 (released at the end of the next `applyFrame…`).
pub fn clear_tweenable_channel_id(world: &mut World, animation: Entity, channel: usize) {
    world.obj_mut::<BcAnimation>(animation).channels[channel].release_next_frame = true;
    replay_current_frame(world, animation);
}

/// `-clearTweenableChannelIdNow:` @ 0x10019fe9c.
pub fn clear_tweenable_channel_id_now(world: &mut World, animation: Entity, channel: usize) {
    unbind_channel(world, animation, channel);
    replay_current_frame(world, animation);
}

/// `-isAnimationComplete` @ 0x1001a0134.
pub fn is_animation_complete(world: &World, animation: Entity) -> bool {
    world.get::<BcAnimation>(animation).is_some_and(|player| player.is_complete)
}

/// `-frameIndex` @ 0x1001a0124.
pub fn frame_index(world: &World, animation: Entity) -> i32 {
    world.get::<BcAnimation>(animation).map_or(0, |player| player.frame_index)
}

/// `-dealloc` @ 0x1001a06b4: unbinds every channel, stops, and destroys the player.
pub fn dealloc(world: &mut World, animation: Entity) {
    for index in 0..CHANNEL_COUNT {
        let tweenable = world.obj::<BcAnimation>(animation).channels[index].tweenable;
        if tweenable.is_some() {
            unbind_channel(world, animation, index);
        }
    }
    stop(world, animation);
    world.despawn(animation);
}

/// `-onGameFrame:` @ 0x1001a04a0 (exact algorithm incl. re-entrancy, timelines.md §4.3–§4.6).
pub fn on_game_frame(world: &mut World, animation: Entity, dt: f32) {
    let Some(player) = world.get::<BcAnimation>(animation) else {
        return;
    };
    if !player.is_running {
        return;
    }

    let (anim_time, def, use_frame_percent, frame_percent) = {
        let mut player = world.obj_mut::<BcAnimation>(animation);
        player.anim_time += dt;
        (player.anim_time, player.def, player.use_frame_percent, player.frame_percent)
    };
    let fps = def_fps(world, def);
    let length = def_length(world, def);
    let raw_frame = raw_frame_number(anim_time, fps, length, use_frame_percent, frame_percent);
    let looping = def_loop(world, def);
    let frame = displayed_frame_number(raw_frame, length, looping);

    let last_frame = world.obj::<BcAnimation>(animation).frame_index;
    if frame != last_frame {
        let (frame_def, listener) = def_and_listener(world, animation);
        if let Some(frame_def) = frame_def {
            apply_frame(world, frame_def, frame, last_frame, animation, listener);
        }

        // A frame action may have restarted this player: the movement goes to the current
        // definition and listener, and the frame index written is still this tick's frame.
        let (movement_def, listener) = def_and_listener(world, animation);
        if let Some(movement_def) = movement_def {
            apply_movement(world, movement_def, raw_frame, listener);
        }
        world.obj_mut::<BcAnimation>(animation).frame_index = frame;
    }

    // `looping`, `length` and `frame` stay the values computed before `applyFrame…`.
    let use_frame_percent = world.obj::<BcAnimation>(animation).use_frame_percent;
    let reached_last_frame = frame >= length.wrapping_sub(1);
    if looping || use_frame_percent || !reached_last_frame {
        return;
    }
    world.obj_mut::<BcAnimation>(animation).is_complete = true;
    stop(world, animation);

    let def = world.obj::<BcAnimation>(animation).def;
    let next_animation = def_next_animation_id(world, def);
    if let Some(next_animation) = next_animation {
        play_animation_id(world, animation, Some(&next_animation));
    }
}

fn def_and_listener(world: &World, animation: Entity) -> (Option<AnimationDefId>, Option<Entity>) {
    let player = world.obj::<BcAnimation>(animation);
    (player.def, player.listener)
}

/// The record behind a player's `mAnimationDef` (`None` for nil).
fn def_record(world: &World, def: Option<AnimationDefId>) -> Option<&TimelineAnimation> {
    let def = def?;
    let timelines = world.resource::<Timelines>();
    Some(timelines.animation(def))
}

/// `-[BCBinaryAnimationDef fps]` @ 0x1001a0acc (0 for nil).
fn def_fps(world: &World, def: Option<AnimationDefId>) -> i32 {
    def_record(world, def).map_or(0, |record| record.fps)
}

/// `-[BCBinaryAnimationDef length]` @ 0x1001a0ae0 (0 for nil).
fn def_length(world: &World, def: Option<AnimationDefId>) -> i32 {
    def_record(world, def).map_or(0, |record| record.length)
}

/// `-[BCBinaryAnimationDef loop]` @ 0x1001a0af8 (NO for nil).
fn def_loop(world: &World, def: Option<AnimationDefId>) -> bool {
    def_record(world, def).is_some_and(|record| record.looping)
}

/// `-[BCBinaryAnimationDef isMoveEnabled]` @ 0x1001a0b18 (NO for nil).
fn def_is_move_enabled(world: &World, def: Option<AnimationDefId>) -> bool {
    def_record(world, def).is_some_and(|record| record.move_enabled)
}

/// `-[BCBinaryAnimationDef nextAnimationId]` @ 0x1001a0b68 (nil for nil).
fn def_next_animation_id(world: &World, def: Option<AnimationDefId>) -> Option<String> {
    let record = def_record(world, def)?;
    record.next_animation.clone()
}

/// The unwrapped frame number of `onGameFrame:`: `trunc(t·fps)`, or `trunc(length·p + 0.5)`
/// while scrubbing (`fcvtzs`; separate multiply and add, no fused multiply-add).
fn raw_frame_number(anim_time: f32, fps: i32, length: i32, use_frame_percent: bool, frame_percent: f32) -> i32 {
    if use_frame_percent {
        let scaled = length as f32 * frame_percent;
        trunc_i32(scaled + 0.5)
    } else {
        let scaled = anim_time * fps as f32;
        trunc_i32(scaled)
    }
}

/// The frame shown for a raw frame number: C `%` when looping (ARM `sdiv` by zero gives 0,
/// so a zero length leaves the number unchanged), clamped to `length - 1` otherwise.
fn displayed_frame_number(raw_frame: i32, length: i32, looping: bool) -> i32 {
    if looping {
        let cycles = if length == 0 { 0 } else { raw_frame.wrapping_div(length) };
        raw_frame.wrapping_sub(cycles.wrapping_mul(length))
    } else if raw_frame < length {
        raw_frame
    } else {
        length.wrapping_sub(1)
    }
}

/// The outcome of the frame-record scan of `applyFrame…`.
#[derive(Debug, Default, PartialEq)]
struct FrameScan {
    /// Records passed on the way to the chosen one, whose sounds and actions fire first.
    passed: Vec<FrameRef>,

    /// The last record whose index is at most the frame shown.
    chosen: Option<FrameRef>,
}

/// The record scan of `applyFrame…`. It depends only on the immutable records, so firing
/// the passed records after it keeps the original's order of events.
fn scan_frames(timelines: &Timelines, def: AnimationDefId, frame: i32, last_frame: i32) -> FrameScan {
    let animation = timelines.animation(def);
    let mut scan = FrameScan::default();
    if (animation.frame_count as i32) < 1 {
        return scan;
    }

    let first_unplayed = last_frame.wrapping_add(1);
    let mut previous: Option<FrameRef> = None;
    for offset in 0..animation.frame_count {
        let record = FrameRef(animation.first_frame + offset);
        let index = timelines.frame(record).index;
        if index > first_unplayed && index <= frame && let Some(previous) = previous {
            scan.passed.push(previous);
        }
        if index > frame {
            break;
        }
        previous = Some(record);
    }
    scan.chosen = previous;
    scan
}

/// `-[BCBinaryAnimationDef applyFrame:lastFrame:animation:animationListener:tweenableChannelArray:]` @ 0x1001a0ca4.
fn apply_frame(
    world: &mut World,
    def: AnimationDefId,
    frame: i32,
    last_frame: i32,
    animation: Entity,
    listener: Option<Entity>,
) {
    let timelines = world.resource::<Timelines>();
    let scan = scan_frames(timelines, def, frame, last_frame);
    for passed in scan.passed {
        apply_sound_and_action(world, passed, listener);
    }

    if let Some(chosen) = scan.chosen {
        apply_sound_and_action(world, chosen, listener);
        if let Some(listener) = listener {
            crate::logic::set_binary_frame_def(world, listener, Some(chosen), Some(animation));
        }

        let timelines = world.resource::<Timelines>();
        let channel_states = channel_states_of(timelines, chosen);
        for state in &channel_states {
            apply_channel_state(world, animation, listener, state);
        }
    }

    update_channels(world, animation);
}

/// `-[BCBinaryAnimationDef applySoundAndAction:animationListener:]` @ 0x1001a0bb4.
fn apply_sound_and_action(world: &mut World, frame: FrameRef, listener: Option<Entity>) {
    let (sound, action) = {
        let timelines = world.resource::<Timelines>();
        let record = timelines.frame(frame);
        (record.sound.clone(), record.action.clone())
    };

    if let Some(sound) = sound {
        let sound_disabled = match listener {
            Some(listener) => crate::logic::is_sound_disabled(world, listener),
            None => false,
        };
        if !sound_disabled {
            crate::sound::play_sound(world, &sound);
        }
    }

    let Some(action) = action else {
        return;
    };
    let Some(listener) = listener else {
        return;
    };
    let responded = crate::object::perform(world, Some(listener), &action);
    if !responded {
        // The original raises "unrecognized selector"; shipped animations only name actions
        // that their listeners implement.
        eprintln!("pg-engine: animation listener {listener} does not respond to frame action `{action}`");
    }
}

/// The fields of a channel-bound frame state that `applyFrame…` reads.
#[derive(Clone, Copy, Debug)]
struct ChannelState {
    channel: usize,
    flags: u32,
    x: f32,
    y: f32,
    rotation: f32,
    scale_x: f32,
    scale_y: f32,
    texture: Option<TextureDefId>,
}

/// The states of `frame` that name a channel (`channel != 0`), in draw order.
fn channel_states_of(timelines: &Timelines, frame: FrameRef) -> Vec<ChannelState> {
    let mut channel_states = Vec::new();
    for &state_index in timelines.frame_states(frame) {
        let state = timelines.state(state_index);
        if state.channel == 0 {
            continue;
        }
        let channel_state = ChannelState {
            channel: state.channel as usize,
            flags: state.flags,
            x: state.x,
            y: state.y,
            rotation: state.rotation,
            scale_x: state.scale_x,
            scale_y: state.scale_y,
            texture: state.texture,
        };
        channel_states.push(channel_state);
    }
    channel_states
}

/// One channel-bound state of the chosen frame: marks the channel as shown by this frame,
/// places its tweenable relative to the listener's `parentTweenable` and hands it the texture.
fn apply_channel_state(world: &mut World, animation: Entity, listener: Option<Entity>, state: &ChannelState) {
    let tweenable = world.obj::<BcAnimation>(animation).channels[state.channel].tweenable;
    let Some(tweenable) = tweenable else {
        return;
    };
    world.obj_mut::<BcAnimation>(animation).channels[state.channel].enable_multi_model_mode = true;

    if state.flags & TRANSFORM_FLAGS != 0 {
        let parent = parent_transform(world, listener);
        // The channel retains its tweenable in the original, so a despawned one cannot occur
        // there; like the `display` dispatchers, writing to it is skipped.
        if let Some(mut target) = world.get_mut::<DisplayObject>(tweenable) {
            transform_tweenable(&mut target, state, &parent);
        }
    }

    if let Some(texture) = state.texture {
        display::set_texture_def(world, tweenable, Some(texture));
    }
}

/// The `parentTweenable` values the channel transform reads.
#[derive(Clone, Copy, Debug)]
struct ParentTransform {
    pos: Vec4,
    rot_z: f32,
    scale: Vec4,
    origin_pos: Vec4,
}

/// `[listener parentTweenable]` (the listener logic's display object) and its vectors.
fn parent_transform(world: &World, listener: Option<Entity>) -> ParentTransform {
    // `[nil parentTweenable]` is nil and the original then reads through NULL vectors.
    let listener = listener.expect(
        "BCAnimation: a channel state moves its tweenable relative to the listener, but the player has none",
    );
    let parent = crate::logic::parent_tweenable(world, listener);
    let object = display::object(world, parent);
    ParentTransform {
        pos: object.pos,
        rot_z: object.rot.z,
        scale: object.scale,
        origin_pos: object.origin_pos,
    }
}

/// `world = (local + origin) · scale + pos` per axis; the parent's pivot, skew, depth and
/// rotation are ignored, and a parent with `scale.x` not above 0 (or NaN) mirrors the rotation.
fn transform_tweenable(target: &mut DisplayObject, state: &ChannelState, parent: &ParentTransform) {
    let flags = state.flags;
    if flags & state_flags::X != 0 {
        let local_x = state.x + parent.origin_pos.x;
        target.pos.x = local_x * parent.scale.x + parent.pos.x;
    }
    if flags & state_flags::Y != 0 {
        let local_y = state.y + parent.origin_pos.y;
        target.pos.y = local_y * parent.scale.y + parent.pos.y;
    }
    if flags & state_flags::ROTATION != 0 {
        target.rot.z = if parent.scale.x > 0.0 {
            state.rotation + parent.rot_z
        } else if flags & state_flags::SCALE_X != 0 {
            (180.0 - state.rotation) + parent.rot_z
        } else {
            parent.rot_z - state.rotation
        };
    }
    if flags & state_flags::SCALE_X != 0 {
        target.scale.x = state.scale_x * parent.scale.x;
    }
    if flags & state_flags::SCALE_Y != 0 {
        target.scale.y = state.scale_y * parent.scale.y;
    }
}

/// The channel loop that ends `applyFrame…`: channels shown by the frame enter multi-model
/// mode, the others leave it, and channels cleared with `clearTweenableChannelId:` are released.
fn update_channels(world: &mut World, animation: Entity) {
    for index in 0..CHANNEL_COUNT {
        let channel = world.obj::<BcAnimation>(animation).channels[index];
        let Some(tweenable) = channel.tweenable else {
            continue;
        };

        if channel.enable_multi_model_mode {
            if !channel.multi_model_mode {
                world.obj_mut::<BcAnimation>(animation).channels[index].multi_model_mode = true;
                display::enable_multi_model_mode(world, tweenable);
            }
        } else if channel.multi_model_mode {
            world.obj_mut::<BcAnimation>(animation).channels[index].multi_model_mode = false;
            display::disable_multi_model_mode(world, tweenable);
        }

        let release_next_frame = {
            let mut player = world.obj_mut::<BcAnimation>(animation);
            let entry = &mut player.channels[index];
            entry.enable_multi_model_mode = false;
            entry.release_next_frame
        };
        if release_next_frame {
            unbind_channel(world, animation, index);
        }
    }
}

/// Unbinds a channel's tweenable: leaves multi-model mode, drops the player's wake reference
/// (unless the player hibernates) and releases it.
fn unbind_channel(world: &mut World, animation: Entity, index: usize) {
    let channel = {
        let mut player = world.obj_mut::<BcAnimation>(animation);
        let entry = &mut player.channels[index];
        entry.release_next_frame = false;
        *entry
    };
    if channel.multi_model_mode {
        world.obj_mut::<BcAnimation>(animation).channels[index].multi_model_mode = false;
        if let Some(tweenable) = channel.tweenable {
            display::disable_multi_model_mode(world, tweenable);
        }
    }

    let (hibernating, tweenable) = {
        let player = world.obj::<BcAnimation>(animation);
        (player.hibernate, player.channels[index].tweenable)
    };
    if !hibernating && let Some(tweenable) = tweenable {
        display::hibernate(world, tweenable);
    }

    let mut player = world.obj_mut::<BcAnimation>(animation);
    let entry = &mut player.channels[index];
    entry.enable_multi_model_mode = false;
    entry.tweenable = None;
}

/// `-[BCBinaryAnimationDef applyMovement:animationListener:]` @ 0x1001a109c.
fn apply_movement(world: &mut World, def: AnimationDefId, raw_frame: i32, listener: Option<Entity>) {
    let timelines = world.resource::<Timelines>();
    let Some((move_x, move_y)) = movement_offset(timelines, def, raw_frame) else {
        return;
    };
    if let Some(listener) = listener {
        crate::logic::apply_move_pos(world, listener, move_x, move_y);
    }
}

/// The displacement at a raw frame number: whole cycles of the definition's move plus the
/// move of the last record at or before the position within the cycle; `None` when the
/// definition does not move.
fn movement_offset(timelines: &Timelines, def: AnimationDefId, raw_frame: i32) -> Option<(f32, f32)> {
    let animation = timelines.animation(def);
    if !animation.move_enabled {
        return None;
    }

    // 64-bit signed divide (ARM `sdiv` by zero gives 0), used as an int.
    let length = animation.length;
    let cycles = if length == 0 {
        0
    } else {
        let quotient = i64::from(raw_frame) / i64::from(length);
        quotient as i32
    };
    let cycles_float = cycles as f32;
    let mut move_x = cycles_float * animation.move_x;
    let mut move_y = cycles_float * animation.move_y;

    if (animation.frame_count as i32) >= 1 {
        let within_cycle = raw_frame.wrapping_sub(cycles.wrapping_mul(length));
        let mut chosen: Option<&TimelineFrame> = None;
        for offset in 0..animation.frame_count {
            let record_ref = FrameRef(animation.first_frame + offset);
            let record = timelines.frame(record_ref);
            if i64::from(within_cycle) < i64::from(record.index) {
                break;
            }
            chosen = Some(record);
        }
        if let Some(record) = chosen {
            move_x += record.move_x;
            move_y += record.move_y;
        }
    }
    Some((move_x, move_y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::model::{self, Model};
    use crate::view::{BcView, run_game_frame_listeners};

    fn test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Timelines>();
        world.init_resource::<Library>();
        world.init_resource::<BcView>();
        world
    }

    struct FrameSpec {
        index: i32,
        action: Option<&'static str>,
        move_x: f32,
        move_y: f32,
        states: Vec<TimelineState>,
    }

    fn frame_spec(index: i32) -> FrameSpec {
        FrameSpec {
            index,
            action: None,
            move_x: 0.0,
            move_y: 0.0,
            states: Vec::new(),
        }
    }

    struct AnimationSpec {
        name: &'static str,
        fps: i32,
        length: i32,
        looping: bool,
        move_enabled: bool,
        move_x: f32,
        move_y: f32,
        next: Option<&'static str>,
        frames: Vec<FrameSpec>,
    }

    /// One record per frame index `0..length`, like 2,343 of the shipped animations.
    fn regular_animation(name: &'static str, fps: i32, length: i32, looping: bool) -> AnimationSpec {
        let frames = (0..length).map(frame_spec).collect();
        AnimationSpec {
            name,
            fps,
            length,
            looping,
            move_enabled: false,
            move_x: 0.0,
            move_y: 0.0,
            next: None,
            frames,
        }
    }

    fn add_animation(world: &mut World, spec: AnimationSpec) -> AnimationDefId {
        let mut timelines = world.resource_mut::<Timelines>();
        let first_frame = timelines.frames.len() as u32;
        let frame_count = spec.frames.len() as u32;
        for frame in spec.frames {
            let first_state = timelines.state_lists.len() as u32;
            let state_count = frame.states.len() as u32;
            for state in frame.states {
                let state_index = timelines.states.len() as u32;
                timelines.states.push(state);
                timelines.state_lists.push(state_index);
            }
            let record = TimelineFrame {
                index: frame.index,
                sound: None,
                action: frame.action.map(str::to_owned),
                move_x: frame.move_x,
                move_y: frame.move_y,
                first_state,
                state_count,
            };
            timelines.frames.push(record);
        }

        let id = AnimationDefId(timelines.animations.len() as u32);
        let record = TimelineAnimation {
            name: spec.name.to_owned(),
            fps: spec.fps,
            length: spec.length,
            looping: spec.looping,
            move_enabled: spec.move_enabled,
            move_x: spec.move_x,
            move_y: spec.move_y,
            next_animation: spec.next.map(str::to_owned),
            first_frame,
            frame_count,
        };
        timelines.animations.push(record);
        world.resource_mut::<Library>().animation_defs.insert(spec.name.to_owned(), id);
        id
    }

    fn channel_state(channel: u8, flags: u32, texture: Option<TextureDefId>) -> TimelineState {
        TimelineState {
            name: "Channel".to_owned(),
            flags,
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
            vertices: [0.0; 12],
        }
    }

    fn player(world: &World, animation: Entity) -> &BcAnimation {
        world.obj::<BcAnimation>(animation)
    }

    fn is_listening(world: &World, animation: Entity) -> bool {
        let view = world.resource::<BcView>();
        view.game_frame_listeners.contains(&animation)
    }

    fn frame_ref(world: &World, def: AnimationDefId, offset: u32) -> FrameRef {
        let first_frame = world.resource::<Timelines>().animation(def).first_frame;
        FrameRef(first_frame + offset)
    }

    fn scan(world: &World, def: AnimationDefId, frame: i32, last_frame: i32) -> FrameScan {
        let timelines = world.resource::<Timelines>();
        scan_frames(timelines, def, frame, last_frame)
    }

    fn frame_refs(world: &World, def: AnimationDefId, offsets: &[u32]) -> Vec<FrameRef> {
        offsets.iter().map(|&offset| frame_ref(world, def, offset)).collect()
    }

    #[test]
    fn scan_fires_every_record_up_to_the_frame_once() {
        let mut world = test_world();
        let def = add_animation(&mut world, regular_animation("Walk", 24, 5, false));

        let from_start = scan(&world, def, 3, -1);
        assert_eq!(from_start.passed, frame_refs(&world, def, &[0, 1, 2]));
        assert_eq!(from_start.chosen, Some(frame_ref(&world, def, 3)));

        let skipping = scan(&world, def, 3, 0);
        assert_eq!(skipping.passed, frame_refs(&world, def, &[1, 2]));
        assert_eq!(skipping.chosen, Some(frame_ref(&world, def, 3)));

        let next = scan(&world, def, 4, 3);
        assert!(next.passed.is_empty());
        assert_eq!(next.chosen, Some(frame_ref(&world, def, 4)));
    }

    #[test]
    fn scan_after_a_loop_wrap_fires_only_the_chosen_record() {
        let mut world = test_world();
        let def = add_animation(&mut world, regular_animation("Loop", 24, 10, true));

        let wrapped = scan(&world, def, 1, 9);
        assert!(wrapped.passed.is_empty());
        assert_eq!(wrapped.chosen, Some(frame_ref(&world, def, 1)));
    }

    #[test]
    fn scan_with_a_duplicate_index_shows_the_later_record() {
        // PygmyWindBlown2Anim: indices 0, 1, 2, 2.
        let mut world = test_world();
        let mut spec = regular_animation("WindBlown", 12, 4, false);
        spec.frames = vec![frame_spec(0), frame_spec(1), frame_spec(2), frame_spec(2)];
        let def = add_animation(&mut world, spec);

        let stepped = scan(&world, def, 2, 1);
        assert!(stepped.passed.is_empty());
        assert_eq!(stepped.chosen, Some(frame_ref(&world, def, 3)));

        let skipped = scan(&world, def, 2, 0);
        assert_eq!(skipped.passed, frame_refs(&world, def, &[1, 2]));
        assert_eq!(skipped.chosen, Some(frame_ref(&world, def, 3)));
    }

    #[test]
    fn scan_with_irregular_indices_shows_the_last_record_at_or_before_the_frame() {
        // PygmyGainBalanceAnim: length 6, indices 0–4 (frame 5 shows record 4).
        let mut world = test_world();
        let mut spec = regular_animation("GainBalance", 12, 6, false);
        spec.frames = (0..5).map(frame_spec).collect();
        let def = add_animation(&mut world, spec);

        let last = scan(&world, def, 5, 4);
        assert!(last.passed.is_empty());
        assert_eq!(last.chosen, Some(frame_ref(&world, def, 4)));
    }

    #[test]
    fn scan_of_a_negative_frame_chooses_nothing() {
        let mut world = test_world();
        let def = add_animation(&mut world, regular_animation("Gauge", 12, 4, false));

        assert_eq!(scan(&world, def, -1, 0), FrameScan::default());
    }

    #[test]
    fn frame_numbers_follow_the_original_arithmetic() {
        assert_eq!(raw_frame_number(0.1, 24, 30, false, 0.0), 2);
        assert_eq!(raw_frame_number(5.0, 0, 30, false, 0.0), 0);
        assert_eq!(raw_frame_number(0.0, 24, 10, true, 0.25), 3);
        assert_eq!(raw_frame_number(0.0, 24, 10, true, 0.24), 2);
        assert_eq!(raw_frame_number(0.0, 24, 10, true, -0.3), -2);
        assert_eq!(raw_frame_number(0.0, 24, 10, true, f32::NAN), 0);
        assert_eq!(raw_frame_number(0.0, 24, 10, true, 1.0e10), i32::MAX);

        assert_eq!(displayed_frame_number(9, 4, true), 1);
        assert_eq!(displayed_frame_number(-3, 4, true), -3);
        assert_eq!(displayed_frame_number(-6, 4, true), -2);
        assert_eq!(displayed_frame_number(7, 0, true), 7);
        assert_eq!(displayed_frame_number(7, 5, false), 4);
        assert_eq!(displayed_frame_number(-2, 5, false), -2);
        assert_eq!(displayed_frame_number(0, 0, false), -1);
    }

    #[test]
    fn movement_adds_whole_cycles_and_the_record_within_the_cycle() {
        let mut world = test_world();
        let mut spec = regular_animation("Walk", 12, 4, true);
        spec.move_enabled = true;
        spec.move_x = 50.0;
        spec.move_y = -8.0;
        for (offset, frame) in spec.frames.iter_mut().enumerate() {
            let step = (offset + 1) as f32;
            frame.move_x = 12.5 * step;
            frame.move_y = -2.0 * step;
        }
        let def = add_animation(&mut world, spec);
        let still = add_animation(&mut world, regular_animation("Still", 12, 4, true));
        let timelines = world.resource::<Timelines>();

        assert_eq!(movement_offset(timelines, def, 3), Some((50.0, -8.0)));
        assert_eq!(movement_offset(timelines, def, 6), Some((87.5, -14.0)));
        assert_eq!(movement_offset(timelines, def, -1), Some((0.0, 0.0)));
        assert_eq!(movement_offset(timelines, def, -5), Some((-50.0, 8.0)));
        assert_eq!(movement_offset(timelines, still, 6), None);
    }

    #[test]
    fn channel_transform_is_relative_to_the_parent() {
        let parent = ParentTransform {
            pos: Vec4::new(100.0, 50.0, 0.0, 1.0),
            rot_z: 10.0,
            scale: Vec4::new(2.0, 3.0, 1.0, 1.0),
            origin_pos: Vec4::new(5.0, -5.0, 0.0, 1.0),
        };
        let all_flags = TRANSFORM_FLAGS;
        let state = ChannelState {
            channel: 1,
            flags: all_flags,
            x: 10.0,
            y: 20.0,
            rotation: 30.0,
            scale_x: 0.5,
            scale_y: 0.25,
            texture: None,
        };

        let mut target = DisplayObject::new(Some("Sword"));
        transform_tweenable(&mut target, &state, &parent);
        assert_eq!((target.pos.x, target.pos.y), (130.0, 95.0));
        assert_eq!(target.rot.z, 40.0);
        assert_eq!((target.scale.x, target.scale.y), (1.0, 0.75));

        let mirrored = ParentTransform {
            scale: Vec4::new(-2.0, 3.0, 1.0, 1.0),
            ..parent
        };
        transform_tweenable(&mut target, &state, &mirrored);
        assert_eq!(target.rot.z, 160.0);
        assert_eq!(target.scale.x, -1.0);

        let rotation_only = ChannelState {
            flags: state_flags::ROTATION,
            ..state
        };
        transform_tweenable(&mut target, &rotation_only, &mirrored);
        assert_eq!(target.rot.z, -20.0);

        let unordered = ParentTransform {
            scale: Vec4::new(f32::NAN, 3.0, 1.0, 1.0),
            ..parent
        };
        transform_tweenable(&mut target, &rotation_only, &unordered);
        assert_eq!(target.rot.z, -20.0);

        let untouched = ChannelState { flags: 0, ..state };
        let mut fresh = DisplayObject::new(Some("Shield"));
        transform_tweenable(&mut fresh, &untouched, &parent);
        assert_eq!(fresh.pos, DisplayObject::new(None).pos);
    }

    #[test]
    fn plays_frames_until_complete() {
        let mut world = test_world();
        add_animation(&mut world, regular_animation("Throw", 10, 3, false));
        let animation = spawn(&mut world);

        play_animation_id(&mut world, animation, Some("Throw"));
        assert!(player(&world, animation).is_running);
        assert!(is_listening(&world, animation));
        assert_eq!(frame_index(&world, animation), -1);

        run_game_frame_listeners(&mut world, 0.05);
        assert_eq!(frame_index(&world, animation), 0);
        run_game_frame_listeners(&mut world, 0.1);
        assert_eq!(frame_index(&world, animation), 1);
        assert!(!is_animation_complete(&world, animation));

        run_game_frame_listeners(&mut world, 0.1);
        assert_eq!(frame_index(&world, animation), 2);
        assert!(is_animation_complete(&world, animation));
        assert!(!player(&world, animation).is_running);
        assert!(!is_listening(&world, animation));
    }

    #[test]
    fn a_long_tick_completes_on_the_clamped_last_frame() {
        let mut world = test_world();
        add_animation(&mut world, regular_animation("Splash", 30, 3, false));
        let animation = spawn(&mut world);

        play_animation_id(&mut world, animation, Some("Splash"));
        run_game_frame_listeners(&mut world, 1.0);
        assert_eq!(frame_index(&world, animation), 2);
        assert!(is_animation_complete(&world, animation));
    }

    #[test]
    fn next_animation_starts_in_the_same_tick() {
        let mut world = test_world();
        let mut turn = regular_animation("GhostFlyTurnAnim", 10, 2, false);
        turn.next = Some("GhostFlyAnim");
        add_animation(&mut world, turn);
        let fly = add_animation(&mut world, regular_animation("GhostFlyAnim", 10, 4, true));
        let animation = spawn(&mut world);

        play_animation_id(&mut world, animation, Some("GhostFlyTurnAnim"));
        run_game_frame_listeners(&mut world, 0.2);
        let state = player(&world, animation);
        assert_eq!(state.def, Some(fly));
        assert_eq!(state.frame_index, -1);
        assert_eq!(state.anim_time, 0.0);
        assert!(!state.is_complete);
        assert!(state.is_running);
    }

    #[test]
    fn a_self_chaining_animation_is_never_observed_complete() {
        let mut world = test_world();
        let mut trapped = regular_animation("GraveyardTreePygmyTrappedAnim", 10, 2, false);
        trapped.next = Some("GraveyardTreePygmyTrappedAnim");
        let def = add_animation(&mut world, trapped);
        let animation = spawn(&mut world);

        play_animation_id(&mut world, animation, Some("GraveyardTreePygmyTrappedAnim"));
        for _ in 0..5 {
            run_game_frame_listeners(&mut world, 0.15);
            assert!(!is_animation_complete(&world, animation));
            assert_eq!(player(&world, animation).def, Some(def));
        }
    }

    #[test]
    fn chaining_moves_the_player_to_the_end_of_the_listeners() {
        let mut world = test_world();
        let mut turn = regular_animation("Turn", 10, 2, false);
        turn.next = Some("Fly");
        add_animation(&mut world, turn);
        add_animation(&mut world, regular_animation("Fly", 10, 4, true));
        let first = spawn(&mut world);
        let second = spawn(&mut world);

        play_animation_id(&mut world, first, Some("Turn"));
        play_animation_id(&mut world, second, Some("Fly"));
        run_game_frame_listeners(&mut world, 0.2);
        let listeners = world.resource::<BcView>().game_frame_listeners.clone();
        assert_eq!(listeners, vec![second, first]);
    }

    #[test]
    fn unknown_and_dangling_animations_complete_at_once() {
        let mut world = test_world();
        let mut cage = regular_animation("PygmyGoGoCageDropAnim", 10, 2, false);
        cage.next = Some("PygmyDanceAnim");
        add_animation(&mut world, cage);
        let animation = spawn(&mut world);

        play_animation_id(&mut world, animation, Some("Missing"));
        assert!(is_animation_complete(&world, animation));
        assert!(!player(&world, animation).is_running);
        assert_eq!(player(&world, animation).def, None);

        play_animation_id(&mut world, animation, None);
        assert!(is_animation_complete(&world, animation));

        play_animation_id(&mut world, animation, Some("PygmyGoGoCageDropAnim"));
        run_game_frame_listeners(&mut world, 0.2);
        let state = player(&world, animation);
        assert_eq!(state.def, None);
        assert_eq!(state.frame_index, -1);
        assert!(state.is_complete);
        assert!(!state.is_running);
    }

    #[test]
    fn frame_percent_scrubs_without_completing() {
        let mut world = test_world();
        add_animation(&mut world, regular_animation("HealthBar", 24, 11, false));
        let animation = spawn(&mut world);
        play_animation_id(&mut world, animation, Some("HealthBar"));
        stop(&mut world, animation);

        set_frame_percent(&mut world, animation, 1.0);
        assert!(player(&world, animation).is_running);
        run_game_frame_listeners(&mut world, 0.5);
        assert_eq!(frame_index(&world, animation), 10);
        assert!(!is_animation_complete(&world, animation));
        assert!(player(&world, animation).is_running);

        set_frame_percent(&mut world, animation, 0.5);
        run_game_frame_listeners(&mut world, 0.5);
        assert_eq!(frame_index(&world, animation), 6);

        play_animation_id(&mut world, animation, Some("HealthBar"));
        assert!(!player(&world, animation).use_frame_percent);
    }

    #[test]
    fn replay_current_frame_reapplies_the_last_frame() {
        let mut world = test_world();
        add_animation(&mut world, regular_animation("Throw", 10, 3, false));
        let animation = spawn(&mut world);
        play_animation_id(&mut world, animation, Some("Throw"));
        run_game_frame_listeners(&mut world, 0.5);
        assert!(is_animation_complete(&world, animation));

        replay_current_frame(&mut world, animation);
        assert_eq!(frame_index(&world, animation), 1);
        assert!(!is_animation_complete(&world, animation));
        assert!(player(&world, animation).is_running);

        run_game_frame_listeners(&mut world, 0.1);
        assert_eq!(frame_index(&world, animation), 2);
        assert!(is_animation_complete(&world, animation));

        replay_current_frame(&mut world, animation);
        replay_current_frame(&mut world, animation);
        assert_eq!(frame_index(&world, animation), 1);
    }

    #[test]
    fn hibernation_defers_play_and_stop() {
        let mut world = test_world();
        add_animation(&mut world, regular_animation("Idle", 10, 4, true));
        let animation = spawn(&mut world);
        play_animation_id(&mut world, animation, Some("Idle"));

        hibernate(&mut world, animation);
        assert!(is_hibernating(&world, animation));
        assert!(!player(&world, animation).is_running);
        assert!(player(&world, animation).hibernate_game_frame_running);
        assert!(!is_listening(&world, animation));

        stop(&mut world, animation);
        assert!(!player(&world, animation).hibernate_game_frame_running);
        play(&mut world, animation);
        assert!(!player(&world, animation).is_running);

        unhibernate(&mut world, animation);
        assert!(!is_hibernating(&world, animation));
        assert!(player(&world, animation).is_running);
        assert!(is_listening(&world, animation));

        hibernate(&mut world, animation);
        stop(&mut world, animation);
        unhibernate(&mut world, animation);
        assert!(!player(&world, animation).is_running);
    }

    #[test]
    fn channels_follow_the_frames_that_show_them() {
        let mut world = test_world();
        let sword_texture = TextureDefId(7);
        let mut spec = regular_animation("PygmyHoldSword", 10, 2, true);
        let sword_state = channel_state(7, state_flags::TEXTURE, Some(sword_texture));
        spec.frames[0].states.push(sword_state);
        add_animation(&mut world, spec);
        let animation = spawn(&mut world);
        let sword = model::spawn(&mut world, Some("Sword"), None);

        set_tweenable(&mut world, animation, Some(sword), 7);
        assert!(!display::object(&world, sword).hibernate);
        assert_eq!(display::object(&world, sword).hibernate_ref_count, 1);

        play_animation_id(&mut world, animation, Some("PygmyHoldSword"));
        run_game_frame_listeners(&mut world, 0.05);
        assert!(display::object(&world, sword).multi_model_mode);
        assert_eq!(world.obj::<Model>(sword).texture_def, Some(sword_texture));
        assert!(player(&world, animation).channels[7].multi_model_mode);
        assert!(!player(&world, animation).channels[7].enable_multi_model_mode);

        run_game_frame_listeners(&mut world, 0.1);
        assert!(!display::object(&world, sword).multi_model_mode);
        assert!(!player(&world, animation).channels[7].multi_model_mode);

        clear_tweenable_channel_id(&mut world, animation, 7);
        assert_eq!(player(&world, animation).channels[7].tweenable, Some(sword));
        run_game_frame_listeners(&mut world, 0.1);
        let channel = player(&world, animation).channels[7];
        assert_eq!(channel.tweenable, None);
        assert!(!channel.multi_model_mode);
        assert!(!channel.release_next_frame);
        assert!(!display::object(&world, sword).multi_model_mode);
        assert!(display::object(&world, sword).hibernate);
        assert_eq!(display::object(&world, sword).hibernate_ref_count, 0);
    }

    #[test]
    fn a_frame_without_a_record_takes_every_channel_out_of_multi_model_mode() {
        let mut world = test_world();
        let mut spec = regular_animation("Lever", 10, 4, false);
        let state = channel_state(2, state_flags::TEXTURE, None);
        spec.frames[0].states.push(state);
        add_animation(&mut world, spec);
        let animation = spawn(&mut world);
        let handle = model::spawn(&mut world, Some("Handle"), None);
        set_tweenable(&mut world, animation, Some(handle), 2);

        play_animation_id(&mut world, animation, Some("Lever"));
        set_frame_percent(&mut world, animation, 0.0);
        run_game_frame_listeners(&mut world, 0.1);
        assert_eq!(frame_index(&world, animation), 0);
        assert!(display::object(&world, handle).multi_model_mode);

        // trunc(4 · -1 + 0.5) = -3: no record has an index at or below it.
        set_frame_percent(&mut world, animation, -1.0);
        run_game_frame_listeners(&mut world, 0.1);
        assert_eq!(frame_index(&world, animation), -3);
        assert!(!display::object(&world, handle).multi_model_mode);
    }

    #[test]
    fn clearing_a_channel_now_unbinds_it_and_replays_a_finished_animation() {
        let mut world = test_world();
        let mut spec = regular_animation("ChainSpear", 10, 2, false);
        let state = channel_state(3, state_flags::TEXTURE, None);
        spec.frames[1].states.push(state);
        add_animation(&mut world, spec);
        let animation = spawn(&mut world);
        let pygmy = model::spawn(&mut world, Some("Pygmy"), None);
        set_tweenable(&mut world, animation, Some(pygmy), 3);

        play_animation_id(&mut world, animation, Some("ChainSpear"));
        run_game_frame_listeners(&mut world, 0.5);
        assert!(is_animation_complete(&world, animation));
        assert!(display::object(&world, pygmy).multi_model_mode);

        clear_tweenable_channel_id_now(&mut world, animation, 3);
        let channel = player(&world, animation).channels[3];
        assert_eq!(channel.tweenable, None);
        assert!(!channel.multi_model_mode);
        assert!(!display::object(&world, pygmy).multi_model_mode);
        assert!(display::object(&world, pygmy).hibernate);
        assert_eq!(frame_index(&world, animation), 0);
        assert!(player(&world, animation).is_running);
    }

    #[test]
    fn replacing_a_tweenable_keeps_the_old_wake_reference() {
        let mut world = test_world();
        let mut spec = regular_animation("Shield", 10, 2, true);
        let state = channel_state(8, state_flags::TEXTURE, None);
        spec.frames[0].states.push(state);
        add_animation(&mut world, spec);
        let animation = spawn(&mut world);
        let old = model::spawn(&mut world, Some("OldShield"), None);
        let new = model::spawn(&mut world, Some("NewShield"), None);

        set_tweenable(&mut world, animation, Some(old), 8);
        play_animation_id(&mut world, animation, Some("Shield"));
        run_game_frame_listeners(&mut world, 0.05);
        assert!(display::object(&world, old).multi_model_mode);

        set_tweenable(&mut world, animation, Some(new), 8);
        assert!(!display::object(&world, old).multi_model_mode);
        assert_eq!(display::object(&world, old).hibernate_ref_count, 1);
        assert_eq!(display::object(&world, new).hibernate_ref_count, 1);
        assert_eq!(player(&world, animation).channels[8].tweenable, Some(new));
    }

    #[test]
    fn hibernation_forwards_to_channel_tweenables() {
        let mut world = test_world();
        add_animation(&mut world, regular_animation("Idle", 10, 4, true));
        let animation = spawn(&mut world);
        let sword = model::spawn(&mut world, Some("Sword"), None);
        set_tweenable(&mut world, animation, Some(sword), 1);

        hibernate(&mut world, animation);
        assert!(display::object(&world, sword).hibernate);
        assert_eq!(display::object(&world, sword).hibernate_ref_count, 0);

        unhibernate(&mut world, animation);
        assert!(!display::object(&world, sword).hibernate);
        assert_eq!(display::object(&world, sword).hibernate_ref_count, 1);

        hibernate(&mut world, animation);
        let spear = model::spawn(&mut world, Some("Spear"), None);
        set_tweenable(&mut world, animation, Some(spear), 2);
        assert_eq!(display::object(&world, spear).hibernate_ref_count, 0);
    }

    #[test]
    fn dealloc_unbinds_channels_and_stops() {
        let mut world = test_world();
        let mut spec = regular_animation("Hold", 10, 2, true);
        let state = channel_state(1, state_flags::TEXTURE, None);
        spec.frames[0].states.push(state);
        add_animation(&mut world, spec);
        let animation = spawn(&mut world);
        let pygmy = model::spawn(&mut world, Some("Pygmy"), None);
        set_tweenable(&mut world, animation, Some(pygmy), 1);
        play_animation_id(&mut world, animation, Some("Hold"));
        run_game_frame_listeners(&mut world, 0.05);
        assert!(display::object(&world, pygmy).multi_model_mode);

        dealloc(&mut world, animation);
        assert!(world.get::<BcAnimation>(animation).is_none());
        assert!(!is_listening(&world, animation));
        assert!(!display::object(&world, pygmy).multi_model_mode);
        assert!(display::object(&world, pygmy).hibernate);
    }

    #[derive(Resource)]
    struct RestartRequest {
        animation: Entity,
        id: &'static str,
    }

    /// A model whose `setTextureDef:` restarts a player, like a frame action that calls
    /// `playAnimationId:` on its own animation.
    fn restart_on_set_texture(world: &mut World, entity: Entity, texture: Option<TextureDefId>) {
        world.obj_mut::<Model>(entity).texture_def = texture;
        let Some(request) = world.remove_resource::<RestartRequest>() else {
            return;
        };
        play_animation_id(world, request.animation, Some(request.id));
    }

    const RESTARTING_MODEL: Class = Class {
        name: "RestartingModel",
        set_texture_def: restart_on_set_texture,
        ..model::BC_MODEL
    };

    fn reentrant_world(next_of_restarted: Option<&'static str>) -> (World, Entity) {
        let mut world = test_world();
        let mut first = regular_animation("First", 10, 3, false);
        let restart_state = channel_state(1, state_flags::TEXTURE, Some(TextureDefId(3)));
        first.frames[2].states.push(restart_state);
        add_animation(&mut world, first);
        let mut restarted = regular_animation("Restarted", 10, 5, false);
        restarted.next = next_of_restarted;
        add_animation(&mut world, restarted);
        add_animation(&mut world, regular_animation("Next", 10, 4, true));

        let animation = spawn(&mut world);
        let trigger = display::spawn(&mut world, &RESTARTING_MODEL, Some("Trigger"));
        world.entity_mut(trigger).insert(Model::default());
        set_tweenable(&mut world, animation, Some(trigger), 1);
        play_animation_id(&mut world, animation, Some("First"));
        world.insert_resource(RestartRequest {
            animation,
            id: "Restarted",
        });
        (world, animation)
    }

    #[test]
    fn a_restart_during_the_last_frame_completes_the_new_animation() {
        let (mut world, animation) = reentrant_world(None);
        let restarted = world.resource::<Library>().animation_def("Restarted");

        run_game_frame_listeners(&mut world, 0.25);
        let state = player(&world, animation);
        assert_eq!(state.def, restarted);
        assert_eq!(state.frame_index, 2);
        assert_eq!(state.anim_time, 0.0);
        assert!(state.is_complete);
        assert!(!state.is_running);
    }

    #[test]
    fn a_restart_during_the_last_frame_plays_the_new_next_animation() {
        let (mut world, animation) = reentrant_world(Some("Next"));
        let next = world.resource::<Library>().animation_def("Next");

        run_game_frame_listeners(&mut world, 0.25);
        let state = player(&world, animation);
        assert_eq!(state.def, next);
        assert_eq!(state.frame_index, -1);
        assert!(!state.is_complete);
        assert!(state.is_running);
    }

    #[derive(Resource, Default)]
    struct ListenerLog {
        actions: Vec<String>,
        moves: Vec<(f32, f32)>,
    }

    fn record_action(world: &mut World, _this: Entity, selector: &str) -> bool {
        if !selector.starts_with("anim") {
            return false;
        }
        let mut log = world.resource_mut::<ListenerLog>();
        log.actions.push(selector.to_owned());
        true
    }

    fn record_move(world: &mut World, _this: Entity, move_x: f32, move_y: f32) {
        let mut log = world.resource_mut::<ListenerLog>();
        log.moves.push((move_x, move_y));
    }

    const RECORDING_LISTENER: Class = Class {
        name: "RecordingListener",
        perform: record_action,
        apply_move_pos: record_move,
        ..Class::NS_OBJECT
    };

    #[test]
    fn frame_actions_go_to_the_listener_only() {
        let mut world = test_world();
        world.init_resource::<ListenerLog>();
        let mut spec = regular_animation("Throw", 10, 3, false);
        spec.frames[1].action = Some("animThrowSpear");
        spec.frames[2].action = Some("unknownAction");
        let def = add_animation(&mut world, spec);
        let listener = world.spawn(Isa(&RECORDING_LISTENER)).id();
        let throw_frame = frame_ref(&world, def, 1);
        let unknown_frame = frame_ref(&world, def, 2);

        apply_sound_and_action(&mut world, throw_frame, None);
        assert!(world.resource::<ListenerLog>().actions.is_empty());

        apply_sound_and_action(&mut world, throw_frame, Some(listener));
        apply_sound_and_action(&mut world, unknown_frame, Some(listener));
        assert_eq!(world.resource::<ListenerLog>().actions, vec!["animThrowSpear".to_owned()]);
    }

    #[test]
    fn movement_uses_the_raw_frame_number() {
        let mut world = test_world();
        world.init_resource::<ListenerLog>();
        let mut spec = regular_animation("DodoWalk", 10, 4, true);
        spec.move_enabled = true;
        spec.move_x = 40.0;
        spec.frames.clear();
        add_animation(&mut world, spec);
        let animation = spawn(&mut world);
        let listener = world.spawn(Isa(&RECORDING_LISTENER)).id();

        play_animation_id(&mut world, animation, Some("DodoWalk"));
        set_animation_listener(&mut world, animation, Some(listener));
        run_game_frame_listeners(&mut world, 1.25);
        assert_eq!(frame_index(&world, animation), 0);
        assert_eq!(world.resource::<ListenerLog>().moves, vec![(120.0, 0.0)]);
    }
}
