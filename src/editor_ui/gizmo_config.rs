use bevy::{
    ecs::{VariantDefaults, event::Trigger},
    feathers::{
        constants::icons,
        containers::{flex_spacer, pane, pane_body, pane_header},
        controls::{ButtonVariant, FeathersButton, FeathersToolButton},
        display::icon,
        rounded_corners::RoundedCorners,
        theme::ThemedText,
    },
    prelude::*,
    ui_widgets::Activate,
};

pub struct GizmoConfigWindowPl;

impl Plugin for GizmoConfigWindowPl {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ui.spawn())
            .insert_resource(WindowVar::default())
            .add_systems(Update, (update_button_style, handle_button_click))
            .add_message::<ButtonClicked>();
    }
}

#[derive(Component, Copy, Clone, Default)]
struct GizmoConfigWindow;

#[derive(Component, Copy, Clone, Default, VariantDefaults, PartialEq)]
enum GizmoType {
    #[default]
    Move,
    Rotate,
    Scale,
}

#[derive(Resource)]
struct WindowVar {
    selected_type: GizmoType,
}

impl Default for WindowVar {
    fn default() -> Self {
        Self {
            selected_type: GizmoType::Move,
        }
    }
}

#[derive(Message, Default)]
struct ButtonClicked(GizmoType);

fn ui() -> impl Scene {
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
        on(|on_drag: On<Pointer<Drag>>, mut query: Query<&mut UiTransform>| {
            if let Ok(mut transform) = query.get_mut(on_drag.event_target()) {
                if let (Val::Px(x), Val::Px(y)) = (transform.translation.x, transform.translation.y) {
                    transform.translation.x = Val::Px(x + on_drag.delta.x);
                    transform.translation.y = Val::Px(y + on_drag.delta.y);
                }
            }
        })
        GizmoConfigWindow
        Children [
            (
                pane() Children [
                    pane_header() Children [
                        Text("GizmoConfig")
                        flex_spacer(),
                        @FeathersToolButton {
                            @variant: ButtonVariant::Plain,
                        }
                        on(|_on_click: On<Activate>, query: Query<Entity, With<GizmoConfigWindow>>, mut commands: Commands| {
                            if let Ok(entity) = query.single() {
                                commands.entity(entity).despawn();
                            }
                        })
                        Children [
                            icon(icons::X)
                        ],

                    ],
                    (
                        pane_body()
                        Children[(
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    column_gap: px(1),
                }
                Children [
                    (
                        @FeathersButton {
                            @caption: bsn! { Text("Move") ThemedText },
                            @corners: RoundedCorners::Left,
                        }
                        Node {
                            flex_grow: 1.0,
                        }
                        AccessibleLabel("Move")
                        on(|_activate: On<Activate>| {
                            info!("Left button clicked!");
                        })
                        GizmoType::Move
                        on(move |_: On<Activate>, mut writer: MessageWriter<ButtonClicked>| {
                            writer.write(ButtonClicked(GizmoType::Move));
                        })
                    ),
                    (
                        @FeathersButton {
                            @caption: bsn! { Text("Rotate") ThemedText },
                            @corners: RoundedCorners::None,
                        }
                        Node {
                            flex_grow: 1.0,
                        }
                        AccessibleLabel("Rotate")
                        on(|_activate: On<Activate>| {
                            info!("Center button clicked!");
                        })
                        GizmoType::Rotate
                        on(move |_: On<Activate>, mut writer: MessageWriter<ButtonClicked>| {
                            writer.write(ButtonClicked(GizmoType::Rotate));
                        })
                    ),
                    (
                        @FeathersButton {
                            @caption: bsn! { Text("Scale") ThemedText },
                            @corners: RoundedCorners::Right,
                        }
                        Node {
                            flex_grow: 1.0,
                        }
                        AccessibleLabel("Scale")
                        on(|_activate: On<Activate>| {
                            info!("Right button clicked!");
                        })
                        GizmoType::Scale
                        on(move |_: On<Activate>, mut writer: MessageWriter<ButtonClicked>| {
                            writer.write(ButtonClicked(GizmoType::Scale));
                        })

                    ),
                ]
            ),]
                    ),
                ]
            ),
        ]
    }
}

fn update_button_style(
    window_var: Res<WindowVar>,
    mut button_query: Query<(&GizmoType, &mut ButtonVariant), With<FeathersButton>>,
) {
    if !window_var.is_changed() {
        return;
    }

    for (gizmo_type, mut variant) in button_query.iter_mut() {
        let is_selected = *gizmo_type == window_var.selected_type;

        match gizmo_type {
            GizmoType::Move => {
                *variant = if is_selected {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Normal
                };
            }
            GizmoType::Rotate => {
                *variant = if is_selected {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Normal
                };
            }
            GizmoType::Scale => {
                *variant = if is_selected {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Normal
                };
            }
        }
    }
}

fn handle_button_click(
    mut reader: MessageReader<ButtonClicked>,
    mut window_var: ResMut<WindowVar>,
    mut gizmo_settings: ResMut<TransformGizmoSettings>,
) {
    for msg in reader.read() {
        window_var.selected_type = msg.0;
        match msg.0 {
            GizmoType::Move => gizmo_settings.mode = TransformGizmoMode::Translate,
            GizmoType::Rotate => gizmo_settings.mode = TransformGizmoMode::Rotate,
            GizmoType::Scale => gizmo_settings.mode = TransformGizmoMode::Scale,
        }
    }
}
