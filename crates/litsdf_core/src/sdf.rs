//! CPU-side SDF primitive evaluation functions.
//! These mirror the WGSL shader implementations for picking and other CPU uses.

use glam::{Vec2, Vec3};
use crate::models::SdfPrimitive;

pub fn sd_sphere(p: Vec3, r: f32) -> f32 {
    p.length() - r
}

pub fn sd_box(p: Vec3, b: Vec3) -> f32 {
    let q = p.abs() - b;
    q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0)
}

pub fn sd_round_box(p: Vec3, b: Vec3, r: f32) -> f32 {
    let q = p.abs() - b + Vec3::splat(r);
    q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0) - r
}

pub fn sd_cylinder(p: Vec3, h: f32, r: f32) -> f32 {
    let d = Vec2::new(Vec2::new(p.x, p.z).length() - r, p.y.abs() - h);
    d.x.max(d.y).min(0.0) + d.max(Vec2::ZERO).length()
}

pub fn sd_capped_cone(p: Vec3, h: f32, r1: f32, r2: f32) -> f32 {
    let q = Vec2::new(Vec2::new(p.x, p.z).length(), p.y);
    let k1 = Vec2::new(r2, h);
    let k2 = Vec2::new(r2 - r1, 2.0 * h);
    let ca = Vec2::new(
        q.x - q.x.min(if q.y < 0.0 { r1 } else { r2 }),
        q.y.abs() - h,
    );
    let cb = q - k1 + k2 * ((k1 - q).dot(k2) / k2.dot(k2)).clamp(0.0, 1.0);
    let s = if cb.x < 0.0 && ca.y < 0.0 { -1.0 } else { 1.0 };
    s * ca.dot(ca).min(cb.dot(cb)).sqrt()
}

pub fn sd_torus(p: Vec3, major: f32, minor: f32) -> f32 {
    let q = Vec2::new(Vec2::new(p.x, p.z).length() - major, p.y);
    q.length() - minor
}

pub fn sd_capsule(p: Vec3, r: f32, h: f32) -> f32 {
    let mut q = p;
    q.y -= q.y.clamp(-h, h);
    q.length() - r
}

pub fn sd_plane(p: Vec3, n: Vec3, d: f32) -> f32 {
    p.dot(n.normalize()) + d
}

pub fn sd_ellipsoid(p: Vec3, r: Vec3) -> f32 {
    let k0 = (p / r).length();
    let k1 = (p / (r * r)).length();
    if k1 == 0.0 { return 0.0; }
    k0 * (k0 - 1.0) / k1
}

pub fn rotate_point(p: Vec3, euler: Vec3) -> Vec3 {
    let (cx, sx) = (euler.x.cos(), euler.x.sin());
    let (cy, sy) = (euler.y.cos(), euler.y.sin());
    let (cz, sz) = (euler.z.cos(), euler.z.sin());
    let mut q = p;
    q = Vec3::new(q.x * cz - q.y * sz, q.x * sz + q.y * cz, q.z);
    q = Vec3::new(q.x, q.y * cx - q.z * sx, q.y * sx + q.z * cx);
    q = Vec3::new(q.x * cy + q.z * sy, q.y, -q.x * sy + q.z * cy);
    q
}

pub fn sd_octahedron(p: Vec3, s: f32) -> f32 {
    let p = p.abs();
    let m = p.x + p.y + p.z - s;
    let q = if 3.0 * p.x < m { p }
    else if 3.0 * p.y < m { Vec3::new(p.y, p.z, p.x) }
    else if 3.0 * p.z < m { Vec3::new(p.z, p.x, p.y) }
    else { return m * 0.57735027; };
    let k = (0.5 * (q.z - q.y + s)).clamp(0.0, s);
    (Vec3::new(q.x, q.y - s + k, q.z - k)).length()
}

pub fn sd_pyramid(p: Vec3, h: f32, base: f32) -> f32 {
    // Approximate: distance to a square pyramid
    let q = p.abs();
    let d = (q.y - h).max((q.x + q.z) * 0.707107 - base * 0.5);
    d.max(-p.y)
}

pub fn sd_hex_prism(p: Vec3, h: f32, r: f32) -> f32 {
    let q = p.abs();
    let d = (q.x * 0.866025 + q.z * 0.5 - r).max(q.z - r);
    Vec2::new(d, q.y - h).max(Vec2::ZERO).length() + d.max(q.y - h).min(0.0)
}

pub fn sd_round_cone(p: Vec3, r1: f32, r2: f32, h: f32) -> f32 {
    let q = Vec2::new(Vec2::new(p.x, p.z).length(), p.y);
    let b = (r1 - r2) / h;
    let a = (1.0 - b * b).sqrt();
    let k = q.dot(Vec2::new(-b, a));
    if k < 0.0 { q.length() - r1 }
    else if k > a * h { (q - Vec2::new(0.0, h)).length() - r2 }
    else { q.dot(Vec2::new(a, b)) - r1 }
}

/// Evaluate an SDF primitive at a point in the primitive's local space.
pub fn eval_primitive(p: Vec3, prim: &SdfPrimitive) -> f32 {
    match prim {
        SdfPrimitive::Sphere { radius } => sd_sphere(p, *radius),
        SdfPrimitive::Box { half_extents } => sd_box(p, Vec3::from_array(*half_extents)),
        SdfPrimitive::RoundBox { half_extents, rounding } => sd_round_box(p, Vec3::from_array(*half_extents), *rounding),
        SdfPrimitive::Cylinder { height, radius } => sd_cylinder(p, *height, *radius),
        SdfPrimitive::CappedCone { height, r1, r2 } => sd_capped_cone(p, *height, *r1, *r2),
        SdfPrimitive::Torus { major_radius, minor_radius } => sd_torus(p, *major_radius, *minor_radius),
        SdfPrimitive::Capsule { radius, half_height } => sd_capsule(p, *radius, *half_height),
        SdfPrimitive::Plane { normal, offset } => sd_plane(p, Vec3::from_array(*normal), *offset),
        SdfPrimitive::Ellipsoid { radii } => sd_ellipsoid(p, Vec3::from_array(*radii)),
        SdfPrimitive::Octahedron { size } => sd_octahedron(p, *size),
        SdfPrimitive::Pyramid { height, base } => sd_pyramid(p, *height, *base),
        SdfPrimitive::HexPrism { height, radius } => sd_hex_prism(p, *height, *radius),
        SdfPrimitive::RoundCone { r1, r2, height } => sd_round_cone(p, *r1, *r2, *height),
    }
}
