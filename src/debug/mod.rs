use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::{app::Plugin, reflect::Uuid};
use bevy::{diagnostic::*, window::WindowId};
use bevy_egui::{
    egui::{self, Id, Ui},
    EguiContext, EguiPlugin,
};

struct DebugInfo {
    num_entities: usize,
    num_components: HashMap<Uuid, usize>,
}

pub struct DebugPlugin;

fn debug_info(egui_context: Res<EguiContext>, diagnostics: Res<Diagnostics>) {
    puffin::profile_function!();
    puffin::profile_scope!("draw_debug");
    egui::SidePanel::left("debug_panel").show(egui_context.ctx(), |ui: &mut Ui| {
        for diagnostic in diagnostics.iter() {
            if let Some(value) = diagnostic.value() {
                ui.heading(&*diagnostic.name);

                ui.label(format!("{}", (value * 100.).round() / 100.));
            }
        }
    });
}

fn profiler_window(egui_context: Res<EguiContext>) {
    puffin_egui::profiler_window(egui_context.ctx());
    puffin::GlobalProfiler::lock().new_frame();
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_plugin(EntityCountDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(EguiPlugin)
            .add_system(debug_info.system())
            .add_system_to_stage(CoreStage::Last, profiler_window.system());
    }
}
