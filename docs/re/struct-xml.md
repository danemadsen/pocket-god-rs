# Struct XML: the compiled XML node tables

Pocket God 1.48.2 has no data files for its display groups, sounds, fonts,
gradients, idols, dance charts, Konkey Dong levels, debug items or store links.
An offline tool compiled that XML into the executable: each XML element became a
static C struct (a "struct XML node") and each element list a NULL-terminated
array of node pointers. v1.45.3 still shipped some of the source XML
(`displaygroups.xml`, `sound.xml`, `texture.xml`, `texture_test.xml`), which
confirms the attribute names below.

- Extractor: `tools/re/extract_struct_xml.py` (no arguments needed; see its
  docstring). It writes the nine JSON files described in [JSON output](#13-json-output)
  to `assets/data/`, validates every runtime-only field and fails on unknown node
  types. Output is deterministic (data order, fixed key order, no timestamps).
- All addresses are in the arm64 slice `tmp/re/PocketGod.arm64`; data was read
  from `tmp/re/data_after_init.bin`. Function names follow `Class::selector @ address`.

Contents: [1 Inventory](#1-inventory) · [2 Conventions](#2-conventions-and-the-node-type-enum) ·
[3 parseStructXml](#3-bclibraryparsestructxml-0x100026674) · [4 Library node layouts](#4-library-node-layouts) ·
[5 parseDisplayObjectAttributes](#5-bclibraryparsedisplayobjectattributesdisplayobject-0x10002626c) ·
[6 Display-object trees](#6-how-display-groups-become-display-object-trees) · [7 Links](#7-store-links-types-37-38) ·
[8 Episodes](#8-episodes-battle-of-the-gods-idols-types-32-20-23) · [9 Dance charts](#9-dance-dance-execution-charts-types-33-35) ·
[10 Konkey Dong](#10-konkey-dong-levels-types-13-19) · [11 Debug items](#11-debug-items-type-36) ·
[12 Extractor checks](#12-what-the-extractor-validates) · [13 JSON output](#13-json-output) · [14 Open questions](#14-open-questions)

## 1. Inventory

| List (root) | Node types | Consumer | Contents (1.48.2) | JSON |
|---|---|---|---|---|
| `0x10071ab98` | 0 | `BCLibrary::parseStructXml:` call 1 | 359 sound defs (`sound.xml`) | `sound-defs.json` |
| `0x10064a270` | 12, 10 | `parseStructXml:` call 2 | 4 font defs, 15 gradient defs | `font-defs.json`, `gradient-defs.json` |
| `0x1006fe1d0` | 2, 3, 4, 6, 7, 8, 9, 11 | `parseStructXml:` call 3 | 229 display groups, 4842 children (`displaygroups.xml`) | `display-groups.json` |
| `0x1007162f0` | 38, 37, 2, 4, 6, 7, 8, 9, 11 | `parseStructXml:` call 4 | 5 links + 169 country links, then 37 display groups with 318 children (menus, islands, overlays) | `links.json`, `display-groups.json` |
| `0x1008162c0` | none | `parseStructXml:` call 5 | empty (see below) | `display-groups.json` |
| `0x100649c68` | 13–19 | `KongStatueLogic::parseStructXml @ 0x100187f04` | 5 Konkey Dong levels (123 nodes) | `konkey-dong-levels.json` |
| `0x1007306f8` | 32 (+20–23 in sub-lists) | `IdolManager`, idol screens, `PygmyLogic(BattleOfTheGods)` | 49 episodes / idols | `episodes.json` |
| `0x10074f818` | 35 (+34, 33 in sub-lists) | `DanceDanceExecutionScreenLogic::manageDanceDanceSymbols: @ 0x100035684` | 80 dance groups, 87 segments, 1684 beats | `dance-beats.json` |
| `0x100730938` | 36 | `DebugScreen::updateDebugItems @ 0x100008c74` | 11 debug item names | `debug-items.json` |

The five library roots are passed, in this order, by
`PocketGodViewController::loadGame @ 0x10006a030` (root addresses formed at `0x10006a2c8`,
`0x10006a2dc`, `0x10006a2f0`, `0x10006a304`, `0x10006a318`), after
`processAtlasTextureDefs:textureDefs:` and `processSequenceDefs:` and before
`parseBinAnim:ext:`.

The fifth root `0x1008162c0` is an 8-byte zero-fill global in `__common`; the words after
it (`0x1008162c8`–`0x1008162d0`) are `BCView::processTouchEvents:` /
`createTouchEvents:withEvent:` touch state, so the root is a one-entry list holding only
the NULL terminator. Nothing writes it: the three module initializers are `0x1000d8290`
(sequence-item tables, last write `0x1008161e8`), `FUN_1001b3a58` (Crashlytics
`mach_timebase_info`) and `FUN_1002875a8` (Flurry static containers at
`0x1007e2be8`–`0x1007e2c30`); the only code reference to the address is `loadGame`
(adrp/add at `0x10006a318`); no data word points to it; and it is zero in the
post-initializer dump. The call reads a NULL first entry and returns immediately, so the
port can ignore it.

`IdolManager` also references four episode nodes directly by address: Episode1
`0x10071b8b0`, Episode2 `0x10071bb68`, Episode39 `0x100728a18` and Episode40 `0x100729760`
(see [8.7](#87-consumers)). The port must look these up by id.

## 2. Conventions and the node type enum

- Each node is a C struct whose first member is `int type`. Pointer-sized members are
  8-byte aligned, so a 4-byte pad follows `type` whenever the next member is a pointer.
- A list is an array of node pointers terminated by a NULL pointer. Sub-lists (episode
  challenges, link countries, dance segments/beats) are separate arrays that a node
  points to (`T **`).
- `id` members are `NSString` literals (`__cfstring`) or NULL. All strings in these
  tables are ASCII.
- The XML compiler converted the two-character escape `\n` into a real newline
  (byte 0x0a): the 51 `<text>` elements that differ between 1.45.3 `displaygroups.xml`
  and 1.48.2 only by this have a literal `\n` in the XML and 0x0a in the binary.
  `true`/`false` became 1/0.
- Members the game writes at runtime (selection slots, counters, logic pointers) are part
  of the same structs and are zero in the data (the extractor checks this). The port keeps
  that state elsewhere.

Node type values (from the `switch` statements of the consumers):

| Value | Name used here | XML element / meaning | Layout section |
|---:|---|---|---|
| 0 | `SOUND` | `<sound>` | [4.1](#41-sound-type-0-size-0x28) |
| 1 | `DISPLAY_OBJECT` | plain `BCDisplayObject` (unused in 1.48.2) | [4.5](#45-display-node-variants-types-19-11) |
| 2 | `DISPLAY_GROUP` | `<displayGroup>` (top level) | 4.5 |
| 3 | `EMPTY_DISPLAY_GROUP` | `<emptyDisplayGroup>` (nested) | 4.5 |
| 4 | `DISPLAY_MARKER` | `<displayMarker>` | 4.5 |
| 5 | `RECT` | rect (unused in 1.48.2) | 4.5 |
| 6 | `MODEL` | `<model>` | 4.5 |
| 7 | `MULTI_MODEL` | `<multiModel>` | 4.5 |
| 8 | `TILE_MODEL` | `<tileModel>` | 4.5 |
| 9 | `GRADIENT` | `<gradient>` | 4.5 |
| 10 | `GRADIENT_DEF` | `<gradientDef>` | [4.3](#43-gradient-def-type-10-size-0x18) |
| 11 | `TEXT` | `<text>` | 4.5 |
| 12 | `FONT_DEF` | `<addFontToAtlas>` | [4.2](#42-font-def-type-12-size-0x30) |
| 13–19 | `KONKEY_DONG_*` | level, kong pos, pygmy start pos, platform, ladder, pick, beetle | [10](#10-konkey-dong-levels-types-13-19) |
| 20–23 | `EPISODE_*` | enemy idol, realm power, challenge, level-up | [8](#8-episodes-battle-of-the-gods-idols-types-32-20-23) |
| 24–31 | — | never used by any consumer or data | — |
| 32 | `EPISODE` | idol / episode | 8 |
| 33–35 | `DANCE_*` | beat, segment, group | [9](#9-dance-dance-execution-charts-types-33-35) |
| 36 | `DEBUG_ITEM` | debug screen entry | [11](#11-debug-items-type-36) |
| 37 | `COUNTRY_LINK` | country → URL | [7](#7-store-links-types-37-38) |
| 38 | `LINK` | store link | 7 |

## 3. `BCLibrary::parseStructXml:` (0x100026674)

Signature `-(void)parseStructXml:(BCStructXmlNodeDef **)list`. Verified against the
disassembly (jump table at `0x100026d60`, 13 entries). State: `current` = the current
top-level `BCDisplayGroup` (starts nil), and a stack array of up to 200
`(marker, linkId)` pairs for deferred marker links (`local_d08[400]`, no bounds check).

```
current = nil; pending = []
for node in list (until NULL):
    switch node.type:
    0  SOUND:        s = [[BCSound alloc] initWithId:+0x08 fileName:+0x10 ext:+0x18
                            loop:+0x20 stream:+0x22 multipleInstances:+0x21 preload:NO]
                     [self addSound:s]; [s release]
    1  DISPLAY_OBJECT: o = [[BCDisplayObject alloc] initWithId:+0x08]          -> ADD_CHILD
    2  DISPLAY_GROUP:  g = [[BCDisplayGroup alloc] initWithId:+0x08]
                       parseDisplayObjectAttributes(node, g)
                       [self addDisplayGroup:g]; [g release]; current = g
    3  EMPTY_DISPLAY_GROUP:
                       g = [[BCDisplayGroup alloc] initWithId:+0x08]
                       parseDisplayObjectAttributes(node, g)
                       [current addDisplayObject:g]; [self addDisplayGroup:g]; [g release]
                       (current is NOT changed)
    4  DISPLAY_MARKER: m = [[BCDisplayMarker alloc] initWithId:+0x08]
                       parseDisplayObjectAttributes(node, m)
                       if +0x88 (linkId) != nil: pending.append((m, linkId))
                       [current addDisplayObject:m]; [self addDisplayMarker:m]; [m release]
    5  RECT:          r = [[BCRect alloc] initWithId:+0x08 minXPos:+0x84 minYPos:+0x88
                                           maxXPos:+0x8c maxYPos:+0x90]
                       parseDisplayObjectAttributes(node, r)
                       [current addDisplayObject:r]; [r release]
                       FALLS THROUGH into case 2 (no break): a new top-level group with the
                       rect's id and attributes is created, registered and becomes current.
    6  MODEL:         tex = (+0x88 != nil) ? [self getTextureDefById:+0x88] : NULL
                       o = [[BCModel alloc] initWithId:+0x08 textureDef:tex]          -> ADD_CHILD
    7  MULTI_MODEL:   o = [[BCMultiModel alloc] initWithId:+0x08]                    -> ADD_CHILD
    8  TILE_MODEL:    o = [[BCTileModel alloc] initWithId:+0x08 wrapWidth:+0x84 wrapHeight:+0x88] -> ADD_CHILD
    9  GRADIENT:      def = [self getGradientDefById:+0x88]
                       o = [[BCGradient alloc] initWithId:+0x08 width:+0x90 height:+0x94 gradientDef:def] -> ADD_CHILD
    10 GRADIENT_DEF:  def = [BCGradient createGradientDef:+0x10]
                       [self addGradientDef:def id:+0x08]        (no release; def is malloc'd)
    11 TEXT:          font = [self getFontDefById:+0x88]
                       t = [[BCText alloc] initWithId:+0x08 fontDef:font]
                       parseDisplayObjectAttributes(node, t)
                       if +0x90: [t setFontJustification:2]      (center)
                       if +0x91: [t setFontJustification:1]      (right; wins over center)
                       [t setText:+0x98]; [current addDisplayObject:t]; [t release]
    12 FONT_DEF:      f = [[BCFontDef alloc] initWithId:+0x08]; [f setCharSet:+0x28]
                       [self addFontDef:f]; [f release]
    default (13-38): ignored
    ADD_CHILD: parseDisplayObjectAttributes(node, o); [current addDisplayObject:o]; [o release]

for (m, linkId) in reversed(pending):           # last marker first
    [m addDisplayObject:[self getDisplayGroupById:linkId]]
```

Consequences the port must reproduce:

- **Children attach to the current top-level group.** A display node that appears
  before the first type-2 node of a call is created and released without a parent
  (messages to nil are no-ops); types 3 and 4 are still registered in the library and a
  marker's link is still resolved. 1.48.2 has no such orphans.
- **Empty groups are leaves.** Type 3 never becomes `current`; nodes after it go to the
  enclosing top-level group. The only one is `TheRunsDemonDisplayGroup`.
- **Marker links are resolved after the whole call**, against the library dictionary, in
  reverse marker order. 48 of the 89 linked markers in 1.48.2 are forward references to
  groups defined later in the same call; call 4 markers link to two groups from call 3
  (`HelpContent → About`, `IslandDisplayMarker → RockIslandDisplayGroup`). A missing
  group gives `[m addDisplayObject:nil]`, which leaves the marker empty. All links resolve
  in 1.48.2.
- **Rect fallthrough.** Type 5 is not present in 1.48.2, so the bug never triggers; the
  extractor mirrors it anyway.
- **Types 37/38 and every type ≥ 13** are skipped here; other consumers read them.
- Textures, gradient defs and fonts are resolved **at parse time**. Root 2 (fonts,
  gradient defs) is parsed before roots 3–4 that use them, and the atlas texture defs
  come from `processAtlasTextureDefs:textureDefs:` earlier in `loadGame`.

## 4. Library node layouts

### 4.1 Sound (type 0, size 0x28)

| Off | Type | Field | XML (`sound.xml`) | Use |
|---|---|---|---|---|
| 0x00 | int | 0 | `<sound>` | |
| 0x08 | id | id | `id` | `BCSound.mId`, key in `mSoundDictionary` |
| 0x10 | id | file name | `filename` | stored as `@"sound/<fileName>"` |
| 0x18 | id | extension | `ext` | `pathForResource:@"sound/<fileName>" ofType:ext` |
| 0x20 | BOOL | loop | `loop` | `mbLoop` (39 sounds) |
| 0x21 | BOOL | multiple instances | `multipleInstances` | `mbMultipleInstances` (18 sounds) |
| 0x22 | BOOL | stream | (not in the 1.45.3 XML) | `mbStream` (never set in 1.48.2) |
| 0x23–0x27 | | padding | | |

`BCSound::initWithId:fileName:ext:loop:stream:multipleInstances:preload: @ 0x10000d648`:
the sound with id `Vibrate` (empty file name and extension) gets `mbVibrate = YES` and no
resource path; the struct XML path always passes `preload:NO`, so samples load on first
play. 359 sounds, all `.caf` except `Vibrate`, no duplicate ids.

### 4.2 Font def (type 12, size 0x30)

| Off | Type | Field | XML (`texture_test.xml` `<addFontToAtlas>`) |
|---|---|---|---|
| 0x08 | id | id | `id` |
| 0x10 | id | folder | `folder` (offline atlas builder only) |
| 0x18 | id | file name | `filename` (offline only) |
| 0x20 | id | extension | `ext` (offline only) |
| 0x28 | id | character set | CDATA content |

`BCFontDef::initWithId: @ 0x100066780` zeroes the 256-entry glyph table;
`BCFontDef::setCharSet: @ 0x100066a34` sets, for each byte index `i` of the UTF-8 char set,
`glyph[byte] = [library getTextureDefById:@"<fontId>:FullSet:<i>"]`. All 297 glyph
textures exist in the atlas texture defs. Fonts: `Arial12Black`, `Arial12`,
`AnimatedGothicLite20` (95 printable ASCII chars each) and `Doop24Green` (`" L0123456789"`).

### 4.3 Gradient def (type 10, size 0x18)

| Off | Type | Field | XML (`texture.xml` `<gradientDef>`) |
|---|---|---|---|
| 0x08 | id | id | `id` |
| 0x10 | id | spec string | `gradient` (the XML `width`/`height` attributes were dropped) |

`BCGradient::createGradientDef: @ 0x100178500` (class method) parses the spec:
`components = [spec componentsSeparatedByString:@","]`; it mallocs
`BCGradientDef { int count; BCGradientColor colors[count] }` with
`BCGradientColor { float pos, r, g, b, a }` (0x14 bytes). For each component it zeroes
`pos` and `argb`, runs `sscanf(cString(ASCII), "%f:%x", &pos, &argb)` and stores
`r = ((argb >> 16) & 0xff) / 255.0f`, `g = ((argb >> 8) & 0xff) / 255.0f`,
`b = (argb & 0xff) / 255.0f`, `a = (argb >> 24) / 255.0f` (u32 → f32, f32 division).
Stops keep the string order (1.0 first in every 1.48.2 spec). Data quirk kept as is:
`StartGradient` has `0.05:0xf5FDEFE` (seven digits), i.e. alpha 0x0f.

`BCLibrary::addGradientDef:id: @ 0x10002384c` stores the pointer in
`std::map<std::string, BCGradientDef*> mpGradientDefMap` (key = id as UTF-8, max 1023
bytes). `getGradientDefById: @ 0x100023774` uses `operator[]` (`FUN_100026e88`), so an
unknown id yields NULL (and inserts a NULL entry).

### 4.4 Display object base (`BCStructXmlDisplayObjectNodeDef`)

ObjC encoding `{i @@ f×15 I f×4 BB f×6}`; every display node (types 1–9, 11) starts
with these 0x84 bytes. "Applied if" is the condition in
`parseDisplayObjectAttributes:displayObject:` ([section 5](#5-bclibraryparsedisplayobjectattributesdisplayobject-0x10002626c)).
XML names in *italics* do not occur in the 1.45.3 XML and are inferred from the setter.

| Off | Type | Field | Setter | Applied if | XML attribute | Count ≠ default (1.48.2) |
|---|---|---|---|---|---|---|
| 0x00 | int | node type | | | element name | |
| 0x08 | id | id | `initWithId:` | always | `id` | |
| 0x10 | id | data | `setData:` | ≠ nil | *data* | 61 menu-button models (label text, e.g. `Meteor`) |
| 0x18 | f32 | origin x | `setOriginXPos:` | ≠ 0 | *originXPos* | 14 |
| 0x1c | f32 | origin y | `setOriginYPos:` | ≠ 0 | *originYPos* | 11 |
| 0x20 | f32 | origin z | `setOriginZPos:` | ≠ 0 | *originZPos* | 0 |
| 0x24 | f32 | x | `setXPos:` | ≠ 0 | `xPos` | |
| 0x28 | f32 | y | `setYPos:` | ≠ 0 | `yPos` | |
| 0x2c | f32 | z | `setZPos:` | ≠ 0 | `zPos` | children: −277.12 ×4881, 0 ×277, −1500 ×1, −800 ×1; groups: always 0 |
| 0x30 | f32 | x scale | `setXScale:` | ≠ 1 | `xScale` | |
| 0x34 | f32 | y scale | `setYScale:` | ≠ 1 | `yScale` | |
| 0x38 | f32 | z scale | `setZScale:` | ≠ 1 | *zScale* | 0 |
| 0x3c | f32 | x rotation | `setXRot:` | ≠ 0 | *xRot* | 0 |
| 0x40 | f32 | y rotation | `setYRot:` | ≠ 0 | *yRot* | 0 |
| 0x44 | f32 | z rotation (degrees) | `setZRot:` | ≠ 0 | `zRot` | |
| 0x48 | f32 | x skew | `setXSkew:` | ≠ 0 | *xSkew* | 92 models |
| 0x4c | f32 | y skew | `setYSkew:` | ≠ 0 | *ySkew* | 123 models |
| 0x50 | f32 | z skew | `setZSkew:` | ≠ 0 | *zSkew* | 0 |
| 0x54 | u32 | packed colour 0xAARRGGBB | `setRed:green:blue:alpha:` | ≠ 0 | *color* | 1 (`SunReflection` = 0x00ffffff) |
| 0x58 | f32 | red | `setRed:` | ≠ 1 | `red` | |
| 0x5c | f32 | green | `setGreen:` | ≠ 1 | `green` | |
| 0x60 | f32 | blue | `setBlue:` | ≠ 1 | `blue` | |
| 0x64 | f32 | alpha | `setAlpha:` | ≠ 1 | `alpha` | |
| 0x68 | BOOL | hidden | `setIsHidden:YES` | ≠ 0 | `hidden` | 9 |
| 0x69 | BOOL | selectable | `setIsSelectable:YES` | ≠ 0 | `selectable` | |
| 0x6a | | padding (2) | | | | |
| 0x6c | f32 | collision x min | `setColXMin:` | ≠ 0 | `colXMin` | |
| 0x70 | f32 | collision x max | `setColXMax:` | ≠ 0 | `colXMax` | |
| 0x74 | f32 | collision y min | `setColYMin:` | ≠ 0 | `colYMin` | |
| 0x78 | f32 | collision y max | `setColYMax:` | ≠ 0 | `colYMax` | |
| 0x7c | f32 | collision z min | `setColZMin:` | ≠ 0 | *colZMin* | 0 |
| 0x80 | f32 | collision z max | `setColZMax:` | ≠ 0 | *colZMax* | 0 |

Unset members hold the default (scales and colour channels 1.0, everything else 0),
so the stored values equal the effective ones except for the packed colour (see section 5).
Field mapping check: decoding roots 3–4 with this table reproduces 107 of the 204
1.45.3 `displaygroups.xml` groups exactly; the other 97 differ by 1.48.2 content changes
(new/renamed children, newline escapes, edited text) and not by any field mismatch.

### 4.5 Display node variants (types 1–9, 11)

Fields after the 0x84-byte base:

| Type | Kind | Size | Extra fields | XML attributes | Count |
|---:|---|---|---|---|---:|
| 1 | display object | 0x88 | — | — | 0 |
| 2 | display group | 0x90 | 0x88 id: element text (whitespace; never read by code) | `<displayGroup>` | 266 |
| 3 | empty display group | 0x88 | — | `<emptyDisplayGroup>` | 1 |
| 4 | display marker | 0x90 | 0x88 id: linked display group id | `linkId` | 148 (59 without link) |
| 5 | rect | 0x98 | 0x84 f32 min x, 0x88 min y, 0x8c max x, 0x90 max y | — | 0 |
| 6 | model | 0x90 | 0x88 id: texture def id (NULL for 81 hit-area models) | `texture` | 3987 |
| 7 | multi model | 0x88 | — | `<multiModel>` | 324 |
| 8 | tile model | 0x90 | 0x84 f32 wrap width, 0x88 f32 wrap height | `wrapWidth`, `wrapHeight` | 21 |
| 9 | gradient | 0x98 | 0x88 id: gradient def id, 0x90 f32 width, 0x94 f32 height | `gradient`, `width`, `height` | 25 |
| 11 | text | 0xa0 | 0x88 id: font id, 0x90 BOOL center, 0x91 BOOL right, 0x98 id: text (NULL for 83) | `font`, `center`, element text | 654 |

`ScreenDisplayGroup` is the only group with element text (`"\n    "`). All 3906
non-NULL model texture ids exist in the texture def tables (`0x1005f9738`,
`0x1007c4988`); all fonts and gradient defs referenced exist. Group ids are unique;
four marker ids repeat (`addDisplayMarker:` keeps the last).

## 5. `BCLibrary::parseDisplayObjectAttributes:displayObject:` (0x10002626c)

Applies, in this order and only when the condition in the 4.4 table holds: data, origin
x/y/z, position x/y/z, scale x/y/z, rotation x/y/z, skew x/y/z, packed colour, red, green,
blue, alpha, hidden, selectable, collision x min/max, y min/max, z min/max. Every setter is
the plain `BCDisplayObject` store (`@ 0x10010e360`–`0x10010f370`); no display subclass
overrides any of them. The setters only store the value, except:

- `setRed:` / `setGreen:` / `setBlue:` / `setAlpha:` / `setRed:green:blue:alpha:` also set
  `mbColor = YES`. `mbColor` is never read in 1.48.2.
- The packed colour is unpacked as `red = ((c >> 16) & 0xff) / 255`,
  `green = ((c >> 8) & 0xff) / 255`, `blue = (c & 0xff) / 255`, `alpha = (c >> 24) / 255`
  (f32; the 255.0 constant is at `0x10029d43c`). A packed value of 0 is "unset", so fully
  transparent black cannot be expressed this way.

Effective colour for the port:
`rgba = packed != 0 ? unpack(packed) : [1,1,1,1]`, then each channel whose stored value
≠ 1.0 replaces the unpacked one.

## 6. How display groups become display-object trees

### 6.1 Objects and defaults

`BCDisplayObject::initWithId: @ 0x10010dfc0` (all display classes go through it):
`mId = id` (retained), `mDisplayGroup = nil`, `mbHidden = NO`, `mbIsSelectable = NO`,
**`mbHibernate = YES`**, `mHibernateRefCount = 0`, `mPos = mRot = mSkew = mPivotPos =
mOriginPos = mMinPos = mMaxPos = (0,0,0,1)`, `mScale = (1,1,1,1)`, `mbColor = NO`,
`mRed = mGreen = mBlue = mAlpha = 1`, all six collision bounds 0, matrix
`mA = 1, mB = 0, mC = 0, mD = 1`; `mData`, `mLogic`, `mbDebug`, `mbMultiModelMode`
stay zero from `alloc`. It also increments a global display-object counter
(`0x1007d1f10`, debug only).

| Kind | Class (superclass) | Initializer | Extra state set at creation |
|---|---|---|---|
| display group / empty group | `BCDisplayGroup` (`BCDisplayObject`) | `initWithId: @ 0x100066f6c` | `mDisplayObjectArray = [NSMutableArray initWithCapacity:10]` |
| marker | `BCDisplayMarker` (`BCDisplayGroup`) | `initWithId: @ 0x10018bb48` | as a group |
| model | `BCModel` | `initWithId:textureDef: @ 0x100118720` | `mpTextureDef` (NULL = draws nothing) |
| multi model | `BCMultiModel` (`BCModel`) | `initWithId: @ 0x100181694` | `mpTextureDef = NULL`, `mModelIndices = {0,1,2,3}`; geometry comes later from a `Timelines.bin` frame (`setBinaryFrameDef:tweenableChannelArray:`) |
| tile model | `BCTileModel` (`BCMultiModel`) | `initWithId:wrapWidth:wrapHeight: @ 0x100021ec0` | `mWrapWidth`, `mWrapHeight`, `mbVertical = (wrapWidth == 0)`, transition max x/y = 1000, min x/y = −1000 |
| gradient | `BCGradient` (`BCModel`) | `initWithId:width:height:gradientDef: @ 0x100178714` | `mModelWidth/Height`, `mpGradientDef`, then `createModel` |
| rect | `BCRect` | `initWithId:minXPos:minYPos:maxXPos:maxYPos: @ 0x1000219e0` | the four bounds, then `createModel` |
| text | `BCText` | `initWithId:fontDef: @ 0x1001a75b0` | `mFontDef` (retained), `mText = nil`, `mLineCount = 100`, justification 0 (left), wrap width 0 |
| display object | `BCDisplayObject` | `initWithId:` | — |

`BCGradient::createModel @ 0x1001788b4`: for stop `i` (spec order) it emits vertices
`(0, height·pos_i, 0)` and `(width, height·pos_i, 0)`, both with the stop's RGBA, and
indices `2i, 2i+1`; `drawOpenGL: @ 0x100178d18` draws them as a `GL_TRIANGLE_STRIP`
of `2·count` unsigned-short indices. With a NULL def no model is built.

`BCText::setText: @ 0x1001a781c` copies at most 1023 UTF-8 bytes into `mTextBuffer`,
drops leading tab/LF/CR/space and trims trailing ones (internal whitespace is kept),
stores `[text copy]` in `mText` and sets `mbModified`. `setFontJustification: @
0x1001a79c0` stores the value and sets `mbModified`. Justification values (from
`BCFontDef::fillFontCharTextureArray:textBuffer:fontJustification: @ 0x100066c08`):
0 = left, 1 = right (line shifted by −width), 2 = center (shifted by −trunc(width/2)).

### 6.2 Parent/child links and library registries

- `BCDisplayGroup::addDisplayObject: @ 0x100067544`: ignores nil; sets the child's
  `mDisplayGroup` to the group, appends it, and calls `[child unhibernate]` only if the
  group is not hibernating. Every new object hibernates, so nothing is unhibernated
  during parsing; groups wake up when the view activates them at runtime.
- `BCDisplayMarker::addDisplayObject: @ 0x10018bd68`: a marker holds at most one child.
  It first removes its current first child (`removeDisplayObject:`), then sets the new
  child's `mDisplayGroup` to the marker, appends it if non-nil, and unhibernates it if the
  marker is awake. `addDisplayObject:atIndex:` forwards to it.
- A group linked by several markers (`StatScreenFrame` ×6, `HelpPageMainMenu` ×13,
  `HelpBackdrop01` ×9, …) is **one shared object** in several markers' child arrays. Its
  `mDisplayGroup` ends up pointing at the marker resolved last, which is the first such
  marker in list order within the call. `BCText::drawOpenGL:` multiplies by the parent's
  colour, so this parent pointer is observable.
- Registries in `BCLibrary`:

| Registry | Storage | Add | Get | Duplicate id |
|---|---|---|---|---|
| sounds | `NSMutableDictionary mSoundDictionary` | `addSound: @ 0x100023d6c` (key `[sound id]`) | `getSoundById: @ 0x100023d50` | last wins |
| fonts | `NSMutableDictionary mFontDefDictionary` | `addFontDef: @ 0x100023af0` | `getFontDefById: @ 0x100023ad4` | last wins |
| display groups (types 2, 3, rect fallthrough) | `NSMutableDictionary mDisplayGroupDictionary` | `addDisplayGroup: @ 0x100023bd0` | `getDisplayGroupById: @ 0x100023bb4` | last wins |
| markers | `NSMutableDictionary mDisplayMarkerDictionary` | `addDisplayMarker: @ 0x100023c5c` | `getDisplayMarkerById: @ 0x100023c24` | last wins |
| gradient defs | `std::map<string, BCGradientDef*>` | `addGradientDef:id:` | `getGradientDefById:` (NULL if missing) | last wins |
| texture defs | `std::map<string, BCTextureDef*>` | (atlas processing) | `getTextureDefById: @ 0x1000235c4` (`operator[]`, NULL if missing) | — |

Sequences then address groups and markers by id: `drawDisplayGroup:::` makes a group the
view's active group, `addDisplayGroup:::` adds a group to a group or marker (or the active
group), `removeDisplayGroup:::` detaches it (`BCLibrary @ 0x1000244c8`–`0x1000246bc`).

### 6.3 Building the tree in the port

1. Load `sound-defs.json` (call 1), then `font-defs.json` (needs the atlas texture defs)
   and `gradient-defs.json` (call 2). They do not depend on each other.
2. For each entry of `display-groups.json → parse_calls` in order: for each group, create
   the group node, apply its attributes, register it by id, then create and attach its
   children in order (empty groups and markers are also registered). Collect markers that
   have a `link_id`.
3. At the end of each parse call, link markers in **reverse** collection order: look up
   the group id in the registry (as filled so far, including earlier calls), make it the
   marker's single child and set the group's parent to that marker.
4. Model textures, fonts and gradient defs that are missing resolve to "none".
5. Display groups are shared nodes referenced by id (they can be re-parented at runtime
   and can sit in several markers), not owned subtrees. Everything starts hibernating.

## 7. Store links (types 37, 38)

| Type | Struct | Off | Field |
|---|---|---|---|
| 38 | `BCStructXmlLinkNodeDef {i ^^CountryLink @}`, 0x18 | 0x08 | `BCStructXmlCountryLinkNodeDef **` country list |
| | | 0x10 | id (`PocketGodFigures`, `PocketGodOogaJump`, `PocketGodUranus`, `PocketGodComic`, `PocketGod`) |
| 37 | `BCStructXmlCountryLinkNodeDef`, 0x18 | 0x08 | ISO country code (`US`, `AE`, …) |
| | | 0x10 | URL |

They sit at the start of root 4 (`0x1007162f0`): each link node is followed by its own
country nodes, flattened into the same list (174 entries). `parseStructXml:` ignores them.
`PocketGodViewController::openCountryURL: @ 0x10006e7b8` walks a link's country list:
the `US` entry's URL is the default, an entry equal to
`[[NSLocale currentLocale] objectForKey:NSLocaleCountryCode]` overrides it (last match
wins in both cases), and the result goes to `openUrl:`; with no URL it does nothing.
**No code calls it in 1.48.2** (no selector reference to `openCountryURL:`, and no code
refers to the link nodes), so the links are dead data kept for completeness.

## 8. Episodes (Battle of the Gods idols, types 32, 20–23)

Episode list `0x1007306f8`: 49 type-32 nodes in episode order (Episode1 … Episode31,
Episode31B, Episode32 … Episode48). Each episode is one god idol. Its challenges unlock it,
its level-ups level it, and its realm powers, attack and stats drive the battles.

### 8.1 Episode (type 32, `BCStructXmlEpisodeNodeDef`, size 0xe8)

Encoding `{i ^^EnemyIdol ^^RealmPower ^^Challenge ^^LevelUp @×9 iii @ i B iiiii B ff i @ {ffff} @ {ffff}}`.

| Off | Type | JSON field / state | Meaning and use |
|---|---|---|---|
| 0x08 | `T**` | `enemy_idols` | opponent line-up (type 20) |
| 0x10 | `T**` | `realm_powers` | power effects (type 21) |
| 0x18 | `T**` | `challenges` | unlock challenges (type 22) |
| 0x20 | `T**` | `level_ups` | level tracks (type 23) |
| 0x28 | id | `id` | `Episode1`…; key in preference names and `ActiveEpisode`; matched by enemy-idol refs |
| 0x30 | id | `episode_number` | `"1"`…`"31B"`; shown as `"Episode %@"` (`IdolDetailEpisodeNumber`, `IdolIconLogic::activate:isSelected: @ 0x100198ec0`) |
| 0x38 | id | `idol_id` | e.g. `AbyssGod`; no reader found |
| 0x40 | id | `idol_name` | detail/summary panels, enemy `PlayerBattleState.mName` (`initBattleOfTheGodsEnemy`) |
| 0x48 | id | `idol_title` | e.g. `God of the Abyss`; no reader found |
| 0x50 | id | `power_description` | summary panel (`openIdolSummaryPanel`) and battle idol selection (`highlightIdolBattleResults: @ 0x1001977e4`), through `fillPowerDescriptionText:` |
| 0x58 | id | `power_short_description` | battle realm-result text (`IdolScreenLogic::showBattleRealmResults @ 0x1001945a0`) |
| 0x60 | id | `icon_texture` | raw image loaded into an icon texture slot (`FUN_100038648(slots, name, @"raw")`) |
| 0x68 | id | `idol_texture` | raw image for the idol (GodIdolLogic, selection icons, award screen) |
| 0x70 | int | `realm` | 0 oceanus, 1 aeris, 2 infernis, 3 animus, 4 pax (`realmName: @ 0x10002ca30`) |
| 0x74 | int | `attack` | 0 strong, 1 counter, 2 fast (`attackName: @ 0x10002c9fc`) |
| 0x78 | int | `attack_style` | 0 physical, 1 energy: picks `PygmyBattleAttack*` vs `PygmyBattleEnergyAttack*` behaviours (`PygmyLogic(BattleOfTheGods)::startBattleAttack:opponentIdol: @ 0x100122554`) |
| 0x80 | id | `episode_title` | e.g. `Nowhere To Go, Nothin' To Do` (`IdolDetailEpisodeName`) |
| 0x88 | int | runtime | idol level (min over level tracks, 0 if none) |
| 0x8c | BOOL | runtime | all challenges complete (idol earned) |
| 0x90 | int | runtime | completed challenge count |
| 0x94 | int | runtime | challenge count |
| 0x98 | int | runtime | level track count |
| 0x9c | int | runtime | hints activated |
| 0xa0 | int | `initial_selection_slot` | runtime selection slot 0–3, −1 = not selected; the data holds −1 for every episode |
| 0xa4 | BOOL | runtime | battle unlocked |
| 0xa8 | f32 | `damage_per_level` | attack strength ×(level+1) |
| 0xac | f32 | `health_per_level` | health ×(level+1) |
| 0xb0 | int | `power_turn_count` | realm-effect duration; 1 → `"1 turn"`, 100 → `"entire battle"`, else `"%d turns"` |
| 0xb8 | id | runtime | `IdolIconLogic` in the idol list panel |
| 0xc0 | {ffff} | runtime | its scroll position (`openIdolListPanel: @ 0x1001925c4`) |
| 0xd0 | id | runtime | `IdolSelectionIconLogic` (selection / battle-enemy panels) |
| 0xd8 | {ffff} | runtime | its scroll position |

### 8.2 Enemy idol (type 20, size 0x18)

| Off | Type | JSON | Meaning |
|---|---|---|---|
| 0x08 | id | `episode_id` | episode whose idol the opponent uses |
| 0x10 | int | `level` | that idol's level in this battle (0–19) |

Every episode lists exactly four. `IdolManager::initBattleOfTheGodsEnemy @ 0x10002f0fc`
resolves the first four entries whose id matches an episode, copies the 0xe8-byte episode
node into a scratch array (`0x1007ce018 + i·0xe8`), and sets the copy's level (+0x88) to
`level` and its slot (+0xa0) to `i`.

### 8.3 Realm power (type 21, size 0x10)

| Off | Type | JSON | Meaning |
|---|---|---|---|
| 0x04 | int | `power` | 0 `realm_damage`, 1 `tie_damage`, 2 `total_boost_damage`, 3 `attack_boost_damage`, 4 `attack_reflect_damage`, 5 `attack_drain_health`, 6 `attack_defense`, 7 `total_defense`, 8 `ignore_attack_defense`, 9 `realm_heal`, 10 `freeze_idol`, 11 `resurrect_after_death`, 12 `power_up_all_idols`, 13 `drain_level_points`, 14 `cancel_enemy_powerups`, 15 `unfreeze_idols` (`realmPowerName: @ 0x10002cb10`) |
| 0x08 | f32 | `base_value` | value at level 0 |
| 0x0c | f32 | `value_per_level` | value = base + per_level × idol level |

`fillPowerDescriptionText:structXmlEpisodeNodeDef: @ 0x10002cd20` formats each power
value: types 2–7 as `"%.0f%%"` of value×100 (`0x10029d35c`), types 0, 1, 9, 13 as `"%.1f"`,
the rest as `""`. The first power's text replaces `[PrimaryValue]`, the last later one
replaces `[SecondaryValue]`, and `[TurnCount]` comes from +0xb0. `initPlayerBattleState:
@ 0x10002efd8` sets `mbHasFreezeIdol` when any selected idol has power 10.

### 8.4 Challenge (type 22, `{i @@@ i @@ BB @@@@}`, size 0x60)

| Off | Type | JSON / state | Meaning |
|---|---|---|---|
| 0x08 | id | `event_id` | completed by `triggerEpisodeEvent: @ 0x10002fd90` with this id |
| 0x10 | id | `sacrifice_id` | completed by `triggerEpisodeEventBySacrificeId: @ 0x10002f930` |
| 0x18 | id | `gift_id` | completed by `triggerEpisodeEventByGiftId: @ 0x10002fb60` |
| 0x20 | int | `gem_id` | Apocalypse gem 1–7 (episodes 43–47 only) or 0 |
| 0x28 | id | `description` | shown for completed challenges (`IdolDetailChallengeItemName%d`) |
| 0x30 | id | `hint` | `IdolDetailHintText` (`IdolScreenLogic::onButtonPress:touch:`) |
| 0x38 | BOOL | runtime | completed |
| 0x39 | BOOL | runtime | hint activated |
| 0x40, 0x48 | id | — | always NULL; no reader found |
| 0x50 | id | runtime | preference key `"%@%@Challenge"` (episode id, event id) |
| 0x58 | id | runtime | preference key `"%@%@ChallengeHint"` |

`loadEpisodeState @ 0x10002b7b8` maps `event_id`, and `sacrifice_id` / `gift_id` when
present, to `gem_id` in `mGemIdDictionary` for every challenge with a non-zero gem;
`checkGemEvent: @ 0x10002f8b4` sends `onApocalypseGemEventTriggered:gemId` for any
triggered id that maps to a gem.

### 8.5 Level-up (type 23, size 0x50)

| Off | Type | JSON / state | Meaning |
|---|---|---|---|
| 0x08 | id | `event_id` | +1 point per `triggerEpisodeEvent:`; +score per `triggerEpisodeGameScoreEvent:score:` |
| 0x10 | id | `sacrifice_id` | +1 point per matching sacrifice event |
| 0x18 | id | `gift_id` | +1 point per matching gift event |
| 0x20 | id | `description` | `IdolDetailLevelUpItemName%d` |
| 0x28 | int | `max_points` | points for level 19 (cap) |
| 0x2c | int | runtime | points (preference `"%@%@LevelUp"`) |
| 0x30 | int | runtime | current level's lower threshold |
| 0x34 | int | runtime | next threshold |
| 0x38 | int | runtime | level 0–19 |
| 0x3c–0x47 | | — | always zero; no reader found |
| 0x48 | id | runtime | preference key `"%@%@LevelUp"` (episode id, event id) |

Event triggers only touch the **active** episode (`mpActiveStructXmlEpisodeNodeDef`, set by
`onEpisodeActivated: @ 0x10002f71c`, saved as the `ActiveEpisode` option preference). A
trigger first completes every not-yet-completed challenge whose id of that kind matches
(sacrifice/gift ids only when non-NULL), then, only if the episode's +0x8c flag (idol
earned) is set, adds 1 point to every matching level-up (`points = min(points + 1, max)`;
`triggerEpisodeGameScoreEvent:score:` adds `score` and only matches `event_id`), then calls
`updateEpisodeStatus:startup:NO`. While `canTriggerChallenge` is false the event is queued
(`delayTriggerEpisode*Event:`). `applyBattleExperience @ 0x10002eb2c` adds battle
experience to every level-up of the four battle idols regardless of the active episode.

### 8.6 Formulas

- Level thresholds (`IdolManager::init @ 0x10002a1c4`): `T[i] = 1000·i(i+1)/2` for
  i = 0…19 (`0x1007cdc28`), `T[19] = 190000` (`0x1007cdc74`). In `updateEpisodeStatus:
  startup: @ 0x10002aa10`, for a track with max M and points P:
  `if P < M { for i in 0..=19 { thr = (int)((float)M / 190000.0f * (float)T[i]);
  if P < thr { level = i - 1; lower = (int)((float)M / 190000.0f * (float)T[i - 1]);
  next = thr; done } } }`; when P ≥ M or no i matched: `P = M; level = 19; next = 0`
  (lower unchanged). `i = 0` only matches a negative P, which the game never produces.
  Episode level = min of its tracks (0 with no tracks).
- The idol is awarded when the "all challenges complete" flag turns on outside startup:
  `openIdolAwardScreen:`, `checkEmptyIdolSlot:`, `checkImmediateUnlocks:`.
- Player/enemy level = (Σ of the four idol levels) / 4 (C integer division); health =
  Σ health_per_level × (level+1); average damage = ¼ Σ damage_per_level × (level+1)
  (`calculatePlayerLevel @ 0x10002ef50`, `calculatePlayerHealth @ 0x10002ef94`,
  `calculateEnemyLevel: @ 0x10002edc4`, `calculateEnemyHealth: @ 0x10002ee7c`,
  `initPlayerBattleState: @ 0x10002efd8`).
- Attack result (`attackResultFromPlayerIdol:enemyIdol: @ 0x10002cc34`; 0 enemy won,
  1 player won, 2 tie): strong beats counter, counter beats fast, fast beats strong.
- Realm result (`realmResultFromPlayerIdol:enemyIdol: @ 0x10002cc84`, verified in the
  disassembly with unsigned compares): oceanus beats aeris and infernis, aeris beats
  infernis and pax, infernis beats animus and pax, animus beats oceanus and aeris,
  pax beats oceanus and animus.

### 8.7 Consumers

`IdolManager` (`loadEpisodeState`, `onPreferencesChanged:`, `updateEpisodeStatus:startup:`,
`updateEpisodeHintStatus @ 0x10002abbc`, `resetAllChallenges @ 0x10002ac2c`,
`unlockIdolChallenges @ 0x10002b4fc`, `setIdolLevels: @ 0x10002b158`, `resetIdolLevels`,
`resetIdolChallenges`, `applyBattleExperience @ 0x10002eb2c`, `unlockNextBattleIdol: @
0x10002c8f8`, `nextBattleIdol: @ 0x10002ed2c`, `isLastBattleIdol: @ 0x10002ed08`,
`accumulateRealmPowers:opponentBattleState: @ 0x10002cf20`, `aiSelectEnemyIdol @
0x10002d92c`, the trigger functions above, multiplayer save/load), `IdolScreenLogic`
(`openIdolDetailPanel: @ 0x100191b90`, `openIdolSummaryPanel @ 0x1001927b0`,
`openIdolBattleEnemyPanel @ 0x100192e9c`, list/selection icon management),
`IdolIconLogic`, `IdolSelectionIconLogic`, `GodIdolLogic`, `IdolBattleRealmEffectLogic`,
`PygmyLogic(BattleOfTheGods)`. Episodes the code singles out:

- Episode1: always battle-unlocked; completing it auto-completes Episode2's first
  challenge (`checkImmediateUnlocks: @ 0x10002a88c`, `loadEpisodeState`).
- Episode39: once every episode before it in the list is complete, its last challenge
  is auto-completed (`checkImmediateUnlocks:`).
- Episode40: beating it also triggers the `DefeatDeva` event (`onGameDeath @ 0x10002dd58`,
  which also triggers `WinBattle` / `LoseBattle`).
- Battle unlock is sequential: winning against episode N unlocks episode N+1
  (`unlockNextBattleIdol:`).

Preference names: `"%@SelectionSlot"` (option prefs, default −1) and `"%@BattleUnlock"`
(challenge prefs) with the episode id; the per-challenge and per-level-up keys above.

## 9. Dance Dance Execution charts (types 33–35)

| Type | Struct | Off | Field | JSON |
|---|---|---|---|---|
| 35 group | size 0x18 | 0x08 | `T**` segment list | `segments` |
| | | 0x10 | BOOL random segment | `random_segment` |
| 34 segment | size 0x10 | 0x08 | `T**` beat list | `beats` |
| 33 beat | `BCStructXmlDanceDanceBeatNodeDef {ii BBBBBBBB f @@@@ i}`, size 0x40 | 0x04 | int duration in beats (1–4) | `duration` |
| | | 0x08–0x0b | BOOL left, down, up, right | `left`, `down`, `up`, `right` |
| | | 0x0c–0x0f | BOOL hit flags per direction | runtime |
| | | 0x10 | f32 symbol y | runtime |
| | | 0x18–0x30 | id ×4 symbol logics | runtime |
| | | 0x38 | int state 0 pending, 1 active, 2 past, 3 remove | runtime |

80 groups; groups 0–78 have one segment each, group 79 is random with 8 segments. Beats
use one or two directions.

Consumer (`DanceDanceExecutionScreenLogic::manageDanceDanceSymbols: @ 0x100035684`;
`openDanceDanceExecution @ 0x1000362e4` resets the group index to 0,
`startDanceDanceExecution::: @ 0x1000360f4` resets the beat counter): when no queued beat is
pending, take `groups[groupIndex]`; if it is not random use its first segment and
increment `groupIndex`, otherwise pick `segments[arc4random() % count]` and keep the index,
so the last group repeats forever. Each beat is copied (0x40 bytes) into a 100-entry ring
(`0x1007ce3b8`, indices `0x1007e43f0`/`0x1007e43f4`); in the copy, +0x04 is replaced by the
absolute start beat (running sum of durations), and the sum grows by the beat's duration.
Beat clock: `(now − start + fudge − 7.5) / 0.2325546875 + 13`. A beat becomes active at its
start (symbols `DanceDanceSymbolLeft/Down/Up/Right` at x = 44, 110, 370, 436, y = 350), moves
by y = 350 − 317·(clock − start)/13, is hittable within ±0.5 beat of start + 13, and is
missed after that. The scoring rules belong to the dance-game spec.
(`DanceDanceExecutionLogic`'s move table at `0x1007516c0` is a separate, non-struct-XML table
using direction bits left 1, down 2, up 4, right 8.)

## 10. Konkey Dong levels (types 13–19)

List `0x100649c68`, parsed by `KongStatueLogic::parseStructXml @ 0x100187f04` from
`KongStatueLogic::initWithDisplayObject: @ 0x100186040` into
`mKonkeyDongLevelArray[10]` (`KonkeyDongLevel`, 0x58 bytes each, no bounds check) and
`mKonkeyDongLevelCount`.

| Type | Size | Off | Field | JSON | Effect |
|---|---|---|---|---|---|
| 13 level | 0x30 | 0x08 | id | `id` | not read (levels are indexed by list order) |
| | | 0x10 | id | `display_group` | `level.mDisplayGroup = getDisplayGroupById:` |
| | | 0x18 | id | `vegetation_object` | object in that group wrapped in a `MultiModelLogic` |
| | | 0x20 | id | `vegetation_behavior` | `setBehavior:` on it |
| | | 0x28, 0x2c | f32 | `level_width`, `level_height` | `mLevelWidth`, `mLevelHeight` |
| 14 kong pos | 0x0c | 0x04, 0x08 | f32 x, y | `kong_pos` | `mKongPos.x/y` |
| 15 pygmy start | 0x0c | 0x04, 0x08 | f32 x, y | `start_pygmy_pos` | `mStartPygmyPos.x/y` |
| 16 platform | 0x28 | 0x08 | id object | `object` | `KonkeyDongPlatformLogic` on `[group getDisplayObject:]`, listener = KongStatueLogic |
| | | 0x10 | id | `behavior` | `setBehavior:` (last call) |
| | | 0x18 | BOOL | `start_platform` | `setStartPlatform:` |
| | | 0x19 | BOOL | `final_platform` | `setFinalPlatform:` |
| | | 0x1c | f32 | `boulder_x_pos_vel` | `setBoulderXPosVel:` |
| | | 0x20 | BOOL | `flip_boulder_x_pos_vel` | `setFlipBoulderXPosVel:` |
| 17 ladder | 0x20 | 0x08, 0x10 | id object, behavior | `object`, `behavior` | `KonkeyDongLadderLogic`, listener set |
| | | 0x18 | BOOL | `force_boulder_drop` | `setForceBoulderDrop:` |
| | | 0x19 | BOOL | `do_not_pass` | `setDoNotPass:` |
| 18 pick | 0x18 | 0x08 | id | `object` | `KonkeyDongPickLogic` |
| | | 0x10 | int | `loop_lock` | `setLoopLock:` |
| 19 beetle | 0x18 | 0x08 | id | `object` | `KonkeyDongBeetleLogic`, listener set |
| | | 0x10 | int | `loop_unlock` | `setLoopUnlock:` |

Type 13 starts a new level: display group lookup, four `NSMutableArray`s (capacity 5) for
platforms, ladders, picks and beetles, the vegetation logic, width/height, then the count
increments. Types 14–19 apply to the current level; with no current level they are
skipped. Other types are ignored. Logics are appended to their array and released. Data
order is `Level0` (KonkeyDongLevel1DisplayGroup), `Level1` (Level3), `Level4` (Level6),
`Level2` (Level5), `Level3` (Level4); 48 platforms, 30 ladders, 9 picks, 21 beetles.

## 11. Debug items (type 36)

Type 36 (size 0x10): +0x08 id = display name. List `0x100730938` (11 items:
`ResetChallenges`, `UnlockChallenges`, `ResetLevels`, `SetLevel5`, `SetLevel10`, `SetLevel15`,
`SetLevel20`, `ResetEnemies`, `UnlockEnemies`, `ClearSkinPacks`, `RemovePrefs`).
`DebugScreen::countDebugItems @ 0x100008c44` counts them; `updateDebugItems @ 0x100008c74`
puts names 15 per page into the `DebugScreen` group's `Option%dButton` texts. Tapping an
item sends `selectDebugItemIndex: @ 0x100008f2c`, which is empty in 1.48.2, so the items do
nothing (the names match `IdolManager` debug helpers such as `resetAllChallenges` and
`setIdolLevels:`).

## 12. What the extractor validates

- Every node in the five library roots has type 0–12, 37 or 38; Konkey Dong nodes 13–19;
  each sub-list node has its expected type.
- Runtime-only members are zero: episode +0x88–0x9f, +0xa4, +0xb8–0xe7; challenge +0x38,
  +0x39, +0x40–0x5f; level-up +0x2c–0x4f; dance beat +0x0c–0x3f.
- Enum values are in range (realm 0–4, attack 0–2, style 0–1, realm power 0–15).
- Every `id` member is NULL or a CFString.

Checks done while writing this document (not in the script): marker links all resolve,
group ids are unique, model texture ids and font glyph ids exist in the texture defs, no
orphan nodes, no type 1/5 nodes, decoding matches 107 unchanged 1.45.3 groups exactly.

## 13. JSON output

Common rules: snake_case keys; key order is fixed; list order is data order;
strings are UTF-8; absent strings are `null`; f32 values are written as the shortest
decimal that parses back to the same f32 (so `"0.41"`, not `0.4099999964`); booleans
are `true`/`false`; enum values are lower snake_case strings. Records are written one per
line.

### `sound-defs.json`

`{"sounds": [Sound]}`, parse order (call 1).
`Sound = {id: String, file_name: String, ext: String, loop: bool, multiple_instances: bool, stream: bool}`.
Resource path: `sound/<file_name>.<ext>`; `Vibrate` has empty strings. `preload` is always
false and is not stored.

### `font-defs.json`

`{"fonts": [Font]}`; `Font = {id, folder, file_name, ext, char_set}` (all String). Glyph
`i` of `char_set` uses texture def `"<id>:FullSet:<i>"`.

### `gradient-defs.json`

`{"gradients": [Gradient]}`;
`Gradient = {id: String, spec: String, stops: [Stop]}`;
`Stop = {position: f32, argb: u32, rgba: [f32; 4]}` (stops in spec order, values exactly as
`createGradientDef:` computes them).

### `display-groups.json`

```
{"parse_calls": [ParseCall]}                 // the five parseStructXml: calls, in order
ParseCall = {root: String /* "0x1006fe1d0" */, groups: [Group], orphans: [Node]}
Group = DisplayAttributes + {element_text: String|null, children: [Node]}
Node  = {kind: NodeKind} + DisplayAttributes + kind-specific fields
DisplayAttributes = {
  id: String, data: String|null,
  origin: [f32;3], pos: [f32;3], scale: [f32;3], rot: [f32;3], skew: [f32;3],
  packed_color: u32|null,        // null when 0 (not applied)
  rgba: [f32;4],                 // stored channels; see section 5 for the effective colour
  hidden: bool, selectable: bool,
  col_min: [f32;3], col_max: [f32;3]   // [x,y,z] of colXMin/colYMin/colZMin and ...Max
}
```

| `kind` | Type | Extra fields |
|---|---:|---|
| `display_object` | 1 | — |
| `empty_group` | 3 | — |
| `marker` | 4 | `link_id: String\|null` |
| `rect` | 5 | `rect: [min_x, min_y, max_x, max_y]` |
| `model` | 6 | `texture: String\|null` |
| `multi_model` | 7 | — |
| `tile_model` | 8 | `wrap_width: f32`, `wrap_height: f32` |
| `gradient` | 9 | `gradient: String\|null`, `width: f32`, `height: f32` |
| `text` | 11 | `font: String\|null`, `text: String\|null`, `justification: "left"\|"right"\|"center"` |

Calls 1, 2 and 5 have no groups. A `rect` child would also start a new group (section 3);
none exist. `orphans` lists display nodes met before the first group of a call (none).
Suggested serde mapping:

```rust
#[derive(Deserialize)]
pub struct DisplayAttributes {
    pub id: String,
    pub data: Option<String>,
    pub origin: [f32; 3],
    pub pos: [f32; 3],
    pub scale: [f32; 3],
    pub rot: [f32; 3],
    pub skew: [f32; 3],
    pub packed_color: Option<u32>,
    pub rgba: [f32; 4],
    pub hidden: bool,
    pub selectable: bool,
    pub col_min: [f32; 3],
    pub col_max: [f32; 3],
}

#[derive(Deserialize)]
pub struct GroupDef {
    #[serde(flatten)]
    pub attributes: DisplayAttributes,

    pub element_text: Option<String>,
    pub children: Vec<NodeDef>,
}

#[derive(Deserialize)]
pub struct NodeDef {
    #[serde(flatten)]
    pub attributes: DisplayAttributes,

    #[serde(flatten)]
    pub kind: NodeKind,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeKind {
    DisplayObject,
    EmptyGroup,
    Marker { link_id: Option<String> },
    Rect { rect: [f32; 4] },
    Model { texture: Option<String> },
    MultiModel,
    TileModel { wrap_width: f32, wrap_height: f32 },
    Gradient { gradient: Option<String>, width: f32, height: f32 },
    Text { font: Option<String>, text: Option<String>, justification: TextJustification },
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextJustification {
    Left,
    Right,
    Center,
}
```

### `links.json`

`{"links": [Link]}`; `Link = {id: String, country_links: [{country: String, url: String}]}`.

### `episodes.json`

`{"episodes": [Episode]}` in list order.
`Episode = {id, episode_number, idol_id, idol_name, idol_title, power_description,
power_short_description, icon_texture, idol_texture: String, realm: Realm, attack: Attack,
attack_style: "physical"|"energy", episode_title: String, initial_selection_slot: i32,
damage_per_level: f32, health_per_level: f32, power_turn_count: i32,
enemy_idols: [{episode_id: String, level: i32}],
realm_powers: [{power: RealmPower, base_value: f32, value_per_level: f32}],
challenges: [{event_id: String, sacrifice_id: String|null, gift_id: String|null,
gem_id: i32, description: String, hint: String}],
level_ups: [{event_id: String, sacrifice_id: String|null, gift_id: String|null,
description: String, max_points: i32}]}`.
`Realm` = `oceanus|aeris|infernis|animus|pax` (0–4), `Attack` = `strong|counter|fast` (0–2),
`RealmPower` = the 16 names of 8.3 in value order. Declare the Rust enums in that order
when the numeric value matters (comparisons, saves).

### `dance-beats.json`

`{"groups": [{random_segment: bool, segments: [{beats: [{duration: i32, left: bool,
down: bool, up: bool, right: bool}]}]}]}`.

### `konkey-dong-levels.json`

`{"levels": [Level]}` in list order (= `mKonkeyDongLevelArray` index).
`Level = {id, display_group, vegetation_object, vegetation_behavior: String,
level_width: f32, level_height: f32, kong_pos: [f32;2]|null, start_pygmy_pos: [f32;2]|null,
platforms: [{object, behavior: String, start_platform, final_platform: bool,
boulder_x_pos_vel: f32, flip_boulder_x_pos_vel: bool}],
ladders: [{object, behavior: String, force_boulder_drop, do_not_pass: bool}],
picks: [{object: String, loop_lock: i32}], beetles: [{object: String, loop_unlock: i32}]}`.
If a level had several type-14/15 nodes, the last would win (each level has exactly one).

### `debug-items.json`

`{"items": [String]}`.

## 14. Open questions

1. XML attribute names for the members absent from the 1.45.3 XML (`data`, origin, z
   scale, x/y rotation, skew, packed colour, z collision bounds, sound `stream`, text
   right-justify) are inferred from the setters. They do not affect the port, because
   the JSON uses its own names.
2. Root 5 (`0x1008162c0`) is empty in 1.48.2; which XML it was compiled from is unknown.
3. Node types 24–31 are not used by any consumer or any data.
4. Episode `idol_id` (+0x38) and `idol_title` (+0x48), challenge +0x40/+0x48 and level-up
   +0x3c–0x47 have no reader in 1.48.2 (searched the idol classes, `PocketGodViewController`
   and categories by decompilation and disassembly). They are exported (the strings) or
   documented (the zero members) only.
5. The links (types 37/38) and the debug items have no working consumer in 1.48.2
   (`openCountryURL:` has no caller; `selectDebugItemIndex:` is empty). The port may drop
   them; they are exported for completeness.
6. The rect fallthrough in `parseStructXml:` looks like a missing `break` in the original
   source. It cannot be observed in 1.48.2 because no rect nodes exist.
