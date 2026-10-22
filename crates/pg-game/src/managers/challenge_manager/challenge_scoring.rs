//! `ChallengeManager`: methods in this part of the class.

use pg_engine::display::text::Text;
use pg_engine::library::Library;

use super::*;

/// `getGameUnitsName:`: the `CounterDescription` of each type.
const GAME_UNITS_NAMES: [&str; 16] = [
    "Coconut Bounces",
    "Hurricane Zaps",
    "Bird Bombs",
    "Shark Snaps",
    "Ooga Jump Height",
    "Piranhas Served",
    "Pygmies Blasted",
    "Spear Throws",
    "Spear Throws",
    "Pygmies",
    "Pain Drain Depth",
    "The Runs Distance",
    "Points",
    "Moron Pests Distance",
    "Dance Dance Execution Points",
    "Pieces of Booty",
];

/// `getCounterNumber:`'s `sprintf` formats for types 4…15 (`PTR_s__d_Oogles_10037d280`).
const COUNTER_NUMBER_FORMATS: [&str; 12] = [
    "%d Oogles", "%d", "%d", "%d", "%d", "%d", "%d Oogles", "%d Oogles", "%d", "%d Oogles", "%d", "%d",
];

/// `getCounterNumber:`'s format for the other types.
const DEFAULT_COUNTER_NUMBER_FORMAT: &str = "%d";

/// The first type of `COUNTER_NUMBER_FORMATS`.
const FIRST_COUNTER_FORMAT_TYPE: i32 = 4;

/// The hunts (T-Rex, Spider, Ice Monster: types 7–9): a running hunt cannot be replaced, and
/// its score counts spears or pygmies, so the lower one is better.
const FIRST_HUNT_TYPE: i32 = 7;
const HUNT_TYPE_COUNT: u32 = 3;

/// Dance Dance Execution shows no counter.
const DANCE_DANCE_EXECUTION_TYPE: i32 = 14;

/// `mChallengeScoreType`.
const HIGHER_SCORE_WINS: i32 = 0;
const LOWER_SCORE_WINS: i32 = 1;

/// The dynamic texts of the counter and the personal-best message.
const COUNTER_DESCRIPTION_TEXT: &str = "CounterDescription";
const COUNTER_NUMBER_TEXT: &str = "CounterNumber";
const PERSONAL_BEST_TEXT: &str = "PersonalBest";
const ACHIEVEMENT_SCORE_TEXT: &str = "AchievementScore";

/// Message behaviours.
const MESSAGE_SHOW_COUNT: &str = "MessageShowCount";
const MESSAGE_HIDE_COUNT: &str = "MessageHideCount";
const MESSAGE_HIGH_SCORE: &str = "MessageHighScore";

/// The sequences that submit an attempt's best score.
const SUBMIT_CHALLENGE_SEQUENCE: &str = "SubmitChallenge";
const SUBMIT_CHALLENGE_RESULT_SEQUENCE: &str = "SubmitChallengeResult";

/// `checkAchievement:score:`'s first message for most games.
const PERSONAL_BEST_MESSAGE: &str = "You've achieved a personal best of";

/// What `checkAchievement:score:` does for one type.
struct ScoreAchievement {
    /// The episode event `triggerEpisodeGameScoreEvent:score:` gets first, if any.
    episode_event: Option<&'static str>,

    /// The god persona's achievement id.
    achievement_id: &'static str,

    message1: &'static str,

    /// A format taking the score.
    message2: &'static str,

    high_score_wins: bool,
}

/// `checkAchievement:score:` by type (`HurricaneLighningScore` is the original's spelling).
const SCORE_ACHIEVEMENTS: [ScoreAchievement; 16] = [
    ScoreAchievement {
        episode_event: Some("CoconutBounceScore"),
        achievement_id: "CoconutBounces",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "%d Coconut Bounces!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("HurricaneLighningScore"),
        achievement_id: "HurricaneLightning",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "%d Hurricane Zaps!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("BirdBombScore"),
        achievement_id: "BirdBombs",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "%d Consecutive Bird Bomb Hits!!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("SharkSnapScore"),
        achievement_id: "SharkSnaps",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "%d Consecutive Shark Snaps!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("OogaJumpScore"),
        achievement_id: "OogaJump",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "a %d Oogle Jump!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("BaitMasterScore"),
        achievement_id: "BaitMaster",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "%d Piranhas Served!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("LaserBlastScore"),
        achievement_id: "LaserShark",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "%d Pygmies Blasted!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: None,
        achievement_id: "KillTRex",
        message1: "You've taken the TRex Down with",
        message2: "%d Spear Throws!",
        high_score_wins: false,
    },
    ScoreAchievement {
        episode_event: None,
        achievement_id: "KillSpider",
        message1: "You've taken the Spider Down with",
        message2: "%d Spear Throws!",
        high_score_wins: false,
    },
    ScoreAchievement {
        episode_event: None,
        achievement_id: "KillIceMonster",
        message1: "You've taken the Ice Monster Down with",
        message2: "%d Pygmies!",
        high_score_wins: false,
    },
    ScoreAchievement {
        episode_event: Some("PainDrainScore"),
        achievement_id: "PainDrain",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "a depth of %d Oogles!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("TheRunsScore"),
        achievement_id: "TheRuns",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "a distance of %d Oogles!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("KonkeyDongScore"),
        achievement_id: "KonkeyDong",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "%d Points!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("MoronPestsScore"),
        achievement_id: "MoronPests",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "a distance of %d Oogles!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("DanceDanceExecutionScore"),
        achievement_id: "DanceDanceExecution",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "%d Points!",
        high_score_wins: true,
    },
    ScoreAchievement {
        episode_event: Some("NoPeeScore"),
        achievement_id: "NoPee",
        message1: PERSONAL_BEST_MESSAGE,
        message2: "%d pieces of booty!",
        high_score_wins: true,
    },
];

/// `(unsigned)(type - 7) < 3`: the T-Rex, Spider and Ice Monster hunts.
fn is_hunt(challenge_type: i32) -> bool {
    let hunt_index = challenge_type.wrapping_sub(FIRST_HUNT_TYPE) as u32;
    hunt_index < HUNT_TYPE_COUNT
}

/// `[[[BCLibrary instance] getDynamicTextById:id] setText:text]`.
fn set_dynamic_text(world: &mut World, id: &str, text: &str) {
    let dynamic_text = world.resource::<Library>().dynamic_text(id);
    if let Some(dynamic_text) = dynamic_text {
        pg_engine::display::text::set_text(world, dynamic_text, Some(text));
    }
}

/// `sprintf([counterNumber useTextBuffer], [self getCounterNumber:type], mActiveGameScore)`,
/// then `[counterNumber showTextBuffer]`.
fn show_counter_number(world: &mut World, this: Entity, challenge_type: i32) {
    let counter_number = world.resource::<Library>().dynamic_text(COUNTER_NUMBER_TEXT);
    let Some(counter_number) = counter_number else {
        return;
    };

    pg_engine::display::text::use_text_buffer(world, counter_number);
    let format = get_counter_number(world, this, challenge_type);
    let active_game_score = world.obj::<ChallengeManager>(this).active_game_score;
    let counter_text = format_with_int(format, active_game_score);
    world.obj_mut::<Text>(counter_number).buffer = counter_text.into_bytes();
    pg_engine::display::text::show_text_buffer(world, counter_number);
}

/// `(!mbChallengeSendAttemptActive && !mbChallengeRespondAttemptActive) || mChallengeType == type`:
/// during a challenge attempt only its own game counts.
fn counts_score_of(world: &World, this: Entity, challenge_type: i32) -> bool {
    let manager = world.obj::<ChallengeManager>(this);
    let attempt_active = manager.challenge_send_attempt_active || manager.challenge_respond_attempt_active;
    !attempt_active || manager.challenge_type == challenge_type
}

/// `[[mChallengeManagerListener godPersona] achievementCount:achievementId]` (0 for a nil
/// persona).
fn god_persona_achievement_count(world: &mut World, this: Entity, achievement_id: &str) -> i32 {
    let listener = listener(world, this);
    let god_persona = match listener {
        Some(listener) => crate::controller::god_persona(world, listener),
        None => None,
    };
    let Some(god_persona) = god_persona else {
        return 0;
    };

    crate::persona::god_persona::achievement_count(world, god_persona, achievement_id)
}

/// `stopGame:`'s update of the attempt's best score: the better of the two by the score type,
/// or this score when the attempt has none yet.
fn update_best_challenge_game_score(world: &mut World, this: Entity) {
    let mut manager = world.obj_mut::<ChallengeManager>(this);
    let score = manager.active_game_score;
    let best = manager.best_challenge_game_score;
    let better = match manager.challenge_score_type {
        LOWER_SCORE_WINS => score < best && score != -1,
        HIGHER_SCORE_WINS => score > best,
        _ => false,
    };
    if better || best == -1 {
        manager.best_challenge_game_score = score;
    }
}

/// The end of a challenge attempt (`stopGame:`, `forfeitGame:`, `onMessageComplete:`): record the
/// best score as a personal best and submit it, then tell the listener.
fn end_challenge_attempt(world: &mut World, this: Entity, challenge_type: i32) {
    let (send_attempt_active, respond_attempt_active) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.challenge_send_attempt_active, manager.challenge_respond_attempt_active)
    };

    if send_attempt_active {
        world.obj_mut::<ChallengeManager>(this).challenge_send_attempt_active = false;
        let best_challenge_game_score = world.obj::<ChallengeManager>(this).best_challenge_game_score;
        check_achievement_score(world, this, challenge_type, best_challenge_game_score);
        set_sequence(world, this, SUBMIT_CHALLENGE_SEQUENCE);
    } else if respond_attempt_active {
        world.obj_mut::<ChallengeManager>(this).challenge_respond_attempt_active = false;
        let best_challenge_game_score = world.obj::<ChallengeManager>(this).best_challenge_game_score;
        check_achievement_score(world, this, challenge_type, best_challenge_game_score);
        set_sequence(world, this, SUBMIT_CHALLENGE_RESULT_SEQUENCE);
    }

    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;
    let listener = listener(world, this);
    if let Some(listener) = listener {
        crate::controller::on_end_challenge(world, listener, challenge_type);
    }
}

/// `[mCountingMessageLogic showMessage:@"MessageHideCount"]`, except for Dance Dance Execution.
fn hide_count(world: &mut World, this: Entity, challenge_type: i32) {
    if challenge_type == DANCE_DANCE_EXECUTION_TYPE {
        return;
    }
    let counting_message_logic = world.obj::<ChallengeManager>(this).counting_message_logic;
    show_message(world, counting_message_logic, MESSAGE_HIDE_COUNT);
}

/// `-[ChallengeManager getGameUnitsName:]` @ 0x1000143a0
pub fn get_game_units_name(_world: &mut World, _this: Entity, challenge_type: i32) -> &'static str {
    challenge_table_entry(&GAME_UNITS_NAMES, challenge_type)
}

/// `-[ChallengeManager getCounterNumber:]` @ 0x1000145ec
pub fn get_counter_number(_world: &mut World, _this: Entity, challenge_type: i32) -> &'static str {
    let format_index = challenge_type.wrapping_sub(FIRST_COUNTER_FORMAT_TYPE) as u32;
    match COUNTER_NUMBER_FORMATS.get(format_index as usize) {
        Some(format) => *format,
        None => DEFAULT_COUNTER_NUMBER_FORMAT,
    }
}

/// `-[ChallengeManager checkAchievement:score:message1:message2:highScoreWins:]` @ 0x1000187b8
pub fn check_achievement_score_message1_message2_high_score_wins(world: &mut World, this: Entity, achievement_id: &str, score: i32, message1: &str, message2: &str, high_score_wins: bool) {
    if score == -1 {
        return;
    }

    let best = god_persona_achievement_count(world, this, achievement_id);
    let higher_best = high_score_wins && best < score;
    let lower_best = !high_score_wins && (score < best || best == 0);
    if !higher_best && !lower_best {
        return;
    }

    set_dynamic_text(world, PERSONAL_BEST_TEXT, message1);
    let achievement_score = format_with_int(message2, score);
    set_dynamic_text(world, ACHIEVEMENT_SCORE_TEXT, &achievement_score);

    let listener = listener(world, this);
    if let Some(listener) = listener {
        crate::controller::on_achievement(world, listener, achievement_id, score, high_score_wins);
    }

    let achievement_message_logic = world.obj::<ChallengeManager>(this).achievement_message_logic;
    show_message(world, achievement_message_logic, MESSAGE_HIGH_SCORE);
}

/// `-[ChallengeManager checkAchievement:score:]` @ 0x10001896c
pub fn check_achievement_score(world: &mut World, this: Entity, challenge_type: i32, score: i32) {
    let row = match usize::try_from(challenge_type) {
        Ok(index) => SCORE_ACHIEVEMENTS.get(index),
        Err(_) => None,
    };
    let Some(row) = row else {
        return;
    };

    if let Some(episode_event) = row.episode_event {
        let idol_manager = crate::managers::idol_manager::instance(world);
        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::trigger_episode_game_score_event(world, idol_manager, episode_event, score);
        }
    }

    check_achievement_score_message1_message2_high_score_wins(world, this, row.achievement_id, score, row.message1, row.message2, row.high_score_wins);
}

/// `-[ChallengeManager isAnyGameActive]` @ 0x100018f00
pub fn is_any_game_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ChallengeManager>(this).game_active
}

/// `-[ChallengeManager isGameActive:]` @ 0x100018f10
pub fn is_game_active(world: &mut World, this: Entity, challenge_type: i32) -> bool {
    let manager = world.obj::<ChallengeManager>(this);
    manager.game_active && manager.active_game_challenge_type == challenge_type
}

/// `-[ChallengeManager startChallengeGame:]` @ 0x100018f40
pub fn start_challenge_game(world: &mut World, this: Entity, challenge_type: i32) {
    let (game_active, active_game_challenge_type) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.game_active, manager.active_game_challenge_type)
    };
    if game_active && active_game_challenge_type != challenge_type {
        stop_game(world, this, active_game_challenge_type);
    }

    let mut manager = world.obj_mut::<ChallengeManager>(this);
    manager.active_game_challenge_type = challenge_type;
    manager.active_game_score = 0;
}

/// `-[ChallengeManager checkStartGame:]` @ 0x100018fe0
pub fn check_start_game(world: &mut World, this: Entity, challenge_type: i32) {
    let (game_active, active_game_challenge_type) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.game_active, manager.active_game_challenge_type)
    };
    if game_active {
        // A running hunt cannot be replaced by another game.
        if active_game_challenge_type == challenge_type || is_hunt(active_game_challenge_type) {
            return;
        }
        stop_game(world, this, active_game_challenge_type);
    }

    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.game_active = true;
        manager.active_game_challenge_type = challenge_type;
        manager.active_game_score = 0;
        manager.challenge_score_type = HIGHER_SCORE_WINS;
        if is_hunt(manager.active_game_challenge_type) {
            manager.challenge_score_type = LOWER_SCORE_WINS;
        }
    }

    let game_units_name = get_game_units_name(world, this, challenge_type);
    set_dynamic_text(world, COUNTER_DESCRIPTION_TEXT, game_units_name);

    let (active_game_challenge_type, counting_message_logic) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.active_game_challenge_type, manager.counting_message_logic)
    };
    if active_game_challenge_type != DANCE_DANCE_EXECUTION_TYPE {
        show_message(world, counting_message_logic, MESSAGE_SHOW_COUNT);
    }

    show_counter_number(world, this, challenge_type);
}

/// `-[ChallengeManager setGameScore:score:]` @ 0x1000191cc
pub fn set_game_score(world: &mut World, this: Entity, challenge_type: i32, score: i32) {
    if !counts_score_of(world, this, challenge_type) {
        return;
    }

    check_start_game(world, this, challenge_type);
    let active_game_challenge_type = world.obj::<ChallengeManager>(this).active_game_challenge_type;
    if active_game_challenge_type != challenge_type {
        return;
    }

    world.obj_mut::<ChallengeManager>(this).active_game_score = score;
    show_counter_number(world, this, challenge_type);
}

/// `-[ChallengeManager incrementGameScore:]` @ 0x100019310
pub fn increment_game_score(world: &mut World, this: Entity, challenge_type: i32) {
    increment_game_score_score(world, this, challenge_type, 1);
}

/// `-[ChallengeManager incrementGameScore:score:]` @ 0x100019480
pub fn increment_game_score_score(world: &mut World, this: Entity, challenge_type: i32, score: i32) {
    if !counts_score_of(world, this, challenge_type) {
        return;
    }

    check_start_game(world, this, challenge_type);
    let active_game_challenge_type = world.obj::<ChallengeManager>(this).active_game_challenge_type;
    if active_game_challenge_type != challenge_type {
        return;
    }

    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.active_game_score = manager.active_game_score.wrapping_add(score);
    }
    show_counter_number(world, this, challenge_type);
}

/// `-[ChallengeManager stopGame:]` @ 0x1000195f4
pub fn stop_game(world: &mut World, this: Entity, challenge_type: i32) {
    let (game_active, active_game_challenge_type) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.game_active, manager.active_game_challenge_type)
    };
    if !game_active || active_game_challenge_type != challenge_type {
        return;
    }

    world.obj_mut::<ChallengeManager>(this).game_active = false;
    hide_count(world, this, active_game_challenge_type);

    let active_game_score = world.obj::<ChallengeManager>(this).active_game_score;
    let bonus_pack_manager = crate::managers::bonus_pack_manager::instance(world);
    if let Some(bonus_pack_manager) = bonus_pack_manager {
        crate::managers::bonus_pack_manager::check_bonus_pack_unlock(world, bonus_pack_manager, challenge_type, active_game_score);
    }

    let (send_attempt_active, respond_attempt_active) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.challenge_send_attempt_active, manager.challenge_respond_attempt_active)
    };
    if !send_attempt_active && !respond_attempt_active {
        let active_game_score = world.obj::<ChallengeManager>(this).active_game_score;
        check_achievement_score(world, this, challenge_type, active_game_score);
        return;
    }

    update_best_challenge_game_score(world, this);

    // While the 61 s timer runs the player may play again; the timer's end submits the best.
    let challenge_timer_active = world.obj::<ChallengeManager>(this).challenge_timer_active;
    if challenge_timer_active {
        return;
    }
    end_challenge_attempt(world, this, challenge_type);
}

/// `-[ChallengeManager forfeitGame:]` @ 0x1000198a4
pub fn forfeit_game(world: &mut World, this: Entity, challenge_type: i32) {
    let (game_active, active_game_challenge_type) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.game_active, manager.active_game_challenge_type)
    };
    if !game_active || active_game_challenge_type != challenge_type {
        return;
    }

    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.game_active = false;
        manager.active_game_score = -1;
    }
    hide_count(world, this, active_game_challenge_type);

    let (send_attempt_active, respond_attempt_active) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.challenge_send_attempt_active, manager.challenge_respond_attempt_active)
    };
    if !send_attempt_active && !respond_attempt_active {
        let active_game_score = world.obj::<ChallengeManager>(this).active_game_score;
        check_achievement_score(world, this, challenge_type, active_game_score);
        return;
    }

    let challenge_timer_active = world.obj::<ChallengeManager>(this).challenge_timer_active;
    if challenge_timer_active {
        return;
    }
    end_challenge_attempt(world, this, challenge_type);
}

/// `-[ChallengeManager onMessageComplete:]` @ 0x10001a0ac
pub fn on_message_complete(world: &mut World, this: Entity, message: Entity) {
    let challenge_timer_logic = world.obj::<ChallengeManager>(this).challenge_timer_logic;
    if challenge_timer_logic != Some(message) {
        return;
    }

    // The 61 s are over. A running game ends the attempt from its own `stopGame:`.
    let (game_active, active_game_challenge_type) = {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.challenge_timer_active = false;
        (manager.game_active, manager.active_game_challenge_type)
    };
    if game_active {
        return;
    }
    end_challenge_attempt(world, this, active_game_challenge_type);
}
