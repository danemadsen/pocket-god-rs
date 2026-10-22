//! `ApocalypseCloudLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::SpriteLogic;
use pg_engine::display;
use pg_engine::logic;

/// `ApocalypseCloudLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApocalypseCloudLogic {
    /// `mApocalypseCloudLogicListener`: `@"<ApocalypseCloudLogicListener>"`
    pub apocalypse_cloud_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mSpriteLogic`: `@"SpriteLogic"`
    pub sprite_logic: Option<Entity>,

    /// `mDragXOffset`: `f`
    pub drag_x_offset: f32,

    /// `mDragYOffset`: `f`
    pub drag_y_offset: f32,
}

/// File-scope globals of `ApocalypseCloudLogic`.
#[derive(Resource, Default, Clone, Debug)]
pub struct ApocalypseCloudLogicStatics {
    /// `DAT_1007d1f00`: every active cloud, in creation order (created on the first cloud).
    pub cloud_logic_array: Vec<Entity>,
}

pub const APOCALYPSE_CLOUD_LOGIC: Class = Class {
    name: "ApocalypseCloudLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Sequence-item handlers implemented by `ApocalypseCloudLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        "setAnimationBasedOnType" => seq_set_animation_based_on_type(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[NSString stringWithFormat:format, mApocalypseCloudStyleId]` for the
/// `ApocalypseCloud%@Activate` / `ApocalypseCloud%@Deactivate` behaviours (`%@` of nil prints
/// `(null)`).
fn cloud_style_behavior(world: &World, this: Entity, suffix: &str) -> String {
    let style_id = world.obj::<SpriteLogic>(this).apocalypse_cloud_style_id.clone();
    let style_id = style_id.unwrap_or_else(|| String::from("(null)"));
    format!("ApocalypseCloud{style_id}{suffix}")
}

/// `-[ApocalypseCloudLogic initWithDisplayObject:spriteLogic:apocalypseDiseaseType:apocalypseCloudStyleId:]` @ 0x1000c97ec
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity, sprite_logic: Entity, apocalypse_disease_type: i32, apocalypse_cloud_style_id: &str) {
    world.entity_mut(this).insert(ApocalypseCloudLogic::default());
    crate::logic::island_core::sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<ApocalypseCloudLogic>(this).sprite_logic = Some(sprite_logic);
    world.obj_mut::<SpriteLogic>(this).apocalypse_disease_type = apocalypse_disease_type;

    let mut statics = world.get_resource_or_insert_with(ApocalypseCloudLogicStatics::default);
    statics.cloud_logic_array.push(this);

    let style_id = apocalypse_cloud_style_id.to_owned();
    world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_style_id = Some(style_id);

    let own_display_object = logic::display_object(world, this);
    let sprite_display_object = logic::display_object(world, sprite_logic);
    let sprite_pos = display::object(world, sprite_display_object).pos;
    display::object_mut(world, own_display_object).pos = sprite_pos;

    world.obj_mut::<ApocalypseCloudLogic>(this).active = true;
    display::insert_after(world, sprite_display_object, Some(own_display_object));

    let behavior = cloud_style_behavior(world, this, "Activate");
    logic::set_behavior(world, this, Some(&behavior));
}

/// `-[ApocalypseCloudLogic setDragXOffset:dragYOffset:]` @ 0x1000c99b0
pub fn set_drag_x_offset(world: &mut World, this: Entity, drag_x_offset: f32, drag_y_offset: f32) {
    let mut cloud = world.obj_mut::<ApocalypseCloudLogic>(this);
    cloud.drag_x_offset = drag_x_offset;
    cloud.drag_y_offset = drag_y_offset;
}

/// `-[ApocalypseCloudLogic setApocalypseDiseaseType:]` @ 0x1000c99cc
pub fn set_apocalypse_disease_type(world: &mut World, this: Entity, apocalypse_disease_type: i32) {
    world.obj_mut::<SpriteLogic>(this).apocalypse_disease_type = apocalypse_disease_type;

    let behavior = cloud_style_behavior(world, this, "Activate");
    logic::set_behavior(world, this, Some(&behavior));
}

/// `-[ApocalypseCloudLogic setApocalypseCloudLogicListener:]` @ 0x1000c9a48
pub fn set_apocalypse_cloud_logic_listener(world: &mut World, this: Entity, apocalypse_cloud_logic_listener: Entity) {
    world.obj_mut::<ApocalypseCloudLogic>(this).apocalypse_cloud_logic_listener = Some(apocalypse_cloud_logic_listener);
}

/// `-[ApocalypseCloudLogic infectedSpriteLogic]` @ 0x1000c9b24
pub fn infected_sprite_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ApocalypseCloudLogic>(this).sprite_logic
}

/// `-[ApocalypseCloudLogic onGameFrame:]` @ 0x1000c9b34
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let infected_sprite = world.obj::<ApocalypseCloudLogic>(this).sprite_logic;
    let mut followed_sprite = infected_sprite;

    // A carried plague body takes its cloud along with the carrier.
    if is_kind_of(world, infected_sprite, "PygmyLogic") {
        if let Some(pygmy) = infected_sprite {
            let plague_body_picked_up = crate::pygmy::is_plague_body_picked_up(world, pygmy);
            if plague_body_picked_up {
                followed_sprite = crate::pygmy::plague_body_carrying_pygmy_logic(world, pygmy);
            }
        }
    }

    let Some(followed_sprite) = followed_sprite else {
        return;
    };

    let followed_display_object = logic::display_object(world, followed_sprite);
    let followed_pos = display::object(world, followed_display_object).pos;

    let cloud = world.obj::<ApocalypseCloudLogic>(this);
    let drag_x_offset = cloud.drag_x_offset;
    let drag_y_offset = cloud.drag_y_offset;

    let own_display_object = logic::display_object(world, this);
    let mut display = display::object_mut(world, own_display_object);
    display.pos = followed_pos;
    display.pos.x = drag_x_offset + display.pos.x;
    display.pos.y = drag_y_offset + display.pos.y;
}

/// `-[ApocalypseCloudLogic die:::]` @ 0x1000c9c60
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    logic::shutdown(world, this);

    // `[self release]`: `deactivate` took the cloud out of the global array and the cured
    // sprite dropped its reference right after it, so this was the last one.
    logic::dealloc(world, this);
    ItemResult::Done
}

/// `-[ApocalypseCloudLogic deactivate]` @ 0x1000c9cc8
pub fn deactivate(world: &mut World, this: Entity) {
    let active = world.obj::<ApocalypseCloudLogic>(this).active;
    if !active {
        return;
    }

    if let Some(mut statics) = world.get_resource_mut::<ApocalypseCloudLogicStatics>() {
        statics.cloud_logic_array.retain(|cloud| *cloud != this);
    }

    {
        let mut cloud = world.obj_mut::<ApocalypseCloudLogic>(this);
        cloud.sprite_logic = None;
        cloud.active = false;
    }

    if logic::is_hibernating(world, this) {
        let display_object = logic::display_object(world, this);
        display::remove_from_display_group(world, display_object);
        logic::shutdown(world, this);

        // `[self release]`: only the cured sprite's reference is left, and its `cureDisease`
        // (the only sender of `deactivate`) releases it as soon as this returns without
        // touching the cloud again, so the cloud is torn down here.
        logic::dealloc(world, this);
        return;
    }

    let behavior = cloud_style_behavior(world, this, "Deactivate");
    logic::set_behavior(world, this, Some(&behavior));
}

/// `-[ApocalypseCloudLogic setAnimationBasedOnType:::]` @ 0x1000c9df4
pub fn seq_set_animation_based_on_type(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let disease_type = world.obj::<SpriteLogic>(this).apocalypse_disease_type;
    let animation_id = match disease_type {
        2 => call.param("purification"),
        1 => call.param("plague"),
        _ => return ItemResult::Done,
    };

    if let Some(animation_id) = animation_id {
        let animation = logic::animation(world, this);
        pg_engine::animation::play_animation_id(world, animation, Some(animation_id));
    }
    ItemResult::Done
}

/// `+[ApocalypseCloudLogic doesDiseaseExist]` @ 0x1000c9560
pub fn class_does_disease_exist(world: &mut World) -> bool {
    match world.get_resource::<ApocalypseCloudLogicStatics>() {
        Some(statics) => !statics.cloud_logic_array.is_empty(),
        None => false,
    }
}

/// `+[ApocalypseCloudLogic apocalypseCloudWithSprite:apocalypseDiseaseType:apocalypseCloudStyleId:]` @ 0x1000c9590
pub fn class_apocalypse_cloud_with_sprite(world: &mut World, apocalypse_cloud_with_sprite: Entity, apocalypse_disease_type: i32, apocalypse_cloud_style_id: &str) -> Option<Entity> {
    let multi_model = pg_engine::display::multi_model::spawn(world, None);

    let cloud = world.spawn(Isa(&APOCALYPSE_CLOUD_LOGIC)).id();
    init_with_display_object(world, cloud, multi_model, apocalypse_cloud_with_sprite, apocalypse_disease_type, apocalypse_cloud_style_id);
    Some(cloud)
}

/// `+[ApocalypseCloudLogic cureAllDisease]` @ 0x1000c9644
pub fn class_cure_all_disease(world: &mut World) {
    let disease_exists = class_does_disease_exist(world);
    if disease_exists {
        let idol_manager = crate::managers::idol_manager::instance(world);
        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::trigger_episode_event(world, idol_manager, "WashAwayAllDiseases");
        }
    }

    // `[[NSArray alloc] initWithArray:DAT_1007d1f00]`: curing removes clouds from the original.
    let clouds = match world.get_resource::<ApocalypseCloudLogicStatics>() {
        Some(statics) => statics.cloud_logic_array.clone(),
        None => Vec::new(),
    };

    for cloud in clouds {
        let infected_sprite = infected_sprite_logic(world, cloud);
        if let Some(infected_sprite) = infected_sprite {
            crate::logic::island_core::sprite_logic::cure_disease(world, infected_sprite);
        }
    }
}
