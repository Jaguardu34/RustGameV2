use bevy::prelude::*;

pub struct MyScenePlugin;


impl Plugin for MyScenePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (scene.spawn(), add_spheres))
            .add_systems(Update, update_scene);
    }
}

#[derive(Component, Default, Clone)]
struct Movable;

#[derive(Component, Default, Clone)]
pub struct CameraTarget;

fn scene() -> impl SceneList {
    bsn_list! [
        (
            #Sphere
            Mesh3d(asset_value(Sphere::new(1.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
            Transform::from_translation(Vec3::ZERO)
            Movable
            CameraTarget
        ),
        (
            PointLight {
                shadow_maps_enabled: true,
            }
            Transform::from_translation(Vec3::new(5.0, 5.0, 5.0))
        )
    ]
}


fn add_spheres(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    
    let sphere = meshes.add(Sphere::new(1.0));

    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                commands.spawn((
                        Mesh3d(sphere.clone()),
                        MeshMaterial3d(materials.add(Color::WHITE)),
                        Transform::from_xyz(x as f32 * 2.0, y as f32 * 2.0, z as f32 * 2.0),
                        Movable
                ));
            }
        }
    }
}



fn update_scene(mut movable: Query<&mut Transform, With<Movable>>, time: Res<Time>) {
    for mut objects in movable.iter_mut() {
        objects.translation.y += f32::cos(time.elapsed_secs() * 5.0);
        objects.translation.x += f32::sin(time.elapsed_secs() * 5.0);
    }
}
