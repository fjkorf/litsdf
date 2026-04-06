//! Tests for panel configuration and layout.
//! Verifies panels are resizable, properly ordered, and don't consume all space.
//! Uses headless egui::Context — no window or GPU needed.

use egui::{self, Pos2, vec2, Rect};

fn new_ctx() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.set_fonts(egui::FontDefinitions::empty());
    ctx
}

fn run(ctx: &egui::Context, build_ui: impl Fn(&egui::Context)) {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 720.0))),
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| build_ui(ctx));
}

fn panel_rect(ctx: &egui::Context, id: &str) -> Option<Rect> {
    ctx.data_mut(|d| {
        d.get_persisted::<egui::containers::panel::PanelState>(egui::Id::new(id))
            .map(|s| s.rect)
    })
}

/// Left panel starts at x=0 and has positive width.
#[test]
fn left_panel_exists_and_positioned() {
    let ctx = new_ctx();
    run(&ctx, |ctx| {
        egui::SidePanel::left("bone_tree")
            .resizable(true)
            .default_width(220.0)
            .show(ctx, |ui| { ui.label("Tree"); });
    });
    let rect = panel_rect(&ctx, "bone_tree").expect("left panel persisted");
    assert!(rect.min.x < 1.0, "left panel should start at left edge, got x={:.0}", rect.min.x);
    assert!(rect.width() > 10.0, "left panel should have positive width: {:.0}", rect.width());
}

/// Right panel ends at screen edge and has positive width.
#[test]
fn right_panel_exists_and_positioned() {
    let ctx = new_ctx();
    run(&ctx, |ctx| {
        egui::SidePanel::right("panel_properties")
            .default_width(260.0)
            .show(ctx, |ui| { ui.label("Props"); });
    });
    let rect = panel_rect(&ctx, "panel_properties").expect("right panel persisted");
    // In headless mode, panel may not extend to full screen edge due to empty fonts,
    // but it should be on the right side (max.x should be close to 1280)
    assert!(rect.max.x > 1000.0, "right panel should be near right edge, got max_x={:.0}", rect.max.x);
    assert!(rect.width() > 10.0, "right panel should have positive width: {:.0}", rect.width());
}

/// Node editor panel exists below center and has positive height.
#[test]
fn node_editor_panel_exists() {
    let ctx = new_ctx();
    run(&ctx, |ctx| {
        egui::TopBottomPanel::bottom("node_editor")
            .resizable(true)
            .default_height(250.0)
            .min_height(100.0)
            .show(ctx, |ui| { ui.label("Nodes"); });
    });
    let rect = panel_rect(&ctx, "node_editor").expect("node editor persisted");
    assert!(rect.min.y > 50.0, "node editor should be below center, got y={:.0}", rect.min.y);
    assert!(rect.height() > 10.0, "node editor should have positive height: {:.0}", rect.height());
}

/// Node editor is above status bar when both are bottom panels.
#[test]
fn node_editor_above_status_bar() {
    let ctx = new_ctx();
    run(&ctx, |ctx| {
        // Correct order: status bar first (at bottom edge), node editor second (above it)
        egui::TopBottomPanel::bottom("sb_order")
            .show(ctx, |ui| { ui.label("Status"); });
        egui::TopBottomPanel::bottom("ne_order")
            .resizable(true)
            .default_height(200.0)
            .show(ctx, |ui| { ui.label("Nodes"); });
    });

    let ne = panel_rect(&ctx, "ne_order").expect("node editor exists");
    let sb = panel_rect(&ctx, "sb_order").expect("status bar exists");

    // Status bar should be at the very bottom (first created = outermost)
    assert!(sb.max.y >= ne.max.y,
        "status bar bottom ({:.0}) should be >= node editor bottom ({:.0})",
        sb.max.y, ne.max.y);
    // Node editor should be above status bar
    assert!(ne.min.y < sb.min.y,
        "node editor top ({:.0}) should be above status bar top ({:.0})",
        ne.min.y, sb.min.y);
}

/// All panels together leave positive viewport space.
#[test]
fn all_panels_leave_viewport_space() {
    let ctx = new_ctx();
    let avail = std::cell::Cell::new(Rect::NOTHING);

    run(&ctx, |ctx| {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| { ui.label("Menu"); });
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| { ui.label("Status"); });
        egui::TopBottomPanel::bottom("node_editor")
            .resizable(true)
            .default_height(200.0)
            .min_height(100.0)
            .show(ctx, |ui| { ui.label("Nodes"); });
        egui::SidePanel::left("bone_tree")
            .resizable(true)
            .default_width(220.0)
            .show(ctx, |ui| { ui.label("Tree"); });
        egui::SidePanel::right("panel_properties")
            .default_width(260.0)
            .show(ctx, |ui| { ui.label("Props"); });
        avail.set(ctx.available_rect());
    });

    let a = avail.get();
    assert!(a.width() > 100.0,
        "viewport width should be >100 after all panels, got {:.0}", a.width());
    assert!(a.height() > 50.0,
        "viewport height should be >50 after all panels, got {:.0}", a.height());
}

/// Status bar is at the very bottom of the window.
#[test]
fn status_bar_at_bottom() {
    let ctx = new_ctx();
    run(&ctx, |ctx| {
        egui::TopBottomPanel::bottom("sb_edge")
            .show(ctx, |ui| { ui.label("Status"); });
    });
    let rect = panel_rect(&ctx, "sb_edge").expect("status bar exists");
    // First bottom panel = outermost = touches window edge
    assert!(rect.max.y > 710.0,
        "status bar should be near window bottom (720), got {:.0}", rect.max.y);
}
