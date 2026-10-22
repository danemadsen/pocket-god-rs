//! `BananaTreeLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ape_monkeys::banana_logic;
use crate::logic::ape_monkeys::monkey_logic;
use crate::logic::island_core::coconut_logic;
use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::touch::BcTouch;

/// `BananaTreeLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BananaTreeLogic {
    /// `mBananaTreeLogicListener`: `@"<BananaTreeLogicListener>"`
    pub banana_tree_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mMonkeyLogic`: `@"MonkeyLogic"`
    pub monkey_logic: Option<Entity>,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbActive`: `B`
    pub active: bool,
}

pub const BANANA_TREE_LOGIC: Class = Class {
    name: "BananaTreeLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The tweenable channel of the tree's animation that carries a climbing or perched monkey.
const MONKEY_CHANNEL: usize = 2;

/// Sequence-item handlers implemented by `BananaTreeLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `BananaTreeLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animMonkeyClimbComplete" => anim_monkey_climb_complete(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[BananaTreeLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&BANANA_TREE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// A perched or climbing monkey is shaken off: it leaves the tree's animation and falls.
fn boot_monkey(world: &mut World, this: Entity) {
    let monkey = world.obj::<BananaTreeLogic>(this).monkey_logic;
    let Some(monkey) = monkey else {
        return;
    };

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, MONKEY_CHANNEL);
    monkey_logic::booted_from_banana_tree(world, monkey);
    world.obj_mut::<BananaTreeLogic>(this).monkey_logic = None;
}

/// `-[BananaTreeLogic initWithDisplayObject:]` @ 0x10013671c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(BananaTreeLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut tree = world.obj_mut::<BananaTreeLogic>(this);
        tree.banana_tree_logic_listener = None;
        tree.active_touch = None;
        tree.touch_began_inside = false;
        tree.touch_screen_pos = Vec4::new(0.0, 0.0, 0.0, 1.0);
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[BananaTreeLogic setBananaTreeLogicListener:]` @ 0x1001367cc
pub fn set_banana_tree_logic_listener(world: &mut World, this: Entity, banana_tree_logic_listener: Entity) {
    world.obj_mut::<BananaTreeLogic>(this).banana_tree_logic_listener = Some(banana_tree_logic_listener);
}

/// `-[BananaTreeLogic enterIsland:]` @ 0x100136828
///
/// The startup flag is unused.
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let active = world.obj::<BananaTreeLogic>(this).active;
    if active {
        return;
    }

    world.obj_mut::<BananaTreeLogic>(this).active = true;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;

    let marker = pg_engine::library::display_marker(world, "ApeIslandObjectsDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
    logic::set_behavior(world, this, Some("BananaTreeEnterIsland"));
}

/// `-[BananaTreeLogic exitIsland:]` @ 0x1001368e8
///
/// The flag is unused: the tree always plays its exit.
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    let active = world.obj::<BananaTreeLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<BananaTreeLogic>(this).active = false;
    boot_monkey(world, this);

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("BananaTreeExitIsland"));
}

/// `-[BananaTreeLogic exitIslandNow]` @ 0x1001369ac
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[BananaTreeLogic isAvailableForMonkey]` @ 0x1001369c0
pub fn is_available_for_monkey(world: &mut World, this: Entity) -> bool {
    let tree = world.obj::<BananaTreeLogic>(this);
    tree.monkey_logic.is_none() && tree.active
}

/// `-[BananaTreeLogic startMonkeyClimb:]` @ 0x1001369f0
pub fn start_monkey_climb(world: &mut World, this: Entity, monkey_climb: Entity) {
    world.obj_mut::<BananaTreeLogic>(this).monkey_logic = Some(monkey_climb);

    let animation = logic::animation(world, this);
    let monkey_display_object = logic::display_object(world, monkey_climb);
    pg_engine::animation::set_tweenable(world, animation, Some(monkey_display_object), MONKEY_CHANNEL);
    logic::set_behavior(world, this, Some("BananaTreeMonkeyClimb"));
}

/// `-[BananaTreeLogic monkeyLand:]` @ 0x100136a70
pub fn monkey_land(world: &mut World, this: Entity, monkey_land: Entity) {
    world.obj_mut::<BananaTreeLogic>(this).monkey_logic = Some(monkey_land);

    let animation = logic::animation(world, this);
    let monkey_display_object = logic::display_object(world, monkey_land);
    pg_engine::animation::set_tweenable(world, animation, Some(monkey_display_object), MONKEY_CHANNEL);
    logic::set_behavior(world, this, Some("BananaTreeMonkeyLand"));
}

/// `-[BananaTreeLogic monkeyJumpOffTree:]` @ 0x100136af0
pub fn monkey_jump_off_tree(world: &mut World, this: Entity, _monkey_jump_off_tree: Entity) {
    world.obj_mut::<BananaTreeLogic>(this).monkey_logic = None;

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, MONKEY_CHANNEL);
}

/// `-[BananaTreeLogic animMonkeyClimbComplete]` @ 0x100136b20
pub fn anim_monkey_climb_complete(world: &mut World, this: Entity) {
    let monkey = world.obj::<BananaTreeLogic>(this).monkey_logic;
    if let Some(monkey) = monkey {
        monkey_logic::banana_tree_climb_complete(world, monkey);
    }
}

/// `-[BananaTreeLogic onGameFrame:]` @ 0x100136ba0
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[BananaTreeLogic die:::]` @ 0x100136ba4
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<BananaTreeLogic>(this).active_touch = None;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    boot_monkey(world, this);
    ItemResult::Done
}

/// `-[BananaTreeLogic touchBeganInside:allTouches:]` @ 0x100136c3c
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<BananaTreeLogic>(this).banana_tree_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    if story_active {
        return;
    }

    // A tap drops the banana from the tree, unless it is already on the island.
    let banana = match listener {
        Some(listener) => crate::controller::banana_logic(world, listener, this),
        None => None,
    };
    let banana_active = match banana {
        Some(banana) => banana_logic::is_active(world, banana),
        None => false,
    };
    if banana_active {
        return;
    }
    let Some(banana) = banana else {
        // PORT: the original writes the position of a nil banana (NULL) and crashes; Ape
        // island always has its banana.
        return;
    };

    // The banana appears under the finger and is picked up at once.
    let banana_display_object = logic::display_object(world, banana);
    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let touch_screen_pos = world.obj::<BcTouch>(touch).pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };
    {
        let mut banana_display = display::object_mut(world, banana_display_object);
        banana_display.pos.x = touch_pos.x;
        banana_display.pos.y = touch_pos.y;
        banana_display.pos.z = coconut_logic::ISLAND_Z_POS;
    }

    banana_logic::enter_island(world, banana);
    let banana_class = isa(world, banana);
    (banana_class.touch_began_inside)(world, banana, touch);

    let monkey_enabled = match listener {
        Some(listener) => crate::controller::is_monkey_enabled(world, listener),
        None => false,
    };
    if !monkey_enabled {
        return;
    }

    // Each drop brings one more monkey onto the island.
    let monkey_logic_array = match listener {
        Some(listener) => crate::controller::monkey_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for &monkey in &monkey_logic_array {
        let monkey_active = monkey_logic::is_active(world, monkey);
        if !monkey_active {
            monkey_logic::enter_island(world, monkey, false);
            break;
        }
    }

    let mut active_monkey_count = 0;
    for &monkey in &monkey_logic_array {
        let monkey_active = monkey_logic::is_active(world, monkey);
        active_monkey_count += usize::from(monkey_active);
    }
    if active_monkey_count != monkey_logic_array.len() {
        return;
    }

    let idol_manager = crate::managers::idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event(world, idol_manager, "GetThreeMonkeys");
    }
}

/// `-[BananaTreeLogic touchBeganOutside:allTouches:]` @ 0x100136fa4
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BananaTreeLogic touchMovedEntered:allTouches:]` @ 0x100136fa8
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BananaTreeLogic touchMovedExited:allTouches:]` @ 0x100136fac
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BananaTreeLogic touchMovedInside:allTouches:]` @ 0x100136fb0
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[BananaTreeLogic touchMovedOutside:allTouches:]` @ 0x100136fc0
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BananaTreeLogic touchEndedInside:allTouches:]` @ 0x100136fc4
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BananaTreeLogic touchEndedOutside:allTouches:]` @ 0x100136fc8
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
