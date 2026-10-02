use bevy::{
    camera::{Viewport, visibility::RenderLayers},
    camera_controller::free_camera::FreeCamera,
    ecs::VariantDefaults,
    feathers::{
        constants::icons,
        containers::{flex_spacer, group, group_body, group_header, pane, pane_body, pane_header},
        controls::{ButtonVariant, FeathersNumberInput, FeathersToolButton, UpdateNumberInput},
        dark_theme::create_dark_theme,
        display::{icon, label, label_dim},
        palette,
        theme::{ThemedText, UiTheme},
        tokens,
    },
    input::{ButtonState, mouse::MouseButtonInput},
    prelude::*,
    ui_widgets::{Activate, ValueChange},
    window::PrimaryWindow,
};

use crate::{
    player::{CameraRotation, Player, PlayerCam},
    resources::GameResources,
};

use crate::editor_ui::entity_inspector::{
    UpdateEntityPosition, entity_inspector, handle_transform_updates, update_entity_inspector,
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EditorVar::default())
            .add_systems(Startup, setup_ui_cam)
            .add_systems(Startup, ui.spawn())
            .insert_resource(UiTheme(create_dark_theme()))
            .add_systems(
                Update,
                (
                    handle_inputs,
                    handle_editor_toggle,
                    sync_gizmo_cam_projection,
                    handle_picking,
                    change_selected_entity,
                    update_viewport,
                    update_entity_inspector,
                    handle_transform_updates,
                ),
            )
            .add_message::<EditorToggled>()
            .add_message::<SelectedEntityChange>()
            .add_message::<UpdateEntityPosition>();
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
    pub size: UVec2,
}
#[derive(Component)]
pub struct UICam;

#[derive(Message, Default)]
struct EditorToggled(bool);

#[derive(Message)]
pub struct SelectedEntityChange;

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

fn setup_ui_cam(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.12)),
            is_active: false,
            ..Default::default()
        },
        UICam,
        IsDefaultUiCamera,
    ));
}

//create Feather UI
fn ui() -> impl SceneList {
    bsn_list![entity_inspector()]
}

// update the gameviewport to the right place on the screen
fn update_viewport(
    mut cam_q: Query<&mut Camera, With<PlayerCam>>,
    game_resource: Res<GameResources>,
) {
    let Ok(mut cam) = cam_q.single_mut() else {
        return;
    };

    if !game_resource.in_editor {
        cam.viewport = None
    } else {
        cam.viewport = Some(Viewport {
            physical_position: UVec2 { x: 0, y: 800 },
            physical_size: UVec2 { x: 1000, y: 500 },
            ..Default::default()
        })
    }
}

// handle the various keymaps for the editor
fn handle_inputs(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_resource: ResMut<GameResources>,
    mut toggle_message_writer: MessageWriter<EditorToggled>,
) {
    if keys.just_pressed(KeyCode::F12) {
        game_resource.in_editor = !game_resource.in_editor;
        toggle_message_writer.write(EditorToggled(game_resource.in_editor));
    }
}

// handle the cam changement from player_cam behaviour to freecam behavior, (same cam for simplicity)
fn handle_editor_toggle(
    mut commands: Commands,
    mut cam_q: Query<(Entity, &mut Transform, &GlobalTransform), (With<PlayerCam>, Without<UICam>)>,
    player_q: Single<(Entity, &CameraRotation), With<Player>>,
    mut toggled: MessageReader<EditorToggled>,
    mut game_resource: ResMut<GameResources>,
    ui_cam_q: Single<&mut Camera, (With<UICam>, Without<PlayerCam>)>,
) {
    let Ok((cam_e, mut cam_t, cam_gt)) = cam_q.single_mut() else {
        return;
    };
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
            *cam_t =
                Transform::from_xyz(0.0, 1.5, 0.0).with_rotation(Quat::from_rotation_x(rot.pitch));
            ui_cam.is_active = false;
        }
    }
}

// custom meshPicking function
fn handle_picking(
    game_resource: Res<GameResources>,
    mut editor_var: ResMut<EditorVar>,
    mut mouse_button: MessageReader<MouseButtonInput>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_q: Single<(&Camera, &GlobalTransform), With<PlayerCam>>,
    mut ray_cast: MeshRayCast,
    gizmo_state: Res<TransformGizmoState>,
    mut msg_writer: MessageWriter<SelectedEntityChange>,
) {
    if !game_resource.in_editor {
        return;
    }

    let (camera, camera_transform) = *camera_q;

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let physical_cursor = cursor_position * window.scale_factor();

    if !camera
        .physical_viewport_rect()
        .unwrap()
        .contains(physical_cursor.as_uvec2())
    {
        return;
    }

    for button in mouse_button.read() {
        if button.state != ButtonState::Pressed || button.button != MouseButton::Left {
            continue;
        }
        if gizmo_state.hovered_axis.is_some() || gizmo_state.active {
            continue;
        }

        let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            continue;
        };

        let settings = MeshRayCastSettings::default().with_visibility(RayCastVisibility::Visible);
        if let Some((entity, _hit)) = ray_cast.cast_ray(ray, &settings).first() {
            editor_var.selected_entity = Some(*entity);
            msg_writer.write(SelectedEntityChange);
        } else {
            editor_var.selected_entity = None;
            msg_writer.write(SelectedEntityChange);
        }
    }
}

// update the game resource and change the TransformGizmoFocus to the entity of EditorVar.selected_entity
fn change_selected_entity(
    mut commands: Commands,
    editor_var: Res<EditorVar>,
    transform_gizmo_focus_q: Query<Entity, With<TransformGizmoFocus>>,
    game_resource: Res<GameResources>,
) {
    if game_resource.in_editor {
        if editor_var.selected_entity.is_some() {
            let entity = editor_var.selected_entity.unwrap();
            commands.entity(entity).insert(TransformGizmoFocus);

            for entity_with_focus in transform_gizmo_focus_q {
                if entity_with_focus != entity {
                    commands
                        .entity(entity_with_focus)
                        .remove::<TransformGizmoFocus>();
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
