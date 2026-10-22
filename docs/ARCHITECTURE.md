# Architecture

Pocket God 1.48.2 was an Objective-C app on Bolt Creative's "BC" engine (OpenGL ES 1.1,
data-driven sequences and timeline animations). This port keeps that architecture and its
behaviour, translated function by function from the decompiled original, and hosts it in
Bevy 0.20.

## Crates

| Crate | Role |
|---|---|
| `pg-assets` | Every asset embedded in the executable (`build.rs` over `assets/`), Bevy asset source for them, serde schemas for the extracted data tables, the `Timelines.bin` parser. |
| `pg-engine` | The BC engine on Bevy ECS: library of definitions, display objects, models, text, timeline animation, sequences, logic framework (`BCLogic`, `SpriteLogic`, `MultiModelLogic`, `BCScreenLogic`, `BCButtonLogic`, …), textures and channels, sound instances, preferences, touches, view/projection, and the draw-list builder. No game-specific code. |
| `pg-render` | Draws the engine's `DrawList` with Bevy (`Mesh2d` batches, custom `Material2d`, letterboxed camera, gamma-space blending). |
| `pg-platform` | Bevy bridges for audio playback, mouse/touch/keyboard input, save files, URL opening and other OS services. |
| `pg-ui` | egui stand-ins for the UIKit pieces of the original (alerts, text entry, web/news views, debug screen). |
| `pg-game` | The game: `PocketGodViewController` (split by feature), managers (idols, story, challenges, skin packs, bonus packs, news, game UI, map, personas), `PygmyLogic` and every other logic class. |
| `pocket-god` | The executable: window, plugins, wiring. |

Gameplay classes call each other freely (the original was one app with circular
references), so they share `pg-game`, organised by feature into modules.

## Platforms

The same code builds for desktop (macOS, Windows, Linux), the web (`wasm32-unknown-unknown`
via `wasm-bindgen`) and Android (GameActivity, `cargo-ndk` + Gradle).

- `pocket-god` is a library plus the desktop/web binary; `pocket-god-android` is the
  Android `cdylib` (`#[bevy_main]`), enabling Bevy's `android-game-activity` feature.
- All platform differences live in `pg-platform` behind `cfg`s: save storage (files on
  desktop, internal storage on Android, `localStorage` on the web), URL opening, and motion
  input. Motion is reported in the original iOS accelerometer convention (g units, device
  axes); Android reads the accelerometer through the NDK sensor API and remaps it for the
  display rotation, the web uses `DeviceMotionEvent`, desktop emulates tilt and shake from
  the keyboard.
- Touch input comes from Bevy touch events (Android, mobile web) or the mouse (desktop, web).
- The window is landscape-only, like the original (`UIInterfaceOrientationLandscapeRight`).

## Object model

The original's object graph lives in Bevy's `World`; `docs/GAME-PORTING.md` has the full rules.

- **Every Objective-C object is an entity.** Each class contributes one component holding its own
  ivars (`BcLogic`, `SpriteLogic`, `BirdLogic`, …; very large classes one per part), so an object
  of class `BirdLogic` carries `BcLogic + SpriteLogic + BirdLogic`.
- **Classes.** An `Isa(&'static Class)` component names the object's class. `Class` holds the
  name, the superclass and the methods the engine dispatches dynamically (`onGameFrame:`,
  sequence items, touches, `setBehavior:`, drawing, …); a subclass's `Class` is built from its
  superclass's with struct-update syntax and overrides what the ObjC subclass overrides.
- **Methods** are free functions `fn name(world: &mut World, this: Entity, args…)`. Never hold a
  component borrow across a call that takes `world`; read what you need, release, call. This keeps
  ObjC-style re-entrancy (A calls B which calls back into A) safe. `[super m]` is a direct call
  to the ancestor's function; selectors that several game classes implement are dispatched by
  `pg_game::send`.
- **Display tree.** Display objects are entities linked by the engine's own `DisplayGroup`
  children and `DisplayObject::display_group` (not Bevy's `ChildOf`): a group can be shared by
  several markers, as in the original. Child order is draw order.
- **Singletons** (`BCLibrary`, `BCView`, …) are resources; the game's `+instance` statics are the
  `Singletons` resource; the logic array `createLogicClasses` fills is `Library::logic_table`.
- **Listeners** (delegate protocols) are direct calls when the listener is always the same class
  (usually the view controller); otherwise the stored listener entity is dispatched.
- **Sequences** call handlers by selector name: the receiver's `Class::sequence_item` matches the
  handlers its class implements and defers to its superclass; the root falls back to the delegate
  and then to `BCLibrary`'s handlers (`docs/re/sequences.md`).
- **nil messaging**: ObjC silently ignores messages to nil. Optional references are
  `Option<Entity>` and keep the original's checks; never add behaviour the original lacks.

## Frame

One exclusive system runs the original frame: input → `onGameFrame` (in the original's
order) → animations/sequences → draw-list build. Timing follows the original frame loop
(`docs/re/rendering.md`). `pg-render` turns the draw list into GPU work; `pg-platform` turns
sound commands into Bevy audio.

## Porting rules

- The decompiled code under `tmp/re/decomp/` is the source of truth. Translate behaviour
  faithfully: same constants, same order of operations, same random choices, same edge
  cases. Do not invent, simplify away or "improve" gameplay.
- Every ported method carries a one-line doc comment naming the original, e.g.
  `` /// `-[PiranhaLogic fightPiranha:]` ``.
- Names: ObjC class names stay as Rust type names (`PiranhaLogic`); ivars lose their
  `m`/`mb`/`mp` prefixes and become snake_case (`mbFlipping` → `flipping`); selectors become
  snake_case functions (`animPiranhaYFlip` → `anim_piranha_y_flip`).
- Follow the workspace style guide (`~/.claude/CLAUDE.md`): Rust conventions for naming,
  no long method chains, no complex expressions inside call arguments, named intermediate
  values, blank lines between decorated fields.
- In-app purchases are free: every purchasable product (skin packs, bonus packs and any other
  paid content) counts as owned from the first launch. There is no store or purchase API;
  wherever the original checks ownership or would start a purchase, treat the product as
  already purchased (so purchase prompts never appear).
- Other platform services that cannot exist here (Game Center, ad networks, analytics,
  remote servers) are replaced as described in `docs/re/managers-platform.md`.

## Assets

`tools/re/` scripts convert the original bundle into `assets/`:

- `assets/textures/*.png` — every `.raw` texture page, decoded to RGBA8 (non-sRGB).
- `assets/sounds/*.flac` — every `.caf`, losslessly.
- `assets/timelines/Timelines.bin` — the original timeline file, parsed at startup.
- `assets/xml/` — `story.xml`, `news.xml`.
- `assets/data/*.json` — tables extracted from the executable (textures, sequences, display
  groups, episodes, challenges, logic objects, sounds, fonts, gradients, …).

`pg-assets/build.rs` embeds all of it; `EmbeddedAssetsPlugin` also exposes it as Bevy's
default asset source.
