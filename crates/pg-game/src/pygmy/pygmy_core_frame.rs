//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use pg_engine::display::text::Justification;
use pg_engine::view::BcView;

use crate::logic::island_core::sprite_logic::SpriteLogic;
use crate::logic::konkeydong::kong_statue_logic::KongStatueLogic;

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyPygmyCoreFrame {
    /// `mPygmyLogicListener`: `@"<PygmyLogicListener>"`
    pub pygmy_logic_listener: Option<Entity>,

    /// `mPygmyPersona`: `@"PygmyPersona"`
    pub pygmy_persona: Option<Entity>,

    /// `mPersonaPersonality`: `i`
    pub persona_personality: i32,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mGravityTarget`: `{?="x"f"y"f"z"f"w"f}`
    pub gravity_target: Vec4,

    /// `mIslandYPos`: `f`
    pub island_y_pos: f32,

    /// `mSlideFriction`: `f`
    pub slide_friction: f32,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mbShakeUp`: `B`
    pub shake_up: bool,

    /// `mShakeCount`: `i`
    pub shake_count: i32,

    /// `mLastShakeTime`: `f`
    pub last_shake_time: f32,

    /// `mbEarthquake`: `B`
    pub earthquake: bool,

    /// `mTiltState`: `i`
    pub tilt_state: i32,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mCookTime`: `f`
    pub cook_time: f32,

    /// `mbDroppedByHurricane`: `B`
    pub dropped_by_hurricane: bool,

    /// `mDragLabelXOffset`: `f`
    pub drag_label_x_offset: f32,

    /// `mDragLabelYOffset`: `f`
    pub drag_label_y_offset: f32,

    /// `mNameTextXOffset`: `f`
    pub name_text_x_offset: f32,

    /// `mNameTextYOffset`: `f`
    pub name_text_y_offset: f32,

    /// `mNameBackground`: `@"BCRect"`
    pub name_background: Option<Entity>,

    /// `mNameText`: `@"BCText"`
    pub name_text: Option<Entity>,

    /// `mName`: `@"NSString"`
    pub name: Option<String>,

    /// `mAshPileDisplayObject`: `@"BCDisplayObject"`
    pub ash_pile_display_object: Option<Entity>,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbEnableIslandSideCollision`: `B`
    pub enable_island_side_collision: bool,

    /// `mbIslandSideCollision`: `B`
    pub island_side_collision: bool,

    /// `mbScreenSideCollision`: `B`
    pub screen_side_collision: bool,

    /// `mbInvalidSacrifice`: `B`
    pub invalid_sacrifice: bool,

    /// `mSacrificeId`: `@"NSString"`
    pub sacrifice_id: Option<String>,

    /// `mbEatenByPiranha`: `B`
    pub eaten_by_piranha: bool,

    /// `mbIsGhost`: `B`
    pub is_ghost: bool,

    /// `mbTouchedByGhost`: `B`
    pub touched_by_ghost: bool,

    /// `mbCarriedAwayByGhost`: `B`
    pub carried_away_by_ghost: bool,

    /// `mbHoldEarsFromBansheeScream`: `B`
    pub hold_ears_from_banshee_scream: bool,

    /// `mbExplodeFromBansheeScream`: `B`
    pub explode_from_banshee_scream: bool,

    /// `mbCaughtByMonkey`: `B`
    pub caught_by_monkey: bool,

    /// `mbMonkeyOnBack`: `B`
    pub monkey_on_back: bool,

    /// `mbEatenByMoronSnake`: `B`
    pub eaten_by_moron_snake: bool,

    /// `mbTrappedInZombieGrave`: `B`
    pub trapped_in_zombie_grave: bool,

    /// `mbIsZombie`: `B`
    pub is_zombie: bool,

    /// `mbZombieExplode`: `B`
    pub zombie_explode: bool,

    /// `mbTrappedInIdolPedestal`: `B`
    pub trapped_in_idol_pedestal: bool,

    /// `mbInsideTempleDoor`: `B`
    pub inside_temple_door: bool,

    /// `mbInsideApocalypseTempleDoor`: `B`
    pub inside_apocalypse_temple_door: bool,

    /// `mbOnTopApocalypseTimeTemplePlatform`: `B`
    pub on_top_apocalypse_time_temple_platform: bool,

    /// `mbEatenByLocustQueen`: `B`
    pub eaten_by_locust_queen: bool,

    /// `mbThrownByLocusts`: `B`
    pub thrown_by_locusts: bool,

    /// `mbKilledByLocusts`: `B`
    pub killed_by_locusts: bool,

    /// `mbMutatedByLocusts`: `B`
    pub mutated_by_locusts: bool,

    /// `mbKilledByPlague`: `B`
    pub killed_by_plague: bool,

    /// `mbKilledByPlagueDisappear`: `B`
    pub killed_by_plague_disappear: bool,

    /// `mbKilledByPurification`: `B`
    pub killed_by_purification: bool,

    /// `mbInApocalypseTempleFountain`: `B`
    pub in_apocalypse_temple_fountain: bool,

    /// `mbPossessedByThetan`: `B`
    pub possessed_by_thetan: bool,

    /// `mbThetanBrawl`: `B`
    pub thetan_brawl: bool,

    /// `mbDiedFromThetanBrawl`: `B`
    pub died_from_thetan_brawl: bool,

    /// `mbOnClosetTempleThetanMeter`: `B`
    pub on_closet_temple_thetan_meter: bool,

    /// `mbOnClosetTempleCouch`: `B`
    pub on_closet_temple_couch: bool,

    /// `mbInClosetTempleClosetDoor`: `B`
    pub in_closet_temple_closet_door: bool,

    /// `mbSlammingClosetTempleClosetDoor`: `B`
    pub slamming_closet_temple_closet_door: bool,

    /// `mbHeldByXenu`: `B`
    pub held_by_xenu: bool,

    /// `mbHeldInTomsArms`: `B`
    pub held_in_toms_arms: bool,

    /// `mbHoldingBabyPygmy`: `B`
    pub holding_baby_pygmy: bool,

    /// `mTimeTemplePygmyType`: `i`
    pub time_temple_pygmy_type: i32,

    /// `mbTrappedInSpiderWeb`: `B`
    pub trapped_in_spider_web: bool,

    /// `mbOnFailBoat`: `B`
    pub on_fail_boat: bool,

    /// `mbCutInHalfByCharlie`: `B`
    pub cut_in_half_by_charlie: bool,

    /// `mbSunBurnt`: `B`
    pub sun_burnt: bool,

    /// `mbSunFrozen`: `B`
    pub sun_frozen: bool,

    /// `mbHitByIcicle`: `B`
    pub hit_by_icicle: bool,

    /// `mbBeheadedBySquid`: `B`
    pub beheaded_by_squid: bool,

    /// `mbAteSwordFish`: `B`
    pub ate_sword_fish: bool,

    /// `mbPainDrain`: `B`
    pub pain_drain: bool,

    /// `mbPainDrainSpiked`: `B`
    pub pain_drain_spiked: bool,

    /// `mbSuckedIntoDrain`: `B`
    pub sucked_into_drain: bool,

    /// `mbIglooRoll`: `B`
    pub igloo_roll: bool,

    /// `mbHitBySnowBall`: `B`
    pub hit_by_snow_ball: bool,

    /// `mbRestingInIgloo`: `B`
    pub resting_in_igloo: bool,

    /// `mbInApeWench`: `B`
    pub in_ape_wench: bool,

    /// `mApeWenchLogic`: `@"ApeWenchLogic"`
    pub ape_wench_logic: Option<Entity>,

    /// `mbInApeBongo`: `B`
    pub in_ape_bongo: bool,

    /// `mApeBongoLogic`: `@"ApeBongoLogic"`
    pub ape_bongo_logic: Option<Entity>,

    /// `mbInApeAltar`: `B`
    pub in_ape_altar: bool,

    /// `mApeAltarLogic`: `@"ApeAltarLogic"`
    pub ape_altar_logic: Option<Entity>,

    /// `mbInDanceDanceExecution`: `B`
    pub in_dance_dance_execution: bool,

    /// `mbDanceDanceExecuted`: `B`
    pub dance_dance_executed: bool,

    /// `mbInGoGoCage`: `B`
    pub in_go_go_cage: bool,

    /// `mbHighOnNitrous`: `B`
    pub high_on_nitrous: bool,

    /// `mbRemoveFromStoryMode`: `B`
    pub remove_from_story_mode: bool,

    /// `mbCaughtByBird`: `B`
    pub caught_by_bird: bool,

    /// `mbInCrackHole`: `B`
    pub in_crack_hole: bool,

    /// `mbTheRunsSideCollision`: `B`
    pub the_runs_side_collision: bool,

    /// `mbTheRunsBurnComplete`: `B`
    pub the_runs_burn_complete: bool,

    /// `mbTheRunsFallInLava`: `B`
    pub the_runs_fall_in_lava: bool,

    /// `mbKonkeyDongHit`: `B`
    pub konkey_dong_hit: bool,

    /// `mKonkeyDongPickTime`: `f`
    pub konkey_dong_pick_time: f32,

    /// `mbMoronPests`: `B`
    pub moron_pests: bool,

    /// `mbMoronPestsStarted`: `B`
    pub moron_pests_started: bool,

    /// `mbMoronPestsEatenByTurtle`: `B`
    pub moron_pests_eaten_by_turtle: bool,

    /// `mbWearingWaterWings`: `B`
    pub wearing_water_wings: bool,

    /// `mbInKongStatue`: `B`
    pub in_kong_statue: bool,

    /// `mbKidnappedByKonkeyDong`: `B`
    pub kidnapped_by_konkey_dong: bool,

    /// `mKongStatueLogic`: `@"KongStatueLogic"`
    pub kong_statue_logic: Option<Entity>,

    /// `mbBattleOfTheGods`: `B`
    pub battle_of_the_gods: bool,

    /// `mbDoodleJumping`: `B`
    pub doodle_jumping: bool,

    /// `mbDoodleKnockedDown`: `B`
    pub doodle_knocked_down: bool,

    /// `mbSkewered`: `B`
    pub skewered: bool,

    /// `mbUnderwater`: `B`
    pub underwater: bool,

    /// `mbCrapping`: `B`
    pub crapping: bool,

    /// `mbEatenByTRex`: `B`
    pub eaten_by_t_rex: bool,

    /// `mbEatenBySpider`: `B`
    pub eaten_by_spider: bool,

    /// `mbExplodeFromNotCrapping`: `B`
    pub explode_from_not_crapping: bool,

    /// `mbExplodeFromVampireSun`: `B`
    pub explode_from_vampire_sun: bool,

    /// `mbExplodeFromVampireGinger`: `B`
    pub explode_from_vampire_ginger: bool,

    /// `mbExplodeFromLovingPocketGod`: `B`
    pub explode_from_loving_pocket_god: bool,

    /// `mbDisableLabels`: `B`
    pub disable_labels: bool,

    /// `mbEnableUpsideDown`: `B`
    pub enable_upside_down: bool,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbCatchShark`: `B`
    pub catch_shark: bool,

    /// `mbSharkOnHead`: `B`
    pub shark_on_head: bool,

    /// `mbCrushed`: `B`
    pub crushed: bool,

    /// `mbBurntByMagnifyingGlass`: `B`
    pub burnt_by_magnifying_glass: bool,

    /// `mbSwarmed`: `B`
    pub swarmed: bool,

    /// `mbSleeping`: `B`
    pub sleeping: bool,

    /// `mbTumble`: `B`
    pub tumble: bool,

    /// `mbDrowning`: `B`
    pub drowning: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,

    /// `mbOnFireFromLava`: `B`
    pub on_fire_from_lava: bool,

    /// `mbOnFireFromBonFire`: `B`
    pub on_fire_from_bon_fire: bool,

    /// `mbAteBurntFood`: `B`
    pub ate_burnt_food: bool,

    /// `mbCooked`: `B`
    pub cooked: bool,

    /// `mbHitByLightning`: `B`
    pub hit_by_lightning: bool,
}

/// The stop list the original repeats inline before every tilt, balance and earthquake
/// behaviour (`S13` in `docs/re/pygmy.md` §1.3).
fn stop_activities(world: &mut World, this: Entity) {
    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);
}

/// The early returns `refreshTiltState` and `undoTiltState` both inline: NO while the pygmy
/// is busy, off the island without a fishing pole, or shaken by an earthquake.
fn can_change_tilt_state(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);
    let island_life = world.obj::<PygmyIslandLife>(this);

    let busy = core.dragging
        || core.touch_began_inside
        || core.remove_from_story_mode
        || core.falling
        || core.in_crack_hole
        || core.doodle_jumping
        || core.crushed
        || core.caught_by_bird
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.crapping
        || core.trapped_in_spider_web
        || core.trapped_in_zombie_grave
        || core.trapped_in_idol_pedestal
        || core.inside_temple_door
        || core.inside_apocalypse_temple_door
        || core.on_top_apocalypse_time_temple_platform
        || core.eaten_by_locust_queen
        || core.in_apocalypse_temple_fountain
        || core.thetan_brawl
        || core.on_closet_temple_thetan_meter
        || core.on_closet_temple_couch
        || core.in_closet_temple_closet_door
        || core.slamming_closet_temple_closet_door
        || core.held_by_xenu
        || core.held_in_toms_arms
        || core.holding_baby_pygmy
        || core.thrown_by_locusts
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.battle_of_the_gods
        || core.is_ghost
        || core.touched_by_ghost
        || core.possessed_by_thetan
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.zombie_explode
        || core.skewered
        || core.underwater
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.catch_shark
        || core.shark_on_head
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.drowning;
    if busy {
        return false;
    }

    if !core.on_island && island_life.fishing_pole_logic.is_none() {
        return false;
    }

    !core.earthquake
}

/// `-[PygmyLogic initWithDisplayObject:]` @ 0x1001286ec
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    insert_components(world, this);
    crate::logic::island_core::sprite_logic::init_with_display_object(world, this, display_object);

    // `DAT_100816200`, the dialog dictionary of `checkPygmyDialog:::`, is created once.
    world.get_resource_or_insert_with(PygmyPygmyBehaviourItemsStatics::default);

    let random = world.resource_mut::<Random>().arc4random();
    let friction_step = (random % 401) as f32;
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.tilt_state = 0;
        core.enable_island_floor_collision = true;
        core.slide_friction = friction_step / 10000.0 + 0.03;
        core.spring_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    }

    let name_background = pg_engine::display::rect::spawn(world, Some("PygmyText"), -50.0, 0.0, 50.0, 14.0);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).name_background = Some(name_background);
    pg_engine::display::object_mut(world, name_background).set_rgba(0.0, 0.0, 0.0, 0.5);

    let name_text = pg_engine::display::text::spawn(world, Some("PygmyText"), Some("Arial12"));
    world.obj_mut::<PygmyPygmyCoreFrame>(this).name_text = Some(name_text);
    pg_engine::display::text::set_font_justification(world, name_text, Justification::Center);

    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.island_y_pos = 86.0;
        core.falling = true;
        core.invalid_sacrifice = true;
    }
    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic isSoundDisabled]` @ 0x100128940
pub fn is_sound_disabled(world: &World, this: Entity) -> bool {
    world.obj::<PygmyPygmyCoreFrame>(this).in_go_go_cage
}

/// `-[PygmyLogic applyMoveXPos:moveYPos:]` @ 0x100128950
pub fn apply_move_x_pos(world: &mut World, this: Entity, move_x: f32, move_y: f32) {
    let in_go_go_cage = world.obj::<PygmyPygmyCoreFrame>(this).in_go_go_cage;
    if !in_go_go_cage {
        pg_engine::logic::apply_move_pos_base(world, this, move_x, move_y);
    }
}

/// `-[PygmyLogic pygmyNameLabel]` @ 0x10012899c
pub fn pygmy_name_label(world: &mut World, this: Entity) -> Option<String> {
    world.obj::<PygmyPygmyCoreFrame>(this).name.clone()
}

/// `-[PygmyLogic setPygmyNameLabel:]` @ 0x1001289ac
pub fn set_pygmy_name_label(world: &mut World, this: Entity, pygmy_name_label: Option<&str>) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).name = pygmy_name_label.map(str::to_owned);

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let time_temple_pygmy_type = core.time_temple_pygmy_type;
    let name_text = core.name_text;
    let name_background = core.name_background;

    // `%@` prints a nil name as "(null)".
    let formatted_name = pygmy_name_label.unwrap_or("(null)");
    let label = match time_temple_pygmy_type {
        1 => Some(format!("Old Fart {formatted_name}")),
        2 => Some(format!("Baby {formatted_name}")),
        3 => Some(format!("Ginger {formatted_name}")),
        4 => Some(format!("Fast {formatted_name}")),
        _ => pygmy_name_label.map(str::to_owned),
    };

    let mut text_width = 0.0;
    if let Some(name_text) = name_text {
        pg_engine::display::text::set_text(world, name_text, label.as_deref());
        text_width = pg_engine::display::text::text_width(world, name_text);
    }

    let half_width = text_width * 0.5;
    if let Some(name_background) = name_background {
        pg_engine::display::rect::set_min_x_pos(world, name_background, -1.0 - half_width);
        pg_engine::display::rect::set_max_x_pos(world, name_background, half_width + 1.0);
        pg_engine::display::rect::set_min_y_pos(world, name_background, -14.0);
        pg_engine::display::rect::set_max_y_pos(world, name_background, 0.0);
    }
    world.obj_mut::<PygmyPygmyCoreFrame>(this).name_text_y_offset = -13.0;
}

/// `-[PygmyLogic setPygmyPersona:]` @ 0x100128bb0
pub fn set_pygmy_persona(world: &mut World, this: Entity, pygmy_persona: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).pygmy_persona = Some(pygmy_persona);

    let persona_personality = crate::persona::pygmy_persona::persona_personality(world, pygmy_persona);
    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.persona_personality = persona_personality;
    let name_background = core.name_background;

    if persona_personality != 0 {
        if let Some(name_background) = name_background {
            pg_engine::display::object_mut(world, name_background).set_rgba(0.4, 0.0, 0.0, 0.5);
        }
    }
}

/// `-[PygmyLogic pygmyPersona]` @ 0x100128c6c
pub fn pygmy_persona(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyPygmyCoreFrame>(this).pygmy_persona
}

/// `-[PygmyLogic personaPersonality]` @ 0x100128c7c
pub fn persona_personality(world: &mut World, this: Entity) -> i32 {
    world.obj::<PygmyPygmyCoreFrame>(this).persona_personality
}

/// `-[PygmyLogic setPygmyLogicListener:]` @ 0x100128d04
pub fn set_pygmy_logic_listener(world: &mut World, this: Entity, pygmy_logic_listener: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener = Some(pygmy_logic_listener);
}

/// `-[PygmyLogic dealloc]` @ 0x100128d60
///
/// Called by whoever releases the pygmy last (the controller's `onPygmyDie:sacrificeId:`
/// path). `mpGhostPathVector` and the retained name, persona and listener go away with the
/// components.
pub fn dealloc(world: &mut World, this: Entity) {
    let ash_pile_display_object = world.obj::<PygmyPygmyCoreFrame>(this).ash_pile_display_object;
    if let Some(ash_pile_display_object) = ash_pile_display_object {
        pg_engine::display::remove_from_display_group(world, ash_pile_display_object);

        let ash_pile_logic = world.obj::<PygmyPygmyBehaviourItems>(this).ash_pile_logic;
        if let Some(ash_pile_logic) = ash_pile_logic {
            pg_engine::logic::shutdown(world, ash_pile_logic);
            pg_engine::logic::dealloc(world, ash_pile_logic);
        }
        world.despawn(ash_pile_display_object);

        world.obj_mut::<PygmyPygmyCoreFrame>(this).ash_pile_display_object = None;
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).ash_pile_logic = None;
    }

    let name_background = world.obj::<PygmyPygmyCoreFrame>(this).name_background;
    if let Some(name_background) = name_background {
        pg_engine::display::remove_from_display_group(world, name_background);
        world.despawn(name_background);
    }
    world.obj_mut::<PygmyPygmyCoreFrame>(this).name_background = None;

    let name_text = world.obj::<PygmyPygmyCoreFrame>(this).name_text;
    if let Some(name_text) = name_text {
        pg_engine::display::remove_from_display_group(world, name_text);
        world.despawn(name_text);
    }
    world.obj_mut::<PygmyPygmyCoreFrame>(this).name_text = None;

    // `[super dealloc]`: `SpriteLogic` only releases its cloud style string.
    pg_engine::logic::dealloc(world, this);
}

/// `-[PygmyLogic updateLabelLayer]` @ 0x100128ed4
pub fn update_label_layer(world: &mut World, this: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let listener = core.pygmy_logic_listener;
    let name_text = core.name_text;
    let name_background = core.name_background;
    let display = pg_engine::logic::display_object(world, this);

    let names_enabled = listener.is_some_and(|listener| crate::controller::are_name_labels_enabled(world, listener));
    if names_enabled {
        if let Some(name_text) = name_text {
            pg_engine::display::remove_from_display_group(world, name_text);
        }
        if let Some(name_background) = name_background {
            pg_engine::display::remove_from_display_group(world, name_background);
        }
        pg_engine::display::insert_after(world, display, name_text);
        pg_engine::display::insert_after(world, display, name_background);
    }

    let apocalypse_cloud_logic = world.obj::<SpriteLogic>(this).apocalypse_cloud_logic;
    if let Some(apocalypse_cloud_logic) = apocalypse_cloud_logic {
        let cloud_display = pg_engine::logic::display_object(world, apocalypse_cloud_logic);
        pg_engine::display::insert_after(world, display, Some(cloud_display));
    }
}

/// `-[PygmyLogic checkPygmyCollision]` @ 0x100128fe8
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    if !core.dragging || !core.cooked {
        return;
    }
    let listener = core.pygmy_logic_listener;

    let display = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display).pos;
    let pos_vel = pg_engine::logic::logic(world, this).pos_vel;
    let speed = pg_engine::math::length3(pos_vel);

    let pygmy_logic_array = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };

    for other in pygmy_logic_array {
        if other != this && 200.0 < speed {
            let other_display = pg_engine::logic::display_object(world, other);
            let other_pos = pg_engine::display::object(world, other_display).pos;
            let delta_x = pos.x - (other_pos.x + 0.0);
            let delta_y = pos.y - (other_pos.y + 46.5);
            let distance_squared = delta_x * delta_x + delta_y * delta_y;
            if distance_squared < 1600.0 && hit_by_cooked_pygmy(world, other, this) {
                break;
            }
        }
    }
}

/// `-[PygmyLogic isAccelEnabled]` @ 0x10012924c
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);
    let listener = core.pygmy_logic_listener;

    let busy = core.tumble
        || (core.earthquake && !core.falling)
        || core.remove_from_story_mode
        || core.in_crack_hole
        || core.doodle_jumping
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.crapping
        || core.trapped_in_spider_web
        || core.trapped_in_zombie_grave
        || core.trapped_in_idol_pedestal
        || core.inside_temple_door
        || core.inside_apocalypse_temple_door
        || core.on_top_apocalypse_time_temple_platform
        || core.eaten_by_locust_queen
        || core.in_apocalypse_temple_fountain
        || core.thetan_brawl
        || core.on_closet_temple_thetan_meter
        || core.on_closet_temple_couch
        || core.in_closet_temple_closet_door
        || core.slamming_closet_temple_closet_door
        || core.held_by_xenu
        || core.held_in_toms_arms
        || core.holding_baby_pygmy
        || core.thrown_by_locusts
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.battle_of_the_gods
        || core.zombie_explode
        || core.is_ghost
        || core.touched_by_ghost
        || core.possessed_by_thetan
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.skewered
        || core.underwater
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || core.crushed
        || core.caught_by_bird
        || core.catch_shark
        || core.dropped_by_hurricane
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.shark_on_head;
    if busy {
        return false;
    }

    let gravity_enabled = listener.is_some_and(|listener| crate::controller::is_gravity_enabled(world, listener));
    if !gravity_enabled {
        return false;
    }

    crate::logic::island_core::sprite_logic::is_accel_enabled(world, this)
}

/// `-[PygmyLogic translateAndMoveToDisplayMarker:]` @ 0x100129894
pub fn translate_and_move_to_display_marker(world: &mut World, this: Entity, marker: Entity) {
    let display = pg_engine::logic::display_object(world, this);
    let display_object = pg_engine::display::object(world, display);
    let pos = display_object.pos;
    let display_group = display_object.display_group;

    // Messages to a nil group answer a zero vector.
    let view_point = match display_group {
        Some(display_group) => pg_engine::display::display_object_to_view(world, display_group, pos),
        None => Vec4::ZERO,
    };
    let marker_group = pg_engine::display::object(world, marker).display_group;
    let marker_point = match marker_group {
        Some(marker_group) => pg_engine::display::view_to_display_object(world, marker_group, view_point),
        None => Vec4::ZERO,
    };

    {
        let mut display_object = pg_engine::display::object_mut(world, display);
        display_object.pos.x = marker_point.x;
        display_object.pos.y = marker_point.y;
    }

    pg_engine::display::insert_after(world, marker, Some(display));
    update_label_layer(world, this);
}

/// `-[PygmyLogic onGameFrame:]` @ 0x100129980
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }

    let no_pee = world.obj::<PygmyPygmyTouchDrag>(this).no_pee;
    if no_pee {
        handle_no_pee_movement(world, this, dt);
        return;
    }

    let display = pg_engine::logic::display_object(world, this);
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;

    let gravity = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let doodle_jet_pack = world.obj::<PygmyMinigames>(this).doodle_jet_pack;
        if core.underwater && !core.dragging && !core.pain_drain {
            30.0
        } else if doodle_jet_pack {
            0.0
        } else if core.doodle_jumping || core.thrown_by_locusts {
            1500.0
        } else {
            3000.0
        }
    };

    let accel_enabled = is_accel_enabled(world, this);
    let eruption_active =
        accel_enabled && listener.is_some_and(|listener| crate::controller::is_eruption_active(world, listener));
    if !accel_enabled || eruption_active {
        world.obj_mut::<SpriteLogic>(this).gravity_force = Vec4::new(0.0, -gravity, 0.0, 1.0);
        undo_tilt_state(world, this);
    } else {
        let accel_force = world.resource::<BcView>().accel_force;
        let direction = Vec4::new(-accel_force.y, accel_force.x, 0.0, 1.0);
        let mut gravity_force = pg_engine::math::normalize3(direction);
        gravity_force.x = gravity * gravity_force.x;
        gravity_force.y = gravity * gravity_force.y;
        gravity_force.z = gravity * gravity_force.z;
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;
        refresh_tilt_state(world, this);
    }
    check_for_earthquake(world, this, dt);

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let tilt_state = core.tilt_state;
    let slide_friction = core.slide_friction;
    let apply_spring = core.apply_spring;
    let spring_force = core.spring_force;
    let (ignore_gravity, gravity_force) = {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        if matches!(tilt_state, 3 | 4) {
            sprite.gravity_force.x = slide_friction * sprite.gravity_force.x;
        } else {
            sprite.gravity_force.x = 0.0;
        }
        (sprite.ignore_gravity, sprite.gravity_force)
    };

    let mut pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    if !ignore_gravity {
        pos_force.x = gravity_force.x + pos_force.x;
        pos_force.y = gravity_force.y + pos_force.y;
        pos_force.z = gravity_force.z + pos_force.z;
    }
    if apply_spring {
        pos_force.x = spring_force.x + pos_force.x;
        pos_force.y = spring_force.y + pos_force.y;
        pos_force.z = spring_force.z + pos_force.z;
    }
    pg_engine::logic::logic_mut(world, this).pos_force = pos_force;

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let tilt_steering = (core.doodle_jumping && !core.doodle_knocked_down) || (core.pain_drain && !core.pain_drain_spiked);
    if tilt_steering {
        let raw_accel_force = world.resource::<BcView>().raw_accel_force;
        pg_engine::logic::logic_mut(world, this).pos_vel.x = raw_accel_force.y * -1000.0;
    }

    let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);
    let konkey_dong = touch_drag.konkey_dong;
    let konkey_dong_ladder_touch = touch_drag.konkey_dong_ladder_touch;
    let minigames = world.obj::<PygmyMinigames>(this);
    let konkey_dong_playing = konkey_dong
        && minigames.konkey_dong_started
        && !minigames.konkey_dong_level_transition
        && !world.obj::<PygmyPygmyCoreFrame>(this).konkey_dong_hit;
    if konkey_dong_playing {
        let minigames = world.obj::<PygmyMinigames>(this);
        let konkey_dong_on_ladder = minigames.konkey_dong_on_ladder;
        let konkey_dong_holding_pick = minigames.konkey_dong_holding_pick;
        let konkey_dong_ladder_up = minigames.konkey_dong_ladder_up;
        let konkey_dong_climbing = minigames.konkey_dong_climbing;

        if !konkey_dong_on_ladder {
            if konkey_dong_holding_pick {
                let pick_time = world.obj::<PygmyPygmyCoreFrame>(this).konkey_dong_pick_time - dt;
                world.obj_mut::<PygmyPygmyCoreFrame>(this).konkey_dong_pick_time = pick_time;
                if pick_time < 0.0 {
                    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
                    minigames.konkey_dong_holding_pick = false;
                    minigames.konkey_dong_running = false;
                }
            }

            let raw_accel_force = world.resource::<BcView>().raw_accel_force;
            let mut x_pos_vel = raw_accel_force.y * -1000.0;
            if x_pos_vel < -100.0 {
                x_pos_vel = -100.0;
            } else if 100.0 < x_pos_vel {
                x_pos_vel = 100.0;
            }
            pg_engine::logic::logic_mut(world, this).pos_vel.x = x_pos_vel;

            let min_pos = pg_engine::display::min_pos(world, display);
            let max_pos = pg_engine::display::max_pos(world, display);
            if min_pos.x < 0.0 {
                world.obj_mut::<PygmyPygmyCoreFrame>(this).screen_side_collision = true;
                let mut display_object = pg_engine::display::object_mut(world, display);
                display_object.pos.x = display_object.pos.x - min_pos.x + 0.0;
            }

            let kong_statue_logic =
                listener.and_then(|listener| crate::controller::kong_statue_logic(world, listener, this));
            if let Some(kong_statue_logic) = kong_statue_logic {
                let konkey_dong_level = crate::logic::konkeydong::kong_statue_logic::konkey_dong_level(world, kong_statue_logic);
                if let Some(konkey_dong_level) = konkey_dong_level {
                    let kong_statue = world.obj::<KongStatueLogic>(kong_statue_logic);
                    let level_width = kong_statue.konkey_dong_level_array[konkey_dong_level].level_width;
                    if level_width < max_pos.x {
                        world.obj_mut::<PygmyPygmyCoreFrame>(this).screen_side_collision = true;
                        let mut display_object = pg_engine::display::object_mut(world, display);
                        display_object.pos.x = level_width - (max_pos.x - display_object.pos.x);
                    }
                }
            }
        } else {
            let climbing = konkey_dong_ladder_touch.is_some();
            let y_pos_vel = if !climbing {
                0.0
            } else if konkey_dong_ladder_up {
                100.0
            } else {
                -100.0
            };
            pg_engine::logic::logic_mut(world, this).pos_vel.y = y_pos_vel;

            if climbing != konkey_dong_climbing {
                world.obj_mut::<PygmyMinigames>(this).konkey_dong_climbing = climbing;
                let behavior = if climbing { "PygmyKonkeyDongLadderClimb" } else { "PygmyKonkeyDongLadderIdle" };
                pg_engine::logic::set_behavior(world, this, Some(behavior));
            }
        }
    }

    let logic = pg_engine::logic::logic(world, this);
    let pos_force = logic.pos_force;
    let mut pos_vel = logic.pos_vel;
    pos_vel.x = pos_vel.x + pos_force.x * dt;
    pos_vel.y = pos_vel.y + pos_force.y * dt;
    pos_vel.z = pos_vel.z + pos_force.z * dt;
    pg_engine::logic::logic_mut(world, this).pos_vel = pos_vel;
    {
        let mut display_object = pg_engine::display::object_mut(world, display);
        display_object.pos.x = display_object.pos.x + pos_vel.x * dt;
        display_object.pos.y = display_object.pos.y + pos_vel.y * dt;
        display_object.pos.z = display_object.pos.z + pos_vel.z * dt;
    }

    let over_fire = world.obj::<PygmyPygmyTouchDrag>(this).over_fire;
    if over_fire {
        let cook_time = world.obj::<PygmyPygmyCoreFrame>(this).cook_time + dt;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).cook_time = cook_time;

        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let healthy =
            !core.cooked && !core.killed_by_plague && !core.cut_in_half_by_charlie && core.time_temple_pygmy_type == 0;
        if 2.0 < cook_time && healthy {
            {
                let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
                core.cooked = true;
                core.sacrifice_id = Some("Cooked".to_owned());
            }
            pg_engine::logic::set_behavior(world, this, Some("PygmyCook"));
        }
    }

    let the_runs = world.obj::<PygmyPygmyTouchDrag>(this).the_runs;
    let the_runs_fall_in_lava = world.obj::<PygmyPygmyCoreFrame>(this).the_runs_fall_in_lava;
    if the_runs && !the_runs_fall_in_lava {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let burn_complete = core.the_runs_burn_complete;
        let side_collision = core.the_runs_side_collision;
        let rock_explode = world.obj::<PygmyMinigames>(this).the_runs_rock_explode;
        if burn_complete && !side_collision && !rock_explode {
            let mut logic = pg_engine::logic::logic_mut(world, this);
            if logic.pos_vel.x < 600.0 {
                logic.pos_vel.x = dt * 50.0 + logic.pos_vel.x;
            } else {
                logic.pos_vel.x = 600.0;
            }
        }

        let y_pos = pg_engine::display::object(world, display).pos.y;
        if y_pos <= 13.0 {
            {
                let mut logic = pg_engine::logic::logic_mut(world, this);
                logic.pos_vel.x = 0.0;
                logic.pos_vel.y = 0.0;
            }
            world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
            world.obj_mut::<PygmyPygmyCoreFrame>(this).the_runs_fall_in_lava = true;
            pg_engine::display::object_mut(world, display).pos.y = 13.0;
            world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("LavaDrown".to_owned());
            pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsSplash"));
            return;
        }
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let moron_pests_racing = core.moron_pests && core.moron_pests_started && !core.moron_pests_eaten_by_turtle;
    if moron_pests_racing {
        let moron_turtle = pg_engine::library::logic(world, crate::logic_table::MORON_TURTLE);
        let turtle_x_pos_vel = pg_engine::logic::logic(world, moron_turtle).pos_vel.x;
        let mut target_x_pos_vel = turtle_x_pos_vel - 10.0;
        if 500.0 < target_x_pos_vel {
            target_x_pos_vel = 500.0;
        }
        let last_target_x_pos_vel = world.obj::<PygmyMinigames>(this).moron_pests_target_x_pos_vel;
        if target_x_pos_vel < last_target_x_pos_vel {
            target_x_pos_vel = last_target_x_pos_vel + dt * -10.0;
        }
        world.obj_mut::<PygmyMinigames>(this).moron_pests_target_x_pos_vel = target_x_pos_vel;

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = logic.pos_vel.x + (target_x_pos_vel - logic.pos_vel.x) * 0.5;
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    if core.underwater && !core.pain_drain {
        let min_pos = pg_engine::display::min_pos(world, display);
        let max_pos = pg_engine::display::max_pos(world, display);
        if min_pos.y < 0.0 {
            let mut display_object = pg_engine::display::object_mut(world, display);
            display_object.pos.y = display_object.pos.y - min_pos.y + 0.0;
        }
        let added_underwater = world.obj::<PygmyUnderwaterNopee>(this).added_underwater;
        if !added_underwater && 320.0 < max_pos.y {
            let mut display_object = pg_engine::display::object_mut(world, display);
            display_object.pos.y = 320.0 - (max_pos.y - display_object.pos.y);
        }
    }

    let is_ghost = world.obj::<PygmyPygmyCoreFrame>(this).is_ghost;
    if is_ghost {
        let ghost = world.obj::<PygmyGhostZombieVampire>(this);
        let ghost_touch_type = ghost.ghost_touch_type;
        let mut ghost_target_z_rot = ghost.ghost_target_z_rot;

        if ghost_touch_type != 1 {
            let mut display_object = pg_engine::display::object_mut(world, display);
            if display_object.pos.y < 40.0 {
                display_object.pos.y = 40.0;
            } else if 280.0 < display_object.pos.y {
                display_object.pos.y = 280.0;
            }
            if display_object.pos.x < 20.0 {
                display_object.pos.x = 20.0;
            } else if 440.0 < display_object.pos.x {
                display_object.pos.x = 440.0;
            }
        }

        let mut z_rot = pg_engine::display::object(world, display).rot.z;
        if 180.0 < (z_rot - ghost_target_z_rot).abs() {
            if z_rot < 0.0 {
                z_rot = z_rot + 360.0;
            } else if 180.0 < z_rot {
                z_rot = z_rot - 360.0;
            } else if ghost_target_z_rot < 0.0 {
                ghost_target_z_rot = ghost_target_z_rot + 360.0;
                world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_target_z_rot = ghost_target_z_rot;
            }
        }
        // Original frame-rate dependent easing (per call, not per second), kept.
        z_rot = z_rot + (ghost_target_z_rot - z_rot) * 0.4;
        pg_engine::display::object_mut(world, display).rot.z = z_rot;
    }

    let island_min_x_pos = listener.map_or(0.0, |listener| crate::controller::island_min_x_pos(world, listener));
    let island_max_x_pos = listener.map_or(0.0, |listener| crate::controller::island_max_x_pos(world, listener));
    let on_fail_boat = world.obj::<PygmyPygmyCoreFrame>(this).on_fail_boat;
    let min_x_pos = if on_fail_boat { -160.0 } else { island_min_x_pos };
    let max_x_pos = if on_fail_boat { 160.0 } else { island_max_x_pos };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).island_side_collision = false;
    let enable_island_side_collision = world.obj::<PygmyPygmyCoreFrame>(this).enable_island_side_collision;
    if enable_island_side_collision {
        let last_x_pos = world.obj::<PygmyPygmyCoreFrame>(this).last_pos.x;
        let x_pos = pg_engine::display::object(world, display).pos.x;
        if min_x_pos <= last_x_pos && x_pos < min_x_pos {
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            pg_engine::display::object_mut(world, display).pos.x = min_x_pos;
            world.obj_mut::<PygmyPygmyCoreFrame>(this).island_side_collision = true;
        }

        let last_x_pos = world.obj::<PygmyPygmyCoreFrame>(this).last_pos.x;
        let x_pos = pg_engine::display::object(world, display).pos.x;
        if last_x_pos <= max_x_pos && max_x_pos < x_pos {
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            pg_engine::display::object_mut(world, display).pos.x = max_x_pos;
            world.obj_mut::<PygmyPygmyCoreFrame>(this).island_side_collision = true;
        }
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).screen_side_collision = false;
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);
    let ghost_touch_type = world.obj::<PygmyGhostZombieVampire>(this).ghost_touch_type;
    let keep_on_screen = !core.caught_by_bird
        && !touch_drag.the_runs
        && !core.on_fail_boat
        && !touch_drag.konkey_dong
        && !core.moron_pests
        && !touch_drag.no_pee
        && !core.battle_of_the_gods
        && !core.carried_away_by_ghost
        && !core.mutated_by_locusts
        && !core.held_by_xenu
        && (!core.killed_by_plague || !core.drowning)
        && ghost_touch_type != 1;
    if keep_on_screen {
        let min_pos = pg_engine::display::min_pos(world, display);
        let max_pos = pg_engine::display::max_pos(world, display);
        if min_pos.x < 0.0 {
            world.obj_mut::<PygmyPygmyCoreFrame>(this).screen_side_collision = true;
            let mut display_object = pg_engine::display::object_mut(world, display);
            display_object.pos.x = display_object.pos.x - min_pos.x + 0.0;
        }
        if 480.0 < max_pos.x {
            world.obj_mut::<PygmyPygmyCoreFrame>(this).screen_side_collision = true;
            let mut display_object = pg_engine::display::object_mut(world, display);
            display_object.pos.x = 480.0 - (max_pos.x - display_object.pos.x);
        }
    }

    world.obj_mut::<PygmyPygmyBehaviourItems>(this).island_floor_collision = false;
    let enable_island_floor_collision = world.obj::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision;
    let mut landed = false;
    if enable_island_floor_collision {
        let island_location_state =
            listener.map_or(0, |listener| crate::controller::island_location_state(world, listener));
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let last_y_pos = core.last_pos.y;
        let island_y_pos = core.island_y_pos;
        let on_fail_boat = core.on_fail_boat;

        if island_location_state != 8 {
            let pos = pg_engine::display::object(world, display).pos;
            if min_x_pos <= pos.x && pos.x <= max_x_pos {
                landed = island_y_pos <= last_y_pos && pos.y < island_y_pos;
            }
        } else if !on_fail_boat {
            let fail_boat = pg_engine::library::logic(world, crate::logic_table::FAIL_BOAT_DISPLAY_GROUP);
            let boat_display = pg_engine::logic::display_object(world, fail_boat);
            let boat_object = pg_engine::display::object(world, boat_display);
            let boat_pos = boat_object.pos;
            let boat_angle = boat_object.rot.z / 180.0 * std::f32::consts::PI;
            let boat_cos = boat_angle.cos();
            let x_pos = pg_engine::display::object(world, display).pos.x;
            let boat_tan = boat_angle.tan();

            let boat_min_x_pos = boat_pos.x + boat_cos * -160.0;
            let boat_max_x_pos = boat_pos.x + boat_cos * 160.0;
            if boat_min_x_pos <= x_pos && x_pos <= boat_max_x_pos {
                let floor_y_pos = boat_pos.y + (x_pos - boat_pos.x) * boat_tan;
                let y_pos = pg_engine::display::object(world, display).pos.y;
                landed = floor_y_pos <= last_y_pos && y_pos < floor_y_pos;
            }
        } else {
            landed = true;
        }
    }

    if landed {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let cooked = core.cooked;
        let killed_by_plague = core.killed_by_plague;
        let on_island = core.on_island;

        if !cooked && !killed_by_plague {
            if !on_island {
                let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
                core.on_island = true;
                core.invalid_sacrifice = false;
                world.obj_mut::<PygmyPygmyBehaviourItems>(this).island_floor_collision = true;
            }
            pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
        } else {
            let y_pos_vel = {
                let mut logic = pg_engine::logic::logic_mut(world, this);
                if !on_island {
                    logic.pos_vel.x = logic.pos_vel.x * 0.4;
                    logic.pos_vel.y = -(logic.pos_vel.y * 0.4);
                    logic.pos_vel.z = logic.pos_vel.z * 0.4;
                } else {
                    logic.pos_vel.y = 0.0;
                }
                logic.pos_vel.y
            };

            if y_pos_vel < 100.0 {
                pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
                if !on_island {
                    {
                        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
                        core.falling = false;
                        core.on_island = true;
                        core.invalid_sacrifice = false;
                    }
                    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                    world.obj_mut::<PygmyPygmyBehaviourItems>(this).island_floor_collision = true;
                }
                pg_engine::display::object_mut(world, display).rot.z = 0.0;
            } else {
                let behavior = if killed_by_plague { "PygmyPlagueBodyBounce" } else { "PygmyCookedBounce" };
                pg_engine::logic::set_behavior(world, this, Some(behavior));
            }
        }

        let island_location_state =
            listener.map_or(0, |listener| crate::controller::island_location_state(world, listener));
        let y_pos = if island_location_state == 8 {
            let on_fail_boat = world.obj::<PygmyPygmyCoreFrame>(this).on_fail_boat;
            if !on_fail_boat {
                world.obj_mut::<PygmyPygmyCoreFrame>(this).on_fail_boat = true;
                let marker = pg_engine::library::display_marker(world, "FailBoatPygmyDisplayMarker");
                if let Some(marker) = marker {
                    translate_and_move_to_display_marker(world, this, marker);
                }
            }
            0.0
        } else {
            world.obj::<PygmyPygmyCoreFrame>(this).island_y_pos
        };
        pg_engine::display::object_mut(world, display).pos.y = y_pos;
        stop_playing_catch_with_monkey(world, this);
    } else {
        let tilt_state = world.obj::<PygmyPygmyCoreFrame>(this).tilt_state;
        if tilt_state != 5 {
            {
                let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
                core.on_island = false;
                core.on_fail_boat = false;
            }

            let core = world.obj::<PygmyPygmyCoreFrame>(this);
            let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);
            let ignore_gravity = world.obj::<SpriteLogic>(this).ignore_gravity;
            let held = core.falling
                || core.dragging
                || core.killed_by_locusts
                || core.killed_by_purification
                || core.dance_dance_executed
                || core.cooked
                || core.killed_by_plague
                || core.cut_in_half_by_charlie
                || ignore_gravity
                || core.underwater
                || touch_drag.the_runs
                || core.battle_of_the_gods
                || touch_drag.konkey_dong
                || core.in_closet_temple_closet_door
                || core.moron_pests
                || touch_drag.no_pee;
            if !held {
                let sun_frozen = core.sun_frozen;
                if sun_frozen {
                    ice_shatter(world, this);
                }
                let time_temple_pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
                world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
                let behavior = if time_temple_pygmy_type == 0 { "PygmyFall" } else { "TemplePygmyFall" };
                pg_engine::logic::set_behavior(world, this, Some(behavior));
            }
        } else {
            let island_y_pos = {
                let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
                core.on_island = true;
                core.on_fail_boat = false;
                core.island_y_pos
            };
            pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
            pg_engine::display::object_mut(world, display).pos.y = island_y_pos;
        }
    }

    let y_pos = pg_engine::display::object(world, display).pos.y;
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);
    let out_of_world = y_pos < -100.0
        || (820.0 < y_pos && !core.doodle_jumping && !touch_drag.konkey_dong && !touch_drag.no_pee && !core.tumble);
    if out_of_world {
        stop_being_high_on_nitrous(world, this);
        stop_eating(world, this);
        stop_carrying(world, this);
        stop_carrying_plague_body(world, this);
        stop_sleeping(world, this);
        stop_fishing(world, this);
        stop_warming_by_bon_fire(world, this);
        stop_staring_at_rainbow(world, this);
        stop_staring_at_kidnapping(world, this);
        stop_waiting_to_crap(world, this);
        stop_throwing_spear(world, this);
        close_word_balloon(world, this);
        stop_charlie_mischief(world, this);

        // Runs again every frame the pygmy stays out of bounds, restarting the sequence.
        let die_in_story = world.obj::<PygmyCharlieStory>(this).die_in_story;
        let behavior = if die_in_story {
            "PygmyStoryModeDie"
        } else {
            let konkey_dong = world.obj::<PygmyPygmyTouchDrag>(this).konkey_dong;
            let konkey_dong_hit = world.obj::<PygmyPygmyCoreFrame>(this).konkey_dong_hit;
            if konkey_dong && !konkey_dong_hit {
                world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("KonkeyDongFall".to_owned());
            }

            let core = world.obj::<PygmyPygmyCoreFrame>(this);
            if core.killed_by_plague && !core.killed_by_plague_disappear {
                "PygmyPlagueBodyFallOffWorld"
            } else {
                "PygmyDie"
            }
        };
        pg_engine::logic::set_behavior(world, this, Some(behavior));
    }

    let ash_pile_display_object = world.obj::<PygmyPygmyCoreFrame>(this).ash_pile_display_object;
    if let Some(ash_pile_display_object) = ash_pile_display_object {
        let alpha = pg_engine::display::object(world, display).alpha;
        pg_engine::display::object_mut(world, ash_pile_display_object).set_alpha(alpha);
    }

    check_pygmy_collision(world, this);

    // The original passes `&mLastPos` (a `VECTOR4 *`) to these checks, which only read it.
    let last_pos = world.obj::<PygmyPygmyCoreFrame>(this).last_pos;
    check_doodle_cloud_collision(world, this, last_pos);
    check_pain_drain_collision(world, this, last_pos);
    check_the_runs_collision(world, this, last_pos);
    check_konkey_dong_collision(world, this, last_pos);
    check_moron_pests_collision(world, this, last_pos);

    let word_balloon_logic = world.obj::<PygmyCharlieStory>(this).word_balloon_logic;
    if let Some(word_balloon_logic) = word_balloon_logic {
        let word_balloon_display = pg_engine::logic::display_object(world, word_balloon_logic);
        let pos = pg_engine::display::object(world, display).pos;
        pg_engine::display::object_mut(world, word_balloon_display).pos = pos;
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let disable_labels = core.disable_labels;
    let name_text = core.name_text;
    let name_background = core.name_background;
    let name_text_group = name_text.and_then(|name_text| pg_engine::display::object(world, name_text).display_group);

    let show_labels = !disable_labels
        && name_text_group.is_none()
        && listener.is_some_and(|listener| crate::controller::are_name_labels_enabled(world, listener));
    if show_labels {
        update_label_layer(world, this);
    } else {
        let labels_wanted = !disable_labels
            && listener.is_some_and(|listener| crate::controller::are_name_labels_enabled(world, listener));
        if !labels_wanted && name_text_group.is_some() {
            if let Some(name_background) = name_background {
                pg_engine::display::remove_from_display_group(world, name_background);
            }
            if let Some(name_text) = name_text {
                pg_engine::display::remove_from_display_group(world, name_text);
            }
        }
    }

    let apocalypse_cloud_logic = world.obj::<SpriteLogic>(this).apocalypse_cloud_logic;
    if let Some(apocalypse_cloud_logic) = apocalypse_cloud_logic {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let drag_label_x_offset = core.drag_label_x_offset;
        let drag_label_y_offset = core.drag_label_y_offset;
        crate::logic::apocalypse_temples::apocalypse_cloud_logic::set_drag_x_offset(
            world,
            apocalypse_cloud_logic,
            drag_label_x_offset,
            drag_label_y_offset,
        );
    }

    move_name_label(world, this);

    let pos = pg_engine::display::object(world, display).pos;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).last_pos = pos;
}

/// `-[PygmyLogic moveNameLabel]` @ 0x10012b080
pub fn move_name_label(world: &mut World, this: Entity) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let (Some(name_background), Some(name_text)) = (core.name_background, core.name_text) else {
        return;
    };
    let drag_label_x_offset = core.drag_label_x_offset;
    let drag_label_y_offset = core.drag_label_y_offset;
    let name_text_x_offset = core.name_text_x_offset;
    let name_text_y_offset = core.name_text_y_offset;

    let apocalypse = world.obj::<PygmyApocalypse>(this);
    let plague_body_picked_up = apocalypse.plague_body_picked_up;
    let plague_body_carrying_pygmy_logic = apocalypse.plague_body_carrying_pygmy_logic;

    let display = pg_engine::logic::display_object(world, this);
    let mut background_pos = pg_engine::display::object(world, display).pos;
    if plague_body_picked_up {
        if let Some(carrying_pygmy_logic) = plague_body_carrying_pygmy_logic {
            let carrying_display = pg_engine::logic::display_object(world, carrying_pygmy_logic);
            background_pos = pg_engine::display::object(world, carrying_display).pos;
            background_pos.x = background_pos.x + 0.0;
            background_pos.y = background_pos.y + 15.0;
        }
    }
    background_pos.x = drag_label_x_offset + background_pos.x;
    background_pos.y = drag_label_y_offset + background_pos.y;
    pg_engine::display::object_mut(world, name_background).pos = background_pos;

    let mut text_pos = background_pos;
    text_pos.x = name_text_x_offset + text_pos.x;
    text_pos.y = name_text_y_offset + text_pos.y;
    pg_engine::display::object_mut(world, name_text).pos = text_pos;
}

/// `-[PygmyLogic checkForEarthquake:]` @ 0x10012b1d0
pub fn check_for_earthquake(world: &mut World, this: Entity, dt: f32) {
    let raw_accel_force = world.resource::<BcView>().raw_accel_force;
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;

    let has_accel = raw_accel_force.x != 0.0 && raw_accel_force.y != 0.0 && raw_accel_force.z != 0.0;
    let gravity_enabled =
        has_accel && listener.is_some_and(|listener| crate::controller::is_gravity_enabled(world, listener));
    if !gravity_enabled {
        let earthquake = world.obj::<PygmyPygmyCoreFrame>(this).earthquake;
        if earthquake {
            stop_activities(world, this);
            world.obj_mut::<PygmyPygmyCoreFrame>(this).earthquake = false;
            if let Some(listener) = listener {
                crate::controller::on_pygmy_earthquake_stop(world, listener, this);
            }

            let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);
            let in_minigame = touch_drag.the_runs
                || touch_drag.konkey_dong
                || world.obj::<PygmyPygmyCoreFrame>(this).moron_pests
                || touch_drag.no_pee;
            if !in_minigame {
                pg_engine::logic::set_behavior(world, this, Some("PygmyGainBalance"));
            }
        }
        return;
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);
    let busy = core.dragging
        || core.touch_began_inside
        || core.remove_from_story_mode
        || core.cut_in_half_by_charlie
        || core.time_temple_pygmy_type != 0
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.cooked
        || core.killed_by_plague
        || core.swarmed
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
        || core.resting_in_igloo
        || core.crapping
        || core.trapped_in_spider_web
        || core.trapped_in_zombie_grave
        || core.trapped_in_idol_pedestal
        || core.inside_temple_door
        || core.inside_apocalypse_temple_door
        || core.on_top_apocalypse_time_temple_platform
        || core.eaten_by_locust_queen
        || core.in_apocalypse_temple_fountain
        || core.thetan_brawl
        || core.on_closet_temple_thetan_meter
        || core.on_closet_temple_couch
        || core.in_closet_temple_closet_door
        || core.slamming_closet_temple_closet_door
        || core.held_by_xenu
        || core.held_in_toms_arms
        || core.holding_baby_pygmy
        || core.thrown_by_locusts
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.battle_of_the_gods
        || core.zombie_explode
        || core.is_ghost
        || core.touched_by_ghost
        || core.possessed_by_thetan
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.skewered
        || core.underwater
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.burnt_by_magnifying_glass
        || core.shark_on_head
        || core.catch_shark
        || core.crushed
        || core.caught_by_bird
        || sprite.blown_by_hurricane
        || sprite.picked_up_by_hurricane
        || core.dropped_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.drowning
        || core.in_crack_hole
        || core.doodle_jumping
        || core.falling
        || core.tumble;
    if busy {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).shake_count = 0;
        return;
    }

    let total_raw_accel_force = world.resource::<BcView>().total_raw_accel_force;
    let (shake_count, last_shake_time, earthquake) = {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.last_shake_time += dt;
        if 1.1 < total_raw_accel_force {
            if !core.shake_up {
                core.shake_count += 1;
                core.shake_up = true;
                core.last_shake_time = 0.0;
            }
        } else if total_raw_accel_force < 0.9 && core.shake_up {
            core.shake_count += 1;
            core.shake_up = false;
            core.last_shake_time = 0.0;
        }
        (core.shake_count, core.last_shake_time, core.earthquake)
    };

    let mut shake_count = shake_count;
    if 0.5 < last_shake_time {
        shake_count = 0;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).shake_count = 0;
        if earthquake {
            stop_activities(world, this);
            world.obj_mut::<PygmyPygmyCoreFrame>(this).earthquake = false;
            if let Some(listener) = listener {
                crate::controller::on_pygmy_earthquake_stop(world, listener, this);
            }
            pg_engine::logic::set_behavior(world, this, Some("PygmyGainBalance"));
        }
    }
    if shake_count < 5 {
        return;
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    if core.is_zombie {
        zombie_explode(world, this);
        if let Some(listener) = listener {
            crate::controller::on_pygmy_earthquake_start(world, listener, this);
            crate::controller::on_pygmy_earthquake_stop(world, listener, this);
        }
        return;
    }
    if core.earthquake {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).earthquake = true;
    let display = pg_engine::logic::display_object(world, this);
    let tilt_state = world.obj::<PygmyPygmyCoreFrame>(this).tilt_state;
    let behavior = if tilt_state == 5 {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
        let mut display_object = pg_engine::display::object_mut(world, display);
        display_object.rot.z = -180.0;
        let x_offset = if 0.0 < display_object.scale.x { -32.0 } else { 32.0 };
        display_object.pos.x = display_object.pos.x + x_offset;
        display_object.pos.y = display_object.pos.y + 32.0;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.tilt_state = 0;
        core.sacrifice_id = Some("LoseGripUpsideDownFromEarthquake".to_owned());
        "PygmyFallUpsideDown"
    } else {
        stop_activities(world, this);
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        pg_engine::display::object_mut(world, display).rot.z = 0.0;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
        let random = world.resource_mut::<Random>().arc4random();
        if random & 1 == 0 { "PygmyLoseBalance1" } else { "PygmyLoseBalance2" }
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    if let Some(listener) = listener {
        crate::controller::on_pygmy_earthquake_start(world, listener, this);
    }
}

/// `-[PygmyLogic refreshTiltState]` @ 0x10012be50
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    if !can_change_tilt_state(world, this) {
        return;
    }

    let display = pg_engine::logic::display_object(world, this);
    let raw_z_rot = world.resource::<BcView>().raw_accel_rot_degrees.z;
    let new_state = if raw_z_rot < -155.0 || 155.0 < raw_z_rot {
        0
    } else if 130.0 < raw_z_rot {
        2
    } else if raw_z_rot < -130.0 {
        1
    } else if 90.0 < raw_z_rot {
        4
    } else if raw_z_rot < -90.0 {
        3
    } else {
        5
    };

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let time_temple_pygmy_type = core.time_temple_pygmy_type;
    let temple_old_or_baby = matches!(time_temple_pygmy_type, 1 | 2);
    let old_state = core.tilt_state;

    if new_state != old_state {
        if old_state == 5 {
            pg_engine::display::object_mut(world, display).rot.z = 0.0;
        }

        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let is_zombie = core.is_zombie;
        let cooked = core.cooked;
        let killed_by_plague = core.killed_by_plague;
        let cut_in_half_by_charlie = core.cut_in_half_by_charlie;
        let sleeping = core.sleeping;
        let enable_upside_down = core.enable_upside_down;

        match new_state {
            0 => {
                world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_upside_down = false;
                if is_zombie {
                    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                    pg_engine::logic::set_behavior(world, this, Some("PygmyZombieNoTiltSlide"));
                } else if cooked || killed_by_plague || cut_in_half_by_charlie || temple_old_or_baby {
                    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                } else if sleeping {
                    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                    pg_engine::logic::set_behavior(world, this, Some("PygmyAsleep"));
                } else {
                    match old_state {
                        1 | 2 => {
                            stop_activities(world, this);
                            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                            pg_engine::logic::set_behavior(world, this, Some("PygmyGainBalanceAfterTilt"));
                        }
                        3 => {
                            stop_activities(world, this);
                            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                            let x_scale = pg_engine::display::object(world, display).scale.x;
                            let behavior = if x_scale < 0.0 { "PygmyPickupBackward" } else { "PygmyPickupForward" };
                            pg_engine::logic::set_behavior(world, this, Some(behavior));
                        }
                        4 => {
                            stop_activities(world, this);
                            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                            let x_scale = pg_engine::display::object(world, display).scale.x;
                            let behavior = if 0.0 < x_scale { "PygmyPickupBackward" } else { "PygmyPickupForward" };
                            pg_engine::logic::set_behavior(world, this, Some(behavior));
                        }
                        5 => {
                            stop_activities(world, this);
                            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                            pg_engine::display::object_mut(world, display).rot.z = 0.0;
                            pg_engine::logic::set_behavior(world, this, Some("PygmyPickupForward"));
                        }
                        _ => {}
                    }
                }
                world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = new_state;
            }
            1 | 2 => {
                // The tip is only stored when it starts; otherwise the test repeats next frame.
                let can_tip = old_state == 0
                    && !sleeping
                    && !cooked
                    && !killed_by_plague
                    && !cut_in_half_by_charlie
                    && !temple_old_or_baby
                    && !is_zombie;
                if can_tip {
                    stop_activities(world, this);
                    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                    let x_scale = pg_engine::display::object(world, display).scale.x;
                    let tip_backward = if new_state == 1 { x_scale < 0.0 } else { 0.0 < x_scale };
                    let behavior = if tip_backward { "PygmyTiltTipBackward" } else { "PygmyTiltTipForward" };
                    pg_engine::logic::set_behavior(world, this, Some(behavior));
                    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = new_state;
                }
            }
            3 => {
                let behavior = if is_zombie {
                    "PygmyZombieTiltSlide"
                } else if cooked {
                    "PygmyCookedTiltSlide"
                } else if killed_by_plague {
                    "PygmyPlagueBodyTiltSlide"
                } else if cut_in_half_by_charlie {
                    "HalfPygmyTiltSlide"
                } else if temple_old_or_baby {
                    "TemplePygmyTiltSlide"
                } else if sleeping {
                    stop_monkey_humping(world, this);
                    "PygmyTiltSlideSleep"
                } else if old_state == 5 {
                    stop_sleeping(world, this);
                    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                    let mut display_object = pg_engine::display::object_mut(world, display);
                    if 0.0 < display_object.scale.x {
                        display_object.pos.x = display_object.pos.x + 40.0;
                        display_object.scale.x = -1.0;
                    }
                    display_object.rot.z = 0.0;
                    "PygmyTiltUpsideDownToFallForward"
                } else {
                    stop_activities(world, this);
                    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                    let x_scale = pg_engine::display::object(world, display).scale.x;
                    if x_scale < 0.0 {
                        "PygmyTiltSlideBackward"
                    } else {
                        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_upside_down = true;
                        "PygmyTiltSlideForward"
                    }
                };
                pg_engine::logic::set_behavior(world, this, Some(behavior));
                world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = new_state;
            }
            4 => {
                let behavior = if is_zombie {
                    "PygmyZombieTiltSlide"
                } else if cooked {
                    "PygmyCookedTiltSlide"
                } else if killed_by_plague {
                    "PygmyPlagueBodyTiltSlide"
                } else if cut_in_half_by_charlie {
                    "HalfPygmyTiltSlide"
                } else if temple_old_or_baby {
                    "TemplePygmyTiltSlide"
                } else if sleeping {
                    stop_monkey_humping(world, this);
                    "PygmyTiltSlideSleep"
                } else {
                    stop_activities(world, this);
                    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                    let x_scale = pg_engine::display::object(world, display).scale.x;
                    if old_state == 5 {
                        let mut display_object = pg_engine::display::object_mut(world, display);
                        if x_scale < 0.0 {
                            display_object.pos.x = display_object.pos.x - 40.0;
                            display_object.scale.x = 1.0;
                        }
                        display_object.rot.z = 0.0;
                        "PygmyTiltUpsideDownToFallForward"
                    } else if 0.0 < x_scale {
                        "PygmyTiltSlideBackward"
                    } else {
                        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_upside_down = true;
                        "PygmyTiltSlideForward"
                    }
                };
                pg_engine::logic::set_behavior(world, this, Some(behavior));
                world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = new_state;
            }
            5 => {
                // Only a pygmy that slid forward (and gripped) can hang.
                if enable_upside_down && time_temple_pygmy_type == 0 {
                    stop_activities(world, this);
                    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = new_state;
                    pg_engine::logic::set_behavior(world, this, Some("PygmyHangUpsideDown"));
                }
            }
            _ => {}
        }
    }

    let z_rot = world.resource::<BcView>().accel_rot_degrees.z;
    let upside_down = -90.0 < z_rot && z_rot < 90.0;
    let enable_upside_down = world.obj::<PygmyPygmyCoreFrame>(this).enable_upside_down;
    if !upside_down || enable_upside_down {
        return;
    }

    stop_activities(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let behavior = if core.is_zombie {
        "PygmyZombieFall"
    } else if core.cooked {
        "PygmyCookedFall"
    } else if core.killed_by_plague {
        "PygmyPlagueBodyFall"
    } else if core.cut_in_half_by_charlie {
        "HalfPygmyFall"
    } else if !temple_old_or_baby {
        let time_temple_pygmy_type = core.time_temple_pygmy_type;
        let tilt_state = core.tilt_state;
        {
            let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
            core.invalid_sacrifice = true;
            core.sacrifice_id = Some("FallUpsideDown".to_owned());
        }
        if time_temple_pygmy_type != 0 {
            "TemplePygmyFall"
        } else if matches!(tilt_state, 3 | 4) {
            "PygmyFallFromBack"
        } else {
            "PygmyFallFar"
        }
    } else {
        "TemplePygmyFall"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic undoTiltState]` @ 0x10012d8c4
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    if !can_change_tilt_state(world, this) {
        return;
    }

    let display = pg_engine::logic::display_object(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let time_temple_pygmy_type = core.time_temple_pygmy_type;
    let tilt_state = core.tilt_state;
    let unbalanced_body = core.is_zombie
        || core.cooked
        || core.killed_by_plague
        || core.cut_in_half_by_charlie
        || matches!(time_temple_pygmy_type, 1 | 2);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_upside_down = false;

    if unbalanced_body {
        if tilt_state != 0 {
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        }
    } else {
        match tilt_state {
            1 | 2 => {
                stop_activities(world, this);
                pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                pg_engine::logic::set_behavior(world, this, Some("PygmyGainBalanceAfterTilt"));
            }
            3 => {
                stop_activities(world, this);
                pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                let x_scale = pg_engine::display::object(world, display).scale.x;
                let behavior = if x_scale < 0.0 { "PygmyPickupBackward" } else { "PygmyPickupForward" };
                pg_engine::logic::set_behavior(world, this, Some(behavior));
            }
            4 => {
                stop_activities(world, this);
                pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                let x_scale = pg_engine::display::object(world, display).scale.x;
                let behavior = if 0.0 < x_scale { "PygmyPickupBackward" } else { "PygmyPickupForward" };
                pg_engine::logic::set_behavior(world, this, Some(behavior));
            }
            5 => {
                stop_activities(world, this);
                pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                pg_engine::display::object_mut(world, display).rot.z = 0.0;
                pg_engine::logic::set_behavior(world, this, Some("PygmyPickupForward"));
            }
            _ => {}
        }
    }
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
}

/// `-[PygmyLogic onDeactivateIslandLocationState:newIslandLocationState:]` @ 0x10012e420
pub fn on_deactivate_island_location_state(world: &mut World, this: Entity, old_state: i32, new_state: i32) {
    let carrying_plague_body_logic = world.obj::<PygmyApocalypse>(this).carrying_plague_body_logic;
    if carrying_plague_body_logic.is_some() {
        stop_carrying_plague_body(world, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    }

    let held_in_toms_arms = world.obj::<PygmyPygmyCoreFrame>(this).held_in_toms_arms;
    if held_in_toms_arms {
        stop_being_held_in_toms_arms(world, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    }

    let holding_baby_pygmy = world.obj::<PygmyPygmyCoreFrame>(this).holding_baby_pygmy;
    if holding_baby_pygmy {
        stop_holding_baby_pygmy(world, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    }

    let sacrificed_to_xenu = world.obj::<PygmyApocalypse>(this).sacrificed_to_xenu;
    if sacrificed_to_xenu {
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
        return;
    }

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let between_underwater_locations = (old_state == 2 && new_state == 9) || (old_state == 9 && new_state == 2);

    let behavior = if between_underwater_locations {
        if !core.underwater {
            return;
        }
        let underwater_nopee = world.obj::<PygmyUnderwaterNopee>(this);
        let hurt = core.eaten_by_piranha
            || core.skewered
            || underwater_nopee.hit_by_jellyfish
            || underwater_nopee.hit_by_underwater_power
            || underwater_nopee.eaten_by_laser_shark
            || underwater_nopee.hit_by_shark_laser;
        if hurt {
            "PygmyDie"
        } else {
            stop_holding_chum(world, this);
            stop_being_eaten_by_piranha(world, this);
            let display = pg_engine::logic::display_object(world, this);
            {
                let mut display_object = pg_engine::display::object_mut(world, display);
                display_object.is_selectable = true;
                display_object.set_alpha(1.0);
                display_object.rot.z = 0.0;
            }
            let picked_up_by_whirl_pool = world.obj::<SpriteLogic>(this).picked_up_by_whirl_pool;
            if picked_up_by_whirl_pool {
                return;
            }
            "PygmyUnderwaterIdle"
        }
    } else if new_state == 5 && core.drowning {
        if core.killed_by_plague && !core.killed_by_plague_disappear {
            "PygmyPlagueBodyFallOffWorld"
        } else {
            "PygmyDie"
        }
    } else if core.eaten_by_moron_snake {
        let moron_snake_logic =
            listener.and_then(|listener| crate::controller::moron_snake_logic(world, listener, this));
        if let Some(moron_snake_logic) = moron_snake_logic {
            crate::logic::moronpests_chase::moron_snake_logic::eat_pygmy_complete(world, moron_snake_logic, this);
        }
        "PygmyDie"
    } else if core.wearing_water_wings {
        let water_wings_logic =
            listener.and_then(|listener| crate::controller::water_wings_logic(world, listener, this));
        if let Some(water_wings_logic) = water_wings_logic {
            crate::logic::nopee::water_wings_logic::wearing_water_wings_complete(world, water_wings_logic, this);
        }
        "PygmyDie"
    } else if core.monkey_on_back || core.swarmed {
        "PygmyDie"
    } else if core.eaten_by_locust_queen {
        let locust_queen_logic = world.obj::<PygmyContainersAndCatchers>(this).locust_queen_logic;
        if let Some(locust_queen_logic) = locust_queen_logic {
            crate::logic::apocalypse_locust_closet_xenu::locust_queen_logic::rescued_from_locust_queen(
                world,
                locust_queen_logic,
                this,
            );
            world.obj_mut::<PygmyContainersAndCatchers>(this).locust_queen_logic = None;
        }
        "PygmyDie"
    } else if core.high_on_nitrous {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).high_on_nitrous = false;
        pg_engine::sound::stop_sound(world, "DavidIDidntFeel");
        pg_engine::sound::stop_sound(world, "DavidIsThisRealLife");
        pg_engine::sound::stop_sound(world, "DavidTwoFingers");
        pg_engine::sound::stop_sound(world, "DavidIFeelFunny");
        pg_engine::sound::stop_sound(world, "DavidIsThisGonnaBeForever");
        pg_engine::sound::stop_sound(world, "DavidScream");
        "PygmyIdle"
    } else if new_state == 5 && core.falling && !core.doodle_knocked_down {
        "PygmyFall"
    } else {
        return;
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic hangUpsideDown:::]` @ 0x10012eab4
pub fn seq_hang_upside_down(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let raw_z_rot = world.resource::<BcView>().raw_accel_rot_degrees.z;
    pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;

    let island_y_pos = world.obj::<PygmyPygmyCoreFrame>(this).island_y_pos;
    let display = pg_engine::logic::display_object(world, this);
    let mut display_object = pg_engine::display::object_mut(world, display);
    display_object.rot.z = raw_z_rot;
    display_object.pos.y = island_y_pos;

    ItemResult::Pending
}

/// `-[PygmyLogic (Apocalypse) stopBeingHighOnNitrous]` @ 0x10003a3e4
pub fn stop_being_high_on_nitrous(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).high_on_nitrous = false;
}

/// `-[PygmyLogic (Apocalypse) setTimeTemplePygmyType:]` @ 0x10003ca9c
pub fn set_time_temple_pygmy_type(world: &mut World, this: Entity, time_temple_pygmy_type: i32) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type = time_temple_pygmy_type;
    let name = world.obj::<PygmyPygmyCoreFrame>(this).name.clone();
    set_pygmy_name_label(world, this, name.as_deref());
}

/// `-[PygmyLogic (Apocalypse) timeTemplePygmyType]` @ 0x10003cacc
pub fn time_temple_pygmy_type(world: &mut World, this: Entity) -> i32 {
    world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type
}
