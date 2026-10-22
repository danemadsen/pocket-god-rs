# Timelines.bin and timeline animation playback

`Timelines.bin` (app bundle root) contains every timeline animation in Pocket God: 2,354
Flash-exported animations, the 176 dynamic text objects they display, and the per-frame state
of every object in every frame. This document specifies the byte layout exactly and the
runtime semantics of `BCBinaryAnimationDef`, `BCAnimation`, the animation listener
(`BCLogic`), and how display objects (`BCMultiModel`, `BCTileModel`, `BCModel`, `BCText`,
`BCDisplayGroup`) apply frames, so that the Rust port can parse and play the file without
reading the decompiled code.

Reference tooling: `tools/re/timelines.py` (run from the repo root with
`tmp/re/venv/bin/python -I`):

| Command | Output |
|---|---|
| `timelines.py validate` | structural checks, every byte accounted for, plus texture-id, vertex-fit and alias checks against the matching build (exit 1 on failure) |
| `timelines.py summary` | JSON summary (sections, histograms, sounds, actions, aliases, vertex fit, anomalies) |
| `timelines.py list` | animation names in file order |
| `timelines.py anim NAME...` | JSON of the named animations plus the states and texts they use |
| `timelines.py texts` | JSON of the dynamic text records |
| `timelines.py dump` | JSON of the whole file (34 MB, compact) |

`--file PATH` selects another file (default: the v1.48.2 file); the word size (8 or 4 bytes) is
detected automatically. For the 8-byte file the texture defs and the skin-pack animation
aliases are read from the arm64 binary (`--binary`, `--data-dump`); for the 4-byte file the
aliases come from v1.45.3's `texture_test.xml` next to the file (`--alias-xml`). Both shipped
files validate with 0 failures. The warnings are the explained anomalies of section 2.10:
irregular frame indices, NaN colours and (v1.48.2 only, the check needs the binary) 24
rescaled-def quads.

Contents

1. Loading context and counts
2. File format
3. Runtime objects
4. Playback semantics
5. How display objects apply frames
6. Notes for the Rust port
7. JSON schema (tool output)
8. Open questions

---

## 1. Loading context and counts

`PocketGodViewController::loadGame` (0x10006a030) loads, in this order: texture atlases and
`BCTextureDef`s (`processAtlasTextureDefs:textureDefs:`), sequences, the struct-XML roots
(sounds, display groups, **fonts**, …) and finally
`BCLibrary::parseBinAnim:ext:` (0x100025a08) with `@"Timelines"`, `@"bin"`. The order matters:
the timeline loader looks up fonts (`getFontDefById:`) and textures (`getTextureDefById:`) that
must already exist.

Counts measured by `timelines.py summary`:

| | v1.48.2 (port target) | v1.45.3 (cross-check) |
|---|---|---|
| file size | 17,217,304 | 9,564,340 |
| word size (W) | 8 (arm64 build) | 4 (armv7 build) |
| string table | 203,696 bytes, 10,531 strings (10,528 referenced + 3 NUL padding) | 170,248 bytes, 8,934 strings (1 padding) |
| animations | 2,354 (568 looping, 56 move-enabled, 168 with a next animation) | 1,824 |
| skin-pack animation aliases (3.2) | 15 names, 245 (alias, target) pairs | 15 names, 195 pairs |
| dynamic texts | 176 | 170 |
| frame records | 56,175 (sum of lengths: 56,165) | 42,558 |
| state pointers | 304,286 | 215,714 |
| frame states (unique) | 81,294 (28,717 used by more than one frame) | 62,614 |
| distinct frame sounds / frames with a sound | 290 / 1,526 | 257 / 1,189 |
| distinct frame actions / frames with an action | 90 / 172 | 68 / 111 |
| max states in one frame | 70 | |

fps histogram (v1.48.2): 12 → 882, 15 → 15, 18 → 10, 20 → 5, 22 → 1, 24 → 1064, 30 → 279,
35 → 7, 36 → 21, 60 → 70.

Every v1.45.3 animation name also exists in v1.48.2 (530 added, none removed). Of the
1,824 shared animations, 1,450 are identical in content and 314 differ only in baked
vertices (re-cut textures). The other 60 differ in other fields as well.

---

## 2. File format

### 2.1 Conventions

- Little-endian throughout.
- **W** = word size: 8 in v1.48.2, 4 in v1.45.3. "word" = unsigned W-byte integer, "sword" =
  signed W-byte integer (`long long`/`q` in 64-bit, `int` in 32-bit). Floats are IEEE-754
  float32 in both versions.
- **String reference**: a word holding a byte offset from the *start of the string data*
  (file offset W). The value `0xFFFFFFFF` (also in 64-bit files, compared as the full
  64-bit value `0x00000000FFFFFFFF`) means "no string" (NULL). Every non-null reference
  points at the first byte of a string (validated).
- **Record offsets** (frames, pointer arrays, states) are byte offsets from the start of the
  *data* of the target section (the first byte after its size word).
- All strings are 7-bit ASCII. The only control character is `\n`, in 5 dynamic-text
  strings: `IdolResultsExp1`–`3` (`"Used: x4\nExp: +4"`, `"Used: x2\nExp: +2"`,
  `"Used: x1\nExp: +1"`), `PygmyThemeAudioNote` (`"INCLUDES \nPYGMY THEME\nBY PARRY GRIPP"`)
  and `UpdateAudioNote` (`"INCLUDES THE\nUPDATE SONG!"`). (v1.45.3 also has a `\t`, in its
  text `WhatsnewLine5`, which v1.48.2 no longer has.) The runtime converts strings with
  `NSASCIIStringEncoding`.

### 2.2 Top-level layout

The file is six sections back to back. Each section is a size word followed by exactly that
many bytes. Nothing precedes, separates or follows them.

| # | Section | Size word at (v1.48.2) | Data at | Size | Content |
|---|---|---|---|---|---|
| 1 | strings | 0x000000 | 0x000008 | 203,696 | NUL-terminated strings |
| 2 | animations | 0x031BB8 | 0x031BC0 | 169,496 | count word N = 2,354, then N × 72-byte animation records |
| 3 | texts | 0x05B1D8 | 0x05B1E0 | 7,040 | 176 × 40-byte text records |
| 4 | frames | 0x05CD60 | 0x05CD68 | 2,696,400 | 56,175 × 48-byte frame records |
| 5 | state_pointers | 0x2EF238 | 0x2EF240 | 2,434,288 | 304,286 words |
| 6 | states | 0x541730 | 0x541738 | 11,706,336 | 81,294 × 144-byte frame-state records |

End of section 6 = 0x106B718 = 17,217,304 = file size.

v1.45.3 (W = 4): size words at 0x0, 0x2990C, 0x3B614, 0x3C360, 0x15F22C, 0x231CB8; record
sizes 40 / 20 / 28 / 4 / 116; same order and meaning.

Byte accounting (v1.48.2): 7 header words (6 size words + the animation count) = 56 bytes,
string table 203,696, records 17,013,552; total 17,217,304. Every record of sections 2–6 is
referenced: each frame record by exactly one animation, each pointer slot by exactly one
frame, each state by at least one pointer (validated).

How `parseBinAnim:ext:` walks it (0x100025a08): `S = word[0]`;
`T = word[S+W]`; `N = word[S+2W]`; `U = word[S+W+W+T]`; and so on, each section starting
right after the previous one. The text count is `U / 40` (unsigned divide); the animation
count is the stored N.

### 2.3 String table (section 1)

Concatenated NUL-terminated strings. The table is padded with NUL bytes so that its size is a
multiple of 4 (v1.48.2: 3 padding bytes, v1.45.3: 1; both sizes also happen to be multiples
of 8). Padding bytes parse as empty strings that nothing references. The table is
deduplicated: every distinct text occurs exactly once (apart from the padding), so records
with the same text share one offset (e.g. a state name, a texture id and a dynamic-text id
that are the same word all point at one string). The empty string is never referenced; absent
strings use the NULL sentinel instead.

### 2.4 Animation record — `BinaryAnimationDef` (section 2)

ObjC type: `{BinaryAnimationDef=*qqqqff*q^{BinaryFrameDef}}`. Records follow the count word.

| Off (W=8) | Off (W=4) | Type | Field | Meaning |
|---|---|---|---|---|
| 0x00 | 0x00 | string ref → `char*` | `name` | animation id (unique) |
| 0x08 | 0x04 | sword | `fps` | frames per second (`BCBinaryAnimationDef::fps` reads the low 32 bits as `int`) |
| 0x10 | 0x08 | sword | `length` | number of frames (`length` reads the low 32 bits) |
| 0x18 | 0x0C | sword | `loop` | 0/1; `loop` returns `value != 0` |
| 0x20 | 0x10 | sword | `move_enabled` | 0/1; `isMoveEnabled` returns `value != 0`, `applyMovement` requires `value == 1` |
| 0x28 | 0x14 | f32 | `move_x` | displacement per full cycle (section 4.6) |
| 0x2C | 0x18 | f32 | `move_y` | |
| 0x30 | 0x1C | string ref → `char*` | `next_animation` | played automatically when a non-looping animation completes; NULL = none |
| 0x38 | 0x20 | sword | `frame_count` | number of frame records |
| 0x40 | 0x24 | word → `BinaryFrameDef*` | `frames_offset` | byte offset of the first frame record in section 4 (multiple of 48) |

Size 72 (W=8) / 40 (W=4). In the shipped files `loop` and `move_enabled` are only 0 or 1,
`fps > 0` whenever `length > 0`, and `move_x/move_y` are 0 unless `move_enabled`.
Animations' frame ranges are contiguous and in file order.

### 2.5 Text record (section 3) — dynamic text objects

No ObjC type name; layout from the loader (0x100025a08):

| Off (W=8) | Off (W=4) | Type | Field | Meaning |
|---|---|---|---|---|
| 0x00 | 0x00 | string ref | `id` | dynamic-text id (unique); also the string that states reference |
| 0x08 | 0x04 | string ref | `text` | initial text (`BCText setText:`); game code may replace it at runtime (2.9) |
| 0x10 | 0x08 | string ref | `font` | `BCFontDef` id: `AnimatedGothicLite20` (82), `Arial12` (84), `Arial12Black` (1), `Doop24Green` (9) |
| 0x18 | 0x0C | word | `font_def` slot | 0 in the file; the loader stores the `BCFontDef*` here |
| 0x20 | 0x10 | sword | `justification` | low 32 bits used: 0 = left (58), 1 = right (9), 2 = centre (109) |

Size 40 / 20. Justification meaning from `BCFontDef::fillFontCharTextureArray:textBuffer:fontJustification:`
(0x100066c08): each line's pen x starts at 0 (0), at `-(float)(int)lineWidth` (1) or at
`-(float)(int)(lineWidth * 0.5)` (2), where `lineWidth` sums the glyph advances up to the
next `\n` (font rendering itself is specified with `BCFontDef`).

### 2.6 Frame record — `BinaryFrameDef` (section 4)

ObjC type: `{BinaryFrameDef=q**ffq^{BinaryFrameStatePointer}}`.

| Off (W=8) | Off (W=4) | Type | Field | Meaning |
|---|---|---|---|---|
| 0x00 | 0x00 | sword | `index` | frame number this record shows (compared as its low 32 bits) |
| 0x08 | 0x04 | string ref → `char*` | `sound` | `BCSound` id played when the frame is reached; NULL = none |
| 0x10 | 0x08 | string ref → `char*` | `action` | selector name sent to the animation listener; NULL = none |
| 0x18 | 0x0C | f32 | `move_x` | cumulative displacement within the cycle at this frame |
| 0x1C | 0x10 | f32 | `move_y` | |
| 0x20 | 0x14 | sword | `state_count` | number of states (= draw-order list) |
| 0x28 | 0x18 | word → `BinaryFrameStatePointer*` | `states_offset` | byte offset of this frame's pointer list in section 5 (multiple of W) |

Size 48 / 28. Frames are stored one per frame index: in 2,343 of 2,354 animations the indices
are exactly `0..length-1` in order. The 11 exceptions (all reproduced exactly by the algorithm
in section 4.4, which never assumes regularity):

| Animation | length | stored indices |
|---|---|---|
| PygmyGainBalanceAnim | 6 | 0–4 (frame 5 shows record 4 again and re-fires it, 4.4) |
| PygmyGrab | 5 | 0–6 (records 5, 6 never shown) |
| PygmyHeatDownAnim | 3 | 0–4 |
| PygmyIdleFartAnim | 19 | 0–19 |
| PygmyIdleLookdownAnim | 14 | 0–14 |
| PygmyIdleLookupAnim | 10 | 0–10 |
| PygmyIdleNosePickAnim | 23 | 0–23 |
| PygmyIdleScratchAnim | 13 | 0–13 |
| PygmyIdleWaveAnim | 15 | 0–15 |
| PygmyIdleYawnAnim | 21 | 0–21 |
| PygmyWindBlown2Anim | 4 | 0, 1, 2, 2 (duplicate; the second record with index 2 wins, and frame 3 shows it again and re-fires it) |

`NullAnim` has `length = 0` and no frames; `BlankAnim` has one frame with no states.

### 2.7 State pointer lists — `BinaryFrameStatePointer` (section 5)

An array of words. Frame `f` owns the `state_count` consecutive words starting at
`states_offset`. Each word is the byte offset of a state record in section 6 (a multiple of
144 / 116); the loader replaces it with the absolute address. The order of a frame's list is
the **draw order** (first = bottom): it is the Flash layer stacking order. States are shared:
an object that does not change between frames points at the same state record.

### 2.8 Frame state record (section 6)

| Off (W=8) | Off (W=4) | Type | Field | Meaning |
|---|---|---|---|---|
| 0x00 | 0x00 | string ref → `char*` | `name` | Flash instance / layer name (`Root`, `Layer 1`, `ChannelSword`, …); only printed by the loader's debug log |
| 0x08 | 0x04 | word | `flags` | see the flag table; bit 31 is set by the loader once the record is fixed up |
| 0x10 | 0x08 | string ref → `char*` | `texture` | texture id (flag 0x2000) or dynamic-text id (flag 0x4000); NULL otherwise |
| 0x18 | 0x0C | word | `texture_def` slot | 0 in file; loader stores `BCTextureDef*` (flag 0x2000) |
| 0x20 | 0x10 | sword | `channel` | tweenable channel 0–29; 0 = no channel |
| 0x28 | 0x14 | word | `dynamic_text` slot | 0 in file; loader stores the `BCText*` (flag 0x4000) |
| 0x30 | 0x18 | f32 | `x` | registration point x, in the owning model's local space (see "Coordinate space" below) |
| 0x34 | 0x1C | f32 | `y` | registration point y |
| 0x38 | 0x20 | f32 | `rotation` | degrees, counter-clockwise positive with y up (GL `glRotatef(r,0,0,1)`) |
| 0x3C | 0x24 | f32 | `skew_x` | degrees; never read at runtime |
| 0x40 | 0x28 | f32 | `skew_y` | degrees; never read at runtime |
| 0x44 | 0x2C | f32 | `scale_x` | |
| 0x48 | 0x30 | f32 | `scale_y` | |
| 0x4C | 0x34 | f32 | `alpha` | colour multipliers (not clamped in the data: 9 states have alpha < 0) |
| 0x50 | 0x38 | f32 | `red` | |
| 0x54 | 0x3C | f32 | `green` | |
| 0x58 | 0x40 | f32 | `blue` | |
| 0x5C | 0x44 | f32 × 12 | `vertices` | quad corners v0..v3, each (x, y, z), in the owning model's local space, z always 0 |
| 0x8C | — | 4 bytes | padding | W=8 only (struct alignment); always a copy of the `blue` bytes; ignore |

Size 144 / 116. The loader's debug log prints `sizeof` values 40, 4, 144, 48, 72, 16
(`@"******************* CHECK ANIM TEXT: %d %d %d %d %d %d"`, 0x10038a150), confirming the
text, state, frame, animation and `TweenableChannel` (16) sizes.

Flags (counts are v1.48.2 states):

| Bit | Mask | Name | Count | Used at runtime by |
|---|---|---|---|---|
| 0 | 0x0001 | x | 80,689 | channel/sub-object transform (sections 4.4, 5.2) |
| 1 | 0x0002 | y | 80,690 | same |
| 5 | 0x0020 | rotation | 78,306 | same |
| 6 | 0x0040 | skew_x | 2,321 | nothing (exporter: matrix had skew) |
| 7 | 0x0080 | skew_y | 2,321 | nothing |
| 8 | 0x0100 | scale_x | 80,666 | channel transform; also selects the `180 - r` rotation rule for mirrored parents |
| 9 | 0x0200 | scale_y | 80,674 | channel transform |
| 11 | 0x0800 | alpha | 10,461 | nothing (exporter: set iff alpha ≠ 1) |
| 12 | 0x1000 | color | 2,812 | nothing (exporter: set iff (r,g,b) ≠ (1,1,1)) |
| 13 | 0x2000 | texture | 78,556 | loader: resolve `texture` with `getTextureDefById:` |
| 14 | 0x4000 | dynamic_text | 508 | loader: resolve `texture` with `getDynamicTextById:` |
| 31 | 0x80000000 | (loaded) | 0 | loader's "already fixed up" marker |

No other bits occur. Bits 13 and 14 never occur together; `texture` is non-NULL exactly when
one of them is set. Alpha and colour multipliers are always applied regardless of bits 11/12.

Exporter invariants (hold for every state; the runtime does not rely on them): skew bits come
in pairs and never with the rotation bit; vertices are all zero iff the state has no texture;
vertex z is 0. For textured states the vertices are the texture's own quad
(`BCTextureDef.vertices`, +0x50) transformed by the state:

```
if flags & SKEW: a = sx·cos(ky), b = sx·sin(ky), c = −sy·sin(kx), d = sy·cos(kx)
else:            a = sx·cos(r),  b = sx·sin(r),  c = −sy·sin(r),  d = sy·cos(r)
v_i = (x + a·qx_i + c·qy_i,  y + b·qx_i + d·qy_i,  0)        (angles in degrees)
```

This reproduces 78,532 of the 78,556 textured states to within 0.01 (`timelines.py validate`,
check `vertex fit`; `summary` → `vertex_fit`). The other 24 use deliberately rescaled texture
defs (textures.md §4): 18 `Bat*` states (5659–5674, 5697, 5700; defs drawn at 50 %) and 6
`ElectricGlow_01`…`06` states (18075–18080; defs drawn at 4×). In the binary those defs' quads
have the rescaled size but the unscaled trim offset, while the exporter scaled both: the
stored vertices are the formula applied to the def quad translated by
`((s_x − 1)·l, (s_y − 1)·b)`, where `(l, b)` is the quad's left/bottom corner and
`s = quad size / texel rect size (+0x20, +0x24)` per axis. The differences reach 13.5 px for
the bats and 30 px for the glows, and all 24 are explained this way. The runtime **never
recomputes vertices**: always draw the stored ones.

Coordinate space: `x`, `y`, `rotation` and the vertices are in the owning multimodel's
**local** space, the space its own transform chain sets up before the states are drawn
(`translate(pos)` … `translate(originPos)`, 5.2). Units are points, y up. For a model at
`pos = (0, 0, −277.12)` (depth factor `k = 1`) with unit scale, no rotation and a zero
`originPos`, this is the 480 × 320 landscape screen. Local coordinates are
converted to the model's parent space only by the channel code of 4.4, which uses
`parent = (local + originPos) · scale + pos` per axis and ignores rotation, pivot, skew and
depth. 5.2 applies the inverse. Texture quads put v0 top-left, v1 top-right, v2 bottom-left
and v3 bottom-right (larger y is higher). The strip is drawn with indices 0, 1, 2, 3.

Channel usage (states with `channel != 0`, by channel → most common state names): 1 →
`Pygmy` (5,079), 2 → `ChannelFemalePygmy`, `ChannelPygmyHead`, `Pygmy2`, 3 →
`ChannelNoPeeBubbles`, `ChannelBeeWings`, 7 → `ChannelSword` (174), 8 → `ChannelShield` (163),
… up to channel 23. `timelines.py summary` lists all names per channel
(`channel_state_names`). Which object occupies a channel is decided by game code
(`setTweenable:channelId:`, section 3.4).

### 2.9 Load-time processing — `BCLibrary::parseBinAnim:ext:` (0x100025a08)

1. Resolve the bundle path, `fopen`, `fseek/ftell` the size, `malloc` a buffer, `fread` the
   whole file. The buffer is kept for the program's lifetime
   (`setBinaryAnimationBuffer:` 0x1000233f4) because every runtime structure points into it.
2. Compute the section addresses (2.2). `NSLog` the struct sizes.
3. For each text record, in file order (count = `U / 40`):
   - `id` and `text` strings → C pointers (NULL stays NULL) and `NSString`s;
   - `font` → C pointer, `fontDef = [library getFontDefById:font]`, stored in the `font_def`
     slot;
   - `text = [[BCText alloc] initWithId:id fontDef:fontDef]`;
     `[text setFontJustification:(int)justification]`; `[text setText:textString]`;
     `[library addDynamicText:text]` (dictionary keyed by `id`; the object is never released
     and is not added to any display group).
4. For each animation record `i` in `0..N`:
   - `name` and `next_animation` → C pointers (NULL for 0xFFFFFFFF);
   - if `name == "OpenIdolScreen"` the loader additionally logs every frame and state (debug
     only, no effect);
   - `frames` pointer = frames data + `frames_offset`;
   - for each of its `frame_count` frames: `sound`, `action` → C pointers; pointer-list
     address = pointer data + `states_offset`; for each of its `state_count` pointer slots:
     `state = states data + slot`, store the absolute pointer back into the slot; then if
     `(int32)state.flags >= 0` (bit 31 clear — not processed yet):
     - set bit 31;
     - `name` → C pointer;
     - if `texture == 0xFFFFFFFF`: store 0 (NULL) in the `texture` field and stop;
     - else `texture` → C pointer, and if flag 0x2000: `texture_def = getTextureDefById(texture)`
       (stored at +0x18); else if flag 0x4000: `dynamic_text = getDynamicTextById(texture)`
       (stored at +0x28);
   - `def = [[BCBinaryAnimationDef alloc] initWithBinaryAnimationDef:record]`;
     `[library addAnimationDef:def]` (keyed by `[def id]`, i.e. the name); release.
5. `fclose`.

Lookups are case-sensitive `std::map`/`NSDictionary` lookups; a missing id yields NULL/nil
(`getTextureDefById:` inserts a NULL entry via `operator[]`). In v1.48.2 every texture id
resolves to one of the 8,337 registered `BCTextureDef`s and every dynamic-text id to a text
record (validated by `timelines.py validate`).

**Dynamic texts are shared, mutable objects.** There is one `BCText` per text id. Every frame
state that names the id draws that same object (5.2), and game code changes it at runtime.
Eight classes fetch dynamic texts with `BCLibrary getDynamicTextById:` (0x100023b44):

- `BattleAlertLogic`, `ChallengeManager`, `IdolIconLogic`, `IdolScreenLogic`,
  `IdolSelectionIconLogic`, `MoronPestsQuestionScreenLogic`, `PygmyLogic(Story)` and
  `StoryManager`;
- they send `setText:`, `setWrapWidth:`, `setLineCount:` and the C-buffer pair
  `useTextBuffer`/`showTextBuffer`;
- examples: `IdolScreenLogic` fills the episode and level labels, `IdolIconLogic` writes
  `"Episode %@"` labels, `StoryManager` sets wrapped story text with `setText:` and
  `setWrapWidth:`, and `ChallengeManager` writes through the text buffer.

`BCText::drawInMultiModelOpenGL:` (0x1001a7d70) re-runs `updateDisplayText` on every draw, so a
change shows the next time a frame that references the id is drawn. The text record gives only
the initial text, font and justification; `setText:` trims leading and trailing whitespace. The
`BCText` fields and setters are in rendering.md §5.7 and the layout and wrapping in
rendering.md §6.

### 2.10 Data anomalies and names that are not records (all explained)

- **Irregular frame indices**: 11 animations (table in 2.6).
- **A next animation that is an alias**: `PygmyGoGoCageDropAnim → PygmyDanceAnim`.
  `PygmyDanceAnim` is not a record. It is the skin-pack alias that start-up always installs
  (3.2): the currently selected pygmy dance, `PygmyDanceDefaultAnim` by default. The chain
  is how a normal pygmy that falls into the GoGo cage starts dancing: the sequence
  `PygmyFallIntoGoGoCageNormal` is `[triggerEpisodeEvent, setAnimation PygmyGoGoCageDropAnim,
  checkDanceChange]`. `PygmyLogic::isSoundDisabled` (0x100128940) returns `mbInGoGoCage`,
  which mutes the caged dancers' frame sounds. The other 167 next animations name records.
  `nextAnimationId` must be resolved by name, through the alias-aware dictionary, when the
  animation completes (4.3). Never resolve it to a record at load time.
- **Animation ids that are neither records nor aliases**: these play as an empty animation
  (4.2). The player completes and stops at once, and the model keeps its previous frame. The
  port must treat an unknown id this way, never as an error. The ids come from the
  animation-naming parameters of the compiled sequence items in `assets/data/sequences.json`
  (`setAnimation`, `setLayerAnimation`, `chooseAnimation`, `chooseRainbowAnimation`,
  `setAnimationBasedOn*`):

  | Id | Played by | Reachable? |
  |---|---|---|
  | `SkinCandylandStatueAnim`, `SkinSuperbowlStatueAnim`, `SkinMondoStatueAnim`, `SkinMonsterStatueAnim`, `SkinGrimmStatueAnim`, `SkinAdultSwimStatueAnim` | `setAnimation` in the sequences `SkinPack{Candyland,Superbowl,Mondo,Monster,Grimm,AdultSwim}StatueIdle` | no caller found in code, skin packs, display groups or logic objects (the skin packs use the `…SandStatueIdle` sequences) |
  | `FailBoatInitAnim` | `setAnimation` in `FailBoatInit` | no caller found |
  | `ShouldntHappen` | the `baby` and `oldFart` values of 10 `setAnimationBasedOnPygmyType` items in 8 sequences (`PygmyTiltTip{Forward,Backward}`, `PygmyTiltSlide{Forward,Backward}` (2 items each), `PygmyGainBalanceAfterTilt`, `PygmyPickup{Forward,Backward}`, `PygmyTiltUpsideDownToFallForward`) | only if a baby or old pygmy reaches these sequences (placeholder) |
  | `InfoScreenTribe`, `PygmyIdleAnim1`, `PygmyIdleAnim2` | `setLayerAnimation` / `chooseAnimation` in `Garbage` | no: `Garbage` is never run (sequences.md) |

  All 269 literal `playAnimationId:` arguments in the game code name records, and the 15 alias
  names (3.2) are only played from sequence data.
- **NaN colours**: states 11388, 70085, 75878 (the single frames of the single-frame animations
  `ButtonNewMatch`, `Statue1`, `UIHeader`) have quiet NaN (0x7FC00000) in alpha, red, green and
  blue. Nothing references these animation ids: not the code (literals and format strings),
  the compiled sequences, the skin-pack alias lists, the display groups or the logic objects.
  The names are also texture ids, so they appear unused. The JSON output writes these four
  floats as `null`.
- **Unresolvable frame sounds**: 10 sound ids used by frames have no `BCSound` definition, so
  `getSoundById:` returns nil and the frame is silent in the original:
  `BitGulp` (SharkCatchBabyAnim@0), `Bubble1` (BattleNegativeRealmEffectResultAnim@6),
  `HitGround1` (PygmyFastLandAnim@0), `Holding It` (JellyEatPygmyAnim@4; the sound is
  `HoldingIt`), `LowSlap` (PygmyDanceKonyAnim@16,20,24,101,117,131), `Oh`
  (IceHolePygmyFishCatchAnim@2, IglooPygmyBlastUpAnim@10, PygmyPulledUnderByGhostAnim@0),
  `Slide` (PygmyLavaSkidAnim@4; the sound is `slide`), `Sqawk` (DodoCatchFishAnim@3),
  `TRexRoar2` (IceMonsterRechargeHitAnim@2, IceMonsterRechargeHitDizzyInitAnim@0,
  TRexPokedAnim@1), `campfire` (WoodFirePokeAnim@6; the sound is `Campfire`). No sound alias
  creates them (`setSoundAlias:sound:` is only used for `BackgroundLoop`).
- **Baked quads of rescaled defs**: 24 states (18 `Bat*`, 6 `ElectricGlow_*`) whose vertices
  differ from the def quad by a translation (2.8). Draw the stored vertices.

### 2.11 v1.45.3 cross-check

The 32-bit file has the identical structure with W = 4: the same six sections, the same field
order, 0xFFFFFFFF as NULL, the same flag bits and invariants, no state padding (116 is already
4-aligned), the same 11 irregular animations, the same three NaN-colour states and the same
next animation through the `PygmyDanceAnim` alias. v1.45.3 still has the source of the
alias lists, `texture_test.xml` (`<animSkinItem skinId="PygmyDanceAnim"
id="PygmyDanceDefaultAnim" …>`). It defines the same 15 alias names, 195 (alias, target)
pairs, every target a record. Field semantics were cross-checked by parsing both files with
the same generic parser (`Layout` in `timelines.py`).

---

## 3. Runtime objects

### 3.1 `BCAnimationDef` / `BCBinaryAnimationDef`

`BCAnimationDef` (ivars `mId, mNextAnimationId, mFps, mLength, mbLoop, mbMoveEnabled,
mMoveXPos, mMoveYPos`) is an abstract base whose `applyFrame…` (0x1001a09c0) and
`applyMovement…` (0x1001a09c4) do nothing; nothing in the game instantiates it directly.
`BCBinaryAnimationDef` (ivar `mpAnimationDef`, a pointer to the record) overrides every
accessor to read the record:

| Method | Addr | Behaviour |
|---|---|---|
| `initWithBinaryAnimationDef:` | 0x1001a0a44 | `[super init]`, store record pointer |
| `id` | 0x1001a0a9c | new `NSString` from `name` (ASCII) |
| `fps` | 0x1001a0acc | `(int)` low 32 bits of `fps` |
| `length` | 0x1001a0ae0 | `(int)` low 32 bits of `length` |
| `loop` | 0x1001a0af8 | `loop != 0` |
| `isMoveEnabled` | 0x1001a0b18 | `move_enabled != 0` |
| `moveXPos` / `moveYPos` | 0x1001a0b38 / 0x1001a0b50 | record floats |
| `nextAnimationId` | 0x1001a0b68 | nil if NULL, else new `NSString` |
| `setLoop:`, `setMoveEnabled:`, `setMoveXPos:`, `setMoveYPos:`, `setNextAnimationId:`, `addFrameDef:` | 0x1001a0af4 … 0x1001a0bb0 | no-ops (the data is immutable) |
| `applySoundAndAction:animationListener:` | 0x1001a0bb4 | section 4.5 |
| `applyFrame:lastFrame:animation:animationListener:tweenableChannelArray:` | 0x1001a0ca4 | section 4.4 |
| `applyMovement:animationListener:` | 0x1001a109c | section 4.6 |

### 3.2 Library registry (`BCLibrary`)

| Method | Addr | Behaviour |
|---|---|---|
| `addAnimationDef:` | 0x100023ccc | `mAnimationDefDictionary[def.id] = def` |
| `getAnimationDefById:` | 0x100023cb0 | dictionary lookup; nil if unknown |
| `setAnimationDefAlias:animationDef:` | 0x100023d20 | if `def != nil`: `mAnimationDefDictionary[alias] = def` — the alias **replaces** any existing entry with that name (including a real animation); nil defs are ignored |
| `getDynamicTextById:` / `addDynamicText:` | 0x100023b44 / 0x100023b60 | `mDynamicTextDictionary` |
| `getSoundById:` | 0x100023d50 | `mSoundDictionary` lookup |

#### Animation aliases (skin packs)

Aliases are installed by skin-pack option values (textures.md §7). `FUN_100038f24` applies a
`BCSkinOptionValueDef`; its step 4 calls `FUN_100038dc0` (0x100038dc0) on the value's
NULL-terminated list (+0x50) of 5-pointer entries
`{alias, animationId, displayObjectId, behaviorSequenceId, displayGroupId}`. For each entry:

1. `setAnimationDefAlias:alias animationDef:getAnimationDefById(animationId)`. The target is
   looked up in the alias-aware dictionary at that moment; every shipped target is a record.
2. If `displayGroupId != @""`:
   `[[[getDisplayGroupById(displayGroupId) getDisplayObject:displayObjectId] logic]
   setBehavior:behaviorSequenceId]`. A nil group, object or logic makes this a no-op, as at
   start-up before `createLogicClasses`.

A player that already holds a def keeps it. Only later `playAnimationId:` calls and
`nextAnimationId` chains see the alias. The behaviour set in step 2 is how the affected
object replays the new target. Aliases are never removed; applying another value re-points
them.

The 15 alias names (`timelines.py summary` → `animation_aliases`; v1.45.3 has the same
names):

| Alias | Targets | Default target (value 0) | Option (object), packs → installed | Behaviour set (object → sequence) | Played by |
|---|---|---|---|---|---|
| `PygmyDanceAnim` | 22 dances | `PygmyDanceDefaultAnim` | Pygmy Dance (Pygmies), packs Dance, Dance2, Dance3, Fishmas; `activate_at_init` → **start-up** | `AllPygmies` (IslandDisplayGroup) → `PygmyStartDancing` | `setAnimation` in `PygmyStartDancing` and `PygmyStoryModeDance`; the `normal` value of `setAnimationBasedOnPygmyType` in `PygmyRefreshGoGoCage` and `TemplePygmyStartDancing`; next animation of `PygmyGoGoCageDropAnim` |
| `SkinDanceVampireDefaultAnim` | 22 | `SkinDanceDefaultAnim` | same option → **start-up** | none (empty `displayGroupId`) | `SkinPackVampireDefaultDanceIdle` |
| `ZombieDanceAnim` | 3 | `PygmyZombieDanceAnim` | Zombie Dance (Zombies), packs Dance, Dance3; init → **start-up** | `AllZombies` → `PygmyStartDancing` | `PygmyStartZombieDancing` |
| `VampireDanceAnim` | 2 | `PygmyDanceDefaultAnim` | Vampire Dance (The Vampire), pack Dance; init → **start-up** | `VampirePygmy` → `PygmyStartDancing` | nothing plays this name (no sequence item, no code literal) |
| `StatueStillAnim`, `StatueTapAnim`, `StatueActivateAnim` | 20 each | `DefaultStatue{Still,Tap,Activate}Anim` | Statue (Sand Island Statue), channel MonsterAtlas, 17 packs → Sand Island atlas load | `OctoStatue` (SandIslandDisplayGroup) → `OctoStatueInit` | `OctoStatueInit`, `OctoStatueTouch`, `OctoStatueActivate` |
| `SandIslandBackgroundInitAnim`, `SandIslandForegroundInitAnim` | 17 each | `DefaultSandIsland{Background,Foreground}InitAnim` | Sand Island Extras, channel ExtrasAtlas, 16 packs → Sand Island atlas load | `Island` → `SandIslandInit`, `Foreground` → `SandForegroundInit` (SandIslandDisplayGroup) | those two sequences |
| `RockIslandBackgroundInitAnim` | 17 | `DefaultRockIslandBackgroundInitAnim` | Rock Island Extras → Rock Island atlas load | `Island` → `RockIslandInit` | `RockIslandInit` |
| `GraveyardBackgroundInitAnim`, `GraveyardForegroundInitAnim` | 17 each | `DefaultGraveyard{Background,Foreground}InitAnim` | Graveyard Island Extras → Graveyard Island atlas load | `Island` → `GraveyardIslandInit`, `Foreground` → `GraveyardForegroundInit` | those two sequences |
| `IceIslandBackgroundInitAnim` | 17 | `DefaultIceIslandBackgroundInitAnim` | Ice Island Extras → Ice Island atlas load | `Island` → `IceIslandInit` | `IceIslandInit` |
| `ApeIslandInitAnim`, `ApeIslandForegroundInitAnim` | 17 each | `DefaultApeIsland{,Foreground}InitAnim` | Ape Island Extras → Ape Island atlas load | `Island` → `ApeIslandInit`, `Foreground` → `ApeIslandForegroundInit` | those two sequences |

(All "Extras" options use channel ExtrasAtlas and appear in 16 packs. Each display object
named here belongs to that island's display group. "Played by" lists every compiled
sequence item that names the alias: 17 items. The game code never plays an alias name
directly.)

The per-value entry lists are `anim_alias_sets` in `assets/data/skin-packs.json`
(textures.md §9.2): 0–19 Statue, 20–36 Sand Island Extras, 37–53 Rock, 54–70 Graveyard,
71–87 Ice, 88–104 Ape, 105–126 Pygmy Dance (`PygmyDanceAnim` and
`SkinDanceVampireDefaultAnim`), 127–129 Zombie Dance, 130–131 Vampire Dance. Set *i* of a
range belongs to option value *i* of that option.

When they are installed:

- **Start-up (dance options, `activate_at_init`).** `PocketGodViewController::loadGame`
  (0x10006a030) runs `parseBinAnim:ext:`, then `SkinPackManager::init` (0x10005cad4), then
  `createLogicClasses`. `SkinPackManager::init` → `initSkinPackDefs` (0x10005d694) →
  `initSkinPackDef:` (0x10005d7c8) → `updateSkinPackDef:…silent:YES init:YES` (0x1000601e4)
  applies the stored selection (keychain, default `"0"`) of every `activate_at_init` option.
  Value 0 of these options needs no other pack. The four dance aliases therefore exist before
  any logic plays an animation.
- **Statue and Extras options (`activate_at_init` = NO).** They are applied when the island's
  location atlas is loaded: `PocketGodViewController::loadLocationTextureAtlas` (0x10007cbc8)
  → `refreshSkinPackOption:…` (location 0 Sand Island: Statue and Sand Island Extras; 1 Rock;
  3 Graveyard; 4 Ice; 5 Ape; textures.md §5.3, §7.4). The skin screen also applies them. Until
  the first such load, these names are unknown ids, which play as empty animations (4.2). The
  behaviour that `FUN_100038dc0` sets then replays them.
- **`ApeIslandForegroundInitAnim` is also a record.** Until Ape Island Extras is first applied,
  the name plays that record. It has the same single frame (state 656,
  `ApeIslandSmallBush_01`) as the default target `DefaultApeIslandForegroundInitAnim`.
  Afterwards the alias replaces the dictionary entry.

**Name resolution rule for the port**: keep one `name → def` map. Insert the records at
load and let alias installation overwrite entries. Resolve every `playAnimationId:` argument
and every `nextAnimationId` through this map at the moment of the call, never at load time.

### 3.3 `BCAnimation` — the player

Each `BCLogic` owns exactly one `BCAnimation` (created in `BCLogic::initWithDisplayObject:`
0x10013841c, listener = the logic). `BCLibrary::setAnimation:::` also creates listener-less
players (section 4.10).

Ivars (from the class dump):

| Ivar | Type | Initial (`init` 0x10019fc8c) |
|---|---|---|
| `mAnimationListener` | `id<BCAnimationListener>` | nil (`setAnimationListener:` 0x10019fd40 retains) |
| `mAnimationDef` | `BCAnimationDef*` | nil |
| `mbDebug` | BOOL | |
| `mFrameIndex` | int | −1 |
| `mAnimTime` | float | 0 |
| `mbUseFramePercent` | BOOL | 0 |
| `mFramePercent` | float | 0 |
| `mbIsAnimationComplete` | BOOL | 0 |
| `mbIsRunning` | BOOL | 0 |
| `mbHibernate` | BOOL | 0 |
| `mbHibernateGameFrameRunning` | BOOL | 0 |
| `mTweenableChannelArray` | `TweenableChannel[30]` | zeroed (0x1E0 bytes) |

Methods:

| Method | Addr | Behaviour |
|---|---|---|
| `play` | 0x1001a01e8 | if hibernating: `mbHibernateGameFrameRunning = 1`; else if not running: running = 1, `[BCView addGameFrameListener:self]` |
| `stop` | 0x1001a0274 | if hibernating: `mbHibernateGameFrameRunning = 0`; else if running: running = 0, remove game-frame listener |
| `reset` | 0x1001a0144 | `mAnimTime = 0; mFrameIndex = -1; mbIsAnimationComplete = 0` |
| `playAnimationId:` | 0x1001a02f8 | section 4.2 |
| `setFramePercent:` | 0x1001a0430 | section 4.8 |
| `onGameFrame:` | 0x1001a04a0 | section 4.3 |
| `replayCurrentFrame` | 0x1001a0170 | if complete: `mFrameIndex = length - 2`, complete = 0, `play` (consequences in 4.7) |
| `setAnimationDef:` | 0x10019ff54 | `mAnimTime = 0`, retain/replace def (does not reset the frame index; unused by game code) |
| `frameIndex` / `isAnimationComplete` / `isHibernating` | 0x1001a0124 / 0x1001a0134 / 0x10019ffbc | getters |
| `hibernate` | 0x10019ffcc | if not hibernating: set, save `mbIsRunning` into `mbHibernateGameFrameRunning`, stop running (remove listener), `hibernate` every bound channel tweenable |
| `unhibernate` | 0x1001a0090 | if hibernating: clear, `play` if it was running, `unhibernate` every bound channel tweenable |
| `setTweenable:channelId:` | 0x10019fdac | section 3.4 |
| `clearTweenableChannelId:` | 0x10019fe74 | section 3.4 |
| `clearTweenableChannelIdNow:` | 0x10019fe9c | section 3.4 |
| `setAnimationListener:` | 0x10019fd40 | retain new, release old (`BCLogic::shutdown` 0x1001389e8 stops the player and sets nil) |
| `id` | 0x1001a047c | `[listener id]` (debug) |
| `gameFrameOrder` | 0x1001a0498 | 20 |
| `setDebug:` | 0x10019fd9c | debug flag only |
| `+traceGlobalStatus` | 0x10019fc40 | debug log of the live-player count (`init`/`dealloc` maintain it) |
| `dealloc` | 0x1001a06b4 | for each bound channel: disable multi-model mode if on, `hibernate` it unless the player is hibernating, release; then `stop`, release def and listener |

### 3.4 Tweenable channels

`TweenableChannel = {BOOL mbMultiModelMode; BOOL mbEnableMultiModelMode; BOOL
mbReleaseNextFrame; id<BCTweenable> mTweenable}` — 16 bytes, 30 per player. A channel binds an
external display object (the "tweenable") to the frame states whose `channel` field equals the
channel number. `mbEnableMultiModelMode` means "referenced by the frame applied last"; it is
set during `applyFrame` and consumed at its end.

- `setTweenable:t channelId:c` (0x10019fdac): if `ch.mbMultiModelMode`: clear it and send
  `disableMultiModelMode` to the **old** tweenable; if the player is not hibernating:
  `[t unhibernate]`; retain `t`, release the old one (the old one is **not** sent
  `hibernate`); store `t`; clear `mbEnableMultiModelMode` and `mbReleaseNextFrame`;
  `replayCurrentFrame`. Passing nil clears the channel the same way.
- `clearTweenableChannelId:c` (0x10019fe74): `ch.mbReleaseNextFrame = 1`; `replayCurrentFrame`.
  The release happens in the bookkeeping loop at the end of the next `applyFrame` this
  player runs.
  - From one of this player's own frame actions, that is the **current** `applyFrame`:
    actions fire before its channel loops (4.4). The tweenable is still positioned by the
    current frame if that frame references the channel, and is then released.
  - Otherwise it is the `applyFrame` of the next frame change, or of the replay if the
    animation was complete.
  - `applyFrame` runs only when the frame index changes. While it does not change, the
    release waits: on a running single-frame looping animation, on a percent-mode player
    whose percent stays the same, or on a stopped player that is not complete.
- `clearTweenableChannelIdNow:c` (0x10019fe9c): clear `mbReleaseNextFrame`; if
  `mbMultiModelMode`: clear it, `disableMultiModelMode`; if the player is not hibernating:
  `[t hibernate]`; release; clear `mbEnableMultiModelMode`; `mTweenable = nil`;
  `replayCurrentFrame`.

Game code binds channels in `BCLibrary(Logic)::initLogicListeners` (0x1001512f8) and in many
logic classes (`[[logic animation] setTweenable:obj channelId:n]`, n = 1…23; e.g.
`ChainSpearLogic::onGameFrame:` assigns skewered pygmies to channels 1–6). Channel 0 is never
bound; treat it as "no channel".

`BCTweenable` protocol (what a channel object must respond to; `BCDisplayObject` and its
subclasses implement it): `id`, `traceStatus:`,
`hibernate`, `unhibernate`, `setTextureDef:`, `textureDef`, `pos`, `rot`, `skew`, `scale`,
`originPos` (pointers to 4-float vectors), `setXPos:`, `setYPos:`, `setZRot:`, `setXSkew:`,
`setYSkew:`, `setXScale:`, `setYScale:`, `setMatrixA:b:c:d:`, `setAlpha:`,
`enableMultiModelMode`, `disableMultiModelMode`,
`drawInMultiModelOpenGL:red:green:blue:alpha:`.

### 3.5 The animation listener — `BCAnimationListener`

Protocol methods: `id`, `onAnimationMoveEnabled`, `isSoundDisabled`,
`applyMoveXPos:moveYPos:`, `parentTweenable`, `setBinaryFrameDef:tweenableChannelArray:`;
plus every frame `action` selector (sent with no arguments). Default implementations in
`BCLogic`:

| Method | Addr | Behaviour |
|---|---|---|
| `parentTweenable` | 0x1001385e8 | the logic's display object |
| `setBinaryFrameDef:tweenableChannelArray:` | 0x100138b50 | forwards to the display object only if it `isKindOfClass:BCMultiModel` (includes `BCTileModel`); otherwise ignored |
| `isSoundDisabled` | 0x100138bf0 | NO |
| `applyMoveXPos:moveYPos:` | 0x100138bf8 | section 4.6 |
| `onAnimationMoveEnabled` | 0x100138cc8 | `mLastMoveXPos = mLastMoveYPos = 0` |
| `animXFlip` / `animYFlip` | 0x100138ce4 / 0x100138d30 | negate the display object's x / y scale (frame actions) |

Overrides: `PygmyLogic::isSoundDisabled` (0x100128940) returns `mbInGoGoCage`;
`PygmyLogic::applyMoveXPos:moveYPos:` (0x100128950) does nothing while `mbInGoGoCage`, else
calls the `BCLogic` version.

---

## 4. Playback semantics

### 4.1 Time base and update order

(The frame loop itself is specified in `docs/re/rendering.md` §2.2–2.3; the facts the player
depends on are repeated here.)

- `BCView::gameLoop` (0x100177730) runs once per display-link tick. The display link frame
  interval is 1 (60 Hz) if `loadGame` took ≤ 13 s, else 2 (30 Hz)
  (`PocketGodViewController::loadGame`).
- `dt = min(now − last, 0.06666667)` in float32 seconds (`now` = seconds since
  `startRenderLoop`; `last` starts at −0.06666667 so the first tick gets 1/15 s). There is no
  lower clamp.
- Each tick: process touches, then sort the game-frame listeners ascending by
  `gameFrameOrder` (`FUN_1001757bc`), snapshot the array, and call `onGameFrame:dt` on each
  snapshot member. Orders: `BCSequence` 10, `BCAnimation` 20, `PygmyLogic` 30, `BCLogic` 40,
  `GameUiManager`/`BCPreferences` 40, `PocketGodViewController(Game)`/`StoryManager` 50, … So
  all sequences step first, then all animation players, then logics. A player started during
  a tick is first updated on the next tick; a player stopped during a tick but still in the
  snapshot returns immediately (it checks `mbIsRunning`).
- Then the display tree is drawn. Animation never interpolates between frames.

### 4.2 `playAnimationId:` (0x1001a02f8)

```
def = library.getAnimationDefById(id)        // by name, aliases included (3.2); nil if unknown or id is nil
mbUseFramePercent = 0
mAnimationDef = def                          // retain new, release old
reset()                                      // time 0, frame index -1, complete 0
if def.length == 0:                          // nil def → length 0 too
    mbIsAnimationComplete = 1
    stop()                                   // nothing is applied; the model keeps its old frame
else:
    play()
    if def.isMoveEnabled: listener.onAnimationMoveEnabled()
```

It does not apply a frame immediately; the first frame is applied by the player's next
`onGameFrame:` — later in the same tick if the player was already running and its update has
not happened yet this tick (e.g. restarted by a sequence item, order 10), otherwise on the
next tick (a newly started player is not in the current tick's listener snapshot). Calling it
with the animation already playing restarts it from time 0.
`BCLogic::playAnimationId:` (0x100138738) just forwards to its player.

### 4.3 `onGameFrame:dt` (0x1001a04a0) — exact algorithm

All arithmetic as in the binary: float32 time, 32-bit ints, `fcvtzs` float→int conversion
(truncate toward zero, saturating, NaN → 0).

```
if !mbIsRunning: return
mAnimTime += dt                                            // f32
def    = mAnimationDef
fps    = def.fps                                           // i32
length = def.length                                        // i32
if mbUseFramePercent: raw = trunc_i32((f32)length * mFramePercent + 0.5)
else:                 raw = trunc_i32(mAnimTime * (f32)fps)
looping = def.loop
if looping: frame = (length == 0) ? raw : raw - (raw / length) * length    // C '%' semantics
else:       frame = (raw < length) ? raw : length - 1
if frame != mFrameIndex:
    def.applyFrame(frame, lastFrame = mFrameIndex, self, listener, channels)    // 4.4; may re-enter
    mAnimationDef.applyMovement(raw, listener)             // re-reads mAnimationDef; raw, not frame
    mFrameIndex = frame
if !looping && mbUseFramePercent != 1 && frame >= length - 1:  // mbUseFramePercent re-read
    mbIsAnimationComplete = 1
    stop()
    next = mAnimationDef.nextAnimationId                  // re-read
    if next != nil: playAnimationId(next)
```

Notes:

- Frame numbers are pure functions of accumulated time: `frame = trunc(t·fps)`. A long `dt`
  skips frames; skipped frames' sounds/actions still fire (4.4).
- A non-looping animation completes on the first tick whose frame reaches `length − 1`; its
  last frame is applied in that same tick. A looping animation never completes.
- `nextAnimationId` chains (168 animations; e.g. `GhostFlyTurnAnim → GhostFlyAnim`) start the
  next animation in the same tick, from time 0. The name is resolved by `playAnimationId:` at
  that moment, so `PygmyGoGoCageDropAnim → PygmyDanceAnim` plays whatever the alias points at
  then (3.2, 2.10). Two non-looping animations name themselves
  (`GraveyardTreePygmyTrappedAnim`, `PygmyFightingPoseAnim`): they restart forever and, because
  the restart clears the complete flag in the same tick, `isAnimationComplete` is never
  observed true. `DiscoCagePygmyIdleAnim` loops and also names itself (a looping animation
  never completes, so its next animation is never used).
- Re-entrancy (from the disassembly): `looping`, `length` and `frame` are captured **before**
  `applyFrame`. If a frame action or sound handler calls `playAnimationId:` on this same
  player, then after `applyFrame` returns: `applyMovement` runs on the *new* def with the old
  `raw`; `mFrameIndex` is overwritten with the old `frame` (undoing the reset to −1); and the
  completion test uses the old `looping/length/frame`, so if the old animation was at its last
  frame the new one is immediately marked complete and stopped, and the *new* def's
  `nextAnimationId` is played. The port must reproduce this ordering to stay
  frame-identical.

  **No shipped content triggers this.** The check covered the IMPs of all 90 frame-action
  selectors and every implementation of every selector they send, on any receiver. None of
  them contains `playAnimationId:` or `setAnimationDef:`. Actions that change what an object
  does call `setBehavior:`, which only schedules:
  - `BCLogic::setBehavior:` (0x1001385f8) → `BCSequence::setSequenceDef:` (0x1000c00ec) →
    `play` (0x1000c01c4), which only adds a game-frame listener;
  - so the new sequence's first item runs on a later tick, at order 10.

  The last-frame actions do not re-enter the player either:
  - `animFlipComplete` clears a flag;
  - `animChooseDirection` and `animXFlip` set the x scale;
  - `animInitBob` calls the fishing pole (`pickupPole`, `initBobTexture`);
  - `animBirthLocust` spawns a locust and sets behaviours;
  - `animPygmyCrawlOut` releases a channel (4.5).

  Frame sounds cannot re-enter, since `BCSound::play` (0x10000db20) only starts playback.
  The re-entrant order only matters if new content plays an animation from a frame action.
  Frame actions do change their own player's channels synchronously, and that case is
  common (4.5).

### 4.4 `applyFrame:lastFrame:animation:animationListener:tweenableChannelArray:` (0x1001a0ca4)

Frame selection, event firing, channel transforms, then channel bookkeeping:

```
if def.frame_count >= 1:
    previous = None; chosen = None
    for f in def.frames (file order):
        idx = (i32)f.index
        if idx > lastFrame + 1 && idx <= frame:
            applySoundAndAction(previous, listener)          // previous == None → nothing
        if idx > frame:
            chosen = previous; break
        previous = f; chosen = f
    if chosen != None:
        applySoundAndAction(chosen, listener)
        listener.setBinaryFrameDef(chosen, channels)         // section 5
        P = listener.parentTweenable                          // the logic's display object
        for s in chosen.states (draw order):
            if s.channel == 0: continue
            T = channels[s.channel].tweenable; if T == nil: continue
            channels[s.channel].mbEnableMultiModelMode = 1
            if s.flags & X:  T.pos.x = (s.x + P.originPos.x) * P.scale.x + P.pos.x
            if s.flags & Y:  T.pos.y = (s.y + P.originPos.y) * P.scale.y + P.pos.y
            if s.flags & ROTATION:
                if P.scale.x > 0:           T.rot.z = s.rotation + P.rot.z
                else if s.flags & SCALE_X:  T.rot.z = (180 - s.rotation) + P.rot.z
                else:                       T.rot.z = P.rot.z - s.rotation
            if s.flags & SCALE_X: T.scale.x = s.scale_x * P.scale.x
            if s.flags & SCALE_Y: T.scale.y = s.scale_y * P.scale.y
            if s.texture_def != NULL: T.setTextureDef(s.texture_def)
for c in 0..30 (always, even if no frame was chosen):
    ch = channels[c]; T = ch.tweenable; if T == nil: continue
    if ch.mbEnableMultiModelMode:
        if !ch.mbMultiModelMode: ch.mbMultiModelMode = 1; T.enableMultiModelMode()
    else if ch.mbMultiModelMode:
        ch.mbMultiModelMode = 0; T.disableMultiModelMode()
    ch.mbEnableMultiModelMode = 0
    if ch.mbReleaseNextFrame:
        ch.mbReleaseNextFrame = 0
        if ch.mbMultiModelMode: ch.mbMultiModelMode = 0; T.disableMultiModelMode()
        if !animation.isHibernating: T.hibernate()
        release T; ch.mTweenable = nil
```

Consequences:

- The frame shown is the **last record whose index ≤ frame** (no interpolation, no
  keyframe tweening; the exporter wrote one record per frame).
- Events. "Fires" means sound first, then action (4.5).
  - Whenever the frame index changes, the chosen record fires, even if the same record
    already fired at the previous change.
  - If frames were skipped (`frame > lastFrame + 1`), each record whose successor's index lies
    in `(lastFrame + 1, frame]` fires first, in file order.
  - For a regular frame list (one record per index, 2,343 of the 2,354 animations), advancing
    from `lastFrame` to `frame` therefore fires records `lastFrame + 1 … frame`, once each, in
    order.
  - After a loop wrap or restart (`frame < lastFrame`), only the chosen record fires. A loop
    that jumps from frame 9 to frame 1 does not fire frame 0.
  - After `playAnimationId:`, `lastFrame = −1`, so frames 0 … `frame` all fire.
- Irregular lists (2.6) follow the same rules.
  - A frame index without its own record re-fires the earlier record:
    `PygmyGainBalanceAnim` frame 5 re-fires record 4, and `PygmyWindBlown2Anim` (indices
    0, 1, 2, 2) frame 3 re-fires its second index-2 record.
  - With that duplicate index, the later record is shown and fires. The earlier one fires
    only when frame 2 is reached by skipping from frame 0.
  - Records past `length − 1` are never shown and never fire.
  - None of the records involved has a sound or action, so the irregular lists change no
    event in shipped data.
- If no record qualifies (an empty frame list, or a negative frame produced by a negative
  frame percent), nothing fires, `setBinaryFrameDef` is not called (the model keeps its
  previous frame) and every channel leaves multi-model mode. In shipped data every animation
  that can tick has a record with index 0.
- Channel transforms ignore the parent's rotation, pivot, skew and depth scaling: they are
  `world = (local + origin) · scale + pos` per axis.
- Channel positions/rotations are written only when the frame index changes; game code may
  move the tweenables freely in between.

### 4.5 Frame sounds and actions — `applySoundAndAction:animationListener:` (0x1001a0bb4)

```
if f == NULL: return
if f.sound != NULL && !listener.isSoundDisabled():
    library.getSoundById(String(f.sound)).play()      // nil sound → nothing (2.10)
if f.action != NULL:
    listener.performSelector(NSSelectorFromString(String(f.action)))   // no arguments
```

Sound first, then action. With a nil listener `isSoundDisabled` is NO (sounds play) and
actions go nowhere. Every action is a zero-argument method of the listener's class. All 90
action names used in v1.48.2 are implemented:

| Action | Frames | Implemented by |
|---|---|---|
| `animBabyThrowSnake` | 1 | PygmyLogic(Island)@1000b1b08 |
| `animBattleChangeWeapon` | 1 | PygmyLogic(BattleOfTheGods)@100123608 |
| `animBattleEnergyCounterHit` | 1 | PygmyLogic(BattleOfTheGods)@100124240 |
| `animBattleFastHit` | 2 | PygmyLogic(BattleOfTheGods)@100124300 |
| `animBattleStrongHit` | 1 | PygmyLogic(BattleOfTheGods)@100124280 |
| `animBattleThrowEnergyBall1`…`4` | 1 each | PygmyLogic(BattleOfTheGods)@100123f5c, 100123ff8, 100124094, 100124130 |
| `animBattleTie` | 3 | PygmyLogic(BattleOfTheGods)@1001241f0 |
| `animBirthLocust` | 2 | LocustQueenLogic@100199c98 |
| `animCatchFish` | 1 | IceHoleLogic@100185640 |
| `animChangeLocations` | 2 | ApocalypseRoomFaderLogic@100178494, DreamBubbleLogic@10019f520 |
| `animCharlieLaunchShark` | 1 | PygmyLogic(Charlie)@1000595e4 |
| `animCharlieThrowFood` | 2 | PygmyLogic(Charlie)@100059648 |
| `animChooseDirection` | 1 | PygmyLogic@10012e920 |
| `animCutPygmyInHalf` | 1 | PygmyLogic(Charlie)@100059588 |
| `animExtended` | 1 | ChainSpearLogic@1000622d8 |
| `animFallOffIsland` | 1 | MonkeyLogic@1000d1fb4 |
| `animFlipComplete` | 3 | PiranhaLogic@100009334, PygmyLogic@10012ea0c |
| `animGhostXFlip` | 1 | PygmyLogic(Island)@1000b7a5c |
| `animGrabPygmy` | 2 | ApeLogic@100088df4, IceHoleLogic@100185c74 |
| `animHalfBeam`, `animHideBeam`, `animShowBeam` | 1 each | LaserGunLogic@1001739e4, 1001739f8, 1001739b4 |
| `animHideChum` | 1 | LaserSharkLogic@1000c5478 |
| `animHideIslandAssets`, `animShowIslandAssets` | 2, 4 | TsunamiLogic@10000c618, 10000c63c |
| `animHideScenery`, `animShowScenery` | 4, 2 | TsunamiLogic@10000c8f8, 10000c660 |
| `animHitBongo`, `animHitWench` | 1 each | ApeLogic@100088ed4, 100088e94 |
| `animHitHumpingMonkey` | 2 | PygmyLogic(Island)@1000aeb2c |
| `animInfectPygmies` | 14 | PygmyLogic(Apocalypse)@100046aac |
| `animInitBob` | 2 | PygmyLogic(Island)@100094cec |
| `animJellyfishAttackComplete` | 2 | PygmyLogic(Underwater)@10011ca94 |
| `animKillPygmy` | 1 | LaserSharkLogic@1000c54b8 |
| `animLandThetanPunch` | 1 | PygmyLogic(Apocalypse)@10003fd58 |
| `animMonkeyClimbComplete` | 3 | ApeTorchLogic@10001fc34, BananaTreeLogic@100136b20 |
| `animMonkeyHelperBlank` | 1 | MonkeyLogic@1000ceef4 |
| `animNoPeeEmbedDie`, `animNoPeeFloatUpward`, `animNoPeeSinkDownward` | 3, 2, 1 | PygmyLogic(NoPee)@10018a6bc, 10018a6e4, 10018a6f8 |
| `animPickUpPygmy` | 1 | MonkeyLogic@1000cefa4 |
| `animPickupPlagueBody` | 6 | PygmyLogic(Apocalypse)@1000479d0 |
| `animPiranhaYFlip` | 2 | PiranhaLogic@1000092c0 |
| `animPounceComplete` | 2 | PygmyLogic(Apocalypse)@10003fc74 |
| `animPygmyCrawlOut`, `animPygmyExitDoorExplode`, `animPygmyExitTopExplode` | 1 each | IglooLogic@1000c3c10, 1000c3ba8, 1000c3b40 |
| `animPygmyReaction`, `animResetIdol`, `animThrowBoulder` | 1 each | KonkeyDongLogic@100090008, 100090024, 1000902ac |
| `animPygmyScaredComplete` | 1 | PygmyLogic(Island)@1000b29a4 |
| `animPygmySplashComplete` | 1 | IceHoleLogic@1001851d4 |
| `animReleaseThetan` | 1 | PygmyLogic(Apocalypse)@10003ecac |
| `animSlamClosetDoorComplete` | 1 | PygmyLogic(Apocalypse)@100041bf4 |
| `animSmashPygmy` | 1 | DiscoBallLogic@10011bb54 |
| `animSpawnSkullAndBone` | 1 | PygmyLogic(Apocalypse)@100044068 |
| `animSpawnThetans` | 1 | ClosetTempleThetanMeterLogic@10019ed0c |
| `animStartDanceIntro` | 1 | DanceDanceExecutionLogic@10017ce74 |
| `animStartFloat` | 5 | PygmyLogic(Apocalypse)@100047810 |
| `animStartPygmyRecoil` | 9 | PygmyLogic(Apocalypse)@1000468c8 |
| `animStartShark`, `animStopShark` | 2, 2 | LaserSharkLogic@1000c5538, 1000c5520 |
| `animStopMoving` | 1 | MoronTurtleLogic@100173518 |
| `animStopThetanPossession` | 1 | PygmyLogic(Apocalypse)@10003fdc4 |
| `animSunCaptured`, `animSunInit`, `animSunPulledIn`, `animSunWhatsUp` | 1 each | ApocalypseCalendarLogic@1000d7990, 1000d78f0, 1000d79e0, 1000d7940 |
| `animSwordFishSwallow` | 1 | PygmyLogic(Island)@1000b8c90 |
| `animThrowLeft`, `animThrowRight` | 2, 1 | IceMonsterLogic@100184590, 1001845b8 |
| `animThrowPeel` | 1 | MonkeyLogic@1000cf0c4 |
| `animThrowPlagueBody` | 5 | PygmyLogic(Apocalypse)@100047918 |
| `animThrowPygmy` | 2 | MonkeyLogic@1000cef18, MoronSnakeLogic@10018c21c |
| `animThrowSpear` | 2 | PygmyLogic(Island)@100094c78 |
| `animThrowWood` | 1 | PygmyLogic(Island)@1000b8724 |
| `animTomBabySacrificeComplete`, `animTomShowXenu` | 1 each | PygmyLogic(Apocalypse)@100040aec, 100040b14 |
| `animWarlockBurnVampire`, `animWarlockStrike` | 1 each | PygmyLogic(Charlie)@100059754, 1000598dc |
| `animXFlip` | 14 | BCLogic@100138ce4 |
| `animYFlip` | 1 | BCLogic@100138d30 |
| `closeWordBalloon`, `openApeKidnapWordBalloon` | 1 each | PygmyLogic(Story)@1001743fc, 1001743e4 |
| `hideFood` | 4 | PygmyLogic(Island)@100094b08 |
| `onHideMap`, `onShowMap` | 1 each | MapPaperLogic@10017a1ec, 10017a208 |

(`timelines.py summary` → `actions` and `sounds` give the full lists with counts.)

**Dispatch must be synchronous.** A frame action is a plain method call made inside
`applyFrame`, after frame selection and before the channel-transform and bookkeeping loops
(4.4). The port must call it at exactly that point, not queue it as an event for later.
16 actions change the channels of the player that is running them (`[self.animation …]`)
and depend on this ordering:

| Actions (class @ IMP) | Call on its own player | Effect within the current `applyFrame` |
|---|---|---|
| `animPickUpPygmy` (MonkeyLogic @1000cefa4, only when the chased pygmy can be caught), `animResetIdol` (KonkeyDongLogic @100090024) | `setTweenable:… channelId:1` | the new tweenable is unhibernated (unless the player is hibernating). If the current frame has a channel-1 state, the tweenable is positioned by it and enters multi-model mode in this same frame |
| `animShowScenery` (TsunamiLogic @10000c660, only at island location 2) | `setTweenable:` channel 1 spear statue, 2 anchor, 3 chum bucket, 4 tile drain | same |
| `animThrowPygmy` (MonkeyLogic @1000cef18) ch 1, `animThrowPeel` (MonkeyLogic @1000cf0c4) ch 3, `animCharlieThrowFood` (PygmyLogic(Charlie) @100059648) ch 14, `animTomShowXenu` (PygmyLogic(Apocalypse) @100040b14) ch 15, `animPounceComplete` (PygmyLogic(Apocalypse) @10003fc74) ch 16 | `clearTweenableChannelIdNow:` | released before the channel loops: the object leaves multi-model mode at once, is hibernated (unless the player is), and is not positioned by this frame |
| `animPygmyCrawlOut` (IglooLogic @1000c3c10) ch 1, `animPygmyExitDoorExplode` (@1000c3ba8) ch 2, `animPygmyExitTopExplode` (@1000c3b40) ch 1, `animThrowSpear` (PygmyLogic(Island) @100094c78) ch 5, `animThrowWood` (@1000b8724) ch 6, `animPygmyScaredComplete` (@1000b29a4) ch 1, `animThrowPlagueBody` (PygmyLogic(Apocalypse) @100047918) ch 6, `animPygmySplashComplete` (IceHoleLogic @1001851d4) ch 1 | `clearTweenableChannelId:` | positioned once more by this frame if it references the channel, then released in this frame's bookkeeping loop |

`MoronSnakeLogic::animThrowPygmy` (0x10018c21c) does not touch channels. The `replayCurrentFrame`
inside these channel calls does nothing here, because a running player is never complete.
Only `BCLogic::setTextureDef:::` (0x100138edc) and `BCLogic::shutdown` (0x1001389e8) stop a
player directly, and nothing outside `BCAnimation` calls its `play`. Other actions affect other
objects or schedule sequences (`setBehavior:`, 4.3); those effects follow their own rules.

There are no frame labels and no "animation complete" callback in the format; completion is
observed by polling (`isAnimationComplete`, `waitForAnimation:::`), by `nextAnimationId`, or
by an action placed on the last frame (e.g. `animFlipComplete`).

### 4.6 Movement — `applyMovement:animationListener:` (0x1001a109c) and `applyMoveXPos:moveYPos:` (0x100138bf8)

Called after every `applyFrame` with the **raw** (unwrapped) frame number:

```
if def.move_enabled != 1: return
loops   = (length != 0) ? (i64)raw / length : 0                // 64-bit signed divide
within  = (i32)(raw - loops * length)
mx = (f32)(i32)loops * def.move_x
my = (f32)(i32)loops * def.move_y
if def.frame_count >= 1:
    g = last frame record whose (i64)index <= within (same scan as 4.4)
    if g: mx += g.move_x; my += g.move_y
listener.applyMoveXPos(mx, my)
```

`BCLogic::applyMoveXPos:mx moveYPos:my` moves the display object by the change since the last
call:

```
pos.x = (f32)( (f64)(scale.x * (mx - mLastMoveXPos)) / (f64)|scale.x| + (f64)pos.x )   // sign of scale.x only
pos.y = pos.y + scale.y * (my - mLastMoveYPos)                                          // full scale.y
mLastMoveXPos = mx; mLastMoveYPos = my
```

`onAnimationMoveEnabled`, sent by `playAnimationId:` for move-enabled defs, resets the last
values to 0, so the first applied frame moves the object by that frame's offset. 56
animations are move-enabled (walks, flights, ghost float bobbing). **The formula above is
the only rule**; do not replace it with a simpler model:

- A frame's `move_x/move_y` is the offset within the current cycle. The per-cycle total
  `def.move_x/move_y` is what each completed cycle adds: at `raw = k·length + i` the result
  is `k·total + frame[i].move`. When frame 0's offset is 0 (every move-enabled animation
  except `PygmyUnderwaterInitAnim`), the displacement at frame index `length`, just after a
  wrap, is exactly the total.
- The total is **not** in general the last frame's value. In 23 of the 56 animations, all
  looping, the last frame's offset differs from the total, which is roughly the last value
  plus one more step. Examples: `MonkeyRunAnim` last 67.5, total 70; `SpiderWalkAnim`
  148.55 / 160; `PygmyBattleFastForwardAnim` 58.7 / 117.4; `GhostIdleAnim` `move_y`
  −0.35 / 0. The other 33 (14 looping, all 19 non-looping) end exactly on the total, e.g.
  `DodoPoopAnim`: 9 frames, 6.25 px steps, total 50.
- `raw` is not clamped. If the completing tick of a non-looping animation overshoots
  (`raw ≥ length`), the formula adds `loops·total + frame[within].move`, which is not
  neutral in general. Overshoot needs `dt > 1/fps`, e.g. a 1/15 s tick at 24 fps.
  `PygmyUnderwaterInitAnim` (24 fps, 13 frames, frame 0 `move_y = +30`, total −115) gives
  −115 at raw 12, −85 at raw 13 and −116.4 at raw 14. A completing tick that skips frame 12
  therefore leaves the pygmy 30 px higher than one that does not. The frame-0 offset of +30
  is real data; do not "fix" it.
- `replayCurrentFrame` (4.7) does not reset `mAnimTime`, so a replayed move-enabled
  animation also computes `raw ≥ length − 1` and can add an overshoot step.

### 4.7 Completion and waiting

`isAnimationComplete` is set only by `onGameFrame:` (4.3) or by `playAnimationId:` with an
empty/unknown animation, and cleared by `reset`, `replayCurrentFrame`, `setFramePercent:`
(when not running). `BCLogic::waitForAnimation:::` (0x100138fcc) returns 2 (sequence item
done, continue with the next item this tick) when complete, else 0 (retry next tick).
`replayCurrentFrame` (0x1001a0170) does nothing unless the player is complete. In that case it
sets `mFrameIndex = [def length] − 2`, clears the complete flag and calls `play`. The three
channel methods (3.4) call it so that a channel change takes effect on a finished
animation. What the next tick does depends on the def:

- **Completed animation with frames** (non-looping, `length ≥ 1`). `mAnimTime` was not
  reset, so `raw = trunc((t + dt)·fps) ≥ length − 1` and the frame clamps to `length − 1`,
  which differs from `length − 2`.
  - `applyFrame(length − 1, lastFrame = length − 2)` re-applies the last record:
    **its sound and action fire again**, `setBinaryFrameDef` is sent and the channel loops
    run.
  - `applyMovement` runs with the new `raw` (4.6). Then the player completes again.
  - Nothing chains. A completed player's def never has a next animation, because a def with
    one hands over to it at completion (4.3).
- **nil def** (unknown id). `mFrameIndex = [nil length] − 2 = −2`. The next tick computes
  frame −1, since `[nil fps]`, `[nil length]` and `[nil loop]` are 0, and sends `applyFrame`
  to nil. Nothing is applied, the channel bookkeeping does **not** run, and the player
  completes again.
- **`NullAnim`** (a record with `length = 0` and no frames; fps 12, not looping, no next
  animation). The next tick computes frame −1 and runs `applyFrame(−1, lastFrame = −2)`. No
  record is chosen, but the bookkeeping loop runs: every bound channel leaves multi-model
  mode and pending `clearTweenableChannelId:` releases happen. Then the player completes
  again.

### 4.8 `setFramePercent:` (0x1001a0430) — scrubbing

```
mbUseFramePercent = 1; mFramePercent = p
if !mbIsRunning: mbIsAnimationComplete = 0; play()
```

While `mbUseFramePercent` is set the frame is `trunc(length·p + 0.5)` (wrapped or clamped as
in 4.3), time is ignored, and the animation never completes. It is cleared only by
`playAnimationId:`. Used for gauges and scrub controls (`HealthBarLogic`, `ApeAltarLogic`,
`ApeWenchLogic`, `ApocalypseTimeTempleLeverLogic`, `IdolScreenLogic`,
`DanceDanceExecutionScreenLogic`) and by the `percent` parameter of `setAnimation:::`.

### 4.9 Hibernation

`BCLogic::hibernate`/`unhibernate` (0x100138874 / 0x100138940) forward to the player; the
player's `hibernate` stops ticking but remembers whether it was running, and
`play`/`stop` while hibernating only update that memory (3.3). Bound channel tweenables are
hibernated/unhibernated with it (display objects keep a hibernate reference count —
`BCDisplayObject::hibernate`/`unhibernate` 0x10010e454 / 0x10010e49c).

### 4.10 Sequence items and other entry points

| Item (positional params) | Handler | Behaviour |
|---|---|---|
| `setAnimation` [0] displayGroupId, [1] animation, [2] stateId, [3] id, [4] percent | `BCLogic::setAnimation:::` 0x100138dac | `playAnimationId(p[1])`; if `p[4] != nil`: `setFramePercent(p[4].floatValue)`; returns 2 |
| same, when the sequence delegate does not implement it | `BCLibrary::setAnimation:::` 0x1000257a8 | player = `mAnimationDictionary[p[3]]`, created (`BCAnimation init`, **no listener**) on first use; `playAnimationId(p[1])`; optional `setFramePercent`; returns 2. Such players only play frame sounds. A nil `p[3]` would throw in ObjC (`setObject:forKey:nil`), so shipped data never reaches this path without an id; only one compiled `setAnimation` item sets `id` (`Pygmy1`, in sequence `Garbage`). |
| `chooseAnimation` [0] comma-separated list | `BCLogic::chooseAnimation:::` 0x100138e30 | split on `","`, `playAnimationId(list[arc4random() % count])`; returns 2 |
| `waitForAnimation` | `BCLogic::waitForAnimation:::` 0x100138fcc | 4.7 |
| `setLayerAnimation` [0] layer (int), [1] animation | `BCScreenLogic::setLayerAnimation:::` 0x10001d054 | `[mScreenLayerArray[layer].mLogic.animation playAnimationId:p[1]]` (screen layers are 10 `{BCLogic*, BCDisplayGroup*}` slots of a screen logic) |
| `setTextureDef` [0] texture, [1] displayGroupId, [2] displayObjectId | `BCLogic::setTextureDef:::` 0x100138edc | if `p[2] != nil`: delegate to `BCLibrary::setTextureDef:::` (another object); else **stop** this player, then `setTextureDef(getTextureDefById(p[0]))` on the own display object (for a `BCMultiModel` this drops the current frame, 5.1) |

Sequence handlers are tried on the sequence's delegate first (`respondsToSelector:`) and fall
back to `BCLibrary` (`BCSequence::runSequence:` 0x1000c02f8). Other animation-choosing items
(`setAnimationBasedOn…`, `chooseRainbowAnimation`, …) are logic-specific and call
`playAnimationId:`.

---

## 5. How display objects apply frames

The renderer spec (`docs/re/rendering.md` §5.3–5.9) documents the same draw functions from
the renderer's side (model transform chain, GL state cache, blending, text layout, tile-model
transition helpers) and agrees with this section, which focuses on how frame data is consumed.

Only the listener decides what a frame does to its display object.
`BCLogic::setBinaryFrameDef:tweenableChannelArray:` forwards to the display object only when
it is a `BCMultiModel` (or subclass `BCTileModel`). For any other display object (plain
`BCModel`, `BCText`, `BCDisplayGroup`, …) an animation drives only sounds, actions, movement
and channels.

### 5.1 `BCMultiModel` state

Ivars: `mpBinaryFrameDef` (current frame), `mpTweenableChannelArray` (pointer to the owning
player's 30 channels), `mModelIndices = {0,1,2,3}` (u16 strip indices), plus `BCModel`'s
`mpTextureDef`.

- `setBinaryFrameDef:f tweenableChannelArray:c` (0x100181950): `frame = f; channels = c;
  mpTextureDef = NULL`.
- `setTextureDef:t` (0x100181924): `mpTextureDef = t; frame = NULL; channels = NULL` (static
  single-texture mode until the next frame arrives).
- `binaryFrameDef` (0x10018197c) exposes the current frame to game code. Direct readers:
  `ChainSpearLogic::onGameFrame:` (0x100061dd8) reads `x` of state `count − 7`;
  `DoodleCloudLogic::checkCollision:lastPos:` (0x1000cd130) reads `(x, y)` of the last state;
  `StormCloudLogic::startTransitionIn:::` (0x1001a8944) only tests for non-NULL.
- Game code clears a multimodel with `setBinaryFrameDef:NULL tweenableChannelArray:NULL`
  (e.g. `IdolScreenLogic::openIdolStartScreen`, `GameUiManager`).

### 5.2 `BCMultiModel::drawOpenGL:` (0x10018198c)

Skipped when `mbHidden` or `mbMultiModelMode`. Otherwise, with `k = −277.12 / pos.z`:
push; translate `(pos.x, k·(pos.y − 160) + 160, pos.z)`; translate
`(k·scale.x·pivot.x, k·scale.y·pivot.y, scale.z·pivot.z)`; rotate x, y, z by `rot`; skew by
`skew.x/skew.y`; scale `(k·scale.x, k·scale.y, scale.z)`; translate `−pivot`; translate
`originPos`. (These model transforms belong to the display-object spec; frame states live in
the resulting local space.) Then:

- if `mpTextureDef != NULL`: colour `(red·alpha, green·alpha, blue·alpha, alpha)` of the model,
  draw the texture's own quad (`FUN_10003841c`: texture `+0x50` vertices, `+0x30` UVs; blend if
  the texture def's blend flag (BOOL at +0x08) is set or `alpha < 1`).
- else, for each state `s` of the current frame, in order (draw order):

```
T = s.dynamic_text (if non-NULL) else channels[s.channel].tweenable
if T == nil:
    if s.texture_def != NULL:
        a = s.alpha * model.alpha
        glColor4f(a·s.red·model.red, a·s.green·model.green, a·s.blue·model.blue, a)
        bind s.texture_def; blend = s.texture_def.blend (+0x08) || a < 1
        draw strip(s.vertices, texture.uv[4], indices 0,1,2,3)
    // else: invisible (34 states have neither texture, text nor channel)
else:
    push
    x  = (s.flags & X) ? s.x : (T.pos.x − model.pos.x) / model.scale.x − model.originPos.x
    y  = (s.flags & Y) ? s.y : (T.pos.y − model.pos.y) / model.scale.y − model.originPos.y
    r  = (s.flags & ROTATION) ? s.rotation
       : model.scale.x > 0 ? T.rot.z − model.rot.z
       : (s.flags & SCALE_X) ? model.rot.z + (180 − T.rot.z)
       : model.rot.z − T.rot.z
    sx = (s.flags & SCALE_X) ? s.scale_x : T.scale.x * model.scale.x
    sy = (s.flags & SCALE_Y) ? s.scale_y : T.scale.y * model.scale.y
    a = s.alpha*model.alpha; cr = s.red*model.red; cg = s.green*model.green; cb = s.blue*model.blue
    if s.texture_def != NULL: T.setTextureDef(s.texture_def)
    glColor4f(a·cr, a·cg, a·cb, a)
    translate(x, y, 0); rotate_z(r); scale(sx, sy, 1)
    T.drawInMultiModelOpenGL(parent: model, red: cr, green: cg, blue: cb, alpha: a)
    pop
```

The non-flagged branches are the exact inverse of the channel transform of 4.4, so an
object positioned by the animation (or by game code) is drawn where it is, but in the
model's draw order and under its transform. Dynamic text takes precedence over a channel.
Notes: a channel's tweenable is drawn even if the state also has a texture (the texture is
passed to it with `setTextureDef:`); if the channel is unbound, the state's own quad is drawn
instead — channels act as optional overrides. The colour handed to OpenGL is premultiplied by
alpha (blend function `GL_ONE, GL_ONE_MINUS_SRC_ALPHA`, `BCView::setupView`); GL clamps
colours to [0, 1], so negative multipliers act as 0. In the original a NULL channel pointer
with a non-text state would crash; it never happens in shipped content.

### 5.3 Nested multimodels — `BCMultiModel::drawInMultiModelOpenGL:red:green:blue:alpha:` (0x100181f60)

A multimodel bound to another animation's channel is in multi-model mode (its own
`drawOpenGL` returns early) and is drawn by the parent through this method with inherited
colour `(r_in, g_in, b_in, a_in)`:

```
if mbHidden: return
translate(originPos)
if mpTextureDef != NULL: draw its quad with the GL colour already set by the caller; return
for s in frame.states:
    a = s.alpha * self.alpha * a_in
    if s.texture_def != NULL:                       // texture first (differs from 5.2)
        glColor4f(a·s.red·self.red·r_in, a·s.green·self.green·g_in, a·s.blue·self.blue·b_in, a)
        draw quad as in 5.2 (blend if texture_def.blend || a < 1)
    else:
        T = s.dynamic_text or channels[s.channel].tweenable; if T == nil: continue
        cr = s.red·self.red·r_in; cg = …; cb = …
        x  = (s.flags & X) ? s.x : T.pos.x          // no inverse transform here
        y  = (s.flags & Y) ? s.y : T.pos.y
        r  = (s.flags & ROTATION) ? s.rotation : T.rot.z
        sx = (s.flags & SCALE_X) ? s.scale_x : T.scale.x
        sy = (s.flags & SCALE_Y) ? s.scale_y : T.scale.y
        glColor4f(a·cr, a·cg, a·cb, a); push; translate(x, y, 0); rotate_z(r); scale(sx, sy, 1)
        T.drawInMultiModelOpenGL(parent: self, red: cr, green: cg, blue: cb, alpha: a); pop
        // no setTextureDef in the nested path
```

### 5.4 Sub-object draw methods (`drawInMultiModelOpenGL:red:green:blue:alpha:`)

| Class | Addr | Behaviour |
|---|---|---|
| `BCDisplayObject` | 0x10010f3fc | nothing (its `setTextureDef:` 0x10010e780 is also a no-op and `textureDef` returns NULL; `BCModel` 0x100118798 and `BCMultiModel` 0x100181924 override them) |
| `BCModel` | 0x1001189d4 | colour = own `(red·alpha, green·alpha, blue·alpha)` (overrides the caller's), translate `originPos`, draw its texture quad; ignores `mbHidden` |
| `BCMultiModel` | 0x100181f60 | 5.3 |
| `BCText` | 0x1001a7d70 | lay out the text (`updateDisplayText`) and draw the glyph quads with the caller's GL colour and transform; ignores its own pos/rot/scale/colour/hidden |
| `BCDisplayGroup` | 0x100067eb4 | `drawOpenGL:` on each child (children apply their own transforms); ignores its own transform and the colour arguments |
| `BCDisplayMarker` | (inherited) | the `BCDisplayGroup` version |
| `BCGradient` | 0x100178e70 | draws its vertex-coloured mesh |
| `BCTileModel` | (inherited) | the `BCMultiModel` version (5.3) |
| `BCRect`, `RainbowModel`, `LightningModel`, `LocustModel`, `HurricaneModel`, `WhirlPoolModel`, `PathModel`, `MagnifyingBeamModel` | (inherited) | the `BCDisplayObject` no-op: drawn nowhere through a channel |

Multi-model mode (`enableMultiModelMode` / `disableMultiModelMode`):

- Only `BCModel` implements it: 0x100118a6c / 0x100118a84 set and clear `mbMultiModelMode`.
  `BCDisplayObject`'s versions (0x10010e56c / 0x10010e570) are no-ops.
- Only two `drawOpenGL:` implementations test the flag and skip: `BCModel` (0x1001187e8) and
  `BCMultiModel` (0x10018198c).
- The other two `BCModel` subclasses inherit the setter but ignore the flag when drawing:
  `BCTileModel::drawOpenGL:` (0x100022b64) and `BCGradient::drawOpenGL:` (0x100178d18),
  which checks only `mbHidden`.

So a channel object that is a `BCText`, `BCDisplayGroup`/`BCDisplayMarker`, `BCTileModel` or
`BCGradient` is still drawn in its own parent group, as well as through the multimodel. A
`BCRect` or custom model is drawn only in its own group, because its
`drawInMultiModelOpenGL:` is the no-op. The dynamic texts created by the loader are in no
group, so they appear only through frames.

### 5.5 `BCTileModel` (scrolling, wrapping strips)

`BCTileModel : BCMultiModel` (`parseStructXml` node type 8: 21 nodes in
`assets/data/display-groups.json` with 18 distinct ids, because `Background` occurs 4 times)
receives frames the same way but draws them with its own `drawOpenGL:` (0x100022b64), choosing
`drawOpenGLVertical:` if `wrapWidth == 0` else `drawOpenGLHorizontal:`. Both are skipped when
hidden or when no frame is set, ignore multi-model mode, channels, dynamic text, rotation,
scale, pivot, skew, the `k` depth factor and the state's red/green/blue (they do translate by
`pos.z`); they require a texture on every state (no NULL check).

Horizontal (0x100022618): `gx` = sum of `xPos` of the ancestor groups excluding the root-most
group; `wx = gx + pos.x`; `w = fmod(wx, wrapWidth)` (double), and if `wx > 0`:
`w −= wrapWidth`. Push; translate `(w − gx, pos.y, pos.z)`. Visible x range
`[max(transitionMinXPos − w, −w), min(transitionMaxXPos − w, 480 − w)]`; draw each state
whose quad x-extent (min/max over the four vertices) overlaps it, with colour
`(a·red, a·green, a·blue, a)`, `a = s.alpha · model.alpha`, model colours only.

Vertical (0x100022908): `gy` = parent group's `yPos`; `wy = gy + pos.y`;
`h = fmod(wy, wrapHeight)`, if `wy > 0`: `h −= wrapHeight`; translate `(pos.x, h − gy, pos.z)`;
visible range `[max(transitionMinYPos − h, −h), min(transitionMaxYPos − h, 320 − h)]`; draw
states with `v2.y ≤ max && min ≤ v0.y`.

Transition limits default to ±1000 (`initWithId:wrapWidth:wrapHeight:` 0x100021ec0) and are
driven by the `startTransitionIn/Out…` helpers (`docs/re/rendering.md` §5.6).

### 5.6 Textures, skins, visibility and draw order

- A state refers to its texture through the `BCTextureDef*` resolved at load, and the quad is
  drawn with that def's *current* GL texture and UVs (`+0x18`, `+0x30`) but the state's
  *baked* vertices. Skin packs re-skin in place: `FUN_100038f24` (used by `SkinPackManager`)
  copies atlas, GL texture, size, UV and quad fields from a skin `BCTextureDef` over the
  original one, so every frame that uses the texture shows the skin immediately, stretched to
  the originally exported quad. The port must therefore keep a mutable texture-def handle per
  state (not baked UVs).
- There is no per-state visibility flag: an object is absent from a frame when the frame has
  no state for it, and transparent when its alpha multiplier is ≤ 0. Whole-model visibility is
  the model's `mbHidden` (and multi-model mode).
- Draw order is the order of the frame's state list; sub-objects drawn through channels or
  dynamic texts take the slot of their state in that list.
- Frame-by-frame sprite animations are exported as one state named `Root` per frame
  (979 such states: 601 with only the texture flag, 378 with `0x2323`), e.g. textures
  `Ant_01`, `Ant_02`, …

---

## 6. Notes for the Rust port

- **Data**: embed `Timelines.bin` as is (17.2 MB) and parse it at start-up with a direct port
  of `timelines.py` (`Layout` with W = 8): six length-prefixed sections, string offsets
  relative to byte 8, record offsets relative to each section's data. Keep states in one
  `Vec` and frames as index lists into it (states are shared). Parsing is a few milliseconds.
  Resolve texture ids against the texture-def registry and dynamic-text ids against the
  text records exactly once, as the loader does.
- **Animation registry**: one `HashMap<String, AnimationId>` (name → def).
  - Insert the 2,354 records at load. Skin-pack alias installation (3.2) overwrites entries,
    and an alias may replace a record (`ApeIslandForegroundInitAnim`).
  - Resolve every `playAnimationId:` argument and every `next_animation` through this map
    **at the moment of the call**. Never pre-resolve `next_animation` to an index at load:
    `PygmyDanceAnim` is an alias.
  - A name missing from the map is an empty animation: complete at once and keep the previous
    frame, with no error and no panic. This covers the unknown ids of 2.10 and an Extras or
    Statue alias before its island has loaded.
- **Dynamic-text registry**: `HashMap<String, Handle>` of 176 shared, mutable text objects
  created from the text records (font, justification and trimmed text: `BCText::setText:`
  strips leading and trailing `\t`, `\n`, `\r` and spaces).
  - Game code looks texts up by id and changes their text, wrap width, line count and text
    buffer at runtime (2.9). Every frame that names the id draws the text's current state.
  - Frame states must hold the text handle, not a copy of the text.
- **Player component** (one per logic): mirror the `BCAnimation` ivars, the 30 channels and
  the algorithms of sections 4.2–4.8 verbatim, including f32 time accumulation, `trunc`
  conversion, the event-firing scan, re-entrancy ordering and `replayCurrentFrame`.
  Schedule players after sequences and before logics (orders 10 < 20 < 30 < 40) with
  `dt = min(real dt, 1/15)`.
- **Events**: frame sounds go through the listener's `is_sound_disabled`. Frame actions are
  zero-argument calls on the owning logic, looked up by name (a `match` on the 90 names or a
  per-logic dispatch table).
  - Actions must run **synchronously inside `apply_frame`**, between frame selection and the
    channel loops (4.4). The 16 actions that bind or release channels on their own player
    rely on this (4.5). Do not turn actions into deferred events.
  - Listener-less players (`BCLibrary::setAnimation:::`) only play sounds.
- **Rendering**: a multimodel renders its current frame's states as textured quads with the
  stored vertices and the texture's UVs, or delegates to channel objects / dynamic texts at
  the state's transform (5.2). A `BCModel` or `BCMultiModel` bound to a channel that the
  current frame references must be hidden from its normal place (multi-model mode). A bound
  `BCTileModel`, `BCGradient`, text or group stays visible there as well (5.4).
- Keep the anomalies: silent unresolved sounds, irregular frame lists (including their
  re-firing, 4.4), the stored vertices of the 24 rescaled-def states, and the movement
  formula with overshoot (4.6). Treat NaN colours as 0, or skip those states; the three
  animations appear unused.

---

## 7. JSON schema (tool output)

`anim` and `dump` print one `TimelineDump` object; `texts` prints `{word_size, texts}`.
Field names are snake_case, key order is fixed, records keep file order, floats are the
shortest decimal that round-trips to the same float32, absent strings are `null`.

```rust
#[derive(serde::Deserialize)]
pub struct TimelineDump {
    pub word_size: u32,               // 8 for v1.48.2
    pub texts: Vec<TextDef>,          // `anim`: only texts used by the dumped animations
    pub animations: Vec<AnimationDef>,
    pub states: Vec<FrameState>,      // `anim`: only referenced states, sorted by index
}

#[derive(serde::Deserialize)]
pub struct TextDef {
    pub id: String,
    pub text: String,
    pub font: String,
    pub justification: i32,           // 0 left, 1 right, 2 centre
}

#[derive(serde::Deserialize)]
pub struct AnimationDef {
    pub name: String,
    pub fps: i32,
    pub length: i32,
    pub looping: bool,
    pub move_enabled: bool,
    pub move_x: f32,
    pub move_y: f32,
    pub next_animation: Option<String>,
    pub frames: Vec<FrameDef>,
}

#[derive(serde::Deserialize)]
pub struct FrameDef {
    pub index: i32,
    pub sound: Option<String>,
    pub action: Option<String>,
    pub move_x: f32,
    pub move_y: f32,
    pub states: Vec<u32>,             // FrameState::index values, draw order
}

#[derive(serde::Deserialize)]
pub struct FrameState {
    pub index: u32,                   // record number in the states section
    pub name: String,
    pub flags: u32,                   // section 2.8
    pub texture: Option<String>,      // set when flags & 0x2000
    pub dynamic_text: Option<String>, // set when flags & 0x4000
    pub channel: u8,                  // 0 = none
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub skew_x: f32,
    pub skew_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub alpha: Option<f32>,           // null only for the three NaN states (2.10)
    pub red: Option<f32>,
    pub green: Option<f32>,
    pub blue: Option<f32>,
    pub vertices: [[f32; 3]; 4],
}
```

`summary` prints `{file, file_size, word_size, sections[{name, size_offset, data_offset, size,
record_size, record_count}], animation_count, looping_count, move_enabled_count,
next_animation_count, total_length, max_states_per_frame, fps_histogram[{fps, count}],
flag_bits[{mask, name, count}], channel_histogram[{channel, count}],
channel_state_names[{channel, state_names[{state_name, count}]}], fonts[{font, count}],
justification_histogram[{justification, count}], sounds[{sound, count}],
actions[{action, count}], irregular_frame_indices[{animation, length, frame_indices}],
alias_source, animation_aliases[{alias, is_record, targets}],
next_animations_via_alias[{animation, next_animation}],
dangling_next_animations[{animation, next_animation}],
non_finite_color_states[{state, used_by}], vertex_fit}`.

The newer fields:

- `alias_source: Option<String>` is the repo-relative path the aliases were read from: the
  arm64 binary for W = 8, `texture_test.xml` for W = 4, `null` if neither is available.
  `animation_aliases` is then `[]`.
- `animation_aliases` is sorted by alias, and `targets: Vec<String>` is sorted.
  `is_record: bool` is true when the alias name is also a record.
- `dangling_next_animations` lists next animations that name neither a record nor an alias.
  It is empty for both shipped files.
- `vertex_fit` is
  `Option<{textured_states: u32, fitted_states: u32, rescaled_def_states: [{state: u32, texture: String}], unexplained_states: [same]}>`,
  and is `null` without the arm64 binary.

---

## 8. Open questions

1. **NaN colours** (2.10). OpenGL ES behaviour for NaN `glColor4f` input is
   implementation-defined. The three affected animations (`ButtonNewMatch`, `Statue1`,
   `UIHeader`) are referenced nowhere in the examined data: not by the 269 literal
   `playAnimationId:` arguments in code, the compiled sequences, the skin-pack alias lists,
   the display groups or the logic objects. (`Statue1` occurs in the display groups only as
   a static model's texture.) They appear unused. Recommendation: treat NaN as 0.
2. **Listener order among equal `gameFrameOrder`**. `NSMutableArray
   sortUsingFunction:context:` re-sorts the listener array in place every tick. This binary
   cannot show whether Apple's implementation is stable, i.e. whether it keeps the order in
   which players were started among the many with order 20. The open-source
   CoreFoundation `CFArraySortValues` uses a merge sort, which is stable. Recommendation:
   keep start order.
3. **Exporter-only fields**. The meaning of `skew_x/skew_y`, of the alpha/colour flag bits
   and of `x/y/rotation/scale` on plain textured states is inferred from the data: the
   vertex fit of 2.8, which `timelines.py validate` reproduces (78,532 fitted, 24 explained
   by rescaled defs, 0 unexplained). The runtime never reads these fields, so they do not
   affect playback.
4. **Channel objects that are drawn twice**. A `BCText`, `BCDisplayGroup`/`BCDisplayMarker`,
   `BCTileModel` or `BCGradient` bound to a channel is not suppressed by multi-model mode,
   so it is drawn both in its own display group and through the multimodel (5.4). Nobody
   has audited whether game code ever binds such objects, or always detaches them from their
   group. The audit needs the class of the argument at each of the 372
   `setTweenable:channelId:` call sites, which are spread over 45 decompiled files.

Resolved since the first version of this document: re-entrant `playAnimationId:` from frame
actions does not occur in shipped content (4.3). Frame actions that change their own
player's channels are listed in 4.5.
