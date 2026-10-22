# Renderer and display tree (BCView / BCDisplayObject hierarchy)

Authoritative spec for the Pocket God v1.48.2 renderer: GL state, projection, frame loop,
display tree, every drawable class, text, gradients, the custom geometry models, touch input,
hit testing and the accelerometer. Every statement cites the function it comes from as
`Class::method @ address` (or `FUN_xxxxxxxx @ address` for free functions). Where the
decompiler output was ambiguous, the disassembly was checked; those places are marked
*(asm-verified)*.

Conventions used in this document:

- **Game coordinates**: the space in which almost all display objects live (children of the
  root display groups). x grows right (0..480 is the 3:2 "safe" area), y grows **up**
  (0..320), z is depth (the standard sprite depth is `-277.12`).
- Matrices use the OpenGL convention: column vectors, `M·v`, stored column-major. "`A·B`"
  applied to a vertex means "B first, then A". `glTranslatef`/`glRotatef`/`glScalef` post-
  multiply the current matrix, so a sequence of calls `C1; C2; C3` produces `M = M0·C1·C2·C3`
  and vertices see `C3` first.
- `Rx(a)`, `Ry(a)`, `Rz(a)` are standard right-handed counter-clockwise rotations by `a`
  degrees (what `glRotatef(a, axis)` produces). `T(x,y,z)` translation, `S(x,y,z)` scale.
- `f32` arithmetic everywhere unless noted. "≤ / <" comparisons are exact as in the binary.

---

## 1. View, GL context and projection

### 1.1 View creation and orientation

- `PocketGodAppDelegate::application:didFinishLaunchingWithOptions: @ 0x100089324` creates
  `PocketGodViewController`, sets the window frame to `UIScreen.mainScreen.bounds`, shows a
  loading image (§12.3) and schedules `loadGame` after 0.1 s.
- `PocketGodViewController::init @ 0x100069ce8` creates the single `BCView` *(asm-verified)*:
  - `frame = UIScreen.mainScreen.applicationFrame`.
  - If `NSFoundationVersionNumber <= 1047.25` (iOS ≤ 7.1): frame width/height are swapped
    (`(x, y, h, w)`) and the view transform is `{a=0, b=1, c=-1, d=0, tx=0, ty=0}` (rotation by
    +90°), so the view's own bounds are landscape.
  - Otherwise (iOS 8+): frame unchanged (already landscape), identity transform.
  - In both cases the BCView bounds are **landscape** (e.g. 480×320 points) and
    `locationInView:` returns landscape points with the origin at the **top-left** of the
    screen as the user holds it.
- Orientation is fixed to **landscape right** (home button on the right):
  `PocketGodViewController::supportedInterfaceOrientations @ 0x10006daf4` returns 8
  (`UIInterfaceOrientationMaskLandscapeRight`), `preferredInterfaceOrientationForPresentation
  @ 0x10006dafc` returns 3 (`UIInterfaceOrientationLandscapeRight`); Info.plist lists only
  `UIInterfaceOrientationLandscapeRight`, `UIStatusBarHidden = true`, device family iPhone+iPad.
- `BCView::initWithFrame: @ 0x100175840`: stores `mTouchWidth = frame.size.width`,
  `mTouchHeight = frame.size.height` (landscape points), `setMultipleTouchEnabled:YES`,
  `srandom(time(0))`, `mDisplayLinkCount = 1`, clears all state (accel vectors to
  `(0,0,0,1)`), creates the three NSMutableArrays (game-frame listeners, UI touches, game
  touches), creates a `CMMotionManager` with device-motion and accelerometer update interval
  `0x3f91111120000000` = 0.016666667535901 s (1/60) and calls `startAccelerometerUpdates`, then
  `initGLES`. `BCView::instance @ 0x10017581c` returns this singleton (`DAT_1007d1f40`).

### 1.2 GL context and framebuffer

- `BCView::layerClass @ 0x100175828` → `CAEAGLLayer`.
- `BCView::initGLES @ 0x1001762b0` *(asm-verified)*: layer `opaque = YES`, drawable
  properties `{kEAGLDrawablePropertyRetainedBacking: NO, kEAGLDrawablePropertyColorFormat:
  kEAGLColorFormatRGBA8}`, `EAGLContext initWithAPI:1` (**OpenGL ES 1.1, fixed function**),
  then `createFramebuffer`. The framebuffer is not retained between frames (every frame is a
  full redraw after `glClear`).
- `BCView::createFramebuffer @ 0x1001763f0`: one framebuffer + one colour renderbuffer
  (storage from the layer). **No depth buffer is ever created** (`mDepthRenderbuffer` stays 0);
  depth testing is never enabled. `mViewWidth/mViewHeight` = renderbuffer size in pixels;
  `mScreenWidth = mViewWidth`, `mScreenHeight = mViewHeight` (as floats), then `setupView`.
- The binary never sets `contentScaleFactor` (no such string in the binary), so the
  CAEAGLLayer keeps its default scale of 1.0: the renderbuffer is in **points** (480×320,
  568×320, 1024×768). This only matters for point-sprite size (§8.3) and scissor rounding
  (§1.5); the logical coordinate system is resolution independent.

### 1.3 Fixed GL state (`BCView::setupView @ 0x1001765a4`, asm-verified)

| State | Value |
|---|---|
| Clear colour | `(1, 0, 0, 1)` (red) initially; changed by `setClearColorRed:green:blue:alpha:` (§1.6) |
| Shade model | `GL_SMOOTH` (per-vertex colour interpolation) |
| Client arrays enabled | `GL_VERTEX_ARRAY`, `GL_TEXTURE_COORD_ARRAY` (colour array off) |
| Blend function | `glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA)` — **premultiplied alpha** — never changed |
| Blending | enabled (toggled per draw afterwards, §3) |
| Depth test / culling / alpha test | never enabled (GL defaults: off) |
| Texture env | `GL_MODULATE` (default; never changed) → fragment = texel × current colour |
| Texture 2D | enabled by the texture loaders (`glEnable(GL_TEXTURE_2D)` in `FUN_100038084`/`FUN_100038168`); toggled by untextured draws |
| Frustum shift | `setFrustumHorizontalShift:0.5 verticalShift:0.5` (the only call in the binary) |

Atlas textures (see the texture spec) are uploaded with `GL_TEXTURE_MIN_FILTER = GL_LINEAR`
(or `GL_LINEAR_MIPMAP_LINEAR` + `GL_GENERATE_MIPMAP` when the atlas mipmap flag at +0x36 is
set, `FUN_100038084 @ 0x100038084`), default `GL_TEXTURE_MAG_FILTER = GL_LINEAR` and default
wrap `GL_REPEAT`. Texel data is premultiplied (checked on several RGBA4444 atlases, R in the
high nibble: ≥ 99.8 % of pixels have rgb ≤ a; the few exceptions are left as they are and
must be blended as-is).

### 1.4 Logical screen size, wide/tall flags and projection

`BCView::setFrustumHorizontalShift:verticalShift: @ 0x100176618` *(asm-verified)*, with
`hs = vs = 0.5`, `W = mScreenWidth`, `H = mScreenHeight` (renderbuffer pixels on entry):

```
glMatrixMode(GL_PROJECTION); glLoadIdentity()
mFrustumHorizontalShift = hs; mFrustumVerticalShift = vs
glRotatef(90, 0, 0, 1)
mbTallScreen = 0; mbWideScreen = 0
A = W / H                                  // aspect of the landscape renderbuffer
if A < 1.5:                                // ('b.pl' at 0x1001766d0; NaN takes the else branch)
    mbTallScreen = 1
    mScreenWidth  = 480
    mScreenHeight = 480 / A
    mScissorWidth  = (float)mViewWidth
    mScissorHeight = (float)mViewWidth / 1.5
else:
    if A > 1.5: mbWideScreen = 1
    mScreenHeight = 320
    mScreenWidth  = A * 320
    mScissorHeight = (float)mViewHeight
    mScissorWidth  = (float)mViewHeight * 1.5
mScissorXPos = ((float)mViewWidth  - mScissorWidth)  * 0.5
mScissorYPos = ((float)mViewHeight - mScissorHeight) * 0.5
W_L = mScreenWidth; H_L = mScreenHeight
glOrthof(left = -H_L*hs, right = (1-hs)*H_L,
         bottom = -W_L*vs, top = (1-vs)*W_L,
         near = 0, far = 5000)
glViewport(0, 0, mViewWidth, mViewHeight)
mScreenZPos = W_L / (2 * tanf(0.5235988))   // 0x3f060a92 = π/6; vestigial, never read
glMatrixMode(GL_MODELVIEW)
```

The projection is therefore `P = Rz(+90°) · Ortho`, i.e. with `hs = vs = 0.5`:

```
clip_x = -e_y / (W_L/2)
clip_y =  e_x / (H_L/2)
clip_z = -2·e_z/5000 - 1      →  visible iff  -5000 ≤ e_z ≤ 0
```

where `e` is the eye-space (modelview) position. Eye x is the **vertical** screen axis and
eye y the horizontal one (flipped). Every group that is ever made the active display group
(`PocketGod`, `ScreenDisplayGroup`, `Help`, `DebugScreen`, `ChallengeScreen`,
`BonusPackScreen`, `CustomPackScreen`, `StoryScreen`, `WhatsNew`, `StatsPageGod`,
`StatsPagePygmy`, `StatsPageTribe`; also `StartScreen`, `ChallengeCompleteAlert`) compensates
with `xPos=-160, yPos=240, zRot=-90`; the two overlay groups use `xPos=-160, yPos=284,
zRot=-90`. For a root group `T(px, py, 0)·Rz(-90°)` a child point `g` lands at
`e = (g_y + px, py - g_x, g_z)`, so for `(-160, 240)`:

```
ndc_x = (g_x - 240) / (W_L/2)      ndc_y = (g_y - 160) / (H_L/2)
pixel_x = (g_x - 240 + W_L/2) · s   pixel_y_from_bottom = (g_y - 160 + H_L/2) · s,  s = mViewWidth/W_L = mViewHeight/H_L
```

Game point (240, 160) is always the screen centre; one game unit = `s` pixels.
`BCView::frustumCenterXPos @ 0x100175e60` = `hs·mScreenWidth` (= W_L/2),
`frustumCenterYPos @ 0x100175e80` = `vs·mScreenHeight` (= H_L/2), `screenWidth @ 0x100175ea0`,
`screenHeight @ 0x100175eb0`, `touchWidth @ 0x100175ec0`, `touchHeight @ 0x100175ed0`,
`isWideScreen @ 0x100176984`, `isTallScreen @ 0x100176994` expose these values.

`isWideScreen` = the screen is wider than 3:2 (extra columns left/right); `isTallScreen` = the
screen is taller than 3:2 (extra rows top/bottom). Exactly 3:2 → neither.

### 1.5 Device table

The app ships launch images only for 480×320, 568×320 and iPad (`Default.png`,
`Default@2x.png`, `Default-568h@2x.png`, `Default-Landscape~ipad.png`,
`Default-Landscape@2x~ipad.png`), so all later iPhones run in 568×320 compatibility mode.

| Device class | view / renderbuffer (pt = px) | A | W_L × H_L | flags | scissor (px, after rounding) | visible game x | visible game y |
|---|---|---|---|---|---|---|---|
| iPhone/iPod 3.5" (incl. retina) | 480×320 | 1.5 | 480×320 | none | not used | 0 … 480 | 0 … 320 |
| iPhone 4"+ (5/5s/SE, 6/7/8/X… in compat mode) | 568×320 | 1.775 | 568×320 | wide | (44, 0, 480, 320) | −44 … 524 | 0 … 320 |
| iPad (all, 1024×768 pt) | 1024×768 | 1.3333 | 480×360 | tall | (0, 42, 1024, 683) | 0 … 480 | −20 … 340 |

Scissor rounding is done by `FUN_10013fc94` (§3.2): `x0 = floor(x)`, `y0 = floor(y)`,
`x1 = (int)(floor(x) + ceil(w))`, `y1 = (int)(floor(y) + ceil(h))`.

### 1.6 Clear colour

`BCView::setClearColorRed:green:blue:alpha: @ 0x1001769a4` is a tail call to `glClearColor`
*(asm-verified; it is 4 bytes long, not a no-op)*. It is called only by
`PocketGodViewController(Game)::updateGradients @ 0x1001592b8` with the colour of stop 0 of the
island sky gradient and alpha 1.0 (§7.4). The clear happens at the **start** of the next
frame (§2.2), so a colour set during frame N takes effect in frame N+1. Initial value: red.

### 1.7 Letterboxing and overlay display groups

- `BCView::gameLoop @ 0x100177730`: when `mbWideScreen || mbTallScreen`, the active display
  group is drawn inside the scissor rect `(mScissorXPos, mScissorYPos, mScissorWidth,
  mScissorHeight)` (the central 3:2 area = game rect 0..480 × 0..320); the scissor is then
  popped (disabled) and the overlay groups are drawn unclipped:
  - `mWideScreenOverlayDisplayGroup` if set **and** `mbWideScreen`;
  - `mTallScreenOverlayDisplayGroup` if set **and** `mbTallScreen`.
- Set by `BCLibrary::drawWideScreenOverlayDisplayGroup::: @ 0x10002452c` /
  `drawTallScreenOverlayDisplayGroup::: @ 0x100024590` (param 0 = display-group id) →
  `BCView::setWideScreenOverlayDisplayGroup: @ 0x1001775e8` / `setTallScreenOverlayDisplayGroup:
  @ 0x100177660` (same hibernate/unhibernate swap as the active group, §4.6). The main
  `PocketGod` sequence runs `drawDisplayGroup PocketGod`, `drawWideScreenOverlayDisplayGroup
  WideScreenOverlay`, `drawTallScreenOverlayDisplayGroup TallScreenOverlay`.
- `PocketGodViewController::loadGame @ 0x10006a030` loads `WidescreenOverlayAtlas0.raw` into
  atlas `widescreenOverlayAtlas0` (256×512) only when wide or tall.
- Overlay groups (struct-XML root `0x1007162f0`) use root transform `xPos=-160, yPos=284,
  zRot=-90`, i.e. overlay x = game x + 44:
  - `WideScreenOverlay`: models `WideScreenOverlayLeft` (texture `WidescreenOverlayLeft`, quad
    x 0..54 in a 64×320 frame) at `x=-9, z=-277.12` → covers game x −53…1; and
    `WideScreenOverlayRight` (texture `WidescreenOverlayRight`, quad x 10..64) at `x=513` →
    covers game x 479…533.
  - `TallScreenOverlay`: `TallScreenOverlayTop` (texture `WidescreenOverlayTop`, 30×480 quad in
    a 64×480 frame, `zRot=-90`) at `(44, 347)` → covers game y 317…347; `TallScreenOverlayBottom`
    (texture `WidescreenOverlayBottom`, quad x 34..64, `zRot=-90`) at `(44, 36)` → covers game
    y −28…2.
- The overlays are authored for exactly 568×320 and 4:3; on other aspect ratios the original
  would leave clear-colour bands (only relevant for a port that supports arbitrary windows).

### 1.8 What zPos means

- With the orthographic projection, eye z only matters for **near/far clipping**: a vertex is
  visible iff its accumulated eye z is within `[-5000, 0]` (triangles are clipped, not
  culled). Drawing order is purely the display-list order (§5.1); there is **no z sorting and
  no depth buffer**.
- The sprite classes `BCModel`, `BCMultiModel`, `BCRect` and `BCText` apply a fake
  perspective using **their own** `pos.z` only (parents' z is ignored for this):
  ```
  k = -277.12 / pos.z                       // 0xc38a8f5c; -277.12 → k = 1
  translate(pos.x, k·(pos.y - 160) + 160, pos.z)
  ... scale x and y by k ...
  ```
  i.e. deeper objects (more negative z) shrink and are pulled vertically toward the horizon
  line y = 160; x is **not** pulled toward the centre. Examples from the data: `Glacier`
  z = −1500 (k = 0.18475), `IceMonster` z = −800 (k = 0.3464).
- `-277.12` = `320 / (2·tan 30°)`: the distance of a 60° vertical-FOV camera from a 320-unit
  screen; a leftover of an earlier perspective camera. `BCView::mScreenZPos` is still computed
  (from `W_L`, so it is 415.7 on 480×320) but never read.
- `pos.z = 0` makes `k = ±inf` and the translation NaN: such a sprite is not visible (do not
  draw it). `BCDisplayObject::initWithId: @ 0x10010dfc0` defaults z to 0, so code-created
  sprites always set z explicitly (usually −277.12).

---

## 2. Frame loop

### 2.1 Render loop

- `BCView::startRenderLoop @ 0x100177af8`: no-op if already active; else sets
  `mbRenderLoopActive = 1`, `enableAccelerometer`, `mStartTime = CFAbsoluteTimeGetCurrent()`,
  `mLastTime = 0xbd888889` (= −0.06666667), creates (once) a `CADisplayLink` targeting
  `gameLoop` with `frameInterval = mDisplayLinkCount`, adds it to the current run loop in
  `NSDefaultRunLoopMode`.
- `BCView::stopRenderLoop @ 0x100177c28`: if active → inactive, `disableAccelerometer`,
  remove the display link from the run loop.
- `PocketGodViewController::loadGame @ 0x10006a030` (end, *asm-verified*): after loading it
  calls `gameLoop` **three times directly** (warm-up frames, before the render loop exists;
  the first has `mStartTime = 0` so its dt clamps to 1/15, the next two have tiny real dts),
  then `setDisplayLinkCount: (loadTime <= 13.0 s ? 1 : 2)` (60 Hz, or 30 Hz on slow devices)
  and schedules `startRenderLoop` after 0.1 s (`NSTimer`).
- App lifecycle (`PocketGodAppDelegate`, all gated on `mbStartupComplete`, which
  `onHideLoadingScreen` sets): `applicationWillResignActive: @ 0x100089a90` stops and
  `applicationDidBecomeActive: @ 0x100089a24` starts the loop unless a dashboard or modal view
  is open; `applicationDidEnterBackground: @ 0x100089afc` first dismisses an open modal view,
  then stops the loop unless a dashboard is open; `applicationWillEnterForeground:
  @ 0x100089b94` (no dashboard) refreshes news and starts the loop. `onModalViewWillAppear
  @ 0x100089c8c` / `onModalViewWillDisappear @ 0x100089cd0` stop/start it. Restarting resets
  the time base, so the first frame after a resume has dt clamped to 1/15.
- `BCView::setDisplayLinkCount: @ 0x100177ad8`, `displayLinkCount @ 0x100177ae8`,
  `isRenderLoopActive @ 0x100177720`, `setGameFrameDebug @ 0x10017770c` (debug flag, unused),
  `gameFrameCount @ 0x100175ef0`, `time @ 0x100175ee0` are accessors.

### 2.2 `BCView::gameLoop @ 0x100177730` (exact order, asm-verified)

```
mGameFrameCount += 1
[EAGLContext setCurrentContext: mContext]; glBindFramebufferOES(mViewFramebuffer)
glClear(GL_COLOR_BUFFER_BIT)                     // whole renderbuffer, scissor is off here
t  = (float)(CFAbsoluteTimeGetCurrent() - mStartTime)
dt = t - mLastTime;  if dt > 0.06666667 { dt = 0.06666667 }   // 0x3d888889; no lower clamp
mLastTime = t; mTime = t
while DAT_100821e50 > 0:                         // deferred atlas loads, LIFO (§2.4)
    DAT_100821e50 -= 1; e = queue[DAT_100821e50]
    FUN_100038d68(e.fileName, e.ext, e.atlasDef)
readAccelerometer                                // §10
processTouchEvents:(t)                           // §9.2 – touch callbacks run here
sort mGameFrameListenerArray with FUN_1001757bc  // ascending gameFrameOrder
snapshot = copy of the array (stack array of 200 IDs, no bounds check)
retain all; for each in snapshot order: [obj onGameFrame:dt]; release all
if mbWideScreen || mbTallScreen: FUN_10013fc94(scissor rect)     // push scissor
[mActiveDisplayGroup drawOpenGL:nil]
if mbWideScreen || mbTallScreen:
    FUN_10013fdec()                              // pop scissor (disables GL_SCISSOR_TEST)
    if mWideScreenOverlayDisplayGroup && mbWideScreen: [it drawOpenGL:nil]
    if mTallScreenOverlayDisplayGroup && mbTallScreen: [it drawOpenGL:nil]
glBindRenderbufferOES; [mContext presentRenderbuffer:]; glGetError → log only
```

- **Variable timestep**, one update per display-link tick, dt clamped to ≤ 1/15 s. Update
  (touch dispatch, then game-frame listeners) always precedes drawing in the same tick.
- A listener removed during the frame still receives this frame's `onGameFrame:` (it is in
  the snapshot and retained); a listener added during the frame starts next frame.
- Many gameplay animations are per-frame constants (e.g. `shakeBackground`, wind decay,
  `FingerTrailLogic` lerp, gradient fades) and assume 60 ticks/s.

### 2.3 Game-frame listeners and ordering

- `BCView::addGameFrameListener: @ 0x100177528` / `removeGameFrameListener: @ 0x100177544`
  append/remove (`removeObject:` removes every occurrence).
- `FUN_1001757bc @ 0x1001757bc` (comparator): `a.gameFrameOrder == b.gameFrameOrder → 0;
  a < b → −1; else 1`. The array is re-sorted every frame with
  `sortUsingFunction:context:` (assumed stable, see open question 3), so equal orders keep
  insertion order.

| gameFrameOrder | Implementer (address) |
|---|---|
| 10 | `BCSequence::gameFrameOrder @ 0x1000c0518` |
| 20 | `BCAnimation::gameFrameOrder @ 0x1001a0498` |
| 30 | `PygmyLogic::gameFrameOrder @ 0x100129978` |
| 40 | `BCLogic::gameFrameOrder @ 0x1001389dc` (default for all `*Logic`), `BCPreferences @ 0x10016e7a4`, `GameUiManager @ 0x1001218f8` |
| 50 | `PocketGodViewController(Game)::gameFrameOrder @ 0x1001585b4`, `StoryManager @ 0x100049470` |
| 59 | `NoPeeLogic @ 0x10019da54` |
| 60 | `MoronBeeLogic @ 0x1001750c0`, `MoronDuckLogic @ 0x1001a8030`, `NoPeeMapItemLogic @ 0x10017f7ac`, `NoPeeMapLogic @ 0x100182fd0`, `NoPeeMapScreenLogic @ 0x10001add8` |
| 70 | `GameServer @ 0x10008ba48` |

Who registers: running `BCSequence`s (`play`/`stop`), running `BCAnimation`s, `BCLogic`
objects between `startGameFrame @ 0x100138754` and `stopGameFrame @ 0x1001387e0` while not
hibernating (`BCLogic::hibernate @ 0x100138874` removes the listener and remembers it,
`unhibernate @ 0x100138940` restores it), `PocketGodViewController` (added near the end of
`loadGame @ 0x10006a030`), `BCPreferences`, `GameUiManager`, `GameServer`, `StoryManager`,
`CrackHoleLogic`, `KongStatueLogic`, `MoronSnakeLogic`. Hibernation follows the display tree
(§4.6): only logic attached to objects reachable from the active display group (or an overlay
group) runs.

### 2.4 Deferred atlas activation

`BCView::activateTextureAtlasNextFrame:fileName:ext: @ 0x1001776d8` appends
`{atlasDef, fileName, ext}` (0x18-byte entries at `DAT_100821e58`, count `DAT_100821e50`, max
10, extra requests silently dropped). They are consumed at the start of the next `gameLoop`
from the **last** entry to the first. `BCLibrary::loadTextureAtlas::: @ 0x100024418` uses it
when its 4th param is non-null, else loads immediately: params `[0]` ext (ignored by
`FUN_100038d68`, which always uses `.raw`), `[1]` file name, `[2]` atlas-def id, `[3]` defer
flag. (Texture formats/loading: see the texture spec.)

---

## 3. GL state helpers (the state machine a port must emulate)

The engine wraps GL calls in small caching helpers (all in the 0x10013f8cc–0x10013fdec block,
which Ghidra filed under `IdolPedestalLogic.c`). A faithful port must reproduce their
**observable** effect, including state that leaks from one draw to the next.

| Helper | Effect |
|---|---|
| `FUN_10013f8cc` | enable `GL_TEXTURE_2D` (cache `DAT_100648cd8`, initial 1) |
| `FUN_10013f8ec` | disable `GL_TEXTURE_2D` |
| `FUN_10013f908` / `FUN_10013f928` | enable / disable `GL_TEXTURE_COORD_ARRAY` (cache initial 1) |
| `FUN_10013f944` / `FUN_10013f964` | enable / disable `GL_COLOR_ARRAY` (cache initial 0) |
| `FUN_10013f980(r,g,b,a)` | `glColor4f(r,g,b,a)` unless equal to the last value passed here (cache initial (0,0,0,0) while GL's real initial colour is (1,1,1,1)) |
| `FUN_10013f9cc(size)` | `glPointSize` (cached, initial −1) |
| `FUN_10013f9e8(name)` | `glBindTexture` unless cached or equal to the "being uploaded" name `DAT_100648cdc` (initial 666) |
| `FUN_10013fa1c(name)` | invalidate the bind cache (sets both caches to 666) |
| `FUN_10013fa40(p)` | `glColorPointer(4, GL_FLOAT, 0, p)` |
| `FUN_10013fa58(p)` | `glVertexPointer(3, GL_FLOAT, 0, p)` |
| `FUN_10013fa70(p)` | `glTexCoordPointer(2, GL_FLOAT, 0, p)` (cached by pointer) |
| `FUN_10013faa4(x,y,z)` | `glTranslatef(x,y,z)` unless x = y = z = 0 |
| `FUN_10013fac4(rx,ry,rz)` | `glRotatef(rx,1,0,0)` if rx≠0; then `glRotatef(ry,0,1,0)` if ry≠0; then `glRotatef(rz,0,0,1)` if rz≠0 → `M·Rx·Ry·Rz` (vertices see Rz first) *(asm-verified)* |
| `FUN_10013fb3c(sx,sy,sz)` | `glScalef` unless all three are exactly 1 |
| `FUN_10013fb58(kx,ky)` | skew, see below; no-op if kx = ky = 0 |
| `FUN_10013fc28` / `FUN_10013fc4c` | enable `GL_POINT_SPRITE_OES` + `glTexEnvi(GL_POINT_SPRITE_OES, GL_COORD_REPLACE_OES, 1)` / disable point sprites |
| `FUN_10013fc58(on)` | blending on/off (cache `DAT_100648cda`, initial 1) |
| `FUN_10013fc94(x,y,w,h)` | push scissor (§3.2) |
| `FUN_10013fdec()` | pop scissor |

**Skew** `FUN_10013fb58(kx, ky) @ 0x10013fb58` *(asm-verified)*: `a = kx·π/180`,
`b = ky·π/180`, `glMultMatrixf` with the column-major matrix

```
col0 = ( cos b,  sin b, 0, 0)
col1 = (-sin a,  cos a, 0, 0)
col2 = ( 0, 0, 1, 0)      col3 = (0, 0, 0, 1)
→  x' = x·cos b − y·sin a,   y' = x·sin b + y·cos a
```

(the x axis is rotated by `ySkew`, the y axis by `xSkew`; `zSkew` is never used).

### 3.1 Implications for a port

- **Blend state leaks**: `BCGradient`, `LightningModel`, `HurricaneModel`, `WhirlPoolModel`,
  `PathModel`, `MagnifyingBeamModel` never set blending; they use whatever the previous draw
  left (persisting across frames). Emulate a `blend_enabled` flag through the traversal.
- **Colour leaks**: `LocustModel` never sets a colour (uses the current one);
  `BCMultiModel`/`BCText`/`BCDisplayGroup` "in multi-model" draws rely on the colour set by the
  caller. Emulate a `current_color` through the traversal.
- With blending **off**, the fragment is written as is: texel alpha is ignored (a fully
  transparent premultiplied texel writes black).

### 3.2 Scissor stack (`FUN_10013fc94 @ 0x10013fc94`, `FUN_10013fdec @ 0x10013fdec`, asm-verified)

State: index `DAT_100648ce8` (initial −1), stack of int rects `{x, y, w, h}` at
`DAT_100816270` (16 bytes each), currently applied rect at `DAT_100816260`.

```
push(x, y, w, h):
    if index == -1: glEnable(GL_SCISSOR_TEST)
    x0 = (int)floorf(x); y0 = (int)floorf(y)
    x1 = (int)(floorf(x) + ceilf(w)); y1 = (int)(floorf(y) + ceilf(h))
    if index >= 0: intersect with top {tx,ty,tw,th}: x0 = max(x0,tx); x1 = min(x1,tx+tw);
                                                      y0 = max(y0,ty); y1 = min(y1,ty+th)
    w = max(0, x1-x0); h = max(0, y1-y0)
    stack[++index] = {x0, y0, w, h}; glScissor if different from the applied rect
pop():
    index -= 1
    if index >= 0 and stack[index] differs from the applied rect: glScissor(stack[index])
    if index == -1: glDisable(GL_SCISSOR_TEST)
```

Only `gameLoop` uses it (one level deep).

---

## 4. The display tree

### 4.1 `BCDisplayObject` fields

All drawables derive from `BCDisplayObject : NSObject <BCTweenable>`. Defaults from
`BCDisplayObject::initWithId: @ 0x10010dfc0`. Byte offsets are ivar offsets in the object.

| Field | Off | Type | Default | Meaning |
|---|---|---|---|---|
| `mId` | 8 | NSString | ctor arg | name; children are looked up by it |
| `mData` | 16 | NSString | nil | free-form data string from XML |
| `mDisplayGroup` | 24 | BCDisplayGroup | nil | parent (weak) |
| `mLogic` | 32 | BCLogic | nil | attached logic (weak) |
| `mbDebug` | 40 | BOOL | 0 | unused by drawing |
| `mbHidden` | 41 | BOOL | 0 | see §5.9 |
| `mbIsSelectable` | 42 | BOOL | 0 | touch hit testing (§9) |
| `mbHibernate` | 43 | BOOL | 1 | §4.6 |
| `mHibernateRefCount` | 44 | int | 0 | §4.6 |
| `mPos` | 48 | vec4 | (0,0,0,1) | position in parent space |
| `mRot` | 64 | vec4 | (0,0,0,1) | Euler degrees x, y, z |
| `mSkew` | 80 | vec4 | (0,0,0,1) | skew degrees x, y (z unused) |
| `mScale` | 96 | vec4 | (1,1,1,1) | |
| `mPivotPos` | 112 | vec4 | (0,0,0,1) | rotation centre (models) |
| `mOriginPos` | 128 | vec4 | (0,0,0,1) | offset of the geometry |
| `mbColor` | 144 | BOOL | 0 | set by every colour setter; never read |
| `mRed/Green/Blue/Alpha` | 148/152/156/160 | f32 | 1 | tint, straight (non-premultiplied) |
| `mA, mB, mC, mD` | 164–176 | f32 | 1,0,0,1 | `setMatrixA:b:c:d:` 2×2 matrix; never read |
| `mMinPos`, `mMaxPos` | 180, 196 | vec4 | (0,0,0,1) | scratch for min/max helpers |
| `mColXMin…mColZMax` | 212–232 | f32 | 0 | collision box (local, before scale) |
| `mbMultiModelMode` | 236 | BOOL | 0 | drawn by a parent BCMultiModel (§5.5) |

Accessors (`pos/setPos:`, `xPos/setXPos:` … `setColZMax:`, `@ 0x10010e7c0 … 0x10010f370`) are
plain field reads/writes (vector setters copy all 4 floats, including w). Colour setters
(`setRed: @ 0x10010ed4c`, `setGreen:`, `setBlue:`, `setAlpha: @ 0x10010eddc`,
`setRed:green:blue:alpha: @ 0x10010edfc`, `setRed:green:blue: @ 0x10010ee40`) also set
`mbColor = 1`. `setTextureDef: @ 0x10010e780` is a no-op and `textureDef @ 0x10010e784`
returns NULL in the base class; `enableMultiModelMode @ 0x10010e56c` /
`disableMultiModelMode @ 0x10010e570` are no-ops in the base class (only `BCModel` and
subclasses implement them). `draw:bounds:` (CoreGraphics path) is empty in every class except
`BCDisplayGroup`, and is never called.

"In place" origin setters move the position so the geometry stays put *(asm-verified)*:

```
setOriginPosInPlace:(o) @ 0x10010eb10:
    d = o.xyz - mOriginPos.xyz
    for i in x,y,z: if mScale[i] < 0: d[i] = -d[i]
    mPos.xyz -= d;  mOriginPos = o (all 4 comps)
setOriginXPosInPlace:(v) @ 0x10010ec08: d = v - origin.x; pos.x = (scale.x > 0) ? pos.x - d : pos.x + d; origin.x = v
setOriginYPosInPlace:(v) @ 0x10010ec74: same with y, scale.y
setOriginZPosInPlace:(v) @ 0x10010ecec: same with z but tests scale.**y** (original bug)
```

Bounding-box helpers (used by gameplay collision), results stored in `mMinPos/mMaxPos`
(z and w copied from `mPos` for `minPos/maxPos`; z = 0 for the rotated versions):

```
minPos @ 0x10010ee78:  x = pos.x + origin.x*sx + sx*(sx > 0 ? colXMin : colXMax)
                       y = pos.y + origin.y*sy + sy*(sy > 0 ? colYMin : colYMax)
maxPos @ 0x10010ef58:  x = pos.x + origin.x*sx + sx*(sx > 0 ? colXMax : colXMin)
                       y = pos.y + origin.y*sy + sy*(sy > 0 ? colYMax : colYMin)
rotatedMinPos @ 0x10010f038 / rotatedMaxPos @ 0x10010f17c:
    p = (origin.x*sx + sx*(...), origin.y*sy + sy*(...))   (same selection as above)
    c = cos(rot.z°), s = sin(rot.z°)
    x = c*p.x - s*p.y + pos.x;  y = s*p.x + c*p.y + pos.y   (asm-verified)
```

`BCTweenable` (protocol declared by `BCDisplayObject`; methods `id`,
`onAnimationMoveEnabled`, `isSoundDisabled`, `applyMoveXPos:moveYPos:`, `parentTweenable`,
`setBinaryFrameDef:tweenableChannelArray:`) is the interface the animation system uses to drive
display objects/logics; the renderer only sees its effect through `BCMultiModel` (§5.5) and
`mbMultiModelMode`. `TweenableChannel = {BOOL mbMultiModelMode; BOOL mbEnableMultiModelMode;
BOOL mbReleaseNextFrame; id<BCTweenable> mTweenable /* +8 */}` (16 bytes; `BCAnimation` holds
30). See `timelines.md` (§4–5) for how channels and multi-model mode are toggled
(`BCBinaryAnimationDef::applyFrame:… @ 0x1001a0ca4`).

### 4.2 Class hierarchy

```
BCDisplayObject
├── BCDisplayGroup ── BCDisplayMarker
├── BCModel ── BCMultiModel ── BCTileModel
│          └── BCGradient
├── BCRect
├── BCText
└── RainbowModel, LightningModel, LocustModel, HurricaneModel,
    WhirlPoolModel, PathModel, MagnifyingBeamModel
```

### 4.3 `BCDisplayGroup` (children list)

`mDisplayObjectArray` (offset 240, NSMutableArray, initial capacity 10,
`initWithId: @ 0x100066f6c`). Order = draw order = touch priority (§9.4).

| Method | Behaviour |
|---|---|
| `displayObjectCount @ 0x100067518` | count |
| `addDisplayObject: @ 0x100067544` | nil → no-op; `child.setDisplayGroup:self`; append; if `!self.mbHibernate` → `child.unhibernate`. **Does not detach the child from a previous parent.** |
| `addDisplayObject:atIndex: @ 0x1000675d0` | same, but `insertObject:atIndex:` when `index < count && count != 0`, else append |
| `removeDisplayObject: @ 0x1000676ac` | nil → no-op; if `!self.mbHibernate` → `child.hibernate`; `child.setDisplayGroup:nil`; `removeObject:` |
| `removeDisplayObjectAtIndex: @ 0x100067738` | object = `index < count ? array[index] : (count ≥ 1 ? lastObject : nil)`; then as above |
| `getDisplayObject:(id) @ 0x10006781c` | first **direct** child whose id `isEqualToString:` (no recursion); nil if none |
| `getDisplayObjectAtIndex: @ 0x100067964` | bounds-checked, else nil |
| `getDisplayObjectIndex:(id) @ 0x1000679d0` | index of first child with that id, −1 if none |
| `getDisplayObjectIndexByDisplayObject: @ 0x100067b1c` | `indexOfObject:` truncated to int (NSNotFound → −1) |
| `addSelectableLogicsToArray: @ 0x100067204` | §9.4 |
| `addLogicsToArray: @ 0x1000672c8` | like the above (also tests `isSelectable` for the group's own logic — copy/paste), recursing with `addLogicsToArray:` |
| `hibernate @ 0x10006738c` / `unhibernate @ 0x10006744c` | §4.6 |
| `drawOpenGL: @ 0x100067cf8` / `drawInMultiModelOpenGL:… @ 0x100067eb4` | §5.2 |

`BCDisplayObject` tree helpers: `removeFromDisplayGroup @ 0x10010e574` (→ parent
`removeDisplayObject:`), `moveToDisplayGroup: @ 0x10010e684` (remove, then
`addDisplayObject:`), `moveToDisplayGroup:atIndex: @ 0x10010e6c8`, `insertBefore:(obj)
@ 0x10010e598` (`obj.removeFromDisplayGroup`; `i = parent.indexOf(self)`;
`parent.addDisplayObject:obj atIndex:i`), `insertAfter:(obj) @ 0x10010e60c` (same with
`i + 1`). `setDisplayGroup: @ 0x10010e500`, `displayGroup @ 0x10010e510`.

### 4.4 `BCDisplayMarker` (single-slot placeholder)

`BCDisplayMarker : BCDisplayGroup` holds at most one child; markers are the anchors used with
`insertAfter:`/`insertBefore:` and `addDisplayGroup … markerId=`.

- `addDisplayObject: @ 0x10018bd68`: if it has a child → `removeDisplayObject:` it (hibernate,
  unparent); `obj.setDisplayGroup:self`; if obj ≠ nil append it; if `!self.mbHibernate` →
  `obj.unhibernate`. Passing nil just clears the slot.
- `addDisplayObject:atIndex: @ 0x10018be4c` ignores the index.
- `drawOpenGL: @ 0x10018be9c`: if it has a child → `[child drawOpenGL:self]`. The marker's
  own transform, colour and hidden flag are **not** applied (but its transform is part of the
  hit-test matrix chain, §9.3; markers in the data have identity transforms).

### 4.5 Library registry and display sequence items

`BCLibrary` keeps id → object dictionaries (`addDisplayGroup:`/`getDisplayGroupById:`,
`addDisplayMarker:`/`getDisplayMarkerById:`, texture/atlas/gradient/font/dynamic-text maps).
Display groups and markers are created by `BCLibrary::parseStructXml: @ 0x100026674` from
the compiled struct-XML node lists (node layout `BCStructXmlDisplayObjectNodeDef`, 0x84
bytes; attributes applied by `BCLibrary::parseDisplayObjectAttributes:displayObject:
@ 0x10002626c`, *asm-verified*):

| Node off | Attribute | Applied when | Call |
|---|---|---|---|
| +0x10 | data | ≠ nil | `setData:` |
| +0x18/+0x1c/+0x20 | origin x/y/z | ≠ 0 | `setOriginX/Y/ZPos:` |
| +0x24/+0x28/+0x2c | pos x/y/z | ≠ 0 | `setX/Y/ZPos:` |
| +0x30/+0x34/+0x38 | scale x/y/z | ≠ 1 | `setX/Y/ZScale:` |
| +0x3c/+0x40/+0x44 | rot x/y/z (deg) | ≠ 0 | `setX/Y/ZRot:` |
| +0x48/+0x4c/+0x50 | skew x/y/z | ≠ 0 | `setX/Y/ZSkew:` |
| +0x54 | packed colour u32 `0xAARRGGBB` | u32 ≠ 0 | `setRed:(R/255) green:(G/255) blue:(B/255) alpha:(A/255)` |
| +0x58/+0x5c/+0x60/+0x64 | red/green/blue/alpha | ≠ 1 | `setRed:` … `setAlpha:` (after the packed colour) |
| +0x68 | hidden | ≠ 0 | `setIsHidden:YES` |
| +0x69 | selectable | ≠ 0 | `setIsSelectable:YES` |
| +0x6c…+0x80 | colXMin, colXMax, colYMin, colYMax, colZMin, colZMax | ≠ 0 | `setColXMin:` … |

Node type (+0x0) → object: 1 `BCDisplayObject`; 2 top-level `BCDisplayGroup` (registered,
becomes the "current group" that following nodes are appended to); 3 nested group (appended
to the current group and registered; does **not** become current); 4 `BCDisplayMarker`
(+0x88 = id of a display group to put in it after all nodes of the list are parsed, last
marker first); 5 `BCRect`
(+0x84..+0x90 = minX, minY, maxX, maxY — the case falls through into the type-2 code and also
creates a phantom group, a latent original bug; no type-5 nodes exist in 1.48.2); 6 `BCModel`
(+0x88 texture-def id or nil); 7 `BCMultiModel`; 8 `BCTileModel` (+0x84 wrapWidth, +0x88
wrapHeight); 9 `BCGradient` (+0x88 gradient-def id, +0x90 width, +0x94 height); 10 gradient
def (+0x10 definition string, §7.1); 11 `BCText` (+0x88 font id, +0x90 centre flag →
justification 2, +0x91 right flag → justification 1, +0x98 text); 12 `BCFontDef` (+0x28
char-set string); 0 sound. (Full node layouts and data extraction: `struct-xml.md`.)

Default sequence-item handlers that manipulate the tree (`BCLibrary`, return 2 = done unless
noted; positional params `[i]`):

| Handler | Params | Behaviour |
|---|---|---|
| `drawDisplayGroup::: @ 0x1000244c8` | [0] group id | `BCView.setActiveDisplayGroup:` (§4.6) |
| `drawWideScreenOverlayDisplayGroup::: @ 0x10002452c` | [0] group id | §1.7 |
| `drawTallScreenOverlayDisplayGroup::: @ 0x100024590` | [0] group id | §1.7 |
| `addDisplayGroup::: @ 0x1000245f4` | [0] parentId, [1] id, [2] markerId | `child = [1] ? group([1]) : nil`; target = `[0] ? group([0]) : ([2] ? marker([2]) : activeDisplayGroup)`; `target.addDisplayObject:child` |
| `removeDisplayGroup::: @ 0x1000246bc` | [0] id | `group.removeFromDisplayGroup` |
| `hideObjects::: @ 0x1000246f8` / `showObjects::: @ 0x1000248bc` | [0] group id (`""` → active group), [1] comma-separated child ids | `getDisplayObject:` each → `hide` / `show` (missing ids ignored) |
| `setTextureDef::: @ 0x1000258b4` | [0] texture id, [1] group id (nil → active), [2] object id | `setTextureDef:` |

(`setAlpha`/`tweenAlpha`/`sinAlpha`/`setPos`/`tweenPos`/`blink`: §11.)

### 4.6 Active display group and hibernation

- `BCView::setActiveDisplayGroup: @ 0x100177570` (same pattern for the two overlay setters):
  if the new group differs: `old.hibernate` (if any), store, `new.unhibernate` (also when new
  is nil, messaging nil is a no-op). `activeDisplayGroup @ 0x100177560`.
- Ref-counted hibernation (`BCDisplayObject::hibernate @ 0x10010e454`,
  `unhibernate @ 0x10010e49c`; groups `@ 0x10006738c / 0x10006744c` also recurse into their
  children on the transitions):
  ```
  unhibernate: if refCount == 0 { mbHibernate = 0; logic.unhibernate; (group) each child.unhibernate }
               refCount += 1
  hibernate:   refCount -= 1
               if refCount == 0 { mbHibernate = 1; logic.hibernate; (group) each child.hibernate }
  ```
  No clamping: hibernating at refCount 0 drives it to −1, after which one `unhibernate` only
  returns it to 0 (still hibernating) — replicate.
- `setLogic: @ 0x10010e520` stores the logic (not retained) and immediately calls
  `logic.hibernate` or `logic.unhibernate` according to `mbHibernate`.
- `BCScreenLogic` (menus) swaps layers: `openScreen @ 0x10001cc90` moves `ScreenDisplayGroup`
  (root transform −160, 240, −90) into the current active group and zeroes its x, y and zRot;
  `disableGameLayer::: @ 0x10001cec4` removes it, copies the active group's pos/rot into it and
  makes it the active group (the island stops drawing and hibernates);
  `enableGameLayer::: @ 0x10001cfa4` moves it back (pos x,y = 0, rot.z = 0) and re-activates
  the remembered group; `closeScreenNow @ 0x10001cd88` restores the group and removes the
  screen layers.

---

## 5. Drawing

### 5.1 Traversal

`[root drawOpenGL:nil]` recursively. Each object draws itself immediately (painter's
algorithm). Draw order = depth-first, children in array order; later = on top. There is no
sorting, no batching semantics and no depth test. **Colour/alpha are not inherited** through
groups (exceptions: `BCText` reads its direct parent's colour, §5.7; multi-model children get
the parent's colour, §5.5).

Common symbols below: `p = mPos`, `r = mRot`, `k = mSkew`, `s = mScale`, `v = mPivotPos`,
`o = mOriginPos`, `(R,G,B,A) = (mRed, mGreen, mBlue, mAlpha)`, `kz = -277.12 / p.z`.

### 5.2 `BCDisplayGroup::drawOpenGL: @ 0x100067cf8`

```
if mbHidden: return
if children.count == 0: return
if mDisplayGroup == nil: glLoadIdentity()          // root of a draw: modelview = I
glPushMatrix()
translate(p.x, p.y, p.z); rotate(r.x, r.y, r.z); scale(s.x, s.y, s.z)
for child in children: [child drawOpenGL:self]
glPopMatrix()
```

No pivot, origin, skew or perspective factor; colour/alpha unused.
`drawInMultiModelOpenGL:red:green:blue:alpha: @ 0x100067eb4`: calls `[child drawOpenGL:self]`
on every child with **no** transform and no hidden check (colours ignored).

### 5.3 `BCModel` — a textured quad

`initWithId:textureDef: @ 0x100118720`; `mpTextureDef` (offset 240, may be NULL);
`mModelViewMatrix` (248, 16 floats) is never used. `textureDef @ 0x100118788`,
`setTextureDef: @ 0x100118798` (plain store). `enableMultiModelMode @ 0x100118a6c` /
`disableMultiModelMode @ 0x100118a84` set/clear `mbMultiModelMode`.

`BCModel::drawOpenGL: @ 0x1001187e8` *(asm-verified)*:

```
if mbHidden || mbMultiModelMode: return
setColor(R*A, G*A, B*A, A)                       // premultiplied tint
glPushMatrix()
translate(p.x, kz*(p.y - 160) + 160, p.z)
translate(kz*s.x*v.x, kz*s.y*v.y, s.z*v.z)
rotate(r.x, r.y, r.z)
skew(k.x, k.y)
scale(kz*s.x, kz*s.y, s.z)
translate(-v.x, -v.y, -v.z)
translate(o.x, o.y, o.z)
drawTextureDef(mpTextureDef, self)               // FUN_10003841c
glPopMatrix()
```

With `P' = (p.x, kz·(p.y − 160) + 160, p.z)`, `S' = diag(kz·s.x, kz·s.y, s.z)`,
`R = Rx·Ry·Rz` and `K` = skew, a local vertex `q` ends (in parent space) at
`P' + S'·v + R·K·S'·(q + o − v)`: the pivot `v` (in local units, after the origin shift) is
the fixed point of rotation and skew, while scaling is about the local origin.

`BCModel::drawInMultiModelOpenGL:red:green:blue:alpha: @ 0x1001189d4`: **no hidden check**,
ignores the passed colour: `setColor(R*A, G*A, B*A, A)`; `translate(o)`;
`drawTextureDef(mpTextureDef, self)`.

**`drawTextureDef` = `FUN_10003841c @ 0x10003841c`**:

```
if texDef == NULL: return
bindTexture(*texDef.textureName)                  // texDef+0x18 → GL name of the atlas
vertexPointer(texDef + 0x50)                      // 4 × (x,y,z) floats
texCoordPointer(texDef + 0x30)                    // 4 × (u,v) floats
setBlend(texDef.blend /* +8 BOOL */ != 0  ||  [displayObject alpha] < 1.0)
glDrawElements(GL_TRIANGLE_STRIP, 4, GL_UNSIGNED_SHORT, {0,1,2,3} @ 0x10029d508)
```

`BCTextureDef` fields used by the renderer (full layout in the texture spec): +0x08 `blend`
BOOL (1 = translucent, always blend), +0x18 `uint*` texture-name pointer, +0x20 sprite width,
+0x24 sprite height, +0x28 frame width, +0x2c frame height, +0x30 `float[8]` uv, +0x50
`float[12]` vertices. The quad is precomputed in the data, in local pixel units with **y up**
and the frame's bottom-left at (0,0); trimmed sprites sit inside the frame. Strip order:
`v0` top-left, `v1` top-right, `v2` bottom-left, `v3` bottom-right; uv `t = 0` is the first
(top) row of the atlas, e.g. `HalloweenCobweb`: sprite 68×54 in an 80×60 frame,
`v0 = (0,60)`, `v1 = (68,60)`, `v2 = (0,6)`, `v3 = (68,6)`. No face culling, so negative
scales (mirroring) render normally. Code commonly centres a sprite with
`origin = (−frameW/2, −frameH/2)`.

### 5.4 Texture-only fragment rule

For every textured draw: `out_rgba = texel_rgba × current_colour` (GL_MODULATE); then, if
blending is on, `dst = out + dst·(1 − out.a)`, else `dst = out`. OpenGL ES 1.1 clamps
`glColor4f` and colour-array components to [0, 1] (tints/alphas > 1 or < 0 from the data act
as 1 or 0), and the framebuffer result is clamped to [0, 1].

### 5.5 `BCMultiModel` — animated composite sprite

Fields: `mpBinaryFrameDef` (312), `mpTweenableChannelArray` (320), `mModelIndices` (328,
`short[4] = {0,1,2,3}`, set in `initWithId: @ 0x100181694`, which passes a NULL texture def).
`setTextureDef: @ 0x100181924` stores the texture def and clears the frame def and channel
array (texture mode); `setBinaryFrameDef:tweenableChannelArray: @ 0x100181950` stores both and
clears the texture def (animation mode); `textureDef @ 0x100181910`, `binaryFrameDef
@ 0x10018197c`. The animation system (`timelines.md`) sets the current `BinaryFrameDef` every
frame.

`BinaryFrameDef`: +0x20 `i64 stateCount`, +0x28 `BinaryFrameState**`. Fields of a frame state
read by the renderer (offsets from the state pointer, built by
`BCLibrary::parseBinAnim:ext: @ 0x100025a08`):

| Off | Type | Meaning |
|---|---|---|
| +0x08 | u32 flags | bit0 use state x; bit1 use state y; bit5 use state rot; bit8 use state scaleX; bit9 use state scaleY; bit13 asset is a texture; bit14 asset is a dynamic text; bit31 "pointers resolved" |
| +0x18 | BCTextureDef* | resolved texture (bit13) or NULL |
| +0x20 | i64 | tweenable channel index (0 = none) |
| +0x28 | id | resolved dynamic `BCText` (bit14) or nil |
| +0x30, +0x34, +0x38 | f32 | x, y, zRot (degrees) |
| +0x3c, +0x40 | f32 | skew x / skew y — never read at runtime (`timelines.md` §2.8) |
| +0x44, +0x48 | f32 | scaleX, scaleY |
| +0x4c | f32 | alpha |
| +0x50, +0x54, +0x58 | f32 | red, green, blue |
| +0x5c | f32[12] | precomputed quad (v0..v3, local to the multi model) |

**`BCMultiModel::drawOpenGL: @ 0x10018198c`** *(asm-verified)*:

```
if mbHidden || mbMultiModelMode: return
glPushMatrix(); <same transform sequence as BCModel, incl. kz, pivot, skew, origin>
if mpTextureDef != NULL:
    setColor(R*A, G*A, B*A, A); drawTextureDef(mpTextureDef, self)
elif mpBinaryFrameDef != NULL:
    for st in frame.states (in order):
        child = st.tweenable (+0x28)  ??  channels[st.channel].mTweenable
        if child == nil:
            if st.textureDef == NULL: continue
            a = st.alpha * A
            setColor(a*st.r*R, a*st.g*G, a*st.b*B, a)
            bindTexture(st.textureDef); setBlend(st.textureDef.blend || a < 1)
            vertexPointer(st + 0x5c); texCoordPointer(st.textureDef + 0x30)
            drawElements(strip, 4, mModelIndices)
        else:
            glPushMatrix()
            x  = bit0 ? st.x : (child.pos.x - p.x) / s.x - o.x
            y  = bit1 ? st.y : (child.pos.y - p.y) / s.y - o.y
            if bit5: rz = st.rot
            elif s.x > 0: rz = child.rot.z - r.z
            elif bit8:    rz = r.z + (180 - child.rot.z)
            else:         rz = r.z - child.rot.z
            sx = bit8 ? st.scaleX : child.scale.x * s.x
            sy = bit9 ? st.scaleY : child.scale.y * s.y
            a = st.alpha*A; cr = st.r*R; cg = st.g*G; cb = st.b*B
            if st.textureDef != NULL: [child setTextureDef: st.textureDef]
            setColor(a*cr, a*cg, a*cb, a)
            translate(x, y, 0); rotate(0, 0, rz); scale(sx, sy, 1)
            [child drawInMultiModelOpenGL:self red:cr green:cg blue:cb alpha:a]
            glPopMatrix()
glPopMatrix()
```

**`BCMultiModel::drawInMultiModelOpenGL:red:green:blue:alpha: @ 0x100181f60`** (nested
multi model, params `r,g,b,a`) *(asm-verified)*:

```
if mbHidden: return
translate(o)
if mpTextureDef != NULL: drawTextureDef(mpTextureDef, self); return   // uses caller's colour
for st in frame.states:
    if st.textureDef != NULL:                                           // texture first here
        aa = st.alpha*A*a
        setColor(aa*st.r*R*r, aa*st.g*G*g, aa*st.b*B*b, aa)
        bind/blend(st.textureDef.blend || aa < 1)/draw st quad as above
    else:
        child = st.tweenable ?? channels[st.channel].mTweenable;  if nil: continue
        glPushMatrix()
        aa = st.alpha*A*a; cr = st.r*R*r; cg = st.g*G*g; cb = st.b*B*b
        x = bit0 ? st.x : child.pos.x;    y = bit1 ? st.y : child.pos.y
        rz = bit5 ? st.rot : child.rot.z
        sx = bit8 ? st.scaleX : child.scale.x;  sy = bit9 ? st.scaleY : child.scale.y
        setColor(aa*cr, aa*cg, aa*cb, aa)
        translate(x, y, 0); rotate(0,0,rz); scale(sx, sy, 1)
        [child drawInMultiModelOpenGL:self red:cr green:cg blue:cb alpha:aa]
        glPopMatrix()
```

Differences to note: the top-level draw checks the tweenable first and calls
`setTextureDef:` on it; the nested draw checks the texture first and never calls it; the
nested draw uses child values without the parent-relative conversion. A child object whose
`mbMultiModelMode` is set skips its own `drawOpenGL:` and is drawn only this way.

### 5.6 `BCTileModel` — wrapping scrolling strip

`initWithId:wrapWidth:wrapHeight: @ 0x100021ec0`: `mbVertical = (wrapWidth == 0)`,
transition window `maxX = maxY = 1000`, `minX = minY = −1000`. `setBinaryFrameDef:
@ 0x100021ff8` stores the frame def and clears the texture def. `drawOpenGL: @ 0x100022b64`
dispatches to the vertical or horizontal draw. The frame's states are the tiles (each must
have a texture; state rgb is ignored).

`drawOpenGLHorizontal: @ 0x100022618` *(asm-verified)*:

```
if mbHidden || mpBinaryFrameDef == NULL: return
off = Σ xPos of every ancestor group except the top-most one (the parent chain is walked
      until a group with no parent; that root's xPos is excluded; parent == root → off = 0)
X = off + p.x
m = (float)fmod((double)X, (double)wrapWidth);  if X > 0: m -= wrapWidth   // m ∈ [-W, 0]
glPushMatrix(); translate(m - off, p.y, p.z)
maxX = min(mTransitionMaxXPos - m, 480 - m)      // strict '<' picks the first
minX = max(mTransitionMinXPos - m, -m)
for st in states:
    lo = min(v0.x, v1.x, v2.x, v3.x); hi = max(...)   // st+0x5c, +0x68, +0x74, +0x80
    if lo <= maxX && hi >= minX:          // otherwise skipped (NaN → skipped)
        a = st.alpha*A; setColor(a*R, a*G, a*B, a)
        bind/setBlend(st.textureDef.blend || a < 1)/draw st quad
glPopMatrix()
```

`drawOpenGLVertical: @ 0x100022908` *(asm-verified)*: `gy = parent.yPos` (direct parent only),
`Y = gy + p.y`, `m` = wrap of `Y` by `wrapHeight` as above, `translate(p.x, m − gy, p.z)`,
`maxY = min(transitionMaxY − m, 320 − m)`, `minY = max(transitionMinY − m, −m)`, draw a state
unless `v2.y > maxY` (bottom, st+0x78) or `v0.y < minY` (top, st+0x60); same colour rule.
No perspective factor, rotation, scale or origin in either variant.

Transition helpers (used by `StormCloudLogic` for storm clouds rolling in/out):

| Method | Effect |
|---|---|
| `resetTransitionVisible @ 0x10002205c` | maxX = 1000, minX = −1000 |
| `resetTransitionInvisible @ 0x100022080` | maxX = −1000, minX = 1000 |
| `startTransitionInLeft @ 0x1000220a4` | `m = wrap(p.x)` (no ancestor offset); `lim = 480 − m`; for each state with `v1.x > −m && lim > v0.x`: `lim = v0.x`; maxX = m + lim; minX = 0 |
| `startTransitionInRight @ 0x100022198` | `lim = −m`; for each state with `v0.x > 480 − m && lim < v0.x`: `lim = v0.x`; minX = m + lim; maxX = 480 |
| `startTransitionOutLeft @ 0x100022178` / `startTransitionOutRight @ 0x10002226c` | maxX = 480, minX = 0 |
| `transitionInLeft:(d) @ 0x10002228c` | maxX += d; return maxX > 480 |
| `transitionOutLeft:(d) @ 0x1000222b4` | minX += d; return minX > 480 |
| `transitionInRight:(d) @ 0x1000222dc` | minX += d; return minX < 0 |
| `transitionOutRight:(d) @ 0x1000222fc` | maxX += d; return maxX < 0 |
| `startTransitionInBottom @ 0x10002235c` | `m = wrap(p.y)`; `lim = 320 − m`; for each state with `v2.y > −m && lim > v0.y`: `lim = v0.y`; maxY = m + lim; minY = 0 |
| `startTransitionInTop @ 0x100022454` | `lim = −m`; for each state with `v0.y > 320 − m && lim < v0.y`: `lim = v0.y`; minY = m + lim; maxY = 320 |
| `startTransitionOutBottom @ 0x100022434` / `startTransitionOutTop @ 0x10002252c` | maxY = **480** (original bug) / 320; minY = 0 |
| `transitionInBottom: @ 0x10002254c` / `transitionOutBottom: @ 0x100022574` / `transitionInTop: @ 0x10002259c` / `transitionOutTop: @ 0x1000225bc` | maxY += d → maxY > 320 / minY += d → minY > 320 / minY += d → minY < 0 / maxY += d → maxY < 0 (vertical variants are unused in 1.48.2) |
| accessors `transitionMax/MinX/YPos`, setters `@ 0x10002201c … 0x10002234c` | plain |

### 5.7 `BCText` — bitmap text

Fields: `mText` (240, copy), `mTextBuffer` (248, `char[1024]`), `mFontDef` (1272),
`mFontJustification` (1280: 0 left, 1 right, 2 centre), `mWrapWidth` (1284, 0 = no wrap),
`mLineCount` (1288, default 100, not used by drawing), `mTextLength` (1292), `mCharCount`
(1296), `mbModified` (237, set by setters, never read). `initWithId:fontDef: @ 0x1001a75b0`
also initialises the shared glyph index buffer `{0,1,2,3}` at `DAT_1007e1f62` once.

`setText: @ 0x1001a781c`: copies the UTF-8 bytes of the string (max 1023) into
`mTextBuffer`, **stripping leading and trailing whitespace** (`' '`, `'\t'`, `'\n'`, `'\r'`;
inner whitespace kept), stores `mTextLength`, keeps a copy in `mText`. nil → empty.
`useTextBuffer @ 0x1001a779c` (releases `mText`, returns the buffer for direct writes) +
`showTextBuffer @ 0x1001a77e4` (`mTextLength = strlen`) are the C-string path.
`textWidth @ 0x1001a7760` = `BCFontDef::textWidth:` of the buffer.

`updateDisplayText @ 0x1001a7670` (called on every draw): if `mWrapWidth == 0` copy the
buffer (≤ 1023 bytes, terminated at `mTextLength`) into the global `DAT_1007e1f6a`, else
`copyWordWrappedText` (§6.3) into it; then `mCharCount = fillFontCharTextureArray(global
glyph array DAT_1007d1f60, DAT_1007e1f6a, mFontJustification)` (§6.4).

`BCText::drawOpenGL: @ 0x1001a7b28` *(asm-verified)*:

```
if mbHidden: return
updateDisplayText
P = mDisplayGroup                                   // direct parent only (nil → 0s)
a = P.alpha * A
setColor(P.red*R*a, P.green*G*a, P.blue*B*a, a)
glPushMatrix()
translate(p.x, kz*(p.y - 160) + 160, p.z); rotate(r); scale(kz*s.x, kz*s.y, s.z)
for each glyph entry e (mCharCount):
    bindTexture(e.texture); setBlend(1); vertexPointer(e.verts); texCoordPointer(e.uv)
    drawElements(strip, 4, {0,1,2,3})
glPopMatrix()
```

No pivot, skew or origin. `drawInMultiModelOpenGL:… @ 0x1001a7d70`: `updateDisplayText`,
then the glyph loop with the caller's current colour and transform (no hidden check, no own
transform). `BCText` never enters multi-model mode itself (no override); dynamic texts from
`Timelines.bin` are not in any display group and are drawn only through multi models.

### 5.8 `BCRect` — untextured rectangle

`initWithId:minXPos:minYPos:maxXPos:maxYPos: @ 0x1000219e0` → `createModel @ 0x100021ca8`
(4 vertices, indices `{0,1,2,3}`) → `calcModelVertices @ 0x100021c28`:
`v0 = (minX, maxY, 0)`, `v1 = (maxX, maxY, 0)`, `v2 = (minX, minY, 0)`,
`v3 = (maxX, minY, 0)`. Each `setMin/MaxX/YPos:` (`@ 0x100021af0 … 0x100021b74`) recomputes the
vertices; `width @ 0x100021b90` = maxX − minX, `height @ 0x100021bb0` = maxY − minY.

`BCRect::drawOpenGL: @ 0x100021d58` *(asm-verified)* — **no hidden check**:

```
setBlend(A < 1); setColor(R*A, G*A, B*A, A)
glPushMatrix(); translate(p.x, kz*(p.y-160)+160, p.z); rotate(r); scale(kz*s.x, kz*s.y, s.z)
disableTexture2D; vertexPointer(verts); drawElements(strip, 4); enableTexture2D
glPopMatrix()
```

Only user: the pygmy name label background (`PygmyLogic::initWithDisplayObject:`,
rect (−50,0)–(50,14), colour (0,0,0,0.5)).

### 5.9 Hidden flag, blending and colour per class (summary)

| Class / call | honours `mbHidden` | blending | colour |
|---|---|---|---|
| `BCDisplayGroup::drawOpenGL:` | yes (skips subtree) | — | not used |
| `BCDisplayMarker::drawOpenGL:` | no | — | — |
| `BCModel::drawOpenGL:` | yes (+ skips if multi-model) | `blend` flag ∨ A<1 | (RA,GA,BA,A) |
| `BCModel::drawInMultiModel…` | **no** | same | own (RA,GA,BA,A) |
| `BCMultiModel::drawOpenGL:` / nested | yes / yes | per state | §5.5 |
| `BCTileModel` | yes | per tile | (aR,aG,aB,a), a = st.alpha·A |
| `BCText::drawOpenGL:` / nested | yes / no | always on | parent × own / caller's |
| `BCRect` | **no** | A < 1 | (RA,GA,BA,A) |
| `BCGradient` | yes / nested no | **inherited** | per-vertex (straight) |
| `RainbowModel` | no | on | per-vertex |
| `LightningModel`, `HurricaneModel`, `WhirlPoolModel`, `PathModel` | no | **inherited** | fixed formulas (§8) |
| `LocustModel` | no | on | **inherited** |
| `MagnifyingBeamModel` | no | **inherited** | per-vertex |

---

## 6. Fonts and text layout (`BCFontDef`)

### 6.1 Font data

Text is a **bitmap font in the texture atlases**, not UIKit rendering. A `BCFontDef`
(`initWithId: @ 0x100066780`) has `mId`, `mCharSet` and `mTextureDefMap[256]`
(`BCTextureDef*` per byte value, zeroed). `setCharSet: @ 0x100066a34` *(asm-verified)*: for
`i` in `0 ..< charSet.length`: `map[(uint8)charSetUTF8[i]] = textureDef("<fontId>:FullSet:<i>")`
(format `@"%@:FullSet:%d"` at 0x1003917d0; `i` is the index in the char-set string, not the
character code). Fonts defined in 1.48.2 (struct-XML root 0x10064a270):

| Font id | Char set |
|---|---|
| `Arial12`, `Arial12Black`, `AnimatedGothicLite20` | set A below (95 glyphs) |
| `Doop24Green` | set B below (12 glyphs) |

Char sets, exactly as stored (index 0 is a space character in both):

```
A:  !"#$%&'()*+,-./0123456789:;<=>?@[\]^_`{|}~ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz
B:  L0123456789
```

(after `A:`/`B:` comes one separator space, then the set starting with its own space).

Glyph metrics come from the glyph's texture def: **advance = frame width (+0x28)**, **line
height = frame height (+0x2c)**; the glyph quad is positioned inside its frame with the
frame's bottom-left at the pen position (e.g. `Arial12:FullSet:33` = 4×12 frame; the space
glyph has a zero-size quad and advance 5). Characters without a mapping are skipped (no
advance).

### 6.2 `textWidth:(cstr) @ 0x100066818`

Max over lines (split at `'\n'`) of the sum of advances of mapped characters.

### 6.3 Word wrap — `copyWordWrappedText:destinationBuffer:textLength:width: @ 0x100066970` *(asm-verified)*

```
out = 0; lastSpace = -1; lineW = 0; widthAtSpace = 0
for i in 0 ..< len:
    c = (signed char)src[i]
    if c == '\n': dst[out++] = '\n'; lineW = 0; continue      // lastSpace NOT reset
    g = map[c]                                              // bytes ≥ 0x80 are dropped
    if g == NULL || c < 0: continue                         // unmapped chars dropped
    lineW += g.advance; dst[out] = c
    if c == ' ': lastSpace = i; widthAtSpace = lineW        // i = SOURCE index
    if lastSpace != -1 && lineW >= width:
        dst[lastSpace] = '\n'                               // indexes dst with a source index
        lineW -= widthAtSpace; lastSpace = -1
    out += 1
dst[out] = 0
```

The source/destination index mismatch (after dropped characters) is original behaviour and
must be kept. `lengthOfWordsThatFit:width: @ 0x1000668a8`: scans the UTF-8 bytes until `'\n'` or the end;
remembers the index of the last space (0 if none yet); as soon as the running sum of mapped
advances reaches `width` (≥) it returns that remembered index, otherwise it returns the index
where the scan stopped (used by news/story/challenge UI code).

### 6.4 Glyph layout — `fillFontCharTextureArray:textBuffer:fontJustification: @ 0x100066c08` *(asm-verified)*

Glyph entry (`FontCharTexture`, 0x40 bytes): `+0 u32 GL texture name`, `+8 float* uv`
(glyph texDef + 0x30), `+0x10 float[12]` vertices.

```
penX = 0; penY = 0; lineH = 0; lineW = 0; n = 0
for i in 0 ..< strlen(text):
    if just != 0 && lineW == 0:
        for k = i; k < len && text[k] != '\n'; k++: lineW += advance(text[k]) (mapped only)
        if just == 1: penX = -floorf(lineW)          // right: text ends at x = 0
        if just == 2: penX = -floorf(lineW * 0.5)    // centre
    if text[i] == '\n': penY -= lineH; penX = 0; lineW = 0
    elif g = map[text[i]]:
        e[n] = {g.texture, g.uv, g.verts with x += penX, y += penY}
        lineH = g.frameHeight; penX += g.advance; n += 1
return n
```

Lines go **down** by the frame height of the most recently emitted glyph. Left-justified
text starts at x = 0 with the first line's frame bottom at y = 0.

`BCFontDef::drawOpenGL:text:fontJustification: @ 0x100066db0` is dead code (no caller); it
would loop forever for justification ≠ 0 (its measuring loop never advances the index).

---

## 7. Gradients (`BCGradient`)

### 7.1 Gradient definitions

`BCGradient::createGradientDef:(string) @ 0x100178500` *(asm-verified)*: split on `","`;
each component is parsed with `sscanf("%f:%x")` → `pos`, `u32 c`;
`r = ((c>>16)&0xff)/255, g = ((c>>8)&0xff)/255, b = (c&0xff)/255, a = (c>>24)/255`. Layout:
`BCGradientDef = {i32 count; BCGradientColor colors[count]}`, `BCGradientColor = {f32 pos,
r, g, b, a}` (20 bytes). Stops are kept in string order (the data lists pos 1.0 → 0.0).
Definitions in 1.48.2: `IslandGradient`, `SunsetGradient`, `SpaceGradient`,
`ApocalypseDay/Sunset/Night/CloudyGradient`, `NightGradient`, `DayGradient`,
`CloudyGradient`, `EclipseGradient`, `BigSunGradient`, `SmallSunGradient`, `StartGradient`
(contains the stop `0.05:0xf5FDEFE`, i.e. alpha 15/255 — data typo, keep it),
`ChallengeGradient`. `gradientColorAtIndex:(i) @ 0x1001787bc` returns `&colors[i]`.

### 7.2 Geometry

`initWithId:width:height:gradientDef: @ 0x100178714` → `createModel @ 0x1001788b4`:
`vertexCount = indexCount = 2·count`; for stop `i`: `vert[2i] = (0, H·pos_i, 0)`,
`vert[2i+1] = (W, H·pos_i, 0)`, both coloured `(r_i, g_i, b_i, a_i)` (straight, not
premultiplied); indices `2i, 2i+1`; drawn as one triangle strip (a vertical ramp, smooth
shaded). The `mpGradientDef` pointer is the library's shared def.

`BCGradient::drawOpenGL: @ 0x100178d18`:

```
if mbHidden: return
glPushMatrix()
translate(p) ; translate(s.x*v.x, s.y*v.y, s.z*v.z); rotate(r); scale(s); translate(-v); translate(o)
enableColorArray; disableTexture2D; colorPointer; vertexPointer
drawElements(strip, indexCount); enableTexture2D; disableColorArray
glPopMatrix()
```

No perspective factor, no skew, ignores the object's colour/alpha, **does not set blending**.
`drawInMultiModelOpenGL:… @ 0x100178e70`: the same draw without any transform.

### 7.3 Fading

- `setGradient:(def) @ 0x100178be4`: copies all stops of `def` into the gradient's own def
  (overwriting the shared library def) and rebuilds vertex y and colours (same count assumed).
- `fadeGradient:(def) percent:(t) @ 0x100178aa0`: for every stop and every field (pos, r, g,
  b, a): `own = own + (def − own)·t`, written back into the def and the vertex arrays.

### 7.4 Island sky (`PocketGodViewController(Game)::updateGradients @ 0x1001592b8`)

Runs at the end of every `PocketGodViewController` game frame. Requires all of
`mpSunset/Night/Day/Cloudy/Eclipse/BigSun/SmallSun/ApocalypseSunset/ApocalypseNight/
ApocalypseDay/ApocalypseCloudyGradient`. If any of the seven fades (`Cloud`, `Sunset`,
`Night`, `Eclipse`, `BigSun`, `SmallSun`, `Apocalypse`) differs from its target:

```
each fade += (target - fade) * 0.13; if |fade - target| < 0.01: fade = target
G = IslandDisplayGroup.getDisplayObject("Gradient")
G.setGradient(Day)
if apoc  != 0: G.fade(ApocalypseDay, apoc)
if cloud != 0: G.fade(Cloudy, cloud);  if apoc != 0: G.fade(ApocalypseCloudy, apoc*cloud)
if bigSun   != 0: G.fade(BigSun, bigSun)
if smallSun != 0: G.fade(SmallSun, smallSun)
if sunset != 0: G.fade(Sunset, sunset); if apoc != 0: G.fade(ApocalypseSunset, apoc*sunset)
if night  != 0: G.fade(Night, night);   if apoc != 0: G.fade(ApocalypseNight, apoc*night)
if eclipse != 0: G.fade(Eclipse, eclipse)
c = G.gradientColorAtIndex(0); BCView.setClearColorRed:c.r green:c.g blue:c.b alpha:1.0
```

(The targets are driven by gameplay; see the gameplay spec.) The `Gradient` model in
`IslandDisplayGroup` uses `IslandGradient`, is 480×461 and sits at (0, 120, −277.12), so the
ramp covers game y 120…581.

---

## 8. Custom geometry models

All of these derive directly from `BCDisplayObject`, ignore `mbHidden`, use only `translate
(pos)` (no rotation/scale/origin/perspective), are created in code with `zPos = -277.12`,
and allocate their own vertex/index arrays (`createModel`/`freeModel`).

### 8.1 Shared path-strip generator

`RainbowModel::calcModelVertices… @ 0x10008f050`, `LightningModel::calcModelVertices…
@ 0x1000bf914` and `PathModel::calcModelVertices… @ 0x1001250e8` use the same algorithm
*(asm-verified)* over an ordered point list `P[0..n-1]` (only x, y used) with half-width `w`:

```
for i in 0 ..< n-1:                                // n-1 iterations
    d = normalize(P[i+1] - P[i]); ang = atan2f(d.y, d.x)
    A = P[i] + (sin ang, -cos ang)*w;  B = P[i] + (-sin ang, cos ang)*w     // A = right side
    if i > 0:
        q = normalize(P[i-1] - P[i]); bx = d.x + q.x
        if bx != 0: by = d.y + q.y
            if by != 0:                             // both components non-zero
                ang = atan2f(by, bx); c = cos ang*w; s = sin ang*w
                if d.x*by - d.y*bx > 0: A = P[i] - (c,s); B = P[i] + (c,s)
                else:                    A = P[i] + (c,s); B = P[i] - (c,s)
    P[i].w = ang                                    // written back into the point (side effect)
    vert[2i] = (A, 0); vert[2i+1] = (B, 0); idx[2i] = 2i; idx[2i+1] = 2i+1
last point: d = normalize(P[n-1] - P[n-2]); ang = atan2f(d.y, d.x)
    A/B = perpendicular offsets as above; P[n-1].w = ang; vert/idx 2(n-1), 2(n-1)+1
```

Result: a triangle strip of `2n` vertices; joints use the angle bisector with the same
half-width (no miter lengthening), so sharp corners narrow. The written-back angle (`.w`) is
used by `LightningLogic` to orient branch sprites.

### 8.2 `RainbowModel`

Created in `PocketGodViewController::loadGame @ 0x10006a030` (id `Rainbow`, zPos −277.12)
over the shared `std::list<VECTOR4>` `mpRainbowList` (the `LocustModel`, id `Locust`, is
created right after it); logic `RainbowLogic` (the model is inserted after
`FireworksDisplayMarker` by `RainbowLogic::startDrawRainbow @ 0x100020ca0`).
`initWithId:pathList: @ 0x10008ef14`, `pathList @ 0x10008ef6c`.

- `drawOpenGL: @ 0x10008f5d8`: if `list.size < 2` return; **`createModel` every frame**
  (`@ 0x10008f4a4`: vertexCount = indexCount = 2·size, half-width **50.0**); push;
  `translate(pos)`; enable colour array; bind the `Rainbow` texture's atlas
  (`TextureAtlasTest0`, def at 0x10052fab0); blend on; texcoord, colour, vertex pointers;
  draw strip; disable colour array; pop.
- Colours and uvs (`calcModelVertices…lineWidth: @ 0x10008f050`), `n = size`:
  ```
  fade = (n >= 10) ? 5 : n/2            (integer division)
  step = (n >= 10) ? 0.2 : 1/(float)(n/2)
  ramp = 0
  for pair i in 0 ..< n-1:
      c = ramp * self.alpha; both vertices colour (c,c,c,c)   // premultiplied white
      uv(A) = (Rainbow.uv[0], Rainbow.uv[1]); uv(B) = (Rainbow.uv[0], Rainbow.uv[5])
      if 2i+2 < 2*(n - fade): ramp = min(ramp + step, 1) else ramp = max(ramp - step, 0)
  last pair: colour (0,0,0,0), same uvs
  ```
  i.e. the whole strip samples one texel column (u = left edge of the `Rainbow` sprite) from
  its top (right-hand side of the path) to its bottom (left side), fading in over the first
  `fade` points and out over the last ones.
- Path data (gameplay): `PocketGodViewController(Game)::startDrawRainbow: @ 0x100161a54`
  clears the list; `drawRainbow: @ 0x100161a9c` appends the touch point (island coordinates)
  whenever it is > 5 units from the last added point (after the first point > 20 units away
  starts drawing and creates the `RainbowTrail` finger sprite); `stopDrawRainbow:
  @ 0x100161db4` appends the final point if it moved.

### 8.3 `LocustModel` (point sprites)

`initWithId: @ 0x100069b88`; `setLocustPointArray:count: @ 0x100069c00` (pointer to `x,y`
float pairs, not copied). `drawOpenGL: @ 0x100069c1c`:

```
if points == NULL: return
push; translate(pos); setBlend(1); enablePointSprite (COORD_REPLACE); disableTexCoordArray
glPointSize(16.0)                       // LocustAtlas0 sprite width (DAT_1007b8fc0)
glVertexPointer(2, GL_FLOAT, 0, points); bindTexture(LocustTextureAtlas0)  // 16×16
glDrawArrays(GL_POINTS, 0, count); disablePointSprite; enableTexCoordArray; pop
```

Each locust is the whole 16×16 `LocustTextureAtlas0` texture as a screen-aligned square of
16 **renderbuffer pixels** (= 16 game units on iPhone, 7.5 on iPad), centred on the point,
modulated by the **current GL colour** (never set here). Point data:
`LocustLogic::sendLocustPosToModel @ 0x10008c628` packs the `(x, y)` (state +0xc, +0x10) of
every active entry (state +0 ≠ 0) of the 60 `LocustState`s (0xb0 bytes each at
`0x1007e50a8`) into `DAT_1007e79e8`, called at the end of `LocustLogic::onGameFrame:
@ 0x10008c840` (locust behaviour: see the gameplay spec).

### 8.4 `LightningModel`

`initWithId:pathList: @ 0x1000bf768` **copies** the list (`FUN_1000bfe3c`) and builds the
geometry once (`createModel @ 0x1000bfbb0`: if size == 1 → empty; else three strips from the
same points with half-widths **2, 6, 9** → inner, middle, outer). `drawOpenGL: @ 0x1000bfd14`
*(asm-verified)*:

```
push; translate(pos); disableTexture2D
c = alpha*0.2: setColor(c,c,c,c); draw outer strip
               setColor(c,c,c,c); draw middle strip
setColor(alpha,alpha,alpha,alpha); draw inner strip
enableTexture2D; pop                     // blending inherited
```

Path (gameplay, `PocketGodViewController(Game)`): `startDrawLightning: @ 0x100160d3c`
starts the list at the touch point and spawns an `ElectricGlow` finger sprite;
`drawLightning: @ 0x1001610cc` appends the touch point whenever it is more than 20 units
(3D length, `FUN_1001a15d4`) from the last appended point, or, when the touch hits a pygmy /
ape torch / monkey / bird / bonfire that accepts `hitByLightning`, appends that object's
position (or the touch point for torches/monkeys) and triggers immediately; `triggerLightning @ 0x100160f74`
creates the model in `IslandDisplayGroup` with a `LightningLogic`
(`initWithDisplayObject: @ 0x1000c8d04` adds `LightningGlow_01` glow sprites at both ends and
two `LightningBranch_01` sprites at random interior points, rotated by the stored `.w` angle
(+180° for one)); `hitPygmyWithWarlockLightning: @ 0x100161818` builds a 7-point vertical bolt
from y = 300 to the pygmy (x jitter ±50 on interior points).
`LightningLogic::blinkLightning::: @ 0x1000c924c` toggles alpha 1 ↔ 0 every frame for its
duration; `die::: @ 0x1000c9424` removes the highlights.

### 8.5 `HurricaneModel` and `WhirlPoolModel` (wind ribbons)

Both hold 5 wind entries (`[5]`, 0x40 bytes each at offset 244; the `…WindPosCount` ivar is
ignored by drawing):

| Off | Field |
|---|---|
| 0x00 | `mPos` vec4 |
| 0x10 | `mPosVel` vec4 |
| 0x20 | `mWindRot` (rad) |
| 0x24 | `mWindRotVel` (rad per segment) |
| 0x28 | `mRadius` |
| 0x2c | `mRadiusVel` (per segment) |
| 0x30 | `mDrawPos` (segment phase offset) |
| 0x34 | `mDrawPosVel` |
| 0x38 | `mLineWidth` (unused by drawing) |
| 0x3c | `mSegmentCount` (i32, ≤ 20: arrays are 40 indices / 40 vertices) |

`HurricaneModel::calcModelVertices:modelIndices:hurricaneWindPos: @ 0x10018b884`
*(asm-verified)*:

```
ang = WindRot + DrawPos*WindRotVel; rad = Radius + DrawPos*RadiusVel
y   = Pos.y + PosVel.y*DrawPos
for i in 0 ..< SegmentCount:
    (s, c) = sincos(ang)
    vert[2i]   = (Pos.x + rad*c, y,       Pos.z + rad*s)
    vert[2i+1] = (Pos.x + rad*c, y - 2.0, Pos.z + rad*s)
    rad += RadiusVel; ang += WindRotVel; y += PosVel.y
counts = 2*SegmentCount
```

`HurricaneModel::drawOpenGL: @ 0x10018ba54`: push; translate(pos); disableTexture2D;
`setColor(0, 0, 0, alpha)`; for each of the 5 entries: calc + draw strip; enableTexture2D;
pop.

`WhirlPoolModel::calcModelVertices:modelIndices:whirlPoolWindPos: @ 0x10011aca4`: same but a
flat spiral: `vert = (Pos.x + rad*c, Pos.y + rad*s, 0)` and `(…, Pos.y + rad*s − 2.0, 0)`
(no per-segment y advance). `drawOpenGL: @ 0x10011ae58`: colour `(0, 0.8a, 0.8a, a)`.
`initWithId: @ 0x10011ab88` zeroes the array and allocates 40 indices / 40 vertices
(`createModel @ 0x10011ae04`; hurricane `@ 0x10018ba00`).

Animation (gameplay logic, per frame, not dt-scaled):

- `HurricaneLogic::initWithDisplayObject: @ 0x100113cd8`, per entry: `Pos = (rand%401+2200)/10,
  86, 0`, `PosVel = (0, 1, 0)`, `WindRot = 0`, `WindRotVel = 0.2`, `Radius = (rand%201+500)/10`,
  `RadiusVel = (rand%21+30)/100`, `DrawPos = (rand%1801)/10`, `DrawPosVel = (rand%151+150)/100`,
  `LineWidth = 2`, `SegmentCount = rand%5+6`. `onGameFrame: @ 0x100114a04`: `DrawPos +=
  DrawPosVel*mWindVel`, wrapping to 0 when > 180; `mWindVel −= 0.01` (−0.05 more while
  stopping), clamped ≥ 0; `vis = mWindVel ≥ 0.5 ? 1 : 2*mWindVel`; model alpha = `vis*0.5`;
  rain sprites alpha = vis, zRot = 30, move (+5, −30) per frame, respawn at
  `x = (rand%5301−500)/10, y = 320` when below −30; particles alpha = vis, zRot += 10.
  The model is created at pos (0, 0, −277.12) in `IslandDisplayGroup` by
  `PocketGodViewController(Game)::startHurricane: @ 0x100162384`.
- `WhirlPoolLogic::initWithDisplayObject: @ 0x100137600`: `Pos = (240, 160, 0)`,
  `PosVel = 0`, `WindRot = 0`, `WindRotVel = 0.2`, `Radius = (rand%401+1000)/10`,
  `RadiusVel = (rand%21−100)/100`, `DrawPos = (rand%1801)/10`, `DrawPosVel = (rand%21+80)/100`,
  `LineWidth = 2`, `SegmentCount = (rand&3)+5`. `onGameFrame: @ 0x100137cf8`: same `DrawPos`
  update and `mWindVel` decay; alpha = `mWindVel ≥ 0.5 ? 0.7 : 2*mWindVel*0.7`. The model is
  created in `UnderwaterDisplayGroup` at (0, 0, −277.12) by
  `PocketGodViewController(Game)::maintainWhirlPool: @ 0x1001627c4`.
- `rand` = `arc4random()` (unsigned) in all of these formulas.

### 8.6 `PathModel`

`initWithId:pathVector: @ 0x100124fa4` over a `std::vector<VECTOR4>` (16-byte elements);
both it and `setPathVector: @ 0x100125018` call `createModel @ 0x100125374` (the geometry is
only rebuilt then, not per frame): if `< 2` points → empty; else the §8.1 strip with
half-width **1.5**. `drawOpenGL: @ 0x100125470`: push; translate(pos); disableTexture2D;
`c = alpha*0.5`, `setColor(c,c,c,c)`; draw; enableTexture2D; pop. Used by `PiranhaLogic` for
the drawn swim path (id `PiranhaPath`, z −277.12, inserted after
`UnderwaterPathDisplayMarker`).

### 8.7 `MagnifyingBeamModel`

Fields `mTopWidth`, `mBottomWidth`, `mFocusHeight` (setters `@ 0x100034eac/ebc/ecc`).
`createModel @ 0x100035078`: 6 vertices, indices `0..5`, colour array (v0, v1, v4, v5 = 0).
`calcModelVertices @ 0x100034f34` (positions are absolute, `pos` baked in):

```
v0 = (x - top/2, y, z)        v1 = (x + top/2, y, z)
v2 = v3 = (x, y - focus, z)
v4 = (x - bottom/2, y - 2*focus, z)   v5 = (x + bottom/2, y - 2*focus, z)
colour(v2) = colour(v3) = 0.7*alpha in all four channels; others 0
```

`drawOpenGL: @ 0x100035168`: `calcModelVertices`; enable colour array; disable texture; draw
the 6-index strip (two visible triangles forming an hourglass, two degenerate ones); restore.
No push/translate; blending inherited. Driver (`MagnifyingGlassLogic::onGameFrame:
@ 0x1000d2eb0`): `t = (150 ≤ y ≤ 210 && day && !storm) ? (y − 150)/60 : 0` (y = glass y);
beam `top = 74.0`, `bottom = 51.8`, `focus = t*120 + 30`, alpha `= 1 − 2|t − 0.5|`, pos = glass
pos; hot-spot sprite `MagnifyingHotSpot` at (glass x, 90) with alpha `max(0, 2(alpha − 0.5))`.

### 8.8 Finger trail sprites

`FingerTrailLogic::onGameFrame: @ 0x100117028` (per frame, not dt-scaled):
`pos.xy += (target − pos.xy)*0.5`; `setTargetXPos:yPos: @ 0x1001170f8`. Used for the
`RainbowTrail`, `ElectricGlow` and hurricane/whirlpool trail `BCMultiModel`s (origin =
−frame/2, z = −277.12); the rainbow trail's zRot follows the drag direction
(`atan2(dy, dx)` in degrees).

---

## 9. Touch input and hit testing

### 9.1 UIKit → event queue (`BCView::createTouchEvents:withEvent: @ 0x100177264`)

`touchesBegan/Moved/Ended/Cancelled:withEvent:` (`@ 0x100177cbc/ccc/cdc/cec`) all forward the
touch set. Per `UITouch` (`loc = [touch locationInView:self]`, landscape points, origin
top-left):

- If `mbTouchActive == 0` (initial; enabled by the `enableTouch` sequence item, see
  `PocketGodViewController(Game)::enableTouch::: @ 0x100159998`): only for phase Began call
  `delegate.preStartTouchBeganXPos:yPos:` (a no-op in `PocketGodViewController
  @ 0x10006ea9c`); the touch is otherwise ignored.
- Else: `bt = getTouch:(touch)` (`@ 0x1001770b0`: the `BCTouch` in `mUITouchArray` with that
  `uiTouch`, or a new one appended), retained; if phase is Ended(3) or Cancelled(4) it is
  removed from `mUITouchArray`; an event `{bt, phase(i64), (float)timestamp, tapCount,
  (float)loc.x, (float)loc.y, valid=1}` is written at the queue's write index
  (`DAT_1008162cc`; 1000 entries × 0x30 bytes at `DAT_1008162d0`; index wraps 999 → 0; no
  overflow check).

### 9.2 Dispatch (`BCView::processTouchEvents: @ 0x100176c54`, called in `gameLoop`)

```
while read != write:
    e = queue[read]; if !e.valid: return
    read = (read < 999) ? read + 1 : 0
    bt.touchPhase = e.phase; bt.timeStamp = e.ts; bt.gameTime = t; bt.tapCount = e.tap
    bt.pos = (e.x, e.y)            // view points, NOT game coordinates
    if bt not in mGameTouchArray: append
    phase 0 Began:     delegate.touchBegan:bt allTouches:mGameTouchArray
    phase 1 Moved:     delegate.touchMoved:…
    phase 2 Stationary: nothing
    phase 3 Ended:     delegate.touchEnded:…;     remove bt from mGameTouchArray
    phase 4 Cancelled: delegate.touchCancelled:…; remove bt from mGameTouchArray
    release bt; e.valid = 0
```

Multi-touch: every finger is an independent persistent `BCTouch`; `allTouches` lists the
fingers currently down. `BCTouch` (`init @ 0x100177de0`): `mUITouch`, `mTouchPhase`,
`mTimeStamp`, `mGameTime`, `mTapCount`, `mPos` (x, y, 0, 1), `mActiveObject` (retained; the
logic currently "owning" the touch). `BCView::clearTouchArrayForModalViewController
@ 0x100176c38` empties `mGameTouchArray` (used when a modal UIKit view appears, after
`PocketGodViewController::cancelAllTouches @ 0x1000707c8`).

### 9.3 View ↔ display-object transforms

Matrix helpers (all `MATRIX = float[16]`, column-major): `FUN_1001a1184` identity,
`FUN_1001a1414(x,y,z)` translation, `FUN_1001a1454(a)` = `Rx(−a)`, `FUN_1001a14b4(a)` =
`Ry(−a)`, `FUN_1001a151c(a)` = `Rz(−a)` (radians; they build the transposed/inverse
rotation), `FUN_1001a11bc(out, A, B)` = `out = B·A`, `FUN_100007b20(out, v, M)` = `M·v`
*(all asm-verified)*.

`createViewToDisplayObjectMatrix:(M) @ 0x10010f400`:
`M = (parent ? parent.viewToObj : I)`, then `M = Rz(−r.z)·Ry(−r.y)·Rx(−r.x)·T(−p)·M` — the exact
inverse of a group's GL transform **without scale**.

`createDisplayObjectToViewMatrix:(M) @ 0x10010f528`:
`M = parentMatrix·T(p)·Rz(r.z)·Ry(r.y)·Rx(r.x)` (note: Rz first in this direction, i.e. not
the exact inverse of the draw order when several axes rotate; scale ignored).

`viewToDisplayObject:(pt) @ 0x10010f664` *(asm-verified)*:

```
x' = (screenWidth / touchWidth) * pt.x - frustumCenterX
y' = (screenHeight / touchHeight) * (touchHeight - pt.y) - frustumCenterY
local = createViewToDisplayObjectMatrix · Rz(-90°) · (x', y', 0, 1)
```

For any object under the standard root group this yields game coordinates:
`game = ((W_L/touchW)·tx − W_L/2 + 240, (H_L/touchH)·(touchH − ty) − H_L/2 + 160)` further
transformed by the inverse transforms of the intermediate groups and of the object itself
(pos and rotation only).

`displayObjectToView:(pt) @ 0x10010f800`: `out = Rz(+90°)·createDisplayObjectToViewMatrix ·
(pt.x, pt.y, 0, 1)`; returns `x = (touchWidth/screenWidth)·(frustumCenterX + out.x)`,
`y = touchHeight − (touchHeight/screenHeight)·(frustumCenterY + out.y)` (view points).

### 9.4 Hit testing

`doesPointIntersect:checkSelect:debug: @ 0x10010f9f0` *(asm-verified)* (`doesPointIntersect:
@ 0x10010f9a8` = checkSelect YES; `…:debug:` and `…:checkSelect:` variants forward):

```
if checkSelect && !mbIsSelectable: return NO
l = viewToDisplayObject(pt)                 // relative to pos, rotation undone, scale NOT undone
if s.x >= 0 (or NaN): xmin = o.x + colXMin; xmax = o.x + colXMax
else:                 xmin = -o.x - colXMax; xmax = -o.x - colXMin
same for y with s.y, colY*
return xmin < l.x < xmax  &&  ymin < l.y < ymax      // strict
```

Boxes are in unscaled units (scale magnitude is ignored; only the sign mirrors the box);
the fake-perspective factor, pivot and skew are ignored. `BCLogic::doesPointIntersect:…
@ 0x10013984c…0x1001398b8` forward to the display object. There is no radius-based test in
the engine (some gameplay code tests distances itself, e.g. sun/moon drag radius 40, pinch 120).

Which logic receives a touch (`PocketGodViewController`). Precedence with the island
gestures (details in the gameplay spec): in `touchBegan` only the two-finger sun pinch is
tested before the logic hit test, and the island gestures (sun/moon drag, lightning, clouds,
ocean, crack, ape wench, hurricane, rainbow, locusts, underwater) start only if no logic was hit
and no screen/mini-game is open; in `touchMoved`/`touchEnded` a touch that already owns a
gesture is routed to it first, and the generic dispatch below runs for every other touch:

1. `createTouchLogicArray: @ 0x10006e730`: `activeDisplayGroup.addSelectableLogicsToArray:`
   — pre-order depth-first over the whole active tree (**hidden objects included**, overlays
   excluded): a display object (or group) contributes its logic if it has one and
   `mbIsSelectable`; groups then recurse into all children in order. NULL-terminated array of
   up to 999 (`mTouchLogicArray[1000]`), each retained during dispatch.
2. `touchBegan:allTouches: @ 0x10006eaa0`: walk the array **backwards** (last drawn = top
   first). For each logic whose display object is selectable: the first one with
   `doesPointIntersect:touch.pos` gets `touch.activeObject = logic` and
   `touchBeganInside:allTouches:`; every other selectable logic (before or after the hit) gets
   `touchBeganOutside:allTouches:`. Only if nothing was hit do the island gestures run.
3. `touchMoved:allTouches: @ 0x10006f7fc` (backwards walk): until the first hit, a logic that
   intersects becomes the hit: if `touch.activeObject ≠ logic` → old active (if any)
   `touchMovedExited:`, `activeObject = logic`, `touchMovedEntered:`; then
   `touchMovedInside:`. Every other logic: if it is the touch's active object → clear it and
   `touchMovedExited:`; then `touchMovedOutside:`.
4. `touchEnded:allTouches: @ 0x1000701a0`: backwards walk; first intersecting logic gets
   `touchEndedInside:`, all others `touchEndedOutside:`.
5. `touchCancelled:allTouches: @ 0x100070a00` simply calls `touchEnded:allTouches:`.
   `cancelAllTouches @ 0x1000707c8` (before presenting a modal UIKit controller) runs the
   step-4 dispatch (with `mCancelTouchLogicArray`) once for every touch in the last
   `mAllTouches`.

Logics whose display object is not selectable at dispatch time receive no `touchBegan…`
callback at all; in the moved/ended walks they fail `doesPointIntersect:` (which checks
selectability) and therefore receive the "outside" callbacks.

The default `BCLogic` touch handlers (`@ 0x1001398d4…0x1001398f0`) are empty.

---

## 10. Accelerometer (`BCView::readAccelerometer @ 0x100175f3c`, asm-verified)

Called once per frame. `a = motionManager.accelerometerData.acceleration` (doubles, device
**portrait** axes, in g, gravity included: flat face-up ≈ (0, 0, −1); held upright in
landscape-right ≈ (−1, 0, 0)). With no data (simulator / desktop) `a = (0, 0, 0)`.

```
raw.xyz = (float)(raw + (a - raw) * 0.699999988079071)          // per component, in double
totalRaw = |raw.xyz|; xyRaw = √(x²+y²); xzRaw = √(x²+z²); yzRaw = √(y²+z²)
rawRotRad = (atan2f(raw.z, raw.y), atan2f(raw.z, raw.x), atan2f(raw.y, raw.x))
rawRotDeg = rawRotRad / 3.1415927 * 180
k = (a.x - accel.x > 0) ? 0.10000000149011612 : 0.699999988079071   // decided on x only
accel.xyz = (float)(accel + (a - accel) * k)
total, xy, xz, yz, rotRad, rotDeg as above from accel
```

Initial values: all vectors `(0,0,0,1)`, scalars 0. Accessors: `totalAccelForce
@ 0x100177cfc`, `xy/xz/yzAccelForce`, `accelForce @ 0x100177d3c`, `accelRotRadians`,
`accelRotDegrees @ 0x100177d5c`, `totalRawAccelForce @ 0x100177d6c`, `xy/xz/yzRawAccelForce`,
`rawAccelForce @ 0x100177dac`, `rawAccelRotRadians`, `rawAccelRotDegrees @ 0x100177dcc`.
Consumers: tilt physics (`SpriteLogic::isAccelEnabled @ 0x100199358` disables tilt when
`rawRotDeg.x ∈ (−100, −80)` and `rawRotDeg.y ∈ (−100, −80)` (device flat) or when any raw
component is exactly 0), device-shake detection (`PygmyLogic::checkForEarthquake:
@ 0x10012b1d0`, per pygmy, only when no raw component is 0 and gravity is enabled: counts a
swing each time `totalRawAccelForce` goes above 1.1 or back below 0.9, and resets the count
when no swing happened for 0.5 s; what the count triggers is in the gameplay spec), and many
`*Logic` classes reading `accelRotDegrees`. A port without a sensor should feed `(0,0,0)`
(reproduces "no tilt, no shake" exactly) or a synthetic gravity vector.

---

## 11. Display-related sequence items (`BCLibrary`) and screen shake

Positional params `[i]`; "group" = `""` → active display group (`hide/show/setAlpha/
tweenAlpha/sinAlpha/blink`) or nil → active group (`setPos/tweenPos/setTextureDef`), else
`getDisplayGroupById:`; the object is `group.getDisplayObject:id` (direct child). **The tween
state is stored in the `BCLibrary` singleton** (`mStallTime`, `mDeltaValue`, `mValue`,
`mDeltaPos`) and is shared by all sequences — two library tweens running at once interfere
(replicate). `dt` = the sequence's `deltaTime` = the frame dt (`BCSequence::runSequence:
@ 0x1000c02f8`). Return 0 = stay on this item, 2 = advance.

| Handler | Params | Behaviour |
|---|---|---|
| `setAlpha::: @ 0x100024e74` | [0] alpha, [1] group, [2] id | `obj.setAlpha:` → 2 |
| `tweenAlpha::: @ 0x100024f68` | [0] group, [1] time, [2] target, [3] id | first time: `stall = time; delta = (target − alpha)/time`. Each call: `v = stall > 0 ? target − stall*delta : target`; `obj.alpha = v`; `stall −= dt`; return `stall_before ≤ 0 ? 2 : 0` (first frame shows the start value) |
| `sinAlpha::: @ 0x100025118` | [0] min, [1] max, [2] id, [3] group, [4] time (unused), [5] speed | `half = (max−min)/2`; first time `mValue = asinf((alpha − (min+half))/half)`; every call `alpha = min + half + half·sinf(speed·dt + mValue)`; returns 0 forever. The phase never accumulates (original bug), so alpha stays ≈ its start value |
| `setPos::: @ 0x1000252cc` | [0] x, [1] z, [2] y, [3] id, [4] group | sets each non-nil component → 2 |
| `tweenPos::: @ 0x1000253d0` | [0] targetZ, [1] targetX, [2] id, [3] group, [4] targetY, [5] time | missing targets = current. First time: `stall = time; dpos = (target − pos)/time`. Each call: `pos = target − dpos·dt` (**dt, not stall — original bug: jumps almost to the target**); if `stall ≤ 0`: `pos = target`, return 2; `stall −= dt`; else 0 |
| `blink::: @ 0x10002562c` | [0] group, [1] interval, [2] id | first time `stall = interval`; each call: if `stall ≤ 0`: `stall = interval`, `alpha = (alpha == 0) ? 1 : 0`; `stall −= dt`; returns 0 forever |

(`BCLogic::setAlpha::: @ 0x1001397f8` and `BCLogic::tweenAlpha::: @ 0x1001396dc` are the
per-logic versions acting on the logic's own display object with the same formulas and the
logic's own `mStallTime/mDeltaValue`; the sequence's delegate is asked first, see
`sequences.md`.)

**Screen shake** `PocketGodViewController(Game)::shakeBackground::: @ 0x10015e374`:

```
first time: mShakeStallTime = -1000                  // "infinite"
            if [0] != nil: mShakeStallTime = (rand % (int(max*1000) - int(min*1000) + 1)
                                              + int(min*1000)) / 1000   // [0] min, [1] max seconds
if mShakeStallTime > 0 || mShakeStallTime == -1000:
    s = sinf(mShakeRot); mShakeRot += mShakeRotVel  // mShakeRotVel = 1.3 (0x3fa66666), per frame
    group = mbTheRuns ? TheRunsDisplayGroup : IslandDisplayGroup
    group.yPos = 2*s - 2                             // oscillates in [-4, 0]
    if mShakeStallTime != -1000: mShakeStallTime -= dt
    return 0
return 2
```

`mShakeRot = 0` and `mShakeRotVel = 1.3` are set in `loadGame`. `stopTheRunsShakeScreen:::
@ 0x10015e608` resets `TheRunsDisplayGroup.yPos = 0`. The island group's y is also eased every
frame by `PocketGodViewController(Game)::onGameFrame: @ 0x1001585bc`
(`y += (mTargetIslandYPos − y)·0.3`, also copied to `DoodleDisplayGroup`), and the mini-game
"cameras" move `TheRuns/MoronPests/NoPee/KonkeyDong` display groups (same function; gameplay
spec) — scrolling is always done by moving display groups.

---

## 12. Other display entry points

`PocketGodViewController` has no `drawOpenGL*` methods of its own: all GL drawing happens in
`BCView::gameLoop` (§2.2) through the display tree. The controller influences what is drawn
only by switching display groups (sequence items, §4.5), moving groups (shake/cameras, §11),
feeding the custom models (§8) and the per-frame easing below.

### 12.1 Menu button easing

Same `onGameFrame:`: menu button `rot.z += (mTargetMenuButtonZRot − rot.z)·0.5`, menu group
`y += (mTargetMenuYPos − y)·0.5`; when y reaches exactly 90 the menu group is removed from the
tree, otherwise it is re-inserted after `MenuDisplayMarker` (initial targets 90, 90).

### 12.2 Main tree skeleton (struct XML, for orientation)

`PocketGod` (root, −160/240/−90) children in draw order: `IslandRootDisplayMarker`,
`UnderwaterDisplayMarker`, `MenuDisplayMarker`, `HealthBar`, `LaserButton`, `NewPygmyButton`,
`MenuButton`, `AchievementMessage`, `CountingMessage`, `ChallengeTimer`, `ChallengeAlert`,
`ChallengeInProgress`, `MapDisplayMarker`. `IslandDisplayGroup` starts with `Gradient`,
`Stars`, `StarDisplayMarker`, `Sun`, `Moon`, `FireworksDisplayMarker`, `BehindOceanTiles`,
`FrontOceanTiles`, … (full contents: `struct-xml.md`).

### 12.3 Loading screen

`PocketGodAppDelegate::showLoadingImage @ 0x100089570` adds a `UIImageView` on top of the GL
view: by `applicationFrame` size — 480 tall → `Default` (portrait image translated to the
centre), 480 wide → `Default` rotated −90°, 568 tall → `Default-568h`, 568 wide →
`Default-568h` rotated −90°, 1024 tall → `Default-Landscape~ipad` rotated +90°, 768 tall →
`Default-Landscape~ipad`, otherwise `Default`. Removed by `onHideLoadingScreen
@ 0x100089c2c` (triggered by the `hideLoadingScreen` sequence item). A port simply shows the
landscape launch image until the game signals it.

---

## 13. Port guidance (Bevy 0.20)

- **Camera/projection**: an orthographic view of game space with the visible rectangle
  `[240 − W_L/2, 240 + W_L/2] × [160 − H_L/2, 160 + H_L/2]`, where `W_L, H_L` follow §1.4 from
  the window aspect. Apply the root groups' `T(−160,240)·Rz(−90°)` and the `Rz(+90°)` of the
  projection literally, or equivalently drop both and treat root children as game space
  (exact for every root group with that transform; the overlay groups' 284 becomes an x
  offset of 44). Clip vertices whose eye z leaves `[−5000, 0]`.
- **Renderer**: traverse the tree on the CPU every frame in draw order, compute final vertex
  positions with the exact matrix sequences of §5/§8 (4×4, 3D rotations included), and emit
  draws in order into a custom 2D pipeline with premultiplied blending
  `(One, OneMinusSrcAlpha)`, no depth, per-draw blend on/off, current colour, scissor for the
  active group when wide/tall. Track `blend_enabled` and `current_color` across draws and
  frames (§3.1). Sample atlases with linear filtering and repeat wrap.
- **Timing**: run the whole update at a fixed 60 Hz with `dt = 1/60` (the original's
  display-link rate; much gameplay is per-frame), with the 3 warm-up frames of §2.1 and the
  1/15 clamp only for resumes. Process input events at the start of each tick in arrival
  order, as `BCTouch` objects with view-space positions.
- **Touch**: map window positions to the original "view points" (`touchWidth × touchHeight`
  = window logical size) so that §9.3 applies unchanged; mouse = one touch.

---

## 14. Open questions

1. `contentScaleFactor` is never set, so the renderbuffer is assumed to be 1× (points), as
   documented for CAEAGLLayer-backed views. If iOS had defaulted it to the screen scale (BCView
   does implement an empty `drawRect:`), locust point sprites would be half as big on retina
   devices and the scissor values would double; nothing else changes.
2. Locust colour and the blend state of the "inherited" classes depend on whatever was drawn
   just before; the port should emulate the GL state machine (§3.1) rather than guess. In
   typical scenes the inherited colour is (1,1,1,1)-premultiplied and blending is on.
3. Listener ordering within one `gameFrameOrder` value assumes
   `-[NSMutableArray sortUsingFunction:context:]` is stable (CoreFoundation's
   `CFArraySortValues` merge sort); if it were not, equal-order listeners could run in a
   different order than insertion order.
4. `TallScreenOverlay` placement was verified only arithmetically (no iPad screenshot); its
   bands cover game y −28…2 and 317…347 against a visible range of −20…340.

---

## Appendix A. Remaining small methods (for completeness)

| Method | Behaviour |
|---|---|
| `BCView::initWithCoder: @ 0x100175dfc` | nib path: super init + `initGLES` only (unused; the view is created in code) |
| `BCView::drawRect: @ 0x100175f00` | empty |
| `BCView::setDelegate: @ 0x1001769a8` | retains the delegate (`PocketGodViewController`) |
| `BCView::destroyFramebuffer @ 0x100176a04` | deletes framebuffer/renderbuffers |
| `BCView::releaseContextBuffers @ 0x100176a70` | frees the unused `mpScreenBuffer`/`mScreenBufferContext` (never allocated) |
| `BCView::dealloc @ 0x100176abc` | clears the singleton, stops the loop, releases everything |
| `BCView::setTouchActive: @ 0x100177254` | `mbTouchActive = flag` (§9.1) |
| `BCView::traceGameFrameListeners @ 0x100175c58`, `traceTouchArray @ 0x100176f0c` | debug logging only |
| `BCView::.cxx_construct @ 0x100177ddc` | returns self |
| `BCDisplayObject::traceGlobalStatus @ 0x10010dd80`, `traceStatus @ 0x10010e1e8`, `traceStatus: @ 0x10010e200`, `traceMatrix:withMessage: @ 0x10010ddcc`, `traceVector:withMessage: @ 0x10010df20` (and the `traceStatus:` overrides of groups, markers, multi/tile models) | debug logging only |
| `BCDisplayObject::id/setId: @ 0x10010e2e4/0x10010e2f4`, `data/setData: @ 0x10010e350/0x10010e360` | retained string properties |
| `BCDisplayObject::setDebug: @ 0x10010e3bc`, `BCMultiModel::setDebug: @ 0x1001818c0` | sets `mbDebug` (unused) |
| `BCDisplayObject::isInMultiModelMode @ 0x10010e3cc` | getter |
| `BCDisplayObject::hide @ 0x10010e71c`, `show @ 0x10010e730`, `isHidden @ 0x10010e740`, `setIsHidden: @ 0x10010e750`, `isSelectable @ 0x10010e760`, `setIsSelectable: @ 0x10010e770` | flag accessors |
| `BCDisplayObject::dealloc @ 0x10010f380` | decrements the live-object counter `DAT_1007d1f10`, releases `mId` (not `mData`) |
| `BCModel::modelViewMatrix @ 0x100118778` | returns the unused matrix field |
| `BCTouch` accessors `@ 0x100177edc … 0x100177ff8` | plain getters/setters (`setXPos:yPos:` writes x, y only) |
