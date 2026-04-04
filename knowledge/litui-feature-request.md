# litui Feature Request: Enhanced Numeric Input Configuration

## Context

litsdf uses litui for its properties panel with 44 numeric inputs (sliders and drag values). Several egui capabilities that would improve the UX are not exposed through litui's frontmatter configuration.

## Requested Features

### 1. Integer Mode for Sliders

**Problem:** `noise_octaves` is a `u32` loop count (0-6) displayed as a continuous slider. Users can set fractional values (e.g., 3.7) that get truncated, which is confusing.

**egui supports:** `Slider::new(...).integer()` which calls `.fixed_decimals(0).smallest_positive(1.0).step_by(1.0)`

**Requested litui config:**
```yaml
noise_oct_cfg:
  min: 0
  max: 6
  integer: true
  label: "Noise (layers)"
```

**Implementation:** In codegen, when `integer: true`, emit `.integer()` after the slider constructor. Alternatively, detect integer state field types automatically.

### 2. Step/Quantization for Sliders

**Problem:** Some parameters benefit from discrete steps (e.g., rotation in 5-degree increments).

**egui supports:** `Slider::new(...).step_by(5.0)`

**Requested litui config:**
```yaml
rotation_cfg:
  min: -180
  max: 180
  step: 5.0
  label: "Rotation"
```

### 3. Range Support for DragValue

**Problem:** DragValue currently only exposes `speed` in litui. Position inputs need unbounded drag but some drag values should be clamped (e.g., roughness should stay 0-1 even as a drag value).

**egui supports:** `DragValue::new(...).range(0.0..=1.0)`

**Requested litui config:**
```yaml
roughness_drag_cfg:
  min: 0.0
  max: 1.0
  speed: 0.01
  label: "Roughness"
```

Currently `min`/`max` are only used by sliders. Extending them to dragvalue would allow constrained drag inputs.

### 4. Suffix/Prefix for DragValue

**Problem:** Slider already supports suffix/prefix (e.g., "°" for degrees). DragValue should too.

**egui supports:** `DragValue::new(...).suffix("°")`

**Requested litui config:**
```yaml
angle_cfg:
  speed: 0.5
  suffix: "°"
  label: "Angle"
```

### 5. Decimal Precision Control

**Problem:** Some values should show 0 decimals (integers), some 1 (roughness), some 3 (small noise amplitudes). Currently all use egui's default auto-precision.

**egui supports:** `Slider/DragValue::new(...).fixed_decimals(2)` or `.max_decimals(3)`

**Requested litui config:**
```yaml
precise_cfg:
  min: 0.0
  max: 0.1
  decimals: 3
  label: "Smooth symmetry"
```

## Priority

1. **Integer mode** — highest impact, simplest to implement (one `.integer()` call)
2. **DragValue range** — reuse existing min/max parsing, just wire to DragValue
3. **Suffix/prefix on DragValue** — reuse existing suffix/prefix parsing
4. **Step quantization** — new config key, straightforward
5. **Decimal precision** — new config key, straightforward
