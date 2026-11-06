// Real Water: the look of every terrain surface.
// Water: wave normals, Fresnel mix of a real reflection and a refracted view of
// what lies under it, Beer-Lambert absorption by depth, foam on falls, flows and
// shores, sun glint. Land: sun shadows, caustics under water, wet ground in rain.

const RW_FOAM: vec3<f32> = vec3<f32>(0.86, 0.91, 0.93);

// Distance from the camera, in blocks, of the point at screen uv and reversed depth.
fn rw_distance(screen: vec2<f32>, depth: f32) -> f32 {
    if (depth <= 0.0) {
        return 1.0e6;
    }
    let ndc = vec4<f32>(screen.x * 2.0 - 1.0, 1.0 - screen.y * 2.0, depth, 1.0);
    let v = frame.projection_inverse * ndc;
    return length(v.xyz / v.w);
}

fn rw_texel(screen: vec2<f32>) -> vec2<i32> {
    let size = frame.resolution;
    return vec2<i32>(clamp(screen * size, vec2<f32>(0.0), size - vec2<f32>(1.0)));
}

// 1 where the sun reaches `position`, 0 in shadow; 1 without a shadow map.
fn rw_sunlit(position: vec3<f32>, normal: vec3<f32>) -> f32 {
    if (!shadows || frame.has_shadow == 0u) {
        return 1.0;
    }
    let p = shadow_position(position + normal * 0.08);
    if (any(p.xy <= vec2<f32>(0.0)) || any(p.xy >= vec2<f32>(1.0)) || p.z >= 1.0) {
        return 1.0;
    }
    return textureSampleCompareLevel(shadow_map, shadow_sampler, p.xy, p.z - 0.0008);
}

// 1 where the light reaching `position` came through water first.
fn rw_under_water(position: vec3<f32>) -> f32 {
    if (frame.has_shadow == 0u) {
        return 0.0;
    }
    let p = shadow_position(position + vec3<f32>(0.0, 0.05, 0.0));
    if (any(p.xy <= vec2<f32>(0.0)) || any(p.xy >= vec2<f32>(1.0)) || p.z >= 1.0) {
        return 0.0;
    }
    let through = textureSampleCompareLevel(shadow_map_translucent, shadow_sampler, p.xy, p.z - 0.0008);
    if (through > 0.5) {
        return 0.0;
    }
    let dims = vec2<f32>(textureDimensions(shadow_color));
    let texel = vec2<i32>(clamp(p.xy * dims, vec2<f32>(0.0), dims - vec2<f32>(1.0)));
    return select(0.0, 1.0, textureLoad(shadow_color, texel, 0).a > 0.995);
}

// A web of bright lines, like sunlight focused by a wavy surface.
fn rw_caustic(xz: vec2<f32>) -> f32 {
    let t = frame.time * wave_speed;
    let a = rw_noise(xz * 1.6 + vec2<f32>(t * 0.45, t * 0.3));
    let b = rw_noise(xz * 1.6 * 1.37 + vec2<f32>(-t * 0.35, t * 0.5) + vec2<f32>(7.3, 2.1));
    let line = 1.0 - clamp(abs(a - b) * 3.0, 0.0, 1.0);
    return pow(line, 5.0);
}

// How strongly the sun lights things now, 0 at night.
fn rw_day() -> f32 {
    return smoothstep(-0.03, 0.18, frame.sun_direction.y);
}

fn rw_sun_color() -> vec3<f32> {
    return mix(vec3<f32>(1.0, 0.52, 0.28), vec3<f32>(1.0, 0.96, 0.88), smoothstep(0.0, 0.35, frame.sun_direction.y));
}

fn rw_land(input: MaterialInput) -> Surface {
    var color = input.albedo * input.tint * vec4<f32>(input.lightmap, 1.0);
    let sky = input.light.y;
    let reach = sky * rw_day() * (1.0 - input.light.x * 0.7);
    if (reach > 0.01) {
        let ndl = dot(input.normal, frame.light_direction);
        let lit = rw_sunlit(input.position, input.normal) * smoothstep(-0.02, 0.15, ndl);
        let shade = 1.0 - shadow_strength * reach * (1.0 - lit);
        let warm = mix(vec3<f32>(1.0), rw_sun_color(), 0.25 * lit * reach);
        color = vec4<f32>(color.rgb * shade * warm * (1.0 + 0.12 * lit * reach), color.a);

        // Sunlight that came through water ripples over the ground below.
        if (caustics > 0.0 && lit > 0.3) {
            let wet = rw_under_water(input.position);
            if (wet > 0.0) {
                let world = input.position + frame.camera_position;
                let slant = frame.light_direction.xz / max(frame.light_direction.y, 0.2);
                let c = rw_caustic(world.xz + slant * world.y * 0.15);
                color = vec4<f32>(color.rgb * (0.82 + caustics * 1.4 * c * reach), color.a);
            }
        }
    }
    // Rain darkens open ground a little, as wet things are.
    if (frame.wetness > 0.0 && input.normal.y > 0.9 && !input.translucent) {
        color = vec4<f32>(color.rgb * (1.0 - 0.22 * frame.wetness * smoothstep(0.85, 1.0, sky)), color.a);
    }
    return Surface(color);
}

fn rw_water(input: MaterialInput) -> Surface {
    let world = input.position + frame.camera_position;
    let dist = length(input.position);
    let view = normalize(-input.position);
    let top = input.face == 1u;
    let sky = input.light.y;
    let light = input.lightmap;
    // The biome's water colour lives in the tint.
    let tint = input.tint.rgb;

    // Surface normal: waves on top faces, the face itself elsewhere.
    var n = input.normal;
    if (top) {
        let flat_far = clamp(dist / 110.0, 0.0, 0.9);
        n = normalize(mix(rw_normal(world.xz), vec3<f32>(0.0, 1.0, 0.0), flat_far));
    }
    if (dot(n, view) < 0.0) {
        n = -n;
    }
    let ndv = clamp(dot(n, view), 0.0, 1.0);
    let below = frame.eye_in == 1u;

    // What is behind the surface, bent by the waves. An offset that lands on
    // something in front of the water is thrown away.
    var ruv = clamp(input.screen + n.xz * refraction * 0.045 / max(1.0, dist * 0.05), vec2<f32>(0.0), vec2<f32>(1.0));
    var od = textureLoad(opaque_depth, rw_texel(ruv), 0);
    var behind = rw_distance(ruv, od);
    if (behind < dist) {
        ruv = input.screen;
        od = textureLoad(opaque_depth, rw_texel(ruv), 0);
        behind = rw_distance(ruv, od);
    }
    let refracted = textureLoad(opaque_color, rw_texel(ruv), 0).rgb;
    let thickness = clamp(behind - dist, 0.0, 256.0);

    // Beer-Lambert: red goes first, then green, so deep water turns blue.
    let absorb = vec3<f32>(0.32, 0.085, 0.05) * mix(3.0, 0.35, clarity);
    let keep = exp(-absorb * thickness);
    let scatter = tint * vec3<f32>(0.11, 0.24, 0.28) * light;
    var under = refracted * keep + scatter * (1.0 - keep);

    if (below) {
        // From under the water: the surface lets the sky through near straight up,
        // and mirrors the depths at a slant.
        let window = smoothstep(0.25, 0.7, ndv);
        let deep = scatter * 1.3;
        return Surface(vec4<f32>(mix(deep, refracted * vec3<f32>(0.85, 0.95, 1.0), window), 1.0));
    }

    // What the surface mirrors.
    let r = reflect(-view, n);
    var mirrored = mix(frame.fog_color, frame.sky_color, clamp(r.y * 2.0, 0.0, 1.0)) * (sky * sky);
    if (reflections && top && frame.has_reflection == 1u && abs(world.y - frame.water_plane) < 1.25) {
        let muv = clamp(input.screen + n.xz * 0.035, vec2<f32>(0.0), vec2<f32>(1.0));
        mirrored = textureSampleLevel(reflection, reflection_sampler, muv, 0.0).rgb;
    }

    let fresnel = 0.02 + 0.98 * pow(1.0 - ndv, 5.0);
    var color = mix(under, mirrored, fresnel);

    // Sun glint, only where the sun really reaches the water.
    let sun = frame.sun_direction;
    if (sun_glint > 0.0 && sun.y > -0.05 && sky > 0.2) {
        let h = normalize(view + sun);
        let spec = pow(max(dot(n, h), 0.0), 380.0) * 7.0 * sun_glint * rw_day() * sky;
        if (spec > 0.001) {
            color += rw_sun_color() * spec * rw_sunlit(input.position, vec3<f32>(0.0, 1.0, 0.0));
        }
    }
    // A faint moon path at night.
    let moon = frame.moon_direction;
    if (sun_glint > 0.0 && moon.y > 0.0 && sky > 0.2) {
        let hm = normalize(view + moon);
        color += vec3<f32>(0.55, 0.62, 0.8) * pow(max(dot(n, hm), 0.0), 500.0) * 1.5 * sun_glint * (1.0 - rw_day());
    }

    // Foam.
    if (foam > 0.0) {
        var white = 0.0;
        let t = frame.time;
        if (input.face >= 2u) {
            // Falling water: streaks running down the face.
            let across = select(world.z, world.x, input.face == 2u || input.face == 3u);
            let s = rw_noise(vec2<f32>(across * 4.0, (world.y + t * 3.2) * 0.7));
            let s2 = rw_noise(vec2<f32>(across * 9.0 + 3.0, (world.y + t * 4.1) * 1.5));
            white = smoothstep(0.42, 0.85, s * 0.65 + s2 * 0.45) * 0.85;
        } else if (top) {
            // Fast, sloped flows churn.
            let slope = 1.0 - abs(input.normal.y);
            let churn = rw_noise(world.xz * 3.0 + vec2<f32>(t * 1.5, t * 1.2));
            white = smoothstep(0.04, 0.35, slope) * smoothstep(0.35, 0.8, churn);
            // Thin water at the shore breaks into foam.
            let depth_here = thickness * ndv;
            let edge = 1.0 - smoothstep(0.03, 0.3, depth_here);
            let lace = rw_noise(world.xz * 5.0 + vec2<f32>(sin(t * 0.8) * 1.5, t * 0.4));
            white = max(white, edge * smoothstep(0.3, 0.75, lace) * 0.8);
        }
        white = clamp(white * foam, 0.0, 1.0) * (1.0 - smoothstep(64.0, 128.0, dist));
        color = mix(color, RW_FOAM * light, white);
    }

    return Surface(vec4<f32>(color, 1.0));
}

fn material(input: MaterialInput) -> Surface {
    if (input.fluid == 1u) {
        return rw_water(input);
    }
    if (input.fluid == 2u) {
        // Lava: a slow, uneven glow.
        let world = input.position + frame.camera_position;
        let pulse = 0.9 + 0.25 * rw_noise(world.xz * 0.7 + vec2<f32>(frame.time * 0.15, -frame.time * 0.1));
        let c = input.albedo * input.tint * vec4<f32>(input.lightmap, 1.0);
        return Surface(vec4<f32>(c.rgb * pulse, c.a));
    }
    return rw_land(input);
}
