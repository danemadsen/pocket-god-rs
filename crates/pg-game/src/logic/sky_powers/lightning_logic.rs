//! `LightningLogic` (superclass `SpriteLogic`).

use std::f32::consts::PI;

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::sky_powers::lightning_model;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::texture::Textures;

/// `LightningLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct LightningLogic {
    /// `mLightningLogicListener`: `@"<LightningLogicListener>"`
    pub lightning_logic_listener: Option<Entity>,

    /// `mHighlightArray`: `@"NSMutableArray"`
    pub highlight_array: Vec<Entity>,
}

pub const LIGHTNING_LOGIC: Class = Class {
    name: "LightningLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Texture of the glows at both ends of the bolt.
const GLOW_TEXTURE_ID: &str = "LightningGlow_01";

/// Model id of the end glows.
const GLOW_MODEL_ID: &str = "LightningGlow";

/// Texture of the two branch sprites.
const BRANCH_TEXTURE_ID: &str = "LightningBranch_01";

/// Model id of the branch sprites.
const BRANCH_MODEL_ID: &str = "LightningHighlight";

/// The branch sprite's origin (its root sits on the bolt).
const BRANCH_ORIGIN_X: f32 = -13.0;
const BRANCH_ORIGIN_Y: f32 = -34.0;

/// Bolts with fewer points get no branches.
const MIN_BRANCH_POINT_COUNT: i32 = 7;

/// The behaviour every new bolt plays.
const STRIKE_BEHAVIOR: &str = "LightningStrike";

/// Sequence-item handlers implemented by `LightningLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "blinkLightning" => seq_blink_lightning(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[LightningLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&LIGHTNING_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// The `(frameWidth, frameHeight)` of a texture definition (`+0x28`, `+0x2c`).
fn texture_frame_size(world: &World, texture: Option<TextureDefId>) -> (f32, f32) {
    let Some(texture) = texture else {
        return (0.0, 0.0);
    };
    let def = world.resource::<Textures>().def(texture);
    (def.frame_width, def.frame_height)
}

/// `-[LightningLogic initWithDisplayObject:]` @ 0x1000c8d04
///
/// Over the bolt's points (with the segment angles `LightningModel` stored in `w`): a glow at
/// both ends and, on bolts of seven points or more, branch sprites at one random interior point
/// of each half, all in the bolt's display group.
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(LightningLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let display_object = logic::display_object(world, this);
    let points = lightning_model::path_list(world, display_object);
    let display_group = display::object(world, display_object).display_group;
    world.obj_mut::<LightningLogic>(this).highlight_array = Vec::with_capacity(5);

    let point_count = points.len() as i32;
    let mut first_branch_index = 0;
    let mut second_branch_index = 0;
    if point_count >= MIN_BRANCH_POINT_COUNT {
        let half_point_count = point_count / 2;

        let first_range = (half_point_count - 2) as u32;
        let first_offset = world.resource_mut::<Random>().arc4random_mod(first_range);
        first_branch_index = first_offset as i32 + 1;

        let second_range = (point_count - half_point_count - 3) as u32;
        let second_offset = world.resource_mut::<Random>().arc4random_mod(second_range);
        second_branch_index = second_offset as i32 + half_point_count + 1;
    }

    let last_index = point_count - 1;
    for (index, point) in points.iter().enumerate() {
        let index = index as i32;

        if index == 0 || index == last_index {
            let texture = library::texture_def_id(world, GLOW_TEXTURE_ID);
            let glow = display::model::spawn(world, Some(GLOW_MODEL_ID), texture);
            let (frame_width, frame_height) = texture_frame_size(world, texture);
            {
                let mut glow_display = display::object_mut(world, glow);
                glow_display.origin_pos.x = frame_width * -0.5;
                glow_display.origin_pos.y = frame_height * -0.5;
                glow_display.pos.x = point.x;
                glow_display.pos.y = point.y;
                glow_display.pos.z = SPRITE_Z;
            }
            display::move_to_display_group(world, glow, display_group);
            world.obj_mut::<LightningLogic>(this).highlight_array.push(glow);
        } else if first_branch_index == index || second_branch_index == index {
            let texture = library::texture_def_id(world, BRANCH_TEXTURE_ID);
            let branch = display::model::spawn(world, Some(BRANCH_MODEL_ID), texture);

            // The first branch points backwards.
            let mut z_rot = (point.w / PI) * 180.0;
            if second_branch_index != index {
                z_rot = z_rot + 180.0;
            }

            {
                let mut branch_display = display::object_mut(world, branch);
                branch_display.origin_pos.x = BRANCH_ORIGIN_X;
                branch_display.origin_pos.y = BRANCH_ORIGIN_Y;
                branch_display.pos.x = point.x;
                branch_display.pos.y = point.y;
                branch_display.pos.z = SPRITE_Z;
                branch_display.rot.z = z_rot;
            }
            display::move_to_display_group(world, branch, display_group);
            world.obj_mut::<LightningLogic>(this).highlight_array.push(branch);
        }
    }

    logic::set_behavior(world, this, Some(STRIKE_BEHAVIOR));
}

/// `-[LightningLogic setLightningLogicListener:]` @ 0x1000c9174
pub fn set_lightning_logic_listener(world: &mut World, this: Entity, lightning_logic_listener: Entity) {
    world.obj_mut::<LightningLogic>(this).lightning_logic_listener = Some(lightning_logic_listener);
}

/// `-[LightningLogic dealloc]` @ 0x1000c91d0
///
/// For whoever releases the bolt last (the view controller's `onLightningDie:`, right after it
/// took the bolt out of the display tree and shut it down). Releasing `mHighlightArray` frees
/// the glows and branches `die:::` took out of the tree, and `BCLogic`'s release of the
/// display object frees the `LightningModel` once it has left the tree too. Objects still in
/// a display group keep living there.
pub fn dealloc(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let highlights = {
        let mut lightning = world.obj_mut::<LightningLogic>(this);
        std::mem::take(&mut lightning.highlight_array)
    };

    for highlight in highlights {
        let in_display_tree = display::object(world, highlight).display_group.is_some();
        if !in_display_tree {
            world.despawn(highlight);
        }
    }

    // `[super dealloc]`: `SpriteLogic` only releases its cloud style string.
    logic::dealloc(world, this);

    let model_out_of_display_tree = match world.get::<DisplayObject>(display_object) {
        Some(model) => model.display_group.is_none(),
        None => false,
    };
    if model_out_of_display_tree {
        world.despawn(display_object);
    }
}

/// `-[LightningLogic blinkLightning:::]` @ 0x1000c924c
///
/// `[0] time`: flips the bolt and its glows between alpha 1 and 0 every frame for that long.
pub fn seq_blink_lightning(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let time = call.float("time");
        logic::logic_mut(world, this).stall_time = time;
    }

    let display_object = logic::display_object(world, this);
    let current_alpha = display::object(world, display_object).alpha;
    let alpha = if current_alpha != 1.0 { 1.0 } else { 0.0 };
    display::object_mut(world, display_object).set_alpha(alpha);

    let highlights = world.obj::<LightningLogic>(this).highlight_array.clone();
    for highlight in highlights {
        display::object_mut(world, highlight).set_alpha(alpha);
    }

    let stall_time = logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        return ItemResult::Done;
    }
    logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[LightningLogic die:::]` @ 0x1000c9424
///
/// Takes the glows and branches out of the display tree and tells the listener, which
/// releases the bolt.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let highlights = world.obj::<LightningLogic>(this).highlight_array.clone();
    for highlight in highlights {
        display::remove_from_display_group(world, highlight);
    }

    let listener = world.obj::<LightningLogic>(this).lightning_logic_listener;
    if let Some(listener) = listener {
        crate::controller::god_powers::on_lightning_die(world, listener, this);
    }
    ItemResult::Done
}
