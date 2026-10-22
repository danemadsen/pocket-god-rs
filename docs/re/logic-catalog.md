# Logic class catalogue

This catalogue covers every gameplay logic class of Pocket God 1.48.2 except `PygmyLogic`
(partitioned separately in `tmp/re/pygmy-partition.json`) and the framework classes
`BCLogic`, `SpriteLogic`, `MultiModelLogic`, `BCScreenLogic`, `BCButtonLogic` and
`MessageLogic` (specified in `docs/re/logic.md`). It also covers the seven custom display
models (`RainbowModel`, `LightningModel`, `LocustModel`, `HurricaneModel`, `WhirlPoolModel`,
`PathModel`, `MagnifyingBeamModel`) and names the logic class that owns each one. That is
149 entries: 142 logic classes and 7 models.

For each class the catalogue gives its superclass, its decompiled file and size, its role
and where it appears, how it is created, its listener protocol, the classes it talks to and
is called from, the sequence, behaviour, animation and other data ids it uses, its
sequence-item handlers, its persistence keys and its notable algorithms. Section 5 groups
the classes into implementation units of at most about 120 KB of decompiled code each, and
`tmp/re/logic-partition.json` holds the same partition as data.

## 1. Overview

### 1.1 Sources and conventions

- Code: `tmp/re/decomp/game/<Class>.c` (Ghidra output, one file per class or category).
  Citations have the form `Class::selector @ 0xADDRESS`. Category methods are written
  `Class(Category)::selector`, for example `PocketGodViewController(Game)::setLogic::: @ 0x10015c11c`.
- Sizes: "decompiled bytes" are the text sizes of the class's own functions in its
  decompiled files, including each function's `// ====` header line. This is the measure
  behind the `bytes` column of `tmp/re/decomp/INDEX.tsv`, which counts whole files. Free C
  helper functions stored in the class file are added when they belong to the class (for
  example the `std::vector` helpers of `PiranhaLogic`). They are left out when they belong to
  another subsystem, such as the 843 KB sequence-definition initializer that Ghidra placed in
  `ApocalypseCalendarLogic.c`. The header line of each entry says which case applies.
- Data: behaviour and sequence ids are in `assets/data/sequences.json`, and the semantics of
  every sequence-item handler are in `docs/re/sequences.md` §5. Animation ids come from
  `Timelines.bin` (`tools/re/timelines.py list`). Display groups, objects and markers are in
  `assets/data/display-groups.json`, sounds in `assets/data/sound-defs.json`, textures in
  `assets/data/textures.json`, and the logic-object table in `assets/data/logic-objects.json`
  (`docs/re/logic.md` §13).
- Coordinates are points on the 480 × 320 landscape screen, with y pointing up. `z = −277.12`
  (`0xc38a8f5c`) is the standard depth of island objects. `dt` is the frame time in seconds.
  "Per frame" factors apply once per game frame, and the original runs at the display rate
  (CADisplayLink, frame interval 1; `docs/re/logic.md` §4.1). Island location states
  (`mIslandLocationState`, saved as the `IslandLocation` preference; `docs/re/audio-prefs-text.md`)
  are numbered as follows:

  | State | Location | State | Location | State | Location |
  |---|---|---|---|---|---|
  | 0 | sand island | 6 | Misfit island | 12 | Locust temple |
  | 1 | rock island | 7 | God island | 13 | Dance temple |
  | 2 | underwater ruins | 8 | Fail Boat | 14 | Plague temple |
  | 3 | graveyard island | 9 | coral reef | 15 | Purification temple |
  | 4 | ice island | 10 | apocalypse island | 16 | Closet temple |
  | 5 | Ape island | 11 | Time temple | 17 | Xenu temple |

### 1.2 How to read an entry

Each entry starts with a header line: the superclass, the decompiled file(s), the number of
ordinary methods, item handlers and class methods, and the decompiled bytes. Bullets follow:

- **Role** and **Where**: what the object is and where it appears in the game.
- **Created**: the creation paths (section 2.1). These are the table entries of
  `createLogicClasses` with their bindings and init steps, the `setLogic` sequence items, and
  the functions that allocate the class in code.
- **Listener**: the listener ivar and its protocol (required and optional methods, plus
  methods inherited from parent protocols), the methods this class never sends, and where
  the listener is set. "Wired by `initLogicListeners`" means the generated table connects it.
- **Talks to**: messages this class sends to other game classes. The handwritten summary
  comes first. It is followed by "also sent in code", which lists sends found in the code and
  not mentioned in the summary, and then by the `getLogic:` indices the class looks up.
  Receivers were resolved from the decompiled text in this order: typed ivars, singleton
  accessors (`+instance`), accessor names (`fishingPoleLogic:`), `getLogic:` indices, and
  selectors that only one class implements. A send to an object fetched from an array is
  only attributed when the selector is unambiguous or the summary names the receiver.
- **Called from**: other classes and categories that send this class's selectors, resolved
  the same way.
- **Data ids**: string ids the class uses, grouped by kind (behaviours, animations, models,
  display groups, objects, markers, textures, sounds, texts, episode events, preference keys,
  achievements).
- **Item handlers**: sequence-item handler methods (`name:::`), with the implementation
  address and the number of uses in `sequences.json`. Their semantics are in
  `docs/re/sequences.md` §5 and are not repeated here.
- **Persistence**: preference keys read or written.
- **Behaviour**: the algorithms, with constants and citations.

### 1.3 Numbers and clusters

Of the 149 entries, 128 derive from `SpriteLogic`, 7 from `BCDisplayObject`, 6 from `NoPeeObjectLogic`, 5 from `BCLogic` and 3 from `BCScreenLogic`. They have 2 846 instance methods, of which 349 are sequence-item handlers, and 2 401 593 decompiled bytes in total. Each class belongs to one feature cluster (section 4) and one implementation unit (section 5):

| § | Cluster | Classes | Bytes | Units |
|---|---|---|---|---|
| 4.1 | Sky, weather and gesture powers | 16 | 177 790 | `sky_weather`, `sky_powers` |
| 4.2 | Main-island objects | 14 | 266 092 | `island_core`, `island_objects`, `island_anthill` |
| 4.3 | Sea creatures and fishing | 7 | 224 389 | `ocean_fishing`, `ocean_creatures`, `ocean_birds` |
| 4.4 | Underwater ruins and coral reef | 18 | 267 899 | `underwater_baitmaster_laser`, `underwater_paindrain`, `underwater_stage` |
| 4.5 | No Pee (Call of Booty) mini-game | 13 | 110 969 | `nopee` |
| 4.6 | Ice island and boss health bar | 8 | 84 294 | `ice_island` |
| 4.7 | Dinosaur (T-Rex) | 2 | 63 788 | `monsters` |
| 4.8 | Graveyard island | 3 | 46 765 | `monsters` |
| 4.9 | Misfit island | 2 | 9 042 | `misfit_oogajump` |
| 4.10 | Ooga Jump mini-game | 1 | 10 296 | `misfit_oogajump` |
| 4.11 | Ape island | 8 | 181 858 | `ape_ritual`, `ape_monkeys` |
| 4.12 | Konkey Dong mini-game | 7 | 101 674 | `konkeydong` |
| 4.13 | The Runs mini-game | 5 | 70 238 | `theruns` |
| 4.14 | Moron Pests mini-game | 5 | 131 024 | `moronpests_quiz`, `moronpests_chase` |
| 4.15 | Apocalypse island and temples | 25 | 310 003 | `apocalypse_temples`, `apocalypse_locust_closet_xenu`, `apocalypse_dance` |
| 4.16 | God island and Battle of the Gods | 6 | 90 655 | `battle` |
| 4.17 | Idol screen and episode UI | 5 | 247 312 | `idol_ui_widgets`, `idol_screen` |
| 4.18 | Map paper and ad blocker | 2 | 5 487 | `idol_ui_widgets` |
| 4.19 | Framework helpers | 2 | 2 018 | `idol_ui_widgets` |

## 2. Shared patterns

Many classes repeat the same small set of mechanisms. They are described once here, and the
entries refer back to them.

### 2.1 Creation paths

1. **Generated table.** `BCLibrary(Logic)::createLogicClasses @ 0x10013feb8` builds the 632
   global logic objects (`assets/data/logic-objects.json`, `docs/re/logic.md` §13). Each entry
   binds a class to a new `BCMultiModel`, a display group or a group object, and applies init
   steps: position, animation id, start sequence, apocalypse-cloud style. Code looks the
   objects up with `getLogic:<index>`. `BCLibrary(Logic)::initLogicListeners @ 0x1001512f8`
   then wires their listeners to singletons: the view controller, or another logic object
   such as `NoPeeLogic`, `IdolScreenLogic` or `MoronSnakeLogic`. In this catalogue, 54 classes own 162 of the 632 entries.
2. **`setLogic` sequence items.** `PocketGodViewController(Game)::setLogic::: @ 0x10015c11c`
   (mostly from the boot script `PocketGod`, `docs/re/sequences.md` §3.14) gives the display
   object `displayObjectId` of group `displayGroupId` a logic chosen by `logicId`. Gameplay ids
   create the class of the same name with the view controller as listener, and store it in a
   view-controller ivar or array. The details below were checked against the code:
   - `CoconutLogic` objects are added only to `mCoconutLogicArray`. Every other created logic
     is also added to `mLogicArray`.
   - `MapPaperLogic` is initialised with `BCLogic::initWithDisplayObject:` and has no listener.
   - `SpriteLogic`, `MultiModelLogic`, `TheRunsPlatformLogic`, `TheRunsStalactiteLogic` and
     `KonkeyDongLogic` are created only. Their owners pick them up later through the display
     object or `getLogic:`.
   - Aliases:

     | `logicId` | Class | Set-up call |
     |---|---|---|
     | `FrontStormCloudLogic`, `BehindStormCloudLogic` | `StormCloudLogic` | `setFrontStormCloud:posVelScale:` YES, 1.0 / NO, 0.8 (`0x3f4ccccd`) |
     | `FrontOceanLogic`, `BehindOceanLogic` | `OceanLogic` | `setFrontOcean:posVelScale:` YES, 1.0 / NO, 0.5 |
     | `FrontTsunamiLogic`, `BehindTsunamiLogic` | `TsunamiLogic` | `setFrontLayer:` YES / NO |
     | `SchoolOfFishLogic`, `UnderwaterBlob1Logic`, `UnderwaterBlob2Logic` | `UnderwaterSceneryLogic` | scenery type 0 / 1 / 2 |
     | `StarsLogic` | `MultiModelLogic` | its display object is removed from the group |

   - An item with `manager=ChumBucket` is forwarded to `ChumBucketLogic::setLogic:`, which
     creates `LaserSharkLogic`, `LaserTargetLogic` and the laser `BCButtonLogic` with the bucket
     as listener. In this catalogue, 53 classes are created this way.
3. **Allocation in code.** A creator allocates the class, initialises it on a display object,
   sets itself (or the view controller) as listener and keeps the reference. Examples: ants,
   lava, icicles, snowballs, bird droppings, chum, thetans, fireworks, clouds and the custom
   models. In this catalogue, 43 classes are allocated in code, and each entry names the functions that do it.
4. **Class factory.**
   `+[ApocalypseCloudLogic apocalypseCloudWithSprite:apocalypseDiseaseType:apocalypseCloudStyleId:] @ 0x1000c9590`,
   called from `SpriteLogic::infectWithDisease:` (`docs/re/logic.md` §11.4).
5. **Singleton accessors.** 16 classes have a class method `+instance` that
   returns a static pointer, which `initWithDisplayObject:` sets. The pointer therefore refers
   to the most recently initialised object: `ApocalypseTempleFountainLogic` has two table
   entries (503 and 507), and `+instance` returns entry 507.
6. **Never instantiated.** `NoPeePillarLogic` and `TestLogic` (no class reference outside
   their own methods) and the abstract base `NoPeeObjectLogic`.

### 2.2 Throwable objects: gravity, tilt, island floor and drag

`CoconutLogic` is the reference implementation. Most objects that can be picked up and thrown
repeat its code with their own constants: wood, spear, boulder, banana, fish, swordfish,
shark, bird, water wings, apocalypse gem, god idol, fishing pole, Moron snake, T-Rex egg,
magnifying glass, and the boss and bat bodies. A port should implement this once and
parameterise it. `Xxx` below stands for the class's behaviour prefix (`Coconut`, `Wood`, …).

- **Gravity** (`CoconutLogic::onGameFrame: @ 0x100189180`). If the listener's
  `isGravityEnabled` and `SpriteLogic::isAccelEnabled` (`CoconutLogic::isAccelEnabled @ 0x100189110`)
  are both true, the gravity force is `normalize(−accel.y, accel.x, 0) · 3000`. Here `accel`
  is `BCView.accelForce` and `FUN_1001a157c` normalises x, y and z. Otherwise the force is
  `(0, −3000, 0)`.
- **Tilt state** (`CoconutLogic::refreshTiltState @ 0x10018971c`; `undoTiltState @ 0x1001898fc`
  runs instead when the accelerometer is off). It only applies when the object is not being
  dragged, not falling, and on the island. With `z = BCView.rawAccelRotDegrees.z`:

  | Condition | State | On entering the state |
  |---|---|---|
  | \|z\| > 130 | 0 | velocity zeroed, `XxxStopSlide` |
  | 90 < \|z\| ≤ 130 | 1 | velocity zeroed, `XxxTiltSlide` |
  | \|z\| ≤ 90 | 2 | nothing, and the stored state is not changed |

  After this, if the filtered `BCView.accelRotDegrees.z` lies in (−90, 90), the object starts
  falling: `mbFalling`, state 0, `XxxFall`. `undoTiltState` turns state 1 back into 0 and plays
  `XxxStopSlide`.
- **Force.** In tilt state 1, `gravity.x *= slideFriction`. In every other state
  `gravity.x = 0`. The force is the gravity (unless `mbIgnoreGravity`) plus the spring force
  (if `mbApplySpring`).
- **Integration** (explicit Euler): `vel += force·dt`, then `pos += vel·dt`, computed per
  component.
- **Screen clamp**: if the display object's `minPos.x < 0`, then `x −= minPos.x`. If
  `maxPos.x > 480`, then `x = 480 − (maxPos.x − x)`.
- **Island floor.** The object hits the floor when floor collision is enabled, the previous
  `y ≥ floor`, the new `y < floor`, and `islandMinXPos ≤ x ≤ islandMaxXPos` (asked of the
  listener). On first contact (not yet `mbOnIsland`) the velocity is multiplied by 0.4 and
  `vel.y` is negated. On later contacts `vel.y = 0`. If `vel.y ≥ 100` the object bounces
  (`XxxBounce`). Otherwise `vel.y = 0`, and on first contact the object lands: falling ends,
  `vel.x = 0`, the listener gets `onXxxHitGround:`, and `rot.z` is reset. In both cases `y` is
  set to the floor and the "don't hit until bounce" flag clears. Without contact, `mbOnIsland`
  clears.
- **Out of range**: `y < −100` or `y > 820` → `XxxDie`.
- **Pygmy collision** (`CoconutLogic::checkPygmyCollision @ 0x100188e00`). While moving down,
  every pygmy in the listener's `pygmyLogicArray:` within 20 px of `(px, py + 31)` is offered
  `hitByXxx:`. If it accepts, the object rebounds (`vel = (dx·10, 700)`, coconut values), a
  drag ends, and the listener is told (`onXxxHitPygmy:`). Each class has its own radius and
  reaction, given in its entry.
- **Drag.** `touchBeganInside:allTouches:` (`@ 0x10018a204`) is refused during story mode,
  falling, water or eating. Otherwise it zeroes the velocity, records the touch, sets
  `mbDragging` and `mbApplySpring`, sends `onXxxDrag:` and plays `XxxDrag`. The `drag:::` item
  (`CoconutLogic::drag::: @ 0x100189a90`) turns gravity on, turns floor collision off, moves the
  object to the drag layer, and every frame sets the spring force
  `F = −k·(pos − touch) − c·vel`, written in the code as `(|d|·−k)/|d|`. Some classes also set
  `rot.z = atan2(−dy, −dx)·180/π − 90`. `touchEndedInside:allTouches:` (`@ 0x10018a4cc`) ends
  the drag: falling, spring off, island layer, `onXxxDragComplete:`, `XxxFall`. The thrown
  object keeps its spring-driven velocity. `fall:::` and `slide:::` re-enable floor collision
  and gravity and splash below a class-specific water line. `fallFromHurricane:::` is the same
  without floor collision.
- **Layers.** `moveToIslandLayer` (after `SmallObjectsMarker`), `moveToDragLayer` (after
  `DragDisplayMarker`), `moveToBehindIslandLayer` (before `IslandDisplayMarker`) and
  class-specific layers re-insert the display object next to a display marker
  (`CoconutLogic::moveToDragLayer @ 0x10018881c`).

Constants found in the code (floor from the `setYPos:` literal or the class's floor
variable, friction from `mSlideFriction`, spring from `drag:::`):

| Class | Floor y | Slide friction | Drag spring k / c | Rotates while dragged | `drag:::` |
|---|---|---|---|---|---|
| `CoconutLogic` | 92 | 0.04 | 100 / 8 | yes | @ 0x100189a90 |
| `WoodLogic` | 80 | 0.035 | 100 / 8 | yes | @ 0x10017b544 |
| `SpearLogic` | 72 | 0.025 | 100 / 8 | no | @ 0x100170514 |
| `BoulderLogic` | 126 | 0.025 | 50 / 6 | no | @ 0x10001e664 |
| `BananaLogic` | 98 | 0.04 | 100 / 8 | yes | @ 0x10001c2e4 |
| `FishLogic` | 76 | 0.04 | 100 / 8 | yes | @ 0x1000127c8 |
| `SwordFishLogic` | 86 | 0.04 | 100 / 8 | yes | @ 0x1000c27e8 |
| `SharkLogic` | 80 | no tilt sliding | 100 / 8 | yes | @ 0x1000296a8 |
| `BirdLogic` | 70 | no tilt sliding | 100 / 8 | yes | @ 0x100093ec0 |
| `WaterWingsLogic` | 80 | 0.04 | 100 / 8 | no | @ 0x100110cd0 |
| `ApocalypseGemLogic` | 92 | 0.04 | 100 / 8 | yes | @ 0x10000fb04 |
| `GodIdolLogic` | 90 | 0.025 | 50 / 6 | yes, at ¼ of the angle | @ 0x10011a298 |
| `FishingPoleLogic` | 72 | 0.04 | 100 / 8 | yes | @ 0x1001a31cc |
| `MoronSnakeLogic` | 85 | 0.025 | 100 / 8 | yes | @ 0x10018d34c |
| `TRexEggLogic` | 119 | 0.025 | 50 / 6 | no | @ 0x1001725d0 |
| `MagnifyingGlassLogic` | 126 | 0.025 | 50 / 6 | no | @ 0x1000d3984 |
| `TRexLogic` | 86 | 0.025 | not draggable (poked) | – | – |
| `SpiderLogic` | 86 | 0.025 | not draggable (poked) | – | – |

### 2.3 Island entry and exit

`PocketGodViewController::setIslandLocationState:startup: @ 0x100083b64` stores the location
(preference `IslandLocation`). It then calls
`PocketGodViewController::deactivateLastIslandLocationState @ 0x10007ed70`, which sends
`exitIsland`, `exitIslandNow` or `exitIsland:` to the objects of the old location, and
`PocketGodViewController::activateIslandLocationState: @ 0x100080fa8`, which sends
`enterIsland:` or `enterIsland` to the objects of the new one. The argument of `enterIsland:`
is the `startup` flag: YES when the state is restored at launch, which places objects at rest
at once (for example the boulder directly on the floor), and NO otherwise, which animates them
in. The per-feature setters do the same when a feature is toggled from the menu. Examples are
`setSandIslandState:startup: @ 0x100083f50`,
`setRockIslandFeatureState:startup: @ 0x100084414`,
`setGraveyardIslandSpiderWebState:startup: @ 0x100085198`,
`setApeIslandState:startup: @ 0x100085f30` and
`setUnderwaterCoralReefState:startup: @ 0x100086d90`.
`PocketGodViewController(Game)::disableIslandLogics @ 0x100165b9c` and
`enableIslandLogics @ 0x100166068` send the same messages in bulk. The three exit forms are:

- `exitIsland`: an animated exit (`XxxExitIsland`).
- `exitIslandNow`: immediate removal.
- `exitIsland: now`: one method taking the choice as a BOOL.

Static props (glacier, ice tree, octopus statue and similar) stop their game frame in init,
remove their display object, and insert it before or after their marker on `enterIsland:`.
`IslandObjectLogic` is the framework version of this (`docs/re/logic.md` §11.3).

### 2.4 Listeners

Every class with a listener declares `m<Name>LogicListener` of type `id<<Name>LogicListener>`
and a setter `set<Name>LogicListener:` that retains the new listener (`docs/re/logic.md` §12;
the protocols are parsed from `__objc_protolist`). 134 of the 149 entries have a listener ivar. The listener is both an
event sink (`onXxxDrag:`, `onXxxDie:`, …) and the object's window onto game state
(`isStoryActive`, `isGravityEnabled`, `islandMinXPos`, `pygmyLogicArray:`, and accessors such as
`fishingPoleLogic:` that return other logic objects). A port can model each protocol as a
trait implemented by the game-state owner, plus events.

Listeners other than the view controller:

- `AnchorLogic` listens to `PiranhaLogic`.
- `AntHillLogic` listens to `AntLogic`.
- `BirdLogic` listens to `BirdPooLogic`.
- `ChumBucketLogic` listens to `ChumLogic`, `LaserSharkLogic`, `LaserTargetLogic`.
- `ClosetTempleThetanMeterLogic` listens to `ThetanLogic`.
- `CrackHoleLogic` listens to `TheRunsPlatformLogic`, `TheRunsStalactiteLogic`.
- `IceHoleLogic` listens to `IcicleLogic`.
- `IceMonsterLogic` listens to `HealthBarLogic`, `SnowBallLogic`.
- `IdolManager` listens to `IdolScreenLogic`.
- `IdolScreenLogic` listens to `IdolBattleRealmEffectLogic`, `IdolIconLogic`, `IdolSelectionIconLogic`.
- `KongStatueLogic` listens to `KonkeyDongBeetleLogic`, `KonkeyDongBoulderLogic`, `KonkeyDongLadderLogic`, `KonkeyDongLogic`, `KonkeyDongPlatformLogic`.
- `LaserSharkLogic` listens to `LaserGunLogic`.
- `MapScreen` listens to `AdBlockerLogic`, `MapPaperLogic`.
- `MoronPestsQuestionScreenLogic` listens to `MoronBeeLogic`, `MoronDuckLogic`.
- `MoronSnakeLogic` listens to `MoronPestsQuestionScreenLogic`, `MoronTurtleLogic`.
- `NewsManager` listens to `MapPaperLogic`.
- `NoPeeLogic` listens to `NoPeeClamLogic`, `NoPeeFishLogic`, `NoPeeMapLogic`, `NoPeeMapScreenLogic`, `NoPeeMineLogic`, `NoPeeParticleLogic`, `NoPeeSeaBlobLogic`, `NoPeeTreasureLogic`.
- `NoPeeMapLogic` listens to `NoPeeMapItemLogic`.
- `SpearStatueLogic` listens to `ChainSpearLogic`.
- `SpiderLogic` listens to `HealthBarLogic`.
- `TRexLogic` listens to `HealthBarLogic`.
- `TileDrainLogic` listens to `PainDrainPlatformLogic`.
- `VolcanoLogic` listens to `LavaLogic`.

### 2.5 Other conventions

- **Behaviours.** `setBehavior:@"Id"` runs the sequence `Id` on the object
  (`docs/re/logic.md` §7). Each sequence item calls a `name:::` handler on the logic object.
  The handler returns whether the item is still running or complete, or it jumps
  (`docs/re/sequences.md` §5).
- **Animation channels.** "Binds X on channel n" means
  `[[self animation] setTweenable:X channelId:n]` (`BCAnimation::setTweenable:channelId: @ 0x10019fdac`).
  The bound display object (usually a pygmy) follows channel n of the current timeline
  animation until `clearTweenableChannelIdNow:` releases it. Pygmies carried by birds, sitting
  in igloos and outhouses, strapped to altars and so on are moved this way. They are not
  moved by physics.
- **Hurricane and whirlpool.** `HurricaneLogic` and `WhirlPoolLogic` iterate the listener's
  `spriteLogicArray:` and use the `SpriteLogic` participation hooks (`docs/re/logic.md` §11.4).
  Throwables override `pickedUpByHurricane:` and `droppedByHurricane:` and use the
  `fallFromHurricane:::` item. A picked-up object is released with the hurricane's or
  whirlpool's `dropSpriteLogic:`.
- **Disease cloud.** Objects that can catch the apocalypse plague or purification use
  `SpriteLogic::infectWithDisease:`, `showApocalypseCloud` and `hideApocalypseCloud`. Table
  init steps set their cloud style (`setApocalypseCloudStyleId:`, see `ApocalypseCloudLogic`).
- **Episode events.** `[[IdolManager instance] triggerEpisodeEvent:@"Id"]` reports player
  actions to the episode and challenge system. The ids are listed under "episode events" in
  each entry.
- **Randomness.** `arc4random() % n` is written `rand%n`, so `(rand%401 + 2200)/10` is a
  uniform value in 220.0 … 260.0 in steps of 0.1.
- **Trigonometry.** `__sincosf_stret` returns the sine in `s0` and the cosine in `s1`.
  Ghidra often prints the cosine operand as a constant or `extraout_s1`. Formulas in this
  catalogue that use both were checked in the disassembly.

## 3. Custom display models

The seven models are `BCDisplayObject` subclasses that draw their own geometry. Their drawing
is specified in `docs/re/rendering.md` §8. Each one belongs to exactly one logic class:

| Model | Owner | Created in | Data the owner passes in | Drawing |
|---|---|---|---|---|
| `RainbowModel` | `RainbowLogic` | `PocketGodViewController::loadGame @ 0x10006a030` (`initWithId:@"Rainbow" pathList:mpRainbowList`) | the shared point list `mpRainbowList`, appended by `(Game)::drawRainbow:` and read, copied and cleared by `RainbowLogic` through `pathList` | §8.2 |
| `LightningModel` | `LightningLogic` | `PocketGodViewController(Game)::triggerLightning @ 0x100160f74`, `(Game)::hitPygmyWithWarlockLightning: @ 0x100161818` (`initWithId:@"Lightning" pathList:mpLightningList`) | a copy of the drawn `std::list<VECTOR4>`, in which the model writes each segment's angle to `w` | §8.4 |
| `LocustModel` | `LocustLogic` | `PocketGodViewController::loadGame @ 0x10006a030` (`initWithId:@"Locust"`) | `setLocustPointArray:count:`, sent to the logic's display object every frame | §8.3 |
| `HurricaneModel` | `HurricaneLogic` | `PocketGodViewController(Game)::startHurricane: @ 0x100162384` (`initWithId:@"Hurricane"`) | five 64-byte wind records (`hurricaneWindPosArray`) that the logic seeds and animates | §8.5 |
| `WhirlPoolModel` | `WhirlPoolLogic` | `PocketGodViewController(Game)::maintainWhirlPool: @ 0x1001627c4` (`initWithId:@"WhirlPool"`) | the wind records (`whirlPoolWindPosArray`) | §8.5 |
| `PathModel` | `PiranhaLogic` | `PiranhaLogic::touchBeganInside:allTouches: @ 0x10000a860` (`initWithId:@"PiranhaPath" pathVector:`) | the drawn `std::vector<VECTOR4>` path (`setPathVector:`) | §8.6 |
| `MagnifyingBeamModel` | `MagnifyingGlassLogic` | `MagnifyingGlassLogic::drag::: @ 0x1000d3984` on the first drag (`initWithId:@"MagnifyingBeam"`, after `DragDisplayMarker`) | `setTopWidth:`, `setBottomWidth:`, `setFocusHeight:` | §8.7 |

## 4. Catalogue

Classes are grouped by feature cluster. Within a cluster they appear in the order of their implementation units (section 5).

### 4.1 Sky, weather and gesture powers

The sky and sea layers of the surface islands (sun, moon, clouds, storm clouds, ocean tiles), short effects (fireworks, the night bat) and the gesture powers the player draws with a finger: lightning, rainbow, hurricane and the underwater whirlpool. The four power models are drawn by custom display objects (section 3). Weather and day/night transitions are driven by the view controller (`setWeatherState:startup:`, `(Game)::startDay`, `startNight`, `startSunset`, `dragClouds:timeStamp:`).

#### `SunLogic`

`SpriteLogic` · `SunLogic.c` · 8 methods · 8 127 B

- **Role**: the sun of the island sky; scales with the sun state and shows the two-layer eclipse overlay when an object passes in front of it.
- **Where**: surface islands (`IslandDisplayGroup/Sun`); inserted after `StarDisplayMarker` on `transitionIn:`.
- **Created**:
  - `setLogic` item(s) `logicId=SunLogic` in `PocketGod`: `IslandDisplayGroup/Sun`
  - stored in `PocketGodViewController.mSunLogic`.
- **Listener**:
  - `<SunLogicListener>` (`mSunLogicListener`): no methods
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
  - the protocol is empty; the view controller is set by `setLogic:::`.
- **Called from**: `setEclipsePercent:obstructionLogic:`, `transitionIn:`, `transitionOut:` ← PocketGodViewController(Game); `setSunState:` ← PocketGodViewController.
- **Data ids**:
  - models: `Eclipse`
  - markers: `StarDisplayMarker`
  - textures: `Eclipse`
- **Persistence**: none.
- **Behaviour**:
  - `setSunState:` @ 0x1000c7930: target scale 1.0 (state 0), 1.5 (1), 0.5 (2); other values ignored. Driven by `PocketGodViewController::setSunState:` @ 0x10007d4a8 (sun drag in `touchMoved:allTouches:`, a menu button).
  - `onGameFrame:` @ 0x1000c7768: during a transition `y += (targetY − y)·0.5`; when `y` equals the target exactly, a transition-out removes the sun and both flags clear. `eclipse += (eclipseTarget − eclipse)·0.13`; while `eclipse > 0` the two eclipse models spin (`rot.z += 0.1` and `−0.13` per frame). `scale.x = scale.y = s + (targetScale − s)·0.2`.
  - `setEclipsePercent:obstructionLogic:` @ 0x1000c7978 (from `(Game)::checkEclipse` @ 0x100159098): `p == 0` hides both `Eclipse` models (two `BCModel`s built in init from texture `Eclipse` with the sun's origin); otherwise, the first time, copies the sun position into both and inserts them right after the sun (index + 1) with alpha 0. Both models' alpha is set to the *target* `p` on every call.
  - `transitionIn: y` (from `startDay`/`startSunset`) inserts the sun after `StarDisplayMarker`; `transitionOut: y` (from `startNight`) only sets the flag and target.

#### `MoonLogic`

`SpriteLogic` · `MoonLogic.c` · 6 methods · 4 405 B

- **Role**: the night moon; brings the star field in and out.
- **Where**: surface islands at night (`IslandDisplayGroup/Moon`).
- **Created**:
  - `setLogic` item(s) `logicId=MoonLogic` in `PocketGod`: `IslandDisplayGroup/Moon`
  - stored in `PocketGodViewController.mMoonLogic`; init removes it from its group.
- **Listener**:
  - `<MoonLogicListener>` (`mMoonLogicListener`): `starsLogic:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
  - `starsLogic:` returns the `StarsLogic` `MultiModelLogic` (created by the `setLogic` alias `StarsLogic`).
- **Called from**: `transitionIn:`, `transitionOut:` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `StarsStartLoop`, `StarsStopLoop`
  - markers: `StarDisplayMarker`
- **Persistence**: none.
- **Behaviour**:
  - `transitionIn: y` @ 0x1000ca900 (from `(Game)::startNight` @ 0x100160388): alpha 0 → target 1, fetch the stars logic, insert moon then stars after `StarDisplayMarker`, stars `StarsStartLoop`.
  - `transitionOut: y` @ 0x1000caa48 (from `startDay`/`startSunset`; only while stars are held): target alpha 0, stars `StarsStopLoop`.
  - `onGameFrame:` @ 0x1000ca788: `y += (targetY − y)·0.5`, `alpha += (targetAlpha − alpha)·0.5`; while `y ≠ targetY` the stars get the alpha; on arrival a transition-out removes moon and stars and forgets the stars logic.

#### `CloudLogic`

`SpriteLogic` · `CloudLogic.c` · 10 methods + 1 item handler · 8 162 B

- **Role**: drifting sky clouds (index 0) and the three doodle clouds of the Ooga Jump sky (indices 1–3).
- **Where**: island sky.
- **Created**:
  - allocated in `PocketGodViewController(Game)::createClouds` @ 0x10015e774
  - `initWithDisplayObject:cloudIndex:` @ 0x1000d4b6c, listener = view controller.
- **Listener**:
  - `<CloudLogicListener>` (`mCloudLogicListener`): `onCloudDie:`
  - set in `PocketGodViewController(Game)::createClouds` @ 0x10015e774
  - `onCloudDie:` is sent when a transitioning-out cloud leaves the screen.
- **Called from**: `initWithDisplayObject:cloudIndex:`, `startDragging`, `stopDragging:`, `transitionIn:`, `transitionOut:` ← PocketGodViewController(Game); `isDoodleCloud` ← PygmyLogic(OogaJump).
- **Data ids**:
  - behaviours: `CloudDrift`
  - textures: `Cloud1Texture`, `Cloud2Texture`, `DoodleCloud`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `drift:::` @ 0x1000d5288 (3 uses).
- **Persistence**: none.
- **Behaviour**:
  - `randomizePosition` @ 0x1000d4cec: `xVelScale = (rand%11 + 5)/10`. Index 0: scale `(rand%51 + 50)/100`, texture `Cloud1Texture` if `rand & 1 == 0` else `Cloud2Texture`, `y = 190`. Indices 1–3: `xVelScale = 1`, texture `DoodleCloud`, scale 1, `y = 350`. Then `z = −277.12` and collision box `(0,0)–(texture width, height)`.
  - Init: index 0 `x = (rand%7801 − 3000)/10`, else `x = (index − 1)·(colXMax + 480)/3`; `vel.x = xVelScale·10`; behaviour `CloudDrift`.
  - `onGameFrame:` @ 0x1000d5044: unless dragging or transitioning out, a speed beyond ±`xVelScale·20` is pulled back by 1 % per frame; then `SpriteLogic` integration.
  - `drift:::` @ 0x1000d5288: during a transition-in waits until the cloud is fully on screen; `minPos.x > 480` → re-randomise and shift left by `colXMax + 480` (or `onCloudDie:` when transitioning out); `maxPos.x < 0` → shift right by `colXMax + 480` (and `onCloudDie:` when transitioning out). Never completes.
  - `transitionIn: v` (`(Game)::transitionInClouds:::`): `vel.x = xVelScale·v`, `x −= 960` if `v > 0` else `x += 960`. `stopDragging: v` / `transitionOut: v`: `vel.x = xVelScale·v`. `startDragging` zeroes the speed (`(Game)::dragClouds:timeStamp:`). `isDoodleCloud` = index ≠ 0 (asked by `PygmyLogic(OogaJump)`).

#### `StormCloudLogic`

`SpriteLogic` · `StormCloudLogic.c` · 11 methods + 1 item handler · 7 297 B

- **Role**: the front and behind storm-cloud bands (`BCTileModel`s) of the weather system.
- **Where**: island sky (`IslandDisplayGroup/FrontStormCloud`, `/BehindStormCloud`).
- **Created**:
  - `setLogic` alias `logicId=BehindStormCloudLogic`: `IslandDisplayGroup/BehindStormCloud`
  - `setLogic` alias `logicId=FrontStormCloudLogic`: `IslandDisplayGroup/FrontStormCloud`
  - stored in `mFrontStormCloudLogic` / `mBehindStormCloudLogic`, then `setFrontStormCloud:posVelScale:` (YES, 1.0) / (NO, 0.8 = `0x3f4ccccd`) — `setLogic:::` @ 0x10015c11c (the boot `setLogic` summary in `docs/re/sequences.md` §3.14 says 0.5 for both behind layers; 0.5 is only the ocean's).
- **Listener**: none (no listener ivar).
- **Called from**: `applyDragOffset:`, `setFrontStormCloud:posVelScale:`, `startDragging`, `stopDragging:`, `transitionIn:`, `transitionOut:` ← PocketGodViewController(Game); `startVisible` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `BehindStormCloudInit`, `BehindStormCloudTransitionIn`, `FrontStormCloudInit`, `FrontStormCloudTransitionIn`
  - markers: `CloudBehindDisplayMarker`, `CloudFrontDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `startTransitionIn:::` @ 0x1001a8944 (2 uses).
- **Persistence**: none.
- **Behaviour**:
  - Init: `resetTransitionInvisible`, removed from its group. `startVisible` (from `PocketGodViewController::setWeatherState:startup:` @ 0x10007d5c4): `vel.x = posVelScale·20`, `resetTransitionVisible`, insert after `CloudFrontDisplayMarker` (front) / `CloudBehindDisplayMarker` (behind).
  - `transitionIn: v`: `vel.x = posVelScale·v`, insert, behaviour `FrontStormCloudTransitionIn` / `BehindStormCloudTransitionIn` (whose `startTransitionIn:::` starts the tile model's `startTransitionInLeft/Right` by the sign of the speed). `transitionOut: v`: `vel.x = posVelScale·v`, `startTransitionOutLeft` when `v > 0` else `…Right`.
  - `onGameFrame:` @ 0x1001a84e4: transition-in polls `transitionInLeft` (vel > 0) / `transitionInRight` until YES; transition-out polls `transitionOutLeft/Right` and removes the object when done; otherwise, unless dragging, the speed is pulled toward ±`posVelScale·20` by 5 % per frame. Then `SpriteLogic` integration. `mbDragging` is a float ivar used as a flag (1.0/0).
  - `applyDragOffset: dx` adds to `pos.x`; `stopDragging: v` sets `vel.x = posVelScale·v` (`(Game)::dragClouds:timeStamp:`).

#### `OceanLogic`

`SpriteLogic` · `OceanLogic.c` · 7 methods · 2 852 B

- **Role**: the scrolling front and behind ocean tile layers.
- **Where**: surface islands (`IslandDisplayGroup/FrontOceanTiles`, `/BehindOceanTiles`).
- **Created**:
  - `setLogic` alias `logicId=BehindOceanLogic`: `IslandDisplayGroup/BehindOceanTiles`
  - `setLogic` alias `logicId=FrontOceanLogic`: `IslandDisplayGroup/FrontOceanTiles`
  - stored in `mFrontOceanLogic` / `mBehindOceanLogic`, then `setFrontOcean:posVelScale:` (YES, 1.0) / (NO, 0.5); behaviours `FrontOceanInit` / `BehindOceanInit`.
- **Listener**: none.
- **Called from**: `applyDragOffset:`, `setFrontOcean:posVelScale:`, `startDragging`, `stopDragging:` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `BehindOceanInit`, `FrontOceanInit`
- **Persistence**: none.
- **Behaviour**: same speed clamp as `StormCloudLogic` (toward ±`posVelScale·20`, 5 % per frame, skipped while dragging; `onGameFrame:` @ 0x100013884); `applyDragOffset: dx` adds to `pos.x`, `stopDragging: v` sets `vel.x = posVelScale·v` (driven by `(Game)::dragOcean:timeStamp:` @ 0x10015fee8).

#### `FireworkLogic`

`SpriteLogic` · `FireworkLogic.c` · 5 methods · 8 386 B

- **Role**: one firework burst — 10 sparks expanding in a ring and fading.
- **Where**: celebrations (island sky).
- **Created**:
  - allocated in `PocketGodViewController(Game)::fireworks:::` @ 0x10015a7c8
  - by the `fireworks` sequence item; listener = view controller.
- **Listener**:
  - `<FireworkLogicListener>` (`mFireworkLogicListener`): `onFireworkDie:`
  - set in `PocketGodViewController(Game)::fireworks:::` @ 0x10015a7c8
- **Data ids**:
  - behaviours: `FireworkPop`
  - models: `Firework`
  - textures: `FireworkSpark`
- **Persistence**: none.
- **Behaviour**: Init @ 0x1000d4290: 10 `Firework` models (texture `FireworkSpark`) copying the burst's colour, position and origin, inserted after it; the burst's own texture is cleared; distance 0 → target `(rand%501 + 500)/10`, alpha 1 → 1; behaviour `FireworkPop`. `onGameFrame:` @ 0x1000d46a0: `t += dt`; `t > 0.7` → target alpha 0; `t > 1.3` → `die` (remove sparks, `onFireworkDie:`); otherwise distance and alpha ease by 10 %/frame and spark *i* of *n* goes to `(x + cos θᵢ·d, y + sin θᵢ·d)`, `θᵢ = i·2π/n` (from the disassembly: `__sincosf_stret` returns sin in s0, cos in s1; the decompiled x term is wrong).

#### `BatLogic`

`SpriteLogic` · `BatLogic.c` · 15 methods + 7 item handlers · 23 520 B

- **Role**: the vampire bat easter egg: a bat flies out of the moon, bites pygmies and can be swatted into the sea.
- **Where**: surface islands at night.
- **Created**:
  - allocated in `PocketGodViewController(Game)::checkForBatTapping:` @ 0x100163740
  - when no pygmy is a vampire, no bat exists and the moon is tapped a third time (`mBatTapTouchCount == 3`): a `Bat` `BCMultiModel` (box from texture `BatForward_01`) at the moon position + (0, 40), after `DragDisplayMarker`, selectable → `mBatLogic`, listener = view controller.
- **Listener**:
  - `<BatLogicListener>` (`mBatLogicListener`): `isOceanEnabled`, `isGravityEnabled`, `pygmyLogicArray:`, `onBatDie:`
  - set in `PocketGodViewController(Game)::checkForBatTapping:` @ 0x100163740
- **Talks to**: `PygmyLogic` (`bitByBat:`, biteable checks); hurricane hooks `pickedUpByHurricane:`, `droppedByHurricane:`.
  - also sent in code: `PygmyLogic` (`canBeBitByBat`).
- **Data ids**:
  - behaviours: `BatDie`, `BatDroppedByHurricane`, `BatFall`, `BatFlyFromMoon`, `BatFlyToPygmy`, `BatFlyToPygmyFromBack`, `BatIdle`, `BatPickedUpByHurricane`, `BatSplash`
  - groups: `IslandDisplayGroup`
  - markers: `DragDisplayMarker`, `IslandDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `flyFromMoonToFront:::` @ 0x100115e58 (1 use), `flyFromFrontToIsland:::` @ 0x100116004 (1 use), `hover:::` @ 0x100116264 (2 uses), `flyToPygmy:::` @ 0x100116618 (2 uses), `flyOffScreen:::` @ 0x1001169f8 (1 use), `fall:::` @ 0x100116ac0 (134 uses), `die:::` @ 0x100116c2c (119 uses).
- **Persistence**: none.
- **Behaviour**: flight is handler-driven (`flyFromMoonToFront`, `flyFromFrontToIsland`, `hover`, `flyToPygmy`, `flyOffScreen`, `fall`; constants in sequences.md §5). `onGameFrame:` @ 0x100115bb8: the shared gravity rule (§2.2) and Euler integration; `y < −100` or `y > 820` → `BatDie`; while held by the hurricane `rot.z += 45` per frame. A tap (`touchBeganInside:allTouches:` while not falling, not coming from the moon, not in water) zeroes the speed and plays `BatFall`. `droppedByHurricane:` re-inserts the bat before `IslandDisplayMarker` when it is behind the island plane (`z < −277.12`).

#### `LightningLogic`

`SpriteLogic` · `LightningLogic.c` · 3 methods + 2 item handlers · 8 799 B

- **Role**: one lightning bolt drawn by the player (or cast by the warlock), with glow and branch sprites; owns **`LightningModel`** (drawing: `docs/re/rendering.md` §8.4).
- **Where**: surface islands (lightning god power, warlock pygmy).
- **Created**:
  - allocated in `PocketGodViewController(Game)::hitPygmyWithWarlockLightning:` @ 0x100161818, `PocketGodViewController(Game)::triggerLightning` @ 0x100160f74
  - the creators first build `LightningModel initWithId:@"Lightning" pathList:mpLightningList`, then the logic → `mLightningLogic`, listener = view controller; a `FingerTrailLogic` follows the finger while drawing (`(Game)::startDrawLightning:`).
- **Listener**:
  - `<LightningLogicListener>` (`mLightningLogicListener`): `onLightningDie:`
  - set in `PocketGodViewController(Game)::hitPygmyWithWarlockLightning:` @ 0x100161818, `PocketGodViewController(Game)::triggerLightning` @ 0x100160f74
- **Talks to**: `LightningModel` (`pathList`).
- **Data ids**:
  - behaviours: `LightningStrike`
  - models: `LightningGlow`, `LightningHighlight`
  - textures: `LightningBranch_01`, `LightningGlow_01`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `blinkLightning:::` @ 0x1000c924c (1 use), `die:::` @ 0x1000c9424 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - `initWithDisplayObject:` @ 0x1000c8d04 reads the model's `std::list<VECTOR4>` (the model stores each segment's angle in `w`). With `n ≥ 7` points it picks branch indices `b1 = rand % (n/2 − 2) + 1` and `b2 = rand % (n − n/2 − 3) + n/2 + 1` (integer halves; `n < 7` → both 0). Nodes 0 and `n − 1` get a `LightningGlow` model (texture `LightningGlow_01`) centred on the point; nodes `b1`/`b2` get a `LightningHighlight` model (texture `LightningBranch_01`) with origin (−13, −34) and `rot.z = w·180/π` (+180 for `b1`). All at `z = −277.12` in the bolt's group, kept in `mHighlightArray`; behaviour `LightningStrike`.

#### `LightningModel`

`BCDisplayObject` · `LightningModel.c` · 7 methods · 8 821 B + 684 B of free helper functions

- **Role**: custom display object drawing a lightning bolt as three triangle strips (widths 9, 6, 2) along a copied `std::list<VECTOR4>`; writes each segment's angle into the node's `w` (drawing spec: `docs/re/rendering.md` §8.4). Owned by **`LightningLogic`**.
- **Created**:
  - allocated in `PocketGodViewController(Game)::hitPygmyWithWarlockLightning:` @ 0x100161818, `PocketGodViewController(Game)::triggerLightning` @ 0x100160f74
  - `PocketGodViewController(Game)::triggerLightning` and `hitPygmyWithWarlockLightning:` (`initWithId:@"Lightning" pathList:mpLightningList`); the free function FUN_1000bfe3c is the list copy.
- **Listener**: none.
- **Called from**: `initWithId:pathList:` ← PocketGodViewController(Game); `pathList` ← LightningLogic.
- **Persistence**: none.

#### `RainbowLogic`

`SpriteLogic` · `RainbowLogic.c` · 18 methods + 7 item handlers · 30 757 B + 2 644 B of free helper functions

- **Role**: the rainbow god power: the player draws a rainbow, sparkles trail the finger, pygmies stare at it and one at a time gushes over it; rain falls during storms; story mode replays a stock arc or the player's last rainbow. Owns **`RainbowModel`** (drawing: `docs/re/rendering.md` §8.2).
- **Where**: surface islands.
- **Created**:
  - allocated in `PocketGodViewController::loadGame` @ 0x10006a030
  - `PocketGodViewController::loadGame` first builds `RainbowModel initWithId:@"Rainbow" pathList:mpRainbowList` (`z = −277.12`); logic → `mRainbowLogic`, listener = view controller.
- **Listener**:
  - `<RainbowLogicListener>` (`mRainbowLogicListener`): `isStormActive`, `pygmyLogicArray:`
  - set in `PocketGodViewController::loadGame` @ 0x10006a030
- **Talks to**: `PygmyLogic` (`startStaringAtRainbow`, `isStaringAtRainbow`, `gushOverRainbow`, `stopStaringAtRainbow`; the pygmy calls back `gushOverRainbowComplete:`); `RainbowGlow` sparkles are plain `MultiModelLogic`s whose listener is the rainbow (`onMultiModelEvent:eventId:` removes a sparkle on `RainbowGlowComplete`).
- **Called from**: `drawRainbow:`, `drawRainbowComplete`, `startDrawRainbow` ← PocketGodViewController(Game); `gushOverRainbowComplete:`, `isRainbowActive` ← PygmyLogic(Island); `storyModeActivateMyRainbow`, `storyModeActivateRainbow`, `storyModeClearRainbow`, `storyModeDeactivateRainbow` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `RainbowGlow`, `RainbowStart`, `RainbowStoryModeActivate`, `RainbowStoryModeActivateMine`, `RainbowStoryModeClear`, `RainbowStoryModeDeactivate`, `RainbowStrike`
  - models: `Rain`, `RainbowGlow`
  - groups: `IslandDisplayGroup`
  - markers: `CloudFrontDisplayMarker`, `FireworksDisplayMarker`
  - textures: `Raindrop`
  - event ids: `RainbowGlowComplete`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `storyModeDrawMyRainbow:::` @ 0x10002126c (1 use), `storyModeDrawRainbow:::` @ 0x100021384 (1 use), `storyModeComplete:::` @ 0x10002149c (2 uses), `waitForGushingPygmy:::` @ 0x100021510 (1 use), `lookForGushingPygmy:::` @ 0x10002152c (1 use), `rainbowStrikeComplete:::` @ 0x100021560 (1 use), `stareAtRainbowComplete:::` @ 0x1000215b4 (1 use).
- **Persistence**: none.
- **Behaviour**:
  - Init: 10 `Rain` models (texture `Raindrop`) at `x = (rand%5301 − 500)/10`, `y = (rand%5001 + 3200)/10`, `z = −277.12`.
  - `onGameFrame:` @ 0x100020044 (no shutdown check): `nextGlowTime −= dt`; rain alpha eases (20 %/frame) toward the rainbow's alpha while `isStormActive`, else toward 0; each drop gets that alpha, `rot.z = 30`, `y −= 30`, `x += 5`, and respawns at `x = (rand%5301 − 500)/10`, `y = 320` when below −30.
  - `startDrawRainbow` @ 0x100020ca0: on first activation inserts the rainbow after `FireworksDisplayMarker` and the rain after `CloudFrontDisplayMarker`, sends `startStaringAtRainbow` to every pygmy and picks a gushing pygmy (`findGushingPygmyLogic`: uniform among `isStaringAtRainbow` pygmies, then `gushOverRainbow`). Always: drawing flag, last point (0,0), clear sparkles, `RainbowStart`.
  - `drawRainbow: p` @ 0x100020f64: angle = `atan2(p − last)` in degrees; when `nextGlowTime < 0` (every 0.07 s) stores `p` as last point and shows a sparkle there (reuses one that is not in a group, else creates a `RainbowGlow` multimodel while fewer than 201 exist) with that angle in `IslandDisplayGroup`, behaviour `RainbowGlow`.
  - `drawRainbowComplete` @ 0x100021018 copies the model's path into `mpRainbowListCopy` and plays `RainbowStrike`. `storyModeActivateMyRainbow` uses `RainbowStoryModeActivateMine` only when the copy has ≥ 3 points, else `RainbowStoryModeActivate`; story drawing handlers in sequences.md.

#### `RainbowModel`

`BCDisplayObject` · `RainbowModel.c` · 7 methods · 11 658 B

- **Role**: custom display object drawing the rainbow ribbon along the shared path list `mpRainbowList` (drawing spec: `docs/re/rendering.md` §8.2). Owned by **`RainbowLogic`**.
- **Created**:
  - allocated in `PocketGodViewController::loadGame` @ 0x10006a030
  - `PocketGodViewController::loadGame` (`initWithId:@"Rainbow" pathList:mpRainbowList`, `z = −277.12`).
- **Listener**: none.
- **Called from**: `initWithId:pathList:` ← PocketGodViewController.
- **Persistence**: none.
- **Behaviour**: `RainbowLogic` reads, copies and clears the list through `pathList`; the view controller appends the drawn points (`(Game)::drawRainbow:`).

#### `HurricaneLogic`

`SpriteLogic` · `HurricaneLogic.c` · 16 methods + 1 item handler · 25 738 B

- **Role**: the hurricane god power: wind ribbons, rain and debris particles; blows objects over and lifts them into a spinning vortex. Owns **`HurricaneModel`** (drawing: `docs/re/rendering.md` §8.5).
- **Where**: surface islands.
- **Created**:
  - allocated in `PocketGodViewController(Game)::startHurricane:` @ 0x100162384
  - when `mHurricaneLogic` is nil the creator builds `HurricaneModel initWithId:@"Hurricane"`, the logic (listener = view controller) and a `FingerTrailLogic` on a `WindGlow` multimodel (`mHurricaneTrailLogic`).
- **Listener**:
  - `<HurricaneLogicListener>` (`mHurricaneLogicListener`): `spriteLogicArray:`, `onHurricaneCantPickUpObjects:`, `onHurricaneDie:`
  - set in `PocketGodViewController(Game)::startHurricane:` @ 0x100162384
- **Talks to**: every `SpriteLogic` returned by the listener's `spriteLogicArray:` through the hooks `isBlownByHurricane`, `blownByHurricane:`, `unblownByHurricane:`, `isPickedUpByHurricane`, `pickedUpByHurricane:`, `droppedByHurricane:` and the sprite's `HurricaneSpinState`; picked sprites call back `dropSpriteLogic:`.
  - also sent in code: `HurricaneModel` (`hurricaneWindPosArray`).
- **Called from**: `hurricaneActive`, `setTouchActive:`, `stopHurricane` ← PocketGodViewController, PocketGodViewController(Game); `setWindVel:`, `windActive` ← PocketGodViewController(Game); `stopHurricaneNow` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `HurricaneDie`, `HurricaneStart`, `HurricaneStop`
  - models: `Particle`, `Rain`
  - markers: `CloudFrontDisplayMarker`
  - textures: `Raindrop`, `WindLeaf`, `WindRock1`, `WindRock2`, `WindWater`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `die:::` @ 0x100115308 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - `HurricaneSpinState = {int stall; float radius, radiusRot, radiusRotVel, yPos, yPosRot, yPosRotVel, angle, angleVel, strength, strengthVel}` (44 bytes). Init @ 0x100113cd8 seeds the model's 5 wind records (stride 0x40: `[0] = (rand%401 + 2200)/10`, `[1] = 86`, `[5] = 1`, `[9] = 0.2`, `[10] = (rand%201 + 500)/10`, `[11] = (rand%21 + 30)/100`, `[12] = (rand%1801)/10`, `[13] = (rand%151 + 150)/100`, `[14] = 2`, `[15] = rand%5 + 6`, others 0), 10 `Rain` models and 4 `Particle` models (`WindLeaf`, `WindRock1`, `WindRock2`, `WindWater`) at `x = (rand%801 + 3400)/10`, `y = (rand%1201 + 1200)/10` with their own spin states (`radius = x − 240`).
  - `setWindVel:` (each circling touch): clears the stop flag, `windVel = min(3, windVel + 0.03)`. `onGameFrame:` @ 0x100114a04: wind record `[12] += [13]·windVel` (reset to 0 above 180); `windVel −= 0.01` (−0.05 more while stopping), floor 0; model alpha `0.5·min(1, 2·windVel)`, rain/particle alpha `min(1, 2·windVel)`, particles `rot.z += 10`, rain falls as in `RainbowLogic`. Crossing 0.5 plays `HurricaneStart`/`HurricaneStop`; `windVel ≥ 2` enables pick-up.
  - Per sprite (`countdownStall:`: a random 1–30 frame delay that ends early while stopping): pick-up on → `pickedUpByHurricane:`; an accepted sprite gets `radius = |x − 240|`, `angle = 0` (right) or π (left), `yPos = (rand%1201 + 1200)/10`, `angleVel = (rand%3 + 1)/10`, `strength = 0`, `strengthVel = (rand%32 + 20)/1000`. Pick-up off → held sprites are released (`droppedByHurricane:`) after their delay; wind on/off toggles `blownByHurricane:` / `unblownByHurricane:` after the delay.
  - `spinDisplayObject:hurricaneSpinState:` @ 0x100114578: `radius ≥ 20`; `x = radius·cos(angle) + 240`; `z = hurricane.z + sin(angle)·radius·strength·0.5`; when `strength > 0.1`: `y += (yPos − y)·0.05`; `angle += angleVel`; `strength = min(1, strength + strengthVel)`.
  - No pick-up and nothing held → `onHurricaneCantPickUpObjects:` every frame; touch inactive and `windVel == 0` → `HurricaneDie` (else spin sprites and particles and integrate).

#### `HurricaneModel`

`BCDisplayObject` · `HurricaneModel.c` · 9 methods · 5 094 B

- **Role**: custom display object drawing the hurricane wind ribbons from five wind records (`hurricaneWindPosArray`, 64-byte records seeded and animated by `HurricaneLogic`; drawing spec `docs/re/rendering.md` §8.5). Owned by **`HurricaneLogic`**.
- **Created**:
  - allocated in `PocketGodViewController(Game)::startHurricane:` @ 0x100162384
  - `PocketGodViewController(Game)::startHurricane:` (`initWithId:@"Hurricane"`).
- **Listener**: none.
- **Called from**: `hurricaneWindPosArray` ← HurricaneLogic.
- **Persistence**: none.

#### `WhirlPoolLogic`

`SpriteLogic` · `WhirlPoolLogic.c` · 13 methods + 1 item handler · 14 464 B

- **Role**: the underwater whirlpool power, a 2-D variant of the hurricane. Owns **`WhirlPoolModel`** (drawing: `docs/re/rendering.md` §8.5).
- **Where**: underwater locations.
- **Created**:
  - allocated in `PocketGodViewController(Game)::maintainWhirlPool:` @ 0x1001627c4
  - when `mWhirlPoolLogic` is nil the creator builds `WhirlPoolModel initWithId:@"WhirlPool"`, the logic (listener = view controller) and a `FingerTrailLogic` on `WindGlow` (`mWhirlPoolTrailLogic`).
- **Listener**:
  - `<WhirlPoolLogicListener>` (`mWhirlPoolLogicListener`): `spriteLogicArray:`, `onWhirlPoolCantPickUpObjects:`, `onWhirlPoolDie:`
  - set in `PocketGodViewController(Game)::maintainWhirlPool:` @ 0x1001627c4
- **Talks to**: sprites via `isPickedUpByWhirlPool`, `pickedUpByWhirlPool:`, `droppedByWhirlPool:` and their `WhirlPoolSpinState`; sprites call back `dropSpriteLogic:`.
  - also sent in code: `WhirlPoolModel` (`whirlPoolWindPosArray`).
- **Called from**: `setTouchActive:` ← PocketGodViewController, PocketGodViewController(Game); `setWindVel:` ← PocketGodViewController(Game); `stopWhirlPoolNow` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `WhirlPoolDie`, `WhirlPoolStart`, `WhirlPoolStop`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `die:::` @ 0x100138398 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Wind model as in the hurricane (`setWindVel:` +0.03, max 3; −0.01/frame, −0.05 more while stopping) with records seeded at (240, 160) (`[10] = (rand%401 + 1000)/10`, `[11] = (rand%21 − 100)/100`, `[13] = (rand%21 + 80)/100`, `[15] = (rand & 3) + 5`). Model alpha `0.7·min(1, 2·windVel)`; `WhirlPoolStart`/`WhirlPoolStop` at 0.5; pick-up at ≥ 2.
  - Picked sprite (`onGameFrame:` @ 0x100137cf8): `r = |p − (240,160)|`, `angle = atan2`, target radius random in `[min(r, 50)·10, (r ≤ 140 ? r·10 + 1 : 1401))/10`, `angleVel = (rand%8 + 3)/100`, `strength 0`, `strengthVel = (rand%32 + 20)/1000`. `spinDisplayObject:whirlPoolSpinState:` @ 0x100137a20: `radius ≥ 20`, `pos = (240,160) + radius·(cos, sin)(angle)`, `angle += angleVel`, `radius += (target − radius)·0.2`, `strength = min(1, strength + strengthVel)`.
  - `stopWhirlPoolNow` releases every held sprite and plays `WhirlPoolDie`; idle (no touch, `windVel == 0`) also plays `WhirlPoolDie`.

#### `WhirlPoolModel`

`BCDisplayObject` · `WhirlPoolModel.c` · 9 methods · 4 970 B

- **Role**: custom display object drawing the whirlpool ribbons (`whirlPoolWindPosArray`; `docs/re/rendering.md` §8.5). Owned by **`WhirlPoolLogic`**.
- **Created**:
  - allocated in `PocketGodViewController(Game)::maintainWhirlPool:` @ 0x1001627c4
  - `PocketGodViewController(Game)::maintainWhirlPool:` (`initWithId:@"WhirlPool"`).
- **Listener**: none.
- **Called from**: `whirlPoolWindPosArray` ← WhirlPoolLogic.
- **Persistence**: none.

#### `FingerTrailLogic`

`SpriteLogic` · `FingerTrailLogic.c` · 4 methods · 1 412 B

- **Role**: a glow that eases toward the finger while a gesture power is drawn (lightning, rainbow, hurricane, whirlpool, underwater drag powers).
- **Where**: wherever the gesture happens.
- **Created**:
  - allocated in `PocketGodViewController(Game)::drawRainbow:` @ 0x100161a9c, `PocketGodViewController(Game)::maintainWhirlPool:` @ 0x1001627c4, `PocketGodViewController(Game)::startDragUnderwaterPower:` @ 0x100162bac, `PocketGodViewController(Game)::startDrawLightning:` @ 0x100160d3c, `PocketGodViewController(Game)::startHurricane:` @ 0x100162384
  - each creator wraps a `WindGlow` `BCMultiModel`.
- **Listener**: none.
- **Called from**: `setTargetXPos:yPos:` ← PocketGodViewController(Game).
- **Persistence**: none.
- **Behaviour**: `onGameFrame:` @ 0x100117028: `pos.xy += (target − pos.xy)·0.5`; `setTargetXPos:yPos:` @ 0x1001170f8 sets the target (sent by the view controller's `(Game)` gesture code).

### 4.2 Main-island objects

Props of the main island and its sand and rock variants. Coconut, wood, spear, boulder and magnifying glass are throwables built on the pattern of section 2.2. The bonfire, ant hill, volcano, outhouse and octopus statue are interactive props that hold or affect pygmies.

#### `CoconutLogic`

`SpriteLogic` · `CoconutLogic.c` · 38 methods + 5 item handlers · 35 904 B

- **Role**: the coconut that grows on the palm tree; it can be shaken off, dragged and thrown at pygmies, split open and eaten. The reference implementation of the throwable-object pattern (§2.2).
- **Where**: main island (`IslandDisplayGroup/Coconut`); extra halves are created when a coconut splits on a pygmy.
- **Created**:
  - `setLogic` item(s) `logicId=CoconutLogic` in `PocketGod`: `IslandDisplayGroup/Coconut`
  - allocated in `PocketGodViewController(Game)::onCoconutHitPygmy:` @ 0x1001657a4
  - the boot instance is added to `mCoconutLogicArray` only (not `mLogicArray`); `PocketGodViewController(Game)::onCoconutHitPygmy:` @ 0x1001657a4 creates the second half when a coconut splits (listener = view controller).
- **Listener**:
  - `<CoconutLogicListener>` (`mCoconutLogicListener`): `isStoryActive`, `isGravityEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onCoconutDrag:`, `onCoconutDragComplete:`, `onCoconutHitPygmy:`, `onCoconutHitGround:`, `onCoconutDie:`
  - set in `PocketGodViewController(Game)::onCoconutHitPygmy:` @ 0x1001657a4, `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic` (`hitByCoconut:`, `stopEating`; pygmies call `startEating:`, `dropWhileEating`, `pickUpForEating`, `finishEating`, `isEdible`; Charlie calls `thrownByCharlie:`).
- **Called from**: `dropWhileEating`, `finishEating`, `isEdible`, `pickUpForEating`, `startEating:` ← PygmyLogic(Island); `fallOffTree`, `grow`, `isInAir`, `isOnTree`, `isSplit`, `split:` ← PocketGodViewController(Game); `thrownByCharlie:` ← PygmyLogic(Charlie).
- **Data ids**:
  - behaviours: `CoconutBounce`, `CoconutDie`, `CoconutDrag`, `CoconutDropWhileEating`, `CoconutDroppedByHurricane`, `CoconutEnterIsland`, `CoconutExitIsland`, `CoconutFall`, `CoconutGrow`, `CoconutPickedUpByHurricane`, `CoconutSplash`, `CoconutSplit`, `CoconutStopSlide`, `CoconutTiltSlide`
  - markers: `DragDisplayMarker`, `IslandDisplayMarker`, `SmallObjectsMarker`, `StartCoconutDisplayMarker`
  - textures: `CoconutHalf`, `Coconut_06`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `slide:::` @ 0x100189990 (20 uses), `drag:::` @ 0x100189a90 (40 uses), `fall:::` @ 0x100189c54 (134 uses), `fallFromHurricane:::` @ 0x100189da0 (9 uses), `die:::` @ 0x100189ee4 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Shared pattern of §2.2 with slide friction 0.04, floor `y = 92`, bounce behaviour `CoconutBounce`, landing → listener `onCoconutHitGround:`, out of range → `CoconutDie`.
  - Starts on the tree (`mbOnTree`, ignores gravity, tree layer). `grow` → `CoconutGrow`; `fallOffTree` → `CoconutFall`. `split:` @ 0x100188cc4 switches the texture to `CoconutHalf`, plays `CoconutSplit`, makes it edible; with YES also inserts it after `SmallObjectsMarker` and pushes `vel.x` by ∓100 (away from its motion).
  - `checkPygmyCollision` @ 0x100188e00 (each frame, unless being eaten or `mbDontHitUntilBounce`): while falling (`vel.y < 0`) hits the first pygmy with `|(x, y) − (px, py + 31)| < 20` that accepts `hitByCoconut:`; then `vel = (dx·10, 700)`, a drag is cancelled (`CoconutSplit`), listener `onCoconutHitPygmy:`.
  - `isEdible` = split ∧ on island ∧ not being eaten ∧ active. `dropWhileEating` / `thrownByCharlie:` (`vel = (pygmyScale.x·−200, 400)`) set `mbDontHitUntilBounce` and play `CoconutDropWhileEating`.
  - `die:::` restores texture `Coconut_06` (the whole coconut).

#### `WoodLogic`

`SpriteLogic` · `WoodLogic.c` · 31 methods + 6 item handlers · 33 413 B

- **Role**: the three firewood logs: pygmies carry them to the bonfire pile; they can be dragged, thrown and floated.
- **Where**: main island (`IslandDisplayGroup/Wood1…3`).
- **Created**:
  - `setLogic` item(s) `logicId=WoodLogic` in `PocketGod`: `IslandDisplayGroup/Wood1`, `IslandDisplayGroup/Wood2`, `IslandDisplayGroup/Wood3`
  - added to `mWoodLogicArray`, listener = view controller.
- **Listener**:
  - `<WoodLogicListener>` (`mWoodLogicListener`): `isStoryActive`, `isGravityEnabled`, `isWoodEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `bonFireLogic:`, `onWoodDrag:`, `onWoodDragComplete:` (never sent by this class: `pygmyLogicArray:`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `BonFireLogic` (`addWoodLogic:`, `removeWoodLogic:`, `nextWoodXOffset`, `nextWoodYOffset`; the fire calls `fallOutOfBonFire`); `PygmyLogic(Island)` calls `startCarrying:`, `pickUpForCarrying`, `dropWhileCarrying`, `throwOnPile`, `isAvailable`.
  - also sent in code: `PygmyLogic` (`stopCarryingFromWood`).
- **Called from**: `dropWhileCarrying`, `isAvailable`, `pickUpForCarrying`, `startCarrying:`, `throwOnPile` ← PygmyLogic(Island); `enterIsland:`, `exitIslandNow` ← PocketGodViewController(Game); `fallOutOfBonFire` ← BonFireLogic.
- **Data ids**:
  - behaviours: `WoodBounce`, `WoodDie`, `WoodDrag`, `WoodDropWhileCarrying`, `WoodDroppedByHurricane`, `WoodEnterIsland`, `WoodFall`, `WoodFallIgnoreIsland`, `WoodFallOutOfBonFire`, `WoodPickedUpByHurricane`, `WoodSplash`, `WoodStopSlide`, `WoodThrowOnPile`, `WoodTiltSlide`
  - markers: `DragDisplayMarker`, `IslandDisplayMarker`, `SmallObjectsMarker`
  - other strings: `Wood`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `slide:::` @ 0x10017b444 (20 uses), `drag:::` @ 0x10017b544 (40 uses), `throwOnPile:::` @ 0x10017b708 (1 use), `fall:::` @ 0x10017b898 (134 uses), `fallFromHurricane:::` @ 0x10017ba08 (9 uses), `die:::` @ 0x10017bb4c (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Shared pattern of §2.2 with slide friction 0.035, floor `y = 80`, `WoodBounce` / `WoodDie`. Init sets the apocalypse cloud style to `Wood` and starts outside the island (`mbExitIsland`).
  - `enterIsland:` @ 0x10017a998 (unless falling): remembers its first x as `mStartXPos`, resets, drops in from `(startX, 400)`, `WoodEnterIsland`; always shows the apocalypse cloud. `exitIsland: now` hides the cloud and plays `WoodDie` (now) or `WoodFallIgnoreIsland`.
  - `isAvailable` = wood enabled ∧ not in the pile ∧ on the island ∧ not carried ∧ active. `throwOnPile` asks the listener for the bonfire, `vel.y = 100`, `WoodThrowOnPile` (the handler arcs to the next log slot and calls `addWoodLogic:`). `fallOutOfBonFire` @ 0x10017a604: `vel = ((rand%6001 − 3000)/10, 500)`, `WoodFallOutOfBonFire`.

#### `BonFireLogic`

`SpriteLogic` · `BonFireLogic.c` · 31 methods + 1 item handler · 19 963 B

- **Role**: the bonfire: three logs make a pile that lightning can ignite; burning fire warms two pygmies and burns those it is poked at.
- **Where**: main island (`IslandDisplayGroup/BonFire`, inserted before `SmallObjectsMarker`).
- **Created**: `setLogic` item(s) `logicId=BonFireLogic` in `PocketGod`: `IslandDisplayGroup/BonFire`
- **Listener**:
  - `<BonFireLogicListener>` (`mBonFireLogicListener`): `isStoryActive`, `isGravityEnabled`, `isBonFireEnabled`, `fishingPoleLogic:`, `pygmyLogicArray:` (never sent by this class: `isBonFireEnabled`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `WoodLogic` (`fallOutOfBonFire`), `FishingPoleLogic` (`checkProximityToBonFire:` via the listener's `fishingPoleLogic:`), `PygmyLogic` (`hitByBonFire:`, `tossFromBonFire`; pygmies use `isLeft/RightWarmingAvailable:`, `start/stopLeft/RightWarming:`); `isOnFire` is read by Bird, Fish and Pygmy logic; `hitByLightning` comes from `(Game)` lightning.
- **Called from**: `addWoodLogic:`, `nextWoodXOffset`, `nextWoodYOffset`, `removeWoodLogic:` ← WoodLogic; `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland` ← PocketGodViewController; `hasAtLeastOneLog` ← FishingPoleLogic; `hitByLightning` ← PocketGodViewController(Game); `isLeftWarmingAvailable:`, `isRightWarmingAvailable:`, `startLeftWarming:`, `startRightWarming:`, `stopLeftWarming:`, `stopRightWarming:` ← PygmyLogic(Island); `isOnFire` ← BirdLogic, FishLogic, PygmyLogic, PygmyLogic(Apocalypse), PygmyLogic(Island).
- **Data ids**:
  - behaviours: `BonFireCatchFire`, `BonFireDropLog1`, `BonFireDropLog2`, `BonFireDropLog3`, `BonFireEnterIsland`, `BonFireExitIsland`, `BonFirePoke`, `BonFireRemoveWood`
  - markers: `SmallObjectsMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `die:::` @ 0x10013b338 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - `addWoodLogic:` @ 0x10013a8f0: log *n* (0–2) is bound to animation channel *n + 1* and plays `BonFireDropLog1…3`; then `checkProximityToBonFire:` on the fishing pole. Log slot offsets (`nextWoodX/YOffset`): x = −1, −3.4, 1 and y = 11, 20.9, 22.5 (`DAT_10029da1c` / `DAT_10029da28`), 0 beyond 3 logs.
  - `hitByLightning` @ 0x10013ac94: with exactly 3 logs and not burning → on fire, selectable, `BonFireCatchFire`, returns YES.
  - `removeWoodLogic:` / `pickedUpByHurricane:` (with logs) / tilting the device (`onGameFrame:` @ 0x10013b264: accelerometer roll within ±90°) put the fire out: warming pygmies are tossed (`tossFromBonFire`), channels 1–3 cleared, every log `fallOutOfBonFire`, `BonFireRemoveWood`. `pickedUpByHurricane:` itself always returns NO.
  - Tap while burning (`touchBeganInside:allTouches:` @ 0x10013b378, no story): `BonFirePoke` and `hitByBonFire:` on every pygmy within 35 px horizontally plus both warming pygmies.

#### `SpearLogic`

`SpriteLogic` · `SpearLogic.c` · 34 methods + 5 item handlers · 39 377 B

- **Role**: the spear that appears while the T-Rex or the spider attacks; pygmies catch it and throw it at the monster.
- **Where**: main island (`IslandDisplayGroup/Spear`), dino and graveyard encounters.
- **Created**:
  - `setLogic` item(s) `logicId=SpearLogic` in `PocketGod`: `IslandDisplayGroup/Spear`
  - stored in `mSpearLogicArray`, listener = view controller.
- **Listener**:
  - `<SpearLogicListener>` (`mSpearLogicListener`): `isGravityEnabled`, `isSpearEnabled`, `isTRexActive`, `isSpiderActive`, `islandMinXPos`, `islandMaxXPos`, `tRexLogic:`, `spiderLogic:`, `pygmyLogicArray:`, `onSpearDrag:`, `onSpearDragComplete:`, `onSpearThrow:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `TRexLogic` / `SpiderLogic` (`canBeHitBySpear:`, `hitBySpear:`; they read `throwXPos`); `PygmyLogic` (`canCatchSpear`, `catchSpear:`; the holding pygmy calls `throwSpear` and `dropWhileHolding` on its `mThrowingSpearLogic`; `startHolding:` is never sent).
  - also sent in code: `SpiderLogic` (`isActive`); `TRexLogic` (`isActive`).
- **Called from**: `activate` ← PocketGodViewController(Game), SpiderLogic, TRexLogic; `deactivate`, `throwXPos` ← SpiderLogic, TRexLogic; `dropWhileHolding`, `throwSpear` ← PygmyLogic(Island); `enterIsland:`, `exitIslandNow` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `SpearBounce`, `SpearDie`, `SpearDrag`, `SpearDroppedByHurricane`, `SpearEnterIsland`, `SpearFall`, `SpearFallIgnoreIsland`, `SpearHitSpider`, `SpearHitTRex`, `SpearPickedUpByHurricane`, `SpearPygmyCatch`, `SpearSplash`, `SpearStopSlide`, `SpearThrow`, `SpearTiltSlide`
  - markers: `DragDisplayMarker`, `IslandDisplayMarker`, `SmallObjectsMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `slide:::` @ 0x100170404 (20 uses), `drag:::` @ 0x100170514 (40 uses), `fall:::` @ 0x1001706c8 (134 uses), `fallFromHurricane:::` @ 0x100170854 (9 uses), `die:::` @ 0x100170998 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Shared pattern of §2.2 with slide friction 0.025, floor `y = 72`, `SpearBounce` / `SpearDie`; collisions are off until thrown or dropped.
  - `enterIsland:` activates only while the listener's T-Rex or spider `isActive` and `isSpearEnabled`: `activate` drops it from (150, 400) after `SmallObjectsMarker`, `SpearEnterIsland`. `deactivate` releases hurricane/drag state and plays `SpearFallIgnoreIsland` after `DragDisplayMarker` (or `SpearDie` when in water).
  - `checkPygmyCollision` @ 0x10016f3ec: a falling spear not thrown by a pygmy is caught by a pygmy with `canCatchSpear` within 50 px of `(px, py − 32)`.
  - `throwSpear` @ 0x10016f13c: `vel = (±500 by xScale sign, 800)`, remembers `throwXPos`, `SpearThrow`, listener `onSpearThrow:`.
  - Hits (each frame while thrown): T-Rex circle `(tx + tScale·41, ty + 103)` r 40; spider circles `(sx + sScale·35, sy + 35)` and `(sx − sScale·16, sy + 32)` r 40. On a hit `vel.x ·= −0.5`, `vel.y = 500`, spin, `SpearHitTRex` / `SpearHitSpider`, target `hitBySpear:`.

#### `BoulderLogic`

`SpriteLogic` · `BoulderLogic.c` · 22 methods + 4 item handlers · 32 069 B

- **Role**: the boulder: dropped onto the island, it can be rolled (bowling pygmies over), dropped on pygmies to crush them, or thrown into the sea; it also casts an eclipse.
- **Where**: main island (`IslandDisplayGroup/Boulder`, inserted after `IslandBackDisplayMarker`).
- **Created**:
  - `setLogic` item(s) `logicId=BoulderLogic` in `PocketGod`: `IslandDisplayGroup/Boulder`
  - stored in `mBoulderLogic`, listener = view controller.
- **Listener**:
  - `<BoulderLogicListener>` (`mBoulderLogicListener`): `isStoryActive`, `isGravityEnabled`, `isBoulderEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onBoulderDrag:`, `onBoulderDragComplete:`, `onPygmiesBowled:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic` (`hitByBoulder:xVel:`, `crushedByBoulder:`, `uncrushedByBoulder:dontWait:`, `charlieSmackBoulder:`, `personaPersonality`; `PygmyLogic(Story)` calls `clearPygmyLogic:`); the sun eclipse code calls `setEclipsePercent:`.
- **Called from**: `clearPygmyLogic:` ← PygmyLogic(Story); `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `setEclipsePercent:` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `BoulderBounce`, `BoulderDie`, `BoulderDrag`, `BoulderEnterIsland`, `BoulderFall`, `BoulderFallIgnoreIsland`, `BoulderSplash`, `BoulderStopSlide`, `BoulderTiltSlide`
  - markers: `DragDisplayMarker`, `IslandBackDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `slide:::` @ 0x10001e550 (20 uses), `drag:::` @ 0x10001e664 (40 uses), `fall:::` @ 0x10001e818 (134 uses), `die:::` @ 0x10001e990 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Shared pattern of §2.2 with slide friction 0.025, floor `y = 126`; rolling: on the floor `rot.z` velocity = `vel.x·−3`; a landing bounce scales `vel.x·0.9`, `vel.y·−0.4`; it settles when `|vel.x| < 30` and `vel.y < 30` (`BoulderBounce` when `vel.y > 100`), and on settling reports `onPygmiesBowled:` with the bowled count (then 0). A crushed pygmy pins the boulder's x.
  - `enterIsland: onFloor` @ 0x10001d3c8: from (158, 400) or directly at `y = 126`, `BoulderEnterIsland`.
  - `checkPygmyCollision:` @ 0x10001d720 (inside the island x range, `y ≥ 126`, no crushed pygmy): pygmies within 50 px of `(px, py + 31)`: Charlie (personality 1) at speed > 20 may `charlieSmackBoulder:` → `vel = (±200, 600)`; a falling boulder (`vel.y < −20`, `|vel.x| < 30`) `crushedByBoulder:` → `vel.x = ±100` (`DAT_10029d410`, sign by side) and the pygmy is held; speed > 200 → `hitByBoulder:xVel:(dx·−3)`, count +1, `vel.y = 300`.
  - Eclipse: `eclipse += (target − eclipse)·0.13` each frame and the boulder is tinted `1 − eclipse` (grey) while it covers the sun.

#### `PygmyBottomLogic`

`SpriteLogic` · `PygmyBottomLogic.c` · 8 methods + 2 item handlers · 7 563 B

- **Role**: a detached body part that flies off and falls: the pygmy's bottom (entry 0) and the bones and skulls left by the locust swarm (entries 464–470).
- **Where**: surface islands / apocalypse island.
- **Created**:
  - `createLogicClasses` table entries 0, 464–469 (new `BCMultiModel` `PygmyBottom`, new `BCMultiModel` `LocustPygmySkull1`, new `BCMultiModel` `LocustPygmySkull2`, … (7 entries))
  - entry 0 `PygmyBottom` and entries 464–466 `LocustPygmySkull1…3`, 467–470 `LocustPygmyBone1…4` (all `BCMultiModel`s), listener = view controller.
- **Listener**:
  - `<PygmyBottomLogicListener>` (`mPygmyBottomLogicListener`): `isOceanEnabled`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `PygmyLogic(Apocalypse)` / `PygmyLogic(Charlie)` call `tossBottom:`, `spawnLocustSkull:`, `spawnLocustBone:`.
- **Called from**: `isActive`, `spawnLocustBone:`, `spawnLocustSkull:` ← PygmyLogic(Apocalypse); `tossBottom:` ← PygmyLogic(Apocalypse), PygmyLogic(Charlie).
- **Data ids**:
  - behaviours: `PygmyBottomInit`, `PygmyBottomLocustBone`, `PygmyBottomLocustSkull`, `PygmyDie`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `fall:::` @ 0x100112f88 (134 uses), `die:::` @ 0x100113048 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - `tossBottom: p` @ 0x100113088: copies the pygmy's position and 1.2× its velocity, inserts itself before the pygmy's display object and plays `PygmyBottomInit` (animation `HalfPygmyFall2Anim`, the `fall` item, then a rock splash and `PygmyBottomDie`).
  - `spawnLocustSkull: p` @ 0x100113190 / `spawnLocustBone: p` @ 0x100113300: start at the pygmy's position + (0, 26) / + (0, 50) with `vel = ((rand%4001 − 2000)/10, (rand%501 + 1000)/10)`, active, gravity off, inserted before the pygmy, behaviour `PygmyBottomLocustSkull` / `PygmyBottomLocustBone`. Those sequences only set the animations `PygmyLocustStripToSkeletonSkullAnim` / `PygmyLocustStripToSkeletonBoneAnim` and never run `fall`, so the skull and bones keep their straight-line velocity.
  - `onGameFrame:` @ 0x100112dd4: gravity `(0, −3000)` unless ignored, Euler integration; with the ocean enabled a part below `y = 16` stops there; `y < −100` or `y > 820` plays `PygmyDie` (the pygmies' one-item sequence `die`, which runs this class's `die:::`). `fall:::` @ 0x100112f88: gravity on; with the ocean enabled at `y ≤ 16`: gravity off, stopped at 16, complete.

#### `VolcanoLogic`

`SpriteLogic` · `VolcanoLogic.c` · 15 methods + 1 item handler · 7 609 B

- **Role**: the volcano behind the island; pygmies thrown into it make it rumble and, on the third hit, erupt and rain lava.
- **Where**: main island (`IslandDisplayGroup/Volcano`, after `VolcanoDisplayMarker`).
- **Created**:
  - `setLogic` item(s) `logicId=VolcanoLogic` in `PocketGod`: `IslandDisplayGroup/Volcano`
  - stored in `mVolcanoLogic`, listener = view controller.
- **Listener**:
  - `<VolcanoLogicListener>` (`mVolcanoLogicListener`): `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onEruptionComplete:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `LavaLogic` (spawned by `lavaRain:::` as `NewLava` multimodels after `LavaDisplayMarker`, listener = the volcano, which forwards `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:` to its own listener).
- **Called from**: `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `erupt1`, `erupt2`, `erupt3`, `hit1`, `hit2`, `hit3` ← PocketGodViewController(Game); `exitIsland` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `VolcanoErupt1`, `VolcanoErupt2`, `VolcanoErupt3`, `VolcanoHit1`, `VolcanoHit2`, `VolcanoHit3`, `VolcanoInit`
  - models: `NewLava`
  - markers: `LavaDisplayMarker`, `VolcanoDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `lavaRain:::` @ 0x1000c7130 (4 uses).
- **Persistence**: none.
- **Behaviour**: `PocketGodViewController(Game)::onVolcanoHit: pygmy` @ 0x100164e00 counts hits (`mVolcanoHitCount` 0, 1, 2; the third sets `mbEruption`) and sends `hit1/2/3`, or `erupt1/2/3` when the pygmy's `timeTemplePygmyType` is 4; they play `VolcanoHit1…3` / `VolcanoErupt1…3`. The lava blobs and their count/timing come from `lavaRain:::` parameters (sequences.md §5). `onLavaDie:` removes, forgets and shuts the blob down. The protocol's `onEruptionComplete:` is sent by the `lavaRain:::` handler.

#### `LavaLogic`

`SpriteLogic` · `LavaLogic.c` · 3 methods + 2 item handlers · 5 047 B

- **Role**: one falling lava blob of a volcano eruption.
- **Where**: main island.
- **Created**:
  - allocated in `VolcanoLogic::lavaRain:::` @ 0x1000c7130
  - by `VolcanoLogic::lavaRain:::` (listener = the volcano).
- **Listener**:
  - `<LavaLogicListener>` (`mLavaLogicListener`): `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onLavaDie:`
  - set in `VolcanoLogic::lavaRain:::` @ 0x1000c7130
- **Talks to**: `PygmyLogic` (`hitByLava:`).
- **Data ids**:
  - behaviours: `LavaFall`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `fall:::` @ 0x10000d288 (134 uses), `die:::` @ 0x10000d5f0 (119 uses).
- **Persistence**: none.
- **Behaviour**: init enables gravity and plays `LavaFall`; the `fall:::` handler hits pygmies within 64 units (`hitByLava:`) and stops at `y = 76` over the island; `die:::` → `onLavaDie:`.

#### `OutHouseLogic`

`SpriteLogic` · `OutHouseLogic.c` · 22 methods + 1 item handler · 8 088 B

- **Role**: the outhouse of the rock island: one pygmy uses it while another may wait; knocking three times hurries the user out.
- **Where**: rock island (location 1), `RockIslandDisplayGroup/Outhouse`.
- **Created**:
  - `setLogic` item(s) `logicId=OutHouseLogic` in `PocketGod`: `RockIslandDisplayGroup/Outhouse`
  - stored in `mOutHouseLogic`, listener = view controller (protocol empty); init stops its game frame, sets the apocalypse cloud style `OutHouse`, plays `OutHouseInit`.
- **Listener**:
  - `<OutHouseLogicListener>` (`mOutHouseLogicListener`): no methods
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic(Island)` (`isCrappingAvailable:`, `isWaitingAvailable:`, `takeCrap:`, `waitToCrap:`, `stopWaitingToCrap:`, `stopCrapping:`; outhouse sends `finishCrapping`, `tossFromOutHouse`).
- **Called from**: `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `isCrappingAvailable:`, `isWaitingAvailable:`, `takeCrap:`, `waitToCrap:` ← PygmyLogic(Island); `stopCrapping:` ← PygmyLogic, PygmyLogic(Story); `stopWaitingToCrap:` ← PygmyLogic, PygmyLogic(Island).
- **Data ids**:
  - behaviours: `OutHouseEnterIsland`, `OutHouseExitIsland`, `OutHouseFinishCrapping`, `OutHouseInit`, `OutHouseKnock`, `OutHouseTakeCrap`
  - other strings: `Default`, `OutHouse`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `finishCrapping:::` @ 0x10002304c (1 use).
- **Persistence**: none.
- **Behaviour**: overrides `setBehavior:` @ 0x100022c2c only to log. `takeCrap:` binds the pygmy on channel 1 → `OutHouseTakeCrap`; `stopCrapping:` → `OutHouseEnterIsland`. Door knocking as in `IglooLogic` (taps < 1 s apart; the third → `OutHouseFinishCrapping`; `finishCrapping:::` releases the pygmy). `exitIsland:` tosses both pygmies out.

#### `OctoStatueLogic`

`SpriteLogic` · `OctoStatueLogic.c` · 15 methods + 2 item handlers · 6 236 B

- **Role**: the octopus statue of the sand island: tapped three times (each within 3 s) it activates and blows up every pygmy ("explode from loving Pocket God").
- **Where**: sand island (location 0), `SandIslandDisplayGroup/OctoStatue`.
- **Created**:
  - `setLogic` item(s) `logicId=OctoStatueLogic` in `PocketGod`: `SandIslandDisplayGroup/OctoStatue`
  - stored in `mOctoStatueLogic`, listener = view controller.
- **Listener**:
  - `<OctoStatueLogicListener>` (`mOctoStatueLogicListener`): `isStoryActive`, `pygmyLogicArray:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic` (`explodeFromLovingPocketGod` via `blowUpPygmies:::`).
- **Called from**: `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `OctoStatueActivate`, `OctoStatueInit`, `OctoStatueTouch`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `onActivateComplete:::` @ 0x100118c40 (1 use), `blowUpPygmies:::` @ 0x100118c54 (1 use).
- **Persistence**: none.
- **Behaviour**: `onGameFrame:` resets the tap count when 3 s pass without a tap; `checkForStatueTapping:` @ 0x100118d7c: taps 1–2 → `OctoStatueTouch`, tap 3 → active, `OctoStatueActivate` (its items `blowUpPygmies` and `onActivateComplete`). Taps are ignored during stories. Game frame runs only while on the island.

#### `AntHillLogic`

`SpriteLogic` · `AntHillLogic.c` · 22 methods + 3 item handlers · 24 308 B

- **Role**: the ant hill: tapping angers it; when frenzied, dragging away from it sends a column of ants that swarm a pygmy and carry off its bones.
- **Where**: main island (`IslandDisplayGroup/AntHill`, inserted after `AntHillMarker`).
- **Created**:
  - `setLogic` item(s) `logicId=AntHillLogic` in `PocketGod`: `IslandDisplayGroup/AntHill`
  - stored in `mAntHillLogic`, listener = view controller.
- **Listener**:
  - `<AntHillLogicListener>` (`mAntHillLogicListener`): `isStoryActive`, `isGravityEnabled`, `isAntHillEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onAntHillDirect:`, `onAntHillDirectComplete:` (never sent by this class: `isAntHillEnabled`, `isGravityEnabled`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `AntLogic` (5 ants created by `directAnts:::`, listener = the hill; `setStartXPos:endXPos:`, `lookForPygmy`, `setLeadAntLogic`, `swarmPygmy:`, `die`); `PygmyLogic` (`canBeSwarmedByAntHill:`, `swarmedByAntHill:`); `MagnifyingGlassLogic` reads `antLogicArray`.
- **Called from**: `antLogicArray` ← MagnifyingGlassLogic; `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `AntHillDie`, `AntHillDirect`, `AntHillEnterIsland`, `AntHillExitIsland`, `AntHillFenzied`, `AntHillIdle`, `AntHillLight`, `AntHillNoActivity`
  - models: `Ant`
  - markers: `AntHillMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `removeAnts:::` @ 0x10013e394 (0 uses), `directAnts:::` @ 0x10013e3b8 (1 use), `die:::` @ 0x10013e7f0 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Stress: a tap adds 0.7, decays 0.01 per frame while no ants are out; state 0 (≤ 0.5, `AntHillNoActivity`), 1 (`AntHillLight`), 2 (> 1.5, `AntHillFenzied` [sic]) (`updateStressLevel` @ 0x10013e258).
  - In state 2 a touch that began on the hill and moves out of it starts directing: listener `onAntHillDirect:`, `AntHillDirect` (its `directAnts:::` spawns the ants and spreads them between the hill and the finger). Releasing sends `lookForPygmy` to every ant, `onAntHillDirectComplete:`, `AntHillIdle`.
  - `checkPygmyCollision:` @ 0x10013df78: an ant within 10 px (x) of a pygmy that `canBeSwarmedByAntHill:` becomes the lead ant and the hill swarms that pygmy (pygmy `swarmedByAntHill:`, every ant `swarmPygmy:`).
  - `onAntDie:` removes and shuts the ant down; when none remain the hill is calm again. `exitIsland`/`exitIslandNow` kill the ants (`AntHillExitIsland` / `AntHillDie`).

#### `AntLogic`

`SpriteLogic` · `AntLogic.c` · 11 methods + 5 item handlers · 10 014 B

- **Role**: one ant of the ant-hill swarm.
- **Where**: main island, next to the ant hill.
- **Created**:
  - allocated in `AntHillLogic::directAnts:::` @ 0x10013e3b8
  - 5 per `directAnts:::` (`Ant` multimodels inserted right after the hill), listener = the `AntHillLogic`.
- **Listener**:
  - `<AntLogicListener>` (`mAntLogicListener`): `onAntDie:`
  - set in `AntHillLogic::directAnts:::` @ 0x10013e3b8
- **Called from**: `burntByMagnifyingGlass:`, `canBeBurntByMagnifyingGlass:` ← MagnifyingGlassLogic; `die`, `lookForPygmy`, `setLeadAntLogic`, `setStartXPos:endXPos:`, `swarmPygmy:` ← AntHillLogic.
- **Data ids**:
  - behaviours: `AntBurn`, `AntCarryBone`, `AntDie`, `AntLineUp`, `AntLookForPygmy`, `AntSwarm`
  - models: `Bone`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `carryBone:::` @ 0x1001a6e40 (1 use), `swarm:::` @ 0x1001a6f60 (1 use), `lookForPygmy:::` @ 0x1001a7198 (1 use), `moveToTarget:::` @ 0x1001a732c (1 use), `die:::` @ 0x1001a73fc (119 uses).
- **Persistence**: none.
- **Behaviour**: walk speed `(rand%201 + 400)/10`, initial behaviour `AntLineUp`; movement is in the handlers (`moveToTarget`, `lookForPygmy`, `swarm`, `carryBone`; sequences.md §5). `onGameFrame:` @ 0x1001a6d48 integrates with zero force. `swarmPygmy:` remembers the pygmy x and plays `AntSwarm`; the lead ant spawns the bone model and plays `AntCarryBone`. `canBeBurntByMagnifyingGlass:` = not the lead ant; `burntByMagnifyingGlass:` stops it and plays `AntBurn`. `die` plays `AntDie`.

#### `MagnifyingGlassLogic`

`SpriteLogic` · `MagnifyingGlassLogic.c` · 21 methods + 4 item handlers · 30 980 B

- **Role**: the magnifying glass: held over the island on a sunny day it focuses a sun beam that burns pygmies and ants. Owns **`MagnifyingBeamModel`** (drawing: `docs/re/rendering.md` §8.7).
- **Where**: main island (`IslandDisplayGroup/MagnifyingGlass`, after `IslandBackDisplayMarker`); available with the ant hill.
- **Created**:
  - `setLogic` item(s) `logicId=MagnifyingGlassLogic` in `PocketGod`: `IslandDisplayGroup/MagnifyingGlass`
  - stored in `mMagnifyingGlassLogic`, listener = view controller. The `drag:::` handler creates the `MagnifyingHotSpot` `BCModel` (texture `HotSpot_01`) and the `MagnifyingBeam` `MagnifyingBeamModel` after `DragDisplayMarker` on the first drag.
- **Listener**:
  - `<MagnifyingGlassLogicListener>` (`mMagnifyingGlassLogicListener`): `isStoryActive`, `isGravityEnabled`, `isAntHillEnabled`, `isDayActive`, `isStormActive`, `islandMinXPos`, `islandMaxXPos`, `antHillLogic:`, `pygmyLogicArray:`, `onMagnifyingGlassDrag:`, `onMagnifyingGlassDragComplete:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic` (`canBeBurntByMagnifyingGlass:`, `burntByMagnifyingGlass:`, `unburntByMagnifyingGlass:`; pygmies call `onPygmyDie:`), `AntLogic` (`canBeBurntByMagnifyingGlass:`, `burntByMagnifyingGlass:`), `AntHillLogic` (`antLogicArray` via the listener's `antHillLogic:`); `MagnifyingBeamModel` (`setTopWidth:`, `setBottomWidth:`, `setFocusHeight:`).
- **Called from**: `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `onPygmyDie:` ← PygmyLogic.
- **Data ids**:
  - behaviours: `MagnifyingGlassBounce`, `MagnifyingGlassDie`, `MagnifyingGlassDrag`, `MagnifyingGlassEnterIsland`, `MagnifyingGlassFall`, `MagnifyingGlassFallIgnoreIsland`, `MagnifyingGlassSplash`, `MagnifyingGlassStopSlide`, `MagnifyingGlassTiltSlide`
  - models: `MagnifyingBeam`, `MagnifyingHotSpot`
  - markers: `DragDisplayMarker`, `IslandBackDisplayMarker`
  - textures: `HotSpot_01`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `slide:::` @ 0x1000d389c (20 uses), `drag:::` @ 0x1000d3984 (40 uses), `fall:::` @ 0x1000d3c68 (134 uses), `die:::` @ 0x1000d3db4 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Shared pattern of §2.2 with slide friction 0.025, floor `y = 126`, bounce ×0.4/−0.4, `MagnifyingGlassBounce`/`Die`; accelerometer only while selectable. Spring drag k 50, damping 6.
  - Beam (`onGameFrame:` @ 0x1000d2eb0, once created): `h = (y − 150)/60` when `150 ≤ y ≤ 210`, it is day and no storm, else 0; beam alpha `a = 1 − 2·|h − 0.5|`; hot-spot alpha `max(0, 2·(a − 0.5))`; beam top width 74, bottom width 51.8, focus height `h·120 + 30`, at the glass position; hot spot at `(x, 90)`.
  - `checkPygmyCollision:hotSpotPercent:` @ 0x1000d2b2c: with a hot spot (> 0) and nobody burning, burns the first pygmy (then every ant) that agrees within 10 px of x; a burning pygmy is released (`unburntByMagnifyingGlass:`) when the hot spot fades or it moves more than 10 px away.

#### `MagnifyingBeamModel`

`BCDisplayObject` · `MagnifyingBeamModel.c` · 9 methods · 5 521 B

- **Role**: custom display object drawing the magnifying glass's light cone (`setTopWidth:`, `setBottomWidth:`, `setFocusHeight:`; `docs/re/rendering.md` §8.7). Owned by **`MagnifyingGlassLogic`**.
- **Created**:
  - allocated in `MagnifyingGlassLogic::drag:::` @ 0x1000d3984
  - `MagnifyingGlassLogic::drag:::` on the first drag (`initWithId:@"MagnifyingBeam"`, after `DragDisplayMarker`).
- **Listener**: none.
- **Called from**: `calcModelVertices` ← BCRect; `setBottomWidth:`, `setFocusHeight:`, `setTopWidth:` ← MagnifyingGlassLogic.
- **Persistence**: none.

### 4.3 Sea creatures and fishing

Creatures of the surface sea and the fishing pole. Fish, swordfish, shark and bird can be dragged and thrown with the section 2.2 rules. Fish and birds can be cooked over the bonfire or by lightning and then eaten.

#### `FishLogic`

`SpriteLogic` · `FishLogic.c` · 44 methods + 6 item handlers · 46 662 B

- **Role**: the fish: swims in the sea, is caught with the fishing pole (or through the ice hole), cooked over the bonfire, eaten by pygmies, stolen by birds, stored in the igloo.
- **Where**: surface islands (`IslandDisplayGroup/Fish`), water layer; ice island for ice fishing.
- **Created**:
  - `setLogic` item(s) `logicId=FishLogic` in `PocketGod`: `IslandDisplayGroup/Fish`
  - stored in `mFishLogic`, listener = view controller.
- **Listener**:
  - `<FishLogicListener>` (`mFishLogicListener`): `isStoryActive`, `isOceanEnabled`, `isBonFireEnabled`, `isGravityEnabled`, `isFishEnabled`, `isIglooEnabled`, `isIceHoleEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `iceHoleLogic:`, `iglooLogic:`, `bonFireLogic:`, `fishingBobLogic:`, `pygmyLogicArray:`, `onFishDrag:`, `onFishDragComplete:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `FishingBobLogic` (`hookFish:`; the bob calls `catch`, `unhook`), `IceHoleLogic` (`catch`, fall checks), `IglooLogic` (`isFishFallAvailable:` via the listener's `iglooLogic:`; the igloo calls `tossFromIgloo`, `finishEatingInIgloo`), `BonFireLogic` (`isOnFire`, via `bonFireLogic:`), `BirdLogic` (`canBeCaughtByBird:`, `caughtByBird:`, `caughtByBirdComplete:`, `droppedByBird:`, `eatingPygmyLogic`), `PygmyLogic` (`hitByFish:`; pygmies call `startEating:`, `pickUpForEating`, `dropWhileEating`, `finishEating`, `isEdible`, `isCooked`, `isBurnt`; Charlie calls `thrownByCharlie:`).
  - also sent in code: `FishingBobLogic` (`isUnhooked`); `IceHoleLogic` (`fishFallIntoIceHole:`, `isAvailableForFalling`); `IglooLogic` (`fishFallIntoIgloo:`).
- **Called from**: `canBeCaughtByBird:`, `caughtByBird:`, `caughtByBirdComplete:`, `droppedByBird:`, `eatingPygmyLogic`, `isBeingEaten` ← BirdLogic; `catch`, `unhook` ← FishingBobLogic; `dropWhileEating`, `finishEating`, `isBurnt`, `isCooked`, `isEdible`, `pickUpForEating`, `startEating:` ← PygmyLogic(Island); `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `finishEatingInIgloo`, `tossFromIgloo` ← IglooLogic; `thrownByCharlie:` ← PygmyLogic(Charlie).
- **Data ids**:
  - behaviours: `FishBounce`, `FishBurn`, `FishBurntBounce`, `FishBurntDrag`, `FishBurntDropWhileEating`, `FishBurntOnGround`, `FishCatch`, `FishCook`, `FishCookedBounce`, `FishCookedDrag`, `FishCookedDropWhileEating`, `FishCookedEaten`, `FishCookedOnGround`, `FishDie`, `FishDrag`, `FishDropWhileEating`, `FishDroppedByHurricane`, `FishEaten`, `FishFall`, `FishFallIntoIgloo`, `FishGetHooked`, `FishIdle`, `FishOnGround`, `FishOnGroundBurn`, `FishOnGroundCook`, `FishPickedUpByHurricane`, `FishSplash`, `FishStopSlide`, `FishTiltSlide`, `FishUnhook`
  - groups: `IslandDisplayGroup`
  - markers: `DragDisplayMarker`, `IslandDisplayMarker`, `SmallObjectsMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `slide:::` @ 0x100012404 (20 uses), `swim:::` @ 0x10001252c (16 uses), `drag:::` @ 0x1000127c8 (40 uses), `fall:::` @ 0x100012d18 (134 uses), `fallFromHurricane:::` @ 0x100012ed4 (9 uses), `die:::` @ 0x100013040 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Shared pattern of §2.2 with slide friction 0.04, floor `y = 76`; bounce/land behaviours depend on the cook state 0/1/2: `FishBounce`/`FishCookedBounce`/`FishBurntBounce`, `FishOnGround`/`FishCookedOnGround`/`FishBurntOnGround`. `enterIsland:` puts it in the water at `y = 20`, `FishIdle` (the `swim:::` handler moves it).
  - While hooked, `x` follows the bob. `catch` @ 0x10001115c: out of the water, `vel = (±200 toward the island centre (DAT_10029d378), 1000)`, island layer, `FishCatch`.
  - Cooking (`onGameFrame:` @ 0x100011a20): while dragged or on the island and the bonfire burns, being within 20 px of its x and 30–130 px above it (or on the ground) accumulates `mCookTime`; > 2 s → cooked (`FishCook` / `FishOnGroundCook`), > 4 s → burnt (`FishBurn` / `FishOnGroundBurn`).
  - `checkPygmyCollision`: at speed > 200, hits the first pygmy within 20 px of `(px, py + 31)` that accepts `hitByFish:`.
  - `checkForIglooDuringFall` / `checkForIceHoleDuringFall`: a raw falling fish dropped onto the igloo door (within 30 px of `(igloo.x, igloo.y + 48)`) or into the ice hole is taken in.

#### `FishingPoleLogic`

`SpriteLogic` · `FishingPoleLogic.c` · 38 methods + 4 item handlers · 36 908 B

- **Role**: the fishing pole a pygmy picks up to fish (sea or ice hole); creates the line and the bob.
- **Where**: main island (`IslandDisplayGroup/FishingPole`); ice island via the ice hole.
- **Created**:
  - `setLogic` item(s) `logicId=FishingPoleLogic` in `PocketGod`: `IslandDisplayGroup/FishingPole`
  - stored in `mFishingPoleLogic`, listener = view controller. Init sets the apocalypse cloud style `FishingPole`.
- **Listener**:
  - `<FishingPoleLogicListener>` (`mFishingPoleLogicListener`): `isStoryActive`, `isBonFireEnabled`, `isGravityEnabled`, `isFishEnabled`, `islandMinXPos`, `islandMaxXPos`, `bonFireLogic:`, `pygmyLogicArray:`, `onFishingPoleDrag:`, `onFishingPoleDragComplete:`, `onFishingPoleCreateBob:fishingBobLogic:`, `onFishingPoleDestroyBob:fishingBobLogic:` (never sent by this class: `pygmyLogicArray:`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `FishingBobLogic` (created by `startFishing:` on a new `FishingBob` multimodel with box (−8,−8)–(24,24) two slots after the pole, plus a texture-less `FishingString` `BCModel`; listener `onFishingPoleCreateBob:fishingBobLogic:` / `onFishingPoleDestroyBob:fishingBobLogic:`), `BonFireLogic` (`checkProximityToBonFire:` burns the pole when it lies within [−70, +k] px of a burning fire, `DAT_10029dbe0`), `PygmyLogic` (pygmies call `isAvailable`, `pickupPole`, `startFishing:`, `startIceFishing:`, `startFishingIdle`, `dropWhileFishing`, `fishingBobLogic`, `fishingStringDisplayObject`; the pole sends `stopFishingFromPole:`; the bob, not the pole, sends `catchFish` / `catchShark` to both).
  - also sent in code: `BonFireLogic` (`hasAtLeastOneLog`).
- **Called from**: `catchFish`, `catchShark`, `pygmyLogic` ← FishingBobLogic; `checkProximityToBonFire:` ← BonFireLogic; `die` ← PygmyLogic; `dropWhileFishing`, `fishingBobLogic`, `fishingStringDisplayObject`, `isAvailable`, `pickupPole`, `startFishing:`, `startFishingIdle`, `startIceFishing:` ← PygmyLogic(Island); `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `FishingPoleBounce`, `FishingPoleDie`, `FishingPoleDrag`, `FishingPoleDroppedByHurricane`, `FishingPoleEnterIsland`, `FishingPoleFall`, `FishingPoleFallIgnoreIsland`, `FishingPolePickedUpByHurricane`, `FishingPoleSplash`, `FishingPoleStartFishingIdle`, `PygmyFishingFaceOtherway`
  - models: `FishingBob`, `FishingPole`, `FishingString`
  - groups: `IslandDisplayGroup`
  - markers: `IslandDisplayMarker`, `SmallObjectsMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `drag:::` @ 0x1001a31cc (40 uses), `fall:::` @ 0x1001a337c (134 uses), `fallFromHurricane:::` @ 0x1001a3650 (9 uses), `die:::` @ 0x1001a3794 (119 uses).
- **Persistence**: none.
- **Behaviour**: Shared pattern of §2.2 with slide friction 0.04, floor `y = 72`; `enterIsland:` drops it from (200, 400) (or straight to 72), shows the apocalypse cloud. The `fall:::` handler eases `rot.z` toward −12.62. While a pygmy fishes, `onGameFrame:` @ 0x1001a29ec keeps the line model stretched from the pole tip to the bob (angle/length; constants 180/π) and the bob's facing (`PygmyFishingFaceOtherway`).

#### `FishingBobLogic`

`SpriteLogic` · `FishingBobLogic.c` · 22 methods + 2 item handlers · 14 580 B

- **Role**: the bob at the end of the fishing line; the player drags it along the water to hook the fish or the shark.
- **Where**: surface sea while a pygmy fishes.
- **Created**:
  - allocated in `FishingPoleLogic::startFishing:` @ 0x1001a25d4
  - by `FishingPoleLogic::startFishing:`; its listener is set by the view controller in `onFishingPoleCreateBob:fishingBobLogic:`.
- **Listener**:
  - `<FishingBobLogicListener>` (`mFishingBobLogicListener`): `isGravityEnabled`, `fishingPoleLogic:`, `onFishingBobDrag:`, `onFishingBobDragComplete:`
  - set in `PocketGodViewController(Game)::onFishingPoleCreateBob:fishingBobLogic:` @ 0x100165240
- **Talks to**: `FishingPoleLogic` (via `fishingPoleLogic:`: `pygmyLogic`, `catchFish`, `catchShark`), `FishLogic` (`hookFish:` from the fish, then `catch`/`unhook`), `SharkLogic` (`hookShark:` from the shark), `PygmyLogic` (`pullPole`, `catchShark`, `catchFish`).
- **Called from**: `dropWhileFishing`, `startFishingIdle` ← FishingPoleLogic, PygmyLogic(Island); `hookFish:` ← FishLogic; `hookShark:` ← SharkLogic; `initBobTexture` ← PygmyLogic(Island); `isUnhooked` ← FishLogic, SharkLogic.
- **Data ids**:
  - behaviours: `FishingBobCatchFish`, `FishingBobCatchShark`, `FishingBobDrag`, `FishingBobIdle`, `FishingBobInit`, `FishingBobNoDrag`, `FishingBobNothing`, `FishingBobPulling`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `moveTowardPygmy:::` @ 0x1000d675c (1 use), `drag:::` @ 0x1000d6934 (40 uses).
- **Persistence**: none.
- **Behaviour**: `initBobTexture` → `FishingBobInit`; `startFishingIdle` → `FishingBobIdle`; `isUnhooked` = idle ∧ nothing hooked ∧ not dragged. Touch drags the bob horizontally (`FishingBobDrag`; the `drag:::` handler follows the finger with `dragXVel = Δx·20`, released with that speed, `FishingBobNoDrag`). `hookFish:` → `FishingBobPulling` + pygmy `pullPole`; `hookShark:` succeeds only when shark and pygmy face the same way (`xScale` product ≥ 0) → `FishingBobCatchShark`. `moveTowardPygmy:::` reels the catch in.

#### `SwordFishLogic`

`SpriteLogic` · `SwordFishLogic.c` · 33 methods + 5 item handlers · 32 870 B

- **Role**: the swordfish, a larger catchable/edible fish.
- **Where**: surface islands (`IslandDisplayGroup/SwordFish`); also caught through the ice hole.
- **Created**:
  - `setLogic` item(s) `logicId=SwordFishLogic` in `PocketGod`: `IslandDisplayGroup/SwordFish`
  - stored in `mSwordFishLogic`, listener = view controller.
- **Listener**:
  - `<SwordFishLogicListener>` (`mSwordFishLogicListener`): `isStoryActive`, `isOceanEnabled`, `isGravityEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyLogicArray:`, `onSwordFishDrag:`, `onSwordFishDragComplete:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `IceHoleLogic` (the hole fetches it with the listener's `swordFishLogic:` and calls `canBeCaught`, `catch`), `PygmyLogic` (`hitBySwordFish:` from `checkPygmyCollision`; pygmies eat it through `isEdible`, `startEating:`, `pickUpForEating`, `finishEating`, `dropWhileEating`).
- **Called from**: `canBeCaught`, `catch` ← IceHoleLogic; `dropWhileEating`, `finishEating`, `isEdible`, `pickUpForEating`, `startEating:` ← PygmyLogic(Island); `exitIsland:` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `SwordFishBounce`, `SwordFishCatch`, `SwordFishDie`, `SwordFishDrag`, `SwordFishDropWhileEating`, `SwordFishDroppedByHurricane`, `SwordFishEaten`, `SwordFishFall`, `SwordFishFallIgnoreIsland`, `SwordFishIdle`, `SwordFishOnGround`, `SwordFishPickedUpByHurricane`, `SwordFishSplash`, `SwordFishStopSlide`, `SwordFishTiltSlide`
  - markers: `DragDisplayMarker`, `IslandDisplayMarker`, `SmallObjectsMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `slide:::` @ 0x1000c26c0 (20 uses), `drag:::` @ 0x1000c27e8 (40 uses), `fall:::` @ 0x1000c29b0 (134 uses), `fallFromHurricane:::` @ 0x1000c2b2c (9 uses), `die:::` @ 0x1000c2c98 (119 uses).
- **Persistence**: none.
- **Behaviour**: Shared pattern of §2.2 with slide friction 0.04, floor `y = 86`; init plays `SwordFishIdle`. `catch` @ 0x1000c1a14 flings it like the fish (`vel.y = 1000`, `vel.x = ±200`) with `xScale = −1`. Edible on the island; `die:::` returns it to the water (y 20). No cooking.

#### `SharkLogic`

`SpriteLogic` · `SharkLogic.c` · 24 methods + 11 item handlers · 34 742 B

- **Role**: the shark: circles in the sea, leaps at pygmies standing near the water and eats them; the player can drag it out, drop it head-first onto pygmies, or hook it on the fishing line.
- **Where**: surface islands (`IslandDisplayGroup/Shark`), after `DragDisplayMarker`.
- **Created**:
  - `setLogic` item(s) `logicId=SharkLogic` in `PocketGod`: `IslandDisplayGroup/Shark`
  - stored in `mSharkLogic`, listener = view controller.
- **Listener**:
  - `<SharkLogicListener>` (`mSharkLogicListener`): `isOceanEnabled`, `isGravityEnabled`, `isSharkEnabled`, `isStoryActive`, `islandMinXPos`, `islandMaxXPos`, `fishingPoleLogic:`, `fishingBobLogic:`, `pygmyLogicArray:`, `onSharkDrag:`, `onSharkDragComplete:`, `onSharkSnap:` (never sent by this class: `fishingPoleLogic:`, `isGravityEnabled`, `isOceanEnabled`, `onSharkSnap:`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic` (`eatenByShark:`, `uneatenByShark:`, `hitByShark:xVel:`, `caughtByShark:`; pygmies call `checkJumpForPygmy:`, `cantCatchPygmy:`, `letGoOfPygmy`, `swallowPygmy`; Charlie calls `launchOffCharlie`), `FishingBobLogic` (`hookShark:` from the `swim:::` handler within 16 px of an unhooked bob).
  - also sent in code: `FishingBobLogic` (`isUnhooked`); `PygmyLogic` (`timeTemplePygmyType`).
- **Called from**: `cantCatchPygmy:`, `checkJumpForPygmy:` ← PygmyLogic; `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:`, `storyModeSharkSnap` ← PocketGodViewController; `launchOffCharlie` ← PygmyLogic(Charlie); `letGoOfPygmy`, `swallowPygmy` ← PygmyLogic(Island).
- **Data ids**:
  - behaviours: `SharkBounce`, `SharkCatchPygmy`, `SharkCatchPygmyBaby`, `SharkDie`, `SharkDrag`, `SharkEatPlaguePygmy`, `SharkFall`, `SharkFallIgnoreIsland`, `SharkIdle`, `SharkJumpForPygmy`, `SharkLaunchOffCharlie`, `SharkLetGoOfPygmy`, `SharkOnPygmyHead`, `SharkSplash`, `SharkStoryModeSharkSnap`, `SharkSwallowBounce`, `SharkSwallowPygmy`
  - markers: `DragDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `resetDropOnHead:::` @ 0x100029208 (1 use), `enableJump:::` @ 0x10002921c (2 uses), `disableJump:::` @ 0x100029234 (3 uses), `setCatchPosition:::` @ 0x100029248 (1 use), `checkForPlaguePygmy:::` @ 0x100029384 (5 uses), `checkCatchPygmy:::` @ 0x1000293c0 (1 use), `drag:::` @ 0x1000296a8 (40 uses), `launchIntoAir:::` @ 0x1000298b8 (1 use), `fall:::` @ 0x100029914 (134 uses), `swim:::` @ 0x100029a50 (16 uses), `die:::` @ 0x100029bd0 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - In the water it ignores gravity; `swim:::`, `setCatchPosition:::`, `checkCatchPygmy:::`, `launchIntoAir:::` and `fall:::` drive it (sequences.md §5). `checkJumpForPygmy: p` @ 0x100029554 (while `mbWaitingToJump`): pygmy at `y ≤ 150` and 0–100 px in front of the shark (by `xScale`) → remember it, `SharkJumpForPygmy`.
  - Out of the water: the shared gravity/floor rule (§2.2) with floor `y = 80` (bounce `SharkBounce`/`SharkSwallowBounce`, out of range `SharkDie`).
  - `checkPygmyCollision:` @ 0x1000285a4 on the island (mouth point = pos + origin + (34, 18), `y ≥ 80`): a falling, nearly upright (`|rot.z| < 30`) shark dropping (`vel.y < −20`, `|vel.x| < 30`) within 20 px of `(px, py + 31)` lands on the pygmy's head: `eatenByShark:`, `SharkOnPygmyHead`, the shark is drawn after the pygmy; while dragged it hits pygmies within 30 px (`hitByShark:xVel:(dx·−3)`).
  - A tap on the shark in the water makes it release an eaten pygmy (`uneatenByShark:`) and starts a drag; a tap-and-release without dragging bounces it up (`vel.y = 200`). `storyModeSharkSnap` places it at `x = (rand%2801 + 1000)/10` and plays `SharkStoryModeSharkSnap`.

#### `BirdLogic`

`SpriteLogic` · `BirdLogic.c` · 43 methods + 10 item handlers · 47 345 B

- **Role**: the seagull: flies over the island, dives for fish (and pygmies eating fish), poops on pygmies when tapped; lightning cooks it into food that pygmies eat.
- **Where**: surface islands (`IslandDisplayGroup/Bird`), flies at `y = 220`.
- **Created**:
  - `setLogic` item(s) `logicId=BirdLogic` in `PocketGod`: `IslandDisplayGroup/Bird`
  - stored in `mBirdLogic`, listener = view controller.
- **Listener**:
  - `<BirdLogicListener>` (`mBirdLogicListener`): `isOceanEnabled`, `isBonFireEnabled`, `isBirdEnabled`, `isStoryActive`, `islandMinXPos`, `islandMaxXPos`, `bonFireLogic:`, `fishLogic:`, `pygmyLogicArray:`, `onBirdDrag:`, `onBirdDragComplete:`, `onBirdPooHitPygmy:`, `onBirdPooMissPygmy:`, `onBirdFlyOffScreen:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `FishLogic` (via the listener's `fishLogic:`: `canBeCaughtByBird:`, `caughtByBird:`, `caughtByBirdComplete:`, `droppedByBird:`, `isBeingEaten`, `eatingPygmyLogic`), `PygmyLogic` (carried on animation channel 1; the same `canBeCaughtByBird:`, `caughtByBird:`, `caughtByBirdComplete:`, `droppedByBird:` protocol; `hitByCookedBird:`; pygmies call `dropPygmyLogic:`, `isCooked`, `isBurnt`; Charlie `thrownByCharlie:`), `BirdPooLogic` (one at a time, listener = the bird), `BonFireLogic` (`isOnFire` via `bonFireLogic:`).
- **Called from**: `dropPygmyLogic:` ← PygmyLogic; `dropWhileEating`, `finishEating`, `isBurnt`, `isCooked`, `isEdible`, `pickUpForEating`, `startEating:` ← PygmyLogic(Island); `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `hitByLightning` ← PocketGodViewController(Game); `thrownByCharlie:` ← PygmyLogic(Charlie).
- **Data ids**:
  - behaviours: `BirdAbortDive`, `BirdBurn`, `BirdBurntDropWhileEating`, `BirdCatchFood`, `BirdCatchPygmy`, `BirdCook`, `BirdCookedBounce`, `BirdCookedDrag`, `BirdCookedDropWhileEating`, `BirdCookedEaten`, `BirdCookedFall`, `BirdCookedPickedUpByHurricane`, `BirdCookedSplash`, `BirdDie`, `BirdDrag`, `BirdDroppedByHurricane`, `BirdFallIgnoreIsland`, `BirdHitByLightning`, `BirdHitByLightningHurricane`, `BirdIdle`, `BirdMakeFeces`, `BirdPickedUpByHurricane`, `BirdSplash`, `BirdTurnAround`
  - models: `BirdPoo`
  - groups: `IslandDisplayGroup`
  - markers: `DragDisplayMarker`, `IslandDisplayMarker`, `LavaDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `fly:::` @ 0x100092f04 (4 uses), `diveForFish:::` @ 0x1000931fc (1 use), `carryFishOffScreen:::` @ 0x100093428 (1 use), `diveForPygmy:::` @ 0x10009357c (1 use), `carryPygmyOffScreen:::` @ 0x10009374c (2 uses), `dropAndFlyOffScreen:::` @ 0x10009389c (1 use), `die:::` @ 0x1000939a4 (119 uses), `fall:::` @ 0x100093b94 (134 uses), `fallFromHurricane:::` @ 0x100093d44 (9 uses), `drag:::` @ 0x100093ec0 (40 uses).
- **Persistence**: none.
- **Behaviour**:
  - Flight, dives and carrying are handler-driven (`fly`, `diveForFish`, `diveForPygmy`, `carryFishOffScreen`, `carryPygmyOffScreen`, `dropAndFlyOffScreen`; sequences.md §5); the carried fish uses channel 2.
  - A tap on a live bird (`touchBeganInside:allTouches:` @ 0x100094670): while diving → `BirdAbortDive`; while carrying → drop it (`dropFood`) and `BirdAbortDive`; otherwise arm a tap. Releasing the tap (`touchEndedInside:allTouches:` @ 0x100094854) poops: a `BirdPoo` multimodel at the bird (`z = −277.12`, after `LavaDisplayMarker`) with `BirdPooLogic` (`setXPosVel: xScale·150`) and `BirdMakeFeces`. Moving out of the bird with the finger down starts a drag (`startDrag:`).
  - `hitByLightning` @ 0x100094170 (bird enabled, raw): drops its load, ends a drag, falls with gravity ignored, cook state 2 (burnt), `BirdHitByLightning` / `BirdHitByLightningHurricane`. A cooked bird is edible (`isEdible` = on island ∧ not eaten ∧ cook state ≠ 0) and draggable.
  - On the ground: the shared rule (§2.2) with floor `y = 70`; over a burning bonfire it cooks like the fish (> 2 s cooked, > 4 s burnt; `BirdCook`, `BirdBurn`). `checkPygmyCollision` (cooked birds at speed > 200) hits pygmies like the fish.
  - `onBirdPooHitPygmy:` / `onBirdPooMissPygmy:` are forwarded to the listener; `onBirdPooDie:` releases the poo.

#### `BirdPooLogic`

`SpriteLogic` · `BirdPooLogic.c` · 6 methods + 2 item handlers · 11 282 B

- **Role**: the bird dropping.
- **Where**: surface islands.
- **Created**:
  - allocated in `BirdLogic::touchEndedInside:allTouches:` @ 0x100094854
  - by `BirdLogic::touchEndedInside:allTouches:` (listener = the bird).
- **Listener**:
  - `<BirdPooLogicListener>` (`mBirdPooLogicListener`): `isOceanEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onBirdPooHitPygmy:`, `onBirdPooMissPygmy:`, `onBirdPooDie:`
  - set in `BirdLogic::touchEndedInside:allTouches:` @ 0x100094854
- **Talks to**: `PygmyLogic` (`hitByBirdPoo:`).
- **Data ids**:
  - behaviours: `BirdPooDie`, `BirdPooFall`, `BirdPooOnPygmyHead`, `BirdPooSplash`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `fall:::` @ 0x100179e88 (134 uses), `die:::` @ 0x10017a028 (119 uses).
- **Persistence**: none.
- **Behaviour**: init enables gravity (`setXPosVel:` gives the horizontal speed). The `fall:::` handler checks pygmies (`checkPygmyCollision:` → listener `onBirdPooHitPygmy:` and the pygmy is hit), lands on the island (`onBirdPooMissPygmy:`) or splashes into the sea below `y = 30` (`BirdPooSplash`); `die:::` → `onBirdPooDie:`. The protocol also forwards `isOceanEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:` to the bird.

### 4.4 Underwater ruins and coral reef

The underwater ruins (state 2) and the coral reef (state 9), which are reached through the tsunami transition. `TsunamiLogic` moves the ruins props (spear statue, anchor, drain, chum bucket) in and out. The ruins host the Bait Master (anchor and piranhas), laser shark and Pain Drain mini-games. The coral reef has the jellyfish and the symbol temple.

#### `AnchorLogic`

`SpriteLogic` · `AnchorLogic.c` · 27 methods + 2 item handlers · 22 725 B

- **Role**: the anchor of the underwater ruins; dropping it to the sea floor starts the "Bait Master" piranha mini-game, during which it spawns piranhas.
- **Where**: underwater ruins (location 2), `UnderwaterRuinsDisplayGroup/Anchor`, after `TsunamiDisplayMarker`.
- **Created**:
  - `setLogic` item(s) `logicId=AnchorLogic` in `PocketGod`: `UnderwaterRuinsDisplayGroup/Anchor`
  - stored in `mAnchorLogic`, listener = view controller; `TsunamiLogic` moves it in and out (`transitionIn`/`transitionOut`, animation channel 2).
- **Listener**:
  - `<AnchorLogicListener>` (`mAnchorLogicListener`): `piranhaLogicArray:`, `pygmyLogicArray:`, `onAnchorDrag:`, `onAnchorDragComplete:`, `onNewPiranhaLogic:`, `onRemovePiranhaLogic:`, `onRemoveAllPiranhaLogics:`, `onBaitMasterStart:`, `onBaitMasterStop:`, `onPiranhaEatPygmy:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PiranhaLogic` (creates them, listener = the anchor; `stopBaitMaster`; piranhas call back `onPiranhaDie:`, `onPiranhaFight:`, `onPiranhaOffScreenAtePygmy:`, `piranhaLogicArray:`, `pygmyLogicArray:`), the `PiranhaArrow` warning arrow (a `MultiModelLogic` with `PiranhaWarningArrow`).
- **Called from**: `activate`, `stopBaitMaster` ← PocketGodViewController(Game); `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland` ← PocketGodViewController; `transitionIn`, `transitionOut` ← TsunamiLogic.
- **Data ids**:
  - behaviours: `AnchorBounce`, `AnchorDrag`, `AnchorFall`, `AnchorTransitionIn`, `AnchorTransitionOut`, `PiranhaWarningArrow`
  - models: `Piranha`, `PiranhaArrow`
  - markers: `TsunamiDisplayMarker`, `UnderwaterPiranhaDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `die:::` @ 0x100055068 (119 uses), `drag:::` @ 0x10005509c (40 uses).
- **Persistence**: none.
- **Behaviour**:
  - `activate` (entered ∧ transitioned in): creates the `PiranhaArrow` multimodel, inserts the anchor after `TsunamiDisplayMarker`, starts its game frame, `AnchorTransitionIn`. `deactivate` plays `AnchorTransitionOut`, removes arrow and piranhas, stops Bait Master (`onBaitMasterStop:`), ends a drag.
  - `onGameFrame:` @ 0x100054348: water gravity `(0, −60)`, integration incl. rotation, screen clamp; the floor is `y = 57`. Landing there from above while not held and active starts Bait Master: `mTimeBetweenPiranha = 8`, `AnchorBounce`, listener `onBaitMasterStart:`.
  - During Bait Master, whenever the countdown runs out and fewer than 5 piranhas exist: next delay = random in `[t − 1, t + 1]` (0.1 s steps), `t = max(2, t − 0.15)`; a `Piranha` multimodel (origin x −20, box (−30,−25)–(30,25)) enters from `x = −120` or `600` (`DAT_10029d608`, random side) at `y = (rand%1801 + 700)/10` after `UnderwaterPiranhaDisplayMarker`, `PiranhaLogic` (listener = anchor, `onNewPiranhaLogic:` to the view controller); the arrow is shown at x 30 (flipped) or 450 at that height (`PiranhaWarningArrow`).
  - `pickedUpByWhirlPool:` and a piranha fight stop Bait Master (`stopBaitMaster` → every piranha `stopBaitMaster`).

#### `PiranhaLogic`

`SpriteLogic` · `PiranhaLogic.c` · 22 methods + 6 item handlers · 34 223 B + 2 298 B of free helper functions

- **Role**: a piranha of Bait Master: the player draws a path from it with the finger; it follows the path and eats the pygmy the path ends on; piranhas that meet fight.
- **Where**: underwater ruins.
- **Created**:
  - allocated in `AnchorLogic::onGameFrame:` @ 0x100054348
  - by `AnchorLogic::onGameFrame:` (listener = the anchor). Owns **`PathModel`** (drawing: `docs/re/rendering.md` §8.6), created on the first touch (`PathModel initWithId:@"PiranhaPath" pathVector:mpPathVector`, `z = −277.12`, after `UnderwaterPathDisplayMarker`).
- **Listener**:
  - `<PiranhaLogicListener>` (`mPiranhaLogicListener`): `piranhaLogicArray:`, `pygmyLogicArray:`, `onPiranhaOffScreenAtePygmy:`, `onPiranhaFight:`, `onPiranhaDie:` (never sent by this class: `onPiranhaOffScreenAtePygmy:`)
  - set in `AnchorLogic::onGameFrame:` @ 0x100054348
- **Talks to**: `PygmyLogic` (`canBeEatenByPiranha:`, `stopEatenByPiranha:`; `PygmyLogic(Underwater)` calls `stopEatingPygmy:`, `PygmyLogic(Island)` `cancelFlip`), other piranhas via `piranhaLogicArray:` (`fightPiranha:`).
  - also sent in code: `PathModel` (`initWithId:pathVector:`, `setPathVector:`); `PygmyLogic` (`eatenByPiranha:`, `eatenByPiranhaComplete:`, `eatingPiranhaLogic`).
- **Called from**: `stopBaitMaster` ← AnchorLogic; `stopEatingPygmy:` ← PygmyLogic(Underwater).
- **Data ids**:
  - behaviours: `PiranhaAgitated`, `PiranhaAttack`, `PiranhaFight`, `PiranhaFlip`, `PiranhaFlipFull`, `PiranhaInit`, `PiranhaSwim`, `PiranhaSwimAway`, `PiranhaSwimFull`
  - markers: `UnderwaterPathDisplayMarker`
  - other strings: `PiranhaPath`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `initPos:::` @ 0x100009b4c (2 uses), `swim:::` @ 0x100009bf8 (16 uses), `swimAway:::` @ 0x10000a370 (1 use), `die:::` @ 0x10000a4b8 (119 uses), `attack:::` @ 0x10000a4f0 (1 use), `eatPygmyComplete:::` @ 0x10000a6ac (4 uses).
- **Persistence**: none.
- **Behaviour**:
  - The path is a `std::vector<VECTOR4>` (`mpPathVector`; the three free functions are its push-back/realloc/destructor). `touchBeganInside:allTouches:` @ 0x10000a860 (not fighting, eating or leaving): restart the path at the piranha, remember the touch offset, create the path model once. `touchMovedOutside:allTouches:` @ 0x10000aad4: at most 41 points; a point within 40 px of `(px, py + 32)` of a pygmy that `canBeEatenByPiranha:` ends the path there (`mbPathLeadsToPygmy`); otherwise a point less than 20 px from the last one replaces it, else it is appended.
  - `swim:::` follows the path at speed 50 (or bounces inside x 50–430, y 50–270); `attack:::`, `eatPygmyComplete:::`, `swimAway:::` (sequences.md §5). `onGameFrame:` @ 0x1000096bc integrates without forces.
  - `checkPiranhaCollision` @ 0x10000936c: piranhas that touch become agitated and fight (`fightPiranha:`, listener `onPiranhaFight:`, which ends Bait Master). `stopBaitMaster` releases the eaten pygmy and swims away (`PiranhaSwimAway`). `animPiranhaYFlip` flips `yScale` by the target heading (±90°).

#### `PathModel`

`BCDisplayObject` · `PathModel.c` · 8 methods · 7 355 B

- **Role**: custom display object drawing the piranha path ribbon from a `std::vector<VECTOR4>` (`setPathVector:`; `docs/re/rendering.md` §8.6). Owned by **`PiranhaLogic`**.
- **Created**:
  - allocated in `PiranhaLogic::touchBeganInside:allTouches:` @ 0x10000a860
  - `PiranhaLogic::touchBeganInside:allTouches:` (`initWithId:@"PiranhaPath" pathVector:`).
- **Listener**: none.
- **Called from**: `initWithId:pathVector:`, `setPathVector:` ← PiranhaLogic.
- **Persistence**: none.

#### `ChumBucketLogic`

`SpriteLogic` · `ChumBucketLogic.c` · 29 methods + 2 item handlers · 13 006 B

- **Role**: the chum bucket of the underwater ruins: tapping it spills a chum that summons the laser shark and shows the laser button; it owns the laser shark, the laser target and the button.
- **Where**: underwater ruins (`UnderwaterRuinsDisplayGroup/ChumBucket`, after `TsunamiDisplayMarker`); moved in and out by `TsunamiLogic` (channel 3).
- **Created**:
  - `setLogic` item(s) `logicId=ChumBucketLogic` in `PocketGod`: `UnderwaterRuinsDisplayGroup/ChumBucket`
  - stored in `mChumBucketLogic`, listener = view controller. Its own `setLogic:` @ 0x10011b7dc (reached through the boot `setLogic` items with `manager=ChumBucket`) builds `LaserButtonLogic` (`BCButtonLogic`, listener = the bucket, kept out of the display until needed), `LaserSharkLogic` and `LaserTargetLogic` (listener = the bucket) and keeps them in `mLogicArray`.
- **Listener**:
  - `<ChumBucketLogicListener>` (`mChumBucketLogicListener`): `pygmyLogicArray:`, `onLaserSharkZapPygmy:`, `onLaserSharkZapComplete:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `ChumLogic` (one at a time: a `Chum` multimodel; `onChumDie:`), `LaserSharkLogic` (`enterIsland`, `exitIsland`, `shootLaser`; it calls `chumLogic:`, `laserTargetLogic:`, `onLaserSharkZapPygmy:`, `onLaserSharkExitScreen:`), `LaserTargetLogic` (`enterIsland`, `exitIsland`), the laser `BCButtonLogic` (`onButtonPress:touch:` → `ShootLaser`, whose `shootLaser:::` fires).
  - also sent in code: `ChumLogic` (`touchBeganInside:allTouches:`).
- **Called from**: `activate` ← PocketGodViewController(Game); `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland`, `laserSharkLogic` ← PocketGodViewController; `transitionIn`, `transitionOut` ← TsunamiLogic.
- **Data ids**:
  - behaviours: `ChumBucketTransitionIn`, `ChumBucketTransitionOut`, `ShootLaser`
  - models: `Chum`
  - markers: `MenuDisplayMarker`, `TsunamiDisplayMarker`
  - event ids: `LaserButtonLogic`, `LaserSharkLogic`, `LaserTargetLogic`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `shootLaser:::` @ 0x10011b3f4 (1 use), `die:::` @ 0x10011b424 (119 uses).
- **Persistence**: none.
- **Behaviour**: `touchBeganInside:allTouches:` @ 0x10011b578: the target and the shark enter; if no chum exists, the laser button is inserted after `MenuDisplayMarker` (in `PocketGod`) and a new chum is created. `activate`/`deactivate` follow the usual enter ∧ transition-in pair (`ChumBucketTransitionIn`/`Out`); deactivation removes the button, ends the zap (`onLaserSharkZapComplete:`), removes shark, target and chum. `onLaserSharkExitScreen:` hides target and button and reports `onLaserSharkZapComplete:`.

#### `ChumLogic`

`SpriteLogic` · `ChumLogic.c` · 20 methods + 2 item handlers · 15 107 B

- **Role**: the chum (fish bait) that sinks from the bucket; pygmies can hold it and the laser shark eats it.
- **Where**: underwater ruins.
- **Created**:
  - allocated in `ChumBucketLogic::touchBeganInside:allTouches:` @ 0x10011b578
  - by `ChumBucketLogic::touchBeganInside:allTouches:` (listener = the bucket).
- **Listener**:
  - `<ChumLogicListener>` (`mChumLogicListener`): `pygmyLogicArray:`, `onChumDie:`
  - set in `ChumBucketLogic::touchBeganInside:allTouches:` @ 0x10011b578
- **Talks to**: `LaserSharkLogic` (`canBeFedToShark`, `holdingPygmyLogic`, `isPygmyHoldingChum`, `eatenByShark`), `PygmyLogic(Underwater)` (`dropChum:`; the chum calls the holder's `dropChum:`).
  - also sent in code: `PygmyLogic` (`canHoldChum:`, `holdChum:`).
- **Called from**: `canBeFedToShark`, `eatenByShark`, `holdingPygmyLogic`, `isPygmyHoldingChum` ← LaserSharkLogic; `dropChum:` ← PygmyLogic(Underwater); `enterIsland`, `exitIsland`, `touchBeganInside:allTouches:` ← ChumBucketLogic.
- **Data ids**:
  - behaviours: `ChumDrag`, `ChumEaten`, `ChumFade`, `ChumHold`, `ChumInit`
  - markers: `TsunamiDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `drag:::` @ 0x10008e8dc (40 uses), `die:::` @ 0x10008ea94 (119 uses).
- **Persistence**: none.
- **Behaviour**: init: origin (−20, −10), box (−10,−10)–(50, …), `ChumInit`. `onGameFrame:` @ 0x10008e5a4: sinking gravity `(0, −15)` (`−3000` while dragged), spring drag (k 100, damping 8 with rotation; rotation decays by half per frame when released), screen clamp; it stops on the floor `y = 30` and, when not held, fades (`ChumFade`, unselectable). `eatenByShark` → `ChumEaten`.

#### `LaserSharkLogic`

`SpriteLogic` · `LaserSharkLogic.c` · 14 methods + 4 item handlers · 15 526 B

- **Role**: the laser shark: drawn by the chum, it swims after the chum (or the pygmy holding it) and eats it; the player fires its head laser at the tilt-controlled target to zap pygmies.
- **Where**: underwater ruins (`UnderwaterRuinsDisplayGroup/LaserShark`, after `TsunamiDisplayMarker`).
- **Created**:
  - `setLogic` item(s) `logicId=LaserSharkLogic` in `PocketGod`: `UnderwaterRuinsDisplayGroup/LaserShark` (manager `ChumBucket`)
  - allocated in `ChumBucketLogic::setLogic:` @ 0x10011b7dc
  - by `ChumBucketLogic::setLogic:` (listener = the bucket). Init creates its `LaserGun` multimodel with `LaserGunLogic` (listener = the shark) bound to an animation channel.
- **Listener**:
  - `<LaserSharkLogicListener>` (`mLaserSharkLogicListener`): `chumLogic:`, `laserTargetLogic:`, `pygmyLogicArray:`, `onLaserSharkZapPygmy:`, `onLaserSharkExitScreen:`
  - set in `ChumBucketLogic::setLogic:` @ 0x10011b7dc
- **Talks to**: `LaserGunLogic` (`enable`, `disable`, `shootLaser`, `updateLaser:`; the gun calls `onShootLaserComplete:`), `ChumLogic` (`eatenByShark`), `LaserTargetLogic` (position), `PygmyLogic` (`canBeHitBySharkLaser`, `hitBySharkLaser`, `eatenByLaserShark:`, `stopEatenByLaserShark:`, `swallowedByLaserShark:`).
  - also sent in code: `ChumLogic` (`canBeFedToShark`, `holdingPygmyLogic`, `isPygmyHoldingChum`); `PygmyLogic` (`startEatenByLaserShark:`).
- **Called from**: `enterIsland`, `shootLaser` ← ChumBucketLogic; `exitIsland` ← ChumBucketLogic, PocketGodViewController; `isEnabled` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `LaserSharkEatChum`, `LaserSharkEatPygmy`, `LaserSharkEnter`, `LaserSharkExit`, `LaserSharkTurn`
  - models: `LaserGun`
  - markers: `TsunamiDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `eatPygmyComplete:::` @ 0x1000c55ec (4 uses), `initPos:::` @ 0x1000c5630 (2 uses), `swim:::` @ 0x1000c5710 (16 uses), `die:::` @ 0x1000c5b00 (119 uses).
- **Persistence**: none.
- **Behaviour**: `initPos:::` enters from x −50 or 530 at `y = 70…250`; `swim:::` steers at speed 150 toward the chum (heading clamped to ±30°). `onGameFrame:` @ 0x1000c505c: integration; on `shootLaser` it snapshots the target position and zaps the first pygmy that `canBeHitBySharkLaser` within 30 px of `(target.x, target.y − 20)`… (exactly: `|shootPos − (px, py + 20)| < 30`), `hitBySharkLaser`, listener `onLaserSharkZapPygmy:`; the gun is rotated toward the target (`atan2`) and `updateLaser:` gets the distance. `animKillPygmy`/`animHideChum`/`animStartShark` (`vel = 150·(cos, sin)(heading)`)/`animStopShark` are animation callbacks. `isEnabled` = active (asked by the view controller).

#### `LaserGunLogic`

`SpriteLogic` · `LaserGunLogic.c` · 10 methods + 1 item handler · 4 927 B

- **Role**: the laser gun on the laser shark's head: beam and blast effects.
- **Where**: underwater ruins.
- **Created**:
  - allocated in `LaserSharkLogic::initWithDisplayObject:` @ 0x1000c4d54
  - by `LaserSharkLogic::initWithDisplayObject:` (listener = the shark).
- **Listener**:
  - `<LaserGunLogicListener>` (`mLaserGunLogicListener`): `onShootLaserComplete:`
  - set in `LaserSharkLogic::initWithDisplayObject:` @ 0x1000c4d54
- **Called from**: `disable`, `enable`, `shootLaser`, `updateLaser:` ← LaserSharkLogic.
- **Data ids**:
  - behaviours: `LaserBeam`, `LaserBlast`, `LaserGun`, `LaserGunShoot`
  - models: `LaserBeam`, `LaserBlast`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `onShootLaserComplete:::` @ 0x1001739fc (1 use).
- **Persistence**: none.
- **Behaviour**: init creates the `LaserBeam` multimodel (`MultiModelLogic`, `LaserBeam`) and a blast multimodel/logic, bound to its animation channels. `shootLaser` → `LaserGunShoot`; `updateLaser: d` sets the beam `xScale = (d − 19)/100` (×0.6 while retracting) and the blast x to `d`; `animShowBeam` (blast `LaserBlast`), `animHalfBeam`, `animHideBeam` are animation callbacks; `onShootLaserComplete:::` reports to the shark. `enable`/`disable` are empty.

#### `LaserTargetLogic`

`SpriteLogic` · `LaserTargetLogic.c` · 8 methods · 4 007 B

- **Role**: the laser crosshair, steered by tilting the device.
- **Where**: underwater ruins (`UnderwaterRuinsDisplayGroup/LaserTarget`).
- **Created**:
  - `setLogic` item(s) `logicId=LaserTargetLogic` in `PocketGod`: `UnderwaterRuinsDisplayGroup/LaserTarget` (manager `ChumBucket`)
  - allocated in `ChumBucketLogic::setLogic:` @ 0x10011b7dc
  - by `ChumBucketLogic::setLogic:` (listener = the bucket; the protocol is empty).
- **Listener**:
  - `<LaserTargetLogicListener>` (`mLaserTargetLogicListener`): no methods
  - set in `ChumBucketLogic::setLogic:` @ 0x10011b7dc
- **Called from**: `enterIsland`, `exitIsland` ← ChumBucketLogic.
- **Data ids**:
  - behaviours: `LaserTargetEnter`, `LaserTargetExit`
  - markers: `TsunamiDisplayMarker`
- **Persistence**: none.
- **Behaviour**: `enterIsland`/`exitIsland` insert/remove it (after `TsunamiDisplayMarker`; `LaserTargetEnter`/`LaserTargetExit`). `updateTargetPos` @ 0x10008fc74 every frame: from `BCView.rawAccelForce` `(ax, ay)`: `ax' = min(ax, −0.5)`, `ay' = min(ay, 0.2)`; target `x = 470` if `ay' < −0.2` else `((ay' + 0.2)/−0.4)·460 + 470`; target `y = 10` if `ax' < −0.9` else `((ax' + 0.9)/0.4)·300 + 10`; the crosshair eases 6 % per frame toward it. (`updateTargetPos1`, a ping-pong at `y = 220` between x 10 and 470 at ±4 per frame, is unused.)

#### `TileDrainLogic`

`SpriteLogic` · `TileDrainLogic.c` · 29 methods + 5 item handlers · 25 094 B

- **Role**: the drain in the floor of the underwater ruins and host of the "Pain Drain" mini-game: with the plug lifted it sucks in a pygmy, who must climb rising platforms away from the spikes.
- **Where**: underwater ruins (`UnderwaterRuinsDisplayGroup/TileDrain`, after `TsunamiDisplayMarker`), mini-game in `PainDrainDisplayGroup`; moved by `TsunamiLogic` (channel 4).
- **Created**:
  - `setLogic` item(s) `logicId=TileDrainLogic` in `PocketGod`: `UnderwaterRuinsDisplayGroup/TileDrain`
  - stored in `mTileDrainLogic`, listener = view controller.
- **Listener**:
  - `<TileDrainLogicListener>` (`mTileDrainLogicListener`): `pygmyLogicArray:`, `tilePlugLogic:`, `onPainDrainStart:pygmyLogic:`, `onPainDrainStop:`, `onPainDrainScore:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `TilePlugLogic` (`plug:`, `unplug:touch:`; the plug calls `canPlugDrain`, `plugDrain:`), `PainDrainPlatformLogic` (4 `Platform` multimodels after `PainDrainPlatformDisplayMarker`, listener = the drain; `setYPosVel:`), the `PainDrainDisplayGroup` objects `Background`, `LeftChain`, `RightChain`, `SpikesTop`, `SpikesBottom` (their logics: `…Init` / `…Clear` behaviours), `PygmyLogic` (`suckedIntoDrain:`, `escapeFromDrain:`, `dieFromPainDrain`; `PygmyLogic(Underwater)` reads `painDrainPlatformLogicArray`).
  - also sent in code: `PygmyLogic` (`startPainDrain`).
- **Called from**: `activate` ← PocketGodViewController(Game); `canPlugDrain`, `plugDrain:` ← TilePlugLogic; `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `escapeFromDrain:` ← PygmyLogic; `exitIsland` ← PocketGodViewController; `painDrainPlatformLogicArray`, `pygmyFallIntoDrainComplete:`, `stopPainDrain` ← PygmyLogic(Underwater); `transitionIn`, `transitionOut` ← TsunamiLogic.
- **Data ids**:
  - behaviours: `PainDrainBackgroundClear`, `PainDrainBackgroundInit`, `PainDrainChainClear`, `PainDrainChainInit`, `PainDrainSpikesBottomClear`, `PainDrainSpikesBottomInit`, `PainDrainSpikesTopClear`, `PainDrainSpikesTopInit`, `TileDrainDropChain`, `TileDrainLiftChain`, `TileDrainPlugDrain`, `TileDrainPygmyFallIntoDrain`, `TileDrainPygmySpiked`, `TileDrainTransitionIn`, `TileDrainTransitionOut`, `TileDrainUnplug`
  - models: `Platform`
  - groups: `PainDrainDisplayGroup`
  - objects: `Background`, `LeftChain`, `RightChain`, `SpikesBottom`, `SpikesTop`
  - markers: `PainDrainPlatformDisplayMarker`, `TsunamiDisplayMarker`, `UnderwaterDisplayMarker`
  - other strings: `Default`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `tileDrainSpikePygmyComplete:::` @ 0x100117b04 (1 use), `unplugTilePlug:::` @ 0x100117f98 (1 use), `catchPygmyComplete:::` @ 0x100118018 (1 use), `catchPygmy:::` @ 0x100118184 (2 uses), `die:::` @ 0x100118400 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Touching the drain and dragging up lifts the plug (`unplugTilePlug:::` → plug `unplug:touch:`). `catchPygmy:::` sucks in a pygmy within `|dx| < 50`, `0 < dy < 130`; `pygmyFallIntoDrainComplete:` binds it on channel 1 (`TileDrainPygmyFallIntoDrain`); `catchPygmyComplete:::` starts Pain Drain: `createPlatforms`, platform speed 50, listener `onPainDrainStart:pygmyLogic:`.
  - `onGameFrame:` @ 0x1001177f8: all platforms get `vel.y = mPlatformYPosVel`; while active the speed grows by `dt` per frame up to 210, the background scrolls by `speed·dt·0.5` and the chains by `speed·dt`, and the score `(int)(background.y·0.3125)` goes to `onPainDrainScore:`.
  - `stopPainDrain` (pygmy spiked) → `TileDrainPygmySpiked`; `tileDrainSpikePygmyComplete:::` clears everything and kills the pygmy. `plugDrain:` releases a pygmy and plays `TileDrainPlugDrain`; `canPlugDrain` = not playing.

#### `TilePlugLogic`

`SpriteLogic` · `TilePlugLogic.c` · 17 methods + 3 item handlers · 14 638 B

- **Role**: the stone plug of the drain; lifted it can be dragged and dropped back into the drain.
- **Where**: underwater ruins (`UnderwaterRuinsDisplayGroup/TilePlug`).
- **Created**:
  - `setLogic` item(s) `logicId=TilePlugLogic` in `PocketGod`: `UnderwaterRuinsDisplayGroup/TilePlug`
  - stored in `mTilePlugLogic`, listener = view controller; init plays `TilePlugInit`.
- **Listener**:
  - `<TilePlugLogicListener>` (`mTilePlugLogicListener`): `pygmyLogicArray:`, `tileDrainLogic:`, `onTilePlugDrag:`, `onTilePlugDragComplete:`, `onPainDrainStart:pygmyLogic:`, `onPainDrainStop:` (never sent by this class: `onPainDrainStart:pygmyLogic:`, `onPainDrainStop:`, `pygmyLogicArray:`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `TileDrainLogic` (via `tileDrainLogic:`: `canPlugDrain`, `plugDrain:`).
- **Called from**: `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland` ← PocketGodViewController, TsunamiLogic; `plug:`, `unplug:touch:` ← TileDrainLogic.
- **Data ids**:
  - behaviours: `TilePlugDrag`, `TilePlugFall`, `TilePlugHitGround`, `TilePlugInDrain`, `TilePlugInit`, `TilePlugLifted`, `TilePlugTransitionOut`
  - markers: `UnderwaterPygmyDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `drag:::` @ 0x1001205ac (40 uses), `fall:::` @ 0x1001207c4 (134 uses), `die:::` @ 0x100120914 (119 uses).
- **Persistence**: none.
- **Behaviour**: `unplug: drain touch: t` @ 0x100120344 puts the plug 50 px above the drain, starts a spring drag with that touch, inserts it before `UnderwaterPygmyDisplayMarker`, `TilePlugLifted`. `onGameFrame:` @ 0x10011ffa0: water gravity `(0, −30)` (`−3000` while dragged), spring force, integration. Release → `TilePlugFall`; the `fall:::` handler plugs a drain within 15 px (x) / 50 px (y) that `canPlugDrain` (`TilePlugInDrain`, `plugDrain:`). `exitIsland` returns it (`TilePlugTransitionOut`).

#### `PainDrainPlatformLogic`

`SpriteLogic` · `PainDrainPlatformLogic.c` · 7 methods · 5 255 B

- **Role**: one rising platform of the Pain Drain mini-game.
- **Where**: Pain Drain (`PainDrainDisplayGroup`).
- **Created**:
  - allocated in `TileDrainLogic::createPlatforms` @ 0x100117454
  - 4 by `TileDrainLogic::createPlatforms` (listener = the drain; the protocol is empty).
- **Listener**:
  - `<PainDrainPlatformLogicListener>` (`mPainDrainPlatformLogicListener`): no methods
  - set in `TileDrainLogic::createPlatforms` @ 0x100117454
- **Talks to**: `PygmyLogic(Underwater)` calls `checkCollision:lastPos:`.
- **Called from**: `checkCollision:lastPos:` ← PygmyLogic(KonkeyDong), PygmyLogic(OogaJump), PygmyLogic(TheRuns), PygmyLogic(Underwater).
- **Data ids**:
  - behaviours: `PainDrainPlatformInit`
- **Persistence**: none.
- **Behaviour**: generation state in globals (like `DoodleCloudLogic`): `DAT_1007e7c18` last platform, `DAT_1007e7c10` its position, `DAT_1007e7c0c` gap (120), `DAT_1007e7c08` flag. `moveToBottom` @ 0x1000cdd4c places the platform at (100, 60) or below the previous one with a random x inside the allowed band. `onGameFrame:` @ 0x1000ce06c: remembers the last position, integrates, and wraps to the bottom above `y = 356`. `checkCollision:lastPos:` @ 0x1000cdf20: a falling pygmy within 40 px horizontally that crosses the platform's top since the last frame lands on it (its `vel.y = 0`, `y = platform.y`); the platform remembers its rider.

#### `TsunamiLogic`

`SpriteLogic` · `TsunamiLogic.c` · 12 methods + 5 item handlers · 17 242 B

- **Role**: the tsunami wave that carries the camera between the island and the underwater locations (front and behind layers), and the stage manager that brings the underwater ruins' objects in and out.
- **Where**: island → underwater transition (`UnderwaterDisplayGroup/FrontTsunami`, `IslandDisplayGroup/BehindTsunami`).
- **Created**:
  - `setLogic` alias `logicId=BehindTsunamiLogic`: `IslandDisplayGroup/BehindTsunami`
  - `setLogic` alias `logicId=FrontTsunamiLogic`: `UnderwaterDisplayGroup/FrontTsunami`
  - `setLogic` aliases `FrontTsunamiLogic` / `BehindTsunamiLogic` → `mFrontTsunamiLogic` / `mBehindTsunamiLogic` with `setFrontLayer:` YES / NO, listener = view controller (`setTsunamiLogicListener:` in `setLogic:::`). Init stops its game frame and removes it.
- **Listener**:
  - `<TsunamiLogicListener>` (`mTsunamiLogicListener`): `islandLocationState`, `lastIslandLocationState`, `chumBucketLogic:`, `anchorLogic:`, `tileDrainLogic:`, `tilePlugLogic:`, `spearStatueLogic:`, `underwaterSceneryLogicArray:`, `onTsunamiActivateComplete:`, `onTsunamiDeactivateComplete:`, `onTsunamiHideIslandAssets:`, `onTsunamiShowIslandAssets:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `SpearStatueLogic`, `AnchorLogic`, `TileDrainLogic`, `ChumBucketLogic` (bound to channels 1, 2, 4, 3 and sent `transitionIn`/`transitionOut`), `TilePlugLogic` (`exitIsland`), `UnderwaterSceneryLogic`s, `UnderwaterTempleLogic` (entry 547 `activate`/`deactivate`), entry 548 (`UnderwaterCoralReefForeground`); all fetched through the listener (`spearStatueLogic:`, `anchorLogic:`, `tileDrainLogic:`, `tilePlugLogic:`, `chumBucketLogic:`, `underwaterSceneryLogicArray:`).
  - also sent in code: `SpearStatueLogic` (`hideSpear`, `showSpear`).
  - `getLogic:` 547 `UnderwaterTempleLogic`, 548 `MultiModelLogic`.
- **Called from**: `activate:`, `deactivate`, `moveBetweenUnderwaterLocations` ← PocketGodViewController; `setFrontLayer:` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `BehindTsunamiStartLeft`, `BehindTsunamiStartNoWater`, `BehindTsunamiStartRight`, `FrontTsunamiChangeUnderwaterLocation`, `FrontTsunamiEnd`, `FrontTsunamiEndNoWater`, `FrontTsunamiStartLeft`, `FrontTsunamiStartNoWater`, `FrontTsunamiStartRight`
  - markers: `CloudBehindDisplayMarker`, `TsunamiDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `activateUnderwaterTemple:::` @ 0x10000cad0 (4 uses), `tsunamiActivateComplete:::` @ 0x10000cb44 (3 uses), `tsunamiDeactivateComplete:::` @ 0x10000cbac (2 uses), `setAnimationBasedOnUnderwaterLocation:::` @ 0x10000cc3c (8 uses), `setAnimationBasedOnLastUnderwaterLocation:::` @ 0x10000ccc8 (1 use).
- **Persistence**: none.
- **Behaviour**:
  - `setFrontLayer: YES` binds entry 548's display object to channel 5 and entry 547's to channel 6.
  - `activate: right` @ 0x10000bf18: front layer inserted before `TsunamiDisplayMarker`, behind layer after `CloudBehindDisplayMarker`; when the previous location (`lastIslandLocationState`) is 5 or 11–17 (Ape island and the apocalypse temples, which have no sea) it plays `…StartNoWater` (front also sends `onTsunamiHideIslandAssets:`), else `…StartLeft` / `…StartRight`.
  - `deactivate` (front): `FrontTsunamiEnd` or `FrontTsunamiEndNoWater`, hides the spear, transitions out anchor, drain and bucket, plug `exitIsland`, temple `deactivate`.
  - `moveBetweenUnderwaterLocations` (between ruins 2 and coral reef 9): at location 2 rebinds and brings in statue (+spear), anchor, drain and bucket and deactivates the temple; otherwise transitions the statue out; then `FrontTsunamiChangeUnderwaterLocation`. `animShowScenery` / `animHideScenery` / `animShowIslandAssets` / `animHideIslandAssets` are animation callbacks. Item handlers pick animations by location (sequences.md §5).

#### `UnderwaterSceneryLogic`

`SpriteLogic` · `UnderwaterSceneryLogic.c` · 6 methods + 1 item handler · 7 493 B

- **Role**: drifting background scenery of the underwater locations: the school of fish (type 0) and two blobs (types 1, 2).
- **Where**: underwater (`UnderwaterDisplayGroup/SchoolOfFish` and the blob objects).
- **Created**:
  - `setLogic` alias `logicId=SchoolOfFishLogic`: `UnderwaterDisplayGroup/SchoolOfFish`
  - `setLogic` alias `logicId=UnderwaterBlob1Logic`: `UnderwaterDisplayGroup/UnderwaterBlob1`
  - `setLogic` alias `logicId=UnderwaterBlob2Logic`: `UnderwaterDisplayGroup/UnderwaterBlob2`
  - by the `setLogic` aliases `SchoolOfFishLogic` (type 0), `UnderwaterBlob1Logic` (1), `UnderwaterBlob2Logic` (2) with `initWithDisplayObject:underwaterSceneryType:` @ 0x10017c228; kept in `mUnderwaterSceneryLogicArray` (no listener).
- **Listener**: none.
- **Talks to**: `TsunamiLogic` sends `transitionIn` / `transitionOut` (through the listener's `underwaterSceneryLogicArray:`).
- **Called from**: `initWithDisplayObject:underwaterSceneryType:` ← PocketGodViewController(Game); `transitionIn`, `transitionOut` ← TsunamiLogic.
- **Data ids**:
  - behaviours: `BlobEnter`, `BlobExit`, `SchoolOfFishEnter`, `SchoolOfFishExit`
  - markers: `TsunamiDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `drift:::` @ 0x10017c6ec (3 uses).
- **Persistence**: none.
- **Behaviour**: `refreshParams` @ 0x10017c3c0: `vel.x = (rand%301 − 600)/10` (type 0) or `(rand%201 − 400)/10` (blobs). `onGameFrame:` @ 0x10017c304: `alpha += (target − alpha)·0.5·dt`, then integration. `transitionIn` fades in from 0 at a random x inside the screen (by its collision box and `xScale`); `transitionOut` sets alpha 0 and plays `SchoolOfFishExit` / `BlobExit`. The `drift:::` handler wraps it around the 480-px screen.

#### `SpearStatueLogic`

`SpriteLogic` · `SpearStatueLogic.c` · 19 methods · 6 562 B

- **Role**: the statue in the underwater ruins holding the chain spear; tapping it fires the spear, skewering pygmies.
- **Where**: underwater ruins (`UnderwaterRuinsDisplayGroup/SpearStatue`, after `TsunamiDisplayMarker`); moved by `TsunamiLogic` (channel 1).
- **Created**:
  - `setLogic` item(s) `logicId=SpearStatueLogic` in `PocketGod`: `UnderwaterRuinsDisplayGroup/SpearStatue`
  - stored in `mSpearStatueLogic`, listener = view controller; init plays `SpearStatueInit`.
- **Listener**:
  - `<SpearStatueLogicListener>` (`mSpearStatueLogicListener`): `pygmyLogicArray:`, `onPygmiesSkewered:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `ChainSpearLogic` (created by `transitionIn`: a `ChainSpear` multimodel at (18, 19, statue z), `rot.z = 40`, inserted before the statue, bound to channel 1; `activate`, `shootSpear`, `clearSkeweredPygmies`; it reports `onPygmiesSkewered:` which the statue forwards), `TsunamiLogic` (`showSpear`, `hideSpear`, `transitionIn`, `transitionOut`).
- **Called from**: `hideSpear`, `showSpear`, `transitionIn`, `transitionOut` ← TsunamiLogic.
- **Data ids**:
  - behaviours: `SpearStatueInit`
  - models: `ChainSpear`
  - markers: `TsunamiDisplayMarker`
- **Persistence**: none.
- **Behaviour**: `showSpear` arms it (`activate` → `ChainSpearInit`); a tap while armed sends `shootSpear`; `hideSpear` clears skewered pygmies and removes the spear.

#### `ChainSpearLogic`

`SpriteLogic` · `ChainSpearLogic.c` · 9 methods + 1 item handler · 11 768 B

- **Role**: the chain spear fired by the spear statue.
- **Where**: underwater ruins.
- **Created**:
  - allocated in `SpearStatueLogic::transitionIn` @ 0x1000c3f9c
  - by `SpearStatueLogic::transitionIn` (listener = the statue).
- **Listener**:
  - `<ChainSpearLogicListener>` (`mChainSpearLogicListener`): `pygmyLogicArray:`, `onPygmiesSkewered:`
  - set in `SpearStatueLogic::transitionIn` @ 0x1000c3f9c
- **Talks to**: `PygmyLogic` (`canBeHitByChainSpear:`, `hitByChainSpear:`, `hitByChainSpearRetract:`, release when cleared).
  - also sent in code: `PygmyLogic` (`hitByChainSpearComplete:`).
- **Called from**: `activate`, `clearSkeweredPygmies`, `shootSpear` ← SpearStatueLogic.
- **Data ids**:
  - behaviours: `ChainSpearInit`, `ChainSpearShoot`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `onShootComplete:::` @ 0x1000626d8 (1 use).
- **Persistence**: none.
- **Behaviour**: `shootSpear` (once at a time) → `ChainSpearShoot`. While extending, `onGameFrame:` @ 0x100061dd8 computes the tip as `pos + (cos, sin)(rot.z)·(L + 120)` with `L` read from the current binary frame (tip-channel x); each new tip makes the segment from the previous tip; a pygmy that `canBeHitByChainSpear:` whose `(px, py + 32)` lies between the two endpoints (both dot-product tests ≤ 0) and within 20 px of the line is skewered: bound to animation channel *k + 1* (k = number already skewered, max 6) and `hitByChainSpear:`. `animExtended` sends `hitByChainSpearRetract:` to all; `onShootComplete:::` → `clearSkeweredPygmies` (channels 1–6 cleared, listener `onPygmiesSkewered: count`).

#### `UnderwaterTempleLogic`

`SpriteLogic` · `UnderwaterTempleLogic.c` · 13 methods + 1 class method · 20 201 B

- **Role**: the underwater temple of the coral reef: nine symbol buttons; entering one of five symbol patterns unlocks an underwater power mode (cold, heat, oil, acid, air).
- **Where**: underwater coral reef (location 9); entry 547 (`UnderwaterTemple`), buttons group `UnderwaterTempleButtonsDisplayGroup`.
- **Created**:
  - `createLogicClasses` table entry 547 (#547 new `BCMultiModel` `UnderwaterTemple`)
  - entry 547; `+instance` returns `DAT_1007d1ef8` (set in init). `TsunamiLogic`'s `activateUnderwaterTemple:::` activates it at location 9 and `deactivate`s it when leaving.
- **Listener**:
  - `<UnderwaterTempleLogicListener>` (`mUnderwaterTempleLogicListener`): `onUnderwaterTempleModeChanged:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `UnderwaterTempleSymbolLogic` (entries 538–546, each paired in init with the shape `MultiModelLogic` 529–537 and behaviours `UnderwaterTempleSymbolShape01…09` / `UnderwaterTempleSymbolShapeGlow01…09`), the nine symbol `BCButtonLogic`s 520–528 (listener = the temple), `IdolManager` (`triggerEpisodeEvent:`), the view controller asks `underwaterTempleMode`.
  - also sent in code: `UnderwaterTempleSymbolLogic` (`isActive`, `setSymbolShapeLogic:shapeBehaviorId:shapeGlowBehaviorId:`, `setUnderwaterTempleMode:`, `toggleSymbol`).
  - `getLogic:` 520–528 `BCButtonLogic`, 529–537 `MultiModelLogic`, 538–546 `UnderwaterTempleSymbolLogic`.
- **Called from**: `activate` ← PocketGodViewController(Game), TsunamiLogic; `deactivate` ← TsunamiLogic; `underwaterTempleMode` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `UnderwaterTempleInit`, `UnderwaterTempleSymbolShape01`, `UnderwaterTempleSymbolShape02`, `UnderwaterTempleSymbolShape03`, `UnderwaterTempleSymbolShape04`, `UnderwaterTempleSymbolShape05`, `UnderwaterTempleSymbolShape06`, `UnderwaterTempleSymbolShape07`, `UnderwaterTempleSymbolShape08`, `UnderwaterTempleSymbolShape09`, `UnderwaterTempleSymbolShapeGlow01`, `UnderwaterTempleSymbolShapeGlow02`, `UnderwaterTempleSymbolShapeGlow03`, `UnderwaterTempleSymbolShapeGlow04`, `UnderwaterTempleSymbolShapeGlow05`, `UnderwaterTempleSymbolShapeGlow06`, `UnderwaterTempleSymbolShapeGlow07`, `UnderwaterTempleSymbolShapeGlow08`, `UnderwaterTempleSymbolShapeGlow09`
  - groups: `UnderwaterTempleButtonsDisplayGroup`
  - markers: `TsunamiDisplayMarker`
  - episode events: `EnableUnderwaterPowerAcid`, `EnableUnderwaterPowerAir`, `EnableUnderwaterPowerCold`, `EnableUnderwaterPowerHeat`, `EnableUnderwaterPowerOil`, `ToggleUnderwaterTempleSymbol`
- **Persistence**: none.
- **Behaviour**:
  - `activate` inserts the buttons group before `TsunamiDisplayMarker` at the temple's x/y; `deactivate` removes it.
  - `onButtonPress:touch:` @ 0x1000c88d8 for button *i* (0-based): plays sound `TempleTone<i+1>`, episode event `ToggleUnderwaterTempleSymbol`, toggles symbol *i*, then compares the nine `isActive` flags with the patterns (rows of 3, buttons 1–9): mode 1 Cold `111 010 111`, 2 Heat `110 100 111`, 3 Oil `111 100 100`, 4 Acid `101 111 100`, 5 Air `111 111 010` (`DAT_100602c58 + 9·k`). A match sets the mode, triggers `EnableUnderwaterPowerCold/Heat/Oil/Acid/Air`, tells the listener `onUnderwaterTempleModeChanged:` and sets every symbol's mode (pattern members get the mode, others 0). No match with a mode active → mode 0, `onUnderwaterTempleModeChanged: 0`, all symbols mode 0.
  - `checkSwitchArray:` compares; `activateSwitchArray:` / `deactivateSwitchArray` apply modes.

#### `UnderwaterTempleSymbolLogic`

`SpriteLogic` · `UnderwaterTempleSymbolLogic.c` · 6 methods · 3 282 B

- **Role**: one of the nine glowing symbols on the underwater temple.
- **Where**: underwater coral reef (entries 538–546, `UnderwaterTempleSymbol01…09`).
- **Created**:
  - `createLogicClasses` table entries 538–546 (new `BCMultiModel` `UnderwaterTempleSymbol01`, new `BCMultiModel` `UnderwaterTempleSymbol02`, new `BCMultiModel` `UnderwaterTempleSymbol03`, … (9 entries))
  - init plays `UnderwaterTempleSymbolInit`; `UnderwaterTempleLogic` init pairs it with its shape logic.
- **Listener**: none.
- **Called from**: `isActive`, `setSymbolShapeLogic:shapeBehaviorId:shapeGlowBehaviorId:`, `setUnderwaterTempleMode:`, `toggleSymbol` ← UnderwaterTempleLogic.
- **Data ids**:
  - behaviours: `UnderwaterTempleSymbolAcid`, `UnderwaterTempleSymbolAir`, `UnderwaterTempleSymbolCold`, `UnderwaterTempleSymbolHeat`, `UnderwaterTempleSymbolInit`, `UnderwaterTempleSymbolOff`, `UnderwaterTempleSymbolOil`, `UnderwaterTempleSymbolOn`
- **Persistence**: none.
- **Behaviour**: `setSymbolShapeLogic:shapeBehaviorId:shapeGlowBehaviorId:` stores the shape logic and plays the normal shape behaviour. `toggleSymbol` flips `mbActive`: on → shape glow behaviour + `UnderwaterTempleSymbolOn`; off → normal shape + `UnderwaterTempleSymbolOff`. `setUnderwaterTempleMode: m` (only while active) plays `…On` (0), `…Cold` (1), `…Heat` (2), `…Oil` (3), `…Acid` (4), `…Air` (5).

#### `JellyfishLogic`

`SpriteLogic` · `JellyfishLogic.c` · 31 methods + 7 item handlers · 37 190 B

- **Role**: the five jellyfish of the coral reef: they drift, can be dragged, sting or suffocate pygmies, and when gathered together form a spinning "circle jerk".
- **Where**: underwater coral reef (location 9), `Jellyfish1…5` multimodels.
- **Created**:
  - `createLogicClasses` table entries 93–97 (new `BCMultiModel` `Jellyfish1`, new `BCMultiModel` `Jellyfish2`, new `BCMultiModel` `Jellyfish3`, … (5 entries))
  - entries 93–97; entry 98 is the circle-jerk effect (`MultiModelLogic`, `JellyfishCircleJerk…` behaviours, placed by `createCircleJerk:::` before `CoralReefObjectDisplayMarker`).
- **Listener**:
  - `<JellyfishLogicListener>` (`mJellyfishLogicListener`): `pygmyLogicArray:`, `jellyfishLogicArray:`, `onJellyfishDrag:`, `onJellyfishDragComplete:`
  - wired by `initLogicListeners` to `PocketGodViewController` (5)
- **Talks to**: `PygmyLogic` (`canBeHitByJellyfish:`, `suffocatedByJellyfish:`, `stungByJellyfish:`, `jellyfishAttackComplete:`; `PygmyLogic(Underwater)` calls `canInteractWithPygmy`, `isCircleJerkLeader`, `circleJerkPos`, `onJellyfishAttackComplete:`, `onJellyfishSuffocateAttackComplete:`; `PygmyLogic` calls `cancelJellyfishAttack:`), other jellyfish via the listener's `jellyfishLogicArray:` (`canFormCircleJerk`, `startCircleJerkXPos:yPos:`, `setCircleLeader`).
  - `getLogic:` 98 `MultiModelLogic`.
- **Called from**: `canInteractWithPygmy`, `circleJerkPos`, `drinkPygmyLogic:`, `eatPygmyLogic:`, `isCircleJerkLeader`, `onJellyfishAttackComplete:`, `onJellyfishSuffocateAttackComplete:`, `shockPygmyLogic:` ← PygmyLogic(Underwater); `cancelJellyfishAttack:` ← PygmyLogic.
- **Data ids**:
  - behaviours: `JellyfishCircleJerk`, `JellyfishCircleJerkExit`, `JellyfishCircleJerkInit`, `JellyfishCircleJerkLeader`, `JellyfishCircleJerkShock`, `JellyfishCircleJerkSoloShock`, `JellyfishDie`, `JellyfishDrag`, `JellyfishDragMove`, `JellyfishDrinkPygmy`, `JellyfishEatPygmy`, `JellyfishEnterIsland`, `JellyfishExitIsland`, `JellyfishIdle`, `JellyfishStingPygmy`, `JellyfishSuffocatePygmy`
  - markers: `CoralReefObjectDisplayMarker`, `UnderwaterPiranhaDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `createCircleJerk:::` @ 0x10013c498 (1 use), `swimToTarget:::` @ 0x10013c59c (2 uses), `enterIslandComplete:::` @ 0x10013c688 (2 uses), `swim:::` @ 0x10013c6b0 (16 uses), `drag:::` @ 0x10013c87c (40 uses), `onJellyfishAttackComplete:::` @ 0x10013ccf0 (1 use), `die:::` @ 0x10013cd84 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Box (−30,−30)–(30,30). `enterIsland:` @ 0x10013b890: start at `x = (rand%3801 + 500)/10`, `y = (rand%1501 − 2000)/10` (below the screen), target `(x, (rand%1501 + 500)/10)`, `JellyfishEnterIsland`. `exitIsland:` swims up to `y = 420` (a circle leader also ends the circle: logic 98 `JellyfishCircleJerkExit`).
  - `onGameFrame:` @ 0x10013c24c: buoyant gravity `(0, −30)`, or `(0, −3000)` while dragged; integration; screen collision once entered.
  - `checkForCircleJerk` @ 0x10013bd10: when every jellyfish that `canFormCircleJerk` lies within a 60×60 box, the centre (clamped to x 100–380, y 100–220) becomes `mCircleJerkPos`, each joins a circle of radius 60 starting at angle π/2 with step 2π/n (`startCircleJerkXPos:yPos:` → `JellyfishCircleJerk`), and the checking jellyfish becomes the leader (`JellyfishCircleJerkLeader`).
  - `checkPygmyCollision` @ 0x10013ce1c: a pygmy that `canBeHitByJellyfish:` within 30 px of `(x, y − 20)` is suffocated (`JellyfishSuffocatePygmy`), within 30 px of `(x, y − 80)` stung (`JellyfishStingPygmy`); the jellyfish stops and becomes unselectable until the attack completes (`onJellyfishSuffocateAttackComplete:` respawns it below the screen).

### 4.5 No Pee (Call of Booty) mini-game

The No Pee mini-game, which players see as "Call of Booty" (episode 48 in `assets/data/episodes.json`). It starts when water wings are strapped on a pygmy. `NoPeeLogic` hosts a horizontally wrapping 3000 px underwater world whose objects all derive from `NoPeeObjectLogic`. The table wires all their listeners to `NoPeeLogic`.

#### `WaterWingsLogic`

`SpriteLogic` · `WaterWingsLogic.c` · 27 methods + 5 item handlers · 31 883 B

- **Role**: the water wings: dropped onto a pygmy it straps them on; the pygmy jumps into the sea and the No Pee mini-game starts.
- **Where**: surface islands (table entry 92, `WaterWings` multimodel, box (−15,−20)–(50,20)).
- **Created**: `createLogicClasses` table entry 92 (#92 new `BCMultiModel` `WaterWings`)
- **Listener**:
  - `<WaterWingsLogicListener>` (`mWaterWingsLogicListener`): `isStoryActive`, `isGravityEnabled`, `areWaterWingsEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onWaterWingsDrag:`, `onWaterWingsDragComplete:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `NoPeeLogic` (`+instance`, `startNoPee:` from `waterWingsSplashComplete:::`), `PygmyLogic` (`wearWaterWings:`, `ejectFromWaterWings`; the pygmy calls `wearingWaterWingsComplete:`).
- **Called from**: `enterIsland:`, `exitIsland:`, `exitIslandNow` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `WaterWingsBounce`, `WaterWingsDie`, `WaterWingsDrag`, `WaterWingsDroppedByHurricane`, `WaterWingsEnterIsland`, `WaterWingsFall`, `WaterWingsFallIgnoreIsland`, `WaterWingsPickedUpByHurricane`, `WaterWingsSplash`, `WaterWingsStrapOnPygmy`
  - groups: `IslandDisplayGroup`
  - markers: `IslandDisplayMarker`, `SmallObjectsMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `waterWingsSplashComplete:::` @ 0x100110558 (1 use), `drag:::` @ 0x100110cd0 (40 uses), `fall:::` @ 0x100110e88 (134 uses), `fallFromHurricane:::` @ 0x100111044 (9 uses), `die:::` @ 0x100111188 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Shared pattern of §2.2 with slide friction 0.04, floor `y = 80`, `WaterWingsBounce` / `WaterWingsDie`; `enterIsland:` drops it at `x = 300` (or straight onto the floor).
  - `checkPygmyCollision` @ 0x100110228: while falling (`vel.y < −20`), no wearer and collisions enabled, a pygmy within 30 px of `(px, py + 31)` that accepts `wearWaterWings:` gets them: the wings snap to the pygmy, face away from the screen centre (`scale.x = −1` when `x > 240`), bind the pygmy on channel 1 and play `WaterWingsStrapOnPygmy`. The `waterWingsSplashComplete:::` item hands the pygmy to `[NoPeeLogic instance] startNoPee:` and re-enters the island.
  - `wearingWaterWingsComplete:` (from the pygmy) clears channel 1 and re-enters the island.

#### `NoPeeLogic`

`SpriteLogic` · `NoPeeLogic.c` · 28 methods + 1 class method · 27 433 B

- **Role**: host of the "No Pee" mini-game (shown to players as "Call of Booty", episode 48): a pygmy in water wings dives into a horizontally wrapping underwater world (3000 px) to collect treasure while sea blobs chase it, mines and clams kill it and its breath runs out; a minimap shows the tracked objects; treasure count is the score (challenge game 15).
- **Where**: entry 91 (`NoPeeDisplayGroup`); the game world is `NoPeeDisplayGroup`, the minimap `NoPeeMapDisplayGroup`.
- **Created**:
  - `createLogicClasses` table entry 91 (#91 group `NoPeeDisplayGroup`)
  - entry 91; `+instance` returns `DAT_1007d1f50`. Init fetches the map (90), the intro (85, listener = self), the overlay (86), the retry overlay button (87, listener = self via the table), the four parallax layers (52–55), the treasure (56), sea blobs (57–60), fish (61–64), mines (65–68), clams (69–72) and particles (73–82), filling the typed arrays and `mNoPeeCollidableLogicArray`.
- **Listener**:
  - `<NoPeeLogicListener>` (`mNoPeeLogicListener`): `pygmyLogicArray:`, `islandMinXPos`, `islandMaxXPos`, `onNoPeeStart:pygmyLogic:`, `onNoPeeStop:`, `onNoPeeRetry:`, `onNoPeeStartMovingCamera` (never sent by this class: `islandMaxXPos`, `islandMinXPos`, `pygmyLogicArray:`)
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: all `NoPeeObjectLogic` subclasses (`activate`; they ask `noPeeLogic`, `noPeePygmyLogic`, `noPeeCollidableLogicArray`, `isGameActive`), `NoPeeMapLogic` (`addTrackedSpriteLogic:type:`, `clearAllTrackedSpriteLogics`, `takeBreath`), `NoPeeSeaBlobLogic` (`onCollectNoPeeTreasure:` score), `PygmyLogic(NoPee)` (`startNoPee`, `onNoPeeIntroComplete`, `triggerActiveSacrifice`; the pygmy calls `onPygmyStartSwimming`, `onPygmyKilled`), `ChallengeManager` (`setGameScore: 15 score:`), `WaterWingsLogic` starts it (`startNoPee:`).
  - also sent in code: `ChallengeManager` (`setGameScore:score:`); `NoPeeClamLogic` (`activate`); `NoPeeFishLogic` (`activate`); `NoPeeMapLogic` (`startTracking`); `NoPeeMineLogic` (`activate`); `NoPeeParticleLogic` (`activate`); `NoPeeTreasureLogic` (`activate`).
  - `getLogic:` 52–55 `MultiModelLogic`, 56 `NoPeeTreasureLogic`, 57–60 `NoPeeSeaBlobLogic`, 61–64 `NoPeeFishLogic`, 65–68 `NoPeeMineLogic`, 69–72 `NoPeeClamLogic`, 73–82 `NoPeeParticleLogic`, 85–86 `MultiModelLogic`, 87 `BCButtonLogic`, 90 `NoPeeMapLogic`.
- **Called from**: `isGameActive` ← NoPeeSeaBlobLogic; `noPeeLogic` ← NoPeeFishLogic, NoPeeParticleLogic; `noPeePygmyLogic` ← NoPeeClamLogic, NoPeeFishLogic, NoPeeMineLogic, NoPeeParticleLogic, NoPeeSeaBlobLogic, NoPeeTreasureLogic; `onPygmyKilled`, `onPygmyStartSwimming`, `stopNoPee` ← PygmyLogic(NoPee); `startNoPee:` ← WaterWingsLogic.
- **Data ids**:
  - behaviours: `NoPeeIntro`, `NoPeeOverlayExit`, `NoPeeOverlayInit`, `NoPeeRetryOverlayEnter`, `NoPeeRetryOverlayHide`
  - groups: `NoPeeDisplayGroup`
  - event ids: `NoPeeIntroComplete`
- **Persistence**: none.
- **Behaviour**:
  - `startNoPee: p` @ 0x10019d774 → `initNoPee` @ 0x10019d7cc: score 0, active, pygmy `startNoPee`, pygmy bound to the intro's channel 1 and `NoPeeIntro`, overlay `NoPeeOverlayInit`, retry overlay hidden, world group at (0, −680), every object array `activate`d and added to the minimap (types: pygmy 0, treasure 1, obstacle 2, blob 3); then `onNoPeeStart:pygmyLogic:`. `NoPeeIntroComplete` (multi-model event) → pygmy `onNoPeeIntroComplete`, `onNoPeeStartMovingCamera`.
  - `onGameFrame:` @ 0x10019da5c (`gameFrameOrder` 59): parallax — foreground `0.3·group`, backgrounds `−0.2`, `−0.65`, `−0.75` × group position (x and y); overlays pinned to the screen (`−group`).
  - `onCollectNoPeeTreasure`: score + 1, the map `takeBreath`, every sea blob `onCollectNoPeeTreasure: score`, challenge score. `onPygmyStartSwimming` → `NoPeeOverlayExit`; `onPygmyKilled` shows the retry overlay (`NoPeeRetryOverlayEnter`); the retry button restarts (`triggerActiveSacrifice`, `initNoPee`, `onNoPeeRetry:`). `stopNoPee` clears the map and reports `onNoPeeStop:`. `isGameActive` = score > 0.

#### `NoPeeObjectLogic`

`SpriteLogic` · `NoPeeObjectLogic.c` · 9 methods · 7 766 B

- **Role**: abstract base of the No Pee world objects (treasure, sea blob, fish, mine, clam, particle): spawning, wrapping and collision.
- **Where**: No Pee world.
- **Created**: never directly (its subclasses are table entries); `NoPeeMapItemLogic` uses `isKindOfClass:` on it.
- **Listener**:
  - `<NoPeeObjectLogicListener>` (`mNoPeeObjectLogicListener`): `noPeePygmyLogic`, `noPeeLogic`, `noPeeCollidableLogicArray`, `isGameActive` (never sent by this class: `isGameActive`)
  - set in `NoPeeClamLogic::setNoPeeClamLogicListener:` @ 0x1000c0850, `NoPeeFishLogic::setNoPeeFishLogicListener:` @ 0x1001359d4, `NoPeeMineLogic::setNoPeeMineLogicListener:` @ 0x1000d58f0, `NoPeeParticleLogic::setNoPeeParticleLogicListener:` @ 0x1001753cc, `NoPeeSeaBlobLogic::setNoPeeSeaBlobLogicListener:` @ 0x100178f18, `NoPeeTreasureLogic::setNoPeeTreasureLogicListener:` @ 0x10008ddf4
- **Talks to**: `PygmyLogic` (`canBeHitByNoPeeObject`).
- **Called from**: `activate` ← NoPeeLogic.
- **Data ids**:
  - markers: `NoPeePygmyDisplayMarker`
- **Persistence**: none.
- **Behaviour**: `activate` inserts the object before `NoPeePygmyDisplayMarker` (once). `randomSpawnPosition`: `x = (rand%30001)/10`, `y = (rand%7001 + 1000)/10`. `randomSpawnPositionNoOverlapWithMinYPos:maxYPos:` @ 0x10003486c: x random within ±1500 of the screen centre in world space (`240 − group.x`), y random in [min, max], retried until it does not overlap any collidable object. `checkCollision:collisionRadius:` = the pygmy `canBeHitByNoPeeObject` and lies within the radius. `onGameFrame:` @ 0x100034c98 wraps the object by ±3000 to stay within ±1500 of the screen centre. `shouldHideNoPeeMapItem` = NO.

#### `NoPeeTreasureLogic`

`NoPeeObjectLogic` · `NoPeeTreasureLogic.c` · 7 methods + 2 item handlers · 4 595 B

- **Role**: the treasure chest of No Pee.
- **Where**: entry 56 (`NoPeeTreasure1`).
- **Created**: `createLogicClasses` table entry 56 (#56 new `BCMultiModel` `NoPeeTreasure1`)
- **Listener**:
  - `<NoPeeTreasureLogicListener>` (`mNoPeeTreasureLogicListener`): `onCollectNoPeeTreasure`; inherited from `<NoPeeObjectLogicListener>`: `noPeePygmyLogic`, `noPeeLogic`, `noPeeCollidableLogicArray`, `isGameActive` (never sent by this class: `isGameActive`)
  - wired by `initLogicListeners` to `NoPeeLogic`
- **Called from**: `activate` ← NoPeeLogic.
- **Data ids**:
  - behaviours: `NoPeeTreasureCollect`, `NoPeeTreasureInit`
  - other strings: `_`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `setAnimationBasedOnLevel:::` @ 0x10008e014 (1 use), `onNoPeeTreasureCollectComplete:::` @ 0x10008e0e8 (1 use).
- **Persistence**: none.
- **Behaviour**: `activate`: level 0, uncollected, `NoPeeTreasureInit`, random spawn y 100–800. `onGameFrame:`: the pygmy within 50 → collected, listener `onCollectNoPeeTreasure`, `NoPeeTreasureCollect` (`onNoPeeTreasureCollectComplete:::` raises the level and respawns it; `setAnimationBasedOnLevel:::` picks the chest look). Hidden on the map while collected.

#### `NoPeeSeaBlobLogic`

`NoPeeObjectLogic` · `NoPeeSeaBlobLogic.c` · 9 methods · 7 957 B

- **Role**: a sea blob that drifts and, once treasure has been collected, chases the pygmy (faster after each treasure).
- **Where**: entries 57–60.
- **Created**: `createLogicClasses` table entries 57–60 (#57 new `BCMultiModel` `NoPeeSeaBlob1`; #58 new `BCMultiModel` `NoPeeSeaBlob2`; #59 new `BCMultiModel` `NoPeeSeaBlob3`; #60 new `BCMultiModel` `NoPeeSeaBlob4`)
- **Listener**:
  - `<NoPeeSeaBlobLogicListener>` (`mNoPeeSeaBlobLogicListener`): inherited from `<NoPeeObjectLogicListener>`: `noPeePygmyLogic`, `noPeeLogic`, `noPeeCollidableLogicArray`, `isGameActive` (never sent by this class: `isGameActive`)
  - wired by `initLogicListeners` to `NoPeeLogic` (4)
- **Talks to**: `PygmyLogic` (`canBeHitByNoPeeObject`, `hitByNoPeeSeaBlob:`).
- **Called from**: `activate`, `onCollectNoPeeTreasure:` ← NoPeeLogic.
- **Data ids**:
  - behaviours: `NoPeeSeaBlobEatPygmy`, `NoPeeSeaBlobInit`
- **Persistence**: none.
- **Behaviour**: `activate`: `NoPeeSeaBlobInit`, spawn y 100–800. `calculateSwimPosVel`: target velocity `((rand%2001 − 1000)/10, (rand%401 − 200)/10)`, y direction forced back inside 100–800. `onGameFrame:` @ 0x100179028: while the game is not active, the pygmy cannot be hit or the swim timer runs, the blob drifts (new target every `(rand%16 + 5)/10` s, velocity eased 50 %); otherwise it chases the pygmy (offset by `mTargetYOffset`) at `mChasePosVel`, and hits it. `onCollectNoPeeTreasure: n`: `targetYOffset = (rand%4001 − 2000)/10`, chase speed `10·n + (rand%501 + 2300)/10`, swim delay `(rand%11 + 10)/10 − n/5` (0 from the fifth treasure).

#### `NoPeeFishLogic`

`NoPeeObjectLogic` · `NoPeeFishLogic.c` · 6 methods · 4 290 B

- **Role**: a decorative fish swimming across the No Pee world.
- **Where**: entries 61–64.
- **Created**: `createLogicClasses` table entries 61–64 (#61 new `BCMultiModel` `NoPeeFish1`; #62 new `BCMultiModel` `NoPeeFish2`; #63 new `BCMultiModel` `NoPeeFish3`; #64 new `BCMultiModel` `NoPeeFish4`)
- **Listener**:
  - `<NoPeeFishLogicListener>` (`mNoPeeFishLogicListener`): inherited from `<NoPeeObjectLogicListener>`: `noPeePygmyLogic`, `noPeeLogic`, `noPeeCollidableLogicArray`, `isGameActive` (never sent by this class: `isGameActive`)
  - wired by `initLogicListeners` to `NoPeeLogic` (4)
- **Called from**: `activate` ← NoPeeLogic.
- **Data ids**:
  - behaviours: `NoPeeFishInit`
- **Persistence**: none.
- **Behaviour**: `activate`: `vel = ((rand%1001 − 500)/10 ± 100, (rand%201 − 100)/10)`; the first frame places it around the screen centre (x ± 440, y random); it swims, flips with its direction and respawns when far off screen (`onGameFrame:` @ 0x100135b58).

#### `NoPeeMineLogic`

`NoPeeObjectLogic` · `NoPeeMineLogic.c` · 6 methods · 2 804 B

- **Role**: a sea mine of No Pee.
- **Where**: entries 65–68.
- **Created**: `createLogicClasses` table entries 65–68 (#65 new `BCMultiModel` `NoPeeMine1`; #66 new `BCMultiModel` `NoPeeMine2`; #67 new `BCMultiModel` `NoPeeMine3`; #68 new `BCMultiModel` `NoPeeMine4`)
- **Listener**:
  - `<NoPeeMineLogicListener>` (`mNoPeeMineLogicListener`): inherited from `<NoPeeObjectLogicListener>`: `noPeePygmyLogic`, `noPeeLogic`, `noPeeCollidableLogicArray`, `isGameActive` (never sent by this class: `isGameActive`)
  - wired by `initLogicListeners` to `NoPeeLogic` (4)
- **Talks to**: `PygmyLogic` (`hitByNoPeeMine:`).
- **Called from**: `activate` ← NoPeeLogic.
- **Data ids**:
  - behaviours: `NoPeeMineHit`, `NoPeeMineInit`
- **Persistence**: none.
- **Behaviour**: `activate`: `NoPeeMineInit`, spawn y 100–800; the pygmy within 50 → `hitByNoPeeMine:`, `NoPeeMineHit` (once).

#### `NoPeeClamLogic`

`NoPeeObjectLogic` · `NoPeeClamLogic.c` · 7 methods · 3 275 B

- **Role**: a giant clam on the No Pee sea floor.
- **Where**: entries 69–72.
- **Created**: `createLogicClasses` table entries 69–72 (#69 new `BCMultiModel` `NoPeeClam1`; #70 new `BCMultiModel` `NoPeeClam2`; #71 new `BCMultiModel` `NoPeeClam3`; #72 new `BCMultiModel` `NoPeeClam4`)
- **Listener**:
  - `<NoPeeClamLogicListener>` (`mNoPeeClamLogicListener`): inherited from `<NoPeeObjectLogicListener>`: `noPeePygmyLogic`, `noPeeLogic`, `noPeeCollidableLogicArray`, `isGameActive` (never sent by this class: `isGameActive`)
  - wired by `initLogicListeners` to `NoPeeLogic` (4)
- **Talks to**: `PygmyLogic` (`hitByNoPeeClam:`).
- **Called from**: `activate` ← NoPeeLogic.
- **Data ids**:
  - behaviours: `NoPeeClamEatPygmy`, `NoPeeClamInit`
- **Persistence**: none.
- **Behaviour**: init `NoPeeClamInit`; `activate` spawns it on the floor (`y = 70`), randomly mirrored; the pygmy within 50 → `hitByNoPeeClam:`, `NoPeeClamEatPygmy`.

#### `NoPeeParticleLogic`

`NoPeeObjectLogic` · `NoPeeParticleLogic.c` · 6 methods · 3 921 B (the file also holds `FUN_1001757bc` (317 B), the `gameFrameOrder` sort comparator used by `BCView`; not counted)

- **Role**: a floating particle (bubble/plankton) of the No Pee world.
- **Where**: entries 73–82.
- **Created**: `createLogicClasses` table entries 73–82 (new `BCMultiModel` `NoPeeParticle1`, new `BCMultiModel` `NoPeeParticle2`, new `BCMultiModel` `NoPeeParticle3`, … (10 entries))
- **Listener**:
  - `<NoPeeParticleLogicListener>` (`mNoPeeParticleLogicListener`): inherited from `<NoPeeObjectLogicListener>`: `noPeePygmyLogic`, `noPeeLogic`, `noPeeCollidableLogicArray`, `isGameActive` (never sent by this class: `isGameActive`)
  - wired by `initLogicListeners` to `NoPeeLogic` (10)
- **Called from**: `activate` ← NoPeeLogic.
- **Data ids**:
  - behaviours: `NoPeeParticleInit`
- **Persistence**: none.
- **Behaviour**: init `NoPeeParticleInit`; `activate` gives a random drift `((rand%1001 − 500)/10, (rand%201 − 100)/10)`; placed around the screen centre on the first frame (x ± 270, y ± 190) and recycled around the view (`onGameFrame:` @ 0x100175528).

#### `NoPeeMapLogic`

`SpriteLogic` · `NoPeeMapLogic.c` · 10 methods · 6 616 B

- **Role**: the No Pee minimap and breath meter.
- **Where**: entry 90 (`NoPeeMapDisplayGroup`).
- **Created**:
  - `createLogicClasses` table entry 90 (#90 group `NoPeeMapDisplayGroup`)
  - listener = `NoPeeLogic` (table); init fetches the map background (entry 88).
- **Listener**:
  - `<NoPeeMapLogicListener>` (`mNoPeeMapLogicListener`): `noPeeLogic`, `noPeePygmyLogic`
  - wired by `initLogicListeners` to `NoPeeLogic`
- **Talks to**: `NoPeeMapItemLogic` (one `NoPeeMapItem` multimodel per tracked object, listener = the map).
  - also sent in code: `NoPeeMapItemLogic` (`setTrackedSpriteLogic:type:`); `PygmyLogic` (`canBeHitByNoPeeObject`, `noPeeRunOutOfBreath`).
  - `getLogic:` 88 `MultiModelLogic`.
- **Called from**: `addTrackedSpriteLogic:type:`, `clearAllTrackedSpriteLogics`, `startTracking`, `takeBreath` ← NoPeeLogic.
- **Data ids**:
  - models: `NoPeeMapItem`
- **Persistence**: none.
- **Behaviour**: `addTrackedSpriteLogic:type:` adds an item at (0, 0, −277.12) inside the map group; `clearAllTrackedSpriteLogics`. Breath: `takeBreath` refills to 1 and starts tracking (also records `CFAbsoluteTimeGetCurrent()` in `DAT_100821f58`); `onGameFrame:` @ 0x100182fd8 (`gameFrameOrder` 60) drains `dt/15` per second while the pygmy can be hit, sends `noPeeRunOutOfBreath` at 0, drives the background's frame percent `1 − breath`, and pins the map to the screen at `(240 − group.x − 74, 320 − group.y − 71)`.

#### `NoPeeMapItemLogic`

`SpriteLogic` · `NoPeeMapItemLogic.c` · 6 methods · 3 330 B

- **Role**: one dot on the No Pee minimap.
- **Where**: inside `NoPeeMapDisplayGroup`.
- **Created**:
  - allocated in `NoPeeMapLogic::addTrackedSpriteLogic:type:` @ 0x100182ca0
  - by `NoPeeMapLogic::addTrackedSpriteLogic:type:`.
- **Listener**:
  - `<NoPeeMapItemLogicListener>` (`mNoPeeMapItemLogicListener`): `noPeeLogic`
  - set in `NoPeeMapLogic::addTrackedSpriteLogic:type:` @ 0x100182ca0
- **Called from**: `setTrackedSpriteLogic:type:` ← NoPeeMapLogic.
- **Data ids**:
  - behaviours: `NoPeeMapItemTypeBlob`, `NoPeeMapItemTypeObstacle`, `NoPeeMapItemTypePygmy`, `NoPeeMapItemTypeTreasure`
- **Persistence**: none.
- **Behaviour**: type → `NoPeeMapItemTypePygmy` (0), `…Treasure` (1), `…Obstacle` (2), `…Blob` (3). `onGameFrame:` @ 0x10017f860 (order 60): `x = (obj.x − (240 − group.x))·148/3000 + 76`, `y = obj.y·50/1000 + 10`; hides it when the tracked `NoPeeObjectLogic` says `shouldHideNoPeeMapItem`.

#### `NoPeeMapScreenLogic`

`SpriteLogic` · `NoPeeMapScreenLogic.c` · 5 methods · 1 852 B

- **Role**: the "view window" rectangle on the No Pee minimap.
- **Where**: entry 89 (`NoPeeMapDisplayGroup/MapScreen`, animation `NoPeeMapScreenAnim`).
- **Created**:
  - `createLogicClasses` table entry 89 (#89 `NoPeeMapDisplayGroup/MapScreen`); init steps: `play_animation NoPeeMapScreenAnim`
  - listener = `NoPeeLogic` (table).
- **Listener**:
  - `<NoPeeMapScreenLogicListener>` (`mNoPeeMapScreenLogicListener`): `noPeeLogic`
  - wired by `initLogicListeners` to `NoPeeLogic`
- **Persistence**: none.
- **Behaviour**: `onGameFrame:` (order 60): `x = 64`, `y = group.y·−50/1000 + 10`.

#### `NoPeePillarLogic`

`SpriteLogic` · `NoPeePillarLogic.c` · 10 methods · 5 247 B

- **Role**: an unused No Pee obstacle (path marker / large obstacle with tunnel, "flappy" and collapsing collision tests) — dead code in 1.48.2.
- **Where**: none.
- **Created**: never instantiated (no class reference outside its own methods).
- **Listener**: `<NoPeePillarLogicListener>` (`mNoPeePillarLogicListener`): no methods
- **Data ids**:
  - behaviours: `NoPeePillarLargeObstacle`, `NoPeePillarPathMarker`
- **Persistence**: none.
- **Behaviour**: never instantiated (no class reference or string). For completeness: `setNoPeePillarType:` 0 → `NoPeePillarPathMarker`, 1 → `NoPeePillarLargeObstacle`; `checkCollision:lastPos:` = `checkTunnelCollision:lastPos:` (a slanted tunnel of half-width `sin·30` between the configured x range); `checkFlappyCollision:lastPos:` / `checkCollapsingCollision:lastPos:` = box (x, x + 80) × (y − 40, y + 40). The port may omit it.

### 4.6 Ice island and boss health bar

Ice island (state 4) with its boss, the ice monster. `HealthBarLogic` is the boss bar shared with the T-Rex and the spider. The boss currently fighting sets itself as the bar's listener.

#### `IceMonsterLogic`

`SpriteLogic` · `IceMonsterLogic.c` · 27 methods + 10 item handlers · 23 385 B

- **Role**: the ice monster boss rising from the sea next to the ice island: dormant until hit by a thrown pygmy, then fights with a health bar, throws snowballs and recharges; killing it earns `ConqueredIceMonster`.
- **Where**: ice island (location 4), `IslandDisplayGroup/IceMonster`.
- **Created**:
  - `setLogic` item(s) `logicId=IceMonsterLogic` in `PocketGod`: `IslandDisplayGroup/IceMonster`
  - stored in `mIceMonsterLogic`, listener = view controller; init stops its game frame and remembers its start x.
- **Listener**:
  - `<IceMonsterLogicListener>` (`mIceMonsterLogicListener`): `islandMinXPos`, `islandMaxXPos`, `healthBarLogic:`, `pygmyLogicArray:`, `onSnowBallHitPygmy:`, `onSnowBallMissPygmy:`, `onIceMonsterVisible:`, `onIceMonsterHitByPygmy:`, `onIceMonsterKilled:`, `onIceMonsterStartBattle:`, `onIceMonsterCancelBattle:`, `onAchievement:count:highScoreWins:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `HealthBarLogic` (via the listener's `healthBarLogic:`: becomes its listener, `activate`, `deactivate`, `setHealthPercent:`; answers `healthBarType` = 2 and `shouldHealthBarBeVisible:` = YES), `SnowBallLogic` (spawned by `throwSnowBallXOffset:yOffset:range:`, listener = the monster; `onSnowBallDie:`, hits/misses forwarded to the listener), `PygmyLogic` (`hitByPygmy:onFire:`, `hitByPygmyInMouth:`, `isDormant`, `isRecharging`, the screen collision getters).
- **Called from**: `enterIsland:`, `exitIsland:` ← PocketGodViewController, PocketGodViewController(Game); `hitByPygmy:onFire:`, `hitByPygmyInMouth:`, `isMouthCollisionActive`, `isRecharging`, `screenCollisionRadius`, `screenCollisionYPos`, `screenMouthCollisionRadius`, `screenMouthCollisionYPos` ← PygmyLogic; `isDormant` ← PygmyLogic(Island).
- **Data ids**:
  - behaviours: `IceMonsterBlockPygmy`, `IceMonsterDie`, `IceMonsterEnter`, `IceMonsterExit`, `IceMonsterHitAndKilled`, `IceMonsterHitByPygmy`, `IceMonsterHitByPygmyDizzy`, `IceMonsterRecharge`, `IceMonsterRechargeExit`, `IceMonsterRechargeStart`, `IceMonsterRechargeTurn`, `IceMonsterRise`, `IceMonsterStartEnter`, `IceMonsterSwallowPygmy`, `IceMonsterThrowSnowBalls`, `IceMonsterWalk`, `IceMonsterWalkTurn`
  - models: `SnowBall`
  - markers: `DragDisplayMarker`, `MidOceanMarker`
  - achievements: `ConqueredIceMonster`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `startTransition:::` @ 0x100183e64 (10 uses), `endTransition:::` @ 0x100183e7c (15 uses), `resetPosition:::` @ 0x100183e90 (1 use), `enableMouthCollision:::` @ 0x100183ee0 (3 uses), `disableMouthCollision:::` @ 0x100183ef8 (6 uses), `chooseIdleBehavior:::` @ 0x100183f0c (1 use), `chooseRechargeBehavior:::` @ 0x100184140 (2 uses), `walk:::` @ 0x1001845dc (16 uses), `rechargeWalk:::` @ 0x100184738 (1 use), `die:::` @ 0x1001848ec (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - Collision getters (screen space, factor `k = −277.12/pos.z`): body `y = k·(pos.y + h·scale.y)`, radius `k·scale.y·r` with `(h, r) = (15, 15)` dormant or `(60, 50)` awake; mouth `y = k·(pos.y + 60·scale.y)`, radius `k·scale.y·30` (`mbMouthCollisionActive` toggled by items).
  - `hitByPygmy:onFire:` @ 0x100183c54 (not in transition): dormant → wake: health 100, health bar activated (listener = monster), `onIceMonsterStartBattle:`, `IceMonsterRise`. Awake and not recharging → `IceMonsterBlockPygmy`, damage 5 (10 when the pygmy is on fire). Recharging → `IceMonsterHitByPygmyDizzy` on the second hit else `IceMonsterHitByPygmy`, damage 10 (15 on fire). `hitByPygmyInMouth:` → `IceMonsterSwallowPygmy`, damage 20.
  - `doDamage:` @ 0x100183ae4: health −= d, bar = health/100, `onIceMonsterHitByPygmy:`; at ≤ 0: dormant again, counters reset, `onIceMonsterKilled:`, bar deactivated, achievement `ConqueredIceMonster` (count −1, high-score wins YES), `IceMonsterHitAndKilled`.
  - `onGameFrame:` @ 0x1001837c8: while recharging and below 100, health += 0.005 per frame (bar updated). Idle/recharge choices (`chooseIdleBehavior:::`, `chooseRechargeBehavior:::`, `walk:::`, `rechargeWalk:::`) in sequences.md §5.
  - Snowballs (`animThrowLeft`: offset (−85, 90), range 100; `animThrowRight`: (85, 90), range 0): a `SnowBall` multimodel before `DragDisplayMarker`; `throwSnowBallXOffset:yOffset:range:` @ 0x100184274 picks the target x at random (0.01 steps) in `[p − range, p + range]` around the first pygmy's x (or 240 without pygmies); a window past 460 is shifted left to end at 460, but a window starting left of 20 only gets `lo = 20` (the shift of `hi` is computed after `lo` changed, so it adds 0; see section 7.2); `vel = (targetX − startX, 1500, 500)`.
  - `exitIsland:` cancels the battle (`onIceMonsterCancelBattle:`), hides the bar, `IceMonsterExit` (or kills the snowballs when leaving at once).

#### `SnowBallLogic`

`SpriteLogic` · `SnowBallLogic.c` · 6 methods + 2 item handlers · 8 991 B

- **Role**: a snowball thrown by the ice monster.
- **Where**: ice island.
- **Created**:
  - allocated in `IceMonsterLogic::throwSnowBallXOffset:yOffset:range:` @ 0x100184274
  - by `IceMonsterLogic::throwSnowBallXOffset:yOffset:range:` (listener = the monster).
- **Listener**:
  - `<SnowBallLogicListener>` (`mSnowBallLogicListener`): `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onSnowBallHitPygmy:`, `onSnowBallMissPygmy:`, `onSnowBallDie:`
  - set in `IceMonsterLogic::throwSnowBallXOffset:yOffset:range:` @ 0x100184274
- **Talks to**: `PygmyLogic` (`hitBySnowBall:`).
- **Data ids**:
  - behaviours: `SnowBallDie`, `SnowBallFly`, `SnowBallSplash`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `fall:::` @ 0x100010b00 (134 uses), `die:::` @ 0x100010c80 (119 uses).
- **Persistence**: none.
- **Behaviour**: init `SnowBallFly`. `onGameFrame:` @ 0x1000107b4: gravity −3000, integration (with `vel.z`), pygmy check. `checkPygmyCollision:` @ 0x100010558 (not yet hit, not landed, `−377.12 ≤ z ≤ −227.12`): a pygmy with `|x − px| < 40` and `|(y + 28) − (py + 20)| < 40` that accepts `hitBySnowBall:` → listener `onSnowBallHitPygmy:`. The `fall:::` handler reports `onSnowBallMissPygmy:` on the island or in the sea (y < 30); `die:::` → `onSnowBallDie:`.

#### `HealthBarLogic`

`SpriteLogic` · `HealthBarLogic.c` · 8 methods + 2 item handlers · 5 692 B

- **Role**: the boss health bar shared by the T-Rex (type 0), the spider (1) and the ice monster (2).
- **Where**: HUD (`PocketGod/HealthBar`, inserted before `MenuDisplayMarker`).
- **Created**:
  - `setLogic` item(s) `logicId=HealthBarLogic` in `PocketGod`: `PocketGod/HealthBar`
  - stored in `mHealthBarLogic` (no listener at creation; the boss currently fighting sets itself as listener with `setHealthBarLogicListener:`).
- **Listener**:
  - `<HealthBarLogicListener>` (`mHealthBarLogicListener`): `healthBarType`, `shouldHealthBarBeVisible:`
  - set in `IceMonsterLogic::hitByPygmy:onFire:` @ 0x100183c54, `SpiderLogic::callMom` @ 0x10019a1e4, `TRexLogic::callMom` @ 0x10017fccc
- **Called from**: `activate` ← IceMonsterLogic, PocketGodViewController(Game), SpiderLogic, TRexLogic; `deactivate`, `setHealthPercent:` ← IceMonsterLogic, SpiderLogic, TRexLogic; `setTargetXPos:yPos:` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `HealthBarExit`, `HealthBarIceMonsterEnter`, `HealthBarSpiderEnter`, `HealthBarTRexEnter`
  - markers: `MenuDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `showHealthBar:::` @ 0x1000d7114 (3 uses), `hideHealthBar:::` @ 0x1000d712c (1 use).
- **Persistence**: none.
- **Behaviour**: init parks it at `y = −30`. `activate` (health 0 → target 1, inserted), `deactivate`. `setHealthPercent:` clamps to [0, 1]. `onGameFrame:` @ 0x1000d6f18 (only with a listener): `y += (targetY − y)·0.5`; inactive or not `shouldHealthBarBeVisible:` → target −30 + `HealthBarExit`; visible → target 7 + `HealthBarTRexEnter` / `HealthBarSpiderEnter` / `HealthBarIceMonsterEnter` by `healthBarType`; `health += (target − health)·0.5` drives the animation's frame percent (`setFramePercent:`). `hideHealthBar:::` waits until it is off screen and then releases the listener and removes the bar.

#### `GlacierLogic`

`SpriteLogic` · `GlacierLogic.c` · 5 methods · 1 892 B

- **Role**: the glacier backdrop seen from the sea.
- **Where**: `IslandDisplayGroup/Glacier`, inserted before `MidOceanMarker` on `enterIsland:` (ice island).
- **Created**:
  - `setLogic` item(s) `logicId=GlacierLogic` in `PocketGod`: `IslandDisplayGroup/Glacier`
  - stored in `mGlacierLogic`, listener = view controller (protocol empty).
- **Listener**:
  - `<GlacierLogicListener>` (`mGlacierLogicListener`): no methods
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Called from**: `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `GlacierInit`
  - markers: `MidOceanMarker`
- **Persistence**: none.
- **Behaviour**: init stops its game frame, removes it and plays `GlacierInit`; `enterIsland:` inserts it before `MidOceanMarker`; `exitIsland` removes it.

#### `IglooLogic`

`SpriteLogic` · `IglooLogic.c` · 25 methods + 5 item handlers · 12 113 B

- **Role**: the igloo of the ice island: pygmies crawl or fall in, a fish dropped in becomes their dinner, knocking three times throws the resting pygmy out, and tapping a pygmy with a fish fires it out like a cannon.
- **Where**: ice island (`IceIslandDisplayGroup/Igloo`).
- **Created**:
  - `setLogic` item(s) `logicId=IglooLogic` in `PocketGod`: `IceIslandDisplayGroup/Igloo`
  - stored in `mIglooLogic`, listener = view controller (protocol empty); init stops its game frame, sets the apocalypse cloud style `Igloo` and plays `IglooExitIsland`.
- **Listener**:
  - `<IglooLogicListener>` (`mIglooLogicListener`): no methods
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic(Island)` (`isPygmyCrawlAvailable:`, `isPygmyFallAvailable:`, `pygmyCrawlIntoIgloo:`, `pygmyFallIntoIgloo:`, `removePygmyFromIgloo:`; igloo sends `tossFromIgloo`, `leaveIglooTopExplode`, `leaveIglooDoorExplode`, `leaveIglooDoorCrawl`), `FishLogic` (`isFishFallAvailable:`, `fishFallIntoIgloo:`; igloo sends `finishEatingInIgloo`).
- **Called from**: `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `fishFallIntoIgloo:`, `isFishFallAvailable:` ← FishLogic; `isPygmyCrawlAvailable:`, `isPygmyFallAvailable:`, `pygmyCrawlIntoIgloo:`, `pygmyFallIntoIgloo:` ← PygmyLogic(Island); `removePygmyFromIgloo:` ← PygmyLogic.
- **Data ids**:
  - behaviours: `IglooEatFish`, `IglooEnterIsland`, `IglooExitIsland`, `IglooFishFallInside`, `IglooFishOnly`, `IglooKnock`, `IglooMakeRoomForNewPygmy`, `IglooMakeRoomForNewPygmyCooking`, `IglooPygmyCannon`, `IglooPygmyCrawlInside`, `IglooPygmyCrawlOutNoEat`, `IglooPygmyFallInside`
  - models: `Igloo`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `startTransition:::` @ 0x1000c3a3c (10 uses), `endTransition:::` @ 0x1000c3a54 (15 uses), `checkForFishDinner:::` @ 0x1000c3a68 (3 uses), `checkForFishOnly:::` @ 0x1000c3ab4 (1 use), `eatFishComplete:::` @ 0x1000c3b00 (2 uses).
- **Persistence**: none.
- **Behaviour**: one resting pygmy on channel 1 (a second falling pygmy becomes the leaving pygmy on channel 2, `IglooPygmyFallInside` / door-explode variants); `isPygmyCrawlAvailable:` needs a fish inside and nobody resting. `checkForDoorKnocking:` @ 0x1000c3c78: taps count when less than 1 s apart (`onGameFrame:` resets the count after 1 s); the third makes the pygmy crawl out (`IglooPygmyCrawlOutNoEat`). A tap with a resting pygmy and a fish → `IglooPygmyCannon` and the fish is eaten. Items: `checkForFishDinner` (`IglooEatFish`), `checkForFishOnly` (`IglooFishOnly`), `eatFishComplete`, `startTransition`/`endTransition`.

#### `IceTreeLogic`

`SpriteLogic` · `IceTreeLogic.c` · 6 methods · 1 720 B

- **Role**: the snowy tree of the ice island (decoration).
- **Where**: ice island (`IceIslandDisplayGroup/Tree`).
- **Created**:
  - `setLogic` item(s) `logicId=IceTreeLogic` in `PocketGod`: `IceIslandDisplayGroup/Tree`
  - stored in `mIceTreeLogic`, listener = view controller (protocol empty).
- **Listener**:
  - `<IceTreeLogicListener>` (`mIceTreeLogicListener`): no methods
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Called from**: `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `IceTreeEnterIsland`, `IceTreeExitIsland`
- **Persistence**: none.
- **Behaviour**: init stops its game frame, unselectable; `enterIsland:` → `IceTreeEnterIsland`; `exitIsland` removes it (see the class's 6 methods).

#### `IceHoleLogic`

`SpriteLogic` · `IceHoleLogic.c` · 30 methods + 9 item handlers · 25 276 B

- **Role**: the ice-fishing hole: melted by a burning pygmy, it lets pygmies fish (for the swordfish), lets things fall through, hosts the ice squid whose tentacle grabs dragged pygmies, and rains icicles.
- **Where**: ice island (`IceIslandDisplayGroup/IceHole`).
- **Created**:
  - `setLogic` item(s) `logicId=IceHoleLogic` in `PocketGod`: `IceIslandDisplayGroup/IceHole`
  - stored in `mIceHoleLogic`, listener = view controller. Init creates the `Tentacle` multimodel with a `MultiModelLogic`.
- **Listener**:
  - `<IceHoleLogicListener>` (`mIceHoleLogicListener`): `isIceSquidEnabled`, `islandMinXPos`, `islandMaxXPos`, `draggedPygmyLogic:`, `swordFishLogic:`, `fishingPoleLogic:`, `pygmyLogicArray:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic` (the hole sends `canMeltIceHole`, `meltIceHoleComplete:`, `iceHoleSplashComplete`, `canBeGrabbedByTentacle:`, `grabbedByTentacle:`, `eatenByIceSquid:`, `catchSwordFish`, `stopFishingFromIceHole:`; pygmies call `meltIceHole:`, `shouldPygmyWalkTowardIceHole:`, `burntPygmyFallIntoIceHole:`, `beheadPygmy:`, `isFrozen`, `isAvailableForFishing`, `startFishing:`, `startFishingSitDown:`, `stopFishing:`, `tentaclePos`, `removePygmyFromIceHole:`; `tripIntoIceHole:fallForward:` is never sent), `FishingPoleLogic` (via the listener's `fishingPoleLogic:`; bound on channel 2 while ice fishing), `SwordFishLogic` (via `swordFishLogic:`: `canBeCaught`, `catch`), `FishLogic` (calls `isAvailableForFalling`, `fishFallIntoIceHole:`), `IcicleLogic` (spawned by `icicleRain:::`, listener = the hole; `onIcicleDie:`).
- **Called from**: `beheadPygmy:`, `burntPygmyFallIntoIceHole:`, `isAvailableForFishing`, `isFrozen`, `meltIceHole:`, `shouldPygmyWalkTowardIceHole:`, `startFishing:`, `startFishingSitDown:`, `stopFishing:`, `tentaclePos` ← PygmyLogic(Island); `disableIceSquid`, `enableIceSquid`, `exitIsland:` ← PocketGodViewController; `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `fishFallIntoIceHole:` ← FishLogic; `isAvailableForFalling` ← FishLogic, PygmyLogic(Island); `removePygmyFromIceHole:` ← PygmyLogic.
- **Data ids**:
  - behaviours: `IceHoleActivate`, `IceHoleBeheadPygmy`, `IceHoleEatPygmy`, `IceHoleEnterIsland`, `IceHoleExitIsland`, `IceHoleFishFall`, `IceHoleFishingCatchFish`, `IceHoleFishingInit`, `IceHoleIceSquidEnter`, `IceHoleIceSquidExit`, `IceHoleIceSquidGrabPygmy`, `IceHoleIceSquidTentacle`, `IceHoleInit`, `IceHolePygmyFall`, `IceHolePygmyFallBackward`, `IceHolePygmyFallForward`
  - models: `Icicle`, `Tentacle`
  - markers: `LavaDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `iceHoleActivateComplete:::` @ 0x100185710 (1 use), `iceHoleWaitForSwordFish:::` @ 0x1001857a0 (2 uses), `waitToShowIceSquid:::` @ 0x100185818 (7 uses), `waitToGrabPygmy:::` @ 0x100185960 (1 use), `tugOfWar:::` @ 0x100185b74 (1 use), `beheadPygmyComplete:::` @ 0x100185bfc (1 use), `eatPygmyComplete:::` @ 0x100185c28 (4 uses), `squidExitComplete:::` @ 0x100185c54 (1 use), `icicleRain:::` @ 0x100185cd0 (1 use).
- **Persistence**: none.
- **Behaviour**:
  - `meltIceHole:` @ 0x100184dbc binds the burning pygmy on channel 1 and moves the hole to its x (240 without pygmy); `iceHoleActivateComplete:::` marks it melted. `isFrozen` = not melted and not melting; fishing/falling need melted ∧ not melting ∧ nobody fishing ∧ no eruption ∧ no squid.
  - Ice fishing: `startFishingSitDown:` binds pygmy (ch 1) and pole (ch 2), `IceHoleFishingInit`; `iceHoleWaitForSwordFish:::` → `IceHoleFishingCatchFish`; `animCatchFish` moves the swordfish to the hole and `catch`es it, pygmy `catchSwordFish`.
  - Ice squid (`waitToShowIceSquid:::`, `waitToGrabPygmy:::`, `tugOfWar:::`; sequences.md §5): `onGameFrame:` @ 0x1001854f8 aims the tentacle at the dragged pygmy (`rot.z = atan2` in degrees, `scale.x = distance/56`); `animGrabPygmy` → `grabbedByTentacle:`; letting go makes the squid eat the pygmy (`IceHoleEatPygmy`). `enableIceSquid` / `disableIceSquid` are empty.
  - `enterIsland:` resets (frozen) and plays `IceHoleEnterIsland`; `exitIsland:` releases melting/fishing pygmies.

#### `IcicleLogic`

`SpriteLogic` · `IcicleLogic.c` · 3 methods + 2 item handlers · 5 225 B

- **Role**: one falling icicle of the ice-hole eruption.
- **Where**: ice island.
- **Created**:
  - allocated in `IceHoleLogic::icicleRain:::` @ 0x100185cd0
  - by `IceHoleLogic::icicleRain:::` (listener = the hole).
- **Listener**:
  - `<IcicleLogicListener>` (`mIcicleLogicListener`): `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onIcicleDie:`
  - set in `IceHoleLogic::icicleRain:::` @ 0x100185cd0
- **Talks to**: `PygmyLogic` (`hitByIcicle:`).
- **Data ids**:
  - behaviours: `IcicleDie`, `IcicleFall`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `fall:::` @ 0x1000ca288 (134 uses), `die:::` @ 0x1000ca614 (119 uses).
- **Persistence**: none.
- **Behaviour**: init enables gravity and plays `IcicleFall`; the `fall:::` handler keeps it on screen, hits a pygmy within 64 units (`hitByIcicle:` → `IcicleDie`) and lands on the island; `die:::` → `onIcicleDie:`.

### 4.7 Dinosaur (T-Rex)

The T-Rex encounter on the main island. The egg is a boulder-like throwable, and three hard bounces crack it so the baby calls its mother. Spears (`SpearLogic`) and the health bar are the fight tools.

#### `TRexEggLogic`

`SpriteLogic` · `TRexEggLogic.c` · 21 methods + 5 item handlers · 33 147 B (the file also holds `entry` (352 B), the program entry point; not counted)

- **Role**: the dinosaur egg: a boulder-like object that crushes and bowls pygmies; three hard bounces crack it and the baby calls its mother, the T-Rex.
- **Where**: main island (`IslandDisplayGroup/TRexEgg`, after `IslandBackDisplayMarker` when thrown).
- **Created**:
  - `setLogic` item(s) `logicId=TRexEggLogic` in `PocketGod`: `IslandDisplayGroup/TRexEgg`
  - stored in `mTRexEggLogic`, listener = view controller.
- **Listener**:
  - `<TRexEggLogicListener>` (`mTRexEggLogicListener`): `isStoryActive`, `isGravityEnabled`, `isTRexEggEnabled`, `isTRexEnabled`, `islandMinXPos`, `islandMaxXPos`, `tRexLogic:`, `pygmyLogicArray:`, `onTRexEggDrag:`, `onTRexEggDragComplete:`, `onPygmiesBowled:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `TRexLogic` (`callMom` via the listener's `tRexLogic:` in `callForMom:::`), `PygmyLogic` (crush/bowl like the boulder: `crushedByTRexEgg:`, `uncrushedByTRexEgg:dontWait:`, `hitByTRexEgg:xVel:`; `PygmyLogic(Story)` calls `clearPygmyLogic:`).
  - also sent in code: `PygmyLogic` (`charlieSmackTRexEgg:`, `personaPersonality`).
- **Called from**: `clearPygmyLogic:` ← PygmyLogic(Story); `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `TRexEggBounce`, `TRexEggBounceCrack1`, `TRexEggBounceCrack2`, `TRexEggBounceCrack3`, `TRexEggCallForMom`, `TRexEggDie`, `TRexEggDrag`, `TRexEggEnterIsland`, `TRexEggFall`, `TRexEggFallIgnoreIsland`, `TRexEggSplash`, `TRexEggStopSlide`, `TRexEggTiltSlide`
  - markers: `DragDisplayMarker`, `IslandBackDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `callForMom:::` @ 0x100172470 (2 uses), `slide:::` @ 0x1001724d0 (20 uses), `drag:::` @ 0x1001725d0 (40 uses), `fall:::` @ 0x100172770 (134 uses), `die:::` @ 0x1001728d4 (119 uses).
- **Persistence**: none.
- **Behaviour**: Shared pattern of §2.2 with slide friction 0.025, floor `y = 119`, the boulder's landing/rolling rules (bounce ×0.4/−0.4, settle below 30) and pygmy collision. A bounce with `vel.y > 100` (not calling) plays `TRexEggBounce`, or — after it has once settled since the last crack and while the T-Rex is enabled — the next crack `TRexEggBounceCrack1/2/3` (`mCrackCount`). Settling after the third crack → `TRexEggCallForMom` (its `callForMom:::` calls the T-Rex). Settling also reports `onPygmiesBowled:`.

#### `TRexLogic`

`SpriteLogic` · `TRexLogic.c` · 26 methods + 8 item handlers · 30 641 B

- **Role**: the T-Rex boss: called by its hatching egg, it jumps onto the island, hunts and eats pygmies, can be poked or tilted off, and is killed with thrown spears (health bar); the "T-Rex hunt" challenge game 7 changes its idle/walk choices.
- **Where**: main island (`IslandDisplayGroup/TRex`), starts at (500, 91) off the right edge.
- **Created**:
  - `setLogic` item(s) `logicId=TRexLogic` in `PocketGod`: `IslandDisplayGroup/TRex`
  - stored in `mTRexLogic`, listener = view controller; init plays `TRexEnterIsland` (it stays inactive until called).
- **Listener**:
  - `<TRexLogicListener>` (`mTRexLogicListener`): `isGravityEnabled`, `isTRexEnabled`, `isSpearEnabled`, `islandMinXPos`, `islandMaxXPos`, `healthBarLogic:`, `spearLogicArray:`, `pygmyLogicArray:`, `onTRexVisible:`, `onTRexHitBySpear:`, `onTRexKilledBySpear:`, `onTRexCancelHunt:`, `onAchievement:count:highScoreWins:` (never sent by this class: `islandMaxXPos`, `islandMinXPos`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `HealthBarLogic` (type 0; visible when spears and the T-Rex are enabled and it was called), `SpearLogic` (`canBeHitBySpear:` → YES, `hitBySpear:`, `throwXPos`; the listener's `spearLogicArray:`: `activate` in `callMom`, `deactivate` on death), `PygmyLogic` (eat: `eatenByTRex:`/`uneatenByTRex:`/`swallowedByTRex:`; `PygmyLogic(Story)` `clearPygmyLogic:`), `ChallengeManager` (`isGameActive: 7`), `TRexEggLogic` calls `callMom`.
  - also sent in code: `ChallengeManager` (`isGameActive:`); `HealthBarLogic` (`setHealthPercent:`); `PygmyLogic` (`timeTemplePygmyType`).
- **Called from**: `callMom` ← TRexEggLogic; `canBeHitBySpear:`, `hitBySpear:` ← SpearLogic; `clearPygmyLogic:` ← PygmyLogic(Story); `enterIsland:`, `exitIsland` ← PocketGodViewController, PocketGodViewController(Game); `isActive` ← PocketGodViewController(Game), SpearLogic.
- **Data ids**:
  - behaviours: `TRexDie`, `TRexEatPlaguePygmy`, `TRexEnterIsland`, `TRexFall`, `TRexHitAndKilledBySpear`, `TRexHitBySpear`, `TRexHitFromBehindAndKilledBySpear`, `TRexHitFromBehindBySpear`, `TRexHuntTurn`, `TRexHuntWalk`, `TRexPoke`, `TRexPokeExitIsland`, `TRexSwallowPygmy`, `TRexSwallowPygmyBaby`, `TRexTurn`, `TRexWalk`
  - markers: `TRexMarker`
  - achievements: `ConqueredTyrannosaurus`
  - other strings: `TRexGameIdle`, `TRexWanderIdle`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `jumpOntoIsland:::` @ 0x100180b98 (2 uses), `fall:::` @ 0x100180bf0 (134 uses), `eatPygmy:::` @ 0x100180ce8 (3 uses), `die:::` @ 0x100180e8c (119 uses), `chooseIdleSequence:::` @ 0x100181060 (0 uses), `whichWayDoIGo:::` @ 0x1001810d8 (5 uses), `walk:::` @ 0x100181230 (16 uses), `idle:::` @ 0x100181398 (17 uses).
- **Persistence**: none.
- **Behaviour**:
  - `callMom` @ 0x10017fccc (T-Rex enabled, not already called): health 100, active, placed at (500, 91), every spear `activate`d (listener `spearLogicArray:`), becomes the health bar's listener and activates it, selectable, `onTRexVisible:`, inserted after `TRexMarker`, `TRexEnterIsland` (its `jumpOntoIsland:::` item gives `vel = (−200, 800)`). `isActive` = called ∧ not killed.
  - `hitBySpear: s` @ 0x10017ff70: releases an eaten pygmy; damage `10·(1 + (300 − d)/300)` for throw distance `d = |s.throwXPos − x| ≤ 300`, else 10; bar updated, `onTRexHitBySpear:`; facing-dependent `TRexHitBySpear` / `TRexHitFromBehindBySpear`; at ≤ 0: killed, `onTRexKilledBySpear:`, bar off, achievement `ConqueredTyrannosaurus`, `TRexHitAndKilledBySpear` / `TRexHitFromBehindAndKilledBySpear`.
  - `eatPygmy:::` restores 10 health (max 100); a plague pygmy kills it.
  - Tap (not in challenge 7): releases its prey and pokes it: `x ≤ 340` → `vel = (200, 800)`, `TRexPoke`; else `vel = (400, 800)`, leaves (`TRexPokeExitIsland`, bar off). Tilting the device off (`refreshTiltState`) → `TRexFall`. The shared gravity/floor rules (§2.2) apply with the island side collision.
  - `exitIsland` while called: `onTRexCancelHunt:`, bar off, `TRexDie`.

### 4.8 Graveyard island

Graveyard island (state 3): the spider boss, the web tree whose babies call it, and the zombie grave.

#### `SpiderLogic`

`SpriteLogic` · `SpiderLogic.c` · 26 methods + 7 item handlers · 28 514 B

- **Role**: the giant spider boss of the graveyard island: the spider tree's babies call it; it hunts and eats pygmies and is killed with spears (health bar); the "spider" challenge game 8 changes its touch handling.
- **Where**: graveyard island (location 3), `IslandDisplayGroup/Spider`.
- **Created**:
  - `setLogic` item(s) `logicId=SpiderLogic` in `PocketGod`: `IslandDisplayGroup/Spider`
  - stored in `mSpiderLogic`, listener = view controller.
- **Listener**:
  - `<SpiderLogicListener>` (`mSpiderLogicListener`): `isGravityEnabled`, `isSpiderEnabled`, `isSpearEnabled`, `islandMinXPos`, `islandMaxXPos`, `healthBarLogic:`, `spearLogicArray:`, `pygmyLogicArray:`, `onSpiderVisible:`, `onSpiderHitBySpear:`, `onSpiderKilledBySpear:`, `onSpiderCancelHunt:`, `onAchievement:count:highScoreWins:` (never sent by this class: `islandMaxXPos`, `islandMinXPos`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `HealthBarLogic` (type 1), `SpearLogic` (`canBeHitBySpear:` → YES, `hitBySpear:`, `throwXPos`, the listener's `spearLogicArray:`), `PygmyLogic` (`eatenBySpider:`, `uneatenBySpider:`, `swallowedBySpider:`; `PygmyLogic(Story)` `clearPygmyLogic:`), `ChallengeManager` (`isGameActive: 8`), `SpiderTreeLogic` calls `callMom`.
  - also sent in code: `ChallengeManager` (`isGameActive:`); `HealthBarLogic` (`activate`, `deactivate`, `setHealthPercent:`); `SpearLogic` (`activate`, `deactivate`).
- **Called from**: `callMom` ← SpiderTreeLogic; `canBeHitBySpear:`, `hitBySpear:` ← SpearLogic; `clearPygmyLogic:` ← PygmyLogic(Story); `enterIsland:`, `exitIsland` ← PocketGodViewController, PocketGodViewController(Game); `isActive` ← PocketGodViewController(Game), SpearLogic.
- **Data ids**:
  - behaviours: `SpiderDie`, `SpiderEnterIsland`, `SpiderFall`, `SpiderHitAndKilledBySpear`, `SpiderHitBySpear`, `SpiderHitFromBehindAndKilledBySpear`, `SpiderHitFromBehindBySpear`, `SpiderPoke`, `SpiderPokeExitIsland`, `SpiderSwallowPygmy`, `SpiderTurn`, `SpiderWalk`
  - markers: `TRexMarker`
  - achievements: `ConqueredSpider`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `jumpOntoIsland:::` @ 0x10019b088 (2 uses), `fall:::` @ 0x10019b0e0 (134 uses), `eatPygmy:::` @ 0x10019b1d8 (3 uses), `die:::` @ 0x10019b2a8 (119 uses), `whichWayDoIGo:::` @ 0x10019b47c (5 uses), `walk:::` @ 0x10019b55c (16 uses), `idle:::` @ 0x10019b6c4 (17 uses).
- **Persistence**: none.
- **Behaviour**: the T-Rex's twin (same structure, constants and spear damage formula `10·(1 + (300 − d)/300)` for `d ≤ 300`; `SpiderJumpOntoIsland`… handlers, `SpiderFall` on tilt, `SpiderDie` on exit, `onSpiderHitBySpear:`/`onSpiderKilledBySpear:`/`onSpiderCancelHunt:` to the listener, achievement for killing it in `hitBySpear:` @ 0x10019a488). Pokes: `x ≤ 340` jumps back on, else leaves.

#### `SpiderTreeLogic`

`SpriteLogic` · `SpiderTreeLogic.c` · 17 methods + 3 item handlers · 7 505 B (the file also holds 6 free functions (about 4.9 KB) of `BCSound` audio-unit helpers; not counted)

- **Role**: the web-covered tree of the graveyard island: a pygmy thrown into it is trapped in the web; tapping cocoons it, tapping again lets the little spiders eat it; the spiders also call their mother.
- **Where**: graveyard island (`GraveyardIslandDisplayGroup/Tree`).
- **Created**:
  - `setLogic` item(s) `logicId=SpiderTreeLogic` in `PocketGod`: `GraveyardIslandDisplayGroup/Tree`
  - stored in `mSpiderTreeLogic`, listener = view controller; init stops its game frame, unselectable, `SpiderTreeInit`.
- **Listener**:
  - `<SpiderTreeLogicListener>` (`mSpiderTreeLogicListener`): `isSpiderEnabled`, `spiderLogic:`, `pygmyLogicArray:` (never sent by this class: `pygmyLogicArray:`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic(Island)` (`trapInWeb:`, `isAvailable`; tree sends `ejectFromSpiderTree`, `eatenByLittleSpiders:`; `PygmyLogic` calls `ejectFromWeb:`), `SpiderLogic` (`callMom` via `spiderLogic:` in `callForMom:::`).
- **Called from**: `ejectFromWeb:` ← PygmyLogic; `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `isAvailable`, `trapInWeb:` ← PygmyLogic(Island).
- **Data ids**:
  - behaviours: `SpiderTreeCatchPygmy`, `SpiderTreeCocoonPygmy`, `SpiderTreeEatPygmy`, `SpiderTreeEnterIsland`, `SpiderTreeExitIsland`, `SpiderTreeInit`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `onTransitionComplete:::` @ 0x100068a30 (1 use), `dieFromSpiders:::` @ 0x100068a44 (1 use), `callForMom:::` @ 0x100068ab8 (2 uses).
- **Persistence**: none.
- **Behaviour**: `trapInWeb:` binds the pygmy on channel 1, state 1, selectable, `SpiderTreeCatchPygmy`. Tap (no transition running): state 1 → 2 `SpiderTreeCocoonPygmy`; state 2 → 3 `SpiderTreeEatPygmy` (then `dieFromSpiders:::`). `ejectFromWeb:` resets to `SpiderTreeInit`. `enterIsland:` → `SpiderTreeEnterIsland`.

#### `ZombieGraveLogic`

`SpriteLogic` · `ZombieGraveLogic.c` · 20 methods + 1 item handler · 10 746 B

- **Role**: the open grave of the graveyard island: tap to dig (twice), pygmies trip or fall in, and burying a pygmy turns it into a zombie.
- **Where**: graveyard island (`GraveyardIslandDisplayGroup/Grave`).
- **Created**:
  - `setLogic` item(s) `logicId=ZombieGraveLogic` in `PocketGod`: `GraveyardIslandDisplayGroup/Grave`
  - stored in `mZombieGraveLogic`, listener = view controller; init stops its game frame, unselectable, `ZombieGraveExitIsland`.
- **Listener**:
  - `<ZombieGraveLogicListener>` (`mZombieGraveLogicListener`): `pygmyLogicArray:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic(Island)` (`isOpenAndUnoccupied`, `isPygmyAboutToBecomeZombie`, `loseBalanceByGrave:`, `tripIntoGrave:fallForward:`, `fallIntoGrave:`, `ejectFromGrave:`; grave sends `startZombieIdle`, `ejectFromZombieGrave`, `zombieExplode`).
- **Called from**: `ejectFromGrave:` ← PygmyLogic, PygmyLogic(Island); `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `fallIntoGrave:`, `isOpenAndUnoccupied`, `isPygmyAboutToBecomeZombie`, `loseBalanceByGrave:`, `tripIntoGrave:fallForward:` ← PygmyLogic(Island).
- **Data ids**:
  - behaviours: `ZombieGraveBury1`, `ZombieGraveBury2`, `ZombieGraveBuryPygmy`, `ZombieGraveDig1`, `ZombieGraveDig2`, `ZombieGraveEjectPygmy`, `ZombieGraveEnterIsland`, `ZombieGraveExitIsland`, `ZombieGravePygmyEmerge`, `ZombieGravePygmyFall`, `ZombieGravePygmyFallBackward`, `ZombieGravePygmyFallForward`, `ZombieGraveUnburyPygmy`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `startZombieIdle:::` @ 0x1000c488c (1 use).
- **Persistence**: none.
- **Behaviour**: dig count 0–2 (`isOpenAndUnoccupied` = count 2 and empty). `touchBeganInside:allTouches:` @ 0x1000c48fc: a tap within 30 px of the grave's mound `(x − 35, y + 20)` buries (count 2 → `ZombieGraveBury2`, count 1 → `ZombieGraveBury1` or burying the trapped pygmy), any other tap digs (`ZombieGraveDig1`, `ZombieGraveDig2` or `ZombieGraveUnburyPygmy`), stepping the count. A pygmy fallen in is bound on channel 1 (`ZombieGravePygmyFall`, `…FallForward/Backward`); `startZombieIdle:::` releases it as a zombie. `exitIsland: now` also makes every pygmy `zombieExplode`.

### 4.9 Misfit island

Misfit island (state 6) and the dream bubble that leads there. The Moron Pests snake that also lives on Misfit island is catalogued with its mini-game in section 4.14.

#### `IslandObjectLogic`

`SpriteLogic` · `IslandObjectLogic.c` · 4 methods · 1 603 B

- **Role**: a static object that joins a display marker's group when its island is entered (framework class, `docs/re/logic.md` §11.3).
- **Where**: Misfit island: entry 49, the `MisfitOceanRock` multimodel (animation `MisfitOceanRockInitAnim`, pos (400, 98, −1200), scale 3), inserted before `MidOceanMarker`.
- **Created**:
  - `createLogicClasses` table entry 49 (#49 new `BCMultiModel` `MisfitOceanRock`); init steps: `play_animation MisfitOceanRockInitAnim`, `set_scale (3, 3)`
  - `initWithDisplayObject:displayMarkerId:` @ 0x100125510 (the only table entry using it).
- **Listener**: none.
- **Called from**: `enterIsland:`, `exitIsland` ← PocketGodViewController; `initWithDisplayObject:displayMarkerId:` ← BCLibrary(Logic).
- **Persistence**: none.
- **Behaviour**: init stops its game frame and removes the object; `enterIsland:` inserts it before its marker; `exitIsland` removes it.

#### `DreamBubbleLogic`

`SpriteLogic` · `DreamBubbleLogic.c` · 16 methods + 3 item handlers · 7 439 B

- **Role**: the dream bubble over a sleeping pygmy; tapping it takes the player to Misfit island (location 6).
- **Where**: surface islands; entry 2 (`DreamBubble` multimodel, box (0,40)–(60,90), selectable).
- **Created**: `createLogicClasses` table entry 2 (#2 new `BCMultiModel` `DreamBubble`)
- **Listener**:
  - `<DreamBubbleLogicListener>` (`mDreamBubbleLogicListener`): `islandLocationState`, `onDreamBubbleChangeIslandLocationState:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `PygmyLogic` (`dreamBubbleSelected`; pygmies call `activate:`/`deactivate`).
- **Called from**: `activate:`, `isActive` ← PygmyLogic(Island); `deactivate` ← PygmyLogic(Island), PygmyLogic(KonkeyDong).
- **Data ids**:
  - behaviours: `DreamBubbleMisfitIslandActivate`, `DreamBubbleMisfitIslandDeactivate`, `DreamBubbleSelected`
  - markers: `MapDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `die:::` @ 0x10019f3fc (119 uses), `transitionComplete:::` @ 0x10019f540 (1 use), `deactivateComplete:::` @ 0x10019f58c (1 use).
- **Persistence**: none.
- **Behaviour**: `activate: pygmy` @ 0x10019f404 inserts the bubble before the pygmy's display object at its position, `DreamBubbleMisfitIslandActivate`; `onGameFrame:` keeps it on the pygmy. A tap (active, not transitioning, not already on location 6) tells the pygmy `dreamBubbleSelected`, inserts the bubble after `MapDisplayMarker` at x 0, `DreamBubbleSelected`; its animation calls `animChangeLocations` → listener `onDreamBubbleChangeIslandLocationState: 6`. `deactivate` → `DreamBubbleMisfitIslandDeactivate`.

### 4.10 Ooga Jump mini-game

Ooga Jump, a vertical jumping mini-game played in the sky. The pygmy side lives in `PygmyLogic(OogaJump)`. Only the platform clouds are separate logic objects. The three background sky clouds are `CloudLogic` indices 1 to 3.

#### `DoodleCloudLogic`

`SpriteLogic` · `DoodleCloudLogic.c` · 6 methods · 10 296 B

- **Role**: one platform cloud of the Ooga Jump mini-game (a vertical jumping game): normal, electric, snow, wispy (dissolves), shooting (knocks the pygmy down) and jet-pack clouds.
- **Where**: Ooga Jump (doodle) sky.
- **Created**:
  - allocated in `PocketGodViewController(Game)::createDoodleClouds` @ 0x10015ee34
  - by `PocketGodViewController(Game)::createDoodleClouds` @ 0x10015ee34 (on `onPygmyStartDoodleJump:`), listener = view controller (protocol empty).
- **Listener**:
  - `<DoodleCloudLogicListener>` (`mDoodleCloudLogicListener`): no methods
  - set in `PocketGodViewController(Game)::createDoodleClouds` @ 0x10015ee34
- **Talks to**: `PygmyLogic(OogaJump)` (calls `checkCollision:lastPos:` and receives `doodleBounce:`, `doodleElectricBounce:`, `doodleSnowBounce:`, `doodleKnockedDown:`, `doodleJetPack:`), `BonusPackManager` (`isBonusPackUnlocked: ComicPreOrder` gates jet packs).
  - also sent in code: `BonusPackManager` (`isBonusPackUnlocked:`).
- **Called from**: `checkCollision:lastPos:` ← PygmyLogic(KonkeyDong), PygmyLogic(OogaJump), PygmyLogic(TheRuns), PygmyLogic(Underwater); `moveToTop` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `DoodleCloudElectricInit`, `DoodleCloudInit`, `DoodleCloudShootInit`, `DoodleCloudSnowCloudInit`, `DoodleJetPack`, `DoodleJetPackCatch`, `DoodleWispyCloudDissolve`, `DoodleWispyCloudInit`
  - other strings: `ComicPreOrder`
- **Persistence**: none.
- **Behaviour**:
  - Generator state in globals: `DAT_1007e7bec` next cloud height (600 at the first cloud), `DAT_1007e7bf4` / `DAT_1007e7bf8` min/max gap (22/22 initially), `DAT_1007e7bf0` previous type (4 initially), `DAT_1007e7be8` last shooter height, `DAT_1007e7be4` last jet-pack height, `DAT_1007e7be0` jet pack pending flag, `DAT_1007e7c00` previous cloud.
  - `moveToTop` @ 0x1000ccab0: `x = (rand%40501)/100`, `z = −277.12`, scale 0.5. When the previous type was 0 (and a previous cloud exists) the type is rolled: 1/11 wispy (3); else, if the max gap is < 80 and another 1/11 → snow (2); else 1/11 electric (1); else shooter (4) when max gap > 36.8, the previous cloud's x > 180, a 1/4 roll hits and the running height is more than 960 above the last shooter; else jet pack (5, scale 1) when `ComicPreOrder` is unlocked, a 1/5 roll hits, max gap > 28.75 and the height gained since the last jet pack `d` exceeds 1280, or 3200 while the pending flag `DAT_1007e7be0` is set (the code compares `(d > 1280 ∧ ¬pending) == (d ≤ 3200 ∨ ¬pending)`, which reduces to `d > (pending ? 3200 : 1280)`; checked in the disassembly at 0x1000ccd18); a jet pack clears the pending flag, gets scale 1 and adds its gap twice to the running height. Heights: types 0–2 and 5 add a gap random in `[min, max]` (0.01 steps, ≤ 110) to the running height (jet pack: twice); wispy (only when `min > 44`, else treated as type 0) sits a random `[22, min − 22)` (0.01 steps) above the running height without advancing it; the shooter sits at x 0 at the running height (remembered as the last shooter height); the jet pack's x is random (0.01 steps) in `[prevX + 125, 455]` when the previous cloud's `prevX ≤ 240`, else in `[25, prevX − 50)`, and its height is remembered as the last jet-pack height. Behaviours `DoodleCloudInit`, `DoodleCloudElectricInit`, `DoodleCloudSnowCloudInit`, `DoodleWispyCloudInit`, `DoodleCloudShootInit`, `DoodleJetPack`. Afterwards `max += 0.1`, and `min += 0.1` when `max − min > 20`.
  - `checkCollision:lastPos:` @ 0x1000cd130: shooter — the pygmy within 50 px of the cloud's current frame point (+10) → `doodleKnockedDown:`; jet pack — within 25 px of `(x, y + 20)` → `DoodleJetPackCatch` + `doodleJetPack:`; otherwise a pygmy whose x is within (x + 5, x + 70) passing downward through `y + 10.5` lands: `doodleBounce:` / `doodleElectricBounce:` / `doodleSnowBounce:`, or the wispy cloud dissolves (`DoodleWispyCloudDissolve`).

### 4.11 Ape island

Ape island (state 5): the summoning ritual (torches, altar, wench, bongo, ape), the monkeys, and the banana tree with its banana.

#### `ApeLogic`

`SpriteLogic` · `ApeLogic.c` · 12 methods + 6 item handlers · 11 835 B

- **Role**: the giant ape of Ape island: summoned by the full ritual (both torches lit, a pygmy stretched on the altar, a pygmy drumming the bongo, then a dance), it eats the altar pygmy and smashes the bongo and wench pygmies; a plague pygmy makes it vomit.
- **Where**: Ape island (location 5), `ApeIslandDisplayGroup/Ape`.
- **Created**:
  - `setLogic` item(s) `logicId=ApeLogic` in `PocketGod`: `ApeIslandDisplayGroup/Ape`
  - stored in `mApeLogic`, listener = view controller; init stops its game frame, unselectable.
- **Listener**:
  - `<ApeLogicListener>` (`mApeLogicListener`): `isApeEnabled`, `apeAltarLogic:`, `apeBongoLogic:`, `apeWenchLogic:`, `apeTorchLogicArray:`, `onApeVisible:` (never sent by this class: `isApeEnabled`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `ApeAltarLogic` (`isPygmyAvailableForApe`, `isAvailable`, `apeGrabPygmy`, `altarPygmyLogic`), `ApeBongoLogic` (`isPlaying`, `apeHitApeBongo`), `ApeWenchLogic` (`apeHitApeWench`), `ApeTorchLogic` (`isLit`; a doused torch calls `onDouseTorch`), all through the listener (`apeAltarLogic:`, `apeBongoLogic:`, `apeWenchLogic:`, `apeTorchLogicArray:`); entry 1 (`ApeVomit`, set to `ApeVomitInit` by `checkForPlaguePygmy:::`).
  - `getLogic:` 1 `MultiModelLogic`.
- **Called from**: `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `onDouseTorch` ← ApeTorchLogic; `startTapping`, `stopTapping` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `ApeCrushBongoAndWenchPygmies`, `ApeDanceStart`, `ApeEatPlaguePygmy`, `ApeEatPygmy`, `ApeEnterIsland`, `ApeExitIsland`, `ApeIdle`, `ApeLickChops`, `ApeVomitInit`
  - markers: `DragDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `enterIslandComplete:::` @ 0x100088f14 (2 uses), `checkForPlaguePygmy:::` @ 0x100088f6c (5 uses), `checkEatPygmy:::` @ 0x1000890c0 (2 uses), `apeExitAfterIdle:::` @ 0x100089198 (1 use), `checkForLickChops:::` @ 0x1000891d0 (6 uses), `lickChopsWaitForPygmy:::` @ 0x100089278 (1 use).
- **Persistence**: none.
- **Behaviour**: `startTapping` @ 0x100088950 (sent by `(Game)::startCelebration`): if not visible and every torch is lit, the altar pygmy is available and the bongo is playing → visible, `ApeEnterIsland`; when visible and not entering → dancing, `ApeDanceStart`. `stopTapping` ends the dance: `ApeEatPygmy` (altar pygmy ready), `ApeIdle`, or `ApeLickChops` (altar busy). `onDouseTorch` / `exitIsland:` send it away (`ApeExitIsland`, deferred while entering). Animation callbacks `animGrabPygmy` (altar `apeGrabPygmy` or `ApeCrushBongoAndWenchPygmies`), `animHitWench`, `animHitBongo`. `enterIsland:` reports `onApeVisible:`.

#### `ApeAltarLogic`

`SpriteLogic` · `ApeAltarLogic.c` · 18 methods + 3 item handlers · 22 996 B

- **Role**: the sacrificial altar: a pygmy dropped on it is stretched by cranking the wench until the ape takes it or it is torn apart.
- **Where**: Ape island (`ApeIslandDisplayGroup/Alter` [sic]).
- **Created**:
  - `setLogic` item(s) `logicId=ApeAltarLogic` in `PocketGod`: `ApeIslandDisplayGroup/Alter`
  - stored in `mApeAltarLogic`, listener = view controller. Init sets the apocalypse cloud style `ApeAltar` and creates five multimodels with `MultiModelLogic`s: `ApeAltarPygmyHead` and the four limbs (hands, legs) used for the tear-apart.
- **Listener**:
  - `<ApeAltarLogicListener>` (`mApeAltarLogicListener`): `apeWenchLogic:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `ApeWenchLogic` (`retractApeWench`, via `apeWenchLogic:`; the wench calls `setApeAltarDeltaStretch:`, `canStretch`, `isAtlarFullyRetracted` [sic], `retractApeAltar`, `stopStretchingApeAltar`), `ApeLogic` (`apeGrabPygmy`, `isPygmyAvailableForApe`, `altarPygmyLogic`), `PygmyLogic(Island)` (`pygmyFallIntoApeAltar:`, `removePygmyFromApeAltar:`; altar sends `apeOwnsPygmy`, `tornApartFromApeAltar`, `grabbedFromApeAltar`), `IdolManager` (`WindPygmyOnAltar` episode event).
  - also sent in code: `IdolManager` (`triggerEpisodeEvent:`); `PygmyLogic` (`tossFromApeAltar`).
- **Called from**: `altarPygmyLogic`, `apeGrabPygmy`, `isPygmyAvailableForApe` ← ApeLogic; `canStretch`, `isAtlarFullyRetracted`, `retractApeAltar`, `setApeAltarDeltaStretch:`, `stopStretchingApeAltar` ← ApeWenchLogic; `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `isAvailable` ← ApeLogic, PygmyLogic(Island); `pygmyFallIntoApeAltar:` ← PygmyLogic(Island); `removePygmyFromApeAltar:` ← PygmyLogic, PygmyLogic(Charlie), PygmyLogic(Island), PygmyLogic(Story).
- **Data ids**:
  - behaviours: `ApeAltarApeGrabPygmy`, `ApeAltarBreakPygmy`, `ApeAltarEnterIsland`, `ApeAltarExitIsland`, `ApeAltarPygmyFallInside`, `ApeAltarPygmyHeadIdle`, `ApeAltarPygmyHeadPanic`, `ApeAltarPygmySpinningArm`, `ApeAltarRetract`, `ApeAltarRetractNoPygmy`, `ApeAltarRetractNoPygmySnap`, `ApeAltarStretch`, `BlankAnim`
  - models: `ApeAltarPygmyHead`, `ApeAltarPygmyLeftHand`, `ApeAltarPygmyLeftLeg`, `ApeAltarPygmyRightHand`, `ApeAltarPygmyRightLeg`
  - markers: `DragDisplayMarker`
  - episode events: `WindPygmyOnAltar`
  - other strings: `ApeAltar`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `breakPygmyComplete:::` @ 0x10017e4f4 (2 uses), `pygmyFallInsideComplete:::` @ 0x10017e508 (5 uses), `apeGrabPygmyComplete:::` @ 0x10017e540 (1 use).
- **Persistence**: none.
- **Behaviour**: `setApeAltarDeltaStretch: d` @ 0x10017d438: `stretch = max(0, stretch − 0.03·d)` (the episode event fires while `stretch > 1`), `ApeAltarStretch` on the first call, head `ApeAltarPygmyHeadPanic` once stretched; `stretch > 1.2` tears the pygmy apart: wench retracted, channel 1 cleared, `tornApartFromApeAltar`, the four limb models are placed at altar + (52, 107), (−52, 107), (21, 42), (−21, 42) after `DragDisplayMarker` and fly with `vel = ((rand%4001 − 2000)/10, 500)` (then falling under the altar's frame update). `isPygmyAvailableForApe` = pygmy ∧ `stretch ≥ 1`; `isAvailable` = empty ∧ not retracting ∧ not breaking. The pygmy is bound on channel 1 (`ApeAltarPygmyFallInside`).

#### `ApeBongoLogic`

`SpriteLogic` · `ApeBongoLogic.c` · 13 methods + 2 item handlers · 6 590 B

- **Role**: the bongo drum: a pygmy inside drums it during dances; the ape smashes it.
- **Where**: Ape island (`ApeIslandDisplayGroup/Bongo`).
- **Created**:
  - `setLogic` item(s) `logicId=ApeBongoLogic` in `PocketGod`: `ApeIslandDisplayGroup/Bongo`
  - stored in `mApeBongoLogic`, listener = view controller (protocol empty); init: apocalypse cloud style `ApeBongo`, `ApeBongoExitIsland`.
- **Listener**:
  - `<ApeBongoLogicListener>` (`mApeBongoLogicListener`): no methods
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic(Island)` (`pygmyFallIntoApeBongo:`, `removePygmyFromApeBongo:`; bongo sends `apeOwnsPygmy`, `apeSmashOnApeBongo`, `tossFromApeBongo`), `ApeLogic` (`isPlaying`, `apeHitApeBongo`).
- **Called from**: `apeHitApeBongo`, `isPlaying` ← ApeLogic; `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `isAvailable`, `pygmyFallIntoApeBongo:` ← PygmyLogic(Island); `removePygmyFromApeBongo:` ← PygmyLogic, PygmyLogic(Charlie), PygmyLogic(Island), PygmyLogic(Story); `startPlaying`, `stopPlaying` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `ApeBongoApeHit`, `ApeBongoApeHitPygmy`, `ApeBongoEnterIsland`, `ApeBongoExitIsland`, `ApeBongoPygmyFallInside`, `ApeBongoPygmyStart`, `ApeBongoPygmyStop`
  - other strings: `ApeBongo`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `pygmyFallInsideComplete:::` @ 0x100137540 (5 uses), `apeHitComplete:::` @ 0x10013758c (4 uses).
- **Persistence**: none.
- **Behaviour**: pygmy bound on channel 1 (`ApeBongoPygmyFallInside`, drumming allowed after `pygmyFallInsideComplete:::`). `startPlaying`/`stopPlaying` (from the view controller's dances) → `ApeBongoPygmyStart`/`Stop`; `isPlaying` = playing ∧ can drum. `apeHitApeBongo` → `ApeBongoApeHit` / `ApeBongoApeHitPygmy`.

#### `ApeWenchLogic`

`SpriteLogic` · `ApeWenchLogic.c` · 14 methods + 4 item handlers · 12 506 B

- **Role**: the wench (winch) next to the altar: a pygmy hangs on it and the player cranks it with circular finger motions to stretch the altar pygmy.
- **Where**: Ape island (`ApeIslandDisplayGroup/Wench`).
- **Created**:
  - `setLogic` item(s) `logicId=ApeWenchLogic` in `PocketGod`: `ApeIslandDisplayGroup/Wench`
  - stored in `mApeWenchLogic`, listener = view controller; init: apocalypse cloud style `ApeWench`, `ApeWenchExitIsland`.
- **Listener**:
  - `<ApeWenchLogicListener>` (`mApeWenchLogicListener`): `apeAltarLogic:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `ApeAltarLogic` (via `apeAltarLogic:`), `PygmyLogic(Island)` (`pygmyFallIntoApeWench:`, `removePygmyFromApeWench:`; wench sends `apeOwnsPygmy`, `apeSmashOnApeWench`, `tossFromApeWench`), `ApeLogic` (`apeHitApeWench`); the view controller asks `canCrankWench` and drives `setApeWenchDeltaRot:` / `stopTurningApeWench`.
  - also sent in code: `ApeAltarLogic` (`canStretch`, `isAtlarFullyRetracted`, `retractApeAltar`, `setApeAltarDeltaStretch:`, `stopStretchingApeAltar`).
- **Called from**: `apeHitApeWench` ← ApeLogic; `canCrankWench`, `exitIsland:` ← PocketGodViewController; `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `isAvailable`, `pygmyFallIntoApeWench:` ← PygmyLogic(Island); `removePygmyFromApeWench:` ← PygmyLogic, PygmyLogic(Charlie), PygmyLogic(Island), PygmyLogic(Story); `retractApeWench` ← ApeAltarLogic; `setApeWenchDeltaRot:`, `stopTurningApeWench` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `ApeWenchApeHit`, `ApeWenchApeHitPygmy`, `ApeWenchEnterIsland`, `ApeWenchExitIsland`, `ApeWenchIdle`, `ApeWenchPygmyFallInside`, `ApeWenchRetract`, `ApeWenchRetractNoPygmy`, `ApeWenchSoundOff`, `ApeWenchSoundOn`, `ApeWenchStruggle`, `ApeWenchTurn`
  - other strings: `ApeWench`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `pygmyFallInsideComplete:::` @ 0x10008a848 (5 uses), `retractComplete:::` @ 0x10008a860 (2 uses), `struggleComplete:::` @ 0x10008a884 (1 use), `apeHitComplete:::` @ 0x10008a898 (4 uses).
- **Persistence**: none.
- **Behaviour**: `setApeWenchDeltaRot:` @ 0x10008a0b0 (crankable, not struggling) accumulates `mApeWenchRot`, forwards the delta to the altar (`setApeAltarDeltaStretch:`) or makes the hanging pygmy struggle when the altar is empty; `onGameFrame:` @ 0x10008a724 eases the crank to the nearest whole turn after release (`ApeWenchIdle`), with `ApeWenchSoundOn` while turning. `retractApeWench` → `ApeWenchRetract`; `apeHitApeWench` → `ApeWenchApeHit` (or retracts the altar when cranked).

#### `ApeTorchLogic`

`SpriteLogic` · `ApeTorchLogic.c` · 21 methods · 7 402 B

- **Role**: the two torches of Ape island: lit by lightning, doused by a tap; monkeys climb unlit torches.
- **Where**: Ape island (`ApeIslandDisplayGroup/RightTorch`, `/LeftTorch`).
- **Created**:
  - `setLogic` item(s) `logicId=ApeTorchLogic` in `PocketGod`: `ApeIslandDisplayGroup/RightTorch`, `ApeIslandDisplayGroup/LeftTorch`
  - added to `mApeTorchLogicArray`, listener = view controller; init stops its game frame and plays `ApeTorchEnterIsland`.
- **Listener**:
  - `<ApeTorchLogicListener>` (`mApeTorchLogicListener`): `islandCenterXPos`, `apeLogic:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `MonkeyLogic` (`isAvailableForMonkey`, `startMonkeyClimb:`, `monkeyLand:`, `monkeyJumpOffTorch:`; torch sends `apeTorchClimbComplete`, `bootedFromApeTorch`, `bootedFromApeTorchByFire`; the climbing monkey is bound on channel 2), `ApeLogic` (`onDouseTorch` via `apeLogic:`).
- **Called from**: `enterIsland:`, `hitByLightning` ← PocketGodViewController(Game); `isAvailableForMonkey`, `monkeyJumpOffTorch:`, `monkeyLand:`, `startMonkeyClimb:` ← MonkeyLogic; `isLit` ← ApeLogic, MonkeyLogic.
- **Data ids**:
  - behaviours: `ApeTorchDouse`, `ApeTorchEnterIsland`, `ApeTorchExitIsland`, `ApeTorchLit`, `ApeTorchMonkeyClimbLeft`, `ApeTorchMonkeyClimbRight`, `ApeTorchMonkeyLandLeft`, `ApeTorchMonkeyLandRight`
- **Persistence**: none.
- **Behaviour**: `hitByLightning` @ 0x10001f8fc: unlit → lit, selectable, `ApeTorchLit`, a perched monkey is burnt off; returns YES. Tap while lit → unlit, ape `onDouseTorch`, `ApeTorchDouse`. Monkey animations are mirrored by the torch's side of the island (`…Right` when left of the centre).

#### `MonkeyLogic`

`SpriteLogic` · `MonkeyLogic.c` · 56 methods + 14 item handlers · 77 358 B

- **Role**: the three monkeys of Ape island: they wander, eat bananas (leaving peels), climb the banana tree and the torches, play catch by throwing pygmies to each other, hump sleeping pygmies, dance, ride Charlie, and can be dragged, thrown, struck by lightning or carried off by the hurricane.
- **Where**: `IslandDisplayGroup/Monkey1…3` on Ape island.
- **Created**:
  - `setLogic` item(s) `logicId=MonkeyLogic` in `PocketGod`: `IslandDisplayGroup/Monkey1`, `IslandDisplayGroup/Monkey2`, `IslandDisplayGroup/Monkey3`
  - added to `mMonkeyLogicArray`, listener = view controller.
- **Listener**:
  - `<MonkeyLogicListener>` (`mMonkeyLogicListener`): `isDanceActive`, `isGravityEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `apeTorchLogicArray:`, `pygmyLogicArray:`, `monkeyLogicArray:`, `bananaLogic:`, `bananaTreeLogic:`, `onMonkeyDrag:`, `onMonkeyDragComplete:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `BananaLogic` (`isEdible`, `startEatingByMonkey:`, `finishEatingByMonkey`), `BananaTreeLogic` / `ApeTorchLogic` (perches: `isAvailableForMonkey`, `startMonkeyClimb:`, `monkeyLand:`, `monkeyJumpOff…:`), other monkeys (`monkeyLogicArray:`: `findMonkeyToTossPygmyTo`, `startPlayingCatch`, `waitForPygmyToss:`), `PygmyLogic` (`canBeChasedByMonkey:`, `canBeCaughtByMonkey:`, `caughtByMonkey:`, `thrownByMonkey:toMonkey:`, `playingCatchWithMonkey`, `canBeHumpedByMonkey:`, `humpedByMonkey:`, `unhumpedByMonkey:`, `monkeyOnBack:`, `monkeyBootedOffCharliesBack:`; pygmies call `canChasePygmy`, `startPlayingCatch`, `stopPlayingCatch`, `stopHumpingPygmy`, `punchedWhileHumping`, `pygmyIsEatingBanana:`, Charlie `bootedOffCharliesBack:`), a helper multimodel (`mHelperLogic`, `animMonkeyHelperBlank`).
  - also sent in code: `ApeTorchLogic` (`isLit`, `monkeyJumpOffTorch:`); `BananaLogic` (`dropWhileEating`); `BananaTreeLogic` (`monkeyJumpOffTree:`); `PygmyLogic` (`personaPersonality`).
- **Called from**: `apeTorchClimbComplete`, `bootedFromApeTorch`, `bootedFromApeTorchByFire` ← ApeTorchLogic; `bananaTreeClimbComplete`, `bootedFromBananaTree` ← BananaTreeLogic; `bootedOffCharliesBack:` ← PygmyLogic(Charlie); `canChasePygmy`, `punchedWhileHumping`, `pygmyIsEatingBanana:`, `startPlayingCatch`, `stopHumpingPygmy`, `stopPlayingCatch` ← PygmyLogic(Island); `enterIsland:`, `isActive` ← BananaTreeLogic, PocketGodViewController(Game); `exitIslandNow`, `hitByLightning`, `startDancing`, `stopDancing` ← PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `MonkeyCancelChasePygmy`, `MonkeyCatchPygmy`, `MonkeyCaughtPlaguePygmy`, `MonkeyChasePygmy`, `MonkeyChasePygmyTurn`, `MonkeyClimbApeTorch`, `MonkeyClimbApeTorchComplete`, `MonkeyClimbBananaTree`, `MonkeyClimbBananaTreeComplete`, `MonkeyDie`, `MonkeyDrag`, `MonkeyEatBanana`, `MonkeyEnterIsland`, `MonkeyFall`, `MonkeyFallIgnoreIsland`, `MonkeyHelperBlank`, `MonkeyHelperClimb`, `MonkeyHitByLightning`, `MonkeyHumpPygmy`, `MonkeyHumpPygmyPunch`, `MonkeyIdle`, `MonkeyJumpOnFire`, `MonkeyOnPygmyBack`, `MonkeyOnPygmyCharlieBack`, `MonkeyStartDancing`, `MonkeyStopDancing`, `MonkeyTouched`, `MonkeyTurn`, `MonkeyWaitForPygmyToss`, `MonkeyWalk`
  - models: `MonkeyHelper`
  - markers: `DragDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `checkForPlaguePygmy:::` @ 0x1000d0728 (5 uses), `checkForDance:::` @ 0x1000d07a0 (3 uses), `resetLastPos:::` @ 0x1000d07f4 (5 uses), `monkeyEnterIslandComplete:::` @ 0x1000d0844 (1 use), `whichWayDoIGo:::` @ 0x1000d08c0 (5 uses), `jumpOffTree:::` @ 0x1000d113c (1 use), `jumpOffTorch:::` @ 0x1000d17f8 (1 use), `finishEatingBanana:::` @ 0x1000d18c0 (2 uses), `chasePygmy:::` @ 0x1000d18c8 (1 use), `walk:::` @ 0x1000d1a9c (16 uses), `idle:::` @ 0x1000d1cf0 (17 uses), `drag:::` @ 0x1000d1e04 (40 uses), `fall:::` @ 0x1000d1fe4 (134 uses), `die:::` @ 0x1000d20e0 (119 uses).
- **Persistence**: none.
- **Behaviour**: walking/idle choices are handler-driven (`walk`, `idle`, `whichWayDoIGo`, `chasePygmy`, `jumpOffTree`, `jumpOffTorch`, `checkForDance`, `checkForPlaguePygmy`; sequences.md §5). `onGameFrame:` @ 0x1000d0264 applies the shared gravity/floor/side rules (§2.2) and the per-frame checks `checkForBanana`, `checkForSleepingPygmy`, `checkForBananaTree(DuringFall)`, `checkForApeTorch(DuringFall)`, `checkToCatchPygmy`, `checkPygmyCollision`, `checkTorchCollision`. `hitByLightning` sets it on fire; `startDancing`/`stopDancing` follow the island dance; `animThrowPygmy`, `animPickUpPygmy`, `animThrowPeel`, `animFallOffIsland` are animation callbacks. (77 KB: the largest gameplay class after the screens; see `MonkeyLogic.c`.)

#### `BananaTreeLogic`

`SpriteLogic` · `BananaTreeLogic.c` · 21 methods + 1 item handler · 11 367 B

- **Role**: the banana tree: a tap drops the banana; a monkey can perch in it.
- **Where**: Ape island (`ApeIslandDisplayGroup/BananaTree`, after `ApeIslandObjectsDisplayMarker`).
- **Created**:
  - `setLogic` item(s) `logicId=BananaTreeLogic` in `PocketGod`: `ApeIslandDisplayGroup/BananaTree`
  - stored in `mBananaTreeLogic`, listener = view controller.
- **Listener**:
  - `<BananaTreeLogicListener>` (`mBananaTreeLogicListener`): `isStoryActive`, `isBananaTreeEnabled`, `isMonkeyEnabled`, `bananaLogic:`, `monkeyLogicArray:` (never sent by this class: `isBananaTreeEnabled`)
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `BananaLogic` (via `bananaLogic:`: drops it when inactive), `MonkeyLogic` (perch protocol, channel 2; `bananaTreeClimbComplete`, `bootedFromBananaTree`).
  - also sent in code: `BananaLogic` (`enterIsland`, `isActive`, `touchBeganInside:allTouches:`); `IdolManager` (`triggerEpisodeEvent:`); `MonkeyLogic` (`enterIsland:`, `isActive`).
- **Called from**: `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `isAvailableForMonkey`, `monkeyJumpOffTree:`, `monkeyLand:`, `startMonkeyClimb:` ← MonkeyLogic.
- **Data ids**:
  - behaviours: `BananaTreeEnterIsland`, `BananaTreeExitIsland`, `BananaTreeMonkeyClimb`, `BananaTreeMonkeyLand`
  - markers: `ApeIslandObjectsDisplayMarker`
  - episode events: `GetThreeMonkeys`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `die:::` @ 0x100136ba4 (119 uses).
- **Persistence**: none.
- **Behaviour**: `touchBeganInside:allTouches:` @ 0x100136c3c (no story): when the banana is not active, places it at the tree's crown and lets it fall (`enterIsland` + fall), else (with monkeys enabled) shakes a perched monkey out. `startMonkeyClimb:` → `BananaTreeMonkeyClimb`, `monkeyLand:` → `BananaTreeMonkeyLand`.

#### `BananaLogic`

`SpriteLogic` · `BananaLogic.c` · 36 methods + 5 item handlers · 31 804 B

- **Role**: the banana: falls from the tree, eaten by pygmies or monkeys; a monkey leaves the peel, on which pygmies slip.
- **Where**: `IslandDisplayGroup/Banana` on Ape island.
- **Created**:
  - `setLogic` item(s) `logicId=BananaLogic` in `PocketGod`: `IslandDisplayGroup/Banana`
  - stored in `mBananaLogic`, listener = view controller.
- **Listener**:
  - `<BananaLogicListener>` (`mBananaLogicListener`): `isGravityEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyLogicArray:`, `onBananaDrag:`, `onBananaDragComplete:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `PygmyLogic` (`isEdible`, `isSlippable`, `slipOnBanana:`, eating protocol, `thrownByCharlie:`), `MonkeyLogic` (`startEatingByMonkey:`, `finishEatingByMonkey`).
  - also sent in code: `PygmyLogic` (`hitByBanana:`, `stopEating`).
- **Called from**: `dropWhileEating`, `isEdible` ← MonkeyLogic, PygmyLogic(Island); `enterIsland`, `touchBeganInside:allTouches:` ← BananaTreeLogic; `exitIsland` ← PocketGodViewController, PocketGodViewController(Game); `finishEating`, `isSlippable`, `pickUpForEating`, `slipOnBanana:`, `startEating:` ← PygmyLogic(Island); `finishEatingByMonkey`, `startEatingByMonkey:` ← MonkeyLogic; `isActive` ← BananaTreeLogic, PocketGodViewController(Game); `thrownByCharlie:` ← PygmyLogic(Charlie).
- **Data ids**:
  - behaviours: `BananaBounce`, `BananaDie`, `BananaDrag`, `BananaDropWhileEating`, `BananaDroppedByHurricane`, `BananaExitIsland`, `BananaFall`, `BananaFinishEatingByMonkey`, `BananaPeelBounce`, `BananaPeelDrag`, `BananaPeelPickedUpByHurricane`, `BananaPickedUpByHurricane`, `BananaStopSlide`, `BananaTiltSlide`
  - markers: `DragDisplayMarker`, `IslandDisplayMarker`, `SmallObjectsMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `slide:::` @ 0x10001c2b8 (20 uses), `drag:::` @ 0x10001c2e4 (40 uses), `fall:::` @ 0x10001c494 (134 uses), `fallFromHurricane:::` @ 0x10001c50c (9 uses), `die:::` @ 0x10001c544 (119 uses).
- **Persistence**: none.
- **Behaviour**: Shared pattern of §2.2 with slide friction 0.04, floor `y = 98` (`BananaBounce` / `BananaPeelBounce`, `BananaDie`). `finishEatingByMonkey` turns it into a peel thrown up (`vel.y = 600`); `isSlippable` = peel on the island; `slipOnBanana: p` sends the peel flying `(±200 by the pygmy's facing (DAT_10029d3f0), 400)`.

### 4.12 Konkey Dong mini-game

Konkey Dong, a Donkey Kong parody started from the Kong statue on Ape island. Level layouts come from `assets/data/konkey-dong-levels.json` (`docs/re/struct-xml.md` §10, `KonkeyDongLevel` records of 0x58 bytes in `mKonkeyDongLevelArray[10]`). The pygmy's climbing and jumping are in `PygmyLogic(KonkeyDong)`.

#### `KongStatueLogic`

`SpriteLogic` · `KongStatueLogic.c` · 41 methods + 1 item handler · 44 074 B

- **Role**: the Kong statue of Ape island and host of the Konkey Dong mini-game (a Donkey Kong parody): a pygmy dropped into the statue is kidnapped by Konkey Dong; another pygmy must climb the level's platforms and ladders, dodging boulders and beetles, to rescue it; levels cycle with rising difficulty.
- **Where**: Ape island (`ApeIslandDisplayGroup/KongStatue`, after `ApeIslandObjectsDisplayMarker`); mini-game in `KonkeyDongDisplayGroup` (levels `KonkeyDongLevel1…6DisplayGroup`, inserted after `KonkeyDongLevelDisplayMarker`).
- **Created**:
  - `setLogic` item(s) `logicId=KongStatueLogic` in `PocketGod`: `ApeIslandDisplayGroup/KongStatue`
  - stored in `mKongStatueLogic`, listener = view controller. Init stops its game frame and builds: `KonkeyDongDisplayGroup/KonkeyDong`'s logic (the `setLogic`-created `KonkeyDongLogic`, listener = statue), a `KonkeyDongOverlay` and a `BonusSplash` `MultiModelLogic` (listener = statue), a `KonkeyDongKidnap` `KonkeyDongLogic` (listener = statue) at the statue position, a `KonkeyDongCage` `MultiModelLogic` bound to the Konkey Dong's channel 2 (`KonkeyDongCageInit`), the `Background` logic (`KonkeyDongBackgroundInit`), then `parseStructXml` @ 0x100187f04 (levels, platforms, ladders, picks, beetles; `docs/re/struct-xml.md` §10, `assets/data/konkey-dong-levels.json`).
- **Listener**:
  - `<KongStatueLogicListener>` (`mKongStatueLogicListener`): `pygmyLogicArray:`, `isKonkeyDongEnabled`, `addPygmy`, `onKonkeyDongStart:pygmyLogic:`, `onKonkeyDongStop:`, `onKonkeyDongScore:`, `onKonkeyDongStartMovingCamera`, `onKonkeyDongMoveCameraToNextLevel:`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: `KonkeyDongLogic` ×2, `KonkeyDongPlatformLogic`, `KonkeyDongLadderLogic`, `KonkeyDongPickLogic`, `KonkeyDongBeetleLogic` (per level), `KonkeyDongBoulderLogic` (pooled by `throwBoulderAtXPos:yPos:`), `PygmyLogic(KonkeyDong)` (`kidnappedByKonkeyDong…`, `startStaringAtKidnapping`/`stopStaringAtKidnapping`, the level/array getters); `endLevel` goes to Konkey Dong, the beetles and the boulders, the overlay and splash via `onMultiModelEvent:eventId:` (`BonusSplashComplete`, `KonkeyDongOverlayComplete`).
  - also sent in code: `KonkeyDongBeetleLogic` (`activate:`, `setLoopUnlock:`); `KonkeyDongBoulderLogic` (`activateAtXPos:yPos:loopCount:`); `KonkeyDongLadderLogic` (`setDoNotPass:`, `setForceBoulderDrop:`); `KonkeyDongLogic` (`cancelKidnapPygmy`, `kidnapPygmy:`, `startKonkeyDong`, `startNextLevel:`); `KonkeyDongPickLogic` (`activate:`, `setLoopLock:`); `KonkeyDongPlatformLogic` (`setBoulderXPosVel:`, `setFinalPlatform:`, `setFlipBoulderXPosVel:`, `setStartPlatform:`); `PygmyLogic` (`canChaseKonkeyDong`, `chaseKonkeyDong`, `jumpToNextKonkeyDongLevel`, `startKonkeyDong`, `tossFromKongStatue`).
- **Called from**: `endLevel`, `konkeyDongBeetleLogicArray`, `konkeyDongBoulderLogicArray`, `konkeyDongLadderLogicArray`, `konkeyDongLevelDisplayObject`, `konkeyDongLogic`, `konkeyDongPickLogicArray`, `pygmyIntroComplete`, `showBonusSplash:` ← PygmyLogic(KonkeyDong); `enterIsland:` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland` ← PocketGodViewController; `isAvailable`, `pygmyFallIntoKongStatue:`, `startKonkeyDong:` ← PygmyLogic(Island); `konkeyDongLevel` ← PocketGodViewController(Game), PygmyLogic; `konkeyDongPlatformLogicArray` ← KonkeyDongBeetleLogic, PygmyLogic(KonkeyDong); `levelTransitionComplete` ← PocketGodViewController(Game); `removePygmyFromKongStatue:` ← PygmyLogic, PygmyLogic(Charlie), PygmyLogic(Island), PygmyLogic(Story); `stopKonkeyDong` ← PygmyLogic.
- **Data ids**:
  - behaviours: `KongStatueInit`, `KongStatuePygmyFallInside`, `KonkeyDongBackgroundInit`, `KonkeyDongBonusSplashInit`, `KonkeyDongCageInit`, `KonkeyDongOverlayInit`
  - models: `BonusSplash`, `KonkeyDongBoulder`, `KonkeyDongCage`, `KonkeyDongKidnap`, `KonkeyDongOverlay`
  - groups: `KonkeyDongDisplayGroup`
  - objects: `Background`, `KonkeyDong`
  - markers: `ApeIslandObjectsDisplayMarker`, `DragDisplayMarker`, `KonkeyDongLevelDisplayMarker`, `KonkeyDongPygmyDisplayMarker`, `MenuDisplayMarker`
  - event ids: `BonusSplashComplete`, `KonkeyDongOverlayComplete`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `startKonkeyDongKidnap:::` @ 0x100187c18 (1 use).
- **Persistence**: none.
- **Behaviour**:
  - `KonkeyDongLevel` (0x58 bytes, `mKonkeyDongLevelArray[10]`): display group, platform/ladder/pick/beetle arrays, vegetation logic, width/height, kong pos, start pygmy pos; getters `konkeyDongLevel`, `konkeyDongLevelDisplayObject`, `konkeyDong{Platform,Ladder,Pick,Beetle,Boulder}LogicArray`.
  - Kidnap: `pygmyFallIntoKongStatue:` binds the pygmy on channel 1 (`KongStatuePygmyFallInside`); `startKonkeyDongKidnap:::` hands it to the kidnap logic; `startPygmyKidnapReaction` makes the other pygmies stare.
  - `startKonkeyDong: pygmy` @ 0x10018747c: registers itself as a game-frame listener, difficulty 0, level index 0, inserts the level and starts the climb; `pygmyIntroComplete` → listener `onKonkeyDongStartMovingCamera`. `initActiveLevel:` positions Konkey Dong at the level's kong pos and (re)activates picks/beetles by the loop count. `endLevel` stops boulders/beetles; `endLevelComplete` advances `index = (index + 1) % count`, increments the completed count (loop = completed / count) and transitions; `levelTransitionComplete` removes the previous level. `showBonusSplash:` reports `onKonkeyDongScore: 100` and shows the splash before `KonkeyDongPygmyDisplayMarker`. `stopKonkeyDong` unregisters and clears everything (`onKonkeyDongStop:`).
  - `throwBoulderAtXPos:yPos:` @ 0x100186f8c reuses an inactive boulder from the pool or creates a `KonkeyDongBoulder` multimodel + logic, then `activateAtXPos:yPos:loopCount:`.

#### `KonkeyDongLogic`

`SpriteLogic` · `KonkeyDongLogic.c` · 12 methods + 3 item handlers · 6 234 B

- **Role**: Konkey Dong himself (the boulder-throwing ape at the top of each level) and, as a second instance, the kidnapper that carries the pygmy away from the statue.
- **Where**: `KonkeyDongDisplayGroup/KonkeyDong` (game) and the `KonkeyDongKidnap` multimodel (Ape island).
- **Created**:
  - `setLogic` item(s) `logicId=KonkeyDongLogic` in `PocketGod`: `KonkeyDongDisplayGroup/KonkeyDong`
  - allocated in `KongStatueLogic::initWithDisplayObject:` @ 0x100186040
  - the game instance by `setLogic` (only created; its listener is set by `KongStatueLogic`'s init); the kidnapper by `KongStatueLogic::initWithDisplayObject:`.
- **Listener**:
  - `<KonkeyDongLogicListener>` (`mKonkeyDongLogicListener`): `kidnapPygmy`, `kidnapPygmyComplete`, `startPygmyKidnapReaction`, `throwBoulderAtXPos:yPos:`, `endLevelComplete`
  - set in `KongStatueLogic::initWithDisplayObject:` @ 0x100186040
- **Talks to**: `PygmyLogic` (`kidnappedByKonkeyDong`, `kidnappedByKonkeyDongComplete`).
- **Called from**: `cancelKidnapPygmy`, `kidnapPygmy:`, `startKonkeyDong`, `startNextLevel:` ← KongStatueLogic; `endLevel` ← KongStatueLogic, PygmyLogic(KonkeyDong).
- **Data ids**:
  - behaviours: `KonkeyDongCancelKidnapPygmy`, `KonkeyDongEndLevel`, `KonkeyDongInit`, `KonkeyDongKidnapPygmy`, `KonkeyDongThrowBoulder`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `kidnapPygmyComplete:::` @ 0x1000900b0 (1 use), `cancelKidnapPygmyComplete:::` @ 0x10009013c (1 use), `endLevelComplete:::` @ 0x10009031c (1 use).
- **Persistence**: none.
- **Behaviour**: init `KonkeyDongInit`. `kidnapPygmy: p` → pygmy `kidnappedByKonkeyDong`, `KonkeyDongKidnapPygmy`, game frame stopped; `animResetIdol` binds the pygmy on channel 1 and tells the listener `kidnapPygmy`; `animPygmyReaction` → `startPygmyKidnapReaction`; `cancelKidnapPygmy` → `KonkeyDongCancelKidnapPygmy`. Game: `startKonkeyDong` (difficulty 1); `startNextLevel: loop` → difficulty `max(0.3, 1 − 0.15·loop)`, `KonkeyDongInit`; `onGameFrame:` @ 0x10009021c counts down `mThrowBoulderTime` and plays `KonkeyDongThrowBoulder`, resetting it to `difficulty·14` s; `animThrowBoulder` asks the listener to `throwBoulderAtXPos: x + 94 yPos: y + 28`; `endLevel` → `KonkeyDongEndLevel`.

#### `KonkeyDongPlatformLogic`

`SpriteLogic` · `KonkeyDongPlatformLogic.c` · 20 methods + 1 item handler · 9 526 B

- **Role**: one sloped girder of a Konkey Dong level.
- **Where**: Konkey Dong level groups.
- **Created**:
  - allocated in `KongStatueLogic::parseStructXml` @ 0x100187f04
  - by `KongStatueLogic::parseStructXml` (listener = the statue).
- **Listener**:
  - `<KonkeyDongPlatformLogicListener>` (`mKonkeyDongPlatformLogicListener`): `konkeyDongLevelDisplayObject`
  - set in `KongStatueLogic::parseStructXml` @ 0x100187f04
- **Talks to**: `PygmyLogic(KonkeyDong)` (`checkCollision:lastPos:`), `KonkeyDongBoulderLogic` (`checkBoulderCollision:radius:lastPos:`, `boulderXPosVel`, `flipBoulderXPosVel`), `KonkeyDongBeetleLogic` (`checkBeetleCollision:radius:lastPos:`).
- **Called from**: `boulderXPosVel`, `checkBoulderCollision:radius:lastPos:`, `flipBoulderXPosVel` ← KonkeyDongBoulderLogic; `checkBeetleCollision:radius:lastPos:`, `platformMaxXPos`, `platformMinXPos` ← KonkeyDongBeetleLogic; `checkCollision:lastPos:` ← PygmyLogic(KonkeyDong), PygmyLogic(OogaJump), PygmyLogic(TheRuns), PygmyLogic(Underwater); `isFinalPlatform`, `isStartPlatform` ← PygmyLogic(KonkeyDong); `setBoulderXPosVel:`, `setFinalPlatform:`, `setFlipBoulderXPosVel:`, `setStartPlatform:` ← KongStatueLogic.
- **Data ids**:
  - behaviours: `KonkeyDongPlatform1Init`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `setPlatformParams:::` @ 0x1000cdc2c (10 uses).
- **Persistence**: none.
- **Behaviour**: init `KonkeyDongPlatform1Init`, game frame stopped (it only records its last position each frame when running). The platform spans `x ∈ [pos.x, pos.x + cos(rot.z)·collisionWidth]` (`setPlatformParams:::` sets the width) along the line of slope `tan(rot.z)` in level coordinates (level group y offset added); the three collision tests check whether the object crossed that line since its last position. Flags from the level data: start/final platform, boulder x speed and flip.

#### `KonkeyDongLadderLogic`

`SpriteLogic` · `KonkeyDongLadderLogic.c` · 14 methods + 1 item handler · 5 172 B

- **Role**: one ladder of a Konkey Dong level.
- **Where**: Konkey Dong level groups.
- **Created**:
  - allocated in `KongStatueLogic::parseStructXml` @ 0x100187f04
  - by `KongStatueLogic::parseStructXml` (listener = the statue).
- **Listener**:
  - `<KonkeyDongLadderLogicListener>` (`mKonkeyDongLadderLogicListener`): `konkeyDongLevelDisplayObject`
  - set in `KongStatueLogic::parseStructXml` @ 0x100187f04
- **Talks to**: `PygmyLogic(KonkeyDong)` (`checkTopCollision:lastPos:`, `checkBottomCollision:lastPos:`, `ladderHeight`), `KonkeyDongBoulderLogic` (calls `forceBoulderDrop`, `ladderHeight`), `KonkeyDongBeetleLogic` (calls `doNotPass`, `ladderHeight`), `KongStatueLogic` (sets `setDoNotPass:`, `setForceBoulderDrop:` from the level layout).
- **Called from**: `checkBottomCollision:lastPos:`, `checkTopCollision:lastPos:` ← PygmyLogic(KonkeyDong); `doNotPass` ← KonkeyDongBeetleLogic; `forceBoulderDrop` ← KonkeyDongBoulderLogic; `ladderHeight` ← KonkeyDongBeetleLogic, KonkeyDongBoulderLogic, PygmyLogic(KonkeyDong); `setDoNotPass:`, `setForceBoulderDrop:` ← KongStatueLogic.
- **Data ids**:
  - behaviours: `KonkeyDongLadder1Init`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `setLadderParams:::` @ 0x10000d104 (4 uses).
- **Persistence**: none.
- **Behaviour**: init `KonkeyDongLadder1Init`, game frame stopped. Top/bottom tests: the object within 10 px of the ladder's x and near `levelY + ladderY (+ ladderHeight)`.

#### `KonkeyDongPickLogic`

`SpriteLogic` · `KonkeyDongPickLogic.c` · 10 methods · 2 722 B

- **Role**: the pick (hammer) power-up of a Konkey Dong level.
- **Where**: Konkey Dong level groups.
- **Created**:
  - allocated in `KongStatueLogic::parseStructXml` @ 0x100187f04
  - by `KongStatueLogic::parseStructXml`; the listener protocol is never set.
- **Listener**: `<KonkeyDongPickLogicListener>` (`mKonkeyDongPickLogicListener`): no methods
- **Called from**: `activate:`, `setLoopLock:` ← KongStatueLogic.
- **Data ids**:
  - behaviours: `KonkeyDongPickDeactivate`, `KonkeyDongPickInit`
- **Persistence**: none.
- **Behaviour**: init `KonkeyDongPickInit`, game frame stopped. `activate: loop` → active with `KonkeyDongPickInit` while `loop < loopLock`, else `KonkeyDongPickDeactivate`.

#### `KonkeyDongBeetleLogic`

`SpriteLogic` · `KonkeyDongBeetleLogic.c` · 16 methods + 6 item handlers · 21 157 B

- **Role**: a beetle walking the girders of a Konkey Dong level; it rolls up and can be jumped on or smashed with the pick.
- **Where**: Konkey Dong level groups.
- **Created**:
  - allocated in `KongStatueLogic::parseStructXml` @ 0x100187f04
  - by `KongStatueLogic::parseStructXml` (listener = the statue).
- **Listener**:
  - `<KonkeyDongBeetleLogicListener>` (`mKonkeyDongBeetleLogicListener`): `konkeyDongLevel`, `konkeyDongLevelDisplayObject`, `konkeyDongPlatformLogicArray`, `konkeyDongLadderLogicArray`
  - set in `KongStatueLogic::parseStructXml` @ 0x100187f04
- **Talks to**: `KonkeyDongPlatformLogic`/`KonkeyDongLadderLogic` (via `konkeyDongPlatformLogicArray`/`…LadderLogicArray`), `PygmyLogic(KonkeyDong)` (`checkCollision:lastPos:`, `isRolledUp`, `jumpOnBeetle:`, `smashBeetle`, `checkKonkeyDongCollision:`).
  - also sent in code: `KonkeyDongLadderLogic` (`doNotPass`, `ladderHeight`); `KonkeyDongPlatformLogic` (`checkBeetleCollision:radius:lastPos:`, `platformMaxXPos`, `platformMinXPos`).
- **Called from**: `activate:`, `endLevel`, `setLoopUnlock:` ← KongStatueLogic; `checkCollision:lastPos:` ← PygmyLogic(KonkeyDong), PygmyLogic(OogaJump), PygmyLogic(TheRuns), PygmyLogic(Underwater); `isRolledUp`, `jumpOnBeetle:`, `smashBeetle` ← PygmyLogic(KonkeyDong).
- **Data ids**:
  - behaviours: `KonkeyDongBeetleChasePygmy`, `KonkeyDongBeetleDownLadder`, `KonkeyDongBeetleIdle`, `KonkeyDongBeetleLeaveLadder`, `KonkeyDongBeetleLocked`, `KonkeyDongBeetleRollTurn`, `KonkeyDongBeetleRollUp`, `KonkeyDongBeetleRollWalk`, `KonkeyDongBeetleSmashed`, `KonkeyDongBeetleTurn`, `KonkeyDongBeetleUpLadder`, `KonkeyDongBeetleWalk`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `whichWayDoIGo:::` @ 0x1000c5fb0 (5 uses), `walk:::` @ 0x1000c614c (16 uses), `rollAfterPygmy:::` @ 0x1000c630c (1 use), `smashComplete:::` @ 0x1000c640c (1 use), `rollUpComplete:::` @ 0x1000c6414 (2 uses), `startLadder:::` @ 0x1000c64a4 (2 uses).
- **Persistence**: none.
- **Behaviour**: remembers its start position; `activate: loop` → locked (`KonkeyDongBeetleLocked`) while `loop < loopUnlock`, else walking (roll counters reset). Collision with the pygmy: distance < 16. It walks platforms, takes ladders (`mbUseNextLadder`, skip counters) and falls like the boulder (`onGameFrame:` / `checkKonkeyDongCollision:` in `KonkeyDongBeetleLogic.c`).

#### `KonkeyDongBoulderLogic`

`SpriteLogic` · `KonkeyDongBoulderLogic.c` · 12 methods + 1 item handler · 12 789 B

- **Role**: a boulder thrown by Konkey Dong that rolls down the girders and ladders.
- **Where**: Konkey Dong level groups.
- **Created**:
  - allocated in `KongStatueLogic::throwBoulderAtXPos:yPos:` @ 0x100186f8c
  - pooled by `KongStatueLogic::throwBoulderAtXPos:yPos:` (listener = the statue).
- **Listener**:
  - `<KonkeyDongBoulderLogicListener>` (`mKonkeyDongBoulderLogicListener`): `konkeyDongLevel`, `konkeyDongLevelDisplayObject`, `konkeyDongPlatformLogicArray`, `konkeyDongLadderLogicArray`
  - set in `KongStatueLogic::throwBoulderAtXPos:yPos:` @ 0x100186f8c
- **Talks to**: platforms and ladders (as the beetle), `PygmyLogic(KonkeyDong)` (`checkCollision:lastPos:` distance < 16, `crackBoulder`).
  - also sent in code: `KonkeyDongLadderLogic` (`forceBoulderDrop`, `ladderHeight`); `KonkeyDongPlatformLogic` (`boulderXPosVel`, `checkBoulderCollision:radius:lastPos:`, `flipBoulderXPosVel`).
- **Called from**: `activateAtXPos:yPos:loopCount:`, `endLevel` ← KongStatueLogic; `checkCollision:lastPos:` ← PygmyLogic(KonkeyDong), PygmyLogic(OogaJump), PygmyLogic(TheRuns), PygmyLogic(Underwater); `crackBoulder` ← PygmyLogic(KonkeyDong).
- **Data ids**:
  - behaviours: `KonkeyDongBoulderCrack`, `KonkeyDongBoulderDownLadder`, `KonkeyDongBoulderInit`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `crackComplete:::` @ 0x10017ecc4 (1 use).
- **Persistence**: none.
- **Behaviour**: init enables gravity, `KonkeyDongBoulderInit`. `activateAtXPos:yPos:loopCount:` @ 0x10017eafc: random first ladder choice (`mbUseNextLadder = !(rand & 1)`), difficulty `1 + 0.15·loop`. `onGameFrame:` @ 0x10017f21c / `checkKonkeyDongCollision:` @ 0x10017ecf8: gravity, rolling along the current platform at its `boulderXPosVel` (flipped as configured) scaled by difficulty, dropping down ladders (forced or random). `crackBoulder` → `KonkeyDongBoulderCrack` (`crackComplete:::` removes it); `endLevel` freezes it.

### 4.13 The Runs mini-game

The Runs, an endless runner in a lava cave entered through the crack hole on the main island. The pygmy side is `PygmyLogic(TheRuns)`.

#### `CrackHoleLogic`

`SpriteLogic` · `CrackHoleLogic.c` · 25 methods · 54 242 B

- **Role**: the ground crack and host of "The Runs" mini-game: dragging a finger along the island opens a chain of cracks; a pygmy that falls in drops into a lava cave and runs an endless platform course (platforms, sinking and crumbling platforms, stalactites, lava rocks, turds) until it dies.
- **Where**: main island (`IslandDisplayGroup/CrackHole`; cracks before `StartCoconutDisplayMarker`); mini-game in `TheRunsDisplayGroup` / `TheRunsDemonDisplayGroup`.
- **Created**:
  - `setLogic` item(s) `logicId=CrackHoleLogic` in `PocketGod`: `IslandDisplayGroup/CrackHole`
  - stored in `mCrackHoleLogic`, listener = view controller. Init creates 4 `Turd` multimodels with `TheRunsTurdLogic` and a `LavaRock` with `TheRunsRockLogic`, and picks up the `TheRunsDisplayGroup` objects' logics: `Background`, `LavaBackground`, `StalactiteBackground`, `LavaForeground1/2`, `Demon` and `Bats` (moved into `TheRunsDemonDisplayGroup`), `Platform1…4` (`TheRunsPlatformLogic`, listener = the crack hole) and `Stalactite1…4` (`TheRunsStalactiteLogic`, deactivated).
- **Listener**:
  - `<CrackHoleLogicListener>` (`mCrackHoleLogicListener`): `pygmyLogicArray:`, `islandMinXPos`, `islandMaxXPos`, `onTheRunsStart:pygmyLogic:`, `onTheRunsStop:`, `onTheRunsScore:`, `onTheRunsStartMovingCamera`, `onTheRunsShakeScreenStart`, `onTheRunsShakeScreenStop`
  - set in `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c
- **Talks to**: the The Runs element logics above (`activateAtXPos:yPos:`, `deactivate`, `setTheRunsPlatformEvent:`, `setCheckSideCollision:`, `theRunsPlatformEvent`), crack `MultiModelLogic`s (listener = the crack hole; `CrackOpen`/`CrackClose` events), `PygmyLogic` (`startTheRuns`, falls into the crack; `PygmyLogic(TheRuns)` reads `theRunsRockLogic`, `theRunsPlatformLogicArray`, `theRunsStalactiteLogicArray`, `theRunsTurdLogicArray`).
  - also sent in code: `PygmyLogic` (`canFallIntoCrackHole:`, `fallIntoCrackHole:`); `TheRunsPlatformLogic` (`activate`, `deactivate`, `setCheckSideCollision:`, `setTheRunsPlatformEvent:`, `theRunsPlatformEvent`); `TheRunsRockLogic` (`activateAtXPos:yPos:`, `deactivate`); `TheRunsStalactiteLogic` (`activateAtXPos:yPos:`, `deactivate`, `isActive`); `TheRunsTurdLogic` (`activateAtXPos:yPos:`, `deactivate`).
- **Called from**: `areCracksActive` ← PocketGodViewController; `drag:`, `startDragging:`, `startTheRuns:`, `stopDragging` ← PocketGodViewController(Game); `stopTheRuns`, `theRunsPlatformLogicArray`, `theRunsRockLogic`, `theRunsStalactiteLogicArray`, `theRunsTurdLogicArray` ← PygmyLogic(TheRuns).
- **Data ids**:
  - behaviours: `BlankAnim`, `CrackClose`, `CrackOpen`, `TheRunsBackgroundInit`, `TheRunsBatsInit`, `TheRunsDemonInit`, `TheRunsLavaBackgroundInit`, `TheRunsLavaForegroundInit`, `TheRunsStalactiteBackgroundInit`
  - models: `Crack`, `LavaRock`, `Turd`
  - groups: `TheRunsDemonDisplayGroup`, `TheRunsDisplayGroup`
  - objects: `Background`, `Bats`, `Demon`, `LavaBackground`, `LavaForeground1`, `LavaForeground2`, `Platform1`, `Platform2`, `Platform3`, `Platform4`, `Stalactite1`, `Stalactite2`, `Stalactite3`, `Stalactite4`, `StalactiteBackground`
  - markers: `StartCoconutDisplayMarker`
  - event ids: `CrackClose`, `CrackOpen`
- **Persistence**: none.
- **Behaviour**:
  - Cracks: `startDragging:` / `drag:` (from `(Game)` world drags) / `stopDragging`: after 25 px of horizontal drag a crack direction is fixed and `createCrack:` @ 0x100125e54 adds `Crack` multimodels at `y = 90` (scale x = direction) as the finger advances; while dragging the frame update drops a pygmy standing over the open crack into it (`mCrackHolePygmyLogic`); on release the first crack closes (`CrackClose`), the rest close in turn through `onMultiModelEvent:eventId:`. `areCracksActive` is asked by the view controller.
  - `startTheRuns: pygmy` @ 0x1001269e0: registers as game-frame listener, difficulty 0, pygmy `startTheRuns`, listener `onTheRunsStart:pygmyLogic:`, seeds the platform events. `TheRunsPlatformEvent` (0x24 bytes, array `mTheRunsPlatformEventArray`) records platform type (4 = crumbling), position and hazards; `chooseTheRunsPlatformEvent:` @ 0x1001260f4 picks the next platform height within ±50 of the last (≥ 60) and gaps reachable by a jump, using `getJumpDistance:heightDelta:jumpYVel:jumpTime:` @ 0x1001260a4 (integrates a jump at 30 Hz with `vel.y −= 100` per step until it falls 100 below the start or below the target height). `placeTheRunsPlatformLogic:`, `processOffScreenPlatforms` (recycle platforms and place rocks, turds, stalactites, the sinking platform) and `processPlatformEvents:` run every frame; the course scrolls faster with difficulty and the score goes to `onTheRunsScore:`.
  - Sinking platform: `onPygmyLandOnSinkingPlatform:` starts the screen shake (`onTheRunsShakeScreenStart`), `onPygmyFallFromSinkingPlatform` stops it; `onPygmyBurnPygmyComplete` → `onTheRunsStartMovingCamera`. `stopTheRuns` unregisters and resets everything (`onTheRunsStop:`).

#### `TheRunsPlatformLogic`

`SpriteLogic` · `TheRunsPlatformLogic.c` · 11 methods + 1 item handler · 5 876 B

- **Role**: one of the four recycled stone platforms of The Runs.
- **Where**: `TheRunsDisplayGroup/Platform1…4`.
- **Created**:
  - `setLogic` item(s) `logicId=TheRunsPlatformLogic` in `PocketGod`: `TheRunsDisplayGroup/Platform1`, `TheRunsDisplayGroup/Platform2`, `TheRunsDisplayGroup/Platform3`, `TheRunsDisplayGroup/Platform4`
  - by `setLogic` (only created); `CrackHoleLogic` init sets its listener.
- **Listener**:
  - `<TheRunsPlatformLogicListener>` (`mTheRunsPlatformLogicListener`): `onPygmyBurnPygmyComplete`, `onPygmyLandOnSinkingPlatform:`, `onPygmyFallFromSinkingPlatform`
  - set in `CrackHoleLogic::initWithDisplayObject:` @ 0x10012566c
- **Talks to**: `PygmyLogic(TheRuns)` (`checkCollision:lastPos:`, `burnPygmy:`; platform sends `theRunsBurnPygmyComplete`).
- **Called from**: `activate` ← CrackHoleLogic, PocketGodViewController(Game); `burnPygmy:` ← PygmyLogic(TheRuns); `checkCollision:lastPos:` ← PygmyLogic(KonkeyDong), PygmyLogic(OogaJump), PygmyLogic(TheRuns), PygmyLogic(Underwater); `deactivate`, `setCheckSideCollision:`, `setTheRunsPlatformEvent:`, `theRunsPlatformEvent` ← CrackHoleLogic.
- **Data ids**:
  - behaviours: `TheRunsPlatformBurnPygmyInit`, `TheRunsPlatformCrumbleInit`, `TheRunsPlatformInit`, `TheRunsPlatformSinking`, `TheRunsPlatformStumble`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `onBurnPygmyComplete:::` @ 0x1000d5d28 (1 use).
- **Persistence**: none.
- **Behaviour**: init `TheRunsPlatformInit`, game frame stopped. `setTheRunsPlatformEvent: e` → `TheRunsPlatformCrumbleInit` for type 4 else `TheRunsPlatformInit`. `checkCollision:lastPos:` @ 0x1000d5da8: top surface at `y + 227` spanning `x ∈ (pos.x, pos.x + 205)`, valid while `y ≥ −193`; landing snaps the pygmy onto it, a crumbling platform starts sinking (`onPygmyLandOnSinkingPlatform:`); with side collision on, running into its left side blocks the pygmy. `burnPygmy:` binds the pygmy on channel 1 (`TheRunsPlatformBurnPygmyInit`).

#### `TheRunsStalactiteLogic`

`SpriteLogic` · `TheRunsStalactiteLogic.c` · 9 methods · 3 615 B

- **Role**: a stalactite hanging from the cave ceiling in The Runs.
- **Where**: `TheRunsDisplayGroup/Stalactite1…4`.
- **Created**:
  - `setLogic` item(s) `logicId=TheRunsStalactiteLogic` in `PocketGod`: `TheRunsDisplayGroup/Stalactite1`, `TheRunsDisplayGroup/Stalactite2`, `TheRunsDisplayGroup/Stalactite3`, `TheRunsDisplayGroup/Stalactite4`
  - by `setLogic` (only created); `CrackHoleLogic` sets the listener (protocol empty) and deactivates them.
- **Listener**:
  - `<TheRunsStalactiteLogicListener>` (`mTheRunsStalactiteLogicListener`): no methods
  - set in `CrackHoleLogic::initWithDisplayObject:` @ 0x10012566c
- **Called from**: `activateAtXPos:yPos:`, `deactivate` ← CrackHoleLogic; `isActive` ← CrackHoleLogic, PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `TheRunsStalactiteInit`
  - markers: `TheRunsPygmyDisplayMarker`
- **Persistence**: none.
- **Behaviour**: init `TheRunsStalactiteInit`. `activateAtXPos:yPos:` places it (`z = −277.12`, before `TheRunsPygmyDisplayMarker`) with collision on; `checkCollision:lastPos:`: the pygmy above `y` and within `(x + 10, x + 209)` collides.

#### `TheRunsRockLogic`

`SpriteLogic` · `TheRunsRockLogic.c` · 7 methods · 3 253 B

- **Role**: the lava rock obstacle of The Runs.
- **Where**: The Runs cave.
- **Created**:
  - allocated in `CrackHoleLogic::initWithDisplayObject:` @ 0x10012566c
  - by `CrackHoleLogic::initWithDisplayObject:` (`LavaRock` multimodel); no listener is set.
- **Listener**: `<TheRunsRockLogicListener>` (`mTheRunsRockLogicListener`): no methods
- **Called from**: `activateAtXPos:yPos:`, `deactivate` ← CrackHoleLogic; `checkCollision:lastPos:` ← PygmyLogic(TheRuns).
- **Data ids**:
  - behaviours: `BlankAnim`, `TheRunsRockExplode`, `TheRunsRockInit`
  - markers: `TheRunsPygmyDisplayMarker`
- **Persistence**: none.
- **Behaviour**: `activateAtXPos:yPos:` → placed before `TheRunsPygmyDisplayMarker`, `TheRunsRockInit`; `checkCollision:lastPos:` (distance < 40) → `TheRunsRockExplode`, once; `deactivate` → removed, `BlankAnim`.

#### `TheRunsTurdLogic`

`SpriteLogic` · `TheRunsTurdLogic.c` · 7 methods · 3 252 B

- **Role**: a turd dropped on the course of The Runs; the pygmy skids on it.
- **Where**: The Runs cave.
- **Created**:
  - allocated in `CrackHoleLogic::initWithDisplayObject:` @ 0x10012566c
  - 4 by `CrackHoleLogic::initWithDisplayObject:` (`Turd` multimodels); no listener is set.
- **Listener**: `<TheRunsTurdLogicListener>` (`mTheRunsTurdLogicListener`): no methods
- **Called from**: `activateAtXPos:yPos:`, `deactivate` ← CrackHoleLogic.
- **Data ids**:
  - behaviours: `BlankAnim`, `TheRunsTurdInit`, `TheRunsTurdSkid`
  - markers: `TheRunsPygmyDisplayMarker`
- **Persistence**: none.
- **Behaviour**: like the rock with distance < 30 and `TheRunsTurdInit` / `TheRunsTurdSkid`.

### 4.14 Moron Pests mini-game

Moron Pests: a pygmy that falls on the Misfit-island snake starts a chase by the turtle, interrupted by quiz questions answered with on-screen pests (ducks, bees).

#### `MoronPestsQuestionScreenLogic`

`BCScreenLogic` · `MoronPestsQuestionScreenLogic.c` · 30 methods + 3 item handlers + 1 class method · 71 037 B + 3 114 B of free helper functions

- **Role**: the Moron Pests quiz overlay: synonym, colour, bee-order and duck-order questions with difficulty-weighted variants and "don't answer" timeouts.
- **Where**: entry 45 (`MoronPestsQuestionScreen`, full screen at (0, 0, −277.12)); buttons and pests are entries 9–44.
- **Created**:
  - `createLogicClasses` table entry 45 (#45 new `BCMultiModel` `MoronPestsQuestionScreen`)
  - entry 45; `+instance` returns `DAT_1007e5080`. Init fetches 29 table entries: synonym models/buttons, colour models/overlays/buttons (entries 10–29), the ducks (30–33, `setHeadLogic:` with their head logics) and bees (40–44); every button's listener is the screen (table), ducks/bees report `onMoronDuckLogicSelected:` / `onMoronBeeLogicSelected:`.
- **Listener**:
  - `<MoronPestsQuestionScreenLogicListener>` (`mMoronPestsQuestionScreenLogicListener`): `onMoronPestsQuestionCorrect`, `onMoronPestsQuestionIncorrect`
  - wired by `initLogicListeners` to `MoronSnakeLogic`
- **Talks to**: `MoronDuckLogic`, `MoronBeeLogic` (`surroundPygmy:…`, `screenPos`, `moronDuckEyeScale`, `exitScreen`), the listener `MoronSnakeLogic` (`onMoronPestsQuestionCorrect` / `onMoronPestsQuestionIncorrect`, sent through selector pointers).
  - also sent in code: `MoronBeeLogic` (`surroundPygmy:screenPos:`); `MoronDuckLogic` (`setHeadLogic:`, `surroundPygmy:screenPos:scale:moronDuckEyeScale:`).
  - `getLogic:` 10–17 `MultiModelLogic`, 18–21 `BCButtonLogic`, 22–23 `MultiModelLogic`, 24–25 `BCButtonLogic`, 26–29 `MultiModelLogic`, 30–33 `MoronDuckLogic`, 40–44 `MoronBeeLogic`.
- **Called from**: `closeScreen`, `closeScreenNow`, `openMoronPestsQuestion`, `setDifficulty:`, `setMoronPestsPygmyLogic:` ← MoronSnakeLogic.
- **Data ids**:
  - behaviours: `CloseMoronPestsAnimalQuestion`, `CloseMoronPestsAnimalQuestionLose`, `CloseMoronPestsColorQuestion`, `CloseMoronPestsColorQuestionLose`, `CloseMoronPestsSynonymQuestion`, `CloseMoronPestsSynonymQuestionLose`, `MoronPestButtonOverlayDown`, `MoronPestButtonOverlayUp`, `ShowMoronPestsAnimalQuestion`, `ShowMoronPestsColorQuestion`, `ShowMoronPestsSynonymQuestion`
  - animations: `MoronPestButtonOverlayUp`
  - texts: `MoronPestsAnimalTitle`, `MoronPestsColorTitle`, `MoronPestsSynonymTitle`
  - other strings: `Blue`, `Don't Press %@, %@, or %@`, `Green`, `Orange`, `Purple`, `Red`, `Touch the bees from bottom to top`, `Touch the bees from left to right`, `Touch the bees from right to left`, `Touch the bees from top to bottom`, `Touch the ducks from biggest to smallest`, `Touch the ducks from biggest to smallest based on its eyes`, `Touch the ducks from smallest to biggest`, `Touch the ducks from smallest to biggest based on its eyes`, `Yellow`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `closeSynonymQuestionComplete:::` @ 0x100065b04 (1 use), `closeColorQuestionComplete:::` @ 0x100065f2c (1 use), `closeAnimalQuestionComplete:::` @ 0x100066130 (1 use).
- **Persistence**: none.
- **Behaviour**:
  - Variant choice `calculateRandomWeightDifficultyIndex:size:` @ 0x100063210 over `WeightDifficulty {f weight; f minDifficulty}` tables: entries whose min difficulty ≤ current difficulty take part with their weight (uniform in 0.1 steps). Question type (`DAT_10029d63c`): synonym 2, colour 2, bee 1, duck 1 (all from 0). Synonym (`DAT_10029d65c`): 5/1/5. Colour (`DAT_10029d674`, 12 variants): weights 4,1,4 from 0; 4,1,4 from 5000; 4 from 10000/15000/20000 (variants 6–8 and 9–11). Bee (`DAT_10029d6d4`): 1,1 from 0; 1,1 from 5000. Duck (`DAT_10029d6f4`): 2,2 from 0; 1,1 from 5000.
  - Colour questions use the six colours `Red`, `Green`, `Blue`, `Yellow`, `Orange`, `Purple`, randomized buttons (`randomizeColorButtons`), "tap several" variants (`displayTapMultipleColorsQuestion`, `calculateColorSwitchIndices:tapCount:`) and "Don't Press %@, %@, or %@" questions; synonym/colour "don't answer" variants close themselves when `mDontAnswerTimer` runs out (`onGameFrame:` @ 0x100063168). Bees: "Touch the bees from left to right / right to left / top to bottom / bottom to top"; ducks: "Touch the ducks from biggest to smallest / smallest to biggest (based on its eyes)" — the expected order comes from sorting the active pests with the comparators FUN_100063d60…1000644a4 (screen y, screen x, scale, eye scale, ascending/descending).
  - Titles from dynamic texts `MoronPestsSynonymTitle`, `MoronPestsColorTitle`, `MoronPestsAnimalTitle`; layer behaviours `ShowMoronPests…Question`, `CloseMoronPests…Question(Lose)`; button overlays `MoronPestButtonOverlayUp/Down`.

#### `MoronDuckLogic`

`SpriteLogic` · `MoronDuckLogic.c` · 19 methods + 2 item handlers · 7 977 B

- **Role**: one of the four ducks of the Moron Pests duck questions.
- **Where**: entries 30–33 (`MoronDuck1…4`), `MoronPestsDisplayGroup`.
- **Created**:
  - `createLogicClasses` table entries 30–33 (#30 new `BCMultiModel` `MoronDuck1`; #31 new `BCMultiModel` `MoronDuck2`; #32 new `BCMultiModel` `MoronDuck3`; #33 new `BCMultiModel` `MoronDuck4`)
  - table; listener = `MoronPestsQuestionScreenLogic`.
- **Listener**:
  - `<MoronDuckLogicListener>` (`mMoronDuckLogicListener`): `onMoronDuckLogicSelected:`
  - wired by `initLogicListeners` to `MoronPestsQuestionScreenLogic` (4)
- **Called from**: `exitScreen`, `moronDuckEyeScale`, `screenPos`, `setHeadLogic:`, `surroundPygmy:screenPos:scale:moronDuckEyeScale:` ← MoronPestsQuestionScreenLogic.
- **Data ids**:
  - behaviours: `MoronDuckExitScreen`, `MoronDuckSurroundPygmy`
  - animations: `RubberDuckHead1`, `RubberDuckHead2`, `RubberDuckHead3`, `RubberDuckHead4`
  - groups: `MoronPestsDisplayGroup`
  - markers: `MoronPestsPygmyDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `die:::` @ 0x1001a80d0 (119 uses), `exitScreenComplete:::` @ 0x1001a833c (0 uses).
- **Persistence**: none.
- **Behaviour**: box (−40, 0)–(40, 80), selectable; `gameFrameOrder` 60. `surroundPygmy:screenPos:scale:moronDuckEyeScale:` @ 0x1001a80d8 places it before `MoronPestsPygmyDisplayMarker` at a screen position (kept on screen by `onGameFrame:`: `x ≥ screenPos.x − group.x`), with a body scale and an eye-scale rank applied to its head logic; a tap while active → `onMoronDuckLogicSelected:`; `exitScreen` → `MoronDuckExitScreen`.

#### `MoronBeeLogic`

`SpriteLogic` · `MoronBeeLogic.c` · 17 methods + 2 item handlers · 6 482 B

- **Role**: one of the five bees of the Moron Pests bee questions.
- **Where**: entries 40–44 (`MoronBee1…5`), `MoronPestsDisplayGroup`.
- **Created**:
  - `createLogicClasses` table entries 40–44 (new `BCMultiModel` `MoronBee1`, new `BCMultiModel` `MoronBee2`, new `BCMultiModel` `MoronBee3`, … (5 entries))
  - table; listener = `MoronPestsQuestionScreenLogic`.
- **Listener**:
  - `<MoronBeeLogicListener>` (`mMoronBeeLogicListener`): `onMoronBeeLogicSelected:`
  - wired by `initLogicListeners` to `MoronPestsQuestionScreenLogic` (5)
- **Called from**: `exitScreen`, `screenPos`, `surroundPygmy:screenPos:` ← MoronPestsQuestionScreenLogic.
- **Data ids**:
  - behaviours: `MoronBeeExitScreen`, `MoronBeeSurroundPygmy`
  - groups: `MoronPestsDisplayGroup`
  - markers: `MoronPestsPygmyDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `die:::` @ 0x100175160 (119 uses), `exitScreenComplete:::` @ 0x1001752bc (0 uses).
- **Persistence**: none.
- **Behaviour**: box (−20,−20)–(20,20), selectable; `gameFrameOrder` 60. `surroundPygmy:screenPos:` @ 0x100175168 places it at `(screenPos.x + 480 − group.x, screenPos.y − group.y)` (flies in from the right), `MoronBeeSurroundPygmy`; on-screen clamp as the duck; tap → `onMoronBeeLogicSelected:`; `exitScreen` → `MoronBeeExitScreen`.

#### `MoronSnakeLogic`

`SpriteLogic` · `MoronSnakeLogic.c` · 32 methods + 7 item handlers + 1 class method · 35 569 B

- **Role**: the snake of Misfit island and host of the "Moron Pests" mini-game: a pygmy that falls on the snake starts a chase in which a turtle pursues the running pygmy while the player answers quick questions; correct answers keep the pygmy ahead.
- **Where**: Misfit island (location 6), entry 51 (`MoronSnake`); mini-game in `MoronPestsDisplayGroup`.
- **Created**:
  - `createLogicClasses` table entry 51 (#51 new `BCMultiModel` `MoronSnake`)
  - entry 51; `+instance` returns `DAT_1007d1f48`. Init fetches the turtle (entry 50), the question screen (45) and the background logics (entries 7 and 8; `MoronPestsBackground…`).
- **Listener**:
  - `<MoronSnakeLogicListener>` (`mMoronSnakeLogicListener`): `isStoryActive`, `isGravityEnabled`, `isMoronSnakeEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyLogicArray:`, `onMoronPestsStart:pygmyLogic:`, `onMoronPestsStartMovingCamera`, `onMoronPestsStopMovingCamera`, `onMoronPestsStop:`, `onMoronPestsScore:`, `onMoronSnakeDrag:`, `onMoronSnakeDragComplete:` (never sent by this class: `islandCenterXPos`, `islandMaxXPos`, `islandMinXPos`)
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `MoronTurtleLogic` (`startMoronPests:`; the turtle's listener is the snake: `onMoronTurtleCatchPygmy`, `onMoronPestsStopMovingCamera`), `MoronPestsQuestionScreenLogic` (`openMoronPestsQuestion`, `setDifficulty:`, `setMoronPestsPygmyLogic:`, `closeScreen(Now)`; its listener is the snake: `onMoronPestsQuestionCorrect`/`Incorrect`), `PygmyLogic(MoronPests)` (`startMoronPests`, `onMoronPestsAnswerCorrect`/`Incorrect`, `eatPygmyComplete:`), Baby pygmy (`thrownByBaby:` → `vel = (pygmyScale·−300, 800)`, `MoronSnakeThrownByBaby`).
  - also sent in code: `MoronPestsQuestionScreenLogic` (`closeScreen`, `closeScreenNow`); `PygmyLogic` (`eatenByMoronSnake:`, `onMoronPestsAnswerIncorrect`, `timeTemplePygmyType`).
  - `getLogic:` 7–8 `MultiModelLogic`, 45 `MoronPestsQuestionScreenLogic`, 50 `MoronTurtleLogic`.
- **Called from**: `eatPygmyComplete:` ← PygmyLogic, PygmyLogic(Island); `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `pygmyIntroComplete`, `stopMoronPests` ← PygmyLogic(MoronPests); `startMoronPests:`, `thrownByBaby:` ← PygmyLogic(Island).
- **Data ids**:
  - behaviours: `MoronSnakeDie`, `MoronSnakeDrag`, `MoronSnakeEatPygmy`, `MoronSnakeEatPygmyBaby`, `MoronSnakeEnterIsland`, `MoronSnakeFall`, `MoronSnakeFallIgnoreIsland`, `MoronSnakeIdle`, `MoronSnakeSlither`, `MoronSnakeSplash`, `MoronSnakeThrownByBaby`, `MoronSnakeTurn`
  - groups: `MoronPestsDisplayGroup`
  - markers: `DragDisplayMarker`, `SmallObjectsMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `resetLastPos:::` @ 0x10018cfac (5 uses), `whichWayDoIGo:::` @ 0x10018cffc (5 uses), `slither:::` @ 0x10018d0e8 (1 use), `idle:::` @ 0x10018d238 (17 uses), `drag:::` @ 0x10018d34c (40 uses), `fall:::` @ 0x10018d4fc (134 uses), `die:::` @ 0x10018d680 (119 uses).
- **Persistence**: none.
- **Behaviour**:
  - On the island it is a draggable creature (§2.2 with slide friction 0.025, floor `y = 85`, side collision, `MoronSnakeFall` when tilted; `slither`/`idle`/`whichWayDoIGo` handlers) that eats pygmies (`checkPygmyCollision` while falling onto them).
  - `startMoronPests: p` @ 0x10018dba8: mini-game on, difficulty 0, pygmy `startMoronPests`, turtle `startMoronPests:`, screen `setMoronPestsPygmyLogic:`, listener `onMoronPestsStart:pygmyLogic:`. `pygmyIntroComplete` registers the snake as a game-frame listener, opens the first question and starts the camera (`onMoronPestsStartMovingCamera`).
  - `onGameFrameMoronPests:` @ 0x10018ca18: `difficulty = groupX·−0.3125` (the distance run), reported as `onMoronPestsScore:` and passed to the question screen; parallax: background `x = groupX·−0.4`, rib background `x = groupX·−0.3`.
  - Each answer opens the next question and tells the pygmy `onMoronPestsAnswerCorrect` / `…Incorrect`; the turtle catching the pygmy closes the screen. `stopMoronPests` unregisters, closes the screen at once and reports `onMoronPestsStop:`.

#### `MoronTurtleLogic`

`SpriteLogic` · `MoronTurtleLogic.c` · 8 methods + 3 item handlers · 6 845 B

- **Role**: the giant turtle that chases the pygmy in Moron Pests (the pygmy first jumps off its back).
- **Where**: entry 50 (`MoronTurtle`), `MoronPestsDisplayGroup`.
- **Created**:
  - `createLogicClasses` table entry 50 (#50 new `BCMultiModel` `MoronTurtle`)
  - entry 50; listener = `MoronSnakeLogic` (table).
- **Listener**:
  - `<MoronTurtleLogicListener>` (`mMoronTurtleLogicListener`): `onMoronPestsStopMovingCamera`, `onMoronTurtleCatchPygmy`
  - wired by `initLogicListeners` to `MoronSnakeLogic`
- **Talks to**: `PygmyLogic(MoronPests)` (`pygmyJumpOnBack:`; turtle sends `jumpOnMoronTurtleBackComplete:`, `eatenByMoronTurtle:`, `eatenByMoronTurtleStopMoving:`, `eatenByMoronTurtleComplete:`).
- **Called from**: `pygmyJumpOnBack:` ← PygmyLogic(MoronPests); `startMoronPests:` ← MoronSnakeLogic.
- **Data ids**:
  - behaviours: `MoronTurtleEatPygmy`, `MoronTurtlePygmyJumpOnBack`, `MoronTurtleStartRunning`, `MoronTurtleWaitForPygmy`
  - groups: `MoronPestsDisplayGroup`
  - markers: `MoronPestsPygmyDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `die:::` @ 0x100173338 (119 uses), `onPygmyJumpOnBackComplete:::` @ 0x10017347c (1 use), `onEatPygmyComplete:::` @ 0x10017357c (1 use).
- **Persistence**: none.
- **Behaviour**: `startMoronPests: p` places it before `MoronPestsPygmyDisplayMarker` at (47, 130), `MoronTurtleWaitForPygmy`. `pygmyJumpOnBack:` binds the pygmy on channel 1 (`MoronTurtlePygmyJumpOnBack`); `onPygmyJumpOnBackComplete:::` starts the chase. `onGameFrame:` @ 0x10017315c while chasing: normally `vel.x += 0.2·dt` up to 525; in catch-up mode `vel.x += (500 − vel.x)·0.2·60·dt` until within 1 of 500; integration and `checkPygmyCollision`: pygmy x < turtle x + 90 → caught (`onMoronTurtleCatchPygmy`, pygmy on channel 1, `eatenByMoronTurtle:`, `MoronTurtleEatPygmy`). `animStopMoving` stops it and the camera.

### 4.15 Apocalypse island and temples

The apocalypse island (state 10) and its temples (states 11 to 17): the calendar clock and portal, disease clouds, the seven gems and pedestals, the room fader and temple doors, and the content of each temple (time, locust, dance, plague and purification fountains, closet, Xenu).

#### `ApocalypseCalendarLogic`

`SpriteLogic` · `ApocalypseCalendarLogic.c` · 21 methods + 9 item handlers · 20 128 B (the file also holds `FUN_1000d8290` (843 330 B), the generated `BCSequenceItemDef` initializer (sequence data, `docs/re/sequences.md`); not counted)

- **Role**: the Mayan calendar clock on the apocalypse island: three dials count down to 21 Dec 2012; when Tom (the pygmy from the future) is around it opens the portal, and at the end of an apocalypse it lights the gem pedestals' torches and raises the gems.
- **Where**: apocalypse island (location 10), entry 447 (`ApocalypseIslandDisplayGroup/ClockWheelMultiModel`).
- **Created**:
  - `createLogicClasses` table entry 447 (#447 `ApocalypseIslandDisplayGroup/ClockWheelMultiModel`)
  - entry 447 (listener = view controller; one tweenable binding). Init fetches the dials (entries 445, 444, 443 → `mCalendarDial1/2/3Logic`), plays `ApocalypseCalendarInit` and stores `mApocalypseTime = CFGregorianDateGetAbsoluteTime({2012, 12, 21, 23, 11, 0 s}, system time zone)`.
- **Listener**:
  - `<ApocalypseCalendarLogicListener>` (`mApocalypseCalendarLogicListener`): `isStoryActive`, `isTomPygmyActive`, `isApocalypseActive`, `endApocalypseComplete`, `endApocalypseCompleteNow`, `apocalypseGemPedestalLogicArray:` (never sent by this class: `isApocalypseActive`, `isStoryActive`)
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `ApocalypseGemPedestalLogic` (via `apocalypseGemPedestalLogicArray:`: `startTorchLit`, `lightTorch`, `elevateGem`, `unlightTorch`, `unlightTorchNow`), entry 446 (`SunPygmy` `MultiModelLogic`: `PygmySunInit`, `PygmySunWhatsUp`, `PygmySunCaptured`, `PygmySunPulledIn` from the `animSun…` callbacks), `IdolManager` (`BringTomToTempleEntrance` episode event); the view controller calls `isOpen`, `closeApocalypsePortal`.
  - also sent in code: `IdolManager` (`triggerEpisodeEvent:`).
  - `getLogic:` 443–446 `MultiModelLogic`.
- **Called from**: `closeApocalypsePortal` ← PocketGodViewController(Game); `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:`, `isOpen` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `ApocalypseCalendarClose`, `ApocalypseCalendarInit`, `ApocalypseCalendarOpen`, `ApocalypseCalendarStartOpen`, `PygmySunCaptured`, `PygmySunInit`, `PygmySunPulledIn`, `PygmySunWhatsUp`
  - objects: `ClockCenter`
  - sounds: `slide`
  - episode events: `BringTomToTempleEntrance`
  - other strings: `activateMyRainbow`, `activateRainbow`, `activateUnderwaterTemple`, `addDisplayGroup`, `addPygmy`, `addUnderwaterComplete`, `apeExitAfterIdle`, `apeGrabPygmyComplete`, `apeHitComplete`, `apocalypseFaderComplete`, `attack`, `battleCheckNoRunBack`, `battleEnergyFastMoveAboveOpponent`, `battleEnergyFastMoveTowardOpponent`, `battleFall`, `battleFastRunTowardOpponent`, `battleRunBackToStart`, `battleShowLavaFlame`, `battleStrongRunTowardOpponent`, `beheadPygmyComplete`, `blink`, `blinkLightning`, `blowUpPygmies`, `brawlAttackComplete`, `breakPygmyComplete`, `burstPygmyEarDrums`, `calendarCloseComplete`, `calendarOpenComplete`, `callForMom`, `cancelKidnapPygmyComplete`, `carryBone`, `carryFishOffScreen`, `carryPygmyOffScreen`, `catchPygmy`, `catchPygmyComplete`, `charlieCutPygmyInHalfComplete`, `charlieMonkeyIdle`, `charlieMonkeyWalk`, `chaseKonkeyDongComplete`, `chasePygmy`, … (513 in total)
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `lightTorches:::` @ 0x1000d7a94 (1 use), `elevateGems:::` @ 0x1000d7b88 (1 use), `unlightTorches:::` @ 0x1000d7c7c (1 use), `calendarOpenComplete:::` @ 0x1000d7d70 (1 use), `calendarCloseComplete:::` @ 0x1000d7d88 (1 use), `tweenClockHands:::` @ 0x1000d7dc4 (1 use), `untweenClockHands:::` @ 0x1000d7ddc (1 use), `hideClockHands:::` @ 0x1000d7df0 (1 use), `showClockHands:::` @ 0x1000d7ed4 (1 use).
- **Persistence**: none.
- **Behaviour**:
  - `onGameFrame:` @ 0x1000d7fb8: unless the hands are tweened (`tweenClockHands:::`), `Δ = mApocalypseTime − CFAbsoluteTimeGetCurrent()` seconds and the dial targets are `fmod(Δ·1.9577625570776256, 360)`, `fmod(Δ·0.003995433789954338, 360)`, `fmod(Δ·1.1415525114155251e−5, 360)` degrees (one turn per 183.9 s, 25.03 h, 365 days); each dial's `rot.z` eases toward its target taking the short way round (±360 unwrapping); tweened hands go to 0.
  - `enterIsland:` @ 0x1000d742c: shows `ClockCenter` and the dials; if not open and Tom is active → open, episode event `BringTomToTempleEntrance`, `ApocalypseCalendarOpen`, else `ApocalypseCalendarInit`; if already open → completely open, every pedestal `startTorchLit`, hands hidden, `ApocalypseCalendarStartOpen`.
  - `closeApocalypsePortal` → `ApocalypseCalendarClose` (its `calendarCloseComplete:::` reports `endApocalypseComplete`); `exitIsland:` while closing finishes at once (`endApocalypseCompleteNow`, torches out). The torch/gem handlers step through the pedestals every 0.5 s.

#### `ApocalypseCloudLogic`

`SpriteLogic` · `ApocalypseCloudLogic.c` · 8 methods + 2 item handlers + 3 class methods · 9 885 B

- **Role**: the plague (green) or purification (white) disease cloud that follows an infected sprite (pygmy or object).
- **Where**: wherever the infected sprite is.
- **Created**:
  - allocated in `ApocalypseCloudLogic::apocalypseCloudWithSprite:apocalypseDiseaseType:apocalypseCloudStyleId:` @ 0x1000c9590
  - only through `+apocalypseCloudWithSprite:apocalypseDiseaseType:apocalypseCloudStyleId:` @ 0x1000c9590 (from `SpriteLogic::infectWithDisease:`, `docs/re/logic.md` §11.4): a nameless `BCMultiModel` and `initWithDisplayObject:spriteLogic:apocalypseDiseaseType:apocalypseCloudStyleId:` @ 0x1000c97ec. The protocol `<ApocalypseCloudLogicListener>` is never set.
- **Listener**: `<ApocalypseCloudLogicListener>` (`mApocalypseCloudLogicListener`): no methods
- **Talks to**: the infected `SpriteLogic` (`infectedSpriteLogic`, `cureDisease`; for a `PygmyLogic` whose plague body is being carried the cloud follows the carrier, `plagueBodyCarryingPygmyLogic`), `IdolManager` (episode event `WashAwayAllDiseases`).
  - also sent in code: `IdolManager` (`triggerEpisodeEvent:`); `PygmyLogic` (`isPlagueBodyPickedUp`).
- **Called from**: `+apocalypseCloudWithSprite:apocalypseDiseaseType:apocalypseCloudStyleId:`, `deactivate`, `setApocalypseDiseaseType:` ← SpriteLogic; `+cureAllDisease`, `+doesDiseaseExist` ← PocketGodViewController; `setDragXOffset:dragYOffset:` ← PygmyLogic, SpriteLogic.
- **Data ids**:
  - episode events: `WashAwayAllDiseases`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `die:::` @ 0x1000c9c60 (119 uses), `setAnimationBasedOnType:::` @ 0x1000c9df4 (17 uses).
- **Persistence**: none.
- **Behaviour**: every live cloud is kept in the global array `DAT_1007d1f00` (`+doesDiseaseExist`; `+cureAllDisease` fires `WashAwayAllDiseases` and cures every infected sprite). Init copies the sprite position, inserts the cloud right after the sprite and plays `ApocalypseCloud<style>Activate` (styles `Default`, `Wood`, `OutHouse`, `Igloo`, `ApeAltar`, `ApeBongo`, `ApeWench`, `FishingPole` exist as sequences; the `DDE` style of entry 500 has no sequence, so nothing plays). `setApocalypseDiseaseType:` replays the activate behaviour; the `setAnimationBasedOnType:::` item picks the plague/purification look. `onGameFrame:` @ 0x1000c9b34 follows the sprite (plus `setDragXOffset:dragYOffset:`). `deactivate` removes it from the global array.

#### `ApocalypseGemLogic`

`SpriteLogic` · `ApocalypseGemLogic.c` · 25 methods + 7 item handlers · 32 019 B

- **Role**: the seven apocalypse gems (Time, Locust, Dance, Plague, Purification, Closet, Xenu), earned in the temples and placed on the matching pedestals to end the apocalypse.
- **Where**: apocalypse island and temples; entries 403–409 (`ApocalypseGemTime…Xenu`, types 1–7).
- **Created**:
  - `createLogicClasses` table entries 403–409 (new `BCMultiModel` `ApocalypseGemTime`, new `BCMultiModel` `ApocalypseGemLocust`, new `BCMultiModel` `ApocalypseGemDance`, … (7 entries)); init steps: `play_animation ApocalypseGem1InitAnim`, `play_animation ApocalypseGem2InitAnim`, `play_animation ApocalypseGem3InitAnim`, `play_animation ApocalypseGem4InitAnim`, `play_animation ApocalypseGem5InitAnim`, `play_animation ApocalypseGem6InitAnim`, `play_animation ApocalypseGem7InitAnim`, `set_apocalypse_gem_type 1`, `set_apocalypse_gem_type 2`, `set_apocalypse_gem_type 3`, `set_apocalypse_gem_type 4`, `set_apocalypse_gem_type 5`, `set_apocalypse_gem_type 6`, `set_apocalypse_gem_type 7`
  - listener = view controller; the view controller's `onApocalypseGemEventTriggered:` @ 0x10016544c activates gem *n* (`getLogic: 0x193 + n − 1`).
- **Listener**:
  - `<ApocalypseGemLogicListener>` (`mApocalypseGemLogicListener`): `isStoryActive`, `isGravityEnabled`, `isOceanEnabled`, `isApocalypseGemPedestalEnabled`, `isApocalypseActive`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyDragDisplayMarker`, `pygmyDisplayMarker`, `apocalypseGemPedestalLogicArray:`, `onApocalypseGemDrag:`, `onApocalypseGemDragComplete:`, `onApocalypseGemDie:` (never sent by this class: `isApocalypseActive`, `islandCenterXPos`)
  - wired by `initLogicListeners` to `PocketGodViewController` (7)
- **Talks to**: `ApocalypseGemPedestalLogic` (`apocalypseGemFallIntoApocalypseGemPedestal:`, `ejectApocalypseGemFromApocalypseGemPedestal:`; pedestals call `apocalypseGemType`, `ejectFromApocalypseGemPedestal`), the listener's markers (`pygmyDisplayMarker`, `pygmyDragDisplayMarker`).
  - also sent in code: `ApocalypseGemPedestalLogic` (`isAvailable`); `IdolManager` (`triggerEpisodeEvent:`).
- **Called from**: `apocalypseGemType`, `ejectFromApocalypseGemPedestal` ← ApocalypseGemPedestalLogic; `prepareToEndApocalypse` ← PocketGodViewController(Game); `setApocalypseGemType:` ← BCLibrary(Logic).
- **Data ids**:
  - behaviours: `ApocalypseGemBounce`, `ApocalypseGemDie`, `ApocalypseGemDrag`, `ApocalypseGemEnterIsland`, `ApocalypseGemFall`, `ApocalypseGemFallIgnoreIsland`, `ApocalypseGemFallIntoApocalypseGemPedestal`, `ApocalypseGemSplash`, `ApocalypseGemStopSlide`, `ApocalypseGemTiltSlide`
  - episode events: `ThrowGemAway`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `slide:::` @ 0x10000f9b4 (20 uses), `setAnimationBasedOnGemType:::` @ 0x10000f9e0 (5 uses), `resetLastPos:::` @ 0x10000fab4 (5 uses), `drag:::` @ 0x10000fb04 (40 uses), `fall:::` @ 0x10000fcb4 (134 uses), `checkRespawn:::` @ 0x10000fe9c (1 use), `die:::` @ 0x10000fea4 (119 uses).
- **Persistence**: none.
- **Behaviour**: Shared pattern of §2.2 with slide friction 0.04, floor `y = 92`, box (−20,−10)–…, `ApocalypseGemBounce`/`ApocalypseGemDie`; spring drag k 100, damping 8 with rotation. `activate` drops it into the scene (gravity off until it slides). `checkForApocalypseGemPedestalDuringFall` lands a falling gem in a free pedestal it touches (the `fall:::` handler); falling in the sea plays `ApocalypseGemSplash` and the episode event `ThrowGemAway`. A tap on a seated gem ejects it. `prepareToEndApocalypse` makes it unselectable. `setAnimationBasedOnGemType:::` picks the look.

#### `ApocalypseGemPedestalLogic`

`SpriteLogic` · `ApocalypseGemPedestalLogic.c` · 18 methods + 2 item handlers · 11 896 B

- **Role**: the seven gem pedestals (with fire and symbol effects) on the apocalypse island; all seven correct gems end the apocalypse.
- **Where**: apocalypse island, entries 433–439 (`ApocalypseIslandDisplayGroup/Pedestal1…7`, types 1–7; each with 3 tweenable bindings to the fire/symbol multimodels 412–432).
- **Created**:
  - `createLogicClasses` table entries 433–439 (`ApocalypseIslandDisplayGroup/Pedestal1`, `ApocalypseIslandDisplayGroup/Pedestal2`, `ApocalypseIslandDisplayGroup/Pedestal3`, … (7 entries)); init steps: `play_animation ApocalypseGemColumnInitAnim` ×7, `set_apocalypse_gem_type 1`, `set_apocalypse_gem_type 2`, `set_apocalypse_gem_type 3`, `set_apocalypse_gem_type 4`, `set_apocalypse_gem_type 5`, `set_apocalypse_gem_type 6`, `set_apocalypse_gem_type 7`
  - listener = view controller; init stops its game frame.
- **Listener**:
  - `<ApocalypseGemPedestalLogicListener>` (`mApocalypseGemPedestalLogicListener`): `isStoryActive`, `isApocalypseActive`, `apocalypseGemPedestalLogicArray:`, `endApocalypse` (never sent by this class: `isStoryActive`)
  - wired by `initLogicListeners` to `PocketGodViewController` (7)
- **Talks to**: `ApocalypseGemLogic` (gem bound on channel 2), `ApocalypseCalendarLogic` (torch/elevate commands), `IdolManager` (episode events `TimeGemInPedestal`, `LocustGemInPedestal`, `DanceGemInPedestal`, `PlagueGemInPedestal`, `PurificationGemInPedestal`, `ClosetGemInPedestal`, `XenuGemInPedestal` when the matching gem lands).
  - also sent in code: `ApocalypseGemLogic` (`apocalypseGemType`, `ejectFromApocalypseGemPedestal`); `IdolManager` (`triggerEpisodeEvent:`).
- **Called from**: `apocalypseGemFallIntoApocalypseGemPedestal:`, `ejectApocalypseGemFromApocalypseGemPedestal:`, `isAvailable` ← ApocalypseGemLogic; `elevateGem`, `lightTorch`, `startTorchLit`, `unlightTorch`, `unlightTorchNow` ← ApocalypseCalendarLogic; `setApocalypseGemType:` ← BCLibrary(Logic).
- **Data ids**:
  - behaviours: `ApocalypseGemPedestalApocalypseGemFall`, `ApocalypseGemPedestalApocalypseGemFallMatch`, `ApocalypseGemPedestalEjectApocalypseGem`, `ApocalypseGemPedestalEjectApocalypseGemApocalypse`, `ApocalypseGemPedestalElevateGem`, `ApocalypseGemPedestalEnterIsland`, `ApocalypseGemPedestalExitIsland`, `ApocalypseGemPedestalLightTorch`, `ApocalypseGemPedestalTorchLit`, `ApocalypseGemPedestalTorchLitApocalypse`, `ApocalypseGemPedestalUnlightTorch`
  - other strings: `ClosetGemInPedestal`, `DanceGemInPedestal`, `LocustGemInPedestal`, `PlagueGemInPedestal`, `PurificationGemInPedestal`, `TimeGemInPedestal`, `XenuGemInPedestal`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `unlightTorchComplete:::` @ 0x100069a3c (1 use), `setAnimationBasedOnGemType:::` @ 0x100069ab4 (5 uses).
- **Persistence**: none.
- **Behaviour**: `apocalypseGemFallIntoApocalypseGemPedestal:` @ 0x100069464: bind the gem; wrong type → `ApocalypseGemPedestalApocalypseGemFall`; right type → episode event, then (when all pedestals hold their gems) `endApocalypse` to the listener, `ApocalypseGemPedestalApocalypseGemFallMatch`. `hasCorrectGem`, `isAvailable`. `startTorchLit` picks `…TorchLit`, `…TorchLitApocalypse`, `…ApocalypseGemFallMatch` or `…ApocalypseGemFall`. `lightTorch`, `elevateGem`, `unlightTorch`, `unlightTorchNow`, eject (`…EjectApocalypseGem[Apocalypse]`).

#### `ApocalypseRoomFaderLogic`

`SpriteLogic` · `ApocalypseRoomFaderLogic.c` · 7 methods + 1 item handler · 2 813 B

- **Role**: the full-screen fade used to move between apocalypse rooms.
- **Where**: entry 440 (`ApocalypseRoomFader`).
- **Created**:
  - `createLogicClasses` table entry 440 (#440 new `BCMultiModel` `ApocalypseRoomFader`)
  - listener = view controller.
- **Listener**:
  - `<ApocalypseRoomFaderLogicListener>` (`mApocalypseRoomFaderLogicListener`): `islandLocationState`, `onApocalypseRoomFaderChangeIslandLocationState:` (never sent by this class: `islandLocationState`)
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Called from**: `activate:` ← ApocalypseTempleDoorLogic.
- **Data ids**:
  - behaviours: `ApocalypseRoomFaderActivate`
  - markers: `MapDisplayMarker`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `apocalypseFaderComplete:::` @ 0x1001784c0 (1 use).
- **Persistence**: none.
- **Behaviour**: `activate: location` inserts it after `MapDisplayMarker` and plays `ApocalypseRoomFaderActivate`; its animation calls `animChangeLocations` → `onApocalypseRoomFaderChangeIslandLocationState: location`; `apocalypseFaderComplete:::` removes it. Used by the temple doors (`ApocalypseTempleDoorLogic`'s `onPygmyFallInsideComplete:::` activates entry 440).

#### `ApocalypseTempleDoorLogic`

`SpriteLogic` · `ApocalypseTempleDoorLogic.c` · 21 methods + 3 item handlers · 11 127 B

- **Role**: the doors linking the apocalypse rooms: tap to open/close, and a pygmy that falls into an open door travels to the linked room.
- **Where**: entries 448 (temple entrance → location 11, type 0), 449 (portal → 17, type 1, always open), 457/456 (Time temple: → 10 type 2, → 12 type 3), 470/471/472 (Locust: → 11 type 4, → 13 type 5, → 14 type 5), 475/476 (Dance: → 12 type 6, → 16 type 7), 504/505 (Plague: → 12 type 8, → 15 type 9), 508 (Purification: → 14 type 10), 515 (Closet: → 13 type 12), 519 (Xenu: → 10 type 13); all but 448 always open.
- **Created**:
  - `createLogicClasses` table entries 448–449, 456–457, 470–472, 475–476, 504–505, 508, 515, 519 (`ApocalypseIslandDisplayGroup/TempleDoor`, `ApocalypseIslandDisplayGroup/TemplePortal`, `TimeTempleDisplayGroup/Door2`, … (14 entries)); init steps: `play_animation ApocalypseIslandTempleDoorInitAnim`, `play_animation ApocalypsePortalInitAnim`, `set_always_open True` ×13, `set_apocalypse_door_type 0`, `set_apocalypse_door_type 1`, `set_apocalypse_door_type 10`, `set_apocalypse_door_type 12`, `set_apocalypse_door_type 13`, `set_apocalypse_door_type 2`, `set_apocalypse_door_type 3`, `set_apocalypse_door_type 4`, `set_apocalypse_door_type 5` ×2, `set_apocalypse_door_type 6`, `set_apocalypse_door_type 7`, `set_apocalypse_door_type 8`, `set_apocalypse_door_type 9`, `set_island_location_state 10` ×2, `set_island_location_state 11` ×2, `set_island_location_state 12` ×3, `set_island_location_state 13` ×2, `set_island_location_state 14` ×2, `set_island_location_state 15`, `set_island_location_state 16`, `set_island_location_state 17`
  - table (init steps `setIslandLocationState:`, `setApocalypseDoorType:`, `setAlwaysOpen:`); listener = view controller.
- **Listener**:
  - `<ApocalypseTempleDoorLogicListener>` (`mApocalypseTempleDoorLogicListener`): `isStoryActive`, `islandLocationState`, `pygmyLogicArray:` (never sent by this class: `pygmyLogicArray:`)
  - wired by `initLogicListeners` to `PocketGodViewController` (14)
- **Talks to**: `PygmyLogic` (`fallIntoApocalypseTempleDoor:`, `ejectFromApocalypseTempleDoor:`, `isAvailable`; the door reads the held pygmy's `timeTemplePygmyType`), `ApocalypseRoomFaderLogic` (entry 440 `activate:`).
  - also sent in code: `PygmyLogic` (`ejectFromApocalypseTempleDoor`).
  - `getLogic:` 440 `ApocalypseRoomFaderLogic`.
- **Called from**: `ejectFromApocalypseTempleDoor:` ← PygmyLogic, PygmyLogic(Apocalypse); `enterIsland:`, `exitIsland:`, `exitIslandNow` ← PocketGodViewController; `fallIntoApocalypseTempleDoor:`, `isAvailable` ← PygmyLogic(Island); `setAlwaysOpen:`, `setApocalypseDoorType:`, `setIslandLocationState:` ← BCLibrary(Logic).
- **Data ids**:
  - behaviours: `ApocalypseTempleDoorClose`, `ApocalypseTempleDoorEjectPygmy`, `ApocalypseTempleDoorEnterIsland`, `ApocalypseTempleDoorExitIsland`, `ApocalypseTempleDoorOpen`, `ApocalypseTempleDoorPygmyFallClosetTempleSideLeft`, `ApocalypseTempleDoorPygmyFallDanceTempleSideLeft`, `ApocalypseTempleDoorPygmyFallDanceTempleSideRight`, `ApocalypseTempleDoorPygmyFallEntrance`, `ApocalypseTempleDoorPygmyFallLocustTempleSideLeft`, `ApocalypseTempleDoorPygmyFallLocustTempleSideRight`, `ApocalypseTempleDoorPygmyFallPlagueTempleSideLeft`, `ApocalypseTempleDoorPygmyFallPlagueTempleSideRight`, `ApocalypseTempleDoorPygmyFallPortal`, `ApocalypseTempleDoorPygmyFallPurificationTempleSideLeft`, `ApocalypseTempleDoorPygmyFallPurificationTempleSideRight`, `ApocalypseTempleDoorPygmyFallTimeTempleSideLeft`, `ApocalypseTempleDoorPygmyFallTimeTempleSideRight`, `ApocalypseTempleDoorPygmyFallXenuTempleSideLeft`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `onPygmyFallInsideComplete:::` @ 0x100136350 (16 uses), `setAnimationBasedOnPygmyType:::` @ 0x100136420 (56 uses), `setAnimationBasedOnDoorType:::` @ 0x100136504 (4 uses).
- **Persistence**: none.
- **Behaviour**: init stops its game frame. `isAvailable` = no pygmy ∧ (open ∨ always open). A tap (not always open, no story) toggles `ApocalypseTempleDoorOpen` / `…Close`. `fallIntoApocalypseTempleDoor:` binds the pygmy on channel 1 and plays `ApocalypseTempleDoorPygmyFall<Entrance|Portal|TimeTempleSideLeft|…>` by door type; `onPygmyFallInsideComplete:::` sends the fader to the door's location and ejects the pygmy (`ApocalypseTempleDoorEjectPygmy`). Item handlers choose animations by door type and by the pygmy's time-temple type/disease (sequences.md §5).

#### `ApocalypseTempleFountainLogic`

`SpriteLogic` · `ApocalypseTempleFountainLogic.c` · 16 methods + 1 item handler + 1 class method · 7 617 B

- **Role**: the plague and purification fountains: pygmies dropped in (three slots) are infected or purified.
- **Where**: Plague temple (entry 503, disease 1) and Purification temple (entry 507, disease 2).
- **Created**:
  - `createLogicClasses` table entries 503, 507 (#503 `PlagueTempleDisplayGroup/PlagueFountain`; #507 `PurificationTempleDisplayGroup/PurificationFountain`); init steps: `set_apocalypse_disease_type 1`, `set_apocalypse_disease_type 2`
  - table (`setApocalypseDiseaseType:`); listener = view controller. `+instance` returns `DAT_100821f78`, which holds whichever fountain was created last (entry 507).
- **Listener**:
  - `<ApocalypseTempleFountainLogicListener>` (`mApocalypseTempleFountainLogicListener`): `isStoryActive` (never sent by this class: `isStoryActive`)
  - wired by `initLogicListeners` to `PocketGodViewController` (2)
- **Talks to**: `PygmyLogic(Apocalypse)` (`isSlot1/2/3Available`, `fallIntoApocalypseTempleFountainSlot1/2/3:`, `ejectFromApocalypseTempleFountain:`; fountain sends `ejectFromApocalypseTempleFountain`).
- **Called from**: `ejectFromApocalypseTempleFountain:` ← PygmyLogic, PygmyLogic(Apocalypse); `isSlot1Available`, `isSlot2Available`, `isSlot3Available` ← PygmyLogic(Apocalypse); `setApocalypseDiseaseType:` ← BCLibrary(Logic).
- **Data ids**:
  - behaviours: `ApocalypseTempleFountainEjectPygmy`, `ApocalypseTempleFountainEnterIsland`, `ApocalypseTempleFountainExitIsland`, `ApocalypseTempleFountainPygmyFall`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `setAnimationBasedOnType:::` @ 0x1001a1bd4 (17 uses).
- **Persistence**: none.
- **Behaviour**: slot *n* binds the pygmy on channel *n* (`ApocalypseTempleFountainPygmyFall`); eject clears it (`ApocalypseTempleFountainEjectPygmy`); `enterIsland:` → `ApocalypseTempleFountainEnterIsland`; `exitIsland:` ejects all three. `setAnimationBasedOnType:::` picks plague/purification looks.

#### `ApocalypseTimeTempleLeverLogic`

`SpriteLogic` · `ApocalypseTimeTempleLeverLogic.c` · 16 methods · 7 372 B

- **Role**: the lever of the Time temple that slides the transformation platform between its four stages.
- **Where**: Time temple (location 11), entry 454 (`TimeTempleDisplayGroup/Lever`).
- **Created**:
  - `createLogicClasses` table entry 454 (#454 `TimeTempleDisplayGroup/Lever`)
  - listener = view controller.
- **Listener**:
  - `<ApocalypseTimeTempleLeverLogicListener>` (`mApocalypseTimeTempleLeverLogicListener`): `isStoryActive`, `apocalypseTimeTemplePlatformLogic:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `ApocalypseTimeTemplePlatformLogic` (via `apocalypseTimeTemplePlatformLogic:`: `movePlatform:`, `stopMovingPlatform`), `IdolManager` (`PullTheLever` on grab).
  - also sent in code: `IdolManager` (`triggerEpisodeEvent:`).
- **Called from**: `enterIsland:`, `exitIsland:`, `exitIslandNow` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `ApocalypseTimeTempleLeverEnterIsland`, `ApocalypseTimeTempleLeverExitIsland`
  - episode events: `PullTheLever`
- **Persistence**: none.
- **Behaviour**: `onGameFrame:` @ 0x100115698: released → frame percent eases back to 0.5 (50 %/frame); held → `d = clamp(startDrag − drag, −100, 100)` maps to the lever frame and the platform `movePlatform:` speed; release → `stopMovingPlatform`.

#### `ApocalypseTimeTemplePlatformLogic`

`SpriteLogic` · `ApocalypseTimeTemplePlatformLogic.c` · 15 methods + 2 item handlers + 1 class method · 9 160 B

- **Role**: the Time temple platform: a pygmy on it is transformed into an old-fart, baby, ginger or fast pygmy depending on the stage it is slid to, when the energy button is pressed.
- **Where**: Time temple, entry 453 (`TimeTempleDisplayGroup/Platform`); energy button entry 455 (listener = the platform).
- **Created**:
  - `createLogicClasses` table entry 453 (#453 `TimeTempleDisplayGroup/Platform`)
  - `+instance` returns `DAT_100821f50`; listener = view controller.
- **Listener**:
  - `<ApocalypseTimeTemplePlatformLogicListener>` (`mApocalypseTimeTemplePlatformLogicListener`): `isStoryActive`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `PygmyLogic(Apocalypse)` (`fallOntoApocalypseTimeTemplePlatform:`, `ejectFromApocalypseTimeTemplePlatform:`, `timeTemplePygmyType`), `IdolManager` (`CreateOldFartPygmy` / `CreateBabyPygmy` / `CreateGingerPygmy` / `CreateFastPygmy` in `onTransformComplete:::`).
  - also sent in code: `IdolManager` (`triggerEpisodeEvent:`); `PygmyLogic` (`ejectFromApocalypseTimeTemplePlatform`, `setTimeTemplePygmyType:`).
- **Called from**: `ejectFromApocalypseTimeTemplePlatform:` ← PygmyLogic, PygmyLogic(Apocalypse); `fallOntoApocalypseTimeTemplePlatform:`, `isAvailable` ← PygmyLogic(Apocalypse); `movePlatform:`, `stopMovingPlatform` ← ApocalypseTimeTempleLeverLogic.
- **Data ids**:
  - behaviours: `ApocalypseTimeTemplePlatformEjectPygmy`, `ApocalypseTimeTemplePlatformEnergy`, `ApocalypseTimeTemplePlatformEnergyPygmy`, `ApocalypseTimeTemplePlatformEnterIsland`, `ApocalypseTimeTemplePlatformExitIsland`, `ApocalypseTimeTemplePlatformPygmyFall`
  - episode events: `CreateBabyPygmy`, `CreateFastPygmy`, `CreateGingerPygmy`, `CreateOldFartPygmy`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `setAnimationBasedOnPygmyType:::` @ 0x100182884 (56 uses), `onTransformComplete:::` @ 0x100182968 (1 use).
- **Persistence**: none.
- **Behaviour**: stages at `x = 145 + 66.666664·k` (k = 0…3). `movePlatform: p` moves `x += p·100·dt` clamped to [145, 345]; `stopMovingPlatform` snaps to the nearest stage (`mActiveStage = (int)((x − 145)/66.666664 + 0.5)`), eased 50 %/frame. The energy button → `ApocalypseTimeTemplePlatformEnergy` or `…EnergyPygmy` (no story); `onTransformComplete:::` sets the pygmy's type to stage + 1 (toggling back to normal when it already has it).

#### `LocustLogic`

`SpriteLogic` · `LocustLogic.c` · 35 methods + 3 item handlers · 38 822 B

- **Role**: the locust swarm (up to 60 locusts) of the Locust temple: born from the queen, they buzz around, follow a path the player draws, catch, lift and throw pygmies, and strip them to bones. Owns **`LocustModel`** (point sprites; `docs/re/rendering.md` §8.3).
- **Where**: Locust temple (location 12) and wherever the swarm is led.
- **Created**:
  - allocated in `PocketGodViewController::loadGame` @ 0x10006a030
  - `PocketGodViewController::loadGame` first builds `LocustModel initWithId:@"Locust"` (`z = −277.12`), then the logic → `mLocustLogic`, listener = view controller.
- **Listener**:
  - `<LocustLogicListener>` (`mLocustLogicListener`): `pygmyLogicArray:`, `locustDisplayMarker`, `startDrawLocust:`, `drawLocust:`, `stopDrawLocust:`
  - set in `PocketGodViewController::loadGame` @ 0x10006a030
- **Talks to**: `LocustQueenLogic` (`addLocust:`, `canCreateNewLocusts`), `PygmyLogic` (`catchInAirByLocusts:`, `startThrownInAirByLocusts:`, `thrownInAirByLocusts:targetPos:targetTime:`, `droppedInAirByLocusts:`, `cutInAirByLocusts:`, `mouthFilledByLocusts:`, `stripFleshByLocusts:`; pygmies call `shouldPygmyPanic` and `stopBeingThrownByLocusts:`), `LocustModel` (`setLocustPointArray:count:`), entries 462/463 (`LocustSlash`, `LocustSplat` effects); the view controller drives `startDrawLocust:startPos:`, `drawLocust:`, `drawLocustComplete`, `deactivateLocusts`.
  - also sent in code: `IdolManager` (`triggerEpisodeEvent:`).
  - `getLogic:` 462–463 `MultiModelLogic`.
- **Called from**: `addLocust:`, `canCreateNewLocusts` ← LocustQueenLogic; `deactivateLocusts`, `drawLocust:`, `drawLocustComplete`, `isLocustActive`, `startDrawLocust:startPos:` ← PocketGodViewController(Game); `shouldPygmyPanic`, `stopBeingThrownByLocusts:` ← PygmyLogic(Apocalypse).
- **Data ids**:
  - behaviours: `LocustIdle`, `LocustSlash`, `LocustSplat`, `LocustStartBuzzing`, `LocustStartThrowPygmy`, `LocustStopBuzzing`, `LocustThrowPygmy`
  - episode events: `DrawLocustLine`, `GatherLocusts`, `LocustsDropPygmy`, `LocustsDropPygmyBored`, `LocustsLeaveBored`, `MaxLocusts`, `PushLocustsOffScreen`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `throwPygmy:::` @ 0x10008d44c (1 use), `liftPygmy:::` @ 0x10008d588 (1 use), `catchPygmy:::` @ 0x10008d694 (2 uses).
- **Persistence**: none.
- **Behaviour**:
  - Global state: 60 locust records of 0xB0 bytes at `0x1007e50a8` (`LocustState`: state, follow index, path index, position, heading, flags, path distance …) and the 60-point output array at `0x1007e79e8`; `sendLocustPosToModel` @ 0x10008c628 packs the active locusts' (x, y) into it each frame.
  - Per-locust state machine `runLocustState:deltaTime:` @ 0x10008c4e8: 0 inactive, 1 idle (heading wanders ±30° per frame, speed 300, re-enters from the edges; `checkForLocustStateInactive:` stops buzzing — `LocustStopBuzzing` — when none is active after the exit timer), gather (fly to the path start, within 40 px join it with a follow index), 3 follow (move along the drawn path, `std::vector<VECTOR4>` `mpLocustVector`, speed from `DAT_10029d828` by swarm mode), 4 hover (at the path end), 5 throw-gather (swarm around the thrown pygmy's target with `mGatherSpeedScale`).
  - Drawing (`startDrawLocust:startPos:`, `drawLocust:` @ 0x10008cdc4, `drawLocustComplete`) builds the path and its length; the listener's `startDrawLocust:`/`drawLocust:`/`stopDrawLocust:` are notified. Item handlers `catchPygmy:::`, `liftPygmy:::`, `throwPygmy:::` carry a pygmy (sequences.md §5). `addLocust: pos` activates a free record at the queen's birth point; `canCreateNewLocusts` / `activeLocustCount` bound the swarm.

#### `LocustModel`

`BCDisplayObject` · `LocustModel.c` · 4 methods · 1 619 B

- **Role**: custom display object drawing the locust swarm as point sprites from the array set by `setLocustPointArray:count:` (`docs/re/rendering.md` §8.3). Owned by **`LocustLogic`**.
- **Created**:
  - allocated in `PocketGodViewController::loadGame` @ 0x10006a030
  - `PocketGodViewController::loadGame` (`initWithId:@"Locust"`, `z = −277.12`).
- **Listener**: none.
- **Called from**: `setLocustPointArray:count:` ← LocustLogic.
- **Persistence**: none.

#### `LocustQueenLogic`

`SpriteLogic` · `LocustQueenLogic.c` · 19 methods + 1 item handler + 1 class method · 9 017 B

- **Role**: the locust queen of the Locust temple: holding a finger on her births locusts while she has embryos; a pygmy dropped in her arms is eaten and refills her with 20 embryos.
- **Where**: Locust temple, entry 461 (`LocustTempleDisplayGroup/Queen`, animation `LocustQueenInitAnim`); belly multimodel entry 460.
- **Created**:
  - `createLogicClasses` table entry 461 (#461 `LocustTempleDisplayGroup/Queen`); init steps: `play_animation LocustQueenInitAnim`
  - `+instance` returns `DAT_100821f68`; listener = view controller.
- **Listener**:
  - `<LocustQueenLogicListener>` (`mLocustQueenLogicListener`): `isStoryActive`, `locustLogic:`, `onLocustQueenVisible:` (never sent by this class: `isStoryActive`)
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `LocustLogic` (via `locustLogic:`: `addLocust:`, `canCreateNewLocusts`), `PygmyLogic(Apocalypse)` (`fallIntoLocustQueenArms:`, `rescuedFromLocustQueen:`; queen sends `rescuedFromLocustQueen`), the belly logic (`LocustQueenBellyEnterIsland` / `LocustQueenBellyPregnant`).
  - also sent in code: `PygmyLogic` (`eatenByLocustQueenComplete:`).
  - `getLogic:` 460 `MultiModelLogic`.
- **Called from**: `fallIntoLocustQueenArms:`, `isAvailable` ← PygmyLogic(Apocalypse); `rescuedFromLocustQueen:` ← PygmyLogic, PygmyLogic(Apocalypse).
- **Data ids**:
  - behaviours: `LocustQueenAttemptBirthLocusts`, `LocustQueenBellyEmpty`, `LocustQueenBellyEnterIsland`, `LocustQueenBellyPregnant`, `LocustQueenBirthLocustsComplete`, `LocustQueenEjectPygmy`, `LocustQueenEnterIsland`, `LocustQueenExitIsland`, `LocustQueenPygmyFall`, `LocustQueenStartBirthLocusts`, `LocustQueenStopBirthLocusts`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `eatPygmyComplete:::` @ 0x100199dc8 (4 uses).
- **Persistence**: none.
- **Behaviour**: `enterIsland:` → `onLocustQueenVisible:`, `LocustQueenEnterIsland`, belly by embryo count. Touch down (no pygmy held) → `LocustQueenStartBirthLocusts` while embryos remain and locusts can be created; release → `LocustQueenStopBirthLocusts`. `animBirthLocust` (animation callback): embryos − 1, a locust is added at `pos + (−108, 140)`. `eatPygmyComplete:::`: +20 embryos (pregnant belly) when locusts can be created. `fallIntoLocustQueenArms:` binds the pygmy on channel 1 (`LocustQueenPygmyFall`).

#### `ClosetTempleCouchLogic`

`SpriteLogic` · `ClosetTempleCouchLogic.c` · 11 methods + 1 class method · 3 559 B

- **Role**: the therapist's couch of the Closet temple: one pygmy can jump or fall onto it.
- **Where**: Closet temple (location 16), entry 512 (`ClosetTempleDisplayGroup/Couch`, animation `CouchInitAnim`).
- **Created**:
  - `createLogicClasses` table entry 512 (#512 `ClosetTempleDisplayGroup/Couch`); init steps: `play_animation CouchInitAnim`
  - `+instance` returns `DAT_100821f48`; listener = view controller (protocol empty); init stops its game frame.
- **Listener**:
  - `<ClosetTempleCouchLogicListener>` (`mClosetTempleCouchLogicListener`): no methods
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `PygmyLogic(Apocalypse)` (`isAvailable`, `jumpOntoClosetTempleCouch:`, `fallOntoClosetTempleCouch:`, `ejectFromClosetTempleCouch:`; couch sends `ejectFromClosetTempleCouch`).
  - also sent in code: `PygmyLogic` (`personaPersonality`).
- **Called from**: `ejectFromClosetTempleCouch:` ← PygmyLogic, PygmyLogic(Apocalypse); `enterIsland:`, `exitIsland:`, `exitIslandNow` ← PocketGodViewController; `fallOntoClosetTempleCouch:`, `jumpOntoClosetTempleCouch:` ← PygmyLogic(Apocalypse); `isAvailable` ← PygmyLogic(Apocalypse), PygmyLogic(Island).
- **Data ids**:
  - behaviours: `ClosetTempleCouchEnterIsland`, `ClosetTempleCouchExitIsland`
- **Persistence**: none.
- **Behaviour**: holds one pygmy reference (no animation binding); `enterIsland:` → `ClosetTempleCouchEnterIsland`; `exitIsland:` ejects the pygmy unless its personality is 2 (Tom), `ClosetTempleCouchExitIsland`.

#### `ClosetTempleClosetDoorLogic`

`SpriteLogic` · `ClosetTempleClosetDoorLogic.c` · 25 methods + 1 class method · 13 861 B

- **Role**: the sliding closet door of the Closet temple: Tom (the pygmy from the future) hides in the closet; the player slides the door open (thetans can hold it open) and a pygmy can be dropped inside.
- **Where**: Closet temple, entry 513 (`ClosetTempleDisplayGroup/SlidingDoor`, animation `ClosetDoorInitAnim`).
- **Created**:
  - `createLogicClasses` table entry 513 (#513 `ClosetTempleDisplayGroup/SlidingDoor`); init steps: `play_animation ClosetDoorInitAnim`
  - `+instance` returns `DAT_1007e3010`; listener = view controller; init stops its game frame.
- **Listener**:
  - `<ClosetTempleClosetDoorLogicListener>` (`mClosetTempleClosetDoorLogicListener`): `closetTempleThetanMeterLogic:`, `isTomPygmyAlive`, `createTomPygmy`, `onClosetTempleClosetDoorDrag:`, `onClosetTempleClosetDoorDragComplete:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `ClosetTempleThetanMeterLogic` (via `closetTempleThetanMeterLogic:`: `areThetansActive`, `holdClosetDoor`, `stopHoldingClosetDoor`), the listener's Tom (`isTomPygmyAlive`, `createTomPygmy` → `grabNewTom`, pygmy `startInClosetTempleClosetDoor`), `PygmyLogic(Apocalypse)` (`isAvailable`, `fallIntoClosetTempleClosetDoor:`, `ejectFromClosetTempleClosetDoor:`, `minClosetXPos`, `maxClosetXPos`, `closetYPos`).
  - also sent in code: `IdolManager` (`triggerEpisodeEvent:`); `PygmyLogic` (`canSlamClosetDoor`, `ejectFromClosetTempleClosetDoor`, `personaPersonality`).
- **Called from**: `closetYPos`, `fallIntoClosetTempleClosetDoor:`, `maxClosetXPos`, `minClosetXPos`, `slamDoor` ← PygmyLogic(Apocalypse); `ejectFromClosetTempleClosetDoor:` ← PygmyLogic, PygmyLogic(Apocalypse); `grabNewTom` ← PygmyLogic; `isAvailable` ← PygmyLogic(Apocalypse), PygmyLogic(Island).
- **Data ids**:
  - behaviours: `ClosetTempleClosetDoorEnterIsland`, `ClosetTempleClosetDoorExitIsland`
  - episode events: `OpenClosetDoor`, `SlideClosetDoor`
- **Persistence**: none.
- **Behaviour**: the closed x is remembered on the first `enterIsland:` (which creates Tom when he is not alive); the closet interior spans `[closedX + 25, closedX + 195]` at `y + 15`. Touch-drag slides the door (`onGameFrame:` @ 0x10000b510: the door follows the finger's x offset within its range, eases back to closed when released unless held open by thetans; `onClosetTempleClosetDoorDrag:` / `…DragComplete:`); `slamDoor` closes it and ends a drag; `isAvailable` = open ∧ empty; `exitIsland:` ejects a non-Tom pygmy.

#### `ClosetTempleThetanMeterLogic`

`SpriteLogic` · `ClosetTempleThetanMeterLogic.c` · 33 methods + 1 item handler + 1 class method · 20 953 B

- **Role**: the "thetan meter" (e-meter parody) of the Closet temple: tapping it with a pygmy on it raises the level until thetans burst out of the pygmy; the thetans fly around, possess pygmies and can hold the closet door open.
- **Where**: Closet temple, entry 514 (`ClosetTempleDisplayGroup/ThetanMeter`, animation `ThetanMeterInitAnim`).
- **Created**:
  - `createLogicClasses` table entry 514 (#514 `ClosetTempleDisplayGroup/ThetanMeter`); init steps: `play_animation ThetanMeterInitAnim`
  - `+instance` returns `DAT_100821f70`; listener = view controller; init stops its game frame.
- **Listener**:
  - `<ClosetTempleThetanMeterLogicListener>` (`mClosetTempleThetanMeterLogicListener`): `pygmyLogicArray:`, `locustDisplayMarker`, `isStoryActive`, `isApocalypseActive`, `onThetanDrag:`, `onThetanDragComplete:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `ThetanLogic` (8 per spawn, `Thetan` multimodels after the listener's `locustDisplayMarker`, listener = the meter, which forwards `isStoryActive`, `isApocalypseActive`, `locustDisplayMarker`, `pygmyLogicArray:`, `onThetanDrag:`, `onThetanDragComplete:` to its own listener; `onThetanDie:` releases them), `ClosetTempleClosetDoorLogic` (the door calls `areThetansActive`, `holdClosetDoor` and `stopHoldingClosetDoor` on the meter, which forwards the hold to its thetans), `PygmyLogic(Apocalypse)` (`fallOntoClosetTempleThetanMeter:`, `ejectFromClosetTempleThetanMeter:`; meter sends `explodeFromThetanBirth`), the view controller (`dropApocalypseThetansIntoApoclaypseTempleRoom` [sic], `spawnApocalypseThetans`, `removeApocalypseThetans`).
  - also sent in code: `PygmyLogic` (`ejectFromClosetTempleThetanMeter`); `ThetanLogic` (`canHoldCloset`, `exitScreen`, `exitScreenNow`, `holdClosetOpenAtXPos:yPos:`, `refreshDisplayMarker`, `setApocalypseMode:`).
- **Called from**: `areThetansActive`, `holdClosetDoor`, `stopHoldingClosetDoor` ← ClosetTempleClosetDoorLogic; `dropApocalypseThetansIntoApoclaypseTempleRoom`, `removeApocalypseThetans`, `spawnApocalypseThetans` ← PocketGodViewController(Game); `ejectFromClosetTempleThetanMeter:` ← PygmyLogic, PygmyLogic(Apocalypse); `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `fallOntoClosetTempleThetanMeter:`, `isAvailable` ← PygmyLogic(Apocalypse).
- **Data ids**:
  - behaviours: `ClosetTempleThetanMeterEjectPygmy`, `ClosetTempleThetanMeterEnterIsland`, `ClosetTempleThetanMeterExitIsland`, `ClosetTempleThetanMeterLevel1`, `ClosetTempleThetanMeterLevel2`, `ClosetTempleThetanMeterPygmyFall`
  - models: `Thetan`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `thetanCreationComplete:::` @ 0x10019e858 (1 use).
- **Persistence**: none.
- **Behaviour**: a pygmy on the meter (channel 1, level 0, `ClosetTempleThetanMeterPygmyFall`); taps → `ClosetTempleThetanMeterLevel1`, `…Level2`, then the release (`animSpawnThetans` → `spawnThetans: NO`, `thetanCreationComplete:::` explodes the pygmy). `spawnThetans: apocalypse` @ 0x10019e8c4: existing thetans `exitScreen`; 8 new ones at the meter + (0, 24), or (apocalypse) at `x = (rand%4801)/10`, `y = 370`, apocalypse mode from the listener. `holdClosetDoor` assigns free thetans to hold the door; `areThetansActive`.

#### `ThetanLogic`

`SpriteLogic` · `ThetanLogic.c` · 21 methods · 16 662 B

- **Role**: one thetan spirit: flies around, can be dragged, possesses pygmies and holds the closet door; in apocalypse mode it roams the apocalypse rooms.
- **Where**: Closet temple / apocalypse rooms.
- **Created**:
  - allocated in `ClosetTempleThetanMeterLogic::spawnThetans:` @ 0x10019e8c4
  - by `ClosetTempleThetanMeterLogic::spawnThetans:` (listener = the meter).
- **Listener**:
  - `<ThetanLogicListener>` (`mThetanLogicListener`): `isStoryActive`, `isApocalypseActive`, `locustDisplayMarker`, `pygmyLogicArray:`, `onThetanDie:`, `onThetanDrag:`, `onThetanDragComplete:` (never sent by this class: `isApocalypseActive`)
  - set in `ClosetTempleThetanMeterLogic::spawnThetans:` @ 0x10019e8c4
- **Talks to**: `PygmyLogic` (from the listener's `pygmyLogicArray:`: `canBePossessedByThetan`, `startPossessedByThetan:`, `possessedByThetan:`, `thetanStopPossessingPygmy:`; the pygmy calls back `stopPossessingPygmy:`), `IdolManager` (episode events `ThetansHoldClosetDoorOpen`, `ThetansFlyOffScreen`, `DragThetan` / `DragApocalypseThetans`); the meter calls `canHoldCloset`, `holdClosetOpenAtXPos:yPos:`, `stopHoldingClosetDoor`, `exitScreen`, `exitScreenNow`, `setApocalypseMode:`.
  - also sent in code: `IdolManager` (`triggerEpisodeEvent:`).
- **Called from**: `canHoldCloset`, `exitScreen`, `exitScreenNow`, `holdClosetOpenAtXPos:yPos:`, `refreshDisplayMarker`, `setApocalypseMode:`, `stopHoldingClosetDoor` ← ClosetTempleThetanMeterLogic; `stopPossessingPygmy:` ← PygmyLogic, PygmyLogic(Apocalypse).
- **Data ids**:
  - behaviours: `ThetanInit`, `ThetanInitApocalypse`, `ThetanPossessPygmy`, `ThetanStopPossession`, `ThetanStopPossessionApocalypse`
  - episode events: `ThetansFlyOffScreen`, `ThetansHoldClosetDoorOpen`
  - other strings: `DragApocalypseThetans`, `DragThetan`
- **Persistence**: none.
- **Behaviour**:
  - Box ±20, selectable. `onGameFrame:` @ 0x10019bf04 runs while no possession is active: the timers `mMoveToTargetTime`, `mStayAliveTime` and `mRotateTime` count down; every 2 s (`mRotateTime`) the turn direction `mMoveDirection` flips sign.
  - Free flight (no touch, not holding the closet, no target): `moveTarget += (moveDirection − moveTarget)·0.05`, `moveRot += moveTarget·(rand%51)/10` degrees, `pos += (cos, sin)(moveRot)·150·dt` (the decompiler prints `180` for the cosine term: `__sincosf_stret` returns the cosine in `s1`). Screen edges re-aim it and zero `moveTarget`: x < 0 → heading `(rand%901 − 450)/10`°, x > 480 → `(rand%901 + 1350)/10`°, y < 0 → `(rand%901 + 450)/10`°, y > 320 → `(rand%901 + 2250)/10`°. Once the stay-alive time is over (not in apocalypse mode) and it is off screen (x < −20, x > 500, y < −20 or y > 340): episode event `ThetansFlyOffScreen` and listener `onThetanDie:`.
  - Release of a drag, `touchEndedInside:allTouches:` @ 0x10019c3fc: episode event `DragThetan` (`DragApocalypseThetans` in apocalypse mode), listener `onThetanDragComplete:`, target = release point, heading toward it, `mMoveToTargetTime = 0.3`; the first pygmy that `canBePossessedByThetan` within 40 px of `(px, py + 20)` is chosen (thetan unselectable, pygmy `startPossessedByThetan:`). With a target, or while holding the closet, the thetan eases 10 % per frame toward the target (or `mClosetPos`); when `mMoveToTargetTime` runs out with a chosen pygmy, possession starts: `ThetanPossessPygmy`, pygmy `possessedByThetan:`.
  - `stopPossessingPygmy: p` @ 0x10019c75c (from the pygmy): reappears at `(px, py + 20)`, selectable, `ThetanStopPossession` (`ThetanStopPossessionApocalypse`). `holdClosetOpenAtXPos:yPos:` @ 0x10019be14 (not while possessing): hold flag and closet position, episode event `ThetansHoldClosetDoorOpen`; `stopHoldingClosetDoor` clears it. `setApocalypseMode:` re-places it on its possessed pygmy (`ThetanInit` / `ThetanInitApocalypse`).

#### `XenuLogic`

`SpriteLogic` · `XenuLogic.c` · 15 methods + 4 item handlers + 1 class method · 9 285 B

- **Role**: Xenu, the alien overlord in the Xenu room: spawning him starts the apocalypse; he eats pygmies dropped into his two slots or uses two cut-in-half pygmies as puppets.
- **Where**: Xenu temple (location 17), entry 518 (`XenuTempleDisplayGroup/Xenu`).
- **Created**:
  - `createLogicClasses` table entry 518 (#518 `XenuTempleDisplayGroup/Xenu`)
  - `+instance` returns `DAT_1008161f8`; listener = view controller; init stops its game frame.
- **Listener**:
  - `<XenuLogicListener>` (`mXenuLogicListener`): `startApocalypse`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `PygmyLogic(Apocalypse)` (`isSlot1Available:`, `isSlot2Available:`, `fallIntoXenuSlot1:`, `fallIntoXenuSlot2:`, `ejectFromXenu:`, `isCutInHalf`; Xenu sends `ejectFromXenu`), the view controller (`spawn`, `banish`, `isAlive`).
- **Called from**: `banish` ← PocketGodViewController(Game); `ejectFromXenu:` ← PygmyLogic, PygmyLogic(Apocalypse); `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:`, `isAlive` ← PocketGodViewController; `fallIntoXenuSlot1:`, `fallIntoXenuSlot2:`, `isSlot1Available:`, `isSlot2Available:`, `spawn` ← PygmyLogic(Apocalypse).
- **Data ids**:
  - behaviours: `XenuEatPygmy1`, `XenuEatPygmy2`, `XenuEjectPygmy`, `XenuEnterIsland`, `XenuEnterIslandAlive`, `XenuExitIsland`, `XenuPuppetPygmy1`, `XenuPuppetPygmy1And2`, `XenuPuppetPygmy2`, `XenuSpawn`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `spawnXenuComplete:::` @ 0x100124bd8 (1 use), `eatPygmy1Complete:::` @ 0x100124c18 (1 use), `eatPygmy2Complete:::` @ 0x100124c84 (1 use), `puppetPygmyComplete:::` @ 0x100124cf0 (1 use).
- **Persistence**: none.
- **Behaviour**: `spawn` → alive, `XenuSpawn` (`spawnXenuComplete:::` → listener `startApocalypse`; leaving the room before that also starts it). Slots: free when alive and the slot is empty, and — if the other slot is taken — only for a cut-in-half pygmy when the other one is cut too. `fallIntoXenuSlot1:` (channel 2) / `…Slot2:` (channel 3): whole pygmy → `XenuEatPygmy1/2`; cut-in-half → `XenuPuppetPygmy1/2` or `XenuPuppetPygmy1And2`. `enterIsland:` → `XenuEnterIslandAlive` / `XenuEnterIsland`.

#### `XenuTempleLogic`

`SpriteLogic` · `XenuTempleLogic.c` · 9 methods + 1 class method · 3 493 B

- **Role**: the Xenu room's background wall, which Tom tears down (revealing the way) and which is rebuilt afterwards.
- **Where**: Xenu temple, entry 517 (`XenuTempleDisplayGroup/Background`, animation `XenuRoomBackgroundInitAnim`).
- **Created**:
  - `createLogicClasses` table entry 517 (#517 `XenuTempleDisplayGroup/Background`); init steps: `play_animation XenuRoomBackgroundInitAnim`
  - `+instance` returns `DAT_1007e7bc8`; listener = view controller; init stops its game frame.
- **Listener**:
  - `<XenuTempleLogicListener>` (`mXenuTempleLogicListener`): `isBabyPygmyActive`, `isTomPygmyActive`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `IdolManager` (episode events `BringTomToXenuRoom`, `BringBabyToXenuRoom` on entering with Tom / the baby active); `PygmyLogic(Apocalypse)` calls `tearDownWall`, the view controller `buildWallBackUp`.
  - also sent in code: `IdolManager` (`triggerEpisodeEvent:`).
- **Called from**: `buildWallBackUp` ← PocketGodViewController(Game); `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController; `tearDownWall` ← PygmyLogic(Apocalypse).
- **Data ids**:
  - behaviours: `XenuTempleEnterIsland`, `XenuTempleExitIsland`, `XenuTempleTearDownWall`, `XenuTempleWallTornDown`
  - episode events: `BringBabyToXenuRoom`, `BringTomToXenuRoom`
- **Persistence**: none.
- **Behaviour**: `tearDownWall` → `XenuTempleTearDownWall`; `enterIsland:` → `XenuTempleEnterIsland` or `XenuTempleWallTornDown`; `exitIsland:` → `XenuTempleExitIsland`.

#### `DanceDanceExecutionLogic`

`SpriteLogic` · `DanceDanceExecutionLogic.c` · 24 methods + 4 item handlers · 9 944 B

- **Role**: the "Dance Dance Execution" stage of the Dance temple: a pygmy dropped on it must dance (the player plays the rhythm game); combinations of held arrows trigger dance moves; losing makes the disco ball crush the pygmy.
- **Where**: Dance temple (location 13), entry 500 (`DanceTempleDisplayGroup/DanceDanceExecution`, init step `setApocalypseCloudStyleId: DDE`).
- **Created**:
  - `createLogicClasses` table entry 500 (#500 `DanceTempleDisplayGroup/DanceDanceExecution`); init steps: `set_apocalypse_cloud_style_id DDE`
  - listener = view controller; init stops its game frame, unselectable, fetches the face logic (entry 499 `DanceDancePygmyFace`).
- **Listener**:
  - `<DanceDanceExecutionLogicListener>` (`mDanceDanceExecutionLogicListener`): `stopBackgroundLoop`, `playBackgroundLoop:`, `onDanceDanceExecutionStart:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `DanceDanceExecutionScreenLogic` (entry 501: `openDanceDanceExecution`, `quitScreen`, `onDanceIntroStarted`; the screen sends `onPressLeft/Down/Up/Right`, `onReleaseLeft/…`, `setDanceFace:`, `isAvailable`, `gameOver`), `DiscoBallLogic` (entry 478 `smashPygmy`; it calls back `smashPygmy`), `PygmyLogic(Apocalypse)` (`pygmyFallIntoDanceDanceExecution:`, `removePygmyFromDanceDanceExecution:`; stage sends `danceDanceExecuted:`, `danceDanceExecutedComplete:`, `tossFromDanceDanceExecution`).
  - `getLogic:` 478 `DiscoBallLogic`, 499 `MultiModelLogic`, 501 `DanceDanceExecutionScreenLogic`.
- **Called from**: `gameOver`, `onPressDown`, `onPressLeft`, `onPressRight`, `onPressUp`, `onReleaseDown`, `onReleaseLeft`, `onReleaseRight`, `onReleaseUp`, `setDanceFace:` ← DanceDanceExecutionScreenLogic; `isAvailable` ← DanceDanceExecutionScreenLogic, PygmyLogic(Apocalypse), PygmyLogic(Island); `pygmyFallIntoDanceDanceExecution:` ← PygmyLogic(Apocalypse); `removePygmyFromDanceDanceExecution:` ← PygmyLogic, PygmyLogic(Story); `smashPygmy` ← DiscoBallLogic.
- **Data ids**:
  - behaviours: `DanceDanceExecutionEnterIsland`, `DanceDanceExecutionExitIsland`, `DanceDanceExecutionPygmyFallInside`, `DanceDanceExecutionRemovePygmy`, `DanceDanceExecutionSmashPygmy`, `DanceFaceBad`, `DanceFaceCocky`, `DanceFaceGood`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `stopBackgroundLoop:::` @ 0x10017cebc (6 uses), `playBackgroundLoop:::` @ 0x10017ceec (25 uses), `smashPygmyComplete:::` @ 0x10017cf20 (1 use), `pygmyFallInsideComplete:::` @ 0x10017cf8c (5 uses).
- **Persistence**: none.
- **Behaviour**: arrow bits left 1, down 2, up 4, right 8 (`pressDanceButton:` / `releaseDanceButton:` keep `mDanceFlags`); `updateDanceMove` @ 0x10017c9ec plays the first move of the table at `0x1007516c0` (count `DAT_100751760`, 16-byte records {sequence id, flags}) whose flags are all held (`docs/re/struct-xml.md` §9 mentions this table). `setDanceFace:` 0 `DanceFaceGood`, 1 `DanceFaceBad`, 2 `DanceFaceCocky`. `pygmyFallIntoDanceDanceExecution:` binds the pygmy (channel 1), `DanceDanceExecutionPygmyFallInside` and opens the screen; `gameOver` → pygmy `danceDanceExecuted:` and the disco ball `smashPygmy`; `smashPygmy` → `DanceDanceExecutionSmashPygmy`. `playBackgroundLoop:::` / `stopBackgroundLoop:::` forward to the listener.

#### `DanceDanceExecutionScreenLogic`

`BCScreenLogic` · `DanceDanceExecutionScreenLogic.c` · 16 methods + 1 item handler + 1 class method · 43 910 B

- **Role**: the rhythm-game overlay of Dance Dance Execution: scrolling arrow symbols, four arrow buttons, hit feedback, a doom meter ("groove") and the score.
- **Where**: entry 501 (`DanceDanceExecutionScreen`, full screen at (0, 0, −277.12)); buttons/overlays entries 482–499.
- **Created**:
  - `createLogicClasses` table entry 501 (#501 new `BCMultiModel` `DanceDanceExecutionScreen`)
  - entry 501; `+instance` returns `DAT_1007e43f8`; listener = view controller. Init fetches 13 entries (bar 490, overlays 491–494, feedback 495–498, buttons 486–489 with this screen as listener, 9 tweenable bindings).
- **Listener**:
  - `<DanceDanceExecutionScreenLogicListener>` (`mDanceDanceExecutionScreenLogicListener`): `onDanceDanceExecutionStart:`, `onDanceDanceExecutionStop:`, `onDanceDanceExecutionScore:` (never sent by this class: `onDanceDanceExecutionStart:`)
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `DanceDanceExecutionLogic` (entry 500), the beat chart (`assets/data/dance-beats.json`, `BCStructXmlDanceDanceBeatNodeDef`, runtime array at `0x1007ce3bc` stride 0x40 with the active window `DAT_1007e43f0`…`DAT_1007e43f4`; timing in `docs/re/struct-xml.md` §9), sounds `DanceLoop2`, `DanceFeedbackGreat`, `DanceFeedbackAwesome`, `DanceFeedbackYouRock`, `DanceFeedbackLookout`.
  - also sent in code: `DanceDanceExecutionLogic` (`gameOver`, `isAvailable`, `onPressDown`, `onPressLeft`, `onPressRight`, `onPressUp`, `onReleaseDown`, `onReleaseLeft`, `onReleaseRight`, `onReleaseUp`, `setDanceFace:`).
  - `getLogic:` 486–489 `BCButtonLogic`, 490–498 `MultiModelLogic`, 500 `DanceDanceExecutionLogic`.
- **Called from**: `onDanceIntroStarted`, `openDanceDanceExecution`, `quitScreen` ← DanceDanceExecutionLogic.
- **Data ids**:
  - behaviours: `CloseDanceDanceExecutionScreen`, `DanceDanceButtonDownCorrect`, `DanceDanceButtonDownIncorrect`, `DanceDanceButtonDownInit`, `DanceDanceButtonDownMiss`, `DanceDanceButtonDownPerfect`, `DanceDanceButtonLeftCorrect`, `DanceDanceButtonLeftIncorrect`, `DanceDanceButtonLeftInit`, `DanceDanceButtonLeftMiss`, `DanceDanceButtonLeftPerfect`, `DanceDanceButtonRightCorrect`, `DanceDanceButtonRightIncorrect`, `DanceDanceButtonRightInit`, `DanceDanceButtonRightMiss`, `DanceDanceButtonRightPerfect`, `DanceDanceButtonUpCorrect`, `DanceDanceButtonUpIncorrect`, `DanceDanceButtonUpInit`, `DanceDanceButtonUpMiss`, `DanceDanceButtonUpPerfect`, `DanceDanceFeedbackGood`, `DanceDanceFeedbackMiss`, `DanceDanceFeedbackOk`, `DanceDanceFeedbackPerfect`, `DanceDanceSymbolDown`, `DanceDanceSymbolLeft`, `DanceDanceSymbolRight`, `DanceDanceSymbolUp`, `QuitDanceDanceExecutionScreen`, `ShowDanceDanceExecutionScreen`
  - models: `DanceSymbol`
  - sounds: `DanceDanceButtonLeftBadHit`, `DanceDanceButtonLeftGoodHit`, `DanceDanceButtonLeftPerfectHit`, `DanceFeedbackAwesome`, `DanceFeedbackGreat`, `DanceFeedbackLookout`, `DanceFeedbackYouRock`, `DanceIntro`, `DanceLoop2`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `startDanceDanceExecution:::` @ 0x1000360f4 (1 use).
- **Persistence**: none.
- **Behaviour**:
  - `onDanceIntroStarted` stores the start time; `onGameFrame:` @ 0x100036118 starts `DanceLoop2` 7.5 s after the intro (and keeps it playing), runs `manageDanceDanceSymbols:` @ 0x100035684 — beat clock `(now − start + fudge − 7.5)/0.2325546875 + 13`, symbols at `y = 350 − 317·(clock − beat)/13`, misses detected 0.5 beat after the target — and eases the doom bar (`groove += (target − groove)·0.5`, bar frame percent).
  - `onButtonPress:touch:` @ 0x100037000 per arrow: on a hittable beat — hit percent > 0.8 → `DanceDanceFeedbackPerfect`, Δ −0.012; > 0.5 → `…Good`, Δ −0.008; else `…Ok`, Δ −0.005 (button overlays `DanceDanceButton<Dir>Perfect/Correct`); pressing with no beat (more than 0.5 beat after the last miss) → `…Miss`, `…Incorrect`, Δ +0.22.
  - `updateGroovePercent: Δ` @ 0x100036ce0: target += Δ; hits (Δ < 0) add `−Δ·1000` to the score (`onDanceDanceExecutionScore:`) and grow the streak (sounds at 30, 60, 120), misses reset it; above 0.6 the "look out" warning plays once; above 1 → game over (`gameOver`, `closeScreen`); below 0 clamps; face cocky when ≤ 0 after 30 beats, good when ≤ 0.6, else bad.
  - `openDanceDanceExecution`, `closeScreen`, `quitScreen`, `closeScreenNow`, `removeSymbols`; `startDanceDanceExecution:::` arms it.

#### `DiscoBallLogic`

`SpriteLogic` · `DiscoBallLogic.c` · 7 methods + 2 item handlers · 2 673 B

- **Role**: the disco ball of the Dance temple; it falls on a pygmy that loses Dance Dance Execution.
- **Where**: entry 478 (`DanceTempleDisplayGroup/DiscoBall`).
- **Created**:
  - `createLogicClasses` table entry 478 (#478 `DanceTempleDisplayGroup/DiscoBall`)
  - listener = view controller; init stops its game frame.
- **Listener**:
  - `<DiscoBallLogicListener>` (`mDiscoBallLogicListener`): `stopBackgroundLoop`, `playBackgroundLoop:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `DanceDanceExecutionLogic` (`smashPygmy`).
  - `getLogic:` 500 `DanceDanceExecutionLogic`.
- **Called from**: `enterIsland:`, `exitIsland:` ← PocketGodViewController; `smashPygmy` ← DanceDanceExecutionLogic.
- **Data ids**:
  - behaviours: `DiscoBallEnterIsland`, `DiscoBallExitIsland`, `DiscoBallSmashPygmy`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `stopBackgroundLoop:::` @ 0x10011bb9c (6 uses), `playBackgroundLoop:::` @ 0x10011bbcc (25 uses).
- **Persistence**: none.
- **Behaviour**: `enterIsland:` → `DiscoBallEnterIsland`, `exitIsland:` → `DiscoBallExitIsland`, `smashPygmy` → `DiscoBallSmashPygmy`; `animSmashPygmy` → entry 500 `smashPygmy`. Background-loop items forward to the listener.

#### `DiscoLightsLogic`

`SpriteLogic` · `DiscoLightsLogic.c` · 5 methods · 1 547 B

- **Role**: the disco lights of the Dance temple (decoration).
- **Where**: entry 479 (`DanceTempleDisplayGroup/DiscoLights`).
- **Created**:
  - `createLogicClasses` table entry 479 (#479 `DanceTempleDisplayGroup/DiscoLights`)
  - listener = view controller (protocol empty); init stops its game frame.
- **Listener**:
  - `<DiscoLightsLogicListener>` (`mDiscoLightsLogicListener`): no methods
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Called from**: `enterIsland:`, `exitIsland:` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `DiscoLightsEnterIsland`, `DiscoLightsExitIsland`
- **Persistence**: none.
- **Behaviour**: `enterIsland:` → `DiscoLightsEnterIsland`, `exitIsland:` → `DiscoLightsExitIsland`.

#### `GoGoCageLogic`

`SpriteLogic` · `GoGoCageLogic.c` · 9 methods + 1 item handler · 3 533 B

- **Role**: the go-go dancer cage of the Dance temple; a pygmy dropped in dances in it.
- **Where**: entry 477 (`DanceTempleDisplayGroup/DiscoCage`).
- **Created**:
  - `createLogicClasses` table entry 477 (#477 `DanceTempleDisplayGroup/DiscoCage`)
  - listener = view controller (protocol empty); init stops its game frame.
- **Listener**:
  - `<GoGoCageLogicListener>` (`mGoGoCageLogicListener`): no methods
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `PygmyLogic` (`tossFromGoGoCage`).
- **Called from**: `enterIsland:`, `exitIsland:` ← PocketGodViewController; `isAvailable`, `pygmyFallIntoGoGoCage:` ← PygmyLogic(Apocalypse); `removePygmyFromGoGoCage:` ← PygmyLogic, PygmyLogic(Story).
- **Data ids**:
  - behaviours: `GoGoCageEnterIsland`, `GoGoCageExitIsland`, `GoGoCagePygmyFallInside`
  - other strings: `Default`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `pygmyFallInsideComplete:::` @ 0x10008f9ac (5 uses).
- **Persistence**: none.
- **Behaviour**: `pygmyFallIntoGoGoCage:` (if empty) binds the pygmy on channel 1, `GoGoCagePygmyFallInside`; `removePygmyFromGoGoCage:` → `GoGoCageEnterIsland`; `exitIsland:` tosses the pygmy (`tossFromGoGoCage`), `GoGoCageExitIsland`.

#### `DanceBalloonBundleLogic`

`SpriteLogic` · `DanceBalloonBundleLogic.c` · 13 methods · 4 262 B

- **Role**: the balloon bundle of the Dance temple: tapping it pulls out a balloon at the finger.
- **Where**: entry 480 (`DanceTempleDisplayGroup/BalloonBundle`).
- **Created**:
  - `createLogicClasses` table entry 480 (#480 `DanceTempleDisplayGroup/BalloonBundle`)
  - listener = view controller; init stops its game frame.
- **Listener**:
  - `<DanceBalloonBundleLogicListener>` (`mDanceBalloonBundleLogicListener`): `isStoryActive`, `danceBalloonLogic:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `DanceBalloonLogic` (via `danceBalloonLogic:`).
  - also sent in code: `DanceBalloonLogic` (`enterIsland`, `isActive`, `touchBeganInside:allTouches:`).
- **Called from**: `enterIsland:`, `exitIsland:` ← PocketGodViewController.
- **Data ids**:
  - behaviours: `DanceBalloonBundleEnterIsland`, `DanceBalloonBundleExitIsland`
- **Persistence**: none.
- **Behaviour**: `enterIsland:`/`exitIsland:` → `DanceBalloonBundleEnterIsland`/`ExitIsland`. A tap (no story) when the balloon is not active moves the balloon to the touch point, enters it and hands it the touch so it is dragged immediately.

#### `DanceBalloonLogic`

`SpriteLogic` · `DanceBalloonLogic.c` · 17 methods + 3 item handlers · 14 846 B

- **Role**: the helium balloon of the Dance temple: dragged onto a pygmy it lifts it away.
- **Where**: entry 481 (`DanceBalloon`).
- **Created**:
  - `createLogicClasses` table entry 481 (#481 new `BCMultiModel` `DanceBalloon`)
  - listener = view controller.
- **Listener**:
  - `<DanceBalloonLogicListener>` (`mDanceBalloonLogicListener`): `pygmyLogicArray:`, `pygmyDragDisplayMarker`, `onDanceBalloonDrag:`, `onDanceBalloonDragComplete:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `PygmyLogic` (balloon pickup via `checkPygmyCollision`).
  - also sent in code: `PygmyLogic` (`inhaleDanceBalloon:`).
- **Called from**: `enterIsland`, `touchBeganInside:allTouches:` ← DanceBalloonBundleLogic; `exitIsland` ← PocketGodViewController, PocketGodViewController(Game); `isActive` ← DanceBalloonBundleLogic, PocketGodViewController(Game).
- **Data ids**:
  - behaviours: `DanceBalloonDie`, `DanceBalloonDrag`, `DanceBalloonEnterIsland`, `DanceBalloonExitIsland`, `DanceBalloonFall`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `drag:::` @ 0x10016a520 (40 uses), `fall:::` @ 0x10016a6b0 (134 uses), `die:::` @ 0x10016a6cc (119 uses).
- **Persistence**: none.
- **Behaviour**: box ±35, selectable. `enterIsland` places it after `pygmyDragDisplayMarker`. `onGameFrame:` @ 0x10016a28c: buoyancy `(0, +600)`, spring drag (k 50, damping 6, hanging below the finger at +90°), integration; flies off the top (`DanceBalloonDie` above the screen). `checkPygmyCollision` attaches it to a pygmy it touches.

### 4.16 God island and Battle of the Gods

God island (state 7) and the Battle of the Gods. The four episode idols on their pedestals open the temple. A pygmy that enters fights an enemy pygmy in the battle arena, with the idol screen as the battle UI. Losers sail on the Fail Boat (state 8).

#### `TempleDoorLogic`

`SpriteLogic` · `TempleDoorLogic.c` · 32 methods + 14 item handlers + 1 class method · 28 719 B

- **Role**: the temple of God island and host of the Battle of the Gods: the four god idols on the pedestals open its door in four steps; a pygmy that enters fights an enemy pygmy in the battle arena, with idols on the battle pedestals, power-ups, attacks, realm effects, fatalities and resurrection.
- **Where**: God island (location 7), entry 374 (`GodIslandDisplayGroup/Temple`); the arena is `BattleArenaDisplayGroup` (entries 362–367, 375–398).
- **Created**:
  - `createLogicClasses` table entry 374 (#374 `GodIslandDisplayGroup/Temple`); init steps: `play_animation GodIslandTempleInitAnim`
  - entry 374; `+instance` returns `DAT_1007e7bd0`; listener = view controller. Init fetches the battle pedestal idols (364/365) and pedestals (366/367, listener = the door for the drop/clear events), creates `mBattleSequence` (a second `BCSequence` driven by `setBattleBehavior:`).
- **Listener**:
  - `<TempleDoorLogicListener>` (`mTempleDoorLogicListener`): `isStoryActive`, `isGravityEnabled`, `isTempleDoorEnabled`, `islandLocationState`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyLogicArray:`, `createPygmyLogic`, `onBattleOfTheGodsStart:pygmyLogic:`, `onBattleOfTheGodsStop:`, `onOpenIdolBattleEnemyScreen`, `onOpenIdolBattleScreen`, `onOpenIdolBattleResultsScreen` (never sent by this class: `isGravityEnabled`, `isStoryActive`, `isTempleDoorEnabled`, `islandCenterXPos`, `islandMaxXPos`, `islandMinXPos`, `pygmyLogicArray:`)
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `IdolPedestalLogic` (`openDoor`, `closeDoor`), `IdolManager` (`enemyName`, `playerName`, `applyBattleExperience`, `quitBattleOfTheGods`, `showBattleRealmResults`, `showBattleRealmResultsOnIdol`, `showBattleRealmEffectResults`, `areBattleRealmEffectResultsComplete`, `onBattleTurnComplete`, `performResurrection`, `isMultiplayerActive`, `canShowIdolBattleMultiplayerResult`, `showIdolBattleMultiplayerResult`), `IdolScreenLogic` (through `+instance`: `showBattleAttackStart`, `openIdolBattleSelectionPanel`; the screen calls back `battlePygmyLogic`, `battleEnemyPygmyLogic`, `cancelBattle`), `GameUiManager` (`allPointsComplete`), `PygmyLogic(BattleOfTheGods)` (`startBattleOfTheGods:`, `startBattleOfTheGodsEnemy:`, `startBattlePowerUp:opponentIdol:`, `startBattleAttack:opponentIdol:`, `fatalityInBattle`, `wasKilledDuringBattle`, `spawnFromBattleOfTheGods`, `removeFromBattleOfTheGods`, `setPygmyNameLabel:`; pygmies call `onPygmyBattleReady:`).
  - also sent in code: `IdolManager` (`activeBattleEnemyEpisodeIdol`, `activeBattlePlayerEpisodeIdol`, `isBattleComplete`, `onMultiplayerGameOver`, `shouldEnemyResurrect`, `shouldPlayerResurrect`); `PygmyLogic` (`ejectFromTempleDoor`, `pygmyPersona`, `updateLabelLayer`); `PygmyPersona` (`name`).
  - `getLogic:` 364–367 `MultiModelLogic`.
- **Called from**: `battleEnemyPygmyLogic`, `cancelBattle` ← IdolScreenLogic; `battlePygmyLogic` ← IdolManager, IdolScreenLogic; `closeDoor`, `openDoor` ← IdolPedestalLogic; `ejectFromTempleDoor:` ← PygmyLogic, PygmyLogic(Island); `enterIsland:`, `exitIsland:`, `exitIslandNow` ← PocketGodViewController; `fallIntoTempleDoor:` ← PygmyLogic(Island); `isAvailable` ← PygmyLogic(Apocalypse), PygmyLogic(Island); `isBattleOfTheGodsActive`, `onEnemyFatality`, `onPlayerFatality`, `playBattleSequence`, `quitGame`, `startBattle` ← IdolManager; `jumpIntoTempleDoor:`, `jumpIntoTempleDoorComplete:` ← PygmyLogic(Apocalypse); `onPygmyBattleReady:` ← PygmyLogic(BattleOfTheGods).
- **Data ids**:
  - behaviours: `BattleArenaFatality`, `BattleArenaFatalityResurrect`, `BattleArenaStartBattle`, `BattleArenaStartNextAttack`, `BattlePedestalClearIdol`, `BattlePedestalDropIdol`, `BattlePedestalInit`, `TempleDoorClose1`, `TempleDoorClose2`, `TempleDoorClose3`, `TempleDoorClose4`, `TempleDoorEjectPygmy`, `TempleDoorEnterIsland`, `TempleDoorExitIsland`, `TempleDoorOpen1`, `TempleDoorOpen2`, `TempleDoorOpen3`, `TempleDoorOpen4`, `TempleDoorPygmyFall`, `TempleDoorPygmyJump`
  - groups: `BattleArenaDisplayGroup`
  - markers: `BattleOfTheGodsPygmyDisplayMarker`
  - event ids: `BattlePedestalClearIdolComplete`, `BattlePedestalDropIdolComplete`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `onPygmyFallInsideComplete:::` @ 0x100090af8 (16 uses), `showBattleIdols:::` @ 0x100090ffc (1 use), `startPygmyPowerUp:::` @ 0x1000911d0 (1 use), `startPygmyAttacks:::` @ 0x100091290 (1 use), `showBattleRealmResults:::` @ 0x100091494 (1 use), `showBattleRealmResultsOnIdol:::` @ 0x1000914d0 (1 use), `showBattleRealmEffectsResults:::` @ 0x10009150c (1 use), `waitForFatalityComplete:::` @ 0x10009157c (2 uses), `waitForBattleResultsComplete:::` @ 0x1000915ac (3 uses), `onBattleTurnComplete:::` @ 0x1000915f0 (2 uses), `checkForFatality:::` @ 0x10009162c (3 uses), `performResurrection:::` @ 0x1000916f8 (1 use), `showBattleSelectionPanel:::` @ 0x1000917dc (3 uses), `onGameOver:::` @ 0x100091880 (1 use).
- **Persistence**: none.
- **Behaviour**:
  - Door: `openDoor` @ 0x1000908c0 steps `mDoorOpenLevel` 0→4 with `TempleDoorOpen1…4`; `closeDoor` ejects a pygmy and steps back (`TempleDoorClose1…4`); `isAvailable` = level 4 and empty. A pygmy falls/jumps in (`fallIntoTempleDoor:` → `TempleDoorPygmyFall`, `jumpIntoTempleDoor:` + `jumpIntoTempleDoorComplete:` → `TempleDoorPygmyJump`, channel 1); `onPygmyFallInsideComplete:::` opens the enemy selection screen (`onOpenIdolBattleEnemyScreen`).
  - `startBattle` @ 0x100090b60: creates the enemy pygmy (`createPygmyLogic`, named after the enemy), inserts it after `BattleOfTheGodsPygmyDisplayMarker`, starts both fighters, `onBattleOfTheGodsStart:pygmyLogic:`, resets the battle pedestals (`BattlePedestalInit`), `onOpenIdolBattleScreen`, battle script `BattleArenaStartBattle`. `playBattleSequence` → `BattleArenaStartNextAttack`. The battle script's items (`showBattleIdols`, `startPygmyPowerUp`, `startPygmyAttacks`, `showBattleRealm…`, `checkForFatality`, `performResurrection`, `showBattleSelectionPanel`, `onGameOver`, …) are in sequences.md §5.
  - `gameOver` removes the enemy, applies experience, `onBattleOfTheGodsStop:` and `onOpenIdolBattleResultsScreen`; `quitGame` quits through `IdolManager` and cancels; `cancelBattle` restores the pygmy's own name and ejects it. `isBattleOfTheGodsActive` = an enemy exists.

#### `GodIdolLogic`

`SpriteLogic` · `GodIdolLogic.c` · 27 methods + 5 item handlers · 34 884 B

- **Role**: the four god idols (episode idols) of God island: throwable figures showing the selected episode idol; placed on the idol pedestals they open the temple.
- **Where**: God island, entries 99–102 (`GodIdol1…4`).
- **Created**:
  - `createLogicClasses` table entries 99–102 (#99 new `BCMultiModel` `GodIdol1`; #100 new `BCMultiModel` `GodIdol2`; #101 new `BCMultiModel` `GodIdol3`; #102 new `BCMultiModel` `GodIdol4`)
  - listener = view controller; the view controller sets `setSlot:` (0–3) and `setSkinTextureBitsDefArray:` and calls `updateIdolTexture`; `IdolManager` sets `setStructXmlEpisodeNodeDef:` (the selected idol).
- **Listener**:
  - `<GodIdolLogicListener>` (`mGodIdolLogicListener`): `isStoryActive`, `isGravityEnabled`, `isGodIdolEnabled`, `isIdolPedestalEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `idolPedestalLogicArray:`, `onGodIdolDrag:`, `onGodIdolDragComplete:` (never sent by this class: `islandCenterXPos`, `islandMaxXPos`, `islandMinXPos`)
  - wired by `initLogicListeners` to `PocketGodViewController` (4)
- **Talks to**: `IdolPedestalLogic` (`godIdolFallIntoIdolPedestal:`, `ejectGodIdolFromIdolPedestal:`, `clearGodIdolSlot:`; pedestals call `ejectFromIdolPedestal`), `PygmyLogic` (`checkForIdolPedestalDuringFall`).
  - also sent in code: `IdolPedestalLogic` (`godIdolSlot`, `isAvailable`).
- **Called from**: `ejectFromIdolPedestal`, `slot` ← IdolPedestalLogic; `enterIsland:`, `exitIsland:`, `exitIslandNow`, `setSkinTextureBitsDefArray:`, `setSlot:`, `updateIdolTexture` ← PocketGodViewController; `setStructXmlEpisodeNodeDef:` ← IdolManager.
- **Data ids**:
  - behaviours: `GodIdolBounce`, `GodIdolDie`, `GodIdolDrag`, `GodIdolEjectFromPedestal`, `GodIdolEnterIsland`, `GodIdolFall`, `GodIdolFallIgnoreIsland`, `GodIdolFallIntoIdolPedestal`, `GodIdolSplash`
  - markers: `DragDisplayMarker`, `PygmyDisplayMarker`
  - other strings: `raw`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `setAnimationBasedOnIdolSlot:::` @ 0x10011a1a8 (1 use), `resetLastPos:::` @ 0x10011a248 (5 uses), `drag:::` @ 0x10011a298 (40 uses), `fall:::` @ 0x10011a450 (134 uses), `die:::` @ 0x10011a5d8 (119 uses).
- **Persistence**: none.
- **Behaviour**: Shared pattern of §2.2 with slide friction 0.025, floor `y = 90`, box from (−40, …); spring drag k 50, damping 6 with quarter-strength tilt. `updateIdolTexture` applies the idol's skin bits (texture slots) for its episode node; `setStructXmlEpisodeNodeDef:` swaps idols (remembering the last one). `enterIsland:` @ 0x100119584 places an idol with an episode def (from its saved pedestal slot, else dropped in). `checkForIdolPedestalDuringFall` @ 0x1001192b8 seats it in a free pedestal it falls into; `ejectFromIdolPedestal` pops it up 20 px with `vel = (−200, 500)`. `setAnimationBasedOnIdolSlot:::` picks the look by slot 0–3.

#### `IdolPedestalLogic`

`SpriteLogic` · `IdolPedestalLogic.c` · 26 methods + 2 item handlers · 13 565 B (the file also holds 22 free functions (about 8.1 KB) of the OpenGL state cache (rendering); not counted)

- **Role**: the four pedestals of God island: a god idol placed on one opens the temple door a step and is remembered across launches; a pygmy can be trapped on one.
- **Where**: God island, entries 370–373 (`GodIslandDisplayGroup/Pedestal1…4`, animation `GodIslandPedestalInitAnim`).
- **Created**:
  - `createLogicClasses` table entries 370–373 (#370 `GodIslandDisplayGroup/Pedestal1`; #371 `GodIslandDisplayGroup/Pedestal2`; #372 `GodIslandDisplayGroup/Pedestal3`; #373 `GodIslandDisplayGroup/Pedestal4`); init steps: `play_animation GodIslandPedestalInitAnim` ×4
  - listener = view controller; box (−50, 0)–(50, 74); `setPedestalIndex:` builds the key `GodIdolSlot%d`.
- **Listener**:
  - `<IdolPedestalLogicListener>` (`mIdolPedestalLogicListener`): `isStoryActive`, `optionPreferences`, `templeDoorLogic:`
  - wired by `initLogicListeners` to `PocketGodViewController` (4)
- **Talks to**: `GodIdolLogic` (channel 2), `TempleDoorLogic` (via `templeDoorLogic:`: `openDoor`, `closeDoor`), `PygmyLogic(Island)` (`fallIntoIdolPedestal:`, `ejectFromIdolPedestal:`, channel 1).
  - also sent in code: `GodIdolLogic` (`ejectFromIdolPedestal`, `slot`); `PygmyLogic` (`ejectFromIdolPedestal`).
- **Called from**: `clearGodIdolSlot:`, `ejectGodIdolFromIdolPedestal:`, `godIdolFallIntoIdolPedestal:`, `godIdolSlot`, `isAvailable` ← GodIdolLogic; `ejectFromIdolPedestal:` ← PygmyLogic, PygmyLogic(Island); `enterIsland:`, `exitIsland:`, `exitIslandNow`, `setPedestalIndex:` ← PocketGodViewController; `fallIntoIdolPedestal:` ← PygmyLogic(Island).
- **Data ids**:
  - behaviours: `IdolPedestalEjectGodIdol`, `IdolPedestalEjectPygmy`, `IdolPedestalEnterIsland`, `IdolPedestalExitIsland`, `IdolPedestalGodIdolFall`, `IdolPedestalPygmyFall`, `IdolPedestalPygmyScream`, `IdolPedestalPygmyStopScream`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `playSoundBasedOnPedestalIndex:::` @ 0x10013f4d0 (1 use), `stopSoundBasedOnPedestalIndex:::` @ 0x10013f598 (2 uses).
- **Persistence**: `Options` preference `GodIdolSlot<n>` (int, default −1): the slot of the idol standing on pedestal *n* (`setGodIdolSlot:` writes it, `godIdolSlot` reads it once with `setDefaultPreference:asInt: −1`); cleared when a pygmy falls on the pedestal or the idol leaves.
- **Behaviour**: `godIdolFallIntoIdolPedestal:` → `IdolPedestalGodIdolFall`, saves the idol's slot and opens the door one step; ejecting it closes it. A tapped trapped pygmy screams (`IdolPedestalPygmyScream` / `…StopScream`). `play/stopSoundBasedOnPedestalIndex:::` items pick a sound by index 1–4.

#### `BattleAlertLogic`

`SpriteLogic` · `BattleAlertLogic.c` · 14 methods + 1 item handler · 6 547 B

- **Role**: the multiplayer battle alert banner ("your turn" messages queued per Game Center match).
- **Where**: entry 400 (`BattleAlert`), shown in the `PocketGod` group (box 0–480 × 0–42, selectable).
- **Created**:
  - `createLogicClasses` table entry 400 (#400 new `BCMultiModel` `BattleAlert`)
  - listener = view controller.
- **Listener**:
  - `<BattleAlertLogicListener>` (`mBattleAlertLogicListener`): `isScreenOpen`, `isStoryActive`, `onBattleAlertSelect`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `IdolManager` (`activateMatchAlert:message:`), `MatchAndMessage` objects.
  - also sent in code: `MatchAndMessage` (`initWithMatch:message:`, `message`).
- **Called from**: `activateMatchAlert:message:` ← IdolManager.
- **Data ids**:
  - behaviours: `BattleAlertActivate`
  - groups: `PocketGod`
  - texts: `BattleAlertMessage`, `BattleAlertMessageTapHere`
  - other strings: `Default`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `onBattleAlertClose:::` @ 0x10002806c (1 use).
- **Persistence**: none.
- **Behaviour**: `activateMatchAlert:message:` queues a `MatchAndMessage`; the first activation moves the banner into `PocketGod` and shows the head of the queue (dynamic texts `BattleAlertMessage`, `BattleAlertMessageTapHere`, `BattleAlertActivate`). A tap (no story, no screen open) → `onBattleAlertSelect`. `onBattleAlertClose:::` pops the queue and shows the next or deactivates.

#### `BattleEnergyBallLogic`

`SpriteLogic` · `BattleEnergyBallLogic.c` · 7 methods · 3 563 B

- **Role**: an energy ball fired at the opponent in Battle of the Gods (4 per side).
- **Where**: entries 380–383 (player) and 391–394 (enemy).
- **Created**:
  - `createLogicClasses` table entries 380–383, 391–394 (new `BCMultiModel` `PlayerEnergyBall1`, new `BCMultiModel` `PlayerEnergyBall2`, new `BCMultiModel` `PlayerEnergyBall3`, … (8 entries)); init steps: `play_animation EnergyBallAerisAnim` ×8
  - table (no listener; the protocol is never set).
- **Listener**: `<BattleEnergyBallLogicListener>` (`mBattleEnergyBallLogicListener`): no methods
- **Talks to**: `PygmyLogic(BattleOfTheGods)` (`activateWithStartPos:targetPygmyLogic:`, `setLastBall:`, `setNoDamage:`; the ball reports its hit to the target pygmy).
- **Called from**: `activateWithStartPos:targetPygmyLogic:`, `setLastBall:`, `setNoDamage:` ← PygmyLogic(BattleOfTheGods).
- **Persistence**: none.
- **Behaviour**: `activateWithStartPos:targetPygmyLogic:` places it after the target's display object; `onGameFrame:` @ 0x1001a936c homes in on `(target.x + target.scale.x·15, target.y + 32)` easing 50 %/frame and, within 1 px vertically, hits the target (respecting `mbNoDamage` and `mbLastBall`).

#### `FailBoatLogic`

`SpriteLogic` · `FailBoatLogic.c` · 9 methods + 1 class method · 3 377 B

- **Role**: the "Fail Boat": the boat of losers (location 8) on which defeated pygmies sail; it rocks gently.
- **Where**: entry 401 (`FailBoatDisplayGroup`); boat model entry 402.
- **Created**:
  - `createLogicClasses` table entry 401 (#401 group `FailBoatDisplayGroup`)
  - `+instance` returns `DAT_1007e5098`; listener = view controller.
- **Listener**:
  - `<FailBoatLogicListener>` (`mFailBoatLogicListener`): `isStoryActive` (never sent by this class: `isStoryActive`)
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: none. `PygmyLogic::onGameFrame:` @ 0x100129980 reads the boat (`getLogic:401`) on location 8 and uses its deck as a sloped floor; `fallIntoFailBoat:` / `ejectFromFailBoat:` (which keep `mPygmyLogicArray`) and `enterIsland:` / `exitIsland:` / `exitIslandNow` (which only toggle `mbActive`) are never sent in 1.48.2.
- **Called from**: `enterIsland:`, `exitIslandNow` ← PocketGodViewController, PocketGodViewController(Game); `exitIsland:` ← PocketGodViewController.
- **Persistence**: none.
- **Behaviour**:
  - `onGameFrame:` @ 0x1000887d8: `rot.z = sin(rotVel.z)·3`, `rotVel.z += 0.01` per frame (the rotation velocity's z is used as a phase).
  - The deck as seen by `PygmyLogic::onGameFrame:` @ 0x100129980 (pygmy code, recorded here because it is the boat's only use): with θ = the boat's `rot.z` in radians and `(bx, by)` its position, a pygmy whose x lies in `[bx − 160·cos θ, bx + 160·cos θ]` lands on `y = by + (x − bx)·tan θ`; while `mbOnFailBoat` its side-collision limits become −160 and 160.

### 4.17 Idol screen and episode UI

The episode-idol screen and its widgets. `IdolScreenLogic` (201 KB) is the largest class in this catalogue and has its own unit, with an internal split proposed in section 5.3.

#### `IdolIconLogic`

`BCLogic` · `IdolIconLogic.c` · 21 methods · 6 954 B

- **Role**: one icon of the idol list (9 per page) on the idol screen.
- **Where**: entries 121–129 (`IdolIconButton1…9`, animations `IdolIconSlot<n>InitAnim`), with halos 103–111 and check marks 112–120.
- **Created**:
  - `createLogicClasses` table entries 121–129 (new `BCMultiModel` `IdolIconButton1`, new `BCMultiModel` `IdolIconButton2`, new `BCMultiModel` `IdolIconButton3`, … (9 entries)); init steps: `play_animation IdolIconSlot1InitAnim`, `play_animation IdolIconSlot2InitAnim`, `play_animation IdolIconSlot3InitAnim`, `play_animation IdolIconSlot4InitAnim`, `play_animation IdolIconSlot5InitAnim`, `play_animation IdolIconSlot6InitAnim`, `play_animation IdolIconSlot7InitAnim`, `play_animation IdolIconSlot8InitAnim`, `play_animation IdolIconSlot9InitAnim`
  - table; listener = `IdolScreenLogic`.
- **Listener**:
  - `<IdolIconLogicListener>` (`mIdolIconLogicListener`): `onIdolIconSelect:`
  - wired by `initLogicListeners` to `IdolScreenLogic` (9)
- **Called from**: `activate:isSelected:`, `deactivate`, `isActive`, `setCheckMarkLogic:`, `setHaloLogic:`, `setLabelId:`, `setSkinTextureBitsDefArray:`, `updateHalo:`, `updateIdolIconTexture` ← IdolScreenLogic.
- **Data ids**:
  - behaviours: `IdolIconHideCheckMark`, `IdolIconHideHalo`, `IdolIconShowCheckMark`, `IdolIconShowHalo`
  - other strings: `raw`
- **Persistence**: none.
- **Behaviour**: box (0, 0)–(100, 100), hidden until `activate:isSelected:` @ 0x100198ec0: shows the icon, loads the idol's icon image into its skin texture slot (`FUN_100038648(skinBits, node.icon, @"raw")`), sets its dynamic text label, the check mark and the halo (`IdolIconShowHalo` / `IdolIconHideHalo`). A tap → `onIdolIconSelect:`. `deactivate` hides it.

#### `IdolSelectionIconLogic`

`BCLogic` · `IdolSelectionIconLogic.c` · 44 methods · 24 757 B

- **Role**: a draggable idol card used in the idol selection library, the four selected slots and the battle screens (with attack/realm type badges, frozen/locked states).
- **Where**: entries 321–328 (`IdolSelectionIcon1…4`, `LibraryIdolSelectionIcon1…4`).
- **Created**:
  - `createLogicClasses` table entries 321–328 (`IdolSelectionLibraryDisplayGroup/IdolSelectionIcon1`, `IdolSelectionLibraryDisplayGroup/IdolSelectionIcon2`, `IdolSelectionLibraryDisplayGroup/IdolSelectionIcon3`, … (8 entries)); init steps: `play_animation IdolSelectionIconSlot1Anim`, `play_animation IdolSelectionIconSlot2Anim`, `play_animation IdolSelectionIconSlot3Anim`, `play_animation IdolSelectionIconSlot4Anim`, `play_animation IdolSelectionIconSlot5Anim`, `play_animation IdolSelectionIconSlot6Anim`, `play_animation IdolSelectionIconSlot7Anim`, `play_animation IdolSelectionIconSlot8Anim`
  - table; listener = `IdolScreenLogic` (table); `IdolScreenLogic` sets its label ids, type logics, home positions and the seven sequence ids (`setSequenceId:battleEnemySequenceId:…battleFrozenSequenceId:`).
- **Listener**:
  - `<IdolSelectionIconLogicListener>` (`mIdolSelectionIconLogicListener`): `onIdolSelectionIconSelect:episodeNodeDef:`
  - wired by `initLogicListeners` to `IdolScreenLogic` (8)
- **Called from**: `activate:`, `activateForBattle:isFrozen:`, `activateForBattleEnemy:`, `deactivate`, `dragXPos:yPos:`, `highlightBasedOnAttackEpisodeNodeDef:`, `highlightForBattle`, `idolSelectionIconType`, `isActive`, `isEnemyIdol`, `selectForBattle`, `setAttackTypeLogic:`, `setBattleHomeXPos:yPos:`, `setBattleNameLabelId:statsLabelId:`, `setBattleSelectedLogic:`, `setEnemyStatsLevel:health:`, `setHomeXPos:yPos:`, `setIdolSelectionIconType:`, `setIsEnemyIdol:`, `setLabelId:`, `setRealmTypeLogic:`, `setScrollXPos:`, `setSequenceId:battleEnemySequenceId:battleEnemyHighlightSequenceId:battleEnemyLockedSequenceId:battleSequenceId:battleHighlightSequenceId:battleFrozenSequenceId:`, `setSkinTextureBitsDefArray:`, `stopDrag`, `structXmlEpisodeNodeDef`, `unhighlightForBattle`, `updateIdolIconTexture` ← IdolScreenLogic.
- **Data ids**:
  - behaviours: `BlankAnim`, `IdolAttackTypeCounter`, `IdolAttackTypeCounterLose`, `IdolAttackTypeCounterWin`, `IdolAttackTypeFast`, `IdolAttackTypeFastLose`, `IdolAttackTypeFastWin`, `IdolAttackTypeStrong`, `IdolAttackTypeStrongLose`, `IdolAttackTypeStrongWin`, `IdolRealmTypeAeris`, `IdolRealmTypeAerisLose`, `IdolRealmTypeAerisWin`, `IdolRealmTypeAnimus`, `IdolRealmTypeAnimusLose`, `IdolRealmTypeAnimusWin`, `IdolRealmTypeInfernis`, `IdolRealmTypeInfernisLose`, `IdolRealmTypeInfernisWin`, `IdolRealmTypeOceanus`, `IdolRealmTypeOceanusLose`, `IdolRealmTypeOceanusWin`, `IdolRealmTypePax`, `IdolRealmTypePaxLose`, `IdolRealmTypePaxWin`, `IdolSelectionIconSelected`
  - other strings: `raw`
- **Persistence**: none.
- **Behaviour**: `activate:` (box (0,−88)–(88,0)), `activateForBattleEnemy:`, `activateForBattle:isFrozen:` (box 50) show it for the current episode node; `resetBattleIcons` sets the attack type badge (`IdolAttackType…` by node +0x74) and realm badge; highlight/unhighlight helpers pick the battle, highlight, enemy or frozen sequence; `highlightBasedOnAttackEpisodeNodeDef:` shows effectiveness against an attacker; `setScrollXPos:` / `dragXPos:yPos:` / `stopDrag` move it relative to its home position (node +0xd8/+0xdc); `updateIdolIconTexture` loads its card image (node +0x68). Touches report `onIdolSelectionIconSelect:episodeNodeDef:`.

#### `IdolBattleRealmEffectLogic`

`BCLogic` · `IdolBattleRealmEffectLogic.c` · 22 methods · 7 111 B

- **Role**: a buff/debuff realm-effect icon shown next to a fighter in the Battle of the Gods HUD.
- **Where**: entries 270–278 (`IdolBattleRealmEffectSlot1…9`).
- **Created**:
  - `createLogicClasses` table entries 270–278 (new `BCMultiModel` `IdolBattleRealmEffectSlot1`, new `BCMultiModel` `IdolBattleRealmEffectSlot2`, new `BCMultiModel` `IdolBattleRealmEffectSlot3`, … (9 entries))
  - table; listener = `IdolScreenLogic`.
- **Listener**:
  - `<IdolBattleRealmEffectLogicListener>` (`mIdolBattleRealmEffectLogicListener`): `onIdolBattleRealmEffectSelect:episodeNodeDef:`
  - wired by `initLogicListeners` to `IdolScreenLogic` (9)
- **Called from**: `activate:isBuffEffect:`, `deactivate`, `idolBattleState`, `isActive`, `isBuffEffect`, `setBuffSequenceId:debuffSequenceId:`, `setIdolBattleState:`, `setTargetXPos:yPos:`, `structXmlEpisodeNodeDef` ← IdolScreenLogic.
- **Persistence**: none.
- **Behaviour**: box (0,−25)–(25,0), hidden; `setBuffSequenceId:debuffSequenceId:`, `setTargetXPos:yPos:`, `setIdolBattleState:`; `activate:isBuffEffect:` shows it with the buff or debuff behaviour for an episode node; a tap → `onIdolBattleRealmEffectSelect:episodeNodeDef:`; `deactivate` hides it.

#### `EpisodeAlertLogic`

`SpriteLogic` · `EpisodeAlertLogic.c` · 15 methods + 2 item handlers · 7 201 B

- **Role**: the episode icon in the corner of the game screen that sparkles when an episode event happens and opens the challenge detail when tapped.
- **Where**: entry 399 (`EpisodeAlert`, box 0–42 × 0–42), shown in the `PocketGod` group.
- **Created**:
  - `createLogicClasses` table entry 399 (#399 new `BCMultiModel` `EpisodeAlert`)
  - table; listener = view controller.
- **Listener**:
  - `<EpisodeAlertLogicListener>` (`mEpisodeAlertLogicListener`): `optionPreferences`, `isScreenOpen`, `isStoryActive`, `onEpisodeAlertOpenChallengeDetail:`
  - wired by `initLogicListeners` to `PocketGodViewController`
- **Talks to**: `IdolManager` (`sparkle`), `IdolScreenLogic` (`onEpisodeIconPrefChanged:`).
- **Called from**: `activate:` ← IdolManager, IdolScreenLogic; `deactivate`, `onEpisodeIconPrefChanged:` ← IdolScreenLogic; `sparkle` ← IdolManager.
- **Data ids**:
  - behaviours: `EpisodeAlertActivate`, `EpisodeAlertDeactivate`, `EpisodeAlertEvent`, `EpisodeAlertFadeOut`
  - groups: `PocketGod`
  - preferences: `ShowEpisodeIcon`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `checkHiddenAlert:::` @ 0x1001a431c (1 use), `onAlertFadeOutComplete:::` @ 0x1001a4358 (1 use).
- **Persistence**: reads the `Options` preference `ShowEpisodeIcon` in `activate:`.
- **Behaviour**: `activate: node` @ 0x1001a3ff8 shows it (`EpisodeAlertActivate`) when `ShowEpisodeIcon` is on; `sparkle` → `EpisodeAlertEvent` (temporarily showing it even when hidden; `checkHiddenAlert:::` / `onAlertFadeOutComplete:::` fade it out again); `deactivate` → `EpisodeAlertDeactivate`; a tap (no story, no screen) → `onEpisodeAlertOpenChallengeDetail: node`.

#### `IdolScreenLogic`

`BCScreenLogic` · `IdolScreenLogic.c` · 63 methods + 2 item handlers + 1 class method · 201 289 B

- **Role**: the episode-idol screen ("Idols" / challenges and Battle of the Gods UI): start panel and options, idol list, idol detail with challenges, hints and level-up bars, summary tables, the award screen, idol selection (drag idols into the 4 god-idol slots) and the battle screens (enemy selection, battle selection, battle HUD with realm effects, results).
- **Where**: entry 337 (`IdolScreen`, full screen at (0, 0, −277.12)); entries 103–336 are its parts (330–336 are display-only multimodels that no code looks up).
- **Created**:
  - `createLogicClasses` table entry 337 (#337 new `BCMultiModel` `IdolScreen`)
  - entry 337; `+instance` returns `DAT_100821f60`; listener = `IdolManager` (table). Init @ 0x10018de5c (61.6 KB of wiring) looks up 213 table entries between 103 and 329: halos/check marks, `IdolIconLogic`s 121–129, detail/challenge items 130–212, level-up bars/items (213–230; `openIdolDetailPanel:` @ 0x100191b90 also computes them as `getLogic:(i + 1) + 0xd4`, `docs/re/logic.md` §13.7), 43 `BCButtonLogic`s (listener = this screen), battle panels, `IdolBattleRealmEffectLogic`s 270–278 (listener = this screen), `IdolSelectionIconLogic`s 321–328, and its display groups (`IdolIconListDisplayGroup`, detail, library, realm-effect and hint button groups).
- **Listener**:
  - `<IdolScreenLogicListener>` (`mIdolScreenLogicListener`): `optionPreferences`, `onLoadLocationTextureAtlas`, `episodeAlertIcon`, `onEpisodeActivated:`, `onEpisodeIdolSelected:slot:`, `onEpisodeIdolDeselected:`, `selectedEpisodeIdolInSlot:`, `activatedHintCount`, `onHintActivated:`, `activeEpisodeNodeDef`, `onIdolAwardScreenChangeIslandLocationState:`, `onIdolAwardScreenOpenChallenges`, `onIdolAwardScreenClosed`, `onBattleIdolSelected:`, `onBattleEnemySelected:`, `onIdolBattleSelectionPanelOpened`, `playerBattleState`, `enemyBattleState`, `playerName`, `enemyName`, `playerHealth`, `enemyHealth`, `playerMaxHealth`, `enemyMaxHealth`, `onBattleIdolSelectedMultiplayer:`, `isMultiplayerActive`, `isItMyTurn`, `attackName:`, `realmName:`, `didPlayerWinLastBattle`, `didPlayerTieLastBattle`, `isLastBattleIdol:`, `fillPowerDescriptionText:structXmlEpisodeNodeDef:`, `activeBattlePlayerEpisodeIdol`, `activeBattleEnemyEpisodeIdol`, `enemyEpisodeIdol`, `nextBattleIdol:`, `calculateEnemyLevel:`, `calculateEnemyHealth:`, `calculatePlayerLevel`, `calculatePlayerHealth`, `showBattleRealmEffectResult:battleState:opponentBattleState:`, `checkBattleRealmEffectDeath`, `resetAllChallenges`, `onIdolScreenClosed`, `onIdolScreenLaunchStoreScreen`, `openMatchPanel` (never sent by this class: `activeBattleEnemyEpisodeIdol`, `activeBattlePlayerEpisodeIdol`, `onBattleEnemySelected:`, `onBattleIdolSelected:`, `onEpisodeIdolDeselected:`, `onIdolAwardScreenClosed`, `onIdolAwardScreenOpenChallenges`, `onIdolScreenClosed`, `onIdolScreenLaunchStoreScreen`, `openMatchPanel`)
  - wired by `initLogicListeners` to `IdolManager`
- **Talks to**: the 47-method `<IdolScreenLogicListener>` (`IdolManager`: episode activation/selection, hints, battle state, names, health, realm/attack names, multiplayer turns, power descriptions …), `IdolIconLogic`, `IdolSelectionIconLogic`, `IdolBattleRealmEffectLogic` (their select callbacks), `TempleDoorLogic` (`battlePygmyLogic`, `battleEnemyPygmyLogic`, `cancelBattle`), `IdolBattleRealmEffectLogic` (`setTargetXPos:yPos:` lays the player's realm-effect icons out from x = 20 in 40 px steps at y = 260), `EpisodeAlertLogic` (`onEpisodeIconPrefChanged:`), `SkinPackManager` (`loadTextureAtlasChannel:` 0/1 for the icon pages).
  - also sent in code: `EpisodeAlertLogic` (`activate:`, `deactivate`); `GameUiManager` (`showMessage:sequenceId:pos:`, `showMessage:sequenceId:pos:messageLogic:`); `IdolBattleRealmEffectLogic` (`activate:isBuffEffect:`, `deactivate`, `idolBattleState`, `isActive`, `isBuffEffect`, `setBuffSequenceId:debuffSequenceId:`, `setIdolBattleState:`, `structXmlEpisodeNodeDef`); `IdolIconLogic` (`activate:isSelected:`, `deactivate`, `isActive`, `setCheckMarkLogic:`, `setHaloLogic:`, `setLabelId:`, `setSkinTextureBitsDefArray:`, `updateHalo:`, `updateIdolIconTexture`); `IdolManager` (`setIdolLevels:`, `unlockIdolChallenges`); `IdolSelectionIconLogic` (`activate:`, `activateForBattle:isFrozen:`, `activateForBattleEnemy:`, `deactivate`, `dragXPos:yPos:`, `highlightBasedOnAttackEpisodeNodeDef:`, `highlightForBattle`, `idolSelectionIconType`, `isActive`, `isEnemyIdol`, `selectForBattle`, `setAttackTypeLogic:`, `setBattleHomeXPos:yPos:`, `setBattleNameLabelId:statsLabelId:`, `setBattleSelectedLogic:`, `setEnemyStatsLevel:health:`, `setHomeXPos:yPos:`, `setIdolSelectionIconType:`, `setIsEnemyIdol:`, `setLabelId:`, `setRealmTypeLogic:`, `setScrollXPos:`, `setSequenceId:battleEnemySequenceId:battleEnemyHighlightSequenceId:battleEnemyLockedSequenceId:battleSequenceId:battleHighlightSequenceId:battleFrozenSequenceId:`, `setSkinTextureBitsDefArray:`, `stopDrag`, `structXmlEpisodeNodeDef`, `unhighlightForBattle`, `updateIdolIconTexture`).
  - `getLogic:` 103–120 `MultiModelLogic`, 121–129 `IdolIconLogic`, 130–212 `MultiModelLogic`, 222–230 `MultiModelLogic`, 231–236 `BCButtonLogic`, 237–239 `MultiModelLogic`, 240–250 `BCButtonLogic`, 251–254 `MultiModelLogic`, 256 `BCButtonLogic`, 257 `MultiModelLogic`, 259–261 `BCButtonLogic`, 262–263 `MultiModelLogic`, 264–267 `BCButtonLogic`, 268–269 `MultiModelLogic`, 270–278 `IdolBattleRealmEffectLogic`, 279–281 `MultiModelLogic`, 282–283 `BCButtonLogic`, 284 `MultiModelLogic`, 285–300 `BCButtonLogic`, 301–320 `MultiModelLogic`, 321–328 `IdolSelectionIconLogic`, 329 `MultiModelLogic`.
- **Called from**: `areBattleRealmEffectResultsComplete`, `closeIdolBattleSelectionPanel`, `closeScreen`, `closeScreenNow`, `idolScreenMode`, `openIdolAwardScreen:`, `openIdolBattleEnemyScreen`, `openIdolBattleResultsScreen`, `openIdolBattleScreen`, `openIdolDetailScreen:`, `openIdolListScreen:`, `openIdolSelectionScreen`, `openIdolStartScreen`, `refreshIdolBattleSelectionPanel`, `showBattleAttackResults`, `showBattleRealmEffectResults`, `showBattleRealmResults`, `showBattleRealmResultsOnIdol`, `showBattleResurrectionResultsForPlayer:forEnemy:`, `showEnemyPanelMultiplayerAlert:message:`, `updateIdolBattleScreen` ← IdolManager; `openIdolBattleSelectionPanel`, `showBattleAttackStart` ← TempleDoorLogic.
- **Data ids**:
  - behaviours: `BattleAttackResult`, `BattleAttackResultTie`, `BattleAttackStart`, `BattleEffectsEnd`, `BattleEffectsStart`, `BattleIdolWinRealm`, `BattleNegativeRealmEffectResult`, `BattleNegativeResult`, `BattlePositiveRealmEffectResult`, `BattlePositiveResult`, `BattleRealmResult`, `BattleRealmResultTie`, `BattleResult`, `CloseIdolBattleSelectionPanel`, `CloseIdolScreen`, `HideIdolStartCheckBox`, `IdolAreYouSureHide`, `IdolAreYouSureShow`, `IdolAttackTableCounter`, `IdolAttackTableFast`, `IdolAttackTableStrong`, `IdolAttackTypeCounter`, `IdolAttackTypeFast`, `IdolAttackTypeStrong`, `IdolBattleEnemyHighlightIconSlot1`, `IdolBattleEnemyHighlightIconSlot2`, `IdolBattleEnemyHighlightIconSlot3`, `IdolBattleEnemyHighlightIconSlot4`, `IdolBattleEnemyHighlightIconSlot5`, `IdolBattleEnemyHighlightIconSlot6`, `IdolBattleEnemyHighlightIconSlot7`, `IdolBattleEnemyHighlightIconSlot8`, `IdolBattleEnemyLockedIconSlot1`, `IdolBattleEnemyLockedIconSlot2`, `IdolBattleEnemyLockedIconSlot3`, `IdolBattleEnemyLockedIconSlot4`, `IdolBattleEnemyLockedIconSlot5`, `IdolBattleEnemyLockedIconSlot6`, `IdolBattleEnemyLockedIconSlot7`, `IdolBattleEnemyLockedIconSlot8`, … (141 in total)
  - groups: `DisplayObjectSandbox`, `IdolAreYouSurePanelButtons`, `IdolBattleRealmEffectDisplayGroup`, `IdolDetailHeaderButtons`, `IdolDetailHintButtons`, `IdolDetailHintPanelButtons`, `IdolIconDetailDisplayGroup`, `IdolIconListDisplayGroup`, `IdolSelectionLibraryDisplayGroup`
  - sounds: `Button`
  - texts: `BEATS`, `HintsUsed`, `IdolAreYouSureText`, `IdolAwardDescription`, `IdolBattleActive`, `IdolBattleDamage`, `IdolBattleDetails`, `IdolBattleDetailsIdolName`, `IdolBattleEnemyName`, `IdolBattleHealth`, `IdolBattleLevel`, `IdolBattlePlayerName`, `IdolDetailEpisodeName`, `IdolDetailEpisodeNumber`, `IdolDetailHintHeader`, `IdolDetailHintText`, `IdolDetailIdolLevel`, `IdolDetailIdolName`, `IdolIconSlot1Label`, `IdolIconSlot2Label`, `IdolIconSlot3Label`, `IdolIconSlot4Label`, `IdolIconSlot5Label`, `IdolIconSlot6Label`, `IdolIconSlot7Label`, `IdolIconSlot8Label`, `IdolIconSlot9Label`, `IdolResultsExp1`, `IdolResultsExp2`, `IdolResultsExp3`, `IdolResultsExp4`, `IdolSelectionIcon1LevelNum`, `IdolSelectionIcon2LevelNum`, `IdolSelectionIcon3LevelNum`, `IdolSelectionIcon4LevelNum`, `IdolSelectionIcon5LevelNum`, `IdolSelectionIcon5Name`, `IdolSelectionIcon5Stats`, `IdolSelectionIcon6LevelNum`, `IdolSelectionIcon6Name`, … (60 in total)
  - preferences: `ShowBattleHelp`, `ShowEpisodeIcon`, `ShowIdolStartScreen`
  - other strings: `BEATS`, `OpponentName`, `PlayerName`, `TIES`, `power up`, `raw`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `showBattleRealmEffectResults:::` @ 0x100195448 (1 use), `loadLocationTextureAtlas:::` @ 0x100196238 (13 uses).
- **Persistence**: `Options` preferences `ShowIdolStartScreen`, `ShowEpisodeIcon`, `ShowBattleHelp` (ints 0/1; read when the start/battle panels open, toggled by the start-screen, icon and battle-help check boxes in `onButtonPress:touch:`).
- **Behaviour**:
  - `mIdolScreenMode`: 0 start (`openIdolStartPanel`), 1 list (`openIdolListPanel:`), 2 detail (`openIdolDetailPanel:`), 3 award (`openIdolAwardScreen:`), 4 selection (`openIdolSelectionPanel`), 5 battle enemy (`openIdolBattleEnemyPanel`), 6 battle (`openIdolBattleScreen`), 7 battle selection (`openIdolBattleSelectionPanel`), 8 battle results (`openIdolBattleResultsPanel`); `mRootIdolScreenMode` remembers how it was opened. Panels are shown with `setScreenLayer:behavior:` on layers 0/1 and button layers (`docs/re/logic.md` §10).
  - Scrolling lists and the idol library are dragged with inertia (`checkScrollDisplayGroupTouch:touch:allTouches:`, `mDragXPosVel`/`mDragYPosVel`, min/max scroll bounds); selection icons are dragged onto slots (`getIntersectingIdolSelectionIconLogic:touch:`, `getIntersectingSlot:`) → listener `onEpisodeIdolSelected:slot:` / `onEpisodeIdolDeselected:`.
  - Battle HUD: `updateIdolBattleScreen`, realm-effect icons (`manageIdolBattleRealmEffectLogics:…`), attack/realm result sequences (`showBattleAttackStart`, `showBattleAttackResults`, `showBattleRealmResults(OnIdol)`, `showBattleRealmEffectResults…`, `showBattleResurrectionResultsForPlayer:forEnemy:`), `highlightIdolBattleResults:`; `onGameFrame:` @ 0x100195ff8 paces the realm-effect result reveal (`mTimeForNextRealmEffectResult`).
  - `onButtonPress:touch:` @ 0x100196380 (25 KB) dispatches all 43 buttons (close, what's new, options, challenges, skin packs, check boxes, hints, are-you-sure reset, summary, award, battle selection/attack/exit, results retry/next/new match, help).
  - At 201 KB this is the largest class outside `PygmyLogic`; section 5.3 splits its unit into four parts.

### 4.18 Map paper and ad blocker

Two screen helpers outside gameplay: the map paper of the island map and news screens, and the ad blocker.

#### `MapPaperLogic`

`BCLogic` · `MapPaperLogic.c` · 8 methods + 4 item handlers · 3 233 B

- **Role**: the unrolling paper of the island map screen (also used by the news screen).
- **Where**: `MapDisplayGroup/MapPaper`.
- **Created**:
  - `setLogic` item(s) `logicId=MapPaperLogic` in `PocketGod`: `MapDisplayGroup/MapPaper`
  - by `setLogic` with `BCLogic`'s `initWithDisplayObject:` (the class overrides only `init`) → `mMapPaperLogic`; `MapScreen` and `NewsManager` set themselves as listener when they open.
- **Listener**:
  - `<MapPaperLogicListener>` (`mMapPaperLogicListener`): `onMapPaperLogicAnimComplete`, `onMapPaperLogicHideMap`, `onMapPaperLogicShowMap`
  - set in `MapScreen::openMapScreen` @ 0x1000c0d78, `NewsManager::openNewsScreen` @ 0x100055b1c
- **Called from**: `activate` ← MapScreen, NewsManager, PocketGodViewController(Game); `deactivate` ← MapScreen, NewsManager.
- **Data ids**:
  - behaviours: `MapPaperEnter`, `MapPaperExit`
  - other strings: `MapPaperLogic`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `onMapPaperAnimComplete:::` @ 0x10017a20c (0 uses), `onMapPaperEnterComplete:::` @ 0x10017a23c (1 use), `onMapPaperHideContentsComplete:::` @ 0x10017a26c (1 use), `onMapPaperExitComplete:::` @ 0x10017a29c (1 use).
- **Persistence**: none.
- **Behaviour**: game frame stopped; `id` returns `@"MapPaperLogic"` (the only `id` override). `activate` → `MapPaperEnter`, `deactivate` → `MapPaperExit`; items report `onMapPaperLogicShowMap` (enter complete), `onMapPaperLogicHideMap` (contents hidden; also `onHideMap`), `onMapPaperLogicAnimComplete` (exit complete). `onShowMap` is empty.

#### `AdBlockerLogic`

`SpriteLogic` · `AdBlockerLogic.c` · 5 methods + 1 item handler · 2 254 B (the file also holds 15 free functions (about 12.9 KB) of third-party crash-reporter code; not counted)

- **Role**: a full-screen touch blocker shown while a Chartboost advert is up.
- **Where**: entry 550 (`AdBlocker`).
- **Created**:
  - `createLogicClasses` table entry 550 (#550 `MapContentDisplayGroup/AdBlocker`)
  - table; listener = `MapScreen`.
- **Listener**:
  - `<AdBlockerLogicListener>` (`mAdBlockerLogicListener`): `isChartboostAdVisible`
  - wired by `initLogicListeners` to `MapScreen`
- **Data ids**:
  - behaviours: `AdBlockerEnter`
  - other strings: `CFBundleShortVersionString`, `CFBundleVersion`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `waitForChartboostAdHide:::` @ 0x1001a9718 (1 use).
- **Persistence**: none.
- **Behaviour**: `activate` → `AdBlockerEnter`, selectable, shown; `deactivate` hides it; `waitForChartboostAdHide:::` waits until the listener's `isChartboostAdVisible` is NO. (Advertising is not ported; the blocker can be kept inert.)

### 4.19 Framework helpers

The framework helper `OneShotAnimLogic` (`docs/re/logic.md` §11.2) and the unused `TestLogic`.

#### `OneShotAnimLogic`

`SpriteLogic` · `OneShotAnimLogic.c` · 2 methods + 1 item handler · 1 404 B (the file also holds 14 free functions (about 21.8 KB) of `BCTextureDef` texture-atlas management; not counted)

- **Role**: a self-owning fire-and-forget effect (framework class, `docs/re/logic.md` §11.2).
- **Where**: e.g. underwater bubbles.
- **Created**:
  - allocated in `PocketGodViewController(Game)::checkUnderwaterBubble:pos:` @ 0x100163ac4
  - `initWithSequenceId:` @ 0x10003794c on a new `OneShot` multimodel (retains itself); `PocketGodViewController(Game)::checkUnderwaterBubble:pos:` spawns `UnderwaterBubbles` at the touch (at most every 0.4 s).
- **Listener**: none.
- **Called from**: `initWithSequenceId:` ← PocketGodViewController(Game).
- **Data ids**:
  - models: `OneShot`
  - resources: `png`, `raw`
  - other strings: `Default`, `TextureDef`
- **Item handlers** (semantics: `docs/re/sequences.md` §5): `oneShotComplete:::` @ 0x100037a74 (1 use).
- **Persistence**: none.
- **Behaviour**: game frame stopped; the sequence must end with `oneShotComplete:::` (removes, shuts down, releases).

#### `TestLogic`

`BCLogic` · `TestLogic.c` · 2 methods · 614 B

- **Role**: empty test class (framework, `docs/re/logic.md` §1); never instantiated.
- **Created**: never instantiated (no class reference outside its own methods).
- **Listener**: none.
- **Persistence**: none.
- **Behaviour**: only `initWithDisplayObject:` (calls `BCLogic`) and `dealloc`. Not ported.

## 5. Implementation units

The units partition the 149 classes into pieces of at most about 120 KB of decompiled code. A unit keeps a feature together, so a unit can be ported and tested as a whole. `depends_on` lists the units whose classes this unit sends messages to, or whose shared code it reuses (the throwable base of section 2.2 lives in `island_core`). The list is acyclic and in file order. Calls in the opposite direction exist and are listed in section 5.2. Every unit also talks to `PygmyLogic` (`tmp/re/pygmy-partition.json`) and to the view controller through its listener protocols.

### 5.1 Units

| Unit | Classes | Bytes | Depends on | Scope |
|---|---|---|---|---|
| `sky_weather` | `SunLogic`, `MoonLogic`, `CloudLogic`, `StormCloudLogic`, `OceanLogic`, `FireworkLogic`, `BatLogic` | 62 749 | – | Sky and weather layers of the surface islands: sun (with eclipse overlay), moon and stars, drifting clouds (also the three Ooga Jump sky clouds), storm-cloud and ocean tile bands, fireworks and the night bat. |
| `sky_powers` | `LightningLogic`, `LightningModel`, `RainbowLogic`, `RainbowModel`, `HurricaneLogic`, `HurricaneModel`, `WhirlPoolLogic`, `WhirlPoolModel`, `FingerTrailLogic` | 115 041 | – | Gesture powers drawn with the finger and their custom models: lightning (LightningModel), rainbow (RainbowModel), hurricane (HurricaneModel; uses the SpriteLogic hurricane hooks of every object), whirlpool (WhirlPoolModel) and the finger-trail glow. |
| `island_core` | `CoconutLogic`, `WoodLogic`, `BonFireLogic` | 89 280 | – | Coconut, firewood and bonfire. CoconutLogic is the reference implementation of the shared throwable-object physics (gravity, tilt, island floor, spring drag; catalogue section 2.2), which this unit should provide as reusable code for every other throwable. |
| `island_objects` | `SpearLogic`, `BoulderLogic`, `PygmyBottomLogic`, `VolcanoLogic`, `LavaLogic`, `OutHouseLogic`, `OctoStatueLogic` | 105 989 | `island_core` | Other main-island props: spear, boulder (bowling, crushing, eclipse), detached pygmy bottoms and locust bones, volcano and lava, rock-island outhouse and sand-island octopus statue. |
| `island_anthill` | `AntHillLogic`, `AntLogic`, `MagnifyingGlassLogic`, `MagnifyingBeamModel` | 70 823 | `island_core` | Ant hill and its ant swarm, and the magnifying glass with its sun beam (MagnifyingBeamModel). |
| `ocean_fishing` | `FishLogic`, `FishingPoleLogic`, `FishingBobLogic` | 98 150 | `island_core` | Fish (swim, hook, cook, eat) and the fishing pole with line and bob. |
| `ocean_creatures` | `SwordFishLogic`, `SharkLogic` | 67 612 | `island_core`, `ocean_fishing` | Swordfish (caught through the ice hole) and shark (attacks on pygmies near the water, hooking with the fishing bob, dropping onto pygmies). |
| `ocean_birds` | `BirdLogic`, `BirdPooLogic` | 58 627 | `island_core`, `ocean_fishing` | Seagull (fish and pygmy dives, cooking by lightning) and its droppings. |
| `underwater_baitmaster_laser` | `AnchorLogic`, `PiranhaLogic`, `PathModel`, `ChumBucketLogic`, `ChumLogic`, `LaserSharkLogic`, `LaserGunLogic`, `LaserTargetLogic` | 119 174 | – | Bait Master (anchor, piranhas following finger-drawn paths drawn by PathModel) and the chum bucket with chum, laser shark, laser gun and tilt-steered laser target. |
| `underwater_paindrain` | `TileDrainLogic`, `TilePlugLogic`, `PainDrainPlatformLogic` | 44 987 | – | Pain Drain: the floor drain, its plug and the rising platforms of the climbing mini-game. |
| `underwater_stage` | `TsunamiLogic`, `UnderwaterSceneryLogic`, `SpearStatueLogic`, `ChainSpearLogic`, `UnderwaterTempleLogic`, `UnderwaterTempleSymbolLogic`, `JellyfishLogic` | 103 738 | `underwater_baitmaster_laser`, `underwater_paindrain` | Underwater stage: the tsunami transition (front and behind layers, stage manager of the ruins props), drifting scenery, the spear statue and chain spear, the coral-reef temple and its nine symbols, and the jellyfish. The tsunami moves the props of the two other underwater units in and out through their transitionIn/transitionOut methods. |
| `nopee` | `WaterWingsLogic`, `NoPeeLogic`, `NoPeeObjectLogic`, `NoPeeTreasureLogic`, `NoPeeSeaBlobLogic`, `NoPeeFishLogic`, `NoPeeMineLogic`, `NoPeeClamLogic`, `NoPeeParticleLogic`, `NoPeeMapLogic`, `NoPeeMapItemLogic`, `NoPeeMapScreenLogic`, `NoPeePillarLogic` | 110 969 | `island_core` | Water wings (a throwable that starts the game) and the No Pee mini-game: host, world objects (treasure, sea blobs, fish, mines, clams, particles), minimap with breath meter. NoPeePillarLogic is dead code and may be skipped. |
| `ice_island` | `IceMonsterLogic`, `SnowBallLogic`, `HealthBarLogic`, `GlacierLogic`, `IglooLogic`, `IceTreeLogic`, `IceHoleLogic`, `IcicleLogic` | 84 294 | `ocean_fishing`, `ocean_creatures` | Ice island: ice monster boss with snowballs, the shared boss health bar, glacier, igloo, tree, ice-fishing hole (with the squid) and icicles. |
| `monsters` | `TRexEggLogic`, `TRexLogic`, `SpiderLogic`, `SpiderTreeLogic`, `ZombieGraveLogic` | 110 553 | `island_core`, `island_objects`, `ice_island` | Spear-fought monsters: T-Rex egg and T-Rex (rock island), giant spider with its web tree, and the zombie grave (graveyard island). |
| `misfit_oogajump` | `IslandObjectLogic`, `DreamBubbleLogic`, `DoodleCloudLogic` | 19 338 | – | Misfit-island ocean rock (IslandObjectLogic), the dream bubble that leads there, and the Ooga Jump platform clouds. |
| `moronpests_quiz` | `MoronPestsQuestionScreenLogic`, `MoronDuckLogic`, `MoronBeeLogic` | 88 610 | – | Moron Pests quiz overlay (question generation, answer checking, timeouts) with its ducks and bees. |
| `moronpests_chase` | `MoronSnakeLogic`, `MoronTurtleLogic` | 42 414 | `island_core`, `moronpests_quiz` | Moron Pests host: the Misfit-island snake (a throwable) that starts the chase, and the turtle that pursues the pygmy between questions. |
| `ape_ritual` | `ApeLogic`, `ApeAltarLogic`, `ApeBongoLogic`, `ApeWenchLogic`, `ApeTorchLogic` | 61 329 | – | Ape-island summoning ritual: ape, altar, bongo, wench (winch) and the two torches. |
| `ape_monkeys` | `MonkeyLogic`, `BananaTreeLogic`, `BananaLogic` | 120 529 | `island_core`, `ape_ritual` | The three monkeys (wandering, climbing, banana eating, pygmy catch and humping), the banana tree and the banana (a throwable). |
| `konkeydong` | `KongStatueLogic`, `KonkeyDongLogic`, `KonkeyDongPlatformLogic`, `KonkeyDongLadderLogic`, `KonkeyDongPickLogic`, `KonkeyDongBeetleLogic`, `KonkeyDongBoulderLogic` | 101 674 | – | Konkey Dong mini-game: Kong statue host with the ten level layouts, Konkey Dong, girders, ladders, pick, beetles and boulders. |
| `theruns` | `CrackHoleLogic`, `TheRunsPlatformLogic`, `TheRunsStalactiteLogic`, `TheRunsRockLogic`, `TheRunsTurdLogic` | 70 238 | – | The Runs mini-game: the crack hole host with the endless-runner course, platforms, stalactites, rocks and turds. |
| `apocalypse_temples` | `ApocalypseCalendarLogic`, `ApocalypseCloudLogic`, `ApocalypseGemLogic`, `ApocalypseGemPedestalLogic`, `ApocalypseRoomFaderLogic`, `ApocalypseTempleDoorLogic`, `ApocalypseTempleFountainLogic`, `ApocalypseTimeTempleLeverLogic`, `ApocalypseTimeTemplePlatformLogic` | 112 017 | `island_core` | Apocalypse framework: calendar clock and portal, disease clouds, the seven gems (throwables) and their pedestals, room fader, temple doors, plague/purification fountains and the Time-temple lever and platform. |
| `apocalypse_locust_closet_xenu` | `LocustLogic`, `LocustModel`, `LocustQueenLogic`, `ClosetTempleCouchLogic`, `ClosetTempleClosetDoorLogic`, `ClosetTempleThetanMeterLogic`, `ThetanLogic`, `XenuLogic`, `XenuTempleLogic` | 117 271 | – | Locust temple (swarm with LocustModel, queen), Closet temple (couch, sliding closet door, thetan meter, thetans) and the Xenu room (Xenu, breakable wall). |
| `apocalypse_dance` | `DanceDanceExecutionLogic`, `DanceDanceExecutionScreenLogic`, `DiscoBallLogic`, `DiscoLightsLogic`, `GoGoCageLogic`, `DanceBalloonBundleLogic`, `DanceBalloonLogic` | 80 715 | – | Dance temple: Dance Dance Execution stage and rhythm-game screen (beat chart from dance-beats.json), disco ball and lights, go-go cage, balloon bundle and balloon. |
| `idol_ui_widgets` | `IdolIconLogic`, `IdolSelectionIconLogic`, `IdolBattleRealmEffectLogic`, `EpisodeAlertLogic`, `MapPaperLogic`, `AdBlockerLogic`, `OneShotAnimLogic`, `TestLogic` | 53 528 | – | Small UI logic objects: idol list icons, draggable idol selection cards, battle realm-effect icons, the episode alert icon, the map paper, the ad blocker, plus the framework OneShotAnimLogic and the unused TestLogic. |
| `idol_screen` | `IdolScreenLogic` | 201 289 | `idol_ui_widgets` | IdolScreenLogic, the episode-idol and Battle of the Gods screen. It is one class of about 201 KB, above the unit budget, so it is split internally into init and wiring, input, battle HUD and panels (catalogue section 5.3). |
| `battle` | `TempleDoorLogic`, `GodIdolLogic`, `IdolPedestalLogic`, `BattleAlertLogic`, `BattleEnergyBallLogic`, `FailBoatLogic` | 90 655 | `island_core`, `idol_screen` | God island and Battle of the Gods: temple door (battle host), god idols (throwables), idol pedestals (persisted slots), battle alert banner, energy balls and the Fail Boat. |

Total: 2 401 593 bytes in 27 units. `ape_monkeys` (120 529 B) is at the budget. `idol_screen` (201 289 B) is one class and exceeds it, so section 5.3 splits it internally.

### 5.2 Calls against the dependency order

These sends go from a unit to a unit that depends on it. In a port they should go through a trait or an event that the earlier unit defines, as the original does through its listener protocols:

- `island_core` → `ocean_fishing`: `BonFireLogic` → `FishingPoleLogic` (`checkProximityToBonFire:`).
- `island_objects` → `monsters`: `SpearLogic` → `SpiderLogic` (`canBeHitBySpear:`, `hitBySpear:`, `isActive`); `SpearLogic` → `TRexLogic` (`canBeHitBySpear:`, `hitBySpear:`, `isActive`).
- `ocean_fishing` → `ice_island`: `FishLogic` → `IceHoleLogic` (`fishFallIntoIceHole:`, `isAvailableForFalling`); `FishLogic` → `IglooLogic` (`fishFallIntoIgloo:`, `isFishFallAvailable:`).
- `moronpests_quiz` → `moronpests_chase`: `MoronPestsQuestionScreenLogic` reports to its listener `MoronSnakeLogic`.
- `ape_ritual` → `ape_monkeys`: `ApeTorchLogic` → `MonkeyLogic` (`apeTorchClimbComplete`, `bootedFromApeTorch`, `bootedFromApeTorchByFire`).
- `idol_ui_widgets` → `idol_screen`: `IdolBattleRealmEffectLogic` reports to its listener `IdolScreenLogic`; `IdolIconLogic` reports to its listener `IdolScreenLogic`; `IdolSelectionIconLogic` reports to its listener `IdolScreenLogic`.
- `idol_screen` → `battle`: `IdolScreenLogic` → `TempleDoorLogic` (`battleEnemyPygmyLogic`, `battlePygmyLogic`, `cancelBattle`).

### 5.3 Splitting `idol_screen`

`IdolScreenLogic` is a single class of 66 functions. Its methods fall into four parts that can be written as separate Rust modules over one shared screen state (sizes measured on `tmp/re/decomp/game/IdolScreenLogic.c`):

| Part | Contents | Methods | Bytes |
|---|---|---|---|
| `init_and_wiring` | construction and wiring of the 213 table parts, dealloc, accessors | 7 | 64 195 |
| `input` | button dispatch, touches, scrolling, drag and drop of selection icons, widget callbacks | 17 | 41 320 |
| `battle_hud` | battle enemy and selection panels, battle screen, attack, realm and resurrection results, results panel | 27 | 66 157 |
| `panels` | start, list, detail, summary, award and selection panels and their icon refresh | 15 | 33 202 |

`init_and_wiring` is almost entirely the 61.6 KB `initWithDisplayObject:`, which looks up 213 table entries between 103 and 329 and wires them. In a port that is data (an id table) and not code, so the remaining parts are the real work.

## 6. Partition file

`tmp/re/logic-partition.json` is the partition of section 5 as data. It is a JSON array with one object per unit, in the order of section 5.1. Keys appear in the order below. The file is UTF-8 with one-space indentation and a trailing newline, and the build regenerates it byte-for-byte. Every catalogued class appears in exactly one unit.

| Field | Type | Meaning |
|---|---|---|
| `unit` | string | unit id, snake_case, unique |
| `description` | string | scope of the unit, one or two sentences |
| `classes` | string[] | class names (Objective-C spelling), in catalogue order |
| `bytes` | integer | sum of the decompiled bytes of the classes (section 1.1) |
| `depends_on` | string[] | ids of units that this unit calls into or reuses; always earlier in the array; `[]` when none |

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LogicUnit {
    pub unit: String,
    pub description: String,
    pub classes: Vec<String>,
    pub bytes: u64,
    pub depends_on: Vec<String>,
}

// let units: Vec<LogicUnit> = serde_json::from_str(&text)?;
```

## 7. Open questions, quirks and corrections

### 7.1 Open questions

These points could not be settled from the decompiled text alone. Each needs a dedicated
disassembly pass before the code that depends on it is written.

1. **`TheRunsPlatformEvent` layout** (0x24 bytes, `CrackHoleLogic.mTheRunsPlatformEventArray`).
   The platform type (4 = crumbling), the position and the hazard choices are identified
   (`CrackHoleLogic::chooseTheRunsPlatformEvent: @ 0x1001260f4`, `processPlatformEvents:`). The
   meaning of every remaining field is not.
2. **`LocustState` layout** (60 records of 0xB0 bytes at `0x1007e50a8`). The fields named in
   the `LocustLogic` entry are the ones the state machine visibly uses
   (`LocustLogic::runLocustState:deltaTime: @ 0x10008c4e8`). The other bytes are unmapped.
3. **Konkey Dong movement details.** The beetle's and boulder's girder and ladder logic is
   summarised (`KonkeyDongBeetleLogic::checkKonkeyDongCollision:`,
   `KonkeyDongBoulderLogic::checkKonkeyDongCollision: @ 0x10017ecf8`). The exact ladder-skip
   counters and roll timings still need a line-by-line transcription.
4. **`MonkeyLogic` (77 KB)** is catalogued at the level of its state machine and per-frame
   checks. The distance thresholds and timers of the individual checks
   (`checkForSleepingPygmy`, `checkForBananaTree(DuringFall)`, the catch game) are not all
   listed.
5. **`IdolScreenLogic` (201 KB)** is catalogued at the level of its panels and modes. The
   wiring of the 213 table entries it uses lives in `IdolScreenLogic::initWithDisplayObject: @ 0x10018de5c`,
   and the 43-button dispatcher is `onButtonPress:touch: @ 0x100196380`. Both need their own
   pass (unit `idol_screen`, section 5.3).
6. **Moron Pests answer order.** The expected order of the bee and duck questions comes from
   sorting the pests with the comparators `FUN_100063d60` … `FUN_1000644a4` through
   `sortUsingFunction:context:`. The order of pests with equal keys (for example two ducks with
   the same eye scale) depends on the stability of Foundation's sort, which the binary does
   not show.
7. **Receivers of sends to array elements.** Messages sent to objects taken from arrays (the
   listener's `pygmyLogicArray:`, `spearLogicArray:`, `underwaterSceneryLogicArray:`, …) have no
   static receiver type. They appear under "Talks to" only when the selector belongs to one
   class or when the handwritten summary names the receiver after reading the code (section
   1.2).

### 7.2 Quirks of the original (reproduce them)

- `IceMonsterLogic::throwSnowBallXOffset:yOffset:range: @ 0x100184274` picks the target x in
  the window `[p − range, p + range]` around the first pygmy. When the window starts left of
  20, the code sets `lo = 20` first and only then adds `20 − lo` (now 0) to `hi`. Only the
  lower bound moves, while the right-edge clamp shifts both bounds as intended (checked in
  the disassembly around `0x100184474`). With `range = 0` (`animThrowRight`) and a pygmy left
  of x = 20, the window is empty. The unsigned modulus then yields a target millions of
  points away, and the snowball leaves the screen at once.
- Table entry 500 (`DanceDanceExecutionLogic`) sets the apocalypse-cloud style `DDE`, but no
  sequence `ApocalypseCloudDDEActivate` exists, so an infected dance stage shows no cloud
  animation.
- `+[ApocalypseTempleFountainLogic instance]` returns the fountain created last (entry 507,
  purification), because both fountains store themselves in the same static.
- `DoodleCloudLogic::moveToTop @ 0x1000ccab0` writes the jet-pack spacing rule as a comparison
  of two booleans. It reduces to `d > (pending ? 3200 : 1280)`.
- Code that is never reached in 1.48.2: `NoPeePillarLogic` and `TestLogic` (never
  instantiated); `FailBoatLogic::fallIntoFailBoat:`, `ejectFromFailBoat:`, `enterIsland:`,
  `exitIsland:` and `exitIslandNow` (never sent); `SpearLogic::startHolding:`;
  `IceHoleLogic::tripIntoIceHole:fallForward:`. A port may omit them.
- These listeners are never set: `KonkeyDongPickLogic`, `TheRunsRockLogic`, `TheRunsTurdLogic`,
  `BattleEnergyBallLogic` and `ApocalypseCloudLogic`. Their protocols exist but nothing assigns
  them. `HealthBarLogic` has no listener until a boss sets itself, and its `onGameFrame:` does
  nothing until then.

### 7.3 Corrections to other documents

- `docs/re/sequences.md` §3.14 gives the behind storm cloud a `posVelScale` of 0.5. The code
  passes 0.8 (`0x3f4ccccd`). 0.5 is the behind ocean's value
  (`PocketGodViewController(Game)::setLogic::: @ 0x10015c11c`).
- `docs/re/sequences.md` §3.14 says that every logic created by `setLogic` is added to
  `mLogicArray`. Coconuts go only to `mCoconutLogicArray`.
- `docs/re/logic.md` appendix A lists the No Pee object listener protocols with no methods.
  They inherit `<NoPeeObjectLogicListener>` (`noPeePygmyLogic`, `noPeeLogic`,
  `noPeeCollidableLogicArray`, `isGameActive`), which `NoPeeLogic` implements.

## Appendix A. Class index

| Class | Superclass | Cluster | Unit | Bytes |
|---|---|---|---|---|
| `AdBlockerLogic` | `SpriteLogic` | 4.18 | `idol_ui_widgets` | 2 254 |
| `AnchorLogic` | `SpriteLogic` | 4.4 | `underwater_baitmaster_laser` | 22 725 |
| `AntHillLogic` | `SpriteLogic` | 4.2 | `island_anthill` | 24 308 |
| `AntLogic` | `SpriteLogic` | 4.2 | `island_anthill` | 10 014 |
| `ApeAltarLogic` | `SpriteLogic` | 4.11 | `ape_ritual` | 22 996 |
| `ApeBongoLogic` | `SpriteLogic` | 4.11 | `ape_ritual` | 6 590 |
| `ApeLogic` | `SpriteLogic` | 4.11 | `ape_ritual` | 11 835 |
| `ApeTorchLogic` | `SpriteLogic` | 4.11 | `ape_ritual` | 7 402 |
| `ApeWenchLogic` | `SpriteLogic` | 4.11 | `ape_ritual` | 12 506 |
| `ApocalypseCalendarLogic` | `SpriteLogic` | 4.15 | `apocalypse_temples` | 20 128 |
| `ApocalypseCloudLogic` | `SpriteLogic` | 4.15 | `apocalypse_temples` | 9 885 |
| `ApocalypseGemLogic` | `SpriteLogic` | 4.15 | `apocalypse_temples` | 32 019 |
| `ApocalypseGemPedestalLogic` | `SpriteLogic` | 4.15 | `apocalypse_temples` | 11 896 |
| `ApocalypseRoomFaderLogic` | `SpriteLogic` | 4.15 | `apocalypse_temples` | 2 813 |
| `ApocalypseTempleDoorLogic` | `SpriteLogic` | 4.15 | `apocalypse_temples` | 11 127 |
| `ApocalypseTempleFountainLogic` | `SpriteLogic` | 4.15 | `apocalypse_temples` | 7 617 |
| `ApocalypseTimeTempleLeverLogic` | `SpriteLogic` | 4.15 | `apocalypse_temples` | 7 372 |
| `ApocalypseTimeTemplePlatformLogic` | `SpriteLogic` | 4.15 | `apocalypse_temples` | 9 160 |
| `BananaLogic` | `SpriteLogic` | 4.11 | `ape_monkeys` | 31 804 |
| `BananaTreeLogic` | `SpriteLogic` | 4.11 | `ape_monkeys` | 11 367 |
| `BatLogic` | `SpriteLogic` | 4.1 | `sky_weather` | 23 520 |
| `BattleAlertLogic` | `SpriteLogic` | 4.16 | `battle` | 6 547 |
| `BattleEnergyBallLogic` | `SpriteLogic` | 4.16 | `battle` | 3 563 |
| `BirdLogic` | `SpriteLogic` | 4.3 | `ocean_birds` | 47 345 |
| `BirdPooLogic` | `SpriteLogic` | 4.3 | `ocean_birds` | 11 282 |
| `BonFireLogic` | `SpriteLogic` | 4.2 | `island_core` | 19 963 |
| `BoulderLogic` | `SpriteLogic` | 4.2 | `island_objects` | 32 069 |
| `ChainSpearLogic` | `SpriteLogic` | 4.4 | `underwater_stage` | 11 768 |
| `ChumBucketLogic` | `SpriteLogic` | 4.4 | `underwater_baitmaster_laser` | 13 006 |
| `ChumLogic` | `SpriteLogic` | 4.4 | `underwater_baitmaster_laser` | 15 107 |
| `ClosetTempleClosetDoorLogic` | `SpriteLogic` | 4.15 | `apocalypse_locust_closet_xenu` | 13 861 |
| `ClosetTempleCouchLogic` | `SpriteLogic` | 4.15 | `apocalypse_locust_closet_xenu` | 3 559 |
| `ClosetTempleThetanMeterLogic` | `SpriteLogic` | 4.15 | `apocalypse_locust_closet_xenu` | 20 953 |
| `CloudLogic` | `SpriteLogic` | 4.1 | `sky_weather` | 8 162 |
| `CoconutLogic` | `SpriteLogic` | 4.2 | `island_core` | 35 904 |
| `CrackHoleLogic` | `SpriteLogic` | 4.13 | `theruns` | 54 242 |
| `DanceBalloonBundleLogic` | `SpriteLogic` | 4.15 | `apocalypse_dance` | 4 262 |
| `DanceBalloonLogic` | `SpriteLogic` | 4.15 | `apocalypse_dance` | 14 846 |
| `DanceDanceExecutionLogic` | `SpriteLogic` | 4.15 | `apocalypse_dance` | 9 944 |
| `DanceDanceExecutionScreenLogic` | `BCScreenLogic` | 4.15 | `apocalypse_dance` | 43 910 |
| `DiscoBallLogic` | `SpriteLogic` | 4.15 | `apocalypse_dance` | 2 673 |
| `DiscoLightsLogic` | `SpriteLogic` | 4.15 | `apocalypse_dance` | 1 547 |
| `DoodleCloudLogic` | `SpriteLogic` | 4.10 | `misfit_oogajump` | 10 296 |
| `DreamBubbleLogic` | `SpriteLogic` | 4.9 | `misfit_oogajump` | 7 439 |
| `EpisodeAlertLogic` | `SpriteLogic` | 4.17 | `idol_ui_widgets` | 7 201 |
| `FailBoatLogic` | `SpriteLogic` | 4.16 | `battle` | 3 377 |
| `FingerTrailLogic` | `SpriteLogic` | 4.1 | `sky_powers` | 1 412 |
| `FireworkLogic` | `SpriteLogic` | 4.1 | `sky_weather` | 8 386 |
| `FishLogic` | `SpriteLogic` | 4.3 | `ocean_fishing` | 46 662 |
| `FishingBobLogic` | `SpriteLogic` | 4.3 | `ocean_fishing` | 14 580 |
| `FishingPoleLogic` | `SpriteLogic` | 4.3 | `ocean_fishing` | 36 908 |
| `GlacierLogic` | `SpriteLogic` | 4.6 | `ice_island` | 1 892 |
| `GoGoCageLogic` | `SpriteLogic` | 4.15 | `apocalypse_dance` | 3 533 |
| `GodIdolLogic` | `SpriteLogic` | 4.16 | `battle` | 34 884 |
| `HealthBarLogic` | `SpriteLogic` | 4.6 | `ice_island` | 5 692 |
| `HurricaneLogic` | `SpriteLogic` | 4.1 | `sky_powers` | 25 738 |
| `HurricaneModel` | `BCDisplayObject` | 4.1 | `sky_powers` | 5 094 |
| `IceHoleLogic` | `SpriteLogic` | 4.6 | `ice_island` | 25 276 |
| `IceMonsterLogic` | `SpriteLogic` | 4.6 | `ice_island` | 23 385 |
| `IceTreeLogic` | `SpriteLogic` | 4.6 | `ice_island` | 1 720 |
| `IcicleLogic` | `SpriteLogic` | 4.6 | `ice_island` | 5 225 |
| `IdolBattleRealmEffectLogic` | `BCLogic` | 4.17 | `idol_ui_widgets` | 7 111 |
| `IdolIconLogic` | `BCLogic` | 4.17 | `idol_ui_widgets` | 6 954 |
| `IdolPedestalLogic` | `SpriteLogic` | 4.16 | `battle` | 13 565 |
| `IdolScreenLogic` | `BCScreenLogic` | 4.17 | `idol_screen` | 201 289 |
| `IdolSelectionIconLogic` | `BCLogic` | 4.17 | `idol_ui_widgets` | 24 757 |
| `IglooLogic` | `SpriteLogic` | 4.6 | `ice_island` | 12 113 |
| `IslandObjectLogic` | `SpriteLogic` | 4.9 | `misfit_oogajump` | 1 603 |
| `JellyfishLogic` | `SpriteLogic` | 4.4 | `underwater_stage` | 37 190 |
| `KongStatueLogic` | `SpriteLogic` | 4.12 | `konkeydong` | 44 074 |
| `KonkeyDongBeetleLogic` | `SpriteLogic` | 4.12 | `konkeydong` | 21 157 |
| `KonkeyDongBoulderLogic` | `SpriteLogic` | 4.12 | `konkeydong` | 12 789 |
| `KonkeyDongLadderLogic` | `SpriteLogic` | 4.12 | `konkeydong` | 5 172 |
| `KonkeyDongLogic` | `SpriteLogic` | 4.12 | `konkeydong` | 6 234 |
| `KonkeyDongPickLogic` | `SpriteLogic` | 4.12 | `konkeydong` | 2 722 |
| `KonkeyDongPlatformLogic` | `SpriteLogic` | 4.12 | `konkeydong` | 9 526 |
| `LaserGunLogic` | `SpriteLogic` | 4.4 | `underwater_baitmaster_laser` | 4 927 |
| `LaserSharkLogic` | `SpriteLogic` | 4.4 | `underwater_baitmaster_laser` | 15 526 |
| `LaserTargetLogic` | `SpriteLogic` | 4.4 | `underwater_baitmaster_laser` | 4 007 |
| `LavaLogic` | `SpriteLogic` | 4.2 | `island_objects` | 5 047 |
| `LightningLogic` | `SpriteLogic` | 4.1 | `sky_powers` | 8 799 |
| `LightningModel` | `BCDisplayObject` | 4.1 | `sky_powers` | 9 505 |
| `LocustLogic` | `SpriteLogic` | 4.15 | `apocalypse_locust_closet_xenu` | 38 822 |
| `LocustModel` | `BCDisplayObject` | 4.15 | `apocalypse_locust_closet_xenu` | 1 619 |
| `LocustQueenLogic` | `SpriteLogic` | 4.15 | `apocalypse_locust_closet_xenu` | 9 017 |
| `MagnifyingBeamModel` | `BCDisplayObject` | 4.2 | `island_anthill` | 5 521 |
| `MagnifyingGlassLogic` | `SpriteLogic` | 4.2 | `island_anthill` | 30 980 |
| `MapPaperLogic` | `BCLogic` | 4.18 | `idol_ui_widgets` | 3 233 |
| `MonkeyLogic` | `SpriteLogic` | 4.11 | `ape_monkeys` | 77 358 |
| `MoonLogic` | `SpriteLogic` | 4.1 | `sky_weather` | 4 405 |
| `MoronBeeLogic` | `SpriteLogic` | 4.14 | `moronpests_quiz` | 6 482 |
| `MoronDuckLogic` | `SpriteLogic` | 4.14 | `moronpests_quiz` | 7 977 |
| `MoronPestsQuestionScreenLogic` | `BCScreenLogic` | 4.14 | `moronpests_quiz` | 74 151 |
| `MoronSnakeLogic` | `SpriteLogic` | 4.14 | `moronpests_chase` | 35 569 |
| `MoronTurtleLogic` | `SpriteLogic` | 4.14 | `moronpests_chase` | 6 845 |
| `NoPeeClamLogic` | `NoPeeObjectLogic` | 4.5 | `nopee` | 3 275 |
| `NoPeeFishLogic` | `NoPeeObjectLogic` | 4.5 | `nopee` | 4 290 |
| `NoPeeLogic` | `SpriteLogic` | 4.5 | `nopee` | 27 433 |
| `NoPeeMapItemLogic` | `SpriteLogic` | 4.5 | `nopee` | 3 330 |
| `NoPeeMapLogic` | `SpriteLogic` | 4.5 | `nopee` | 6 616 |
| `NoPeeMapScreenLogic` | `SpriteLogic` | 4.5 | `nopee` | 1 852 |
| `NoPeeMineLogic` | `NoPeeObjectLogic` | 4.5 | `nopee` | 2 804 |
| `NoPeeObjectLogic` | `SpriteLogic` | 4.5 | `nopee` | 7 766 |
| `NoPeeParticleLogic` | `NoPeeObjectLogic` | 4.5 | `nopee` | 3 921 |
| `NoPeePillarLogic` | `SpriteLogic` | 4.5 | `nopee` | 5 247 |
| `NoPeeSeaBlobLogic` | `NoPeeObjectLogic` | 4.5 | `nopee` | 7 957 |
| `NoPeeTreasureLogic` | `NoPeeObjectLogic` | 4.5 | `nopee` | 4 595 |
| `OceanLogic` | `SpriteLogic` | 4.1 | `sky_weather` | 2 852 |
| `OctoStatueLogic` | `SpriteLogic` | 4.2 | `island_objects` | 6 236 |
| `OneShotAnimLogic` | `SpriteLogic` | 4.19 | `idol_ui_widgets` | 1 404 |
| `OutHouseLogic` | `SpriteLogic` | 4.2 | `island_objects` | 8 088 |
| `PainDrainPlatformLogic` | `SpriteLogic` | 4.4 | `underwater_paindrain` | 5 255 |
| `PathModel` | `BCDisplayObject` | 4.4 | `underwater_baitmaster_laser` | 7 355 |
| `PiranhaLogic` | `SpriteLogic` | 4.4 | `underwater_baitmaster_laser` | 36 521 |
| `PygmyBottomLogic` | `SpriteLogic` | 4.2 | `island_objects` | 7 563 |
| `RainbowLogic` | `SpriteLogic` | 4.1 | `sky_powers` | 33 401 |
| `RainbowModel` | `BCDisplayObject` | 4.1 | `sky_powers` | 11 658 |
| `SharkLogic` | `SpriteLogic` | 4.3 | `ocean_creatures` | 34 742 |
| `SnowBallLogic` | `SpriteLogic` | 4.6 | `ice_island` | 8 991 |
| `SpearLogic` | `SpriteLogic` | 4.2 | `island_objects` | 39 377 |
| `SpearStatueLogic` | `SpriteLogic` | 4.4 | `underwater_stage` | 6 562 |
| `SpiderLogic` | `SpriteLogic` | 4.8 | `monsters` | 28 514 |
| `SpiderTreeLogic` | `SpriteLogic` | 4.8 | `monsters` | 7 505 |
| `StormCloudLogic` | `SpriteLogic` | 4.1 | `sky_weather` | 7 297 |
| `SunLogic` | `SpriteLogic` | 4.1 | `sky_weather` | 8 127 |
| `SwordFishLogic` | `SpriteLogic` | 4.3 | `ocean_creatures` | 32 870 |
| `TRexEggLogic` | `SpriteLogic` | 4.7 | `monsters` | 33 147 |
| `TRexLogic` | `SpriteLogic` | 4.7 | `monsters` | 30 641 |
| `TempleDoorLogic` | `SpriteLogic` | 4.16 | `battle` | 28 719 |
| `TestLogic` | `BCLogic` | 4.19 | `idol_ui_widgets` | 614 |
| `TheRunsPlatformLogic` | `SpriteLogic` | 4.13 | `theruns` | 5 876 |
| `TheRunsRockLogic` | `SpriteLogic` | 4.13 | `theruns` | 3 253 |
| `TheRunsStalactiteLogic` | `SpriteLogic` | 4.13 | `theruns` | 3 615 |
| `TheRunsTurdLogic` | `SpriteLogic` | 4.13 | `theruns` | 3 252 |
| `ThetanLogic` | `SpriteLogic` | 4.15 | `apocalypse_locust_closet_xenu` | 16 662 |
| `TileDrainLogic` | `SpriteLogic` | 4.4 | `underwater_paindrain` | 25 094 |
| `TilePlugLogic` | `SpriteLogic` | 4.4 | `underwater_paindrain` | 14 638 |
| `TsunamiLogic` | `SpriteLogic` | 4.4 | `underwater_stage` | 17 242 |
| `UnderwaterSceneryLogic` | `SpriteLogic` | 4.4 | `underwater_stage` | 7 493 |
| `UnderwaterTempleLogic` | `SpriteLogic` | 4.4 | `underwater_stage` | 20 201 |
| `UnderwaterTempleSymbolLogic` | `SpriteLogic` | 4.4 | `underwater_stage` | 3 282 |
| `VolcanoLogic` | `SpriteLogic` | 4.2 | `island_objects` | 7 609 |
| `WaterWingsLogic` | `SpriteLogic` | 4.5 | `nopee` | 31 883 |
| `WhirlPoolLogic` | `SpriteLogic` | 4.1 | `sky_powers` | 14 464 |
| `WhirlPoolModel` | `BCDisplayObject` | 4.1 | `sky_powers` | 4 970 |
| `WoodLogic` | `SpriteLogic` | 4.2 | `island_core` | 33 413 |
| `XenuLogic` | `SpriteLogic` | 4.15 | `apocalypse_locust_closet_xenu` | 9 285 |
| `XenuTempleLogic` | `SpriteLogic` | 4.15 | `apocalypse_locust_closet_xenu` | 3 493 |
| `ZombieGraveLogic` | `SpriteLogic` | 4.8 | `monsters` | 10 746 |
