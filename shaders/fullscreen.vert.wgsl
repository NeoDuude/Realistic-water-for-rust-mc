struct VsOut { @builtin(position) clip : vec4<f32>, @location(0) uv : vec2<f32>, };
@vertex
fn main(@builtin(vertex_index) vid : u32) -> VsOut {
  var p = array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0), vec2<f32>(3.0,-1.0), vec2<f32>(-1.0,3.0));
  var out : VsOut;
  out.clip = vec4<f32>(p[vid], 0.0, 1.0);
  out.uv = p[vid]*0.5 + vec2<f32>(0.5,0.5);
  return out;
}
