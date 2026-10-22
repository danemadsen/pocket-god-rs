//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::ape_monkeys::{banana_logic, monkey_logic};
use crate::logic::ice_island::{ice_hole_logic, igloo_logic};
use crate::logic::island_core::sprite_logic::SpriteLogic;
use crate::logic::island_core::{bon_fire_logic, coconut_logic, wood_logic};
use crate::logic::island_objects::out_house_logic;
use crate::logic::misfit_oogajump::dream_bubble_logic;
use crate::logic::ocean_birds::bird_logic;
use crate::logic::ocean_creatures::{shark_logic, sword_fish_logic};
use crate::logic::ocean_fishing::{fish_logic, fishing_pole_logic};
use crate::logic::sky_powers::rainbow_logic;
use crate::managers::skin_pack_manager;

/// Tweenable channel of the food being eaten (coconut, fish, bird, banana).
const FOOD_CHANNEL: usize = 14;

/// Tweenable channel of the carried wood.
const WOOD_CHANNEL: usize = 6;

/// Tweenable channels of the fishing pole, its bob and its string.
const FISHING_POLE_CHANNEL: usize = 2;
const FISHING_BOB_CHANNEL: usize = 3;
const FISHING_STRING_CHANNEL: usize = 4;

/// `mPersonaPersonality` of Charlie.
const CHARLIE_PERSONALITY: i32 = 1;

/// `mTimeTemplePygmyType` of a normal pygmy and of an old fart.
const NORMAL_PYGMY_TYPE: i32 = 0;
const OLD_FART_PYGMY_TYPE: i32 = 1;

/// `mStallTime` of `waitToCrap:::` without a time: wait until the out-house is free.
const WAIT_FOREVER: f32 = -1000.0;

/// How close (in x) the pygmy must be to a food, a wood, a banana peel or a fishing pole.
const PICK_UP_DISTANCE: f32 = 16.0;

/// How close (in x) the pygmy must be to its spot at the out-house, the igloo or the bonfire.
const SPOT_DISTANCE: f32 = 4.0;

/// `mPygmyLogicListener` (always the view controller).
fn listener_of(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener
}

/// `[[logic displayObject] pos]`.
fn display_pos(world: &World, logic: Entity) -> Vec4 {
    let display = pg_engine::logic::display_object(world, logic);
    pg_engine::display::object(world, display).pos
}

/// `[[logic displayObject] xPos]`: a nil object answers 0.
fn optional_x_pos(world: &World, logic: Option<Entity>) -> f32 {
    match logic {
        Some(logic) => display_pos(world, logic).x,
        None => 0.0,
    }
}

/// `[[logic displayObject] pos]` of an object the running sequence always has. The original
/// reads through the returned pointer, which a nil object would make NULL (a crash).
fn optional_pos(world: &World, logic: Option<Entity>) -> Vec4 {
    match logic {
        Some(logic) => display_pos(world, logic),
        // PORT: the original dereferences NULL here; the port reads zero.
        None => Vec4::ZERO,
    }
}

/// `[mDisplayObject xScale]`: the pygmy's facing.
fn own_x_scale(world: &World, this: Entity) -> f32 {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object(world, display).scale.x
}

/// `[mDisplayObject setXScale:x_scale]`.
fn set_x_scale(world: &mut World, this: Entity, x_scale: f32) {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).scale.x = x_scale;
}

/// `[mDisplayObject setIsSelectable:is_selectable]`.
fn set_selectable(world: &mut World, this: Entity, is_selectable: bool) {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).is_selectable = is_selectable;
}

/// `[mDisplayObject setZRot:0]`.
fn reset_z_rot(world: &mut World, this: Entity) {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).rot.z = 0.0;
}

/// `mPosVel.x = mPosVel.y = 0` (z and w keep their values).
fn stop_moving(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;
}

/// `mSacrificeId = sacrifice_id`: what the pygmy's death will be recorded as.
fn set_sacrifice_id(world: &mut World, this: Entity, sacrifice_id: &str) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(String::from(sacrifice_id));
}

/// `[mPygmyLogicListener onPygmyGiftGiven:self giftId:gift_id]`.
fn give_gift(world: &mut World, this: Entity, gift_id: &str) {
    let listener = listener_of(world, this);
    if let Some(listener) = listener {
        crate::controller::on_pygmy_gift_given(world, listener, this, gift_id);
    }
}

/// `[mAnimation setTweenable:display channelId:channel]`: the display object follows the
/// animation's attachment point `channel`.
fn attach_to_channel(world: &mut World, this: Entity, display: Option<Entity>, channel: usize) {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::set_tweenable(world, animation, display, channel);
}

/// `[mAnimation clearTweenableChannelIdNow:14]`: the food leaves the pygmy's hands at once.
fn release_food(world: &mut World, this: Entity) {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FOOD_CHANNEL);
}

/// Whether eating counts (a gift and `finishEating`): not for Charlie as a normal pygmy, nor for
/// an old fart.
fn food_counts(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let is_normal_charlie =
        core.persona_personality == CHARLIE_PERSONALITY && core.time_temple_pygmy_type == NORMAL_PYGMY_TYPE;

    !is_normal_charlie && core.time_temple_pygmy_type != OLD_FART_PYGMY_TYPE
}

/// A burnt bird or fish: a normal pygmy gets `mbAteBurntFood` and dies of `sacrifice_id`.
fn eat_burnt_food(world: &mut World, this: Entity, sacrifice_id: &str) {
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    if pygmy_type == NORMAL_PYGMY_TYPE {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).ate_burnt_food = true;
        set_sacrifice_id(world, this, sacrifice_id);
    }
}

/// `[[mPygmyLogicListener pygmyIslandBackDisplayMarker] insertAfter:mDisplayObject]`: the pygmy
/// is drawn behind the island's front objects (inside the out-house).
fn insert_after_island_back_marker(world: &mut World, this: Entity) {
    let listener = listener_of(world, this);
    let marker = listener.and_then(|listener| crate::controller::pygmy_island_back_display_marker(world, listener));

    if let Some(marker) = marker {
        let display = pg_engine::logic::display_object(world, this);
        pg_engine::display::insert_after(world, marker, Some(display));
    }
}

/// `mFishingPoleLogic`.
fn fishing_pole(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyIslandLife>(this).fishing_pole_logic
}

/// `[mPygmyLogicListener pygmyLogicArray:self]` (nil listener: no pygmies).
fn pygmy_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = listener_of(world, this);
    match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    }
}

/// `-[PygmyLogic (Island) thrownByMonkey:toMonkey:]` @ 0x1000b0608
pub fn thrown_by_monkey(world: &mut World, this: Entity, _monkey: Entity, to_monkey: Option<Entity>) {
    let caught_by_monkey = world.obj::<PygmyPygmyCoreFrame>(this).caught_by_monkey;
    if !caught_by_monkey {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = true;

    let x_pos = display_pos(world, this).x;
    let x_scale = own_x_scale(world, this);

    let y_vel: f32 = 700.0;
    let mut x_vel = x_scale * 200.0;

    // Toward a catching monkey: the x speed that reaches it in the flight time `2 * yVel / 3000`.
    if let Some(to_monkey) = to_monkey {
        let monkey_x_pos = display_pos(world, to_monkey).x;
        let flight_time = (y_vel + y_vel) / 3000.0;
        x_vel = (monkey_x_pos - x_pos) / flight_time;
    }

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_vel;
    logic.pos_vel.y = y_vel;

    reset_z_rot(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).caught_by_monkey = false;
    world.obj_mut::<PygmyIslandLife>(this).throwing_monkey_logic = None;

    set_selectable(world, this, true);
    set_sacrifice_id(world, this, "MonkeyThrow");
    pg_engine::logic::set_behavior(world, this, Some("PygmyThrownByMonkey"));
}

/// `-[PygmyLogic (Island) pickUpCoconut:::]` @ 0x1000b845c
pub fn seq_pick_up_coconut(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let coconut = world.obj::<PygmyIslandLife>(this).eating_coconut_logic;
    if let Some(coconut) = coconut {
        coconut_logic::pick_up_for_eating(world, coconut);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) finishEatingCoconut:::]` @ 0x1000b8490
pub fn seq_finish_eating_coconut(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let counts = food_counts(world, this);
    if counts {
        give_gift(world, this, "Coconut");

        let coconut = world.obj::<PygmyIslandLife>(this).eating_coconut_logic;
        if let Some(coconut) = coconut {
            coconut_logic::finish_eating(world, coconut);
        }
    }

    release_food(world, this);
    world.obj_mut::<PygmyIslandLife>(this).eating_coconut_logic = None;

    ItemResult::Done
}

/// `-[PygmyLogic (Island) pickUpWood:::]` @ 0x1000b8570
pub fn seq_pick_up_wood(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let wood = world.obj::<PygmyIslandLife>(this).carrying_wood_logic;
    incubate_disease(world, this, wood, "InfectWood", "InfectPygmyFromWood");

    let wood = world.obj::<PygmyIslandLife>(this).carrying_wood_logic;
    if let Some(wood) = wood {
        wood_logic::pick_up_for_carrying(world, wood);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) walkToBonFire:::]` @ 0x1000b85d4
pub fn seq_walk_to_bon_fire(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let bon_fire = listener.and_then(|listener| crate::controller::bon_fire_logic(world, listener, this));
    let fire_x_pos = optional_x_pos(world, bon_fire);

    // First call: face the fire and walk toward it.
    if call.first_time {
        let x_pos = display_pos(world, this).x;
        let (x_scale, x_vel) = if fire_x_pos < x_pos { (-1.0, -50.0) } else { (1.0, 50.0) };

        set_x_scale(world, this, x_scale);
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let x_pos = display_pos(world, this).x;
    let x_scale = own_x_scale(world, this);

    // Done within 50 of the fire on the walking side.
    let reached_from_left = x_scale > 0.0 && x_pos > fire_x_pos - 50.0;
    let reached_from_right = x_scale < 0.0 && x_pos < fire_x_pos + 50.0;

    if reached_from_left || reached_from_right {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        return ItemResult::Done;
    }

    ItemResult::Pending
}

/// `-[PygmyLogic (Island) animThrowWood]` @ 0x1000b8724
pub fn anim_throw_wood(world: &mut World, this: Entity) {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, WOOD_CHANNEL);

    let wood = world.obj::<PygmyIslandLife>(this).carrying_wood_logic;
    if let Some(wood) = wood {
        wood_logic::throw_on_pile(world, wood);
    }

    world.obj_mut::<PygmyIslandLife>(this).carrying_wood_logic = None;
}

/// `-[PygmyLogic (Island) pickUpBanana:::]` @ 0x1000b8788
pub fn seq_pick_up_banana(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let banana = world.obj::<PygmyIslandLife>(this).eating_banana_logic;
    if let Some(banana) = banana {
        banana_logic::pick_up_for_eating(world, banana);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) finishEatingBanana:::]` @ 0x1000b87bc
pub fn seq_finish_eating_banana(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let counts = food_counts(world, this);
    if counts {
        give_gift(world, this, "Banana");

        let banana = world.obj::<PygmyIslandLife>(this).eating_banana_logic;
        if let Some(banana) = banana {
            banana_logic::finish_eating(world, banana);
        }
    }

    release_food(world, this);
    world.obj_mut::<PygmyIslandLife>(this).eating_banana_logic = None;

    ItemResult::Done
}

/// `-[PygmyLogic (Island) pickUpBird:::]` @ 0x1000b889c
pub fn seq_pick_up_bird(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let bird = world.obj::<PygmyIslandLife>(this).eating_bird_logic;
    if let Some(bird) = bird {
        bird_logic::pick_up_for_eating(world, bird);
    }

    let bird = world.obj::<PygmyIslandLife>(this).eating_bird_logic;
    let burnt = bird.is_some_and(|bird| bird_logic::is_burnt(world, bird));
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;

    if !burnt || pygmy_type != NORMAL_PYGMY_TYPE {
        return ItemResult::Done;
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyEatBirdChoke"));
    ItemResult::Replaced
}

/// `-[PygmyLogic (Island) finishEatingBird:::]` @ 0x1000b892c
pub fn seq_finish_eating_bird(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let bird = world.obj::<PygmyIslandLife>(this).eating_bird_logic;
    let burnt = bird.is_some_and(|bird| bird_logic::is_burnt(world, bird));

    let finished = if burnt {
        eat_burnt_food(world, this, "EatBurntBird");
        true
    } else if food_counts(world, this) {
        // Only a cooked bird is eaten up (and counts as a gift).
        let cooked = bird.is_some_and(|bird| bird_logic::is_cooked(world, bird));
        if cooked {
            give_gift(world, this, "CookedBird");
        }
        cooked
    } else {
        false
    };

    if finished {
        let bird = world.obj::<PygmyIslandLife>(this).eating_bird_logic;
        if let Some(bird) = bird {
            bird_logic::finish_eating(world, bird);
        }
    }

    release_food(world, this);
    world.obj_mut::<PygmyIslandLife>(this).eating_bird_logic = None;

    ItemResult::Done
}

/// `-[PygmyLogic (Island) pickUpFish:::]` @ 0x1000b8a74
pub fn seq_pick_up_fish(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let fish = world.obj::<PygmyIslandLife>(this).eating_fish_logic;
    if let Some(fish) = fish {
        fish_logic::pick_up_for_eating(world, fish);
    }

    let fish = world.obj::<PygmyIslandLife>(this).eating_fish_logic;
    let burnt = fish.is_some_and(|fish| fish_logic::is_burnt(world, fish));
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;

    if !burnt || pygmy_type != NORMAL_PYGMY_TYPE {
        return ItemResult::Done;
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyEatFishChoke"));
    ItemResult::Replaced
}

/// `-[PygmyLogic (Island) finishEatingFish:::]` @ 0x1000b8b04
pub fn seq_finish_eating_fish(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let fish = world.obj::<PygmyIslandLife>(this).eating_fish_logic;
    let burnt = fish.is_some_and(|fish| fish_logic::is_burnt(world, fish));

    let finished = if burnt {
        eat_burnt_food(world, this, "EatBurntFish");
        true
    } else if food_counts(world, this) {
        let cooked = fish.is_some_and(|fish| fish_logic::is_cooked(world, fish));
        let gift_id = if cooked { "CookedFish" } else { "Fish" };
        give_gift(world, this, gift_id);
        true
    } else {
        false
    };

    if finished {
        let fish = world.obj::<PygmyIslandLife>(this).eating_fish_logic;
        if let Some(fish) = fish {
            fish_logic::finish_eating(world, fish);
        }
    }

    release_food(world, this);
    world.obj_mut::<PygmyIslandLife>(this).eating_fish_logic = None;

    ItemResult::Done
}

/// `-[PygmyLogic (Island) pickUpSwordFish:::]` @ 0x1000b8c5c
pub fn seq_pick_up_sword_fish(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let sword_fish = world.obj::<PygmyIslandLife>(this).eating_sword_fish_logic;
    if let Some(sword_fish) = sword_fish {
        sword_fish_logic::pick_up_for_eating(world, sword_fish);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) animSwordFishSwallow]` @ 0x1000b8c90
pub fn anim_sword_fish_swallow(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).ate_sword_fish = true;
}

/// `-[PygmyLogic (Island) finishEatingSwordFish:::]` @ 0x1000b8ca8
pub fn seq_finish_eating_sword_fish(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).ate_sword_fish = true;
    set_sacrifice_id(world, this, "SwordSwallower");

    // The sword fish is not on the food channel, so there is none to clear.
    let sword_fish = world.obj::<PygmyIslandLife>(this).eating_sword_fish_logic;
    if let Some(sword_fish) = sword_fish {
        sword_fish_logic::finish_eating(world, sword_fish);
    }

    world.obj_mut::<PygmyIslandLife>(this).eating_sword_fish_logic = None;

    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForNoBonFire:::]` @ 0x1000b999c
pub fn seq_check_for_no_bon_fire(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return ItemResult::Done;
    };

    let bon_fire_enabled = crate::controller::is_bon_fire_enabled(world, listener);
    if bon_fire_enabled {
        let bon_fire = crate::controller::bon_fire_logic(world, listener, this);
        let on_fire = bon_fire.is_some_and(|bon_fire| bon_fire_logic::is_on_fire(world, bon_fire));

        // The behaviour changes, but the item still answers 2.
        if !on_fire {
            pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
        }
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForRainbow:::]` @ 0x1000b9a28
pub fn seq_check_for_rainbow(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return ItemResult::Done;
    };

    let rainbow = crate::controller::rainbow_logic(world, listener, this);
    let rainbow_active = rainbow.is_some_and(|rainbow| rainbow_logic::is_rainbow_active(world, rainbow));

    let stares = rainbow_active && !crate::controller::is_inside_apocalypse_temple(world, listener);
    if stares {
        start_staring_at_rainbow(world, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForSunset:::]` @ 0x1000b9aac
pub fn seq_check_for_sunset(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return ItemResult::Done;
    };

    let sunset_active = crate::controller::is_sunset_active(world, listener);
    let watches_sunset = sunset_active && !crate::controller::is_inside_apocalypse_temple(world, listener);
    if !watches_sunset {
        return ItemResult::Done;
    }

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

    pg_engine::logic::set_behavior(world, this, Some("PygmySunset"));
    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForNoSunset:::]` @ 0x1000ba2c4
pub fn seq_check_for_no_sunset(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let sunset_over = match listener {
        Some(listener) => {
            !crate::controller::is_sunset_active(world, listener)
                || crate::controller::is_inside_apocalypse_temple(world, listener)
        }
        None => true,
    };

    if sunset_over {
        stop_sleeping(world, this);
        world.obj_mut::<PygmyIslandLife>(this).charlie_not_sleeping = false;
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForNight:::]` @ 0x1000ba360
pub fn seq_check_for_night(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let night = match listener {
        Some(listener) => {
            crate::controller::is_night_active(world, listener)
                && !crate::controller::is_inside_apocalypse_temple(world, listener)
        }
        None => false,
    };

    if !night {
        world.obj_mut::<PygmyIslandLife>(this).charlie_not_sleeping = false;
        return ItemResult::Done;
    }

    // Charlie refuses to sleep, once per night.
    let personality = world.obj::<PygmyPygmyCoreFrame>(this).persona_personality;
    let behavior = if personality == CHARLIE_PERSONALITY {
        let charlie_not_sleeping = world.obj::<PygmyIslandLife>(this).charlie_not_sleeping;
        if charlie_not_sleeping {
            return ItemResult::Done;
        }

        world.obj_mut::<PygmyIslandLife>(this).charlie_not_sleeping = true;
        "PygmyCharlieNoSleep"
    } else {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).sleeping = true;
        "PygmyFallAsleep"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForDay:::]` @ 0x1000ba44c
pub fn seq_check_for_day(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let day = match listener {
        Some(listener) => {
            !crate::controller::is_night_active(world, listener)
                || crate::controller::is_inside_apocalypse_temple(world, listener)
        }
        None => true,
    };

    if day {
        stop_sleeping(world, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyWakeUp"));
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForDreamBubble:::]` @ 0x1000ba4d8
pub fn seq_check_for_dream_bubble(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let dream_bubble = pg_engine::library::logic(world, crate::logic_table::DREAM_BUBBLE);

    let listener = listener_of(world, this);
    let dream_bubble_enabled = listener.is_some_and(|listener| crate::controller::is_dream_bubble_enabled(world, listener));
    if !dream_bubble_enabled {
        return ItemResult::Done;
    }

    let active = dream_bubble_logic::is_active(world, dream_bubble);
    if !active {
        world.obj_mut::<PygmyIslandLife>(this).dream_bubble_logic = Some(dream_bubble);
        dream_bubble_logic::activate(world, dream_bubble, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForDance:::]` @ 0x1000ba58c
pub fn seq_check_for_dance(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let dance_active = listener.is_some_and(|listener| crate::controller::is_dance_active(world, listener));
    if !dance_active {
        return ItemResult::Done;
    }

    // The theme-song backup dance when the "Pygmy Theme Song" dance of the Dance 3 pack is chosen.
    let manager = skin_pack_manager::instance(world);
    let theme_song_backup = manager.is_some_and(|manager| {
        skin_pack_manager::is_skin_pack_option_active_by_skin_pack_id(
            world,
            manager,
            "Dance3",
            "Pygmies",
            "Pygmy Dance",
            "Pygmy Theme Song",
        )
    });

    start_dancing(world, this, theme_song_backup);
    ItemResult::Done
}

/// `-[PygmyLogic (Island) startSleeping:::]` @ 0x1000ba630
pub fn seq_start_sleeping(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    if let Some(listener) = listener {
        crate::controller::on_pygmy_sleeping_start(world, listener, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) resetIgloo:::]` @ 0x1000ba918
pub fn seq_reset_igloo(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyIslandLife>(this).crawled_out_of_igloo = false;
    ItemResult::Done
}

/// `-[PygmyLogic (Island) resetCrapper:::]` @ 0x1000ba930
pub fn seq_reset_crapper(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyIslandLife>(this).finished_crapping = false;
    ItemResult::Done
}

/// `-[PygmyLogic (Island) waitToCrap:::]` @ 0x1000ba948
pub fn seq_wait_to_crap(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    // First call: wait forever, or a random time in [minTime, maxTime] when it is given.
    if call.first_time {
        let mut stall_time = WAIT_FOREVER;

        if call.has("minTime") {
            let min_time = call.float("minTime");
            let max_time = call.float("maxTime");
            let random = world.resource_mut::<Random>().arc4random();
            stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        }

        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
    }

    let listener = listener_of(world, this);
    let out_house = listener.and_then(|listener| crate::controller::out_house_logic(world, listener, this));
    let crapping_available =
        out_house.is_some_and(|out_house| out_house_logic::is_crapping_available(world, out_house, this));

    if !crapping_available {
        // The time is tested before it counts down, so the item ends one frame after it ran out.
        let stall_time = pg_engine::logic::logic(world, this).stall_time;
        let waits_forever = stall_time == WAIT_FOREVER;
        let result = if !waits_forever && stall_time <= 0.0 { ItemResult::Done } else { ItemResult::Pending };

        if !waits_forever {
            pg_engine::logic::logic_mut(world, this).stall_time -= call.delta_time;
        }

        return result;
    }

    stop_moving(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).crapping = true;
    set_selectable(world, this, false);
    world.obj_mut::<PygmyIslandLife>(this).crapping_out_house_logic = out_house;

    insert_after_island_back_marker(world, this);
    update_label_layer(world, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyTakeCrapAfterWaiting"));
    ItemResult::Replaced
}

/// `-[PygmyLogic (Island) takeCrap:::]` @ 0x1000bab7c
pub fn seq_take_crap(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let waiting_out_house = world.obj::<PygmyIslandLife>(this).waiting_out_house_logic;
    if let Some(waiting_out_house) = waiting_out_house {
        out_house_logic::stop_waiting_to_crap(world, waiting_out_house, this);
        world.obj_mut::<PygmyIslandLife>(this).waiting_out_house_logic = None;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;

    insert_after_island_back_marker(world, this);
    update_label_layer(world, this);

    let crapping_out_house = world.obj::<PygmyIslandLife>(this).crapping_out_house_logic;
    incubate_disease(world, this, crapping_out_house, "InfectOutHouse", "InfectPygmyFromOutHouse");

    let crapping_out_house = world.obj::<PygmyIslandLife>(this).crapping_out_house_logic;
    if let Some(crapping_out_house) = crapping_out_house {
        out_house_logic::take_crap(world, crapping_out_house, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForCoconut]` @ 0x1000bad1c
pub fn check_for_coconut(world: &mut World, this: Entity) -> bool {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return false;
    };

    let coconut_enabled = crate::controller::is_coconut_enabled(world, listener);
    if !coconut_enabled {
        return false;
    }

    let x_pos = display_pos(world, this).x;
    let coconuts = crate::controller::coconut_logic_array(world, listener, this);

    for coconut in coconuts {
        let edible = coconut_logic::is_edible(world, coconut);
        if !edible {
            continue;
        }

        let x_distance = x_pos - display_pos(world, coconut).x;
        let near = x_distance.abs() <= PICK_UP_DISTANCE;
        if !near {
            continue;
        }

        let x_scale = if x_distance > 0.0 { -1.0 } else { 1.0 };
        set_x_scale(world, this, x_scale);
        stop_moving(world, this);

        world.obj_mut::<PygmyIslandLife>(this).eating_coconut_logic = Some(coconut);

        let coconut_display = pg_engine::logic::display_object(world, coconut);
        attach_to_channel(world, this, Some(coconut_display), FOOD_CHANNEL);

        coconut_logic::start_eating(world, coconut, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyEatCoconut"));
        return true;
    }

    false
}

/// `-[PygmyLogic (Island) checkForWood]` @ 0x1000bafbc
pub fn check_for_wood(world: &mut World, this: Entity) -> bool {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return false;
    };

    let wood_enabled = crate::controller::is_wood_enabled(world, listener);
    if !wood_enabled {
        return false;
    }

    let x_pos = display_pos(world, this).x;
    let woods = crate::controller::wood_logic_array(world, listener, this);

    for wood in woods {
        let available = wood_logic::is_available(world, wood);
        if !available {
            continue;
        }

        let x_distance = x_pos - display_pos(world, wood).x;
        let near = x_distance.abs() <= PICK_UP_DISTANCE;
        if !near {
            continue;
        }

        let x_scale = if x_distance > 0.0 { -1.0 } else { 1.0 };
        set_x_scale(world, this, x_scale);
        stop_moving(world, this);

        world.obj_mut::<PygmyIslandLife>(this).carrying_wood_logic = Some(wood);

        let wood_display = pg_engine::logic::display_object(world, wood);
        attach_to_channel(world, this, Some(wood_display), WOOD_CHANNEL);

        wood_logic::start_carrying(world, wood, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyCarryWood"));
        return true;
    }

    false
}

/// `-[PygmyLogic (Island) checkForOutHouse]` @ 0x1000bb60c
pub fn check_for_out_house(world: &mut World, this: Entity) -> bool {
    let finished_crapping = world.obj::<PygmyIslandLife>(this).finished_crapping;
    if finished_crapping {
        return false;
    }

    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return false;
    };

    let out_house_enabled = crate::controller::is_out_house_enabled(world, listener);
    if !out_house_enabled {
        return false;
    }

    let x_pos = display_pos(world, this).x;
    let out_house = crate::controller::out_house_logic(world, listener, this);
    let Some(out_house) = out_house else {
        return false;
    };

    // The pygmy's spot is 90 right of the out-house.
    let crapping_available = out_house_logic::is_crapping_available(world, out_house, this);
    if crapping_available {
        let spot_x_pos = display_pos(world, out_house).x + 90.0;
        let at_spot = (x_pos - spot_x_pos).abs() <= SPOT_DISTANCE;
        if !at_spot {
            return false;
        }

        stop_moving(world, this);
        world.obj_mut::<PygmyPygmyCoreFrame>(this).crapping = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;
        set_selectable(world, this, false);

        world.obj_mut::<PygmyIslandLife>(this).crapping_out_house_logic = Some(out_house);

        insert_after_island_back_marker(world, this);
        update_label_layer(world, this);

        let crapping_out_house = world.obj::<PygmyIslandLife>(this).crapping_out_house_logic;
        incubate_disease(world, this, crapping_out_house, "InfectOutHouse", "InfectPygmyFromOutHouse");

        let crapping_out_house = world.obj::<PygmyIslandLife>(this).crapping_out_house_logic;
        if let Some(crapping_out_house) = crapping_out_house {
            out_house_logic::take_crap(world, crapping_out_house, this);
        }

        pg_engine::logic::set_behavior(world, this, Some("PygmyTakeCrap"));
        return true;
    }

    // Occupied: only a pygmy walking left joins the queue.
    let x_vel = pg_engine::logic::logic(world, this).pos_vel.x;
    let walking_left = x_vel < 0.0;
    if !walking_left {
        return false;
    }

    let waiting_available = out_house_logic::is_waiting_available(world, out_house, this);
    if !waiting_available {
        return false;
    }

    let spot_x_pos = display_pos(world, out_house).x + 90.0;
    let at_spot = (x_pos - spot_x_pos).abs() <= SPOT_DISTANCE;
    if !at_spot {
        return false;
    }

    stop_moving(world, this);
    world.obj_mut::<PygmyIslandLife>(this).waiting_out_house_logic = Some(out_house);
    out_house_logic::wait_to_crap(world, out_house, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyWaitToCrap"));
    true
}

/// `-[PygmyLogic (Island) checkForIgloo]` @ 0x1000bb8e0
pub fn check_for_igloo(world: &mut World, this: Entity) -> bool {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return false;
    };

    let igloo_enabled = crate::controller::is_igloo_enabled(world, listener);
    let crawled_out_of_igloo = world.obj::<PygmyIslandLife>(this).crawled_out_of_igloo;
    if !igloo_enabled || crawled_out_of_igloo {
        return false;
    }

    let igloo = crate::controller::igloo_logic(world, listener, this);
    let Some(igloo) = igloo else {
        return false;
    };

    let crawl_available = igloo_logic::is_pygmy_crawl_available(world, igloo, this);
    if !crawl_available {
        return false;
    }

    // The igloo's door is 99 right of the igloo.
    let door_x_pos = display_pos(world, igloo).x + 99.0;
    let x_pos = display_pos(world, this).x;
    let at_door = (x_pos - door_x_pos).abs() <= SPOT_DISTANCE;
    if !at_door {
        return false;
    }

    stop_moving(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = false;
    core.resting_in_igloo = true;

    world.obj_mut::<PygmyContainersAndCatchers>(this).igloo_logic = Some(igloo);
    igloo_logic::pygmy_crawl_into_igloo(world, igloo, this);
    set_selectable(world, this, false);

    let resting_igloo = world.obj::<PygmyContainersAndCatchers>(this).igloo_logic;
    incubate_disease(world, this, resting_igloo, "InfectIgloo", "InfectPygmyFromIgloo");

    pg_engine::logic::set_behavior(world, this, Some("PygmyCrawlIntoIgloo"));
    true
}

/// `-[PygmyLogic (Island) checkForBonFire]` @ 0x1000bbabc
pub fn check_for_bon_fire(world: &mut World, this: Entity) -> bool {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return false;
    };

    let bon_fire_enabled = crate::controller::is_bon_fire_enabled(world, listener);
    if !bon_fire_enabled {
        return false;
    }

    let x_pos = display_pos(world, this).x;
    let bon_fire = crate::controller::bon_fire_logic(world, listener, this);
    let Some(bon_fire) = bon_fire else {
        return false;
    };

    // The warming spots are 35 left and 35 right of the fire, facing it.
    let fire_x_pos = display_pos(world, bon_fire).x;

    let left_available = bon_fire_logic::is_left_warming_available(world, bon_fire, this);
    let near_left_spot = (x_pos - (fire_x_pos - 35.0)).abs() <= SPOT_DISTANCE;

    if left_available && near_left_spot {
        stop_moving(world, this);
        set_x_scale(world, this, 1.0);
        world.obj_mut::<PygmyIslandLife>(this).left_warming_bon_fire_logic = Some(bon_fire);
        bon_fire_logic::start_left_warming(world, bon_fire, this);
    } else {
        let right_available = bon_fire_logic::is_right_warming_available(world, bon_fire, this);
        let near_right_spot = (x_pos - (fire_x_pos + 35.0)).abs() <= SPOT_DISTANCE;

        if !right_available || !near_right_spot {
            return false;
        }

        stop_moving(world, this);
        set_x_scale(world, this, -1.0);
        world.obj_mut::<PygmyIslandLife>(this).right_warming_bon_fire_logic = Some(bon_fire);
        bon_fire_logic::start_right_warming(world, bon_fire, this);
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyWarmHands"));
    true
}

/// `-[PygmyLogic (Island) checkForFish]` @ 0x1000bbcc0
pub fn check_for_fish(world: &mut World, this: Entity) -> bool {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return false;
    };

    let inside_temple = crate::controller::is_inside_apocalypse_temple(world, listener);
    if inside_temple {
        return false;
    }

    let fish = crate::controller::fish_logic(world, listener, this);
    if let Some(fish) = fish {
        let edible = fish_logic::is_edible(world, fish);
        let x_distance = display_pos(world, this).x - display_pos(world, fish).x;
        let near = x_distance.abs() <= PICK_UP_DISTANCE;

        if edible && near {
            stop_moving(world, this);
            world.obj_mut::<PygmyIslandLife>(this).eating_fish_logic = Some(fish);

            let fish_display = pg_engine::logic::display_object(world, fish);
            attach_to_channel(world, this, Some(fish_display), FOOD_CHANNEL);

            fish_logic::start_eating(world, fish, this);
            pg_engine::logic::set_behavior(world, this, Some("PygmyEatFish"));
            return true;
        }
    }

    // Only a normal pygmy eats a sword fish; it is swallowed whole (no channel, no facing).
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    if pygmy_type != NORMAL_PYGMY_TYPE {
        return false;
    }

    let sword_fish = crate::controller::sword_fish_logic(world, listener, this);
    let Some(sword_fish) = sword_fish else {
        return false;
    };

    let edible = sword_fish_logic::is_edible(world, sword_fish);
    if !edible {
        return false;
    }

    let x_distance = display_pos(world, this).x - display_pos(world, sword_fish).x;
    let near = x_distance.abs() <= PICK_UP_DISTANCE;
    if !near {
        return false;
    }

    stop_moving(world, this);
    world.obj_mut::<PygmyIslandLife>(this).eating_sword_fish_logic = Some(sword_fish);
    sword_fish_logic::start_eating(world, sword_fish, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmyEatSwordFish"));
    true
}

/// `-[PygmyLogic (Island) checkForBird]` @ 0x1000bbf44
pub fn check_for_bird(world: &mut World, this: Entity) -> bool {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return false;
    };

    let inside_temple = crate::controller::is_inside_apocalypse_temple(world, listener);
    if inside_temple {
        return false;
    }

    let bird = crate::controller::bird_logic(world, listener, this);
    let Some(bird) = bird else {
        return false;
    };

    let edible = bird_logic::is_edible(world, bird);
    if !edible {
        return false;
    }

    let x_distance = display_pos(world, this).x - display_pos(world, bird).x;
    let near = x_distance.abs() <= PICK_UP_DISTANCE;
    if !near {
        return false;
    }

    stop_moving(world, this);
    world.obj_mut::<PygmyIslandLife>(this).eating_bird_logic = Some(bird);

    let bird_display = pg_engine::logic::display_object(world, bird);
    attach_to_channel(world, this, Some(bird_display), FOOD_CHANNEL);

    bird_logic::start_eating(world, bird, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyEatBird"));
    true
}

/// `-[PygmyLogic (Island) checkForBanana]` @ 0x1000bc0cc
pub fn check_for_banana(world: &mut World, this: Entity) -> bool {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return false;
    };

    let banana = crate::controller::banana_logic(world, listener, this);
    let Some(banana) = banana else {
        return false;
    };

    let x_pos = display_pos(world, this).x;
    let x_distance = x_pos - display_pos(world, banana).x;
    let near = x_distance.abs() <= PICK_UP_DISTANCE;

    let edible = banana_logic::is_edible(world, banana);
    if edible && near {
        stop_moving(world, this);
        world.obj_mut::<PygmyIslandLife>(this).eating_banana_logic = Some(banana);

        let banana_display = pg_engine::logic::display_object(world, banana);
        attach_to_channel(world, this, Some(banana_display), FOOD_CHANNEL);

        banana_logic::start_eating(world, banana, this);
        pg_engine::logic::set_behavior(world, this, Some("PygmyEatBanana"));

        // A normal pygmy's banana draws the nearest monkey that can chase it.
        let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
        if pygmy_type == NORMAL_PYGMY_TYPE {
            let monkeys = crate::controller::monkey_logic_array(world, listener, this);
            let mut nearest_monkey = None;
            let mut nearest_distance: f32 = 10000.0;

            for monkey in monkeys {
                let can_chase = monkey_logic::can_chase_pygmy(world, monkey);
                if !can_chase {
                    continue;
                }

                let distance = (display_pos(world, monkey).x - x_pos).abs();
                if distance < nearest_distance {
                    nearest_monkey = Some(monkey);
                    nearest_distance = distance;
                }
            }

            if let Some(monkey) = nearest_monkey {
                monkey_logic::pygmy_is_eating_banana(world, monkey, this);
            }
        }

        return true;
    }

    // A peel on the ground: a normal pygmy slips on it.
    let slippable = banana_logic::is_slippable(world, banana);
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    if !slippable || pygmy_type != NORMAL_PYGMY_TYPE || !near {
        return false;
    }

    stop_moving(world, this);
    banana_logic::slip_on_banana(world, banana, this);

    pg_engine::logic::set_behavior(world, this, Some("PygmySlipOnBanana"));
    true
}

/// `-[PygmyLogic (Island) checkForFishingPole]` @ 0x1000bc48c
pub fn check_for_fishing_pole(world: &mut World, this: Entity) -> bool {
    let listener = listener_of(world, this);
    let Some(listener) = listener else {
        return false;
    };

    let pole = crate::controller::fishing_pole_logic(world, listener, this);
    let Some(pole) = pole else {
        return false;
    };

    let available = fishing_pole_logic::is_available(world, pole);
    if !available {
        return false;
    }

    let x_pos = display_pos(world, this).x;
    let pole_x_pos = display_pos(world, pole).x;

    let ice_hole = crate::controller::ice_hole_logic(world, listener, this);
    let ice_hole_enabled = crate::controller::is_ice_hole_enabled(world, listener);
    let open_ice_hole =
        ice_hole.filter(|&ice_hole| ice_hole_enabled && ice_hole_logic::is_available_for_fishing(world, ice_hole));

    // Ice fishing when the pole lies 40 to 100 left of an open ice hole.
    if let Some(ice_hole) = open_ice_hole {
        let hole_x_pos = display_pos(world, ice_hole).x;
        let pole_offset = pole_x_pos - hole_x_pos;

        if (-100.0..=-40.0).contains(&pole_offset) {
            // The pygmy's seat is 65 left of the hole.
            let seat_distance = (x_pos - (hole_x_pos - 65.0)).abs();
            let at_seat = seat_distance <= 25.0;
            if !at_seat {
                return false;
            }

            stop_moving(world, this);

            let mut island_life = world.obj_mut::<PygmyIslandLife>(this);
            island_life.ice_fishing = true;
            island_life.fishing_pole_logic = Some(pole);

            fishing_pole_logic::start_ice_fishing(world, pole, this);

            let held_pole = fishing_pole(world, this);
            incubate_disease(world, this, held_pole, "InfectFishingPole", "InfectPygmyFromFishingPole");

            // The pygmy sits in front of the hole.
            let hole_display = pg_engine::logic::display_object(world, ice_hole);
            let display = pg_engine::logic::display_object(world, this);
            pg_engine::display::insert_after(world, hole_display, Some(display));
            update_label_layer(world, this);

            world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
            world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;

            pg_engine::logic::set_behavior(world, this, Some("PygmyStartIceFishing"));
            ice_hole_logic::start_fishing(world, ice_hole, this);
            return true;
        }
    }

    let x_distance = x_pos - pole_x_pos;
    let near = x_distance.abs() <= PICK_UP_DISTANCE;
    if !near {
        return false;
    }

    stop_moving(world, this);
    world.obj_mut::<PygmyIslandLife>(this).fishing_pole_logic = Some(pole);

    // The pole is drawn just behind the pygmy.
    let pole_display = pg_engine::logic::display_object(world, pole);
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::insert_before(world, pole_display, Some(display));
    update_label_layer(world, this);

    let held_pole = fishing_pole(world, this);
    if let Some(held_pole) = held_pole {
        fishing_pole_logic::start_fishing(world, held_pole, this);
    }

    let held_pole = fishing_pole(world, this);
    incubate_disease(world, this, held_pole, "InfectFishingPole", "InfectPygmyFromFishingPole");

    // The pole, its bob and its string follow the pygmy's hands.
    let held_pole = fishing_pole(world, this);
    let held_pole_display = held_pole.map(|held_pole| pg_engine::logic::display_object(world, held_pole));
    attach_to_channel(world, this, held_pole_display, FISHING_POLE_CHANNEL);

    let held_pole = fishing_pole(world, this);
    let bob = held_pole.and_then(|held_pole| fishing_pole_logic::fishing_bob_logic(world, held_pole));
    let bob_display = bob.map(|bob| pg_engine::logic::display_object(world, bob));
    attach_to_channel(world, this, bob_display, FISHING_BOB_CHANNEL);

    let held_pole = fishing_pole(world, this);
    let string_display =
        held_pole.and_then(|held_pole| fishing_pole_logic::fishing_string_display_object(world, held_pole));
    attach_to_channel(world, this, string_display, FISHING_STRING_CHANNEL);

    pg_engine::logic::set_behavior(world, this, Some("PygmyStartFishing"));
    true
}

/// `-[PygmyLogic (Island) walkToIceFishingPole:::]` @ 0x1000bc86c
pub fn seq_walk_to_ice_fishing_pole(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let ice_hole = listener.and_then(|listener| crate::controller::ice_hole_logic(world, listener, this));

    // The seat is 67 left of the hole and 5 above it.
    let hole_pos = optional_pos(world, ice_hole);
    let seat_y_pos = hole_pos.y + 5.0;

    // First call: fly to the seat in one second, facing the way there.
    if call.first_time {
        let pos = display_pos(world, this);
        let x_vel = (hole_pos.x - 67.0) - pos.x;

        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = x_vel;
        logic.pos_vel.y = seat_y_pos - pos.y;

        let x_scale = if x_vel < 0.0 { -1.0 } else { 1.0 };
        set_x_scale(world, this, x_scale);
    }

    let y_pos = display_pos(world, this).y;
    if y_pos > seat_y_pos {
        stop_moving(world, this);

        if let Some(ice_hole) = ice_hole {
            ice_hole_logic::start_fishing_sit_down(world, ice_hole, this);
        }

        return ItemResult::Done;
    }

    ItemResult::Pending
}

/// `-[PygmyLogic (Island) startFishing:::]` @ 0x1000bd51c
pub fn seq_start_fishing(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pole = fishing_pole(world, this);
    let available = pole.is_some_and(|pole| fishing_pole_logic::is_available(world, pole));

    // The pygmy still holds the pole (so it is not available): the pole starts fishing idle.
    // Otherwise the pygmy lost it and goes back to the ground.
    if !available {
        let pole = fishing_pole(world, this);
        if let Some(pole) = pole {
            fishing_pole_logic::start_fishing_idle(world, pole);
        }

        return ItemResult::Done;
    }

    let pole = fishing_pole(world, this);
    if pole.is_some() {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = true;

        let island_y_pos = world.obj::<PygmyPygmyCoreFrame>(this).island_y_pos;
        let display = pg_engine::logic::display_object(world, this);
        pg_engine::display::object_mut(world, display).pos.y = island_y_pos;

        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id(world, animation, FISHING_POLE_CHANNEL);
        pg_engine::animation::clear_tweenable_channel_id(world, animation, FISHING_BOB_CHANNEL);
        pg_engine::animation::clear_tweenable_channel_id(world, animation, FISHING_STRING_CHANNEL);

        world.obj_mut::<PygmyIslandLife>(this).fishing_pole_logic = None;
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    ItemResult::Replaced
}

/// `-[PygmyLogic (Island) walkToFishingPole:::]` @ 0x1000bd648
pub fn seq_walk_to_fishing_pole(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let pole = fishing_pole(world, this);

    // The fishing spot is up on the pole's rock: 26.55 right of the pole and 8.05 above it.
    let pole_pos = optional_pos(world, pole);
    let spot_y_pos = pole_pos.y + 8.05;

    // First call: fly to the spot in one second, facing the way there.
    if call.first_time {
        let pos = display_pos(world, this);
        let x_vel = (pole_pos.x + 26.55) - pos.x;

        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;

        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = x_vel;
        logic.pos_vel.y = spot_y_pos - pos.y;

        let x_scale = if x_vel < 0.0 { -1.0 } else { 1.0 };
        set_x_scale(world, this, x_scale);
    }

    let y_pos = display_pos(world, this).y;
    if y_pos < spot_y_pos {
        stop_moving(world, this);
        return ItemResult::Done;
    }

    ItemResult::Pending
}

/// `-[PygmyLogic (Island) jumpOnFire:::]` @ 0x1000be72c
pub fn seq_jump_on_fire(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.on_fire_from_lava = false;
    core.on_fire_from_bon_fire = false;
    core.falling = true;

    pg_engine::logic::logic_mut(world, this).pos_vel.y = 600.0;

    // A jump the way the pygmy faces.
    let x_scale = own_x_scale(world, this);
    let x_vel = if x_scale < 0.0 { -100.0 } else { 100.0 };
    pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = false;

    ItemResult::Done
}

/// `-[PygmyLogic (Island) sharkSwallow:::]` @ 0x1000be808
pub fn seq_shark_swallow(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let shark = world.obj::<PygmyIslandLife>(this).eating_shark_logic;
    if let Some(shark) = shark {
        shark_logic::swallow_pygmy(world, shark);
    }

    world.obj_mut::<PygmyIslandLife>(this).eating_shark_logic = None;
    set_sacrifice_id(world, this, "SharkOnHead");

    // The item answers 2 although the behaviour changed.
    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
    ItemResult::Done
}

/// `-[PygmyLogic (Island) watchShark:::]` @ 0x1000be880
pub fn seq_watch_shark(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pygmies = pygmy_logic_array(world, this);

    for pygmy in pygmies {
        let shark_on_head = has_shark_on_head(world, pygmy);
        if !shark_on_head {
            continue;
        }

        // Forward when the shark victim is in front of the pygmy (or level with it).
        let other_x_pos = display_pos(world, pygmy).x;
        let x_pos = display_pos(world, this).x;
        let x_scale = own_x_scale(world, this);
        let behind = (x_pos - other_x_pos) * x_scale > 0.0;
        let behavior = if behind { "PygmyAlreadyReactBackward" } else { "PygmyAlreadyReactForward" };

        pg_engine::logic::set_behavior(world, this, Some(behavior));
        return ItemResult::Pending;
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) checkForShark]` @ 0x1000bea64
pub fn check_for_shark(world: &mut World, this: Entity) -> bool {
    let pygmies = pygmy_logic_array(world, this);

    for pygmy in pygmies {
        let shark_on_head = has_shark_on_head(world, pygmy);
        if !shark_on_head {
            continue;
        }

        let other_x_pos = display_pos(world, pygmy).x;
        let x_distance = display_pos(world, this).x - other_x_pos;
        let near = x_distance.abs() < 50.0;
        if !near {
            continue;
        }

        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;

        // Forward when the shark victim is in front of the pygmy (or level with it).
        let x_scale = own_x_scale(world, this);
        let behind = x_distance * x_scale > 0.0;
        let behavior = if behind { "PygmyReactBackward" } else { "PygmyReactForward" };

        pg_engine::logic::set_behavior(world, this, Some(behavior));
        return true;
    }

    false
}
