//! `PocketGodViewController`: methods in this part of the class.

use pg_engine::display;
use pg_engine::display::group;
use pg_engine::display::text;
use pg_engine::library;
use pg_engine::prefs;
use pg_engine::sequence;

use crate::persona::persona::Persona;
use crate::persona::tribe_persona::TribePersona;
use crate::prelude::*;
use super::*;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerStatsAndAchievements {
    /// `mGameServer`: `@"GameServer"`
    pub game_server: Option<Entity>,

    /// `mbCheckAllAchievementsAndLeaderboards`: `B`
    pub check_all_achievements_and_leaderboards: bool,

    /// `mGodPersona`: `@"GodPersona"`
    pub god_persona: Option<Entity>,

    /// `mTribePersona`: `@"TribePersona"`
    pub tribe_persona: Option<Entity>,

    /// `mCharliePygmyPersona`: `@"PygmyPersona"`
    pub charlie_pygmy_persona: Option<Entity>,

    /// `mTomPygmyPersona`: `@"PygmyPersona"`
    pub tom_pygmy_persona: Option<Entity>,

    /// `mTotalGifts`: `i`
    pub total_gifts: i32,

    /// `mTotalSacrifices`: `i`
    pub total_sacrifices: i32,

    /// `mbSubmittingScore`: `B`
    pub submitting_score: bool,

    /// `mbSubmitSacrificesWhenComplete`: `B`
    pub submit_sacrifices_when_complete: bool,

    /// `mbSubmitGiftsWhenComplete`: `B`
    pub submit_gifts_when_complete: bool,

    /// `mStatsTribeUiLogicArray`: `@"NSMutableArray"`
    pub stats_tribe_ui_logic_array: Vec<Entity>,

    /// `mStatsPygmyUiLogicArray`: `@"NSMutableArray"`
    pub stats_pygmy_ui_logic_array: Vec<Entity>,

    /// `mStatsGodUiLogicArray`: `@"NSMutableArray"`
    pub stats_god_ui_logic_array: Vec<Entity>,

    /// `mActiveStatsScreen`: `i`
    pub active_stats_screen: i32,

    /// `mStatsTribeCloseButtonLogic`: `@"BCButtonLogic"`
    pub stats_tribe_close_button_logic: Option<Entity>,

    /// `mStatsTribeEditNameButtonLogic`: `@"BCButtonLogic"`
    pub stats_tribe_edit_name_button_logic: Option<Entity>,

    /// `mStatsTribeCancelEditNameButtonLogic`: `@"BCButtonLogic"`
    pub stats_tribe_cancel_edit_name_button_logic: Option<Entity>,

    /// `mGodStatsButtonLogic`: `@"BCButtonLogic"`
    pub god_stats_button_logic: Option<Entity>,

    /// `mPygmyStatsButtonLogicArray`: `@"NSMutableArray"`
    pub pygmy_stats_button_logic_array: Vec<Entity>,

    /// `mStatsPygmyCloseButtonLogic`: `@"BCButtonLogic"`
    pub stats_pygmy_close_button_logic: Option<Entity>,

    /// `mStatsPygmyNextButtonLogic`: `@"BCButtonLogic"`
    pub stats_pygmy_next_button_logic: Option<Entity>,

    /// `mStatsPygmyEditNameButtonLogic`: `@"BCButtonLogic"`
    pub stats_pygmy_edit_name_button_logic: Option<Entity>,

    /// `mStatsPygmyEditDescriptionButtonLogic`: `@"BCButtonLogic"`
    pub stats_pygmy_edit_description_button_logic: Option<Entity>,

    /// `mStatsPygmyCancelEditNameButtonLogic`: `@"BCButtonLogic"`
    pub stats_pygmy_cancel_edit_name_button_logic: Option<Entity>,

    /// `mStatsPygmyCancelEditDescriptionButtonLogic`: `@"BCButtonLogic"`
    pub stats_pygmy_cancel_edit_description_button_logic: Option<Entity>,

    /// `mActivePygmyPersona`: `@"PygmyPersona"`
    pub active_pygmy_persona: Option<Entity>,

    /// `mPygmyStatsType`: `i`
    pub pygmy_stats_type: i32,

    /// `mStatsGodCloseButtonLogic`: `@"BCButtonLogic"`
    pub stats_god_close_button_logic: Option<Entity>,

    /// `mStatsGodNextGodTypeButtonLogic`: `@"BCButtonLogic"`
    pub stats_god_next_god_type_button_logic: Option<Entity>,

    /// `mStatsGodEditNameButtonLogic`: `@"BCButtonLogic"`
    pub stats_god_edit_name_button_logic: Option<Entity>,

    /// `mStatsGodEditDescriptionButtonLogic`: `@"BCButtonLogic"`
    pub stats_god_edit_description_button_logic: Option<Entity>,

    /// `mStatsGodCancelEditNameButtonLogic`: `@"BCButtonLogic"`
    pub stats_god_cancel_edit_name_button_logic: Option<Entity>,

    /// `mStatsGodCancelEditDescriptionButtonLogic`: `@"BCButtonLogic"`
    pub stats_god_cancel_edit_description_button_logic: Option<Entity>,
}

/// The sacrifice ids `getSacrificeDescription:` compares, in its order, with what it returns for
/// each.
const SACRIFICE_DESCRIPTIONS: [(&str, &str); 116] = [
    ("SharkFishingPole", "by fishing pole shark grab"),
    ("LoseGripUpsideDownFromEarthquake", "by earthquake on upside-down island"),
    ("FallUpsideDown", "by fall from upside-down island"),
    ("ThrownExplosionDrown", "thrown from explosion then drowned"),
    ("Lightning", "by lightning"),
    ("HurricaneLightning", "by hurricane then lightning"),
    ("Lava", "by lava"),
    ("BoulderDrown", "by boulder-hit then drowning"),
    ("TRexEggDrown", "by t-rex egg-hit then drowning"),
    ("SharkDrown", "by shark-slap then drowning"),
    ("Vampire", "by vampire attack"),
    ("MagnifyingGlass", "by magnifying glass"),
    ("FireAnts", "by fire ants"),
    ("HurricaneDrown", "by hurricane then drowning"),
    ("FedToShark", "by shark feed"),
    ("VampireInSunlight", "as vampire in sunlight"),
    ("SlideDrown", "by gravity slide then drowning"),
    ("ThrownInVolcano", "by volcano toss"),
    ("FlickDrown", "by distance flick then drowning"),
    ("SharkOnHead", "by shark on head"),
    ("UpsideDownLoseGrip", "by losing grip upside-down"),
    ("PushedDrown", "by pushing then drowning"),
    ("DroppedDrown", "by dropping then drowning"),
    ("NotCrapping", "by holding it too long"),
    ("EatenByTRex", "by t-rex feed"),
    ("LovingPocketGod", "by loving pocket god"),
    ("Skewered", "by a good skewering"),
    ("EatBurntFish", "by choking on burnt fish"),
    ("BonFire", "by standing too close to bon fire"),
    ("Cooked", "by being roasted like a marshmallow"),
    ("EatBurntBird", "by choking on burnt bird"),
    ("CaughtByBird", "carried away by bird"),
    ("Flattened", "flattened like pancake"),
    ("FedToPiranha", "by piranha bait"),
    ("FedToLaserShark", "by underwater shark attack"),
    ("ZappedByLaserShark", "by shark laser beams"),
    ("ZombieExplode", "by being buried alive"),
    ("ZombieExplodeBrainsEaten", "by brains being eaten"),
    ("CarriedAwayGhost", "carried away by ghost"),
    ("PulledIntoGround", "pulled into ghostworld"),
    ("ScaredByGhost", "scared to death"),
    ("FromBansheeScream", "by burst eardrums"),
    ("EatenBySpider", "by spider feed"),
    ("EatenByLittleSpiders", "by baby spider feed"),
    ("HitBySnowBall", "by snow ball"),
    ("EatenByIceMonster", "by ice monster feed"),
    ("ExplodeOnIceMonster", "by damaging ice monster"),
    ("BounceOffIceMonster", "by ice monster deflection"),
    ("ShotOutIgloo", "by igloo overcrowding"),
    ("IglooPygmy", "by igloo pygmy-hit then drowning"),
    ("IglooCannon", "by being shot out of an igloo cannon"),
    ("SwordSwallower", "by swallowing a sword fish"),
    ("LostMyHead", "by losing ones head"),
    ("SquidBait", "by being fed to a squid"),
    ("GlobalWarming", "through global warming"),
    ("Shattered", "frozen, then shattered"),
    ("IciclePinCushion", "as an icicle pin cushion"),
    ("PainDrainSpikes", "spiked in the pain drain"),
    ("CrackHole", "fell in crack hole"),
    ("LavaDrown", "drown in lava"),
    ("LavaRock", "explode from lava rock"),
    ("ApeAltar", "torn apart on altar"),
    ("ApeEaten", "eaten by ape"),
    ("ApeWenchSmash", "smashed by ape on winch"),
    ("ApeBongoSmash", "smashed by ape on bongos"),
    ("MonkeyOnBack", "face peeled by monkey on back"),
    ("MonkeyThrow", "thrown by monkey"),
    ("KidnappedByKonkeyDong", "eaten by konkey dong"),
    ("KonkeyDongBoulder", "bouldered by konkey dong"),
    ("KonkeyDongBeetle", "beetle juiced"),
    ("KonkeyDongFall", "fell off konkey dong level"),
    ("EatenByMoronSnake", "eaten by moron snake"),
    ("EatenByMoronTurtle", "eaten by moron turtle"),
    ("ZappedByWarlockAssassin", "zapped by warlock assassin"),
    ("BurntByWarlockAssassin", "burnt by warlock assassin"),
    ("CutInHalfByCharlie", "cut in half by charlie"),
    ("AbsorbedByHalfPygmy", "absorbed by half pygmy"),
    ("BattleFatality", "fatality in battle"),
    ("JellyfishScubaMask", "scuba masked"),
    ("StungByJellyfish", "stung by jellyfish"),
    ("EatenByJellyfish", "eaten by jellyfish"),
    ("DrankByJellyfish", "pygmy smoothy"),
    ("ElectrocuteByJellyfish", "circle shocked"),
    ("HitByUnderwaterPowerCold", "frozen underwater"),
    ("HitByUnderwaterPowerHeat", "boiled underwater"),
    ("HitByUnderwaterPowerOil", "bp'ed"),
    ("HitByUnderwaterPowerAcid", "disolved in acid"),
    ("HitByUnderwaterPowerAir", "got the bends underwater"),
    ("BabyPygmyEatenByTrex", "veal dinner for t-rex"),
    ("BabyPygmyEatenByShark", "veal cutlets for shark"),
    ("OldFartPygmyHeartAttack", "heart attack"),
    ("FastPygmyHitIceMonster", "flamed ice monster"),
    ("FastPygmyHitVolano", "instant eruption"),
    ("GingerPygmySunBurn", "skin cancer"),
    ("GingerPygmyKillVampire", "vampire meets his match"),
    ("EatenByLocustQueen", "a queen's snack"),
    ("SlicedByLocust", "pygmy sashimi"),
    ("PopLocustBalloon", "pop the balloon"),
    ("SquashedLikeBug", "squashed like a bug"),
    ("LocustFood", "eaten like a leaf"),
    ("TurnedIntoMutantLocust", "mutant leaf eater"),
    ("DanceDanceExecuted", "dance dance executed"),
    ("KilledByPurification", "vaporized by purehell"),
    ("PlagueBodyBurntOnFire", "cremated"),
    ("ExplodeFromThetanBirth", "thetan burst"),
    ("BecameXenu", "birth of lord xenu"),
    ("XenuPuppetShow", "impromptu puppet show"),
    ("XenuStripped", "flesh stripped off bone by xenu"),
    ("EatenByXenu", "eaten by xenu"),
    ("ThetanPummeled", "pummeled"),
    ("BittenToDeath", "face bitten off"),
    ("WaterWingsDrowned", "what lies above"),
    ("EatenByClam", "clam baked"),
    ("ImpregnatedByBlob", "impregnated by blob"),
    ("Stoned", "stoned"),
    ("RunOutOfBreath", "asphyxiated"),
];

/// What `getSacrificeDescription:` returns for an id it does not know.
const UNDEFINED_SACRIFICE_DESCRIPTION: &str = "by undefined death";

/// The gift ids `getGiftDescription:` and `getGiftLeaderboardId:` compare, in their order, with
/// what `getGiftDescription:` returns for each.
const GIFT_DESCRIPTIONS: [(&str, &str); 12] = [
    ("TakeCrap", "drop kids off at pool"),
    ("CatchFish", "caught fish"),
    ("Dance", "the gift of dance"),
    ("ZombieDance", "the gift of zombie dance"),
    ("Coconut", "coconut"),
    ("CookedFish", "cooked fish"),
    ("Fish", "fish"),
    ("CookedBird", "cooked bird"),
    ("BecomeGhost", "become ghost"),
    ("IntoTheLight", "go into the light"),
    ("CatchSwordFish", "caught sword fish"),
    ("Banana", "banana"),
];

/// What `getGiftDescription:` returns for an id it does not know.
const UNDEFINED_GIFT_DESCRIPTION: &str = "by undefined gift";

/// The achievement ids `getAchievementDescription:` compares, in its order, with what it returns
/// for each.
const ACHIEVEMENT_DESCRIPTIONS: [(&str, &str); 20] = [
    ("ConqueredTyrannosaurus", "conquered tyrannosaurus"),
    ("CoconutBounces", "max coconut bounces"),
    ("HurricaneLightning", "max hurricane lightning"),
    ("SharkSnaps", "max shark snaps"),
    ("Skewered", "max skewered"),
    ("Bowled", "max bowled"),
    ("BirdBombs", "max bird bombs"),
    ("OogaJump", "longest ooga jump"),
    ("BaitMaster", "most piranhas served"),
    ("LaserShark", "most shark laser blasts"),
    ("KillTRex", "least spears to kill t-rex"),
    ("KillSpider", "least spears to kill spider"),
    ("ConqueredIceMonster", "conquered ice monster"),
    ("KillIceMonster", "least pygmies to kill ice monster"),
    ("PainDrain", "deepest pain drain"),
    ("TheRuns", "longest the runs"),
    ("KonkeyDong", "highest konkey dong score"),
    ("MoronPests", "longest moron pests"),
    ("DanceDanceExecution", "highest dance dance execution score"),
    ("NoPee", "most pieces of booty collected"),
];

/// What `getAchievementDescription:` returns for an id it does not know.
const UNDEFINED_ACHIEVEMENT_DESCRIPTION: &str = "undefined achievement";

/// The achievement ids `getAchievementLeaderboardId:` compares, in its order, with what it returns
/// for each (nil for any other id).
const ACHIEVEMENT_LEADERBOARD_IDS: [(&str, Option<&str>); 14] = [
    ("ConqueredTyrannosaurus", None),
    ("CoconutBounces", Some("pg.mostcoconutbounces")),
    ("HurricaneLightning", Some("pg.mosthurricanelightning")),
    ("SharkSnaps", Some("pg.mostsharksnaps")),
    ("BirdBombs", Some("pg.mostbirdbombs")),
    ("OogaJump", Some("pg.oogajump")),
    ("BaitMaster", Some("pg.mostpiranhasserved")),
    ("LaserShark", Some("pg.mostsharkblasts")),
    ("PainDrain", Some("pg.paindrain")),
    ("TheRuns", Some("pg.theruns")),
    ("KonkeyDong", Some("pg.konkeydong")),
    ("MoronPests", Some("pg.moronpests")),
    ("DanceDanceExecution", Some("pg.dancedanceexecution")),
    ("NoPee", Some("pg.nopee")),
];

/// The achievements `checkAllAchievementsAndLeaderboards` checks, in its order.
const CHECKED_ACHIEVEMENT_IDS: [&str; 18] = [
    "CoconutBounces",
    "HurricaneLightning",
    "SharkSnaps",
    "Skewered",
    "Bowled",
    "BirdBombs",
    "OogaJump",
    "BaitMaster",
    "LaserShark",
    "KillTRex",
    "KillSpider",
    "KillIceMonster",
    "PainDrain",
    "TheRuns",
    "KonkeyDong",
    "MoronPests",
    "NoPee",
    "DanceDanceExecution",
];

/// How the score of a tiered achievement reaches its tiers.
enum FeintScoring {
    /// Each tier needs a score of at least its threshold.
    AtLeast([i32; 4]),

    /// The first tier needs a score of 1 to 1000 (tested unsigned, as `score - 1 < 1000`); each
    /// further tier needs a score of at most its threshold (fewer spears or pygmies is better).
    AtMost([i32; 3]),
}

/// A tiered achievement of `addFeintAchievementsToArray:achievementId:score:`: its tier ids
/// (minion, shaman, demigod, god), added cumulatively while the score reaches each tier.
struct FeintAchievement {
    achievement_id: &'static str,
    tier_ids: [&'static str; 4],
    scoring: FeintScoring,
}

/// The tiered achievements `addFeintAchievementsToArray:achievementId:score:` compares, in its
/// order.
const FEINT_ACHIEVEMENTS: [FeintAchievement; 18] = [
    FeintAchievement {
        achievement_id: "CoconutBounces",
        tier_ids: ["pg.coconutbouncing.minion", "pg.coconutbouncing.shaman", "pg.coconutbouncing.demigod", "pg.coconutbouncing.god"],
        scoring: FeintScoring::AtLeast([1, 20, 50, 100]),
    },
    FeintAchievement {
        achievement_id: "HurricaneLightning",
        tier_ids: ["pg.hurricanezap.minion", "pg.hurricanezap.shaman", "pg.hurricanezap.demigod", "pg.hurricanezap.god"],
        scoring: FeintScoring::AtLeast([1, 6, 30, 100]),
    },
    FeintAchievement {
        achievement_id: "SharkSnaps",
        tier_ids: ["pg.sharksnap.minion", "pg.sharksnap.shaman", "pg.sharksnap.demigod", "pg.sharksnap.god"],
        scoring: FeintScoring::AtLeast([1, 50, 500, 1000]),
    },
    FeintAchievement {
        achievement_id: "Skewered",
        tier_ids: ["pg.pygmykabob.minion", "pg.pygmykabob.shaman", "pg.pygmykabob.demigod", "pg.pygmykabob.god"],
        scoring: FeintScoring::AtLeast([1, 4, 5, 6]),
    },
    FeintAchievement {
        achievement_id: "Bowled",
        tier_ids: ["pg.pygmybowling.minion", "pg.pygmybowling.shaman", "pg.pygmybowling.demigod", "pg.pygmybowling.god"],
        scoring: FeintScoring::AtLeast([1, 4, 5, 6]),
    },
    FeintAchievement {
        achievement_id: "BirdBombs",
        tier_ids: ["pg.birdbomb.minion", "pg.birdbomb.shaman", "pg.birdbomb.demigod", "pg.birdbomb.god"],
        scoring: FeintScoring::AtLeast([1, 10, 30, 100]),
    },
    FeintAchievement {
        achievement_id: "OogaJump",
        tier_ids: ["pg.oogajump.minion", "pg.oogajump.shaman", "pg.oogajump.demigod", "pg.oogajump.god"],
        scoring: FeintScoring::AtLeast([1, 1000, 3000, 6000]),
    },
    FeintAchievement {
        achievement_id: "BaitMaster",
        tier_ids: ["pg.baitmaster.minion", "pg.baitmaster.shaman", "pg.baitmaster.demigod", "pg.baitmaster.god"],
        scoring: FeintScoring::AtLeast([1, 10, 50, 200]),
    },
    FeintAchievement {
        achievement_id: "LaserShark",
        tier_ids: ["pg.sharkblast.minion", "pg.sharkblast.shaman", "pg.sharkblast.demigod", "pg.sharkblast.god"],
        scoring: FeintScoring::AtLeast([1, 50, 300, 800]),
    },
    FeintAchievement {
        achievement_id: "KillTRex",
        tier_ids: ["pg.trexhunt.minion", "pg.trexhunt.shaman", "pg.trexhunt.demigod", "pg.trexhunt.god"],
        scoring: FeintScoring::AtMost([35, 20, 12]),
    },
    FeintAchievement {
        achievement_id: "KillSpider",
        tier_ids: ["pg.spiderhunt.minion", "pg.spiderhunt.shaman", "pg.spiderhunt.demigod", "pg.spiderhunt.god"],
        scoring: FeintScoring::AtMost([35, 20, 12]),
    },
    FeintAchievement {
        achievement_id: "KillIceMonster",
        tier_ids: ["pg.icemonsterbattle.minion", "pg.icemonsterbattle.shaman", "pg.icemonsterbattle.demigod", "pg.icemonsterbattle.god"],
        scoring: FeintScoring::AtMost([30, 15, 10]),
    },
    FeintAchievement {
        achievement_id: "PainDrain",
        tier_ids: ["pg.paindrain.minion", "pg.paindrain.shaman", "pg.paindrain.demigod", "pg.paindrain.god"],
        scoring: FeintScoring::AtLeast([1, 1000, 4000, 10000]),
    },
    FeintAchievement {
        achievement_id: "TheRuns",
        tier_ids: ["pg.theruns.minion", "pg.theruns.shaman", "pg.theruns.demigod", "pg.theruns.god"],
        scoring: FeintScoring::AtLeast([1, 8000, 30000, 60000]),
    },
    FeintAchievement {
        achievement_id: "KonkeyDong",
        tier_ids: ["pg.konkeydong.minion", "pg.konkeydong.shaman", "pg.konkeydong.demigod", "pg.konkeydong.god"],
        scoring: FeintScoring::AtLeast([1, 15000, 50000, 100000]),
    },
    FeintAchievement {
        achievement_id: "MoronPests",
        tier_ids: ["pg.moronpests.minion", "pg.moronpests.shaman", "pg.moronpests.demigod", "pg.moronpests.god"],
        scoring: FeintScoring::AtLeast([1, 20000, 50000, 100000]),
    },
    FeintAchievement {
        achievement_id: "DanceDanceExecution",
        tier_ids: ["pg.dancedanceexecution.minion", "pg.dancedanceexecution.shaman", "pg.dancedanceexecution.demigod", "pg.dancedanceexecution.god"],
        scoring: FeintScoring::AtLeast([1, 1000, 5000, 10000]),
    },
    FeintAchievement {
        achievement_id: "NoPee",
        tier_ids: ["pg.nopee.minion", "pg.nopee.shaman", "pg.nopee.demigod", "pg.nopee.god"],
        scoring: FeintScoring::AtLeast([1, 4, 10, 20]),
    },
];

/// `-[PocketGodViewController checkAchievementAndLeaderboard:]` @ 0x100070a30
pub fn check_achievement_and_leaderboard(world: &mut World, this: Entity, achievement_id: &str) {
    let god_persona = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
    let achievement_count = god_persona_achievement_count(world, god_persona, achievement_id);
    check_feint_achievements(world, this, achievement_id, achievement_count);

    let leaderboard_id = get_achievement_leaderboard_id(world, this, achievement_id);
    if let Some(leaderboard_id) = leaderboard_id {
        submit_to_leaderboard(world, this, leaderboard_id, achievement_count);
    }
}

/// `-[PocketGodViewController checkAllAchievementsAndLeaderboards]` @ 0x100070ae4
pub fn check_all_achievements_and_leaderboards(world: &mut World, this: Entity) {
    let god_persona = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
    if god_persona.is_some() {
        for achievement_id in CHECKED_ACHIEVEMENT_IDS {
            check_achievement_and_leaderboard(world, this, achievement_id);
        }
        return;
    }

    world.obj_mut::<ControllerStatsAndAchievements>(this).check_all_achievements_and_leaderboards = true;
}

/// `-[PocketGodViewController onAchievement:count:highScoreWins:]` @ 0x100070cc4
pub fn on_achievement(world: &mut World, this: Entity, achievement_id: &str, count: i32, high_score_wins: bool) {
    let god_persona = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
    if count == -1 {
        god_persona_add_incremental_achievement(world, god_persona, achievement_id);
    } else {
        god_persona_set_achievement(world, god_persona, achievement_id, count, high_score_wins);
    }

    let god_persona = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
    let achievement_count = god_persona_achievement_count(world, god_persona, achievement_id);
    check_feint_achievements(world, this, achievement_id, achievement_count);

    let leaderboard_id = get_achievement_leaderboard_id(world, this, achievement_id);
    if let Some(leaderboard_id) = leaderboard_id {
        submit_to_leaderboard(world, this, leaderboard_id, achievement_count);
    }
}

/// `-[PocketGodViewController onPygmySacrificeEvent:]` @ 0x100070dc4
pub fn on_pygmy_sacrifice_event(world: &mut World, this: Entity, sacrifice_id: &str) {
    let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event_by_sacrifice_id(world, idol_manager, sacrifice_id);
    }
}

/// `-[PocketGodViewController onTriggerSacrifice:sacrificeId:]` @ 0x100070de0
pub fn on_trigger_sacrifice(world: &mut World, this: Entity, pygmy_logic: Entity, sacrifice_id: Option<&str>) {
    let tribe_persona = world.obj::<ControllerStatsAndAchievements>(this).tribe_persona;
    tribe_persona_sacrifice(world, tribe_persona, sacrifice_id, pygmy_logic);

    let Some(sacrifice_id) = sacrifice_id else {
        return;
    };

    let tribe_persona = world.obj::<ControllerStatsAndAchievements>(this).tribe_persona;
    let total_sacrifices = persona_total_sacrifices(world, tribe_persona);
    world.obj_mut::<ControllerStatsAndAchievements>(this).total_sacrifices = total_sacrifices;
    submit_to_leaderboard(world, this, "pg.totalsacrifices", total_sacrifices);

    let leaderboard_id = get_sacrifice_leaderboard_id(world, this, sacrifice_id);
    if let Some(leaderboard_id) = leaderboard_id {
        let tribe_persona = world.obj::<ControllerStatsAndAchievements>(this).tribe_persona;
        let sacrifice_count = persona_sacrifice_count(world, tribe_persona, sacrifice_id);
        submit_to_leaderboard(world, this, leaderboard_id, sacrifice_count);
    }
}

/// `-[PocketGodViewController getSacrificeDescription:]` @ 0x100071888
pub fn get_sacrifice_description(_world: &mut World, _this: Entity, sacrifice_id: &str) -> &'static str {
    let known_sacrifice = SACRIFICE_DESCRIPTIONS.iter().find(|(known_id, _)| *known_id == sacrifice_id);
    match known_sacrifice {
        Some((_, description)) => description,
        None => UNDEFINED_SACRIFICE_DESCRIPTION,
    }
}

/// `-[PocketGodViewController getSacrificeLeaderboardId:]` @ 0x100072908
pub fn get_sacrifice_leaderboard_id(_world: &mut World, _this: Entity, _sacrifice_id: &str) -> Option<&'static str> {
    // The original compares the id with every id of `getSacrificeDescription:` and returns nil
    // whatever matched: the per-sacrifice leaderboards are compiled out.
    None
}

/// `-[PocketGodViewController getGiftDescription:]` @ 0x100073410
pub fn get_gift_description(_world: &mut World, _this: Entity, gift_id: &str) -> &'static str {
    let known_gift = GIFT_DESCRIPTIONS.iter().find(|(known_id, _)| *known_id == gift_id);
    match known_gift {
        Some((_, description)) => description,
        None => UNDEFINED_GIFT_DESCRIPTION,
    }
}

/// `-[PocketGodViewController getGiftLeaderboardId:]` @ 0x1000735f0
pub fn get_gift_leaderboard_id(_world: &mut World, _this: Entity, gift_id: &str) -> Option<&'static str> {
    let is_known_gift = GIFT_DESCRIPTIONS.iter().any(|(known_id, _)| *known_id == gift_id);
    if is_known_gift {
        return None;
    }

    // Original bug, kept: an unknown gift id gets the undefined-gift description as leaderboard.
    Some(UNDEFINED_GIFT_DESCRIPTION)
}

/// `-[PocketGodViewController getMoodTextureId:]` @ 0x10007374c
pub fn get_mood_texture_id(_world: &mut World, _this: Entity, mood: i32) -> &'static str {
    match mood {
        0 => "PygmyMoodHappy",
        1 => "PygmyMoodContent",
        3 => "PygmyMoodSad",
        4 => "PygmyMoodAnnoyed",
        5 => "PygmyMoodAngry",
        6 => "PygmyMoodFreakedOut",
        _ => "PygmyMoodNormal",
    }
}

/// `-[PocketGodViewController getMoodDescription:]` @ 0x1000737d8
pub fn get_mood_description(_world: &mut World, _this: Entity, mood: i32) -> Option<&'static str> {
    match mood {
        0 => Some("Happy"),
        1 => Some("Content"),
        2 => Some("Normal"),
        3 => Some("Sad"),
        4 => Some("Annoyed"),
        5 => Some("Angry"),
        6 => Some("FreakedOut"),
        _ => None,
    }
}

/// `-[PocketGodViewController getGodTypeTextureId:]` @ 0x100073870
pub fn get_god_type_texture_id(_world: &mut World, _this: Entity, god_type: i32) -> Option<&'static str> {
    match god_type {
        0 => Some("GodTypeVolcano"),
        1 => Some("GodTypeThunder"),
        2 => Some("GodTypeSun"),
        3 => Some("GodTypeOcean"),
        4 => Some("GodTypeMoon"),
        5 => Some("GodTypeDinosaur"),
        6 => Some("GodTypeOctopus"),
        _ => None,
    }
}

/// `-[PocketGodViewController getGodTypeDescription:]` @ 0x100073908
pub fn get_god_type_description(_world: &mut World, _this: Entity, god_type: i32) -> Option<&'static str> {
    match god_type {
        0 => Some("Volcano God"),
        1 => Some("Thunder God"),
        2 => Some("Sun God"),
        3 => Some("Ocean God"),
        4 => Some("Moon God"),
        5 => Some("Dinosaur God"),
        6 => Some("Octopus God"),
        _ => None,
    }
}

/// `-[PocketGodViewController getAchievementDescription:]` @ 0x1000739a0
pub fn get_achievement_description(_world: &mut World, _this: Entity, achievement_id: &str) -> &'static str {
    let known_achievement = ACHIEVEMENT_DESCRIPTIONS.iter().find(|(known_id, _)| *known_id == achievement_id);
    match known_achievement {
        Some((_, description)) => description,
        None => UNDEFINED_ACHIEVEMENT_DESCRIPTION,
    }
}

/// `-[PocketGodViewController getAchievementLeaderboardId:]` @ 0x100073ca0
pub fn get_achievement_leaderboard_id(_world: &mut World, _this: Entity, achievement_id: &str) -> Option<&'static str> {
    let known_achievement = ACHIEVEMENT_LEADERBOARD_IDS.iter().find(|(known_id, _)| *known_id == achievement_id);
    match known_achievement {
        Some((_, leaderboard_id)) => *leaderboard_id,
        None => None,
    }
}

/// `-[PocketGodViewController addFeintAchievementsToArray:achievementId:score:]` @ 0x100073ebc
pub fn add_feint_achievements_to_array(_world: &mut World, _this: Entity, achievement_array: &mut Vec<&'static str>, achievement_id: &str, score: i32) {
    let feint_achievement = FEINT_ACHIEVEMENTS.iter().find(|feint_achievement| feint_achievement.achievement_id == achievement_id);
    let Some(feint_achievement) = feint_achievement else {
        return;
    };

    match feint_achievement.scoring {
        FeintScoring::AtLeast(thresholds) => {
            for (tier_id, threshold) in feint_achievement.tier_ids.into_iter().zip(thresholds) {
                if score < threshold {
                    return;
                }
                achievement_array.push(tier_id);
            }
        }
        FeintScoring::AtMost(thresholds) => {
            let [minion_id, shaman_id, demigod_id, god_id] = feint_achievement.tier_ids;
            let score_minus_one = score.wrapping_sub(1) as u32;
            if score_minus_one >= 1000 {
                return;
            }
            achievement_array.push(minion_id);

            let further_tier_ids = [shaman_id, demigod_id, god_id];
            for (tier_id, threshold) in further_tier_ids.into_iter().zip(thresholds) {
                if score > threshold {
                    return;
                }
                achievement_array.push(tier_id);
            }
        }
    }
}

/// `-[PocketGodViewController checkFeintAchievements:score:]` @ 0x10007492c
pub fn check_feint_achievements(world: &mut World, this: Entity, achievement_id: &str, score: i32) {
    let mut achievement_array = Vec::with_capacity(5);
    add_feint_achievements_to_array(world, this, &mut achievement_array, achievement_id, score);

    for feint_achievement_id in achievement_array {
        let system_preferences = world.obj::<ControllerBootAndFrame>(this).system_preferences;
        if let Some(system_preferences) = system_preferences {
            prefs::set_preference_as_int(world, system_preferences, feint_achievement_id, 1);
        }

        submit_achievement(world, this, feint_achievement_id);
    }
}

/// `-[PocketGodViewController updatePygmyStats]` @ 0x100074ac4
pub fn update_pygmy_stats(world: &mut World, this: Entity) {
    let stats_page = library::display_group(world, "StatsPagePygmy");
    let pygmy_mood_pic = stats_page_object(world, stats_page, "PygmyMoodPic");
    let pygmy_name_text = stats_page_object(world, stats_page, "PygmyName");
    let pygmy_description_text = stats_page_object(world, stats_page, "PygmyDescription");
    let pygmy_mood_text = stats_page_object(world, stats_page, "PygmyMood");
    let list_tag_text = stats_page_object(world, stats_page, "ListTag");
    let overall_stat_text = stats_page_object(world, stats_page, "OverallStat");
    let stat_texts = [
        stats_page_object(world, stats_page, "Stat0"),
        stats_page_object(world, stats_page, "Stat1"),
        stats_page_object(world, stats_page, "Stat2"),
        stats_page_object(world, stats_page, "Stat3"),
        stats_page_object(world, stats_page, "Stat4"),
    ];

    let active_pygmy_persona = world.obj::<ControllerStatsAndAchievements>(this).active_pygmy_persona;
    let pygmy_name = persona_name(world, active_pygmy_persona);
    set_text_of(world, pygmy_name_text, pygmy_name.as_deref());
    let pygmy_description = persona_description(world, active_pygmy_persona);
    set_text_of(world, pygmy_description_text, pygmy_description.as_deref());
    let mood = persona_mood(world, active_pygmy_persona);
    let mood_description = get_mood_description(world, this, mood);
    set_text_of(world, pygmy_mood_text, mood_description);
    let mood = persona_mood(world, active_pygmy_persona);
    let mood_texture_id = get_mood_texture_id(world, this, mood);
    set_texture_of(world, pygmy_mood_pic, Some(mood_texture_id));

    let pygmy_stats_type = world.obj::<ControllerStatsAndAchievements>(this).pygmy_stats_type;
    if pygmy_stats_type == 1 {
        set_text_of(world, list_tag_text, Some("GIFTS:"));
        let total_gifts = persona_total_gifts(world, active_pygmy_persona);
        let overall_stat = total_gifts.to_string();
        set_text_of(world, overall_stat_text, Some(&overall_stat));

        let gift_ids = persona_sorted_gift_ids(world, active_pygmy_persona);
        let gift_id_count = gift_ids.len() as i32;
        for stat_text in stat_texts {
            set_text_of(world, stat_text, Some(""));
        }

        for (index, stat_text) in stat_texts.into_iter().enumerate() {
            if gift_id_count <= index as i32 {
                break;
            }

            let gift_id = &gift_ids[index];
            let gift_count = persona_gift_count(world, active_pygmy_persona, gift_id);
            let gift_description = get_gift_description(world, this, gift_id);
            let stat = format!("{} {}", gift_count, gift_description);
            set_text_of(world, stat_text, Some(&stat));
        }
    } else if pygmy_stats_type == 0 {
        set_text_of(world, list_tag_text, Some("SACRIFICES:"));
        let total_sacrifices = persona_total_sacrifices(world, active_pygmy_persona);
        let overall_stat = total_sacrifices.to_string();
        set_text_of(world, overall_stat_text, Some(&overall_stat));

        let sacrifice_ids = persona_sorted_sacrifice_ids(world, active_pygmy_persona);
        let sacrifice_id_count = sacrifice_ids.len() as i32;
        for stat_text in stat_texts {
            set_text_of(world, stat_text, Some(""));
        }

        for (index, stat_text) in stat_texts.into_iter().enumerate() {
            if sacrifice_id_count <= index as i32 {
                break;
            }

            let sacrifice_id = &sacrifice_ids[index];
            let sacrifice_count = persona_sacrifice_count(world, active_pygmy_persona, sacrifice_id);
            let sacrifice_description = get_sacrifice_description(world, this, sacrifice_id);
            let stat = format!("{} {}", sacrifice_count, sacrifice_description);
            set_text_of(world, stat_text, Some(&stat));
        }
    }
}

/// `-[PocketGodViewController updateGodStats]` @ 0x100075440
pub fn update_god_stats(world: &mut World, this: Entity) {
    let stats_page = library::display_group(world, "StatsPageGod");
    let god_type_pic = stats_page_object(world, stats_page, "GodTypePic");
    let god_name_text = stats_page_object(world, stats_page, "GodName");
    let god_description_text = stats_page_object(world, stats_page, "GodDescription");
    let god_type_text = stats_page_object(world, stats_page, "GodType");
    let stat_texts = [
        stats_page_object(world, stats_page, "Stat0"),
        stats_page_object(world, stats_page, "Stat1"),
        stats_page_object(world, stats_page, "Stat2"),
        stats_page_object(world, stats_page, "Stat3"),
        stats_page_object(world, stats_page, "Stat4"),
        stats_page_object(world, stats_page, "Stat5"),
    ];

    let god_persona = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
    let god_name = persona_name(world, god_persona);
    set_text_of(world, god_name_text, god_name.as_deref());
    let god_description = persona_description(world, god_persona);
    set_text_of(world, god_description_text, god_description.as_deref());
    let god_type = god_persona_god_type(world, god_persona);
    let god_type_description = get_god_type_description(world, this, god_type);
    set_text_of(world, god_type_text, god_type_description);
    let god_type = god_persona_god_type(world, god_persona);
    let god_type_texture_id = get_god_type_texture_id(world, this, god_type);
    set_texture_of(world, god_type_pic, god_type_texture_id);

    let achievement_ids = god_persona_achievement_ids(world, god_persona);
    let achievement_id_count = achievement_ids.len() as i32;
    for stat_text in stat_texts {
        set_text_of(world, stat_text, Some(""));
    }

    for (index, stat_text) in stat_texts.into_iter().enumerate() {
        if achievement_id_count <= index as i32 {
            break;
        }

        let achievement_id = &achievement_ids[index];
        let achievement_count = god_persona_achievement_count(world, god_persona, achievement_id);
        let achievement_description = get_achievement_description(world, this, achievement_id);
        let stat = format!("{} {}", achievement_count, achievement_description);
        set_text_of(world, stat_text, Some(&stat));
    }
}

/// `-[PocketGodViewController updateTribeStats]` @ 0x100075a88
pub fn update_tribe_stats(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerStatsAndAchievements>(this).pygmy_stats_type = 0;

    let stats_page = library::display_group(world, "StatsPageTribe");
    let tribe_name_text = stats_page_object(world, stats_page, "TribeName");
    let total_gifts_text = stats_page_object(world, stats_page, "TotalGifts");
    let total_sacrifices_text = stats_page_object(world, stats_page, "TotalSacrifices");
    let god_name_text = stats_page_object(world, stats_page, "GodName");
    let god_type_pic = stats_page_object(world, stats_page, "GodType");
    let pygmy_mood_pics = [
        stats_page_object(world, stats_page, "Pygmy0MoodPic"),
        stats_page_object(world, stats_page, "Pygmy1MoodPic"),
        stats_page_object(world, stats_page, "Pygmy2MoodPic"),
        stats_page_object(world, stats_page, "Pygmy3MoodPic"),
        stats_page_object(world, stats_page, "Pygmy4MoodPic"),
        stats_page_object(world, stats_page, "Pygmy5MoodPic"),
    ];
    let pygmy_name_texts = [
        stats_page_object(world, stats_page, "Pygmy0Name"),
        stats_page_object(world, stats_page, "Pygmy1Name"),
        stats_page_object(world, stats_page, "Pygmy2Name"),
        stats_page_object(world, stats_page, "Pygmy3Name"),
        stats_page_object(world, stats_page, "Pygmy4Name"),
        stats_page_object(world, stats_page, "Pygmy5Name"),
    ];

    let tribe_persona = world.obj::<ControllerStatsAndAchievements>(this).tribe_persona;
    let tribe_name = persona_name(world, tribe_persona);
    set_text_of(world, tribe_name_text, tribe_name.as_deref());
    let total_gifts = persona_total_gifts(world, tribe_persona);
    let total_gifts_stat = total_gifts.to_string();
    set_text_of(world, total_gifts_text, Some(&total_gifts_stat));
    let total_sacrifices = persona_total_sacrifices(world, tribe_persona);
    let total_sacrifices_stat = total_sacrifices.to_string();
    set_text_of(world, total_sacrifices_text, Some(&total_sacrifices_stat));

    let god_persona = world.obj::<ControllerStatsAndAchievements>(this).god_persona;
    let god_name = persona_name(world, god_persona);
    set_text_of(world, god_name_text, god_name.as_deref());
    let god_type = god_persona_god_type(world, god_persona);
    let god_type_texture_id = get_god_type_texture_id(world, this, god_type);
    set_texture_of(world, god_type_pic, god_type_texture_id);

    let pygmy_persona_array = tribe_pygmy_persona_array(world, tribe_persona);
    for index in 0..pygmy_mood_pics.len() {
        let pygmy_persona = object_at_index(pygmy_persona_array.as_deref(), index);
        let mood = persona_mood(world, pygmy_persona);
        let mood_texture_id = get_mood_texture_id(world, this, mood);
        set_texture_of(world, pygmy_mood_pics[index], Some(mood_texture_id));
        let pygmy_name = persona_name(world, pygmy_persona);
        set_text_of(world, pygmy_name_texts[index], pygmy_name.as_deref());
    }
}

/// `-[PocketGodViewController closeStatsScreen]` @ 0x1000777d0
pub fn close_stats_screen(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerMenusAndScreens>(this).stats_screen_open = false;

    let keyboard_open = world.obj::<ControllerAppPlatform>(this).keyboard_open;
    if keyboard_open {
        world.obj_mut::<ControllerAppPlatform>(this).keyboard_open = false;
        let active_cancel_edit_display_object = world.obj::<ControllerAppPlatform>(this).active_cancel_edit_display_object;
        if let Some(active_cancel_edit_display_object) = active_cancel_edit_display_object {
            let mut cancel_edit_display_object = display::object_mut(world, active_cancel_edit_display_object);
            cancel_edit_display_object.is_selectable = false;
            cancel_edit_display_object.hidden = true;
        }
        perform_selector_on_main_thread(world, this, hide_keyboard);
    }

    let controller_sequence = world.obj::<ControllerBootAndFrame>(this).sequence;
    let close_stats_tribe = library::sequence_def(world, "CloseStatsTribe");
    if let Some(controller_sequence) = controller_sequence {
        sequence::set_sequence_def(world, controller_sequence, close_stats_tribe);
    }
}

/// `-[PocketGodViewController (Game) onPygmyGiftGiven:giftId:]` @ 0x100164adc
pub fn on_pygmy_gift_given(world: &mut World, this: Entity, pygmy_logic: Entity, gift_id: &str) {
    let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event_by_gift_id(world, idol_manager, gift_id);
    }

    let tribe_persona = world.obj::<ControllerStatsAndAchievements>(this).tribe_persona;
    tribe_persona_add_gift(world, tribe_persona, gift_id, pygmy_logic);
    let total_gifts = persona_total_gifts(world, tribe_persona);
    world.obj_mut::<ControllerStatsAndAchievements>(this).total_gifts = total_gifts;
    submit_to_leaderboard(world, this, "pg.totalgifts", total_gifts);

    let leaderboard_id = get_gift_leaderboard_id(world, this, gift_id);
    if let Some(leaderboard_id) = leaderboard_id {
        let gift_count = persona_gift_count(world, tribe_persona, gift_id);
        submit_to_leaderboard(world, this, leaderboard_id, gift_count);
    }
}

/// `[statsPage getDisplayObject:id]` (nil for a nil page).
fn stats_page_object(world: &World, stats_page: Option<Entity>, id: &str) -> Option<Entity> {
    let stats_page = stats_page?;
    group::get_display_object(world, stats_page, id)
}

/// `[textObject setText:newText]` (nothing for a nil object).
fn set_text_of(world: &mut World, text_object: Option<Entity>, new_text: Option<&str>) {
    if let Some(text_object) = text_object {
        text::set_text(world, text_object, new_text);
    }
}

/// `[model setTextureDef:[[BCLibrary instance] getTextureDefById:textureId]]` (nothing for a nil
/// model).
fn set_texture_of(world: &mut World, model: Option<Entity>, texture_id: Option<&str>) {
    // PORT: `getTextureDefById:nil` reads an uninitialised length in the original; no texture
    // is the closest behaviour (a nil id only comes from a god type outside 0 to 6).
    let texture = match texture_id {
        Some(texture_id) => library::texture_def_id(world, texture_id),
        None => None,
    };
    if let Some(model) = model {
        display::set_texture_def(world, model, texture);
    }
}

/// `[array objectAtIndex:index]` (nil for a nil array; past the end it raises, as `NSArray` does).
fn object_at_index(array: Option<&[Entity]>, index: usize) -> Option<Entity> {
    let array = array?;
    Some(array[index])
}

/// `[persona name]`: the persona's `mName` (nil for a nil persona).
fn persona_name(world: &World, persona: Option<Entity>) -> Option<String> {
    let persona = persona?;
    world.obj::<Persona>(persona).name.clone()
}

/// `[persona description]`: the persona's `mDescription` (nil for a nil persona).
fn persona_description(world: &World, persona: Option<Entity>) -> Option<String> {
    let persona = persona?;
    world.obj::<Persona>(persona).description.clone()
}

/// `[persona mood]` (0 for a nil persona).
fn persona_mood(world: &mut World, persona: Option<Entity>) -> i32 {
    match persona {
        Some(persona) => crate::persona::persona::mood(world, persona),
        None => 0,
    }
}

/// `[persona totalSacrifices]` (0 for a nil persona).
fn persona_total_sacrifices(world: &mut World, persona: Option<Entity>) -> i32 {
    match persona {
        Some(persona) => crate::persona::persona::total_sacrifices(world, persona),
        None => 0,
    }
}

/// `[persona totalGifts]` (0 for a nil persona).
fn persona_total_gifts(world: &mut World, persona: Option<Entity>) -> i32 {
    match persona {
        Some(persona) => crate::persona::persona::total_gifts(world, persona),
        None => 0,
    }
}

/// `[persona sacrificeCount:sacrificeId]` (0 for a nil persona).
fn persona_sacrifice_count(world: &mut World, persona: Option<Entity>, sacrifice_id: &str) -> i32 {
    let Some(persona) = persona else {
        return 0;
    };

    crate::persona::persona::sacrifice_count(world, persona, sacrifice_id)
}

/// `[persona giftCount:giftId]` (0 for a nil persona).
fn persona_gift_count(world: &mut World, persona: Option<Entity>, gift_id: &str) -> i32 {
    let Some(persona) = persona else {
        return 0;
    };

    crate::persona::persona::gift_count(world, persona, gift_id)
}

/// `[persona sortedSacrificeIdArray]` (empty for a nil persona).
fn persona_sorted_sacrifice_ids(world: &mut World, persona: Option<Entity>) -> Vec<String> {
    let Some(persona) = persona else {
        return Vec::new();
    };

    crate::persona::persona::sorted_sacrifice_id_array(world, persona)
}

/// `[persona sortedGiftIdArray]` (empty for a nil persona).
fn persona_sorted_gift_ids(world: &mut World, persona: Option<Entity>) -> Vec<String> {
    let Some(persona) = persona else {
        return Vec::new();
    };

    crate::persona::persona::sorted_gift_id_array(world, persona)
}

/// `[godPersona godType]` (0 for a nil persona).
fn god_persona_god_type(world: &mut World, god_persona: Option<Entity>) -> i32 {
    match god_persona {
        Some(god_persona) => crate::persona::god_persona::god_type(world, god_persona),
        None => 0,
    }
}

/// `[godPersona achievementCount:achievementId]` (0 for a nil persona).
fn god_persona_achievement_count(world: &mut World, god_persona: Option<Entity>, achievement_id: &str) -> i32 {
    let Some(god_persona) = god_persona else {
        return 0;
    };

    crate::persona::god_persona::achievement_count(world, god_persona, achievement_id)
}

/// `[godPersona setAchievement:achievementId count:count highScoreWins:highScoreWins]`.
fn god_persona_set_achievement(world: &mut World, god_persona: Option<Entity>, achievement_id: &str, count: i32, high_score_wins: bool) {
    let Some(god_persona) = god_persona else {
        return;
    };

    crate::persona::god_persona::set_achievement(world, god_persona, achievement_id, count, high_score_wins);
}

/// `[godPersona addIncrementalAchievement:achievementId]`.
fn god_persona_add_incremental_achievement(world: &mut World, god_persona: Option<Entity>, achievement_id: &str) {
    let Some(god_persona) = god_persona else {
        return;
    };

    crate::persona::god_persona::add_incremental_achievement(world, god_persona, achievement_id);
}

/// `[godPersona achievementIdArray]`: the keys of its achievement dictionary, in the
/// dictionary's order (empty for a nil persona).
fn god_persona_achievement_ids(world: &mut World, god_persona: Option<Entity>) -> Vec<String> {
    let Some(god_persona) = god_persona else {
        return Vec::new();
    };

    crate::persona::god_persona::achievement_id_array(world, god_persona)
}

/// `[tribePersona sacrifice:sacrificeId pygmyLogic:pygmyLogic]`.
fn tribe_persona_sacrifice(world: &mut World, tribe_persona: Option<Entity>, sacrifice_id: Option<&str>, pygmy_logic: Entity) {
    let Some(tribe_persona) = tribe_persona else {
        return;
    };

    crate::persona::tribe_persona::sacrifice(world, tribe_persona, sacrifice_id, pygmy_logic);
}

/// `[tribePersona addGift:giftId pygmyLogic:pygmyLogic]`.
fn tribe_persona_add_gift(world: &mut World, tribe_persona: Option<Entity>, gift_id: &str, pygmy_logic: Entity) {
    let Some(tribe_persona) = tribe_persona else {
        return;
    };

    crate::persona::tribe_persona::add_gift(world, tribe_persona, gift_id, pygmy_logic);
}

/// `[tribePersona pygmyPersonaArray]`: the persona's `mPygmyPersonaArray` (nil for a nil
/// persona).
fn tribe_pygmy_persona_array(world: &World, tribe_persona: Option<Entity>) -> Option<Vec<Entity>> {
    let tribe_persona = tribe_persona?;
    let pygmy_persona_array = world.obj::<TribePersona>(tribe_persona).pygmy_persona_array.clone();
    Some(pygmy_persona_array)
}

/// `[mGameServer submitToLeaderboard:leaderboardId score:score]`.
fn submit_to_leaderboard(world: &mut World, this: Entity, leaderboard_id: &str, score: i32) {
    let game_server = world.obj::<ControllerStatsAndAchievements>(this).game_server;
    let Some(game_server) = game_server else {
        return;
    };

    crate::managers::game_server::submit_to_leaderboard(world, game_server, leaderboard_id, score);
}

/// `[mGameServer submitAchievement:achievementId]`.
fn submit_achievement(world: &mut World, this: Entity, achievement_id: &str) {
    let game_server = world.obj::<ControllerStatsAndAchievements>(this).game_server;
    let Some(game_server) = game_server else {
        return;
    };

    crate::managers::game_server::submit_achievement(world, game_server, achievement_id);
}
