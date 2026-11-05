struct Particle { pos : vec4<f32>, vel : vec4<f32>, };
struct Cam { viewProj:mat4x4<f32>, right:vec4<f32>, up:vec4<f32>, origin:vec4<f32>, misc:vec4<f32>, };
@group(0) @binding(0) var<uniform> C : Cam;
@group(0) @binding(1) var<storage, read> parts : array<Particle>;
struct VsOut { @builtin(position) clip:vec4<f32>, @location(0) off:vec2<f32>, @location(1) shade:f32, };
@vertex
fn main(@builtin(vertex_index) vid:u32, @builtin(instance_index) iid:u32) -> VsOut {
  var corners = array<vec2<f32>,6>(vec2<f32>(-1.0,-1.0),vec2<f32>(1.0,-1.0),vec2<f32>(1.0,1.0),vec2<f32>(-1.0,-1.0),vec2<f32>(1.0,1.0),vec2<f32>(-1.0,1.0));
  let o = corners[vid]; let p = parts[iid];
  var out : VsOut;
  if (p.pos.w < 0.0) { out.clip = vec4<f32>(2.0,2.0,2.0,1.0); out.off=o; out.shade=0.0; return out; }
  let world = C.origin.xyz + p.pos.xyz + (C.right.xyz*o.x + C.up.xyz*o.y)*C.misc.x;
  out.clip = C.viewProj * vec4<f32>(world,1.0);
  out.off = o;
  out.shade = clamp(0.5 + 0.5*p.pos.y*0.05, 0.3, 1.0);
  return out;
}
