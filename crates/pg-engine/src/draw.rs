//! Immediate-mode draw output, consumed by `pg-render`.
//!
//! The original engine drew with OpenGL ES 1.1 in display-list order. A frame is therefore a
//! list of batches in submission order; each batch is a triangle list sharing one texture
//! and one blend mode. Vertex positions are in clip space (the engine applies the original
//! projection and modelview math on the CPU), colours are the raw (gamma-space) GL colours.

/// Index of a texture page (one `.raw` file / one GL texture) in the engine texture table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TextureId(pub u32);

/// GL blend states used by the original renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// `glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA)`, the engine default (`BCView::setupView`).
    #[default]
    Premultiplied,

    /// `glDisable(GL_BLEND)`: the engine skips blending for opaque textures at full alpha.
    Opaque,

    /// `glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA)`
    Alpha,

    /// `glBlendFunc(GL_SRC_ALPHA, GL_ONE)`
    Additive,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DrawVertex {
    /// Homogeneous clip-space position (x, y, z, w).
    pub position: [f32; 4],
    pub uv: [f32; 2],

    /// Raw RGBA vertex colour (GL_MODULATE with the texture sample).
    pub color: [f32; 4],
}

#[derive(Clone, Debug, Default)]
pub struct DrawBatch {
    /// `None` draws untextured (white texel) geometry.
    pub texture: Option<TextureId>,
    pub blend: BlendMode,

    /// Triangle list.
    pub vertices: Vec<DrawVertex>,
}

#[derive(Clone, Debug, Default)]
pub struct DrawList {
    pub clear_color: [f32; 4],
    pub batches: Vec<DrawBatch>,
}

impl DrawList {
    pub fn clear(&mut self) {
        self.batches.clear();
    }

    /// Appends triangles, extending the last batch when texture and blend mode match.
    pub fn push_triangles(&mut self, texture: Option<TextureId>, blend: BlendMode, vertices: &[DrawVertex]) {
        let can_extend = self
            .batches
            .last()
            .is_some_and(|batch| batch.texture == texture && batch.blend == blend);

        if !can_extend {
            let batch = DrawBatch {
                texture,
                blend,
                vertices: Vec::new(),
            };
            self.batches.push(batch);
        }

        let batch = self.batches.last_mut().expect("a batch was just ensured");
        batch.vertices.extend_from_slice(vertices);
    }

    /// Appends a quad given its four corners in triangle-strip order (0, 1, 2, 3).
    pub fn push_quad(&mut self, texture: Option<TextureId>, blend: BlendMode, corners: [DrawVertex; 4]) {
        let triangles = [corners[0], corners[1], corners[2], corners[2], corners[1], corners[3]];
        self.push_triangles(texture, blend, &triangles);
    }
}
