# PygmyLogic: the pygmy actor

Reverse-engineered specification of `PygmyLogic` (the islanders) with all eleven categories
(`Island`, `Apocalypse`, `Charlie`, `Underwater`, `BattleOfTheGods`, `KonkeyDong`, `NoPee`,
`TheRuns`, `Story`, `OogaJump`, `MoronPests`) and the helper classes `PygmyBottomLogic`,
`Persona`, `PygmyPersona`, `TribePersona` and `GodPersona`. Pocket God 1.48.2, arm64
binary `tmp/re/PocketGod.arm64`.

Companion documents: [logic.md](logic.md) (BCLogic / SpriteLogic framework, touch dispatch,
frame order), [sequences.md](sequences.md) (sequence engine, item return codes, the
parameter selection tables of §3.12, the per-item catalogue), [struct-xml.md](struct-xml.md)
(episode nodes used by Battle of the Gods), [audio-prefs-text.md](audio-prefs-text.md)
(preference keys, island location states).

Implementation partition: `tmp/re/pygmy-partition.json` (schema and summary in §29).

## 1. Sources, notation and conventions

Every behaviour is cited as `Class(Category)::selector @ address` (the IMP address). The
decompilation is `tmp/re/decomp/game/<file>.c`; constants and ambiguous ivar offsets were
checked against the disassembly. Values written `0.4`, `-277.12` are the exact `f32`
constants of the binary.

### 1.1 Objects and fields

| Notation | Meaning |
|---|---|
| `display`, `pos`, `xScale`, `rot` | the pygmy's `BCLogic::mDisplayObject`, its position (VECTOR4), x scale (facing: `+1` right, `-1` left; Konkey Dong and other mini-games use `±0.75`), rotation |
| `vel` | `BCLogic::mPosVel` (VECTOR4, ivar offset 52). Every write to it was classified from the disassembly (`str wzr` vs `str xzr` / `stp wzr,wzr`): `vel.x=0` means only x is cleared, `vel.xy=0` both |
| `listener` | `mPygmyLogicListener` (always the `PocketGodViewController`, §23) |
| `ignoreGrav` | `SpriteLogic::mbIgnoreGravity` (1 = gravity off) |
| `floorCol`, `sideCol` | `mbEnableIslandFloorCollision`, `mbEnableIslandSideCollision` |
| `sel0` / `sel1` | `[display setIsSelectable:NO/YES]` (touch dispatch skips unselectable objects) |
| `zrot0` | `[display setZRot:0]` (Ghidra drops the float argument; the disassembly loads `movi v0, #0`) |
| `layer(M)` | `[[listener M] insertAfter:display]` (the pygmy is moved into M's display group right after marker M, so it draws above M); `layer<(M)` uses `insertBefore:`. Markers: `pygmyDisplayMarker`, `pygmyDragDisplayMarker`, `pygmyIslandDisplayMarker`, `pygmyIslandBackDisplayMarker`, `pygmySmallObjectsDisplayMarker`, `pygmyUnderwaterPygmyDisplayMarker`, `pygmyCloudBehindDisplayMarker` |
| `X display insertAfter:` / `insertBefore:` | `[X.display insertAfter:pygmyDisplay]`: the pygmy is moved into X's group right after (before) X, i.e. drawn above (below) X |
| `+labels` | followed by `updateLabelLayer` (§3.3) |
| `→ Seq` | `setBehavior:@"Seq"` (replaces the running sequence, logic.md §3); inside a sequence item the handler then returns 3 |
| `sacrifice Id` | `mSacrificeId = @"Id"` (what the death will be recorded as, §8) |
| `gift Id` | `[listener onPygmyGiftGiven:self giftId:@"Id"]` |
| `episode E` | `[[IdolManager instance] triggerEpisodeEvent:@"E"]` |
| `P` | `mPersonaPersonality`: 0 normal, 1 Charlie, 2 Tom (§7) |
| `T` | `mTimeTemplePygmyType`: 0 normal, 1 old fart, 2 baby, 3 ginger, 4 fast (§7) |
| `D` | `SpriteLogic::mApocalypseDiseaseType`: 0 healthy, 1 plague, 2 purification (§7) |
| `L` | island location state (`[listener islandLocationState]`): 0 Sand, 1 Rock, 2 Underwater, 3 Graveyard, 4 Ice, 5 Ape, 6 Misfit, 7 God, 8 Boat, 9 Coral reef, 10 Apocalypse, 11 Time temple, 12 Locust temple, 13 Dance temple, 14 Plague temple, 15 Purification temple, 16 Closet temple, 17 Xenu temple |
| `getLogic(n)` | global logic object `n` of `BCLibrary` (logic.md); used: 0 the half-pygmy bottom, 2 dream bubble, 50 the Moron Pests turtle, 84 No Pee embed, 375/376 battle lava flames, 377..387 player battle objects, 388..398 enemy battle objects, 401 fail boat, 464..466 locust skulls, 467..469 locust bones |

### 1.2 Sequence item handlers

Signature `int item(self, _cmd, BCSequence *seq, BOOL firstTime, BCSequenceItemDef *def)`;
`p0, p1, ...` are `def->params[i]` (NSString or NSNumber, nil when absent). Return codes
(sequences.md §3.5): `0` stay on the item, `1` advance and stop this frame, `2` advance and
keep running, `3` = 0 after a `setBehavior:` replaced the sequence. `dt` is
`[seq deltaTime]`. Many items keep a timer in `BCLogic::mStallTime`.

Random helpers (all `arc4random()`, unsigned 32-bit `r`):

- `RAND100(lo, hi)` = `(float)(int)(r % (u32)((int)(hi*100) - (int)(lo*100) + 1) + (int)(lo*100)) / 100`
  (two-decimal uniform value; ARM `udiv` by 0 yields 0, so a zero-width range gives `r`
  itself in the modulo step: reproduce with `if m == 0 { r } else { r % m }`).
- `RAND10(lo, hi)`: the same with factor 10 (spawn x positions).
- `coin` = `r & 1`.
- Direction rule used by every timed walk (`walk`, `templePygmyWalk`, `zombieWalk` via
  `animChooseDirection`, `possessedByThetanWalk`, `halfPygmyWalk`, `charlieMonkeyWalk`,
  `swim`, `panicSwim`): go left (`xScale -1`, negative speed) if `x == islandMaxX`, or
  if `coin == 0 && x != islandMinX`; otherwise go right.

### 1.3 The busy set FREE and the stop lists

Almost every "can X happen to this pygmy" method is a long conjunction of cleared flags.
They all derive from one 83-condition base set, here called **FREE** (all must be clear):

`mbTouchBeganInside, mbRemoveFromStoryMode, mbCutInHalfByCharlie, T!=0,
mbBlownByHurricane, mbPickedUpByHurricane, mbDroppedByHurricane, mbHitByLightning,
mbDiedFromThetanBrawl, mbCatchShark, mbEatenByPiranha, mbSuckedIntoDrain, mbPainDrain,
mbEatenByTRex, mbEatenBySpider, mbHitBySnowBall, mbIglooRoll, mbSunFrozen, mbSunBurnt,
mbKidnappedByKonkeyDong, mbInKongStatue, mbInApeWench, mbInApeBongo, mbInApeAltar,
mbInGoGoCage, mbInDanceDanceExecution, mbRestingInIgloo, mbCrapping,
mbTrappedInSpiderWeb, mbTrappedInZombieGrave, mbTrappedInIdolPedestal,
mbInsideTempleDoor, mbInsideApocalypseTempleDoor, mbOnTopApocalypseTimeTemplePlatform,
mbEatenByLocustQueen, mbInApocalypseTempleFountain, mbThetanBrawl,
mbOnClosetTempleThetanMeter, mbOnClosetTempleCouch, mbInClosetTempleClosetDoor,
mbSlammingClosetTempleClosetDoor, mbHeldByXenu, mbHeldInTomsArms, mbHoldingBabyPygmy,
mbThrownByLocusts, mbKilledByLocusts, mbKilledByPurification, mbDanceDanceExecuted,
mbBattleOfTheGods, mbZombieExplode, mbIsZombie, mbIsGhost, mbTouchedByGhost,
mbPossessedByThetan, mbCaughtByMonkey, mbMonkeyOnBack, mbEatenByMoronSnake,
mbWearingWaterWings, mbHoldEarsFromBansheeScream, mbExplodeFromBansheeScream,
mbSkewered, mbUnderwater, mbExplodeFromNotCrapping, mbExplodeFromVampireSun,
mbExplodeFromVampireGinger, mbExplodeFromLovingPocketGod, mbDragging, mbCrushed,
mbCaughtByBird, mbFalling, mbInCrackHole, mbDoodleJumping, mbSwarmed,
mbBurntByMagnifyingGlass, mbTumble, mbOnFireFromLava, mbOnFireFromBonFire,
mbAteBurntFood, mbAteSwordFish, mbHitByIcicle, mbCooked, mbKilledByPlague, mbDrowning`.

Each guard is written `FREE −{…} +{…}` (conditions removed from / added to the set) plus
required-true flags and other terms; §25 lists all 76 guards. The order of the checks does
not matter (pure reads), so an implementation can keep FREE as one `is_busy()` predicate
plus per-guard exceptions.

Activities are cancelled with fixed lists of `stop*` calls, executed in order:

| Name | Calls (in order) |
|---|---|
| **S13** | `stopBeingHighOnNitrous, stopEating, stopCarrying, stopCarryingPlagueBody, stopFishing, stopSleeping, stopWarmingByBonFire, stopStaringAtRainbow, stopStaringAtKidnapping, stopWaitingToCrap, stopThrowingSpear, closeWordBalloon, stopCharlieMischief` |
| **S14** | S13 with `stopBeingEaten` inserted after `stopSleeping` |
| **S12** | S13 without `stopSleeping` (used by hits that have an "Asleep" variant) |
| **S12′** | S13 without `stopThrowingSpear` (T-Rex / spider / locust panic) |
| **S15** | S13 with `stopBeingTrappedInZombieGrave, stopBeingTrappedInIdolPedestal` before `closeWordBalloon` |
| **S27** | `dropIntoApocalypseTempleRoom` list: nitrous, eating, carrying, carryingPlague, fishing, sleeping, warming, rainbow, kidnapping, waitCrap, spear, trappedZombieGrave, trappedIdolPedestal, insideTempleDoor, insideApocalypseTempleDoor, onTopApocalypseTimeTemplePlatform, inApocalypseTempleFountain, thetanBrawl, onClosetTempleThetanMeter, onClosetTempleCouch, inClosetTempleClosetDoor, heldInTomsArms, holdingBabyPygmy, thrownByLocusts, playingCatchWithMonkey, closeWordBalloon, charlieMischief |
| **S28** | `waitUnderwater` list: S27 with `possessedByThetan` before `holdingBabyPygmy` |
| **S31** | infection list: sleeping, beingEaten, beingEatenByPiranha, holdingChum, nitrous, eating, carrying, carryingPlague, spear, trappedZombieGrave, trappedIdolPedestal, insideTempleDoor, insideApocalypseTempleDoor, onTopPlatform, inFountain, thetanBrawl, onThetanMeter, onCouch, inClosetDoor, heldInTomsArms, holdingBaby, eatenByLocustQueen, thrownByLocusts, warming, rainbow, kidnapping, fishing, waitCrap, playingCatchWithMonkey, closeWordBalloon, charlieMischief |
| **S34** | `removeFromStoryMode` list: S31 with `slammingClosetDoor, heldByXenu` after `inClosetDoor` and `possessedByThetan` after `heldInTomsArms` |
| **SDIE** | `die:::` list (§3.6): S34 with `closeWordBalloonNow` instead of `closeWordBalloon` |

Recurring blocks:

- **EJ** (eject and drop): `scale = (1,1)`, `falling=1`, `ignoreGrav=0`, `floorCol=1`,
  `sel1`, `sacrifice DroppedDrown`, `→ PygmyFall` (temple variants use `TemplePygmyFall`).
- **drag-end**: `mbDragging = 0`, `mbApplySpring = 0`, `[listener onPygmyDragComplete:self]`;
  the temple drop, the underwater transitions and the snowball also clear `mbOverFire`
  (except the snowball) and the label offsets; death, the snowball and bait master also
  `zrot0`.
- **doodle-end**: `mbDoodleOnIce=0, mbDoodleJumping=0, mbDoodleKnockedDown=0`,
  `[listener onPygmyStopDoodleJump:self]`.
- **hurricane-drop**: if `mPickedUpByHurricaneLogic` (SpriteLogic): `[it dropSpriteLogic:self]`,
  ref = nil (the hurricane forgets the pygmy without calling back `droppedByHurricane:`),
  some callers also clear `mbPickedUpByHurricane`.

## 2. Class map

`PygmyLogic : SpriteLogic : BCLogic` (SpriteLogic ivars end at offset 280, PygmyLogic adds
329 ivars from 280 to 1605). `PygmyBottomLogic : SpriteLogic`. `Persona : NSObject`,
`PygmyPersona`, `TribePersona`, `GodPersona : Persona`.

| Group | File | Methods | Code bytes | IMP range |
|---|---|---:|---:|---|
| PygmyLogic | PygmyLogic.c | 55 | 225375 | 0x1001286ec..0x100135964 |
| PygmyLogic(Island) | PygmyLogic+Island.c | 274 | 745345 | 0x100094b08..0x1000bf554 |
| PygmyLogic(Apocalypse) | PygmyLogic+Apocalypse.c | 142 | 251716 | 0x10003a3e4..0x100047c74 |
| PygmyLogic(Charlie) | PygmyLogic+Charlie.c | 28 | 81198 | 0x1000580f4..0x10005cabc |
| PygmyLogic(Underwater) | PygmyLogic+Underwater.c | 49 | 76697 | 0x10011bc64..0x10011fd50 |
| PygmyLogic(BattleOfTheGods) | PygmyLogic+BattleOfTheGods.c | 41 | 49813 | 0x100121c54..0x1001246e4 |
| PygmyLogic(KonkeyDong) | PygmyLogic+KonkeyDong.c | 9 | 25948 | 0x100039028..0x10003a250 |
| PygmyLogic(NoPee) | PygmyLogic+NoPee.c | 19 | 20505 | 0x10018a5a8..0x10018b6f0 |
| PygmyLogic(TheRuns) | PygmyLogic+TheRuns.c | 10 | 15854 | 0x100139998..0x10013a5a4 |
| PygmyLogic(Story) | PygmyLogic+Story.c | 18 | 14427 | 0x100174250..0x100174e74 |
| PygmyLogic(OogaJump) | PygmyLogic+OogaJump.c | 8 | 12560 | 0x100007bf8..0x1000085fc |
| PygmyLogic(MoronPests) | PygmyLogic+MoronPests.c | 12 | 7944 | 0x100113474..0x100113958 |
| PygmyBottomLogic | PygmyBottomLogic.c | 10 | 7977 | 0x100112c9c..0x100113300 |
| Persona | Persona.c | 21 | 23332 | 0x1001a44d8..0x1001a5924 |
| TribePersona | TribePersona.c | 11 | 7596 | 0x1001a5b64..0x1001a61b4 |
| GodPersona | GodPersona.c | 9 | 6723 | 0x1001a646c..0x1001a6a80 |
| PygmyPersona | PygmyPersona.c | 9 | 3117 | 0x1001a6278..0x1001a63fc |

725 methods in total, 194 of them sequence items (`:::`) and 42 animation frame
actions (`anim*`, §27). The code is large because nearly every entry point inlines the
83-flag busy test and a 13..34-call stop list; the logic itself is small per method.
`FUN_100047ee4` / `FUN_100048000`, which Ghidra placed in the Apocalypse file, are
`StoryManager` zlib helpers (compressBlob / uncompressBlob), not pygmy code.

What each category adds:

| Category | Adds |
|---|---|
| (main) | construction, labels, persona binding, `onGameFrame:` physics, tilt and earthquake, touch/drag/throw/flick, the generic movement items (idle, walk, run, panicRun, slide, fall, tumble, drag, hangUpsideDown), death and ghost conversion, explode, dialog throttle, type/personality selection items |
| Island | every island interaction: food, wood, fishing (pole and ice hole), out-house, bonfire, sleeping and dream bubble, dancing and the go-go cage refresh, rainbow, monkeys, sharks, birds, Konkey Dong kidnapping, all island hazards (lightning, explosions, lava, icicles, snowballs, boulders, T-Rex and eggs, spiders and webs, magnifying glass, ants, crack hole, hurricane, vampires, sun burn/freeze, ice monster squid), the catch-and-eject containers (Kong statue, apes, igloo, zombie grave, idol pedestal, temple doors, ice hole), ghosts (player-drawn flight paths and their victims), zombies, vampires, water wings, Moron snake |
| Apocalypse | the temple-room pygmies: time-temple types (old fart, baby, ginger, fast), disease incubation and infection, plague bodies (carry, burn), locust attacks, dance balloons, thetan possession and brawls, the closet temple with Tom, the baby/Tom/Xenu sacrifice, temple fountains, platforms, locust queen, go-go cage and Dance Dance Execution, dropping into a temple room |
| Charlie | Charlie Sheen (P=1): tiger-blood cutting in half, half pygmies that walk and absorb others, the warlock lightning/burn, monkey on Charlie's back, smacking boulders and eggs |
| Underwater | the underwater locations (L 2 and 9): spawning and sinking, swimming, whirlpools, chain spears, laser shark, jellyfish, piranhas and bait master, chum, the drain and the pain drain mini-game, underwater powers |
| Story | story mode: word balloons, scripted character states, story deaths |
| BattleOfTheGods | the pygmy fighters of the idol battle (spawn, attacks by realm and style, ties, fatalities, energy balls, beams) |
| KonkeyDong, TheRuns, NoPee, MoronPests, OogaJump | the five mini-games played with one pygmy (§17..§20, §15) |

## 3. Lifecycle

### 3.1 Creation (controller)

`PocketGodViewController(Game)::createPygmyLogic @ 0x10015ba88`:

1. `BCMultiModel initWithId:@"NewPygmy"`; `setIsSelectable:YES`; `originX = -32`;
   `x = RAND10(islandMinX + 15, islandMaxX - 15)`; `y = 340`; `z = -277.12`; collision box
   `colX 0..64`, `colY 0..64`.
2. Display parent: right after marker `PygmyDisplayMarker`, or for L 11..17 after
   `TimeTemplePygmyDisplayMarker`, `LocustTemplePygmyDisplayMarker`,
   `DanceTemplePygmyDisplayMarker`, `PlagueTemplePygmyDisplayMarker`,
   `PurificationTemplePygmyDisplayMarker`, `ClosetTemplePygmyDisplayMarker`,
   `XenuTemplePygmyDisplayMarker`.
3. `PygmyLogic initWithDisplayObject:` (§3.2), then `setPygmyLogicListener:` with the
   controller (`setPygmyLogicListener: @ 0x100128d04` retains it).
4. If `BCView.rawAccelRotDegrees.z` is strictly inside (-90, 90) (device upside down):
   `vel.y = -1200`.
5. If L is 2 or 9 and no location transition is running: `y = (r % 1401 + 2000) / 10`
   (200.0..340.0), `coin == 0` → `xScale = -1`, then `addedUnderwater` (§14).
6. If the controller's bait master is active: `startBaitMaster`.
7. Appended to `mLogicArray` and `mPygmyLogicArray`; `mPygmyCount++`, `mLivePygmyCount++`.

Callers:

- `addPygmy @ 0x10015bfa4`: limit = 6 (7 when the `CharlieState` option is 1) plus 1 when
  Tom is alive (`isTomPygmyAlive` = `mTomPygmyLogic != nil`); if `mLivePygmyCount` is below
  the limit: `createPygmyLogic` then `[tribe resurrectPygmyLogic:p]` (binds the next dead
  persona, §22). Sequence item `addPygmy::: @ 0x10015c064` wraps it (returns 2).
- `addBabyPygmy::: @ 0x10015c088`: create, resurrect, `setTimeTemplePygmyType:2` (no
  limit check).
- `addTomPygmy:::` → `createTomPygmy @ 0x10015b9d8`: `[tribe activateGuestPygmyPersona:tom]`,
  create, remember as `mTomPygmyLogic`, resurrect (picks Tom's persona, which activation
  inserted at index 0 of the dead list), `[tribe deactivateGuestPygmyPersona:tom]`.
- `ApocalypseTimeTemplePlatformLogic` turns existing pygmies into types 1..4 via
  `setTimeTemplePygmyType:` (sequences.md §3.12).

### 3.2 `initWithDisplayObject: @ 0x1001286ec`

1. `SpriteLogic` init (gravity force (0,-3000,0,1), `ignoreGrav = 1`, game frame started).
2. Once per process: the global dialog dictionary (`NSMutableDictionary` capacity 5, used
   by `checkPygmyDialog`, §9).
3. `mTiltState = 0`, `floorCol = 1`, `mSlideFriction = (r % 401) / 10000 + 0.03`
   (0.03..0.07), `mSpringForce = (0,0,0,1)`.
4. Name background `BCRect` id `PygmyText`, rect (-50,0)..(50,14), colour rgba
   (0,0,0,0.5); name text `BCText` id `PygmyText`, font `Arial12`, justification 2.
5. `mIslandYPos = 86`, `mbFalling = 1`, `mbInvalidSacrifice = 1`, `→ PygmyFall`.

Every new pygmy therefore drops in from y 340 with an invalid sacrifice (a pygmy that
falls into the sea on arrival does not count as a sacrifice, §8.1). Landing clears
`mbInvalidSacrifice` (§4.4).

`.cxx_construct @ 0x100135964` is empty. `isSoundDisabled @ 0x100128940` returns
`mbInGoGoCage`; `applyMoveXPos:moveYPos: @ 0x100128950` forwards to BCLogic unless in the
go-go cage (animation-driven moves are ignored while caged). `gameFrameOrder @ 0x100129978`
= 30.

### 3.3 Persona binding and the name label

- `setPygmyPersona: @ 0x100128bb0`: retains; `P = [persona personaPersonality]`; P != 0
  tints the label background rgba (0.4,0,0,0.5).
- `setPygmyNameLabel: @ 0x1001289ac`: copies the name to `mName`; label text = name, or
  `OldFart%@`, `Baby%@`, `Ginger%@`, `Fast%@` for T 1..4 (format with the name); background
  rect x `-1 - w/2 .. w/2 + 1`, y `-14 .. 0` (w = `textWidth`); `mNameTextYOffset = -13`.
  `PygmyPersona::setName:` and `setPygmyLogic:` call it, and so does
  `PygmyLogic(Apocalypse)::setTimeTemplePygmyType: @ 0x10003ca9c` with `mName`.
- Getters `pygmyPersona @ 0x100128c6c`, `personaPersonality @ 0x100128c7c`,
  `pygmyNameLabel @ 0x10012899c` (the label text object).
- `translateAndMoveToDisplayMarker: @ 0x100129894` (marker): converts the position into the
  view through the current group (`displayObjectToView:`) and back into the marker's group
  (`viewToDisplayObject:`), sets x and y, `[marker insertAfter:display]`,
  `updateLabelLayer` (used for the fail boat and the drag layer).
- `updateLabelLayer @ 0x100128ed4`: if `[listener areNameLabelsEnabled]`: remove text and
  background and re-insert them right after the pygmy (`[display insertAfter:text]`, then
  `[display insertAfter:bg]`, so the order is pygmy, background, text). With an
  apocalypse cloud (SpriteLogic) the cloud display is re-inserted right after the pygmy.
  Called after every layer change.
- `moveNameLabel @ 0x10012b080` (every frame): background pos = pygmy pos, or the carrying
  pygmy's pos + (0, 15) while `mbPlagueBodyPickedUp`; plus `(mDragLabelXOffset,
  mDragLabelYOffset)`; text = background + `(mNameTextXOffset, mNameTextYOffset)`.
- Visibility (end of every `onGameFrame:`): if `!mbDisableLabels`, names are enabled and the
  text is not in a display group → `updateLabelLayer`; else if (`mbDisableLabels` or names
  disabled) and the text is in a group → remove background and text. `mDragLabelYOffset`
  is `-64` while dragged, `-20` for ghosts and No Pee, 0 otherwise; the apocalypse cloud
  gets `setDragXOffset:dragYOffset:` with the same offsets.

### 3.4 Per frame

`onGameFrame:` (order 30, after sequences 10 and animations 20) runs the physics of §4
unless `mbShutdown`. The running sequence (driven by the controller's sequence pass)
moves the pygmy between behaviours; touches arrive through the touch dispatcher
(logic.md §5) and §5.

### 3.5 States in practice

There is no state enum: the "state" is the running sequence plus about 150 BOOL flags.
The flags that define exclusive situations are exactly the FREE set (§1.3). A pygmy is
"free" when none is set; most interactions (`canBeX:` / `X:` pairs called by other logic
objects) test FREE first, so at most one exclusive situation is active. §6 lists the
sequences.

### 3.6 Death: `die::: @ 0x10012ecb8`

Item of `PygmyDie` (and many death sequences). In order:

1. `mbTumble` → `[listener onPygmyTumbleComplete:self]`; `mbEarthquake` →
   `[listener onPygmyEarthquakeStop:self]`; `mbKonkeyDong` →
   `[[listener kongStatueLogic:self] stopKonkeyDong]`.
2. `SpriteLogic::cureDisease`, `stopWarmingByBonFire`.
3. Release every container, each followed by clearing the reference: waiting out-house
   `stopWaitingToCrap:`, zombie grave `ejectFromGrave:`, idol pedestal
   `ejectFromIdolPedestal:`, temple door `ejectFromTempleDoor:`, apocalypse temple door
   `ejectFromApocalypseTempleDoor:`, time platform `ejectFromApocalypseTimeTemplePlatform:`,
   locust queen `rescuedFromLocustQueen:`, fountain `ejectFromApocalypseTempleFountain:`,
   thetan `stopPossessingPygmy:`, brawl attacker (`clearTweenableChannelIdNow:16` first)
   and defender `stopThetanBrawlByPygmy:`, thetan meter, couch, closet door `ejectFrom…:`,
   Xenu `ejectFromXenu:`, Tom `ejectFromTomsArms:`, crapping out-house `stopCrapping:`,
   magnifying glass `onPygmyDie:`, ash pile (remove, release, shutdown), fishing pole
   (`clearTweenableChannelIdNow:` 2, 3, 4, `[pole die]`), ghost `cancelGhostHold:`, ghost
   victim `cancelGhostTouchDead:`, jellyfish `cancelJellyfishAttack:`, hurricane
   `dropSpriteLogic:`, spider web `[[listener spiderTreeLogic:self] ejectFromWeb:self]`,
   drain `[[listener tileDrainLogic:self] escapeFromDrain:self]`, Kong statue, ape wench,
   ape bongo, ape altar, go-go cage, Dance Dance Execution, igloo `removePygmyFromX:`,
   ice hole `removePygmyFromIceHole:` (reference kept), carrying bird `dropPygmyLogic:`.
4. SDIE stop list (§1.3), then `[[listener sharkLogic:self] cantCatchPygmy:self]`.
5. Sacrifice resolution:
   - if `mbInvalidSacrifice`: when `mSacrificeId` is set, `[listener onPygmySacrificeEvent:id]`
     (the episode event still fires), then `mSacrificeId = nil`;
   - overrides, first match: doodle jumping → `Flattened`; zombie → `ZombieExplode`
     (`ZombieExplodeBrainsEaten` when `mbBrainsEaten`); cut in half → `CutInHalfByCharlie`
     (`XenuPuppetShow` when held by Xenu); `mbHadHeartAttack` → `OldFartPygmyHeartAttack`;
   - if an override applied or the id is still set: `[listener onPygmySacrificeEvent:id]`.
6. If dragging: `mbDragging = 0`, `mbApplySpring = 0`, `zrot0`,
   `[listener onPygmyDragComplete:self]`. Then `mbTouchBeganInside = 0`, `mActiveTouch = nil`.
7. **Ghost conversion** when all hold: not a ghost, not story mode, not carried away by a
   ghost, `!mbInvalidSacrifice`, not skewered, not caught by a bird, not removed from story
   mode, not in The Runs / Konkey Dong / No Pee, and `[listener areGhostsEnabled]`:
   `mbIsGhost = 1`, `mbDisableLabels = 0`, floor/side/slide collision off, `falling = 0`,
   `ignoreGrav = 1`, locust death flags cleared (`mbKilledByLocusts`,
   `mbMouthFilledByLocusts`, `mbMutatedByLocusts`), label offset (0, -20), `vel.xy = 0`,
   `z = -277.12`, `zrot0`, alpha 1, `sel1`, allocate the ghost path
   (`std::vector<VECTOR4>`), `layer<(pygmyDragDisplayMarker)` +labels,
   `gift BecomeGhost`, `→ PygmyGhostInit`; returns 3. The pygmy stays alive as a ghost
   (§10.9) and keeps `mSacrificeId`; the sacrifice is recorded when the ghost itself dies.
8. Otherwise (real death): if the banshee scream had started, stop it (every pygmy
   `stopHoldEarsFromBansheeScream:self`, `→ PygmyGhostStopScream`); remove the name label;
   `[listener onPygmyDie:self sacrificeId:mSacrificeId]` (§8, §23); if
   `mbTellClosetToGrabNewTom`: `[[listener closetTempleClosetDoorLogic:self] grabNewTom]`;
   returns 2.

`onPygmyDie:sacrificeId:` (controller, §23) records the sacrifice, decrements
`mLivePygmyCount`, removes the display object, shuts the logic down and drops it from the
logic arrays.

A ghost's own end: `onMultiModelEvent:eventId: @ 0x100132e94` receives `TheLightExit`
from the light it flew into, releases the light and, if `mbGoIntoTheLight`, sets
`→ PygmyDie` and `gift IntoTheLight`. That second `die:::` runs with `mbIsGhost` set, so it
takes the real-death branch and reports the original `mSacrificeId` (the episode event
of step 5 therefore fires once at death and once when the ghost leaves).

`explode::: @ 0x10012fcc0` (item used by the explosion deaths): `[listener onPygmyExplode:self]`
(the controller throws every pygmy away from this one, §23); if struck by lightning
and not held by a hurricane, Konkey Dong, the Kong statue, an ape, the go-go cage or
the DDE: create an `AshPile` multimodel with a `MultiModelLogic` (`→ AshPileInit`), place it
before `pygmyDisplayMarker` at the pygmy's position and origin; leave the out-house queue
(`ignoreGrav = 1`, `stopWaitingToCrap:`); tell a burning magnifying glass `onPygmyDie:`;
returns 2.

`dealloc @ 0x100128d60`: deletes the ghost path vector, removes/releases/shuts down the ash
pile, removes and releases both label objects, releases name, persona and listener.

### 3.7 Location changes

`onDeactivateIslandLocationState:newIslandLocationState: @ 0x10012e420` (old, new), called
for every pygmy when the island location changes:

1. Carrying a plague body → `stopCarryingPlagueBody`, `→ PygmyIdle`. Held in Tom's arms →
   `stopBeingHeldInTomsArms`, `→ PygmyIdle`. Holding a baby → `stopHoldingBabyPygmy`,
   `→ PygmyIdle`.
2. Sacrificed to Xenu: nothing more.
3. Between the two underwater locations (2 ↔ 9): only if underwater and not eaten by
   piranhas, skewered, hit by a jellyfish / power / laser, or eaten by the laser shark:
   `stopHoldingChum`, `stopBeingEatenByPiranha`, `sel1`, alpha 1, `zrot0`, and unless
   picked up by a whirlpool `→ PygmyUnderwaterIdle`.
4. New location 5 (Ape island) while drowning: a plague body that is not disappearing
   `→ PygmyPlagueBodyFallOffWorld`.
5. Otherwise: eaten by the Moron snake → `[[listener moronSnakeLogic:self] eatPygmyComplete:self]`
   then `→ PygmyDie`; wearing water wings → `[[listener waterWingsLogic:self] wearingWaterWingsComplete:self]`
   then `→ PygmyDie`; monkey on back or swarmed → `→ PygmyDie`; eaten by the locust
   queen → `rescuedFromLocustQueen:` (ref cleared) then `→ PygmyDie`; high on nitrous →
   clear, stop the sounds `DavidIDidntFeel, DavidIsThisRealLife, DavidTwoFingers,
   DavidIFeelFunny, DavidIsThisGonnaBeForever, DavidScream`, `→ PygmyIdle`; new location 5
   while falling and not knocked down in Ooga Jump → `→ PygmyFall`.

Entering a temple room uses `dropIntoApocalypseTempleRoom` (§11.2), the underwater
locations use `startUnderwater` / `waitUnderwater` / `stopUnderwater` (§14.2), and
`moveToApocalypseTempleDisplayGroup` / `moveFromApocalypseTempleDisplayGroup` re-layer
every pygmy (`layer(pygmyDisplayMarker)` +labels, except Tom while inside the closet).

## 4. Physics: `onGameFrame: @ 0x100129980`

Screen space is the 480x320 landscape view; the island floor is `mIslandYPos` (86; 55
during a battle), the ocean surface is y 16 (§9.6), world z of the island plane is
-277.12. `dt` is the frame time passed by the logic loop.

### 4.1 Step by step

0. `mbShutdown` → return. `mbNoPee` → `handleNoPeeMovement:dt` (§15) and return.
1. Gravity magnitude `g`: if underwater and neither dragged nor in the pain drain: 30.
   Otherwise 0 with the Ooga Jump jet pack, 1500 while doodle jumping or thrown by
   locusts, else 3000.
2. Gravity direction. If `isAccelEnabled` (§4.3) is NO or `[listener isEruptionActive]`:
   `mGravityForce = (0, -g, 0, 1)` and `undoTiltState` (§4.4). Otherwise
   `a = BCView.accelForce` (smoothed), `mGravityForce = normalize(-a.y, a.x, 0) * g` (w 1)
   and `refreshTiltState`.
3. `checkForEarthquake:dt` (§4.5).
4. `mGravityForce.x *= mSlideFriction` in tilt states 3 and 4, else `mGravityForce.x = 0`
   (gravity only ever pulls sideways while sliding).
5. `mPosForce = (0,0,0,1)`; `+= mGravityForce` unless `ignoreGrav`; `+= mSpringForce` when
   `mbApplySpring` (drag, §5.2).
6. Doodle jumping and not knocked down, or in the pain drain and not spiked:
   `vel.x = rawAccelForce.y * -1000` (direct tilt steering).
7. Konkey Dong (playing, started, no level transition, not hit) (§17).
8. **Integration (semi-implicit Euler)**: `vel.xyz += mPosForce.xyz * dt`, then
   `pos.xyz += vel.xyz * dt`.
9. Cooking: while `mbOverFire` (set by the drag item when dangled over a lit bonfire):
   `mCookTime += dt`; when it exceeds 2.0 and the pygmy is healthy (`!cooked, !plague,
   !cut, T==0`): `mbCooked = 1`, `sacrifice Cooked`, `→ PygmyCook`. `mCookTime` is never
   reset.
10. The Runs (§18): acceleration `+50/s` up to 600 after the burn, lava at y <= 13.
11. Moron Pests (§19): pace from the turtle (`getLogic(50)`, the `MoronTurtleLogic`).
12. Underwater and not in the pain drain: keep `minPos.y >= 0`; before `addUnderwaterComplete`
    also keep `maxPos.y <= 320`.
13. Ghost (§10.9): unless carrying a pygmy away (`mGhostTouchType == 1`) clamp
    `y ∈ [40, 280]`, `x ∈ [20, 440]`; ease `rot.z` toward `mGhostTargetZRot`: if the
    difference exceeds 180, wrap (rot < 0 → rot += 360; rot > 180 → rot -= 360; else a
    negative target += 360), then `rot.z += (target - rot.z) * 0.4` (per frame, not per
    second).
14. Island sides (`islandMinX/MaxX` from the listener; ±160 while on the fail boat):
    `mbIslandSideCollision = 0`; if side collision is enabled and the pygmy crossed a side
    this frame (`lastPos.x >= min && x < min`, or `lastPos.x <= max && x > max`):
    `vel.x = 0`, `x = side`, `mbIslandSideCollision = 1`.
15. Screen sides: `mbScreenSideCollision = 0`; unless caught by a bird, in The Runs, on the
    fail boat, in Konkey Dong, Moron Pests, No Pee, a battle, carried away by a ghost,
    mutated by locusts, held by Xenu, a drowning plague body, or a ghost carrying a victim:
    `minPos.x < 0` → shift right, `maxPos.x > 480` → shift left, and set
    `mbScreenSideCollision`.
16. Floor (`mbIslandFloorCollision = 0` first):
    - with floor collision enabled and L != 8: if `islandMinX <= x <= islandMaxX` and
      `lastPos.y >= islandY && y < islandY` → **landing**;
    - L == 8 (fail boat) and not yet on the boat: boat = `getLogic(401)`, `θ` = boat rot.z
      in radians; if `boat.x - 160 cos θ <= x <= boat.x + 160 cos θ`, the floor is
      `boat.y + (x - boat.x) tan θ` and the same crossing test applies; once on the boat
      the landing branch runs every frame;
    - **landing**: healthy pygmy: first contact sets `mbOnIsland`, `mbIslandFloorCollision`,
      clears `mbInvalidSacrifice`; `vel.y = 0`. Cooked or plague body: first contact
      `vel *= (0.4, -0.4, 0.4)` (bounce) otherwise `vel.y = 0`; if `vel.y >= 100` →
      `PygmyCookedBounce` / `PygmyPlagueBodyBounce`; else `vel.y = 0`, on first contact
      `falling = 0`, `mbOnIsland = 1`, `vel.x = 0`, `mbIslandFloorCollision = 1`,
      `mbInvalidSacrifice = 0`, then `zrot0`. Then on L 8: first time `mbOnFailBoat = 1` and
      `translateAndMoveToDisplayMarker:FailBoatPygmyDisplayMarker`, `y = 0` (boat-local);
      else `y = islandY`. Always `stopPlayingCatchWithMonkey`;
    - **no floor** (floor collision off, outside the island span or no crossing): if
      `mTiltState != 5`: `mbOnIsland = 0`, `mbOnFailBoat = 0`, and start a fall unless
      already falling, dragged, killed by locusts or purification, DDE-executed, cooked,
      plague, cut, ignoring gravity, underwater, in The Runs, a battle, Konkey Dong, the
      closet door, Moron Pests or No Pee: sun-frozen → `iceShatter`; `falling = 1`,
      `→ PygmyFall` (`TemplePygmyFall` for T != 0). In tilt state 5 (hanging upside down):
      `mbOnIsland = 1`, `vel.y = 0`, `y = islandY`.
    Note that a normal landing does not clear `mbFalling`; the running `fall:::` item does
    (§9.6).
17. Out of the world: `y < -100`, or `y > 820` unless doodle jumping, Konkey Dong, No Pee
    or tumbling: stop list (`stopBeingHighOnNitrous, stopEating, stopCarrying,
    stopCarryingPlagueBody, stopSleeping, stopFishing, stopWarmingByBonFire,
    stopStaringAtRainbow, stopStaringAtKidnapping, stopWaitingToCrap, stopThrowingSpear,
    closeWordBalloon, stopCharlieMischief`); `mbDieInStory` → `→ PygmyStoryModeDie`; else
    (Konkey Dong and not hit → `sacrifice KonkeyDongFall`) a plague body that is not
    disappearing → `PygmyPlagueBodyFallOffWorld`, otherwise `→ PygmyDie`. This runs on
    every frame the pygmy stays out of bounds, restarting the sequence each frame until
    `die:::` removes it (keep: the death sequence's first item runs immediately).
18. Ash pile alpha follows the pygmy's alpha; `checkPygmyCollision` (§4.2); then, each
    with `&mLastPos`: `checkDoodleCloudCollision:`, `checkPainDrainCollision:`,
    `checkTheRunsCollision:`, `checkKonkeyDongCollision:`, `checkMoronPestsCollision:`.
19. Word balloon follows `pos`; label visibility (§3.3); apocalypse cloud drag offsets;
    `moveNameLabel`; `mLastPos = pos`.

### 4.2 Pygmy-pygmy collision: `checkPygmyCollision @ 0x100128fe8`

Only while dragging a cooked pygmy: `speed = |vel.xyz|`; for every other pygmy, if
`speed > 200` and `(x - o.x)^2 + (y - (o.y + 46.5))^2 < 1600` and
`[o hitByCookedPygmy:self]` returns YES → stop at the first hit. (Throwing a cooked pygmy
at another knocks it over, §10.3.)

### 4.3 `isAccelEnabled @ 0x10012924c`

NO if any of `FREE −{mbTouchBeganInside, mbCutInHalfByCharlie, T!=0, mbIsZombie,
mbDragging, mbFalling, mbCooked, mbKilledByPlague, mbDrowning} +{mbSharkOnHead}` is set, or
`mbEarthquake && !mbFalling`, or `![listener isGravityEnabled]`; otherwise
`SpriteLogic::isAccelEnabled` (NO when the device lies flat or no accelerometer data,
logic.md). So zombies, half pygmies, temple types, cooked and plague bodies do slide
with tilt, while the busy states do not.

`PocketGodViewController::isGravityEnabled @ 0x1000871d8`: option `GravityHazard == 1`,
L not 2 or 9, no doodle-jumping pygmy, no The Runs / Konkey Dong / Moron Pests / No Pee /
Dance Dance Execution, and no screen open.

### 4.4 Tilt states

`refreshTiltState @ 0x10012be50` classifies `z = BCView.rawAccelRotDegrees.z`
(`atan2(raw.y, raw.x)` in degrees; landscape upright is ±180):

| z | state | meaning |
|---|---|---|
| \|z\| > 155 | 0 | level |
| (130, 155] | 2 | tipping (one way) |
| [-155, -130) | 1 | tipping (other way) |
| (90, 130] | 4 | sliding |
| [-130, -90) | 3 | sliding |
| [-90, 90] | 5 | upside down |

Early return (no change) when any of `FREE −{mbCutInHalfByCharlie, T!=0,
mbDroppedByHurricane, mbIsZombie, mbTumble, mbCooked, mbKilledByPlague} +{mbSharkOnHead,
mbEarthquake}` is set, or when the pygmy is neither on the island nor holding a fishing
pole (`!mbOnIsland && mFishingPoleLogic == nil`). Half pygmies, temple types, zombies,
cooked and plague bodies therefore do tilt (with their own animations).

On a change of state (`new != mTiltState`; leaving state 5 first does `zrot0`), with
`Tx = T ∈ {1,2}` (old fart and baby use temple animations; ginger and fast behave like
normal pygmies here):

- **→ 0**: `mbEnableUpsideDown = 0`. Zombie → `vel.x = 0`, `PygmyZombieNoTiltSlide`.
  Cooked / plague / cut / Tx → `vel.x = 0`, no behaviour. Sleeping → `vel.x = 0`,
  `PygmyAsleep`. Otherwise by the old state, after S13 and `vel.x = 0`: 1 or 2 →
  `PygmyGainBalanceAfterTilt`; 3 → `PygmyPickupBackward` if `xScale < 0` else
  `PygmyPickupForward`; 4 → `PygmyPickupBackward` if `xScale > 0` else forward; 5 →
  `zrot0`, `PygmyPickupForward`.
- **→ 1** (only from 0, awake, healthy, not a zombie, not Tx): S13, `vel.x = 0`,
  `PygmyTiltTipForward` if `xScale >= 0` else `PygmyTiltTipBackward`.
- **→ 2** (same filter): S13, `vel.x = 0`, `PygmyTiltTipForward` if `xScale <= 0` else
  backward.
- For 1 and 2 the new state is stored only when the tip behaviour was started; otherwise
  `mTiltState` keeps its old value and the test repeats next frame.
- **→ 3**: zombie → `PygmyZombieTiltSlide`; cooked → `PygmyCookedTiltSlide`; plague →
  `PygmyPlagueBodyTiltSlide`; cut → `HalfPygmyTiltSlide`; Tx → `TemplePygmyTiltSlide`;
  sleeping → `stopMonkeyHumping`, `PygmyTiltSlideSleep`; from state 5 → `stopSleeping`,
  `vel.x = 0`, if `xScale > 0` then `x += 40` and `xScale = -1`, `zrot0`,
  `PygmyTiltUpsideDownToFallForward`; else S13, `vel.x = 0`, `xScale < 0` →
  `PygmyTiltSlideBackward`, otherwise `mbEnableUpsideDown = 1`, `PygmyTiltSlideForward`.
- **→ 4**: mirror of 3 (from 5: if `xScale < 0` then `x -= 40` and `xScale = 1`;
  `xScale > 0` → backward, otherwise grip + forward).
- **→ 5**: if `mbEnableUpsideDown` and T == 0: S13, `vel.x = 0`, `mTiltState = 5`,
  `PygmyHangUpsideDown`; otherwise the state is not stored.

States 0, 3 and 4 are always stored. Then, on every call (changed or not): if
`BCView.accelRotDegrees.z` (smoothed) is strictly inside (-90, 90) and the pygmy did not
grip (`!mbEnableUpsideDown`): S13, `falling = 1`, and zombie → `PygmyZombieFall`, cooked →
`PygmyCookedFall`, plague → `PygmyPlagueBodyFall`, cut → `HalfPygmyFall`, T ∈ {0,3,4} →
`mbInvalidSacrifice = 1`, `sacrifice FallUpsideDown`, and T == 0 → `PygmyFallFromBack` in
tilt states 3/4 else `PygmyFallFar`; T ∈ {1,2} or the ginger/fast case → `TemplePygmyFall`.

Only a pygmy that slid forward (grip) can hang; `hangUpsideDown::: @ 0x10012eab4` then
keeps `vel.y = 0`, `rot.z = rawAccelRotDegrees.z`, `y = islandY` every frame (never
completes). Touching a hanging pygmy makes it lose its grip (§5.1).

`undoTiltState @ 0x10012d8c4` (gravity disabled or eruption): same early-return list;
`mbEnableUpsideDown = 0`; zombie / cooked / plague / cut / Tx → `vel.x = 0` if tilted, no
behaviour; else by the old state with S13 and `vel.x = 0`: 1, 2 →
`PygmyGainBalanceAfterTilt`; 3, 4 → pickup forward/backward as above; 5 → `zrot0`,
`PygmyPickupForward`; then `mTiltState = 0`.

`slide::: @ 0x100130b94` (the item of the slide sequences): first time a healthy normal
pygmy gets `sacrifice SlideDrown`, side collision off, floor on, gravity on. Each frame:
screen side reached → zombie `zombieExplode` (3), else (`SlideDrown` for healthy normal)
`→ PygmyFallIgnoreIsland` (3). During an eruption a healthy normal non-zombie in tilt state
4 (3) does `stopSleeping`, `vel.x = 0`, `PygmyPickupBackward` when `xScale > 0`
(`< 0`), else forward. Below the ocean surface (`y < 16`, ocean enabled): `stopSleeping`,
`mTiltState = 0`, `ignoreGrav = 1`, `mbDrowning = 1`, `vel.xy = 0`, then cooked →
`PygmyCookedSplash`, plague → `PygmyPlagueBodySplash`, cut → `HalfPygmySplash`, T != 0 →
`TemplePygmySplash`, else `sacrifice SlideDrown`, `PygmySplash` (3). Otherwise 0.

### 4.5 Earthquake (device shaking): `checkForEarthquake: @ 0x10012b1d0`

- If any `rawAccelForce` component is exactly 0 or gravity is disabled: if an earthquake
  is active, S13, `mbEarthquake = 0`, `[listener onPygmyEarthquakeStop:self]`, and unless
  in The Runs, Konkey Dong, Moron Pests or No Pee `→ PygmyGainBalance`. Return.
- Busy (`FREE −{mbIsZombie} +{mbSharkOnHead}`, includes T != 0): `mShakeCount = 0`, return.
- `f = totalRawAccelForce`, `mLastShakeTime += dt`. `f > 1.1` and not `mbShakeUp` →
  count++, `mbShakeUp = 1`, `mLastShakeTime = 0`; `f < 0.9` and `mbShakeUp` → count++,
  `mbShakeUp = 0`, `mLastShakeTime = 0`.
- `mLastShakeTime > 0.5`: `mShakeCount = 0`; an active earthquake ends (S13,
  `mbEarthquake = 0`, `onPygmyEarthquakeStop:`, `→ PygmyGainBalance`).
- `mShakeCount >= 5`: zombie → `zombieExplode`, then `onPygmyEarthquakeStart:` and
  `onPygmyEarthquakeStop:` back to back. Otherwise, if no earthquake yet: `mbEarthquake = 1`;
  in tilt state 5 (hanging): `falling = 1`, `zrot0`, `x += (xScale > 0 ? -32 : 32)`,
  `y += 32`, `mTiltState = 0`, `sacrifice LoseGripUpsideDownFromEarthquake`,
  `→ PygmyFallUpsideDown`; otherwise S13, `vel.x = 0`, `zrot0`, `mTiltState = 0`,
  `coin == 0 ? PygmyLoseBalance1 : PygmyLoseBalance2`; then
  `[listener onPygmyEarthquakeStart:self]` (the controller starts the island earthquake on
  the first pygmy, §23).

## 5. Touch: poke, drag, throw, flick

The touch dispatcher (logic.md §5) calls the eight `touch*:allTouches:` methods of every
selectable logic object front to back. `mActiveTouch` binds one finger to the pygmy;
`mTouchScreenPos` is the latest view position of that finger; `mbTouchBeganInside` means
"pressed on this pygmy, not yet dragging". Controller predicates used below:
`isDraggingPygmy` (`mDraggedPygmyLogic` or `mDraggedGhostPygmyLogic` set) and
`canTouchPygmy @ 0x100163c34` (no help/debug/challenge/stats screen, no eruption, no other
dragged object of 23 kinds, no earthquake sequence running).

### 5.1 `touchBeganInside:allTouches: @ 0x10013321c` (press on the pygmy)

1. Mouth filled by locusts → `sel0`, `→ PygmyMouthFilledByLocustsSplat`. Mutated by
   locusts → `vel.xy = 0`, `sacrifice SquashedLikeBug`, `sel0`,
   `→ PygmyMutatedByLocustsSplat`.
2. Dispatch: The Runs → `touchBeganTheRuns:allTouches:`; No Pee → `touchBeganNoPee:…`;
   Konkey Dong → `touchBeganKonkeyDong:…`; ghost → `touchBeganInsideGhost:…`; zombie →
   `zombieExplode`; sun-frozen → `iceShatter`. Each returns.
3. Possessed by a thetan: clear `mbPossessedByThetan` and
   `episode NormalThetanUnpossessPygmy` (`ApocalypseThetanUnpossessPygmy` during the
   apocalypse); the thetan reference stays until a stop list releases it.
4. Pull the pygmy out of a container: Kong statue (unless kidnapped), ape wench, ape
   bongo, ape altar, go-go cage, Dance Dance Execution: `removePygmyFromX:self`, reference
   nil, flag 0.
5. Grab, when: `!isDraggingPygmy`, `canTouchPygmy`, `mActiveTouch == nil`, tilt state 0,
   `FREE −{(see §25 row)} +{mbBeheadedBySquid, mbStoryMode}` and the drowning rule
   (a drowning pygmy can be grabbed only when not cooked, not cut, and plague body or
   T == 0: rescue from the water). Then: stop list (`stopHoldingChum, stopBeingHighOnNitrous,
   stopEating, stopCarrying, stopCarryingPlagueBody, stopFishing, stopSleeping,
   stopBeingEaten, stopWarmingByBonFire, stopStaringAtRainbow, stopStaringAtKidnapping,
   stopWaitingToCrap, stopThrowingSpear, closeWordBalloon, stopCharlieMischief,
   stopBeingTrappedInIdolPedestal, stopBeingOnTopApocalypseTimeTemplePlatform,
   stopBeingInApocalypseTempleFountain, stopThetanBrawl, stopBeingOnClosetTempleThetanMeter,
   stopBeingOnClosetTempleCouch, stopBeingInClosetTempleClosetDoor,
   stopBeingHeldInTomsArms, stopBeingPossessedByThetan, stopHoldingBabyPygmy,
   stopBeingEatenByLocustQueen, stopBeingThrownByLocusts`), `mbDrowning = 0`,
   `mActiveTouch = touch`, `vel.xy = 0`, alpha 1, `mbTouchBeganInside = 1`; on the fail
   boat: leave it (`mbOnFailBoat = 0`, `mTouchScreenPos = touch.pos`,
   `translateAndMoveToDisplayMarker:pygmyDragDisplayMarker`) and `startDrag` at once;
   otherwise the press pose: cooked `PygmyCookedTouch`, plague `PygmyPlagueBodyTouch`, cut
   `HalfPygmyTouch`, T != 0 `TemplePygmyTouch`, else `PygmyTouch`.
6. If the grab did not happen and the pygmy hangs upside down (tilt state 5, T == 0): the
   poke makes it let go: `stopSleeping`, `falling = 1`, `rot.z = -180`,
   `x += (xScale <= 0 ? 32 : -32)`, `y += 32`, `lastPos.y = y`, `mTiltState = 0`,
   `sacrifice UpsideDownLoseGrip`, `→ PygmyFallUpsideDown`.

`PygmyTouch` = `PygmyGrab` sound/anim, `PygmyPanicAnim`, `resetVel`.

`touchBeganOutside:allTouches: @ 0x100133df4`: mini-game dispatch as above (The Runs, No
Pee, Konkey Dong get the touch even when it is not on the pygmy); otherwise, with no
active touch: `mTouchScreenPos = touch.pos`, `mbTouchBeganInside = 0`.

### 5.2 Drag

`startDrag @ 0x100132fdc`: S14, label offset (0, -64), `mbDragging = 1`,
`mbApplySpring = 1`, `mbTouchBeganInside = 0`, `[listener onPygmyDrag:self]`, then cooked
`PygmyCookedDrag`, plague `PygmyPlagueBodyDrag`, cut `HalfPygmyDrag`, T != 0
`TemplePygmyDrag`, else `PygmyDrag`. The drag sequences play the type animation
(`setAnimationBasedOnPygmyType`) and run `drag (yOrigin -64, xOrigin -29)`.

Started from `touchMovedExited:allTouches: @ 0x100134b58` (the finger leaves the pygmy's
box while pressed): if this is the active touch (or none) and the pygmy is not a ghost
with a pressed touch, and `canTouchPygmy`, tilt 0, not in story mode and
`FREE −{mbTouchBeganInside, mbCutInHalfByCharlie, T!=0, mbDroppedByHurricane, mbSunFrozen,
the six containers, mbTrappedInSpiderWeb, mbUnderwater, mbDragging, mbInCrackHole, mbTumble,
mbCooked, mbKilledByPlague, mbDrowning} +{mbBeheadedBySquid, mbKonkeyDong, mbNoPee,
mbStoryMode, mbTheRuns}`: `mTouchScreenPos = touch.pos`; if nobody is dragged and
`mbTouchBeganInside` → `startDrag`; else if another pygmy is being dragged and this one
was pressed (not underwater, cooked, plague or cut): small hop `vel.y = 200`,
`falling = 1`, `mbTouchBeganInside = 0`, `sacrifice DroppedDrown`, `→ PygmyFall`
(`TemplePygmyFall`).

`touchMovedInside:allTouches: @ 0x1001351d4` forwards to `touchMovedOutside:allTouches:
@ 0x1001351e4`: the Konkey Dong ladder touch updates `mbKonkeyDongLadderUp` (view y,
converted to the `PocketGod` display group, above 160 = up); for the active touch (or none)
`mTouchScreenPos = touch.pos`, and a ghost forwards to `touchMovedGhost:allTouches:`.

`drag::: @ 0x100132094` (params yOrigin, xOrigin), every frame, never completes:

- First time: gravity on, floor and side collision off, origin = (xOrigin, yOrigin),
  `mOhStall = 0`, and unless underwater `layer(pygmyDragDisplayMarker)` +labels.
- `t` = `mTouchScreenPos` converted into the pygmy's display group; underwater
  `t.y = max(t.y, 65)`. `d = pos - t` (z component 0). If `|d| == 0` return 0.
- If `d.y != 0`: `rot.z = atan2(-d.y, -d.x) * 180 / π - 90` (the pygmy hangs from the
  finger, head up).
- "Oh" sounds (not cooked, plague or beheaded): `mOhStall--`; when `|vel.xy| > 500` and
  `mOhStall < 0`: `mOhStall = 60` (frames) and play `BabySound1` (T 2), `OldOh` (T 1),
  else `Oh1` / `Oh2` / `Oh3` for `r % 3` = 0 / 1 / 2.
- `mbOverFire = 0`; with a lit bonfire (enabled, `isOnFire`) and not beheaded:
  `|x - fire.x| < 20 && 40 < y - fire.y < 130` → `mbOverFire = 1` (feeds the 2 s cook
  timer of §4.1 step 9).
- Healthy pygmies of T 0, 2, 4: `[[listener sharkLogic:self] checkJumpForPygmy:self]`
  (the shark may leap at a dangled pygmy).
- Spring: `mSpringForce = (-100 d.x - 8 vel.x, -100 d.y - 8 vel.y, -8 vel.z, pos.w)`
  (the code computes `k = (|d| * -100) / |d|`, i.e. a constant -100).

The spring force is added in `onGameFrame:` (§4.1 step 5) together with gravity, so the
pygmy lags behind the finger and swings.

### 5.3 Release and throw: `touchEndedInside:allTouches: @ 0x100135320`

(`touchEndedOutside:allTouches: @ 0x100135954` forwards to it.)

1. Konkey Dong: the jump touch ends → nil, and unless holding the pick
   `→ PygmyKonkeyDongJumpDown`; the ladder touch ends → `mKonkeyDongLadderTouch = nil`.
   Return.
2. Ignore foreign touches (active touch set and different).
3. The Runs: with an active touch → nil, `→ PygmyTheRunsJumpDown`. No Pee →
   `touchEndedInsideNoPee:allTouches:`. Ghost with a pressed touch: unless going into the
   light, `[mTheLightLogic setBehavior:"TheLightExit"]`; touch nil, pressed 0; path with
   more than one point → `PygmyGhostFollowPath`; `[listener onPygmyGhostDragComplete:self]`.
4. **Release after a drag** (`mbDragging`): `mbOverFire = 0`, `mbDragging = 0`,
   `mbApplySpring = 0`, label offsets 0, `[[listener sharkLogic:self] cantCatchPygmy:self]`,
   `[listener onPygmyDragComplete:self]`. Unless a healthy normal pygmy hit the spider
   tree web (`isSpiderTreeEnabled` and `checkSpiderTreeCollision`, §10.4):
   - on land: `falling = 1`, `layer(pygmyDisplayMarker)` +labels, then cooked
     `PygmyCookedFall`, plague `PygmyPlagueBodyFall`, cut `HalfPygmyFall`, T != 0
     `TemplePygmyFall`, beheaded by the squid `sacrifice LostMyHead` +
     `PygmyBeheadedFall`, else `sacrifice DroppedDrown` + `PygmyFallFar`.
     **The throw is simply the velocity the spring gave the pygmy**; it keeps flying with
     it under gravity (`fall:::` decides landing or drowning, §9.6).
   - underwater: unless `checkJellyfishCollision` caught it (§14): side and floor
     collision off, `zrot0`, `vel.xy = 0`, then bait master → `ignoreGrav = 1`,
     `PygmyDropDuringBaitMaster`, else `PygmyDropUnderwater`.
5. **Tap** (pressed, never dragged): on land, `checkForZombieGrave` (a zombie grave next to
   the pygmy swallows it) ends here; else a hop: `vel.y = 200`, `falling = 1`, then
   cooked `PygmyCookedFall`, plague `PygmyPlagueBodyFall`, cut `HalfPygmyFall`, T == 0
   `sacrifice DroppedDrown` + `PygmyFall`, T == 1 (old fart) heart attack:
   `mbHadHeartAttack = 1`, `vel.xy = 0`, `TempleOldPygmyHeartAttack`, T 2..4
   `TemplePygmyFall`. Underwater: bait master → `ignoreGrav = 1`,
   `PygmyDropDuringBaitMaster`, else `PygmyDropUnderwater`.
6. `mbTouchBeganInside = 0`, `mActiveTouch = nil`.

### 5.4 Flick and push: `touchMovedEntered:allTouches: @ 0x100133eb4`

A finger that slides **into** a standing pygmy without having pressed it. (Ghidra mangles
the parameters of this method; the logic below is from the disassembly.)

- Foreign active touch → return. Sun-frozen → `iceShatter`, return.
- `old` = previous `mTouchScreenPos` converted to the pygmy's group; store the new touch
  position; `new` = the same conversion.
- Requires `!isDraggingPygmy`, `canTouchPygmy`, tilt state 0 and
  `FREE −{mbTouchBeganInside, T!=0, mbTrappedInSpiderWeb, mbSunFrozen (handled above)}
  +{mbStoryMode, mbBeheadedBySquid, mbTheRuns, mbNoPee, mbKonkeyDong}` (so temple types
  can be flicked; `mbFalling` is tested last).
- `max`, `min` = the pygmy's display bounds. `d = new - old`.
  - **From above** (`old.y >= max.y`): S14, `y -= 1`, `lastPos.y = y`, `falling = 1`,
    `mbDrowning = 0`, `sacrifice PushedDrown`, `→ PygmyFall` (`TemplePygmyFall`): the
    pygmy is pushed off its feet.
  - **From below** (`old.y <= min.y`): S14, `vel = (10 d.x, 40 d.y, -400)`,
    `mTumblePos = pos`, `falling = 1`, `mbDrowning = 0`; outside the temple rooms (L not
    11..17) `→ PygmyTumble` (`TemplePygmyTumble`): the pygmy is flicked into the air,
    away from the camera (`vel.z -400`). Inside the temple rooms
    `vel = (5 d.x, 20 d.y, 0)` and `→ PygmyFall` (`TemplePygmyFall`).
  - **From the side**: S14; if `d.y <= 0`: `y -= 1`, `lastPos.y = y`; `vel.x = 10 d.x`,
    `vel.y = 20 d.y`, `falling = 1`, `mbDrowning = 0`, `sacrifice PushedDrown`,
    `→ PygmyFall` (`TemplePygmyFall`). The side test compares `old.x` with `max.x` twice
    (`b.ge` / `b.ls`), so every entry that is neither from above nor from below takes
    this branch (keep).

`tumble:::` (§9.7) then flies the flicked pygmy in 3D: ice monster hits, the volcano,
the ocean.

### 5.5 Other touch consumers

- Ghost path drawing: `touchBeganInsideGhost:` / `touchMovedGhost:` (§10.9).
- Mini-games: `touchBeganKonkeyDong:` (§17), `touchBeganTheRuns:` (§18),
  `touchBeganNoPee:` / `touchEndedInsideNoPee:` (§15).
- `checkSpiderTreeCollision` (release over the spider tree, §10.4) and
  `checkJellyfishCollision` (underwater release, §14).

## 6. Behaviour state machine

### 6.1 Model

The pygmy's behaviour is the running sequence (`setBehavior:` replaces it, items advance it)
plus flags. A port can model it as: `current sequence + item index + per-item scratch
(mStallTime, mNextTurnStallTime...)` and a struct of flags; the "states" a designer would
name map to flags as follows.

| State | Flags / data | Entered by | Left by |
|---|---|---|---|
| falling | `mbFalling` (+ `mStartFallYPos`) | init, `fall:::`, every drop/eject/throw, no floor under the feet (§4.1 step 16) | `fall:::` landing or splash, catchers (§10.5) |
| walking / idle | none of the FREE flags; running `PygmyIdle` / `PygmyIdleLoop` / `walk` / `idle` | landing (`PygmyFall` → `PygmyIdle`) | any interaction |
| pressed | `mbTouchBeganInside`, `mActiveTouch` | `touchBeganInside:` | release (tap hop) or drag start |
| dragged | `mbDragging`, `mbApplySpring`, `mSpringForce` | `startDrag` | release (§5.3), `hitBySnowBall:`, temple drop, bait master |
| tumbling (flicked) | `mbTumble`, `mTumblePos`, `mbDescending`, `mbHitVolcano`, `mbHitIceMonster`, `mbFartherThanIceMonster` | `touchMovedEntered:` from below, `storyModeFlick:::`, igloo cannon | `tumble:::` splash / volcano / ice monster, `die:::` |
| tilted | `mTiltState` 1..5, `mbEnableUpsideDown` | `refreshTiltState` | tilt back, `undoTiltState`, touch, earthquake |
| earthquake | `mbEarthquake`, `mShakeCount`, `mbShakeUp`, `mLastShakeTime` | `checkForEarthquake:` | 0.5 s without shaking |
| drowning | `mbDrowning` (+ `ignoreGrav`) | `fall:::`, `slide:::`, `tumble:::` reaching the ocean | `die:::`, rescue (touch) |
| dead | removed by `onPygmyDie:` | `die:::` real branch | - |
| ghost | `mbIsGhost` and the ghost ivars | `die:::` with ghosts enabled | the light, `die:::` again |
| eaten | `mbEatenByTRex`, `mbEatenBySpider`, `mbSharkOnHead`, `mbEatenByMoronSnake`, `mbEatenByPiranha`, `mbEatenByLocustQueen`, `mbEatenByLaserShark` | the predator's call | the predator's swallow/release call |
| burning | `mbOnFireFromLava`, `mbOnFireFromBonFire`, `mbBurntByMagnifyingGlass`, `mbCooked` (body), `mbSunBurnt` | `hitByLava:`, `hitByBonFire:`, `burntByMagnifyingGlass:`, cook timer, sun | `jumpOnFire:::` (lava/fire run into the sea), `unburntByMagnifyingGlass:`, death |
| in a container | the 20 `mbIn*/mbTrapped*/mbOn*/mbHeld*/mbResting*` flags + reference | `checkFor*DuringFall`, walk-time checks | `tossFrom*`, `ejectFrom*`, `stopBeing*`, `removePygmyFrom*` callbacks, death |
| carrying / eating / fishing / warming / waiting / crapping / sleeping / dancing / staring | object references (`mEating*Logic`, `mCarryingWoodLogic`, `mFishingPoleLogic`, `m*WarmingBonFireLogic`, `mWaitingOutHouseLogic`, `mCrappingOutHouseLogic`), `mbSleeping`, `mbStoppingDance`, `mbStaringAtRainbow`, `mbStaringAtKidnapping`, `mbThrowingSpear` | walk-time `checkFor*` (§10.1) | the matching `stop*` (stop lists, §1.3) |
| mini-game | `mbTheRuns`, `mbKonkeyDong`, `mbNoPee`, `mbMoronPests`, `mbDoodleJumping`, `mbBattleOfTheGods` and their sub-flags | the mini-game controller | its own completion |
| story | `mbStoryMode`, `mbStoryModeReady`, `mbDieInStory*`, `mbRemoveFromStoryMode` | `setStoryMode` | `removeFromStoryMode`, death |

Exclusivity rules come from the guards (§25): nearly every "start X" requires the FREE set
to be clear, then runs a stop list, sets its own flag and switches behaviour.

### 6.2 Core loop

```
init → PygmyFall: PygmyFallAnim; setOrigin(-32,0,0,inPlace); fall; HitGround; PygmyLandAnim → PygmyIdle
PygmyIdle: checkForPossessedByThetan; checkForStoryMode; checkForVampire; checkForLocusts;
           checkForTRex; checkForSpider; checkForSunState;
           push walk anim by personality (Pygmy/Charlie/Tom/Purification/Sick Walk2Anim);
           walk(0.5, 2.0); resetCrapper; resetIgloo;
           checkForLocusts; checkForTRex; checkForSpider; checkForRainbow; checkForDance;
           checkForNight; checkForSunset;
           push idle anim by personality; idle(0.5, 5.0) → PygmyIdleLoop
PygmyIdleLoop: push walk anim; walk(0.5, 2.0); checkForLocusts; checkForTRex; checkForSpider;
           checkForDance; checkForNight; checkForSunState; checkForSunset;
           runSequenceBasedOnPersonality(default PygmyDefaultChooseIdle, charlie
           PygmyCharlieChooseIdle, tom PygmyTomChooseIdle, purification
           PygmyPurificationChooseIdle, plague PygmyPlagueChooseIdle)
PygmyDefaultChooseIdle: chooseSequence over 36 entries (Lookup x6, Lookdown x6, Sneeze,
           SkirtFall, SkirtFallBlackBox, ChestBeat, FixHair, ThoughtBubbleFish/Coconut/Dodo,
           Scratch, NosePick, Fart, Wave x2, Yawn, FinishPygmyIdle x10)
FinishPygmyIdle: push idle anim; idle(0.5, 5.0) → PygmyIdleLoop
```

Every `checkFor…` item returns 2 when nothing happens; after switching behaviour some
return 3 and some still return 2, which is equivalent because the sequence runner notices
the replaced sequence before applying the code (sequences.md §3.5). So the loop is "walk,
look around, react". `walk:::` itself scans for interactions every
frame (§9.2). Temple types loop through `TemplePygmyIdleLoop`
(`runSequenceBasedOnPersonality` by type and disease: `TempleBabyPygmyIdleLoop`,
`TempleFastPygmyIdleLoop`, `TempleOldPygmyIdleLoop`, `TempleGingerPygmyIdleLoop` and the
`<Type>PygmyPlague/PurificationChooseIdle` sequences; default `PygmyIdle`). Diseased
pygmies end their idles in `FinishPygmyPlagueIdle` / `FinishPygmyPurificationIdle`, which
run `checkDiseaseIncubation` (§11.5). Sleeping: `PygmyFallAsleep` → `PygmyAsleep` loop
(`checkForDay`, `checkForDreamBubble`) → `PygmyWakeUp` → `PygmyIdle`. Ghosts:
`PygmyGhostInit` → `ghostFlyUp` → `PygmyGhostIdle` loop. Underwater: `PygmyUnderwaterIdle`
swim loop (§14). Death: `PygmyDie` (just `die`), the splashes (`PygmySplash`,
`PygmySplashDistance` = drown animation, `SplashDrown`, `die`).

### 6.3 All behaviours started by the pygmy classes

The code of the pygmy classes starts 514 distinct sequence ids with `setBehavior:` (on the
pygmy itself or on helper objects such as the battle weapons, the light and the bottoms);
following `runSequence` / `chooseSequence` / push items from them over `sequences.json`
reaches 707 sequences. The 514 roots grouped by feature (first matching keyword); the
dynamic `PygmyStoryMode%@` ids of `setStoryModeCharacterState:` (§13) are not listed:

- **Battle of the Gods** (32): BattleLavaPitFlame, PygmyBattleAttackCounter, PygmyBattleAttackCounterFastAttack, PygmyBattleAttackCounterTie, PygmyBattleAttackFast, PygmyBattleAttackFastTie, PygmyBattleAttackFastTieAgainstEnergy, PygmyBattleAttackFastWaitForCounter, PygmyBattleAttackStrong, PygmyBattleAttackStrongEnergyTie, PygmyBattleAttackStrongTie, PygmyBattleAttackStrongWin, PygmyBattleAttackWin, PygmyBattleEnergyAttackCounter, PygmyBattleEnergyAttackCounterFastAttack, PygmyBattleEnergyAttackCounterTie, PygmyBattleEnergyAttackFast, PygmyBattleEnergyAttackFastTie, PygmyBattleEnergyAttackStrong, PygmyBattleEnergyAttackStrongTie, PygmyBattleEnergyAttackStrongWin, PygmyBattleFatalityAeris, PygmyBattleFatalityAerisTornado, PygmyBattleFatalityAnimus, PygmyBattleFatalityInfernis, PygmyBattleFatalityOceanus, PygmyBattleFatalityPax, PygmyBattleHitByEnergyBall, PygmyBattleKilled, PygmyBattleNoRunBack, PygmyBattlePowerUp, PygmyBattleSpawn
- **Battle weapon objects (global logic 377..398)** (40): PygmyEnergyAeris, PygmyEnergyAnimus, PygmyEnergyBallAeris, PygmyEnergyBallAnimus, PygmyEnergyBallInfernis, PygmyEnergyBallOceanus, PygmyEnergyBallPax, PygmyEnergyBallShieldAeris, PygmyEnergyBallShieldAnimus, PygmyEnergyBallShieldInfernis, PygmyEnergyBallShieldOceanus, PygmyEnergyBallShieldPax, PygmyEnergyBallSwordAeris, PygmyEnergyBallSwordAnimus, PygmyEnergyBallSwordInfernis, PygmyEnergyBallSwordOceanus, PygmyEnergyBallSwordPax, PygmyEnergyBeamAeris, PygmyEnergyBeamAnimus, PygmyEnergyBeamInfernis, PygmyEnergyBeamOceanus, PygmyEnergyBeamPax, PygmyEnergyInfernis, PygmyEnergyOceanus, PygmyEnergyPax, PygmyEnergyShieldAeris, PygmyEnergyShieldAnimus, PygmyEnergyShieldInfernis, PygmyEnergyShieldOceanus, PygmyEnergyShieldPax, PygmyShieldAeris, PygmyShieldAnimus, PygmyShieldInfernis, PygmyShieldOceanus, PygmyShieldPax, PygmySwordAeris, PygmySwordAnimus, PygmySwordInfernis, PygmySwordOceanus, PygmySwordPax
- **Konkey Dong** (15): PygmyChaseKonkeyDong, PygmyKidnappedByKonkeyDong, PygmyKonkeyDongFall, PygmyKonkeyDongFirstLand, PygmyKonkeyDongGrabPick, PygmyKonkeyDongHit, PygmyKonkeyDongJumpDown, PygmyKonkeyDongJumpToNextLevel, PygmyKonkeyDongJumpUp, PygmyKonkeyDongLadderClimb, PygmyKonkeyDongLadderIdle, PygmyKonkeyDongLand, PygmyKonkeyDongRun, PygmyKonkeyDongStart, PygmyKonkeyDongWatchKidnap
- **The Runs** (14): PygmyTheRunsFall, PygmyTheRunsHitSide, PygmyTheRunsHitStalactite, PygmyTheRunsJumpDown, PygmyTheRunsJumpUp, PygmyTheRunsRockExplode, PygmyTheRunsRun, PygmyTheRunsRunFast, PygmyTheRunsRunStartFast, PygmyTheRunsSplash, PygmyTheRunsStart, PygmyTheRunsStumble, PygmyTheRunsTumble, PygmyTheRunsTurdSkid
- **No Pee** (11): NoPeeEmbedBubbles, NoPeeEmbedDie, NoPeeEmbedSwimBubbles, PygmyNoPeeGlide, PygmyNoPeeHitClam, PygmyNoPeeHitMine, PygmyNoPeeHitPillar, PygmyNoPeeHitSeaBlob, PygmyNoPeeRoll, PygmyNoPeeStart, PygmyNoPeeSwimFlap
- **Moron Pests** (5): PygmyEatenByMoronTurtle, PygmyMoronPestsAnswerCorrect, PygmyMoronPestsAnswerIncorrect, PygmyMoronPestsLand, PygmyMoronPestsStart
- **Ooga Jump (doodle)** (7): PygmyDoodleBounce, PygmyDoodleBounceStart, PygmyDoodleBreakIceBounce, PygmyDoodleElectricBounce, PygmyDoodleJetPack, PygmyDoodleKnockedDown, PygmyDoodleSnowBounce
- **Story mode** (4): PygmyStoryModeDie, PygmyStoryModeIdle, StoryWordBalloonCloseRight, StoryWordBalloonOpenRight
- **PygmyBottomLogic** (3): PygmyBottomInit, PygmyBottomLocustBone, PygmyBottomLocustSkull
- **Ghost / The Light / banshee** (26): PygmyCarriedAwayByGhost, PygmyExplodeFromBansheeScream, PygmyGhostCarryAwayPygmy, PygmyGhostEnterTheLight, PygmyGhostFollowPath, PygmyGhostFollowPathFlip, PygmyGhostIdle, PygmyGhostInit, PygmyGhostPossessPygmy, PygmyGhostPullPygmyUnder, PygmyGhostPullPygmyUnderComplete, PygmyGhostScarePygmy, PygmyGhostStartCarryAwayPygmy, PygmyGhostStartScream, PygmyGhostStopScream, PygmyGhostWaitForPath, PygmyHoldEarsFromBansheeScream, PygmyPossessedByGhost, PygmyPulledUnderByGhost, PygmyScaredByGhost, PygmyStopHoldEarsFromBansheeScream, PygmyWaitForGhostTouch, PygmyWaitForGhostTouchSleep, TheLightEnter, TheLightExit, TheLightInit
- **Zombie** (13): PygmyFallIntoGrave, PygmyPushedIntoGraveBackward, PygmyPushedIntoGraveForward, PygmyStartZombieDancing, PygmyStopZombieDancing, PygmyZombieEatBrains, PygmyZombieExplode, PygmyZombieFall, PygmyZombieHaveBrainsEaten, PygmyZombieHaveBrainsEatenSleep, PygmyZombieIdle, PygmyZombieNoTiltSlide, PygmyZombieTiltSlide
- **Vampire / bat** (10): PygmyBitByBat, PygmyHitByVampire, PygmyKillPygmies, PygmyStartVampireDancing, PygmyVampireAttackGinger, PygmyVampireBurntByWarlock, PygmyVampireIdle, PygmyVampireSun, PygmyVampireSunset, TemplePygmyHitByVampire
- **Thetan** (19): PygmyFallOntoClosetTempleThetanMeter, PygmyPossessedByApocalypseThetan, PygmyPossessedByThetan, PygmyPossessedByThetanIdle, PygmyStartPossessedByThetan, PygmyThetanBrawlAttack, PygmyThetanBrawlAttackLoop, PygmyThetanBrawlDefend, PygmyThetanBrawlEatFace, PygmyThetanBrawlGetFaceEaten, PygmyThetanBrawlGetHit, PygmyThetanBrawlGetJumped, PygmyThetanBrawlGetPummeled, PygmyThetanBrawlJumpOff, PygmyThetanBrawlPummel, PygmyThetanBrawlSplash, PygmyThetanBrawlStartSlapFight, PygmyThetanBrawlThrowPunch, PygmyUnpossessedByThetan
- **Locusts** (15): PygmyCutInAirByLocusts, PygmyFallIntoLocustQueenArms, PygmyInhaleDanceBalloon, PygmyLocustPanicIdle, PygmyMouthFilledByLocusts, PygmyMouthFilledByLocustsSplat, PygmyMutatedByLocusts, PygmyMutatedByLocustsFlyOff, PygmyMutatedByLocustsSplat, PygmyStripFleshByLocusts, PygmyThrownByLocusts1, PygmyThrownByLocusts2, PygmyThrownByLocustsCatch1, PygmyThrownByLocustsCatch2, PygmyThrownByLocustsInitialLift
- **Plague / purification / disease** (23): PygmyCarryBabyPlagueBody, PygmyCarryFastPlagueBody, PygmyCarryGingerPlagueBody, PygmyCarryOldPlagueBody, PygmyCarryPlagueBody, PygmyKilledByPlague, PygmyKilledByPlagueDisappear, PygmyKilledByPurification, PygmyPlagueBodyBounce, PygmyPlagueBodyBurnOnFire, PygmyPlagueBodyDrag, PygmyPlagueBodyDropWhileCarrying, PygmyPlagueBodyDroppedByHurricane, PygmyPlagueBodyFall, PygmyPlagueBodyFallOffWorld, PygmyPlagueBodyHurricaneSplash, PygmyPlagueBodyPickedUpByHurricane, PygmyPlagueBodySplash, PygmyPlagueBodySplashDisappear, PygmyPlagueBodyThrowOnPile, PygmyPlagueBodyTiltSlide, PygmyPlagueBodyTouch, PygmyRecoilFromDisease
- **Closet temple / Tom / baby / Xenu** (14): FinishPygmyTomClosetIdle, PygmyFallIntoClosetTempleClosetDoor, PygmyFallIntoXenu, PygmyFallOntoClosetTempleCouch, PygmyTomCatchBabyPygmyLeft, PygmyTomCatchBabyPygmyRight, PygmyTomEjectBabyPygmy, PygmyTomFallOntoClosetTempleCouch, PygmyTomJumpOffCouchAndSlamDoor, PygmyTomJumpOnCouch, PygmyTomSlamDoor, TempleBabyPygmyEatenByMoronSnake, TempleBabyPygmyEatenByTRex, TempleBabyPygmyUneatenByTRex
- **Time temple / temple pygmy types** (21): TempleFastPygmyExplodeOnIceMonster, TempleGingerPygmySunBurnt, TempleOldPygmyFallDown, TempleOldPygmyFallDownFromSlide, TempleOldPygmyHeartAttack, TempleOldPygmyHeartAttackSplash, TempleOldPygmyLandOnFeet, TemplePygmyDrag, TemplePygmyDroppedByHurricane, TemplePygmyFall, TemplePygmyFallInsideApocalypseTempleRoom, TemplePygmyFallIntoApocalypseTempleDoor, TemplePygmyHitByCoconut, TemplePygmyIdleLoop, TemplePygmyPickedUpByHurricane, TemplePygmySplash, TemplePygmyStartDancing, TemplePygmyStopDancing, TemplePygmyTiltSlide, TemplePygmyTouch, TemplePygmyTumble
- **Charlie / half pygmy / warlock** (21): FinishPygmyCharlieIdle, HalfPygmyAbsorbCharlie, HalfPygmyAbsorbPygmy, HalfPygmyDrag, HalfPygmyDroppedByHurricane, HalfPygmyFall, HalfPygmyFallInsideApocalypseTempleRoom, HalfPygmyPickedUpByHurricane, HalfPygmySplash, HalfPygmyTiltSlide, HalfPygmyTouch, PygmyAbsorbedByHalfPygmy, PygmyCharlieAbsorbedByHalfPygmy, PygmyCharlieCutPygmyInHalf, PygmyCharlieEatenByShark, PygmyCharlieMonkeyOnBack, PygmyCharlieNoSleep, PygmyCharlieSmackBoulder, PygmyCharlieSmackTRexEgg, PygmyCutInHalfByCharlie, PygmyCutInHalfByCharlieComplete
- **Underwater** (39): PygmyAddedUnderwater, PygmyDrankByJellyfish, PygmyDropDuringBaitMaster, PygmyDropUnderwater, PygmyDroppedByWhirlPool, PygmyEatenByLaserShark, PygmyEscapeFromDrain, PygmyHitByChainSpearLeft, PygmyHitByChainSpearLeftRetract, PygmyHitByChainSpearRight, PygmyHitByChainSpearRightRetract, PygmyHitByJellyfish, PygmyHitBySharkLaser, PygmyHitByUnderwaterPowerAcid, PygmyHitByUnderwaterPowerAir, PygmyHitByUnderwaterPowerCold, PygmyHitByUnderwaterPowerHeat, PygmyHitByUnderwaterPowerOil, PygmyHoldChum, PygmyPainDrainFall, PygmyPainDrainLand, PygmyPainDrainSpiked, PygmyPainDrainStart, PygmyPainDrainTurn, PygmyPickedUpByWhirlPool, PygmyPiranhaAttack, PygmyPiranhaAttackComplete, PygmyShockedByJellyfish, PygmyStartBaitMaster, PygmyStartUnderwater, PygmyStopBaitMaster, PygmyStopPiranhaAttack, PygmyStopUnderwater, PygmyStungByJellyfish, PygmySuckedIntoDrain, PygmySuffocatedByJellyfish, PygmyUnderwaterIdle, PygmyUnderwaterPanicIdle, PygmyWaitUnderwater
- **Containers (catch and eject)** (26): PygmyCrawlIntoIgloo, PygmyCrawlOutOfIgloo, PygmyFallIntoApeAltar, PygmyFallIntoApeBongo, PygmyFallIntoApeWench, PygmyFallIntoApocalypseTempleDoor, PygmyFallIntoApocalypseTempleFountain, PygmyFallIntoCrackHole, PygmyFallIntoDanceDanceExecution, PygmyFallIntoGoGoCage, PygmyFallIntoIdolPedestal, PygmyFallIntoIgloo, PygmyFallIntoKongStatue, PygmyFallIntoTempleDoor, PygmyFallOntoApocalypseTimeTemplePlatform, PygmyHitByLightningInApeSomething, PygmyHitByLightningInKongStatue, PygmyHitByPygmyIglooRoll, PygmyJumpIntoTempleDoor, PygmyLeaveIglooDoorExplode, PygmyLeaveIglooTopExplode, PygmyLeaveIglooTopExplodeFall, PygmyRefreshGoGoCage, PygmyStuckInWeb, PygmyTornApartFromApeAltar, PygmyWearWaterWings
- **Ice / sun / weather** (32): PygmyBeheadedFall, PygmyBeheadedSplash, PygmyBlownByHurricane, PygmyBurntFallIntoIceHole, PygmyCookedDroppedByHurricane, PygmyCookedPickedUpByHurricane, PygmyDroppedByHurricane, PygmyExplodeOnIceMonster, PygmyGainBalance, PygmyGainBalanceAfterTilt, PygmyGrabbedByTentacle, PygmyHeadRippedOffByIceMonster, PygmyHitByIcicle, PygmyHitByIcicleSplash, PygmyHitByLava, PygmyHitByLightning, PygmyHitByLightningInHurricane, PygmyHitBySnowBall, PygmyIceShatter, PygmyLoseBalance1, PygmyLoseBalance2, PygmyPickedUpByHurricane, PygmyStartIceFishing, PygmyStopIceFishing, PygmySunBurn, PygmySunBurnt, PygmySunBurntCreateSoloIceHole, PygmySunBurntExplode, PygmySunFreeze, PygmySunFrozen, PygmySunset, PygmyUnblownByHurricane
- **Island hazards** (63): PygmyBurntByMagnifyingGlass, PygmyCatchShark, PygmyCatchSpear, PygmyCaughtByBird, PygmyCaughtByMonkey, PygmyCaughtByShark, PygmyChokeSplash, PygmyCook, PygmyCookedBounce, PygmyCookedDrag, PygmyCookedFall, PygmyCookedSplash, PygmyCookedTiltSlide, PygmyCookedTouch, PygmyCrushedByBoulder, PygmyCrushedByTRexEgg, PygmyEatBird, PygmyEatBirdChoke, PygmyEatFishChoke, PygmyEatenByMoronSnake, PygmyEatenByMoronSnakeComplete, PygmyEatenByShark, PygmyEatenBySpider, PygmyEatenByTRex, PygmyExplodeFromLovingPocketGod, PygmyHitByBanana, PygmyHitByBananaAsleep, PygmyHitByBirdPoo, PygmyHitByBirdPooAsleep, PygmyHitByBonFire, PygmyHitByBoulder, PygmyHitByCoconut, PygmyHitByCoconutAsleep, PygmyHitByCookedBird, PygmyHitByCookedBirdAsleep, PygmyHitByCookedPygmy, PygmyHitByCookedPygmyAsleep, PygmyHitByFish, PygmyHitByFishAsleep, PygmyHitByShark, PygmyHitBySwordFish, PygmyHitBySwordFishAsleep, PygmyHitByTRexEgg, PygmyHumpedByMonkeyBehind, PygmyHumpedByMonkeyFront, PygmyMonkeyOnBack, PygmyRunUnderSpider, PygmyRunUnderTRex, PygmyStartStaringAtKidnapping, PygmyStayAwayFromSpider, PygmyStayAwayFromTRex, PygmyStepAwayFromSpider, PygmyStepAwayFromTRex, PygmyStopStaringAtKidnapping, PygmySwarmedByAntHill, PygmyThrowFromExplosion, PygmyThrownByMonkey, PygmyUnburntByMagnifyingGlass, PygmyUncrushedByBoulder, PygmyUncrushedByTRexEgg, PygmyUneatenByShark, PygmyUneatenBySpider, PygmyUneatenByTRex
- **Island life** (37): PygmyAlreadyReactBackward, PygmyAlreadyReactForward, PygmyAsleep, PygmyAteSwordFishSplash, PygmyCarryWood, PygmyCatchFish, PygmyEatBanana, PygmyEatCoconut, PygmyEatFish, PygmyEatSwordFish, PygmyFallAsleep, PygmyFinishCrapping, PygmyGushOverRainbow, PygmyPullPole, PygmyReactBackward, PygmyReactForward, PygmySlipOnBanana, PygmyStartDancing, PygmyStartDancingThemeSongBackup, PygmyStartFishing, PygmyStartStaringAtRainbow, PygmyStopDancing, PygmyStopFishing, PygmyStopFishmasDancing, PygmyStopNapoleonDancing, PygmyStopPantsDancing, PygmyStopStaringAtRainbow, PygmyStopThemeSongDancing, PygmyStopThemeSongDancingBackup, PygmyStopTwelveDaysDancing, PygmyStopUpdateSongDancing, PygmyTakeCrap, PygmyTakeCrapAfterWaiting, PygmyTiltSlideSleep, PygmyWaitToCrap, PygmyWakeUp, PygmyWarmHands
- **Tilt / fall / splash / touch / drag / idle** (24): AshPileInit, BlankAnim, PygmyDie, PygmyDrag, PygmyFall, PygmyFallFar, PygmyFallFromBack, PygmyFallIgnoreIsland, PygmyFallInsideApocalypseTempleRoom, PygmyFallIntoTomsArms, PygmyFallScream, PygmyFallUpsideDown, PygmyHangUpsideDown, PygmyIdle, PygmyPickupBackward, PygmyPickupForward, PygmySplash, PygmyTiltSlideBackward, PygmyTiltSlideForward, PygmyTiltTipBackward, PygmyTiltTipForward, PygmyTiltUpsideDownToFallForward, PygmyTouch, PygmyTumble

## 7. Personalities, pygmy types and diseases

Three independent axes select behaviour variants.

### 7.1 Personality P (`mPersonaPersonality`, from the persona)

| P | Persona | Differences |
|---|---|---|
| 0 | Pygmy0..5 (OOGA, BOOGA, NOOBY, DOOBY, KLIK, KLAK) | default |
| 1 | `CharliePygmy` "Charlie" (guest, active when option `CharlieState == 1`) | red label background; `Charlie*` walk/idle animations; `idle:::` and `walk:::` run `checkForTigerBlood` first (cuts a pygmy in front of him in half, §12); `PygmyCharlieChooseIdle` (warlock idles 11 of 19 entries, quote idles); never falls asleep at night (`PygmyCharlieNoSleep`, `mbCharlieNotSleeping`); `PygmyCharlieEatenByShark`; a monkey can ride him (`mCharlieOnBackMonkeyLogic`, `PygmyCharlieMonkeyOnBack`, kicked off with `monkeyBootedOffCharliesBack:`); absorbed variant `PygmyCharlieAbsorbedByHalfPygmy` (`AbsorbedByCharlie` from the half pygmy's side); vampires do not bite a normal-type Charlie (`canBeHitByVampire` requires `P != 1 || T != 0`); a normal-type Charlie throws food away (`animCharlieThrowFood`) instead of eating it: no food gift and the food is not finished (§10.1); `PygmyTumble` pushes `CharlieSlapWinningScream2` instead of `SmackPygmyOhNo` |
| 2 | `TomPygmy` "Tom" (guest, created by `addTomPygmy:::`) | red label background; `Tom*` animations; `PygmyTomChooseIdle` (Xenu-room variant via `runSequenceIfInXenuRoom`); closet temple behaviours (couch jumps, door slam, catching the baby, walking to Xenu, §11.6); `trappedInCloset`; dropping into a room is skipped while in the closet; the closet door grabs a new Tom after his death (`mbTellClosetToGrabNewTom`) |

### 7.2 Pygmy type T (`mTimeTemplePygmyType`)

Set by `setTimeTemplePygmyType: @ 0x10003ca9c` (also refreshes the label) from the time
temple platform, or 2 for `addBabyPygmy:::`. Any T != 0 is in FREE, so temple types are
excluded from almost every island interaction; they walk with `templePygmyWalk:::` (which
only checks coconut, fish, bird and banana, §11.1) and idle in `TemplePygmyIdleLoop`.

| T | Label | Specific behaviour |
|---|---|---|
| 1 old fart | `OldFart%@` | tap → heart attack (`mbHadHeartAttack`, `TempleOldPygmyHeartAttack`), dies as `OldFartPygmyHeartAttack` (water: `TempleOldPygmyHeartAttackSplash`); `fall:::` landing from 50+ above the start → `TempleOldPygmyFallDown`, else `TempleOldPygmyLandOnFeet` (`mbTempleOldPygmyUpright`); slides knock him down (`checkTempleOldPygmyFallDownFromSlide`); drag sound `OldOh`; eats without gifts and without finishing the food; tilt uses temple animations |
| 2 baby | `Baby%@` | drag sound `BabySound1`; can be eaten by the T-Rex (`TempleBabyPygmyEatenByTRex`, `BabyPygmyEatenByTrex`), the shark (`BabyPygmyEatenByShark`) and the Moron snake (`TempleBabyPygmyEatenByMoronSnake`, may throw the snake, `animBabyThrowSnake`); may wear water wings; the only type Tom can catch (`checkForTomPygmyDuringFall`) and sacrifice to Xenu; tilt uses temple animations |
| 3 ginger | `Ginger%@` | burns in daylight (`checkForGingerSunBurnt`, `GingerPygmySunBurn`); kills an attacking vampire (`GingerPygmyKillVampire`, attack tolerance 40 instead of 5); tilts like a normal pygmy |
| 4 fast | `Fast%@` | `episode FastPygmyFlick` when flicked; explodes on the ice monster (`FastPygmyHitIceMonster`) and hits the volcano as `FastPygmyHitVolano`; `hitByPygmy:onFire:NO`; tilts like a normal pygmy |

T 0, 2 and 4 make the shark check for a jump while dragged; T 0 and 2 can be eaten by
the T-Rex and the Moron snake and wear water wings (`(T | 2) == 2`).

### 7.3 Disease D (`SpriteLogic::mApocalypseDiseaseType`)

1 plague, 2 purification (0 healthy). Set by `SpriteLogic::infectWithDisease:` (fountains,
infected objects, other pygmies), cleared by `cureDisease` (death, or the 1-in-3 cure of
`incubateDisease:…`). Diseased pygmies use the sick / purification animations, idle via
`PygmyPlagueChooseIdle` (cough, sneeze, barf) or `PygmyPurificationChooseIdle` (spray,
clean hands), infect neighbours from their idle animations (`animStartPygmyRecoil`,
`animInfectPygmies`) and die when `checkDiseaseIncubation` runs out (§11.5).
`mbKilledByPlague` turns the pygmy into a **plague body** (`PygmyPlagueBody*` variants of
touch, drag, fall, splash, bounce, tilt, hurricane), which other pygmies carry to the fire.

Other body variants that replace the normal behaviour family in the same places:
`mbCooked` (`PygmyCooked*`), `mbCutInHalfByCharlie` (`HalfPygmy*`), `mbIsZombie`
(`PygmyZombie*`, explodes instead of most reactions).

### 7.4 Selection items

`pushSequenceBasedOnPersonality::: @ 0x100132a64` (push) and
`runSequenceBasedOnPersonality::: @ 0x100132c9c` (setBehavior) share one algorithm
(sequences.md §3.12):

```
r = nil
T==2: r = (D==2 ? babyPurificationId : D==1 ? babyPlagueId : nil) ?? babyId
T==1: r = (D==2 ? oldFartPurificationId : D==1 ? oldFartPlagueId : nil) ?? oldFartId
T==3: r = (D==2 ? gingerPurificationId : D==1 ? gingerPlagueId : nil) ?? gingerId
T==4: r = (D==2 ? fastPurificationId : D==1 ? fastPlagueId : nil) ?? fastId
r = r ?? (D==2 ? purificationId : D==1 ? plagueId : nil)
r = r ?? (P==2 ? tomId : P==1 ? charlieId : nil)
r = r ?? defaultId
r == nil → nothing; else push (pushSequenceDef: getSequenceDefById) / run (setBehavior:)
return 2
```

So disease beats personality: a sick Charlie walks with the sick animation.

`setAnimationBasedOnPygmyType::: @ 0x100132898` plays an animation id (params in order:
0 oldFartPurification, 1 oldFartPlague, 2 normal, 3 babyPlague, 4 gingerPlague,
5 purification, 6 fastPurification, 7 plague, 8 gingerPurification, 9 babyPurification,
10 fastPlague, 11 fast, 12 ginger, 13 baby, 14 oldFart, 15 normalPurification,
16 normalPlague):

```
T==1: r = (D==2 ? p0 : D==1 ? p1 : nil) ?? p14
T==2: r = (D==2 ? p9 : D==1 ? p3 : nil) ?? p13
T==3: r = (D==2 ? p8 : D==1 ? p4 : nil) ?? p12
T==4: r = (D==2 ? p6 : D==1 ? p10 : nil) ?? p11
r = r ?? (D==2 ? p5 : D==1 ? p7 : nil) ?? p2
[animation playAnimationId:r]   (even when nil); return 2
```

Parameters 15 and 16 are never read (only `TemplePygmyDrag` fills them; keep the
fall-through to `normal`).

Battle-specific selection (`pushSequenceBasedOnOpponentWeaponType:::`,
`pushSequenceBasedOnOpponentBattleType:::`) is in §16.

## 8. Deaths, sacrifices, gifts and statistics

### 8.1 Pipeline

1. Whatever starts a lethal sequence sets `mSacrificeId` (the table below) and often
   `mbInvalidSacrifice` decides whether it counts. `mbInvalidSacrifice` is set by `init`
   (new arrivals), `respawnPlagueBody:::`, `droppedInAirByLocusts:`,
   `dropIntoApocalypseTempleRoom`, the upside-down fall of `refreshTiltState`, and cleared by
   landing (§4.1 step 16), `addedUnderwater`, `cutInAirByLocusts:` and
   `startInClosetTempleClosetDoor`. A pygmy that dies before ever touching the ground (it
   fell into the sea on arrival, or was thrown into a temple room and fell out) does not
   count.
2. `die:::` (§3.6) resolves overrides, fires `[listener onPygmySacrificeEvent:id]` →
   `PocketGodViewController::onPygmySacrificeEvent: @ 0x100070dc4` →
   `[IdolManager triggerEpisodeEventBySacrificeId:id]` (episode progress; fired even for
   invalid sacrifices), and either converts the pygmy into a ghost or calls
   `onPygmyDie:sacrificeId:`.
3. `onPygmyDie:sacrificeId: @ 0x10016485c` → `onTriggerSacrifice:sacrificeId:
   @ 0x100070de0`: `[tribe sacrifice:id pygmyLogic:p]` (§22: tribe and persona totals and
   per-id counts, persona moved to the dead list); if id is set: `mTotalSacrifices =
   tribe.totalSacrifices`, `[GameServer submitToLeaderboard:@"pg_totalsacrifices" score:]`,
   and a per-id leaderboard via `getSacrificeLeaderboardId: @ 0x100072908`, which returns
   nil for every id (its comparisons against the 116 known ids are dead code), so nothing
   more is submitted. Then the controller removes the pygmy (live count, display,
   `shutdown`, arrays), increments ChallengeManager game 9 when active ("kill pygmies"
   challenge), and forgets it as Tom / doodle / pain-drain / The Runs / Konkey Dong / Moron
   Pests / No Pee pygmy (clearing those modes).
4. `triggerActiveSacrifice @ 0x100128c8c` (called only by `NoPeeLogic`): if an id is set,
   `onPygmySacrificeEvent:` + `onTriggerSacrifice:sacrificeId:` immediately, then
   `mSacrificeId = DroppedDrown`. `fatalityInBattle` (§16) records `BattleFatality` the
   same way directly.

Achievements are not driven by the pygmy: `GodPersona` keeps an achievement dictionary
(§22) that other systems update; the pygmy only feeds episode events (IdolManager),
sacrifice/gift statistics (TribePersona, leaderboards) and ChallengeManager scores
(game 9 deaths, game 1 lightning in a hurricane, game 4 Ooga Jump via
`onPygmyStopDoodleJump:`, game 12 Konkey Dong score).

### 8.2 Every sacrifice id

117 ids: the 116 the controller knows plus `AbsorbedByCharlie`. "Set by" is the method that
assigns `mSacrificeId` (or reports it).

| Sacrifice id | Set by | Trigger | Behaviour |
|---|---|---|---|
| `DroppedDrown` | `touchEndedInside:allTouches:` @ 0x100135320<br>`touchMovedExited:allTouches:` @ 0x100134b58<br>`tossFromOutHouse` @ 0x10009562c<br>`droppedInAirByLocusts:` @ 0x100042ad4<br>`dropIntoApocalypseTempleRoom` @ 0x10003bd74<br>`triggerActiveSacrifice` @ 0x100128c8c | default after a release, tap hop, eject (EJ block) or temple drop; reset value after triggerActiveSacrifice | PygmyFallFar / PygmyFall / TemplePygmyFall, then a splash |
| `PushedDrown` | `touchMovedEntered:allTouches:` @ 0x100133eb4 | finger slides in from above or the side | PygmyFall |
| `FlickDrown` | `tumble:::` @ 0x1001317ec | flicked pygmy lands in the ocean | PygmySplash / TemplePygmySplash |
| `SlideDrown` | `slide:::` @ 0x100130b94 | tilt slide off the island (healthy, T 0) | PygmyFallIgnoreIsland, PygmySplash |
| `FallUpsideDown` | `refreshTiltState` @ 0x10012be50 | device upside down, pygmy did not grip (T 0, 3, 4; invalid sacrifice) | PygmyFallFromBack / PygmyFallFar |
| `UpsideDownLoseGrip` | `touchBeganInside:allTouches:` @ 0x10013321c | poke a hanging pygmy | PygmyFallUpsideDown |
| `LoseGripUpsideDownFromEarthquake` | `checkForEarthquake:` @ 0x10012b1d0 | shake while hanging | PygmyFallUpsideDown |
| `LostMyHead` | `touchEndedInside:allTouches:` @ 0x100135320 | release a pygmy beheaded by the ice squid | PygmyBeheadedFall |
| `Cooked` | `onGameFrame:` @ 0x100129980 | held over a lit bonfire for more than 2 s | PygmyCook |
| `ThrownExplosionDrown` | `throwFromExplosion:` @ 0x10009731c | blown away by an explosion (controller onPygmyExplode:) | PygmyThrowFromExplosion |
| `Lightning` | `hitByLightning` @ 0x100097c2c | lightning strike | PygmyHitByLightning (InKongStatue / InApe* variants) |
| `HurricaneLightning` | `hitByLightning` @ 0x100097c2c | struck while carried by a hurricane | PygmyHitByLightningInHurricane |
| `ZappedByWarlockAssassin` | `hitByLightningByWarlock` @ 0x1000581d0 | Charlie's warlock lightning | PygmyHitByLightning (variants) |
| `BurntByWarlockAssassin` | `burntByWarlock:` @ 0x10005a3e8 | warlock burns a vampire pygmy | PygmyVampireBurntByWarlock |
| `LovingPocketGod` | `explodeFromLovingPocketGod:::` @ 0x1000ba66c | octo statue love | explode::: (PygmyExplodeFromLovingPocketGod) |
| `Lava` | `hitByLava:` @ 0x1000992dc | volcano lava | PygmyHitByLava |
| `IciclePinCushion` | `hitByIcicle:` @ 0x100099aec | falling icicle | PygmyHitByIcicle |
| `BonFire` | `hitByBonFire:` @ 0x10009a328 | bonfire | PygmyHitByBonFire |
| `HitBySnowBall` | `hitBySnowBall:` @ 0x10009bc0c | snowball | PygmyHitBySnowBall |
| `BoulderDrown` | `hitByBoulder:xVel:` @ 0x10009ef00 | rolling boulder knocks it into the sea | PygmyHitByBoulder |
| `TRexEggDrown` | `hitByTRexEgg:xVel:` @ 0x10009ffcc | rolling T-Rex egg | PygmyHitByTRexEgg |
| `SharkDrown` | `hitByShark:xVel:` @ 0x1000a0830 | thrown shark | PygmyHitByShark |
| `IglooPygmy` | `hitByPygmyIglooRoll:` @ 0x10009fe04 | hit by a pygmy rolling out of the igloo | PygmyHitByPygmyIglooRoll |
| `ShotOutIgloo` | `leaveIglooDoorExplode` @ 0x1000963b0<br>`leaveIglooTopExplode` @ 0x10009651c | igloo explodes with the pygmy inside | PygmyLeaveIglooDoorExplode / PygmyLeaveIglooTopExplode |
| `IglooCannon` | `tumble:::` @ 0x1001317ec | shot from the igloo top into the ocean | PygmySplash |
| `Vampire` | `hitByVampire:xVel:` @ 0x1000a1798 | bitten by a vampire pygmy | PygmyHitByVampire / TemplePygmyHitByVampire |
| `VampireInSunlight` | `checkKillPygmies:::` @ 0x1000bd798<br>`vampireRun:::` @ 0x1000bda9c<br>`waitForNight:::` @ 0x1000bde8c<br>`killPygmies:::` @ 0x1000be098 | vampire caught by daylight | PygmyVampireSun |
| `GingerPygmyKillVampire` | `killPygmies:::` @ 0x1000be098 | vampire attacks a ginger pygmy | PygmyVampireAttackGinger |
| `EatenByTRex` | `swallowedByTRex:` @ 0x1000a2e1c | T-Rex swallows | PygmyDie |
| `BabyPygmyEatenByTrex` | `swallowedByTRex:` @ 0x1000a2e1c | same, T 2 | PygmyDie |
| `EatenBySpider` | `swallowedBySpider:` @ 0x1000a383c | spider swallows | PygmyDie |
| `EatenByLittleSpiders` | `eatenByLittleSpiders:` @ 0x1000a4270 | trapped in the spider-tree web | PygmyDie |
| `MagnifyingGlass` | `burntByMagnifyingGlass:` @ 0x1000a4a40 | magnifying glass beam | PygmyBurntByMagnifyingGlass |
| `FireAnts` | `swarmedByAntHill:` @ 0x1000a6a10 | ant hill | PygmySwarmedByAntHill |
| `CrackHole` | `fallIntoCrackHole:` @ 0x1000a78f0 | falls into the crack hole (may start The Runs) | PygmyFallIntoCrackHole |
| `KidnappedByKonkeyDong` | `kidnappedByKonkeyDongComplete` @ 0x1000a7afc | Konkey Dong carries it off | PygmyKidnappedByKonkeyDong |
| `HurricaneDrown` | `droppedByHurricane:` @ 0x1000a9e3c | dropped by the hurricane | PygmyDroppedByHurricane |
| `FedToShark` | `caughtByShark:` @ 0x1000aa068 | shark catches a dangled/thrown pygmy | PygmyCaughtByShark |
| `BabyPygmyEatenByShark` | `caughtByShark:` @ 0x1000aa068 | same, T 2 | PygmyCaughtByShark |
| `SharkOnHead` | `sharkSwallow:::` @ 0x1000be808 | the shark on its head swallows it | PygmyDie |
| `SharkFishingPole` | `catchShark` @ 0x100097290 | catches a shark while fishing | PygmyCatchShark |
| `CaughtByBird` | `caughtByBirdComplete:` @ 0x1000ab930 | carried off by the bird | PygmyDie |
| `EatBurntBird` | `finishEatingBird:::` @ 0x1000b892c | ate a burnt bird (T 0) | eat sequence (PygmyEatBirdChoke) |
| `EatBurntFish` | `finishEatingFish:::` @ 0x1000b8b04 | ate a burnt fish (T 0) | eat sequence (PygmyEatFishChoke) |
| `SwordSwallower` | `finishEatingSwordFish:::` @ 0x1000b8ca8 | swallows a sword fish | eat sequence |
| `NotCrapping` | `explodeFromNotCrapping:::` @ 0x1000ba894 | out-house wait ran out | explode::: |
| `MonkeyOnBack` | `monkeyOnBack:` @ 0x1000aec6c | monkey jumps on its back | PygmyMonkeyOnBack |
| `MonkeyThrow` | `thrownByMonkey:toMonkey:` @ 0x1000b0608 | monkeys play catch with it | PygmyThrownByMonkey |
| `EatenByMoronSnake` | `eatenByMoronSnake:` @ 0x1000b07c4<br>`eatenByMoronSnakeComplete:::` @ 0x1000b1bd0 | Moron snake | PygmyEatenByMoronSnake(Complete) |
| `WaterWingsDrowned` | `wearWaterWings:` @ 0x1000b1210<br>`checkNoPeeCollision:` @ 0x10018a914 | water wings; No Pee: sank below y 1000 | PygmyWearWaterWings / PygmyNoPeeHitPillar |
| `SquidBait` | `eatenByIceSquid:` @ 0x1000b6b8c | ice-hole squid eats a dangled pygmy | PygmyDie |
| `Shattered` | `iceShatter` @ 0x1000ba23c | frozen pygmy touched or falling | PygmyIceShatter |
| `GlobalWarming` | `sunBurntComplete:::` @ 0x1000bc9e0 | burnt by the sun | PygmySunBurntExplode / ice-hole melt |
| `GingerPygmySunBurn` | `checkForGingerSunBurnt:::` @ 0x1000b9e38 | ginger in daylight | TempleGingerPygmySunBurnt |
| `EatenByIceMonster` | `tumble:::` @ 0x1001317ec | flicked into the ice monster mouth | PygmyDie |
| `ExplodeOnIceMonster` | `tumble:::` @ 0x1001317ec | hits the recharging ice monster | PygmyExplodeOnIceMonster |
| `BounceOffIceMonster` | `tumble:::` @ 0x1001317ec | bounced off the ice monster into the sea | PygmySplash |
| `FastPygmyHitIceMonster` | `tumble:::` @ 0x1001317ec | fast pygmy hits the ice monster | TempleFastPygmyExplodeOnIceMonster |
| `ThrownInVolcano` | `tumble:::` @ 0x1001317ec | flicked into the volcano | PygmyDie + onVolcanoHit: |
| `FastPygmyHitVolano` | `tumble:::` @ 0x1001317ec | same, T 4 (sic, "Volano") | PygmyDie + onVolcanoHit: |
| `ApeAltar` | `tornApartFromApeAltar` @ 0x100095f10 | ape altar | PygmyTornApartFromApeAltar |
| `ApeEaten` | `grabbedFromApeAltar` @ 0x1000960bc | ape grabs it from the altar | PygmyDie |
| `ApeWenchSmash` | `apeSmashOnApeWench` @ 0x10009614c | ape smashes the wench | PygmyDie |
| `ApeBongoSmash` | `apeSmashOnApeBongo` @ 0x1000961dc | ape smashes the bongo | PygmyDie |
| `PulledIntoGround` | `pulledUnderByGhost:` @ 0x1000b2a50 | ghost pulls it underground | PygmyPulledUnderByGhost |
| `ScaredByGhost` | `scaredByGhost:` @ 0x1000b2818 | ghost scares it | PygmyScaredByGhost |
| `CarriedAwayGhost` | `carriedAwayByGhost:` @ 0x1000b2bf4 | ghost carries it off screen | PygmyCarriedAwayByGhost |
| `FromBansheeScream` | `explodeFromBansheeScream:` @ 0x1000b3814<br>`explodeFromBansheeScream:::` @ 0x1000b75d8 | banshee scream | PygmyExplodeFromBansheeScream / explode::: |
| `ZombieExplode` | `die:::` @ 0x10012ecb8 | any death of a zombie (override) | - |
| `ZombieExplodeBrainsEaten` | `die:::` @ 0x10012ecb8 | same, after its brains were eaten | - |
| `Flattened` | `die:::` @ 0x10012ecb8 | death while doodle jumping (override) | - |
| `CutInHalfByCharlie` | `die:::` @ 0x10012ecb8 | death of a half pygmy (override) | - |
| `XenuPuppetShow` | `die:::` @ 0x10012ecb8<br>`checkForXenuDuringFall` @ 0x100041784 | half pygmy held by Xenu | PygmyFallIntoXenu |
| `OldFartPygmyHeartAttack` | `die:::` @ 0x10012ecb8<br>`fall:::` @ 0x100130fb8 | old fart after a tap heart attack (override / water) | TempleOldPygmyHeartAttackSplash |
| `AbsorbedByHalfPygmy` | `absorbedByHalfPygmy:` @ 0x10005abe8 | a falling half pygmy lands on it | PygmyAbsorbedByHalfPygmy |
| `AbsorbedByCharlie` | `checkToAbsorbPygmyDuringFall` @ 0x10005ad8c | the half pygmy absorbs Charlie (not in the controller list) | HalfPygmyAbsorbCharlie |
| `Skewered` | `hitByChainSpearComplete:` @ 0x10011c558 | chain spear | PygmyDie |
| `ZappedByLaserShark` | `hitBySharkLaser` @ 0x10011c5e8 | laser shark beam | PygmyHitBySharkLaser |
| `FedToLaserShark` | `eatenByLaserShark:` @ 0x10011c7ac | laser shark bite | PygmyEatenByLaserShark |
| `JellyfishScubaMask` | `suffocatedByJellyfish:` @ 0x10011c840 | jellyfish on the face | PygmySuffocatedByJellyfish |
| `StungByJellyfish` | `stungByJellyfish:` @ 0x10011c948 | jellyfish sting | PygmyStungByJellyfish |
| `EatenByJellyfish` | `checkJellyfishCollision` @ 0x10011cb20 | dropped onto a jellyfish | PygmyHitByJellyfish |
| `DrankByJellyfish` | `checkJellyfishCollision` @ 0x10011cb20 | dropped above a jellyfish | PygmyDrankByJellyfish |
| `ElectrocuteByJellyfish` | `checkJellyfishCollision` @ 0x10011cb20 | touches the circle leader | PygmyShockedByJellyfish |
| `HitByUnderwaterPowerCold` | `canBeTouchedByUnderwaterPower:` @ 0x10011d0d4 | underwater power 1 | PygmyHitByUnderwaterPowerCold |
| `HitByUnderwaterPowerHeat` | `canBeTouchedByUnderwaterPower:` @ 0x10011d0d4 | underwater power 2 | PygmyHitByUnderwaterPowerHeat |
| `HitByUnderwaterPowerOil` | `canBeTouchedByUnderwaterPower:` @ 0x10011d0d4 | underwater power 3 | PygmyHitByUnderwaterPowerOil |
| `HitByUnderwaterPowerAcid` | `canBeTouchedByUnderwaterPower:` @ 0x10011d0d4 | underwater power 4 | PygmyHitByUnderwaterPowerAcid |
| `HitByUnderwaterPowerAir` | `canBeTouchedByUnderwaterPower:` @ 0x10011d0d4 | underwater power 5 | PygmyHitByUnderwaterPowerAir |
| `FedToPiranha` | `eatenByPiranhaComplete:` @ 0x10011d808 | piranhas (bait master) | PygmyPiranhaAttackComplete |
| `PainDrainSpikes` | `dieFromPainDrain` @ 0x10011f758 | pain-drain mini-game | PygmyDie |
| `ImpregnatedByBlob` | `hitByNoPeeSeaBlob:` @ 0x10018a724 | No Pee sea blob | PygmyNoPeeHitSeaBlob |
| `Stoned` | `hitByNoPeeMine:` @ 0x10018a780 | No Pee mine | PygmyNoPeeHitMine |
| `EatenByClam` | `hitByNoPeeClam:` @ 0x10018a7dc | No Pee clam | PygmyNoPeeHitClam |
| `RunOutOfBreath` | `noPeeRunOutOfBreath` @ 0x10018a894 | No Pee air ran out | PygmyNoPeeHitPillar |
| `EatenByLocustQueen` | `eatenByLocustQueenComplete:` @ 0x100041fe0 | locust queen | PygmyDie |
| `SlicedByLocust` | `cutInAirByLocusts:` @ 0x100042ff4 | locusts cut it in the air | PygmyCutInAirByLocusts |
| `LocustFood` | `stripFleshByLocusts:` @ 0x100043814 | locusts strip its flesh | PygmyStripFleshByLocusts |
| `PopLocustBalloon` | `mouthFilledByLocusts:` @ 0x100044200 | locusts fill its mouth | PygmyMouthFilledByLocusts |
| `TurnedIntoMutantLocust` | `mutatedByLocusts:` @ 0x100044a50 | mutated into a locust | PygmyMutatedByLocusts |
| `SquashedLikeBug` | `touchBeganInside:allTouches:` @ 0x10013321c | tap a mutant pygmy | PygmyMutatedByLocustsSplat |
| `DanceDanceExecuted` | `danceDanceExecutedComplete:` @ 0x100045b14 | Dance Dance Execution | PygmyDie |
| `KilledByPurification` | `checkDiseaseIncubation:::` @ 0x100041d60 | purification incubation ran out | PygmyKilledByPurification |
| `PlagueBodyBurntOnFire` | `throwPlagueBodyOnPile:::` @ 0x100047c74 | plague body thrown on the lit bonfire | PygmyPlagueBodyBurnOnFire |
| `ExplodeFromThetanBirth` | `explodeFromThetanBirth` @ 0x100046d60 | thetan meter overload | PygmyDie |
| `BecameXenu` | `babySacrificeComplete` @ 0x100040bf4 | baby sacrificed by Tom | PygmyDie |
| `EatenByXenu` | `checkForXenuDuringFall` @ 0x100041784 | falls into Xenu slot 1 | PygmyFallIntoXenu |
| `XenuStripped` | `checkForXenuDuringFall` @ 0x100041784 | falls into Xenu slot 2 | PygmyFallIntoXenu |
| `ThetanPummeled` | `thetanBrawlGetPummeled` @ 0x10003fc14 | possessed pygmy pummels it | PygmyThetanBrawlGetPummeled |
| `BittenToDeath` | `defendAgainstThetanBrawl:` @ 0x10003f478<br>`thetanBrawlGetFaceEaten` @ 0x10003fc44 | possessed pygmy jumps it / eats its face | PygmyThetanBrawlGetJumped / ...GetFaceEaten |
| `BattleFatality` | `fatalityInBattle` @ 0x10012297c | Battle of the Gods fatality (recorded immediately) | PygmyBattleFatality<Realm> |
| `KonkeyDongBoulder` | `checkKonkeyDongCollision:` @ 0x100039028 | Konkey Dong boulder | PygmyKonkeyDongHit |
| `KonkeyDongBeetle` | `checkKonkeyDongCollision:` @ 0x100039028 | Konkey Dong beetle | PygmyKonkeyDongHit |
| `KonkeyDongFall` | `onGameFrame:` @ 0x100129980 | falls out of the Konkey Dong level | PygmyDie |
| `LavaRock` | `checkTheRunsCollision:` @ 0x100139998 | The Runs rock | PygmyTheRunsRockExplode |
| `LavaDrown` | `onGameFrame:` @ 0x100129980 | The Runs lava (y <= 13) | PygmyTheRunsSplash |
| `EatenByMoronTurtle` | `eatenByMoronTurtleComplete:` @ 0x1001137f8 | Moron Pests turtle | PygmyEatenByMoronTurtle |

### 8.3 Gifts

`[listener onPygmyGiftGiven:self giftId:]` → `onPygmyGiftGiven:giftId: @ 0x100164adc`:
`[IdolManager triggerEpisodeEventByGiftId:]`, `[tribe addGift:pygmyLogic:]` (only when the
pygmy has a persona), `mTotalGifts`, leaderboard `pg_totalgifts`; `getGiftLeaderboardId:
@ 0x1000735f0` returns nil for the 12 known gifts (`@"by undefined gift"` otherwise, never
reached).

| Gift | Given by |
|---|---|
| `TakeCrap` | `finishCrapping` (out-house) |
| `CatchFish` | `catchFish` (fishing bob) |
| `CatchSwordFish` | `catchSwordFish` |
| `Dance` | `startDancing:` (normal), `startVampireDancing` (non-zombie), `startStaringAtRainbow` |
| `ZombieDance` | `startDancing:` / `startVampireDancing` as a zombie |
| `Coconut` | `finishEatingCoconut:::` |
| `Banana` | `finishEatingBanana:::` |
| `Fish` / `CookedFish` | `finishEatingFish:::` (cooked or not) |
| `CookedBird` | `finishEatingBird:::` (only cooked birds) |
| `BecomeGhost` | `die:::` ghost conversion |
| `IntoTheLight` | `onMultiModelEvent:eventId:` (ghost leaves through the light) |

## 9. Main-class sequence items

(`die`, `explode`: §3.6; `drag`: §5.2; `slide`, `hangUpsideDown`: §4.4; the three
selection items: §7.4.)

### 9.1 `idle::: @ 0x10012eb58` (minTime, maxTime)

First time: `mStallTime = RAND100(minTime, maxTime)`, `ignoreGrav = 0`. Every frame: Charlie
with `checkForTigerBlood` YES, or `checkForShark` YES → 3. Else `mStallTime <= 0` → 2,
otherwise `mStallTime -= dt`, 0.

### 9.2 `walk::: @ 0x1001301e8` (minTime, maxTime, direction, endOfIsland)

First time: `mStallTime = RAND100(minTime, maxTime)`, side collision on, gravity on;
direction nil → the walk direction rule (§1.2); `"Right"` → right; any other string →
left. Speed `vel.x = ±50`, `xScale = ±1`. (`endOfIsland` is never read.)

Every frame, unless in story mode, the first interaction that fires returns 3, in this
order: Charlie `checkForTigerBlood`, `checkForCoconut`, `checkForIgloo`, `checkForWood`,
`checkForPlagueBody`, `checkForOutHouse`, `checkForBonFire`, `checkForFish`,
`checkForBird`, `checkForBanana`, `checkForFishingPole`, `checkForShark` (§10.1). Then
`r = mbIslandSideCollision ? 2 : 0`. When `mStallTime <= 0`: if the bonfire is enabled and
burning and `fire.x - 35 < x < fire.x + 35`, `mStallTime += RAND100(minTime, maxTime)` and
return `r` (never stop right next to the fire); otherwise `vel.x = 0`, `r = 2`. Then
`mStallTime -= dt`; return `r`.

### 9.3 `run::: @ 0x1001306bc` (minTime, maxTime) and `panicRun::: @ 0x10013089c` (minTime, totalTime, maxTime)

`run`: first time `mStallTime = RAND100`, side collision and gravity on, run toward
`islandCenterXPos` at ±100 (`x <= centre` → right). Side collision → 2; timeout →
`vel.x = 0`, 2; `mStallTime -= dt`.

`panicRun`: first time `mStallTime = totalTime`, `mNextTurnStallTime = RAND100(minTime,
maxTime)`, side collision and gravity on, run toward the centre at ±200. Every frame: side
collision or `mNextTurnStallTime <= 0` → turn around (`xScale` and `vel.x` negated) and
re-roll `mNextTurnStallTime`; `mStallTime <= 0` → `vel.x = 0`, 2, else 0; both timers
`-= dt`.

### 9.4 Small items

| Item | Behaviour |
|---|---|
| `disableLabels::: @ 0x10012ea44` | `mbDisableLabels = 1`; 2 |
| `flipXScale::: @ 0x10012ea5c` | `xScale = -xScale`; 2 |
| `resetVel::: @ 0x1001301c8` | `vel.xyz = 0`; 2 |
| `setOrigin::: @ 0x1001326a4` (zPos, xPos, yPos, inPlace) | without `inPlace`: `setOriginX/Y/ZPos:`; with it: `mLastPos -= (new - old origin) * scale` per axis, then `setOriginX/Y/ZPosInPlace:` (keeps the sprite where it is); 2 |
| `checkPygmyDialog::: @ 0x10012ff78` (alternateSequenceId, id) | dialog throttle over the global dictionary id → `CFAbsoluteTime`: drop entries older than 15 s, remember the newest start time; if a dialog started less than 3 s ago or `id` is still in the dictionary → `setBehavior:alternateSequenceId` (3); else store `id` = now, 2 |

Frame actions of the main class: `animChooseDirection @ 0x10012e920` (zombie walks only,
when `mbZombieChooseDirection`: the walk direction rule sets `xScale`),
`animFlipComplete @ 0x10012ea0c` (`mbGhostFlipping = 0`), and `cancelFlip @ 0x10012ea1c`
(method: if flipping, clear it and run `animGhostXFlip`).

### 9.5 `onMultiModelEvent:eventId: @ 0x100132e94`

Listener of the word balloon and the light: `WordBalloonCloseComplete` →
`closeWordBalloonNow`; `TheLightExit` → remove, release and shut down the light; if
`mbGoIntoTheLight` → `→ PygmyDie`, `gift IntoTheLight`.

### 9.6 `fall::: @ 0x100130fb8` (ignoreIsland)

The universal falling item (134 uses).

- First time: if not standing on the floor (`!mbIslandFloorCollision`) `mbOnIsland = 0`;
  side collision off, floor collision on (off when `ignoreIsland` is YES), gravity on,
  `falling = 1`, `mStartFallYPos = y`.
- Every frame, ocean level = 16 when `z >= -277.12` (island plane or nearer), else
  `mIslandYPos` (objects behind the island plane drown at island height).
- Catchers (→ `falling = 0`, return 3): unless beheaded by the squid, torn apart by the ape
  altar or in story mode, the first YES of `checkToAbsorbPygmyDuringFall`,
  `checkForKongStatueDuringFall`, `checkForApeWenchDuringFall`, `checkForApeBongoDuringFall`,
  `checkForApeAltarDuringFall`, `checkForGoGoCageDuringFall`,
  `checkForDanceDanceExecutionDuringFall`, `checkForIglooDuringFall`,
  `checkForZombieGraveDuringFall`, `checkForIdolPedestalDuringFall`,
  `checkForTempleDoorDuringFall`, `checkForApocalypseTempleDoorDuringFall`,
  `checkForApocalypseTimeTemplePlatformDuringFall`, `checkForLocustQueenDuringFall`,
  `checkForApocalypseTempleFountainDuringFall`, `checkForClosetTempleThetanMeterDuringFall`,
  `checkForClosetTempleCouchDuringFall`, `checkForClosetTempleClosetDoorDuringFall`,
  `checkForTomPygmyDuringFall`, `checkForXenuDuringFall`; or a cooked pygmy and
  `checkForIceHoleDuringFall` (§10.5, §11).
- Not on the floor this frame:
  - **Scream**: below the island top (`y < islandY`), moving down, not screamed yet,
    healthy, not beheaded / torn apart / story, and **no ocean**:
    `mbScreamDuringFall = 1`, `→ PygmyFallScream` (3).
  - **Splash**: ocean enabled and `y < level`: `stopSleeping`, doodle jumping →
    `onPygmyStopDoodleJump:`, `falling = 0`, `ignoreGrav = 1`, `mbDrowning = 1`,
    `vel.xy = 0`, `y = level`, `mTiltState = 0`, `zrot0`; story mode → 2; zombie →
    `zombieExplode` (3); heart attack → `sacrifice OldFartPygmyHeartAttack`,
    `TempleOldPygmyHeartAttackSplash`; cooked `PygmyCookedSplash`; plague body:
    dropped by a hurricane `PygmyPlagueBodyHurricaneSplash`, disappearing
    `PygmyPlagueBodySplashDisappear`, else `PygmyPlagueBodySplash`; cut `HalfPygmySplash`;
    T != 0 `TemplePygmySplash`; T 0: ate burnt food `PygmyChokeSplash`, beheaded
    `PygmyBeheadedSplash`, ate a sword fish `PygmyAteSwordFishSplash`, hit by an icicle
    `PygmyHitByIcicleSplash`, died in a thetan brawl `PygmyThetanBrawlSplash`, else
    `PygmySplash` (3). The sacrifice id already set (e.g. DroppedDrown, PushedDrown)
    stays.
  - Otherwise 0.
- On the floor (`mbIslandFloorCollision` set by the frame): doodle jumping →
  `onPygmyStopDoodleJump:`; `falling = 0`, `vel.x = 0`, `zrot0`; old fart (T 1, not a
  plague body): `start - y >= 50` → `mbTempleOldPygmyUpright = 0`,
  `TempleOldPygmyFallDown` (3), else `mbTempleOldPygmyUpright = 1`,
  `TempleOldPygmyLandOnFeet` (3); others 2 (the sequence continues with the landing
  animation).

### 9.7 `tumble::: @ 0x1001317ec` (flick flight)

- First time: T 4 → `episode FastPygmyFlick`; `[listener onPygmyTumble:self]` (moves this
  pygmy to the front of the pygmy array); side and floor collision off, gravity on,
  `falling = 1`, `mbTumble = 1`; 0.
- **Ice monster** (enabled and not hit yet): screen y of the pygmy
  `sy = (-277.12 / z) * y`, monster `m`. Mouth (when `isMouthCollisionActive`):
  `|x - m.x| < 50`, `|sy - mouthY| < mouthRadius` and `m.z + 100 < z < m.z + 300` →
  `mbHitIceMonster = 1`, `[m hitByPygmyInMouth:self]`, `sacrifice EatenByIceMonster`,
  `→ PygmyDie` (3). Body (`screenCollisionYPos/Radius`, same x and z window):
  `mbHitIceMonster = 1`, `[m hitByPygmy:self onFire:(mbIglooCannon || T != 4)]`; T 4 →
  `vel.xyz = 0`, `ignoreGrav = 1`, `sacrifice FastPygmyHitIceMonster`,
  `TempleFastPygmyExplodeOnIceMonster` (3); monster not recharging → bounce
  `vel.y = 1000`, `vel.z *= -0.5`, 0; recharging → `vel.xyz = 0`, `ignoreGrav = 1`,
  `sacrifice ExplodeOnIceMonster`, `PygmyExplodeOnIceMonster` (3).
- Passing the monster: first time `z < m.z + 100`: `mbFartherThanIceMonster = 1` and
  `layer<(monster display)` +labels (drawn behind it).
- At the apex (first frame with `vel.y <= 0`, `!mbDescending`): extrapolate
  `x' = 2x - tumblePos.x`, `z' = 2z - tumblePos.z`; volcano enabled, no eruption,
  `x' > 230` and `z' < -700` → into the crater: `pos = (270, max(y, 1000), -1200)`,
  `vel.x = vel.z = 0`, `mbHitVolcano = 1`, `mbDisableLabels = 1`, and the layer marker
  becomes `pygmyCloudBehindDisplayMarker`. `mbDescending = 1`, `layer<(marker)` (island
  marker normally) +labels; igloo cannon → `PygmyLeaveIglooTopExplodeFall` (3).
- `mbHitVolcano` is cancelled if an eruption is active.
- Ocean (enabled): level `islandY` (+100 when hitting the volcano); at or below it:
  `stopSleeping`, `ignoreGrav = 1`, `mbDrowning = 1`, `vel.xyz = 0`, `y = islandY`,
  `zrot0`. Not volcano: story → 2; `sacrifice IglooCannon` / `BounceOffIceMonster` /
  `FlickDrown`; `PygmySplash` (T 0) or `TemplePygmySplash` (3). Volcano:
  `y = islandY + 100`, `sacrifice FastPygmyHitVolano` (T 4) or `ThrownInVolcano`,
  `→ PygmyDie`, `[listener onVolcanoHit:self]` (3).
- Else 0. (The sequence `PygmyTumble` then runs `PygmySplashDistance`.)

## 10. Island category (`PygmyLogic(Island)`, 274 methods)

Addresses of every method are in §26 (items) and `tmp/re/pygmy-partition.json`; the most
important are repeated inline.

### 10.1 Island life: walk-time interactions

Called from `walk:::` in this order (§9.2); each returns YES after taking over. "Face" =
`xScale = (x - obj.x <= 0) ? 1 : -1`. Tween channel N = `[animation setTweenable:obj.display
channelId:N]` (the object follows the animation's attachment point N).

| Check | Condition | Effect |
|---|---|---|
| `checkForCoconut @ 0x1000bad1c` | `isCoconutEnabled`; first coconut with `isEdible` and `\|x - c.x\| <= 16` | face, `vel.xy = 0`, `mEatingCoconutLogic`, channel 14, `[c startEating:self]`, `→ PygmyEatCoconut` |
| `checkForIgloo` | igloo enabled, `!mbCrawledOutOfIgloo`, `[i isPygmyCrawlAvailable:self]`, `\|x - (i.x + 99)\| <= 4` | `vel.xy = 0`, `ignoreGrav = 1`, floor off, `mbRestingInIgloo`, `mIglooLogic`, `[i pygmyCrawlIntoIgloo:self]`, `sel0`, incubate (InfectIgloo / InfectPygmyFromIgloo), `→ PygmyCrawlIntoIgloo` |
| `checkForWood` | wood enabled; first `isAvailable` wood with `\|dx\| <= 16` | face, `vel.xy = 0`, `mCarryingWoodLogic`, channel 6, `[w startCarrying:self]`, `→ PygmyCarryWood` |
| `checkForPlagueBody` | bonfire enabled and burning; first pygmy with `isAvailablePlagueBodyToBeCarried` and `\|dx\| <= 16` | face, `vel.xy = 0`, `mCarryingPlagueBodyLogic`, channel 6 = body display, `[p startCarryingPlagueBody:self]`, incubate (InfectPlagueBody / InfectPygmyFromPlagueBody), behaviour by the body's T: `PygmyCarryPlagueBody`, `PygmyCarryOldPlagueBody`, `PygmyCarryBabyPlagueBody`, `PygmyCarryGingerPlagueBody`, `PygmyCarryFastPlagueBody` (T >= 5: none, still YES) |
| `checkForOutHouse` | `!mbFinishedCrapping`, out-house enabled, `o = outHouseLogic:` | if `[o isCrappingAvailable:self]` and `\|x - (o.x + 90)\| <= 4`: `vel.xy = 0`, `mbCrapping`, `ignoreGrav = 1`, floor off, `sel0`, `mCrappingOutHouseLogic`, `layer(pygmyIslandBackDisplayMarker)` +labels, incubate (InfectOutHouse / InfectPygmyFromOutHouse), `[o takeCrap:self]`, `→ PygmyTakeCrap`. Else walking left (`vel.x < 0`), `[o isWaitingAvailable:self]`, same distance: `vel.xy = 0`, `mWaitingOutHouseLogic`, `[o waitToCrap:self]`, `→ PygmyWaitToCrap` |
| `checkForBonFire` | bonfire enabled; left slot `[f isLeftWarmingAvailable:self]` and `\|x - (f.x - 35)\| <= 4` → `xScale = 1`, `mLeftWarmingBonFireLogic`, `[f startLeftWarming:self]`; else right slot at `f.x + 35` → `xScale = -1`, `mRightWarmingBonFireLogic`, `[f startRightWarming:self]` | `vel.xy = 0`, `→ PygmyWarmHands` |
| `checkForFish` | not inside the temples; `f = fishLogic:`, `isEdible`, `\|dx\| <= 16` → `vel.xy = 0`, `mEatingFishLogic`, channel 14, `[f startEating:self]`, `→ PygmyEatFish`. Else T 0 and the sword fish (`swordFishLogic:`) edible and near → `vel.xy = 0`, `mEatingSwordFishLogic`, `[s startEating:self]`, `→ PygmyEatSwordFish` (no tween, no facing) |
| `checkForBird` | not inside the temples; `b = birdLogic:`, edible, near | `vel.xy = 0`, `mEatingBirdLogic`, channel 14, `[b startEating:self]`, `→ PygmyEatBird` |
| `checkForBanana` | `b = bananaLogic:` (no enabled test); edible and near → `vel.xy = 0`, `mEatingBananaLogic`, channel 14, `[b startEating:self]`, `→ PygmyEatBanana`; for T 0 the nearest monkey (`\|m.x - x\|`, starting at 10000) with `canChasePygmy` gets `pygmyIsEatingBanana:self`. Else `isSlippable`, T 0, near → `vel.xy = 0`, `[b slipOnBanana:self]`, `→ PygmySlipOnBanana` |
| `checkForFishingPole` | `p = fishingPoleLogic:`, `isAvailable`. Ice fishing when the ice hole is enabled and `isAvailableForFishing` and `-100 <= p.x - h.x <= -40`: needs `\|x - (h.x - 65)\| <= 25` (else NO); `vel.xy = 0`, `mbIceFishing`, `mFishingPoleLogic`, `[p startIceFishing:self]`, incubate (InfectFishingPole / InfectPygmyFromFishingPole), `layer` after the hole display +labels, `ignoreGrav = 1`, floor off, `→ PygmyStartIceFishing`, `[h startFishing:self]`. Else `\|x - p.x\| <= 16`: `vel.xy = 0`, `mFishingPoleLogic`, pole display `insertBefore:` the pygmy +labels, `[p startFishing:self]`, incubate, channels 2/3/4 = pole, bob, string, `→ PygmyStartFishing` |
| `checkForShark` | another pygmy with a shark on its head within `\|x - o.x\| < 50` | `vel.x = 0`, `PygmyReactForward` if `(x - o.x) * xScale <= 0` else `PygmyReactBackward` |

`incubate (A / B)` = `incubateDisease:obj infectChallengeEvent:@"A"
infectedByChallengeEvent:@"B"` (§11.5).

Food items and helpers:

- `pickUpCoconut:::`, `pickUpBanana:::`, `pickUpSwordFish:::` → `[food pickUpForEating]`, 2.
  `pickUpBird:::` / `pickUpFish:::` → `pickUpForEating`; a burnt one with T 0 →
  `PygmyEatBirdChoke` / `PygmyEatFishChoke` (3), else 2.
- `finishEatingCoconut:::` / `finishEatingBanana:::`: if `(P != 1 || T != 0) && T != 1`:
  `gift Coconut` / `Banana` and `[food finishEating]`; always
  `clearTweenableChannelIdNow:14`, reference nil; 2.
- `finishEatingBird:::`: not burnt: Charlie (T 0), old fart or uncooked → nothing; else
  `gift CookedBird` + `finishEating`. Burnt: T 0 → `mbAteBurntFood = 1`,
  `sacrifice EatBurntBird`; `finishEating`. Always clear channel 14, nil; 2.
  `finishEatingFish:::`: like the bird but any non-burnt fish gives `CookedFish` or `Fish`
  (Charlie T 0 and old fart excepted); burnt → `EatBurntFish`.
- `animSwordFishSwallow` (frame action): `mbAteSwordFish = 1`. `finishEatingSwordFish:::`:
  `mbAteSwordFish = 1`, `sacrifice SwordSwallower`, `finishEating`, nil (no channel clear).
- `hideFood`: texture `BlankTexture` on whichever eaten food exists.
- `stopEating`: for each food: `clearTweenableChannelIdNow:14`, `[food dropWhileEating]`, nil.
- Wood: `pickUpWood:::` (incubate InfectWood / InfectPygmyFromWood, `[w pickUpForCarrying]`,
  2); `walkToBonFire:::` (first: face the fire, `vel.x = ±50`; done when within 50 of
  `fire.x` in the walking direction: `vel.x = 0`, 2); `animThrowWood` (clear channel 6,
  `[w throwOnPile]`, nil); `stopCarrying` (clear 6 now, `dropWhileCarrying`, nil);
  `stopCarryingFromWood` (same + `→ PygmyIdle`, called by WoodLogic).
- Fishing: `animInitBob` (`[pole pickupPole]`, `[[pole fishingBobLogic] initBobTexture]`),
  `pullPole` (`→ PygmyPullPole`), `catchFish` (`gift CatchFish`, `→ PygmyCatchFish`),
  `catchSwordFish` (`gift CatchSwordFish`), `catchShark` (`mbCatchShark = 1`,
  `sacrifice SharkFishingPole`, hide the apocalypse clouds of pygmy and pole,
  `→ PygmyCatchShark`), `walkToFishingPole:::` (first: gravity off, floor off,
  `vel = target - pos` with target `(p.x + 26.55, p.y + 8.05)` (reached in 1 s),
  facing by `vel.x`; continues while `y >= target.y`, then `vel.xy = 0`, 2),
  `walkToIceFishingPole:::` (same toward `(h.x - 67, h.y + 5)`, completes when
  `y > h.y + 5`: `vel.xy = 0`, `[h startFishingSitDown:self]`, 2), `startFishing:::` (pole
  busy → `[pole startFishingIdle]`, 2; free → reset to the ground, clear 2/3/4, nil,
  `→ PygmyIdle`, 3), `stopFishing` (pole: stop animation, gravity and floor on,
  `lastPos.y = islandY + 2`, `y = islandY - 1`, clear channels 2..4 or stop ice fishing
  (`[[listener iceHoleLogic:] stopFishing:self]`), `[pole dropWhileFishing]`, nil,
  `layer(pygmyDisplayMarker)` +labels), `stopFishingFromPole:` (called by the pole: also
  clears `mbCatchShark` with `zrot0`; `→ PygmyStopFishing` / `PygmyStopIceFishing`; the pole
  is not told to drop), `stopFishingFromIceHole:` (ice hole: `→ PygmyStopIceFishing`,
  `dropWhileFishing`).
- Out-house: `waitToCrap:::` (wait forever unless p0 gives a time; each frame
  `[o isCrappingAvailable:self]` → `vel.xy = 0`, `mbCrapping`, `sel0`, reference,
  `layer(pygmyIslandBackDisplayMarker)` +labels, `→ PygmyTakeCrapAfterWaiting` (3)),
  `takeCrap:::` (leave the queue, gravity off, floor off, layer, incubate,
  `[o takeCrap:self]`, 2), `finishCrapping` (called by the out-house: gravity and floor on,
  `sel1`, `mbFinishedCrapping`, `y = islandY`, `lastPos.y = islandY + 2`,
  `layer(pygmyDisplayMarker)` +labels, `gift TakeCrap`, `→ PygmyFinishCrapping`),
  `tossFromOutHouse` (waiting: leave queue + `→ PygmyIdle`; crapping: EJ with `vel.y = 200`,
  `layer(pygmyDisplayMarker)` +labels), `stopWaitingToCrap`, `resetCrapper:::`
  (`mbFinishedCrapping = 0`; the next walk may crap again), `explodeFromNotCrapping:::`
  (`sacrifice NotCrapping`, flag, `explode:::`).
- Bonfire: `stopWarmingByBonFire` (`stopLeftWarming:` / `stopRightWarming:`, nil),
  `tossFromBonFire` (same + `→ PygmyIdle`), `checkForNoBonFire:::` (fire enabled and not
  burning → `→ PygmyIdle`; 2), `jumpOnFire:::` (lava / bonfire burn run: clear both fire
  flags, `falling = 1`, `vel = (xScale < 0 ? -100 : 100, 600)`, gravity on, side collision
  off; 2).
- Igloo: `resetIgloo:::` (`mbCrawledOutOfIgloo = 0`), `leaveIglooDoorCrawl`
  (gravity on, floor on, `y = islandY`, `lastPos.y = islandY + 2`, `mbCrawledOutOfIgloo`,
  out, `sel1`, `→ PygmyCrawlOutOfIgloo`), `tossFromIgloo` (EJ with `vel = (200, 500)`),
  `leaveIglooDoorExplode` (`vel.x = 400`, `y = islandY`, `mbIglooRoll`,
  `sacrifice ShotOutIgloo`, `→ PygmyLeaveIglooDoorExplode`), `leaveIglooTopExplode`
  (`vel = (50, 1500, -300)` aimed at a non-dormant ice monster: `vel.x = m.x - x`;
  `mbIglooCannon`, `mTumblePos = pos`, `falling = 1`, `sacrifice ShotOutIgloo`,
  `→ PygmyLeaveIglooTopExplode`, flies through `tumble:::`), `iglooRoll:::` (first: side and
  floor off, gravity off; knocks over every pygmy with
  `canBeHitByPygmyIglooRoll:` within 40 (`hitByPygmyIglooRoll:`); past `islandMaxX` →
  `vel.y = 500`, gravity on, `→ PygmyFallIgnoreIsland` (3); else 0).

### 10.2 Night, sleep, dream bubble, sunset, dance, rainbow

- `checkForNight:::`: night (and not inside the temples): Charlie → `PygmyCharlieNoSleep`
  once (`mbCharlieNotSleeping`); others `mbSleeping = 1`, `→ PygmyFallAsleep`. Not night
  → `mbCharlieNotSleeping = 0`. Returns 2 in every case.
- `startSleeping:::`: `[listener onPygmySleepingStart:self]` (the controller starts
  `AllPygmiesSleepingStart` with the first sleeper); 2.
- `stopSleeping`: if sleeping: `stopMonkeyHumping`, `stopDreamBubble`, `mbSleeping = 0`,
  `onPygmySleepingStop:`.
- `checkForDay:::`: not night or inside the temples → `stopSleeping`, `→ PygmyWakeUp`; 2.
- `checkForDreamBubble:::`: `b = getLogic(2)`; dream bubbles enabled and `!b.isActive` →
  `mDreamBubbleLogic = b`, `[b activate:self]`; 2. `dreamBubbleSelected` (nil),
  `stopDreamBubble` (`[b deactivate]`, nil).
- `checkForSunset:::` / `checkForNoSunset:::`: sunset (outside the temples) → S13,
  `→ PygmySunset`; end of sunset → `stopSleeping`, `mbCharlieNotSleeping = 0`,
  `→ PygmyIdle`.
- Monkeys humping sleepers: `canBeHumpedByMonkey:` (sleeping and no humper),
  `humpedByMonkey:` (`PygmyHumpedByMonkeyFront` if `xScale * (m.x - x) <= 0` else
  `Behind`), `unhumpedByMonkey:` (`→ PygmyAsleep`), `animHitHumpingMonkey`
  (`[m punchedWhileHumping]`), `stopMonkeyHumping` (`[m stopHumpingPygmy]`).
- `checkForDance:::`: `isDanceActive` → `startDancing:` with the skin-pack option
  "Pygmy Theme Song" of `Dance3`/`Pygmies` ("Pygmy Dance") active; 2.
- `canDoNormalDance` = guard `FREE −{T} +{sharkOnHead}`, tilt 0.
- `startDancing: @ 0x1000ac0e0` (themeSongBackup): `mbIsThemeSongBackup = arg`. Guard
  `FREE −{T, zombie} +{sharkOnHead}`, tilt 0: S13, `vel.xy = 0`, `zrot0`, tilt 0,
  `xScale = 1`; T 0: non-zombie `gift Dance` + (`PygmyStartDancingThemeSongBackup` when
  arg, else `PygmyStartDancing`), zombie `gift ZombieDance` + `PygmyStartZombieDancing`;
  T != 0 → `TemplePygmyStartDancing`. Guard fails while in the go-go cage →
  `PygmyRefreshGoGoCage`.
- `startVampireDancing` (guard without the zombie exception): non-zombie `gift Dance`,
  `PygmyStartVampireDancing`; zombie `xScale = 1`, `gift ZombieDance`,
  `PygmyStartZombieDancing`.
- `stopDancing:updateSong:themeSong:napoleonDynamite:fishmas:fishmasTwelveDays:
  @ 0x1000ad2b8` (six BOOLs): guard as startDancing: S13, tilt 0, `vel.xy = 0`, `zrot0`;
  T 0 non-zombie: first true flag → `mbStoppingDance = 1` and
  `PygmyStopPantsDancing` / `PygmyStopUpdateSongDancing` /
  (`PygmyStopThemeSongDancingBackup` if the backup flag else `PygmyStopThemeSongDancing`) /
  `PygmyStopNapoleonDancing` / `PygmyStopFishmasDancing` / `PygmyStopTwelveDaysDancing`;
  none → `PygmyStopDancing`. Zombie → `PygmyStopZombieDancing`; T != 0 →
  `TemplePygmyStopDancing`.
- `stopDanceComplete:::` (`mbStoppingDance = 1`, 2); `stopDancingNow` (if stopping:
  clear, `TemplePygmyIdleLoop` for T != 0 else `PygmyIdle`); `checkDanceChange:::` (first:
  remember the "Pygmy Dance" option value; later a different value →
  `PygmyRefreshGoGoCage` (3), else 0).
- Rainbow: `checkForRainbow:::` (`[[listener rainbowLogic:] isRainbowActive]` outside the
  temples → `startStaringAtRainbow`; 2). `startStaringAtRainbow @ 0x1000adcd4`: guard
  `FREE +{sharkOnHead}`, tilt 0: S12 without the rainbow stop, `mbStaringAtRainbow`,
  tilt 0, `zrot0`, `xScale = 1`, `gift Dance`, `→ PygmyStartStaringAtRainbow`.
  `startStaringAtRainbow:::` (`vel.xy = 0`, 2), `isStaringAtRainbow` (getter), `gushOverRainbow` (`mbGushingOverRainbow`,
  `→ PygmyGushOverRainbow`), `chooseRainbowAnimation:::` (comma list → global pool,
  refilled when empty; random entry; episode event per animation:
  `PygmyDoubleRainbow1Anim` DoubleRainbowAllTheWay, `2` DoubleRainbowSoIntense,
  `3` DoubleRainbowAllTheWayAcrossTheSky, `4` DoubleRainbowOhMyGod,
  `5` DoubleRainbowWhatDoesThisMean, `6` DoubleRainbowTripleRainbow; animation listener =
  self; play; remove from the pool; 2), `gushOverRainbowComplete:::` /
  `stopStaringAtRainbow` (stop the six rainbow sounds `TripleRainbow, WhoaOhMyGod,
  WhatDoesThisMean, SoIntense, DoubleRainbowAllTheWay, AcrossTheSky` and tell the rainbow
  `gushOverRainbowComplete:self`; `→ PygmyStopStaringAtRainbow`).
- Konkey Dong kidnapping (a pygmy watching another being kidnapped):
  `startStaringAtKidnapping` (guard, S12 without kidnapping stop, `mbStaringAtKidnapping`,
  tilt 0, `vel.xy = 0`, `zrot0`, face Kong, `→ PygmyStartStaringAtKidnapping`),
  `canChaseKonkeyDong`, `chaseKonkeyDong` (`→ PygmyChaseKonkeyDong`),
  `stopStaringAtKidnapping` (`→ PygmyStopStaringAtKidnapping`), `stopStaringAtKidnapping:::`
  and `chaseKonkeyDongComplete:::` (items), `kidnappedByKonkeyDong`
  (`mbKidnappedByKonkeyDong = 1`), `kidnappedByKonkeyDongComplete`
  (`sacrifice KidnappedByKonkeyDong`, `→ PygmyKidnappedByKonkeyDong`).

### 10.3 Hazards (called by other logic objects)

Unless stated, each first checks its guard (§25), returns NO / does nothing when busy,
otherwise runs the stop list, sets its flag, `mTiltState = 0`, `zrot0`, and switches
behaviour. "→ (Asleep)" means `stopMonkeyHumping` + `PygmyHitByXAsleep` when sleeping,
else `PygmyHitByX` (S12 keeps the pygmy asleep).

| Method | Guard (§25) | Effect |
|---|---|---|
| `throwFromExplosion: @ 0x10009731c` (VECTOR4 *centre) | FREE +{mbWarlockSkipNextExplosion} | S14, `falling = 1`, tilt 0, `zrot0`; `dx = centre.x - x`, `e = expf(-\|dx / 200\|)`, `vel = (e * 100, e * 1000)`; `dx <= 0` → `xScale = -1` (vel.x stays positive), else `xScale = 1`, `vel.x = -vel.x`; side collision off; `sacrifice ThrownExplosionDrown`; `→ PygmyThrowFromExplosion`. Always clears `mbWarlockSkipNextExplosion` at the end. Sent to every pygmy by the controller's `onPygmyExplode:` |
| `canBeHitByLightning @ 0x100098630`, `hitByLightning @ 0x100097c2c` → BOOL | blocked by FREE −{hurricane x2, Kong statue, apes x3, go-go cage, DDE, zombie} | a zombie (not ghost) → `zombieExplode`, YES. S14; earthquake → `mbEarthquake = 0` + `onPygmyEarthquakeStop:`; hurricane-drop + `[listener onPygmyHitByLightningInHurricane:self]` (ChallengeManager game 1); `mbHitByLightning`, tilt 0, `vel.xy = 0`, `zrot0`, `sacrifice Lightning`; in the Kong statue → `[k removePygmyFromKongStatue:self]`, nil, `PygmyHitByLightningInKongStatue`; in ape wench / bongo / altar → remove from each, `PygmyHitByLightningInApe…`; carried by a hurricane (flag still set) → `sacrifice HurricaneLightning`, `PygmyHitByLightningInHurricane`; else `PygmyHitByLightning`. The pygmy keeps `mbPickedUpByHurricane` (the hurricane dropped it silently) |
| `explodeFromLovingPocketGod @ 0x100098c30` | FREE −{hurricane x2} | `stopStaringAtRainbow`, `mbExplodeFromLovingPocketGod`, `→ PygmyExplodeFromLovingPocketGod` |
| `explodeFromLovingPocketGod:::` (item) | - | `stopBeingHighOnNitrous` + S13 (without nitrous, with `stopBeingEaten`), hurricane-drop (clears `mbPickedUpByHurricane`), `ignoreGrav = 1`, flag, tilt 0, `vel.xy = 0`, `zrot0`, `sacrifice LovingPocketGod`, then `explode:::` |
| `hitByLava: @ 0x1000992dc` | FREE | S13 with `stopBeingEaten` (no closeWordBalloon), `mbOnFireFromLava`, `sacrifice Lava`, `→ PygmyHitByLava` (no velocity change) |
| `hitByIcicle: @ 0x100099aec` → BOOL | FREE | S14, `mbHitByIcicle`, `vel.xy = 0`, `sacrifice IciclePinCushion`, `→ PygmyHitByIcicle` |
| `hitByBonFire: @ 0x10009a328` | FREE | S14, `mbOnFireFromBonFire`, `sacrifice BonFire`, `→ PygmyHitByBonFire` |
| `hitByCoconut: @ 0x10009ab4c` → BOOL | FREE −{T} +{sharkOnHead}, tilt 0 | S12, `vel.xy = 0`; T 0 → (Asleep) `PygmyHitByCoconut`; T != 0 → `TemplePygmyHitByCoconut` (no sacrifice: a knock-out) |
| `hitByBirdPoo:`, `hitByFish:`, `hitBySwordFish:`, `hitByBanana:`, `hitByCookedBird:`, `hitByCookedPygmy:` → BOOL | FREE +{sharkOnHead}, tilt 0 | S12, `vel.xy = 0`, (Asleep) `PygmyHitByBirdPoo` / `…Fish` / `…SwordFish` / `…Banana` / `…CookedBird` / `…CookedPygmy` |
| `hitBySnowBall: @ 0x10009bc0c` → BOOL | FREE −{touchBeganInside, dragging} +{sharkOnHead}, tilt 0 | if dragged: drag-end with `zrot0`, `layer(pygmyDisplayMarker)` +labels, floor on, `y = islandY`; `mbTouchBeganInside = 0`, touch nil; S12; `mbHitBySnowBall`, `vel.xy = 0`, `sacrifice HitBySnowBall`, `→ PygmyHitBySnowBall` |
| `hitByBoulder:xVel: @ 0x10009ef00`, `hitByTRexEgg:xVel:`, `hitByShark:xVel:` → BOOL | FREE, tilt 0 | S14, `falling = 1`, `vel = (xVel, 500)`, `sacrifice BoulderDrown` / `TRexEggDrown` / `SharkDrown`, `→ PygmyHitByBoulder` / `PygmyHitByTRexEgg` / `PygmyHitByShark` |
| `canBeHitByPygmyIglooRoll:`, `hitByPygmyIglooRoll:` | FREE, tilt 0 (only the can-method checks) | S14, `falling = 1`, `vel = (400, 500)`, `sacrifice IglooPygmy`, `→ PygmyHitByPygmyIglooRoll` |
| `crushedByBoulder:`, `crushedByTRexEgg:` → BOOL | FREE +{sharkOnHead}, tilt 0 | S13, `mbCrushedByBoulder` (egg: 0), `mbCrushed`, `vel.xy = 0`, `sel0`, `→ PygmyCrushedByBoulder` / `PygmyCrushedByTRexEgg`, then `[[group getDisplayObject:@"Boulder" / @"TRexEgg"] insertAfter:pygmyDisplay]` +labels (the pygmy is re-parented right after the boulder / egg) |
| `uncrushedByBoulder:dontWait:`, `uncrushedByTRexEgg:dontWait:` | crushed | `dontWait` → `mbCrushed = 0`; `sel1`, `layer(pygmyDisplayMarker)` +labels, `→ PygmyUncrushedByBoulder` / `…TRexEgg`. `uncrushed:::` item clears `mbCrushed` |
| `eatenByTRex: @ 0x1000a2e98` → BOOL | FREE −{T} +{sharkOnHead}, tilt 0, T ∈ {0, 2} | S13, `mbEatenByTRex`, `ignoreGrav = 1`, `vel.xy = 0`, `sel0`, `TempleBabyPygmyEatenByTRex` (T 2) / `PygmyEatenByTRex`, +labels |
| `uneatenByTRex:` / `swallowedByTRex:` | eaten | release: gravity on, `zrot0`, `sel1`, `layer<(pygmyDisplayMarker)` +labels, `TempleBabyPygmyUneatenByTRex` / `PygmyUneatenByTRex`; swallow: `mbDisableLabels`, `sacrifice BabyPygmyEatenByTrex` / `EatenByTRex`, `→ PygmyDie` |
| `eatenBySpider:`, `uneatenBySpider:`, `swallowedBySpider:` | as T-Rex (any T) | `PygmyEatenBySpider`, `PygmyUneatenBySpider`, `sacrifice EatenBySpider` + `→ PygmyDie` |
| `canBeBurntByMagnifyingGlass:`, `burntByMagnifyingGlass: @ 0x1000a4a40`, `unburntByMagnifyingGlass:` | FREE +{sharkOnHead}, tilt 0 | S13, `mbBurntByMagnifyingGlass`, `mBurningMagnifyingGlassLogic`, `vel.xy = 0`, `sel0`, `sacrifice MagnifyingGlass`, `→ PygmyBurntByMagnifyingGlass`; un-burn: clear, `sel1`, `→ PygmyUnburntByMagnifyingGlass` |
| `canBeSwarmedByAntHill:`, `swarmedByAntHill: @ 0x1000a6a10` | FREE −{piranha, drain, pain drain} +{sharkOnHead}, tilt 0 | S13, `mbSwarmed`, `vel.xy = 0`, `sel0`, `sacrifice FireAnts`, `→ PygmySwarmedByAntHill` |
| `canFallIntoCrackHole:`, `fallIntoCrackHole:` | FREE +{sharkOnHead}, tilt 0 (can only) | S13, `mbInCrackHole`, `vel.xy = 0`, `sel0`, `sacrifice CrackHole`, `layer<(pygmyDisplayMarker)`, `→ PygmyFallIntoCrackHole`; the item `onFallIntoCrackHoleComplete:::` calls `[listener onPygmyFallIntoCrackHoleComplete:self]` (may start The Runs, §18) |
| `blownByHurricane: @ 0x1000a8c50` → BOOL | FREE −{pickedUp} +{sharkOnHead}, tilt 0 | S13, `mbBlownByHurricane`, `vel.xy = 0`, `sel0`, `→ PygmyBlownByHurricane`. `unblownByHurricane:` (blown and not struck): clear, `sel1`, `→ PygmyUnblownByHurricane` |
| `pickedUpByHurricane: @ 0x1000a9518` → BOOL | zombie → `zombieExplode`, NO. FREE −{cut, T, blown, trapped grave/pedestal, zombie, cooked, plague} +{sharkOnHead}, a trapped grave pygmy only if `![grave isPygmyAboutToBecomeZombie]`, tilt 0 | S15, `ignoreGrav = 1`, `mbBlownByHurricane = 0`, `mbPickedUpByHurricane`, `vel.xy = 0`, reference, `sel0`; cooked `PygmyCookedPickedUpByHurricane`, plague `PygmyPlagueBodyPickedUpByHurricane`, cut `HalfPygmyPickedUpByHurricane`, T 0 `PygmyPickedUpByHurricane`, else `TemplePygmyPickedUpByHurricane`; YES |
| `droppedByHurricane:` | picked up, not catching a shark, not struck | `falling = 1`, gravity on, both hurricane flags 0, `mbDroppedByHurricane`, nil, `sel1`, `z < -277.12` → `layer<(pygmyIslandDisplayMarker)` +labels; cooked / plague / cut / T != 0 variants (`PygmyCookedDroppedByHurricane`, `PygmyPlagueBodyDroppedByHurricane`, `HalfPygmyDroppedByHurricane`, `TemplePygmyDroppedByHurricane`); normal `sacrifice HurricaneDrown`, `→ PygmyDroppedByHurricane` |
| `caughtByShark:` | - | `sacrifice BabyPygmyEatenByShark` (T 2) / `FedToShark`, `→ PygmyCaughtByShark` |
| `eatenByShark: @ 0x1000aa0cc` → BOOL, `uneatenByShark:` | FREE +{sharkOnHead, sleeping}, tilt 0 | S13, `mbSharkOnHead`, `vel.xy = 0`, `mEatingSharkLogic`, `PygmyCharlieEatenByShark` (P 1) / `PygmyEatenByShark`, `layer<(pygmySmallObjectsDisplayMarker)` +labels; release: clear, nil, `→ PygmyUneatenByShark`. `hasSharkOnHead` getter; `stopBeingEaten` (`[shark letGoOfPygmy]`); item `sharkSwallow:::` (`[shark swallowPygmy]`, nil, `sacrifice SharkOnHead`, `→ PygmyDie`); `watchShark:::` (another pygmy's shark: `PygmyAlreadyReactForward/Backward`, 0; none: 2) |
| `canBeCaughtByBird:`, `caughtByBird: @ 0x1000ab0a4` | FREE +{sharkOnHead, sleeping}, tilt 0 | S13, `mbCaughtByBird`, `ignoreGrav = 1`, `vel.xy = 0`, `mCarryingBirdLogic`, `sel0`, `→ PygmyCaughtByBird`. `caughtByBirdComplete:` (nil, `sacrifice CaughtByBird`, `→ PygmyDie`); `droppedByBird:` (clear, gravity on, nil, scale (1,1), `sel1`, `→ PygmyFall`) |
| `canBeBitByBat` (= sleeping), `bitByBat:` | - | S13, `mbBitByBat`, `vel.xy = 0`, `→ PygmyBitByBat` (becomes a vampire, §10.8) |
| `canCatchSpear`, `catchSpear: @ 0x1000a5a10` | FREE +{sharkOnHead, throwingSpear}, tilt 0 | S12 without the spear stop, `mbThrowingSpear`, `mThrowingSpearLogic`, spear on channel 5, `vel.xy = 0`, faces the active T-Rex (else the spider), `→ PygmyCatchSpear`. `animThrowSpear` (clear flag and channel 5, `[spear throwSpear]`, nil); `stopThrowingSpear` (`[spear dropWhileHolding]`) |
| `monkeyOnBack: @ 0x1000aec6c` → BOOL | FREE +{sharkOnHead, sleeping}, `mCharlieOnBackMonkeyLogic == nil`, tilt 0 | S13, `vel.xy = 0`; Charlie → `mCharlieOnBackMonkeyLogic`, `PygmyCharlieMonkeyOnBack`; else `mbMonkeyOnBack`, `sacrifice MonkeyOnBack`, `PygmyMonkeyOnBack`, `layer<(pygmySmallObjectsDisplayMarker)` +labels |
| `canBeCaughtByMonkey:` (requires falling and `vel.y < 0`), `canBeChasedByMonkey:`, `caughtByMonkey:` | FREE (−falling, +sharkOnHead) | S14, `vel.xy = 0`, `ignoreGrav = 1`, floor off, `sel0`, `mbCaughtByMonkey`, `mbPlayingCatchWithMonkey`, `mThrowingMonkeyLogic`, `→ PygmyCaughtByMonkey`, every monkey `startPlayingCatch` |
| `thrownByMonkey:toMonkey:` | caught | `falling = 1`, gravity and floor on, `vel = (xScale * 200, 700)`, with a target monkey `vel.x = (t.x - x) / (2 * 700 / 3000)`; `zrot0`, clear, `sel1`, `sacrifice MonkeyThrow`, `→ PygmyThrownByMonkey`. `stopPlayingCatchWithMonkey`: every monkey `stopPlayingCatch` |
| `eatenByMoronSnake: @ 0x1000b07c4` → BOOL | FREE −{T} +{sharkOnHead, sleeping}, tilt 0, T ∈ {0, 2} | S13, `vel.xy = 0`, `mbEatenByMoronSnake`, `sel0`, `sacrifice EatenByMoronSnake`, `TempleBabyPygmyEatenByMoronSnake` / `PygmyEatenByMoronSnake`, snake display `insertAfter:` the pygmy +labels. `animBabyThrowSnake` (baby escapes: `layer(pygmyDisplayMarker)`, `sel1`, flag 0, `[snake thrownByBaby:self]`); `eatenByMoronSnakeComplete:::` (flag → sacrifice + `PygmyEatenByMoronSnakeComplete`; `[snake eatPygmyComplete:self]`; Moron Pests enabled → `[snake startMoronPests:self]`; 2) |
| `wearWaterWings: @ 0x1000b1210` → BOOL | FREE −{T} +{sharkOnHead, sleeping}, tilt 0, T ∈ {0, 2} | S13, `vel.xy = 0`, `ignoreGrav = 1`, side and floor off, `mbWearingWaterWings`, `sacrifice WaterWingsDrowned`, `sel0`, `→ PygmyWearWaterWings`, wings display `insertAfter:` +labels. `ejectFromWaterWings`: `y += 20`, EJ-like with `vel = (-200, 500)`, `layer<(pygmyDisplayMarker)` +labels, no sacrifice change |

### 10.4 T-Rex and spider avoidance, spider tree

- `checkForTRex:::` / `checkForSpider:::`: when
  the monster is active: S12′, roll a safe zone and a run zone, face the monster
  (`t <= x` → `xScale = -1`), `→ PygmyStayAwayFromTRex` / `PygmyStayAwayFromSpider`.
  Always 2. Zones (`/100`): T-Rex `safeMin = r%3642 + 6819`, `safeMax = r%3641 + 35940`,
  `runMin = r%9101 + 10460`, `runMax = r%9101 + 26840`; spider `safeMin = r%4161 + 5280`,
  `safeMax = r%4161 + 38560`, `runMin = r%10401 + 9440`, `runMax = r%10402 + 28159`.
- `checkStayAwayFromTRex:::` / `checkStayAwayFromSpider:::`: monster gone → `→ PygmyIdle`. `c` = island
  centre, `t` = monster x. Both on the same side of the centre → face away from it and
  `→ PygmyRunUnderTRex` / `PygmyRunUnderSpider`. Else safe (`(c <= t || x >= safeMax) &&
  (t <= c || x <= safeMin)`) → 2; else `→ PygmyStepAwayFromTRex` / `…Spider`.
- `runUnderTRex:::` / `runUnderSpider:::`: first: side collision on, gravity on. Each
  frame: run at ±200 away from the monster's side of the centre; done when past the run
  zone (`t < c && x > runMax` or `c < t && x < runMin`): `vel.x = 0`, face the monster, 2.
- `checkSpiderTreeCollision`: tree available and `(x - (s.x + 130))^2 + (y - (s.y + 110))^2
  < 900` → touch released, `mbTrappedInSpiderWeb`, `vel.xy = 0`, `ignoreGrav = 1`, floor off,
  `sel0`, `layer<(pygmyIslandBackDisplayMarker)` +labels, `→ PygmyStuckInWeb`,
  `[s trapInWeb:self]`, YES. `ejectFromSpiderTree` (y + 20, `vel = (-200, 500)`, EJ without
  sacrifice), `eatenByLittleSpiders:` (`sacrifice EatenByLittleSpiders`, `→ PygmyDie`).

### 10.5 Containers: catching a falling pygmy and ejecting it

Fall catchers are polled by `fall:::` (§9.6) in the listed order. Base filter **BG**:
not cooked, not a plague body, not cut, T 0, not doodle jumping, not dropped by a
hurricane, not thrown by locusts, and (except the zombie grave) `!mbInvalidSacrifice`.
All require `vel.y < 0` and use inclusive boxes. On catch: `vel.xy = 0`, `ignoreGrav = 1`,
floor collision off, flag and reference set, the container's `…:self` call, behaviour,
return YES.

| Catcher (Island unless noted) | Filter | Box (relative to the container) | Container call → behaviour |
|---|---|---|---|
| `checkForKongStatueDuringFall` | BG, `isKongStatueEnabled`, `isAvailable` | `\|(x - k.x) + 2\| <= 20`, `\|y - (k.y + 23)\| <= 20` | `pygmyFallIntoKongStatue:`; Kong display `insertAfter:` +labels; `PygmyFallIntoKongStatue` |
| `checkForApeWenchDuringFall`, `…ApeBongo…`, `…ApeAltar…` | BG, enabled, available | `\|dx\| <= 20`, `-15 <= y - a.y <= 0` | `pygmyFallIntoApeWench:` / `ApeBongo:` / `ApeAltar:`; incubate (InfectApeWinch / InfectPygmyFromApeWinch, InfectApeBongos / InfectPygmyFromApeBongos, InfectApeAltar / InfectPygmyFromApeAltar); ape display `insertAfter:` +labels; `PygmyFallIntoApeWench` / `…ApeBongo` / `…ApeAltar` |
| `checkForIglooDuringFall` | BG, enabled, `[i isPygmyFallAvailable:self]` | `\|dx\| <= 30`, `\|y - (i.y + 48)\| <= 30` | `mbRestingInIgloo`, `pygmyFallIntoIgloo:`, `sel0`, incubate (InfectIgloo), `PygmyFallIntoIgloo` |
| `checkForZombieGraveDuringFall` | BG without the invalid test, `isOpenAndUnoccupied` | `\|dx\| <= 30`, `\|y - (g.y + 30)\| <= 30` | `mbTrappedInZombieGrave`, `fallIntoGrave:`, `sel0`, `PygmyFallIntoGrave` |
| `checkForIdolPedestalDuringFall` | BG, enabled; every pedestal with `isAvailable` | `\|dx\| <= 20`, `\|y - (p.y + 43)\| <= 20` | `mbTrappedInIdolPedestal`, `fallIntoIdolPedestal:`, `PygmyFallIntoIdolPedestal` (stays selectable) |
| `checkForTempleDoorDuringFall` | BG, enabled, `isAvailable` | `\|x - (d.x + 90)\| <= 30`, `\|y - (d.y + 10)\| <= 30` | tilt 0, `mbInsideTempleDoor`, `fallIntoTempleDoor:`, `sel0`, `PygmyFallIntoTempleDoor` (Battle of the Gods entrance) |
| `checkForApocalypseTempleDoorDuringFall` | not cooked/plague/cut/doodle/invalid/dropped/thrown, no door yet (any T); doors tried in order: apocalypse temple door, portal door, then the time, locust, dance, plague, purification, closet and Xenu temple door arrays | per door `checkForApocalypseTempleDoorDuringFall:`: `isAvailable`, `\|dx\| <= 30`, `\|y - (door.y + 50)\| <= 30` | tilt 0, `mbInsideApocalypseTempleDoor`, `fallIntoApocalypseTempleDoor:`, `sel0`, `PygmyFallIntoApocalypseTempleDoor` (T 0) / `TemplePygmyFallIntoApocalypseTempleDoor` |
| `checkForIceHoleDuringFall` | **cooked** (burnt) pygmy, not plague/cut, T 0, not invalid/doodle/dropped/thrown; hole enabled and `isAvailableForFalling` | `\|dx\| <= 30`, `\|y - (h.y + 30)\| <= 10` | `mIceHoleLogic`, `burntPygmyFallIntoIceHole:`, `PygmyBurntFallIntoIceHole`; `iceHoleSplashComplete` (nil, `→ PygmyDie`) |
| Apocalypse catchers | §11 | | |

Ejecting (all called by the container or by `die:::`): each checks its flag or reference,
clears both, and uses the EJ block (§1.3) with the given velocity:

| Method | Velocity / extras |
|---|---|
| `tossFromKongStatue` | `(200, 500)`, also clears `mbKidnappedByKonkeyDong`; `layer(pygmyDisplayMarker)` +labels |
| `tossFromApeWench`, `tossFromApeAltar` / `tossFromApeBongo` | `(200, 500)` / `(-200, 500)`; layer +labels |
| `tornApartFromApeAltar` | `vel.x = (int)(r % 4001 - 2000) / 10`, `vel.y = 500`, floor **off**, `layer(pygmyDragDisplayMarker)` +labels, `sacrifice ApeAltar`, `mbTornApartByApeAltar`, `→ PygmyTornApartFromApeAltar` (`mbInApeAltar` stays set) |
| `grabbedFromApeAltar` / `apeSmashOnApeWench` / `apeSmashOnApeBongo` | `sacrifice ApeEaten` / `ApeWenchSmash` / `ApeBongoSmash`, `sel0`, `→ PygmyDie` |
| `apeOwnsPygmy` | `sel0` |
| `tossFromIgloo` | `(200, 500)` |
| `ejectFromZombieGrave` | `y += 20`, `(-200, 500)` |
| `ejectFromIdolPedestal` | as the grave, no selectable change |
| `ejectFromTempleDoor` | clears `mbBattleOfTheGods`, `mIslandYPos = 86`, `pos = (440, 100)`, `(-200, 500)`, clears tween channels 7..13 (battle objects) |
| `stopBeingTrappedInZombieGrave`, `stopBeingTrappedInIdolPedestal`, `stopBeingInsideTempleDoor` | tell the container `ejectFromGrave:` / `ejectFromIdolPedestal:` / `ejectFromTempleDoor:`, nil |
| `checkForZombieGrave` (walking) | not cooked/plague/cut, T 0, grave enabled and `isOpenAndUnoccupied`, `\|x - g.x\| <= 30` → `vel.xy = 0`, gravity off, floor off, `mbTrappedInZombieGrave`, `[g loseBalanceByGrave:self]`, `sel0`, `PygmyPushedIntoGraveBackward` when facing away from the grave, else `…Forward`; YES |

### 10.6 Sun, ice and the ice hole

- `checkForSunState:::` (loop item, p0 = "keep looping"): inside the temples → 2. Sun state
  `s = [listener sunState]` (0 at night): `s != 0`: S13,
  `mSunBurnFreezeCycleCount = r % 3 + 2`; 2 → `PygmySunFreeze`, 1 → `PygmySunBurn`.
  Returns 0 when p0 is YES, else 2.
- `checkForSunBurnt:::`: s 2 → new count, `→ PygmySunFreeze`; s 1: `ok` unless (ice hole
  enabled, frozen and `[hole shouldPygmyWalkTowardIceHole:self]`) — standing at
  `290 < x < 350` is always ok; decrement the count; while cycles remain (or not ok) → 2;
  else `mbSunBurnt`, `vel.x = 0`, `→ PygmySunBurnt`. s 0 → `→ PygmyIdle`.
- `checkForSunFrozen:::`: s 2: decrement; cycles remain → 2; else `mbSunFrozen`,
  `vel.x = 0`, `→ PygmySunFrozen`. s 1 → new count, `→ PygmySunBurn`. s 0 → `PygmyIdle`.
- `sunWalk:::` (min, max): `towardHole` = sun burning, hole enabled and frozen, and the hole
  wants the pygmy. First: timer, side collision and gravity on; direction: not toward
  the hole → coin; toward the hole: `x <= 290` → right, `x >= 350` → left, between →
  coin. Speed ±50. Done on side collision, on reaching 290..350 while walking toward
  the hole, or on timeout.
- `sunBurntComplete:::`: `sacrifice GlobalWarming`; on the ice island (L 4): hole enabled
  and frozen and `290 <= x <= 350` → `mIceHoleLogic`, `[h meltIceHole:self]`; else
  `→ PygmySunBurntCreateSoloIceHole`. Elsewhere `→ PygmySunBurntExplode`. 2.
  `meltIceHoleComplete:` (nil, `→ PygmyDie`).
- `canMeltIceHole` (asked by `IceHoleLogic::shouldPygmyWalkTowardIceHole:`) returns **YES
  when the pygmy is busy** (any of `FREE +{sharkOnHead}` set or tilted) and NO when it is
  free. Keep the inverted result: the ice hole consumes it as is.
- `iceShatter @ 0x1000ba23c`: frozen and not yet shattering: `vel.xy = 0`, `mbIceShatter`,
  `ignoreGrav = 1`, `sacrifice Shattered`, `→ PygmyIceShatter`. Triggered by touching,
  flicking or falling while frozen.
- Ice squid (ice hole tentacle): `canBeGrabbedByTentacle:` (dragging, not beheaded, healthy,
  T 0, not frozen), `grabbedByTentacle:` (dragging → `mbGrabbedByTentacle`,
  `→ PygmyGrabbedByTentacle`), `eatenByIceSquid:` (`sacrifice SquidBait`, `→ PygmyDie`),
  `tugOfWarWithSquid:::` (first: gravity on, floor and side off,
  `mTentacleStrengthRotVel = (r % 121 + 80) / 10`; each frame: `t` = touch in the pygmy's
  group, `tp = [hole tentaclePos]`, `d = tp - pos`, `a = deg(atan2(d.y, d.x)) + 90`,
  `rot.z = a`; `a > 60 || a < -60 || |d| > 180` → beheaded: `mbBeheadedBySquid`,
  `[hole beheadPygmy:self]`, `→ PygmyHeadRippedOffByIceMonster` (3); else `v = pos - t`
  (z 0), `k = -100`, `mTentacleStrengthRot += rotVel`; at 1000 wrap (-1000) and re-roll the
  speed; pull `f = (strength rot >= 700) ? 10 : 25`; `mSpringForce = (-8 vel.x + k v.x +
  f d.x, -8 vel.y + k v.y + f d.y, -8 vel.z, pos.w)`; 0).
- Snowball and icicles: §10.3.

### 10.7 Zombies

- Becoming one: `canHaveBrainsEaten:` (FREE +{sharkOnHead}, tilt 0) and
  `haveBrainsEaten: @ 0x1000b4080` (called by a zombie): remember `mbSleeping`, S13,
  `mbBitByBat = 0`, `mbIsZombie`, `mbBrainsEaten`, tilt 0, `vel.xy = 0`, `zrot0`,
  `layer<(pygmyDisplayMarker)`; awake → face away from the zombie,
  `PygmyZombieHaveBrainsEaten`; asleep → `mEatingBrainsPygmyLogic = zombie`,
  `PygmyZombieHaveBrainsEatenSleep` (`animSleepBrainsEatenXFlip` faces it later and forgets
  it). Or from the zombie grave: `startZombieIdle` (trapped: release the grave reference,
  `mbBitByBat = 0`, `mbIsZombie`, gravity, side and floor on, `vel.xy = 0`, `zrot0`,
  `xScale = 1`, `y = islandY`, `sel1`, `layer<(pygmyDisplayMarker)`, `→ PygmyZombieIdle`).
- `zombieWalk:::` (min, max, direction): first: timer; `mbZombieChooseDirection = (p2 ==
  nil)` (the walk animation's `animChooseDirection` then picks the direction). Each frame:
  `checkForFreshBrains` → 3; side collision → 2; timeout → 2.
- `zombieIdle:::`: timer; fresh brains → 3.
- `checkForFreshBrains`: another pygmy that `canHaveBrainsEaten:self` within
  `|x + xScale * 18 - o.x| < 5` → face it, `→ PygmyZombieEatBrains`,
  `[o haveBrainsEaten:self]`, YES. `eatBrainsComplete:::` flips `xScale`.
- `tripIntoZombieGrave:::`: `[mZombieGraveLogic tripIntoGrave:self fallForward:]` with
  forward = not facing the grave; 2.
- `zombieExplode @ 0x1000b664c`: zombie and not yet exploding: `ignoreGrav = 1`,
  `mbZombieExplode`, `→ PygmyZombieExplode`. Zombies explode instead of most reactions
  (touch, slide off, splash, lightning, hurricane, earthquake). `isZombie` getter. Death
  records `ZombieExplode` / `ZombieExplodeBrainsEaten`.

### 10.8 Vampires

A pygmy bitten in its sleep (`bitByBat:`) has `mbBitByBat`; `isVampire` = bitten and T 0.

- `checkForVampire:::`: bitten → `→ PygmyKillPygmies` (3), else 2.
- `canBeHitByVampire @ 0x1000a1094` → `!mbDrowning` after the guard (FREE −{T, drowning}
  +{bitByBat}, tilt 0, not a normal-type Charlie). `hitByVampire:xVel:`: S14 (spear before
  `stopBeingEaten`), `falling = 1`, tilt 0, `vel = (xVel, 500)`, `zrot0`,
  `sacrifice Vampire`, `PygmyHitByVampire` (T 0) / `TemplePygmyHitByVampire`.
- Daylight rule (in `checkKillPygmies:::`, `vampireRun:::`, `waitForNight:::`,
  `killPygmies:::`): when neither night nor eclipse: sunset → `→ PygmyVampireSunset`,
  else `sacrifice VampireInSunlight`, `→ PygmyVampireSun` (3).
- `checkKillPygmies:::` (optional timer p0/p1, gravity on): daylight rule; any pygmy that
  `canBeHitByVampire` → `→ PygmyKillPygmies` (3); timer.
- `vampireRun:::` (min, max): first: timer, side and gravity on, run toward the centre at
  ±100; daylight rule; victims → `PygmyKillPygmies`; side collision / timeout → 2.
- `waitForNight:::`: like `checkKillPygmies` but waits through the sunset; at night →
  `PygmyKillPygmies`.
- `killPygmies:::`: first: side collision on. Daylight rule (with `vel.x = 0`). Nearest
  victim (`|x - v.x|` from 1000 down): tolerance 40 for a ginger, 5 otherwise; within
  it: ginger → `sacrifice GingerPygmyKillVampire`, `→ PygmyVampireAttackGinger` (3);
  others `[v hitByVampire:self xVel:vel.x]`. Then run toward the victim at ±200; 0. No
  victim → `vel.x = 0`, `→ PygmyVampireIdle`, 2.
- `explodeVampireSun:::` / `explodeVampireGinger:::`: set `mbExplodeFromVampireSun` /
  `…Ginger`, then `explode:::`.
- Charlie's warlock burns vampires (§12).

### 10.9 Ghosts

A dead pygmy with ghosts enabled becomes a ghost (§3.6). The player draws its flight path.

- `ghostFlyUp:::`: first: `sel1`, `mGhostTargetYPos = (r % 7001 + 13000) / 100` (130..200),
  `vel.y = 100`; reaching the target → `vel.xy = 0`, 2.
- `ghostFlyIdle:::` (min, max): first: timer; horizontal `x > 20 && coin == 0` or
  `x >= 440` → left `vel.x = -50` else right 50; vertical `y > 40 && coin == 0` or
  `y >= 280` → `vel.y = -20` else 20. Each frame: leaving the box (y 40..280, x 20..440)
  in the moving direction or the timer → `vel.xy = 0`, 2; timer `-= dt` always.
- `animGhostXFlip` (frame action): `xScale = 1` when the target heading is in [-90, 90] or
  >= 270, else -1.
- Player input: `touchBeganInsideGhost:allTouches: @ 0x1000bec98`: `mActiveTouch`,
  `vel.xy = 0`, `mbTouchBeganInside`, clear the path and push the current position,
  `mGhostTouchX/YOffset = touch - pos`, `[listener onPygmyGhostDrag:self]`; create the
  light if missing (`BCMultiModel` `TheLight`, `MultiModelLogic` with listener self, at
  (50, 250, -277.12), placed before the ghost, `→ TheLightInit`); `mbGhostActionTriggered
  = 0`, `mbGhostBansheeScream = 1`, `→ PygmyGhostStartScream`.
  `touchMovedGhost:allTouches: @ 0x1000bef70` (active touch, pressed, not going into the
  light, path shorter than 40 points): `t` = touch in the ghost's group. Without a victim:
  with more than 2 points and `|t - light.pos| < 20` → `mbGoIntoTheLight`, return.
  Otherwise the first pygmy that `canBeTouchedByGhost:self` and contains `t` strictly
  becomes the victim, with a type from how the finger entered its box (last touch
  `mGhostLastTouchPos`): from below (`last.y <= min.y`) and the ghost higher than 100 →
  type 0 (pull under); from above → 1 (carry away); from the left or right → 2
  (possess); `[victim touchedByGhost:self]`. A type-2 victim whose box the finger leaves
  becomes type 3 (scare). Then `mGhostLastTouchPos = t`; target point
  `(t.x - offsetX, t.y - offsetY)` clamped to y 40..280, x 21..440; within 20 of the last
  path point it replaces it, else it is appended.
  Release (§5.3): the light exits unless going in; path longer than 1 →
  `PygmyGhostFollowPath`; `onPygmyGhostDragComplete:`.
- `ghostFollowPath::: @ 0x1000b7ad4`: moves `16.666666` units **per call** along
  the path (frame-rate dependent), consuming points; heading
  `mGhostTargetZRot = deg(atan2(dy, dx)) - 90`. When the next point is not yet reached:
  no victim → flip if needed (`PygmyGhostFollowPathFlip`, 3); a victim of type 1..3 and
  the ghost strictly inside the victim's bounds → stop (`vel.xy = 0`, target rot 0,
  `mbGhostActionTriggered`, `cancelFlip`, path cleared, `sel0`): type 3
  `PygmyGhostScarePygmy` + `[v scaredByGhost:self]`, type 2 `PygmyGhostPossessPygmy` +
  `[v possessedByGhost:self]`, type 1 `PygmyGhostStartCarryAwayPygmy` +
  `[v carriedAwayByGhost:self]` (3); type 0 and `y < 96` → `mGhostEmergeXPos = x`,
  `PygmyGhostPullPygmyUnder` (3). End of path: `vel.xy = 0`, target 0, `cancelFlip`; into
  the light → `sel0`, `[light setBehavior:"TheLightEnter"]`, `→ PygmyGhostEnterTheLight`
  (3); else a victim not acted on gets `cancelGhostTouch:self`; 2.
- Victim side: `canBeTouchedByGhost:` (FREE +{sharkOnHead}, tilt 0); `touchedByGhost:`
  (S14, `vel.xy = 0`, `zrot0`, gravity off, tilt 0, `sel0`, `mbTouchedByGhost`,
  `mGhostPygmyLogic`, `PygmyWaitForGhostTouchSleep` / `PygmyWaitForGhostTouch`);
  `possessedByGhost:` (face the ghost, ghost display on tween channel 1,
  `→ PygmyPossessedByGhost`); `scaredByGhost:` (`sacrifice ScaredByGhost`, face, channel 1,
  `→ PygmyScaredByGhost`); `pulledUnderByGhost:` (`sacrifice PulledIntoGround`,
  `→ PygmyPulledUnderByGhost`); `carriedAwayByGhost:` (`sacrifice CarriedAwayGhost`,
  `xScale = x > 240 ? -1 : 1`, `mbCarriedAwayByGhost`, channel 1,
  `→ PygmyCarriedAwayByGhost`); `cancelGhostTouch:` (back to normal, `→ PygmyIdle`).
  Items: `possessedByGhostComplete:::` / `pulledUnderComplete:::` (reset, notify the
  ghost, clear channel 1, 2), `initCarriedAwayByGhostComplete:::` (gravity off, floor
  off, notify), `carryPygmyOffScreen:::` (`x < -50 || x > 530` → notify
  `carriedAwayByGhostComplete:`, 2), `animPygmyScaredComplete` (frame action: notify
  `scaredByGhostComplete:`).
- Ghost side callbacks: `possessedByGhostComplete:` / `scaredByGhostComplete:`
  (forget the victim, `sel1`, `→ PygmyGhostIdle`), `pulledUnderByGhostComplete:`
  (`x = mGhostEmergeXPos`, `y = 96`, `→ PygmyGhostPullPygmyUnderComplete`),
  `initCarriedAwayByGhostComplete:` (`vel = (xScale * 100, 100)`, channel 1 = victim,
  `→ PygmyGhostCarryAwayPygmy`), `carriedAwayByGhostComplete:` (`→ PygmyDie`),
  `cancelGhostHold:` (victim died: forget it, `sel1`, `PygmyGhostIdle` unless removed from
  story), `cancelGhostTouchDead:` (nil), `penetrateGroundComplete:::`
  (`[victim pulledUnderByGhost:self]`).
- Banshee scream (while the player holds the ghost): `startBansheeScream:::`
  (`mbGhostBansheeScreamStarted`, every pygmy `holdEarsFromBansheeScream:self`),
  `burstPygmyEarDrums:::` (every pygmy `explodeFromBansheeScream:self`),
  `stopBansheeScream` (not started → `PygmyGhostWaitForPath`; started → every pygmy
  `stopHoldEarsFromBansheeScream:`, stop sound `BansheeCryLoop`,
  `→ PygmyGhostStopScream`). Victims: `holdEarsFromBansheeScream:` (FREE: S13,
  `vel.xy = 0`, `zrot0`, tilt 0, `sel0`, `mbHoldEarsFromBansheeScream`,
  `→ PygmyHoldEarsFromBansheeScream`), `explodeFromBansheeScream:` (holding ears and not
  a ghost: `sacrifice FromBansheeScream`, flag, `→ PygmyExplodeFromBansheeScream`),
  `explodeFromBansheeScream:::` (`vel.xy = 0`, `zrot0`, sacrifice, `explode:::`),
  `stopHoldEarsFromBansheeScream:` (not exploding: clear, `sel1`,
  `→ PygmyStopHoldEarsFromBansheeScream`).

## 11. Apocalypse category (`PygmyLogic(Apocalypse)`, 142 methods)

### 11.1 Temple types

- `setTimeTemplePygmyType:` / `timeTemplePygmyType` (§7.2).
- `templePygmyWalk:::` (minTime, xPosVel, maxTime): first: timer `RAND100(p0, p2)`, side
  collision and gravity on, walk direction rule with speed `p1`. Each frame:
  `checkForCoconut || checkForFish || checkForBird || checkForBanana` → 3; side collision
  or timeout → `vel.x = 0`, 2; else 0.
- `templePygmyIdle:::` (min, max): timer only.
- Old fart: `checkTempleOldPygmyFallDownFromSlide:::` (upright old fart → clear upright,
  `→ TempleOldPygmyFallDownFromSlide` (3), else 2), `templeOldPygmyFallDown:::` (clear
  upright; 2). Heart attack: §5.3.
- Ginger: `checkForGingerSunBurnt:::` (Island, in `TempleGingerPygmyIdleLoop`): no sunset,
  no night, no storm cloud, outside the temples → `mbGingerSunBurn`,
  `sacrifice GingerPygmySunBurn`, `→ TempleGingerPygmySunBurnt`; 2.

### 11.2 Entering a temple room: `dropIntoApocalypseTempleRoom @ 0x10003bd74`

Called by the controller for every pygmy when the island moves into a temple room.

1. Tom inside the closet → return. Held by Xenu → `stopBeingHeldByXenu`, `→ PygmyDie`.
   Sacrificed to Xenu → `→ PygmyDie`. Died in a thetan brawl → `stopThetanBrawl`,
   `→ PygmyDie`.
2. Guard (§25; FREE minus the temple containers and a few more, drowning allowed for
   plague bodies, not underwater). If it fails: Tom in the closet room (L 16) →
   `mbTellClosetToGrabNewTom = 1`; `→ PygmyDie`.
3. S27, hurricane-drop (clears the flag), drag-end, doodle-end,
   `x = RAND10(islandMinX + 15, islandMaxX - 15)`, `y = 340`, `z = -277.12`, scale (1,1),
   `vel.xy = 0`, tilt 0, side off, floor on, apocalypse temple door released, `falling = 1`,
   `ignoreGrav = 1`, `sel1`, `mbInvalidSacrifice = 1`, `sacrifice DroppedDrown`, then: plague
   body → `PygmyPlagueBodyFallOffWorld` (disappearing → `PygmyDie`); cut →
   `HalfPygmyFallInsideApocalypseTempleRoom`; T 0 → `PygmyFallInsideApocalypseTempleRoom`;
   else `TemplePygmyFallInsideApocalypseTempleRoom`.

`moveToApocalypseTempleDisplayGroup` / `moveFromApocalypseTempleDisplayGroup`: re-layer
(`layer(pygmyDisplayMarker)` +labels) except Tom inside the closet.

### 11.3 Temple containers

| Catcher / starter | Filter | Box | Effect |
|---|---|---|---|
| `checkForTempleDoorJump:::` (item) | BG-like (no T), temple door enabled and available | `\|x - (d.x + 90)\| <= 80` | `vel.xy = 0`, `xScale = 1`, gravity off, floor off, `mbInsideTempleDoor`, `[d jumpIntoTempleDoor:self]`, `sel0`, `→ PygmyJumpIntoTempleDoor` (3); `jumpIntoTempleDoorComplete:::` notifies the door |
| `checkForApocalypseTimeTemplePlatformDuringFall` | not cooked/plague/cut/invalid/doodle/dropped/heart attack/thrown (any T) | `\|dx\| <= 30`, `\|y - (p.y + 50)\| <= 30` | tilt 0, `mbOnTopApocalypseTimeTemplePlatform`, `fallOntoApocalypseTimeTemplePlatform:`, `PygmyFallOntoApocalypseTimeTemplePlatform` |
| `checkForLocustQueenDuringFall` | + T 0 | `\|x - (q.x + 40)\| <= 30`, `\|y - (q.y + 130)\| <= 30` | tilt 0, `mbEatenByLocustQueen`, `sel0`, `fallIntoLocustQueenArms:`, `PygmyFallIntoLocustQueenArms`; `eatenByLocustQueenComplete:` → `mbDisableLabels`, `sacrifice EatenByLocustQueen`, `→ PygmyDie` |
| `checkForApocalypseTempleFountainDuringFall` | as the platform; plague fountain if enabled, else purification fountain | `checkForApocalypseTempleFountainDuringFall:` (fountain) tries the slots via
`checkForApocalypseTempleFountainDuringFall:xOffset:yOffset:` with offsets (-51, 40), (-3, 40), (47, 40) for available slots 1..3: `\|x - (f.x + xo)\| <= 30`, `\|y - (f.y + yo)\| <= 10` | tilt 0, `mbInApocalypseTempleFountain`; a different disease than the fountain's → incubation 5; `layer<(pygmyDisplayMarker)` +labels; `infectWithDisease:` the fountain's type; `PygmyFallIntoApocalypseTempleFountain`; `fallIntoApocalypseTempleFountainSlotN:` |
| `checkForClosetTempleThetanMeterDuringFall` | + T 0 | `\|dx\| <= 20`, `\|dy\| <= 20` | tilt 0, `mbOnClosetTempleThetanMeter`, `fallOntoClosetTempleThetanMeter:`, `PygmyFallOntoClosetTempleThetanMeter` |
| `checkForClosetTempleCouchDuringFall` | + T 0, couch and closet door enabled and available | `\|x - (c.x + 50)\| <= 20`, `\|y - (c.y - 25)\| <= 20` | `zrot0`, `y = c.y - 35`, tilt 0, layer after `ApocalypseTempleClosetDisplayMarker` +labels; if not yet in the closet: `mbInClosetTempleClosetDoor` + `fallIntoClosetTempleClosetDoor:`; `mbOnClosetTempleCouch` + `fallOntoClosetTempleCouch:`; `PygmyTomFallOntoClosetTempleCouch` (Tom) / `PygmyFallOntoClosetTempleCouch` |
| `checkForClosetTempleClosetDoorDuringFall` | + T 0 | `minClosetX < x < maxClosetX`, `\|y - closetY\| <= 20` | `y = closetY`, `zrot0`, tilt 0, closet layer +labels, `mbInClosetTempleClosetDoor`, `fallIntoClosetTempleClosetDoor:`, not Tom → `episode PlacePygmyInCloset`, `PygmyFallIntoClosetTempleClosetDoor` |
| `checkForTomPygmyDuringFall` | baby only (T 2, P != 2), Xenu temple enabled, Xenu not spawned, Tom alive, `[tom isBabyHoldingAvailable]` | `\|dx\| <= 20`, `\|y - (tom.y + 20)\| <= 20` | tilt 0, `mbHeldInTomsArms`, `mTomPygmyLogic`, `[tom fallIntoTomsArms:self]`, `PygmyFallIntoTomsArms` |
| `checkForXenuDuringFall` | not cooked/plague/invalid/doodle/dropped/thrown, T 0 (half pygmies allowed) | slot 1 `\|x - (X.x + 74)\| <= 20`; slot 2 `\|x - (X.x - 68)\| <= 20`; both `\|y - (X.y + 2)\| <= 20` | tilt 0, `sel0`, `sacrifice XenuPuppetShow` (cut) else `EatenByXenu` (slot 1) / `XenuStripped` (slot 2), `mbHeldByXenu`, `mXenuLogic`, `fallIntoXenuSlot1:` / `Slot2:`, `PygmyFallIntoXenu` |
| `checkForGoGoCageDuringFall` | not cooked/plague/cut/invalid/doodle/dropped/thrown | `\|dx\| <= 20`, `-15 <= y - g.y <= 0` | `mbInGoGoCage`, `pygmyFallIntoGoGoCage:`, cage display `insertAfter:` +labels, `PygmyFallIntoGoGoCage` |
| `checkForDanceDanceExecutionDuringFall` | + T 0 | same box | `mbInDanceDanceExecution`, `pygmyFallIntoDanceDanceExecution:`, display `insertAfter:` +labels, `PygmyFallIntoDanceDanceExecution` |
| `startInClosetTempleClosetDoor` | (controller: Tom starts inside) | centre of the closet | `x = (minClosetX + maxClosetX) / 2`, `y = closetY`, `zrot0`, `vel.xy = 0`, `falling = 0`, `mbInvalidSacrifice = 0`, gravity off, floor off, tilt 0, closet layer +labels, `mbInClosetTempleClosetDoor`, `fallIntoClosetTempleClosetDoor:`, `PygmyFallIntoClosetTempleClosetDoor` |

Ejects (EJ with `vel = (-200, 500)` unless noted; the temple ones also restore
`mIslandYPos = 86` and re-layer with `layer(pygmyDisplayMarker)` +labels):
`ejectFromApocalypseTempleDoor`, `ejectFromApocalypseTimeTemplePlatform` (`TemplePygmyFall`
for T != 0), `rescuedFromLocustQueen`, `ejectFromApocalypseTempleFountain` (T variant),
`ejectFromClosetTempleThetanMeter`, `ejectFromClosetTempleCouch`,
`ejectFromClosetTempleClosetDoor`, `tossFromGoGoCage`, `tossFromDanceDanceExecution`,
`ejectFromXenu` (`→ PygmyDie`), `ejectFromTomsArms` (sacrificed → `PygmyDie`, else
`TemplePygmyFall`), `danceDanceExecuted:` (in the DDE: `mbDanceDanceExecuted`, `sel0`),
`danceDanceExecutedComplete:` (`sacrifice DanceDanceExecuted`, `→ PygmyDie`).
The matching stop methods (`stopBeingInsideApocalypseTempleDoor`,
`stopBeingOnTopApocalypseTimeTemplePlatform`, `stopBeingEatenByLocustQueen`,
`stopBeingInApocalypseTempleFountain`, `stopBeingOnClosetTempleThetanMeter`,
`stopBeingOnClosetTempleCouch`, `stopBeingInClosetTempleClosetDoor`,
`stopSlammingClosetTempleClosetDoor`, `stopBeingHeldByXenu`, `stopBeingHeldInTomsArms`)
tell the container (`ejectFromX:self` / `rescued…`)
and clear the reference; `stopBeingInClosetTempleClosetDoor` also fires
`episode RemoveTomFromCloset` for Tom; `stopBeingHeldByXenu` does not clear
`mbHeldByXenu`.

### 11.4 Closet temple, Tom, the baby and Xenu

- `canSlamClosetDoor` (not slamming, in the closet, Tom, T 0), `slamClosetDoor`
  (`mbSlammingClosetTempleClosetDoor`, `→ PygmyTomSlamDoor`), `animSlamClosetDoorComplete`
  (frame action: clear, `[door slamDoor]`), `trappedInCloset` (= in the closet).
- `checkTomJumpOnCouch:::`: Tom in the closet, couch enabled and available; `a = c.x + 10`,
  `b = c.x + 90`; jumps only when facing the couch centre and `a <= x <= b`: floor off,
  tilt 0, `mbOnClosetTempleCouch`, `[c jumpOntoClosetTempleCouch:self]`, gravity on,
  `vel = (xScale * 100, 600)`, `→ PygmyTomJumpOnCouch` (3); else 2.
  `waitApexTomJumpOnCouch:::` (2 when falling), `waitLandTomJumpOnCouch:::` (falling and
  within 20 of `c.y - 35` → gravity off, `vel.xy = 0`, `y = c.y - 35`, 2),
  `tomJumpOffCouch:::` (gravity on, `stopBeingOnClosetTempleCouch`,
  `vel = (xScale * 100, 600)`, 2), `waitLandTomJumpOffCouch:::` (land at the closet y),
  `checkSlamDoorBouncingOnCouch:::` (on the couch → `PygmyTomJumpOffCouchAndSlamDoor`),
  `checkForFinishClosetIdle:::` (in the closet → `FinishPygmyTomClosetIdle`),
  `runToClosetDoor:::` (`xScale = 1`, `vel.x = 400` until `x > maxClosetX`, 2; no clamp),
  `closetIdle:::` (timer), `closetWalk:::` (min, max: direction between the closet bounds,
  ±50, clamped to them).
- Baby and Tom: `isBabyHoldingAvailable` (Tom: `FREE −{hurricanes, Kong, apes, go-go,
  DDE, zombie}`, P 2, no baby), `fallIntoTomsArms:` (Tom: tilt 0, `vel.xy = 0`,
  `mbHoldingBabyPygmy`, `mBabyPygmyLogic`, baby on channel 15, `x >= 240` →
  `xScale = -1`, `PygmyTomCatchBabyPygmyRight`, else `PygmyTomCatchBabyPygmyLeft`),
  `walkToXenu:::` (Tom walks at ±50 to the Xenu x; arriving: snap,
  `[[listener xenuTempleLogic:] tearDownWall]`, `mbStartTomBabySacrifice`, `sel0`,
  `[baby startTomBabySacrifice]`, 2), `startTomBabySacrifice` (baby:
  `mbSacrificedToXenu`, `sel0`), `animTomBabySacrificeComplete` (Tom frame action:
  `[baby babySacrificeHideLabel]` → baby `mbDisableLabels`), `animTomShowXenu` (Tom:
  drop the baby (`clearTweenableChannelIdNow:15`, `[baby babySacrificeComplete]`), spawn
  Xenu, clear the sacrifice flag, `sel1`), `babySacrificeComplete` (baby:
  `sacrifice BecameXenu`, `→ PygmyDie`), `ejectFromTomsArms:` (Tom: an interrupted
  sacrifice restarts the apocalypse `[listener startApocalypse]` and spawns Xenu;
  `PygmyTomEjectBabyPygmy`), `stopHoldingBabyPygmy` (Tom: same reset,
  `episode HaveTomDropBaby`, `[baby ejectFromTomsArms]`), `stopBeingHeldInTomsArms`
  (baby).
- `runSequenceIfInXenuRoom:::` (L 17, no apocalypse, id given → `setBehavior:` id (3);
  else 2).

### 11.5 Disease

- `incubateDisease:infectChallengeEvent:infectedByChallengeEvent:` (src, E1, E2): `s` =
  src disease, `m` = own. Nothing when both 0 or equal. Healthy pygmy, sick object:
  `mDiseaseIncubateIdleCycles = max(cycles, 5)`, `episode E2`, catch `s`. Sick pygmy,
  clean object: `episode E1`, `[src infectWithDisease:m]`. Both sick with different
  diseases: `r % 3`: 2 → cured (`infectWithDisease:0`); 1 → infect the object; 0 → incubate
  5 and catch `s`.
- `canBeInfected:` (FREE −{T} +{sharkOnHead}, tilt 0).
  `infectWithDisease:recoilFromPygmyLogic:` (type, src): a new, different disease → S31,
  face away from src (`xScale = src.x < x ? -1 : 1`), `vel.x = 0`,
  `→ PygmyRecoilFromDisease`, incubation >= 5; always `SpriteLogic::infectWithDisease:`.
  `recoilFromPygmyDisease:` (S31, face away, `vel.x = 0`, recoil).
- Frame actions: `animStartPygmyRecoil` (pygmies that can be infected within 100 recoil),
  `animInfectPygmies` (those within 60 of `x + xScale * 40`: `episode
  infectPygmyWithGerms` (plague) / `infectPygmyWithPureHell` (purification) and
  `infectWithDisease:D recoilFromPygmyLogic:self`).
- `checkDiseaseIncubation:::` (only in `FinishPygmyPlagueIdle` /
  `FinishPygmyPurificationIdle`): `cycles--`; when it reaches 0 (or was already < 1):
  plague → `mbKilledByPlague`; fewer than 6 pygmies, or another pygmy not killed by the
  plague → `PygmyKilledByPlague`, else `mbKilledByPlagueDisappear`, `sel0`,
  `PygmyKilledByPlagueDisappear` (the last sick pygmy vanishes instead of leaving a body);
  purification (any D != 1) → `mbKilledByPurification`, `sel0`,
  `sacrifice KilledByPurification`, `PygmyKilledByPurification` (3). Else 2.
- Plague bodies: `isAvailablePlagueBodyToBeCarried` (`!mbPlagueBodyInPile`, on the island,
  not carried, killed by plague), `startCarryingPlagueBody:` (body: gravity off, floor
  off, carried, `sel0`), `animPickupPlagueBody` → `pickupPlagueBody`
  (`mbPlagueBodyPickedUp`; the label then follows the carrier), `animThrowPlagueBody`
  (carrier: clear channel 6, `[body throwPlagueBodyOnPile]`), `throwPlagueBodyOnPile`
  (body: layer after `SmallObjectsMarker` +labels, `vel.y = 100`,
  `→ PygmyPlagueBodyThrowOnPile`), `throwPlagueBodyOnPile:::` (body item: first: floor
  and gravity on, falling, `zrot0`; every frame `x += (fire.x - x) * 0.5`; when falling
  below `fire.y`: stop; burning fire → `sacrifice PlagueBodyBurntOnFire`,
  `PygmyPlagueBodyBurnOnFire`, else layer back to the island, `sel1`,
  `PygmyPlagueBodyFall` (3)), `dropPlagueBodyWhileCarrying` (`vel.y = 400`, `zrot0`,
  `sel1`, `PygmyPlagueBodyDropWhileCarrying`), `stopCarryingPlagueBody` (carrier: clear
  channel 6 now, `[body dropPlagueBodyWhileCarrying]`, nil), `respawnPlagueBody:::` (back
  to `RAND10` x, y 340, reset, `mbInvalidSacrifice = 1`), `animStartFloat` (`vel.x = 20`),
  `plagueBodyFloat:::` (wraps around the screen horizontally, never completes), getters
  `isPlagueBodyPickedUp`, `isKilledByPlague`, `plagueBodyCarryingPygmyLogic`.

### 11.6 Thetans

- `canBePossessedByThetan` (FREE +{sharkOnHead, sleeping}, tilt 0),
  `startPossessedByThetan:` (S14, `vel.xy = 0`, `zrot0`, gravity off, tilt 0, `sel0`,
  `mbPossessedByThetan`, `mbWaitingForPossessionByThetan`, `mThetanLogic`,
  `→ PygmyStartPossessedByThetan`), `possessedByThetan:` (waiting 0, `sel1`,
  `PygmyPossessedByApocalypseThetan` during the apocalypse, else
  `PygmyPossessedByThetan`), `checkForPossessedByThetan:::` (possessed →
  `PygmyPossessedByThetanIdle`), `animReleaseThetan` / `animStopThetanPossession` →
  `stopBeingPossessedByThetan` (`[thetan stopPossessingPygmy:self]`),
  `thetanStopPossessingPygmy:` (`→ PygmyUnpossessedByThetan`), `isPossessedByThetan`.
- `possessedByThetanIdle:::` (min, max): timer only. `possessedByThetanWalk:::` (min, max):
  walk at ±50; `mThetanBrawlCoolDown -= dt`; when
  below 0, `checkForThetanBrawl` → 3.
- `canDoThetanBrawl:` (FREE −{possessed} +{sharkOnHead, sleeping, waiting}, tilt 0).
  `checkForThetanBrawl`: first other pygmy that can brawl, `d = x - o.x`, facing it: an
  unpossessed one at `60 < |d| < 70` → pounce (`o.xScale = xScale`, `vel.xy = 0`, gravity
  off, tilt 0, `sel0`, brawl, defender, cooldown 2, `[o defendAgainstThetanBrawl:self]`,
  `coin == 0` → `PygmyThetanBrawlPummel` else `PygmyThetanBrawlEatFace`); a possessed one
  at `25 < |d| < 35` → fight (`o.xScale = -xScale`, …, `PygmyThetanBrawlAttack`).
- Defender: `defendAgainstThetanBrawl:` (S14, `vel.xy = 0`, brawl, attacker reference;
  unpossessed → `mbDiedFromThetanBrawl`, `sel0`, attacker on channel 16,
  `sacrifice BittenToDeath`, `PygmyThetanBrawlGetJumped`; possessed →
  `PygmyThetanBrawlDefend`), `thetanBrawlGetPummeled` (`sacrifice ThetanPummeled`),
  `thetanBrawlGetFaceEaten` (`sacrifice BittenToDeath`), `animPounceComplete`,
  `thetanPounceComplete`.
- Slap fight: `chooseBrawlAttack:::` (`r % 3`: 2 slap fight for both, 1 the defender
  punches, 0 this one punches), `brawlAttackComplete:::`, `thetanBrawlThrowPunch`,
  `thetanBrawlLandPunch` (`PygmyThetanBrawlGetHit`), `animLandThetanPunch`,
  `thetanBrawlStartSlapFight`, `thetanPummelPygmy:::`, `thetanEatPygmyFace:::`,
  `stopThetanBrawl`, `stopThetanBrawlByPygmy:` (died → `PygmyDie`, else `PygmyIdle`).
- `explodeFromThetanBirth` (`mbDisableLabels`, `sacrifice ExplodeFromThetanBirth`,
  `→ PygmyDie`; called by the thetan meter).

### 11.7 Locusts and dance balloons

- `checkForLocusts:::` / `checkForLocustsComplete:::`: `[[listener locustLogic:]
  shouldPygmyPanic]` → S12′, `→ PygmyLocustPanicIdle`; when the panic is over →
  `→ PygmyIdle`.
- `startThrownInAirByLocusts:` (FREE +{sharkOnHead}, tilt 0): S13, `mbThrownByLocusts`,
  locust reference, `falling = 1`, gravity off, floor and side off, `vel.xy = 0`, `zrot0`,
  `sel0`, `mbUseLocustCatch1`, `→ PygmyThrownByLocustsInitialLift`,
  `layer(pygmyDragDisplayMarker)` +labels.
  `thrownInAirByLocusts:targetPos:targetTime:` (ballistic throw under the 1500 gravity:
  `vel.x = (tx - x) / t`, `vel.y = 750 t + (ty - y) / t`, gravity on, alternate
  `PygmyThrownByLocusts1/2`), `catchInAirByLocusts:` (gravity off, `vel.xy = 0`,
  `…Catch1/2`), `locustThrowWaitForApex:::`, `droppedInAirByLocusts:` (release, invalid
  sacrifice, `sacrifice DroppedDrown`, `PygmyFall`), `stopBeingThrownByLocusts`.
- Kills (FREE +{sharkOnHead}, tilt 0; all `sel0` and `mbKilledByLocusts`):
  `cutInAirByLocusts:` (falling only: `vel = (100, 700)`, the bottom half flies off via
  `[getLogic(0) tossBottom:self]`, `sacrifice SlicedByLocust`, `PygmyCutInAirByLocusts`),
  `stripFleshByLocusts:` (`sacrifice LocustFood`), `animSpawnSkullAndBone` (first free of
  skulls 464..466 / bones 467..469, `spawnLocustSkull:` / `spawnLocustBone:`),
  `mouthFilledByLocusts:` (`mbMouthFilledByLocusts`, `sacrifice PopLocustBalloon`),
  `mutatedByLocusts:` (`mbMutatedByLocusts`, `sacrifice TurnedIntoMutantLocust`),
  `mouthFilledByLocustsComplete:::` / `mutatedByLocustsComplete:::` (`sel1` so they can be
  tapped: §5.1), `checkMutantLocustFlyOff:::` (third idle: `vel = (xScale * 100, 100)`,
  gravity off, `PygmyMutatedByLocustsFlyOff`), `waitForMutantLocustFlyOffComplete:::`
  (off screen → 2).
- `inhaleDanceBalloon:` (FREE +{sharkOnHead}, tilt 0): S13, `mbHighOnNitrous`,
  `vel.xy = 0`, `→ PygmyInhaleDanceBalloon`. `stopBeingHighOnNitrous` clears the flag.

## 12. Charlie category (`PygmyLogic(Charlie)`, 28 methods)

Tiger blood (Charlie cuts a pygmy in half):

- `checkForTigerBlood` (Charlie, from `idle:::` and `walk:::`): first other pygmy for which
  `canBeCutInHalfByCharlie:` (FREE +{bitByBat, sharkOnHead, sleeping}, tilt 0) is YES with
  `|x + xScale * 40 - o.x| < 20` → face it, `vel.xy = 0`, `mCharlieCuttingPygmyLogic = o`,
  `→ PygmyCharlieCutPygmyInHalf`, `[o cutInHalfByCharlie:self]`, YES.
- Victim `cutInHalfByCharlie:` (S13, tilt 0, `vel.xy = 0`, `zrot0`,
  `mCharliePygmyLogic = charlie`, `layer<(pygmyDisplayMarker)`,
  `→ PygmyCutInHalfByCharlie`).
- `animCutPygmyInHalf` (Charlie frame action, not in story mode):
  `[victim cutInHalfByCharlieComplete:self]`, nil. `cutInHalfByCharlieComplete:` (victim;
  skipped when exploding from love): `vel = (100, 700)` (`vel.x = -200` when left of
  Charlie), `[getLogic(0) tossBottom:self]` (the legs fly off as a `PygmyBottomLogic`,
  §21), `mbBitByBat = 0`, `falling = 1`, `mbCutInHalfByCharlie`,
  `→ PygmyCutInHalfByCharlieComplete`. From now on the `HalfPygmy*` variants apply.
- Half pygmy: `halfPygmyWalk:::` (direction rule sets only `xScale`, the animation moves
  it), `halfPygmyIdle:::`, `checkToAbsorbPygmyDuringFall` (a falling half pygmy, not
  dropped by a hurricane, lands on another pygmy for which `canBeAbsorbedByHalfPygmy:`
  (FREE +{sharkOnHead}, tilt 0) is YES, with `|x - o.x| < 20` and `|y - (o.y + 32)| < 20`
  (strict): `vel.xy = 0`, copies the other's pos, rot and scale; Charlie →
  `sacrifice AbsorbedByCharlie`, `HalfPygmyAbsorbCharlie`; else **`mbCutInHalfByCharlie =
  0`** (the half becomes whole again) and `HalfPygmyAbsorbPygmy`;
  `[o absorbedByHalfPygmy:self]`; YES). Victim `absorbedByHalfPygmy:` (S13, `vel.xy = 0`;
  Charlie `PygmyCharlieAbsorbedByHalfPygmy`; else `mbAbsorbedByHalfPygmy`,
  `sacrifice AbsorbedByHalfPygmy`, `PygmyAbsorbedByHalfPygmy`). `isCutInHalf` getter.
- Interruption: `stopCharlieMischief` (Charlie's references: the victim gets
  `stopCutInHalfByCharlie:` (`PygmyIdle` unless exploding), the cutter
  `stopCharlieMischief:`, the monkey `bootedOffCharliesBack:`); part of S13.

Warlock (Charlie's idle `PygmyCharlieWarlockIdle`):

- `checkForWarlock:::`: some other pygmy `canBeBurntByWarlock:` (vampires: FREE
  +{sharkOnHead}, requires `mbBitByBat`, tilt 0) → 2; else if
  `![listener canWarlockUseLightning:self]` (weather state 1 outside the temples) or no
  pygmy `canBeHitByLightning` → `→ FinishPygmyCharlieIdle`; 2.
- `animWarlockBurnVampire` (not in story mode): first burnable pygmy →
  `mbWarlockSkipNextExplosion = 1`, `[o burntByWarlock:self]` (S13,
  `sacrifice BurntByWarlockAssassin`, `→ PygmyVampireBurntByWarlock`).
- `animWarlockStrike` (not story, no skip pending): first other pygmy whose
  `hitByLightningByWarlock` returns YES → skip flag, `[listener
  hitPygmyWithWarlockLightning:o]` (the controller draws a 7-point lightning bolt to it).
  `hitByLightningByWarlock`: as `hitByLightning` with `sacrifice ZappedByWarlockAssassin`
  and no earthquake reset; in a hurricane only the behaviour changes.
- Other frame actions: `animCharlieLaunchShark` (`mbSharkOnHead = 0`,
  `[shark launchOffCharlie]`), `animCharlieThrowFood` (each eaten food
  `thrownByCharlie:self`, nil, clear channel 14).
- `charlieSmackBoulder:` / `charlieSmackTRexEgg:` (FREE +{sharkOnHead}, tilt 0): S13, tilt 0,
  `vel.xy = 0`, `zrot0`, face it, `PygmyCharlieSmackBoulder` / `PygmyCharlieSmackTRexEgg`;
  YES.
- Monkey on Charlie: `monkeyBootedOffCharliesBack:` (nil, `→ PygmyIdle`),
  `charlieMonkeyWalk:::` (±50), `charlieMonkeyIdle:::`,
  `charlieCutPygmyInHalfComplete:::` (item).

## 13. Story category (`PygmyLogic(Story)`, 18 methods)

Driven by `StoryManager`.

- `setStoryMode` (`mbStoryMode = 1`), `isReadyForStoryMode` (`mbStoryModeReady`),
  `checkForStoryMode:::` (story → `mbStoryModeReady`, `→ PygmyStoryModeIdle` (3)).
  In story mode `walk:::` does no interaction checks and `fall:::` / `tumble:::` /
  splashes complete without dying.
- `setStoryModeCharacterState:` (state string): `vel.xy = 0`,
  `→ [NSString stringWithFormat:@"PygmyStoryMode%@", state]`.
  `resumeFromStoryModeCharacterState`: not dying → state `Idle`; else
  `mbDieInStoryResume = 1` and, if the death already completed, `→ PygmyDie`.
- `startStoryEarthquake` (`vel.xy = 0`, `PygmyLoseBalance1/2` by coin),
  `stopStoryEarthquake` (`→ PygmyIdle`).
- Word balloon: `openWordBalloon:` (text; once: `BCMultiModel` `WordBalloon` with a
  `MultiModelLogic` listener self, inserted after `pygmyDragDisplayMarker`; dynamic text
  `DialogueLine1` reset to `@""`, wrap width 104, set the text;
  `→ StoryWordBalloonOpenRight` on the balloon; it follows the pygmy each frame),
  `openApeKidnapWordBalloon` (text `I'll save you!`), `closeWordBalloon` (balloon
  `→ StoryWordBalloonCloseRight`, which ends with the `WordBalloonCloseComplete` event),
  `closeWordBalloonNow` (shutdown, remove, release), items `openWordBalloon:::` (p0) and
  `closeWordBalloon:::`.
- `removeFromStoryMode`: S34, doodle-end, out-house `stopCrapping:`, T-Rex / spider
  `clearPygmyLogic:`, containers `removePygmyFrom…:` (flags kept), crushed →
  boulder / egg `clearPygmyLogic:`, `mbRemoveFromStoryMode = 1`, `→ PygmyDie` (the death
  never becomes a ghost).
- Items: `storyModeKnockOffIsland:::` (`falling = 1`,
  `vel = ((int)(r % 40001 - 20000) / 100, 600)`, `zrot0`), `storyModeFlick:::`
  (`vel = ((int)(r % 20001 - 10000) / 100, 1200, -400)`, `mTumblePos = pos`, falling, not
  drowning), `dieInStory:::` (`mbDieInStory`, not ready), `dieInStoryComplete:::`
  (complete, labels off, resumed → `PygmyDie`). Falling out of the world while dying in
  the story → `PygmyStoryModeDie` (§4.1 step 17).

## 14. Underwater category (`PygmyLogic(Underwater)`, 49 methods)

Locations 2 (underwater) and 9 (coral reef). Gravity is 30 (sinking) for an underwater
pygmy that is not dragged; `mbUnderwater` keeps it inside the screen (§4.1 step 12).

### 14.1 Spawning and transitions

- `addedUnderwater @ 0x10011d90c` (new pygmies created underwater): tilt 0, `falling = 0`,
  `mbUnderwater`, `ignoreGrav = 1`, `mbAddedUnderwater`, side and floor off,
  `mbInvalidSacrifice = 0`, `vel.xy = 0`, `zrot0`, `sel0`,
  `layer(pygmyUnderwaterPygmyDisplayMarker)` +labels, `→ PygmyAddedUnderwater`;
  `addUnderwaterComplete:::` (gravity on, `mbAddedUnderwater = 0`, `mbActiveUnderwater`,
  `sel1`, bait master → `startBaitMaster`; 2).
- `startUnderwater @ 0x10011da5c` (existing pygmies when the island goes underwater):
  guard (§25), then `waitUnderwater`, `zrot0`, `layer(pygmyUnderwaterPygmyDisplayMarker)`
  +labels, `→ PygmyStartUnderwater`.
- `waitUnderwater @ 0x10011dfb8`: Tom in the closet → return; held by Xenu →
  `stopBeingHeldByXenu`, `→ PygmyDie`; died in a brawl → `stopThetanBrawl`, `→ PygmyDie`;
  returns early for any of 60 busy conditions (FREE-like plus beheaded, T != 0, tilt);
  otherwise S28, `sel0`, hurricane-drop, drag-end, doodle-end, tilt 0, `falling = 0`,
  `mbUnderwater`, `ignoreGrav = 1`, side and floor off,
  `x = (r % 32001 + 8000) / 100` (80..400), `y = (r % 12001 + 16000) / 100` (160..280),
  `vel.xy = 0`, `z = -277.12`, `zrot0`, alpha 0, `→ PygmyWaitUnderwater`.
- `startSinking:::` (Island; gravity on, `mbActiveUnderwater`, `sel1`, bait master →
  `startBaitMaster`; 2).
- `stopUnderwater @ 0x10011e9c4` (back to land): if underwater and not hit by a power or
  eaten by piranhas: drag-end, `mbUnderwater = 0`, gravity on, `mbActiveUnderwater = 0`,
  `stopHoldingChum`, `stopBeingEatenByPiranha`, `sel1`, alpha 1, side and floor on,
  `layer(pygmyDisplayMarker)` +labels, `→ PygmyStopUnderwater`; otherwise `→ PygmyDie`.
- Swimming: `swim:::` (min, max, noReset: first: timer, `vel.y = 50`, direction rule with
  ±10; timeout → `vel.xy = 0`, 2; `noReset` unused), `panicSwim:::` (±30, `vel.y = 30`),
  `checkForLaserSharkEnabled:::` (enabled → `PygmyUnderwaterPanicIdle`),
  `checkForLaserSharkDisabled:::` (disabled → `PygmyUnderwaterIdle`).

### 14.2 Underwater hazards

| Method | Condition | Effect |
|---|---|---|
| `pickedUpByWhirlPool:` (SpriteLogic override) → BOOL | not pressed, not removed from story, selectable, underwater, not already picked up, not eaten by piranhas / drain / pain drain / laser / jellyfish / power / laser shark, not skewered, not bait master, not dragged | `mbPickedUpByWhirlPool`, `stopHoldingChum`, tilt 0, `vel.xy = 0`, reference, `zrot0`, `sel0`, `→ PygmyPickedUpByWhirlPool` |
| `droppedByWhirlPool:` | picked up | `vel.xy = 0`, clear, `sel1`, `→ PygmyDroppedByWhirlPool` |
| `canBeHitByChainSpear:`, `canBeHitBySharkLaser` | underwater, not skewered, not eaten by / hit by the laser shark | |
| `hitByChainSpear:` | | `mbSkewered`, tilt 0, `vel.xy = 0`, `zrot0`, `sel0`, `stopBeingEatenByPiranha`, `stopHoldingChum`, whirlpool drop; facing left (`xScale <= 0`) → `xScale = 1`, `PygmyHitByChainSpearLeft`; else `mbSkeweredRight`, `PygmyHitByChainSpearRight`. `hitByChainSpearRetract:` (`…RightRetract` / `…LeftRetract`), `hitByChainSpearComplete:` (`sacrifice Skewered`, `→ PygmyDie`) |
| `hitBySharkLaser` | | `mbHitBySharkLaser`, tilt 0, piranha / chum stops, `vel.xy = 0`, `zrot0`, `sel0`, `sacrifice ZappedByLaserShark`, whirlpool drop, `→ PygmyHitBySharkLaser` |
| `startEatenByLaserShark:` / `eatenByLaserShark:` / `swallowedByLaserShark:` / `stopEatenByLaserShark:` | | `mbEatenByLaserShark` + `sel0`; `sacrifice FedToLaserShark`, `PygmyEatenByLaserShark`; `→ PygmyDie`; release: clear, gravity off, `sel1`, `PygmyDropUnderwater` |
| `canBeHitByJellyfish:` | underwater, not hit by a jellyfish or power | |
| `suffocatedByJellyfish:` / `stungByJellyfish:` | | whirlpool drop, `mbHitByJellyfish`, `mJellyfishLogic`, `vel.xy = 0`, `zrot0`, `sel0`, gravity off, `sacrifice JellyfishScubaMask` / `StungByJellyfish`, `PygmySuffocatedByJellyfish` / `PygmyStungByJellyfish`. `jellyfishAttackComplete:` (`→ PygmyDie`, nil), `animJellyfishAttackComplete` (frame: `[j onJellyfishAttackComplete:self]`), `onJellyfishSuffocateAttackComplete:::` |
| `checkJellyfishCollision` (dropped pygmy) | jellyfish enabled; probe `(x, y + originY + 20)` | for a non-leader that `canInteractWithPygmy`: probe within 30 of its pos → `EatenByJellyfish`, `PygmyHitByJellyfish`, `[j eatPygmyLogic:self]`; within 30 of `(j.x, j.y - 60)` → `DrankByJellyfish`, `PygmyDrankByJellyfish`, `[j drinkPygmyLogic:self]`; the circle leader: within 30 of `circleJerkPos` → `ElectrocuteByJellyfish`, `PygmyShockedByJellyfish`, `[j shockPygmyLogic:self]` (no reference kept). Each: touch released, `mbHitByJellyfish`, `vel.xy = 0`, `zrot0`, `sel0`, gravity off; YES |
| `canBeTouchedByUnderwaterPower` | underwater, not hit by a jellyfish or power | |
| `canBeTouchedByUnderwaterPower:` (kind) — **the hit itself** despite the name | | whirlpool drop (clears the flag), `mbHitByUnderwaterPower`, `vel.xy = 0`, `zrot0`, `sel0`, gravity off; kind 1..5 → `sacrifice HitByUnderwaterPowerCold / Heat / Oil / Acid / Air` and `PygmyHitByUnderwaterPower<Kind>`; other kinds: no behaviour |
| `canHoldChum:` / `holdChum:` / `dropChum:` / `stopHoldingChum` | underwater, not busy, not bait master | `mbHoldingChum`, reference, gravity off, `vel.xy = 0`, `zrot0`, `PygmyHoldChum`; drop → gravity on, `PygmyUnderwaterPanicIdle`; stop → `[chum dropChum:self]` |
| Bait master (`startBaitMaster` / `stopBaitMaster`, controller and the anchor) | underwater, not skewered | `mbBaitMaster`; active swimmers: gravity off, `vel.xy = 0`, whirlpool drop, drag-end, `PygmyStartBaitMaster`; stop → gravity on, `PygmyStopBaitMaster`, `stopBeingEatenByPiranha` |
| Piranhas: `canBeEatenByPiranha:`, `eatenByPiranha:`, `eatenByPiranhaComplete:`, `stopEatenByPiranha:`, `stopBeingEatenByPiranha`, `eatingPiranhaLogic` | bait master, active, not busy | `mEatingPiranhaLogic`, `sel0`, `PygmyPiranhaAttack`; complete: `mbEatenByPiranha`, `sacrifice FedToPiranha`, `PygmyPiranhaAttackComplete`; interrupted: `sel1`, `PygmyStopPiranhaAttack` |
| Drain: `suckedIntoDrain:` (guard §25), `escapeFromDrain:`, `fallIntoDrain:::` | | `mbSuckedIntoDrain`, gravity off, `sel0`, `vel.xy = 0`, `zrot0`, alpha 1, `PygmySuckedIntoDrain`; escape: clear, gravity on, `sel1`, `PygmyEscapeFromDrain`; `fallIntoDrain:::` moves to `(d.x, d.y + 16)` in 1 s, then `[d pygmyFallIntoDrainComplete:self]` |
| Pain drain mini-game: `startPainDrain` (guard), `checkPainDrainCollision:`, `painDrainCanTurn:::`, `dieFromPainDrain` | | start: `mbPainDrain`, gravity on, `sel0`, `vel.xy = 0`, `pos = (240, 250, -277.12)`, `zrot0`, alpha 1, `PygmyPainDrainStart`. Steering `vel.x = rawAccel.y * -1000` (§4.1 step 6). Each frame (not spiked): standing on any platform of `[drain painDrainPlatformLogicArray]` (`checkCollision:lastPos:`) changes → `PygmyPainDrainLand` / (`mbPainDrainCanTurn = 0`, `PygmyPainDrainFall`); on a platform, allowed to turn and moving against the facing → `PygmyPainDrainTurn`; leaving `0 <= y <= 350` (bounds clamp) → spiked: `vel.xy = 0`, `mbPainDrainSpiked`, gravity off, `[drain stopPainDrain]`, `PygmyPainDrainSpiked`. `dieFromPainDrain`: `sacrifice PainDrainSpikes`, underwater layer, `→ PygmyDie` |

## 15. No Pee category (`PygmyLogic(NoPee)`, 19 methods)

Underwater swimming mini-game driven by `NoPeeLogic` (`[listener noPeeLogic:self]`). While
`mbNoPee`, `onGameFrame:` is replaced by `handleNoPeeMovement:`.

- `startNoPee`: `mbNoPee`, all sub-flags cleared, `mNoPeeSwimXVel = mNoPeeSwimTargetXVel =
  100`, gravity off, side and floor off, `sel1`, thrust 0, turn time 0, `pos.xy = (250, 12)`,
  `scale = (0.5, 0.5)`, `rot.z = 0`, `mNoPeePosVel.xy = 0`, `mNoPeeEmbedLogic =
  getLogic(84)` → `NoPeeEmbedBubbles`, alpha 1, `→ PygmyNoPeeStart`,
  `mDragLabelYOffset = -20`.
- `onNoPeeIntroComplete`: intro complete, scale 0.5, `mNoPeePosVel.x = 100`, embed on
  channel 17 → `NoPeeEmbedSwimBubbles`, `→ PygmyNoPeeGlide`.
- Touch: `touchBeganNoPee:allTouches:` (intro complete, alive, no active touch, not
  hibernating): bind the touch; first touch starts the game (`[noPee onPygmyStartSwimming]`);
  gravity on, `mbNoPeeThrustOn`; left half of the screen (`PocketGod` group x <= 240): if
  swimming right, start a turn left (`mbNoPeeChangingDirectionDownward` when pointing
  down-right: `-90 < rot.z < 0` or `270 < rot.z < 360`), `mbNoPeeSwimmingLeft`, turn time 0,
  `mbNoPeeChangeDirection`, target −100, `→ PygmyNoPeeRoll`; right half mirrored
  (`-180 < rot.z < -90` or `180 < rot.z < 270`, target 100); no turn → `PygmyNoPeeSwimFlap`.
  `touchEndedInsideNoPee:allTouches:` → thrust off, touch nil.
  `onPygmyNoPeeRollComplete:::` clears the change flag.
- `handleNoPeeMovement: @ 0x10018ad28`, reconstructed from the disassembly:

```
if !noPee || !started || killed:            // ragdoll
    if !ignoreGrav: nv.y += 50 dt
    if turnToStone: nv.y -= 50 dt
    vel = nv; pos += vel dt; moveNameLabel; return
s    = swimmingLeft ? -1 : 1
tilt = 20 s
a    = fmodf(rot.z + tilt - 90 + 540, 360) - 180      // heading, deg
w    = down ? swimmingLeft : !swimmingLeft
if thrustOn || down:
    R = w ? -600 : 600
    if (w && rotVel.z > 0) || (!w && rotVel.z < 0): rotVel.z = 0
    F = 500
else: F = 0; R = 0
T = 300 s
inRange = w ? (-160 < a <= 0) : (0 <= a < 160)
if !inRange: T = -300 s; if !down: R = 0
thrustVel = min(thrustVel + (F - 0.9 thrustVel) dt, 200)
ang = fmodf(a - tilt + 90 + 360, 360); (sn, cs) = sincos(rad(ang))
nv.x -= nv.x * 0.5 dt
nv.y += (50 - 0.5 nv.y) dt
if swimXVel != target: turnTime += dt; f = min(turnTime * 0.5, 1); swimXVel += f (target - swimXVel)
vel = nv; vel.x += thrustVel cs + swimXVel; vel.y += thrustVel (sn - 0.6)
pos += vel dt
rotVel.z += (R + T - 0.9 rotVel.z) dt
rot.z = a + rotVel.z dt - tilt + 90
if down && ((0 < rot.z < 180) || (-360 < rot.z < -180)): down = 0
checkNoPeeCollision:&lastPos; moveNameLabel; lastPos = pos
```

  (`nv` = `mNoPeePosVel`, `rotVel` = `BCLogic::mRotVel`, `down` =
  `mbNoPeeChangingDirectionDownward`, `turnTime` = `mNoPeeTurnAroundTime`.)
- `checkNoPeeCollision:` (started, alive): `y > 1000` → `collideAndDie`,
  `sacrifice WaterWingsDrowned`, `→ PygmyNoPeeHitPillar`; `y < 0` → `y = 0`.
- Deaths: `canBeHitByNoPeeObject` (alive and intro complete), `collideAndDie` (once:
  killed, gravity off, `mNoPeePosVel.xy = 0`, `[noPee onPygmyKilled]`, `rot.z = 0`,
  `scale.y = 0.5`), `hitByNoPeeSeaBlob:` (`ImpregnatedByBlob`, `PygmyNoPeeHitSeaBlob`),
  `hitByNoPeeMine:` (`Stoned`, `PygmyNoPeeHitMine`), `hitByNoPeeClam:` (snap to the clam,
  `EatenByClam`, `PygmyNoPeeHitClam`), `noPeeRunOutOfBreath` (`RunOutOfBreath`,
  `PygmyNoPeeHitPillar`). `NoPeeLogic` records them with `triggerActiveSacrifice` (§8.1).
- Frame actions and items: `animNoPeeEmbedDie` (embed `NoPeeEmbedDie`),
  `animNoPeeFloatUpward` (gravity on), `animNoPeeSinkDownward` (`mbNoPeeTurnToStone`),
  `waitUntilOffScreen:::` (`y > 1000 || y < 0` → 2), `onNoPeeHitComplete:::` (clear
  channel 17, `[noPee stopNoPee]`, 2), `noPeeSwimmingLeft` getter.

## 16. Battle of the Gods (`PygmyLogic(BattleOfTheGods)`, 41 methods)

Two pygmies fight for the idols (IdolManager, `TempleDoorLogic`). Each fighter has an
episode node (`mpStructXmlEpisodeNodeDef`, the opponent's in
`mpOpponentStructXmlEpisodeNodeDef`; struct-xml.md): `+0x70` realm (0 Oceanus, 1 Aeris,
2 Infernis, 3 Animus, 4 Pax), `+0x74` attack (0 strong, 1 counter, 2 fast), `+0x78` style
(0 physical/sword, 1 energy). The player fighter uses global logic objects 377..387, the
enemy 388..398 (same layout, +11): A sword, B shield, C, D..G energy balls 1..4, H, I beam,
J, K beam end. Lava flames: 376 (player side), 375 (enemy side).

- `startBattleOfTheGods:` / `startBattleOfTheGodsEnemy:` (opponent): `mbBattleOfTheGods`
  (enemy: `mbBattleOfTheGodsEnemy`), opponent reference, `mIslandYPos = 55`, tween
  channels 7..13 = displays of objects 377, 378, 379, 384, 385, 386, 387 (enemy 388, 389,
  390, 395, 396, 397, 398), objects 377/378 (388/389) `→ BlankAnim`, `sel0`,
  `spawnFromBattleOfTheGods`.
- `spawnFromBattleOfTheGods`: `pos = (100, 400)`, scale (1,1); enemy `x = 380`,
  `xScale = -1`; `falling = 1`, gravity on, `mbKilledDuringBattle = 0`, floor on,
  `→ PygmyBattleSpawn`. `battleRespawn:::` calls it; `onBattleReady:::` →
  `[[listener templeDoorLogic:self] onPygmyBattleReady:self]`.
- `startBattlePowerUp:opponentIdol:` (store both nodes, `→ PygmyBattlePowerUp`).
- `startBattleAttack:opponentIdol:` (store both nodes; the player side fires
  `episode FastAttack / CounterAttack / StrongAttack` by its attack and
  `OceanusAttack / AerisAttack / InfernisAttack / AnimusAttack / PaxAttack` by its realm).
  Behaviour: fast → `PygmyBattleEnergyAttackFast` (energy) / `PygmyBattleAttackFast`;
  counter → opponent also counter: `PygmyBattleEnergyAttackCounterTie` /
  `PygmyBattleAttackCounterTie`, else energy `setBattleBeamPos` +
  `PygmyBattleEnergyAttackCounter` / `PygmyBattleAttackCounter`; strong → energy
  `setBattleBeamPos` + `PygmyBattleEnergyAttackStrong` / `PygmyBattleAttackStrong`.
- `setBattleBeamPos`: beam I (385/396) `scale.x = xScale * 5`; opponent strong: physical
  `scale.x = xScale * (|opp.x - x| - 55) / 100`, energy `xScale * 1.1`; beam end K
  (387/398) `x = x + beam.scale.x * 100 + xScale * 23`.
- Running and results: `battleCheckNoRunBack:::` (player `x <= 100` / enemy `x >= 380` →
  snap, face forward, `PygmyBattleNoRunBack`), `battleRunBackToStart:::`,
  `battleStrongRunTowardOpponent:::` (within 50: opponent strong →
  `PygmyBattleAttackStrongTie`), `battleFastRunTowardOpponent:::` (within 100: opponent
  fast → `PygmyBattleAttackFastTieAgainstEnergy` (energy) / `PygmyBattleAttackFastTie`),
  `battleEnergyFastMoveTowardOpponent:::` / `battleEnergyFastMoveAboveOpponent:::` (place
  at an offset from the opponent: physical opponent `TBL[attack == 1]` from the 2-float
  table at `0x10029d9b0`, otherwise 100; energy fast opponent above: `x += xScale * 60`),
  `battleEnergyFastWaitForOpponent:::` (within 100 → 2; unused by the data),
  `checkForStrongAttackWin:::` (opponent counter → `PygmyBattleAttackStrongWin`),
  `checkForEnergyStrongAttackWin:::` (counter → `PygmyBattleEnergyAttackStrongWin` +
  `[opp killedInBattle]`; strong physical → `setBattleBeamPos` +
  `[opp parryAgainstStrongEnergy]`; → `PygmyBattleEnergyAttackStrongTie`),
  `checkForFastAttackWin:::` (opponent fast: energy → `PygmyBattleEnergyAttackFastTie`;
  counter → `[opp battleCounterFastAttack]`; strong → `PygmyBattleAttackWin`),
  `checkForAttackTie:::` (same attack, player side → `[IdolManager onBattleAttackTie]`),
  `battleShowLavaFlame:::` (flame `→ BattleLavaPitFlame`), `battleFall:::` (ignoreIsland:
  falls with x clamped to 20..460, or -50..530 once killed; killed and below 0 → stop at
  y 0, tilt 0; landing → `vel.x = 0`).
- Selection: `pushSequenceBasedOnOpponentWeaponType:::` (energyId, swordId: opponent style
  1 → energyId else swordId) and `pushSequenceBasedOnOpponentBattleType:::`
  (fastEnergyId, fastSwordId, strongSwordId: opponent fast: style 0 → fastSwordId,
  style 1 → fastEnergyId; opponent strong physical → strongSwordId; else nothing).
- Hits and deaths: `killedInBattle` (`vel = (±500, 600)` away from the opponent,
  `falling = 1`, `mbKilledDuringBattle`, `PygmyBattleKilled`,
  `[IdolManager onBattleDamageComplete]`), `wasKilledDuringBattle`, `fatalityInBattle`
  (`vel.xy = 0`, records `BattleFatality` at once through `onTriggerSacrifice:`,
  `falling = 0`, gravity off; by the **opponent's** realm: `PygmyBattleFatalityOceanus`,
  Aeris (beam object → `PygmyBattleFatalityAerisTornado` + `PygmyBattleFatalityAeris`),
  `…Infernis`, `…Animus`, `…Pax`), `hitByBattleEnergyBall` (`PygmyBattleHitByEnergyBall`),
  `killedByBattleEnergyBall` (`killedInBattle`), `parryAgainstStrongEnergy`
  (`PygmyBattleAttackStrongEnergyTie`), `battleCounterFastAttack` (energy →
  `PygmyBattleEnergyAttackCounterFastAttack` else `PygmyBattleAttackCounterFastAttack`),
  `removeFromBattleOfTheGods` (clear channels 7..13, `die:::` called directly).
- Energy balls: `throwEnergyBall:` (start `(x + xScale * 18, y + 111, z)`; no damage unless
  the opponent's attack is strong; `activateWithStartPos:targetPygmyLogic:opponent`),
  frame actions `animBattleThrowEnergyBall1..4` (balls 380..383 / 391..394, the 4th is
  `setLastBall:1`).
- Frame actions: `animBattleChangeWeapon` (realm R and style: energy → A
  `PygmyEnergyBallSword<R>`, B `PygmyEnergyBallShield<R>`, C `PygmyEnergy<R>`, D..G and J,
  K `PygmyEnergyBall<R>`, H `PygmyEnergyShield<R>`, I `PygmyEnergyBeam<R>`; physical → A
  `PygmySword<R>`, B `PygmyShield<R>`), `animBattleTie` (player side
  `onBattleAttackTie`), `animBattleEnergyCounterHit` (opponent fast → killed),
  `animBattleStrongHit` (counter beats fast, strong beats counter → opponent killed),
  `animBattleFastHit` (opponent counter → `battleCounterFastAttack` +
  `PygmyBattleAttackFastWaitForCounter`; opponent strong → killed).

## 17. Konkey Dong (`PygmyLogic(KonkeyDong)`, 9 methods)

`kong = [listener kongStatueLogic:self]`, `L = kong.konkeyDongLevelDisplayObject.pos`.

- `startKonkeyDong`: `mbKonkeyDong`, gravity off, side and floor off, `sel1`,
  `vel.xy = 0`, `pos = (34, 350, -277.12)`, scale 0.75, `zrot0`, alpha 1,
  `→ PygmyKonkeyDongStart`. `konkeyDongIntroComplete:::` (started, no transition, `sel1`,
  `[kong pygmyIntroComplete]`), `jumpToNextKonkeyDongLevel`
  (`→ PygmyKonkeyDongJumpToNextLevel`), `konkeyDongLevelJumpComplete:::` (gravity on,
  running, transition and platform flags cleared).
- Per frame in `onGameFrame:` (started, not in a transition, not hit): off the ladder:
  pick timer (`mKonkeyDongPickTime -= dt`, at 0 drop the pick and stop running),
  `vel.x = clamp(rawAccel.y * -1000, -100, 100)`, keep within x 0..level width (`*(float
  *)(konkeyDongLevel + 0x30)`); on the ladder: ladder touch → `vel.y = ±100` (up/down),
  climbing; no touch → `vel.y = 0`; changes switch `PygmyKonkeyDongLadderClimb` /
  `PygmyKonkeyDongLadderIdle`.
- `touchBeganKonkeyDong:allTouches:` (not falling, not hit): touch in the `PocketGod`
  group; right half while started, off the ladder, without the pick and no transition →
  jump (`falling = 1`, jump touch, `→ PygmyKonkeyDongJumpUp`); left half → ladder touch,
  up when `y > 160`. Release: §5.3. `konkeyDongJumpUp:::` (0.2 s of `vel.y = 300`, then
  `PygmyKonkeyDongJumpDown`), `konkeyDongFall:::` (gravity on, forever),
  `onStartKonkeyDongFall:::` (gravity on).
- `checkKonkeyDongCollision:` (every frame; playing, not hit, no transition):
  - boulders (`konkeyDongBoulderLogicArray`, active, started): jumping over one
    (`|bx - x| < 16`, `0 < y - by < 45`, falling) → `mbKonkeyDongJumpBoulder`; touching
    (`|bx - x| < 16`, `by - 20 < y < by + 14`): with the pick
    `[listener onKonkeyDongScore:100]` and `[b crackBoulder]`; else hit: gravity on,
    `vel = (100, 400)`, `sacrifice KonkeyDongBoulder`, `→ PygmyKonkeyDongHit`;
  - beetles (positions + L): same jump marker (`mbKonkeyDongJumpBeetle`); touching
    (`ey - 20 < y < ey + 10`): pick → score 100 + `[e smashBeetle]`; falling onto a beetle
    that is not rolled up → bounce `vel.y = 525`, `[e jumpOnBeetle:self]`; else hit with
    `sacrifice KonkeyDongBeetle`;
  - picks: within 16 → holding, `[p deactivate]`, `mKonkeyDongPickTime = 5`,
    `→ PygmyKonkeyDongGrabPick`;
  - ladder: while on it, stay between its bottom (`L.y + lad.y`) and top
    (+ `ladderHeight`), else leave it (gravity on, flags off);
  - platforms (`checkCollision:lastPos:` on the first started or start platform):
    leaving → falling, `PygmyKonkeyDongFall` (without the pick); landing → first landing
    `PygmyKonkeyDongFirstLand`; later landings award the jump bonuses
    (`episode JumpBoulder` / `JumpBeetle` + `[kong showBonusSplash:&pos]`), clear falling
    and running, `PygmyKonkeyDongLand`;
  - `xScale = vel.x > 0 ? 0.75 : -0.75`;
  - on a platform, started, without the pick: ladder entry by touch (top when going
    down, bottom when going up) → snap to the ladder, gravity off, climbing,
    `PygmyKonkeyDongLadderClimb`; on the final platform left of `konkeyDongLogic.x + 110`
    → `vel.xy = 0`, gravity off, level transition, `[kong endLevel]`,
    `PygmyKonkeyDongWatchKidnap`; else start running (`PygmyKonkeyDongRun`).
- Falling out → `KonkeyDongFall` (§4.1 step 17); death stops the game (`die:::` →
  `[kong stopKonkeyDong]`; the controller clears its Konkey Dong state).

## 18. The Runs (`PygmyLogic(TheRuns)`, 10 methods)

Lava runner started from the crack hole (`ch = [listener crackHoleLogic:self]`).

- `startTheRuns` (in the crack hole): `mbTheRuns`, gravity off, side and floor off, `sel0`,
  `vel.xy = 0`, `pos = (34, 350, -277.12)`, scale 0.75, `zrot0`, alpha 1,
  `→ PygmyTheRunsStart`. `theRunsBurnPygmyComplete` (from the platform: gravity on,
  burn complete, `vel.x = 300`, `sel1`, `PygmyTheRunsRun`).
- Per frame (§4.1 step 10): after the burn, not side-hit, not exploded:
  `vel.x = min(vel.x + 50 dt, 600)`; `y <= 13` → lava: `vel.xy = 0`, gravity off,
  `mbTheRunsFallInLava`, `y = 13`, `sacrifice LavaDrown`, `PygmyTheRunsSplash`.
- `touchBeganTheRuns:allTouches:` (started, not exploded, on a platform): bind the touch,
  not skidding, jumping, `PygmyTheRunsJumpUp`. `theRunsJumpUp:::` (`k = (vel.x - 300) /
  300`; duration `0.05 k + 0.15`; `vel.y = 100 k + 300` each frame; then
  `PygmyTheRunsJumpDown`). Release → `PygmyTheRunsJumpDown`.
- `checkTheRunsCollision:` (every frame unless in lava, exploded or sinking): platform
  codes from `checkCollision:lastPos:`: 1 land; 2 side hit (`vel = (-100, 200)`,
  `PygmyTheRunsHitSide`); 3 still on; 4 sinking platform (`vel.y = 500`, falls,
  `PygmyTheRunsFall`); 5 rough landing. Stalactite → `vel = (0, 500)`,
  `PygmyTheRunsHitStalactite`. Turd → skid (`vel.x = max(vel.x - 300, 300)`, `vel.y = 0`,
  touch released, `PygmyTheRunsTurdSkid`). Rock → `vel.xy = 0`, gravity off,
  `sacrifice LavaRock`, `PygmyTheRunsRockExplode`. Landing changes: first landing starts
  the run (gravity off, `[platform burnPygmy:self]`); a hard landing (previous
  `vel.y < -900`) → speed −100 (min 300), `PygmyTheRunsTumble`; rough → same loss,
  `PygmyTheRunsStumble`; else `PygmyTheRunsRunStartFast` (>= 600) / `PygmyTheRunsRun`.
- `theRunsRun:::` (reaching 600 → `PygmyTheRunsRunFast`), `theRunsFall:::`,
  `onStartTheRunsFall:::`, `onTheRunsSplashComplete:::` / `onTheRunsExplodeComplete:::`
  (`[ch stopTheRuns]`).

## 19. Moron Pests (`PygmyLogic(MoronPests)`, 12 methods)

Turtle race quiz: the pygmy rides `getLogic(50)` (the `MoronTurtleLogic`); the Moron snake
(`[listener moronSnakeLogic:self]`) runs the game.

- `startMoronPests`: `mbMoronPests`, gravity off, side and floor off, `vel.xy = 0`,
  `pos = (60, 350, -277.12)`, scale 0.75, `zrot0`, alpha 1, `→ PygmyMoronPestsStart`.
  `moronPestsIntroComplete:::` (started, `[snake pygmyIntroComplete]`, gravity off,
  `vel.x = 500`, target 500).
- `checkMoronPestsCollision:`: before landing on the turtle: `y <= 170` → stop at 170,
  gravity off, `[turtle pygmyJumpOnBack:self]`; while running: `y <= 130` → stop at 130
  and the first time `PygmyMoronPestsLand`. `jumpOnMoronTurtleBackComplete:` (gravity on,
  start running).
- Per frame (§4.1 step 11): `t = min(turtle.vel.x - 10, 500)`; if `t` is below the
  current target, the target only decays by 10/s; `vel.x += (target - vel.x) * 0.5`.
- Answers: `onMoronPestsAnswerCorrect` (target `min(target + 100, 540)`,
  `PygmyMoronPestsAnswerCorrect`), `onMoronPestsAnswerIncorrect` (target −10,
  `PygmyMoronPestsAnswerIncorrect`).
- Ending: `eatenByMoronTurtle:` (eaten, gravity off), `eatenByMoronTurtleStopMoving:`
  (`vel.x = 0`), `eatenByMoronTurtleComplete:` (`sacrifice EatenByMoronTurtle`,
  `PygmyEatenByMoronTurtle`), `onEatenByMoronTurtleEatComplete:::` (`[snake stopMoronPests]`),
  `onStartMoronPestsFall:::`, `moronPestsRun:::` (forever).

## 20. Ooga Jump (`PygmyLogic(OogaJump)`, 8 methods)

Doodle-jump on the clouds. Gravity 1500 while jumping (0 with the jet pack), horizontal
steering from the accelerometer, no death above y 820.

- `checkDoodleCloudCollision:` (every frame): falling, healthy, T 0, not thrown / invalid /
  tumbling / zombie / ghost / beheaded / knocked down, doodle clouds enabled, `y >= 350`,
  `z == -277.12` (exact float compare). Not yet jumping and nobody else is: a cloud of
  `cloudLogicArray:` with `isDoodleCloud`, `c.x + 10 < x < c.x + 140`, `y < c.y + 21` and
  `lastPos.y > c.y + 21` → `mbDoodleJumping`, `vel.y = 1800`, `zrot0`,
  `PygmyDoodleBounceStart`, `[listener onPygmyStartDoodleJump:self]`. Jumping: the first
  doodle cloud whose `checkCollision:lastPos:` hits ends the check (the cloud calls the
  bounce methods).
- Cloud callbacks: `doodleBounce:` (jet pack off, `vel.y = 600`, `y = c.y + 10.5`; on ice
  → `PygmyDoodleBreakIceBounce`, else `PygmyDoodleBounce`), `doodleElectricBounce:`
  (`vel.y = 1400`, `PygmyDoodleElectricBounce`), `doodleSnowBounce:` (`mbDoodleOnIce`,
  `vel.y = 500`, `PygmyDoodleSnowBounce`), `doodleKnockedDown:` (not with the jet pack:
  knocked down, `vel.y = 1000`, `vel.x = x > 240 ? -30 : 30`, `PygmyDoodleKnockedDown`,
  `[listener onPygmyDoodleJumpFall:self]`), `doodleJetPack:` (`mbDoodleJetPack`,
  `vel.y = 500`, `PygmyDoodleJetPack`), `doodleJetPackComplete:::` (`vel.y = 1000`, jet
  pack off).
- `doodleCheckOffScreen:` (bottom): fell 8 below the screen bottom → knocked down,
  `vel.x = (240 - x) * 80 / 240`, `PygmyDoodleKnockedDown`, `onPygmyDoodleJumpFall:`.
- Death while jumping is recorded as `Flattened` (§3.6); the controller ends the game in
  `onPygmyStopDoodleJump:` (ChallengeManager game 4).

## 21. `PygmyBottomLogic` (10 methods)

A `SpriteLogic` for detached body parts: the legs of a pygmy cut in half (global logic 0)
and the locust skulls (464..466) and bones (467..469). Ivars: 280
`mPygmyBottomLogicListener` (the controller; only `isOceanEnabled` is used), 288
`mbIsActive`.

- `initWithDisplayObject: @ 0x100112c9c`: SpriteLogic init, gravity force (0, -3000, 0, 1).
  `setPygmyBottomLogicListener:` (retain), `dealloc` (release), `isActive`.
- `onGameFrame:` (not shut down): unless ignoring gravity `vel += gravity * dt`; `pos +=
  vel * dt` (own semi-implicit Euler, no SpriteLogic call); ocean enabled and `y < 16` →
  `vel.xy = 0`, `y = 16`; `y < -100 || y > 820` → `→ PygmyDie` (every frame).
- `fall:::` (ignoreIsland, unused): first: gravity on; ocean enabled and `y <= 16` →
  gravity off, `vel.xy = 0`, `y = 16`, 2; else 0.
- `die:::`: `mbIsActive = 0`, remove the display; 2.
- `tossBottom:` (pygmy): pos = pygmy pos, `vel.xy = pygmy vel.xy * 1.2`,
  `[pygmyDisplay insertBefore:bottomDisplay]` (the bottom joins the pygmy's group just
  before it, so it is drawn behind the pygmy), `→ PygmyBottomInit` (does not set
  `mbIsActive`).
- `spawnLocustSkull:` / `spawnLocustBone:` (pygmy): pos = pygmy pos, `y += 26` / `50`,
  `vel.x = (int)(r % 4001 - 2000) / 10`, `vel.y = (r % 501 + 1000) / 10`, active,
  gravity off, inserted the same way, `→ PygmyBottomLocustSkull` / `PygmyBottomLocustBone`.

## 22. Personas and statistics

### 22.1 `Persona` (21 methods)

Ivars: 8 `mPreferences`, 16 `mId`, 24 `mName`, 32 `mDescription`, 40 `mMood`, 44
`mTotalSacrifices`, 48 `mSacrificeDictionary`, 56 `mTotalGifts`, 64 `mGiftDictionary`.
Preference keys, formatted with the id: `%@:Name`, `%@:Description`, `%@:PersonaMood`
(default 2), `%@:TotalSacrifices` (0), `%@:SacrificeDictionary`, `%@:TotalGifts` (0),
`%@:GiftDictionary`.

- `initWithId:defaultName:defaultDescription:preferences:`: reads all keys
  (`setDefaultPreference:` stores the default when missing and returns the stored value),
  copies the dictionaries into mutable ones (capacity 5), `updateMood`.
- `refreshFromPreferences`: re-reads everything; if the stored total is below the sum of
  the dictionary counts, logs `NEW SACRIFICE COUNT: %@ %d %d` / `NEW GIFT COUNT…` and
  raises the total to the sum; `updateMood`. (The gift dictionary is re-read with
  `setDefaultPreference:asDictionary:nil`.)
- `setName:` / `setDescription:` store and write the preference.
- `updateMood`: `r = sacrifices`, or `sacrifices / gifts` (float) when gifts > 0;
  `r < 4 && sacrifices >= 21` → 0; `r < 7 && sacrifices >= 11` → 1; `r < 10` → 2;
  `r < 20` → 3; `r < 30` → 4; `r < 40` → 5; else 6; stored in `%@:PersonaMood`. Names
  (`getMoodDescription:`): 0 Happy, 1 Content, 2 Normal, 3 Sad, 4 Annoyed, 5 Angry,
  6 FreakedOut; textures `PygmyMoodHappy` … `PygmyMoodFreakedOut` (Normal is the
  default).
- Getters `name`, `description`, `mood`, `totalSacrifices`, `totalGifts`.
- `setTotalSacrifices:`, `incrementTotalSacrifices`, `setTotalGifts:`,
  `incrementTotalGifts` (store, write, `updateMood`); `sacrificeCount:` / `giftCount:`
  (dictionary int or 0); `addSacrifice:` (nil ignored) / `addGift:` (no nil check):
  `dict[id] += 1`, write; `sortedSacrificeIdArray` / `sortedGiftIdArray`: distinct counts
  in descending order, each followed by `allKeysForObject:` of that count (key order
  within a count is the dictionary's).

### 22.2 `PygmyPersona` (9 methods)

72 `mbActive`, 76 `mPersonaPersonality`, 80 `mPygmyLogic`. `setPygmyLogic:` (retain,
`[logic setPygmyNameLabel:mName]`), `setName:` (super + label), `setActive:` / `isActive`,
`setPersonaPersonality:` / `personaPersonality`, `pygmyLogic`.

### 22.3 `TribePersona` (11 methods)

72 `mPygmyPersonaArray`, 80 `mDeadPygmyPersonaArray`, 88 `mLivingPygmyPersonaArray`,
96 `mGuestPygmyPersonaArray` (never allocated: `addGuestPygmyPersona:` is a no-op).

- `pygmyPersonaArray` (getter). `addPygmyPersona:` → active, appended to the pygmy array and to the **dead** list (all
  personas start dead, waiting to be resurrected).
- `resurrectPygmyLogic:` (logic): `per = dead[0]` (`objectAtIndex:0`, raises when the list
  is empty), bind both ways, move to the living list.
- `sacrifice:pygmyLogic:` (id, logic): id → tribe `incrementTotalSacrifices` +
  `addSacrifice:`; the pygmy's persona: id → the same on the persona; unbind
  (`setPygmyLogic:nil`), remove from living, and if still active append to the end of
  the dead list (so personas rotate).
- `addGift:pygmyLogic:`: only when the pygmy has a persona: tribe and persona `addGift:` +
  `incrementTotalGifts`.
- `activateGuestPygmyPersona:` (inactive → active; if not living insert at index 0 of
  the dead list: the next resurrection picks it), `deactivateGuestPygmyPersona:` (active
  → inactive, removed from the dead list).
- `refreshFromPreferences`: super plus every pygmy persona (guests are not refreshed).

### 22.4 `GodPersona` (9 methods)

72 `mGodType` (`%@:PersonaGodType`, default 1), 80 `mAchievementDictionary`
(`%@:AchievementDictionary`). `achievementCount:`, `setAchievement:count:highScoreWins:`
(stores `n` when `highScoreWins && c < n` or `!highScoreWins && (n < c || c == 0)`),
`addIncrementalAchievement:` (+1), `achievementIdArray` (all keys), `setGodType:` /
`godType`, refresh.

### 22.5 Controller setup (`PocketGodViewController::loadPreferences…`)

God `God` "GENE SIMIANS" ("God of Thunder and Rocks that Roll"); tribe `Tribe` "TRIBE
OOKGA CHAKA" ("One Tribe, One World"); pygmies `Pygmy0` OOGA ("The fearless leader."),
`Pygmy1` BOOGA ("Booga is gross and smells horrible."), `Pygmy2` NOOBY ("Nooby is dazed
and confused."), `Pygmy3` DOOBY ("The happy-go-lucky islander."), `Pygmy4` KLIK ("Klik
likes to organize and build."), `Pygmy5` KLAK ("Klak likes to break things."), all
`addPygmyPersona:`; guests `CharliePygmy` "Charlie" ("Winning!.", P 1, activated when
`CharlieState == 1`) and `TomPygmy` "Tom" ("Show me the money!.", P 2).

## 23. Listener protocol `PygmyLogicListener` (159 methods) and the controller side

Implemented by `PocketGodViewController` (base class and `(Game)` category). A port can
split it into a read-only world query interface and an event sink. Methods marked † are
declared but never sent by the pygmy classes.

### 23.1 Queries

- Feature switches (48): `isOceanEnabled, isCoconutEnabled, isWoodEnabled,
  isBonFireEnabled, isVolcanoEnabled, isGravityEnabled, isOutHouseEnabled,
  isZombieGraveEnabled, isIdolPedestalEnabled, isTempleDoorEnabled,
  isApocalypseTempleDoorEnabled, isApocalypsePortalTempleDoorEnabled,
  isApocalypseTimeTemplePlatformEnabled, isLocustQueenEnabled,
  isApocalypsePlagueTempleFountainEnabled, isApocalypsePurificationTempleFountainEnabled,
  isClosetTempleThetanMeterEnabled, isClosetTempleCouchEnabled,
  isClosetTempleClosetDoorEnabled, isXenuTempleEnabled, isXenuEnabled, isTomPygmyAlive,
  isTimeTempleDoorEnabled, isLocustTempleDoorEnabled, isDanceTempleDoorEnabled,
  isPlagueTempleDoorEnabled, isPurificationTempleDoorEnabled, isClosetTempleDoorEnabled,
  isXenuTempleDoorEnabled, areNameLabelsEnabled, areGhostsEnabled, isSpiderTreeEnabled,
  isIceMonsterEnabled, isIceHoleEnabled, isGlacierEnabled†, isIglooEnabled,
  isApeWenchEnabled, isApeBongoEnabled, isApeAltarEnabled, isGoGoCageEnabled,
  isDanceDanceExecutionEnabled, isKongStatueEnabled, isMoronPestsEnabled,
  isJellyfishEnabled, isDreamBubbleEnabled, isInsideApocalypseTemple, isStormCloudEnabled,
  isDoodleCloudEnabled`.
- World state (20): `isDanceActive, isNightActive, isSunsetActive, isEruptionActive,
  isEarthquakeActive†, isEclipseActive, isTRexActive, isSpiderActive, isLocustActive†,
  isApocalypseActive, isDraggingPygmy, isDoodleJumpingPygmy, canTouchPygmy,
  isLaserSharkEnabled, canWarlockUseLightning:, sunState, islandLocationState,
  islandMinXPos, islandMaxXPos, islandCenterXPos`.
- Object getters (44, argument = the asking pygmy): `rainbowLogic:, crackHoleLogic:,
  kongStatueLogic:, moronSnakeLogic:, noPeeLogic:, tileDrainLogic:, iceHoleLogic:,
  iceMonsterLogic:, glacierLogic:†, iglooLogic:, apeWenchLogic:, apeBongoLogic:,
  apeAltarLogic:, goGoCageLogic:, danceDanceExecutionLogic:, bonFireLogic:, tRexLogic:,
  tRexEggLogic:, spiderLogic:, spiderTreeLogic:, outHouseLogic:, zombieGraveLogic:,
  birdLogic:, bananaLogic:, fishLogic:, swordFishLogic:, fishingPoleLogic:,
  waterWingsLogic:, sharkLogic:, boulderLogic:, templeDoorLogic:,
  apocalypseTempleDoorLogic:, apocalypsePortalTempleDoorLogic:,
  apocalypseTimeTemplePlatformLogic:, locustQueenLogic:, locustLogic:,
  apocalypsePlagueTempleFountainLogic:, apocalypsePurificationTempleFountainLogic:,
  closetTempleThetanMeterLogic:, closetTempleCouchLogic:, closetTempleClosetDoorLogic:,
  xenuTempleLogic:, xenuLogic:, tomPygmyLogic:`.
- Array getters (15): `timeTempleDoorLogicArray:, locustTempleDoorLogicArray:,
  danceTempleDoorLogicArray:, plagueTempleDoorLogicArray:,
  purificationTempleDoorLogicArray:, closetTempleDoorLogicArray:,
  xenuTempleDoorLogicArray:, idolPedestalLogicArray:, monkeyLogicArray:,
  jellyfishLogicArray:, cloudLogicArray:, doodleCloudLogicArray:, coconutLogicArray:,
  woodLogicArray:, pygmyLogicArray:`.
- Display markers (7): `pygmyDragDisplayMarker, pygmyDisplayMarker,
  pygmyIslandDisplayMarker, pygmySmallObjectsDisplayMarker,
  pygmyUnderwaterPygmyDisplayMarker, pygmyIslandBackDisplayMarker,
  pygmyCloudBehindDisplayMarker`.

Non-trivial controller answers: `isDraggingPygmy` (a pygmy or ghost is dragged),
`canTouchPygmy @ 0x100163c34` (§5), `isGravityEnabled @ 0x1000871d8` (§4.3),
`sunState @ 0x100163e88` (0 at night, else `mSunState`: 1 burn, 2 freeze),
`canWarlockUseLightning: @ 0x100087184` (weather state 1 and L not 11..17),
`isInsideApocalypseTemple @ 0x1000871bc` (L 11..17), `isTomPygmyAlive @ 0x10015b9a8`
(`mTomPygmyLogic != nil`), `tomPygmyLogic:` (`mTomPygmyLogic`).

### 23.2 Events (25)

| Callback | Sent by | Controller reaction |
|---|---|---|
| `onPygmySleepingStart:` | `startSleeping:::` | first sleeper → controller sequence `AllPygmiesSleepingStart`; count++ |
| `onPygmySleepingStop:` | `stopSleeping` | count--; last → `AllPygmiesSleepingStop` |
| `onPygmyDrag:` | `startDrag` | `mDraggedPygmyLogic = p` |
| `onPygmyDragComplete:` | release, `die:::`, `hitBySnowBall:`, `startBaitMaster`, the temple drop and underwater transitions | `mDraggedPygmyLogic = nil`, `checkCountSharkSnaps` |
| `onPygmyGhostDrag:` / `onPygmyGhostDragComplete:` | ghost touch begin / end | `mDraggedGhostPygmyLogic = p / nil` |
| `onPygmyTumble:` | `tumble:::` | moves p to index 0 of `mPygmyLogicArray` |
| `onPygmyTumbleComplete:` | `die:::` | nothing |
| `onPygmyExplode:` | `explode:::` | every pygmy `throwFromExplosion:&p.pos` |
| `onPygmySacrificeEvent:` | `die:::`, `triggerActiveSacrifice` | `[IdolManager triggerEpisodeEventBySacrificeId:]` |
| `onTriggerSacrifice:sacrificeId:` | `triggerActiveSacrifice`, `fatalityInBattle`, via `onPygmyDie:` | tribe statistics, leaderboard (§8.1) |
| `onPygmyDie:sacrificeId:` | `die:::` | §8.1 step 3 |
| `onPygmyGiftGiven:giftId:` | the gift sites (§8.3) | §8.3 |
| `onPygmyStartDoodleJump:` | `checkDoodleCloudCollision:` | `mbDoodleJumpFall = 0`, remember the pygmy, create the doodle clouds |
| `onPygmyStopDoodleJump:` | `fall:::`, doodle-end sites | hide the doodle group, clear, remove the clouds, `[ChallengeManager stopGame:4]` |
| `onPygmyDoodleJumpFall:` | `doodleKnockedDown:`, `doodleCheckOffScreen:` | `mbDoodleJumpFall = 1` |
| `onPygmyFallIntoCrackHoleComplete:` | `onFallIntoCrackHoleComplete:::` | option `TheRunsState == 1`, L not 2/9, no doodle pygmy → `cancelAllTouches`, `[crackHole startTheRuns:p]` |
| `onPygmyHitByLightningInHurricane:` | `hitByLightning`, `hitByLightningByWarlock` | ChallengeManager `incrementGameScore:1` |
| `hitPygmyWithWarlockLightning:` | `animWarlockStrike` | unless a bolt is already drawn: a 7-point `LightningModel` from (x, 300) down to the pygmy (points 1..5 jittered by `(int)(r % 1001 - 500) / 10`), z -277.12, with a `LightningLogic` |
| `onPygmyEarthquakeStart:` | `checkForEarthquake:` | first one (and no earthquake sequence): `episode Earthquake`, stop the celebration music, the first coconut falls off its tree, controller sequence `EarthquakeStart`; count++ |
| `onPygmyEarthquakeStop:` | `checkForEarthquake:`, `hitByLightning`, `die:::` | count--; last → `EarthquakeStop` |
| `onVolcanoHit:` | `tumble:::` | hit count 0..2: the volcano plays `erupt1/2/3` (fast pygmy) or `hit1/2/3`; the third hit starts the eruption |
| `onKonkeyDongScore:` | `checkKonkeyDongCollision:` | ChallengeManager `incrementGameScore:12 score:n` |
| `onKonkeyDongStop`† | - | (declared; the controller implements `onKonkeyDongStop:` for `KongStatueLogic`) |
| `startApocalypse` | `ejectFromTomsArms:`, `stopHoldingBabyPygmy` (Tom, interrupted sacrifice) | starts the apocalypse |

### 23.3 Other controller entry points into the pygmy

From `PocketGodViewController` and `(Game)`: `addedUnderwater`, `startUnderwater`,
`waitUnderwater`, `stopUnderwater`, `startBaitMaster`, `stopBaitMaster`,
`canBeTouchedByUnderwaterPower` / `canBeTouchedByUnderwaterPower:` (underwater powers),
`dropIntoApocalypseTempleRoom`, `moveTo/FromApocalypseTempleDisplayGroup`,
`setTimeTemplePygmyType:`, `timeTemplePygmyType`, `canDoNormalDance`, `startDancing:`,
`startVampireDancing`, `stopDancingNow`, `hitByLightning` (storm), `throwFromExplosion:`,
`isZombie`, `isVampire`, `isKilledByPlague`, `doodleKnockedDown:`,
`doodleCheckOffScreen:`, `noPeeSwimmingLeft`, `updateLabelLayer`, `setPygmyNameLabel:`,
`pygmyNameLabel`, `personaPersonality`, `removeFromStoryMode`, `setStoryMode`,
`startStoryEarthquake`, `stopStoryEarthquake`, `trappedInCloset`, and on location
changes `onDeactivateIslandLocationState:newIslandLocationState:` (§3.7).

## 24. Interactions with the other logic classes

Found by scanning every decompiled class for message sends of PygmyLogic selectors
(receiver not `self`), and every send in the pygmy files whose selector is implemented by
exactly one other class. Sends through selector variables (`op = "…"; msg(x, op)`) are not
always caught: `HurricaneLogic` also sends `droppedByHurricane:` and `unblownByHurricane:`,
and `WhirlPoolLogic` uses the `SpriteLogic` whirlpool overrides; `getLogic(n)` objects are
listed under their class. The behaviour behind each call is in the feature sections.

| class | calls into PygmyLogic | PygmyLogic calls on it |
|---|---|---|
| AnchorLogic | `stopBaitMaster` | - |
| AntHillLogic | `canBeSwarmedByAntHill:`, `swarmedByAntHill:` | - |
| ApeAltarLogic | `grabbedFromApeAltar`, `tornApartFromApeAltar`, `tossFromApeAltar` | `pygmyFallIntoApeAltar:`, `removePygmyFromApeAltar:` |
| ApeBongoLogic | `apeSmashOnApeBongo`, `tossFromApeBongo` | `pygmyFallIntoApeBongo:`, `removePygmyFromApeBongo:` |
| ApeWenchLogic | `apeSmashOnApeWench`, `tossFromApeWench` | `pygmyFallIntoApeWench:`, `removePygmyFromApeWench:` |
| ApocalypseCloudLogic | `isPlagueBodyPickedUp`, `plagueBodyCarryingPygmyLogic` | - |
| ApocalypseTempleDoorLogic | `ejectFromApocalypseTempleDoor` | `ejectFromApocalypseTempleDoor:`, `fallIntoApocalypseTempleDoor:` |
| ApocalypseTempleFountainLogic | `ejectFromApocalypseTempleFountain` | `ejectFromApocalypseTempleFountain:`, `isSlot1Available`, `isSlot2Available`, `isSlot3Available` |
| ApocalypseTimeTemplePlatformLogic | `ejectFromApocalypseTimeTemplePlatform`, `setTimeTemplePygmyType:`, `timeTemplePygmyType` | `ejectFromApocalypseTimeTemplePlatform:`, `fallOntoApocalypseTimeTemplePlatform:` |
| BananaLogic | `hitByBanana:`, `stopEating` | `isSlippable`, `slipOnBanana:` |
| BatLogic | `bitByBat:`, `canBeBitByBat` | - |
| BattleEnergyBallLogic | - | `activateWithStartPos:targetPygmyLogic:`, `setLastBall:`, `setNoDamage:` |
| BirdLogic | `canBeCaughtByBird:`, `caughtByBird:`, `caughtByBirdComplete:`, `droppedByBird:`, `hitByCookedBird:` | `dropPygmyLogic:` |
| BirdPooLogic | `hitByBirdPoo:` | - |
| BonFireLogic | `hitByBonFire:` | `isLeftWarmingAvailable:`, `isOnFire`, `isRightWarmingAvailable:`, `stopLeftWarming:`, `stopRightWarming:` |
| BoulderLogic | `charlieSmackBoulder:`, `crushedByBoulder:`, `hitByBoulder:xVel:`, `personaPersonality`, `uncrushedByBoulder:dontWait:` | - |
| ChainSpearLogic | `canBeHitByChainSpear:`, `hitByChainSpear:`, `hitByChainSpearComplete:`, `hitByChainSpearRetract:` | - |
| ChumLogic | `canHoldChum:`, `dropChum:`, `holdChum:` | `dropChum:` |
| ClosetTempleClosetDoorLogic | `canSlamClosetDoor`, `ejectFromClosetTempleClosetDoor`, `personaPersonality`, `startInClosetTempleClosetDoor` | `closetYPos`, `ejectFromClosetTempleClosetDoor:`, `fallIntoClosetTempleClosetDoor:`, `grabNewTom`, `maxClosetXPos`, `minClosetXPos`, `slamDoor` |
| ClosetTempleCouchLogic | `ejectFromClosetTempleCouch`, `personaPersonality` | `ejectFromClosetTempleCouch:`, `fallOntoClosetTempleCouch:`, `jumpOntoClosetTempleCouch:` |
| ClosetTempleThetanMeterLogic | `ejectFromClosetTempleThetanMeter`, `explodeFromThetanBirth` | `ejectFromClosetTempleThetanMeter:`, `fallOntoClosetTempleThetanMeter:` |
| CloudLogic | - | `isDoodleCloud` |
| CoconutLogic | `hitByCoconut:`, `stopEating` | - |
| CrackHoleLogic | `canFallIntoCrackHole:`, `fallIntoCrackHole:`, `startTheRuns` | `stopTheRuns`, `theRunsPlatformLogicArray`, `theRunsRockLogic`, `theRunsStalactiteLogicArray`, `theRunsTurdLogicArray` |
| DanceBalloonLogic | `inhaleDanceBalloon:` | - |
| DanceDanceExecutionLogic | `danceDanceExecutedComplete:`, `tossFromDanceDanceExecution` | `pygmyFallIntoDanceDanceExecution:`, `removePygmyFromDanceDanceExecution:` |
| DreamBubbleLogic | `dreamBubbleSelected` | - |
| FishLogic | `hitByFish:` | - |
| FishingBobLogic | `catchFish`, `catchShark`, `pullPole` | `initBobTexture` |
| FishingPoleLogic | `stopFishingFromPole:` | `fishingBobLogic`, `fishingStringDisplayObject`, `pickupPole`, `startIceFishing:` |
| GoGoCageLogic | `tossFromGoGoCage` | `pygmyFallIntoGoGoCage:`, `removePygmyFromGoGoCage:` |
| HurricaneLogic | `blownByHurricane:`, `pickedUpByHurricane:` | - |
| IceHoleLogic | `canBeGrabbedByTentacle:`, `canMeltIceHole`, `grabbedByTentacle:`, `iceHoleSplashComplete`, `meltIceHoleComplete:`, `stopFishingFromIceHole:` | `beheadPygmy:`, `burntPygmyFallIntoIceHole:`, `isAvailableForFalling`, `isAvailableForFishing`, `isFrozen`, `shouldPygmyWalkTowardIceHole:`, `startFishingSitDown:`, `stopFishing:`, `tentaclePos` |
| IceMonsterLogic | - | `hitByPygmy:onFire:`, `hitByPygmyInMouth:`, `isDormant`, `isMouthCollisionActive`, `isRecharging`, `screenCollisionRadius`, `screenCollisionYPos`, `screenMouthCollisionRadius`, `screenMouthCollisionYPos` |
| IcicleLogic | `hitByIcicle:` | - |
| IdolManager | `pygmyNameLabel` | - |
| IdolPedestalLogic | `ejectFromIdolPedestal` | `ejectFromIdolPedestal:`, `fallIntoIdolPedestal:` |
| IglooLogic | `leaveIglooDoorCrawl`, `leaveIglooDoorExplode`, `leaveIglooTopExplode`, `tossFromIgloo` | `isPygmyCrawlAvailable:`, `isPygmyFallAvailable:`, `pygmyCrawlIntoIgloo:`, `pygmyFallIntoIgloo:`, `removePygmyFromIgloo:` |
| JellyfishLogic | `canBeHitByJellyfish:`, `jellyfishAttackComplete:`, `stungByJellyfish:`, `suffocatedByJellyfish:` | `canInteractWithPygmy`, `cancelJellyfishAttack:`, `circleJerkPos`, `isCircleJerkLeader`, `onJellyfishAttackComplete:`, `onJellyfishSuffocateAttackComplete:` |
| KongStatueLogic | `canChaseKonkeyDong`, `chaseKonkeyDong`, `jumpToNextKonkeyDongLevel`, `startKonkeyDong`, `startStaringAtKidnapping`, `stopStaringAtKidnapping`, `tossFromKongStatue` | `konkeyDongBeetleLogicArray`, `konkeyDongBoulderLogicArray`, `konkeyDongLadderLogicArray`, `konkeyDongLevel`, `konkeyDongLevelDisplayObject`, `konkeyDongLogic`, `konkeyDongPickLogicArray`, `konkeyDongPlatformLogicArray`, `pygmyFallIntoKongStatue:`, `removePygmyFromKongStatue:`, `showBonusSplash:`, `startKonkeyDong:`, `stopKonkeyDong` |
| KonkeyDongBeetleLogic | - | `isRolledUp`, `jumpOnBeetle:`, `smashBeetle` |
| KonkeyDongBoulderLogic | - | `crackBoulder` |
| KonkeyDongLadderLogic | - | `checkBottomCollision:lastPos:`, `checkTopCollision:lastPos:`, `ladderHeight` |
| KonkeyDongLogic | `kidnappedByKonkeyDong`, `kidnappedByKonkeyDongComplete` | - |
| KonkeyDongPlatformLogic | - | `isFinalPlatform`, `isStartPlatform` |
| LaserSharkLogic | `canBeHitBySharkLaser`, `hitBySharkLaser`, `startEatenByLaserShark:`, `stopEatenByLaserShark:`, `swallowedByLaserShark:` | - |
| LavaLogic | `hitByLava:` | - |
| LocustLogic | `catchInAirByLocusts:`, `cutInAirByLocusts:`, `droppedInAirByLocusts:`, `startThrownInAirByLocusts:`, `thrownInAirByLocusts:targetPos:targetTime:` | `shouldPygmyPanic`, `stopBeingThrownByLocusts:` |
| LocustQueenLogic | `eatenByLocustQueenComplete:`, `rescuedFromLocustQueen` | `fallIntoLocustQueenArms:`, `rescuedFromLocustQueen:` |
| MagnifyingGlassLogic | `burntByMagnifyingGlass:`, `canBeBurntByMagnifyingGlass:`, `unburntByMagnifyingGlass:` | `onPygmyDie:` |
| MonkeyLogic | `canBeCaughtByMonkey:`, `canBeChasedByMonkey:`, `canBeHumpedByMonkey:`, `caughtByMonkey:`, `humpedByMonkey:`, `monkeyBootedOffCharliesBack:`, `monkeyOnBack:`, `personaPersonality`, `playingCatchWithMonkey`, `thrownByMonkey:toMonkey:`, `unhumpedByMonkey:` | `bootedOffCharliesBack:`, `canChasePygmy`, `punchedWhileHumping`, `pygmyIsEatingBanana:`, `startPlayingCatch`, `stopHumpingPygmy`, `stopPlayingCatch` |
| MoronSnakeLogic | `eatenByMoronSnake:`, `startMoronPests`, `timeTemplePygmyType` | `eatPygmyComplete:`, `stopMoronPests`, `thrownByBaby:` |
| MoronTurtleLogic | `eatenByMoronTurtle:` | - |
| NoPeeClamLogic | `hitByNoPeeClam:` | - |
| NoPeeLogic | `startNoPee` | `onPygmyKilled`, `onPygmyStartSwimming`, `stopNoPee` |
| NoPeeMapLogic | `canBeHitByNoPeeObject`, `noPeeRunOutOfBreath` | - |
| NoPeeMineLogic | `hitByNoPeeMine:` | - |
| NoPeeObjectLogic | `canBeHitByNoPeeObject` | - |
| NoPeeSeaBlobLogic | `canBeHitByNoPeeObject`, `hitByNoPeeSeaBlob:` | - |
| OctoStatueLogic | `explodeFromLovingPocketGod` | - |
| OutHouseLogic | `finishCrapping`, `tossFromOutHouse` | `isCrappingAvailable:`, `isWaitingAvailable:`, `stopCrapping:`, `stopWaitingToCrap:`, `takeCrap:`, `waitToCrap:` |
| PiranhaLogic | `canBeEatenByPiranha:`, `eatenByPiranha:`, `eatenByPiranhaComplete:`, `eatingPiranhaLogic`, `stopEatenByPiranha:` | `stopEatingPygmy:` |
| PocketGodViewController | `doodleKnockedDown:`, `personaPersonality`, `pygmyNameLabel`, `removeFromStoryMode`, `setPygmyNameLabel:`, `setStoryMode`, `startStoryEarthquake`, `stopDancingNow`, `stopStoryEarthquake`, `stopUnderwater`, `timeTemplePygmyType`, `trappedInCloset`, `waitUnderwater` | `apeAltarLogic:`, `apeBongoLogic:`, `apeWenchLogic:`, `apocalypsePortalTempleDoorLogic:`, `apocalypseTempleDoorLogic:`, `apocalypseTimeTemplePlatformLogic:`, `areGhostsEnabled`, `areNameLabelsEnabled`, `bananaLogic:`, `birdLogic:`, `bonFireLogic:`, `canTouchPygmy`, `canWarlockUseLightning:`, `closetTempleClosetDoorLogic:`, `closetTempleCouchLogic:`, `closetTempleDoorLogicArray:`, `closetTempleThetanMeterLogic:`, `cloudLogicArray:`, `coconutLogicArray:`, `danceDanceExecutionLogic:`, `danceTempleDoorLogicArray:`, `doodleCloudLogicArray:`, `fishLogic:`, `fishingPoleLogic:`, `goGoCageLogic:`, `hitPygmyWithWarlockLightning:`, `iceHoleLogic:`, `iceMonsterLogic:`, `idolPedestalLogicArray:`, `iglooLogic:`, `isApeAltarEnabled`, `isApeBongoEnabled`, `isApeWenchEnabled`, `isApocalypsePlagueTempleFountainEnabled`, `isApocalypsePortalTempleDoorEnabled`, `isApocalypsePurificationTempleFountainEnabled`, `isApocalypseTempleDoorEnabled`, `isApocalypseTimeTemplePlatformEnabled`, `isBonFireEnabled`, `isClosetTempleClosetDoorEnabled`, `isClosetTempleCouchEnabled`, `isClosetTempleDoorEnabled`, `isClosetTempleThetanMeterEnabled`, `isCoconutEnabled`, `isDanceDanceExecutionEnabled`, `isDanceTempleDoorEnabled`, `isDoodleCloudEnabled`, `isDoodleJumpingPygmy`, `isDraggingPygmy`, `isEclipseActive`, `isEruptionActive`, `isGoGoCageEnabled`, `isGravityEnabled`, `isIceHoleEnabled`, `isIceMonsterEnabled`, `isIdolPedestalEnabled`, `isIglooEnabled`, `isInsideApocalypseTemple`, `isJellyfishEnabled`, `isKongStatueEnabled`, `isLocustQueenEnabled`, `isLocustTempleDoorEnabled`, `isMoronPestsEnabled`, `isNightActive`, `isOutHouseEnabled`, `isPlagueTempleDoorEnabled`, `isPurificationTempleDoorEnabled`, `isSpiderActive`, `isSpiderTreeEnabled`, `isStormCloudEnabled`, `isSunsetActive`, `isTRexActive`, `isTempleDoorEnabled`, `isTimeTempleDoorEnabled`, `isTomPygmyAlive`, `isVolcanoEnabled`, `isWoodEnabled`, `isXenuEnabled`, `isXenuTempleDoorEnabled`, `isXenuTempleEnabled`, `isZombieGraveEnabled`, `islandCenterXPos`, `islandLocationState`, `jellyfishLogicArray:`, `kongStatueLogic:`, `locustQueenLogic:`, `locustTempleDoorLogicArray:`, `monkeyLogicArray:`, `moronSnakeLogic:`, `onKonkeyDongScore:`, `onPygmyDie:sacrificeId:`, `onPygmyDragComplete:`, `onPygmyEarthquakeStart:`, `onPygmyEarthquakeStop:`, `onPygmyExplode:`, `onPygmyGhostDrag:`, `onPygmyGiftGiven:giftId:`, `onPygmySacrificeEvent:`, `onPygmyStartDoodleJump:`, `onPygmyStopDoodleJump:`, `onTriggerSacrifice:sacrificeId:`, `outHouseLogic:`, `plagueTempleDoorLogicArray:`, `purificationTempleDoorLogicArray:`, `pygmyCloudBehindDisplayMarker`, `pygmyDisplayMarker`, `pygmyDragDisplayMarker`, `pygmyIslandBackDisplayMarker`, `pygmyIslandDisplayMarker`, `rainbowLogic:`, `sharkLogic:`, `spiderLogic:`, `spiderTreeLogic:`, `startApocalypse`, `sunState`, `swordFishLogic:`, `tRexLogic:`, `templeDoorLogic:`, `timeTempleDoorLogicArray:`, `tomPygmyLogic:`, `woodLogicArray:`, `xenuLogic:`, `xenuTempleDoorLogicArray:`, `xenuTempleLogic:`, `zombieGraveLogic:` |
| PocketGodViewController+Game | `addedUnderwater`, `canBeTouchedByUnderwaterPower`, `canBeTouchedByUnderwaterPower:`, `canDoNormalDance`, `doodleCheckOffScreen:`, `doodleKnockedDown:`, `dropIntoApocalypseTempleRoom`, `hitByLightning`, `isKilledByPlague`, `isVampire`, `isZombie`, `moveFromApocalypseTempleDisplayGroup`, `moveToApocalypseTempleDisplayGroup`, `noPeeSwimmingLeft`, `setTimeTemplePygmyType:`, `startBaitMaster`, `startDancing:`, `startUnderwater`, `startVampireDancing`, `stopBaitMaster`, `stopDancingNow`, `throwFromExplosion:`, `timeTemplePygmyType`, `updateLabelLayer` | - |
| PygmyBottomLogic | - | `spawnLocustBone:`, `spawnLocustSkull:`, `tossBottom:` |
| PygmyPersona | - | `personaPersonality` |
| RainbowLogic | `gushOverRainbow`, `isStaringAtRainbow`, `startStaringAtRainbow`, `stopStaringAtRainbow` | `gushOverRainbowComplete:` |
| SharkLogic | `caughtByShark:`, `eatenByShark:`, `hitByShark:xVel:`, `timeTemplePygmyType`, `uneatenByShark:` | `cantCatchPygmy:`, `checkJumpForPygmy:`, `launchOffCharlie`, `letGoOfPygmy`, `swallowPygmy` |
| SnowBallLogic | `hitBySnowBall:` | - |
| SpearLogic | `canCatchSpear`, `catchSpear:` | `dropWhileHolding`, `throwSpear` |
| SpiderLogic | `eatenBySpider:`, `swallowedBySpider:`, `uneatenBySpider:` | - |
| SpiderTreeLogic | `eatenByLittleSpiders:`, `ejectFromSpiderTree` | `ejectFromWeb:`, `trapInWeb:` |
| SpriteLogic | - | `infectWithDisease:` |
| StoryManager | `isReadyForStoryMode`, `resumeFromStoryModeCharacterState`, `setStoryModeCharacterState:` | - |
| SwordFishLogic | `hitBySwordFish:` | - |
| TRexEggLogic | `charlieSmackTRexEgg:`, `crushedByTRexEgg:`, `hitByTRexEgg:xVel:`, `personaPersonality`, `uncrushedByTRexEgg:dontWait:` | - |
| TRexLogic | `eatenByTRex:`, `swallowedByTRex:`, `timeTemplePygmyType`, `uneatenByTRex:` | - |
| TempleDoorLogic | `ejectFromTempleDoor`, `pygmyPersona`, `removeFromBattleOfTheGods`, `setPygmyNameLabel:`, `spawnFromBattleOfTheGods`, `startBattleOfTheGods:`, `startBattleOfTheGodsEnemy:`, `updateLabelLayer`, `wasKilledDuringBattle` | `ejectFromTempleDoor:`, `fallIntoTempleDoor:`, `jumpIntoTempleDoor:`, `onPygmyBattleReady:` |
| TheRunsPlatformLogic | `theRunsBurnPygmyComplete` | - |
| ThetanLogic | `canBePossessedByThetan`, `possessedByThetan:`, `startPossessedByThetan:`, `thetanStopPossessingPygmy:` | `stopPossessingPygmy:` |
| TileDrainLogic | `dieFromPainDrain`, `escapeFromDrain:`, `startPainDrain`, `suckedIntoDrain:` | `escapeFromDrain:`, `painDrainPlatformLogicArray`, `pygmyFallIntoDrainComplete:`, `stopPainDrain` |
| TribePersona | `pygmyPersona`, `setPygmyPersona:` | - |
| WaterWingsLogic | `ejectFromWaterWings`, `wearWaterWings:` | - |
| WhirlPoolLogic | `droppedByWhirlPool:`, `pickedUpByWhirlPool:` | - |
| WoodLogic | `stopCarryingFromWood` | `dropWhileCarrying`, `pickUpForCarrying`, `startCarrying:`, `throwOnPile` |
| XenuLogic | `ejectFromXenu`, `isCutInHalf` | `ejectFromXenu:`, `isSlot1Available:`, `isSlot2Available:`, `spawn` |
| XenuTempleLogic | - | `tearDownWall` |
| ZombieGraveLogic | `ejectFromZombieGrave`, `startZombieIdle`, `zombieExplode` | `ejectFromGrave:`, `fallIntoGrave:`, `isOpenAndUnoccupied`, `loseBalanceByGrave:` |

Selectors whose receiver class is ambiguous (implemented by several classes):

- ApocalypseTempleFountainLogic / SpriteLogic: `apocalypseDiseaseType`
- BCDisplayObject / BCModel / BCMultiModel: `setTextureDef:`
- BCDisplayObject / BCTouch: `pos`, `setXPos:`, `setYPos:`, `xPos`, `yPos`
- BirdLogic / FishLogic: `isBurnt`, `isCooked`
- BirdLogic / PocketGodViewController: `isOceanEnabled`
- CBConfig / CLSUserDefaults / FlurryKeychainWrapper: `setObject:forKey:`
- CLSUserDefaults / FlurryKeychainWrapper: `objectForKey:`, `removeObjectForKey:`
- ClosetTempleThetanMeterLogic / PocketGodViewController: `isApocalypseActive`
- FishingBobLogic / FishingPoleLogic: `dropWhileFishing`, `startFishingIdle`
- FishingPoleLogic / IceHoleLogic: `startFishing:`
- HurricaneLogic / WhirlPoolLogic: `dropSpriteLogic:`
- KongStatueLogic / MoronSnakeLogic: `pygmyIntroComplete`
- MoronSnakeLogic / MoronTurtleLogic: `startMoronPests:`
- PocketGodViewController / RainbowLogic: `isRainbowActive`
- many: `activate:`, `checkCollision:lastPos:`, `clearPygmyLogic:`, `deactivate`, `die`, `dropWhileEating`, `endLevel`, `finishEating`, `isActive`, `isAvailable`, `isEdible`, `islandMaxXPos`, `islandMinXPos`, `pickUpForEating`, `play`, `pygmyLogicArray:`, `startEating:`, `stop`, `thrownByCharlie:`

## 25. Guards

Every busy test as a difference from FREE (§1.3). "all must hold": the method proceeds only
when no listed flag is set and the other conditions hold; "blocked if any": it returns early
when one is set (same set semantics). `n` = number of terms in the source condition (a flag
can be tested twice). Extracted mechanically from
the decompilation (`/*?lVarN*/` = the offset was resolved through a reused register).

| method @ address | mode | FREE minus | plus (also must be clear) | must be set | other conditions |
|---|---|---|---|---|---|
| `PygmyLogic(Apocalypse)::dropIntoApocalypseTempleRoom` @ 0x10003bd74 | all must hold (52) | mbTouchBeganInside mbCutInHalfByCharlie T!=0 mbBlownByHurricane mbPickedUpByHurricane mbDiedFromThetanBrawl mbCrapping mbTrappedInZombieGrave mbTrappedInIdolPedestal mbInsideTempleDoor mbInsideApocalypseTempleDoor mbOnTopApocalypseTimeTemplePlatform mbInApocalypseTempleFountain mbThetanBrawl mbOnClosetTempleThetanMeter mbOnClosetTempleCouch mbInClosetTempleClosetDoor mbHeldByXenu mbHeldInTomsArms mbHoldingBabyPygmy mbThrownByLocusts mbBattleOfTheGods mbPossessedByThetan mbCaughtByMonkey mbMonkeyOnBack mbEatenByMoronSnake mbWearingWaterWings mbSkewered mbDragging mbCrushed mbFalling mbDoodleJumping mbKilledByPlague mbDrowning | mbBeheadedBySquid mbSharkOnHead | - | ANY{!mbDrowning, mbKilledByPlague} |
| `PygmyLogic(Apocalypse)::canBePossessedByThetan` @ 0x10003e3ec | all must hold (86) | - | mbSharkOnHead mbSleeping | - | mTiltState == 0 |
| `PygmyLogic(Apocalypse)::canDoThetanBrawl:` @ 0x10003edb0 | all must hold (86) | mbPossessedByThetan | mbSharkOnHead mbSleeping mbWaitingForPossessionByThetan | - | mTiltState == 0 |
| `PygmyLogic(Apocalypse)::isBabyHoldingAvailable` @ 0x100040dfc | all must hold (76) | mbBlownByHurricane mbPickedUpByHurricane mbInKongStatue mbInApeWench mbInApeBongo mbInApeAltar mbInGoGoCage mbInDanceDanceExecution mbIsZombie | - | - | mPersonaPersonality == 2; mBabyPygmyLogic == 0 |
| `PygmyLogic(Apocalypse)::startThrownInAirByLocusts:` @ 0x10004203c | all must hold (88) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Apocalypse)::cutInAirByLocusts:` @ 0x100042ff4 | all must hold (87) | mbFalling | mbSharkOnHead | mbFalling | mTiltState == 0 |
| `PygmyLogic(Apocalypse)::stripFleshByLocusts:` @ 0x100043814 | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Apocalypse)::mouthFilledByLocusts:` @ 0x100044200 | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Apocalypse)::mutatedByLocusts:` @ 0x100044a50 | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Apocalypse)::inhaleDanceBalloon:` @ 0x1000452b4 | all must hold (85) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Apocalypse)::canBeInfected:` @ 0x100045b54 | all must hold (84) | T!=0 | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Charlie)::hitByLightningByWarlock` @ 0x1000581d0 | blocked if any (75) | mbBlownByHurricane mbPickedUpByHurricane mbInKongStatue mbInApeWench mbInApeBongo mbInApeAltar mbInGoGoCage mbInDanceDanceExecution mbIsZombie | - | - | - |
| `PygmyLogic(Charlie)::canBeCutInHalfByCharlie:` @ 0x100058b80 | all must hold (87) | - | mbBitByBat mbSharkOnHead mbSleeping | - | mTiltState == 0 |
| `PygmyLogic(Charlie)::canBeBurntByWarlock:` @ 0x100059d20 | all must hold (86) | - | mbSharkOnHead | mbBitByBat | mTiltState == 0 |
| `PygmyLogic(Charlie)::canBeAbsorbedByHalfPygmy:` @ 0x10005a534 | all must hold (85) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Charlie)::charlieSmackBoulder:` @ 0x10005b164 | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Charlie)::charlieSmackTRexEgg:` @ 0x10005ba0c | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::throwFromExplosion:` @ 0x10009731c | all must hold (85) | - | mbWarlockSkipNextExplosion | - | - |
| `PygmyLogic(Island)::hitByLightning` @ 0x100097c2c | blocked if any (75) | mbBlownByHurricane mbPickedUpByHurricane mbInKongStatue mbInApeWench mbInApeBongo mbInApeAltar mbInGoGoCage mbInDanceDanceExecution mbIsZombie | - | - | - |
| `PygmyLogic(Island)::canBeHitByLightning` @ 0x100098630 | blocked if any (74) | mbBlownByHurricane mbPickedUpByHurricane mbInKongStatue mbInApeWench mbInApeBongo mbInApeAltar mbInGoGoCage mbInDanceDanceExecution mbIsZombie | - | - | - |
| `PygmyLogic(Island)::explodeFromLovingPocketGod` @ 0x100098c30 | all must hold (82) | mbBlownByHurricane mbPickedUpByHurricane | - | - | - |
| `PygmyLogic(Island)::hitByLava:` @ 0x1000992dc | all must hold (84) | - | - | - | - |
| `PygmyLogic(Island)::hitByIcicle:` @ 0x100099aec | all must hold (84) | - | - | - | - |
| `PygmyLogic(Island)::hitByBonFire:` @ 0x10009a328 | all must hold (84) | - | - | - | - |
| `PygmyLogic(Island)::hitByCoconut:` @ 0x10009ab4c | all must hold (85) | T!=0 | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::hitByBirdPoo:` @ 0x10009b3b8 | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::hitBySnowBall:` @ 0x10009bc0c | all must hold (84) | mbTouchBeganInside mbDragging | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::hitByFish:` @ 0x10009c55c | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::hitBySwordFish:` @ 0x10009cdb0 | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::hitByBanana:` @ 0x10009d604 | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::hitByCookedBird:` @ 0x10009de58 | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::hitByCookedPygmy:` @ 0x10009e6ac | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::hitByBoulder:xVel:` @ 0x10009ef00 | all must hold (86) | - | - | - | mTiltState == 0 |
| `PygmyLogic(Island)::canBeHitByPygmyIglooRoll:` @ 0x10009f764 | all must hold (84) | - | - | - | mTiltState == 0 |
| `PygmyLogic(Island)::hitByTRexEgg:xVel:` @ 0x10009ffcc | all must hold (86) | - | - | - | mTiltState == 0 |
| `PygmyLogic(Island)::hitByShark:xVel:` @ 0x1000a0830 | all must hold (86) | - | - | - | mTiltState == 0 |
| `PygmyLogic(Island)::canBeHitByVampire` @ 0x1000a1094 | all must hold (84) | T!=0 mbDrowning | mbBitByBat | - | ANY{mPersonaPersonality != 1, mTimeTemplePygmyType != 0}; mTiltState == 0 |
| `PygmyLogic(Island)::crushedByBoulder:` @ 0x1000a1b2c | all must hold (87) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::crushedByTRexEgg:` @ 0x1000a24a4 | all must hold (87) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::eatenByTRex:` @ 0x1000a2e98 | all must hold (88) | T!=0 | mbSharkOnHead | - | mTiltState == 0; (mTimeTemplePygmyType \| 2) == 2 |
| `PygmyLogic(Island)::eatenBySpider:` @ 0x1000a3898 | all must hold (87) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::canBeBurntByMagnifyingGlass:` @ 0x1000a438c | all must hold (85) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::burntByMagnifyingGlass:` @ 0x1000a4a40 | all must hold (87) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::canCatchSpear` @ 0x1000a5348 | all must hold (86) | - | mbSharkOnHead mbThrowingSpear | - | mTiltState == 0 |
| `PygmyLogic(Island)::catchSpear:` @ 0x1000a5a10 | all must hold (88) | - | mbSharkOnHead mbThrowingSpear | - | mTiltState == 0 |
| `PygmyLogic(Island)::canBeSwarmedByAntHill:` @ 0x1000a635c | all must hold (85) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::swarmedByAntHill:` @ 0x1000a6a10 | all must hold (84) | mbEatenByPiranha mbSuckedIntoDrain mbPainDrain | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::canFallIntoCrackHole:` @ 0x1000a723c | all must hold (85) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::startStaringAtKidnapping` @ 0x1000a7b2c | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::canChaseKonkeyDong` @ 0x1000a8424 | all must hold (85) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::blownByHurricane:` @ 0x1000a8c50 | all must hold (86) | mbPickedUpByHurricane | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::pickedUpByHurricane:` @ 0x1000a9518 | all must hold (80) | mbCutInHalfByCharlie T!=0 mbBlownByHurricane mbTrappedInZombieGrave mbTrappedInIdolPedestal mbIsZombie mbCooked mbKilledByPlague | mbSharkOnHead | - | ANY{!mbTrappedInZombieGrave, !msg(mZombieGraveLogic, "isPygmyAboutToBecomeZombie")}; mTiltState == 0 |
| `PygmyLogic(Island)::eatenByShark:` @ 0x1000aa0cc | all must hold (88) | - | mbSharkOnHead mbSleeping | - | mTiltState == 0 |
| `PygmyLogic(Island)::canBeCaughtByBird:` @ 0x1000aa9dc | all must hold (86) | - | mbSharkOnHead mbSleeping | - | mTiltState == 0 |
| `PygmyLogic(Island)::caughtByBird:` @ 0x1000ab0a4 | all must hold (88) | - | mbSharkOnHead mbSleeping | - | mTiltState == 0 |
| `PygmyLogic(Island)::canDoNormalDance` @ 0x1000aba40 | all must hold (84) | T!=0 | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::startDancing:` @ 0x1000ac0e0 | all must hold (84) | T!=0 mbIsZombie | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::startVampireDancing` @ 0x1000aca00 | all must hold (85) | mbIsZombie | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::stopDancing:updateSong:themeSong:napoleonDynamite:fishmas:fishmasTwelveDays:` @ 0x1000ad2b8 | all must hold (84) | T!=0 mbIsZombie | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::startStaringAtRainbow` @ 0x1000adcd4 | all must hold (86) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::monkeyOnBack:` @ 0x1000aec6c | all must hold (90) | - | mbSharkOnHead mbSleeping | - | mCharlieOnBackMonkeyLogic == 0; mTiltState == 0 |
| `PygmyLogic(Island)::canBeCaughtByMonkey:` @ 0x1000af574 | all must hold (86) | mbFalling | mbSharkOnHead | mbFalling | mTiltState == 0; *(float *)(OFF(mPosVel) + this + 4) < 0.0 |
| `PygmyLogic(Island)::canBeChasedByMonkey:` @ 0x1000afc44 | all must hold (85) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::eatenByMoronSnake:` @ 0x1000b07c4 | all must hold (89) | T!=0 | mbSharkOnHead mbSleeping | - | mTiltState == 0; (mTimeTemplePygmyType \| 2) == 2 |
| `PygmyLogic(Island)::wearWaterWings:` @ 0x1000b1210 | all must hold (88) | T!=0 | mbSharkOnHead mbSleeping | - | mTiltState == 0; (mTimeTemplePygmyType \| 2) == 2 |
| `PygmyLogic(Island)::canBeTouchedByGhost:` @ 0x1000b1cac | all must hold (85) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::holdEarsFromBansheeScream:` @ 0x1000b2fe4 | all must hold (84) | - | - | - | - |
| `PygmyLogic(Island)::canHaveBrainsEaten:` @ 0x1000b3914 | all must hold (85) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Island)::canMeltIceHole` @ 0x1000bcb38 | all must hold (85) | - | mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Underwater)::startUnderwater` @ 0x10011da5c | all must hold (58) | mbBlownByHurricane mbCrapping mbTrappedInZombieGrave mbTrappedInIdolPedestal mbInsideTempleDoor mbInsideApocalypseTempleDoor mbOnTopApocalypseTimeTemplePlatform mbInApocalypseTempleFountain mbThetanBrawl mbOnClosetTempleThetanMeter mbOnClosetTempleCouch mbInClosetTempleClosetDoor mbSlammingClosetTempleClosetDoor mbHeldByXenu mbHeldInTomsArms mbHoldingBabyPygmy mbThrownByLocusts mbBattleOfTheGods mbPossessedByThetan mbCaughtByMonkey mbMonkeyOnBack mbEatenByMoronSnake mbWearingWaterWings mbSkewered mbUnderwater mbDragging mbCrushed mbFalling mbDoodleJumping | mbBeheadedBySquid mbSharkOnHead | - | ANY{mPersonaPersonality != 2, !mbInClosetTempleClosetDoor}; mTiltState == 0 |
| `PygmyLogic(Underwater)::suckedIntoDrain:` @ 0x10011ebd8 | all must hold (59) | mbBlownByHurricane mbCrapping mbTrappedInZombieGrave mbTrappedInIdolPedestal mbInsideTempleDoor mbInsideApocalypseTempleDoor mbOnTopApocalypseTimeTemplePlatform mbEatenByLocustQueen mbInApocalypseTempleFountain mbThetanBrawl mbOnClosetTempleThetanMeter mbOnClosetTempleCouch mbInClosetTempleClosetDoor mbSlammingClosetTempleClosetDoor mbHeldByXenu mbHeldInTomsArms mbHoldingBabyPygmy mbThrownByLocusts mbKilledByLocusts mbKilledByPurification mbDanceDanceExecuted mbBattleOfTheGods mbSkewered mbUnderwater mbCrushed mbFalling mbInCrackHole mbDoodleJumping | mbBeheadedBySquid mbPickedUpByWhirlPool mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic(Underwater)::startPainDrain` @ 0x10011f208 | all must hold (56) | mbBlownByHurricane mbSuckedIntoDrain mbCrapping mbTrappedInZombieGrave mbTrappedInIdolPedestal mbInsideTempleDoor mbInsideApocalypseTempleDoor mbOnTopApocalypseTimeTemplePlatform mbEatenByLocustQueen mbInApocalypseTempleFountain mbThetanBrawl mbOnClosetTempleThetanMeter mbOnClosetTempleCouch mbInClosetTempleClosetDoor mbSlammingClosetTempleClosetDoor mbHeldByXenu mbHeldInTomsArms mbHoldingBabyPygmy mbThrownByLocusts mbKilledByLocusts mbKilledByPurification mbDanceDanceExecuted mbBattleOfTheGods mbSkewered mbUnderwater mbDragging mbCrushed mbFalling mbInCrackHole mbDoodleJumping | mbBeheadedBySquid mbSharkOnHead | - | mTiltState == 0 |
| `PygmyLogic::isAccelEnabled` @ 0x10012924c | all must hold (77) | mbTouchBeganInside mbCutInHalfByCharlie T!=0 mbIsZombie mbDragging mbFalling mbCooked mbKilledByPlague mbDrowning | mbSharkOnHead | - | ANY{!mbEarthquake, mbFalling}; msg(mPygmyLogicListener,"isGravityEnabled") |
| `PygmyLogic::checkForEarthquake:` @ 0x10012b1d0 | blocked if any (84) | mbIsZombie | mbSharkOnHead | - | - |
| `PygmyLogic::touchBeganInside:allTouches:` @ 0x10013321c | all must hold (53) | mbTouchBeganInside mbCutInHalfByCharlie T!=0 mbSunFrozen mbInKongStatue mbInApeWench mbInApeBongo mbInApeAltar mbInGoGoCage mbInDanceDanceExecution mbTrappedInSpiderWeb mbTrappedInZombieGrave mbTrappedInIdolPedestal mbInsideTempleDoor mbInsideApocalypseTempleDoor mbOnTopApocalypseTimeTemplePlatform mbEatenByLocustQueen mbInApocalypseTempleFountain mbThetanBrawl mbOnClosetTempleThetanMeter mbOnClosetTempleCouch mbInClosetTempleClosetDoor mbHeldByXenu mbHeldInTomsArms mbHoldingBabyPygmy mbThrownByLocusts mbKilledByLocusts mbKilledByPurification mbDanceDanceExecuted mbBattleOfTheGods mbIsZombie mbPossessedByThetan mbUnderwater mbDragging mbInCrackHole mbCooked mbKilledByPlague mbDrowning | mbBeheadedBySquid mbStoryMode | - | (IVar1 & 1) == 0; msg(mPygmyLogicListener,"canTouchPygmy"); mTiltState == 0; mActiveTouch == 0; ANY{lVar9 = OFF(mbDrowning), mbDrowning == '\0', ALL{!mbCooked, !mbCutInHalfByCharlie, ANY{mbKilledByPlague, mTimeTemplePygmyType == 0}}} |
| `PygmyLogic::touchMovedExited:allTouches:` @ 0x100134b58 | all must hold (73) | mbTouchBeganInside mbCutInHalfByCharlie T!=0 mbDroppedByHurricane mbSunFrozen mbInKongStatue mbInApeWench mbInApeBongo mbInApeAltar mbInGoGoCage mbInDanceDanceExecution mbTrappedInSpiderWeb mbUnderwater mbDragging mbInCrackHole mbTumble mbCooked mbKilledByPlague mbDrowning | mbBeheadedBySquid mbKonkeyDong mbNoPee mbStoryMode mbTheRuns | - | (int)IVar1 != 0; mTiltState == 0 |

## 26. Sequence-item handlers (194)

All `:::` items of the pygmy classes with their parameter names (from the sequence data),
number of uses in `sequences.json`, partition unit and other classes implementing an item
of the same name (the sequence data calls whichever class owns the sequence). Semantics:
main class §9 (plus `die`/`explode` §3.6, `drag` §5.2, `slide`/`hangUpsideDown` §4.4,
selection §7.4), Island §10, Apocalypse §11, Charlie §12, Story §13, Underwater §14, No Pee
§15, battle §16, Konkey Dong §17, The Runs §18, Moron Pests §19, Ooga Jump §20,
`PygmyBottomLogic` §21.

| handler | address | params | uses | unit | other classes with this item |
|---|---|---|---:|---|---|
| `PygmyBottomLogic::die:::` | 0x100113048 | - | 119 | persona_and_bottom | 47 classes |
| `PygmyBottomLogic::fall:::` | 0x100112f88 | ignoreIsland | 134 | persona_and_bottom | 26 classes |
| `PygmyLogic(Apocalypse)::brawlAttackComplete:::` | 0x10003ff14 | - | 2 | apocalypse | - |
| `PygmyLogic(Apocalypse)::checkDiseaseIncubation:::` | 0x100041d60 | - | 10 | apocalypse | - |
| `PygmyLogic(Apocalypse)::checkForFinishClosetIdle:::` | 0x100041c48 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::checkForLocusts:::` | 0x100042c34 | - | 3 | apocalypse | - |
| `PygmyLogic(Apocalypse)::checkForLocustsComplete:::` | 0x100042d98 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::checkForPossessedByThetan:::` | 0x10003ed54 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::checkForTempleDoorJump:::` | 0x10003cf08 | - | 1 | containers_and_catchers | - |
| `PygmyLogic(Apocalypse)::checkMutantLocustFlyOff:::` | 0x100042e44 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::checkSlamDoorBouncingOnCouch:::` | 0x10003e3a4 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::checkTempleOldPygmyFallDownFromSlide:::` | 0x10003cae0 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::checkTomJumpOnCouch:::` | 0x10003df04 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::chooseBrawlAttack:::` | 0x10003fe24 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::closetIdle:::` | 0x100047290 | minTime, maxTime | 3 | apocalypse | - |
| `PygmyLogic(Apocalypse)::closetWalk:::` | 0x100047394 | minTime, maxTime | 3 | apocalypse | - |
| `PygmyLogic(Apocalypse)::jumpIntoTempleDoorComplete:::` | 0x10003d14c | - | 1 | containers_and_catchers | - |
| `PygmyLogic(Apocalypse)::locustThrowWaitForApex:::` | 0x100042c10 | - | 2 | apocalypse | - |
| `PygmyLogic(Apocalypse)::mouthFilledByLocustsComplete:::` | 0x100042fbc | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::mutatedByLocustsComplete:::` | 0x100042e0c | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::plagueBodyFloat:::` | 0x100047828 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::possessedByThetanIdle:::` | 0x1000401dc | minTime, maxTime | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::possessedByThetanWalk:::` | 0x10003ff5c | minTime, maxTime | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::respawnPlagueBody:::` | 0x1000475f8 | - | 2 | apocalypse | - |
| `PygmyLogic(Apocalypse)::runSequenceIfInXenuRoom:::` | 0x10003de70 | id | 2 | apocalypse | - |
| `PygmyLogic(Apocalypse)::runToClosetDoor:::` | 0x100041c90 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::templeOldPygmyFallDown:::` | 0x10003cb44 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::templePygmyIdle:::` | 0x10003ce08 | minTime, maxTime | 14 | apocalypse | - |
| `PygmyLogic(Apocalypse)::templePygmyWalk:::` | 0x10003cb5c | minTime, xPosVel, maxTime | 19 | apocalypse | - |
| `PygmyLogic(Apocalypse)::thetanEatPygmyFace:::` | 0x10003fd0c | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::thetanPummelPygmy:::` | 0x10003fcd8 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::throwPlagueBodyOnPile:::` | 0x100047c74 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::tomJumpOffCouch:::` | 0x10003e238 | - | 2 | apocalypse | - |
| `PygmyLogic(Apocalypse)::waitApexTomJumpOnCouch:::` | 0x10003e114 | - | 3 | apocalypse | - |
| `PygmyLogic(Apocalypse)::waitForMutantLocustFlyOffComplete:::` | 0x100042f20 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::waitLandTomJumpOffCouch:::` | 0x10003e2c4 | - | 2 | apocalypse | - |
| `PygmyLogic(Apocalypse)::waitLandTomJumpOnCouch:::` | 0x10003e138 | - | 1 | apocalypse | - |
| `PygmyLogic(Apocalypse)::walkToXenu:::` | 0x100040c34 | - | 2 | apocalypse | - |
| `PygmyLogic(BattleOfTheGods)::battleCheckNoRunBack:::` | 0x100122dd4 | - | 1 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::battleEnergyFastMoveAboveOpponent:::` | 0x100123240 | - | 1 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::battleEnergyFastMoveTowardOpponent:::` | 0x100123164 | - | 1 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::battleEnergyFastWaitForOpponent:::` | 0x100123360 | - | 0 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::battleFall:::` | 0x1001244c8 | ignoreIsland | 2 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::battleFastRunTowardOpponent:::` | 0x10012306c | - | 1 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::battleRespawn:::` | 0x1001246c0 | - | 0 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::battleRunBackToStart:::` | 0x100122ea4 | - | 1 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::battleShowLavaFlame:::` | 0x100124428 | - | 1 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::battleStrongRunTowardOpponent:::` | 0x100122f90 | - | 1 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::checkForAttackTie:::` | 0x1001243a8 | - | 5 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::checkForEnergyStrongAttackWin:::` | 0x100123458 | - | 1 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::checkForFastAttackWin:::` | 0x100123538 | - | 2 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::checkForStrongAttackWin:::` | 0x100123410 | - | 1 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::onBattleReady:::` | 0x1001246e4 | - | 10 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::pushSequenceBasedOnOpponentBattleType:::` | 0x100122d10 | fastEnergyId, fastSwordId, strongSwordId | 3 | minigames | - |
| `PygmyLogic(BattleOfTheGods)::pushSequenceBasedOnOpponentWeaponType:::` | 0x100122c80 | energyId, swordId | 1 | minigames | - |
| `PygmyLogic(Charlie)::charlieCutPygmyInHalfComplete:::` | 0x10005cabc | - | 1 | charlie_story | - |
| `PygmyLogic(Charlie)::charlieMonkeyIdle:::` | 0x10005c9bc | minTime, maxTime | 1 | charlie_story | - |
| `PygmyLogic(Charlie)::charlieMonkeyWalk:::` | 0x10005c78c | minTime, maxTime | 1 | charlie_story | - |
| `PygmyLogic(Charlie)::checkForWarlock:::` | 0x10005c2b4 | - | 1 | charlie_story | - |
| `PygmyLogic(Charlie)::halfPygmyIdle:::` | 0x10005c68c | minTime, maxTime | 2 | charlie_story | - |
| `PygmyLogic(Charlie)::halfPygmyWalk:::` | 0x10005c484 | minTime, maxTime | 2 | charlie_story | - |
| `PygmyLogic(Island)::burstPygmyEarDrums:::` | 0x1000b74a0 | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::carryPygmyOffScreen:::` | 0x1000b728c | - | 2 | ghost_zombie_vampire | BirdLogic |
| `PygmyLogic(Island)::chaseKonkeyDongComplete:::` | 0x1000a8bf8 | - | 1 | island_life | - |
| `PygmyLogic(Island)::checkDanceChange:::` | 0x1000a8b0c | - | 6 | island_life | - |
| `PygmyLogic(Island)::checkForDance:::` | 0x1000ba58c | - | 3 | island_life | MonkeyLogic |
| `PygmyLogic(Island)::checkForDay:::` | 0x1000ba44c | - | 1 | island_life | - |
| `PygmyLogic(Island)::checkForDreamBubble:::` | 0x1000ba4d8 | - | 1 | island_life | - |
| `PygmyLogic(Island)::checkForGingerSunBurnt:::` | 0x1000b9e38 | - | 3 | island_hazards | - |
| `PygmyLogic(Island)::checkForNight:::` | 0x1000ba360 | - | 2 | island_life | - |
| `PygmyLogic(Island)::checkForNoBonFire:::` | 0x1000b999c | - | 1 | island_life | - |
| `PygmyLogic(Island)::checkForNoSunset:::` | 0x1000ba2c4 | - | 1 | island_life | - |
| `PygmyLogic(Island)::checkForRainbow:::` | 0x1000b9a28 | - | 1 | island_life | - |
| `PygmyLogic(Island)::checkForSpider:::` | 0x1000b96c0 | - | 3 | island_hazards | - |
| `PygmyLogic(Island)::checkForSunBurnt:::` | 0x1000b9f0c | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::checkForSunFrozen:::` | 0x1000ba108 | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::checkForSunset:::` | 0x1000b9aac | - | 2 | island_life | - |
| `PygmyLogic(Island)::checkForSunState:::` | 0x1000b9c28 | loop | 5 | island_hazards | - |
| `PygmyLogic(Island)::checkForTRex:::` | 0x1000b9078 | - | 3 | island_hazards | - |
| `PygmyLogic(Island)::checkForVampire:::` | 0x1000bde44 | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::checkKillPygmies:::` | 0x1000bd798 | minTime, maxTime | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::checkStayAwayFromSpider:::` | 0x1000b9360 | - | 2 | island_hazards | - |
| `PygmyLogic(Island)::checkStayAwayFromTRex:::` | 0x1000b8d18 | - | 2 | island_hazards | - |
| `PygmyLogic(Island)::chooseRainbowAnimation:::` | 0x1000ae75c | list | 2 | island_life | - |
| `PygmyLogic(Island)::eatBrainsComplete:::` | 0x1000b80a8 | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::eatenByMoronSnakeComplete:::` | 0x1000b1bd0 | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::explodeFromBansheeScream:::` | 0x1000b75d8 | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::explodeFromLovingPocketGod:::` | 0x1000ba66c | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::explodeFromNotCrapping:::` | 0x1000ba894 | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::explodeVampireGinger:::` | 0x1000ba8f4 | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::explodeVampireSun:::` | 0x1000ba8d0 | - | 2 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::finishEatingBanana:::` | 0x1000b87bc | - | 2 | island_life | MonkeyLogic |
| `PygmyLogic(Island)::finishEatingBird:::` | 0x1000b892c | - | 2 | island_life | - |
| `PygmyLogic(Island)::finishEatingCoconut:::` | 0x1000b8490 | - | 1 | island_life | - |
| `PygmyLogic(Island)::finishEatingFish:::` | 0x1000b8b04 | - | 2 | island_life | - |
| `PygmyLogic(Island)::finishEatingSwordFish:::` | 0x1000b8ca8 | - | 1 | island_life | - |
| `PygmyLogic(Island)::ghostFlyIdle:::` | 0x1000b7778 | minTime, maxTime | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::ghostFlyUp:::` | 0x1000b7674 | - | 2 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::ghostFollowPath:::` | 0x1000b7ad4 | - | 2 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::gushOverRainbowComplete:::` | 0x1000aea84 | - | 1 | island_life | - |
| `PygmyLogic(Island)::iglooRoll:::` | 0x1000be498 | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::initCarriedAwayByGhostComplete:::` | 0x1000b71c8 | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::jumpOnFire:::` | 0x1000be72c | - | 2 | island_life | - |
| `PygmyLogic(Island)::killPygmies:::` | 0x1000be098 | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::onFallIntoCrackHoleComplete:::` | 0x1000b6f70 | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::penetrateGroundComplete:::` | 0x1000b709c | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::pickUpBanana:::` | 0x1000b8788 | - | 1 | island_life | - |
| `PygmyLogic(Island)::pickUpBird:::` | 0x1000b889c | - | 1 | island_life | - |
| `PygmyLogic(Island)::pickUpCoconut:::` | 0x1000b845c | - | 1 | island_life | - |
| `PygmyLogic(Island)::pickUpFish:::` | 0x1000b8a74 | - | 1 | island_life | - |
| `PygmyLogic(Island)::pickUpSwordFish:::` | 0x1000b8c5c | - | 1 | island_life | - |
| `PygmyLogic(Island)::pickUpWood:::` | 0x1000b8570 | - | 1 | island_life | - |
| `PygmyLogic(Island)::possessedByGhostComplete:::` | 0x1000b6fac | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::pulledUnderComplete:::` | 0x1000b70d8 | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::resetCrapper:::` | 0x1000ba930 | - | 1 | island_life | - |
| `PygmyLogic(Island)::resetIgloo:::` | 0x1000ba918 | - | 1 | island_life | - |
| `PygmyLogic(Island)::runUnderSpider:::` | 0x1000b94f8 | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::runUnderTRex:::` | 0x1000b8eb0 | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::sharkSwallow:::` | 0x1000be808 | - | 1 | island_life | - |
| `PygmyLogic(Island)::startBansheeScream:::` | 0x1000b7354 | - | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::startFishing:::` | 0x1000bd51c | - | 2 | island_life | - |
| `PygmyLogic(Island)::startSinking:::` | 0x1000bac8c | - | 1 | underwater_nopee | - |
| `PygmyLogic(Island)::startSleeping:::` | 0x1000ba630 | - | 1 | island_life | - |
| `PygmyLogic(Island)::startStaringAtRainbow:::` | 0x1000aea68 | - | 2 | island_life | - |
| `PygmyLogic(Island)::stopDanceComplete:::` | 0x1000ae740 | - | 7 | island_life | - |
| `PygmyLogic(Island)::stopStaringAtKidnapping:::` | 0x1000a8bd4 | - | 1 | island_life | - |
| `PygmyLogic(Island)::sunBurntComplete:::` | 0x1000bc9e0 | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::sunWalk:::` | 0x1000bd214 | minTime, maxTime | 2 | island_hazards | - |
| `PygmyLogic(Island)::takeCrap:::` | 0x1000bab7c | - | 1 | island_life | - |
| `PygmyLogic(Island)::tripIntoZombieGrave:::` | 0x1000b80e8 | - | 2 | containers_and_catchers | - |
| `PygmyLogic(Island)::tugOfWarWithSquid:::` | 0x1000b6bbc | - | 1 | island_hazards | - |
| `PygmyLogic(Island)::uncrushed:::` | 0x1000be7f0 | - | 2 | island_hazards | - |
| `PygmyLogic(Island)::vampireRun:::` | 0x1000bda9c | minTime, maxTime | 1 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::waitForNight:::` | 0x1000bde8c | minTime, maxTime | 2 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::waitToCrap:::` | 0x1000ba948 | minTime, maxTime | 2 | island_life | - |
| `PygmyLogic(Island)::walkToBonFire:::` | 0x1000b85d4 | - | 6 | island_life | - |
| `PygmyLogic(Island)::walkToFishingPole:::` | 0x1000bd648 | - | 1 | island_life | - |
| `PygmyLogic(Island)::walkToIceFishingPole:::` | 0x1000bc86c | - | 1 | island_life | - |
| `PygmyLogic(Island)::watchShark:::` | 0x1000be880 | - | 4 | island_life | - |
| `PygmyLogic(Island)::zombieIdle:::` | 0x1000b833c | minTime, maxTime | 2 | ghost_zombie_vampire | - |
| `PygmyLogic(Island)::zombieWalk:::` | 0x1000b81e0 | minTime, maxTime, noFlip | 6 | ghost_zombie_vampire | - |
| `PygmyLogic(KonkeyDong)::konkeyDongFall:::` | 0x10003a220 | - | 1 | minigames | - |
| `PygmyLogic(KonkeyDong)::konkeyDongIntroComplete:::` | 0x10003a07c | - | 1 | minigames | - |
| `PygmyLogic(KonkeyDong)::konkeyDongJumpUp:::` | 0x10003a160 | - | 1 | minigames | - |
| `PygmyLogic(KonkeyDong)::konkeyDongLevelJumpComplete:::` | 0x10003a118 | - | 1 | minigames | - |
| `PygmyLogic(KonkeyDong)::onStartKonkeyDongFall:::` | 0x10003a238 | - | 1 | minigames | - |
| `PygmyLogic(MoronPests)::moronPestsIntroComplete:::` | 0x10011389c | - | 1 | minigames | - |
| `PygmyLogic(MoronPests)::moronPestsRun:::` | 0x100113958 | - | 4 | minigames | - |
| `PygmyLogic(MoronPests)::onEatenByMoronTurtleEatComplete:::` | 0x100113838 | - | 1 | minigames | - |
| `PygmyLogic(MoronPests)::onStartMoronPestsFall:::` | 0x100113884 | - | 1 | minigames | - |
| `PygmyLogic(NoPee)::onNoPeeHitComplete:::` | 0x10018b6f0 | - | 4 | underwater_nopee | - |
| `PygmyLogic(NoPee)::onPygmyNoPeeRollComplete:::` | 0x10018b680 | - | 1 | underwater_nopee | - |
| `PygmyLogic(NoPee)::waitUntilOffScreen:::` | 0x10018b698 | - | 3 | underwater_nopee | - |
| `PygmyLogic(OogaJump)::doodleJetPackComplete:::` | 0x1000085fc | - | 1 | minigames | - |
| `PygmyLogic(Story)::checkForStoryMode:::` | 0x100174e74 | - | 1 | charlie_story | - |
| `PygmyLogic(Story)::closeWordBalloon:::` | 0x1001744d4 | - | 1 | charlie_story | - |
| `PygmyLogic(Story)::dieInStory:::` | 0x100174de4 | - | 10 | charlie_story | - |
| `PygmyLogic(Story)::dieInStoryComplete:::` | 0x100174e10 | - | 11 | charlie_story | - |
| `PygmyLogic(Story)::openWordBalloon:::` | 0x1001744a8 | message | 1 | charlie_story | - |
| `PygmyLogic(Story)::storyModeFlick:::` | 0x100174d10 | - | 1 | charlie_story | - |
| `PygmyLogic(Story)::storyModeKnockOffIsland:::` | 0x100174c58 | - | 2 | charlie_story | - |
| `PygmyLogic(TheRuns)::onStartTheRunsFall:::` | 0x10013a358 | - | 1 | minigames | - |
| `PygmyLogic(TheRuns)::onTheRunsExplodeComplete:::` | 0x10013a558 | - | 1 | minigames | - |
| `PygmyLogic(TheRuns)::onTheRunsSplashComplete:::` | 0x10013a50c | - | 1 | minigames | - |
| `PygmyLogic(TheRuns)::theRunsFall:::` | 0x10013a4f4 | - | 2 | minigames | - |
| `PygmyLogic(TheRuns)::theRunsJumpUp:::` | 0x10013a370 | - | 1 | minigames | - |
| `PygmyLogic(TheRuns)::theRunsRun:::` | 0x10013a46c | - | 3 | minigames | - |
| `PygmyLogic(Underwater)::addUnderwaterComplete:::` | 0x10011f8a4 | - | 1 | underwater_nopee | - |
| `PygmyLogic(Underwater)::checkForLaserSharkDisabled:::` | 0x10011f844 | - | 1 | underwater_nopee | - |
| `PygmyLogic(Underwater)::checkForLaserSharkEnabled:::` | 0x10011f7e4 | - | 1 | underwater_nopee | - |
| `PygmyLogic(Underwater)::fallIntoDrain:::` | 0x10011fd50 | - | 1 | underwater_nopee | - |
| `PygmyLogic(Underwater)::onJellyfishSuffocateAttackComplete:::` | 0x10011cad8 | - | 1 | underwater_nopee | - |
| `PygmyLogic(Underwater)::painDrainCanTurn:::` | 0x10011fd34 | - | 4 | underwater_nopee | - |
| `PygmyLogic(Underwater)::panicSwim:::` | 0x10011fb3c | minTime, maxTime | 1 | underwater_nopee | - |
| `PygmyLogic(Underwater)::swim:::` | 0x10011f944 | minTime, maxTime, noReset | 16 | underwater_nopee | FishLogic, JellyfishLogic, LaserSharkLogic, PiranhaLogic, SharkLogic |
| `PygmyLogic::checkPygmyDialog:::` | 0x10012ff78 | alternateSequenceId, id | 32 | pygmy_behaviour_items | - |
| `PygmyLogic::die:::` | 0x10012ecb8 | - | 119 | pygmy_behaviour_items | 47 classes |
| `PygmyLogic::disableLabels:::` | 0x10012ea44 | - | 18 | pygmy_behaviour_items | - |
| `PygmyLogic::drag:::` | 0x100132094 | yOrigin, xOrigin | 40 | pygmy_touch_drag | 23 classes |
| `PygmyLogic::explode:::` | 0x10012fcc0 | - | 8 | pygmy_behaviour_items | - |
| `PygmyLogic::fall:::` | 0x100130fb8 | ignoreIsland | 134 | pygmy_behaviour_items | 26 classes |
| `PygmyLogic::flipXScale:::` | 0x10012ea5c | - | 1 | pygmy_behaviour_items | - |
| `PygmyLogic::hangUpsideDown:::` | 0x10012eab4 | - | 1 | pygmy_core_frame | - |
| `PygmyLogic::idle:::` | 0x10012eb58 | minTime, maxTime | 17 | pygmy_behaviour_items | MonkeyLogic, MoronSnakeLogic, SpiderLogic, TRexLogic |
| `PygmyLogic::panicRun:::` | 0x10013089c | minTime, totalTime, maxTime | 3 | pygmy_behaviour_items | - |
| `PygmyLogic::pushSequenceBasedOnPersonality:::` | 0x100132a64 | defaultId, charlieId, tomId, babyPlagueId, babyPurificationId, oldFartPlagueId, purificationId, gingerPlagueId, plagueId, fastPurificationId, gingerPurificationId, fastPlagueId, oldFartPurificationId, fastId, babyId, oldFartId, gingerId | 34 | pygmy_behaviour_items | - |
| `PygmyLogic::resetVel:::` | 0x1001301c8 | - | 5 | pygmy_behaviour_items | - |
| `PygmyLogic::run:::` | 0x1001306bc | minTime, maxTime | 2 | pygmy_behaviour_items | - |
| `PygmyLogic::runSequenceBasedOnPersonality:::` | 0x100132c9c | defaultId, babyId, charlieId, tomId, fastId, oldFartId, gingerId, babyPurificationId, babyPlagueId, oldFartPlagueId, gingerPlagueId, fastPurificationId, gingerPurificationId, fastPlagueId, oldFartPurificationId, purificationId, plagueId | 14 | pygmy_behaviour_items | - |
| `PygmyLogic::setAnimationBasedOnPygmyType:::` | 0x100132898 | oldFartPurification, oldFartPlague, normal, babyPlague, gingerPlague, purification, fastPurification, plague, gingerPurification, babyPurification, fastPlague, fast, ginger, baby, oldFart, normalPurification, normalPlague | 56 | pygmy_behaviour_items | ApocalypseTempleDoorLogic, ApocalypseTimeTemplePlatformLogic |
| `PygmyLogic::setOrigin:::` | 0x1001326a4 | zPos, xPos, yPos, inPlace | 72 | pygmy_behaviour_items | BCLogic |
| `PygmyLogic::slide:::` | 0x100130b94 | - | 20 | pygmy_behaviour_items | 10 classes |
| `PygmyLogic::tumble:::` | 0x1001317ec | - | 5 | pygmy_behaviour_items | - |
| `PygmyLogic::walk:::` | 0x1001301e8 | minTime, maxTime, direction, endOfIsland | 16 | pygmy_behaviour_items | IceMonsterLogic, KonkeyDongBeetleLogic, MonkeyLogic, SpiderLogic, TRexLogic |

## 27. Animation frame actions (42)

Selectors called by animation frames (logic.md) with no arguments.

| Frame action | Address | Effect |
|---|---|---|
| `PygmyLogic::animChooseDirection` | 0x10012e920 | zombie walk: pick the walk direction (§10.7) |
| `PygmyLogic::animFlipComplete` | 0x10012ea0c | `mbGhostFlipping = 0` |
| `PygmyLogic(Island)::animGhostXFlip` | 0x1000b7a5c | ghost faces its heading (§10.9) |
| `PygmyLogic(Island)::animThrowSpear` | 0x100094c78 | throw the held spear (§10.3) |
| `PygmyLogic(Island)::animInitBob` | 0x100094cec | pick up the pole, init the bob texture (§10.1) |
| `PygmyLogic(Island)::animHitHumpingMonkey` | 0x1000aeb2c | punch the humping monkey (§10.2) |
| `PygmyLogic(Island)::animBabyThrowSnake` | 0x1000b1b08 | baby escapes the Moron snake (§10.3) |
| `PygmyLogic(Island)::animPygmyScaredComplete` | 0x1000b29a4 | scared victim notifies the ghost (§10.9) |
| `PygmyLogic(Island)::animSleepBrainsEatenXFlip` | 0x1000b3fc8 | sleeping zombie victim faces its eater (§10.7) |
| `PygmyLogic(Island)::animThrowWood` | 0x1000b8724 | throw the wood on the bonfire pile (§10.1) |
| `PygmyLogic(Island)::animSwordFishSwallow` | 0x1000b8c90 | `mbAteSwordFish = 1` (§10.1) |
| `PygmyLogic(Apocalypse)::animReleaseThetan` | 0x10003ecac | `stopBeingPossessedByThetan` (§11.6) |
| `PygmyLogic(Apocalypse)::animStopThetanPossession` | 0x10003fdc4 | `stopBeingPossessedByThetan` (§11.6) |
| `PygmyLogic(Apocalypse)::animPounceComplete` | 0x10003fc74 | brawl victim: release channel 16, `[attacker thetanPounceComplete]` (§11.6) |
| `PygmyLogic(Apocalypse)::animLandThetanPunch` | 0x10003fd58 | the opponent `thetanBrawlLandPunch` (§11.6) |
| `PygmyLogic(Apocalypse)::animTomBabySacrificeComplete` | 0x100040aec | Tom: hide the baby label (§11.4) |
| `PygmyLogic(Apocalypse)::animTomShowXenu` | 0x100040b14 | Tom: drop the baby, spawn Xenu (§11.4) |
| `PygmyLogic(Apocalypse)::animSlamClosetDoorComplete` | 0x100041bf4 | Tom: `[door slamDoor]` (§11.4) |
| `PygmyLogic(Apocalypse)::animSpawnSkullAndBone` | 0x100044068 | locust victim: launch a skull and a bone (§11.7) |
| `PygmyLogic(Apocalypse)::animStartPygmyRecoil` | 0x1000468c8 | diseased pygmy: neighbours within 100 recoil (§11.5) |
| `PygmyLogic(Apocalypse)::animInfectPygmies` | 0x100046aac | diseased pygmy: infect neighbours (§11.5) |
| `PygmyLogic(Apocalypse)::animStartFloat` | 0x100047810 | plague body `vel.x = 20` (§11.5) |
| `PygmyLogic(Apocalypse)::animThrowPlagueBody` | 0x100047918 | carrier throws the body on the pile (§11.5) |
| `PygmyLogic(Apocalypse)::animPickupPlagueBody` | 0x1000479d0 | carrier picks the body up (§11.5) |
| `PygmyLogic(Charlie)::animCutPygmyInHalf` | 0x100059588 | Charlie completes the cut (§12) |
| `PygmyLogic(Charlie)::animCharlieLaunchShark` | 0x1000595e4 | Charlie launches the shark off his head (§12) |
| `PygmyLogic(Charlie)::animCharlieThrowFood` | 0x100059648 | Charlie throws his food away (§12) |
| `PygmyLogic(Charlie)::animWarlockBurnVampire` | 0x100059754 | warlock burns a vampire (§12) |
| `PygmyLogic(Charlie)::animWarlockStrike` | 0x1000598dc | warlock lightning (§12) |
| `PygmyLogic(Underwater)::animJellyfishAttackComplete` | 0x10011ca94 | `[jellyfish onJellyfishAttackComplete:self]` (§14.2) |
| `PygmyLogic(BattleOfTheGods)::animBattleChangeWeapon` | 0x100123608 | battle weapon behaviours by realm and style (§16) |
| `PygmyLogic(BattleOfTheGods)::animBattleThrowEnergyBall1` | 0x100123f5c | throw energy ball 1 (§16) |
| `PygmyLogic(BattleOfTheGods)::animBattleThrowEnergyBall2` | 0x100123ff8 | throw energy ball 2 (§16) |
| `PygmyLogic(BattleOfTheGods)::animBattleThrowEnergyBall3` | 0x100124094 | throw energy ball 3 (§16) |
| `PygmyLogic(BattleOfTheGods)::animBattleThrowEnergyBall4` | 0x100124130 | throw energy ball 4, the last (§16) |
| `PygmyLogic(BattleOfTheGods)::animBattleTie` | 0x1001241f0 | player side: `[IdolManager onBattleAttackTie]` (§16) |
| `PygmyLogic(BattleOfTheGods)::animBattleEnergyCounterHit` | 0x100124240 | opponent fast → killed (§16) |
| `PygmyLogic(BattleOfTheGods)::animBattleStrongHit` | 0x100124280 | counter beats fast, strong beats counter (§16) |
| `PygmyLogic(BattleOfTheGods)::animBattleFastHit` | 0x100124300 | fast hit resolution (§16) |
| `PygmyLogic(NoPee)::animNoPeeEmbedDie` | 0x10018a6bc | embed `NoPeeEmbedDie` (§15) |
| `PygmyLogic(NoPee)::animNoPeeFloatUpward` | 0x10018a6e4 | gravity on (§15) |
| `PygmyLogic(NoPee)::animNoPeeSinkDownward` | 0x10018a6f8 | `mbNoPeeTurnToStone` (§15) |

## 28. Ivar catalogue by feature

PygmyLogic declares 329 ivars (offsets 280..1604, read from the ivar offset variables in
`__DATA`); the helper classes follow. Grouped by the implementation unit that owns them
(§29): an ivar referenced from five or more units is shared state and belongs to
`pygmy_core_frame`, otherwise to the unit with the most references (ties: earlier unit).
The reference counts are loads of the ivar offset variable found in the disassembly per
method. Four ivars are never referenced (`mGravityTarget`, `mFollowOutHouseStall`,
`mbGhostFollowingPath`, `mbPainDrainTurnRight`) and were assigned by name. Types: BOOL is
one byte, VECTOR4 is four floats (x, y, z, w).

#### persona_and_bottom

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 72 | `GodPersona::mGodType` | int | - |
| 80 | `GodPersona::mAchievementDictionary` | NSMutableDictionary | - |
| 8 | `Persona::mPreferences` | BCPreferences | - |
| 16 | `Persona::mId` | NSString | - |
| 24 | `Persona::mName` | NSString | - |
| 32 | `Persona::mDescription` | NSString | - |
| 40 | `Persona::mMood` | int | - |
| 44 | `Persona::mTotalSacrifices` | int | - |
| 48 | `Persona::mSacrificeDictionary` | NSMutableDictionary | - |
| 56 | `Persona::mTotalGifts` | int | - |
| 64 | `Persona::mGiftDictionary` | NSMutableDictionary | - |
| 280 | `PygmyBottomLogic::mPygmyBottomLogicListener` | <PygmyBottomLogicListener> | - |
| 288 | `PygmyBottomLogic::mbIsActive` | BOOL | - |
| 72 | `PygmyPersona::mbActive` | BOOL | - |
| 76 | `PygmyPersona::mPersonaPersonality` | int | - |
| 80 | `PygmyPersona::mPygmyLogic` | PygmyLogic | - |
| 72 | `TribePersona::mPygmyPersonaArray` | NSMutableArray | - |
| 80 | `TribePersona::mDeadPygmyPersonaArray` | NSMutableArray | - |
| 88 | `TribePersona::mLivingPygmyPersonaArray` | NSMutableArray | - |
| 96 | `TribePersona::mGuestPygmyPersonaArray` | NSMutableArray | - |

#### pygmy_core_frame

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 280 | `mPygmyLogicListener` | <PygmyLogicListener> | pygmy_core_frame (12), pygmy_behaviour_items (19), pygmy_touch_drag (9), island_life (50), island_hazards (29), containers_and_catchers (28), ghost_zombie_vampire (14), apocalypse (44), charlie_story (14), underwater_nopee (18), minigames (12) |
| 288 | `mPygmyPersona` | PygmyPersona | pygmy_core_frame (3) |
| 296 | `mPersonaPersonality` | int | pygmy_core_frame (2), pygmy_behaviour_items (4), island_life (7), ghost_zombie_vampire (1), apocalypse (10), charlie_story (1), underwater_nopee (2) |
| 316 | `mSpringForce` | VECTOR4 | pygmy_core_frame (2), pygmy_touch_drag (1), island_hazards (1) |
| 332 | `mLastPos` | VECTOR4 | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (3), island_life (4), containers_and_catchers (2), underwater_nopee (1) |
| 364 | `mGravityTarget` | VECTOR4 | unreferenced |
| 380 | `mIslandYPos` | float | pygmy_core_frame (5), pygmy_behaviour_items (3), island_life (5), island_hazards (1), containers_and_catchers (7), ghost_zombie_vampire (1), apocalypse (4), minigames (2) |
| 388 | `mSlideFriction` | float | pygmy_core_frame (2) |
| 400 | `mActiveTouch` | BCTouch | pygmy_behaviour_items (1), pygmy_touch_drag (7), island_hazards (1), ghost_zombie_vampire (2), underwater_nopee (5), minigames (4) |
| 408 | `mbShakeUp` | BOOL | pygmy_core_frame (2) |
| 412 | `mShakeCount` | int | pygmy_core_frame (5) |
| 416 | `mLastShakeTime` | float | pygmy_core_frame (1) |
| 420 | `mbEarthquake` | BOOL | pygmy_core_frame (6), pygmy_behaviour_items (1), island_hazards (1) |
| 424 | `mTiltState` | int | pygmy_core_frame (7), pygmy_behaviour_items (3), pygmy_touch_drag (4), island_life (12), island_hazards (38), containers_and_catchers (5), ghost_zombie_vampire (8), apocalypse (24), charlie_story (7), underwater_nopee (8), minigames (1) |
| 428 | `mbOnIsland` | BOOL | pygmy_core_frame (5), pygmy_behaviour_items (1), apocalypse (2), minigames (1) |
| 504 | `mCookTime` | float | pygmy_core_frame (1) |
| 536 | `mbDroppedByHurricane` | BOOL | pygmy_core_frame (2), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_life (11), island_hazards (38), containers_and_catchers (16), ghost_zombie_vampire (4), apocalypse (17), charlie_story (7), underwater_nopee (4) |
| 584 | `mDragLabelXOffset` | float | pygmy_core_frame (2), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_hazards (1), apocalypse (1), underwater_nopee (2) |
| 588 | `mDragLabelYOffset` | float | pygmy_core_frame (2), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_hazards (1), apocalypse (1), underwater_nopee (3) |
| 592 | `mNameTextXOffset` | float | pygmy_core_frame (1) |
| 596 | `mNameTextYOffset` | float | pygmy_core_frame (2) |
| 600 | `mNameBackground` | BCRect | pygmy_core_frame (7), pygmy_behaviour_items (1) |
| 608 | `mNameText` | BCText | pygmy_core_frame (7), pygmy_behaviour_items (1) |
| 616 | `mName` | NSString | pygmy_core_frame (4) |
| 624 | `mAshPileDisplayObject` | BCDisplayObject | pygmy_core_frame (2), pygmy_behaviour_items (2) |
| 640 | `mbEnableIslandFloorCollision` | BOOL | pygmy_core_frame (1), pygmy_behaviour_items (4), pygmy_touch_drag (3), island_life (14), island_hazards (6), containers_and_catchers (35), ghost_zombie_vampire (2), apocalypse (22), underwater_nopee (5), minigames (7) |
| 641 | `mbEnableIslandSideCollision` | BOOL | pygmy_core_frame (1), pygmy_behaviour_items (7), pygmy_touch_drag (2), island_life (1), island_hazards (7), ghost_zombie_vampire (3), apocalypse (8), charlie_story (2), underwater_nopee (4), minigames (6) |
| 644 | `mbIslandSideCollision` | BOOL | pygmy_core_frame (1), pygmy_behaviour_items (3), island_hazards (1), ghost_zombie_vampire (2), apocalypse (2), charlie_story (2) |
| 645 | `mbScreenSideCollision` | BOOL | pygmy_core_frame (3), pygmy_behaviour_items (1) |
| 646 | `mbInvalidSacrifice` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), containers_and_catchers (15), apocalypse (10), underwater_nopee (1), minigames (1) |
| 648 | `mSacrificeId` | NSString | pygmy_core_frame (5), pygmy_behaviour_items (17), pygmy_touch_drag (6), island_life (10), island_hazards (27), containers_and_catchers (22), ghost_zombie_vampire (11), apocalypse (20), charlie_story (4), underwater_nopee (20), minigames (4) |
| 677 | `mbEatenByPiranha` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (3), island_life (11), island_hazards (36), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (11) |
| 689 | `mbIsGhost` | BOOL | pygmy_core_frame (5), pygmy_behaviour_items (1), pygmy_touch_drag (5), island_life (11), island_hazards (39), ghost_zombie_vampire (5), apocalypse (11), charlie_story (7), underwater_nopee (4), minigames (1) |
| 794 | `mbTouchedByGhost` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (9), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 808 | `mbCarriedAwayByGhost` | BOOL | pygmy_core_frame (1), pygmy_behaviour_items (1), ghost_zombie_vampire (1) |
| 809 | `mbHoldEarsFromBansheeScream` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (6), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 810 | `mbExplodeFromBansheeScream` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (6), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 811 | `mbCaughtByMonkey` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (13), island_hazards (37), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6), underwater_nopee (2) |
| 825 | `mbMonkeyOnBack` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6), underwater_nopee (2) |
| 840 | `mbEatenByMoronSnake` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (3), island_life (11), island_hazards (39), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6), underwater_nopee (2) |
| 873 | `mbTrappedInZombieGrave` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (4), ghost_zombie_vampire (5), apocalypse (10), charlie_story (6) |
| 874 | `mbIsZombie` | BOOL | pygmy_core_frame (8), pygmy_behaviour_items (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (8), apocalypse (10), charlie_story (6), underwater_nopee (4), minigames (1) |
| 875 | `mbZombieExplode` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (5), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 877 | `mbTrappedInIdolPedestal` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (2), island_life (11), island_hazards (36), containers_and_catchers (3), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6) |
| 888 | `mbInsideTempleDoor` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (4), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6) |
| 904 | `mbInsideApocalypseTempleDoor` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (3), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6) |
| 920 | `mbOnTopApocalypseTimeTemplePlatform` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (3), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6) |
| 936 | `mbEatenByLocustQueen` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (4), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (2) |
| 952 | `mbThrownByLocusts` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (15), ghost_zombie_vampire (4), apocalypse (19), charlie_story (6), minigames (1) |
| 968 | `mbKilledByLocusts` | BOOL | pygmy_core_frame (5), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (2), minigames (1) |
| 970 | `mbMutatedByLocusts` | BOOL | pygmy_core_frame (1), pygmy_behaviour_items (1), pygmy_touch_drag (1), apocalypse (1) |
| 976 | `mbKilledByPlague` | BOOL | pygmy_core_frame (15), pygmy_behaviour_items (7), pygmy_touch_drag (10), island_life (11), island_hazards (39), containers_and_catchers (17), ghost_zombie_vampire (4), apocalypse (20), charlie_story (6), underwater_nopee (4), minigames (1) |
| 977 | `mbKilledByPlagueDisappear` | BOOL | pygmy_core_frame (2), pygmy_behaviour_items (1), apocalypse (2) |
| 978 | `mbKilledByPurification` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (2), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (12), charlie_story (6), underwater_nopee (2), minigames (1) |
| 979 | `mbInApocalypseTempleFountain` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (3), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6) |
| 1016 | `mbPossessedByThetan` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (15), charlie_story (6), underwater_nopee (2) |
| 1032 | `mbThetanBrawl` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (2), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (20), charlie_story (6) |
| 1033 | `mbDiedFromThetanBrawl` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (13), charlie_story (6), underwater_nopee (4) |
| 1056 | `mbOnClosetTempleThetanMeter` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (2), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (13), charlie_story (6) |
| 1072 | `mbOnClosetTempleCouch` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (2), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (15), charlie_story (6) |
| 1089 | `mbInClosetTempleClosetDoor` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (2), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (22), charlie_story (6), underwater_nopee (2) |
| 1090 | `mbSlammingClosetTempleClosetDoor` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (15), charlie_story (6), underwater_nopee (1) |
| 1104 | `mbHeldByXenu` | BOOL | pygmy_core_frame (5), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (13), charlie_story (6), underwater_nopee (1) |
| 1122 | `mbHeldInTomsArms` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (2), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (13), charlie_story (6) |
| 1136 | `mbHoldingBabyPygmy` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (2), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (14), charlie_story (6) |
| 1156 | `mTimeTemplePygmyType` | int | pygmy_core_frame (8), pygmy_behaviour_items (14), pygmy_touch_drag (12), island_life (23), island_hazards (41), containers_and_catchers (16), ghost_zombie_vampire (6), apocalypse (17), charlie_story (6), underwater_nopee (4), minigames (1) |
| 1162 | `mbTrappedInSpiderWeb` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (1), island_life (11), island_hazards (38), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1163 | `mbOnFailBoat` | BOOL | pygmy_core_frame (1), pygmy_touch_drag (1) |
| 1164 | `mbCutInHalfByCharlie` | BOOL | pygmy_core_frame (10), pygmy_behaviour_items (6), pygmy_touch_drag (8), island_life (11), island_hazards (39), containers_and_catchers (17), ghost_zombie_vampire (4), apocalypse (17), charlie_story (9), underwater_nopee (4), minigames (1) |
| 1195 | `mbSunBurnt` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (38), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1196 | `mbSunFrozen` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (2), island_life (11), island_hazards (40), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1205 | `mbHitByIcicle` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1206 | `mbBeheadedBySquid` | BOOL | pygmy_behaviour_items (1), pygmy_touch_drag (7), island_hazards (2), apocalypse (1), underwater_nopee (4), minigames (1) |
| 1224 | `mbAteSwordFish` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (3), island_life (13), island_hazards (37), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1225 | `mbPainDrain` | BOOL | pygmy_core_frame (7), pygmy_touch_drag (3), island_life (11), island_hazards (36), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (8) |
| 1229 | `mbPainDrainSpiked` | BOOL | pygmy_core_frame (1), underwater_nopee (1) |
| 1230 | `mbSuckedIntoDrain` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (3), island_life (11), island_hazards (36), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (7) |
| 1231 | `mbIglooRoll` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), containers_and_catchers (1), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1233 | `mbHitBySnowBall` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1234 | `mbRestingInIgloo` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (12), island_hazards (37), containers_and_catchers (5), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1248 | `mbInApeWench` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_life (11), island_hazards (36), containers_and_catchers (3), ghost_zombie_vampire (4), apocalypse (10), charlie_story (7), underwater_nopee (4) |
| 1256 | `mApeWenchLogic` | ApeWenchLogic | pygmy_behaviour_items (1), pygmy_touch_drag (1), island_hazards (1), containers_and_catchers (2), charlie_story (2) |
| 1264 | `mbInApeBongo` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_life (11), island_hazards (36), containers_and_catchers (3), ghost_zombie_vampire (4), apocalypse (10), charlie_story (7), underwater_nopee (4) |
| 1272 | `mApeBongoLogic` | ApeBongoLogic | pygmy_behaviour_items (1), pygmy_touch_drag (1), island_hazards (1), containers_and_catchers (2), charlie_story (2) |
| 1280 | `mbInApeAltar` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (4), ghost_zombie_vampire (4), apocalypse (10), charlie_story (8), underwater_nopee (4) |
| 1288 | `mApeAltarLogic` | ApeAltarLogic | pygmy_behaviour_items (1), pygmy_touch_drag (1), island_hazards (1), containers_and_catchers (3), charlie_story (2) |
| 1296 | `mbInDanceDanceExecution` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_life (11), island_hazards (35), containers_and_catchers (3), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6), underwater_nopee (4) |
| 1297 | `mbDanceDanceExecuted` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (1), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (2), minigames (1) |
| 1312 | `mbInGoGoCage` | BOOL | pygmy_core_frame (6), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_life (12), island_hazards (35), containers_and_catchers (2), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6), underwater_nopee (4) |
| 1328 | `mbHighOnNitrous` | BOOL | pygmy_core_frame (2), apocalypse (1) |
| 1334 | `mbRemoveFromStoryMode` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (5), apocalypse (11), charlie_story (7), underwater_nopee (7) |
| 1338 | `mbCaughtByBird` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (3), island_life (11), island_hazards (39), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1360 | `mbInCrackHole` | BOOL | pygmy_core_frame (4), island_life (11), island_hazards (38), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (2), minigames (1) |
| 1365 | `mbTheRunsSideCollision` | BOOL | pygmy_core_frame (1), minigames (1) |
| 1367 | `mbTheRunsBurnComplete` | BOOL | pygmy_core_frame (1), minigames (1) |
| 1369 | `mbTheRunsFallInLava` | BOOL | pygmy_core_frame (1), minigames (1) |
| 1373 | `mbKonkeyDongHit` | BOOL | pygmy_core_frame (2), minigames (2) |
| 1384 | `mKonkeyDongPickTime` | float | pygmy_core_frame (1), minigames (1) |
| 1388 | `mbMoronPests` | BOOL | pygmy_core_frame (2), minigames (2) |
| 1391 | `mbMoronPestsStarted` | BOOL | pygmy_core_frame (1), minigames (1) |
| 1393 | `mbMoronPestsEatenByTurtle` | BOOL | pygmy_core_frame (1), minigames (1) |
| 1400 | `mbWearingWaterWings` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (3), island_life (11), island_hazards (38), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6), underwater_nopee (2) |
| 1488 | `mbInKongStatue` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_life (11), island_hazards (36), containers_and_catchers (2), ghost_zombie_vampire (4), apocalypse (10), charlie_story (7), underwater_nopee (4) |
| 1489 | `mbKidnappedByKonkeyDong` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (4), island_life (12), island_hazards (37), containers_and_catchers (1), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1496 | `mKongStatueLogic` | KongStatueLogic | pygmy_behaviour_items (1), pygmy_touch_drag (1), island_hazards (1), containers_and_catchers (2), charlie_story (2) |
| 1512 | `mbBattleOfTheGods` | BOOL | pygmy_core_frame (6), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (1), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6), minigames (3) |
| 1561 | `mbDoodleJumping` | BOOL | pygmy_core_frame (6), pygmy_behaviour_items (3), pygmy_touch_drag (3), island_life (11), island_hazards (37), containers_and_catchers (16), ghost_zombie_vampire (4), apocalypse (16), charlie_story (7), underwater_nopee (1), minigames (1) |
| 1562 | `mbDoodleKnockedDown` | BOOL | pygmy_core_frame (2), apocalypse (1), charlie_story (1), underwater_nopee (1), minigames (3) |
| 1567 | `mbSkewered` | BOOL | pygmy_core_frame (5), pygmy_behaviour_items (1), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6), underwater_nopee (6) |
| 1572 | `mbUnderwater` | BOOL | pygmy_core_frame (6), pygmy_touch_drag (6), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (10) |
| 1573 | `mbCrapping` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (15), island_hazards (37), ghost_zombie_vampire (4), apocalypse (10), charlie_story (6) |
| 1574 | `mbEatenByTRex` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (39), ghost_zombie_vampire (4), apocalypse (11), charlie_story (7), underwater_nopee (4) |
| 1575 | `mbEatenBySpider` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (39), ghost_zombie_vampire (4), apocalypse (11), charlie_story (7), underwater_nopee (4) |
| 1576 | `mbExplodeFromNotCrapping` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (38), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1577 | `mbExplodeFromVampireSun` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (5), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1578 | `mbExplodeFromVampireGinger` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (5), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1579 | `mbExplodeFromLovingPocketGod` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (38), ghost_zombie_vampire (4), apocalypse (11), charlie_story (8), underwater_nopee (4) |
| 1581 | `mbDisableLabels` | BOOL | pygmy_core_frame (1), pygmy_behaviour_items (3), island_hazards (2), containers_and_catchers (1), apocalypse (2), charlie_story (1) |
| 1582 | `mbEnableUpsideDown` | BOOL | pygmy_core_frame (6) |
| 1583 | `mbTouchBeganInside` | BOOL | pygmy_core_frame (3), pygmy_behaviour_items (1), pygmy_touch_drag (10), island_life (11), island_hazards (37), ghost_zombie_vampire (6), apocalypse (10), charlie_story (6), underwater_nopee (9) |
| 1584 | `mbCatchShark` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (13), island_hazards (38), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1585 | `mbSharkOnHead` | BOOL | pygmy_core_frame (4), island_life (14), island_hazards (26), ghost_zombie_vampire (2), apocalypse (10), charlie_story (6), underwater_nopee (4) |
| 1587 | `mbCrushed` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (40), ghost_zombie_vampire (4), apocalypse (10), charlie_story (7) |
| 1589 | `mbBurntByMagnifyingGlass` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (11), island_hazards (38), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1590 | `mbSwarmed` | BOOL | pygmy_core_frame (5), pygmy_touch_drag (3), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1591 | `mbSleeping` | BOOL | pygmy_core_frame (5), island_life (5), island_hazards (11), ghost_zombie_vampire (3), apocalypse (2), charlie_story (1) |
| 1592 | `mbTumble` | BOOL | pygmy_core_frame (3), pygmy_behaviour_items (2), pygmy_touch_drag (2), island_life (11), island_hazards (37), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4), minigames (1) |
| 1593 | `mbDrowning` | BOOL | pygmy_core_frame (5), pygmy_behaviour_items (3), pygmy_touch_drag (2), island_life (11), island_hazards (37), containers_and_catchers (1), ghost_zombie_vampire (4), apocalypse (12), charlie_story (7), underwater_nopee (4) |
| 1594 | `mbDragging` | BOOL | pygmy_core_frame (6), pygmy_behaviour_items (1), pygmy_touch_drag (5), island_life (11), island_hazards (39), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (6) |
| 1595 | `mbFalling` | BOOL | pygmy_core_frame (8), pygmy_behaviour_items (6), pygmy_touch_drag (6), island_life (14), island_hazards (41), containers_and_catchers (17), ghost_zombie_vampire (7), apocalypse (22), charlie_story (9), underwater_nopee (2), minigames (7) |
| 1597 | `mbApplySpring` | BOOL | pygmy_core_frame (1), pygmy_behaviour_items (1), pygmy_touch_drag (2), island_hazards (1), apocalypse (1), underwater_nopee (3) |
| 1598 | `mbOnFireFromLava` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (12), island_hazards (37), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1599 | `mbOnFireFromBonFire` | BOOL | pygmy_core_frame (4), pygmy_touch_drag (3), island_life (12), island_hazards (37), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1600 | `mbAteBurntFood` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (3), island_life (13), island_hazards (37), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |
| 1601 | `mbCooked` | BOOL | pygmy_core_frame (12), pygmy_behaviour_items (5), pygmy_touch_drag (8), island_life (11), island_hazards (39), containers_and_catchers (17), ghost_zombie_vampire (4), apocalypse (16), charlie_story (6), underwater_nopee (4), minigames (1) |
| 1602 | `mbHitByLightning` | BOOL | pygmy_core_frame (4), pygmy_behaviour_items (1), pygmy_touch_drag (3), island_life (11), island_hazards (39), ghost_zombie_vampire (4), apocalypse (11), charlie_story (6), underwater_nopee (4) |

#### pygmy_behaviour_items

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 348 | `mTumblePos` | VECTOR4 | pygmy_behaviour_items (1), pygmy_touch_drag (1), containers_and_catchers (1), charlie_story (1) |
| 384 | `mNextTurnStallTime` | float | pygmy_behaviour_items (4) |
| 432 | `mBurningMagnifyingGlassLogic` | MagnifyingGlassLogic | pygmy_behaviour_items (2), island_hazards (2) |
| 560 | `mbHitIceMonster` | BOOL | pygmy_behaviour_items (2) |
| 632 | `mAshPileLogic` | BCLogic | pygmy_core_frame (1), pygmy_behaviour_items (2) |
| 642 | `mbEnableSlide` | BOOL | pygmy_behaviour_items (1) |
| 643 | `mbIslandFloorCollision` | BOOL | pygmy_core_frame (1), pygmy_behaviour_items (2), apocalypse (1), minigames (2) |
| 672 | `mStartFallYPos` | float | pygmy_behaviour_items (2) |
| 876 | `mbBrainsEaten` | BOOL | pygmy_behaviour_items (1), ghost_zombie_vampire (1) |
| 969 | `mbMouthFilledByLocusts` | BOOL | pygmy_behaviour_items (1), pygmy_touch_drag (1), apocalypse (1) |
| 1088 | `mbTellClosetToGrabNewTom` | BOOL | pygmy_behaviour_items (1), apocalypse (1) |
| 1160 | `mbHadHeartAttack` | BOOL | pygmy_behaviour_items (2), pygmy_touch_drag (1), containers_and_catchers (2) |
| 1232 | `mbIglooCannon` | BOOL | pygmy_behaviour_items (3), containers_and_catchers (1) |
| 1281 | `mbTornApartByApeAltar` | BOOL | pygmy_behaviour_items (2), containers_and_catchers (1) |
| 1331 | `mbFartherThanIceMonster` | BOOL | pygmy_behaviour_items (1) |
| 1596 | `mbDescending` | BOOL | pygmy_behaviour_items (1) |
| 1603 | `mbHitVolcano` | BOOL | pygmy_behaviour_items (2) |
| 1604 | `mbScreamDuringFall` | BOOL | pygmy_behaviour_items (1) |

#### pygmy_touch_drag

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 300 | `mTouchScreenPos` | VECTOR4 | pygmy_touch_drag (6), island_hazards (1) |
| 392 | `mOhStall` | int | pygmy_touch_drag (2) |
| 508 | `mbOverFire` | BOOL | pygmy_core_frame (1), pygmy_touch_drag (2), apocalypse (1), underwater_nopee (2) |
| 1361 | `mbTheRuns` | BOOL | pygmy_core_frame (2), pygmy_behaviour_items (1), pygmy_touch_drag (5), minigames (2) |
| 1372 | `mbKonkeyDong` | BOOL | pygmy_core_frame (2), pygmy_behaviour_items (1), pygmy_touch_drag (5), minigames (2) |
| 1401 | `mbNoPee` | BOOL | pygmy_core_frame (2), pygmy_behaviour_items (1), pygmy_touch_drag (5), underwater_nopee (3) |
| 1472 | `mKonkeyDongLadderTouch` | BCTouch | pygmy_core_frame (1), pygmy_touch_drag (2), minigames (2) |

#### island_life

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 448 | `mEatingSharkLogic` | SharkLogic | island_life (4), charlie_story (1) |
| 456 | `mEatingCoconutLogic` | CoconutLogic | island_life (6), charlie_story (1) |
| 464 | `mCarryingWoodLogic` | WoodLogic | island_life (5) |
| 472 | `mEatingFishLogic` | FishLogic | island_life (5), charlie_story (1) |
| 480 | `mEatingSwordFishLogic` | SwordFishLogic | island_life (5) |
| 488 | `mEatingBirdLogic` | BirdLogic | island_life (5), charlie_story (1) |
| 496 | `mEatingBananaLogic` | BananaLogic | island_life (6), charlie_story (1) |
| 512 | `mFishingPoleLogic` | FishingPoleLogic | pygmy_core_frame (2), pygmy_behaviour_items (1), island_life (9) |
| 520 | `mbThrowingSpear` | BOOL | island_life (2), island_hazards (2) |
| 528 | `mThrowingSpearLogic` | SpearLogic | island_life (2), island_hazards (1) |
| 540 | `mFollowOutHouseStall` | int | unreferenced |
| 544 | `mCrappingOutHouseLogic` | OutHouseLogic | pygmy_behaviour_items (1), island_life (5), charlie_story (1) |
| 552 | `mWaitingOutHouseLogic` | OutHouseLogic | pygmy_behaviour_items (2), island_life (4) |
| 568 | `mLeftWarmingBonFireLogic` | BonFireLogic | island_life (3) |
| 576 | `mRightWarmingBonFireLogic` | BonFireLogic | island_life (3) |
| 816 | `mThrowingMonkeyLogic` | MonkeyLogic | island_life (2) |
| 824 | `mbPlayingCatchWithMonkey` | BOOL | island_life (3) |
| 832 | `mHumpingMonkeyLogic` | MonkeyLogic | island_life (5) |
| 848 | `mDreamBubbleLogic` | DreamBubbleLogic | island_life (3) |
| 1193 | `mbCharlieNotSleeping` | BOOL | island_life (3) |
| 1204 | `mbIceFishing` | BOOL | island_life (4) |
| 1235 | `mbCrawledOutOfIgloo` | BOOL | island_life (2), containers_and_catchers (1) |
| 1329 | `mbStaringAtRainbow` | BOOL | island_life (4) |
| 1330 | `mbGushingOverRainbow` | BOOL | island_life (3) |
| 1490 | `mbStaringAtKidnapping` | BOOL | island_life (3) |
| 1544 | `mbStoppingDance` | BOOL | island_life (8) |
| 1545 | `mbIsThemeSongBackup` | BOOL | island_life (2) |
| 1552 | `mLastDanceSkinOptionValueId` | NSString | island_life (2) |
| 1580 | `mbFinishedCrapping` | BOOL | island_life (4) |

#### island_hazards

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 656 | `mSafeIslandMinXPos` | float | island_hazards (4) |
| 660 | `mSafeIslandMaxXPos` | float | island_hazards (4) |
| 664 | `mSafeRunIslandMinXPos` | float | island_hazards (4) |
| 668 | `mSafeRunIslandMaxXPos` | float | island_hazards (4) |
| 1161 | `mbGingerSunBurn` | BOOL | island_hazards (1) |
| 1192 | `mbWarlockSkipNextExplosion` | BOOL | island_hazards (2), charlie_story (2) |
| 1197 | `mbIceShatter` | BOOL | island_hazards (1) |
| 1200 | `mSunBurnFreezeCycleCount` | int | island_hazards (5) |
| 1207 | `mbGrabbedByTentacle` | BOOL | island_hazards (1) |
| 1208 | `mTentacleStrengthRot` | float | island_hazards (1) |
| 1212 | `mTentacleStrengthRotVel` | float | island_hazards (2) |
| 1216 | `mIceHoleLogic` | IceHoleLogic | pygmy_behaviour_items (1), island_hazards (2), containers_and_catchers (2) |
| 1344 | `mCarryingBirdLogic` | BirdLogic | pygmy_behaviour_items (1), island_hazards (3) |
| 1588 | `mbCrushedByBoulder` | BOOL | island_hazards (2), charlie_story (1) |

#### containers_and_catchers

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 856 | `mZombieGraveLogic` | ZombieGraveLogic | pygmy_behaviour_items (1), island_hazards (1), containers_and_catchers (5), ghost_zombie_vampire (1) |
| 880 | `mIdolPedestalLogic` | IdolPedestalLogic | pygmy_behaviour_items (1), containers_and_catchers (3) |
| 896 | `mTempleDoorLogic` | TempleDoorLogic | pygmy_behaviour_items (1), containers_and_catchers (5) |
| 912 | `mApocalypseTempleDoorLogic` | ApocalypseTempleDoorLogic | pygmy_behaviour_items (1), containers_and_catchers (4), apocalypse (1) |
| 928 | `mApocalypseTimeTemplePlatformLogic` | ApocalypseTimeTemplePlatformLogic | pygmy_behaviour_items (1), containers_and_catchers (3) |
| 944 | `mLocustQueenLogic` | LocustQueenLogic | pygmy_core_frame (1), pygmy_behaviour_items (1), containers_and_catchers (3) |
| 984 | `mApocalypseTempleFountainLogic` | ApocalypseTempleFountainLogic | pygmy_behaviour_items (1), containers_and_catchers (6) |
| 1240 | `mIglooLogic` | IglooLogic | pygmy_behaviour_items (1), island_life (1), containers_and_catchers (5) |
| 1304 | `mDanceDanceExecutionLogic` | DanceDanceExecutionLogic | pygmy_behaviour_items (1), pygmy_touch_drag (1), containers_and_catchers (3), charlie_story (1) |
| 1320 | `mGoGoCageLogic` | GoGoCageLogic | pygmy_behaviour_items (1), pygmy_touch_drag (1), containers_and_catchers (2), charlie_story (1) |

#### ghost_zombie_vampire

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 690 | `mbGhostFollowingPath` | BOOL | unreferenced |
| 691 | `mbGhostFlipping` | BOOL | ghost_zombie_vampire (3) |
| 692 | `mGhostTouchXOffset` | float | ghost_zombie_vampire (2) |
| 696 | `mGhostTouchYOffset` | float | ghost_zombie_vampire (2) |
| 700 | `mGhostTargetZRot` | float | pygmy_core_frame (1), ghost_zombie_vampire (3) |
| 704 | `mGhostTargetYPos` | float | ghost_zombie_vampire (2) |
| 712 | `mpGhostPathVector` | std::vector<VECTOR4>* | pygmy_core_frame (1), pygmy_behaviour_items (1), pygmy_touch_drag (1), ghost_zombie_vampire (3) |
| 720 | `mGhostLastPathPos` | VECTOR4 | ghost_zombie_vampire (1) |
| 736 | `mGhostLastTouchPos` | VECTOR4 | ghost_zombie_vampire (2) |
| 752 | `mGhostTouchPygmyLogic` | PygmyLogic | pygmy_behaviour_items (1), ghost_zombie_vampire (10) |
| 760 | `mGhostTouchType` | int | pygmy_core_frame (2), ghost_zombie_vampire (5) |
| 764 | `mGhostEmergeXPos` | float | ghost_zombie_vampire (2) |
| 768 | `mbGhostBansheeScream` | BOOL | ghost_zombie_vampire (2) |
| 769 | `mbGhostBansheeScreamStarted` | BOOL | pygmy_behaviour_items (1), ghost_zombie_vampire (2) |
| 776 | `mTheLightDisplayObject` | BCDisplayObject | pygmy_behaviour_items (1), ghost_zombie_vampire (3) |
| 784 | `mTheLightLogic` | MultiModelLogic | pygmy_behaviour_items (1), pygmy_touch_drag (1), ghost_zombie_vampire (2) |
| 792 | `mbGoIntoTheLight` | BOOL | pygmy_behaviour_items (1), pygmy_touch_drag (1), ghost_zombie_vampire (2) |
| 793 | `mbGhostActionTriggered` | BOOL | ghost_zombie_vampire (4) |
| 800 | `mGhostPygmyLogic` | PygmyLogic | pygmy_behaviour_items (1), ghost_zombie_vampire (11) |
| 864 | `mEatingBrainsPygmyLogic` | PygmyLogic | ghost_zombie_vampire (2) |
| 872 | `mbZombieChooseDirection` | BOOL | ghost_zombie_vampire (2) |
| 1586 | `mbBitByBat` | BOOL | ghost_zombie_vampire (6), apocalypse (2), charlie_story (3) |

#### apocalypse

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 953 | `mbUseLocustCatch1` | BOOL | apocalypse (3) |
| 960 | `mThrowingLocustLogic` | LocustLogic | apocalypse (3) |
| 972 | `mMutantLocustIdleLoopCount` | int | apocalypse (2) |
| 992 | `mDiseaseIncubateIdleCycles` | int | containers_and_catchers (1), apocalypse (4) |
| 996 | `mbPlagueBodyInPile` | BOOL | apocalypse (1) |
| 997 | `mbPlagueBodyPickedUp` | BOOL | pygmy_core_frame (1), apocalypse (5) |
| 998 | `mbPlagueBodyBeingCarried` | BOOL | apocalypse (4) |
| 1000 | `mPlagueBodyCarryingPygmyLogic` | PygmyLogic | pygmy_core_frame (1), apocalypse (4) |
| 1008 | `mCarryingPlagueBodyLogic` | PygmyLogic | pygmy_core_frame (1), apocalypse (4) |
| 1017 | `mbWaitingForPossessionByThetan` | BOOL | apocalypse (5) |
| 1024 | `mThetanLogic` | ThetanLogic | pygmy_behaviour_items (1), apocalypse (3) |
| 1036 | `mThetanBrawlCoolDown` | float | apocalypse (3) |
| 1040 | `mThetanAttackerPygmyLogic` | PygmyLogic | pygmy_behaviour_items (1), apocalypse (6) |
| 1048 | `mThetanDefenderPygmyLogic` | PygmyLogic | pygmy_behaviour_items (1), apocalypse (11) |
| 1064 | `mClosetTempleThetanMeterLogic` | ClosetTempleThetanMeterLogic | pygmy_behaviour_items (1), apocalypse (3) |
| 1080 | `mClosetTempleCouchLogic` | ClosetTempleCouchLogic | pygmy_behaviour_items (1), apocalypse (4) |
| 1096 | `mClosetTempleClosetDoorLogic` | ClosetTempleClosetDoorLogic | pygmy_behaviour_items (1), apocalypse (6) |
| 1112 | `mXenuLogic` | XenuLogic | pygmy_behaviour_items (1), apocalypse (4) |
| 1120 | `mbStartTomBabySacrifice` | BOOL | apocalypse (4) |
| 1121 | `mbSacrificedToXenu` | BOOL | pygmy_core_frame (1), apocalypse (3) |
| 1128 | `mTomPygmyLogic` | PygmyLogic | pygmy_behaviour_items (1), apocalypse (4) |
| 1144 | `mBabyPygmyLogic` | PygmyLogic | apocalypse (7) |
| 1152 | `mbTempleOldPygmyUpright` | BOOL | pygmy_behaviour_items (1), apocalypse (2) |

#### charlie_story

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 1168 | `mCharlieCuttingPygmyLogic` | PygmyLogic | charlie_story (4) |
| 1176 | `mCharliePygmyLogic` | PygmyLogic | charlie_story (4) |
| 1184 | `mCharlieOnBackMonkeyLogic` | MonkeyLogic | island_life (1), charlie_story (2) |
| 1194 | `mbAbsorbedByHalfPygmy` | BOOL | charlie_story (1) |
| 1332 | `mbStoryMode` | BOOL | pygmy_behaviour_items (6), pygmy_touch_drag (3), charlie_story (7) |
| 1333 | `mbStoryModeReady` | BOOL | charlie_story (3) |
| 1335 | `mbDieInStory` | BOOL | pygmy_core_frame (1), charlie_story (2) |
| 1336 | `mbDieInStoryResume` | BOOL | charlie_story (2) |
| 1337 | `mbDieInStoryComplete` | BOOL | charlie_story (2) |
| 1352 | `mWordBalloonLogic` | MultiModelLogic | pygmy_core_frame (1), charlie_story (3) |

#### underwater_nopee

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 440 | `mEatingPiranhaLogic` | PiranhaLogic | underwater_nopee (5) |
| 676 | `mbBaitMaster` | BOOL | pygmy_touch_drag (1), underwater_nopee (10) |
| 680 | `mHoldingChumLogic` | ChumLogic | underwater_nopee (3) |
| 688 | `mbHoldingChum` | BOOL | underwater_nopee (4) |
| 1226 | `mbPainDrainPlatformCollision` | BOOL | underwater_nopee (1) |
| 1227 | `mbPainDrainCanTurn` | BOOL | underwater_nopee (3) |
| 1228 | `mbPainDrainTurnRight` | BOOL | unreferenced |
| 1402 | `mbNoPeeIntroComplete` | BOOL | underwater_nopee (4) |
| 1403 | `mbNoPeeStarted` | BOOL | underwater_nopee (4) |
| 1404 | `mbNoPeeKilled` | BOOL | underwater_nopee (7) |
| 1405 | `mbNoPeeThrustOn` | BOOL | underwater_nopee (4) |
| 1406 | `mbNoPeeChangeDirection` | BOOL | underwater_nopee (5) |
| 1407 | `mbNoPeeTurnToStone` | BOOL | underwater_nopee (3) |
| 1408 | `mNoPeeThrustVel` | float | underwater_nopee (2) |
| 1412 | `mNoPeeTurnAroundTime` | float | underwater_nopee (4) |
| 1416 | `mNoPeeSwimXVel` | float | underwater_nopee (2) |
| 1420 | `mNoPeeSwimTargetXVel` | float | underwater_nopee (4) |
| 1424 | `mbNoPeeSwimmingLeft` | BOOL | underwater_nopee (4) |
| 1425 | `mbNoPeeChangingDirectionDownward` | BOOL | underwater_nopee (4) |
| 1428 | `mNoPeePosVel` | VECTOR4 | underwater_nopee (7) |
| 1448 | `mNoPeeEmbedLogic` | MultiModelLogic | underwater_nopee (3) |
| 1504 | `mJellyfishLogic` | JellyfishLogic | pygmy_behaviour_items (1), underwater_nopee (7) |
| 1564 | `mbAddedUnderwater` | BOOL | pygmy_core_frame (1), underwater_nopee (2) |
| 1565 | `mbActiveUnderwater` | BOOL | underwater_nopee (6) |
| 1566 | `mbSkeweredRight` | BOOL | underwater_nopee (2) |
| 1568 | `mbHitBySharkLaser` | BOOL | pygmy_core_frame (1), underwater_nopee (5) |
| 1569 | `mbEatenByLaserShark` | BOOL | pygmy_core_frame (1), underwater_nopee (6) |
| 1570 | `mbHitByJellyfish` | BOOL | pygmy_core_frame (1), underwater_nopee (8) |
| 1571 | `mbHitByUnderwaterPower` | BOOL | pygmy_core_frame (1), underwater_nopee (5) |

#### minigames

| offset | ivar | type | units referencing it (refs) |
|---:|---|---|---|
| 1362 | `mbTheRunsStarted` | BOOL | minigames (2) |
| 1363 | `mbTheRunsJumping` | BOOL | minigames (3) |
| 1364 | `mbTheRunsSkidding` | BOOL | minigames (5) |
| 1366 | `mbTheRunsRockExplode` | BOOL | pygmy_core_frame (1), minigames (2) |
| 1368 | `mbTheRunsPlatformCollision` | BOOL | minigames (3) |
| 1370 | `mbTheRunsFallOffSinkingPlatform` | BOOL | minigames (1) |
| 1371 | `mbTheRunsMaxSpeed` | BOOL | minigames (4) |
| 1374 | `mbKonkeyDongStarted` | BOOL | pygmy_core_frame (1), minigames (8) |
| 1375 | `mbKonkeyDongRunning` | BOOL | pygmy_core_frame (1), minigames (3) |
| 1376 | `mbKonkeyDongFalling` | BOOL | minigames (5) |
| 1377 | `mbKonkeyDongOnLadder` | BOOL | pygmy_core_frame (1), minigames (2) |
| 1378 | `mbKonkeyDongHoldingPick` | BOOL | pygmy_core_frame (1), pygmy_touch_drag (1), minigames (8) |
| 1379 | `mbKonkeyDongLevelTransition` | BOOL | pygmy_core_frame (1), minigames (4) |
| 1380 | `mbKonkeyDongPlatformCollision` | BOOL | minigames (3) |
| 1381 | `mbKonkeyDongJumpBoulder` | BOOL | minigames (2) |
| 1382 | `mbKonkeyDongJumpBeetle` | BOOL | minigames (2) |
| 1389 | `mbMoronPestsHitTurtle` | BOOL | minigames (2) |
| 1390 | `mbMoronPestsStartRunning` | BOOL | minigames (3) |
| 1392 | `mbMoronPestsFloorCollision` | BOOL | minigames (1) |
| 1396 | `mMoronPestsTargetXPosVel` | float | pygmy_core_frame (1), minigames (3) |
| 1456 | `mbKonkeyDongClimbing` | BOOL | pygmy_core_frame (1), minigames (2) |
| 1457 | `mbKonkeyDongLadderUp` | BOOL | pygmy_core_frame (1), pygmy_touch_drag (1), minigames (2) |
| 1464 | `mKonkeyDongJumpTouch` | BCTouch | pygmy_touch_drag (1), minigames (2) |
| 1480 | `mKonkeyDongLadderLogic` | KonkeyDongLadderLogic | minigames (3) |
| 1513 | `mbBattleOfTheGodsEnemy` | BOOL | minigames (19) |
| 1514 | `mbKilledDuringBattle` | BOOL | minigames (5) |
| 1520 | `mOpponentPygmyLogic` | PygmyLogic | minigames (16) |
| 1528 | `mpStructXmlEpisodeNodeDef` | pointer | minigames (8) |
| 1536 | `mpOpponentStructXmlEpisodeNodeDef` | pointer | minigames (19) |
| 1560 | `mbDoodleOnIce` | BOOL | apocalypse (1), charlie_story (1), underwater_nopee (1), minigames (6) |
| 1563 | `mbDoodleJetPack` | BOOL | pygmy_core_frame (1), minigames (7) |

## 29. Implementation partition

`tmp/re/pygmy-partition.json` (written by the reverse-engineering scratch script, sorted
and deterministic) splits all 725 methods into 12 units. Schema (one object per unit, in
dependency order):

```json
[
  {
    "unit": "pygmy_core_frame",
    "description": "PygmyLogic object core: ...",
    "methods": ["PygmyLogic::initWithDisplayObject:", "PygmyLogic::isSoundDisabled"],
    "ivars": ["mPygmyLogicListener", "mPygmyPersona"],
    "bytes": 99272,
    "depends_on": []
  }
]
```

| Field | Type | Meaning |
|---|---|---|
| `unit` | string | snake_case unit name, unique |
| `description` | string | what the unit covers |
| `methods` | string[] | `Class::selector` or `PygmyLogic(Category)::selector`; every one of the 725 methods appears exactly once; sorted by IMP address |
| `ivars` | string[] | PygmyLogic ivars by name, helper-class ivars as `Class::ivar`; declaration order; every ivar exactly once |
| `bytes` | integer | sum of the methods' code sizes |
| `depends_on` | string[] | earlier units whose methods this unit calls or whose ivars it uses; empty array when none |

Rust mapping: `struct PartitionUnit { unit: String, description: String, methods:
Vec<String>, ivars: Vec<String>, bytes: u64, depends_on: Vec<String> }`, the file is
`Vec<PartitionUnit>`.

| # | unit | methods | code bytes | ivars | depends_on | forward references (stubs) |
|---:|---|---:|---:|---:|---|---|
| 1 | `persona_and_bottom` | 60 | 48745 | 20 | - | pygmy_core_frame |
| 2 | `pygmy_core_frame` | 26 | 99272 | 136 | - | pygmy_behaviour_items, pygmy_touch_drag, island_life, island_hazards, containers_and_catchers, ghost_zombie_vampire, apocalypse, charlie_story, underwater_nopee, minigames |
| 3 | `pygmy_behaviour_items` | 19 | 72021 | 18 | pygmy_core_frame | pygmy_touch_drag, island_life, island_hazards, containers_and_catchers, ghost_zombie_vampire, apocalypse, charlie_story, underwater_nopee |
| 4 | `pygmy_touch_drag` | 11 | 55578 | 7 | pygmy_core_frame, pygmy_behaviour_items | island_life, island_hazards, containers_and_catchers, ghost_zombie_vampire, apocalypse, charlie_story, underwater_nopee, minigames |
| 5 | `island_life` | 104 | 194301 | 29 | pygmy_core_frame | containers_and_catchers, apocalypse, charlie_story |
| 6 | `island_hazards` | 78 | 366890 | 14 | pygmy_core_frame, pygmy_behaviour_items, pygmy_touch_drag, island_life | containers_and_catchers, ghost_zombie_vampire, apocalypse, charlie_story |
| 7 | `containers_and_catchers` | 55 | 100266 | 10 | pygmy_core_frame, pygmy_behaviour_items, island_life, island_hazards | apocalypse |
| 8 | `ghost_zombie_vampire` | 58 | 110594 | 22 | pygmy_core_frame, pygmy_behaviour_items, island_life, containers_and_catchers | apocalypse, charlie_story |
| 9 | `apocalypse` | 119 | 222853 | 23 | pygmy_core_frame, pygmy_behaviour_items, pygmy_touch_drag, island_life, containers_and_catchers, ghost_zombie_vampire | charlie_story, underwater_nopee, minigames |
| 10 | `charlie_story` | 46 | 95625 | 10 | pygmy_core_frame, pygmy_behaviour_items, island_life, island_hazards, containers_and_catchers, ghost_zombie_vampire, apocalypse | underwater_nopee, minigames |
| 11 | `underwater_nopee` | 69 | 97863 | 29 | pygmy_core_frame, pygmy_touch_drag, island_life, containers_and_catchers, apocalypse, charlie_story | minigames |
| 12 | `minigames` | 80 | 112119 | 31 | pygmy_core_frame, pygmy_behaviour_items, pygmy_touch_drag | - |

`depends_on` is a DAG in the listed order. References to later units ("forward
references") are almost all the stop lists (`stopEating`, `stopFishing`,
`stopBeingInsideTempleDoor`, … called by the core, touch and death code) and the
`checkFor*DuringFall` catchers called by `fall:::`; plus `onGameFrame:` calling the
mini-game collision checks and `handleNoPeeMovement:`. Implement them as one trait or
function table declared by `pygmy_core_frame` (every unit fills in its stop and catcher
entries), so the units can be written in order. The shared busy flags (the FREE set) live
in the core; every feature unit sets its own flag when its interaction starts.

Suggested Rust shape (one Bevy entity per pygmy): a `Pygmy` component with the core state
(flags of the FREE set, physics, tilt, touch, labels, persona link), feature components
or sub-structs per unit for their private ivars, the sequence player as its own component,
and the listener as a set of Bevy resources / events (§23).

## 30. Open questions and quirks

Kept as-is in a faithful port unless noted:

1. `canMeltIceHole` returns YES when the pygmy is **busy** (§10.6); `IceHoleLogic` uses
   the value directly. Kept.
2. `touchMovedEntered:allTouches:` compares the old touch x with `maxPos.x` twice
   (`b.ge` / `b.ls`), so every entry that is not from above or below is a side push (§5.4).
3. `TribePersona` never allocates its guest array: `addGuestPygmyPersona:` does nothing;
   guests only exist through `activateGuestPygmyPersona:` (§22.3).
4. `TribePersona::resurrectPygmyLogic:` takes `objectAtIndex:0` of the dead list without
   a check. `addPygmy` keeps the live count at or below the persona count, but
   `addBabyPygmy:::` has no limit; whether the data can create more pygmies than personas
   (which would raise an exception in the original) is not verified.
5. `mbPlagueBodyInPile` is read by `isAvailablePlagueBodyToBeCarried` but never written
   (always NO): a burnt-out body on the pile stays carriable in principle.
6. `AbsorbedByCharlie` is not in the controller's list of 116 known sacrifice ids; the
   list only feeds `getSacrificeLeaderboardId:`, which returns nil for every id anyway, so
   no per-id leaderboard is ever submitted.
7. `checkDiseaseIncubation:::` treats any disease other than 1 as purification (D 0 would
   die of purification); it is only reached from the plague/purification idle sequences.
8. Frame-rate dependent steps (per call, not per second): `ghostFollowPath:::` (16.666 per
   call), the ghost rotation easing (0.4), Moron Pests speed easing (0.5), the drag "Oh"
   cool-down (60 frames), the tentacle strength rotation. At the original 60 Hz these are
   per-frame constants; a port with a different tick rate must scale them.
9. `checkDoodleCloudCollision:` requires `z == -277.12` exactly (float equality).
10. A zombie shaken by an earthquake sends `onPygmyEarthquakeStart:` and
    `onPygmyEarthquakeStop:` back to back.
11. The out-of-world test (§4.1 step 17) calls `setBehavior:` again on every frame the
    pygmy is out of bounds.
12. `hitByLightning` drops the pygmy from the hurricane with `dropSpriteLogic:` (no
    `droppedByHurricane:` callback) and leaves `mbPickedUpByHurricane` set; harmless
    because the pygmy dies.
13. `mCookTime` is never reset: dangling time over the fire accumulates across drags.
14. A ghost death reports its sacrifice episode event twice (at the first death and when
    the ghost leaves through the light, §3.6).
15. Write-only or unreferenced ivars: never referenced `mGravityTarget`,
    `mFollowOutHouseStall`, `mbGhostFollowingPath`, `mbPainDrainTurnRight`; only written
    `mbEnableSlide` (cleared by `die:::`), `mbGrabbedByTentacle`, `mbAbsorbedByHalfPygmy`,
    `mbGingerSunBurn`.
16. `stopBeingHeldByXenu` does not clear `mbHeldByXenu`; `tornApartFromApeAltar` keeps
    `mbInApeAltar` set; `removeFromStoryMode` keeps the container flags. All are followed
    by death, so the flags only matter to guards evaluated during the death sequence.
17. `getLogic(50)` is the `MoronTurtleLogic` (logic.md object table); the pygmy reads its
    `xPosVel` for the race pace.
18. The exact meaning of `hitByPygmy:onFire:`'s second argument on the ice monster
    (`mbIglooCannon || T != 4`, i.e. YES for every normal flicked pygmy) belongs to
    `IceMonsterLogic`.
19. `onKonkeyDongStop`, `isGlacierEnabled`, `glacierLogic:`, `isEarthquakeActive` and
    `isLocustActive` are protocol members the pygmy classes never send.
