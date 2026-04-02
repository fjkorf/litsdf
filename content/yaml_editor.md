---
page:
  name: YamlEditor
  label: YAML Editor
  panel: window
  open: show_yaml_editor
  width: 450.0

widgets:
  yaml_text_cfg:
    hint: "Shape YAML..."
    rows: 20
  on_apply_yaml: {}
---

## Shape YAML

[textarea](yaml_text){yaml_text_cfg}

[button](Apply){on_apply_yaml}
