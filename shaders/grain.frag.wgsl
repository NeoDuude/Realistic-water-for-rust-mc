struct FsIn { @location(0) off:vec2<f32>, @location(1) shade:f32, };
@fragment
fn main(in:FsIn) -> @location(0) vec4<f32> {
  if (dot(in.off,in.off) > 1.0) { discard; }
  let sand = vec3<f32>(0.76, 0.70, 0.50) * in.shade;
  return vec4<f32>(sand, 1.0);
}
