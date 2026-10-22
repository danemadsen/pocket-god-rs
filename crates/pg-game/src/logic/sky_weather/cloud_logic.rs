//! `CloudLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;
use pg_engine::texture::Textures;

/// `CloudLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct CloudLogic {
    /// `mCloudLogicListener`: `@"<CloudLogicListener>"`
    pub cloud_logic_listener: Option<Entity>,

    /// `mCloudIndex`: `i`
    pub cloud_index: i32,

    /// `mXVelScale`: `f`
    pub x_vel_scale: f32,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbTransitionOut`: `B`
    pub transition_out: bool,

    /// `mbTransitionIn`: `B`
    pub transition_in: bool,
}

pub const CLOUD_LOGIC: Class = Class {
    name: "CloudLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The width of the 3:2 game area the clouds wrap around.
const SCREEN_WIDTH: f32 = 480.0;

/// Sequence-item handlers implemented by `CloudLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "drift" => seq_drift(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[CloudLogic alloc] initWithDisplayObject:cloudIndex:]`.
pub fn spawn(world: &mut World, display_object: Entity, cloud_index: i32) -> Entity {
    let this = world.spawn(Isa(&CLOUD_LOGIC)).id();
    init_with_display_object(world, this, display_object, cloud_index);
    this
}

/// `-[CloudLogic initWithDisplayObject:cloudIndex:]` @ 0x1000d4b6c
///
/// Index 0 is a drifting sky cloud; indices 1 to 3 are the Ooga Jump sky clouds, spaced a
/// third of a wrap apart.
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity, cloud_index: i32) {
    world.entity_mut(this).insert(CloudLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut cloud = world.obj_mut::<CloudLogic>(this);
        cloud.dragging = false;
        cloud.transition_out = false;
        cloud.cloud_index = cloud_index;
    }
    randomize_position(world, this);

    // Drawn for every cloud, used only by the sky cloud.
    let random = world.resource_mut::<Random>().arc4random();
    let cloud_index = world.obj::<CloudLogic>(this).cloud_index;
    let x_pos = if cloud_index == 0 {
        let offset = (random % 7801) as i32 - 3000;
        offset as f32 / 10.0
    } else {
        let col_x_max = display::object(world, display_object).col_x_max;
        let slot = cloud_index.wrapping_sub(1) as f32;
        (slot * (col_x_max + SCREEN_WIDTH)) / 3.0
    };
    display::object_mut(world, display_object).pos.x = x_pos;

    let x_vel_scale = world.obj::<CloudLogic>(this).x_vel_scale;
    logic::logic_mut(world, this).pos_vel.x = x_vel_scale * 10.0;
    logic::set_behavior(world, this, Some("CloudDrift"));
}

/// `-[CloudLogic randomizePosition]` @ 0x1000d4cec
///
/// A new speed, texture, height and size; the collision box covers the texture.
pub fn randomize_position(world: &mut World, this: Entity) {
    let speed_random = world.resource_mut::<Random>().arc4random();
    let x_vel_scale = (speed_random % 11 + 5) as f32 / 10.0;
    world.obj_mut::<CloudLogic>(this).x_vel_scale = x_vel_scale;

    let cloud_index = world.obj::<CloudLogic>(this).cloud_index;
    let (texture, scale, y_pos) = match cloud_index {
        0 => {
            let scale_random = world.resource_mut::<Random>().arc4random();
            let scale = (scale_random % 51 + 50) as f32 / 100.0;

            let texture_random = world.resource_mut::<Random>().arc4random();
            let texture_id = if texture_random & 1 == 0 { "Cloud1Texture" } else { "Cloud2Texture" };
            let texture = pg_engine::library::texture_def_id(world, texture_id);
            (texture, scale, 190.0)
        }
        1..=3 => {
            world.obj_mut::<CloudLogic>(this).x_vel_scale = 1.0;
            let texture = pg_engine::library::texture_def_id(world, "DoodleCloud");
            (texture, 1.0, 350.0)
        }
        _ => return,
    };
    let Some(texture) = texture else {
        return;
    };

    let (frame_width, frame_height) = {
        let def = world.resource::<Textures>().def(texture);
        (def.frame_width, def.frame_height)
    };

    let display_object = logic::display_object(world, this);
    display::set_texture_def(world, display_object, Some(texture));

    let mut display = display::object_mut(world, display_object);
    display.pos.y = y_pos;
    display.pos.z = SPRITE_Z;
    display.scale.x = scale;
    display.scale.y = scale;
    display.col_x_min = 0.0;
    display.col_x_max = frame_width;
    display.col_y_min = 0.0;
    display.col_y_max = frame_height;
}

/// `-[CloudLogic setCloudLogicListener:]` @ 0x1000d4f84
pub fn set_cloud_logic_listener(world: &mut World, this: Entity, cloud_logic_listener: Entity) {
    world.obj_mut::<CloudLogic>(this).cloud_logic_listener = Some(cloud_logic_listener);
}

/// `-[CloudLogic onGameFrame:]` @ 0x1000d5044
///
/// Unless dragged or leaving, a speed beyond `xVelScale · 20` either way is pulled back by 1 %
/// per frame (in double).
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let cloud = world.obj::<CloudLogic>(this);
    if !cloud.dragging && !cloud.transition_out {
        let max_speed = cloud.x_vel_scale * 20.0;
        let mut state = logic::logic_mut(world, this);

        let mut vel_x = state.pos_vel.x;
        if max_speed < vel_x {
            let step = f64::from(max_speed - vel_x) * 0.01;
            vel_x = (f64::from(vel_x) + step) as f32;
            state.pos_vel.x = vel_x;
        }

        let min_speed = -max_speed;
        if vel_x < min_speed {
            let step = f64::from(min_speed - vel_x) * 0.01;
            state.pos_vel.x = (f64::from(vel_x) + step) as f32;
        }
    }

    sprite_logic::on_game_frame(world, this, dt);
}

/// `-[CloudLogic isDoodleCloud]` @ 0x1000d5148
pub fn is_doodle_cloud(world: &mut World, this: Entity) -> bool {
    world.obj::<CloudLogic>(this).cloud_index != 0
}

/// `-[CloudLogic startDragging]` @ 0x1000d5160
pub fn start_dragging(world: &mut World, this: Entity) {
    logic::logic_mut(world, this).pos_vel.x = 0.0;
    world.obj_mut::<CloudLogic>(this).dragging = true;
}

/// `-[CloudLogic stopDragging:]` @ 0x1000d5184
pub fn stop_dragging(world: &mut World, this: Entity, dragging: f32) {
    let x_vel_scale = {
        let mut cloud = world.obj_mut::<CloudLogic>(this);
        cloud.dragging = false;
        cloud.x_vel_scale
    };
    logic::logic_mut(world, this).pos_vel.x = x_vel_scale * dragging;
}

/// `-[CloudLogic transitionOut:]` @ 0x1000d51b4
pub fn transition_out(world: &mut World, this: Entity, transition_out: f32) {
    let x_vel_scale = {
        let mut cloud = world.obj_mut::<CloudLogic>(this);
        cloud.transition_out = true;
        cloud.x_vel_scale
    };
    logic::logic_mut(world, this).pos_vel.x = x_vel_scale * transition_out;
}

/// `-[CloudLogic transitionIn:]` @ 0x1000d51e8
///
/// Starts the cloud two screens away on the side it drifts in from.
pub fn transition_in(world: &mut World, this: Entity, transition_in: f32) {
    let x_vel_scale = {
        let mut cloud = world.obj_mut::<CloudLogic>(this);
        cloud.transition_in = true;
        cloud.x_vel_scale
    };

    let vel_x = x_vel_scale * transition_in;
    logic::logic_mut(world, this).pos_vel.x = vel_x;

    let offset = if vel_x > 0.0 { -960.0 } else { 960.0 };
    let display_object = logic::display_object(world, this);
    let mut display = display::object_mut(world, display_object);
    display.pos.x = display.pos.x + offset;
}

/// `-[CloudLogic drift:::]` @ 0x1000d5288
///
/// Wraps the cloud around the screen (re-randomised when it leaves on the right). While
/// transitioning in, waits until the cloud has crossed the whole screen; while transitioning
/// out, a cloud leaving the screen tells the listener instead. Never completes.
pub fn seq_drift(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let min_x = display::min_pos(world, display_object).x;
    let max_x = display::max_pos(world, display_object).x;

    let transition_in = world.obj::<CloudLogic>(this).transition_in;
    if transition_in {
        let vel_x = logic::logic(world, this).pos_vel.x;
        let crossed_right = min_x > SCREEN_WIDTH && vel_x > 0.0;
        let crossed_left = max_x < 0.0 && vel_x < 0.0;
        if !(crossed_right || crossed_left) {
            return ItemResult::Pending;
        }
        world.obj_mut::<CloudLogic>(this).transition_in = false;
    }

    let cloud = world.obj::<CloudLogic>(this);
    let transition_out = cloud.transition_out;
    let listener = cloud.cloud_logic_listener;

    if min_x > SCREEN_WIDTH {
        if transition_out {
            if let Some(listener) = listener {
                crate::controller::on_cloud_die(world, listener, this);
            }
        } else {
            randomize_position(world, this);
            let col_x_max = display::object(world, display_object).col_x_max;
            let mut display = display::object_mut(world, display_object);
            display.pos.x = display.pos.x - (col_x_max + SCREEN_WIDTH);
        }
    }

    // `maxPos` is the value computed above (`randomizePosition` does not recompute it).
    if max_x < 0.0 {
        let col_x_max = display::object(world, display_object).col_x_max;
        {
            let mut display = display::object_mut(world, display_object);
            display.pos.x = display.pos.x + (col_x_max + SCREEN_WIDTH);
        }
        if transition_out {
            if let Some(listener) = listener {
                crate::controller::on_cloud_die(world, listener, this);
            }
        }
    }

    ItemResult::Pending
}
