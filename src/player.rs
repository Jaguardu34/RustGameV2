use avian3d::{collision::collider::Collider, dynamics::rigid_body::{LinearVelocity, RigidBody, mass_properties::components::Mass}};
use bevy::{input::{ButtonState, mouse::{MouseButtonInput, MouseMotion}}, math::VectorSpace, post_process::bloom::Bloom, prelude::*, window::{CursorOptions, PrimaryWindow}};

pub struct PlayerPlugin;


#[derive(Resource, Default)]
struct MouseGrabbed(bool);

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MouseGrabbed(false))
            .add_systems(Startup, create_player)
            .add_systems(Update , (handle_mouse_motion, update, handle_move_input, handle_mouse_grab));
    }
}

const MOUSE_SENSITIVITY_YAW: f32 = 0.005;
const MOUSE_SENSITIVITY_PITCH: f32 = 0.005;

#[derive(Component)]
pub struct Player {
    life: f32,
    jump_strenght: f32,
    speed: f32,
}

impl Player {
    fn new(life: f32, jump_strenght: f32, speed: f32) -> Self {
        Self { life, jump_strenght, speed }
    }
}

#[derive(Component, Default)]
pub struct CameraRotation{
    yaw: f32,
    pitch: f32,
}

#[derive(Component)]
pub struct PlayerCam;

fn create_player(mut commands: Commands) {
    commands.spawn((
            Player::new(10.0, 10.0, 10.0),
            RigidBody::Dynamic,
            Collider::capsule(0.5, 1.5),
            Transform::from_xyz(0.0, 10.0, 4.0),
            CameraRotation::default(),
            Mass(5.0),
            children![
                (
                    Camera3d::default(),
                    Transform::from_xyz(0.0, 1.5, 0.0),
                    PlayerCam,
                    Bloom::NATURAL
                )
            ]
    ));
}


fn handle_mouse_motion(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut player_query: Query<(&mut Transform, &mut CameraRotation), (With<Player>, Without<PlayerCam>)>,
    mut player_cam_query: Query<&mut Transform, (With<PlayerCam>, Without<Player>)>,
    mouse_grabbed: Res<MouseGrabbed>
) {

    if !mouse_grabbed.0 {
        return;
    };
    let Ok(mut player_cam) = player_cam_query.single_mut() else {
        return;
    };
    let Ok((mut player_transform, mut rot)) = player_query.single_mut() else {
        return;
    };

    for motion in mouse_motion.read() {
        rot.yaw -= motion.delta.x * MOUSE_SENSITIVITY_YAW;
        rot.pitch = (rot.pitch - motion.delta.y * MOUSE_SENSITIVITY_PITCH).clamp(-1.54, 1.54); // +/-88°
        //println!("Rot: yaw: {}, pitch: {}", rot.yaw, rot.pitch)
    }

    player_transform.rotation = Quat::from_rotation_y(rot.yaw);
    player_cam.rotation = Quat::from_rotation_x(rot.pitch);
}


fn update(
    global_transform_query: Query<&GlobalTransform, With<Player>>
) {
    let Ok(global_transform) = global_transform_query.single() else {return;};

}


const MAX_SPEED: f32 = 10.0;

fn handle_move_input(keys: Res<ButtonInput<KeyCode>>, mut player_query: Query<(&mut LinearVelocity, &GlobalTransform), With<Player>>) {
    let Ok((mut player_velocity, player_transform)) = player_query.single_mut() else {return;};

    let mut vec = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        vec += *player_transform.forward();
      }
    if keys.pressed(KeyCode::KeyS) {
        vec -= *player_transform.forward();
    }
    if keys.pressed(KeyCode::KeyA) {
        vec -= *player_transform.right();
    }
    if keys.pressed(KeyCode::KeyD) {
        vec += *player_transform.right();
    }
    

        
        vec.y = 0.0;

    vec = vec.normalize_or_zero();
    
    if vec != Vec3::ZERO {
        player_velocity.x = vec.x * 10.0;
        player_velocity.z = vec.z * 10.0;
    } else {
        player_velocity.x = player_velocity.x * 0.98;
        player_velocity.z = player_velocity.z * 0.98;
    }
    
}


fn handle_mouse_grab(
    mut mouse_grabbed: ResMut<MouseGrabbed>, 
    mut mouse_button: MessageReader<MouseButtonInput>, 
    keys: Res<ButtonInput<KeyCode>>, 
    mut cursor_option: Single<&mut CursorOptions>, 
    window: Single<&Window, With<PrimaryWindow>>)
{
    if keys.just_pressed(KeyCode::Escape) && mouse_grabbed.0 && window.cursor_position().is_some(){
        mouse_grabbed.0 = false;
    }
    for button in mouse_button.read() {
        if button.state == ButtonState::Pressed && button.button == MouseButton::Left && !mouse_grabbed.0 {
            mouse_grabbed.0 = true;
        }
    }

    if mouse_grabbed.0 {
        cursor_option.visible = false;
        cursor_option.grab_mode = bevy::window::CursorGrabMode::Confined
    } else {
        cursor_option.visible = true;
        cursor_option.grab_mode = bevy::window::CursorGrabMode::None;
    }
}
