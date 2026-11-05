// Narrow-range filter: smooth the depth across the surface but keep silhouette edges.
@group(0) @binding(0) var depthTex : texture_2d<f32>;
@group(0) @binding(1) var samp : sampler;
struct U { texel : vec2<f32>, params : vec2<f32>, };  // params: radius(px), depthThreshold
@group(0) @binding(2) var<uniform> U0 : U;
@fragment
fn main(@location(0) uv : vec2<f32>) -> @location(0) f32 {
  let c = textureSample(depthTex, samp, uv).r;
  if (c >= 1.0) { return c; }
  var sum = 0.0; var wsum = 0.0;
  let r = i32(U0.params.x);
  for (var dy = -r; dy <= r; dy++) {
    for (var dx = -r; dx <= r; dx++) {
      let o = vec2<f32>(f32(dx), f32(dy)) * U0.texel;
      let s = textureSample(depthTex, samp, uv + o).r;
      let rr = f32(dx*dx + dy*dy);
      let ws = exp(-rr / max(U0.params.x*U0.params.x, 1.0));
      let wd = exp(-abs(s - c) / max(U0.params.y, 1e-4));
      let wgt = ws * wd;
      sum = sum + s * wgt; wsum = wsum + wgt;
    }
  }
  return select(c, sum / wsum, wsum > 0.0);
}
