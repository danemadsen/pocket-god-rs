use bevy::asset::{AssetPath, embedded_path};
use bevy::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, CompareFunction,
    RenderPipelineDescriptor, SpecializedMeshPipelineError, VertexFormat,
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dKey, Material2dPipeline};
use pg_engine::draw::BlendMode;

/// Clip-space vertex position computed on the CPU by the engine.
pub const ATTRIBUTE_CLIP_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("BcClipPosition", 1_684_120_391, VertexFormat::Float32x4);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(BcMaterialKey)]
pub struct BcMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,

    pub blend: BlendMode,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BcMaterialKey {
    blend: BlendMode,
}

impl From<&BcMaterial> for BcMaterialKey {
    fn from(material: &BcMaterial) -> Self {
        Self {
            blend: material.blend,
        }
    }
}

fn shader_path() -> ShaderRef {
    let path = AssetPath::from_path_buf(embedded_path!("bc.wgsl"));
    let embedded = path.with_source("embedded");
    ShaderRef::Path(embedded)
}

/// The GL blend state for a mode; `None` disables blending.
fn gl_blend_state(blend: BlendMode) -> Option<BlendState> {
    let (src, dst) = match blend {
        BlendMode::Premultiplied => (BlendFactor::One, BlendFactor::OneMinusSrcAlpha),
        BlendMode::Opaque => return None,
        BlendMode::Alpha => (BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha),
        BlendMode::Additive => (BlendFactor::SrcAlpha, BlendFactor::One),
    };

    // glBlendFunc applies the same factors to the alpha channel.
    let component = BlendComponent {
        src_factor: src,
        dst_factor: dst,
        operation: BlendOperation::Add,
    };

    let state = BlendState {
        color: component,
        alpha: component,
    };
    Some(state)
}

impl Material2d for BcMaterial {
    fn vertex_shader() -> ShaderRef {
        shader_path()
    }

    fn fragment_shader() -> ShaderRef {
        shader_path()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        _pipeline: &Material2dPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let attributes = [
            ATTRIBUTE_CLIP_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(2),
        ];
        let vertex_layout = layout.0.get_layout(&attributes)?;
        descriptor.vertex.buffers = vec![vertex_layout];

        if let Some(depth_stencil) = descriptor.depth_stencil.as_mut() {
            depth_stencil.depth_write_enabled = Some(false);
            depth_stencil.depth_compare = Some(CompareFunction::Always);
        }

        let blend_state = gl_blend_state(key.bind_group_data.blend);
        let fragment = descriptor.fragment.as_mut().expect("material has a fragment stage");
        for target in fragment.targets.iter_mut().flatten() {
            target.blend = blend_state;
        }

        Ok(())
    }
}
