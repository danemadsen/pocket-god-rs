//! Atlas pages and texture definitions (`BCAtlasTextureDef`, `BCTextureDef`; `docs/re/textures.md`).
//!
//! An atlas is one GL texture (a "page") loaded from a `.raw` file (converted to
//! `assets/textures/<image>.png`). A texture definition is a sprite inside an atlas: its UVs
//! and its precomputed quad. Skin packs re-skin by copying another definition's atlas, UVs
//! and quad over a definition in place, and by patching rectangular regions of a loaded page
//! with small images ("bits"); both are reproduced here, so every reference to a definition
//! sees the skin immediately.

use bevy_ecs::prelude::*;

use crate::draw::TextureId;

/// Index of a `BCAtlasTextureDef` in [`Textures::atlases`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AtlasId(pub u32);

/// Index of a `BCTextureDef` in [`Textures::defs`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextureDefId(pub u32);

/// `BCAtlasTextureDef`.
#[derive(Clone, Debug, Default)]
pub struct AtlasDef {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub has_alpha: bool,

    /// Texture channel (`LocationAtlas`, `MainAtlas`, …) used by channel loads.
    pub channel: Option<String>,

    /// `GL_LINEAR_MIPMAP_LINEAR` + generated mipmaps.
    pub mipmap: bool,

    /// The image currently uploaded to this slot (`<image>.png`); `None` = never loaded
    /// (the original leaves the GL texture without storage: draws sample nothing).
    pub image: Option<String>,

    /// The slot's GL texture name (stable for the slot's lifetime).
    pub page: TextureId,
}

/// `BCTextureDef`: a sprite in an atlas.
#[derive(Clone, Debug, Default)]
pub struct TextureDef {
    pub id: String,
    pub atlas: Option<AtlasId>,

    /// `+0x08`: translucent texture, always blend.
    pub blend: bool,

    /// Sprite size in pixels (`+0x20`, `+0x24`).
    pub width: f32,
    pub height: f32,

    /// Frame size (`+0x28`, `+0x2c`): glyph advance / line height for fonts.
    pub frame_width: f32,
    pub frame_height: f32,

    /// `+0x30`: UVs of v0..v3 (top-left, top-right, bottom-left, bottom-right).
    pub uvs: [f32; 8],

    /// `+0x50`: quad v0..v3 as (x, y, z), local pixel units, y up.
    pub vertices: [f32; 12],
}

/// A change the renderer has to apply to its GPU images.
#[derive(Clone, Debug, PartialEq)]
pub enum TextureRequest {
    /// Replace the whole image of `page` with `assets/textures/<image>.png`
    /// (`glTexImage2D` at the slot's size).
    Load { page: TextureId, image: String, width: u32, height: u32, mipmap: bool },

    /// Copy the `width × height` rect at (`src_x`, `src_y`) of `assets/textures/<image>.png`
    /// into `page` at (`dst_x`, `dst_y`) (`glTexSubImage2D`, skin "bits").
    Patch {
        page: TextureId,
        image: String,
        src_x: u32,
        src_y: u32,
        width: u32,
        height: u32,
        dst_x: u32,
        dst_y: u32,
    },
}

/// All atlas slots and texture definitions, plus pending renderer requests.
#[derive(Resource, Default)]
pub struct Textures {
    pub atlases: Vec<AtlasDef>,
    pub defs: Vec<TextureDef>,
    pub requests: Vec<TextureRequest>,
}

impl Textures {
    pub fn atlas(&self, id: AtlasId) -> &AtlasDef {
        &self.atlases[id.0 as usize]
    }

    pub fn atlas_mut(&mut self, id: AtlasId) -> &mut AtlasDef {
        &mut self.atlases[id.0 as usize]
    }

    pub fn def(&self, id: TextureDefId) -> &TextureDef {
        &self.defs[id.0 as usize]
    }

    pub fn def_mut(&mut self, id: TextureDefId) -> &mut TextureDef {
        &mut self.defs[id.0 as usize]
    }

    /// The GL texture sampled by a definition (`*texDef.textureName`), if its slot has an image.
    pub fn page_of(&self, id: TextureDefId) -> Option<TextureId> {
        let atlas = self.def(id).atlas?;
        let slot = self.atlas(atlas);
        slot.image.as_ref()?;
        Some(slot.page)
    }

    /// Registers a slot (`addAtlasTextureDef:`); its page id is fixed from now on.
    pub fn add_atlas(&mut self, mut atlas: AtlasDef) -> AtlasId {
        let id = AtlasId(self.atlases.len() as u32);
        atlas.page = TextureId(id.0 + 1);
        self.atlases.push(atlas);
        id
    }
}

/// `FUN_100038d68(fileName, ext, atlasDef)` / `activateTextureAtlas`: upload an image into a slot.
pub fn load_atlas(world: &mut World, atlas: AtlasId, image: &str) {
    let mut textures = world.resource_mut::<Textures>();
    let slot = textures.atlas_mut(atlas);
    slot.image = Some(image.to_owned());
    let request = TextureRequest::Load {
        page: slot.page,
        image: image.to_owned(),
        width: slot.width,
        height: slot.height,
        mipmap: slot.mipmap,
    };
    textures.requests.push(request);
}

/// One `glTexSubImage2D` of skin bits into a slot.
pub struct Patch<'a> {
    pub image: &'a str,
    pub src_x: u32,
    pub src_y: u32,
    pub width: u32,
    pub height: u32,
    pub dst_x: u32,
    pub dst_y: u32,
}

/// `activateSkinTextureBitsBatchDef`: copy a rect of an image into a slot.
pub fn patch_atlas(world: &mut World, atlas: AtlasId, patch: Patch) {
    let mut textures = world.resource_mut::<Textures>();
    let page = textures.atlas(atlas).page;
    let request = TextureRequest::Patch {
        page,
        image: patch.image.to_owned(),
        src_x: patch.src_x,
        src_y: patch.src_y,
        width: patch.width,
        height: patch.height,
        dst_x: patch.dst_x,
        dst_y: patch.dst_y,
    };
    textures.requests.push(request);
}

/// `FUN_100038f24`: copy atlas, size, UV and quad fields of `source` over `target` (re-skin in place).
pub fn copy_texture_def(world: &mut World, target: TextureDefId, source: TextureDefId) {
    let mut textures = world.resource_mut::<Textures>();
    let source_def = textures.def(source).clone();
    let target_def = textures.def_mut(target);
    let id = std::mem::take(&mut target_def.id);
    *target_def = source_def;
    target_def.id = id;
}
