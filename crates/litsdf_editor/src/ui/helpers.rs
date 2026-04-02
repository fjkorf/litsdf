use litsdf_core::models::{CombinationOp, SdfPrimitive};

pub const PRIM_NAMES: &[&str] = &[
    "Sphere", "Box", "RoundBox", "Cylinder", "CappedCone",
    "Torus", "Capsule", "Plane", "Ellipsoid",
    "Octahedron", "Pyramid", "HexPrism", "RoundCone",
];

pub fn prim_to_index(p: &SdfPrimitive) -> usize {
    match p {
        SdfPrimitive::Sphere { .. } => 0,
        SdfPrimitive::Box { .. } => 1,
        SdfPrimitive::RoundBox { .. } => 2,
        SdfPrimitive::Cylinder { .. } => 3,
        SdfPrimitive::CappedCone { .. } => 4,
        SdfPrimitive::Torus { .. } => 5,
        SdfPrimitive::Capsule { .. } => 6,
        SdfPrimitive::Plane { .. } => 7,
        SdfPrimitive::Ellipsoid { .. } => 8,
        SdfPrimitive::Octahedron { .. } => 9,
        SdfPrimitive::Pyramid { .. } => 10,
        SdfPrimitive::HexPrism { .. } => 11,
        SdfPrimitive::RoundCone { .. } => 12,
    }
}

pub fn prim_params(p: &SdfPrimitive) -> (f64, f64, f64, f64) {
    match p {
        SdfPrimitive::Sphere { radius } => (*radius as f64, 0.0, 0.0, 0.0),
        SdfPrimitive::Box { half_extents: h } => (h[0] as f64, h[1] as f64, h[2] as f64, 0.0),
        SdfPrimitive::RoundBox { half_extents: h, rounding: r } => (h[0] as f64, h[1] as f64, h[2] as f64, *r as f64),
        SdfPrimitive::Cylinder { height: h, radius: r } => (*h as f64, *r as f64, 0.0, 0.0),
        SdfPrimitive::CappedCone { height: h, r1, r2 } => (*h as f64, *r1 as f64, *r2 as f64, 0.0),
        SdfPrimitive::Torus { major_radius: a, minor_radius: b } => (*a as f64, *b as f64, 0.0, 0.0),
        SdfPrimitive::Capsule { radius: r, half_height: h } => (*r as f64, *h as f64, 0.0, 0.0),
        SdfPrimitive::Plane { normal: n, offset: o } => (n[0] as f64, n[1] as f64, n[2] as f64, *o as f64),
        SdfPrimitive::Ellipsoid { radii: r } => (r[0] as f64, r[1] as f64, r[2] as f64, 0.0),
        SdfPrimitive::Octahedron { size } => (*size as f64, 0.0, 0.0, 0.0),
        SdfPrimitive::Pyramid { height, base } => (*height as f64, *base as f64, 0.0, 0.0),
        SdfPrimitive::HexPrism { height, radius } => (*height as f64, *radius as f64, 0.0, 0.0),
        SdfPrimitive::RoundCone { r1, r2, height } => (*r1 as f64, *r2 as f64, *height as f64, 0.0),
    }
}

pub fn set_prim_params(p: &mut SdfPrimitive, a: f32, b: f32, c: f32, d: f32) {
    match p {
        SdfPrimitive::Sphere { radius } => *radius = a,
        SdfPrimitive::Box { half_extents: h } => *h = [a, b, c],
        SdfPrimitive::RoundBox { half_extents: h, rounding: r } => { *h = [a, b, c]; *r = d; }
        SdfPrimitive::Cylinder { height: h, radius: r } => { *h = a; *r = b; }
        SdfPrimitive::CappedCone { height: h, r1, r2 } => { *h = a; *r1 = b; *r2 = c; }
        SdfPrimitive::Torus { major_radius: ma, minor_radius: mi } => { *ma = a; *mi = b; }
        SdfPrimitive::Capsule { radius: r, half_height: h } => { *r = a; *h = b; }
        SdfPrimitive::Plane { normal: n, offset: o } => { *n = [a, b, c]; *o = d; }
        SdfPrimitive::Ellipsoid { radii: r } => *r = [a, b, c],
        SdfPrimitive::Octahedron { size } => *size = a,
        SdfPrimitive::Pyramid { height, base } => { *height = a; *base = b; }
        SdfPrimitive::HexPrism { height, radius } => { *height = a; *radius = b; }
        SdfPrimitive::RoundCone { r1, r2, height } => { *r1 = a; *r2 = b; *height = c; }
    }
}

pub fn combo_to_index(c: &CombinationOp) -> usize {
    match c {
        CombinationOp::Union => 0,
        CombinationOp::Intersection => 1,
        CombinationOp::Subtraction => 2,
        CombinationOp::SmoothUnion { .. } => 3,
        CombinationOp::SmoothIntersection { .. } => 4,
        CombinationOp::SmoothSubtraction { .. } => 5,
        CombinationOp::ChamferUnion { .. } => 6,
        CombinationOp::ChamferIntersection { .. } => 7,
    }
}

pub fn combo_smooth_k(c: &CombinationOp) -> f32 {
    match c {
        CombinationOp::SmoothUnion { k }
        | CombinationOp::SmoothIntersection { k }
        | CombinationOp::SmoothSubtraction { k }
        | CombinationOp::ChamferUnion { k }
        | CombinationOp::ChamferIntersection { k } => *k,
        _ => 0.0,
    }
}

pub fn index_to_combo(i: usize, k: f32) -> CombinationOp {
    match i {
        0 => CombinationOp::Union,
        1 => CombinationOp::Intersection,
        2 => CombinationOp::Subtraction,
        3 => CombinationOp::SmoothUnion { k },
        4 => CombinationOp::SmoothIntersection { k },
        5 => CombinationOp::SmoothSubtraction { k },
        6 => CombinationOp::ChamferUnion { k },
        7 => CombinationOp::ChamferIntersection { k },
        _ => CombinationOp::Union,
    }
}
