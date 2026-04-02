---
page:
  name: Properties
  label: Properties
  panel: right
  width: 260.0
  default: true

widgets:
  prim_type_opts:
    options: ["Sphere", "Box", "RoundBox", "Cylinder", "CappedCone", "Torus", "Capsule", "Plane", "Ellipsoid"]
  combo_op_opts:
    options: ["Union", "Intersection", "Subtraction", "SmoothUnion", "SmoothIntersection", "SmoothSubtraction"]
  tx_cfg:
    min: -10
    max: 10
    label: X
  ty_cfg:
    min: -10
    max: 10
    label: Y
  tz_cfg:
    min: -10
    max: 10
    label: Z
  rx_cfg:
    min: -180
    max: 180
    label: RotX
  ry_cfg:
    min: -180
    max: 180
    label: RotY
  rz_cfg:
    min: -180
    max: 180
    label: RotZ
  scale_cfg:
    min: 0.1
    max: 5.0
    label: Scale
  param_a_cfg:
    min: 0.01
    max: 5.0
    label: A
  param_b_cfg:
    min: 0.01
    max: 5.0
    label: B
  param_c_cfg:
    min: 0.01
    max: 5.0
    label: C
  param_d_cfg:
    min: 0.0
    max: 2.0
    label: D
  smooth_k_cfg:
    min: 0.0
    max: 2.0
    label: k
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
    label: Rim
  color_mode_opts:
    options: ["Solid", "Cosine Palette", "Noise Tint"]
  noise_amp_cfg:
    min: 0.0
    max: 0.5
    label: Amplitude
  noise_freq_cfg:
    min: 0.1
    max: 20.0
    label: Frequency
  noise_oct_cfg:
    min: 0
    max: 6
    label: Octaves
  symmetry_cfg:
    min: 0.0
    max: 0.1
    label: Smooth
  rounding_cfg:
    min: 0.0
    max: 1.0
    label: Rounding
  onion_cfg:
    min: 0.0
    max: 0.5
    label: Shell
  twist_cfg:
    min: -5.0
    max: 5.0
    label: Twist
  bend_cfg:
    min: -5.0
    max: 5.0
    label: Bend
  elong_cfg:
    min: 0.0
    max: 2.0
    label: E
  rep_cfg:
    min: 0.0
    max: 5.0
    label: Rep
  anim_amp_cfg:
    min: 0.0
    max: 2.0
    label: Amp
  anim_freq_cfg:
    min: 0.0
    max: 5.0
    label: Hz
  bone_tx_cfg:
    min: -10
    max: 10
    label: X
  bone_ty_cfg:
    min: -10
    max: 10
    label: Y
  bone_tz_cfg:
    min: -10
    max: 10
    label: Z
  bone_rx_cfg:
    min: -180
    max: 180
    label: RotX
  bone_ry_cfg:
    min: -180
    max: 180
    label: RotY
  bone_rz_cfg:
    min: -180
    max: 180
    label: RotZ
  bone_name_cfg:
    hint: Bone name
  on_select_shape: {}
  on_delete_shape: {}
  on_edit_yaml: {}
  scene_name_cfg:
    hint: Scene name
  light_x_cfg:
    min: -1.0
    max: 1.0
    label: LX
  light_y_cfg:
    min: 0.0
    max: 1.0
    label: LY
  light_z_cfg:
    min: -1.0
    max: 1.0
    label: LZ
  on_save: {}
  on_load: {}
---

[textedit](scene_name){scene_name_cfg}

---

## Properties

::: if has_bone_selection

::: if bone_editable

[textedit](bone_name){bone_name_cfg}

#### Bone Transform

[slider](bone_tx){bone_tx_cfg}

[slider](bone_ty){bone_ty_cfg}

[slider](bone_tz){bone_tz_cfg}

[slider](bone_rx){bone_rx_cfg}

[slider](bone_ry){bone_ry_cfg}

[slider](bone_rz){bone_rz_cfg}

#### Bone Animation

[slider](bone_anim_ty_amp){anim_amp_cfg}

[slider](bone_anim_ty_freq){anim_freq_cfg}

[slider](bone_anim_ry_amp){anim_amp_cfg}

[slider](bone_anim_ry_freq){anim_freq_cfg}

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

#### Transform

[slider](tx){tx_cfg}

[slider](ty){ty_cfg}

[slider](tz){tz_cfg}

[slider](rx){rx_cfg}

[slider](ry){ry_cfg}

[slider](rz){rz_cfg}

[slider](uniform_scale){scale_cfg}

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

[slider](mod_elong_x){elong_cfg}

[slider](mod_elong_y){elong_cfg}

[slider](mod_elong_z){elong_cfg}

[slider](mod_rep_x){rep_cfg}

[slider](mod_rep_y){rep_cfg}

[slider](mod_rep_z){rep_cfg}

#### Animation

[slider](anim_tx_amp){anim_amp_cfg}

[slider](anim_tx_freq){anim_freq_cfg}

[slider](anim_ty_amp){anim_amp_cfg}

[slider](anim_ty_freq){anim_freq_cfg}

[slider](anim_ry_amp){anim_amp_cfg}

[slider](anim_ry_freq){anim_freq_cfg}

[slider](anim_scale_amp){anim_amp_cfg}

[slider](anim_scale_freq){anim_freq_cfg}

#### Combine

[combobox](combo_op){combo_op_opts}

[slider](smooth_k){smooth_k_cfg}

[button](Delete_Shape){on_delete_shape} [button](Edit_YAML){on_edit_yaml}

:::

:::

::: if no_selection

*Select a bone to view its shapes.*

:::

---

#### Light Direction

[slider](light_x){light_x_cfg}

[slider](light_y){light_y_cfg}

[slider](light_z){light_z_cfg}

---

::: horizontal

[button](Save){on_save} [button](Load){on_load}

:::
