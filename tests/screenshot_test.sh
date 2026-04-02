#!/bin/bash
# Screenshot integration test — captures screenshots for visual verification.
# Runs the app with different scene files and captures screenshots.
#
# Usage: ./tests/screenshot_test.sh
# Outputs: tests/screenshots/*.png
# Requires: cargo build first (or will build on first run)

set -euo pipefail
cd "$(dirname "$0")/.."

SCREENSHOTS=tests/screenshots
mkdir -p "$SCREENSHOTS"

echo "=== Screenshot Test Suite ==="
echo ""

# 1. Default scene (no scene file)
echo "[1/4] Default scene..."
LITSDF_SCREENSHOT="$SCREENSHOTS/test_default_scene.png" \
  cargo run --bin litsdf --quiet 2>&1 | tail -1
echo "  -> $SCREENSHOTS/test_default_scene.png"

# 2. Load the real nested scene (my_shape_2.yaml)
SCENE_FILE="$HOME/Library/Application Support/litsdf/scenes/my_shape_2.yaml"
if [ -f "$SCENE_FILE" ]; then
  echo "[2/4] Nested scene (my_shape_2.yaml)..."
  LITSDF_SCENE="$SCENE_FILE" \
  LITSDF_SCREENSHOT="$SCREENSHOTS/test_nested_scene.png" \
    cargo run --bin litsdf --quiet 2>&1 | tail -1
  echo "  -> $SCREENSHOTS/test_nested_scene.png"
else
  echo "[2/4] SKIP — $SCENE_FILE not found"
fi

# 3. Programmatic test scene — write a YAML with known structure, screenshot it
echo "[3/4] Programmatic multi-bone scene..."
TMPSCENE=$(mktemp /tmp/litsdf_test_XXXXXX.yaml)
cat > "$TMPSCENE" << 'YAML'
name: Test Scene
root_bone:
  id: 00000000-0000-0000-0000-000000000000
  name: Root
  transform:
    translation: [0.0, 0.0, 0.0]
    rotation: [0.0, 0.0, 0.0]
    scale: 1.0
  children:
    - id: 11111111-1111-1111-1111-111111111111
      name: Left
      transform:
        translation: [-2.0, 0.0, 0.0]
        rotation: [0.0, 0.0, 0.0]
        scale: 1.0
      children: []
      shapes:
        - id: aaaa1111-1111-1111-1111-111111111111
          name: LeftSphere
          primitive: !Sphere
            radius: 0.8
          transform:
            translation: [0.0, 0.0, 0.0]
            rotation: [0.0, 0.0, 0.0]
            scale: 1.0
          material:
            color: [0.9, 0.2, 0.2]
            roughness: 0.5
            metallic: 0.0
          modifiers: []
          combination: Union
    - id: 22222222-2222-2222-2222-222222222222
      name: Right
      transform:
        translation: [2.0, 0.0, 0.0]
        rotation: [0.0, 0.0, 0.0]
        scale: 1.0
      children:
        - id: 33333333-3333-3333-3333-333333333333
          name: RightTip
          transform:
            translation: [1.0, 1.0, 0.0]
            rotation: [0.0, 0.0, 0.0]
            scale: 1.0
          children: []
          shapes:
            - id: bbbb3333-3333-3333-3333-333333333333
              name: TipTorus
              primitive: !Torus
                major_radius: 0.5
                minor_radius: 0.15
              transform:
                translation: [0.0, 0.0, 0.0]
                rotation: [0.0, 0.0, 0.0]
                scale: 1.0
              material:
                color: [0.2, 0.9, 0.3]
                roughness: 0.3
                metallic: 0.0
              modifiers: []
              combination: Union
      shapes:
        - id: bbbb2222-2222-2222-2222-222222222222
          name: RightBox
          primitive: !Box
            half_extents: [0.6, 0.6, 0.6]
          transform:
            translation: [0.0, 0.0, 0.0]
            rotation: [0.0, 0.0, 0.0]
            scale: 1.0
          material:
            color: [0.2, 0.4, 0.9]
            roughness: 0.5
            metallic: 0.0
          modifiers: []
          combination: Union
  shapes:
    - id: aaaa0000-0000-0000-0000-000000000000
      name: CenterSphere
      primitive: !Sphere
        radius: 1.0
      transform:
        translation: [0.0, 0.0, 0.0]
        rotation: [0.0, 0.0, 0.0]
        scale: 1.0
      material:
        color: [0.8, 0.8, 0.8]
        roughness: 0.5
        metallic: 0.0
      modifiers: []
      combination: Union
combination: Union
YAML

LITSDF_SCENE="$TMPSCENE" \
LITSDF_SCREENSHOT="$SCREENSHOTS/test_multi_bone.png" \
  cargo run --bin litsdf --quiet 2>&1 | tail -1
rm -f "$TMPSCENE"
echo "  -> $SCREENSHOTS/test_multi_bone.png"

# 4. Unit tests (non-visual)
echo "[4/4] Unit tests..."
cargo test --workspace --lib --quiet 2>&1 | tail -5

echo ""
echo "=== All screenshots captured ==="
ls -la "$SCREENSHOTS"/test_*.png 2>/dev/null || echo "No test screenshots found"
