use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui::{self, Pos2}};


use crate::{player::Player, resources::GameResources};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, setup_windows);
    }
}


fn setup_windows(
    mut context: EguiContexts,
    mut player_q: Query<&mut Player>,
    ) {
    let Ok(mut player) = player_q.single_mut() else {return;};
    egui::Window::new("Player Var").default_pos(Pos2::new(500.0, 500.0)).show(context.ctx_mut().expect("can't use egui context"), |ui| {
        ui.add(egui::Slider::new(&mut player.speed, 1.0..=20.0))
    });
}
