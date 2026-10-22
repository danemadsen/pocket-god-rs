# Audio, preferences / save data, fonts / text, gradients and other engine services

Spec for the port of Pocket God 1.48.2. All addresses are in the arm64 slice
`tmp/re/PocketGod.arm64`; `Class::selector @ address` names follow `docs/re/README.md`.
Statements marked *(asm)* were checked against the disassembly, not only the decompiler.

Deliverables described here:

| File | Producer | Contents |
|---|---|---|
| `assets/sounds/<name>.flac` (355 files) | `tools/re/convert_sounds.py` | every `sound/*.caf`, losslessly re-encoded, bit-exact |
| `assets/data/sounds.json` | `tools/re/extract_engine_tables.py` | 359 sound defs + audio format + runtime aliases ([1.10](#110-soundsjson)) |
| `assets/data/fonts.json` | same | 4 bitmap fonts, 297 glyphs with metrics ([3.2](#32-fontsjson)) |
| `assets/data/gradients.json` | same | 15 gradient defs, parsed like the game ([4](#4-gradients)) |
| `assets/data/preferences.json` | same | the 4 preference documents, 48 defaults, 10 personas, 9 persona key formats ([2.8](#28-preferencesjson)) |

Both scripts run from the repository root with `tmp/re/venv/bin/python -I` and need no
arguments (defaults point into `tmp/`). Output is deterministic (checked by running twice).
`sounds.json`, `fonts.json` and `gradients.json` overlap with `sound-defs.json`,
`font-defs.json` and `gradient-defs.json` written by `extract_struct_xml.py`
(`docs/re/struct-xml.md` §4.1–4.3); both read the same struct-XML nodes and agree on every
value (gradient stops compared float-for-float). The files here add what the engine layer
needs: audio formats, aliases, glyph metrics, `looping` instead of the Rust keyword `loop`.

Contents: [1 Audio](#1-audio) · [2 Preferences and save data](#2-preferences-and-save-data) ·
[3 Fonts and text](#3-fonts-and-text) · [4 Gradients](#4-gradients) ·
[5 Other engine services](#5-other-engine-services) · [6 Open questions](#6-open-questions)

---

## 1. Audio

### 1.1 Overview

- One sound engine: **OpenAL** (Apple's implementation), fully in-memory buffers. No
  AVAudioPlayer, no AVAudioSession, no music streaming in use, no mixer groups.
- No per-sound or global **volume**, **pitch**, **pan** or **fade**: no `alSourcef`
  (`AL_GAIN`/`AL_PITCH`) or `alListenerf` call exists in the binary. Every source plays at
  gain 1.0, pitch 1.0, positioned at the listener. Music changes are hard cuts (stop/play).
- At most **32** simultaneous sound instances (voices); see 1.4.
- The only user options are **Mute** (all sound off) and **Vibrate** (vibration off), both
  in the `Options` preference document (2.6.1). There is no separate music/sfx switch.
- Sounds are referenced by string id through `BCLibrary`: from compiled sequences
  (`playSound`, `stopSound`, `playBackgroundLoop`, …), from `Timelines.bin` frames
  (`docs/re/timelines.md` §4.5) and from code.

### 1.2 Sound files and the FLAC assets

`Payload/PocketGod.app/sound/*.caf` — 355 files, all Core Audio Format with a `desc`,
`free`, `data` chunk layout, linear PCM, **signed 16-bit little-endian** integer
(CAF flags 2), 1 frame per packet:

| Format | Files |
|---|---|
| 22000 Hz mono | 317 |
| 22000 Hz stereo | 22 (`BattleLoop`, `TheRuns`, `apeslurp`, `blaze`, `climbladder`, `crunch`, `earthcrack`, `earthretract`, `fallin`, `gorillaooh_short`, `hitwall`, `konkeydongclang`, `konkeydongdie`, `konkeydongjump`, `konkeydongloop`, `longfall`, `successfuljump`, `swordhit`, `swordswish`, `weaponzap`, `winchretract1`, `winchturn1`) |
| 44100 Hz mono | 16 (`burp1`, `facetear`, `fart1`, `fart2`, `fart3`, `monkeyscreech`, `monkeyscreeches`, `monkeysound1`, `monkeysound3`, `monkeysqueak`, `ooh`, `shock`, `slap`, `slip`, `splashdrown`, `updatesong`) |

Note the unusual **22000 Hz** (not 22050). Total length 1123.4 s; longest `pygmiestheme`
(93.8 s), `updatesong` (48.7 s), `apocalypseloop` (30.4 s).

`tools/re/convert_sounds.py` parses each CAF itself (rejects anything but little-endian
integer 16-bit LPCM), pipes the raw samples to `flac -8 --no-padding` and then verifies every
output: the FLAC `STREAMINFO` sample rate, channel count, bits per sample and total frame
count equal the CAF values, and decoding the FLAC back gives exactly the CAF sample bytes.
All 355 pass; an independent `ffprobe` pass over `assets/sounds/` confirms codec `flac`,
16 bits, rate, channels and frame count (`duration_ts`) for every file. The audio is **not**
altered: the engine's click fix (1.4.4) must be applied by the port at load time.

The bundle has every file referenced by a def and no unreferenced file. Three files are
shared by two ids each: `earthquake` (`earthquake`, `earthquake2`), `windstorm` (`Wind`,
`WindNoLoop`), `fleshdissolveshort` (`FleshDissolveShort`, `FleshDissolve`).

### 1.3 Sound definitions

1.48.2 has no `sound.xml`; the defs are compiled struct-XML nodes (type 0) in the
NULL-terminated list at `0x10071ab98`, the first list passed to
`BCLibrary::parseStructXml: @ 0x100026674` by `PocketGodViewController::loadGame @ 0x10006a030`
(node layout: `struct-xml.md` §4.1). For each node *(asm)*:

```
s = [[BCSound alloc] initWithId:+0x08 fileName:+0x10 ext:+0x18
                       loop:+0x20 stream:+0x22 multipleInstances:+0x21 preload:NO]
[library addSound:s]
```

- 359 defs, unique ids, in this order in `sounds.json`.
- `looping` 39 defs: the island, temple, mini-game and celebration music (`*BackgroundLoop`,
  `*IslandLoop`, `*RoomLoop`, `CoralReefTheme`, `ApocalypseLoop`, `KonkeyDongLoop`,
  `MoronPestLoop`, `BattleLoop`, `LocustLoop`, `PlagueLoop`, `PurificationLoop`, `DanceLoop1`,
  `ThrillerLoop`, `PantsOnTheGround`, `UpdateSong`, `MerryFishmas`, `TwelveDays`) plus
  `JetPack`, `earthquake`, `Snore`, `Wind`, `Whirlpool`, `BansheeCryLoop`, `WinchTurn`, the
  four `PygmySing*` notes and `InsectBuzzLoop`.
  `DanceLoop2` is deliberately **not** looping (v1.45.3 `sound.xml`:
  `loop="false" loopNeedsToStayFalse="1"`); `DanceDanceExecutionScreenLogic::onGameFrame:`
  restarts it by hand (1.7.5).
- `multiple_instances` 18 defs: `Splash3`, `Fireball`, `Sparkle`, `Clang`,
  `KonkeyDongSuccessfulJump`, `KonkeyDongClimbLadder`, `JellyfishBubble1`,
  `TempleTone1`–`TempleTone9`, `XenuRoar`, `Flap`. None of them loops.
- `stream` is false for every def (the streaming path, 1.5, is dead code).
- `Vibrate` has empty file name and extension: `initWithId:…` sets `mbVibrate` for the id
  `@"Vibrate"` and never resolves a resource path. Playing it vibrates the device (1.4.2).
- `preload` is always NO here; `BCSound -preload` can set it later (1.4.2).

Cross-check with v1.45.3 `sound.xml` (332 `<sound>` elements, attributes `id`, `filename`,
`ext`, `loop`, `multipleInstances`): the 323 ids present in both versions have identical
file names, loop and multiple-instance flags. 1.48.2 adds 36 ids (`ApocalypseLoop`,
`ClosetRoomLoop`, `PlagueLoop`, `PurificationLoop`, `XenuRoomLoop`, `XenuRoar`, the `Tom*`
and `Kaytee*` voice lines, `Gibberish1`–`4`, …) and drops 9 (`BabySound2`, `CloudDissolve`,
`DuhWinning`, `FastHitByMeteor`, `LowSlap`, `MonkeySound2`, `YouCantProcessMe`,
`YouPygmiesAreTrolls`, `ZombieMoan`).

### 1.4 `BCSound` and the OpenAL engine

#### 1.4.1 Data

`BCSound : NSObject <BCSoundInstanceListener>` (the protocol, `<NSObject>` plus one required
method `onSoundInstanceComplete:(BCSoundInstance *)`, is how an instance reports its deletion
to its owner). Ivars: `mId`, `mFileName` (`@"sound/<file>"`), `mExt`, `mResourcePath`
(`[[NSBundle mainBundle] pathForResource:mFileName ofType:mExt]`), `mbLoop`, `mbStream`,
`mbVibrate`, `mbPlaying`, `mbMultipleInstances`, `mbLoaded` (never used), `mbPreload`,
`mpSoundInstance` (the *current* instance, or NULL).

Globals:

| Address | Meaning |
|---|---|
| `0x1007cd7f0` (byte) | mute flag, `+[BCSound mute:] @ 0x10000d628` |
| `0x1007cd7f1` (byte) | vibrate-mute flag, `+[BCSound muteVibrate:] @ 0x10000d638` |
| `0x1007e3018` (int) | number of live `BCSound` objects |
| `0x1007e301c` (byte) | "manager created" flag |
| `0x1007e3020` (ptr) | the C++ sound manager (below) |
| `0x1007cd800` (ptr) | cached `alcMacOSXMixerOutputRate` proc |

Sound manager (C++, 0x120 bytes, vtable `0x10037d0e0`): `+0x08 ALCcontext*`,
`+0x10 ALCdevice*`, `+0x18` unused, `+0x20` **32 instance slots** (`BCSoundInstance*`).
Created by the first `BCSound` init (`FUN_10000de78`): `alcOpenDevice(NULL)`;
`alcMacOSXMixerOutputRate(44100.0)` (`FUN_10000de18`); `alcCreateContext`;
`alcMakeContextCurrent`; `alListener3f(AL_POSITION, 0, 0, 0)`. Errors are printed and
ignored. It is destroyed only when the live `BCSound` count drops to 0 (never in practice):
every slot deleted, context destroyed, device closed.

`BCSoundInstance` (C++, `operator new(0x230)`, vtable `0x10037d100`; the ObjC type string is
`{BCSoundInstance=^^?@[512c]III^vBBBB}`):

| Off | Type | Field |
|---|---|---|
| 0x000 | vtable | |
| 0x008 | `BCSound*` | owner (`BCSoundInstanceListener`, receives `onSoundInstanceComplete:`) |
| 0x010 | `char[512]` | resource path (strcpy) |
| 0x210 | `ALuint` | source |
| 0x214 | `ALuint` | buffer |
| 0x218 | `UInt32` | PCM byte count |
| 0x220 | `void*` | PCM data (malloc; kept until unload) |
| 0x228 | bool | file loaded into the buffer |
| 0x229 | bool | loop |
| 0x22a | bool | preload (pinned: never garbage-collected) |
| 0x22b | bool | buffer attached to the source |

#### 1.4.2 `BCSound` methods

- `initWithId:fileName:ext:loop:stream:multipleInstances:preload: @ 0x10000d648`: stores the
  fields (`mbVibrate = mbPlaying = NO`), increments the live count, creates the manager on the
  first call, resolves `mResourcePath` unless the id is `Vibrate` (then `mbVibrate = YES`),
  stores `mbPreload` and, if preloading, creates the instance immediately (`FUN_10000d8c4`).
- `play @ 0x10000db20` *(asm)*:

```
if mbLoop && mbPlaying: return                         // a running loop is never restarted
if mbVibrate:
    if !vibrateMuted: AudioServicesPlaySystemSound(kSystemSoundID_Vibrate = 0xFFF)
    return                                             // mbPlaying unchanged
if mbStream:
    if !muted: mbPlaying = YES; streamStart(path)      // dead path, 1.5
    return
if muted: return                                       // mbPlaying unchanged
mbPlaying = YES
if manager == NULL: return
if mpSoundInstance == NULL || mbMultipleInstances:
    mpSoundInstance = createInstance(path, loop: mbLoop, preload: NO, owner: self)
    if mpSoundInstance == NULL: return                 // all 32 slots busy: silently dropped
instancePlay(mpSoundInstance)                          // FUN_10000e0e0
collectGarbage()                                       // FUN_10000e060
```

  Consequences: a non-multiple sound that is played again while still sounding **restarts
  from the beginning** on the same source (`alSourcePlay` on a playing source rewinds it).
  A multiple-instance sound gets a new voice each time; earlier voices keep playing and
  `mpSoundInstance` points at the newest. Every new instance re-reads the file.
- `stop @ 0x10000dca8`: only if `mbPlaying`: `mbPlaying = NO`; stream → `streamStop()`;
  else if there is a current instance: `alSourceStop`, `collectGarbage()`, and unless
  `mbPreload`, `mpSoundInstance = NULL`. Older instances of a multiple-instance sound are not
  stopped. `stop` on a sound whose `mbPlaying` is NO does nothing (e.g. one played while muted).
- `isPlaying @ 0x10000dd60`: `mbPlaying && mpSoundInstance && AL_SOURCE_STATE == AL_PLAYING`.
  A finished one-shot therefore reports NO even though `mbPlaying` stays YES.
- `playTime @ 0x10000ddc0`: `AL_SEC_OFFSET` of the current instance (float seconds), 0 when
  not playing. Unused by the game.
- `preload @ 0x10000da50`: if not yet preloaded: `mbPreload = YES`; no instance → create one
  pinned (`preload: YES`, file loaded now); else pin the current instance.
- `unload @ 0x10000daf0`: if preloaded: `mbPreload = NO` and unpin the current instance (it is
  collected by the next garbage pass once it is not playing). It does not stop the sound.
- `onSoundInstanceComplete: @ 0x10000da24` (instance listener): if the completed instance is
  `mpSoundInstance` and the sound is not preloaded, `mpSoundInstance = NULL`.
- `dealloc @ 0x10000d980`: `stop`, decrement the live count (destroy the manager at 0).
- `test @ 0x10000dd38` (logs source state), `id @ 0x10000d970`.

#### 1.4.3 Instances, voices and garbage collection

- `createInstance` = `FUN_10000d8c4(manager, path, loop, preload, owner)`: runs
  `collectGarbage()` first, then takes the **first empty slot of the 32**; returns NULL if
  none. The instance constructor (`FUN_10000e290`) zeroes the fields, copies the path,
  `alGenSources(1)` and immediately **loads** the file (`FUN_10000e300` →
  `FUN_10000e638`), then `alSourcei(AL_BUFFER)` and `alSource3f(AL_POSITION, 0, 0, 0)`.
- `instancePlay` (`FUN_10000e0e0`): ensure loaded/attached, `alSourcei(AL_LOOPING, loop)`,
  `alSourcePlay`.
- `collectGarbage` (`FUN_10000e060`): every slot whose source state is not `AL_PLAYING` and
  that is not pinned is deleted: `alDeleteSources`, `alDeleteBuffers`, `free(data)`, then the
  owner gets `onSoundInstanceComplete:`. It runs before every instance creation and after every
  `play` and `stop`, so finished voices are reclaimed lazily (no completion callback).
- Pinned (preloaded) instances occupy a slot until `unload` + a later garbage pass.

#### 1.4.4 Loading a file — `FUN_10000e638` *(asm-checked branch layout)*

```
AudioFileOpenURL(path, read); get kAudioFilePropertyDataFormat (ASBD, 40 bytes)
and kAudioFilePropertyAudioDataByteCount
data = malloc(byteCount); bzero; AudioFileReadBytes(file, NO, 0, &byteCount, data)
if !(formatID == 'lpcm' && !(flags & kAudioFormatFlagIsBigEndian)) && bitsPerChannel > 8:
    puts("Invalid File Format:"); return 4             // (leaks data; never happens)
alGenBuffers(1)
// click fix: the last three int16 values of the interleaved data
n = (byteCount - 6) >> 1
for i in n, n+1, n+2: if sample[i] == -32768: sample[i] = 0
format = AL_FORMAT_MONO16 (0x1101) / STEREO16 (0x1103) for 16-bit,
         MONO8 (0x1100) / STEREO8 (0x1102) for 8-bit, only if formatID == 'lpcm'
         and channels is 1 or 2; otherwise 4 (an invalid enum: alBufferData fails)
alBufferData(buffer, format, data, byteCount, (int)sampleRate)
AudioFileClose; loaded = YES
```

The click fix changes 10 files at load time (their last samples are −32768):
`bubble3`, `fall_whoa`, `fall_whoa2`, `hitground`, `hitground2`, `hitground3`, `newoh`,
`newoh2`, `newoh3`, `tip_whoa`. The port must apply the same rule to the decoded samples
(the FLAC files keep the original values).

Playback rate is the file's own rate; OpenAL resamples to the 44100 Hz mixer. Buffers keep the
file's channel count (OpenAL plays mono buffers through the 3D path at the listener position
and stereo buffers unspatialised; see [6](#6-open-questions) for the level question).

### 1.5 Streaming player (dead code)

`mbStream` is never set in 1.48.2, so this path never runs; documented for completeness.
`streamStart` = `FUN_100068c20` lazily creates a 0x10888-byte player (`FUN_100068cdc`): a
RemoteIO AudioUnit (`'auou'/'rioc'/'appl'`), output enabled, stream format 44100 Hz LPCM
signed-packed 16-bit **mono**, render callback `FUN_100068f40` → `FUN_100068fd0`, which
copies from a ring of 8 buffers of 4096 bytes (12-byte headers, stride 0x100c) and zero-fills
when empty. `streamStart` copies the path to `+0x100ac` and sets the request flag `+0x104ac`;
`streamStop` (`FUN_100068cb8`) sets `+0x104ad`. The thread that would fill the ring was not
found (see open questions). The port does not need this path.

### 1.6 Library registry, aliases and sequence handlers

`BCLibrary.mSoundDictionary` (NSMutableDictionary, id → `BCSound`):

- `addSound: @ 0x100023d6c`: `dict[[sound id]] = sound` (last wins).
- `getSoundById: @ 0x100023d50`: `dict[id]` or nil. Lookups are **case-sensitive**;
  messages to nil do nothing, so unknown ids are silent no-ops everywhere.
- `setSoundAlias:sound: @ 0x100023dc0`: `if sound != nil: dict[alias] = sound` — an alias is a
  second key for the **same object** (shared state: playing flag, instance).
- `stopAllSounds @ 0x100024b4c`: `stop` on the object of every key (aliases included, which
  is harmless).

Sequence handlers (all return 2 = done; `sequences.md` lists their uses):

| Handler | Address | Behaviour |
|---|---|---|
| `playSound:::` (soundId) | `0x100024ad4` | `[getSoundById(p0) play]` |
| `stopSound:::` (soundId) | `0x100024b10` | `[getSoundById(p0) stop]` |
| `setSoundAlias:::` (soundId, aliasId) | `0x100024a80` | `setSoundAlias:p1 sound:getSoundById(p0)` (only used by the unused `Garbage` sequence: `BackgroundLoop → SandBackgroundLoop`) |
| `stopAllSounds:::` | `0x100024c7c` | `stopAllSounds` (unused) |
| `mute:::` / `unmute:::` | `0x100024ca0` / `0x100024cd0` | `+[BCSound mute:YES/NO]` (unused) |

### 1.7 Music, loops and the sound options

#### 1.7.1 `BackgroundLoop` and `ApocalypseLoop`

`PocketGodViewController::playBackgroundLoop:(id) @ 0x100070ee8` *(asm)*:

```
loopId = mbApocalypseActive ? "ApocalypseLoop" : "BackgroundLoop"
if id != nil:
    if !mbApocalypseActive: [getSound("BackgroundLoop") stop]
    setSoundAlias("BackgroundLoop", getSound(id))       // ignored when id is unknown
if ![getSound(loopId) isPlaying]: [getSound(loopId) play]
```

`BackgroundLoop` does not exist until the first call with an id. The alias targets come from
the island start sequences (`playBackgroundLoop` sequence item, handler `0x10007104c`):

| Sequence | Loop | | Sequence | Loop |
|---|---|---|---|---|
| `StartSandIsland` | `SandBackgroundLoop` | | `StartUnderwaterCoralReef` | `CoralReefTheme` |
| `StartRockIsland` | `RockBackgroundLoop` | | `StartApocalypseIsland` | `ApocolypseIslandLoop` |
| `StartUnderwater` | `UnderwaterBackgroundLoop` | | `StartTimeTemple` | `TimeRoomLoop` |
| `StartGraveyardIsland` | `GraveyardBackgroundLoop` | | `StartLocustTemple` | `LocustLoop` |
| `StartIceIsland` | `IceBackgroundLoop` | | `StartDanceTemple` | `DanceLoop1` |
| `StartApeIsland` | `ApeIslandLoop` | | `StartPlagueTemple` | `PlagueLoop` |
| `StartMisfitIsland` | `MisfitIslandLoop` | | `StartPurificationTemple` | `PurificationLoop` |
| `StartGodIsland`, `StartFailBoat` | `GodIslandLoop` | | `StartClosetTemple` | `ClosetRoomLoop` |
| | | | `StartXenuTemple` | `XenuRoomLoop` |

With no id (`StopTheRuns`, `StopNoPee`, `StopKonkeyDong`, `StopMoronPests`,
`StopBattleOfTheGods`, `DiscoBallSmashPygmy`, `DanceDanceExecutionRemovePygmy`, unmute,
`stopCelebrationMusic`, `loadPreferences:::`) it just resumes the current loop if silent.
`DiscoBallLogic::playBackgroundLoop::: @ 0x10011bbcc` and
`DanceDanceExecutionLogic::playBackgroundLoop::: @ 0x10017ceec` forward
`playBackgroundLoop:nil` to their listener (the view controller) — they ignore their
parameter. `stopBackgroundLoop` (`PocketGodViewController(Game) @ 0x100168eec`, also the
handlers `0x100071078`, `0x10011bb9c`, `0x10017cebc`) stops `ApocalypseLoop` during an
apocalypse, else `BackgroundLoop`.

Apocalypse: `startApocalypse @ 0x100168898` stops `BackgroundLoop`, plays `ApocalypseLoop`;
`endApocalypseComplete @ 0x100168b44` / `endApocalypseCompleteNow @ 0x100168c0c` stop
`ApocalypseLoop`, play `BackgroundLoop`.

#### 1.7.2 Celebration songs

`PocketGodViewController(Game)::startCelebration @ 0x100157ca4` picks the first match:
pygmy dance skin option `Dance/Pygmies/PygmyDance = PantsontheGround`, `Dance2/…/PocketGodUpdateSong`,
`Dance3/…/PygmyThemeSong`, `Fishmas/…/FishmasDance`, `Fishmas/…/12DaysofFishmas` (each only
if `canDoNormalDance`) → sets the matching flag and `stopBackgroundLoop` (the song itself is
played by the dance animation/sequence); else if a zombie is present → `ThrillerLoop` is
played. `stopCelebrationMusic @ 0x100158120` clears `mbCelebrationActive`, then takes the
first set flag in the order `PantsOnTheGround`, `UpdateSong`, `ThemeSong`, `MerryFishmas`,
`TwelveDays`, `ThrillerLoop`, clears it and stops that sound (no flag set → return), and
finally calls `playBackgroundLoop:nil` unless The Runs, Konkey Dong, Moron Pests, No Pee,
Dance Dance Execution or Battle of the Gods is active.

#### 1.7.3 Pause/resume helpers (`PocketGodViewController(Game)`)

| Helper | Address | Sound | Condition |
|---|---|---|---|
| `pauseHurricaneSounds` / `resume…` | `0x100168df4` / `0x100168e70` | `Wind` stop / play | hurricane logic exists and `windActive` |
| `pauseSleepingSounds` / `resume…` | `0x100168f64` / `0x100168fd0` | `Snore` | `mSleepingCount > 0` |
| `pauseLocustSounds` / `resume…` | `0x100169084` / `0x1001690fc` | `InsectBuzzLoop` | `[mLocustLogic isLocustActive]` |

Each has a sequence handler (`…:::`); the pause handlers run in the `Start…` and the resume
handlers in the `Stop…` sequences of The Runs, No Pee, Konkey Dong, Moron Pests and Battle of
the Gods (5 uses each).
`onStoryStart @ 0x10007821c` also calls `pauseSleepingSounds` and stops `WinchTurn`;
`onStoryComplete @ 0x100078344` resumes the snoring.

#### 1.7.4 Mute and Vibrate options

- At load (`PocketGodViewController::loadPreferences::: @ 0x10007bdcc`): `Options.Mute == 1`
  → `mute:YES`, mute button behaviour `StartMuteOn`, `stopAllSounds`; else `mute:NO`,
  `StartMuteOff`, `playBackgroundLoop:nil`. `Options.Vibrate == 1` → `muteVibrate:YES`,
  `StartVibrateOn`; else `muteVibrate:NO`.
- Toggle (`onButtonPress:touch: @ 0x100079090`, mute button): `Mute` 0 → `mute:YES`,
  `Mute = 1`, `ToggleMuteOn`, `stopAllSounds`; 1 → `mute:NO`, `Mute = 0`, `ToggleMuteOff`,
  `playBackgroundLoop:nil`. Vibrate button: `Vibrate` 0 → `muteVibrate:YES`, `Vibrate = 1`,
  `ToggleVibrateOn`; 1 → `muteVibrate:NO`, `Vibrate = 0`, `ToggleVibrateOff`.
  So **`Vibrate == 1` means vibration is off**.
- Muting does not stop sounds by itself (the callers do `stopAllSounds`); while muted, `play`
  does nothing and does not set `mbPlaying`, so `isPlaying`-guarded restarts work after unmute.
- Vibration is triggered by playing the sound id `Vibrate` (40 `playSound` items, 13
  timeline frames). The port maps it to gamepad rumble or ignores it, honouring the option.

#### 1.7.5 Other engine-relevant uses

- `DanceDanceExecutionScreenLogic::openDanceDanceExecution @ 0x1000362e4` preloads
  `DanceIntro`, `DanceLoop2`, `DanceFeedbackGreat`, `DanceFeedbackAwesome`,
  `DanceFeedbackYouRock`, `DanceFeedbackLookout` and the missing ids
  `DanceDanceButtonLeftPerfectHit/GoodHit/BadHit`; `closeScreen @ 0x100036894` and
  `quitScreen @ 0x100036a64` unload the same list.
- `DanceDanceExecutionScreenLogic::onGameFrame: @ 0x100036118`: while not game-over,
  once 7.5 s have passed since the intro started it plays `DanceLoop2`, and afterwards plays it
  again whenever `isPlaying` is NO (a manual loop with a gap of up to one frame).
- `UnderwaterTempleLogic::onButtonPress:touch: @ 0x1000c88d8` plays
  `TempleTone<index+1>` *(asm: `add w8, w22, #1`)*.
- `PygmyLogic::drag::: @ 0x100132094` picks `BabySound1`, `OldOh` or one of `Oh1`–`Oh3`.
- No audio-session handling: the app never pauses or resumes audio on background/foreground
  (`PocketGodAppDelegate` only stops/starts the render loop, `0x100089a24`–`0x100089c0c`).
  With iOS's default session category the hardware silent switch muted the game; the port has
  no equivalent.

### 1.8 Sound id references

| Source | Distinct ids | Unresolved (silent in the original) |
|---|---|---|
| Compiled sequences (`playSound` 750, `stopSound` 49, `playBackgroundLoop`, pedestal handlers) | 150 | `DoorCreek` in `OutHouseFinishCrapping` (the def is `DoorCreak`) |
| `Timelines.bin` frame sounds (1,526 frames) | 290 | `BitGulp`, `Bubble1`, `HitGround1`, `Holding It`, `LowSlap`, `Oh`, `Slide`, `Sqawk`, `TRexRoar2`, `campfire` (`timelines.md` §2.10) |
| Code (`getSoundById:` literals, formats, aliases) | ~40 | `DanceDanceButtonLeftPerfectHit`, `…GoodHit`, `…BadHit` (preload/unload only) |

Every one of the 359 defs is referenced by at least one source. The port must keep the
unresolved ids silent (do **not** "fix" them, e.g. `campfire` → `Campfire`).

### 1.9 Port notes (Bevy)

- Load `sounds.json`; decode FLAC to i16, apply the click fix, keep the samples resident (the
  original loads on first play; all 355 files decode to 55.7 MB of i16).
- Model per-sound state exactly: `playing` flag, current voice handle, `preload` flag; a pool of
  32 voices with the "first free slot after collecting finished, unpinned voices" rule.
- Restart semantics: replaying a non-multiple sound restarts its voice; replaying a running
  looping sound is ignored; multiple-instance sounds stack voices.
- `isPlaying` must report "voice exists and is still sounding" (DanceDance relies on it).
- Aliases are extra keys pointing at the same sound object.
- Gain 1.0 everywhere, no fades. Mono vs stereo level: see open question 6.1.

### 1.10 `sounds.json`

```
{
  "format_version": 1,
  "sounds": [SoundDef],        // struct-XML order (parse order)
  "aliases": [SoundAlias]
}
SoundDef = {
  "id": String,
  "file_name": String|null,    // base name in sound/, null for Vibrate
  "asset": String|null,        // "sounds/<file_name>.flac", relative to assets/
  "looping": bool,             // BCSound.mbLoop (AL_LOOPING)
  "multiple_instances": bool,  // new voice per play
  "stream": bool,              // always false in 1.48.2
  "vibrate": bool,             // true only for "Vibrate"
  "sample_rate": u32|null,     // 22000 or 44100
  "channels": u8|null,         // 1 or 2
  "frames": u32|null           // sample frames; duration = frames / sample_rate
}
SoundAlias = {"alias": String, "set_by": String, "targets": [String]}
```

There is no volume, pitch or priority field because the engine has none (gain and pitch are
always 1.0). `aliases` lists the runtime alias `BackgroundLoop` with every sound id that
`playBackgroundLoop:` can point it at (sorted), plus the `Garbage` sequence's
`setSoundAlias` item. Rust:

```rust
#[derive(Deserialize)]
pub struct SoundTable { pub format_version: u32, pub sounds: Vec<SoundDef>, pub aliases: Vec<SoundAlias> }
#[derive(Deserialize)]
pub struct SoundDef {
    pub id: String, pub file_name: Option<String>, pub asset: Option<String>,
    pub looping: bool, pub multiple_instances: bool, pub stream: bool, pub vibrate: bool,
    pub sample_rate: Option<u32>, pub channels: Option<u8>, pub frames: Option<u32>,
}
#[derive(Deserialize)]
pub struct SoundAlias { pub alias: String, pub set_by: String, pub targets: Vec<String> }
```

---

## 2. Preferences and save data

### 2.1 The four documents

`PocketGodViewController::loadGame @ 0x10006a030` creates four `BCPreferences` objects right
after `parseBinAnim:ext:` and calls `loadPreferences` on each:

| Document | ivar | `initWithId:secure:iCloud:` | Legacy file | Storage |
|---|---|---|---|---|
| `Persona` | `mPersonaPreferences` | NO, NO | `Preferences.plist` | `Documents/Persona.plist` |
| `Options` | `mOptionPreferences` | NO, NO | `Preferences.plist` | `Documents/Options.plist` |
| `System` | `mSystemPreferences` | NO, NO | `Preferences.plist` | `Documents/System.plist` |
| `Challenge` | `mChallengePreferences` | **YES, YES** | — | keychain item + iCloud document |

Getters `personaPreferences @ 0x10006da98`, `optionPreferences @ 0x10006daa8`,
`systemPreferences @ 0x10006dab8`, `challengePreferences @ 0x10006dac8` let managers reach
them through their listener. Multiplayer code also uses throw-away `BCPreferences` objects
as match-data containers (2.6.5).

### 2.2 `BCPreferences` (class)

Ivars: `mPreferencesListener`, `mPreferenceId`, `mPreferenceFileName` (`"%@.plist"`),
`mPreferenceICloudFileName` (`"iCloud%@.plist"`), `mLegacyPreferencesFileName`,
`mPreferenceDictionary` (NSMutableDictionary, the data), `mPreferenceDictionaryForThread`,
`mKeyChain`, `mbPreferencesLoadComplete`, `mbICloudEnabled`, `mbICloudPreferences`,
`mbSecurePreferences`, `mbSavePreferences` (dirty), `mbWaitingToSave`, `mLastSaveTime` (f32),
`mQuery`, `mPreferencesDocument`, `mpLoadInstanceVector` (pending downloads), `mUrlConnection`,
`mReceivedData`, `mbLoaded`, `mbDebug`.
`initWithId:secure:iCloud: @ 0x10016b2a8`; `id @ 0x10016b3e0` returns `@"BCPreferences"`;
`gameFrameOrder @ 0x10016e7a4` = 40.

#### 2.2.1 Value model and accessors

The dictionary is a property list: keys are NSStrings; values are NSNumber (int or float),
NSString, or NSDictionary (string → NSNumber). The game only ever stores those.

| Method | Address | Semantics |
|---|---|---|
| `getIntPreference:` | `0x10016d580` | `[dict[key] intValue]`, **0 when absent** |
| `getFloatPreference:` | `0x10016d6cc` | `floatValue`, 0.0 when absent |
| `getStringPreference:` | `0x10016d814` | object or nil |
| `getDictionaryPreference:` | `0x10016d8f8` | object or nil |
| `doesPreferenceExist:` | `0x10016d54c` | object != nil |
| `setPreference:asInt:` | `0x10016d5c8` | `dict[key] = @(int)`; `onPreferencesChanged` |
| `setPreference:asFloat:` | `0x10016d714` | `@(float)`; changed |
| `setPreference:asString:` | `0x10016d830` | `setObject:forKey:` (a nil string raises in Foundation; never happens); changed |
| `setPreference:asDictionary:` | `0x10016d914` | nil → ignored; else stores a **mutable copy**; changed |
| `setDefaultPreference:asInt:` | `0x10016d644` | existing → returns its `intValue`, no write; absent → set, return the default |
| `setDefaultPreference:asFloat:` | `0x10016d78c` | same for floats |
| `setDefaultPreference:asString:` | `0x10016d884` | existing → returns it; absent → set |
| `setDefaultPreference:asDictionary:` | `0x10016d9e0` | existing → returns it; absent → `setPreference:asDictionary:` (a nil default stores nothing) and returns the default |
| `removePreference:` | `0x10016d504` | remove; changed |
| `loadPreferencesFromData:` | `0x10016da54` | replace the dictionary with the plist in the data (empty data → empty dict; parse error → `initPreferences`); marks load complete |
| `dataFromPreferences` | `0x10016db90` | binary plist of the dictionary |

`onPreferencesChanged @ 0x10016d3b4` sets `mbWaitingToSave` and `mbSavePreferences` and, with a
document, `updateChangeCount:UIDocumentChangeDone`. `waitingToSave @ 0x10016d358` =
`mbWaitingToSave || document.hasUnsavedChanges` (used by `DebugScreen::waitForDataUpdate:::
@ 0x100009008` on the Challenge document).

#### 2.2.2 Loading

```
loadPreferences @ 0x10016c920:
    if mbICloudPreferences && isICloudAvailable: start the iCloud query (2.2.4)
    else: onPreferencesLoadComplete
onPreferencesLoadComplete @ 0x10016cc58:
    mbPreferencesLoadComplete = YES
    if dict == nil: loadLocalPreferences          // secure → keychain, else file
    [BCView addGameFrameListener:self]            // enables the save timer
loadFilePreferences @ 0x10016ce40:
    data = Documents/<Id>.plist, else Documents/<legacy> (if a legacy name is set)
    neither → initPreferences
    parse (any plist format, immutable) → dict = mutable copy; parse failure → initPreferences
loadSecurePreferences @ 0x10016cfe0:
    keychain = keyChainWithKeyUsingAppName:"PocketGod" key:"Preferences"
    if hasBeenSet: parse [secret dataUsingEncoding:NSASCIIStringEncoding] → dict
    else (or parse failure): initPreferences
initPreferences @ 0x10016cd50:
    dict = {"TEST": "TEST"}; mbSavePreferences = YES
```

Legacy migration: all three plain documents fall back to the same pre-split file
`Documents/Preferences.plist`, so on the first launch after the split each of them starts
from a full copy of the old dictionary (stale keys of other documents included) and later
saves only to its own file. The legacy file is never deleted or written.

Non-iCloud loads are synchronous, so `PocketGodViewController::waitForPreferencesLoadComplete:::
@ 0x10007bce8` (returns 2 once all four report `isPreferencesLoadComplete`) only waits for the
iCloud case.

#### 2.2.3 Saving

```
onGameFrame:(dt) @ 0x10016e7ac → onGameFrameManageSavingPreferences:(dt) @ 0x10016d3f8:
    mLastSaveTime += dt
    if mLastSaveTime > 2.0:
        mLastSaveTime = 0
        if mbSavePreferences:
            if !secure: mbWaitingToSave = NO; mbSavePreferences = NO; saveFilePreferences
            elif no save thread running (mPreferenceDictionaryForThread == nil):
                mbSavePreferences = NO
                mPreferenceDictionaryForThread = [NSDictionary initWithDictionary:dict]  // shallow
                detach thread saveSecurePreferences
flush @ 0x10016d344: mLastSaveTime = 2.0          // the next frame saves if dirty
```

- `saveFilePreferences @ 0x10016d11c`: binary plist (`NSPropertyListBinaryFormat_v1_0`) written
  atomically to `Documents/<Id>.plist`.
- `saveSecurePreferences @ 0x10016d1d0` (background thread): clears `mbWaitingToSave`, XML plist
  (`NSPropertyListXMLFormat_v1_0`) of the snapshot → `NSString` with **ASCII** encoding →
  keychain secret, then releases the snapshot (allowing the next save). (Non-ASCII text would make the string nil; the Challenge
  document only holds numbers.)
- Saving is therefore throttled to at most once per 2 s of game time, and only while the render
  loop runs. `IdolManager` calls `flush` after resets/unlocks.

#### 2.2.4 iCloud (Challenge document only)

- `isICloudSupported @ 0x10016b68c`: `UIDocument` exists and iOS ≥ 5.0;
  `isICloudAvailable @ 0x10016b70c`: supported and
  `URLForUbiquityContainerIdentifier:@"CGMFV4QFBB.<bundle id>"` (`ubiquitousContainerUrl @
  0x10016b5e4`) is non-nil.
- `loadPreferences`: `mbICloudEnabled = YES`, observe `UIApplicationDidBecomeActive`
  (`applicationDidBecomeActive @ 0x10016b760` only logs and toggles `mbICloudEnabled`), create
  `<container>/Prefs`, start an `NSMetadataQuery` (ubiquitous data scope,
  `"%K like '*.plist'"` on the file name).
- `onFoundPreferencesPlist @ 0x10016c2b0` (gathering finished / updated): if a result is named
  `iCloudChallenge.plist`: dict = new empty dict, open a `BCPreferencesDocument` on it
  (`createPreferencesDocument: @ 0x10016c0ac`; its open completion calls
  `onPreferencesLoadComplete`). Otherwise: `loadLocalPreferences` (keychain), write the
  dictionary as a binary plist to `Documents/iCloudChallenge.plist`, move it into
  `<container>/Prefs/` with `setUbiquitous:` and open the document there; on any failure
  `mbICloudEnabled = NO` and `onPreferencesLoadComplete`. The query is stopped once a document
  exists.
- Conflicts (`onPreferencesDocumentStateChanged @ 0x10016bb48`, state has
  `UIDocumentStateInConflict`): every unresolved conflict version is parsed and merged into the
  dictionary with `mergeDictionary:fromDictionary: @ 0x10016b850`, other versions are removed,
  conflicts marked resolved, then `onPreferencesChanged` and the listener's
  `onPreferencesChanged:` (IdolManager reloads episode state, 2.6.4).
- Merge rule, for each key of the other version: absent locally → copy; NSString → keep
  local; NSNumber → keep the **larger** (`[other compare:local] == NSOrderedDescending`);
  NSDictionary → replace the local value by a mutable copy and merge recursively.
- With iCloud the dictionary is saved both by the document (autosave after
  `updateChangeCount:`) and to the keychain (secure flag).

#### 2.2.5 `BCPreferencesDocument : UIDocument`

`initWithFileURL:delegate:preferenceDictionary: @ 0x10016acb8` (shares the `BCPreferences`
dictionary object), `loadFromContents:ofType:error: @ 0x10016ae14` (parse plist → replace all
entries of the shared dictionary → `preferencesDocumentContentsUpdatedCallBack:` → main thread
→ listener `onPreferencesChanged:`), `contentsForType:error: @ 0x10016afd4` (binary plist),
`handleError:userInteractionPermitted: @ 0x10016b0e0` (logs and **throws**
`PocketGodiCloudException`).

#### 2.2.6 Server downloads

`loadDataFromServer:listenerData:url: @ 0x10016e3dc` queues `{listener, data, url}` and
downloads them one at a time with `NSURLConnection` (`startNextLoadInstance @ 0x10016e2fc`);
completion is delivered on the next game frame as
`[listener onLoadDataFromServerComplete:listenerData url:url data:]`, failures as
`onLoadDataFromServerError:url:error:`. Only `NewsManager` uses it (news feed
`http://appdata.boltcreative.com.s3.amazonaws.com/news/news.xml` and news icons). The host is
gone; the port should ship without network loading (or make it optional).

#### 2.2.7 File helpers (used by other managers)

All relative to `NSDocumentDirectory` unless noted: `applicationDataFromFile: @ 0x10016df3c`,
`writeApplicationData:toFile: @ 0x10016e090` (atomic), `removeFile: @ 0x10016de80`,
`doesFileExistInDocuments:ext: @ 0x10016dc90`, `doesFileExistInResources:ext: @ 0x10016dc14`
(bundle), `applicationDataFromResourceFile:ext: @ 0x10016e004`; Library/Caches variants
`applicationDataFromLibraryCacheFile: @ 0x10016e138`,
`writeApplicationData:toLibraryCacheFile: @ 0x10016e200` (sets the
`com.apple.MobileBackup` xattr = 1, i.e. excluded from backup),
`doesFileExistInLibraryCache:ext: @ 0x10016dd88`.

### 2.3 `BCKeyChain`

`keyChainWithKeyUsingAppName:key: @ 0x10001ef04` returns a cached object per
`"<app><key>"`. `initWithKeyUsingAppName:key: @ 0x10001f0a8` *(asm)* queries
`{kSecClass: kSecClassGenericPassword, kSecAttrAccount: key, kSecAttrService: app,
kSecReturnAttributes: YES}`; found → `mSecret = attributes[kSecAttrGeneric]`; not found
(−25300) → adds the item with `kSecAttrGeneric = "THISISTHEUNSETSECRET"`; other errors set
`mbError`. `setSecret: @ 0x10001f39c` updates `kSecAttrGeneric`; `reset @ 0x10001f528` deletes
the item; `hasBeenSet @ 0x10001f660` = secret != `"THISISTHEUNSETSECRET"`;
`keyChainWithKeyUsingAppName:key:defaultSecret: @ 0x10001f028` sets the default only when the
secret equals `@""` *(asm)*; a freshly created item holds `"THISISTHEUNSETSECRET"`, so the
`"0"` default of the skin options never applies to new items (their `intValue` is 0 anyway). The secret lives in an **attribute**, not the password data.

Keychain items (service `"PocketGod"`):

| Account | Value | Owner |
|---|---|---|
| `Preferences` | XML plist of the Challenge document (ASCII) | `BCPreferences` (2.2.3) |
| `<Pack>SkinPack` (24 packs, table `0x10037d620`) | `"BabaBooey"` = purchased | `SkinPackManager::initSkinPackDefs @ 0x10005d694`, `completeTransaction:`/`restoreTransaction: @ 0x10005e4e8` (`textures.md` §7) |
| `<skin option id>` (e.g. `TRex Body`) | selected value index as a decimal string, default `"0"` | `SkinPackManager::initSkinPackDef: @ 0x10005d7c8`, `updateSkinPackDef:… @ 0x1000601e4` |
| `ComicPreOrder` | `"BabaBooey"` = bonus pack unlocked | `BonusPackManager::initBonusPackDefs @ 0x1000cad40`, `checkBonusPackUnlock:score: @ 0x1000cb0c8`, `onKeyboardTextEntered: @ 0x1000cba6c` |

Products whose id is `"FreeBee"` are owned without a keychain item.

### 2.4 Other persistent files

| File | Format | Owner |
|---|---|---|
| `Documents/Persona.plist`, `Options.plist`, `System.plist` | binary plist dictionaries | 2.2 |
| `Documents/Preferences.plist` | legacy (pre-split) dictionary, read only | 2.2.2 |
| `Documents/iCloudChallenge.plist` | staging copy moved into iCloud | 2.2.4 |
| `Documents/StoryMap` | XML list of saved stories (`StoryManager::saveStoryMapXml @ 0x100048e8c` / `loadStoryMapXml @ 0x100048f10`) | story editor |
| `Documents/story-<yyyyMMddHHmmss>` | one XML story each (`<story date=…><storyTitle><![CDATA[…]]></storyTitle>…</story>`, name from `FUN_100048be0`: date format `yyyy-MM-dd HH:mm:ss` with `:`, `-`, space removed); `saveStoryToFile: @ 0x1000489bc`, `loadStoryFromFile: @ 0x100048d28`, `deleteActiveStoryDef @ 0x10004dae8` | story editor |
| `Library/Caches/news.xml` | cached news feed incl. the per-item read flag: `<news>` + per item (`FUN_100055864`) `<newsItem order="%d" wasRead="%d" id="%@" imageUrl="%@" imageFileName="%@" imageExt="%@" linkUrl="%@">` with `newsTitle`, `newsTease`, `newsArticle` CDATA children (`NewsManager::saveNewsToFile @ 0x100055740`, `loadNewsFromFile @ 0x100055964`; the bundled `news.xml` uses `newsItemId=` instead of `id=`) | news |
| `Library/Caches/<icon>.raw` | downloaded news icons (`onLoadDataFromServerComplete:url:data: @ 0x100057ab4`) | news |

The story XML schema belongs to the story spec; only the storage is described here.

### 2.5 Personas

`loadPreferences::: @ 0x10007bdcc` creates the personas on the **Persona** document
(`preferences.json → personas`):

| id | class | default name | default description | role |
|---|---|---|---|---|
| `God` | GodPersona | GENE SIMIANS | God of Thunder and Rocks that Roll | main |
| `Tribe` | TribePersona | TRIBE OOKGA CHAKA | One Tribe, One World | main |
| `Pygmy0`…`Pygmy5` | PygmyPersona | OOGA, BOOGA, NOOBY, DOOBY, KLIK, KLAK | see JSON | `addPygmyPersona:` |
| `CharliePygmy` | PygmyPersona | Charlie | Winning!. | guest, personality 1; activated at load when `Options.CharlieState == 1` |
| `TomPygmy` | PygmyPersona | Tom | Show me the money!. | guest, personality 2 |

Every persona registers its keys with `setDefaultPreference` in
`Persona::initWithId:defaultName:defaultDescription:preferences: @ 0x1001a44d8` and
`GodPersona::initWithId:… @ 0x1001a646c`, then the view controller becomes the Persona
document's listener (`onPreferencesChanged: @ 0x10007bd80` → God and Tribe
`refreshFromPreferences`).

`Persona::updateMood @ 0x1001a502c` (called after every sacrifice/gift count change),
with `s = TotalSacrifices`, `g = TotalGifts`, `r = (g > 0) ? s / g : s` (f32):

```
mood = 0  if r < 4 and s >= 21
       1  elif r < 7 and s >= 11
       2  elif r < 10
       3  elif r < 20
       4  elif r < 30
       5  elif r < 40
       6  otherwise
```

`Persona::refreshFromPreferences @ 0x1001a4904` reloads every key and repairs
`TotalSacrifices` / `TotalGifts` upward to the sum of the sacrifice / gift dictionary if it is
smaller (through `setTotalSacrifices:` / `setTotalGifts:`, which also update the mood).
`TribePersona::sacrifice:pygmyLogic: @ 0x1001a6090` and `addGift:pygmyLogic: @ 0x1001a5fe4`
update the tribe **and** the pygmy's own persona.

### 2.6 Key catalogue

Types: `int` = NSNumber from `numberWithInt:`; `float` = `numberWithFloat:`; `string`;
`dict` = dictionary of string → int. "Default" is the value registered by
`setDefaultPreference` (absent keys read as 0 / nil otherwise). Every document that starts
empty (no file, no legacy file, no keychain item) also holds `{"TEST": "TEST"}` from
`initPreferences`; nothing reads it.

#### 2.6.1 Options

Defaults registered by `loadPreferences::: @ 0x10007bdcc` (in this order; `preferences.json →
defaults`). Feature toggles use 1 = on. Setters `PocketGodViewController::set…State:startup:`
(`0x10007d5c4`–`0x100086d90`) store the new state as `(current == new) ? 0 : new` for the
multi-valued ones (choosing the active feature again turns it off), and the plain toggles are
flipped in `onButtonPress:touch: @ 0x100079090`.

| Key | Default | Values / meaning | Writers |
|---|---|---|---|
| `WeatherHazard` | 0 | storm-cloud weather on | `setWeatherState:startup:`, `stopDragClouds @ 0x10015f50c` |
| `ShowIdolStartScreen` | 1 | show the idol start screen at launch | check box in the idol options panel, toggled 0 ↔ 1 by `IdolScreenLogic::onButtonPress:touch: @ 0x100196380` |
| `ShowEpisodeIcon` | 1 | show the episode alert icon (`EpisodeAlertLogic::activate: @ 0x1001a3ff8`) | check box, same function |
| `ShowBattleHelp` | 1 | show the battle help panel | check box, same function |
| `IslandLocation` | 5 | island location state: 0 Sand, 1 Rock, 2 Underwater, 3 Graveyard, 4 Ice, 5 Ape, 6 Misfit, 7 God, 8 Boat, 9 Coral reef, 10 Apocalypse, 11 Time temple, 12 Locust temple, 13 Dance temple, 14 Plague temple, 15 Purification temple, 16 Closet temple, 17 Xenu temple | `setIslandLocationState:startup: @ 0x100083b64` |
| `SandIslandHazard` | 3 | 0 none, 1 boulder, 2 fishing pole, 3 ant hill + magnifying glass, 4 water wings | `@ 0x100083f50` |
| `RockIslandHazard` | 1 | fishing | `@ 0x1000842bc` |
| `RockIslandFeature` | 1 | 0 none, 1 wood (camp fire), 2 T-Rex egg | `@ 0x100084414` |
| `RockIslandOutHouseState` | 1 | out-house | `@ 0x10008490c` |
| `RockIslandTRexState` | 1 | T-Rex | `@ 0x100084a48` |
| `RockIslandSpearState` | 1 | spear statue | `@ 0x100084b80` |
| `GraveyardIslandState` | 0 | water wings | `@ 0x100084e24` |
| `GraveyardIslandZombieGraveState` | 1 | zombie graves | `@ 0x100084f60` |
| `GraveyardIslandGhostState` | 1 | ghost | `@ 0x10008509c` |
| `GraveyardIslandSpiderWebState` | 1 | spider web | `@ 0x100085198` |
| `GraveyardIslandSpiderState` | 1 | spider | `@ 0x100085488` |
| `GraveyardIslandSpearState` | 1 | spear statue | `@ 0x1000855c0` |
| `IceIslandState` | 1 | 0 none, 1 ice fishing, 2 water wings | `@ 0x100085864` |
| `IceIslandIceMonsterState` | 1 | ice monster | `@ 0x100085a48` |
| `IceIslandIglooState` | 1 | igloo | `@ 0x100085b84` |
| `IceIslandIceHoleState` | 1 | ice hole | `@ 0x100085cc0` |
| `IceIslandIceSquidState` | 1 | ice squid | `@ 0x100085dfc` |
| `ApeIslandState` | 2 | 0 none, 1 ape altar, 2 banana tree | `@ 0x100085f30` |
| `ApeIslandApeState` | 1 | ape | `@ 0x10008619c` |
| `ApeIslandMonkeyState` | 1 | monkeys | `@ 0x1000862d8` |
| `MisfitIslandState` | 1 | 0 none, 1 moron snake, 2 water wings | `@ 0x1000863d4` |
| `GodIslandState` | 0 | water wings | `@ 0x10008658c` |
| `GodIslandIdolsState` | 1 | idols on God island | `@ 0x1000866c8` |
| `ApocalypseIslandState` | 0 | water wings | `@ 0x100086988` |
| `UnderwaterHazard` | 2 | 0 none, 1 piranha, 2 laser shark, 3 tile drain | `@ 0x100086b00` |
| `UnderwaterCoralReefState` | 1 | jellyfish | `@ 0x100086d90` |
| `SharkState` | 1 | shark | `setSharkState:startup: @ 0x10007d894` |
| `BirdState` | 1 | bird | `setBirdState:startup: @ 0x10007d9c4` |
| `TemperatureState` | 1 | temperature power button | `onButtonPress:touch:` |
| `HurricaneState` | 1 | hurricane power button | same |
| `RainbowState` | 1 | rainbow | same |
| `CrackHoleState` | 1 | crack hole | same |
| `TheRunsState` | 1 | The Runs | same |
| `CharlieState` | 1 | Charlie guest pygmy | same |
| `KonkeyDongState` | 1 | Konkey Dong | same |
| `MoronPestsState` | 1 | Moron Pests | same |
| `DrumHazard` | 1 | drum (celebration) | same |
| `GravityHazard` | 1 | gravity | same |
| `Mute` | 0 | 1 = all sound off (1.7.4) | same |
| `Vibrate` | 0 | 1 = vibration off (1.7.4) | same |
| `DisplayNames` | 1 | pygmy name labels; story mode forces 1 and restores it (`onStoryStart`/`onStoryComplete`) | same |
| `ButtonLabels` | 1 | menu button labels | same |
| `FailBoatState` | — (0) | fail boat | `setFailBoatState:startup: @ 0x100086ac4` |
| `IdolsState` | — (0) | read at load (`StartIdolsOn`), **never written** in 1.48.2 (legacy) | — |
| `ActiveEpisode` | — (nil) | id of the active episode, `""` when none | `IdolManager::onEpisodeActivated: @ 0x10002f71c`, `resetEpisodeState @ 0x10002ae14`, `resetIdolChallenges @ 0x10002b338` |
| `<episodeId>SelectionSlot` | −1 (`loadEpisodeState @ 0x10002b7b8`) | idol slot the episode idol occupies, −1 none | `onEpisodeIdolSelected:slot: @ 0x10002c6a0`, `onEpisodeIdolDeselected: @ 0x10002c560` |
| `GodIdolSlot<pedestal>` | −1 (`IdolPedestalLogic::godIdolSlot @ 0x10013f1ac`) | slot shown on pedestal 1–4 | `setGodIdolSlot: @ 0x10013ee9c` (key built in `setPedestalIndex: @ 0x10013ee28`) |

Readers: `updateIslandStateAtStartup @ 0x10007e398` reads every island key and replays the
setters with `startup:YES`; the `is…Enabled`/`are…Enabled` getters (`0x100087034`–`0x100088168`)
read single keys at run time.

#### 2.6.2 System

| Key | Type | Default | Meaning | Code |
|---|---|---|---|---|
| `MisfitIslandFound` | int | 0 | Misfit island discovered (set to 1 from a dream bubble) | `loadPreferences:::`, `onDreamBubbleChangeIslandLocationState: @ 0x100169ef4`, `hasMisfitIslandBeenFound @ 0x1000881d8` |
| `InfoShown` | int | — | skin-pack info shown once | `openSkinPackManager @ 0x1000778b8` |
| `BonusInfoShown` | int | — | bonus-pack info shown once | `openBonusPackManager @ 0x100077abc` |
| `WhatsNewVersion` | string | — | last seen `CFBundleVersion` (`"7"` in 1.48.2); the idol start screen opens when it differs or `ShowIdolStartScreen != 0` | `checkForIdolStartScreen @ 0x1001599f0` |
| `pg.<game>.<tier>` (72 keys) | int | — | 1 once the Game Center achievement was earned | `checkFeintAchievements:score: @ 0x10007492c` |

Achievements (`addFeintAchievementsToArray:achievementId:score: @ 0x100073ebc`): the score is
`God:AchievementDictionary[id]`; each tier is added when its condition holds (earlier tiers
included), and `checkFeintAchievements:` writes `System[achievement] = 1` and submits it:

| Achievement id (dictionary key) | Game Center prefix | minion | shaman | demigod | god |
|---|---|---|---|---|---|
| `CoconutBounces` | `pg.coconutbouncing` | ≥1 | ≥20 | ≥50 | ≥100 |
| `HurricaneLightning` | `pg.hurricanezap` | ≥1 | ≥6 | ≥30 | ≥100 |
| `SharkSnaps` | `pg.sharksnap` | ≥1 | ≥50 | ≥500 | ≥1000 |
| `Skewered` | `pg.pygmykabob` | ≥1 | ≥4 | ≥5 | ≥6 |
| `Bowled` | `pg.pygmybowling` | ≥1 | ≥4 | ≥5 | ≥6 |
| `BirdBombs` | `pg.birdbomb` | ≥1 | ≥10 | ≥30 | ≥100 |
| `OogaJump` | `pg.oogajump` | ≥1 | ≥1000 | ≥3000 | ≥6000 |
| `BaitMaster` | `pg.baitmaster` | ≥1 | ≥10 | ≥50 | ≥200 |
| `LaserShark` | `pg.sharkblast` | ≥1 | ≥50 | ≥300 | ≥800 |
| `KillTRex` (lower is better) | `pg.trexhunt` | 1–1000 | <36 | <21 | <13 |
| `KillSpider` (lower is better) | `pg.spiderhunt` | 1–1000 | <36 | <21 | <13 |
| `KillIceMonster` (lower is better) | `pg.icemonsterbattle` | 1–1000 | <31 | <16 | <11 |
| `PainDrain` | `pg.paindrain` | ≥1 | ≥1000 | ≥4000 | ≥10000 |
| `TheRuns` | `pg.theruns` | ≥1 | ≥8000 | ≥30000 | ≥60000 |
| `KonkeyDong` | `pg.konkeydong` | ≥1 | ≥15000 | ≥50000 | ≥100000 |
| `MoronPests` | `pg.moronpests` | ≥1 | ≥20000 | ≥50000 | ≥100000 |
| `DanceDanceExecution` | `pg.dancedanceexecution` | ≥1 | ≥1000 | ≥5000 | ≥10000 |
| `NoPee` | `pg.nopee` | ≥1 | ≥4 | ≥10 | ≥20 |

Tier ids are `<prefix>.minion`, `.shaman`, `.demigod`, `.god`. The checks run from
`onAchievement:count:highScoreWins: @ 0x100070cc4` and, for all 18 ids,
`checkAllAchievementsAndLeaderboards @ 0x100070ae4` (once after load when
`mbCheckAllAchievementsAndLeaderboards` is set).

#### 2.6.3 Persona (per persona id `P`, 2.5)

| Key | Type | Default | Meaning |
|---|---|---|---|
| `P:Name` | string | default name | display name (`setName:` @ `0x1001a4eac`) |
| `P:Description` | string | default description | (`setDescription:` @ `0x1001a4f74`) |
| `P:PersonaMood` | int | 2 | mood 0–6 (2.5) |
| `P:TotalSacrifices` | int | 0 | sacrifices made |
| `P:SacrificeDictionary` | dict | — | sacrifice id → count (`addSacrifice:` @ `0x1001a5350`) |
| `P:TotalGifts` | int | 0 | gifts given |
| `P:GiftDictionary` | dict | — | gift id → count (`addGift:` @ `0x1001a5838`) |
| `God:PersonaGodType` | int | 1 | god type (`GodPersona::setGodType:` @ `0x1001a675c`) |
| `God:AchievementDictionary` | dict | — | achievement id → best count; `setAchievement:count:highScoreWins: @ 0x1001a6840` keeps the larger value when `highScoreWins`, else the smaller non-zero one; `addIncrementalAchievement: @ 0x1001a6978` adds 1 (`count == −1`) |

#### 2.6.4 Challenge (episode / idol progress, `IdolManager`)

Episode and event ids come from `episodes.json` (`struct-xml.md` §8; the key strings live in
runtime fields of the episode nodes).

| Key | Type | Meaning | Code |
|---|---|---|---|
| `<episodeId>BattleUnlock` | int 0/1 | battle unlocked for the episode | `loadEpisodeState @ 0x10002b7b8`, `unlockNextBattleIdol: @ 0x10002c8f8`, `unlockBattleState @ 0x10002b04c`, `resetBattleState @ 0x10002af40`, `resetAllChallenges @ 0x10002ac2c` |
| `<episodeId><eventId>Challenge` (node +0x50) | int 0/1 | challenge completed | `triggerEpisodeEvent: @ 0x10002fd90`, `…BySacrificeId: @ 0x10002f930`, `…ByGiftId: @ 0x10002fb60`, `checkImmediateUnlocks: @ 0x10002a88c`, resets |
| `<episodeId><eventId>ChallengeHint` (node +0x58) | int 0/1 | hint bought/activated | `onHintActivated: @ 0x10002f840`, resets |
| `<episodeId><eventId>LevelUp` (level-up node +0x48) | int | accumulated points, capped at the node maximum (+0x28) | `triggerEpisodeGameScoreEvent:score: @ 0x10002ffb8`, `applyBattleExperience @ 0x10002eb2c`, `setIdolLevels: @ 0x10002b158`, `resetIdolLevels @ 0x10002b28c` |

`IdolManager::onPreferencesChanged: @ 0x10002b5fc` (iCloud updates) re-reads these and
refreshes every episode.

#### 2.6.5 Match data (Game Center turn-based matches, not a file)

`ChallengeManager::parseMatchData: @ 0x1000158e4` and `IdolManager::parseMatchData: @
0x100032d0c` wrap `GKTurnBasedMatch.matchData` in a fresh `BCPreferences`
(`loadPreferencesFromData:`); `dataFromPreferences` (binary plist) is sent back at turn end.

| Key | Type | Meaning |
|---|---|---|
| `GameCategory` | string | `MiniChallenge` or `BattleOfTheGods` |
| `GameId` | string | mini-game id (mini challenges) |
| `GameVersion` | string | highest `CFBundleVersion` of the two players (`NSNumericSearch`) |
| `ChallengerId`, `RecipientId` | string | Game Center player ids |
| `ChallengerAlias`, `RecipientAlias` | string | player aliases |
| `ChallengerGameScore`, `RecipientGameScore` | int | mini-challenge scores (`submitChallenge::: @ 0x100017330`, `submitChallengeResult::: @ 0x100017440`) |
| `<Side>Idol<0-3>Id`, `<Side>Idol<0-3>Level` | string, int | battle team (`IdolManager::setActiveMatch: @ 0x1000321e0`) |
| `<Side>BattleIdolId` | string | idol chosen for the current round |
| `<Side>RandomFrozenIdolId` | string | |
| `<Side>Health`, `<Side>MaxHealth`, `<Side>TotalDrainLevelPoints` | float | battle state (`saveTurnPlayerBattleState:preferencePrefix: @ 0x100030668`) |
| `<Side>Idol<n>ActiveFrameCount`, `…UsedCount`, `…FrozenRefCount` | int | per-idol state (`saveTurnIdolBattleState:preferencePrefix: @ 0x100030358`) |
| `<Side>Idol<n>FrozenIdolId` | string | |
| `StateSaved` | int | 1 once a turn state was saved |

`<Side>` is `Challenger` or `Recipient`; `IdolManager::initBattleOfTheGodsMultiPlayer @
0x100030c60` sets the player prefix to `Challenger` when `ChallengerId` is the local player,
else `Recipient` (the enemy prefix is the other one). Multiplayer needs Game Center; the port
can keep this layout for a local or future online mode.

### 2.7 Port notes

- Keep the four-document model with a typed value enum (`Int(i32)`, `Float(f32)`,
  `String(String)`, `Dict(BTreeMap<String, i32>)`) and the exact accessor semantics (absent →
  0/nil, `set_default` returns the existing value, dirty flag + 2 s game-time save throttle,
  `flush`).
- Store each document in the platform data directory (e.g. JSON or a binary plist via the
  `plist` crate). Keep the Challenge document local; replace the keychain with a small
  `keychain` document holding the items of 2.3. iCloud, the legacy file and the merge rule
  only matter for importing iOS saves; supporting `Preferences.plist`/`*.plist` import with the
  documented precedence is a nice-to-have.
- Run `loadPreferences:::` logic (defaults from `preferences.json`, then personas) before
  anything reads options.

### 2.8 `preferences.json`

```
{
  "format_version": 1,
  "documents": [{"id", "ivar", "file_name", "icloud_file_name",
                 "legacy_file_name": String|null, "secure": bool, "icloud": bool}],
  "defaults": [{"document": String, "key": String, "value_type": "int", "value": i32,
                "address": String}],          // loadPreferences::: call order
  "personas": [{"id", "class", "default_name", "default_description",
                "document": "Persona", "personality": i32|null,
                "role": "main"|"pygmy"|"guest_pygmy"}],
  "persona_keys": [{"class": "Persona"|"GodPersona", "key_format": String,  // "%@" = persona id
                    "value_type": "int"|"string"|"dictionary",
                    "default_value": i32|null,
                    "default_from": "default_name"|"default_description"|null}]
}
```

All four tables are read from the machine code by the small symbolic arm64 scanner in
`extract_engine_tables.py` (it follows `adrp/add/ldr/mov` and stack spills into the
`objc_msgSend` argument registers): `initWithId:secure:iCloud:` and
`setLegacyPreferenceFileName:` in `loadGame`, the `setDefaultPreference:asInt:` and persona
calls in `loadPreferences:::`, and the `stringWithFormat:` + `setDefaultPreference:as…:` pairs
in the two persona initialisers. `GodPersona` keys apply to the `God` persona only; `Persona`
keys to every persona. `GodPersona`'s init also sets every `Persona` key (it calls super).

---

## 3. Fonts and text

### 3.1 Bitmap fonts

Text is drawn from **pre-rendered glyph sprites in the texture atlases**; there is no
UIFont/CoreGraphics/CoreText text rendering anywhere in the binary. The layout and drawing
code is specified in `rendering.md` §5.7 (`BCText`) and §6 (`BCFontDef`: `textWidth:`,
word wrap, `fillFontCharTextureArray:…`, justification); this section adds the data.

- Font defs: struct-XML node type 12 in root `0x10064a270` (`struct-xml.md` §4.2), parsed by
  `BCLibrary::parseStructXml:` → `BCFontDef::initWithId: @ 0x100066780` +
  `setCharSet: @ 0x100066a34` → `BCLibrary::addFontDef: @ 0x100023af0` (dictionary,
  `getFontDefById: @ 0x100023ad4`). The `folder`/`file_name`/`ext` fields
  (`../BoltLib/AppFonts/Arial12-White.png`, …) were inputs of the offline atlas builder; the
  PNGs are not shipped.
- Glyph `i` of the char set uses texture def `"<font>:FullSet:<i>"` from the atlas texture-def
  tables (`0x1005f9738`, `0x1007c4988`); the font maps the glyph's **byte** value to that
  def (256-entry table, zeroed = unmapped).
- Metrics come from the `BCTextureDef`: advance = `+0x28` (frame width), line advance =
  `+0x2c` (frame height of the last emitted glyph), quad = the def's 4 vertices
  (`+0x50`, x/y/z each, already offset inside the frame), UVs `+0x30`. `textWidth: @
  0x100066818` and `lengthOfWordsThatFit:width: @ 0x1000668a8` use `+0x28` *(checked)*.

| Font | Glyphs | Char set | Frame height | Advance range | Space advance | Used by |
|---|---|---|---|---|---|---|
| `Arial12` | 95 | printable ASCII | 12 | 2–12 | 5 | 84 timeline texts, 114 display-group texts |
| `Arial12Black` | 95 | printable ASCII | 12 | 2–12 | 5 | 1 timeline text, 2 display-group texts |
| `AnimatedGothicLite20` | 95 | printable ASCII | 21, 22 or 24 (per glyph) | 4–19 | 6 | 82 timeline texts, 538 display-group texts |
| `Doop24Green` | 12 | `" L0123456789"` | 22 | 6–11 | 8 | 9 timeline texts (scores) |

Because `AnimatedGothicLite20` glyphs have different frame heights, its line spacing depends
on the last glyph of the previous line (original behaviour, keep it). Glyph sprites are spread
over the atlas pages `TextureAtlasTest0`–`TextureAtlasTest6` (`atlasTest0`–`atlasTest6`);
space glyphs have a zero-size quad. All glyph coordinates are integers.

### 3.2 `fonts.json`

```
{
  "format_version": 1,
  "fonts": [{
    "id": String, "folder": String, "file_name": String, "ext": String,
    "char_set": String,                       // exactly as stored, index = glyph number
    "glyphs": [{
      "index": u32, "character": String, "byte": u8,
      "texture_id": String,                   // "<font>:FullSet:<index>"
      "atlas": String, "atlas_image": String, // e.g. "atlasTest0", "TextureAtlasTest0"
      "advance": f32,                         // BCTextureDef +0x28
      "height": f32,                          // BCTextureDef +0x2c (line advance)
      "quad_width": f32, "quad_height": f32,  // BCTextureDef +0x20, +0x24 (0 for space)
      "uvs": [f32; 8],                        // 4 × (u, v), vertex order of "vertices"
      "vertices": [f32; 12]                   // 4 × (x, y, z): top-left, top-right,
    }]                                        //   bottom-left, bottom-right; y up
  }]
}
```

Floats are the exact f32 values, written as the shortest decimal that round-trips. The same
values are in `textures.json` under the glyph texture ids; `fonts.json` is self-contained for
the text renderer. Every char set is ASCII without repeats (checked).

### 3.3 Dynamic texts

`BCLibrary::addDynamicText: @ 0x100023b60` / `getDynamicTextById: @ 0x100023b44`
(`mDynamicTextDictionary`, id → `BCText`). The only producer is the `Timelines.bin` loader
(176 text records, `timelines.md` §2.5); game code then fills them by id with `setText:`
(43 distinct literal ids at 75 call sites, e.g. `BattleAlertMessage`,
`BattleAlertMessageTapHere`, `PersonalBest`, `AchievementScore`, `CounterNumber`,
`IdolAreYouSureText`). Static texts are type-11 struct-XML nodes (`struct-xml.md` §6).

### 3.4 Keyboard text entry

`PocketGodViewController::init @ 0x100069ce8` creates a hidden `UITextField`
(`clearsOnBeginEditing`, no autocapitalisation, autocorrection off, return key "Done"),
observed with `UITextFieldTextDidChangeNotification`. The visible text is always a `BCText`.

- `onShowKeyboard:editingText:cancelEditDisplayObject:maxEditCharCount:preventEmpty: @
  0x10006d84c`: ignored while open; stores the listener, the `BCText`, the cancel button
  (made selectable and shown), the max length and saves the current text; `preventEmpty` →
  `enablesReturnKeyAutomatically`; shows the keyboard (`showKeyboard @ 0x10006d444`).
- Each edit (`textField:shouldChangeCharactersInRange:replacementString: @ 0x10006d4f0`) is
  accepted only if the resulting length ≤ max, and the `BCText` is updated immediately
  (`textDidChange: @ 0x10006d49c` mirrors the field too). `BCText setText:` trims leading and
  trailing whitespace and unmapped characters are simply not drawn.
- Return (`textFieldShouldReturn: @ 0x10006d6b8`): if the text's id is `PygmyName`,
  `PygmyDescription`, `TribeName`, `GodName` or `GodDescription` the active persona's
  `setName:`/`setDescription:` is called (no 1.48.2 caller opens the keyboard on those texts),
  then the field resigns. `textFieldDidEndEditing: @ 0x10006d5b8` hides the cancel button and
  calls `[listener onKeyboardTextEntered:text]`.
- Cancel (`onHideKeyboard @ 0x10006d994`): restores the saved text, hides the keyboard, no
  callback.

Callers: `StoryManager` (story editor; max 10, 30, 150 and 60 characters at
`0x10004e1cc`, `0x10004f550`, `0x10004f7b4`, `0x10004f990`) and `BonusPackManager` (unlock code,
max 30, `updateUnlockPanel @ 0x1000cb664`, `onKeyboardTextEntered: @ 0x1000cba6c`). The port
can implement this with egui text input or its own key handling, keeping the max-length rule
and the live mirroring into the `BCText`.

---

## 4. Gradients

`BCGradientDef = {int count; BCGradientColor colors[count]}`,
`BCGradientColor = {f32 pos, r, g, b, a}`. The only producer is `BCLibrary::parseStructXml:`
(node type 10 in root `0x10064a270`) → `BCGradient::createGradientDef: @ 0x100178500` →
`BCLibrary::addGradientDef:id: @ 0x10002384c` (`std::map`, `getGradientDefById: @ 0x100023774`
returns NULL and inserts it for an unknown id). Consumers: `BCGradient` display objects
(struct-XML type 9) and the eleven sky gradients that `PocketGodViewController::loadGame @
0x10006a030` fetches into `mpSunsetGradient` … `mpApocalypseCloudyGradient` (used by
`updateGradients @ 0x1001592b8`).
Parsing, geometry and fading are in `rendering.md` §7 and `struct-xml.md` §4.3.

`extract_engine_tables.py` parses each spec exactly like `createGradientDef:`: split on `,`,
each component `sscanf("%f:%x")` — the position is rounded correctly to f32 (as `strtof`), the
colour is a 32-bit hex ARGB with an optional `0x` — and each channel is
`f32(byte) / f32(255)` in single precision. `StartGradient` keeps its 7-digit colour
`0xf5FDEFE` (alpha 0x0f).

`gradients.json`:

```
{"format_version": 1,
 "gradients": [{"id": String, "spec": String,
                "stops": [{"position": f32, "argb": u32, "rgba": [f32; 4]}]}]}  // spec order
```

---

## 5. Other engine services

| Service | Address(es) | Port |
|---|---|---|
| Logging: `FUN_1001a8b20(file, line, function, tag, format, …)` appends a `BCLogItem` (`initWithLogTag:trace: @ 0x1001a89e8`) to a global array under a spin lock and prints `"%s: %s\n"`; `FUN_1001a8ccc` is a variant; `FUN_1001a8dec`/`FUN_1001a9010` flush the cache (by tag) | `0x1001a8b20`–`0x1001a9234` | `log` crate; no game behaviour depends on it |
| `BCLLDBProxy` (`logFlushCache:`, `logFlushCacheTags:`, `logClearCache:`, `logActiveDisplayGroup`, `logGameFrameListeners`, `logResourceLoaderQueue`) | `0x100013a14`–`0x100013aa8` | debugger helpers, not ported |
| `BCProfile` (hierarchical timing: `profileWithId:`, `enableProfiling`, `stop`, `logWithPad:`) — **no caller** outside the class | `0x100173a34`–`0x1001741c0` | not ported |
| `GameServer` (Game Center): `authenticateUser @ 0x10008b298`, `submitToLeaderboard:score: @ 0x10008b894` (`GKScore`, context 0, fire-and-forget), `submitAchievement: @ 0x10008b970` (100 % complete), turn-based match events `0x10008ae78`–`0x10008b134`; leaderboard ids come from `PocketGodViewController::getSacrificeLeaderboardId: @ 0x100072908`, `getGiftLeaderboardId: @ 0x1000735f0`, `getAchievementLeaderboardId: @ 0x100073ca0`, plus `pg.totalsacrifices` / `pg.totalgifts` | `0x10008a90c`–`0x10008ba50` | online only: stub (log) or an optional platform service; the local achievement state is the `System` document (2.6.2) |
| `BCKeyChain` | 2.3 | save-file section |
| `BCPreferences` network queue | 2.2.6 | optional |
| Vibration | 1.4.2 | rumble or no-op |

Timing, the game-frame listener list, touches and the render loop are in `rendering.md` and
`logic.md`; random numbers in `logic.md`.

---

## 6. Open questions

1. **Mono vs stereo loudness.** Mono buffers go through Apple OpenAL's 3D mixer at the
   listener position, stereo buffers bypass it. Whether a centred mono source came out at full
   amplitude on both channels or at −3 dB (equal-power pan) is not determinable from the
   binary; the 22 stereo files (Konkey Dong, The Runs, battle loop, a few effects) may play
   ~3 dB louder or quieter relative to the mono ones than in the original. Recommendation:
   play mono at full amplitude on both channels, check by ear against a device recording.
2. **Streaming fill thread.** The code that fills the 8-buffer ring of the RemoteIO player
   (1.5) was not identified. It is unreachable in 1.48.2 (no streamed sound), so it does not
   affect the port.
3. **Keychain secret type.** `kSecAttrGeneric` is stored from an `NSString` and read back with
   `objectForKey:` and compared with `isEqualToString:`. On iOS the attribute normally comes
   back as `NSData`; how the original survived that (or whether purchases and the Challenge
   document were effectively reset on each launch on some iOS versions) is unverified. Only
   relevant to importing iOS saves.
4. **`IdolsState`.** Read at load to set `StartIdolsOn`, never written in 1.48.2; presumably a
   leftover from an older version's saves. The port should read it (0 when absent) and never
   write it.
5. **Device behaviour of the silent switch** (default audio session category) and of audio
   while the app is backgrounded are platform behaviour, not game logic; nothing to port.
