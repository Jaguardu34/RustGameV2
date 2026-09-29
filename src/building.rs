use avian3d::{collision::collider::{Collider, CollidingEntities}, dynamics::rigid_body::RigidBody};
use bevy::{ecs::entity, input::mouse::{MouseScrollUnit, MouseWheel}, light::{NotShadowCaster, NotShadowReceiver}, prelude::*};

use crate::{player::PlayerCam, resources::GameResources};

pub struct BuildingPlugin;


impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_overlay);
    }
}

#[derive(Component)]
pub struct Overlay;

#[derive(Component)]
pub struct PlacedByPlayer;

pub struct BuildDistance {
    value: f32
}

impl Default for BuildDistance {
    fn default() -> Self {
        Self { value: 4.0 }
    }
}

const MAX_BUILD_DISTANCE: f32 = 10.0;
const MIN_BUILD_DISTANCE: f32 = 2.0;

fn update_overlay(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    player_cam_q: Query<&GlobalTransform, With<PlayerCam>>,
    mut game_resource: ResMut<GameResources>,
    mut build_distance: Local<BuildDistance>,
    mut overlay_spawned: Local<bool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut overlay_q: Query<(Entity, &mut Transform), With<Overlay>>,
    mut ev_mouse_scroll: MessageReader<MouseWheel>
    ) {

    if !game_resource.mouse_grabbed {
        return
    }

    if keys.just_pressed(KeyCode::KeyQ) {
        game_resource.building = !game_resource.building;
        
    }

    let Ok(player_cam_transform) = player_cam_q.single() else {return;};
    
    let pos = player_cam_transform.translation() + player_cam_transform.forward() * build_distance.value;


    

    if game_resource.building{
        if !*overlay_spawned {
            commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                    MeshMaterial3d(materials.add(Color::srgba(0.0, 1.0, 0.0, 0.8))),
                    Transform::from_translation(pos),
                    Overlay,
                    NotShadowCaster,
                    NotShadowReceiver,
            ));
            *overlay_spawned = true;
        } else {
            for (_entity, mut overlay_transform) in overlay_q.iter_mut() {
                overlay_transform.translation = pos;
            }
        }
        for ev in ev_mouse_scroll.read() {
            build_distance.value += ev.y;
        }
        build_distance.value = build_distance.value.clamp(MIN_BUILD_DISTANCE, MAX_BUILD_DISTANCE);

        if buttons.just_pressed(MouseButton::Left) {
                commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
                        Transform::from_translation(pos),
                        PlacedByPlayer,
                        RigidBody::Static,
                        Collider::cuboid(1.0, 1.0, 1.0)
            ));
        }
    } else {
        for (entity, _overlay_transform) in overlay_q {
            commands.entity(entity).despawn();
        }
        *overlay_spawned = false;
    }

}
