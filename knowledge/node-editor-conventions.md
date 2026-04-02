# Node Editor Conventions — Industry Research

Research from Blender, Houdini, Unity Shader Graph, Substance Designer, and SDF-specific tools.

---

## 1. Universal Patterns

### Typed Sockets with Automatic Conversion

Every system uses typed connection ports. Common implicit conversions:
- Float → Vector (broadcast to all components)
- Color → Vector (channel mapping)
- Grayscale → Color (replicate)

Invalid connections rejected at link time.

### Default Values on Unconnected Inputs

Every input socket has an editable default value. When unconnected, the UI shows a widget (slider, color picker, value field) directly on the node face. This enables partial graphs — not every input needs a connection.

### Node Groups / Subgraphs

All systems support encapsulation:
- **Blender**: Node Groups (nested bNodeTree)
- **Houdini**: Digital Assets / HDAs
- **Unity**: Sub Graphs (separate asset files)
- **Substance**: Graph Instances (compound nodes from atomics)

Pattern: expose selected internal inputs/outputs as the group interface, hide implementation.

### Preview Thumbnails

Shader-oriented systems show per-node output previews. Substance previews every node's image. Unity compiles preview shaders in background. Blender shows material previews.

### Evaluation Models

Two dominant approaches:
- **Pull-based / lazy** (Blender Geometry Nodes, Houdini): compute only when output is requested. Efficient for large graphs with unused branches.
- **Compilation** (Blender Shaders, Unity Shader Graph, Houdini VOPs): graph compiled to GPU code. "Evaluation" is the compiled program running.
- **Hybrid** (Substance): incremental re-evaluation with caching, only dirty subgraph recomputes.

### Dirty Propagation

When a parameter changes, the node and downstream dependents are marked dirty. Upstream nodes with valid caches are skipped. Universal optimization for interactive editing.

---

## 2. Blender Shader / Geometry Nodes

### Socket Types
Shader: Float, Integer, Boolean, Vector, Rotation, Color, String, Shader.
Geometry Nodes adds: Geometry, Object, Collection, Image, Material, Bundle.

Socket **shapes** encode semantics:
- Circle = single value
- Diamond = field (varies per element, e.g., per vertex)
- Grid = sampled volume/surface data

### GPU Shader Compilation
Shader nodes compile via: (1) graph analysis, (2) stack-based IR via `GPUNodeStack`, (3) backend code generation to GLSL/MSL/SPIR-V. The graph is a code generation DAG.

### Geometry Nodes: Lazy Evaluation
Pull-based using "lazy functions." A node's output computed only when requested by downstream. Supports simulation zones and repeat zones.

### Default Values
Every `bNodeSocket` has `default_value`. When no incoming link, evaluation reads this directly. UI exposes it as widget on the node face.

---

## 3. Houdini VEX/VOPs

### Context Types
- **SOP** — geometry (time-independent)
- **VOP** — visual VEX programming (compiles to VEX)
- **DOP** — simulation (time-dependent)
- **CHOP** — animation channels

Critical distinction: SOPs are functional (same inputs → same outputs), DOPs are stateful (depend on previous timestep).

### Cooking Model
Pull-based, functional: "Nodes are never cooked unless asked for their data." When a downstream node detects a dirty upstream, it triggers recursive cooking. Side effects prohibited during cooking.

### Dual Value Sources
Parameters driven by either:
- **Node connections** (wires in the graph)
- **Parameter expressions** (HScript/Python, e.g., `$F` for frame, `ch("../other/param")`)

This means values can be animated via keyframes or driven by expressions without explicit graph connections.

### VOP → VEX Compilation
VOP networks linearized into VEX code at cook time. Each VOP node maps to a VEX function. Compiled code runs in parallel across geometry elements.

---

## 4. Unity Shader Graph

### Master Stack
Modular output node with vertex and fragment stage blocks:
- Vertex: Position, Normal, Tangent
- Fragment: Base Color, Metallic, Smoothness, Normal, Emission, Alpha

Different render pipelines (URP, HDRP) expose different block sets.

### Property Types (become shader uniforms)
Float, Vector2/3/4, Color (HDR), Boolean, Texture2D/3D, Cubemap, Gradient, Matrix.
Each has a Reference Name becoming the HLSL variable.

### Code Generation
Each node has a code generation function emitting HLSL snippets. Custom Function Node allows inline HLSL or external file include. Final output is a complete shader file.

### Sub Graphs
Reusable node groups saved as separate assets. Can be nested. Auto-generates properties for disconnected inputs.

---

## 5. Substance Designer

### Two-Tier Node Architecture
- **Atomic nodes** (26): engine-native operations (Blend, Levels, Transform, Gradient Map...)
- **Compound nodes** (200+): built entirely from atomics, reusable subgraphs

The engine only implements 26 operations; all complexity emerges from composition.

### Connection Types
- Color connections (RGBA image data)
- Grayscale connections (single-channel image)
- Value connections (scalar/vector, not images)

### Resolution Propagation
Images carry metadata (resolution, bit depth, tiling). Attributes propagate via inheritance methods. Set at graph level, nodes inherit unless overridden.

### Function Graphs
Separate from compositing graphs. Process single values (int, float, vector) for driving parameters procedurally.

---

## 6. SDF-Specific Node Systems

### Existing Tools
- **Womp 3D** — browser SDF modeler with "Goop" smooth blending. Layer/hierarchy UI, not graph-based.
- **Material Maker** — open-source (Godot), dedicated 3D SDF nodes: shapes, boolean ops, render
- **b3dsdf** — Blender addon with 170+ SDF/vector shader node groups
- **fogleman/sdf** — Python library: `SDF3` class with chainable methods, function composition

### SDF Composition as Node Graph
SDF booleans map to a binary tree/DAG:
- **Leaf nodes** = primitives (sphere, box...) with transform parameters
- **Internal nodes** = operators (union, subtraction, intersection, smooth blend)
- **Evaluation** = recursive: operator calls children, applies min/max/smooth-min

The entire SDF scene is a single function composed from sub-functions. The node graph is literally a visual function composition diagram.

### SDF-Specific Challenges

1. **Order dependence**: Union/intersection are commutative, subtraction is not (`A-B ≠ B-A`). Graph must preserve operand order.
2. **Smooth blend radius**: affects distance field globally near blend region. Changing one node visually affects distant geometry.
3. **Evaluation cost scales with complexity**: SDF ray marching evaluates the entire function tree per ray step. Node count is performance-critical.
4. **No natural LOD**: can't simplify an SDF tree like decimating a mesh.
5. **Transform inversion**: SDF transforms work "backwards" — transform the query point, not the shape. Node system must invert internally.
6. **Material at boundaries**: smooth blending requires material interpolation, carrying material IDs through the distance field.

---

## 7. Implications for litsdf Node Editor

### Socket Types Needed
- **Distance** (f32) — SDF field value
- **Position** (Vec3) — spatial coordinates
- **Color** (Vec3) — RGB material color
- **Float** (f32) — scalar parameter
- **Transform** (Mat4 or translation+rotation+scale)

### Evaluation Model
**Compilation to WGSL**, not runtime graph traversal. SDF evaluation is per-pixel in the ray marcher. The node graph becomes a code generation DAG — same approach as Blender shaders and Unity Shader Graph.

### Key Design Decisions
1. Default values on every input (editable when unconnected)
2. Order-aware connections for subtraction nodes
3. Performance indicator (node count → ray march cost)
4. Subgraph support for reusable compositions
5. Dual-context: some nodes evaluate per-frame CPU (bone transforms), others per-pixel GPU (SDF/material)
