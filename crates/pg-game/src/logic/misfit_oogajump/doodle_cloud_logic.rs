//! `DoodleCloudLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::managers::bonus_pack_manager;
use pg_engine::animation::Timelines;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::display::multi_model;
use pg_engine::logic;

/// `DoodleCloudLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct DoodleCloudLogic {
    /// `mDoodleCloudLogicListener`: `@"<DoodleCloudLogicListener>"`
    pub doodle_cloud_logic_listener: Option<Entity>,

    /// `mDoodleCloudType`: `i`
    pub doodle_cloud_type: i32,

    /// `mbWispDissolved`: `B`
    pub wisp_dissolved: bool,

    /// `mbJumpOn`: `B`
    pub jump_on: bool,
}

/// The file-scope generator state the clouds share: each `moveToTop` places its cloud above the
/// one placed before it.
#[derive(Resource, Clone, Debug, Default)]
pub struct DoodleCloudLogicStatics {
    /// `DAT_1007e7be0`: the pygmy caught a jet pack since the last one was placed.
    pub jet_pack_caught: bool,

    /// `DAT_1007e7be4`: the height of the last jet pack.
    pub last_jet_pack_height: f32,

    /// `DAT_1007e7be8`: the height of the last shooting cloud.
    pub last_shooter_height: f32,

    /// `DAT_1007e7bec`: the running height the next cloud is placed above.
    pub next_height: f32,

    /// `DAT_1007e7bf0`: the type of the cloud placed last.
    pub previous_type: i32,

    /// `DAT_1007e7bf4`: the smallest gap between two clouds.
    pub min_gap: f32,

    /// `DAT_1007e7bf8`: the largest gap between two clouds.
    pub max_gap: f32,

    /// `DAT_1007e7c00`: the cloud placed last (not retained).
    pub previous_cloud: Option<Entity>,
}

pub const DOODLE_CLOUD_LOGIC: Class = Class {
    name: "DoodleCloudLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `mDoodleCloudType` values.
const DOODLE_CLOUD_NORMAL: i32 = 0;
const DOODLE_CLOUD_ELECTRIC: i32 = 1;
const DOODLE_CLOUD_SNOW: i32 = 2;
const DOODLE_CLOUD_WISPY: i32 = 3;
const DOODLE_CLOUD_SHOOT: i32 = 4;
const DOODLE_CLOUD_JET_PACK: i32 = 5;

/// The generator state of the first cloud of an Ooga Jump game.
const FIRST_NEXT_HEIGHT: f32 = 600.0;
const FIRST_PREVIOUS_TYPE: i32 = DOODLE_CLOUD_SHOOT;
const FIRST_GAP: f32 = 22.0;

/// Clouds are drawn at half size; jet packs at full size.
const CLOUD_SCALE: f32 = 0.5;
const JET_PACK_SCALE: f32 = 1.0;

/// The gap between two clouds never exceeds this.
const MAX_CLOUD_GAP: f32 = 110.0;

/// The points above the pygmy's position that a shot hits (31) and that grabs a jet pack (20).
const SHOT_TARGET_Y_OFFSET: f32 = 31.0;
const JET_PACK_GRAB_Y_OFFSET: f32 = 20.0;

/// Squared reach of a shooting cloud's shot (50) and of a jet pack (25).
const SHOT_HIT_DISTANCE_SQUARED: f32 = 2500.0;
const JET_PACK_CATCH_DISTANCE_SQUARED: f32 = 625.0;

/// A cloud's top, and the horizontal band of it the pygmy can land on.
const CLOUD_TOP_Y_OFFSET: f32 = 10.5;
const CLOUD_MIN_X_OFFSET: f32 = 5.0;
const CLOUD_MAX_X_OFFSET: f32 = 70.0;

/// `[[logic displayObject] pos]`.
fn logic_pos(world: &World, logic: Entity) -> Vec4 {
    let display_object = logic::display_object(world, logic);
    display::object(world, display_object).pos
}

/// The generator state (all zero before the first cloud).
fn statics(world: &World) -> DoodleCloudLogicStatics {
    match world.get_resource::<DoodleCloudLogicStatics>() {
        Some(statics) => statics.clone(),
        None => DoodleCloudLogicStatics::default(),
    }
}

/// The generator state, for writing.
fn statics_mut(world: &mut World) -> Mut<'_, DoodleCloudLogicStatics> {
    world.get_resource_or_insert_with(DoodleCloudLogicStatics::default)
}

/// `[[BonusPackManager instance] isBonusPackUnlocked:@"ComicPreOrder"]` (NO without a manager).
fn is_comic_pre_order_unlocked(world: &mut World) -> bool {
    let Some(bonus_pack_manager) = bonus_pack_manager::instance(world) else {
        return false;
    };

    bonus_pack_manager::is_bonus_pack_unlocked(world, bonus_pack_manager, "ComicPreOrder")
}

/// `[[DoodleCloudLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&DOODLE_CLOUD_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[DoodleCloudLogic initWithDisplayObject:]` @ 0x1000cc978
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(DoodleCloudLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    // The first cloud since the last ones were released starts a new generator.
    let previous_cloud = statics(world).previous_cloud;
    if previous_cloud.is_none() {
        let mut statics = statics_mut(world);
        statics.jet_pack_caught = false;
        statics.next_height = FIRST_NEXT_HEIGHT;
        statics.last_shooter_height = 0.0;
        statics.last_jet_pack_height = 0.0;
        statics.previous_type = FIRST_PREVIOUS_TYPE;
        statics.min_gap = FIRST_GAP;
        statics.max_gap = FIRST_GAP;
    }

    {
        let mut display = display::object_mut(world, display_object);
        display.scale.x = CLOUD_SCALE;
        display.scale.y = CLOUD_SCALE;
    }

    move_to_top(world, this);
}

/// `-[DoodleCloudLogic setDoodleCloudLogicListener:]` @ 0x1000cca54
pub fn set_doodle_cloud_logic_listener(world: &mut World, this: Entity, doodle_cloud_logic_listener: Entity) {
    world.obj_mut::<DoodleCloudLogic>(this).doodle_cloud_logic_listener = Some(doodle_cloud_logic_listener);
}

/// `-[DoodleCloudLogic dealloc]` @ 0x1000cd3d0, when the controller's cloud array lets go of the
/// cloud (`removeDoodleClouds`): the generator forgets it, so the next cloud starts afresh.
pub fn dealloc(world: &mut World, this: Entity) {
    let previous_cloud = statics(world).previous_cloud;
    if previous_cloud == Some(this) {
        statics_mut(world).previous_cloud = None;
    }

    logic::dealloc(world, this);
}

/// `-[DoodleCloudLogic moveToTop]` @ 0x1000ccab0
pub fn move_to_top(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let previous_cloud = statics(world).previous_cloud;

    let x_roll = world.resource_mut::<Random>().arc4random_mod(40501);
    let x_pos = x_roll as f32 / 100.0;
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = x_pos;
        display.pos.z = SPRITE_Z;
        display.scale.x = CLOUD_SCALE;
        display.scale.y = CLOUD_SCALE;
    }

    {
        let mut cloud = world.obj_mut::<DoodleCloudLogic>(this);
        cloud.jump_on = false;
        cloud.doodle_cloud_type = DOODLE_CLOUD_NORMAL;
    }

    // A plain cloud may be followed by a special one.
    let previous_type = statics(world).previous_type;
    if let Some(previous_cloud) = previous_cloud {
        if previous_type == DOODLE_CLOUD_NORMAL {
            roll_doodle_cloud_type(world, this, display_object, previous_cloud);
        }
    }

    if let Some(previous_cloud) = previous_cloud {
        place_above(world, this, display_object, previous_cloud);
    }

    let doodle_cloud_type = world.obj::<DoodleCloudLogic>(this).doodle_cloud_type;
    let behavior = match doodle_cloud_type {
        DOODLE_CLOUD_NORMAL => Some("DoodleCloudInit"),
        DOODLE_CLOUD_ELECTRIC => Some("DoodleCloudElectricInit"),
        DOODLE_CLOUD_SNOW => Some("DoodleCloudSnowCloudInit"),
        DOODLE_CLOUD_WISPY => Some("DoodleWispyCloudInit"),
        DOODLE_CLOUD_SHOOT => Some("DoodleCloudShootInit"),
        DOODLE_CLOUD_JET_PACK => Some("DoodleJetPack"),
        _ => None,
    };
    if let Some(behavior) = behavior {
        logic::set_behavior(world, this, Some(behavior));
    }

    // The gaps widen as the pygmy climbs.
    {
        let mut statics = statics_mut(world);
        statics.max_gap = statics.max_gap + 0.1;
        if statics.max_gap - statics.min_gap > 20.0 {
            statics.min_gap = statics.min_gap + 0.1;
        }
    }

    let doodle_cloud_type = world.obj::<DoodleCloudLogic>(this).doodle_cloud_type;
    {
        let mut statics = statics_mut(world);
        statics.previous_cloud = Some(this);
        statics.previous_type = doodle_cloud_type;
    }
    world.obj_mut::<DoodleCloudLogic>(this).wisp_dissolved = false;
}

/// The type roll of `moveToTop`: a 1/11 chance of a wispy cloud; else, while the gaps are under
/// 80, 1/11 of a snow cloud; else 1/11 of an electric cloud; else a shooting cloud when the gaps
/// exceed 36.8, the last cloud is right of 180, a 1/4 roll hits and the last shooter is more
/// than 960 below; else, with `ComicPreOrder`, a 1/5 jet pack once the gaps exceed 28.75 and
/// the climb since the last jet pack is long enough.
fn roll_doodle_cloud_type(world: &mut World, this: Entity, display_object: Entity, previous_cloud: Entity) {
    let wispy_roll = world.resource_mut::<Random>().arc4random_mod(11);
    if wispy_roll == 0 {
        world.obj_mut::<DoodleCloudLogic>(this).doodle_cloud_type = DOODLE_CLOUD_WISPY;
        return;
    }

    let max_gap = statics(world).max_gap;
    if max_gap < 80.0 {
        let snow_roll = world.resource_mut::<Random>().arc4random_mod(11);
        if snow_roll == 0 {
            world.obj_mut::<DoodleCloudLogic>(this).doodle_cloud_type = DOODLE_CLOUD_SNOW;
            return;
        }
    }

    let electric_roll = world.resource_mut::<Random>().arc4random_mod(11);
    if electric_roll == 0 {
        world.obj_mut::<DoodleCloudLogic>(this).doodle_cloud_type = DOODLE_CLOUD_ELECTRIC;
        return;
    }

    if is_shooter_due(world, previous_cloud) {
        world.obj_mut::<DoodleCloudLogic>(this).doodle_cloud_type = DOODLE_CLOUD_SHOOT;
        return;
    }

    if !is_comic_pre_order_unlocked(world) {
        return;
    }

    let jet_pack_roll = world.resource_mut::<Random>().arc4random_mod(5);
    if jet_pack_roll != 0 {
        return;
    }

    let statics = statics(world);
    let gaps_wide_enough = statics.max_gap > 28.75;
    if !gaps_wide_enough {
        return;
    }

    // The code compares two flags; this reduces to "more than 1280 above the last jet pack, or
    // more than 3200 after one was caught" (`cset le` is also set for NaN).
    let height_since_jet_pack = statics.next_height - statics.last_jet_pack_height;
    let jet_pack_not_caught = !statics.jet_pack_caught;
    let beyond_far_spacing = height_since_jet_pack > 3200.0;
    let near_spacing_passed = height_since_jet_pack > 1280.0 && jet_pack_not_caught;
    let far_spacing_pending = !beyond_far_spacing || jet_pack_not_caught;
    if near_spacing_passed != far_spacing_pending {
        return;
    }

    statics_mut(world).jet_pack_caught = false;
    world.obj_mut::<DoodleCloudLogic>(this).doodle_cloud_type = DOODLE_CLOUD_JET_PACK;

    let mut display = display::object_mut(world, display_object);
    display.scale.x = JET_PACK_SCALE;
    display.scale.y = JET_PACK_SCALE;
}

/// The shooting-cloud conditions of `moveToTop`, in the original's short-circuit order (the
/// 1/4 roll is only drawn when the first two hold).
fn is_shooter_due(world: &mut World, previous_cloud: Entity) -> bool {
    let max_gap = statics(world).max_gap;
    let gaps_wide_enough = max_gap > 36.8;
    if !gaps_wide_enough {
        return false;
    }

    let previous_x_pos = logic_pos(world, previous_cloud).x;
    let previous_on_right = previous_x_pos > 180.0;
    if !previous_on_right {
        return false;
    }

    let shooter_roll = world.resource_mut::<Random>().arc4random();
    if shooter_roll & 3 != 0 {
        return false;
    }

    let statics = statics(world);
    statics.next_height - statics.last_shooter_height > 960.0
}

/// The placement of `moveToTop` once a previous cloud exists: the cloud goes a random gap above
/// the running height (wispy clouds without raising it, jet packs raising it twice), shooting
/// clouds at the left edge.
fn place_above(world: &mut World, this: Entity, display_object: Entity, previous_cloud: Entity) {
    let doodle_cloud_type = world.obj::<DoodleCloudLogic>(this).doodle_cloud_type;
    match doodle_cloud_type {
        DOODLE_CLOUD_NORMAL | DOODLE_CLOUD_ELECTRIC | DOODLE_CLOUD_SNOW => {
            place_at_next_gap(world, display_object);
        }
        DOODLE_CLOUD_WISPY => {
            let min_gap = statics(world).min_gap;
            if min_gap > 44.0 {
                place_wispy(world, display_object);
            } else {
                place_at_next_gap(world, display_object);
            }
        }
        DOODLE_CLOUD_SHOOT => {
            let next_height = statics(world).next_height;
            statics_mut(world).last_shooter_height = next_height;

            let mut display = display::object_mut(world, display_object);
            display.pos.x = 0.0;
            display.pos.y = next_height;
        }
        DOODLE_CLOUD_JET_PACK => {
            place_jet_pack(world, display_object, previous_cloud);
        }
        _ => {}
    }
}

/// A random gap in `[minGap, maxGap]` (0.01 steps), at most 110.
fn next_gap(world: &mut World) -> f32 {
    let statics = statics(world);
    let random = world.resource_mut::<Random>().arc4random();
    let gap = logic::random_scaled_time(statics.min_gap, statics.max_gap, 100.0, random);

    // `fcsel …, gt`: only a gap above the cap is replaced.
    if gap > MAX_CLOUD_GAP { MAX_CLOUD_GAP } else { gap }
}

/// Types 0–2 (and a wispy cloud while the gaps are small): one gap above the running height,
/// which moves up to the cloud.
fn place_at_next_gap(world: &mut World, display_object: Entity) {
    let gap = next_gap(world);
    let next_height = statics(world).next_height;
    display::object_mut(world, display_object).pos.y = next_height + gap;
    statics_mut(world).next_height = next_height + gap;
}

/// A wispy cloud: a random `[22, minGap − 22]` (0.01 steps) above the running height, which
/// stays where it is.
fn place_wispy(world: &mut World, display_object: Entity) {
    let random = world.resource_mut::<Random>().arc4random();
    let min_gap = statics(world).min_gap;
    let offset = logic::random_scaled_time(22.0, min_gap - 22.0, 100.0, random);

    let next_height = statics(world).next_height;
    display::object_mut(world, display_object).pos.y = offset + next_height;
}

/// A jet pack: one gap above the running height, which rises by two gaps; its x is random
/// (0.01 steps) in `[prevX + 125, 455]` when the previous cloud is left of 240, else in
/// `[25, prevX − 50]`.
fn place_jet_pack(world: &mut World, display_object: Entity, previous_cloud: Entity) {
    let gap = next_gap(world);
    let next_height = statics(world).next_height;
    let y_pos = next_height + gap;
    display::object_mut(world, display_object).pos.y = y_pos;

    {
        let mut statics = statics_mut(world);
        statics.next_height = next_height + (gap + gap);
        statics.last_jet_pack_height = y_pos;
    }

    // The previous cloud's x is read through its live position (it may be this cloud).
    let previous_x_pos = logic_pos(world, previous_cloud).x;
    let x_pos = if previous_x_pos > 240.0 {
        let random = world.resource_mut::<Random>().arc4random();
        let previous_x_pos = logic_pos(world, previous_cloud).x;
        logic::random_scaled_time(25.0, previous_x_pos - 50.0, 100.0, random)
    } else {
        let min_x_pos = previous_x_pos + 75.0 + 50.0;
        let random = world.resource_mut::<Random>().arc4random();
        logic::random_scaled_time(min_x_pos, 455.0, 100.0, random)
    };
    display::object_mut(world, display_object).pos.x = x_pos;
}

/// `-[DoodleCloudLogic checkCollision:lastPos:]` @ 0x1000cd130
pub fn check_collision(world: &mut World, this: Entity, pygmy_logic: Entity, last_pos: Vec4) -> bool {
    let cloud = world.obj::<DoodleCloudLogic>(this);
    let wisp_dissolved = cloud.wisp_dissolved;
    let doodle_cloud_type = cloud.doodle_cloud_type;
    if wisp_dissolved {
        return false;
    }

    let pygmy_pos = logic_pos(world, pygmy_logic);
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    if doodle_cloud_type == DOODLE_CLOUD_SHOOT {
        return check_shot_collision(world, this, pygmy_logic, display_object, pygmy_pos, pos);
    }

    if doodle_cloud_type == DOODLE_CLOUD_JET_PACK {
        let delta_x = (pygmy_pos.x + 0.0) - pos.x;
        let delta_y = (pygmy_pos.y + JET_PACK_GRAB_Y_OFFSET) - pos.y;
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        let caught = distance_squared < JET_PACK_CATCH_DISTANCE_SQUARED;
        if !caught {
            return false;
        }

        statics_mut(world).jet_pack_caught = true;
        logic::set_behavior(world, this, Some("DoodleJetPackCatch"));
        crate::pygmy::doodle_jet_pack(world, pygmy_logic, this);
        return true;
    }

    // A pygmy over the cloud that passed down through its top since the last frame lands.
    let over_cloud = pygmy_pos.x > pos.x + CLOUD_MIN_X_OFFSET && pygmy_pos.x < pos.x + CLOUD_MAX_X_OFFSET;
    let cloud_top = pos.y + CLOUD_TOP_Y_OFFSET;
    let crossed_top = pygmy_pos.y < cloud_top && last_pos.y > cloud_top;
    if !over_cloud || !crossed_top {
        return false;
    }

    world.obj_mut::<DoodleCloudLogic>(this).jump_on = true;
    match doodle_cloud_type {
        DOODLE_CLOUD_NORMAL => crate::pygmy::doodle_bounce(world, pygmy_logic, this),
        DOODLE_CLOUD_ELECTRIC => crate::pygmy::doodle_electric_bounce(world, pygmy_logic, this),
        DOODLE_CLOUD_SNOW => crate::pygmy::doodle_snow_bounce(world, pygmy_logic, this),
        DOODLE_CLOUD_WISPY => {
            world.obj_mut::<DoodleCloudLogic>(this).wisp_dissolved = true;
            logic::set_behavior(world, this, Some("DoodleWispyCloudDissolve"));
        }
        _ => {}
    }
    true
}

/// The shooting cloud of `checkCollision:lastPos:`: the pygmy is knocked down within 50 of the
/// shot, the top state of the cloud's current frame (`(x + 10) / 2`, `(y + 10) / 2` from the
/// cloud), measured from a point 31 above the pygmy.
fn check_shot_collision(world: &mut World, this: Entity, pygmy_logic: Entity, display_object: Entity, pygmy_pos: Vec4, pos: Vec4) -> bool {
    let Some(frame) = multi_model::binary_frame_def(world, display_object) else {
        return false;
    };

    let timelines = world.resource::<Timelines>();
    let frame_states = timelines.frame_states(frame);

    // PORT: the original reads `states[stateCount - 1]` unchecked, so a frame without states
    // reads before the list (every frame of the shooting animations has states); no hit here.
    let Some(&top_state) = frame_states.last() else {
        return false;
    };

    let state = timelines.state(top_state);
    let shot_x_pos = pos.x + (state.x + 10.0) * 0.5;
    let shot_y_pos = pos.y + (state.y + 10.0) * 0.5;

    let delta_x = (pygmy_pos.x + 0.0) - shot_x_pos;
    let delta_y = (pygmy_pos.y + SHOT_TARGET_Y_OFFSET) - shot_y_pos;
    let distance_squared = delta_x * delta_x + delta_y * delta_y;
    let hit = distance_squared < SHOT_HIT_DISTANCE_SQUARED;
    if !hit {
        return false;
    }

    crate::pygmy::doodle_knocked_down(world, pygmy_logic, this);
    true
}

/// `-[DoodleCloudLogic onGameFrame:]` @ 0x1000cd448
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}
