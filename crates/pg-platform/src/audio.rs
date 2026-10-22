//! The engine's sound commands played with `bevy_audio` (`docs/re/audio-prefs-text.md` §1 and
//! the bridge contract of `pg_engine::sound`).
//!
//! Each engine voice (one OpenAL source) is one entity with an [`AudioPlayer`]. `Play`
//! replaces whatever the voice was playing and starts its sample from the first frame, looping
//! or once, at gain 1 (the original has no volume, pitch or pan; mono samples play at full
//! level on both channels). `Stop` stops it. After a frame's commands, voices that reached the
//! end of their sample are reported with `Sounds::voice_finished` (the moment the original's
//! source turned `AL_STOPPED` by itself) and their playback offsets with `voice_offset`.
//! Samples are decoded on first use and kept (the original loads a file when a voice is
//! created; keeping them saves decoding again).
//!
//! While the game is off screen (the app suspended or its window hidden) every voice is paused,
//! as the suspended original falls silent: Android keeps an app's audio running in the
//! background, and `bevy_audio` has no lifecycle handling of its own. The voices go on where
//! they stopped when the game shows again; a paused voice is not finished.
//!
//! Without an audio output (no sound device) Bevy never starts a sound. The voices then play
//! silently on a stand-in clock of real time: offsets advance and one-shots end after their
//! sample's length, so `isPlaying` and the logic that waits for sounds behave as with sound.

use bevy::audio::{AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use pg_engine::platform::PlatformRequests;
use pg_engine::sound::{SoundCommand, Sounds};

use crate::lifecycle::LifecycleTracker;
use crate::wav;

/// A decoded sample.
#[derive(Clone, Debug)]
struct Sample {
    source: Handle<AudioSource>,

    /// Its length in seconds.
    duration: f32,
}

/// Every sample decoded so far, by asset path (`None`: missing or not a valid FLAC file).
#[derive(Resource, Default)]
pub struct SoundBank {
    samples: HashMap<String, Option<Sample>>,
}

impl SoundBank {
    /// The playable sample of `asset`, decoding it on first use.
    fn sample(&mut self, asset: &str, audio_sources: &mut Assets<AudioSource>) -> Option<Sample> {
        if let Some(cached) = self.samples.get(asset) {
            return cached.clone();
        }

        let sample = match decode_sample(asset) {
            Some(playable) => {
                let source = AudioSource {
                    bytes: playable.wav.into(),
                };
                let sample = Sample {
                    source: audio_sources.add(source),
                    duration: playable.duration,
                };
                Some(sample)
            }
            None => None,
        };
        self.samples.insert(asset.to_owned(), sample.clone());
        sample
    }
}

/// The click-fixed WAV of an embedded FLAC sample, with its length.
fn decode_sample(asset: &str) -> Option<wav::PlayableSample> {
    let Some(flac) = pg_assets::file(asset) else {
        warn!("sound {asset} is not embedded");
        return None;
    };
    match wav::playable_sample(flac) {
        Ok(sample) => Some(sample),
        Err(error) => {
            warn!("sound {asset} could not be decoded: {error}");
            None
        }
    }
}

/// How far a voice got this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Progress {
    /// Playing (or paused) this many seconds into its sample.
    At(f32),

    /// A one-shot reached the end of its sample.
    Finished,
}

/// The entity playing one engine voice.
#[derive(Clone, Copy, Debug)]
struct Voice {
    entity: Entity,
    serial: u64,
    looping: bool,

    /// The sample's length in seconds.
    duration: f32,

    /// Spawned this frame: Bevy starts it after the frame's systems.
    just_started: bool,

    /// Seconds into the sample on the stand-in clock, while there is no audio output.
    silent_position: f32,
}

impl Voice {
    /// Advances the stand-in clock of a voice that has no audio output by `elapsed` seconds.
    fn advance_silently(&mut self, elapsed: f32) -> Progress {
        self.silent_position += elapsed.max(0.0);
        if !self.looping {
            if self.silent_position >= self.duration {
                return Progress::Finished;
            }
            return Progress::At(self.silent_position);
        }

        if self.duration > 0.0 {
            self.silent_position %= self.duration;
        } else {
            self.silent_position = 0.0;
        }
        Progress::At(self.silent_position)
    }
}

/// Keeps a started voice paused or playing with the game's visibility, and tells how far it
/// got.
fn sink_progress(sink: &AudioSink, looping: bool, audible: bool) -> Progress {
    if sink.is_paused() == audible {
        if audible {
            sink.play();
        } else {
            sink.pause();
        }
    }

    if !looping && sink.empty() {
        return Progress::Finished;
    }
    let position = sink.position();
    Progress::At(position.as_secs_f32())
}

/// The engine voices that are playing.
#[derive(Resource, Default)]
pub struct Voices {
    playing: HashMap<u64, Voice>,
}

impl Voices {
    /// `alSourcePlay` of `sample` on `voice`; off screen the voice waits, paused, for the game
    /// to show again.
    fn start(
        &mut self,
        commands: &mut Commands,
        voice: u64,
        serial: u64,
        sample: &Sample,
        looping: bool,
        audible: bool,
    ) {
        let mut settings = if looping { PlaybackSettings::LOOP } else { PlaybackSettings::ONCE };
        settings.paused = !audible;

        let player = AudioPlayer::new(sample.source.clone());
        let entity = commands.spawn((player, settings)).id();
        let playing = Voice {
            entity,
            serial,
            looping,
            duration: sample.duration,
            just_started: true,
            silent_position: 0.0,
        };
        self.playing.insert(voice, playing);
    }

    /// `alSourceStop`: the voice falls silent.
    fn stop(&mut self, commands: &mut Commands, voice: u64) {
        if let Some(playing) = self.playing.remove(&voice) {
            commands.entity(playing.entity).despawn();
        }
    }
}

/// Reports how far every voice got (`elapsed`: the real time this frame while audible), and
/// removes the one-shots that ended.
fn report_progress(
    commands: &mut Commands,
    sounds: &mut Sounds,
    voices: &mut Voices,
    sinks: &Query<&AudioSink>,
    audible: bool,
    elapsed: f32,
) {
    let mut ended = Vec::new();
    for (voice, playing) in voices.playing.iter_mut() {
        if playing.just_started {
            playing.just_started = false;
            continue;
        }

        // A voice still without a sink a frame after it started has no audio output.
        let progress = match sinks.get(playing.entity) {
            Ok(sink) => sink_progress(sink, playing.looping, audible),
            Err(_) => playing.advance_silently(elapsed),
        };
        match progress {
            Progress::Finished => ended.push(*voice),
            Progress::At(offset) => sounds.voice_offset(*voice, playing.serial, offset),
        }
    }

    for voice in ended {
        let Some(playing) = voices.playing.remove(&voice) else {
            continue;
        };
        debug!("voice {voice} finished its sample (play {})", playing.serial);
        sounds.voice_finished(voice, playing.serial);
        commands.entity(playing.entity).despawn();
    }
}

/// Plays the frame's sound commands, keeps the voices silent while the game is off screen,
/// then reports the voices that ended and the offsets of the others.
#[allow(clippy::too_many_arguments)]
pub fn play_sound_commands(
    mut commands: Commands,
    sounds: Option<ResMut<Sounds>>,
    mut bank: ResMut<SoundBank>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
    mut voices: ResMut<Voices>,
    mut requests: ResMut<PlatformRequests>,
    tracker: Res<LifecycleTracker>,
    time: Res<Time<Real>>,
    sinks: Query<&AudioSink>,
) {
    let Some(mut sounds) = sounds else {
        return;
    };
    let audible = tracker.is_visible();

    // A source whose buffer failed to load stops as soon as it is played.
    let mut silent = Vec::new();
    let pending = std::mem::take(&mut sounds.commands);
    for command in pending {
        match command {
            SoundCommand::Play {
                voice,
                serial,
                asset,
                looping,
            } => {
                voices.stop(&mut commands, voice);
                let Some(sample) = bank.sample(&asset, &mut audio_sources) else {
                    silent.push((voice, serial));
                    continue;
                };
                voices.start(&mut commands, voice, serial, &sample, looping, audible);
            }
            SoundCommand::Stop { voice } => voices.stop(&mut commands, voice),
            SoundCommand::Vibrate => requests.vibrate = true,
            SoundCommand::StreamStart { asset } => {
                warn_once!("the stream player is dead code in 1.48.2; {asset} is not streamed");
            }
            SoundCommand::StreamStop => {}
        }
    }

    for (voice, serial) in silent {
        sounds.voice_finished(voice, serial);
    }

    let elapsed = if audible { time.delta_secs() } else { 0.0 };
    report_progress(&mut commands, &mut sounds, &mut voices, &sinks, audible, elapsed);
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;
    use std::time::Duration;

    use pg_engine::sound::{SoundDef, SoundId};

    use super::*;

    const BURP_ASSET: &str = "sounds/burp1.flac";

    /// The bridge's resources, with the real-time clock started.
    fn audio_world() -> World {
        let mut world = World::new();
        world.init_resource::<Sounds>();
        world.init_resource::<SoundBank>();
        world.init_resource::<Assets<AudioSource>>();
        world.init_resource::<Voices>();
        world.init_resource::<PlatformRequests>();
        world.init_resource::<LifecycleTracker>();
        world.insert_resource(Time::<Real>::default());
        advance_time(&mut world, 0.0);
        world
    }

    fn advance_time(world: &mut World, seconds: f32) {
        let elapsed = Duration::from_secs_f32(seconds);
        world.resource_mut::<Time<Real>>().update_with_duration(elapsed);
    }

    /// `[burp play]`: a one-shot whose `Play` the bridge sees on its next run.
    fn play_burp(world: &mut World) -> SoundId {
        let def = SoundDef {
            id: "Burp".to_owned(),
            file_name: "burp1".to_owned(),
            ..default()
        };
        let mut sounds = world.resource_mut::<Sounds>();
        let burp = sounds.init_sound(def, false);
        sounds.play(burp);
        burp
    }

    fn set_in_background(world: &mut World, background: bool) {
        let mut transitions = Vec::new();
        let mut tracker = world.resource_mut::<LifecycleTracker>();
        tracker.set_suspended(background, &mut transitions);
    }

    fn run_bridge(world: &mut World) {
        world.run_system_cached(play_sound_commands).expect("the bridge runs");
    }

    fn is_playing(world: &World, sound: SoundId) -> bool {
        world.resource::<Sounds>().is_playing(sound)
    }

    /// The entity of the only voice.
    fn voice_entity(world: &World) -> Entity {
        let voices = world.resource::<Voices>();
        let playing = voices.playing.values().next().expect("a voice is playing");
        playing.entity
    }

    /// The sink Bevy's audio output attaches to a started voice, here without a device:
    /// `played_out` = the sample reached its end.
    fn output_sink(played_out: bool) -> AudioSink {
        let (player, _device_side) = rodio::Player::new();
        if !played_out {
            let channels = NonZero::new(1).expect("one channel");
            let sample_rate = NonZero::new(44_100).expect("a sample rate");
            let endless = rodio::source::Zero::new(channels, sample_rate);
            player.append(endless);
        }
        AudioSink::new(player)
    }

    fn sink_is_paused(world: &World, entity: Entity) -> bool {
        let sink = world.get::<AudioSink>(entity).expect("the voice has a sink");
        sink.is_paused()
    }

    #[test]
    fn voices_fall_silent_in_the_background_and_go_on_in_the_foreground() {
        let mut world = audio_world();
        let burp = play_burp(&mut world);

        // Started in the background: the voice waits, paused.
        set_in_background(&mut world, true);
        run_bridge(&mut world);
        let entity = voice_entity(&world);
        let settings = world.get::<PlaybackSettings>(entity).expect("a started voice");
        assert!(settings.paused);

        // Bevy's audio output attaches the sink paused, as the settings ask.
        let sink = output_sink(false);
        sink.pause();
        world.entity_mut(entity).insert(sink);
        run_bridge(&mut world);
        assert!(sink_is_paused(&world, entity));
        assert!(is_playing(&world, burp), "a paused voice has not finished");

        set_in_background(&mut world, false);
        run_bridge(&mut world);
        assert!(!sink_is_paused(&world, entity));

        set_in_background(&mut world, true);
        run_bridge(&mut world);
        assert!(sink_is_paused(&world, entity));
        assert!(is_playing(&world, burp));
    }

    #[test]
    fn one_shots_that_played_out_are_reported_finished() {
        let mut world = audio_world();
        let burp = play_burp(&mut world);
        run_bridge(&mut world);
        let entity = voice_entity(&world);

        world.entity_mut(entity).insert(output_sink(false));
        run_bridge(&mut world);
        assert!(is_playing(&world, burp));

        world.entity_mut(entity).insert(output_sink(true));
        run_bridge(&mut world);
        assert!(!is_playing(&world, burp));
        assert!(world.resource::<Voices>().playing.is_empty());
        assert!(world.get_entity(entity).is_err());
    }

    #[test]
    fn without_an_output_voices_play_on_a_clock_that_stops_in_the_background() {
        let mut world = audio_world();
        let burp = play_burp(&mut world);
        run_bridge(&mut world);
        let bank = world.resource::<SoundBank>();
        let cached = bank.samples.get(BURP_ASSET).cloned().flatten();
        let duration = cached.expect("burp1 decodes").duration;

        set_in_background(&mut world, true);
        advance_time(&mut world, duration + 1.0);
        run_bridge(&mut world);
        assert!(is_playing(&world, burp));

        set_in_background(&mut world, false);
        advance_time(&mut world, duration * 0.5);
        run_bridge(&mut world);
        assert!(is_playing(&world, burp));
        let offset = world.resource::<Sounds>().play_time(burp);
        assert!((offset - duration * 0.5).abs() < 1e-3, "{offset} of {duration}");

        advance_time(&mut world, duration * 0.6);
        run_bridge(&mut world);
        assert!(!is_playing(&world, burp));
    }

    fn voice(looping: bool, duration: f32) -> Voice {
        Voice {
            entity: Entity::PLACEHOLDER,
            serial: 7,
            looping,
            duration,
            just_started: false,
            silent_position: 0.0,
        }
    }

    #[test]
    fn without_an_output_one_shots_end_after_their_length() {
        let mut one_shot = voice(false, 0.05);
        assert_eq!(one_shot.advance_silently(0.02), Progress::At(0.02));
        assert_eq!(one_shot.advance_silently(0.0), Progress::At(0.02));
        assert_eq!(one_shot.advance_silently(0.04), Progress::Finished);
    }

    #[test]
    fn without_an_output_loops_wrap_around() {
        let mut looping = voice(true, 2.0);
        let mut offset = Progress::At(0.0);
        for _ in 0..5 {
            offset = looping.advance_silently(0.5);
        }
        assert_eq!(offset, Progress::At(0.5));
        assert!(looping.silent_position < 2.0);

        let mut empty = voice(true, 0.0);
        assert_eq!(empty.advance_silently(1.0), Progress::At(0.0));
    }

    #[test]
    fn decoded_samples_are_kept() {
        let mut bank = SoundBank::default();
        let mut audio_sources = Assets::<AudioSource>::default();
        let first = bank.sample("sounds/burp1.flac", &mut audio_sources).expect("embedded sample");
        let again = bank.sample("sounds/burp1.flac", &mut audio_sources).expect("embedded sample");
        assert_eq!(first.source, again.source);
        assert!(first.duration > 0.0);
        assert_eq!(audio_sources.len(), 1);

        assert!(bank.sample("sounds/missing.flac", &mut audio_sources).is_none());
        assert_eq!(audio_sources.len(), 1);
    }
}
