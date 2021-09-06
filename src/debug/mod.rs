use bevy::diagnostic::*;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::{app::Plugin, reflect::Uuid};
use bevy_egui::{egui, EguiContext, EguiPlugin};

struct DebugInfo {
    num_entities: usize,
    num_components: HashMap<Uuid, usize>,
}

pub struct DebugPlugin;

fn debug_info(egui_context: Res<EguiContext>, diagnostics: Res<Diagnostics>) {
    egui::Window::new("Debug").show(egui_context.ctx(), |ui| {
        for diagnostic in diagnostics.iter() {
            if let Some(value) = diagnostic.value() {
                ui.heading(&*diagnostic.name);

                ui.label(format!("{}", value as usize));
            }
        }
    });
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_plugin(EntityCountDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(EguiPlugin)
            .add_system(debug_info.system());
    }
}
