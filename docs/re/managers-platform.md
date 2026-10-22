# Managers and platform services

Reverse-engineering spec for the Pocket God 1.48.2 manager singletons and every platform service the game
code touches: UIKit, StoreKit, GameKit, iCloud, network, ads and analytics. It also holds the replacement
plan for the Rust/Bevy port. The other specs: `controller.md` (the `PocketGodViewController` side of every
listener protocol), `audio-prefs-text.md` (`BCPreferences`, `BCKeyChain`, the preference key catalogue),
`struct-xml.md` §8 (episode data and formulas), `textures.md` §7 (skin packs), `sequences.md` (sequence
items), `logic.md` (`IdolScreenLogic`, `TempleDoorLogic` and other logic classes).

Sources: `tmp/re/decomp/game/*.c` and `tmp/re/decomp/thirdparty/Reachability.c` (Ghidra), checked
against the arm64 disassembly where Ghidra dropped variadic or block arguments or mistyped fields
(marked "asm"). Strings come from `__cfstring`. Addresses are virtual addresses in `PocketGod.arm64`.
Methods are cited as `Class::selector @ 0xADDRESS`; C functions and blocks by their Ghidra name.
SDK keys and ids are cited by address only.

**Contents:**
1. [Overview](#1-overview)
2. [PocketGodAppDelegate](#2-pocketgodappdelegate)
3. [GameServer](#3-gameserver)
4. [IdolManager](#4-idolmanager)
   (4.4 [battle engine](#44-battle-of-the-gods-battle-engine),
   4.5 [multiplayer](#45-battle-of-the-gods-multiplayer-game-center))
5. [ChallengeManager](#5-challengemanager)
   (5.2 [scoring](#52-mini-game-scoring-and-personal-bests),
   5.3 [challenges](#53-mini-game-challenges-game-center))
6. [StoryManager](#6-storymanager)
7. [SkinPackManager](#7-skinpackmanager)
8. [BonusPackManager](#8-bonuspackmanager)
9. [NewsManager](#9-newsmanager)
10. [GameUiManager](#10-gameuimanager)
11. [MapScreen](#11-mapscreen)
12. [DebugScreen](#12-debugscreen)
13. [Controller contract](#13-controller-contract)
14. [Persistence summary](#14-persistence-summary)
15. [Reachability](#15-reachability)
16. [Keychain and preference documents](#16-keychain-and-preference-documents)
17. [Platform touchpoints inventory](#17-platform-touchpoints-inventory)
18. [Platform replacement plan](#18-platform-replacement-plan)
19. [Partition file](#19-partition-file)
20. [Open questions](#20-open-questions)

## 1. Overview

| Class | Kind | Singleton | Created by | Job |
|---|---|---|---|---|
| `PocketGodAppDelegate` | `UIApplicationDelegate` | — | UIKit | launch, splash, lifecycle, SDK start-up |
| `GameServer` | Game Center wrapper | `DAT_1007d1ee0` | controller `loadGame` | sign-in, scores, achievements, turn events |
| `IdolManager` | game | `DAT_1007cdc20` | controller | episodes, idol rewards, Battle of the Gods (single and multiplayer) |
| `ChallengeManager` | game | `DAT_1007cd808` | controller | mini-game counters and personal bests, Mini Game Challenges over Game Center |
| `StoryManager` | game | none | controller | story mode: built-in and user stories, editor, playback |
| `SkinPackManager` | game and StoreKit | `DAT_1007d1e80` | controller | 24 skin packs, keychain state, store |
| `BonusPackManager` | game | `DAT_1007d1f08` | controller | Comic Pre-Order jet-pack bonus, code entry |
| `NewsManager` | game and network | none | controller | news teaser and screen, feed merge and cache |
| `GameUiManager` | game | `DAT_1008161f0` | controller | floating points and messages (idol battles) |
| `MapScreen` | game | `DAT_1007d1ef0` | controller | island map |
| `DebugScreen` | game | `DAT_1007cd7e0` | controller | debug list (unreachable, items do nothing) |
| `MatchAndMessage` | value | — | `BattleAlertLogic` | (match, message) pair |
| `Reachability` | Apple sample | — | `SkinPackManager` | internet check |

All game managers share a pattern:
- a private `BCSequence` (`mSequence`; `StoryManager` adds `mStorySequence`) whose items (`name:::` handlers
  returning 0 wait, 2 done, 3 restarted) drive their screens;
- a listener pointing at the controller;
- logic objects created by the boot script's `setLogic` items or by `BCLibrary getLogic:<id>`.

Screens are display groups. Every close path ends in `loadLocationTextureAtlas`, so the controller can
restore the island atlases that the full-screen UI replaced.

Key findings:
- **Battle of the Gods is multiplayer.** It is a two-player asynchronous Game Center turn-based mode
  (§4.5), alongside the single-player campaign against 49 enemy line-ups (§4.3–§4.4). Mini Game
  Challenges (§5.3) are a second turn-based mode: the challenger sends a 61 s score attempt, the recipient
  tries to beat it.
- Both multiplayer modes, the store, the ads, the news server and iCloud are platform services with no
  equivalent here. §18 replaces each with the behaviour an iOS player sees when the service is unavailable or
  already satisfied, and offers an optional local hot-seat service.
- Dead or unreachable code worth knowing about:
  - the debug screen, plus its no-op items (§12);
  - the fail-boat map button (§11);
  - the news screen's MoreInfo button (§9.3);
  - `GameServer`'s score queue (§3.4);
  - the Battle of the Gods match alerts for inactive matches (§4.5);
  - `ChallengeManager`'s alert and unused text getters (§5.1, §5.3);
  - `leaveMatch` (§4.5).
- Quirks to keep:
  - the news teaser on every launch (§9.2);
  - the replay of queued episode events against a newly activated episode (§4.2);
  - Mini Game Challenge outcomes that always favour the higher score (§5.3);
  - the AI that sometimes throws a round (§4.4);
  - "Rechallenge" failing after a result (§5.3).

## 2. PocketGodAppDelegate

Source `game/PocketGodAppDelegate.c` (16 methods). The controller spec (`controller.md` §2.1, §2.7)
describes the same class from the controller side; this section is the platform view. Ivars:
`mPocketGodViewController`, `mImageView`, `mbStartupComplete`, `mbDashboardOpen` (never written, so
always NO), `mbModalViewOpen`, `window`.

### 2.1 Launch

`PocketGodAppDelegate::application:didFinishLaunchingWithOptions: @ 0x100089324`, in order:

1. `[Crashlytics startWithAPIKey:<key @ 0x1003c7ad0>]`.
2. `[Flurry setCrashReportingEnabled:NO]`, `[Flurry startSession:<key @ 0x1003c7af0>]`.
3. `[application setIdleTimerDisabled:YES]` (the screen never dims while the game runs).
4. `controller = [[PocketGodViewController alloc] init]`, `[controller setDelegate:self]`, stored in
   `mPocketGodViewController`.
5. `window.frame = UIScreen.mainScreen.bounds` (logged as `"WINDOW FRAME: %f %f %f %f"`),
   `rootViewController = controller`, `makeKeyAndVisible`, then `showLoadingImage`.
6. `[Chartboost startWithAppId:<0x1003c7b30> appSignature:<0x1003c7b50> delegate:controller]`, then
   `[Chartboost cacheInterstitial:@"Home Screen"]` (the CFString behind `cf_HomeScreen` has a space;
   it is Chartboost's home-screen location name).
7. `[NSTimer scheduledTimerWithTimeInterval:0.1 target:self selector:@selector(loadGame) userInfo:nil
   repeats:NO]` (interval stored as the double `0x3fb99999a0000000`, i.e. `(double)0.1f`).
8. Returns YES.

`loadGame @ 0x100089c10` only forwards to `[mPocketGodViewController loadGame]`.

### 2.2 Loading image

`showLoadingImage @ 0x100089570` reads `UIScreen.mainScreen.applicationFrame` = (x, y, w, h) and
builds a `UIImageView` added to the window, with a transform that starts at
`CGAffineTransformIdentity` and is translated to (w/2, h/2):

| Frame test (first match) | `imageNamed:` | Extra rotation | View size |
|---|---|---|---|
| h == 480 | `Default` | none | w × h |
| w == 480 | `Default` | −π/2 (`0xbff921fb54442d18`) | h × w |
| h == 568 | `Default-568h.png` | none | w × h |
| w == 568 | `Default-568h.png` | −π/2 | h × w |
| h == 1024 | `Default-Landscape~ipad.png` | +π/2 (`0x3ff921fb54442d18`) | h × w |
| h == 768 | `Default-Landscape~ipad.png` | none | w × h |
| otherwise | `Default` | none | w × h |

Strings: `0x1003c7b90` `Default-568h.png`, `0x1003c7bb0` `Default-Landscape~ipad.png`. The bundle
has `Default.png` 320×480, `Default@2x.png` 640×960, `Default-568h@2x.png` 640×1136,
`Default-Landscape~ipad.png` 1024×768 and `Default-Landscape@2x~ipad.png` 2048×1536, all in Apple's
CgBI PNG variant (byte-swapped BGRA, raw deflate, premultiplied), so they need converting before use. The
portrait files hold the landscape artwork turned 90°, which is why the phone cases rotate by −π/2.

`onHideLoadingScreen @ 0x100089c2c` (called by `PocketGodViewController(Game)::hideLoadingScreen`):
`mbStartupComplete = YES`, `removeFromSuperview`, release, `mImageView = nil`.

### 2.3 Lifecycle

| Method | Condition | Action |
|---|---|---|
| `applicationDidBecomeActive: @ 0x100089a24` | startup complete, `!mbDashboardOpen`, `!mbModalViewOpen` | `[[BCView instance] startRenderLoop]` |
| `applicationWillResignActive: @ 0x100089a90` | same | `stopRenderLoop` |
| `applicationDidEnterBackground: @ 0x100089afc` | `mbModalViewOpen` → `[controller dismissModalViewController]`; then if startup complete and `!mbDashboardOpen` | `stopRenderLoop` |
| `applicationWillEnterForeground: @ 0x100089b94` | startup complete, `!mbDashboardOpen` | `[controller refreshNews]` (= `[mNewsManager openNewsTeaserOnLoadComplete]` + `[mNewsManager loadNewsDefs]`, `PocketGodViewController(Game)::refreshNews @ 0x1001579f8`), then `startRenderLoop` |
| `applicationWillTerminate: @ 0x100089c0c` | — | nothing |
| `onModalViewWillAppear @ 0x100089c8c` | — | `mbModalViewOpen = YES`, `stopRenderLoop` |
| `onModalViewWillDisappear @ 0x100089cd0` | — | `mbModalViewOpen = NO`, `startRenderLoop` |

`dealloc @ 0x100089d10`, `window @ 0x100089d8c`, `setWindow: @ 0x100089d9c`,
`mPocketGodViewController @ 0x100089db8` and `setMPocketGodViewController: @ 0x100089dc8` are
memory management and synthesized accessors.

### 2.4 Port

- The Bevy `App` replaces the delegate. Build the window, show the launch image (converted from CgBI) until
  the controller's `hideLoadingScreen` fires, then start the game. Run `loadGame` on the first update after
  the window exists. The 0.1 s timer only lets UIKit draw the splash, so a one-frame deferral is
  enough.
- Idle-timer disabling has no desktop equivalent. On Android, keep the screen on (`FLAG_KEEP_SCREEN_ON`).
- Focus lost or minimized maps to `stopRenderLoop`: pause the game clock and skip `onGameFrame`.
  Focus regained maps to `startRenderLoop` (the first frame then gets dt = 1/15 s, `controller.md` §2.7).
  Do not call `refreshNews` on refocus: the original reloads news only on a true background→foreground
  transition, which a desktop window does not have (on Android, call it from `Resumed`).
- `mbDashboardOpen` is dead, so drop it. Model `mbModalViewOpen` as "an egui modal is open" only if a
  modal must pause the world. The only modals in the original are Game Center sheets (§18).
- Crashlytics, Flurry and Chartboost are removed. See §18.

## 3. GameServer

Source `game/GameServer.c` (39 methods and 8 block functions). Game Center wrapper. Singleton
`DAT_1007d1ee0`, set in `init`. `instance @ 0x10008a90c`, `id @ 0x10008aa78`.

### 3.1 State

| Ivar | Meaning |
|---|---|
| `mGameServerListener` | `PocketGodViewController`. Presents and dismisses the sign-in sheet and receives `checkAllAchievementsAndLeaderboards` |
| `mGameServerLeaderboardListener`, `mGameServerAchievementListener` | `PocketGodViewController` (set in `loadGame`) but never called back |
| `mGameServerChallengeListener` | never set |
| `mGameServerAuthenticationListenerArray` | `NSMutableArray` (strong references). In order: `ChallengeManager` (its `init`), `IdolManager` (its `init`) |
| `mLeaderboardDictionary` | empty `NSMutableDictionary`. Never read |
| `mbGameCenterNotAvailable`, `mbSubmittingScore` | never set (the score queue is dead code) |
| `mAuthenticationViewController` | sign-in sheet handed over by GameKit (iOS 6+) |
| `mbStartupCheckShowAuthenticationViewController` | set once the boot script reached `checkShowAuthenticationViewController:::` |

`PocketGodViewController::loadGame` creates it (`alloc`/`init`), then calls `setGameServerListener:`,
`setGameServerLeaderboardListener:` and `setGameServerAchievementListener:` with itself
(`controller.md` §2.3). All setters retain their argument (`setGameServerListener: @ 0x10008aa84`,
`…LeaderboardListener: @ 0x10008aae0`, `…AchievementListener: @ 0x10008ab3c`,
`…ChallengeListener: @ 0x10008ab98`). `addGameServerAuthenticationListener: @ 0x10008abf4` and
`removeGameServerAuthenticationListener: @ 0x10008ac10` add to and remove from the array.
`dealloc @ 0x10008ac2c`.

### 3.2 Authentication

`GameServer::init @ 0x10008a918`:
1. Creates the dictionary and the listener array (capacity 5).
2. Registers as a game-frame listener. `gameFrameOrder @ 0x10008ba48` = 70 (`0x46`), and
   `onGameFrame: @ 0x10008ba50` is empty.
3. Observes `UIApplicationDidBecomeActiveNotification` with selector `applicationDidBecomeActive`.
   `applicationDidBecomeActive @ 0x10008aa68` calls `authenticateUser`.
4. Calls `authenticateUser` immediately. The listener array is still empty at this point, because
   `ChallengeManager` and `IdolManager` are created later in `loadGame`.

`authenticateUser @ 0x10008b298`:
1. Sends `onGameServerAuthenticationStarted` to every authentication listener.
2. If `GKLocalPlayer` responds to `authenticateHandler` (iOS 6 and later), sets the handler to the
   block `FUN_10008b48c`. Otherwise it calls `authenticateWithCompletionHandler:` with the block
   `FUN_10008b678`.

| Block | Behaviour |
|---|---|
| `FUN_10008b48c(block, viewController, error)` (iOS 6+) | Logs `isAuthenticated` and the error. If `viewController` is non-nil, it retains it into `mAuthenticationViewController` and, only if `mbStartupCheckShowAuthenticationViewController` is set, performs `showAuthenticationViewController` on the main thread. If `viewController` is nil, it first performs `hideAuthenticationViewController` (only when a sheet is stored), then performs `onGameCenterAuthenticationComplete` on the main thread. This also happens when authentication failed or the player cancelled. |
| `FUN_10008b678(block, error)` (iOS 5) | Logs the error and performs `onGameCenterAuthenticationComplete` on the main thread |
| `FUN_10008b60c` / `FUN_10008b648`, `FUN_10008b6f0` / `FUN_10008b700` | block copy and dispose helpers |

- `checkShowAuthenticationViewController @ 0x10008b1b4` sets `mbStartupCheckShowAuthenticationViewController = YES`.
  It returns whether a sheet is pending, and shows it on the main thread if so. The boot script reaches it
  through `PocketGodViewController(Game)::checkShowAuthenticationViewController::: @ 0x100159c44`. That handler
  ignores the result and returns 2. The handler appears in the `StartScreen` and `NoStartScreen` boot branches,
  after `hideLoadingScreen` (`controller.md` §2.4).
- `showAuthenticationViewController @ 0x10008b214` calls `[mGameServerListener presentModalViewController:sheet]`.
- `hideAuthenticationViewController @ 0x10008b240` calls `[mGameServerListener dismissModalViewController]`
  and releases the sheet.
- `onGameCenterAuthenticationComplete @ 0x10008ad18` sends `onGameServerAuthenticationComplete` to every
  authentication listener in array order. It then sets
  `[GKTurnBasedEventHandler sharedTurnBasedEventHandler].delegate = self` and calls
  `[mGameServerListener checkAllAchievementsAndLeaderboards]`.
- Listeners must call `[GKLocalPlayer localPlayer].isAuthenticated` themselves (§5.3, §4.5).
- `isTurnBasedMultiplayerAvailable @ 0x10008b134` is true when both conditions hold:
  `NSClassFromString(@"GKTurnBasedMatch")` exists, and
  `[systemVersion compare:@"5.0" options:NSNumericSearch]` (`0x40`) is not `NSOrderedAscending`.
- `canReceiveCallbacksNow @ 0x10008b70c` returns YES.

### 3.3 Turn-based events (GKTurnBasedEventHandlerDelegate)

| Method | Behaviour |
|---|---|
| `handleInviteFromGameCenter: @ 0x10008af98` | empty (invites are not handled) |
| `handleTurnEventForMatch: @ 0x10008af9c`, `handleTurnEventForMatch:didBecomeActive: @ 0x10008afc4` | `performSelectorOnMainThread:@selector(mainHandleTurnEventForMatch:) withObject:match waitUntilDone:NO` |
| `handleMatchEnded: @ 0x10008b10c` | same, with `mainHandleMatchEnded:` |
| `mainHandleTurnEventForMatch: @ 0x10008ae78` | for each authentication listener in order: `if ([l handleTurnEventForActiveMatch:match]) break;` |
| `mainHandleMatchEnded: @ 0x10008afec` | same, with `handleMatchEndedForActiveMatch:` |

The listeners' `handle…NotificationForInactiveMatch:` methods are never called by `GameServer`.

### 3.4 Scores and achievements

- `submitToLeaderboard:score: @ 0x10008b894`:
  `[[GKScore alloc] initWithCategory:id]`, `value = (int64)score`, `context = 0`,
  `reportScoreWithCompletionHandler:` with the global block `FUN_10008b928`, which only logs
  `LEADERBOARD ERROR`. The score object is never released.
- `submitAchievement: @ 0x10008b970`: `[[GKAchievement alloc] initWithIdentifier:id]`.
  `percentComplete = 100.0`. Its block `FUN_10008ba00` logs `ACHIEVEMENT ERROR`.
- Callers (all in `PocketGodViewController`, `controller.md` §4.12, `audio-prefs-text.md` §2.6.2):
  - `pg.totalsacrifices` from `onTriggerSacrifice:sacrificeId: @ 0x100070de0`.
  - `pg.totalgifts` from `PocketGodViewController(Game)::onPygmyGiftGiven:giftId: @ 0x100164adc`.
  - The 13 achievement leaderboards from `checkAchievementAndLeaderboard: @ 0x100070a30` and
    `onAchievement:count:highScoreWins: @ 0x100070cc4`:
    `getAchievementLeaderboardId: @ 0x100073ca0` maps `CoconutBounces` `pg.mostcoconutbounces`,
    `HurricaneLightning` `pg.mosthurricanelightning`, `SharkSnaps` `pg.mostsharksnaps`, `BirdBombs`
    `pg.mostbirdbombs`, `OogaJump` `pg.oogajump`, `BaitMaster` `pg.mostpiranhasserved`, `LaserShark`
    `pg.mostsharkblasts`, `PainDrain` `pg.paindrain`, `TheRuns` `pg.theruns`, `KonkeyDong`
    `pg.konkeydong`, `MoronPests` `pg.moronpests`, `DanceDanceExecution` `pg.dancedanceexecution`,
    `NoPee` `pg.nopee`; anything else → nil.
  - The 72 tier achievements `pg.<game>.<minion|shaman|demigod|god>` from
    `checkFeintAchievements:score: @ 0x10007492c`. The thresholds are in `audio-prefs-text.md` §2.6.2,
    checked against `addFeintAchievementsToArray:achievementId:score: @ 0x100073ebc`. The three hunts
    use `(unsigned)(score − 1) ≤ 999` for minion (`sub w8, w20, #1; cmp w8, #0x3e7; b.hi`).
- `getSacrificeLeaderboardId: @ 0x100072908` always returns nil. `getGiftLeaderboardId: @ 0x1000735f0`
  returns nil for the 12 known gift ids (`TakeCrap` … `Banana`). For any other id it returns the string `"by undefined gift"`
  (copied from the description table, asm `0x100073724–0x100073748`), which would be submitted as a
  leaderboard id. No such gift id reaches it in 1.48.2.
- The following are logging stubs or empty, and nothing in the game calls them:
  - Dead score-queue methods: `onScoreSuccess @ 0x10008b714`, `onScoreFailure @ 0x10008b74c` (calls
    `onScoreSuccess`), `onPendingHighScoreSuccess @ 0x10008b75c`, `onPendingHighScoreFailure @ 0x10008b794`,
    `onPendingAchievementSuccess @ 0x10008b7cc`, `onPendingAchievementFailure @ 0x10008b804`,
    `onUnlockAchievementsSuccess @ 0x10008b83c`, `onUnlockAchievementsFailure @ 0x10008b874`. They compare
    `DAT_1007e50a0`/`DAT_1007e50a4` and clear `mbSubmittingScore`.
  - Empty: `submitNextScore @ 0x10008b884`, `sendPendingData @ 0x10008b888`,
    `checkLeaderboardQueue @ 0x10008b88c`, `checkLeaderboardQueuePool @ 0x10008b890`.

### 3.5 Port

`managers::game_server` keeps the same API as a local stand-in (§18). Behaviour:
- `authenticate_user` immediately reports "not authenticated". The original fires
  `onGameServerAuthenticationComplete` on every outcome, so the listeners must still get the call. It
  then calls `check_all_achievements_and_leaderboards` (the call only re-submits).
- `is_turn_based_multiplayer_available` = false unless a local match service is enabled (§18.3).
- Sign-in sheet calls are no-ops.
- Leaderboards keep a local best per id (the max, as Game Center's default sort for these ids is unknown).
- Achievements are stored only through the System document keys that the controller already writes.

## 4. IdolManager

Source `game/IdolManager.c` (170 methods, 1 class method, 21 block functions). The class owns:
- the **episodes**: 49 god idols, each unlocked by challenges and levelled by level-up tracks;
- the four-idol **selection**, shown as the four god idols on God island;
- the episode and battle **alerts**;
- the **Battle of the Gods** battle engine;
- its **Game Center multiplayer**.

The data layouts, the level-threshold formula, the attack and realm triangles, and the special episodes
(Episode1, Episode39, Episode40) are specified in `struct-xml.md` §8. This section covers the behaviour
around them.

Singleton `DAT_1007cdc20`. `instance @ 0x10002a1b4`, `id @ 0x10002a420`,
`setIdolManagerListener: @ 0x10002a42c` (the controller), `dealloc @ 0x10002a488`,
`.cxx_construct @ 0x10003466c`.

### 4.1 State

| Ivar | Meaning |
|---|---|
| `mIdolScreenLogic` (logic 0x151), `mEpisodeAlertLogic` (399), `mBattleAlertLogic` (400) | the idol screens (`IdolScreenLogic`), the episode-alert icon, the match-alert icon (`episodeAlertIcon @ 0x10002a694`, `battleAlertIcon @ 0x10002a6a4`) |
| `mbIdolScreenActive` | an idol screen is open |
| `mActivatedHintCount` | hints bought, over all episodes (`activatedHintCount @ 0x10002f830`) |
| `mGemIdDictionary` | challenge `event_id`, `sacrifice_id` and `gift_id` → Apocalypse gem 1–7 |
| `mDelayedEpisodeEventArray`, `…GiftEventArray`, `…SacrificeEventArray`, `mDelayedAwardScreenArray` | queues used while events cannot be applied |
| `mpActiveStructXmlEpisodeNodeDef` | the active episode, the only one events apply to |
| `mSelectedStructXmlEpisodeNodeDefArray[4]` | the selection slots |
| `mpEnemyStructXmlEpisodeNodeDef`, `mAttackResult`, `mRealmResult`, `mPlayerBattleState`, `mEnemyBattleState` | battle (§4.4) |
| `mPlayerRandomFrozenIdolId`, `mEnemyRandomFrozenIdolId`, `mActiveMatch`, `mMatchPreferences`, `mbLoadActiveMatchData`, `mbMultiPlayerActive`, `mbMultiplayerMyTurn`, `mbIAmChallenger`, `mPlayerPreferencePrefix`, `mEnemyPreferencePrefix`, `mActivePlayerId`, `mbGameCenterAuthenticationComplete`, `mbGameCenterNotAvailable`, `mbOpenMatchPanelOnAuthenticationComplete` | multiplayer (§4.5) |

`init @ 0x10002a1c4` does four things:
- creates the sequence, the dictionary and the four queues;
- fills the level-threshold table `T[i] = 1000·i(i+1)/2` (`0x1007cdc28`, 20 ints, `T[19]` = 190000);
- clears the Game Center flags;
- registers as the **second** `GameServer` authentication listener.

Persistence. The key formats come from `loadEpisodeState`; the full catalogue is `audio-prefs-text.md` §2.6.4.
- Challenge document (secure and in iCloud):
  - `<episode>BattleUnlock` (0/1). Episode1 is forced to 1 at every load.
  - `<episode><event>Challenge` and `<episode><event>ChallengeHint` (0/1).
  - `<episode><event>LevelUp` (points).
- Options document:
  - `ActiveEpisode` (episode id string, `""` = none);
  - `<episode>SelectionSlot` (0–3, default −1).

### 4.2 Episodes, events and awards

Gates (all ask the listener):
- `canTriggerChallenge @ 0x10002a6b4` = `(!isScreenOpen || [[TempleDoorLogic instance]
  isBattleOfTheGodsActive]) && !isStoryActive`.
- `canShowAwardScreen @ 0x10002a73c` and `canShowIdolScreen @ 0x10002a7c8` =
  `!isScreenOpen && !isStoryActive && !isAnyGameActive && !isDialogActive`.

Loading:
- `loadEpisodeState @ 0x10002b7b8` (from the controller's `loadPreferences:::`):
  1. Become the Challenge document's listener, which routes iCloud updates to `onPreferencesChanged:`.
  2. For every episode in list order:
     - activate it if its id equals `ActiveEpisode`;
     - read its slot (default −1); a valid slot fills `selected[slot]` and sets
       `[[listener godIdolLogicArray] objectAtIndex:slot]`'s episode;
     - read `BattleUnlock`, with Episode1 forced to 1 and saved;
     - for each challenge: register its gem ids, build the two keys and load the completed and hint flags;
     - for each level-up: build its key and load the points;
     - `updateEpisodeStatus:startup:YES`.
  3. Recount the hints.
- `onPreferencesChanged: @ 0x10002b5fc` re-reads the same keys. It only ever **sets** flags to 1 (never
  clears them) and replaces the points, then updates every episode with startup YES.

Episode status:
- `updateEpisodeStatus:e startup:s @ 0x10002aa10` recomputes the challenge count, completed count, hint count
  and the all-complete flag (+0x8c). It also recomputes each level track and the episode level (the minimum
  over its tracks, 0 without tracks), following `struct-xml.md` §8.6.
- **Award:** when the all-complete flag turns from 0 to 1 and `s == NO`, it calls
  `openIdolAwardScreen:e`, `checkEmptyIdolSlot:e` and `checkImmediateUnlocks:e`.
- `updateEpisodeHintStatus @ 0x10002abbc` recounts the hints per episode (+0x9c) and in total.

Triggers. Callers:
- `triggerEpisodeEvent:`: the controller, about 40 logic classes and the `triggerEpisodeEvent` sequence
  item.
- `triggerEpisodeGameScoreEvent:score:`: `ChallengeManager`.
- `delayTriggerEpisodeEvent:`: the controller, `ChallengeManager`, `NewsManager` and `StoryManager`.

The methods:
- `triggerEpisodeEvent:id @ 0x10002fd90`:
  1. If `!canTriggerChallenge`, queue it with `delayTriggerEpisodeEvent:` and stop.
  2. Otherwise `checkGemEvent:id @ 0x10002f8b4`: a mapped gem → listener `onApocalypseGemEventTriggered:gem`.
  3. For the **active** episode only: every incomplete challenge whose `event_id` equals id is completed
     (`[mEpisodeAlertLogic sparkle]`, preference 1).
  4. If the episode is already earned: every level-up with that `event_id` gains 1 point (capped at its
     max), with a sparkle and a save.
  5. `updateEpisodeStatus:startup:NO`.
- `triggerEpisodeEventBySacrificeId: @ 0x10002f930` and `triggerEpisodeEventByGiftId: @ 0x10002fb60` do the
  same on `sacrifice_id` or `gift_id`, ignoring NULL ids, and queue into their own arrays.
- `triggerEpisodeGameScoreEvent:id score:n @ 0x10002ffb8` (§5.2) is **never queued**. It only acts when the
  active episode is earned: matching `event_id` level-ups gain `n`, capped.
- `delayTriggerEpisodeEvent: @ 0x1000300fc`, `delayTriggerEpisodeGiftEvent: @ 0x100030184` and
  `delayTriggerEpisodeSacrificeEvent: @ 0x10003020c` append to their queue and start sequence
  `IdolManagerCheckAutoUnlockChallenges` (`waitForScreenClosed`, `stall 2.0`, `checkAutoUnlockChallenges`).
- `waitForScreenClosed::: @ 0x10002c490` returns 0 until `canTriggerChallenge`.
- `checkAutoUnlockChallenges::: @ 0x10002c4bc`: if it still cannot trigger, it restarts the sequence (3);
  otherwise it calls `checkAutoUnlockChallenges` and returns 2.
- `checkAutoUnlockChallenges @ 0x10002bcd8`:
  1. **Only if the active episode exists and is not earned:**
     - Fire the environment challenges that are currently true, through `triggerEpisodeEvent:` with the
       challenge's own id:

       | Challenge id | True when |
       |---|---|
       | `StormClouds` | `isStormCloudEnabled` |
       | `Night` | `isNightActive` |
       | `Sunset` | `isSunsetActive` |
       | `RockIsland`, `GraveyardIsland`, `IceIsland`, `ApeMountain`, `MisfitIsland`, `GodIsland` | island location state 1, 3, 4, 5, 6, 7 |
       | `Underwater`, `UnderwaterCoralReef`, `ApocalypseIsland` | state 2, 9, 10 |
       | `TimeTemple` … `XenuTemple` | states 11–17 |

     - Replay and clear the three event queues.
  2. If an award is queued and `canShowAwardScreen`, open the first one. If more remain, restart the
     sequence.

  Quirk: with a complete (or no) active episode, the queues are kept. They are replayed against the next
  incomplete episode activated, so events from earlier play can complete its challenges.

Activation, hints and selection:
- `onEpisodeActivated:e @ 0x10002f71c`:
  1. Becomes active and is saved as `ActiveEpisode`.
  2. Loads its icon (+0x60) into `EpisodeAlertIconSlot` (`0x100569a68`) and its idol (+0x68) into
     `EpisodeAlertIdolSlot` (`0x100569a98`), through `FUN_100038648(slots, name, @"raw")`
     (`textures.md` §7.3).
  3. `[mEpisodeAlertLogic activate:e]`.
  4. Starts the auto-unlock sequence, so challenges already satisfied complete about 2 s later.
- `onHintActivated:challenge @ 0x10002f840`: save the hint key 1, set the flag, recount.
- `onEpisodeIdolSelected:e slot:s @ 0x10002c6a0` (s < 4): deselect e from its old slot, deselect the occupant
  of s, save `<e>SelectionSlot = s`, `selected[s] = e`, and set the god idol logic s.
- `onEpisodeIdolDeselected:e @ 0x10002c560`: if e really occupies its slot, save −1, clear the god idol
  logic and the slot.
- `selectedEpisodeIdolInSlot: @ 0x10002c81c` reads a slot.
- `checkEmptyIdolSlot:e @ 0x10002a854` puts e into the first empty slot, if any.
- `checkImmediateUnlocks:e @ 0x10002a88c`:
  - For Episode1: complete Episode2's single challenge `UnlockEpisode1`, update it (startup YES) and slot it.
  - Then, if every episode before Episode39 in list order is earned: complete Episode39's last challenge
    (`UnlockAllChallenges`), update it, and slot it if it is now earned.

Screens (all through `IdolScreenLogic`, owned by the logic partition):
- `openIdolStartScreen @ 0x10002f2b4`, `openIdolListScreen @ 0x10002f2ec` (with the active episode),
  `openIdolSelectionScreen @ 0x10002f334`, `openIdolBattleEnemyScreen @ 0x10002f36c`,
  `openIdolBattleScreen @ 0x10002f3a4` and `openIdolDetailScreen: @ 0x10002f408`: each does nothing while
  `mbIdolScreenActive`, otherwise sets it and forwards.
- `openIdolBattleResultsScreen @ 0x10002f3dc` always forwards.
- `closeIdolScreen @ 0x10002f440` → `closeScreen`.
- `onIdolScreenClosed @ 0x10002f474` clears the flag and calls the listener.
- `onIdolScreenLaunchStoreScreen @ 0x10002f49c` forwards to the listener.
- `openIdolAwardScreen:e @ 0x10002f4b8`:
  - If `!canShowAwardScreen`: queue e (`NSValue valueWithPointer:`) and start the auto-unlock sequence.
  - Else, if an idol screen is open, **the award is dropped**.
  - Else: set the flag, listener `onIdolAwardScreenOpened`, load e's idol into `EpisodeAlertIdolSlot`,
    `[mIdolScreenLogic openIdolAwardScreen:e]`.
- `onIdolAwardScreenClosed @ 0x10002f600`, `onIdolAwardScreenOpenChallenges @ 0x10002f664` and
  `onIdolAwardScreenChangeIslandLocationState:s @ 0x10002f6bc` restore the active episode's idol image and
  forward to the listener. The first one also clears the flag.

Maintenance helpers, used by the `IdolScreenLogic` "are you sure" panel (`IdolScreenLogic.c:4273`: type 0
reset, 1 unlock, 2 level-up):
- `resetAllChallenges @ 0x10002ac2c`: deselect all, `BattleUnlock` 0 except Episode1, clear all challenge,
  hint and level-up keys.
- `unlockIdolChallenges @ 0x10002b4fc`: complete every challenge without awards (startup YES).
- `setIdolLevels:20 @ 0x10002b158`: points = `(int)(M / 190000.0f × T[n − 1])`, i.e. the maximum for n = 20.
- `resetEpisodeState @ 0x10002ae14`, `resetBattleState @ 0x10002af40`, `unlockBattleState @ 0x10002b04c`,
  `resetIdolLevels @ 0x10002b28c` and `resetIdolChallenges @ 0x10002b338` are not called by the game.

Other members:
- `activeEpisodeNodeDef @ 0x10002f2a4` and `optionPreferences @ 0x100030294` are accessors.
- `onLoadLocationTextureAtlas @ 0x1000302b0` forwards to the listener.
- `onButtonPress:touch: @ 0x1000302cc`, `onButtonRelease:touch: @ 0x1000302d0` and
  `onButtonReleaseOutside:touch: @ 0x1000302d4` are empty.

### 4.3 Battle of the Gods: overview

Battles happen in the God island temple (`TempleDoorLogic`, logic spec):
1. The player picks four selected idols.
2. The player picks an opponent episode: `onBattleEnemySelected:e @ 0x10002ed68` sets single-player mode,
   enemy = e, `initBattleOfTheGodsEnemy`, `[TempleDoorLogic startBattle]`.
3. Then rounds run until one side's health reaches 0.

Each round:
1. The player picks one of their idols (`onBattleIdolSelected: @ 0x10002dce4`).
2. The AI picks one of the enemy's.
3. Attack types and realms are compared, and `TempleDoorLogic playBattleSequence` animates the round.
4. The animation calls back, in order:
   - `PygmyLogic(BattleOfTheGods)` → `onBattleDamageComplete` / `onBattleAttackTie`;
   - `TempleDoorLogic` → `showBattleRealmResults`, `…OnIdol`, `showBattleRealmEffectResults`,
     `areBattleRealmEffectResultsComplete`, `onBattleTurnComplete`, `isBattleComplete`, the resurrect checks,
     `performResurrection` and `applyBattleExperience`.

Winning against episode N unlocks episode N+1 for battle (`unlockNextBattleIdol: @ 0x10002c8f8`, persisted
`BattleUnlock`). `nextBattleIdol: @ 0x10002ed2c` returns the following episode if it is battle-unlocked.
`isLastBattleIdol: @ 0x10002ed08` tests the last list entry.

**Is Battle of the Gods multiplayer?** Yes. Besides the single-player campaign against the 49 episode
line-ups, it has a full asynchronous two-player mode over Game Center turn-based matches (§4.5).

### 4.4 Battle of the Gods battle engine

`PlayerBattleState` is 0x250 bytes (`mPlayerBattleState`, `mEnemyBattleState`; offsets from the ivar type
encoding, checked against the accessors and asm):

| Off | Type | Field |
|---|---|---|
| 0x00 | NSString | `mName` (player: the battle pygmy's name label; enemy: idol name +0x40; multiplayer: `%@Alias`) |
| 0x08 | i32 | `mLevel` = Σ idol levels / 4 |
| 0x0c | f32 | `mHealth` |
| 0x10 | f32 | `mMaxHealth` |
| 0x18 | ptr | `mpPlayerStructXmlEpisodeNodeDef` (unused) |
| 0x20 | ptr | `mpBattleStructXmlEpisodeNodeDef`: this round's idol (a scratch copy) |
| 0x28 / 0x29 | bool | `mbWinAttack` / `mbWinRealm` |
| 0x2c / 0x30 / 0x34 | f32 | `mActualAttackDamage` / `mActualReflectDamage` / `mActualDrainHealth` (display) |
| 0x38 / 0x3c | f32 | `mTotalBoostDamage` (power 2) / `mAttackBoostDamage` (3) |
| 0x40 / 0x44 | f32 | `mAttackReflectDamage` (4) / `mAttackDrainHealth` (5) |
| 0x48 / 0x4c | f32 | `mRealmDamage` (0) / `mTieDamage` (1) |
| 0x50 / 0x54 | f32 | `mTotalDefense` (7) / `mAttackDefense` (6) |
| 0x58 | f32 | `mRealmHealth` (9) |
| 0x5c / 0x60 | f32 | `mTotalDrainLevelPoints` (13, never reset) / `mDrainLevelPoints` (13) |
| 0x64 / 0x65 / 0x66 | bool | `mbIgnoreAttackDefense` (8) / `mbCancelEnemyPowerUps` (14) / `mbUnfreezeIdols` (15) |
| 0x68 | f32 | `mAverageDamage` = ¼ Σ damage_per_level·(level+1) |
| 0x6c | bool | `mbHasFreezeIdol` (some idol has power 10) |
| 0x70 | i32[3] | `mAttackTypeCountArray` |
| 0x7c | i32[5] | `mRealmTypeCountArray` |
| 0x90 | `IdolBattleState`[4] | one per idol, 0x70 bytes each |

`IdolBattleState`:

| Off | Type | Field |
|---|---|---|
| 0x00 | i32 | `mActiveFrameCount`: turns its realm effect stays active |
| 0x08 | ptr | `mpStructXmlEpisodeNodeDef` (scratch copy) |
| 0x10 | id | `mIdolBattleRealmEffectLogic` (set by `IdolScreenLogic`) |
| 0x18 / 0x1c | f32 | `mDamage` (power 0) / `mHealth` (power 9): this turn's realm damage and heal |
| 0x20 | i32 | `mUsedCount`: rounds fought |
| 0x24 | f32 | `mExperience`: last battle's XP |
| 0x28 | i32 | `mFrozenRefCount`: number of enemy idols freezing it |
| 0x2c | bool | `mbFreezeIdol`: its freeze power is active |
| 0x30 | ptr | `mpFrozenIdolBattleState`: the enemy idol it froze |
| 0x38 / 0x3c | bool / f32 | `mbResurrect` / `mResurrectHealthPerLevel` (power 11) |
| 0x40 / 0x44 / 0x48 | f32 | `mPotentialAttackStrength` / `mPotentialRealmStrength` / `mPotentialStrength` (AI) |
| 0x4c–0x60 | i32×6 | enemy attack lose/win/tie and realm lose/win/tie counts against the four opposing idols (AI) |
| 0x64 / 0x68 | i32 | `mAttackTypeIndex` / `mRealmTypeIndex`: rank among same-type idols |
| 0x6c | f32 | `mChoiceScore`: AI weight |

Ghidra mistypes several fields. `mAverageDamage` (shown as bool) and `mPotentialStrength` (shown as int) are
f32 in the disassembly. The episode's selection slot +0xa0 (shown as float) is an i32 (`str w0, [x20,
#0xa0]` at `0x10002b9ec`).

Set-up:
- `initBattleOfTheGodsEnemy @ 0x10002f0fc`:
  1. Copy the first four enemy line-up episodes into `0x1007ce018 + i·0xe8`, with level = line-up level and
     slot = i. Copy the four selected idols into `0x1007cdc78 + i·0xe8`.
  2. Player name = `[[TempleDoorLogic battlePygmyLogic] pygmyNameLabel]`; enemy name = the episode's idol
     name.
  3. `initPlayerBattleState:` for both.
- `initPlayerBattleState:s @ 0x10002efd8`:
  1. Zero the battle idol, level, health, total drain, the cancel and unfreeze flags, the average damage and
     the has-freeze flag.
  2. Per idol: point it at its copy and zero frames, used count, damage, health, experience, frozen count,
     freeze flag and pointer, and resurrect flag and value.
  3. Sum the health and the level, and average the damage.
  4. Max health = health, level /= 4 (C division).
- `calculatePlayerLevel @ 0x10002ef50`, `calculatePlayerHealth @ 0x10002ef94`, `calculateEnemyLevel: @
  0x10002edc4` and `calculateEnemyHealth: @ 0x10002ee7c` give the same numbers for the selection screens
  (`struct-xml.md` §8.6).

Each round:
- `onBattleIdolSelected:e @ 0x10002dce4`: player battle idol = e, `aiSelectEnemyIdol`,
  `calculateBattleStateResults`, `[TempleDoorLogic playBattleSequence]`.
- `aiSelectEnemyIdol @ 0x10002d92c` (asm checked):
  1. Reset the enemy's type counts. For each enemy idol, record its rank among same-attack and same-realm
     idols (`mAttackTypeIndex`, `mRealmTypeIndex`; the counts are incremented afterwards). Also count its
     attack and realm lose/win/tie results against each player idol.
  2. `aiEvaluateIdolStrength:opponentBattleState:` both ways (below; the result does not affect the choice).
  3. Throw chance `c`: if `player.level ≤ enemy.level`, `c` = 20, then 15 if enemy level > 4, 10 if > 9,
     5 if > 14, and 0 if ≥ 20. Otherwise `c` = 0.
  4. `r1 = arc4random()`. For each enemy idol k: `w = 100 / attackTypeCount[k.attack]`. If
     `r1 % 101 < c` **and the player's current idol beats k on attack** (`attackResult(player.battle, k) == 1`),
     `w = 30000`, so the AI deliberately throws the round. `w = 0` if k is frozen (`mFrozenRefCount ≥ 1`).
     `mChoiceScore = w`.
  5. `r2 = arc4random()`. `x = (r2 mod ((int)(Σw·10) + 1)) / 10.0`. Walk the idols subtracting weights and
     stop at the first k where the remainder is ≤ 0, at most k = 3. A frozen idol 0 is still chosen when
     `x == 0`.
  6. Enemy battle idol = k's copy.
- `aiEvaluateIdolStrength:idolBattleState:opponentBattleState: @ 0x10002d768`, per idol with level L and
  turns T:
  - attack strength = `damage_per_level·(L+1)`;
  - realm strength = Σ over powers of a term in `v = base + L·per_level`:

    | Power | Term |
    |---|---|
    | 0, 9 | `T·v` |
    | 1 | `T·v/3` |
    | 2, 3, 5 | `T·self.avgDamage·v` |
    | 4, 6, 7 | `T·opp.avgDamage·v` |
    | 8 | `T·self.avgDamage·0.4` |
    | 10, 14 | `T·10·self.level` |
    | 11, 12 | `10·self.level` |
    | 13 | 0 |
    | 15 | `T·10·self.level` if the opponent has a freeze idol, else 0 |

  - potential = attack + realm (f32 at +0x48).
- `calculateBattleStateResults @ 0x10002d5a0`:
  1. `mAttackResult` and `mRealmResult` come from `attackResultFromPlayerIdol:enemyIdol: @ 0x10002cc34` and
     `realmResultFromPlayerIdol:enemyIdol: @ 0x10002cc84`: 0 enemy wins, 1 player wins, 2 tie.
  2. Set `mbWinAttack` and `mbWinRealm` on the winners.
  3. +1 `mUsedCount` on both battle idols.
  4. The realm winner's battle idol activates its realm effect (`activateRealmEffectInBattleState:opponentBattleState:
     @ 0x10002d55c`), unless the loser has `mbCancelEnemyPowerUps`.
  5. `accumulateRealmPowers` for both sides.
- `activateRealmEffectInBattleState:s idolBattleState:i opponentBattleState:o @ 0x10002d2ac`:
  1. `prev = i.frames`, then `i.frames = power_turn_count`.
  2. Only if `prev < 1` (newly activated), for each power:
     - 10 freeze: if `o.mbUnfreezeIdols`, frames = 0;
     - 12 power-up-all: `activateAllRealmEffectsInBattleState: @ 0x10002d4f4` (every idol of s), event
       `PowerUpAllIdols`;
     - 14 cancel: `s.mbCancelEnemyPowerUps = YES`, every idol of o gets frames = 0
       (`cancelAllRealmEffectsInBattleState: @ 0x10002d488`), event `CancelRealmEffects`;
     - 15 unfreeze: `s.mbUnfreezeIdols = YES`, `unfreezeIdolsInBattleState:opponentBattleState: @ 0x10002d4a4`
       (o's freezing idols lose frames, flag and target; s's idols' frozen counts become 0), event
       `UnfreezeIdols`.

  Events fire only when s is the player.
- `accumulateRealmPowers:s opponentBattleState:o @ 0x10002cf20`:
  1. Zero `+0x2c…+0x58`, `+0x60` and the three flags. Per idol, zero damage, health, freeze flag and
     resurrect flag.
  2. For each idol with frames > 0, and each of its powers with `v = base + per_level·level`:
     - 0: `mRealmDamage` and `idol.mDamage` += v;
     - 1: `mTieDamage` += v;
     - 2: `mTotalBoostDamage` += v (event `BoostTotalDamage`);
     - 3: `mAttackBoostDamage` (`BoostAttackDamage`);
     - 4: `mAttackReflectDamage`;
     - 5: `mAttackDrainHealth`;
     - 6: `mAttackDefense` (`ActivateAttackDefense`);
     - 7: `mTotalDefense` (`ActivateTotalDefense`);
     - 8: `mbIgnoreAttackDefense`;
     - 9: `mRealmHealth` and `idol.mHealth` += v;
     - 10: `idol.mbFreezeIdol` if frames > 1;
     - 11: `idol.mbResurrect`, `idol.mResurrectHealthPerLevel = v`;
     - 13: `mDrainLevelPoints` and `mTotalDrainLevelPoints` += v (`DrainLevelPoints`);
     - 14 / 15: the cancel / unfreeze flag if frames > 1;
     - 12 adds nothing.

     Events fire for the player only.
- `onBattleDamageComplete @ 0x10002e214` (`onBattleAttackTie @ 0x10002e9d8` calls it):
  `calculateBattleAttackResults`; if someone died, `onGameDeath` and `showBattleAttackResults`, else
  `showBattleAttackResults` and `updateIdolBattleScreen` on the screen logic.
- `calculateBattleAttackResults @ 0x10002df88`, with `pos(x) = max(x, 0)`:
  - **Tie:**
    - `E.health −= P.tieDamage · pos(P.totalBoost + 1 − E.totalDefense)` (event `TieDamage` if > 0);
    - `P.health −= E.tieDamage · pos(E.totalBoost + 1 − P.totalDefense)`.
  - **Win** (W = winner, L = loser):
    - `def = W.ignoreAttackDefense ? 0 : L.attackDefense + L.totalDefense`;
    - `dmg = W.idol.damage_per_level·(W.idol.level+1) · pos(W.totalBoost + W.attackBoost + 1 − def)`;
      `L.health −= dmg`, `L.actualAttack = dmg`;
    - `reflect = L.attackReflect · dmg · pos(L.totalBoost + 1 − W.totalDefense)`, `W.actualReflect = reflect`;
    - `drain = dmg · W.attackDrain`, `W.actualDrain = drain`;
    - `W.health = min(W.health − reflect + drain, W.maxHealth)`;
    - events `DrainHealth` (W = player, drain > 0) and `ReflectDamage` (L = player, reflect > 0).
  - Returns whether either health ≤ 0.
- `showBattleRealmResults @ 0x10002e468` and `showBattleRealmResultsOnIdol @ 0x10002e4b0` are UI forwards.
  `showBattleRealmEffectResults @ 0x10002e750` runs `handleIdolFreeze:opponentBattleState:` both ways, then
  the UI.
- `handleIdolFreeze:s opponentBattleState:o @ 0x10002e644`, per idol of s:
  - Freeze inactive and a target held: the target's frozen count −1, target cleared.
  - Freeze active and no target: target = `chooseFrozenIdolBattleState:o useRandomFrozenIdolIds:YES`; if
    found, count +1 and event `FreezeIdol` (player).
- `chooseFrozenIdolBattleState:o useRandomFrozenIdolIds:u @ 0x10002e520`: with `n = countFrozenIdols:o @
  0x10002e4f8` (frozen count > 0), if n ≥ 3, NULL.
  - Single player, or u = NO: the `(arc4random() % (4 − n))`-th idol of o whose frozen count is 0.
  - Multiplayer with u: the idol whose episode id equals the agreed `m{Player,Enemy}RandomFrozenIdolId`.
- `calculateBattleRealmEffectResult:i battleState:s opponentBattleState:o @ 0x10002e7e8`:
  - `dmg = i.mDamage · (s.totalBoost + 1 − min(o.totalDefense, 0.9))`; `o.health −= dmg` (event
    `RealmDamage`);
  - `s.health += i.mHealth` (event `RealmHealing`).

  `showBattleRealmEffectResult:… @ 0x10002e8fc` = this, then a UI update. The screen logic calls it per
  active idol.
- `checkBattleRealmEffectDeath @ 0x10002e940`: clamp both healths to max; if either is ≤ 0, `onGameDeath`.
- `areBattleRealmEffectResultsComplete @ 0x10002e9bc` asks the screen logic.
- `onBattleTurnComplete @ 0x10002df1c`: every idol of both sides with frames > 0 loses one
  (`completeTurnForRealmPowers: @ 0x10002d284`), then a UI update.
- `isBattleComplete @ 0x10002e9e8`: either health ≤ 0. `didPlayerWinLastBattle @ 0x10002ea24`: E ≤ 0 < P.
  `didPlayerTieLastBattle @ 0x10002ea60`: both ≤ 0.
- `shouldPlayerResurrect @ 0x10002ea9c` / `shouldEnemyResurrect @ 0x10002eae4`: health ≤ 0 and some idol has
  `mbResurrect`.
- `performResurrection @ 0x10002e378`: for each side that should, `performResurrection:s @ 0x10002e29c`. Each
  of its idols with frames > 0 and `mbResurrect` while health ≤ 0 sets
  `health = min(maxHealth·mResurrectHealthPerLevel, maxHealth)`, clears its resurrect flag and frames, and
  fires `ResurrectAfterDeath` (player). The screen is then told who resurrected.
- `onGameDeath @ 0x10002dd58`:
  1. Fatality callbacks to `TempleDoorLogic` for each dead side, then a UI update.
  2. If the player is alive and this is single player: events `WinBattle`, plus `DefeatDeva` if the enemy is
     Episode40, then `unlockNextBattleIdol:enemy`.
  3. If the enemy is alive: `LoseBattle`.

  A double KO gives neither event.
- `applyBattleExperience @ 0x10002eb2c`:
  - `xp = XP[enemy.level] · 4 · (P.totalDrainLevelPoints + 1) · 0.25`, or 0 if the player is dead. The XP
    table at `0x10029d46c` gives, for levels 0…19: 500, 1000, 1500, 1900, 2300, 2600, 2900, 3100, 3300, 3400,
    3500, 3600, 3700, 3800, 3900, 4000, 4100, 4200, 4300, 4400.
  - For each of the four player idols: `mExperience = xp`. Each level-up of its real episode gets
    `points = min((int)((int)(xp · max/190000) + points), max)`, and is saved. Then the status is updated with
    startup YES.

Name helpers (`0x10038a830`–`0x10038abd0`):
- `attackName: @ 0x10002c9fc`: Strong, Counter, Fast.
- `realmName: @ 0x10002ca30`: Oceanus, Aeris, Infernis, Animus, Pax.
- `attackResultName: @ 0x10002caa8` and `realmResultName: @ 0x10002cadc`: "Attack Enemy Won" … "Realm Tie".
- `realmPowerName: @ 0x10002cb10`: "Realm Damage" … "Unfreeze Idols".

`fillPowerDescriptionText:structXmlEpisodeNodeDef: @ 0x10002cd20` is in `struct-xml.md` §8.3.

Accessors:
- `playerBattleState @ 0x10002c830` and `enemyBattleState @ 0x10002c840` return pointers to the two 0x250 structs;
- `playerName @ 0x10002c850`, `enemyName @ 0x10002c860`, `playerHealth @ 0x10002c870`,
  `enemyHealth @ 0x10002c884`, `playerMaxHealth @ 0x10002c898`, `enemyMaxHealth @ 0x10002c8ac`;
- `activeBattlePlayerEpisodeIdol @ 0x10002c8c0` and `activeBattleEnemyEpisodeIdol @ 0x10002c8d4` (+0x20);
- `enemyEpisodeIdol @ 0x10002c8e8`.

### 4.5 Battle of the Gods multiplayer (Game Center)

**The flow.**
1. Matchmaking. The idol screen's multiplayer button calls `openMatchPanel @ 0x100033b24`:
   - Not available: alert `"Multiplayer Not Available:"` / `"Please log into Game Center or upgrade to iOS5."`.
   - Not authenticated yet: `mbOpenMatchPanelOnAuthenticationComplete = YES`. It is honoured by
     `onGameServerAuthenticationComplete @ 0x10003384c`.
   - Otherwise: `setActiveMatch:nil` and a `GKMatchRequest` (player group 2, two players, invite message
     `"Want to play Pocket God: Battle of the Gods?"`) shown in a `GKTurnBasedMatchmakerViewController`
     through the listener.
2. Picking a match. `turnBasedMatchmakerViewController:didFindMatch: @ 0x100033d44` dismisses the sheet and
   loads the match data. The block `FUN_100033de4` calls `parseMatchData: @ 0x100032d0c`, then
   `setActiveMatch:`.
   - `didFailWithError: @ 0x100033eb4`: alert `"Error Occured:"` / `"An error occured trying to load the
     match."`.
   - `playerQuitForMatch: @ 0x100033f40`: `cancelMatch:`, alert `"Player Quit:"` / `"Other player quit
     game."`.
   - `WasCancelled: @ 0x100033ff0`: clears the match.

Every alert goes through `[mIdolScreenLogic showEnemyPanelMultiplayerAlert:title message:text]`.

`setActiveMatch:m @ 0x1000321e0` branches on `m.status`. The alerts of statuses 0, 2 and 3 keep the match
object (asm `0x100032284`–`0x100032440`):

| Status | Condition | Result |
|---|---|---|
| 0 | — | multiplayer off; alert `"Unknown Result:"` / `"Try again.."` |
| 2 | — | multiplayer off; `"Battle Complete:"` / `"The Battle is complete."` |
| 3 | — | multiplayer off; `"Invite Sent:"` / `"Waiting for other player to accept invite."` |
| 1 | not exactly 2 participants, or someone's `matchOutcome == 1` (Quit) | if multiplayer was on: off, `closeScreenNow`, `[TempleDoorLogic quitGame]`. Then `cancelMatch:m`, alert `"Player Quit:"` / `"Other player quit game."` |
| 1 | `GameCategory` set and ≠ `BattleOfTheGods` | `"Not Battle of The Gods:"` / `"This match is not a Battle of the Gods Match,\nopen it in the Challenge friends screen."` |
| 1, their turn | no `RecipientAlias` | multiplayer off; `"Invite Sent:"` / `"Waiting for other player to accept invite."`; release the match |
| 1, their turn | my `CFBundleVersion` < `GameVersion` (`compare:options:NSNumericSearch`) | multiplayer off; `"Please Update Pocket God:"` / `"Opponent has a newer version of Pocket God\nthat is incompatible with your version."`; release |
| 1, their turn | otherwise | `mbMultiplayerMyTurn = NO`, `updateBattleOfTheGodsMultiPlayer` (spectate) |
| 1, my turn | no `ChallengerAlias` (new match: I challenge) | `m.message = "Battle of the Gods"`. Write `GameVersion` = my `CFBundleVersion`, `GameCategory`, `ChallengerAlias`, `ChallengerId`, `ChallengerIdol0…3Id` and `…Level` from my four slots. `endTurn`; the "Invite Sent" alert; release |
| 1, my turn | no `RecipientAlias` (I accept) | version check (older → update alert); `GameVersion` = mine (thus the higher); write `RecipientAlias`, `RecipientId`, `RecipientIdol0…3Id` and `…Level` |
| 1, my turn | then | version check again; `mbMultiplayerMyTurn = YES`, `updateBattleOfTheGodsMultiPlayer` |

The version is `CFBundleVersion` `"7"` in 1.48.2.

Per-round protocol. One "turn" is one idol choice; Game Center passes the turn back and forth.
- `updateBattleOfTheGodsMultiPlayer @ 0x1000313f4`:
  - **Entering** (multiplayer was off):
    1. Turn it on, clear the enemy episode.
    2. `initBattleOfTheGodsMultiPlayer @ 0x100030c60`:
       - The prefix is `Challenger` if `ChallengerId` is me, else `Recipient`; the enemy prefix is the other.
       - Load the eight idols from `%@Idol%dId` / `%@Idol%dLevel`.
       - Load the idol images into `EpisodeIdolSlot1–4` (player: `0x100569dc8`, `…d98`, `…d68`, `…d38`) and
         `5–8` (enemy: `0x100569d08`, `…cd8`, `…ca8`, `…c78`).
       - Names from `%@Alias`.
       - `initPlayerBattleState:` both.
    3. `loadMultiplayerBattleIdols @ 0x1000311b0` reads both `%@BattleIdolId`.
    4. `loadTurnGameState @ 0x100030ae0`: if `StateSaved == 1`, restore `%@RandomFrozenIdolId`, then for both
       sides `%@Health`, `%@MaxHealth`, `%@TotalDrainLevelPoints` and per idol `%@Idol%d` + `ActiveFrameCount`,
       `UsedCount`, `FrozenRefCount` and `FrozenIdolId` (resolved to the opposing idol with that episode id).
    5. If it is **not** my turn and both battle idols are recorded: replay the round the opponent resolved
       (`calculateIdolBattleMultiplayerResult @ 0x100031314`, which removes both ids and computes the
       results). Then the attack; if nobody died, freezes and realm effects. If the battle is over and nobody
       resurrects: multiplayer off and alert `"Battle Complete:"` / `"Waiting for other player to view
       results."`. Otherwise resurrections.
    6. `closeScreenNow`, `[TempleDoorLogic startBattle]`.
  - **Already in** and my turn: reload the battle idols and the state, update the screen. If
    `canShowIdolBattleMultiplayerResult @ 0x1000310b4` (both ids present and my turn),
    `showIdolBattleMultiplayerResult @ 0x100031848` (compute and `playBattleSequence`) and
    `closeIdolBattleSelectionPanel`; else `refreshIdolBattleSelectionPanel`.
- `onBattleIdolSelectedMultiplayer:e @ 0x100031920` (`IdolScreenLogic.c:4215`):
  - Player battle idol = e.
  - Opponent has not chosen (`<enemy>BattleIdolId` absent): if my turn, my turn = NO,
    `<player>BattleIdolId = e.id`, `saveTurnGameState`, `endTurn`. Returns NO.
  - Opponent has chosen: write my id, then pick both freeze targets locally
    (`chooseFrozenIdolBattleState:… useRandomFrozenIdolIds:NO`) and store their ids
    (`setPlayerRandomFrozenIdolId:enemyRandomFrozenIdolId: @ 0x10003188c`). My turn = NO;
    `saveTurnGameState`; `endTurn`. Then resolve locally: the enemy idol from its id, remove both ids,
    `calculateBattleStateResults`, `playBattleSequence`. Returns YES.
  - If a side has ≥ 3 frozen idols, `chooseFrozenIdol…` returns NULL and the code dereferences it, so the
    original crashes (§20).
- `saveTurnGameState @ 0x100030910`: `StateSaved = 1`; remove both `%@RandomFrozenIdolId`, then write the
  non-nil ones; `saveTurnPlayerBattleState:preferencePrefix: @ 0x100030668` for both sides (floats via
  `setPreference:asFloat:`, idols via `saveTurnIdolBattleState:preferencePrefix: @ 0x100030358`).
  `FrozenIdolId` is removed, then set to the frozen target's episode id.
- `loadTurnPlayerBattleState:… @ 0x1000307b4` and `loadTurnIdolBattleState:… @ 0x1000304e4` reverse it.
- `onIdolBattleSelectionPanelOpened @ 0x100030304`: in multiplayer on the opponent's turn,
  `loadActiveMatchData`.
- `isMultiplayerActive @ 0x100030338` and `isItMyTurn @ 0x100030348` are getters.
- `quitBattleOfTheGods @ 0x1000302d8` (`TempleDoorLogic.c:1096`) clears the flags and the match.
- `onMultiplayerGameOver @ 0x100031c48` (`TempleDoorLogic.c:1071`): multiplayer off. If it is my turn, set
  outcomes: enemy alive → me 3 (Lost), other 2 (Won); enemy dead and me alive → me 2 / other 3; both dead →
  4 / 4. Then `endMatchInTurnWithMatchData:` → `FUN_100031ed0` → `setActiveMatch:nil`.
- `endTurn @ 0x100034050`: `endTurnWithNextParticipant:<the other> matchData:[mMatchPreferences
  dataFromPreferences]`. On error, `FUN_10003424c` calls `closeScreenNow`, `[TempleDoorLogic quitGame]` and
  shows the alert `"Error Occured:"` / `"An error occured trying to load the match."`.
- `cancelMatch:m @ 0x100031f80`: every participant gets outcome 1 (Quit), then `endMatchInTurnWithMatchData:`
  → `FUN_100032130` → `setActiveMatch:nil`.
- `leaveMatch @ 0x100034354`: in turn, `participantQuitInTurnWithOutcome:3 nextParticipant:<other>
  matchData:`; out of turn, `participantQuitOutOfTurnWithOutcome:3`. Then `setActiveMatch:nil`
  (`FUN_1000345bc`). **No caller exists** (no selector reference in the decompilation).

Events:
- `handleTurnEventForActiveMatch:m @ 0x1000335c4`: if m is the active match, `loadActiveMatchData @ 0x100032d8c`
  when the screen mode is 7 (battle), else `mbLoadActiveMatchData = YES`. Returns YES. `loadActiveMatchData`
  clears the flag and loads the data into `parseMatchData:` + `setActiveMatch:`.
- `handleMatchEndedForActiveMatch: @ 0x100033748` logs and returns NO.
- `handleTurnEventNotificationForInactiveMatch: @ 0x1000336cc` and
  `handleMatchEndedNotificationForInactiveMatch: @ 0x1000337a8` call `showMatchAlert: @ 0x100033150`, which
  loads the data (`FUN_100033344`):
  - the opponent alias = (`ChallengerId` is me) ? `RecipientAlias` : `ChallengerAlias`;
  - someone quit → `showMatchAlertQuitMatch: @ 0x100032e90` (`"%@ quit the match"`);
  - an outcome of 2, 3 or 4 → `showMatchAlertMatchComplete: @ 0x100032f40` (`"Match against %@ is
    complete"`);
  - no `RecipientAlias` → `showMatchAlertInvite: @ 0x100032ff0` (`"%@ invited you to a match"`);
  - else `showMatchAlertYourTurn: @ 0x1000330a0` (`"Your turn against %@"`).

  Each ends in `[mBattleAlertLogic activateMatchAlert:match message:]`. **`GameServer` never calls the
  inactive-match handlers** (§3.3), so these alerts never appear in 1.48.2.
- `handleInviteFromGameCenter: @ 0x100032e8c` is empty.

Authentication:
- `onGameServerAuthenticationStarted @ 0x100033830` clears the flags.
- `onGameServerAuthenticationComplete @ 0x10003384c` works as in §5.3 (no `SameUser` variant), then the queued
  `openMatchPanel`.
- `finishGameCenterAuthentication @ 0x100033a3c`: in the battle screen, multiplayer, on the opponent's turn →
  `loadActiveMatchData`.
- `onGameCenterAuthenticationChanged @ 0x100033ab8`: in multiplayer, `closeScreenNow` + `quitGame`.
- `onGameServerAuthenticationViewControllerClosed @ 0x100033b20` is empty.

Match data is the same `BCPreferences` binary plist as the mini challenges. The keys are in
`audio-prefs-text.md` §2.6.5: `GameCategory = "BattleOfTheGods"`, `GameVersion`, `Challenger…` and
`Recipient…` `Alias`, `Id`, `Idol%dId`, `Idol%dLevel`, `BattleIdolId`, `RandomFrozenIdolId`, `Health`,
`MaxHealth`, `TotalDrainLevelPoints`, `Idol%dActiveFrameCount`, `…UsedCount`, `…FrozenRefCount`,
`…FrozenIdolId`, and `StateSaved`.

### 4.6 Port

`pg-game::managers::idol` ports §4.2–§4.4 unchanged:
- The battle engine is pure arithmetic on two structs. Port it as plain Rust with `f32` and the same
  operation order.
- Use a seedable RNG wrapper with an `arc4random`-style `u32` source, so the AI rolls and freeze targets can
  be tested.
- Keep the episode event queues and their quirk.

§4.5 needs a match service. The original's local game logic (round resolution split between the two
clients) already works with any transport that stores the match dictionary and passes the turn. A
**local hot-seat** service (§18.3) can drive both prefixes on one machine with the unchanged code paths.
Without it, `openMatchPanel` shows the "Multiplayer Not Available" alert, which is exactly what an iOS player
without Game Center sees.

## 5. ChallengeManager

Source `game/ChallengeManager.c` (77 methods, 1 class method, 21 block functions). The class has two jobs.
The first is the **mini-game score counter and personal bests**, active during normal play. The second is
**Mini Game Challenges**: a 1-minute attempt at a mini-game whose best score is sent to a friend through a
Game Center turn-based match, after which the friend tries to beat it.

Singleton `DAT_1007cd808`. `instance @ 0x100013aac`, `id @ 0x100013c18`,
`canReceiveCallbacksNow @ 0x100013c24` (YES), `setChallengeManagerListener: @ 0x100013c2c`
(the controller), `dealloc @ 0x100013c88`.

`init @ 0x100013ab8` does the following:
- creates the sequence and the arrays `mLogicArray`, `mChallengeUiLogicArray` and
  `mChallengeCreateUiLogicArray`;
- clears `mbGameCenterAuthenticationComplete` and `mbGameCenterNotAvailable`;
- registers as the **first** `GameServer` authentication listener.

### 5.1 Challenge types

A type is an index 0–15 (`mChallengeType`, `mActiveGameChallengeType`). The match data stores its
**GameId** string. The per-type getters read these tables (values read from the binary):
- `getChallengeId: @ 0x100014000` (CFStrings `0x100386ed0 + 0x20·i`);
- `getChallengeIconId: @ 0x100014128` and `getChallengeName: @ 0x100014254`;
- `getChallengeGameCenterGroupId: @ 0x100014380` (i32 table `0x10029d3a0`, = type + 3);
- `getGameUnitsName: @ 0x1000143a0`;
- `getLowerCaseChallengeName: @ 0x1000144c0`, `getChallengeText: @ 0x100014614` and
  `getChallengeUserMessage: @ 0x100014740`, **never called**;
- `getChallengeStartDescription: @ 0x10001486c` and `getChallengeRecapDescription: @ 0x10001498c`;
- `getCounterNumber: @ 0x1000145ec`, a C string: `"%d"`, or the table `0x10037d280[type − 4]` for
  types 4–15.

Every getter returns nil or `"%d"` for out-of-range types.

| Type | GameId | Name | Units name (`CounterDescription`) | Icon texture | Counter | GC player group |
|---|---|---|---|---|---|---|
| 0 | `1` | Coconut Bounce | Coconut Bounces | `AchievementIconCoconutbounce` | `%d` | 3 |
| 1 | `11` | Hurricane Zap | Hurricane Zaps | `AchievementIconHurricanezaps` | `%d` | 4 |
| 2 | `21` | Bird Bomb | Bird Bombs | `AchievementIconBirdbomb` | `%d` | 5 |
| 3 | `31` | Shark Snap | Shark Snaps | `AchievementIconSharksnaps` | `%d` | 6 |
| 4 | `41` | Ooga Jump | Ooga Jump Height | `AchievementIconOogajump` | `%d Oogles` | 7 |
| 5 | `51` | Bait Master | Piranhas Served | `AchievementIconPiranhaFeed` | `%d` | 8 |
| 6 | `933` | Shark Blast | Pygmies Blasted | `AchievementSharkBlasts` | `%d` | 9 |
| 7 | `2333` | T-Rex Hunt | Spear Throws | `AchievementIconTrex` | `%d` | 10 |
| 8 | `2503` | Spider Hunt | Spear Throws | `AchievementIconSpider` | `%d` | 11 |
| 9 | `3793` | Ice Monster Battle | Pygmies | `AchievementIceMonster` | `%d` | 12 |
| 10 | `6733` | Pain Drain | Pain Drain Depth | `AchievementPainDrain` | `%d Oogles` | 13 |
| 11 | `10064` | The Runs | The Runs Distance | `AchievementTheRuns` | `%d Oogles` | 14 |
| 12 | `18482` | Konkey Dong | Points | `AchievementKonkeyDong` | `%d` | 15 |
| 13 | `20753` | Moron Pests | Moron Pests Distance | `AchievementMoronPests` | `%d Oogles` | 16 |
| 14 | `30903` | Dance Dance Execution | Dance Dance Execution Points | `AchievementDanceExecution` | `%d` | 17 |
| 15 | `40000` | Call of Booty (the No Pee game) | Pieces of Booty | `AchievementLongTimeNoPee` | `%d` | 18 |

| Type | Start description | Recap (formatted with the best score) |
|---|---|---|
| 0 | You have 1 minute to bounce as many coconuts as you can! | You've scored %d coconut bounces! |
| 1 | You have 1 minute to zap as many pygmies in a hurricane as you can! | You zapped %d pygmies! |
| 2 | You have 1 minute to poo on as many pygmies as you can! | You pooed on %d pygmies! |
| 3 | You have 1 minute to tease as many sharks as you can! | You teased %d sharks! |
| 4 | You have 1 minute to jump as many Oogles as you can! | You jumped %d Oogles! |
| 5 | You have 1 minute to serve as many piranhas as you can! | You served %d piranhas! |
| 6 | You have 1 minute to blast as many pygmies as you can! | You blasted %d pygmies! |
| 7 | You have 1 minute to hunt the T-Rex! | Used %d spears! |
| 8 | You have 1 minute to hunt the Spider! | Used %d spears! |
| 9 | You have 1 minute to battle the Ice Monster! | Used %d pygmies! |
| 10 | You have 1 minute to dive as many Oogles as you can! | You dove %d Oogles! |
| 11 | You have 1 minute to travel as many Oogles as you can! | You ran %d Oogles! |
| 12 | You have 1 minute to score as high as you can! | You scored %d Points! |
| 13 | You have 1 minute to travel as many Oogles as you can! | You ran %d Oogles! |
| 14 | You have 1 minute to get as many points as you can! | You scored %d Points! |
| 15 | You have 1 minute to get as much booty as you can! | You collected %d booties! |

The three getters that nothing calls still return these strings. Note the quirk `"ooga jump:"` in the
lower-case name of type 4.

| Type | Lower-case name | Challenge text (`getChallengeText:`) | User message (`getChallengeUserMessage:`) |
|---|---|---|---|
| 0 | `coconut bounce` | How many Coconut Bounces can you get? | Just try to kick my butt at Coconut Bounces! |
| 1 | `hurricane zap` | How many Hurricane Zaps can you get? | Just try to beat me at Hurricane Zaps! |
| 2 | `bird bomb` | How many Bird Bombs can you get? | Just try to kick my butt at Bird Bombs! |
| 3 | `shark snap` | How many Shark Snaps can you get? | Just try to kick my butt at Shark Snaps! |
| 4 | `ooga jump:` | How high can you get on Ooga Jump? | Just try to kick my butt at Ooga Jump! |
| 5 | `bait master` | How many Piranha can you serve in Bait Master? | Just try to kick my butt at Bait Master! |
| 6 | `shark blast` | How many Pygmies can you blast in Shark Blast? | Just try to kick my butt at Shark Blast! |
| 7 | `t-rex hunt` | How many spears does it take to kill the T-Rex? | Just try to kick my butt at T-Rex Hunt! |
| 8 | `spider hunt` | How many spears does it take to kill the Spider? | Just try to kick my butt at Spider Hunt! |
| 9 | `ice monster battle` | How many pygmies does it take to kill the Ice Monster? | Just try to kick my butt at Ice Monster Battle! |
| 10 | `pain drain` | How deep can you go in the Pain Drain? | Just try to kick my butt in the Pain Drain! |
| 11 | `the runs` | How far can you go in The Runs? | Just try to kick my butt in The Runs! |
| 12 | `konkey dong` | How high can you get in Konkey Dong? | Just try to kick my butt in Konkey Dong! |
| 13 | `moron pests` | How far can you go in Moron Pests? | Just try to kick my butt in Moron Pests! |
| 14 | `dance dance execution` | How many points can get in Dance Dance Execution? | Just try to kick my butt in Dance Dance Execution! |
| 15 | `call of booty` | How many can you collect in Call of Booty? | Just try to kick my butt in Call of Booty! |

`getChallengeType:gameId @ 0x100013dd4` is the inverse of `getChallengeId:`. An unrecognised id gives
**17**, the "not recognized" marker.

### 5.2 Mini-game scoring and personal bests

State:
- `mbGameActive`;
- `mActiveGameChallengeType` and `mActiveGameScore`;
- `mChallengeScoreType`: 0 = higher wins; 1 = lower wins, set for types 7–9;
- `mBestChallengeGameScore`: the best of the current challenge attempt, −1 = none or forfeit.

The display uses these messages and texts:
- the logic messages `CountingMessageLogic` (`CountingMessage`) and `AchievementMessageLogic`
  (`AchievementMessage`);
- the dynamic texts `CounterDescription`, `CounterNumber`, `PersonalBest` and `AchievementScore`.

The controller and a few logics drive it (`controller.md` §4.11). The callers are
`PocketGodViewController(Game)`, `NoPeeLogic.c:535` (type 15), and `TRexLogic` / `SpiderLogic`
(`isGameActive:`). `IdolManager` asks `isAnyGameActive`.

- `checkStartGame:type @ 0x100018fe0`:
  1. If a game is active and its type is either `type` or a hunt (unsigned `active − 7 < 3`, i.e. 7, 8
     or 9), **return**: a running hunt cannot be replaced. Otherwise stop the active game (`stopGame:`).
  2. Set `mbGameActive = YES`, the type, score = 0, and score type = (type ∈ 7…9 ? 1 : 0).
  3. `CounterDescription` = units name. Unless type == 14, `CountingMessageLogic showMessage:@"MessageShowCount"`
     (`setAnimation AchievementShowCountAnim`).
  4. `CounterNumber` text buffer = `sprintf(getCounterNumber(type), score)`, then `showTextBuffer`.
- `setGameScore:type score:s @ 0x1000191cc`, `incrementGameScore:type @ 0x100019310` (+1) and
  `incrementGameScore:type score:n @ 0x100019480` (+n) work as follows:
  1. They are ignored while a challenge attempt is running for a different type
     (`(send || respond) && mChallengeType != type`).
  2. `checkStartGame:type`. If the active type is now `type`, update the score and redraw
     `CounterNumber` with `sprintf(counterFormat, score)` (asm `0x1000192c0`).
- `isAnyGameActive @ 0x100018f00` = `mbGameActive`. `isGameActive:t @ 0x100018f10` = active and the type
  matches.
- `stopGame:type @ 0x1000195f4`, only if active with this type:
  1. `mbGameActive = NO`. Unless type 14, `showMessage:@"MessageHideCount"` (`stall 4.0`,
     `AchievementHideCountAnim`, `onMessageComplete`).
  2. `[[BonusPackManager instance] checkBonusPackUnlock:type score:score]` (§8.2).
  3. Not in a challenge attempt: `checkAchievement:type score:score`, done.
  4. In an attempt, update the best score:
     - lower wins: `if (score < best && score != −1) best = score; else if (best == −1) best = score`;
     - higher wins: `if (score > best) best = score; else if (best == −1) best = score`.

     If the 61 s challenge timer is still running, return: the player may play again.
  5. Otherwise end the attempt. For a send attempt: clear the flag, `checkAchievement:type score:best`,
     then sequence `SubmitChallenge`. For a respond attempt: the same, with `SubmitChallengeResult`.
     Finally `[listener onEndChallenge:mChallengeType]`.
- `forfeitGame:type @ 0x1000198a4` follows the same path with score = −1 and **no** best update. Outside
  an attempt it calls `checkAchievement:score:−1`, which does nothing.
- `onMessageComplete:logic @ 0x10001a0ac`: when the challenge timer message finishes, set
  `mbChallengeTimerActive = NO`. If no game is running, end the attempt as in `stopGame:` step 5 (using
  `mActiveGameChallengeType`). If a game is running, its `stopGame:` ends the attempt later.

Personal bests:
- `checkAchievement:type score:s @ 0x10001896c`:
  1. Types 0–6 and 10–15 first send `[[IdolManager instance] triggerEpisodeGameScoreEvent:<event>
     score:s]` (§4.2).
  2. Then it calls `checkAchievement:id score:s message1:m1 message2:m2 highScoreWins:h`. `m1` is
     "You've achieved a personal best of" with h = YES, except for the hunts.

  | Type | Episode event | Achievement id (god persona key) | message1 | message2 | h |
  |---|---|---|---|---|---|
  | 0 | `CoconutBounceScore` | `CoconutBounces` | personal best | `%d Coconut Bounces!` | YES |
  | 1 | `HurricaneLighningScore` (sic) | `HurricaneLightning` | personal best | `%d Hurricane Zaps!` | YES |
  | 2 | `BirdBombScore` | `BirdBombs` | personal best | `%d Consecutive Bird Bomb Hits!!` | YES |
  | 3 | `SharkSnapScore` | `SharkSnaps` | personal best | `%d Consecutive Shark Snaps!` | YES |
  | 4 | `OogaJumpScore` | `OogaJump` | personal best | `a %d Oogle Jump!` | YES |
  | 5 | `BaitMasterScore` | `BaitMaster` | personal best | `%d Piranhas Served!` | YES |
  | 6 | `LaserBlastScore` | `LaserShark` | personal best | `%d Pygmies Blasted!` | YES |
  | 7 | — | `KillTRex` | `You've taken the TRex Down with` | `%d Spear Throws!` | NO |
  | 8 | — | `KillSpider` | `You've taken the Spider Down with` | `%d Spear Throws!` | NO |
  | 9 | — | `KillIceMonster` | `You've taken the Ice Monster Down with` | `%d Pygmies!` | NO |
  | 10 | `PainDrainScore` | `PainDrain` | personal best | `a depth of %d Oogles!` | YES |
  | 11 | `TheRunsScore` | `TheRuns` | personal best | `a distance of %d Oogles!` | YES |
  | 12 | `KonkeyDongScore` | `KonkeyDong` | personal best | `%d Points!` | YES |
  | 13 | `MoronPestsScore` | `MoronPests` | personal best | `a distance of %d Oogles!` | YES |
  | 14 | `DanceDanceExecutionScore` | `DanceDanceExecution` | personal best | `%d Points!` | YES |
  | 15 | `NoPeeScore` | `NoPee` | personal best | `%d pieces of booty!` | YES |

  Other types do nothing.
- `checkAchievement:score:message1:message2:highScoreWins: @ 0x1000187b8` also handles the controller's
  `Skewered` and `Bowled`. It ignores s == −1. With `best = [[listener godPersona] achievementCount:id]`,
  a new best is when `(h && best < s) || (!h && (s < best || best == 0))`. In that case:
  1. `PersonalBest` = message1 and `AchievementScore` = `[NSString stringWithFormat:message2, s]`.
  2. `[listener onAchievement:id count:s highScoreWins:h]`, which stores the best, submits Game Center
     tiers and the leaderboard (§3.4).
  3. `AchievementMessageLogic showMessage:@"MessageHighScore"` (`setAnimation
     AchievementGenericAnim`, `waitForAnimation`, `onMessageComplete`).

### 5.3 Mini Game Challenges (Game Center)

Screens:
- `openChallengeScreen @ 0x100018668`:
  1. `[[IdolManager instance] delayTriggerEpisodeEvent:@"OpenChallengeScreen"]`.
  2. `mbChallengeScreenActive = YES` and sequence `OpenChallengeScreen`: load `ScreenTextureAtlas0`,
     draw `ChallengeScreen`, show then remove `ChallengeLoading`, `updateChallengeCreatePanel`, add
     `ChallengeCreatePanel` at `ChallengePanel`.

  The controller calls `cantOpenChallengeScreen @ 0x100018644` instead
  (`ChallengeInProgressLogic showMessage:@"CantOpenChallengeScreen"`: `playSound Click`,
  `ChallengeOpenErrorAnim`) while a challenge runs.
- `closeChallengeScreen @ 0x100018718`: clears the error and screen flags, runs sequence
  `CloseChallengeScreen` (`loadLocationTextureAtlas`, draw `PocketGod`) and calls listener
  `onChallengeScreenClosed`.
- `updateChallengeType:t @ 0x1000185a4`: clamp t to [0, 15]; if it changed, store it and run
  `UpdateChallengeType` (`playSound Button`, `updateChallengeCreatePanel`).

Panel updates (each display group named after the panel):
- `updateChallengeCreatePanel @ 0x100017748`: `ChallengeTitle` = name, `ChallengeIcon` = icon texture.
- `updateChallengeStartSendScreen @ 0x100017870`: `ChallengeStartMessage` = start description,
  `ChallengeStartTitle` = `"You've challenged %@!"` (opponent alias), icon.
- `updateChallengeConfirmSendScreen @ 0x100017a04`: `ChallengeConfirmMessage` = `"Your challenge has been
  sent to %@."` (alias). `ChallengeConfirmTitle` = `"You quit and forfeited the game."` if the best is
  −1, else the recap formatted with the best score. Icon.
- `updateChallengeStartRespondScreen @ 0x100017be8`: start description, `"%@ has challenged you!"`
  (alias), icon.
- `updateChallengeResultsScreen @ 0x100017d7c`, with `c = match[ChallengerGameScore]` and `b = best`:

  | c | b | `ChallengeConfirmMessage` | `ChallengeConfirmTitle` |
  |---|---|---|---|
  | −1 | −1 | `You tied! The results have been recorded.` | `you: forfeit, %@: forfeit` (alias) |
  | −1 | ≥0 | `You won! …` | `you: %d, %@: forfeit` (b, alias) |
  | ≥0 | −1 | `You lost! …` | `you: forfeit, %@: %d` (alias, c) |
  | ≥0 | ≥0 | won if (score type 0 and b > c) or (type 1 and b < c), lost in the opposite case, tied otherwise | `you: %d, %@: %d` (b, alias, c) |

  The "…" stands for "The results have been recorded."
- `updateLoadingScreen @ 0x1000180a0`: the `ChallengeLoadingScreen` icon.
- `updateChallengeTimer @ 0x100018170`: `ChallengeTimer` / `Timer` = challenge name, plus the icon. The
  sequence item `countDownTimer` then overwrites `Timer` with the seconds left (`sequences.md`).
- `updateChallengeAlert @ 0x100018298` (`ChallengeAlert`: `AlertNum`, icon of `mNewChallengeType`) is
  **never called**.

Sequence items, each returning 2 unless stated:
- `updateChallengeCreatePanel::: @ 0x1000183e0`;
- `updateChallengeStartSendScreen::: @ 0x100018404` and `updateChallengeStartRespondScreen::: @
  0x1000184bc` (loading screen + panel);
- `showChallengeConfirmSendScreen::: @ 0x100018448` and `showChallengeResultsScreen::: @ 0x100018500`
  (set the screen flag, listener `onChallengeDialogOpen`, update);
- `loadLocationTextureAtlas::: @ 0x100018574`;
- `waitForSaveMatchDataComplete::: @ 0x100017304` returns 0 while `mbSavingMatchData`, then clears
  `mbStartingMatchFromViewController` and returns 2.

`setLogic: @ 0x10001a238` builds the logics from the boot script. Group `PocketGod` holds the
`MessageLogic`s: `AchievementMessageLogic` (`AchievementMessage`), `CountingMessageLogic`,
`ChallengeTimerLogic`, `ChallengeAlertLogic` and `ChallengeInProgressLogic`, each with this object as
listener. The buttons are:
- `ChallengeScreen/CloseButton`;
- `ChallengeCreatePanel/NextChallenge`, `PrevChallenge`, `ChallengeButton`;
- `ChallengeStartSendScreen/CancelButton`, `GoButton`;
- `ChallengeConfirmSendScreen/CloseButton`;
- `ChallengeStartRespondScreen/CancelButton`, `GoButton`;
- `ChallengeResultsScreen/CloseButton`, `RechallengeButton`;
- `ChallengeError/Go`;
- `SystemError/CloseButton`.

`onButtonPress:touch: @ 0x100019ab4` ignores every button while `mbStartingMatchFromViewController`:

| Button | Action |
|---|---|
| Close | `closeChallengeScreen` |
| Next / Prev challenge | `updateChallengeType:type ± 1` |
| Challenge (create) | `openMatchPanel` |
| Send: Cancel | `SendChallengeCancel`, listener `onChallengeScreenClosed` |
| Send: Go | best = −1, `mbChallengeTimerActive = YES`, `mbChallengeSendAttemptActive = YES`, `startChallengeGame:type` (stops another active type, sets type and score 0 without activating), `SendChallengeStart`, `ChallengeTimerLogic showMessage:@"MessageShowChallengeTimer"` (add `ChallengeTimer`, `countDownTimer 61.0`, then `MessageHideChallengeTimer`: remove it and `onMessageComplete`), `updateChallengeTimer`, listener `onStartChallenge:type` |
| Confirm: Close | listener `onChallengeScreenClosed`, `SubmitChallengeClose` |
| Respond: Cancel | `RespondChallengeCancel`, `onChallengeScreenClosed` |
| Respond: Go | as Send Go with `mbChallengeRespondAttemptActive` and `RespondChallengeStart` |
| Results: Close | `onChallengeScreenClosed`, `SubmitChallengeResultClose` |
| Results: Rechallenge | send screen flag, `SubmitChallengeResultRechallenge` (back to the send screen). The match was already ended and released, so the later `endTurn` fails with "Submit Challenge Fail" (see below) |
| Error: Go | `ChallengeCloseError` (back to the create panel) |
| System error: Close | clear the system-error flags, `ChallengeDialogErrorClose`, `onChallengeScreenClosed` |

Game Center flow. Match data is a `BCPreferences` serialised as a binary plist (`parseMatchData: @
0x1000158e4` → `loadPreferencesFromData:`, `dataFromPreferences`). The keys are listed in
`audio-prefs-text.md` §2.6.5. Mini challenges use `GameCategory = "MiniChallenge"`, `GameId`,
`ChallengerAlias`, `ChallengerId`, `ChallengerGameScore`, `RecipientAlias`, `RecipientId` and
`RecipientGameScore`.

Authentication:
- `onGameServerAuthenticationStarted @ 0x100014a94` clears both flags.
- `onGameServerAuthenticationComplete @ 0x100014b08`:
  - If `!isTurnBasedMultiplayerAvailable`: `mbGameCenterNotAvailable = YES`, done.
  - If not authenticated: `NotAvailable = YES`, then `onGameCenterAuthenticationChanged`.
  - If authenticated: `AuthenticationComplete = YES`. Same `playerID` as `mActivePlayerId` →
    `onGameCenterAuthenticationChangedSameUser`. Otherwise store the id and call
    `onGameCenterAuthenticationChanged`.

  Both are performed on the main thread, followed by `finishGameCenterAuthentication @ 0x100014d04`
  (empty). A queued `openMatchPanel` would follow when `mbOpenMatchPanelOnAuthenticationComplete` is set,
  but nothing sets it.
- `onGameCenterAuthenticationChangedSameUser @ 0x100014d08`: an in-flight submit is cancelled with
  `mbError`.
- `onGameCenterAuthenticationChanged @ 0x100014dbc`: `setActiveMatch:nil` and fail in-flight submits.

Matchmaking:
- `openMatchPanel @ 0x100014e38`:
  - Game Center not available, or not yet authenticated: `throwErrorMessage1:"Game Center not
    available." message2:"Check that you are logged in to Game Center and try again." message3:""`.
  - Otherwise: `setActiveMatch:nil`. Build a `GKMatchRequest` (player group = type + 3, min = max = 2
    players, `inviteMessage = "Want to play Pocket God: %@?"` with the challenge name when supported).
    Present a `GKTurnBasedMatchmakerViewController` through `[listener presentModalViewController:]`.
- Matchmaker delegate:
  - `turnBasedMatchmakerViewController:didFindMatch: @ 0x1000150dc`:
    1. `mbStartingMatchFromViewController = YES`, opponent alias `"Unknown"`, dismiss.
    2. `[match loadMatchDataWithCompletionHandler:]` → `FUN_1000151c0`: for the participant that is not
       me, a participant without `playerID` (auto-match not filled) gets the alias
       `"an Auto Match Player"`. Otherwise `GKPlayer loadPlayersForIdentifiers:` → `FUN_1000154d0`
       takes the alias of the first player.
    3. Then, on the main thread, `parseMatchData:data` and `setActiveMatch:match`.
  - `…didFailWithError: @ 0x1000156e4`: `setActiveMatch:nil`, the "Game Center not available." error,
    dismiss.
  - `…playerQuitForMatch: @ 0x1000157ac`: `setActiveMatch:nil`, `cancelMatch:match`, error
    `"Quit match confirmed."`, dismiss.
  - `…WasCancelled: @ 0x100015884`: `setActiveMatch:nil`, dismiss.

`setActiveMatch:m @ 0x100015a58` (retains m; nil ends here) branches on `m.status`:

| Status | Condition | Result |
|---|---|---|
| 0 unknown | — | error `Game Center Error.` / `Verify that you are logged into Game Center.` / `and have an internet connection.` |
| 1 open | `GameCategory` set and ≠ `MiniChallenge` | error `Not a Mini Game Challenge match.` / `Load this match in Battle of the Gods.` |
| 1 open | current participant ≠ me | error `Waiting for other player.` / `Other player has not taken their turn yet.` |
| 1 open | no `ChallengerId` (new match; I challenge) | write `GameCategory`, `GameId` (current type), `ChallengerAlias`, `ChallengerId`. `m.message = "%@ Challenge!"` (name). `saveMatchData`. Send screen |
| 1 open | no `ChallengerGameScore` (challenger re-opened) | if `mbSavingMatchData`: clear it and return (the turn event caused by our own save). Otherwise type from `GameId` (17 → error `Mini Game Challenge not recognized.` / `Make sure you are updated to the latest version of Pocket God.`, type 0). Send screen |
| 1 open | score present, no `RecipientId` (I respond, first time) | type from `GameId` (17 → error). Write `RecipientAlias`, `RecipientId`. `m.message = "%@ Challenge!"`. `saveMatchData`. Respond screen |
| 1 open | `RecipientGameScore` present | return (nothing to do) |
| 1 open | otherwise | saving guard as above, then type from `GameId` (17 → error). Respond screen |
| 2 ended | — | error `Match has ended.` / `Match has already ended.` |
| 3 matching | — | error `Finding a match.` / `Need more time to find a match for game.` |

Every error path clears `mbStartingMatchFromViewController`, shows `throwErrorMessage…` and releases
the match.
- "Send screen": `mbChallengeScreenActive = NO`, `mbChallengeStartSendScreenActive = YES`, sequence
  `SendChallengeDialog` (draw `PocketGod`, `updateChallengeStartSendScreen`, add `ChallengeLoadingScreen`,
  `waitForSaveMatchDataComplete`, remove it, add `ChallengeStartSendScreen`).
- "Respond screen": the same with `…StartRespond…` and `RespondToChallengeDialog`.

Match operations:
- `saveMatchData @ 0x1000165e0`:
  - No match: `mbError = YES`, `mbSubmittingChallenge = NO`.
  - Otherwise: `mbSavingMatchData = YES`, then
    `endTurnWithNextParticipant:<me> matchData:data completionHandler:FUN_100016848`. The iOS 5 idiom
    passes the turn to oneself to save. The block sets `mbError` on error.
- `endTurn @ 0x1000168dc`: `endTurnWithNextParticipant:<the participant that is not me> matchData:…`.
  Its block `FUN_100016b34` sets `mbError` on error and clears `mbSubmittingChallenge`. With no match it
  fails like `saveMatchData`.
- `endMatch @ 0x100016be4`, with challenger score c and recipient score r:
  - Outcome of me (the recipient) / the other: c < r → Won(2) / Lost(3); c > r → Lost(3) / Won(2);
    equal → Tied(4)/Tied(4). **The higher score wins even for the hunts** (types 7–9), while the local
    results text (above) honours the score type.
  - `m.message`: `"%@ - %@: %d to %@: %d"` (name, challenger alias, c, recipient alias, r). A −1 becomes
    `forfeit` in four format variants, `0x1003887b0`–`0x100388810`.
  - `endMatchInTurnWithMatchData:` with the block `FUN_10001704c`: error → `mbError`; clear
    `mbSubmittingChallengeResult`; `setActiveMatch:nil`.
- `cancelMatch:m @ 0x100016358`: every participant's `matchOutcome = 1` (Quit), then
  `endMatchInTurnWithMatchData:(data ?: [NSData data])`. Its block `FUN_100016530` logs and calls
  `setActiveMatch:nil`.
- `loadActiveMatchData @ 0x100015964`: `[mActiveMatch loadMatchDataWithCompletionHandler:FUN_1000159cc]`
  → `parseMatchData:`, then `setActiveMatch:` (same match).

Submitting:
- `submitChallenge::: @ 0x100017330`, a waiting item in sequence `SubmitChallenge`
  (`showChallengeConfirmSendScreen`, loading screen, `submitChallenge`, confirm screen). On its first
  call: `mbError = NO`, `mbSubmittingChallenge = YES`, `ChallengerGameScore = best`, `endTurn`, release
  the match. It waits for `mbSubmittingChallenge` to clear. On error it calls
  `throwDialogError:@"Submit Challenge Fail"`.
- `submitChallengeResult::: @ 0x100017440`: the same with `RecipientGameScore`, `endMatch` and
  `"Submit Challenge Result Fail"`.
- `throwErrorMessage1:message2:message3: @ 0x100017550`: `mbChallengeErrorActive = YES`, set
  `Message1`–`3` of `ChallengeError`, then run sequence `ChallengeError` (remove `ChallengeLoading`, add
  `ChallengeError` at `ChallengePanel`).
- `throwDialogError: @ 0x100017698`: remember whether the results or confirm screen was open, clear
  both, `mbChallengeSystemErrorActive = YES`, then run `ChallengeDialogError` (remove the loading screen,
  add `SystemError` under `PocketGod`). The message argument is ignored.

Turn events (`GameServer` §3.3):
- `handleTurnEventForActiveMatch:m @ 0x100017120`: if `m.matchID` equals the active match's,
  `loadActiveMatchData` and return YES; else NO. Since `ChallengeManager` is the first listener, it sees
  every event first.
- `handleMatchEndedForActiveMatch: @ 0x100017244` and the two `…ForInactiveMatch:` handlers
  (`0x1000171f0`, `0x1000172a4`) only log and return NO.

The remaining members: `onButtonRelease:touch: @ 0x10001a0a4` and
`onButtonReleaseOutside:touch: @ 0x10001a0a8` are empty.

### 5.4 Port

- §5.1 and §5.2 are pure game logic and port unchanged (`pg-game::managers::challenge`), including:
  - the "hunts cannot be replaced" rule;
  - the 4 s count hide;
  - the jet-pack check;
  - the episode score events, which drive idol challenges (§4.2);
  - the personal-best rules.
- §5.3 needs an opponent. Without Game Center the faithful choices are:
  - (a) keep the challenge screen but have Create show the "Game Center not available." error panel.
    This is exactly what an unauthenticated iOS player sees.
  - (b) a local **hot-seat** match service that implements the same match-data protocol (§18.3).

  Option (a) is the default. The 61 s timed attempt and the send/respond/results screens can only run
  under (b).

## 6. StoryManager

Source `game/StoryManager.c` (98 methods and 41 C++ helpers). Story mode: it plays the 23 built-in stories
and the player's own stories ("My List") with the island's pygmies as actors, and provides the story editor.
`StoryManager` has **no singleton**; the controller owns it (`PocketGodViewController.c:1275` creates it and
calls `loadStoryDefs`). `init @ 0x100048238`, `id @ 0x1000485f0`,
`setStoryManagerListener: @ 0x1000485fc`, `dealloc @ 0x100048658`, `canReceiveCallbacksNow @ 0x100048938`
(YES).

### 6.1 Responsibilities

- Load the built-in stories (bundle `story.xml`) and the user stories (`Documents/StoryMap` plus one file per
  story), and save every edit immediately.
- Run the story screen: Watch (built-in list), My List, and Create (the editor), with an option menu, an add
  menu, text entry and confirm dialogs.
- Play a story: drive the controller's world through listener callbacks, and draw the title, caption and
  word-balloon overlays.

### 6.2 Data

C++ objects (vtables in `__const`). The element helpers are at `0x1000530b8`–`0x100053a7c`
(constructors, destructors, `clone`, `isEqual`, `toXml`), plus the `std::vector` helpers
`FUN_100053a7c` (insert), `FUN_100053d2c`/`FUN_100053e3c` (grow and push_back) and
`FUN_100053cdc`/`FUN_100053f4c`/`FUN_100053f9c`/`FUN_100053fec` (destructors).

| Object | Size | vtable | Fields (defaults from the parser and the add menu) |
|---|---|---|---|
| `BCStoryDef` | 0x28 | `0x10037d440` | +0x08 title (`"ENTER TITLE"`), +0x10 `NSDate*` date (nil for built-ins), +0x18 i32 (0, unused), +0x20 `std::vector<BCStoryElementDef*>*` |
| element base | — | — | +0x08 i32 type (0 environment, 1 caption, 2 character, 3 event), +0x0c f32 delay (5.0, `0x40a00000`) |
| `BCStoryEnvironmentElementDef` | 0x20 | `0x10037d460` | +0x10 location index (0), +0x14 weather (0), +0x18 time of day (0) |
| `BCStoryCaptionElementDef` | 0x18 | `0x10037d4a0` | +0x10 caption (`"ENTER CAPTION"`) |
| `BCStoryCharacterElementDef` | 0x28 | `0x10037d4e0` | +0x10 name (`"ENTER NAME"`), +0x18 dialog (`"ENTER DIALOG"`), +0x20 state id (`"Idle"`) |
| `BCStoryEventElementDef` | 0x18 | `0x10037d520` | +0x10 state id (`"Dance"`) |

`BCStoryDef` methods (free functions after `onButtonPress:touch:`):
- `FUN_10004fd3c(def, i)` erases element i and deletes it;
- `FUN_10004fdd8(def, e)` appends e;
- `FUN_10004fe2c(def, i)` moves element i one place later (erase, re-insert at i+1);
- `FUN_10004fee0(def, i)` moves element i one place earlier;
- `FUN_10004ff94(def)` clones the def: title and cloned elements, date nil.

`FUN_10004dce8(a, b)` compares two defs: equal titles, equal element counts, and the element vtable `+0x18`
`isEqual` on every pair.

State tables are pointer lists ending in NULL, of `BCStoryStateDef {NSString *id; NSString *displayName;
u32 flags}` (read from the binary). The option menus show the display names; the files store the ids.

Character states (`0x1007cc1b8`, 57 entries):

| # | id | display name | flags |
|---|---|---|---|
| 0 | `Idle` | Idle | `0x1` |
| 1 | `Happy` | Happy | `0x1` |
| 2 | `VeryHappy` | Very Happy | `0x1` |
| 3 | `Sad` | Sad | `0x1` |
| 4 | `Crying` | Crying | `0x1` |
| 5 | `Angry` | Angry | `0x1` |
| 6 | `VeryAngry` | Very Angry | `0x1` |
| 7 | `Confused` | Confused | `0x1` |
| 8 | `VeryConfused` | Very Confused | `0x1` |
| 9 | `Scared` | Scared | `0x1` |
| 10 | `FreakedOut` | Freaked Out | `0x1` |
| 11 | `Prophecy` | Prophecy | `0x1` |
| 12 | `Dance` | Dance | `0x1` |
| 13 | `Fart` | Gassy | `0x1` |
| 14 | `Sneeze` | Sneeze | `0x1` |
| 15 | `Wave` | Wave | `0x1` |
| 16 | `SkirtFall1` | Skirt Fall 1 | `0x1` |
| 17 | `SkirtFall2` | Skirt Fall 2 | `0x1` |
| 18 | `NosePick` | Pick Nose | `0x1` |
| 19 | `Scratch` | Scratch Butt | `0x1` |
| 20 | `Yawn` | Yawn | `0x1` |
| 21 | `ChestBeat` | Beat Chest | `0x1` |
| 22 | `FixHair` | Fix Hair | `0x1` |
| 23 | `Cold` | Cold | `0x1` |
| 24 | `Hot` | Hot | `0x1` |
| 25 | `Explode` | Explode | `0x1` |
| 26 | `Lasered` | Lasered | `0x1` |
| 27 | `KnockOffIsland` | Knock Off Island | `0x1` |
| 28 | `Flick` | Flick | `0x2000` |
| 29 | `FallInCrack` | Fall In Crack | `0x1` |
| 30 | `LocustMutate` | Locust Mutate | `0x1` |
| 31 | `JumpToDeath` | Jump Off Cliff | `0x200000` |
| 32 | `Waterwings` | Waterwings | `0x22000` |
| 33 | `DoubleRainbow1` | Double Rainbow OMG | `0x2000` |
| 34 | `DoubleRainbow2` | So Intense | `0x2000` |
| 35 | `DoubleRainbow3` | Whooa OMG OMG | `0x2000` |
| 36 | `DoubleRainbow4` | Full On Double RB | `0x2000` |
| 37 | `DoubleRainbow5` | Whaduzdismean? | `0x2000` |
| 38 | `DoubleRainbow6` | Triple RB | `0x2000` |
| 39 | `CharlieIdle` | Charlie Stare | `0x1` |
| 40 | `CharlieWinning` | Charlie Winning | `0x1` |
| 41 | `CharlieDoubleRainbow` | Charlie Rainbow | `0x1` |
| 42 | `CharlieImATotalFrickinRockStar` | Charlie RockStar | `0x1` |
| 43 | `CharlieItsOnBringIt` | Charlie Bring It | `0x1` |
| 44 | `CharlieOneSpeedOneGear` | Charlie One Speed | `0x1` |
| 45 | `CharlieTigerBlood` | Charlie Tiger Blood | `0x1` |
| 46 | `CharlieBiPolar` | Charlie BiPolar | `0x1` |
| 47 | `CharlieVaticanAssasinWarlock` | Charlie Warlock | `0x1` |
| 48 | `Sick` | Sick with Plague | `0x1` |
| 49 | `Barf` | Barf | `0x1` |
| 50 | `Cough` | Cough | `0x1` |
| 51 | `Sneeze2` | Sneeze with phlegm | `0x1` |
| 52 | `DieFromPlague` | Die From Plague | `0x1` |
| 53 | `Suspicious` | Suspicious | `0x1` |
| 54 | `CleanHands` | Clean hands. | `0x1` |
| 55 | `SprayPurell` | Spray PureHell. | `0x1` |
| 56 | `PurificationDeath` | Purification Death. | `0x1` |

Event states (`0x1007cc418`):

| # | id | display name | flags |
|---|---|---|---|
| 0 | `Dance` | Dance | `0x1` |
| 1 | `Earthquake` | Earthquake | `0x1` |
| 2 | `SharkSnap` | Shark Snap | `0x22000` |
| 3 | `ActivateRainbow` | Activate Rainbow | `0x2000` |
| 4 | `ActivateMyRainbow` | Activate My Rainbow | `0x2000` |
| 5 | `DeactivateRainbow` | Deactivate Rainbow | `0x2000` |

Locations (`0x1007cc5e8`):

| # | id | display name | flags |
|---|---|---|---|
| 0 | `SandIsland` | Sand Island | `0x23000` |
| 1 | `RockIsland` | Rock Island | `0x22004` |
| 2 | `GraveyardIsland` | Graveyard Island | `0x22040` |
| 3 | `IceIsland` | Ice Island | `0x2022000` |
| 4 | `ApeIsland` | Ape Mountain w/Altar | `0xa02000` |
| 5 | `ApeIslandTree` | Ape Mountain w/Tree | `0xa02000` |
| 6 | `MisfitIsland` | Misfit Island | `0x422000` |
| 7 | `GodIsland` | God Island | `0x22002` |
| 8 | `FailBoat` | Fail Boat | `0x22100` |
| 9 | `ApocalypseIsland` | Apocalypse Island | `0x22008` |
| 10 | `TimeTemple` | Time Chamber | `0x200a00` |
| 11 | `LocustTemple` | Locust Chamber | `0x200880` |
| 12 | `DanceTemple` | Dance Chamber | `0x200820` |
| 13 | `PlagueTemple` | Plague Chamber | `0x300800` |
| 14 | `PurificationTemple` | Purification Chamber | `0x1200800` |
| 15 | `ClosetTemple` | Closet Chamber | `0x200810` |
| 16 | `XenuTemple` | Xenu Chamber | `0x208800` |

Weather (`0x1007cc6a8`):

| # | id | display name | flags |
|---|---|---|---|
| 0 | `Clear` | Clear | `0x4000` |
| 1 | `Cloudy` | Cloudy | `0x10000` |

Time of day (`0x1007cc708`):

| # | id | display name | flags |
|---|---|---|---|
| 0 | `Day` | Day | `0x400` |
| 1 | `Night` | Night | `0x80000` |
| 2 | `Sunset` | Sunset | `0x40000` |

Lookups:
- `countStoryStateArray: @ 0x10004ac58` counts the entries up to the NULL terminator.
- `getStoryEnvironment{Location,Weather,TimeOfDay}StateDef: @ 0x10004ac6c/0x10004ac80/0x10004ac94` index
  the tables directly.
- `getStoryEventStateDef: @ 0x10004aca8` and `getStoryCharacterStateDef: @ 0x10004ad10` search by id and
  return NULL when nothing matches. Playback then dereferences NULL, so an unknown state crashes the original.

`isStoryStateDef:s compatibleWithEnvironmentElementDef:e @ 0x10004ad78` (asm checked) returns 0 when
compatible:
- 1 if `(s.flags & 0x3f2bbfe) & loc.flags != s.flags & 0x3f2bbfe`;
- else 2 if the same test fails for weather with mask `0x14000`;
- else 3 if it fails for time with mask `0xc0400`.

Only the bits `0x2000` (sky and ocean), `0x20000` (sharks) and `0x200000` (cliff or temple floor) occur in
state flags, so only the location test can fail. For example, `SharkSnap` and `Waterwings` need `0x22000`,
which the Ape locations and the temples lack.

File formats. All writes go through `[[listener systemPreferences] writeApplicationData:toFile:]` in
`Documents/`. Each file is the XML string encoded with `dataUsingEncoding:10` (UTF-16 with BOM) and then
**zlib-compressed** by `compressBlob` (`FUN_100047ee4`, `compress()` with the default level and an output
buffer of `2·len + 128`). The files are read back with `uncompressBlob` (`FUN_100048000`: `inflateInit`
with zlib 1.2.5, `Z_SYNC_FLUSH` until `Z_STREAM_END`). Both functions sit in `PygmyLogic+Apocalypse.c` in
the decompilation but are compiled from `StoryManager.mm`. NewsManager uses them too.

- Story file `story-<yyyyMMddHHmmss>` (`FUN_100048be0`: the date formatted `yyyy-MM-dd HH:mm:ss`, then `:`,
  `-` and space removed; a nil date is first set to now). The body comes from `FUN_100048a44` and the element
  `toXml` methods at vtable +0x20:
  ```
  <story date="yyyy-MM-dd HH:mm:ss">\n
      <storyTitle><![CDATA[%@]]></storyTitle>\n
      <storyEnvironment delay="%.1f" location="%@" weather="%@" timeOfDay="%@"/>\n      (FUN_100053268)
      <storyCaption delay="%.1f"><![CDATA[%@]]></storyCaption>\n                          (FUN_1000535c8)
      <storyCharacter delay="%.1f" state="%@">\n                                          (FUN_100053858)
          <storyCharacterName><![CDATA[%@]]></storyCharacterName>\n
          <storyCharacterDialog><![CDATA[%@]]></storyCharacterDialog>\n
      </storyCharacter>\n
      <storyEvent delay="%.1f" state="%@"/>\n                                             (FUN_100053a08)
  </story>\n
  ```
  Location strings come from `FUN_100053334`: index → id, with any out-of-range index written as `SandIsland`.
  Weather is `Cloudy` iff 1, else `Clear`. Time is `Night` iff 1, `Sunset` iff 2, else `Day`. Texts are not
  escaped: a `]]>` in user text breaks the file.
- `StoryMap` (`createStoryMapXml @ 0x100048da4`, `saveStoryMapXml @ 0x100048e8c`):
  `<storyMap>\n` + `    <storyMapEntry id="story-…"/>\n` per My List entry, in order, + `</storyMap>\n`.
- `loadStoryDefs @ 0x100048940` parses the bundle `story.xml` and then calls `loadStoryMapXml @ 0x100048f10`:
  1. Read and inflate `StoryMap` and parse it. Each `storyMapEntry@id` is collected into
     `mParseStoryXmlArray`.
  2. `loadStoryFromFile:id @ 0x100048d28` for each: read, inflate, parse. It returns `mpLastParsedStoryDef`.
     A missing or corrupt file is skipped (`mMissingStoryIdArray` is never filled).
  3. Clear the array.
- `saveStoryToFile: @ 0x1000489bc` writes one story.

Parser (`parseXmlFromData: @ 0x10005213c`, `parseXmlFromFile: @ 0x1000521dc`, NSXMLParser with external
entities on; `parserDidStartDocument: @ 0x100052294` clears `mpLastParsedStoryDef`;
`parserDidEndDocument: @ 0x1000522a8` and `parser:parseErrorOccurred: @ 0x100052ff0` only log):

- `parser:didStartElement:… @ 0x1000522a8`:
  - `storyMapEntry`: collect `id`.
  - `story`: a new `BCStoryDef`.
    - **No `date` attribute** (the built-ins): append it to `mpMainStoryDefVector`.
    - With a date (`FUN_10005299c`: `dateFromString` with `yyyy-MM-dd HH:mm:ss`): if a My List story has the
      same file name, the new def replaces it in place (and becomes the active def if the old one was). The
      old one is deleted. Otherwise the new def is appended to `mpMyListStoryDefVector`.
  - `storyTitle`, `storyCharacterName`, `storyCharacterDialog`: set their text flag.
  - `storyEnvironment`: a new element with the `delay` (`floatValue`).
    - `location` goes through `FUN_100052a5c`, an exact match against the 17 ids; any other string leaves 0.
      The bundle uses `Sand Island`, `SANDISLAND` and `Sand`, which all become Sand.
    - `weather` is `Clear` → 0, `Cloudy` → 1, otherwise unchanged (0).
    - `timeOfDay` goes through `FUN_100052cb0`: `Day` 0, `Night` 1, `Sunset` 2, otherwise unchanged.
    - The element is appended.
  - `storyCaption`: a new caption with the delay; the caption flag is set; the element is appended.
  - `storyCharacter`: a new character with `delay` and `state`; appended.
  - `storyEvent`: a new event with `delay` and `state`; appended.
- `parser:foundCharacters: @ 0x100052d40`: the chunk, with the literal `\n` replaced by a newline,
  **replaces** the flagged field: title (+0x08), caption or character name (+0x10), or dialog (+0x18).
- `parser:didEndElement:… @ 0x100052e84` clears the flag or the current element pointer.

`assets/xml/story.xml` holds 23 stories (`CALL OF BOOTY` … `WELCOME TO POCKET GOD!`) with 43 environment,
63 caption, 590 character and 13 event elements. Several characters have an empty dialog
`<![CDATA[]]>`, used to place a pygmy silently. If the parser reports no characters for an empty CDATA,
the default `"ENTER DIALOG"` would survive and a balloon would appear (§20). The port must treat an
empty element as `""`.

### 6.3 State

The main ivars are:
- `mStoryScreenMode` / `mLastStoryScreenMode`: 0 watch, 1 my list, 2 create, 3 clear-confirm,
  4 edit-main-confirm, 7 playing;
- `mMainStoryIndex` / `mMyListStoryIndex`: the first row shown;
- `mpActiveStoryDef` and `mpTemplateStoryDef` (the built-in being cloned);
- `mActiveStoryElementIndex`: 0 = the title page, i = element i−1;
- `mpActiveStoryEnvironmentElementDef`: during playback, the last environment;
- `mPlayingStoryElementIndex` and `mStoryStallTime`;
- the option-menu state `mbOptionMenuOpen`, `mbWideOptionMenu`, `mOptionButtonCount`,
  `mStoryOptionMenuOptionIndex` and `mStoryOptionMode` (0 location, 1 weather, 2 time, 3 name,
  4 character state, 5 event);
- `mStoryEditTextType` (0 title, 1 caption, 2 name, 3 dialog);
- the flags `mbKeyboardOpen`, `mbAddMenuOpen`, `mbCaptionOpen`, `mbWordBalloonOpen` and
  `mbWordBalloonRight`.

Nothing is stored in preferences: the stories are the only persistent data.

### 6.4 Screens

- `openStoryScreen @ 0x100049298`:
  1. Both list offsets = 0, `mbStoryScreenActive = YES`.
  2. Register as a game-frame listener (`gameFrameOrder @ 0x100049470` = 50).
  3. `[[IdolManager instance] delayTriggerEpisodeEvent:@"OpenTheStoryScreen"]`.
  4. Sequence `OpenStoryScreen` (load `ScreenTextureAtlas0`, draw `StoryScreen`, `updateWatchPanel`, add
     `StoryWatchPanel` at `StoryPanel`).
- `closeStoryScreen @ 0x10004937c`: closes the option menu, then `checkForUnmodifiedStoryDef`, listener
  `onHideKeyboard`, unregister from frames, sequence `CloseStoryScreen` (`loadLocationTextureAtlas`, draw
  `PocketGod`), and listener `onStoryScreenClosed`.

Lists:
- `updateWatchPanel @ 0x100049548` (mode 0, `StoryWatchPanel`): rows 0–5 show `StoryTitle%d` = title, with
  `Arrow%d`, `BlackBox%d` (selectable) and `EditStory%dButton` (selectable) for stories
  `index … index+5`. The other rows are emptied and hidden.
  - `PrevStory` is shown iff index > 0, `NextStory` iff `index < count − 1`.
  - Paging is ±6, clamped: previous `max(index − 6, 0)`; next `min(index + 6, count − 1)`.
- `updateMyListPanel @ 0x100049e70` (mode 1, `StoryMyListPanel`): the same layout over the My List vector.

The Create panel works on the active element:
- `viewStoryElementDef:i @ 0x100049094`:
  1. `delayTriggerEpisodeEvent:@"CreateStory"`.
  2. Clamp i to [0, count] and make it active.
  3. Sequence `OpenStoryCreateTitlePanel` (i = 0), or `OpenStoryCreate{Environment,Caption,Character,Event}Panel`
     by the type of element i−1. Each runs `updateCreatePanel` and its panel update, then shows
     `StoryCreatePanel` at `StoryPanel` and the node panel at `StoryNode`.
- `updateCreatePanel @ 0x10004a79c` (mode 2, `StoryCreatePanel`):
  - `StepNumber` = `""` on the title page, else `"STEP %d of %d (ENVIRONMENT|CAPTION|CHARACTER|EVENT)"`
    with (i, count).
  - Previous-node is shown iff i > 0, next-node iff i < count.
  - Delete-node iff i > 1, so the first environment and the title can never be deleted.
  - Move-up (later) iff 1 < i < count; move-down (earlier) iff i > 2. The first element is pinned.
- `updateCreateTitlePanel @ 0x10004ae64` (`StoryMainNodePanel`): `StoryTitle`.
- `updateCreateEnvironmentPanel @ 0x10004aee4` (`StoryEnvironmentNodePanel`): `DelaySeconds` =
  `"%1.1f SECONDS"`; `Island`, `Weather` and `Time` = display names.
- `updateCreateCaptionPanel @ 0x10004b0a0`: `CaptionText` (wrap 288.889, 4 lines) and the delay.
- `updateCreateCharacterPanel @ 0x10004b1e4`: `PygmyName`, `Dialogue` (wrap 277.778, 2 lines),
  `PygmyState` display name, delay.
- `updateCreateEventPanel @ 0x10004b3a4`: `EventState` display name, delay.
- `updateClearConfirm @ 0x10004b4d8` (mode 3) and `updateEditMainStoryConfirm @ 0x10004b5b4` (mode 4):
  `StoryConfirmPanel/Message` = `"Remove Story From Your List?"` or
  `"Edit Story?\nThe edited version will be located in your story list."`, with `ButtonCancel` shown.
- `updateAddMenu @ 0x10004b690` is empty.

Option menu:
- `getStoryOptionMenuStateArray @ 0x10004b694` gives the list per mode: locations, weather, times, the
  name list, character states or events.
- `getStoryOptionMenuActiveIndex @ 0x10004b72c` gives the current value's index, or −1. Names are matched
  on their display string, up to 100 entries.
- `updateOptionMenu @ 0x10004b994` (`StoryOptionMenu` with 15 rows, or `StoryWideOptionMenu` with 10):
  `NextPage`/`PrevPage` are shown like the debug screen (§12). Rows `Option%dButton`/`BlackBox%d` show the
  display names from the page offset on. `SelectionBox` is moved onto the active row and shown, or stays
  hidden.
- `updateStoryOptionMenuPage:±1 @ 0x10004d51c`: `offset += rows × d`. If `offset ≥ count`, then
  `offset = (count / rows) × rows`. Clamped at 0. Then it reopens the menu sequence.
- `createStoryCharacterNameArray @ 0x10004cf50`:
  1. Fill an `NSMutableDictionary` with the tribe's pygmy names (`[[listener tribePersona]
     pygmyPersonaArray]`), then `OOGA`, `BOOGA`, `KLIK`, `KLAK`, `DOOBY`, `NOOBY`, then every non-empty
     character name in the active story.
  2. Emit `"[NEW NAME]"` first, followed by the dictionary keys in **`NSDictionary` enumeration order**
     (hash order), at most 99 more, into the static array `DAT_1007e4400` (records `DAT_1007e4720`).
- `selectStoryOptionIndex:i @ 0x10004dda8`, by mode:
  - 0, 1, 2: store the index into the environment and run `UpdateStoryCreateEnvironmentValue`.
  - 3 (name): `"[NEW NAME]"` opens the keyboard (type 2, `PygmyName`, cancel object = cancel-edit-name,
    **max 10**, preventEmpty, sequence `StoryOptionMenuEditPygmyName`) unless it is already open. Any other
    choice stores the name and runs `UpdateStoryCreateCharacterValue`.
  - 4: store the state id → `UpdateStoryCreateCharacterValue`.
  - 5: store the event id → `UpdateStoryCreateEventValue`.

  Every change is saved at once with `saveStoryToFile:`.

Delays:
- `updateStoryCreate{Environment,Caption,Character,Event}Delay:±1 @ 0x10004cc80/0x10004cd70/0x10004ce60/0x10004d344`:
  if `delay < 1` or (`delay == 1` and decreasing), `delay += 0.5·d`, clamped at ≥ 0; otherwise
  `delay += d`. There is no upper limit.
- Each saves the story and runs `UpdateStoryCreate…Value`.

`onKeyboardTextEntered:t @ 0x10004d614`: type 0 → title; types 1 and 2 → field +0x10 (caption or name);
type 3 → dialog (+0x18). Then it saves.

The text limits passed to `onShowKeyboard:…` are:
- title: 30 characters, preventEmpty, field `StoryMainNodePanel/StoryTitle`;
- caption: 150, preventEmpty;
- dialog: 60, empty allowed;
- name: 10, preventEmpty.

`onButtonPress:touch: @ 0x10004e218`. Any button closes an open add menu first, with sequence
`StoryAddMenuClose`.

| Button | Action |
|---|---|
| Add menu: Environment / Caption / Character / Event | insert a new default element at vector position `p = (i == 0 ? 1 : i)` (just after the shown element, never before the first one; asm `0x10004e888`–`0x10004e9cc`), save, `viewStoryElementDef:p+1` |
| Add menu: Delete node | if the story has elements: erase element `max(i − 1, 0)`, save, view the same index |
| Add menu: Close | `StoryAddMenuClose` |
| Close | `closeStoryScreen` |
| Tab Watch / My List | `mbKeyboardOpen = NO`, `checkForUnmodifiedStoryDef`, `onHideKeyboard`, `OpenStoryWatchPanel` / `OpenStoryMyListPanel` |
| Tab Create (ignored in mode 2) | as above, then a new story: title `"ENTER TITLE"`, one default environment. Append it to My List, `saveStoryMapXml`, `viewStoryElementDef:0` |
| Watch row r | `playStoryDef:main[index + r]` |
| Watch edit r | template = `main[index + r]`, `OpenStoryEditMainStoryConfirmPanel` |
| My List row r | `playStoryDef:mine[index + r]` |
| My List edit r | template = nil, element index 0, active = `mine[index + r]`, `viewStoryElementDef:0` |
| Prev / next story | ±6 as above, `UpdateStoryWatchPanel` / `UpdateStoryMyListPanel` |
| Create: Preview | `playStoryDef:active` |
| Create: Clear | `OpenStoryClearConfirmPanel` |
| Create: Next / prev node | hide the keyboard, `viewStoryElementDef:i ± 1` |
| Create: Move up / down | hide the keyboard, move element i−1 later (`FUN_10004fe2c`) or earlier (`FUN_10004fee0`), save, view i+1 or i−1 |
| Create: Add | `mbAddMenuOpen = YES`, `OpenStoryAddMenu` |
| Title: Edit | keyboard type 0, `StoryCreateTitleEditTitle` |
| Location / Weather / Time | wide option menu (10 rows), mode 0 / 1 / 2, `OpenStoryWideOptionMenu` |
| Caption: Edit | keyboard type 1 |
| Character: State | option menu (15 rows), mode 4, `OpenStoryOptionMenu` |
| Character: Edit name | option menu (15 rows), mode 3 |
| Character: Edit dialog | keyboard type 3, unless it is already open |
| Event: State | wide menu, mode 5 |
| Delay next / prev (four panels) | `updateStoryCreate…Delay:±1` |
| Any cancel-edit button | `mbKeyboardOpen = NO`, listener `onHideKeyboard` (the controller restores the text) |
| Confirm OK, mode 4 | active = clone of the template with date = now, appended to My List, `saveStoryMapXml`, element 0, view 0. The clone is saved only when edited |
| Confirm OK, mode 3 | `deleteActiveStoryDef`, `StoryClearConfirmOk` (back to My List) |
| Confirm Cancel | mode 4 → `StoryEditMainStoryConfirmCancel` (Watch), mode 3 → `StoryClearConfirmCancel` (Create) |
| Option row | close the menu, `selectStoryOptionIndex:offset + row` |
| Option menu Cancel | `StoryOptionMenuClose` / `StoryWideOptionMenuClose` |
| Option page next / prev | `updateStoryOptionMenuPage:±1` |
| Overlay close (during playback) | `onPlayStoryDefComplete:NO` |

Cleanup:
- `deleteActiveStoryDef @ 0x10004dae8` removes the active def from My List, deletes its file
  (`removeFile:`), saves the map and deletes the object. Active becomes nil.
- `checkForUnmodifiedStoryDef @ 0x10004dc18` runs whenever the editor is left. If the active story is an
  unmodified clone of its template, or still titled `"ENTER TITLE"` with fewer than 2 elements, it is
  deleted. Template = nil.

`setLogic: @ 0x100050080` creates the 83 logics named by the boot script (`setLogic` items with
`manager = StoryManager`). They are button logics plus the four `MultiModelLogic` overlays
`StoryOverlayTitleLogic`, `…CaptionLogic`, `…WordBalloonLogic` and `…FadeOutLogic`.
`loadLocationTextureAtlas::: @ 0x10004c148` and the `update…:::` items (`0x10004bf44`–`0x10004c124`) return 2.
`onButtonRelease:touch: @ 0x100050078` and `onButtonReleaseOutside:touch: @ 0x10005007c` are empty.

### 6.5 Playback

- `playStoryDef:def @ 0x10004d78c`:
  1. Mode 7 (previous mode saved), `delayTriggerEpisodeEvent:@"WatchStory"`, active = def.
  2. Fade-out overlay `StoryFadeOutClear`.
  3. `mStorySequence` runs `PlayStoryDef`, and the main sequence runs `StartStory` (`loadLocationTextureAtlas`
     twice, add `StoryOverlayDisplayGroup`, draw `PocketGod`).
  4. Listener `onStoryStart`. The controller hides the UI and freezes world logic (`controller.md` §4.10).
- `PlayStoryDef` = `playStoryDef:::` (`0x10004c178`: element index 0, the dynamic text `StoryTitle` = title,
  title overlay `StoryTitleInit` = `StoryTitleAnim`), then `runSequence StoryDefLoop`.
- `StoryDefLoop` = `playNextStoryElementDef::: @ 0x10004c218`:
  - Past the end: fade overlay `StoryFadeOutInit` (`StoryFadeOutAnim`, then event `FadeOutComplete`, which
    `onMultiModelEvent:eventId: @ 0x10004d434` turns into sequence `StoryDefComplete` → `storyDefComplete:::`
    → `onPlayStoryDefComplete:YES`).
  - Otherwise, by type:
    - 0 → `PlayStoryEnvironmentElementDef`;
    - 1 → `PlayStoryCaptionElementDef`;
    - 2 and 3: if the state is incompatible with the current environment (§6.2) →
      `PlayStoryIncompatibleElementDef`; else `PlayStoryCharacterElementDef` / `PlayStoryEventElementDef`.
- Element sequences (each ends with `storyElementDefComplete` (index + 1) and `runSequence StoryDefLoop`):

  | Sequence | Items |
  |---|---|
  | Environment | `playStoryEnvironmentElementDef` (`0x10004c52c`: remember the element, listener `onStoryEnvironmentChange:`), `storyStall`, `storyEnvironmentElementDefComplete` (2) |
  | Caption | `playStoryCaptionElementDef` (`0x10004c594`: if the text is non-nil and non-empty, `mbCaptionOpen`, `CaptionLine1` wrap 412.5, `StoryCaptionOpen`), `storyStall`, `storyCaptionElementDefComplete` (`0x10004c6b4`: if open, on the first call `StoryCaptionClose`; wait until `CaptionCloseComplete`) |
  | Character | `playStoryCharacterElementDef` (see below), `storyStall`, `storyCharacterElementDefComplete` (`0x10004c93c`: if a balloon is open, close it (`StoryWordBalloonCloseLeft/Right`) and wait for `WordBalloonCloseComplete`; otherwise, if the state ≠ `Idle`, `[pygmy resumeFromStoryModeCharacterState]`) |
  | Incompatible | `updateBadStateScreen` (`0x10004bd20`: `StoryBadStateScreen/ErrorText` wrap 360, `"The %@ action is not compatible with the %@ location. Please select a different action."`, or the `%@ weather` / `%@ time` variants, with the state's and the environment value's display names), add the screen, `stall 5.0`, remove it |
  | Event | `playStoryEventElementDef` (`0x10004ca30`) switches `mStorySequence` to `StoryModeEvent<state>` and returns 3 |
  | `StoryModeEventDance` / `…Earthquake` | `stallFrames 4`, `startCelebration`/`startEarthquake`, `storyStall`, stop, `stallFrames 4` |
  | `StoryModeEventSharkSnap` / `…ActivateRainbow` / `…ActivateMyRainbow` / `…DeactivateRainbow` | the item, `storyStall` |

- `storyStall::: @ 0x10004c47c`: on its first call it loads the element's delay. It returns 0 and subtracts
  `deltaTime` while the delay is > 0.
- Event items call the listener: `startCelebration::: @ 0x10004cb00` → `onStoryModeStartCelebration`; the
  others (`0x10004cb30`–`0x10004cc50`) → `onStoryModeStopCelebration`, `…StartEarthquake`,
  `…StopEarthquake`, `…SharkSnap`, `…ActivateRainbow`, `…ActivateMyRainbow`, `…DeactivateRainbow`.
- `playStoryCharacterElementDef::: @ 0x10004c714`:
  1. `pygmy = [listener onStoryAddPygmy:name]`; the controller finds or creates one, at most 6.
  2. If nil, skip. If `![pygmy isReadyForStoryMode]`, return 0 and wait.
  3. If the dialog is non-empty or the state ≠ `Idle`: `setStoryModeCharacterState:state`.
  4. If the dialog is non-empty:
     - `mbWordBalloonOpen`; `DialogueLine1` = dialog with wrap 104;
     - pygmy x ≥ 240 → `StoryWordBalloonOpenLeft` (`mbWordBalloonRight = NO`), else
       `StoryWordBalloonOpenRight`.
- `onGameFrame: @ 0x100049478`: while a balloon is open, copy the speaker's x and y to the balloon every
  frame. It looks the speaker up with `onStoryAddPygmy:`, which can create one.
- `updateBadStateScreen::: @ 0x10004c124` returns 2.
- `onPlayStoryDefComplete:completed @ 0x10004d8e4` (also the overlay close button with NO):
  1. Close an open caption or balloon.
  2. By the previous mode 0 / 1 / 2, run `StoryComplete…` / `CancelStory…` `FromWatchPanel` /
     `FromMyListPanel` / `FromCreatePanel`. Each refreshes the panel, removes the overlays, reloads
     `ScreenTextureAtlas0` and draws `StoryScreen`.
  3. **Always** listener `onStoryCancel`; `onStoryComplete` is never sent. Then `[mStorySequence stop]`.

### 6.6 Port

`pg-game::managers::story` holds the model, the XML codec (`quick-xml`) and the playback state machine.
- Keep the exact XML (including `%.1f` delays) so stories are interchangeable with the original format.
- Store them as plain XML or zlib + UTF-16 files: the original encoding is cheap with `flate2`, and keeping it
  lets iOS saves be imported.
- The editor UI stays in the game's own display groups. Only text entry uses the egui text field
  (`pg-ui`, §13.3).
- Use a deterministic name-list order (insertion order); see §20.

## 7. SkinPackManager

Source `game/SkinPackManager.c` (62 methods and 1 class method). Owns the 24 skin packs (paid cosmetic
packs), their keychain-backed purchase and selection state, the "Custom Pack" store screens and StoreKit.
The data structures and how a skin value is applied are specified in `textures.md` §7. The data itself is
in `assets/data/skin-packs.json`. Singleton `DAT_1007d1e80` (set in `init`); `instance @ 0x10005cac4`,
`id @ 0x10005db8c`, `setSkinPackManagerListener: @ 0x10005db98` (the controller).

### 7.1 Responsibilities

- Apply the selected skin values at boot and whenever the controller refreshes an option for a location
  or monster (`controller.md` §4.13).
- Answer skin queries:
  - `isSkinPackOptionActiveBySkinPackId:…` for the dance songs;
  - `getSkinPackOptionValueBySkinPackId:…` for `PygmyLogic(Island)::checkDanceChange:::`.
- Load the shared monster-atlas pages for non-skin screens (`loadTextureAtlasChannel:`).
- Run the store: list (5 per page), customize preview, purchase, success, failure, error and info panels.

### 7.2 State and persistence

The pack table is at `0x10037d620`: 24 records of `{BCSkinPackDef *pack; NSString *keychainKey}`
(0x10 bytes), in this order. It is also the list order on screen and the search order of
`getSkinPackDefById: @ 0x10005ce4c`:

| # | Pack id | Keychain account | Product id (`com.boltcreative.pocketgod.` + …) |
|---|---|---|---|
| 0 | AdultSwim | AdultSwimSkinPack | adultswimskinpack |
| 1 | Grimm | GrimmSkinPack | grimmskinpack |
| 2 | ZombiePack | ZombiePackSkinPack | zombiepack |
| 3 | MonsterPack | MonsterPackSkinPack | monsterskinpack |
| 4 | Dance3 | Dance3SkinPack | dancepack3 |
| 5 | Mondo | MondoSkinPack | mondoskinpack |
| 6 | Superbowl | SuperbowlSkinPack | superbowlskinpack |
| 7 | Dance2 | Dance2SkinPack | dancepack2 |
| 8 | Dance | DanceSkinPack | dancepack |
| 9 | Alien | AlienSkinPack | alienpack |
| 10 | Candyland | CandylandSkinPack | candylandskinpack |
| 11 | UnderwaterWorld | UnderwaterWorldSkinPack | underwaterworldskinpack |
| 12 | Mafia | MafiaSkinPack | mafiaskinpack |
| 13 | Shark | SharkSkinPack | sharkskinpack |
| 14 | Japanese | JapaneseSkinPack | japanesepack |
| 15 | Pimp | PimpSkinPack | pimpskinpack |
| 16 | Medieval | MedievalSkinPack | medievalpack |
| 17 | Fishmas | FishmasSkinPack | `FreeBee` (free; never sold) |
| 18 | Spider | SpiderSkinPack | spiderpack |
| 19 | Gamer | GamerSkinPack | gamerpack |
| 20 | IceMonster | IceMonsterSkinPack | icemonsterpack |
| 21 | TRex | TRexSkinPack | trexskinpack |
| 22 | Halloween | HalloweenSkinPack | halloweenskinpack |
| 23 | Christmas | ChristmasSkinPack | christmaspack |

`init @ 0x10005cad4`:
1. Creates the sequence, `mSkinOptionStateDictionary` (option id → `"%d"` of the applied value index)
   and `mSkinOptionChannelDictionary` (atlas channel → option id currently owning it).
2. Creates the UI arrays `mLogicArray`, `mSkinPackUiLogicArray`, `…List…`, `…Customize…`,
   `…Purchase…`, `…Success…`, `…Failure…`, `…Error…` and `…Info…`.
3. Calls `initSkinPackDefs`.
4. Sets `mbAllSkinPacksArePurchased` = every pack's +0x40 flag. It is computed once and never updated
   after a purchase.
5. `[[SKPaymentQueue defaultQueue] addTransactionObserver:self]`.

Initialising the packs:
- `initSkinPackDefs @ 0x10005d694`, for each pack:
  1. purchased = NO;
  2. keychain = `[BCKeyChain keyChainWithKeyUsingAppName:@"PocketGod" key:keychainKey]`, retained;
  3. purchased = `productId == @"FreeBee"` or the keychain secret is `@"BabaBooey"`.

  Then it calls `initSkinPackDef:` for every pack.
- `initSkinPackDef: @ 0x10005d7c8`: for every object (made current in turn) and every option, it sets
  the option keychain to `keyChainWithKeyUsingAppName:@"PocketGod" key:option.id defaultSecret:@"0"`.
  It then calls `updateSkinPackDef:pack skinOptionIndex:i skinOptionValueIndex:[secret intValue]
  silent:YES init:YES`. A new item's secret is `"THISISTHEUNSETSECRET"`, whose `intValue` is 0
  (`audio-prefs-text.md` §2.3), so the effective default is value 0. Finally current object = 0.

Persistence keys (all keychain, service `PocketGod`): `<Pack>SkinPack` = `"BabaBooey"` once bought;
`<option id>` = decimal value index. Option items are shared by every pack that has an option with
that id, so the selection is global per option id. Only a purchased pack writes it (`textures.md` §7.4).

Lookups and applying values:
- `getSkinPackDefByProductIdentifier: @ 0x10005cdd8` and `getSkinPackDefById: @ 0x10005ce4c` do a
  linear search of the 24 packs and return NULL when nothing matches. `refreshSkinPackOption:…` and the
  two query methods dereference that result without a check.
- Applying values is described in `textures.md` §7.4: `updateSkinPackDef:skinOptionIndex:skinOptionValueIndex:silent:init: @
  0x1000601e4`, `refreshSkinPackOption:skinObjectId:skinOptionId:useFirstPurchased: @ 0x10005d450`
  (the first *purchased* pack in table order that has the object and option, else the given pack),
  `refreshSkinPackOption:skinObjectId:skinOptionId: @ 0x10005d57c`,
  `clearUnpurchasedSkinPackDef:skinOptionIndex: @ 0x10006042c`,
  `clearUnpurchasedSkinOptionsFromSkinPackDef: @ 0x10005d94c` (every object and option of a pack;
  current object reset to 0) and `activateSkinPackDef: @ 0x1000605b0`.
- `getSkinPackOptionValueBySkinPackId:p skinObjectId:o skinOptionId:x @ 0x10005d14c` returns the id of
  the value i of option x for which `"%d"(i)` equals `state[x]`, else nil.
- `isSkinPackOptionActiveBySkinPackId:p skinObjectId:o skinOptionId:x skinOptionValueId:v @ 0x10005d2c4`
  finds the index i of value v, then compares `state[x]` with `"%d"(i)`.
- `skinOptionValueIndex:i @ 0x100060068` = current value index of option i of the active pack's current
  object.
- `updateSkinPackDef:skinObjectIndex:silent: @ 0x100060090` wraps the index (≥ count → 0, < 0 → count − 1).
  If it equals the current object and the call is not silent, it returns. Otherwise it sets the current
  object and gives the customize preview logic a behaviour: the behaviour (+0x10) of the current value of
  each option that has one (the last such option wins), else the object's behaviour (+0x08). If not
  silent, it starts sequence `UpdateSkinObject` (`playSound Button`, `updateCustomizePanel`).

Keychain maintenance:
- `resetKeyChainsFromSkinPackDef: @ 0x10005d9ec` resets the pack item and every option item.
- `resetKeyChainsForAllSkinPackDefs @ 0x10005da8c` does this for all packs. Nothing calls it.
- `releaseKeyChainsFromSkinPackDef: @ 0x10005dae4` releases the keychains (from `dealloc @ 0x10005dbf4`).

`loadTextureAtlasChannel:n @ 0x10005cec0` loads full pages into the two `MonsterAtlas` slots,
`atlasTRexGreen0` (`0x1004d8bf0`) and `atlasTRexGreen1` (`0x1004f0538`). It does nothing when
`channels[@"MonsterAtlas"]` already equals the tag. Otherwise it sets the tag and loads:

| n | Tag | Page 0 | Page 1 | Callers |
|---|---|---|---|---|
| 0 | `IconPage` | `IconPageTextureAtlas0` | `IconPageTextureAtlas0` | `IdolScreenLogic.c:1909` |
| 1 | `IdolPage` | `IdolPageTextureAtlas0` | `IdolPageTextureAtlas0` | `IdolScreenLogic` (3), controller God island / idols (`PocketGodViewController.c:10388`, 13258, 13291, 15493) |
| 2 | `Disco` | `DiscoTextureAtlas0` | `DiscoTextureAtlas1` | controller Dance temple (`:10455`) |
| 3 | `Xenu` | `XenuTextureAtlas0` | `XenuTextureAtlas1` | controller Xenu temple (`:10481`) |

The sequence item `loadTextureAtlasChannel:::` of the controller (`0x10007d3bc`) forwards to it. A
later `refreshSkinPackOption` for a `MonsterAtlas` option re-owns the channel, because the option id
differs from the tag.

### 7.3 Screens and purchase flow

`mSkinPackScreenMode`: 0 list, 1 customize, 2 purchase, 3 success, 4 failure, 5 info, 6 error.
The list offset `DAT_1007d1e78` is a global: the index of the first pack shown, kept across openings.
Logic ivars come from the boot script's `setLogic` items (`setLogic: @ 0x1000611d4`; every one is a
`BCButtonLogic` with this object as listener, except `SkinPackCustomizeItemLogic`, a `MultiModelLogic`):

| Group | Logic id → display object |
|---|---|
| `CustomPackScreen` | `SkinPackCloseButtonLogic` → `CloseButton`, `SkinPackInfoButtonLogic` → `InfoButton` |
| `CustomPackListPanel` | `SkinPackPrevButtonLogic` → `SkinPackPrevButton`, `SkinPackNextButtonLogic` → `SkinPackNextButton`, `SkinPackListItem0…4ButtonLogic` → `Blackbox0…4` |
| `CustomPackPanel` | `SkinPackCustomizeItemLogic` → `CustomItem` (preview), `…BuyButtonLogic` → `BuyButton`, `…Option0…3Next/PrevButtonLogic` → `PackOption0…3Next/PrevButton`, `…PrevPackButtonLogic` → `PrevPack`, `…NextPackButtonLogic` → `NextPack`, `…BackButtonLogic` → `BackButton` |
| `CustomPackBuyPanel` | `SkinPackPurchaseBuyButtonLogic` → `BuyButton`, `SkinPackPurchaseCancelButtonLogic` → `CancelButton` |
| `CustomPackBuySuccessPanel` | `SkinPackSuccessOkButtonLogic` → `OkButton` |
| `CustomPackBuyIncompletePanel` | `SkinPackFailureCancelButtonLogic` → `CancelButton` |
| `CustomPackErrorPanel` | `SkinPackErrorCancelButtonLogic` → `CancelButton` |
| `CustomPackInfoPanel` | `SkinPackInfoCloseButtonLogic` → `CloseButton` |

Open and close:
- `openSkinPackScreen:firstTime @ 0x10005e9a8`: `mbSkinPackScreenActive = YES`, then sequence
  `OpenSkinPackScreenFirstTime` or `OpenSkinPackScreen`. Both run: `playSound Button`, load
  `ScreenTextureAtlas0`, draw `CustomPackScreen`, clear `CustomPackPanel`, show `PurchaseProcessing` at
  `CustomPackLoadingLayer`, **`getProductData`**, remove `PurchaseProcessing`, then show the info panel
  (first time) or the list panel. The controller passes System `InfoShown == 0` as `firstTime` and then
  sets it to 1 (`controller.md` §4.8.4).
- `closeSkinPackScreen @ 0x10005ea34`:
  1. `mbSkinPackScreenActive = NO`.
  2. `clearUnpurchasedSkinOptionsFromSkinPackDef:` for all 24 packs, which removes previews of unowned
     skins.
  3. Preview behaviour `SkinPackStopIdle` (`setAnimation BlankAnim`).
  4. Sequence `CloseSkinPackScreen` (`loadLocationTextureAtlas`, draw `PocketGod`).
  5. Listener `onSkinPackScreenClosed`.

StoreKit (the only StoreKit user in the game):
- `getProductData::: @ 0x10005e0ec` (waiting item):
  - On its first call: if `mbAllSkinPacksArePurchased` or `!isInternetReachable` it returns 2 at once
    (no request, no error). Otherwise it sets `mbError = NO`, `mbLoading = YES` and
    `mbLoadingProductData = YES`, and calls `requestProductData`.
  - While loading it returns 0. Then `mbLoading = NO`. On error it logs `"Loading Product Data Fail"` and
    switches to `OpenSkinPackScreenNoInternet` (whatever the cause). It returns 2.
- `requestProductData @ 0x10005ddf8`: an `SKProductsRequest` for the 23 product ids of the table above
  (all except `FreeBee`; `NSSet setWithObjects:` order in asm: adultswim … trex). Delegate = self.
- `productsRequest:didReceiveResponse: @ 0x10005dfa0`: `mProductArray = response.products` (retained).
  An empty array sets `mbError`. Clears `mbLoadingProductData` and releases the request. The code never
  reports a request failure (`request:didFailWithError:` is not implemented), so a failed request leaves
  the screen waiting until Close.
- Prices: every row and panel reads `[[mProductArray objectAtIndex:0] price]` (the **first product**,
  whichever pack it is), then `[price stringValue]`, then `getPriceString: @ 0x10005eb28`:

  | `stringValue` | Shown |
  |---|---|
  | `0.59` | `}0.59` |
  | `0.79` | `0,79 ~` |
  | `115` | `\|115` |
  | `1.1` | `1.10Fr` |
  | `1.19` | `$1.19` |
  | `1.29` | `$1.29` |
  | `10` | `$10.00` |
  | `6` | `6.00Kr` |
  | `7` | `7.00Kr` |
  | anything else | `$0.99` |

  (`}`, `~` and `|` are presumably the bitmap font's £, € and ¥ glyphs.) An empty but non-nil
  product array makes `objectAtIndex:0` throw.
- `purchaseSkinPack @ 0x10005e218`: for every product whose `productIdentifier` equals the active pack's
  product id, `[[SKPaymentQueue defaultQueue] addPayment:[SKPayment paymentWithProduct:p]]`. If no
  product matches (or there is no product array), nothing is queued.
- `purchaseSkinPack::: @ 0x10005e8a4` (waiting item of sequence `PurchaseSkinPack`):
  - On its first call: `mbError = NO`, `mbLoading = YES`, `mbPurchasingSkinPack = YES`,
    `purchaseSkinPack`.
  - It returns 0 until `mbPurchasingSkinPack` clears. If nothing was queued, that never happens; only
    Close still works.
  - Then `mbLoading = NO`. On error it logs `"Purchasing Skin Pack Fail"` and switches to
    `OpenSkinPackScreenFailure`. Otherwise the sequence continues to the success panel.
- `paymentQueue:updatedTransactions: @ 0x10005e720`, for each transaction by `transactionState`:
  3 → `restoreTransaction:`, 2 → `failedTransaction:`, 1 → `completeTransaction:`. Purchasing (0) is
  ignored.
- `completeTransaction: @ 0x10005e604` and `restoreTransaction: @ 0x10005e4e8`: find the pack by product
  id, set its keychain secret to `"BabaBooey"` and purchased = YES, call `finishTransaction:`, and clear
  `mbPurchasingSkinPack`. The game never calls `restoreCompletedTransactions`, so restores only arrive
  unsolicited.
- `failedTransaction: @ 0x10005e3cc`: logs unless the error code is 2 (`SKErrorPaymentCancelled`),
  calls `finishTransaction:`, `mbPurchasingSkinPack = NO`, `mbError = YES`.
- `throwError: @ 0x10005ddb0` only logs. `purchaseSkinPackPanel::: @ 0x10006000c` (calls
  `purchaseSkinPack`, returns 2) and `setSkinPackPanel::: @ 0x100060030` (returns 2) are not used by any
  sequence.

Panels (each `update…` has a `:::` sequence item returning 2, `0x10005fec8`–`0x10005ffe8`):
- `updateListPanel @ 0x10005ec9c` (mode 0, `CustomPackListPanel`):
  1. `SkinPackPrevButton` is shown and selectable iff offset > 0. `SkinPackNextButton` iff offset + 5 < 24.
  2. For the 5 rows r with pack = table[offset + r]:
     - Past the end of the table: hide `PackName`, `Blackbox` (unselectable), `ViewButton`,
       `PriceText`, `BuyStatus` and `Purchased`.
     - Otherwise `PackName%d` = name, and `Blackbox%d` (selectable) and `ViewButton%d` are shown. Then:
       - `FreeBee`: `PriceText` = `"FREE!"` shown, `BuyStatus` shown, `Purchased` hidden.
       - Product 0 exists and the pack is not purchased: price text from `getPriceString:`, `BuyStatus`
         shown, `Purchased` hidden.
       - Otherwise: `PriceText` hidden and `""`, `BuyStatus` hidden, `Purchased` shown iff purchased
         (row logic checked in asm `0x10005f0d0`–`0x10005f188`).
- `updateCustomizePanel @ 0x10005f1bc` (mode 1, `CustomPackPanel`):
  1. `PriceText` and `99cents`: `"FREE!"` for FreeBee, otherwise the price if product 0 exists, else
     both hidden.
  2. `PackName` = name. The `NextPack` and `PrevPack` buttons are always hidden and unselectable.
     `PackOption0…3Text` = `""`, and all 8 option buttons are hidden.
  3. Option 0 next/prev (object cycling) is shown iff the pack has more than one object.
  4. `PackOption0Text` = current object id.
  5. For option i = 0…2 of the object: next/prev of option i+1 are shown when it has more than one
     value, and `PackOption<i+1>Text` = the id of its current value.
  6. `BuyButton` is shown and selectable iff not purchased.
- `updatePurchasePanel @ 0x10005f974` (mode 2, `CustomPackBuyPanel`): `PurchaseHeader`, `Purchase0`,
  `Purchase1` and `Purchase2` = buy text lines 0–3 (+0x10…+0x28). `PriceText` = the price of product 0,
  with no nil check.
- `updatePurchaseSuccessPanel @ 0x10005fb3c` (mode 3): `PurchaseSuccess1` = success text (+0x30).
- `updatePurchaseFailurePanel @ 0x10005fbcc` (mode 4) and `updateInfoPanel @ 0x10005f960` (mode 5) only
  set the mode. These panels are static art.
- Error panel (mode 6, `CustomPackErrorPanel`, objects `ErrorHeader` / `Error1` / `Error2`):

  | Method | `ErrorHeader` | `Error1` | `Error2` |
  |---|---|---|---|
  | `updateNoInternetPanel @ 0x10005fbe0` | `You aren't connected to the internet!` | `To make in-app purchases, you must have` | `an active internet connection.` |
  | `updateNoKeyChainPanel @ 0x10005fcd8` | `Something is wrong with your keychain services!` | `To make in-app purchases, you must have` | `a keychain for us to store your purchases.` |
  | `updateNotThreeOhPanel @ 0x10005fdd0` | `NO 3.0 OS` | `You need the latest 3.0 firmware to purchase Skin Packs` | `""` |

  The third is reachable only through the unused sequence `OpenSkinPackScreenNotThreeOh`.

`onButtonPress:touch: @ 0x1000607d8`. **Close** always works and calls `closeSkinPackScreen`. Every other
button is ignored while `mbLoading`.

| Button | Action |
|---|---|
| Info | mode 5 → `CloseSkinPackScreenInfo` (= `runSequence OpenSkinPackScreenList`), else `OpenSkinPackScreenInfo` |
| List next / prev | `updateSkinPackDefList:offset ± 5 @ 0x100060738`: clamp to [0, 19]; if changed, store it and run `UpdateSkinPackList` (`playSound Button`, `updateListPanel`) |
| Row r | `activateSkinPackDef:table[offset + r]`: re-apply every option silently (keychain index if purchased, else 0), object 0, `updateSkinPackDef:skinObjectIndex:0 silent:YES`, then `OpenSkinPackScreenCustomize` (`textures.md` §7.4) |
| Customize Buy | not reachable (`isInternetReachable` false) → `OpenSkinPackScreenNoInternet`; keychain `hasError` → `OpenSkinPackScreenNoKeyChain`; else `OpenSkinPackScreenPurchase` |
| Option0 next / prev | `updateSkinPackDef:skinObjectIndex:current ± 1 silent:NO`, then `updateSkinPackDef:… skinOptionIndex:0 skinOptionValueIndex:current silent:YES init:NO` |
| Option1/2/3 next / prev | for option k = 0/1/2: `i = current`; repeat `i ± 1` until `updateSkinPackDef:… skinOptionIndex:k skinOptionValueIndex:i silent:NO init:NO` returns YES. Values that need an unowned pack return NO and are skipped, and returning to the current index returns YES. |
| Next / prev pack | log only (`"CUSTOMIZE NEXT/PREV PACK BUTTON:"`); the buttons are never shown |
| Customize Back | `clearUnpurchasedSkinOptionsFromSkinPackDef:active`, `OpenSkinPackScreenList` |
| Purchase Buy | `PurchaseSkinPack` (`playSound Button`, show `PurchaseProcessing`, `purchaseSkinPack`, remove it, `updatePurchaseSuccessPanel`, show `CustomPackBuySuccessPanel`) |
| Purchase Cancel / Success OK / Failure Cancel | `OpenSkinPackScreenPurchaseHide` / `…SuccessHide` / `…FailureHide`, which all go back to the customize panel |
| Error Cancel | if `mbError`, close the whole screen; else `OpenSkinPackScreenErrorHide` (customize panel) |
| Info Close | `CloseSkinPackScreenInfo` |

The remaining members:
- `isInternetReachable @ 0x10005cd90` is §15.
- `loadLocationTextureAtlas::: @ 0x100060038` calls the listener.
- `onButtonRelease:touch: @ 0x1000611cc` and `onButtonReleaseOutside:touch: @ 0x1000611d0` are empty.

### 7.4 Port

The `ARCHITECTURE.md` policy is that every product is owned. With every pack owned from first launch:
- `purchased = true` for all 24 packs, `mbAllSkinPacksArePurchased = true`, and `getProductData` returns
  at once. No price, Buy or error panel can appear: every row shows `Purchased`, and Buy stays hidden.
- Selection still persists per option id in the port's keychain document (`audio-prefs-text.md` §2.7).
  Keep the `"THISISTHEUNSETSECRET"` → 0 rule, which is also what a missing key gives.
- `refreshSkinPackOption…useFirstPurchased` then always resolves to the first pack in table order that has
  the object and option. Shared options keep the same value order, so the result does not change.

The store screen stays a full port, including the code paths, but `pg-platform` gets no store backend:
`purchase` is unreachable. A `store` trait with a "nothing to buy" implementation keeps the state machine
honest if a different policy is ever needed (§18).

## 8. BonusPackManager

Source `game/BonusPackManager.c` (31 methods). Owns the single bonus pack, the "Comic Pre-Order"
Ooga Jump jet pack, together with its screen and code entry. Singleton `DAT_1007d1f08`.
`instance @ 0x1000caaa0`, `id @ 0x1000cac90`, `setBonusPackManagerListener: @ 0x1000cac9c` (the
controller), `dealloc @ 0x1000cae3c`.

### 8.1 Data

`BonusPackDef` (0x48 bytes). The only instance is at `0x100602c88`, and the list table at
`0x10037d930` holds just that one pointer (its next four slots hold `0`, `0`, `0x28` and a code pointer,
see 8.3):

| Offset | Type | Value |
|---|---|---|
| +0x00 | NSString | id `ComicPreOrder` |
| +0x08 | NSString | name `Pocket God Comic Pre-Order Bonus` |
| +0x10 | NSString | unlock code `journey to uranus` (compared with `caseInsensitiveCompare:`) |
| +0x18 | NSString | details URL `http://www.boltcreative.com/` |
| +0x20 | NSString | preview behaviour `BonusPackComicPreOrderPreview` (sequence: `setAnimation PygmyDoodleJumpJetPackBonusPageAnim`) |
| +0x28 | NSString | detail text (`0x1003cbd50`, 14 lines: the comic-store promotion, a $1000 Apple gift certificate drawing, "or achieve the 'God' score on Ooga Jump of 6000 Oogles") |
| +0x30 | i32 | unlock challenge type 4 (Ooga Jump, §5.1) |
| +0x34 | i32 | unlock score 6000 |
| +0x38 | BCKeyChain* | keychain item, filled at init |
| +0x40 | BOOL | unlocked |

### 8.2 State and persistence

- `init @ 0x1000caaac` creates the sequence and eight UI arrays: `mLogicArray`, `mBonusPackUiLogicArray`,
  and the `…List…`, `…Detail…`, `…Unlock…`, `…Info…`, `…Error…` and `…UnlockDialog…` arrays.
  It then calls `initBonusPackDefs`.
- `initBonusPackDefs @ 0x1000cad40`: `unlocked = NO`. The keychain is
  `[BCKeyChain keyChainWithKeyUsingAppName:@"PocketGod" key:@"ComicPreOrder"]`, retained.
  `unlocked = (secret == @"BabaBooey")`. A fresh item holds `"THISISTHEUNSETSECRET"`
  (`audio-prefs-text.md` §2.3).
- `isBonusPackUnlocked:id @ 0x1000cacf8` = `[@"ComicPreOrder" isEqualToString:id] && unlocked`.
- `resetKeyChainsForAllBonusPackDefs @ 0x1000cade8` resets the item and
  `releaseKeyChainsForAllBonusPackDefs @ 0x1000cae04` releases it. Nothing calls either.
- `checkBonusPackUnlock:type score:score @ 0x1000cb0c8` is called by `ChallengeManager::stopGame:`
  (§5.2) with the game type and its score. The condition is `type == 4 && !unlocked && score ≥ 6000`.
  When it holds:
  1. `mpUnlockBonusPackDef = def`, `unlocked = YES`, `setSecret:@"BabaBooey"`.
  2. Sequence `OpenBonusPackUnlockDialog` (`updateUnlockDialog`, then `addDisplayGroup
     BonusPackUnlockDialog` under `PocketGod`).
  3. `mbBonusPackUnlockDialogActive = YES` and listener `onBonusPackUnlockDialogOpened`.

  Returns whether it unlocked.
- Gameplay effect: `DoodleCloudLogic::moveToTop` (`DoodleCloudLogic.c:107`) may turn a new Ooga Jump
  cloud into type 5, the jet-pack cloud (scale 1.0). This needs `isBonusPackUnlocked:@"ComicPreOrder"`,
  a 1-in-5 roll, and the cloud logic's own spacing conditions. Nothing else reads the flag.

### 8.3 Screens

Panel modes in `mBonusPackScreenMode`: 0 list, 1 detail, 2 unlock (code entry), 3 info, 4 no keychain.
The logic ivars are created by the `setLogic` items of the boot script `PocketGod` (`setLogic: @
0x1000cc1c0`, params `displayGroupId`, `logicId`, `displayObjectId`). The table below gives the
item for each logic id. Every item makes a `BCButtonLogic` with this object as listener,
except the two `MultiModelLogic` previews (no listener):

| Logic id | Group / object |
|---|---|
| `BonusPackCloseButtonLogic`, `BonusPackInfoButtonLogic` | `BonusPackScreen` / `CloseButton`, `InfoButton` |
| `BonusPackListPrevButtonLogic`, `…NextButtonLogic` | `BonusPackListPanel` / `BonusPackPrevButton`, `BonusPackNextButton` |
| `BonusPackListItem0…4ButtonLogic` | `BonusPackListPanel` / `Blackbox0…4` |
| `BonusPackDetailPreviewItemLogic` | `BonusPackDetailPanel` / `PreviewItem` |
| `BonusPackDetailUnlockButtonLogic`, `…DetailButtonLogic`, `…BackButtonLogic` | `BonusPackDetailPanel` / `UnlockButton`, `DetailButton`, `BackButton` |
| `BonusPackUnlockCancelButtonLogic` | `BonusPackUnlockPanel` / `ButtonCancel` |
| `BonusPackInfoCloseButtonLogic` | `BonusPackInfoPanel` / `CloseButton` |
| `BonusPackErrorCloseButtonLogic` | `BonusPackErrorPanel` / `CancelButton` |
| `BonusPackUnlockDialogPreviewItemLogic`, `…CloseButtonLogic` | `BonusPackUnlockDialog` / `PreviewItem`, `CloseButton` |

Opening and closing:
- `openBonusPackScreen:firstTime @ 0x1000caf84`: `mbBonusPackScreenActive = YES`, then sequence
  `OpenBonusPackScreenFirstTime` (info panel) or `OpenBonusPackScreen` (list panel). Both load
  `ScreenTextureAtlas0` and draw `BonusPackScreen`. The controller passes `firstTime` from System
  `BonusInfoShown` (`controller.md` §4.8.4).
- `closeBonusPackScreen @ 0x1000cb010`: `mbBonusPackScreenActive = NO`, preview behaviour
  `BonusPackStopIdle`, sequence `CloseBonusPackScreen` (`loadLocationTextureAtlas`, draw `PocketGod`),
  then listener `onBonusPackScreenClosed`.

Panel updates (each also has a `:::` sequence item returning 2, `0x1000cb964`–`0x1000cba18`):
- `updateListPanel @ 0x1000cb238`: mode 0. Next and previous are hidden and unselectable. Row 0
  shows `PackName0` = def name, `Blackbox0` selectable and shown, `ViewButton0` shown, and `Unlocked0`
  shown only if unlocked. Rows 1–4 hide `PackName%d`, `Blackbox%d` (unselectable), `ViewButton%d`
  and `Unlocked%d`.
- `updateDetailPanel @ 0x1000cb4f8`: mode 1. `UnlockButton` is selectable and shown only while
  locked. `PackName` = name, `DetailText` = text, and the preview behaviour is the def's +0x20.
- `updateUnlockPanel @ 0x1000cb664`: mode 2. `Message` = `"Enter Code"`, `Code` = `""`,
  `mbKeyboardOpen = YES`, then listener
  `onShowKeyboard:self editingText:Code cancelEditDisplayObject:nil maxEditCharCount:30 preventEmpty:YES`.
- `updateInfoPanel @ 0x1000cb774`: mode 3 only (the info panel is static art).
- `updateNoKeyChainPanel @ 0x1000cb788`: mode 4. `ErrorHeader` = `"Something is wrong with your
  keychain services!"`, `Error1` = `"To unlock Bonus Packs you must have a keychain."`, `Error2` = `""`.
- `updateUnlockDialog @ 0x1000cb880`: `BonusPackText` = `[NSString stringWithFormat:@"You unlocked
  the %@\nBonus Pack!", def.name]`. The dialog preview plays the def's preview behaviour.

`onButtonPress:touch: @ 0x1000cbc14`:

| Button | Action |
|---|---|
| Close | If the keyboard is open: `mbKeyboardOpen = NO`, listener `onHideKeyboard`. Then `closeBonusPackScreen` |
| Info | Hide the keyboard as above. Mode 3 → `CloseBonusPackScreenInfo` (= `runSequence OpenBonusPackScreenList`), otherwise `OpenBonusPackScreenInfo` |
| Info close | `CloseBonusPackScreenInfo` |
| List previous / next | log only (`"BONUS PACK PREV:"` / `"BONUS PACK NEXT:"`) |
| List item 0 | `activateBonusPackDef:def @ 0x1000cb1c4`: `mpActiveBonusPackDef = def`, sequence `OpenBonusPackScreenDetail` |
| List items 1–4 | `activateBonusPackDef:` with table slots 1–4 (`0`, `0`, `0x28`, a code pointer; asm `0x1000cbfa0`–`0x1000cbfd8`). These rows are never selectable, so this cannot happen |
| Detail unlock | If `[def.keychain hasError]`: sequence `OpenBonusPackScreenNoKeyChain`. Otherwise `OpenBonusPackScreenUnlock` |
| Detail details | `[listener openUrl:def.url]` if non-nil |
| Detail back | `OpenBonusPackScreenList` |
| Unlock cancel | `mbKeyboardOpen = NO`, listener `onHideKeyboard`, `CloseBonusPackScreenUnlock` (detail panel) |
| Error close | `OpenBonusPackScreenErrorHide` (detail panel) |
| Unlock-dialog close | `mbBonusPackUnlockDialogActive = NO`, preview `BonusPackStopIdle`, `CloseBonusPackUnlockDialog`, listener `onBonusPackUnlockDialogClosed` |

`onKeyboardTextEntered:text @ 0x1000cba6c` (the controller forwards the text field contents on Return):
- Wrong code (`[text caseInsensitiveCompare:def.code] != NSOrderedSame`): `Message` =
  `"Incorrect Code"`. The keyboard reopens with the same parameters. The controller cleared
  `mbKeyboardOpen` in `textFieldDidEndEditing:` before calling back, so the reopen succeeds. `Code` keeps
  showing the wrong text until the next keystroke, because the hidden field has `clearsOnBeginEditing`
  (§13.3).
- Correct code: `mbKeyboardOpen = NO`, `unlocked = YES`, `setSecret:@"BabaBooey"`, sequence
  `CloseBonusPackScreenUnlock`. The detail panel then hides the unlock button. The unlock dialog is
  not shown in this path.

The remaining members:
- `loadLocationTextureAtlas::: @ 0x1000cba3c` calls the listener's `onLoadLocationTextureAtlas`.
- `onButtonRelease:touch: @ 0x1000cc1b8` and `onButtonReleaseOutside:touch: @ 0x1000cc1bc` are empty.

### 8.4 Port

`managers::bonus_pack` keeps the def, the screens and the code check (case-insensitive ASCII
compare).

Persistence:
- The keychain item becomes the `ComicPreOrder` entry of the port's keychain document
  (`audio-prefs-text.md` §2.7).
- `has_error` is always false, so the no-keychain panel is unreachable.

Policy. `ARCHITECTURE.md` names bonus packs explicitly among the content that "counts as owned from
the first launch". The port therefore starts with `unlocked = true`, as if the keychain held
`"BabaBooey"`. The consequences:
- Jet-pack clouds can spawn in Ooga Jump from the first game.
- The list row shows `Unlocked0` and the detail panel hides `UnlockButton`, so the code entry, the wrong-code
  path and the no-keychain panel cannot be reached.
- `checkBonusPackUnlock:score:` never fires, because it requires `!unlocked`, so the unlock dialog
  (`OpenBonusPackUnlockDialog`) never appears.

All of that code is still ported unchanged, so a different policy only needs a different initial flag.
The pack is a promotion unlocked by a code or a 6000-point Ooga Jump, not an App Store product. §20 records
the alternative of starting locked.

The details URL opens in the system browser.

## 9. NewsManager

Source `game/NewsManager.c` (37 methods and 7 C++ helpers). In-game news: a teaser pop-up at boot and a
paged news screen drawn on the map paper. Unlike the other managers it has **no singleton**: there is no
`instance`, and the controller owns the only object (`mNewsManager`). `init @ 0x100055444`,
`id @ 0x10005559c`, `setNewsManagerListener: @ 0x1000555a8`, `dealloc @ 0x100055604`.
`PocketGodViewController::loadGame` creates it, sets itself as listener and calls `loadNewsDefs`
immediately (`PocketGodViewController.c:1280`).

### 9.1 Data

`BCNewsDef` is a C++ object of 0x50 bytes with vtable `0x10037d600`:

| Offset | Type | Field | XML source |
|---|---|---|---|
| +0x00 | vptr | `0x10037d600` | — |
| +0x08 | NSString | id | `newsItem@id` (nil → `""`) |
| +0x10 | i32 | order | `newsItem@order` (`intValue`, absent → 0) |
| +0x14 | bool | wasRead | `newsItem@wasRead` (`boolValue`, only when present) |
| +0x18 | NSString | title | `newsTitle` text |
| +0x20 | NSString | tease | `newsTease` text |
| +0x28 | NSString | article | `newsArticle` text |
| +0x30 | NSString | imageUrl | `newsItem@imageUrl` (absent → `""`) |
| +0x38 | NSString | imageFileName (icon name) | `newsItem@imageFileName` (absent → `""`) |
| +0x40 | NSString | imageExt | `newsItem@imageExt` (absent → `""`) |
| +0x48 | NSString | linkUrl | `newsItem@linkUrl` (absent → `""`), or a matching `countryLink` |

`mpNewsDefVector` is a `std::vector<BCNewsDef*>`, created empty in `init`. The C++ helpers that
manage it:
- `FUN_100057cb0`: destructor body, releases the eight strings;
- `FUN_100057d50` and `FUN_100057d74`: complete and deleting destructors;
- `FUN_100057d98`: `vector::insert(pos, value)`;
- `FUN_100058054`: `__split_buffer` destructor;
- `FUN_1000580a4`: vector destructor.

`FUN_100055864` serialises one item for the cache (9.2). Other state:
- `mNewsDefIndex` and `mpActiveNewsDef`;
- `mbNewsScreenActive` and `mbNewsTeaserActive`;
- `mbOpenNewsTeaserOnLoadComplete`, set by `openNewsTeaserOnLoadComplete @ 0x100055a54` and never cleared;
- `mLoadingIconDictionary`, the icon names already requested;
- the parse flags `mbParseNewsTitle`, `mbParseNewsTease`, `mbParseNewsArticle` and `mbParseCountryLink`;
- `mpParsedNewsDef`.

### 9.2 Loading, merging and the cache

`loadNewsDefs @ 0x100055a68` (at boot and on every foreground, §2.3) runs three steps in order:
1. `loadNewsFromFile @ 0x100055964` reads `Library/Caches/news.xml`
   (`applicationDataFromLibraryCacheFile:`). It inflates it with `uncompressBlob` (`FUN_100048000`, §6.2)
   and runs `parseXmlFromData:`.
2. It parses the bundled `news.xml` (`pathForResource:@"news" ofType:@"xml"`,
   `parseXmlFromFile: @ 0x100057164`).
3. It calls `[[listener systemPreferences] loadDataFromServer:self listenerData:nil
   url:@"http://appdata.boltcreative.com.s3.amazonaws.com/news/news.xml"]` (`PTR_… @ 0x1004d1a40`,
   `audio-prefs-text.md` §2.2.6).

The parsers (`parseXmlFromData: @ 0x1000570c4`, `parseXmlFromFile:`) use `NSXMLParser` with
`shouldResolveExternalEntities = YES` and this object as delegate.
- `parserDidStartDocument: @ 0x10005721c` clears `mbParseCountryLink`.
- `parserDidEndDocument: @ 0x10005722c` is empty.
- `parser:parseErrorOccurred: @ 0x1000579ec` only logs.

`parser:didStartElement:… @ 0x100057230` (merge rule checked in asm):

- **`newsItem`**:
  1. Read the attributes.
  2. Look for an existing def whose id equals the attribute id (`[attrId isEqualToString:def.id]`, so a
     missing id attribute never matches). If found, remove it from the vector and reuse it, keeping its
     texts.
  3. Compute the insert index: `best = 0`; for each element i (after the removal),
     `if (elem.order < newOrder) best = i`. Then `index = best + 1`, clamped to the vector size, so an
     empty vector gives 0. An item whose order is not greater than any existing order therefore goes to
     index 1, not 0.
  4. If no def was reused, allocate a new one: zeroed, id = attribute or `""`.
  5. Assign the fields: `wasRead` only if the attribute exists; `order` always; the four URL and file
     fields are overwritten with the attribute or `""`.
  6. Insert at the index. The def stays the current parse target.
- **`newsTitle`, `newsTease`, `newsArticle`**: set the matching parse flag.
- **`countryLink`**: if its `id` attribute equals `[[NSLocale currentLocale]
  objectForKey:NSLocaleCountryCode]`, set `mbParseCountryLink`.

Other parser callbacks:
- `parser:foundCharacters: @ 0x10005773c`:
  1. `text = [chunk stringByReplacingOccurrencesOfString:@"\\n" withString:@"\n"]` (the two-character
     sequence backslash-n becomes a newline).
  2. Each flagged field is **replaced** by this chunk, not appended to, so the last chunk wins. The
     delegate has no `parser:foundCDATA:`, so CDATA arrives here.
  3. A pending country link replaces `linkUrl` with the chunk and clears its flag.
- `parser:didEndElement:… @ 0x100057924` clears the matching flag. `</newsItem>` clears `mpParsedNewsDef`.

Writing and the server callbacks:
- `saveNewsToFile @ 0x100055740` writes `<news>`, then `FUN_100055864(def)` for **the last 10**
  vector elements only, then `</news>`. The text is encoded as UTF-16 (`dataUsingEncoding:10`, with BOM),
  compressed with `compressBlob` (`FUN_100047ee4`) and stored with
  `writeApplicationData:toLibraryCacheFile:@"news.xml"`. The item format, with the strings and argument
  order from asm `0x100055864`:
  ```
  <newsItem order="%d" wasRead="%d" id="%@" imageUrl="%@" imageFileName="%@" imageExt="%@" linkUrl="%@">\n
      <newsTitle><![CDATA[%@]]></newsTitle>\n
      <newsTease><![CDATA[%@]]></newsTease>\n
      <newsArticle><![CDATA[%@]]></newsArticle>\n
  </newsItem>\n
  ```
  The values are not escaped. The arguments are +0x10, +0x14, +0x08, +0x30, +0x38, +0x40, +0x48, then
  +0x18, +0x20, +0x28.
- `onLoadDataFromServerComplete:listenerData url:data: @ 0x100057ab4`:
  - `listenerData == nil` (the feed): parse it, `saveNewsToFile`, then `openNewsTeaser` if
    `mbOpenNewsTeaserOnLoadComplete` is set and `hasUnreadNews`.
  - Otherwise `listenerData` is the `BCNewsDef` of an icon request. If `data.length ≤ 1024` it only
    logs. Otherwise it writes the data to `Library/Caches/<imageFileName>.raw` (`"%@.%@"` with
    `@"raw"`). If the active def still shows that icon, it re-uploads it from the cache: slot name =
    imageFileName, then `FUN_100038800`.
- `onLoadDataFromServerError:url:error: @ 0x100057c68` only logs.

Read state:
- `hasUnreadNews @ 0x1000559dc` = the vector is non-empty and **its last element** has
  `wasRead == NO`. No other element is checked.
- `markAsRead @ 0x100055a18` marks every def as read, then calls `saveNewsToFile`.

The bundled `news.xml` (`assets/xml/news.xml`) has 67 `newsItem`s, plus one commented out. The orders
run from 96 down to 4, and none has `wasRead`. 65 items carry `id`. Two, order 96 "Ooga Jump x10!"
and order 94 "Pocket God Toy Commercial!", carry `newsItemId` instead, so their id is `""`. A
simulation of the rules above over the bundled file gives:
- First launch: 67 defs; the last is order 94 "Pocket God Toy Commercial!", unread.
- Second launch: the cache's `id=""` items collapse into the first `""` def, and the two bundled
  `newsItemId` items are added again as new unread defs. That makes 68 defs; the last is order 96
  "Ooga Jump x10!", unread.

So **the teaser opens on every launch**. Within one run, every `loadNewsDefs` call (boot and each
return to the foreground) appends those two items again, because the vector is never cleared. The
cache only ever stores the last 10.

The news server is gone (the bucket answers `NoSuchBucket`). An archived 2019 bucket listing shows:
- `news/news.xml` (2014-06-19, 37270 bytes);
- 87 `.raw16` icons of 8192 bytes (64×64 RGBA4444, the format this version requests);
- 40 `RGBA8888_256x256/*.zlib` icons and 2 PNGs that other app versions use.

None of these files is archived, so only the bundled icons exist:
- `DefaultNewsIcon`, `StoreNewsIcon`, `StoryNewsIcon` and `TipsNewsIcon` in 1.48.2;
- `DoodleNewsIcon` in 1.45.3.

### 9.3 Screens

Opening and closing:
- `openNewsTeaser @ 0x100055cb8`: `mbNewsTeaserActive = YES`, sequence `OpenNewsTeaser`
  (`updateNewsTeaser`, `addDisplayGroup NewsPopupDisplayGroup` under `PocketGod`), then `markAsRead`.
  The controller opens it from the boot item `checkForNewNews::: @ 0x100159dc0`, used in
  `NoStartScreen` and `StartGameAfterIdolScreen`: if `hasUnreadNews`, set `mbNewsTeaserOpen` and
  `openNewsTeaser`.
- `openNewsScreen @ 0x100055b1c`:
  1. Queue the delayed episode event `LookAtNews` (`[[IdolManager instance]
     delayTriggerEpisodeEvent:@"LookAtNews"]`, §4.2).
  2. If the teaser is active, close it (flag cleared, listener `onNewsTeaserClosed`).
  3. `mNewsDefIndex = count − 1` (0 if empty), `mbNewsScreenActive = YES`, sequence `OpenNewsScreen`
     (`playSound Button`, `updateNewsScreen`, remove the teaser group, add `MapDisplayGroup` at
     `MapDisplayMarker` and `NewsContentDisplayGroup` at `MapContentMarker`).
  4. Borrow the controller's `MapPaperLogic`: set this object as its listener, `activate`, and hide
     `NewsContentDisplayGroup` until the paper is open.

  The controller opens it from its News button (`closeNewsScreen`, `mbNewsScreenOpen`, `CloseMenu`)
  or from the teaser's MoreInfo button (`onNewsTeaserLaunchNewsScreen @ 0x100078a0c`).
- Paper callbacks, as for the map (§11.2):
  - `onMapPaperLogicShowMap @ 0x100055e74` loads `MapTextureAtlas0` and shows the content.
  - `onMapPaperLogicHideMap @ 0x100055edc` calls listener `onNewsScreenClosed`, then
    `onLoadLocationTextureAtlas`, and hides the content.
  - `onMapPaperLogicAnimComplete @ 0x100055f64` sets sequence `CloseNewsScreen`
    (`addDisplayGroup markerId MapDisplayMarker`, which clears the marker).
- `closeNewsScreen @ 0x100055d48`: if the screen is active, clear the flag, call listener
  `onNewsScreenClosed` and set `CloseNewsScreen`. Then if the teaser is active, clear its flag, call
  `onNewsTeaserClosed` and set `CloseNewsTeaser` (`removeDisplayGroup NewsPopupDisplayGroup`). The
  controller calls it before opening any other screen.

Panel updates:
- `updateNewsScreen @ 0x10005615c`, in group `NewsContentDisplayGroup`:
  1. Hide `PrevButton` and `NextButton` (unselectable) and `ClickMe`. Make `NewsImage` unselectable.
     `Title` = `Article` = `""`.
  2. If `mNewsDefIndex ≥ 0`, the active def is `vector[index]`. `PrevButton` is shown and selectable if
     index > 0, `NextButton` if `index < count − 1`. If `linkUrl != ""`, `ClickMe` is shown and
     `NewsImage` made selectable.
  3. `Title` = title, wrap width 320.0 (`0x43a00000`), 1 line.
  4. `Article` = `[NSString stringWithFormat:@"%@\n\n%@", tease, article]` (asm `0x100056468`),
     wrap 320, 17 lines.
  5. `updateNewsIcon:`.
- `updateNewsTeaser @ 0x100056514`, in group `NewsPopupDisplayGroup`: `ClickMe` hidden, `NewsImage`
  unselectable, `NewsBlurb` = `""`. If the vector is non-empty, the active def is the **last** one.
  `ClickMe` and the link work as above. `NewsBlurb` = tease, wrap 320, 3 lines. Then `updateNewsIcon:`.
- `updateNewsIcon:def @ 0x100055fcc`, the 64×64 `NewsIcon` value in `NewsAtlas0` (`textures.md` §7.3):
  - If the bundle has `<imageFileName>.raw`: slot name = imageFileName, upload from the bundle
    (`FUN_1000384b0`).
  - Otherwise, if `Library/Caches/<imageFileName>.raw` exists: upload from the cache (`FUN_100038800`).
  - Otherwise upload `DefaultNewsIcon`. If the name is not yet in `mLoadingIconDictionary`, add it and
    request `"%@%@.%@"` (imageUrl, imageFileName, imageExt; asm `0x100056104`) with
    `listenerData = def`.

Links:
- `openNewsLink:url @ 0x10005677c`: `launchStoreScreen`, `launchChallengeScreen` and
  `launchStoryScreen` call `closeNewsScreen`, then the listener's `onNewsLaunchStoreScreen`,
  `onNewsLaunchChallengeScreen` or `onNewsLaunchStoryScreen`. Anything else calls `[listener openUrl:url]`.
  The bundle only uses `launchStoreScreen`, `launchStoryScreen` and http(s) links.

`onButtonPress:touch: @ 0x1000568e0`. The boot script creates these logics (`setLogic: @ 0x100056ba8`):

| Logic (group / object) | Action |
|---|---|
| `NewsScreenCloseButtonLogic` (`NewsContentDisplayGroup` / `CloseButton`) | `mbNewsScreenActive = NO`, paper `deactivate`, `ClickNewsScreenButton` (`playSound Button`); the paper callbacks finish the close |
| `NewsScreenNextButtonLogic` (`NextButton`) | `index = min(index + 1, count − 1)`, `UpdateNewsScreen` (`playSound Button`, `updateNewsScreen`) |
| `NewsScreenPrevButtonLogic` (`PrevButton`) | `index = max(index − 1, 0)`, `UpdateNewsScreen` |
| `NewsScreenLinkButtonLogic` (`NewsImage`) | `openNewsLink:active.linkUrl` |
| `NewsTeaserCloseButtonLogic` (`NewsPopupDisplayGroup` / `CloseButton`) | `mbNewsTeaserActive = NO`, listener `onNewsTeaserClosed`, `CloseNewsTeaser` |
| `NewsTeaserMoreInfoButtonLogic` (`MoreInfoButton`) | `mbNewsTeaserActive = NO`, listener `onNewsTeaserLaunchNewsScreen` |
| `NewsTeaserLinkButtonLogic` (`NewsImage`) | `openNewsLink:active.linkUrl` |
| `NewsScreenMoreInfoButtonLogic` | would `openUrl:@"http://www.boltcreative.com"`, but the boot script never creates this logic, so it is unreachable |

The remaining members:
- `updateNewsScreen::: @ 0x100056704` and `updateNewsTeaser::: @ 0x100056728` are sequence items
  returning 2.
- `loadLocationTextureAtlas::: @ 0x10005674c` calls the listener.
- `onButtonRelease:touch: @ 0x100056ba0` and `onButtonReleaseOutside:touch: @ 0x100056ba4` are empty.

### 9.4 Port

`managers::news` keeps the vector semantics and the merge quirks.

Loading:
- The cache file can be any format, because the original cache is never imported. A JSON or XML file
  in the data directory is fine. Keep the last-10 rule, so the "two new unread items every launch"
  behaviour survives.
- Parse the embedded `assets/xml/news.xml` with an XML reader that treats CDATA as text.
- **Never contact the dead S3 bucket.** The name could be re-registered by anyone (bucket takeover).
  A configurable feed URL that defaults to none is acceptable.

Icons:
- Icons come from the embedded `*NewsIcon` textures, else from `DefaultNewsIcon`.
- The download path stays unimplemented, or behind the same opt-in feed setting.

Links:
- External links open in the system browser through `pg-platform`.
- The `bit.ly` links in the bundle are mostly dead, but they are content and stay unchanged.

## 10. GameUiManager

Source `game/GameUiManager.c` (16 methods and `.cxx_construct`). A queue of floating "points" and
message pop-ups used by the idol screens. Singleton `DAT_1008161f0` (set in
`init @ 0x100120bd0`). `instance @ 0x100120bc4`, `id @ 0x100120d74`. The listener
(`setGameUiManagerListener: @ 0x100120d80`, the controller) is stored but never called. The private
`mSequence` (`setBehavior: @ 0x100120ddc`) is never started.

### 10.1 Data

`mGamePointsArray` is an inline ring of 30 entries of 0x30 bytes:

| Offset | Type | Field |
|---|---|---|
| +0x00 | id | custom message logic (`MultiModelLogic`, may be nil) |
| +0x08 | i32 | points |
| +0x10 | NSString | sequence (behaviour) id to play |
| +0x18 | NSString | label, retained |
| +0x20 | f32×4 | position (x, y, z, w) |

- Indices `mNextFreeGamePointsIndex` (write) and `mNextActiveGamePointsIndex` (read) both advance as
  `i < 29 ? i + 1 : 0`.
- `mTimeForNextGamePoints` is an f32 countdown.
- Pools, all created with capacity 5:
  - `mGamePointsLogicArray`, with the parallel `mGamePointsTextArray` and `mGamePointsLabelTextArray`;
  - `mCustomGamePointsLogicArray`.
- `mPocketGodDisplayGroup` is the display group `PocketGod` (`0x10038a270`).
- The queue has no overflow check. Queuing 30 entries ahead of activation brings the write index back to the
  read index, so the queue looks empty and those entries are lost.

### 10.2 Behaviour

- `showPoints:label:sequenceId:pos:messageLogic: @ 0x1001215e0` writes the entry at the free index
  (label retained), advances the index and calls `checkForNextGamePoints`.
- `showMessage:sequenceId:pos:messageLogic: @ 0x10012169c` and `showMessage:sequenceId:pos: @ 0x1001216c8`
  call it with points 0, the message as the label and, for the second, a nil logic.
- `onGameFrame: @ 0x100121900` runs `mTimeForNextGamePoints -= dt`, then `checkForNextGamePoints`.
  `gameFrameOrder @ 0x1001218f8` = 40.
- `checkForNextGamePoints @ 0x10012158c`: if the timer ≤ 0 and the queue is not empty,
  timer = `0x3e2aaaab` (1/6 s) and `activateNextGamePoints`. At most 6 pop-ups start per second.
- `activateNextGamePoints @ 0x10012102c`:
  1. Take the entry at the read index, then advance the index.
  2. Pick the logic:
     - If the entry has a custom logic, add it to the custom pool if it is not there yet, with this
       object as listener. It gets no texts.
     - Otherwise reuse the first pooled logic whose display object has no display group (free), together
       with the texts at the same index.
     - If none is free, create a pooled set. The logic is a `BCMultiModel` `GamePoints` (`0x1003ee3d0`)
       wrapped in a `MultiModelLogic` with this object as listener. The texts are `BCText`
       `PointsText` and `PointsLabelText` in font `AnimatedGothicLite20`, both with justification 2.
       `PointsText` is attached to animation tweenable channel 3 and `PointsLabelText` to channel 2.
  3. Add the display object to `PocketGod`, call `setBinaryFrameDef:nil tweenableChannelArray:nil`,
     then `setBehavior:entry.sequenceId`.
  4. Set `pos` = entry position with z forced to `0xc38a8f5c` (−277.12).
  5. If the label is non-nil and texts exist: `PointsText` = `[NSString stringWithFormat:@"%d",
     entry.points]` (asm `0x100121508`, the format argument is the entry's +0x08), `PointsLabelText` =
     label, then the label is released. A label given with a custom logic is never released.
- `onMultiModelEvent:eventId: @ 0x100121924`: event `GamePointsComplete` removes the display object
  from its group, which frees the pooled set.
- `allPointsComplete @ 0x1001216f0` = no pooled or custom logic is still in a display group.
  Queued, not yet activated entries are ignored.
- `removeAllGamePointsDisplayObjects @ 0x100120e44` removes every pooled and custom display object.
  It is never called.
- `dealloc @ 0x10012198c`. `.cxx_construct @ 0x100121c50` returns `self` (the array is
  zero-initialised by the allocation).

Callers:
- `IdolScreenLogic` uses both `showMessage` variants. The sequence ids are CFStrings at
  `0x1003e8c90`–`0x1003e8f50`. Their messages are battle texts (owned by the logic spec).
- `TempleDoorLogic` (`TempleDoorLogic.c:858`) polls `allPointsComplete`.

Port: a `GamePoints` resource with the same ring, pool and 1/6 s pacing. Spawning is in
`pg-game::managers::game_ui`. The multi-model playback comes from the engine.

## 11. MapScreen

Source `game/MapScreen.c` (16 methods). The island map. Singleton `DAT_1007d1ef0`.
`instance @ 0x1000c0a0c`, `id @ 0x1000c0c80`, `setMapScreenListener: @ 0x1000c0c8c` (the controller),
`dealloc @ 0x1000c0ce8`.

### 11.1 Buttons

`init @ 0x1000c0a18` fetches `BCLibrary getLogic:` ids:

| Logic id | Ivar | Location state on press |
|---|---|---|
| 0x225 | `mMapScreenCloseButtonLogic` | close (no change) |
| 0x226 | `mMapScreenAdBlockerLogic` (`AdBlockerLogic`) | — |
| 0x227 | Sand island button | 0 Sand |
| 0x228 | Rock island | 1 Rock |
| 0x229 | Graveyard island | 3 Graveyard |
| 0x22a | Ice island | 4 Ice |
| 0x22b | Ape island | 5 Ape |
| 0x22c | Misfit island | 6 Misfit |
| 0x22d | God island | 7 God |
| 0x22e | Apocalypse island | 10 Apocalypse |
| 0x22f | Underwater | 2 Underwater |
| 0x230 | Underwater coral reef | 9 Coral reef |

`mMapScreenFailBoatButtonLogic` (state 8) is never assigned. Its comparison against a nil ivar can
never match, so the fail boat cannot be reached from the map. The location-state enum (controller
`mIslandLocationState`) is 0 Sand, 1 Rock, 2 Underwater, 3 Graveyard, 4 Ice, 5 Ape, 6 Misfit, 7 God,
8 FailBoat, 9 UnderwaterCoralReef, 10 Apocalypse, 11–17 Time, Locust, Dance, Plague, Purification,
Closet and Xenu temples (`controller.md` §4.1).

### 11.2 Flow

1. Opening. The controller's `IslandLocationButton` handler cancels touches, closes the news screen,
   sets `mbMapScreenOpen` and calls `openMapScreen`.
2. `openMapScreen @ 0x1000c0d78`:
   1. `mbMapScreenActive = YES` and the sequence is `OpenMapScreen` (`playSound Button`,
      `addDisplayGroup MapDisplayGroup@MapDisplayMarker`, `addDisplayGroup
      MapContentDisplayGroup@MapContentMarker`).
   2. The misfit button (logic 0x22c) is shown and selectable only if
      `[listener hasMisfitIslandBeenFound]`.
   3. `[[listener mapPaperLogic] setMapPaperLogicListener:self]`, then `activate`.
   4. `MapContentDisplayGroup` is hidden.
   5. The ad blocker is activated if `[listener showChartboostAd]` returned YES, otherwise deactivated
      (§18).
3. The paper unrolls. `MapPaperLogic`'s `onMapPaperEnterComplete` item calls
   `onMapPaperLogicShowMap @ 0x1000c1050`: the atlas `MapTextureAtlas0.raw` is loaded into the atlas
   def at `0x10077b790` (`FUN_100038d68`, `textures.md`), then `MapContentDisplayGroup` is shown.
4. `onButtonPress:touch: @ 0x1000c1300` (only while `mbMapScreenActive`):
   - Close button: `mbMapScreenActive = NO`, the paper logic is deactivated, the sequence is
     `ClickMapScreenButton` (`playSound Button`), `mbCancelLocationStateChange = YES`, target 0.
   - A location button: `changeLocation:<state> @ 0x1000c1234` does the same, but with
     `mbCancelLocationStateChange = NO` and target = state.
   - `closeMapScreen @ 0x1000c0f8c` (the controller's `closeScreens`) behaves like the close button,
     but its sequence is `CloseMapScreen` (`addDisplayGroup` with only `markerId MapDisplayMarker`,
     which clears the marker).
5. The paper rolls up. `onMapPaperHideContentsComplete` calls `onMapPaperLogicHideMap @ 0x1000c10b8`
   (decompiler condition checked in asm). With `current = [listener islandLocationState]` and
   `target = mIslandLocationState`:
   1. `[listener onMapScreenClosed]` (the controller clears `mbMapScreenOpen`).
   2. If `current == target`, or either is 2 or 9, or the change was cancelled, call
      `[listener onLoadLocationTextureAtlas]` (reload the current island's atlases that the map
      replaced). A cancelled change stops here.
   3. If `current != target`, call `[listener onMapScreenChangeIslandLocationState:target]`
      (controller `setIslandLocationState:target startup:NO`, `controller.md` §4.2).
   4. Hide `MapContentDisplayGroup`.
6. `onMapPaperExitComplete` calls `onMapPaperLogicAnimComplete @ 0x1000c11c4`, which sets the
   sequence `CloseMapScreen`.

The remaining members:
- `onMapCloseComplete::: @ 0x1000c122c` returns 2 and is not referenced by any sequence.
- `isChartboostAdVisible @ 0x1000c0f70` forwards to the listener. It serves as the
  `AdBlockerLogic` listener method.
- `onButtonRelease:touch: @ 0x1000c15b8` and `onButtonReleaseOutside:touch: @ 0x1000c15bc` are empty.
- The display group is `0x1003ca790`. The sequence id strings are at `0x1003ca7b0`, `0x1003ca7d0`
  and `0x1003ca7f0`.

Port: unchanged. Without ads, `show_chartboost_ad` is false and the ad blocker stays hidden.

## 12. DebugScreen

Source `game/DebugScreen.c` (18 methods). **Unreachable in 1.48.2:** the controller opens it from
`mDebugButtonLogic`, an ivar that `loadGame` never assigns (`PocketGodViewController.c:9020`, the
only reference besides the declaration). When it is pressed, the controller first runs its own
sequence `CloseMenu` (`0x1003964b0`).

Singleton `DAT_1007cd7e0`. `instance @ 0x10000862c`, `id @ 0x100008a40`,
`setDebugScreenListener: @ 0x100008a4c`, `dealloc @ 0x100008aa8`. Logic ids from `init @ 0x100008638`:
- 0x158 close, 0x159 next page, 0x15a previous page;
- 0x15b–0x169: the 15 item buttons, in `mDebugItemButtonLogicArray` order.

- `openDebugScreen @ 0x100008b38`: `mbDebugScreenActive = YES`, sequence `OpenDebugScreen`
  (`playSound Button`, `loadTextureAtlas ScreenTextureAtlas0.raw → screenAtlas0`, `drawDisplayGroup
  DebugScreen`, `updateDebugItems`).
- `closeDebugScreen @ 0x100008bb0`: `mbDebugScreenActive = NO`, sequence `CloseDebugScreen`
  (`playSound Button`, `loadLocationTextureAtlas`, `drawDisplayGroup PocketGod`), then
  `[listener onDebugScreenClosed]`.
- `countDebugItems @ 0x100008c44` counts the nil-terminated pointer list at `0x100730938`: 11 items,
  `ResetChallenges`, `UnlockChallenges`, `ResetLevels`, `SetLevel5`, `SetLevel10`, `SetLevel15`,
  `SetLevel20`, `ResetEnemies`, `UnlockEnemies`, `ClearSkinPacks`, `RemovePrefs`
  (`debug-items.json`, `struct-xml.md` §11). The name is at +8 of each record.
- `updateDebugItems @ 0x100008c74`, in the `DebugScreen` display group:
  1. `NextPage` and `PrevPage` are hidden and unselectable.
  2. `PrevPage` is shown if `mDebugItemMenuIndex > 0`. `NextPage` is shown if
     `mDebugItemMenuIndex + 15 < count`.
  3. For i = 0, 1, … while `Option%dButton` (i) exists: if `index + i < count`, its text is the item
     name and `BlackBox%d` is selectable and shown; otherwise its text is `""` (asm `0x100008ee8`) and the box is
     unselectable and hidden.
- `onButtonPress:touch: @ 0x1000090ac` is ignored while `mbExitApp` (never set):
  - close → `closeDebugScreen`;
  - an item button → `selectDebugItemIndex:(menuIndex + position)`;
  - next / previous → `updateDebugItemPage:±1`.
- `selectDebugItemIndex: @ 0x100008f2c` is **empty**. No debug item does anything.
- `updateDebugItemPage: @ 0x100008f30`: `index += delta × 15`. If `index ≥ count`, then
  `index = (count / 15) × 15`. Clamped at 0. Then the sequence is `UpdateDebugItems` (`playSound
  Button`, `updateDebugItems`).
- Sequence items:
  - `updateDebugItems::: @ 0x100009058` → 2;
  - `loadLocationTextureAtlas::: @ 0x10000907c` → listener `onLoadLocationTextureAtlas`, then 2;
  - `waitForDataUpdate::: @ 0x100009008` → 0 while `[[listener challengePreferences] waitingToSave]`,
    else 2;
  - `exitApp::: @ 0x100009050` → 2 (does not exit).
  - `DebugScreenDelayExit` (`waitForDataUpdate`, `exitApp`) is never started.
- `onButtonRelease:touch: @ 0x1000091d4` and `onButtonReleaseOutside:touch: @ 0x1000091d8` are empty.

Port: port it as data plus an egui overlay, reachable only behind a developer flag (for example
`--debug-screen`). Keep `select_debug_item` a no-op for fidelity. Optionally, wire it to the
`IdolManager` helpers the names describe (`resetAllChallenges`, `unlockIdolChallenges`,
`setIdolLevels:`, …, §4.6) as a clearly marked developer extension.

## 13. Controller contract

Every manager has one listener: the `PocketGodViewController`, set in `loadGame`. `controller.md` §6 lists the
protocols from the controller's side. This section lists what the managers need from it, and what the
controller and the logic classes call on them. The port implements the listener side as a trait or as
system parameters in `pg-game`.

### 13.1 Manager → controller

| Manager | Calls on the controller |
|---|---|
| `GameServer` | `presentModalViewController:`, `dismissModalViewController`, `checkAllAchievementsAndLeaderboards` |
| `IdolManager` | `challengePreferences`, `optionPreferences`, `godIdolLogicArray`, `isScreenOpen`, `isStoryActive`, `isAnyGameActive`, `isDialogActive`, `isStormCloudEnabled`, `isNightActive`, `isSunsetActive`, `islandLocationState`, `onApocalypseGemEventTriggered:`, `onIdolAwardScreenOpened`, `onIdolAwardScreenClosed`, `onIdolAwardScreenOpenChallenges`, `onIdolAwardScreenChangeIslandLocationState:`, `onIdolScreenClosed`, `onIdolScreenLaunchStoreScreen`, `onLoadLocationTextureAtlas`, `presentModalViewController:`, `dismissModalViewController` |
| `ChallengeManager` | `godPersona`, `onAchievement:count:highScoreWins:`, `onChallengeDialogOpen`, `onChallengeScreenClosed`, `onStartChallenge:`, `onEndChallenge:`, `onLoadLocationTextureAtlas`, `presentModalViewController:`, `dismissModalViewController` |
| `StoryManager` | `systemPreferences` (file I/O), `tribePersona`, `onShowKeyboard:…`, `onHideKeyboard`, `onStoryAddPygmy:`, `onStoryStart`, `onStoryCancel`, `onStoryEnvironmentChange:`, `onStoryModeStart/StopCelebration`, `onStoryModeStart/StopEarthquake`, `onStoryModeSharkSnap`, `onStoryModeActivateRainbow`, `onStoryModeActivateMyRainbow`, `onStoryModeDeactivateRainbow`, `onStoryScreenClosed`, `onLoadLocationTextureAtlas` |
| `SkinPackManager` | `onSkinPackScreenClosed`, `onLoadLocationTextureAtlas` |
| `BonusPackManager` | `onBonusPackScreenClosed`, `onBonusPackUnlockDialogOpened`, `onBonusPackUnlockDialogClosed`, `onShowKeyboard:…`, `onHideKeyboard`, `openUrl:`, `onLoadLocationTextureAtlas` |
| `NewsManager` | `systemPreferences` (cache and network), `mapPaperLogic`, `onNewsScreenClosed`, `onNewsTeaserClosed`, `onNewsTeaserLaunchNewsScreen`, `onNewsLaunchStoreScreen`, `onNewsLaunchChallengeScreen`, `onNewsLaunchStoryScreen`, `openUrl:`, `onLoadLocationTextureAtlas` |
| `MapScreen` | `hasMisfitIslandBeenFound`, `islandLocationState`, `mapPaperLogic`, `showChartboostAd`, `isChartboostAdVisible`, `onMapScreenClosed`, `onMapScreenChangeIslandLocationState:`, `onLoadLocationTextureAtlas` |
| `DebugScreen` | `challengePreferences`, `onDebugScreenClosed`, `onLoadLocationTextureAtlas` |
| `GameUiManager` | none |

### 13.2 Controller and logic → managers

- The controller's `loadGame` creates the managers in this order:
  1. `SkinPackManager`, early (`PocketGodViewController.c:813`);
  2. later (`:1255`–`1298`): `GameServer`, `ChallengeManager`, `BonusPackManager`;
  3. `StoryManager`, with `loadStoryDefs`;
  4. `DebugScreen`;
  5. `NewsManager`, with `loadNewsDefs`;
  6. `IdolManager`, `GameUiManager` and `MapScreen`, then `initLogicListeners`.

  It sets itself as each one's listener and wires the sequence items through the boot script's `setLogic`
  and item registry (`sequences.md`).
- Screens:
  - `openSkinPackScreen:` / `closeSkinPackScreen`;
  - `openBonusPackScreen:` / `closeBonusPackScreen`;
  - `openStoryScreen` / `closeStoryScreen`;
  - `openChallengeScreen` / `cantOpenChallengeScreen` / `closeChallengeScreen`;
  - `openNewsScreen` / `openNewsTeaser` / `closeNewsScreen` / `hasUnreadNews` / `openNewsTeaserOnLoadComplete` /
    `loadNewsDefs`;
  - `openMapScreen` / `closeMapScreen`;
  - `openDebugScreen` (unreachable);
  - the `openIdol…Screen` family and `closeIdolScreen`.
- Game feeds:
  - `triggerEpisodeEvent:` (also from about 40 logic classes and the `triggerEpisodeEvent` sequence item),
    `triggerEpisodeEventBySacrificeId:`, `triggerEpisodeEventByGiftId:`, `delayTriggerEpisodeEvent:`;
  - `loadEpisodeState`;
  - the `ChallengeManager` scoring API (§5.2), also from `NoPeeLogic`, `TRexLogic` and `SpiderLogic`;
  - `refreshSkinPackOption:…`, `isSkinPackOptionActiveBySkinPackId:…`, `loadTextureAtlasChannel:`;
  - `onKeyboardTextEntered:` for the story and bonus-pack text entry.
- Logic classes:
  - `IdolScreenLogic` uses the idol and battle API (§4).
  - `TempleDoorLogic` uses the battle-round API, `isMultiplayerActive`, `onMultiplayerGameOver`,
    `quitBattleOfTheGods` and `GameUiManager allPointsComplete`.
  - `PygmyLogic(BattleOfTheGods)` calls `onBattleDamageComplete` and `onBattleAttackTie`.
  - `DoodleCloudLogic` calls `isBonusPackUnlocked:`.
  - `PygmyLogic(Island)` calls `getSkinPackOptionValueBySkinPackId:…` and `isSkinPackOptionActive…`.
  - `MapPaperLogic` calls the paper callbacks on `MapScreen` and `NewsManager`.
  - `AdBlockerLogic` calls `MapScreen isChartboostAdVisible`.
  - `BattleAlertLogic` calls `MatchAndMessage initWithMatch:message:` (`0x100027ac4`; accessors `match @ 0x100027b60`
    and `message @ 0x100027b70`, `dealloc @ 0x100027b80`). It is reached only from the dead match-alert path
    (§4.5).
  - `BCPreferences` calls `IdolManager onPreferencesChanged:` and the `NewsManager` download callbacks.

### 13.3 Text entry, modal views and URLs

These platform services belong to the controller, but the managers depend on them:

- **Text entry.** The controller keeps a hidden `UITextField`, frame (60, 214, 30, 30) (`init`,
  `PocketGodViewController.c:644`, constants `0x10029d788`/`0x10029d790`). Its settings:
  `clearsOnBeginEditing = YES`, `autocapitalizationType = None`, `autocorrectionType = No`,
  `returnKeyType = Done`. It observes `UITextFieldTextDidChangeNotification`.

  The calls:
  - `onShowKeyboard:listener editingText:t cancelEditDisplayObject:c maxEditCharCount:n preventEmpty:p @
    0x10006d84c`:
    1. Ignored while the keyboard is open.
    2. Store the listener, t and n, and save t's text.
    3. Show c (selectable).
    4. `enablesReturnKeyAutomatically = p` (Return is disabled while the field is empty).
    5. `showKeyboard`: add the field and make it first responder.
  - `textField:shouldChangeCharactersInRange:replacementString: @ 0x10006d4f0`: refuse when the new length
    > n, otherwise mirror into t. `textDidChange: @ 0x10006d49c` mirrors again.
  - `textFieldShouldReturn: @ 0x10006d6b8` writes the persona fields and resigns.
  - `textFieldDidEndEditing: @ 0x10006d5b8`: keyboard closed, c hidden, field removed, then
    `[listener onKeyboardTextEntered:t.text]`.
  - `onHideKeyboard @ 0x10006d994` cancels: restores t's saved text and hides c.

  Port: an egui single-line text field in `pg-ui` with the same rules: max length n, Return disabled when
  empty if p, the text starts empty on each open, live mirroring into the `BCText`, Escape or the cancel object
  → `on_hide_keyboard`, Enter → `on_keyboard_text_entered`. On Android it raises the soft keyboard.
- **Modal views.** Only Game Center sheets are presented (`presentModalViewController: @ 0x10006d2dc`,
  `dismissModalViewController @ 0x10006d388`; `controller.md` §2.7). They stop the render loop through the
  app delegate. Port: none (§18).
- **URLs.** `openUrl: @ 0x10006e9d4` stores the URL. Once the loading screen is hidden, it performs
  `openLaunchUrl @ 0x10006e93c`: stop the render loop, then `[UIApplication openURL:]`, which leaves the app.
  Port: `pg-platform::open_url`, which opens the system browser on desktop and Android and a new tab on the
  web. The game keeps running; pause it only if the platform backgrounds the app.

## 14. Persistence summary

What the managers persist, and where. The document and keychain mechanics are in `audio-prefs-text.md` §2.

| Data | Storage | Owner | Port |
|---|---|---|---|
| Episode progress: `<ep>BattleUnlock`, `<ep><ev>Challenge`, `…ChallengeHint`, `…LevelUp` | Challenge document (secure, in iCloud) | `IdolManager` | Challenge document in the save directory |
| `ActiveEpisode`, `<ep>SelectionSlot` | Options document | `IdolManager` | Options document |
| Personal bests (`God:AchievementDictionary`), achievement tiers `pg.*` | Persona and System documents (written by the controller) | `ChallengeManager` via the controller | unchanged |
| `InfoShown`, `BonusInfoShown` | System document (controller) | skin and bonus screens | unchanged |
| Skin purchases `<Pack>SkinPack`, skin selections `<option id>`, `ComicPreOrder` | keychain, service `PocketGod` | `SkinPackManager`, `BonusPackManager` | keychain document (`"BabaBooey"` / index strings) |
| User stories `story-<yyyyMMddHHmmss>` and `StoryMap` | `Documents/`, UTF-16 + zlib XML | `StoryManager` | save directory (§6.6) |
| News cache `news.xml` (last 10 items, read flags) | `Library/Caches/`, UTF-16 + zlib XML, excluded from backup | `NewsManager` | cache directory (any format) |
| News icons `<name>.raw` | `Library/Caches/` | `NewsManager` | not produced (no downloads) |
| Match data | Game Center match (not local) | `ChallengeManager`, `IdolManager` | local match store only with the hot-seat service |

## 15. Reachability

Source `thirdparty/Reachability.c` (Apple's sample, an old revision; 7 instance methods, 5 class methods and
1 callback). Its only game use is `SkinPackManager::isInternetReachable @ 0x10005cd90`:
`[[Reachability reachabilityWithHostName:@"www.apple.com"] currentReachabilityStatus] != 0`. The string is
at `0x10038f990`. The class's own `isInternetReachable @ 0x10019f8d8` (class method) does the same and is
not called.

The functions:
- `reachabilityWithHostName: @ 0x10019f920` (`SCNetworkReachabilityCreateWithName`),
  `reachabilityWithAddress: @ 0x10019f9b8`, `reachabilityForInternetConnection @ 0x10019fa48` and
  `reachabilityForLocalWiFi @ 0x10019fab0` (sets the BOOL ivar `localWiFiRef`).
- `currentReachabilityStatus @ 0x10019fbc8`: `SCNetworkReachabilityGetFlags`, then
  `localWiFiStatusForFlags: @ 0x10019fb34` (`(flags & 0x20002) == 0x20002`) or
  `networkStatusForFlags: @ 0x10019fb48`.
- `networkStatusForFlags:`, from asm: 0 unless Reachable (bit 1). Otherwise
  `r = !ConnectionRequired(bit 2)`. If ConnectionOnDemand (bit 5) or ConnectionOnTraffic (bit 3) is set,
  `r = !InterventionRequired(bit 4) | !ConnectionRequired`. The result is `r | IsWWAN(bit 18)`, so 0 or 1.
- `connectionRequired @ 0x10019fb88`; `startNotifier @ 0x10019f714`; `stopNotifier @ 0x10019f820`;
  `dealloc @ 0x10019f86c`. The callback `FUN_10019f794` posts `kNetworkReachabilityChangedNotification`
  and nobody observes it.

Port: `pg-platform::is_internet_reachable()` returns `true`. The only caller is the skin store's product
step, which the "everything owned" policy skips anyway (§7.4), so the value is never observable. No
networking crate is needed.

## 16. Keychain and preference documents

`BCKeyChain` and `BCPreferences` belong to the persistence spec (`audio-prefs-text.md` §2.2–§2.3).
Points that matter to the managers:

- Keychain items are generic passwords with service = `"PocketGod"` and account = key. The value lives in
  `kSecAttrGeneric`, and a new item is created holding `"THISISTHEUNSETSECRET"`. The managers read
  `secret`, compare it with `"BabaBooey"` or call `intValue`, write with `setSecret:`, and check `hasError`
  before purchases (skin store, bonus code). `reset` is used only by dead helpers.
- The Challenge document is the only one in iCloud. Conflict resolution keeps the larger number and the local
  string (`audio-prefs-text.md` §2.2.4). `IdolManager` is its listener and re-applies merged progress with
  `onPreferencesChanged:`.
- File helpers used here: `writeApplicationData:toFile:` (atomic, `Documents/`), `applicationDataFromFile:`,
  `removeFile:`, `writeApplicationData:toLibraryCacheFile:`, `applicationDataFromLibraryCacheFile:`,
  `doesFileExistInResources:ext:`, `doesFileExistInLibraryCache:ext:`, `loadDataFromServer:listenerData:url:`.
- Match data uses throw-away `BCPreferences` objects: `init` without an id, then `loadPreferencesFromData:`
  (`propertyListFromData:mutabilityOption:0 format:…`, nil or empty data → empty dictionary, a parse error →
  `initPreferences`). Writing uses `dataFromPreferences` (`dataFromPropertyList:format:200`, a binary
  plist v1.0).

Port: one `KeychainStore` (a string map in the save directory) and the four documents
(`audio-prefs-text.md` §2.7). Match data, when the hot-seat service exists, uses the same typed map
serialised to a binary plist with the `plist` crate, so iOS match blobs stay readable.

## 17. Platform touchpoints inventory

Every platform API reached from game code (the `game/` classes, excluding the SDK internals of
`game/Chartboost.c` and the Crashlytics helpers that Ghidra filed under `AdBlockerLogic.c`). The engine
rows (rendering, input, audio) are listed for completeness; their specs live elsewhere.

| # | Framework / API | Where (method @ address) | Purpose |
|---|---|---|---|
| 1 | Crashlytics `startWithAPIKey:` | `PocketGodAppDelegate::application:didFinishLaunchingWithOptions: @ 0x100089324` (key `0x1003c7ad0`) | crash reporting |
| 2 | Flurry `setCrashReportingEnabled:NO`, `startSession:` | same (key `0x1003c7af0`) | analytics session (no events are logged anywhere) |
| 3 | Chartboost `startWithAppId:appSignature:delegate:`, `cacheInterstitial:@"Home Screen"` | same (ids `0x1003c7b30`, `0x1003c7b50`) | interstitial ads |
| 4 | Chartboost `hasInterstitial:`, `showInterstitial:`, `cacheInterstitial:` | `PocketGodViewController::showChartboostAd @ 0x100088244` (from `MapScreen::openMapScreen`) | ad on every map open, when one is cached |
| 5 | `ChartboostDelegate` | `PocketGodViewController` `shouldRequestInterstitial: @ 0x100088364`, `shouldDisplayInterstitial: @ 0x100088390`, `didDisplayInterstitial: @ 0x1000883bc`, `didCacheInterstitial: @ 0x1000883e4`, `didFailToLoadInterstitial:withError: @ 0x10008840c`, `didDismissInterstitial: @ 0x100088474`, `didCloseInterstitial: @ 0x1000884d8`, `didClickInterstitial: @ 0x10008853c` | ad visibility flag; recache after close or click |
| 6 | `UIApplication setIdleTimerDisabled:`, `UIWindow`, `UIScreen`, `UIImageView` / `UIImage imageNamed:` | `PocketGodAppDelegate @ 0x100089324`, `showLoadingImage @ 0x100089570` | window and splash |
| 7 | `UIApplicationDelegate` lifecycle | `PocketGodAppDelegate` (§2.3) | pause and resume, news refresh |
| 8 | `NSTimer` | `PocketGodAppDelegate` (0.1 s `loadGame`) | deferred boot |
| 9 | `UIApplication openURL:` | `PocketGodViewController::openLaunchUrl @ 0x10006e93c` (from `openUrl:`: news links, bonus details; `controller.md` §4.18) | external links |
| 10 | `UITextField` and the keyboard | `PocketGodViewController` (§13.3) | text entry (names, story, bonus code) |
| 11 | `UIViewController` modal presentation | `PocketGodViewController::presentModalViewController: @ 0x10006d2dc` / `dismissModalViewController @ 0x10006d388` | Game Center sheets |
| 12 | `UIDevice systemVersion` | `GameServer::isTurnBasedMultiplayerAvailable @ 0x10008b134`, `BCPreferences::isICloudSupported` | feature gates |
| 13 | GameKit `GKLocalPlayer` authentication | `GameServer::authenticateUser @ 0x10008b298` and blocks `FUN_10008b48c`, `FUN_10008b678` | sign-in |
| 14 | GameKit `GKScore`, `GKAchievement` | `GameServer::submitToLeaderboard:score: @ 0x10008b894`, `submitAchievement: @ 0x10008b970` | 15 leaderboards, 72 achievements |
| 15 | GameKit `GKTurnBasedEventHandler` | `GameServer::onGameCenterAuthenticationComplete @ 0x10008ad18`, `handleTurnEventForMatch:…`, `handleMatchEnded:` | match notifications |
| 16 | GameKit `GKMatchRequest`, `GKTurnBasedMatchmakerViewController`, `GKTurnBasedMatch` (`loadMatchData…`, `endTurnWithNextParticipant:…`, `endMatchInTurnWithMatchData:…`, `participantQuit…`, `setMessage:`, `setMatchOutcome:`), `GKPlayer loadPlayersForIdentifiers:` | `ChallengeManager` (§5.3), `IdolManager` (§4.5) | Mini Game Challenges and Battle of the Gods multiplayer |
| 17 | StoreKit `SKProductsRequest`, `SKPaymentQueue`, `SKPayment`, `SKProduct price` | `SkinPackManager` (§7.3) | 23 skin-pack purchases |
| 18 | SystemConfiguration `SCNetworkReachability` | `Reachability` (§15) via `SkinPackManager::isInternetReachable` | store pre-check |
| 19 | Security `SecItem*` | `BCKeyChain` (`audio-prefs-text.md` §2.3) via `SkinPackManager`, `BonusPackManager`, `BCPreferences` | purchase, selection and Challenge-document storage |
| 20 | iCloud: `UIDocument`, `NSMetadataQuery`, `NSFileManager URLForUbiquityContainerIdentifier:` (`CGMFV4QFBB.<bundle id>`) | `BCPreferences`, `BCPreferencesDocument` (`audio-prefs-text.md` §2.2.4) | Challenge-document sync |
| 21 | `NSURLConnection` | `BCPreferences::loadDataFromServer:listenerData:url:` (only from `NewsManager`) | news feed and icons |
| 22 | `NSLocale currentLocale` country code | `NewsManager` `countryLink` (§9.2); `PocketGodViewController::openCountryURL: @ 0x10006e7b8` (dead store links, `struct-xml.md` §7) | localized links |
| 23 | `NSBundle infoDictionary[CFBundleVersion]` | `IdolManager::setActiveMatch: @ 0x1000321e0` | multiplayer version gate (`"7"`) |
| 24 | `NSXMLParser` | `NewsManager`, `StoryManager` | XML |
| 25 | zlib `compress` / `inflate` | `FUN_100047ee4`, `FUN_100048000` | story and news files |
| 26 | AudioToolbox `AudioServicesPlaySystemSound(0xfff)` | `BCSound` ("Vibrate" sound, `audio-prefs-text.md` §1.7.4) | vibration |
| 27 | CoreMotion `CMMotionManager` | `BCView` (`rendering.md` §10) | tilt and shake |
| 28 | OpenGL ES `EAGLContext`, `CADisplayLink`, `UITouch` | `BCView`, `BCTouch` (`rendering.md`) | rendering, frame loop, touch |
| 29 | `NSNotificationCenter` `UIApplicationDidBecomeActiveNotification` | `GameServer::init @ 0x10008a918`, `BCPreferences` | re-authentication, iCloud |

Absent from game code: `UIAlertView`, `UIActionSheet`, `UIWebView`, mail and social sharing, local and push
notifications, rating prompts, photo and music libraries, `NSUserDefaults` (only the Chartboost SDK uses it),
background tasks.

## 18. Platform replacement plan

The target builds are desktop (macOS, Windows, Linux), web and Android, with no App Store, no Game Center, no
ad network, no analytics and no remote server (`ARCHITECTURE.md`). The guiding rule: reproduce exactly what
an iOS player sees when the service is unavailable or already satisfied, so no gameplay is invented or lost
beyond what the service itself provided.

### 18.1 Per touchpoint

| # | Port behaviour | Gameplay impact |
|---|---|---|
| 1–2 | Removed. No crash or analytics SDK | none |
| 3–5 | Removed. `show_chartboost_ad()` returns `false`, `is_chartboost_ad_visible()` returns `false`, so `AdBlockerLogic` stays hidden | none (no interstitial on the map) |
| 6 | Bevy window; splash = the converted launch image, letterboxed, until `hideLoadingScreen`. Android keeps the screen on | none |
| 7–8 | Bevy focus and `AppLifecycle` events → pause and resume (§2.4) | none |
| 9 | `pg-platform::open_url` | none (the game is not left on desktop) |
| 10 | egui text field (`pg-ui`) with the §13.3 rules | none |
| 11 | No modal sheets exist | none |
| 12 | Constant: "iOS ≥ 5". The feature gates read local service availability instead | none |
| 13 | Local stand-in: authentication completes at once as "not authenticated". Every authentication listener still gets `onGameServerAuthenticationComplete`, and `checkAllAchievementsAndLeaderboards` still runs | none |
| 14 | Local records only. The tier achievements are already written to the System document by the controller. Leaderboards are kept as a per-id best in the save (optional). Nothing is shown in-game in the original either | none (Game Center UI only) |
| 15–16 | Default: no match service. `ChallengeManager::openMatchPanel` shows "Game Center not available." and `IdolManager::openMatchPanel` shows "Multiplayer Not Available:", exactly as on an iOS device without Game Center. Optional: the hot-seat service (§18.3) | Mini Game Challenges and BotG multiplayer are unavailable by default. Single-player BotG, personal bests and the 61 s timer code are unaffected |
| 17 | No store. Every pack counts as owned at boot (`ARCHITECTURE.md`); the store screens still work as a skin browser | none (all paid skins free) |
| 18 | `true` constant | none |
| 19 | `KeychainStore` document | none |
| 20 | No sync; the Challenge document is local. Optional import of iOS saves (binary plists and the keychain export are not accessible, so only `Documents/` files) | none |
| 21 | No network. News comes from the embedded `news.xml` and icons from the embedded `*NewsIcon` textures. **Never fetch the old S3 bucket** (it no longer exists and its name could be re-registered by a third party) | Only the 67 bundled news items, with the bundled quirks (§9.2) |
| 22 | Country code from the OS locale (`sys-locale` crate, or `navigator.language` on the web); only `countryLink` uses it | none |
| 23 | Constant `"7"` | none |
| 24–25 | `quick-xml`, `flate2` | none |
| 26 | Android: vibrator. Desktop and web: no-op (web may use `navigator.vibrate`) | none |
| 27 | `ARCHITECTURE.md`: Android sensors, web `DeviceMotionEvent`, desktop emulation | as specified there |
| 28 | Bevy rendering and input | none |
| 29 | Bevy lifecycle events | none |

### 18.2 Policy summary

- Purchases: everything owned (skin packs, Fishmas, and the Comic Pre-Order bonus pack, §8.4).
- Ads, analytics and crash reporting: removed.
- Game Center sign-in: none. Achievements and leaderboards: local.
- Turn-based multiplayer: off by default, with an optional local hot-seat (§18.3).
- Network: none (news and icons come from the bundle).
- iCloud: none.
- Debug screen: developer flag only (§12).

### 18.3 Optional local match service ("hot-seat")

BotG and Mini Game Challenges keep all their game logic in the clients and only need a store for the match
dictionary plus turn passing. `pg-platform::matches` can provide that on one device for two local players.

- **Identity.** Players "Player 1" and "Player 2", with ids `local:1` and `local:2` and aliases taken from
  the god persona name and a prompt for the second player. `GKLocalPlayer.playerID` becomes the *current
  seat*.
- **Matches.** A list saved in the save directory. Each entry holds `match_id`, the two participants (id,
  outcome), `current_participant`, `status` (0/1/2/3 as in GameKit), `message` and `match_data` (a binary
  plist).
- **Matchmaker sheet.** An egui window listing the open local matches, with "New match" and "Quit match".
  It returns the same delegate callbacks: `did_find_match`, `was_cancelled`, `player_quit_for_match`.
  Auto-match participants (no id yet) are filled by the second seat when it first opens the match.
- **Turns.** `end_turn_with_next_participant(p, data)` stores the data and sets the current participant. If
  p is the same seat (the `saveMatchData` idiom), nothing changes hands. Otherwise the UI asks the next player
  to take the device, switches the seat, and delivers `handle_turn_event_for_active_match` on the next frame.
- **End and quit.** `end_match_in_turn` stores the outcomes and sets status 2. `participant_quit…` sets
  outcome Quit and passes the turn.
- **Availability.** `is_turn_based_multiplayer_available()` = the service is enabled. Authentication
  completes as authenticated when it is.

Both managers then run unchanged, including the version gate (constant `"7"`), the higher-score-wins outcome
rule for challenges and the multiplayer freeze-target agreement. The only new UI is the matchmaker list and
the "pass the device" prompt. Whether to ship it is a product decision (§20).

## 19. Partition file

`tmp/re/managers-partition.json` (UTF-8, pretty-printed, keys in fixed order, no floating point). It uses
the same top-level shape as the other partitions: a JSON array of units in the order below.

```
[ Unit ]

Unit {
  unit: String,                  // snake_case id
  description: String,
  methods: [String],             // "Class::selector @ 0x…" (class methods unmarked, as in the other
                                 // partitions) or "FUN_xxxxxxxxx @ 0x…" for C functions and blocks;
                                 // address order; every function of the partition exactly once
  ivars: [String],               // ivars owned by this unit, as "Class::ivar"
  ivars_shared: [String],        // "Class::ivar" it uses but another unit of this partition owns
  bytes: u64,                    // UTF-8 size of the unit's decompiled text (effort proxy, as in the other partitions)
  depends_on: [String],          // other units of this partition it calls or whose ivars it uses
  doc_section: String,           // "docs/re/managers-platform.md#…"
  crate: String | null,          // target crate and module; null = not ported
  port: String,                  // "port", "port_owned" (purchases treated as owned), "replace" (stand-in), "drop"
  shared_with: [String],         // units of other partitions that also claim these methods
  function_roles: {String: String}  // address → role, for the C functions and blocks
}
```

Units, with their method counts:

| Unit | Count | Section | Port |
|---|---|---|---|
| `app_delegate` | 16 | §2 | port (also claimed by the controller partition's `app_platform`) |
| `game_center_server` | 48 | §3 | replace |
| `reachability` | 13 | §15 | replace |
| `chartboost_sdk` | 200 | §17 | drop (third-party SDK class that Ghidra put under `game/`) |
| `idol_episodes` | 62 | §4.2 | port |
| `idol_battle` | 64 | §4.4 | port |
| `idol_multiplayer` | 70 | §4.5 | replace (match service), incl. `MatchAndMessage` |
| `challenge_scoring` | 14 | §5.2 | port |
| `challenge_match` | 85 | §5.3 | replace (screens ported; match service) |
| `story_data` | 70 | §6.2 | port |
| `story_screens` | 42 | §6.4 | port |
| `story_playback` | 27 | §6.5 | port |
| `skin_pack_state` | 23 | §7.2 | port_owned |
| `skin_pack_store` | 40 | §7.3 | port_owned |
| `bonus_pack` | 31 | §8 | port_owned |
| `news` | 44 | §9 | port (bundled data only) |
| `game_ui_points` | 17 | §10 | port |
| `map_screen` | 16 | §11 | port |
| `debug_screen` | 18 | §12 | port (developer flag) |

The total is 900 functions: every method, class method, block and C helper in the 14 source files
(`game/{IdolManager, StoryManager, ChallengeManager, SkinPackManager, BonusPackManager, NewsManager,
GameUiManager, MapScreen, GameServer, MatchAndMessage, DebugScreen, PocketGodAppDelegate, Chartboost}.c`,
`thirdparty/Reachability.c`). The list was cross-checked against `classes.json`. `compressBlob` and
`uncompressBlob` (`FUN_100047ee4`, `FUN_100048000`) are compiled from `StoryManager.mm`, but Ghidra filed
them in `game/PygmyLogic+Apocalypse.c`. They are therefore **not** listed here; they belong to whichever
partition owns that file (§20).

## 20. Open questions

1. **Empty CDATA.** It is not known whether `NSXMLParser` reports an empty `<![CDATA[]]>` (story dialogs,
   §6.2). If it does not, built-in stories would show `"ENTER DIALOG"` balloons. The port treats an empty
   element as `""` (no balloon), which matches what players would have reported as normal.
2. **Name-list order.** The story editor's name list follows `NSDictionary` hash order (§6.4), which cannot
   be reproduced exactly. The port uses insertion order (tribe names, the six fixed names, then the story's
   own names).
3. **News teaser.** The teaser reappears on every launch because two bundled items use `newsItemId` (§9.2).
   Kept for fidelity; confirm that this is wanted.
4. **Hot-seat multiplayer.** Ship the optional local match service (§18.3), or keep the faithful "not
   available" alerts only?
5. **Bonus pack ownership.** `ARCHITECTURE.md` makes bonus packs owned (§8.4), which removes the code-entry
   and unlock-dialog content from play. The alternative is to start locked and keep the code and
   6000-Oogle unlock.
6. **Three frozen idols in multiplayer.** `onBattleIdolSelectedMultiplayer:` dereferences the NULL result of
   `chooseFrozenIdolBattleState:` when a side has ≥ 3 frozen idols (§4.5). The original would crash. The port
   must pick a behaviour; storing nil ids is proposed.
7. **`leaveMatch`** has no caller in the decompilation (no selector reference). It is treated as dead; a
   selector built at run time cannot be fully excluded.
8. **`compressBlob` / `uncompressBlob` ownership.** They sit in `PygmyLogic+Apocalypse.c` but belong to
   StoryManager and NewsManager. Whoever owns that file should hand them to `pg-game` utilities (or
   `pg-platform`), not to the pygmy port.
9. **Chartboost and PocketGodAppDelegate double claims.** `game/Chartboost.c` is SDK code and is claimed
   here with `port: drop`. `PocketGodAppDelegate` is claimed both here and by the controller partition. The
   merger should keep one owner.
