//! `BCSound`: sound objects, OpenAL instances, aliases, mute and vibration
//! (`docs/re/audio-prefs-text.md` §1).
//!
//! The engine keeps the original's sound state exactly: every `BCSound` object (its playing
//! flag, current instance and preload pin), the class-wide mute flags, and the C++ sound
//! manager's 32 instance slots with the state of each instance's OpenAL source. Playback
//! happens in the platform audio bridge, which drains [`Sounds::commands`] every frame and
//! reports through [`Sounds::voice_finished`] when a voice reaches the end of its sample: the
//! moment the original's source turned `AL_STOPPED` by itself, which `isPlaying` and the
//! instance garbage collection observe.
//!
//! Bridge contract:
//! - A voice is one OpenAL source. [`SoundCommand::Play`] (re)starts it from the first frame
//!   (`alSourcePlay` rewinds a playing source), replacing whatever the voice was playing.
//! - Samples are `assets/sounds/<file_name>.flac`, decoded to interleaved i16 and passed once
//!   through [`apply_click_fix`] (the original's loader does it to every buffer); they play at
//!   their own sample rate.
//! - The original has no volume, pitch, pan or fade: every voice plays at gain 1.0 at the
//!   listener position. Mono samples play at full amplitude on both channels (open question
//!   6.1 of the spec).
//! - Apply a frame's commands in order before reporting finished voices. A report carries the
//!   `serial` of the `Play` it ends; stale reports are ignored.

use bevy_ecs::prelude::*;

use crate::library::Library;

/// Instance slots of the sound manager (`+0x20`: 32 `BCSoundInstance*`).
pub const INSTANCE_SLOT_COUNT: usize = 32;

/// The id `initWithId:…` turns into the vibration pseudo-sound (`@"Vibrate"`).
pub const VIBRATE_SOUND_ID: &str = "Vibrate";

/// Index of a `BCSound` object in [`Sounds::defs`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SoundId(pub u32);

/// A sound definition: the arguments of
/// `initWithId:fileName:ext:loop:stream:multipleInstances:preload:`, which the object keeps as
/// `mId`, `mFileName`, `mbLoop`, `mbStream` and `mbMultipleInstances`.
#[derive(Clone, Debug, Default)]
pub struct SoundDef {
    pub id: String,

    /// `assets/sounds/<file_name>.flac` (empty for `Vibrate`).
    pub file_name: String,
    pub looping: bool,
    pub multiple_instances: bool,
    pub stream: bool,
}

impl SoundDef {
    /// `mbVibrate`: `initWithId:…` sets it for the id `Vibrate` (and then never resolves a
    /// resource path).
    pub fn vibrate(&self) -> bool {
        self.id == VIBRATE_SOUND_ID
    }

    /// `mResourcePath`: the sample's asset path.
    pub fn resource_path(&self) -> String {
        sound_asset_path(&self.file_name)
    }
}

/// The changing ivars of a `BCSound`. A new object has the default state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoundState {
    pub playing: bool,
    pub preload: bool,

    /// `mpSoundInstance`: the voice of the current instance.
    pub sound_instance: Option<u64>,
}

/// `AL_SOURCE_STATE` of an instance's source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceState {
    #[default]
    Initial,
    Playing,
    Stopped,
}

/// `BCSoundInstance`: one OpenAL source with its buffer, held in a slot of the sound manager.
#[derive(Clone, Debug)]
pub struct SoundInstance {
    /// The handle the audio bridge knows this source by.
    pub voice: u64,

    /// `+0x08`: the `BCSound` told about the deletion (`onSoundInstanceComplete:`).
    pub owner: SoundId,

    /// `+0x10`: the owner's resource path.
    pub resource_path: String,

    /// `+0x229`: `AL_LOOPING`.
    pub looping: bool,

    /// `+0x22a`: pinned by `preload`; garbage collection keeps it.
    pub preload: bool,
    pub state: SourceState,

    /// Serial of the latest `Play` of this voice, echoed by the bridge when the sample ends.
    pub serial: u64,

    /// `AL_SEC_OFFSET`, as last reported by the bridge.
    pub offset: f32,
}

/// A request for the platform audio bridge.
#[derive(Clone, Debug, PartialEq)]
pub enum SoundCommand {
    /// `alSourcei(AL_LOOPING)` + `alSourcePlay` (`FUN_10000e0e0`): play `asset` (relative to
    /// `assets/`) on `voice` from its first frame, replacing whatever the voice was playing.
    /// The end of a non-looping sample is reported with [`Sounds::voice_finished`].
    Play {
        voice: u64,
        serial: u64,
        asset: String,
        looping: bool,
    },

    /// `alSourceStop` (`FUN_10000e180`).
    Stop { voice: u64 },

    /// `AudioServicesPlaySystemSound(kSystemSoundID_Vibrate)`: rumble, or nothing.
    Vibrate,

    /// Starts the RemoteIO stream player on `asset` (`FUN_100068c20`). No sound of 1.48.2
    /// streams, so this is never sent with the shipped data.
    StreamStart { asset: String },

    /// Stops the stream player (`FUN_100068cb8`).
    StreamStop,
}

/// All sound state: the `BCSound` objects, the class-wide flags and the sound manager.
#[derive(Resource, Default)]
pub struct Sounds {
    /// Every `BCSound` object's definition, indexed by [`SoundId`].
    pub defs: Vec<SoundDef>,

    /// The changing ivars of each object, indexed like `defs`; a missing entry is a new
    /// object's state (the loader only pushes definitions).
    pub states: Vec<SoundState>,

    /// Requests for the audio bridge, in order; the bridge drains them every frame.
    pub commands: Vec<SoundCommand>,

    /// `+[BCSound mute:]` (`DAT_1007cd7f0`).
    pub muted: bool,

    /// `+[BCSound muteVibrate:]` (`DAT_1007cd7f1`).
    pub vibrate_muted: bool,

    /// The sound manager's instance slots (`DAT_1007e3020 + 0x20`). The first `initWithId:…`
    /// creates the manager (`FUN_10000de78`, which also opens the OpenAL device and context
    /// the bridge owns), so it exists whenever there is a sound to play.
    pub slots: [Option<SoundInstance>; INSTANCE_SLOT_COUNT],

    /// The handle of the next instance's voice.
    pub next_instance: u64,

    /// The serial of the next `Play`.
    pub next_serial: u64,
}

impl Sounds {
    /// The definition of `sound`.
    pub fn def(&self, sound: SoundId) -> &SoundDef {
        &self.defs[sound.0 as usize]
    }

    /// The changing ivars of `sound`.
    pub fn state(&self, sound: SoundId) -> SoundState {
        let index = sound.0 as usize;
        match self.states.get(index) {
            Some(state) => *state,
            None => SoundState::default(),
        }
    }

    fn state_mut(&mut self, sound: SoundId) -> &mut SoundState {
        let index = sound.0 as usize;
        if self.states.len() <= index {
            self.states.resize(index + 1, SoundState::default());
        }
        &mut self.states[index]
    }

    /// The instance whose source is `voice`, unless it has been deleted.
    pub fn instance(&self, voice: u64) -> Option<&SoundInstance> {
        let mut instances = self.slots.iter().flatten();
        instances.find(|instance| instance.voice == voice)
    }

    fn instance_mut(&mut self, voice: u64) -> Option<&mut SoundInstance> {
        let mut instances = self.slots.iter_mut().flatten();
        instances.find(|instance| instance.voice == voice)
    }

    /// `-[BCSound initWithId:fileName:ext:loop:stream:multipleInstances:preload:]` @ 0x10000d648.
    pub fn init_sound(&mut self, def: SoundDef, preload: bool) -> SoundId {
        let looping = def.looping;
        let index = u32::try_from(self.defs.len()).expect("fewer than 2^32 sounds");
        let sound = SoundId(index);
        self.defs.push(def);

        let state = self.state_mut(sound);
        *state = SoundState::default();
        state.preload = preload;
        if preload {
            let instance = self.create_instance(sound, looping, preload);
            self.state_mut(sound).sound_instance = instance;
        }
        sound
    }

    /// `FUN_10000d8c4`: garbage-collects, then creates an instance in the first free slot;
    /// `None` when all 32 slots are taken. The original also loads the sample into the new
    /// instance's buffer here; the bridge keeps every sample resident instead.
    fn create_instance(&mut self, owner: SoundId, looping: bool, preload: bool) -> Option<u64> {
        self.collect_garbage();

        let resource_path = self.def(owner).resource_path();
        let voice = self.next_instance;
        let mut slots = self.slots.iter_mut();
        let free_slot = slots.find(|slot| slot.is_none())?;
        let instance = SoundInstance {
            voice,
            owner,
            resource_path,
            looping,
            preload,
            state: SourceState::Initial,
            serial: 0,
            offset: 0.0,
        };
        *free_slot = Some(instance);
        self.next_instance += 1;
        Some(voice)
    }

    /// `FUN_10000e060`: deletes every instance whose source is not playing and that is not
    /// pinned (`alDeleteSources`, `alDeleteBuffers`, `free`), telling its owner.
    fn collect_garbage(&mut self) {
        for index in 0..INSTANCE_SLOT_COUNT {
            let slot = &mut self.slots[index];
            let deleted = slot.take_if(|instance| is_collectable(instance));
            if let Some(instance) = deleted {
                self.on_sound_instance_complete(instance.owner, instance.voice);
            }
        }
    }

    /// `-[BCSound onSoundInstanceComplete:]` @ 0x10000da24.
    fn on_sound_instance_complete(&mut self, sound: SoundId, voice: u64) {
        let state = self.state_mut(sound);
        if state.sound_instance == Some(voice) && !state.preload {
            state.sound_instance = None;
        }
    }

    /// `FUN_10000e0e0`: `alSourcei(AL_LOOPING)` and `alSourcePlay` on the instance's source.
    fn instance_play(&mut self, voice: u64) {
        let serial = self.next_serial;
        let command = {
            let Some(instance) = self.instance_mut(voice) else {
                // `mpSoundInstance` outlived its instance (only after a preload that found no
                // free slot): the original plays a deleted source, which fails silently.
                return;
            };
            instance.state = SourceState::Playing;
            instance.serial = serial;
            instance.offset = 0.0;
            SoundCommand::Play {
                voice,
                serial,
                asset: instance.resource_path.clone(),
                looping: instance.looping,
            }
        };
        self.next_serial += 1;
        self.commands.push(command);
    }

    /// `FUN_10000e180`: `alSourceStop` on the instance's source.
    fn instance_stop(&mut self, voice: u64) {
        let Some(instance) = self.instance_mut(voice) else {
            return;
        };
        // Stopping an initial or stopped source is a legal no-op in OpenAL.
        if instance.state != SourceState::Playing {
            return;
        }
        instance.state = SourceState::Stopped;
        instance.offset = 0.0;
        self.commands.push(SoundCommand::Stop { voice });
    }

    /// `-[BCSound play]` @ 0x10000db20.
    pub fn play(&mut self, sound: SoundId) {
        let def = self.def(sound);
        let looping = def.looping;
        let vibrate = def.vibrate();
        let stream = def.stream;
        let multiple_instances = def.multiple_instances;
        let state = self.state(sound);

        if looping && state.playing {
            // A running loop is never restarted.
            return;
        }
        if vibrate {
            // Vibration honours only its own switch; `playing` is left alone.
            if !self.vibrate_muted {
                self.commands.push(SoundCommand::Vibrate);
            }
            return;
        }
        if stream {
            if !self.muted {
                self.state_mut(sound).playing = true;
                let asset = self.def(sound).resource_path();
                self.commands.push(SoundCommand::StreamStart { asset });
            }
            return;
        }
        if self.muted {
            return;
        }

        self.state_mut(sound).playing = true;
        let mut instance = state.sound_instance;
        if instance.is_none() || multiple_instances {
            instance = self.create_instance(sound, looping, false);
            self.state_mut(sound).sound_instance = instance;
        }
        let Some(voice) = instance else {
            // All 32 slots are taken: the sound is dropped silently.
            return;
        };
        self.instance_play(voice);
        self.collect_garbage();
    }

    /// `-[BCSound stop]` @ 0x10000dca8. Only the current instance stops: older voices of a
    /// multiple-instance sound play on.
    pub fn stop(&mut self, sound: SoundId) {
        let stream = self.def(sound).stream;
        let instance = {
            let state = self.state_mut(sound);
            if !state.playing {
                return;
            }
            state.playing = false;
            state.sound_instance
        };
        if stream {
            self.commands.push(SoundCommand::StreamStop);
            return;
        }
        let Some(voice) = instance else {
            return;
        };
        self.instance_stop(voice);
        self.collect_garbage();

        let state = self.state_mut(sound);
        if !state.preload {
            state.sound_instance = None;
        }
    }

    /// `-[BCSound isPlaying]` @ 0x10000dd60: playing, with a current instance whose source is
    /// `AL_PLAYING` (a finished one-shot reports NO although `playing` stays set).
    pub fn is_playing(&self, sound: SoundId) -> bool {
        let state = self.state(sound);
        if !state.playing {
            return false;
        }
        let Some(voice) = state.sound_instance else {
            return false;
        };
        match self.instance(voice) {
            Some(instance) => instance.state == SourceState::Playing,
            None => false,
        }
    }

    /// `-[BCSound playTime]` @ 0x10000ddc0: `AL_SEC_OFFSET` of the current instance, 0 when not
    /// playing.
    pub fn play_time(&self, sound: SoundId) -> f32 {
        let state = self.state(sound);
        if !state.playing {
            return 0.0;
        }
        let Some(voice) = state.sound_instance else {
            return 0.0;
        };
        match self.instance(voice) {
            Some(instance) => instance.offset,
            None => 0.0,
        }
    }

    /// `-[BCSound preload]` @ 0x10000da50: pins the current instance, creating one if needed.
    pub fn preload(&mut self, sound: SoundId) {
        let looping = self.def(sound).looping;
        let instance = {
            let state = self.state_mut(sound);
            if state.preload {
                return;
            }
            state.preload = true;
            state.sound_instance
        };
        match instance {
            None => {
                let created = self.create_instance(sound, looping, true);
                self.state_mut(sound).sound_instance = created;
            }
            Some(voice) => {
                if let Some(instance) = self.instance_mut(voice) {
                    instance.preload = true;
                }
            }
        }
    }

    /// `-[BCSound unload]` @ 0x10000daf0: unpins the current instance, which a later garbage
    /// pass deletes once it is not playing. The sound keeps playing.
    pub fn unload(&mut self, sound: SoundId) {
        let instance = {
            let state = self.state_mut(sound);
            if !state.preload {
                return;
            }
            state.preload = false;
            state.sound_instance
        };
        let Some(voice) = instance else {
            return;
        };
        if let Some(instance) = self.instance_mut(voice) {
            instance.preload = false;
        }
    }

    /// Reported by the audio bridge: the sample started by the `Play` with `serial` on `voice`
    /// ended, so the source is now `AL_STOPPED`. Stale reports (the voice was restarted,
    /// stopped or deleted since) and reports for looping voices, which never end, are ignored.
    pub fn voice_finished(&mut self, voice: u64, serial: u64) {
        let Some(instance) = self.instance_mut(voice) else {
            return;
        };
        let current = instance.serial == serial && instance.state == SourceState::Playing;
        if !current || instance.looping {
            return;
        }
        instance.state = SourceState::Stopped;
        instance.offset = 0.0;
    }

    /// Reported by the audio bridge (optional): the playback offset in seconds
    /// (`AL_SEC_OFFSET`) of the `Play` with `serial` on `voice`.
    pub fn voice_offset(&mut self, voice: u64, serial: u64, seconds: f32) {
        let Some(instance) = self.instance_mut(voice) else {
            return;
        };
        if instance.serial == serial && instance.state == SourceState::Playing {
            instance.offset = seconds;
        }
    }
}

/// The garbage collector's test: the source is not `AL_PLAYING` and the instance is not pinned.
fn is_collectable(instance: &SoundInstance) -> bool {
    instance.state != SourceState::Playing && !instance.preload
}

/// The asset path of a sample, relative to `assets/` (`sound/<file_name>.caf` in the bundle).
pub fn sound_asset_path(file_name: &str) -> String {
    format!("sounds/{file_name}.flac")
}

/// The click fix of the original's loader (`FUN_10000e638`), for the bridge to apply to every
/// decoded sample: each of the last three 16-bit values of the interleaved data that holds
/// -32768 becomes 0.
pub fn apply_click_fix(samples: &mut [i16]) {
    // The original indexes from `(byteCount - 6) >> 1` without a length check; no shipped
    // sample is shorter than three values.
    if samples.len() < 3 {
        return;
    }
    let first = samples.len() - 3;
    for sample in &mut samples[first..] {
        if *sample == i16::MIN {
            *sample = 0;
        }
    }
}

/// `-[BCSound initWithId:fileName:ext:loop:stream:multipleInstances:preload:]` @ 0x10000d648.
pub fn init_sound(world: &mut World, def: SoundDef, preload: bool) -> SoundId {
    let mut sounds = world.resource_mut::<Sounds>();
    sounds.init_sound(def, preload)
}

/// `-[BCLibrary addSound:]` @ 0x100023d6c: registers the sound under its id (a later sound
/// with the same id replaces it).
pub fn add_sound(world: &mut World, sound: SoundId) {
    let id = world.resource::<Sounds>().def(sound).id.clone();
    let mut library = world.resource_mut::<Library>();
    library.sounds.insert(id, sound);
}

/// A sound node (type 0) of `-[BCLibrary parseStructXml:]` @ 0x100026674:
/// `initWithId:fileName:ext:loop:stream:multipleInstances:preload:NO`, then `addSound:`.
pub fn add_sound_def(world: &mut World, def: SoundDef) -> SoundId {
    let sound = init_sound(world, def, false);
    add_sound(world, sound);
    sound
}

/// `-[BCLibrary getSoundById:]` @ 0x100023d50: case-sensitive; `None` (nil) for unknown ids.
pub fn sound_by_id(world: &World, id: &str) -> Option<SoundId> {
    world.resource::<Library>().sound(id)
}

/// `-[BCLibrary setSoundAlias:sound:]` @ 0x100023dc0: `alias` becomes another key of the same
/// object (shared playing flag and instance), replacing what it named; nil sounds are ignored.
/// `playBackgroundLoop:` points `BackgroundLoop` at the island's music this way.
pub fn set_sound_alias(world: &mut World, alias: &str, sound: Option<SoundId>) {
    let mut library = world.resource_mut::<Library>();
    library.set_sound_alias(alias, sound);
}

/// `-[BCSound id]` @ 0x10000d970.
pub fn id(world: &World, sound: SoundId) -> String {
    world.resource::<Sounds>().def(sound).id.clone()
}

/// `+[BCSound mute:]` @ 0x10000d628. Muting stops nothing by itself; callers follow it with
/// `stopAllSounds`.
pub fn mute(world: &mut World, mute: bool) {
    world.resource_mut::<Sounds>().muted = mute;
}

/// `+[BCSound muteVibrate:]` @ 0x10000d638.
pub fn mute_vibrate(world: &mut World, mute: bool) {
    world.resource_mut::<Sounds>().vibrate_muted = mute;
}

/// `-[BCSound play]` @ 0x10000db20.
pub fn play(world: &mut World, sound: SoundId) {
    world.resource_mut::<Sounds>().play(sound);
}

/// `-[BCSound stop]` @ 0x10000dca8.
pub fn stop(world: &mut World, sound: SoundId) {
    world.resource_mut::<Sounds>().stop(sound);
}

/// `-[BCSound isPlaying]` @ 0x10000dd60.
pub fn is_playing(world: &World, sound: SoundId) -> bool {
    world.resource::<Sounds>().is_playing(sound)
}

/// `-[BCSound playTime]` @ 0x10000ddc0.
pub fn play_time(world: &World, sound: SoundId) -> f32 {
    world.resource::<Sounds>().play_time(sound)
}

/// `-[BCSound preload]` @ 0x10000da50.
pub fn preload(world: &mut World, sound: SoundId) {
    world.resource_mut::<Sounds>().preload(sound);
}

/// `-[BCSound unload]` @ 0x10000daf0.
pub fn unload(world: &mut World, sound: SoundId) {
    world.resource_mut::<Sounds>().unload(sound);
}

/// `-[BCSound test]` @ 0x10000dd38 (`FUN_10000e1c8`): logs the current instance's source
/// (`"TEST %s %i %i %f %f %d"`: path, buffer, source, offset, offset / 60, playing).
pub fn test(world: &World, sound: SoundId) {
    let sounds = world.resource::<Sounds>();
    let state = sounds.state(sound);
    if !state.playing {
        return;
    }
    let Some(voice) = state.sound_instance else {
        return;
    };
    let Some(instance) = sounds.instance(voice) else {
        return;
    };
    let offset = instance.offset;
    let minutes = offset / 60.0;
    let playing = i32::from(instance.state == SourceState::Playing);
    let path = &instance.resource_path;
    eprintln!("TEST {path} {voice} {voice} {offset:.6} {minutes:.6} {playing}");
}

/// `[[BCLibrary getSoundById:id] play]`: unknown ids do nothing (a message to nil).
pub fn play_sound(world: &mut World, id: &str) {
    let Some(sound) = sound_by_id(world, id) else {
        return;
    };
    play(world, sound);
}

/// `[[BCLibrary getSoundById:id] stop]`.
pub fn stop_sound(world: &mut World, id: &str) {
    let Some(sound) = sound_by_id(world, id) else {
        return;
    };
    stop(world, sound);
}

/// `[[BCLibrary getSoundById:id] isPlaying]` (NO for unknown ids).
pub fn is_sound_playing(world: &World, id: &str) -> bool {
    match sound_by_id(world, id) {
        Some(sound) => is_playing(world, sound),
        None => false,
    }
}

/// `[[BCLibrary getSoundById:id] preload]`.
pub fn preload_sound(world: &mut World, id: &str) {
    let Some(sound) = sound_by_id(world, id) else {
        return;
    };
    preload(world, sound);
}

/// `[[BCLibrary getSoundById:id] unload]`.
pub fn unload_sound(world: &mut World, id: &str) {
    let Some(sound) = sound_by_id(world, id) else {
        return;
    };
    unload(world, sound);
}

/// `-[BCLibrary stopAllSounds]` @ 0x100024b4c: `stop` on the object of every key, aliases
/// included. The original enumerates in hash order; stopping an object twice does nothing
/// (it is no longer playing), so each object is stopped once, in a stable order.
pub fn stop_all_sounds(world: &mut World) {
    let library = world.resource::<Library>();
    let mut targets: Vec<SoundId> = library.sounds.values().copied().collect();
    targets.sort();
    targets.dedup();

    let mut sounds = world.resource_mut::<Sounds>();
    for sound in targets {
        sounds.stop(sound);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Library>();
        world.init_resource::<Sounds>();
        world
    }

    fn def(id: &str, looping: bool, multiple_instances: bool) -> SoundDef {
        SoundDef {
            id: id.to_owned(),
            file_name: id.to_lowercase(),
            looping,
            multiple_instances,
            stream: false,
        }
    }

    fn register(world: &mut World, def: SoundDef) -> SoundId {
        add_sound_def(world, def)
    }

    fn take_commands(world: &mut World) -> Vec<SoundCommand> {
        let mut sounds = world.resource_mut::<Sounds>();
        std::mem::take(&mut sounds.commands)
    }

    fn live_instances(world: &World) -> Vec<SoundInstance> {
        let sounds = world.resource::<Sounds>();
        let instances = sounds.slots.iter().flatten();
        instances.cloned().collect()
    }

    fn state(world: &World, sound: SoundId) -> SoundState {
        world.resource::<Sounds>().state(sound)
    }

    fn finish(world: &mut World, voice: u64) {
        let mut sounds = world.resource_mut::<Sounds>();
        let serial = sounds.instance(voice).expect("live voice").serial;
        sounds.voice_finished(voice, serial);
    }

    fn played_voices(commands: &[SoundCommand]) -> Vec<(u64, u64)> {
        let mut voices = Vec::new();
        for command in commands {
            if let SoundCommand::Play { voice, serial, .. } = command {
                voices.push((*voice, *serial));
            }
        }
        voices
    }

    #[test]
    fn replaying_a_one_shot_restarts_its_voice() {
        let mut world = test_world();
        let splash = register(&mut world, def("Splash", false, false));
        play_sound(&mut world, "Splash");
        play_sound(&mut world, "Splash");

        let commands = take_commands(&mut world);
        let voices = played_voices(&commands);
        assert_eq!(voices.len(), 2);
        assert_eq!(voices[0].0, voices[1].0);
        assert!(voices[1].1 > voices[0].1);
        assert_eq!(live_instances(&world).len(), 1);
        assert!(is_playing(&world, splash));

        let expected = SoundCommand::Play {
            voice: voices[0].0,
            serial: voices[0].1,
            asset: "sounds/splash.flac".to_owned(),
            looping: false,
        };
        assert_eq!(commands[0], expected);
    }

    #[test]
    fn sounds_pushed_by_the_loader_start_in_the_new_object_state() {
        let mut world = test_world();
        let pushed = def("Crunch", false, false);
        world.resource_mut::<Sounds>().defs.push(pushed);
        let crunch = SoundId(0);
        world
            .resource_mut::<Library>()
            .sounds
            .insert("Crunch".to_owned(), crunch);

        assert_eq!(state(&world, crunch), SoundState::default());
        play_sound(&mut world, "Crunch");
        assert!(is_playing(&world, crunch));
        assert_eq!(played_voices(&take_commands(&mut world)).len(), 1);
    }

    #[test]
    fn a_running_loop_is_never_restarted() {
        let mut world = test_world();
        let wind = register(&mut world, def("Wind", true, false));
        play(&mut world, wind);
        play(&mut world, wind);
        assert_eq!(played_voices(&take_commands(&mut world)).len(), 1);

        stop(&mut world, wind);
        let stop_commands = take_commands(&mut world);
        assert!(matches!(stop_commands.as_slice(), [SoundCommand::Stop { .. }]));
        assert!(!is_playing(&world, wind));
        assert_eq!(state(&world, wind).sound_instance, None);
        assert!(live_instances(&world).is_empty());

        play(&mut world, wind);
        assert_eq!(played_voices(&take_commands(&mut world)).len(), 1);
    }

    #[test]
    fn looping_voices_never_finish() {
        let mut world = test_world();
        let wind = register(&mut world, def("Wind", true, false));
        play(&mut world, wind);
        let voice = state(&world, wind).sound_instance.expect("voice");
        finish(&mut world, voice);
        assert!(is_playing(&world, wind));
    }

    #[test]
    fn multiple_instance_sounds_stack_voices_and_stop_only_the_newest() {
        let mut world = test_world();
        let tone = register(&mut world, def("TempleTone1", false, true));
        for _ in 0..3 {
            play(&mut world, tone);
        }
        let voices = played_voices(&take_commands(&mut world));
        assert_eq!(voices.len(), 3);
        assert_ne!(voices[0].0, voices[1].0);
        assert_ne!(voices[1].0, voices[2].0);
        assert_eq!(state(&world, tone).sound_instance, Some(voices[2].0));

        stop_all_sounds(&mut world);
        let commands = take_commands(&mut world);
        assert_eq!(commands, vec![SoundCommand::Stop { voice: voices[2].0 }]);

        let instances = live_instances(&world);
        assert_eq!(instances.len(), 2);
        assert!(instances.iter().all(|instance| instance.state == SourceState::Playing));
    }

    #[test]
    fn muted_sounds_neither_play_nor_stop() {
        let mut world = test_world();
        let splash = register(&mut world, def("Splash", false, false));
        mute(&mut world, true);
        play(&mut world, splash);
        stop(&mut world, splash);
        assert!(take_commands(&mut world).is_empty());
        assert!(!state(&world, splash).playing);

        mute(&mut world, false);
        play(&mut world, splash);
        assert_eq!(played_voices(&take_commands(&mut world)).len(), 1);
    }

    #[test]
    fn vibrate_honours_only_the_vibrate_switch() {
        let mut world = test_world();
        let vibrate = register(&mut world, def(VIBRATE_SOUND_ID, false, false));

        mute(&mut world, true);
        play(&mut world, vibrate);
        assert_eq!(take_commands(&mut world), vec![SoundCommand::Vibrate]);

        mute_vibrate(&mut world, true);
        play(&mut world, vibrate);
        assert!(take_commands(&mut world).is_empty());
        assert!(!state(&world, vibrate).playing);
        assert!(live_instances(&world).is_empty());
    }

    #[test]
    fn the_thirty_third_voice_is_dropped_until_one_finishes() {
        let mut world = test_world();
        let flap = register(&mut world, def("Flap", false, true));
        for _ in 0..INSTANCE_SLOT_COUNT {
            play(&mut world, flap);
        }
        let voices = played_voices(&take_commands(&mut world));
        assert_eq!(voices.len(), INSTANCE_SLOT_COUNT);

        play(&mut world, flap);
        assert!(take_commands(&mut world).is_empty());
        assert_eq!(state(&world, flap).sound_instance, None);
        assert!(state(&world, flap).playing);
        assert!(!is_playing(&world, flap));

        finish(&mut world, voices[5].0);
        play(&mut world, flap);
        assert_eq!(played_voices(&take_commands(&mut world)).len(), 1);
        assert_eq!(live_instances(&world).len(), INSTANCE_SLOT_COUNT);
    }

    #[test]
    fn finished_one_shots_stop_reporting_playing_and_are_collected() {
        let mut world = test_world();
        let splash = register(&mut world, def("Splash", false, false));
        let crunch = register(&mut world, def("Crunch", false, false));
        play(&mut world, splash);
        let voice = state(&world, splash).sound_instance.expect("voice");
        finish(&mut world, voice);

        assert!(state(&world, splash).playing);
        assert!(!is_playing(&world, splash));
        assert_eq!(live_instances(&world).len(), 1);

        play(&mut world, crunch);
        assert_eq!(state(&world, splash).sound_instance, None);
        assert_eq!(live_instances(&world).len(), 1);

        // `stop` of a finished sound still clears `playing`.
        take_commands(&mut world);
        stop(&mut world, splash);
        assert!(take_commands(&mut world).is_empty());
        assert!(!state(&world, splash).playing);
    }

    #[test]
    fn stale_finish_reports_are_ignored() {
        let mut world = test_world();
        let splash = register(&mut world, def("Splash", false, false));
        play(&mut world, splash);
        play(&mut world, splash);
        let voices = played_voices(&take_commands(&mut world));
        world.resource_mut::<Sounds>().voice_finished(voices[0].0, voices[0].1);
        assert!(is_playing(&world, splash));

        world.resource_mut::<Sounds>().voice_finished(voices[1].0, voices[1].1);
        assert!(!is_playing(&world, splash));
    }

    #[test]
    fn preloaded_instances_are_pinned_until_unloaded() {
        let mut world = test_world();
        let intro = register(&mut world, def("DanceIntro", false, false));
        let other = register(&mut world, def("Other", false, false));
        preload(&mut world, intro);
        let voice = state(&world, intro).sound_instance.expect("preloaded voice");
        assert_eq!(live_instances(&world)[0].state, SourceState::Initial);

        play(&mut world, other);
        assert_eq!(state(&world, intro).sound_instance, Some(voice));

        play(&mut world, intro);
        stop(&mut world, intro);
        assert_eq!(state(&world, intro).sound_instance, Some(voice));
        let commands = take_commands(&mut world);
        assert!(commands.contains(&SoundCommand::Stop { voice }));

        unload(&mut world, intro);
        assert_eq!(state(&world, intro).sound_instance, Some(voice));
        play(&mut world, other);
        assert_eq!(state(&world, intro).sound_instance, None);
    }

    #[test]
    fn preloading_at_init_creates_a_pinned_instance() {
        let mut world = test_world();
        let looping_def = def("Loop", true, false);
        let sound = init_sound(&mut world, looping_def, true);
        let instances = live_instances(&world);
        assert_eq!(instances.len(), 1);
        assert!(instances[0].preload);
        assert!(instances[0].looping);
        assert_eq!(state(&world, sound).sound_instance, Some(instances[0].voice));
    }

    #[test]
    fn aliases_share_one_sound_object() {
        let mut world = test_world();
        let sand = register(&mut world, def("SandBackgroundLoop", true, false));
        set_sound_alias(&mut world, "BackgroundLoop", Some(sand));
        set_sound_alias(&mut world, "BackgroundLoop", None);

        play_sound(&mut world, "BackgroundLoop");
        assert!(is_sound_playing(&world, "SandBackgroundLoop"));
        play_sound(&mut world, "SandBackgroundLoop");
        assert_eq!(played_voices(&take_commands(&mut world)).len(), 1);

        stop_all_sounds(&mut world);
        let stops = take_commands(&mut world);
        assert_eq!(stops.len(), 1);
        assert!(!is_sound_playing(&world, "BackgroundLoop"));
    }

    #[test]
    fn unknown_ids_are_silent() {
        let mut world = test_world();
        register(&mut world, def("Campfire", false, false));
        play_sound(&mut world, "campfire");
        stop_sound(&mut world, "campfire");
        preload_sound(&mut world, "campfire");
        unload_sound(&mut world, "campfire");
        assert!(!is_sound_playing(&world, "campfire"));
        assert!(take_commands(&mut world).is_empty());
        assert!(live_instances(&world).is_empty());
    }

    #[test]
    fn every_shipped_sound_resolves_to_an_embedded_sample() {
        let table = pg_assets::data::sounds::load().expect("sounds.json matches its schema");
        let mut world = test_world();
        for entry in &table.sounds {
            let file_name = entry.file_name.clone().unwrap_or_default();
            let def = SoundDef {
                id: entry.id.clone(),
                file_name,
                looping: entry.looping,
                multiple_instances: entry.multiple_instances,
                stream: entry.stream,
            };
            let sound = add_sound_def(&mut world, def);

            let sounds = world.resource::<Sounds>();
            let registered = sounds.def(sound);
            assert_eq!(registered.vibrate(), entry.vibrate, "{}", entry.id);
            if !entry.vibrate {
                let path = registered.resource_path();
                assert_eq!(Some(&path), entry.asset.as_ref());
                assert!(pg_assets::file(&path).is_some(), "{path} is embedded");
            }
        }

        play_sound(&mut world, VIBRATE_SOUND_ID);
        play_sound(&mut world, "SandBackgroundLoop");
        let commands = take_commands(&mut world);
        assert_eq!(commands[0], SoundCommand::Vibrate);
        let expected_loop = SoundCommand::Play {
            voice: 0,
            serial: 0,
            asset: "sounds/burrowersLoop03.flac".to_owned(),
            looping: true,
        };
        assert_eq!(commands[1], expected_loop);
    }

    #[test]
    fn click_fix_zeroes_only_the_last_three_minimum_values() {
        let mut samples = [i16::MIN, 1, i16::MIN, i16::MIN, 7];
        apply_click_fix(&mut samples);
        assert_eq!(samples, [i16::MIN, 1, 0, 0, 7]);

        let mut short = [i16::MIN, i16::MIN];
        apply_click_fix(&mut short);
        assert_eq!(short, [i16::MIN, i16::MIN]);
    }
}
