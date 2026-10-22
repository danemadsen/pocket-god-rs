# Textures: atlases, texture defs, .raw formats, channels and skins

Spec for everything the port needs to draw sprites exactly like Pocket God 1.48.2: the
pixel files, the compiled atlas / sprite tables, when each GL texture is (re)loaded, how
skins swap textures, and how a texture def becomes a quad on screen.

Generated data (from `tools/re/extract_textures.py`, see [§10](#10-tool)):

| Output | Content |
|---|---|
| `assets/textures/<Name>.png` | One RGBA8 PNG per bundle `.raw` (530 files), **premultiplied alpha** |
| `assets/data/textures.json` | 26 atlas slots, 8337 texture defs, 530 images, 19 bits slots |
| `assets/data/skin-packs.json` | 24 skin packs → objects → options → values (5149 values) |

All addresses are in the arm64 slice `tmp/re/PocketGod.arm64`; function names follow
`docs/re/README.md`. The source file of the texture engine is
`BoltLib/UniversalClasses/Shared/BCTextureDef.mm`; its free functions are named below from
their log strings (`FUN_xxxx` in the decompilation, file `tmp/re/decomp/game/OneShotAnimLogic.c`).

## 1. Summary

- Every texture the game draws lives in one of **26 GL texture slots** (`BCAtlasTextureDef`).
  A slot has a fixed size and pixel format; different `.raw` images are uploaded into the
  same slot over time (27 island atlases share `screenAtlas0`, 55 monster/statue/idol
  images share `atlasTRexGreen0`, ...).
- A sprite is a `BCTextureDef`: a pointer to its slot, a texel rect (as UVs inset by half a
  texel) and a ready-made quad (4 vertices in pixels, y up, origin = bottom-left of the
  untrimmed source frame). Models draw the def's quad with the def's UVs on the slot's
  current GL texture.
- Pixel data is headerless. 527 files are RGBA4444 (u16 little-endian, R in bits 15-12,
  A in bits 3-0); the 3 `TextureAtlasPvr*` files are PVRTC1 4 bpp RGBA, 1024×1024.
  All data is **premultiplied alpha**; the game blends with `glBlendFunc(GL_ONE,
  GL_ONE_MINUS_SRC_ALPHA)`. Filtering is `GL_LINEAR` (min and mag), no mipmaps, wrap is the
  GL default `GL_REPEAT` (never reached: every UV is inset by half a texel).
- Skins change textures three ways: upload a whole image into a slot, overwrite rects of
  a slot with sub-images from a "bits" file (`glTexSubImage2D`), and repoint base texture
  defs at another def (copy of the def's slot pointer, sizes, UVs and quad).

## 2. Pixel files (`*.raw`)

### 2.1 Formats

`BCAtlasTextureDef.format` (+0x28) selects the GL upload in `atlasTextureDefLoadRaw`
(`FUN_100037adc`) and `activateTextureAtlas` (`FUN_100038b68`):

| Code | Name in JSON | GL internal format / format | GL type | Bytes per texel | Used |
|---|---|---|---|---|---|
| 1 | `png` | loaded from `<file>.png` through UIImage, converted to RGBA4444 (`FUN_100038168`) | `GL_UNSIGNED_SHORT_4_4_4_4` | – | no |
| 2 | `rgb888` | `GL_RGB` | `GL_UNSIGNED_BYTE` | 3 | no |
| 3 | `rgb565` | `GL_RGB` | `GL_UNSIGNED_SHORT_5_6_5` | 2 | no |
| 4 | `rgba8888` | `GL_RGBA` | `GL_UNSIGNED_BYTE` | 4 | no |
| 5 | `rgba4444` | `GL_RGBA` | `GL_UNSIGNED_SHORT_4_4_4_4` | 2 | 527 files |
| 6 | `rgba5551` | `GL_RGBA` | `GL_UNSIGNED_SHORT_5_5_5_1` | 2 | no |
| 7 | `l8` | `GL_LUMINANCE` | `GL_UNSIGNED_BYTE` | 1 | no |
| 8 | `la88` | `GL_LUMINANCE_ALPHA` | `GL_UNSIGNED_BYTE` | 2 | no |
| 9 | `pvrtc4` | `glCompressedTexImage2D`, `GL_COMPRESSED_RGBA_PVRTC_4BPPV1_IMG` (0x8C02) if the slot's `has_alpha`, else `..._RGB_...` (0x8C00) | – | ½ | 3 files |
| 10 | `pvrtc2` | as 9 with 0x8C03 / 0x8C01 | – | ¼ | no |

Packed 16-bit formats are little-endian `u16` per texel, rows top to bottom (the first row
is texture row 0, `v = 0`, which the quads put at the top of the sprite; see §6).
RGBA4444 channel layout, as confirmed by the PNG converter in `FUN_100038168`
(`r4<<12 | g4<<8 | b4<<4 | a4`): R = bits 15-12, G = 11-8, B = 7-4, A = 3-0. Expansion to
8 bits is bit replication (`v * 17` for 4 bits; `(v<<3)|(v>>2)` for 5; `(v<<2)|(v>>4)` for 6).

Format 1 (no def uses it): `FUN_100038168` reads `<file>.png`, draws it with
`CGContextDrawImage` into a zeroed premultiplied RGBA8 context of the slot size at
(0, H − h) in CoreGraphics coordinates, i.e. the image's top-left at texel (0, 0), with
`w×h` = the slot's own def width/height (+0x68/+0x6c); converts the first `w·h` texels to
RGBA4444 by keeping the high nibble of each channel; `glGenTextures`, `GL_LINEAR` min
filter, `glTexImage2D` of the slot size.

PVRTC1: 64-bit blocks (4×4 texels at 4 bpp, 8×4 at 2 bpp) stored in Morton order (block y
bit in the even positions, block x bit in the odd ones, extra high bits of the larger
dimension appended); each block is a little-endian modulation word followed by a colour
word. `decode_pvrtc` in the tool is bit-exact with `PVRTDecompress.cpp` of the PowerVR SDK
(checked on `TextureAtlasPvr0` and on random 4 bpp and 2 bpp data, square and non-square).

### 2.2 Premultiplied alpha

The bundle textures are premultiplied: in every one of the 527 RGBA4444 files at least
99.2 % of texels have `r, g, b <= a` (the rest is dithering noise from the PVRTexTool conversion,
`usePVRTexTool="true"` in texture.xml); the PNG path draws into a
`kCGImageAlphaPremultipliedLast` context; `BCView::setupView` (0x1001765a4) sets
`glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA)` once and nothing else ever changes it; the
vertex colour is premultiplied (`glColor4f(r*a, g*a, b*a, a)`, `BCModel::drawOpenGL:`
0x1001187e8). The PNGs therefore keep the premultiplied values unchanged (4→8 bit
replication only). The port must blend with `(ONE, ONE_MINUS_SRC_ALPHA)` and sample with
linear filtering; converting to straight alpha would lose the few `a = 0, rgb > 0`
texels (they add light in the original) and introduce fringes.

### 2.3 Every file

The tool classifies each of the 530 files; `textures.json` `images[]` has every file's
size and use. Two kinds:

- **atlas**: a full slot image, uploaded with `glTexImage2D`/`glCompressedTexImage2D`.
- **bits**: concatenated RGBA4444 sub-images, uploaded one by one with `glTexSubImage2D`
  into rects of slots (§7.3). The PNG stacks the sub-images vertically in file order,
  left-aligned (`regions[].src_y`); PNG width = widest region.

| Kind | Format | PNG size | Bytes | Regions | Count | Files |
|---|---|---|---|---|---|---|
| atlas | pvrtc4 | 1024×1024 | 524288 | | 3 | TextureAtlasPvr0-2 |
| atlas | rgba4444 | 1024×1024 | 2097152 | | 3 | TextureAtlas0, TextureAtlas1, MainBattleAtlas0 |
| atlas | rgba4444 | 512×512 | 524288 | | 34 | 27 island/screen atlases (§5.3) + TextureAtlasTest0-6 |
| atlas | rgba4444 | 256×512 | 262144 | | 1 | WidescreenOverlayAtlas0 |
| atlas | rgba4444 | 256×256 | 131072 | | 133 | every `*TextureAtlas0/1` of monsters, sharks, TRex skins, `*Extras0`, `*SandStatue0`, dance atlases, IconPage/IdolPage0-1, Disco0-1, Xenu0-1, EpisodeAlertTextureAtlas0, VampireStayingAlive0 |
| atlas | rgba4444 | 128×128 | 32768 | | 1 | NewsAtlas0 |
| atlas | rgba4444 | 16×16 | 512 | | 3 | LocustTextureAtlas0-2 |
| bits | rgba4444 | 74×967 | 110884 | 26 | 2 | SharkDefault, SharkAdultSwim (unreferenced, §2.4) |
| bits | rgba4444 | 96×445 | 52860 | 6 | 15 | OutHouse* |
| bits | rgba4444 | 30×1210 | 46780 | 60 | 2 | MonkeyStandard, MonkeyZombie |
| bits | rgba4444 | 40×644 | 50540 | 22 | 15 | Fish* |
| bits | rgba4444 | 118×209 | 41548 | 2 | 17 | Statue* (underwater statue) |
| bits | rgba4444 | 40×501 | 29460 | 20 | 18 | Dodo* |
| bits | rgba4444 | 68×249 | 17956 | 9 | 8 | Shark*Laser |
| bits | rgba4444 | 128×128 | 32768 | 1 | 49 | EpisodeIdol1-48, 31b |
| bits | rgba4444 | 130×124 | 21880 | 3 | 15 | Igloo* |
| bits | rgba4444 | 58×141 | 13904 | 5 | 2 | PineTreeStandard, PineTreeChristmas |
| bits | rgba4444 | 84×84 | 14112 | 1 | 49 | EpisodeIcon1-48, 31b |
| bits | rgba4444 | 52×129 | 12072 | 3 | 18 | TRexEgg* |
| bits | rgba4444 | 80×80 | 12800 | 1 | 27 | Moon* |
| bits | rgba4444 | 100×60 | 12000 | 1 | 21 | Volcano* |
| bits | rgba4444 | 64×64 | 8192 | 1 | 31 | Meteor* (27), Default/Store/Story/TipsNewsIcon |
| bits | rgba4444 | 40×96 | 6308 | 4 | 16 | Doodle* |
| bits | rgba4444 | 18×108 | 3588 | 7 | 17 | Coconut* (16), DiceCoconut |
| bits | rgba4444 | 50×35 | 3500 | 1 | 7 | TRexNoHat, TRexSantaHat, TRexTopHat, TRexHeadPhone, TRexPygmyHair, TRexVikingHat, TRexSombrero |
| bits | rgba4444 | 80×12 | 1920 | 1 | 17 | Spear* |
| bits | rgba4444 | 32×25 | 1600 | 1 | 6 | TRexNoTattoo, TRexTattoo* |

Every file's byte size equals the size implied by its use (checked by the tool).
`StatueCandyland.raw` and `StatueUnderwaterWorld.raw` are byte-identical, as are
`IdolPageTextureAtlas0.raw` and `IdolPageTextureAtlas1.raw`. `DefaultSandStatue0.raw` is a
uniform opaque black 256×256 image (every texel `0x000F`): the "Regular Statue" value loads
it to blank the monster slot (the regular octopus statue is drawn from the island atlas).

### 2.4 Files the 1.48.2 code never loads

- `IdolPageTextureAtlas1`: page 1 of the IdolPage atlas. `SkinPackManager::loadTextureAtlasChannel:`
  (0x10005cec0, checked in the disassembly at 0x10005cfc4-0x10005d138) loads page **0**
  into both monster slots; page 1 is identical, so the result is the same.
- `SharkDefault`, `SharkAdultSwim`: leftovers of the 1.45.3 bits-based shark skins
  (texture_test.xml `<skin filename="SharkDefault">` for every shark skin). The 26 non-empty
  sub-images follow the skinBase order Fin 01-06, BackFin, TopFin, SideFin, Tail 01-05,
  Head 01-15 (empty textures Tail_03, Head_04, Head_07 have no bytes); the tool exports
  them with the base shark defs as targets. The pixels match no shipped atlas exactly
  (best: `SharkDefault` ≈ Toy shark 63 %, `SharkAdultSwim` ≈ AdultSwim shark 60 % equal
  texels); `referenced: false` in the JSON.

## 3. `BCAtlasTextureDef` (one GL texture slot)

Type encoding `{BCAtlasTextureDef=@@@@@iiiBBBffI{BCTextureDef=...}}`, size 0xc8.

| Off | Type | JSON field | Meaning | Evidence |
|---|---|---|---|---|
| 0x00 | NSString* | `init_image` | `.raw` base name uploaded when the slot is created; NULL = slot created empty | `atlasTextureDefLoadRaw` returns when NULL |
| 0x08 | NSString* | – | never read; NULL in all 26 defs | tool asserts NULL |
| 0x10 | NSString* | `alt_image` | alternative file used when `atlasTextureDefLoadRaw`'s 3rd argument is non-zero (no caller passes it) | `FUN_100037adc` |
| 0x18 | NSString* | – | never read; NULL in all defs | tool asserts NULL |
| 0x20 | NSString* | `ext` | resource extension, `@"raw"` in all defs | `FUN_100037adc` |
| 0x28 | int | `format_code` | pixel format, §2.1 (5 or 9 in the data) | |
| 0x2c | int | `alt_format_code` | format of `alt_image` (0 everywhere) | |
| 0x30 | int | `zlib` (== 1) | 1 = file is zlib (`uncompress` into a buffer of the format's byte size; for format 2 the buffer is sized w·h·4) | `FUN_100037adc` |
| 0x34 | BOOL | `from_caches_dir` | read `<Library/Caches>/<file>.<ext>` instead of the bundle | `NSSearchPathForDirectoriesInDomains(13,1,1)` |
| 0x35 | BOOL | `no_init` | skip slot creation in `atlasTextureDefInitRaw` (no GL name) | `FUN_100038084` |
| 0x36 | BOOL | `mipmap` | `GL_GENERATE_MIPMAP` + min filter `GL_LINEAR_MIPMAP_LINEAR`; else min filter `GL_LINEAR` | `FUN_100038084` |
| 0x38 | float | `width` | texture width in texels (cast to int for GL) | |
| 0x3c | float | `height` | texture height | |
| 0x40 | GLuint | – | GL texture name (runtime) | |
| 0x48 | BCTextureDef | (texture with id = atlas id) | the whole slot as a sprite; its `id` is the slot's id and map key; its BOOL is the slot's `has_alpha` | `BCLibrary::addAtlasTextureDef:` keys on +0x48 |

All 26 slots: formats 5/9, `zlib`, `from_caches_dir`, `no_init`, `mipmap` false, no alt file.

v1.45.3 XML → fields: `<externalTextureAtlas id="atlas" filename="TextureAtlas" type="RGBA4444"
atlasWidth atlasHeight xPad yPad channelId>` produced one slot per page, id = `id` + page
(`atlas0`, `atlas1`), file = `filename` + page (`TextureAtlas0`), `type` → format
(`RGBA4444` 5, `PVR4BBP` 9). `folder`, `ext="xml"`, `xPad`/`yPad` (packing padding, 4 for
PVR, else 0) and `noCrop` only affected the build. Atlases with the same `channelId` were
packed against the channel's first atlas def: their sprites point at that slot (§5.3).

## 4. `BCTextureDef` (one sprite)

Type encoding `{BCTextureDef=@B^{BCAtlasTextureDef}^Iffff[8f][12f]}`, size 0x80.

| Off | Type | JSON | Meaning |
|---|---|---|---|
| 0x00 | NSString* | `id` | unique id (map key of `BCLibrary.mpTextureDefMap`) |
| 0x08 | BOOL | `has_alpha` | draw with blending even at alpha 1 (`FUN_10003841c`); on a slot's own def it also picks PVRTC RGBA vs RGB. 79 defs are opaque (UI boxes, backgrounds, achievement icons, ...) |
| 0x10 | BCAtlasTextureDef* | `atlas` | slot the def samples |
| 0x18 | GLuint* | – | `&atlas->glName` (always slot + 0x40; checked) |
| 0x20 | float | `width` | trimmed width = texel rect width |
| 0x24 | float | `height` | trimmed height |
| 0x28 | float | `original_width` | untrimmed source frame width (also the glyph advance in `BCFontDef::textWidth:` 0x100066818, and the divisor of the fishing string's x scale in `FishingPoleLogic::onGameFrame:` 0x1001a29ec) |
| 0x2c | float | `original_height` | untrimmed source frame height |
| 0x30 | float[8] | `uvs` | (u,v) of top-left, top-right, bottom-left, bottom-right |
| 0x50 | float[12] | `vertices` | (x,y,z) of the same 4 corners, pixels, z = 0 |

UVs (verified for all 8337 defs, to float32 precision after rounding to 6 decimals, which
is how the generator printed them into the source):

```
u0 = round6((x + 0.5) / W)      u1 = round6((x + width - 0.5) / W)
v0 = round6((y + 0.5) / H)      v1 = round6((y + height - 0.5) / H)
uvs = [u0,v0, u1,v0, u0,v1, u1,v1]
```

`(x, y)` is the texel rect's top-left in the image (row 0 = first row of the file), W×H the
slot size. The tool derives `x = round(u0·W − 0.5)`, `y = round(v0·H − 0.5)`. There is no
rotation anywhere (all defs have `u` constant per column and `v` per row).

Vertices: `[l,t,0, r,t,0, l,b,0, r,b,0]` in pixels, y up, relative to the bottom-left
corner of the untrimmed source frame (`original_width × original_height`). For a normally
trimmed sprite `r − l = width`, `t − b = height`, `l` = trim offset from the left and
`b` = trim offset from the bottom (so the trim offset from the top is
`original_height − t`). Exceptions, kept exactly in the JSON:

- 740 defs in skinned slots (`atlasTRexGreen0` 253, `atlasTRexGreen1` 288,
  `atlasSharkDefault0` 144, `atlasTest0-6` 55) have a quad exactly 1 px narrower than the
  texel rect, same height (e.g. `TRex_Head_04`: rect 100×72, quad (1,21)-(100,93)); the
  texture is squeezed by that pixel in the original too.
- 26 defs are deliberately rescaled: PVR bats drawn at 50 % (`BatTurn_05` rect 75×41, quad
  37.5×20.5), `ElectricGlow_*` at 4×, `VolcanoGeyser_01/02` 18×120 → 18×385, `MenuBG`
  50×10 → 395×50, `SunsetReflection` 200×150 → 200×165.
- 47 defs are empty (`empty: true`), from fully transparent source frames: the 36 shark
  `Tail_03`/`Head_04`/`Head_07` defs (every skin and the base) have size 0×−1 and a 1×1
  inverted quad at x −2..−1, y h−1..h (h = original height) sampling a reversed 1×2 texel
  span; the other 11 (font space glyphs `*:FullSet:0`, `BlankTexture`, `Scorch`,
  `LavaGlob_11`, `PurificationDissolve_15`, `PygmyFinalDie_16/17`,
  `PygmyDanceDreamHandsDance_47`) have size 0×0 and a zero-area quad at (0, h−1). Keep the
  data as is; they draw (almost) nothing.

So the port must draw the def's `vertices` with the def's `uvs`; never rebuild the quad
from the rect.

v1.45.3 XML → defs: `<addTextureToAtlas id>` is one def; `<addTextureSequenceToAtlas
id="X_" count="N">` gives `X_01` ... `X_NN`; `noCrop="1"` disables trimming (size =
original size, quad = whole frame; 328 defs are untrimmed in total); `folder`/`filename`/
`ext` name the source PNG (build only); `skinId` names the base def a skin texture replaces
(§7.1).

Font glyphs are ordinary defs named `<FontId>:FullSet:<n>` (n = 0-based index in the
font's char set, see `font-defs.json`), spread over the `atlasTest0-6` slots
(`BCFontDef::fillFontCharTextureArray:...` 0x100066c08).

## 5. Def tables, slots and channels

### 5.1 Registration

`PocketGodViewController::loadGame` (0x10006a030) calls
`BCLibrary processAtlasTextureDefs:textureDefs:` (0x1000240b0) twice:

| Atlas list (NULL-terminated `BCAtlasTextureDef*`) | Texture list (`BCTextureDef*`) | Slots | Defs |
|---|---|---|---|
| 0x100602950 | 0x1005f9738 | 16 (skin, test, news, alert, vampire slots) | 4674 |
| 0x1007cbc08 | 0x1007c4988 | 10 (atlas0, atlas1, widescreenOverlayAtlas0, screenAtlas0, LocustAtlas0-2, pvratlas0-2) | 3663 |

`processAtlasTextureDefs:` (0x100024000) registers each slot (`addAtlasTextureDef:`,
key = embedded def id) and creates it (`FUN_1000383c4`: format 1 → PNG loader
`FUN_100038168`, 2-10 → `atlasTextureDefInitRaw` `FUN_100038084`). `processTextureDefs:`
(0x10002405c) registers every def by id; the texture lists include each slot's own def.
Lookups (`getAtlasTextureDefById:` 0x100023414, `getTextureDefById:` 0x1000235c4) copy the
NSString into a 1 KB static buffer and use `std::map<std::string, T*>::operator[]`, so an
unknown id yields NULL (and inserts an empty entry). JSON order = binary list order.

`atlasTextureDefInitRaw`: unless `no_init`, `glGenTextures` (once), bind, set
`GL_TEXTURE_MIN_FILTER` (`GL_LINEAR`, or mipmapped), then `atlasTextureDefLoadRaw(def, 0, 0)`
which uploads `init_image` when set. `GL_TEXTURE_MAG_FILTER` and the wrap modes are never
set (GLES defaults: `GL_LINEAR`, `GL_REPEAT`). A slot without `init_image` stays without
an image until something loads one.

Then loadGame loads `TextureAtlas0` → `atlas0` (def 0x100751f38) and `TextureAtlas1` →
`atlas1` (def 0x100767d00) (their defs have no `init_image`), and
`WidescreenOverlayAtlas0` → `widescreenOverlayAtlas0` (looked up by id) when
`BCView isWideScreen` or `isTallScreen`. Other slot addresses used directly by code:
`screenAtlas0` 0x10077b790, `atlasTRexGreen0` 0x1004d8bf0, `atlasTRexGreen1` 0x1004f0538;
every slot's address is `atlases[].address` in the JSON.

### 5.2 Loading an image into a slot

`FUN_100038d68(file, ext, atlas)` resolves `<file>.raw` in the bundle (the ext argument is
ignored) and calls `activateTextureAtlas` (`FUN_100038b68`): read the whole file, bind the
slot (cached bind `FUN_10013f9e8`), `glTexImage2D(GL_TEXTURE_2D, 0, fmt, W, H, 0, fmt, type,
data)` with the slot's format and size (PVRTC: `glCompressedTexImage2D` with the file
length), `glFlush`, reset the bind cache (`FUN_10013fa1c`). No zlib, no alt file, no cache
directory on this path; texture parameters are unchanged.

`BCView activateTextureAtlasNextFrame:fileName:ext:` (0x1001776d8) queues up to 10 loads
(static array at 0x100821e58, count 0x100821e50; extra requests are dropped);
`BCView::gameLoop` (0x100177730) runs the queue at the start of the next frame, **last
queued first**, before input and `onGameFrame:`.

Sequence handlers (params are `BCSequenceItemDef` positional strings; all return 2):

| Handler | Params | Effect |
|---|---|---|
| `BCLibrary::loadTextureAtlas:::` 0x100024418 | 0 ext (ignored), 1 file, 2 slot id, 3 next-frame flag (non-NULL) | load now, or queue for the next frame. In 1.48.2 every use targets `screenAtlas0` (ScreenTextureAtlas0 when screens open; 14 idol screens use the next-frame form; help pages load the page's island atlas) |
| `PocketGodViewController::loadTextureAtlasChannel:::` 0x10007d3bc | 0 channel name | `"Disco"` → channel 2, `"Xenu"` → channel 3 (§5.4); any other name does nothing |
| `PocketGodViewController(Game)::loadLocationTextureAtlas:::` 0x10015a118 (also on DebugScreen, ChallengeManager, StoryManager, NewsManager, SkinPackManager, BonusPackManager, IdolScreenLogic via their listener) | – | `loadLocationTextureAtlas` |
| `PocketGodViewController::refreshSkinPackOption:::` 0x10007d468 | 0 pack id, 1 option id, 2 object id | `refreshSkinPackOption:skinObjectId:skinOptionId:useFirstPurchased:` (p0, p2, p1, YES) |
| `BCLibrary::setTextureDef:::` 0x1000258b4 | 0 texture id, 1 display group id (NULL = active group), 2 display object id | `setTextureDef:` on that display object |
| `BCLogic::setTextureDef:::` 0x100138edc | as above; when param 2 is NULL: stop the logic's animation and set the texture of the logic's own display object | |

### 5.3 The 26 slots

`channel` is the build-time `channelId` (§3); slots of one channel are its pages.
Def counts include the slot's own def.

| Slot | Size | Fmt | Channel | Images that can be in it | Defs | Loaded |
|---|---|---|---|---|---|---|
| atlas0 | 1024² | 4444 | MainAtlas p0 | TextureAtlas0, MainBattleAtlas0 | 699 | loadGame; MainBattleAtlas0 in `showBattleOfTheGodsDisplayGroup` (0x100167d18); TextureAtlas0 again in `hideBattleOfTheGodsDisplayGroup` (0x100167e88, unless an idol screen is open) and `onIdolScreenClosed` (0x100078b28, after battle results) |
| atlas1 | 1024² | 4444 | MainAtlas p1 | TextureAtlas1 | 623 | loadGame |
| screenAtlas0 | 512² | 4444 | LocationAtlas | 27 island/screen atlases | 1967 | `loadLocationTextureAtlas`, map, screen sequences (below) |
| atlasTRexGreen0 | 256² | 4444 | MonsterAtlas p0 | 55 monster, statue, idol-page, Disco, Xenu images | 754 | monster skin options, statue option, `loadTextureAtlasChannel:` |
| atlasTRexGreen1 | 256² | 4444 | MonsterAtlas p1 | 26 images (page-1 files, IconPage0, IdolPage0/1) | 511 | same |
| atlasSharkDefault0 | 256² | 4444 | SharkAtlas | 11 TextureAtlasShark*0 | 349 | "Shark Skins" option (loadGame refreshes it) |
| atlasHalloweenExtras0 | 256² | 4444 | ExtrasAtlas | 16 *Extras0 | 213 | "<Island> Extras" options from `loadLocationTextureAtlas` |
| atlasPygmyUpdateSong0 | 256² | 4444 | PygmyDanceAtlas | 22 dance atlases | 923 | "Pygmy Dance" option (applied at init) |
| atlasPygmyBadZombieDance0 | 256² | 4444 | ZombieDanceAtlas | PygmyBadZombieDance0, ZombieThriller0, ZombieSmoothCriminal0 | 144 | "Zombie Dance" option |
| atlasVampireStayingAlive0 | 256² | 4444 | – | VampireStayingAlive0 | 43 | init |
| EpisodeAlertAtlas0 | 256² | 4444 | – | EpisodeAlertTextureAtlas0 | 26 | init; idol bits slots |
| NewsAtlas0 | 128² | 4444 | – | NewsAtlas0 | 2 | init; news icon bits |
| atlasTest0-6 | 512² | 4444 | – | TextureAtlasTest0-6 | 207,152,176,219,288,497,170 | init; bits skins patch them |
| widescreenOverlayAtlas0 | 256×512 | 4444 | – | WidescreenOverlayAtlas0 | 5 | loadGame on wide/tall screens |
| LocustAtlas0-2 | 16² | 4444 | – | LocustTextureAtlas0-2 | 2 each | init |
| pvratlas0-2 | 1024² | PVRTC4 | – | TextureAtlasPvr0-2 | 80,194,89 | init |

Which def samples which image is the `image` field of each def (§9.3 explains how it was
derived). Images per slot are `atlases[].images`.

`PocketGodViewController::loadLocationTextureAtlas` (0x10007cbc8). Does nothing while any
of these screens is open: map, stats, what's new, skin pack, bonus pack, story (unless a
story is playing), news, help, debug. Otherwise the first matching mini-game flag wins:

| Flag | Image |
|---|---|
| `mbShowDoodleAssets` | OogaJumpTextureAtlas0 |
| `mbShowPainDrainAssets` | PainDrainTextureAtlas0 |
| `mbShowTheRunsAssets` | TheRunsTextureAtlas0 |
| `mbShowKonkeyDongAssets` | KonkeyDongTextureAtlas0 |
| `mbShowMoronPestsAssets` | MoronPestsTextureAtlas0 |
| `mbShowNoPeeAssets` | NoPeeTextureAtlas0 |
| `mbShowBattleOfTheGodsAssets` | BattleTextureAtlas0 |

else by `mIslandLocationState` (all into `screenAtlas0`; skin refreshes run first, with
`useFirstPurchased: YES` and pack argument `Halloween`, see §7.4):

| State | Image | Also |
|---|---|---|
| 0 | SandTextureAtlas0 | refresh "Sand Island Statue"/"Statue" and "Sand Island"/"Sand Island Extras" |
| 1 | RockTextureAtlas0 | refresh "Rock Island Extras" |
| 2 | UnderwaterTextureAtlas0 | |
| 3 | GraveyardTextureAtlas0 | refresh "Graveyard Island Extras" |
| 4 | IceTextureAtlas0 | refresh `IceMonster`/"Ice Monster"/"Ice Monster", then "Ice Island Extras" |
| 5 | ApeIslandTextureAtlas0 | refresh "Ape Island Extras" |
| 6 | MisfitTextureAtlas0 | |
| 7 | GodTextureAtlas0 | first `loadTextureAtlasChannel: 1` and `updateIdolTexture` on every GodIdolLogic |
| 8 | BoatTextureAtlas0 | |
| 9 | UnderwaterCoralReefTextureAtlas0 | |
| 10 | ApocalypseTextureAtlas0 | |
| 11 | TimeTempleTextureAtlas0 | |
| 12 | LocustTempleTextureAtlas0 | |
| 13 | DanceTempleTextureAtlas0 | first `loadTextureAtlasChannel: 2` (Disco) |
| 14 | PlagueTempleTextureAtlas0 | |
| 15 | PurificationTempleTextureAtlas0 | |
| 16 | ClosetTempleTextureAtlas0 | |
| 17 | XenuTempleTextureAtlas0 | first `loadTextureAtlasChannel: 3` (Xenu) |

Callers: `activateIslandLocationState:` (0x100080fa8), the show/hide functions of the
Doodle, PainDrain, TheRuns, KonkeyDong, MoronPests, NoPee, BattleOfTheGods and
ApocalypseTemple display groups (0x100164104 ... 0x1001684e8), `onLoadLocationTextureAtlas`
(listener callback) and the sequence handler. `MapTextureAtlas0` is loaded into
`screenAtlas0` by `MapScreen::onMapPaperLogicShowMap` (0x1000c1050) and
`NewsManager::onMapPaperLogicShowMap` (0x100055e74); `ScreenTextureAtlas0` by the
screen-opening sequences.

### 5.4 Runtime channel ownership

`SkinPackManager.mSkinOptionChannelDictionary` maps a channel name to the id of whatever
currently owns that slot set: a skin option id, or `IconPage`/`IdolPage`/`Disco`/`Xenu`
for `MonsterAtlas`. A skin value is only re-applied when it is not already the owner
(§7.4), and a channel user that overwrites the slot takes ownership, so the next refresh of
the monster option reloads it.

`SkinPackManager::loadTextureAtlasChannel:` (0x10005cec0), all into the monster slots,
each case skipped if `channels["MonsterAtlas"]` already names it:

| Arg | Owner | `atlasTRexGreen0` | `atlasTRexGreen1` | Callers |
|---|---|---|---|---|
| 0 | IconPage | IconPageTextureAtlas0 | IconPageTextureAtlas0 | `IdolScreenLogic::openIdolListPanel:` 0x1001925c4 |
| 1 | IdolPage | IdolPageTextureAtlas0 | IdolPageTextureAtlas0 | idol selection / battle enemy / battle results panels (0x100192b28, 0x100192e9c, 0x100193268), `activateIslandLocationState:`, `setGodIslandIdolsState:startup:` (0x1000866c8), location 7 |
| 2 | Disco | DiscoTextureAtlas0 | DiscoTextureAtlas1 | location 13; sequences HelpPage51/52 (`loadTextureAtlasChannel:::`) |
| 3 | Xenu | XenuTextureAtlas0 | XenuTextureAtlas1 | location 17 |

`PocketGodViewController(Game)::loadMonsterTextureAtlas` (0x100169bf0) restores the
monster slots after screens that used them (`closeScreens` 0x1000775a8,
`onSkinPackScreenClosed` 0x100077d6c, `onIdolScreenClosed`, a button in `onButtonPress:`
0x100079090): the first enabled of Spider, Ape, Locust Queen, Ice Monster, T-Rex is
refreshed (`Spider/Spider/Spider`, `ZombiePack/Ape/Ape`, `ZombiePack/Locust Queen/Locust
Queen`, `IceMonster/Ice Monster/Ice Monster`, `TRex/Tyrannosaurus Rex/TRex Body`, as
pack/object/option, `useFirstPurchased: YES`).

## 6. Draw time

`FUN_10003841c(def, displayObject)` (the quad draw used by `BCModel::drawOpenGL:`
0x1001187e8, `BCModel drawInMultiModelOpenGL:...`, `BCMultiModel` and `BCFontDef`):

1. NULL def → nothing.
2. bind `*def->glNamePtr`; vertex pointer = `def->vertices` (3 floats); texcoord pointer =
   `def->uvs` (2 floats).
3. `GL_BLEND` on iff `def->has_alpha` or `displayObject.alpha < 1.0` (blend function is
   always `ONE, ONE_MINUS_SRC_ALPHA`).
4. `glDrawElements(GL_TRIANGLE_STRIP, 4, GL_UNSIGNED_SHORT, {0,1,2,3})`: triangles
   (TL,TR,BL) and (TR,BL,BR). Texture env is the default `GL_MODULATE`.

Before the call `BCModel::drawOpenGL:` sets `glColor4f(r·a, g·a, b·a, a)` from the display
object's mRed/mGreen/mBlue/mAlpha and builds the model matrix (pos with the
`-277.12 / pos.z` perspective factor, pivot, rotation, skew, scale, origin; see the display
object spec). The vertices are model-space pixels with y up, so row 0 of an image ends up
at the top of the sprite (`v0` goes with `t`). With `has_alpha = false` and alpha 1 the quad
overwrites the framebuffer (transparent texels become black).

Animated multi-models (`BCMultiModel::drawOpenGL:` 0x10018198c, frames from
Timelines.bin) bind the frame part's def texture and use the def's `uvs`, but take the
4 vertex positions from the frame state record (+0x5c) instead of `def->vertices` (see
`docs/re/timelines.md`).

Because models keep a pointer to the def, the remap step of a skin (§7.2) changes every
model that uses the base def at once.

## 7. Skins

### 7.1 Structures

Table at 0x10037d620: 24 entries `{BCSkinPackDef *pack; NSString *keychainKey}`
(`SkinPackManager::initSkinPackDefs` 0x10005d694, `getSkinPackDefById:` 0x10005ce4c,
`getSkinPackDefByProductIdentifier:` 0x10005cdd8). Objects, options and values are
separate copies per pack (290 objects, 294 options, 5149 values); the lists they point at
are shared (42 remap lists, 248 bits record lists, 67 atlas-image lists, 132 alias lists).

`BCSkinPackDef {@@@@@@@@ B @ ii ^^^BCSkinObjectDef}`:

| Off | Field | JSON |
|---|---|---|
| 0x00 | id (`"TRex"`) | `id` |
| 0x08 | display name | `name` |
| 0x10-0x28 | purchase panel lines 0-3 | `buy_text[4]` |
| 0x30 | success panel line | `success_text` |
| 0x38 | App Store product id; `"FreeBee"` = free (Fishmas) | `product_id`, `free` |
| 0x40 | BOOL purchased (runtime: FreeBee, or keychain secret == `"BabaBooey"`) | – |
| 0x48 | BCKeyChain* (app `"PocketGod"`, key = table keychainKey) | `keychain_key` |
| 0x50 | int object count | – |
| 0x54 | int current object index (runtime) | – |
| 0x58 | `BCSkinObjectDef**` objects | `objects` |

`BCSkinObjectDef` (0x20): +0x00 id (also the display name, e.g. `"Tyrannosaurus Rex"`),
+0x08 behavior set on the customize preview logic (`behavior`), +0x10 int option count,
+0x14 unused, +0x18 `BCSkinOptionDef**`.

`BCSkinOptionDef` (0x30): +0x00 id (e.g. `"TRex Body"`; keychain key holding the selected
value index, default `"0"`), +0x08 BCKeyChain* (runtime), +0x10 BOOL `activate_at_init`,
+0x18 NSString* `channel` (`MonsterAtlas`, `SharkAtlas`, `ExtrasAtlas` or NULL), +0x20 int
value count, +0x24 int current value index (runtime), +0x28 `BCSkinOptionValueDef**`.

`BCSkinOptionValueDef` (0x58), applied by `FUN_100038f24`:

| Off | Field | JSON |
|---|---|---|
| 0x00 | id / display name | `id` |
| 0x08 | pack that must be purchased (NULL = default value or this pack's own item) | `required_pack` |
| 0x10 | behavior for the customize preview (`setBehavior:`), may be NULL | `behavior` |
| 0x18 | `.raw` loaded into the slot at +0x20 | `atlas_image` |
| 0x20 | BCAtlasTextureDef* | `atlas` |
| 0x28 | NULL-terminated `{BCTextureDef *target, *source}` pairs | `remap_set` (index into `remap_sets`) |
| 0x30 | bits `.raw` name | `bits_image` |
| 0x38 | int bits format (5) | – (always 5) |
| 0x40 | `BCSkinTextureBitsDef[]` records, NULL-terminated | `textures.json` `images[bits_image].regions` |
| 0x48 | `{NSString *file; int format; BCTextureDef *slotDef}[]`, ends with slotDef == NULL | `atlas_images` |
| 0x50 | `{alias, animation, displayObject, behavior, displayGroup}` NSString records, ends with alias == NULL | `anim_alias_set` (index into `anim_alias_sets`) |

`BCSkinTextureBitsDef {^BCTextureDef; iiii}` (0x18): texture def, then x, y, width, height
of the destination rect in that def's slot (top-left origin, texels). Every record in the
game covers exactly its def's texel rect (checked).

v1.45.3 XML (texture_test.xml): `<skinPack id name productId buyText0-3 successText0>`,
`<skinObject id name sequenceId>`, `<skinOption id name activateAtInit skinOptionChannel>`,
`<skinOptionValue textureSkinId|animSkinId name packId sequenceId atlasFileName atlasId>`.
A `textureSkinId` names a `<skin id filename>`: either a bits file (its textures become
regions over the `skinId` base defs) or, for skin bases with `allTexturesInAtlas="1"`, a
whole-atlas swap plus a remap of every `<skinBase>` texture (`skinTexture`,
`skinTextureSequence count` → `<id>01..NN`) onto the skin's own defs.
`<animSkin textureAtlasId>`/`<animSkinItem skinId id displayObjectId sequenceId
displayGroupId>` became `atlas_images` + `anim_aliases`. In 1.48.2 object and option ids
equal their display names.

### 7.2 Applying a value (`FUN_100038f24`)

In this order:

1. **Remaps**: for each pair copy bytes 0x10-0x7f of `source` over `target` (slot pointer,
   GL-name pointer, sizes, UVs, vertices); `id` and `has_alpha` stay.
2. **Bits** (if +0x40 non-NULL): `activateSkinTextureBitsBatchDef` (`FUN_1000384b0`) reads
   `<bits_image>.raw` from the bundle; for each record: bind the record def's slot,
   `glTexSubImage2D(GL_TEXTURE_2D, 0, x, y, w, h, GL_RGBA, GL_UNSIGNED_SHORT_4_4_4_4, p)`,
   `p += w·h·2`.
3. **Atlas images** (if +0x48 non-NULL, `FUN_100038a20`): for each record load
   `<file>.raw` and `glTexImage2D` it as RGBA4444 at the size of the record def's slot
   (the record's format field is ignored).
4. **Animation aliases** (if +0x50 non-NULL, `FUN_100038dc0`): `setAnimationDefAlias:alias
   animationDef:getAnimationDefById:animation`; if `displayGroup != @""`:
   `[[[group(displayGroup) getDisplayObject:displayObject] logic] setBehavior:behavior]`.
5. **Atlas image** (if +0x18 non-NULL): `activateTextureAtlas(<atlas_image>.raw, atlas)`.

What the options use:

| Option (object) | Channel | Init | Values | Mechanism |
|---|---|---|---|---|
| Meteor, Dodo Bird, Doodle Jump, Egg (T-Rex Egg), Coconut, Outhouse, Spear, Volcano, Underwater Statue, Moon, Fish, Igloo, Monkeys, Laser Skins (Shark), TRex Hat, TRex Tattoo, Pine Tree | – | yes | 2-27 | bits into the `atlasTest0-6` slots |
| Shark Skins | SharkAtlas | no | 11 | atlas images + remap set |
| TRex Body, Ice Monster, Spider, Locust Queen, Ape | MonsterAtlas | no | 2-10 | atlas images (pages 0 and 1) + remap set |
| Statue (Sand Island Statue) | MonsterAtlas | no | 20 | atlas image into `atlasTRexGreen0` + animation aliases |
| Sand/Rock/Graveyard/Ice/Ape Island Extras | ExtrasAtlas | no | 17 | atlas image into `atlasHalloweenExtras0` + animation aliases |
| Pygmy Dance, Zombie Dance | – | yes | 22, 3 | atlas images + animation aliases |
| Vampire Dance | – | yes | 2 | animation aliases only |

Base defs (remap targets) are emitted right before the default skin's copy
(`TRex_Head_01`, `Green_TRex_Head_01`), so before any skin is applied they already show the
default skin. A shared option (e.g. "Meteor" in 15 packs) has the same value order in
every pack; only `required_pack` differs (NULL for the pack's own values), which keeps the
keychain-stored index (keyed by option id) meaningful in every pack.

### 7.3 Bits slots used outside skin packs

`textures.json` `bits_slots`: static `BCSkinTextureBitsDef` arrays filled by
`activateSkinTextureBitsBatchDef` `FUN_100038648(records, file, @"raw")` (same upload as
step 2, but the type is `GL_UNSIGNED_SHORT_4_4_4_4` only if the record's slot format is 5,
otherwise `GL_UNSIGNED_BYTE`; the data pointer always advances by w·h·2):

- icon slots `EpisodeIconSlot1-9` (84×84, IconPage, `atlasTRexGreen0`),
  `EpisodeAlertIconSlot` (EpisodeAlertAtlas0): an episode's icon raw
  (`BCStructXmlEpisodeNodeDef` +0x60, `icon_texture` in `episodes.json`, files
  `EpisodeIcon*`);
- idol slots `EpisodeIdolSlot1-4` (`atlasTRexGreen1`), `5-8` (`atlasTRexGreen0`),
  `EpisodeAlertIdolSlot`: the idol raw (+0x68, `idol_texture`, files `EpisodeIdol*`).
  GodIdolLogic objects 99-102 use idol slots 1-4 (`GodIdolLogic::updateIdolTexture`
  0x100119038). `used_by` in the JSON lists every user of each array.

News icon: the stand-alone value at 0x1005f96e0 (`skin-packs.json` `standalone_values`)
has one record `NewsIcon` (0,0,64,64) in `NewsAtlas0`. `NewsManager::updateNewsIcon:`
(0x100055fcc) writes the news def's icon name (+0x38) into the value's bits-file field
(0x1005f9710) and uploads it from the bundle (`FUN_1000384b0`) if `<name>.raw` exists
there, else from `Library/Caches/<name>.raw` (`activateSkinTextureBitsBatchDefLibraryCache`
`FUN_100038800`), else uploads `DefaultNewsIcon` and downloads the icon from the server.

### 7.4 When values are applied

`updateSkinPackDef:skinOptionIndex:skinOptionValueIndex:silent:init:` (0x1000601e4), on
the pack's current object:

```
option = pack.objects[pack.currentObject].options[optionIndex]
i = valueIndex mod option.count            (negative results wrap to positive)
if i == option.current and not silent: return YES
value = option.values[i]
if value.requiredPack and (not packById(value.requiredPack).purchased or not pack.purchased): return NO
option.current = i
key = "%d" of i
alreadyApplied = state[option.id] == key and channels[option.channel] == option.id
if not alreadyApplied and (not init or option.activateAtInit):
    state[option.id] = key
    if option.channel: channels[option.channel] = option.id
    apply(value)                                   # §7.2
    if value.behavior: customizePreview.setBehavior(value.behavior)
if not silent:
    if pack.purchased: option.keychain.secret = key
    run sequence "UpdateSkinOptionValue"
return YES
```

(For an option without channel `channels[nil]` is nil, so a silent refresh always
re-applies it.)

- `initSkinPackDefs` (from `SkinPackManager::init` 0x10005cad4): for every pack, every
  object, every option: read the keychain index (default `"0"`) and update with
  `silent: YES, init: YES` → only `activate_at_init` options are applied at startup.
- `refreshSkinPackOption:skinObjectId:skinOptionId:useFirstPurchased:` (0x10005d450): the
  first **purchased** pack in table order that has that object and option replaces the
  pack argument (the BOOL is never read); then
  `refreshSkinPackOption:skinObjectId:skinOptionId:` (0x10005d57c) sets the object as
  current and updates with the option's current index, `silent: YES, init: NO`.
- `activateSkinPackDef:` (0x1000605b0, opening a pack in the skin screen) re-applies every
  option of the pack (index from the keychain if the pack is purchased, else 0,
  `silent: YES, init: NO`), then runs `OpenSkinPackScreenCustomize`.
- `clearUnpurchasedSkinPackDef:skinOptionIndex:` (0x10006042c), for a pack that is not
  purchased: sets the option's current index from its keychain value and, if
  `state[option.id]` differs from it and the option has no channel or owns its channel,
  stores the state and re-applies that value (plus its preview behavior).
- loadGame refreshes `Shark/Shark/Shark Skins`; island and monster refreshes are in §5.

## 8. Port notes

- Keep one CPU-side image per slot (26). Loading an image replaces it; bits uploads copy
  sub-rects (premultiplied RGBA8 from the PNGs, with `regions[].src_*`) into it; re-upload
  after change. Track the GL "name" by slot id, not by image.
- Keep the 8337 defs in a mutable table indexed by id; models store the index. A remap
  copies `atlas`, sizes, `uvs`, `vertices` from source to target.
- Draw each def as a 4-vertex strip with its `vertices` and `uvs`, linear filtering,
  premultiplied blending, colour `(r·a, g·a, b·a, a)`; disable blending when
  `has_alpha` is false and alpha is 1.
- Treat a slot that has never received an image as empty (the original leaves the GL
  texture without storage).

## 9. JSON schema

Both files: UTF-8, keys in fixed order, lists in binary order, absent values `null`,
floats are the shortest decimal that round-trips to the same `f32`.

### 9.1 `textures.json`

```
{ format_version: u32 (1), premultiplied_alpha: bool (true),
  atlases: [Atlas], textures: [TextureDef], images: [RawImage], bits_slots: [BitsSlot] }

Atlas { id: String, address: String ("0x..."), width: u32, height: u32,
  format: String (§2.1 name), format_code: i32, has_alpha: bool, channel: Option<String>,
  page: u32, init_image: Option<String>, ext: Option<String>, alt_image: Option<String>,
  alt_format_code: i32, zlib: bool, from_caches_dir: bool, no_init: bool, mipmap: bool,
  images: Vec<String> }                 // every atlas-kind image that can be loaded into it

TextureDef { id: String, atlas: String, image: String,   // image sampled by default
  has_alpha: bool, x: i32, y: i32, width: i32, height: i32,
  original_width: i32, original_height: i32, empty: bool,
  uvs: [f32; 8], vertices: [f32; 12] }

RawImage { name: String, png: String ("textures/<name>.png"), raw_size: u64,
  kind: String ("atlas" | "bits"), format: String, width: u32, height: u32,
  atlases: Vec<String>,                 // atlas: slots it is loaded into; bits: slots it patches
  regions: Vec<Region>,                 // empty for atlas
  loaded_by: Vec<String>,               // human-readable call sites / sequences / skin values
  referenced: bool }                    // false: never loaded by 1.48.2 (§2.4)

Region { src_x: u32, src_y: u32, width: u32, height: u32,   // rect in the PNG
  texture: Option<String>, atlas: Option<String>, x: Option<i32>, y: Option<i32> }
                                        // destination def / slot / texel; null = chosen at
                                        // run time (episode icons and idols, §7.3)

BitsSlot { name: String, address: String, used_by: String,
  regions: Vec<{ texture: String, atlas: String, x: i32, y: i32, width: i32, height: i32 }> }
```

### 9.2 `skin-packs.json`

```
{ format_version: u32, packs: [SkinPack], standalone_values: [StandaloneValue],
  remap_sets: [[Remap]], anim_alias_sets: [[AnimAlias]] }

SkinPack { id, address, name: String, buy_text: [String; 4], success_text: String,
  product_id: String, free: bool, keychain_key: String, objects: [SkinObject] }
SkinObject { id: String, behavior: String, options: [SkinOption] }
SkinOption { id: String, activate_at_init: bool, channel: Option<String>, values: [SkinValue] }
SkinValue { id: String, required_pack: Option<String>, behavior: Option<String>,
  atlas_image: Option<String>, atlas: Option<String>,        // +0x18 / +0x20
  remap_set: Option<usize>,                                 // +0x28
  bits_image: Option<String>,                               // +0x30, regions in textures.json
  atlas_images: [{ image: String, format_code: i32, atlas: String }],   // +0x48
  anim_alias_set: Option<usize> }                           // +0x50
Remap { target: String, source: String }
AnimAlias { alias, animation, display_object, behavior, display_group: String }  // "" = none
StandaloneValue { address: String, used_by: String, value: SkinValue }
```

### 9.3 How `TextureDef.image` was derived

The binary does not store which file a def's rect refers to (all island defs point at
`screenAtlas0`, all monster defs at `atlasTRexGreen0/1`). The tool:

1. splits each slot's defs (binary order) into runs: a run starts at every def whose rect
   is at texel (0,0), except a skin def that directly follows its identical base def
   (each packed image starts with its largest sprite at the origin);
2. assigns runs exactly from skin values: a value that loads image I into slot S and
   remaps base defs onto its own defs in S fixes those defs' run to I (all T-Rex, spider,
   ice monster, ape, locust queen and shark runs);
3. assigns the remaining runs greedily to the remaining candidate images of the slot by
   score = (share of the image's opaque texels inside the run's rects) × (mean tightness of
   the opaque bounding box in each rect); uniform images (DefaultSandStatue0) are excluded,
   ties prefer the image whose page number matches the slot.

All 177 runs got an image. 81 were fixed by skin values or had a single candidate; the
other 96 were matched by score (island/screen atlases, TextureAtlas0/MainBattleAtlas0,
statues, extras, dances, zombie dances, icon/idol pages, Disco, Xenu), all with a score
≥ 0.955, and every match agrees with the names (`IslandTexture2` → SandTextureAtlas0,
`SandStatueSurgeon` → AdultSwimSurgeonSandStatue0, ...). The tool warns about contested
matches below 0.9 and fails if a run stays unassigned.

## 10. Tool

```
tmp/re/venv/bin/python -I tools/re/extract_textures.py [--bundle DIR] [--binary FILE]
    [--data-dump FILE] [--assets DIR] [--no-png] [--report]
```

Defaults: bundle `tmp/pocket-god-v1.48.2/Payload/PocketGod.app`, binary
`tmp/re/PocketGod.arm64`, data dump `tmp/re/data_after_init.bin`, output `assets/`.
`--report` prints the run → image assignment. It asserts the invariants above (field
values, UV formula, GL-name pointers, bits rects, byte sizes of all 530 files) and is
deterministic (identical output on reruns). Runtime ≈ 10 s.

Visual checks done on the output: SandTextureAtlas0, TextureAtlas0, TextureAtlasPvr0/1,
TextureAtlasTRexMetal0, PygmyRobot0, ZombieThriller0, TextureAtlasSharkAlien0,
IconPageTextureAtlas0, MainBattleAtlas0, EpisodeIcon12, EpisodeIdol20, the news icons and
NewsAtlas0, several bits files (Dodo, PineTree, Igloo, SharkDefault, TRexSantaHat), and
sprites cut out with `x/y/width/height` and placed in their frames with `vertices`: all
upright, colours correct, text readable.

## 11. Open questions

- BCAtlasTextureDef +0x08 and +0x18 are never read by the loaders and are NULL in every
  def; their purpose (probably folder strings for the alt/library-cache path) is unknown.
- `traceDynamicAtlasTextureDefArray` (`FUN_100037f2c`) walks 40 entries of 0xd8 bytes at
  0x1007cfcb8 that overlap unrelated globals (`SkinPackManager` list index 0x1007d1e78,
  `BCView` instance 0x1007d1f40); the dynamic-atlas feature is gone and
  `BCLLDBProxy logDynamicAtlasTextureDefArray` is a debugger hook. Ignored by the port.
- Values with `required_pack == null` can be selected even when the pack itself is not
  purchased (the check only runs for non-null `required_pack`); whether the skin screen
  UI blocks this is for the skin-pack screen spec.
- What a slot shows before its first upload (incomplete GLES 1 texture, texturing
  effectively disabled) never matters in normal play; the port can draw nothing.
