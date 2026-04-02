// --- Normals ---

fn calc_normal(p: vec3<f32>) -> vec3<f32> {
    let h = 0.0005;
    let k = vec2(1.0, -1.0);
    return normalize(
        k.xyy * sdf_scene(p + k.xyy * h) +
        k.yyx * sdf_scene(p + k.yyx * h) +
        k.yxy * sdf_scene(p + k.yxy * h) +
        k.xxx * sdf_scene(p + k.xxx * h)
    );
}

// --- Soft shadows ---

fn soft_shadow(ro: vec3<f32>, rd: vec3<f32>, mint: f32, maxt: f32, k_shadow: f32) -> f32 {
    var res = 1.0;
    var t = mint;
    for (var i = 0u; i < 64u; i++) {
        let h = sdf_scene(ro + rd * t);
        if h < EPSILON { return 0.0; }
        res = min(res, k_shadow * h / t);
        t += h;
        if t > maxt { break; }
    }
    return clamp(res, 0.0, 1.0);
}

// --- Ambient occlusion ---

fn calc_ao(pos: vec3<f32>, nor: vec3<f32>) -> f32 {
    var occ = 0.0;
    var sca = 1.0;
    for (var i = 0u; i < 5u; i++) {
        let h = 0.01 + 0.12 * f32(i) / 4.0;
        let d = sdf_scene(pos + h * nor);
        occ += (h - d) * sca;
        sca *= 0.95;
    }
    return clamp(1.0 - params.ao_intensity * occ, 0.0, 1.0);
}

// --- Ray marching ---

fn ray_march(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var t = 0.0;
    for (var i = 0u; i < MAX_STEPS; i++) {
        let p = ro + rd * t;
        let d = sdf_scene(p);
        if d < EPSILON { return t; }
        t += d;
        if t > MAX_DIST { break; }
    }
    return -1.0;
}

// --- PBR BRDF functions ---

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn fresnel_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return f0 + (max(vec3<f32>(1.0 - roughness, 1.0 - roughness, 1.0 - roughness), f0) - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn geometry_schlick_ggx(n_dot_x: f32, k: f32) -> f32 {
    return n_dot_x / (n_dot_x * (1.0 - k) + k);
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return geometry_schlick_ggx(n_dot_v, k) * geometry_schlick_ggx(n_dot_l, k);
}

// --- Fragment shader ---

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray_origin = view.world_position;
    let ray_dir = normalize(in.world_position.xyz - ray_origin);

    let t = ray_march(ray_origin, ray_dir);

    if t < 0.0 {
        discard;
    }

    let pos = ray_origin + ray_dir * t;
    let nor = calc_normal(pos);

    // Get blended material at hit point
    let mat = sdf_scene_material(pos);
    var albedo = mat.color;
    let roughness = mat.roughness;
    let metallic = mat.metallic;

    // Gradient-based coloring (mode 5): snow on upward faces, moss on sides
    if mat.color_mode == 5u {
        let snow = smoothstep(0.3, 0.7, nor.y);
        albedo = mix(albedo, vec3(0.95, 0.95, 0.98), snow);
    }

    // PBR lighting
    let ao = calc_ao(pos, nor);
    let V = -ray_dir;
    let n_dot_v = max(dot(nor, V), 0.0001);
    let f0 = mix(vec3<f32>(0.04, 0.04, 0.04), albedo, metallic);

    // --- Key light (sun) ---
    let key_dir = normalize(params.light_dir);
    let key_shadow = soft_shadow(pos + nor * 0.01, key_dir, 0.02, 20.0, params.shadow_softness);
    let key_H = normalize(V + key_dir);
    let key_n_dot_l = max(dot(nor, key_dir), 0.0);
    let key_n_dot_h = max(dot(nor, key_H), 0.0);
    let key_h_dot_v = max(dot(key_H, V), 0.0);

    let key_D = distribution_ggx(key_n_dot_h, roughness);
    let key_G = geometry_smith(n_dot_v, key_n_dot_l, roughness);
    let key_F = fresnel_schlick(key_h_dot_v, f0);
    let key_spec = (key_D * key_G * key_F) / (4.0 * n_dot_v * key_n_dot_l + 0.0001);
    let key_kD = (vec3<f32>(1.0, 1.0, 1.0) - key_F) * (1.0 - metallic);
    var Lo = (key_kD * albedo / PI + key_spec) * key_n_dot_l * key_shadow;

    // --- Back light ---
    let back_dir = normalize(vec3(-0.4, 0.3, -0.6));
    let back_n_dot_l = max(dot(nor, back_dir), 0.0);
    let back_H = normalize(V + back_dir);
    let back_h_dot_v = max(dot(back_H, V), 0.0);
    let back_F = fresnel_schlick(back_h_dot_v, f0);
    let back_kD = (vec3<f32>(1.0, 1.0, 1.0) - back_F) * (1.0 - metallic);
    Lo += (back_kD * albedo / PI) * params.back_color * params.back_intensity * back_n_dot_l;

    // --- Ambient (hemisphere diffuse + reflective sky specular) ---
    let sky_color = params.fill_color * params.fill_intensity;
    let ground_color = sky_color * 0.1;
    let ambient_diffuse = mix(ground_color, sky_color, 0.5 + 0.5 * nor.y) * albedo;

    let reflect_dir = reflect(-V, nor);
    let sky_reflect = mix(ground_color, sky_color, 0.5 + 0.5 * reflect_dir.y);
    let F_ambient = fresnel_schlick_roughness(n_dot_v, f0, roughness);
    let ambient_specular = sky_reflect * F_ambient;

    let kD_ambient = (vec3<f32>(1.0, 1.0, 1.0) - F_ambient) * (1.0 - metallic);
    let ambient = kD_ambient * ambient_diffuse + ambient_specular;

    var color = ambient * ao + Lo;

    // Subsurface scattering approximation
    let sss_factor = clamp(-dot(nor, key_dir), 0.0, 1.0);
    color += albedo * params.sss_color * sss_factor * params.sss_intensity;

    // Artistic fresnel override (optional per-shape boost)
    if mat.fresnel_power > 1.0 {
        let view_factor = 1.0 - n_dot_v;
        let rim = pow(view_factor, mat.fresnel_power) * 0.3;
        color += vec3<f32>(rim, rim, rim);
    }

    // --- Post-processing ---

    // Color grading: warm multiply + cool lift
    color *= vec3<f32>(1.05, 1.0, 0.95);
    color += vec3<f32>(0.01, 0.01, 0.03);
    // Vignette: darken edges of screen
    let screen_uv = (in.position.xy - view.viewport.xy) / view.viewport.zw;
    let vig_uv = screen_uv * 2.0 - 1.0; // [-1, 1]
    let vig = 1.0 - params.vignette_intensity * dot(vig_uv, vig_uv);
    color *= vig;

    // Gamma correction
    return vec4(pow(max(color, vec3<f32>(0.0, 0.0, 0.0)), vec3(1.0 / 2.2)), 1.0);
}
