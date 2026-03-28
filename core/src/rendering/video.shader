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
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_pos = vec4<f32>(in.pos, 0.0, 1.0);
    out.color = in.color;
    out.uv = in.uv;
    return out;
}

// TODO conditional texture vs color?

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    //return vec4<f32>(abs(in.uv), 0.0, 1.0);
    return textureSample(texture, texture_sampler, in.uv);
    //return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    //return vec4<f32>(in.color);
}
