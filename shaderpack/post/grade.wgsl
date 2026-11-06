// Real Water: a gentle colour grade, a soft filmic shoulder so bright glints
// roll off instead of clipping.

fn rw_luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn post(input: PostInput) -> vec4<f32> {
    var c = max(input.color.rgb, vec3<f32>(0.0));
    // Soft shoulder above 0.8.
    let over = max(c - vec3<f32>(0.8), vec3<f32>(0.0));
    c = min(c, vec3<f32>(0.8)) + over / (vec3<f32>(1.0) + over * 2.5);
    // Contrast around mid grey, then saturation.
    c = (c - vec3<f32>(0.45)) * contrast + vec3<f32>(0.45);
    let y = rw_luma(c);
    c = mix(vec3<f32>(y), c, saturation);
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), input.color.a);
}
