use bevy::prelude::*;

use crate::scene::CameraTarget;

pub struct MyCamPlugin;

impl Plugin for MyCamPlugin {
    fn build(&self, app: &mut App) {
       app
           .add_systems(Startup, create_cam)
           .add_systems(Update, update_cam); 
    }
}

#[derive(Component)]
struct MainCamera;

fn create_cam(mut commands: Commands) {
    commands.spawn((
            Camera3d::default(),
            Transform::from_translation(Vec3 { x: 10.0, y: 2.0, z: 50.0 }).looking_at(Vec3::ZERO, Vec3::Y),
            MainCamera
    ));
}



fn update_cam(mut cameras_query: Query<&mut Transform, With<MainCamera>>, target_query: Query<&GlobalTransform, With<CameraTarget>>) {
    let target = target_query.single().unwrap();
    for mut camera in cameras_query.iter_mut() {
        let last_pos = camera.translation;
        *camera = Transform::from_translation(last_pos).looking_at(target.translation(), Vec3::Y);
    }
}


