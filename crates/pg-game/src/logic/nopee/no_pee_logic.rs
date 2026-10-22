//! `NoPeeLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::nopee::no_pee_map_logic;
use crate::logic::nopee::no_pee_sea_blob_logic;
use crate::logic_table;
use pg_engine::display;
use pg_engine::logic;

/// `NoPeeLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeLogic {
    /// `mNoPeeLogicListener`: `@"<NoPeeLogicListener>"`
    pub no_pee_logic_listener: Option<Entity>,

    /// `mNoPeeDisplayGroup`: `@"BCDisplayGroup"`
    pub no_pee_display_group: Option<Entity>,

    /// `mNoPeeCollidableLogicArray`: `@"NSMutableArray"`
    pub no_pee_collidable_logic_array: Vec<Entity>,

    /// `mNoPeeTreasureLogicArray`: `@"NSMutableArray"`
    pub no_pee_treasure_logic_array: Vec<Entity>,

    /// `mNoPeeSeaBlobLogicArray`: `@"NSMutableArray"`
    pub no_pee_sea_blob_logic_array: Vec<Entity>,

    /// `mNoPeeFishLogicArray`: `@"NSMutableArray"`
    pub no_pee_fish_logic_array: Vec<Entity>,

    /// `mNoPeeMineLogicArray`: `@"NSMutableArray"`
    pub no_pee_mine_logic_array: Vec<Entity>,

    /// `mNoPeeClamLogicArray`: `@"NSMutableArray"`
    pub no_pee_clam_logic_array: Vec<Entity>,

    /// `mNoPeeParticleLogicArray`: `@"NSMutableArray"`
    pub no_pee_particle_logic_array: Vec<Entity>,

    /// `mNoPeePygmyLogic`: `@"PygmyLogic"`
    pub no_pee_pygmy_logic: Option<Entity>,

    /// `mNoPeeOverlayLogic`: `@"MultiModelLogic"`
    pub no_pee_overlay_logic: Option<Entity>,

    /// `mNoPeeRetryOverlayLogic`: `@"MultiModelLogic"`
    pub no_pee_retry_overlay_logic: Option<Entity>,

    /// `mNoPeeIntroLogic`: `@"MultiModelLogic"`
    pub no_pee_intro_logic: Option<Entity>,

    /// `mNoPeeUnderwaterForegroundLogic`: `@"MultiModelLogic"`
    pub no_pee_underwater_foreground_logic: Option<Entity>,

    /// `mNoPeeUnderwaterBackground1Logic`: `@"MultiModelLogic"`
    pub no_pee_underwater_background1_logic: Option<Entity>,

    /// `mNoPeeUnderwaterBackground2Logic`: `@"MultiModelLogic"`
    pub no_pee_underwater_background2_logic: Option<Entity>,

    /// `mNoPeeUnderwaterBackground3Logic`: `@"MultiModelLogic"`
    pub no_pee_underwater_background3_logic: Option<Entity>,

    /// `mNoPeeMapLogic`: `@"NoPeeMapLogic"`
    pub no_pee_map_logic: Option<Entity>,

    /// `mbGameActive`: `B`
    pub game_active: bool,

    /// `mNoPeeScore`: `i`
    pub no_pee_score: i32,
}

pub const NO_PEE_LOGIC: Class = Class {
    name: "NoPeeLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    game_frame_order: 59,
    on_multi_model_event,
    on_game_frame,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The logic-array slots of the world objects, by kind.
const TREASURE_SLOTS: [usize; 1] = [logic_table::NO_PEE_TREASURE1];

const SEA_BLOB_SLOTS: [usize; 4] = [
    logic_table::NO_PEE_SEA_BLOB1,
    logic_table::NO_PEE_SEA_BLOB2,
    logic_table::NO_PEE_SEA_BLOB3,
    logic_table::NO_PEE_SEA_BLOB4,
];

const FISH_SLOTS: [usize; 4] = [
    logic_table::NO_PEE_FISH1,
    logic_table::NO_PEE_FISH2,
    logic_table::NO_PEE_FISH3,
    logic_table::NO_PEE_FISH4,
];

const MINE_SLOTS: [usize; 4] = [
    logic_table::NO_PEE_MINE1,
    logic_table::NO_PEE_MINE2,
    logic_table::NO_PEE_MINE3,
    logic_table::NO_PEE_MINE4,
];

const CLAM_SLOTS: [usize; 4] = [
    logic_table::NO_PEE_CLAM1,
    logic_table::NO_PEE_CLAM2,
    logic_table::NO_PEE_CLAM3,
    logic_table::NO_PEE_CLAM4,
];

const PARTICLE_SLOTS: [usize; 10] = [
    logic_table::NO_PEE_PARTICLE1,
    logic_table::NO_PEE_PARTICLE2,
    logic_table::NO_PEE_PARTICLE3,
    logic_table::NO_PEE_PARTICLE4,
    logic_table::NO_PEE_PARTICLE5,
    logic_table::NO_PEE_PARTICLE6,
    logic_table::NO_PEE_PARTICLE7,
    logic_table::NO_PEE_PARTICLE8,
    logic_table::NO_PEE_PARTICLE9,
    logic_table::NO_PEE_PARTICLE10,
];

/// The minimap item types of `addTrackedSpriteLogic:type:`.
const MAP_ITEM_PYGMY: i32 = 0;
const MAP_ITEM_TREASURE: i32 = 1;
const MAP_ITEM_OBSTACLE: i32 = 2;
const MAP_ITEM_BLOB: i32 = 3;

/// `ChallengeManager`'s game number of No Pee ("Call of Booty").
const CHALLENGE_GAME_NO_PEE: i32 = 15;

/// Where `initNoPee` puts the world group: the camera starts at the surface.
const START_GROUP_Y_POS: f32 = -680.0;

/// The parallax factors of the foreground and the three backgrounds (of the group position).
const FOREGROUND_PARALLAX: f32 = 0.3;
const BACKGROUND1_PARALLAX: f32 = -0.2;
const BACKGROUND2_PARALLAX: f32 = -0.65;
const BACKGROUND3_PARALLAX: f32 = -0.75;

/// `[[NoPeeLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeLogic initWithDisplayObject:]` @ 0x10019c880
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.get_resource_or_insert_with(Singletons::default).no_pee_logic = Some(this);

    let no_pee_display_group = pg_engine::library::display_group(world, "NoPeeDisplayGroup");
    world.obj_mut::<NoPeeLogic>(this).no_pee_display_group = no_pee_display_group;

    let map = pg_engine::library::logic(world, logic_table::NO_PEE_MAP_DISPLAY_GROUP);
    world.obj_mut::<NoPeeLogic>(this).no_pee_map_logic = Some(map);

    let intro = pg_engine::library::logic(world, logic_table::NO_PEE_INTRO);
    world.obj_mut::<NoPeeLogic>(this).no_pee_intro_logic = Some(intro);
    logic::multi_model::set_multi_model_logic_listener(world, intro, Some(this));

    let overlay = pg_engine::library::logic(world, logic_table::OVERLAY);
    let retry_overlay = pg_engine::library::logic(world, logic_table::RETRY_OVERLAY);
    let foreground = pg_engine::library::logic(world, logic_table::UNDERWATER_FOREGROUND);
    let background1 = pg_engine::library::logic(world, logic_table::UNDERWATER_BACKGROUND1);
    let background2 = pg_engine::library::logic(world, logic_table::UNDERWATER_BACKGROUND2);
    let background3 = pg_engine::library::logic(world, logic_table::UNDERWATER_BACKGROUND3);

    let treasures = library_logics(world, &TREASURE_SLOTS);
    let sea_blobs = library_logics(world, &SEA_BLOB_SLOTS);
    let fish = library_logics(world, &FISH_SLOTS);
    let mines = library_logics(world, &MINE_SLOTS);
    let clams = library_logics(world, &CLAM_SLOTS);
    let particles = library_logics(world, &PARTICLE_SLOTS);

    // Fish and particles are scenery: only these can block a spawn.
    let mut collidables = Vec::with_capacity(5);
    collidables.extend_from_slice(&treasures);
    collidables.extend_from_slice(&sea_blobs);
    collidables.extend_from_slice(&mines);
    collidables.extend_from_slice(&clams);

    let mut no_pee = world.obj_mut::<NoPeeLogic>(this);
    no_pee.no_pee_overlay_logic = Some(overlay);
    no_pee.no_pee_retry_overlay_logic = Some(retry_overlay);
    no_pee.no_pee_underwater_foreground_logic = Some(foreground);
    no_pee.no_pee_underwater_background1_logic = Some(background1);
    no_pee.no_pee_underwater_background2_logic = Some(background2);
    no_pee.no_pee_underwater_background3_logic = Some(background3);
    no_pee.no_pee_treasure_logic_array = treasures;
    no_pee.no_pee_sea_blob_logic_array = sea_blobs;
    no_pee.no_pee_fish_logic_array = fish;
    no_pee.no_pee_mine_logic_array = mines;
    no_pee.no_pee_clam_logic_array = clams;
    no_pee.no_pee_particle_logic_array = particles;
    no_pee.no_pee_collidable_logic_array = collidables;
}

/// `[BCLibrary getLogic:]` for each slot, in order.
fn library_logics(world: &World, slots: &[usize]) -> Vec<Entity> {
    let mut logics = Vec::with_capacity(slots.len());
    for &slot in slots {
        let slot_logic = pg_engine::library::logic(world, slot);
        logics.push(slot_logic);
    }
    logics
}

/// `-[NoPeeLogic setNoPeeLogicListener:]` @ 0x10019d0bc
pub fn set_no_pee_logic_listener(world: &mut World, this: Entity, no_pee_logic_listener: Entity) {
    world.obj_mut::<NoPeeLogic>(this).no_pee_logic_listener = Some(no_pee_logic_listener);
}

/// `-[NoPeeLogic shutDownNoPeeLogicArray:]` @ 0x10019d28c
pub fn shut_down_no_pee_logic_array(world: &mut World, _this: Entity, shut_down_no_pee_logic_array: &[Entity]) {
    for &object in shut_down_no_pee_logic_array {
        logic::shutdown(world, object);
        let object_display_object = logic::display_object(world, object);
        display::remove_from_display_group(world, object_display_object);
    }
}

/// `-[NoPeeLogic noPeeLogic]` @ 0x10019d3c8
pub fn no_pee_logic(_world: &mut World, this: Entity) -> Option<Entity> {
    Some(this)
}

/// `-[NoPeeLogic noPeeCollidableLogicArray]` @ 0x10019d3cc
pub fn no_pee_collidable_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<NoPeeLogic>(this).no_pee_collidable_logic_array.clone()
}

/// `-[NoPeeLogic noPeeTreasureLogicArray]` @ 0x10019d3dc
pub fn no_pee_treasure_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<NoPeeLogic>(this).no_pee_treasure_logic_array.clone()
}

/// `-[NoPeeLogic noPeeSeaBlobLogicArray]` @ 0x10019d3ec
pub fn no_pee_sea_blob_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<NoPeeLogic>(this).no_pee_sea_blob_logic_array.clone()
}

/// `-[NoPeeLogic noPeeFishLogicArray]` @ 0x10019d3fc
pub fn no_pee_fish_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<NoPeeLogic>(this).no_pee_fish_logic_array.clone()
}

/// `-[NoPeeLogic noPeeMineLogicArray]` @ 0x10019d40c
pub fn no_pee_mine_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<NoPeeLogic>(this).no_pee_mine_logic_array.clone()
}

/// `-[NoPeeLogic noPeeClamLogicArray]` @ 0x10019d41c
pub fn no_pee_clam_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<NoPeeLogic>(this).no_pee_clam_logic_array.clone()
}

/// `-[NoPeeLogic noPeeParticleLogicArray]` @ 0x10019d42c
pub fn no_pee_particle_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<NoPeeLogic>(this).no_pee_particle_logic_array.clone()
}

/// `-[NoPeeLogic noPeePygmyLogic]` @ 0x10019d43c
pub fn no_pee_pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<NoPeeLogic>(this).no_pee_pygmy_logic
}

/// `-[NoPeeLogic isGameActive]` @ 0x10019d44c
///
/// Active once the first treasure is collected (not `mbGameActive`).
pub fn is_game_active(world: &mut World, this: Entity) -> bool {
    world.obj::<NoPeeLogic>(this).no_pee_score > 0
}

/// `-[NoPeeLogic onCollectNoPeeTreasure]` @ 0x10019d464
pub fn on_collect_no_pee_treasure(world: &mut World, this: Entity) {
    world.obj_mut::<NoPeeLogic>(this).no_pee_score += 1;

    let map = world.obj::<NoPeeLogic>(this).no_pee_map_logic;
    if let Some(map) = map {
        no_pee_map_logic::take_breath(world, map);
    }

    let sea_blobs = world.obj::<NoPeeLogic>(this).no_pee_sea_blob_logic_array.clone();
    for sea_blob in sea_blobs {
        let score = world.obj::<NoPeeLogic>(this).no_pee_score;
        no_pee_sea_blob_logic::on_collect_no_pee_treasure(world, sea_blob, score);
    }

    let challenge_manager = crate::managers::challenge_manager::instance(world);
    let score = world.obj::<NoPeeLogic>(this).no_pee_score;
    if let Some(challenge_manager) = challenge_manager {
        crate::managers::challenge_manager::set_game_score(world, challenge_manager, CHALLENGE_GAME_NO_PEE, score);
    }
}

/// `-[NoPeeLogic onMultiModelEvent:eventId:]` @ 0x10019d5e0
pub fn on_multi_model_event(world: &mut World, this: Entity, _logic: Entity, event_id: &str) {
    if event_id != "NoPeeIntroComplete" {
        return;
    }

    // Original bug, kept: clears channel 1 of this logic's own player, not of the intro's,
    // which still holds the pygmy.
    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 1);

    let pygmy = world.obj::<NoPeeLogic>(this).no_pee_pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::on_no_pee_intro_complete(world, pygmy);
    }

    let listener = world.obj::<NoPeeLogic>(this).no_pee_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_no_pee_start_moving_camera(world, listener);
    }
}

/// `-[NoPeeLogic onPygmyStartSwimming]` @ 0x10019d680
pub fn on_pygmy_start_swimming(world: &mut World, this: Entity) {
    let overlay = world.obj::<NoPeeLogic>(this).no_pee_overlay_logic;
    if let Some(overlay) = overlay {
        logic::set_behavior(world, overlay, Some("NoPeeOverlayExit"));
    }
}

/// `-[NoPeeLogic onPygmyKilled]` @ 0x10019d6a4
pub fn on_pygmy_killed(world: &mut World, this: Entity) {
    let retry_overlay = world.obj::<NoPeeLogic>(this).no_pee_retry_overlay_logic;
    let Some(retry_overlay) = retry_overlay else {
        return;
    };

    let retry_display_object = logic::display_object(world, retry_overlay);
    display::object_mut(world, retry_display_object).is_selectable = true;
    logic::set_behavior(world, retry_overlay, Some("NoPeeRetryOverlayEnter"));
}

/// `-[NoPeeLogic stopNoPee]` @ 0x10019d708
pub fn stop_no_pee(world: &mut World, this: Entity) {
    {
        let mut no_pee = world.obj_mut::<NoPeeLogic>(this);
        no_pee.game_active = false;
        no_pee.no_pee_pygmy_logic = None;
    }

    let map = world.obj::<NoPeeLogic>(this).no_pee_map_logic;
    if let Some(map) = map {
        no_pee_map_logic::clear_all_tracked_sprite_logics(world, map);
    }

    let listener = world.obj::<NoPeeLogic>(this).no_pee_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_no_pee_stop(world, listener, this);
    }
}

/// `-[NoPeeLogic startNoPee:]` @ 0x10019d774
pub fn start_no_pee(world: &mut World, this: Entity, pygmy_logic: Entity) {
    world.obj_mut::<NoPeeLogic>(this).no_pee_pygmy_logic = Some(pygmy_logic);
    init_no_pee(world, this);

    let listener = world.obj::<NoPeeLogic>(this).no_pee_logic_listener;
    let pygmy = world.obj::<NoPeeLogic>(this).no_pee_pygmy_logic;
    if let (Some(listener), Some(pygmy)) = (listener, pygmy) {
        crate::controller::on_no_pee_start(world, listener, this, pygmy);
    }
}

/// `-[NoPeeLogic initNoPee]` @ 0x10019d7cc
///
/// A new game: the pygmy swims down in the intro, the instructions show, every world object
/// respawns and the minimap tracks the pygmy, clams, blobs, mines and treasure.
pub fn init_no_pee(world: &mut World, this: Entity) {
    {
        let mut no_pee = world.obj_mut::<NoPeeLogic>(this);
        no_pee.no_pee_score = 0;
        no_pee.game_active = true;
    }

    let pygmy = world.obj::<NoPeeLogic>(this).no_pee_pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::start_no_pee(world, pygmy);
    }

    let intro = world.obj::<NoPeeLogic>(this).no_pee_intro_logic;
    if let Some(intro) = intro {
        let intro_animation = logic::animation(world, intro);
        let pygmy = world.obj::<NoPeeLogic>(this).no_pee_pygmy_logic;
        let pygmy_display_object = match pygmy {
            Some(pygmy) => Some(logic::display_object(world, pygmy)),
            None => None,
        };
        pg_engine::animation::set_tweenable(world, intro_animation, pygmy_display_object, 1);
        logic::set_behavior(world, intro, Some("NoPeeIntro"));
    }

    let overlay = world.obj::<NoPeeLogic>(this).no_pee_overlay_logic;
    if let Some(overlay) = overlay {
        logic::set_behavior(world, overlay, Some("NoPeeOverlayInit"));
    }

    let retry_overlay = world.obj::<NoPeeLogic>(this).no_pee_retry_overlay_logic;
    if let Some(retry_overlay) = retry_overlay {
        let retry_display_object = logic::display_object(world, retry_overlay);
        display::object_mut(world, retry_display_object).is_selectable = false;
        logic::set_behavior(world, retry_overlay, Some("NoPeeRetryOverlayHide"));
    }

    let no_pee_display_group = world.obj::<NoPeeLogic>(this).no_pee_display_group;
    if let Some(no_pee_display_group) = no_pee_display_group {
        let mut group_display = display::object_mut(world, no_pee_display_group);
        group_display.pos.x = 0.0;
        group_display.pos.y = START_GROUP_Y_POS;
    }

    let treasures = world.obj::<NoPeeLogic>(this).no_pee_treasure_logic_array.clone();
    init_no_pee_object_logic_array(world, this, &treasures);
    let sea_blobs = world.obj::<NoPeeLogic>(this).no_pee_sea_blob_logic_array.clone();
    init_no_pee_object_logic_array(world, this, &sea_blobs);
    let fish = world.obj::<NoPeeLogic>(this).no_pee_fish_logic_array.clone();
    init_no_pee_object_logic_array(world, this, &fish);
    let mines = world.obj::<NoPeeLogic>(this).no_pee_mine_logic_array.clone();
    init_no_pee_object_logic_array(world, this, &mines);
    let clams = world.obj::<NoPeeLogic>(this).no_pee_clam_logic_array.clone();
    init_no_pee_object_logic_array(world, this, &clams);
    let particles = world.obj::<NoPeeLogic>(this).no_pee_particle_logic_array.clone();
    init_no_pee_object_logic_array(world, this, &particles);

    let map = world.obj::<NoPeeLogic>(this).no_pee_map_logic;
    if let Some(map) = map {
        no_pee_map_logic::start_tracking(world, map);
        let pygmy = world.obj::<NoPeeLogic>(this).no_pee_pygmy_logic;
        no_pee_map_logic::add_tracked_sprite_logic(world, map, pygmy, MAP_ITEM_PYGMY);
    }

    let clams = world.obj::<NoPeeLogic>(this).no_pee_clam_logic_array.clone();
    add_no_pee_object_logic_array_to_map(world, this, &clams, MAP_ITEM_OBSTACLE);
    let sea_blobs = world.obj::<NoPeeLogic>(this).no_pee_sea_blob_logic_array.clone();
    add_no_pee_object_logic_array_to_map(world, this, &sea_blobs, MAP_ITEM_BLOB);
    let mines = world.obj::<NoPeeLogic>(this).no_pee_mine_logic_array.clone();
    add_no_pee_object_logic_array_to_map(world, this, &mines, MAP_ITEM_OBSTACLE);
    let treasures = world.obj::<NoPeeLogic>(this).no_pee_treasure_logic_array.clone();
    add_no_pee_object_logic_array_to_map(world, this, &treasures, MAP_ITEM_TREASURE);
}

/// `-[NoPeeLogic onGameFrame:]` @ 0x10019da5c
///
/// Parallax: the foreground and backgrounds follow the world group at their own rates, and the
/// overlays stay pinned to the screen.
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    let game_active = world.obj::<NoPeeLogic>(this).game_active;
    if shutdown || !game_active {
        return;
    }

    let no_pee = world.obj::<NoPeeLogic>(this);
    let Some(no_pee_display_group) = no_pee.no_pee_display_group else {
        return;
    };
    let layers = [
        (no_pee.no_pee_underwater_foreground_logic, FOREGROUND_PARALLAX),
        (no_pee.no_pee_underwater_background1_logic, BACKGROUND1_PARALLAX),
        (no_pee.no_pee_underwater_background2_logic, BACKGROUND2_PARALLAX),
        (no_pee.no_pee_underwater_background3_logic, BACKGROUND3_PARALLAX),
    ];
    let overlays = [no_pee.no_pee_overlay_logic, no_pee.no_pee_retry_overlay_logic];

    let group_pos = display::object(world, no_pee_display_group).pos;

    for (layer, parallax) in layers {
        let Some(layer) = layer else {
            continue;
        };
        let layer_display_object = logic::display_object(world, layer);
        let mut layer_display = display::object_mut(world, layer_display_object);
        layer_display.pos.x = group_pos.x * parallax;
        layer_display.pos.y = group_pos.y * parallax;
    }

    for overlay in overlays {
        let Some(overlay) = overlay else {
            continue;
        };
        let overlay_display_object = logic::display_object(world, overlay);
        let mut overlay_display = display::object_mut(world, overlay_display_object);
        overlay_display.pos.x = -group_pos.x;
        overlay_display.pos.y = -group_pos.y;
    }
}

/// `-[NoPeeLogic initNoPeeObjectLogicArray:]` @ 0x10019dc44
pub fn init_no_pee_object_logic_array(world: &mut World, _this: Entity, init_no_pee_object_logic_array: &[Entity]) {
    for &object in init_no_pee_object_logic_array {
        crate::send::activate(world, Some(object));
    }
}

/// `-[NoPeeLogic addNoPeeObjectLogicArrayToMap:type:]` @ 0x10019dd48
pub fn add_no_pee_object_logic_array_to_map(world: &mut World, this: Entity, no_pee_object_logic_array_to_map: &[Entity], type_: i32) {
    for &object in no_pee_object_logic_array_to_map {
        let map = world.obj::<NoPeeLogic>(this).no_pee_map_logic;
        if let Some(map) = map {
            no_pee_map_logic::add_tracked_sprite_logic(world, map, Some(object), type_);
        }
    }
}

/// `-[NoPeeLogic onButtonPress:touch:]` @ 0x10019de6c
///
/// The retry button: a new game with the same pygmy.
pub fn on_button_press(world: &mut World, this: Entity, _button: Entity, _touch: Entity) {
    let map = world.obj::<NoPeeLogic>(this).no_pee_map_logic;
    if let Some(map) = map {
        no_pee_map_logic::clear_all_tracked_sprite_logics(world, map);
    }

    let pygmy = world.obj::<NoPeeLogic>(this).no_pee_pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::trigger_active_sacrifice(world, pygmy);
    }

    init_no_pee(world, this);

    let listener = world.obj::<NoPeeLogic>(this).no_pee_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_no_pee_retry(world, listener, this);
    }

    let retry_overlay = world.obj::<NoPeeLogic>(this).no_pee_retry_overlay_logic;
    if let Some(retry_overlay) = retry_overlay {
        logic::set_behavior(world, retry_overlay, Some("NoPeeRetryOverlayHide"));
    }
}

/// `-[NoPeeLogic onButtonRelease:touch:]` @ 0x10019df14
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[NoPeeLogic onButtonReleaseOutside:touch:]` @ 0x10019df18
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `+[NoPeeLogic instance]` @ 0x10019c874
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().no_pee_logic
}

/// `[[noPeeLogic displayObject] pos]`: the position of the No Pee world group, which the world
/// objects and the minimap read through their listeners' `noPeeLogic`. The listeners are wired
/// before any of them runs; a nil one (whose `pos` the original would read through a null
/// pointer) answers the origin.
pub(crate) fn world_group_pos(world: &World, no_pee_logic: Option<Entity>) -> Vec4 {
    let Some(no_pee_logic) = no_pee_logic else {
        return Vec4::ZERO;
    };
    let no_pee_display_group = logic::display_object(world, no_pee_logic);
    display::object(world, no_pee_display_group).pos
}
