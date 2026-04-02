# Node Editor Architecture — Design Document

Synthesis of library research, industry patterns, and litsdf feature mapping. **Phase A (Foundation) and Phase B (Evaluation) are implemented.** Animation triplet fields have been removed entirely — all animation is now done via node graphs.

---

## Core Concept

Every shape property that currently has a hardcoded value becomes a **socket** — a typed input that can receive either a default value or a connection from another node. Animation, noise, math, and procedural nodes produce values that flow into these sockets.

The node graph **compiles to shader parameters**, not to runtime-evaluated code. The existing `ShaderShape` struct remains the GPU interface; nodes determine what values fill it each frame.

---

## Library Choice

**egui-snarl** — `Snarl<SdfNode>` where `SdfNode` is an enum of all node types.

Version compatibility: egui-snarl requires egui 0.34, litsdf uses 0.33. Must resolve before implementation (bevy_egui upgrade or snarl version pin).

---

## Data Model

### Socket Types

```rust
enum SocketType {
    Float,      // f32 — distance, scalar params
    Vec3,       // [f32; 3] — position, color, direction
    Distance,   // f32 — semantic alias for SDF values (enables type checking)
    Transform,  // translation + rotation + scale
}
```

Float and Distance are both f32 but semantically distinct — a Distance output can connect to a Float input (implicit upcast) but not vice versa, preventing accidental use of arbitrary floats as SDF distances.

### Node Types

```rust
enum SdfNode {
    // Primitives (output: Distance)
    Sphere { radius: f32 },
    Box { half_extents: [f32; 3] },
    RoundBox { half_extents: [f32; 3], rounding: f32 },
    Cylinder { height: f32, radius: f32 },
    CappedCone { height: f32, r1: f32, r2: f32 },
    Torus { major_radius: f32, minor_radius: f32 },
    Capsule { radius: f32, half_height: f32 },
    Plane { normal: [f32; 3], offset: f32 },
    Ellipsoid { radii: [f32; 3] },

    // Boolean operations (input: 2x Distance, output: Distance)
    Union,
    Intersection,
    Subtraction,
    SmoothUnion { k: f32 },
    SmoothIntersection { k: f32 },
    SmoothSubtraction { k: f32 },

    // Domain modifiers (input: Distance, output: Distance)
    Twist { amount: f32 },
    Bend { amount: f32 },
    Elongate { amount: [f32; 3] },
    Repeat { period: [f32; 3] },
    Round { radius: f32 },
    Shell { thickness: f32 },

    // Value generators (output: Float)
    Time,                                    // outputs current time in seconds
    SinOscillator { amplitude: f32, frequency: f32, phase: f32 },
    Noise { frequency: f32, octaves: u32 },
    Constant { value: f32 },

    // Math (input: Float(s), output: Float)
    Add,
    Multiply,
    Mix { factor: f32 },                     // lerp(a, b, factor)
    Clamp { min: f32, max: f32 },

    // Vec3 operations
    Vec3Compose,                             // 3x Float → Vec3
    Vec3Decompose,                           // Vec3 → 3x Float

    // Material (output node — collects PBR properties)
    Material {
        color: [f32; 3],
        roughness: f32,
        metallic: f32,
        fresnel: f32,
    },

    // Color generators (output: Vec3)
    CosinePalette {
        a: [f32; 3],
        b: [f32; 3],
        c: [f32; 3],
        d: [f32; 3],
    },
    SolidColor { color: [f32; 3] },
}
```

Each variant's fields are the **default values** for unconnected inputs.

### Graph Structure

Each shape has an optional node graph. When present, the graph overrides the shape's static properties. When absent, the shape uses its current flat property model (backward compatible).

```rust
// In SdfShape:
pub struct SdfShape {
    pub id: ShapeId,
    pub name: String,
    pub primitive: SdfPrimitive,
    pub visible: bool,
    pub transform: ShapeTransform,
    pub material: ShapeMaterial,
    pub modifiers: Vec<ShapeModifier>,
    pub combination: CombinationOp,
    // NEW: optional node graph
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph: Option<NodeGraph>,
}
```

The `NodeGraph` wraps `Snarl<SdfNode>` with additional metadata:

```rust
pub struct NodeGraph {
    pub snarl: Snarl<SdfNode>,
    // Which node's output feeds each shape property
    pub output_map: OutputMap,
}

pub struct OutputMap {
    pub distance: Option<(NodeId, usize)>,  // which node/pin provides SDF distance
    pub color: Option<(NodeId, usize)>,
    pub roughness: Option<(NodeId, usize)>,
    // ... etc
}
```

---

## Evaluation Strategy

### Not Runtime Graph Walking

The node graph does NOT evaluate at runtime by walking the DAG. Instead:

1. **Per-frame CPU pass:** Walk the graph, evaluate time-dependent nodes (Time, SinOscillator), compute final values for each shape property.
2. **Write values into ShaderShape** — the existing flat struct that gets sent to the GPU.
3. **GPU evaluates as before** — the shader doesn't know about nodes.

This means:
- No shader code generation (complex, fragile)
- No performance regression (same shader, same GPU cost)
- Nodes are a CPU-side value computation layer
- The 32-shape limit and existing ShaderShape format are preserved

### Per-Pixel Nodes (Future)

Some nodes (noise displacement, cosine palette) currently evaluate per-pixel in the shader. These can't be replaced by per-frame CPU values. Two approaches:

1. **Keep shader features as-is** — noise params, color mode, palette params remain flat fields on ShaderShape. Nodes just set those params.
2. **Future: shader code generation** — compile per-pixel node subgraphs to WGSL fragments. Much more complex, defer to later.

**Recommendation:** Start with approach 1. Nodes compute per-frame values that fill ShaderShape fields. Per-pixel shader behavior stays hardcoded. This gives 80% of the value (animation, dynamic transforms, parameter linking) with 20% of the complexity.

---

## Phased Implementation

### Phase A: Foundation ✅ COMPLETE
- egui-snarl 0.9.0 added (compatible with egui 0.33)
- `SdfNode` enum initially with 13 variants, expanded to 24 across all phases
- `SnarlViewer<SdfNode>` with typed pins (Float=blue, Vec3=yellow), connection validation, context menus
- Node graphs stored in editor as `HashMap<ShapeId/BoneId, Snarl<SdfNode>>` (not on core model — keeps core snarl-free)
- Toggleable bottom panel via View > Node Editor

### Phase B: Evaluation ✅ COMPLETE
- Recursive pull-based evaluator with output caching
- ShapeOutput (10 pins: pos/rot/scale/color) and BoneOutput (7 pins: pos/rot/scale)
- Per-frame evaluation in `editor_ui`, values override shape/bone properties
- Animation triplets (`anim_*`) removed entirely from SdfBone and ShapeMaterial
- Old YAML files fail with descriptive migration message
- "Create Starter Graph" button creates Time → SinOscillator → Output template
- 4 evaluator unit tests

### Phase C: Full Integration ✅ COMPLETE
- ProjectFile persistence (scene + graphs in single YAML, backward compatible with old scenes)
- ShapeOutput expanded to 27 pins: transform(7) + color(3) + material(3) + noise(3) + symmetry(1) + modifiers(10)
- All material and modifier properties drivable by nodes
- 7 graph presets: bob, spin, pulse, orbit, color_cycle + bone variants
- Default scene auto-initializes with node-driven animation
- CosinePalette node: `a + b * cos(2π(ct+d))` with Vec3 I/O
- 11 new node types: SquareWave, TriangleWave, SawtoothWave, EaseInOut, Remap, Abs, Modulo, CosinePalette, ExpImpulse, SmoothStep, Noise1D (24 total)

### Phase D: Shader + Polish ✅ COMPLETE
- SceneSettings with 9 tunable lighting/post-processing params (fill, back, SSS, AO, shadow, vignette)
- SceneSettings UI sliders in properties panel (bidirectional sync)
- Cellular/Voronoi noise in shader (color_mode 3)
- Ridged multifractal noise (color_mode 4), Gradient Snow (color_mode 5) — 6 color modes total
- ChamferUnion + ChamferIntersection boolean ops — 8 combination ops total
- 4 new SDF primitives: Octahedron, Pyramid, HexPrism, RoundCone — 13 primitives total
- 3 animation shaping nodes: ExpImpulse, SmoothStep, Noise1D — 24 node types total
- Node color-coding by category (green/teal/blue/amber/red)
- Graph undo snapshots on preset apply/clear
- Picking delegates to core eval_primitive (eliminated duplicated SDF match)
- Gotcha: SdfShaderParams settings MUST go after shapes array (Metal compat)

### Phase F: PBR + Shader Codegen ✅ COMPLETE
- Cook-Torrance PBR BRDF replaces Blinn-Phong (GGX + Fresnel-Schlick + Smith geometry)
- Shader split into preamble (426 lines) + generated body + postamble (181 lines)
- `codegen::generate_shader(scene)` unrolls shape evaluation loop into static indexed code
- `codegen::topology_hash(scene)` triggers recompilation only on structural changes
- Runtime injection: writes generated WGSL to assets, Bevy hot-reloads
- 3 codegen tests (default scene, empty scene, hash change detection)
- Gotcha: `vec3(scalar)` not supported by Bevy's naga — use `vec3<f32>(x,x,x)`

### Future
- Per-pixel node expression codegen (emit node formulas as inline WGSL)
- Subgraph encapsulation (node groups)
- Custom nodes (user-defined WGSL snippets)
- Performance profiling per-node

---

## Scope Boundaries

### What Nodes Have Replaced
- Animation triplets (anim_tx/ty/tz/rx/ry/rz/scale) → **removed entirely**, replaced by Time + Oscillator nodes
- Static property values → Constant nodes (or default values on sockets)
- Modifier params → can be driven by nodes in future (e.g., twist amount oscillates)
- Material properties → can be driven by nodes in future

### What Nodes Don't Replace
- Bone hierarchy (spatial organization, stays as tree)
- Shape ordering (determines combination order)
- Visibility flags (boolean, not continuous)
- Scene-level settings (light direction, scene name)
- The WGSL shader itself (still hand-written, nodes fill its uniform inputs)

---

## Serialization

Node graphs serialize to YAML via serde, nested inside the shape:

```yaml
- id: abc123
  name: BouncingBall
  primitive: !Sphere
    radius: 1.0
  graph:
    nodes:
      - type: Time
        position: [100, 100]
      - type: SinOscillator
        amplitude: 0.5
        frequency: 2.0
        phase: 0.0
        position: [300, 100]
    connections:
      - from: [0, 0]  # Time output pin 0
        to: [1, 3]    # Oscillator input pin 3 (time)
    outputs:
      ty: [1, 0]      # Shape's Y translation comes from Oscillator output 0
```

Shapes without graphs keep the current flat format. Full backward compatibility.

---

## UI Integration

The node editor appears as a new panel (bottom or floating window), toggled via View menu. When a shape is selected and has a graph, the node editor shows it. A "Create Graph" button converts a shape's flat properties into an initial graph with connected nodes matching the current values.

The properties panel (right side) continues to work — it shows the effective values of each property, whether they come from a graph or from flat defaults. Editing a slider when a node is connected could either disconnect the node or be blocked (TBD — Blender blocks, Houdini allows override).
