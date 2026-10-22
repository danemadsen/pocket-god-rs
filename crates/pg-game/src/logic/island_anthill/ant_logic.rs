//! `AntLogic` (superclass `SpriteLogic`).
//!
//! One ant of the column an `AntHillLogic` sends out: it lines up between the hill and the
//! finger, then looks for pygmies; a swarming ant marches onto its pygmy, and the lead ant
//! leaves a bone there and walks home.

use crate::prelude::*;

use crate::logic::island_anthill::ant_hill_logic;
use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `AntLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct AntLogic {
    /// `mAntLogicListener`: `@"<AntLogicListener>"`
    pub ant_logic_listener: Option<Entity>,

    /// `mbLeadAntLogic`: `B`
    pub lead_ant_logic: bool,

    /// `mBoneDisplayObject`: `@"BCDisplayObject"`
    pub bone_display_object: Option<Entity>,

    /// `mbBurning`: `B`
    pub burning: bool,

    /// `mbSwarming`: `B`
    pub swarming: bool,

    /// `mbMovingAway`: `B`
    pub moving_away: bool,

    /// `mSwarmXPos`: `f`
    pub swarm_x_pos: f32,

    /// `mWalkVel`: `f`
    pub walk_vel: f32,

    /// `mStartXPos`: `f`
    pub start_x_pos: f32,

    /// `mEndXPos`: `f`
    pub end_x_pos: f32,
}

pub const ANT_LOGIC: Class = Class {
    name: "AntLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The speed of an ant marching onto a swarmed pygmy or carrying a bone home.
const MARCH_VEL: f32 = 60.0;

/// How far from the hill a bone carrier stops.
const HOME_RANGE: f32 = 25.0;

/// Sequence-item handlers implemented by `AntLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "carryBone" => seq_carry_bone(world, this, call),
        "swarm" => seq_swarm(world, this, call),
        "lookForPygmy" => seq_look_for_pygmy(world, this, call),
        "moveToTarget" => seq_move_to_target(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[AntLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&ANT_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[AntLogic initWithDisplayObject:]` @ 0x1001a6a9c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(AntLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    // A walking speed between 40 and 60.
    let speed_offset = world.resource_mut::<Random>().arc4random_mod(201);
    let speed_tenths = (speed_offset + 400) as i32;
    world.obj_mut::<AntLogic>(this).walk_vel = speed_tenths as f32 / 10.0;

    logic::set_behavior(world, this, Some("AntLineUp"));
}

/// `-[AntLogic setAntLogicListener:]` @ 0x1001a6b44
pub fn set_ant_logic_listener(world: &mut World, this: Entity, ant_logic_listener: Entity) {
    world.obj_mut::<AntLogic>(this).ant_logic_listener = Some(ant_logic_listener);
}

/// `-[AntLogic dealloc]` @ 0x1001a6ba0
///
/// For the ant hill's `onAntDie:`, which drops the ant's last owner. `BCLogic` then releases
/// the `Ant` multi-model, whose only other owner (the display tree) the hill already removed.
pub fn dealloc(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);

    // `[mAntLogicListener release]` needs no port; `[super dealloc]`.
    logic::dealloc(world, this);

    let out_of_display_tree = match world.get::<DisplayObject>(display_object) {
        Some(object) => object.display_group.is_none(),
        None => false,
    };
    if out_of_display_tree {
        world.despawn(display_object);
    }
}

/// `-[AntLogic die]` @ 0x1001a6c04
pub fn die(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("AntDie"));
}

/// `-[AntLogic setStartXPos:endXPos:]` @ 0x1001a6c1c
pub fn set_start_x_pos(world: &mut World, this: Entity, start_x_pos: f32, end_x_pos: f32) {
    let mut ant = world.obj_mut::<AntLogic>(this);
    ant.start_x_pos = start_x_pos;
    ant.end_x_pos = end_x_pos;
}

/// `-[AntLogic canBeBurntByMagnifyingGlass:]` @ 0x1001a6c38
///
/// Every ant but the one leading a swarm.
pub fn can_be_burnt_by_magnifying_glass(world: &mut World, this: Entity, _magnifying_glass_logic: Entity) -> bool {
    !world.obj::<AntLogic>(this).lead_ant_logic
}

/// `-[AntLogic burntByMagnifyingGlass:]` @ 0x1001a6c50
pub fn burnt_by_magnifying_glass(world: &mut World, this: Entity, _magnifying_glass_logic: Entity) {
    if world.obj::<AntLogic>(this).burning {
        return;
    }

    world.obj_mut::<AntLogic>(this).burning = true;
    logic::logic_mut(world, this).pos_vel.x = 0.0;
    logic::set_behavior(world, this, Some("AntBurn"));
}

/// `-[AntLogic setLeadAntLogic]` @ 0x1001a6c94
pub fn set_lead_ant_logic(world: &mut World, this: Entity) {
    world.obj_mut::<AntLogic>(this).lead_ant_logic = true;
}

/// `-[AntLogic swarmPygmy:]` @ 0x1001a6ca8
pub fn swarm_pygmy(world: &mut World, this: Entity, pygmy_logic: Entity) {
    world.obj_mut::<AntLogic>(this).swarming = true;

    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
    world.obj_mut::<AntLogic>(this).swarm_x_pos = pygmy_x_pos;

    logic::set_behavior(world, this, Some("AntSwarm"));
}

/// `-[AntLogic lookForPygmy]` @ 0x1001a6d1c
pub fn look_for_pygmy(world: &mut World, this: Entity) {
    if world.obj::<AntLogic>(this).swarming {
        return;
    }
    logic::set_behavior(world, this, Some("AntLookForPygmy"));
}

/// `-[AntLogic onGameFrame:]` @ 0x1001a6d48
///
/// Moves the ant with its velocity: the force is always zero.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let vel = {
        let mut state = logic::logic_mut(world, this);
        let force = Vec4::new(0.0, 0.0, 0.0, 1.0);
        state.pos_force = force;
        state.pos_vel.x = state.pos_vel.x + force.x * dt;
        state.pos_vel.y = state.pos_vel.y + force.y * dt;
        state.pos_vel.z = state.pos_vel.z + force.z * dt;
        state.pos_vel
    };

    let mut object = display::object_mut(world, display_object);
    object.pos.x = object.pos.x + vel.x * dt;
    object.pos.y = object.pos.y + vel.y * dt;
    object.pos.z = object.pos.z + vel.z * dt;
}

/// `-[AntLogic carryBone:::]` @ 0x1001a6e40
///
/// Walks back towards the hill and stops at the near edge of the range around it.
pub fn seq_carry_bone(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        world.obj_mut::<AntLogic>(this).moving_away = true;

        let start_x_pos = world.obj::<AntLogic>(this).start_x_pos;
        let pos_x = display::object(world, display_object).pos.x;
        let (x_scale, x_vel) = if pos_x < start_x_pos { (1.0, MARCH_VEL) } else { (-1.0, -MARCH_VEL) };
        display::object_mut(world, display_object).scale.x = x_scale;
        logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let start_x_pos = world.obj::<AntLogic>(this).start_x_pos;
    let pos_x = display::object(world, display_object).pos.x;
    let upper_x_pos = start_x_pos + HOME_RANGE;
    let lower_x_pos = start_x_pos - HOME_RANGE;
    let home = pos_x < upper_x_pos && lower_x_pos < pos_x;
    if !home {
        return ItemResult::Pending;
    }

    let x_scale = display::object(world, display_object).scale.x;
    let stop_x_pos = if x_scale > 0.0 { lower_x_pos } else { upper_x_pos };
    display::object_mut(world, display_object).pos.x = stop_x_pos;
    logic::logic_mut(world, this).pos_vel.x = 0.0;
    ItemResult::Done
}

/// `-[AntLogic swarm:::]` @ 0x1001a6f60
///
/// Marches onto the swarmed pygmy. There the lead ant leaves a bone in front of itself and
/// carries one home (`AntCarryBone`); the others are done.
pub fn seq_swarm(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        let swarm_x_pos = world.obj::<AntLogic>(this).swarm_x_pos;
        let pos_x = display::object(world, display_object).pos.x;
        let (x_scale, x_vel) = if pos_x < swarm_x_pos { (1.0, MARCH_VEL) } else { (-1.0, -MARCH_VEL) };
        display::object_mut(world, display_object).scale.x = x_scale;
        logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let vel_x = logic::logic(world, this).pos_vel.x;
    let swarm_x_pos = world.obj::<AntLogic>(this).swarm_x_pos;
    let pos_x = display::object(world, display_object).pos.x;
    let passed_going_right = 0.0 < vel_x && swarm_x_pos < pos_x;
    let passed_going_left = vel_x < 0.0 && pos_x < swarm_x_pos;
    if !passed_going_right && !passed_going_left {
        return ItemResult::Pending;
    }

    if !world.obj::<AntLogic>(this).lead_ant_logic {
        return ItemResult::Done;
    }

    logic::logic_mut(world, this).pos_vel.x = 0.0;

    let display_group = display::object(world, display_object).display_group;
    let ant_index = match display_group {
        Some(display_group) => display::group::index_of_display_object(world, display_group, display_object),
        None => 0,
    };
    let bone = display::model::spawn(world, Some("Bone"), None);
    world.obj_mut::<AntLogic>(this).bone_display_object = Some(bone);
    display::move_to_display_group_at_index(world, bone, display_group, ant_index);

    // `[mBoneDisplayObject release]`: the display group owns the bone from here on.
    let pos = display::object(world, display_object).pos;
    {
        let mut bone_object = display::object_mut(world, bone);
        bone_object.pos.x = pos.x;
        bone_object.pos.y = pos.y;
        bone_object.pos.z = pos.z;
    }

    logic::set_behavior(world, this, Some("AntCarryBone"));
    ItemResult::Replaced
}

/// `-[AntLogic lookForPygmy:::]` @ 0x1001a7198
///
/// Walks out to the end of the column, turns round and walks back past the hill.
pub fn seq_look_for_pygmy(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        world.obj_mut::<AntLogic>(this).moving_away = true;

        let (end_x_pos, walk_vel) = {
            let ant = world.obj::<AntLogic>(this);
            (ant.end_x_pos, ant.walk_vel)
        };
        let pos_x = display::object(world, display_object).pos.x;
        let (x_scale, x_vel) = if pos_x < end_x_pos { (1.0, walk_vel) } else { (-1.0, -walk_vel) };
        display::object_mut(world, display_object).scale.x = x_scale;
        logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let (moving_away, start_x_pos, end_x_pos) = {
        let ant = world.obj::<AntLogic>(this);
        (ant.moving_away, ant.start_x_pos, ant.end_x_pos)
    };
    let vel_x = logic::logic(world, this).pos_vel.x;
    let pos_x = display::object(world, display_object).pos.x;

    if !moving_away {
        let home_going_right = 0.0 < vel_x && start_x_pos < pos_x;
        let home_going_left = vel_x < 0.0 && pos_x < start_x_pos;
        if home_going_right || home_going_left {
            return ItemResult::Done;
        }
        return ItemResult::Pending;
    }

    let out_going_right = 0.0 < vel_x && end_x_pos < pos_x;
    let out_going_left = vel_x < 0.0 && pos_x < end_x_pos;
    if out_going_right || out_going_left {
        world.obj_mut::<AntLogic>(this).moving_away = false;
        {
            let mut object = display::object_mut(world, display_object);
            object.scale.x = -object.scale.x;
        }
        logic::logic_mut(world, this).pos_vel.x = -vel_x;
    }
    ItemResult::Pending
}

/// `-[AntLogic moveToTarget:::]` @ 0x1001a732c
///
/// Keeps walking towards the end of the column.
pub fn seq_move_to_target(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<AntLogic>(this).moving_away = true;
    }

    let display_object = logic::display_object(world, this);
    let (end_x_pos, walk_vel) = {
        let ant = world.obj::<AntLogic>(this);
        (ant.end_x_pos, ant.walk_vel)
    };
    let pos_x = display::object(world, display_object).pos.x;
    let (x_scale, x_vel) = if pos_x < end_x_pos { (1.0, walk_vel) } else { (-1.0, -walk_vel) };
    display::object_mut(world, display_object).scale.x = x_scale;
    logic::logic_mut(world, this).pos_vel.x = x_vel;
    ItemResult::Pending
}

/// `-[AntLogic die:::]` @ 0x1001a73fc
///
/// Takes the bone away and reports to the hill, which frees the ant.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let bone = world.obj::<AntLogic>(this).bone_display_object;
    if let Some(bone) = bone {
        display::remove_from_display_group(world, bone);

        // The display group held the bone's last reference.
        world.despawn(bone);
    }
    world.obj_mut::<AntLogic>(this).bone_display_object = None;

    let listener = world.obj::<AntLogic>(this).ant_logic_listener;
    if let Some(ant_hill) = listener {
        ant_hill_logic::on_ant_die(world, ant_hill, this);
    }
    ItemResult::Done
}
