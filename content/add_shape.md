---
page:
  name: AddShape
  label: Add Shape
  panel: window
  open: show_add_shape

widgets:
  new_shape_type_opts:
    options: ["Sphere", "Box", "RoundBox", "Cylinder", "CappedCone", "Torus", "Capsule", "Plane"]
  on_confirm_add: {}
---

## Add Shape

[combobox](new_shape_type){new_shape_type_opts}

[button](Add){on_confirm_add}
