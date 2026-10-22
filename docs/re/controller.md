# PocketGodViewController — the game hub

`PocketGodViewController` (+ its category `Game`) is the centre of Pocket God 1.48.2: it owns the
`BCLibrary`, the four preference documents, every manager singleton, ~100 named world objects, the
island/location state machine, the sky, the god powers, the HUD menus, the minigame hosts and the
touch router, and it is the listener of almost every logic object. This document is the map of that
class: what each part does, which methods and ivars belong together, and how to split the port into
units. It does not transcribe every method; where a formula or constant matters it is given with the
function it comes from (`Class::selector @ address`).

Related specs (read them for the engine side):

- `docs/re/logic.md` — `BCLogic`, frame order, touch dispatch to logic objects, the generated
  logic-object table (`assets/data/logic-objects.json`, "table #N" below).
- `docs/re/sequences.md` — sequences, `BCSequence`, the item handlers (incl. the controller's).
- `docs/re/rendering.md` — display groups/markers, drawing, gradients.
- `docs/re/struct-xml.md` — display groups (`assets/data/display-groups.json`), episodes, links.

Deliverable next to this file: `tmp/re/controller-partition.json` — the method partition of section 9.

## 0. Conventions

- `PocketGodViewController::sel @ 0x…` is a method of the main class
  (`tmp/re/decomp/game/PocketGodViewController.c`); `PocketGodViewController(Game)::sel @ 0x…` is in
  the `Game` category (`PocketGodViewController+Game.c`); `PocketGodAppDelegate::…` is in
  `PocketGodAppDelegate.c`.
- `loc` = `mIslandLocationState` (section 4.1). "option `X`" = integer key `X` of the `Options`
  preferences (`mOptionPreferences`), "system `X`" = the `System` document.
- Positions are display coordinates (480×320 landscape, y up) unless called "view position" (UIKit
  points, y down, as stored in `BCTouch.pos`); conversions use `[group viewToDisplayObject:]`.
- Sequence names refer to `assets/data/sequences.json`; "runs `X`" means
  `[mSequence setSequenceDef:getSequenceDefById(X)]` (replaces whatever the main sequence was running)
  unless another sequence object is named.
- Item handlers (`sel:::`) return 2 (done), 0 (keep running) or 3 (sequence replaced) as described in
  `sequences.md`.
- dt = the frame delta passed to `onGameFrame:` (clamped to 1/15 s by `BCView::gameLoop`).

## 1. Overview

| Fact | Value |
|---|---|
| Superclass | `UIViewController` |
| Methods | 354 instance + 1 class (`+instance` @ 0x100069cdc returns the static `DAT_1007d1ed8`, set in `init`) in the main class; 321 in `(Game)`; 16 in `PocketGodAppDelegate` |
| Decompiled size | 593,916 + 357,658 + 15,988 bytes of C; machine code 123,544 + 74,836 + 2,752 bytes |
| Ivars | 581 (`instanceSize` 20277; two `id[1000]` touch arrays take 16000 bytes); 12 are never used |
| Protocols | 18 on the class, 101 on the category, 107 distinct (section 6) |
| Game-frame order | 50 (`gameFrameOrder` @ 0x1001585b4): runs after sequences (10), animations (20), pygmies (30) and other logics (40) |

Collaborators created and owned by the controller (all in `loadGame`, section 2.3): `BCLibrary`,
`BCPreferences` ×4, `SkinPackManager`, `GameServer`, `ChallengeManager`, `BonusPackManager`,
`StoryManager`, `DebugScreen`, `NewsManager`, `IdolManager`, `GameUiManager`, `MapScreen`,
`RainbowLogic`, `LocustLogic`, 10 `BCSequence`s; the god/tribe/guest personas are created by the boot
script (`loadPreferences:::`, section 2.5). `BCView` is created in `init`.

## 2. Lifecycle

### 2.1 Launch — `PocketGodAppDelegate`

`PocketGodAppDelegate` ivars: `mPocketGodViewController`, `mImageView` (loading image),
`mbStartupComplete`, `mbDashboardOpen` (never set; always NO), `mbModalViewOpen`, `window`.

`application:didFinishLaunchingWithOptions:` @ 0x100089324:

1. Start Crashlytics and Flurry (analytics; not ported), `[application setIdleTimerDisabled:YES]`.
2. `controller = [[PocketGodViewController alloc] init]` (section 2.2), `setDelegate:self`
   (`mDelegate`, protocol `PocketGodViewDelegate`), make it the window's root view controller,
   `makeKeyAndVisible`.
3. `showLoadingImage` @ 0x100089570: a `UIImageView` over the window showing the launch image
   (`Default`, `Default-568h.png` or `Default-Landscape~ipad.png` chosen by screen size, rotated −90°
   on 480/568-wide portrait frames and +90° on iPad). The port shows the same image until
   `onHideLoadingScreen`.
4. Start Chartboost (app id/signature at 0x1003c7b30/0x1003c7b50, delegate = controller) and cache
   the `"Home Screen"` interstitial (not ported; see 4.18).
5. Schedule a one-shot 0.1 s `NSTimer` → `loadGame` @ 0x100089c10 → `[controller loadGame]`.

Other delegate methods:

| Method | Behaviour |
|---|---|
| `applicationDidBecomeActive:` @ 0x100089a24 | if startup complete and no modal: `[BCView startRenderLoop]` |
| `applicationWillResignActive:` @ 0x100089a90 | if startup complete and no modal: `stopRenderLoop` |
| `applicationDidEnterBackground:` @ 0x100089afc | a modal view open → `[controller dismissModalViewController]`; then if startup complete: `stopRenderLoop` |
| `applicationWillEnterForeground:` @ 0x100089b94 | if startup complete: `[controller refreshNews]` (news teaser on load + reload `news.xml`), `startRenderLoop` |
| `applicationWillTerminate:` @ 0x100089c0c | nothing (preferences are autosaved, 2.8) |
| `onHideLoadingScreen` @ 0x100089c2c | `mbStartupComplete = YES`; remove and release the image view |
| `onModalViewWillAppear` / `onModalViewWillDisappear` @ 0x100089c8c / 0x100089cd0 | `mbModalViewOpen = YES/NO`; stop / start the render loop |
| `dealloc`, `window`, `setWindow:`, `mPocketGodViewController`, `setMPocketGodViewController:` | memory management and property accessors |

### 2.2 `init` @ 0x100069ce8

1. `DAT_1007d1ed8 = self` (`+instance`), `mAppStartTime = CFAbsoluteTimeGetCurrent()`.
2. Creates `mTextField` (a `UITextField`, delegate = self, clears on begin, no autocapitalisation,
   autocorrection off, return key type 9 = Done) and observes `UITextFieldTextDidChangeNotification`
   → `textDidChange:` (iOS keyboard plumbing for name editing, section 4.8.3).
3. `mbHideLoadingScreen = NO`; `mIslandLocationState = mLastIslandLocationState = -1`.
4. Creates the `BCView` (`mView`) with the application frame (rotated by 90° on iOS < 8,
   `NSFoundationVersionNumber <= 1047.25`), `setDelegate:self` (the controller receives the touch
   callbacks of section 5), makes it the view.

`loadView` @ 0x10006dad8 is `[self setView:mView]`. The game is locked to landscape-right:
`supportedInterfaceOrientations` @ 0x10006daf4 returns 8 (mask of `UIInterfaceOrientationLandscapeRight`),
`preferredInterfaceOrientationForPresentation` returns 3 (landscape right), `shouldAutorotate`
returns NO and `shouldAutorotateToInterfaceOrientation:` accepts only 3.

### 2.3 `loadGame` @ 0x10006a030, step by step

Called once from the 0.1 s timer. The order matters only where noted; everything is synchronous.

1. Constants: `mShakeRotVel = 1.3`, `mShakeRot = 0`, `mIslandMinXPos = 50`, `mIslandMaxXPos = 390`,
   counters `mPygmyCount`, `mLivePygmyCount`, `mVolcanoHitCount`, `mEarthquakeCount`,
   `mSleepingCount` = 0, stats/help/debug-screen flags NO, draw flags (lightning, rainbow, locust) NO,
   `mWeatherState = 0`, `mCloudGradientFade = 0.01`, `mTargetMenuButtonZRot = 90`, `mTargetMenuYPos = 90`.
2. Library (`mLibrary = [[BCLibrary alloc] init]`):
   `processAtlasTextureDefs:textureDefs:` for (`0x100602950`, `0x1005f9738`) and (`0x1007cbc08`,
   `0x1007c4988`); load atlas pixels `TextureAtlas0.raw` (def `0x100751f38`) and `TextureAtlas1.raw`
   (`0x100767d00`) with `FUN_100038d68`; if `[mView isWideScreen] || [mView isTallScreen]` also
   `WidescreenOverlayAtlas0.raw` (def `widescreenOverlayAtlas0`); `processSequenceDefs:` (list
   `0x100643a60`); `parseStructXml:` ×5 (roots `0x10071ab98`, `0x10064a270`, `0x1006fe1d0`,
   `0x1007162f0`, `0x1008162c0`); `parseBinAnim:@"Timelines" ext:@"bin"`. (Formats: textures, sequences,
   struct-xml and timelines specs.)
3. Preferences: `BCPreferences initWithId:secure:iCloud:` — `Persona` (NO, NO), `Options` (NO, NO),
   `System` (NO, NO), each with `setLegacyPreferenceFileName:@"Preferences.plist"` and
   `loadPreferences`; `Challenge` (secure YES, iCloud YES) + `loadPreferences`. Loading is
   asynchronous; the boot script waits for it (2.4).
4. `mSkinPackManager = [[SkinPackManager alloc] init]`, listener = self,
   `refreshSkinPackOption:@"Shark" skinObjectId:@"Shark" skinOptionId:@"Shark Skins" useFirstPurchased:YES`.
5. `[mLibrary createLogicClasses]` — builds the 632 table logics (logic.md §13). Then the controller
   caches table entries in ivars (full list in the ivar catalogue, section 8):
   - `mTempleDoorLogic` #374; apocalypse `mApocalypseTempleDoorLogic` #448, `mApocalypsePortalTempleDoorLogic`
     #449, `mApocalypseCalendarLogic` #447, `mApocalypseRoomFaderLogic` #440;
   - arrays (capacity 5): `mApocalypseGemPedestalLogicArray` #433–439, `mTimeTempleDoorLogicArray`
     #457, #456, `mLocustTempleDoorLogicArray` #470–472, `mDanceTempleDoorLogicArray` #475–476,
     `mPlagueTempleDoorLogicArray` #504–505, `mPurificationTempleDoorLogicArray` #508,
     `mClosetTempleDoorLogicArray` #515, `mXenuTempleDoorLogicArray` #519, empty `mApocalypseGemLogicArray`;
   - temple objects: lever #454, platform #453, locust queen #461, go-go cage #477, disco ball #478,
     disco lights #479, balloon bundle #480, balloon #481, DDE #500, plague fountain #503,
     purification fountain #507, thetan meter #514, couch #512, closet door #513, Xenu temple #517,
     Xenu #518;
   - the HUD/menu/screen buttons #561–631: 69 of the 71 are cached in ivars, all but #569
     (`DebugButton`: its display object is removed from
     `MenuDisplayGroup` and the ivar `mDebugButtonLogic` stays nil, so the debug screen is
     unreachable) and #624 (`OpenFeintButton`, not cached; its object does not exist in
     `CommunityMenuDisplayGroup`);
   - `mIdolPedestalLogicArray` #370–373 with `setPedestalIndex:` 1–4; `mGodIdolLogicArray` #99–102
     with `setSlot:` 0–3 and `setSkinTextureBitsDefArray:` `0x100569dc8`, `0x100569d98`,
     `0x100569d68`, `0x100569d38`;
   - `mMoronSnakeLogic` #51, `mMisfitOceanRockLogic` #49, `mWaterWingsLogic` #92, `mNoPeeLogic` #91,
     `mUnderwaterTempleLogic` #547.
6. Managers, each with the controller as listener: `GameServer` (+ leaderboard and achievement
   listeners), `ChallengeManager`, `BonusPackManager`, `StoryManager` (+ `loadStoryDefs`),
   `DebugScreen`, `NewsManager` (+ `loadNewsDefs`), `IdolManager`, `GameUiManager`, `MapScreen`.
7. `[mLibrary initLogicListeners]` (logic.md §13.3).
8. Display references: 36 display groups (`IslandDisplayGroup`, minigame groups, underwater groups,
   all menu groups), 20 display markers, 17 island/temple groups, `SunReflection` (an object of
   `IslandDisplayGroup`, retained and removed from its group), 11 gradient defs (`SunsetGradient`,
   `NightGradient`, `DayGradient`, `CloudyGradient`, `EclipseGradient`, `BigSunGradient`,
   `SmallSunGradient`, `Apocalypse{Sunset,Night,Day,Cloudy}Gradient`). Exact ids: ivar catalogue.
9. Empty `NSMutableArray`s (capacity 5): main/help/whats-new/stats UI arrays, `mLogicArray`,
   `mPygmyLogicArray`, `mFireworkLogicArray`, `mCoconutLogicArray`, `mWoodLogicArray`,
   `mCloudLogicArray`, `mDoodleCloudLogicArray`, `mSpearLogicArray`, `mUnderwaterSceneryLogicArray`,
   `mPiranhaLogicArray`, `mApeTorchLogicArray`, `mMonkeyLogicArray`, `mUnderwaterPowerTrailLogicArray`;
   `mJellyfishLogicArray` = table #93–97.
10. `[BCView addGameFrameListener:self]`.
11. Path containers: `mpLightningList`, `mpRainbowList` (`std::list<VECTOR4>`), `mpLocustVector`
    (`std::vector<VECTOR4>`). `mRainbowLogic` = `RainbowLogic` on `RainbowModel initWithId:@"Rainbow"
    pathList:mpRainbowList` (z −277.12), listener self; `mLocustLogic` = `LocustLogic` on
    `LocustModel @"Locust"` (z −277.12), listener self. Neither display object is inserted yet.
12. Sequences (delegate = self): `mSequence` = `PocketGod` (the boot script, 2.4); `mMiscSequence`,
    `mTheRunsSequence`, `mKonkeyDongSequence`, `mMoronPestsSequence`, `mNoPeeSequence`,
    `mBattleOfTheGodsSequence`, `mWeatherSequence`, `mMessageSequence` with no def.
13. Three warm-up `[mView gameLoop]` calls (each in its own autorelease pool) — the boot script
    starts executing here.
14. Measures the load time t = now − start; `[mView setDisplayLinkCount: t <= 13 s ? 1 : 2]`
    (CADisplayLink frame interval: 60 Hz, or 30 Hz on slow devices); schedules
    `[mView startRenderLoop]` after 0.1 s; `mbGameStarted = NO`.

### 2.4 The boot script `PocketGod` (287 items)

1. Items 0–278: `setLogic` (275 uses, the logic factory of 4.15) and a few `setBehavior` — creates
   the island scenery `MultiModelLogic`s, underwater objects, tsunami/ocean/storm-cloud layers,
   volcano, sky (stars/sun/moon), all island objects, the HUD buttons, the help/stats buttons, and
   forwards the manager-owned buttons (`manager` = `ChallengeManager`, `SkinPackManager`,
   `BonusPackManager`, `StoryManager`, `NewsManager`, `ChumBucket`) to those objects.
2. `waitForPreferencesLoadComplete` (0x10007bce8): returns 0 until all four documents report
   `isPreferencesLoadComplete`.
3. `loadPreferences` (0x10007bdcc), section 2.5.
4. `drawDisplayGroup PocketGod`, `drawWideScreenOverlayDisplayGroup WideScreenOverlay`,
   `drawTallScreenOverlayDisplayGroup TallScreenOverlay`, `stallFrames 4`.
5. `enableTouch` (0x100159998): `[mView setTouchActive:YES]` — before this, touches only reach
   `preStartTouchBeganXPos:yPos:` (empty).
6. `checkStartScreen` (0x100159c80): `mbAddPygmyAfterIdolStartScreen = YES`, `mbInitComplete = YES`;
   `checkForIdolStartScreen` (0x1001599f0) decides; runs `StartScreen` or `NoStartScreen`; returns 3.

`checkForIdolStartScreen`: reads `CFBundleVersion`, compares with system `WhatsNewVersion` (then
stores the new version). If option `ShowIdolStartScreen` == 0 and the version is unchanged →
`mbAddPygmyAfterIdolStartScreen = NO`, returns NO. Otherwise `mbIdolScreenOpen = YES`, runs
`CloseMenu`, `[mIdolManager openIdolStartScreen]`, returns YES.

- `StartScreen`: `stallFrames 3`, `hideLoadingScreen`, `checkShowAuthenticationViewController`
  (Game Center sign-in). The game proper starts when the idol start screen closes:
  `onIdolScreenClosed` → `StartGameAfterIdolScreen` (`checkForNewNews`, `drawDisplayGroup PocketGod`,
  `showMessage OpeningTitle at (0,160)`, `runSequence AddPygmy`).
- `NoStartScreen`: `hideLoadingScreen`, `checkShowAuthenticationViewController`, `stallFrames 2`,
  `showMessage OpeningTitle`, `checkForNewNews`, `runSequence AddPygmy` (`playSound AddPygmy`, `addPygmy`).

`hideLoadingScreen:::` (0x100159d40): `mbHideLoadingScreen = YES`, `hideLoadingScreen` →
`[mDelegate onHideLoadingScreen]`; if a URL is pending (`mLaunchUrl`) it is opened (4.18).
`checkForNewNews:::`: if `[mNewsManager hasUnreadNews]` → `mbNewsTeaserOpen = YES`, `openNewsTeaser`.

### 2.5 `loadPreferences:::` @ 0x10007bdcc

1. `[mIdolManager loadEpisodeState]`.
2. Option defaults (`setDefaultPreference:asInt:` on `Options`; the complete option list with
   meanings is section 4.4): `WeatherHazard 0`, `ShowIdolStartScreen 1`, `ShowEpisodeIcon 1`,
   `ShowBattleHelp 1`, `IslandLocation 5`, `SandIslandHazard 3`, `RockIslandHazard 1`,
   `RockIslandFeature 1`, `RockIslandOutHouseState 1`, `RockIslandTRexState 1`,
   `RockIslandSpearState 1`, `GraveyardIslandState 0`, `GraveyardIslandZombieGraveState 1`,
   `GraveyardIslandGhostState 1`, `GraveyardIslandSpiderWebState 1`, `GraveyardIslandSpiderState 1`,
   `GraveyardIslandSpearState 1`, `IceIslandState 1`, `IceIslandIceMonsterState 1`,
   `IceIslandIglooState 1`, `IceIslandIceHoleState 1`, `IceIslandIceSquidState 1`, `ApeIslandState 2`,
   `ApeIslandApeState 1`, `ApeIslandMonkeyState 1`, `MisfitIslandState 1`, `GodIslandState 0`,
   `GodIslandIdolsState 1`, `ApocalypseIslandState 0`, `UnderwaterHazard 2`,
   `UnderwaterCoralReefState 1`, `SharkState 1`, `BirdState 1`, `TemperatureState 1`,
   `HurricaneState 1`, `RainbowState 1`, `CrackHoleState 1`, `TheRunsState 1`, `CharlieState 1`,
   `KonkeyDongState 1`, `MoronPestsState 1`, `DrumHazard 1`, `GravityHazard 1`, `Mute 0`, `Vibrate 0`,
   `DisplayNames 1`, `ButtonLabels 1`; system default `MisfitIslandFound 0`.
   So a fresh install starts on **Ape island** (5).
3. Personas (`initWithId:defaultName:defaultDescription:preferences:` on the `Persona` document):
   `GodPersona` `God` ("GENE SIMIANS", "God of Thunder and Rocks that Roll") → `mGodPersona`;
   `TribePersona` `Tribe` ("TRIBE OOKGA CHAKA", "One Tribe, One World") → `mTribePersona`; six
   `PygmyPersona`s added with `addPygmyPersona:` — `Pygmy0` OOGA "The fearless leader.", `Pygmy1`
   BOOGA "Booga is gross and smells horrible.", `Pygmy2` NOOBY "Nooby is dazed and confused.",
   `Pygmy3` DOOBY "The happy-go-lucky islander.", `Pygmy4` KLIK "Klik likes to organize and build.",
   `Pygmy5` KLAK "Klak likes to break things."; guests (`addGuestPygmyPersona:`) `CharliePygmy`
   "Charlie"/"Winning!." personality 1 (`activateGuestPygmyPersona:` if option `CharlieState` == 1) and
   `TomPygmy` "Tom"/"Show me the money!." personality 2. `[mPersonaPreferences setPreferencesListener:self]`.
4. `setSharkState:SharkState startup:YES`, `setBirdState:BirdState startup:YES`,
   `setWeatherState:WeatherHazard startup:YES`, `updateIslandStateAtStartup` (4.3), which ends with
   `setIslandLocationState:IslandLocation startup:YES`.
5. Button start behaviours for the options that are 1: `StartGravityOn`, `StartTemperatureOn`,
   `StartHurricaneOn`, `StartRainbowOn`, `StartCrackHoleOn`, `StartIdolsOn` (option `IdolsState`, which
   nothing else writes), `StartTheRunsOn`, `StartCharlieOn`, `StartKonkeyDongOn`, `StartMoronPestsOn`,
   `StartDrumOn`, `StartNamesOn`, `StartLabelsOn`; Mute: 1 → `BCSound mute:YES`, `StartMuteOn`,
   `stopAllSounds`; 0 → `mute:NO`, `StartMuteOff`, `playBackgroundLoop:nil`; Vibrate 1 →
   `muteVibrate:YES`, `StartVibrateOn` (else `muteVibrate:NO`).
6. If `checkAllAchievementsAndLeaderboards` was requested before the personas existed
   (`mbCheckAllAchievementsAndLeaderboards`), run it now. Returns 2.

`onPreferencesChanged:` @ 0x10007bd80 (BCPreferences listener, iCloud merges):
`[mGodPersona refreshFromPreferences]`, `[mTribePersona refreshFromPreferences]`.

### 2.6 Game start

The first `onGameFrame:` sets `mbGameStarted = YES` (written, never read). The play session begins
when the boot script's `AddPygmy` adds the first pygmy (4.14). Nothing else distinguishes "started".

### 2.7 Background, foreground and modal views

- Background: the render loop stops (no frames, no touches, no sequence time); a presented modal
  is dismissed first. Foreground: `refreshNews` then `startRenderLoop` (which resets `mLastTime` so the
  first frame gets dt = 1/15 s).
- `presentModalViewController:` @ 0x10006d2dc (Game Center UI, from `GameServer`/`IdolManager`/
  `ChallengeManager`): once (`mbModalViewControllerActive`), `cancelAllTouches`,
  `[mView clearTouchArrayForModalViewController]`, `[mDelegate onModalViewWillAppear]` (render loop
  stops), present. `dismissModalViewController` @ 0x10006d388 reverses it
  (`onModalViewWillDisappear`, render loop restarts).

### 2.8 Save points

There is no explicit save. Every `setPreference:as…:` marks its document dirty
(`BCPreferences::onPreferencesChanged` @ 0x10016d3b4) and `BCPreferences::onGameFrame:` (order 40)
saves a dirty document at most every 2 s (`onGameFrameManageSavingPreferences:` @ 0x10016d3f8:
`mLastSaveTime += dt; if > 2: reset, save` — the secure `Challenge` document on a thread). The
controller writes:

| Document / key | Written by |
|---|---|
| Options `IslandLocation` | `setIslandLocationState:startup:` (every location change) |
| Options feature keys (section 4.3) | each `set…State:startup:` (also at startup, rewriting the same value) |
| Options `WeatherHazard` | `setWeatherState:startup:` (not at startup), `stopDragClouds` |
| Options `SharkState`, `BirdState` | `setSharkState:startup:`, `setBirdState:startup:` |
| Options toggles `KonkeyDongState`, `MoronPestsState`, `CrackHoleState`, `TheRunsState`, `CharlieState`, `DrumHazard`, `TemperatureState`, `HurricaneState`, `RainbowState`, `Mute`, `Vibrate`, `GravityHazard`, `DisplayNames`, `ButtonLabels` | `onButtonPress:touch:` (4.7); `DisplayNames` also by story start/end (4.10) |
| System `WhatsNewVersion` | `checkForIdolStartScreen` (each launch) |
| System `MisfitIslandFound` = 1 | `onDreamBubbleChangeIslandLocationState:` |
| System `InfoShown`, `BonusInfoShown` = 1 | first `openSkinPackManager` / `openBonusPackManager` |
| System `<feint achievement id>` = 1 | `checkFeintAchievements:score:` (4.12) |
| Persona document | personas (`setName:`, `setDescription:` from the stats pages, `setGodType:`, sacrifices, gifts, moods, achievements) |
| Challenge document | `ChallengeManager`, `IdolManager` (episode state) |

### 2.9 Memory warning, dealloc

`didReceiveMemoryWarning` @ 0x10006db18 only logs. `dealloc` @ 0x10006db88 releases everything
(not needed by the port; Rust ownership replaces it). `.cxx_construct` @ 0x1000885a0 is the empty
C++ ivar constructor.

## 3. The main loop

### 3.1 Frame order

There is no `update`/`drawOpenGL` in the controller. `BCView::gameLoop` (logic.md §4.1) per frame:
dt (≤ 1/15 s) → queued atlas loads → accelerometer → **all queued touch events are dispatched to the
controller** (section 5) → `onGameFrame:` of every listener sorted by `gameFrameOrder` (sequences 10,
animations 20, pygmies 30, logics/`BCPreferences`/`GameUiManager` 40, **controller 50** and
`StoryManager` 50, `NoPeeLogic` 59, moron/no-pee map 60, `GameServer` 70) → draw the active display
group (plus wide/tall overlays) and present. The controller's own sequences (`mSequence`,
`mMiscSequence`, …) are `BCSequence` listeners at order 10, so their items run before
`onGameFrame:` in the same frame.

### 3.2 `PocketGodViewController(Game)::onGameFrame:` @ 0x1001585bc

In order:

1. `mbGameStarted = YES`.
2. Drum timer: `mTapTouchTime += dt`; if no story is active and `mTapTouchTime > 1.0` →
   `mTapTouchCount = 0`, `stopCelebration` (so the dance stops 1 s after the last drum tap; it is
   called every frame while idle and returns early when no celebration runs).
3. Bat timer: `mBatTapTouchTime += dt`; `> 1.0` → `mBatTapTouchCount = 0`.
4. Island vertical scroll target (`mTargetIslandYPos`, `mTargetIslandYPosVel`):
   - loc 2 or 9 (underwater) or `mbIslandLocationTransition`: target = 0.
   - else if a pygmy (`mDraggedPygmyLogic`) or an apocalypse gem (`mDraggedApocalypseGemLogic`) is
     dragged — **drag auto-scroll**: vel = 0; `q = [IslandDisplayGroup viewToDisplayObject:mTouchScreenPos]`,
     `y = [IslandDisplayGroup yPos]`; target = y; if `q.y + y > 256`: target = y − 600·dt; if
     `q.y + y < 192`: target += 600·dt; clamp target to [−260, 0].
   - else if Ooga Jump runs (`mDoodleJumpPygmyLogic`) — **doodle camera**: `y` = island yPos,
     `p` = pygmy position. Not falling: `t = 128 − p.y`; if `t < y`: `[ChallengeManager setGameScore:4
     score:(int)((p.y − 128)·0.3125)]` and target = t (else target unchanged). Falling
     (`mbDoodleJumpFall`): target = `p.y − 32 >= 0 ? 32 − p.y : 0` and the island y is set to it
     immediately. Then target = min(target, 0); crossing −480 downward (`y > −480`, target < −480) →
     `showDoodleDisplayGroup:`, upward (`y < −480`, target > −480) → `hideDoodleDisplayGroup:`;
     vel = (target − old target)/dt; `recycleDoodleClouds:−y`; `[pygmy doodleCheckOffScreen:−y]`.
   - else (idle): vel += 3000·dt; target += vel·dt; target > 0 → target = 0 (springs back up to 0;
     vel is not reset while clamped).
5. Island y easing: `y = y + (target − y)·0.3` computed **in double** (`0.3` is a double constant at
   0x10029da98) and stored as float into both `IslandDisplayGroup.pos.y` and `DoodleDisplayGroup.pos.y`.
6. Minigame cameras (each display group's `pos`; `v` = minigame pygmy `posVel`, `p` = its position):
   - The Runs (`mbTheRunsMoveCamera`): `x += (p.x + x > 70 ? −1.2·v.x : −v.x)·dt`.
   - Moron Pests (`mbMoronPestsMoveCamera`): `x += (p.x + x < 340 ? −0.9·v.x : −v.x)·dt`.
   - No Pee (`mbNoPeeMoveCamera`, pygmy exists and not hibernating): `x += −1.2·v.x·dt` when
     (`p.x + x > 200` and swimming right) or (swimming left and `p.x + x < 280`), else `x −= v.x·dt`;
     `ty = clamp((p.y − 150)/700, 0, 1)·−680` (−680 when the ratio > 1); `y += 2·(ty − y)·dt`.
   - Konkey Dong (`mbKonkeyDongMoveCamera`): if moving to the next level
     (`mbKonkeyDongMoveCameraToNextLevel`): `x += −0.07·x`, `y += (−mKonkeyDongLevelYPos − y)·0.07`;
     when `|−mKonkeyDongLevelYPos − y| <= 0.1` → flag cleared and `[KongStatue levelTransitionComplete]`.
     Otherwise follow the pygmy: if `p.x + x` leaves [200, 280] → `x = (280 or 200) − p.x`; clamp
     `x <= 0` and `x >= −(levelWidth − 480)` (`levelWidth` = float at +0x30 of the
     `KonkeyDongLevel` struct returned by `[KongStatue konkeyDongLevel]`, encoding
     `{KonkeyDongLevel=@@@@@@ff{?=ffff}{?=ffff}}`: the first float after six ids = `mLevelWidth`,
     struct-xml.md §10); if `p.y + y` leaves [150, 170] → `y = (170 or 150) − p.y`; clamp
     `y <= −mKonkeyDongLevelYPos`.
7. Menu easing (if `mMenuButtonLogic` exists): menu button `rot.z += (mTargetMenuButtonZRot − rot.z)·0.5`;
   `MenuDisplayGroup.pos.y += (mTargetMenuYPos − y)·0.5`; when y == 90 exactly the group is removed
   from its parent, otherwise (re)inserted after `MenuDisplayMarker`. (Targets: 90/90 closed, 0/0 open;
   the exact-equality test relies on f32 convergence: halving the gap reaches exactly 90.0 after
   about 25 frames, so the port must ease in f32 without fused multiply-add, or remove the group
   when the gap is below one ulp of 90.)
8. `checkEclipse` (3.3), `updateGradients` (3.4).

### 3.3 `checkEclipse` @ 0x100159098

If a boulder is being dragged, it is not night and `mSunState == 0`: `d` = distance between the
boulder and the sun display objects; if `d < 20`: `e = 1 − d/20`; when `e > 0.5` and the previous
eclipse target was ≤ 0.5 → runs `Eclipse` (sound + episode event `MakeEclipse`); target = e;
`[mBoulderLogic setEclipsePercent:e]`, `[mSunLogic setEclipsePercent:e obstructionLogic:mBoulderLogic]`.
In every other case: eclipse target 0, `[mBoulderLogic setEclipsePercent:0]`,
`[mSunLogic setEclipsePercent:0 obstructionLogic:nil]`.

### 3.4 `updateGradients` @ 0x1001592b8

The sky is the `Gradient` object of `IslandDisplayGroup` (a `BCGradient`, rendering.md). Seven
weights ease toward their targets: cloud, sunset, night, eclipse, big sun, small sun, apocalypse —
`w += (target − w)·0.13`, snapping to the target when `|w − target| < 0.01`. Only when at least one
weight differs from its target (and all 11 gradient defs exist):

1. `setGradient: DayGradient`;
2. apocalypse ≠ 0 → `fadeGradient:ApocalypseDayGradient percent:apocalypse`;
3. cloud ≠ 0 → `fadeGradient:CloudyGradient percent:cloud`; and if apocalypse ≠ 0 →
   `ApocalypseCloudyGradient` with `apocalypse·cloud`;
4. big sun ≠ 0 → `BigSunGradient`; small sun ≠ 0 → `SmallSunGradient`;
5. sunset ≠ 0 → `SunsetGradient` (+ `ApocalypseSunsetGradient` with `apocalypse·sunset`);
6. night ≠ 0 → `NightGradient` (+ `ApocalypseNightGradient` with `apocalypse·night`);
7. eclipse ≠ 0 → `EclipseGradient`;
8. clear colour = `[gradient gradientColorAtIndex:0]` RGB, alpha 1 (`BCView setClearColorRed:…`).

(The percent argument of each fade is the weight tested just before it — verified in the disassembly.)

Who sets the targets: cloud — `setWeatherState:startup:` (startup storm 1, clear 0), the
`setCloudGradientFade` item (`StartStorm` 1, `StopStorm` 0); sunset/night — `startNight`,
`startDay`, `startSunset`, `dragSun:`; big/small sun — `setSunState:`; eclipse — `checkEclipse`;
apocalypse — `startApocalypse` (1), `endApocalypseComplete(Now)` (0).

### 3.5 What is drawn

The controller never draws; it shapes the display tree that `BCView` draws (rendering.md). The
active root is `PocketGod` (set by `drawDisplayGroup` items; help/stats screens switch to their own
groups). The controller moves groups in and out of the tree:

- `IslandDisplayGroup` (sky gradient, sun/moon/stars, clouds, ocean layers, volcano, the shared
  world objects, pygmies) sits at `IslandRootDisplayMarker`; the current island/temple group is
  inserted at `IslandDisplayMarker` (temples replace the whole island group at the root marker,
  4.17). Underwater groups go to `UnderwaterRealmDisplayMarker` and `UnderwaterDisplayGroup` to
  `UnderwaterDisplayMarker` during transitions (4.2).
- Minigames swap `IslandDisplayGroup` (or `UnderwaterDisplayGroup` for Pain Drain) for their own
  group at the same marker and re-parent the playing pygmy after their pygmy marker (4.16).
- `MenuDisplayGroup` after `MenuDisplayMarker` (removed when fully closed); the open category menu
  group after `CategoryMenuDisplayMarker`, followed by `MenuLabelsDisplayGroup` when button labels are
  on (4.7). The `Message` model goes after `MenuDisplayMarker`.
- Fireworks after `FireworksDisplayMarker`; clouds after `CloudBehindDisplayMarker`; god-power
  models (lightning, rainbow trail, hurricane, finger glows) are added to `IslandDisplayGroup`
  (whirlpool/underwater power to `UnderwaterDisplayGroup` / before `UnderwaterPygmyDisplayMarker`).

## 4. Game state and features

### 4.1 Locations (`mIslandLocationState`)

18 locations. `activateIslandLocationState:` @ 0x100080fa8 sets the walk bounds
(`islandMinXPos`/`MaxXPos`/`CenterXPos`), the horizon and the per-location script (run on
`mMiscSequence`), makes the location's objects `enterIsland:startup`, attaches the location's display
group and loads its atlas (`loadLocationTextureAtlas` for islands, `showApocalypseTempleDisplayGroup`
for temples). Start scripts are `triggerEpisodeEvent <event>; playBackgroundLoop <loop>`.

| loc | Location | Display group → marker | Menu group (category 1) | Start script: episode event, loop | Atlas (`screenAtlas0`) | min / max / centre x | horizon |
|---|---|---|---|---|---|---|---|
| 0 | Sand island | `SandIslandDisplayGroup` → `IslandDisplayMarker` | `SandIslandMenuDisplayGroup` | `StartSandIsland`: —, `SandBackgroundLoop` | `SandTextureAtlas0` | 50 / 390 / 220 | 160 |
| 1 | Rock island | `RockIslandDisplayGroup` | `RockIslandMenuDisplayGroup` | `RockIsland`, `RockBackgroundLoop` | `RockTextureAtlas0` | 50 / 414 / 232 | 160 |
| 2 | Underwater (ruins) | `UnderwaterRuinsDisplayGroup` → `UnderwaterRealmDisplayMarker` | `UnderwaterMenuDisplayGroup` | `Underwater`, `UnderwaterBackgroundLoop` | `UnderwaterTextureAtlas0` | unchanged | unchanged |
| 3 | Graveyard island | `GraveyardIslandDisplayGroup` | `GraveyardIslandMenuDisplayGroup` | `GraveyardIsland`, `GraveyardBackgroundLoop` | `GraveyardTextureAtlas0` | 32 / 448 / 240 | 160 |
| 4 | Ice island | `IceIslandDisplayGroup` | `IceIslandMenuDisplayGroup` | `IceIsland`, `IceBackgroundLoop` | `IceTextureAtlas0` | 50 / 410 / 230 | 160 |
| 5 | Ape island ("Ape Mountain") | `ApeIslandDisplayGroup` | `ApeIslandMenuDisplayGroup` | `ApeMountain`, `ApeIslandLoop` | `ApeIslandTextureAtlas0` | 80 / 400 / 240 | **120** |
| 6 | Misfit island | `MisfitIslandDisplayGroup` | `MisfitIslandMenuDisplayGroup` | `MisfitIsland`, `MisfitIslandLoop` | `MisfitTextureAtlas0` | 32 / 420 / 226 | 160 |
| 7 | God island | `GodIslandDisplayGroup` | `GodIslandMenuDisplayGroup` | `GodIsland`, `GodIslandLoop` | `GodTextureAtlas0` | 32 / 448 / 240 | 160 |
| 8 | Fail boat | `FailBoatOceanDisplayGroup` | `FailBoatMenuDisplayGroup` | `FailBoat`, `GodIslandLoop` | `BoatTextureAtlas0` | 32 / 448 / 240 | 160 |
| 9 | Underwater coral reef | `UnderwaterCoralReefDisplayGroup` → `UnderwaterRealmDisplayMarker` | `UnderwaterCoralReefMenuDisplayGroup` | `UnderwaterCoralReef`, `CoralReefTheme` | `UnderwaterCoralReefTextureAtlas0` | unchanged | unchanged |
| 10 | Apocalypse island | `ApocalypseIslandDisplayGroup` | `ApocalypseIslandMenuDisplayGroup` | `ApocalypseIsland`, `ApocolypseIslandLoop` | `ApocalypseTextureAtlas0` | 32 / 448 / 240 | 160 |
| 11 | Time temple | `TimeTempleDisplayGroup` → `IslandRootDisplayMarker` (replaces `IslandDisplayGroup`) | `TimeTempleMenuDisplayGroup` (empty) | `TimeTemple`, `TimeRoomLoop` | `TimeTempleTextureAtlas0` | 80 / 400 / 240 | 160 |
| 12 | Locust temple | `LocustTempleDisplayGroup` (same) | `LocustTempleMenuDisplayGroup` | `LocustTemple`, `LocustLoop` | `LocustTempleTextureAtlas0` | 80 / 400 / 240 | 160 |
| 13 | Dance temple | `DanceTempleDisplayGroup` | `DanceTempleMenuDisplayGroup` | `DanceTemple`, `DanceLoop1` | `DanceTempleTextureAtlas0` (+ skin channel 2) | 80 / 400 / 240 | 160 |
| 14 | Plague temple | `PlagueTempleDisplayGroup` | `PlagueTempleMenuDisplayGroup` | `PlagueTemple`, `PlagueLoop` | `PlagueTempleTextureAtlas0` | 65 / 415 / 240 | 160 |
| 15 | Purification temple | `PurificationTempleDisplayGroup` | `PurificationTempleMenuDisplayGroup` | `PurificationTemple`, `PurificationLoop` | `PurificationTempleTextureAtlas0` | 32 / 448 / 240 | 160 |
| 16 | Closet temple | `ClosetTempleDisplayGroup` | `ClosetTempleMenuDisplayGroup` | `ClosetTemple`, `ClosetRoomLoop` | `ClosetTempleTextureAtlas0` | 45 / 420 / 232.5 | 160 |
| 17 | Xenu temple | `XenuTempleDisplayGroup` | `XenuTempleMenuDisplayGroup` | `XenuTemple`, `XenuRoomLoop` | `XenuTempleTextureAtlas0` (+ skin channel 3) | 55 / 448 / 251.5 | 160 |

After the switch (all locations): `mSunTransitionYPos = horizon`, `mSunUnderHorizonTriggerYPos =
mMoonUnderHorizonTriggerYPos = horizon − 25`, `mSunUnderHorizonYPos = mMoonUnderHorizonYPos = horizon − 40`,
`mSunOverHorizonYPos = mMoonOverHorizonYPos = horizon + 40` (the "over" values are never read). If
the new location is not underwater but the previous one was, `IslandDisplayGroup` is re-attached at
`IslandRootDisplayMarker`.

Objects entered per location (`enterIsland:startup` unless noted; "0+startup → exit" = when the
toggle is 0 and this is the startup activation, `exitIsland:NO` is sent instead so the object starts
hidden):

| loc | Objects |
|---|---|
| 0 | Sand toggle: 1 boulder; 2 fishing pole + fish (skipped when coming from Ice with Ice=1 or Rock with Rock=1); 3 magnifying glass + ant hill; 4 water wings (carry rule). Always: coconut #0 moved to (163, 205) + `enterIsland`, volcano, octo statue |
| 1 | Rock fishing (skipped from Ice=1 / Sand=2); feature 2 → T-Rex egg, 1 → bonfire + 3 wood; outhouse (0+startup → exit); T-Rex; spears; `PlagueSign` (table #509) behaviour `PlagueSignShow` if `[ApocalypseCloudLogic doesDiseaseExist]` or `doesPlagueBodyExist`, else `PlagueSignHide` |
| 2 | Underwater toggle 3 → tile drain + tile plug, 2 → chum bucket, 1 → anchor; if not coming from 9: every pygmy `waitUnderwater`; `[ApocalypseCloudLogic cureAllDisease]` |
| 3 | water wings (Graveyard=1, carry rule); zombie grave (0+startup → exit); spider tree when spider web = 1 (0+startup → exit); spider; spears |
| 4 | Ice=2 water wings (carry), Ice=1 fishing (skipped from Rock=1 / Sand=2); ice monster, igloo, ice hole (each 0+startup → exit); ice squid → `[iceHole enableIceSquid]` (0+startup → `disableIceSquid`); glacier, ice tree |
| 5 | Ape=2 banana tree + Kong statue; Ape=1 altar + bongo + wench; giant ape (0+startup → exit); both ocean layers and the sun reflection are removed from the tree, `Gradient.pos.y = 60`, sun and moon y −40 unless at 245; both torches; `[shark exitIslandNow]` |
| 6 | Misfit=2 water wings (carry), Misfit=1 moron snake; misfit ocean rock |
| 7 | God=1 water wings (carry); idols=1 → skin channel 1 + 4 god idols; skin channel 1 again; 4 idol pedestals; temple door |
| 8 | nothing |
| 9 | coral reef=1 → 5 jellyfish; if not coming from 2: pygmies `waitUnderwater`; `cureAllDisease` |
| 10 | Apocalypse=1 water wings (carry); 7 gem pedestals, calendar, temple door, portal door |
| 11 | 2 doors, lever, platform |
| 12 | 3 doors, locust queen |
| 13 | 2 doors, go-go cage, disco ball, disco lights, balloon bundle, Dance Dance Execution |
| 14 | 2 doors, plague fountain |
| 15 | door, purification fountain |
| 16 | door, thetan meter, couch, closet door |
| 17 | door, Xenu temple, Xenu |

Carry rule (water wings and fishing): the shared object is not re-entered (and on leaving not
exited) when the previous/next location also has the same feature enabled — the object stays in
the world across the trip. Water-wings states: Sand 4, Graveyard 1, Ice 2, Misfit 2, God 1,
Apocalypse 1; fishing states: Sand 2, Rock 1, Ice 1.

`deactivateLastIslandLocationState` @ 0x10007ed70 is the mirror: first every pygmy
`onDeactivateIslandLocationState:old newIslandLocationState:new`; then by the old location the
objects exit (`exitIslandNow` for the carried/draggable ones, `exitIsland`/`exitIsland:YES` for the
rest) and the old group is removed. Specifics: Sand also removes the second (split) coconut and calls
`checkCountCoconutBounces`; Underwater/coral reef going to a non-underwater location stop the
whirlpool (`stopWhirlPoolNow`, finger trail removed), the coral reef also ends the underwater power
stroke and clears its trail puffs, and pygmies `stopUnderwater`; Ape restores both ocean layers
(after `FireworksDisplayMarker`), the sun reflection when sunset, `Gradient.pos.y = 120`, sun/moon
y +40 (max 245), torches/monkeys/banana exit and the shark re-enters if `SharkState` == 1 (note: the
giant ape is only exited in the `ApeIslandState == 1` branch); temples `stopHurricane` and end with
`hideApocalypseTempleDisplayGroup`. Tail: entering 2/9 from a surface location → the bat flies off,
the hurricane stops now (finger trail removed) and `IslandDisplayGroup` is removed.

### 4.2 Changing location

`setIslandLocationState:startup:` @ 0x100083b64 (the only entry point):

1. Category menu 1 (the island menu) open → `setCategoryMenuState:0`.
2. Same location and not startup → return.
3. A tsunami transition still running → finish it now: `deactivateLastIslandLocationState`,
   `activateIslandLocationState:NO`.
4. `mLastIslandLocationState = loc`, `loc = new`, option `IslandLocation = new`.
5. Transitions (U = {2, 9}):
   - old ∈ U, new ∈ U (2↔9): `[FrontTsunami moveBetweenUnderwaterLocations]`, deactivate, activate.
   - old ∈ U, new ∉ U: `mbIslandLocationTransition = YES`, both tsunami layers `deactivate`; leaving 9
     with jellyfish on → jellyfish `exitIsland:NO`. Activation happens later in
     `onTsunamiHideIslandAssets:`/`onTsunamiShowIslandAssets:` (both: deactivate + activate:NO);
     `onTsunamiDeactivateComplete:` clears the flag and removes `UnderwaterDisplayGroup`.
   - old ∉ U, new ∈ U: (startup → `refreshSkinPackOption:@"Halloween" skinObjectId:@"Rock Island"
     skinOptionId:@"Rock Island Extras"`)
     `mbIslandLocationTransition = YES`, `[FrontTsunami activate:(new == 2)]`, `[BehindTsunami activate:(new == 2)]`,
     `UnderwaterDisplayGroup` moved to `UnderwaterDisplayMarker`; the tsunami calls back
     `onTsunamiHideIslandAssets:` (swap) and `onTsunamiActivateComplete:` (flag cleared, pygmies
     `startUnderwater`).
   - otherwise: deactivate (skipped at startup), activate:startup.

Who changes location:

| Trigger | Method | Target |
|---|---|---|
| Map screen | `onMapScreenChangeIslandLocationState:` @ 0x100078fe8 | chosen location |
| Ocean flick (surface) | `stopDragOcean` @ 0x10015fdd4 | `abs(vel) >= 3000`, not underwater, no transition: vel > 0 → 2, else 9 |
| Swipe in the underwater view | `dragUnderwater:` @ 0x100160228 | drag angle in (−120°, −60°) (finger moving down) for > 100 units → previous location; −1/2/9 → 0 (Sand); temples 11–17 → 10 |
| Dream bubble | `onDreamBubbleChangeIslandLocationState:` | sets system `MisfitIslandFound = 1`, travels (to Misfit) |
| Apocalypse temple doors (room fader) | `onApocalypseRoomFaderChangeIslandLocationState:` | temple room; pygmies `dropIntoApocalypseTempleRoom`; thetans dropped in during apocalypse |
| Idol award screen | `onIdolAwardScreenChangeIslandLocationState:` | location |
| Story environment node | `onStoryEnvironmentChange:` (4.10) | node location |
| Startup | `updateIslandStateAtStartup` | option `IslandLocation` (startup:YES) |

`hasMisfitIslandBeenFound` (system `MisfitIslandFound`) is asked by the map screen.

### 4.3 Island feature toggles

All feature setters share one pattern (e.g. `setSandIslandState:startup:` @ 0x100083f50):

```
old = state
if old has objects: their exitIsland, button behaviour Toggle<Old>Off
state = (old == value) ? 0 : value          // pressing the active choice turns it off
option <Key> = state
if state has objects: button behaviour (startup ? Start<New>On : Toggle<New>On), objects enterIsland:startup
```

(`Start…On`/`Toggle…On/Off` are sequences that animate the menu button.) The setters change objects
even when the player is on another island; `enterIsland:` of objects on other islands is guarded by
their own logic via the `is…Enabled` predicates. `updateIslandStateAtStartup` @ 0x10007e398 only
reads the options into the state ivars and sets the `Start…On` behaviours (no objects), then
`setIslandLocationState:IslandLocation startup:YES` enters them.

| State ivar | Option key (default) | Values → objects / button | Setter |
|---|---|---|---|
| `mSandIslandState` | `SandIslandHazard` (3) | 1 boulder (Boulder "Meteor"), 2 fishing pole+fish (Fish "Fishing"), 3 magnifying glass+ant hill (AntHill), 4 water wings | `setSandIslandState:startup:` @ 0x100083f50 |
| `mRockIslandState` | `RockIslandHazard` (1) | 1 fishing | `setRockIslandState:startup:` @ 0x1000842bc |
| `mRockIslandFeatureState` | `RockIslandFeature` (1) | 1 bonfire + 3 wood (Wood), 2 T-Rex egg + spears (Egg) | `setRockIslandFeatureState:startup:` @ 0x100084414 (2: T-Rex `exitIsland` and spears exit on leaving) |
| `mRockIslandOutHouseState` | `RockIslandOutHouseState` (1) | outhouse | @ 0x10008490c |
| `mRockIslandTRexState` | `RockIslandTRexState` (1) | T-Rex boss | @ 0x100084a48 |
| `mRockIslandSpearState` | `RockIslandSpearState` (1) | spears | @ 0x100084b80 |
| `mGraveyardIslandState` | `GraveyardIslandState` (0) | water wings | @ 0x100084e24 |
| `mGraveyardIslandZombieGraveState` | `GraveyardIslandZombieGraveState` (1) | zombie grave | @ 0x100084f60 |
| `mGraveyardIslandGhostState` | `GraveyardIslandGhostState` (1) | ghosts (no object; `areGhostsEnabled`) | @ 0x10008509c |
| `mGraveyardIslandSpiderWebState` | `GraveyardIslandSpiderWebState` (1) | spider tree (+ spider and spears follow it) | @ 0x100085198 |
| `mGraveyardIslandSpiderState` | `GraveyardIslandSpiderState` (1) | spider boss | @ 0x100085488 |
| `mGraveyardIslandSpearState` | `GraveyardIslandSpearState` (1) | spears | @ 0x1000855c0 |
| `mIceIslandState` | `IceIslandState` (1) | 1 fishing, 2 water wings | @ 0x100085864 |
| `mIceIslandIceMonsterState` | `IceIslandIceMonsterState` (1) | ice monster | @ 0x100085a48 |
| `mIceIslandIglooState` | `IceIslandIglooState` (1) | igloo | @ 0x100085b84 |
| `mIceIslandIceHoleState` | `IceIslandIceHoleState` (1) | ice hole | @ 0x100085cc0 |
| `mIceIslandIceSquidState` | `IceIslandIceSquidState` (1) | `enableIceSquid`/`disableIceSquid` on the ice hole | @ 0x100085dfc |
| `mApeIslandState` | `ApeIslandState` (2) | 1 altar+bongo+wench (ApeAltar), 2 banana tree+Kong statue (ApeBananaTree) | @ 0x100085f30 |
| `mApeIslandApeState` | `ApeIslandApeState` (1) | giant ape | @ 0x10008619c |
| `mApeIslandMonkeyState` | `ApeIslandMonkeyState` (1) | monkeys (no direct object; `isMonkeyEnabled`) | @ 0x1000862d8 |
| `mMisfitIslandState` | `MisfitIslandState` (1) | 1 moron snake, 2 water wings | @ 0x1000863d4 |
| `mGodIslandState` | `GodIslandState` (0) | water wings | @ 0x10008658c |
| `mGodIslandIdolsState` | `GodIslandIdolsState` (1) | 4 god idols (+ `loadTextureAtlasChannel:1`) | @ 0x1000866c8 |
| `mApocalypseIslandState` | `ApocalypseIslandState` (0) | water wings | @ 0x100086988 |
| `mFailBoatState` | `FailBoatState` (—) | value only, no button, no objects | @ 0x100086ac4 |
| `mUnderwaterState` | `UnderwaterHazard` (2) | 1 anchor (Piranha "Bait Master Game"), 2 chum bucket (LaserShark "Shark Blast Game"), 3 tile drain+plug (TileDrain "Pain Drain Game") | @ 0x100086b00 |
| `mUnderwaterCoralReefState` | `UnderwaterCoralReefState` (1) | 5 jellyfish | @ 0x100086d90 |
| `mSharkState` | `SharkState` (1) | shark (not entered on Ape island); off: `exitIsland:NO`, `ToggleSharkOff` only when not startup | `setSharkState:startup:` @ 0x10007d894 |
| `mBirdState` | `BirdState` (1) | bird | `setBirdState:startup:` @ 0x10007d9c4 |

Quirk kept from the original: `updateIslandStateAtStartup` gives Misfit state 1 the start behaviour
`StartMoronSnakeOn` on `mMisfitMoronPestsButtonLogic` (the Moron Pests button), not on the moron
snake button.

The predicates the logic objects ask (`is…Enabled`, section 6) combine `loc` with these states.

### 4.4 Option preferences (complete list)

`Options` document keys read or written by the controller (defaults from 2.5; `—` = no default):

| Key | Meaning |
|---|---|
| `IslandLocation` | current location (default 5) |
| feature keys of 4.3 | island features |
| `WeatherHazard` | 0 clear, 1 storm (button toggles 0↔1) |
| `SharkState`, `BirdState` | global hazards |
| `TemperatureState` | pinch-sun power on (turning it off resets the sun size to 0) |
| `HurricaneState` | hurricane power on |
| `RainbowState` | rainbow power on |
| `CrackHoleState` | drawing cracks on the ground on |
| `TheRunsState` | a pygmy falling into a crack starts The Runs |
| `CharlieState` | Charlie guest pygmy active (+1 pygmy slot) |
| `KonkeyDongState` | Konkey Dong starts when the ape kidnaps a pygmy (`isKonkeyDongEnabled`) |
| `MoronPestsState` | Moron Pests game enabled (`isMoronPestsEnabled`) |
| `DrumHazard` | tap-to-drum/dance on |
| `GravityHazard` | tilt gravity on (`isGravityEnabled`) |
| `Mute`, `Vibrate` | sound / vibration off (`BCSound mute:`, `muteVibrate:`) |
| `DisplayNames` | pygmy name labels (`areNameLabelsEnabled`) |
| `ButtonLabels` | menu button labels (`areButtonLabelsEnabled`) |
| `ShowIdolStartScreen` | show the idol start screen at launch (also forced when the app version changed) |
| `ShowEpisodeIcon`, `ShowBattleHelp` | defaults only (read by `IdolManager`/episode UI) |
| `IdolsState` | only read (start behaviour of the idols button); never written by the controller |

System document: `MisfitIslandFound`, `WhatsNewVersion`, `InfoShown`, `BonusInfoShown`, feint
achievement ids (4.12).

### 4.5 Time of day, sun, moon, weather

State: `mbNightActive`, `mbSunsetActive` (day = neither), `mSunState` (0 normal, 1 big, 2 small),
`mWeatherState` (0 clear, 1 storm), gradient weights (3.4).

- **Drag the sun** (touch on the sun within 40 units, not night; 5.2): `startDragSun:` stores the
  position; `dragSun:` @ 0x1001608e4 moves the sun vertically by the finger's dy, y clamped to
  [`horizon − 40`, 245]; `s = max(0, 1 − |horizon − y|/40)`; above the horizon: sunset target = s and
  "sunset" means `s > 0.2`; at or below: sunset target 1, sunset; night target = `|horizon − y|/40`
  below the horizon, else 0; sun reflection: s == 0 → removed, else inserted before
  `CloudBehindDisplayMarker` (not on Ape island) with alpha s; when the sunset flag flips → runs
  `StartSunset` / `StartDay`. `stopDragSun` → if sun y < `horizon − 25`: `startNight`.
- **Drag the moon** (night, within 40 units): `dragMoon:` moves it vertically, clamped to
  [`horizon − 40`, 245]; `stopDragMoon` → moon y < `horizon − 25` → `startSunset`.
- `startNight` @ 0x100160388: night, no sunset, targets sunset 0 / night 1, episode event `Night`,
  `[moon transitionIn:245]`, `[sun transitionOut:sunUnderHorizonY]`, runs `StartNight`.
- `startDay` @ 0x1001604c0: bat flies off (`BatFlyOffScreen`), day, targets 0/0, sun reflection
  removed, `[sun transitionIn:245]`, `[moon transitionOut:moonUnderHorizonY]`, runs `StartDay`.
- `startSunset` @ 0x100160624: bat flies off, sunset, sunset weight reset to 0 with target 1, night
  target 0, episode event `Sunset`, sun reflection inserted (not on Ape island) with alpha 1,
  `[sun transitionIn:horizon]`, `[moon transitionOut:…]`, runs `StartSunset`.
- **Sun size** (pinch, 5.2/5.3) → `setSunState:` @ 0x10007d4a8: no-op if unchanged; 0: big target 0
  (small target 0); 1: big 1, small 0; 2: big 0, small 1; runs `ChangeSunState`;
  `[mSunLogic setSunState:]`. Turning `TemperatureState` off calls `setSunState:0`.
  `sunState` (asked by logics) returns 0 at night.
- **Eclipse**: drag the boulder over the sun (3.3).
- **Weather** `setWeatherState:startup:` @ 0x10007d5c4 (`mWeatherState` = value, no toggle logic):
  1 → an Ooga Jump in progress is ended (doodle clouds removed, `SkyTiles` hidden,
  `doodleKnockedDown:NO`); not startup: `mCloudDragVel = 4000`, `mWeatherSequence` runs `StartStorm`
  (`transitionOutClouds`, `transitionInStormClouds`, sound, `setCloudGradientFade 1`), option saved,
  `ToggleWeatherOn`; startup: cloud target 1, both storm clouds `startVisible`, `StartWeatherOn`;
  always episode event `StormClouds`. 0 → not startup: vel 4000, `StopStorm`
  (`transitionOutStormClouds`, `transitionInClouds` = stop hurricane + `createClouds` + clouds
  `transitionIn:`, `setCloudGradientFade 0`), option saved, `ToggleWeatherOff`; startup:
  `createClouds`, cloud target 0. (The weather button computes the new value as `old < 1 ? old + 1 : 0`.)
- **Clouds**: `createClouds` @ 0x10015e774 removes old clouds and creates 4 `CloudLogic`
  (`initWithDisplayObject:cloudIndex:` 0–3) on new `BCModel "NewCloud"` (texture `Cloud1Texture`)
  inserted after `CloudBehindDisplayMarker`; `onCloudDie:` removes one.
- **Cloud drag** (touch above the horizon; 5.2): `startDragClouds:timeStamp:` → drag on, average
  velocity −1 (unset), vel 0; all clouds (or both storm clouds) `startDragging`.
  `dragClouds:timeStamp:` @ 0x10015f8b8: `dt' = t − lastT` (0.02 if 0); `dx = x − lastX`;
  if `atan2(dy, dx)` is within 30° of vertical (|angle| in (30°, 150°)) the drag is abandoned
  (`stopDragging: dx·10`, touch released); else `v = clamp(dx/dt', ±7000)`, avg = v first time else
  `avg + (v − avg)·0.5`; `mCloudDragVel = clamp(avg, ±5000)`; storm clouds `applyDragOffset: dx`, or
  each cloud `pos.x += dx`. `stopDragClouds` @ 0x10015f50c: `stopDragging: vel`; if `|vel| >= 3000`
  and no hurricane is active: clear → storm (ends Ooga Jump, episode event `StormClouds`, option 1,
  `StartWeatherOn`, `StartStorm`), storm → clear (option 0, `StopStorm`).
- **Bat**: tapping the moon (`checkForBatTapping:` @ 0x100163740, from `touchBegan` when the moon
  drag starts): skipped if any pygmy `isVampire`; taps must come within 1 s of each other
  (`mBatTapTouchTime`); the third tap creates `BCMultiModel "Bat"` (texture `BatForward_01`, after
  `DragDisplayMarker`, at moon + (0, 40), origin −w/2,−h/2, collision box w × (h+40), selectable)
  with `BatLogic` (listener self) → `mBatLogic`. `onBatDie:` releases it.

### 4.6 God powers

Option-gated powers are started by world gestures (section 5); the rest come from objects.

| Power | Trigger | Implementation |
|---|---|---|
| Lightning | storm weather, touch above y 250 with no bolt alive (5.2) | `startDrawLightning:` @ 0x100160d3c: path list cleared, first point, `ElectricGlow` finger trail (`BCMultiModel "ElectricGlow"`, texture `ElectricGlow_01` centred, behaviour `ElectricGlow`). `drawLightning:` @ 0x1001610cc: trail follows; the first pygmy/ape torch/monkey/bird/bonfire under the finger (`doesPointIntersect:checkSelect:NO`) that accepts `hitByLightning` → `mbShockObject`, the target point (pygmy/bird/bonfire position, or the finger for torch/monkey) is appended and `triggerLightning` fires; otherwise a point is appended when > 20 from the last. `stopDrawLightning:` (no shock yet): append the release point if new, `triggerLightning`. `triggerLightning` @ 0x100160f74: `LightningModel "Lightning"` on the path list (z −277.12, in `IslandDisplayGroup`) + `LightningLogic` (listener self) → `mLightningLogic`; the trail is removed. `onLightningDie:` releases it. `hitPygmyWithWarlockLightning:` (warlock pygmies) builds its own 7-point bolt from y 300 to the pygmy (x jitter ±50.0 in 0.1 steps on points 1–5) unless the player is drawing. `canWarlockUseLightning:` = storm and not in a temple. |
| Double rainbow | option `RainbowState`, touch above the horizon, not night (5.2) | `startDrawRainbow:` clears the path; `drawRainbow:` @ 0x100161a9c: starts after the finger moves > 20 (first point, `RainbowTrail` finger trail, `[mRainbowLogic startDrawRainbow]`), then rotates the trail to the stroke direction and every > 5 units sends `drawRainbow:` and appends a point. `stopDrawRainbow:`: final point, `drawRainbowComplete`, trail removed. `mRainbowLogic` (created in `loadGame`) owns the `RainbowModel` and its effect on pygmies (RainbowLogic spec); the controller does not start a celebration for it. |
| Locust swarm | `[mLocustLogic isLocustActive]` (enabled by temple/episode logic) — the first touch draws the path, also in temples | `startDrawLocust:` @ 0x100161fac: vector cleared + first point, `startDrawLocust:startPos:`, runs `LocustStartDraw`. `drawLocust:`: point every > 5. `stopDrawLocust:`: final point, `drawLocustComplete`; released outside x ∈ [10, 470], y ∈ [10, 310] → `deactivateLocusts`. |
| Hurricane | option `HurricaneState`, storm weather, touch above the horizon (5.2) | `startHurricane:` @ 0x100162384: creates (once) `HurricaneModel "Hurricane"` + `HurricaneLogic` (listener self) and a `WindGlow` finger trail; `setTouchActive:YES`. `maintainHurricane:` moves the trail, `setWindVel:0`. `stopHurricane:` → `setTouchActive:NO`, trail removed. Hurricane lightning on pygmies scores challenge game 1; `onHurricaneCantPickUpObjects:` → `stopGame:1`; `onHurricaneDie:` releases. `spriteLogicArray:` gives it the pick-up candidates (bat if any, anchor, shark, bird, boulder, T-Rex egg, T-Rex, spider, outhouse, ant hill, bonfire, magnifying glass, fish, swordfish, fishing pole, banana, moron snake, then all pygmies, coconuts, wood, spears). |
| Whirlpool | underwater, a touch that is neither the camera drag nor the coral-reef power (5.2) | `maintainWhirlPool:` @ 0x1001627c4 creates (once) `WhirlPoolModel "WhirlPool"` in `UnderwaterDisplayGroup` + `WhirlPoolLogic`, the `WindGlow` trail, `setTouchActive:YES`, `setWindVel:0`; `stopWhirlPool:` deactivates; `onWhirlPoolDie:` releases. |
| Underwater power | coral reef with `underwaterTempleMode` ≠ 0 | `startDragUnderwaterPower:` @ 0x100162bac: `BCMultiModel "UnderwaterPower"` before `UnderwaterPygmyDisplayMarker` + finger trail; episode event `UnleashUnderwaterPower{Cold,Heat,Oil,Acid,Air}` for modes 1–5. `dragUnderwaterPower:`: every > 30 units a `UnderwaterPowerTrail` puff (`MultiModelLogic`, behaviour `UnderwaterPower<Mode>Trail`, removed on its `UnderwaterPowerTrailComplete` multi-model event); pygmies within 30 of (x, y+20) that `canBeTouchedByUnderwaterPower` get `canBeTouchedByUnderwaterPower:mode`. `onUnderwaterTempleModeChanged:` retints the trail (`UnderwaterPower<Mode>`), mode 0 ends the stroke. |
| Drum and dance | option `DrumHazard`; any world tap (`checkForTapping:` @ 0x100163550) | a tap counts only when ≥ 1 frame passed since the last (`mTapTouchTime != 0`); from the 4th tap each tap plays `TapLeft`/`TapRight` (x ≤ or > half the touch width) unless a dance song plays; the 4th tap `startCelebration`; 1 s without taps → `stopCelebration` (3.2). |
| Celebration | drum (4th tap) and story (`onStoryModeStartCelebration`) | `startCelebration` @ 0x100157ca4: `mbCelebrationActive`; picks a dance song from the active skin options (4.13) — Pants on the Ground, Update Song, Theme Song, Fishmas, 12 Days of Fishmas (each stops the background loop) or, with a zombie on Graveyard island, `ThrillerLoop`; ape bongo `startPlaying`, ape `startTapping`, monkeys `startDancing`; runs `StartCelebration` → `startPygmiesDancing:::` (vampires `startVampireDancing` with the Vampire Dance option, otherwise `startDancing:` with the theme-song lead rule; new `mCelebrationSequence` `Celebration` = endless `fireworks`). `stopCelebration` reverses (song stopped, `playBackgroundLoop:nil` unless a minigame runs) and runs `StopCelebration` on `mCelebrationSequence` → `stopPygmiesDancing:::`. `isDanceActive` (asked by pygmies and monkeys) and `isRainbowActive` (asked by `PygmyLogic(Island)`) both return `mbCelebrationActive`. |
| Fireworks | `fireworks:::` item (0x10015a7c8) of `Celebration` | not in temples; waits a random time in [min, max] (0.001 s steps), then a `BCModel "Firework"` (texture `FireworkSpark`, origin (−8, −8)) after `FireworksDisplayMarker` at x ∈ [50, 430], y ∈ [190, 320] (0.1 steps) with `FireworkLogic`; colour: Graveyard (rand % 5) (1,0,0), (1,.2,.2), (.8,0,0), (.6,0,0), (.4,0,0); elsewhere (rand & 7) (1,.8,0), (1,.2,0), (1,.6,.2), (0,1,1), (1,1,.8), (.6,1,0), (.4,.6,1), (1,.4,1). Never completes. |
| Earthquake | pygmies (`onPygmyEarthquakeStart/Stop:`), story | first quaker: episode event `Earthquake`, `stopCelebrationMusic`, coconut on the tree falls, new `mEarthquakeSequence` `EarthquakeStart` (sounds, tree behaviours, endless `shakeBackground`); last quaker leaves → `EarthquakeStop` (shake 1–2 s more, `stopEarthquake` which restarts `EarthquakeStart` if quakers remain). `shakeBackground:::` @ 0x10015e374: `IslandDisplayGroup` (or The Runs group) `y = 2·sin(phase) − 2`, `phase += 1.3` per frame. |
| Volcano | pygmies thrown in (`onVolcanoHit:` @ 0x100164e00) | hit count 0/1/2 → `hit1/hit2/hit3` (`erupt1/2/3` when the pygmy's `timeTemplePygmyType` is 4); the third starts the eruption (`mbEruption` until `onEruptionComplete:`). |
| Pinch sun / temperature | option `TemperatureState` | 5.2/5.3 → `setSunState:` |
| Crack | option `CrackHoleState` | 5.2: `startDragCrack:` → `[crackHole startDragging:]`, `drag:`, `stopDragging` |
| Ocean flick | always (surface, below the horizon, not Ape) | 4.2 |

Looping sounds paused while minigames run (each also exists as a sequence item `…:::` returning 2):
`pauseHurricaneSounds`/`resumeHurricaneSounds` @ 0x100168df4/0x100168e70 stop/play `Wind` when the
hurricane exists and `windActive`; `pauseLocustSounds`/`resumeLocustSounds` stop/play
`InsectBuzzLoop` when `[mLocustLogic isLocustActive]`; `pauseSleepingSounds`/`resumeSleepingSounds`
stop/play `Snore` when `mSleepingCount > 0`. Story start/end pause/resume the sleeping sound too.

### 4.7 Menus and the button dispatcher

HUD (`PocketGod` group): `NewPygmyButton` (`mNewPygmyButtonLogic`) and `MenuButton` (`mMenuButtonLogic`).
The main menu `MenuDisplayGroup` (pos y 90 closed → 0 open) holds the travel (map) button, the six
category buttons, Help and the removed Debug button. Each category menu is a display group
inserted after `CategoryMenuDisplayMarker` (button labels from each object's `data` string):

| Category (state) | Menu group | Buttons (label) |
|---|---|---|
| 1 Island (per location) | `<Location>MenuDisplayGroup` | Sand: Boulder "Meteor", Fish "Fishing", AntHill "Ant Hill", WaterWings "Water Wings"; Rock: Fish, OutHouse "Outhouse", Wood, Egg "T-Rex Egg", TRex "T-Rex Boss", Spear; Graveyard: ZombieGrave, Ghost "Ghosts", SpiderWeb, Spider "Spider Boss", Spear, WaterWings; Ice: Fish, WaterWings, IceMonster, Igloo, IceHole, IceSquid; Ape: ApeAltar, ApeApe "Giant Ape", ApeBananaTree, ApeMonkey "Monkeys", ApeKonkeyDong "Start Konkey Dong When Kidnapped"; Misfit: MoronSnake, WaterWings, MisfitMoronPests "Moron Pests Game"; God: IdolSelection "Idols Selection", Idols, WaterWings; Apocalypse: WaterWings; Underwater: Piranha "Bait Master Game", LaserShark "Shark Blast Game", TileDrain "Pain Drain Game"; Coral reef: Jellyfish; Fail boat and temples: none |
| 2 Global island | `GlobalIslandMenuDisplayGroup` | Weather "Storm Clouds", Shark, Bird, CrackHole "Draw Crack on Ground", TheRuns "The Runs Game", Charlie |
| 3 God powers | `GodPowerMenuDisplayGroup` | Drum "Tap to Drum and Dance", Temperature "Pinch Sun to Change Temp", Hurricane "Swirl Hurricane", Rainbow "Draw Double Rainbow" |
| 4 Customize | `CustomizeMenuDisplayGroup` | SkinPack "Customize with Skin Packs", BonusPack "Bonus Packs", Story "Create Stories", Stats "Tribe Stats", IdolsChallenge "Idols Challenge" |
| 5 Community | `CommunityMenuDisplayGroup` | News "Pocket God News", Challenge "Challenge Friends", StartScreen "Show Start Screen" |
| 6 Settings | `SettingsMenuDisplayGroup` | Mute, Vibrate, Gravity "Tilt for Gravity", Names "Name Labels", Labels "Menu Labels" |

`setCategoryMenuState:` @ 0x10007dd04: the open menu group is removed and its category button gets
`Toggle<Cat>Off`; new state = (old == value) ? 0 : value; the new group gets `Toggle<Cat>On`
(`Island`, `GlobalIsland`, `GodPower`, `Customize`, `Community`, `Settings`),
`updateMenuLabelsDisplayGroup:` and is inserted after `CategoryMenuDisplayMarker`; the labels group
follows it when a menu is open and option `ButtonLabels` == 1, else it is removed.
`updateMenuLabelsDisplayGroup:` @ 0x10007dae0: for labels `Label1Text`…`Label6Text` take the next
selectable child of the menu group (in child order) and copy its `data` into the label; label x =
`MenuBG.x + 60` when `MenuBG.x <= 240`, else `MenuBG.x − textWidth·scale.x − 10`; labels without a
button get the text `""` (the `cf___` literal).

`openMenu:::` → targets 0/0; `closeMenu:::` → menu closed, `setCategoryMenuState:0`, targets 90/90.
`hideUi` @ 0x1000773ac hides and disables the HUD buttons (closing an open menu, `CloseMenu`);
`showUi` re-enables them. `closeScreens` @ 0x1000775a8 closes every open screen (map, stats,
challenge, skin pack, bonus pack, story, news, idol, help → `loadMonsterTextureAtlas` + `CloseHelp`,
debug). `isScreenOpen`/`isDialogActive`: section 6.

`onButtonPress:touch:` @ 0x100079090 (releases: `onButtonRelease:touch:`,
`onButtonReleaseOutside:touch:` are empty). "Opening a screen" below = `cancelAllTouches`,
`[mNewsManager closeNewsScreen]`, set the screen flag, run `CloseMenu`, call the manager.

| Button | Action |
|---|---|
| NewPygmy | if live pygmies < 6 (+1 if `CharlieState` == 1, +1 if Tom alive): runs `AddPygmy` |
| Menu | toggles `mbMenuOpen`: runs `OpenMenu`, or `setCategoryMenuState:0` + `CloseMenu` |
| IslandLocation | cancel touches, close news, `mbMapScreenOpen`, `[mMapScreen openMapScreen]` |
| Island/GlobalIsland/GodPower/Customize/Community/Settings menu | `setCategoryMenuState:1…6` |
| Help | cancel touches, close news, `mbHelpScreenOpen`, runs `OpenHelp` (help group, `closeMenu`, `updateHelpPage`) |
| Debug (never wired) | debug flag, `CloseMenu`, `[mDebugScreen openDebugScreen]` |
| Help close / next / prev | close: flag off, `loadMonsterTextureAtlas`, `CloseHelp`; `updateHelpPage:index±1` |
| island feature buttons | `set<…>State:value startup:NO` with the value of 4.3 (e.g. Rock Egg → `setRockIslandFeatureState:2`) |
| ApeKonkeyDong, MisfitMoronPests, CrackHole, TheRuns, Drum, Hurricane, Rainbow, Gravity, Names | flip the option 0↔1, button `Toggle<X>On/Off` |
| Charlie | flip `CharlieState`, `Toggle…`, tribe `activate`/`deactivateGuestPygmyPersona:` Charlie |
| Temperature | flip `TemperatureState`; turning off first `setSunState:0` |
| Weather | `setWeatherState:(WeatherHazard < 1 ? +1 : 0) startup:NO` |
| Shark / Bird | `setSharkState:1` / `setBirdState:1` (toggle) |
| IdolSelection | open screen: `[mIdolManager openIdolSelectionScreen]` |
| SkinPack / BonusPack / Story / Challenge | cancel touches (+ close news except BonusPack) → `openSkinPackManager`, `openBonusPackManager`, `openStoryManager`, `openChallengeManager` |
| Stats | open stats: `mActiveStatsScreen = 1`, `updateTribeStats`, `OpenStatsTribe`, `[mIdolManager delayTriggerEpisodeEvent:@"OpenTribeScreen"]` |
| IdolsChallenge / StartScreen / News | open screen: `openIdolListScreen` / `openIdolStartScreen` / news (`mbNewsScreenOpen`, `openNewsScreen`) |
| Mute | flip `Mute`: on → `BCSound mute:YES`, `ToggleMuteOn`, `stopAllSounds`; off → `mute:NO`, `ToggleMuteOff`, `playBackgroundLoop:nil` |
| Vibrate | flip `Vibrate` with `BCSound muteVibrate:` |
| Labels | flip `ButtonLabels`; on → labels group after `CategoryMenuDisplayMarker`, off → removed |
| WhatsNew Go (screen never opened) | flag off, `CloseWhatsNewScreen` |
| stats pages | section 4.8.3 |

`showMessage:::` @ 0x100159ebc (items `zPos`, `xPos`, `texture`, `yPos`): replaces `mMessageDisplayObject`
with a `BCModel "Message"` of that texture at (x, y, z) after `MenuDisplayMarker`, runs `ShowMessage`
on `mMessageSequence` (fade in 0.5 s, 3 s, fade out, `removeMessage`). Used for `OpeningTitle`.

### 4.8 Screens

All screens are display groups driven by sequences or by manager singletons; the controller tracks
which are open (`isScreenOpen`) to gate touches (5.2), gravity, ads and atlases.

#### 4.8.1 Help

`updateHelpPage:` @ 0x1000761b4: index < 0 → 68, > 68 → 0; runs `HelpPageStart` (index 0) or
`HelpPage01`…`HelpPage68` (each adds its page group at `HelpContent` and loads the page's atlas).
`updateHelpPage:::` = `updateHelpPage:mHelpPageIndex`.

#### 4.8.2 Map

`IslandLocationButton` opens `MapScreen` (its own spec); callbacks `onMapScreenClosed`,
`onMapScreenChangeIslandLocationState:`; the map asks `islandLocationState`, `mapPaperLogic`,
`hasMisfitIslandBeenFound`, `showChartboostAd`/`isChartboostAdVisible` (4.18).

#### 4.8.3 Stats pages and text entry

Three groups: `StatsPageTribe` (`mActiveStatsScreen` 1), `StatsPagePygmy` (2), `StatsPageGod` (3).

- `updateTribeStats` @ 0x100075a88: `mPygmyStatsType = 0`; `TribeName` = tribe name,
  `TotalGifts`/`TotalSacrifices` = `"%d"`, `GodName`, `GodType` texture =
  `getGodTypeTextureId:godType`, for i = 0..5 `Pygmy<i>MoodPic` texture = `getMoodTextureId:mood`
  and `Pygmy<i>Name` of `pygmyPersonaArray[i]`.
- `updatePygmyStats` @ 0x100074ac4 (for `mActivePygmyPersona`): `PygmyMoodPic` texture
  (`getMoodTextureId:mood`), `PygmyName`, `PygmyDescription`, `PygmyMood` (`getMoodDescription:mood`);
  then by `mPygmyStatsType`: **0 = sacrifices** — `ListTag` "SACRIFICES:", `OverallStat` =
  `"%d"` of `totalSacrifices`, `Stat0`–`Stat4` = `"%d %@"` (`sacrificeCount:id`,
  `getSacrificeDescription:id`) for the first five ids of `sortedSacrificeIdArray`; **1 = gifts** —
  "GIFTS:", `totalGifts`, `sortedGiftIdArray`, `giftCount:`, `getGiftDescription:`. Rows without an
  id are set to "" (argument order checked in the disassembly). Description tables: section 10.
- `updateGodStats` @ 0x100075440: `GodTypePic` texture (`getGodTypeTextureId:`), `GodName`,
  `GodDescription`, `GodType` (`getGodTypeDescription:`), `Stat0`–`Stat5` = `"%d %@"`
  (`achievementCount:id`, `getAchievementDescription:id`) for the first six ids of
  `achievementIdArray` (unused rows "").
- Buttons (`onButtonPress:touch:`): Stats → tribe page; GodStats → page 3 (`OpenStatsGod`);
  `Pygmy%dStatsButton` i → page 2 for `pygmyPersonaArray[i]` (`OpenStatsPygmy`); pygmy Next →
  `mPygmyStatsType` toggles 0↔1 (`NextStatsPygmy`); god Next (GodTypePic) → god type
  `(type < 6 ? type + 1 : 0)`, episode event `ChangeGodType`, `NextGodType`; Close buttons → back to
  the tribe page (`CloseStatsPygmy`/`CloseStatsGod`); tribe Close → `closeStatsScreen` (`CloseStatsTribe`).
- Name editing: tapping `TribeName` (max 16 chars), `PygmyName` (10, + episode event
  `ChangePygmyName`), `PygmyDescription` (30), `GodName` (10), `GodDescription` (30) opens text entry
  (`mbKeyboardOpen`, `mEditingText`, the page's cancel button shown and selectable, `showKeyboard`
  on the main thread, sequence `<Page>StatsEdit<Field>`); the cancel button restores the persona's
  value; Return (`textFieldShouldReturn:`) writes `setName:`/`setDescription:` to the persona. Edits
  are blocked while the keyboard is open. `textField:shouldChangeCharactersInRange:replacementString:`
  enforces `mMaxEditCharCount` and mirrors the text into the `BCText`.
- Managers (`BonusPackManager`, `StoryManager`) use the same machinery through
  `onShowKeyboard:editingText:cancelEditDisplayObject:maxEditCharCount:preventEmpty:` /
  `onHideKeyboard` / `isKeyboardOpen`; on end the text goes to `[mKeyboardListener onKeyboardTextEntered:]`.
  The port replaces the hidden `UITextField` with an egui text field.

#### 4.8.4 Store (skin packs), bonus packs, story, news, idols, challenges, debug

| Screen | Open | Close callback | Notes |
|---|---|---|---|
| Skin pack store | `openSkinPackManager` @ 0x1000778b8: every pygmy `stopDancingNow`, flag, `CloseMenu`, `[mSkinPackManager openSkinPackScreen:firstTime]` (system `InfoShown` 0 → shows info, sets 1) | `onSkinPackScreenClosed` (+ `loadMonsterTextureAtlas`) | also from news (`onNewsLaunchStoreScreen`) and idols (`onIdolScreenLaunchStoreScreen`) |
| Bonus packs | `openBonusPackManager` (system `BonusInfoShown`) | `onBonusPackScreenClosed`; `onBonusPackUnlockDialogOpened/Closed` set/clear the same flag | |
| Story | `openStoryManager` → `openStoryScreen` | `onStoryScreenClosed` | story mode 4.10 |
| Challenge | `openChallengeManager`: a challenge running → `cantOpenChallengeScreen`, else flag, `CloseMenu`, `openChallengeScreen` | `onChallengeScreenClosed` | `onChallengeDialogOpen`, `onStartChallenge:`, `onEndChallenge:` (4.11) |
| News | `checkForNewNews` teaser at boot; News button → news screen | `onNewsTeaserClosed`, `onNewsScreenClosed` | teaser "more" → `onNewsTeaserLaunchNewsScreen`; news links → store/challenge/story screens |
| Idols | start screen (2.4), StartScreen button, IdolSelection, IdolsChallenge, episode alerts (4.9) | `onIdolScreenClosed` | award dialog: `onIdolAwardScreenOpened/Closed/OpenChallenges/ChangeIslandLocationState:` |
| Debug | unreachable (`mDebugButtonLogic` nil) | `onDebugScreenClosed` | |
| What's new | never opened in 1.48.2 (`mbWhatsNewScreenOpen` only cleared) | Go button → `CloseWhatsNewScreen` | |

### 4.9 Episodes and idols

`IdolManager` owns idols, episodes (challenges with idol rewards) and Battle of the Gods screens. The
controller feeds it events (`triggerEpisodeEvent:` from code and from the `triggerEpisodeEvent` item,
392 uses in sequences): `Night`, `Sunset`, `StormClouds`, `Earthquake`, `StartApocalypse`,
`EndApocalypse`, `Spawn<Gem>Gem` ×7, `UnleashUnderwaterPower<Mode>` ×5, location events from the start
scripts, `MakeEclipse` (sequence `Eclipse`), delayed `OpenTribeScreen`, `ChangePygmyName`,
`ChangeGodType`, and `triggerEpisodeEventBySacrificeId:` / `triggerEpisodeEventByGiftId:` from pygmy
deaths and gifts. `loadEpisodeState` runs in `loadPreferences:::`. Episode alerts (`EpisodeAlertLogic`)
call `onEpisodeAlertOpenChallengeDetail:` → `openIdolDetailScreen:` unless a screen, story, timed
challenge game (4, 5, 6, 10–15), No Pee or dialog is active. Temple doors on God island start Battle
of the Gods (4.16). The idol manager also asks `godIdolLogicArray` (god idols on God island) and
`onApocalypseGemEventTriggered:` (4.17).

### 4.10 Story mode

`StoryManager` plays user stories; the controller lends the world:

- `onStoryStart` @ 0x10007821c: `mbStoryActive`, sleeping sounds paused, `WinchTurn` stopped,
  every pygmy `removeFromStoryMode`, rainbow `storyModeClearRainbow`, option `DisplayNames` saved in
  `mStorySaveLabelMode` and forced to 1, `disableIslandLogics` (T-Rex/spider exit when active, ice
  monster, ant hill, bird, shark, banana tree, balloon bundle, monkeys, moron snake, jellyfish, god idols
  exit when enabled), `hideUi`.
- `onStoryComplete` / `onStoryCancel`: the reverse, `enableIslandLogics` (re-enter ant hill, ice
  monster, bird, shark, moron snake, god idols when enabled; `stopCelebration`; stop a story
  earthquake), `showUi`.
- `onStoryEnvironmentChange:` @ 0x100078460 (`BCStoryEnvironmentElementDef`: location +0x10,
  weather +0x14, time +0x18): location code 1 Rock, 2 Graveyard, 3 Ice, 4 Ape with state 1, 5 Ape with
  state 2, 6 Misfit, 7 God, 8 Fail boat, 9 Apocalypse, 10–16 temples 11–17, other → Sand; travels when
  different (and then `removeAllPygmiesFromStoryMode`); weather 1 → storm else clear
  (`setWeatherState:startup:NO` when different); time 1 → `startNight`, 2 → `startSunset`, else
  `startDay` (each only if not already).
- `onStoryAddPygmy:` @ 0x100078840: returns the pygmy whose `pygmyNameLabel` equals the name, else
  (fewer than 6 live) creates one (`createPygmyLogic`, `setPygmyNameLabel:`, `setStoryMode`), else nil.
- Story events: `onStoryModeStart/StopCelebration` (`start/stopCelebration`),
  `onStoryModeStart/StopEarthquake` (own `EarthquakeStart`/`EarthquakeStop` sequence; pygmies
  `start/stopStoryEarthquake`), `onStoryModeSharkSnap` (shark `storyModeSharkSnap`), rainbow
  `onStoryModeActivateRainbow`, `onStoryModeActivateMyRainbow`, `onStoryModeDeactivateRainbow`.
- Many logics ask `isStoryActive` to freeze themselves.

### 4.11 Challenges and challenge games

`ChallengeManager` runs friend challenges: timed "games" identified by an integer. The controller
scores them:

| Game | Name (achievement id) | Start / score / stop in the controller |
|---|---|---|
| 0 | Coconut bounces | `onCoconutHitPygmy:` +1; `checkCountCoconutBounces` stops when no coconut is in the air |
| 1 | Hurricane lightning | `onPygmyHitByLightningInHurricane:` +1; `onHurricaneCantPickUpObjects:` stop |
| 2 | Bird bombs | `onBirdPooHitPygmy:` +1; miss / bird off screen → `stopCountingBirdBombs` |
| 3 | Shark snaps | `onSharkSnap:` +1; `checkCountSharkSnaps` stops when no pygmy is dragged (also after pygmy drag ends or dies) |
| 4 | Ooga Jump | `onGameFrame:` `setGameScore:4` (height); `onPygmyStopDoodleJump:` stop |
| 5 | Bait Master | `onPiranhaEatPygmy:` +1 while `mbBaitMaster`; `onBaitMasterStop:` stop |
| 6 | Laser shark (shark blast) | `onLaserSharkZapPygmy:` +1; `onLaserSharkZapComplete:` stop |
| 7 | Kill T-Rex | hit → `setGameScore:7 score:1` unless running; spear throws +1; killed → stop; cancel → `forfeitGame:` |
| 8 | Kill spider | same pattern with the spider |
| 9 | Kill ice monster | battle start → score 0; pygmy deaths +1 while active; killed → stop; cancel → forfeit |
| 10 | Pain Drain | `onPainDrainScore:` sets; stop on `onPainDrainStop:` |
| 11 | The Runs | `onTheRunsScore:` sets; stop |
| 12 | Konkey Dong | start sets 0; `onKonkeyDongScore:` increments by the score; stop |
| 13 | Moron Pests | sets; stop |
| 14 | Dance Dance Execution | sets; stop |
| 15 | No Pee | stop on stop and on retry |

`onChallengeDialogOpen` (cancel touches, screen+dialog flags), `onStartChallenge:` (screen closed,
`mbChallengeActive`), `onEndChallenge:`; `isAnyGameActive` → `[ChallengeManager isAnyGameActive]`
(asked by `IdolManager`). `onPygmiesSkewered:` / `onPygmiesBowled:` call
`checkAchievement:@"Skewered"|@"Bowled" score:n message1:@"You've achieved a personal best of"
message2:@"%d Skewered Pygmies!"|@"%d Pygmies Bowled!" highScoreWins:YES`.

### 4.12 Game Center, leaderboards, achievements

- Leaderboards (`GameServer submitToLeaderboard:score:`): `pg.totalsacrifices` (after every
  sacrifice, `onTriggerSacrifice:sacrificeId:`), `pg.totalgifts` (`onPygmyGiftGiven:giftId:`). The
  per-sacrifice and per-gift leaderboard functions (`getSacrificeLeaderboardId:`,
  `getGiftLeaderboardId:`) return nil for every id in 1.48.2.
- Achievements: `onAchievement:count:highScoreWins:` (from `ChallengeManager`, T-Rex/spider/ice monster
  logics) records the value on the god persona (count −1 = increment), then
  `checkFeintAchievements:score:` (adds the reached tiers from `addFeintAchievementsToArray:…`, sets each
  tier id to 1 in the System document and `submitAchievement:`) and submits the achievement's
  leaderboard (`getAchievementLeaderboardId:`). `checkAllAchievementsAndLeaderboards` (GameServer
  listener, e.g. after sign-in) re-runs this for the 18 ids. Tables: section 10.3.
- Turn-based match callbacks (`handleTurnEventForActiveMatch:` etc.) are empty.
- `checkShowAuthenticationViewController:::` asks `GameServer` to present sign-in.

The port can map these to local achievements; the ids and thresholds are kept in section 10.

### 4.13 Skin packs and dance songs

The controller asks `SkinPackManager` to apply skin options and to load extra atlas channels:

- `refreshSkinPackOption:<pack> skinObjectId:<object> skinOptionId:<option> useFirstPurchased:YES`
  calls (pack / object / option, exact strings — note the spaces; resolved from the disassembly
  because Ghidra gives "Rock Island" and "RockIsland" the same `cf_` label):
  boot: Shark / Shark / Shark Skins; `loadLocationTextureAtlas` per location: Sand → Halloween /
  Sand Island Statue / Statue, then Halloween / Sand Island / Sand Island Extras; Rock → Halloween /
  Rock Island / Rock Island Extras; Graveyard → Halloween / Graveyard Island / Graveyard Island
  Extras; Ice → IceMonster / Ice Monster / Ice Monster, then Halloween / Ice Island / Ice Island
  Extras; Ape → Halloween / Ape Island / Ape Island Extras; `setIslandLocationState:` going
  underwater at startup → Halloween / Rock Island / Rock Island Extras; monster visibility:
  `onTRexVisible:` TRex / Tyrannosaurus Rex / TRex Body, `onSpiderVisible:` Spider / Spider / Spider,
  `onIceMonsterVisible:` IceMonster / Ice Monster / Ice Monster, `onApeVisible:` ZombiePack / Ape /
  Ape, `onLocustQueenVisible:` ZombiePack / Locust Queen / Locust Queen; `loadMonsterTextureAtlas`
  (after the help, skin store and idol screens close) applies the first of spider, ape, locust queen,
  ice monster, T-Rex that `is…Enabled`, with the same triples; the `refreshSkinPackOption` item
  (params `skinPackId`, `skinOptionId`, `skinObjectId`).
- `loadTextureAtlasChannel:` 1 (God island / idols), 2 (Dance temple, item `Disco`), 3 (Xenu temple,
  item `Xenu`).
- Dance songs (`isSkinPackOptionActiveBySkinPackId:skinObjectId:skinOptionId:skinOptionValueId:`,
  object `Pygmies`, option `Pygmy Dance` unless noted), checked in this order by `startCelebration`
  when `canDoNormalDance` (some pygmy can): `Dance` "Pants on the Ground" → `mbPantsOnTheGroundActive`;
  `Dance2` "Pocket God Update Song" → `mbUpdateSongActive`; `Dance3` "Pygmy Theme Song" →
  `mbThemeSongActive`; `Fishmas` "Fishmas Dance" → `mbFishmasActive`; `Fishmas` "12 Days of Fishmas" →
  `mbFishmasTwelveDaysActive`; otherwise a zombie on Graveyard island → `mbThrillerActive` +
  `ThrillerLoop` play. `stopCelebrationMusic` stops the matching sound (`PantsOnTheGround`,
  `UpdateSong`, `ThemeSong`, `MerryFishmas`, `TwelveDays`, `ThrillerLoop`). Vampire dance: object
  `The Vampire`, option `Vampire Dance`, value "Staying Alive (Lead)".

### 4.14 Pygmies

- `createPygmyLogic` @ 0x10015ba88: `BCMultiModel "NewPygmy"`, selectable, origin x −32, collision box
  (0, 0)–(64, 64), x random in [min + 15, max − 15] (0.1 steps), y 340, z −277.12, inserted after
  `PygmyDisplayMarker` (temples: `<Temple>PygmyDisplayMarker`); `PygmyLogic` with listener self; if
  `rawAccelRotDegrees.z` ∈ (−90, 90) → y velocity −1200; underwater (2/9, no transition): y random in
  [200, 340] (`(rand % 1401 + 2000)/10`), 50 % `xScale −1`, `addedUnderwater`; Bait Master running →
  `startBaitMaster`; added to `mLogicArray` and `mPygmyLogicArray`; `mPygmyCount`, `mLivePygmyCount` +1.
- `addPygmy` (NewPygmy button through `AddPygmy`): limit 6 (+1 Charlie, +1 while Tom lives); new pygmy
  is `resurrectPygmyLogic:`-ed by the tribe persona (assigns the persona). `addBabyPygmy:::` (unused
  item) makes a `timeTemplePygmyType 2` baby; `createTomPygmy` (closet door) temporarily activates the
  Tom guest persona; `addTomPygmy:::` (unused item).
- `onPygmyDie:sacrificeId:` @ 0x10016485c: `onTriggerSacrifice:sacrificeId:` (tribe `sacrifice:`,
  leaderboard), live −1, display object removed, drag reference cleared (+ `checkCountSharkSnaps`),
  `shutdown`, removed from both arrays, ice-monster game +1, Tom/doodle/pain drain/The Runs/Konkey
  Dong/Moron Pests/No Pee references cleared (and those flags off).
- Drag bookkeeping: `onPygmyDrag:`/`onPygmyDragComplete:` (`mDraggedPygmyLogic`; completion also
  `checkCountSharkSnaps`), ghost drag; `onPygmyTumble:` moves the pygmy to index 0 of
  `mPygmyLogicArray` (draw/update order); `onPygmyExplode:` → all pygmies `throwFromExplosion:`;
  sleeping counter → `AllPygmiesSleepingStart`/`Stop`; `canTouchPygmy` (section 6).
- Display markers the pygmy asks (`pygmy…DisplayMarker`): section 6.

### 4.15 World objects

`setLogic:::` @ 0x10015c11c (item params: 0 `displayGroupId`, 1 `logicId`, 2 `displayObjectId`,
3 `manager`, 4 `count`). Manager items go to that object's `setLogic:`. Otherwise the object
`displayObjectId` of group `displayGroupId` gets:

| logicId | Created | Stored in | UI array |
|---|---|---|---|
| `NewPygmyButtonLogic`, `MenuButtonLogic`, `LavaButtonLogic` (unused) | `BCButtonLogic`, listener self | `mNewPygmyButtonLogic`, `mMenuButtonLogic`, `mLavaButtonLogic` | main UI |
| `WhatsNewScreenGoButtonLogic` | button | `mWhatsNewScreenGoButtonLogic` | whats-new UI |
| `StatsTribe{Close,EditName,CancelEditName}ButtonLogic`, `GodStatsButtonLogic` | button | matching ivar | tribe stats UI |
| `PygmyStatsButtonLogic` (count n) | n buttons on `[NSString stringWithFormat:displayObjectId, i]` | `mPygmyStatsButtonLogicArray` | tribe stats UI |
| `StatsPygmy{Close,Next,EditName,EditDescription,CancelEditName,CancelEditDescription}ButtonLogic` | button | matching ivar | pygmy stats UI |
| `StatsGod{Close,NextGodType,EditName,EditDescription,CancelEditName,CancelEditDescription}ButtonLogic` | button | matching ivar | god stats UI |
| `Help{Close,Next,Prev}ButtonLogic` | button | matching ivar | help UI |
| `CoconutLogic` | `CoconutLogic`, listener | `mCoconutLogicArray` (only; not `mLogicArray`) | |
| `WoodLogic`, `SpearLogic`, `ApeTorchLogic`, `MonkeyLogic` | class of that name, listener | `mWoodLogicArray`, `mSpearLogicArray`, `mApeTorchLogicArray`, `mMonkeyLogicArray` | |
| `BonFireLogic`, `FishLogic`, `SwordFishLogic`, `FishingPoleLogic`, `BoulderLogic`, `TRexEggLogic`, `TRexLogic`, `SpiderLogic`, `SpiderTreeLogic`, `OutHouseLogic`, `IceMonsterLogic`, `GlacierLogic`, `IglooLogic`, `IceTreeLogic`, `IceHoleLogic`, `ZombieGraveLogic`, `ApeLogic`, `ApeAltarLogic`, `ApeBongoLogic`, `ApeWenchLogic`, `BananaTreeLogic`, `BananaLogic`, `CrackHoleLogic`, `OctoStatueLogic`, `KongStatueLogic`, `SpearStatueLogic`, `AnchorLogic`, `TileDrainLogic`, `TilePlugLogic`, `ChumBucketLogic`, `AntHillLogic`, `MagnifyingGlassLogic`, `SharkLogic`, `BirdLogic`, `VolcanoLogic`, `SunLogic`, `MoonLogic` | class of that name, listener self (`set<Class>Listener:`) | `m<Name>` | |
| `SchoolOfFishLogic`, `UnderwaterBlob1Logic`, `UnderwaterBlob2Logic` | `UnderwaterSceneryLogic initWithDisplayObject:underwaterSceneryType:` 0/1/2 | `mUnderwaterSceneryLogicArray` | |
| `HealthBarLogic` | `HealthBarLogic` (no listener) | `mHealthBarLogic` | |
| `StarsLogic` | `MultiModelLogic`; the object is removed from its group | `mStarsLogic` | |
| `Front/BehindStormCloudLogic` | `StormCloudLogic`, `setFrontStormCloud:YES posVelScale:1.0` / `NO, 0.8` | `mFront/BehindStormCloudLogic` | |
| `Front/BehindOceanLogic` | `OceanLogic`, `setFrontOcean:YES posVelScale:1.0` / `NO, 0.5` | `mFront/BehindOceanLogic` | |
| `Front/BehindTsunamiLogic` | `TsunamiLogic`, `setFrontLayer:YES/NO`, listener | `mFront/BehindTsunamiLogic` | |
| `MapPaperLogic` | `MapPaperLogic` (plain `initWithDisplayObject:`) | `mMapPaperLogic` | |
| `SpriteLogic`, `MultiModelLogic`, `TheRunsPlatformLogic`, `TheRunsStalactiteLogic`, `KonkeyDongLogic` | class, no listener | — | |
| unknown | nothing | | |

Every created logic except coconuts is also added to `mLogicArray`. Every listener method the objects
call back is in section 6; the drag callbacks set/clear `mDragged<X>Logic`. Of these refs, 22
(plus `mDirectedAntHillLogic`) make `canTouchPygmy` @ 0x100163c34 answer NO; the pygmy, ghost pygmy,
apocalypse gem and fishing bob refs do not (pygmy and gem drive the drag auto-scroll of 3.2; the
fishing bob ref is never read). Object events with game effects are in 4.11 and below:

- Coconut: `onCoconutHitPygmy:` — game 0 +1; an unsplit coconut `split:NO` and a second
  `CoconutLogic` is made on a new `BCMultiModel "Coconut"` copying position, selectability, origin
  and collision box, `split:YES` (its `setPosVel:` copies its own velocity — a no-op in the original);
  `onCoconutDie:` — `checkCountCoconutBounces`; with two coconuts the dying one is removed, else it is
  put back at (163, 205) and `grow`s on the tree.
- Fishing: `onFishingPoleCreateBob:fishingBobLogic:` makes the controller the bob's listener and stores
  it (`fishingBobLogic:` answers fish/shark/ice hole); `onFishingPoleDestroyBob:…` forgets it.
- Piranhas: `onNewPiranhaLogic:`/`onRemovePiranhaLogic:`/`onRemoveAllPiranhaLogics:` maintain
  `mPiranhaLogicArray` (answered by `piranhaLogicArray:`); Bait Master start/stop flag all pygmies.
- Bosses: visibility hooks refresh skins (4.13); hunts drive challenge games 7–9.

### 4.16 Mini-games

Common pattern (`on<Game>Start:pygmyLogic:`): `cancelAllTouches`, `closeScreens`, `hideUi`, set
`mb<Game>`, remember the pygmy, run `Start<Game>` on the game's own sequence (which stops the
background loop, pauses sleeping/hurricane/locust sounds, then calls `show<Game>DisplayGroup`).
`show<Game>DisplayGroup` (if not attached): `mbShow<Game>Assets` (selects the minigame atlas in
`loadLocationTextureAtlas`), `loadLocationTextureAtlas`, `IslandDisplayGroup` removed, game group
attached at `IslandRootDisplayMarker`, pygmy display object moved after `<Game>PygmyDisplayMarker`,
`updateLabelLayer`. Stop: flags off, `hide<Game>DisplayGroup` (reverse; pygmy back after
`PygmyDisplayMarker`), group x reset to 0 (Runs, Konkey Dong, Moron Pests, No Pee), `showUi`,
`[ChallengeManager stopGame:id]`, run `Stop<Game>` (resume sounds, stop the game loop,
`playBackgroundLoop`). Cameras: 3.2.

| Game | Started by | Flag / pygmy ivar | Group / pygmy marker | Atlas | Sequences | Score → game id |
|---|---|---|---|---|---|---|
| Ooga Jump | pygmy (`onPygmyStartDoodleJump:`): 50 `DoodleCloudLogic` on `BCMultiModel "DoodleCloud"` after `DoodleCloudDisplayMarker` | `mDoodleJumpPygmyLogic`, `mbDoodleJumpFall` | `DoodleDisplayGroup` (shown when the camera passes y −480; `SkyTiles` shown) / `DoodlePygmyDisplayMarker` | `OogaJumpTextureAtlas0` | — | height → 4 |
| Pain Drain | tile drain/plug (`onPainDrainStart:pygmyLogic:`) | `mbPainDrain`, `mPainDrainPygmyLogic` | `PainDrainDisplayGroup` replaces `UnderwaterDisplayGroup` at `UnderwaterDisplayMarker` / `PainDrainPygmyDisplayMarker` (shown directly, no start sequence) | `PainDrainTextureAtlas0` | — | 10 |
| The Runs | crack hole: `onPygmyFallIntoCrackHoleComplete:` → `[crackHole startTheRuns:]` when `TheRunsState` == 1, not underwater, no Ooga Jump | `mbTheRuns`, `mTheRunsPygmyLogic`, `mbTheRunsMoveCamera` | `TheRunsDisplayGroup` / `TheRunsPygmyDisplayMarker` | `TheRunsTextureAtlas0` | `StartTheRuns`, `StartTheRunsLoop` (camera on), `StopTheRuns`, shake `TheRunsShakeBackgroundStart/Stop` on `mEarthquakeSequence` (`stopTheRunsShakeScreen:::` resets y 0) | 11 |
| Konkey Dong | Kong statue (ape kidnap) | `mbKonkeyDong`, `mKonkeyDongPygmyLogic`, camera flags, `mKonkeyDongLevelYPos` | `KonkeyDongDisplayGroup` / `KonkeyDongPygmyDisplayMarker` | `KonkeyDongTextureAtlas0` | `StartKonkeyDong`, `StartKonkeyDongLoop`, `StopKonkeyDong` | 12 (incremented) |
| Moron Pests | moron snake | `mbMoronPests`, `mMoronPestsPygmyLogic`, `mbMoronPestsMoveCamera` | `MoronPestsDisplayGroup` / `MoronPestsPygmyDisplayMarker` | `MoronPestsTextureAtlas0` | `StartMoronPests`, `StartMoronPestsLoop`, `StopMoronPests` | 13 |
| No Pee | `NoPeeLogic` | `mbNoPee`, `mNoPeePygmyLogic`, `mbNoPeeMoveCamera` | `NoPeeDisplayGroup` / `NoPeePygmyDisplayMarker` | `NoPeeTextureAtlas0` | `StartNoPee`, `StopNoPee`; `onNoPeeRetry:` stops game 15 | 15 |
| Dance Dance Execution | DDE logic/screen (Dance temple) | `mbDanceDanceExecution` | its own screen logic (no group swap) | — | — | 14 |
| Battle of the Gods | temple door on God island | `mbBattleOfTheGods`, `mBattleOfTheGodsPygmyLogic` | `BattleArenaDisplayGroup` / `BattleOfTheGodsPygmyDisplayMarker`; shows load `MainBattleAtlas0` into the `TextureAtlas0` slot; hide reloads `TextureAtlas0` (deferred to `onIdolScreenClosed` when idol screens are open) | `BattleTextureAtlas0` | `StartBattleOfTheGods` (`BattleLoop`), `StopBattleOfTheGods` | — |
| Bait Master | anchor/piranhas (underwater toggle 1) | `mbBaitMaster` | — | — | — | 5 |
| Shark blast | chum bucket laser shark (toggle 2) | — | — | — | — | 6 |

Battle screens: the temple door calls `onOpenIdolBattleEnemyScreen` (cancel touches),
`onOpenIdolBattleScreen`, `onOpenIdolBattleResultsScreen` (`mbIdolBattleResultsScreenOpen`) → the
matching `IdolManager` screens; `onBattleAlertSelect` only logs. `show<Game>DisplayGroup:::` items
call the show methods. `isNoPeeActive`, `isDoodleJumpingPygmy` are asked by logics. Ending a game
because its pygmy died: 4.14.

### 4.17 Apocalypse and temples

- `startApocalypse` (from `XenuLogic`): flag, apocalypse gradient target 1, thetan meter
  `spawnApocalypseThetans`, `BackgroundLoop` stop + `ApocalypseLoop` play, episode event
  `StartApocalypse`. While active `playBackgroundLoop:`/`stopBackgroundLoop` use `ApocalypseLoop`.
- Gems: `onApocalypseGemEventTriggered:` (IdolManager) 1–7 → episode event `Spawn…Gem`, gem logic
  #403–409 (Time, Locust, Dance, Plague, Purification, Closet, Xenu) `activate` and added to
  `mApocalypseGemLogicArray` if inactive (only during apocalypse); `onApocalypseGemDie:` removes.
- `endApocalypse` (gem pedestals): flag off, gems `prepareToEndApocalypse`, Xenu `banish`, Xenu temple
  `buildWallBackUp`, calendar `closeApocalypsePortal`, `removeApocalypseThetans`; the calendar later
  calls `endApocalypseComplete` / `endApocalypseCompleteNow` (identical bodies): gradient target 0,
  `ApocalypseLoop` → `BackgroundLoop`, episode event `EndApocalypse`.
- `doesPlagueBodyExist` (any pygmy `isKilledByPlague`) and `ApocalypseCloudLogic doesDiseaseExist`
  drive the Rock island plague sign; going underwater cures all disease.
- Temples (loc 11–17): `showApocalypseTempleDisplayGroup` @ 0x1001680d0 (if the temple group is not
  attached): `loadLocationTextureAtlas`, `cancelAllTouches`, `IslandDisplayGroup` removed, temple group
  at `IslandRootDisplayMarker`, pygmies `moveToApocalypseTempleDisplayGroup`, thetans dropped in
  during apocalypse, gems `refreshDisplayMarker`, an active locust swarm moved after the temple's
  `DragDisplayMarker`. `hideApocalypseTempleDisplayGroup` @ 0x1001684e8 is the reverse (pygmies
  `moveFromApocalypseTempleDisplayGroup`, locusts back after `DragDisplayMarker`).
- `isInsideApocalypseTemple` (loc 11–17) is asked by pygmies.

### 4.18 Ads and URLs

- Chartboost (not ported): `showChartboostAd` (map screen) shows a cached `"Home Screen"`
  interstitial (`mbChartboostAdVisible`) or caches one; delegate callbacks re-cache and clear the flag.
  `AdBlockerLogic` asks `isChartboostAdVisible`.
- URLs: `openUrl:` stores `mLaunchUrl` and opens it (`openLaunchUrl`: stop the render loop,
  `UIApplication openURL:`) once the loading screen is gone; `openCountryURL:` picks the link of the
  device country from a `BCStructXmlLinkNodeDef` (falls back to `US`). News and bonus packs call
  `openUrl:`.

## 5. Touch handling

### 5.1 Pipeline

`BCView` queues UIKit touches and, at the start of each frame, calls the controller once per event:
`touchBegan:allTouches:`, `touchMoved:allTouches:`, `touchEnded:allTouches:`,
`touchCancelled:allTouches:` (= ended) — logic.md §9.1. `allTouches` = every finger currently down.
Each handler first stores `mAllTouches = allTouches` and `mTouchScreenPos = touch.pos` (view
position). Logic dispatch (`createTouchLogicArray:` @ 0x10006e730 fills `mTouchLogicArray` with the
selectable logics of the active display tree, walked front-to-back; `retain`/`releaseTouchLogicArray:`)
is specified in logic.md §9.2; this section adds the world gestures that run when no logic takes the
touch. Gesture state is one `BCTouch` reference per gesture (a finger can own at most one gesture at
begin, but the moved/ended handlers test the references independently).

### 5.2 `touchBegan:allTouches:` @ 0x10006eaa0

1. If Ooga Jump runs: `mbDoodleDebugPress = YES` (write-only).
2. **Pinch-sun pre-check** — when not Konkey Dong, not night, not underwater, exactly 2 fingers down,
   no active hurricane (`hurricaneActive`), the island has a `Sun` object and option
   `TemperatureState` == 1: both fingers' island positions within 120 of the sun centre → the other
   finger (the one that is not this event's touch) is ended through `touchEnded:allTouches:`;
   `mPinchSunState = mSunState`, `mPinchSunTouch0/1` = the two fingers, `mPinchSunDistance` = their
   distance; return.
3. Logic dispatch (logic.md §9.2). If a logic took the touch → return.
4. If any screen or minigame is active → return: help, debug, challenge, stats, skin pack, bonus
   pack, story, news, idol, map screens; Pain Drain, The Runs, Konkey Dong, Moron Pests, No Pee, DDE,
   Battle of the Gods.
5. Underwater (loc 2/9): `p` = `UnderwaterDisplayGroup` position of the touch;
   `checkUnderwaterBubble:pos:` (a `OneShotAnimLogic` `UnderwaterBubbles` at p in the underwater group,
   at most every 0.4 s of touch game time); no camera drag yet → `mUnderwaterTouch` = touch,
   `startDragUnderwater:p`; then if no power stroke and `isUnderwaterPowerActive` → `mUnderwaterPowerTouch`,
   `startDragUnderwaterPower:p`; else if no whirlpool finger → `mWhirlPoolTouch` = touch (the pool appears
   on the first move). Return.
6. Temples (11–17): `p` = temple group position; `checkForTapping:`; locust swarm active and no
   locust finger → `mLocustTouch`, `startDrawLocust:p`. Return.
7. Islands (`p` = `IslandDisplayGroup` position), first match wins:
   1. sun exists, not night, `|p − sun| < 40`, no sun/moon drag → `mDragSunTouch`, `startDragSun:p`;
   2. moon exists, night, `|p − moon| < 40`, no sun/moon drag → `mDragMoonTouch`,
      `checkForBatTapping:`, `startDragMoon:p`;
   3. `p.y > 250`, storm, no bolt alive, no lightning finger → `mLightningTouch`, `startDrawLightning:p`;
   4. otherwise: `checkForTapping:` (drum); then **in addition** if `p.y > horizon` and no cloud drag →
      `mDragCloudsTouch`, `startDragClouds:p timeStamp:touch.timeStamp`; then first of:
      - locust swarm active, no locust finger → `startDrawLocust:p`;
      - option `CrackHoleState` == 1, `min < p.x < max`, `50 < p.y < 120` → if cracks are active
        (`areCracksActive`) or a crack finger exists: nothing; else `mDragCrackTouch`,
        `startDragCrack:p timeStamp:`;
      - not Ape island, `p.y < horizon`, no ocean drag → `mDragOceanTouch`, `startDragOcean:p timeStamp:`;
      - Ape island, `|p − (wench.x + 54, wench.y)| < 130` and `[wench canCrankWench]` →
        `mApeWenchTouch`, `startApeWench:p`;
      - storm, no hurricane finger, `p.y > horizon`, option `HurricaneState` == 1 →
        `mHurricaneTouch`, `startHurricane:p`;
      - `p.y > horizon`, not night, no rainbow finger, option `RainbowState` == 1 → `mRainbowTouch`,
        `startDrawRainbow:p`.

### 5.3 `touchMoved:allTouches:` @ 0x10006f7fc

- A pinch finger: if both pinch fingers exist, `d = |p0 − p1| − mPinchSunDistance` (island
  positions): `d < −30` → pinch state 1 → 0, 0 → 2 (else nothing); `−30 <= d < 30` → back to the
  pinch-start state; `d >= 30` → 0 → 1, 2 → 0 (else nothing); each via `setSunState:`. Return.
- Sun finger → `dragSun:p`; moon finger → `dragMoon:p`. Return.
- Otherwise, independently: clouds (`mbDragClouds` and finger) → `dragClouds:timeStamp:`; crack →
  `dragCrack:timeStamp:` and **return**; ocean → `dragOcean:timeStamp:`; wench → `dragApeWench:p`.
- Then the first of: hurricane finger → `maintainHurricane:p`; else (underwater bubble check when
  underwater; camera finger → `dragUnderwater:` in addition) power finger → `dragUnderwaterPower:`;
  whirlpool finger → `maintainWhirlPool:`; lightning finger while drawing → `drawLightning:p` unless
  `mbShockObject`; else rainbow finger → `drawRainbow:` (return if the rainbow is being drawn); locust
  finger → `drawLocust:` (temple or island position; return if drawing); finally the logic dispatch
  (moved/entered/exited/inside/outside, logic.md §9.2).

Gesture moves: `dragOcean:timeStamp:` @ 0x10015fee8 is `dragClouds` without the storm branch (same
30° abandon rule, ±7000/±5000 clamps, smoothing 0.5, `applyDragOffset:dx` on both ocean layers).
`dragUnderwater:` (4.2). `dragApeWench:p` @ 0x100162228: angle change of the finger around the crank
centre (wench.x + 54, wench.y), wrapped to (−π, π], clamped to ±π/8 → `setApeWenchDeltaRot:`.
`dragCrack:` → `[crackHole drag:p]`.

### 5.4 `touchEnded:allTouches:` @ 0x1000701a0, `cancelAllTouches` @ 0x1000707c8

`mbDoodleDebugPress = NO`. A pinch finger → its reference cleared, return. Sun → `stopDragSun`;
moon → `stopDragMoon`; crack (while dragging) → `stopDragCrack`. Otherwise: clouds → `stopDragClouds`;
ocean → `stopDragOcean`; wench → `stopApeWench:` (`stopTurningApeWench`); then the first of: rainbow
finger (cleared; if drawing → `stopDrawRainbow:p`), locust finger (cleared; if drawing →
`stopDrawLocust:p`), hurricane → `stopHurricane:p`; else whirlpool finger → `stopWhirlPool:` and
camera finger → `stopDragUnderwater:` (both in addition), then power finger →
`stopDragUnderwaterPower:`, lightning finger while drawing → `stopDrawLightning:p`, else the logic
dispatch (ended inside/outside).

`cancelAllTouches` runs the ended-inside/outside logic loop for every touch in `mAllTouches` (with
`mCancelTouchLogicArray`) without clearing gesture references; called before screens open,
minigames start, modal views and temple/battle group swaps. `isPointLeftOfLineStartXPos:…` @
0x10006ea74 (cross product > 0) is a helper used by no controller code.

### 5.5 Gesture summary

| Gesture | Begin | Move | End | Effect |
|---|---|---|---|---|
| Pinch sun | 2 fingers near the sun, temperature on | distance ±30 | finger refs cleared | sun size |
| Sun drag | near the sun, day | `dragSun:` | `stopDragSun` | sunset/day blend, night below horizon−25 |
| Moon drag | near the moon, night | `dragMoon:` | `stopDragMoon` | sunset when below horizon−25; 3 taps → bat |
| Lightning | storm, y > 250 | `drawLightning:` | `stopDrawLightning:` | bolt |
| Clouds | above horizon | `dragClouds:` | `stopDragClouds` | flick ≥ 3000 toggles storm |
| Locust | swarm active | `drawLocust:` | `stopDrawLocust:` | swarm path |
| Crack | crack option, ground band | `dragCrack:` | `stopDragCrack` | crack hole (→ The Runs) |
| Ocean | below horizon, not Ape | `dragOcean:` | `stopDragOcean` | flick ≥ 3000 → underwater |
| Ape wench | near the crank | `dragApeWench:` | `stopApeWench:` | turns the wench |
| Hurricane | storm, above horizon, option | `maintainHurricane:` | `stopHurricane:` | hurricane follows the finger |
| Rainbow | above horizon, day, option | `drawRainbow:` | `stopDrawRainbow:` | double rainbow, dance |
| Underwater camera | underwater | `dragUnderwater:` | `stopDragUnderwater:` | swipe down > 100 leaves |
| Underwater power | coral temple mode ≠ 0 | `dragUnderwaterPower:` | `stopDragUnderwaterPower:` | power on pygmies |
| Whirlpool | underwater otherwise | `maintainWhirlPool:` | `stopWhirlPool:` | whirlpool |
| Drum | any world tap, drum option | — | — | dance after 4 taps |
| Bubbles | underwater began/moved | — | — | bubble effect |

## 6. Listener protocols

The class declares 18 protocols and the `Game` category 101; 11 are declared on both, 107 are distinct (`tmp/re/objc_meta.txt`). They contain 464 distinct selectors; the controller implements 436 of them. The 28 it does not implement are optional `ChartboostDelegate` methods (more-apps, rewarded-video and in-play callbacks), which the SDK then skips. Almost every protocol is the listener interface of one logic or manager class (`XLogicListener` → `XLogic`), so a selector shared by several protocols (e.g. `optionPreferences`, `islandLocationState`, `isStoryActive`) is one controller method answering many senders.

### 6.1 Protocols and their methods

"On": where the conformance is declared (`class`, `Game` category or both). Methods in declaration order; the descriptions are in 6.2.

| Protocol | On | Methods |
|---|---|---|
| `AnchorLogicListener` | Game | `piranhaLogicArray:`, `pygmyLogicArray:`, `onAnchorDrag:`, `onAnchorDragComplete:`, `onNewPiranhaLogic:`, `onRemovePiranhaLogic:`, `onRemoveAllPiranhaLogics:`, `onBaitMasterStart:`, `onBaitMasterStop:`, `onPiranhaEatPygmy:` |
| `AntHillLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `isAntHillEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onAntHillDirect:`, `onAntHillDirectComplete:` |
| `ApeAltarLogicListener` | Game | `apeWenchLogic:` |
| `ApeBongoLogicListener` | Game | — |
| `ApeLogicListener` | Game | `isApeEnabled`, `apeAltarLogic:`, `apeBongoLogic:`, `apeWenchLogic:`, `apeTorchLogicArray:`, `onApeVisible:` |
| `ApeTorchLogicListener` | Game | `islandCenterXPos`, `apeLogic:` |
| `ApeWenchLogicListener` | Game | `apeAltarLogic:` |
| `ApocalypseCalendarLogicListener` | Game | `isStoryActive`, `isTomPygmyActive`, `isApocalypseActive`, `endApocalypseComplete`, `endApocalypseCompleteNow`, `apocalypseGemPedestalLogicArray:` |
| `ApocalypseGemLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `isOceanEnabled`, `isApocalypseGemPedestalEnabled`, `isApocalypseActive`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyDragDisplayMarker`, `pygmyDisplayMarker`, `apocalypseGemPedestalLogicArray:`, `onApocalypseGemDrag:`, `onApocalypseGemDragComplete:`, `onApocalypseGemDie:` |
| `ApocalypseGemPedestalLogicListener` | Game | `isStoryActive`, `isApocalypseActive`, `apocalypseGemPedestalLogicArray:`, `endApocalypse` |
| `ApocalypseRoomFaderLogicListener` | Game | `islandLocationState`, `onApocalypseRoomFaderChangeIslandLocationState:` |
| `ApocalypseTempleDoorLogicListener` | Game | `isStoryActive`, `islandLocationState`, `pygmyLogicArray:` |
| `ApocalypseTempleFountainLogicListener` | Game | `isStoryActive` |
| `ApocalypseTimeTempleLeverLogicListener` | Game | `isStoryActive`, `apocalypseTimeTemplePlatformLogic:` |
| `ApocalypseTimeTemplePlatformLogicListener` | Game | `isStoryActive` |
| `BananaLogicListener` | Game | `isGravityEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyLogicArray:`, `onBananaDrag:`, `onBananaDragComplete:` |
| `BananaTreeLogicListener` | Game | `isStoryActive`, `isBananaTreeEnabled`, `isMonkeyEnabled`, `bananaLogic:`, `monkeyLogicArray:` |
| `BatLogicListener` | Game | `isOceanEnabled`, `isGravityEnabled`, `pygmyLogicArray:`, `onBatDie:` |
| `BattleAlertLogicListener` | both | `isScreenOpen`, `isStoryActive`, `onBattleAlertSelect` |
| `BCButtonLogicListener` | class | `onButtonPress:touch:`, `onButtonRelease:touch:`, `onButtonReleaseOutside:touch:` |
| `BCGameFrameListener` | Game | `id`, `onGameFrame:`, `gameFrameOrder` |
| `BCPreferencesListener` | both | `onPreferencesChanged:` |
| `BCSequenceDelegate` | both | `id` |
| `BCViewDelegate` | class | `preStartTouchBeganXPos:yPos:`, `touchBegan:allTouches:`, `touchMoved:allTouches:`, `touchEnded:allTouches:`, `touchCancelled:allTouches:` |
| `BirdLogicListener` | Game | `isOceanEnabled`, `isBonFireEnabled`, `isBirdEnabled`, `isStoryActive`, `islandMinXPos`, `islandMaxXPos`, `bonFireLogic:`, `fishLogic:`, `pygmyLogicArray:`, `onBirdDrag:`, `onBirdDragComplete:`, `onBirdPooHitPygmy:`, `onBirdPooMissPygmy:`, `onBirdFlyOffScreen:` |
| `BonFireLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `isBonFireEnabled`, `fishingPoleLogic:`, `pygmyLogicArray:` |
| `BonusPackManagerListener` | both | `onBonusPackScreenClosed`, `onBonusPackUnlockDialogOpened`, `onBonusPackUnlockDialogClosed`, `onLoadLocationTextureAtlas`, `onShowKeyboard:editingText:cancelEditDisplayObject:maxEditCharCount:preventEmpty:`, `onHideKeyboard`, `isKeyboardOpen`, `openUrl:` |
| `BoulderLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `isBoulderEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onBoulderDrag:`, `onBoulderDragComplete:`, `onPygmiesBowled:` |
| `ChallengeManagerListener` | Game | `godPersona`, `onLoadLocationTextureAtlas`, `onChallengeDialogOpen`, `onChallengeScreenClosed`, `onStartChallenge:`, `onEndChallenge:`, `onAchievement:count:highScoreWins:`, `presentModalViewController:`, `dismissModalViewController` |
| `ChartboostDelegate` | class | `shouldRequestInterstitial:`, `shouldDisplayInterstitial:`, `didDisplayInterstitial:`, `didCacheInterstitial:`, `didFailToLoadInterstitial:withError:`, `didFailToRecordClick:withError:`, `didDismissInterstitial:`, `didCloseInterstitial:`, `didClickInterstitial:`; not implemented: `shouldDisplayMoreApps:`, `didDisplayMoreApps:`, `didCacheMoreApps:`, `didDismissMoreApps:`, `didCloseMoreApps:`, `didClickMoreApps:`, `didFailToLoadMoreApps:withError:`, `shouldDisplayRewardedVideo:`, `didDisplayRewardedVideo:`, `didCacheRewardedVideo:`, `didFailToLoadRewardedVideo:withError:`, `didDismissRewardedVideo:`, `didCloseRewardedVideo:`, `didClickRewardedVideo:`, `didCompleteRewardedVideo:withReward:`, `didCacheInPlay:`, `didFailToLoadInPlay:withError:`, `willDisplayVideo:`, `didCompleteAppStoreSheetFlow`, `didPauseClickForConfirmation`, `shouldDisplayMoreApps`, `didDisplayMoreApps`, `didCacheMoreApps`, `didDismissMoreApps`, `didCloseMoreApps`, `didClickMoreApps`, `didFailToLoadMoreApps:`, `didLoadInPlay` |
| `ChumBucketLogicListener` | Game | `pygmyLogicArray:`, `onLaserSharkZapPygmy:`, `onLaserSharkZapComplete:` |
| `ClosetTempleClosetDoorLogicListener` | Game | `closetTempleThetanMeterLogic:`, `isTomPygmyAlive`, `createTomPygmy`, `onClosetTempleClosetDoorDrag:`, `onClosetTempleClosetDoorDragComplete:` |
| `ClosetTempleCouchLogicListener` | Game | — |
| `ClosetTempleThetanMeterLogicListener` | Game | `pygmyLogicArray:`, `locustDisplayMarker`, `isStoryActive`, `isApocalypseActive`, `onThetanDrag:`, `onThetanDragComplete:` |
| `CloudLogicListener` | Game | `onCloudDie:` |
| `CoconutLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onCoconutDrag:`, `onCoconutDragComplete:`, `onCoconutHitPygmy:`, `onCoconutHitGround:`, `onCoconutDie:` |
| `CrackHoleLogicListener` | Game | `pygmyLogicArray:`, `islandMinXPos`, `islandMaxXPos`, `onTheRunsStart:pygmyLogic:`, `onTheRunsStop:`, `onTheRunsScore:`, `onTheRunsStartMovingCamera`, `onTheRunsShakeScreenStart`, `onTheRunsShakeScreenStop` |
| `DanceBalloonBundleLogicListener` | Game | `isStoryActive`, `danceBalloonLogic:` |
| `DanceBalloonLogicListener` | Game | `pygmyLogicArray:`, `pygmyDragDisplayMarker`, `onDanceBalloonDrag:`, `onDanceBalloonDragComplete:` |
| `DanceDanceExecutionLogicListener` | Game | `stopBackgroundLoop`, `playBackgroundLoop:`, `onDanceDanceExecutionStart:` |
| `DanceDanceExecutionScreenLogicListener` | Game | `onDanceDanceExecutionStart:`, `onDanceDanceExecutionStop:`, `onDanceDanceExecutionScore:` |
| `DebugScreenListener` | both | `onDebugScreenClosed`, `onLoadLocationTextureAtlas`, `systemPreferences`, `challengePreferences` |
| `DiscoBallLogicListener` | Game | `stopBackgroundLoop`, `playBackgroundLoop:` |
| `DiscoLightsLogicListener` | Game | — |
| `DoodleCloudLogicListener` | Game | — |
| `DreamBubbleLogicListener` | Game | `islandLocationState`, `onDreamBubbleChangeIslandLocationState:` |
| `EpisodeAlertLogicListener` | both | `optionPreferences`, `isScreenOpen`, `isStoryActive`, `onEpisodeAlertOpenChallengeDetail:` |
| `FailBoatLogicListener` | Game | `isStoryActive` |
| `FireworkLogicListener` | Game | `onFireworkDie:` |
| `FishingBobLogicListener` | Game | `isGravityEnabled`, `fishingPoleLogic:`, `onFishingBobDrag:`, `onFishingBobDragComplete:` |
| `FishingPoleLogicListener` | Game | `isStoryActive`, `isBonFireEnabled`, `isGravityEnabled`, `isFishEnabled`, `islandMinXPos`, `islandMaxXPos`, `bonFireLogic:`, `pygmyLogicArray:`, `onFishingPoleDrag:`, `onFishingPoleDragComplete:`, `onFishingPoleCreateBob:fishingBobLogic:`, `onFishingPoleDestroyBob:fishingBobLogic:` |
| `FishLogicListener` | Game | `isStoryActive`, `isOceanEnabled`, `isBonFireEnabled`, `isGravityEnabled`, `isFishEnabled`, `isIglooEnabled`, `isIceHoleEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `iceHoleLogic:`, `iglooLogic:`, `bonFireLogic:`, `fishingBobLogic:`, `pygmyLogicArray:`, `onFishDrag:`, `onFishDragComplete:` |
| `GameServerAchievementListener` | class | — |
| `GameServerLeaderboardListener` | class | — |
| `GameServerListener` | class | `presentModalViewController:`, `dismissModalViewController`, `checkAllAchievementsAndLeaderboards` |
| `GameUiManagerListener` | both | — |
| `GlacierLogicListener` | Game | — |
| `GodIdolLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `isGodIdolEnabled`, `isIdolPedestalEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `idolPedestalLogicArray:`, `onGodIdolDrag:`, `onGodIdolDragComplete:` |
| `GoGoCageLogicListener` | Game | — |
| `HurricaneLogicListener` | Game | `spriteLogicArray:`, `onHurricaneCantPickUpObjects:`, `onHurricaneDie:` |
| `IceHoleLogicListener` | Game | `isIceSquidEnabled`, `islandMinXPos`, `islandMaxXPos`, `draggedPygmyLogic:`, `swordFishLogic:`, `fishingPoleLogic:`, `pygmyLogicArray:` |
| `IceMonsterLogicListener` | Game | `islandMinXPos`, `islandMaxXPos`, `healthBarLogic:`, `pygmyLogicArray:`, `onSnowBallHitPygmy:`, `onSnowBallMissPygmy:`, `onIceMonsterVisible:`, `onIceMonsterHitByPygmy:`, `onIceMonsterKilled:`, `onIceMonsterStartBattle:`, `onIceMonsterCancelBattle:`, `onAchievement:count:highScoreWins:` |
| `IceTreeLogicListener` | Game | — |
| `IdolManagerListener` | Game | `challengePreferences`, `optionPreferences`, `isScreenOpen`, `isStoryActive`, `isAnyGameActive`, `isDialogActive`, `isStormCloudEnabled`, `isNightActive`, `isSunsetActive`, `islandLocationState`, `godIdolLogicArray`, `onApocalypseGemEventTriggered:`, `onIdolAwardScreenClosed`, `onIdolAwardScreenOpened`, `onIdolAwardScreenOpenChallenges`, `onIdolAwardScreenChangeIslandLocationState:`, `onIdolScreenClosed`, `onIdolScreenLaunchStoreScreen`, `onLoadLocationTextureAtlas`, `presentModalViewController:`, `dismissModalViewController` |
| `IdolPedestalLogicListener` | Game | `isStoryActive`, `optionPreferences`, `templeDoorLogic:` |
| `IglooLogicListener` | Game | — |
| `JellyfishLogicListener` | Game | `pygmyLogicArray:`, `jellyfishLogicArray:`, `onJellyfishDrag:`, `onJellyfishDragComplete:` |
| `KongStatueLogicListener` | Game | `pygmyLogicArray:`, `isKonkeyDongEnabled`, `addPygmy`, `onKonkeyDongStart:pygmyLogic:`, `onKonkeyDongStop:`, `onKonkeyDongScore:`, `onKonkeyDongStartMovingCamera`, `onKonkeyDongMoveCameraToNextLevel:` |
| `LightningLogicListener` | Game | `onLightningDie:` |
| `LocustLogicListener` | Game | `pygmyLogicArray:`, `locustDisplayMarker`, `startDrawLocust:`, `drawLocust:`, `stopDrawLocust:` |
| `LocustQueenLogicListener` | Game | `isStoryActive`, `locustLogic:`, `onLocustQueenVisible:` |
| `MagnifyingGlassLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `isAntHillEnabled`, `isDayActive`, `isStormActive`, `islandMinXPos`, `islandMaxXPos`, `antHillLogic:`, `pygmyLogicArray:`, `onMagnifyingGlassDrag:`, `onMagnifyingGlassDragComplete:` |
| `MapScreenListener` | both | `islandLocationState`, `mapPaperLogic`, `hasMisfitIslandBeenFound`, `onMapScreenClosed`, `onMapScreenChangeIslandLocationState:`, `onLoadLocationTextureAtlas`, `showChartboostAd`, `isChartboostAdVisible` |
| `MonkeyLogicListener` | Game | `isDanceActive`, `isGravityEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `apeTorchLogicArray:`, `pygmyLogicArray:`, `monkeyLogicArray:`, `bananaLogic:`, `bananaTreeLogic:`, `onMonkeyDrag:`, `onMonkeyDragComplete:` |
| `MoonLogicListener` | Game | `starsLogic:` |
| `MoronSnakeLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `isMoronSnakeEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyLogicArray:`, `onMoronPestsStart:pygmyLogic:`, `onMoronPestsStartMovingCamera`, `onMoronPestsStopMovingCamera`, `onMoronPestsStop:`, `onMoronPestsScore:`, `onMoronSnakeDrag:`, `onMoronSnakeDragComplete:` |
| `NewsManagerListener` | both | `systemPreferences`, `mapPaperLogic`, `onNewsLaunchStoreScreen`, `onNewsLaunchChallengeScreen`, `onNewsLaunchStoryScreen`, `onNewsTeaserLaunchNewsScreen`, `onNewsTeaserClosed`, `onNewsScreenClosed`, `onLoadLocationTextureAtlas`, `openUrl:` |
| `NoPeeLogicListener` | Game | `pygmyLogicArray:`, `islandMinXPos`, `islandMaxXPos`, `onNoPeeStart:pygmyLogic:`, `onNoPeeStop:`, `onNoPeeRetry:`, `onNoPeeStartMovingCamera` |
| `OctoStatueLogicListener` | Game | `isStoryActive`, `pygmyLogicArray:` |
| `OutHouseLogicListener` | Game | — |
| `PygmyBottomLogicListener` | Game | `isOceanEnabled` |
| `PygmyLogicListener` | Game | `isOceanEnabled`, `isCoconutEnabled`, `isWoodEnabled`, `isBonFireEnabled`, `isVolcanoEnabled`, `isGravityEnabled`, `isOutHouseEnabled`, `isZombieGraveEnabled`, `isIdolPedestalEnabled`, `isTempleDoorEnabled`, `isApocalypseTempleDoorEnabled`, `isApocalypsePortalTempleDoorEnabled`, `isApocalypseTimeTemplePlatformEnabled`, `isLocustQueenEnabled`, `isApocalypsePlagueTempleFountainEnabled`, `isApocalypsePurificationTempleFountainEnabled`, `isClosetTempleThetanMeterEnabled`, `isClosetTempleCouchEnabled`, `isClosetTempleClosetDoorEnabled`, `isXenuTempleEnabled`, `isXenuEnabled`, `isTomPygmyAlive`, `isTimeTempleDoorEnabled`, `isLocustTempleDoorEnabled`, `isDanceTempleDoorEnabled`, `isPlagueTempleDoorEnabled`, `isPurificationTempleDoorEnabled`, `isClosetTempleDoorEnabled`, `isXenuTempleDoorEnabled`, `areNameLabelsEnabled`, `areGhostsEnabled`, `isSpiderTreeEnabled`, `isIceMonsterEnabled`, `isIceHoleEnabled`, `isGlacierEnabled`, `isIglooEnabled`, `isApeWenchEnabled`, `isApeBongoEnabled`, `isApeAltarEnabled`, `isGoGoCageEnabled`, `isDanceDanceExecutionEnabled`, `isKongStatueEnabled`, `isMoronPestsEnabled`, `isJellyfishEnabled`, `isDreamBubbleEnabled`, `isInsideApocalypseTemple`, `isStormCloudEnabled`, `isDoodleCloudEnabled`, `isDanceActive`, `isNightActive`, `isSunsetActive`, `isEruptionActive`, `isEarthquakeActive`, `isEclipseActive`, `isTRexActive`, `isSpiderActive`, `isLocustActive`, `isApocalypseActive`, `isDraggingPygmy`, `isDoodleJumpingPygmy`, `canTouchPygmy`, `isLaserSharkEnabled`, `canWarlockUseLightning:`, `sunState`, `islandLocationState`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `rainbowLogic:`, `crackHoleLogic:`, `kongStatueLogic:`, `moronSnakeLogic:`, `noPeeLogic:`, `tileDrainLogic:`, `iceHoleLogic:`, `iceMonsterLogic:`, `glacierLogic:`, `iglooLogic:`, `apeWenchLogic:`, `apeBongoLogic:`, `apeAltarLogic:`, `goGoCageLogic:`, `danceDanceExecutionLogic:`, `bonFireLogic:`, `tRexLogic:`, `tRexEggLogic:`, `spiderLogic:`, `spiderTreeLogic:`, `outHouseLogic:`, `zombieGraveLogic:`, `birdLogic:`, `bananaLogic:`, `fishLogic:`, `swordFishLogic:`, `fishingPoleLogic:`, `waterWingsLogic:`, `sharkLogic:`, `boulderLogic:`, `templeDoorLogic:`, `apocalypseTempleDoorLogic:`, `apocalypsePortalTempleDoorLogic:`, `apocalypseTimeTemplePlatformLogic:`, `locustQueenLogic:`, `locustLogic:`, `apocalypsePlagueTempleFountainLogic:`, `apocalypsePurificationTempleFountainLogic:`, `closetTempleThetanMeterLogic:`, `closetTempleCouchLogic:`, `closetTempleClosetDoorLogic:`, `xenuTempleLogic:`, `xenuLogic:`, `tomPygmyLogic:`, `timeTempleDoorLogicArray:`, `locustTempleDoorLogicArray:`, `danceTempleDoorLogicArray:`, `plagueTempleDoorLogicArray:`, `purificationTempleDoorLogicArray:`, `closetTempleDoorLogicArray:`, `xenuTempleDoorLogicArray:`, `idolPedestalLogicArray:`, `monkeyLogicArray:`, `jellyfishLogicArray:`, `cloudLogicArray:`, `doodleCloudLogicArray:`, `coconutLogicArray:`, `woodLogicArray:`, `pygmyLogicArray:`, `pygmyDragDisplayMarker`, `pygmyDisplayMarker`, `pygmyIslandDisplayMarker`, `pygmySmallObjectsDisplayMarker`, `pygmyUnderwaterPygmyDisplayMarker`, `pygmyIslandBackDisplayMarker`, `pygmyCloudBehindDisplayMarker`, `onPygmySleepingStart:`, `onPygmySleepingStop:`, `onPygmyDrag:`, `onPygmyDragComplete:`, `onPygmyGhostDrag:`, `onPygmyGhostDragComplete:`, `onPygmyTumble:`, `onPygmyTumbleComplete:`, `onPygmyExplode:`, `onPygmySacrificeEvent:`, `onTriggerSacrifice:sacrificeId:`, `onPygmyDie:sacrificeId:`, `onPygmyGiftGiven:giftId:`, `onPygmyStartDoodleJump:`, `onPygmyStopDoodleJump:`, `onPygmyDoodleJumpFall:`, `onPygmyFallIntoCrackHoleComplete:`, `onPygmyHitByLightningInHurricane:`, `hitPygmyWithWarlockLightning:`, `onPygmyEarthquakeStart:`, `onPygmyEarthquakeStop:`, `onVolcanoHit:`, `onKonkeyDongScore:`, `onKonkeyDongStop:`, `startApocalypse` |
| `RainbowLogicListener` | Game | `isStormActive`, `pygmyLogicArray:` |
| `SharkLogicListener` | Game | `isOceanEnabled`, `isGravityEnabled`, `isSharkEnabled`, `isStoryActive`, `islandMinXPos`, `islandMaxXPos`, `fishingPoleLogic:`, `fishingBobLogic:`, `pygmyLogicArray:`, `onSharkDrag:`, `onSharkDragComplete:`, `onSharkSnap:` |
| `SkinPackManagerListener` | both | `onSkinPackScreenClosed`, `onLoadLocationTextureAtlas` |
| `SpearLogicListener` | Game | `isGravityEnabled`, `isSpearEnabled`, `isTRexActive`, `isSpiderActive`, `islandMinXPos`, `islandMaxXPos`, `tRexLogic:`, `spiderLogic:`, `pygmyLogicArray:`, `onSpearDrag:`, `onSpearDragComplete:`, `onSpearThrow:` |
| `SpearStatueLogicListener` | Game | `pygmyLogicArray:`, `onPygmiesSkewered:` |
| `SpiderLogicListener` | Game | `isGravityEnabled`, `isSpiderEnabled`, `isSpearEnabled`, `islandMinXPos`, `islandMaxXPos`, `healthBarLogic:`, `spearLogicArray:`, `pygmyLogicArray:`, `onSpiderVisible:`, `onSpiderHitBySpear:`, `onSpiderKilledBySpear:`, `onSpiderCancelHunt:`, `onAchievement:count:highScoreWins:` |
| `SpiderTreeLogicListener` | Game | `isSpiderEnabled`, `spiderLogic:`, `pygmyLogicArray:` |
| `StoryManagerListener` | both | `systemPreferences`, `tribePersona`, `onStoryScreenClosed`, `onLoadLocationTextureAtlas`, `onStoryStart`, `onStoryComplete`, `onStoryCancel`, `onStoryEnvironmentChange:`, `onStoryAddPygmy:`, `onShowKeyboard:editingText:cancelEditDisplayObject:maxEditCharCount:preventEmpty:`, `onHideKeyboard`, `isKeyboardOpen`, `onStoryModeStartCelebration`, `onStoryModeStopCelebration`, `onStoryModeStartEarthquake`, `onStoryModeStopEarthquake`, `onStoryModeSharkSnap`, `onStoryModeActivateRainbow`, `onStoryModeActivateMyRainbow`, `onStoryModeDeactivateRainbow` |
| `SunLogicListener` | Game | — |
| `SwordFishLogicListener` | Game | `isStoryActive`, `isOceanEnabled`, `isGravityEnabled`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyLogicArray:`, `onSwordFishDrag:`, `onSwordFishDragComplete:` |
| `TempleDoorLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `isTempleDoorEnabled`, `islandLocationState`, `islandMinXPos`, `islandMaxXPos`, `islandCenterXPos`, `pygmyLogicArray:`, `createPygmyLogic`, `onBattleOfTheGodsStart:pygmyLogic:`, `onBattleOfTheGodsStop:`, `onOpenIdolBattleEnemyScreen`, `onOpenIdolBattleScreen`, `onOpenIdolBattleResultsScreen` |
| `TileDrainLogicListener` | Game | `pygmyLogicArray:`, `tilePlugLogic:`, `onPainDrainStart:pygmyLogic:`, `onPainDrainStop:`, `onPainDrainScore:` |
| `TilePlugLogicListener` | Game | `pygmyLogicArray:`, `tileDrainLogic:`, `onTilePlugDrag:`, `onTilePlugDragComplete:`, `onPainDrainStart:pygmyLogic:`, `onPainDrainStop:` |
| `TRexEggLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `isTRexEggEnabled`, `isTRexEnabled`, `islandMinXPos`, `islandMaxXPos`, `tRexLogic:`, `pygmyLogicArray:`, `onTRexEggDrag:`, `onTRexEggDragComplete:`, `onPygmiesBowled:` |
| `TRexLogicListener` | Game | `isGravityEnabled`, `isTRexEnabled`, `isSpearEnabled`, `islandMinXPos`, `islandMaxXPos`, `healthBarLogic:`, `spearLogicArray:`, `pygmyLogicArray:`, `onTRexVisible:`, `onTRexHitBySpear:`, `onTRexKilledBySpear:`, `onTRexCancelHunt:`, `onAchievement:count:highScoreWins:` |
| `TsunamiLogicListener` | Game | `islandLocationState`, `lastIslandLocationState`, `chumBucketLogic:`, `anchorLogic:`, `tileDrainLogic:`, `tilePlugLogic:`, `spearStatueLogic:`, `underwaterSceneryLogicArray:`, `onTsunamiActivateComplete:`, `onTsunamiDeactivateComplete:`, `onTsunamiHideIslandAssets:`, `onTsunamiShowIslandAssets:` |
| `UITextFieldDelegate` | class | `textFieldShouldBeginEditing:`, `textFieldDidBeginEditing:`, `textFieldShouldEndEditing:`, `textFieldDidEndEditing:`, `textField:shouldChangeCharactersInRange:replacementString:`, `textFieldShouldClear:`, `textFieldShouldReturn:` |
| `UnderwaterTempleLogicListener` | Game | `onUnderwaterTempleModeChanged:` |
| `VolcanoLogicListener` | Game | `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onEruptionComplete:` |
| `WaterWingsLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `areWaterWingsEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `onWaterWingsDrag:`, `onWaterWingsDragComplete:` |
| `WhirlPoolLogicListener` | Game | `spriteLogicArray:`, `onWhirlPoolCantPickUpObjects:`, `onWhirlPoolDie:` |
| `WoodLogicListener` | Game | `isStoryActive`, `isGravityEnabled`, `isWoodEnabled`, `islandMinXPos`, `islandMaxXPos`, `pygmyLogicArray:`, `bonFireLogic:`, `onWoodDrag:`, `onWoodDragComplete:` |
| `XenuLogicListener` | Game | `startApocalypse` |
| `XenuTempleLogicListener` | Game | `isBabyPygmyActive`, `isTomPygmyActive` |
| `ZombieGraveLogicListener` | Game | `pygmyLogicArray:` |

### 6.2 Implemented protocol methods, by unit

One line per selector, grouped by the unit of section 9 that owns it. Address suffix `G` = `Game` category (`PocketGodViewController+Game.c`), none = main class. "Protocols" is the number of protocols that declare the selector. `loc` and option names as in section 0; predicates give the exact condition evaluated.

#### 6.2.1 `app_platform` (27)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `didCacheInterstitial:` | 0x1000883e4 | 1 | Chartboost: log |
| `didClickInterstitial:` | 0x10008853c | 1 | Chartboost: re-cache, ad hidden |
| `didCloseInterstitial:` | 0x1000884d8 | 1 | Chartboost: re-cache, ad hidden |
| `didDismissInterstitial:` | 0x100088474 | 1 | Chartboost: re-cache, ad hidden |
| `didDisplayInterstitial:` | 0x1000883bc | 1 | Chartboost: log |
| `didFailToLoadInterstitial:withError:` | 0x10008840c | 1 | Chartboost: log |
| `didFailToRecordClick:withError:` | 0x10008844c | 1 | Chartboost: log |
| `dismissModalViewController` | 0x10006d388 | 3 | dismisses the presented UIKit modal, tells the app delegate (render loop restarts) |
| `id` | 0x10006e6a4 | 2 | returns @"PocketGodViewController" (BCSequenceDelegate / BCGameFrameListener id) |
| `isChartboostAdVisible` | 0x100088354 | 1 | mbChartboostAdVisible |
| `isKeyboardOpen` | 0x10006da88 | 2 | mbKeyboardOpen |
| `onHideKeyboard` | 0x10006d994 | 2 | closes text entry: restores saved text, hides cancel button, hides keyboard |
| `onShowKeyboard:editingText:cancelEditDisplayObject:maxEditCharCount:preventEmpty:` | 0x10006d84c | 2 | opens text entry for a BCText on behalf of a manager |
| `openUrl:` | 0x10006e9d4 | 2 | stores the URL; opens now if the loading screen is gone |
| `playBackgroundLoop:` | 0x100070ee8 | 2 | optionally re-aliases BackgroundLoop to a sound, then plays BackgroundLoop (ApocalypseLoop during apocalypse) if not playing |
| `presentModalViewController:` | 0x10006d2dc | 3 | once: cancel touches, clear BCView touches, tell the app delegate, present |
| `shouldDisplayInterstitial:` | 0x100088390 | 1 | Chartboost: YES (logs) |
| `shouldRequestInterstitial:` | 0x100088364 | 1 | Chartboost: YES (logs) |
| `showChartboostAd` | 0x100088244 | 1 | shows a cached "Home Screen" interstitial (else caches one); returns whether shown |
| `stopBackgroundLoop` | 0x100168eec G | 2 | stops BackgroundLoop (ApocalypseLoop during apocalypse) |
| `textField:shouldChangeCharactersInRange:replacementString:` | 0x10006d4f0 | 1 | applies the edit to the BCText, rejects beyond mMaxEditCharCount |
| `textFieldDidBeginEditing:` | 0x10006d5ac | 1 | empty |
| `textFieldDidEndEditing:` | 0x10006d5b8 | 1 | closes text entry, hands the text to mKeyboardListener (onKeyboardTextEntered:) |
| `textFieldShouldBeginEditing:` | 0x10006d5a4 | 1 | YES |
| `textFieldShouldClear:` | 0x10006d6b0 | 1 | YES |
| `textFieldShouldEndEditing:` | 0x10006d5b0 | 1 | YES |
| `textFieldShouldReturn:` | 0x10006d6b8 | 1 | commits the stats-page field (tribe/god/pygmy name or description) to its persona, resigns first responder |

#### 6.2.2 `boot_and_frame` (8)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `challengePreferences` | 0x10006dac8 | 2 | returns the "Challenge" BCPreferences |
| `gameFrameOrder` | 0x1001585b4 G | 1 | returns 50 |
| `godPersona` | 0x100070db4 | 1 | returns mGodPersona |
| `onGameFrame:` | 0x1001585bc G | 1 | per-frame controller update (section 3.2) |
| `onPreferencesChanged:` | 0x10007bd80 | 1 | god and tribe personas refreshFromPreferences (iCloud merge) |
| `optionPreferences` | 0x10006daa8 | 3 | returns the "Options" BCPreferences |
| `systemPreferences` | 0x10006dab8 | 3 | returns the "System" BCPreferences |
| `tribePersona` | 0x1000715e4 | 1 | returns mTribePersona |

#### 6.2.3 `world_registry` (87)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `anchorLogic:` | 0x100071514 | 1 | returns mAnchorLogic (argument ignored) |
| `antHillLogic:` | 0x100071564 | 1 | returns mAntHillLogic (argument ignored) |
| `apeAltarLogic:` | 0x100071254 | 3 | returns mApeAltarLogic (argument ignored) |
| `apeBongoLogic:` | 0x100071264 | 2 | returns mApeBongoLogic (argument ignored) |
| `apeLogic:` | 0x100071244 | 1 | returns mApeLogic (argument ignored) |
| `apeTorchLogicArray:` | 0x100071284 | 2 | returns mApeTorchLogicArray (argument ignored) |
| `apeWenchLogic:` | 0x100071274 | 3 | returns mApeWenchLogic (argument ignored) |
| `apocalypseGemPedestalLogicArray:` | 0x1000712c4 | 3 | returns mApocalypseGemPedestalLogicArray (argument ignored) |
| `apocalypsePlagueTempleFountainLogic:` | 0x100071464 | 1 | returns mApocalypsePlagueTempleFountainLogic (argument ignored) |
| `apocalypsePortalTempleDoorLogic:` | 0x1000713c4 | 1 | returns mApocalypsePortalTempleDoorLogic (argument ignored) |
| `apocalypsePurificationTempleFountainLogic:` | 0x100071474 | 1 | returns mApocalypsePurificationTempleFountainLogic (argument ignored) |
| `apocalypseTempleDoorLogic:` | 0x1000713b4 | 1 | returns mApocalypseTempleDoorLogic (argument ignored) |
| `apocalypseTimeTemplePlatformLogic:` | 0x1000713e4 | 2 | returns mApocalypseTimeTemplePlatformLogic (argument ignored) |
| `bananaLogic:` | 0x100071364 | 3 | returns mBananaLogic (argument ignored) |
| `bananaTreeLogic:` | 0x100071354 | 1 | returns mBananaTreeLogic (argument ignored) |
| `birdLogic:` | 0x1000711c4 | 1 | returns mBirdLogic (argument ignored) |
| `bonFireLogic:` | 0x100071574 | 5 | returns mBonFireLogic (argument ignored) |
| `boulderLogic:` | 0x100071134 | 1 | returns mBoulderLogic (argument ignored) |
| `chumBucketLogic:` | 0x100071544 | 1 | returns mChumBucketLogic (argument ignored) |
| `closetTempleClosetDoorLogic:` | 0x1000714a4 | 1 | returns mClosetTempleClosetDoorLogic (argument ignored) |
| `closetTempleCouchLogic:` | 0x100071494 | 1 | returns mClosetTempleCouchLogic (argument ignored) |
| `closetTempleDoorLogicArray:` | 0x100071334 | 1 | returns mClosetTempleDoorLogicArray (argument ignored) |
| `closetTempleThetanMeterLogic:` | 0x100071484 | 2 | returns mClosetTempleThetanMeterLogic (argument ignored) |
| `cloudLogicArray:` | 0x100071604 | 1 | returns mCloudLogicArray (argument ignored) |
| `coconutLogicArray:` | 0x100071584 | 1 | returns mCoconutLogicArray (argument ignored) |
| `crackHoleLogic:` | 0x1000714d4 | 1 | returns mCrackHoleLogic (argument ignored) |
| `danceBalloonLogic:` | 0x100071444 | 1 | returns mDanceBalloonLogic (argument ignored) |
| `danceDanceExecutionLogic:` | 0x100071454 | 1 | returns mDanceDanceExecutionLogic (argument ignored) |
| `danceTempleDoorLogicArray:` | 0x100071304 | 1 | returns mDanceTempleDoorLogicArray (argument ignored) |
| `doodleCloudLogicArray:` | 0x100071614 | 1 | returns mDoodleCloudLogicArray (argument ignored) |
| `draggedPygmyLogic:` | 0x1000715f4 | 1 | returns mDraggedPygmyLogic (argument ignored) |
| `fishingBobLogic:` | 0x100071184 | 2 | returns mFishingBobLogic (argument ignored) |
| `fishingPoleLogic:` | 0x100071194 | 5 | returns mFishingPoleLogic (argument ignored) |
| `fishLogic:` | 0x100071164 | 2 | returns mFishLogic (argument ignored) |
| `glacierLogic:` | 0x1000711f4 | 1 | returns mGlacierLogic (argument ignored) |
| `godIdolLogicArray` | 0x1000713a4 | 1 | returns mGodIdolLogicArray (argument ignored) |
| `goGoCageLogic:` | 0x100071404 | 1 | returns mGoGoCageLogic (argument ignored) |
| `healthBarLogic:` | 0x100071554 | 3 | returns mHealthBarLogic (argument ignored) |
| `iceHoleLogic:` | 0x100071224 | 2 | returns mIceHoleLogic (argument ignored) |
| `iceMonsterLogic:` | 0x1000711e4 | 1 | returns mIceMonsterLogic (argument ignored) |
| `idolPedestalLogicArray:` | 0x1000712b4 | 2 | returns mIdolPedestalLogicArray (argument ignored) |
| `iglooLogic:` | 0x100071204 | 2 | returns mIglooLogic (argument ignored) |
| `jellyfishLogicArray:` | 0x1000712a4 | 2 | returns mJellyfishLogicArray (argument ignored) |
| `kongStatueLogic:` | 0x1000714f4 | 1 | returns mKongStatueLogic (argument ignored) |
| `locustDisplayMarker` | 0x10015ad2c G | 2 | temple DragDisplayMarker (<Temple>DragDisplayMarker) or DragDisplayMarker |
| `locustLogic:` | 0x1000710f4 | 2 | returns mLocustLogic (argument ignored) |
| `locustQueenLogic:` | 0x1000713f4 | 1 | returns mLocustQueenLogic (argument ignored) |
| `locustTempleDoorLogicArray:` | 0x1000712f4 | 1 | returns mLocustTempleDoorLogicArray (argument ignored) |
| `mapPaperLogic` | 0x100078fc8 | 2 | returns mMapPaperLogic (argument ignored) |
| `monkeyLogicArray:` | 0x100071294 | 3 | returns mMonkeyLogicArray (argument ignored) |
| `moronSnakeLogic:` | 0x100071374 | 1 | returns mMoronSnakeLogic (argument ignored) |
| `noPeeLogic:` | 0x100071384 | 1 | returns mNoPeeLogic (argument ignored) |
| `outHouseLogic:` | 0x1000711d4 | 1 | returns mOutHouseLogic (argument ignored) |
| `piranhaLogicArray:` | 0x1000715c4 | 1 | returns mPiranhaLogicArray (argument ignored) |
| `plagueTempleDoorLogicArray:` | 0x100071314 | 1 | returns mPlagueTempleDoorLogicArray (argument ignored) |
| `purificationTempleDoorLogicArray:` | 0x100071324 | 1 | returns mPurificationTempleDoorLogicArray (argument ignored) |
| `pygmyCloudBehindDisplayMarker` | 0x10015b7e8 G | 1 | <Temple>FlickedDisplayMarker in temples, else CloudBehindDisplayMarker |
| `pygmyDisplayMarker` | 0x10015b0ac G | 2 | <Temple>PygmyDisplayMarker in temples, else PygmyDisplayMarker |
| `pygmyDragDisplayMarker` | 0x10015aeec G | 3 | <Temple>DragDisplayMarker in temples, else DragDisplayMarker |
| `pygmyIslandBackDisplayMarker` | 0x10015b628 G | 1 | <Temple>FlickedDisplayMarker in temples, else IslandBackDisplayMarker |
| `pygmyIslandDisplayMarker` | 0x10015b26c G | 1 | <Temple>FlickedDisplayMarker in temples, else IslandDisplayMarker |
| `pygmyLogicArray:` | 0x1000715d4 | 42 | returns mPygmyLogicArray (argument ignored) |
| `pygmySmallObjectsDisplayMarker` | 0x10015b42c G | 1 | <Temple>PygmyDisplayMarker in temples, else SmallObjectsMarker |
| `pygmyUnderwaterPygmyDisplayMarker` | 0x10015b5ec G | 1 | UnderwaterPygmyDisplayMarker |
| `rainbowLogic:` | 0x1000710e4 | 1 | returns mRainbowLogic (argument ignored) |
| `sharkLogic:` | 0x1000711b4 | 1 | returns mSharkLogic (argument ignored) |
| `spearLogicArray:` | 0x1000715a4 | 2 | returns mSpearLogicArray (argument ignored) |
| `spearStatueLogic:` | 0x100071504 | 1 | returns mSpearStatueLogic (argument ignored) |
| `spiderLogic:` | 0x100071144 | 3 | returns mSpiderLogic (argument ignored) |
| `spiderTreeLogic:` | 0x100071154 | 1 | returns mSpiderTreeLogic (argument ignored) |
| `spriteLogicArray:` | 0x100071624 | 2 | new autoreleased array of what a hurricane or whirlpool can pick up: bat (if any), anchor, shark, bird, boulder, T-Rex egg, T-Rex, spider, outhouse, ant hill, bonfire, magnifying glass, fish, sword fish, fishing pole, banana, moron snake, then all pygmies, coconuts, wood and spears |
| `starsLogic:` | 0x100071104 | 1 | returns mStarsLogic (argument ignored) |
| `swordFishLogic:` | 0x100071174 | 2 | returns mSwordFishLogic (argument ignored) |
| `templeDoorLogic:` | 0x100071394 | 2 | returns mTempleDoorLogic (argument ignored) |
| `tileDrainLogic:` | 0x100071524 | 3 | returns mTileDrainLogic (argument ignored) |
| `tilePlugLogic:` | 0x100071534 | 2 | returns mTilePlugLogic (argument ignored) |
| `timeTempleDoorLogicArray:` | 0x1000712e4 | 1 | returns mTimeTempleDoorLogicArray (argument ignored) |
| `tomPygmyLogic:` | 0x10015b9c4 G | 1 | returns mTomPygmyLogic (argument ignored) |
| `tRexEggLogic:` | 0x100071114 | 1 | returns mTRexEggLogic (argument ignored) |
| `tRexLogic:` | 0x100071124 | 3 | returns mTRexLogic (argument ignored) |
| `underwaterSceneryLogicArray:` | 0x1000715b4 | 1 | returns mUnderwaterSceneryLogicArray (argument ignored) |
| `waterWingsLogic:` | 0x1000711a4 | 1 | returns mWaterWingsLogic (argument ignored) |
| `woodLogicArray:` | 0x100071594 | 1 | returns mWoodLogicArray (argument ignored) |
| `xenuLogic:` | 0x1000714c4 | 1 | returns mXenuLogic (argument ignored) |
| `xenuTempleDoorLogicArray:` | 0x100071344 | 1 | returns mXenuTempleDoorLogicArray (argument ignored) |
| `xenuTempleLogic:` | 0x1000714b4 | 1 | returns mXenuTempleLogic (argument ignored) |
| `zombieGraveLogic:` | 0x100071234 | 1 | returns mZombieGraveLogic (argument ignored) |

#### 6.2.4 `touch_input` (5)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `preStartTouchBeganXPos:yPos:` | 0x10006ea9c | 1 | empty (touches before touch is enabled) |
| `touchBegan:allTouches:` | 0x10006eaa0 | 1 | touch router (section 5) |
| `touchCancelled:allTouches:` | 0x100070a00 | 1 | same as touchEnded:allTouches: |
| `touchEnded:allTouches:` | 0x1000701a0 | 1 | touch router (section 5) |
| `touchMoved:allTouches:` | 0x10006f7fc | 1 | touch router (section 5) |

#### 6.2.5 `menus_and_screens` (6)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `areNameLabelsEnabled` | 0x100087034 | 1 | option DisplayNames == 1 |
| `isDialogActive` | 0x100078430 | 1 | challenge dialog or idol award screen open |
| `isScreenOpen` | 0x1000870c4 | 3 | map, stats, whats-new, challenge, skin pack, bonus pack, story, news, idol, help or debug screen open |
| `onButtonPress:touch:` | 0x100079090 | 1 | dispatch on the pressed BCButtonLogic (section 4.7) |
| `onButtonRelease:touch:` | 0x10007bc28 | 1 | empty |
| `onButtonReleaseOutside:touch:` | 0x10007bc2c | 1 | empty |

#### 6.2.6 `island_locations` (20)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `hasMisfitIslandBeenFound` | 0x1000881d8 | 1 | System option MisfitIslandFound != 0 |
| `isDoodleCloudEnabled` | 0x100087868 | 1 | loc not in 11..17 (temples) |
| `isDreamBubbleEnabled` | 0x100087814 | 1 | loc != 6 (Misfit) |
| `isInsideApocalypseTemple` | 0x1000871bc | 1 | loc in 11..17 |
| `islandCenterXPos` | 0x100088234 | 10 | centre x of the current location |
| `islandLocationState` | 0x100077d18 | 8 | current location (0..17) |
| `islandMaxXPos` | 0x100088224 | 27 | walkable max x |
| `islandMinXPos` | 0x100088214 | 27 | walkable min x |
| `isOceanEnabled` | 0x1000881a4 | 8 | loc not in {5, 11..17} |
| `lastIslandLocationState` | 0x100077d28 | 1 | previous location |
| `onApocalypseRoomFaderChangeIslandLocationState:` | 0x100169d90 G | 1 | travel to a temple room: setIslandLocationState, pygmies dropIntoApocalypseTempleRoom, thetans dropped in during apocalypse |
| `onDreamBubbleChangeIslandLocationState:` | 0x100169ef4 G | 1 | System MisfitIslandFound = 1 and travel (dream bubble leads to Misfit island) |
| `onIdolAwardScreenChangeIslandLocationState:` | 0x100078d64 | 1 | travel |
| `onLoadLocationTextureAtlas` | 0x10007d3ac | 8 | loadLocationTextureAtlas (managers call it when they close) |
| `onMapScreenChangeIslandLocationState:` | 0x100078fe8 | 1 | travel |
| `onMapScreenClosed` | 0x100078fd8 | 1 | mbMapScreenOpen = NO |
| `onTsunamiActivateComplete:` | 0x100164f88 G | 1 | transition over; pygmies startUnderwater |
| `onTsunamiDeactivateComplete:` | 0x1001650a8 G | 1 | transition over; underwater group removed |
| `onTsunamiHideIslandAssets:` | 0x100169f58 G | 1 | deactivateLastIslandLocationState + activateIslandLocationState:NO |
| `onTsunamiShowIslandAssets:` | 0x100169f98 G | 1 | deactivateLastIslandLocationState + activateIslandLocationState:NO |

#### 6.2.7 `island_features` (60)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `areGhostsEnabled` | 0x100087764 | 1 | loc == 3 && Ghost == 1 |
| `areWaterWingsEnabled` | 0x1000874a4 | 1 | Sand == 4 on 0; Graveyard == 1 on 3; Ice == 2 on 4; Misfit == 2 on 6; God == 1 on 7; Apocalypse == 1 on 10; else NO |
| `isAntHillEnabled` | 0x100087474 | 2 | loc == 0 && Sand == 3 |
| `isApeAltarEnabled` | 0x100087a20 | 1 | loc == 5 && ApeIslandState == 1 |
| `isApeBongoEnabled` | 0x100087a54 | 1 | loc == 5 && ApeIslandState == 1 |
| `isApeEnabled` | 0x1000879ec | 1 | loc == 5 && ApeIslandApeState == 1 |
| `isApeWenchEnabled` | 0x100087a88 | 1 | loc == 5 && ApeIslandState == 1 |
| `isApocalypseGemPedestalEnabled` | 0x100087c08 | 1 | loc == 10 |
| `isApocalypsePlagueTempleFountainEnabled` | 0x100087d58 | 1 | loc == 14 |
| `isApocalypsePortalTempleDoorEnabled` | 0x100087c38 | 1 | loc == 10 && [calendar isOpen] |
| `isApocalypsePurificationTempleFountainEnabled` | 0x100087d88 | 1 | loc == 15 |
| `isApocalypseTempleDoorEnabled` | 0x100087c20 | 1 | loc == 10 |
| `isApocalypseTimeTemplePlatformEnabled` | 0x100087cb0 | 1 | loc == 11 |
| `isBananaTreeEnabled` | 0x100087ad4 | 1 | loc == 5 && ApeIslandState == 2 |
| `isBirdEnabled` | 0x100087308 | 1 | BirdState == 1 && loc not in {2, 9} |
| `isBonFireEnabled` | 0x1000875f8 | 5 | loc == 1 && RockIslandFeature == 1 |
| `isBoulderEnabled` | 0x1000873b8 | 1 | loc == 0 && SandIslandHazard == 1 |
| `isClosetTempleClosetDoorEnabled` | 0x100087e00 | 1 | loc == 16 |
| `isClosetTempleCouchEnabled` | 0x100087de8 | 1 | loc == 16 |
| `isClosetTempleDoorEnabled` | 0x100087db8 | 1 | loc == 16 |
| `isClosetTempleThetanMeterEnabled` | 0x100087dd0 | 1 | loc == 16 |
| `isCoconutEnabled` | 0x1000873e8 | 1 | loc == 0 |
| `isDanceDanceExecutionEnabled` | 0x100087d10 | 1 | loc == 13 |
| `isDanceTempleDoorEnabled` | 0x100087d28 | 1 | loc == 13 |
| `isFishEnabled` | 0x100087400 | 2 | (loc 0 && Sand == 2) \|\| (loc 1 && RockIslandHazard == 1) \|\| (loc 4 && IceIslandState == 1) |
| `isGlacierEnabled` | 0x100087924 | 1 | loc == 4 |
| `isGodIdolEnabled` | 0x100087bd4 | 1 | loc == 7 && GodIslandIdolsState == 1 |
| `isGoGoCageEnabled` | 0x100087cf8 | 1 | loc == 13 |
| `isGravityEnabled` | 0x1000871d8 | 24 | GravityHazard == 1 && loc not in {2, 9} && no Ooga Jump && !TheRuns && !KonkeyDong && !MoronPests && !NoPee && !DDE && !isScreenOpen |
| `isIceHoleEnabled` | 0x10008793c | 2 | loc == 4 && IceHole == 1 |
| `isIceMonsterEnabled` | 0x1000878f0 | 1 | loc == 4 && IceMonster == 1 |
| `isIceSquidEnabled` | 0x100087970 | 1 | loc == 4 && IceHole == 1 && IceSquid == 1 |
| `isIdolPedestalEnabled` | 0x100087ba4 | 2 | loc == 7 |
| `isIglooEnabled` | 0x1000879b8 | 2 | loc == 4 && Igloo == 1 |
| `isJellyfishEnabled` | 0x100087b70 | 1 | loc == 9 && UnderwaterCoralReefState == 1 |
| `isKongStatueEnabled` | 0x1000880f8 | 1 | loc == 5 && ApeIslandState == 2 |
| `isKonkeyDongEnabled` | 0x10008812c | 1 | option KonkeyDongState == 1 |
| `isLaserSharkEnabled` | 0x100087884 | 1 | loc == 2 && UnderwaterHazard == 2 && [[chumBucket laserSharkLogic] isEnabled] |
| `isLocustQueenEnabled` | 0x100087cc8 | 1 | loc == 12 |
| `isLocustTempleDoorEnabled` | 0x100087ce0 | 1 | loc == 12 |
| `isMonkeyEnabled` | 0x100087b08 | 1 | loc == 5 && ApeIslandMonkeyState == 1 |
| `isMoronPestsEnabled` | 0x100088168 | 1 | option MoronPestsState == 1 |
| `isMoronSnakeEnabled` | 0x100087b3c | 1 | loc == 6 && MisfitIslandState == 1 |
| `isOutHouseEnabled` | 0x1000876fc | 1 | loc == 1 && RockIslandOutHouse == 1 |
| `isPlagueTempleDoorEnabled` | 0x100087d70 | 1 | loc == 14 |
| `isPurificationTempleDoorEnabled` | 0x100087da0 | 1 | loc == 15 |
| `isSharkEnabled` | 0x1000872b8 | 1 | SharkState == 1 && loc not in {2, 5, 9} |
| `isSpearEnabled` | 0x10008762c | 3 | (loc 3 && GraveyardSpear == 1 && SpiderWeb == 1) \|\| (loc 1 && RockSpear == 1 && RockIslandFeature == 2) |
| `isSpiderEnabled` | 0x100087798 | 2 | loc == 3 && SpiderWeb == 1 && Spider == 1 |
| `isSpiderTreeEnabled` | 0x1000877e0 | 1 | loc == 3 && SpiderWeb == 1 |
| `isTempleDoorEnabled` | 0x100087bbc | 2 | loc == 7 |
| `isTimeTempleDoorEnabled` | 0x100087c80 | 1 | loc == 11 |
| `isTRexEggEnabled` | 0x100087590 | 1 | loc == 1 && RockIslandFeature == 2 |
| `isTRexEnabled` | 0x1000876b4 | 2 | loc == 1 && RockIslandTRex == 1 && RockIslandFeature == 2 |
| `isVolcanoEnabled` | 0x1000870ac | 1 | loc == 0 (Sand) |
| `isWoodEnabled` | 0x1000875c4 | 2 | loc == 1 && RockIslandFeature == 1 |
| `isXenuEnabled` | 0x1000880b0 | 1 | loc == 17 && [xenu isAlive] |
| `isXenuTempleDoorEnabled` | 0x100088080 | 1 | loc == 17 |
| `isXenuTempleEnabled` | 0x100088098 | 1 | loc == 17 |
| `isZombieGraveEnabled` | 0x100087730 | 1 | loc == 3 && ZombieGrave == 1 |

#### 6.2.8 `sky_weather` (9)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `isDayActive` | 0x100163f8c G | 1 | !night && !sunset |
| `isEclipseActive` | 0x100163f6c G | 1 | eclipse gradient target > 0.5 |
| `isNightActive` | 0x100164008 G | 2 | mbNightActive |
| `isStormActive` | 0x100163fec G | 2 | weather == 1 |
| `isStormCloudEnabled` | 0x10008782c | 2 | option WeatherHazard == 1 |
| `isSunsetActive` | 0x10016401c G | 2 | mbSunsetActive |
| `onBatDie:` | 0x100169b24 G | 1 | removes and releases the bat |
| `onCloudDie:` | 0x10015f2ac G | 1 | removes the cloud from the scene and mCloudLogicArray |
| `sunState` | 0x100163e88 G | 1 | 0 at night, else mSunState |

#### 6.2.9 `god_powers` (21)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `canWarlockUseLightning:` | 0x100087184 | 1 | storm weather and not inside a temple (loc not in 11..17) |
| `drawLocust:` | 0x100161ed8 G | 1 | appends the point when > 5 from the last, forwards drawLocust: to LocustLogic |
| `hitPygmyWithWarlockLightning:` | 0x100161818 G | 1 | builds a 7-point jagged bolt from y=300 down to the pygmy (x jitter +-50 on points 1-5) and fires a LightningLogic (skipped while the player draws lightning) |
| `isDanceActive` | 0x100163f44 G | 2 | mbCelebrationActive |
| `isEarthquakeActive` | 0x100164030 G | 1 | mEarthquakeSequence != nil |
| `isEruptionActive` | 0x100163f58 G | 1 | mbEruption |
| `isLocustActive` | 0x100163f24 G | 1 | [locustLogic isLocustActive] |
| `onEruptionComplete:` | 0x10015f0ec G | 1 | volcano hit count reset, eruption over |
| `onFireworkDie:` | 0x100164f10 G | 1 | removes the firework |
| `onHurricaneCantPickUpObjects:` | 0x10015f18c G | 1 | stopGame 1 |
| `onHurricaneDie:` | 0x10015f1b0 G | 1 | removes and releases the hurricane |
| `onLightningDie:` | 0x10015f110 G | 1 | removes and releases the bolt |
| `onPygmyEarthquakeStart:` | 0x100164c10 G | 1 | first quaker: episode event Earthquake, stop dance music, coconut falls, new EarthquakeStart sequence |
| `onPygmyEarthquakeStop:` | 0x100164d6c G | 1 | last quaker: EarthquakeStop |
| `onPygmyHitByLightningInHurricane:` | 0x100164668 G | 1 | challenge game 1 +1 |
| `onUnderwaterTempleModeChanged:` | 0x100163394 G | 1 | retints the power trail (mode 0 ends the stroke) |
| `onVolcanoHit:` | 0x100164e00 G | 1 | volcano hit1/2/3 (erupt1/2/3 for timeTemplePygmyType 4); 3rd hit starts the eruption |
| `onWhirlPoolCantPickUpObjects:` | 0x10015f22c G | 1 | empty |
| `onWhirlPoolDie:` | 0x10015f230 G | 1 | removes and releases the whirlpool |
| `startDrawLocust:` | 0x100161fac G | 1 | starts a locust path, LocustLogic startDrawLocust:startPos:, LocustStartDraw |
| `stopDrawLocust:` | 0x1001620f0 G | 1 | final point, drawLocustComplete; releasing outside (10..470, 10..310) deactivates the swarm |

#### 6.2.10 `pygmies` (18)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `addPygmy` | 0x10015bfa4 G | 1 | adds a pygmy if live count < 6 (+1 with Charlie, +1 while Tom lives): createPygmyLogic + [tribe resurrectPygmyLogic:] |
| `canTouchPygmy` | 0x100163c34 G | 1 | NO while the help, debug, challenge or stats screen is open, an eruption runs, one of the 22 blocking drag refs (see on…Drag:) or mDirectedAntHillLogic is set, or mEarthquakeSequence exists |
| `createPygmyLogic` | 0x10015ba88 G | 1 | creates, places and registers a new PygmyLogic and returns it (section 4.14) |
| `createTomPygmy` | 0x10015b9d8 G | 1 | activates the Tom guest persona, creates + resurrects a pygmy for it, deactivates the guest; stores mTomPygmyLogic |
| `isBabyPygmyActive` | 0x100087f60 | 1 | any pygmy with timeTemplePygmyType == 2 |
| `isDraggingPygmy` | 0x100163bfc G | 1 | a pygmy or ghost pygmy is being dragged |
| `isTomPygmyActive` | 0x100087e18 | 2 | a pygmy with personaPersonality 2 that is not trappedInCloset |
| `isTomPygmyAlive` | 0x10015b9a8 G | 2 | mTomPygmyLogic != nil |
| `onPygmyDie:sacrificeId:` | 0x10016485c G | 1 | records the sacrifice, live count -1, removes the pygmy everywhere, ends a minigame it was in |
| `onPygmyDrag:` | 0x1001645d0 G | 1 | mDraggedPygmyLogic = sender (drag auto-scroll in onGameFrame:, section 3.2; isDraggingPygmy) |
| `onPygmyDragComplete:` | 0x1001645e4 G | 1 | mDraggedPygmyLogic = nil, then checkCountSharkSnaps (ends a running shark-snaps game, 4.11) |
| `onPygmyExplode:` | 0x100164714 G | 1 | every pygmy throwFromExplosion: at the exploding pygmy position |
| `onPygmyGhostDrag:` | 0x1001645a8 G | 1 | mDraggedGhostPygmyLogic = sender (isDraggingPygmy) |
| `onPygmyGhostDragComplete:` | 0x1001645bc G | 1 | mDraggedGhostPygmyLogic = nil |
| `onPygmySleepingStart:` | 0x100164474 G | 1 | first sleeper -> AllPygmiesSleepingStart |
| `onPygmySleepingStop:` | 0x100164514 G | 1 | last sleeper -> AllPygmiesSleepingStop |
| `onPygmyTumble:` | 0x100164604 G | 1 | moves the pygmy to index 0 of mPygmyLogicArray |
| `onPygmyTumbleComplete:` | 0x100164664 G | 1 | empty |

#### 6.2.11 `world_object_events` (89)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `isSpiderActive` | 0x10016409c G | 2 | loc == 3 && [spider isActive] |
| `isTRexActive` | 0x10016404c G | 2 | loc == 1 && [tRex isActive] |
| `onAnchorDrag:` | 0x1001695c0 G | 1 | mDraggedAnchorLogic = sender (blocks canTouchPygmy) |
| `onAnchorDragComplete:` | 0x1001695d4 G | 1 | mDraggedAnchorLogic = nil |
| `onAntHillDirect:` | 0x10016998c G | 1 | mDirectedAntHillLogic = sender |
| `onAntHillDirectComplete:` | 0x1001699a0 G | 1 | mDirectedAntHillLogic = nil |
| `onApeVisible:` | 0x100169868 G | 1 | refreshSkinPackOption ZombiePack / Ape / Ape |
| `onApocalypseGemDrag:` | 0x100165404 G | 1 | mDraggedApocalypseGemLogic = sender (drag auto-scroll in onGameFrame:, section 3.2) |
| `onApocalypseGemDragComplete:` | 0x100165418 G | 1 | mDraggedApocalypseGemLogic = nil |
| `onBaitMasterStart:` | 0x100169270 G | 1 | mbBaitMaster = YES, every pygmy startBaitMaster |
| `onBaitMasterStop:` | 0x100169394 G | 1 | mbBaitMaster = NO, stopGame 5, every pygmy stopBaitMaster |
| `onBananaDrag:` | 0x100165314 G | 1 | mDraggedBananaLogic = sender (blocks canTouchPygmy) |
| `onBananaDragComplete:` | 0x100165328 G | 1 | mDraggedBananaLogic = nil |
| `onBirdDrag:` | 0x100169afc G | 1 | mDraggedBirdLogic = sender (blocks canTouchPygmy) |
| `onBirdDragComplete:` | 0x100169b10 G | 1 | mDraggedBirdLogic = nil |
| `onBirdFlyOffScreen:` | 0x100169aec G | 1 | stopCountingBirdBombs |
| `onBirdPooHitPygmy:` | 0x100169ab8 G | 1 | challenge game 2 +1 |
| `onBirdPooMissPygmy:` | 0x100169adc G | 1 | stopCountingBirdBombs |
| `onBoulderDrag:` | 0x100169610 G | 1 | mDraggedBoulderLogic = sender (blocks canTouchPygmy) |
| `onBoulderDragComplete:` | 0x100169624 G | 1 | mDraggedBoulderLogic = nil |
| `onClosetTempleClosetDoorDrag:` | 0x100169ba0 G | 1 | mDraggedClosetTempleClosetDoorLogic = sender (blocks canTouchPygmy) |
| `onClosetTempleClosetDoorDragComplete:` | 0x100169bb4 G | 1 | mDraggedClosetTempleClosetDoorLogic = nil |
| `onCoconutDie:` | 0x100165a80 G | 1 | checkCountCoconutBounces; extra coconut removed, the last one regrows on the tree at (163, 205) |
| `onCoconutDrag:` | 0x1001650d8 G | 1 | mDraggedCoconutLogic = sender (blocks canTouchPygmy) |
| `onCoconutDragComplete:` | 0x1001650ec G | 1 | mDraggedCoconutLogic = nil |
| `onCoconutHitGround:` | 0x100165a70 G | 1 | empty |
| `onCoconutHitPygmy:` | 0x1001657a4 G | 1 | challenge game 0 +1; an unsplit coconut splits into two |
| `onDanceBalloonDrag:` | 0x100165364 G | 1 | mDraggedDanceBalloonLogic = sender (blocks canTouchPygmy) |
| `onDanceBalloonDragComplete:` | 0x100165378 G | 1 | mDraggedDanceBalloonLogic = nil |
| `onFishDrag:` | 0x1001652c4 G | 1 | mDraggedFishLogic = sender (blocks canTouchPygmy) |
| `onFishDragComplete:` | 0x1001652d8 G | 1 | mDraggedFishLogic = nil |
| `onFishingBobDrag:` | 0x10016529c G | 1 | mDraggedFishingBobLogic = sender (never read elsewhere) |
| `onFishingBobDragComplete:` | 0x1001652b0 G | 1 | mDraggedFishingBobLogic = nil |
| `onFishingPoleCreateBob:fishingBobLogic:` | 0x100165240 G | 1 | becomes the bob's listener, remembers it |
| `onFishingPoleDestroyBob:fishingBobLogic:` | 0x100165288 G | 1 | forgets the bob |
| `onFishingPoleDrag:` | 0x100165218 G | 1 | mDraggedFishingPoleLogic = sender (blocks canTouchPygmy) |
| `onFishingPoleDragComplete:` | 0x10016522c G | 1 | mDraggedFishingPoleLogic = nil |
| `onGodIdolDrag:` | 0x1001653dc G | 1 | mDraggedGodIdolLogic = sender (blocks canTouchPygmy) |
| `onGodIdolDragComplete:` | 0x1001653f0 G | 1 | mDraggedGodIdolLogic = nil |
| `onIceMonsterCancelBattle:` | 0x100169968 G | 1 | forfeitGame 9 |
| `onIceMonsterHitByPygmy:` | 0x1001698d8 G | 1 | empty |
| `onIceMonsterKilled:` | 0x1001698dc G | 1 | stopGame 9 |
| `onIceMonsterStartBattle:` | 0x100169900 G | 1 | setGameScore 9 = 0 unless game 9 already runs |
| `onIceMonsterVisible:` | 0x1001698a0 G | 1 | refreshSkinPackOption IceMonster / Ice Monster / Ice Monster |
| `onJellyfishDrag:` | 0x1001653b4 G | 1 | mDraggedJellyfishLogic = sender (blocks canTouchPygmy) |
| `onJellyfishDragComplete:` | 0x1001653c8 G | 1 | mDraggedJellyfishLogic = nil |
| `onLaserSharkZapComplete:` | 0x10016959c G | 1 | stopGame 6 |
| `onLaserSharkZapPygmy:` | 0x100169578 G | 1 | challenge game 6 +1 |
| `onLocustQueenVisible:` | 0x100169830 G | 1 | refreshSkinPackOption ZombiePack / Locust Queen / Locust Queen |
| `onMagnifyingGlassDrag:` | 0x1001699b4 G | 1 | mDraggedMagnifyingGlassLogic = sender (blocks canTouchPygmy) |
| `onMagnifyingGlassDragComplete:` | 0x1001699c8 G | 1 | mDraggedMagnifyingGlassLogic = nil |
| `onMonkeyDrag:` | 0x10016533c G | 1 | mDraggedMonkeyLogic = sender (blocks canTouchPygmy) |
| `onMonkeyDragComplete:` | 0x100165350 G | 1 | mDraggedMonkeyLogic = nil |
| `onMoronSnakeDrag:` | 0x10016538c G | 1 | mDraggedMoronSnakeLogic = sender (blocks canTouchPygmy) |
| `onMoronSnakeDragComplete:` | 0x1001653a0 G | 1 | mDraggedMoronSnakeLogic = nil |
| `onNewPiranhaLogic:` | 0x1001694dc G | 1 | adds the piranha to mPiranhaLogicArray |
| `onPiranhaEatPygmy:` | 0x10016953c G | 1 | challenge game 5 +1 while Bait Master runs |
| `onPygmiesBowled:` | 0x1001646d0 G | 2 | ChallengeManager checkAchievement "Bowled" (personal best of bowled pygmies) |
| `onPygmiesSkewered:` | 0x10016468c G | 1 | ChallengeManager checkAchievement "Skewered" |
| `onRemoveAllPiranhaLogics:` | 0x10016951c G | 1 | clears mPiranhaLogicArray |
| `onRemovePiranhaLogic:` | 0x1001694fc G | 1 | removes the piranha from mPiranhaLogicArray |
| `onSharkDrag:` | 0x100169a18 G | 1 | mDraggedSharkLogic = sender (blocks canTouchPygmy) |
| `onSharkDragComplete:` | 0x100169a2c G | 1 | mDraggedSharkLogic = nil |
| `onSharkSnap:` | 0x100169a40 G | 1 | checkCountSharkSnaps, challenge game 3 +1 |
| `onSnowBallHitPygmy:` | 0x100169ab0 G | 1 | empty |
| `onSnowBallMissPygmy:` | 0x100169ab4 G | 1 | empty |
| `onSpearDrag:` | 0x100165128 G | 1 | mDraggedSpearLogic = sender (blocks canTouchPygmy) |
| `onSpearDragComplete:` | 0x10016513c G | 1 | mDraggedSpearLogic = nil |
| `onSpearThrow:` | 0x100165150 G | 1 | challenge game 7 or 8 +1 (whichever runs) |
| `onSpiderCancelHunt:` | 0x10016980c G | 1 | forfeitGame 8 |
| `onSpiderHitBySpear:` | 0x100169780 G | 1 | setGameScore 8 = 1 unless running |
| `onSpiderKilledBySpear:` | 0x1001697e8 G | 1 | stopGame 8 |
| `onSpiderVisible:` | 0x10016974c G | 1 | refreshSkinPackOption Spider / Spider / Spider |
| `onSwordFishDrag:` | 0x1001652ec G | 1 | mDraggedSwordFishLogic = sender (blocks canTouchPygmy) |
| `onSwordFishDragComplete:` | 0x100165300 G | 1 | mDraggedSwordFishLogic = nil |
| `onThetanDrag:` | 0x100169bc8 G | 1 | mDraggedThetanLogic = sender (blocks canTouchPygmy) |
| `onThetanDragComplete:` | 0x100169bdc G | 1 | mDraggedThetanLogic = nil |
| `onTilePlugDrag:` | 0x1001695e8 G | 1 | mDraggedTilePlugLogic = sender (blocks canTouchPygmy) |
| `onTilePlugDragComplete:` | 0x1001695fc G | 1 | mDraggedTilePlugLogic = nil |
| `onTRexCancelHunt:` | 0x100169728 G | 1 | forfeitGame 7 |
| `onTRexEggDrag:` | 0x100169638 G | 1 | mDraggedTRexEggLogic = sender (blocks canTouchPygmy) |
| `onTRexEggDragComplete:` | 0x10016964c G | 1 | mDraggedTRexEggLogic = nil |
| `onTRexHitBySpear:` | 0x10016969c G | 1 | setGameScore 7 = 1 unless running |
| `onTRexKilledBySpear:` | 0x100169704 G | 1 | stopGame 7 |
| `onTRexVisible:` | 0x100169660 G | 1 | refreshSkinPackOption TRex / Tyrannosaurus Rex / TRex Body |
| `onWaterWingsDrag:` | 0x1001651f0 G | 1 | mDraggedWaterWingsLogic = sender (blocks canTouchPygmy) |
| `onWaterWingsDragComplete:` | 0x100165204 G | 1 | mDraggedWaterWingsLogic = nil |
| `onWoodDrag:` | 0x100165100 G | 1 | mDraggedWoodLogic = sender (blocks canTouchPygmy) |
| `onWoodDragComplete:` | 0x100165114 G | 1 | mDraggedWoodLogic = nil |

#### 6.2.12 `minigames` (36)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `isDoodleJumpingPygmy` | 0x100163be0 G | 1 | an Ooga Jump is running |
| `onBattleOfTheGodsStart:pygmyLogic:` | 0x100167b94 G | 1 | cancel touches, close screens, hide UI, mbBattleOfTheGods, remember pygmy, StartBattleOfTheGods |
| `onBattleOfTheGodsStop:` | 0x100167c64 G | 1 | hide arena, forget pygmy, show UI, StopBattleOfTheGods |
| `onDanceDanceExecutionScore:` | 0x100167b68 G | 1 | setGameScore 14 |
| `onDanceDanceExecutionStart:` | 0x100167aa4 G | 2 | cancel touches, close screens, hide UI, mbDanceDanceExecution = YES |
| `onDanceDanceExecutionStop:` | 0x100167b0c G | 1 | mbDanceDanceExecution = NO, show UI, stopGame 14 |
| `onKonkeyDongMoveCameraToNextLevel:` | 0x100166f94 G | 1 | camera eases to level y |
| `onKonkeyDongScore:` | 0x100166ec8 G | 2 | incrementGameScore 12 |
| `onKonkeyDongStart:pygmyLogic:` | 0x100166ca4 G | 1 | start Konkey Dong (section 4.16) |
| `onKonkeyDongStartMovingCamera` | 0x100166ef4 G | 1 | camera follow on, StartKonkeyDongLoop |
| `onKonkeyDongStop:` | 0x100166dac G | 2 | stop Konkey Dong |
| `onMoronPestsScore:` | 0x100167398 G | 1 | setGameScore 13 |
| `onMoronPestsStart:pygmyLogic:` | 0x1001671ac G | 1 | start Moron Pests |
| `onMoronPestsStartMovingCamera` | 0x1001673c4 G | 1 | camera scroll on, StartMoronPestsLoop |
| `onMoronPestsStop:` | 0x10016728c G | 1 | stop Moron Pests |
| `onMoronPestsStopMovingCamera` | 0x100167444 G | 1 | camera scroll off |
| `onNoPeeRetry:` | 0x100167858 G | 1 | stopGame 15 (retry) |
| `onNoPeeStart:pygmyLogic:` | 0x10016765c G | 1 | start No Pee |
| `onNoPeeStartMovingCamera` | 0x10016789c G | 1 | camera follow on |
| `onNoPeeStop:` | 0x10016773c G | 1 | stop No Pee |
| `onOpenIdolBattleEnemyScreen` | 0x100167ffc G | 1 | idol flag, cancel touches, IdolManager openIdolBattleEnemyScreen |
| `onOpenIdolBattleResultsScreen` | 0x10016808c G | 1 | idol + results flags, IdolManager openIdolBattleResultsScreen |
| `onOpenIdolBattleScreen` | 0x100168058 G | 1 | idol flag, IdolManager openIdolBattleScreen |
| `onPainDrainScore:` | 0x100166430 G | 1 | setGameScore 10 |
| `onPainDrainStart:pygmyLogic:` | 0x100166324 G | 2 | start Pain Drain |
| `onPainDrainStop:` | 0x1001663b0 G | 2 | stop Pain Drain |
| `onPygmyDoodleJumpFall:` | 0x1001640ec G | 1 | mbDoodleJumpFall = YES |
| `onPygmyFallIntoCrackHoleComplete:` | 0x10016664c G | 1 | starts The Runs when TheRunsState == 1, not underwater, no Ooga Jump |
| `onPygmyStartDoodleJump:` | 0x10016439c G | 1 | Ooga Jump begins: remember pygmy, create 50 doodle clouds |
| `onPygmyStopDoodleJump:` | 0x1001643cc G | 1 | Ooga Jump ends: hide doodle group, clear state, remove clouds, stopGame 4 |
| `onTheRunsScore:` | 0x100166908 G | 1 | setGameScore 11 |
| `onTheRunsShakeScreenStart` | 0x1001669b4 G | 1 | new TheRunsShakeBackgroundStart sequence |
| `onTheRunsShakeScreenStop` | 0x100166a48 G | 1 | TheRunsShakeBackgroundStop |
| `onTheRunsStart:pygmyLogic:` | 0x10016670c G | 1 | start The Runs |
| `onTheRunsStartMovingCamera` | 0x100166934 G | 1 | camera scroll on, StartTheRunsLoop |
| `onTheRunsStop:` | 0x1001667ec G | 1 | stop The Runs |

#### 6.2.13 `apocalypse` (7)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `endApocalypse` | 0x100168998 G | 1 | clears apocalypse flag, gems prepareToEndApocalypse, Xenu banish, Xenu temple wall rebuilt, calendar portal closed, thetans removed |
| `endApocalypseComplete` | 0x100168b44 G | 1 | apocalypse gradient target 0, ApocalypseLoop -> BackgroundLoop, episode event EndApocalypse |
| `endApocalypseCompleteNow` | 0x100168c0c G | 1 | same body as endApocalypseComplete |
| `isApocalypseActive` | 0x100168884 G | 5 | mbApocalypseActive |
| `onApocalypseGemDie:` | 0x10016542c G | 1 | removes the gem from mApocalypseGemLogicArray |
| `onApocalypseGemEventTriggered:` | 0x10016544c G | 1 | during apocalypse: event 1-7 -> episode event Spawn<Time\|Locust\|Dance\|Plague\|Purification\|Closet\|Xenu>Gem and activates gem logic #403-409 if inactive |
| `startApocalypse` | 0x100168898 G | 2 | apocalypse on: gradient, thetans, ApocalypseLoop, episode event StartApocalypse |

#### 6.2.14 `stats_and_achievements` (5)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `checkAllAchievementsAndLeaderboards` | 0x100070ae4 | 1 | re-submits all 18 achievement counters (deferred until personas exist) |
| `onAchievement:count:highScoreWins:` | 0x100070cc4 | 4 | records an achievement on the god persona (count -1 = increment), checks tiers, submits its leaderboard |
| `onPygmyGiftGiven:giftId:` | 0x100164adc G | 1 | episode event by gift, tribe addGift, pg.totalgifts leaderboard |
| `onPygmySacrificeEvent:` | 0x100070dc4 | 1 | IdolManager triggerEpisodeEventBySacrificeId: |
| `onTriggerSacrifice:sacrificeId:` | 0x100070de0 | 1 | tribe sacrifice:pygmyLogic:, pg.totalsacrifices leaderboard (per-sacrifice boards are nil) |

#### 6.2.15 `managers_bridge` (38)

| Method | Address | Protocols | Description |
|---|---|---|---|
| `isAnyGameActive` | 0x100166304 G | 1 | [challengeManager isAnyGameActive] |
| `isStoryActive` | 0x100078410 | 33 | mbStoryActive |
| `onBattleAlertSelect` | 0x100078f90 | 1 | log only |
| `onBonusPackScreenClosed` | 0x100077d38 | 1 | mbBonusPackScreenOpen = NO |
| `onBonusPackUnlockDialogClosed` | 0x100077d5c | 1 | mbBonusPackScreenOpen = NO |
| `onBonusPackUnlockDialogOpened` | 0x100077d48 | 1 | mbBonusPackScreenOpen = YES |
| `onChallengeDialogOpen` | 0x100079018 | 1 | cancel touches, challenge screen + dialog open flags |
| `onChallengeScreenClosed` | 0x100078ffc | 1 | challenge screen and dialog flags cleared |
| `onDebugScreenClosed` | 0x100078420 | 1 | mbDebugScreenOpen = NO |
| `onEndChallenge:` | 0x100079080 | 1 | mbChallengeActive = NO |
| `onEpisodeAlertOpenChallengeDetail:` | 0x100078d78 | 1 | opens the idol detail screen for the episode unless a screen, story, timed game, No Pee or dialog is active |
| `onIdolAwardScreenClosed` | 0x100078c78 | 1 | idol + award flags cleared |
| `onIdolAwardScreenOpenChallenges` | 0x100078c94 | 1 | opens the idol list screen |
| `onIdolAwardScreenOpened` | 0x100078c64 | 1 | mbIdolAwardScreenOpen = YES |
| `onIdolScreenClosed` | 0x100078b28 | 1 | flags cleared; after battle results reload TextureAtlas0 and restore the island; monster atlas; StartGameAfterIdolScreen if pending |
| `onIdolScreenLaunchStoreScreen` | 0x100078c48 | 1 | idol screen closed -> skin pack store |
| `onNewsLaunchChallengeScreen` | 0x100078ad8 | 1 | news closed -> challenge screen |
| `onNewsLaunchStoreScreen` | 0x100078ab0 | 1 | news closed -> skin pack store |
| `onNewsLaunchStoryScreen` | 0x100078b00 | 1 | news closed -> story screen |
| `onNewsScreenClosed` | 0x1000789ec | 1 | mbNewsScreenOpen = NO |
| `onNewsTeaserClosed` | 0x1000789fc | 1 | mbNewsTeaserOpen = NO |
| `onNewsTeaserLaunchNewsScreen` | 0x100078a0c | 1 | teaser -> news screen (CloseMenu, openNewsScreen) |
| `onSkinPackScreenClosed` | 0x100077d6c | 1 | flag cleared, loadMonsterTextureAtlas |
| `onStartChallenge:` | 0x100079060 | 1 | challenge screen closed, mbChallengeActive = YES |
| `onStoryAddPygmy:` | 0x100078840 | 1 | returns the pygmy labelled name, or creates one in story mode (live count < 6), else nil |
| `onStoryCancel` | 0x100078400 | 1 | onStoryComplete |
| `onStoryComplete` | 0x100078344 | 1 | story flags off, sounds resume, story pygmies released, rainbow cleared, DisplayNames restored, island logics on, UI shown |
| `onStoryEnvironmentChange:` | 0x100078460 | 1 | story environment node: location, weather, time of day (section 4.10) |
| `onStoryModeActivateMyRainbow` | 0x1000780d8 | 1 | rainbow storyModeActivateMyRainbow |
| `onStoryModeActivateRainbow` | 0x1000780bc | 1 | rainbow storyModeActivateRainbow |
| `onStoryModeDeactivateRainbow` | 0x1000780f4 | 1 | rainbow storyModeDeactivateRainbow |
| `onStoryModeSharkSnap` | 0x1000780a0 | 1 | shark storyModeSharkSnap |
| `onStoryModeStartCelebration` | 0x100077d98 | 1 | startCelebration |
| `onStoryModeStartEarthquake` | 0x100077db8 | 1 | new EarthquakeStart sequence, pygmies startStoryEarthquake |
| `onStoryModeStopCelebration` | 0x100077da8 | 1 | stopCelebration |
| `onStoryModeStopEarthquake` | 0x100077f3c | 1 | EarthquakeStop, pygmies stopStoryEarthquake |
| `onStoryScreenClosed` | 0x100077d88 | 1 | mbStoryScreenOpen = NO |
| `onStoryStart` | 0x10007821c | 1 | story flag, sleeping sounds paused, winch sound stopped, pygmies leave story mode, DisplayNames forced on, island logics off, UI hidden |

## 7. Sequence-item handlers

The controller is the delegate of its own sequences only (`mSequence`, `mMiscSequence`, the minigame,
weather, message, celebration and earthquake sequences — sequences.md §3.10); items it does not
implement fall back to `BCLibrary` (§3.7). These 47 `:::` methods are therefore reachable from the
scripts those sequences run (`PocketGod`, `Start<Location>`, `Start<Game>`, help/stats/menu scripts,
…); logic objects reach a few of them through explicit listener calls (e.g.
`DiscoBallLogic::playBackgroundLoop:::`). Behaviour is in sequences.md ("PocketGodViewController" /
"(Game)"); summary and owning unit:

| Item | Address | Unit | Behaviour |
|---|---|---|---|
| `startTime`, `stopTime` | 0x10007bc30, 0x10007bc50 | boot_and_frame | profiling (unused) |
| `waitForHoldLoadingAd` | 0x10007bca8 | boot_and_frame | done once 5 s passed since `init` (unused) |
| `waitForPreferencesLoadComplete`, `loadPreferences` | 0x10007bce8, 0x10007bdcc | boot_and_frame | 2.4, 2.5 |
| `enableTouch`, `checkStartScreen`, `hideLoadingScreen`, `checkShowAuthenticationViewController`, `checkForNewNews` | 0x100159998, 0x100159c80, 0x100159d40, 0x100159c44, 0x100159dc0 | boot_and_frame | 2.4 |
| `playBackgroundLoop` (soundId), `stopBackgroundLoop` | 0x10007104c, 0x100071078 | app_platform | `playBackgroundLoop:`, `stopBackgroundLoop` |
| `pause/resumeHurricaneSounds`, `pause/resumeSleepingSounds`, `pause/resumeLocustSounds` | 0x10007109c, 0x1000710c0, 0x10016903c, 0x100169060, 0x100169174, 0x100169198 | app_platform | 4.6 |
| `updateHelpPage` | 0x10007737c | menus_and_screens | `updateHelpPage:mHelpPageIndex` |
| `openMenu`, `closeMenu` | 0x100159e2c, 0x100159e54 | menus_and_screens | 4.7 |
| `showMessage` (zPos, xPos, texture, yPos), `removeMessage` | 0x100159ebc, 0x10015a0bc | menus_and_screens | 4.7 |
| `loadLocationTextureAtlas`, `loadTextureAtlasChannel` (channel), `refreshSkinPackOption` (skinPackId, skinOptionId, skinObjectId) | 0x10015a118, 0x10007d3bc, 0x10007d468 | island_locations | 4.13 |
| `setLogic` | 0x10015c11c | world_registry | 4.15 |
| `triggerEpisodeEvent` (event) | 0x10015a78c | managers_bridge | `[mIdolManager triggerEpisodeEvent:]` |
| `addPygmy`, `addBabyPygmy`, `addTomPygmy` | 0x10015c064, 0x10015c088, 0x10015c0f8 | pygmies | 4.14 |
| `startPygmiesDancing`, `stopPygmiesDancing`, `fireworks` (minTime, maxTime) | 0x10015a13c, 0x10015a454, 0x10015a7c8 | god_powers | 4.6 |
| `shakeBackground` (minTime, maxTime), `stopEarthquake` | 0x10015e374, 0x10015e524 | god_powers | 4.6 |
| `setCloudGradientFade` (fade), `transitionIn/OutClouds`, `transitionIn/OutStormClouds` | 0x10015e32c, 0x10015ea30, 0x10015eb94, 0x10015e684, 0x10015e6fc | sky_weather | 4.5 |
| `stopTheRunsShakeScreen`, `show{TheRuns,KonkeyDong,MoronPests,NoPee,BattleOfTheGods}DisplayGroup` | 0x10015e608, 0x1001691bc, 0x1001691e0, 0x100169204, 0x100169228, 0x10016924c | minigames | 4.16 |

## 8. Ivar catalogue

All 581 ivars of `PocketGodViewController` (`instanceSize` 20277; offsets from `tmp/re/objc_meta.txt`), grouped by the unit of section 9 that owns them (the unit whose feature the value belongs to; other units may read it — see `ivars_shared` in the JSON). "Refs" = number of the 692 methods whose decompiled code names the ivar (`init`, `loadGame`, `dealloc` and `.cxx_construct` included). Types are simplified from the ObjC encodings: `X*` = object of class X, `BOOL`/`int`/`float`/`double` = `B`/`i`/`f`/`d`, `VECTOR4` = `{?="x"f"y"f"z"f"w"f}` (engine 4-float vector: position, velocity, colour), `BCGradientDef*` = pointer to a gradient definition of the library, `std::list<VECTOR4>*`/`std::vector<VECTOR4>*` = C++ point lists, `id[1000]` = the two inline touch-logic arrays. Prefixes: `mb` = BOOL flag, `mp` = C++ pointer, `m…Logic` = a cached logic object from the generated table (logic.md §13; "#N" = table index), `m…LogicArray` = an `NSMutableArray` of logic objects registered by `setLogic:::`.

Never referenced by any method (12): `mbComicUrlOpened`, `mFpsTime`, `mTargetFps`, `mLongestFrameTime`, `mSpikeCount`, `mbSubmittingScore`, `mbSubmitSacrificesWhenComplete`, `mbSubmitGiftsWhenComplete`, `mMuteSequence`, `mStartTime`, `mpHurricaneList`, `mpWhirlPoolList`.

### 8.1 `app_platform` (15)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x140 | `mView` | `BCView*` | 6 | BCView (OpenGL view) created in init; game loop host; touch source |
| 0x169 | `mbComicUrlOpened` | `BOOL` | 0 | unused |
| 0x170 | `mLaunchUrl` | `NSString*` | 4 | NSString URL to open once the loading screen is hidden (openUrl:/openLaunchUrl) |
| 0x178 | `mbHideLoadingScreen` | `BOOL` | 3 | YES once hideLoadingScreen::: ran; openUrl: opens immediately only after this |
| 0x18c | `mbModalViewControllerActive` | `BOOL` | 2 | guard: a UIKit modal (Game Center) is being presented |
| 0x1d8 | `mTextField` | `UITextField*` | 7 | off-screen UITextField used to raise the iOS keyboard (delegate = self) |
| 0x208 | `mSaveEditText` | `NSString*` | 4 | copy of the text being edited (restored on cancel) |
| 0x210 | `mKeyboardListener` | `id<KeyboardListener>` | 3 | object that asked for the keyboard via onShowKeyboard:...; receives onKeyboardTextEntered: |
| 0x40f0 | `mMuteSequence` | `BCSequence*` | 0 | unused (never created) |
| 0x40f8 | `mDelegate` | `id<PocketGodViewDelegate>` | 5 | PocketGodViewDelegate = PocketGodAppDelegate (onHideLoadingScreen, onModalViewWillAppear/Disappear) |
| 0x4dd0 | `mbKeyboardOpen` | `BOOL` | 6 | text entry active |
| 0x4dd4 | `mMaxEditCharCount` | `int` | 3 | max characters for the field being edited |
| 0x4dd8 | `mEditingText` | `BCText*` | 7 | BCText being edited |
| 0x4de0 | `mActiveCancelEditDisplayObject` | `BCDisplayObject*` | 5 | cancel button shown while editing |
| 0x4f34 | `mbChartboostAdVisible` | `BOOL` | 6 | Chartboost interstitial showing (map screen ad blocker) |

### 8.2 `boot_and_frame` (19)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x148 | `mLibrary` | `BCLibrary*` | 2 | BCLibrary created in loadGame (defs, sequences, logic table) |
| 0x160 | `mAppStartTime` | `double` | 2 | CFAbsoluteTime at init; waitForHoldLoadingAd::: compares against it (5 s) |
| 0x168 | `mbGameStarted` | `BOOL` | 2 | set NO at end of loadGame, YES at the top of every onGameFrame: (never read otherwise) |
| 0x17c | `mFpsTime` | `float` | 0 | unused |
| 0x180 | `mTargetFps` | `float` | 0 | unused |
| 0x184 | `mLongestFrameTime` | `float` | 0 | unused |
| 0x188 | `mSpikeCount` | `int` | 0 | unused |
| 0x1e0 | `mbInitComplete` | `BOOL` | 1 | set YES in checkStartScreen::: (write-only) |
| 0x1e8 | `mPersonaPreferences` | `BCPreferences*` | 5 | BCPreferences "Persona" (legacy file Preferences.plist): god/tribe/pygmy personas |
| 0x1f0 | `mOptionPreferences` | `BCPreferences*` | 54 | BCPreferences "Options": every toggle/option (table in section 4.2) |
| 0x1f8 | `mSystemPreferences` | `BCPreferences*` | 11 | BCPreferences "System": MisfitIslandFound, WhatsNewVersion, InfoShown, BonusInfoShown, achievement flags |
| 0x200 | `mChallengePreferences` | `BCPreferences*` | 4 | BCPreferences "Challenge" (secure + iCloud): challenge/episode progress |
| 0x40e0 | `mSequence` | `BCSequence*` | 28 | main BCSequence; created with "PocketGod" (boot script); setSequenceDef: is used to run UI/story sequences |
| 0x4158 | `mMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `MenuDisplayGroup` |
| 0x4240 | `mMenuDisplayMarker` | `BCDisplayMarker*` | 3 | marker `MenuDisplayMarker` |
| 0x4674 | `mStartTime` | `float` | 0 | unused |
| 0x4ac8 | `mTargetIslandYPosVel` | `float` | 1 | island vertical scroll velocity (Ooga Jump / drag auto-scroll) |
| 0x4acc | `mTargetIslandYPos` | `float` | 1 | island vertical scroll target; island y eases 30 %/frame toward it |
| 0x4dc1 | `mbAddPygmyAfterIdolStartScreen` | `BOOL` | 3 | run StartGameAfterIdolScreen when the idol start screen closes |

### 8.3 `world_registry` (84)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x1d0 | `mMapPaperLogic` | `MapPaperLogic*` | 2 | map paper handed to MapScreen/NewsManager via mapPaperLogic; setLogic `MapPaperLogic` -> MapPaperLogic on MapDisplayGroup/MapPaper (no listener) |
| 0x4238 | `mSmallObjectDisplayMarker` | `BCDisplayMarker*` | 1 | marker `SmallObjectsMarker` |
| 0x4340 | `mLogicArray` | `NSMutableArray*` | 5 | every logic created by setLogic::: and createPygmyLogic (ownership list) |
| 0x4348 | `mMainUiLogicArray` | `NSMutableArray*` | 3 | main HUD buttons from setLogic (NewPygmy, Menu, Lava) |
| 0x4350 | `mHelpUiLogicArray` | `NSMutableArray*` | 3 | help screen buttons from setLogic |
| 0x4358 | `mWhatsNewScreenUiLogicArray` | `NSMutableArray*` | 3 | WhatsNew screen button from setLogic |
| 0x4388 | `mCoconutLogicArray` | `NSMutableArray*` | 11 | setLogic `CoconutLogic` -> CoconutLogic on IslandDisplayGroup/Coconut (array; NOT added to mLogicArray) |
| 0x4390 | `mWoodLogicArray` | `NSMutableArray*` | 8 | setLogic `WoodLogic` -> WoodLogic on IslandDisplayGroup/Wood1..Wood3 (array) |
| 0x4398 | `mUnderwaterSceneryLogicArray` | `NSMutableArray*` | 4 | setLogic `SchoolOfFishLogic / UnderwaterBlob1Logic / UnderwaterBlob2Logic` -> UnderwaterSceneryLogic (type 0/1/2) on UnderwaterDisplayGroup/SchoolOfFish, UnderwaterBlob1, UnderwaterBlob2 (array) |
| 0x43a8 | `mSpearLogicArray` | `NSMutableArray*` | 11 | setLogic `SpearLogic` -> SpearLogic on IslandDisplayGroup/Spear (array (one entry)) |
| 0x43b0 | `mApeTorchLogicArray` | `NSMutableArray*` | 7 | setLogic `ApeTorchLogic` -> ApeTorchLogic on ApeIslandDisplayGroup/RightTorch, LeftTorch (array) |
| 0x43b8 | `mMonkeyLogicArray` | `NSMutableArray*` | 9 | setLogic `MonkeyLogic` -> MonkeyLogic on IslandDisplayGroup/Monkey1..Monkey3 (array) |
| 0x43c0 | `mJellyfishLogicArray` | `NSMutableArray*` | 8 | NSMutableArray of JellyfishLogic #93 (new BCMultiModel "Jellyfish1"); JellyfishLogic #94 (new BCMultiModel "Jellyfish2"); JellyfishLogic #95 (new BCMultiModel "Jellyfish3"); JellyfishLogic #96 (new BCMultiModel "Jellyfish4"); JellyfishLogic #97 (new BCMultiModel "Jellyfish5") |
| 0x43c8 | `mGodIdolLogicArray` | `NSMutableArray*` | 9 | NSMutableArray of GodIdolLogic #99 (new BCMultiModel "GodIdol1"); GodIdolLogic #100 (new BCMultiModel "GodIdol2"); GodIdolLogic #101 (new BCMultiModel "GodIdol3"); GodIdolLogic #102 (new BCMultiModel "GodIdol4") |
| 0x43d0 | `mApocalypseGemPedestalLogicArray` | `NSMutableArray*` | 5 | NSMutableArray of ApocalypseGemPedestalLogic #433 (ApocalypseIslandDisplayGroup/Pedestal1); ApocalypseGemPedestalLogic #434 (ApocalypseIslandDisplayGroup/Pedestal2); ApocalypseGemPedestalLogic #435 (ApocalypseIslandDisplayGroup/Pedestal3); ApocalypseGemPedestalLogic #436 (ApocalypseIslandDisplayGroup/Pedestal4); ApocalypseGemPedestalLogic #437 (ApocalypseIslandDisplayGroup/Pedestal5); ApocalypseGemPedestalLogic #438 (ApocalypseIslandDisplayGroup/Pedestal6); ApocalypseGemPedestalLogic #439 (ApocalypseIslandDisplayGroup/Pedestal7) |
| 0x43e0 | `mTimeTempleDoorLogicArray` | `NSMutableArray*` | 5 | NSMutableArray of ApocalypseTempleDoorLogic #457 (TimeTempleDisplayGroup/Door1); ApocalypseTempleDoorLogic #456 (TimeTempleDisplayGroup/Door2) |
| 0x43e8 | `mLocustTempleDoorLogicArray` | `NSMutableArray*` | 5 | NSMutableArray of ApocalypseTempleDoorLogic #470 (LocustTempleDisplayGroup/Door1); ApocalypseTempleDoorLogic #471 (LocustTempleDisplayGroup/Door2); ApocalypseTempleDoorLogic #472 (LocustTempleDisplayGroup/Door3) |
| 0x43f0 | `mDanceTempleDoorLogicArray` | `NSMutableArray*` | 5 | NSMutableArray of ApocalypseTempleDoorLogic #475 (DanceTempleDisplayGroup/Door1); ApocalypseTempleDoorLogic #476 (DanceTempleDisplayGroup/Door2) |
| 0x43f8 | `mPlagueTempleDoorLogicArray` | `NSMutableArray*` | 5 | NSMutableArray of ApocalypseTempleDoorLogic #504 (PlagueTempleDisplayGroup/Door1); ApocalypseTempleDoorLogic #505 (PlagueTempleDisplayGroup/Door2) |
| 0x4400 | `mPurificationTempleDoorLogicArray` | `NSMutableArray*` | 5 | NSMutableArray of ApocalypseTempleDoorLogic #508 (PurificationTempleDisplayGroup/Door1) |
| 0x4408 | `mClosetTempleDoorLogicArray` | `NSMutableArray*` | 5 | NSMutableArray of ApocalypseTempleDoorLogic #515 (ClosetTempleDisplayGroup/Door1) |
| 0x4410 | `mXenuTempleDoorLogicArray` | `NSMutableArray*` | 5 | NSMutableArray of ApocalypseTempleDoorLogic #519 (XenuTempleDisplayGroup/Door1) |
| 0x4418 | `mIdolPedestalLogicArray` | `NSMutableArray*` | 5 | NSMutableArray of IdolPedestalLogic #370 (GodIslandDisplayGroup/Pedestal1); IdolPedestalLogic #371 (GodIslandDisplayGroup/Pedestal2); IdolPedestalLogic #372 (GodIslandDisplayGroup/Pedestal3); IdolPedestalLogic #373 (GodIslandDisplayGroup/Pedestal4) |
| 0x4430 | `mWaterWingsLogic` | `WaterWingsLogic*` | 10 | shared water wings (carried between islands that enable them) - WaterWingsLogic #92 (new BCMultiModel "WaterWings") |
| 0x4438 | `mFishingPoleLogic` | `FishingPoleLogic*` | 9 | setLogic `FishingPoleLogic` -> FishingPoleLogic on IslandDisplayGroup/FishingPole |
| 0x4448 | `mTRexEggLogic` | `TRexEggLogic*` | 7 | setLogic `TRexEggLogic` -> TRexEggLogic on IslandDisplayGroup/TRexEgg |
| 0x4450 | `mTRexLogic` | `TRexLogic*` | 10 | setLogic `TRexLogic` -> TRexLogic on IslandDisplayGroup/TRex |
| 0x4458 | `mSpiderLogic` | `SpiderLogic*` | 10 | setLogic `SpiderLogic` -> SpiderLogic on IslandDisplayGroup/Spider |
| 0x4460 | `mSpiderTreeLogic` | `SpiderTreeLogic*` | 6 | setLogic `SpiderTreeLogic` -> SpiderTreeLogic on GraveyardIslandDisplayGroup/Tree |
| 0x4468 | `mFishLogic` | `FishLogic*` | 8 | setLogic `FishLogic` -> FishLogic on IslandDisplayGroup/Fish |
| 0x4470 | `mSwordFishLogic` | `SwordFishLogic*` | 4 | setLogic `SwordFishLogic` -> SwordFishLogic on IslandDisplayGroup/SwordFish |
| 0x4478 | `mBoulderLogic` | `BoulderLogic*` | 8 | setLogic `BoulderLogic` -> BoulderLogic on IslandDisplayGroup/Boulder |
| 0x4480 | `mOutHouseLogic` | `OutHouseLogic*` | 7 | setLogic `OutHouseLogic` -> OutHouseLogic on RockIslandDisplayGroup/Outhouse |
| 0x4488 | `mIceMonsterLogic` | `IceMonsterLogic*` | 8 | setLogic `IceMonsterLogic` -> IceMonsterLogic on IslandDisplayGroup/IceMonster |
| 0x4490 | `mGlacierLogic` | `GlacierLogic*` | 5 | setLogic `GlacierLogic` -> GlacierLogic on IslandDisplayGroup/Glacier |
| 0x4498 | `mIglooLogic` | `IglooLogic*` | 6 | setLogic `IglooLogic` -> IglooLogic on IceIslandDisplayGroup/Igloo |
| 0x44a0 | `mIceTreeLogic` | `IceTreeLogic*` | 5 | setLogic `IceTreeLogic` -> IceTreeLogic on IceIslandDisplayGroup/Tree |
| 0x44a8 | `mIceHoleLogic` | `IceHoleLogic*` | 7 | setLogic `IceHoleLogic` -> IceHoleLogic on IceIslandDisplayGroup/IceHole |
| 0x44b0 | `mApeLogic` | `ApeLogic*` | 8 | setLogic `ApeLogic` -> ApeLogic on ApeIslandDisplayGroup/Ape |
| 0x44b8 | `mApeAltarLogic` | `ApeAltarLogic*` | 6 | setLogic `ApeAltarLogic` -> ApeAltarLogic on ApeIslandDisplayGroup/Alter |
| 0x44c0 | `mApeBongoLogic` | `ApeBongoLogic*` | 8 | setLogic `ApeBongoLogic` -> ApeBongoLogic on ApeIslandDisplayGroup/Bongo |
| 0x44c8 | `mApeWenchLogic` | `ApeWenchLogic*` | 9 | setLogic `ApeWenchLogic` -> ApeWenchLogic on ApeIslandDisplayGroup/Wench |
| 0x44d0 | `mGoGoCageLogic` | `GoGoCageLogic*` | 4 | GoGoCageLogic #477 (DanceTempleDisplayGroup/DiscoCage) |
| 0x44d8 | `mDiscoBallLogic` | `DiscoBallLogic*` | 4 | DiscoBallLogic #478 (DanceTempleDisplayGroup/DiscoBall) |
| 0x44e0 | `mDiscoLightsLogic` | `DiscoLightsLogic*` | 4 | DiscoLightsLogic #479 (DanceTempleDisplayGroup/DiscoLights) |
| 0x44e8 | `mDanceBalloonBundleLogic` | `DanceBalloonBundleLogic*` | 4 | DanceBalloonBundleLogic #480 (DanceTempleDisplayGroup/BalloonBundle) |
| 0x44f0 | `mDanceBalloonLogic` | `DanceBalloonLogic*` | 4 | DanceBalloonLogic #481 (new BCMultiModel "DanceBalloon") |
| 0x44f8 | `mDanceDanceExecutionLogic` | `DanceDanceExecutionLogic*` | 4 | DanceDanceExecutionLogic #500 (DanceTempleDisplayGroup/DanceDanceExecution) |
| 0x4500 | `mBananaTreeLogic` | `BananaTreeLogic*` | 6 | setLogic `BananaTreeLogic` -> BananaTreeLogic on ApeIslandDisplayGroup/BananaTree |
| 0x4508 | `mBananaLogic` | `BananaLogic*` | 6 | setLogic `BananaLogic` -> BananaLogic on IslandDisplayGroup/Banana |
| 0x4510 | `mMoronSnakeLogic` | `MoronSnakeLogic*` | 8 | Moron Snake; starts Moron Pests - MoronSnakeLogic #51 (new BCMultiModel "MoronSnake") |
| 0x4518 | `mNoPeeLogic` | `NoPeeLogic*` | 2 | No Pee minigame driver - NoPeeLogic #91 (NoPeeDisplayGroup) |
| 0x4520 | `mUnderwaterTempleLogic` | `UnderwaterTempleLogic*` | 4 | coral-reef temple; underwaterTempleMode 0 none, 1 cold, 2 heat, 3 oil, 4 acid, 5 air - UnderwaterTempleLogic #547 (new BCMultiModel "UnderwaterTemple") |
| 0x4528 | `mTempleDoorLogic` | `TempleDoorLogic*` | 4 | TempleDoorLogic #374 (GodIslandDisplayGroup/Temple) |
| 0x4530 | `mApocalypseTempleDoorLogic` | `ApocalypseTempleDoorLogic*` | 4 | ApocalypseTempleDoorLogic #448 (ApocalypseIslandDisplayGroup/TempleDoor) |
| 0x4538 | `mApocalypsePortalTempleDoorLogic` | `ApocalypseTempleDoorLogic*` | 4 | ApocalypseTempleDoorLogic #449 (ApocalypseIslandDisplayGroup/TemplePortal) |
| 0x4540 | `mApocalypseCalendarLogic` | `ApocalypseCalendarLogic*` | 5 | ApocalypseCalendarLogic #447 (ApocalypseIslandDisplayGroup/ClockWheelMultiModel) |
| 0x4548 | `mApocalypseRoomFaderLogic` | `ApocalypseRoomFaderLogic*` | 1 | ApocalypseRoomFaderLogic #440 (new BCMultiModel "ApocalypseRoomFader") |
| 0x4550 | `mApocalypseTimeTempleLeverLogic` | `ApocalypseTimeTempleLeverLogic*` | 4 | ApocalypseTimeTempleLeverLogic #454 (TimeTempleDisplayGroup/Lever) |
| 0x4558 | `mApocalypseTimeTemplePlatformLogic` | `ApocalypseTimeTemplePlatformLogic*` | 4 | ApocalypseTimeTemplePlatformLogic #453 (TimeTempleDisplayGroup/Platform) |
| 0x4560 | `mLocustQueenLogic` | `LocustQueenLogic*` | 4 | LocustQueenLogic #461 (LocustTempleDisplayGroup/Queen) |
| 0x4568 | `mApocalypsePlagueTempleFountainLogic` | `ApocalypseTempleFountainLogic*` | 4 | ApocalypseTempleFountainLogic #503 (PlagueTempleDisplayGroup/PlagueFountain) |
| 0x4570 | `mApocalypsePurificationTempleFountainLogic` | `ApocalypseTempleFountainLogic*` | 4 | ApocalypseTempleFountainLogic #507 (PurificationTempleDisplayGroup/PurificationFountain) |
| 0x4578 | `mClosetTempleThetanMeterLogic` | `ClosetTempleThetanMeterLogic*` | 9 | ClosetTempleThetanMeterLogic #514 (ClosetTempleDisplayGroup/ThetanMeter) |
| 0x4580 | `mClosetTempleCouchLogic` | `ClosetTempleCouchLogic*` | 4 | ClosetTempleCouchLogic #512 (ClosetTempleDisplayGroup/Couch) |
| 0x4588 | `mClosetTempleClosetDoorLogic` | `ClosetTempleClosetDoorLogic*` | 4 | ClosetTempleClosetDoorLogic #513 (ClosetTempleDisplayGroup/SlidingDoor) |
| 0x4590 | `mXenuTempleLogic` | `XenuTempleLogic*` | 5 | XenuTempleLogic #517 (XenuTempleDisplayGroup/Background) |
| 0x4598 | `mXenuLogic` | `XenuLogic*` | 6 | XenuLogic #518 (XenuTempleDisplayGroup/Xenu) |
| 0x45a0 | `mMisfitOceanRockLogic` | `IslandObjectLogic*` | 3 | rock in the ocean on Misfit island (enterIsland: on Misfit) - IslandObjectLogic #49 (new BCMultiModel "MisfitOceanRock") |
| 0x45a8 | `mZombieGraveLogic` | `ZombieGraveLogic*` | 6 | setLogic `ZombieGraveLogic` -> ZombieGraveLogic on GraveyardIslandDisplayGroup/Grave |
| 0x45b0 | `mOctoStatueLogic` | `OctoStatueLogic*` | 5 | setLogic `OctoStatueLogic` -> OctoStatueLogic on SandIslandDisplayGroup/OctoStatue |
| 0x45b8 | `mKongStatueLogic` | `KongStatueLogic*` | 7 | setLogic `KongStatueLogic` -> KongStatueLogic on ApeIslandDisplayGroup/KongStatue |
| 0x45c0 | `mSpearStatueLogic` | `SpearStatueLogic*` | 3 | setLogic `SpearStatueLogic` -> SpearStatueLogic on UnderwaterRuinsDisplayGroup/SpearStatue |
| 0x45c8 | `mAnchorLogic` | `AnchorLogic*` | 7 | setLogic `AnchorLogic` -> AnchorLogic on UnderwaterRuinsDisplayGroup/Anchor |
| 0x45d0 | `mTileDrainLogic` | `TileDrainLogic*` | 6 | setLogic `TileDrainLogic` -> TileDrainLogic on UnderwaterRuinsDisplayGroup/TileDrain |
| 0x45d8 | `mTilePlugLogic` | `TilePlugLogic*` | 6 | setLogic `TilePlugLogic` -> TilePlugLogic on UnderwaterRuinsDisplayGroup/TilePlug |
| 0x45e0 | `mChumBucketLogic` | `ChumBucketLogic*` | 7 | setLogic `ChumBucketLogic` -> ChumBucketLogic on UnderwaterRuinsDisplayGroup/ChumBucket (also the target of manager "ChumBucket" items) |
| 0x45e8 | `mHealthBarLogic` | `HealthBarLogic*` | 3 | setLogic `HealthBarLogic` -> HealthBarLogic on PocketGod/HealthBar |
| 0x45f0 | `mAntHillLogic` | `AntHillLogic*` | 9 | setLogic `AntHillLogic` -> AntHillLogic on IslandDisplayGroup/AntHill |
| 0x45f8 | `mBonFireLogic` | `BonFireLogic*` | 8 | setLogic `BonFireLogic` -> BonFireLogic on IslandDisplayGroup/BonFire |
| 0x4600 | `mMagnifyingGlassLogic` | `MagnifyingGlassLogic*` | 6 | setLogic `MagnifyingGlassLogic` -> MagnifyingGlassLogic on IslandDisplayGroup/MagnifyingGlass |
| 0x4608 | `mSharkLogic` | `SharkLogic*` | 10 | setLogic `SharkLogic` -> SharkLogic on IslandDisplayGroup/Shark |
| 0x4610 | `mBirdLogic` | `BirdLogic*` | 8 | setLogic `BirdLogic` -> BirdLogic on IslandDisplayGroup/Bird |
| 0x4ae0 | `mCrackHoleLogic` | `CrackHoleLogic*` | 8 | ground crack drawn with a finger; a pygmy falling in starts The Runs; setLogic `CrackHoleLogic` -> CrackHoleLogic on IslandDisplayGroup/CrackHole |

### 8.4 `touch_input` (24)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x248 | `mAllTouches` | `NSArray*` | 4 | BCView mGameTouchArray of the event being dispatched (all fingers down) |
| 0x250 | `mTouchScreenPos` | `VECTOR4` | 4 | view position of the touch being dispatched (also read by onGameFrame: drag-scroll) |
| 0x260 | `mTouchLogicArray` | `id[1000]` | 3 | C array [1000] of retained selectable logics (front-to-back walk) for touch dispatch |
| 0x21a0 | `mCancelTouchLogicArray` | `id[1000]` | 1 | second [1000] array used by cancelAllTouches |
| 0x4798 | `mApeWenchTouch` | `BCTouch*` | 4 | finger cranking the ape wench |
| 0x47a0 | `mLastApeWenchPos` | `VECTOR4` | 2 | last wench-crank touch position |
| 0x47b4 | `mLastBubbleTime` | `float` | 1 | game time of the last underwater touch bubble (>= 0.4 s apart) |
| 0x499c | `mPinchSunState` | `int` | 2 | mSunState when the pinch began |
| 0x49a0 | `mPinchSunDistance` | `float` | 2 | finger distance when the pinch began |
| 0x49a8 | `mPinchSunTouch0` | `BCTouch*` | 3 | pinch finger 0 |
| 0x49b0 | `mPinchSunTouch1` | `BCTouch*` | 3 | pinch finger 1 |
| 0x4ad0 | `mbDragCrack` | `BOOL` | 4 | crack drawing in progress |
| 0x4ad8 | `mDragCrackTouch` | `BCTouch*` | 4 | finger drawing the crack |
| 0x4ae8 | `mbDragOcean` | `BOOL` | 5 | ocean drag in progress |
| 0x4aec | `mOceanDragVel` | `float` | 3 | ocean flick velocity (+-5000) |
| 0x4af0 | `mDragOceanAveVel` | `float` | 2 | smoothed ocean drag velocity (-1 = unset) |
| 0x4af4 | `mDragOceanTimeStamp` | `float` | 2 | last ocean-drag timestamp |
| 0x4af8 | `mDragOceanTouch` | `BCTouch*` | 5 | finger dragging the ocean |
| 0x4b00 | `mDragOceanTouchPos` | `VECTOR4` | 2 | last ocean-drag position |
| 0x4b10 | `mFrontOceanLogic` | `OceanLogic*` | 7 | OceanLogic front tiles; setLogic `FrontOceanLogic` -> OceanLogic on IslandDisplayGroup/FrontOceanTiles (setFrontOcean:YES posVelScale:1.0) |
| 0x4b18 | `mBehindOceanLogic` | `OceanLogic*` | 7 | OceanLogic behind tiles; setLogic `BehindOceanLogic` -> OceanLogic on IslandDisplayGroup/BehindOceanTiles (setFrontOcean:NO posVelScale:0.5) |
| 0x4b20 | `mUnderwaterTouchStartYPos` | `float` | 2 | y where the underwater camera drag began |
| 0x4b28 | `mUnderwaterTouch` | `BCTouch*` | 5 | finger dragging underwater |
| 0x4b30 | `mUnderwaterTouchPos` | `VECTOR4` | 2 | last underwater drag position |

### 8.5 `menus_and_screens` (73)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x4160 | `mSandIslandMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `SandIslandMenuDisplayGroup` |
| 0x4168 | `mRockIslandMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `RockIslandMenuDisplayGroup` |
| 0x4170 | `mGraveyardIslandMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `GraveyardIslandMenuDisplayGroup` |
| 0x4178 | `mIceIslandMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `IceIslandMenuDisplayGroup` |
| 0x4180 | `mApeIslandMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `ApeIslandMenuDisplayGroup` |
| 0x4188 | `mMisfitIslandMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `MisfitIslandMenuDisplayGroup` |
| 0x4190 | `mGodIslandMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `GodIslandMenuDisplayGroup` |
| 0x4198 | `mFailBoatMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `FailBoatMenuDisplayGroup` |
| 0x41a0 | `mApocalypseIslandMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `ApocalypseIslandMenuDisplayGroup` |
| 0x41a8 | `mTimeTempleMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `TimeTempleMenuDisplayGroup` |
| 0x41b0 | `mLocustTempleMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `LocustTempleMenuDisplayGroup` |
| 0x41b8 | `mDanceTempleMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `DanceTempleMenuDisplayGroup` |
| 0x41c0 | `mPlagueTempleMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `PlagueTempleMenuDisplayGroup` |
| 0x41c8 | `mPurificationTempleMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `PurificationTempleMenuDisplayGroup` |
| 0x41d0 | `mClosetTempleMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `ClosetTempleMenuDisplayGroup` |
| 0x41d8 | `mXenuTempleMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `XenuTempleMenuDisplayGroup` |
| 0x41e0 | `mUnderwaterMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `UnderwaterMenuDisplayGroup` |
| 0x41e8 | `mUnderwaterCoralReefMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `UnderwaterCoralReefMenuDisplayGroup` |
| 0x41f0 | `mGlobalIslandMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `GlobalIslandMenuDisplayGroup` |
| 0x41f8 | `mGodPowerMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `GodPowerMenuDisplayGroup` |
| 0x4200 | `mCustomizeMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `CustomizeMenuDisplayGroup` |
| 0x4208 | `mCommunityMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `CommunityMenuDisplayGroup` |
| 0x4210 | `mSettingsMenuDisplayGroup` | `BCDisplayGroup*` | 2 | group `SettingsMenuDisplayGroup` |
| 0x4218 | `mMenuLabelsDisplayGroup` | `BCDisplayGroup*` | 4 | group `MenuLabelsDisplayGroup` |
| 0x4248 | `mCategoryMenuDisplayMarker` | `BCDisplayMarker*` | 3 | marker `CategoryMenuDisplayMarker` |
| 0x4338 | `mCategoryMenuState` | `int` | 2 | open category menu: 0 none, 1 island, 2 global island, 3 god powers, 4 customize, 5 community, 6 settings |
| 0x4680 | `mMessageDisplayObject` | `BCDisplayObject*` | 2 | BCModel "Message" shown by showMessage::: (after MenuDisplayMarker) |
| 0x4688 | `mMessageSequence` | `BCSequence*` | 3 | BCSequence running "ShowMessage" |
| 0x4758 | `mbMenuOpen` | `BOOL` | 3 | main menu (MenuButton) open |
| 0x475c | `mTargetMenuButtonZRot` | `float` | 4 | menu button target z-rotation (90 closed, 0 open), eased 50 %/frame |
| 0x4760 | `mTargetMenuYPos` | `float` | 4 | MenuDisplayGroup target y (90 closed, 0 open), eased 50 %/frame |
| 0x4b50 | `mNewPygmyButtonLogic` | `BCButtonLogic*` | 4 | setLogic `NewPygmyButtonLogic` -> BCButtonLogic on PocketGod/NewPygmyButton (main UI) |
| 0x4b58 | `mMenuButtonLogic` | `BCButtonLogic*` | 5 | setLogic `MenuButtonLogic` -> BCButtonLogic on PocketGod/MenuButton (main UI) |
| 0x4b60 | `mIslandLocationButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #561 (MenuDisplayGroup/IslandLocationButton) |
| 0x4b68 | `mIslandMenuButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #562 (MenuDisplayGroup/IslandMenuButton) |
| 0x4b70 | `mGlobalIslandMenuButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #563 (MenuDisplayGroup/GlobalIslandMenuButton) |
| 0x4b78 | `mGodPowerMenuButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #564 (MenuDisplayGroup/GodPowerMenuButton) |
| 0x4b80 | `mCustomizeMenuButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #565 (MenuDisplayGroup/CustomizeMenuButton) |
| 0x4b88 | `mCommunityMenuButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #566 (MenuDisplayGroup/CommunityMenuButton) |
| 0x4b90 | `mSettingsMenuButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #567 (MenuDisplayGroup/SettingsMenuButton) |
| 0x4b98 | `mHelpButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #568 (MenuDisplayGroup/HelpButton) |
| 0x4ba0 | `mDebugButtonLogic` | `BCButtonLogic*` | 1 | never assigned (loadGame removes table #569 DebugButton from MenuDisplayGroup instead), so the debug screen is unreachable |
| 0x4c98 | `mIdolSelectionButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #600 (GodIslandMenuDisplayGroup/IdolSelectionButton) "Idols Selection" |
| 0x4cf0 | `mCrackHoleButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #611 (GlobalIslandMenuDisplayGroup/CrackHoleButton) "Draw Crack on Ground" |
| 0x4cf8 | `mTheRunsButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #612 (GlobalIslandMenuDisplayGroup/TheRunsButton) "The Runs Game" |
| 0x4d00 | `mCharlieButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #613 (GlobalIslandMenuDisplayGroup/CharlieButton) "Charlie" |
| 0x4d08 | `mDrumButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #614 (GodPowerMenuDisplayGroup/DrumButton) "Tap to Drum and Dance" |
| 0x4d10 | `mTemperatureButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #615 (GodPowerMenuDisplayGroup/TemperatureButton) "Pinch Sun to Change Temp" |
| 0x4d18 | `mHurricaneButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #616 (GodPowerMenuDisplayGroup/HurricaneButton) "Swirl Hurricane" |
| 0x4d20 | `mRainbowButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #617 (GodPowerMenuDisplayGroup/RainbowButton) "Draw Double Rainbow" |
| 0x4d28 | `mSkinPackButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #618 (CustomizeMenuDisplayGroup/SkinPackButton) "Customize with Skin Packs" |
| 0x4d30 | `mBonusPackButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #619 (CustomizeMenuDisplayGroup/BonusPackButton) "Bonus Packs" |
| 0x4d38 | `mStoryButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #620 (CustomizeMenuDisplayGroup/StoryButton) "Create Stories" |
| 0x4d40 | `mStatsButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #621 (CustomizeMenuDisplayGroup/StatsButton) "Tribe Stats" |
| 0x4d48 | `mIdolsChallengeButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #622 (CustomizeMenuDisplayGroup/IdolsChallengeButton) "Idols Challenge" |
| 0x4d50 | `mNewsButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #623 (CommunityMenuDisplayGroup/NewsButton) "Pocket God News" |
| 0x4d58 | `mChallengeButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #625 (CommunityMenuDisplayGroup/ChallengeButton) "Challenge Friends" |
| 0x4d60 | `mStartScreenButtonLogic` | `BCButtonLogic*` | 2 | BCButtonLogic #626 (CommunityMenuDisplayGroup/StartScreenButton) "Show Start Screen" |
| 0x4d68 | `mMuteButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #627 (SettingsMenuDisplayGroup/MuteButton) "Mute" |
| 0x4d70 | `mVibrateButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #628 (SettingsMenuDisplayGroup/VibrateButton) "Vibrate" |
| 0x4d78 | `mGravityButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #629 (SettingsMenuDisplayGroup/GravityButton) "Tilt for Gravity" |
| 0x4d80 | `mNamesButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #630 (SettingsMenuDisplayGroup/NamesButton) "Name Labels" |
| 0x4d88 | `mLabelsButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #631 (SettingsMenuDisplayGroup/LabelsButton) "Menu Labels" |
| 0x4d90 | `mHelpPageIndex` | `int` | 3 | help page 0..68 (wraps) |
| 0x4d94 | `mbHelpScreenOpen` | `BOOL` | 7 | help open |
| 0x4d98 | `mHelpCloseButtonLogic` | `BCButtonLogic*` | 2 | setLogic `HelpCloseButtonLogic` -> BCButtonLogic on Help/CloseButton (Help UI) |
| 0x4da0 | `mHelpNextButtonLogic` | `BCButtonLogic*` | 2 | setLogic `HelpNextButtonLogic` -> BCButtonLogic on Help/NextButton (Help UI) |
| 0x4da8 | `mHelpPrevButtonLogic` | `BCButtonLogic*` | 2 | setLogic `HelpPrevButtonLogic` -> BCButtonLogic on Help/PrevButton (Help UI) |
| 0x4db0 | `mbDebugScreenOpen` | `BOOL` | 8 | debug screen open |
| 0x4dc5 | `mbWhatsNewScreenOpen` | `BOOL` | 3 | WhatsNew screen open (never set YES in 1.48.2) |
| 0x4dc8 | `mWhatsNewScreenGoButtonLogic` | `BCButtonLogic*` | 2 | setLogic `WhatsNewScreenGoButtonLogic` -> BCButtonLogic on WhatsNew/GoButton (WhatsNew UI) |
| 0x4de8 | `mbStatsScreenOpen` | `BOOL` | 8 | stats screens open |
| 0x4e88 | `mLavaButtonLogic` | `BCButtonLogic*` | 1 | setLogic `LavaButtonLogic` -> BCButtonLogic on (no sequence uses this id) (main UI) |

### 8.6 `island_locations` (41)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x40e8 | `mMiscSequence` | `BCSequence*` | 3 | BCSequence running the per-location Start<Location> script |
| 0x4138 | `mUnderwaterDisplayGroup` | `BCDisplayGroup*` | 10 | group `UnderwaterDisplayGroup` |
| 0x4140 | `mUnderwaterRuinsDisplayGroup` | `BCDisplayGroup*` | 2 | group `UnderwaterRuinsDisplayGroup` |
| 0x4148 | `mUnderwaterCoralReefDisplayGroup` | `BCDisplayGroup*` | 2 | group `UnderwaterCoralReefDisplayGroup` |
| 0x4150 | `mIslandDisplayGroup` | `BCDisplayGroup*` | 33 | display group `IslandDisplayGroup` (world root: sky, ocean, current island inserted at IslandDisplayMarker); vertical scroll in onGameFrame: |
| 0x4250 | `mIslandRootDisplayMarker` | `BCDisplayMarker*` | 17 | marker `IslandRootDisplayMarker` |
| 0x4258 | `mUnderwaterDisplayMarker` | `BCDisplayMarker*` | 4 | marker `UnderwaterDisplayMarker` |
| 0x4260 | `mUnderwaterRealmDisplayMarker` | `BCDisplayMarker*` | 2 | marker `UnderwaterRealmDisplayMarker` |
| 0x42b0 | `mIslandDisplayMarker` | `BCDisplayMarker*` | 2 | marker `IslandDisplayMarker` |
| 0x42b8 | `mSandIslandDisplayGroup` | `BCDisplayGroup*` | 3 | group `SandIslandDisplayGroup` |
| 0x42c0 | `mRockIslandDisplayGroup` | `BCDisplayGroup*` | 3 | group `RockIslandDisplayGroup` |
| 0x42c8 | `mGraveyardIslandDisplayGroup` | `BCDisplayGroup*` | 3 | group `GraveyardIslandDisplayGroup` |
| 0x42d0 | `mIceIslandDisplayGroup` | `BCDisplayGroup*` | 3 | group `IceIslandDisplayGroup` |
| 0x42d8 | `mApeIslandDisplayGroup` | `BCDisplayGroup*` | 3 | group `ApeIslandDisplayGroup` |
| 0x42e0 | `mMisfitIslandDisplayGroup` | `BCDisplayGroup*` | 3 | group `MisfitIslandDisplayGroup` |
| 0x42e8 | `mGodIslandDisplayGroup` | `BCDisplayGroup*` | 3 | group `GodIslandDisplayGroup` |
| 0x42f0 | `mFailBoatOceanDisplayGroup` | `BCDisplayGroup*` | 3 | group `FailBoatOceanDisplayGroup` |
| 0x42f8 | `mApocalypseIslandDisplayGroup` | `BCDisplayGroup*` | 3 | group `ApocalypseIslandDisplayGroup` |
| 0x4300 | `mTimeTempleDisplayGroup` | `BCDisplayGroup*` | 6 | group `TimeTempleDisplayGroup` |
| 0x4308 | `mLocustTempleDisplayGroup` | `BCDisplayGroup*` | 6 | group `LocustTempleDisplayGroup` |
| 0x4310 | `mDanceTempleDisplayGroup` | `BCDisplayGroup*` | 6 | group `DanceTempleDisplayGroup` |
| 0x4318 | `mPlagueTempleDisplayGroup` | `BCDisplayGroup*` | 6 | group `PlagueTempleDisplayGroup` |
| 0x4320 | `mPurificationTempleDisplayGroup` | `BCDisplayGroup*` | 6 | group `PurificationTempleDisplayGroup` |
| 0x4328 | `mClosetTempleDisplayGroup` | `BCDisplayGroup*` | 6 | group `ClosetTempleDisplayGroup` |
| 0x4330 | `mXenuTempleDisplayGroup` | `BCDisplayGroup*` | 6 | group `XenuTempleDisplayGroup` |
| 0x4b40 | `mFrontTsunamiLogic` | `TsunamiLogic*` | 3 | TsunamiLogic front layer (UnderwaterDisplayGroup/FrontTsunami); setLogic `FrontTsunamiLogic` -> TsunamiLogic on UnderwaterDisplayGroup/FrontTsunami (setFrontLayer:YES) |
| 0x4b48 | `mBehindTsunamiLogic` | `TsunamiLogic*` | 3 | TsunamiLogic back layer (IslandDisplayGroup/BehindTsunami); setLogic `BehindTsunamiLogic` -> TsunamiLogic on IslandDisplayGroup/BehindTsunami (setFrontLayer:NO) |
| 0x4e90 | `mbIslandLocationTransition` | `BOOL` | 7 | tsunami transition to/from underwater in progress |
| 0x4e94 | `mLastIslandLocationState` | `int` | 7 | previous location (-1 at init) |
| 0x4e98 | `mIslandLocationState` | `int` | 99 | current location 0..17 (-1 at init) |
| 0x4f08 | `mIslandMinXPos` | `float` | 5 | walkable min x of the location |
| 0x4f0c | `mIslandMaxXPos` | `float` | 5 | walkable max x |
| 0x4f10 | `mIslandCenterXPos` | `float` | 2 | island centre x |
| 0x4f14 | `mHorizonYPos` | `float` | 3 | horizon y (160; 120 on Ape island) |
| 0x4f18 | `mSunTransitionYPos` | `float` | 2 | = horizon |
| 0x4f1c | `mSunUnderHorizonTriggerYPos` | `float` | 2 | horizon - 25 (night trigger) |
| 0x4f20 | `mSunUnderHorizonYPos` | `float` | 3 | horizon - 40 (sun floor) |
| 0x4f24 | `mSunOverHorizonYPos` | `float` | 1 | horizon + 40 (unused) |
| 0x4f28 | `mMoonUnderHorizonTriggerYPos` | `float` | 2 | horizon - 25 (sunset trigger) |
| 0x4f2c | `mMoonUnderHorizonYPos` | `float` | 4 | horizon - 40 (moon floor) |
| 0x4f30 | `mMoonOverHorizonYPos` | `float` | 1 | horizon + 40 (unused) |

### 8.7 `island_features` (68)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x4a78 | `mSharkState` | `int` | 3 | shark hazard 0/1 (option SharkState) |
| 0x4a7c | `mBirdState` | `int` | 2 | bird hazard 0/1 (option BirdState) |
| 0x4ba8 | `mSandBoulderButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #570 (SandIslandMenuDisplayGroup/BoulderButton) "Meteor" |
| 0x4bb0 | `mSandFishButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #571 (SandIslandMenuDisplayGroup/FishButton) "Fishing" |
| 0x4bb8 | `mSandAntHillButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #572 (SandIslandMenuDisplayGroup/AntHillButton) "Ant Hill" |
| 0x4bc0 | `mSandWaterWingsButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #573 (SandIslandMenuDisplayGroup/WaterWingsButton) "Water Wings" |
| 0x4bc8 | `mRockFishButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #574 (RockIslandMenuDisplayGroup/FishButton) "Fishing" |
| 0x4bd0 | `mRockWoodButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #578 (RockIslandMenuDisplayGroup/WoodButton) "Wood" |
| 0x4bd8 | `mRockEggButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #579 (RockIslandMenuDisplayGroup/EggButton) "T-Rex Egg" |
| 0x4be0 | `mRockOutHouseButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #575 (RockIslandMenuDisplayGroup/OutHouseButton) "Outhouse" |
| 0x4be8 | `mRockTRexButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #576 (RockIslandMenuDisplayGroup/TRexButton) "T-Rex Boss" |
| 0x4bf0 | `mRockSpearButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #577 (RockIslandMenuDisplayGroup/SpearButton) "Spear" |
| 0x4bf8 | `mGraveyardZombieGraveButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #580 (GraveyardIslandMenuDisplayGroup/ZombieGraveButton) "Zombie Grave" |
| 0x4c00 | `mGraveyardGhostButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #581 (GraveyardIslandMenuDisplayGroup/GhostButton) "Ghosts" |
| 0x4c08 | `mGraveyardSpiderWebButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #582 (GraveyardIslandMenuDisplayGroup/SpiderWebButton) "Spider Web" |
| 0x4c10 | `mGraveyardSpiderButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #583 (GraveyardIslandMenuDisplayGroup/SpiderButton) "Spider Boss" |
| 0x4c18 | `mGraveyardSpearButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #584 (GraveyardIslandMenuDisplayGroup/SpearButton) "Spear" |
| 0x4c20 | `mGraveyardWaterWingsButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #585 (GraveyardIslandMenuDisplayGroup/WaterWingsButton) "Water Wings" |
| 0x4c28 | `mIceFishButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #586 (IceIslandMenuDisplayGroup/FishButton) "Fishing" |
| 0x4c30 | `mIceWaterWingsButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #587 (IceIslandMenuDisplayGroup/WaterWingsButton) "Water Wings" |
| 0x4c38 | `mIceIceMonsterButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #588 (IceIslandMenuDisplayGroup/IceMonsterButton) "Ice Monster" |
| 0x4c40 | `mIceIglooButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #589 (IceIslandMenuDisplayGroup/IglooButton) "Igloo" |
| 0x4c48 | `mIceIceHoleButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #590 (IceIslandMenuDisplayGroup/IceHoleButton) "Ice Hole" |
| 0x4c50 | `mIceIceSquidButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #591 (IceIslandMenuDisplayGroup/IceSquidButton) "Ice Squid" |
| 0x4c58 | `mApeApeAltarButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #592 (ApeIslandMenuDisplayGroup/ApeAltarButton) "Ape Altar" |
| 0x4c60 | `mApeApeButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #593 (ApeIslandMenuDisplayGroup/ApeApeButton) "Giant Ape" |
| 0x4c68 | `mApeBananaTreeButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #594 (ApeIslandMenuDisplayGroup/ApeBananaTreeButton) "Banana Tree" |
| 0x4c70 | `mApeMonkeyButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #595 (ApeIslandMenuDisplayGroup/ApeMonkeyButton) "Monkeys" |
| 0x4c78 | `mApeKonkeyDongButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #596 (ApeIslandMenuDisplayGroup/ApeKonkeyDongButton) "Start Konkey Dong When Kidnapped" |
| 0x4c80 | `mMisfitMoronSnakeButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #597 (MisfitIslandMenuDisplayGroup/MoronSnakeButton) "Moron Snake" |
| 0x4c88 | `mMisfitWaterWingsButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #598 (MisfitIslandMenuDisplayGroup/WaterWingsButton) "Water Wings" |
| 0x4c90 | `mMisfitMoronPestsButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #599 (MisfitIslandMenuDisplayGroup/MisfitMoronPestsButton) "Moron Pests Game" |
| 0x4ca0 | `mIdolsButtonLogic` | `BCButtonLogic*` | 5 | BCButtonLogic #601 (GodIslandMenuDisplayGroup/IdolsButton) "Idols" |
| 0x4ca8 | `mIdolWaterWingsButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #602 (GodIslandMenuDisplayGroup/WaterWingsButton) "Water Wings" |
| 0x4cb0 | `mApocalypseWaterWingsButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #603 (ApocalypseIslandMenuDisplayGroup/WaterWingsButton) "Water Wings" |
| 0x4cb8 | `mUnderwaterPiranhaButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #604 (UnderwaterMenuDisplayGroup/PiranhaButton) "Bait Master Game" |
| 0x4cc0 | `mUnderwaterLaserSharkButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #605 (UnderwaterMenuDisplayGroup/LaserSharkButton) "Shark Blast Game" |
| 0x4cc8 | `mUnderwaterTileDrainButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #606 (UnderwaterMenuDisplayGroup/TileDrainButton) "Pain Drain Game" |
| 0x4cd0 | `mUnderwaterCoralReefJellyfishButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #607 (UnderwaterCoralReefMenuDisplayGroup/JellyfishButton) "Jellyfish" |
| 0x4ce0 | `mSharkButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #609 (GlobalIslandMenuDisplayGroup/SharkButton) "Shark" |
| 0x4ce8 | `mBirdButtonLogic` | `BCButtonLogic*` | 3 | BCButtonLogic #610 (GlobalIslandMenuDisplayGroup/BirdButton) "Bird" |
| 0x4e9c | `mSandIslandState` | `int` | 8 | option `SandIslandHazard`: 0 none, 1 boulder (meteor), 2 fishing, 3 ant hill, 4 water wings; default 3 |
| 0x4ea0 | `mRockIslandState` | `int` | 5 | option `RockIslandHazard`: 0/1 fishing; default 1 |
| 0x4ea4 | `mRockIslandFeatureState` | `int` | 9 | option `RockIslandFeature`: 0 none, 1 wood + bonfire, 2 T-Rex egg (spears/T-Rex need it); default 1 |
| 0x4ea8 | `mRockIslandOutHouseState` | `int` | 4 | option `RockIslandOutHouseState`: 0/1 outhouse; default 1 |
| 0x4eac | `mRockIslandTRexState` | `int` | 4 | option `RockIslandTRexState`: 0/1 T-Rex boss; default 1 |
| 0x4eb0 | `mRockIslandSpearState` | `int` | 4 | option `RockIslandSpearState`: 0/1 spears; default 1 |
| 0x4eb4 | `mGraveyardIslandState` | `int` | 5 | option `GraveyardIslandState`: 0/1 water wings; default 0 |
| 0x4eb8 | `mGraveyardIslandZombieGraveState` | `int` | 4 | option `GraveyardIslandZombieGraveState`: 0/1 zombie grave; default 1 |
| 0x4ebc | `mGraveyardIslandGhostState` | `int` | 3 | option `GraveyardIslandGhostState`: 0/1 ghosts; default 1 |
| 0x4ec0 | `mGraveyardIslandSpiderWebState` | `int` | 6 | option `GraveyardIslandSpiderWebState`: 0/1 spider web tree (spider + spears need it); default 1 |
| 0x4ec4 | `mGraveyardIslandSpiderState` | `int` | 4 | option `GraveyardIslandSpiderState`: 0/1 spider boss; default 1 |
| 0x4ec8 | `mGraveyardIslandSpearState` | `int` | 4 | option `GraveyardIslandSpearState`: 0/1 spears; default 1 |
| 0x4ecc | `mIceIslandState` | `int` | 6 | option `IceIslandState`: 0 none, 1 fishing, 2 water wings; default 1 |
| 0x4ed0 | `mIceIslandIceMonsterState` | `int` | 4 | option `IceIslandIceMonsterState`: 0/1 ice monster; default 1 |
| 0x4ed4 | `mIceIslandIglooState` | `int` | 4 | option `IceIslandIglooState`: 0/1 igloo; default 1 |
| 0x4ed8 | `mIceIslandIceHoleState` | `int` | 5 | option `IceIslandIceHoleState`: 0/1 ice hole; default 1 |
| 0x4edc | `mIceIslandIceSquidState` | `int` | 4 | option `IceIslandIceSquidState`: 0/1 ice squid (needs ice hole); default 1 |
| 0x4ee0 | `mApeIslandState` | `int` | 10 | option `ApeIslandState`: 0 none, 1 ape altar + bongo + wench, 2 banana tree + Kong statue; default 2 |
| 0x4ee4 | `mApeIslandApeState` | `int` | 4 | option `ApeIslandApeState`: 0/1 giant ape; default 1 |
| 0x4ee8 | `mApeIslandMonkeyState` | `int` | 3 | option `ApeIslandMonkeyState`: 0/1 monkeys; default 1 |
| 0x4eec | `mMisfitIslandState` | `int` | 6 | option `MisfitIslandState`: 0 none, 1 moron snake, 2 water wings; default 1 |
| 0x4ef0 | `mGodIslandState` | `int` | 5 | option `GodIslandState`: 0/1 water wings; default 0 |
| 0x4ef4 | `mGodIslandIdolsState` | `int` | 5 | option `GodIslandIdolsState`: 0/1 god idols on pedestals; default 1 |
| 0x4ef8 | `mApocalypseIslandState` | `int` | 5 | option `ApocalypseIslandState`: 0/1 water wings; default 0 |
| 0x4efc | `mFailBoatState` | `int` | 2 | option `FailBoatState`: toggle value only (no objects) |
| 0x4f00 | `mUnderwaterState` | `int` | 5 | option `UnderwaterHazard`: 0 none, 1 piranha (anchor, Bait Master), 2 laser shark (chum bucket), 3 tile drain + plug (Pain Drain); default 2 |
| 0x4f04 | `mUnderwaterCoralReefState` | `int` | 6 | option `UnderwaterCoralReefState`: 0/1 jellyfish; default 1 |

### 8.8 `sky_weather` (55)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x4220 | `mCloudFrontDisplayMarker` | `BCDisplayMarker*` | 1 | marker `CloudFrontDisplayMarker` |
| 0x4228 | `mCloudBehindDisplayMarker` | `BCDisplayMarker*` | 2 | marker `CloudBehindDisplayMarker` |
| 0x4618 | `mBatTapTouchCount` | `int` | 2 | taps on the moon (resets after 1 s idle); 3rd tap spawns the bat |
| 0x461c | `mBatTapTouchTime` | `float` | 2 | seconds since last moon tap (onGameFrame: accumulates) |
| 0x4620 | `mBatLogic` | `BatLogic*` | 7 | BatLogic spawned by checkForBatTapping: (nil otherwise) |
| 0x46b0 | `mWeatherSequence` | `BCSequence*` | 4 | BCSequence for StartStorm/StopStorm |
| 0x4968 | `mSunLogic` | `SunLogic*` | 9 | the sun (setSunState:, transitionIn:/transitionOut:, eclipse); setLogic `SunLogic` -> SunLogic on IslandDisplayGroup/Sun |
| 0x4970 | `mDragSunTouch` | `BCTouch*` | 4 | finger dragging the sun |
| 0x4978 | `mDragSunTouchPos` | `VECTOR4` | 2 | last sun-drag position |
| 0x4988 | `mSunReflectionDisplayObject` | `BCDisplayObject*` | 7 | island object "SunReflection" (retained, inserted before CloudBehindDisplayMarker at sunset) |
| 0x4990 | `mbSunReflectionVisible` | `BOOL` | 3 | sun reflection inserted |
| 0x4994 | `mDragSunXOrigin` | `float` | 1 | x at sun-drag start (write-only) |
| 0x4998 | `mSunState` | `int` | 4 | sun size: 0 normal, 1 big (spread), 2 small (pinch) |
| 0x49b8 | `mStarsLogic` | `MultiModelLogic*` | 3 | night stars (MoonLogic re-inserts it); setLogic `StarsLogic` -> MultiModelLogic on IslandDisplayGroup/Stars (display object removed from its group at creation) |
| 0x49c0 | `mMoonLogic` | `MoonLogic*` | 8 | the moon (transitionIn:/transitionOut:; listener asks starsLogic:); setLogic `MoonLogic` -> MoonLogic on IslandDisplayGroup/Moon |
| 0x49c8 | `mDragMoonTouch` | `BCTouch*` | 4 | finger dragging the moon |
| 0x49d0 | `mDragMoonTouchPos` | `VECTOR4` | 2 | last moon-drag position |
| 0x49e0 | `mbNightActive` | `BOOL` | 9 | night |
| 0x49e1 | `mbSunsetActive` | `BOOL` | 8 | sunset |
| 0x49e4 | `mCloudGradientTargetFade` | `float` | 3 | target weight of CloudyGradient (1 storm) |
| 0x49e8 | `mCloudGradientFade` | `float` | 2 | current weight (eased 13 %/frame); 0.01 at load |
| 0x49ec | `mSunsetGradientTargetFade` | `float` | 5 | target weight of SunsetGradient |
| 0x49f0 | `mSunsetGradientFade` | `float` | 2 | current weight |
| 0x49f4 | `mNightGradientTargetFade` | `float` | 5 | target weight of NightGradient |
| 0x49f8 | `mNightGradientFade` | `float` | 1 | current weight |
| 0x49fc | `mEclipseGradientTargetFade` | `float` | 3 | target weight of EclipseGradient (boulder over sun) |
| 0x4a00 | `mEclipseGradientFade` | `float` | 1 | current weight |
| 0x4a04 | `mBigSunGradientTargetFade` | `float` | 2 | target weight of BigSunGradient |
| 0x4a08 | `mBigSunGradientFade` | `float` | 1 | current weight |
| 0x4a0c | `mSmallSunGradientTargetFade` | `float` | 2 | target weight of SmallSunGradient |
| 0x4a10 | `mSmallSunGradientFade` | `float` | 1 | current weight |
| 0x4a14 | `mApocalypseGradientTargetFade` | `float` | 4 | target weight of the Apocalypse* gradients (1 during apocalypse) |
| 0x4a18 | `mApocalypseGradientFade` | `float` | 1 | current weight |
| 0x4a20 | `mpSunsetGradient` | `BCGradientDef*` | 2 | gradient `SunsetGradient` |
| 0x4a28 | `mpNightGradient` | `BCGradientDef*` | 2 | gradient `NightGradient` |
| 0x4a30 | `mpDayGradient` | `BCGradientDef*` | 2 | gradient `DayGradient` |
| 0x4a38 | `mpCloudyGradient` | `BCGradientDef*` | 2 | gradient `CloudyGradient` |
| 0x4a40 | `mpEclipseGradient` | `BCGradientDef*` | 2 | gradient `EclipseGradient` |
| 0x4a48 | `mpBigSunGradient` | `BCGradientDef*` | 2 | gradient `BigSunGradient` |
| 0x4a50 | `mpSmallSunGradient` | `BCGradientDef*` | 2 | gradient `SmallSunGradient` |
| 0x4a58 | `mpApocalypseSunsetGradient` | `BCGradientDef*` | 2 | gradient `ApocalypseSunsetGradient` |
| 0x4a60 | `mpApocalypseNightGradient` | `BCGradientDef*` | 2 | gradient `ApocalypseNightGradient` |
| 0x4a68 | `mpApocalypseDayGradient` | `BCGradientDef*` | 2 | gradient `ApocalypseDayGradient` |
| 0x4a70 | `mpApocalypseCloudyGradient` | `BCGradientDef*` | 2 | gradient `ApocalypseCloudyGradient` |
| 0x4a80 | `mWeatherState` | `int` | 9 | weather 0 clear, 1 storm (option WeatherHazard) |
| 0x4a84 | `mbDragClouds` | `BOOL` | 5 | cloud drag in progress |
| 0x4a88 | `mCloudDragVel` | `float` | 8 | cloud flick velocity (+-5000 clamp; 4000 when toggled by button) |
| 0x4a8c | `mDragCloudsAveVel` | `float` | 2 | smoothed cloud drag velocity (-1 = unset) |
| 0x4a90 | `mDragCloudsTimeStamp` | `float` | 2 | last cloud-drag timestamp |
| 0x4a98 | `mDragCloudsTouch` | `BCTouch*` | 5 | finger dragging the clouds |
| 0x4aa0 | `mDragCloudsTouchPos` | `VECTOR4` | 2 | last cloud-drag position |
| 0x4ab0 | `mCloudLogicArray` | `NSMutableArray*` | 10 | 4 CloudLogic ("NewCloud" BCModel with Cloud1Texture) in fair weather |
| 0x4ab8 | `mFrontStormCloudLogic` | `StormCloudLogic*` | 8 | StormCloudLogic front layer; setLogic `FrontStormCloudLogic` -> StormCloudLogic on IslandDisplayGroup/FrontStormCloud (setFrontStormCloud:YES posVelScale:1.0) |
| 0x4ac0 | `mBehindStormCloudLogic` | `StormCloudLogic*` | 8 | StormCloudLogic back layer; setLogic `BehindStormCloudLogic` -> StormCloudLogic on IslandDisplayGroup/BehindStormCloud (setFrontStormCloud:NO posVelScale:0.8) |
| 0x4cd8 | `mWeatherButtonLogic` | `BCButtonLogic*` | 4 | BCButtonLogic #608 (GlobalIslandMenuDisplayGroup/WeatherButton) "Storm Clouds" |

### 8.9 `god_powers` (54)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x4230 | `mFireworkDisplayMarker` | `BCDisplayMarker*` | 3 | marker `FireworksDisplayMarker` |
| 0x4380 | `mFireworkLogicArray` | `NSMutableArray*` | 4 | live FireworkLogic objects (fireworks:::) |
| 0x4420 | `mUnderwaterPowerTrailLogicArray` | `NSMutableArray*` | 5 | MultiModelLogic trail puffs of the underwater power (removed on "UnderwaterPowerTrailComplete") |
| 0x4650 | `mTapTouchCount` | `int` | 2 | drum taps counted by checkForTapping: (reset after 1 s idle) |
| 0x4654 | `mTapTouchTime` | `float` | 2 | seconds since last drum tap (onGameFrame: accumulates) |
| 0x4658 | `mbCelebrationActive` | `BOOL` | 5 | pygmies dancing (drum/rainbow/story); isDanceActive / isRainbowActive return it |
| 0x4660 | `mCelebrationSequence` | `BCSequence*` | 4 | BCSequence "Celebration" (new object per startPygmiesDancing:::) |
| 0x4668 | `mFireworksStallTime` | `float` | 1 | fireworks::: wait timer |
| 0x466c | `mbThrillerActive` | `BOOL` | 3 | zombie Thriller dance music playing |
| 0x466d | `mbPantsOnTheGroundActive` | `BOOL` | 3 | Pants on the Ground dance song playing |
| 0x466e | `mbUpdateSongActive` | `BOOL` | 3 | Pocket God Update Song playing |
| 0x466f | `mbThemeSongActive` | `BOOL` | 3 | Pygmy Theme Song playing |
| 0x4670 | `mbFishmasActive` | `BOOL` | 3 | Fishmas Dance song playing |
| 0x4671 | `mbFishmasTwelveDaysActive` | `BOOL` | 3 | 12 Days of Fishmas song playing |
| 0x4690 | `mbEruption` | `BOOL` | 4 | volcano erupted (3rd hit) until onEruptionComplete: |
| 0x4694 | `mVolcanoHitCount` | `int` | 3 | pygmies thrown into the volcano (0-2 -> hit1..3/erupt1..3) |
| 0x4698 | `mVolcanoLogic` | `VolcanoLogic*` | 5 | volcano: hit1-3 / erupt1-3 on pygmies thrown in; setLogic `VolcanoLogic` -> VolcanoLogic on IslandDisplayGroup/Volcano |
| 0x46a4 | `mEarthquakeCount` | `int` | 4 | pygmies currently causing an earthquake |
| 0x46a8 | `mEarthquakeSequence` | `BCSequence*` | 12 | BCSequence "EarthquakeStart"/"TheRunsShakeBackgroundStart" (nil when none) |
| 0x46b8 | `mShakeStallTime` | `float` | 1 | shakeBackground::: remaining time (-1000 = endless) |
| 0x46bc | `mShakeRotVel` | `float` | 2 | shake phase increment per frame (1.3, set in loadGame) |
| 0x46c0 | `mShakeRot` | `float` | 2 | shake phase (y = 2*sin(phase) - 2) |
| 0x4768 | `mHurricaneTouch` | `BCTouch*` | 6 | finger steering the hurricane |
| 0x4770 | `mLastHurricanePos` | `VECTOR4` | 1 | hurricane start position (write-only) |
| 0x4780 | `mLastHurricaneTouchTime` | `float` | 1 | touch gameTime at hurricane start (write-only) |
| 0x4788 | `mpHurricaneList` | `std::list<VECTOR4>*` | 0 | unused std::list<VECTOR4>* |
| 0x4790 | `mHurricaneLogic` | `HurricaneLogic*` | 11 | HurricaneLogic on HurricaneModel "Hurricane" (nil when none) |
| 0x47b8 | `mWhirlPoolTouch` | `BCTouch*` | 5 | finger steering the whirlpool (underwater) |
| 0x47c0 | `mpWhirlPoolList` | `std::list<VECTOR4>*` | 0 | unused std::list<VECTOR4>* |
| 0x47c8 | `mWhirlPoolLogic` | `WhirlPoolLogic*` | 5 | WhirlPoolLogic on WhirlPoolModel "WhirlPool" (nil when none) |
| 0x47d0 | `mUnderwaterPowerTouch` | `BCTouch*` | 5 | finger drawing the coral-reef underwater power |
| 0x47d8 | `mLastUnderwaterTempleMode` | `int` | 2 | mode the power trail currently shows (0 = none) |
| 0x47dc | `mLastUnderwaterPowerPos` | `VECTOR4` | 3 | last trail-puff position (puffs every > 30 units) |
| 0x47ec | `mbShockObject` | `BOOL` | 3 | lightning stroke hit an object (bolt already fired) |
| 0x47ed | `mbDrawLightning` | `BOOL` | 6 | lightning stroke in progress |
| 0x47f0 | `mLightningTouch` | `BCTouch*` | 5 | finger drawing lightning |
| 0x47f8 | `mLastLightningPos` | `VECTOR4` | 3 | last lightning path point |
| 0x4808 | `mpLightningList` | `std::list<VECTOR4>*` | 7 | std::list<VECTOR4>* lightning path (shared with LightningModel) |
| 0x4810 | `mLightningLogic` | `LightningLogic*` | 5 | LightningLogic of the current bolt (nil when none) |
| 0x4818 | `mLightningTrailLogic` | `FingerTrailLogic*` | 4 | FingerTrailLogic "ElectricGlow" under the finger |
| 0x4820 | `mHurricaneTrailLogic` | `FingerTrailLogic*` | 5 | FingerTrailLogic "WindGlow" |
| 0x4828 | `mWhirlPoolTrailLogic` | `FingerTrailLogic*` | 4 | FingerTrailLogic "WindGlow" (underwater) |
| 0x4830 | `mUnderwaterPowerTrailLogic` | `FingerTrailLogic*` | 6 | FingerTrailLogic on "UnderwaterPower" model |
| 0x4838 | `mbDrawRainbow` | `BOOL` | 5 | rainbow stroke started (moved > 20) |
| 0x4840 | `mRainbowTouch` | `BCTouch*` | 3 | finger drawing the rainbow |
| 0x4848 | `mLastRainbowPos` | `VECTOR4` | 3 | last rainbow path point |
| 0x4858 | `mpRainbowList` | `std::list<VECTOR4>*` | 5 | std::list<VECTOR4>* rainbow path (shared with RainbowModel) |
| 0x4860 | `mRainbowLogic` | `RainbowLogic*` | 10 | RainbowLogic on RainbowModel "Rainbow" (created in loadGame) |
| 0x4868 | `mRainbowTrailLogic` | `FingerTrailLogic*` | 3 | FingerTrailLogic "RainbowTrail" |
| 0x4870 | `mbDrawLocust` | `BOOL` | 5 | locust stroke in progress |
| 0x4878 | `mLocustTouch` | `BCTouch*` | 3 | finger drawing the locust swarm |
| 0x4880 | `mLastLocustPos` | `VECTOR4` | 3 | last locust path point |
| 0x4890 | `mpLocustVector` | `std::vector<VECTOR4>*` | 5 | std::vector<VECTOR4>* locust path |
| 0x4898 | `mLocustLogic` | `LocustLogic*` | 11 | LocustLogic on LocustModel "Locust" (created in loadGame) |

### 8.10 `pygmies` (7)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x4378 | `mPygmyLogicArray` | `NSMutableArray*` | 31 | all live PygmyLogic objects (order matters: onPygmyTumble: moves a pygmy to index 0) |
| 0x4428 | `mTomPygmyLogic` | `PygmyLogic*` | 4 | the Tom (guest) pygmy while alive (createTomPygmy); nil when dead |
| 0x4678 | `mPygmyCount` | `int` | 2 | pygmies created (never decremented) |
| 0x467c | `mLivePygmyCount` | `int` | 6 | live pygmies (createPygmyLogic +1, onPygmyDie:sacrificeId: -1); cap 6 (+1 Charlie, +1 Tom) |
| 0x46a0 | `mSleepingCount` | `int` | 5 | number of sleeping pygmies (AllPygmiesSleepingStart/Stop on 0<->1) |
| 0x4748 | `mDraggedGhostPygmyLogic` | `PygmyLogic*` | 3 | ghost pygmy being dragged (onPygmyGhostDrag:) |
| 0x4750 | `mDraggedPygmyLogic` | `PygmyLogic*` | 7 | pygmy being dragged (onPygmyDrag:); drives drag auto-scroll in onGameFrame: |

### 8.11 `world_object_events` (28)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x43a0 | `mPiranhaLogicArray` | `NSMutableArray*` | 6 | live piranhas (onNewPiranhaLogic:/onRemovePiranhaLogic:/onRemoveAllPiranhaLogics:) |
| 0x4440 | `mFishingBobLogic` | `FishingBobLogic*` | 4 | FishingBobLogic of the cast line (onFishingPoleCreateBob:/onFishingPoleDestroyBob:) |
| 0x47b0 | `mbBaitMaster` | `BOOL` | 4 | Bait Master (piranha) game running |
| 0x48a0 | `mDraggedSwordFishLogic` | `SwordFishLogic*` | 3 | SwordFish being dragged (onSwordFishDrag: sets, onSwordFishDragComplete: clears); blocks canTouchPygmy |
| 0x48a8 | `mDraggedFishLogic` | `FishLogic*` | 3 | Fish being dragged (onFishDrag: sets, onFishDragComplete: clears); blocks canTouchPygmy |
| 0x48b0 | `mDraggedDanceBalloonLogic` | `DanceBalloonLogic*` | 3 | DanceBalloon being dragged (onDanceBalloonDrag: sets, onDanceBalloonDragComplete: clears); blocks canTouchPygmy |
| 0x48b8 | `mDraggedBananaLogic` | `BananaLogic*` | 3 | Banana being dragged (onBananaDrag: sets, onBananaDragComplete: clears); blocks canTouchPygmy |
| 0x48c0 | `mDraggedMonkeyLogic` | `MonkeyLogic*` | 3 | Monkey being dragged (onMonkeyDrag: sets, onMonkeyDragComplete: clears); blocks canTouchPygmy |
| 0x48c8 | `mDraggedMoronSnakeLogic` | `MoronSnakeLogic*` | 3 | MoronSnake being dragged (onMoronSnakeDrag: sets, onMoronSnakeDragComplete: clears); blocks canTouchPygmy |
| 0x48d0 | `mDraggedJellyfishLogic` | `JellyfishLogic*` | 3 | Jellyfish being dragged (onJellyfishDrag: sets, onJellyfishDragComplete: clears); blocks canTouchPygmy |
| 0x48d8 | `mDraggedGodIdolLogic` | `GodIdolLogic*` | 3 | GodIdol being dragged (onGodIdolDrag: sets, onGodIdolDragComplete: clears); blocks canTouchPygmy |
| 0x48e0 | `mDraggedApocalypseGemLogic` | `ApocalypseGemLogic*` | 3 | ApocalypseGem being dragged (onApocalypseGemDrag: sets, onApocalypseGemDragComplete: clears); drives the drag auto-scroll in onGameFrame: (does not block canTouchPygmy) |
| 0x48e8 | `mDraggedSpearLogic` | `SpearLogic*` | 3 | Spear being dragged (onSpearDrag: sets, onSpearDragComplete: clears); blocks canTouchPygmy |
| 0x48f0 | `mDraggedFishingBobLogic` | `FishingBobLogic*` | 2 | FishingBob being dragged (onFishingBobDrag: sets, onFishingBobDragComplete: clears); never read otherwise |
| 0x48f8 | `mDraggedFishingPoleLogic` | `FishingPoleLogic*` | 3 | FishingPole being dragged (onFishingPoleDrag: sets, onFishingPoleDragComplete: clears); blocks canTouchPygmy |
| 0x4900 | `mDraggedWaterWingsLogic` | `WaterWingsLogic*` | 3 | WaterWings being dragged (onWaterWingsDrag: sets, onWaterWingsDragComplete: clears); blocks canTouchPygmy |
| 0x4908 | `mDraggedCoconutLogic` | `CoconutLogic*` | 3 | Coconut being dragged (onCoconutDrag: sets, onCoconutDragComplete: clears); blocks canTouchPygmy |
| 0x4910 | `mDraggedWoodLogic` | `WoodLogic*` | 3 | Wood being dragged (onWoodDrag: sets, onWoodDragComplete: clears); blocks canTouchPygmy |
| 0x4918 | `mDraggedAnchorLogic` | `AnchorLogic*` | 3 | Anchor being dragged (onAnchorDrag: sets, onAnchorDragComplete: clears); blocks canTouchPygmy |
| 0x4920 | `mDraggedTilePlugLogic` | `TilePlugLogic*` | 3 | TilePlug being dragged (onTilePlugDrag: sets, onTilePlugDragComplete: clears); blocks canTouchPygmy |
| 0x4928 | `mDraggedBoulderLogic` | `BoulderLogic*` | 4 | Boulder being dragged (onBoulderDrag: sets, onBoulderDragComplete: clears); blocks canTouchPygmy |
| 0x4930 | `mDraggedTRexEggLogic` | `TRexEggLogic*` | 3 | TRexEgg being dragged (onTRexEggDrag: sets, onTRexEggDragComplete: clears); blocks canTouchPygmy |
| 0x4938 | `mDirectedAntHillLogic` | `AntHillLogic*` | 3 | ant hill whose ants are being directed (onAntHillDirect: sets, onAntHillDirectComplete: clears); blocks canTouchPygmy |
| 0x4940 | `mDraggedMagnifyingGlassLogic` | `MagnifyingGlassLogic*` | 3 | MagnifyingGlass being dragged (onMagnifyingGlassDrag: sets, onMagnifyingGlassDragComplete: clears); blocks canTouchPygmy |
| 0x4948 | `mDraggedSharkLogic` | `SharkLogic*` | 3 | Shark being dragged (onSharkDrag: sets, onSharkDragComplete: clears); blocks canTouchPygmy |
| 0x4950 | `mDraggedBirdLogic` | `BirdLogic*` | 3 | Bird being dragged (onBirdDrag: sets, onBirdDragComplete: clears); blocks canTouchPygmy |
| 0x4958 | `mDraggedClosetTempleClosetDoorLogic` | `ClosetTempleClosetDoorLogic*` | 3 | ClosetTempleClosetDoor being dragged (onClosetTempleClosetDoorDrag: sets, onClosetTempleClosetDoorDragComplete: clears); blocks canTouchPygmy |
| 0x4960 | `mDraggedThetanLogic` | `ThetanLogic*` | 3 | Thetan being dragged (onThetanDrag: sets, onThetanDragComplete: clears); blocks canTouchPygmy |

### 8.12 `minigames` (54)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x4100 | `mDoodleDisplayGroup` | `BCDisplayGroup*` | 5 | group `DoodleDisplayGroup` |
| 0x4108 | `mPainDrainDisplayGroup` | `BCDisplayGroup*` | 3 | group `PainDrainDisplayGroup` |
| 0x4110 | `mTheRunsDisplayGroup` | `BCDisplayGroup*` | 7 | group `TheRunsDisplayGroup` |
| 0x4118 | `mKonkeyDongDisplayGroup` | `BCDisplayGroup*` | 5 | group `KonkeyDongDisplayGroup` |
| 0x4120 | `mMoronPestsDisplayGroup` | `BCDisplayGroup*` | 5 | group `MoronPestsDisplayGroup` |
| 0x4128 | `mNoPeeDisplayGroup` | `BCDisplayGroup*` | 5 | group `NoPeeDisplayGroup` |
| 0x4130 | `mBattleOfTheGodsDisplayGroup` | `BCDisplayGroup*` | 4 | group `BattleArenaDisplayGroup` |
| 0x4268 | `mDoodleCloudDisplayMarker` | `BCDisplayMarker*` | 2 | marker `DoodleCloudDisplayMarker` |
| 0x4270 | `mDoodlePygmyDisplayMarker` | `BCDisplayMarker*` | 1 | marker `DoodlePygmyDisplayMarker` |
| 0x4278 | `mPainDrainPlatformDisplayMarker` | `BCDisplayMarker*` | 1 | marker `PainDrainPlatformDisplayMarker` |
| 0x4280 | `mPainDrainPygmyDisplayMarker` | `BCDisplayMarker*` | 1 | marker `PainDrainPygmyDisplayMarker` |
| 0x4288 | `mTheRunsPygmyDisplayMarker` | `BCDisplayMarker*` | 1 | marker `TheRunsPygmyDisplayMarker` |
| 0x4290 | `mKonkeyDongPygmyDisplayMarker` | `BCDisplayMarker*` | 1 | marker `KonkeyDongPygmyDisplayMarker` |
| 0x4298 | `mMoronPestsPygmyDisplayMarker` | `BCDisplayMarker*` | 1 | marker `MoronPestsPygmyDisplayMarker` |
| 0x42a0 | `mNoPeePygmyDisplayMarker` | `BCDisplayMarker*` | 1 | marker `NoPeePygmyDisplayMarker` |
| 0x42a8 | `mBattleOfTheGodsPygmyDisplayMarker` | `BCDisplayMarker*` | 1 | marker `BattleOfTheGodsPygmyDisplayMarker` |
| 0x4628 | `mTheRunsSequence` | `BCSequence*` | 5 | BCSequence for The Runs (StartTheRuns/StartTheRunsLoop/StopTheRuns) |
| 0x4630 | `mKonkeyDongSequence` | `BCSequence*` | 5 | BCSequence for Konkey Dong |
| 0x4638 | `mMoronPestsSequence` | `BCSequence*` | 5 | BCSequence for Moron Pests |
| 0x4640 | `mNoPeeSequence` | `BCSequence*` | 4 | BCSequence for No Pee |
| 0x4648 | `mBattleOfTheGodsSequence` | `BCSequence*` | 4 | BCSequence for Battle of the Gods |
| 0x46c8 | `mDoodleJumpPygmyLogic` | `PygmyLogic*` | 10 | pygmy doing Ooga Jump (nil otherwise) |
| 0x46d0 | `mDoodleCloudLogicArray` | `NSMutableArray*` | 6 | 50 DoodleCloudLogic platforms while Ooga Jump runs |
| 0x46d8 | `mbDoodleJumpFall` | `BOOL` | 4 | Ooga Jump pygmy is falling back (camera follows down) |
| 0x46d9 | `mbDoodleDebugPress` | `BOOL` | 3 | write-only: set when a finger goes down during Ooga Jump, cleared on touch end and on stop (debug leftover) |
| 0x46da | `mbShowDoodleAssets` | `BOOL` | 4 | Ooga Jump atlas wanted by loadLocationTextureAtlas |
| 0x46e0 | `mPainDrainPygmyLogic` | `PygmyLogic*` | 4 | pygmy in Pain Drain |
| 0x46e8 | `mbShowPainDrainAssets` | `BOOL` | 3 | Pain Drain atlas wanted |
| 0x46e9 | `mbTheRuns` | `BOOL` | 7 | The Runs active |
| 0x46ea | `mbTheRunsStarting` | `BOOL` | 1 | cleared on stop (never set) |
| 0x46eb | `mbTheRunsMoveCamera` | `BOOL` | 5 | The Runs camera scroll on |
| 0x46f0 | `mTheRunsPygmyLogic` | `PygmyLogic*` | 5 | pygmy in The Runs |
| 0x46f8 | `mbShowTheRunsAssets` | `BOOL` | 3 | The Runs atlas wanted |
| 0x46f9 | `mbKonkeyDong` | `BOOL` | 6 | Konkey Dong active |
| 0x46fa | `mbKonkeyDongStarting` | `BOOL` | 1 | cleared on stop (never set) |
| 0x46fb | `mbKonkeyDongMoveCamera` | `BOOL` | 5 | Konkey Dong camera follow on |
| 0x46fc | `mbKonkeyDongMoveCameraToNextLevel` | `BOOL` | 3 | camera easing to the next level |
| 0x4700 | `mKonkeyDongLevelYPos` | `float` | 3 | target camera level y (set by onKonkeyDongMoveCameraToNextLevel:) |
| 0x4708 | `mKonkeyDongPygmyLogic` | `PygmyLogic*` | 5 | pygmy in Konkey Dong |
| 0x4710 | `mbShowKonkeyDongAssets` | `BOOL` | 3 | Konkey Dong atlas wanted |
| 0x4711 | `mbMoronPests` | `BOOL` | 6 | Moron Pests active |
| 0x4712 | `mbMoronPestsMoveCamera` | `BOOL` | 6 | Moron Pests camera scroll on |
| 0x4718 | `mMoronPestsPygmyLogic` | `PygmyLogic*` | 5 | pygmy in Moron Pests |
| 0x4720 | `mbShowMoronPestsAssets` | `BOOL` | 3 | Moron Pests atlas wanted |
| 0x4721 | `mbNoPee` | `BOOL` | 7 | No Pee active |
| 0x4722 | `mbNoPeeStarting` | `BOOL` | 2 | cleared on stop/retry (never set) |
| 0x4723 | `mbNoPeeMoveCamera` | `BOOL` | 6 | No Pee camera follow on |
| 0x4728 | `mNoPeePygmyLogic` | `PygmyLogic*` | 5 | pygmy in No Pee |
| 0x4730 | `mbShowNoPeeAssets` | `BOOL` | 3 | No Pee atlas wanted |
| 0x4731 | `mbBattleOfTheGods` | `BOOL` | 4 | Battle of the Gods active |
| 0x4732 | `mbShowBattleOfTheGodsAssets` | `BOOL` | 3 | battle atlas wanted |
| 0x4738 | `mBattleOfTheGodsPygmyLogic` | `PygmyLogic*` | 4 | pygmy sent to Battle of the Gods |
| 0x4740 | `mbDanceDanceExecution` | `BOOL` | 5 | Dance Dance Execution active |
| 0x47b1 | `mbPainDrain` | `BOOL` | 3 | Pain Drain active |

### 8.13 `apocalypse` (2)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x43d8 | `mApocalypseGemLogicArray` | `NSMutableArray*` | 8 | activated apocalypse gems (onApocalypseGemEventTriggered: adds, onApocalypseGemDie: removes) |
| 0x4672 | `mbApocalypseActive` | `BOOL` | 9 | apocalypse running (startApocalypse/endApocalypse); switches background loop and gradients |

### 8.14 `stats_and_achievements` (34)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x150 | `mGameServer` | `GameServer*` | 6 | GameServer (Game Center): leaderboards/achievements; listener = self |
| 0x18d | `mbCheckAllAchievementsAndLeaderboards` | `BOOL` | 2 | deferred request: checkAllAchievementsAndLeaderboards ran before mGodPersona existed; replayed at the end of loadPreferences::: |
| 0x218 | `mGodPersona` | `GodPersona*` | 10 | GodPersona "God" (default name "GENE SIMIANS", description "God of Thunder and Rocks that Roll") |
| 0x220 | `mTribePersona` | `TribePersona*` | 12 | TribePersona "Tribe" ("TRIBE OOKGA CHAKA", "One Tribe, One World"); owns Pygmy0-5 personas + guests |
| 0x228 | `mCharliePygmyPersona` | `PygmyPersona*` | 2 | guest PygmyPersona "CharliePygmy" ("Charlie", "Winning!.", personality 1) |
| 0x230 | `mTomPygmyPersona` | `PygmyPersona*` | 2 | guest PygmyPersona "TomPygmy" ("Tom", "Show me the money!.", personality 2) |
| 0x238 | `mTotalGifts` | `int` | 1 | cache of tribe totalGifts (last submitted to pg.totalgifts) |
| 0x23c | `mTotalSacrifices` | `int` | 1 | cache of tribe totalSacrifices (last submitted to pg.totalsacrifices) |
| 0x240 | `mbSubmittingScore` | `BOOL` | 0 | unused |
| 0x241 | `mbSubmitSacrificesWhenComplete` | `BOOL` | 0 | unused |
| 0x242 | `mbSubmitGiftsWhenComplete` | `BOOL` | 0 | unused |
| 0x4360 | `mStatsTribeUiLogicArray` | `NSMutableArray*` | 3 | tribe stats page buttons (incl. the 6 pygmy buttons) |
| 0x4368 | `mStatsPygmyUiLogicArray` | `NSMutableArray*` | 3 | pygmy stats page buttons |
| 0x4370 | `mStatsGodUiLogicArray` | `NSMutableArray*` | 3 | god stats page buttons |
| 0x4dec | `mActiveStatsScreen` | `int` | 1 | 1 tribe, 2 pygmy, 3 god |
| 0x4df0 | `mStatsTribeCloseButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsTribeCloseButtonLogic` -> BCButtonLogic on StatsPageTribe/CloseButton (StatsTribe UI) |
| 0x4df8 | `mStatsTribeEditNameButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsTribeEditNameButtonLogic` -> BCButtonLogic on StatsPageTribe/TribeName (StatsTribe UI) |
| 0x4e00 | `mStatsTribeCancelEditNameButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsTribeCancelEditNameButtonLogic` -> BCButtonLogic on StatsPageTribe/CancelEditNameButton (StatsTribe UI) |
| 0x4e08 | `mGodStatsButtonLogic` | `BCButtonLogic*` | 2 | setLogic `GodStatsButtonLogic` -> BCButtonLogic on StatsPageTribe/GodStatsButton (StatsTribe UI) |
| 0x4e10 | `mPygmyStatsButtonLogicArray` | `NSMutableArray*` | 4 | setLogic `PygmyStatsButtonLogic` -> BCButtonLogic x count on StatsPageTribe/Pygmy%dStatsButton (count 6: Pygmy0..5) (StatsTribe UI) |
| 0x4e18 | `mStatsPygmyCloseButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsPygmyCloseButtonLogic` -> BCButtonLogic on StatsPagePygmy/CloseButton (StatsPygmy UI) |
| 0x4e20 | `mStatsPygmyNextButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsPygmyNextButtonLogic` -> BCButtonLogic on StatsPagePygmy/NextButton (StatsPygmy UI) |
| 0x4e28 | `mStatsPygmyEditNameButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsPygmyEditNameButtonLogic` -> BCButtonLogic on StatsPagePygmy/PygmyName (StatsPygmy UI) |
| 0x4e30 | `mStatsPygmyEditDescriptionButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsPygmyEditDescriptionButtonLogic` -> BCButtonLogic on StatsPagePygmy/PygmyDescription (StatsPygmy UI) |
| 0x4e38 | `mStatsPygmyCancelEditNameButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsPygmyCancelEditNameButtonLogic` -> BCButtonLogic on StatsPagePygmy/CancelEditNameButton (StatsPygmy UI) |
| 0x4e40 | `mStatsPygmyCancelEditDescriptionButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsPygmyCancelEditDescriptionButtonLogic` -> BCButtonLogic on StatsPagePygmy/CancelEditDescriptionButton (StatsPygmy UI) |
| 0x4e48 | `mActivePygmyPersona` | `PygmyPersona*` | 3 | PygmyPersona shown on the pygmy stats page |
| 0x4e50 | `mPygmyStatsType` | `int` | 3 | pygmy stats list: 0 gifts, 1 sacrifices |
| 0x4e58 | `mStatsGodCloseButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsGodCloseButtonLogic` -> BCButtonLogic on StatsPageGod/CloseButton (StatsGod UI) |
| 0x4e60 | `mStatsGodNextGodTypeButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsGodNextGodTypeButtonLogic` -> BCButtonLogic on StatsPageGod/GodTypePic (StatsGod UI) |
| 0x4e68 | `mStatsGodEditNameButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsGodEditNameButtonLogic` -> BCButtonLogic on StatsPageGod/GodName (StatsGod UI) |
| 0x4e70 | `mStatsGodEditDescriptionButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsGodEditDescriptionButtonLogic` -> BCButtonLogic on StatsPageGod/GodDescription (StatsGod UI) |
| 0x4e78 | `mStatsGodCancelEditNameButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsGodCancelEditNameButtonLogic` -> BCButtonLogic on StatsPageGod/CancelEditNameButton (StatsGod UI) |
| 0x4e80 | `mStatsGodCancelEditDescriptionButtonLogic` | `BCButtonLogic*` | 2 | setLogic `StatsGodCancelEditDescriptionButtonLogic` -> BCButtonLogic on StatsPageGod/CancelEditDescriptionButton (StatsGod UI) |

### 8.15 `managers_bridge` (23)

| Offset | Ivar | Type | Refs | Meaning |
|---|---|---|---|---|
| 0x158 | `mChallengeManager` | `ChallengeManager*` | 47 | ChallengeManager: challenge screens, timed challenge games (game ids 0-15), achievements |
| 0x190 | `mSkinPackManager` | `SkinPackManager*` | 18 | SkinPackManager (store: skin packs, skin options, texture atlas channels) |
| 0x198 | `mBonusPackManager` | `BonusPackManager*` | 5 | BonusPackManager (bonus pack screen/unlocks) |
| 0x1a0 | `mStoryManager` | `StoryManager*` | 5 | StoryManager (story list/editor/player); loadStoryDefs at boot |
| 0x1a8 | `mDebugScreen` | `DebugScreen*` | 4 | DebugScreen (unreachable: mDebugButtonLogic is never set) |
| 0x1b0 | `mNewsManager` | `NewsManager*` | 10 | NewsManager (news teaser/screen); loadNewsDefs at boot and on foreground |
| 0x1b8 | `mIdolManager` | `IdolManager*` | 19 | IdolManager (idols, episodes/challenges, Battle of the Gods screens, episode events) |
| 0x1c0 | `mGameUiManager` | `GameUiManager*` | 2 | GameUiManager (created, listener = self; protocol declares no methods) |
| 0x1c8 | `mMapScreen` | `MapScreen*` | 4 | MapScreen (island map / travel) |
| 0x4db1 | `mbSkinPackScreenOpen` | `BOOL` | 7 | skin pack store open |
| 0x4db2 | `mbBonusPackScreenOpen` | `BOOL` | 8 | bonus pack screen/dialog open |
| 0x4db3 | `mbMapScreenOpen` | `BOOL` | 6 | map open |
| 0x4db4 | `mbStoryScreenOpen` | `BOOL` | 6 | story screen open |
| 0x4db5 | `mbStoryActive` | `BOOL` | 5 | a story is playing |
| 0x4db8 | `mStorySaveLabelMode` | `int` | 2 | DisplayNames option saved while a story plays (forced to 1) |
| 0x4dbc | `mbNewsTeaserOpen` | `BOOL` | 6 | news teaser popup open |
| 0x4dbd | `mbNewsScreenOpen` | `BOOL` | 10 | news screen open |
| 0x4dbe | `mbIdolScreenOpen` | `BOOL` | 14 | any IdolManager screen open |
| 0x4dbf | `mbIdolAwardScreenOpen` | `BOOL` | 5 | idol award dialog open |
| 0x4dc0 | `mbIdolBattleResultsScreenOpen` | `BOOL` | 2 | battle results screen open (reload TextureAtlas0 on close) |
| 0x4dc2 | `mbChallengeScreenOpen` | `BOOL` | 8 | challenge screen open |
| 0x4dc3 | `mbChallengeDialogOpen` | `BOOL` | 3 | challenge dialog open |
| 0x4dc4 | `mbChallengeActive` | `BOOL` | 3 | a challenge is running |

## 9. Partition into port units

The 692 methods (354 instance + `+instance` of the main class, 321 of `Game`, 16 of `PocketGodAppDelegate`) are split into 15 units, each a candidate module/crate area of the port with one Bevy resource for its owned ivars and systems/functions for its methods. Every method is in exactly one unit and every ivar is owned by exactly one unit. The machine-readable form is `tmp/re/controller-partition.json`:

```text
[{
  "unit": "snake_case name (order below)",
  "description": "what the unit does",
  "methods": ["Class(Category)::selector @ 0xADDRESS", ...],   // ascending address
  "ivars": ["mName", ...],          // owned ivars, ascending offset (section 8)
  "ivars_shared": ["mName", ...],   // ivars of other units this unit's methods touch
  "bytes": 12345,                   // decompiled C bytes of its methods
  "depends_on": ["unit", ...]       // units whose methods or ivars it uses
}]
```

`depends_on` comes from the call graph (`[self sel]` sends between controller methods) and ivar use, ignoring `init`, `loadGame`, `dealloc` and `.cxx_construct` (which touch everything once). The graph is dense and cyclic — the controller is a hub — so the units are not layers; a port should put the owned state of every unit in resources that any system can query, and keep the callbacks as functions of the owning unit. The weights in 9.2 (number of method→method or method→ivar references) show which edges are structural and which are incidental.

### 9.1 Units

| # | Unit | Methods | Owned ivars | Shared ivars used | Decompiled bytes | Summary |
|---|---|---|---|---|---|---|
| 1 | `app_platform` | 77 | 15 | 7 | 55,258 | iOS shell: app delegate, view and orientation, modal views, text entry, URLs, ads, sound pause helpers |
| 2 | `boot_and_frame` | 23 | 19 | 61 | 101,040 | `loadGame`, boot-script handlers, preferences and personas, `onGameFrame:` |
| 3 | `world_registry` | 95 | 84 | 54 | 68,865 | `setLogic:::` factory and the logic, array and marker accessors |
| 4 | `touch_input` | 23 | 24 | 56 | 46,973 | touch router and world gestures (pinch, ocean flick, crack, underwater swipe, wench) |
| 5 | `menus_and_screens` | 18 | 73 | 89 | 76,076 | button dispatcher, category menus and labels, help, message, screen gates |
| 6 | `island_locations` | 29 | 41 | 135 | 114,339 | the 18 locations and travel between them, location atlases |
| 7 | `island_features` | 95 | 68 | 47 | 92,955 | feature toggles, their options and the `is…Enabled` predicates |
| 8 | `sky_weather` | 32 | 55 | 17 | 54,594 | sun, moon, day/sunset/night, sky gradients, eclipse, storm, clouds, bat |
| 9 | `god_powers` | 51 | 54 | 26 | 85,367 | lightning, rainbow, locusts, hurricane, whirlpool, coral power, drum, earthquake, volcano, fireworks |
| 10 | `pygmies` | 21 | 7 | 54 | 21,975 | pygmy creation and limits, drag/death bookkeeping, sleeping |
| 11 | `world_object_events` | 92 | 28 | 8 | 33,235 | world-object callbacks (drags, coconut, fishing, shark, bird, bosses, spears, piranhas) and challenge scoring |
| 12 | `minigames` | 60 | 54 | 12 | 44,918 | Ooga Jump, Pain Drain, The Runs, Konkey Dong, Moron Pests, No Pee, DDE, Battle of the Gods |
| 13 | `apocalypse` | 10 | 2 | 18 | 18,954 | apocalypse start/end, gems, plague check, temple groups |
| 14 | `stats_and_achievements` | 22 | 34 | 6 | 124,653 | stats pages, text tables, sacrifices and gifts, Game Center |
| 15 | `managers_bridge` | 44 | 23 | 18 | 29,052 | manager screens and callbacks: store, bonus packs, story, news, idols, challenges, debug |
| | **total** | 692 | 581 | | 968,254 | |

### 9.2 Units in detail

#### 9.2.1 `app_platform`

UIKit/iOS shell and platform services: PocketGodAppDelegate (launch, background/foreground, loading image), controller init/view/orientation/dealloc, modal view controllers, keyboard + hidden UITextField text entry, URL opening, Chartboost ads, Game Center turn stubs, background-loop and paused-sound helpers.

- Depends on: `touch_input` (1), `god_powers` (4), `pygmies` (2), `apocalypse` (2), `stats_and_achievements` (3).
- Owned ivars: 15 (section 8.1); ivars of other units used: 7.
- `PocketGodViewController.c` (50): `instance` 0x100069cdc, `init` 0x100069ce8, `presentModalViewController:` 0x10006d2dc, `dismissModalViewController` 0x10006d388, `hideKeyboard` 0x10006d3f8, `showKeyboard` 0x10006d444, `textDidChange:` 0x10006d49c, `textField:shouldChangeCharactersInRange:replacementString:` 0x10006d4f0, `textFieldShouldBeginEditing:` 0x10006d5a4, `textFieldDidBeginEditing:` 0x10006d5ac, `textFieldShouldEndEditing:` 0x10006d5b0, `textFieldDidEndEditing:` 0x10006d5b8, `textFieldShouldClear:` 0x10006d6b0, `textFieldShouldReturn:` 0x10006d6b8, `onShowKeyboard:editingText:cancelEditDisplayObject:maxEditCharCount:preventEmpty:` 0x10006d84c, `onHideKeyboard` 0x10006d994, `isKeyboardOpen` 0x10006da88, `loadView` 0x10006dad8, `supportedInterfaceOrientations` 0x10006daf4, `preferredInterfaceOrientationForPresentation` 0x10006dafc, `shouldAutorotate` 0x10006db04, `shouldAutorotateToInterfaceOrientation:` 0x10006db0c, `didReceiveMemoryWarning` 0x10006db18, `dealloc` 0x10006db88, `setDelegate:` 0x10006e648, `id` 0x10006e6a4, `openCountryURL:` 0x10006e7b8, `openLaunchUrl` 0x10006e93c, `openUrl:` 0x10006e9d4, `handleTurnEventForActiveMatch:` 0x100070a10, `handleMatchEndedForActiveMatch:` 0x100070a18, `handleTurnEventNotificationForInactiveMatch:` 0x100070a20, `handleMatchEndedNotificationForInactiveMatch:` 0x100070a28, `playBackgroundLoop:` 0x100070ee8, `playBackgroundLoop:::` 0x10007104c, `stopBackgroundLoop:::` 0x100071078, `pauseHurricaneSounds:::` 0x10007109c, `resumeHurricaneSounds:::` 0x1000710c0, `showChartboostAd` 0x100088244, `isChartboostAdVisible` 0x100088354, `shouldRequestInterstitial:` 0x100088364, `shouldDisplayInterstitial:` 0x100088390, `didDisplayInterstitial:` 0x1000883bc, `didCacheInterstitial:` 0x1000883e4, `didFailToLoadInterstitial:withError:` 0x10008840c, `didFailToRecordClick:withError:` 0x10008844c, `didDismissInterstitial:` 0x100088474, `didCloseInterstitial:` 0x1000884d8, `didClickInterstitial:` 0x10008853c, `.cxx_construct` 0x1000885a0.
- `PocketGodAppDelegate.c` (16): `application:didFinishLaunchingWithOptions:` 0x100089324, `showLoadingImage` 0x100089570, `applicationDidBecomeActive:` 0x100089a24, `applicationWillResignActive:` 0x100089a90, `applicationDidEnterBackground:` 0x100089afc, `applicationWillEnterForeground:` 0x100089b94, `applicationWillTerminate:` 0x100089c0c, `loadGame` 0x100089c10, `onHideLoadingScreen` 0x100089c2c, `onModalViewWillAppear` 0x100089c8c, `onModalViewWillDisappear` 0x100089cd0, `dealloc` 0x100089d10, `window` 0x100089d8c, `setWindow:` 0x100089d9c, `mPocketGodViewController` 0x100089db8, `setMPocketGodViewController:` 0x100089dc8.
- `PocketGodViewController+Game.c` (11): `pauseHurricaneSounds` 0x100168df4, `resumeHurricaneSounds` 0x100168e70, `stopBackgroundLoop` 0x100168eec, `pauseSleepingSounds` 0x100168f64, `resumeSleepingSounds` 0x100168fd0, `pauseSleepingSounds:::` 0x10016903c, `resumeSleepingSounds:::` 0x100169060, `pauseLocustSounds` 0x100169084, `resumeLocustSounds` 0x1001690fc, `pauseLocustSounds:::` 0x100169174, `resumeLocustSounds:::` 0x100169198.

#### 9.2.2 `boot_and_frame`

Data loading (loadGame), the PocketGod boot script handlers (preferences wait/load, start screen, loading screen, touch enable), option defaults and personas, onPreferencesChanged:, and the per-frame onGameFrame: (island scroll, minigame cameras, menu easing, then eclipse + gradients).

- Depends on: `app_platform` (5), `world_registry` (1), `touch_input` (1), `menus_and_screens` (15), `island_locations` (3), `island_features` (6), `sky_weather` (5), `god_powers` (3), `pygmies` (1), `world_object_events` (1), `minigames` (20), `stats_and_achievements` (10), `managers_bridge` (8).
- Owned ivars: 19 (section 8.2); ivars of other units used: 61.
- `PocketGodViewController.c` (13): `loadGame` 0x10006a030, `personaPreferences` 0x10006da98, `optionPreferences` 0x10006daa8, `systemPreferences` 0x10006dab8, `challengePreferences` 0x10006dac8, `godPersona` 0x100070db4, `tribePersona` 0x1000715e4, `startTime:::` 0x10007bc30, `stopTime:::` 0x10007bc50, `waitForHoldLoadingAd:::` 0x10007bca8, `waitForPreferencesLoadComplete:::` 0x10007bce8, `onPreferencesChanged:` 0x10007bd80, `loadPreferences:::` 0x10007bdcc.
- `PocketGodViewController+Game.c` (10): `refreshNews` 0x1001579f8, `gameFrameOrder` 0x1001585b4, `onGameFrame:` 0x1001585bc, `enableTouch:::` 0x100159998, `hideLoadingScreen` 0x1001599d0, `checkForIdolStartScreen` 0x1001599f0, `checkShowAuthenticationViewController:::` 0x100159c44, `checkStartScreen:::` 0x100159c80, `hideLoadingScreen:::` 0x100159d40, `checkForNewNews:::` 0x100159dc0.

#### 9.2.3 `world_registry`

The world-object registry: setLogic::: factory (logic id -> class, listener, ivar/array), every logic/array accessor the logic listeners call (xLogic:, xLogicArray:, spriteLogicArray:), and the display-marker getters for pygmies/locusts.

- Depends on: `touch_input` (2), `menus_and_screens` (7), `island_locations` (9), `sky_weather` (8), `god_powers` (3), `pygmies` (4), `world_object_events` (2), `minigames` (1), `apocalypse` (1), `stats_and_achievements` (20), `managers_bridge` (5).
- Owned ivars: 84 (section 8.3); ivars of other units used: 54.
- `PocketGodViewController.c` (85): `rainbowLogic:` 0x1000710e4, `locustLogic:` 0x1000710f4, `starsLogic:` 0x100071104, `tRexEggLogic:` 0x100071114, `tRexLogic:` 0x100071124, `boulderLogic:` 0x100071134, `spiderLogic:` 0x100071144, `spiderTreeLogic:` 0x100071154, `fishLogic:` 0x100071164, `swordFishLogic:` 0x100071174, `fishingBobLogic:` 0x100071184, `fishingPoleLogic:` 0x100071194, `waterWingsLogic:` 0x1000711a4, `sharkLogic:` 0x1000711b4, `birdLogic:` 0x1000711c4, `outHouseLogic:` 0x1000711d4, `iceMonsterLogic:` 0x1000711e4, `glacierLogic:` 0x1000711f4, `iglooLogic:` 0x100071204, `iceTreeLogic:` 0x100071214, `iceHoleLogic:` 0x100071224, `zombieGraveLogic:` 0x100071234, `apeLogic:` 0x100071244, `apeAltarLogic:` 0x100071254, `apeBongoLogic:` 0x100071264, `apeWenchLogic:` 0x100071274, `apeTorchLogicArray:` 0x100071284, `monkeyLogicArray:` 0x100071294, `jellyfishLogicArray:` 0x1000712a4, `idolPedestalLogicArray:` 0x1000712b4, `apocalypseGemPedestalLogicArray:` 0x1000712c4, `apocalypseGemLogicArray:` 0x1000712d4, `timeTempleDoorLogicArray:` 0x1000712e4, `locustTempleDoorLogicArray:` 0x1000712f4, `danceTempleDoorLogicArray:` 0x100071304, `plagueTempleDoorLogicArray:` 0x100071314, `purificationTempleDoorLogicArray:` 0x100071324, `closetTempleDoorLogicArray:` 0x100071334, `xenuTempleDoorLogicArray:` 0x100071344, `bananaTreeLogic:` 0x100071354, `bananaLogic:` 0x100071364, `moronSnakeLogic:` 0x100071374, `noPeeLogic:` 0x100071384, `templeDoorLogic:` 0x100071394, `godIdolLogicArray` 0x1000713a4, `apocalypseTempleDoorLogic:` 0x1000713b4, `apocalypsePortalTempleDoorLogic:` 0x1000713c4, `apocalypseTimeTempleLeverLogic:` 0x1000713d4, `apocalypseTimeTemplePlatformLogic:` 0x1000713e4, `locustQueenLogic:` 0x1000713f4, `goGoCageLogic:` 0x100071404, `discoBallLogic:` 0x100071414, `discoLightsLogic:` 0x100071424, `danceBalloonBundleLogic:` 0x100071434, `danceBalloonLogic:` 0x100071444, `danceDanceExecutionLogic:` 0x100071454, `apocalypsePlagueTempleFountainLogic:` 0x100071464, `apocalypsePurificationTempleFountainLogic:` 0x100071474, `closetTempleThetanMeterLogic:` 0x100071484, `closetTempleCouchLogic:` 0x100071494, `closetTempleClosetDoorLogic:` 0x1000714a4, `xenuTempleLogic:` 0x1000714b4, `xenuLogic:` 0x1000714c4, `crackHoleLogic:` 0x1000714d4, `octoStatueLogic:` 0x1000714e4, `kongStatueLogic:` 0x1000714f4, `spearStatueLogic:` 0x100071504, `anchorLogic:` 0x100071514, `tileDrainLogic:` 0x100071524, `tilePlugLogic:` 0x100071534, `chumBucketLogic:` 0x100071544, `healthBarLogic:` 0x100071554, `antHillLogic:` 0x100071564, `bonFireLogic:` 0x100071574, `coconutLogicArray:` 0x100071584, `woodLogicArray:` 0x100071594, `spearLogicArray:` 0x1000715a4, `underwaterSceneryLogicArray:` 0x1000715b4, `piranhaLogicArray:` 0x1000715c4, `pygmyLogicArray:` 0x1000715d4, `draggedPygmyLogic:` 0x1000715f4, `cloudLogicArray:` 0x100071604, `doodleCloudLogicArray:` 0x100071614, `spriteLogicArray:` 0x100071624, `mapPaperLogic` 0x100078fc8.
- `PocketGodViewController+Game.c` (10): `locustDisplayMarker` 0x10015ad2c, `pygmyDragDisplayMarker` 0x10015aeec, `pygmyDisplayMarker` 0x10015b0ac, `pygmyIslandDisplayMarker` 0x10015b26c, `pygmySmallObjectsDisplayMarker` 0x10015b42c, `pygmyUnderwaterPygmyDisplayMarker` 0x10015b5ec, `pygmyIslandBackDisplayMarker` 0x10015b628, `pygmyCloudBehindDisplayMarker` 0x10015b7e8, `tomPygmyLogic:` 0x10015b9c4, `setLogic:::` 0x10015c11c.

#### 9.2.4 `touch_input`

Touch router (touchBegan/Moved/Ended/Cancelled, cancelAllTouches, selectable-logic arrays) and the world gestures not owned by a god power: pinch-sun detection, ocean flick (go underwater), ground crack, underwater camera swipe (leave underwater), touch bubbles, ape wench crank.

- Depends on: `boot_and_frame` (1), `world_registry` (7), `menus_and_screens` (3), `island_locations` (40), `sky_weather` (19), `god_powers` (33), `minigames` (10), `managers_bridge` (7).
- Owned ivars: 24 (section 8.4); ivars of other units used: 56.
- `PocketGodViewController.c` (10): `retainTouchLogicArray:` 0x10006e6b0, `releaseTouchLogicArray:` 0x10006e6f0, `createTouchLogicArray:` 0x10006e730, `isPointLeftOfLineStartXPos:lineStartYPos:lineEndXPos:lineEndYPos:xPos:yPos:` 0x10006ea74, `preStartTouchBeganXPos:yPos:` 0x10006ea9c, `touchBegan:allTouches:` 0x10006eaa0, `touchMoved:allTouches:` 0x10006f7fc, `touchEnded:allTouches:` 0x1000701a0, `cancelAllTouches` 0x1000707c8, `touchCancelled:allTouches:` 0x100070a00.
- `PocketGodViewController+Game.c` (13): `startDragOcean:timeStamp:` 0x10015fd24, `stopDragOcean` 0x10015fdd4, `dragOcean:timeStamp:` 0x10015fee8, `startDragCrack:timeStamp:` 0x100160134, `stopDragCrack` 0x100160168, `dragCrack:timeStamp:` 0x1001601c4, `startDragUnderwater:` 0x1001601e4, `stopDragUnderwater:` 0x100160214, `dragUnderwater:` 0x100160228, `startApeWench:` 0x100162210, `dragApeWench:` 0x100162228, `stopApeWench:` 0x100162354, `checkUnderwaterBubble:pos:` 0x100163ac4.

#### 9.2.5 `menus_and_screens`

HUD and menus: onButtonPress:touch: dispatch for every controller-owned button (69 cached from logic table #561-631), the 6 category menus and their labels, menu open/close easing targets, hide/show UI, closeScreens, isScreenOpen/isDialogActive, message popup, help pages.

- Depends on: `app_platform` (4), `boot_and_frame` (9), `touch_input` (1), `island_locations` (3), `island_features` (39), `sky_weather` (2), `pygmies` (2), `stats_and_achievements` (27), `managers_bridge` (31).
- Owned ivars: 73 (section 8.5); ivars of other units used: 89.
- `PocketGodViewController.c` (14): `updateHelpPage:` 0x1000761b4, `updateHelpPage:::` 0x10007737c, `hideUi` 0x1000773ac, `showUi` 0x1000774f4, `closeScreens` 0x1000775a8, `isDialogActive` 0x100078430, `onButtonPress:touch:` 0x100079090, `onButtonRelease:touch:` 0x10007bc28, `onButtonReleaseOutside:touch:` 0x10007bc2c, `updateMenuLabelsDisplayGroup:` 0x10007dae0, `setCategoryMenuState:` 0x10007dd04, `areNameLabelsEnabled` 0x100087034, `areButtonLabelsEnabled` 0x100087070, `isScreenOpen` 0x1000870c4.
- `PocketGodViewController+Game.c` (4): `openMenu:::` 0x100159e2c, `closeMenu:::` 0x100159e54, `showMessage:::` 0x100159ebc, `removeMessage:::` 0x10015a0bc.

#### 9.2.6 `island_locations`

The 18 locations: setIslandLocationState:startup:, activate/deactivate (enter/exit objects, island bounds, horizon), tsunami transitions to/from underwater, per-location and minigame texture atlases, map/dream bubble/room fader/idol award travel callbacks.

- Depends on: `boot_and_frame` (3), `world_registry` (137), `touch_input` (4), `menus_and_screens` (6), `island_features` (43), `sky_weather` (8), `god_powers` (12), `pygmies` (4), `world_object_events` (1), `minigames` (7), `apocalypse` (3), `managers_bridge` (12).
- Owned ivars: 41 (section 8.6); ivars of other units used: 135.
- `PocketGodViewController.c` (20): `islandLocationState` 0x100077d18, `lastIslandLocationState` 0x100077d28, `onIdolAwardScreenChangeIslandLocationState:` 0x100078d64, `onMapScreenClosed` 0x100078fd8, `onMapScreenChangeIslandLocationState:` 0x100078fe8, `loadLocationTextureAtlas` 0x10007cbc8, `onLoadLocationTextureAtlas` 0x10007d3ac, `loadTextureAtlasChannel:::` 0x10007d3bc, `refreshSkinPackOption:::` 0x10007d468, `deactivateLastIslandLocationState` 0x10007ed70, `activateIslandLocationState:` 0x100080fa8, `setIslandLocationState:startup:` 0x100083b64, `isInsideApocalypseTemple` 0x1000871bc, `isDreamBubbleEnabled` 0x100087814, `isDoodleCloudEnabled` 0x100087868, `isOceanEnabled` 0x1000881a4, `hasMisfitIslandBeenFound` 0x1000881d8, `islandMinXPos` 0x100088214, `islandMaxXPos` 0x100088224, `islandCenterXPos` 0x100088234.
- `PocketGodViewController+Game.c` (9): `loadLocationTextureAtlas:::` 0x10015a118, `isTsunamiActive` 0x100163fc4, `onTsunamiActivateComplete:` 0x100164f88, `onTsunamiDeactivateComplete:` 0x1001650a8, `loadMonsterTextureAtlas` 0x100169bf0, `onApocalypseRoomFaderChangeIslandLocationState:` 0x100169d90, `onDreamBubbleChangeIslandLocationState:` 0x100169ef4, `onTsunamiHideIslandAssets:` 0x100169f58, `onTsunamiShowIslandAssets:` 0x100169f98.

#### 9.2.7 `island_features`

Per-island feature toggles (28 setXState:startup: incl. shark and bird), their option preferences and Start/Toggle button behaviours, startup restore, the ~60 isXEnabled predicates the logic objects query, and disable/enable of island logics for story mode.

- Depends on: `boot_and_frame` (33), `world_registry` (69), `menus_and_screens` (1), `island_locations` (63), `god_powers` (2), `world_object_events` (2), `minigames` (6), `managers_bridge` (1).
- Owned ivars: 68 (section 8.7); ivars of other units used: 47.
- `PocketGodViewController.c` (93): `setSharkState:startup:` 0x10007d894, `setBirdState:startup:` 0x10007d9c4, `updateIslandStateAtStartup` 0x10007e398, `setSandIslandState:startup:` 0x100083f50, `setRockIslandState:startup:` 0x1000842bc, `setRockIslandFeatureState:startup:` 0x100084414, `setRockIslandOutHouseState:startup:` 0x10008490c, `setRockIslandTRexState:startup:` 0x100084a48, `setRockIslandSpearState:startup:` 0x100084b80, `setGraveyardIslandState:startup:` 0x100084e24, `setGraveyardIslandZombieGraveState:startup:` 0x100084f60, `setGraveyardIslandGhostState:startup:` 0x10008509c, `setGraveyardIslandSpiderWebState:startup:` 0x100085198, `setGraveyardIslandSpiderState:startup:` 0x100085488, `setGraveyardIslandSpearState:startup:` 0x1000855c0, `setIceIslandState:startup:` 0x100085864, `setIceIslandIceMonsterState:startup:` 0x100085a48, `setIceIslandIglooState:startup:` 0x100085b84, `setIceIslandIceHoleState:startup:` 0x100085cc0, `setIceIslandIceSquidState:startup:` 0x100085dfc, `setApeIslandState:startup:` 0x100085f30, `setApeIslandApeState:startup:` 0x10008619c, `setApeIslandMonkeyState:startup:` 0x1000862d8, `setMisfitIslandState:startup:` 0x1000863d4, `setGodIslandState:startup:` 0x10008658c, `setGodIslandIdolsState:startup:` 0x1000866c8, `setApocalypseIslandState:startup:` 0x100086988, `setFailBoatState:startup:` 0x100086ac4, `setUnderwaterState:startup:` 0x100086b00, `setUnderwaterCoralReefState:startup:` 0x100086d90, `isVolcanoEnabled` 0x1000870ac, `isGravityEnabled` 0x1000871d8, `isSharkEnabled` 0x1000872b8, `isBirdEnabled` 0x100087308, `isBoulderEnabled` 0x1000873b8, `isCoconutEnabled` 0x1000873e8, `isFishEnabled` 0x100087400, `isAntHillEnabled` 0x100087474, `areWaterWingsEnabled` 0x1000874a4, `isTRexEggEnabled` 0x100087590, `isWoodEnabled` 0x1000875c4, `isBonFireEnabled` 0x1000875f8, `isSpearEnabled` 0x10008762c, `isTRexEnabled` 0x1000876b4, `isOutHouseEnabled` 0x1000876fc, `isZombieGraveEnabled` 0x100087730, `areGhostsEnabled` 0x100087764, `isSpiderEnabled` 0x100087798, `isSpiderTreeEnabled` 0x1000877e0, `isLaserSharkEnabled` 0x100087884, `isIceMonsterEnabled` 0x1000878f0, `isGlacierEnabled` 0x100087924, `isIceHoleEnabled` 0x10008793c, `isIceSquidEnabled` 0x100087970, `isIglooEnabled` 0x1000879b8, `isApeEnabled` 0x1000879ec, `isApeAltarEnabled` 0x100087a20, `isApeBongoEnabled` 0x100087a54, `isApeWenchEnabled` 0x100087a88, `isApeTorchEnabled` 0x100087abc, `isBananaTreeEnabled` 0x100087ad4, `isMonkeyEnabled` 0x100087b08, `isMoronSnakeEnabled` 0x100087b3c, `isJellyfishEnabled` 0x100087b70, `isIdolPedestalEnabled` 0x100087ba4, `isTempleDoorEnabled` 0x100087bbc, `isGodIdolEnabled` 0x100087bd4, `isApocalypseGemPedestalEnabled` 0x100087c08, `isApocalypseTempleDoorEnabled` 0x100087c20, `isApocalypsePortalTempleDoorEnabled` 0x100087c38, `isTimeTempleDoorEnabled` 0x100087c80, `isApocalypseTimeTempleLeverEnabled` 0x100087c98, `isApocalypseTimeTemplePlatformEnabled` 0x100087cb0, `isLocustQueenEnabled` 0x100087cc8, `isLocustTempleDoorEnabled` 0x100087ce0, `isGoGoCageEnabled` 0x100087cf8, `isDanceDanceExecutionEnabled` 0x100087d10, `isDanceTempleDoorEnabled` 0x100087d28, `isDanceBalloonBundleEnabled` 0x100087d40, `isApocalypsePlagueTempleFountainEnabled` 0x100087d58, `isPlagueTempleDoorEnabled` 0x100087d70, `isApocalypsePurificationTempleFountainEnabled` 0x100087d88, `isPurificationTempleDoorEnabled` 0x100087da0, `isClosetTempleDoorEnabled` 0x100087db8, `isClosetTempleThetanMeterEnabled` 0x100087dd0, `isClosetTempleCouchEnabled` 0x100087de8, `isClosetTempleClosetDoorEnabled` 0x100087e00, `isXenuTempleDoorEnabled` 0x100088080, `isXenuTempleEnabled` 0x100088098, `isXenuEnabled` 0x1000880b0, `isKongStatueEnabled` 0x1000880f8, `isKonkeyDongEnabled` 0x10008812c, `isMoronPestsEnabled` 0x100088168.
- `PocketGodViewController+Game.c` (2): `disableIslandLogics` 0x100165b9c, `enableIslandLogics` 0x100166068.

#### 9.2.8 `sky_weather`

Sky: sun/moon drag and day/sunset/night transitions, pinch-sun size, eclipse, the 11 sky gradients blend and clear colour, storm weather (button and cloud flick), fair-weather clouds, storm clouds, bat summoned by tapping the moon.

- Depends on: `boot_and_frame` (9), `world_registry` (1), `island_locations` (16), `god_powers` (2), `pygmies` (1), `world_object_events` (1), `minigames` (6), `managers_bridge` (4).
- Owned ivars: 55 (section 8.8); ivars of other units used: 17.
- `PocketGodViewController.c` (3): `setSunState:` 0x10007d4a8, `setWeatherState:startup:` 0x10007d5c4, `isStormCloudEnabled` 0x10008782c.
- `PocketGodViewController+Game.c` (29): `checkEclipse` 0x100159098, `updateGradients` 0x1001592b8, `setCloudGradientFade:::` 0x10015e32c, `transitionInStormClouds:::` 0x10015e684, `transitionOutStormClouds:::` 0x10015e6fc, `createClouds` 0x10015e774, `transitionInClouds:::` 0x10015ea30, `transitionOutClouds:::` 0x10015eb94, `onCloudDie:` 0x10015f2ac, `startDragClouds:timeStamp:` 0x10015f324, `stopDragClouds` 0x10015f50c, `dragClouds:timeStamp:` 0x10015f8b8, `startDragSun:` 0x10016035c, `startNight` 0x100160388, `startDay` 0x1001604c0, `startSunset` 0x100160624, `stopDragSun` 0x100160848, `dragSun:` 0x1001608e4, `startDragMoon:` 0x100160bc4, `stopDragMoon` 0x100160bdc, `dragMoon:` 0x100160c78, `checkForBatTapping:` 0x100163740, `sunState` 0x100163e88, `isEclipseActive` 0x100163f6c, `isDayActive` 0x100163f8c, `isStormActive` 0x100163fec, `isNightActive` 0x100164008, `isSunsetActive` 0x10016401c, `onBatDie:` 0x100169b24.

#### 9.2.9 `god_powers`

Drawn and triggered god powers: lightning, double rainbow, locust swarm, hurricane, underwater whirlpool and coral-reef temple power; drum tapping, celebration dance and dance songs; earthquakes and background shake; volcano hits/eruption; fireworks.

- Depends on: `app_platform` (2), `boot_and_frame` (5), `world_registry` (14), `island_locations` (12), `island_features` (4), `sky_weather` (1), `pygmies` (6), `minigames` (8), `managers_bridge` (7).
- Owned ivars: 54 (section 8.9); ivars of other units used: 26.
- `PocketGodViewController.c` (2): `canWarlockUseLightning:` 0x100087184, `isCelebrationEnabled` 0x100087348.
- `PocketGodViewController+Game.c` (49): `zombieIsPresent` 0x100157a44, `canDoNormalDance` 0x100157b84, `startCelebration` 0x100157ca4, `stopCelebrationMusic` 0x100158120, `stopCelebration` 0x1001583a4, `startPygmiesDancing:::` 0x10015a13c, `stopPygmiesDancing:::` 0x10015a454, `fireworks:::` 0x10015a7c8, `shakeBackground:::` 0x10015e374, `stopEarthquake:::` 0x10015e524, `onEruptionComplete:` 0x10015f0ec, `onLightningDie:` 0x10015f110, `onHurricaneCantPickUpObjects:` 0x10015f18c, `onHurricaneDie:` 0x10015f1b0, `onWhirlPoolCantPickUpObjects:` 0x10015f22c, `onWhirlPoolDie:` 0x10015f230, `startDrawLightning:` 0x100160d3c, `triggerLightning` 0x100160f74, `drawLightning:` 0x1001610cc, `stopDrawLightning:` 0x100161730, `hitPygmyWithWarlockLightning:` 0x100161818, `startDrawRainbow:` 0x100161a54, `drawRainbow:` 0x100161a9c, `stopDrawRainbow:` 0x100161db4, `drawLocust:` 0x100161ed8, `startDrawLocust:` 0x100161fac, `stopDrawLocust:` 0x1001620f0, `startHurricane:` 0x100162384, `maintainHurricane:` 0x1001626b8, `stopHurricane:` 0x100162718, `maintainWhirlPool:` 0x1001627c4, `stopWhirlPool:` 0x100162af8, `startDragUnderwaterPower:` 0x100162bac, `stopDragUnderwaterPower:` 0x100162eb4, `dragUnderwaterPower:` 0x100162f40, `onUnderwaterTempleModeChanged:` 0x100163394, `onMultiModelEvent:eventId:` 0x1001634ac, `checkForTapping:` 0x100163550, `isUnderwaterPowerActive` 0x100163eb8, `isRainbowActive` 0x100163f10, `isLocustActive` 0x100163f24, `isDanceActive` 0x100163f44, `isEruptionActive` 0x100163f58, `isEarthquakeActive` 0x100164030, `onPygmyHitByLightningInHurricane:` 0x100164668, `onPygmyEarthquakeStart:` 0x100164c10, `onPygmyEarthquakeStop:` 0x100164d6c, `onVolcanoHit:` 0x100164e00, `onFireworkDie:` 0x100164f10.

#### 9.2.10 `pygmies`

Pygmy lifecycle: createPygmyLogic placement, addPygmy limits (6 + Charlie + Tom), baby/Tom pygmies, pygmy drag/tumble/explode/die bookkeeping, sleeping sequence, canTouchPygmy gate.

- Depends on: `boot_and_frame` (3), `world_registry` (2), `menus_and_screens` (3), `island_locations` (4), `god_powers` (2), `world_object_events` (26), `minigames` (14), `stats_and_achievements` (5), `managers_bridge` (2).
- Owned ivars: 7 (section 8.10); ivars of other units used: 54.
- `PocketGodViewController.c` (2): `isTomPygmyActive` 0x100087e18, `isBabyPygmyActive` 0x100087f60.
- `PocketGodViewController+Game.c` (19): `isTomPygmyAlive` 0x10015b9a8, `createTomPygmy` 0x10015b9d8, `createPygmyLogic` 0x10015ba88, `addPygmy` 0x10015bfa4, `addPygmy:::` 0x10015c064, `addBabyPygmy:::` 0x10015c088, `addTomPygmy:::` 0x10015c0f8, `isDraggingPygmy` 0x100163bfc, `canTouchPygmy` 0x100163c34, `onPygmySleepingStart:` 0x100164474, `onPygmySleepingStop:` 0x100164514, `onPygmyGhostDrag:` 0x1001645a8, `onPygmyGhostDragComplete:` 0x1001645bc, `onPygmyDrag:` 0x1001645d0, `onPygmyDragComplete:` 0x1001645e4, `onPygmyTumble:` 0x100164604, `onPygmyTumbleComplete:` 0x100164664, `onPygmyExplode:` 0x100164714, `onPygmyDie:sacrificeId:` 0x10016485c.

#### 9.2.11 `world_object_events`

Callbacks from world objects: every onXDrag:/onXDragComplete: (dragged-object refs), coconut split/regrow, fishing bob, shark snaps, bird bombs, boss visibility and hunts (T-Rex, spider, ice monster), spears, piranha/Bait Master, laser shark, skewered/bowled achievements, with challenge-game scoring.

- Depends on: `world_registry` (5), `island_locations` (2), `pygmies` (3), `managers_bridge` (27).
- Owned ivars: 28 (section 8.11); ivars of other units used: 8.
- `PocketGodViewController+Game.c` (92): `isTRexActive` 0x10016404c, `isSpiderActive` 0x10016409c, `onPygmiesSkewered:` 0x10016468c, `onPygmiesBowled:` 0x1001646d0, `onCoconutDrag:` 0x1001650d8, `onCoconutDragComplete:` 0x1001650ec, `onWoodDrag:` 0x100165100, `onWoodDragComplete:` 0x100165114, `onSpearDrag:` 0x100165128, `onSpearDragComplete:` 0x10016513c, `onSpearThrow:` 0x100165150, `onWaterWingsDrag:` 0x1001651f0, `onWaterWingsDragComplete:` 0x100165204, `onFishingPoleDrag:` 0x100165218, `onFishingPoleDragComplete:` 0x10016522c, `onFishingPoleCreateBob:fishingBobLogic:` 0x100165240, `onFishingPoleDestroyBob:fishingBobLogic:` 0x100165288, `onFishingBobDrag:` 0x10016529c, `onFishingBobDragComplete:` 0x1001652b0, `onFishDrag:` 0x1001652c4, `onFishDragComplete:` 0x1001652d8, `onSwordFishDrag:` 0x1001652ec, `onSwordFishDragComplete:` 0x100165300, `onBananaDrag:` 0x100165314, `onBananaDragComplete:` 0x100165328, `onMonkeyDrag:` 0x10016533c, `onMonkeyDragComplete:` 0x100165350, `onDanceBalloonDrag:` 0x100165364, `onDanceBalloonDragComplete:` 0x100165378, `onMoronSnakeDrag:` 0x10016538c, `onMoronSnakeDragComplete:` 0x1001653a0, `onJellyfishDrag:` 0x1001653b4, `onJellyfishDragComplete:` 0x1001653c8, `onGodIdolDrag:` 0x1001653dc, `onGodIdolDragComplete:` 0x1001653f0, `onApocalypseGemDrag:` 0x100165404, `onApocalypseGemDragComplete:` 0x100165418, `checkCountCoconutBounces` 0x1001656c0, `onCoconutHitPygmy:` 0x1001657a4, `onCoconutHitGround:` 0x100165a70, `onCoconutDie:` 0x100165a80, `onBaitMasterStart:` 0x100169270, `onBaitMasterStop:` 0x100169394, `onNewPiranhaLogic:` 0x1001694dc, `onRemovePiranhaLogic:` 0x1001694fc, `onRemoveAllPiranhaLogics:` 0x10016951c, `onPiranhaEatPygmy:` 0x10016953c, `onLaserSharkZapPygmy:` 0x100169578, `onLaserSharkZapComplete:` 0x10016959c, `onAnchorDrag:` 0x1001695c0, `onAnchorDragComplete:` 0x1001695d4, `onTilePlugDrag:` 0x1001695e8, `onTilePlugDragComplete:` 0x1001695fc, `onBoulderDrag:` 0x100169610, `onBoulderDragComplete:` 0x100169624, `onTRexEggDrag:` 0x100169638, `onTRexEggDragComplete:` 0x10016964c, `onTRexVisible:` 0x100169660, `onTRexHitBySpear:` 0x10016969c, `onTRexKilledBySpear:` 0x100169704, `onTRexCancelHunt:` 0x100169728, `onSpiderVisible:` 0x10016974c, `onSpiderHitBySpear:` 0x100169780, `onSpiderKilledBySpear:` 0x1001697e8, `onSpiderCancelHunt:` 0x10016980c, `onLocustQueenVisible:` 0x100169830, `onApeVisible:` 0x100169868, `onIceMonsterVisible:` 0x1001698a0, `onIceMonsterHitByPygmy:` 0x1001698d8, `onIceMonsterKilled:` 0x1001698dc, `onIceMonsterStartBattle:` 0x100169900, `onIceMonsterCancelBattle:` 0x100169968, `onAntHillDirect:` 0x10016998c, `onAntHillDirectComplete:` 0x1001699a0, `onMagnifyingGlassDrag:` 0x1001699b4, `onMagnifyingGlassDragComplete:` 0x1001699c8, `checkCountSharkSnaps` 0x1001699dc, `onSharkDrag:` 0x100169a18, `onSharkDragComplete:` 0x100169a2c, `onSharkSnap:` 0x100169a40, `stopCountingBirdBombs` 0x100169a8c, `onSnowBallHitPygmy:` 0x100169ab0, `onSnowBallMissPygmy:` 0x100169ab4, `onBirdPooHitPygmy:` 0x100169ab8, `onBirdPooMissPygmy:` 0x100169adc, `onBirdFlyOffScreen:` 0x100169aec, `onBirdDrag:` 0x100169afc, `onBirdDragComplete:` 0x100169b10, `onClosetTempleClosetDoorDrag:` 0x100169ba0, `onClosetTempleClosetDoorDragComplete:` 0x100169bb4, `onThetanDrag:` 0x100169bc8, `onThetanDragComplete:` 0x100169bdc.

#### 9.2.12 `minigames`

Minigames hosted by the controller: Ooga Jump (doodle clouds and camera), Pain Drain, The Runs, Konkey Dong, Moron Pests, No Pee, Dance Dance Execution, Battle of the Gods (arena and idol battle screens); start/stop/score hooks and show/hide display groups.

- Depends on: `boot_and_frame` (1), `world_registry` (1), `touch_input` (10), `menus_and_screens` (21), `island_locations` (43), `god_powers` (3), `managers_bridge` (22).
- Owned ivars: 54 (section 8.12); ivars of other units used: 12.
- `PocketGodViewController+Game.c` (60): `stopTheRunsShakeScreen:::` 0x10015e608, `removeDoodleClouds` 0x10015ecc4, `createDoodleClouds` 0x10015ee34, `recycleDoodleClouds:` 0x10015ef84, `isDoodleJumpingPygmy` 0x100163be0, `onPygmyDoodleJumpFall:` 0x1001640ec, `showDoodleDisplayGroup:` 0x100164104, `hideDoodleDisplayGroup:` 0x100164268, `onPygmyStartDoodleJump:` 0x10016439c, `onPygmyStopDoodleJump:` 0x1001643cc, `onPainDrainStart:pygmyLogic:` 0x100166324, `onPainDrainStop:` 0x1001663b0, `onPainDrainScore:` 0x100166430, `showPainDrainDisplayGroup` 0x10016645c, `hidePainDrainDisplayGroup` 0x10016659c, `onPygmyFallIntoCrackHoleComplete:` 0x10016664c, `onTheRunsStart:pygmyLogic:` 0x10016670c, `onTheRunsStop:` 0x1001667ec, `onTheRunsScore:` 0x100166908, `onTheRunsStartMovingCamera` 0x100166934, `onTheRunsShakeScreenStart` 0x1001669b4, `onTheRunsShakeScreenStop` 0x100166a48, `showTheRunsDisplayGroup` 0x100166ab4, `hideTheRunsDisplayGroup` 0x100166bf4, `onKonkeyDongStart:pygmyLogic:` 0x100166ca4, `onKonkeyDongStop:` 0x100166dac, `onKonkeyDongScore:` 0x100166ec8, `onKonkeyDongStartMovingCamera` 0x100166ef4, `onKonkeyDongMoveCameraToNextLevel:` 0x100166f94, `showKonkeyDongDisplayGroup` 0x100166fbc, `hideKonkeyDongDisplayGroup` 0x1001670fc, `onMoronPestsStart:pygmyLogic:` 0x1001671ac, `onMoronPestsStop:` 0x10016728c, `onMoronPestsScore:` 0x100167398, `onMoronPestsStartMovingCamera` 0x1001673c4, `onMoronPestsStopMovingCamera` 0x100167444, `showMoronPestsDisplayGroup` 0x100167458, `hideMoronPestsDisplayGroup` 0x100167598, `isNoPeeActive` 0x100167648, `onNoPeeStart:pygmyLogic:` 0x10016765c, `onNoPeeStop:` 0x10016773c, `onNoPeeRetry:` 0x100167858, `onNoPeeStartMovingCamera` 0x10016789c, `showNoPeeDisplayGroup` 0x1001678b4, `hideNoPeeDisplayGroup` 0x1001679f4, `onDanceDanceExecutionStart:` 0x100167aa4, `onDanceDanceExecutionStop:` 0x100167b0c, `onDanceDanceExecutionScore:` 0x100167b68, `onBattleOfTheGodsStart:pygmyLogic:` 0x100167b94, `onBattleOfTheGodsStop:` 0x100167c64, `showBattleOfTheGodsDisplayGroup` 0x100167d18, `hideBattleOfTheGodsDisplayGroup` 0x100167e88, `onOpenIdolBattleEnemyScreen` 0x100167ffc, `onOpenIdolBattleScreen` 0x100168058, `onOpenIdolBattleResultsScreen` 0x10016808c, `showTheRunsDisplayGroup:::` 0x1001691bc, `showKonkeyDongDisplayGroup:::` 0x1001691e0, `showMoronPestsDisplayGroup:::` 0x100169204, `showNoPeeDisplayGroup:::` 0x100169228, `showBattleOfTheGodsDisplayGroup:::` 0x10016924c.

#### 9.2.13 `apocalypse`

Apocalypse event: start/end (gradients, music, thetans, gems, Xenu, calendar portal), gem spawning from episode events, plague body check, temple display group show/hide.

- Depends on: `world_registry` (7), `touch_input` (1), `island_locations` (22), `sky_weather` (3), `god_powers` (4), `pygmies` (3).
- Owned ivars: 2 (section 8.13); ivars of other units used: 18.
- `PocketGodViewController+Game.c` (10): `onApocalypseGemDie:` 0x10016542c, `onApocalypseGemEventTriggered:` 0x10016544c, `showApocalypseTempleDisplayGroup` 0x1001680d0, `hideApocalypseTempleDisplayGroup` 0x1001684e8, `isApocalypseActive` 0x100168884, `startApocalypse` 0x100168898, `endApocalypse` 0x100168998, `endApocalypseComplete` 0x100168b44, `endApocalypseCompleteNow` 0x100168c0c, `doesPlagueBodyExist` 0x100168cd4.

#### 9.2.14 `stats_and_achievements`

Tribe/pygmy/god stats pages, sacrifice/gift/mood/god-type/achievement text tables, sacrifice and gift recording, Game Center leaderboards and tiered achievements.

- Depends on: `app_platform` (2), `boot_and_frame` (2), `menus_and_screens` (1), `managers_bridge` (2).
- Owned ivars: 34 (section 8.14); ivars of other units used: 6.
- `PocketGodViewController.c` (21): `checkAchievementAndLeaderboard:` 0x100070a30, `checkAllAchievementsAndLeaderboards` 0x100070ae4, `onAchievement:count:highScoreWins:` 0x100070cc4, `onPygmySacrificeEvent:` 0x100070dc4, `onTriggerSacrifice:sacrificeId:` 0x100070de0, `getSacrificeDescription:` 0x100071888, `getSacrificeLeaderboardId:` 0x100072908, `getGiftDescription:` 0x100073410, `getGiftLeaderboardId:` 0x1000735f0, `getMoodTextureId:` 0x10007374c, `getMoodDescription:` 0x1000737d8, `getGodTypeTextureId:` 0x100073870, `getGodTypeDescription:` 0x100073908, `getAchievementDescription:` 0x1000739a0, `getAchievementLeaderboardId:` 0x100073ca0, `addFeintAchievementsToArray:achievementId:score:` 0x100073ebc, `checkFeintAchievements:score:` 0x10007492c, `updatePygmyStats` 0x100074ac4, `updateGodStats` 0x100075440, `updateTribeStats` 0x100075a88, `closeStatsScreen` 0x1000777d0.
- `PocketGodViewController+Game.c` (1): `onPygmyGiftGiven:giftId:` 0x100164adc.

#### 9.2.15 `managers_bridge`

Bridge to the manager singletons: open/close callbacks for skin packs, bonus packs, story (incl. story-mode environment, pygmies, earthquake, rainbow, celebration), news, idols/episodes, challenges, debug screen; episode events.

- Depends on: `app_platform` (2), `boot_and_frame` (13), `world_registry` (1), `touch_input` (3), `menus_and_screens` (5), `island_locations` (6), `island_features` (3), `sky_weather` (4), `god_powers` (9), `pygmies` (7), `minigames` (2).
- Owned ivars: 23 (section 8.15); ivars of other units used: 18.
- `PocketGodViewController.c` (42): `openSkinPackManager` 0x1000778b8, `openBonusPackManager` 0x100077abc, `openStoryManager` 0x100077bc0, `openChallengeManager` 0x100077c58, `onBonusPackScreenClosed` 0x100077d38, `onBonusPackUnlockDialogOpened` 0x100077d48, `onBonusPackUnlockDialogClosed` 0x100077d5c, `onSkinPackScreenClosed` 0x100077d6c, `onStoryScreenClosed` 0x100077d88, `onStoryModeStartCelebration` 0x100077d98, `onStoryModeStopCelebration` 0x100077da8, `onStoryModeStartEarthquake` 0x100077db8, `onStoryModeStopEarthquake` 0x100077f3c, `onStoryModeSharkSnap` 0x1000780a0, `onStoryModeActivateRainbow` 0x1000780bc, `onStoryModeActivateMyRainbow` 0x1000780d8, `onStoryModeDeactivateRainbow` 0x1000780f4, `removeAllPygmiesFromStoryMode` 0x100078110, `onStoryStart` 0x10007821c, `onStoryComplete` 0x100078344, `onStoryCancel` 0x100078400, `isStoryActive` 0x100078410, `onDebugScreenClosed` 0x100078420, `onStoryEnvironmentChange:` 0x100078460, `onStoryAddPygmy:` 0x100078840, `onNewsScreenClosed` 0x1000789ec, `onNewsTeaserClosed` 0x1000789fc, `onNewsTeaserLaunchNewsScreen` 0x100078a0c, `onNewsLaunchStoreScreen` 0x100078ab0, `onNewsLaunchChallengeScreen` 0x100078ad8, `onNewsLaunchStoryScreen` 0x100078b00, `onIdolScreenClosed` 0x100078b28, `onIdolScreenLaunchStoreScreen` 0x100078c48, `onIdolAwardScreenOpened` 0x100078c64, `onIdolAwardScreenClosed` 0x100078c78, `onIdolAwardScreenOpenChallenges` 0x100078c94, `onEpisodeAlertOpenChallengeDetail:` 0x100078d78, `onBattleAlertSelect` 0x100078f90, `onChallengeScreenClosed` 0x100078ffc, `onChallengeDialogOpen` 0x100079018, `onStartChallenge:` 0x100079060, `onEndChallenge:` 0x100079080.
- `PocketGodViewController+Game.c` (2): `triggerEpisodeEvent:::` 0x10015a78c, `isAnyGameActive` 0x100166304.

## 10. Appendix: id and text tables

Extracted from the `isEqualToString:` chains in the binary (strings resolved by address, not by Ghidra label). Ids are the strings `PygmyLogic` passes to `onPygmyDie:sacrificeId:` / `onTriggerSacrifice:sacrificeId:` / `onPygmySacrificeEvent:` (sacrifices) and `onPygmyGiftGiven:giftId:` (gifts), and that `ChallengeManager` and the boss logics pass to `onAchievement:count:highScoreWins:`; the tribe, pygmy and god personas store their counts under the same ids.

### 10.1 Sacrifices (116) — `getSacrificeDescription:` @ 0x100071888

Unknown id → "by undefined death". `getSacrificeLeaderboardId:` @ 0x100072908 compares the same ids and returns nil for every one (per-sacrifice boards compiled out). Order = order of the comparisons.

| Sacrifice id | Description | Sacrifice id | Description |
|---|---|---|---|
| `SharkFishingPole` | by fishing pole shark grab | `CrackHole` | fell in crack hole |
| `LoseGripUpsideDownFromEarthquake` | by earthquake on upside-down island | `LavaDrown` | drown in lava |
| `FallUpsideDown` | by fall from upside-down island | `LavaRock` | explode from lava rock |
| `ThrownExplosionDrown` | thrown from explosion then drowned | `ApeAltar` | torn apart on altar |
| `Lightning` | by lightning | `ApeEaten` | eaten by ape |
| `HurricaneLightning` | by hurricane then lightning | `ApeWenchSmash` | smashed by ape on winch |
| `Lava` | by lava | `ApeBongoSmash` | smashed by ape on bongos |
| `BoulderDrown` | by boulder-hit then drowning | `MonkeyOnBack` | face peeled by monkey on back |
| `TRexEggDrown` | by t-rex egg-hit then drowning | `MonkeyThrow` | thrown by monkey |
| `SharkDrown` | by shark-slap then drowning | `KidnappedByKonkeyDong` | eaten by konkey dong |
| `Vampire` | by vampire attack | `KonkeyDongBoulder` | bouldered by konkey dong |
| `MagnifyingGlass` | by magnifying glass | `KonkeyDongBeetle` | beetle juiced |
| `FireAnts` | by fire ants | `KonkeyDongFall` | fell off konkey dong level |
| `HurricaneDrown` | by hurricane then drowning | `EatenByMoronSnake` | eaten by moron snake |
| `FedToShark` | by shark feed | `EatenByMoronTurtle` | eaten by moron turtle |
| `VampireInSunlight` | as vampire in sunlight | `ZappedByWarlockAssassin` | zapped by warlock assassin |
| `SlideDrown` | by gravity slide then drowning | `BurntByWarlockAssassin` | burnt by warlock assassin |
| `ThrownInVolcano` | by volcano toss | `CutInHalfByCharlie` | cut in half by charlie |
| `FlickDrown` | by distance flick then drowning | `AbsorbedByHalfPygmy` | absorbed by half pygmy |
| `SharkOnHead` | by shark on head | `BattleFatality` | fatality in battle |
| `UpsideDownLoseGrip` | by losing grip upside-down | `JellyfishScubaMask` | scuba masked |
| `PushedDrown` | by pushing then drowning | `StungByJellyfish` | stung by jellyfish |
| `DroppedDrown` | by dropping then drowning | `EatenByJellyfish` | eaten by jellyfish |
| `NotCrapping` | by holding it too long | `DrankByJellyfish` | pygmy smoothy |
| `EatenByTRex` | by t-rex feed | `ElectrocuteByJellyfish` | circle shocked |
| `LovingPocketGod` | by loving pocket god | `HitByUnderwaterPowerCold` | frozen underwater |
| `Skewered` | by a good skewering | `HitByUnderwaterPowerHeat` | boiled underwater |
| `EatBurntFish` | by choking on burnt fish | `HitByUnderwaterPowerOil` | bp'ed |
| `BonFire` | by standing too close to bon fire | `HitByUnderwaterPowerAcid` | disolved in acid |
| `Cooked` | by being roasted like a marshmallow | `HitByUnderwaterPowerAir` | got the bends underwater |
| `EatBurntBird` | by choking on burnt bird | `BabyPygmyEatenByTrex` | veal dinner for t-rex |
| `CaughtByBird` | carried away by bird | `BabyPygmyEatenByShark` | veal cutlets for shark |
| `Flattened` | flattened like pancake | `OldFartPygmyHeartAttack` | heart attack |
| `FedToPiranha` | by piranha bait | `FastPygmyHitIceMonster` | flamed ice monster |
| `FedToLaserShark` | by underwater shark attack | `FastPygmyHitVolano` | instant eruption |
| `ZappedByLaserShark` | by shark laser beams | `GingerPygmySunBurn` | skin cancer |
| `ZombieExplode` | by being buried alive | `GingerPygmyKillVampire` | vampire meets his match |
| `ZombieExplodeBrainsEaten` | by brains being eaten | `EatenByLocustQueen` | a queen's snack |
| `CarriedAwayGhost` | carried away by ghost | `SlicedByLocust` | pygmy sashimi |
| `PulledIntoGround` | pulled into ghostworld | `PopLocustBalloon` | pop the balloon |
| `ScaredByGhost` | scared to death | `SquashedLikeBug` | squashed like a bug |
| `FromBansheeScream` | by burst eardrums | `LocustFood` | eaten like a leaf |
| `EatenBySpider` | by spider feed | `TurnedIntoMutantLocust` | mutant leaf eater |
| `EatenByLittleSpiders` | by baby spider feed | `DanceDanceExecuted` | dance dance executed |
| `HitBySnowBall` | by snow ball | `KilledByPurification` | vaporized by purehell |
| `EatenByIceMonster` | by ice monster feed | `PlagueBodyBurntOnFire` | cremated |
| `ExplodeOnIceMonster` | by damaging ice monster | `ExplodeFromThetanBirth` | thetan burst |
| `BounceOffIceMonster` | by ice monster deflection | `BecameXenu` | birth of lord xenu |
| `ShotOutIgloo` | by igloo overcrowding | `XenuPuppetShow` | impromptu puppet show |
| `IglooPygmy` | by igloo pygmy-hit then drowning | `XenuStripped` | flesh stripped off bone by xenu |
| `IglooCannon` | by being shot out of an igloo cannon | `EatenByXenu` | eaten by xenu |
| `SwordSwallower` | by swallowing a sword fish | `ThetanPummeled` | pummeled |
| `LostMyHead` | by losing ones head | `BittenToDeath` | face bitten off |
| `SquidBait` | by being fed to a squid | `WaterWingsDrowned` | what lies above |
| `GlobalWarming` | through global warming | `EatenByClam` | clam baked |
| `Shattered` | frozen, then shattered | `ImpregnatedByBlob` | impregnated by blob |
| `IciclePinCushion` | as an icicle pin cushion | `Stoned` | stoned |
| `PainDrainSpikes` | spiked in the pain drain | `RunOutOfBreath` | asphyxiated |

### 10.2 Gifts (12) — `getGiftDescription:` @ 0x100073410

Unknown id → "by undefined gift"; `getGiftLeaderboardId:` @ 0x1000735f0 returns nil for every id.

| Gift id | Description |
|---|---|
| `TakeCrap` | drop kids off at pool |
| `CatchFish` | caught fish |
| `Dance` | the gift of dance |
| `ZombieDance` | the gift of zombie dance |
| `Coconut` | coconut |
| `CookedFish` | cooked fish |
| `Fish` | fish |
| `CookedBird` | cooked bird |
| `BecomeGhost` | become ghost |
| `IntoTheLight` | go into the light |
| `CatchSwordFish` | caught sword fish |
| `Banana` | banana |

### 10.3 Achievements, leaderboards and tiers

Description: `getAchievementDescription:` @ 0x1000739a0 (unknown id → "undefined achievement"; shown on the god stats page). Leaderboard: `getAchievementLeaderboardId:` @ 0x100073ca0 (nil when not listed); the achievement count is submitted to it by `onAchievement:count:highScoreWins:` @ 0x100070cc4 and `checkAchievementAndLeaderboard:` @ 0x100070a30. Tiers: `addFeintAchievementsToArray:achievementId:score:` @ 0x100073ebc adds `<base>.minion`, `.shaman`, `.demigod`, `.god` cumulatively while the score meets each threshold (lower is better for the three hunts: 1–1000 is checked unsigned as `score − 1 < 1000`); `checkFeintAchievements:score:` @ 0x10007492c sets every returned id to 1 in the System document and calls `[GameServer submitAchievement:]` each time (no "already awarded" test). `checkAllAchievementsAndLeaderboards` @ 0x100070ae4 runs `checkAchievementAndLeaderboard:` for the 18 tiered ids in the order of this table except that `NoPee` comes before `DanceDanceExecution`; with no god persona yet it only sets `mbCheckAllAchievementsAndLeaderboards`, and `loadPreferences:::` runs it later. Totals leaderboards: `pg.totalsacrifices`, `pg.totalgifts` (4.12).

| Achievement id | Description | Leaderboard | Tier base | Minion | Shaman | Demigod | God |
|---|---|---|---|---|---|---|---|
| `ConqueredTyrannosaurus` | conquered tyrannosaurus | — | — |  |  |  |  |
| `CoconutBounces` | max coconut bounces | `pg.mostcoconutbounces` | `pg.coconutbouncing` | ≥ 1 | ≥ 20 | ≥ 50 | ≥ 100 |
| `HurricaneLightning` | max hurricane lightning | `pg.mosthurricanelightning` | `pg.hurricanezap` | ≥ 1 | ≥ 6 | ≥ 30 | ≥ 100 |
| `SharkSnaps` | max shark snaps | `pg.mostsharksnaps` | `pg.sharksnap` | ≥ 1 | ≥ 50 | ≥ 500 | ≥ 1000 |
| `Skewered` | max skewered | — | `pg.pygmykabob` | ≥ 1 | ≥ 4 | ≥ 5 | ≥ 6 |
| `Bowled` | max bowled | — | `pg.pygmybowling` | ≥ 1 | ≥ 4 | ≥ 5 | ≥ 6 |
| `BirdBombs` | max bird bombs | `pg.mostbirdbombs` | `pg.birdbomb` | ≥ 1 | ≥ 10 | ≥ 30 | ≥ 100 |
| `OogaJump` | longest ooga jump | `pg.oogajump` | `pg.oogajump` | ≥ 1 | ≥ 1000 | ≥ 3000 | ≥ 6000 |
| `BaitMaster` | most piranhas served | `pg.mostpiranhasserved` | `pg.baitmaster` | ≥ 1 | ≥ 10 | ≥ 50 | ≥ 200 |
| `LaserShark` | most shark laser blasts | `pg.mostsharkblasts` | `pg.sharkblast` | ≥ 1 | ≥ 50 | ≥ 300 | ≥ 800 |
| `KillTRex` | least spears to kill t-rex | — | `pg.trexhunt` | 1–1000 | ≤ 35 | ≤ 20 | ≤ 12 |
| `KillSpider` | least spears to kill spider | — | `pg.spiderhunt` | 1–1000 | ≤ 35 | ≤ 20 | ≤ 12 |
| `ConqueredIceMonster` | conquered ice monster | — | — |  |  |  |  |
| `KillIceMonster` | least pygmies to kill ice monster | — | `pg.icemonsterbattle` | 1–1000 | ≤ 30 | ≤ 15 | ≤ 10 |
| `PainDrain` | deepest pain drain | `pg.paindrain` | `pg.paindrain` | ≥ 1 | ≥ 1000 | ≥ 4000 | ≥ 10000 |
| `TheRuns` | longest the runs | `pg.theruns` | `pg.theruns` | ≥ 1 | ≥ 8000 | ≥ 30000 | ≥ 60000 |
| `KonkeyDong` | highest konkey dong score | `pg.konkeydong` | `pg.konkeydong` | ≥ 1 | ≥ 15000 | ≥ 50000 | ≥ 100000 |
| `MoronPests` | longest moron pests | `pg.moronpests` | `pg.moronpests` | ≥ 1 | ≥ 20000 | ≥ 50000 | ≥ 100000 |
| `DanceDanceExecution` | highest dance dance execution score | `pg.dancedanceexecution` | `pg.dancedanceexecution` | ≥ 1 | ≥ 1000 | ≥ 5000 | ≥ 10000 |
| `NoPee` | most pieces of booty collected | `pg.nopee` | `pg.nopee` | ≥ 1 | ≥ 4 | ≥ 10 | ≥ 20 |

### 10.4 Moods and god types

`getMoodDescription:` @ 0x1000737d8 / `getMoodTextureId:` @ 0x10007374c (texture for any other value: `PygmyMoodNormal`; description: nil) and `getGodTypeDescription:` @ 0x100073908 / `getGodTypeTextureId:` @ 0x100073870 (other values: nil). The god-type Next button cycles 0→6→0 (4.8.3).

| Mood | Description | Texture | God type | Description | Texture |
|---|---|---|---|---|---|
| 0 | Happy | `PygmyMoodHappy` | 0 | Volcano God | `GodTypeVolcano` |
| 1 | Content | `PygmyMoodContent` | 1 | Thunder God | `GodTypeThunder` |
| 2 | Normal | `PygmyMoodNormal` | 2 | Sun God | `GodTypeSun` |
| 3 | Sad | `PygmyMoodSad` | 3 | Ocean God | `GodTypeOcean` |
| 4 | Annoyed | `PygmyMoodAnnoyed` | 4 | Moon God | `GodTypeMoon` |
| 5 | Angry | `PygmyMoodAngry` | 5 | Dinosaur God | `GodTypeDinosaur` |
| 6 | FreakedOut | `PygmyMoodFreakedOut` | 6 | Octopus God | `GodTypeOctopus` |

## 11. Porting notes and kept quirks

- **One hub, many resources.** The controller is a god object; the units of section 9 map naturally
  to Bevy resources (state ivars) + systems. Cross-unit access goes through the queries the logic
  objects already use (`islandLocationState`, `is…Enabled`, `…Logic:` accessors), so a port can
  keep those as functions over shared resources.
- **Ordering inside a frame** matters: touches → sequences (controller scripts included) →
  animations → pygmies → logics → `onGameFrame:` (controller) → gradients → draw (3.1). Gesture
  handlers run during touch dispatch, before any `onGameFrame:` of that frame.
- **Shared stall timer**: the controller's sequences use `BCLibrary`'s single `stall`/`stallFrames`
  state (sequences.md §3.8); two controller scripts stalling at once interfere in the original.
- **Double-precision easing**: the island y easing (3.2 step 5) is done in double; everything else
  is f32.
- `mTargetIslandYPosVel` keeps growing by 3000·dt per idle frame while the target is clamped at 0
  (harmless; reset by a drag).
- `updateIslandStateAtStartup` puts `StartMoronSnakeOn` on the Moron Pests button (4.3).
- The second coconut's `setPosVel:` copies its own velocity (4.15) — the split coconut starts at rest
  unless `CoconutLogic split:` sets a velocity.
- Leaving Ape island exits the giant ape only when `ApeIslandState == 1` (4.1).
- `isSharkEnabled` is YES in temples and on the Fail boat when the shark option is on (bitmask 0x76
  over loc − 2); `isOceanEnabled` is NO on Ape island and in temples.
- Dead UI: the debug button (#569) is removed and never wired; `OpenFeintButton` (#624) has no display
  object; the What's New screen is never opened; `LavaButtonLogic` is supported by `setLogic:::` but no
  script creates it. `mbGameStarted`, `mbInitComplete`, `mDragSunXOrigin`, `mLastHurricanePos`,
  `mLastHurricaneTouchTime`, `mbDoodleDebugPress`, `mb…Starting` are written but never read; 12 ivars
  are never touched (catalogue).
- Per-sacrifice and per-gift leaderboards are compiled out (always nil); only `pg.totalsacrifices`,
  `pg.totalgifts` and the achievement boards are submitted.
- `endApocalypseComplete` and `endApocalypseCompleteNow` are identical.
- `isRainbowActive` and `isDanceActive` both return the celebration flag.
- `onGameFrame:` calls `stopCelebration` every frame once 1 s has passed since the last drum tap
  (it is a no-op when no celebration runs); while a story runs the check is skipped, so story
  celebrations last until `onStoryModeStopCelebration` or the end of the story.
- Two drum taps in the same frame count once (`mTapTouchTime != 0` test).
- Platform pieces to replace, not port: Crashlytics/Flurry, Chartboost, Game Center (map to local
  achievements or drop), the hidden `UITextField` (egui text input), orientation lock (landscape
  window), `openURL:` (open in the system browser).
- Ghidra labels CFStrings by a sanitised form, so different strings can share a label
  (`cf_RockIsland` is both "Rock Island" and "RockIsland"; 111 such labels exist, 21 are used by the
  controller). Every string quoted in this document for those labels was resolved from the
  disassembly; when in doubt, resolve the address, not the label.

## 12. Open questions

1. **`OpenFeintButton` (table #624)** is created by `createLogicClasses` for an object that does not
   exist in `CommunityMenuDisplayGroup` (its display object is nil). The controller never caches it;
   how `BCButtonLogic` behaves with a nil display object belongs to logic.md (§13 open questions).
2. **`isCelebrationEnabled`, `isApeTorchEnabled`, `isApocalypseTimeTempleLeverEnabled`** are
   implemented but no class sends them (searched every decompiled file for the selector string).
   They are listed for completeness; a port may drop them unless a sequence or logic spec finds a
   caller that is not a direct message send.
3. **Fail boat (loc 8)** has a menu and a state option (`FailBoatState`) with no button and no
   controller-cached objects; `FailBoatLogic` asks only `isStoryActive`. Whether the boat's content
   is entirely driven by `FailBoatLogic` and its sequences was not checked here (outside the
   controller).
4. **`hitPygmyWithWarlockLightning:`** builds the bolt from y = 300 in steps of
   `(pygmy.y − 300)/7`, with no x jitter on the first and the last (7th) point; the last point is one
   step short of the pygmy (y = 300 + 6·step). Transcribed as decompiled; the warlock/lightning spec
   should confirm that `LightningLogic` does not append the target point itself.
5. **Display-link interval.** `loadGame` sets `displayLinkCount` to 1 when loading took at most
   13 s and to 2 otherwise (2.3), so slow devices ran the game at 30 fps. Movement is dt-scaled, but
   several easings are per frame (island y 0.3, menu 0.5, gradients 0.13, Konkey Dong next-level
   0.07) and run twice as fast in wall time at 60 fps. Which rate the designers tuned these for
   cannot be read from the binary; the recommended port target is a fixed 60 Hz step (the
   fast-device behaviour).
