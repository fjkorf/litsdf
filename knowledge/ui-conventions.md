# UI Conventions — What Users Expect

Research findings from Blender, Unity, Houdini, MagicaCSG, Maya, and 3ds Max. Describes the standard patterns that litsdf should follow.

---

## 1. Panel Layout Convention

**Universal pattern across all 3D editors:**

```
┌──────────────────────────────────────────────────────────────┐
│ Menu Bar: File  Edit  Select  Add  View  Help                │
├──────────┬──────────────────────────────────┬────────────────┤
│ Tools    │                                  │ Hierarchy      │
│ (left)   │       3D Viewport                │ (top-right)    │
│          │       (dominates screen)         │                │
│ optional │                                  ├────────────────┤
│          │                                  │ Properties     │
│          │                                  │ (bottom-right) │
├──────────┴──────────────────────────────────┴────────────────┤
│ Timeline / Status Bar (bottom)                                │
└──────────────────────────────────────────────────────────────┘
```

**litsdf current:** Left = bone tree, Center = viewport, Right = properties, Top = menu bar (File/Edit/Add/View), Bottom = status bar. Missing: timeline.

---

## 2. Properties Panel Section Order

**Blender convention (top to bottom, broad → specific):**
1. Scene/World properties (global settings)
2. Object properties (which object is selected)
3. Transform (position, rotation, scale) — **always visible first**
4. Modifiers (ordered stack)
5. Constraints
6. Object data (mesh-specific)
7. Material/Shader

**Unity convention (top to bottom on Inspector):**
1. Transform — **always first, always present**
2. Components in add order (each collapsible with enable checkbox)
3. "Add Component" button at bottom

**litsdf current:** Scene name + light at top/bottom of right panel. When bone selected: bone name → bone transform → bone animation → shape list. When shape selected: primitive → transform → material → noise → symmetry → modifiers → animation → combine.

**Recommended order for litsdf properties:**
1. **Scene** — name, light direction (always visible at top)
2. **Bone** (when bone selected) — name, transform, animation
3. **Shapes list** (under selected bone)
4. **Shape** (when shape selected):
   - **Identity** — name, primitive type
   - **Transform** — position, rotation, scale
   - **Geometry** — primitive params (A/B/C/D), combination op, blend radius
   - **Material** — color, roughness, metallic, fresnel, color mode, palette
   - **Surface** — noise displacement, smooth symmetry
   - **Modifiers** — rounding, shell, twist, bend, elongation, repetition (as a stack)
   - **Animation** — move, spin, pulse channels
5. **Actions** — delete, edit YAML (at bottom)

---

## 3. Menu Bar Categories

**Standard order across Maya, Blender, 3ds Max:**

| Position | Menu | Contents for litsdf |
|----------|------|---------------------|
| 1 | **File** | New Scene, Save, Load, List Scenes, Export, Quit |
| 2 | **Edit** | Undo, Redo, Duplicate, Delete, Select All, Deselect |
| 3 | **Add** | Add Bone, Add Shape (submenu: Sphere, Box, etc.) |
| 4 | **Select** | Select Bone/Shape, Frame Selection, Select by Type |
| 5 | **View** | Toggle Bone Gizmos, Toggle Grid, Reset Camera, Front/Side/Top views |
| 6 | **Help** | Documentation, About, Keyboard Shortcuts |

**litsdf current:** Menu bar with File (New/Open/Save), Edit (Undo/Redo/Duplicate/Delete/Deselect), Add (Bone + 9 primitives), View (Bone Gizmos/Frame Selection/Reset Camera). All with keyboard shortcut hints.

---

## 4. Keyboard Shortcuts

### Universal (all editors)

| Key | Action | Priority for litsdf |
|-----|--------|---------------------|
| Ctrl/Cmd+Z | Undo | **Implemented** |
| Ctrl/Cmd+Shift+Z | Redo | **Implemented** |
| Ctrl/Cmd+S | Save | **Implemented** |
| Ctrl/Cmd+O | Open/Load | **Implemented** |
| Ctrl/Cmd+N | New Scene | **Implemented** |
| Delete/Backspace | Delete selected | **Implemented** |
| Ctrl/Cmd+D | Duplicate | **Implemented** |
| A | Select All | Not applicable (single select) |
| Escape | Deselect / Cancel | **Implemented** |

### 3D-specific (Blender-influenced)

| Key | Action | Priority for litsdf |
|-----|--------|---------------------|
| G | Grab/Move (translate gizmo mode) | **Implemented** |
| R | Rotate (rotate gizmo mode) | **Implemented** |
| S | Scale (enter scale mode) | Future |
| F | Frame selection (zoom to fit) | **Implemented** |
| H | Hide selected | **Implemented** |
| Alt+H | Unhide all | **Implemented** |
| Shift+A | Add menu popup | Future |
| N | Toggle properties panel | Future |
| Numpad 1/3/7 | Front/Side/Top view | Future |
| Numpad 5 | Toggle perspective/ortho | Future |
| Middle mouse | Pan camera | **Implemented** |

---

## 5. Modifier Stack Pattern

**Blender convention:**
- Modifiers are a **linear ordered list** (stack)
- Evaluation order: top to bottom
- Each modifier has:
  - Name (editable)
  - **Enable/disable toggle** (checkbox in header)
  - **Viewport visibility toggle** (eye icon)
  - **Reorder controls** (drag handle or up/down arrows)
  - **Delete button** (X)
  - Collapsible parameter body
- Categories: Generate, Modify, Deform, Physics
- "Add Modifier" dropdown at top of stack

**litsdf current:** Modifiers are flat sliders (Rounding, Shell, Twist, Bend, Elongation, Repetition). No stack, no ordering, no enable/disable. All are always visible.

**Recommended for litsdf:**
- Show modifiers as a collapsible stack with per-modifier enable/disable
- Map to categories:
  - **Shape** (Rounding, Shell/Onion)
  - **Deform** (Twist, Bend)
  - **Domain** (Elongation, Repetition, Symmetry)
- Allow reordering (affects evaluation order in shader)
- "Add Modifier" button with type picker

---

## 6. Material / Color Editor Pattern

**Blender convention:**
- Material is a separate tab in Properties
- Multiple material slots per object
- Each material has: Surface shader, Color, Roughness, Metallic, Normal, Emission
- Color can be a solid value, image texture, or procedural node

**Unity convention:**
- Material is a component (MeshRenderer + Material reference)
- Shader dropdown + per-shader properties
- Color picker for base color, sliders for roughness/metallic

**MagicaCSG convention:**
- Toggle between Diffuse and Metal modes
- Simple color picker
- Per-stroke material (not per-vertex)

**litsdf current:** Color + roughness + metallic + fresnel + color_mode + palette inline in the properties panel. No separate material editor or material library.

**Recommended:**
- Keep inline for now (SDF editor is simpler than mesh editors)
- Add material presets/library as future feature
- Group material properties under a clear "Material" section header

---

## 7. Scene Hierarchy / Outliner Pattern

**Blender Outliner:**
- Tree view of all scene objects
- Each item has: visibility toggle (eye), selectability toggle, renderability toggle
- Drag to reparent objects
- Right-click context menu: Rename, Delete, Duplicate, Select Hierarchy

**Unity Hierarchy:**
- Tree view of GameObjects
- Drag to reparent
- Right-click: Create Empty, Rename, Duplicate, Delete, Copy/Paste

**MagicaCSG:**
- Layer > Stroke flat list
- Eye icon per layer and per stroke
- Right-click for rename

**litsdf current:** egui CollapsingHeader tree showing bones with shapes underneath. Click to select. Per-bone and per-shape visibility toggles (👁 eye icon). Right-click context menus (Add Child Bone, Add Shape, Duplicate, Delete). No drag-to-reparent.

**Remaining gaps:**
- Drag to reorder/reparent (complex but high value)
- Inline rename via context menu

---

## 8. How litsdf Compares

| Convention | Industry Standard | litsdf Status | Gap |
|------------|------------------|---------------|-----|
| Center viewport | Yes | **Yes** | — |
| Right-side properties | Yes | **Yes** | — |
| Left-side hierarchy | Yes (Blender outliner) | **Yes** (bone tree) | — |
| Transform always first | Yes | **Partial** (under bone/shape sections) | Should promote |
| Menu bar | Yes (all editors) | **Yes** (File/Edit/Add/View) | — |
| Keyboard shortcuts | Yes (Ctrl+S, Delete, etc.) | **Yes** (Cmd+S/O/N/Z/D, Del, Esc, F) | — |
| Modifier stack | Yes (ordered, enable/disable) | **Flat sliders** | Should redesign |
| Visibility toggles in tree | Yes (eye icon) | **Yes** (👁 per bone/shape) | — |
| Right-click context menu | Yes (all editors) | **Yes** (add, duplicate, delete) | — |
| Status bar | Yes (most editors) | **Yes** (selection + scene stats) | — |
| Timeline | Yes (animation editors) | **Missing** | Future (procedural anim, no keyframes) |
| Middle-mouse pan | Yes (all 3D editors) | **Yes** | — |
| Frame selection (F key) | Yes (Blender, Maya) | **Yes** | — |

---

## 9. Recommended Action Groups for litsdf

Based on the research, here's how litsdf's 76 actions should map to conventional UI groups:

### Menu Bar
```
File: New, Save, Load, List, Export, Quit
Edit: Undo, Redo, Duplicate, Delete, Select All, Deselect
Add:  Bone, Shape > (Sphere, Box, ..., Ellipsoid)
View: Bone Gizmos, Translation Handles, Compass, Frame Selection, Reset Camera
Help: Keyboard Shortcuts, About
```

### Left Panel (Hierarchy)
- Bone tree with collapsible nodes
- Shapes listed under each bone
- Eye icon per bone/shape for visibility
- Right-click: Rename, Duplicate, Delete, Reparent

### Right Panel (Properties)
```
[Scene Name]
─────────────
[Bone Properties]  (when bone selected)
  Name
  Transform: X, Y, Z, Pitch, Yaw, Roll
  Animation: Bob, Sway
─────────────
[Shape List]  (bone's shapes)
─────────────
[Shape Properties]  (when shape selected)
  Identity: Name, Primitive Type
  Transform: X, Y, Z, Pitch, Yaw, Roll, Scale
  Geometry: Params A-D, Combine Op, Blend K
  Material: Color, Roughness, Metallic, Rim, Color Mode, Palette
  Surface: Noise, Symmetry
  Modifiers: [Stack with enable/disable/reorder]
  Animation: Move X/Y, Spin Y, Pulse
  [Delete] [Edit YAML]
─────────────
[Light Direction]
[Save] [Load]
```

### Viewport
- Left drag: Orbit camera
- Middle drag: Pan camera
- Scroll: Zoom
- Left click: Pick shape
- Left drag on handle: Translate shape
- F key: Frame selection
- Delete key: Delete selected
- Escape: Deselect
