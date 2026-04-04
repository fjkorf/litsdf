---
page:
  name: Properties
  label: Properties
  panel: right
  width: 260.0
  default: true

widgets:
  prim_type_opts:
    options: ["Sphere", "Box", "RoundBox", "Cylinder", "CappedCone", "Torus", "Capsule", "Plane", "Ellipsoid", "Octahedron", "Pyramid", "HexPrism", "RoundCone"]
  combo_op_opts:
    options: ["Union", "Intersection", "Subtraction", "SmoothUnion", "SmoothIntersection", "SmoothSubtraction", "ChamferUnion", "ChamferIntersection"]
  tx_cfg:
    speed: 0.05
    label: X position
  ty_cfg:
    speed: 0.05
    label: Y position
  tz_cfg:
    speed: 0.05
    label: Z position
  rx_cfg:
    min: -180
    max: 180
    label: Pitch (X rot)
  ry_cfg:
    min: -180
    max: 180
    label: Yaw (Y rot)
  rz_cfg:
    min: -180
    max: 180
    label: Roll (Z rot)
  scale_cfg:
    min: 0.1
    max: 5.0
    label: Uniform scale
  param_a_cfg:
    min: 0.01
    max: 5.0
    label: "A (size/radius)"
  param_b_cfg:
    min: 0.01
    max: 5.0
    label: "B (width/radius)"
  param_c_cfg:
    min: 0.01
    max: 5.0
    label: "C (depth/r2)"
  param_d_cfg:
    min: 0.0
    max: 2.0
    label: "D (rounding)"
  smooth_k_cfg:
    min: 0.0
    max: 2.0
    label: Blend radius (k)
  roughness_cfg:
    min: 0.0
    max: 1.0
    label: Roughness
  metallic_cfg:
    min: 0.0
    max: 1.0
    label: Metallic
  fresnel_cfg:
    min: 0.0
    max: 5.0
    label: Rim exponent
  color_mode_opts:
    options: ["Solid", "Cosine Palette", "Noise Tint", "Cellular", "Ridged", "Gradient Snow"]
  noise_amp_cfg:
    min: 0.0
    max: 0.5
    label: "Noise (roughness)"
  noise_freq_cfg:
    min: 0.1
    max: 20.0
    label: "Noise (detail)"
  noise_oct_cfg:
    min: 0
    max: 6
    label: "Noise (layers)"
  symmetry_cfg:
    min: 0.0
    max: 0.1
    label: "Mirror (smoothness)"
  rounding_cfg:
    min: 0.0
    max: 1.0
    label: "Rounding (edge radius)"
  onion_cfg:
    min: 0.0
    max: 0.5
    label: "Shell (wall thickness)"
  twist_cfg:
    min: -5.0
    max: 5.0
    label: "Twist (Y-axis warp)"
  bend_cfg:
    min: -5.0
    max: 5.0
    label: "Bend (X-axis warp)"
  elong_x_cfg:
    min: 0.0
    max: 2.0
    label: Elongate X
  elong_y_cfg:
    min: 0.0
    max: 2.0
    label: Elongate Y
  elong_z_cfg:
    min: 0.0
    max: 2.0
    label: Elongate Z
  rep_x_cfg:
    min: 0.0
    max: 5.0
    label: Repeat X (period)
  rep_y_cfg:
    min: 0.0
    max: 5.0
    label: Repeat Y (period)
  rep_z_cfg:
    min: 0.0
    max: 5.0
    label: Repeat Z (period)
  bone_tx_cfg:
    speed: 0.05
    label: X position
  bone_ty_cfg:
    speed: 0.05
    label: Y position
  bone_tz_cfg:
    speed: 0.05
    label: Z position
  bone_rx_cfg:
    min: -180
    max: 180
    label: Pitch (X rot)
  bone_ry_cfg:
    min: -180
    max: 180
    label: Yaw (Y rot)
  bone_rz_cfg:
    min: -180
    max: 180
    label: Roll (Z rot)
  bone_name_cfg:
    hint: Bone name
  on_select_shape: {}
  on_delete_shape: {}
  on_edit_yaml: {}
  on_reset_transform: {}
  on_clear_modifiers: {}
  scene_name_cfg:
    hint: Scene name
  light_x_cfg:
    min: -1.0
    max: 1.0
    label: "Sun X (left/right)"
  light_y_cfg:
    min: 0.0
    max: 1.0
    label: "Sun Y (height)"
  light_z_cfg:
    min: -1.0
    max: 1.0
    label: "Sun Z (front/back)"
  fill_intensity_cfg:
    min: 0.0
    max: 1.0
    label: "Fill light"
  back_intensity_cfg:
    min: 0.0
    max: 1.0
    label: "Back light"
  sss_intensity_cfg:
    min: 0.0
    max: 1.0
    label: "SSS glow"
  ao_intensity_cfg:
    min: 0.0
    max: 5.0
    label: "Ambient occlusion"
  shadow_softness_cfg:
    min: 1.0
    max: 32.0
    label: "Shadow softness"
  vignette_cfg:
    min: 0.0
    max: 1.0
    label: "Vignette"
---

[textedit](scene_name){scene_name_cfg}

#### Light Direction

[slider](light_x){light_x_cfg}

[slider](light_y){light_y_cfg}

[slider](light_z){light_z_cfg}

#### Rendering

[slider](fill_intensity){fill_intensity_cfg}

[slider](back_intensity){back_intensity_cfg}

[slider](sss_intensity){sss_intensity_cfg}

[slider](ao_intensity){ao_intensity_cfg}

[slider](shadow_softness){shadow_softness_cfg}

[slider](vignette_intensity){vignette_cfg}

---

## Properties

::: if has_bone_selection

::: if bone_editable

[textedit](bone_name){bone_name_cfg}

#### Bone Transform

[dragvalue](bone_tx){bone_tx_cfg}

[dragvalue](bone_ty){bone_ty_cfg}

[dragvalue](bone_tz){bone_tz_cfg}

[slider](bone_rx){bone_rx_cfg}

[slider](bone_ry){bone_ry_cfg}

[slider](bone_rz){bone_rz_cfg}

:::

::: if bone_is_root

*Root bone (fixed at origin)*

:::

#### Shapes

::: foreach bone_shapes

| {shape_name} | [button](>){on_select_shape} |
|---|---|

:::

::: if has_selection

### [display](selected_shape_name)

#### Primitive

[combobox](prim_type){prim_type_opts}

[slider](param_a){param_a_cfg}

[slider](param_b){param_b_cfg}

[slider](param_c){param_c_cfg}

[slider](param_d){param_d_cfg}

#### Geometry

[combobox](combo_op){combo_op_opts}

[slider](smooth_k){smooth_k_cfg}

#### Transform

[dragvalue](tx){tx_cfg}

[dragvalue](ty){ty_cfg}

[dragvalue](tz){tz_cfg}

[slider](rx){rx_cfg}

[slider](ry){ry_cfg}

[slider](rz){rz_cfg}

[slider](uniform_scale){scale_cfg}

[button](Reset_Transform){on_reset_transform}

#### Material

[color](shape_color)

[slider](roughness){roughness_cfg}

[slider](metallic){metallic_cfg}

[slider](fresnel_power){fresnel_cfg}

[combobox](color_mode){color_mode_opts}

::: if is_palette_mode

[color](palette_a_color) Bias

[color](palette_b_color) Amplitude

[color](palette_c_color) Frequency

[color](palette_d_color) Phase

:::

#### Noise

[slider](noise_amp){noise_amp_cfg}

[slider](noise_freq){noise_freq_cfg}

[slider](noise_oct){noise_oct_cfg}

#### Symmetry

[slider](smooth_sym){symmetry_cfg}

#### Modifiers

[slider](mod_rounding){rounding_cfg}

[slider](mod_onion){onion_cfg}

[slider](mod_twist){twist_cfg}

[slider](mod_bend){bend_cfg}

[slider](mod_elong_x){elong_x_cfg}

[slider](mod_elong_y){elong_y_cfg}

[slider](mod_elong_z){elong_z_cfg}

[slider](mod_rep_x){rep_x_cfg}

[slider](mod_rep_y){rep_y_cfg}

[slider](mod_rep_z){rep_z_cfg}

[button](Clear_Modifiers){on_clear_modifiers}

---

[button](Delete_Shape){on_delete_shape} [button](Edit_YAML){on_edit_yaml}

:::

:::

::: if no_selection

*Select a bone to view its shapes.*

:::
