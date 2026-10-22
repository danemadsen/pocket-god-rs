//! Start-up data loading in `PocketGodViewController::loadGame` order: texture atlases and
//! definitions, sequences, struct XML (sounds, fonts, gradients, display groups), then
//! `Timelines.bin`.
//!
//! The tables come from `pg_assets::data` (extracted from the original executable, in binary
//! order) and the original `Timelines.bin` (`pg_assets::timelines`). Registries follow the
//! original's dictionaries: a later definition with the same id replaces the earlier one.

use std::sync::Arc;

use bevy_ecs::prelude::*;
use pg_assets::data::{font_defs, gradient_defs, sequences, sound_defs, textures as texture_data};
use pg_assets::timelines::{self as timeline_file, AnimationRecord, FrameRecord, StateRecord, TextRecord, TimelineFile};

use crate::animation::{AnimationDefId, TimelineAnimation, TimelineFrame, TimelineState, Timelines};
use crate::display::text::{self, FontDef, Justification};
use crate::display::{self, build};
use crate::library::Library;
use crate::sequence::{SequenceDef, SequenceItemDef};
use crate::sound::{SoundDef, SoundId, Sounds};
use crate::texture::{self, AtlasDef, AtlasId, TextureDef, TextureDefId, Textures};
use crate::view::BcView;

/// Slots in the first atlas list of `loadGame` (0x100602950); `textures.json` concatenates
/// the two lists (`docs/re/textures.md` §5.1).
const FIRST_ATLAS_LIST_LEN: usize = 16;

/// Defs in the first texture list (0x1005f9738).
const FIRST_TEXTURE_LIST_LEN: usize = 4674;

/// Loads every definition from the embedded assets into the engine resources: the part of
/// `-[PocketGodViewController loadGame]` @ 0x10006a030 that creates `BCLibrary` and fills it,
/// up to and including `parseBinAnim:ext:`.
pub fn load_game_data(world: &mut World) {
    world.init_resource::<Library>();
    world.init_resource::<Textures>();
    world.init_resource::<Sounds>();
    world.init_resource::<Timelines>();
    world.init_resource::<BcView>();

    let texture_file = texture_data::load().expect("assets/data/textures.json matches its schema");
    let (first_atlases, second_atlases) = split_list(&texture_file.atlases, FIRST_ATLAS_LIST_LEN);
    let (first_textures, second_textures) = split_list(&texture_file.textures, FIRST_TEXTURE_LIST_LEN);
    process_atlas_texture_defs(world, first_atlases, first_textures);
    process_atlas_texture_defs(world, second_atlases, second_textures);
    load_main_atlases(world);

    process_sequence_defs(world);

    // parseStructXml: calls 1 (sounds) and 2 (fonts, then gradient defs); the display nodes
    // of calls 3–5 follow (calls 1, 2 and 5 have none).
    parse_sound_defs(world);
    parse_font_defs(world);
    parse_gradient_defs(world);
    build::build_display_groups(world);

    parse_bin_anim(world);
}

/// Splits a concatenated table into the original's first list and the rest.
fn split_list<T>(entries: &[T], first_len: usize) -> (&[T], &[T]) {
    let split_at = first_len.min(entries.len());
    entries.split_at(split_at)
}

/// `BCLibrary::processAtlasTextureDefs:textureDefs:` @ 0x1000240b0.
fn process_atlas_texture_defs(
    world: &mut World,
    atlases: &[texture_data::Atlas],
    textures: &[texture_data::TextureDef],
) {
    process_atlas_defs(world, atlases);
    process_texture_defs(world, textures);
}

/// `BCLibrary::processAtlasTextureDefs:` @ 0x100024000: register each slot, then create it.
fn process_atlas_defs(world: &mut World, atlases: &[texture_data::Atlas]) {
    for atlas in atlases {
        let atlas_id = add_atlas_texture_def(world, atlas);
        create_atlas(world, atlas_id, atlas);
    }
}

/// `BCLibrary::addAtlasTextureDef:` @ 0x1000234ec, keyed by the slot's own def id.
fn add_atlas_texture_def(world: &mut World, atlas: &texture_data::Atlas) -> AtlasId {
    let slot = AtlasDef {
        id: atlas.id.clone(),
        width: atlas.width,
        height: atlas.height,
        has_alpha: atlas.has_alpha,
        channel: atlas.channel.clone(),
        mipmap: atlas.mipmap,
        image: None,
        ..Default::default()
    };
    let atlas_id = world.resource_mut::<Textures>().add_atlas(slot);

    let mut library = world.resource_mut::<Library>();
    library.atlas_ids.insert(atlas.id.clone(), atlas_id);
    atlas_id
}

/// `FUN_1000383c4`: create the slot's texture. Format 1 goes through the PNG loader
/// (`FUN_100038168`); formats 2–10 through `atlasTextureDefInitRaw` (`FUN_100038084`), which
/// skips `no_init` slots and uploads `init_image` when there is one (`atlasTextureDefLoadRaw`
/// `FUN_100037adc`). The zlib and caches-directory flags only change how the original read
/// the file (all false in 1.48.2; the images are converted offline).
fn create_atlas(world: &mut World, atlas_id: AtlasId, atlas: &texture_data::Atlas) {
    let creates_texture = match atlas.format_code {
        1 => true,
        2..=10 => !atlas.no_init,
        _ => false,
    };
    if !creates_texture {
        return;
    }

    if let Some(image) = &atlas.init_image {
        texture::load_atlas(world, atlas_id, image);
    }
}

/// `BCLibrary::processTextureDefs:` @ 0x10002405c (the lists include each slot's own def).
fn process_texture_defs(world: &mut World, textures: &[texture_data::TextureDef]) {
    for def in textures {
        add_texture_def(world, def);
    }
}

/// `BCLibrary::addTextureDef:` @ 0x10002369c.
fn add_texture_def(world: &mut World, def: &texture_data::TextureDef) {
    let atlas = world.resource::<Library>().atlas_id(&def.atlas);
    let texture_def = TextureDef {
        id: def.id.clone(),
        atlas,
        blend: def.has_alpha,
        width: def.width as f32,
        height: def.height as f32,
        frame_width: def.original_width as f32,
        frame_height: def.original_height as f32,
        uvs: def.uvs,
        vertices: def.vertices,
    };

    let mut textures = world.resource_mut::<Textures>();
    let texture_def_id = TextureDefId(textures.defs.len() as u32);
    textures.defs.push(texture_def);

    let mut library = world.resource_mut::<Library>();
    library.texture_def_ids.insert(def.id.clone(), texture_def_id);
}

/// The uploads `loadGame` does after registering the slots: `TextureAtlas0` into `atlas0`
/// (def 0x100751f38), `TextureAtlas1` into `atlas1` (0x100767d00), then
/// `WidescreenOverlayAtlas0` into `getAtlasTextureDefById:@"widescreenOverlayAtlas0"` when the
/// view `isWideScreen` or `isTallScreen`.
fn load_main_atlases(world: &mut World) {
    load_atlas_by_id(world, "TextureAtlas0", "atlas0");
    load_atlas_by_id(world, "TextureAtlas1", "atlas1");

    let view = world.resource::<BcView>();
    let wide_or_tall = view.wide_screen || view.tall_screen;
    if wide_or_tall {
        load_atlas_by_id(world, "WidescreenOverlayAtlas0", "widescreenOverlayAtlas0");
    }
}

/// `FUN_100038d68(file, @"raw", atlas)`: upload `<file>.raw` into the slot `atlas_id`.
fn load_atlas_by_id(world: &mut World, file: &str, atlas_id: &str) {
    let atlas = world.resource::<Library>().atlas_id(atlas_id);
    let Some(atlas) = atlas else {
        panic!("loadGame: there is no atlas slot {atlas_id}");
    };
    texture::load_atlas(world, atlas, file);
}

/// `BCLibrary::processSequenceDefs:` @ 0x1000240f4 with the list at 0x100643a60.
fn process_sequence_defs(world: &mut World) {
    let sequence_list = sequences::load().expect("assets/data/sequences.json matches its schema");
    for sequence in sequence_list {
        add_sequence_def(world, sequence);
    }
}

/// `BCLibrary::addSequenceDef:` @ 0x1000239fc. Parameters stay in positional order.
fn add_sequence_def(world: &mut World, sequence: sequences::SequenceDef) {
    let mut items = Vec::with_capacity(sequence.items.len());
    for item in sequence.items {
        let item_def = SequenceItemDef {
            method: item.method,
            params: item.params.0,
        };
        items.push(item_def);
    }

    let def = SequenceDef {
        id: sequence.id.clone(),
        items,
    };
    let shared_def = Arc::new(def);

    let mut library = world.resource_mut::<Library>();
    library.sequence_defs.insert(sequence.id, shared_def);
}

/// `parseStructXml:` call 1 (root 0x10071ab98): node type 0 for every sound.
fn parse_sound_defs(world: &mut World) {
    let file = sound_defs::load().expect("assets/data/sound-defs.json matches its schema");
    for sound in &file.sounds {
        add_sound(world, sound);
    }
}

/// `-[BCSound initWithId:fileName:ext:loop:stream:multipleInstances:preload:]` @ 0x10000d648
/// (preload NO: samples load on first play) and `BCLibrary::addSound:` @ 0x100023d6c.
fn add_sound(world: &mut World, sound: &sound_defs::SoundDef) {
    let def = SoundDef {
        id: sound.id.clone(),
        file_name: sound.file_name.clone(),
        looping: sound.looping,
        multiple_instances: sound.multiple_instances,
        stream: sound.stream,
    };

    let mut sounds = world.resource_mut::<Sounds>();
    let sound_id = SoundId(sounds.defs.len() as u32);
    sounds.defs.push(def);

    let mut library = world.resource_mut::<Library>();
    library.sounds.insert(sound.id.clone(), sound_id);
}

/// `parseStructXml:` call 2 (root 0x10064a270), node type 12 for every font.
fn parse_font_defs(world: &mut World) {
    let file = font_defs::load().expect("assets/data/font-defs.json matches its schema");
    for font in &file.fonts {
        add_font_def(world, font);
    }
}

/// `-[BCFontDef initWithId:]` @ 0x100066780 (empty glyph table), `-setCharSet:` @ 0x100066a34
/// and `BCLibrary::addFontDef:` @ 0x100023af0.
fn add_font_def(world: &mut World, font: &font_defs::FontDef) {
    let library = world.resource::<Library>();
    let glyphs = font_glyphs(library, &font.id, &font.char_set);
    let def = FontDef {
        id: font.id.clone(),
        char_set: font.char_set.clone(),
        glyphs,
    };

    let mut library = world.resource_mut::<Library>();
    library.font_defs.insert(font.id.clone(), def);
}

/// `-[BCFontDef setCharSet:]` @ 0x100066a34: for `i` below `[charSet length]` (UTF-16 units),
/// the glyph of the `i`-th byte of the UTF-8 char set is the texture def
/// `"<fontId>:FullSet:<i>"` (NULL when unknown).
fn font_glyphs(library: &Library, font_id: &str, char_set: &str) -> [Option<TextureDefId>; 256] {
    let mut glyphs = [None; 256];
    let length = char_set.encode_utf16().count();
    let bytes = char_set.as_bytes();

    for (index, byte) in bytes.iter().take(length).enumerate() {
        let glyph_id = format!("{font_id}:FullSet:{index}");
        glyphs[*byte as usize] = library.texture_def_id(&glyph_id);
    }
    glyphs
}

/// `parseStructXml:` call 2 (root 0x10064a270), node type 10 for every gradient def:
/// `+[BCGradient createGradientDef:]` on the spec, then `BCLibrary::addGradientDef:id:`
/// @ 0x10002384c.
fn parse_gradient_defs(world: &mut World) {
    let file = gradient_defs::load().expect("assets/data/gradient-defs.json matches its schema");
    for gradient in &file.gradients {
        let def = display::gradient::create_gradient_def(&gradient.spec);
        let mut library = world.resource_mut::<Library>();
        library.gradient_defs.insert(gradient.id.clone(), def);
    }
}

/// `BCLibrary::parseBinAnim:ext:` @ 0x100025a08 with `@"Timelines"`, `@"bin"`.
fn parse_bin_anim(world: &mut World) {
    let file = timeline_file::load().expect("assets/timelines/Timelines.bin validates");

    for record in &file.texts {
        add_dynamic_text(world, record);
    }

    let library = world.resource::<Library>();
    let timelines = build_timelines(library, &file);

    // addAnimationDef: @ 0x100023ccc for every record, keyed by its name.
    let mut library = world.resource_mut::<Library>();
    for (index, animation) in timelines.animations.iter().enumerate() {
        let animation_def = AnimationDefId(index as u32);
        library.animation_defs.insert(animation.name.clone(), animation_def);
    }

    world.insert_resource(timelines);
}

/// `parseBinAnim:ext:` step 3 for one text record: `[[BCText alloc] initWithId:fontDef:]`
/// with `getFontDefById:` of the record's font, `setFontJustification:`, `setText:`, then
/// `BCLibrary::addDynamicText:` @ 0x100023b60. The text is never put in a display group.
fn add_dynamic_text(world: &mut World, record: &TextRecord) {
    let font_known = world.resource::<Library>().font_def(record.font).is_some();
    let font_def = if font_known { Some(record.font) } else { None };

    let created = text::spawn(world, Some(record.id), font_def);
    let justification = font_justification(record.justification);
    text::set_font_justification(world, created, justification);
    text::set_text(world, created, Some(record.text));

    let mut library = world.resource_mut::<Library>();
    library.dynamic_texts.insert(record.id.to_owned(), created);
}

/// The low 32 bits of a text record's justification (`FontJustification`; the parser only
/// accepts 0, 1 and 2).
fn font_justification(value: i64) -> Justification {
    match value as i32 {
        1 => Justification::Right,
        2 => Justification::Center,
        _ => Justification::Left,
    }
}

/// `parseBinAnim:ext:` steps 4–5: every animation, frame and state record, with each state's
/// texture id (flag 0x2000, `getTextureDefById:`) or dynamic-text id (flag 0x4000,
/// `getDynamicTextById:`) resolved once. The lookups have no side effects, so resolving the
/// states in record order gives the original's result.
fn build_timelines(library: &Library, file: &TimelineFile) -> Timelines {
    let mut states = Vec::with_capacity(file.states.len());
    for record in &file.states {
        let state = timeline_state(library, record);
        states.push(state);
    }

    let mut frames = Vec::with_capacity(file.frames.len());
    for record in &file.frames {
        let frame = timeline_frame(record);
        frames.push(frame);
    }

    let mut animations = Vec::with_capacity(file.animations.len());
    for record in &file.animations {
        let animation = timeline_animation(record);
        animations.push(animation);
    }

    Timelines {
        animations,
        frames,
        state_lists: file.state_pointers.clone(),
        states,
    }
}

/// `BCBinaryAnimationDef` accessors: `fps` and `length` read the low 32 bits, `loop` and
/// `isMoveEnabled` test for non-zero (the parser only accepts 0 and 1, so `applyMovement`'s
/// `== 1` test agrees).
fn timeline_animation(record: &AnimationRecord) -> TimelineAnimation {
    TimelineAnimation {
        name: record.name.to_owned(),
        fps: record.fps as i32,
        length: record.length as i32,
        looping: record.looping != 0,
        move_enabled: record.move_enabled != 0,
        move_x: record.move_x,
        move_y: record.move_y,
        next_animation: record.next_animation.map(str::to_owned),
        first_frame: record.first_frame as u32,
        frame_count: record.frame_count as u32,
    }
}

/// A `BinaryFrameDef`; its index is compared as its low 32 bits.
fn timeline_frame(record: &FrameRecord) -> TimelineFrame {
    TimelineFrame {
        index: record.index as i32,
        sound: record.sound.map(str::to_owned),
        action: record.action.map(str::to_owned),
        move_x: record.move_x,
        move_y: record.move_y,
        first_state: record.first_pointer as u32,
        state_count: record.state_count as u32,
    }
}

/// A frame state with its texture def or dynamic text resolved. `flags` keeps the file's low
/// 32 bits (the loader's own "fixed up" bit 31 is never read). Colours stay as stored, NaN
/// included (three unused states, `docs/re/timelines.md` §2.10): the draw code's blend test
/// sees the NaN as the original did, and `GlContext` turns NaN channels into 0.
fn timeline_state(library: &Library, record: &StateRecord) -> TimelineState {
    let mut texture = None;
    let mut dynamic_text = None;
    if let Some(texture_id) = record.texture {
        if record.flags & timeline_file::flags::TEXTURE != 0 {
            texture = library.texture_def_id(texture_id);
        } else if record.flags & timeline_file::flags::DYNAMIC_TEXT != 0 {
            dynamic_text = library.dynamic_text(texture_id);
        }
    }

    let name = record.name.unwrap_or_default();
    TimelineState {
        name: name.to_owned(),
        flags: record.flags as u32,
        texture,
        dynamic_text,
        channel: record.channel as u8,
        x: record.x,
        y: record.y,
        rotation: record.rotation,
        scale_x: record.scale_x,
        scale_y: record.scale_y,
        alpha: record.alpha,
        red: record.red,
        green: record.green,
        blue: record.blue,
        vertices: record.vertices,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::texture::TextureRequest;

    fn world_with_textures() -> World {
        let mut world = World::new();
        world.init_resource::<Library>();
        world.init_resource::<Textures>();
        world.init_resource::<Sounds>();

        let texture_file = texture_data::load().expect("textures.json matches its schema");
        let (first_atlases, second_atlases) = split_list(&texture_file.atlases, FIRST_ATLAS_LIST_LEN);
        let (first_textures, second_textures) = split_list(&texture_file.textures, FIRST_TEXTURE_LIST_LEN);
        process_atlas_texture_defs(&mut world, first_atlases, first_textures);
        process_atlas_texture_defs(&mut world, second_atlases, second_textures);
        world
    }

    #[test]
    fn the_first_lists_hold_their_own_slots_defs() {
        let texture_file = texture_data::load().expect("textures.json matches its schema");
        let (first_atlases, second_atlases) = split_list(&texture_file.atlases, FIRST_ATLAS_LIST_LEN);
        let (first_textures, second_textures) = split_list(&texture_file.textures, FIRST_TEXTURE_LIST_LEN);
        assert_eq!((first_atlases.len(), second_atlases.len()), (16, 10));
        assert_eq!((first_textures.len(), second_textures.len()), (4674, 3663));

        let first_ids: HashSet<&str> = first_atlases.iter().map(|atlas| atlas.id.as_str()).collect();
        assert!(first_textures.iter().all(|def| first_ids.contains(def.atlas.as_str())));
        assert!(second_textures.iter().all(|def| !first_ids.contains(def.atlas.as_str())));
    }

    #[test]
    fn registers_every_slot_and_def() {
        let world = world_with_textures();
        let textures = world.resource::<Textures>();
        let library = world.resource::<Library>();
        assert_eq!(textures.atlases.len(), 26);
        assert_eq!(textures.defs.len(), 8337);
        assert_eq!(library.atlas_ids.len(), 26);
        assert_eq!(library.texture_def_ids.len(), 8337);

        let atlas0 = library.atlas_id("atlas0").expect("atlas0 is registered");
        let slot = textures.atlas(atlas0);
        assert_eq!((slot.width, slot.height), (1024, 1024));
        assert_eq!(slot.channel.as_deref(), Some("MainAtlas"));
        assert_eq!(slot.image, None, "atlas0 has no init image");

        let own_def = library.texture_def_id("atlas0").expect("the slot's own def is registered");
        assert_eq!(textures.def(own_def).atlas, Some(atlas0));

        let glow = library.texture_def_id("BatTurn_05").expect("def exists");
        let def = textures.def(glow);
        assert_eq!((def.width, def.height), (75.0, 41.0));
        assert_eq!(textures.atlas(def.atlas.expect("def has a slot")).id, "pvratlas0");
    }

    #[test]
    fn creating_slots_uploads_their_init_images_in_list_order() {
        let world = world_with_textures();
        let textures = world.resource::<Textures>();

        let mut loaded = Vec::new();
        for request in &textures.requests {
            let TextureRequest::Load { image, .. } = request else {
                panic!("only whole images are loaded at start-up");
            };
            loaded.push(image.as_str());
        }
        let expected = [
            "EpisodeAlertTextureAtlas0",
            "VampireStayingAlive0",
            "TextureAtlasTest0",
            "TextureAtlasTest1",
            "TextureAtlasTest2",
            "TextureAtlasTest3",
            "TextureAtlasTest4",
            "TextureAtlasTest5",
            "TextureAtlasTest6",
            "NewsAtlas0",
            "LocustTextureAtlas0",
            "LocustTextureAtlas1",
            "LocustTextureAtlas2",
            "TextureAtlasPvr0",
            "TextureAtlasPvr1",
            "TextureAtlasPvr2",
        ];
        assert_eq!(loaded, expected);
    }

    fn main_atlas_images(wide_screen: bool, tall_screen: bool) -> Vec<String> {
        let mut world = world_with_textures();
        let view = BcView {
            wide_screen,
            tall_screen,
            ..Default::default()
        };
        world.insert_resource(view);

        let request_count = world.resource::<Textures>().requests.len();
        load_main_atlases(&mut world);

        let textures = world.resource::<Textures>();
        let mut images = Vec::new();
        for request in &textures.requests[request_count..] {
            let TextureRequest::Load { image, .. } = request else {
                panic!("only whole images are loaded at start-up");
            };
            images.push(image.clone());
        }
        images
    }

    #[test]
    fn main_atlases_load_and_the_overlay_only_on_wide_or_tall_screens() {
        assert_eq!(main_atlas_images(false, false), ["TextureAtlas0", "TextureAtlas1"]);

        let with_overlay = ["TextureAtlas0", "TextureAtlas1", "WidescreenOverlayAtlas0"];
        assert_eq!(main_atlas_images(true, false), with_overlay);
        assert_eq!(main_atlas_images(false, true), with_overlay);

        let mut world = world_with_textures();
        world.init_resource::<BcView>();
        load_main_atlases(&mut world);
        let textures = world.resource::<Textures>();
        let library = world.resource::<Library>();
        let atlas1 = library.atlas_id("atlas1").expect("atlas1 is registered");
        assert_eq!(textures.atlas(atlas1).image.as_deref(), Some("TextureAtlas1"));
    }

    #[test]
    fn sequences_keep_positional_parameters() {
        let mut world = World::new();
        world.init_resource::<Library>();
        process_sequence_defs(&mut world);

        let library = world.resource::<Library>();
        assert_eq!(library.sequence_defs.len(), 2548);

        let garbage = library.sequence_def("Garbage").expect("Garbage exists");
        let first_item = &garbage.items[0];
        assert_eq!(first_item.method, "setSoundAlias");
        let names: Vec<&str> = first_item.params.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["soundId", "aliasId"]);
        assert_eq!(first_item.params[0].1, "SandBackgroundLoop");

        let empty = library.sequence_def("CoconutStopSlide").expect("CoconutStopSlide exists");
        assert!(empty.items.is_empty());
        assert!(library.sequence_def("NoSuchSequence").is_none());
    }

    #[test]
    fn sounds_are_registered_in_parse_order() {
        let mut world = World::new();
        world.init_resource::<Library>();
        world.init_resource::<Sounds>();
        parse_sound_defs(&mut world);

        let sounds = world.resource::<Sounds>();
        let library = world.resource::<Library>();
        assert_eq!(sounds.defs.len(), 359);
        assert_eq!(library.sounds.len(), 359);

        let first = library.sound("RockBackgroundLoop").expect("sound exists");
        assert_eq!(first, SoundId(0));
        let def = sounds.def(first);
        assert_eq!(def.file_name, "jungleadventure");
        assert!(def.looping && !def.multiple_instances && !def.stream);

        let vibrate = library.sound("Vibrate").expect("Vibrate exists");
        assert_eq!(sounds.def(vibrate).file_name, "");
    }

    #[test]
    fn font_glyphs_come_from_the_full_set_defs() {
        let mut world = world_with_textures();
        parse_font_defs(&mut world);

        let library = world.resource::<Library>();
        assert_eq!(library.font_defs.len(), 4);

        let doop = library.font_def("Doop24Green").expect("font exists");
        assert_eq!(doop.glyphs[b' ' as usize], library.texture_def_id("Doop24Green:FullSet:0"));
        assert_eq!(doop.glyphs[b'L' as usize], library.texture_def_id("Doop24Green:FullSet:1"));
        assert_eq!(doop.glyphs[b'9' as usize], library.texture_def_id("Doop24Green:FullSet:11"));
        assert!(doop.glyphs[b'9' as usize].is_some());
        assert_eq!(doop.glyphs[b'A' as usize], None);

        let mut glyph_count = 0;
        for font in library.font_defs.values() {
            glyph_count += font.glyphs.iter().filter(|glyph| glyph.is_some()).count();
        }
        assert_eq!(glyph_count, 3 * 95 + 12, "all 297 glyph defs exist");
    }

    #[test]
    fn gradient_defs_match_the_extractor() {
        let mut world = World::new();
        world.init_resource::<Library>();
        parse_gradient_defs(&mut world);

        let library = world.resource::<Library>();
        let file = gradient_defs::load().expect("gradient-defs.json matches its schema");
        assert_eq!(library.gradient_defs.len(), 15);
        for gradient in &file.gradients {
            let def = library.gradient_def(&gradient.id).expect("gradient def is registered");
            assert_eq!(def.colors.len(), gradient.stops.len(), "{}", gradient.id);
            for (color, stop) in def.colors.iter().zip(&gradient.stops) {
                assert_eq!(color.pos.to_bits(), stop.position.to_bits(), "{}", gradient.id);
                let rgba = [color.r, color.g, color.b, color.a];
                assert_eq!(rgba, stop.rgba, "{}", gradient.id);
            }
        }

        // StartGradient keeps its seven-digit colour 0xf5FDEFE (alpha 0x0f).
        let start = library.gradient_def("StartGradient").expect("StartGradient exists");
        assert!(start.colors.iter().any(|color| color.a == 15.0 / 255.0));
    }

    #[test]
    fn timeline_states_resolve_textures_and_dynamic_texts() {
        let world = world_with_textures();
        let texture_def_ids = world.resource::<Library>().texture_def_ids.clone();
        let mut library = Library {
            texture_def_ids,
            ..Default::default()
        };

        let file = timeline_file::load().expect("Timelines.bin validates");
        let mut fake_world = World::new();
        for record in &file.texts {
            let placeholder = fake_world.spawn_empty().id();
            library.dynamic_texts.insert(record.id.to_owned(), placeholder);
        }

        let timelines = build_timelines(&library, &file);
        assert_eq!(timelines.animations.len(), 2354);
        assert_eq!(timelines.frames.len(), 56_175);
        assert_eq!(timelines.state_lists.len(), 304_286);
        assert_eq!(timelines.states.len(), 81_294);

        let textured = timelines.states.iter().filter(|state| state.texture.is_some()).count();
        assert_eq!(textured, 78_556, "every texture id resolves");
        let with_text = timelines.states.iter().filter(|state| state.dynamic_text.is_some()).count();
        assert_eq!(with_text, 508, "every dynamic text id resolves");

        for state_index in [11_388, 70_085, 75_878] {
            let state = &timelines.states[state_index];
            let colors = [state.alpha, state.red, state.green, state.blue];
            assert!(colors.iter().all(|value| value.is_nan()), "state {state_index} keeps its NaN colour");
        }

        let first = &timelines.states[0];
        assert_eq!(first.name, "AchievementBG");
        assert_eq!(first.flags, 0x2323);
        assert_eq!(first.texture, library.texture_def_id("AchievementBG"));

        let animation = &timelines.animations[0];
        assert_eq!(animation.name, "AchievementGenericAnim");
        assert_eq!((animation.fps, animation.length, animation.first_frame, animation.frame_count), (20, 104, 0, 104));
        let frame = &timelines.frames[0];
        assert_eq!(frame.sound.as_deref(), Some("AchievementBell"));
        assert_eq!((frame.first_state, frame.state_count), (0, 13));
    }
}
