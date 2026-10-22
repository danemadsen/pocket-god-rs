//! `GodIdolLogic` (superclass `SpriteLogic`).
//!
//! A throwable (`docs/re/logic-catalog.md` §2.2) with its own floor (y = 90 between x = 32 and
//! 420), no slide friction, and a drag spring at a quarter of the usual tilt.

use crate::prelude::*;

use crate::logic::battle::idol_pedestal_logic;
use crate::logic::idol_screen::idol_screen_logic::activate_skin_texture_bits_def_array;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::managers::idol_manager::episode_table;
use pg_engine::animation;
use pg_engine::display;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::math::{length3, normalize3, rad_to_deg};
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

/// `GodIdolLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct GodIdolLogic {
    /// `mGodIdolLogicListener`: `@"<GodIdolLogicListener>"`
    pub god_idol_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mpStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub struct_xml_episode_node_def: Option<usize>,

    /// `mpLastStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub last_struct_xml_episode_node_def: Option<usize>,

    /// `mSkinTextureBitsDefArray`: `^{BCSkinTextureBitsDef=^{BCTextureDef}iiii}`
    pub skin_texture_bits_def_array: Option<usize>,

    /// `mSlot`: `i`
    pub slot: i32,

    /// `mIdolPedestalLogic`: `@"IdolPedestalLogic"`
    pub idol_pedestal_logic: Option<Entity>,

    /// `mbDontHitUntilBounce`: `B`
    pub dont_hit_until_bounce: bool,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mTiltState`: `i`
    pub tilt_state: i32,

    /// `mSlideFriction`: `f`
    pub slide_friction: f32,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,

    /// `mbInWater`: `B`
    pub in_water: bool,
}

pub const GOD_IDOL_LOGIC: Class = Class {
    name: "GodIdolLogic",
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

/// `mSlideFriction` (`0x3ccccccd`); `onGameFrame:` never applies it.
const SLIDE_FRICTION: f32 = 0.025;

/// `mGravityForce` while the accelerometer does not tilt the world.
const GRAVITY_FORCE: Vec4 = Vec4::new(0.0, -3000.0, 0.0, 1.0);

/// The strength of the tilted gravity.
const TILTED_GRAVITY: f32 = 3000.0;

/// The island floor of `onGameFrame:` and the x range over which it holds.
const FLOOR_Y: f32 = 90.0;
const FLOOR_MIN_X: f32 = 32.0;
const FLOOR_MAX_X: f32 = 420.0;

/// A floor contact this fast upwards bounces (`GodIdolBounce`).
const BOUNCE_SPEED: f32 = 100.0;

/// A floor contact settles when both speeds stay below this.
const SETTLE_SPEED: f32 = 30.0;

/// The idol dies outside this height range.
const MIN_Y: f32 = -100.0;
const MAX_Y: f32 = 820.0;

/// `fall:::` splashes below this height.
const WATER_LINE_Y: f32 = 10.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 50.0;
const DRAG_DAMPING: f32 = 6.0;

/// `drag:::` tilts the idol by a quarter of the drag angle.
const DRAG_TILT_SHARE: f32 = 0.25;

/// `checkForIdolPedestalDuringFall`: a falling idol lands in a pedestal when it is within 20
/// (on each axis) of the point 43 above the pedestal's position.
const PEDESTAL_Y_OFFSET: f32 = 43.0;
const PEDESTAL_HIT_DISTANCE: f32 = 20.0;

/// `ejectFromIdolPedestal`: the idol pops up by this much, with this velocity.
const PEDESTAL_EJECT_LIFT: f32 = 20.0;
const PEDESTAL_EJECT_VEL: Vec2 = Vec2::new(-200.0, 500.0);

/// `enterIsland:` drops the idol in from this height and depth.
const ENTER_Y_POS: f32 = 400.0;
const ENTER_Z_POS: f32 = -277.12;

/// `IdolPedestalLogic::godIdolSlot` of a pedestal without an idol.
const NO_GOD_IDOL_SLOT: i32 = -1;

/// Sequence-item handlers implemented by `GodIdolLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "setAnimationBasedOnIdolSlot" => seq_set_animation_based_on_idol_slot(world, this, call),
        "resetLastPos" => seq_reset_last_pos(world, this, call),
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[GodIdolLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&GOD_IDOL_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// The listener (the view controller).
fn god_idol_listener(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<GodIdolLogic>(this).god_idol_logic_listener
}

/// `[mGodIdolLogicListener isStoryActive]`.
fn is_story_active(world: &mut World, this: Entity) -> bool {
    match god_idol_listener(world, this) {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    }
}

/// `[mGodIdolLogicListener isGodIdolEnabled]`.
fn is_god_idol_enabled(world: &mut World, this: Entity) -> bool {
    match god_idol_listener(world, this) {
        Some(listener) => crate::controller::is_god_idol_enabled(world, listener),
        None => false,
    }
}

/// `[mGodIdolLogicListener onGodIdolDragComplete:self]`.
fn send_drag_complete(world: &mut World, this: Entity) {
    if let Some(listener) = god_idol_listener(world, this) {
        crate::controller::on_god_idol_drag_complete(world, listener, this);
    }
}

/// `[[BCLibrary getDisplayMarkerById:@"PygmyDisplayMarker"] insertBefore:mDisplayObject]`.
fn insert_before_pygmy_display_marker(world: &mut World, this: Entity) {
    let marker = library::display_marker(world, "PygmyDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `[[BCLibrary getDisplayMarkerById:@"DragDisplayMarker"] insertAfter:mDisplayObject]`.
fn insert_after_drag_display_marker(world: &mut World, this: Entity) {
    let marker = library::display_marker(world, "DragDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// The idol leaves its pedestal (`exitIsland:`, `die:::`): `mbDontHitUntilBounce = YES`,
/// `[mIdolPedestalLogic ejectGodIdolFromIdolPedestal:self]`, then `mIdolPedestalLogic = nil`.
fn leave_idol_pedestal(world: &mut World, this: Entity) {
    let pedestal = world.obj::<GodIdolLogic>(this).idol_pedestal_logic;
    let Some(pedestal) = pedestal else {
        return;
    };

    world.obj_mut::<GodIdolLogic>(this).dont_hit_until_bounce = true;
    idol_pedestal_logic::eject_god_idol_from_idol_pedestal(world, pedestal, this);
    world.obj_mut::<GodIdolLogic>(this).idol_pedestal_logic = None;
}

/// `FUN_100038648(mSkinTextureBitsDefArray, node->idolTexture, @"raw")`: uploads the idol image
/// of episode node `node` into this idol's texture slot. The original dereferences both
/// pointers.
fn activate_idol_texture(world: &mut World, this: Entity, node: Option<usize>) {
    let skin_texture_bits_def_array = world.obj::<GodIdolLogic>(this).skin_texture_bits_def_array;
    let skin_texture_bits_def_array =
        skin_texture_bits_def_array.expect("GodIdolLogic: loadGame sets mSkinTextureBitsDefArray before the idol is used");
    let node = node.expect("GodIdolLogic: the idol's episode node is NULL where the original dereferences it");

    let idol_texture = episode_table(world).def(node).idol_texture.as_str();
    activate_skin_texture_bits_def_array(world, skin_texture_bits_def_array, idol_texture);
}

/// `[[pedestal displayObject] pos].x`.
fn pedestal_x_pos(world: &World, pedestal: Entity) -> f32 {
    let pedestal_display_object = logic::display_object(world, pedestal);
    display::object(world, pedestal_display_object).pos.x
}

/// `mTouchScreenPos = [touch pos]`.
fn track_touch(world: &mut World, this: Entity, touch: Entity) {
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<GodIdolLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[GodIdolLogic initWithDisplayObject:]` @ 0x100118e90
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(GodIdolLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut idol = world.obj_mut::<GodIdolLogic>(this);
        idol.tilt_state = 0;
        idol.slide_friction = SLIDE_FRICTION;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<GodIdolLogic>(this).enable_island_floor_collision = true;

    let own_display_object = logic::display_object(world, this);
    let mut display = display::object_mut(world, own_display_object);
    display.col_x_min = -40.0;
    display.col_x_max = 40.0;
    display.col_y_min = -0.0;
    display.col_y_max = 80.0;
}

/// `-[GodIdolLogic setGodIdolLogicListener:]` @ 0x100118f9c
pub fn set_god_idol_logic_listener(world: &mut World, this: Entity, god_idol_logic_listener: Entity) {
    world.obj_mut::<GodIdolLogic>(this).god_idol_logic_listener = Some(god_idol_logic_listener);
}

/// `-[GodIdolLogic setSkinTextureBitsDefArray:]` @ 0x100118ff8
///
/// The array is an index into the idol screen's `SKIN_TEXTURE_BITS_DEF_ARRAYS`.
pub fn set_skin_texture_bits_def_array(world: &mut World, this: Entity, skin_texture_bits_def_array: Option<usize>) {
    world.obj_mut::<GodIdolLogic>(this).skin_texture_bits_def_array = skin_texture_bits_def_array;
}

/// `-[GodIdolLogic slot]` @ 0x100119008
pub fn slot(world: &mut World, this: Entity) -> i32 {
    world.obj::<GodIdolLogic>(this).slot
}

/// `-[GodIdolLogic setSlot:]` @ 0x100119018
pub fn set_slot(world: &mut World, this: Entity, slot: i32) {
    world.obj_mut::<GodIdolLogic>(this).slot = slot;
}

/// `-[GodIdolLogic structXmlEpisodeNodeDef]` @ 0x100119028
pub fn struct_xml_episode_node_def(world: &mut World, this: Entity) -> Option<usize> {
    world.obj::<GodIdolLogic>(this).struct_xml_episode_node_def
}

/// `-[GodIdolLogic updateIdolTexture]` @ 0x100119038
///
/// Reloads the idol image: the current episode's while on the island, else the last one's
/// while the idol is still in the display tree.
pub fn update_idol_texture(world: &mut World, this: Entity) {
    let active = world.obj::<GodIdolLogic>(this).active;
    if active {
        let node = world.obj::<GodIdolLogic>(this).struct_xml_episode_node_def;
        activate_idol_texture(world, this, node);
        return;
    }

    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    if display_group.is_none() {
        return;
    }

    let last_node = world.obj::<GodIdolLogic>(this).last_struct_xml_episode_node_def;
    if last_node.is_none() {
        return;
    }
    activate_idol_texture(world, this, last_node);
}

/// `-[GodIdolLogic setStructXmlEpisodeNodeDef:]` @ 0x1001190d8
///
/// Shows the idol of episode node `struct_xml_episode_node_def` (an index into the idol
/// manager's `EpisodeTable`), or removes the idol for `None`.
pub fn set_struct_xml_episode_node_def(world: &mut World, this: Entity, struct_xml_episode_node_def: Option<usize>) {
    {
        let mut idol = world.obj_mut::<GodIdolLogic>(this);
        if idol.struct_xml_episode_node_def.is_some() {
            idol.last_struct_xml_episode_node_def = idol.struct_xml_episode_node_def;
        }
        idol.struct_xml_episode_node_def = struct_xml_episode_node_def;
    }

    if struct_xml_episode_node_def.is_none() {
        let active = world.obj::<GodIdolLogic>(this).active;
        if active {
            exit_island(world, this, false);
        }
        return;
    }

    let god_idol_enabled = is_god_idol_enabled(world, this);
    if !god_idol_enabled {
        return;
    }

    let active = world.obj::<GodIdolLogic>(this).active;
    if active {
        let node = world.obj::<GodIdolLogic>(this).struct_xml_episode_node_def;
        activate_idol_texture(world, this, node);
    } else {
        enter_island(world, this, false);
    }
}

/// `-[GodIdolLogic ejectFromIdolPedestal]` @ 0x1001191b0
///
/// The pedestal throws the idol off (up and to the left).
pub fn eject_from_idol_pedestal(world: &mut World, this: Entity) {
    let pedestal = world.obj::<GodIdolLogic>(this).idol_pedestal_logic;
    if pedestal.is_none() {
        return;
    }
    world.obj_mut::<GodIdolLogic>(this).idol_pedestal_logic = None;

    let display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.y = display.pos.y + PEDESTAL_EJECT_LIFT;
        display.scale.x = 1.0;
        display.scale.y = 1.0;
    }
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = PEDESTAL_EJECT_VEL.x;
        state.pos_vel.y = PEDESTAL_EJECT_VEL.y;
    }

    world.obj_mut::<GodIdolLogic>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut idol = world.obj_mut::<GodIdolLogic>(this);
        idol.enable_island_floor_collision = true;
        idol.dont_hit_until_bounce = true;
    }
    logic::set_behavior(world, this, Some("GodIdolEjectFromPedestal"));
}

/// `-[GodIdolLogic checkForIdolPedestalDuringFall]` @ 0x1001192b8
///
/// Seats a falling idol in the first free pedestal it drops onto.
pub fn check_for_idol_pedestal_during_fall(world: &mut World, this: Entity) -> bool {
    let dont_hit_until_bounce = world.obj::<GodIdolLogic>(this).dont_hit_until_bounce;
    if dont_hit_until_bounce {
        return false;
    }

    let Some(listener) = god_idol_listener(world, this) else {
        return false;
    };
    let pedestal_enabled = crate::controller::is_idol_pedestal_enabled(world, listener);
    if !pedestal_enabled {
        return false;
    }

    let own_display_object = logic::display_object(world, this);
    let pedestals = crate::controller::idol_pedestal_logic_array(world, listener, this);
    for pedestal in pedestals {
        let available = idol_pedestal_logic::is_available(world, pedestal);
        if !available {
            continue;
        }

        let pedestal_display_object = logic::display_object(world, pedestal);
        let pedestal_pos = display::object(world, pedestal_display_object).pos;
        let pos = display::object(world, own_display_object).pos;

        let falling_down = logic::logic(world, this).pos_vel.y < 0.0;
        if !falling_down {
            continue;
        }

        let x_distance = (pos.x - pedestal_pos.x).abs();
        let y_distance = (pos.y - (pedestal_pos.y + PEDESTAL_Y_OFFSET)).abs();
        let over_pedestal = x_distance <= PEDESTAL_HIT_DISTANCE && y_distance <= PEDESTAL_HIT_DISTANCE;
        if !over_pedestal {
            continue;
        }

        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        {
            let mut idol = world.obj_mut::<GodIdolLogic>(this);
            idol.enable_island_floor_collision = false;
            idol.idol_pedestal_logic = Some(pedestal);
        }
        idol_pedestal_logic::god_idol_fall_into_idol_pedestal(world, pedestal, this);

        let seated_pedestal = world.obj::<GodIdolLogic>(this).idol_pedestal_logic;
        if let Some(seated_pedestal) = seated_pedestal {
            let seated_display_object = logic::display_object(world, seated_pedestal);
            display::insert_after(world, seated_display_object, Some(own_display_object));
        }
        logic::set_behavior(world, this, Some("GodIdolFallIntoIdolPedestal"));
        return true;
    }
    false
}

/// `-[GodIdolLogic enterIsland:]` @ 0x100119584
///
/// Drops the idol in: straight onto the pedestal that remembers its slot, else above the free
/// pedestal matching its rank among the idols not yet placed, else at a random x.
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let idol = world.obj::<GodIdolLogic>(this);
    if idol.struct_xml_episode_node_def.is_none() || idol.falling || idol.active {
        return;
    }

    world.obj_mut::<GodIdolLogic>(this).active = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<GodIdolLogic>(this).in_water = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<GodIdolLogic>(this).enable_island_floor_collision = false;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;

    // x in [100, 300] in tenths: `(arc4random() % 2001 + 1000) / 10`.
    let random_tenths = world.resource_mut::<Random>().arc4random_mod(2001);
    let x_tenths = random_tenths + 1000;
    let x_pos = x_tenths as i32 as f32 / 10.0;
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = x_pos;
        display.pos.y = ENTER_Y_POS;
        display.pos.z = ENTER_Z_POS;
    }
    world.obj_mut::<GodIdolLogic>(this).dont_hit_until_bounce = true;

    let mut pedestal_rank = world.obj::<GodIdolLogic>(this).slot;
    let pedestals = match god_idol_listener(world, this) {
        Some(listener) => crate::controller::idol_pedestal_logic_array(world, listener, this),
        None => Vec::new(),
    };

    'placed: {
        for &pedestal in &pedestals {
            let pedestal_slot = idol_pedestal_logic::god_idol_slot(world, pedestal);
            let own_slot = world.obj::<GodIdolLogic>(this).slot;
            if pedestal_slot == own_slot {
                let x_pos = pedestal_x_pos(world, pedestal);
                display::object_mut(world, display_object).pos.x = x_pos;
                world.obj_mut::<GodIdolLogic>(this).dont_hit_until_bounce = false;
                break 'placed;
            }

            let holds_lower_slot = pedestal_slot != NO_GOD_IDOL_SLOT && pedestal_slot < own_slot;
            pedestal_rank = pedestal_rank.wrapping_sub(holds_lower_slot as i32);
        }

        if pedestal_rank == -1 {
            break 'placed;
        }

        for &pedestal in &pedestals {
            let pedestal_slot = idol_pedestal_logic::god_idol_slot(world, pedestal);
            if pedestal_slot != NO_GOD_IDOL_SLOT {
                continue;
            }

            if pedestal_rank == 0 {
                let x_pos = pedestal_x_pos(world, pedestal);
                display::object_mut(world, display_object).pos.x = x_pos;
                break 'placed;
            }
            pedestal_rank = pedestal_rank.wrapping_sub(1);
        }
    }

    display::object_mut(world, display_object).scale.x = 1.0;
    let node = world.obj::<GodIdolLogic>(this).struct_xml_episode_node_def;
    activate_idol_texture(world, this, node);
    display::object_mut(world, display_object).is_selectable = true;
    insert_before_pygmy_display_marker(world, this);
    logic::set_behavior(world, this, Some("GodIdolEnterIsland"));
}

/// `-[GodIdolLogic exitIsland:]` @ 0x1001199e4
///
/// Takes the idol off the island: it dies at once when `now`, else it falls away.
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let active = world.obj::<GodIdolLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<GodIdolLogic>(this).active = false;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;

    leave_idol_pedestal(world, this);

    let dragging = world.obj::<GodIdolLogic>(this).dragging;
    if dragging {
        {
            let mut idol = world.obj_mut::<GodIdolLogic>(this);
            idol.falling = true;
            idol.dragging = false;
            idol.apply_spring = false;
        }
        send_drag_complete(world, this);
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<GodIdolLogic>(this).enable_island_floor_collision = false;

    if now {
        logic::set_behavior(world, this, Some("GodIdolDie"));
        return;
    }

    let in_water = world.obj::<GodIdolLogic>(this).in_water;
    if in_water {
        return;
    }

    insert_after_drag_display_marker(world, this);
    logic::set_behavior(world, this, Some("GodIdolFallIgnoreIsland"));
}

/// `-[GodIdolLogic exitIslandNow]` @ 0x100119b84
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[GodIdolLogic refreshTiltState]` @ 0x100119bfc
///
/// A resting idol falls once the island is tipped far enough (roll within ±90°).
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let ignore_gravity = world.obj::<SpriteLogic>(this).ignore_gravity;
    let dragging = world.obj::<GodIdolLogic>(this).dragging;
    if ignore_gravity || dragging {
        return;
    }

    let roll = world.resource::<BcView>().accel_rot_degrees.z;
    let tipped_over = roll > -90.0 && roll < 90.0;
    if !tipped_over {
        return;
    }

    world.obj_mut::<GodIdolLogic>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<GodIdolLogic>(this).tilt_state = 0;
    logic::set_behavior(world, this, Some("GodIdolFall"));
}

/// `-[GodIdolLogic undoTiltState]` @ 0x100119ccc
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    world.obj_mut::<GodIdolLogic>(this).tilt_state = 0;
}

/// `-[GodIdolLogic isAccelEnabled]` @ 0x100119cdc
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let gravity_enabled = match god_idol_listener(world, this) {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }

    let display_object = logic::display_object(world, this);
    let selectable = display::object(world, display_object).is_selectable;
    if !selectable {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[GodIdolLogic onGameFrame:]` @ 0x100119d70
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }
    let display_object = logic::display_object(world, this);

    let accel_enabled = is_accel_enabled(world, this);
    if accel_enabled {
        // `(-accelForce.y, accelForce.x, 0, 1)` normalised; y and z scaled, and x dropped.
        let accel_force = world.resource::<BcView>().accel_force;
        let tilted = Vec4::new(-accel_force.y, accel_force.x, 0.0, 1.0);
        let mut gravity_force = normalize3(tilted);
        gravity_force.y = gravity_force.y * TILTED_GRAVITY;
        gravity_force.z = gravity_force.z * TILTED_GRAVITY;
        gravity_force.x = 0.0;
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = GRAVITY_FORCE;
        undo_tilt_state(world, this);
    }

    let mut pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let sprite = world.obj::<SpriteLogic>(this);
    if !sprite.ignore_gravity {
        let gravity_force = sprite.gravity_force;
        pos_force.x = gravity_force.x + pos_force.x;
        pos_force.y = gravity_force.y + pos_force.y;
        pos_force.z = gravity_force.z + pos_force.z;
    }
    let idol = world.obj::<GodIdolLogic>(this);
    if idol.apply_spring {
        let spring_force = idol.spring_force;
        pos_force.x = spring_force.x + pos_force.x;
        pos_force.y = spring_force.y + pos_force.y;
        pos_force.z = spring_force.z + pos_force.z;
    }
    logic::logic_mut(world, this).pos_force = pos_force;

    // The velocity first, then the position moves with the new velocity.
    let pos_vel = {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = state.pos_vel.x + pos_force.x * dt;
        state.pos_vel.y = state.pos_vel.y + pos_force.y * dt;
        state.pos_vel.z = state.pos_vel.z + pos_force.z * dt;
        state.pos_vel
    };
    let pos = {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = display.pos.x + pos_vel.x * dt;
        display.pos.y = display.pos.y + pos_vel.y * dt;
        display.pos.z = display.pos.z + pos_vel.z * dt;
        display.pos
    };

    world.obj_mut::<GodIdolLogic>(this).island_floor_collision = false;

    let idol = world.obj::<GodIdolLogic>(this);
    let over_island = FLOOR_MIN_X <= pos.x && pos.x <= FLOOR_MAX_X;
    let crossed_floor = FLOOR_Y <= idol.last_pos.y && pos.y < FLOOR_Y;
    let reached_floor = over_island && crossed_floor && idol.enable_island_floor_collision;

    let mut y_pos = pos.y;
    if reached_floor {
        let on_island = idol.on_island;
        {
            let mut state = logic::logic_mut(world, this);
            if on_island {
                state.pos_vel.y = 0.0;
            } else {
                state.pos_vel.x = state.pos_vel.x * 0.5;
                state.pos_vel.y = state.pos_vel.y * -0.4;
            }
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let settled = pos_vel.x.abs() < SETTLE_SPEED && pos_vel.y < SETTLE_SPEED;
        if settled {
            logic::logic_mut(world, this).pos_vel.y = 0.0;
            world.obj_mut::<GodIdolLogic>(this).dont_hit_until_bounce = false;

            let on_island = world.obj::<GodIdolLogic>(this).on_island;
            if !on_island {
                world.obj_mut::<GodIdolLogic>(this).on_island = true;
                logic::logic_mut(world, this).pos_vel.x = 0.0;
                world.obj_mut::<GodIdolLogic>(this).island_floor_collision = true;
            }
        } else if pos_vel.y > BOUNCE_SPEED {
            logic::set_behavior(world, this, Some("GodIdolBounce"));
        }

        {
            let mut display = display::object_mut(world, display_object);
            display.rot.z = 0.0;
            display.pos.y = FLOOR_Y;
        }
        y_pos = FLOOR_Y;
    } else {
        world.obj_mut::<GodIdolLogic>(this).on_island = false;
    }

    let out_of_range = y_pos < MIN_Y || y_pos > MAX_Y;
    if out_of_range {
        logic::set_behavior(world, this, Some("GodIdolDie"));
    }

    let last_pos = display::object(world, display_object).pos;
    world.obj_mut::<GodIdolLogic>(this).last_pos = last_pos;
}

/// `-[GodIdolLogic setAnimationBasedOnIdolSlot:::]` @ 0x10011a1a8
///
/// Plays `slot1` … `slot4` for slot 0–3 (nil for other slots).
pub fn seq_set_animation_based_on_idol_slot(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let slot = world.obj::<GodIdolLogic>(this).slot;
    let animation_id = match slot {
        0 => call.param("slot1"),
        1 => call.param("slot2"),
        2 => call.param("slot3"),
        3 => call.param("slot4"),
        _ => None,
    };

    let animation = logic::animation(world, this);
    animation::play_animation_id(world, animation, animation_id);
    ItemResult::Done
}

/// `-[GodIdolLogic resetLastPos:::]` @ 0x10011a248
pub fn seq_reset_last_pos(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    world.obj_mut::<GodIdolLogic>(this).last_pos = pos;
    ItemResult::Done
}

/// `-[GodIdolLogic drag:::]` @ 0x10011a298
///
/// Spring drag toward the finger (k 50, damping 6), tilting the idol along the drag.
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<GodIdolLogic>(this).enable_island_floor_collision = false;
    }

    let display_object = logic::display_object(world, this);
    let own_display = display::object(world, display_object);
    let pos = own_display.pos;
    let display_group = own_display.display_group;

    // `[nil viewToDisplayObject:]` answers the zero point.
    let touch_screen_pos = world.obj::<GodIdolLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // The binary reloads pos.z over the touch point's z before subtracting, so the offset has
    // no depth (`pos.z - pos.z`); w is the position's.
    let offset_x = pos.x - touch_pos.x;
    let offset_y = pos.y - touch_pos.y;
    let offset_z = pos.z - pos.z;
    let offset = Vec4::new(offset_x, offset_y, offset_z, pos.w);

    let length = length3(offset);
    if length == 0.0 {
        return ItemResult::Pending;
    }

    if offset.y != 0.0 {
        let angle = (-offset.y).atan2(-offset.x);
        let z_rot = (rad_to_deg(angle) + -90.0) * DRAG_TILT_SHARE;
        display::object_mut(world, display_object).rot.z = z_rot;
    }

    let pos_vel = logic::logic(world, this).pos_vel;
    let stretch = (length * -DRAG_STIFFNESS) / length;

    let damping_x = -DRAG_DAMPING * pos_vel.x;
    let damping_y = -DRAG_DAMPING * pos_vel.y;
    let damping_z = -DRAG_DAMPING * pos_vel.z;

    let spring_x = damping_x + stretch * offset.x;
    let spring_y = damping_y + stretch * offset.y;
    let spring_z = damping_z + stretch * offset.z;
    let spring_force = Vec4::new(spring_x, spring_y, spring_z, offset.w);
    world.obj_mut::<GodIdolLogic>(this).spring_force = spring_force;
    ItemResult::Pending
}

/// `-[GodIdolLogic fall:::]` @ 0x10011a450
///
/// Falls (`ignoreIsland` disables the floor) until it lands in a pedestal (3), on the island
/// (2) or in the water (`GodIdolSplash`, 3).
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut idol = world.obj_mut::<GodIdolLogic>(this);
            if !idol.island_floor_collision {
                idol.on_island = false;
            }
            idol.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<GodIdolLogic>(this).falling = true;

        let ignore_island = call.bool("ignoreIsland");
        if ignore_island {
            world.obj_mut::<GodIdolLogic>(this).enable_island_floor_collision = false;
        }
    }

    let landed_in_pedestal = check_for_idol_pedestal_during_fall(world, this);
    if landed_in_pedestal {
        world.obj_mut::<GodIdolLogic>(this).falling = false;
        return ItemResult::Replaced;
    }

    let display_object = logic::display_object(world, this);
    let y_pos = display::object(world, display_object).pos.y;

    let island_floor_collision = world.obj::<GodIdolLogic>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<GodIdolLogic>(this).falling = false;
        return ItemResult::Done;
    }

    let in_water = y_pos < WATER_LINE_Y;
    if !in_water {
        return ItemResult::Pending;
    }

    {
        let mut idol = world.obj_mut::<GodIdolLogic>(this);
        idol.falling = false;
        idol.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("GodIdolSplash"));
    ItemResult::Replaced
}

/// `-[GodIdolLogic die:::]` @ 0x10011a5d8
///
/// Removes the idol and resets it; it comes straight back while idols are enabled and no
/// story is playing.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    let dragging = world.obj::<GodIdolLogic>(this).dragging;
    if dragging {
        world.obj_mut::<GodIdolLogic>(this).dragging = false;
        send_drag_complete(world, this);
    }

    leave_idol_pedestal(world, this);

    {
        let mut idol = world.obj_mut::<GodIdolLogic>(this);
        idol.active = false;
        idol.falling = false;
        idol.in_water = false;
        idol.apply_spring = false;
        idol.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let god_idol_enabled = is_god_idol_enabled(world, this);
    if god_idol_enabled {
        let story_active = is_story_active(world, this);
        if !story_active {
            enter_island(world, this, false);
        }
    }
    ItemResult::Done
}

/// `-[GodIdolLogic touchBeganInside:allTouches:]` @ 0x10011a73c
///
/// Picks the idol up (off its pedestal, whose saved slot is cleared) and starts the drag.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let story_active = is_story_active(world, this);
    if story_active {
        return;
    }

    let pedestal = world.obj::<GodIdolLogic>(this).idol_pedestal_logic;
    if let Some(pedestal) = pedestal {
        world.obj_mut::<GodIdolLogic>(this).dont_hit_until_bounce = true;
        idol_pedestal_logic::clear_god_idol_slot(world, pedestal, this);

        let pedestal = world.obj::<GodIdolLogic>(this).idol_pedestal_logic;
        if let Some(pedestal) = pedestal {
            idol_pedestal_logic::eject_god_idol_from_idol_pedestal(world, pedestal, this);
        }
        world.obj_mut::<GodIdolLogic>(this).idol_pedestal_logic = None;
    }

    let idol = world.obj::<GodIdolLogic>(this);
    if idol.falling || idol.in_water || idol.active_touch.is_some() {
        return;
    }

    world.obj_mut::<GodIdolLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    track_touch(world, this, touch);
    {
        let mut idol = world.obj_mut::<GodIdolLogic>(this);
        idol.tilt_state = 0;
        idol.dont_hit_until_bounce = false;
        idol.dragging = true;
        idol.apply_spring = true;
        idol.enable_island_floor_collision = false;
    }

    if let Some(listener) = god_idol_listener(world, this) {
        crate::controller::on_god_idol_drag(world, listener, this);
    }
    insert_after_drag_display_marker(world, this);
    logic::set_behavior(world, this, Some("GodIdolDrag"));
}

/// `-[GodIdolLogic touchBeganOutside:allTouches:]` @ 0x10011a914
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<GodIdolLogic>(this).active_touch;
    if active_touch.is_none() {
        track_touch(world, this, touch);
    }
}

/// `-[GodIdolLogic touchMovedEntered:allTouches:]` @ 0x10011a964
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<GodIdolLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        track_touch(world, this, touch);
    }
}

/// `-[GodIdolLogic touchMovedExited:allTouches:]` @ 0x10011a9bc
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<GodIdolLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        track_touch(world, this, touch);
    }
}

/// `-[GodIdolLogic touchMovedInside:allTouches:]` @ 0x10011aa14
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[GodIdolLogic touchMovedOutside:allTouches:]` @ 0x10011aa24
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<GodIdolLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        track_touch(world, this, touch);
    }
}

/// `-[GodIdolLogic touchEndedInside:allTouches:]` @ 0x10011aa7c
///
/// Lets go of a dragged idol: it falls.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<GodIdolLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let dragging = world.obj::<GodIdolLogic>(this).dragging;
    if dragging {
        {
            let mut idol = world.obj_mut::<GodIdolLogic>(this);
            idol.falling = true;
            idol.dragging = false;
            idol.apply_spring = false;
        }
        insert_before_pygmy_display_marker(world, this);
        send_drag_complete(world, this);
        logic::set_behavior(world, this, Some("GodIdolFall"));
    }

    world.obj_mut::<GodIdolLogic>(this).active_touch = None;
}

/// `-[GodIdolLogic touchEndedOutside:allTouches:]` @ 0x10011ab74
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
