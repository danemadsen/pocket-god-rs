# Pocket God v1.48.2 — reverse-engineering notes

This directory holds the reverse-engineering specs that drive the Rust/Bevy port.
Everything here is derived from the iOS release `com.boltcreative.pocketgod` 1.48.2
(the last IPA). The port must be complete: every island, power, episode, mini-game,
screen, sound and texture.

## Source material (all under `tmp/`, which is git-ignored)

| What | Path |
|---|---|
| App bundle (target version) | `tmp/pocket-god-v1.48.2/Payload/PocketGod.app/` |
| arm64 slice of the executable | `tmp/re/PocketGod.arm64` |
| Ghidra project (already analysed) | `tmp/re/ghidra_proj/PocketGod.gpr` |
| Decompiled C, one file per ObjC class / category | `tmp/re/decomp/game/<Class>.c`, `<Class>+<Category>.c` |
| Third-party SDK decompilation (ads/analytics, not ported) | `tmp/re/decomp/thirdparty/` |
| Index of groups (functions, bytes) | `tmp/re/decomp/INDEX.tsv` |
| Raw single-file decompilation + address index | `tmp/re/decomp_raw/all.c`, `tmp/re/decomp_raw/index.tsv` |
| ObjC metadata dump (`otool -oV`) | `tmp/re/objc_meta.txt` |
| Parsed ObjC classes (ivars with types, methods with IMP addresses, superclass) | `tmp/re/classes.json` |
| All strings in the binary | `tmp/re/bin_strings.txt` |
| All Ghidra symbols (`address \t name \t type`) — maps labels such as `cf_Foo`, `DAT_…`, `PTR_…` to addresses | `tmp/re/symbols.tsv` |
| `__DATA` segment after running the module initializers | `tmp/re/data_after_init.bin` |
| Older v1.45.3 bundle (has source XML for data later compiled into the binary) | `tmp/ipas/x1452/Payload/PocketGod.app/` (`sequence.xml`, `displaygroups.xml`, `texture.xml`, `texture_test.xml`, `texture_news.xml`, `sound.xml`) |

Each class file in `tmp/re/decomp/game/` starts with a comment listing the class's
ivars (with ObjC type encodings) and properties. Free C functions (`FUN_xxxxxxxx`) are
placed in the file of the class whose methods precede them in address order (same
compilation unit, usually), tagged `(free function near this class)`.

## Tools

Python with `unicorn` and `capstone` is available at `tmp/re/venv/bin/python`. Run it
with `-I`.

- `tools/re/macho_image.py` — `MachOImage(binary, data_dump)`: read u8/i32/u32/f32/u64,
  C strings, CFStrings and `id` values at virtual addresses, using the post-initializer
  `__DATA` image.
- `tools/re/resolve.py <addr>...` — decode addresses (CFString, C string, pointer).
  `--floats <addr> <n>`, `--u64 <addr> <n>`, `--hexfloat <0xbits>...`.
- `tools/re/emulate_init.py` — runs `__mod_init_func` initializers under Unicorn and
  dumps `__DATA` (already done → `tmp/re/data_after_init.bin`).
- Ghidra 12.1.4 headless: `JAVA_HOME=/Library/Java/JavaVirtualMachines/temurin-21.jdk/Contents/Home /opt/homebrew/opt/ghidra/libexec/support/analyzeHeadless tmp/re/ghidra_proj PocketGod -process PocketGod.arm64 -noanalysis -scriptPath <dir> -postScript <Script.java> <args>`.
  Only one process can open the project at a time — prefer grep over the decompiled
  files and `resolve.py` for data.

## Reading the decompiled code

- Methods are named `Class::selector_with_underscores`; `param_1` is `self`,
  `param_2` is `_cmd`. Categories are `objc::Categories::Class_Category_::sel`.
- Ivars appear as `*(T *)(self + (long)mIvarName)`; inherited ivars as
  `::BCLogic::mDisplayObject`.
- `_objc_msgSend(obj, "selector", args...)` is a message send. Ghidra sometimes turns
  sends into direct calls such as `cancelFlip(param_1, "cancelFlip")`.
- NSString literals are `&cf_Name` labels (the label is a sanitised form of the string;
  use `resolve.py` on the address for the exact text) or bare addresses such as
  `0x100386470` → `@"PiranhaInit"`.
- Float constants often appear as IEEE bit patterns (`0x3f800000` = 1.0,
  `0xc38a8f5c` = -277.12). Use `resolve.py --hexfloat`.
- Sequence-item handlers have selectors ending in `:::` with signature
  `int handler(id self, SEL _cmd, id sequence, BOOL firstTime, BCSequenceItemDef *def)`;
  `def->field2_0x10[i]` is the i-th positional parameter (an NSString or NULL).

## Architecture in one page

- ObjC app with a custom engine, "BC" (Bolt Creative): `BCView` (UIView hosting OpenGL
  ES), `BCLibrary` (singleton registry of all defs and the default sequence-item
  handlers), `BCDisplayObject` → `BCDisplayGroup`/`BCModel`/`BCText`/`BCRect`
  (+ `BCMultiModel`, `BCTileModel`, `BCGradient`, `BCDisplayMarker` and custom models
  such as `RainbowModel`, `LightningModel`, `LocustModel`, `HurricaneModel`,
  `WhirlPoolModel`, `PathModel`, `MagnifyingBeamModel`), `BCAnimation`/`BCAnimationDef`/
  `BCBinaryAnimationDef` (timeline animation), `BCSequence` (data-driven scripts),
  `BCLogic` → `SpriteLogic` → ~130 gameplay `*Logic` classes, `MultiModelLogic` →
  `BCScreenLogic` → screen logics, `BCButtonLogic`, `BCSound`, `BCPreferences`.
- `PocketGodViewController` (355 methods + 321 in category `Game`) is the hub: it owns
  the library, preferences, managers and logic objects, and is the listener (delegate)
  for most logic objects. `PygmyLogic` has ~650 methods across categories `Island`,
  `Apocalypse`, `Charlie`, `Underwater`, `BattleOfTheGods`, `KonkeyDong`, `NoPee`,
  `TheRuns`, `Story`, `OogaJump`, `MoronPests`.
- Data loading happens in `PocketGodViewController::loadGame` (0x10006a030):
  - `processAtlasTextureDefs:textureDefs:` with (`0x100602950`, `0x1005f9738`) and
    (`0x1007cbc08`, `0x1007c4988`)
  - `FUN_100038d68(@"TextureAtlas0", @"raw", 0x100751f38)` loads atlas pixel data;
    likewise `TextureAtlas1` (0x100767d00) and `WidescreenOverlayAtlas0`.
  - `processSequenceDefs:` with the NULL-terminated `BCSequenceDef*` list at
    `0x100643a60` (2548 sequences, 7258 items).
  - `parseStructXml:` with roots `0x10071ab98`, `0x10064a270`, `0x1006fe1d0`,
    `0x1007162f0`, `0x1008162c0`.
  - `parseBinAnim:ext:` with `Timelines.bin`.
  - `BCPreferences` documents `Persona`, `Options`, `System`, `Challenge`.
  - `BCLibrary(Logic)::createLogicClasses` (generated) builds the global logic objects
    (indexed by `getLogic:<int>`); `initLogicListeners` wires their listeners.
- Struct encodings (from ObjC type strings):
  - `BCSequenceDef = {id name; int count; BCSequenceItemDef **items}`
  - `BCSequenceItemDef = {id methodName; SEL selector; id *params}` (params positional)
  - `BCAtlasTextureDef = {@@@@@ iii BBB ff I BCTextureDef}`
  - `BCTextureDef = {id; BOOL; BCAtlasTextureDef*; uint*; ffff; float[8]; float[12]}`
  - `BinaryAnimationDef = {char*; q q q q; f f; char*; q; BinaryFrameDef*}`
  - `BinaryFrameDef = {q; char*; char*; f f; q; BinaryFrameStatePointer*}`
  - `BCStructXmlDisplayObjectNodeDef = {i @@ f×15 I f×4 BB f×6}`
  - `BCGradientDef = {int count; BCGradientColor colors[] = {f pos, r, g, b, a}}`
  - `BCSkinPackDef`, `BCSkinTextureBitsDef`, `BonusPackDef`, `BCNewsDef`, `BCStoryDef`,
    `BCStructXmlEpisodeNodeDef`, `BCStructXmlEpisodeChallengeNodeDef`,
    `BCStructXmlDanceDanceBeatNodeDef`, `BCStructXmlLinkNodeDef`, ...
- Textures: `.raw` files are headerless pixel data. Atlases are mostly RGBA4444
  (u16 little-endian, R in the high nibble); `TextureAtlasPvr*` is PVRTC 4bpp
  1024×1024. Dimensions and formats come from the atlas/texture defs.
- Sounds: `sound/*.caf`, all linear PCM s16 (22 kHz or 44.1 kHz, mono or stereo).
- `Timelines.bin`: u64 string-table size, string table (NUL-separated, 10532 strings),
  then 64-bit-field records (the v1.45.3 file is the same layout with 32-bit fields).
- `story.xml`, `news.xml`: plain XML, loaded at runtime.

## Port layout

- `crates/` — Rust workspace (Bevy 0.20, bevy_egui 0.43, Rust 1.99, edition 2024).
- `assets/` — converted assets, embedded into the binary at build time.
- `tools/re/` — the scripts that produce `assets/` from the IPA.
- `docs/re/` — these notes.
