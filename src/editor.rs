use bevy::{
    camera::{Viewport, visibility::RenderLayers},
    camera_controller::free_camera::FreeCamera,
    color::palettes::css::BLUE,
    core_pipeline::tonemapping::Tonemapping,
    ecs::{VariantDefaults, system::entity_command::observe},
    feathers::{
        constants::icons,
        containers::{
            flex_spacer, group, group_body, group_header, pane, pane_body, pane_header,
            pane_header_divider, subpane, subpane_body, subpane_header,
        },
        controls::{ButtonVariant, FeathersButton, FeathersNumberInput, FeathersToolButton},
        dark_theme::create_dark_theme,
        display::{icon, label, label_dim, label_small},
        palette,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    input::{ButtonState, gamepad, mouse::MouseButtonInput},
    math::VectorSpace,
    post_process::bloom::Bloom,
    prelude::*,
    ui_widgets::{Activate, ValueChange},
    window::{CursorOptions, PrimaryWindow},
};

use crate::{
    player::{CameraRotation, Player, PlayerCam},
    resources::GameResources,
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
                ),
            )
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
    pub size: UVec2,
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

#[derive(Component, Clone, Copy, Default, VariantDefaults)]
enum PositionVec3Field {
    #[default]
    X,
    Y,
    Z,
}

#[derive(Component, Clone, Copy, Default)]
pub struct EntityInspectorWindow;

fn entity_inspector() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Start,
            padding: px(8),
            row_gap: px(8),
            width: percent(30),
            min_width: px(200),
        }
        EntityInspectorWindow
        on(|on_drag: On<Pointer<Drag>>, mut query: Query<&mut UiTransform>| {
            if let Ok(mut transform) = query.get_mut(on_drag.event_target()) {
                if let (Val::Px(x), Val::Px(y)) = (transform.translation.x, transform.translation.y) {
                    transform.translation.x = Val::Px(x + on_drag.delta.x);
                    transform.translation.y = Val::Px(y + on_drag.delta.y);
                }
            }
        })
        Children [
            (
                pane() Children [
                    pane_header() Children [
                        Text("Entity Inspector")
                        flex_spacer(),
                        @FeathersToolButton {
                            @variant: ButtonVariant::Plain,
                        }
                        on(|_on_click: On<Activate>, query: Query<Entity, With<EntityInspectorWindow>>, mut commands: Commands| {
                            if let Ok(entity) = query.single() {
                                commands.entity(entity).despawn();
                            }
                        })
                        Children [
                            icon(icons::X)
                        ],

                    ],
                    (
                        pane_body() Children [
                            label_dim("Entity Name"),
                            group()
                            Children [
                                group_header() Children [
                                    (Text("Transform") ThemedText),
                                ],
                                group_body()
                                Children [
                                    label("Translation"),
                                    Node {
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Row,
                                        column_gap: px(6),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::SpaceBetween,
                                    }
                                    Children [
                                        (
                                            @FeathersNumberInput {
                                                @sigil_color: tokens::TEXT_INPUT_X_AXIS,
                                                @label_text: "X",
                                            }
                                            PositionVec3Field::X
                                            Node {
                                                flex_grow: 1.0,
                                            }
                                            BorderColor::all(palette::X_AXIS)
                                        ),
                                        (
                                            @FeathersNumberInput {
                                                @sigil_color: tokens::TEXT_INPUT_Y_AXIS,
                                                @label_text: "Y",
                                            }
                                            PositionVec3Field::Y
                                            Node {
                                                flex_grow: 1.0,
                                            }
                                        ),
                                        (
                                            @FeathersNumberInput {
                                                @sigil_color: tokens::TEXT_INPUT_Z_AXIS,
                                                @label_text: "Z",
                                            }
                                            PositionVec3Field::Z
                                            Node {
                                                flex_grow: 1.0,
                                            }
                                        ),
                                    ],
                                ],
                            ]
                        ]
                    ),
                ]
            ),
        ]
    }
}

fn update_viewport(
    mut cam_q: Query<&mut Camera, With<PlayerCam>>,
    editor_var: Res<EditorVar>,
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

fn handle_editor_toggle(
    mut commands: Commands,
    mut cam_q: Query<
        (Entity, &mut Transform, &GlobalTransform, &mut Camera),
        (With<PlayerCam>, Without<UICam>),
    >,
    player_q: Single<(Entity, &CameraRotation), With<Player>>,
    mut toggled: MessageReader<EditorToggled>,
    mut game_resource: ResMut<GameResources>,
    ui_cam_q: Single<&mut Camera, (With<UICam>, Without<PlayerCam>)>,
) {
    let Ok((cam_e, mut cam_t, cam_gt, mut player_cam)) = cam_q.single_mut() else {
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

fn handle_picking(
    game_resource: Res<GameResources>,
    mut editor_var: ResMut<EditorVar>,
    mut mouse_button: MessageReader<MouseButtonInput>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_q: Single<(&Camera, &GlobalTransform), With<PlayerCam>>,
    mut ray_cast: MeshRayCast,
    gizmo_state: Res<TransformGizmoState>,
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
        } else {
            editor_var.selected_entity = None;
        }
    }
}

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
