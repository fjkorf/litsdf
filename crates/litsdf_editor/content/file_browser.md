---
page:
  name: FileBrowser
  label: File Browser
  panel: window
  open: show_file_browser
  width: 350.0

widgets:
  filename_cfg:
    hint: scene_name.yaml
  on_confirm_save: {}
  on_pick_file: {}
---

## File Browser

[textedit](filename){filename_cfg}

::: foreach file_rows

| {name} | [button](Load){on_pick_file} |
|---|---|

:::

[button](Save){on_confirm_save}
