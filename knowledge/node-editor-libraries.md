# Node Editor Libraries for egui

## Recommendation: egui-snarl

**egui-snarl** is the clear choice for litsdf. Active maintenance (pushed 2026-03-31), 537 GitHub stars, clean API, serde support, egui 0.34.

## Library Survey

### egui-snarl (zakarumych) — RECOMMENDED

| Field | Value |
|-------|-------|
| Version | 0.9.0 |
| egui | 0.34 |
| Downloads | 35k total, 5.4k recent |
| Stars | 537 |
| Last push | 2026-03-31 |
| Serde | Yes (feature flag) |

**API Design:** `Snarl<T>` is generic over a single node data type (typically an enum). A `SnarlViewer<T>` trait defines rendering — title, pin count, pin UI, context menus, wire connections. Graph data is separate from rendering logic, making it cleanly serializable.

**Nodes/Ports/Connections:**
- Nodes hold user data `T`, indexed by `NodeId`
- Pins defined dynamically by viewer (input/output count per node)
- Wires connect output pins to input pins
- Multi-connection supported (shift-drag bundles)

**Bevy integration:** Pure egui widget — call `snarl.show(viewer, style, id, ui)` inside any `bevy_egui` panel. No special integration needed.

### egui-graph-edit (kamirr) — Alternative

| Field | Value |
|-------|-------|
| Version | 0.7.2 |
| egui | 0.33 |
| Downloads | 5.5k total |
| Stars | 38 |
| Last push | 2026-01-25 |

Fork of the original egui_node_graph. Heavily trait-based with 5 generic parameters (`NodeData`, `DataType`, `ValueType`, `NodeTemplate`, `CategoryType`). Has built-in node finder menu. Used by Modal (modular synth). Semi-active maintenance.

### egui_node_editor (cyloncore) — GitLab

| Field | Value |
|-------|-------|
| Version | 0.9.0 |
| Downloads | 14k total, 10.5k recent |

Another egui_node_graph fork on GitLab. Strong recent downloads suggest adoption. Could not verify exact egui version (GitLab auth required for raw files).

### egui_graphs (blitzarx1) — NOT a node editor

Graph **visualization** library (force-directed layouts, petgraph). No pins, no wire connections. Not suitable for SDF node editing.

### egui_node_graph (setzer22) — DEAD

Original library. All versions yanked on crates.io. Archived.

### egui_node_graph2 (philpax) — STALE

Low-activity fork. Last updated November 2024. Not recommended.

## Why egui-snarl for litsdf

1. **Single generic parameter** — `Snarl<SdfNode>` where `SdfNode` is our enum. Clean and Rusty.
2. **Viewer trait** — rendering logic separated from data. We can have different viewers for material graphs vs animation graphs.
3. **Serde** — `Snarl` serializes graph topology + node data. Fits our YAML scene format.
4. **egui 0.34** — one version ahead of our current 0.33 (bevy_egui 0.39). May need to align versions when upgrading.
5. **Active** — 17 releases, Discord server, responsive maintainer.
6. **No unsafe** — `#![forbid(unsafe_code)]`.

## Version Compatibility (Resolved)

egui-snarl **0.9.0** is compatible with egui 0.33 (confirmed, in production use with 21 node types, graph persistence, and color-coded node headers). The 0.9.0 version was the right choice — later versions (0.10+) require egui 0.34.
