# Shader Code Generation from Node Graphs

Research into compiling visual node graphs into per-pixel WGSL shader code.

## Architecture: String-Template Codegen

Three patterns exist in the industry. **String concatenation** (Godot's approach) is the right fit for litsdf:

1. Walk the SDF scene tree depth-first
2. For each shape: emit `let d_N = sd_primitive(q_N, params_N);` with that shape's transform applied
3. For each combination: emit `let d_N = op_combine(d_left, d_right, k);`
4. Root node's variable is the scene distance

The SDF tree IS the AST — no intermediate representation needed.

**Key advantage:** Generated shader has no dynamic dispatch (`switch` on primitive type), no loop with `shape_count`, no array indexing. GPU compiler can inline everything and optimize register allocation.

## Bevy Runtime Shader Injection

**Confirmed working in Bevy 0.18:**

```rust
let mut shaders = world.resource_mut::<Assets<Shader>>();
shaders.insert(&handle, Shader::from_wgsl(generated_string, file_path));
```

When the shader asset changes, Bevy detects it and recompiles the render pipeline automatically. The `Material` trait's `fragment_shader()` returns a `ShaderRef` which can be a handle to a programmatically-created shader.

**When to recompile:** Only when scene topology changes (add/remove/reorder shapes). Parameter changes (position, color, roughness) still flow through the existing uniform buffer — no shader recompilation needed.

## What Changes vs Current Architecture

| Current | Codegen |
|---------|---------|
| Shapes in uniform array `shapes[32]` | Shapes inlined as named variables `d_0`, `d_1`... |
| `switch` on `primitive_type` per eval | Direct function call `sd_sphere(q, 1.0)` |
| Loop `for i in 0..shape_count` | Unrolled: `let d = op_union(d_0, d_1)` |
| 32-shape limit (uniform buffer) | No limit (shader size scales with scene) |
| Parameter changes = uniform update | Parameter changes = uniform update (same) |
| Topology changes = dirty flag | Topology changes = shader recompile |

## Implementation Plan

**Phase 1: Template-based codegen module**
- New module `litsdf_core::codegen` (or `litsdf_render::codegen`)
- Function: `generate_sdf_shader(scene: &SdfScene) -> String`
- Fixed preamble (structs, imports, noise, primitives, lighting) + generated `sdf_scene()`/`sdf_scene_material()`
- Keep existing uniform buffer for per-shape parameters

**Phase 2: Runtime shader replacement**
- On scene topology change, regenerate shader and insert into `Assets<Shader>`
- Cache the compiled shader — only regenerate when shapes/bones/combinations change
- Fall back to the loop-based shader if codegen fails

**Phase 3: Node graph integration**
- The node graph already produces shape property values per-frame
- For per-pixel nodes (noise color, cosine palette), the codegen emits the node's formula inline in WGSL instead of using the uniform `color_mode` switch

## Naga IR: Not Recommended

Naga's `Module` struct can be constructed programmatically, but it's dramatically more verbose than string templates. Every expression, type, and statement must be manually constructed via arena handles. String-template generation that feeds into `Shader::from_wgsl()` is simpler — naga validates the string anyway during wgpu compilation.

## Gotchas

- `Material::fragment_shader()` is a static method — cannot return different shaders per material instance. Use a single handle that gets replaced.
- Shader recompilation takes a frame or two (pipeline re-specialization is async). Brief visual glitch during topology changes.
- Uniform buffer limits (~64KB) are not an issue at current scale (8KB for 32 shapes) but codegen eliminates this constraint entirely.
