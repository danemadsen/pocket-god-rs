//! `BCLibrary`'s sequence-item handlers: the fallback for items the delegate does not
//! implement (`docs/re/sequences.md` §5 "BCLibrary", `docs/re/rendering.md` §11).
//!
//! The timers and tweens keep their state in the [`Library`] singleton (`mStallTime`,
//! `mDeltaValue`, `mValue`, `mStallFrames`, `mDeltaPos`), shared by every sequence that falls
//! back to these handlers (`docs/re/sequences.md` §3.8): two such sequences timing in the same
//! frames overwrite each other's state, exactly as in the original.
//!
//! Display objects are addressed as `[group getDisplayObject:id]` (a direct child). The alpha
//! items (`hideObjects`, `showObjects`, `setAlpha`, `tweenAlpha`, `sinAlpha`, `blink`) read a
//! `displayGroupId` of `""` as the active display group; `setPos`, `tweenPos` and
//! `setTextureDef` use the active display group when the parameter is absent; `setBehavior`
//! always looks the group up. Messages to a missing object do nothing (nil messaging).

use bevy_ecs::prelude::*;
use bevy_math::Vec4;

use crate::display::{self, group};
use crate::library::{self, Library};
use crate::math::trunc_i32;
use crate::random::Random;
use crate::sequence::{self, ItemResult, SequenceCall, SequenceDefRef, ns_float_value};
use crate::sound::{self, Sounds};
use crate::texture::{AtlasId, TextureDefId};
use crate::view::{self, BcView};
use crate::{animation, logic, texture};

/// `stall:::` draws random durations in milliseconds (`docs/re/sequences.md` §3.13).
const STALL_TIME_SCALE: f32 = 1000.0;

/// Dispatches to the `BCLibrary` handler for `call.method()`; `None` if it has none.
pub fn sequence_item(world: &mut World, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "trace" => seq_trace(world, call),
        "pushSequence" => seq_push_sequence(world, call),
        "runSequence" => seq_run_sequence(world, call),
        "chooseSequence" => seq_choose_sequence(world, call),
        "choosePushSequence" => seq_choose_push_sequence(world, call),
        "loadTextureAtlas" => seq_load_texture_atlas(world, call),
        "drawDisplayGroup" => seq_draw_display_group(world, call),
        "drawWideScreenOverlayDisplayGroup" => seq_draw_wide_screen_overlay_display_group(world, call),
        "drawTallScreenOverlayDisplayGroup" => seq_draw_tall_screen_overlay_display_group(world, call),
        "addDisplayGroup" => seq_add_display_group(world, call),
        "removeDisplayGroup" => seq_remove_display_group(world, call),
        "hideObjects" => seq_hide_objects(world, call),
        "showObjects" => seq_show_objects(world, call),
        "setSoundAlias" => seq_set_sound_alias(world, call),
        "playSound" => seq_play_sound(world, call),
        "stopSound" => seq_stop_sound(world, call),
        "stopAllSounds" => seq_stop_all_sounds(world, call),
        "mute" => seq_mute(world, call),
        "unmute" => seq_unmute(world, call),
        "stall" => seq_stall(world, call),
        "stallFrames" => seq_stall_frames(world, call),
        "setAlpha" => seq_set_alpha(world, call),
        "tweenAlpha" => seq_tween_alpha(world, call),
        "sinAlpha" => seq_sin_alpha(world, call),
        "setPos" => seq_set_pos(world, call),
        "tweenPos" => seq_tween_pos(world, call),
        "blink" => seq_blink(world, call),
        "setAnimation" => seq_set_animation(world, call),
        "setTextureDef" => seq_set_texture_def(world, call),
        "setBehavior" => seq_set_behavior(world, call),
        _ => return None,
    };
    Some(result)
}

/// `-[BCLibrary trace:::]` @ 0x100024148: logs `message` under the `Default` tag.
pub fn seq_trace(_world: &mut World, call: &mut SequenceCall) -> ItemResult {
    // `BCLog` formats the parameter with `@"%@"` (nil prints "(null)") and prints
    // "<tag>: <text>"; the log item it also records is only read by a debugger helper.
    let message = call.param("message").unwrap_or("(null)");
    println!("Default: {message}");
    ItemResult::Done
}

/// `-[BCLibrary pushSequence:::]` @ 0x100024198.
pub fn seq_push_sequence(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let def = sequence_def_by_id(world, call.param("id"));
    sequence::push_sequence_def(world, call.sequence, def);
    ItemResult::Done
}

/// `-[BCLibrary runSequence:::]` @ 0x100024204.
pub fn seq_run_sequence(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let def = sequence_def_by_id(world, call.param("id"));
    sequence::set_sequence_def(world, call.sequence, def);
    ItemResult::Done
}

/// `-[BCLibrary chooseSequence:::]` @ 0x100024270.
pub fn seq_choose_sequence(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let def = choose_sequence_def(world, call);
    sequence::set_sequence_def(world, call.sequence, def);
    ItemResult::Done
}

/// `-[BCLibrary choosePushSequence:::]` @ 0x100024344.
pub fn seq_choose_push_sequence(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let def = choose_sequence_def(world, call);
    sequence::push_sequence_def(world, call.sequence, def);
    ItemResult::Done
}

/// `-[BCLibrary loadTextureAtlas:::]` @ 0x100024418: now, or next frame when `delay` is present.
pub fn seq_load_texture_atlas(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    // `ext` is passed along in the original but the loader always opens `<file>.raw`.
    let file = call.param("file");
    let atlas = atlas_by_id(world, call.param("id"));
    let delay = call.has("delay");

    let (Some(atlas), Some(file)) = (atlas, file) else {
        // The original dereferences the NULL slot (at once or next frame), or loads whichever
        // `.raw` file the bundle finds first for a nil name.
        report_unreachable(call, "unknown atlas slot or no file name");
        return ItemResult::Done;
    };

    if delay {
        view::activate_texture_atlas_next_frame(world, atlas, file);
    } else {
        texture::load_atlas(world, atlas, file);
    }
    ItemResult::Done
}

/// `-[BCLibrary drawDisplayGroup:::]` @ 0x1000244c8.
pub fn seq_draw_display_group(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = display_group_by_id(world, call.param("id"));
    view::set_active_display_group(world, group);
    ItemResult::Done
}

/// `-[BCLibrary drawWideScreenOverlayDisplayGroup:::]` @ 0x10002452c.
pub fn seq_draw_wide_screen_overlay_display_group(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = display_group_by_id(world, call.param("id"));
    view::set_wide_screen_overlay_display_group(world, group);
    ItemResult::Done
}

/// `-[BCLibrary drawTallScreenOverlayDisplayGroup:::]` @ 0x100024590.
pub fn seq_draw_tall_screen_overlay_display_group(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = display_group_by_id(world, call.param("id"));
    view::set_tall_screen_overlay_display_group(world, group);
    ItemResult::Done
}

/// `-[BCLibrary addDisplayGroup:::]` @ 0x1000245f4: into the parent, marker or active group.
pub fn seq_add_display_group(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let child = display_group_by_id(world, call.param("id"));
    let target = match (call.param("parentId"), call.param("markerId")) {
        (Some(parent_id), _) => library::display_group(world, parent_id),
        (None, Some(marker_id)) => library::display_marker(world, marker_id),
        (None, None) => active_display_group(world),
    };

    // A nil child is still sent: a marker then empties its slot.
    if let Some(target) = target {
        group::add_display_object(world, target, child);
    }
    ItemResult::Done
}

/// `-[BCLibrary removeDisplayGroup:::]` @ 0x1000246bc.
pub fn seq_remove_display_group(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = display_group_by_id(world, call.param("id"));
    if let Some(group) = group {
        display::remove_from_display_group(world, group);
    }
    ItemResult::Done
}

/// `-[BCLibrary hideObjects:::]` @ 0x1000246f8.
pub fn seq_hide_objects(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    set_objects_hidden(world, call, true);
    ItemResult::Done
}

/// `-[BCLibrary showObjects:::]` @ 0x1000248bc.
pub fn seq_show_objects(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    set_objects_hidden(world, call, false);
    ItemResult::Done
}

/// `-[BCLibrary setSoundAlias:::]` @ 0x100024a80: unknown sounds are ignored.
pub fn seq_set_sound_alias(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let alias = call.param("aliasId");
    let sound_id = call.param("soundId");

    let mut library = world.resource_mut::<Library>();
    let sound = match sound_id {
        Some(sound_id) => library.sound(sound_id),
        None => None,
    };
    match alias {
        Some(alias) => library.set_sound_alias(alias, sound),
        None if sound.is_some() => {
            // `setObject:forKey:nil` throws in the original.
            report_unreachable(call, "no alias name");
        }
        None => {}
    }
    ItemResult::Done
}

/// `-[BCLibrary playSound:::]` @ 0x100024ad4.
pub fn seq_play_sound(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    // `getSoundById:nil` is nil, so an absent id plays nothing.
    if let Some(sound_id) = call.param("soundId") {
        sound::play_sound(world, sound_id);
    }
    ItemResult::Done
}

/// `-[BCLibrary stopSound:::]` @ 0x100024b10.
pub fn seq_stop_sound(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    if let Some(sound_id) = call.param("soundId") {
        sound::stop_sound(world, sound_id);
    }
    ItemResult::Done
}

/// `-[BCLibrary stopAllSounds:::]` @ 0x100024c7c.
pub fn seq_stop_all_sounds(world: &mut World, _call: &mut SequenceCall) -> ItemResult {
    sound::stop_all_sounds(world);
    ItemResult::Done
}

/// `-[BCLibrary mute:::]` @ 0x100024ca0: `[BCSound mute:YES]` sets the class-wide mute flag.
pub fn seq_mute(world: &mut World, _call: &mut SequenceCall) -> ItemResult {
    world.resource_mut::<Sounds>().muted = true;
    ItemResult::Done
}

/// `-[BCLibrary unmute:::]` @ 0x100024cd0: `[BCSound mute:NO]`.
pub fn seq_unmute(world: &mut World, _call: &mut SequenceCall) -> ItemResult {
    world.resource_mut::<Sounds>().muted = false;
    ItemResult::Done
}

/// `-[BCLibrary stall:::]` @ 0x100024d00: library-wide timer (`docs/re/sequences.md` §3.13).
pub fn seq_stall(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let stall_time = if call.has("minTime") {
            let min_time = call.float("minTime");
            let max_time = call.float("maxTime");
            random_stall_time(world, min_time, max_time)
        } else {
            call.float("time")
        };
        world.resource_mut::<Library>().stall_time = stall_time;
    }

    // The check precedes the subtraction, so the item completes one frame after the time ran
    // out; a NaN time never completes.
    let mut library = world.resource_mut::<Library>();
    if library.stall_time <= 0.0 {
        return ItemResult::Done;
    }
    library.stall_time -= call.delta_time;
    ItemResult::Pending
}

/// `-[BCLibrary stallFrames:::]` @ 0x100024e10: completes on the `frames`-th call.
pub fn seq_stall_frames(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let mut library = world.resource_mut::<Library>();
    if call.first_time {
        library.stall_frames = call.int("frames");
    }

    let remaining = library.stall_frames;
    library.stall_frames = remaining.wrapping_sub(1);
    if remaining < 2 { ItemResult::Done } else { ItemResult::Pending }
}

/// `-[BCLibrary setAlpha:::]` @ 0x100024e74.
pub fn seq_set_alpha(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = group_or_active_if_empty(world, call.param("displayGroupId"));
    let object = display_object_in(world, group, call.param("id"));
    let alpha = call.float("alpha");
    set_object_alpha(world, object, alpha);
    ItemResult::Done
}

/// `-[BCLibrary tweenAlpha:::]` @ 0x100024f68: linear alpha tween on the library-wide timer.
pub fn seq_tween_alpha(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = group_or_active_if_empty(world, call.param("displayGroupId"));
    let object = display_object_in(world, group, call.param("id"));
    let current_alpha = object_alpha(world, object);
    let target = call.float("target");

    let (alpha, result) = {
        let mut library = world.resource_mut::<Library>();
        if call.first_time {
            let time = call.float("time");
            library.stall_time = time;
            library.delta_value = (target - current_alpha) / time;
        }

        // A NaN time keeps tweening: the original selects with `hi`, true when unordered.
        let remaining = library.stall_time;
        if remaining <= 0.0 {
            (target, ItemResult::Done)
        } else {
            let distance = remaining * library.delta_value;
            (target - distance, ItemResult::Pending)
        }
    };

    // The first call shows the start value; the last one lands exactly on the target.
    set_object_alpha(world, object, alpha);
    world.resource_mut::<Library>().stall_time -= call.delta_time;
    result
}

/// `-[BCLibrary sinAlpha:::]` @ 0x100025118: alpha oscillation that never completes.
pub fn seq_sin_alpha(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = group_or_active_if_empty(world, call.param("displayGroupId"));
    let object = display_object_in(world, group, call.param("id"));
    let speed = call.float("speed");
    let min = call.float("min");
    let max = call.float("max");
    let half_range = (max - min) * 0.5;
    let middle = min + half_range;

    if call.first_time {
        let current_alpha = object_alpha(world, object);
        let ratio = (current_alpha - middle) / half_range;
        world.resource_mut::<Library>().value = ratio.asin();
    }

    // Original bug: the phase is never advanced, so the alpha stays near its start value.
    // `time` is never read.
    let phase = world.resource::<Library>().value;
    let angle = speed * call.delta_time + phase;
    let wave = angle.sin();
    let alpha = middle + half_range * wave;
    set_object_alpha(world, object, alpha);
    ItemResult::Pending
}

/// `-[BCLibrary setPos:::]` @ 0x1000252cc: sets each present coordinate.
pub fn seq_set_pos(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = group_or_active_if_absent(world, call.param("displayGroupId"));
    let object = display_object_in(world, group, call.param("id"));
    let has_x = call.has("xPos");
    let has_y = call.has("yPos");
    let has_z = call.has("zPos");

    let Some(object) = object else {
        if has_x || has_y || has_z {
            // `[nil pos]` is a NULL pointer the original writes through.
            report_unreachable(call, "no such display object");
        }
        return ItemResult::Done;
    };

    let mut display_object = display::object_mut(world, object);
    if has_x {
        display_object.pos.x = call.float("xPos");
    }
    if has_y {
        display_object.pos.y = call.float("yPos");
    }
    if has_z {
        display_object.pos.z = call.float("zPos");
    }
    ItemResult::Done
}

/// `-[BCLibrary tweenPos:::]` @ 0x1000253d0: position tween with the original's dt bug.
pub fn seq_tween_pos(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = group_or_active_if_absent(world, call.param("displayGroupId"));
    let object = display_object_in(world, group, call.param("id"));
    let Some(object) = object else {
        // `[nil pos]` is a NULL pointer the original reads through.
        report_unreachable(call, "no such display object");
        return ItemResult::Done;
    };

    let pos = display::object(world, object).pos;
    let target_x = float_or(call, "targetXPos", pos.x);
    let target_y = float_or(call, "targetYPos", pos.y);
    let target_z = float_or(call, "targetZPos", pos.z);
    let target = Vec4::new(target_x, target_y, target_z, pos.w);

    if call.first_time {
        let mut library = world.resource_mut::<Library>();
        library.stall_time = call.float("time");

        // `(target - pos) / time` is computed as a multiplication by the reciprocal.
        let inverse_time = 1.0 / library.stall_time;
        let velocity_x = inverse_time * (target.x - pos.x);
        let velocity_y = inverse_time * (target.y - pos.y);
        let velocity_z = inverse_time * (target.z - pos.z);
        library.delta_pos = [velocity_x, velocity_y, velocity_z, target.w];
    }

    // Original bug: each frame steps back from the target by one frame of velocity (`dt`, not
    // the remaining time), so the object jumps next to the target on the first frame.
    let dt = call.delta_time;
    let (velocity, remaining) = {
        let library = world.resource::<Library>();
        (library.delta_pos, library.stall_time)
    };
    let stepped_x = target.x - velocity[0] * dt;
    let stepped_y = target.y - velocity[1] * dt;
    let stepped_z = target.z - velocity[2] * dt;
    let stepped = Vec4::new(stepped_x, stepped_y, stepped_z, target.w);

    let (new_pos, result) = if remaining <= 0.0 {
        (target, ItemResult::Done)
    } else {
        (stepped, ItemResult::Pending)
    };
    display::object_mut(world, object).pos = new_pos;
    world.resource_mut::<Library>().stall_time = remaining - dt;
    result
}

/// `-[BCLibrary blink:::]` @ 0x10002562c: toggles alpha 0 / 1 every `time` seconds, forever.
pub fn seq_blink(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = group_or_active_if_empty(world, call.param("displayGroupId"));
    let interval = call.float("time");
    if call.first_time {
        world.resource_mut::<Library>().stall_time = interval;
    }

    let remaining = world.resource::<Library>().stall_time;
    if remaining <= 0.0 {
        world.resource_mut::<Library>().stall_time = interval;
        let object = display_object_in(world, group, call.param("id"));
        let current_alpha = object_alpha(world, object);
        let toggled_alpha = if current_alpha == 0.0 { 1.0 } else { 0.0 };
        set_object_alpha(world, object, toggled_alpha);
    }

    world.resource_mut::<Library>().stall_time -= call.delta_time;
    ItemResult::Pending
}

/// `-[BCLibrary setAnimation:::]` @ 0x1000257a8: plays on a listener-less player kept per `id`.
pub fn seq_set_animation(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let Some(key) = call.param("id") else {
        // The original creates a player and throws on `setObject:forKey:nil`.
        report_unreachable(call, "no player id");
        return ItemResult::Done;
    };

    let library = world.resource::<Library>();
    let existing = library.animation_players.get(key).copied();
    let player = match existing {
        Some(player) => player,
        None => {
            let player = animation::spawn(world);
            let mut library = world.resource_mut::<Library>();
            library.animation_players.insert(key.to_owned(), player);
            player
        }
    };

    animation::play_animation_id(world, player, call.param("animation"));
    if call.has("percent") {
        let percent = call.float("percent");
        animation::set_frame_percent(world, player, percent);
    }
    ItemResult::Done
}

/// `-[BCLibrary setTextureDef:::]` @ 0x1000258b4.
pub fn seq_set_texture_def(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = group_or_active_if_absent(world, call.param("displayGroupId"));
    let object = display_object_in(world, group, call.param("id"));
    let texture = texture_def_by_id(world, call.param("texture"));
    if let Some(object) = object {
        display::set_texture_def(world, object, texture);
    }
    ItemResult::Done
}

/// `-[BCLibrary setBehavior:::]` @ 0x10002598c: `setBehavior:` on another object's logic.
pub fn seq_set_behavior(world: &mut World, call: &mut SequenceCall) -> ItemResult {
    let group = display_group_by_id(world, call.param("displayGroupId"));
    let object = display_object_in(world, group, call.param("id"));
    let behavior = call.param("behavior");
    let logic = match object {
        Some(object) => display::object(world, object).logic,
        None => None,
    };
    if let Some(logic) = logic {
        logic::set_behavior(world, logic, behavior);
    }
    ItemResult::Done
}

/// The pick of `chooseSequence:::` / `choosePushSequence:::`: `list[arc4random() % count]`.
/// An absent list (a nil array, count 0) still draws a number and picks nil.
fn choose_sequence_def(world: &mut World, call: &SequenceCall) -> Option<SequenceDefRef> {
    let entries = call.list("list");
    let count = entries.len() as u32;
    let index = world.resource_mut::<Random>().arc4random_mod(count);
    let chosen = entries.get(index as usize).copied();
    sequence_def_by_id(world, chosen)
}

/// The body of `hideObjects:::` / `showObjects:::`: `hide` / `show` each named child.
fn set_objects_hidden(world: &mut World, call: &SequenceCall, hidden: bool) {
    let names = call.list("idList");
    let group = group_or_active_if_empty(world, call.param("displayGroupId"));
    for name in names {
        let object = display_object_in(world, group, Some(name));
        if let Some(object) = object {
            display::object_mut(world, object).hidden = hidden;
        }
    }
}

/// `stall:::`'s random duration in `[min_time, max_time]` (`docs/re/sequences.md` §3.13): the
/// products are truncated like `fcvtzs`, the span wraps as u32, a zero span keeps the raw draw.
fn random_stall_time(world: &mut World, min_time: f32, max_time: f32) -> f32 {
    let low = trunc_i32(min_time * STALL_TIME_SCALE);
    let high = trunc_i32(max_time * STALL_TIME_SCALE);
    let span = 1i32.wrapping_sub(low).wrapping_add(high);
    let offset = world.resource_mut::<Random>().arc4random_mod(span as u32);
    let milliseconds = offset.wrapping_add(low as u32) as i32;
    milliseconds as f32 / STALL_TIME_SCALE
}

/// `[param floatValue]` when the parameter is present, else `default`.
fn float_or(call: &SequenceCall, name: &str, default: f32) -> f32 {
    match call.param(name) {
        Some(value) => ns_float_value(value),
        None => default,
    }
}

/// `[BCView activeDisplayGroup]`.
fn active_display_group(world: &World) -> Option<Entity> {
    world.resource::<BcView>().active_display_group
}

/// `[BCLibrary getDisplayGroupById:id]`: nil for a nil or unknown id.
fn display_group_by_id(world: &World, id: Option<&str>) -> Option<Entity> {
    let id = id?;
    library::display_group(world, id)
}

/// The group of the alpha items: `@""` is the active display group; anything else, nil
/// included (`[nil isEqualToString:]` is NO), is looked up with `getDisplayGroupById:`.
fn group_or_active_if_empty(world: &World, id: Option<&str>) -> Option<Entity> {
    if id == Some("") {
        return active_display_group(world);
    }
    display_group_by_id(world, id)
}

/// The group of `setPos`, `tweenPos` and `setTextureDef`: the active display group when the
/// parameter is absent.
fn group_or_active_if_absent(world: &World, id: Option<&str>) -> Option<Entity> {
    match id {
        Some(id) => library::display_group(world, id),
        None => active_display_group(world),
    }
}

/// `[group getDisplayObject:id]`: nil for a nil group, and for a nil id (which matches no child).
fn display_object_in(world: &World, group: Option<Entity>, id: Option<&str>) -> Option<Entity> {
    let group = group?;
    let id = id?;
    group::get_display_object(world, group, id)
}

/// `[object alpha]` (0 for nil).
fn object_alpha(world: &World, object: Option<Entity>) -> f32 {
    match object {
        Some(object) => display::object(world, object).alpha,
        None => 0.0,
    }
}

/// `[object setAlpha:alpha]` (ignored by nil).
fn set_object_alpha(world: &mut World, object: Option<Entity>, alpha: f32) {
    if let Some(object) = object {
        display::object_mut(world, object).set_alpha(alpha);
    }
}

/// `[BCLibrary getSequenceDefById:id]`: NULL for an unknown id, and for nil, whose lookup
/// reads an uninitialised length in the original (`docs/re/sequences.md` §3.1).
fn sequence_def_by_id(world: &World, id: Option<&str>) -> Option<SequenceDefRef> {
    let id = id?;
    library::sequence_def(world, id)
}

/// `[BCLibrary getAtlasTextureDefById:id]` (NULL for nil, like an unknown id).
fn atlas_by_id(world: &World, id: Option<&str>) -> Option<AtlasId> {
    let id = id?;
    world.resource::<Library>().atlas_id(id)
}

/// `[BCLibrary getTextureDefById:id]` (NULL for nil, like an unknown id).
fn texture_def_by_id(world: &World, id: Option<&str>) -> Option<TextureDefId> {
    let id = id?;
    library::texture_def_id(world, id)
}

/// Reports an item the original cannot survive (a NULL dereference or an Objective-C
/// exception). Shipped data never makes one; the port skips the operation instead of crashing,
/// like the sequence runner does for an unrecognised selector.
fn report_unreachable(call: &SequenceCall, reason: &str) {
    let method = call.method();
    eprintln!("pg-engine: BCLibrary `{method}:::` skipped: {reason}");
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::sequence::{BcSequence, SequenceDef, SequenceItemDef};
    use crate::sound::SoundId;
    use crate::texture::{AtlasDef, Textures};

    const SEED: u64 = 7;

    fn test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Library>();
        world.init_resource::<BcView>();
        world.init_resource::<Sounds>();
        world.init_resource::<Textures>();
        world.insert_resource(Random::with_seed(SEED));
        world
    }

    fn item(method: &str, params: &[(&str, &str)]) -> SequenceItemDef {
        let mut owned_params = Vec::new();
        for (name, value) in params {
            owned_params.push((name.to_string(), value.to_string()));
        }
        SequenceItemDef {
            method: method.to_owned(),
            params: owned_params,
        }
    }

    /// Runs one item through the `BCLibrary` dispatcher.
    fn run(world: &mut World, sequence: Entity, item: &SequenceItemDef, first_time: bool, delta_time: f32) -> ItemResult {
        let mut call = SequenceCall {
            sequence,
            first_time,
            delta_time,
            item,
        };
        let result = sequence_item(world, &mut call);
        result.expect("BCLibrary implements the item")
    }

    fn register_sequence(world: &mut World, id: &str) -> SequenceDefRef {
        let def = Arc::new(SequenceDef {
            id: id.to_owned(),
            items: Vec::new(),
        });
        let mut library = world.resource_mut::<Library>();
        library.sequence_defs.insert(id.to_owned(), def.clone());
        def
    }

    fn stack_def(world: &World, sequence: Entity, index: usize) -> Option<SequenceDefRef> {
        let state = world.get::<BcSequence>(sequence).expect("a BCSequence");
        state.stack[index].def.clone()
    }

    fn stall_time(world: &World) -> f32 {
        world.resource::<Library>().stall_time
    }

    #[test]
    fn other_methods_are_not_library_items() {
        let mut world = test_world();
        let fall = item("fall", &[]);
        let mut call = SequenceCall {
            sequence: Entity::PLACEHOLDER,
            first_time: true,
            delta_time: 0.0,
            item: &fall,
        };
        assert!(sequence_item(&mut world, &mut call).is_none());
    }

    #[test]
    fn trace_continues() {
        let mut world = test_world();
        let trace = item("trace", &[("message", "SEQUENCE STOP NO PEE")]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &trace, true, 0.0), ItemResult::Done);
    }

    #[test]
    fn stall_completes_one_frame_after_the_time_runs_out() {
        let mut world = test_world();
        let stall = item("stall", &[("time", "0.05")]);
        let sequence = Entity::PLACEHOLDER;
        assert_eq!(run(&mut world, sequence, &stall, true, 0.02), ItemResult::Pending);
        assert_eq!(run(&mut world, sequence, &stall, false, 0.02), ItemResult::Pending);
        assert_eq!(run(&mut world, sequence, &stall, false, 0.02), ItemResult::Pending);
        assert!(stall_time(&world) <= 0.0);
        assert_eq!(run(&mut world, sequence, &stall, false, 0.02), ItemResult::Done);
    }

    #[test]
    fn stall_without_a_positive_time_completes_at_once() {
        let mut world = test_world();
        let absent = item("stall", &[]);
        let negative = item("stall", &[("time", "-1")]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &absent, true, 0.02), ItemResult::Done);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &negative, true, 0.02), ItemResult::Done);
        assert_eq!(stall_time(&world), -1.0);
    }

    #[test]
    fn stall_with_a_nan_time_never_completes() {
        let mut world = test_world();
        world.resource_mut::<Library>().stall_time = f32::NAN;
        let stall = item("stall", &[("time", "1")]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &stall, false, 0.02), ItemResult::Pending);
    }

    #[test]
    fn stall_draws_whole_milliseconds_between_min_and_max() {
        let mut world = test_world();
        let stall = item("stall", &[("minTime", "0.5"), ("maxTime", "1.5")]);
        for seed in 0..200 {
            world.insert_resource(Random::with_seed(seed));
            assert_eq!(run(&mut world, Entity::PLACEHOLDER, &stall, true, 0.0), ItemResult::Pending);
            let drawn = stall_time(&world);
            assert!((0.5..=1.5).contains(&drawn), "{drawn} out of range");
            let milliseconds = drawn * 1000.0;
            assert!((milliseconds - milliseconds.round()).abs() < 1e-3);
        }
    }

    #[test]
    fn stall_prefers_min_time_over_time() {
        let mut world = test_world();
        // span = 1 - 1000 + 1000 = 1, so the draw is always exactly 1000 ms.
        let stall = item("stall", &[("time", "5"), ("minTime", "1"), ("maxTime", "1")]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &stall, true, 0.0), ItemResult::Pending);
        assert_eq!(stall_time(&world), 1.0);
    }

    #[test]
    fn stall_with_a_zero_span_keeps_the_raw_draw() {
        let mut world = test_world();
        // 0.001 s → 1 ms, 0 s → 0 ms: span = 1 - 1 + 0 = 0 and `udiv` by 0 keeps the raw value.
        let raw = Random::with_seed(SEED).arc4random();
        let milliseconds = raw.wrapping_add(1) as i32;
        let expected = milliseconds as f32 / 1000.0;
        let stall = item("stall", &[("minTime", "0.001"), ("maxTime", "0")]);
        run(&mut world, Entity::PLACEHOLDER, &stall, true, 0.0);
        assert_eq!(stall_time(&world).to_bits(), expected.to_bits());
    }

    #[test]
    fn stall_frames_completes_on_the_nth_call() {
        let mut world = test_world();
        let three = item("stallFrames", &[("frames", "3")]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &three, true, 0.0), ItemResult::Pending);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &three, false, 0.0), ItemResult::Pending);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &three, false, 0.0), ItemResult::Done);
        assert_eq!(world.resource::<Library>().stall_frames, 0);

        let one = item("stallFrames", &[("frames", "1")]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &one, true, 0.0), ItemResult::Done);

        let absent = item("stallFrames", &[]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &absent, true, 0.0), ItemResult::Done);
        assert_eq!(world.resource::<Library>().stall_frames, -1);
    }

    #[test]
    fn tween_alpha_keeps_its_timer_in_the_library() {
        let mut world = test_world();
        // No object: `[nil alpha]` is 0 and `setAlpha:` is ignored, but the timer still runs.
        let tween = item("tweenAlpha", &[("time", "0.5"), ("target", "1")]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &tween, true, 0.25), ItemResult::Pending);
        assert_eq!(world.resource::<Library>().delta_value, 2.0);
        assert_eq!(stall_time(&world), 0.25);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &tween, false, 0.25), ItemResult::Pending);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &tween, false, 0.25), ItemResult::Done);
        assert_eq!(stall_time(&world), -0.25);
    }

    #[test]
    fn tween_alpha_with_a_nan_timer_never_completes() {
        let mut world = test_world();
        world.resource_mut::<Library>().stall_time = f32::NAN;
        let tween = item("tweenAlpha", &[("time", "0.5"), ("target", "1")]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &tween, false, 0.25), ItemResult::Pending);
    }

    #[test]
    fn library_timers_are_shared_by_every_sequence() {
        let mut world = test_world();
        let tween = item("tweenAlpha", &[("time", "1"), ("target", "1")]);
        let stall = item("stall", &[("time", "0")]);
        let first_sequence = world.spawn_empty().id();
        let second_sequence = world.spawn_empty().id();
        assert_eq!(run(&mut world, first_sequence, &tween, true, 0.1), ItemResult::Pending);

        // Another sequence's stall overwrites the tween's remaining time.
        assert_eq!(run(&mut world, second_sequence, &stall, true, 0.1), ItemResult::Done);
        assert_eq!(run(&mut world, first_sequence, &tween, false, 0.1), ItemResult::Done);
    }

    #[test]
    fn sin_alpha_never_advances_its_phase() {
        let mut world = test_world();
        let sin = item("sinAlpha", &[("min", "0"), ("max", "1"), ("time", "1"), ("speed", "0.3")]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &sin, true, 0.5), ItemResult::Pending);

        // `[nil alpha]` is 0: asin((0 - 0.5) / 0.5) = -π/2 (to libm's rounding).
        let phase = world.resource::<Library>().value;
        assert!((phase + std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &sin, false, 0.5), ItemResult::Pending);
        assert_eq!(world.resource::<Library>().value.to_bits(), phase.to_bits());
    }

    #[test]
    fn blink_restarts_its_interval_and_never_completes() {
        let mut world = test_world();
        let blink = item("blink", &[("time", "2")]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &blink, true, 1.0), ItemResult::Pending);
        assert_eq!(stall_time(&world), 1.0);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &blink, false, 1.0), ItemResult::Pending);
        assert_eq!(stall_time(&world), 0.0);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &blink, false, 1.0), ItemResult::Pending);
        assert_eq!(stall_time(&world), 1.0);
    }

    #[test]
    fn run_sequence_replaces_the_stack_and_unknown_ids_stop_it() {
        let mut world = test_world();
        let first = register_sequence(&mut world, "First");
        let second = register_sequence(&mut world, "Second");
        let sequence = sequence::spawn(&mut world, Some(first), None);

        let run_second = item("runSequence", &[("id", "Second")]);
        assert_eq!(run(&mut world, sequence, &run_second, true, 0.0), ItemResult::Done);
        let root = stack_def(&world, sequence, 0).expect("a root sequence");
        assert!(Arc::ptr_eq(&root, &second));

        let run_missing = item("runSequence", &[("id", "Missing")]);
        assert_eq!(run(&mut world, sequence, &run_missing, true, 0.0), ItemResult::Done);
        assert!(stack_def(&world, sequence, 0).is_none());
        assert!(!world.get::<BcSequence>(sequence).unwrap().is_running);
    }

    #[test]
    fn push_sequence_calls_and_ignores_unknown_ids() {
        let mut world = test_world();
        let first = register_sequence(&mut world, "First");
        let second = register_sequence(&mut world, "Second");
        let sequence = sequence::spawn(&mut world, Some(first), None);

        let push_second = item("pushSequence", &[("id", "Second")]);
        assert_eq!(run(&mut world, sequence, &push_second, true, 0.0), ItemResult::Done);
        assert_eq!(world.get::<BcSequence>(sequence).unwrap().stack_index, 1);
        let pushed = stack_def(&world, sequence, 1).expect("a pushed sequence");
        assert!(Arc::ptr_eq(&pushed, &second));

        let push_missing = item("pushSequence", &[("id", "Missing")]);
        assert_eq!(run(&mut world, sequence, &push_missing, true, 0.0), ItemResult::Done);
        assert_eq!(world.get::<BcSequence>(sequence).unwrap().stack_index, 1);
    }

    #[test]
    fn choose_sequence_picks_arc4random_modulo_the_count() {
        let mut world = test_world();
        let mut defs = Vec::new();
        for name in ["A", "B", "C"] {
            defs.push(register_sequence(&mut world, name));
        }
        let root = defs[0].clone();
        let sequence = sequence::spawn(&mut world, Some(root), None);

        let expected_index = Random::with_seed(SEED).arc4random() % 3;
        let choose = item("chooseSequence", &[("list", "A,B,C")]);
        assert_eq!(run(&mut world, sequence, &choose, true, 0.0), ItemResult::Done);
        let root = stack_def(&world, sequence, 0).expect("a root sequence");
        assert!(Arc::ptr_eq(&root, &defs[expected_index as usize]));
    }

    #[test]
    fn choose_sequence_without_a_list_still_draws_and_stops() {
        let mut world = test_world();
        let first = register_sequence(&mut world, "First");
        let sequence = sequence::spawn(&mut world, Some(first), None);

        let choose = item("chooseSequence", &[]);
        assert_eq!(run(&mut world, sequence, &choose, true, 0.0), ItemResult::Done);
        assert!(stack_def(&world, sequence, 0).is_none());

        let mut reference = Random::with_seed(SEED);
        reference.arc4random();
        let next_expected = reference.arc4random();
        assert_eq!(world.resource_mut::<Random>().arc4random(), next_expected);
    }

    #[test]
    fn choose_push_sequence_pushes_the_pick() {
        let mut world = test_world();
        let first = register_sequence(&mut world, "First");
        let second = register_sequence(&mut world, "Second");
        let sequence = sequence::spawn(&mut world, Some(first), None);

        let choose = item("choosePushSequence", &[("list", "Second")]);
        assert_eq!(run(&mut world, sequence, &choose, true, 0.0), ItemResult::Done);
        let pushed = stack_def(&world, sequence, 1).expect("a pushed sequence");
        assert!(Arc::ptr_eq(&pushed, &second));
    }

    #[test]
    fn load_texture_atlas_loads_now_or_queues_for_the_next_frame() {
        let mut world = test_world();
        let slot = AtlasDef {
            id: "screenAtlas0".to_owned(),
            width: 512,
            height: 512,
            ..Default::default()
        };
        let atlas = world.resource_mut::<Textures>().add_atlas(slot);
        world.resource_mut::<Library>().atlas_ids.insert("screenAtlas0".to_owned(), atlas);

        let now_params = [("ext", "raw"), ("file", "SandTextureAtlas0"), ("id", "screenAtlas0")];
        let load_now = item("loadTextureAtlas", &now_params);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &load_now, true, 0.0), ItemResult::Done);
        let textures = world.resource::<Textures>();
        assert_eq!(textures.atlas(atlas).image.as_deref(), Some("SandTextureAtlas0"));
        assert_eq!(textures.requests.len(), 1);

        let later_params = [
            ("ext", "raw"),
            ("file", "ScreenTextureAtlas0"),
            ("id", "screenAtlas0"),
            ("delay", "1"),
        ];
        let load_later = item("loadTextureAtlas", &later_params);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &load_later, true, 0.0), ItemResult::Done);
        let pending = &world.resource::<BcView>().pending_atlas_loads;
        assert_eq!(pending, &vec![(atlas, "ScreenTextureAtlas0".to_owned())]);
        assert_eq!(world.resource::<Textures>().requests.len(), 1);
    }

    #[test]
    fn set_sound_alias_overwrites_and_ignores_unknown_sounds() {
        let mut world = test_world();
        {
            let mut library = world.resource_mut::<Library>();
            library.sounds.insert("SandBackgroundLoop".to_owned(), SoundId(3));
            library.sounds.insert("BackgroundLoop".to_owned(), SoundId(1));
        }

        let alias_params = [("soundId", "SandBackgroundLoop"), ("aliasId", "BackgroundLoop")];
        let alias = item("setSoundAlias", &alias_params);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &alias, true, 0.0), ItemResult::Done);
        assert_eq!(world.resource::<Library>().sound("BackgroundLoop"), Some(SoundId(3)));

        let unknown_params = [("soundId", "Missing"), ("aliasId", "BackgroundLoop")];
        let unknown = item("setSoundAlias", &unknown_params);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &unknown, true, 0.0), ItemResult::Done);
        assert_eq!(world.resource::<Library>().sound("BackgroundLoop"), Some(SoundId(3)));
    }

    #[test]
    fn mute_and_unmute_set_the_class_wide_flag() {
        let mut world = test_world();
        let mute = item("mute", &[]);
        let unmute = item("unmute", &[]);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &mute, true, 0.0), ItemResult::Done);
        assert!(world.resource::<Sounds>().muted);
        assert_eq!(run(&mut world, Entity::PLACEHOLDER, &unmute, true, 0.0), ItemResult::Done);
        assert!(!world.resource::<Sounds>().muted);
    }

    #[test]
    fn display_group_parameter_conventions() {
        let mut world = test_world();
        let active = world.spawn_empty().id();
        let island = world.spawn_empty().id();
        world.resource_mut::<BcView>().active_display_group = Some(active);
        world.resource_mut::<Library>().display_groups.insert("Island".to_owned(), island);

        assert_eq!(group_or_active_if_empty(&world, Some("")), Some(active));
        assert_eq!(group_or_active_if_empty(&world, None), None);
        assert_eq!(group_or_active_if_empty(&world, Some("Island")), Some(island));
        assert_eq!(group_or_active_if_empty(&world, Some("Missing")), None);

        assert_eq!(group_or_active_if_absent(&world, None), Some(active));
        assert_eq!(group_or_active_if_absent(&world, Some("")), None);
        assert_eq!(group_or_active_if_absent(&world, Some("Island")), Some(island));
    }

    #[test]
    fn items_without_a_target_do_nothing() {
        let mut world = test_world();
        let hide_params = [("displayGroupId", "Missing"), ("idList", "a,b")];
        let behavior_params = [("displayGroupId", "Missing"), ("behavior", "X"), ("id", "Tree")];
        let set_alpha = item("setAlpha", &[("alpha", "1.0")]);
        let hide = item("hideObjects", &hide_params);
        let add = item("addDisplayGroup", &[("parentId", "Missing"), ("id", "Island")]);
        let remove = item("removeDisplayGroup", &[("id", "Missing")]);
        let behavior = item("setBehavior", &behavior_params);
        let texture = item("setTextureDef", &[("texture", "BlankTexture")]);

        // No active display group either, so every lookup ends at nil.
        let target_less_items = [&set_alpha, &hide, &add, &remove, &behavior, &texture];
        for target_less in target_less_items {
            let result = run(&mut world, Entity::PLACEHOLDER, target_less, true, 0.0);
            assert_eq!(result, ItemResult::Done);
        }
    }
}
