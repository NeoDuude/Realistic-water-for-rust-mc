// Real Water: rolling water. Every fluid corner moves by the wave height at its
// world x/z, so corners shared by a top and a side face move together (no cracks).
// Waves fade out far away, where they would only shimmer.

fn vertex(input: VertexInput) -> vec3<f32> {
    if (input.fluid == 0u) {
        return input.position;
    }
    let world = input.position + frame.camera_position;
    let fade = 1.0 - smoothstep(48.0, 96.0, length(input.position.xz));
    let lift = rw_height(world.xz) * fade;
    return input.position + vec3<f32>(0.0, lift, 0.0);
}
