@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

struct VertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) barycentric: vec3<f32>,
};

@vertex
fn vs_main(
    in: VertexIn,
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOut {
    var out: VertexOut;
    out.clip_pos = vec4<f32>(in.pos, 0.0, 1.0);
    out.color = in.color;
    out.uv = in.uv;

    // TODO temp
    let index = in_vertex_index % 3u;
    if (index == 0u) {
        out.barycentric = vec3<f32>(1.0, 0.0, 0.0);
    } else if (index == 1u) {
        out.barycentric = vec3<f32>(0.0, 1.0, 0.0);
    } else {
        out.barycentric = vec3<f32>(0.0, 0.0, 1.0);
    }

    return out;
}

// TODO conditional texture vs color?

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {

    let f = fwidth(in.barycentric);
    let edge = smoothstep(vec3<f32>(0.0), f * 2.0, in.barycentric);
    let is_edge = 1.0 - min(min(edge.x, edge.y), edge.z);

    if (is_edge > 0.5) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    return textureSample(texture, texture_sampler, in.uv); // TODO mix color with texture?
    //return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    //return vec4<f32>(in.color);
}
