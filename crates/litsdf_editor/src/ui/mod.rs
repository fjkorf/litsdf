mod handlers;
pub mod helpers;
mod modifier_stack;
pub mod populate;
pub mod shortcuts;
mod sync;
mod tree;

use std::collections::HashMap;

use bevy::prelude::*;
use egui_snarl::Snarl;
use egui_snarl::ui::SnarlStyle;

use litsdf_core::models::{BoneId, ShapeId, SdfPrimitive};
use litsdf_render::camera::OrbitCamera;
use litsdf_render::picking;
use litsdf_render::scene_sync::SdfSceneState;

use crate::nodes::{SdfNode, SdfNodeViewer};

pub mod app {
    use litui::*;

    define_litui_app! {
        parent: "content/_app.md",
        "content/properties.md",
        "content/add_shape.md",
        "content/file_browser.md",
        "content/yaml_editor.md",
    }
}

#[derive(Resource)]
pub struct EditorUi {
    pub md: app::LituiApp,
    pub(crate) prev_on_delete_shape: u32,
    pub(crate) prev_on_edit_yaml: u32,
    pub(crate) prev_on_apply_yaml: u32,
    pub(crate) prev_on_confirm_add: u32,
    pub(crate) prev_on_reset_transform: u32,

    pub(crate) prev_on_confirm_save: u32,
    pub(crate) prev_pick_file_counts: Vec<u32>,
    pub(crate) file_browser_save_mode: bool,
    pub(crate) prev_selected_shape: Option<ShapeId>,
    pub(crate) prev_selected_bone: Option<BoneId>,
    pub(crate) prev_shape_clicks: HashMap<ShapeId, u32>,
    pub(crate) shape_order: Vec<ShapeId>,
    // Node editor state
    pub(crate) show_node_editor: bool,
    pub(crate) node_graphs: HashMap<ShapeId, Snarl<SdfNode>>,
    pub(crate) bone_graphs: HashMap<BoneId, Snarl<SdfNode>>,
    pub(crate) node_style: SnarlStyle,
    // Graph undo (separate from scene undo)
    pub(crate) graph_undo_stack: Vec<(ShapeId, Snarl<SdfNode>)>,
    pub(crate) rename_state: Option<(tree::RenameTarget, String)>,
    pub(crate) clipboard: Option<litsdf_core::models::SdfShape>,
    // Animation / physics playback
    pub(crate) animation_time: f32,
    pub(crate) animation_playing: bool,
    pub(crate) animation_enabled: bool,
    pub(crate) physics_enabled: bool,
    pub(crate) rest_pose: Option<litsdf_core::models::SdfBone>,
    pub(crate) physics_states: HashMap<BoneId, litsdf_core::physics::BonePhysicsState>,
    pub(crate) show_help: bool,
    pub(crate) show_description: bool,
}

impl Default for EditorUi {
    fn default() -> Self {
        Self {
            md: app::LituiApp::default(),
            prev_on_delete_shape: 0,
            prev_on_edit_yaml: 0,
            prev_on_apply_yaml: 0,
            prev_on_confirm_add: 0,
            prev_on_reset_transform: 0,

            prev_on_confirm_save: 0,
            prev_pick_file_counts: Vec::new(),
            file_browser_save_mode: false,
            prev_selected_shape: None,
            prev_selected_bone: None,
            prev_shape_clicks: HashMap::new(),
            shape_order: Vec::new(),
            show_node_editor: false,
            node_graphs: HashMap::new(),
            bone_graphs: HashMap::new(),
            node_style: SnarlStyle::new(),
            graph_undo_stack: Vec::new(),
            rename_state: None,
            clipboard: None,
            animation_time: 0.0,
            animation_playing: false,
            animation_enabled: true,
            physics_enabled: true,
            rest_pose: None,
            physics_states: HashMap::new(),
            show_help: false,
            show_description: false,
        }
    }
}

/// Actions collected from the left panel to apply after rendering.
#[derive(Default)]
struct TreePanelActions {
    select_bone: Option<BoneId>,
    select_shape: Option<ShapeId>,
    add_bone: bool,
    add_shape: bool,
    delete_selected: bool,
    show_gizmos: Option<bool>,
    context_action: tree::ContextAction,
}

impl Default for tree::ContextAction {
    fn default() -> Self { Self::None }
}

pub fn editor_ui(
    mut contexts: bevy_egui::EguiContexts,
    mut ui: ResMut<EditorUi>,
    mut scene: ResMut<SdfSceneState>,
    mut undo_history: ResMut<crate::undo::UndoHistory>,
    drag_state: Res<litsdf_render::picking::DragState>,
    mut gizmo_mode: ResMut<litsdf_render::picking::GizmoMode>,
    mut camera_query: Query<&mut OrbitCamera>,
    time: Res<Time>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(c) => c.clone(),
        Err(_) => return,
    };

    // ── On first frame, sync animation_playing from scene state ──
    // (handles LITSDF_DEMO=chain setting physics_paused=false before editor starts)
    if ui.animation_time == 0.0 && !scene.physics_paused {
        ui.animation_playing = true;
        if ui.rest_pose.is_none() {
            ui.rest_pose = Some(scene.scene.root_bone.clone());
        }
    }

    // ── Keyboard shortcuts (must be checked every frame, outside menus) ──
    let mut shortcut_action = ShortcutAction::None;
    if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::REDO)) {
        shortcut_action = ShortcutAction::Redo;
    } else if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::UNDO)) {
        shortcut_action = ShortcutAction::Undo;
    }
    if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::SAVE)) {
        shortcut_action = ShortcutAction::Save;
    }
    if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::OPEN)) {
        shortcut_action = ShortcutAction::Open;
    }
    if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::NEW)) {
        shortcut_action = ShortcutAction::NewScene;
    }
    if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::DUPLICATE)) {
        shortcut_action = ShortcutAction::Duplicate;
    }
    if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::COPY)) {
        shortcut_action = ShortcutAction::Copy;
    }
    if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::PASTE)) {
        shortcut_action = ShortcutAction::Paste;
    }
    // Single-key shortcuts only fire when no text widget has focus
    if !ctx.wants_keyboard_input() {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete))
            || ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace))
        {
            shortcut_action = ShortcutAction::Delete;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            shortcut_action = ShortcutAction::Deselect;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F)) {
            shortcut_action = ShortcutAction::FrameSelection;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::H)) {
            shortcut_action = ShortcutAction::HideSelected;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::H)) {
            shortcut_action = ShortcutAction::ShowAll;
        }
        // Camera views (numpad and regular number keys both map to Num*)
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Num1)) {
            shortcut_action = ShortcutAction::CameraFront;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Num3)) {
            shortcut_action = ShortcutAction::CameraRight;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Num7)) {
            shortcut_action = ShortcutAction::CameraTop;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Num5)) {
            shortcut_action = ShortcutAction::ToggleOrtho;
        }

        // Gizmo mode switching
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::G)) {
            *gizmo_mode = litsdf_render::picking::GizmoMode::Translate;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::R)) {
            *gizmo_mode = litsdf_render::picking::GizmoMode::Rotate;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::S)) {
            *gizmo_mode = litsdf_render::picking::GizmoMode::Scale;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::E)) {
            *gizmo_mode = litsdf_render::picking::GizmoMode::Elongation;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::P)) {
            *gizmo_mode = litsdf_render::picking::GizmoMode::Repetition;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Space)) {
            shortcut_action = ShortcutAction::TogglePlayback;
        }
        // ? key — check for the character since it's Shift+/ on most layouts
        let question_pressed = ctx.input(|i| {
            i.events.iter().any(|e| matches!(e, egui::Event::Text(t) if t == "?"))
        });
        if question_pressed {
            ui.show_help = !ui.show_help;
        }
    }

    // ── Menu bar ──
    let mut ui_show_node_editor = ui.show_node_editor;
    let mut ui_animation_enabled = ui.animation_enabled;
    let mut ui_physics_enabled = ui.physics_enabled;
    let mut selected_demo: Option<crate::demos::DemoScene> = None;
    egui::TopBottomPanel::top("menu_bar").show(&ctx, |bar_ui| {
        egui::MenuBar::new().ui(bar_ui, |bar_ui| {
            bar_ui.menu_button("File", |ui| {
                if ui.add(egui::Button::new("New Scene").shortcut_text(ctx.format_shortcut(&shortcuts::NEW))).clicked() {
                    shortcut_action = ShortcutAction::NewScene;
                    ui.close();
                }
                ui.menu_button("Demo Scenes", |ui| {
                    for demo in crate::demos::DemoScene::all() {
                        if ui.button(demo.label()).clicked() {
                            selected_demo = Some(*demo);
                            ui.close();
                        }
                    }
                });
                ui.separator();
                if ui.add(egui::Button::new("Open...").shortcut_text(ctx.format_shortcut(&shortcuts::OPEN))).clicked() {
                    shortcut_action = ShortcutAction::Open;
                    ui.close();
                }
                if ui.add(egui::Button::new("Save").shortcut_text(ctx.format_shortcut(&shortcuts::SAVE))).clicked() {
                    shortcut_action = ShortcutAction::Save;
                    ui.close();
                }
            });
            bar_ui.menu_button("Edit", |ui| {
                if ui.add(egui::Button::new("Undo").shortcut_text(ctx.format_shortcut(&shortcuts::UNDO))).clicked() {
                    shortcut_action = ShortcutAction::Undo;
                    ui.close();
                }
                if ui.add(egui::Button::new("Redo").shortcut_text(ctx.format_shortcut(&shortcuts::REDO))).clicked() {
                    shortcut_action = ShortcutAction::Redo;
                    ui.close();
                }
                ui.separator();
                if ui.add(egui::Button::new("Duplicate").shortcut_text(ctx.format_shortcut(&shortcuts::DUPLICATE))).clicked() {
                    shortcut_action = ShortcutAction::Duplicate;
                    ui.close();
                }
                if ui.add(egui::Button::new("Delete").shortcut_text("Del")).clicked() {
                    shortcut_action = ShortcutAction::Delete;
                    ui.close();
                }
                ui.separator();
                if ui.add(egui::Button::new("Deselect").shortcut_text("Esc")).clicked() {
                    shortcut_action = ShortcutAction::Deselect;
                    ui.close();
                }
            });
            bar_ui.menu_button("Add", |ui| {
                if ui.button("Bone").clicked() {
                    shortcut_action = ShortcutAction::AddBone;
                    ui.close();
                }
                ui.separator();
                for name in helpers::PRIM_NAMES {
                    if ui.button(*name).clicked() {
                        shortcut_action = ShortcutAction::AddShape(name.to_string());
                        ui.close();
                    }
                }
            });
            bar_ui.menu_button("View", |ui| {
                let mut gizmos = scene.show_bone_gizmos;
                if ui.checkbox(&mut gizmos, "Bone Gizmos").changed() {
                    scene.show_bone_gizmos = gizmos;
                }
                ui.checkbox(&mut ui_show_node_editor, "Node Editor");
                ui.separator();
                ui.checkbox(&mut ui_animation_enabled, "Animation");
                ui.checkbox(&mut ui_physics_enabled, "Physics");
                ui.separator();
                if ui.add(egui::Button::new("Frame Selection").shortcut_text("F")).clicked() {
                    shortcut_action = ShortcutAction::FrameSelection;
                    ui.close();
                }
                if ui.button("Reset Camera").clicked() {
                    shortcut_action = ShortcutAction::ResetCamera;
                    ui.close();
                }
            });
        });
    });

    ui.show_node_editor = ui_show_node_editor;
    ui.animation_enabled = ui_animation_enabled;
    ui.physics_enabled = ui_physics_enabled;

    // ── Status bar ──
    egui::TopBottomPanel::bottom("status_bar").show(&ctx, |bar_ui| {
        bar_ui.horizontal(|bar_ui| {
            if let Some(shape_id) = scene.selected_shape {
                if let Some((shape, bone_id)) = scene.scene.root_bone.find_shape(shape_id) {
                    if let Some(bone) = scene.scene.root_bone.find_bone(bone_id) {
                        bar_ui.label(format!("{} ({}) on {}", shape.name, shape.primitive.label(), bone.name));
                    }
                }
            } else if let Some(bone_id) = scene.selected_bone {
                if let Some(bone) = scene.scene.root_bone.find_bone(bone_id) {
                    bar_ui.label(format!("Bone: {}", bone.name));
                }
            } else {
                bar_ui.label("No selection");
            }
            bar_ui.separator();
            bar_ui.label(gizmo_mode.label());
            bar_ui.separator();
            // Playback controls
            let play_label = if ui.animation_playing { "||" } else { ">" };
            if bar_ui.small_button(play_label).clicked() {
                shortcut_action = ShortcutAction::TogglePlayback;
            }
            if bar_ui.small_button("Reset").clicked() {
                shortcut_action = ShortcutAction::ResetPlayback;
            }
            bar_ui.label(format!("{:.1}s", ui.animation_time));
            bar_ui.separator();
            let info = scene.scene.info();
            bar_ui.label(format!("{} bones, {} shapes", info.bone_count, info.shape_count));
        });
    });

    // ── Node editor panel (bottom, above status bar) ──
    if ui.show_node_editor {
        egui::TopBottomPanel::bottom("node_editor")
            .resizable(true)
            .default_height(250.0)
            .min_height(100.0)
            .show(&ctx, |panel_ui| {
                let style = ui.node_style.clone();
                if let Some(shape_id) = scene.selected_shape {
                    // Shape node graph
                    let has_graph = ui.node_graphs.contains_key(&shape_id);
                    panel_ui.horizontal(|hui| {
                        hui.label("Shape Graph");
                        if let Some((shape, _)) = scene.scene.root_bone.find_shape(shape_id) {
                            hui.label(format!("— {}", shape.name));
                        }
                        let mut shape_preset: Option<Snarl<SdfNode>> = None;
                        hui.menu_button("Presets", |menu_ui| {
                            if menu_ui.button("Bob (Y bounce)").clicked() {
                                shape_preset = Some(crate::nodes::bob_preset(0.3, 0.5));
                                menu_ui.close();
                            }
                            if menu_ui.button("Spin (Y rotation)").clicked() {
                                shape_preset = Some(crate::nodes::spin_preset(45.0));
                                menu_ui.close();
                            }
                            if menu_ui.button("Pulse (breathe)").clicked() {
                                shape_preset = Some(crate::nodes::pulse_preset(0.1, 0.5));
                                menu_ui.close();
                            }
                            if menu_ui.button("Orbit (XZ circle)").clicked() {
                                shape_preset = Some(crate::nodes::orbit_preset(1.0, 0.3));
                                menu_ui.close();
                            }
                            if menu_ui.button("Color Cycle").clicked() {
                                shape_preset = Some(crate::nodes::color_cycle_preset(0.3));
                                menu_ui.close();
                            }
                        });
                        if let Some(preset) = shape_preset {
                            let old = ui.node_graphs.get(&shape_id).cloned();
                            if let Some(old) = old {
                                ui.graph_undo_stack.push((shape_id, old));
                            }
                            ui.node_graphs.insert(shape_id, preset);
                        }
                        if has_graph {
                            if hui.button("Clear").clicked() {
                                let old = ui.node_graphs.get(&shape_id).cloned();
                                if let Some(old) = old {
                                    ui.graph_undo_stack.push((shape_id, old));
                                }
                                ui.node_graphs.remove(&shape_id);
                            }
                        }
                    });
                    panel_ui.separator();
                    let snarl = ui.node_graphs.entry(shape_id).or_insert_with(Snarl::new);
                    snarl.show(&mut SdfNodeViewer, &style, egui::Id::new("sdf_node_editor"), panel_ui);
                } else if let Some(bone_id) = scene.selected_bone {
                    // Bone node graph
                    if !bone_id.is_root() {
                        let has_graph = ui.bone_graphs.contains_key(&bone_id);
                        panel_ui.horizontal(|hui| {
                            hui.label("Bone Graph");
                            if let Some(bone) = scene.scene.root_bone.find_bone(bone_id) {
                                hui.label(format!("— {}", bone.name));
                            }
                            let mut bone_preset: Option<Snarl<SdfNode>> = None;
                            hui.menu_button("Presets", |menu_ui| {
                                if menu_ui.button("Bob (Y bounce)").clicked() {
                                    bone_preset = Some(crate::nodes::bone_bob_preset(0.3, 0.5));
                                    menu_ui.close();
                                }
                                if menu_ui.button("Spin (Y rotation)").clicked() {
                                    bone_preset = Some(crate::nodes::bone_spin_preset(30.0));
                                    menu_ui.close();
                                }
                            });
                            if let Some(preset) = bone_preset {
                                ui.bone_graphs.insert(bone_id, preset);
                            }
                            if has_graph {
                                if hui.button("Clear").clicked() {
                                    ui.bone_graphs.remove(&bone_id);
                                }
                            }
                        });
                        panel_ui.separator();
                        let snarl = ui.bone_graphs.entry(bone_id).or_insert_with(Snarl::new);
                        snarl.show(&mut SdfNodeViewer, &style, egui::Id::new("bone_node_editor"), panel_ui);
                    } else {
                        panel_ui.label("Root bone has no node graph");
                    }
                } else {
                    panel_ui.label("Select a shape or bone to edit its node graph");
                }
            });
    }

    // Detect scene change before name sync overwrites the comparison
    let settings_need_init = ui.md.state.scene_name != scene.scene.name || ui.md.state.scene_name.is_empty();

    // Scene name (bidirectional — always sync)
    if ui.md.state.scene_name != scene.scene.name && !ui.md.state.scene_name.is_empty() {
        scene.scene.name = ui.md.state.scene_name.clone();
    } else {
        ui.md.state.scene_name = scene.scene.name.clone();
    }

    // Light direction (bidirectional)
    let ui_light = [ui.md.state.light_x as f32, ui.md.state.light_y as f32, ui.md.state.light_z as f32];
    if ui_light != scene.scene.light_dir && ui_light != [0.0, 0.0, 0.0] {
        scene.scene.light_dir = ui_light;
        scene.dirty = true;
    } else {
        ui.md.state.light_x = scene.scene.light_dir[0] as f64;
        ui.md.state.light_y = scene.scene.light_dir[1] as f64;
        ui.md.state.light_z = scene.scene.light_dir[2] as f64;
    }

    // Scene settings (bidirectional)
    // On scene change (new/load/demo), force-populate UI from model to avoid
    // the first-frame bug where UI defaults (0.0) overwrite non-zero model values.
    {
        let s = &mut scene.scene.settings;
        if settings_need_init {
            ui.md.state.fill_intensity = s.fill_intensity as f64;
            ui.md.state.back_intensity = s.back_intensity as f64;
            ui.md.state.sss_intensity = s.sss_intensity as f64;
            ui.md.state.ao_intensity = s.ao_intensity as f64;
            ui.md.state.shadow_softness = s.shadow_softness as f64;
            ui.md.state.vignette_intensity = s.vignette_intensity as f64;
            ui.md.state.gravity = s.gravity as f64;
        } else {
            let mut changed = false;
            macro_rules! sync_setting {
                ($ui_field:ident, $model_field:expr) => {
                    let ui_val = ui.md.state.$ui_field as f32;
                    if (ui_val - $model_field).abs() > 1e-6 {
                        $model_field = ui_val;
                        changed = true;
                    } else {
                        ui.md.state.$ui_field = $model_field as f64;
                    }
                };
            }
            sync_setting!(fill_intensity, s.fill_intensity);
            sync_setting!(back_intensity, s.back_intensity);
            sync_setting!(sss_intensity, s.sss_intensity);
            sync_setting!(ao_intensity, s.ao_intensity);
            sync_setting!(shadow_softness, s.shadow_softness);
            sync_setting!(vignette_intensity, s.vignette_intensity);
            sync_setting!(gravity, s.gravity);
            if changed { scene.dirty = true; }
        }
    }

    // Populate litui state for properties + file browser
    populate::populate_bone_shapes(&mut ui, &scene);
    populate::populate_shape_properties(&mut ui, &scene);
    populate::populate_bone_properties(&mut ui, &scene);
    populate::populate_file_browser(&mut ui);

    // ── Left panel: pure egui bone tree ──
    let tree_actions = render_tree_panel(&ctx, &scene, &mut ui.rename_state);

    // ── Right panel: litui properties + egui modifier stack ──
    egui::SidePanel::right("panel_properties")
        .default_width(260.0)
        .show(&ctx, |panel_ui| {
            egui::ScrollArea::vertical().show(panel_ui, |panel_ui| {
                app::render_properties(panel_ui, &mut ui.md.state);
                // Modifier stack (pure egui, below litui properties)
                if scene.selected_shape.is_some() {
                    panel_ui.separator();
                    if modifier_stack::render_modifier_stack(panel_ui, &mut scene) {
                        scene.dirty = true;
                    }
                }
            });
        });

    // ── Windows: litui dialogs ──
    {
        let mut open = ui.md.state.show_add_shape;
        egui::Window::new("Add Shape")
            .default_width(350.0)
            .open(&mut open)
            .show(&ctx, |wui| {
                egui::ScrollArea::vertical().show(wui, |wui| {
                    app::render_add_shape(wui, &mut ui.md.state);
                });
            });
        ui.md.state.show_add_shape = open;
    }
    {
        let mut open = ui.md.state.show_file_browser;
        egui::Window::new("File Browser")
            .default_width(350.0)
            .open(&mut open)
            .show(&ctx, |wui| {
                egui::ScrollArea::vertical().show(wui, |wui| {
                    app::render_file_browser(wui, &mut ui.md.state);
                });
            });
        ui.md.state.show_file_browser = open;
    }
    {
        let mut open = ui.md.state.show_yaml_editor;
        egui::Window::new("YAML Editor")
            .default_width(450.0)
            .open(&mut open)
            .show(&ctx, |wui| {
                egui::ScrollArea::vertical().show(wui, |wui| {
                    app::render_yaml_editor(wui, &mut ui.md.state);
                });
            });
        ui.md.state.show_yaml_editor = open;
    }

    // ── Help overlay ──
    if ui.show_help {
        let mut open = true;
        egui::Window::new("Keyboard Shortcuts")
            .collapsible(false)
            .default_width(340.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .show(&ctx, |wui| {
                egui::Grid::new("help_grid").num_columns(2).spacing([20.0, 4.0]).show(wui, |gui| {
                    let shortcuts: &[(&str, &str)] = &[
                        ("General", ""),
                        ("Cmd+S", "Save"),
                        ("Cmd+O", "Open"),
                        ("Cmd+N", "New Scene"),
                        ("Cmd+Z", "Undo"),
                        ("Cmd+Shift+Z", "Redo"),
                        ("Cmd+D", "Duplicate"),
                        ("Cmd+C / Cmd+V", "Copy / Paste"),
                        ("Delete", "Delete selected"),
                        ("Escape", "Deselect"),
                        ("?", "Toggle this help"),
                        ("", ""),
                        ("Viewport", ""),
                        ("F", "Frame selection"),
                        ("H", "Hide/show selected"),
                        ("Alt+H", "Show all"),
                        ("1", "Front view"),
                        ("3", "Right view"),
                        ("7", "Top view"),
                        ("5", "Toggle ortho/perspective"),
                        ("", ""),
                        ("Gizmo Modes", ""),
                        ("G", "Translate"),
                        ("R", "Rotate"),
                        ("S", "Scale"),
                        ("E", "Elongation"),
                        ("P", "Repetition"),
                        ("", ""),
                        ("Playback", ""),
                        ("Space", "Play / Pause"),
                    ];
                    for (key, desc) in shortcuts {
                        if desc.is_empty() && !key.is_empty() {
                            gui.label(egui::RichText::new(*key).strong());
                            gui.end_row();
                        } else if key.is_empty() {
                            gui.end_row();
                        } else {
                            gui.label(egui::RichText::new(*key).monospace());
                            gui.label(*desc);
                            gui.end_row();
                        }
                    }
                });
            });
        if !open { ui.show_help = false; }
    }

    // ── Scene description popup ──
    if ui.show_description && !scene.scene.description.is_empty() {
        let mut open = true;
        egui::Window::new(&scene.scene.name)
            .collapsible(false)
            .default_width(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .show(&ctx, |wui| {
                wui.label(&scene.scene.description);
            });
        if !open { ui.show_description = false; }
    }

    // ── Snapshot for undo before mutations ──
    let scene_before = scene.scene.clone();

    // ── Apply shortcut actions ──
    let mut bone_changed = false;
    match shortcut_action {
        ShortcutAction::Undo => {
            if let Some(prev) = undo_history.undo(&scene.scene) {
                scene.scene = prev;
                scene.selected_shape = None;
                scene.selected_bone = None;
                scene.dirty = true;
                bone_changed = true;
            }
        }
        ShortcutAction::Redo => {
            if let Some(next) = undo_history.redo(&scene.scene) {
                scene.scene = next;
                scene.selected_shape = None;
                scene.selected_bone = None;
                scene.dirty = true;
                bone_changed = true;
            }
        }
        ShortcutAction::Save => {
            ui.file_browser_save_mode = true;
            ui.md.state.show_file_browser = true;
        }
        ShortcutAction::Open => {
            ui.file_browser_save_mode = false;
            ui.md.state.show_file_browser = true;
        }
        ShortcutAction::NewScene => {
            scene.scene = litsdf_core::models::SdfScene::new("Untitled");
            ui.node_graphs.clear();
            ui.bone_graphs.clear();
            scene.selected_shape = None;
            scene.selected_bone = None;
            scene.dirty = true;
            bone_changed = true;
        }
        ShortcutAction::Duplicate => {
            if let Some(shape_id) = scene.selected_shape {
                if let Some((shape, bone_id)) = scene.scene.root_bone.find_shape(shape_id) {
                    let dup = shape.duplicate();
                    let dup_id = dup.id;
                    if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
                        bone.shapes.push(dup);
                    }
                    scene.selected_shape = Some(dup_id);
                    scene.dirty = true;
                    bone_changed = true;
                }
            } else if let Some(bone_id) = scene.selected_bone {
                if !bone_id.is_root() {
                    if let Some(bone) = scene.scene.root_bone.find_bone(bone_id) {
                        let dup = bone.duplicate_deep();
                        let dup_id = dup.id;
                        // Find parent and add duplicate as sibling
                        if let Some(parent_id) = find_parent_of_bone(&scene.scene.root_bone, bone_id) {
                            if let Some(parent) = scene.scene.root_bone.find_bone_mut(parent_id) {
                                parent.children.push(dup);
                            }
                        }
                        scene.selected_bone = Some(dup_id);
                        scene.selected_shape = None;
                        scene.dirty = true;
                        bone_changed = true;
                    }
                }
            }
        }
        ShortcutAction::Delete => {
            if let Some(shape_id) = scene.selected_shape {
                scene.scene.root_bone.remove_shape(shape_id);
                scene.selected_shape = None;
                scene.dirty = true;
                bone_changed = true;
            } else if let Some(bone_id) = scene.selected_bone {
                if !bone_id.is_root() {
                    scene.scene.root_bone.remove_bone(bone_id);
                    scene.selected_bone = None;
                    scene.selected_shape = None;
                    scene.dirty = true;
                    bone_changed = true;
                }
            }
        }
        ShortcutAction::Deselect => {
            scene.selected_shape = None;
            scene.selected_bone = None;
            bone_changed = true;
        }
        ShortcutAction::FrameSelection => {
            if let Some(pos) = picking::get_selected_world_pos(&scene) {
                if let Ok(mut cam) = camera_query.single_mut() {
                    cam.frame_target = Some(pos);
                }
            }
        }
        ShortcutAction::ResetCamera => {
            if let Ok(mut cam) = camera_query.single_mut() {
                cam.target = Vec3::new(0.0, 0.8, 0.0);
                cam.distance = 5.0;
                cam.yaw = 0.0;
                cam.pitch = 0.15;
            }
        }
        ShortcutAction::HideSelected => {
            if let Some(shape_id) = scene.selected_shape {
                if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(shape_id) {
                    shape.visible = !shape.visible;
                    scene.dirty = true;
                }
            } else if let Some(bone_id) = scene.selected_bone {
                if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
                    bone.visible = !bone.visible;
                    scene.dirty = true;
                }
            }
        }
        ShortcutAction::ShowAll => {
            fn show_all(bone: &mut litsdf_core::models::SdfBone) {
                bone.visible = true;
                for shape in &mut bone.shapes { shape.visible = true; }
                for child in &mut bone.children { show_all(child); }
            }
            show_all(&mut scene.scene.root_bone);
            scene.dirty = true;
        }
        ShortcutAction::AddBone => {
            let parent_id = scene.selected_bone.unwrap_or(BoneId::root());
            let new_bone = litsdf_core::models::SdfBone::new("Bone");
            let new_id = new_bone.id;
            if let Some(parent) = scene.scene.root_bone.find_bone_mut(parent_id) {
                parent.children.push(new_bone);
            }
            scene.selected_bone = Some(new_id);
            scene.selected_shape = None;
            scene.dirty = true;
            bone_changed = true;
        }
        ShortcutAction::AddShape(prim_name) => {
            let bone_id = scene.selected_bone.unwrap_or(BoneId::root());
            let prim = SdfPrimitive::default_for(&prim_name);
            let shape = litsdf_core::models::SdfShape::new(&prim_name, prim);
            let shape_id = shape.id;
            if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
                bone.shapes.push(shape);
            }
            scene.selected_shape = Some(shape_id);
            scene.dirty = true;
            bone_changed = true;
        }
        ShortcutAction::CameraFront => {
            if let Ok(mut cam) = camera_query.single_mut() {
                cam.yaw = 0.0;
                cam.pitch = 0.0;
            }
        }
        ShortcutAction::CameraRight => {
            if let Ok(mut cam) = camera_query.single_mut() {
                cam.yaw = -std::f32::consts::FRAC_PI_2;
                cam.pitch = 0.0;
            }
        }
        ShortcutAction::CameraTop => {
            if let Ok(mut cam) = camera_query.single_mut() {
                cam.yaw = 0.0;
                cam.pitch = -std::f32::consts::FRAC_PI_2 + 0.001; // slight offset to avoid gimbal lock
            }
        }
        ShortcutAction::ToggleOrtho => {
            if let Ok(mut cam) = camera_query.single_mut() {
                cam.toggle_ortho = true;
            }
        }
        ShortcutAction::Copy => {
            if let Some(shape_id) = scene.selected_shape {
                if let Some((shape, _)) = scene.scene.root_bone.find_shape(shape_id) {
                    ui.clipboard = Some(shape.clone());
                }
            }
        }
        ShortcutAction::Paste => {
            if let Some(ref clip) = ui.clipboard {
                let bone_id = scene.selected_bone.unwrap_or(BoneId::root());
                let mut pasted = clip.duplicate();
                let pasted_id = pasted.id;
                if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
                    bone.shapes.push(pasted);
                }
                scene.selected_shape = Some(pasted_id);
                scene.dirty = true;
                bone_changed = true;
            }
        }
        ShortcutAction::TogglePlayback => {
            if !ui.animation_playing {
                // Starting playback — snapshot rest pose if first time
                if ui.rest_pose.is_none() {
                    ui.rest_pose = Some(scene.scene.root_bone.clone());
                }
            }
            ui.animation_playing = !ui.animation_playing;
        }
        ShortcutAction::ResetPlayback => {
            ui.animation_time = 0.0;
            ui.animation_playing = false;
            litsdf_core::physics::reset_physics(&mut ui.physics_states);
            // Force Avian to respawn entities on next play
            scene.topology_hash = scene.topology_hash.wrapping_add(1);
            if let Some(rest) = ui.rest_pose.take() {
                scene.scene.root_bone = rest;
                scene.dirty = true;
            }
        }
        ShortcutAction::None => {}
    }

    // Load demo scene if selected from menu
    if let Some(demo) = selected_demo {
        let result = crate::demos::load_demo(demo);
        scene.scene = result.scene;
        ui.node_graphs = result.shape_graphs;
        ui.bone_graphs = result.bone_graphs;
        ui.show_description = !scene.scene.description.is_empty();
        // Auto-play if the scene has physics bones
        if litsdf_core::models::SdfBone::has_physics_bones(&scene.scene.root_bone) {
            ui.animation_playing = true;
            ui.rest_pose = Some(scene.scene.root_bone.clone());
        }
        scene.selected_shape = None;
        scene.selected_bone = None;
        scene.dirty = true;
        bone_changed = true;
    }

    // ── Apply tree panel actions ──
    if let Some(gizmos) = tree_actions.show_gizmos {
        scene.show_bone_gizmos = gizmos;
    }
    if tree_actions.add_bone {
        let parent_id = scene.selected_bone.unwrap_or(BoneId::root());
        let new_bone = litsdf_core::models::SdfBone::new("Bone");
        let new_id = new_bone.id;
        if let Some(parent) = scene.scene.root_bone.find_bone_mut(parent_id) {
            parent.children.push(new_bone);
        }
        scene.selected_bone = Some(new_id);
        scene.selected_shape = None;
        scene.dirty = true;
        bone_changed = true;
    }
    if tree_actions.add_shape {
        ui.md.state.show_add_shape = true;
    }
    if tree_actions.delete_selected {
        if let Some(shape_id) = scene.selected_shape {
            scene.scene.root_bone.remove_shape(shape_id);
            scene.selected_shape = None;
            scene.dirty = true;
            bone_changed = true;
        } else if let Some(bone_id) = scene.selected_bone {
            if !bone_id.is_root() {
                scene.scene.root_bone.remove_bone(bone_id);
                scene.selected_bone = None;
                scene.selected_shape = None;
                scene.dirty = true;
                bone_changed = true;
            }
        }
    }
    if let Some(id) = tree_actions.select_bone {
        if scene.selected_bone != Some(id) {
            scene.selected_bone = Some(id);
            if tree_actions.select_shape.is_none() {
                scene.selected_shape = None;
            }
            bone_changed = true;
        }
    }
    if let Some(shape_id) = tree_actions.select_shape {
        if scene.selected_shape != Some(shape_id) {
            scene.selected_shape = Some(shape_id);
            bone_changed = true;
        }
    }

    // ── Handle context menu actions from tree ──
    match tree_actions.context_action {
        tree::ContextAction::None => {}
        tree::ContextAction::AddChildBone(parent_id) => {
            let new_bone = litsdf_core::models::SdfBone::new("Bone");
            let new_id = new_bone.id;
            if let Some(parent) = scene.scene.root_bone.find_bone_mut(parent_id) {
                parent.children.push(new_bone);
            }
            scene.selected_bone = Some(new_id);
            scene.selected_shape = None;
            scene.dirty = true;
            bone_changed = true;
        }
        tree::ContextAction::AddShapeToBone(bone_id, prim_name) => {
            let prim = SdfPrimitive::default_for(&prim_name);
            let shape = litsdf_core::models::SdfShape::new(&prim_name, prim);
            let shape_id = shape.id;
            if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
                bone.shapes.push(shape);
            }
            scene.selected_shape = Some(shape_id);
            scene.dirty = true;
            bone_changed = true;
        }
        tree::ContextAction::DuplicateBone(bone_id) => {
            if let Some(bone) = scene.scene.root_bone.find_bone(bone_id) {
                let dup = bone.duplicate_deep();
                let dup_id = dup.id;
                if let Some(parent_id) = find_parent_of_bone(&scene.scene.root_bone, bone_id) {
                    if let Some(parent) = scene.scene.root_bone.find_bone_mut(parent_id) {
                        parent.children.push(dup);
                    }
                }
                scene.selected_bone = Some(dup_id);
                scene.selected_shape = None;
                scene.dirty = true;
                bone_changed = true;
            }
        }
        tree::ContextAction::DuplicateShape(shape_id) => {
            if let Some((shape, bone_id)) = scene.scene.root_bone.find_shape(shape_id) {
                let dup = shape.duplicate();
                let dup_id = dup.id;
                if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
                    bone.shapes.push(dup);
                }
                scene.selected_shape = Some(dup_id);
                scene.dirty = true;
                bone_changed = true;
            }
        }
        tree::ContextAction::DeleteBone(bone_id) => {
            if !bone_id.is_root() {
                scene.scene.root_bone.remove_bone(bone_id);
                if scene.selected_bone == Some(bone_id) {
                    scene.selected_bone = None;
                    scene.selected_shape = None;
                }
                scene.dirty = true;
                bone_changed = true;
            }
        }
        tree::ContextAction::DeleteShape(shape_id) => {
            scene.scene.root_bone.remove_shape(shape_id);
            if scene.selected_shape == Some(shape_id) {
                scene.selected_shape = None;
            }
            scene.dirty = true;
            bone_changed = true;
        }
        tree::ContextAction::DeleteBoneRecursive(bone_id) => {
            if !bone_id.is_root() {
                scene.scene.root_bone.extract_bone(bone_id);
                if scene.selected_bone == Some(bone_id) {
                    scene.selected_bone = None;
                    scene.selected_shape = None;
                }
                scene.dirty = true;
                bone_changed = true;
            }
        }
        tree::ContextAction::ReparentBone { bone, new_parent } => {
            if scene.scene.root_bone.reparent_bone(bone, new_parent) {
                scene.dirty = true;
                bone_changed = true;
            }
        }
        tree::ContextAction::ReparentShape { shape, new_bone } => {
            if scene.scene.root_bone.reparent_shape(shape, new_bone) {
                scene.dirty = true;
                bone_changed = true;
            }
        }
        tree::ContextAction::ToggleBoneVisibility(bone_id) => {
            if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
                bone.visible = !bone.visible;
                scene.dirty = true;
            }
        }
        tree::ContextAction::ToggleShapeVisibility(shape_id) => {
            if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(shape_id) {
                shape.visible = !shape.visible;
                scene.dirty = true;
            }
        }
        tree::ContextAction::RenameBone(bone_id, new_name) => {
            if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
                if !new_name.is_empty() {
                    bone.name = new_name;
                }
            }
        }
        tree::ContextAction::RenameShape(shape_id, new_name) => {
            if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(shape_id) {
                if !new_name.is_empty() {
                    shape.name = new_name;
                }
            }
        }
        tree::ContextAction::ToggleBonePhysics(bone_id) => {
            if let Some(bone) = scene.scene.root_bone.find_bone_mut(bone_id) {
                bone.physics.mass = if bone.physics.mass > 0.0 { 0.0 } else { 1.0 };
                // Re-populate bone properties since mass changed
                ui.prev_selected_bone = None;
            }
        }
    }

    // ── Handle litui button clicks ──
    handlers::handle_confirm_add(&mut ui, &mut scene);
    handlers::handle_delete_shape(&mut ui, &mut scene);
    handlers::handle_edit_yaml(&mut ui, &scene);
    handlers::handle_apply_yaml(&mut ui, &mut scene);
    let shape_changed = handlers::handle_shape_selection(&mut ui, &mut scene);
    handlers::handle_reset_transform(&mut ui, &mut scene);
    handlers::handle_save_load(&mut ui);
    handlers::handle_file_browser(&mut ui, &mut scene);

    if drag_state.active {
        if let Some(sel_id) = scene.selected_shape {
            if let Some((shape, _)) = scene.scene.root_bone.find_shape(sel_id) {
                match *gizmo_mode {
                    litsdf_render::picking::GizmoMode::Translate => {
                        ui.md.state.tx = shape.transform.translation[0] as f64;
                        ui.md.state.ty = shape.transform.translation[1] as f64;
                        ui.md.state.tz = shape.transform.translation[2] as f64;
                    }
                    litsdf_render::picking::GizmoMode::Rotate => {
                        ui.md.state.rx = shape.transform.rotation[0] as f64;
                        ui.md.state.ry = shape.transform.rotation[1] as f64;
                        ui.md.state.rz = shape.transform.rotation[2] as f64;
                    }
                    litsdf_render::picking::GizmoMode::Scale => {
                        ui.md.state.uniform_scale = shape.transform.scale as f64;
                    }
                    litsdf_render::picking::GizmoMode::Elongation => {
                        // Modifiers are edited directly via egui modifier stack;
                        // drag handles write to shape.modifiers, which egui reads next frame.
                    }
                    litsdf_render::picking::GizmoMode::Repetition => {
                        // Same — drag handles write directly to shape.modifiers.
                    }
                }
            }
        } else if let Some(bone_id) = scene.selected_bone {
            if let Some(bone) = scene.scene.root_bone.find_bone(bone_id) {
                match *gizmo_mode {
                    litsdf_render::picking::GizmoMode::Translate => {
                        ui.md.state.bone_tx = bone.transform.translation[0] as f64;
                        ui.md.state.bone_ty = bone.transform.translation[1] as f64;
                        ui.md.state.bone_tz = bone.transform.translation[2] as f64;
                    }
                    litsdf_render::picking::GizmoMode::Rotate => {
                        ui.md.state.bone_rx = bone.transform.rotation[0] as f64;
                        ui.md.state.bone_ry = bone.transform.rotation[1] as f64;
                        ui.md.state.bone_rz = bone.transform.rotation[2] as f64;
                    }
                    _ => {} // bones don't have modifiers
                }
            }
        }
    } else if !bone_changed && !shape_changed {
        sync::sync_shape_properties(&mut ui, &mut scene);
        sync::sync_bone_properties(&mut ui, &mut scene);
    }

    // ── Advance managed animation clock ──
    if ui.animation_playing {
        ui.animation_time += time.delta_secs();
    }
    scene.physics_paused = !ui.physics_enabled;

    // ── Evaluate node graphs ──
    // Node outputs override shape properties each frame (additive to base values).
    // This runs after sync so slider edits are captured, and before undo so
    // node-driven changes don't pollute the undo stack.
    let elapsed = ui.animation_time;
    let mut any_graph_active = false;
    if ui.animation_enabled {
    // Snapshot rest pose on first animation frame (for reset)
    let has_graphs = ui.node_graphs.values().any(|s| s.node_ids().next().is_some())
        || ui.bone_graphs.values().any(|s| s.node_ids().next().is_some());
    if has_graphs && ui.rest_pose.is_none() {
        ui.rest_pose = Some(scene.scene.root_bone.clone());
    }
    for (shape_id, snarl) in &ui.node_graphs {
        if snarl.node_ids().next().is_none() { continue; } // empty graph
        // Find the bone that owns this shape, to get its physics readings
        let bone_id = scene.scene.root_bone.find_shape(*shape_id).map(|(_, bid)| bid);
        let shape_physics = bone_id.and_then(|bid| scene.physics_readings.get(&bid));
        let outputs = crate::nodes::evaluate_graph(snarl, elapsed, shape_physics);
        if let Some((shape, _)) = scene.scene.root_bone.find_shape_mut(*shape_id) {
            let mut changed = false;
            if let Some(v) = outputs.tx { shape.transform.translation[0] = v; changed = true; }
            if let Some(v) = outputs.ty { shape.transform.translation[1] = v; changed = true; }
            if let Some(v) = outputs.tz { shape.transform.translation[2] = v; changed = true; }
            if let Some(v) = outputs.rx { shape.transform.rotation[0] = v; changed = true; }
            if let Some(v) = outputs.ry { shape.transform.rotation[1] = v; changed = true; }
            if let Some(v) = outputs.rz { shape.transform.rotation[2] = v; changed = true; }
            if let Some(v) = outputs.scale { shape.transform.scale = v; changed = true; }
            if let Some(v) = outputs.red { shape.material.color[0] = v; changed = true; }
            if let Some(v) = outputs.green { shape.material.color[1] = v; changed = true; }
            if let Some(v) = outputs.blue { shape.material.color[2] = v; changed = true; }
            if let Some(v) = outputs.roughness { shape.material.roughness = v; changed = true; }
            if let Some(v) = outputs.metallic { shape.material.metallic = v; changed = true; }
            if let Some(v) = outputs.fresnel { shape.material.fresnel_power = v; changed = true; }
            if let Some(v) = outputs.noise_amp { shape.material.noise_amplitude = v; changed = true; }
            if let Some(v) = outputs.noise_freq { shape.material.noise_frequency = v; changed = true; }
            if let Some(v) = outputs.noise_oct { shape.material.noise_octaves = v as u32; changed = true; }
            if let Some(v) = outputs.symmetry { shape.material.smooth_symmetry = v; changed = true; }
            // Modifier overrides from nodes — rebuild modifier list if any are connected
            if outputs.rounding.is_some() || outputs.onion.is_some() || outputs.twist.is_some()
                || outputs.bend.is_some() || outputs.elongate_x.is_some() || outputs.repeat_x.is_some()
            {
                use litsdf_core::models::ShapeModifier;
                let mut mods = Vec::new();
                if let Some(v) = outputs.rounding { if v > 0.0 { mods.push(ShapeModifier::Rounding(v)); } }
                if let Some(v) = outputs.onion { if v > 0.0 { mods.push(ShapeModifier::Onion(v)); } }
                if let Some(v) = outputs.twist { if v.abs() > 0.0 { mods.push(ShapeModifier::Twist(v)); } }
                if let Some(v) = outputs.bend { if v.abs() > 0.0 { mods.push(ShapeModifier::Bend(v)); } }
                let ex = outputs.elongate_x.unwrap_or(0.0);
                let ey = outputs.elongate_y.unwrap_or(0.0);
                let ez = outputs.elongate_z.unwrap_or(0.0);
                if ex > 0.0 || ey > 0.0 || ez > 0.0 { mods.push(ShapeModifier::Elongation([ex, ey, ez])); }
                let rx = outputs.repeat_x.unwrap_or(0.0);
                let ry = outputs.repeat_y.unwrap_or(0.0);
                let rz = outputs.repeat_z.unwrap_or(0.0);
                if rx > 0.0 || ry > 0.0 || rz > 0.0 {
                    mods.push(ShapeModifier::Repetition { period: [rx, ry, rz], count: [3, 3, 3] });
                }
                shape.modifiers = mods;
                changed = true;
            }
            if changed {
                scene.dirty = true;
                any_graph_active = true;
            }
        }
    }
    // Evaluate bone graphs
    scene.force_outputs.clear();
    for (bone_id, snarl) in &ui.bone_graphs {
        if snarl.node_ids().next().is_none() { continue; }
        let physics_reading = scene.physics_readings.get(bone_id);
        let outputs = crate::nodes::evaluate_bone_graph(snarl, elapsed, physics_reading);
        // Collect force outputs for Avian
        let has_forces = outputs.force_x.is_some() || outputs.force_y.is_some()
            || outputs.force_z.is_some() || outputs.torque_x.is_some()
            || outputs.torque_y.is_some() || outputs.torque_z.is_some();
        if has_forces {
            scene.force_outputs.insert(*bone_id, litsdf_render::scene_sync::BoneForceOutputs {
                force: [
                    outputs.force_x.unwrap_or(0.0),
                    outputs.force_y.unwrap_or(0.0),
                    outputs.force_z.unwrap_or(0.0),
                ],
                torque: [
                    outputs.torque_x.unwrap_or(0.0),
                    outputs.torque_y.unwrap_or(0.0),
                    outputs.torque_z.unwrap_or(0.0),
                ],
            });
        }
        if let Some(bone) = scene.scene.root_bone.find_bone_mut(*bone_id) {
            let mut changed = false;
            if let Some(v) = outputs.tx { bone.transform.translation[0] = v; changed = true; }
            if let Some(v) = outputs.ty { bone.transform.translation[1] = v; changed = true; }
            if let Some(v) = outputs.tz { bone.transform.translation[2] = v; changed = true; }
            if let Some(v) = outputs.rx { bone.transform.rotation[0] = v; changed = true; }
            if let Some(v) = outputs.ry { bone.transform.rotation[1] = v; changed = true; }
            if let Some(v) = outputs.rz { bone.transform.rotation[2] = v; changed = true; }
            if let Some(v) = outputs.scale { bone.transform.scale = v; changed = true; }
            if changed {
                scene.dirty = true;
                any_graph_active = true;
            }
        }
    }

    } // if ui.animation_enabled

    // ── Physics step (custom solver, only when Avian is disabled) ──
    if ui.physics_enabled && !scene.use_avian {
        let offsets = litsdf_core::physics::step_physics(
            &scene.scene.root_bone,
            &mut ui.physics_states,
            scene.scene.settings.gravity,
            time.delta_secs(),
        );
        for (bone_id, offset) in &offsets {
            if let Some(bone) = scene.scene.root_bone.find_bone_mut(*bone_id) {
                bone.transform.translation[0] += offset[0];
                bone.transform.translation[1] += offset[1];
                bone.transform.translation[2] += offset[2];
            }
        }
        if !offsets.is_empty() {
            scene.dirty = true;
            any_graph_active = true; // treat physics as animation for undo skip
        }
    }

    // Keep scene dirty while graphs are active (animation/physics)
    if any_graph_active {
        scene.dirty = true;
    }

    // Push undo snapshot if scene changed this frame (skip node/physics-driven changes)
    if !any_graph_active && scene.scene != scene_before {
        undo_history.push(scene_before);
    }
}

#[derive(Default)]
enum ShortcutAction {
    #[default]
    None,
    Undo, Redo,
    Save, Open, NewScene,
    Duplicate, Delete, Deselect,
    FrameSelection, ResetCamera,
    HideSelected, ShowAll,
    Copy, Paste,
    CameraFront, CameraRight, CameraTop, ToggleOrtho,
    AddBone, AddShape(String),
    TogglePlayback, ResetPlayback,
}

fn find_parent_of_bone(bone: &litsdf_core::models::SdfBone, target: BoneId) -> Option<BoneId> {
    for child in &bone.children {
        if child.id == target { return Some(bone.id); }
        if let Some(id) = find_parent_of_bone(child, target) { return Some(id); }
    }
    None
}

fn render_tree_panel(ctx: &egui::Context, scene: &SdfSceneState, rename_state: &mut Option<(tree::RenameTarget, String)>) -> TreePanelActions {
    let mut actions = TreePanelActions::default();

    egui::SidePanel::left("bone_tree")
        .default_width(220.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let result = tree::render_bone_tree(ui, &scene.scene.root_bone, scene.selected_bone, scene.selected_shape, rename_state);
                match result.action {
                    tree::TreeAction::SelectBone(id) => {
                        actions.select_bone = Some(id);
                    }
                    tree::TreeAction::SelectShape(shape_id, bone_id) => {
                        actions.select_bone = Some(bone_id);
                        actions.select_shape = Some(shape_id);
                    }
                    tree::TreeAction::None => {}
                }
                actions.context_action = result.context;
            });
        });

    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use litsdf_core::models::{BoneId, CombinationOp, SceneSettings, SdfBone, SdfPrimitive, SdfScene, SdfShape};
    use populate::*;
    use sync::*;

    fn make_test_scene() -> SdfScene {
        let mut root = SdfBone::root();
        root.shapes.push(SdfShape::default_sphere());
        let mut arm = SdfBone::new("Arm");
        arm.shapes.push(SdfShape::new("Box", SdfPrimitive::Box { half_extents: [0.5, 0.5, 0.5] }));
        let mut hand = SdfBone::new("Hand");
        hand.shapes.push(SdfShape::new("Capsule", SdfPrimitive::Capsule { radius: 0.2, half_height: 0.3 }));
        arm.children.push(hand);
        root.children.push(arm);
        SdfScene { name: "test".into(), description: String::new(), root_bone: root, combination: CombinationOp::Union, light_dir: [0.6, 0.8, 0.4], settings: SceneSettings::default() }
    }

    fn make_state(scene: SdfScene) -> (EditorUi, SdfSceneState) {
        (EditorUi::default(), SdfSceneState {
            scene, selected_shape: None, selected_bone: None, show_bone_gizmos: false, dirty: false, topology_hash: 0, use_avian: false, physics_readings: HashMap::new(), force_outputs: HashMap::new(), physics_paused: true,
        })
    }

    #[test]
    fn click_through_bone_to_shape_edit() {
        let scene = make_test_scene();
        let arm_id = scene.root_bone.children[0].id;
        let box_id = scene.root_bone.children[0].shapes[0].id;
        let (mut ui, mut scene) = make_state(scene);

        scene.selected_bone = Some(arm_id);
        populate_bone_shapes(&mut ui, &scene);
        populate_shape_properties(&mut ui, &scene);

        assert!(ui.md.state.has_bone_selection);
        assert_eq!(ui.md.state.bone_shapes.len(), 1);
        assert!(ui.md.state.bone_shapes[0].shape_name.contains("Box"));
        assert_eq!(ui.shape_order[0], box_id);

        scene.selected_shape = Some(box_id);
        populate_bone_shapes(&mut ui, &scene);
        populate_shape_properties(&mut ui, &scene);

        assert!(ui.md.state.has_selection);
        assert_eq!(ui.md.state.prim_type, 1);
        assert_eq!(ui.md.state.param_a, 0.5);

        populate_shape_properties(&mut ui, &scene);
        ui.md.state.param_a = 2.0;
        ui.md.state.param_c = 0.3;
        sync_shape_properties(&mut ui, &mut scene);

        assert!(scene.dirty);
        let (b, _) = scene.scene.root_bone.find_shape(box_id).unwrap();
        match &b.primitive {
            SdfPrimitive::Box { half_extents } => {
                assert_eq!(half_extents[0], 2.0);
                assert_eq!(half_extents[1], 0.5);
                assert_eq!(half_extents[2], 0.3);
            }
            other => panic!("Expected Box, got {:?}", other),
        }
    }

    #[test]
    fn no_dirty_when_no_change() {
        let scene = make_test_scene();
        let arm_id = scene.root_bone.children[0].id;
        let box_id = scene.root_bone.children[0].shapes[0].id;
        let (mut ui, mut scene) = make_state(scene);

        scene.selected_bone = Some(arm_id);
        scene.selected_shape = Some(box_id);
        populate_shape_properties(&mut ui, &scene);

        scene.dirty = false;
        populate_shape_properties(&mut ui, &scene);
        sync_shape_properties(&mut ui, &mut scene);
        assert!(!scene.dirty);
    }

    #[test]
    fn nested_bone_shape_selection() {
        let mut root = SdfBone::root();
        root.shapes.push(SdfShape::default_sphere());
        let mut arm = SdfBone::new("Arm");
        arm.transform.translation = [1.2, 0.0, 0.0];
        arm.shapes.push(SdfShape::new("Box", SdfPrimitive::Box { half_extents: [0.6, 0.6, 0.6] }));
        let mut arm1 = SdfBone::new("Arm 1");
        arm1.transform.translation = [1.2, 0.0, 0.0];
        let mut torus = SdfShape::new("Torus", SdfPrimitive::Torus { major_radius: 2.95, minor_radius: 1.15 });
        torus.transform.scale = 0.1;
        arm1.shapes.push(torus);
        arm.children.push(arm1);
        root.children.push(arm);
        let scene = SdfScene { name: "test".into(), description: String::new(), root_bone: root, combination: CombinationOp::Union, light_dir: [0.6, 0.8, 0.4], settings: SceneSettings::default() };

        let arm1_id = scene.root_bone.children[0].children[0].id;
        let (mut ui, mut scene_state) = make_state(scene);

        scene_state.selected_bone = Some(arm1_id);
        populate_bone_shapes(&mut ui, &scene_state);
        assert_eq!(ui.md.state.bone_shapes.len(), 1);
        assert!(ui.md.state.bone_shapes[0].shape_name.contains("Torus"));

        let torus_id = ui.shape_order[0];
        scene_state.selected_shape = Some(torus_id);
        populate_shape_properties(&mut ui, &scene_state);

        assert_eq!(ui.md.state.prim_type, 5);
        assert!((ui.md.state.param_a - 2.95).abs() < 0.01);

        populate_shape_properties(&mut ui, &scene_state);
        ui.md.state.param_a = 1.0;
        ui.md.state.param_b = 0.3;
        sync_shape_properties(&mut ui, &mut scene_state);

        assert!(scene_state.dirty);
        let (t, _) = scene_state.scene.root_bone.find_shape(torus_id).unwrap();
        match &t.primitive {
            SdfPrimitive::Torus { major_radius, minor_radius } => {
                assert!((major_radius - 1.0).abs() < 0.01);
                assert!((minor_radius - 0.3).abs() < 0.01);
            }
            other => panic!("Expected Torus, got {:?}", other),
        }
    }
}
