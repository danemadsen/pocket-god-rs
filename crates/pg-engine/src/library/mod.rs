//! `BCLibrary`: the registry of every definition the engine loads at start-up and the
//! default sequence-item handlers (`handlers`).
//!
//! The registries mirror the original's dictionaries / `std::map`s, including their quirks:
//! lookups of unknown ids return `None` (NULL / nil), later registrations with the same id win,
//! and animation / sound aliases overwrite existing entries.

pub mod handlers;
pub mod loader;

use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::animation::AnimationDefId;
use crate::sequence::SequenceDefRef;
use crate::sound::SoundId;
use crate::display::text::FontDef;
use crate::texture::{AtlasId, TextureDefId};

/// `BCGradientColor`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientColor {
    pub pos: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// `BCGradientDef`: stops in string order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GradientDef {
    pub colors: Vec<GradientColor>,
}

/// The `BCLibrary` singleton: registries plus the scratch state of its own item handlers.
#[derive(Resource, Default)]
pub struct Library {
    pub atlas_ids: HashMap<String, AtlasId>,
    pub texture_def_ids: HashMap<String, TextureDefId>,
    pub sequence_defs: HashMap<String, SequenceDefRef>,
    pub gradient_defs: HashMap<String, GradientDef>,
    pub font_defs: HashMap<String, FontDef>,

    /// `mDynamicTextDictionary`: the `BCText` objects created from `Timelines.bin` text records.
    pub dynamic_texts: HashMap<String, Entity>,
    pub display_groups: HashMap<String, Entity>,
    pub display_markers: HashMap<String, Entity>,

    /// `mAnimationDefDictionary` (aliases overwrite).
    pub animation_defs: HashMap<String, AnimationDefId>,

    /// `mSoundDictionary` (aliases overwrite).
    pub sounds: HashMap<String, SoundId>,

    /// `getLogic:` table (`DAT_1007e3030`, 632 entries), filled by `createLogicClasses`.
    pub logic_table: Vec<Entity>,

    /// `mAnimationDictionary`: listener-less players created by `setAnimation:::`.
    pub animation_players: HashMap<String, Entity>,

    // Scratch state of BCLibrary's own handlers, shared by every sequence that falls back to
    // the library (`docs/re/sequences.md` §3.8).
    pub stall_time: f32,
    pub stall_frames: i32,
    pub delta_value: f32,
    pub value: f32,
    pub delta_pos: [f32; 4],
}

impl Library {
    /// `-getTextureDefById:` (NULL for unknown ids).
    pub fn texture_def_id(&self, id: &str) -> Option<TextureDefId> {
        self.texture_def_ids.get(id).copied()
    }

    /// `-getAtlasTextureDefById:`.
    pub fn atlas_id(&self, id: &str) -> Option<AtlasId> {
        self.atlas_ids.get(id).copied()
    }

    /// `-getSequenceDefById:` (NULL for unknown ids).
    pub fn sequence_def(&self, id: &str) -> Option<SequenceDefRef> {
        self.sequence_defs.get(id).cloned()
    }

    /// `-getGradientDefById:`.
    pub fn gradient_def(&self, id: &str) -> Option<&GradientDef> {
        self.gradient_defs.get(id)
    }

    /// `-getFontDefById:`.
    pub fn font_def(&self, id: &str) -> Option<&FontDef> {
        self.font_defs.get(id)
    }

    /// `-getDynamicTextById:`.
    pub fn dynamic_text(&self, id: &str) -> Option<Entity> {
        self.dynamic_texts.get(id).copied()
    }

    /// `-getDisplayGroupById:`.
    pub fn display_group(&self, id: &str) -> Option<Entity> {
        self.display_groups.get(id).copied()
    }

    /// `-getDisplayMarkerById:`.
    pub fn display_marker(&self, id: &str) -> Option<Entity> {
        self.display_markers.get(id).copied()
    }

    /// `-getAnimationDefById:`.
    pub fn animation_def(&self, id: &str) -> Option<AnimationDefId> {
        self.animation_defs.get(id).copied()
    }

    /// `-setAnimationDefAlias:animationDef:`: replaces any entry with that name; nil defs are ignored.
    pub fn set_animation_def_alias(&mut self, alias: &str, def: Option<AnimationDefId>) {
        if let Some(def) = def {
            self.animation_defs.insert(alias.to_owned(), def);
        }
    }

    /// `-getSoundById:`.
    pub fn sound(&self, id: &str) -> Option<SoundId> {
        self.sounds.get(id).copied()
    }

    /// `-setSoundAlias:sound:`.
    pub fn set_sound_alias(&mut self, alias: &str, sound: Option<SoundId>) {
        if let Some(sound) = sound {
            self.sounds.insert(alias.to_owned(), sound);
        }
    }

    /// `-getLogic:` (the original does no bounds check).
    pub fn logic(&self, index: usize) -> Entity {
        self.logic_table[index]
    }
}

/// `[BCLibrary getDisplayGroupById:]` from a world.
pub fn display_group(world: &World, id: &str) -> Option<Entity> {
    world.resource::<Library>().display_group(id)
}

/// `[BCLibrary getDisplayMarkerById:]` from a world.
pub fn display_marker(world: &World, id: &str) -> Option<Entity> {
    world.resource::<Library>().display_marker(id)
}

/// `[BCLibrary getLogic:]` from a world.
pub fn logic(world: &World, index: usize) -> Entity {
    world.resource::<Library>().logic(index)
}

/// `[BCLibrary getTextureDefById:]` from a world.
pub fn texture_def_id(world: &World, id: &str) -> Option<TextureDefId> {
    world.resource::<Library>().texture_def_id(id)
}

/// `[BCLibrary getSequenceDefById:]` from a world.
pub fn sequence_def(world: &World, id: &str) -> Option<SequenceDefRef> {
    world.resource::<Library>().sequence_def(id)
}
