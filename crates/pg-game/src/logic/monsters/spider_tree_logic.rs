//! `SpiderTreeLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::monsters::spider_logic;

/// `SpiderTreeLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct SpiderTreeLogic {
    /// `mSpiderTreeLogicListener`: `@"<SpiderTreeLogicListener>"`
    pub spider_tree_logic_listener: Option<Entity>,

    /// `mTrappedPygmyLogic`: `@"PygmyLogic"`
    pub trapped_pygmy_logic: Option<Entity>,

    /// `mSpiderTreeState`: `i`
    pub spider_tree_state: i32,

    /// `mbTransitionActive`: `B`
    pub transition_active: bool,
}

pub const SPIDER_TREE_LOGIC: Class = Class {
    name: "SpiderTreeLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
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

/// The animation channel that carries the pygmy caught in the web.
const PYGMY_CHANNEL: usize = 1;

/// `mSpiderTreeState`: no pygmy in the web.
const STATE_EMPTY: i32 = 0;

/// `mSpiderTreeState`: a pygmy is stuck in the web.
const STATE_CAUGHT: i32 = 1;

/// `mSpiderTreeState`: the stuck pygmy is wrapped in a cocoon.
const STATE_COCOONED: i32 = 2;

/// `mSpiderTreeState`: the little spiders are eating the cocoon.
const STATE_EATEN: i32 = 3;

/// Sequence-item handlers implemented by `SpiderTreeLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "onTransitionComplete" => seq_on_transition_complete(world, this, call),
        "dieFromSpiders" => seq_die_from_spiders(world, this, call),
        "callForMom" => seq_call_for_mom(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[SpiderTreeLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&SPIDER_TREE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[SpiderTreeLogic initWithDisplayObject:]` @ 0x100068608
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(SpiderTreeLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::stop_game_frame(world, this);

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, own_display_object).is_selectable = false;

    pg_engine::logic::set_behavior(world, this, Some("SpiderTreeInit"));
}

/// `-[SpiderTreeLogic setSpiderTreeLogicListener:]` @ 0x1000686ac
pub fn set_spider_tree_logic_listener(world: &mut World, this: Entity, spider_tree_logic_listener: Entity) {
    world.obj_mut::<SpiderTreeLogic>(this).spider_tree_logic_listener = Some(spider_tree_logic_listener);
}

/// `-[SpiderTreeLogic isAvailable]` @ 0x100068708
pub fn is_available(world: &mut World, this: Entity) -> bool {
    world.obj::<SpiderTreeLogic>(this).trapped_pygmy_logic.is_none()
}

/// `-[SpiderTreeLogic trapInWeb:]` @ 0x100068720
///
/// Catches a pygmy thrown into the tree; the tree becomes tappable.
pub fn trap_in_web(world: &mut World, this: Entity, trap_in_web: Entity) {
    world.obj_mut::<SpiderTreeLogic>(this).trapped_pygmy_logic = Some(trap_in_web);

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = true;

    let animation = pg_engine::logic::animation(world, this);
    let pygmy_display_object = pg_engine::logic::display_object(world, trap_in_web);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);

    world.obj_mut::<SpiderTreeLogic>(this).spider_tree_state = STATE_CAUGHT;
    pg_engine::logic::set_behavior(world, this, Some("SpiderTreeCatchPygmy"));
}

/// `-[SpiderTreeLogic ejectFromWeb:]` @ 0x1000687dc
pub fn eject_from_web(world: &mut World, this: Entity, eject_from_web: Entity) {
    if world.obj::<SpiderTreeLogic>(this).trapped_pygmy_logic != Some(eject_from_web) {
        return;
    }

    world.obj_mut::<SpiderTreeLogic>(this).transition_active = false;

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

    pg_engine::logic::set_behavior(world, this, Some("SpiderTreeInit"));

    let mut tree = world.obj_mut::<SpiderTreeLogic>(this);
    tree.spider_tree_state = STATE_EMPTY;
    tree.trapped_pygmy_logic = None;
}

/// `-[SpiderTreeLogic enterIsland:]` @ 0x10006888c
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    pg_engine::logic::start_game_frame(world, this);

    {
        let mut tree = world.obj_mut::<SpiderTreeLogic>(this);
        tree.transition_active = false;
        tree.spider_tree_state = STATE_EMPTY;
    }
    pg_engine::logic::set_behavior(world, this, Some("SpiderTreeEnterIsland"));
}

/// `-[SpiderTreeLogic exitIsland:]` @ 0x1000688e8
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    let trapped = world.obj::<SpiderTreeLogic>(this).trapped_pygmy_logic;
    if let Some(trapped) = trapped {
        world.obj_mut::<SpiderTreeLogic>(this).spider_tree_state = STATE_EMPTY;

        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

        crate::pygmy::eject_from_spider_tree(world, trapped);
        world.obj_mut::<SpiderTreeLogic>(this).trapped_pygmy_logic = None;
    }

    if exit_island {
        pg_engine::logic::stop_game_frame(world, this);
    }

    world.obj_mut::<SpiderTreeLogic>(this).transition_active = false;

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    pg_engine::logic::set_behavior(world, this, Some("SpiderTreeExitIsland"));
}

/// `-[SpiderTreeLogic onGameFrame:]` @ 0x100068a2c
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[SpiderTreeLogic onTransitionComplete:::]` @ 0x100068a30
pub fn seq_on_transition_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpiderTreeLogic>(this).transition_active = false;
    ItemResult::Done
}

/// `-[SpiderTreeLogic dieFromSpiders:::]` @ 0x100068a44
pub fn seq_die_from_spiders(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpiderTreeLogic>(this).spider_tree_state = STATE_EMPTY;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, PYGMY_CHANNEL);

    let trapped = world.obj::<SpiderTreeLogic>(this).trapped_pygmy_logic;
    if let Some(trapped) = trapped {
        crate::pygmy::eaten_by_little_spiders(world, trapped, this);
    }
    world.obj_mut::<SpiderTreeLogic>(this).trapped_pygmy_logic = None;
    ItemResult::Done
}

/// `-[SpiderTreeLogic callForMom:::]` @ 0x100068ab8
///
/// The little spiders call the giant spider onto the island.
pub fn seq_call_for_mom(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<SpiderTreeLogic>(this).spider_tree_logic_listener;
    let Some(listener) = listener else {
        return ItemResult::Done;
    };

    if crate::controller::is_spider_enabled(world, listener) {
        let spider = crate::controller::spider_logic(world, listener, this);
        if let Some(spider) = spider {
            spider_logic::call_mom(world, spider);
        }
    }
    ItemResult::Done
}

/// `-[SpiderTreeLogic touchBeganInside:allTouches:]` @ 0x100068b20
///
/// The first tap wraps the caught pygmy in a cocoon, the second feeds it to the little spiders.
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let (transition_active, spider_tree_state) = {
        let tree = world.obj::<SpiderTreeLogic>(this);
        (tree.transition_active, tree.spider_tree_state)
    };
    if transition_active {
        return;
    }

    if spider_tree_state == STATE_COCOONED {
        world.obj_mut::<SpiderTreeLogic>(this).spider_tree_state = STATE_EATEN;
        pg_engine::logic::set_behavior(world, this, Some("SpiderTreeEatPygmy"));

        let display_object = pg_engine::logic::display_object(world, this);
        pg_engine::display::object_mut(world, display_object).is_selectable = false;
        return;
    }

    if spider_tree_state == STATE_CAUGHT {
        {
            let mut tree = world.obj_mut::<SpiderTreeLogic>(this);
            tree.transition_active = true;
            tree.spider_tree_state = STATE_COCOONED;
        }
        pg_engine::logic::set_behavior(world, this, Some("SpiderTreeCocoonPygmy"));
    }
}

/// `-[SpiderTreeLogic touchBeganOutside:allTouches:]` @ 0x100068bec
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpiderTreeLogic touchMovedEntered:allTouches:]` @ 0x100068bf0
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpiderTreeLogic touchMovedExited:allTouches:]` @ 0x100068bf4
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpiderTreeLogic touchMovedInside:allTouches:]` @ 0x100068bf8
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[SpiderTreeLogic touchMovedOutside:allTouches:]` @ 0x100068c08
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpiderTreeLogic touchEndedInside:allTouches:]` @ 0x100068c0c
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpiderTreeLogic touchEndedOutside:allTouches:]` @ 0x100068c10
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
