use avian3d::PhysicsPlugins;
use bevy::{prelude::*, window::WindowResolution};

use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    prelude::*,
    text::FontSmoothing,
};

pub mod player;
pub mod scene;
pub mod my_camera;
use my_camera::MyCamPlugin;
use scene::MyScenePlugin;

use crate::player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "RustGameV2".to_string(),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }))
        // avian3d for physics
        .add_plugins(PhysicsPlugins::default())
        .add_plugins((PlayerPlugin, MyScenePlugin))
        // to show FPS
        .add_plugins(FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: FontSize::Px(42.0),
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::srgb(1.0, 0.0, 0.0),
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: true,
                        // The minimum acceptable fps
                        min_fps: 30.0,
                        // The target fps
                        target_fps: 180.0,
                    },
                },
            })
        .run();   
}
