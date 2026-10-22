//! Emulation of the original's OpenGL ES 1.1 fixed-function state (`docs/re/rendering.md` §1.3, §3).
//!
//! Display objects draw through a [`GlContext`] exactly as the original drew through GL: a
//! modelview matrix stack, a current colour, blending and texturing switches (which leak from
//! one draw to the next, as in the original), a bound texture and a scissor. Vertices are
//! transformed and clipped on the CPU and appended to a [`DrawList`] in clip space.

use bevy_math::{Mat4, Vec3, Vec4};

use crate::draw::{BlendMode, DrawList, DrawVertex, TextureId};

/// A scissor rectangle in normalised device coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NdcRect {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// The GL state machine of one frame (and its persistent leaks across frames).
pub struct GlContext {
    pub list: DrawList,
    projection: Mat4,
    stack: Vec<Mat4>,
    color: [f32; 4],

    /// `FUN_10013f980`'s cache of the last colour passed in; starts at (0,0,0,0) while GL's
    /// real colour starts at (1,1,1,1), so an initial (0,0,0,0) is skipped.
    color_cache: [f32; 4],
    blend: bool,
    texture_2d: bool,
    bound_texture: Option<TextureId>,
    scissor: Option<NdcRect>,

    /// Game units per renderbuffer pixel (point-sprite sizes are in pixels).
    pub units_per_pixel: f32,
}

impl Default for GlContext {
    fn default() -> Self {
        Self {
            list: DrawList::default(),
            projection: Mat4::IDENTITY,
            stack: vec![Mat4::IDENTITY],
            color: [1.0, 1.0, 1.0, 1.0],
            color_cache: [0.0, 0.0, 0.0, 0.0],
            blend: true,
            texture_2d: true,
            bound_texture: None,
            scissor: None,
            units_per_pixel: 1.0,
        }
    }
}

impl GlContext {
    /// Starts a frame: clears the draw list and the modelview stack. Colour, blending and
    /// texturing state persist (the original never resets them between frames).
    pub fn begin_frame(&mut self, projection: Mat4, clear_color: [f32; 4]) {
        self.list.clear();
        self.list.clear_color = clear_color;
        self.projection = projection;
        self.stack.clear();
        self.stack.push(Mat4::IDENTITY);
        self.scissor = None;
    }

    /// `BCView::setFrustumHorizontalShift:verticalShift:` projection with shifts 0.5:
    /// `Rz(+90°) · glOrthof(-H/2, H/2, -W/2, W/2, 0, 5000)` for a logical screen `W × H`.
    pub fn view_projection(screen_width: f32, screen_height: f32) -> Mat4 {
        let rotation = Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let half_height = screen_height * 0.5;
        let half_width = screen_width * 0.5;
        let ortho = gl_ortho(-half_height, half_height, -half_width, half_width, 0.0, 5000.0);
        rotation * ortho
    }

    fn top(&self) -> Mat4 {
        *self.stack.last().expect("the modelview stack is never empty")
    }

    fn set_top(&mut self, matrix: Mat4) {
        *self.stack.last_mut().expect("the modelview stack is never empty") = matrix;
    }

    /// `glLoadIdentity`.
    pub fn load_identity(&mut self) {
        self.set_top(Mat4::IDENTITY);
    }

    /// `glPushMatrix`.
    pub fn push_matrix(&mut self) {
        let top = self.top();
        self.stack.push(top);
    }

    /// `glPopMatrix`.
    pub fn pop_matrix(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// `glMultMatrixf`.
    pub fn mult_matrix(&mut self, matrix: Mat4) {
        let product = self.top() * matrix;
        self.set_top(product);
    }

    /// `FUN_10013faa4`: `glTranslatef` unless all zero.
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        if x == 0.0 && y == 0.0 && z == 0.0 {
            return;
        }
        let translation = Mat4::from_translation(Vec3::new(x, y, z));
        self.mult_matrix(translation);
    }

    /// `FUN_10013fac4`: `glRotatef(rx,1,0,0)`, then y, then z, each skipped when zero (degrees).
    pub fn rotate(&mut self, rx: f32, ry: f32, rz: f32) {
        if rx != 0.0 {
            self.mult_matrix(Mat4::from_rotation_x(rx.to_radians()));
        }
        if ry != 0.0 {
            self.mult_matrix(Mat4::from_rotation_y(ry.to_radians()));
        }
        if rz != 0.0 {
            self.mult_matrix(Mat4::from_rotation_z(rz.to_radians()));
        }
    }

    /// `FUN_10013fb3c`: `glScalef` unless all three are exactly 1.
    pub fn scale(&mut self, sx: f32, sy: f32, sz: f32) {
        if sx == 1.0 && sy == 1.0 && sz == 1.0 {
            return;
        }
        let scaling = Mat4::from_scale(Vec3::new(sx, sy, sz));
        self.mult_matrix(scaling);
    }

    /// `FUN_10013fb58`: skew (degrees); no-op when both are zero.
    pub fn skew(&mut self, kx: f32, ky: f32) {
        if kx == 0.0 && ky == 0.0 {
            return;
        }
        let a = kx * std::f32::consts::PI / 180.0;
        let b = ky * std::f32::consts::PI / 180.0;
        let column0 = Vec4::new(b.cos(), b.sin(), 0.0, 0.0);
        let column1 = Vec4::new(-a.sin(), a.cos(), 0.0, 0.0);
        let skew = Mat4::from_cols(column0, column1, Vec4::Z, Vec4::W);
        self.mult_matrix(skew);
    }

    /// `FUN_10013f980`: `glColor4f` unless equal to the last value passed here (NaN is never equal).
    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        let color = [r, g, b, a];
        if color == self.color_cache {
            return;
        }
        self.color_cache = color;
        self.color = color;
    }

    pub fn color(&self) -> [f32; 4] {
        self.color
    }

    /// `FUN_10013fc58`: blending on/off.
    pub fn set_blend(&mut self, enabled: bool) {
        self.blend = enabled;
    }

    pub fn blend(&self) -> bool {
        self.blend
    }

    /// `glEnable(GL_TEXTURE_2D)` / `glDisable`.
    pub fn set_texture_2d(&mut self, enabled: bool) {
        self.texture_2d = enabled;
    }

    /// `FUN_10013f9e8`: `glBindTexture`.
    pub fn bind_texture(&mut self, page: Option<TextureId>) {
        self.bound_texture = page;
    }

    /// Scissor push (`FUN_10013fc94`, one level is all the original uses).
    pub fn set_scissor(&mut self, rect: Option<NdcRect>) {
        self.scissor = rect;
    }

    fn current_texture(&self) -> Option<TextureId> {
        if self.texture_2d { self.bound_texture } else { None }
    }

    fn current_blend_mode(&self) -> BlendMode {
        if self.blend { BlendMode::Premultiplied } else { BlendMode::Opaque }
    }

    fn to_clip(&self, position: [f32; 3]) -> Vec4 {
        let model_view = self.top();
        let eye = model_view * Vec4::new(position[0], position[1], position[2], 1.0);
        self.projection * eye
    }

    fn clamp_color(color: [f32; 4]) -> [f32; 4] {
        color.map(|channel| if channel.is_nan() { 0.0 } else { channel.clamp(0.0, 1.0) })
    }

    /// `glDrawElements(GL_TRIANGLE_STRIP, …)` with the current colour (`uvs` ignored when
    /// texturing is off).
    pub fn draw_strip(&mut self, vertices: &[[f32; 3]], uvs: &[[f32; 2]]) {
        let color = Self::clamp_color(self.color);
        let mut strip = Vec::with_capacity(vertices.len());
        for (index, position) in vertices.iter().enumerate() {
            let uv = uvs.get(index).copied().unwrap_or([0.0, 0.0]);
            strip.push((self.to_clip(*position), uv, color));
        }
        self.emit_strip(&strip);
    }

    /// Triangle strip with a per-vertex colour array (`GL_COLOR_ARRAY` enabled).
    pub fn draw_colored_strip(&mut self, vertices: &[[f32; 3]], uvs: &[[f32; 2]], colors: &[[f32; 4]]) {
        let mut strip = Vec::with_capacity(vertices.len());
        for (index, position) in vertices.iter().enumerate() {
            let uv = uvs.get(index).copied().unwrap_or([0.0, 0.0]);
            let color = colors.get(index).copied().unwrap_or(self.color);
            strip.push((self.to_clip(*position), uv, Self::clamp_color(color)));
        }
        self.emit_strip(&strip);
    }

    /// `glDrawArrays(GL_POINTS)` with point sprites (`GL_COORD_REPLACE_OES`): each point is a
    /// screen-aligned square of `size` renderbuffer pixels showing the whole bound texture.
    pub fn draw_point_sprites(&mut self, points: &[[f32; 2]], size: f32) {
        let color = Self::clamp_color(self.color);
        let half = size * self.units_per_pixel * 0.5;
        for point in points {
            let center = self.to_clip([point[0], point[1], 0.0]);
            let corners = self.point_sprite_corners(center, half);
            let strip = [
                (corners[0], [0.0, 0.0], color),
                (corners[1], [1.0, 0.0], color),
                (corners[2], [0.0, 1.0], color),
                (corners[3], [1.0, 1.0], color),
            ];
            self.emit_strip(&strip);
        }
    }

    fn point_sprite_corners(&self, center: Vec4, half_units: f32) -> [Vec4; 4] {
        // Offsets in clip space for `half_units` game units along screen x and y.
        let origin = self.projection * Vec4::new(0.0, 0.0, 0.0, 1.0);
        let along_x = self.projection * Vec4::new(0.0, -half_units, 0.0, 1.0) - origin;
        let along_y = self.projection * Vec4::new(half_units, 0.0, 0.0, 1.0) - origin;
        let dx = along_x * center.w;
        let dy = along_y * center.w;
        [center - dx + dy, center + dx + dy, center - dx - dy, center + dx - dy]
    }

    fn emit_strip(&mut self, strip: &[(Vec4, [f32; 2], [f32; 4])]) {
        if strip.len() < 3 {
            return;
        }
        let texture = self.current_texture();
        let blend = self.current_blend_mode();
        let mut triangles: Vec<DrawVertex> = Vec::new();
        for index in 0..strip.len() - 2 {
            let (a, b, c) = if index % 2 == 0 {
                (strip[index], strip[index + 1], strip[index + 2])
            } else {
                (strip[index + 1], strip[index], strip[index + 2])
            };
            self.clip_triangle([a, b, c], &mut triangles);
        }
        if !triangles.is_empty() {
            self.list.push_triangles(texture, blend, &triangles);
        }
    }

    /// Clips one triangle against near/far (GL clip z) and the scissor, appending the result.
    fn clip_triangle(&self, triangle: [(Vec4, [f32; 2], [f32; 4]); 3], out: &mut Vec<DrawVertex>) {
        let mut polygon: Vec<(Vec4, [f32; 2], [f32; 4])> = triangle.to_vec();
        let mut planes: Vec<Vec4> = vec![Vec4::new(0.0, 0.0, 1.0, 1.0), Vec4::new(0.0, 0.0, -1.0, 1.0)];
        if let Some(rect) = self.scissor {
            planes.push(Vec4::new(1.0, 0.0, 0.0, -rect.min_x));
            planes.push(Vec4::new(-1.0, 0.0, 0.0, rect.max_x));
            planes.push(Vec4::new(0.0, 1.0, 0.0, -rect.min_y));
            planes.push(Vec4::new(0.0, -1.0, 0.0, rect.max_y));
        }

        for plane in planes {
            polygon = clip_polygon(&polygon, plane);
            if polygon.len() < 3 {
                return;
            }
        }

        for index in 1..polygon.len() - 1 {
            for vertex in [polygon[0], polygon[index], polygon[index + 1]] {
                let (position, uv, color) = vertex;
                if !position.is_finite() {
                    return;
                }
                out.push(DrawVertex {
                    position: position.to_array(),
                    uv,
                    color,
                });
            }
        }
    }
}

/// `glOrthof(left, right, bottom, top, near, far)`.
fn gl_ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    let column0 = Vec4::new(2.0 / (right - left), 0.0, 0.0, 0.0);
    let column1 = Vec4::new(0.0, 2.0 / (top - bottom), 0.0, 0.0);
    let column2 = Vec4::new(0.0, 0.0, -2.0 / (far - near), 0.0);
    let translation_x = -(right + left) / (right - left);
    let translation_y = -(top + bottom) / (top - bottom);
    let translation_z = -(far + near) / (far - near);
    let column3 = Vec4::new(translation_x, translation_y, translation_z, 1.0);
    Mat4::from_cols(column0, column1, column2, column3)
}

type ClipVertex = (Vec4, [f32; 2], [f32; 4]);

fn lerp_vertex(a: ClipVertex, b: ClipVertex, t: f32) -> ClipVertex {
    let position = a.0 + (b.0 - a.0) * t;
    let uv = [a.1[0] + (b.1[0] - a.1[0]) * t, a.1[1] + (b.1[1] - a.1[1]) * t];
    let mut color = [0.0; 4];
    for channel in 0..4 {
        color[channel] = a.2[channel] + (b.2[channel] - a.2[channel]) * t;
    }
    (position, uv, color)
}

/// Sutherland–Hodgman against the half-space `dot(plane, position) >= 0`.
fn clip_polygon(polygon: &[ClipVertex], plane: Vec4) -> Vec<ClipVertex> {
    let mut result = Vec::with_capacity(polygon.len() + 2);
    for index in 0..polygon.len() {
        let current = polygon[index];
        let next = polygon[(index + 1) % polygon.len()];
        let current_distance = plane.dot(current.0);
        let next_distance = plane.dot(next.0);
        if current_distance >= 0.0 {
            result.push(current);
        }
        let crosses = (current_distance >= 0.0) != (next_distance >= 0.0);
        if crosses {
            let t = current_distance / (current_distance - next_distance);
            result.push(lerp_vertex(current, next, t));
        }
    }
    result
}
