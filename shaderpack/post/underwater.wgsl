// Real Water: under the surface the picture wobbles and fades into blue-green
// with distance, red first.

fn post(input: PostInput) -> vec4<f32> {
    let wet = frame.in_water;
    if (wet < 0.01 || underwater_fog <= 0.0) {
        return input.color;
    }
    let t = frame.time;
    let wobble = vec2<f32>(
        sin(input.uv.y * 38.0 + t * 2.1),
        cos(input.uv.x * 31.0 + t * 1.7)
    ) * 0.0025 * wet;
    let uv = clamp(input.uv + wobble, vec2<f32>(0.0), vec2<f32>(1.0));
    let seen = textureSampleLevel(previous, linear_sampler, uv, 0.0);
    let depth = depth_at(uv);
    var dist = 200.0;
    if (depth > 0.0) {
        dist = length(view_position(uv, depth));
    }
    let light = max(frame.eye_light.y, frame.eye_light.x * 0.6) * mix(0.25, 1.0, smoothstep(-0.05, 0.2, frame.sun_direction.y));
    let water = vec3<f32>(0.05, 0.22, 0.30) * (0.25 + 0.75 * light);
    let absorb = vec3<f32>(0.11, 0.045, 0.032) * underwater_fog;
    let keep = exp(-absorb * dist);
    let color = seen.rgb * keep + water * (1.0 - keep);
    return vec4<f32>(mix(input.color.rgb, color, wet), input.color.a);
}
