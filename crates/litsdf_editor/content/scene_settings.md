---
page:
  name: SceneSettings
  label: Scene Settings
  panel: window
  open: show_scene_settings
  width: 320.0

widgets:
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
  sun_sharpness_cfg:
    min: 4.0
    max: 256.0
    decimals: 0
    label: "Sun sharpness"
  sun_brightness_cfg:
    min: 0.0
    max: 10.0
    decimals: 1
    label: "Sun brightness"
  gravity_cfg:
    min: -20.0
    max: 0.0
    decimals: 1
    label: "Gravity"
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

#### Environment

[color](fill_color) Sky

[color](ground_color) Ground

[slider](sun_sharpness){sun_sharpness_cfg}

[slider](sun_brightness){sun_brightness_cfg}

#### Physics

[slider](gravity){gravity_cfg}
