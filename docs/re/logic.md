# Logic framework and the generated logic-object table

Spec for the `BCLogic` family of the BC engine (Pocket God 1.48.2, arm64) and for the
632 global logic objects built by the generated `BCLibrary(Logic)` category. Every
statement cites the function it comes from as `Class::selector @ address`. Addresses are
virtual addresses in `tmp/re/PocketGod.arm64`. Where the decompiler could mislead, the
disassembly was read instead and that is noted.

Related deliverables:

- `assets/data/logic-objects.json` — the logic-object table (schema in section 13.4).
- `tools/re/extract_logic_objects.py` — produces it (section 13.10 explains how).

Out of scope here and documented by other specs: the display tree and rendering
(`BCDisplayObject`, `BCDisplayGroup`, `BCMultiModel`, projection — `docs/re/rendering.md`),
the animation engine (`BCAnimation`, `BCBinaryAnimationDef`, `Timelines.bin` —
`docs/re/timelines.md`), the full sequence engine and the `BCLibrary` default item
handlers (`assets/data/sequence-methods.json` and its spec), the view controller's world
gestures, and the behaviour of each gameplay `*Logic` class. This document describes
those systems only as far as a logic object sees them.

---

## 1. Class map

| Class | Superclass | Role | Decompiled file |
|---|---|---|---|
| `BCLogic` | `NSObject` | Base of every logic object: binding to a display object, animation, behaviour sequence, hibernation, game-frame registration, touch hooks, physics state. 59 instance methods + 1 class method. | `BCLogic.c` |
| `SpriteLogic` | `BCLogic` | Adds gravity and Euler integration, hurricane/whirlpool hooks, apocalypse disease cloud. 135 classes descend from it. | `SpriteLogic.c` |
| `MultiModelLogic` | `BCLogic` | Animation-only object with a listener and `die`/`multiModelEvent`/`triggerEpisodeEvent` items. | `MultiModelLogic.c` |
| `BCScreenLogic` | `MultiModelLogic` | Modal screen with 10 layers and button layers. Subclasses: `IdolScreenLogic`, `MoronPestsQuestionScreenLogic`, `DanceDanceExecutionScreenLogic`. | `BCScreenLogic.c` |
| `BCButtonLogic` | `BCLogic` | Button: press / release / release-outside callbacks. | `BCButtonLogic.c` |
| `MessageLogic` | `BCLogic` | Timed message overlay shown in the `PocketGod` group. | `MessageLogic.c` |
| `OneShotAnimLogic` | `SpriteLogic` | Self-owning fire-and-forget effect running one sequence. | `OneShotAnimLogic.c` |
| `IslandObjectLogic` | `SpriteLogic` | Object inserted before a display marker when an island is entered. | `IslandObjectLogic.c` |
| `TestLogic` | `BCLogic` | Empty; never instantiated (no reference outside its own file). | `TestLogic.c` |
| `BCTouch` | `NSObject` | One finger: phase, timestamps, tap count, view position, owning object. | `BCTouch.c` |
| `BCLogItem` | `NSObject` | Debug log record (appendix B). | `BCLogItem.c` |

149 classes derive from `BCLogic` (including it). Direct children of `BCLogic`:
`SpriteLogic`, `MultiModelLogic`, `BCButtonLogic`, `MessageLogic`, `TestLogic`,
`IdolSelectionIconLogic`, `IdolIconLogic`, `IdolBattleRealmEffectLogic`, `MapPaperLogic`.

Framework-level overrides found in subclasses (from `tmp/re/classes.json`):
`shutdown` is overridden only by `PiranhaLogic`; `setBehavior:` only by `OutHouseLogic`;
`isSoundDisabled` and `applyMoveXPos:moveYPos:` only by `PygmyLogic`; `id` only by
`MapPaperLogic`; `gameFrameOrder` by the classes listed in section 4.2. `hibernate`,
`unhibernate`, `startGameFrame`, `stopGameFrame`, `parentTweenable`, `doesPointIntersect:`,
`onAnimationMoveEnabled` and `setBinaryFrameDef:tweenableChannelArray:` are never overridden.

---

## 2. Object layouts

Offsets are the values stored in the `__objc_ivar` offset variables (the labels
`Class::mIvar` in `tmp/re/symbols.tsv`). `vec4` is `{float x, y, z, w}` (16 bytes).
All ivars not set by an initializer are zero (Objective-C `alloc`).

### 2.1 BCLogic (instance size 132)

| Off | Ivar | Type | Set by `BCLogic::initWithDisplayObject:` @ 0x10013841c |
|---|---|---|---|
| 8 | `mbShutdown` | BOOL | (0) |
| 16 | `mDisplayObject` | `BCDisplayObject*` (retained) | the argument |
| 24 | `mbGameFrameRunning` | BOOL | 0 |
| 32 | `mAnimation` | `BCAnimation*` (owned) | new `BCAnimation`, listener = self |
| 40 | `mSequence` | `BCSequence*` (owned) | nil |
| 48 | `mbHibernate` | BOOL | see 3.1 (may become 1 during `setLogic:`) |
| 49 | `mbHibernateGameFrameRunning` | BOOL | see 3.1 |
| 52 | `mPosVel` | vec4 | (0, 0, 0, 1) |
| 68 | `mPosForce` | vec4 | (0, 0, 0, 1) |
| 84 | `mRotVel` | vec4 | (0, 0, 0, 1) |
| 100 | `mRotForce` | vec4 | (0, 0, 0, 1) |
| 116 | `mStallTime` | float | 0 |
| 120 | `mDeltaValue` | float | 0 |
| 124 | `mLastMoveXPos` | float | (0) |
| 128 | `mLastMoveYPos` | float | (0) |

`mStallTime`/`mDeltaValue` are scratch state shared by the `stall`, `tweenAlpha` and
`countDownTimer` items (only one item of a logic's sequence runs at a time).
`mRotForce` is never used outside this initializer; `mRotVel` is used by 10 gameplay
classes (`AnchorLogic`, `BoulderLogic`, `ChumLogic`, `FishingPoleLogic`, `FailBoatLogic`,
`MoronSnakeLogic`, `PygmyLogic(NoPee)`, `SpearLogic`, `WaterWingsLogic`, `WoodLogic`) and
never by the framework.

### 2.2 SpriteLogic (adds, up to 276)

| Off | Ivar | Type | Set by `SpriteLogic::initWithDisplayObject:` @ 0x100199178 |
|---|---|---|---|
| 132 | `mGravityForce` | vec4 | (0, -3000.0 (`0xc53b8000`), 0, 1) |
| 148 | `mbIgnoreGravity` | BOOL | **1** |
| 149 | `mbBlownByHurricane` | BOOL | (0) |
| 150 | `mbPickedUpByHurricane` | BOOL | (0) |
| 152 | `mPickedUpByHurricaneLogic` | `HurricaneLogic*` | (nil) |
| 160 | `mHurricaneSpinState` | `HurricaneSpinState` (44 B): `int mStall; float mRadius, mRadiusRot, mRadiusRotVel, mYPos, mYPosRot, mYPosRotVel, mAngle, mAngleVel, mStrength, mStrengthVel` | (0) |
| 204 | `mbBlownByWhirlPool` | BOOL | (0) |
| 205 | `mbPickedUpByWhirlPool` | BOOL | (0) |
| 208 | `mPickedUpByWhirlPoolLogic` | `WhirlPoolLogic*` | (nil) |
| 216 | `mWhirlPoolSpinState` | `WhirlPoolSpinState` (36 B): `int mStall; float mRadius, mTargetRadius, mRadiusRot, mRadiusRotVel, mAngle, mAngleVel, mStrength, mStrengthVel` | (0) |
| 256 | `mApocalypseCloudStyleId` | `NSString*` | `@"Default"` (literal 0x100386830) |
| 264 | `mApocalypseCloudLogic` | `ApocalypseCloudLogic*` (retained) | (nil) |
| 272 | `mApocalypseDiseaseType` | int | (0) |

### 2.3 Other framework classes

| Class | Off | Ivar | Type / initial value |
|---|---|---|---|
| `MultiModelLogic` | 136 | `mMultiModelLogicListener` | `id<MultiModelLogicListener>`, retained, nil |
| `BCScreenLogic` | 144 | `mScreenDisplayGroup` | `BCDisplayGroup*` = `getDisplayGroupById:@"ScreenDisplayGroup"` (0x100389b50), not retained |
| | 152 | `mActiveDisplayGroup` | `BCDisplayGroup*`, set by `openScreen` |
| | 160 | `mScreenLayerArray` | `ScreenLayer[10]`, 16 B each: `{BCLogic* mLogic; BCDisplayGroup* mButtonDisplayGroup}`; memset 0 (0xa0 bytes) |
| `BCButtonLogic` | 136 | `mButtonLogicListener` | `id<BCButtonLogicListener>`, retained, nil |
| | 144 | `mbTouchBeganInside` | BOOL, 0 |
| `MessageLogic` | 136 | `mMessageLogicListener` | `id<MessageLogicListener>`, retained |
| | 144 | `mbActive` | BOOL |
| `IslandObjectLogic` | 280 | `mDisplayMarkerId` | `NSString*`, assigned without retain |
| `BCTouch` | 8 | `mUITouch` | `UITouch*`, not retained |
| | 16 | `mTouchPhase` | int64 (`UITouchPhase`: 0 began, 1 moved, 2 stationary, 3 ended, 4 cancelled) |
| | 24 | `mTimeStamp` | float (`UITouch.timestamp`, seconds since boot, narrowed to f32) |
| | 28 | `mGameTime` | float (`BCView.mTime` when the event was processed) |
| | 32 | `mTapCount` | int |
| | 36 | `mPos` | vec4; x, y = `locationInView:` in view points, z = 0, w = 1 |
| | 56 | `mActiveObject` | `NSObject*`, retained (the logic currently holding the touch) |
| `BCLogItem` | 8 / 16 | `mLogTag` / `mTrace` | `NSString*`, retained |

`BCTouch::init` @ 0x100177de0 zeroes everything and sets `mPos = (0,0,0,1)`. Its
accessors (0x100177edc–0x100177ff8) are plain getters/setters; `setXPos:yPos:` writes x
and y only; `setActiveObject:` retains the new object and releases the old one.

---

## 3. Lifecycle

### 3.1 Construction: `BCLogic::initWithDisplayObject:` @ 0x10013841c

In this exact order:

1. `[super init]` (NSObject). If nil, return nil.
2. `DAT_1007d1f14 += 1` (global live-logic counter, only used by `+traceGlobalStatus` @ 0x1001383d0).
3. `mDisplayObject = displayObject`.
4. `[displayObject setLogic:self]` — `BCDisplayObject::setLogic:` @ 0x10010e520 stores the
   pointer (not retained) and immediately sends `[self hibernate]` if the display object
   is hibernating, otherwise `[self unhibernate]`. Because `mAnimation`/`mSequence` are
   still nil, `hibernate` here only sets `mbHibernate = 1` and
   `mbHibernateGameFrameRunning = mbGameFrameRunning (= 0)`; `unhibernate` is a no-op.
   Both this and step 5 are `objc_msgSend`s (checked in the decompiled code), so a nil
   display object is accepted and skips them (section 3.7).
5. `[displayObject retain]`.
6. `mPosVel = mPosForce = mRotVel = mRotForce = (0, 0, 0, 1)`; `mbGameFrameRunning = NO`.
7. `mAnimation = [[BCAnimation alloc] init]`, then
   `BCAnimation::setAnimationListener:(self)` (retains self, see 3.2).
   If `mbHibernate`, `[mAnimation hibernate]`.
8. `mSequence = nil; mStallTime = 0; mDeltaValue = 0`.

`BCLogic` itself never starts its game frame. Subclass initializers then run:

| Initializer | After `[super initWithDisplayObject:]` |
|---|---|
| `SpriteLogic::initWithDisplayObject:` @ 0x100199178 | `mbIgnoreGravity = YES`; `mApocalypseCloudStyleId = @"Default"`; `mGravityForce = (0,-3000,0,1)`; **`[self startGameFrame]`**. |
| `MultiModelLogic::initWithDisplayObject:` @ 0x10000bb14 | `[self stopGameFrame]` (no effect: never started). |
| `BCScreenLogic::initWithDisplayObject:` @ 0x10001cbb0 | `mScreenDisplayGroup = [BCLibrary getDisplayGroupById:@"ScreenDisplayGroup"]`; zero the 10 layers. |
| `BCButtonLogic::initWithDisplayObject:` @ 0x100091c20 | `mbTouchBeganInside = NO; mButtonLogicListener = nil`. |
| `MessageLogic::initWithDisplayObject:` @ 0x10019ee38 | `[self stopGameFrame]`; `[mDisplayObject removeFromDisplayGroup]` (hidden until `showMessage:`). |
| `IslandObjectLogic::initWithDisplayObject:displayMarkerId:` @ 0x100125510 | (super = `SpriteLogic`, so the game frame was requested) `[self stopGameFrame]`; `mDisplayMarkerId = markerId`; `[mDisplayObject removeFromDisplayGroup]`. |
| `OneShotAnimLogic::initWithSequenceId:` @ 0x10003794c | Creates `[[BCMultiModel alloc] initWithId:@"OneShot"]` (0x10038bd30), calls `SpriteLogic` init with it, releases the model, then `[self stopGameFrame]`, `[self setBehavior:sequenceId]`, `[self retain]` (self-ownership, released by `oneShotComplete`). |
| `TestLogic::initWithDisplayObject:` @ 0x100113c60 | nothing. |

A new `BCMultiModel` (`BCMultiModel::initWithId:` @ 0x100181694 → `BCModel initWithId:textureDef:nil`)
is only a named container: it has no texture, and everything it shows comes from the
binary animation frames the logic plays (section 6). A freshly created display object
is hibernating (`BCDisplayObject::initWithId:` @ 0x10010dfc0 sets `mbHibernate = 1`,
`mHibernateRefCount = 0`, `mbIsSelectable = 0`), so a logic bound to a new model, or to an
object of a group that is not on screen, starts hibernated: its animation, behaviour and
game frame are all deferred until the display object becomes part of the active tree.
No display group is awake while the table is built (the first one is activated by the
boot script's `drawDisplayGroup` item, `BCLibrary::drawDisplayGroup:::` @ 0x1000244c8, during
the first warm-up frame of section 4.1), so every table entry starts hibernated except entry 624,
whose display object is nil (section 3.7).

### 3.2 Ownership and references

| From | To | Kind | Where |
|---|---|---|---|
| logic | display object | strong (retain in init, release in dealloc) | `BCLogic::initWithDisplayObject:` / `BCLogic::dealloc` @ 0x100138a74 |
| display object | logic | weak (`mLogic`, assign) | `BCDisplayObject::setLogic:` @ 0x10010e520 |
| logic | animation | strong | init / dealloc |
| animation | logic (`mAnimationListener`) | strong (retain) → cycle | `BCAnimation::setAnimationListener:` @ 0x10019fd40; broken by `shutdown` |
| logic | sequence | strong (alloc + an extra retain; dealloc releases once, leaking one count — irrelevant to a port) | `BCLogic::setBehavior:` @ 0x1001385f8 |
| sequence | logic (`mDelegate`) | weak (assign) | `BCSequence::initWithSequenceDef:delegate:` @ 0x1000bff20 |
| logic | listener | strong (retain new, release old) | every `set*Listener:` (e.g. `BCButtonLogic::setButtonLogicListener:` @ 0x100091c78) |
| `BCView` | game-frame listeners | strong while registered (`NSMutableArray`) | `BCView::addGameFrameListener:` @ 0x100177528 |
| `BCTouch` | active object | strong | `BCTouch::setActiveObject:` @ 0x100177ff8 |

The 632 table objects are never released (they live for the process lifetime). Section 3.6
lists every other creation path and the owner of what it creates; two framework idioms
destroy themselves (section 3.5).

### 3.3 Hibernation

A logic is "hibernating" when its display object is not reachable from an awake display
group. The display side drives it (summary of the display-object rules, cited because
they decide when logic runs):

- `BCDisplayObject::hibernate` @ 0x10010e454: `mHibernateRefCount -= 1`; when it reaches 0,
  `mbHibernate = 1` and `[mLogic hibernate]`.
- `BCDisplayObject::unhibernate` @ 0x10010e49c: if the count is 0, `mbHibernate = 0` and
  `[mLogic unhibernate]`; then `mHibernateRefCount += 1`.
- `BCDisplayGroup::hibernate`/`unhibernate` @ 0x10006738c / 0x10006744c do the same and
  forward to every child.
- `BCDisplayGroup::addDisplayObject:` @ 0x100067544 (and `…atIndex:` @ 0x1000675d0) send
  `unhibernate` to the added object if the group is awake; `removeDisplayObject:` @ 0x1000676ac
  sends `hibernate` if the group is awake. `moveToDisplayGroup:`, `insertBefore:`,
  `insertAfter:` are remove + add.
- `BCView::setActiveDisplayGroup:` @ 0x100177570 hibernates the previous root and
  unhibernates the new root (only if they differ).
- `BCAnimation::setTweenable:channelId:` @ 0x10019fdac adds a wake reference to the
  tweenable when the animation is awake; `BCAnimation::hibernate`/`unhibernate`
  @ 0x10019ffcc / 0x1001a0090 forward to all 30 tweenable channels.

The logic side:

- `BCLogic::hibernate` @ 0x100138874: if not already hibernating: `mbHibernate = 1`;
  `mbHibernateGameFrameRunning = mbGameFrameRunning`; if the game frame was running,
  `mbGameFrameRunning = 0` and `[BCView removeGameFrameListener:self]`; then
  `[mAnimation hibernate]`, `[mSequence hibernate]`.
- `BCLogic::unhibernate` @ 0x100138940: if hibernating: `mbHibernate = 0`; if
  `mbHibernateGameFrameRunning`, `[self startGameFrame]`; then `[mAnimation unhibernate]`,
  `[mSequence unhibernate]`. The flag is not cleared.
- `BCLogic::isHibernating` @ 0x100138864 returns `mbHibernate`.
- `BCSequence::hibernate`/`unhibernate` @ 0x1000c0028 / 0x1000c00b4 and the `BCAnimation`
  equivalents follow the same pattern with their own `mbIsRunning`.

### 3.4 Game-frame registration

- `BCLogic::startGameFrame` @ 0x100138754: if hibernating, only
  `mbHibernateGameFrameRunning = 1` (deferred start). Else, if not running:
  `mbGameFrameRunning = 1` and `[BCView addGameFrameListener:self]` (appends).
- `BCLogic::stopGameFrame` @ 0x1001387e0: if hibernating, only
  `mbHibernateGameFrameRunning = 0`. Else, if running: `mbGameFrameRunning = 0` and
  `[BCView removeGameFrameListener:self]` (`removeObject:`).

Quirk to reproduce: `BCLogic::setBehavior:` sets `mbHibernateGameFrameRunning = 1` when it
creates the first sequence while the logic is hibernating (disassembly at 0x1001386f8:
`strb #1` into `BCLogic::mbHibernateGameFrameRunning` of self). The logic's own game frame
is therefore started on the next unhibernate even for classes that stopped it. Example:
`OneShotAnimLogic` stops its game frame and then calls `setBehavior:` while its new model
is hibernating, so once the model is put into an awake group it runs
`SpriteLogic::onGameFrame:` (pos += vel·dt) as well as its sequence.

### 3.5 Shutdown and destruction

- `BCLogic::shutdown` @ 0x1001389e8 (all calls via `objc_msgSend`, checked in the
  disassembly): `mbShutdown = 1`; `[self stopGameFrame]`; `[mSequence stop]`;
  `[mAnimation stop]`; `[mAnimation setAnimationListener:nil]` (breaks the retain cycle;
  the animation no longer calls back into the logic). It does not detach the display
  object, the listener or the sequence delegate.
- `mbShutdown` is the "dead" flag. `SpriteLogic::onGameFrame:` skips its integration when
  it is set, and 83 gameplay classes read it (typically as the first test of their own
  `onGameFrame:`). It is never cleared.
- `MultiModelLogic::die:::` @ 0x10000bc38 (sequence item): `[mDisplayObject removeFromDisplayGroup]`,
  `[self shutdown]`, `[self release]`, returns 2. Used by dynamically created
  `MultiModelLogic`s whose sequence ends in `die`.
- `OneShotAnimLogic::oneShotComplete:::` @ 0x100037a74: identical (remove, shutdown, release, 2).
- `BCLogic::dealloc` @ 0x100138a74: `DAT_1007d1f14 -= 1`, `[self stopGameFrame]`,
  `[mSequence stop]`, release sequence, `[mAnimation stop]`, release animation, release
  display object. Subclass deallocs release their listener / retained ivars first
  (`SpriteLogic::dealloc` @ 0x100199224 releases `mApocalypseCloudStyleId`).
- The only override of `shutdown` is in `PiranhaLogic`.

### 3.6 Creation paths and owners

Every logic object is made with `[[Class alloc] initWithDisplayObject:obj]` (or one of the
two special initializers of 3.1). There is no dynamic class lookup (`NSClassFromString` is
never used for logic classes) and no registry of logic objects. Every allocation of a
`BCLogic` subclass in the decompiled game code belongs to one of these paths:

| Path | Code | Creates | Owner (holds the reference) |
|---|---|---|---|
| Generated table | `BCLibrary(Logic)::createLogicClasses` @ 0x10013feb8, once in `loadGame` (section 13) | 632 objects of 56 classes | the table slots `0x1007e3030 + 8·i`; never released |
| `setLogic` sequence item (logic factory) | `PocketGodViewController(Game)::setLogic:::` @ 0x10015c11c; 275 items, almost all in the boot script `PocketGod` (`docs/re/sequences.md` §3.14 maps ids to classes) | for the object `params[2]` of group `params[0]` (looked up with `getDisplayGroupById:`/`getDisplayObject:`), a logic chosen by `params[1]`. 53 classes: 78 literal `alloc` sites over 50 classes (24 of the sites are `BCButtonLogic`), plus `SpriteLogic`, `MultiModelLogic`, `TheRunsPlatformLogic`, `TheRunsStalactiteLogic` and `KonkeyDongLogic` allocated through a class variable. An unknown id creates nothing (returns 2) | the view controller: the ivar and/or UI array of that id, plus `mLogicArray` (coconuts go to `mCoconutLogicArray` only); the factory releases its own reference; listener = the view controller |
| Manager factories | the same item with `params[3]` naming a manager is forwarded to `ChallengeManager::setLogic:` @ 0x10001a238, `SkinPackManager::setLogic:` @ 0x1000611d4, `BonusPackManager::setLogic:` @ 0x1000cc1c0, `StoryManager::setLogic:` @ 0x100050080, `NewsManager::setLogic:` @ 0x100056ba8 or `ChumBucketLogic::setLogic:` @ 0x10011b7dc. Their `alloc` sites: 13 `BCButtonLogic` + 5 `MessageLogic` (the game's only `MessageLogic`s); 27 `BCButtonLogic` + 1 `MultiModelLogic`; 16 + 2; 78 + 4; 8 `BCButtonLogic`; and `BCButtonLogic`, `LaserSharkLogic`, `LaserTargetLogic` | same pattern; `count` items create `count` objects named `[NSString stringWithFormat:params[2], i]`, i = 0… | the manager's ivars/UI arrays and its own `mLogicArray`; listener = the manager |
| Direct creation in `loadGame` | `PocketGodViewController::loadGame` @ 0x10006a030 | `RainbowLogic` on a `RainbowModel`, `LocustLogic` on a `LocustModel` (both `z = -277.12`) | view-controller ivars `mRainbowLogic`, `mLocustLogic` |
| Runtime spawns | 45 other functions (60 sites, 34 classes), e.g. `PocketGodViewController(Game)::createPygmyLogic` @ 0x10015ba88, `createClouds` @ 0x10015e774, `triggerLightning` @ 0x100160f74, `startHurricane:` @ 0x100162384, `fireworks:::` @ 0x10015a7c8, `checkUnderwaterBubble:pos:` @ 0x100163ac4, `VolcanoLogic::lavaRain:::` @ 0x1000c7130, `AntHillLogic::directAnts:::` @ 0x10013e3b8, `PygmyLogic::explode:::` @ 0x10012fcc0, and the initializers of composite logics (`KongStatueLogic`, `ApeAltarLogic`, `CrackHoleLogic`, `LaserGunLogic`, `IceHoleLogic`, `MonkeyLogic`, `LaserSharkLogic`) | pygmies, clouds, effects, projectiles, sub-objects | the creating object's ivar or array; `OneShotAnimLogic` and `die`-terminated `MultiModelLogic`s own themselves (3.5) |

Creating a logic registers it nowhere. It reaches the frame loop only through
`startGameFrame` (3.4) and its sequence's and animation's `play`, all deferred while it
hibernates. It reaches touch dispatch (9.2), `BCLibrary::setBehavior:::` (7.1) and every
other display-addressed item only through its display object's `mLogic` back-pointer: the
display tree is the de-facto registry.

### 3.7 Logic objects with a nil display object

`BCDisplayGroup::getDisplayObject:` @ 0x10006781c returns nil when the group has no direct
child with that id, and neither `createLogicClasses` nor the factories check the result.
The original then behaves as follows (every message to nil is a no-op returning 0/nil):

- `BCLogic::initWithDisplayObject:nil` skips `setLogic:` and `retain`, so `mbHibernate`
  stays 0: the logic is awake from the start, unlike every logic with a display object.
  Its `BCAnimation` is not hibernated either (`BCAnimation::init` @ 0x10019fc8c leaves its
  `mbHibernate` at 0). Subclass initializers run normally.
- It is in no display tree, so it never receives touches (9.2), is never hibernated or
  unhibernated, and no display-addressed item (`setBehavior`, `setLogic`, …) can reach it.
- Its listener is set as usual and never hears from it.
- `id`, `parentTweenable` and `displayObject` return nil, and `setBinaryFrameDef:…` is not
  forwarded (`isKindOfClass:` sent to nil is NO). A store through `[[logic displayObject] pos]`
  would dereference NULL and crash.
- If something started its behaviour or animation, it would run at once (nothing is
  hibernated) with every display effect a no-op.

Port: model the binding as `Option<Entity>`, create the logic with `None` and leave it awake
(not hibernated), keep it in its slot or array so indices line up, and never put it in touch
dispatch.

Instances in 1.48.2:

- **Table entry 624** (`BCButtonLogic`, binding `CommunityMenuDisplayGroup`/`OpenFeintButton`,
  generated from `BCLibrary+Logic.c` line 4377, listener `PocketGodViewController` via
  `setButtonLogicListener:`). `CommunityMenuDisplayGroup` has only `MenuCap`, `MenuBG`,
  `NewsButton`, `ChallengeButton` and `StartScreenButton`, and nothing removes or adds
  children before the lookup (13.8). No struct-XML node of 1.48.2 is named `OpenFeintButton`:
  `@"OpenFeintButton"` @ 0x100405c10 has no pointer in `__DATA`, and its only code reference is
  this lookup. v1.45.3 has neither the group nor the object; the entry is a leftover of the
  removed OpenFeint integration. No code calls `getLogic:` with 624 and no tweenable targets
  it, so the object is inert. The JSON marks it with `binding.resolved = false`.
- **Factory-created buttons** (a static check of the `setLogic` items against
  `display-groups.json` that does not replay the boot script's own tree edits; these
  belong to the manager specs): `StoryManager`'s `StoryWideOptionMenuOptionButtonLogic` item
  (`count` 15) asks for `BlackBox0`…`BlackBox14` but `StoryWideOptionMenu` has only
  `BlackBox0`…`BlackBox9`, so positions 10–14 of `mStoryWideOptionButtonLogicArray` are
  nil-display buttons. Also `ChallengeManager`'s `ChallengeResultsRechallengeButtonLogic`
  (`ChallengeResultsScreen`/`RechallengeButton`, which exists only in `ChallengeHistoryPanel`)
  and `SkinPackManager`'s `SkinPackCustomizePrevPackButtonLogic`/`…NextPackButtonLogic`
  (`CustomPackPanel`/`PrevPack`, `NextPack`, which exist nowhere).

---

## 4. The frame as seen by a logic object

### 4.1 `BCView::gameLoop` @ 0x100177730

Who calls it:

- **Three warm-up frames.** At the end of `PocketGodViewController::loadGame` @ 0x10006a030,
  after the boot sequence `PocketGod` and the view controller's other sequences exist,
  `[mView gameLoop]` is called three times directly, each inside its own autorelease pool
  (`docs/re/controller.md` §2.3, steps 13–14). These are the first frames in which any
  logic, sequence or animation runs; the boot script starts in the first one.
- **The display link.** `loadGame` then sends `[mView setDisplayLinkCount:n]` with n = 1 if
  `(float)(t_end − t_start) <= 13.0` and n = 2 otherwise, where t_start/t_end are
  `CFAbsoluteTimeGetCurrent()` at the start of `loadGame` and after the warm-ups. It then
  schedules `startRenderLoop` with an `NSTimer` of 0.1 s (`0x3fb99999a0000000` =
  `(double)0.1f`). `BCView::startRenderLoop` @ 0x100177af8 creates, the first time only, a
  `CADisplayLink` that calls `gameLoop` with `frameInterval = mDisplayLinkCount`. 1 means every
  screen refresh; 2 means every second one (30 Hz on a 60 Hz screen), a fallback for devices
  that took more than 13 s to load. `BCView::initWithFrame:` @ 0x100175840 initialises
  `mDisplayLinkCount = 1`.
- The app delegate stops and restarts the loop (`stopRenderLoop` on resign-active, background,
  a modal view or `openLaunchUrl`; `startRenderLoop` on become-active, enter-foreground and
  modal dismissal; `docs/re/rendering.md` §2.1). Every restart resets the time base (below).

Per frame:

1. `mGameFrameCount += 1`; bind the framebuffer and clear.
2. Time (disassembly 0x1001777b8–0x1001777fc): `now = (float)(CFAbsoluteTimeGetCurrent() −
   mStartTime)`. `mStartTime` is a double; the subtraction is an `fsub d` and the result is
   narrowed with `fcvt`. Then `x = now − mLastTime` (f32) and
   `dt = (x > 0.06666667f) ? 0.06666667f : x` (`fcmp` + `fcsel gt`): an upper clamp of 1/15 s,
   no lower clamp, and a NaN would pass through. Finally `mLastTime = mTime = now`.
3. Run queued texture-atlas loads.
4. `readAccelerometer`.
5. `processTouchEvents:(now)` — all queued touch events are dispatched here, before any
   `onGameFrame:` (section 9.1).
6. Sort `mGameFrameListenerArray` with `sortUsingFunction:FUN_1001757bc` (ascending
   `gameFrameOrder`; the comparator returns -1/0/1).
7. Snapshot the array into a 200-entry stack buffer (`getObjects:range:`), retain all,
   call `onGameFrame:dt` on each in order, release all.
8. Draw the active display group (plus wide/tall overlays), present.

Time base and the first dts:

| Moment | State | dt |
|---|---|---|
| `BCView::initWithFrame:` | `mStartTime = 0.0` (double), `mLastTime = 0`, `mTime = 0` | — |
| warm-up 1 | `now = (float)T`, with T = `CFAbsoluteTimeGetCurrent()` in seconds since 2001-01-01 (≈ 4.4e8–5.0e8 in 2015–2016, ≈ 8.1e8 in 2026) | x = now, far above 1/15 → **1/15** |
| warm-ups 2 and 3 | `now = (float)T'`. At this magnitude consecutive f32 values are 32 s apart (T < 2²⁹ s, before 2018-01-05) or 64 s apart (T < 2³⁰ s, until 2035-01-10) | x = 0 → **0**, unless a 32 s/64 s rounding boundary falls between two calls (x = 32 or 64 → **1/15**) |
| `startRenderLoop` | `mStartTime = CFAbsoluteTimeGetCurrent()` (double), `mLastTime = −0.06666667` (`0xbd888889`) | — |
| first display-link frame | `now` = seconds since `startRenderLoop` (≥ 0) | x = now + 0.0667 ≥ 1/15 → **1/15** |
| later frames | | measured frame time, clamped |

So the warm-ups do not get small real dts: they get 1/15, then 0 and 0 in the usual case.
`docs/re/rendering.md` §2.1 calls the second and third "tiny real dts"; the f32 rounding above
makes them 0 (or 1/15). During the warm-ups `mTime` holds that huge value. `BCView::time`
@ 0x100175ee0 has no caller in the game code, and no touch can be queued while `loadGame`
runs synchronously, so the value does not leak into `BCTouch.mGameTime`. Every later
`startRenderLoop` (it returns at once if the loop is already active) resets `mStartTime` and
`mLastTime` the same way. The first frame after a resume or a closed modal view therefore
gets dt = 1/15, and `now`, which becomes `BCTouch.mGameTime`, restarts near 0. Code that
compares game times across such a restart sees time jump backwards.

Port: run the three warm-up updates synchronously at the end of loading with dt = 1/15, 0, 0,
then the per-frame loop (the original's rate is the display refresh, frame interval 1),
starting with dt = 1/15 after every (re)start. Much item code is frame-based (for example
`rotate` adds 2° per call, and items returning 2 chain within one frame), so the update rate
matters, not only dt.

Consequences a port must keep:

- **Snapshot semantics**: listeners registered during the `onGameFrame:` calls first run
  next frame; listeners removed during them (stopped, hibernated, shut down) still receive
  this frame's `onGameFrame:`. That is why gameplay code checks `mbShutdown` and why
  `BCSequence`/`BCAnimation` re-check `mbIsRunning` inside `onGameFrame:`. Touch callbacks
  (step 5) run before the snapshot, so whatever they register (a behaviour started with
  `setBehavior:`, `startGameFrame`, an animation) already runs in the same frame.
- **Order**: ascending `gameFrameOrder`. Within one value the order is the array order,
  i.e. the order in which objects (last) registered. The sort is assumed stable
  (Apple's `CFSortIndexes` merge sort; see open questions).
- The same `dt` is passed to every listener and stored by each `BCSequence` as `deltaTime`.
- The original overflows its stack buffer above 200 simultaneous listeners; the port needs no limit.

### 4.2 `gameFrameOrder` values

| Value | Implementations |
|---|---|
| 10 | `BCSequence::gameFrameOrder` @ 0x1000c0518 |
| 20 | `BCAnimation::gameFrameOrder` @ 0x1001a0498 |
| 30 | `PygmyLogic` @ 0x100129978 |
| 40 | `BCLogic` @ 0x1001389dc (default for all logic), `BCPreferences` @ 0x10016e7a4, `GameUiManager` @ 0x1001218f8 |
| 50 | `PocketGodViewController(Game)` @ 0x1001585b4, `StoryManager` @ 0x100049470 |
| 59 | `NoPeeLogic` @ 0x10019da54 |
| 60 | `MoronBeeLogic` @ 0x1001750c0, `MoronDuckLogic` @ 0x1001a8030, `NoPeeMapLogic` @ 0x100182fd0, `NoPeeMapItemLogic` @ 0x10017f7ac, `NoPeeMapScreenLogic` @ 0x10001add8 |
| 70 | `GameServer` @ 0x10008ba48 |

So in each frame all running behaviours advance first, then all animations, then pygmies,
then the other logic objects, then the view controller.

---

## 5. Per-frame update

`BCLogic::onGameFrame:` @ 0x1001389e4 is empty.

`SpriteLogic::onGameFrame:` @ 0x100199420 (checked instruction by instruction; plain
`fmul` + `fadd`, no fused multiply-add):

```
if mbShutdown: return
p = displayObject.pos                     // pointer to the display object's vec4
mPosForce = (0, 0, 0, 1)
if !mbIgnoreGravity:
    mPosForce.xyz = mGravityForce.xyz + mPosForce.xyz   // (0, -3000, 0) by default
p.x += mPosVel.x * dt;  p.y += mPosVel.y * dt;  p.z += mPosVel.z * dt
mPosVel.x += mPosForce.x * dt;  mPosVel.y += mPosForce.y * dt;  mPosVel.z += mPosForce.z * dt
```

- Explicit Euler: position uses the old velocity, then velocity is updated. All f32.
- `mPosForce` is rebuilt from scratch every frame, so a subclass that wants extra force
  must add it after calling this method or integrate itself.
- No drag, no rotation integration, no floor or bounds in the base class. Gravity is off
  by default (`mbIgnoreGravity = YES`); gameplay code clears the flag (e.g. drag/fall items).
- Only 15 classes inherit this method unchanged (`LavaLogic`, `GlacierLogic`,
  `BattleAlertLogic`, `OneShotAnimLogic`, `VolcanoLogic`, `LightningLogic`,
  `DanceBalloonBundleLogic`, `IcicleLogic`, `DiscoBallLogic`, `IslandObjectLogic`,
  `LaserGunLogic`, `UnderwaterTempleSymbolLogic`, `EpisodeAlertLogic`, `DiscoLightsLogic`,
  `AdBlockerLogic`); 6 call it as `super` (`CloudLogic`, `HurricaneLogic`, `OceanLogic`,
  `WhirlPoolLogic`, `UnderwaterSceneryLogic`, `StormCloudLogic`); the remaining
  `SpriteLogic` descendants replace it with their own integration (drag, floors, springs).
- Non-sprite classes with their own `onGameFrame:`: `DanceDanceExecutionScreenLogic`,
  `MoronPestsQuestionScreenLogic`, `IdolSelectionIconLogic`, `IdolBattleRealmEffectLogic`,
  `IdolScreenLogic`, `IdolIconLogic` (they must start their game frame themselves).

Velocity accessors (`BCLogic`): `posVel` @ 0x1001398f4 (pointer to `mPosVel`),
`setPosVel:` @ 0x100139904 (copies all 16 bytes), `xPosVel`/`setXPosVel:`,
`yPosVel`/`setYPosVel:`, `zPosVel`/`setZPosVel:` @ 0x100139924–0x100139980.

`SpriteLogic::isAccelEnabled` @ 0x100199358 (used by tilt-driven subclasses): returns YES
iff NOT (`rawAccelRotDegrees.x` ∈ (−100, −80) and `.y` ∈ (−100, −80)) and all three
components of `rawAccelForce` are non-zero; otherwise NO.

---

## 6. Animation hooks (`BCAnimationListener`)

Every logic owns one `BCAnimation` whose listener is the logic. The protocol
`BCAnimationListener` requires `id`, `onAnimationMoveEnabled`, `isSoundDisabled`,
`applyMoveXPos:moveYPos:`, `parentTweenable`, `setBinaryFrameDef:tweenableChannelArray:`.

| Method | Behaviour |
|---|---|
| `BCLogic::playAnimationId:` @ 0x100138738 | `[mAnimation playAnimationId:id]`. |
| `BCLogic::animation` @ 0x1001385c8 / `displayObject` @ 0x1001385d8 | getters. |
| `BCLogic::id` @ 0x1001385ac | `[mDisplayObject id]`. |
| `BCLogic::parentTweenable` @ 0x1001385e8 | returns `mDisplayObject`: the frame of every extra channel is applied relative to it (`BCBinaryAnimationDef::applyFrame…` @ 0x1001a0ca4). |
| `BCLogic::setBinaryFrameDef:tweenableChannelArray:` @ 0x100138b50 | if `mDisplayObject` `isKindOfClass:BCMultiModel`, forward the call to it (this is how a multi-model gets its picture each frame); otherwise nothing. |
| `BCLogic::isSoundDisabled` @ 0x100138bf0 | NO (frame sounds play). `PygmyLogic` overrides. |
| `BCLogic::onAnimationMoveEnabled` @ 0x100138cc8 | `mLastMoveXPos = mLastMoveYPos = 0`. Sent by `BCAnimation::playAnimationId:` @ 0x1001a02f8 when the new animation is move-enabled. |
| `BCLogic::applyMoveXPos:moveYPos:` @ 0x100138bf8 | Root motion, see below. Sent by `BCBinaryAnimationDef::applyMovement:animationListener:` @ 0x1001a109c. |
| `BCLogic::animXFlip` @ 0x100138ce4 / `animYFlip` @ 0x100138d30 | `setXScale:(-xScale)` / `setYScale:(-yScale)` on the display object. |

`applyMoveXPos:mx moveYPos:my` (from the disassembly; the x term is computed in double
precision because of `fabs`):

```
p = displayObject.pos; s = displayObject.scale
p.x = (float)( (double)(s.x * (mx - mLastMoveXPos)) / (double)fabsf(s.x) + (double)p.x )
p.y = p.y + s.y * (my - mLastMoveYPos)          // f32
mLastMoveXPos = mx; mLastMoveYPos = my
```

So x moves by the sign of `scale.x` only (a mirrored sprite walks the other way), while y
is scaled by `scale.y`. `scale.x == 0` gives NaN.

Frame actions: each binary frame may name a selector; `BCBinaryAnimationDef::applySoundAndAction:animationListener:`
@ 0x1001a0bb4 sends `NSSelectorFromString(name)` to the logic (and plays the frame's sound
unless `isSoundDisabled`). `animXFlip`/`animYFlip` are the framework's two action
methods; gameplay classes define many more (`anim…` methods). A port needs a
string → method table per class.

---

## 7. Behaviours

A behaviour is a `BCSequence` (data-driven script, see the sequence spec) whose delegate
is the logic.

### 7.1 `BCLogic::setBehavior:` @ 0x1001385f8

```
def = [BCLibrary getSequenceDefById:id]          // NULL for nil or unknown ids
if mSequence != nil:
    [mSequence setSequenceDef:def]               // reset stack to [def @ item 0], firstTime = YES,
                                                 // play (or stop if def is NULL)
else:
    mSequence = [[BCSequence alloc] initWithSequenceDef:def delegate:self]   // plays if def != NULL
    if mbHibernate:
        mbHibernateGameFrameRunning = YES        // quirk, section 3.4
        [mSequence hibernate]
```

- `BCLibrary::getSequenceDefById:` @ 0x100023924 converts the id to UTF-8 in a static
  buffer (`DAT_1007cd818`, `getBytes:maxLength:1023 usedLength:&n encoding:4 …`), stores a
  NUL at `buffer[n]` and looks the C string up with `std::map::operator[]`
  (`FUN_100026f00`). An unknown id therefore yields NULL (and inserts an empty entry).
  Setting a NULL def stops the behaviour.
- **A nil id is undefined behaviour** in the original. The message to nil does not write
  `n`, so `ldur x8, [x29, #-0x28]` @ 0x100023990 reads an uninitialised stack slot and
  `strb wzr, [x21, x8]` stores a NUL at that offset from the buffer, which still holds the
  previous id. The most likely result is a stray write and a lookup of the previous id.
  No shipped data path triggers it: all 12 `setBehavior` items carry a `behavior` param.
  The port treats a nil or absent id like an unknown one (NULL def, the behaviour stops).
  This is a deliberate deviation, and the same rule as `docs/re/sequences.md` §3.1.
- `BCSequence::play` @ 0x1000c01c4 registers the sequence as a game-frame listener
  (order 10) unless hibernating, in which case the start is deferred.
- `setBehavior:X` from inside one of the logic's own item handlers. Let D be the definition
  at the stack top when the handler was called, s the stack level and k the item index
  (`BCSequence::runSequence:` @ 0x1000c02f8 compares exactly these three after the call;
  `BCSequence::onGameFrame:` @ 0x1000c0520 re-runs the top while its definition changed to
  a non-NULL one):
  - X unknown (NULL def): the stack is cleared and the sequence stops at once.
  - X ≠ D: `runSequence:` returns and `onGameFrame:` runs X from item 0 (`firstTime = YES`)
    in the same frame.
  - X = D with s > 0 or k > 0: the restart is noticed (the level or the item index changed)
    and X starts at item 0 (`firstTime = YES`) on the next frame.
  - X = D with s = 0 and k = 0: the restart is invisible. None of the three values changed,
    so `runSequence:` clears `firstTime` (losing the YES set by `setSequenceDef:`) and applies
    the handler's return code as if nothing happened: 2 advances to item 1 and continues in
    the same frame, 1 advances and yields, 0 or 3 re-run item 0 next frame with
    `firstTime = NO`.
- Outside the sequence's own run (touch callbacks, listener calls, other objects'
  `onGameFrame:`), `setSequenceDef:` resets the stack and `play`s. The new definition runs
  the next time the sequence's `onGameFrame:` is called. From a touch callback, which runs
  before the frame's listener snapshot is taken (4.1), that is always the same frame. From
  anywhere else it is the same frame only if the sequence is in this frame's snapshot and
  its turn has not come yet, otherwise the next frame (for instance when set from a logic's
  own `onGameFrame:`: order 40 runs after every sequence).

Other entry points:

- `BCLibrary::setBehavior:::` @ 0x10002598c (sequence item, any delegate): params
  `[0] displayGroupId`, `[1] behavior`, `[2] id` →
  `[[[BCLibrary getDisplayGroupById:p0] getDisplayObject:p2] logic] setBehavior:p1]`;
  returns 2. A display object without logic makes it a no-op (message to nil).
- `BCScreenLogic::setScreenLayer:behavior:` @ 0x10001ce60: removes layer *n*'s button
  group (`removeFromDisplayGroup`, then nil) and sends `setBehavior:` **to self** (not to
  the layer's logic; confirmed in the disassembly: receiver is `x20 = self`).
- `MessageLogic::showMessage:` (section 11.1), `OneShotAnimLogic::initWithSequenceId:` (3.1).
- `OutHouseLogic` overrides `setBehavior:` (gameplay spec).

### 7.2 Item dispatch from the logic side

For the current item, `BCSequence::runSequence:` @ 0x1000c02f8 does:

```
if delegate != nil and [delegate respondsToSelector:item.selector]:
    r = [delegate item.selector](sequence, firstTime, item)          // the logic handles it
else:
    r = [BCLibrary handleSequenceItem:item.selector sequence:sequence
                            firstTime:firstTime sequenceItemDef:item]
```

`BCLibrary::handleSequenceItem:sequence:firstTime:sequenceItemDef:` @ 0x100023df0 simply
sends the same selector to the `BCLibrary` singleton. `respondsToSelector:` walks the
class hierarchy, so any handler defined on `BCLogic`, `SpriteLogic`, … or the concrete
class takes precedence over `BCLibrary`. Handler signature:
`int handler(id self, SEL _cmd, BCSequence *seq, BOOL firstTime, BCSequenceItemDef *def)`;
`def->params[i]` is the *i*-th positional `NSString` (or NULL); a NULL param's
`floatValue`/`intValue`/`boolValue` is 0 (message to nil). `[seq deltaTime]`
(`BCSequence::deltaTime` @ 0x1000c0018) is the frame's `dt`.

Return values as interpreted by `runSequence:`. Before looking at them it checks whether
the handler changed the sequence, by comparing the stack index, the definition at the
calling level and that level's item index with their values before the call. If the stack
index changed (a push, or a reset to level 0 from a pushed level), the entry at the old
index is advanced past the calling item and `firstTime` is set. If the definition or the
item index changed (`setBehavior:`/`setSequenceDef:`), nothing is advanced. In these cases
`runSequence:` returns at once. `BCSequence::onGameFrame:` then runs the new stack top in
the same frame if its definition is a different, non-NULL one (a push or a new
behaviour), and otherwise continues next frame (for example a behaviour restarted with the
same definition starts at item 0 next frame). A restart with the same definition from item
0 at level 0 changes none of the three values and goes unnoticed (7.1). Otherwise:

| r | Meaning |
|---|---|
| 0 or 3 | Not finished: keep this item, call it again next frame with `firstTime = NO`. |
| 1 | Finished: advance to the next item, but stop processing for this frame. |
| 2 | Finished: advance and immediately run the next item in the same frame. |
| other | Re-run the same item immediately (no handler returns this). |

Completion: when the bottom sequence runs past its last item, `BCSequence::onGameFrame:`
clears the stack (`def = NULL`, index 0) and stops itself. Nothing is sent to the logic.
Pushed sub-sequences pop back to their caller in the same frame. Logic-level "done"
signals are explicit items at the end of the scripts (`die`, `oneShotComplete`,
`onMessageComplete`, `closeScreenComplete`, `multiModelEvent`, class-specific items that
call listeners) and `waitForAnimation` synchronises with the animation.

Selectors handled by both `BCLogic` and `BCLibrary`: `setPos`, `setAlpha`, `tweenAlpha`,
`setAnimation`, `setTextureDef`, `stall`. Inside a logic's behaviour the `BCLogic`
versions always win, which means their `id`/`displayGroupId` params are ignored and they
act on the logic's own display object — except `setTextureDef`, which defers to
`BCLibrary` when its `id` param is present.

---

## 8. Sequence-item handlers of the framework classes

Parameter names are the v1.45.3 `sequence.xml` attribute names. Positions were matched by
comparing compiled 1.48.2 items (`processSequenceDefs:` list at 0x100643a60) with the
1.45.3 XML items of the same sequence ("data"), or deduced from the handler's use of
the remaining positions ("code"). They agree with the independently produced
`assets/data/sequence-methods.json` (which also gives the positions of the params the
logic versions ignore, e.g. `setAnimation`: `[0] displayGroupId`, `[2] stateId`, `[3] id`;
`setAlpha`: `[1] displayGroupId`, `[2] id`; `setPos`: `[3] id`, `[4] displayGroupId`).
All handlers return 2 unless stated.

| Item | Handler | Params | Semantics |
|---|---|---|---|
| `removeFromDisplayGroup` | `BCLogic` @ 0x100138d7c | — | `[mDisplayObject removeFromDisplayGroup]`. |
| `setAnimation` | `BCLogic` @ 0x100138dac | `[1] animation`, `[4] percent` (data) | `[mAnimation playAnimationId:p1]`; if p4 non-NULL, `[mAnimation setFramePercent:p4.floatValue]` (`BCAnimation::setFramePercent:` @ 0x1001a0430; `BCAnimation::onGameFrame:` @ 0x1001a04a0 then shows frame `(uint)(length·percent + 0.5)` and never completes on its own). Other params ignored. |
| `chooseAnimation` | `BCLogic` @ 0x100138e30 | `[0] list` (data) | Split p0 on `","` (literal 0x10038a130 — the decompiler shows `&cf__`, which is the `"_"` string; the disassembly uses `","`); `r = arc4random()` (u32); play `items[r % count]`. |
| `setTextureDef` | `BCLogic` @ 0x100138edc | `[0] texture`, `[1] displayGroupId`, `[2] id` (data) | If p2 non-NULL: return `[BCLibrary setTextureDef:::]` (acts on that other object). Else `[mAnimation stop]`, `[mDisplayObject setTextureDef:[BCLibrary getTextureDefById:p0]]`. |
| `waitForAnimation` | `BCLogic` @ 0x100138fcc | — | 2 if `[mAnimation isAnimationComplete]`, else 0. |
| `setOrigin` | `BCLogic` @ 0x100139004 | `[0] zPos`, `[1] xPos`, `[2] yPos`, `[3] inPlace` (data) | Reads all three (NULL → 0). If p3 non-NULL and `boolValue`: `setOriginXPosInPlace:`/`Y…`/`Z…`; else `setOriginXPos:`/`setOriginYPos:`/`setOriginZPos:` on the display object. |
| `setVel` | `BCLogic` @ 0x100139140 | `[0] zVel`, `[1] yVel`, `[2] xVel` (x data, y/z code) | Each non-NULL param overwrites that component of `mPosVel`. |
| `setScale` | `BCLogic` @ 0x1001391e4 | `[0] xScale`, `[1] zScale`, `[2] yScale` (x data) | Non-NULL params overwrite components of the display object's scale. |
| `setRot` | `BCLogic` @ 0x100139284 | `[0] zRot`, `[1] xRot`, `[2] yRot` (code) | Non-NULL params overwrite components of the display object's rotation (degrees). |
| `setPos` | `BCLogic` @ 0x100139324 | `[0] xPos`, `[1] zPos`, `[2] yPos` (x, y data) | Non-NULL params overwrite components of the display object's position. |
| `offsetPos` | `BCLogic` @ 0x1001393c4 | `[0] zPos`, `[1] xPos`, `[2] yPos` (y data) | Non-NULL params are added to the position. |
| `setPivot` | `BCLogic` @ 0x10013947c | `[0] zPos`, `[1] xPos`, `[2] yPos` (data) | `setPivotXPos:`/`Y`/`Z` with all three (NULL → 0). |
| `rotate` | `BCLogic` @ 0x100139550 | — | `z = zRot + 2.0`; if `z <= 360` set `zRot = z`, return 0; else set `zRot = 0`, return 2. Frame-rate dependent (+2° per call). |
| `stall` | `BCLogic` @ 0x1001395cc | `[0] time`, `[1] minTime`, `[2] maxTime` (data) | On `firstTime`: if p1 NULL, `mStallTime = p0.floatValue`; else `lo = (int)(p1·1000)`, `hi = (int)(p2·1000)` (truncation), `r = arc4random()`, `n = (u32)(hi − lo + 1)`, `mStallTime = (float)(int)(r % n + lo) / 1000` (u32 arithmetic; `n == 0` gives `r`). Then every call: if `mStallTime <= 0` return 2, else `mStallTime −= dt`, return 0. |
| `tweenAlpha` | `BCLogic` @ 0x1001396dc | `[0] displayGroupId`, `[1] time`, `[2] target`, `[3] id` (data) | `a = alpha; t = p2`. On `firstTime`: `mStallTime = p1`, `mDeltaValue = (t − a)/p1`. Every call: `s = mStallTime`; `v = (s <= 0) ? t : t − s·mDeltaValue`; `mStallTime = s − dt`; `setAlpha:v`; return `s <= 0 ? 2 : 0`. Own display object only. |
| `setAlpha` | `BCLogic` @ 0x1001397f8 | `[0] alpha` (data) | `[mDisplayObject setAlpha:p0]`. Own display object only. |
| `triggerEpisodeEvent` | `SpriteLogic` @ 0x100199570, `MultiModelLogic` @ 0x10000bce0 | `[0] event` (data) | `[[IdolManager instance] triggerEpisodeEvent:p0]`. |
| `setApocalypseCloudDragOffset` | `SpriteLogic` @ 0x1001995c0 | `[0]` x offset, `[1]` y offset (no XML sample; compiled items use e.g. `-32`, `-16`) | If `mApocalypseCloudLogic`: `[cloud setDragXOffset:p0 dragYOffset:p1]`. |
| `die` | `MultiModelLogic` @ 0x10000bc38 | — | Section 3.5. |
| `multiModelEvent` | `MultiModelLogic` @ 0x10000bca0 | `[0] eventId` (data) | `[mMultiModelLogicListener onMultiModelEvent:self eventId:p0]`. |
| `disableGameLayer` | `BCScreenLogic` @ 0x10001cec4 | — | Section 10. |
| `enableGameLayer` | `BCScreenLogic` @ 0x10001cfa4 | — | Section 10. |
| `setLayerAnimation` | `BCScreenLogic` @ 0x10001d054 | `[0] layer`, `[1] animation` (data) | `[[layers[p0].mLogic animation] playAnimationId:p1]`. |
| `enableButtonLayer` | `BCScreenLogic` @ 0x10001d0d0 | `[0] layer`, `[1] displayGroup` (data) | Section 10. |
| `disableButtonLayer` | `BCScreenLogic` @ 0x10001d208 | `[0] layer` (data) | Section 10. |
| `closeScreenComplete` | `BCScreenLogic` @ 0x10001d278 | — | `[self closeScreenNow]`. |
| `countDownTimer` | `MessageLogic` @ 0x10019efb8 | `[0] time` (data) | On `firstTime`: `mStallTime = p0`. Every call: set the text of object `@"Timer"` in group `@"ChallengeTimer"` to `[NSString stringWithFormat:@"%d", (int)mStallTime]` (truncation, before decrementing); then if `mStallTime <= 0` return 2, else `mStallTime −= dt`, return 0. |
| `onMessageComplete` | `MessageLogic` @ 0x10019f0e8 | — | `mbActive = NO`; remove the display object from its group; `[mMessageLogicListener onMessageComplete:self]`. |
| `oneShotComplete` | `OneShotAnimLogic` @ 0x100037a74 | — | Section 3.5. |

All float comparisons above are `fcmp` + `b.ls`, i.e. `<=` with NaN treated as "not <=".

---

## 9. Touch API

### 9.1 Event pipeline

1. UIKit → `BCView::touchesBegan/Moved/Ended/Cancelled:withEvent:` @ 0x100177cbc–0x100177cec,
   all forwarding to `BCView::createTouchEvents:withEvent:` @ 0x100177264.
2. For each `UITouch`: if `mbTouchActive` is NO (`BCView::setTouchActive:` @ 0x100177254;
   off at start, enabled by the view controller, e.g. its `enableTouch` item), only a
   *began* touch produces `[delegate preStartTouchBeganXPos:yPos:]` (a no-op in
   `PocketGodViewController`) and the touch is otherwise dropped. Otherwise:
   `BCView::getTouch:` @ 0x1001770b0 finds or creates the `BCTouch` wrapping that
   `UITouch` (one object per finger for its whole life), retains it, removes it from
   `mUITouchArray` if the phase is ended/cancelled, and appends an event to a 1000-record
   ring buffer. Record *i* is 0x30 bytes at `0x1008162d0 + 0x30·i`:

   | Off | Field | Value |
   |---|---|---|
   | 0x00 | `BCTouch*` | the touch (retained by the producer) |
   | 0x08 | u8 valid | 1, written last (`DAT_1008162d8` for record 0) |
   | 0x10 | u64 phase | `[uiTouch phase]` |
   | 0x18 | f32 timestamp | `(float)[uiTouch timestamp]` |
   | 0x1c | i32 tap count | `[uiTouch tapCount]` |
   | 0x20, 0x24 | f32 x, y | `locationInView:` (points) |

   The write index `DAT_1008162cc` is advanced first (999 wraps to 0), then the record is
   filled. Nothing checks for overflow. With 1000 or more events queued between two frames
   the write index laps the read index: unread records are overwritten (their touch retains
   leak) and the queue looks empty or short, so events are lost or delivered out of order.
   This cannot happen in practice; a port can use an unbounded queue.
3. At the start of the next frame `BCView::processTouchEvents:` @ 0x100176c54 drains the
   ring while the read index `DAT_1008162c8` differs from the write index. It stops
   (without advancing) at the first record whose valid byte is 0. Otherwise it advances the
   read index, then writes phase, timestamp, game time (`now`), tap count and x/y into the
   `BCTouch` and adds the touch to `mGameTouchArray` if absent. It then dispatches to the
   view delegate (`PocketGodViewController`) by phase (`cmp x8, #4; b.hi`):
   began → `touchBegan:allTouches:`; moved → `touchMoved:allTouches:`;
   ended → `touchEnded:allTouches:`, then remove from `mGameTouchArray`;
   cancelled → `touchCancelled:allTouches:` (= `touchEnded:allTouches:`, @ 0x100070a00), then
   remove; stationary (2) and any phase above 4 → no dispatch. Last, it releases the touch
   and clears the valid byte. So every event, stationary included, updates the `BCTouch`
   fields and puts the touch in `mGameTouchArray`; only ended/cancelled events remove it.
   `allTouches` is `mGameTouchArray`: every finger currently down, including this one.
   (The `cbz x9` @ 0x100176dac tests the record's address and is dead code.)

`BCTouch.mTimeStamp` is f32 seconds since boot. Its resolution is 2⁻⁷ s (7.8 ms) after one
day of uptime and 2⁻⁴ s after 12 days, so code that differentiates timestamps (the view
controller's world drags) inherits that coarseness; `mGameTime` is the frame's `now` (4.1).

Touch positions stay in view points (UIKit, origin top-left). Logic code converts with
`[displayGroup viewToDisplayObject:]` when it needs world coordinates.

### 9.2 Dispatch to logic objects

`PocketGodViewController::createTouchLogicArray:` @ 0x10006e730 fills a 1000-slot C array
(`mTouchLogicArray`, `[1000@]`) with
`[BCView.activeDisplayGroup addSelectableLogicsToArray:]` and retains every entry.
`BCDisplayGroup::addSelectableLogicsToArray:` @ 0x100067204 /
`BCDisplayObject::addSelectableLogicsToArray:` @ 0x10010e3dc append an object's logic if it
has one and `mbIsSelectable`, group first, then its children in order (pre-order). The
array is walked **from the last entry to the first** (front-most drawn object first) and
released afterwards. Only logics whose display objects are in the active display tree
and selectable can receive touches. That is the whole eligibility rule: neither
`addSelectableLogicsToArray:` (both versions; `isSelectable` @ 0x10010e760 is a plain
getter) nor the hit test (9.3) looks at `mbHidden`, alpha or colour. A hidden or fully
transparent selectable object still receives touches and still blocks the objects behind
it. A group's own visibility does not matter either: its children are walked regardless.
A Bevy port must therefore not filter touch targets by `Visibility`, `ViewVisibility` or
alpha; it must walk the active tree with the original's rules.

`touchBegan:allTouches:` @ 0x10006eaa0 (after the view controller's two-finger "pinch the
sun" pre-check, which may consume the touch):

```
found = NO
for logic in reversed(array):
    if not logic.displayObject.isSelectable: continue          // re-checked here only
    if not found and [logic doesPointIntersect:touch.pos]:
        touch.activeObject = logic                            // retained by the touch
        [logic touchBeganInside:touch allTouches:all]
        found = YES
    else:
        [logic touchBeganOutside:touch allTouches:all]
if found: return          // otherwise world gestures (sun, rainbow, ocean...) and only
                          // when no screen/minigame flag is set — view-controller spec
```

`touchMoved:allTouches:` @ 0x10006f7fc (logic dispatch runs unless the touch is held by
an exclusive world gesture: pinch-sun, sun/moon drag, crack drag, hurricane, underwater
power, whirlpool, lightning, or an active rainbow/locust stroke; cloud/ocean/ape-wench/
underwater-camera drags happen in addition):

```
found = NO
for logic in reversed(array):
    if not found and [logic doesPointIntersect:touch.pos]:
        if touch.activeObject != logic:
            if touch.activeObject != nil: [touch.activeObject touchMovedExited:touch allTouches:all]
            touch.activeObject = logic
            [logic touchMovedEntered:touch allTouches:all]
        [logic touchMovedInside:touch allTouches:all]
        found = YES
    else:
        if touch.activeObject == logic:
            touch.activeObject = nil
            [logic touchMovedExited:touch allTouches:all]
        [logic touchMovedOutside:touch allTouches:all]
```

`touchEnded:allTouches:` @ 0x1000701a0 (same exclusivity rule for the end of world
gestures):

```
found = NO
for logic in reversed(array):
    if not found and [logic doesPointIntersect:touch.pos]:
        [logic touchEndedInside:touch allTouches:all]; found = YES
    else:
        [logic touchEndedOutside:touch allTouches:all]
```

`touch.activeObject` is not cleared on end. `PocketGodViewController::cancelAllTouches`
@ 0x1000707c8 (called by the view controller on many screen transitions) runs exactly
the `touchEnded` logic loop for every touch in `allTouches`, using a second array
`mCancelTouchLogicArray`; it does not remove the touches, so logics may later receive a
second, real end for the same finger.

Every selectable logic in the active tree sees every touch: one *inside* callback at most
per event (front-most hit), *outside* callbacks for all the others (for `began`, only
those still selectable).

### 9.3 Hit test

`BCLogic::doesPointIntersect:` @ 0x10013984c and its variants (0x100139870, 0x100139898,
0x1001398b8) forward to `BCDisplayObject::doesPointIntersect:checkSelect:debug:`
@ 0x10010f9f0 with `checkSelect = YES` unless given:

```
if checkSelect and not mbIsSelectable: return NO
q = [self viewToDisplayObject:viewPoint]                 // local coordinates
xmin = origin.x + colXMin; xmax = origin.x + colXMax
if scale.x < 0: xmin = -origin.x - colXMax; xmax = -origin.x - colXMin
(same for y with origin.y, colYMin/colYMax, scale.y)
return xmin < q.x < xmax and ymin < q.y < ymax           // strict
```

No other state is consulted (not `mbHidden`, alpha, hibernation or the parent's
selectability); the `debug` flag only logs.

### 9.4 Callback contract

All eight touch methods of `BCLogic` (0x1001398d4–0x1001398f0) are empty:
`touchBeganInside:allTouches:`, `touchBeganOutside:…`, `touchMovedEntered:…`,
`touchMovedExited:…`, `touchMovedInside:…`, `touchMovedOutside:…`,
`touchEndedInside:…`, `touchEndedOutside:…`. 60 gameplay classes override them (plus
`BCButtonLogic`).
Typical gameplay pattern (e.g. `BoulderLogic` @ 0x10001eb04–0x10001eef0): remember the
`BCTouch` as `mActiveTouch` on began-inside, ignore other fingers
(`mActiveTouch == nil || mActiveTouch == touch`), copy `touch.pos` into
`mTouchScreenPos` on every moved/entered/exited/outside callback, and let a sequence item
(`drag`) pull the object toward the touch each frame.

### 9.5 Buttons: `BCButtonLogic`

| Callback | Behaviour |
|---|---|
| `touchBeganInside` @ 0x100091d38 | `mbTouchBeganInside = YES`; `[listener onButtonPress:self touch:touch]`. |
| `touchBeganOutside` @ 0x100091d74 | `mbTouchBeganInside = NO`. |
| `touchMovedEntered` @ 0x100091d84 | nothing (does not re-arm). |
| `touchMovedExited` @ 0x100091d88 | `mbTouchBeganInside = NO`. |
| `touchMovedInside` / `touchMovedOutside` @ 0x100091d98 / 0x100091d9c | nothing. |
| `touchEndedInside` @ 0x100091da0 | `mbTouchBeganInside ? onButtonRelease:touch: : onButtonReleaseOutside:touch:` to the listener; then `mbTouchBeganInside = NO`. |
| `touchEndedOutside` @ 0x100091e04 | `[listener onButtonReleaseOutside:self touch:touch]`; `mbTouchBeganInside = NO`. |

Consequences to keep: the armed flag is per button, not per finger (a second finger
beginning elsewhere disarms it); sliding off and back on cancels the click; every button
in the active tree receives `onButtonReleaseOutside:` for every finger that lifts
anywhere else (whenever the end reaches the logic dispatch, i.e. it does not end an
exclusive world gesture), and once more for each finger in `cancelAllTouches`. Buttons
have no game frame and no visual feedback of their own (the listener animates them).

**Touch blockers**: 16 table buttons get no listener (indices 3–6 `Blackbox`/`BlackBox`
of the story panels, 255, 258, 338–343, 482–485 `ButtonBlocker*`). Messages to their nil
listener do nothing, but a hit still sets `found`, so they swallow touches for whatever
is behind them and keep them from reaching the world gestures.

### 9.6 Drag, throw and flick

The framework has no drag, throw or flick velocity: `BCTouch` only carries position,
timestamps and tap count, and no `BCLogic`/`SpriteLogic` method reads touch history.
Each gameplay class implements its own, typically as a spring toward the touch inside a
`drag` sequence item, with the object's `mPosVel` carrying the release velocity into the
`fall`/free-flight item. Examples (documented with those classes): `PygmyLogic::drag:::`
@ 0x100132094, `BoulderLogic::drag:::` @ 0x10001e664 (spring force `-50·(pos − touch) − 6·vel`
computed as `(|d|·−50)/|d|`), `PygmyLogic(Story)::storyModeFlick:::` @ 0x100174d10, the
`on…Drag:` / `on…DragComplete:` listener callbacks in appendix A, and the world drags of
`PocketGodViewController` (sun, clouds, ocean, …, which do use `timeStamp`). Shared helper:
`FUN_1001a15d4` @ 0x1001a15d4 = `sqrtf(x² + y² + z²)` of a vec4.

---

## 10. Screens and layers: `BCScreenLogic`

A screen logic is a `MultiModelLogic` whose display object is placed in the shared group
`ScreenDisplayGroup`, which is itself attached on top of the current game root.

| Method | Behaviour |
|---|---|
| `openScreen` @ 0x10001cc90 | `mActiveDisplayGroup = BCView.activeDisplayGroup`; `mScreenDisplayGroup = getDisplayGroupById:@"ScreenDisplayGroup"`; `[mScreenDisplayGroup moveToDisplayGroup:mActiveDisplayGroup]` (appended, so drawn last); `[mDisplayObject moveToDisplayGroup:mScreenDisplayGroup]`; screen group `pos.x = pos.y = 0`, `rot.z = 0`. Subclasses call it via `super` and then usually `setScreenLayer:behavior:`. |
| `closeScreenNow` @ 0x10001cd88 | `[BCView setActiveDisplayGroup:mActiveDisplayGroup]`; remove the screen group and the screen's display object from their groups; for each of the 10 layers remove and clear `mButtonDisplayGroup`. |
| `setScreenLayer:logic:` @ 0x10001ce48 | `layers[n].mLogic = logic` (not retained). Subclasses use layer 0 = self, layer 1 = a secondary panel logic. |
| `setScreenLayer:behavior:` @ 0x10001ce60 | Section 7.1. |
| `disableGameLayer` item @ 0x10001cec4 | Remove the screen group from the game root; copy the game root's `pos` and `rot` (all 16 bytes each) into the screen group; `[BCView setActiveDisplayGroup:screenGroup]`. The screen group becomes the root: the game world is hibernated (stops updating) and no longer drawn. |
| `enableGameLayer` item @ 0x10001cfa4 | `[screenGroup moveToDisplayGroup:mActiveDisplayGroup]`; screen group `pos.x = pos.y = rot.z = 0`; `[BCView setActiveDisplayGroup:mActiveDisplayGroup]`. |
| `enableButtonLayer` item @ 0x10001d0d0 | `n = p0.intValue`; remove the old `layers[n].mButtonDisplayGroup` if any; `g = getDisplayGroupById:p1`; `g.pos.x/y = layers[n].mLogic.displayObject.pos.x/y`; `layers[n].mButtonDisplayGroup = g`; `[g moveToDisplayGroup:mScreenDisplayGroup]`. Requires `layers[n].mLogic` (the original dereferences a NULL `pos` otherwise). |
| `disableButtonLayer` item @ 0x10001d208 | If `layers[p0].mButtonDisplayGroup`: remove it and clear. |
| `setLayerAnimation` item @ 0x10001d054 | Section 8. |
| `closeScreenComplete` item @ 0x10001d278 | `closeScreenNow`. |

Button layers are how screens enable and disable their buttons: the `BCButtonLogic`s
live in display groups (e.g. `IdolDetailButtons`) that are only selectable while the
group is attached under the screen group, because touch dispatch only walks the active
tree.

---

## 11. Other framework classes

### 11.1 MessageLogic

- `showMessage:` @ 0x10019eebc: if not `mbActive`: `mbActive = YES` and move the display
  object into group `@"PocketGod"` (0x10038a270). Then `[self setBehavior:behaviorId]`.
- `countDownTimer` and `onMessageComplete` items: section 8. `setMessageLogicListener:`
  @ 0x10019ef5c retains. No table entry uses this class (5 dynamic creations elsewhere).

### 11.2 OneShotAnimLogic

`[[OneShotAnimLogic alloc] initWithSequenceId:seq]` (3.1); the creator positions
`displayObject`, moves it into a display group (which unhibernates it and starts the
sequence) and releases its own reference, e.g.
`PocketGodViewController(Game)::checkUnderwaterBubble:pos:` @ 0x100163ac4 spawns
`UnderwaterBubbles` at the touch position with `z = -277.12` in the underwater group (at
most every 0.4 s). The sequence must end with `oneShotComplete`.

### 11.3 IslandObjectLogic

- `enterIsland:` @ 0x1001255a4 (BOOL argument unused):
  `[[BCLibrary getDisplayMarkerById:mDisplayMarkerId] insertBefore:mDisplayObject]` —
  the object is placed in the marker's group just before the marker.
- `exitIsland` @ 0x100125610: `[mDisplayObject removeFromDisplayGroup]`.
- Its game frame is stopped in init; physics only runs if the quirk of 3.4 restarts it.

### 11.4 SpriteLogic extras

- Hurricane / whirlpool participation hooks, all inert in the base class (subclasses
  override): `blownByHurricane:` @ 0x100199304 → NO, `unblownByHurricane:` @ 0x10019930c
  no-op, `pickedUpByHurricane:` @ 0x100199310 → NO, `droppedByHurricane:` @ 0x100199318
  no-op, `pickedUpByWhirlPool:` @ 0x10019933c → NO, `droppedByWhirlPool:` @ 0x100199344
  no-op; getters `isBlownByHurricane` @ 0x1001992e4, `isPickedUpByHurricane` @ 0x1001992f4,
  `isPickedUpByWhirlPool` @ 0x10019932c, `hurricaneSpinState` @ 0x10019931c and
  `whirlPoolSpinState` @ 0x100199348 (pointers into the ivars). No getter for
  `mbBlownByWhirlPool`.
- Apocalypse disease cloud (`ApocalypseCloudLogic` is specified elsewhere):
  - `infectWithDisease:type` @ 0x1001997c8 (from the disassembly): if no cloud and
    `type != 0`: `mApocalypseDiseaseType = type`, `mApocalypseCloudLogic = [[ApocalypseCloudLogic apocalypseCloudWithSprite:self apocalypseDiseaseType:type apocalypseCloudStyleId:mApocalypseCloudStyleId] retain]`.
    Else if no cloud (type 0): nothing. Else if `type != 0`: `mApocalypseDiseaseType = type`,
    `[cloud setApocalypseDiseaseType:type]`. Else: `[self cureDisease]`.
  - `cureDisease` @ 0x100199768: `mApocalypseDiseaseType = 0`; if cloud: `deactivate`,
    `release`, nil.
  - `showApocalypseCloud` @ 0x1001996a4: copy the sprite's `pos` (16 bytes) into the
    cloud's display object, then `[spriteDisplay insertAfter:cloudDisplay]` (cloud drawn
    right after the sprite). `hideApocalypseCloud` @ 0x100199660 removes the cloud's
    display object. `apocalypseDiseaseType` @ 0x100199650 getter.
  - `setApocalypseCloudStyleId:` @ 0x100199288 retains (`@"Default"` initially; the table
    sets `@"DDE"` on entry 500).

---

## 12. Listener pattern

Engine → logic protocols (from `tmp/re/objc_meta.txt`):

| Protocol | Required methods | Implemented by |
|---|---|---|
| `BCGameFrameListener` | `id`, `onGameFrame:`, `gameFrameOrder` | `BCLogic`, `BCSequence`, `BCAnimation`, managers |
| `BCSequenceDelegate` | `id` (+ any item selectors, found by `respondsToSelector:`) | `BCLogic` and other script owners |
| `BCAnimationListener` | `id`, `onAnimationMoveEnabled`, `isSoundDisabled`, `applyMoveXPos:moveYPos:`, `parentTweenable`, `setBinaryFrameDef:tweenableChannelArray:` | `BCLogic` |
| `BCViewDelegate` | `preStartTouchBeganXPos:yPos:`, `touchBegan:allTouches:`, `touchMoved:allTouches:`, `touchEnded:allTouches:`, `touchCancelled:allTouches:` | `PocketGodViewController` |

Logic → owner listeners: each class declares `m<Name>LogicListener` typed
`id<<Name>LogicListener>` (exceptions: `BCButtonLogic.mButtonLogicListener` is
`<BCButtonLogicListener>`), set through `set<Name>LogicListener:` which retains the new
listener and releases the old. Six setters do one more thing. `NoPeeClamLogic`,
`NoPeeFishLogic`, `NoPeeMineLogic`, `NoPeeParticleLogic`, `NoPeeSeaBlobLogic` and
`NoPeeTreasureLogic` (`setNoPee…LogicListener:` @ 0x1000c0850, 0x1001359d4, 0x1000d58f0,
0x1001753cc, 0x100178f18, 0x10008ddf4) end with a tail-call
`[self setNoPeeObjectLogicListener:listener]` (`NoPeeObjectLogic` @ 0x1000346ac, never
overridden). That retains the same object again into the base class's
`mNoPeeObjectLogicListener` (`<NoPeeObjectLogicListener>`: `noPeePygmyLogic`, `noPeeLogic`,
`noPeeCollidableLogicArray`, `isGameActive`), and their protocols adopt
`NoPeeObjectLogicListener`. In the table this makes `NoPeeLogic` (entry 91) both listeners of
all 27 No Pee object entries (10 particles, 4 each of clams, fish, mines and sea blobs, and 1
treasure). Every other class/setter pair the table uses (47 of 53) only retains and assigns. The listener is called synchronously from touch
callbacks, item handlers and `onGameFrame:`, and is also queried (`isStoryActive`,
`islandMinXPos`, `pygmyLogicArray:`, …): listeners are both event sinks and the logic's
window onto game state. Framework listener protocols: `BCButtonLogicListener`
(`onButtonPress:touch:`, `onButtonRelease:touch:`, `onButtonReleaseOutside:touch:`),
`MultiModelLogicListener` (`onMultiModelEvent:eventId:`), `MessageLogicListener`
(`onMessageComplete:`). Every listener wired by `initLogicListeners` is a class
singleton (appendix A lists the protocols and their methods).

---

## 13. The generated logic-object table

### 13.1 `getLogic:`

`BCLibrary::getLogic:` @ 0x100023404 is

```
adrp x8, 0x1007e3000 ; add x8, x8, #0x30 ; ldr x0, [x8, w2, sxtw #3] ; ret
```

i.e. `return ((id *)0x1007e3030)[(int)index]` with no bounds check. The table occupies
`0x1007e3030 … 0x1007e43e8` in `__common`, 632 contiguous 8-byte slots: index *i* is
`DAT_<0x1007e3030 + 8·i>`. Every slot is written exactly once, by
`createLogicClasses`, in increasing index order, and no other code writes them. All 602
`getLogic:` call sites in the decompiled game code use indices 0–631 (600 with a constant
index, 2 computed — 13.7).

### 13.2 `BCLibrary(Logic) -createLogicClasses` @ 0x10013feb8

Straight-line generated code (3964 `bl _objc_msgSend` + `ret`, no other branch). It is
sent by `PocketGodViewController::loadGame` @ 0x10006a030 after all data is loaded and
the preferences/skin managers exist. Per entry, in index order:

1. Obtain the display object, one of
   - `[[BCMultiModel alloc] initWithId:ID]` (353 entries) — released right after the
     logic is created (the logic keeps it alive);
   - `[[BCLibrary instance] getDisplayGroupById:G]` (3 entries: `NoPeeMapDisplayGroup`,
     `NoPeeDisplayGroup`, `FailBoatDisplayGroup`);
   - `[[[BCLibrary instance] getDisplayGroupById:G] getDisplayObject:O]` (276 entries).
2. `[[Class alloc] initWithDisplayObject:obj]` (631) or, for `IslandObjectLogic`,
   `initWithDisplayObject:obj displayMarkerId:@"MidOceanMarker"` (1), stored into the slot.
3. Optional init steps (their order differs per entry and is kept in `init_steps`):
   `[[logic animation] playAnimationId:A]` (268), stores through `[obj pos]` (210: 207×
   `z = -277.12`, 2× `x = 0, y = 0, z = -277.12` for the two full-screen screen logics,
   1× `(400, 98, -1200)`), one store through `[obj scale]` (`x = y = 3`), and setters:
   `setIslandLocationState:` (14), `setApocalypseDoorType:` (14), `setAlwaysOpen:YES` (13),
   `setApocalypseGemType:` (14), `setApocalypseDiseaseType:` (2), `setBehavior:`
   (2, `BattleLavaPitInit`), `setApocalypseCloudStyleId:@"DDE"` (1).

`-277.12` (`0xc38a8f5c`) is the engine's unit depth: the draw methods compute
`k = −277.12 / pos.z` (`BCModel::drawOpenGL:` @ 0x1001187e8; the same constant appears in
`BCMultiModel`, `BCText` and `BCRect` drawing) and use it to scale the object and its
y distance from 160, so a model at `z = −277.12` is drawn at 1:1 (`docs/re/rendering.md`). A new
`BCMultiModel` starts at `z = 0`, hence the explicit store.

Init methods of composite logics fetch earlier entries with `getLogic:` (for instance
`IdolScreenLogic::initWithDisplayObject:` takes 213 entries in 103–329, `NoPeeLogic` 35 in
52–90, `MoronPestsQuestionScreenLogic` 29 in 10–44, `UnderwaterTempleLogic` 27 in 520–546,
`DanceDanceExecutionScreenLogic` 13 in 486–498). Every such reference points to a lower
index, so creating the table in index order and running each class's init as it is
created reproduces the original.

Boot order around the table (`PocketGodViewController::loadGame` @ 0x10006a030):
`createLogicClasses` → the view controller stores entries in its ivars and arrays
(`mTempleDoorLogic = getLogic:0x176`, `mApocalypseGemPedestalLogicArray` = 0x1b1…0x1b7, …;
`loadGame` fetches 128 distinct indices) → `StoryManager`, `DebugScreen`, `IdolManager`,
`GameUiManager`, `MapScreen` are created (their inits also fetch entries) →
`initLogicListeners`.

### 13.3 `BCLibrary(Logic) -initLogicListeners` @ 0x1001512f8

Straight-line (1592 `bl` + 1 tail-call `b _objc_msgSend`). Two kinds of statements:

- `[logic set<Name>LogicListener:[Singleton instance]]` — 304 entries get exactly one
  listener; 328 get none. Singletons: `PocketGodViewController` (141), `IdolScreenLogic`
  (69), `NoPeeLogic` (31), `DebugScreen` (18), `MoronPestsQuestionScreenLogic` (15),
  `MapScreen` (12), `UnderwaterTempleLogic` (9), `DanceDanceExecutionScreenLogic` (4),
  `MoronSnakeLogic` (2), `IdolManager`, `TempleDoorLogic`,
  `ApocalypseTimeTemplePlatformLogic` (1 each).
- `[[logic animation] setTweenable:D channelId:N]` — 261 bindings on 145 entries, where D
  is another entry's `displayObject` (160) or a group object (101). Channel N (1–23) of
  this logic's animation then drives D relative to the logic's own display object
  (section 6, `BCAnimation::setTweenable:channelId:` @ 0x10019fdac).

### 13.4 JSON schema (`assets/data/logic-objects.json`)

A JSON array of 632 objects ordered by `index`; keys always present (null when absent);
UTF-8 (all ASCII); floats are the shortest decimals that parse back, through f64 as
`serde_json` does, to the exact f32 the code stores.

| Field | Type | Meaning |
|---|---|---|
| `index` | int | `getLogic:` index = creation order. |
| `address` | string | Slot address, `"0x1007e3030"` + 8·index. |
| `class` | string | ObjC class of the logic object. |
| `init_selector` | string | `"initWithDisplayObject:"` or `"initWithDisplayObject:displayMarkerId:"`. |
| `display_marker_id` | string \| null | Second argument of the latter (`"MidOceanMarker"`). |
| `binding` | DisplayRef | The display object given to the initializer. |
| `init_steps` | InitStep[] | Steps run right after creation, in machine-code order. |
| `listeners` | Listener[] | `set…Listener:` calls of `initLogicListeners`, in order (0 or 1 here). |
| `tweenables` | Tweenable[] | `setTweenable:channelId:` calls on this entry's animation, in order. |
| `references` | Reference[] | Informational: functions that call `getLogic:` with this constant index, sorted by address. |

DisplayRef (`kind` tag): `new_model {class, id}` (`class` is always `"BCMultiModel"`),
`display_group {group}`, `group_object {group, object}`, and, only inside tweenables,
`logic_display_object {index}` (the `displayObject` of table entry `index`).

InitStep (`op` tag): `set_pos {x, y, z}` and `set_scale {x, y, z}` (each `f32 | null`;
null = component not written), `play_animation {animation_id}`,
`set_behavior {sequence_id}`, `set_island_location_state {value: int}`,
`set_apocalypse_door_type {value: int}`, `set_always_open {value: bool}`,
`set_apocalypse_gem_type {value: int}`, `set_apocalypse_disease_type {value: int}`,
`set_apocalypse_cloud_style_id {value: string}`. Setter ops are the selector in
snake_case; the value type comes from the method's ObjC type encoding.

Listener: `{selector, target}`, target `{kind: "singleton", class, index}` where `index`
is the table entry that the singleton's `+instance` returns, or null for non-logic
singletons (`PocketGodViewController`, `DebugScreen`, `MapScreen`, `IdolManager`).

Tweenable: `{channel_id: int, target: DisplayRef}`. Reference: `{function, address}`.

```rust
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LogicObjectDef {
    pub index: usize,
    pub address: String,
    pub class: String,
    pub init_selector: String,
    pub display_marker_id: Option<String>,
    pub binding: DisplayRef,
    pub init_steps: Vec<InitStep>,
    pub listeners: Vec<ListenerDef>,
    pub tweenables: Vec<TweenableDef>,
    pub references: Vec<CodeReference>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DisplayRef {
    NewModel { class: String, id: String },
    DisplayGroup { group: String },
    GroupObject { group: String, object: String },
    LogicDisplayObject { index: usize },
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum InitStep {
    SetPos { x: Option<f32>, y: Option<f32>, z: Option<f32> },
    SetScale { x: Option<f32>, y: Option<f32>, z: Option<f32> },
    PlayAnimation { animation_id: String },
    SetBehavior { sequence_id: String },
    SetIslandLocationState { value: i32 },
    SetApocalypseDoorType { value: i32 },
    SetAlwaysOpen { value: bool },
    SetApocalypseGemType { value: i32 },
    SetApocalypseDiseaseType { value: i32 },
    SetApocalypseCloudStyleId { value: String },
}

#[derive(Deserialize)]
pub struct ListenerDef {
    pub selector: String,
    pub target: ListenerTarget,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ListenerTarget {
    Singleton { class: String, index: Option<usize> },
}

#[derive(Deserialize)]
pub struct TweenableDef {
    pub channel_id: i32,
    pub target: DisplayRef,
}

#[derive(Deserialize)]
pub struct CodeReference {
    pub function: String,
    pub address: String,
}
```

Example (index 49):

```json
{"index": 49, "address": "0x1007e31b8", "class": "IslandObjectLogic",
 "init_selector": "initWithDisplayObject:displayMarkerId:", "display_marker_id": "MidOceanMarker",
 "binding": {"kind": "new_model", "class": "BCMultiModel", "id": "MisfitOceanRock"},
 "init_steps": [{"op": "play_animation", "animation_id": "MisfitOceanRockInitAnim"},
                {"op": "set_pos", "x": 400.0, "y": 98.0, "z": -1200.0},
                {"op": "set_scale", "x": 3.0, "y": 3.0, "z": null}],
 "listeners": [], "tweenables": [],
 "references": [{"function": "PocketGodViewController::loadGame", "address": "0x10006a030"}]}
```

Applying an entry in the port: build or look up the display object from `binding`, run
the class's initializer (section 3.1 plus the gameplay class's own init), execute
`init_steps` in order (`set_pos`/`set_scale` write only the non-null components), and,
after every entry exists and the singletons are created, apply `listeners` and
`tweenables` in table order.

### 13.5 Contents

56 classes: `MultiModelLogic` 289, `BCButtonLogic` 181, `ApocalypseTempleDoorLogic` 14,
`NoPeeParticleLogic` 10, `IdolIconLogic` 9, `IdolBattleRealmEffectLogic` 9,
`UnderwaterTempleSymbolLogic` 9, `IdolSelectionIconLogic` 8, `BattleEnergyBallLogic` 8,
`PygmyBottomLogic` 7, `ApocalypseGemLogic` 7, `ApocalypseGemPedestalLogic` 7,
`MoronBeeLogic` 5, `JellyfishLogic` 5, seven classes with 4 (`MoronDuckLogic`,
`NoPeeSeaBlobLogic`, `NoPeeFishLogic`, `NoPeeMineLogic`, `NoPeeClamLogic`, `GodIdolLogic`,
`IdolPedestalLogic`), `ApocalypseTempleFountainLogic` 2, and 34 classes with a single
instance. 537 entries have constant `getLogic:` references
(590 in total, from 62 functions); the remaining 95 are reached through the display tree
(touch dispatch, `[displayObject logic]`), listener callbacks or the computed sites below.

Index map (by display area):

| Indices | Area |
|---|---|
| 0–6 | `PygmyBottom`, `ApeVomit`, `DreamBubble` models; story panel blockers |
| 7–51 | Moron Pests minigame (backdrops, colour/synonym buttons, ducks, bees, question screen 45), Misfit island (46–49), `MoronTurtle` 50, `MoronSnake` 51 |
| 52–91 | No Pee minigame (objects, particles, map 88–90, `NoPeeLogic` 91) |
| 92–98 | `WaterWings`, jellyfish |
| 99–337 | Idol screen: idols, icons, detail/challenge items (130–212), level-up bars and items (213–230), buttons, battle panels, selection icons, `IdolScreen` 337 |
| 338–361 | Challenge/system-error blockers, debug screen buttons (344–361) |
| 362–402 | Battle of the Gods arena, God island (`TempleDoorLogic` 374), player/enemy weapons and energy, alerts, Fail Boat |
| 403–548 | Apocalypse: gems (403–409), pedestals, island, temples (Time 450–457, Locust 458–472, Dance 473–501, Plague 502–505, Purification 506–508, Closet 510–515, Xenu 516–519), underwater temple (520–548) |
| 549–631 | Map screen content and the in-game menus (island/god-power/customize/community/settings buttons) |

### 13.6 Singleton listeners

For every logic class used as a listener, `+instance` returns a static that the class's
`initWithDisplayObject:` sets to `self`, the class is allocated only in
`createLogicClasses`, and the table holds exactly one instance; that entry is the
listener (`target.index`):

| Class | `+instance` returns (set in init) | Entry |
|---|---|---|
| `MoronPestsQuestionScreenLogic` | `DAT_1007e5080` (`initWithDisplayObject:` @ 0x10006270c) | 45 |
| `MoronSnakeLogic` | `DAT_1007d1f48` (@ 0x10018bf1c) | 51 |
| `NoPeeLogic` | `DAT_1007d1f50` (@ 0x10019c880) | 91 |
| `IdolScreenLogic` | `DAT_100821f60` (@ 0x10018de5c) | 337 |
| `TempleDoorLogic` | `DAT_1007e7bd0` (@ 0x1000903bc) | 374 |
| `ApocalypseTimeTemplePlatformLogic` | `DAT_100821f50` (@ 0x100182350) | 453 |
| `DanceDanceExecutionScreenLogic` | `DAT_1007e43f8` (@ 0x1000351f0) | 501 |
| `UnderwaterTempleLogic` | `DAT_1007d1ef8` (@ 0x1000c7cb8) | 547 |

Non-logic singletons: `PocketGodViewController` (`DAT_1007d1ed8`), `DebugScreen`
(`DAT_1007cd7e0`, set in `DebugScreen::init` @ 0x100008638), `MapScreen` (`DAT_1007d1ef0`,
`MapScreen::init` @ 0x1000c0a18), `IdolManager` (`DAT_1007cdc20`, `IdolManager::init` @ 0x10002a1c4).

### 13.7 Computed `getLogic:` sites (not in `references`)

- `IdolScreenLogic::openIdolDetailPanel:` @ 0x100191b90: `getLogic:(i + 1) + 0xd4` for
  the *i*-th level-up item of the idol (i from 0) → entries 213 + i
  (`IdolDetailLevelUpBar1`…`9` are 213–221; the matching `IdolDetailLevelUpItem1`…`9`
  are 222–230).
- `PocketGodViewController(Game)::onApocalypseGemEventTriggered:` @ 0x10016544c: gem
  event 1–7 → `getLogic:0x193…0x199` (403–409, `ApocalypseGemTime`, `Locust`, `Dance`,
  `Plague`, `Purification`, `Closet`, `Xenu`); activates the gem if inactive and adds it
  to `mApocalypseGemLogicArray`.

### 13.8 Regenerating

```
tmp/re/venv/bin/python -I tools/re/extract_logic_objects.py \
    [--binary tmp/re/PocketGod.arm64] [--data tmp/re/data_after_init.bin] \
    [--classes tmp/re/classes.json] [--symbols tmp/re/symbols.tsv] \
    [--decomp-dir tmp/re/decomp/game] [--out assets/data/logic-objects.json]
```

Run from the repository root; it prints the cross-check summary and the two computed
`getLogic:` sites. Output is deterministic (fixed ordering, fixed float formatting).

### 13.9 How exactness was ensured

1. **Machine code as ground truth.** Both generated functions are executed under Unicorn
   (`__TEXT` from the binary, `__DATA` from `data_after_init.bin`). The `_objc_msgSend`
   stub (0x10029bab8, located through `symbols.tsv` and checked to be in `__stubs`) is
   hooked and replaced by a symbolic runtime: receivers are either class objects (name
   read from `class_t → class_ro_t → name`) or symbolic objects on a fake heap;
   selectors are read from `__objc_methname`; NSString arguments are decoded from their
   CFString structures (exact text — no reliance on the sanitised `cf_` labels, which
   are ambiguous, e.g. `cf_MoronSnake` labels both `"MoronSnake"` and `"Moron Snake"`,
   and can be wrong, e.g. `BCLogic::chooseAnimation:::` shows `&cf__` = `"_"` where the
   code uses `","`); int/BOOL/float arguments are decoded with the method's type
   encoding from `classes.json`; the floats written into the pointers returned by `pos`
   and `scale` are captured bit-exactly by a memory-write hook; table slots are captured
   by a write hook on `__DATA`.
2. **Coverage.** A capstone scan shows both functions are branch-free apart from the
   `objc_msgSend` calls (3964 + `ret`; 1592 + 1 tail call), so one run executes every
   statement once; the emulator counts 3964 + 1593 messages.
3. **Strictness.** The extractor aborts on any unknown receiver/selector combination, any
   read from a symbolic object, any `__DATA` store outside the table, a slot written twice,
   a display object bound twice or configured before its logic exists, a `pos`/`scale`
   access without stores, a component written twice, creation order ≠ index order, a
   singleton class with several table entries, or a float that does not round-trip.
4. **Independent cross-check against the decompiled C.** For both functions, the multiset
   of quoted selectors in `BCLibrary+Logic.c` must equal the emulated message multiset
   (5557 messages), the multiset of NSString literals must match after mapping each
   emulated CFString address to its label via `symbols.tsv` (1382 literals), and the
   class assigned to each of the 632 `DAT_` globals (parsed from the C text) must equal
   the emulated one. A negative test (one label and one class edited in a copy of the
   decompiled file) is reported as three mismatches and exit status 1.
5. Spot checks: the `getLogic:` instruction sequence (disassembly); the order of the
   `DAT_` assignments in the C text (strictly increasing, 632 distinct); the emulated
   steps of entries 45, 49, 337 and 500 read against the C text, including the 8-byte
   stores `0xc38a8f5c00000000` (y = 0, z = −277.12) and `0xc496000042c40000`
   (y = 98, z = −1200).

---

## 14. Porting notes

- Keep one dense table `Vec<LogicHandle>` indexed exactly like `getLogic:`; create entries
  in index order with the class's initializer, then apply listeners/tweenables after the
  singletons exist (13.2).
- Represent `mbHibernate` / `mbHibernateGameFrameRunning` / `mbGameFrameRunning` and the
  display hibernation refcount literally; the timing of when logic, sequences and
  animations run depends on them (3.3, 3.4).
- Run one system per frame that snapshots the registered listeners, sorts them stably
  by `gameFrameOrder`, and calls them with the clamped `dt` (4.1). Touch events must be
  dispatched before it.
- Use f32 everywhere the original does, with separate multiply and add (no `mul_add`)
  in `SpriteLogic` integration, and the f64 x-term of `applyMoveXPos` (6).
- Item handlers return the 0/1/2/3 codes of 7.2; logic handlers shadow library handlers
  of the same name.
- Touch dispatch walks the active display tree in reverse pre-order and calls inside /
  outside callbacks on every selectable logic (9.2); button and blocker semantics follow 9.5.

---

## 15. Open questions

- `NSMutableArray -sortUsingFunction:context:` is treated as stable (CoreFoundation's
  `CFSortIndexes` merge sort); this was not verified on device. If it were not, the
  relative order of listeners with equal `gameFrameOrder` would be unspecified. Whether
  any gameplay code depends on that order was not investigated.
- `mbHibernateGameFrameRunning = 1` in `BCLogic::setBehavior:` (3.4) looks like an
  original bug; it is reproduced as-is. Any logic whose first behaviour is set while it
  is hibernating starts receiving `onGameFrame:` at its next unhibernate, even if it keeps
  its game frame stopped. For the framework classes this only means `SpriteLogic`
  integration for `OneShotAnimLogic`/`IslandObjectLogic` (a no-op unless a velocity is
  set); gameplay subclasses with their own `onGameFrame:` were not audited for it.
- The XML names of the two `setApocalypseCloudDragOffset` params are unknown (the item is
  newer than v1.45.3); only their positions (x, y) are certain (`sequence-methods.json`
  calls them `dragXOffset`/`dragYOffset`, inferred from the setter). For `setRot` all three
  name↔position pairs, and for `setVel`, `setScale`, `offsetPos` some of them, come from
  the handler code rather than from matched data.
- `IdolScreenLogic::openIdolDetailPanel:` iterates a NULL-terminated list of level-up
  items; 9 bars exist (213–221), the maximum list length was not checked against the idol data.
- `references` only covers constant `getLogic:` indices found in the decompiled C; other
  access paths (display tree, listener arguments, `+instance`) are not listed there.

---

## Appendix A — listener protocols wired by `initLogicListeners`

Generated from the table and `tmp/re/objc_meta.txt` (required instance methods; protocols
marked "(none)" declare no methods of their own beyond `NSObject`). Counts in parentheses
are numbers of table entries.

| Protocol | Setter | Listener singleton(s) | Methods |
|---|---|---|---|
| `AdBlockerLogicListener` | `setAdBlockerLogicListener:` | MapScreen | `isChartboostAdVisible` |
| `ApocalypseCalendarLogicListener` | `setApocalypseCalendarLogicListener:` | PocketGodViewController | `isStoryActive`, `isTomPygmyActive`, `isApocalypseActive`, `endApocalypseComplete`, `endApocalypseCompleteNow`, `apocalypseGemPedestalLogicArray:` |
| `ApocalypseGemLogicListener` | `setApocalypseGemLogicListener:` | PocketGodViewController (7) | `isStoryActive`, `isGravityEnabled`, `isOceanEnabled`, `isApocalypseGemPedestalEnabled`, `isApocalypseActive`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyDragDisplayMarker`, `pygmyDisplayMarker`, `apocalypseGemPedestalLogicArray:`, `onApocalypseGemDrag:`, `onApocalypseGemDragComplete:`, `onApocalypseGemDie:` |
| `ApocalypseGemPedestalLogicListener` | `setApocalypseGemPedestalLogicListener:` | PocketGodViewController (7) | `isStoryActive`, `isApocalypseActive`, `apocalypseGemPedestalLogicArray:`, `endApocalypse` |
| `ApocalypseRoomFaderLogicListener` | `setApocalypseRoomFaderLogicListener:` | PocketGodViewController | `islandLocationState`, `onApocalypseRoomFaderChangeIslandLocationState:` |
| `ApocalypseTempleDoorLogicListener` | `setApocalypseTempleDoorLogicListener:` | PocketGodViewController (14) | `isStoryActive`, `islandLocationState`, `pygmyLogicArray:` |
| `ApocalypseTempleFountainLogicListener` | `setApocalypseTempleFountainLogicListener:` | PocketGodViewController (2) | `isStoryActive` |
| `ApocalypseTimeTempleLeverLogicListener` | `setApocalypseTimeTempleLeverLogicListener:` | PocketGodViewController | `isStoryActive`, `apocalypseTimeTemplePlatformLogic:` |
| `ApocalypseTimeTemplePlatformLogicListener` | `setApocalypseTimeTemplePlatformLogicListener:` | PocketGodViewController | `isStoryActive` |
| `BCButtonLogicListener` | `setButtonLogicListener:` | MoronPestsQuestionScreenLogic (6), NoPeeLogic, IdolScreenLogic (43), DebugScreen (18), TempleDoorLogic, ApocalypseTimeTemplePlatformLogic, DanceDanceExecutionScreenLogic (4), UnderwaterTempleLogic (9), MapScreen (11), PocketGodViewController (71) | `onButtonPress:touch:`, `onButtonRelease:touch:`, `onButtonReleaseOutside:touch:` |
| `BattleAlertLogicListener` | `setBattleAlertLogicListener:` | PocketGodViewController | `isScreenOpen`, `isStoryActive`, `onBattleAlertSelect` |
| `ClosetTempleClosetDoorLogicListener` | `setClosetTempleClosetDoorLogicListener:` | PocketGodViewController | `closetTempleThetanMeterLogic:`, `isTomPygmyAlive`, `createTomPygmy`, `onClosetTempleClosetDoorDrag:`, `onClosetTempleClosetDoorDragComplete:` |
| `ClosetTempleCouchLogicListener` | `setClosetTempleCouchLogicListener:` | PocketGodViewController | (none) |
| `ClosetTempleThetanMeterLogicListener` | `setClosetTempleThetanMeterLogicListener:` | PocketGodViewController | `pygmyLogicArray:`, `locustDisplayMarker`, `isStoryActive`, `isApocalypseActive`, `onThetanDrag:`, `onThetanDragComplete:` |
| `DanceBalloonBundleLogicListener` | `setDanceBalloonBundleLogicListener:` | PocketGodViewController | `isStoryActive`, `danceBalloonLogic:` |
| `DanceBalloonLogicListener` | `setDanceBalloonLogicListener:` | PocketGodViewController | `pygmyLogicArray:`, `pygmyDragDisplayMarker`, `onDanceBalloonDrag:`, `onDanceBalloonDragComplete:` |
| `DanceDanceExecutionLogicListener` | `setDanceDanceExecutionLogicListener:` | PocketGodViewController | `stopBackgroundLoop`, `playBackgroundLoop:`, `onDanceDanceExecutionStart:` |
| `DanceDanceExecutionScreenLogicListener` | `setDanceDanceExecutionScreenLogicListener:` | PocketGodViewController | `onDanceDanceExecutionStart:`, `onDanceDanceExecutionStop:`, `onDanceDanceExecutionScore:` |
| `DiscoBallLogicListener` | `setDiscoBallLogicListener:` | PocketGodViewController | `stopBackgroundLoop`, `playBackgroundLoop:` |
| `DiscoLightsLogicListener` | `setDiscoLightsLogicListener:` | PocketGodViewController | (none) |
| `DreamBubbleLogicListener` | `setDreamBubbleLogicListener:` | PocketGodViewController | `islandLocationState`, `onDreamBubbleChangeIslandLocationState:` |
| `EpisodeAlertLogicListener` | `setEpisodeAlertLogicListener:` | PocketGodViewController | `optionPreferences`, `isScreenOpen`, `isStoryActive`, `onEpisodeAlertOpenChallengeDetail:` |
| `FailBoatLogicListener` | `setFailBoatLogicListener:` | PocketGodViewController | `isStoryActive` |
| `GoGoCageLogicListener` | `setGoGoCageLogicListener:` | PocketGodViewController | (none) |
| `GodIdolLogicListener` | `setGodIdolLogicListener:` | PocketGodViewController (4) | `isStoryActive`, `isGravityEnabled`, `isGodIdolEnabled`, `isIdolPedestalEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `idolPedestalLogicArray:`, `onGodIdolDrag:`, `onGodIdolDragComplete:` |
| `IdolBattleRealmEffectLogicListener` | `setIdolBattleRealmEffectLogicListener:` | IdolScreenLogic (9) | `onIdolBattleRealmEffectSelect:episodeNodeDef:` |
| `IdolIconLogicListener` | `setIdolIconLogicListener:` | IdolScreenLogic (9) | `onIdolIconSelect:` |
| `IdolPedestalLogicListener` | `setIdolPedestalLogicListener:` | PocketGodViewController (4) | `isStoryActive`, `optionPreferences`, `templeDoorLogic:` |
| `IdolScreenLogicListener` | `setIdolScreenLogicListener:` | IdolManager | `optionPreferences`, `onLoadLocationTextureAtlas`, `episodeAlertIcon`, `onEpisodeActivated:`, `onEpisodeIdolSelected:slot:`, `onEpisodeIdolDeselected:`, `selectedEpisodeIdolInSlot:`, `activatedHintCount`, `onHintActivated:`, `activeEpisodeNodeDef`, `onIdolAwardScreenChangeIslandLocationState:`, `onIdolAwardScreenOpenChallenges`, `onIdolAwardScreenClosed`, `onBattleIdolSelected:`, `onBattleEnemySelected:`, `onIdolBattleSelectionPanelOpened`, `playerBattleState`, `enemyBattleState`, `playerName`, `enemyName`, `playerHealth`, `enemyHealth`, `playerMaxHealth`, `enemyMaxHealth`, `onBattleIdolSelectedMultiplayer:`, `isMultiplayerActive`, `isItMyTurn`, `attackName:`, `realmName:`, `didPlayerWinLastBattle`, `didPlayerTieLastBattle`, `isLastBattleIdol:`, `fillPowerDescriptionText:structXmlEpisodeNodeDef:`, `activeBattlePlayerEpisodeIdol`, `activeBattleEnemyEpisodeIdol`, `enemyEpisodeIdol`, `nextBattleIdol:`, `calculateEnemyLevel:`, `calculateEnemyHealth:`, `calculatePlayerLevel`, `calculatePlayerHealth`, `showBattleRealmEffectResult:battleState:opponentBattleState:`, `checkBattleRealmEffectDeath`, `resetAllChallenges`, `onIdolScreenClosed`, `onIdolScreenLaunchStoreScreen`, `openMatchPanel` |
| `IdolSelectionIconLogicListener` | `setIdolSelectionIconLogicListener:` | IdolScreenLogic (8) | `onIdolSelectionIconSelect:episodeNodeDef:` |
| `JellyfishLogicListener` | `setJellyfishLogicListener:` | PocketGodViewController (5) | `pygmyLogicArray:`, `jellyfishLogicArray:`, `onJellyfishDrag:`, `onJellyfishDragComplete:` |
| `LocustQueenLogicListener` | `setLocustQueenLogicListener:` | PocketGodViewController | `isStoryActive`, `locustLogic:`, `onLocustQueenVisible:` |
| `MoronBeeLogicListener` | `setMoronBeeLogicListener:` | MoronPestsQuestionScreenLogic (5) | `onMoronBeeLogicSelected:` |
| `MoronDuckLogicListener` | `setMoronDuckLogicListener:` | MoronPestsQuestionScreenLogic (4) | `onMoronDuckLogicSelected:` |
| `MoronPestsQuestionScreenLogicListener` | `setMoronPestsQuestionScreenLogicListener:` | MoronSnakeLogic | `onMoronPestsQuestionCorrect`, `onMoronPestsQuestionIncorrect` |
| `MoronSnakeLogicListener` | `setMoronSnakeLogicListener:` | PocketGodViewController | `isStoryActive`, `isGravityEnabled`, `isMoronSnakeEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyLogicArray:`, `onMoronPestsStart:pygmyLogic:`, `onMoronPestsStartMovingCamera`, `onMoronPestsStopMovingCamera`, `onMoronPestsStop:`, `onMoronPestsScore:`, `onMoronSnakeDrag:`, `onMoronSnakeDragComplete:` |
| `MoronTurtleLogicListener` | `setMoronTurtleLogicListener:` | MoronSnakeLogic | `onMoronPestsStopMovingCamera`, `onMoronTurtleCatchPygmy` |
| `MultiModelLogicListener` | `setMultiModelLogicListener:` | NoPeeLogic | `onMultiModelEvent:eventId:` |
| `NoPeeClamLogicListener` | `setNoPeeClamLogicListener:` | NoPeeLogic (4) | (none) |
| `NoPeeFishLogicListener` | `setNoPeeFishLogicListener:` | NoPeeLogic (4) | (none) |
| `NoPeeLogicListener` | `setNoPeeLogicListener:` | PocketGodViewController | `pygmyLogicArray:`, `islandMinXPos`, `islandMaxXPos`, `onNoPeeStart:pygmyLogic:`, `onNoPeeStop:`, `onNoPeeRetry:`, `onNoPeeStartMovingCamera` |
| `NoPeeMapLogicListener` | `setNoPeeMapLogicListener:` | NoPeeLogic | `noPeeLogic`, `noPeePygmyLogic` |
| `NoPeeMapScreenLogicListener` | `setNoPeeMapScreenLogicListener:` | NoPeeLogic | `noPeeLogic` |
| `NoPeeMineLogicListener` | `setNoPeeMineLogicListener:` | NoPeeLogic (4) | (none) |
| `NoPeeParticleLogicListener` | `setNoPeeParticleLogicListener:` | NoPeeLogic (10) | (none) |
| `NoPeeSeaBlobLogicListener` | `setNoPeeSeaBlobLogicListener:` | NoPeeLogic (4) | (none) |
| `NoPeeTreasureLogicListener` | `setNoPeeTreasureLogicListener:` | NoPeeLogic | `onCollectNoPeeTreasure` |
| `PygmyBottomLogicListener` | `setPygmyBottomLogicListener:` | PocketGodViewController | `isOceanEnabled` |
| `TempleDoorLogicListener` | `setTempleDoorLogicListener:` | PocketGodViewController | `isStoryActive`, `isGravityEnabled`, `isTempleDoorEnabled`, `islandLocationState`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyLogicArray:`, `createPygmyLogic`, `onBattleOfTheGodsStart:pygmyLogic:`, `onBattleOfTheGodsStop:`, `onOpenIdolBattleEnemyScreen`, `onOpenIdolBattleScreen`, `onOpenIdolBattleResultsScreen` |
| `UnderwaterTempleLogicListener` | `setUnderwaterTempleLogicListener:` | PocketGodViewController | `onUnderwaterTempleModeChanged:` |
| `WaterWingsLogicListener` | `setWaterWingsLogicListener:` | PocketGodViewController | `isStoryActive`, `isGravityEnabled`, `areWaterWingsEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onWaterWingsDrag:`, `onWaterWingsDragComplete:` |
| `XenuLogicListener` | `setXenuLogicListener:` | PocketGodViewController | `startApocalypse` |
| `XenuTempleLogicListener` | `setXenuTempleLogicListener:` | PocketGodViewController | `isBabyPygmyActive`, `isTomPygmyActive` |

## Appendix B — debug logging (`BCLogItem`)

Debug only; a port can map it to `tracing`/`log` or drop it.

- `FUN_1001a8b20` @ 0x1001a8b20 (`BCLog(file, line, function, tag, format, ...)`): formats
  the message, creates `[[BCLogItem alloc] initWithLogTag:tag trace:message]`
  (@ 0x1001a89e8), appends it to a global `NSMutableArray` `DAT_1007e2378` (capacity 20,
  guarded by spin lock `DAT_1007e2370`) and prints `"%s: %s\n"` (tag, message).
- `FUN_1001a8ccc` @ 0x1001a8ccc: same without printing.
- `FUN_1001a8dec` @ 0x1001a8dec (tag set) / `FUN_1001a9010` @ 0x1001a9010 (single tag): print
  the stored items whose tag matches. `FUN_1001a9234` @ 0x1001a9234: clear the array.
- `BCLogItem::logTag` @ 0x1001a8b00 and `BCLogItem::trace` @ 0x1001a8b10 are getters (the
  decompiler shows `trace` as `void`; the disassembly is `ldr x0, [x0, #mTrace]; ret`).
- `+[BCLogic traceGlobalStatus]` @ 0x1001383d0 logs the live-logic counter `DAT_1007d1f14`.
