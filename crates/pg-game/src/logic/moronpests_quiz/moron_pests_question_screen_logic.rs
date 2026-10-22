//! `MoronPestsQuestionScreenLogic` (superclass `BCScreenLogic`).

use std::cmp::Ordering;

use crate::prelude::*;

use crate::logic::moronpests_chase::moron_snake_logic;
use crate::logic::moronpests_quiz::moron_bee_logic;
use crate::logic::moronpests_quiz::moron_duck_logic;
use crate::logic_table;
use pg_engine::display;
use pg_engine::display::text;
use pg_engine::library::{self, Library};
use pg_engine::logic;
use pg_engine::logic::screen;

/// `MoronPestsQuestionScreenLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MoronPestsQuestionScreenLogic {
    /// `mMoronPestsQuestionScreenLogicListener`: `@"<MoronPestsQuestionScreenLogicListener>"`
    pub moron_pests_question_screen_logic_listener: Option<Entity>,

    /// `mMoronPestsQuestionType`: `i`
    pub moron_pests_question_type: i32,

    /// `mColorQuestionType`: `i`
    pub color_question_type: i32,

    /// `mSynonymQuestionType`: `i`
    pub synonym_question_type: i32,

    /// `mBeeQuestionType`: `i`
    pub bee_question_type: i32,

    /// `mDuckQuestionType`: `i`
    pub duck_question_type: i32,

    /// `mDifficulty`: `f`
    pub difficulty: f32,

    /// `mbDontAnswerActive`: `B`
    pub dont_answer_active: bool,

    /// `mDontAnswerTimer`: `f`
    pub dont_answer_timer: f32,

    /// `mMoronPestsPygmyLogic`: `@"PygmyLogic"`
    pub moron_pests_pygmy_logic: Option<Entity>,

    /// `mRandomizeArray`: `@"NSMutableArray"` of colour names and synonyms. The duck question
    /// also passes its `NSNumber` eye scales through it; the port keeps those in a local.
    pub randomize_array: Vec<&'static str>,

    /// `mRandomizeScaleArray`: `@"NSMutableArray"` of `NSNumber` floats.
    pub randomize_scale_array: Vec<f32>,

    /// `mSynonymLogicArray`: `@"NSMutableArray"`
    pub synonym_logic_array: Vec<Entity>,

    /// `mColorLogicArray`: `@"NSMutableArray"`
    pub color_logic_array: Vec<Entity>,

    /// `mColorOverlayLogicArray`: `@"NSMutableArray"`
    pub color_overlay_logic_array: Vec<Entity>,

    /// `mMoronBeeLogicArray`: `@"NSMutableArray"`
    pub moron_bee_logic_array: Vec<Entity>,

    /// `mMoronDuckLogicArray`: `@"NSMutableArray"`
    pub moron_duck_logic_array: Vec<Entity>,

    /// `mButtonColorsArray`: `@"NSMutableArray"` of colour names not on a button.
    pub button_colors_array: Vec<&'static str>,

    /// `mActiveButtonColorsArray`: `@"NSMutableArray"` of the colour names on the four buttons.
    pub active_button_colors_array: Vec<&'static str>,

    /// `mActiveMoronBeeLogicArray`: `@"NSMutableArray"`
    pub active_moron_bee_logic_array: Vec<Entity>,

    /// `mActiveMoronDuckLogicArray`: `@"NSMutableArray"`
    pub active_moron_duck_logic_array: Vec<Entity>,

    /// `mCorrectSynonymIndex`: `i`
    pub correct_synonym_index: i32,

    /// `mCorrectColorArray`: `[20@"NSString"]`
    pub correct_color_array: [Option<&'static str>; 20],

    /// `mSelectedButtonIndex`: `i`
    pub selected_button_index: i32,

    /// `mColorTapCount`: `i`
    pub color_tap_count: i32,

    /// `mColorTapIndex`: `i`
    pub color_tap_index: i32,

    /// `mPressedButtonLogic`: `@"BCButtonLogic"`
    pub pressed_button_logic: Option<Entity>,

    /// `mSynonym1Logic`: `@"MultiModelLogic"`
    pub synonym1_logic: Option<Entity>,

    /// `mSynonym2Logic`: `@"MultiModelLogic"`
    pub synonym2_logic: Option<Entity>,

    /// `mSynonym1ButtonLogic`: `@"BCButtonLogic"`
    pub synonym1_button_logic: Option<Entity>,

    /// `mSynonym2ButtonLogic`: `@"BCButtonLogic"`
    pub synonym2_button_logic: Option<Entity>,

    /// `mColor1Logic`: `@"MultiModelLogic"`
    pub color1_logic: Option<Entity>,

    /// `mColor2Logic`: `@"MultiModelLogic"`
    pub color2_logic: Option<Entity>,

    /// `mColor3Logic`: `@"MultiModelLogic"`
    pub color3_logic: Option<Entity>,

    /// `mColor4Logic`: `@"MultiModelLogic"`
    pub color4_logic: Option<Entity>,

    /// `mColorOverlay1Logic`: `@"MultiModelLogic"`
    pub color_overlay1_logic: Option<Entity>,

    /// `mColorOverlay2Logic`: `@"MultiModelLogic"`
    pub color_overlay2_logic: Option<Entity>,

    /// `mColorOverlay3Logic`: `@"MultiModelLogic"`
    pub color_overlay3_logic: Option<Entity>,

    /// `mColorOverlay4Logic`: `@"MultiModelLogic"`
    pub color_overlay4_logic: Option<Entity>,

    /// `mColor1ButtonLogic`: `@"BCButtonLogic"`
    pub color1_button_logic: Option<Entity>,

    /// `mColor2ButtonLogic`: `@"BCButtonLogic"`
    pub color2_button_logic: Option<Entity>,

    /// `mColor3ButtonLogic`: `@"BCButtonLogic"`
    pub color3_button_logic: Option<Entity>,

    /// `mColor4ButtonLogic`: `@"BCButtonLogic"`
    pub color4_button_logic: Option<Entity>,
}

/// `WeightDifficulty`: one variant of a difficulty-weighted random choice.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WeightDifficulty {
    pub weight: f32,

    /// The `mDifficulty` from which the variant can be chosen.
    pub min_difficulty: f32,
}

impl WeightDifficulty {
    pub const fn new(weight: f32, min_difficulty: f32) -> Self {
        Self { weight, min_difficulty }
    }
}

/// File-scope statics of `MoronPestsQuestionScreenLogic`.
#[derive(Resource, Default)]
pub struct MoronPestsQuestionScreenLogicStatics {
    /// `DAT_1007d1e88`: the colour-switch weights of the current colour question
    /// (`calculateColorSwitchIndices:tapCount:`). A tap whose index has weight 0 shuffles the
    /// colour buttons.
    pub color_switch_weights: [WeightDifficulty; 10],
}

pub const MORON_PESTS_QUESTION_SCREEN_LOGIC: Class = Class {
    name: "MoronPestsQuestionScreenLogic",
    superclass: Some(&pg_engine::logic::screen::BC_SCREEN_LOGIC),
    sequence_item,
    on_game_frame,
    close_screen_now,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..pg_engine::logic::screen::BC_SCREEN_LOGIC
};

/// `DAT_10029d63c`: synonym, colour, bee and duck questions.
const QUESTION_TYPE_WEIGHTS: [WeightDifficulty; 4] = [
    WeightDifficulty::new(2.0, 0.0),
    WeightDifficulty::new(2.0, 0.0),
    WeightDifficulty::new(1.0, 0.0),
    WeightDifficulty::new(1.0, 0.0),
];

/// `DAT_10029d65c`: "Press the …", "Don't choose the …", "Press anything other than the …".
const SYNONYM_QUESTION_WEIGHTS: [WeightDifficulty; 3] = [
    WeightDifficulty::new(5.0, 0.0),
    WeightDifficulty::new(1.0, 0.0),
    WeightDifficulty::new(5.0, 0.0),
];

/// `DAT_10029d674`: the twelve colour questions (see `openMoronPestsColorQuestion`).
const COLOR_QUESTION_WEIGHTS: [WeightDifficulty; 12] = [
    WeightDifficulty::new(4.0, 0.0),
    WeightDifficulty::new(1.0, 0.0),
    WeightDifficulty::new(4.0, 0.0),
    WeightDifficulty::new(4.0, 5000.0),
    WeightDifficulty::new(1.0, 5000.0),
    WeightDifficulty::new(4.0, 5000.0),
    WeightDifficulty::new(4.0, 10000.0),
    WeightDifficulty::new(4.0, 15000.0),
    WeightDifficulty::new(4.0, 20000.0),
    WeightDifficulty::new(4.0, 10000.0),
    WeightDifficulty::new(4.0, 15000.0),
    WeightDifficulty::new(4.0, 20000.0),
];

/// `DAT_10029d6d4`: right to left, left to right, top to bottom, bottom to top.
const BEE_QUESTION_WEIGHTS: [WeightDifficulty; 4] = [
    WeightDifficulty::new(1.0, 0.0),
    WeightDifficulty::new(1.0, 0.0),
    WeightDifficulty::new(1.0, 5000.0),
    WeightDifficulty::new(1.0, 5000.0),
];

/// `DAT_10029d6f4`: by size (biggest, smallest first), by eyes (biggest, smallest first).
const DUCK_QUESTION_WEIGHTS: [WeightDifficulty; 4] = [
    WeightDifficulty::new(2.0, 0.0),
    WeightDifficulty::new(2.0, 0.0),
    WeightDifficulty::new(1.0, 5000.0),
    WeightDifficulty::new(1.0, 5000.0),
];

/// `DAT_10029d714`: the initial colour-switch weights, copied into `DAT_1007d1e88`.
const COLOR_SWITCH_WEIGHTS: [WeightDifficulty; 10] = [
    WeightDifficulty::new(1.0, 0.0),
    WeightDifficulty::new(2.0, 0.0),
    WeightDifficulty::new(3.0, 0.0),
    WeightDifficulty::new(4.0, 0.0),
    WeightDifficulty::new(5.0, 0.0),
    WeightDifficulty::new(6.0, 0.0),
    WeightDifficulty::new(7.0, 0.0),
    WeightDifficulty::new(8.0, 0.0),
    WeightDifficulty::new(9.0, 0.0),
    WeightDifficulty::new(10.0, 0.0),
];

/// `mButtonColorsArray` as `initWithDisplayObject:` fills it (literals 0x100391030 –
/// 0x1003910d0).
const BUTTON_COLORS: [&str; 6] = ["Red", "Green", "Blue", "Orange", "Purple", "Yellow"];

/// The homophone pairs of the synonym question (`PTR_cf_Male_10037d7e0`).
const SYNONYM_PAIRS: [(&str, &str); 7] = [
    ("Male", "Mail"),
    ("Sun", "Son"),
    ("Fairy", "Ferry"),
    ("Bee", "B"),
    ("Pi", "Pie"),
    ("Carrot", "Karat"),
    ("Mouse", "Mousse"),
];

/// The repeat suffix of a colour in "press … then … then …", by press count − 1
/// (`PTR_cf_""_10037d7a0`).
const TAP_COUNT_SUFFIXES: [&str; 8] = [
    "",
    " twice",
    " three times",
    " four times",
    " five times",
    " six times",
    " seven times",
    " eight times",
];

/// The duck question's eye-scale ranks (`numberWithInt:` 0 – 3 in `mRandomizeArray`).
const DUCK_EYE_SCALES: [i32; 4] = [0, 1, 2, 3];

/// The duck question's body scales (`numberWithFloat:` in `mRandomizeScaleArray`).
const DUCK_SCALES: [f32; 4] = [1.1, 0.9, 0.7, 0.5];

/// The first duck's screen position (`DAT_10029d770`); each next duck is `DUCK_SPACING` right.
const DUCK_FIRST_SCREEN_POS: Vec4 = Vec4::new(30.0, 130.0, 0.0, 0.0);

/// `DAT_10029d2a4`.
const DUCK_SPACING: f32 = 80.0;

/// The bees' screen positions keep this far apart on each axis …
const BEE_MIN_AXIS_DISTANCE: f32 = 10.0;

/// … and this far apart (`DAT_10029d2ec`).
const BEE_MIN_DISTANCE: f32 = 40.0;

/// How long a "don't answer" question waits (0x40800000).
const DONT_ANSWER_TIME: f32 = 4.0;

const SYNONYM_TITLE_TEXT_ID: &str = "MoronPestsSynonymTitle";
const COLOR_TITLE_TEXT_ID: &str = "MoronPestsColorTitle";
const ANIMAL_TITLE_TEXT_ID: &str = "MoronPestsAnimalTitle";

const SHOW_ANIMAL_QUESTION_BEHAVIOR: &str = "ShowMoronPestsAnimalQuestion";
const CLOSE_SYNONYM_QUESTION_BEHAVIOR: &str = "CloseMoronPestsSynonymQuestion";
const CLOSE_COLOR_QUESTION_BEHAVIOR: &str = "CloseMoronPestsColorQuestion";
const CLOSE_ANIMAL_QUESTION_BEHAVIOR: &str = "CloseMoronPestsAnimalQuestion";
const CLOSE_ANIMAL_QUESTION_LOSE_BEHAVIOR: &str = "CloseMoronPestsAnimalQuestionLose";

const BUTTON_OVERLAY_UP: &str = "MoronPestButtonOverlayUp";
const BUTTON_OVERLAY_DOWN: &str = "MoronPestButtonOverlayDown";

/// A comparison function of `sortUsingFunction:context:` (`NSOrderedAscending` = `Less`).
type PestComparator = fn(&mut World, Entity, Entity) -> Ordering;

/// Sequence-item handlers implemented by `MoronPestsQuestionScreenLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "closeSynonymQuestionComplete" => seq_close_synonym_question_complete(world, this, call),
        "closeColorQuestionComplete" => seq_close_color_question_complete(world, this, call),
        "closeAnimalQuestionComplete" => seq_close_animal_question_complete(world, this, call),
        _ => return (pg_engine::logic::screen::BC_SCREEN_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[MoronPestsQuestionScreenLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&MORON_PESTS_QUESTION_SCREEN_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[[BCLibrary instance] getDynamicTextById:id]`.
fn dynamic_text(world: &World, id: &str) -> Option<Entity> {
    world.resource::<Library>().dynamic_text(id)
}

/// `[title setText:value]` (nothing for a nil title).
fn set_title_text(world: &mut World, title: Option<Entity>, value: &str) {
    if let Some(title) = title {
        text::set_text(world, title, Some(value));
    }
}

/// `arc4random() % count` as the original turns it into an array index: the `u32` remainder
/// (the raw value for a count of 0), truncated to `int` and sign-extended.
fn random_index(world: &mut World, count: usize) -> usize {
    let remainder = world.resource_mut::<Random>().arc4random_mod(count as u32);
    remainder as i32 as usize
}

/// A random entry of `mRandomizeArray` (`objectAtIndex:`), removed from the array.
fn take_random_randomize_entry(world: &mut World, this: Entity) -> &'static str {
    let count = world.obj::<MoronPestsQuestionScreenLogic>(this).randomize_array.len();
    let index = random_index(world, count);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).randomize_array.remove(index)
}

/// `mCorrectColorArray[0]` = a random colour of the buttons, which is returned.
fn choose_correct_color(world: &mut World, this: Entity) -> &'static str {
    let count = world.obj::<MoronPestsQuestionScreenLogic>(this).active_button_colors_array.len();
    let index = random_index(world, count);

    let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
    let color = screen.active_button_colors_array[index];
    screen.correct_color_array[0] = Some(color);
    color
}

/// `mCorrectColorArray[index]`. PORT: an index outside the array (the original reads the
/// neighbouring ivars there) reads as nil.
fn correct_color_at(screen: &MoronPestsQuestionScreenLogic, index: i32) -> Option<&'static str> {
    match usize::try_from(index) {
        Ok(slot) if slot < screen.correct_color_array.len() => screen.correct_color_array[slot],
        _ => None,
    }
}

/// `[expected isEqualToString:[mActiveButtonColorsArray objectAtIndex:mSelectedButtonIndex]]`
/// (NO for a nil `expected`).
fn is_selected_color(screen: &MoronPestsQuestionScreenLogic, expected: Option<&str>) -> bool {
    let selected_index = screen.selected_button_index as usize;
    let selected_color = screen.active_button_colors_array[selected_index];
    expected == Some(selected_color)
}

/// `DAT_1007d1e88[index].weight == 0.0`: the colour buttons shuffle after this tap. The
/// statics start zeroed. PORT: indices past the 10 entries (the original reads the next
/// globals) never occur, since a question closes on its last tap; they read as non-zero.
fn is_color_switch_tap(world: &World, index: i32) -> bool {
    let Some(statics) = world.get_resource::<MoronPestsQuestionScreenLogicStatics>() else {
        return true;
    };
    match usize::try_from(index) {
        Ok(slot) if slot < statics.color_switch_weights.len() => statics.color_switch_weights[slot].weight == 0.0,
        _ => false,
    }
}

/// Tells the listener (always the `MoronSnakeLogic`, `initLogicListeners`) how the question
/// was answered: `onMoronPestsQuestionCorrect` or `onMoronPestsQuestionIncorrect`.
fn report_answer(world: &mut World, listener: Option<Entity>, correct: bool) {
    let Some(listener) = listener else {
        return;
    };
    if correct {
        moron_snake_logic::on_moron_pests_question_correct(world, listener);
    } else {
        moron_snake_logic::on_moron_pests_question_incorrect(world, listener);
    }
}

/// `-[MoronPestsQuestionScreenLogic initWithDisplayObject:]` @ 0x10006270c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(MoronPestsQuestionScreenLogic::default());
    screen::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().moron_pests_question_screen_logic = Some(this);

    let synonym1_logic = library::logic(world, logic_table::MORON_PESTS_SYNONYM1);
    let synonym2_logic = library::logic(world, logic_table::MORON_PESTS_SYNONYM2);
    let synonym1_button_logic = library::logic(world, logic_table::SYNONYM1_BUTTON);
    let synonym2_button_logic = library::logic(world, logic_table::SYNONYM2_BUTTON);
    let color1_logic = library::logic(world, logic_table::MORON_PESTS_COLOR1);
    let color2_logic = library::logic(world, logic_table::MORON_PESTS_COLOR2);
    let color3_logic = library::logic(world, logic_table::MORON_PESTS_COLOR3);
    let color4_logic = library::logic(world, logic_table::MORON_PESTS_COLOR4);
    let color_overlay1_logic = library::logic(world, logic_table::MORON_PESTS_COLOR_OVERLAY1);
    let color_overlay2_logic = library::logic(world, logic_table::MORON_PESTS_COLOR_OVERLAY2);
    let color_overlay3_logic = library::logic(world, logic_table::MORON_PESTS_COLOR_OVERLAY3);
    let color_overlay4_logic = library::logic(world, logic_table::MORON_PESTS_COLOR_OVERLAY4);
    let color1_button_logic = library::logic(world, logic_table::COLOR1_BUTTON);
    let color2_button_logic = library::logic(world, logic_table::COLOR2_BUTTON);
    let color3_button_logic = library::logic(world, logic_table::COLOR3_BUTTON);
    let color4_button_logic = library::logic(world, logic_table::COLOR4_BUTTON);

    let bee1_logic = library::logic(world, logic_table::MORON_BEE1);
    let bee2_logic = library::logic(world, logic_table::MORON_BEE2);
    let bee3_logic = library::logic(world, logic_table::MORON_BEE3);
    let bee4_logic = library::logic(world, logic_table::MORON_BEE4);
    let bee5_logic = library::logic(world, logic_table::MORON_BEE5);

    let duck1_logic = library::logic(world, logic_table::MORON_DUCK1);
    let duck2_logic = library::logic(world, logic_table::MORON_DUCK2);
    let duck3_logic = library::logic(world, logic_table::MORON_DUCK3);
    let duck4_logic = library::logic(world, logic_table::MORON_DUCK4);

    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        screen.synonym1_logic = Some(synonym1_logic);
        screen.synonym2_logic = Some(synonym2_logic);
        screen.synonym1_button_logic = Some(synonym1_button_logic);
        screen.synonym2_button_logic = Some(synonym2_button_logic);
        screen.color1_logic = Some(color1_logic);
        screen.color2_logic = Some(color2_logic);
        screen.color3_logic = Some(color3_logic);
        screen.color4_logic = Some(color4_logic);
        screen.color_overlay1_logic = Some(color_overlay1_logic);
        screen.color_overlay2_logic = Some(color_overlay2_logic);
        screen.color_overlay3_logic = Some(color_overlay3_logic);
        screen.color_overlay4_logic = Some(color_overlay4_logic);
        screen.color1_button_logic = Some(color1_button_logic);
        screen.color2_button_logic = Some(color2_button_logic);
        screen.color3_button_logic = Some(color3_button_logic);
        screen.color4_button_logic = Some(color4_button_logic);

        screen.synonym_logic_array = vec![synonym1_logic, synonym2_logic];
        screen.color_logic_array = vec![color1_logic, color2_logic, color3_logic, color4_logic];
        screen.color_overlay_logic_array = vec![color_overlay1_logic, color_overlay2_logic, color_overlay3_logic, color_overlay4_logic];
        screen.moron_bee_logic_array = vec![bee1_logic, bee2_logic, bee3_logic, bee4_logic, bee5_logic];
        screen.moron_duck_logic_array = vec![duck1_logic, duck2_logic, duck3_logic, duck4_logic];
    }

    let duck_head1_logic = library::logic(world, logic_table::MORON_DUCK_HEAD1);
    moron_duck_logic::set_head_logic(world, duck1_logic, duck_head1_logic);
    let duck_head2_logic = library::logic(world, logic_table::MORON_DUCK_HEAD2);
    moron_duck_logic::set_head_logic(world, duck2_logic, duck_head2_logic);
    let duck_head3_logic = library::logic(world, logic_table::MORON_DUCK_HEAD3);
    moron_duck_logic::set_head_logic(world, duck3_logic, duck_head3_logic);
    let duck_head4_logic = library::logic(world, logic_table::MORON_DUCK_HEAD4);
    moron_duck_logic::set_head_logic(world, duck4_logic, duck_head4_logic);

    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        screen.active_moron_bee_logic_array = Vec::new();
        screen.active_moron_duck_logic_array = Vec::new();
        screen.randomize_array = Vec::new();
        screen.randomize_scale_array = Vec::new();
        screen.button_colors_array = BUTTON_COLORS.to_vec();
        screen.active_button_colors_array = Vec::new();
    }

    logic::start_game_frame(world, this);
    screen::set_screen_layer_logic(world, this, 0, Some(this));
}

/// `-[MoronPestsQuestionScreenLogic setMoronPestsQuestionScreenLogicListener:]` @ 0x10006310c
pub fn set_moron_pests_question_screen_logic_listener(world: &mut World, this: Entity, moron_pests_question_screen_logic_listener: Entity) {
    let listener = Some(moron_pests_question_screen_logic_listener);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).moron_pests_question_screen_logic_listener = listener;
}

/// `-[MoronPestsQuestionScreenLogic onGameFrame:]` @ 0x100063168
///
/// Runs the "don't answer" countdown; when it expires the question closes as answered.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let question_type = {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        if !screen.dont_answer_active {
            return;
        }

        let dont_answer_timer = screen.dont_answer_timer - dt;
        screen.dont_answer_timer = dont_answer_timer;
        let timer_expired = dont_answer_timer <= 0.0;
        if !timer_expired {
            return;
        }

        screen.dont_answer_active = false;
        screen.moron_pests_question_type
    };

    match question_type {
        0 => screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_SYNONYM_QUESTION_BEHAVIOR)),
        1 => screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_COLOR_QUESTION_BEHAVIOR)),
        _ => {}
    }
}

/// `-[MoronPestsQuestionScreenLogic setDifficulty:]` @ 0x100063200
pub fn set_difficulty(world: &mut World, this: Entity, difficulty: f32) {
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).difficulty = difficulty;
}

/// `-[MoronPestsQuestionScreenLogic calculateRandomWeightDifficultyIndex:size:]` @ 0x100063210
///
/// Picks one of the first `size` entries of `weights`, in proportion to its weight (in steps
/// of 0.1), among those whose minimum difficulty `mDifficulty` has reached; 0 when none is
/// picked.
pub fn calculate_random_weight_difficulty_index(world: &mut World, this: Entity, weights: &[WeightDifficulty], size: i32) -> i32 {
    if size < 1 {
        world.resource_mut::<Random>().arc4random();
        return 0;
    }

    let entries = &weights[..size as usize];
    let difficulty = world.obj::<MoronPestsQuestionScreenLogic>(this).difficulty;

    let mut total_weight = 0.0f32;
    for entry in entries {
        if entry.min_difficulty <= difficulty {
            total_weight += entry.weight;
        }
    }

    let step_count = ((total_weight * 10.0) as i32).wrapping_add(1) as u32;
    let random_steps = world.resource_mut::<Random>().arc4random_mod(step_count);
    let mut remaining_weight = (random_steps as i32) as f32 / 10.0;

    for (index, entry) in entries.iter().enumerate() {
        if entry.min_difficulty <= difficulty {
            remaining_weight -= entry.weight;
            if remaining_weight <= 0.0 {
                return index as i32;
            }
        }
    }
    0
}

/// `-[MoronPestsQuestionScreenLogic chooseMoronPestsQuestionType]` @ 0x100063300
pub fn choose_moron_pests_question_type(world: &mut World, this: Entity) -> i32 {
    calculate_random_weight_difficulty_index(world, this, &QUESTION_TYPE_WEIGHTS, 4)
}

/// `-[MoronPestsQuestionScreenLogic chooseSynonymQuestionType]` @ 0x10006331c
pub fn choose_synonym_question_type(world: &mut World, this: Entity) -> i32 {
    calculate_random_weight_difficulty_index(world, this, &SYNONYM_QUESTION_WEIGHTS, 3)
}

/// `-[MoronPestsQuestionScreenLogic chooseColorQuestionType]` @ 0x100063338
pub fn choose_color_question_type(world: &mut World, this: Entity) -> i32 {
    calculate_random_weight_difficulty_index(world, this, &COLOR_QUESTION_WEIGHTS, 12)
}

/// `-[MoronPestsQuestionScreenLogic chooseBeeQuestionType]` @ 0x100063354
pub fn choose_bee_question_type(world: &mut World, this: Entity) -> i32 {
    calculate_random_weight_difficulty_index(world, this, &BEE_QUESTION_WEIGHTS, 4)
}

/// `-[MoronPestsQuestionScreenLogic chooseDuckQuestionType]` @ 0x100063370
pub fn choose_duck_question_type(world: &mut World, this: Entity) -> i32 {
    calculate_random_weight_difficulty_index(world, this, &DUCK_QUESTION_WEIGHTS, 4)
}

/// `-[MoronPestsQuestionScreenLogic displayMultipleColorQuestion:]` @ 0x10006338c
///
/// `format` has three `%@`: the three button colours other than `mCorrectColorArray[0]`, in
/// random order.
pub fn display_multiple_color_question(world: &mut World, this: Entity, format: &str) {
    let active_count = world.obj::<MoronPestsQuestionScreenLogic>(this).active_button_colors_array.len();
    let correct_index = random_index(world, active_count);
    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        screen.color_tap_count = 1;
        let correct_color = screen.active_button_colors_array[correct_index];
        screen.correct_color_array[0] = Some(correct_color);

        let active_colors = screen.active_button_colors_array.clone();
        screen.randomize_array.clear();
        screen.randomize_array.extend(active_colors);
        screen.randomize_array.remove(correct_index);
    }

    let first_color = take_random_randomize_entry(world, this);
    let second_color = take_random_randomize_entry(world, this);
    let third_color = take_random_randomize_entry(world, this);

    let title = dynamic_text(world, COLOR_TITLE_TEXT_ID);
    let mut question = format.to_string();
    for color in [first_color, second_color, third_color] {
        question = question.replacen("%@", color, 1);
    }
    set_title_text(world, title, &question);
}

/// `-[MoronPestsQuestionScreenLogic calculateColorSwitchIndices:tapCount:]` @ 0x1000635c8
///
/// Marks `switch_count` random taps (of the first `tap_count − 1`) after which the colour
/// buttons shuffle.
pub fn calculate_color_switch_indices(world: &mut World, this: Entity, switch_count: i32, tap_count: i32) {
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).color_tap_index = 0;

    let mut statics = world.get_resource_or_insert_with(MoronPestsQuestionScreenLogicStatics::default);
    statics.color_switch_weights = COLOR_SWITCH_WEIGHTS;

    for _ in 0..switch_count {
        let switch_weights = world.resource::<MoronPestsQuestionScreenLogicStatics>().color_switch_weights;
        let switch_index = calculate_random_weight_difficulty_index(world, this, &switch_weights, tap_count - 1);

        let mut statics = world.resource_mut::<MoronPestsQuestionScreenLogicStatics>();
        statics.color_switch_weights[switch_index as usize].weight = 0.0;
    }
}

/// `-[MoronPestsQuestionScreenLogic randomizeColorButtons]` @ 0x100063660
///
/// Deals the button colours to the four buttons again. Button `i` draws a random entry of the
/// remaining colours, skipping the entry at index `i`; if the last remaining index was never
/// drawn, the last button's colour swaps with a random earlier button's.
pub fn randomize_color_buttons(world: &mut World, this: Entity) {
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).randomize_array.clear();

    let active_count = world.obj::<MoronPestsQuestionScreenLogic>(this).active_button_colors_array.len();
    if active_count != 0 {
        let mut button_index: i64 = 0;
        let mut last_index_drawn = false;
        loop {
            let remaining_count = world.obj::<MoronPestsQuestionScreenLogic>(this).active_button_colors_array.len();
            let last_index = (remaining_count as u32).wrapping_sub(1);
            let color_index = if last_index == 0 {
                0
            } else {
                let mut drawn_index = world.resource_mut::<Random>().arc4random_mod(last_index);
                if (drawn_index as i32) >= (button_index as i32) {
                    drawn_index += 1;
                }
                last_index_drawn |= drawn_index == last_index;
                drawn_index as i32 as usize
            };

            let mut shown_color = {
                let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
                let color = screen.active_button_colors_array[color_index];
                screen.randomize_array.push(color);
                screen.active_button_colors_array.remove(color_index);
                color
            };

            if remaining_count == 1 && !last_index_drawn {
                let dealt_count = world.obj::<MoronPestsQuestionScreenLogic>(this).randomize_array.len();
                let dealt_last_index = (dealt_count as u32).wrapping_sub(1);
                let swap_random = world.resource_mut::<Random>().arc4random_mod(dealt_last_index);
                let swap_index = swap_random as i32 as usize;
                let last_slot = dealt_last_index as i32 as usize;

                let (swapped_color, swapped_logic, last_color) = {
                    let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
                    screen.randomize_array.swap(swap_index, last_slot);
                    let swapped_color = screen.randomize_array[swap_index];
                    let swapped_logic = screen.color_logic_array[swap_index];
                    let last_color = screen.randomize_array[last_slot];
                    (swapped_color, swapped_logic, last_color)
                };
                let swapped_animation_id = format!("MoronPestButton{swapped_color}");
                logic::play_animation_id(world, swapped_logic, Some(&swapped_animation_id));
                shown_color = last_color;
            }

            let button_logic = world.obj::<MoronPestsQuestionScreenLogic>(this).color_logic_array[button_index as usize];
            let animation_id = format!("MoronPestButton{shown_color}");
            logic::play_animation_id(world, button_logic, Some(&animation_id));
            button_index += 1;

            let remaining_after = world.obj::<MoronPestsQuestionScreenLogic>(this).active_button_colors_array.len();
            if remaining_after == 0 {
                break;
            }
        }
    }

    let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
    let dealt_colors = screen.randomize_array.clone();
    screen.active_button_colors_array.extend(dealt_colors);
}

/// `-[MoronPestsQuestionScreenLogic displayTapMultipleColorsQuestion]` @ 0x100063954
///
/// "press A, then B, then C" with four presses: one of the three colours is pressed twice.
/// `mCorrectColorArray` lists the expected colour of each press.
pub fn display_tap_multiple_colors_question(world: &mut World, this: Entity) {
    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        let active_colors = screen.active_button_colors_array.clone();
        screen.randomize_array.clear();
        screen.randomize_array.extend(active_colors);
    }

    let first_color = take_random_randomize_entry(world, this);
    let second_color = take_random_randomize_entry(world, this);
    let third_color = take_random_randomize_entry(world, this);

    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        screen.color_tap_count = 4;
        screen.color_tap_index = 0;
    }

    let mut press_counts = [1i32; 3];
    let mut extra_press = 0;
    while extra_press < world.obj::<MoronPestsQuestionScreenLogic>(this).color_tap_count - 3 {
        let random = world.resource_mut::<Random>().arc4random();
        press_counts[(random % 3) as usize] += 1;
        extra_press += 1;
    }

    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        let presses = [
            (first_color, press_counts[0]),
            (second_color, press_counts[1]),
            (third_color, press_counts[2]),
        ];
        let mut press_index = 0;
        for (color, press_count) in presses {
            for _ in 0..press_count {
                screen.correct_color_array[press_index] = Some(color);
                press_index += 1;
            }
        }
    }

    let title = dynamic_text(world, COLOR_TITLE_TEXT_ID);
    let first_suffix = TAP_COUNT_SUFFIXES[(press_counts[0] - 1) as usize];
    let second_suffix = TAP_COUNT_SUFFIXES[(press_counts[1] - 1) as usize];
    let third_suffix = TAP_COUNT_SUFFIXES[(press_counts[2] - 1) as usize];
    let question = format!("press {first_color}{first_suffix}, then {second_color}{second_suffix}, then {third_color}{third_suffix}");
    set_title_text(world, title, &question);
}

/// `-[MoronPestsQuestionScreenLogic setMoronPestsPygmyLogic:]` @ 0x100063ca8
pub fn set_moron_pests_pygmy_logic(world: &mut World, this: Entity, moron_pests_pygmy_logic: Entity) {
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).moron_pests_pygmy_logic = Some(moron_pests_pygmy_logic);
}

/// `-[MoronPestsQuestionScreenLogic openMoronPestsQuestion]` @ 0x100063cb8
pub fn open_moron_pests_question(world: &mut World, this: Entity) {
    let question_type = choose_moron_pests_question_type(world, this);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).moron_pests_question_type = question_type;

    match question_type {
        0 => open_moron_pests_synonym_question(world, this),
        1 => open_moron_pests_color_question(world, this),
        2 => open_moron_pests_bee_question(world, this),
        3 => open_moron_pests_duck_question(world, this),
        _ => {}
    }
}

/// `FUN_100063d60`: bees by screen y, highest first (`sortUsingFunction:context:`).
fn compare_screen_y_descending(world: &mut World, first: Entity, second: Entity) -> Ordering {
    let first_pos = moron_bee_logic::screen_pos(world, first);
    let second_pos = moron_bee_logic::screen_pos(world, second);
    if first_pos.y > second_pos.y {
        Ordering::Less
    } else if first_pos.y < second_pos.y {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// `FUN_100063dc4`: bees by screen y, lowest first.
fn compare_screen_y_ascending(world: &mut World, first: Entity, second: Entity) -> Ordering {
    let first_pos = moron_bee_logic::screen_pos(world, first);
    let second_pos = moron_bee_logic::screen_pos(world, second);
    if first_pos.y < second_pos.y {
        Ordering::Less
    } else if first_pos.y > second_pos.y {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// `FUN_100063e28`: bees by screen x, rightmost first.
fn compare_screen_x_descending(world: &mut World, first: Entity, second: Entity) -> Ordering {
    let first_pos = moron_bee_logic::screen_pos(world, first);
    let second_pos = moron_bee_logic::screen_pos(world, second);
    if first_pos.x > second_pos.x {
        Ordering::Less
    } else if first_pos.x < second_pos.x {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// `FUN_100063e8c`: bees by screen x, leftmost first.
fn compare_screen_x_ascending(world: &mut World, first: Entity, second: Entity) -> Ordering {
    let first_pos = moron_bee_logic::screen_pos(world, first);
    let second_pos = moron_bee_logic::screen_pos(world, second);
    if first_pos.x < second_pos.x {
        Ordering::Less
    } else if first_pos.x > second_pos.x {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// `[[pest displayObject] scale]`.
fn display_scale(world: &World, pest: Entity) -> Vec4 {
    let display_object = logic::display_object(world, pest);
    display::object(world, display_object).scale
}

/// `FUN_100064338`: ducks by display scale, smallest first. The original compares the first
/// duck's scale.x with the second's scale.y, then with its scale.x.
fn compare_scale_ascending(world: &mut World, first: Entity, second: Entity) -> Ordering {
    let first_scale = display_scale(world, first);
    let second_scale = display_scale(world, second);
    if first_scale.x < second_scale.y {
        Ordering::Less
    } else if first_scale.x > second_scale.x {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// `FUN_1000643c0`: ducks by display scale, biggest first (the same mix of x and y).
fn compare_scale_descending(world: &mut World, first: Entity, second: Entity) -> Ordering {
    let first_scale = display_scale(world, first);
    let second_scale = display_scale(world, second);
    if first_scale.x > second_scale.y {
        Ordering::Less
    } else if first_scale.x < second_scale.x {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// `FUN_100064448`: ducks by eye-scale rank, lowest first.
fn compare_eye_scale_ascending(world: &mut World, first: Entity, second: Entity) -> Ordering {
    let first_eye_scale = moron_duck_logic::moron_duck_eye_scale(world, first);
    let second_eye_scale = moron_duck_logic::moron_duck_eye_scale(world, second);
    if first_eye_scale < second_eye_scale {
        Ordering::Less
    } else if first_eye_scale > second_eye_scale {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// `FUN_1000644a4`: ducks by eye-scale rank, highest first.
fn compare_eye_scale_descending(world: &mut World, first: Entity, second: Entity) -> Ordering {
    let first_eye_scale = moron_duck_logic::moron_duck_eye_scale(world, first);
    let second_eye_scale = moron_duck_logic::moron_duck_eye_scale(world, second);
    if first_eye_scale > second_eye_scale {
        Ordering::Less
    } else if first_eye_scale < second_eye_scale {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// `[pests sortUsingFunction:comparator context:NULL]`: the order in which the pests must be
/// touched. The pests' keys are all distinct, so the sort's stability does not matter.
fn sort_pests(world: &mut World, pests: &mut [Entity], comparator: PestComparator) {
    pests.sort_by(|first, second| comparator(world, *first, *second));
}

/// `-[MoronPestsQuestionScreenLogic openMoronPestsBeeQuestion]` @ 0x100063ef0
///
/// Five bees fly in at random screen positions (x 120 – 280, y 150 – 250), each at least 10
/// apart on both axes and 40 apart from the others.
pub fn open_moron_pests_bee_question(world: &mut World, this: Entity) {
    let mut screen_positions = [Vec4::ZERO; 5];
    for slot in 0..screen_positions.len() {
        loop {
            let x_random = world.resource_mut::<Random>().arc4random();
            let x_pos = ((x_random % 1601 + 1200) as i32) as f32 / 10.0;
            screen_positions[slot].x = x_pos;

            let y_random = world.resource_mut::<Random>().arc4random();
            let y_pos = ((y_random % 1001 + 1500) as i32) as f32 / 10.0;
            screen_positions[slot].y = y_pos;

            let mut too_close = false;
            for placed in &screen_positions[..slot] {
                let x_distance = x_pos - placed.x;
                if x_distance.abs() < BEE_MIN_AXIS_DISTANCE {
                    too_close = true;
                    break;
                }

                let y_distance = y_pos - placed.y;
                let distance = (x_distance * x_distance + y_distance * y_distance).sqrt();
                if y_distance.abs() < BEE_MIN_AXIS_DISTANCE || distance < BEE_MIN_DISTANCE {
                    too_close = true;
                    break;
                }
            }
            if !too_close {
                break;
            }
        }
    }

    let (bees, pygmy) = {
        let screen = world.obj::<MoronPestsQuestionScreenLogic>(this);
        (screen.moron_bee_logic_array.clone(), screen.moron_pests_pygmy_logic)
    };
    for (index, bee) in bees.iter().enumerate() {
        moron_bee_logic::surround_pygmy(world, *bee, pygmy, screen_positions[index]);
    }

    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        let all_bees = screen.moron_bee_logic_array.clone();
        screen.active_moron_bee_logic_array.clear();
        screen.active_moron_bee_logic_array.extend(all_bees);
    }

    screen::open_screen(world, this);
    let bee_question_type = choose_bee_question_type(world, this);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).bee_question_type = bee_question_type;
    let title = dynamic_text(world, ANIMAL_TITLE_TEXT_ID);
    screen::set_screen_layer_behavior(world, this, 0, Some(SHOW_ANIMAL_QUESTION_BEHAVIOR));

    let (comparator, question): (PestComparator, &str) = match bee_question_type {
        0 => (compare_screen_x_descending, "Touch the bees from right to left"),
        1 => (compare_screen_x_ascending, "Touch the bees from left to right"),
        2 => (compare_screen_y_descending, "Touch the bees from top to bottom"),
        3 => (compare_screen_y_ascending, "Touch the bees from bottom to top"),
        _ => return,
    };

    let mut active_bees = world.obj::<MoronPestsQuestionScreenLogic>(this).active_moron_bee_logic_array.clone();
    sort_pests(world, &mut active_bees, comparator);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).active_moron_bee_logic_array = active_bees;
    set_title_text(world, title, question);
}

/// `-[MoronPestsQuestionScreenLogic openMoronPestsDuckQuestion]` @ 0x100064500
///
/// Four ducks line up from (30, 130), each with a random body scale and a random eye-scale
/// rank.
pub fn open_moron_pests_duck_question(world: &mut World, this: Entity) {
    // PORT: the original deals the eye-scale ranks as NSNumbers through mRandomizeArray, which
    // ends up empty; the port deals them from a local and empties the ivar the same way.
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).randomize_array.clear();
    let mut eye_scales = DUCK_EYE_SCALES.to_vec();
    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        screen.randomize_scale_array.clear();
        screen.randomize_scale_array.extend(DUCK_SCALES);
    }

    let (ducks, pygmy) = {
        let screen = world.obj::<MoronPestsQuestionScreenLogic>(this);
        (screen.moron_duck_logic_array.clone(), screen.moron_pests_pygmy_logic)
    };
    let mut screen_pos = DUCK_FIRST_SCREEN_POS;
    for duck in ducks {
        let eye_scale_index = random_index(world, eye_scales.len());
        let eye_scale = eye_scales.remove(eye_scale_index);

        let scale_count = world.obj::<MoronPestsQuestionScreenLogic>(this).randomize_scale_array.len();
        let scale_index = random_index(world, scale_count);
        let scale = world.obj_mut::<MoronPestsQuestionScreenLogic>(this).randomize_scale_array.remove(scale_index);

        moron_duck_logic::surround_pygmy(world, duck, pygmy, screen_pos, scale, eye_scale);
        screen_pos.x += DUCK_SPACING;
    }

    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        let all_ducks = screen.moron_duck_logic_array.clone();
        screen.active_moron_duck_logic_array.clear();
        screen.active_moron_duck_logic_array.extend(all_ducks);
    }

    screen::open_screen(world, this);
    let duck_question_type = choose_duck_question_type(world, this);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).duck_question_type = duck_question_type;
    let title = dynamic_text(world, ANIMAL_TITLE_TEXT_ID);
    screen::set_screen_layer_behavior(world, this, 0, Some(SHOW_ANIMAL_QUESTION_BEHAVIOR));

    let (comparator, question): (PestComparator, &str) = match duck_question_type {
        0 => (compare_scale_descending, "Touch the ducks from biggest to smallest"),
        1 => (compare_scale_ascending, "Touch the ducks from smallest to biggest"),
        2 => (compare_eye_scale_descending, "Touch the ducks from biggest to smallest based on its eyes"),
        3 => (compare_eye_scale_ascending, "Touch the ducks from smallest to biggest based on its eyes"),
        _ => return,
    };

    let mut active_ducks = world.obj::<MoronPestsQuestionScreenLogic>(this).active_moron_duck_logic_array.clone();
    sort_pests(world, &mut active_ducks, comparator);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).active_moron_duck_logic_array = active_ducks;
    set_title_text(world, title, question);
}

/// `-[MoronPestsQuestionScreenLogic openMoronPestsSynonymQuestion]` @ 0x100064ac4
///
/// The two buttons show a homophone pair in random order; the question names one of them.
pub fn open_moron_pests_synonym_question(world: &mut World, this: Entity) {
    screen::open_screen(world, this);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).moron_pests_question_type = 0;
    screen::set_screen_layer_behavior(world, this, 0, Some("ShowMoronPestsSynonymQuestion"));

    let pair_random = world.resource_mut::<Random>().arc4random();
    let (first_word, second_word) = SYNONYM_PAIRS[(pair_random % 7) as usize];
    let correct_random = world.resource_mut::<Random>().arc4random();
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).correct_synonym_index = (correct_random & 1) as i32;

    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).randomize_array.clear();
    let order_random = world.resource_mut::<Random>().arc4random();
    let words = if order_random & 1 == 0 {
        [first_word, second_word]
    } else {
        [second_word, first_word]
    };
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).randomize_array.extend(words);

    let (button_words, synonym_logics) = {
        let screen = world.obj::<MoronPestsQuestionScreenLogic>(this);
        (screen.randomize_array.clone(), screen.synonym_logic_array.clone())
    };
    for (index, word) in button_words.iter().enumerate() {
        let animation_id = format!("MoronPestButton{word}");
        logic::play_animation_id(world, synonym_logics[index], Some(&animation_id));
    }

    let synonym_question_type = choose_synonym_question_type(world, this);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).synonym_question_type = synonym_question_type;
    let title = dynamic_text(world, SYNONYM_TITLE_TEXT_ID);

    let correct_word = {
        let screen = world.obj::<MoronPestsQuestionScreenLogic>(this);
        let correct_index = screen.correct_synonym_index as usize;
        screen.randomize_array[correct_index]
    };
    let question = match synonym_question_type {
        2 => format!("Press anything other than the {correct_word}"),
        1 => {
            let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
            screen.dont_answer_timer = DONT_ANSWER_TIME;
            screen.dont_answer_active = true;
            format!("Don't choose the {correct_word}")
        }
        0 => format!("Press the {correct_word}"),
        _ => return,
    };
    set_title_text(world, title, &question);
}

/// `-[MoronPestsQuestionScreenLogic openMoronPestsColorQuestion]` @ 0x100064efc
///
/// Four of the six colours go on the buttons; then one of twelve questions: press / don't
/// press / press anything but one colour (0 – 2) or three colours (3 – 5), press one colour
/// 5 – 7 times (6 – 8) or "press A, then B, then C" (9 – 11), the last two kinds shuffling
/// the buttons after 0, 1 or 2 – 3 random presses.
pub fn open_moron_pests_color_question(world: &mut World, this: Entity) {
    screen::open_screen(world, this);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).moron_pests_question_type = 1;
    screen::set_screen_layer_behavior(world, this, 0, Some("ShowMoronPestsColorQuestion"));

    let overlay_logics = world.obj::<MoronPestsQuestionScreenLogic>(this).color_overlay_logic_array.clone();
    for overlay_logic in overlay_logics {
        logic::play_animation_id(world, overlay_logic, Some(BUTTON_OVERLAY_UP));
    }

    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        let previous_colors = screen.active_button_colors_array.clone();
        screen.button_colors_array.extend(previous_colors);
        screen.active_button_colors_array.clear();
    }
    for _ in 0..4 {
        let color_count = world.obj::<MoronPestsQuestionScreenLogic>(this).button_colors_array.len();
        let color_index = random_index(world, color_count);

        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        let color = screen.button_colors_array[color_index];
        screen.active_button_colors_array.push(color);
        screen.button_colors_array.remove(color_index);
    }

    let (button_colors, color_logics) = {
        let screen = world.obj::<MoronPestsQuestionScreenLogic>(this);
        (screen.active_button_colors_array.clone(), screen.color_logic_array.clone())
    };
    for (index, color) in button_colors.iter().enumerate() {
        let animation_id = format!("MoronPestButton{color}");
        logic::play_animation_id(world, color_logics[index], Some(&animation_id));
    }

    let color_question_type = choose_color_question_type(world, this);
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).color_question_type = color_question_type;
    let title = dynamic_text(world, COLOR_TITLE_TEXT_ID);

    match color_question_type {
        0 => {
            world.obj_mut::<MoronPestsQuestionScreenLogic>(this).color_tap_count = 1;
            let color = choose_correct_color(world, this);
            let question = format!("Press the {color} button");
            set_title_text(world, title, &question);
        }
        1 => {
            world.obj_mut::<MoronPestsQuestionScreenLogic>(this).color_tap_count = 1;
            let color = choose_correct_color(world, this);
            let question = format!("Don't Press the {color} button");
            set_title_text(world, title, &question);

            let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
            screen.dont_answer_timer = DONT_ANSWER_TIME;
            screen.dont_answer_active = true;
        }
        2 => {
            world.obj_mut::<MoronPestsQuestionScreenLogic>(this).color_tap_count = 1;
            let color = choose_correct_color(world, this);
            let question = format!("Press anything but the {color} button");
            set_title_text(world, title, &question);
        }
        3 => display_multiple_color_question(world, this, "Press %@, %@, or %@"),
        4 => {
            display_multiple_color_question(world, this, "Don't Press %@, %@, or %@");

            let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
            screen.dont_answer_timer = DONT_ANSWER_TIME;
            screen.dont_answer_active = true;
        }
        5 => display_multiple_color_question(world, this, "Press anything but %@, %@, or %@"),
        6..=8 => {
            let color = choose_correct_color(world, this);
            let tap_random = world.resource_mut::<Random>().arc4random();
            let tap_count = (tap_random % 3 + 5) as i32;
            world.obj_mut::<MoronPestsQuestionScreenLogic>(this).color_tap_count = tap_count;

            let switch_count = match color_question_type {
                6 => 0,
                7 => 1,
                _ => {
                    let switch_random = world.resource_mut::<Random>().arc4random();
                    ((switch_random & 1) | 2) as i32
                }
            };
            calculate_color_switch_indices(world, this, switch_count, tap_count);

            let tap_count = world.obj::<MoronPestsQuestionScreenLogic>(this).color_tap_count;
            let question = format!("Press the {color} button {tap_count} times");
            set_title_text(world, title, &question);
        }
        9..=11 => {
            display_tap_multiple_colors_question(world, this);
            let switch_count = match color_question_type {
                9 => 0,
                10 => 1,
                _ => {
                    let switch_random = world.resource_mut::<Random>().arc4random();
                    ((switch_random & 1) | 2) as i32
                }
            };
            let tap_count = world.obj::<MoronPestsQuestionScreenLogic>(this).color_tap_count;
            calculate_color_switch_indices(world, this, switch_count, tap_count);
        }
        _ => {}
    }
}

/// `-[MoronPestsQuestionScreenLogic closeScreen]` @ 0x1000657e8
///
/// The pygmy was caught: the open question closes without an answer.
pub fn close_screen(world: &mut World, this: Entity) {
    // Original bug, kept: mbDontAnswerActive stays set, so the countdown of a "don't answer"
    // question still runs out in `onGameFrame:` and closes the question as answered.
    let question_type = world.obj::<MoronPestsQuestionScreenLogic>(this).moron_pests_question_type;
    match question_type {
        0 => screen::set_screen_layer_behavior(world, this, 0, Some("CloseMoronPestsSynonymQuestionLose")),
        1 => screen::set_screen_layer_behavior(world, this, 0, Some("CloseMoronPestsColorQuestionLose")),
        2 => {
            screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_ANIMAL_QUESTION_LOSE_BEHAVIOR));
            let bees = world.obj::<MoronPestsQuestionScreenLogic>(this).moron_bee_logic_array.clone();
            for bee in bees {
                moron_bee_logic::exit_screen(world, bee);
            }
        }
        3 => {
            screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_ANIMAL_QUESTION_LOSE_BEHAVIOR));
            let ducks = world.obj::<MoronPestsQuestionScreenLogic>(this).moron_duck_logic_array.clone();
            for duck in ducks {
                moron_duck_logic::exit_screen(world, duck);
            }
        }
        _ => {}
    }
}

/// `-[MoronPestsQuestionScreenLogic closeScreenNow]` @ 0x100065a94
pub fn close_screen_now(world: &mut World, this: Entity) {
    screen::close_screen_now(world, this);
}

/// `-[MoronPestsQuestionScreenLogic onSynonymButtonSelected:]` @ 0x100065ad0
pub fn on_synonym_button_selected(world: &mut World, this: Entity, synonym_button_selected: i32) {
    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        screen.dont_answer_active = false;
        screen.selected_button_index = synonym_button_selected;
    }
    screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_SYNONYM_QUESTION_BEHAVIOR));
}

/// `-[MoronPestsQuestionScreenLogic closeSynonymQuestionComplete:::]` @ 0x100065b04
pub fn seq_close_synonym_question_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let (answer_correct, listener) = {
        let screen = world.obj::<MoronPestsQuestionScreenLogic>(this);
        let answer_correct = match screen.synonym_question_type {
            2 => screen.correct_synonym_index != screen.selected_button_index,
            1 => screen.dont_answer_timer <= 0.0,
            0 => screen.correct_synonym_index == screen.selected_button_index,
            _ => return ItemResult::Done,
        };
        (answer_correct, screen.moron_pests_question_screen_logic_listener)
    };

    report_answer(world, listener, answer_correct);
    ItemResult::Done
}

/// `-[MoronPestsQuestionScreenLogic onColorButtonSelected:]` @ 0x100065bd4
///
/// The "press several times" questions (6 – 11) count the presses down and close on a wrong
/// colour or the last press; the others close at once.
pub fn on_color_button_selected(world: &mut World, this: Entity, color_button_selected: i32) {
    {
        let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
        screen.dont_answer_active = false;
        screen.selected_button_index = color_button_selected;
    }
    let overlay_logic = world.obj::<MoronPestsQuestionScreenLogic>(this).color_overlay_logic_array[color_button_selected as usize];
    logic::set_behavior(world, overlay_logic, Some(BUTTON_OVERLAY_UP));

    let color_question_type = world.obj::<MoronPestsQuestionScreenLogic>(this).color_question_type;
    match color_question_type {
        6 => {
            let (pressed_correct_color, presses_left) = {
                let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
                screen.color_tap_count -= 1;
                let expected = screen.correct_color_array[0];
                (is_selected_color(&screen, expected), screen.color_tap_count)
            };
            if pressed_correct_color && presses_left != 0 {
                return;
            }
            screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_COLOR_QUESTION_BEHAVIOR));
        }
        7 | 8 => {
            let (pressed_correct_color, presses_left) = {
                let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
                screen.color_tap_count -= 1;
                let expected = screen.correct_color_array[0];
                (is_selected_color(&screen, expected), screen.color_tap_count)
            };
            if !pressed_correct_color || presses_left == 0 {
                screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_COLOR_QUESTION_BEHAVIOR));
            } else {
                let color_tap_index = world.obj::<MoronPestsQuestionScreenLogic>(this).color_tap_index;
                if is_color_switch_tap(world, color_tap_index) {
                    randomize_color_buttons(world, this);
                }
            }
            world.obj_mut::<MoronPestsQuestionScreenLogic>(this).color_tap_index += 1;
        }
        9 => {
            let (pressed_correct_color, presses_left) = {
                let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
                screen.color_tap_count -= 1;
                let expected = correct_color_at(&screen, screen.color_tap_index);
                (is_selected_color(&screen, expected), screen.color_tap_count)
            };
            if !pressed_correct_color || presses_left == 0 {
                screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_COLOR_QUESTION_BEHAVIOR));
            }
            world.obj_mut::<MoronPestsQuestionScreenLogic>(this).color_tap_index += 1;
        }
        10 | 11 => {
            let (pressed_correct_color, presses_left) = {
                let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
                screen.color_tap_count -= 1;
                let expected = correct_color_at(&screen, screen.color_tap_index);
                (is_selected_color(&screen, expected), screen.color_tap_count)
            };
            if pressed_correct_color && presses_left != 0 {
                let color_tap_index = world.obj::<MoronPestsQuestionScreenLogic>(this).color_tap_index;
                if is_color_switch_tap(world, color_tap_index) {
                    randomize_color_buttons(world, this);
                }
            } else {
                screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_COLOR_QUESTION_BEHAVIOR));
            }
            world.obj_mut::<MoronPestsQuestionScreenLogic>(this).color_tap_index += 1;
        }
        _ => screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_COLOR_QUESTION_BEHAVIOR)),
    }
}

/// `-[MoronPestsQuestionScreenLogic closeColorQuestionComplete:::]` @ 0x100065f2c
pub fn seq_close_color_question_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let (answer_correct, listener) = {
        let screen = world.obj::<MoronPestsQuestionScreenLogic>(this);
        let answer_correct = match screen.color_question_type {
            0 | 5 | 6 | 7 | 8 => is_selected_color(screen, screen.correct_color_array[0]),
            1 | 4 => screen.dont_answer_timer <= 0.0,
            2 | 3 => !is_selected_color(screen, screen.correct_color_array[0]),
            9..=11 => {
                let expected = correct_color_at(screen, screen.color_tap_index - 1);
                is_selected_color(screen, expected)
            }
            _ => return ItemResult::Done,
        };
        (answer_correct, screen.moron_pests_question_screen_logic_listener)
    };

    report_answer(world, listener, answer_correct);
    ItemResult::Done
}

/// `-[MoronPestsQuestionScreenLogic closeAnimalQuestionComplete:::]` @ 0x100066130
///
/// The question was answered right when every pest was touched in order.
pub fn seq_close_animal_question_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let (pests_left, listener) = {
        let screen = world.obj::<MoronPestsQuestionScreenLogic>(this);
        let pests_left = match screen.moron_pests_question_type {
            3 => screen.active_moron_duck_logic_array.len(),
            2 => screen.active_moron_bee_logic_array.len(),
            _ => return ItemResult::Done,
        };
        (pests_left, screen.moron_pests_question_screen_logic_listener)
    };

    report_answer(world, listener, pests_left == 0);
    ItemResult::Done
}

/// `-[MoronPestsQuestionScreenLogic onButtonPress:touch:]` @ 0x1000661c4
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).pressed_button_logic = Some(button);

    let overlay_logic = {
        let screen = world.obj::<MoronPestsQuestionScreenLogic>(this);
        let pressed = Some(button);
        if screen.synonym1_button_logic == pressed || screen.synonym2_button_logic == pressed {
            return;
        }

        if screen.color1_button_logic == pressed {
            screen.color_overlay1_logic
        } else if screen.color2_button_logic == pressed {
            screen.color_overlay2_logic
        } else if screen.color3_button_logic == pressed {
            screen.color_overlay3_logic
        } else if screen.color4_button_logic == pressed {
            screen.color_overlay4_logic
        } else {
            return;
        }
    };

    if let Some(overlay_logic) = overlay_logic {
        logic::set_behavior(world, overlay_logic, Some(BUTTON_OVERLAY_DOWN));
    }
}

/// `-[MoronPestsQuestionScreenLogic onButtonRelease:touch:]` @ 0x100066294
pub fn on_button_release(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let pressed = Some(button);
    let pressed_button_logic = world.obj::<MoronPestsQuestionScreenLogic>(this).pressed_button_logic;
    if pressed_button_logic != pressed {
        return;
    }
    world.obj_mut::<MoronPestsQuestionScreenLogic>(this).pressed_button_logic = None;

    let buttons = {
        let screen = world.obj::<MoronPestsQuestionScreenLogic>(this);
        [
            screen.synonym1_button_logic,
            screen.synonym2_button_logic,
            screen.color1_button_logic,
            screen.color2_button_logic,
            screen.color3_button_logic,
            screen.color4_button_logic,
        ]
    };
    let [synonym1_button, synonym2_button, color1_button, color2_button, color3_button, color4_button] = buttons;

    if synonym1_button == pressed {
        on_synonym_button_selected(world, this, 0);
    } else if synonym2_button == pressed {
        on_synonym_button_selected(world, this, 1);
    } else if color1_button == pressed {
        on_color_button_selected(world, this, 0);
    } else if color2_button == pressed {
        on_color_button_selected(world, this, 1);
    } else if color3_button == pressed {
        on_color_button_selected(world, this, 2);
    } else if color4_button == pressed {
        on_color_button_selected(world, this, 3);
    }
}

/// `-[MoronPestsQuestionScreenLogic onButtonReleaseOutside:touch:]` @ 0x100066398
pub fn on_button_release_outside(world: &mut World, this: Entity, button: Entity, touch: Entity) {
    on_button_release(world, this, button, touch);
}

/// `-[MoronPestsQuestionScreenLogic onMoronBeeLogicSelected:]` @ 0x1000663a8
///
/// The bee next in order leaves; any other bee ends the question with every bee leaving.
pub fn on_moron_bee_logic_selected(world: &mut World, this: Entity, moron_bee_logic_selected: Entity) {
    let selected_bee = moron_bee_logic_selected;
    let first_active_bee = world.obj::<MoronPestsQuestionScreenLogic>(this).active_moron_bee_logic_array.first().copied();
    if first_active_bee == Some(selected_bee) {
        moron_bee_logic::exit_screen(world, selected_bee);

        let bees_left = {
            let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
            screen.active_moron_bee_logic_array.retain(|bee| *bee != selected_bee);
            screen.active_moron_bee_logic_array.len()
        };
        if bees_left == 0 {
            screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_ANIMAL_QUESTION_BEHAVIOR));
        }
    } else {
        let bees = world.obj::<MoronPestsQuestionScreenLogic>(this).moron_bee_logic_array.clone();
        for bee in bees {
            moron_bee_logic::exit_screen(world, bee);
        }
        screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_ANIMAL_QUESTION_BEHAVIOR));
    }
}

/// `-[MoronPestsQuestionScreenLogic onMoronDuckLogicSelected:]` @ 0x100066594
///
/// The duck next in order leaves; any other duck ends the question with every duck leaving.
pub fn on_moron_duck_logic_selected(world: &mut World, this: Entity, moron_duck_logic_selected: Entity) {
    let selected_duck = moron_duck_logic_selected;
    let first_active_duck = world.obj::<MoronPestsQuestionScreenLogic>(this).active_moron_duck_logic_array.first().copied();
    if first_active_duck == Some(selected_duck) {
        moron_duck_logic::exit_screen(world, selected_duck);

        let ducks_left = {
            let mut screen = world.obj_mut::<MoronPestsQuestionScreenLogic>(this);
            screen.active_moron_duck_logic_array.retain(|duck| *duck != selected_duck);
            screen.active_moron_duck_logic_array.len()
        };
        if ducks_left == 0 {
            screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_ANIMAL_QUESTION_BEHAVIOR));
        }
    } else {
        let ducks = world.obj::<MoronPestsQuestionScreenLogic>(this).moron_duck_logic_array.clone();
        for duck in ducks {
            moron_duck_logic::exit_screen(world, duck);
        }
        screen::set_screen_layer_behavior(world, this, 0, Some(CLOSE_ANIMAL_QUESTION_BEHAVIOR));
    }
}

/// `+[MoronPestsQuestionScreenLogic instance]` @ 0x100062700
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().moron_pests_question_screen_logic
}
