use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

fn update(mut ctx: EguiContexts) {
    egui::Window::new("hey!").show(ctx.ctx_mut(), |ui| {
        ui.label("hi");
    });
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin))
        .add_systems(Update, update)
        .run();
}
