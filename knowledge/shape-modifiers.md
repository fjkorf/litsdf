# Shape Modifiers

## Overview

Shape modifiers transform either the sample point (domain modifiers) or the distance value (post-distance modifiers). They are applied in `eval_shape()` in the WGSL shader.

## Modifier Types

### Post-Distance Modifiers (applied after primitive evaluation)

| Modifier | Parameter | Effect | Formula |
|----------|-----------|--------|---------|
| **Rounding** | `f32` radius | Rounds all edges | `d -= rounding` |
| **Onion** | `f32` thickness | Hollows shape into a shell | `d = abs(d) - thickness` |

### Domain Modifiers (applied before primitive evaluation, transform the sample point)

| Modifier | Parameter | Effect | Formula |
|----------|-----------|--------|---------|
| **Twist** | `f32` amount | Twists shape along Y axis | Rotate XZ plane by `amount * p.y` |
| **Bend** | `f32` amount | Bends shape along X axis | Rotate XY plane by `amount * p.x` |
| **Elongation** | `[f32; 3]` | Stretches shape along each axis | `p = p - clamp(p, -h, h)` |
| **Repetition** | `[f32; 3]` period | Tiles shape infinitely | `p = p - period * round(p / period)` |

## Encoding

Modifiers are encoded as a bitmask on `ShaderShape`:

```
bit 0 (1)  = Rounding
bit 1 (2)  = Onion
bit 2 (4)  = Twist
bit 3 (8)  = Bend
bit 4 (16) = Elongation
bit 5 (32) = Repetition
```

The `modifier_flags: u32` field is checked with bitwise AND. Each modifier has a dedicated parameter field on ShaderShape (not packed dynamically).

## Application Order

In `eval_shape()`:
1. Transform point to shape local space (translate, scale, rotate)
2. Apply domain modifiers in order: Twist → Bend → Elongation → Repetition
3. Evaluate primitive SDF
4. Apply post-distance modifiers: Rounding → Onion
5. Scale distance back to world space

## Data Model

In `models.rs`, modifiers are `Vec<ShapeModifier>` on each `SdfShape`. In `scene.rs`, `encode_modifier_flags()` converts the Vec to a bitmask + fields. Multiple modifiers of the same type: last one wins.

## CPU Picking

The Rust SDF in `picking.rs` does NOT currently apply modifiers. Shapes with modifiers will have inaccurate picking. This is a known limitation — porting the modifier functions to Rust is the fix.
