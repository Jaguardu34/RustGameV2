use bevy::{camera::{Viewport, visibility::RenderLayers}, camera_controller::free_camera::FreeCamera, core_pipeline::tonemapping::Tonemapping, input::{ButtonState, gamepad, mouse::MouseButtonInput}, math::VectorSpace, post_process::bloom::Bloom, prelude::*, window::{CursorOptions, PrimaryWindow}};
use bevy_egui::{EguiContext, EguiContexts, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui::{self, Pos2}};
use bevy_inspector_egui::bevy_inspector;

use crate::{player::{CameraRotation, Player, PlayerCam}, resources::GameResources};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(EditorVar::default())
            .add_systems(Startup, setup_ui_cam)
            .add_systems(EguiPrimaryContextPass, (inspector_window, game_view_window, add_elements_window, inspect_element_window))
            .add_systems(Update, (
                handle_inputs, 
                handle_editor_toggle, 
                sync_gizmo_cam_projection, 
                handle_picking, 
                change_selected_entity, 
                update_viewport
            ))
            .add_message::<EditorToggled>();
    }
}

#[derive(Resource, Default)]
pub struct EditorVar {
    pub selected_entity: Option<Entity>,
    pub game_view_info: GameView,
    pub pointer_on_viewport: bool,
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

// UI
fn add_elements_window(
    mut commands: Commands,
    mut context: EguiContexts,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut editor_var: ResMut<EditorVar>
) {
    egui::Window::new("Add Object").show(context.ctx_mut().expect("cant use context"), |ui| {
        if ui.button("Cube").clicked() {
            let entity = commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MeshMaterial3d(materials.add(Color::WHITE)),
                Transform::from_translation(Vec3::ZERO),
            ));
            editor_var.selected_entity = Some(entity.id());
        }
    });
}

fn inspect_element_window(
    world: &mut World
) {

    let (mut context, selected_entity) = {
        let mut context = world
            .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
            .single(world)
            .expect("cant find egui context")
            .clone();

        let selected_entity = world
            .get_resource::<EditorVar>()
            .and_then(|var| var.selected_entity);

        (context, selected_entity)
    };

    egui::Window::new("Inspect Element").show(context.get_mut(), |ui| {
        if selected_entity.is_some() {
            bevy_inspector::ui_for_entity(world, selected_entity.unwrap(), ui);
        } else {
            ui.label("Select an entity first");
        }
    });
}

fn game_view_window(
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
            editor_var.pointer_on_viewport = ui.ui_contains_pointer();
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
        });
    });
}


fn update_viewport(
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
    camera_q: Single<(&Camera, &GlobalTransform), With<PlayerCam>>,
    mut ray_cast: MeshRayCast,
    gizmo_state: Res<TransformGizmoState>
) {
    if !game_resource.in_editor {
        return;
    }

    let (camera, camera_transform) = *camera_q;

    for button in mouse_button.read() {
        if button.state != ButtonState::Pressed || button.button != MouseButton::Left {
            continue;
        }
        if !editor_var.pointer_on_viewport {
            continue;
        }
        if gizmo_state.hovered_axis.is_some() || gizmo_state.active {
            continue;
        }
        let Some(cursor_position) = window.cursor_position() else { continue };
        let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else { continue };

        let settings = MeshRayCastSettings::default()
            .with_visibility(RayCastVisibility::Visible); 

        if let Some((entity, hit)) = ray_cast.cast_ray(ray, &settings).first() {
            editor_var.selected_entity = Some(*entity);
        } else {
            editor_var.selected_entity = None; 
        }
    }
}


fn change_selected_entity(mut commands: Commands, editor_var: Res<EditorVar>, transform_gizmo_focus_q: Query<Entity, With<TransformGizmoFocus>>, game_resource: Res<GameResources>) {
    if game_resource.in_editor {
        if editor_var.selected_entity.is_some() {
            let entity = editor_var.selected_entity.unwrap();
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