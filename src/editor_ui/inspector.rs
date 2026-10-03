use bevy::ecs::VariantDefaults;
use bevy::ecs::message::MessageReader;
use bevy::feathers::constants::icons;
use bevy::feathers::containers::{
    flex_spacer, group, group_body, group_header, pane, pane_body, pane_header,
};
use bevy::feathers::controls::{
    ButtonVariant, FeathersNumberInput, FeathersToolButton, UpdateNumberInput,
};
use bevy::feathers::display::{icon, label, label_dim};
use bevy::feathers::theme::ThemedText;
use bevy::feathers::{palette, tokens};
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, ValueChange};

use crate::editor::{EditorVar, SelectedEntityChange};

// my own axis enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialAxis {
    X,
    Y,
    Z,
}

pub struct InspectorWindowPl;

impl Plugin for InspectorWindowPl {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_entity_inspector, handle_transform_updates))
            .add_systems(Startup, ui.spawn())
            .add_message::<UpdateEntityPosition>();
    }
}

// the message send when the use input a transform
#[derive(Message)]
pub struct UpdateEntityPosition {
    pub transform_type: TransformType,
    pub entity: Entity,
    pub axis: SpatialAxis,
    pub value: f32,
}

#[derive(Component, Clone, Copy, Default, VariantDefaults)]
pub enum TransformType {
    #[default]
    Scale,
    Rotation,
    Translation,
}

// components to query the transforms input
#[derive(Component, Clone, Copy, Default, VariantDefaults)]
enum TranslationVec3Field {
    #[default]
    X,
    Y,
    Z,
}
#[derive(Component, Clone, Copy, Default, VariantDefaults)]
enum ScaleVec3Field {
    #[default]
    X,
    Y,
    Z,
}
#[derive(Component, Clone, Copy, Default, VariantDefaults)]
enum RotationVec3Field {
    #[default]
    X,
    Y,
    Z,
}

//the component on the whole windows
#[derive(Component, Clone, Copy, Default)]
struct EntityInspectorWindow;

#[derive(Component, Clone, Copy, Default)]
struct EntityInspectorWindowContent;

// initialize the entity_inspector window
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
                        EntityInspectorWindowContent
                        pane_body()
                        Children[Text("No Entity Selected")]
                    ),
                ]
            ),
        ]
    }
}

//update the entity inspector window
fn update_entity_inspector(
    node_entity_q: Query<Entity, With<EntityInspectorWindowContent>>,
    mut commands: Commands,
    mut msg_reader: MessageReader<SelectedEntityChange>,
    editor_var: Res<EditorVar>,
    info_q: Query<&Name>,
    q_vec3_translation: Query<(Entity, &TranslationVec3Field)>,
    q_vec3_scale: Query<(Entity, &ScaleVec3Field)>,
    q_vec3_rotation: Query<(Entity, &RotationVec3Field)>,
    transform_q: Query<&Transform>,
) {
    if editor_var.selected_entity.is_some() {
        let target_entity = editor_var.selected_entity.unwrap();
        if let Ok(transform) = transform_q.get(target_entity) {
            for (vec3_input_ent, axis) in q_vec3_translation.iter() {
                let new_value = match axis {
                    TranslationVec3Field::X => transform.translation.x,
                    TranslationVec3Field::Y => transform.translation.y,
                    TranslationVec3Field::Z => transform.translation.z,
                };

                commands.trigger(UpdateNumberInput {
                    entity: vec3_input_ent,
                    value: bevy::feathers::controls::NumberInputValue::F32(
                        (new_value * 100.0).round() / 100.0,
                    ),
                });
            }
            for (vec3_input_ent, axis) in q_vec3_scale.iter() {
                let new_value = match axis {
                    ScaleVec3Field::X => transform.scale.x,
                    ScaleVec3Field::Y => transform.scale.y,
                    ScaleVec3Field::Z => transform.scale.z,
                };

                commands.trigger(UpdateNumberInput {
                    entity: vec3_input_ent,
                    value: bevy::feathers::controls::NumberInputValue::F32(
                        (new_value * 100.0).round() / 100.0,
                    ),
                });
            }
            for (vec3_input_ent, axis) in q_vec3_rotation.iter() {
                let new_value = match axis {
                    RotationVec3Field::X => transform.rotation.to_euler(EulerRot::XYZ).0,
                    RotationVec3Field::Y => transform.rotation.to_euler(EulerRot::XYZ).1,
                    RotationVec3Field::Z => transform.rotation.to_euler(EulerRot::XYZ).2,
                };

                commands.trigger(UpdateNumberInput {
                    entity: vec3_input_ent,
                    value: bevy::feathers::controls::NumberInputValue::F32(
                        (new_value * 100.0).round() / 100.0,
                    ),
                });
            }
        }
    }
    for _ in msg_reader.read() {
        let Ok(entity) = node_entity_q.single() else {
            return;
        };
        if editor_var.selected_entity.is_some() {
            let target_entity = editor_var.selected_entity.unwrap();
            let name = if info_q.get(editor_var.selected_entity.unwrap()).is_ok() {
                info_q
                    .get(editor_var.selected_entity.unwrap())
                    .unwrap()
                    .to_string()
            } else {
                commands.entity(target_entity).id().to_string()
            };
            commands.entity(entity).despawn_children();
            let child = commands
                .spawn_scene(bsn! {
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        width: percent(100.0),
                    }
                    Children [
                        label_dim(format!("Name: {}", name)),
                        group() Children [
                            group_header() Children [
                                (Text("Transform") ThemedText),
                            ],
                            group_body() Children [
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
                                        TranslationVec3Field::X
                                        Node { flex_grow: 1.0 }
                                        BorderColor::all(palette::X_AXIS)
                                        on(move |value_change: On<ValueChange<f32>>, mut writer: MessageWriter<UpdateEntityPosition>| {
                                            if value_change.is_final {
                                                writer.write(UpdateEntityPosition {
                                                    transform_type: TransformType::Translation,
                                                    entity:  target_entity,
                                                    axis: SpatialAxis::X,
                                                    value: value_change.value,
                                                });
                                            }
                                        })
                                    ),
                                    (
                                        @FeathersNumberInput {
                                            @sigil_color: tokens::TEXT_INPUT_Y_AXIS,
                                            @label_text: "Y",
                                        }
                                        TranslationVec3Field::Y
                                        Node { flex_grow: 1.0 }
                                        on(move |value_change: On<ValueChange<f32>>, mut writer: MessageWriter<UpdateEntityPosition>| {
                                            if value_change.is_final {
                                                writer.write(UpdateEntityPosition {
                                                    transform_type: TransformType::Translation,
                                                    entity:  target_entity,
                                                    axis: SpatialAxis::Y,
                                                    value: value_change.value,
                                                });
                                            }
                                        })
                                    ),
                                    (
                                        @FeathersNumberInput {
                                            @sigil_color: tokens::TEXT_INPUT_Z_AXIS,
                                            @label_text: "Z",
                                        }
                                        TranslationVec3Field::Z
                                        Node { flex_grow: 1.0 }
                                        on(move |value_change: On<ValueChange<f32>>, mut writer: MessageWriter<UpdateEntityPosition>| {
                                            if value_change.is_final {
                                                writer.write(UpdateEntityPosition {
                                                    transform_type: TransformType::Translation,
                                                    entity: target_entity,
                                                    axis: SpatialAxis::Z,
                                                    value: value_change.value,
                                                });
                                            }
                                        })
                                    ),
                                ],
                            ],
                            group_body() Children [
                                label("Scale"),
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
                                        ScaleVec3Field::X
                                        Node { flex_grow: 1.0 }
                                        BorderColor::all(palette::X_AXIS)
                                        on(move |value_change: On<ValueChange<f32>>, mut writer: MessageWriter<UpdateEntityPosition>| {
                                            if value_change.is_final {
                                                writer.write(UpdateEntityPosition {
                                                    transform_type: TransformType::Scale,
                                                    entity:  target_entity,
                                                    axis: SpatialAxis::X,
                                                    value: value_change.value,
                                                });
                                            }
                                        })
                                    ),
                                    (
                                        @FeathersNumberInput {
                                            @sigil_color: tokens::TEXT_INPUT_Y_AXIS,
                                            @label_text: "Y",
                                        }
                                        ScaleVec3Field::Y
                                        Node { flex_grow: 1.0 }
                                        on(move |value_change: On<ValueChange<f32>>, mut writer: MessageWriter<UpdateEntityPosition>| {
                                            if value_change.is_final {
                                                writer.write(UpdateEntityPosition {
                                                    transform_type: TransformType::Scale,
                                                    entity:  target_entity,
                                                    axis: SpatialAxis::Y,
                                                    value: value_change.value,
                                                });
                                            }
                                        })
                                    ),
                                    (
                                        @FeathersNumberInput {
                                            @sigil_color: tokens::TEXT_INPUT_Z_AXIS,
                                            @label_text: "Z",
                                        }
                                        ScaleVec3Field::Z
                                        Node { flex_grow: 1.0 }
                                        on(move |value_change: On<ValueChange<f32>>, mut writer: MessageWriter<UpdateEntityPosition>| {
                                            if value_change.is_final {
                                                writer.write(UpdateEntityPosition {
                                                    transform_type: TransformType::Scale,
                                                    entity: target_entity,
                                                    axis: SpatialAxis::Z,
                                                    value: value_change.value,
                                                });
                                            }
                                        })
                                    ),
                                ],
                            ],
                            group_body() Children [
                                label("Rotation"),
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
                                        RotationVec3Field::X
                                        Node { flex_grow: 1.0 }
                                        BorderColor::all(palette::X_AXIS)
                                        on(move |value_change: On<ValueChange<f32>>, mut writer: MessageWriter<UpdateEntityPosition>| {
                                            if value_change.is_final {
                                                writer.write(UpdateEntityPosition {
                                                    transform_type: TransformType::Rotation,
                                                    entity:  target_entity,
                                                    axis: SpatialAxis::X,
                                                    value: value_change.value,
                                                });
                                            }
                                        })
                                    ),
                                    (
                                        @FeathersNumberInput {
                                            @sigil_color: tokens::TEXT_INPUT_Y_AXIS,
                                            @label_text: "Y",
                                        }
                                        RotationVec3Field::Y
                                        Node { flex_grow: 1.0 }
                                        on(move |value_change: On<ValueChange<f32>>, mut writer: MessageWriter<UpdateEntityPosition>| {
                                            if value_change.is_final {
                                                writer.write(UpdateEntityPosition {
                                                    transform_type: TransformType::Rotation,
                                                    entity:  target_entity,
                                                    axis: SpatialAxis::Y,
                                                    value: value_change.value,
                                                });
                                            }
                                        })
                                    ),
                                    (
                                        @FeathersNumberInput {
                                            @sigil_color: tokens::TEXT_INPUT_Z_AXIS,
                                            @label_text: "Z",
                                        }
                                        RotationVec3Field::Z
                                        Node { flex_grow: 1.0 }
                                        on(move |value_change: On<ValueChange<f32>>, mut writer: MessageWriter<UpdateEntityPosition>| {
                                            if value_change.is_final {
                                                writer.write(UpdateEntityPosition {
                                                    transform_type: TransformType::Rotation,
                                                    entity: target_entity,
                                                    axis: SpatialAxis::Z,
                                                    value: value_change.value,
                                                });
                                            }
                                        })
                                    ),
                                ],
                            ],
                        ],
                    ]
                })
                .id();
            commands.entity(entity).add_child(child);
        } else {
            commands.entity(entity).despawn_children();
            let child = commands.spawn(Text("No Entity Selected".to_string())).id();
            commands.entity(entity).add_child(child);
        }
    }
}

// handle the entity transformations from the entity editor
fn handle_transform_updates(
    mut reader: MessageReader<UpdateEntityPosition>,
    mut query: Query<&mut Transform>,
) {
    for msg in reader.read() {
        if let Ok(mut transform) = query.get_mut(msg.entity) {
            match msg.transform_type {
                TransformType::Translation => match msg.axis {
                    SpatialAxis::X => transform.translation.x = msg.value,
                    SpatialAxis::Y => transform.translation.y = msg.value,
                    SpatialAxis::Z => transform.translation.z = msg.value,
                },
                TransformType::Scale => match msg.axis {
                    SpatialAxis::X => transform.scale.x = msg.value,
                    SpatialAxis::Y => transform.scale.y = msg.value,
                    SpatialAxis::Z => transform.scale.z = msg.value,
                },
                TransformType::Rotation => {
                    let (mut yaw, mut pitch, mut roll) = transform.rotation.to_euler(EulerRot::YXZ);

                    match msg.axis {
                        SpatialAxis::X => pitch = msg.value,
                        SpatialAxis::Y => yaw = msg.value,
                        SpatialAxis::Z => roll = msg.value,
                    }
                    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
                }
            }
        }
    }
}
