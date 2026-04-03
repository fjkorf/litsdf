#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

const MAX_SHAPES: u32 = 32u;
const MAX_STEPS: u32 = 128u;
const MAX_DIST: f32 = 100.0;
const EPSILON: f32 = 0.001;
const PI: f32 = 3.14159265;

struct ShaderShape {
    primitive_type: u32,
    combination_op: u32,
    smooth_k: f32,
    _pad0: f32,
    params: vec4<f32>,
    translation: vec3<f32>,
    _pad1: f32,
    rotation: vec3<f32>,
    scale: f32,
    color: vec3<f32>,
    roughness: f32,
    metallic: f32,
    fresnel_power: f32,
    color_mode: u32,
    _pad3: f32,
    palette_a: vec3<f32>,
    _pad4: f32,
    palette_b: vec3<f32>,
    _pad5: f32,
    palette_c: vec3<f32>,
    _pad6: f32,
    palette_d: vec3<f32>,
    _pad7: f32,
    modifier_flags: u32,
    rounding: f32,
    onion_thickness: f32,
    twist_amount: f32,
    bend_amount: f32,
    _pad_mod0: vec3<f32>,
    elongation: vec3<f32>,
    _pad_mod1: f32,
    rep_period: vec3<f32>,
    _pad_mod2: f32,
    noise_amplitude: f32,
    noise_frequency: f32,
    noise_octaves: u32,
    smooth_symmetry: f32,
};

struct SdfParams {
    shape_count: u32,
    time: f32,
    _pad_h: vec2<f32>,
    light_dir: vec3<f32>,
    _pad_l: f32,
    shapes: array<ShaderShape, 32>,
    // Scene settings (after shapes to preserve array offset)
    fill_color: vec3<f32>,
    fill_intensity: f32,
    back_color: vec3<f32>,
    back_intensity: f32,
    sss_color: vec3<f32>,
    sss_intensity: f32,
    ao_intensity: f32,
    shadow_softness: f32,
    vignette_intensity: f32,
    _pad_s: f32,
};

// Accumulated material result from scene evaluation
struct MatResult {
    color: vec3<f32>,
    roughness: f32,
    metallic: f32,
    fresnel_power: f32,
    color_mode: u32,
};

@group(3) @binding(0) var<uniform> params: SdfParams;

// --- Hash and Noise ---

fn hash3(p: vec3<f32>) -> f32 {
    var q = fract(p * 0.3183099 + vec3<f32>(0.1, 0.1, 0.1));
    q *= 17.0;
    return fract(q.x * q.y * q.z * (q.x + q.y + q.z));
}

fn noise3d(x: vec3<f32>) -> f32 {
    let p = floor(x);
    let w = fract(x);
    let u = w * w * w * (w * (w * 6.0 - 15.0) + 10.0);

    let a = hash3(p + vec3<f32>(0.0, 0.0, 0.0));
    let b = hash3(p + vec3<f32>(1.0, 0.0, 0.0));
    let c = hash3(p + vec3<f32>(0.0, 1.0, 0.0));
    let d = hash3(p + vec3<f32>(1.0, 1.0, 0.0));
    let e = hash3(p + vec3<f32>(0.0, 0.0, 1.0));
    let f = hash3(p + vec3<f32>(1.0, 0.0, 1.0));
    let g = hash3(p + vec3<f32>(0.0, 1.0, 1.0));
    let h = hash3(p + vec3<f32>(1.0, 1.0, 1.0));

    let k0 = a;
    let k1 = b - a;
    let k2 = c - a;
    let k3 = e - a;
    let k4 = a - b - c + d;
    let k5 = a - c - e + g;
    let k6 = a - b - e + f;
    let k7 = -a + b + c - d + e - f - g + h;

    return -1.0 + 2.0 * (k0 + k1*u.x + k2*u.y + k3*u.z
        + k4*u.x*u.y + k5*u.y*u.z + k6*u.z*u.x
        + k7*u.x*u.y*u.z);
}

fn fbm(p: vec3<f32>, octaves: i32) -> f32 {
    var freq = 1.0;
    var amp = 1.0;
    var total = 0.0;
    var q = p;
    for (var i = 0; i < octaves; i += 1) {
        total += amp * noise3d(q * freq);
        freq *= 2.0;
        amp *= 0.5;
    }
    return total;
}

// Ridged multifractal — abs(noise) creates ridge patterns (mountains, veins)
fn fbm_ridged(p: vec3<f32>, octaves: i32) -> f32 {
    var freq = 1.0;
    var amp = 1.0;
    var total = 0.0;
    var q = p;
    for (var i = 0; i < octaves; i += 1) {
        total += amp * abs(noise3d(q * freq));
        freq *= 2.0;
        amp *= 0.5;
    }
    return total;
}

// Cellular/Voronoi noise — returns vec2(f1, f2) distances to nearest two cell centers
fn cellular3d(p: vec3<f32>) -> vec2<f32> {
    let i = floor(p);
    let f = p - i;
    var d1 = 8.0;
    var d2 = 8.0;
    for (var z = -1; z <= 1; z += 1) {
        for (var y = -1; y <= 1; y += 1) {
            for (var x = -1; x <= 1; x += 1) {
                let offset = vec3<f32>(f32(x), f32(y), f32(z));
                let cell = hash3(i + offset);
                let r = offset + cell - f;
                let dist = dot(r, r);
                if dist < d1 {
                    d2 = d1;
                    d1 = dist;
                } else if dist < d2 {
                    d2 = dist;
                }
            }
        }
    }
    return vec2(sqrt(d1), sqrt(d2));
}

// --- SDF Primitives ---

fn sd_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0, 0.0, 0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sd_round_box(p: vec3<f32>, b: vec3<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec3<f32>(r, r, r);
    return length(max(q, vec3<f32>(0.0, 0.0, 0.0))) + min(max(q.x, max(q.y, q.z)), 0.0) - r;
}

fn sd_cylinder(p: vec3<f32>, h: f32, r: f32) -> f32 {
    let d = vec2(length(p.xz) - r, abs(p.y) - h);
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2(0.0)));
}

fn sd_capped_cone(p: vec3<f32>, h: f32, r1: f32, r2: f32) -> f32 {
    let q = vec2(length(p.xz), p.y);
    let k1 = vec2(r2, h);
    let k2 = vec2(r2 - r1, 2.0 * h);
    let ca = vec2(q.x - min(q.x, select(r2, r1, q.y < 0.0)), abs(q.y) - h);
    let cb = q - k1 + k2 * clamp(dot(k1 - q, k2) / dot(k2, k2), 0.0, 1.0);
    let s = select(1.0, -1.0, cb.x < 0.0 && ca.y < 0.0);
    return s * sqrt(min(dot(ca, ca), dot(cb, cb)));
}

fn sd_torus(p: vec3<f32>, major: f32, minor: f32) -> f32 {
    let q = vec2(length(p.xz) - major, p.y);
    return length(q) - minor;
}

fn sd_capsule(p: vec3<f32>, r: f32, h: f32) -> f32 {
    var q = p;
    q.y -= clamp(q.y, -h, h);
    return length(q) - r;
}

fn sd_plane(p: vec3<f32>, n: vec3<f32>, d: f32) -> f32 {
    return dot(p, normalize(n)) + d;
}

fn sd_octahedron(p: vec3<f32>, s: f32) -> f32 {
    let q = abs(p);
    let m = q.x + q.y + q.z - s;
    var r: vec3<f32>;
    if 3.0 * q.x < m { r = q; }
    else if 3.0 * q.y < m { r = vec3(q.y, q.z, q.x); }
    else if 3.0 * q.z < m { r = vec3(q.z, q.x, q.y); }
    else { return m * 0.57735027; }
    let k = clamp(0.5 * (r.z - r.y + s), 0.0, s);
    return length(vec3(r.x, r.y - s + k, r.z - k));
}

fn sd_pyramid(p: vec3<f32>, h: f32, base: f32) -> f32 {
    let q = abs(p);
    let d = max(q.y - h, (q.x + q.z) * 0.707107 - base * 0.5);
    return max(d, -p.y);
}

fn sd_hex_prism(p: vec3<f32>, h: f32, r: f32) -> f32 {
    let q = abs(p);
    let d = max(q.x * 0.866025 + q.z * 0.5 - r, q.z - r);
    return length(max(vec2(d, q.y - h), vec2(0.0))) + min(max(d, q.y - h), 0.0);
}

fn sd_round_cone(p: vec3<f32>, r1: f32, r2: f32, h: f32) -> f32 {
    let q = vec2(length(p.xz), p.y);
    let b = (r1 - r2) / h;
    let a = sqrt(1.0 - b * b);
    let k = dot(q, vec2(-b, a));
    if k < 0.0 { return length(q) - r1; }
    if k > a * h { return length(q - vec2(0.0, h)) - r2; }
    return dot(q, vec2(a, b)) - r1;
}

fn sd_ellipsoid(p: vec3<f32>, r: vec3<f32>) -> f32 {
    let k0 = length(p / r);
    let k1 = length(p / (r * r));
    if k1 == 0.0 { return 0.0; }
    return k0 * (k0 - 1.0) / k1;
}

// --- Rotation ---

fn rotate_point(p: vec3<f32>, euler: vec3<f32>) -> vec3<f32> {
    let cx = cos(euler.x); let sx = sin(euler.x);
    let cy = cos(euler.y); let sy = sin(euler.y);
    let cz = cos(euler.z); let sz = sin(euler.z);

    var q = p;
    q = vec3(q.x * cz - q.y * sz, q.x * sz + q.y * cz, q.z);
    q = vec3(q.x, q.y * cx - q.z * sx, q.y * sx + q.z * cx);
    q = vec3(q.x * cy + q.z * sy, q.y, -q.x * sy + q.z * cy);

    return q;
}

// --- Domain modifiers (transform sample point) ---

fn op_twist(p: vec3<f32>, k: f32) -> vec3<f32> {
    let c = cos(k * p.y);
    let s = sin(k * p.y);
    return vec3(c * p.x - s * p.z, p.y, s * p.x + c * p.z);
}

fn op_bend(p: vec3<f32>, k: f32) -> vec3<f32> {
    let c = cos(k * p.x);
    let s = sin(k * p.x);
    return vec3(c * p.x - s * p.y, s * p.x + c * p.y, p.z);
}

fn op_elongate(p: vec3<f32>, h: vec3<f32>) -> vec3<f32> {
    return p - clamp(p, -h, h);
}

fn op_repeat(p: vec3<f32>, period: vec3<f32>) -> vec3<f32> {
    return p - period * round(p / period);
}

// --- Evaluate single shape ---

fn eval_shape(p: vec3<f32>, shape: ShaderShape) -> f32 {
    let trans = shape.translation;
    let rot = shape.rotation;
    let sc = max(shape.scale, 0.001);

    var q = (p - trans) / sc;
    q = rotate_point(q, -rot);

    // Smooth symmetry (crease-free abs)
    if shape.smooth_symmetry > 0.0 {
        q.x = sqrt(q.x * q.x + shape.smooth_symmetry);
    }

    // Pre-primitive modifiers (transform the sample point)
    if (shape.modifier_flags & 4u) != 0u { q = op_twist(q, shape.twist_amount); }
    if (shape.modifier_flags & 8u) != 0u { q = op_bend(q, shape.bend_amount); }
    if (shape.modifier_flags & 16u) != 0u { q = op_elongate(q, shape.elongation); }
    if (shape.modifier_flags & 32u) != 0u { q = op_repeat(q, shape.rep_period); }

    var d: f32;
    switch shape.primitive_type {
        case 0u: { d = sd_sphere(q, shape.params.x); }
        case 1u: { d = sd_box(q, shape.params.xyz); }
        case 2u: { d = sd_round_box(q, shape.params.xyz, shape.params.w); }
        case 3u: { d = sd_cylinder(q, shape.params.x, shape.params.y); }
        case 4u: { d = sd_capped_cone(q, shape.params.x, shape.params.y, shape.params.z); }
        case 5u: { d = sd_torus(q, shape.params.x, shape.params.y); }
        case 6u: { d = sd_capsule(q, shape.params.x, shape.params.y); }
        case 7u: { d = sd_plane(q, vec3(shape.params.x, shape.params.y, shape.params.z), shape.params.w); }
        case 8u: { d = sd_ellipsoid(q, shape.params.xyz); }
        case 9u: { d = sd_octahedron(q, shape.params.x); }
        case 10u: { d = sd_pyramid(q, shape.params.x, shape.params.y); }
        case 11u: { d = sd_hex_prism(q, shape.params.x, shape.params.y); }
        case 12u: { d = sd_round_cone(q, shape.params.x, shape.params.y, shape.params.z); }
        default: { d = MAX_DIST; }
    }

    // Post-primitive modifiers (transform the distance)
    if (shape.modifier_flags & 1u) != 0u { d -= shape.rounding; }
    if (shape.modifier_flags & 2u) != 0u { d = abs(d) - shape.onion_thickness; }

    // Noise displacement
    if shape.noise_octaves > 0u {
        d += shape.noise_amplitude * fbm(q * shape.noise_frequency, i32(shape.noise_octaves));
    }

    return d * sc;
}

// --- Get shape color (solid or cosine palette) ---

fn get_shape_color(shape: ShaderShape, p: vec3<f32>) -> vec3<f32> {
    if shape.color_mode == 0u {
        return shape.color;
    }
    let local_p = (p - shape.translation) / max(shape.scale, 0.001);
    if shape.color_mode == 1u {
        // Cosine palette: a + b * cos(2π(ct + d))
        let t = length(local_p) * 0.5;
        return shape.palette_a + shape.palette_b * cos(2.0 * PI * (shape.palette_c * t + shape.palette_d));
    }
    if shape.color_mode == 2u {
        // Mode 2: Noise-modulated color
        let n = noise3d(local_p * max(shape.noise_frequency, 0.1) * 2.0) * 0.5 + 0.5;
        return shape.color * (0.7 + 0.3 * n);
    }
    if shape.color_mode == 3u {
        // Mode 3: Cellular/Voronoi pattern
        let cell = cellular3d(local_p * max(shape.noise_frequency, 0.1));
        let edge = smoothstep(0.0, 0.1, cell.y - cell.x);
        return shape.color * (0.5 + 0.5 * edge);
    }
    if shape.color_mode == 4u {
        // Mode 4: Ridged multifractal — abs(noise) creates ridges
        let n = fbm_ridged(local_p * max(shape.noise_frequency, 0.1), i32(shape.noise_octaves));
        return shape.color * (0.5 + 0.5 * n);
    }
    // Mode 5: Gradient-based (normal.y → snow, normal.x → moss)
    // Handled in fragment shader after normal is computed — falls through to solid here
    return shape.color;
}

// --- Combination with blend factor ---
// Returns vec2(blended_distance, blend_factor)
// blend_factor: 0.0 = scene dominates, 1.0 = new shape dominates

fn combine_blend(d_scene: f32, d_shape: f32, op: u32, k: f32) -> vec2<f32> {
    switch op {
        case 0u: { // Union
            if d_scene <= d_shape { return vec2(d_scene, 0.0); }
            else { return vec2(d_shape, 1.0); }
        }
        case 1u: { // Intersection
            if d_scene >= d_shape { return vec2(d_scene, 0.0); }
            else { return vec2(d_shape, 1.0); }
        }
        case 2u: { // Subtraction
            if d_scene >= -d_shape { return vec2(d_scene, 0.0); }
            else { return vec2(-d_shape, 1.0); }
        }
        case 3u: { // SmoothUnion
            let h = clamp(0.5 + 0.5 * (d_shape - d_scene) / k, 0.0, 1.0);
            let d = mix(d_shape, d_scene, h) - k * h * (1.0 - h);
            return vec2(d, 1.0 - h);
        }
        case 4u: { // SmoothIntersection
            let h = clamp(0.5 - 0.5 * (d_shape - d_scene) / k, 0.0, 1.0);
            let d = mix(d_shape, d_scene, h) + k * h * (1.0 - h);
            return vec2(d, 1.0 - h);
        }
        case 5u: { // SmoothSubtraction
            let h = clamp(0.5 - 0.5 * (d_shape + d_scene) / k, 0.0, 1.0);
            let d = mix(d_scene, -d_shape, h) + k * h * (1.0 - h);
            return vec2(d, h);
        }
        case 6u: { // ChamferUnion — beveled edge between shapes
            let d = min(min(d_scene, d_shape), (d_scene + d_shape - k) * 0.707107);
            let blend = select(0.0, 1.0, d_shape < d_scene);
            return vec2(d, blend);
        }
        case 7u: { // ChamferIntersection — beveled edge at intersection
            let d = max(max(d_scene, d_shape), (d_scene + d_shape + k) * 0.707107);
            let blend = select(0.0, 1.0, d_shape > d_scene);
            return vec2(d, blend);
        }
        default: {
            if d_scene <= d_shape { return vec2(d_scene, 0.0); }
            else { return vec2(d_shape, 1.0); }
        }
    }
}

// --- Distance-only scene evaluation (for normals, shadows, AO) ---

fn sdf_scene(p: vec3<f32>) -> f32 {
    if params.shape_count == 0u {
        return MAX_DIST;
    }

    var d = eval_shape(p, params.shapes[0]);

    for (var i = 1u; i < params.shape_count; i++) {
        let shape = params.shapes[i];
        let d_shape = eval_shape(p, shape);
        d = combine_blend(d, d_shape, shape.combination_op, shape.smooth_k).x;
    }

    return d;
}

// --- Material scene evaluation (at hit point only) ---

fn sdf_scene_material(p: vec3<f32>) -> MatResult {
    if params.shape_count == 0u {
        return MatResult(vec3<f32>(0.5, 0.5, 0.5), 0.5, 0.0, 0.0, 0u);
    }

    let s0 = params.shapes[0];
    var result = MatResult(
        get_shape_color(s0, p),
        s0.roughness,
        s0.metallic,
        s0.fresnel_power,
        s0.color_mode,
    );
    var d = eval_shape(p, s0);

    for (var i = 1u; i < params.shape_count; i++) {
        let shape = params.shapes[i];
        let d_shape = eval_shape(p, shape);
        let blend = combine_blend(d, d_shape, shape.combination_op, shape.smooth_k);
        let t = blend.y;

        d = blend.x;
        result.color = mix(result.color, get_shape_color(shape, p), t);
        result.roughness = mix(result.roughness, shape.roughness, t);
        result.metallic = mix(result.metallic, shape.metallic, t);
        result.fresnel_power = mix(result.fresnel_power, shape.fresnel_power, t);
        if t > 0.5 { result.color_mode = shape.color_mode; }
    }

    return result;
}

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
