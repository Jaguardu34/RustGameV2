use bevy::{camera::{Viewport, visibility::RenderLayers}, camera_controller::free_camera::FreeCamera, core_pipeline::tonemapping::Tonemapping, input::{ButtonState, gamepad, mouse::MouseButtonInput}, math::VectorSpace, post_process::bloom::Bloom, prelude::*, window::{CursorOptions, PrimaryWindow}};
use bevy_egui::{EguiContext, EguiContexts, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui::{self, Pos2}};

use crate::{player::{CameraRotation, Player, PlayerCam}, resources::GameResources};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(EditorVar::default())
            .add_systems(Startup, setup_ui_cam)
            .add_systems(EguiPrimaryContextPass, (setup_windows, inspector_window, setup_game_view_window))
            .add_systems(Update, (
                handle_inputs, 
                handle_editor_toggle, 
                sync_gizmo_cam_projection, 
                handle_picking, 
                change_selected_entity, 
                setup_game_cam_viewport
            ))
            .add_message::<EditorToggled>();
    }
}

#[derive(Resource, Default)]
pub struct EditorVar {
    pub entity_selected: Option<Entity>,
    pub game_view_info: GameView
}

#[derive(Default, Clone, Copy)]
pub struct GameView {
    pub pos: UVec2,
    pub size: UVec2
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

#[derive(Component)]
pub struct UICam;

fn setup_ui_cam(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.12)),
            is_active: false,
            ..Default::default()
        },
        UICam,
        PrimaryEguiContext,
    ));
}


fn setup_windows(
    mut context: EguiContexts,
    mut player_q: Query<&mut Player>,
) {
    let Ok(mut player) = player_q.single_mut() else {return;};
    egui::Window::new("Player Var").default_pos(Pos2::new(500.0, 500.0)).show(context.ctx_mut().expect("can't use egui context"), |ui| {
        ui.add(egui::Slider::new(&mut player.speed, 1.0..=20.0));

    });
}


fn setup_game_cam_viewport(
    mut cam_q: Query<&mut Camera, With<PlayerCam>>,
    editor_var: Res<EditorVar>,
    game_resource: Res<GameResources>
) {
    let Ok(mut cam) = cam_q.single_mut() else {return};

    if !game_resource.in_editor {
        cam.viewport = None
    } else {
        cam.viewport = Some(Viewport { physical_position: editor_var.game_view_info.pos, physical_size: editor_var.game_view_info.size, ..Default::default()})
    }

}

fn setup_game_view_window(
    mut context: EguiContexts,
    mut editor_var: ResMut<EditorVar>
) {
    let response = egui::Window::new("Game_View")
        .min_size(egui::Vec2::new(200.0, 200.0))
        .resizable(true)
        .interactable(true)
        .movable(false)
        .scroll(false)
        .collapsible(false)
        .show(context.ctx_mut().expect("cant_use_context"), |ui| {
            ui.take_available_space();
        });

    if let Some(r) = response {
        let rect = r.response.rect;

        editor_var.game_view_info.pos = UVec2 { x: rect.min.x as u32, y: rect.min.y as u32 + 20 };
        editor_var.game_view_info.size = UVec2 { x: rect.size().x as u32, y: rect.size().y as u32 - 20 }
    }
}



fn inspector_window(world: &mut World) {
    let Ok(mut egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single_mut(world)
        .map(|c| c.clone())
    else {
        return;
    };

    egui::Window::new("World Inspector").show(egui_context.get_mut(), |ui| {
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
    mut cam_q: Query<(Entity, &mut Transform, &GlobalTransform, &mut Camera), (With<PlayerCam>, Without<UICam>)>,
    player_q: Single<(Entity, &CameraRotation), With<Player>>,
    mut toggled: MessageReader<EditorToggled>,
    mut game_resource: ResMut<GameResources>,
    ui_cam_q: Single<&mut Camera, (With<UICam>, Without<PlayerCam>)>,
) {
    let Ok((cam_e, mut cam_t, cam_gt, mut player_cam)) = cam_q.single_mut() else {return;};
    let (player_e, rot) = *player_q;
    let mut ui_cam = ui_cam_q;

    for message in toggled.read() {
        if message.0 {
            game_resource.mouse_grabbed = false;
            *cam_t = cam_gt.compute_transform();
            commands.entity(player_e).detach_child(cam_e);
            commands
                .entity(cam_e)
                .insert((FreeCamera::default(), RenderLayers::from_layers(&[0, 1])));
            ui_cam.is_active = true;
        } else {
            commands
                .entity(cam_e)
                .remove::<FreeCamera>()
                .insert(RenderLayers::layer(0));
            commands.entity(player_e).add_child(cam_e);
            *cam_t = Transform::from_xyz(0.0, 1.5, 0.0)
                .with_rotation(Quat::from_rotation_x(rot.pitch));
            ui_cam.is_active = false;
        }
    }
}



fn handle_picking(
    game_resource: Res<GameResources>,
    mut editor_var: ResMut<EditorVar>,
    mut mouse_button: MessageReader<MouseButtonInput>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut contexts: EguiContexts,
    camera_q: Single<(&Camera, &GlobalTransform), With<PlayerCam>>,
    mut ray_cast: MeshRayCast,
) {
    if !game_resource.in_editor {
        return;
    }

    let (camera, camera_transform) = *camera_q;

    for button in mouse_button.read() {
        if button.state != ButtonState::Pressed || button.button != MouseButton::Left {
            continue;
        }

        // let ctx = contexts.ctx_mut().expect("cant use context");
        // if ctx.egui_wants_pointer_input() || ctx.is_pointer_over_egui() {
        //     continue;
        // }

        let Some(cursor_position) = window.cursor_position() else { continue };
        let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else { continue };

        let settings = MeshRayCastSettings::default()
            .with_visibility(RayCastVisibility::Visible); 

        // hits: &[(Entity, RayMeshHit)], triés par distance croissante
        if let Some((entity, hit)) = ray_cast.cast_ray(ray, &settings).first() {
            info!("Touché {entity:?} à {:?} (distance {})", hit.point, hit.distance);
            editor_var.entity_selected = Some(*entity);
        } else {
            editor_var.entity_selected = None; 
        }
    }
}


fn change_selected_entity(mut commands: Commands, editor_var: Res<EditorVar>, transform_gizmo_focus_q: Query<Entity, With<TransformGizmoFocus>>, game_resource: Res<GameResources>) {
    if game_resource.in_editor {
        if editor_var.entity_selected.is_some() {
            let entity = editor_var.entity_selected.unwrap();
            commands.entity(entity).insert(TransformGizmoFocus);

            for entity_with_focus in transform_gizmo_focus_q {
                if entity_with_focus != entity {
                    commands.entity(entity_with_focus).remove::<TransformGizmoFocus>();
                }
            }
        } else {
            for entity in transform_gizmo_focus_q {
                commands.entity(entity).remove::<TransformGizmoFocus>();
            }
        }
    } else {
        for entity in transform_gizmo_focus_q {
            commands.entity(entity).remove::<TransformGizmoFocus>();
        }
    }
}