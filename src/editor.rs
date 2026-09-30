use bevy::{camera::visibility::RenderLayers, camera_controller::free_camera::FreeCamera, core_pipeline::tonemapping::Tonemapping, input::gamepad, math::VectorSpace, post_process::bloom::Bloom, prelude::*};
use bevy_egui::{EguiContext, EguiContexts, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui::{self, Pos2}};

use crate::{player::{CameraRotation, Player, PlayerCam}, resources::GameResources};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(EditorVar::default())
            .add_systems(EguiPrimaryContextPass, (setup_windows, inspector_window))
            .add_systems(Update, (handle_inputs, handle_editor_toggle, sync_gizmo_cam_projection))
            .add_message::<EditorToggled>();
    }
}

#[derive(Resource, Default)]
pub struct EditorVar {
}

#[derive(Message, Default)]
struct EditorToggled(bool);


fn sync_gizmo_cam_projection(
    main: Single<&Projection, With<PlayerCam>>,
    mut others: Query<(&Camera, &mut Projection), Without<PlayerCam>>,
) {
    for (cam, mut proj) in &mut others {
        if cam.order > 0 {
            *proj = (*main).clone();
        }
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

fn inspector_window(world: &mut World) {
    let Ok(mut egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single_mut(world)
        .map(|c| c.clone())
    else {
        return;
    };

    egui::Window::new("UI").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

            egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_assets::<StandardMaterial>(world, ui);
            });

            ui.heading("Entities");
            bevy_inspector_egui::bevy_inspector::ui_for_entities(world, ui);
        });
    });
}



fn handle_inputs(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_resource: ResMut<GameResources>,
    mut toggle_message_writer: MessageWriter<EditorToggled>
) {
    if keys.just_pressed(KeyCode::F12) {
        game_resource.in_editor = !game_resource.in_editor;
        toggle_message_writer.write(EditorToggled(game_resource.in_editor));
    }

}

fn handle_editor_toggle(
    mut commands: Commands,
    mut cam_q: Query<(Entity, &mut Transform, &GlobalTransform), With<PlayerCam>>,
    player_q: Query<(Entity, &CameraRotation), With<Player>>,
    mut toggled: MessageReader<EditorToggled>,
    mut game_resource: ResMut<GameResources>,
) {
    let Ok((cam_e, mut cam_t, cam_gt)) = cam_q.single_mut() else { return };
    let Ok((player_e, rot)) = player_q.single() else { return };

    for message in toggled.read() {
        if message.0 {
            game_resource.mouse_grabbed = false;
            *cam_t = cam_gt.compute_transform();
            commands.entity(player_e).detach_child(cam_e);
            commands
                .entity(cam_e)
                .insert((FreeCamera::default(), RenderLayers::from_layers(&[0, 1])));
        } else {
            commands
                .entity(cam_e)
                .remove::<FreeCamera>()
                .insert(RenderLayers::layer(0));
            commands.entity(player_e).add_child(cam_e);
            *cam_t = Transform::from_xyz(0.0, 1.5, 0.0)
                .with_rotation(Quat::from_rotation_x(rot.pitch));
        }
    }
}

fn toggle_cam(
    game_resource: Res<GameResources>,
    mut free_cam_q: Query<(&mut Camera, &mut Transform), (With<FreeCamera>, Without<PlayerCam>)>,
    mut player_cam_q: Query<(&mut Camera, &GlobalTransform), (With<PlayerCam>, Without<FreeCamera>)>,
    mut last_state: Local<bool>
) {
    let Ok((mut free_cam, mut free_cam_transform)) = free_cam_q.single_mut() else {return;};
    let Ok((mut player_cam, player_cam_transform)) = player_cam_q.single_mut() else {return;};
    if game_resource.in_editor != *last_state {
        if game_resource.in_editor {
            free_cam.is_active = true;
            player_cam.is_active = false;
            free_cam_transform.translation = player_cam_transform.translation();
            free_cam_transform.rotation = player_cam_transform.rotation();
        } else {
            player_cam.is_active = true;
            free_cam.is_active = false;
        }
    }
    *last_state = game_resource.in_editor;
}
