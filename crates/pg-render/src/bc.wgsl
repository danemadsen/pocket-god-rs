// Draws pg-engine triangles exactly like the original OpenGL ES 1.1 fixed pipeline:
// positions arrive already in clip space, the texture is modulated by the vertex colour,
// and blending happens in gamma space (CompositingSpace::Srgb + non-sRGB textures).

struct VertexInput {
    @location(0) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(2) @binding(0) var bc_texture: texture_2d<f32>;
@group(2) @binding(1) var bc_sampler: sampler;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // GL clip z is in [-w, w]; depth testing is disabled, so park every vertex mid-range.
    out.position = vec4<f32>(in.clip_position.xy, 0.5 * in.clip_position.w, in.clip_position.w);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(bc_texture, bc_sampler, in.uv);
    return texel * in.color;
}
