//! Turns a frame into `Mesh2d` batches: the clear colour, the engine's draw list and the splash.
//!
//! The window is cleared black (the letterbox bars) by the camera, whose clear always covers
//! the whole window; `glClear` of the original's renderbuffer therefore becomes an opaque quad
//! over the game viewport, drawn first. Each batch is one pooled `Mesh2d` entity; batch order
//! is kept through the transparent 2D phase's sort key (translation z).

use std::borrow::Cow;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::PrimitiveTopology;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use pg_engine::draw::{BlendMode, DrawVertex};

use crate::material::{ATTRIBUTE_CLIP_POSITION, BcMaterial};
use crate::pages::TexturePages;
use crate::presentation::SplashPresented;
use crate::{FrameDrawList, LogicalScreen, SplashScreen};

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

#[derive(Component)]
pub(crate) struct BatchSlot;

/// Reused batch entities and one material per (image, blend mode).
#[derive(Resource, Default)]
pub(crate) struct BatchPool {
    slots: Vec<(Entity, Handle<Mesh>)>,
    materials: HashMap<(AssetId<Image>, BlendMode), Handle<BcMaterial>>,

    /// The splash image the materials were made for.
    splash_image: Option<AssetId<Image>>,

    /// The batch entity drawing the splash image, while one is up.
    pub(crate) splash_slot: Option<Entity>,
}

impl BatchPool {
    /// Drops the materials of `image`, which would otherwise keep it alive.
    fn forget_image(&mut self, image: AssetId<Image>) {
        self.materials.retain(|(material_image, _), _| *material_image != image);
    }
}

/// One batch ready for the GPU.
struct ComposedBatch<'a> {
    image: Handle<Image>,
    blend: BlendMode,
    vertices: Cow<'a, [DrawVertex]>,
}

fn vertex(x: f32, y: f32, u: f32, v: f32, color: [f32; 4]) -> DrawVertex {
    DrawVertex {
        position: [x, y, 0.0, 1.0],
        uv: [u, v],
        color,
    }
}

/// The two triangles of an axis-aligned quad in clip space (`top > bottom`), texture row 0 at
/// the top.
fn clip_quad(left: f32, right: f32, top: f32, bottom: f32, color: [f32; 4]) -> Vec<DrawVertex> {
    let top_left = vertex(left, top, 0.0, 0.0, color);
    let top_right = vertex(right, top, 1.0, 0.0, color);
    let bottom_left = vertex(left, bottom, 0.0, 1.0, color);
    let bottom_right = vertex(right, bottom, 1.0, 1.0, color);
    vec![top_left, top_right, bottom_left, bottom_left, top_right, bottom_right]
}

/// `glClear` with the frame's clear colour. The original's layer is opaque, so the alpha of
/// the clear colour never shows.
fn clear_quad(clear_color: [f32; 4]) -> Vec<DrawVertex> {
    let [red, green, blue, _] = clear_color;
    let color = [red, green, blue, 1.0];
    clip_quad(-1.0, 1.0, 1.0, -1.0, color)
}

/// Half extents, in clip units, of the largest rect of `image_aspect` that fits a viewport of
/// `viewport_aspect` (width / height).
pub(crate) fn fit_half_extents(image_aspect: f32, viewport_aspect: f32) -> Vec2 {
    // NaN fails the comparisons too.
    let valid = image_aspect > 0.0 && viewport_aspect > 0.0;
    if !valid {
        return Vec2::ONE;
    }
    if image_aspect > viewport_aspect {
        Vec2::new(1.0, viewport_aspect / image_aspect)
    } else {
        Vec2::new(image_aspect / viewport_aspect, 1.0)
    }
}

/// The aspect ratio of the viewport the game draws into.
fn viewport_aspect(screen: &LogicalScreen) -> f32 {
    match screen.viewport {
        Some(rect) if rect.height() > 0 => rect.width() as f32 / rect.height() as f32,
        _ => screen.width / screen.height,
    }
}

/// The splash: black over the whole viewport, then the image letterboxed inside it.
fn splash_batches(splash: &SplashScreen, screen: &LogicalScreen, white: &Handle<Image>) -> Vec<ComposedBatch<'static>> {
    let Some(image) = splash.image.clone() else {
        return Vec::new();
    };

    let backdrop_quad = clear_quad(BLACK);
    let backdrop = ComposedBatch {
        image: white.clone(),
        blend: BlendMode::Opaque,
        vertices: Cow::Owned(backdrop_quad),
    };

    let image_aspect = splash.size.x as f32 / splash.size.y.max(1) as f32;
    let screen_aspect = viewport_aspect(screen);
    let half = fit_half_extents(image_aspect, screen_aspect);
    let picture_quad = clip_quad(-half.x, half.x, half.y, -half.y, WHITE);
    let picture = ComposedBatch {
        image,
        blend: BlendMode::Opaque,
        vertices: Cow::Owned(picture_quad),
    };
    vec![backdrop, picture]
}

/// Everything the game camera draws this frame, in order.
fn compose<'a>(
    draw_list: &'a FrameDrawList,
    pages: &TexturePages,
    splash: &SplashScreen,
    screen: &LogicalScreen,
) -> Vec<ComposedBatch<'a>> {
    let mut composed = Vec::with_capacity(draw_list.0.batches.len() + 3);

    let clear_vertices = clear_quad(draw_list.0.clear_color);
    let clear = ComposedBatch {
        image: pages.white.clone(),
        blend: BlendMode::Opaque,
        vertices: Cow::Owned(clear_vertices),
    };
    composed.push(clear);

    for batch in &draw_list.0.batches {
        // A page without an image yet draws nothing.
        let Some(image) = pages.handle_for(batch.texture) else {
            continue;
        };
        let composed_batch = ComposedBatch {
            image,
            blend: batch.blend,
            vertices: Cow::Borrowed(&batch.vertices),
        };
        composed.push(composed_batch);
    }

    let splash_overlay = splash_batches(splash, screen, &pages.white);
    composed.extend(splash_overlay);
    composed
}

fn empty_batch_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
}

fn write_batch_mesh(mesh: &mut Mesh, vertices: &[DrawVertex]) {
    let positions: Vec<[f32; 4]> = vertices.iter().map(|vertex| vertex.position).collect();
    let uvs: Vec<[f32; 2]> = vertices.iter().map(|vertex| vertex.uv).collect();
    let colors: Vec<[f32; 4]> = vertices.iter().map(|vertex| vertex.color).collect();
    mesh.insert_attribute(ATTRIBUTE_CLIP_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
}

fn material_for(
    pool: &mut BatchPool,
    materials: &mut Assets<BcMaterial>,
    image: &Handle<Image>,
    blend: BlendMode,
) -> Handle<BcMaterial> {
    let key = (image.id(), blend);
    if let Some(existing) = pool.materials.get(&key) {
        return existing.clone();
    }

    let material = BcMaterial {
        texture: image.clone(),
        blend,
    };
    let handle = materials.add(material);
    pool.materials.insert(key, handle.clone());
    handle
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_batches(
    mut commands: Commands,
    draw_list: Res<FrameDrawList>,
    mut pages: ResMut<TexturePages>,
    splash: Res<SplashScreen>,
    screen: Res<LogicalScreen>,
    mut pool: ResMut<BatchPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BcMaterial>>,
    mut slots: Query<(&mut Visibility, &mut Transform, &mut MeshMaterial2d<BcMaterial>), With<BatchSlot>>,
    presented: Res<SplashPresented>,
) {
    let changed = draw_list.is_changed() || pages.is_changed() || splash.is_changed() || screen.is_changed();
    if !changed {
        return;
    }

    // Materials of images that are no longer drawn would keep those images alive.
    if !pages.retired.is_empty() {
        let retired = std::mem::take(&mut pages.retired);
        for image in retired {
            pool.forget_image(image);
        }
    }
    let splash_image = splash.image.as_ref().map(Handle::id);
    if pool.splash_image != splash_image {
        if let Some(previous) = pool.splash_image {
            pool.forget_image(previous);
        }
        pool.splash_image = splash_image;
        presented.set(false);
    }

    let composed = compose(&draw_list, &pages, &splash, &screen);
    for (index, batch) in composed.iter().enumerate() {
        let material = material_for(&mut pool, &mut materials, &batch.image, batch.blend);
        let depth = index as f32;

        if index >= pool.slots.len() {
            let mut mesh = empty_batch_mesh();
            write_batch_mesh(&mut mesh, &batch.vertices);
            let mesh_handle = meshes.add(mesh);

            let slot = (
                Mesh2d(mesh_handle.clone()),
                MeshMaterial2d(material),
                Transform::from_xyz(0.0, 0.0, depth),
                Visibility::Visible,
                NoFrustumCulling,
                BatchSlot,
            );
            let entity = commands.spawn(slot).id();
            pool.slots.push((entity, mesh_handle));
            continue;
        }

        let (entity, mesh_handle) = pool.slots[index].clone();
        if let Some(mut mesh) = meshes.get_mut(&mesh_handle) {
            write_batch_mesh(&mut mesh, &batch.vertices);
        }
        let Ok((mut visibility, mut transform, mut slot_material)) = slots.get_mut(entity) else {
            continue;
        };
        if *visibility != Visibility::Visible {
            *visibility = Visibility::Visible;
        }
        if transform.translation.z != depth {
            transform.translation.z = depth;
        }
        if slot_material.0 != material {
            slot_material.0 = material;
        }
    }

    for (entity, _) in pool.slots.iter().skip(composed.len()) {
        if let Ok((mut visibility, _, _)) = slots.get_mut(*entity)
            && *visibility != Visibility::Hidden
        {
            *visibility = Visibility::Hidden;
        }
    }

    // The splash picture is the last batch.
    pool.splash_slot = None;
    if splash.image.is_some()
        && let Some(last) = composed.len().checked_sub(1)
    {
        pool.splash_slot = pool.slots.get(last).map(|(entity, _)| *entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splash_images_letterbox_inside_the_viewport() {
        let wide_in_narrow = fit_half_extents(1136.0 / 640.0, 1.5);
        assert_eq!(wide_in_narrow.x, 1.0);
        assert!((wide_in_narrow.y - 1.5 / 1.775).abs() < 1e-6);

        let narrow_in_wide = fit_half_extents(4.0 / 3.0, 1.775);
        assert_eq!(narrow_in_wide.y, 1.0);
        assert!((narrow_in_wide.x - (4.0 / 3.0) / 1.775).abs() < 1e-6);

        assert_eq!(fit_half_extents(1.5, 1.5), Vec2::ONE);
        assert_eq!(fit_half_extents(0.0, 1.5), Vec2::ONE);
    }

    #[test]
    fn the_clear_quad_is_opaque_and_covers_the_viewport() {
        let quad = clear_quad([0.25, 0.5, 0.75, 0.0]);
        assert_eq!(quad.len(), 6);
        for corner in &quad {
            assert_eq!(corner.color, [0.25, 0.5, 0.75, 1.0]);
            assert_eq!(corner.position[0].abs(), 1.0);
            assert_eq!(corner.position[1].abs(), 1.0);
        }
    }
}
