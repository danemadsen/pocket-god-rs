//! The Objective-C object model of the original, on Bevy ECS.
//!
//! Every ObjC object (display objects, logic objects, sequences, animation players, touches,
//! managers, the view controller) is an entity. Each class in the object's hierarchy
//! contributes one component holding that class's own ivars, and the [`Isa`] component points
//! at the object's [`Class`]: a static table of the methods that the engine or the game
//! dispatches dynamically (`objc_msgSend` on a receiver whose class is not known statically).
//!
//! A subclass table is built with struct-update syntax from its superclass's constant and
//! overrides only what the ObjC subclass overrides. An override that calls `[super …]` calls
//! the superclass constant's function pointer explicitly.
//!
//! ```ignore
//! pub const PIRANHA_LOGIC: Class = Class {
//!     name: "PiranhaLogic",
//!     superclass: Some(&SPRITE_LOGIC),
//!     on_game_frame: piranha_logic::on_game_frame,
//!     sequence_item: piranha_logic::sequence_item,
//!     ..SPRITE_LOGIC
//! };
//! ```

use std::any::Any;

use bevy_ecs::prelude::*;

use crate::gl::GlContext;
use crate::sequence::{ItemResult, SequenceCall};
use crate::texture::TextureDefId;

pub type ObjFn = fn(&mut World, Entity);
pub type FrameFn = fn(&mut World, Entity, f32);
pub type TouchFn = fn(&mut World, Entity, Entity);
pub type ButtonFn = fn(&mut World, Entity, Entity, Entity);
pub type SequenceItemFn = fn(&mut World, Entity, &mut SequenceCall) -> Option<ItemResult>;
pub type PerformFn = fn(&mut World, Entity, &str) -> bool;
pub type DrawFn = fn(&mut World, Entity, &mut GlContext);
pub type DrawInMultiModelFn = fn(&mut World, Entity, &mut GlContext, Entity, [f32; 4]);

/// Dynamically dispatched methods of one ObjC class.
#[derive(Clone, Copy)]
pub struct Class {
    pub name: &'static str,
    pub superclass: Option<&'static Class>,

    /// Zero-argument selectors sent by name (`NSSelectorFromString`: animation frame actions,
    /// `performSelector:`). Returns `false` when the class does not respond, like
    /// `respondsToSelector:`.
    pub perform: PerformFn,

    /// Sequence-item handlers (`<method>:::`). Returns `None` when neither this class nor a
    /// superclass responds, in which case the sequence falls back to `BCLibrary`.
    pub sequence_item: SequenceItemFn,

    /// `-gameFrameOrder` (`BCGameFrameListener`).
    pub game_frame_order: i32,

    /// `-onGameFrame:`.
    pub on_game_frame: FrameFn,

    // ---- BCLogic ----
    pub touch_began_inside: TouchFn,
    pub touch_began_outside: TouchFn,
    pub touch_moved_entered: TouchFn,
    pub touch_moved_exited: TouchFn,
    pub touch_moved_inside: TouchFn,
    pub touch_moved_outside: TouchFn,
    pub touch_ended_inside: TouchFn,
    pub touch_ended_outside: TouchFn,
    pub shutdown: ObjFn,
    pub set_behavior: fn(&mut World, Entity, Option<&str>),
    pub is_sound_disabled: fn(&World, Entity) -> bool,
    pub apply_move_pos: fn(&mut World, Entity, f32, f32),

    /// `-id`: the object's name (`BCLogic` returns its display object's id).
    pub id: fn(&World, Entity) -> Option<String>,

    // ---- BCScreenLogic ----
    pub close_screen_now: ObjFn,

    // ---- listener protocols the engine calls back ----
    /// `BCButtonLogicListener -onButtonPress:touch:` (listener, button, touch).
    pub on_button_press: ButtonFn,

    /// `BCButtonLogicListener -onButtonRelease:touch:`.
    pub on_button_release: ButtonFn,

    /// `BCButtonLogicListener -onButtonReleaseOutside:touch:`.
    pub on_button_release_outside: ButtonFn,

    /// `MultiModelLogicListener -onMultiModelEvent:eventId:` (listener, logic, event id).
    pub on_multi_model_event: fn(&mut World, Entity, Entity, &str),

    /// `MessageLogicListener -onMessageComplete:` (listener, message logic).
    pub on_message_complete: fn(&mut World, Entity, Entity),

    // ---- BCDisplayObject ----
    /// `-drawOpenGL:`.
    pub draw: DrawFn,

    /// `-drawInMultiModelOpenGL:red:green:blue:alpha:` (this, parent model, rgba).
    pub draw_in_multi_model: DrawInMultiModelFn,
    pub texture_def: fn(&World, Entity) -> Option<TextureDefId>,
    pub set_texture_def: fn(&mut World, Entity, Option<TextureDefId>),
    pub enable_multi_model_mode: ObjFn,
    pub disable_multi_model_mode: ObjFn,
    pub hibernate: ObjFn,
    pub unhibernate: ObjFn,

    /// `-addSelectableLogicsToArray:`.
    pub add_selectable_logics: fn(&World, Entity, &mut Vec<Entity>),

    /// Game-defined virtual methods (for example `SpriteLogic`'s hurricane hooks); the game
    /// crate downcasts this to its own table type.
    pub ext: Option<&'static (dyn Any + Sync)>,
}

fn perform_none(_world: &mut World, _this: Entity, _selector: &str) -> bool {
    false
}

fn sequence_item_none(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> Option<ItemResult> {
    None
}

fn frame_noop(_world: &mut World, _this: Entity, _dt: f32) {}

fn touch_noop(_world: &mut World, _this: Entity, _touch: Entity) {}

fn obj_noop(_world: &mut World, _this: Entity) {}

fn set_behavior_noop(_world: &mut World, _this: Entity, _behavior: Option<&str>) {}

fn bool_false(_world: &World, _this: Entity) -> bool {
    false
}

fn apply_move_noop(_world: &mut World, _this: Entity, _x: f32, _y: f32) {}

fn id_none(_world: &World, _this: Entity) -> Option<String> {
    None
}

fn button_noop(_world: &mut World, _listener: Entity, _button: Entity, _touch: Entity) {}

fn multi_model_event_noop(_world: &mut World, _listener: Entity, _logic: Entity, _event_id: &str) {}

fn message_complete_noop(_world: &mut World, _listener: Entity, _message: Entity) {}

fn draw_noop(_world: &mut World, _this: Entity, _gl: &mut GlContext) {}

fn draw_in_multi_model_noop(_world: &mut World, _this: Entity, _gl: &mut GlContext, _parent: Entity, _rgba: [f32; 4]) {}

fn texture_def_none(_world: &World, _this: Entity) -> Option<TextureDefId> {
    None
}

fn set_texture_def_noop(_world: &mut World, _this: Entity, _texture: Option<TextureDefId>) {}

fn add_selectable_logics_noop(_world: &World, _this: Entity, _logics: &mut Vec<Entity>) {}

impl Class {
    /// `NSObject`: responds to nothing.
    pub const NS_OBJECT: Class = Class {
        name: "NSObject",
        superclass: None,
        perform: perform_none,
        sequence_item: sequence_item_none,
        game_frame_order: 0,
        on_game_frame: frame_noop,
        touch_began_inside: touch_noop,
        touch_began_outside: touch_noop,
        touch_moved_entered: touch_noop,
        touch_moved_exited: touch_noop,
        touch_moved_inside: touch_noop,
        touch_moved_outside: touch_noop,
        touch_ended_inside: touch_noop,
        touch_ended_outside: touch_noop,
        shutdown: obj_noop,
        set_behavior: set_behavior_noop,
        is_sound_disabled: bool_false,
        apply_move_pos: apply_move_noop,
        id: id_none,
        close_screen_now: obj_noop,
        on_button_press: button_noop,
        on_button_release: button_noop,
        on_button_release_outside: button_noop,
        on_multi_model_event: multi_model_event_noop,
        on_message_complete: message_complete_noop,
        draw: draw_noop,
        draw_in_multi_model: draw_in_multi_model_noop,
        texture_def: texture_def_none,
        set_texture_def: set_texture_def_noop,
        enable_multi_model_mode: obj_noop,
        disable_multi_model_mode: obj_noop,
        hibernate: obj_noop,
        unhibernate: obj_noop,
        add_selectable_logics: add_selectable_logics_noop,
        ext: None,
    };

    /// `isKindOfClass:` against a class name.
    pub fn is_kind_of(&self, class_name: &str) -> bool {
        let mut class = Some(self);
        while let Some(current) = class {
            if current.name == class_name {
                return true;
            }
            class = current.superclass;
        }
        false
    }
}

/// The object's class (`isa`).
#[derive(Component, Clone, Copy)]
pub struct Isa(pub &'static Class);

/// The class of `object`. Panics if the entity is not an ObjC object.
pub fn isa(world: &World, object: Entity) -> &'static Class {
    let Some(isa) = world.get::<Isa>(object) else {
        panic!("entity {object} is not an Objective-C object (no Isa)");
    };
    isa.0
}

/// The class of `object`, or `None` for despawned / non-object entities (messaging nil).
pub fn try_isa(world: &World, object: Entity) -> Option<&'static Class> {
    let isa = world.get::<Isa>(object)?;
    Some(isa.0)
}

/// `[object isKindOfClass:…]`; false for nil.
pub fn is_kind_of(world: &World, object: Option<Entity>, class_name: &str) -> bool {
    let Some(object) = object else {
        return false;
    };
    match try_isa(world, object) {
        Some(class) => class.is_kind_of(class_name),
        None => false,
    }
}

/// `[object performSelector:NSSelectorFromString(selector)]` for a zero-argument selector.
/// Returns false if the object does not respond (or is nil).
pub fn perform(world: &mut World, object: Option<Entity>, selector: &str) -> bool {
    let Some(object) = object else {
        return false;
    };
    let Some(class) = try_isa(world, object) else {
        return false;
    };
    (class.perform)(world, object, selector)
}

/// Typed component access with readable panics: the port treats a missing component as the
/// original would treat sending a message to an object of the wrong class (a crash).
pub trait ObjectWorldExt {
    fn obj<T: Component>(&self, object: Entity) -> &T;
    fn obj_mut<T: Component<Mutability = bevy_ecs::component::Mutable>>(&mut self, object: Entity) -> Mut<'_, T>;
}

impl ObjectWorldExt for World {
    fn obj<T: Component>(&self, object: Entity) -> &T {
        match self.get::<T>(object) {
            Some(component) => component,
            None => panic!("{object} has no {}", std::any::type_name::<T>()),
        }
    }

    fn obj_mut<T: Component<Mutability = bevy_ecs::component::Mutable>>(&mut self, object: Entity) -> Mut<'_, T> {
        match self.get_mut::<T>(object) {
            Some(component) => component,
            None => panic!("{object} has no {}", std::any::type_name::<T>()),
        }
    }
}
