//! Texture pages: the GPU images of the engine's atlas slots (`docs/re/textures.md` §5.2, §7.2,
//! §8).
//!
//! Every slot keeps one RGBA8 image, non-sRGB so texel values reach the shader unchanged (the
//! converted PNGs hold premultiplied texels, kept as they are), sampled with linear filtering
//! and repeat wrap: the GLES defaults the original never changes. A `Load` replaces the whole
//! image (`glTexImage2D` at the slot's size); a `Patch` copies a rect of a decoded image into
//! it (`glTexSubImage2D` of skin bits). The CPU copy stays in the main world so patches can
//! edit it, and Bevy re-uploads a modified image into the same GPU texture, so a page keeps its
//! handle (and the materials that sample it) for the slot's lifetime.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use pg_engine::draw::TextureId;
use pg_engine::texture::{TextureRequest, Textures};

/// Bytes per RGBA8 texel.
const TEXEL_BYTES: usize = 4;

/// A decoded RGBA8 image, rows top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Decodes PNG bytes to RGBA8 without changing any texel value.
pub fn decode_png(bytes: &[u8]) -> Option<DecodedImage> {
    let decoded = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok()?;
    let rgba = decoded.into_rgba8();
    let width = rgba.width();
    let height = rgba.height();

    let image = DecodedImage {
        width,
        height,
        pixels: rgba.into_raw(),
    };
    Some(image)
}

/// `assets/textures/<image>.png` (the original's `<image>.raw`), decoded from the embedded
/// assets.
pub fn decode_texture_image(image: &str) -> Option<DecodedImage> {
    let path = format!("textures/{image}.png");
    let bytes = pg_assets::file(&path)?;
    decode_png(bytes)
}

/// The rect of one `glTexSubImage2D`: `width × height` texels from (`src_x`, `src_y`) of the
/// source image to (`dst_x`, `dst_y`) of the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RectCopy {
    pub src_x: u32,
    pub src_y: u32,
    pub width: u32,
    pub height: u32,
    pub dst_x: u32,
    pub dst_y: u32,
}

/// Whether the `width × height` rect at (`x`, `y`) lies inside a `limit_width × limit_height`
/// image.
fn rect_fits(x: u32, y: u32, width: u32, height: u32, limit_width: u32, limit_height: u32) -> bool {
    let right = u64::from(x) + u64::from(width);
    let bottom = u64::from(y) + u64::from(height);
    right <= u64::from(limit_width) && bottom <= u64::from(limit_height)
}

/// Copies the rect of `copy` from `source` into `target`, an RGBA8 image of
/// `target_width × target_height`. Like `glTexSubImage2D`, a rect that leaves the target (or
/// the source) changes nothing; returns whether the texels were copied.
pub fn copy_rect(source: &DecodedImage, copy: RectCopy, target: &mut [u8], target_width: u32, target_height: u32) -> bool {
    let source_fits = rect_fits(copy.src_x, copy.src_y, copy.width, copy.height, source.width, source.height);
    let target_fits = rect_fits(copy.dst_x, copy.dst_y, copy.width, copy.height, target_width, target_height);
    let target_size = target_width as usize * target_height as usize * TEXEL_BYTES;
    if !source_fits || !target_fits || target.len() < target_size {
        return false;
    }

    let row_bytes = copy.width as usize * TEXEL_BYTES;
    for row in 0..copy.height as usize {
        let source_row = copy.src_y as usize + row;
        let target_row = copy.dst_y as usize + row;
        let source_start = (source_row * source.width as usize + copy.src_x as usize) * TEXEL_BYTES;
        let target_start = (target_row * target_width as usize + copy.dst_x as usize) * TEXEL_BYTES;

        let source_texels = &source.pixels[source_start..source_start + row_bytes];
        target[target_start..target_start + row_bytes].copy_from_slice(source_texels);
    }
    true
}

/// The level-0 data of a `width × height` slot loaded with `image`: the image at the slot's
/// top-left, transparent black elsewhere (every shipped image has its slot's size).
pub fn fit_to_slot(image: &DecodedImage, width: u32, height: u32) -> Vec<u8> {
    let mut data = vec![0; width as usize * height as usize * TEXEL_BYTES];
    let copy = RectCopy {
        src_x: 0,
        src_y: 0,
        width: image.width.min(width),
        height: image.height.min(height),
        dst_x: 0,
        dst_y: 0,
    };
    copy_rect(image, copy, &mut data, width, height);
    data
}

/// Levels of a full mip chain down to 1 × 1.
pub fn mip_level_count(width: u32, height: u32) -> u32 {
    let largest = width.max(height).max(1);
    32 - largest.leading_zeros()
}

/// One level of a mip chain: each texel is the average of the (up to) 2 × 2 texels of the
/// level above.
fn downsample(level: &[u8], width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    let next_width = (width / 2).max(1);
    let next_height = (height / 2).max(1);
    let mut next = vec![0; next_width as usize * next_height as usize * TEXEL_BYTES];

    for y in 0..next_height {
        for x in 0..next_width {
            let x0 = (x * 2).min(width - 1);
            let x1 = (x * 2 + 1).min(width - 1);
            let y0 = (y * 2).min(height - 1);
            let y1 = (y * 2 + 1).min(height - 1);
            let corners = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];

            for channel in 0..TEXEL_BYTES {
                let mut sum = 0u32;
                for (corner_x, corner_y) in corners {
                    let index = (corner_y as usize * width as usize + corner_x as usize) * TEXEL_BYTES + channel;
                    sum += u32::from(level[index]);
                }
                let target = (y as usize * next_width as usize + x as usize) * TEXEL_BYTES + channel;
                next[target] = ((sum + 2) / 4) as u8;
            }
        }
    }
    (next, next_width, next_height)
}

/// `GL_GENERATE_MIPMAP`: level 0 followed by every smaller level, down to 1 × 1.
pub fn build_mip_chain(level0: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut chain = level0.to_vec();
    let mut level = level0.to_vec();
    let mut level_width = width;
    let mut level_height = height;

    for _ in 1..mip_level_count(width, height) {
        let (next, next_width, next_height) = downsample(&level, level_width, level_height);
        chain.extend_from_slice(&next);
        level = next;
        level_width = next_width;
        level_height = next_height;
    }
    chain
}

/// The sampler of every page: `GL_LINEAR` (or `GL_LINEAR_MIPMAP_LINEAR`) and `GL_REPEAT`.
fn page_sampler(mipmap: bool) -> ImageSampler {
    let mipmap_filter = if mipmap { ImageFilterMode::Linear } else { ImageFilterMode::Nearest };
    let descriptor = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter,
        ..default()
    };
    ImageSampler::Descriptor(descriptor)
}

/// The GPU image of a page from its level-0 RGBA8 data (with a generated mip chain when the
/// slot asks for mipmaps). The data stays in the main world for later patches.
pub fn page_image(level0: Vec<u8>, width: u32, height: u32, mipmap: bool) -> Image {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    let mut image = Image::new(size, TextureDimension::D2, level0, TextureFormat::Rgba8Unorm, usage);

    if mipmap {
        let level0 = image.data.take().unwrap_or_default();
        let chain = build_mip_chain(&level0, width, height);
        image.data = Some(chain);
        image.texture_descriptor.mip_level_count = mip_level_count(width, height);
    }
    image.sampler = page_sampler(mipmap);
    image
}

/// One GL texture name of the engine (`TextureId`) and the image behind it.
#[derive(Clone, Debug)]
struct TexturePage {
    handle: Handle<Image>,

    /// Mipmapped pages are rebuilt whole on every change: Bevy's in-place re-upload only
    /// writes level 0.
    mipmap: bool,
}

/// GPU images for engine texture pages, keyed by [`TextureId`] (stable per slot).
#[derive(Resource, Default)]
pub struct TexturePages {
    pages: HashMap<TextureId, TexturePage>,
    pub(crate) white: Handle<Image>,

    /// Images replaced by new handles since the renderer last dropped their materials.
    pub(crate) retired: Vec<AssetId<Image>>,
}

impl TexturePages {
    /// Registers an image for a page (images made outside the engine, e.g. examples).
    pub fn insert(&mut self, id: TextureId, image: Handle<Image>) {
        let page = TexturePage {
            handle: image,
            mipmap: false,
        };
        self.replace(id, page);
    }

    pub fn remove(&mut self, id: TextureId) -> Option<Handle<Image>> {
        let page = self.pages.remove(&id)?;
        self.retired.push(page.handle.id());
        Some(page.handle)
    }

    pub fn get(&self, id: TextureId) -> Option<&Handle<Image>> {
        let page = self.pages.get(&id)?;
        Some(&page.handle)
    }

    /// The image a batch samples: its page, or white for untextured draws (`None`); `None`
    /// when the page has no image yet.
    pub(crate) fn handle_for(&self, texture: Option<TextureId>) -> Option<Handle<Image>> {
        match texture {
            Some(id) => self.get(id).cloned(),
            None => Some(self.white.clone()),
        }
    }

    fn replace(&mut self, id: TextureId, page: TexturePage) {
        let new_id = page.handle.id();
        let previous = self.pages.insert(id, page);
        if let Some(previous) = previous
            && previous.handle.id() != new_id
        {
            self.retired.push(previous.handle.id());
        }
    }
}

/// Decoded patch sources (skin bits images), kept for the whole run: the same bits are
/// applied again whenever a skin is re-selected.
#[derive(Resource, Default)]
pub struct SourceImages {
    images: HashMap<String, Option<Arc<DecodedImage>>>,
}

impl SourceImages {
    /// The decoded `assets/textures/<image>.png`, decoding it on first use.
    fn get(&mut self, image: &str) -> Option<Arc<DecodedImage>> {
        if let Some(cached) = self.images.get(image) {
            return cached.clone();
        }

        let decoded = decode_texture_image(image);
        if decoded.is_none() {
            warn!("texture image {image} is not embedded or not a valid PNG");
        }
        let shared = decoded.map(Arc::new);
        self.images.insert(image.to_owned(), shared.clone());
        shared
    }
}

/// `glTexImage2D` of `assets/textures/<image>.png` into `page` at the slot's size.
fn load_page(
    page: TextureId,
    image: &str,
    width: u32,
    height: u32,
    mipmap: bool,
    images: &mut Assets<Image>,
    pages: &mut TexturePages,
) {
    let Some(decoded) = decode_texture_image(image) else {
        warn!("texture image {image} is not embedded or not a valid PNG; page {} keeps its image", page.0);
        return;
    };
    if decoded.width != width || decoded.height != height {
        warn!(
            "texture image {image} is {}x{}, its slot {width}x{height}",
            decoded.width, decoded.height
        );
    }

    let level0 = fit_to_slot(&decoded, width, height);
    let new_image = page_image(level0, width, height, mipmap);

    let existing = pages.pages.get(&page).cloned();
    let reusable = existing.filter(|existing| !mipmap && !existing.mipmap);
    if let Some(existing) = reusable {
        // Same handle: Bevy writes the new texels into the page's GPU texture.
        let replaced = images.insert(existing.handle.id(), new_image);
        if let Err(error) = replaced {
            warn!("texture page {} could not take {image}: {error}", page.0);
        }
        return;
    }

    let handle = images.add(new_image);
    let texture_page = TexturePage { handle, mipmap };
    pages.replace(page, texture_page);
}

/// `glTexSubImage2D` of a rect of `assets/textures/<image>.png` into `page`. A page that never
/// received an image has no storage in the original, so the patch does nothing.
fn patch_page(
    page: TextureId,
    image: &str,
    copy: RectCopy,
    images: &mut Assets<Image>,
    pages: &mut TexturePages,
    sources: &mut SourceImages,
) {
    let Some(texture_page) = pages.pages.get(&page).cloned() else {
        warn!("patch of texture page {} before any image was loaded into it", page.0);
        return;
    };
    let Some(source) = sources.get(image) else {
        return;
    };
    let Some(mut target) = images.get_mut(texture_page.handle.id()) else {
        return;
    };

    let target_width = target.width();
    let target_height = target.height();
    let Some(data) = target.data.as_mut() else {
        return;
    };
    let copied = copy_rect(&source, copy, data, target_width, target_height);
    if !copied {
        warn!("patch {copy:?} of {image} does not fit texture page {}", page.0);
        return;
    }
    if !texture_page.mipmap {
        return;
    }

    let level0_len = target_width as usize * target_height as usize * TEXEL_BYTES;
    let level0 = data[..level0_len].to_vec();
    drop(target);

    let rebuilt = page_image(level0, target_width, target_height, true);
    let handle = images.add(rebuilt);
    let rebuilt_page = TexturePage { handle, mipmap: true };
    pages.replace(page, rebuilt_page);
}

/// Applies the engine's pending texture requests ([`Textures::requests`]) in order.
pub(crate) fn apply_texture_requests(
    textures: Option<ResMut<Textures>>,
    mut images: ResMut<Assets<Image>>,
    mut pages: ResMut<TexturePages>,
    mut sources: ResMut<SourceImages>,
) {
    let Some(mut textures) = textures else {
        return;
    };
    if textures.requests.is_empty() {
        return;
    }

    let requests = std::mem::take(&mut textures.requests);
    for request in requests {
        match request {
            TextureRequest::Load {
                page,
                image,
                width,
                height,
                mipmap,
            } => {
                load_page(page, &image, width, height, mipmap, &mut images, &mut pages);
            }
            TextureRequest::Patch {
                page,
                image,
                src_x,
                src_y,
                width,
                height,
                dst_x,
                dst_y,
            } => {
                let copy = RectCopy {
                    src_x,
                    src_y,
                    width,
                    height,
                    dst_x,
                    dst_y,
                };
                patch_page(page, &image, copy, &mut images, &mut pages, &mut sources);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `width × height` image whose texel at (x, y) is `[x, y, tag, 255]`.
    fn labelled_image(width: u32, height: u32, tag: u8) -> DecodedImage {
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let texel = [x as u8, y as u8, tag, 255];
                pixels.extend_from_slice(&texel);
            }
        }
        DecodedImage { width, height, pixels }
    }

    fn texel(data: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
        let start = (y as usize * width as usize + x as usize) * TEXEL_BYTES;
        let mut value = [0; 4];
        value.copy_from_slice(&data[start..start + TEXEL_BYTES]);
        value
    }

    #[test]
    fn patches_copy_the_source_rect_to_the_destination() {
        let source = labelled_image(8, 8, 7);
        let mut page = vec![0; 16 * 16 * TEXEL_BYTES];
        let copy = RectCopy {
            src_x: 2,
            src_y: 3,
            width: 4,
            height: 2,
            dst_x: 10,
            dst_y: 5,
        };

        assert!(copy_rect(&source, copy, &mut page, 16, 16));
        assert_eq!(texel(&page, 16, 10, 5), [2, 3, 7, 255]);
        assert_eq!(texel(&page, 16, 13, 6), [5, 4, 7, 255]);
        assert_eq!(texel(&page, 16, 9, 5), [0, 0, 0, 0]);
        assert_eq!(texel(&page, 16, 14, 5), [0, 0, 0, 0]);
        assert_eq!(texel(&page, 16, 10, 7), [0, 0, 0, 0]);
    }

    #[test]
    fn patches_that_leave_the_page_change_nothing() {
        let source = labelled_image(8, 8, 1);
        let mut page = vec![0; 16 * 16 * TEXEL_BYTES];
        let outside_target = RectCopy {
            src_x: 0,
            src_y: 0,
            width: 4,
            height: 4,
            dst_x: 14,
            dst_y: 0,
        };
        let outside_source = RectCopy {
            src_x: 6,
            src_y: 0,
            width: 4,
            height: 4,
            dst_x: 0,
            dst_y: 0,
        };

        assert!(!copy_rect(&source, outside_target, &mut page, 16, 16));
        assert!(!copy_rect(&source, outside_source, &mut page, 16, 16));
        assert!(page.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn loads_keep_premultiplied_texels_unchanged() {
        let mut source = labelled_image(4, 4, 0);
        // a = 0 with rgb > 0: additive texels the original blends as they are.
        source.pixels[0..4].copy_from_slice(&[200, 10, 30, 0]);
        let data = fit_to_slot(&source, 4, 4);
        assert_eq!(data, source.pixels);
    }

    #[test]
    fn smaller_images_fill_the_slot_from_its_top_left() {
        let source = labelled_image(2, 2, 9);
        let data = fit_to_slot(&source, 4, 4);
        assert_eq!(data.len(), 4 * 4 * TEXEL_BYTES);
        assert_eq!(texel(&data, 4, 1, 1), [1, 1, 9, 255]);
        assert_eq!(texel(&data, 4, 2, 1), [0, 0, 0, 0]);
        assert_eq!(texel(&data, 4, 0, 3), [0, 0, 0, 0]);
    }

    #[test]
    fn mip_chains_average_down_to_one_texel() {
        assert_eq!(mip_level_count(1024, 1024), 11);
        assert_eq!(mip_level_count(256, 512), 10);
        assert_eq!(mip_level_count(1, 1), 1);

        let level0 = [0, 0, 0, 0, 4, 8, 12, 16, 8, 16, 24, 32, 12, 24, 36, 48];
        let chain = build_mip_chain(&level0, 2, 2);
        assert_eq!(chain.len(), level0.len() + TEXEL_BYTES);
        assert_eq!(&chain[16..], &[6, 12, 18, 24]);
    }

    #[test]
    fn page_images_are_linear_repeating_and_non_srgb() {
        let image = page_image(vec![0; 4 * 4 * TEXEL_BYTES], 4, 4, false);
        assert_eq!(image.texture_descriptor.format, TextureFormat::Rgba8Unorm);
        assert_eq!(image.texture_descriptor.mip_level_count, 1);
        let ImageSampler::Descriptor(sampler) = &image.sampler else {
            panic!("pages carry their own sampler");
        };
        assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
        assert_eq!(sampler.address_mode_v, ImageAddressMode::Repeat);
        assert_eq!(sampler.min_filter, ImageFilterMode::Linear);
        assert_eq!(sampler.mag_filter, ImageFilterMode::Linear);

        let mipmapped = page_image(vec![0; 4 * 4 * TEXEL_BYTES], 4, 4, true);
        assert_eq!(mipmapped.texture_descriptor.mip_level_count, 3);
        let data = mipmapped.data.expect("pages keep their texels");
        assert_eq!(data.len(), (16 + 4 + 1) * TEXEL_BYTES);
    }

    #[test]
    fn every_atlas_image_decodes_at_its_slot_size() {
        let decoded = decode_texture_image("TextureAtlas0").expect("TextureAtlas0 is embedded");
        assert_eq!((decoded.width, decoded.height), (1024, 1024));
        assert_eq!(decoded.pixels.len(), 1024 * 1024 * TEXEL_BYTES);
    }
}
