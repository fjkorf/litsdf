# litui Numeric Config Features — All Implemented

All 5 requested features have been implemented in litui and are in use by litsdf.

## Features

| Feature | Config | egui Method | Status |
|---------|--------|-------------|--------|
| Integer slider | `integer: true` | `.integer()` | **Implemented** |
| Step quantization | `step: 5.0` | `.step_by(5.0)` | **Implemented** |
| Fixed decimals | `decimals: 3` | `.fixed_decimals(3)` | **Implemented** |
| DragValue range | `min: 0, max: 1` | `.range(0.0..=1.0)` | **Implemented** |
| DragValue suffix/prefix | `suffix: "°"` | `.suffix("°")` | **Implemented** |

## Usage in litsdf

```yaml
# Integer slider (noise octaves)
noise_oct_cfg: { min: 0, max: 6, integer: true, label: "Noise (layers)" }

# Rotation with step and suffix
rx_cfg: { min: -180, max: 180, step: 1.0, suffix: "°", label: "Pitch (X rot)" }

# Material with precision
roughness_cfg: { min: 0.0, max: 1.0, decimals: 2, label: "Roughness" }

# Small value precision
symmetry_cfg: { min: 0.0, max: 0.1, decimals: 3, label: "Mirror (smoothness)" }

# Position drag value with precision
tx_cfg: { speed: 0.05, decimals: 2, label: "X position" }
```
