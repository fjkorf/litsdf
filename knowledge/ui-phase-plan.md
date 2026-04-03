# UI Phase — Feature Status

## Completed

- **Shape YAML Editor Modal** — textarea with Apply/Cancel, serde round-trip
- **Light Positioning UI** — 3 sliders for sun direction (top of properties panel)
- **Scene Rename** — textedit at top of right panel
- **Menu Bar** — File (New/Open/Save), Edit (Undo/Redo/Duplicate/Delete/Deselect), Add (Bone + 9 primitives), View (Gizmos/Frame Selection/Reset Camera/Node Editor)
- **Keyboard Shortcuts** — Cmd+S/O/N/Z/Shift+Z/D, Del, Esc, F — all with menu hints
- **Status Bar** — selection info + scene stats at bottom
- **Visibility Toggles** — eye icon per bone/shape in tree, skips invisible in GPU flatten
- **Right-click Context Menus** — bone: add child, add shape, duplicate, reparent, delete, delete recursive; shape: duplicate, move to bone, delete
- **Camera Pan** — middle-mouse drag to shift target
- **Frame Selection** — F key zooms to selected shape/bone
- **Properties Reorder** — Light at top, Geometry (combine) promoted, Save/Load removed (now in File menu)
- **Reset Transform / Clear Modifiers buttons** — in properties panel
- **Node Editor (Phase A+B+C)** — 21 node types, graph persistence, 7 presets, 27-pin ShapeOutput, 7-pin BoneOutput
- **Graph Persistence** — ProjectFile format bundles scene + graphs (backward compatible)
- **Shader Parameterization** — SceneSettings with tunable fill/back/SSS/AO/shadow/vignette
- **Cellular Noise** — Voronoi pattern as color_mode 3 in shader
- **Chamfer Boolean Ops** — ChamferUnion, ChamferIntersection (8 total ops)
- **Node Color-Coding** — green (generators), teal (oscillators), blue (math), amber (vec3/color), red (output)
- **SceneSettings UI sliders** — Rendering section with fill/back/SSS/AO/shadow/vignette sliders
- **4 new primitives** — Octahedron, Pyramid, HexPrism, RoundCone (13 total)
- **Ridged multifractal noise** — color_mode 4
- **Gradient Snow coloring** — color_mode 5 (normal.y → white)
- **3 animation shaping nodes** — ExpImpulse, SmoothStep, Noise1D (24 total)
- **Graph undo** — snapshots before preset apply/clear
- **Drag-to-reparent** — drag shapes/bones between tree items with visual drop feedback
- **Inline rename** — double-click bone/shape in tree to edit name
- **H/Alt+H shortcuts** — toggle selected visibility / show all
- **Render sequence** — `LITSDF_RENDER_SEQUENCE=dir,frames,fps` for video frame export
- **PBR Lighting** — Cook-Torrance BRDF (GGX + Fresnel-Schlick + Smith geometry)
- **Shader codegen** — unrolled shape evaluation, preamble/body/postamble split, topology-driven recompilation
- **6 demo scenes** — Primitive Gallery, Boolean Sampler, Modifier Parade, Mushroom Garden, Robot Friend, Abstract Sculpture (File > Demo Scenes menu)

## Removed (replaced by node editor)

- Animation sliders (bone bob/sway, shape move/spin/pulse) — use node graphs instead
- `bone animate` and `shape animate` CLI commands — animation is now visual, not CLI-driven
- `clear_animation()` helper and button — clear the node graph instead

## Remaining

### Should Implement
- **Modifier stack redesign** — replace flat sliders with node-driven stack (modifier pins exist)

### Future
- Per-pixel node expression codegen (emit node formulas as inline WGSL)
- Subgraph encapsulation / node groups
- Custom expression nodes
- Copy/paste clipboard for shapes
- Drag to reorder/reparent in tree
- Group selection
- Render sequence (animation frames to disk)
- Inline rename in tree via context menu
- Node preview thumbnails (show computed values on node face)
- Node minimap
