// Billboard each fluid particle as a camera-facing quad. Instanced: 6 verts per particle.
struct Particle { pos : vec4<f32>, vel : vec4<f32>, };
struct Cam {
  viewProj : mat4x4<f32>,
  view     : mat4x4<f32>,
  right    : vec4<f32>,
  up       : vec4<f32>,
  origin   : vec4<f32>,   // domain origin in camera-relative world space
  misc     : vec4<f32>,   // particleRadius, _, _, _
};
@group(0) @binding(0) var<uniform> C : Cam;
@group(0) @binding(1) var<storage, read> parts : array<Particle>;

struct VsOut {
  @builtin(position) clip : vec4<f32>,
  @location(0) offset : vec2<f32>,   // -1..1 across the sprite
  @location(1) viewPos : vec3<f32>,  // particle centre in view space
};
@vertex
fn main(@builtin(vertex_index) vid : u32, @builtin(instance_index) iid : u32) -> VsOut {
  var corners = array<vec2<f32>,6>(
    vec2<f32>(-1.0,-1.0), vec2<f32>(1.0,-1.0), vec2<f32>(1.0,1.0),
    vec2<f32>(-1.0,-1.0), vec2<f32>(1.0,1.0), vec2<f32>(-1.0,1.0));
  let o = corners[vid];
  let p = parts[iid];
  var out : VsOut;
  if (p.pos.w < 0.0) { out.clip = vec4<f32>(2.0,2.0,2.0,1.0); out.offset = o; out.viewPos = vec3<f32>(0.0); return out; }
  let r = C.misc.x;
  let world = C.origin.xyz + p.pos.xyz + (C.right.xyz*o.x + C.up.xyz*o.y)*r;
  out.clip = C.viewProj * vec4<f32>(world, 1.0);
  out.offset = o;
  out.viewPos = (C.view * vec4<f32>(C.origin.xyz + p.pos.xyz, 1.0)).xyz;
  return out;
}
