# Sequences: the data-driven scripting system

Pocket God drives almost all behaviour (every logic object's "behaviour", the boot
script, screens, mini-game flow) with *sequences*: named lists of items, each item
naming a handler method plus string parameters. In 1.48.2 there are **2548
sequences with 7258 items using 514 methods**, implemented by **722 handler
methods (533 selectors) in 126 classes/categories**.

| Deliverable | Content |
|---|---|
| `assets/data/sequences.json` | every sequence, items in order, parameters by name |
| `assets/data/sequence-methods.json` | per method: positional parameter names, handler implementations, use counts |
| `tools/re/extract_sequences.py` | regenerates both files and checks them against the 1.45.3 `sequence.xml` |

Source references are `Class::method @ address` (arm64 slice). Ghidra's decompiled
code is in `tmp/re/decomp/game/`; everything below was checked against it and,
where the C was ambiguous, against the disassembly.

## 1. Data format in the binary

### 1.1 Structures

ObjC type encodings (from `tmp/re/objc_meta.txt`):

```
BCSequenceDef     = {@ name; i count; ^^{BCSequenceItemDef} items}     // 24 bytes
BCSequenceItemDef = {@ methodName; : selector; ^^@ params}             // 24 bytes
handler           = i36@0:8@16B24^{BCSequenceItemDef}28                 // int (id self, SEL, id sequence, BOOL firstTime, BCSequenceItemDef *)
BCSequenceStackItem = {^{BCSequenceDef} mpSequenceDef; i mSequenceItemIndex}  // 16 bytes
```

| Offset | BCSequenceDef | BCSequenceItemDef |
|---|---|---|
| 0x00 | `name`: NSString (CFString constant) | `methodName`: NSString, e.g. `@"setAnimation"` |
| 0x08 | `count`: int32 (+4 bytes padding) | `selector`: SEL = methodName + `":::"` (checked for all 7258 items) |
| 0x10 | `items`: `BCSequenceItemDef **`, `count` pointers | `params`: array of NSString (NULL = absent) |

`params` is declared `id **` but every slot holds the NSString object itself; the
handlers do `[(id)def->params[i] floatValue]` etc. The parameter count is not
stored anywhere (see 1.2). `methodName` is never read at runtime; dispatch uses
`selector`.

`PocketGodViewController::loadGame @ 0x10006a030` calls
`BCLibrary::processSequenceDefs: @ 0x1000240f4` with the NULL-terminated
`BCSequenceDef *` list at **0x100643a60** (2548 entries, terminator at 0x100648a00).
`processSequenceDefs:` calls `addSequenceDef: @ 0x1000239fc` for each, which stores
the pointer in `BCLibrary.mpSequenceDefMap`, a `std::map<std::string, BCSequenceDef*>`
keyed by the UTF-8 bytes of `name` (converted through a 1023-byte static buffer
`DAT_1007cd818`, `getBytes:maxLength:0x3ff ... encoding:NSUTF8StringEncoding`).
A later duplicate name would overwrite an earlier one; there are no duplicates.

The `BCSequenceDef`s, item-pointer arrays and non-empty parameter arrays are
static data in `__data` (0x100602cd0..0x100643a60). The `BCSequenceItemDef`
structs are in `__common` (from 0x1007e7c20) and are filled at start-up by the
module initializer `FUN_1000d8290` (219,888 bytes of straight-line stores:
`methodName`, `selector`, `params` for every item), so they must be read from the
post-initializer image `tmp/re/data_after_init.bin`.

### 1.2 Parameter arrays and the parameter count

The generator (`outputSource source="dataClasses/SequenceData.mm"` in 1.45.3's
`sequence.xml`) emitted, per sequence and in declaration order:

1. in `__data`: the parameter array of each item that has at least one value,
   then the item-pointer array, then the `BCSequenceDef`;
2. in `__common`: for each item, its parameter array if all its values are NULL
   (zero-fill), immediately followed by its `BCSequenceItemDef`. A sequence with
   no items owns one NULL item-pointer slot there.

Both regions are tiled exactly by these objects (verified: no gaps), so the length
of every parameter array is the distance to the next object. The length is
identical for all items of a method (verified for all 514 methods), i.e. it is
the method's schema size. A method without parameters still gets a one-slot
array holding NULL (C forbids empty arrays): `param_slots` is then 1 and the
method has no parameters. A method whose only slot is used by some item has one
parameter.

Positions are not alphabetical or in XML attribute order; e.g. `setPos` is
`[xPos, zPos, yPos, id, displayGroupId]` and `offsetPos` is `[zPos, xPos, yPos]`.
The order is fixed per method by the generator (probably the iteration order of a
hash container; no tested hash scheme reproduces it), so the names had to be
recovered per position:

* **xml**: the value at this position equals the attribute of that name in the
  aligned 1.45.3 item strictly more often than any other attribute
  (sequences with the same id are aligned item by item on method names);
* **xml-name**: the attribute exists for this method in 1.45.3 but the values do
  not disambiguate the position (e.g. all `"0"`); the position comes from the
  handler code (e.g. `BCLogic::setVel::: @ 0x100139140` stores `params[2]` in x,
  `params[1]` in y, `params[0]` in z);
* **inferred**: the parameter (or method) is new in 1.48.2; the name was chosen
  from what the handler does with it, following the 1.45.3 naming patterns.

Validation (`extract_sequences.py`, printed on every run): of the 2221 sequence
ids present in both versions, 2088 are identical after naming; 6208 of their 6380
items equal a 1.45.3 item; no position has a better-matching rival attribute. The
remaining differences are genuine content changes (new `tomId`/`plagueId`/
`purificationId`/door parameters, renamed animations, other atlas files, help pages).

### 1.3 Parameter values

Values are NSString constants exactly as written in the XML (UTF-8; no empty
strings or surrounding whitespace occur). Handlers convert them on use:

* `floatValue` (most numbers): leading whitespace skipped, then a decimal float
  (`-277.12`, `0.5`), 0.0 if none. All numeric values in the data are plain
  `-?\d+(\.\d+)?`.
* `intValue` (`frames`, `layer`, `count`, `minCount`/`maxCount`): decimal integer,
  0 if none.
* `boolValue` (`ignoreIsland`, `inPlace`, `loop`, `noReset`, `noFlip`): YES when
  the first non-blank character (after an optional sign and leading zeros) is
  `Y`/`y`/`T`/`t` or `1`-`9`. Every flag value in the data is `"1"`.
* Several handlers only test presence (`delay`, `endOfIsland`, `noFlip`, the
  `...Id` alternatives): any non-NULL value counts.
* Lists (`list`, `idList`, `setAnimationBasedOnLevel`) are split with
  `componentsSeparatedByString:@","` (no trimming).
* An absent parameter is `nil`. A handler that does not test for `nil` gets 0 /
  0.0 / NO / nil from the message to nil (e.g. `BCLogic::setOrigin:::` with a
  missing `zPos` sets z to 0, `PygmyLogic::drag:::` sets the origin to (0,0) for
  the 31 `drag` items without parameters).

### 1.4 Statistics

* 2548 sequences, 7258 items; 40 sequences are empty (e.g. `CoconutStopSlide`,
  `AnchorFall`, `PygmyCaughtByBird`); the largest is the boot script `PocketGod`
  (287 items). `Garbage` (47 items) is a sample of methods that is never run.
* 514 methods used: 102 take parameters, 412 take none. 19 more handler
  selectors exist but are not used by any sequence.
* Most used: `setAnimation` 1993, `playSound` 750, `waitForAnimation` 744,
  `runSequence` 516, `triggerEpisodeEvent` 392, `setLogic` 275,
  `addDisplayGroup` 179, `setTextureDef` 166, `stall` 156, `fall` 134, `die` 119.
* Sequence references: every `runSequence`/`chooseSequence`/`push...`/`...BasedOn...`
  target exists except in `Garbage` (`CallTest`, `TumbleSound`,
  `CharlieTumbleSound`, `TempleBabyPreEatFoodAnim`) and the placeholder
  `ShouldntHappen` (`runSequenceBasedOnPersonality`). Of the 1093 distinct
  sequence ids passed as literals to `setBehavior:`/`getSequenceDefById:`/
  `setBattleBehavior:` in code, only `IceHolePygmyFallForward`/`...Backward`
  (`IceHoleLogic::tripIntoIceHole:fallForward: @ 0x100184fa8`, which has no
  callers) are missing.
* Push depth: the deepest static push chain is 3 stack entries (in `Garbage`);
  real scripts use at most 2.

## 2. JSON files

### 2.1 `assets/data/sequences.json`

Array in the order of the list at 0x100643a60:

```json
[{"id": "BlankAnim", "items": [{"method": "setAnimation", "params": {"animation": "BlankAnim"}}]}]
```

| Field | Type | Meaning |
|---|---|---|
| `id` | string | sequence name (unique, case-sensitive lookup key) |
| `items` | array | items in execution order (may be empty) |
| `items[].method` | string | handler method; the ObjC selector is `method + ":::"` |
| `items[].params` | object string→string | parameters present in the item, keyed by schema name, in positional order; **absent parameters are omitted** (never null) |

Keys of `params` are the original attribute names (camelCase); they are map keys,
not struct fields. Rust: `struct SequenceDef { id: String, items: Vec<SequenceItem> }`,
`struct SequenceItem { method: String, params: BTreeMap<String, String> }` (or an
`IndexMap` to keep order). Positional access is through `sequence-methods.json`.

### 2.2 `assets/data/sequence-methods.json`

Array sorted by `method`, one entry per method used by the data or implemented
by any handler (533):

| Field | Type | Meaning |
|---|---|---|
| `method` | string | method name |
| `selector` | string | `method + ":::"` |
| `params` | array | `{index: int, name: string, source: "xml" \| "xml-name" \| "inferred"}` in positional order; empty for methods without parameters |
| `param_slots` | int or null | length of the generated array (1 for methods without parameters); null for selectors no sequence uses |
| `implementations` | array | `{class, category (string or null), symbol, address}` for every `sel:::` method, e.g. `{"class": "PygmyLogic", "category": "Island", "symbol": "PygmyLogic(Island)::fall:::", "address": "0x..."}`, sorted by symbol |
| `uses` | int | number of items using the method |
| `sequence_count` | int | number of sequences using it |

### 2.3 Regenerating

```
tmp/re/venv/bin/python -I tools/re/extract_sequences.py [--report]
```

Inputs default to `tmp/re/PocketGod.arm64`, `tmp/re/data_after_init.bin`,
`tmp/ipas/x1452/Payload/PocketGod.app/sequence.xml` (v1.45.3) and
`tmp/re/decomp/game/`. The run fails if the schema (`PARAM_NAMES` in the script)
does not cover the measured slot counts, if a method has no handler, or if a
name conflicts with the 1.45.3 values. `--report` prints the full comparison.

## 3. Runtime

### 3.1 Looking up a sequence

`BCLibrary::getSequenceDefById: @ 0x100023924` converts the NSString to UTF-8
(max 1023 bytes, static buffer) and does `std::map::operator[]`
(`FUN_100026f00`): **an unknown id returns NULL** (and inserts a NULL entry, which
is harmless). Every caller passes the result straight to `setSequenceDef:` or
`pushSequenceDef:`, so an unknown id *stops* the sequence (set) or is *ignored*
(push). A nil id is never passed by the shipped code paths (the `...BasedOn...`
handlers test for nil first); `runSequence:::` and friends would read an
uninitialised length for nil, so treat a nil id like an unknown one.

### 3.2 BCSequence

```
BCSequence : NSObject
  mSequenceStack[10]  : BCSequenceStackItem {BCSequenceDef *def; int itemIndex}
  mSequenceStackIndex : int      // index of the top entry
  mDelegate           : id<BCSequenceDelegate>  (not retained)
  mDeltaTime          : float    // dt of the current frame
  mbDebug, mbFirstTime, mbIsRunning, mbHibernate, mbHibernateGameFrameRunning : BOOL
```

| Method | Behaviour |
|---|---|
| `+traceGlobalStatus @ 0x1000bfed4` | logs the live-instance counter `DAT_1007d1ee8` (incremented in init, decremented in dealloc) |
| `-initWithSequenceDef:delegate: @ 0x1000bff20` | index = 0; stack[0] = {def, 0}; delegate = arg; mbFirstTime = YES; mbIsRunning = NO; mDeltaTime = 0; if def != NULL: `play` |
| `-setDebug: @ 0x1000c0008` | stores mbDebug (never read) |
| `-deltaTime @ 0x1000c0018` | returns mDeltaTime (handlers use it for timers) |
| `-hibernate @ 0x1000c0028` | if not hibernating: mbHibernate = YES, remember mbIsRunning in mbHibernateGameFrameRunning; if running: mbIsRunning = NO and `BCView removeGameFrameListener:` |
| `-unhibernate @ 0x1000c00b4` | if hibernating: mbHibernate = NO; if it was running: `play` |
| `-setSequenceDef: @ 0x1000c00ec` | **replace**: mbFirstTime = YES; index = 0; stack[0] = {def, 0}; `play` if def != NULL else `stop` |
| `-pushSequenceDef: @ 0x1000c0148` | **call**: ignored if def == NULL. If index != 0 or stack[0].def != NULL: index += 1 (no bound check; the array has 10 entries). mbFirstTime = YES; stack[index] = {def, 0}; `play` |
| `-play @ 0x1000c01c4` | hibernating: mbHibernateGameFrameRunning = YES. Else if not running and stack[index].def != NULL: mbIsRunning = YES, `BCView addGameFrameListener:` (appends to the listener array) |
| `-stop @ 0x1000c0274` | hibernating: mbHibernateGameFrameRunning = NO. Else if running: mbIsRunning = NO, `removeGameFrameListener:` (does not touch the stack) |
| `-runSequence: @ 0x1000c02f8` | executes items of the top entry, see 3.5 |
| `-id @ 0x1000c04fc` | returns `[mDelegate id]` |
| `-gameFrameOrder @ 0x1000c0518` | 10 |
| `-onGameFrame: @ 0x1000c0520` | per-frame driver, see 3.4 |
| `-dealloc @ 0x1000c0654` | `stop`, counter -1 |

`BCSequenceDelegate` (protocol) is `<NSObject>` plus the required `- (id)id`.
Adopters: `BCLogic`, `BonusPackManager`, `DebugScreen`, `GameUiManager`,
`IdolManager`, `MapScreen`, `NewsManager`, `SkinPackManager`, `StoryManager`
(the view controller and the other managers are passed as delegates too). Nothing
else is called on the delegate except the handler selectors.

### 3.3 The frame loop

`BCView::gameLoop @ 0x100177730`, once per display-link tick:

1. `t = CFAbsoluteTimeGetCurrent() - mStartTime` (float); `dt = min(t - mLastTime, 0.06666667)`; `mLastTime = t`.
2. pending atlas loads (`activateTextureAtlasNextFrame:`), accelerometer, touches.
3. `mGameFrameListenerArray` is sorted with `sortUsingFunction:` by
   `gameFrameOrder` ascending (`FUN_1001757bc`), copied into a 200-entry stack
   snapshot, each listener is retained, sent `onGameFrame:dt` in order, released.
4. draw.

Orders: BCSequence 10, BCAnimation 20, PygmyLogic 30, BCLogic / GameUiManager /
BCPreferences 40, StoryManager / PocketGodViewController 50, NoPeeLogic 59,
NoPee map logics / MoronBee / MoronDuck 60, GameServer 70. So **all sequences
step before animations and logic physics** in a frame. Sequences that start
during a frame are not in the snapshot and first run next frame; a sequence
stopped during the frame is still called but returns at once (`mbIsRunning` = NO).
Among equal orders the array order is insertion order (`play` appends;
`stop` removes), assuming the CF sort is stable (see Open questions).

### 3.4 `onGameFrame:` (stack driver)

```
fn on_game_frame(seq, dt):
    if seq.hibernating or not seq.running: return
    loop {
        // run the top sequence; repeat at once if it switched to another def
        loop {
            before = seq.stack[seq.index].def
            seq.run_sequence(dt)
            after = seq.stack[seq.index].def
            if !(before != after && after != NULL) { break }
        }
        top = seq.stack[seq.index]
        done = after == NULL || top.item_index >= before.count   // count of `before`
        if seq.index >= 1 {
            if !done { return }
            seq.index -= 1          // pop; the popped entry is not cleared
            continue                // resume the parent in the same frame
        }
        if done { seq.stack[0] = {NULL, 0}; seq.stop() }
        return
    }
```

A pushed sequence therefore returns to its parent in the same frame it finishes,
and the parent continues with the item after the push (that item was already
counted as complete, 3.5). A finished root sequence clears entry 0 and stops.
(`@ 0x1000c0520`: the inner loop is `b.ne` on `before != after && after != 0`;
`done` compares the top entry's index with `before->count`.)

### 3.5 `runSequence:` (item stepping)

```
fn run_sequence(seq, dt):
    seq.delta_time = dt
    idx = seq.index; entry = seq.stack[idx]
    while entry.item_index < entry.def.count {
        item = entry.def.items[entry.item_index]
        r = dispatch(seq, item)                       // 3.7
        if seq.index != idx {                         // handler pushed (or reset from a deeper level)
            seq.stack[idx].item_index += 1            // the calling item counts as done
            seq.first_time = true
            return
        }
        if seq.stack[idx].def != entry.def { return }                 // replaced (setSequenceDef)
        if seq.stack[idx].item_index != entry.item_index { return }   // restarted / moved
        seq.first_time = false
        match r as u32 {
            0 | 3 => return,                          // not complete: same item next frame
            1 => { top.item_index += 1; seq.first_time = true; return }    // complete, yield
            2 => { top.item_index += 1; seq.first_time = true }            // complete, continue
            _ => {}                                   // re-run the same item now (never used)
        }
        idx = seq.index; entry = seq.stack[idx]       // re-read the top
    }
```

`top` is `stack[seq.index]` re-read after the handler. The switch at
0x1000c0438 is `cmp w0,#3; b.hi` (unsigned) with the jump table 0 → return,
1 → advance+return, 2 → advance+loop, 3 → return.

**Return codes**

| Value | Meaning | Used by |
|---|---|---|
| 0 | not finished; call the same item again next frame with `firstTime = NO` | 230 handlers (waits, timers, movement) |
| 1 | finished; advance, but stop for this frame (next item runs next frame with `firstTime = YES`) | no handler |
| 2 | finished; advance and run the next item immediately (same frame) | 632 handlers |
| 3 | "I replaced the sequence"; returned after `setBehavior:`/`setSequenceDef:`. Identical to 0 in the runner: if the sequence really changed the checks above return first; if it did not, the item runs again next frame with `firstTime = NO` | 115 handlers |
| other (>3 or negative) | not finished, run again immediately in the same frame (infinite loop if it persists) | none |

Six handlers return the result of another handler (the five
`explodeFrom...`/`explodeVampire...` → `PygmyLogic::explode:::`, and
`BCLogic::setTextureDef:::` → `BCLibrary::setTextureDef:::`), both always 2.
The counts above are handlers that can return the value. Consequences:

* Items returning 2 chain: a frame runs items until one returns 0/1/3 or the
  sequence ends. There is no per-frame item limit.
* A switch (runSequence/chooseSequence/setBehavior on the running sequence) or a
  push takes effect in the same frame: `onGameFrame:` immediately runs the new
  top.
* "Restart" detection compares stack index, def pointer and item index only:
  `setSequenceDef:` with the *same* def while executing item 0 of the root is not
  detected, so the handler's return value is applied to the restarted sequence
  (2 would skip its item 0). Shipped data does not rely on this.
* A handler that stops its own sequence (e.g. `stopEarthquake:::` releasing
  `mEarthquakeSequence`) does not end the current `runSequence:` loop: items
  returning 2 keep running this frame; the object stays alive until the end of
  the frame because `gameLoop` retained it.

### 3.6 `firstTime`

`mbFirstTime` is passed as the BOOL argument. It is YES on the first call of an
item reached by: `initWithSequenceDef:`, `setSequenceDef:`, `pushSequenceDef:`,
an advance (return 1 or 2), the break after a push, or a pop (the flag is still
YES from the child's last advance). It is cleared after every handler call that
returns normally (before the switch), so re-runs after 0/3 see NO. Handlers use
it to initialise per-item state (timers, velocities, flags); most per-item state
lives in the *delegate's* ivars, not in the sequence.

### 3.7 Dispatch

For each item (`runSequence:` 0x1000c03a0..0x1000c0400):

1. If `mDelegate != nil` and `[mDelegate respondsToSelector:item->selector]`:
   `objc_msgSend(mDelegate, item->selector, seq, firstTime, item)`. Normal ObjC
   lookup: the most derived class (including categories) wins, e.g. a
   `PygmyLogic` runs `PygmyLogic::fall:::`, a `MonkeyLogic` runs
   `MonkeyLogic::fall:::`, a plain `MultiModelLogic` falls back to
   `BCLogic::setAnimation:::`.
2. Otherwise `[[BCLibrary instance] handleSequenceItem:sel sequence:seq
   firstTime:first sequenceItemDef:item]` (`@ 0x100023df0`), which is just
   `objc_msgSend(BCLibrary, sel, seq, first, item)`.

There is no other fallback: no class overrides `respondsToSelector:`,
`forwardingTargetForSelector:` or `forwardInvocation:`. If BCLibrary does not
implement the selector either, the original throws "unrecognized selector" and
crashes. BCLibrary implements 30 selectors (catalogue below); 21 used methods
exist only there (`runSequence`, `playSound`, `addDisplayGroup`, `stallFrames`,
...), and `setAlpha`, `setAnimation`, `setPos`, `setTextureDef`, `stall`,
`tweenAlpha` exist in both BCLibrary and BCLogic, so their behaviour depends on
the delegate (BCLogic versions act on the logic's own display object).
"Listener" fallbacks are explicit inside individual handlers (e.g.
`DiscoBallLogic::playBackgroundLoop:::` calls its listener), never in the runner.

Static check (scratch analysis, not part of the extractor): for every sequence a
class starts on itself (literal `setBehavior:` on `self` in 111 logic classes,
literal `getSequenceDefById:` in the view controller and the 8 managers) plus
everything reachable through `runSequence`/`push...`/`choose...`/`...BasedOn...`,
each method is implemented by that class, a superclass or BCLibrary. So the
shipped scripts never hit an unrecognised selector on those paths.

Port recommendation: build a dispatch table per delegate kind from
`sequence-methods.json` (`implementations`, honouring the class hierarchy), fall
back to the BCLibrary set, and treat a miss as a fatal script error.

### 3.8 State shared between sequences

Handlers keep their per-item state in the object that handles them:

* **BCLibrary** (one instance) owns `mStallTime`, `mDeltaValue`, `mValue`,
  `mStallFrames`, `mDeltaPos` and `mAnimationDictionary`. Every sequence whose
  delegate does not implement `stall`/`stallFrames`/`tweenAlpha`/`tweenPos`/
  `blink`/`sinAlpha` (all managers and the view controller's sequences) shares
  them: two such sequences stalling in the same frames overwrite each other's
  timer. This is how the original behaves (e.g. `ShowMessage` on the view
  controller's `mMessageSequence` and `IdolManagerCheckAutoUnlockChallenges` on
  IdolManager's `mSequence` both use `BCLibrary::stall:::`).
* **BCLogic** owns `mStallTime`/`mDeltaValue` per logic (`stall`, `tweenAlpha`);
  the timed handlers of the subclasses (`idle`, `walk`, `swim`, `closetIdle`, ...)
  reuse the same inherited `BCLogic::mStallTime`. Only one item of a logic runs
  at a time, so this never overlaps.

### 3.9 Hibernation

`BCLogic::hibernate @ 0x100138874` / `unhibernate @ 0x100138940` forward to the
logic's sequence and animation. A hibernating sequence keeps its stack;
`play`/`stop` while hibernating only record whether it should run after
`unhibernate`.

### 3.10 Who owns sequences

| Owner (delegate) | Sequence ivars | Started by |
|---|---|---|
| every `BCLogic` | `mSequence`, created lazily by `setBehavior:` | `setBehavior:` from code, `BCLibrary::setBehavior:::`, the `...BasedOn...` handlers |
| `PocketGodViewController` | `mSequence` (created in `loadGame` with `PocketGod`, the boot script), `mMiscSequence`, `mTheRunsSequence`, `mKonkeyDongSequence`, `mMoronPestsSequence`, `mNoPeeSequence`, `mBattleOfTheGodsSequence`, `mWeatherSequence`, `mMessageSequence` (`ShowMessage`), `mCelebrationSequence` (new object per `Celebration`), `mEarthquakeSequence` (new object per `EarthquakeStart`, `TheRunsShakeBackground...`); `mMuteSequence` is never created | its own code |
| `TempleDoorLogic` | `mBattleSequence` (`setBattleBehavior:`) in addition to `mSequence` | battle code |
| `StoryManager` | `mSequence`, `mStorySequence` | story player |
| `BonusPackManager`, `ChallengeManager`, `DebugScreen`, `GameUiManager`, `IdolManager`, `MapScreen`, `NewsManager`, `SkinPackManager` | `mSequence` (created with NULL in `init`) | their own code (`GameUiManager::setBehavior:` is a wrapper) |

### 3.11 Logic objects

`BCLogic::setBehavior: @ 0x1001385f8`:

```
def = BCLibrary.getSequenceDefById(name)          // NULL if unknown
if self.mSequence: self.mSequence.setSequenceDef(def)   // replace, restart at item 0
else:
    self.mSequence = BCSequence(def, delegate=self)     // plays if def != NULL
    if self.hibernating:
        self.mbHibernateGameFrameRunning = YES          // note: the *logic's* flag
        self.mSequence.hibernate()
```

`initWithDisplayObject:` leaves `mSequence` nil; `shutdown @ 0x1001389e8` and
`dealloc @ 0x100138a74` send `stop` (dealloc also releases it).
`OutHouseLogic::setBehavior:` only adds a log line. Subclasses switch behaviour
with `setBehavior:` from code and, inside handlers, return 3 afterwards.

There is no separate "run/push/choose" API on BCLogic: the sequence-control
items are handlers that operate on the *running sequence object* passed in:

| Item | Effect on the running BCSequence |
|---|---|
| `runSequence id` | `setSequenceDef:` (replace whole stack) |
| `pushSequence id` | `pushSequenceDef:` (call; returns to the next item afterwards) |
| `chooseSequence list` / `choosePushSequence list` | pick `list[arc4random() % n]` (n = number of comma-separated entries; an empty string yields one empty entry), then replace / push |
| `runSequenceBasedOnPersonality` | `setBehavior:` of the chosen id on the *PygmyLogic* (same object as the delegate, so equivalent to replace) |
| `pushSequenceBasedOnPersonality`, `pushSequenceBasedOnOpponentWeaponType`, `pushSequenceBasedOnOpponentBattleType` | push the chosen id (nothing if nil) |
| `runSequenceIfInXenuRoom id` | `setBehavior:` only when the island location state is 17, no apocalypse is active and `id` is present (returns 3), else 2 |
| `checkPygmyDialog` | throttled `setBehavior: alternateSequenceId` (3) or continue (2) |
| `setBehavior displayGroupId behavior id` (BCLibrary) | `setBehavior:` on *another* object's logic |

Apart from `runSequenceIfInXenuRoom` and `checkPygmyDialog` (3 when they switch)
these return 2. Because the runner checks the stack after the call, a replace or
push takes effect immediately.

### 3.12 Parameter selection tables

`T` = `PygmyLogic.mTimeTemplePygmyType` (0 normal, 1 old fart, 2 baby, 3 ginger,
4 fast; `ApocalypseTimeTemplePlatformLogic::onTransformComplete:::` creates them
in that order), `D` = `SpriteLogic.mApocalypseDiseaseType` (1 plague, 2
purification), `P` = `PygmyLogic.mPersonaPersonality` (1 Charlie, 2 Tom). `a ?? b`
means "a if non-nil else b".

**`PygmyLogic::runSequenceBasedOnPersonality::: @ 0x100132c9c`** and
**`pushSequenceBasedOnPersonality::: @ 0x100132a64`** (same algorithm, names refer
to each method's own positions):

```
r = nil
T==2: r = (D==2 ? babyPurificationId : D==1 ? babyPlagueId : nil) ?? babyId
T==1: r = (D==2 ? oldFartPurificationId : D==1 ? oldFartPlagueId : nil) ?? oldFartId
T==3: r = (D==2 ? gingerPurificationId : D==1 ? gingerPlagueId : nil) ?? gingerId
T==4: r = (D==2 ? fastPurificationId : D==1 ? fastPlagueId : nil) ?? fastId
r = r ?? (D==2 ? purificationId : D==1 ? plagueId : nil)
r = r ?? (P==2 ? tomId : P==1 ? charlieId : nil)
r = r ?? defaultId
if r == nil: return 2              // nothing happens
run: setBehavior:r / push: pushSequenceDef:getSequenceDefById(r); return 2
```

**`PygmyLogic::setAnimationBasedOnPygmyType::: @ 0x100132898`**: the same chain
with `oldFartPurification/oldFartPlague/oldFart`, `babyPurification/babyPlague/
baby`, `gingerPurification/gingerPlague/ginger`, `fastPurification/fastPlague/fast`,
then `purification`/`plague`, then `normal`; no personality step; plays the
result even if nil. Positions 15/16 (`normalPurification`, `normalPlague`) are
never read; only `TemplePygmyDrag` fills them (with the values `PygmyDrag` puts in
`purification`/`plague`), so in that sequence a purified/sick normal pygmy falls
through to `normal` (original data quirk, keep it).

**`ApocalypseTempleDoorLogic::setAnimationBasedOnPygmyType::: @ 0x100136420`**,
**`ApocalypseTimeTemplePlatformLogic::... @ 0x100182884`**: by the held pygmy's
type 0 normal, 1 oldFart, 2 baby, 3 ginger, 4 fast; `normal` when no pygmy; no
fallback when the chosen value is nil.

**`ApocalypseTempleDoorLogic::setAnimationBasedOnDoorType::: @ 0x100136504`** by
`mApocalypseDoorType` (set in `BCLibrary(Logic)::createLogicClasses`):

| type | door | param |
|---|---|---|
| 0 | ApocalypseIsland TempleDoor | `entrance` |
| 1 | ApocalypseIsland TemplePortal | `portal` |
| 2 / 3 | TimeTemple Door1 / Door2 | `timeSideLeft` / `timeSideRight` |
| 4 / 5 | LocustTemple Door1 / Door2, Door3 | `locustSideLeft` / `locustSideRight` |
| 6 / 7 | DanceTemple Door1 / Door2 | `danceSideLeft` / `danceSideRight` |
| 8 / 9 | PlagueTemple Door1 / Door2 | `plagueSideLeft` / `plagueSideRight` |
| 10 / 11 | PurificationTemple Door1 / (none created) | `purificationSideLeft` / `purificationSideRight` |
| 12 | ClosetTemple Door1 | `closetSideLeft` |
| 13 | XenuTemple Door1 | `xenuSideLeft` |

**Gem / slot / pedestal / location tables**

| Handler | Selector variable | Mapping |
|---|---|---|
| `ApocalypseGemLogic::setAnimationBasedOnGemType::: @ 0x10000f9e0`, `ApocalypseGemPedestalLogic::... @ 0x100069ab4` | `mApocalypseGemType` | 0,1 → gem1; 2 → gem2; ... 7 → gem7; other → nil |
| `GodIdolLogic::setAnimationBasedOnIdolSlot::: @ 0x10011a1a8` | `mSlot` | 0 → slot1, 1 → slot2, 2 → slot3, 3 → slot4 |
| `IdolPedestalLogic::play/stopSoundBasedOnPedestalIndex::: @ 0x10013f4d0 / 0x10013f598` | `mPedestalIndex` | 1..4 → soundId1..soundId4; other → nil |
| `TsunamiLogic::setAnimationBasedOnUnderwaterLocation::: @ 0x10000cc3c` / `...LastUnderwaterLocation @ 0x10000ccc8` | (last) island location state | 9 and `coralReef` present → `coralReef`, else `ruins` |
| `ApocalypseCloudLogic::setAnimationBasedOnType::: @ 0x1000c9df4` | D | 2 → `purification`, 1 → `plague`, else nothing |
| `ApocalypseTempleFountainLogic::setAnimationBasedOnType::: @ 0x1001a1bd4` | D | 2 → `purification`, 1 → `plague`, else plays nil |
| `PygmyLogic(BattleOfTheGods)::pushSequenceBasedOnOpponentWeaponType::: @ 0x100122c80` | opponent episode node +0x78 (weapon) | 1 → `energyId`, else `swordId` |
| `...pushSequenceBasedOnOpponentBattleType::: @ 0x100122d10` | node +0x74 (type), +0x78 (weapon) | type 2: weapon 1 → `fastEnergyId`, weapon 0 → `fastSwordId`; type 0 + weapon 0 → `strongSwordId`; anything else → nothing |
| `NoPeeTreasureLogic::setAnimationBasedOnLevel::: @ 0x10008e014` | `mTreasureLevel` | `list[min(level, n-1)]` |

### 3.13 Timers and tweens

Random durations. Handlers with `minTime`/`maxTime` (and `stall`, `fireworks`,
`shakeBackground`, `lavaRain`, `icicleRain`, the x positions of spawned objects)
use the same integer formula with a scale `k`:

```
lo = trunc_i32(min * k)          // fcvtzs on the float product (single precision)
m  = (1 - lo + trunc_i32(max * k)) as u32
r  = arc4random()                // u32
v  = (if m == 0 { r } else { r % m }) + lo     // u32 arithmetic, then as i32
value = (v as i32) as f32 / k    // inclusive range [min, max] in 1/k steps
```

`k` = 1000 for `stall` (BCLibrary/BCLogic), `fireworks`, `shakeBackground`,
`lavaRain` delays; 100 for the `idle`/`walk`/`swim`/... families and `icicleRain`
delays; 10 for spawn x positions. Integer counts (`minCount`/`maxCount`) use
`(r % (1 - min + max)) + min`.

**Stall** (`BCLibrary::stall::: @ 0x100024d00`, `BCLogic::stall::: @ 0x1001395cc`):

```
if first: T = minTime present ? R(minTime, maxTime, 1000) : float(time)
if T <= 0: return 2
T -= seq.deltaTime; return 0
```

The check precedes the subtraction, so `stall time=1` completes on the first frame
whose remaining time is already <= 0 (one frame after it ran out); `time` <= 0
completes immediately. A NaN never completes.

**stallFrames** (`BCLibrary::stallFrames::: @ 0x100024e10`): on first
`N = intValue(frames)`; each call `v = N; N -= 1; return v < 2 ? 2 : 0` (completes
on the N-th call; immediately for N <= 1).

**tweenAlpha** (BCLibrary `@ 0x100024f68`, BCLogic `@ 0x1001396dc`): on first
`T = time; delta = (target - alpha_now) / T`; each call
`alpha = T > 0 ? target - T*delta : target`, then `T -= dt`; returns 2 on the call
where `T <= 0` (alpha = target exactly), else 0. The first call re-sets the
current alpha.

**tweenPos** (`BCLibrary @ 0x1000253d0`, only used by `Garbage`): target = given
coordinates (current ones for absent params); on first `T = time`,
`v = (target - pos)/T`; each call `pos = target - v*dt` (sic: dt, not the
remaining time, so it jumps next to the target), 2 when `T <= 0`, `T -= dt`.

**blink** (`BCLibrary @ 0x10002562c`): every `time` seconds toggles the object's
alpha between 0 and 1 (alpha 0 → 1, anything else → 0); never completes.

**sinAlpha** (`BCLibrary @ 0x100025118`): `mid = (min+max)/2`, `amp = (max-min)/2`;
on first `phase = asinf((alpha - mid)/amp)`; each call
`alpha = mid + amp * sinf(speed * dt + phase)` (the phase is not accumulated, so
alpha barely moves); never completes. `time` is unused.

**Spring drag** (the `drag:::` handlers): `d = pos - touch` (touch converted with
`viewToDisplayObject:`), `L = |d|`; if `L != 0`: `mSpringForce = -k*d - c*mPosVel`
with `k, c` = 100, 8 (default) or 50, 6 (where noted); objects that rotate set
`zRot = atan2(-d.y, -d.x)*180/pi - 90` when `d.y != 0`. They never complete; the
logic leaves the drag behaviour from its touch code.

### 3.14 `setLogic` (logic factory)

`PocketGodViewController(Game)::setLogic::: @ 0x10015c11c` (275 uses, mostly in
the boot script `PocketGod`):

1. If `manager` is `ChallengeManager`, `SkinPackManager`, `BonusPackManager`,
   `StoryManager`, `NewsManager` or `ChumBucket`, the item is forwarded unchanged
   to that object's `setLogic:(BCSequenceItemDef *)` (`ChallengeManager @
   0x10001a238`, `SkinPackManager @ 0x1000611d4`, `BonusPackManager @ 0x1000cc1c0`,
   `StoryManager @ 0x100050080`, `NewsManager @ 0x100056ba8`, `ChumBucketLogic @
   0x10011b7dc`), which apply the same pattern to their own logic ids (e.g.
   ChumBucket: `LaserButtonLogic`, `LaserSharkLogic`, `LaserTargetLogic`).
2. Otherwise the display object `displayObjectId` of display group
   `displayGroupId` gets a logic chosen by `logicId` (unknown ids do nothing):
   `BCButtonLogic` for the `...ButtonLogic` ids (listener = view controller, kept
   in the matching ivar and UI array), the class of the same name for the gameplay
   ids (`CoconutLogic`, `WoodLogic`, `BonFireLogic`, `SpearLogic`, `FishLogic`,
   `SwordFishLogic`, `FishingPoleLogic`, `BoulderLogic`, `TRexEggLogic`,
   `TRexLogic`, `SpiderLogic`, `SpiderTreeLogic`, `OutHouseLogic`,
   `IceMonsterLogic`, `GlacierLogic`, `IglooLogic`, `IceTreeLogic`, `IceHoleLogic`,
   `ZombieGraveLogic`, `ApeLogic`, `ApeAltarLogic`, `ApeBongoLogic`,
   `ApeWenchLogic`, `ApeTorchLogic`, `MonkeyLogic`, `BananaTreeLogic`,
   `BananaLogic`, `CrackHoleLogic`, `TheRunsPlatformLogic`,
   `TheRunsStalactiteLogic`, `OctoStatueLogic`, `KongStatueLogic`,
   `KonkeyDongLogic`, `SpearStatueLogic`, `AnchorLogic`, `TileDrainLogic`,
   `TilePlugLogic`, `ChumBucketLogic`, `HealthBarLogic`, `AntHillLogic`,
   `MagnifyingGlassLogic`, `SharkLogic`, `BirdLogic`, `VolcanoLogic`, `SunLogic`,
   `MoonLogic`, `MapPaperLogic`, `SpriteLogic`, `MultiModelLogic`), and
   `SchoolOfFishLogic`/`UnderwaterBlob1Logic`/`UnderwaterBlob2Logic` →
   `UnderwaterSceneryLogic`, `StarsLogic` → `MultiModelLogic`,
   `Front/BehindStormCloudLogic` → `StormCloudLogic` and `Front/BehindOceanLogic` →
   `OceanLogic` (`setFrontStormCloud:posVelScale:` / `setFrontOcean:posVelScale:`
   with YES, 1.0 for front and NO, 0.5 for behind), `Front/BehindTsunamiLogic` →
   `TsunamiLogic` (`setFrontLayer:` YES / NO). Gameplay
   logics are stored in their view-controller ivar or array with the view
   controller as listener; `SpriteLogic`, `MultiModelLogic`, `TheRunsPlatformLogic`,
   `TheRunsStalactiteLogic` and `KonkeyDongLogic` are only created
   (`initWithDisplayObject:`); every created logic is added to `mLogicArray`.
   `PygmyStatsButtonLogic` creates `count` buttons named
   `[NSString stringWithFormat:displayObjectId, i]` (`Pygmy%dStatsButton`), and
   `StarsLogic` removes its display object from the group. The exact id → ivar
   wiring belongs to the view-controller spec.

## 4. Parameter schema

Positional names per method (`†` = name from 1.45.3, position from the handler; `*` = inferred, see 1.2). The 412 other used methods take no parameters (one NULL slot).

| method | slots | positional params | uses |
|---|---|---|---|
| `addDisplayGroup` | 3 | 0:parentId, 1:id, 2:markerId | 179 |
| `battleFall` | 1 | 0:ignoreIsland | 2 |
| `blink` | 3 | 0:displayGroupId, 1:time, 2:id | 1 |
| `blinkLightning` | 1 | 0:time | 1 |
| `charlieMonkeyIdle` | 2 | 0:minTime, 1:maxTime | 1 |
| `charlieMonkeyWalk` | 2 | 0:minTime, 1:maxTime | 1 |
| `checkForSunState` | 1 | 0:loop | 5 |
| `checkKillPygmies` | 2 | 0:minTime, 1:maxTime | 1 |
| `checkPygmyDialog` | 2 | 0:alternateSequenceId, 1:id | 32 |
| `chooseAnimation` | 1 | 0:list | 3 |
| `choosePushSequence` | 1 | 0:list* | 1 |
| `chooseRainbowAnimation` | 1 | 0:list | 2 |
| `chooseSequence` | 1 | 0:list | 32 |
| `closetIdle` | 2 | 0:minTime*, 1:maxTime* | 3 |
| `closetWalk` | 2 | 0:minTime*, 1:maxTime* | 3 |
| `countDownTimer` | 1 | 0:time | 1 |
| `disableButtonLayer` | 1 | 0:layer | 6 |
| `drag` | 2 | 0:yOrigin, 1:xOrigin | 40 |
| `drawDisplayGroup` | 1 | 0:id | 33 |
| `drawTallScreenOverlayDisplayGroup` | 1 | 0:id* | 1 |
| `drawWideScreenOverlayDisplayGroup` | 1 | 0:id* | 1 |
| `enableButtonLayer` | 2 | 0:layer, 1:displayGroup | 41 |
| `fall` | 1 | 0:ignoreIsland | 134 |
| `fireworks` | 2 | 0:minTime, 1:maxTime | 1 |
| `ghostFlyIdle` | 2 | 0:minTime, 1:maxTime | 1 |
| `halfPygmyIdle` | 2 | 0:minTime, 1:maxTime | 2 |
| `halfPygmyWalk` | 2 | 0:minTime, 1:maxTime | 2 |
| `hideObjects` | 2 | 0:displayGroupId, 1:idList | 1 |
| `hover` | 2 | 0:minTime, 1:maxTime | 2 |
| `icicleRain` | 4 | 0:minTime, 1:minCount, 2:maxTime, 3:maxCount | 1 |
| `idle` | 2 | 0:minTime, 1:maxTime | 17 |
| `lavaRain` | 4 | 0:minTime, 1:minCount, 2:maxTime, 3:maxCount | 4 |
| `loadTextureAtlas` | 4 | 0:ext, 1:file, 2:id, 3:delay | 97 |
| `loadTextureAtlasChannel` | 1 | 0:channel | 3 |
| `multiModelEvent` | 1 | 0:eventId | 30 |
| `offsetPos` | 3 | 0:zPos†, 1:xPos†, 2:yPos | 2 |
| `openWordBalloon` | 1 | 0:message | 1 |
| `panicRun` | 3 | 0:minTime, 1:totalTime, 2:maxTime | 3 |
| `panicSwim` | 2 | 0:minTime, 1:maxTime | 1 |
| `playBackgroundLoop` | 1 | 0:soundId* | 25 |
| `playSound` | 1 | 0:soundId | 750 |
| `playSoundBasedOnPedestalIndex` | 4 | 0:soundId2, 1:soundId3, 2:soundId1, 3:soundId4 | 1 |
| `possessedByThetanIdle` | 2 | 0:minTime*, 1:maxTime* | 1 |
| `possessedByThetanWalk` | 2 | 0:minTime*, 1:maxTime* | 1 |
| `pushSequence` | 1 | 0:id | 1 |
| `pushSequenceBasedOnOpponentBattleType` | 3 | 0:fastEnergyId, 1:fastSwordId†, 2:strongSwordId† | 3 |
| `pushSequenceBasedOnOpponentWeaponType` | 2 | 0:energyId, 1:swordId | 1 |
| `pushSequenceBasedOnPersonality` | 17 | 0:defaultId, 1:charlieId, 2:tomId*, 3:babyPlagueId*, 4:babyPurificationId*, 5:oldFartPlagueId*, 6:purificationId*, 7:gingerPlagueId*, 8:plagueId*, 9:fastPurificationId*, 10:gingerPurificationId*, 11:fastPlagueId*, 12:oldFartPurificationId*, 13:fastId†, 14:babyId, 15:oldFartId, 16:gingerId | 34 |
| `refreshSkinPackOption` | 3 | 0:skinPackId, 1:skinOptionId, 2:skinObjectId | 8 |
| `removeDisplayGroup` | 1 | 0:id | 40 |
| `run` | 2 | 0:minTime, 1:maxTime | 2 |
| `runSequence` | 1 | 0:id | 516 |
| `runSequenceBasedOnPersonality` | 17 | 0:defaultId, 1:babyId, 2:charlieId, 3:tomId*, 4:fastId, 5:oldFartId, 6:gingerId, 7:babyPurificationId*, 8:babyPlagueId*, 9:oldFartPlagueId*, 10:gingerPlagueId*, 11:fastPurificationId*, 12:gingerPurificationId*, 13:fastPlagueId*, 14:oldFartPurificationId*, 15:purificationId*, 16:plagueId* | 14 |
| `runSequenceIfInXenuRoom` | 1 | 0:id* | 2 |
| `setAlpha` | 3 | 0:alpha, 1:displayGroupId, 2:id | 7 |
| `setAnimation` | 5 | 0:displayGroupId, 1:animation, 2:stateId, 3:id, 4:percent | 1993 |
| `setAnimationBasedOnDoorType` | 14 | 0:purificationSideLeft*, 1:entrance, 2:purificationSideRight*, 3:locustSideLeft, 4:plagueSideRight*, 5:timeSideLeft, 6:timeSideRight, 7:locustSideRight, 8:danceSideLeft, 9:portal*, 10:xenuSideLeft*, 11:closetSideLeft*, 12:danceSideRight*, 13:plagueSideLeft* | 4 |
| `setAnimationBasedOnGemType` | 7 | 0:gem3, 1:gem2, 2:gem1, 3:gem7, 4:gem6, 5:gem5, 6:gem4 | 5 |
| `setAnimationBasedOnIdolSlot` | 4 | 0:slot1, 1:slot3, 2:slot2, 3:slot4 | 1 |
| `setAnimationBasedOnLastUnderwaterLocation` | 2 | 0:coralReef, 1:ruins | 1 |
| `setAnimationBasedOnLevel` | 1 | 0:list* | 1 |
| `setAnimationBasedOnPygmyType` | 17 | 0:oldFartPurification*, 1:oldFartPlague*, 2:normal, 3:babyPlague*, 4:gingerPlague*, 5:purification*, 6:fastPurification*, 7:plague*, 8:gingerPurification*, 9:babyPurification*, 10:fastPlague*, 11:fast, 12:ginger, 13:baby, 14:oldFart, 15:normalPurification*, 16:normalPlague* | 56 |
| `setAnimationBasedOnType` | 2 | 0:plague*, 1:purification* | 17 |
| `setAnimationBasedOnUnderwaterLocation` | 2 | 0:coralReef, 1:ruins | 8 |
| `setApocalypseCloudDragOffset` | 2 | 0:dragXOffset*, 1:dragYOffset* | 7 |
| `setBehavior` | 3 | 0:displayGroupId, 1:behavior, 2:id | 12 |
| `setCloudGradientFade` | 1 | 0:fade | 2 |
| `setLadderParams` | 1 | 0:height | 4 |
| `setLayerAnimation` | 2 | 0:layer, 1:animation | 45 |
| `setLogic` | 5 | 0:displayGroupId, 1:logicId, 2:displayObjectId, 3:manager, 4:count† | 275 |
| `setOrigin` | 4 | 0:zPos, 1:xPos, 2:yPos, 3:inPlace | 72 |
| `setPivot` | 3 | 0:zPos, 1:xPos, 2:yPos | 1 |
| `setPlatformParams` | 1 | 0:width | 10 |
| `setPos` | 5 | 0:xPos, 1:zPos†, 2:yPos, 3:id, 4:displayGroupId | 10 |
| `setRot` | 3 | 0:zRot†, 1:xRot†, 2:yRot† | 1 |
| `setScale` | 3 | 0:xScale, 1:zScale†, 2:yScale† | 5 |
| `setSoundAlias` | 2 | 0:soundId†, 1:aliasId† | 1 |
| `setTextureDef` | 3 | 0:texture, 1:displayGroupId, 2:id | 166 |
| `setVel` | 3 | 0:zVel†, 1:yVel†, 2:xVel | 9 |
| `shakeBackground` | 2 | 0:minTime, 1:maxTime | 4 |
| `showMessage` | 4 | 0:zPos, 1:xPos, 2:texture, 3:yPos | 3 |
| `showObjects` | 2 | 0:displayGroupId, 1:idList | 1 |
| `sinAlpha` | 6 | 0:min, 1:max†, 2:id, 3:displayGroupId, 4:time†, 5:speed | 1 |
| `slither` | 2 | 0:minTime, 1:maxTime | 1 |
| `stall` | 3 | 0:time, 1:minTime, 2:maxTime | 156 |
| `stallFrames` | 1 | 0:frames | 26 |
| `stopSound` | 1 | 0:soundId | 49 |
| `stopSoundBasedOnPedestalIndex` | 4 | 0:soundId2, 1:soundId3, 2:soundId1, 3:soundId4 | 2 |
| `sunWalk` | 2 | 0:minTime, 1:maxTime | 2 |
| `swim` | 3 | 0:minTime, 1:maxTime, 2:noReset | 16 |
| `templePygmyIdle` | 2 | 0:minTime, 1:maxTime | 14 |
| `templePygmyWalk` | 3 | 0:minTime, 1:xPosVel, 2:maxTime | 19 |
| `trace` | 1 | 0:message | 3 |
| `triggerEpisodeEvent` | 1 | 0:event | 392 |
| `tweenAlpha` | 4 | 0:displayGroupId, 1:time, 2:target, 3:id | 19 |
| `tweenPos` | 6 | 0:targetZPos†, 1:targetXPos†, 2:id, 3:displayGroupId, 4:targetYPos†, 5:time | 1 |
| `vampireRun` | 2 | 0:minTime, 1:maxTime | 1 |
| `waitForNight` | 2 | 0:minTime†, 1:maxTime† | 2 |
| `waitToCrap` | 2 | 0:minTime, 1:maxTime | 2 |
| `walk` | 4 | 0:minTime, 1:maxTime, 2:direction, 3:endOfIsland | 16 |
| `zombieIdle` | 2 | 0:minTime, 1:maxTime | 2 |
| `zombieWalk` | 3 | 0:minTime, 1:maxTime, 2:noFlip | 6 |

## 5. Method catalogue

Grouped by implementing class (category in parentheses); `BCLibrary` handles every item the delegate does not respond to. Format: `method` (positional params) `address` [uses of the method in the data, all implementations together]: behaviour; the trailing number is the return value (0 = not finished, 2 = finished and continue, 3 = replaced the sequence). Item parameters are written as `name`.

### BCLibrary

- `addDisplayGroup` (parentId, id, markerId) `0x1000245f4` [179 uses]: Adds display group `id` (nil if absent) to: display group `parentId` if given, else display marker `markerId` if given, else the active display group; 2.
- `blink` (displayGroupId, time, id) `0x10002562c` [1 use]: Every `time` seconds toggles alpha (0 <-> 1) of object `id` in `displayGroupId` ("" = active display group; absent = no target); never completes (0).
- `choosePushSequence` (list) `0x100024344` [1 use]: Splits `list` on ",", picks arc4random() % count, then like pushSequence; 2.
- `chooseSequence` (list) `0x100024270` [32 uses]: Splits `list` on ",", picks arc4random() % count, then like runSequence; 2.
- `drawDisplayGroup` (id) `0x1000244c8` [33 uses]: BCView setActiveDisplayGroup: display group `id`; 2.
- `drawTallScreenOverlayDisplayGroup` (id) `0x100024590` [1 use]: BCView setTallScreenOverlayDisplayGroup: display group `id`; 2.
- `drawWideScreenOverlayDisplayGroup` (id) `0x10002452c` [1 use]: BCView setWideScreenOverlayDisplayGroup: display group `id`; 2.
- `hideObjects` (displayGroupId, idList) `0x1000246f8` [1 use]: For each comma-separated name in `idList`: hide that child of display group `displayGroupId` ("" = active display group; absent = no target); 2.
- `loadTextureAtlas` (ext, file, id, delay) `0x100024418` [97 uses]: Loads texture atlas `id` from file `file`.`ext` now (FUN_100038d68), or, when `delay` is present, queues it with BCView activateTextureAtlasNextFrame:fileName:ext: (at most 10 pending, extra requests dropped; the queue is drained last-in-first-out at the start of the next gameLoop); 2.
- `mute` () `0x100024ca0` [unused]: BCSound mute:YES; 2.
- `playSound` (soundId) `0x100024ad4` [750 uses]: Plays sound `soundId`; 2.
- `pushSequence` (id) `0x100024198` [1 use]: pushSequenceDef: getSequenceDefById(id) on the running sequence; 2.
- `removeDisplayGroup` (id) `0x1000246bc` [40 uses]: Removes display group `id` from its parent; 2.
- `runSequence` (id) `0x100024204` [516 uses]: setSequenceDef: getSequenceDefById(id) on the running sequence (replaces the whole stack); 2.
- `setAlpha` (alpha, displayGroupId, id) `0x100024e74` [7 uses]: Sets alpha of object `id` in display group `displayGroupId` ("" = active display group; absent = no target); 2.
- `setAnimation` (displayGroupId, animation, stateId, id, percent) `0x1000257a8` [1993 uses]: Plays `animation` on a library-owned BCAnimation keyed by `id` (created on first use); applies `percent` via setFramePercent: if present; 2.
- `setBehavior` (displayGroupId, behavior, id) `0x10002598c` [12 uses]: Calls setBehavior: `behavior` on the logic of object `id` in display group `displayGroupId`; 2.
- `setPos` (xPos, zPos, yPos, id, displayGroupId) `0x1000252cc` [10 uses]: Sets x/y/z (each only if present) of object `id` in display group `displayGroupId` (active group if absent); 2.
- `setSoundAlias` (soundId, aliasId) `0x100024a80` [1 use]: Registers sound `soundId` under the alias `aliasId` in the sound dictionary (ignored if the sound does not exist); 2.
- `setTextureDef` (texture, displayGroupId, id) `0x1000258b4` [166 uses]: Sets texture def `texture` on object `id` of display group `displayGroupId` (active group if absent); 2.
- `showObjects` (displayGroupId, idList) `0x1000248bc` [1 use]: Same as hideObjects but calls show; 2.
- `sinAlpha` (min, max, id, displayGroupId, time, speed) `0x100025118` [1 use]: Oscillates alpha of `id` in `displayGroupId` ("" = active display group; absent = no target) between `min` and `max` at `speed`; phase is not accumulated (original bug); never completes (0).
- `stall` (time, minTime, maxTime) `0x100024d00` [156 uses]: Timer using the library-wide mStallTime (shared by every sequence that falls back to BCLibrary). See Stall.
- `stallFrames` (frames) `0x100024e10` [26 uses]: Frame counter using the library-wide mStallFrames; completes on the `frames`-th call (first call if frames <= 1).
- `stopAllSounds` () `0x100024c7c` [unused]: Stops every sound in the sound dictionary; 2.
- `stopSound` (soundId) `0x100024b10` [49 uses]: Stops sound `soundId`; 2.
- `trace` (message) `0x100024148` [3 uses]: Logs a fixed format string (debug); 2.
- `tweenAlpha` (displayGroupId, time, target, id) `0x100024f68` [19 uses]: Linear alpha tween of object `id` in `displayGroupId` ("" = active display group; absent = no target) to `target` over `time` s (library-wide timer); 0 until done, then 2.
- `tweenPos` (targetZPos, targetXPos, id, displayGroupId, targetYPos, time) `0x1000253d0` [1 use]: Position tween of `id` toward targetX/Y/ZPos over `time` (original bug: pos = target - velocity*deltaTime each frame); 0 until done, then 2.
- `unmute` () `0x100024cd0` [unused]: BCSound mute:NO; 2.

### BCLogic

- `chooseAnimation` (list) `0x100138e30` [3 uses]: Plays a random entry of the comma-separated `list`; 2.
- `offsetPos` (zPos, xPos, yPos) `0x1001393c4` [2 uses]: Adds x/y/z (each if present) to the display object position; 2.
- `removeFromDisplayGroup` () `0x100138d7c` [12 uses]: Removes the logic's display object from its display group; 2.
- `rotate` () `0x100139550` [unused]: zRot += 2 per call; completes (2) when it passes 360 (zRot reset to 0), else 0.
- `setAlpha` (alpha, displayGroupId, id) `0x1001397f8` [7 uses]: Sets the display object's alpha; 2.
- `setAnimation` (displayGroupId, animation, stateId, id, percent) `0x100138dac` [1993 uses]: Plays `animation` on the logic's BCAnimation; setFramePercent: `percent` if present; 2.
- `setOrigin` (zPos, xPos, yPos, inPlace) `0x100139004` [72 uses]: Sets origin x/y/z (setOrigin?Pos: or, if `inPlace` boolValue, setOrigin?PosInPlace:); missing values read as 0; 2.
- `setPivot` (zPos, xPos, yPos) `0x10013947c` [1 use]: Sets pivot x/y/z (missing values read as 0); 2.
- `setPos` (xPos, zPos, yPos, id, displayGroupId) `0x100139324` [10 uses]: Sets display object position x/y/z for each present param; 2.
- `setRot` (zRot, xRot, yRot) `0x100139284` [1 use]: Sets display object rotation x/y/z (degrees) for each present param; 2.
- `setScale` (xScale, zScale, yScale) `0x1001391e4` [5 uses]: Sets display object scale x/y/z for each present param; 2.
- `setTextureDef` (texture, displayGroupId, id) `0x100138edc` [166 uses]: If `id` is present forwards to BCLibrary setTextureDef:::; else stops the animation and sets texture def `texture` on the display object; 2.
- `setVel` (zVel, yVel, xVel) `0x100139140` [9 uses]: Sets mPosVel x/y/z for each present param; 2.
- `stall` (time, minTime, maxTime) `0x1001395cc` [156 uses]: Per-logic timer (mStallTime). See Stall.
- `tweenAlpha` (displayGroupId, time, target, id) `0x1001396dc` [19 uses]: Linear alpha tween of the display object to `target` over `time` s (per-logic timer); 0 until done, then 2.
- `waitForAnimation` () `0x100138fcc` [744 uses]: Completes (2) when the logic's animation isAnimationComplete, else 0.

### SpriteLogic

- `setApocalypseCloudDragOffset` (dragXOffset, dragYOffset) `0x1001995c0` [7 uses]: If attached to an apocalypse cloud: setDragXOffset:`dragXOffset` dragYOffset:`dragYOffset`; 2.
- `triggerEpisodeEvent` (event) `0x100199570` [392 uses]: IdolManager triggerEpisodeEvent: `event`; 2.

### MultiModelLogic

- `die` () `0x10000bc38` [119 uses]: Removes the display object, shuts down and releases the logic; 2.
- `multiModelEvent` (eventId) `0x10000bca0` [30 uses]: Listener onMultiModelEvent:eventId: `eventId`; 2.
- `triggerEpisodeEvent` (event) `0x10000bce0` [392 uses]: IdolManager triggerEpisodeEvent: `event`; 2.

### BCScreenLogic

- `closeScreenComplete` () `0x10001d278` [9 uses]: closeScreenNow; 2.
- `disableButtonLayer` (layer) `0x10001d208` [6 uses]: Removes the button display group attached to layer `layer`; 2.
- `disableGameLayer` () `0x10001cec4` [14 uses]: Removes the screen display group, copies the active group's pos/rot to it and makes it the active display group; 2.
- `enableButtonLayer` (layer, displayGroup) `0x10001d0d0` [41 uses]: Attaches button display group `displayGroup` to screen layer `layer` (replacing the previous one, copying the layer's x/y) inside the screen display group; 2.
- `enableGameLayer` () `0x10001cfa4` [1 use]: Moves the screen display group into mActiveDisplayGroup (pos x/y and z-rot reset to 0) and makes that the active display group; 2.
- `setLayerAnimation` (layer, animation) `0x10001d054` [45 uses]: Plays `animation` on screen layer number `layer`; 2.

### AdBlockerLogic

- `waitForChartboostAdHide` () `0x1001a9718` [1 use]: Waits (0) while the listener reports a Chartboost ad visible; then makes the display object unselectable and completes (2).

### AnchorLogic

- `die` () `0x100055068` [119 uses]: Removes the display object from its display group; 2.
- `drag` (yOrigin, xOrigin) `0x10005509c` [40 uses]: First time: stop spin, gravity on. Spring drag toward the touch point (k 50, damping 6, no rotation); never completes (0).

### AntHillLogic

- `die` () `0x10013e7f0` [119 uses]: Removes the display object, clears mbDirecting and mActiveTouch; 2.
- `directAnts` () `0x10013e3b8` [1 use]: First time (if ants inactive): spawns 5 ant multimodels (AntLogic, listener = hill) right after the hill in its display group. Every frame spreads the ants evenly between the hill x and the touch x (clamped to island min/max). Never completes (0).
- `removeAnts` () `0x10013e394` [unused]: Calls removeAntLogics; 2.

### AntLogic

- `carryBone` () `0x1001a6e40` [1 use]: First time: face away from mStartXPos and walk at +/-60. Completes (2) once within +/-25 of mStartXPos (snaps to the edge, stops); else 0.
- `die` () `0x1001a73fc` [119 uses]: Removes the carried bone model and tells the listener onAntDie:; 2.
- `lookForPygmy` () `0x1001a7198` [1 use]: First time: walk toward mEndXPos at mWalkVel. Turns around once at mEndXPos, completes (2) when back past mStartXPos; else 0.
- `moveToTarget` () `0x1001a732c` [1 use]: Faces/moves toward mEndXPos at mWalkVel every frame; never completes (0).
- `swarm` () `0x1001a6f60` [1 use]: First time: face mSwarmXPos, velocity +/-60. On passing it: non-lead ants complete (2); the lead ant stops, spawns the bone model at its position and setBehavior: AntCarryBone (3). Else 0.

### ApeAltarLogic

- `apeGrabPygmyComplete` () `0x10017e540` [1 use]: Retracts the ape wench, clears tweenable channel 1, tells the pygmy grabbedFromApeAltar and forgets it; 2.
- `breakPygmyComplete` () `0x10017e4f4` [2 uses]: Clears mbBreakPygmy; 2.
- `pygmyFallInsideComplete` () `0x10017e508` [5 uses]: Sets the pygmy-head logic's behaviour to ApeAltarPygmyHeadIdle; 2.

### ApeBongoLogic

- `apeHitComplete` () `0x10013758c` [4 uses]: Clears mbBeingSmashed; if a pygmy is on the bongo: apeSmashOnApeBongo, clear channel 1, forget it; 2.
- `pygmyFallInsideComplete` () `0x100137540` [5 uses]: Enables drumming (mbCanDrum); if mbPlaying, setBehavior: ApeBongoPygmyStart; 2.

### ApeLogic

- `apeExitAfterIdle` () `0x100089198` [1 use]: Hides the ape and setBehavior: ApeExitIsland; 3.
- `checkEatPygmy` () `0x1000890c0` [2 uses]: setBehavior: ApeEatPygmy if the altar has a pygmy for the ape (records whether it has the plague), else ApeCrushBongoAndWenchPygmies; 3.
- `checkForLickChops` () `0x1000891d0` [6 uses]: If no pygmy is available for the ape and the altar is not available: setBehavior: ApeLickChops (3); else 2.
- `checkForPlaguePygmy` () `0x100088f6c` [5 uses]: If the eaten pygmy had the plague: hides the ape, sets global logic 1 to ApeVomitInit (inserted before DragDisplayMarker, at the ape position) and setBehavior: ApeEatPlaguePygmy (3); else 2.
- `enterIslandComplete` () `0x100088f14` [2 uses]: Clears mbEnteringIsland; if an exit was queued, hides the ape and setBehavior: ApeExitIsland; 2.
- `lickChopsWaitForPygmy` () `0x100089278` [1 use]: If the altar becomes available: setBehavior: ApeCrushBongoAndWenchPygmies (3); completes (2) when a pygmy is available for the ape; else 0.

### ApeWenchLogic

- `apeHitComplete` () `0x10008a898` [4 uses]: Clears mbBeingSmashed; if a pygmy hangs on the wench: apeSmashOnApeWench, clear channel 1, forget it; 2.
- `pygmyFallInsideComplete` () `0x10008a848` [5 uses]: Allows cranking the wench again; 2.
- `retractComplete` () `0x10008a860` [2 uses]: Allows cranking, resets mApeWenchRot to 0; 2.
- `struggleComplete` () `0x10008a884` [1 use]: Clears mbStruggling; 2.

### ApocalypseCalendarLogic

- `calendarCloseComplete` () `0x1000d7d88` [1 use]: Clears mbClosing and tells the listener endApocalypseComplete; 2.
- `calendarOpenComplete` () `0x1000d7d70` [1 use]: Sets mbCompletelyOpen; 2.
- `elevateGems` () `0x1000d7b88` [1 use]: Same timing as lightTorches but calls elevateGem on each pedestal; 2 after the last one.
- `hideClockHands` () `0x1000d7df0` [1 use]: Hides ClockCenter and the three calendar dial display objects; 2.
- `lightTorches` () `0x1000d7a94` [1 use]: First time: pedestal index 0. Every 0.5 s (mNextPedestalTime countdown by deltaTime) lights the next gem pedestal's torch; completes (2) after the last pedestal, else 0.
- `showClockHands` () `0x1000d7ed4` [1 use]: Shows ClockCenter and the three calendar dial display objects; 2.
- `tweenClockHands` () `0x1000d7dc4` [1 use]: Sets mbTweenClockHands; 2.
- `unlightTorches` () `0x1000d7c7c` [1 use]: Same timing as lightTorches but calls unlightTorch on each pedestal; 2 after the last one.
- `untweenClockHands` () `0x1000d7ddc` [1 use]: Clears mbTweenClockHands; 2.

### ApocalypseCloudLogic

- `die` () `0x1000c9c60` [119 uses]: Removes the display object, shuts the logic down and releases it; 2.
- `setAnimationBasedOnType` (plague, purification) `0x1000c9df4` [17 uses]: Plays `purification` when mApocalypseDiseaseType==2, `plague` when 1 (nothing for other types or nil); 2.

### ApocalypseGemLogic

- `checkRespawn` () `0x10000fe9c` [1 use]: No-op; 2.
- `die` () `0x10000fea4` [119 uses]: Stops, removes the display object, ends a drag (listener onApocalypseGemDragComplete:), ejects from its pedestal, resets state and tells the listener onApocalypseGemDie:; 2.
- `drag` (yOrigin, xOrigin) `0x10000fb04` [40 uses]: First time: gravity on, floor collision off. Spring drag (k 100, damping 8) with rotation toward the drag direction; never completes (0).
- `fall` (ignoreIsland) `0x10000fcb4` [134 uses]: First time: floor collision on unless ignoreIsland; gravity on. Lands on a pedestal (3), on the island (2), or below y=40 in an enabled ocean: setBehavior: ApocalypseGemSplash + episode event ThrowGemAway (3); else 0.
- `resetLastPos` () `0x10000fab4` [5 uses]: mLastPos = current position; 2.
- `setAnimationBasedOnGemType` (gem3, gem2, gem1, gem7, gem6, gem5, gem4) `0x10000f9e0` [5 uses]: Plays the param for mApocalypseGemType: 0/1 gem1, 2 gem2, 3 gem3, 4 gem4, 5 gem5, 6 gem6, 7 gem7 (other: nil); 2.
- `slide` () `0x10000f9b4` [20 uses]: First time: enable island floor collision and gravity; never completes (0).

### ApocalypseGemPedestalLogic

- `setAnimationBasedOnGemType` (gem3, gem2, gem1, gem7, gem6, gem5, gem4) `0x100069ab4` [5 uses]: Same mapping as ApocalypseGemLogic, using the pedestal's mApocalypseGemType; 2.
- `unlightTorchComplete` () `0x100069a3c` [1 use]: Clears mbUnlightingTorch; if a gem is held: clear channel 2 now and eject it; 2.

### ApocalypseRoomFaderLogic

- `apocalypseFaderComplete` () `0x1001784c0` [1 use]: Deactivates and removes the display object; 2.

### ApocalypseTempleDoorLogic

- `onPygmyFallInsideComplete` () `0x100136350` [16 uses]: Global logic 440 activate: (the door's own mIslandLocationState; the listener's islandLocationState is queried but unused), clears channel 1 now, setBehavior: ApocalypseTempleDoorEjectPygmy, forgets the pygmy; 2.
- `setAnimationBasedOnDoorType` (purificationSideLeft, entrance, purificationSideRight, locustSideLeft, plagueSideRight, timeSideLeft, timeSideRight, locustSideRight, danceSideLeft, portal, xenuSideLeft, closetSideLeft, danceSideRight, plagueSideLeft) `0x100136504` [4 uses]: Plays by mApocalypseDoorType: 0 entrance, 1 portal, 2 timeSideLeft, 3 timeSideRight, 4 locustSideLeft, 5 locustSideRight, 6 danceSideLeft, 7 danceSideRight, 8 plagueSideLeft, 9 plagueSideRight, 10 purificationSideLeft, 11 purificationSideRight, 12 closetSideLeft, 13 xenuSideLeft; 2.
- `setAnimationBasedOnPygmyType` (oldFartPurification, oldFartPlague, normal, babyPlague, gingerPlague, purification, fastPurification, plague, gingerPurification, babyPurification, fastPlague, fast, ginger, baby, oldFart, normalPurification, normalPlague) `0x100136420` [56 uses]: Plays by the held pygmy's timeTemplePygmyType: 0 normal, 1 oldFart, 2 baby, 3 ginger, 4 fast (normal when no pygmy); 2.

### ApocalypseTempleFountainLogic

- `setAnimationBasedOnType` (plague, purification) `0x1001a1bd4` [17 uses]: Plays `purification` when mApocalypseDiseaseType==2, `plague` when 1, else nil; 2.

### ApocalypseTimeTemplePlatformLogic

- `onTransformComplete` () `0x100182968` [1 use]: Triggers CreateOldFartPygmy/CreateBabyPygmy/CreateGingerPygmy/CreateFastPygmy for mActiveStage 0..3 and sets the pygmy's timeTemplePygmyType to stage+1 (toggles back to 0 if it already had it); 2.
- `setAnimationBasedOnPygmyType` (oldFartPurification, oldFartPlague, normal, babyPlague, gingerPlague, purification, fastPurification, plague, gingerPurification, babyPurification, fastPlague, fast, ginger, baby, oldFart, normalPurification, normalPlague) `0x100182884` [56 uses]: Same mapping as ApocalypseTempleDoorLogic (held pygmy's timeTemplePygmyType); 2.

### BananaLogic

- `die` () `0x10001c544` [119 uses]: reset, deactivate, remove display object; 2.
- `drag` (yOrigin, xOrigin) `0x10001c2e4` [40 uses]: First time: gravity on, floor collision off. Spring drag (k 100, damping 8) with rotation; never completes (0).
- `fall` (ignoreIsland) `0x10001c494` [134 uses]: First time: floor collision + gravity on, falling. Completes (2) on island floor collision, else 0.
- `fallFromHurricane` () `0x10001c50c` [9 uses]: First time: floor collision off, gravity on, falling; never completes (0).
- `slide` () `0x10001c2b8` [20 uses]: First time: floor collision and gravity on; never completes (0).

### BananaTreeLogic

- `die` () `0x100136ba4` [119 uses]: Clears touch, removes the display object; boots a monkey sitting in the tree (clear channel 2); 2.

### BatLogic

- `die` () `0x100116c2c` [119 uses]: Stops, clears touch, ignores gravity and tells the listener onBatDie:; 2.
- `fall` (ignoreIsland) `0x100116ac0` [134 uses]: First time: gravity on. When z >= -277.12, the ocean is enabled and y+originY < 10: lands in the water (y clamped), setBehavior: BatSplash (3); else 0.
- `flyFromFrontToIsland` () `0x100116004` [1 use]: First time: velocity to reach (350,250,-277.12) in 2 s. Any biteable pygmy -> setBehavior: BatFlyToPygmyFromBack; completes (2) when z < -277.12 (clamped), else 0.
- `flyFromMoonToFront` () `0x100115e58` [1 use]: First time: alpha 0, z=-700, velocity to reach (240,200,-177) in 3 s. Alpha eases to 1 (+10%/frame); completes (2) once z > -177 (stops), else 0.
- `flyOffScreen` () `0x1001169f8` [1 use]: First time: velocity (200,100) (x negated when x < 240). Completes (2) once x < -80 or x > 560 (stops); else 0.
- `flyToPygmy` () `0x100116618` [2 uses]: Flies toward the biteable pygmy that is frontmost in the display group (moves just in front of it); with none left: stop, insert after DragDisplayMarker, setBehavior: BatIdle (2). Within 1 unit of the target: stop and bitByBat: (2); else 0.
- `hover` (minTime, maxTime) `0x100116264` [2 uses]: First time: random target x 100..380, y 160..300 (0.1 steps); duration = distance*0.1. Lerps toward it; any biteable pygmy -> BatFlyToPygmy (2); completes (2) within +/-3 of target x; else 0.

### BattleAlertLogic

- `onBattleAlertClose` () `0x10002806c` [1 use]: Drops the first queued match; deactivate when none remain, else activateNextMatch; 0.

### BirdLogic

- `carryFishOffScreen` () `0x100093428` [1 use]: First time: velocity (150,150) (x mirrored by facing). Off screen: caughtByBirdComplete: on the fish, clear channel 2 (2); else 0.
- `carryPygmyOffScreen` () `0x10009374c` [2 uses]: As carryFishOffScreen for the carried pygmy (channel 1); 2 when off screen.
- `die` () `0x1000939a4` [119 uses]: Removes the bird, drops carried fish/pygmy (droppedByBird:), ends a drag, resets all state; re-enters the island if birds are enabled and no story is active; 2.
- `diveForFish` () `0x1000931fc` [1 use]: Steers toward the fish at speed 150. Fish being eaten: target its eater, setBehavior: BirdCatchPygmy (3); fish no longer catchable: setBehavior: BirdAbortDive (3); reached (fish above beak y-36): carries it on channel 2 (2); else 0.
- `diveForPygmy` () `0x10009357c` [1 use]: Steers toward mTargetPygmyLogic at speed 150; not catchable: setBehavior: BirdAbortDive (3); reached: carries it on channel 1 (2); else 0.
- `drag` (yOrigin, xOrigin) `0x100093ec0` [40 uses]: First time: gravity on, floor collision off, moveToDragLayer. Spring drag (k 100, damping 8) with rotation; never completes (0).
- `dropAndFlyOffScreen` () `0x10009389c` [1 use]: First time: velocity (150,150) mirrored by facing; 2 when off screen (stops), else 0.
- `fall` (ignoreIsland) `0x100093b94` [134 uses]: First time: floor collision unless ignoreIsland, gravity on. Lands on the island (2) or, in an enabled ocean below y=20, splashes (y=20): BirdSplash or BirdCookedSplash (3); else 0.
- `fallFromHurricane` () `0x100093d44` [9 uses]: Falls without floor collision; splash height 86 when z < -277.12 else 20: BirdSplash/BirdCookedSplash (3); else 0.
- `fly` () `0x100092f04` [4 uses]: First time: y = 220. If a fish can be caught within 40 units ahead: dive velocity 150 toward it, setBehavior: BirdCatchFood (3). If poo hit a pygmy: setBehavior: BirdTurnAround (3). Off screen (maxX < -40 going left / minX > 520 going right): listener onBirdFlyOffScreen:, 2. Else 0.

### BirdPooLogic

- `die` () `0x10017a028` [119 uses]: Marks in water, stops falling, ignores gravity, listener onBirdPooDie:; 2.
- `fall` (ignoreIsland) `0x100179e88` [134 uses]: Falls; on island floor: listener onBirdPooMissPygmy:, 2; in an enabled ocean below y=30: onBirdPooMissPygmy:, setBehavior: BirdPooSplash (3); else 0.

### BonFireLogic

- `die` () `0x10013b338` [119 uses]: Clears touch, removes the display object; 2.

### BonusPackManager

- `loadLocationTextureAtlas` () `0x1000cba3c` [13 uses]: Listener onLoadLocationTextureAtlas; 2.
- `updateDetailPanel` () `0x1000cb988` [3 uses]: Calls updateDetailPanel; 2.
- `updateInfoPanel` () `0x1000cb9d0` [4 uses]: Calls updateInfoPanel; 2.
- `updateListPanel` () `0x1000cb964` [5 uses]: Calls updateListPanel; 2.
- `updateNoKeyChainPanel` () `0x1000cb9f4` [2 uses]: Calls updateNoKeyChainPanel; 2.
- `updateUnlockDialog` () `0x1000cba18` [1 use]: Calls updateUnlockDialog; 2.
- `updateUnlockPanel` () `0x1000cb9ac` [1 use]: Calls updateUnlockPanel; 2.

### BoulderLogic

- `die` () `0x10001e990` [119 uses]: Removes the boulder, reports bowled pygmies (onPygmiesBowled:), releases a crushed pygmy, ends a drag, resets state, re-enters the island if boulders are enabled; 2.
- `drag` (yOrigin, xOrigin) `0x10001e664` [40 uses]: First time: stop spin, gravity on, floor collision off, insert after DragDisplayMarker. Spring drag (k 50, damping 6), no rotation; never completes (0).
- `fall` (ignoreIsland) `0x10001e818` [134 uses]: First time: floor collision unless ignoreIsland. On the island: 2; below y=50: setBehavior: BoulderSplash (3); else 0.
- `slide` () `0x10001e550` [20 uses]: First time: floor collision + gravity on. Below y=50: lands in the water, setBehavior: BoulderSplash (3); else 0.

### ChainSpearLogic

- `onShootComplete` () `0x1000626d8` [1 use]: clearSkeweredPygmies; 2.

### ChallengeManager

- `loadLocationTextureAtlas` () `0x100018574` [13 uses]: Listener onLoadLocationTextureAtlas; 2.
- `showChallengeConfirmSendScreen` () `0x100018448` [1 use]: Marks the confirm-send screen active, listener onChallengeDialogOpen, updates it; 2.
- `showChallengeResultsScreen` () `0x100018500` [1 use]: Marks the results screen active, listener onChallengeDialogOpen, updates it; 2.
- `submitChallenge` () `0x100017330` [2 uses]: First time: stores ChallengerGameScore, ends the turn, releases the active match. 0 while submitting; then shows "Submit Challenge Fail" on error, 2.
- `submitChallengeResult` () `0x100017440` [2 uses]: First time: stores RecipientGameScore, ends the match. 0 while submitting; then shows "Submit Challenge Result Fail" on error, 2.
- `updateChallengeCreatePanel` () `0x1000183e0` [3 uses]: Calls updateChallengeCreatePanel; 2.
- `updateChallengeStartRespondScreen` () `0x1000184bc` [1 use]: updateLoadingScreen + updateChallengeStartRespondScreen; 2.
- `updateChallengeStartSendScreen` () `0x100018404` [2 uses]: updateLoadingScreen + updateChallengeStartSendScreen; 2.
- `waitForSaveMatchDataComplete` () `0x100017304` [2 uses]: 0 while mbSavingMatchData; then clears mbStartingMatchFromViewController, 2.

### ChumBucketLogic

- `die` () `0x10011b424` [119 uses]: Removes the display object; 2.
- `shootLaser` () `0x10011b3f4` [1 use]: Tells the laser shark to shootLaser; 2.

### ChumLogic

- `die` () `0x10008ea94` [119 uses]: Makes the holding pygmy dropChum:, removes the chum, listener onChumDie:; 2.
- `drag` (yOrigin, xOrigin) `0x10008e8dc` [40 uses]: First time: stop spin, gravity on. Spring drag (k 100, damping 8) with rotation; never completes (0).

### ClosetTempleThetanMeterLogic

- `thetanCreationComplete` () `0x10019e858` [1 use]: If a pygmy is held: clear channel 1 now, explodeFromThetanBirth, forget it; 2.

### CloudLogic

- `drift` () `0x1000d5288` [3 uses]: Wraps the cloud horizontally around the 480-wide screen (re-randomising when not transitioning out; listener onCloudDie: when transitioning out); handles transition-in; never completes (0).

### CoconutLogic

- `die` () `0x100189ee4` [119 uses]: reset, texture Coconut_06, listener onCoconutDie:; 2.
- `drag` (yOrigin, xOrigin) `0x100189a90` [40 uses]: First time: gravity on, floor collision off, moveToDragLayer. Spring drag with rotation; never completes (0).
- `fall` (ignoreIsland) `0x100189c54` [134 uses]: First time: floor collision + gravity on. On the island: 2; below y=50: setBehavior: CoconutSplash (3); else 0.
- `fallFromHurricane` () `0x100189da0` [9 uses]: Falls without floor collision; splashes below y 86 when z < -277.12, else below y 50 (table DAT_10029db08), clamping y: CoconutSplash (3); else 0.
- `slide` () `0x100189990` [20 uses]: First time: floor collision + gravity on. Below y=50: setBehavior: CoconutSplash (3); else 0.

### DanceBalloonLogic

- `die` () `0x10016a6cc` [119 uses]: Deactivates, ends a drag (listener onDanceBalloonDragComplete:), removes the display object; 2.
- `drag` (yOrigin, xOrigin) `0x10016a520` [40 uses]: Spring drag (k 50, damping 6) with rotation +90 deg (balloon hangs below the finger); never completes (0).
- `fall` (ignoreIsland) `0x10016a6b0` [134 uses]: First time: mbFalling; never completes (0).

### DanceDanceExecutionLogic

- `playBackgroundLoop` (soundId) `0x10017ceec` [25 uses]: Listener playBackgroundLoop: nil (ignores the soundId param); 2.
- `pygmyFallInsideComplete` () `0x10017cf8c` [5 uses]: Listener onDanceDanceExecutionStart:; 2.
- `smashPygmyComplete` () `0x10017cf20` [1 use]: Clears channel 1 now, pygmy danceDanceExecutedComplete:, forgets it; 2.
- `stopBackgroundLoop` () `0x10017cebc` [6 uses]: Listener stopBackgroundLoop; 2.

### DanceDanceExecutionScreenLogic

- `startDanceDanceExecution` () `0x1000360f4` [1 use]: Resets the segment beat count and activates the mini-game; 2.

### DebugScreen

- `exitApp` () `0x100009050` [1 use]: No-op; 2.
- `loadLocationTextureAtlas` () `0x10000907c` [13 uses]: Listener onLoadLocationTextureAtlas; 2.
- `updateDebugItems` () `0x100009058` [2 uses]: Calls updateDebugItems; 2.
- `waitForDataUpdate` () `0x100009008` [1 use]: 0 while the listener's challengePreferences waitingToSave; then 2.

### DiscoBallLogic

- `playBackgroundLoop` (soundId) `0x10011bbcc` [25 uses]: Listener playBackgroundLoop: nil; 2.
- `stopBackgroundLoop` () `0x10011bb9c` [6 uses]: Listener stopBackgroundLoop; 2.

### DreamBubbleLogic

- `deactivateComplete` () `0x10019f58c` [1 use]: Deactivates, removes the display object; 2.
- `die` () `0x10019f3fc` [119 uses]: No-op; 2.
- `transitionComplete` () `0x10019f540` [1 use]: Ends the transition, deactivates, removes the display object; 2.

### EpisodeAlertLogic

- `checkHiddenAlert` () `0x1001a431c` [1 use]: If the episode icon is not shown: setBehavior: EpisodeAlertFadeOut; 2.
- `onAlertFadeOutComplete` () `0x1001a4358` [1 use]: If the episode icon is not shown: remove the display object, setBehavior: EpisodeAlertDeactivate; 2.

### FishLogic

- `die` () `0x100013040` [119 uses]: Deactivates, removes the fish, ends a drag, resets state; re-enters the island if fish are enabled; 2.
- `drag` (yOrigin, xOrigin) `0x1000127c8` [40 uses]: First time: gravity on, floor collision off, moveToDragLayer. Spring drag with rotation; never completes (0).
- `fall` (ignoreIsland) `0x100012d18` [134 uses]: Falls; lands in an igloo / ice hole (3), on the island (2), or below y=20 in an enabled ocean: setBehavior: FishSplash (3); else 0.
- `fallFromHurricane` () `0x100012ed4` [9 uses]: Falls without floor collision; splash height 86 (z < -277.12) or 20: FishSplash (3); else 0.
- `slide` () `0x100012404` [20 uses]: First time: floor collision + gravity on. In an enabled ocean below y=20: setBehavior: FishSplash (3); else 0.
- `swim` (minTime, maxTime, noReset) `0x10001252c` [16 uses]: First time: no gravity/floor; random direction at +/-50 (x reset to 520 or -40 unless `noReset`). Ocean disabled: setBehavior: FishIdle (3). Within 16 of an unhooked bob: hookFish:, setBehavior: FishGetHooked (3). Off screen: stop, 2. Else 0.

### FishingBobLogic

- `drag` (yOrigin, xOrigin) `0x1000d6934` [40 uses]: Bob x follows mDragPos (drag x velocity = delta*20); never completes (0).
- `moveTowardPygmy` () `0x1000d675c` [1 use]: Moves the bob toward x = pygmy x +/-40 (by facing); on arrival: catch the hooked fish, pygmy catchFish, pole catchFish, setBehavior: FishingBobCatchFish (2); else 0.

### FishingPoleLogic

- `die` () `0x1001a3794` [119 uses]: Stops, z=-277.12, removes the pole and its parts, ends a drag, resets state, re-enters the island if fish are enabled; 2.
- `drag` (yOrigin, xOrigin) `0x1001a31cc` [40 uses]: First time: gravity on, floor collision off. Spring drag with rotation; never completes (0).
- `fall` (ignoreIsland) `0x1001a337c` [134 uses]: First time: unit scale, floor collision unless ignoreIsland, gravity on. z-rot eases toward -12.62. Below y=20 off-island: setBehavior: FishingPoleSplash (3). On the island: if a bonfire with a log is within [-70,+35] in x: setBehavior: FishingPoleFallIgnoreIsland (3); else 2. Else 0.
- `fallFromHurricane` () `0x1001a3650` [9 uses]: Falls without floor collision; splash at y 72 (z < -277.12) or 20: FishingPoleSplash (3); else 0.

### GoGoCageLogic

- `pygmyFallInsideComplete` () `0x10008f9ac` [5 uses]: Debug log only; 2.

### GodIdolLogic

- `die` () `0x10011a5d8` [119 uses]: Removes the idol, ends a drag, ejects it from its pedestal, resets state, re-enters the island if idols are enabled and no story is active; 2.
- `drag` (yOrigin, xOrigin) `0x10011a298` [40 uses]: First time: gravity on, floor collision off. Spring drag (k 50, damping 6) with a quarter-strength tilt; never completes (0).
- `fall` (ignoreIsland) `0x10011a450` [134 uses]: Falls (ignoreIsland disables floor collision); lands on an idol pedestal (3), on the island (2), or below y=10: setBehavior: GodIdolSplash (3); else 0.
- `resetLastPos` () `0x10011a248` [5 uses]: mLastPos = current position; 2.
- `setAnimationBasedOnIdolSlot` (slot1, slot3, slot2, slot4) `0x10011a1a8` [1 use]: Plays by mSlot: 0 slot1, 1 slot2, 2 slot3, 3 slot4 (else nil); 2.

### HealthBarLogic

- `hideHealthBar` () `0x1000d712c` [1 use]: Waits (0) until y == -30; then if inactive releases the listener and removes the bar; 2.
- `showHealthBar` () `0x1000d7114` [3 uses]: mTargetYPos = 7; 2.

### HurricaneLogic

- `die` () `0x100115308` [119 uses]: Removes all rain and particle display objects, listener onHurricaneDie:; 2.

### IceHoleLogic

- `beheadPygmyComplete` () `0x100185bfc` [1 use]: Resets tentacle aim, squid active flag and grabbed pygmy; 2.
- `eatPygmyComplete` () `0x100185c28` [4 uses]: Resets tentacle aim, squid active flag and grabbed pygmy; 2.
- `iceHoleActivateComplete` () `0x100185710` [1 use]: Clears channel 1 now, melting pygmy meltIceHoleComplete:, marks the hole melted; 2.
- `iceHoleWaitForSwordFish` () `0x1001857a0` [2 uses]: When the swordfish canBeCaught: setBehavior: IceHoleFishingCatchFish (2); else 0.
- `icicleRain` (minTime, minCount, maxTime, maxCount) `0x100185cd0` [1 use]: First time: icicle count = rand int in [minCount, maxCount]. Each eruption waits a random [minTime, maxTime] (0.01 s steps), then spawns an Icicle multimodel (IcicleLogic) at a random island x (0.1 steps), y 400, z -277.12 after LavaDisplayMarker; completes (2) when the last icicle is spawned, else 0.
- `squidExitComplete` () `0x100185c54` [1 use]: Resets tentacle aim and squid active flag; 2.
- `tugOfWar` () `0x100185b74` [1 use]: When the pygmy is no longer dragged: it is eatenByIceSquid:, setBehavior: IceHoleEatPygmy (3); else 0.
- `waitToGrabPygmy` () `0x100185960` [1 use]: First time: grab/exit timers 0.5 s. A dragged pygmy within |dx|<35, 100<dy<150 for 0.5 s: setBehavior: IceHoleIceSquidGrabPygmy (3). No pygmy near (|dx|<50, 0<dy<200) for 0.5 s: setBehavior: IceHoleIceSquidExit (3). Else 0.
- `waitToShowIceSquid` () `0x100185818` [7 uses]: If the ice squid is enabled and a dragged pygmy that can be grabbed is within |dx|<50, 0<dy<200: activates the squid, setBehavior: IceHoleIceSquidEnter (3); else 0.

### IceMonsterLogic

- `chooseIdleBehavior` () `0x100183f0c` [1 use]: After 2 snowball throws: start recharging (IceMonsterRechargeStart). Else 50%: IceMonsterThrowSnowBalls (faces the screen centre). Else walks: clamps x to [50,430] and picks IceMonsterWalk or IceMonsterWalkTurn by facing; 3.
- `chooseRechargeBehavior` () `0x100184140` [2 uses]: 5th call ends recharging (IceMonsterRechargeExit); else clamps x to [50,430] and picks IceMonsterRecharge or IceMonsterRechargeTurn so it heads toward the centre; 3.
- `die` () `0x1001848ec` [119 uses]: Removes the display object; 2.
- `disableMouthCollision` () `0x100183ef8` [6 uses]: Clears mbMouthCollisionActive; 2.
- `enableMouthCollision` () `0x100183ee0` [3 uses]: Sets mbMouthCollisionActive; 2.
- `endTransition` () `0x100183e7c` [15 uses]: Clears mbInTransition; 2.
- `rechargeWalk` () `0x100184738` [1 use]: Regains 0.01 health per frame (health bar updated). On the 4th recharge walks to a random x in [140,340] (0.01 steps). Completes (2) at that x or when x leaves [50,430]; else 0.
- `resetPosition` () `0x100183e90` [1 use]: x = mStartXPos; 2.
- `startTransition` () `0x100183e64` [10 uses]: Sets mbInTransition; 2.
- `walk` (minTime, maxTime, direction, endOfIsland) `0x1001845dc` [16 uses]: First time: random duration [minTime, maxTime] (0.01 s). Completes (2) when the time is up or x leaves [50,430]; else 0.

### IcicleLogic

- `die` () `0x1000ca614` [119 uses]: Listener onIcicleDie:; 2.
- `fall` (ignoreIsland) `0x1000ca288` [134 uses]: Keeps the icicle on screen; hits a pygmy within 64 units (hitByIcicle:) -> setBehavior: IcicleDie. Below y=86 over the island: stops at y=86 (2); else 0.

### IdolManager

- `checkAutoUnlockChallenges` () `0x10002c4bc` [1 use]: If a challenge cannot be triggered yet: restarts its own sequence IdolManagerCheckAutoUnlockChallenges (3); else checkAutoUnlockChallenges (2).
- `waitForScreenClosed` () `0x10002c490` [1 use]: Completes (2) when canTriggerChallenge, else 0.

### IdolPedestalLogic

- `playSoundBasedOnPedestalIndex` (soundId2, soundId3, soundId1, soundId4) `0x10013f4d0` [1 use]: Plays the sound for mPedestalIndex 1..4 = soundId1..soundId4; 2.
- `stopSoundBasedOnPedestalIndex` (soundId2, soundId3, soundId1, soundId4) `0x10013f598` [2 uses]: Stops the sound for mPedestalIndex 1..4 = soundId1..soundId4; 2.

### IdolScreenLogic

- `loadLocationTextureAtlas` () `0x100196238` [13 uses]: Listener onLoadLocationTextureAtlas; 2.
- `showBattleRealmEffectResults` () `0x100195448` [1 use]: First time: shows the "BEATS" BattleEffectsStart message. Every 0.5 s shows the next realm-effect result (player's, then enemy's) with BattleNegative/PositiveRealmEffectResult messages; when both lists are done: BattleEffectsEnd on the result message logic, listener checkBattleRealmEffectDeath, 2; else 0.

### IglooLogic

- `checkForFishDinner` () `0x1000c3a68` [3 uses]: If a pygmy rests inside and a fish is inside: setBehavior: IglooEatFish; 2.
- `checkForFishOnly` () `0x1000c3ab4` [1 use]: If only a fish is inside: setBehavior: IglooFishOnly; 2.
- `eatFishComplete` () `0x1000c3b00` [2 uses]: Fish finishEatingInIgloo, forget it; 2.
- `endTransition` () `0x1000c3a54` [15 uses]: Clears mbInTransition; 2.
- `startTransition` () `0x1000c3a3c` [10 uses]: Sets mbInTransition; 2.

### JellyfishLogic

- `createCircleJerk` () `0x10013c498` [1 use]: Places global logic 98's display object at mCircleJerkPos (z -277.12) before CoralReefObjectDisplayMarker and sets it to JellyfishCircleJerkInit; 2.
- `die` () `0x10013cd84` [119 uses]: Removes the display object; 2.
- `drag` (yOrigin, xOrigin) `0x10013c87c` [40 uses]: Spring drag; switches between JellyfishDrag and JellyfishDragMove behaviours depending on drag angle (5 degree threshold); never completes (0).
- `enterIslandComplete` () `0x10013c688` [2 uses]: Gravity on, screen collision on; 2.
- `onJellyfishAttackComplete` () `0x10013ccf0` [1 use]: Gravity on, selectable, setBehavior: JellyfishIdle, pygmy jellyfishAttackComplete:, forget it; 2.
- `swim` (minTime, maxTime, noReset) `0x10013c6b0` [16 uses]: First time: random duration [minTime, maxTime] (0.01 s), y velocity 50, x velocity +/-10 (random, forced inward near x 50/430). Completes (2) when time is up (stops); else 0.
- `swimToTarget` () `0x10013c59c` [2 uses]: First time: 1 s timer. Eases 10%/frame toward mTargetPos; completes (2) when the timer runs out, else 0.

### KongStatueLogic

- `startKonkeyDongKidnap` () `0x100187c18` [1 use]: Moves the Konkey Dong logic before DragDisplayMarker and starts kidnapPygmy:; if no pygmy can chase Konkey Dong and it is enabled, listener addPygmy; 2.

### KonkeyDongBeetleLogic

- `rollAfterPygmy` () `0x1000c630c` [1 use]: First time: roll toward the pygmy at +/-100. Completes (2) on platform side collision (unrolls, stops); else 0.
- `rollUpComplete` () `0x1000c6414` [2 uses]: Marks rolled up; on a ladder: stop, ignore gravity, setBehavior: KonkeyDongBeetleUpLadder/DownLadder (3); else 2.
- `smashComplete` () `0x1000c640c` [1 use]: No-op; 2.
- `startLadder` () `0x1000c64a4` [2 uses]: y velocity = 50 when moving up the ladder, else -50 (table DAT_10029d410); 2.
- `walk` (minTime, maxTime, direction, endOfIsland) `0x1000c614c` [16 uses]: First time: random duration [minTime, maxTime]; velocity = xScale*50 (100 rolled up); ladder skipping bookkeeping. Completes (2) on platform side collision or timeout; else 0.
- `whichWayDoIGo` () `0x1000c5fb0` [5 uses]: Randomly (forced at platform ends) picks KonkeyDongBeetleWalk/Turn or, when rolled up, KonkeyDongBeetleRollWalk/RollTurn; 3.

### KonkeyDongBoulderLogic

- `crackComplete` () `0x10017ecc4` [1 use]: Removes the display object; 2.

### KonkeyDongLadderLogic

- `setLadderParams` (height) `0x10000d104` [4 uses]: mLadderHeight = height; 2.

### KonkeyDongLogic

- `cancelKidnapPygmyComplete` () `0x10009013c` [1 use]: Removes the display object; 2.
- `endLevelComplete` () `0x10009031c` [1 use]: Listener endLevelComplete; 2.
- `kidnapPygmyComplete` () `0x1000900b0` [1 use]: Releases the kidnapped pygmy (clear channel 1 now), listener kidnapPygmyComplete, pygmy kidnappedByKonkeyDongComplete; 2.

### KonkeyDongPlatformLogic

- `setPlatformParams` (width) `0x1000cdc2c` [10 uses]: mCollisionWidth = width; 2.

### LaserGunLogic

- `onShootLaserComplete` () `0x1001739fc` [1 use]: Listener onShootLaserComplete:; 2.

### LaserSharkLogic

- `die` () `0x1000c5b00` [119 uses]: exitIsland; 2.
- `eatPygmyComplete` () `0x1000c55ec` [4 uses]: Pygmy swallowedByLaserShark:, forget it; 2.
- `initPos` () `0x1000c5630` [2 uses]: Random side: x -50 (facing right) or 530 (facing left), z-rot 0, y random 70..250 (0.1 steps); 2.
- `swim` (minTime, maxTime, noReset) `0x1000c5710` [16 uses]: Steers (z-rot clamped to +/-30 deg, eased 50%) at speed 150 toward chum (or a pygmy holding chum) or level y 160. Mouth within 20 of feedable chum: LaserSharkEatChum, or LaserSharkEatPygmy for a holding pygmy (3). After a shot/chum and past the screen edge: LaserSharkTurn (3). Leaves the screen (x < -50 / > 530): listener onLaserSharkExitScreen:, 2. Else 0.

### LavaLogic

- `die` () `0x10000d5f0` [119 uses]: Listener onLavaDie:; 2.
- `fall` (ignoreIsland) `0x10000d288` [134 uses]: Keeps the lava blob on screen; hits every pygmy within 64 units (hitByLava:). Below y=76 over the island: stops at y=76 (2); else 0.

### LightningLogic

- `blinkLightning` (time) `0x1000c924c` [1 use]: First time: timer = time. Toggles alpha 1 <-> 0 of the bolt and its highlights every frame; completes (2) when the timer runs out, else 0.
- `die` () `0x1000c9424` [119 uses]: Removes the highlight display objects, listener onLightningDie:; 2.

### LocustLogic

- `catchPygmy` () `0x10008d694` [2 uses]: When the swarm target is within 10 of the pygmy (+20 y) and the pygmy is not rising: holds it (catchInAirByLocusts:), 2; else 0.
- `liftPygmy` () `0x10008d588` [1 use]: Lifts the carried pygmy 100 units/s; completes (2) once 100 above the start y; else 0.
- `throwPygmy` () `0x10008d44c` [1 use]: Lifts the carried pygmy 400 units/s with the swarm target 20 above it; once above the start y: releases it with throwPygmy: (2); else 0.

### LocustQueenLogic

- `eatPygmyComplete` () `0x100199dc8` [4 uses]: If new locusts can be created: +20 embryos and the belly logic -> LocustQueenBellyPregnant; clears channel 1 now, pygmy eatenByLocustQueenComplete:; 2.

### MagnifyingGlassLogic

- `die` () `0x1000d3db4` [119 uses]: Removes the glass, ends a drag, resets state; re-enters the island if the ant hill is enabled; 2.
- `drag` (yOrigin, xOrigin) `0x1000d3984` [40 uses]: First time: gravity on, floor collision off, insert after DragDisplayMarker, creates the MagnifyingHotSpot model (texture HotSpot_01) and the MagnifyingBeam model. Spring drag (k 50, damping 6), no rotation; never completes (0).
- `fall` (ignoreIsland) `0x1000d3c68` [134 uses]: Falls (ignoreIsland disables floor collision); on the island 2; below y=50: setBehavior: MagnifyingGlassSplash (3); else 0.
- `slide` () `0x1000d389c` [20 uses]: First time: floor collision + gravity on. Below y=50: setBehavior: MagnifyingGlassSplash (3); else 0.

### MapPaperLogic

- `onMapPaperAnimComplete` () `0x10017a20c` [unused]: Listener onMapPaperLogicAnimComplete; 2.
- `onMapPaperEnterComplete` () `0x10017a23c` [1 use]: Listener onMapPaperLogicShowMap; 2.
- `onMapPaperExitComplete` () `0x10017a29c` [1 use]: Listener onMapPaperLogicAnimComplete; 2.
- `onMapPaperHideContentsComplete` () `0x10017a26c` [1 use]: Listener onMapPaperLogicHideMap; 2.

### MapScreen

- `onMapCloseComplete` () `0x1000c122c` [unused]: No-op; 2.

### MessageLogic

- `countDownTimer` (time) `0x10019efb8` [1 use]: First time: timer = time. Shows the remaining time (format "%d") in ChallengeTimer/Timer; completes (2) when it runs out, else 0.
- `onMessageComplete` () `0x10019f0e8` [5 uses]: Deactivates, removes the display object, listener onMessageComplete:; 2.

### MonkeyLogic

- `chasePygmy` () `0x1000d18c8` [1 use]: Side collision + gravity on. Pygmy gone: MonkeyIdle (3); pygmy cannot be chased: MonkeyCancelChasePygmy (3); within 10 in x: faces the throw-to monkey, 2; else 0.
- `checkForDance` () `0x1000d07a0` [3 uses]: startDancing if the listener reports isDanceActive; 2.
- `checkForPlaguePygmy` () `0x1000d0728` [5 uses]: If the caught pygmy had the plague: unselectable, setBehavior: MonkeyCaughtPlaguePygmy (3); else 2.
- `die` () `0x1000d20e0` [119 uses]: Removes the monkey and helper, stops interactions, ends a drag, resets state; 2.
- `drag` (yOrigin, xOrigin) `0x1000d1e04` [40 uses]: First time: gravity on, floor collision off. Spring drag with rotation; never completes (0).
- `fall` (ignoreIsland) `0x1000d1fe4` [134 uses]: Falls (ignoreIsland disables floor collision); lands in a banana tree / ape torch (3), on the island (side collision on, 2); else 0.
- `finishEatingBanana` () `0x1000d18c0` [2 uses]: No-op; 2.
- `idle` (minTime, maxTime) `0x1000d1cf0` [17 uses]: First time: random duration [minTime, maxTime], gravity on; completes (2) when the time is up, else 0.
- `jumpOffTorch` () `0x1000d17f8` [1 use]: Gravity + floor collision on, velocity (xScale*200, 200), leaves the perch, ape torch monkeyJumpOffTorch:; 2.
- `jumpOffTree` () `0x1000d113c` [1 use]: Gravity + floor collision on, velocity (200,200), leaves the perch, banana tree monkeyJumpOffTree:; 2.
- `monkeyEnterIslandComplete` () `0x1000d0844` [1 use]: Floor and side collision on, removes the helper display object, selectable; 2.
- `resetLastPos` () `0x1000d07f4` [5 uses]: mLastPos = current position; 2.
- `walk` (minTime, maxTime, direction, endOfIsland) `0x1000d1a9c` [16 uses]: First time: random duration [minTime, maxTime], side collision + gravity on. Banana / banana tree / ape torch / sleeping pygmy found: 3. On side collision or timeout: mbEnableHump = (rand%10001)/100 < 20; mbEnableClimb ends up YES unconditionally (its 30% roll is overwritten by a second store, original quirk); stops, 2. Else 0.
- `whichWayDoIGo` () `0x1000d08c0` [5 uses]: Random direction (forced at x 80 / 400): MonkeyWalk if already facing it, else MonkeyTurn; 3.

### MoronBeeLogic

- `die` () `0x100175160` [119 uses]: No-op; 2.
- `exitScreenComplete` () `0x1001752bc` [unused]: Removes the display object; 2.

### MoronDuckLogic

- `die` () `0x1001a80d0` [119 uses]: No-op; 2.
- `exitScreenComplete` () `0x1001a833c` [unused]: Removes the display object; 2.

### MoronPestsQuestionScreenLogic

- `closeAnimalQuestionComplete` () `0x100066130` [1 use]: For duck (3) / bee (2) questions: correct when no such animals remain active; tells the listener; 2.
- `closeColorQuestionComplete` () `0x100065f2c` [1 use]: Grades the colour question by mColorQuestionType (colour match, don't-answer timer, inverted match, or tap-sequence match) and tells the listener Correct/Incorrect; 2.
- `closeSynonymQuestionComplete` () `0x100065b04` [1 use]: Grades the synonym question (type 0: selected == correct; 1: correct only if the don't-answer timer expired; 2: selected != correct) and tells the listener onMoronPestsQuestionCorrect / onMoronPestsQuestionIncorrect; 2.

### MoronSnakeLogic

- `die` () `0x10018d680` [119 uses]: Removes the snake, stops interactions, ends a drag, resets state; re-enters the island if enabled and no story is active; 2.
- `drag` (yOrigin, xOrigin) `0x10018d34c` [40 uses]: First time: gravity on, floor collision off. Spring drag with rotation; never completes (0).
- `fall` (ignoreIsland) `0x10018d4fc` [134 uses]: Falls (ignoreIsland disables floor collision); on the island: side collision on, 2; below y=10: setBehavior: MoronSnakeSplash (3); else 0.
- `idle` (minTime, maxTime) `0x10018d238` [17 uses]: First time: random duration [minTime, maxTime], gravity on; 2 when the time is up, else 0.
- `resetLastPos` () `0x10018cfac` [5 uses]: mLastPos = current position; 2.
- `slither` (minTime, maxTime) `0x10018d0e8` [1 use]: First time: random duration [minTime, maxTime], side collision + gravity on. Completes (2) on side collision or timeout (stops); else 0.
- `whichWayDoIGo` () `0x10018cffc` [5 uses]: Random direction (forced at x 32 / 420): MoronSnakeSlither if facing it, else MoronSnakeTurn; 3.

### MoronTurtleLogic

- `die` () `0x100173338` [119 uses]: No-op; 2.
- `onEatPygmyComplete` () `0x10017357c` [1 use]: Pygmy eatenByMoronTurtleComplete:, clear channel 1 now; 2.
- `onPygmyJumpOnBackComplete` () `0x10017347c` [1 use]: Pygmy jumpOnMoronTurtleBackComplete:, clear channel 1 now, starts chasing, setBehavior: MoronTurtleStartRunning; 2.

### NewsManager

- `loadLocationTextureAtlas` () `0x10005674c` [13 uses]: Listener onLoadLocationTextureAtlas; 2.
- `updateNewsScreen` () `0x100056704` [2 uses]: Calls updateNewsScreen; 2.
- `updateNewsTeaser` () `0x100056728` [1 use]: Calls updateNewsTeaser; 2.

### NoPeeTreasureLogic

- `onNoPeeTreasureCollectComplete` () `0x10008e0e8` [1 use]: Level +1, uncollected, respawns at a random non-overlapping y in [100,800], setBehavior: NoPeeTreasureInit; 2.
- `setAnimationBasedOnLevel` (list) `0x10008e014` [1 use]: Plays entry mTreasureLevel of the comma-separated `list` (clamping the level to the last entry); 2.

### OctoStatueLogic

- `blowUpPygmies` () `0x100118c54` [1 use]: Every pygmy explodeFromLovingPocketGod; 2.
- `onActivateComplete` () `0x100118c40` [1 use]: Clears mbActive; 2.

### OneShotAnimLogic

- `oneShotComplete` () `0x100037a74` [1 use]: Removes the display object, shuts down and releases the logic; 2.

### OutHouseLogic

- `finishCrapping` () `0x10002304c` [1 use]: If a pygmy is inside: clears channel 1, pygmy finishCrapping, forget it; 2.

### PiranhaLogic

- `attack` () `0x10000a4f0` [1 use]: First time: 2 s timer, z-rot target 0 or +/-180, cancelFlip. Eases rotation; when the timer runs out: finish eating, pygmy eatenByPiranhaComplete: (2); else 0.
- `die` () `0x10000a4b8` [119 uses]: Listener onPiranhaDie:; 2.
- `eatPygmyComplete` () `0x10000a6ac` [4 uses]: Stops eating and swims off along its heading at speed 50; 2.
- `initPos` () `0x100009b4c` [2 uses]: x velocity +50 (or -50 with z-rot 180 when x > 0); mTargetZRot = current/180; 2.
- `swim` (minTime, maxTime, noReset) `0x100009bf8` [16 uses]: Follows the drawn path (mpPathVector, speed 50) or bounces inside x 50..430 / y 50..270, easing z-rot 40%/frame toward the heading (PiranhaFlip / PiranhaFlipFull when the facing must flip). Reaching a path pygmy within 20: eats it (PiranhaAttack); a pygmy already being eaten makes the two piranhas fight (listener onPiranhaFight:); after eating, leaving x -20..500 -> listener onPiranhaOffScreenAtePygmy:. Those cases return 3; otherwise 0.
- `swimAway` () `0x10000a370` [1 use]: First time: swims away from the centre at +/-200. Completes (2) once x < -20 or > 500 (listener onPiranhaDie:); else 0.

### PocketGodViewController

- `loadPreferences` () `0x10007bdcc` [1 use]: Loads episode state, registers every default option preference, creates the God/Tribe/Pygmy0-5/Charlie/Tom personas, applies shark/bird/weather/island state, sets each toggle button's Start...On behaviour from the saved options, applies mute/vibrate (and starts the background loop when not muted); 2.
- `loadTextureAtlasChannel` (channel) `0x10007d3bc` [3 uses]: `channel` "Disco" -> skin pack manager loadTextureAtlasChannel: 2, "Xenu" -> 3; anything else (e.g. "Spider") does nothing; 2.
- `pauseHurricaneSounds` () `0x10007109c` [5 uses]: pauseHurricaneSounds; 2.
- `playBackgroundLoop` (soundId) `0x10007104c` [25 uses]: playBackgroundLoop: `soundId` (nil when absent); 2.
- `refreshSkinPackOption` (skinPackId, skinOptionId, skinObjectId) `0x10007d468` [8 uses]: Skin pack manager refreshSkinPackOption:`skinPackId` skinObjectId:`skinObjectId` skinOptionId:`skinOptionId` useFirstPurchased:YES; 2.
- `resumeHurricaneSounds` () `0x1000710c0` [5 uses]: resumeHurricaneSounds; 2.
- `startTime` () `0x10007bc30` [unused]: Records the current time (profiling); 2.
- `stopBackgroundLoop` () `0x100071078` [6 uses]: stopBackgroundLoop; 2.
- `stopTime` () `0x10007bc50` [unused]: Logs the elapsed script time (profiling); 2.
- `updateHelpPage` () `0x10007737c` [1 use]: updateHelpPage: mHelpPageIndex; 2.
- `waitForHoldLoadingAd` () `0x10007bca8` [unused]: Completes (2) once 5 s have passed since app start, else 0.
- `waitForPreferencesLoadComplete` () `0x10007bce8` [1 use]: Completes (2) once the Persona, Challenge, Options and System preference documents have loaded, else 0.

### PocketGodViewController(Game)

- `addBabyPygmy` () `0x10015c088` [unused]: Creates a pygmy logic, resurrects it in the tribe and makes it a baby (timeTemplePygmyType 2); 2.
- `addPygmy` () `0x10015c064` [1 use]: addPygmy; 2.
- `addTomPygmy` () `0x10015c0f8` [unused]: createTomPygmy; 2.
- `checkForNewNews` () `0x100159dc0` [2 uses]: If the news manager has unread news: opens the news teaser; 2.
- `checkShowAuthenticationViewController` () `0x100159c44` [2 uses]: GameServer checkShowAuthenticationViewController; 2.
- `checkStartScreen` () `0x100159c80` [1 use]: Marks init complete; restarts the main sequence (mSequence) with StartScreen if checkForIdolStartScreen, else NoStartScreen; 3.
- `closeMenu` () `0x100159e54` [4 uses]: Closes the menu (category menu state 0), targets z-rot 90 / y 90; 2.
- `enableTouch` () `0x100159998` [1 use]: BCView setTouchActive:YES; 2.
- `fireworks` (minTime, maxTime) `0x10015a7c8` [1 use]: Skipped (2) for island location states 11..17. Waits a random [minTime, maxTime] (0.001 s steps), then spawns one Firework model (FireworkSpark, random colour, x 50..430, y 190..320 in 0.1 steps) with a FireworkLogic after the firework marker; never completes otherwise (0).
- `hideLoadingScreen` () `0x100159d40` [2 uses]: hideLoadingScreen; opens a pending launch URL on the main thread; 2.
- `loadLocationTextureAtlas` () `0x10015a118` [13 uses]: loadLocationTextureAtlas; 2.
- `openMenu` () `0x100159e2c` [1 use]: Menu button z-rot target 0, menu y target 0; 2.
- `pauseLocustSounds` () `0x100169174` [5 uses]: pauseLocustSounds; 2.
- `pauseSleepingSounds` () `0x10016903c` [5 uses]: pauseSleepingSounds; 2.
- `removeMessage` () `0x10015a0bc` [1 use]: Removes the "Message" model; 2.
- `resumeLocustSounds` () `0x100169198` [5 uses]: resumeLocustSounds; 2.
- `resumeSleepingSounds` () `0x100169060` [5 uses]: resumeSleepingSounds; 2.
- `setCloudGradientFade` (fade) `0x10015e32c` [2 uses]: mCloudGradientTargetFade = fade; 2.
- `setLogic` (displayGroupId, logicId, displayObjectId, manager, count) `0x10015c11c` [275 uses]: Logic factory. If `manager` is ChallengeManager/SkinPackManager/BonusPackManager/StoryManager/NewsManager/ChumBucket, forwards the whole item to that object's setLogic:. Otherwise creates the logic named by `logicId` for object `displayObjectId` of display group `displayGroupId`, wires its listener and stores it (see setLogic table); `count` is used by PygmyStatsButtonLogic; 2.
- `shakeBackground` (minTime, maxTime) `0x10015e374` [4 uses]: First time: duration random [minTime, maxTime] (0.001 s) or endless when minTime is absent. Shakes the island (or The Runs) display group y = 2*sin(phase) - 2; completes (2) when the time is up, never when endless; else 0.
- `showBattleOfTheGodsDisplayGroup` () `0x10016924c` [1 use]: showBattleOfTheGodsDisplayGroup; 2.
- `showKonkeyDongDisplayGroup` () `0x1001691e0` [1 use]: showKonkeyDongDisplayGroup; 2.
- `showMessage` (zPos, xPos, texture, yPos) `0x100159ebc` [3 uses]: Replaces the "Message" model with texture `texture` at (xPos, yPos, zPos), inserted after the menu marker, and runs ShowMessage on mMessageSequence; 2.
- `showMoronPestsDisplayGroup` () `0x100169204` [1 use]: showMoronPestsDisplayGroup; 2.
- `showNoPeeDisplayGroup` () `0x100169228` [1 use]: showNoPeeDisplayGroup; 2.
- `showTheRunsDisplayGroup` () `0x1001691bc` [1 use]: showTheRunsDisplayGroup; 2.
- `startPygmiesDancing` () `0x10015a13c` [1 use]: Starts every pygmy dancing (vampire dance / normal-dance lead selection by skin-pack options) and starts a new mCelebrationSequence with Celebration; 2.
- `stopEarthquake` () `0x10015e524` [2 uses]: Stops mEarthquakeSequence; if earthquakes remain queued starts a new one with EarthquakeStart; 2.
- `stopPygmiesDancing` () `0x10015a454` [1 use]: Stops every pygmy dancing (song flags from skin-pack options; immediately when the skin-pack screen is open) and stops mCelebrationSequence; 2.
- `stopTheRunsShakeScreen` () `0x10015e608` [1 use]: Stops mEarthquakeSequence and resets The Runs display group y to 0; 2.
- `transitionInClouds` () `0x10015ea30` [1 use]: Stops the hurricane, createClouds, every cloud transitionIn:; 2.
- `transitionInStormClouds` () `0x10015e684` [1 use]: Front/behind storm clouds transitionIn: mCloudDragVel; 2.
- `transitionOutClouds` () `0x10015eb94` [1 use]: Every cloud transitionOut:; 2.
- `transitionOutStormClouds` () `0x10015e6fc` [1 use]: Front/behind storm clouds transitionOut: mCloudDragVel; 2.
- `triggerEpisodeEvent` (event) `0x10015a78c` [392 uses]: IdolManager triggerEpisodeEvent: `event`; 2.

### PygmyBottomLogic

- `die` () `0x100113048` [119 uses]: Deactivates and removes the display object; 2.
- `fall` (ignoreIsland) `0x100112f88` [134 uses]: First time: gravity on. In an enabled ocean at y <= 16: stops at y=16 (2); else 0.

### PygmyLogic

- `checkPygmyDialog` (alternateSequenceId, id) `0x10012ff78` [32 uses]: Dialog throttle over a global id->time dictionary (entries older than 15 s dropped): if any dialog started < 3 s ago or `id` ran < 15 s ago, setBehavior: `alternateSequenceId` (3); else records `id` now (2).
- `die` () `0x10012ecb8` [119 uses]: Death bookkeeping: tumble/earthquake/Konkey Dong listeners, cures disease, ejects the pygmy from every container (outhouse, grave, pedestals, temple doors, platform, locust queen, fountain, web, drain...), resolves the sacrifice id (heart attack, Charlie, Xenu, zombie, flattened...) and reports onPygmySacrificeEvent:, ends a drag. If ghosts are enabled and the death allows it: becomes a ghost (gift BecomeGhost, setBehavior: PygmyGhostInit, 3). Otherwise stops a banshee scream (PygmyGhostStopScream), removes the name label, listener onPygmyDie:sacrificeId: (closet grabNewTom when flagged); 2.
- `disableLabels` () `0x10012ea44` [18 uses]: Sets mbDisableLabels; 2.
- `drag` (yOrigin, xOrigin) `0x100132094` [40 uses]: First time: gravity on, no island collisions, origin = (xOrigin, yOrigin) (0 when absent), moves to the drag layer (unless underwater). Spring drag with rotation; "Oh" sounds (BabySound1 / OldOh / Oh1-3) when flung faster than 500 (60-frame cool-down); tracks being over a burning bonfire; lets the shark check for a jump; never completes (0).
- `explode` () `0x10012fcc0` [8 uses]: Listener onPygmyExplode:; a lightning-struck pygmy (not held anywhere) leaves an AshPile multimodel (MultiModelLogic, AshPileInit) at its position; leaves the outhouse queue; tells a burning magnifying glass onPygmyDie:; 2.
- `fall` (ignoreIsland) `0x100130fb8` [134 uses]: First time: floor collision unless ignoreIsland. Checks every catcher during the fall (absorb, Kong statue, ape wench/bongo/altar, go-go cage, DDE, igloo, zombie grave, idol pedestal, temple doors, time platform, locust queen, fountain, closet temple, Tom, Xenu, ice hole) -> 3. Ocean landing: one of ~14 splash behaviours (PygmySplash, PygmyChokeSplash, PygmyBeheadedSplash, ...) or fall scream; island landing: tumble/flick handling (FastPygmyFlick event), old-fart TempleOldPygmyFallDown/LandOnFeet; returns 0 while falling, 2 on a plain landing, 3 after setBehavior:.
- `flipXScale` () `0x10012ea5c` [1 use]: Negates the x scale; 2.
- `hangUpsideDown` () `0x10012eab4` [1 use]: Each frame: y velocity 0, z-rot = raw accelerometer z rotation, y = island y; never completes (0).
- `idle` (minTime, maxTime) `0x10012eb58` [17 uses]: First time: random duration [minTime, maxTime], gravity on. Charlie tiger blood or a shark found: 3. Timeout: 2; else 0.
- `panicRun` (minTime, totalTime, maxTime) `0x10013089c` [3 uses]: First time: total time = totalTime, turn timer random [minTime, maxTime], runs toward the centre at +/-200. Turns around on side collision or when the turn timer expires; completes (2) when the total time is up; else 0.
- `pushSequenceBasedOnPersonality` (defaultId, charlieId, tomId, babyPlagueId, babyPurificationId, oldFartPlagueId, purificationId, gingerPlagueId, plagueId, fastPurificationId, gingerPurificationId, fastPlagueId, oldFartPurificationId, fastId, babyId, oldFartId, gingerId) `0x100132a64` [34 uses]: pushSequenceDef: the id selected by temple type / disease / personality (see Sequence parameter selection); does nothing when the final choice is nil; 2.
- `resetVel` () `0x1001301c8` [5 uses]: Zeroes mPosVel x/y/z; 2.
- `run` (minTime, maxTime) `0x1001306bc` [2 uses]: First time: random duration [minTime, maxTime], runs toward the island centre at +/-100. Side collision or timeout: 2; else 0.
- `runSequenceBasedOnPersonality` (defaultId, babyId, charlieId, tomId, fastId, oldFartId, gingerId, babyPurificationId, babyPlagueId, oldFartPlagueId, gingerPlagueId, fastPurificationId, gingerPurificationId, fastPlagueId, oldFartPurificationId, purificationId, plagueId) `0x100132c9c` [14 uses]: setBehavior: the id selected by temple type / disease / personality; does nothing when nil; 2.
- `setAnimationBasedOnPygmyType` (oldFartPurification, oldFartPlague, normal, babyPlague, gingerPlague, purification, fastPurification, plague, gingerPurification, babyPurification, fastPlague, fast, ginger, baby, oldFart, normalPurification, normalPlague) `0x100132898` [56 uses]: Plays the param for (timeTemplePygmyType, mApocalypseDiseaseType): type-specific plague/purification variant, then the type param, then plague/purification, then normal (see Sequence parameter selection); 2.
- `setOrigin` (zPos, xPos, yPos, inPlace) `0x1001326a4` [72 uses]: As BCLogic::setOrigin; with `inPlace` also adjusts mLastPos by the origin change times scale; 2.
- `slide` () `0x100130b94` [20 uses]: First time: sacrifice id SlideDrown (healthy normal pygmies), floor collision + gravity on. Off the screen side: zombie explode (3) or PygmyFallIgnoreIsland (3). During an eruption a tilted (state 3/4) healthy pygmy is picked up (PygmyPickupForward/Backward). Below y=16 in an enabled ocean: drowns with PygmySplash / TemplePygmySplash / HalfPygmySplash / PygmyPlagueBodySplash / PygmyCookedSplash (3). Else 0.
- `tumble` () `0x1001317ec` [5 uses]: First time: FastPygmyFlick event for fast temple pygmies, listener onPygmyTumble:, no collisions, gravity on, falling (0). Then handles the flick arc: ice monster collisions (PygmyExplodeOnIceMonster / TempleFastPygmyExplodeOnIceMonster, eaten -> PygmyDie), volcano (ThrownInVolcano / FastPygmyHitVolano -> PygmyDie), igloo cannon / top explode, ocean splashes (PygmySplash / TemplePygmySplash, FlickDrown); 0 while airborne, 2 or 3 at the end.
- `walk` (minTime, maxTime, direction, endOfIsland) `0x1001301e8` [16 uses]: First time: random duration [minTime, maxTime], side collision + gravity on; `direction` "Right" walks right, any other value left, absent = random (forced inward at the island ends); speed +/-50. Outside story mode, any interaction found (tiger blood, coconut, igloo, wood, plague body, outhouse, bonfire, fish, bird, banana, fishing pole, shark): 3. Side collision: 2. Timeout: stops and 2, except within 35 of a burning bonfire, where the timer is extended by another random [minTime, maxTime] and it keeps walking. Else 0. (`endOfIsland` is not read.)

### PygmyLogic(Apocalypse)

- `brawlAttackComplete` () `0x10003ff14` [2 uses]: If still brawling with a defender: setBehavior: PygmyThetanBrawlAttackLoop (3); else 2.
- `checkDiseaseIncubation` () `0x100041d60` [10 uses]: Counts down mDiseaseIncubateIdleCycles; at 0: plague -> PygmyKilledByPlague (or PygmyKilledByPlagueDisappear when 6+ pygmies and all others are plague-dead), purification -> PygmyKilledByPurification (sacrifice id KilledByPurification); 3. Otherwise 2.
- `checkForFinishClosetIdle` () `0x100041c48` [1 use]: In the closet: setBehavior: FinishPygmyTomClosetIdle (3); else 2.
- `checkForLocusts` () `0x100042c34` [3 uses]: If the locusts make pygmies panic: stops all activities and setBehavior: PygmyLocustPanicIdle; 2.
- `checkForLocustsComplete` () `0x100042d98` [1 use]: If the locusts no longer cause panic: setBehavior: PygmyIdle; 2.
- `checkForPossessedByThetan` () `0x10003ed54` [1 use]: If possessed: setBehavior: PygmyPossessedByThetanIdle (3); else 2.
- `checkForTempleDoorJump` () `0x10003cf08` [1 use]: A normal, healthy, non-temple pygmy within 80 of the available temple door (+90): stops, jumps into it (door jumpIntoTempleDoor:, unselectable), setBehavior: PygmyJumpIntoTempleDoor (3); else 2.
- `checkMutantLocustFlyOff` () `0x100042e44` [1 use]: On the 3rd idle loop: flies off (velocity (xScale*100, 100), no gravity/collision), setBehavior: PygmyMutatedByLocustsFlyOff; 2.
- `checkSlamDoorBouncingOnCouch` () `0x10003e3a4` [1 use]: On the couch: setBehavior: PygmyTomJumpOffCouchAndSlamDoor (3); else 2.
- `checkTempleOldPygmyFallDownFromSlide` () `0x10003cae0` [1 use]: An upright old-fart temple pygmy falls over: setBehavior: TempleOldPygmyFallDownFromSlide (3); else 2.
- `checkTomJumpOnCouch` () `0x10003df04` [1 use]: Tom (personality 2) in the closet near an available closet-temple couch (between couch x+10 and x+90 in his facing): jumps onto it (velocity (xScale*100, 600)), setBehavior: PygmyTomJumpOnCouch (3); else 2.
- `chooseBrawlAttack` () `0x10003fe24` [1 use]: Random of 3: slap fight (ends the brawl; both start PygmyThetanBrawlStartSlapFight / thetanBrawlStartSlapFight; 3), defender throws a punch (2), or self PygmyThetanBrawlThrowPunch (3).
- `closetIdle` (minTime, maxTime) `0x100047290` [3 uses]: Random duration [minTime, maxTime]; 2 when done, else 0.
- `closetWalk` (minTime, maxTime) `0x100047394` [3 uses]: First time: random duration [minTime, maxTime], walks +/-50 randomly (inward at the closet ends). Completes (2) at the closet min/max x (clamped) or timeout; else 0.
- `jumpIntoTempleDoorComplete` () `0x10003d14c` [1 use]: Temple door jumpIntoTempleDoorComplete:; 2.
- `locustThrowWaitForApex` () `0x100042c10` [2 uses]: Completes (2) when the y velocity becomes negative, else 0.
- `mouthFilledByLocustsComplete` () `0x100042fbc` [1 use]: Selectable again; 2.
- `mutatedByLocustsComplete` () `0x100042e0c` [1 use]: Selectable again; 2.
- `plagueBodyFloat` () `0x100047828` [1 use]: First time: no island collisions. Wraps x around the 480 screen; never completes (0).
- `possessedByThetanIdle` (minTime, maxTime) `0x1000401dc` [1 use]: Random duration [minTime, maxTime]; 2 when done, else 0.
- `possessedByThetanWalk` (minTime, maxTime) `0x10003ff5c` [1 use]: Like walk (random [minTime, maxTime], +/-50, inward at island ends) but starts a Thetan brawl when the cool-down has expired and one is possible (3); side collision or timeout: 2; else 0.
- `respawnPlagueBody` () `0x1000475f8` [2 uses]: Respawns the plague body at a random island x (inset 15, 0.1 steps), y 340, z -277.12, unit scale, upright, falling onto the island; restores a hurricane-dropped body to the pygmy layer; marks the sacrifice invalid; 2.
- `runSequenceIfInXenuRoom` (id) `0x10003de70` [2 uses]: If the island location state is 17 (Xenu temple), no apocalypse is active and `id` is present: setBehavior: `id` (3); else 2.
- `runToClosetDoor` () `0x100041c90` [1 use]: Runs right at 400 until past maxClosetXPos (2); else 0.
- `templeOldPygmyFallDown` () `0x10003cb44` [1 use]: Clears mbTempleOldPygmyUpright; 2.
- `templePygmyIdle` (minTime, maxTime) `0x10003ce08` [14 uses]: First time: random duration [minTime, maxTime]; 2 when the time is up, else 0.
- `templePygmyWalk` (minTime, xPosVel, maxTime) `0x10003cb5c` [19 uses]: First time: random duration [minTime, maxTime], walk speed xPosVel in a random direction (forced inward at island ends), side collision + gravity on. Coconut/fish/bird/banana found: 3. Side collision or timeout: stop, 2. Else 0.
- `thetanEatPygmyFace` () `0x10003fd0c` [1 use]: The defending pygmy thetanBrawlGetFaceEaten; 2.
- `thetanPummelPygmy` () `0x10003fcd8` [1 use]: The defending pygmy thetanBrawlGetPummeled; 2.
- `throwPlagueBodyOnPile` () `0x100047c74` [1 use]: Arcs toward the bonfire x (50%/frame); when falling below the fire: on a burning fire PygmyPlagueBodyBurnOnFire (sacrifice id PlagueBodyBurntOnFire), else back to the pygmy layer and PygmyPlagueBodyFall; 3. Else 0.
- `tomJumpOffCouch` () `0x10003e238` [2 uses]: Gravity on, stopBeingOnClosetTempleCouch, velocity (xScale*100, 600); 2.
- `waitApexTomJumpOnCouch` () `0x10003e114` [3 uses]: Completes (2) when the y velocity becomes negative, else 0.
- `waitForMutantLocustFlyOffComplete` () `0x100042f20` [1 use]: Completes (2) once off screen (maxX < 0, minX > 480 or minY > 580), else 0.
- `waitLandTomJumpOffCouch` () `0x10003e2c4` [2 uses]: While falling within 20 of the closet y: lands there (no gravity, stop), 2; else 0.
- `waitLandTomJumpOnCouch` () `0x10003e138` [1 use]: While falling within 20 of couch y-35: lands (y = couch y-35, no gravity, stop), 2; else 0.
- `walkToXenu` () `0x100040c34` [2 uses]: Walks (+/-50) to Xenu's x; on arrival: Xenu temple tearDownWall, unselectable, baby pygmy startTomBabySacrifice (2); else 0.

### PygmyLogic(BattleOfTheGods)

- `battleCheckNoRunBack` () `0x100122dd4` [1 use]: Already at the start line (x <= 100 for the player, >= 380 for the enemy): clamps, faces forward, setBehavior: PygmyBattleNoRunBack (3); else 2.
- `battleEnergyFastMoveAboveOpponent` () `0x100123240` [1 use]: Like battleEnergyFastMoveTowardOpponent; vs a fast energy opponent moves 60 ahead of itself instead; 2.
- `battleEnergyFastMoveTowardOpponent` () `0x100123164` [1 use]: Snaps x to the opponent x -/+ offset (100, or table DAT_10029d9b0 by opponent type when the opponent uses a sword); 2.
- `battleEnergyFastWaitForOpponent` () `0x100123360` [unused]: Completes (2) when within 100 of the opponent, else 0.
- `battleFall` (ignoreIsland) `0x1001244c8` [2 uses]: Falls with side collision off (ignoreIsland disables floor collision); x clamped to [20,460] (or [-50,530] when killed in battle). Lands (2), or a killed fighter stops at y=0 below the floor (2); else 0.
- `battleFastRunTowardOpponent` () `0x10012306c` [1 use]: Within 100 of the opponent: vs a fast opponent -> PygmyBattleAttackFastTieAgainstEnergy (energy weapon) or PygmyBattleAttackFastTie (3), else 2; else 0.
- `battleRespawn` () `0x1001246c0` [unused]: spawnFromBattleOfTheGods; 2.
- `battleRunBackToStart` () `0x100122ea4` [1 use]: First time: faces backward. Completes (2) on reaching x 100 (player) / 380 (enemy), facing forward; else 0.
- `battleShowLavaFlame` () `0x100124428` [1 use]: Sets global logic 376 (for the player's fighter) or 375 (for the enemy fighter) to BattleLavaPitFlame; 2.
- `battleStrongRunTowardOpponent` () `0x100122f90` [1 use]: Within 50 of the opponent: tie vs a strong opponent -> PygmyBattleAttackStrongTie (3), else 2; else 0.
- `checkForAttackTie` () `0x1001243a8` [5 uses]: Same battle types and this is the player: IdolManager onBattleAttackTie; 2.
- `checkForEnergyStrongAttackWin` () `0x100123458` [1 use]: Opponent type 1: PygmyBattleEnergyAttackStrongWin and the opponent is killedInBattle; type 0: (sword opponent: beam position + parryAgainstStrongEnergy) PygmyBattleEnergyAttackStrongTie; 2.
- `checkForFastAttackWin` () `0x100123538` [2 uses]: Opponent fast + own energy weapon: PygmyBattleEnergyAttackFastTie (3); opponent type 1 with own energy weapon: opponent battleCounterFastAttack (2); opponent strong: PygmyBattleAttackWin (3); else 2.
- `checkForStrongAttackWin` () `0x100123410` [1 use]: Opponent type 1: setBehavior: PygmyBattleAttackStrongWin; 2.
- `onBattleReady` () `0x1001246e4` [10 uses]: Temple door logic onPygmyBattleReady:; 2.
- `pushSequenceBasedOnOpponentBattleType` (fastEnergyId, fastSwordId, strongSwordId) `0x100122d10` [3 uses]: Opponent battle type (+0x74) 2 (fast): weapon 1 -> push `fastEnergyId`, weapon 0 -> `fastSwordId`; type 0 (strong) with weapon 0 -> `strongSwordId`; any other combination or nil id pushes nothing; 2.
- `pushSequenceBasedOnOpponentWeaponType` (energyId, swordId) `0x100122c80` [1 use]: Pushes `energyId` when the opponent's weapon type (episode node +0x78) is 1 (energy), else `swordId` (nothing if nil); 2.

### PygmyLogic(Charlie)

- `charlieCutPygmyInHalfComplete` () `0x10005cabc` [1 use]: No-op; 2.
- `charlieMonkeyIdle` (minTime, maxTime) `0x10005c9bc` [1 use]: Random duration [minTime, maxTime]; 2 when done, else 0.
- `charlieMonkeyWalk` (minTime, maxTime) `0x10005c78c` [1 use]: Like walk: random duration [minTime, maxTime], +/-50 random direction (inward at ends). Side collision or timeout: 2; else 0.
- `checkForWarlock` () `0x10005c2b4` [1 use]: If no other pygmy can be burnt by the warlock and lightning is not usable on anyone: setBehavior: FinishPygmyCharlieIdle; 2.
- `halfPygmyIdle` (minTime, maxTime) `0x10005c68c` [2 uses]: Random duration [minTime, maxTime]; 2 when done, else 0.
- `halfPygmyWalk` (minTime, maxTime) `0x10005c484` [2 uses]: First time: random duration [minTime, maxTime], random facing (inward at island ends), side collision + gravity on (velocity untouched). Side collision or timeout: stop, 2; else 0.

### PygmyLogic(Island)

- `burstPygmyEarDrums` () `0x1000b74a0` [1 use]: Every pygmy explodeFromBansheeScream:; 2.
- `carryPygmyOffScreen` () `0x1000b728c` [2 uses]: Completes (2) once x < -50 or > 530 (stops, ghost-touch pygmy carriedAwayByGhostComplete:, clears channel 1); else 0.
- `chaseKonkeyDongComplete` () `0x1000a8bf8` [1 use]: Kong statue startKonkeyDong:; 2.
- `checkDanceChange` () `0x1000a8b0c` [6 uses]: First time: remembers the Dance3/Pygmies/Pygmy Dance skin option. If it changes: setBehavior: PygmyRefreshGoGoCage (3); else 0.
- `checkForDance` () `0x1000ba58c` [3 uses]: If the dance is active: startDancing: (Pygmy Theme Song skin option active); 2.
- `checkForDay` () `0x1000ba44c` [1 use]: When night is over (or inside the temple): stopSleeping, PygmyWakeUp; 2.
- `checkForDreamBubble` () `0x1000ba4d8` [1 use]: If dream bubbles are enabled and global logic 2 (dream bubble) is inactive: activate: it for this pygmy; 2.
- `checkForGingerSunBurnt` () `0x1000b9e38` [3 uses]: In daylight without storm clouds (not in the temple): ginger sun burn, sacrifice id GingerPygmySunBurn, setBehavior: TempleGingerPygmySunBurnt; 2.
- `checkForNight` () `0x1000ba360` [2 uses]: At night (outside the temple): Charlie (once) PygmyCharlieNoSleep, others fall asleep (PygmyFallAsleep); otherwise clears Charlie's not-sleeping flag; 2.
- `checkForNoBonFire` () `0x1000b999c` [1 use]: If the bonfire is enabled but not burning: setBehavior: PygmyIdle; 2.
- `checkForNoSunset` () `0x1000ba2c4` [1 use]: If sunset is over (or inside the temple): stopSleeping, PygmyIdle; 2.
- `checkForRainbow` () `0x1000b9a28` [1 use]: If the rainbow is active (and not inside the apocalypse temple): startStaringAtRainbow; 2.
- `checkForSpider` () `0x1000b96c0` [3 uses]: Same as checkForTRex for the spider (stand min 52.80..94.40, stand max 385.60..427.20, run min 94.40..198.40, run max 281.59..385.60), setBehavior: PygmyStayAwayFromSpider; 2.
- `checkForSunBurnt` () `0x1000b9f0c` [1 use]: Inside the temple: 2. Sun state 2: 2..4 freeze cycles, PygmySunFreeze (3). State 1: counts down burn cycles; at 0 (and, on the ice island with a frozen hole that attracts him, only between x 290..350) PygmySunBurnt (stops, 3), else 2. State 0: PygmyIdle (3); other: 3.
- `checkForSunFrozen` () `0x1000ba108` [1 use]: Inside the temple: 2. State 2: counts down freeze cycles; at 0 PygmySunFrozen (stops, 3), else 2. State 1: 2..4 cycles, PygmySunBurn (3). State 0: PygmyIdle (3); other: 3.
- `checkForSunState` (loop) `0x1000b9c28` [5 uses]: Inside the temple: 2. Sun state 1/2: stops activities, 2..4 burn/freeze cycles, setBehavior: PygmySunBurn / PygmySunFreeze. Then returns 0 when `loop` is true (polls every frame), else 2.
- `checkForSunset` () `0x1000b9aac` [2 uses]: If sunset is active (not in the temple): stops all activities, setBehavior: PygmySunset; 2.
- `checkForTRex` () `0x1000b9078` [3 uses]: If the T-Rex is active: stops all activities, rolls the safe ranges (stand min 68.19..104.60, stand max 359.40..395.80, run min 104.60..195.60, run max 268.40..359.40; 0.01 steps), faces it, setBehavior: PygmyStayAwayFromTRex; 2.
- `checkForVampire` () `0x1000bde44` [1 use]: If bitten by a bat: setBehavior: PygmyKillPygmies (3); else 2.
- `checkKillPygmies` (minTime, maxTime) `0x1000bd798` [1 use]: Vampire check: in daylight PygmyVampireSun (sacrifice id VampireInSunlight) or at sunset PygmyVampireSunset (3); a pygmy that can be hit by a vampire -> PygmyKillPygmies (3); otherwise waits random [minTime, maxTime] (forever when absent): 2 on timeout, else 0.
- `checkStayAwayFromSpider` () `0x1000b9360` [2 uses]: Same as checkStayAwayFromTRex for the spider (PygmyIdle / PygmyRunUnderSpider / PygmyStepAwayFromSpider).
- `checkStayAwayFromTRex` () `0x1000b8d18` [2 uses]: T-Rex gone: PygmyIdle (3). Pygmy between the T-Rex and the island centre: faces away, PygmyRunUnderTRex (3). Outside its safe x range: PygmyStepAwayFromTRex (3). Else 2.
- `chooseRainbowAnimation` (list) `0x1000ae75c` [2 uses]: Draws (without replacement, global pool refilled from `list` when empty) a random animation; the six double-rainbow anims trigger the matching DoubleRainbow... episode event; plays it with self as animation listener; 2.
- `eatBrainsComplete` () `0x1000b80a8` [1 use]: Flips the x scale; 2.
- `eatenByMoronSnakeComplete` () `0x1000b1bd0` [1 use]: If eaten by the moron snake: sacrifice id EatenByMoronSnake, setBehavior: PygmyEatenByMoronSnakeComplete; snake eatPygmyComplete:; starts Moron Pests when enabled; 2.
- `explodeFromBansheeScream` () `0x1000b75d8` [1 use]: Stops, upright, sacrifice id FromBansheeScream, then runs explode::: and returns its result.
- `explodeFromLovingPocketGod` () `0x1000ba66c` [1 use]: Stops all activities, leaves the hurricane, stops, sacrifice id LovingPocketGod, then explode::: (its result).
- `explodeFromNotCrapping` () `0x1000ba894` [1 use]: Sacrifice id NotCrapping, then explode::: (its result).
- `explodeVampireGinger` () `0x1000ba8f4` [1 use]: Marks vampire-ginger explosion, then explode::: (its result).
- `explodeVampireSun` () `0x1000ba8d0` [2 uses]: Marks vampire-sun explosion, then explode::: (its result).
- `finishEatingBanana` () `0x1000b87bc` [2 uses]: Same as finishEatingCoconut with gift id Banana; 2.
- `finishEatingBird` () `0x1000b892c` [2 uses]: Unburnt: gift CookedBird when cooked (skipped for Charlie/old-fart); burnt (non-temple): marks ate-burnt-food, sacrifice id EatBurntBird; bird finishEating; clears channel 14 now; 2.
- `finishEatingCoconut` () `0x1000b8490` [1 use]: Unless Charlie (non-temple) or an old-fart temple pygmy: listener onPygmyGiftGiven:giftId: Coconut and the coconut finishEating; clears channel 14 now; forgets it; 2.
- `finishEatingFish` () `0x1000b8b04` [2 uses]: Unburnt: gift Fish or CookedFish (skipped for Charlie/old-fart); burnt (non-temple): sacrifice id EatBurntFish; fish finishEating; clears channel 14 now; 2.
- `finishEatingSwordFish` () `0x1000b8ca8` [1 use]: Marks swordfish eaten, sacrifice id SwordSwallower, swordfish finishEating; 2.
- `ghostFlyIdle` (minTime, maxTime) `0x1000b7778` [1 use]: First time: random duration [minTime, maxTime], random drift +/-50 in x and +/-20 in y (inward near the bounds x 20..440, y 40..280). Completes (2) on reaching a bound or timeout (stops); else 0.
- `ghostFlyUp` () `0x1000b7674` [2 uses]: First time: selectable, target y random 130..200, rises at 100. Completes (2) at the target y (stops); else 0.
- `ghostFollowPath` () `0x1000b7ad4` [2 uses]: Moves the ghost along mpGhostPathVector at 16.67 units per call (consuming points), facing the heading (PygmyGhostFollowPathFlip when the facing must flip, returns 0). Entering the touched pygmy's bounds triggers PygmyGhostScarePygmy/PossessPygmy/StartCarryAwayPygmy by touch type (scaredByGhost:/possessedByGhost:/carriedAwayByGhost:), or below y 96 with no touch type PygmyGhostPullPygmyUnder (3). End of path: into the light (TheLightEnter + PygmyGhostEnterTheLight, 3) or cancel the touch (2).
- `gushOverRainbowComplete` () `0x1000aea84` [1 use]: If gushing: rainbow logic gushOverRainbowComplete:; 2.
- `iglooRoll` () `0x1000be498` [1 use]: First time: no collisions or gravity. Knocks over pygmies within 40 in x (hitByPygmyIglooRoll:). Past the island max x: y velocity 500, gravity, PygmyFallIgnoreIsland (3); else 0.
- `initCarriedAwayByGhostComplete` () `0x1000b71c8` [1 use]: Stops, upright, no gravity / floor collision; ghost initCarriedAwayByGhostComplete:; clears channel 1; 2.
- `jumpOnFire` () `0x1000be72c` [2 uses]: Clears the on-fire flags, falls with velocity (+/-100 by facing, 600), gravity on, side collision off; 2.
- `killPygmies` () `0x1000be098` [1 use]: Vampire hunt: daylight/sunset as above (3). Runs (+/-200) at the nearest vampire-hittable pygmy; on contact (5 units, 40 for gingers) hits it (hitByVampire:xVel:) or, for a ginger temple pygmy, PygmyVampireAttackGinger (sacrifice id GingerPygmyKillVampire, 3); 0 while hunting. No victims: PygmyVampireIdle, 2.
- `onFallIntoCrackHoleComplete` () `0x1000b6f70` [1 use]: Listener onPygmyFallIntoCrackHoleComplete:; 2.
- `penetrateGroundComplete` () `0x1000b709c` [1 use]: Ghost-touch pygmy pulledUnderByGhost:; 2.
- `pickUpBanana` () `0x1000b8788` [1 use]: Banana pickUpForEating; 2.
- `pickUpBird` () `0x1000b889c` [1 use]: Bird pickUpForEating; a burnt bird eaten by a normal pygmy: setBehavior: PygmyEatBirdChoke (3); else 2.
- `pickUpCoconut` () `0x1000b845c` [1 use]: Coconut pickUpForEating; 2.
- `pickUpFish` () `0x1000b8a74` [1 use]: Fish pickUpForEating; a burnt fish eaten by a normal pygmy: setBehavior: PygmyEatFishChoke (3); else 2.
- `pickUpSwordFish` () `0x1000b8c5c` [1 use]: Swordfish pickUpForEating; 2.
- `pickUpWood` () `0x1000b8570` [1 use]: Incubates disease from the wood (InfectWood / InfectPygmyFromWood), wood pickUpForCarrying; 2.
- `possessedByGhostComplete` () `0x1000b6fac` [1 use]: Stops, upright, falls with gravity, selectable; ghost possessedByGhostComplete:; clears channel 1; 2.
- `pulledUnderComplete` () `0x1000b70d8` [1 use]: Stops, upright, falls, selectable; ghost pulledUnderByGhostComplete:; clears channel 1; 2.
- `resetCrapper` () `0x1000ba930` [1 use]: Clears mbFinishedCrapping; 2.
- `resetIgloo` () `0x1000ba918` [1 use]: Clears mbCrawledOutOfIgloo; 2.
- `runUnderSpider` () `0x1000b94f8` [1 use]: Same as runUnderTRex relative to the spider.
- `runUnderTRex` () `0x1000b8eb0` [1 use]: Runs (+/-200) toward the far side of the island; completes (2) past the safe run bound, turning to face the T-Rex; else 0.
- `sharkSwallow` () `0x1000be808` [1 use]: Shark swallowPygmy, sacrifice id SharkOnHead, setBehavior: PygmyDie; 2.
- `startBansheeScream` () `0x1000b7354` [1 use]: Marks the banshee scream started; every pygmy holdEarsFromBansheeScream:; 2.
- `startFishing` () `0x1000bd51c` [2 uses]: If the fishing pole is unavailable: pole startFishingIdle (2). Otherwise detaches from the pole (gravity/floor on, y = island y, clear channels 2..4) and PygmyIdle (3).
- `startSinking` () `0x1000bac8c` [1 use]: Gravity on, active underwater, selectable; starts Bait Master when flagged; 2.
- `startSleeping` () `0x1000ba630` [1 use]: Listener onPygmySleepingStart:; 2.
- `startStaringAtRainbow` () `0x1000aea68` [2 uses]: Stops moving; 2.
- `stopDanceComplete` () `0x1000ae740` [7 uses]: Sets mbStoppingDance; 2.
- `stopStaringAtKidnapping` () `0x1000a8bd4` [1 use]: stopStaringAtKidnapping; 2.
- `sunBurntComplete` () `0x1000bc9e0` [1 use]: Sacrifice id GlobalWarming. Ice island (4): over a frozen ice hole (x 290..350) meltIceHole:, else PygmySunBurntCreateSoloIceHole; elsewhere PygmySunBurntExplode; 2.
- `sunWalk` (minTime, maxTime) `0x1000bd214` [2 uses]: Like walk (random [minTime, maxTime], +/-50) but steers toward a frozen ice hole (x 290..350) in the sun; completes (2) on side collision, at the hole, or timeout; else 0.
- `takeCrap` () `0x1000bab7c` [1 use]: Stops waiting at the outhouse, no gravity/floor, moves behind it, incubates outhouse disease (InfectOutHouse / InfectPygmyFromOutHouse), outhouse takeCrap:; 2.
- `tripIntoZombieGrave` () `0x1000b80e8` [2 uses]: Zombie grave tripIntoGrave:fallForward: (forward when walking toward the grave); 2.
- `tugOfWarWithSquid` () `0x1000b6bbc` [1 use]: First time: no gravity/collision, random tentacle strength. Faces the tentacle; beyond +/-60 deg or 180 units: beheaded (ice hole beheadPygmy:, PygmyHeadRippedOffByIceMonster, 3). Else spring toward the touch plus tentacle pull (25, or 10 in the weak phase); 0.
- `uncrushed` () `0x1000be7f0` [2 uses]: Clears mbCrushed; 2.
- `vampireRun` (minTime, maxTime) `0x1000bda9c` [1 use]: First time: random duration [minTime, maxTime], runs toward the island centre at +/-100. Same daylight/sunset/victim checks as checkKillPygmies (3). Side collision or timeout: 2; else 0.
- `waitForNight` (minTime, maxTime) `0x1000bde8c` [2 uses]: First time: wait random [minTime, maxTime] (endless when absent). Night/eclipse: PygmyKillPygmies (3). Daylight: endless -> PygmyVampireSun (sacrifice id VampireInSunlight, 3), timed -> keeps waiting. Sunset: timed -> PygmyVampireSunset (3), endless -> keeps waiting. While waiting: 2 on timeout, else 0.
- `waitToCrap` (minTime, maxTime) `0x1000ba948` [2 uses]: First time: wait random [minTime, maxTime] or forever when absent. When the outhouse becomes available: stop, unselectable, behind the outhouse, PygmyTakeCrapAfterWaiting (3). Timeout: 2; else 0.
- `walkToBonFire` () `0x1000b85d4` [6 uses]: First time: faces the bonfire, +/-50. Completes (2) within 50 of it (stops); else 0.
- `walkToFishingPole` () `0x1000bd648` [1 use]: First time: no gravity/floor, velocity to reach (pole x+26.55, pole y+8.05) in 1 s. Completes (2) once above that y (stops); else 0.
- `walkToIceFishingPole` () `0x1000bc86c` [1 use]: First time: no gravity/floor, velocity to reach (hole x-67, hole y+5) in 1 s. Completes (2) once above hole y+5 (startFishingSitDown:); else 0.
- `watchShark` () `0x1000be880` [4 uses]: If some pygmy has a shark on its head: PygmyAlreadyReactForward/Backward depending on facing (0); else 2.
- `zombieIdle` (minTime, maxTime) `0x1000b833c` [2 uses]: First time: random duration [minTime, maxTime]. Fresh brains found: 3; timeout: 2; else 0.
- `zombieWalk` (minTime, maxTime, noFlip) `0x1000b81e0` [6 uses]: First time: random duration [minTime, maxTime]; chooses its own direction unless `noFlip`. Fresh brains found: 3. Side collision or timeout: 2; else 0.

### PygmyLogic(KonkeyDong)

- `konkeyDongFall` () `0x10003a220` [1 use]: Gravity on; never completes (0).
- `konkeyDongIntroComplete` () `0x10003a07c` [1 use]: Konkey Dong started, selectable, Kong statue pygmyIntroComplete; 2.
- `konkeyDongJumpUp` () `0x10003a160` [1 use]: First time: 0.2 s timer. y velocity 300 every frame; when the timer runs out: clears the jump touch, setBehavior: PygmyKonkeyDongJumpDown (3); else 0.
- `konkeyDongLevelJumpComplete` () `0x10003a118` [1 use]: Gravity on, clears running / level-transition / platform-collision flags; 2.
- `onStartKonkeyDongFall` () `0x10003a238` [1 use]: Gravity on; 2.

### PygmyLogic(MoronPests)

- `moronPestsIntroComplete` () `0x10011389c` [1 use]: Moron Pests started, snake pygmyIntroComplete, no gravity/collisions, x velocity and target velocity 500; 2.
- `moronPestsRun` () `0x100113958` [4 uses]: Never completes (0) (movement is driven by onGameFrame).
- `onEatenByMoronTurtleEatComplete` () `0x100113838` [1 use]: Moron snake logic stopMoronPests; 2.
- `onStartMoronPestsFall` () `0x100113884` [1 use]: Gravity on; 2.

### PygmyLogic(NoPee)

- `onNoPeeHitComplete` () `0x10018b6f0` [4 uses]: Clears channel 17 now; No Pee logic stopNoPee; 2.
- `onPygmyNoPeeRollComplete` () `0x10018b680` [1 use]: Clears mbNoPeeChangeDirection; 2.
- `waitUntilOffScreen` () `0x10018b698` [3 uses]: Completes (2) when y > 1000 or y < 0, else 0.

### PygmyLogic(OogaJump)

- `doodleJetPackComplete` () `0x1000085fc` [1 use]: y velocity 1000, jet pack off; 2.

### PygmyLogic(Story)

- `checkForStoryMode` () `0x100174e74` [1 use]: In story mode: ready, setBehavior: PygmyStoryModeIdle (3); else 2.
- `closeWordBalloon` () `0x1001744d4` [1 use]: closeWordBalloon; 2.
- `dieInStory` () `0x100174de4` [10 uses]: Marks dying in story, not story-ready; 2.
- `dieInStoryComplete` () `0x100174e10` [11 uses]: Marks the story death complete, disables labels; setBehavior: PygmyDie when resuming; 2.
- `openWordBalloon` (message) `0x1001744a8` [1 use]: openWordBalloon: `message`; 2.
- `storyModeFlick` () `0x100174d10` [1 use]: Velocity (random -100..100, 1200, -400), remembers the tumble start, falling; 2.
- `storyModeKnockOffIsland` () `0x100174c58` [2 uses]: Falls with velocity (random -200..200 in 0.01 steps, 600); 2.

### PygmyLogic(TheRuns)

- `onStartTheRunsFall` () `0x10013a358` [1 use]: Gravity on; 2.
- `onTheRunsExplodeComplete` () `0x10013a558` [1 use]: Crack hole stopTheRuns; 2.
- `onTheRunsSplashComplete` () `0x10013a50c` [1 use]: Crack hole stopTheRuns; 2.
- `theRunsFall` () `0x10013a4f4` [2 uses]: Gravity on; never completes (0).
- `theRunsJumpUp` () `0x10013a370` [1 use]: Jump strength k = (xVel-300)/300; first time timer 0.15+0.05k s; y velocity 300+100k each frame; when the timer runs out: clears touch, PygmyTheRunsJumpDown (3); else 0.
- `theRunsRun` () `0x10013a46c` [3 uses]: Clears skidding; on first reaching x velocity 600: max speed, PygmyTheRunsRunFast (3); else 0.

### PygmyLogic(Underwater)

- `addUnderwaterComplete` () `0x10011f8a4` [1 use]: Gravity on, active underwater, selectable; starts Bait Master when flagged; 2.
- `checkForLaserSharkDisabled` () `0x10011f844` [1 use]: If the laser shark is disabled: setBehavior: PygmyUnderwaterIdle; 2.
- `checkForLaserSharkEnabled` () `0x10011f7e4` [1 use]: If the laser shark is enabled: setBehavior: PygmyUnderwaterPanicIdle; 2.
- `fallIntoDrain` () `0x10011fd50` [1 use]: First time: no gravity/floor, velocity to reach (drain x, drain y+16) in 1 s. Completes (2) on passing that y (stops, drain pygmyFallIntoDrainComplete:); else 0.
- `onJellyfishSuffocateAttackComplete` () `0x10011cad8` [1 use]: Jellyfish onJellyfishSuffocateAttackComplete:, forget it; 2.
- `painDrainCanTurn` () `0x10011fd34` [4 uses]: Sets mbPainDrainCanTurn; 2.
- `panicSwim` (minTime, maxTime) `0x10011fb3c` [1 use]: As swim with y velocity 30 and x +/-30.
- `swim` (minTime, maxTime, noReset) `0x10011f944` [16 uses]: First time: random duration [minTime, maxTime], y velocity 50, x +/-10 random (inward at island ends). Timeout: stop, 2; else 0.

### RainbowLogic

- `lookForGushingPygmy` () `0x10002152c` [1 use]: findGushingPygmyLogic while none; never completes (0).
- `rainbowStrikeComplete` () `0x100021560` [1 use]: Clears the glow logics and path; 2.
- `stareAtRainbowComplete` () `0x1000215b4` [1 use]: Deactivates, removes the rainbow and rain models, every pygmy stopStaringAtRainbow; 2.
- `storyModeComplete` () `0x10002149c` [2 uses]: Clears the glow logics and path, deactivates and removes the rainbow; 2.
- `storyModeDrawMyRainbow` () `0x10002126c` [1 use]: First time: index 0. Draws one point of the saved rainbow path per call (drawRainbow:, appended to the path list); completes (2) after the last point, else 0.
- `storyModeDrawRainbow` () `0x100021384` [1 use]: First time: angle pi. Draws the arc point (240+200cos a, 110+150sin a) and steps the angle by -5 deg per call; completes (2) when the angle passes 0, else 0.
- `waitForGushingPygmy` () `0x100021510` [1 use]: Completes (2) when no pygmy is gushing, else 0.

### SharkLogic

- `checkCatchPygmy` () `0x1000293c0` [1 use]: Target pygmy in reach (y 60..150, within 10 of x-50*xScale): caughtByShark:, records plague, SharkCatchPygmyBaby / SharkCatchPygmy; otherwise listener onSharkSnap: with the miss; 2.
- `checkForPlaguePygmy` () `0x100029384` [5 uses]: If the eaten pygmy had the plague: setBehavior: SharkEatPlaguePygmy; 2.
- `die` () `0x100029bd0` [119 uses]: Removes the shark, ends a drag, releases an eaten pygmy (uneatenByShark:), resets state, re-enters the island if sharks are enabled and no story is active; 2.
- `disableJump` () `0x100029234` [3 uses]: Clears mbWaitingToJump; 2.
- `drag` (yOrigin, xOrigin) `0x1000296a8` [40 uses]: First time: spring on, gravity on, floor collision off, insert after DragDisplayMarker. Spring drag with rotation; never completes (0).
- `enableJump` () `0x10002921c` [2 uses]: Sets mbWaitingToJump; 2.
- `fall` (ignoreIsland) `0x100029914` [134 uses]: First time: gravity on. When y+originY < 10: lands in the water (clamped), setBehavior: SharkSplash (3); else 0.
- `launchIntoAir` () `0x1000298b8` [1 use]: First time: falling. Completes (2) once y > 960, else 0.
- `resetDropOnHead` () `0x100029208` [1 use]: Clears mbDropOnHead; 2.
- `setCatchPosition` () `0x100029248` [1 use]: If the target pygmy is at y 60..150: stops and moves to its x -50*xScale (2). Otherwise forgets it and setBehavior: SharkDie (3).
- `swim` (minTime, maxTime, noReset) `0x100029a50` [16 uses]: Within 16 of an unhooked bob that hooks it (hookShark:): stops, 2. Off screen: stops, 2. Else 0.

### SkinPackManager

- `getProductData` () `0x10005e0ec` [2 uses]: First time: returns 2 at once if every pack is bought or there is no internet; else requests product data. 0 while loading; on error shows "Loading Product Data Fail" and restarts its sequence with OpenSkinPackScreenNoInternet; then 2.
- `loadLocationTextureAtlas` () `0x100060038` [13 uses]: Listener onLoadLocationTextureAtlas; 2.
- `purchaseSkinPack` () `0x10005e8a4` [1 use]: First time: starts the purchase. 0 while purchasing; on error shows "Purchasing Skin Pack Fail" and restarts its sequence with OpenSkinPackScreenFailure; then 2.
- `purchaseSkinPackPanel` () `0x10006000c` [unused]: Calls purchaseSkinPack; 2.
- `setSkinPackPanel` () `0x100060030` [unused]: No-op; 2.
- `updateCustomizePanel` () `0x10005feec` [7 uses]: Calls updateCustomizePanel; 2.
- `updateInfoPanel` () `0x10005ff10` [4 uses]: Calls updateInfoPanel; 2.
- `updateListPanel` () `0x10005fec8` [5 uses]: Calls updateListPanel; 2.
- `updateNoInternetPanel` () `0x10005ff34` [1 use]: Calls updateNoInternetPanel; 2.
- `updateNoKeyChainPanel` () `0x10005ff58` [2 uses]: Calls updateNoKeyChainPanel; 2.
- `updateNotThreeOhPanel` () `0x10005ff7c` [1 use]: Calls updateNotThreeOhPanel; 2.
- `updatePurchaseFailurePanel` () `0x10005ffe8` [1 use]: Calls updatePurchaseFailurePanel; 2.
- `updatePurchasePanel` () `0x10005ffa0` [1 use]: Calls updatePurchasePanel; 2.
- `updatePurchaseSuccessPanel` () `0x10005ffc4` [1 use]: Calls updatePurchaseSuccessPanel; 2.

### SnowBallLogic

- `die` () `0x100010c80` [119 uses]: Marks in water, stops falling, ignores gravity, listener onSnowBallDie:; 2.
- `fall` (ignoreIsland) `0x100010b00` [134 uses]: Falls; on the island: listener onSnowBallMissPygmy:, 2; below y=30: onSnowBallMissPygmy:, setBehavior: SnowBallSplash (3); else 0.

### SpearLogic

- `die` () `0x100170998` [119 uses]: Removes the spear, ends a drag, resets state; re-activates when the T-Rex or spider is active and spears are enabled; 2.
- `drag` (yOrigin, xOrigin) `0x100170514` [40 uses]: First time: stop spin, gravity on, floor collision off, insert after DragDisplayMarker. Spring drag (k 100, damping 8), no rotation; never completes (0).
- `fall` (ignoreIsland) `0x1001706c8` [134 uses]: Falls (ignoreIsland disables floor collision); on the island 2; below y=20: stops spinning, SpearSplash (3); else 0.
- `fallFromHurricane` () `0x100170854` [9 uses]: Falls without floor collision; splash at y 72 (z < -277.12) or 20: SpearSplash (3); else 0.
- `slide` () `0x100170404` [20 uses]: First time: floor collision + gravity on. Below y=20: setBehavior: SpearSplash (3); else 0.

### SpiderLogic

- `die` () `0x10019b2a8` [119 uses]: Removes the spider, uneatPygmy, deactivates every spear; 2.
- `eatPygmy` () `0x10019b1d8` [3 uses]: Health +10 (max 100, health bar updated), clears channel 1, pygmy swallowedBySpider:; 2.
- `fall` (ignoreIsland) `0x10019b0e0` [134 uses]: Falls (ignoreIsland disables floor collision); on landing stops, side collision on, upright (2); else 0.
- `idle` (minTime, maxTime) `0x10019b6c4` [17 uses]: First time: random duration [minTime, maxTime], gravity on; 2 when done, else 0.
- `jumpOntoIsland` () `0x10019b088` [2 uses]: Velocity (-200, 800), faces left; 2.
- `walk` (minTime, maxTime, direction, endOfIsland) `0x10019b55c` [16 uses]: First time: duration random [minTime, maxTime], or 100 s when `endOfIsland` is present; side collision + gravity on. Side collision or timeout: 2; else 0.
- `whichWayDoIGo` () `0x10019b47c` [5 uses]: Random direction (forced right at x <= 60, left at x >= 400): SpiderWalk if facing it, else SpiderTurn; 3.

### SpiderTreeLogic

- `callForMom` () `0x100068ab8` [2 uses]: If spiders are enabled: spider callMom; 2.
- `dieFromSpiders` () `0x100068a44` [1 use]: Tree state 0, clears channel 1, trapped pygmy eatenByLittleSpiders:; 2.
- `onTransitionComplete` () `0x100068a30` [1 use]: Clears mbTransitionActive; 2.

### StormCloudLogic

- `startTransitionIn` () `0x1001a8944` [2 uses]: Waits (0) until the display object has a binary frame def; then starts startTransitionInLeft/Right by velocity sign (2).

### StoryManager

- `activateMyRainbow` () `0x10004cc20` [1 use]: Listener onStoryModeActivateMyRainbow; 2.
- `activateRainbow` () `0x10004cbf0` [1 use]: Listener onStoryModeActivateRainbow; 2.
- `deactivateRainbow` () `0x10004cc50` [1 use]: Listener onStoryModeDeactivateRainbow; 2.
- `loadLocationTextureAtlas` () `0x10004c148` [13 uses]: Listener onLoadLocationTextureAtlas; 2.
- `playNextStoryElementDef` () `0x10004c218` [1 use]: After the last element: fade-out overlay StoryFadeOutInit. Otherwise restarts mStorySequence with PlayStoryEnvironmentElementDef / PlayStoryCaptionElementDef / PlayStoryCharacterElementDef / PlayStoryEventElementDef by element type, or PlayStoryIncompatibleElementDef when a character/event state is incompatible with the environment; 2.
- `playStoryCaptionElementDef` () `0x10004c594` [1 use]: If the caption text is non-empty: opens the caption (CaptionLine1 text, wrap 412.5, overlay StoryCaptionOpen); 2.
- `playStoryCharacterElementDef` () `0x10004c714` [1 use]: Gets/creates the character pygmy (listener onStoryAddPygmy:); 0 until it isReadyForStoryMode. Applies its state (unless plain Idle without text) and opens a word balloon (DialogueLine1, wrap 104, StoryWordBalloonOpenLeft/Right by side) when there is dialogue; 2.
- `playStoryDef` () `0x10004c178` [1 use]: Element index 0, StoryTitle dynamic text = story title, title overlay logic StoryTitleInit; 2.
- `playStoryEnvironmentElementDef` () `0x10004c52c` [1 use]: Makes the current element the active environment and tells the listener onStoryEnvironmentChange:; 2.
- `playStoryEventElementDef` () `0x10004ca30` [1 use]: Restarts mStorySequence with "StoryModeEvent<event>" (format string cf_StoryModeEvent__); 3.
- `sharkSnap` () `0x10004cbc0` [1 use]: Listener onStoryModeSharkSnap; 2.
- `startCelebration` () `0x10004cb00` [1 use]: Listener onStoryModeStartCelebration; 2.
- `startEarthquake` () `0x10004cb60` [1 use]: Listener onStoryModeStartEarthquake; 2.
- `stopCelebration` () `0x10004cb30` [1 use]: Listener onStoryModeStopCelebration; 2.
- `stopEarthquake` () `0x10004cb90` [2 uses]: Listener onStoryModeStopEarthquake; 2.
- `storyCaptionElementDefComplete` () `0x10004c6b4` [1 use]: If a caption is open: first call starts StoryCaptionClose, then 0 until mbCaptionOpen clears; 2 when closed.
- `storyCharacterElementDefComplete` () `0x10004c93c` [1 use]: Balloon open: first call closes it (StoryWordBalloonCloseLeft/Right), then 0 until closed. Else resumes a non-Idle character from story state; 2.
- `storyDefComplete` () `0x10004c454` [1 use]: onPlayStoryDefComplete: YES; 2.
- `storyElementDefComplete` () `0x10004c510` [10 uses]: Advances to the next story element; 2.
- `storyEnvironmentElementDefComplete` () `0x10004c58c` [1 use]: No-op; 2.
- `storyStall` () `0x10004c47c` [9 uses]: First time: timer = the current element's stall time (+0xc). 2 when it runs out, else 0.
- `updateActiveCreateElementPanel` () `0x10004bfb0` [unused]: viewStoryElementDef: mActiveStoryElementIndex; 2.
- `updateAddMenu` () `0x10004c0dc` [1 use]: Calls updateAddMenu; 2.
- `updateBadStateScreen` () `0x10004c124` [1 use]: Calls updateBadStateScreen; 2.
- `updateClearConfirm` () `0x10004c094` [1 use]: Calls updateClearConfirm; 2.
- `updateCreateCaptionPanel` () `0x10004c028` [2 uses]: Calls updateCreateCaptionPanel; 2.
- `updateCreateCharacterPanel` () `0x10004c04c` [2 uses]: Calls updateCreateCharacterPanel; 2.
- `updateCreateEnvironmentPanel` () `0x10004c004` [2 uses]: Calls updateCreateEnvironmentPanel; 2.
- `updateCreateEventPanel` () `0x10004c070` [2 uses]: Calls updateCreateEventPanel; 2.
- `updateCreatePanel` () `0x10004bf8c` [11 uses]: Calls updateCreatePanel; 2.
- `updateCreateTitlePanel` () `0x10004bfe0` [2 uses]: Calls updateCreateTitlePanel; 2.
- `updateEditMainStoryConfirm` () `0x10004c0b8` [1 use]: Calls updateEditMainStoryConfirm; 2.
- `updateMyListPanel` () `0x10004bf68` [5 uses]: Calls updateMyListPanel; 2.
- `updateOptionMenu` () `0x10004c100` [2 uses]: Calls updateOptionMenu; 2.
- `updateWatchPanel` () `0x10004bf44` [6 uses]: Calls updateWatchPanel; 2.

### SwordFishLogic

- `die` () `0x1000c2c98` [119 uses]: Resets to the water (y 20, z -277.12, unit scale, water layer), ends a drag, clears state, setBehavior: SwordFishIdle; 2.
- `drag` (yOrigin, xOrigin) `0x1000c27e8` [40 uses]: First time: gravity on, floor collision off, moveToDragLayer. Spring drag with rotation; never completes (0).
- `fall` (ignoreIsland) `0x1000c29b0` [134 uses]: Falls; on the island 2; below y=20 in an enabled ocean: SwordFishSplash (3); else 0.
- `fallFromHurricane` () `0x1000c2b2c` [9 uses]: Falls without floor collision; splash height 86 (z < -277.12) or 20: SwordFishSplash (3); else 0.
- `slide` () `0x1000c26c0` [20 uses]: First time: floor collision + gravity on. In an enabled ocean below y=20: setBehavior: SwordFishSplash (3); else 0.

### TRexEggLogic

- `callForMom` () `0x100172470` [2 uses]: Stops falling/calling; T-Rex callMom; 2.
- `die` () `0x1001728d4` [119 uses]: Removes the egg, reports bowled pygmies, releases a crushed pygmy (uncrushedByTRexEgg:dontWait:), ends a drag, resets state, re-enters the island if eggs are enabled; 2.
- `drag` (yOrigin, xOrigin) `0x1001725d0` [40 uses]: First time: gravity on, floor collision off, insert after DragDisplayMarker. Spring drag (k 50, damping 6), no rotation; never completes (0).
- `fall` (ignoreIsland) `0x100172770` [134 uses]: Falls (ignoreIsland disables floor collision); on the island 2; below y=50: setBehavior: TRexEggSplash (3); else 0.
- `slide` () `0x1001724d0` [20 uses]: First time: floor collision + gravity on. Below y=50: setBehavior: TRexEggSplash (3); else 0.

### TRexLogic

- `chooseIdleSequence` () `0x100181060` [unused]: setBehavior: TRexGameIdle when challenge game 7 is active, else TRexWanderIdle; 3.
- `die` () `0x100180e8c` [119 uses]: Removes the T-Rex, uneatPygmy, deactivates every spear; 2.
- `eatPygmy` () `0x100180ce8` [3 uses]: Health +10 (max 100); swallows the eaten pygmy (clear channel 1, swallowedByTRex:); a plague pygmy kills the T-Rex (unselectable, listener onTRexKilledBySpear:, health bar deactivate, setBehavior: TRexEatPlaguePygmy, 3); else 2.
- `fall` (ignoreIsland) `0x100180bf0` [134 uses]: Falls (ignoreIsland disables floor collision); on landing stops, side collision on, upright (2); else 0.
- `idle` (minTime, maxTime) `0x100181398` [17 uses]: First time: random duration [minTime, maxTime], gravity on; 2 when done, else 0.
- `jumpOntoIsland` () `0x100180b98` [2 uses]: Velocity (-200, 800), faces left; 2.
- `walk` (minTime, maxTime, direction, endOfIsland) `0x100181230` [16 uses]: First time: duration random [minTime, maxTime], or 100 s when `endOfIsland` is present; side collision + gravity on. Side collision or timeout: 2; else 0.
- `whichWayDoIGo` () `0x1001810d8` [5 uses]: Random direction (forced at x 60 / 400): TRexWalk / TRexHuntWalk (game 7 active) if facing it, else TRexTurn / TRexHuntTurn; 3.

### TempleDoorLogic

- `checkForFatality` () `0x10009162c` [3 uses]: If the battle is complete: setBattleBehavior: BattleArenaFatalityResurrect when either side resurrects, else BattleArenaFatality; 2.
- `onBattleTurnComplete` () `0x1000915f0` [2 uses]: IdolManager onBattleTurnComplete; 2.
- `onGameOver` () `0x100091880` [1 use]: gameOver; 2.
- `onPygmyFallInsideComplete` () `0x100090af8` [16 uses]: Listener onOpenIdolBattleEnemyScreen; 2.
- `performResurrection` () `0x1000916f8` [1 use]: Respawns the resurrecting fighter(s) (spawnFromBattleOfTheGods) and IdolManager performResurrection; 2.
- `showBattleIdols` () `0x100090ffc` [1 use]: First time: drops the active player/enemy episode idols onto the battle pedestals (BattlePedestalDropIdol, BattlePedestalIdolSlot%d) or clears them (BattlePedestalClearIdol). Completes (2) once mbIdolsReady, else 0.
- `showBattleRealmEffectsResults` () `0x10009150c` [1 use]: First time: IdolManager showBattleRealmEffectResults. Completes (2) when areBattleRealmEffectResultsComplete, else 0.
- `showBattleRealmResults` () `0x100091494` [1 use]: IdolManager showBattleRealmResults; 2.
- `showBattleRealmResultsOnIdol` () `0x1000914d0` [1 use]: IdolManager showBattleRealmResultsOnIdol; 2.
- `showBattleSelectionPanel` () `0x1000917dc` [3 uses]: Multiplayer with a result to show: IdolManager showIdolBattleMultiplayerResult; else IdolScreenLogic openIdolBattleSelectionPanel; 2.
- `startPygmyAttacks` () `0x100091290` [1 use]: First time: both fighters startBattleAttack:opponentIdol:, IdolScreenLogic showBattleAttackStart. Eases the BattleArenaDisplayGroup x toward the loser's side; completes (2) when both fighters are ready (x reset to 0), else 0.
- `startPygmyPowerUp` () `0x1000911d0` [1 use]: First time: both fighters startBattlePowerUp:opponentIdol:; 2.
- `waitForBattleResultsComplete` () `0x1000915ac` [3 uses]: Completes (2) when GameUiManager allPointsComplete, else 0.
- `waitForFatalityComplete` () `0x10009157c` [2 uses]: Completes (2) when both fighters are ready, else 0.

### TheRunsPlatformLogic

- `onBurnPygmyComplete` () `0x1000d5d28` [1 use]: Clears channel 1, pygmy theRunsBurnPygmyComplete, listener onPygmyBurnPygmyComplete; 2.

### TileDrainLogic

- `catchPygmy` () `0x100118184` [2 uses]: Until a pygmy is caught: sucks in a pygmy within |dx|<50, 0<dy<130 (suckedIntoDrain:); never completes (0).
- `catchPygmyComplete` () `0x100118018` [1 use]: Starts the pain drain (platforms, platform y velocity 50, listener onPainDrainStart:pygmyLogic:, pygmy startPainDrain, plug plug:), moves before UnderwaterDisplayMarker, setBehavior: TileDrainTransitionIn; 2.
- `die` () `0x100118400` [119 uses]: Removes the display object; 2.
- `tileDrainSpikePygmyComplete` () `0x100117b04` [1 use]: Removes the platforms, moves after TsunamiDisplayMarker, pygmy dieFromPainDrain, clears channel 1 now, listener onPainDrainStop:; 2.
- `unplugTilePlug` () `0x100117f98` [1 use]: While lifting: marks unplugged and the tile plug unplug:touch:; 2.

### TilePlugLogic

- `die` () `0x100120914` [119 uses]: Removes the display object, setBehavior: TilePlugInit; 2.
- `drag` (yOrigin, xOrigin) `0x1001205ac` [40 uses]: First time: gravity on, origin = (xOrigin, yOrigin) (0 when absent). Spring drag (k 50, damping 6) with rotation; never completes (0).
- `fall` (ignoreIsland) `0x1001207c4` [134 uses]: Within 15 x / 50 y of a drain that can be plugged: unselectable, setBehavior: TilePlugInDrain, plugDrain: (3); else 0.

### TsunamiLogic

- `activateUnderwaterTemple` () `0x10000cad0` [4 uses]: If the island location state is 9: activates global logic 547; 2.
- `setAnimationBasedOnLastUnderwaterLocation` (coralReef, ruins) `0x10000ccc8` [1 use]: Same as setAnimationBasedOnUnderwaterLocation using the last island location state; 2.
- `setAnimationBasedOnUnderwaterLocation` (coralReef, ruins) `0x10000cc3c` [8 uses]: Plays `coralReef` when the island location state is 9 and it is present, else `ruins`; 2.
- `tsunamiActivateComplete` () `0x10000cb44` [3 uses]: Spear statue showSpear, listener onTsunamiActivateComplete:; 2.
- `tsunamiDeactivateComplete` () `0x10000cbac` [2 uses]: Clears channels 1, 2 and 4 now, listener onTsunamiDeactivateComplete:; 2.

### UnderwaterSceneryLogic

- `drift` () `0x10017c6ec` [3 uses]: Wraps the scenery around the 480 screen with refreshParams; never completes (0).

### VolcanoLogic

- `lavaRain` (minTime, minCount, maxTime, maxCount) `0x1000c7130` [4 uses]: First time: lava count = rand int in [minCount, maxCount]. Each blob waits random [minTime, maxTime] (0.001 s steps), then spawns a NewLava multimodel (LavaLogic) at a random island x, y 400, z -277.12 after LavaDisplayMarker, listener onEruptionComplete:; 2 after the last blob, else 0.

### WaterWingsLogic

- `die` () `0x100111188` [119 uses]: z -277.12, stops, removes the display object, ends a drag, resets state, re-enters the island if water wings are enabled; 2.
- `drag` (yOrigin, xOrigin) `0x100110cd0` [40 uses]: First time: gravity on, floor collision off. Spring drag; z-rot follows the drag angle; never completes (0).
- `fall` (ignoreIsland) `0x100110e88` [134 uses]: First time: unit scale; floor collision unless ignoreIsland. Rotation decays 50%/frame; on the island 2 (upright); below y=20: WaterWingsSplash (3); else 0.
- `fallFromHurricane` () `0x100111044` [9 uses]: Falls without floor collision; splash at y 80 (z < -277.12) or 20: WaterWingsSplash (3); else 0.
- `waterWingsSplashComplete` () `0x100110558` [1 use]: NoPeeLogic startNoPee: for the pygmy, clears channel 1 now, re-enters the island; 2.

### WhirlPoolLogic

- `die` () `0x100138398` [119 uses]: Listener onWhirlPoolDie:; 2.

### WoodLogic

- `die` () `0x10017bb4c` [119 uses]: reset, removes the display object, re-enters the island if wood is enabled; 2.
- `drag` (yOrigin, xOrigin) `0x10017b544` [40 uses]: First time: gravity on, floor collision off, moveToDragLayer. Spring drag with rotation; never completes (0).
- `fall` (ignoreIsland) `0x10017b898` [134 uses]: Falls (ignoreIsland disables floor collision); on the island 2; below y=50: WoodSplash (3); else 0.
- `fallFromHurricane` () `0x10017ba08` [9 uses]: Falls without floor collision; splashes below y 86 when z < -277.12, else below y 50 (table DAT_10029db08), clamping y: WoodSplash (3); else 0.
- `slide` () `0x10017b444` [20 uses]: First time: floor collision + gravity on. Below y=50: setBehavior: WoodSplash (3); else 0.
- `throwOnPile` () `0x10017b708` [1 use]: First time: no floor collision, gravity on. Arcs toward the bonfire's next log slot; when falling below it: stops and bonfire addWoodLogic: (2); else 0.

### XenuLogic

- `eatPygmy1Complete` () `0x100124c18` [1 use]: Clears channel 2 now and ejects pygmy 1 (ejectFromXenu); 2.
- `eatPygmy2Complete` () `0x100124c84` [1 use]: Clears channel 3 now and ejects pygmy 2; 2.
- `puppetPygmyComplete` () `0x100124cf0` [1 use]: Ejects both held pygmies (channels 2 and 3); 2.
- `spawnXenuComplete` () `0x100124bd8` [1 use]: Marks Xenu spawned, listener startApocalypse; 2.

### ZombieGraveLogic

- `startZombieIdle` () `0x1000c488c` [1 use]: Unburies, clears channel 1, trapped pygmy startZombieIdle, forgets it; 2.

## 6. Open questions

1. **Names of 1.48.2-only parameters.** Positions are exact (measured from the
   data and checked against the handlers); the 60 parameters marked `inferred`
   in `sequence-methods.json` (e.g. `tomId`, `babyPlagueId`, `portal`,
   `xenuSideLeft`, `closetSideLeft`, `normalPurification`, `dragXOffset`, the
   `minTime`/`maxTime` of `closetIdle`/`closetWalk`/`possessedByThetan...`,
   `list` of `choosePushSequence`/`setAnimationBasedOnLevel`, `id` of the overlay
   draws and `runSequenceIfInXenuRoom`, `soundId` of `playBackgroundLoop`) have no
   source XML; their names are descriptive guesses. Positions 15/16 of
   `setAnimationBasedOnPygmyType` are never read by any handler, so their meaning
   is a guess from the only item that fills them (`TemplePygmyDrag`).
2. **Generator order.** The per-method parameter order is neither alphabetical
   nor XML order; Python 2 dict, Java HashMap and CFDictionary orders were tested
   and do not reproduce it. Irrelevant for the port (positions are measured), but
   it means positions cannot be predicted for methods absent from the data.
3. **Listener order among equal `gameFrameOrder`.** `BCView::gameLoop` uses
   `-[NSMutableArray sortUsingFunction:context:]`; this spec assumes the sort is
   stable, so sequences step in the order they started playing. If CF's sort is
   not stable on device, the relative order of sequences within a frame (which
   matters only for the shared BCLibrary timers, 3.8) could differ.
4. **Unrecognised selectors** crash the original (3.7). The static check in 3.7
   covers sequences a class starts on itself; behaviours set on a logic by
   *another* class (e.g. the view controller calling `setBehavior:` on a bird)
   were not attributed, so they are covered only by "every method has an
   implementation somewhere".
5. **`BCLibrary::setAnimation:::`** keeps a library-owned BCAnimation per `id`
   that is never attached to a display object here; only `Garbage` uses it with an
   `id`, so its visible effect is undetermined (and irrelevant for shipped data).
6. **`getSequenceDefById:` with nil** would index the static buffer with an
   uninitialised length (undefined behaviour); not reachable with shipped data.
7. **Stack overflow.** `pushSequenceDef:` does not bound-check the 10-entry stack;
   static push depth in the data is at most 3.
