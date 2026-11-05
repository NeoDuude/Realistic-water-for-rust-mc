// Sphere-imposter: shape the sprite into a round depth, add thickness.
struct Cam {
  viewProj : mat4x4<f32>,
  view     : mat4x4<f32>,
  right    : vec4<f32>,
  up       : vec4<f32>,
  origin   : vec4<f32>,
  misc     : vec4<f32>,  // particleRadius, nearThick, _, _
};
@group(0) @binding(0) var<uniform> C : Cam;
struct FsIn {
  @location(0) offset : vec2<f32>,
  @location(1) viewPos : vec3<f32>,
};
struct FsOut {
  @location(0) thickness : vec4<f32>,
  @builtin(frag_depth) depth : f32,
};
@fragment
fn main(in : FsIn) -> FsOut {
  let r2 = dot(in.offset, in.offset);
  if (r2 > 1.0) { discard; }
  var out : FsOut;
  // view-space z pushed toward camera by the sphere cap
  let zc = sqrt(1.0 - r2);
  // encode a 0..1 depth from view z (caller supplies a simple linear map via misc if needed)
  out.depth = clamp(0.5 - in.viewPos.z * 0.001 - zc * C.misc.x * 0.001, 0.0, 1.0);
  let thick = zc * C.misc.x * 2.0;
  out.thickness = vec4<f32>(thick, 0.0, 0.0, 1.0);
  return out;
}
