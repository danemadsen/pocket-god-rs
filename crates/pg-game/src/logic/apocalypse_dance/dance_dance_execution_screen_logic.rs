//! `DanceDanceExecutionScreenLogic` (superclass `BCScreenLogic`).
//!
//! The rhythm game of the Dance temple. The beat chart (`dance-beats.json`) is fed segment by
//! segment into a ring of 100 beats; each beat's arrow symbols enter above the top of the
//! screen and scroll down to the four targets, and the arrow buttons are judged against the
//! beat in the hit window. Misses and wrong presses fill the doom meter ("groove"); a full
//! meter executes the dancing pygmy.

use std::sync::OnceLock;

use pg_assets::data::dance_beats::{DanceBeat, DanceBeatsFile, DanceGroup};

use crate::prelude::*;

use crate::controller::cf_absolute_time_get_current;
use crate::logic::apocalypse_dance::dance_dance_execution_logic;
use crate::logic_table;
use pg_engine::animation;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::logic::screen;
use pg_engine::math::trunc_i32;
use pg_engine::sound;
use pg_engine::view::BcView;

/// `DanceDanceExecutionScreenLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct DanceDanceExecutionScreenLogic {
    /// `mDanceDanceExecutionScreenLogicListener`: `@"<DanceDanceExecutionScreenLogicListener>"`
    pub dance_dance_execution_screen_logic_listener: Option<Entity>,

    /// `mDanceBarLogic`: `@"MultiModelLogic"`
    pub dance_bar_logic: Option<Entity>,

    /// `mLeftOverlayLogic`: `@"MultiModelLogic"`
    pub left_overlay_logic: Option<Entity>,

    /// `mDownOverlayLogic`: `@"MultiModelLogic"`
    pub down_overlay_logic: Option<Entity>,

    /// `mUpOverlayLogic`: `@"MultiModelLogic"`
    pub up_overlay_logic: Option<Entity>,

    /// `mRightOverlayLogic`: `@"MultiModelLogic"`
    pub right_overlay_logic: Option<Entity>,

    /// `mLeftFeedbackLogic`: `@"MultiModelLogic"`
    pub left_feedback_logic: Option<Entity>,

    /// `mDownFeedbackLogic`: `@"MultiModelLogic"`
    pub down_feedback_logic: Option<Entity>,

    /// `mUpFeedbackLogic`: `@"MultiModelLogic"`
    pub up_feedback_logic: Option<Entity>,

    /// `mRightFeedbackLogic`: `@"MultiModelLogic"`
    pub right_feedback_logic: Option<Entity>,

    /// `mLeftButtonLogic`: `@"BCButtonLogic"`
    pub left_button_logic: Option<Entity>,

    /// `mDownButtonLogic`: `@"BCButtonLogic"`
    pub down_button_logic: Option<Entity>,

    /// `mUpButtonLogic`: `@"BCButtonLogic"`
    pub up_button_logic: Option<Entity>,

    /// `mRightButtonLogic`: `@"BCButtonLogic"`
    pub right_button_logic: Option<Entity>,

    /// `mLeftTouch`: `@"BCTouch"`
    pub left_touch: Option<Entity>,

    /// `mDownTouch`: `@"BCTouch"`
    pub down_touch: Option<Entity>,

    /// `mUpTouch`: `@"BCTouch"`
    pub up_touch: Option<Entity>,

    /// `mRightTouch`: `@"BCTouch"`
    pub right_touch: Option<Entity>,

    /// `mbGameOver`: `B`
    pub game_over: bool,

    /// `mbLoopStarted`: `B`
    pub loop_started: bool,

    /// `mActiveBeatCount`: `f`
    pub active_beat_count: f32,

    /// `mDanceGroupIndex`: `i`
    pub dance_group_index: i32,

    /// `mSymbolAlpha`: `f`
    pub symbol_alpha: f32,

    /// `mSymbolTargetAlpha`: `f`
    pub symbol_target_alpha: f32,

    /// `mTargetGroovePercent`: `f`
    pub target_groove_percent: f32,

    /// `mGroovePercent`: `f`
    pub groove_percent: f32,

    /// `mStartDanceTime`: `d`
    pub start_dance_time: f64,

    /// `mbDanceDanceExecutionActive`: `B`
    pub dance_dance_execution_active: bool,

    /// `mActiveSegmentBeatCount`: `i`
    pub active_segment_beat_count: i32,

    /// `mpActiveDanceDanceBeatNodeDef`: `^{BCStructXmlDanceDanceBeatNodeDef=iiBBBBBBBBf@@@@i}`
    ///
    /// The hittable beat: an index into the beat ring
    /// ([`DanceDanceExecutionScreenLogicStatics::beat_ring`]).
    pub active_dance_dance_beat_node_def: Option<usize>,

    /// `mBeatHitPercent`: `f`
    pub beat_hit_percent: f32,

    /// `mLeftMissBeatCount`: `f`
    pub left_miss_beat_count: f32,

    /// `mDownMissBeatCount`: `f`
    pub down_miss_beat_count: f32,

    /// `mUpMissBeatCount`: `f`
    pub up_miss_beat_count: f32,

    /// `mRightMissBeatCount`: `f`
    pub right_miss_beat_count: f32,

    /// `mLeftBadHitBeatCount`: `f`
    pub left_bad_hit_beat_count: f32,

    /// `mDownBadHitBeatCount`: `f`
    pub down_bad_hit_beat_count: f32,

    /// `mUpBadHitBeatCount`: `f`
    pub up_bad_hit_beat_count: f32,

    /// `mRightBadHitBeatCount`: `f`
    pub right_bad_hit_beat_count: f32,

    /// `mSymbolLogicArray`: `@"NSMutableArray"`
    pub symbol_logic_array: Vec<Entity>,

    /// `mIntroStartTime`: `d`
    pub intro_start_time: f64,

    /// `mTimingFudgeFactor`: `d`
    pub timing_fudge_factor: f64,

    /// `mDanceScore`: `i`
    pub dance_score: i32,

    /// `mbTriggerDeathWarning`: `B`
    pub trigger_death_warning: bool,

    /// `mStreakCount`: `i`
    pub streak_count: i32,
}

pub const DANCE_DANCE_EXECUTION_SCREEN_LOGIC: Class = Class {
    name: "DanceDanceExecutionScreenLogic",
    superclass: Some(&pg_engine::logic::screen::BC_SCREEN_LOGIC),
    sequence_item,
    on_game_frame,
    close_screen_now,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..pg_engine::logic::screen::BC_SCREEN_LOGIC
};

/// Seconds per beat of `DanceLoop2`.
const SECONDS_PER_BEAT: f64 = 0.2325546875;

/// Seconds from the intro's start to the first beat on target, and to the start of `DanceLoop2`.
const INTRO_SECONDS: f64 = 7.5;

/// `mTimingFudgeFactor` while the display link draws every second screen refresh: one frame.
const HALF_RATE_TIMING_FUDGE: f64 = 1.0 / 30.0;

/// Beats a symbol scrolls from appearing to reaching its target.
const SCROLL_BEATS: f32 = 13.0;

/// The symbols appear at this height and move by this much over `SCROLL_BEATS` beats.
const SYMBOL_START_Y_POS: f32 = 350.0;
const SYMBOL_SCROLL_DISTANCE: f32 = -317.0;

/// A past beat leaves the ring once its symbols are below this height.
const SYMBOL_REMOVE_Y_POS: f32 = -50.0;

/// A beat can be hit within this many beats of reaching its target, and is missed after that.
const HIT_WINDOW_BEATS: f32 = 0.5;

/// A miss is not counted this soon after a wrong press of the same arrow, nor a wrong press
/// this soon after a miss.
const MISS_AFTER_BAD_HIT_BEATS: f32 = 2.0;
const BAD_HIT_AFTER_MISS_BEATS: f32 = 0.5;

/// `updateGroovePercent:` steps: misses and wrong presses fill the doom meter, hits drain it.
const MISS_GROOVE_DELTA: f32 = 0.22;
const PERFECT_GROOVE_DELTA: f32 = -0.012;
const GOOD_GROOVE_DELTA: f32 = -0.008;
const OK_GROOVE_DELTA: f32 = -0.005;

/// `mBeatHitPercent` above which a hit is perfect, or good.
const PERFECT_HIT_PERCENT: f32 = 0.8;
const GOOD_HIT_PERCENT: f32 = 0.5;

/// Hits are worth `-delta · 1000` points.
const POINTS_PER_GROOVE: f32 = -1000.0;

/// Above this groove the "look out" warning plays and the pygmy looks bad.
const DEATH_WARNING_GROOVE: f32 = 0.6;

/// An empty doom meter after this many beats makes the pygmy cocky.
const COCKY_BEAT_COUNT: f32 = 30.0;

/// `setDanceFace:` values.
const DANCE_FACE_GOOD: i32 = 0;
const DANCE_FACE_BAD: i32 = 1;
const DANCE_FACE_COCKY: i32 = 2;

/// The sounds the screen preloads when it opens and unloads when it closes.
const DANCE_SOUND_IDS: [&str; 9] = [
    "DanceIntro",
    "DanceLoop2",
    "DanceFeedbackGreat",
    "DanceFeedbackAwesome",
    "DanceFeedbackYouRock",
    "DanceFeedbackLookout",
    "DanceDanceButtonLeftPerfectHit",
    "DanceDanceButtonLeftGoodHit",
    "DanceDanceButtonLeftBadHit",
];

/// Entries of the beat ring.
const BEAT_RING_SIZE: i32 = 100;

/// [`DanceDanceBeat::state`] values.
const BEAT_QUEUED: i32 = 0;
const BEAT_ACTIVE: i32 = 1;
const BEAT_PAST: i32 = 2;
const BEAT_REMOVE: i32 = 3;

/// One beat in the ring: the copy of its `BCStructXmlDanceDanceBeatNodeDef` (0x40 bytes; the
/// node type at +0x00 is copied too but never read).
#[derive(Clone, Copy, Debug, Default)]
pub struct DanceDanceBeat {
    /// +0x04: the beat on which the symbols appear; they reach their targets `SCROLL_BEATS`
    /// later. The chart holds the beat's duration here; the copy holds the sum of the durations
    /// queued before it.
    pub start_beat: i32,

    /// +0x08–0x0b: the arrows of the beat (left, down, up, right).
    pub arrows: [bool; 4],

    /// +0x0c–0x0f: the arrows already hit.
    pub hits: [bool; 4],

    /// +0x10: the symbols' y position.
    pub symbol_y_pos: f32,

    /// +0x18–0x30: the arrows' symbols, retained by the ring.
    pub symbol_logics: [Option<Entity>; 4],

    /// +0x38: `BEAT_QUEUED`, `BEAT_ACTIVE` (scrolling, hittable), `BEAT_PAST` or `BEAT_REMOVE`.
    pub state: i32,
}

impl DanceDanceBeat {
    /// The ring's copy of a chart beat. The def's runtime members are zero in the binary, so
    /// only the arrows and the start beat are set.
    fn queued(chart_beat: &DanceBeat, start_beat: i32) -> Self {
        let arrows = [chart_beat.left, chart_beat.down, chart_beat.up, chart_beat.right];
        Self {
            start_beat,
            arrows,
            ..Self::default()
        }
    }
}

/// `DanceDanceExecutionScreenLogic`'s file-scope globals: the ring of queued beats.
#[derive(Resource)]
pub struct DanceDanceExecutionScreenLogicStatics {
    /// `DAT_1007ce3b8`: the ring.
    pub beat_ring: [DanceDanceBeat; BEAT_RING_SIZE as usize],

    /// `DAT_1007e43f0`: the oldest beat still in the ring.
    pub beat_ring_head: i32,

    /// `DAT_1007e43f4`: where the next beat is queued.
    pub beat_ring_tail: i32,
}

impl Default for DanceDanceExecutionScreenLogicStatics {
    fn default() -> Self {
        Self {
            beat_ring: [DanceDanceBeat::default(); BEAT_RING_SIZE as usize],
            beat_ring_head: 0,
            beat_ring_tail: 0,
        }
    }
}

/// The four arrows, in the original's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arrow {
    Left,
    Down,
    Up,
    Right,
}

const ARROWS: [Arrow; 4] = [Arrow::Left, Arrow::Down, Arrow::Up, Arrow::Right];

impl Arrow {
    /// The arrow's slot in a beat's per-arrow members.
    fn index(self) -> usize {
        self as usize
    }

    /// The x position of the arrow's symbols.
    fn symbol_x_pos(self) -> f32 {
        match self {
            Arrow::Left => 44.0,
            Arrow::Down => 110.0,
            Arrow::Up => 370.0,
            Arrow::Right => 436.0,
        }
    }

    fn symbol_behavior(self) -> &'static str {
        match self {
            Arrow::Left => "DanceDanceSymbolLeft",
            Arrow::Down => "DanceDanceSymbolDown",
            Arrow::Up => "DanceDanceSymbolUp",
            Arrow::Right => "DanceDanceSymbolRight",
        }
    }

    /// The button overlay's behaviour when the screen opens.
    fn init_behavior(self) -> &'static str {
        match self {
            Arrow::Left => "DanceDanceButtonLeftInit",
            Arrow::Down => "DanceDanceButtonDownInit",
            Arrow::Up => "DanceDanceButtonUpInit",
            Arrow::Right => "DanceDanceButtonRightInit",
        }
    }

    /// The button overlay's behaviour for an arrow that scrolled past unhit.
    fn miss_behavior(self) -> &'static str {
        match self {
            Arrow::Left => "DanceDanceButtonLeftMiss",
            Arrow::Down => "DanceDanceButtonDownMiss",
            Arrow::Up => "DanceDanceButtonUpMiss",
            Arrow::Right => "DanceDanceButtonRightMiss",
        }
    }

    fn perfect_behavior(self) -> &'static str {
        match self {
            Arrow::Left => "DanceDanceButtonLeftPerfect",
            Arrow::Down => "DanceDanceButtonDownPerfect",
            Arrow::Up => "DanceDanceButtonUpPerfect",
            Arrow::Right => "DanceDanceButtonRightPerfect",
        }
    }

    fn correct_behavior(self) -> &'static str {
        match self {
            Arrow::Left => "DanceDanceButtonLeftCorrect",
            Arrow::Down => "DanceDanceButtonDownCorrect",
            Arrow::Up => "DanceDanceButtonUpCorrect",
            Arrow::Right => "DanceDanceButtonRightCorrect",
        }
    }

    /// The button overlay's behaviour for a press with nothing to hit.
    fn incorrect_behavior(self) -> &'static str {
        match self {
            Arrow::Left => "DanceDanceButtonLeftIncorrect",
            Arrow::Down => "DanceDanceButtonDownIncorrect",
            Arrow::Up => "DanceDanceButtonUpIncorrect",
            Arrow::Right => "DanceDanceButtonRightIncorrect",
        }
    }

    fn button_logic(self, screen_logic: &DanceDanceExecutionScreenLogic) -> Option<Entity> {
        match self {
            Arrow::Left => screen_logic.left_button_logic,
            Arrow::Down => screen_logic.down_button_logic,
            Arrow::Up => screen_logic.up_button_logic,
            Arrow::Right => screen_logic.right_button_logic,
        }
    }

    fn overlay_logic(self, screen_logic: &DanceDanceExecutionScreenLogic) -> Option<Entity> {
        match self {
            Arrow::Left => screen_logic.left_overlay_logic,
            Arrow::Down => screen_logic.down_overlay_logic,
            Arrow::Up => screen_logic.up_overlay_logic,
            Arrow::Right => screen_logic.right_overlay_logic,
        }
    }

    fn feedback_logic(self, screen_logic: &DanceDanceExecutionScreenLogic) -> Option<Entity> {
        match self {
            Arrow::Left => screen_logic.left_feedback_logic,
            Arrow::Down => screen_logic.down_feedback_logic,
            Arrow::Up => screen_logic.up_feedback_logic,
            Arrow::Right => screen_logic.right_feedback_logic,
        }
    }

    fn touch(self, screen_logic: &DanceDanceExecutionScreenLogic) -> Option<Entity> {
        match self {
            Arrow::Left => screen_logic.left_touch,
            Arrow::Down => screen_logic.down_touch,
            Arrow::Up => screen_logic.up_touch,
            Arrow::Right => screen_logic.right_touch,
        }
    }

    fn touch_mut(self, screen_logic: &mut DanceDanceExecutionScreenLogic) -> &mut Option<Entity> {
        match self {
            Arrow::Left => &mut screen_logic.left_touch,
            Arrow::Down => &mut screen_logic.down_touch,
            Arrow::Up => &mut screen_logic.up_touch,
            Arrow::Right => &mut screen_logic.right_touch,
        }
    }

    fn miss_beat_count(self, screen_logic: &DanceDanceExecutionScreenLogic) -> f32 {
        match self {
            Arrow::Left => screen_logic.left_miss_beat_count,
            Arrow::Down => screen_logic.down_miss_beat_count,
            Arrow::Up => screen_logic.up_miss_beat_count,
            Arrow::Right => screen_logic.right_miss_beat_count,
        }
    }

    fn miss_beat_count_mut(self, screen_logic: &mut DanceDanceExecutionScreenLogic) -> &mut f32 {
        match self {
            Arrow::Left => &mut screen_logic.left_miss_beat_count,
            Arrow::Down => &mut screen_logic.down_miss_beat_count,
            Arrow::Up => &mut screen_logic.up_miss_beat_count,
            Arrow::Right => &mut screen_logic.right_miss_beat_count,
        }
    }

    fn bad_hit_beat_count(self, screen_logic: &DanceDanceExecutionScreenLogic) -> f32 {
        match self {
            Arrow::Left => screen_logic.left_bad_hit_beat_count,
            Arrow::Down => screen_logic.down_bad_hit_beat_count,
            Arrow::Up => screen_logic.up_bad_hit_beat_count,
            Arrow::Right => screen_logic.right_bad_hit_beat_count,
        }
    }

    fn bad_hit_beat_count_mut(self, screen_logic: &mut DanceDanceExecutionScreenLogic) -> &mut f32 {
        match self {
            Arrow::Left => &mut screen_logic.left_bad_hit_beat_count,
            Arrow::Down => &mut screen_logic.down_bad_hit_beat_count,
            Arrow::Up => &mut screen_logic.up_bad_hit_beat_count,
            Arrow::Right => &mut screen_logic.right_bad_hit_beat_count,
        }
    }

    /// `[stage onPress<Arrow>]`.
    fn press(self, world: &mut World, dance_dance_execution: Entity) {
        match self {
            Arrow::Left => dance_dance_execution_logic::on_press_left(world, dance_dance_execution),
            Arrow::Down => dance_dance_execution_logic::on_press_down(world, dance_dance_execution),
            Arrow::Up => dance_dance_execution_logic::on_press_up(world, dance_dance_execution),
            Arrow::Right => dance_dance_execution_logic::on_press_right(world, dance_dance_execution),
        }
    }

    /// `[stage onRelease<Arrow>]`.
    fn release(self, world: &mut World, dance_dance_execution: Entity) {
        match self {
            Arrow::Left => dance_dance_execution_logic::on_release_left(world, dance_dance_execution),
            Arrow::Down => dance_dance_execution_logic::on_release_down(world, dance_dance_execution),
            Arrow::Up => dance_dance_execution_logic::on_release_up(world, dance_dance_execution),
            Arrow::Right => dance_dance_execution_logic::on_release_right(world, dance_dance_execution),
        }
    }
}

/// Sequence-item handlers implemented by `DanceDanceExecutionScreenLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "startDanceDanceExecution" => seq_start_dance_dance_execution(world, this, call),
        _ => return (pg_engine::logic::screen::BC_SCREEN_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[DanceDanceExecutionScreenLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&DANCE_DANCE_EXECUTION_SCREEN_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[DanceDanceExecutionScreenLogic initWithDisplayObject:]` @ 0x1000351f0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(DanceDanceExecutionScreenLogic::default());
    screen::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().dance_dance_execution_screen_logic = Some(this);

    // `getLogic:` has no side effects, so the parts are looked up in the original's order and
    // stored together.
    let parts = DanceDanceExecutionScreenLogic {
        dance_bar_logic: table_logic(world, logic_table::DANCE_BAR),
        left_overlay_logic: table_logic(world, logic_table::DANCE_LEFT_OVERLAY),
        down_overlay_logic: table_logic(world, logic_table::DANCE_DOWN_OVERLAY),
        up_overlay_logic: table_logic(world, logic_table::DANCE_UP_OVERLAY),
        right_overlay_logic: table_logic(world, logic_table::DANCE_RIGHT_OVERLAY),
        left_feedback_logic: table_logic(world, logic_table::DANCE_LEFT_FEEDBACK),
        down_feedback_logic: table_logic(world, logic_table::DANCE_DOWN_FEEDBACK),
        up_feedback_logic: table_logic(world, logic_table::DANCE_UP_FEEDBACK),
        right_feedback_logic: table_logic(world, logic_table::DANCE_RIGHT_FEEDBACK),
        left_button_logic: table_logic(world, logic_table::LEFT_BUTTON),
        down_button_logic: table_logic(world, logic_table::DOWN_BUTTON),
        up_button_logic: table_logic(world, logic_table::UP_BUTTON),
        right_button_logic: table_logic(world, logic_table::RIGHT_BUTTON),
        symbol_logic_array: Vec::with_capacity(5),
        ..DanceDanceExecutionScreenLogic::default()
    };
    *world.obj_mut::<DanceDanceExecutionScreenLogic>(this) = parts;

    logic::start_game_frame(world, this);
    screen::set_screen_layer_logic(world, this, 0, Some(this));
}

/// `-[DanceDanceExecutionScreenLogic setDanceDanceExecutionScreenLogicListener:]` @ 0x100035524
pub fn set_dance_dance_execution_screen_logic_listener(world: &mut World, this: Entity, dance_dance_execution_screen_logic_listener: Entity) {
    let listener = Some(dance_dance_execution_screen_logic_listener);
    world.obj_mut::<DanceDanceExecutionScreenLogic>(this).dance_dance_execution_screen_logic_listener = listener;
}

/// `-[DanceDanceExecutionScreenLogic createSymbolLogicAtXPos:]` @ 0x100035580
///
/// A new `DanceSymbol` model above the top of the screen, put into the screen's group just
/// before the screen's own model (so it is drawn under it). The model's only owners are then
/// its group and the returned logic.
pub fn create_symbol_logic_at_x_pos(world: &mut World, this: Entity, create_symbol_logic_at_x_pos: f32) -> Option<Entity> {
    let symbol_model = display::multi_model::spawn(world, Some("DanceSymbol"));
    let symbol_logic = logic::multi_model::spawn(world, symbol_model);

    let display_object = logic::display_object(world, this);
    display::insert_before(world, display_object, Some(symbol_model));

    let mut model = display::object_mut(world, symbol_model);
    model.pos.x = create_symbol_logic_at_x_pos;
    model.pos.y = SYMBOL_START_Y_POS;
    model.pos.z = SPRITE_Z;

    Some(symbol_logic)
}

/// `-[DanceDanceExecutionScreenLogic manageDanceDanceSymbols:]` @ 0x100035684
///
/// Advances the beat clock, then walks the ring: scrolls the active beats (judging misses and
/// the hittable beat), retires the beats that scrolled off and starts the queued ones that are
/// due. Once nothing is queued, the next chart segment is queued.
pub fn manage_dance_dance_symbols(world: &mut World, this: Entity, _dt: f32) {
    let now = cf_absolute_time_get_current(world);
    let screen_logic = world.obj::<DanceDanceExecutionScreenLogic>(this);
    let dance_seconds = (now - screen_logic.start_dance_time) + screen_logic.timing_fudge_factor;
    let beats_since_first_target = (dance_seconds + -INTRO_SECONDS) / SECONDS_PER_BEAT;

    let active_beat_count = {
        let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
        screen_logic.active_beat_count = beats_since_first_target as f32;
        screen_logic.active_dance_dance_beat_node_def = None;
        screen_logic.active_beat_count += SCROLL_BEATS;
        screen_logic.active_beat_count
    };
    if active_beat_count < 0.0 {
        return;
    }

    {
        let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
        let alpha_step = (screen_logic.symbol_target_alpha - screen_logic.symbol_alpha) * 0.5;
        screen_logic.symbol_alpha += alpha_step;
    }

    let (head, tail) = beat_ring_window(world);
    if head != tail {
        let mut beat_queued = false;
        let mut index = head;
        loop {
            let still_queued = manage_beat(world, this, index);
            if still_queued {
                beat_queued = true;
            }

            index = (index + 1) % BEAT_RING_SIZE;
            let (_, tail) = beat_ring_window(world);
            if index == tail {
                break;
            }
        }

        if beat_queued {
            return;
        }
    }

    queue_next_dance_segment(world, this);
}

/// `-[DanceDanceExecutionScreenLogic startDanceDanceExecution:::]` @ 0x1000360f4
pub fn seq_start_dance_dance_execution(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
    screen_logic.active_segment_beat_count = 0;
    screen_logic.dance_dance_execution_active = true;
    ItemResult::Done
}

/// `-[DanceDanceExecutionScreenLogic onGameFrame:]` @ 0x100036118
///
/// Starts `DanceLoop2` once the intro is over (and restarts it whenever it stops), runs the
/// symbols and eases the doom bar towards its target.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    let screen_logic = world.obj::<DanceDanceExecutionScreenLogic>(this);
    let active = screen_logic.dance_dance_execution_active;
    let game_over = screen_logic.game_over;
    if shutdown || !active {
        return;
    }

    if !game_over {
        play_dance_loop(world, this);
    }

    manage_dance_dance_symbols(world, this, dt);

    let (groove_percent, dance_bar_logic) = {
        let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
        let groove_step = (screen_logic.target_groove_percent - screen_logic.groove_percent) * 0.5;
        screen_logic.groove_percent += groove_step;
        (screen_logic.groove_percent, screen_logic.dance_bar_logic)
    };
    if let Some(dance_bar_logic) = dance_bar_logic {
        let bar_animation = logic::animation(world, dance_bar_logic);
        animation::set_frame_percent(world, bar_animation, groove_percent);
    }
}

/// `-[DanceDanceExecutionScreenLogic onDanceIntroStarted]` @ 0x1000362a8
pub fn on_dance_intro_started(world: &mut World, this: Entity) {
    let intro_start_time = cf_absolute_time_get_current(world);
    world.obj_mut::<DanceDanceExecutionScreenLogic>(this).intro_start_time = intro_start_time;

    let start_dance_time = cf_absolute_time_get_current(world);
    world.obj_mut::<DanceDanceExecutionScreenLogic>(this).start_dance_time = start_dance_time;
}

/// `-[DanceDanceExecutionScreenLogic openDanceDanceExecution]` @ 0x1000362e4
///
/// Opens the screen with a fresh game; the chart starts with `startDanceDanceExecution:::`.
pub fn open_dance_dance_execution(world: &mut World, this: Entity) {
    remove_symbols(world, this);
    screen::open_screen(world, this);

    let display_link_count = world.resource::<BcView>().display_link_count;
    let listener = {
        let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
        screen_logic.timing_fudge_factor = 0.0;
        if display_link_count == 2 {
            screen_logic.timing_fudge_factor = HALF_RATE_TIMING_FUDGE;
        }

        screen_logic.symbol_alpha = 1.0;
        screen_logic.symbol_target_alpha = 1.0;
        screen_logic.left_touch = None;
        screen_logic.down_touch = None;
        screen_logic.up_touch = None;
        screen_logic.right_touch = None;
        screen_logic.game_over = false;
        screen_logic.loop_started = false;
        screen_logic.dance_group_index = 0;
        screen_logic.groove_percent = 0.0;
        screen_logic.target_groove_percent = 0.0;
        screen_logic.dance_dance_execution_active = false;
        screen_logic.trigger_death_warning = false;
        screen_logic.streak_count = 0;
        screen_logic.left_miss_beat_count = 0.0;
        screen_logic.down_miss_beat_count = 0.0;
        screen_logic.up_miss_beat_count = 0.0;
        screen_logic.right_miss_beat_count = 0.0;
        screen_logic.left_bad_hit_beat_count = 0.0;
        screen_logic.down_bad_hit_beat_count = 0.0;
        screen_logic.up_bad_hit_beat_count = 0.0;
        screen_logic.right_bad_hit_beat_count = 0.0;
        screen_logic.dance_score = 0;
        screen_logic.dance_dance_execution_screen_logic_listener
    };

    if let Some(listener) = listener {
        crate::controller::on_dance_dance_execution_score(world, listener, 0);
    }
    screen::set_screen_layer_behavior(world, this, 0, Some("ShowDanceDanceExecutionScreen"));

    for sound_id in DANCE_SOUND_IDS {
        sound::preload_sound(world, sound_id);
    }

    for arrow in ARROWS {
        let screen_logic = world.obj::<DanceDanceExecutionScreenLogic>(this);
        let overlay_logic = arrow.overlay_logic(screen_logic);
        set_logic_behavior(world, overlay_logic, arrow.init_behavior());
    }
}

/// `-[DanceDanceExecutionScreenLogic removeSymbols]` @ 0x1000366cc
///
/// Releases every symbol in the ring and empties it; the game stops.
pub fn remove_symbols(world: &mut World, this: Entity) {
    let (head, tail) = beat_ring_window(world);
    if head != tail {
        let mut index = head;
        loop {
            for arrow in ARROWS {
                let symbol_logic = beat(world, index).symbol_logics[arrow.index()];
                if symbol_logic.is_none() {
                    continue;
                }

                release_symbol_logic(world, symbol_logic);
                beat_mut(world, index).symbol_logics[arrow.index()] = None;
            }

            index = (index + 1) % BEAT_RING_SIZE;
            let (_, tail) = beat_ring_window(world);
            if index == tail {
                break;
            }
        }
    }

    world.obj_mut::<DanceDanceExecutionScreenLogic>(this).dance_dance_execution_active = false;

    let mut statics = statics_mut(world);
    statics.beat_ring_head = 0;
    statics.beat_ring_tail = 0;
}

/// `-[DanceDanceExecutionScreenLogic closeScreen]` @ 0x100036894
///
/// The game is lost: the symbols fade out while the screen closes.
pub fn close_screen(world: &mut World, this: Entity) {
    end_dance_dance_execution(world, this, "CloseDanceDanceExecutionScreen");
}

/// `-[DanceDanceExecutionScreenLogic quitScreen]` @ 0x100036a64
///
/// The pygmy left the stage: the symbols fade out while the screen closes.
pub fn quit_screen(world: &mut World, this: Entity) {
    end_dance_dance_execution(world, this, "QuitDanceDanceExecutionScreen");
}

/// `-[DanceDanceExecutionScreenLogic closeScreenNow]` @ 0x100036c34
pub fn close_screen_now(world: &mut World, this: Entity) {
    let listener = world.obj::<DanceDanceExecutionScreenLogic>(this).dance_dance_execution_screen_logic_listener;
    let dance_dance_execution = library::logic(world, logic_table::DANCE_DANCE_EXECUTION);
    if let Some(listener) = listener {
        crate::controller::on_dance_dance_execution_stop(world, listener, dance_dance_execution);
    }

    remove_symbols(world, this);
    screen::close_screen_now(world, this);
}

/// `-[DanceDanceExecutionScreenLogic updateGroovePercent:]` @ 0x100036ce0
///
/// Moves the doom meter. Hits score and grow the streak (cheers at 30, 60 and 120), anything
/// else ends it; past 0.6 the warning plays once; a full meter ends the game unless the stage
/// is empty. The pygmy's face follows the meter.
pub fn update_groove_percent(world: &mut World, this: Entity, update_groove_percent: f32) {
    let groove_delta = update_groove_percent;
    world.obj_mut::<DanceDanceExecutionScreenLogic>(this).target_groove_percent += groove_delta;

    if groove_delta < 0.0 {
        let (dance_score, listener) = {
            let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
            screen_logic.streak_count = screen_logic.streak_count.wrapping_add(1);
            let points = trunc_i32(groove_delta * POINTS_PER_GROOVE);
            screen_logic.dance_score = screen_logic.dance_score.wrapping_add(points);
            (screen_logic.dance_score, screen_logic.dance_dance_execution_screen_logic_listener)
        };
        if let Some(listener) = listener {
            crate::controller::on_dance_dance_execution_score(world, listener, dance_score);
        }
    } else {
        world.obj_mut::<DanceDanceExecutionScreenLogic>(this).streak_count = 0;
    }

    let mut streak_count = world.obj::<DanceDanceExecutionScreenLogic>(this).streak_count;
    if streak_count == 30 {
        sound::play_sound(world, "DanceFeedbackGreat");
        streak_count = world.obj::<DanceDanceExecutionScreenLogic>(this).streak_count;
    }
    if streak_count == 60 {
        sound::play_sound(world, "DanceFeedbackAwesome");
        streak_count = world.obj::<DanceDanceExecutionScreenLogic>(this).streak_count;
    }
    if streak_count == 120 {
        sound::play_sound(world, "DanceFeedbackYouRock");
    }

    let screen_logic = world.obj::<DanceDanceExecutionScreenLogic>(this);
    let warn = screen_logic.target_groove_percent > DEATH_WARNING_GROOVE && !screen_logic.trigger_death_warning;
    if warn {
        world.obj_mut::<DanceDanceExecutionScreenLogic>(this).trigger_death_warning = true;
        sound::play_sound(world, "DanceFeedbackLookout");
    }

    let dance_dance_execution = library::logic(world, logic_table::DANCE_DANCE_EXECUTION);
    let target_groove_percent = world.obj::<DanceDanceExecutionScreenLogic>(this).target_groove_percent;
    if target_groove_percent > 1.0 {
        world.obj_mut::<DanceDanceExecutionScreenLogic>(this).target_groove_percent = 1.0;

        let game_over = world.obj::<DanceDanceExecutionScreenLogic>(this).game_over;
        if !game_over && !dance_dance_execution_logic::is_available(world, dance_dance_execution) {
            world.obj_mut::<DanceDanceExecutionScreenLogic>(this).game_over = true;
            dance_dance_execution_logic::game_over(world, dance_dance_execution);
            close_screen(world, this);
        }
    }

    let mut target_groove_percent = world.obj::<DanceDanceExecutionScreenLogic>(this).target_groove_percent;
    if target_groove_percent < 0.0 {
        let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
        screen_logic.trigger_death_warning = false;
        screen_logic.target_groove_percent = 0.0;
        target_groove_percent = screen_logic.target_groove_percent;
    }

    let active_beat_count = world.obj::<DanceDanceExecutionScreenLogic>(this).active_beat_count;
    let cocky = target_groove_percent <= 0.0 && active_beat_count > COCKY_BEAT_COUNT;
    let dance_face = if cocky {
        DANCE_FACE_COCKY
    } else if target_groove_percent > DEATH_WARNING_GROOVE {
        DANCE_FACE_BAD
    } else {
        DANCE_FACE_GOOD
    };
    dance_dance_execution_logic::set_dance_face(world, dance_dance_execution, dance_face);
}

/// `-[DanceDanceExecutionScreenLogic onButtonPress:touch:]` @ 0x100037000
///
/// The first arrow button that is pressed and not already held takes the touch, makes the
/// stage dance and is judged.
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, touch: Entity) {
    if world.obj::<DanceDanceExecutionScreenLogic>(this).game_over {
        return;
    }

    let dance_dance_execution = library::logic(world, logic_table::DANCE_DANCE_EXECUTION);
    for arrow in ARROWS {
        let screen_logic = world.obj::<DanceDanceExecutionScreenLogic>(this);
        let is_button = arrow.button_logic(screen_logic) == Some(button);
        let is_free = arrow.touch(screen_logic).is_none();
        if !is_button || !is_free {
            continue;
        }

        let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
        *arrow.touch_mut(&mut screen_logic) = Some(touch);

        arrow.press(world, dance_dance_execution);
        judge_arrow_press(world, this, arrow);
        return;
    }
}

/// `-[DanceDanceExecutionScreenLogic onButtonRelease:touch:]` @ 0x1000377f4
pub fn on_button_release(world: &mut World, this: Entity, button: Entity, touch: Entity) {
    let dance_dance_execution = library::logic(world, logic_table::DANCE_DANCE_EXECUTION);
    for arrow in ARROWS {
        let screen_logic = world.obj::<DanceDanceExecutionScreenLogic>(this);
        let is_button = arrow.button_logic(screen_logic) == Some(button);
        let is_holder = arrow.touch(screen_logic) == Some(touch);
        if !is_button || !is_holder {
            continue;
        }

        let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
        *arrow.touch_mut(&mut screen_logic) = None;

        arrow.release(world, dance_dance_execution);
        return;
    }
}

/// `-[DanceDanceExecutionScreenLogic onButtonReleaseOutside:touch:]` @ 0x10003793c
pub fn on_button_release_outside(world: &mut World, this: Entity, button: Entity, touch: Entity) {
    on_button_release(world, this, button, touch);
}

/// `+[DanceDanceExecutionScreenLogic instance]` @ 0x1000351e4
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().dance_dance_execution_screen_logic
}

/// `[[BCLibrary instance] getLogic:index]`, for an ivar.
fn table_logic(world: &World, index: usize) -> Option<Entity> {
    let logic = library::logic(world, index);
    Some(logic)
}

/// `[receiver setBehavior:behavior]` on an ivar that may be nil.
fn set_logic_behavior(world: &mut World, receiver: Option<Entity>, behavior: &str) {
    if let Some(receiver) = receiver {
        logic::set_behavior(world, receiver, Some(behavior));
    }
}

/// The charts of `dance-beats.json` (the group list at `0x10074f818`), parsed once.
fn dance_groups() -> &'static [DanceGroup] {
    static DANCE_BEATS: OnceLock<DanceBeatsFile> = OnceLock::new();

    let dance_beats = DANCE_BEATS.get_or_init(load_dance_beats);
    dance_beats.groups.as_slice()
}

fn load_dance_beats() -> DanceBeatsFile {
    match pg_assets::data::dance_beats::load() {
        Ok(dance_beats) => dance_beats,
        Err(error) => panic!("manageDanceDanceSymbols: {error}"),
    }
}

/// The beat ring, created all zero on first use (the original's statics).
fn statics_mut(world: &mut World) -> Mut<'_, DanceDanceExecutionScreenLogicStatics> {
    world.get_resource_or_insert_with(DanceDanceExecutionScreenLogicStatics::default)
}

/// The ring's head and tail indices.
fn beat_ring_window(world: &World) -> (i32, i32) {
    match world.get_resource::<DanceDanceExecutionScreenLogicStatics>() {
        Some(statics) => (statics.beat_ring_head, statics.beat_ring_tail),
        None => (0, 0),
    }
}

/// A copy of ring entry `index`.
fn beat(world: &World, index: i32) -> DanceDanceBeat {
    match world.get_resource::<DanceDanceExecutionScreenLogicStatics>() {
        Some(statics) => statics.beat_ring[index as usize],
        None => DanceDanceBeat::default(),
    }
}

/// Ring entry `index`, to change in place.
fn beat_mut(world: &mut World, index: i32) -> Mut<'_, DanceDanceBeat> {
    let statics = statics_mut(world);
    statics.map_unchanged(|statics| &mut statics.beat_ring[index as usize])
}

/// One ring entry of `manageDanceDanceSymbols:`. Returns whether the beat is still queued.
fn manage_beat(world: &mut World, this: Entity, index: i32) -> bool {
    let state = beat(world, index).state;
    match state {
        BEAT_ACTIVE | BEAT_PAST => {
            scroll_beat(world, this, index, state);
            false
        }
        BEAT_REMOVE => {
            retire_beat(world, index);
            false
        }
        BEAT_QUEUED => {
            start_beat_when_due(world, this, index);
            true
        }
        _ => false,
    }
}

/// An active or past beat: its symbols scroll; an active beat is missed half a beat after
/// its target (`BEAT_PAST`) and is the hittable beat within half a beat of it; a past beat
/// below the screen is marked for removal.
fn scroll_beat(world: &mut World, this: Entity, index: i32, state: i32) {
    let active_beat_count = world.obj::<DanceDanceExecutionScreenLogic>(this).active_beat_count;
    let start_beat = beat(world, index).start_beat as f32;
    let scrolled_beats = active_beat_count - start_beat;
    let symbol_y_pos = (scrolled_beats / SCROLL_BEATS) * SYMBOL_SCROLL_DISTANCE + SYMBOL_START_Y_POS;
    beat_mut(world, index).symbol_y_pos = symbol_y_pos;

    let mut check_scrolled_off = state == BEAT_PAST;
    if state == BEAT_ACTIVE && !world.obj::<DanceDanceExecutionScreenLogic>(this).game_over {
        let beat_offset = active_beat_count - (start_beat + SCROLL_BEATS);
        if beat_offset > HIT_WINDOW_BEATS {
            for arrow in ARROWS {
                report_missed_arrow(world, this, index, arrow);
            }
            beat_mut(world, index).state = BEAT_PAST;
            check_scrolled_off = true;
        } else if beat_offset > -HIT_WINDOW_BEATS {
            let offset_magnitude = f64::from(beat_offset.abs());
            let beat_hit_percent = (offset_magnitude * -2.0 + 1.0) as f32;

            let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
            screen_logic.beat_hit_percent = beat_hit_percent;
            screen_logic.active_dance_dance_beat_node_def = Some(index as usize);
        }
    }

    if check_scrolled_off {
        let symbol_y_pos = beat(world, index).symbol_y_pos;
        if symbol_y_pos < SYMBOL_REMOVE_Y_POS {
            beat_mut(world, index).state = BEAT_REMOVE;
        }
    }

    for arrow in ARROWS {
        place_symbol(world, this, index, arrow);
    }
}

/// An arrow of a beat that scrolled past its target unhit: a miss, unless the arrow was pressed
/// wrongly less than two beats ago.
fn report_missed_arrow(world: &mut World, this: Entity, index: i32, arrow: Arrow) {
    let missed_beat = beat(world, index);
    let has_arrow = missed_beat.arrows[arrow.index()];
    let hit = missed_beat.hits[arrow.index()];
    if !has_arrow || hit {
        return;
    }

    let screen_logic = world.obj::<DanceDanceExecutionScreenLogic>(this);
    let beats_since_bad_hit = screen_logic.active_beat_count - arrow.bad_hit_beat_count(screen_logic);
    let feedback_logic = arrow.feedback_logic(screen_logic);
    let overlay_logic = arrow.overlay_logic(screen_logic);
    let counts_as_miss = beats_since_bad_hit > MISS_AFTER_BAD_HIT_BEATS;
    if !counts_as_miss {
        return;
    }

    set_logic_behavior(world, feedback_logic, "DanceDanceFeedbackMiss");
    set_logic_behavior(world, overlay_logic, arrow.miss_behavior());
    update_groove_percent(world, this, MISS_GROOVE_DELTA);

    let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
    let active_beat_count = screen_logic.active_beat_count;
    *arrow.miss_beat_count_mut(&mut screen_logic) = active_beat_count;
}

/// Fades and moves one arrow's symbol with its beat.
fn place_symbol(world: &mut World, this: Entity, index: i32, arrow: Arrow) {
    let scrolled_beat = beat(world, index);
    if !scrolled_beat.arrows[arrow.index()] {
        return;
    }
    let Some(symbol_logic) = scrolled_beat.symbol_logics[arrow.index()] else {
        // The symbol was made when the beat started; a nil one would crash the original.
        return;
    };

    let symbol_alpha = world.obj::<DanceDanceExecutionScreenLogic>(this).symbol_alpha;
    let symbol_display_object = logic::display_object(world, symbol_logic);
    let mut symbol_display = display::object_mut(world, symbol_display_object);
    symbol_display.set_alpha(symbol_alpha);
    symbol_display.pos.y = scrolled_beat.symbol_y_pos;
}

/// A beat marked for removal: the ring's head moves on (the beats retire in order) and the
/// beat's symbols are released.
fn retire_beat(world: &mut World, index: i32) {
    {
        let mut statics = statics_mut(world);
        statics.beat_ring_head = (statics.beat_ring_head + 1) % BEAT_RING_SIZE;
    }

    for arrow in ARROWS {
        let retired_beat = beat(world, index);
        if !retired_beat.arrows[arrow.index()] {
            continue;
        }

        let symbol_logic = retired_beat.symbol_logics[arrow.index()];
        release_symbol_logic(world, symbol_logic);
        beat_mut(world, index).symbol_logics[arrow.index()] = None;
    }
}

/// A queued beat whose start beat has come: it becomes active with a new symbol per arrow
/// above the top of the screen.
fn start_beat_when_due(world: &mut World, this: Entity, index: i32) {
    let start_beat = beat(world, index).start_beat as f32;
    let active_beat_count = world.obj::<DanceDanceExecutionScreenLogic>(this).active_beat_count;
    let due = start_beat <= active_beat_count;
    if !due {
        return;
    }

    {
        let mut started_beat = beat_mut(world, index);
        started_beat.state = BEAT_ACTIVE;
        started_beat.symbol_y_pos = SYMBOL_START_Y_POS;
    }

    for arrow in ARROWS {
        if !beat(world, index).arrows[arrow.index()] {
            continue;
        }

        let symbol_logic = create_symbol_logic_at_x_pos(world, this, arrow.symbol_x_pos());
        beat_mut(world, index).symbol_logics[arrow.index()] = symbol_logic;
        set_logic_behavior(world, symbol_logic, arrow.symbol_behavior());
    }
}

/// Queues the beats of the next chart segment, each starting where the previous ones end. A
/// plain group gives its first segment and moves on to the next group; the last group is
/// random and repeats forever.
fn queue_next_dance_segment(world: &mut World, this: Entity) {
    let dance_group_index = world.obj::<DanceDanceExecutionScreenLogic>(this).dance_group_index;
    let dance_group = &dance_groups()[dance_group_index as usize];

    let segment_index = if dance_group.random_segment {
        let segment_count = dance_group.segments.len() as u32;
        let random_index = world.resource_mut::<Random>().arc4random_mod(segment_count);
        random_index as usize
    } else {
        world.obj_mut::<DanceDanceExecutionScreenLogic>(this).dance_group_index = dance_group_index + 1;
        0
    };
    let dance_segment = &dance_group.segments[segment_index];

    for chart_beat in &dance_segment.beats {
        let start_beat = world.obj::<DanceDanceExecutionScreenLogic>(this).active_segment_beat_count;
        let queued_beat = DanceDanceBeat::queued(chart_beat, start_beat);
        {
            let mut statics = statics_mut(world);
            let tail = statics.beat_ring_tail as usize;
            statics.beat_ring[tail] = queued_beat;
        }

        let next_start_beat = start_beat.wrapping_add(chart_beat.duration);
        world.obj_mut::<DanceDanceExecutionScreenLogic>(this).active_segment_beat_count = next_start_beat;

        let mut statics = statics_mut(world);
        statics.beat_ring_tail = (statics.beat_ring_tail + 1) % BEAT_RING_SIZE;
    }
}

/// `[[symbol displayObject] removeFromDisplayGroup]; [symbol shutdown]; [symbol release]`.
///
/// The ring held the symbol's last reference (`shutdown` breaks its retain cycle with its
/// animation), and the symbol held the last one of its model, which has just left the tree.
fn release_symbol_logic(world: &mut World, symbol_logic: Option<Entity>) {
    let Some(symbol_logic) = symbol_logic else {
        return;
    };

    let symbol_model = logic::display_object(world, symbol_logic);
    display::remove_from_display_group(world, symbol_model);
    logic::shutdown(world, symbol_logic);

    logic::dealloc(world, symbol_logic);
    world.despawn(symbol_model);
}

/// `onGameFrame:`'s background music: `DanceLoop2` starts 7.5 s after the intro started and
/// starts again whenever it has stopped.
fn play_dance_loop(world: &mut World, this: Entity) {
    let dance_loop = sound::sound_by_id(world, "DanceLoop2");
    let loop_playing = match dance_loop {
        Some(dance_loop) => sound::is_playing(world, dance_loop),
        None => false,
    };
    let now = cf_absolute_time_get_current(world);

    let screen_logic = world.obj::<DanceDanceExecutionScreenLogic>(this);
    let play_loop = if screen_logic.loop_started {
        !loop_playing
    } else {
        let intro_seconds = now - screen_logic.intro_start_time;
        let intro_over = intro_seconds > INTRO_SECONDS;
        if intro_over {
            world.obj_mut::<DanceDanceExecutionScreenLogic>(this).loop_started = true;
        }
        intro_over
    };

    if play_loop && let Some(dance_loop) = dance_loop {
        sound::play(world, dance_loop);
    }
}

/// The rest of `onButtonPress:touch:` for the pressed arrow: a hit on the hittable beat
/// (perfect, good or ok by timing), or a wrong press more than half a beat after the arrow's
/// last miss.
fn judge_arrow_press(world: &mut World, this: Entity, arrow: Arrow) {
    let screen_logic = world.obj::<DanceDanceExecutionScreenLogic>(this);
    let hittable_beat = screen_logic.active_dance_dance_beat_node_def;
    let feedback_logic = arrow.feedback_logic(screen_logic);
    let overlay_logic = arrow.overlay_logic(screen_logic);

    if let Some(beat_index) = hittable_beat {
        let beat_index = beat_index as i32;
        if beat(world, beat_index).arrows[arrow.index()] {
            beat_mut(world, beat_index).hits[arrow.index()] = true;

            let beat_hit_percent = world.obj::<DanceDanceExecutionScreenLogic>(this).beat_hit_percent;
            if beat_hit_percent > PERFECT_HIT_PERCENT {
                set_logic_behavior(world, feedback_logic, "DanceDanceFeedbackPerfect");
                set_logic_behavior(world, overlay_logic, arrow.perfect_behavior());
                update_groove_percent(world, this, PERFECT_GROOVE_DELTA);
                return;
            }

            let groove_delta = if beat_hit_percent > GOOD_HIT_PERCENT {
                set_logic_behavior(world, feedback_logic, "DanceDanceFeedbackGood");
                GOOD_GROOVE_DELTA
            } else {
                set_logic_behavior(world, feedback_logic, "DanceDanceFeedbackOk");
                OK_GROOVE_DELTA
            };
            update_groove_percent(world, this, groove_delta);
            set_logic_behavior(world, overlay_logic, arrow.correct_behavior());
            return;
        }
    }

    let screen_logic = world.obj::<DanceDanceExecutionScreenLogic>(this);
    let beats_since_miss = screen_logic.active_beat_count - arrow.miss_beat_count(screen_logic);
    let counts_as_bad_hit = beats_since_miss > BAD_HIT_AFTER_MISS_BEATS;
    if !counts_as_bad_hit {
        return;
    }

    set_logic_behavior(world, feedback_logic, "DanceDanceFeedbackMiss");
    set_logic_behavior(world, overlay_logic, arrow.incorrect_behavior());
    update_groove_percent(world, this, MISS_GROOVE_DELTA);

    let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
    let active_beat_count = screen_logic.active_beat_count;
    *arrow.bad_hit_beat_count_mut(&mut screen_logic) = active_beat_count;
}

/// The shared body of `closeScreen` and `quitScreen`: the dance sounds are unloaded, the game
/// is over, the symbols fade out and the screen plays its closing behaviour.
fn end_dance_dance_execution(world: &mut World, this: Entity, behavior: &str) {
    for sound_id in DANCE_SOUND_IDS {
        sound::unload_sound(world, sound_id);
    }

    {
        let mut screen_logic = world.obj_mut::<DanceDanceExecutionScreenLogic>(this);
        screen_logic.game_over = true;
        screen_logic.symbol_target_alpha = 0.0;
    }

    screen::set_screen_layer_behavior(world, this, 0, Some(behavior));
}
