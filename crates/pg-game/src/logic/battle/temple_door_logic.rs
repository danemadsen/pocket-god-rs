//! `TempleDoorLogic` (superclass `SpriteLogic`).
//!
//! The temple of God island and the host of the Battle of the Gods: the god idols on the
//! pedestals open its door step by step, and the pygmy that enters fights an enemy pygmy in the
//! battle arena, driven by a second sequence (`mBattleSequence`).

use crate::prelude::*;

use crate::logic::idol_screen::idol_screen_logic;
use crate::logic::island_core::sprite_logic;
use crate::logic_table;
use crate::managers::game_ui_manager;
use crate::managers::idol_manager::{self, IdolManager, episode_table};
use crate::persona::persona::Persona;
use pg_engine::animation;
use pg_engine::display;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::logic::multi_model;
use pg_engine::sequence;

/// `TempleDoorLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct TempleDoorLogic {
    /// `mTempleDoorLogicListener`: `@"<TempleDoorLogicListener>"`
    pub temple_door_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mDoorOpenLevel`: `i`
    pub door_open_level: i32,

    /// `mbJumpIntoTempleDoor`: `B`
    pub jump_into_temple_door: bool,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,

    /// `mbPygmyReady`: `B`
    pub pygmy_ready: bool,

    /// `mbEnemyPygmyReady`: `B`
    pub enemy_pygmy_ready: bool,

    /// `mbIdolsReady`: `B`
    pub idols_ready: bool,

    /// `mbPedestalHasIdol`: `B`
    pub pedestal_has_idol: bool,

    /// `mEnemyPygmyLogic`: `@"PygmyLogic"`
    pub enemy_pygmy_logic: Option<Entity>,

    /// `mBattleSequence`: `@"BCSequence"`
    pub battle_sequence: Option<Entity>,

    /// `mBattlePedestalRightLogic`: `@"MultiModelLogic"`
    pub battle_pedestal_right_logic: Option<Entity>,

    /// `mBattlePedestalLeftLogic`: `@"MultiModelLogic"`
    pub battle_pedestal_left_logic: Option<Entity>,

    /// `mBattlePedestalRightIdolLogic`: `@"MultiModelLogic"`
    pub battle_pedestal_right_idol_logic: Option<Entity>,

    /// `mBattlePedestalLeftIdolLogic`: `@"MultiModelLogic"`
    pub battle_pedestal_left_idol_logic: Option<Entity>,
}

pub const TEMPLE_DOOR_LOGIC: Class = Class {
    name: "TempleDoorLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    on_multi_model_event,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The animation channel the pygmy inside the temple is bound to.
const PYGMY_CHANNEL: usize = 1;

/// `isAvailable`: the door admits a pygmy once the four idols have opened it.
const DOOR_FULLY_OPEN: i32 = 4;

/// Sequence-item handlers implemented by `TempleDoorLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "onPygmyFallInsideComplete" => seq_on_pygmy_fall_inside_complete(world, this, call),
        "showBattleIdols" => seq_show_battle_idols(world, this, call),
        "startPygmyPowerUp" => seq_start_pygmy_power_up(world, this, call),
        "startPygmyAttacks" => seq_start_pygmy_attacks(world, this, call),
        "showBattleRealmResults" => seq_show_battle_realm_results(world, this, call),
        "showBattleRealmResultsOnIdol" => seq_show_battle_realm_results_on_idol(world, this, call),
        "showBattleRealmEffectsResults" => seq_show_battle_realm_effects_results(world, this, call),
        "waitForFatalityComplete" => seq_wait_for_fatality_complete(world, this, call),
        "waitForBattleResultsComplete" => seq_wait_for_battle_results_complete(world, this, call),
        "onBattleTurnComplete" => seq_on_battle_turn_complete(world, this, call),
        "checkForFatality" => seq_check_for_fatality(world, this, call),
        "performResurrection" => seq_perform_resurrection(world, this, call),
        "showBattleSelectionPanel" => seq_show_battle_selection_panel(world, this, call),
        "onGameOver" => seq_on_game_over(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[TempleDoorLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&TEMPLE_DOOR_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// The listener (the view controller).
fn door_listener(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<TempleDoorLogic>(this).temple_door_logic_listener
}

/// `mPygmyLogic` while a battle starts: the pygmy that entered the temple, which the original
/// also hands to the enemy and the listener.
fn battle_pygmy(world: &World, this: Entity) -> Entity {
    let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
    pygmy.expect("TempleDoorLogic: a battle only starts with a pygmy inside the temple")
}

/// `mEnemyPygmyLogic` while a battle starts: `createPygmyLogic` always creates one.
fn battle_enemy_pygmy(world: &World, this: Entity) -> Entity {
    let enemy = world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic;
    enemy.expect("TempleDoorLogic: createPygmyLogic always creates the enemy pygmy")
}

/// `[target setBehavior:behavior]` on a logic that may be nil.
fn set_logic_behavior(world: &mut World, target: Option<Entity>, behavior: &str) {
    if let Some(target) = target {
        logic::set_behavior(world, target, Some(behavior));
    }
}

/// `[pygmy wasKilledDuringBattle]` (NO for nil).
fn was_killed_during_battle(world: &mut World, pygmy: Option<Entity>) -> bool {
    match pygmy {
        Some(pygmy) => crate::pygmy::was_killed_during_battle(world, pygmy),
        None => false,
    }
}

/// `[[IdolManager instance] activeBattlePlayerEpisodeIdol]` and `…EnemyEpisodeIdol`, in that
/// order: the idols fighting this turn, as `EpisodeTable` nodes.
fn active_battle_episode_idols(world: &mut World) -> (Option<usize>, Option<usize>) {
    let Some(idol_manager) = idol_manager::instance(world) else {
        return (None, None);
    };

    let player_idol = idol_manager::active_battle_player_episode_idol(world, idol_manager);
    let enemy_idol = idol_manager::active_battle_enemy_episode_idol(world, idol_manager);
    (player_idol, enemy_idol)
}

/// `idol->selectionSlot` of an active battle idol, which the original reads through the node
/// pointer.
fn episode_selection_slot(world: &mut World, episode_idol: Option<usize>) -> i32 {
    let node = episode_idol.expect("TempleDoorLogic: the battle has an active idol on each side");
    episode_table(world).nodes[node].selection_slot
}

/// `[[IdolManager instance] playerName]`, which returns `mPlayerBattleState.mName`.
fn idol_manager_player_name(world: &World) -> Option<String> {
    let idol_manager = idol_manager::instance(world)?;
    world.obj::<IdolManager>(idol_manager).player_battle_state.name.clone()
}

/// `[[IdolManager instance] enemyName]`, which returns `mEnemyBattleState.mName`.
fn idol_manager_enemy_name(world: &World) -> Option<String> {
    let idol_manager = idol_manager::instance(world)?;
    world.obj::<IdolManager>(idol_manager).enemy_battle_state.name.clone()
}

/// `[[pygmy pygmyPersona] name]`: the persona's name ivar.
fn pygmy_persona_name(world: &mut World, pygmy: Entity) -> Option<String> {
    let persona = crate::pygmy::pygmy_persona(world, pygmy)?;
    world.obj::<Persona>(persona).name.clone()
}

/// The enemy pygmy leaves the arena (`gameOver`, `quitGame`).
fn remove_enemy_pygmy(world: &mut World, this: Entity) {
    let enemy = world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic;
    let Some(enemy) = enemy else {
        return;
    };

    crate::pygmy::remove_from_battle_of_the_gods(world, enemy);
    world.obj_mut::<TempleDoorLogic>(this).enemy_pygmy_logic = None;
}

/// Both battle pedestals drop their idols, and the idol models show the active idols: the
/// player's (`BattlePedestalIdolSlot1…4`) on the left, the enemy's (`…5…8`) on the right
/// (`showBattleIdols:::`, `onMultiModelEvent:eventId:`).
fn drop_battle_idols(world: &mut World, this: Entity) {
    let (player_idol, enemy_idol) = active_battle_episode_idols(world);

    let door = world.obj::<TempleDoorLogic>(this);
    let right_pedestal = door.battle_pedestal_right_logic;
    let left_pedestal = door.battle_pedestal_left_logic;
    set_logic_behavior(world, right_pedestal, "BattlePedestalDropIdol");
    set_logic_behavior(world, left_pedestal, "BattlePedestalDropIdol");

    let left_idol = world.obj::<TempleDoorLogic>(this).battle_pedestal_left_idol_logic;
    let player_slot = episode_selection_slot(world, player_idol);
    let left_behavior = format!("BattlePedestalIdolSlot{}", player_slot + 1);
    set_logic_behavior(world, left_idol, &left_behavior);

    let right_idol = world.obj::<TempleDoorLogic>(this).battle_pedestal_right_idol_logic;
    let enemy_slot = episode_selection_slot(world, enemy_idol);
    let right_behavior = format!("BattlePedestalIdolSlot{}", enemy_slot + 5);
    set_logic_behavior(world, right_idol, &right_behavior);
}

/// `-[TempleDoorLogic initWithDisplayObject:]` @ 0x1000903bc
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(TempleDoorLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().temple_door_logic = Some(this);

    let right_idol = library::logic(world, logic_table::BATTLE_PEDESTAL_RIGHT_IDOL);
    world.obj_mut::<TempleDoorLogic>(this).battle_pedestal_right_idol_logic = Some(right_idol);

    let left_idol = library::logic(world, logic_table::BATTLE_PEDESTAL_LEFT_IDOL);
    world.obj_mut::<TempleDoorLogic>(this).battle_pedestal_left_idol_logic = Some(left_idol);

    let right_pedestal = library::logic(world, logic_table::BATTLE_PEDESTAL_RIGHT);
    world.obj_mut::<TempleDoorLogic>(this).battle_pedestal_right_logic = Some(right_pedestal);
    multi_model::set_multi_model_logic_listener(world, right_pedestal, Some(this));

    let left_pedestal = library::logic(world, logic_table::BATTLE_PEDESTAL_LEFT);
    world.obj_mut::<TempleDoorLogic>(this).battle_pedestal_left_logic = Some(left_pedestal);
    multi_model::set_multi_model_logic_listener(world, left_pedestal, Some(this));

    let battle_sequence = sequence::spawn(world, None, Some(this));
    world.obj_mut::<TempleDoorLogic>(this).battle_sequence = Some(battle_sequence);
}

/// `-[TempleDoorLogic setBattleBehavior:]` @ 0x100090530
///
/// Runs the battle script `battle_behavior` on `mBattleSequence`.
pub fn set_battle_behavior(world: &mut World, this: Entity, battle_behavior: &str) {
    let battle_sequence = world.obj::<TempleDoorLogic>(this).battle_sequence;
    let def = library::sequence_def(world, battle_behavior);
    if let Some(battle_sequence) = battle_sequence {
        sequence::set_sequence_def(world, battle_sequence, def);
    }
}

/// `-[TempleDoorLogic setTempleDoorLogicListener:]` @ 0x100090598
pub fn set_temple_door_logic_listener(world: &mut World, this: Entity, temple_door_logic_listener: Entity) {
    world.obj_mut::<TempleDoorLogic>(this).temple_door_logic_listener = Some(temple_door_logic_listener);
}

/// `-[TempleDoorLogic isAvailable]` @ 0x1000905f4
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let door = world.obj::<TempleDoorLogic>(this);
    if door.pygmy_logic.is_some() {
        return false;
    }
    door.door_open_level == DOOR_FULLY_OPEN
}

/// `-[TempleDoorLogic stopInteractingWithObjects]` @ 0x100090624
pub fn stop_interacting_with_objects(_world: &mut World, _this: Entity) {}

/// `-[TempleDoorLogic fallIntoTempleDoor:]` @ 0x100090628
pub fn fall_into_temple_door(world: &mut World, this: Entity, pygmy_logic: Entity) {
    {
        let mut door = world.obj_mut::<TempleDoorLogic>(this);
        door.pygmy_logic = Some(pygmy_logic);
        door.jump_into_temple_door = false;
    }

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("TempleDoorPygmyFall"));
}

/// `-[TempleDoorLogic jumpIntoTempleDoor:]` @ 0x1000906b4
pub fn jump_into_temple_door(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let mut door = world.obj_mut::<TempleDoorLogic>(this);
    door.pygmy_logic = Some(pygmy_logic);
    door.jump_into_temple_door = true;
}

/// `-[TempleDoorLogic jumpIntoTempleDoorComplete:]` @ 0x1000906d4
pub fn jump_into_temple_door_complete(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    let animation = logic::animation(world, this);

    // `[nil displayObject]` binds nil.
    let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
    let pygmy_display_object = match pygmy {
        Some(pygmy) => Some(logic::display_object(world, pygmy)),
        None => None,
    };
    animation::set_tweenable(world, animation, pygmy_display_object, PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("TempleDoorPygmyJump"));
}

/// `-[TempleDoorLogic ejectFromTempleDoor:]` @ 0x100090750
pub fn eject_from_temple_door(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    let animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("TempleDoorEjectPygmy"));
    world.obj_mut::<TempleDoorLogic>(this).pygmy_logic = None;
}

/// `-[TempleDoorLogic enterIsland:]` @ 0x1000907b8
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let active = world.obj::<TempleDoorLogic>(this).active;
    if active {
        return;
    }

    {
        let mut door = world.obj_mut::<TempleDoorLogic>(this);
        door.active = true;
        door.door_open_level = 0;
    }
    logic::set_behavior(world, this, Some("TempleDoorEnterIsland"));
}

/// `-[TempleDoorLogic exitIsland:]` @ 0x1000907f8
pub fn exit_island(world: &mut World, this: Entity, _now: bool) {
    let active = world.obj::<TempleDoorLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<TempleDoorLogic>(this).active = false;

    let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        let animation = logic::animation(world, this);
        animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::eject_from_temple_door(world, pygmy);
        world.obj_mut::<TempleDoorLogic>(this).pygmy_logic = None;
    }

    stop_interacting_with_objects(world, this);
    logic::set_behavior(world, this, Some("TempleDoorExitIsland"));
}

/// `-[TempleDoorLogic exitIslandNow]` @ 0x1000908ac
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[TempleDoorLogic openDoor]` @ 0x1000908c0
///
/// One idol more on the pedestals: the door opens one step (`TempleDoorOpen1…4`).
pub fn open_door(world: &mut World, this: Entity) {
    let level = world.obj::<TempleDoorLogic>(this).door_open_level;
    world.obj_mut::<TempleDoorLogic>(this).door_open_level = level.wrapping_add(1);

    let behavior = match level {
        0 => "TempleDoorOpen1",
        1 => "TempleDoorOpen2",
        2 => "TempleDoorOpen3",
        3 => "TempleDoorOpen4",
        _ => return,
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[TempleDoorLogic closeDoor]` @ 0x100090964
///
/// One idol fewer: a pygmy inside is ejected and the door closes one step (`TempleDoorClose4…1`).
pub fn close_door(world: &mut World, this: Entity) {
    let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        let animation = logic::animation(world, this);
        animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::eject_from_temple_door(world, pygmy);
        world.obj_mut::<TempleDoorLogic>(this).pygmy_logic = None;
    }

    let level = world.obj::<TempleDoorLogic>(this).door_open_level;
    let behavior = match level {
        1 => Some("TempleDoorClose1"),
        2 => Some("TempleDoorClose2"),
        3 => Some("TempleDoorClose3"),
        4 => Some("TempleDoorClose4"),
        _ => None,
    };
    if let Some(behavior) = behavior {
        logic::set_behavior(world, this, Some(behavior));
    }

    // Original bug, kept: the level also drops when no step was open, so it can go below 0.
    let level = world.obj::<TempleDoorLogic>(this).door_open_level;
    world.obj_mut::<TempleDoorLogic>(this).door_open_level = level.wrapping_sub(1);
}

/// `-[TempleDoorLogic onGameFrame:]` @ 0x100090af4
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[TempleDoorLogic onPygmyFallInsideComplete:::]` @ 0x100090af8
///
/// The pygmy is inside: the enemy selection screen opens.
pub fn seq_on_pygmy_fall_inside_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(listener) = door_listener(world, this) {
        // The original asks for the location state and ignores the answer.
        crate::controller::island_location_state(world, listener);
        crate::controller::on_open_idol_battle_enemy_screen(world, listener);
    }
    ItemResult::Done
}

/// `-[TempleDoorLogic isBattleOfTheGodsActive]` @ 0x100090b48
pub fn is_battle_of_the_gods_active(world: &mut World, this: Entity) -> bool {
    world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic.is_some()
}

/// `-[TempleDoorLogic startBattle]` @ 0x100090b60
///
/// Creates the enemy pygmy, starts both fighters, resets the battle pedestals and runs
/// `BattleArenaStartBattle`.
pub fn start_battle(world: &mut World, this: Entity) {
    {
        let mut door = world.obj_mut::<TempleDoorLogic>(this);
        door.pygmy_ready = false;
        door.enemy_pygmy_ready = false;
    }

    let listener = door_listener(world, this);
    let created_enemy = match listener {
        Some(listener) => crate::controller::create_pygmy_logic(world, listener),
        None => None,
    };
    world.obj_mut::<TempleDoorLogic>(this).enemy_pygmy_logic = created_enemy;
    let enemy = battle_enemy_pygmy(world, this);

    let enemy_name = idol_manager_enemy_name(world);
    crate::pygmy::set_pygmy_name_label(world, enemy, enemy_name.as_deref());

    let enemy = battle_enemy_pygmy(world, this);
    let pygmy = battle_pygmy(world, this);
    crate::pygmy::start_battle_of_the_gods_enemy(world, enemy, pygmy);

    let marker = library::display_marker(world, "BattleOfTheGodsPygmyDisplayMarker");
    let enemy = battle_enemy_pygmy(world, this);
    let enemy_display_object = logic::display_object(world, enemy);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(enemy_display_object));
    }

    let enemy = battle_enemy_pygmy(world, this);
    crate::pygmy::update_label_layer(world, enemy);

    let animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

    if let Some(listener) = listener {
        let pygmy = battle_pygmy(world, this);
        crate::controller::on_battle_of_the_gods_start(world, listener, this, pygmy);
    }

    world.obj_mut::<TempleDoorLogic>(this).pedestal_has_idol = false;
    let door = world.obj::<TempleDoorLogic>(this);
    let right_pedestal = door.battle_pedestal_right_logic;
    let left_pedestal = door.battle_pedestal_left_logic;
    set_logic_behavior(world, right_pedestal, "BattlePedestalInit");
    set_logic_behavior(world, left_pedestal, "BattlePedestalInit");

    let pygmy = battle_pygmy(world, this);
    let player_name = idol_manager_player_name(world);
    crate::pygmy::set_pygmy_name_label(world, pygmy, player_name.as_deref());

    let pygmy = battle_pygmy(world, this);
    let enemy = battle_enemy_pygmy(world, this);
    crate::pygmy::start_battle_of_the_gods(world, pygmy, enemy);

    if let Some(listener) = listener {
        crate::controller::on_open_idol_battle_screen(world, listener);
    }
    set_battle_behavior(world, this, "BattleArenaStartBattle");
}

/// `-[TempleDoorLogic cancelBattle]` @ 0x100090db8
///
/// The pygmy gets its own name back and leaves the temple.
pub fn cancel_battle(world: &mut World, this: Entity) {
    let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
    let Some(pygmy) = pygmy else {
        return;
    };

    let animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

    let pygmy_name = pygmy_persona_name(world, pygmy);
    crate::pygmy::set_pygmy_name_label(world, pygmy, pygmy_name.as_deref());

    let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::eject_from_temple_door(world, pygmy);
    }
    world.obj_mut::<TempleDoorLogic>(this).pygmy_logic = None;
}

/// `-[TempleDoorLogic onPlayerFatality]` @ 0x100090e68
pub fn on_player_fatality(world: &mut World, this: Entity) {
    world.obj_mut::<TempleDoorLogic>(this).pygmy_ready = false;

    let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::fatality_in_battle(world, pygmy);
    }
}

/// `-[TempleDoorLogic onEnemyFatality]` @ 0x100090e90
pub fn on_enemy_fatality(world: &mut World, this: Entity) {
    world.obj_mut::<TempleDoorLogic>(this).enemy_pygmy_ready = false;

    let enemy = world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic;
    if let Some(enemy) = enemy {
        crate::pygmy::fatality_in_battle(world, enemy);
    }
}

/// `-[TempleDoorLogic onPygmyBattleReady:]` @ 0x100090eb8
///
/// A fighter finished its move; an opponent killed during the move comes back.
pub fn on_pygmy_battle_ready(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
    if pygmy == Some(pygmy_logic) {
        world.obj_mut::<TempleDoorLogic>(this).pygmy_ready = true;

        let enemy = world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic;
        let enemy_killed = was_killed_during_battle(world, enemy);
        if enemy_killed {
            world.obj_mut::<TempleDoorLogic>(this).enemy_pygmy_ready = false;

            let enemy = world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic;
            if let Some(enemy) = enemy {
                crate::pygmy::spawn_from_battle_of_the_gods(world, enemy);
            }
        }
    }

    let enemy = world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic;
    if enemy == Some(pygmy_logic) {
        world.obj_mut::<TempleDoorLogic>(this).enemy_pygmy_ready = true;

        let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
        let pygmy_killed = was_killed_during_battle(world, pygmy);
        if pygmy_killed {
            world.obj_mut::<TempleDoorLogic>(this).pygmy_ready = false;

            let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
            if let Some(pygmy) = pygmy {
                crate::pygmy::spawn_from_battle_of_the_gods(world, pygmy);
            }
        }
    }
}

/// `-[TempleDoorLogic onButtonPress:touch:]` @ 0x100090fac
pub fn on_button_press(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[TempleDoorLogic onButtonRelease:touch:]` @ 0x100090fb0
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[TempleDoorLogic onButtonReleaseOutside:touch:]` @ 0x100090fb4
pub fn on_button_release_outside(world: &mut World, this: Entity, button: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.on_button_release)(world, this, button, touch);
}

/// `-[TempleDoorLogic battlePygmyLogic]` @ 0x100090fc4
pub fn battle_pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<TempleDoorLogic>(this).pygmy_logic
}

/// `-[TempleDoorLogic battleEnemyPygmyLogic]` @ 0x100090fd4
pub fn battle_enemy_pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic
}

/// `-[TempleDoorLogic playBattleSequence]` @ 0x100090fe4
pub fn play_battle_sequence(world: &mut World, this: Entity) {
    set_battle_behavior(world, this, "BattleArenaStartNextAttack");
}

/// `-[TempleDoorLogic showBattleIdols:::]` @ 0x100090ffc
///
/// First time: clears idols already on the battle pedestals (their `…ClearIdolComplete` event
/// then drops the new ones), else drops the idols at once. Completes when they are in place.
pub fn seq_show_battle_idols(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<TempleDoorLogic>(this).idols_ready = false;

        let pedestal_has_idol = world.obj::<TempleDoorLogic>(this).pedestal_has_idol;
        if pedestal_has_idol {
            let right_pedestal = world.obj::<TempleDoorLogic>(this).battle_pedestal_right_logic;
            set_logic_behavior(world, right_pedestal, "BattlePedestalClearIdol");

            let left_pedestal = world.obj::<TempleDoorLogic>(this).battle_pedestal_left_logic;
            set_logic_behavior(world, left_pedestal, "BattlePedestalClearIdol");
        } else {
            drop_battle_idols(world, this);
        }
    }

    let idols_ready = world.obj::<TempleDoorLogic>(this).idols_ready;
    if idols_ready {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[TempleDoorLogic startPygmyPowerUp:::]` @ 0x1000911d0
pub fn seq_start_pygmy_power_up(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let (player_idol, enemy_idol) = active_battle_episode_idols(world);

        let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
        if let Some(pygmy) = pygmy {
            crate::pygmy::start_battle_power_up(world, pygmy, player_idol, enemy_idol);
        }

        let enemy = world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic;
        if let Some(enemy) = enemy {
            crate::pygmy::start_battle_power_up(world, enemy, enemy_idol, player_idol);
        }
    }
    ItemResult::Done
}

/// `-[TempleDoorLogic startPygmyAttacks:::]` @ 0x100091290
///
/// First time: both fighters attack. Meanwhile the arena pans toward a fighter killed by the
/// attack; it completes, re-centred, once both fighters are ready.
pub fn seq_start_pygmy_attacks(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut door = world.obj_mut::<TempleDoorLogic>(this);
            door.pygmy_ready = false;
            door.enemy_pygmy_ready = false;
        }

        let (player_idol, enemy_idol) = active_battle_episode_idols(world);

        let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
        if let Some(pygmy) = pygmy {
            crate::pygmy::start_battle_attack(world, pygmy, player_idol, enemy_idol);
        }

        let enemy = world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic;
        if let Some(enemy) = enemy {
            crate::pygmy::start_battle_attack(world, enemy, enemy_idol, player_idol);
        }

        if let Some(idol_screen) = idol_screen_logic::instance(world) {
            idol_screen_logic::show_battle_attack_start(world, idol_screen);
        }
    }

    // The original writes through `[[BCLibrary getDisplayGroupById:…] pos]`.
    let arena = library::display_group(world, "BattleArenaDisplayGroup");
    let arena = arena.expect("TempleDoorLogic: the library holds BattleArenaDisplayGroup");

    let enemy = world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic;
    let enemy_killed = was_killed_during_battle(world, enemy);
    let mut pan_target = 0.0;
    let mut pan_rate = 0.2;
    if enemy_killed {
        pan_target = -128.0;
        pan_rate = 0.1;
    }

    let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
    let pygmy_killed = was_killed_during_battle(world, pygmy);
    if pygmy_killed {
        pan_target = 128.0;
        pan_rate = 0.1;
    }

    let arena_x = display::object(world, arena).pos.x;
    let panned_x = arena_x + pan_rate * (pan_target - arena_x);
    display::object_mut(world, arena).pos.x = panned_x;

    let door = world.obj::<TempleDoorLogic>(this);
    let both_ready = door.pygmy_ready && door.enemy_pygmy_ready;
    if !both_ready {
        return ItemResult::Pending;
    }

    display::object_mut(world, arena).pos.x = 0.0;
    ItemResult::Done
}

/// `-[TempleDoorLogic showBattleRealmResults:::]` @ 0x100091494
pub fn seq_show_battle_realm_results(world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(idol_manager) = idol_manager::instance(world) {
        idol_manager::show_battle_realm_results(world, idol_manager);
    }
    ItemResult::Done
}

/// `-[TempleDoorLogic showBattleRealmResultsOnIdol:::]` @ 0x1000914d0
pub fn seq_show_battle_realm_results_on_idol(world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(idol_manager) = idol_manager::instance(world) {
        idol_manager::show_battle_realm_results_on_idol(world, idol_manager);
    }
    ItemResult::Done
}

/// `-[TempleDoorLogic showBattleRealmEffectsResults:::]` @ 0x10009150c
pub fn seq_show_battle_realm_effects_results(world: &mut World, _this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        if let Some(idol_manager) = idol_manager::instance(world) {
            idol_manager::show_battle_realm_effect_results(world, idol_manager);
        }
    }

    let complete = match idol_manager::instance(world) {
        Some(idol_manager) => idol_manager::are_battle_realm_effect_results_complete(world, idol_manager),
        None => false,
    };
    if complete {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[TempleDoorLogic waitForFatalityComplete:::]` @ 0x10009157c
pub fn seq_wait_for_fatality_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let door = world.obj::<TempleDoorLogic>(this);
    if door.pygmy_ready && door.enemy_pygmy_ready {
        return ItemResult::Done;
    }
    ItemResult::Pending
}

/// `-[TempleDoorLogic waitForBattleResultsComplete:::]` @ 0x1000915ac
pub fn seq_wait_for_battle_results_complete(world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let points_complete = match game_ui_manager::instance(world) {
        Some(game_ui_manager) => game_ui_manager::all_points_complete(world, game_ui_manager),
        None => false,
    };
    if points_complete {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[TempleDoorLogic onBattleTurnComplete:::]` @ 0x1000915f0
pub fn seq_on_battle_turn_complete(world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if let Some(idol_manager) = idol_manager::instance(world) {
        idol_manager::on_battle_turn_complete(world, idol_manager);
    }
    ItemResult::Done
}

/// `-[TempleDoorLogic checkForFatality:::]` @ 0x10009162c
///
/// A finished battle switches the battle script to the fatality, with a resurrection when
/// either side resurrects.
pub fn seq_check_for_fatality(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let Some(idol_manager) = idol_manager::instance(world) else {
        return ItemResult::Done;
    };

    let battle_complete = idol_manager::is_battle_complete(world, idol_manager);
    if !battle_complete {
        return ItemResult::Done;
    }

    let resurrect = idol_manager::should_player_resurrect(world, idol_manager) || idol_manager::should_enemy_resurrect(world, idol_manager);
    let behavior = if resurrect {
        "BattleArenaFatalityResurrect"
    } else {
        "BattleArenaFatality"
    };
    set_battle_behavior(world, this, behavior);
    ItemResult::Done
}

/// `-[TempleDoorLogic performResurrection:::]` @ 0x1000916f8
pub fn seq_perform_resurrection(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let enemy_resurrects = match idol_manager::instance(world) {
        Some(idol_manager) => idol_manager::should_enemy_resurrect(world, idol_manager),
        None => false,
    };
    if enemy_resurrects {
        world.obj_mut::<TempleDoorLogic>(this).enemy_pygmy_ready = false;

        let enemy = world.obj::<TempleDoorLogic>(this).enemy_pygmy_logic;
        if let Some(enemy) = enemy {
            crate::pygmy::spawn_from_battle_of_the_gods(world, enemy);
        }
    }

    let player_resurrects = match idol_manager::instance(world) {
        Some(idol_manager) => idol_manager::should_player_resurrect(world, idol_manager),
        None => false,
    };
    if player_resurrects {
        world.obj_mut::<TempleDoorLogic>(this).pygmy_ready = false;

        let pygmy = world.obj::<TempleDoorLogic>(this).pygmy_logic;
        if let Some(pygmy) = pygmy {
            crate::pygmy::spawn_from_battle_of_the_gods(world, pygmy);
        }
    }

    if let Some(idol_manager) = idol_manager::instance(world) {
        idol_manager::perform_resurrection(world, idol_manager);
    }
    ItemResult::Done
}

/// `-[TempleDoorLogic showBattleSelectionPanel:::]` @ 0x1000917dc
///
/// The next turn: a multiplayer result when one can be shown, else the selection panel.
pub fn seq_show_battle_selection_panel(world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let multiplayer_active = match idol_manager::instance(world) {
        Some(idol_manager) => idol_manager::is_multiplayer_active(world, idol_manager),
        None => false,
    };
    if multiplayer_active {
        let can_show_result = match idol_manager::instance(world) {
            Some(idol_manager) => idol_manager::can_show_idol_battle_multiplayer_result(world, idol_manager),
            None => false,
        };
        if can_show_result {
            if let Some(idol_manager) = idol_manager::instance(world) {
                idol_manager::show_idol_battle_multiplayer_result(world, idol_manager);
            }
            return ItemResult::Done;
        }
    }

    if let Some(idol_screen) = idol_screen_logic::instance(world) {
        idol_screen_logic::open_idol_battle_selection_panel(world, idol_screen);
    }
    ItemResult::Done
}

/// `-[TempleDoorLogic onGameOver:::]` @ 0x100091880
pub fn seq_on_game_over(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    game_over(world, this);
    ItemResult::Done
}

/// `-[TempleDoorLogic onMultiModelEvent:eventId:]` @ 0x1000918a4
///
/// The right battle pedestal reports its animations: cleared idols are replaced by the new
/// ones, dropped idols are in place.
pub fn on_multi_model_event(world: &mut World, this: Entity, multi_model_logic: Entity, event_id: &str) {
    if event_id == "BattlePedestalClearIdolComplete" {
        let right_pedestal = world.obj::<TempleDoorLogic>(this).battle_pedestal_right_logic;
        if right_pedestal == Some(multi_model_logic) {
            drop_battle_idols(world, this);
        }
        return;
    }

    if event_id == "BattlePedestalDropIdolComplete" {
        let right_pedestal = world.obj::<TempleDoorLogic>(this).battle_pedestal_right_logic;
        if right_pedestal == Some(multi_model_logic) {
            let mut door = world.obj_mut::<TempleDoorLogic>(this);
            door.pedestal_has_idol = true;
            door.idols_ready = true;
        }
    }
}

/// `-[TempleDoorLogic gameOver]` @ 0x100091a94
pub fn game_over(world: &mut World, this: Entity) {
    remove_enemy_pygmy(world, this);

    if let Some(idol_manager) = idol_manager::instance(world) {
        idol_manager::apply_battle_experience(world, idol_manager);
    }

    if let Some(listener) = door_listener(world, this) {
        crate::controller::on_battle_of_the_gods_stop(world, listener, this);
        crate::controller::on_open_idol_battle_results_screen(world, listener);
    }

    let multiplayer_active = match idol_manager::instance(world) {
        Some(idol_manager) => idol_manager::is_multiplayer_active(world, idol_manager),
        None => false,
    };
    if !multiplayer_active {
        return;
    }

    if let Some(idol_manager) = idol_manager::instance(world) {
        idol_manager::on_multiplayer_game_over(world, idol_manager);
    }
}

/// `-[TempleDoorLogic quitGame]` @ 0x100091b84
pub fn quit_game(world: &mut World, this: Entity) {
    remove_enemy_pygmy(world, this);

    if let Some(idol_manager) = idol_manager::instance(world) {
        idol_manager::quit_battle_of_the_gods(world, idol_manager);
    }

    if let Some(listener) = door_listener(world, this) {
        crate::controller::on_battle_of_the_gods_stop(world, listener, this);
    }
    cancel_battle(world, this);
}

/// `+[TempleDoorLogic instance]` @ 0x1000903b0
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().temple_door_logic
}
