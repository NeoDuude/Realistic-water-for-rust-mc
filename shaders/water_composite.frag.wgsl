// Shade the water: rebuild normal from smoothed depth, refract the scene behind it,
// darken by thickness (Beer-Lambert), add a fresnel sky reflection and a sun highlight.
@group(0) @binding(0) var depthTex : texture_2d<f32>;    // smoothed surface depth 0..1
@group(0) @binding(1) var thickTex : texture_2d<f32>;    // accumulated thickness
@group(0) @binding(2) var sceneTex : texture_2d<f32>;    // world colour behind the water
@group(0) @binding(3) var samp : sampler;
struct U {
  invProj  : mat4x4<f32>,
  texel    : vec4<f32>,   // 1/w, 1/h, _, _
  sunDir   : vec4<f32>,   // view-space sun
  absorb   : vec4<f32>,   // per-channel absorption
  skyColor : vec4<f32>,
  misc     : vec4<f32>,   // refraction strength, ior, specular, _
};
@group(0) @binding(4) var<uniform> U0 : U;

fn viewPos(uv : vec2<f32>, d : f32) -> vec3<f32> {
  let ndc = vec4<f32>(uv*2.0 - vec2<f32>(1.0,1.0), d, 1.0);
  let v = U0.invProj * ndc;
  return v.xyz / v.w;
}
@fragment
fn main(@location(0) uv : vec2<f32>) -> @location(0) vec4<f32> {
  let d = textureSample(depthTex, samp, uv).r;
  let scene = textureSample(sceneTex, samp, uv);
  if (d >= 1.0) { return scene; }   // no water here
  let p = viewPos(uv, d);
  // normal from depth derivatives (screen-space)
  let dx = viewPos(uv + vec2<f32>(U0.texel.x, 0.0), textureSample(depthTex, samp, uv + vec2<f32>(U0.texel.x,0.0)).r) - p;
  let dy = viewPos(uv + vec2<f32>(0.0, U0.texel.y), textureSample(depthTex, samp, uv + vec2<f32>(0.0,U0.texel.y)).r) - p;
  let n = normalize(cross(dx, dy));
  let thick = textureSample(thickTex, samp, uv).r;
  // refract the scene behind, offset by normal and thickness
  let off = n.xy * U0.misc.x * clamp(thick, 0.0, 1.0);
  let refr = textureSample(sceneTex, samp, uv + off).rgb;
  // Beer-Lambert: thicker water keeps less of the background colour
  let trans = exp(-U0.absorb.rgb * thick);
  var col = refr * trans;
  // fresnel toward the sky
  let viewDir = normalize(-p);
  let f0 = 0.02;
  let fres = f0 + (1.0 - f0) * pow(1.0 - max(dot(n, viewDir), 0.0), 5.0);
  col = mix(col, U0.skyColor.rgb, fres);
  // sun specular
  let h = normalize(U0.sunDir.xyz + viewDir);
  let spec = pow(max(dot(n, h), 0.0), 120.0) * U0.misc.z;
  col = col + vec3<f32>(spec);
  return vec4<f32>(col, 1.0);
}
