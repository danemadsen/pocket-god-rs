//! `PiranhaLogic` (superclass `SpriteLogic`).
//!
//! A Bait Master piranha: the player draws a path from it with the finger, it follows the
//! path and eats the pygmy the path ends on; piranhas that meet fight. Its listener is always
//! the `AnchorLogic` that spawned it.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::underwater_baitmaster_laser::anchor_logic;
use crate::logic::underwater_baitmaster_laser::path_model::{self, PATH_MODEL};
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;
use pg_engine::math::{deg_to_rad, length3, rad_to_deg};
use pg_engine::touch::BcTouch;

/// `PiranhaLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct PiranhaLogic {
    /// `mPiranhaLogicListener`: `@"<PiranhaLogicListener>"`
    pub piranha_logic_listener: Option<Entity>,

    /// `mbFollowingPath`: `B`
    pub following_path: bool,

    /// `mbFlipping`: `B`
    pub flipping: bool,

    /// `mbPathLeadsToPygmy`: `B`
    pub path_leads_to_pygmy: bool,

    /// `mPathPygmyLogic`: `@"PygmyLogic"`
    pub path_pygmy_logic: Option<Entity>,

    /// `mEatingPygmyLogic`: `@"PygmyLogic"`
    pub eating_pygmy_logic: Option<Entity>,

    /// `mbEating`: `B`
    pub eating: bool,

    /// `mbFinishEating`: `B`
    pub finish_eating: bool,

    /// `mbAgitated`: `B`
    pub agitated: bool,

    /// `mbFighting`: `B`
    pub fighting: bool,

    /// `mbStopBaitMaster`: `B`
    pub stop_bait_master: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mpPathVector`: `^{vector<VECTOR4, std::__1::allocator<VECTOR4> >=^{?}^{?}{__compressed_pair<VECTOR4 *, std::__1::allocator<VECTOR4> >=^{?}}}`
    pub path_vector: Vec<Vec4>,

    /// `mPathModel`: `@"PathModel"`
    pub path_model: Option<Entity>,

    /// `mLastPathPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_path_pos: Vec4,

    /// `mTouchXOffset`: `f`
    pub touch_x_offset: f32,

    /// `mTouchYOffset`: `f`
    pub touch_y_offset: f32,

    /// `mTargetZRot`: `f`
    pub target_z_rot: f32,
}

pub const PIRANHA_LOGIC: Class = Class {
    name: "PiranhaLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    shutdown,
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

/// Swimming speed along a path and after eating or stopping to eat.
const SWIM_SPEED: f32 = 50.0;

/// Speed when Bait Master ends and the piranha leaves the screen.
const SWIM_AWAY_SPEED: f32 = 200.0;

/// `touchMovedOutside:allTouches:` stops drawing once the path holds this many points.
const MAX_PATH_POINTS: usize = 41;

/// A pygmy's mouth, where paths end and piranhas bite: this far above its position.
const PYGMY_MOUTH_Y_OFFSET: f32 = 32.0;

/// Sequence-item handlers implemented by `PiranhaLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "initPos" => seq_init_pos(world, this, call),
        "swim" => seq_swim(world, this, call),
        "swimAway" => seq_swim_away(world, this, call),
        "die" => seq_die(world, this, call),
        "attack" => seq_attack(world, this, call),
        "eatPygmyComplete" => seq_eat_pygmy_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `PiranhaLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animPiranhaYFlip" => anim_piranha_y_flip(world, this),
        "animFlipComplete" => anim_flip_complete(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[PiranhaLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&PIRANHA_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[PiranhaLogic initWithDisplayObject:]` @ 0x1000091dc
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(PiranhaLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    // `mpPathVector = new std::vector<VECTOR4>`: the component starts with an empty path.
    logic::set_behavior(world, this, Some("PiranhaInit"));
}

/// `-[PiranhaLogic setPiranhaLogicListener:]` @ 0x100009264
pub fn set_piranha_logic_listener(world: &mut World, this: Entity, piranha_logic_listener: Entity) {
    world.obj_mut::<PiranhaLogic>(this).piranha_logic_listener = Some(piranha_logic_listener);
}

/// `-[PiranhaLogic dealloc]` @ 0x10000a7a8
///
/// For the owner that releases the piranha last: the controller's `onRemovePiranhaLogic:` and
/// `onRemoveAllPiranhaLogics:` (`mPiranhaLogicArray`). Takes the drawn path off the display.
///
/// PORT: the original frees the object only at the end of the game frame (`BCView::gameLoop`
/// retains every listener for the frame) and the piranha's own sequence keeps running items on
/// it after it was released (`PiranhaSwimAway` goes on with `runSequence PiranhaInit`). The
/// entity therefore stays alive, shut down and unreachable, instead of going through
/// `BCLogic`'s dealloc.
pub fn dealloc(world: &mut World, this: Entity) {
    let path_model = world.obj_mut::<PiranhaLogic>(this).path_model.take();
    if let Some(path_model) = path_model {
        display::remove_from_display_group(world, path_model);

        // `[mPathModel release]`: the piranha was its only owner.
        path_model::free_model(world, path_model);
        world.despawn(path_model);
    }
}

/// `-[PiranhaLogic animPiranhaYFlip]` @ 0x1000092c0
pub fn anim_piranha_y_flip(world: &mut World, this: Entity) {
    let target_z_rot = world.obj::<PiranhaLogic>(this).target_z_rot;
    let y_scale = upright_y_scale(target_z_rot);

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).scale.y = y_scale;
}

/// The `yScale` that keeps the piranha upright at a heading: −1 while it faces left.
fn upright_y_scale(target_z_rot: f32) -> f32 {
    let facing_left = (target_z_rot > 90.0 && target_z_rot < 270.0) || target_z_rot < -90.0;
    if facing_left { -1.0 } else { 1.0 }
}

/// `-[PiranhaLogic animFlipComplete]` @ 0x100009334
pub fn anim_flip_complete(world: &mut World, this: Entity) {
    world.obj_mut::<PiranhaLogic>(this).flipping = false;
}

/// `-[PiranhaLogic cancelFlip]` @ 0x100009344
pub fn cancel_flip(world: &mut World, this: Entity) {
    let flipping = world.obj::<PiranhaLogic>(this).flipping;
    if !flipping {
        return;
    }
    world.obj_mut::<PiranhaLogic>(this).flipping = false;
    anim_piranha_y_flip(world, this);
}

/// `-[PiranhaLogic checkPiranhaCollision]` @ 0x10000936c
///
/// Another piranha within 60 makes this one agitated (unless it is eating); within 30 the two
/// fight, which ends Bait Master.
pub fn check_piranha_collision(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    let (fighting, stop_bait_master, listener) = {
        let piranha = world.obj::<PiranhaLogic>(this);
        (piranha.fighting, piranha.stop_bait_master, piranha.piranha_logic_listener)
    };
    if fighting || stop_bait_master || pos.x > 480.0 || pos.x < 0.0 {
        return;
    }

    let piranhas = match listener {
        Some(listener) => anchor_logic::piranha_logic_array(world, listener, this),
        None => Vec::new(),
    };

    let mut near_other_piranha = false;
    for other in piranhas {
        if other == this {
            continue;
        }

        let other_display_object = logic::display_object(world, other);
        let other_pos = display::object(world, other_display_object).pos;
        let offset = Vec4::new(pos.x - other_pos.x, pos.y - other_pos.y, pos.z - other_pos.z, pos.w);
        let distance = length3(offset);

        let other_on_screen = other_pos.x < 480.0 && other_pos.x > 0.0;
        if distance < 60.0 && other_on_screen {
            let eating = world.obj::<PiranhaLogic>(this).eating;
            near_other_piranha = near_other_piranha || !eating;
        }

        if distance < 30.0 {
            fight_piranha(world, this, None);
            fight_piranha(world, other, Some(this));
            if let Some(listener) = listener {
                anchor_logic::on_piranha_fight(world, listener, this);
            }
            return;
        }
    }

    let (agitated, finish_eating) = {
        let piranha = world.obj::<PiranhaLogic>(this);
        (piranha.agitated, piranha.finish_eating)
    };

    if near_other_piranha {
        if agitated {
            return;
        }
        world.obj_mut::<PiranhaLogic>(this).agitated = true;
        cancel_flip(world, this);
        logic::set_behavior(world, this, Some("PiranhaAgitated"));
        return;
    }

    if !agitated {
        return;
    }
    world.obj_mut::<PiranhaLogic>(this).agitated = false;

    let behavior = if finish_eating { "PiranhaSwimFull" } else { "PiranhaSwim" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[PiranhaLogic onGameFrame:]` @ 0x1000096bc
///
/// Integrates without any force (no `[super onGameFrame:]`), then checks for other piranhas.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let pos_vel = {
        let mut state = logic::logic_mut(world, this);
        state.pos_force = pos_force;
        state.pos_vel.x = state.pos_vel.x + pos_force.x * dt;
        state.pos_vel.y = state.pos_vel.y + pos_force.y * dt;
        state.pos_vel.z = state.pos_vel.z + pos_force.z * dt;
        state.pos_vel
    };

    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = object.pos.x + pos_vel.x * dt;
        object.pos.y = object.pos.y + pos_vel.y * dt;
        object.pos.z = object.pos.z + pos_vel.z * dt;
    }

    check_piranha_collision(world, this);
}

/// `-[PiranhaLogic clearPath]` @ 0x1000097d4
pub fn clear_path(world: &mut World, this: Entity) {
    let (path_model, path) = {
        let mut piranha = world.obj_mut::<PiranhaLogic>(this);
        piranha.touch_began_inside = false;
        piranha.path_vector.clear();
        (piranha.path_model, piranha.path_vector.clone())
    };

    if let Some(path_model) = path_model {
        path_model::set_path_vector(world, path_model, path);
    }
}

/// `posVel = 50·(cos, sin)(zRot)`: swim on along the current heading.
fn swim_along_heading(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let z_rot = display::object(world, display_object).rot.z;
    let (sin, cos) = deg_to_rad(z_rot).sin_cos();

    let mut state = logic::logic_mut(world, this);
    state.pos_vel.x = cos * SWIM_SPEED;
    state.pos_vel.y = sin * SWIM_SPEED;
}

/// `-[PiranhaLogic stopEatingPygmy:]` @ 0x100009830
pub fn stop_eating_pygmy(world: &mut World, this: Entity, eating_pygmy: Entity) {
    let is_eating_it = {
        let piranha = world.obj::<PiranhaLogic>(this);
        piranha.eating && piranha.eating_pygmy_logic == Some(eating_pygmy)
    };
    if !is_eating_it {
        return;
    }

    {
        let mut piranha = world.obj_mut::<PiranhaLogic>(this);
        piranha.eating = false;
        piranha.eating_pygmy_logic = None;
    }
    swim_along_heading(world, this);
    logic::set_behavior(world, this, Some("PiranhaSwim"));
}

/// `-[PiranhaLogic stopBaitMaster]` @ 0x100009900
pub fn stop_bait_master(world: &mut World, this: Entity) {
    {
        let mut piranha = world.obj_mut::<PiranhaLogic>(this);
        piranha.eating = false;
        piranha.stop_bait_master = true;
    }
    clear_path(world, this);

    let eating_pygmy = world.obj::<PiranhaLogic>(this).eating_pygmy_logic;
    if let Some(eating_pygmy) = eating_pygmy {
        crate::pygmy::stop_eaten_by_piranha(world, eating_pygmy, this);
        world.obj_mut::<PiranhaLogic>(this).eating_pygmy_logic = None;
    }

    let fighting = world.obj::<PiranhaLogic>(this).fighting;
    if fighting {
        return;
    }
    cancel_flip(world, this);
    logic::set_behavior(world, this, Some("PiranhaSwimAway"));
}

/// `-[PiranhaLogic fightPiranha:]` @ 0x1000099b8
///
/// `fight_piranha` is the opponent, or nil for the piranha that started the fight: the other
/// one jumps onto its position and turns to face it.
pub fn fight_piranha(world: &mut World, this: Entity, fight_piranha: Option<Entity>) {
    world.obj_mut::<PiranhaLogic>(this).eating = false;

    let eating_pygmy = world.obj::<PiranhaLogic>(this).eating_pygmy_logic;
    if let Some(eating_pygmy) = eating_pygmy {
        crate::pygmy::stop_eaten_by_piranha(world, eating_pygmy, this);
        world.obj_mut::<PiranhaLogic>(this).eating_pygmy_logic = None;
    }

    {
        let mut piranha = world.obj_mut::<PiranhaLogic>(this);
        piranha.fighting = true;
        piranha.agitated = false;
    }
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    let z_rot = match fight_piranha {
        None => {
            display::object_mut(world, display_object).scale.y = 1.0;
            0.0
        }
        Some(opponent) => {
            let opponent_display_object = logic::display_object(world, opponent);
            let opponent_pos = display::object(world, opponent_display_object).pos;

            let mut object = display::object_mut(world, display_object);
            object.pos.x = opponent_pos.x;
            object.pos.y = opponent_pos.y;
            object.scale.y = -1.0;
            180.0
        }
    };
    display::object_mut(world, display_object).rot.z = z_rot;

    cancel_flip(world, this);
    clear_path(world, this);
    logic::set_behavior(world, this, Some("PiranhaFight"));
}

/// `-[PiranhaLogic initPos:::]` @ 0x100009b4c
///
/// Heads into the screen from the side the piranha starts on.
pub fn seq_init_pos(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let pos_x = display::object(world, display_object).pos.x;
    logic::logic_mut(world, this).pos_vel.x = SWIM_SPEED;

    let target_z_rot = if pos_x > 0.0 {
        logic::logic_mut(world, this).pos_vel.x = -SWIM_SPEED;
        display::object_mut(world, display_object).rot.z = 180.0;
        180.0
    } else {
        display::object(world, display_object).rot.z
    };
    world.obj_mut::<PiranhaLogic>(this).target_z_rot = target_z_rot;

    ItemResult::Done
}

/// `-[PiranhaLogic swim:::]` @ 0x100009bf8
///
/// Follows the drawn path point by point (eating the pygmy at its end), or swims freely and
/// bounces off the edges; then turns toward the target heading.
pub fn seq_swim(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    let mut point_count = world.obj::<PiranhaLogic>(this).path_vector.len();
    if point_count >= 2 {
        let following_path = world.obj::<PiranhaLogic>(this).following_path;
        if following_path && has_reached_next_point(world, this, pos) {
            world.obj_mut::<PiranhaLogic>(this).path_vector.remove(1);
            point_count = world.obj::<PiranhaLogic>(this).path_vector.len();
            if point_count < 2 {
                let result = finish_path(world, this, pos);
                if let Some(result) = result {
                    return result;
                }
            }
        }
    }

    if point_count >= 2 {
        follow_path(world, this, pos);
    } else {
        let result = swim_freely(world, this, pos);
        if let Some(result) = result {
            return result;
        }
    }

    turn_toward_target(world, this, display_object)
}

/// Whether the piranha has reached point 1 of its path: on neither axis is it still moving
/// toward it.
fn has_reached_next_point(world: &World, this: Entity, pos: Vec4) -> bool {
    let next_point = world.obj::<PiranhaLogic>(this).path_vector[1];
    let delta_x = next_point.x - pos.x;
    let delta_y = next_point.y - pos.y;
    let pos_vel = logic::logic(world, this).pos_vel;

    let approaching_x = !((delta_x <= 0.0 && pos_vel.x >= 0.0) || (delta_x >= 0.0 && pos_vel.x <= 0.0));
    let approaching_y = !((delta_y <= 0.0 && pos_vel.y >= 0.0) || (delta_y >= 0.0 && pos_vel.y <= 0.0));
    !approaching_x && !approaching_y
}

/// The end of a followed path: bite the pygmy it leads to, or fight the piranha already eating
/// it. `Some` when `swim:::` returns at once.
fn finish_path(world: &mut World, this: Entity, pos: Vec4) -> Option<ItemResult> {
    let (path_model, listener) = {
        let mut piranha = world.obj_mut::<PiranhaLogic>(this);
        piranha.following_path = false;
        piranha.path_vector.clear();
        (piranha.path_model, piranha.piranha_logic_listener)
    };
    if let Some(path_model) = path_model {
        path_model::set_path_vector(world, path_model, Vec::new());
    }

    let pygmies = match listener {
        Some(listener) => anchor_logic::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    let path_pygmy = world.obj::<PiranhaLogic>(this).path_pygmy_logic;
    let pygmy_still_there = match path_pygmy {
        Some(path_pygmy) => pygmies.contains(&path_pygmy),
        None => false,
    };
    if !pygmy_still_there {
        let mut piranha = world.obj_mut::<PiranhaLogic>(this);
        piranha.path_leads_to_pygmy = false;
        piranha.path_pygmy_logic = None;
    }

    let path_leads_to_pygmy = world.obj::<PiranhaLogic>(this).path_leads_to_pygmy;
    let edible_pygmy = match path_pygmy {
        Some(pygmy) if path_leads_to_pygmy => {
            let edible = crate::pygmy::can_be_eaten_by_piranha(world, pygmy, this);
            if edible { Some(pygmy) } else { None }
        }
        _ => None,
    };

    if let Some(pygmy) = edible_pygmy {
        let eating_piranha = crate::pygmy::eating_piranha_logic(world, pygmy);
        if let Some(eating_piranha) = eating_piranha {
            {
                let mut piranha = world.obj_mut::<PiranhaLogic>(this);
                piranha.path_pygmy_logic = None;
                piranha.path_leads_to_pygmy = false;
            }
            fight_piranha(world, this, None);
            fight_piranha(world, eating_piranha, Some(this));
            if let Some(listener) = listener {
                anchor_logic::on_piranha_fight(world, listener, this);
            }
            return Some(ItemResult::Replaced);
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let mouth_y = pygmy_pos.y + PYGMY_MOUTH_Y_OFFSET;
        let offset = Vec4::new(pos.x - pygmy_pos.x, pos.y - mouth_y, pos.z - pygmy_pos.z, pos.w);
        if length3(offset) < 20.0 {
            {
                let mut piranha = world.obj_mut::<PiranhaLogic>(this);
                piranha.eating = true;
                piranha.agitated = false;
            }
            {
                let mut state = logic::logic_mut(world, this);
                state.pos_vel.x = 0.0;
                state.pos_vel.y = 0.0;
            }
            world.obj_mut::<PiranhaLogic>(this).eating_pygmy_logic = Some(pygmy);
            crate::pygmy::eaten_by_piranha(world, pygmy, this);
            logic::set_behavior(world, this, Some("PiranhaAttack"));
            return Some(ItemResult::Replaced);
        }
    }

    let mut piranha = world.obj_mut::<PiranhaLogic>(this);
    piranha.path_pygmy_logic = None;
    piranha.path_leads_to_pygmy = false;
    None
}

/// Heads for point 1 of the path at swimming speed; point 0 moves to the piranha.
fn follow_path(world: &mut World, this: Entity, pos: Vec4) {
    let next_point = world.obj::<PiranhaLogic>(this).path_vector[1];
    let delta_x = next_point.x - pos.x;
    let delta_y = next_point.y - pos.y;
    world.obj_mut::<PiranhaLogic>(this).following_path = true;

    let heading = delta_y.atan2(delta_x);
    let (sin, cos) = heading.sin_cos();
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = cos * SWIM_SPEED;
        state.pos_vel.y = sin * SWIM_SPEED;
    }

    let display_object = logic::display_object(world, this);
    let current_pos = display::object(world, display_object).pos;
    let (path_model, path) = {
        let mut piranha = world.obj_mut::<PiranhaLogic>(this);
        piranha.path_vector[0] = current_pos;
        (piranha.path_model, piranha.path_vector.clone())
    };
    if let Some(path_model) = path_model {
        path_model::set_path_vector(world, path_model, path);
    }

    world.obj_mut::<PiranhaLogic>(this).target_z_rot = rad_to_deg(heading);
}

/// Without a path: a full piranha that has left the screen is reported, a hungry one bounces
/// off the edges of the play area. `Some` when `swim:::` returns at once.
fn swim_freely(world: &mut World, this: Entity, pos: Vec4) -> Option<ItemResult> {
    let (finish_eating, listener) = {
        let piranha = world.obj::<PiranhaLogic>(this);
        (piranha.finish_eating, piranha.piranha_logic_listener)
    };

    if finish_eating {
        if pos.x < -20.0 || pos.x > 500.0 {
            if let Some(listener) = listener {
                anchor_logic::on_piranha_off_screen_ate_pygmy(world, listener, this);
            }
            return Some(ItemResult::Replaced);
        }
    } else {
        let pos_vel = logic::logic(world, this).pos_vel;
        let leaving_left = pos.x < 50.0 && pos_vel.x < 0.0;
        let leaving_right = pos.x > 430.0 && pos_vel.x > 0.0;
        if leaving_left || leaving_right {
            let bounced_vel_x = -pos_vel.x;
            logic::logic_mut(world, this).pos_vel.x = bounced_vel_x;

            let heading = pos_vel.y.atan2(bounced_vel_x);
            world.obj_mut::<PiranhaLogic>(this).target_z_rot = rad_to_deg(heading);
        }
    }

    let pos_vel = logic::logic(world, this).pos_vel;
    let leaving_bottom = pos.y < 50.0 && pos_vel.y < 0.0;
    let leaving_top = pos.y > 270.0 && pos_vel.y > 0.0;
    if leaving_bottom || leaving_top {
        let bounced_vel_y = -pos_vel.y;
        logic::logic_mut(world, this).pos_vel.y = bounced_vel_y;

        let heading = bounced_vel_y.atan2(pos_vel.x);
        world.obj_mut::<PiranhaLogic>(this).target_z_rot = rad_to_deg(heading);
    }
    None
}

/// The end of `swim:::`: turns 40 % of the way to `mTargetZRot` (the short way round), then
/// starts a flip when the heading needs the other `yScale`.
fn turn_toward_target(world: &mut World, this: Entity, display_object: Entity) -> ItemResult {
    let mut target_z_rot = world.obj::<PiranhaLogic>(this).target_z_rot;
    let mut z_rot = display::object(world, display_object).rot.z;

    if (z_rot - target_z_rot).abs() > 180.0 {
        if z_rot < 0.0 {
            z_rot = z_rot + 360.0;
            display::object_mut(world, display_object).rot.z = z_rot;
        } else if z_rot <= 180.0 {
            if target_z_rot < 0.0 {
                target_z_rot = target_z_rot + 360.0;
                world.obj_mut::<PiranhaLogic>(this).target_z_rot = target_z_rot;
            }
        } else {
            z_rot = z_rot - 360.0;
            display::object_mut(world, display_object).rot.z = z_rot;
        }
    }

    let turned_z_rot = z_rot + (target_z_rot - z_rot) * 0.4;
    display::object_mut(world, display_object).rot.z = turned_z_rot;

    let (flipping, finish_eating) = {
        let piranha = world.obj::<PiranhaLogic>(this);
        (piranha.flipping, piranha.finish_eating)
    };
    if flipping {
        return ItemResult::Pending;
    }

    let upright = upright_y_scale(target_z_rot);
    let y_scale = display::object(world, display_object).scale.y;
    if upright == y_scale {
        return ItemResult::Pending;
    }

    world.obj_mut::<PiranhaLogic>(this).flipping = true;
    let behavior = if finish_eating { "PiranhaFlipFull" } else { "PiranhaFlip" };
    logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[PiranhaLogic swimAway:::]` @ 0x10000a370
pub fn seq_swim_away(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        let pos_x = display::object(world, display_object).pos.x;
        logic::logic_mut(world, this).pos_vel.y = 0.0;

        let (vel_x, y_scale, z_rot) = if pos_x > 240.0 {
            (SWIM_AWAY_SPEED, 1.0, 0.0)
        } else {
            (-SWIM_AWAY_SPEED, -1.0, 180.0)
        };
        logic::logic_mut(world, this).pos_vel.x = vel_x;

        let mut object = display::object_mut(world, display_object);
        object.scale.y = y_scale;
        object.rot.z = z_rot;
    }

    let pos_x = display::object(world, display_object).pos.x;
    if pos_x < -20.0 || pos_x > 500.0 {
        logic::logic_mut(world, this).pos_vel.x = 0.0;

        let listener = world.obj::<PiranhaLogic>(this).piranha_logic_listener;
        if let Some(listener) = listener {
            anchor_logic::on_piranha_die(world, listener, this);
        }
        return ItemResult::Done;
    }
    ItemResult::Pending
}

/// `-[PiranhaLogic die:::]` @ 0x10000a4b8
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<PiranhaLogic>(this).piranha_logic_listener;
    if let Some(listener) = listener {
        anchor_logic::on_piranha_die(world, listener, this);
    }
    ItemResult::Done
}

/// `-[PiranhaLogic attack:::]` @ 0x10000a4f0
///
/// Chews for two seconds while turning level, then finishes the pygmy.
pub fn seq_attack(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        logic::logic_mut(world, this).stall_time = 2.0;

        let z_rot = display::object(world, display_object).rot.z;
        if z_rot >= -90.0 && z_rot <= 90.0 {
            world.obj_mut::<PiranhaLogic>(this).target_z_rot = 0.0;
        } else if z_rot > 90.0 && z_rot <= 270.0 {
            world.obj_mut::<PiranhaLogic>(this).target_z_rot = 180.0;
        } else if z_rot > 270.0 {
            display::object_mut(world, display_object).rot.z = z_rot - 360.0;
        } else if z_rot < -90.0 {
            world.obj_mut::<PiranhaLogic>(this).target_z_rot = -180.0;
        }
        cancel_flip(world, this);
    }

    let stall_time = logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        world.obj_mut::<PiranhaLogic>(this).finish_eating = true;

        let eating_pygmy = world.obj::<PiranhaLogic>(this).eating_pygmy_logic;
        if let Some(eating_pygmy) = eating_pygmy {
            crate::pygmy::eaten_by_piranha_complete(world, eating_pygmy, this);
        }
        world.obj_mut::<PiranhaLogic>(this).eating_pygmy_logic = None;
        return ItemResult::Done;
    }

    let target_z_rot = world.obj::<PiranhaLogic>(this).target_z_rot;
    {
        let mut object = display::object_mut(world, display_object);
        object.rot.z = object.rot.z + (target_z_rot - object.rot.z) * 0.4;
    }
    logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[PiranhaLogic eatPygmyComplete:::]` @ 0x10000a6ac
pub fn seq_eat_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PiranhaLogic>(this).eating = false;
    swim_along_heading(world, this);
    ItemResult::Done
}

/// `-[PiranhaLogic shutdown]` @ 0x10000a738
pub fn shutdown(world: &mut World, this: Entity) {
    let eating_pygmy = world.obj::<PiranhaLogic>(this).eating_pygmy_logic;
    if let Some(eating_pygmy) = eating_pygmy {
        crate::pygmy::stop_eaten_by_piranha(world, eating_pygmy, this);
        world.obj_mut::<PiranhaLogic>(this).eating_pygmy_logic = None;
    }

    // `[super shutdown]`: `SpriteLogic` inherits `BCLogic`'s.
    logic::shutdown_base(world, this);
}

/// The touch position in the coordinates of the piranha's display group
/// (`[[mDisplayObject displayGroup] viewToDisplayObject:[touch pos]]`, zero without a group).
fn touch_pos_in_display_group(world: &World, display_object: Entity, touch: Entity) -> Vec4 {
    let display_group = display::object(world, display_object).display_group;
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_pos),
        None => Vec4::ZERO,
    }
}

/// `-[PiranhaLogic touchBeganInside:allTouches:]` @ 0x10000a860
///
/// Starts a new path at the piranha; the path model is created on the first touch.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let busy = {
        let piranha = world.obj::<PiranhaLogic>(this);
        piranha.fighting || piranha.stop_bait_master || piranha.eating
    };
    if busy {
        return;
    }
    world.obj_mut::<PiranhaLogic>(this).active_touch = Some(touch);

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let local_touch_pos = touch_pos_in_display_group(world, display_object, touch);

    let (path, path_model) = {
        let mut piranha = world.obj_mut::<PiranhaLogic>(this);
        piranha.touch_began_inside = true;
        piranha.path_leads_to_pygmy = false;
        piranha.path_pygmy_logic = None;
        piranha.path_vector.clear();
        piranha.path_vector.push(pos);
        piranha.touch_x_offset = local_touch_pos.x - pos.x;
        piranha.touch_y_offset = local_touch_pos.y - pos.y;
        (piranha.path_vector.clone(), piranha.path_model)
    };
    if path_model.is_some() {
        return;
    }

    let path_model = world.spawn(Isa(&PATH_MODEL)).id();
    path_model::init_with_id(world, path_model, "PiranhaPath", path);
    world.obj_mut::<PiranhaLogic>(this).path_model = Some(path_model);
    display::object_mut(world, path_model).pos.z = SPRITE_Z;

    let marker = pg_engine::library::display_marker(world, "UnderwaterPathDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(path_model));
    }
}

/// `-[PiranhaLogic touchBeganOutside:allTouches:]` @ 0x10000aab8
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[PiranhaLogic touchMovedEntered:allTouches:]` @ 0x10000aabc
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[PiranhaLogic touchMovedExited:allTouches:]` @ 0x10000aac0
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[PiranhaLogic touchMovedInside:allTouches:]` @ 0x10000aac4
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[PiranhaLogic touchMovedOutside:allTouches:]` @ 0x10000aad4
///
/// Extends the path under the finger: a point near a pygmy that can be eaten ends the path at
/// its mouth; otherwise a point within 20 of the last one replaces it.
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let (active_touch, touch_began_inside, point_count) = {
        let piranha = world.obj::<PiranhaLogic>(this);
        (piranha.active_touch, piranha.touch_began_inside, piranha.path_vector.len())
    };
    let touch_is_ours = active_touch.is_none() || active_touch == Some(touch);
    if !touch_is_ours || !touch_began_inside {
        return;
    }
    if point_count >= MAX_PATH_POINTS {
        world.obj_mut::<PiranhaLogic>(this).touch_began_inside = false;
        return;
    }

    let display_object = logic::display_object(world, this);
    let local_touch_pos = touch_pos_in_display_group(world, display_object, touch);

    let (finish_eating, listener) = {
        let piranha = world.obj::<PiranhaLogic>(this);
        (piranha.finish_eating, piranha.piranha_logic_listener)
    };
    if !finish_eating {
        let pygmies = match listener {
            Some(listener) => anchor_logic::pygmy_logic_array(world, listener, this),
            None => Vec::new(),
        };
        for pygmy in pygmies {
            let pygmy_display_object = logic::display_object(world, pygmy);
            let mut mouth = display::object(world, pygmy_display_object).pos;
            mouth.y = mouth.y + PYGMY_MOUTH_Y_OFFSET;

            let offset_x = mouth.x - local_touch_pos.x;
            let offset_y = mouth.y - local_touch_pos.y;
            let distance = (offset_x * offset_x + offset_y * offset_y).sqrt();
            if distance < 40.0 && crate::pygmy::can_be_eaten_by_piranha(world, pygmy, this) {
                let mut piranha = world.obj_mut::<PiranhaLogic>(this);
                piranha.touch_began_inside = false;
                piranha.path_leads_to_pygmy = true;
                piranha.path_pygmy_logic = Some(pygmy);
                piranha.path_vector.push(mouth);
                return;
            }
        }
    }

    let (path_model, path) = {
        let mut piranha = world.obj_mut::<PiranhaLogic>(this);
        let point_x = local_touch_pos.x - piranha.touch_x_offset;
        let point_y = local_touch_pos.y - piranha.touch_y_offset;
        let point = Vec4::new(point_x, point_y, local_touch_pos.z, local_touch_pos.w);

        let last = piranha.last_path_pos;
        let offset = Vec4::new(point.x - last.x, point.y - last.y, point.z - last.z, point.w);
        if length3(offset) > 20.0 {
            piranha.path_vector.push(point);
            piranha.last_path_pos = point;
        } else {
            // PORT: when the path is empty (the piranha finished it while the finger was still
            // down) the original writes the point just before the vector's storage; the port
            // leaves the path empty.
            if let Some(last_point) = piranha.path_vector.last_mut() {
                *last_point = point;
            }
        }
        (piranha.path_model, piranha.path_vector.clone())
    };

    if let Some(path_model) = path_model {
        path_model::set_path_vector(world, path_model, path);
    }
}

/// `-[PiranhaLogic touchEndedInside:allTouches:]` @ 0x10000aeb8
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let mut piranha = world.obj_mut::<PiranhaLogic>(this);
    let touch_is_ours = piranha.active_touch.is_none() || piranha.active_touch == Some(touch);
    if touch_is_ours {
        piranha.touch_began_inside = false;
    }
}

/// `-[PiranhaLogic touchEndedOutside:allTouches:]` @ 0x10000aee0
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
