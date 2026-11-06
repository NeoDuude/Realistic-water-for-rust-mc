// Real Water: shared code, put before every stage.
// Original work (MIT). Waves are a few travelling sines plus value noise,
// evaluated on world x/z so neighbouring blocks always agree.

override wave_height: f32 = 0.07;
override wave_speed: f32 = 1.0;
override reflections: bool = true;
override refraction: f32 = 0.5;
override clarity: f32 = 0.6;
override foam: f32 = 0.7;
override sun_glint: f32 = 1.0;
override caustics: f32 = 0.6;
override shadows: bool = true;
override shadow_strength: f32 = 0.45;
override underwater_fog: f32 = 1.0;
override saturation: f32 = 1.08;
override contrast: f32 = 1.05;

fn rw_hash(p: vec2<f32>) -> f32 {
    var q = fract(p * vec2<f32>(0.1031, 0.1030));
    q += dot(q, q.yx + 33.33);
    return fract((q.x + q.y) * q.x);
}

// Smooth value noise, 0..1.
fn rw_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = rw_hash(i);
    let b = rw_hash(i + vec2<f32>(1.0, 0.0));
    let c = rw_hash(i + vec2<f32>(0.0, 1.0));
    let d = rw_hash(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// The long swell, about -1..1.
fn rw_swell(xz: vec2<f32>, t: f32) -> f32 {
    var h = sin(dot(xz, vec2<f32>(0.62, 0.78)) * 0.9 + t * 1.3) * 0.40;
    h += sin(dot(xz, vec2<f32>(-0.85, 0.53)) * 1.7 + t * 1.9) * 0.25;
    h += sin(dot(xz, vec2<f32>(0.21, -0.98)) * 2.9 + t * 2.6) * 0.15;
    h += (rw_noise(xz * 0.8 + vec2<f32>(t * 0.35, t * 0.25)) - 0.5) * 0.4;
    return h;
}

// Small chop for the surface normal only, about -0.5..0.5. Rain makes it busier.
fn rw_chop(xz: vec2<f32>, t: f32) -> f32 {
    var h = rw_noise(xz * 3.1 + vec2<f32>(t * 0.9, -t * 0.7)) - 0.5;
    h += (rw_noise(xz * 6.3 - vec2<f32>(t * 1.4, t * 1.1)) - 0.5) * 0.5;
    h += (rw_noise(xz * 13.0 + vec2<f32>(t * 3.0, t * 2.3)) - 0.5) * 0.5 * frame.wetness;
    return h;
}

// Where the surface sits above its block height, in blocks.
fn rw_height(xz: vec2<f32>) -> f32 {
    return rw_swell(xz, frame.time * wave_speed) * wave_height;
}

// The shading surface: swell plus chop, for the normal.
fn rw_shape(xz: vec2<f32>) -> f32 {
    let t = frame.time * wave_speed;
    return rw_swell(xz, t) * (0.10 + wave_height) + rw_chop(xz, t) * 0.035;
}

// World-axis normal of the wavy surface at world x/z.
fn rw_normal(xz: vec2<f32>) -> vec3<f32> {
    let e = 0.04;
    let hx = rw_shape(xz + vec2<f32>(e, 0.0)) - rw_shape(xz - vec2<f32>(e, 0.0));
    let hz = rw_shape(xz + vec2<f32>(0.0, e)) - rw_shape(xz - vec2<f32>(0.0, e));
    return normalize(vec3<f32>(-hx / (2.0 * e), 1.0, -hz / (2.0 * e)));
}
