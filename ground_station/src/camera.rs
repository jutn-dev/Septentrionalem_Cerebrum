use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};
use bevy_inspector_egui::bevy_egui::input::egui_wants_any_input;

use crate::data::Data;

#[derive(Component, Debug)]
pub struct OrbitCamera {
    move_button: MouseButton,
    secondary_move_button: MouseButton,
    secondary_button: KeyCode,
    x_speed: f32,
    y_speed: f32,
    zoom_size: f32,
    zoom: f32,
    target_position: Vec3,
}

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Update, camera_movement.run_if(not(egui_wants_any_input)));
        app.add_systems(Update, set_camera_target);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        OrbitCamera {
            move_button: MouseButton::Middle,
            secondary_move_button: MouseButton::Left,
            secondary_button: KeyCode::ControlLeft,
            x_speed: 0.005,
            y_speed: 0.005,
            zoom_size: 10.,
            zoom: 50.,
            target_position: Vec3::ZERO,
        },
    ));
}

fn camera_movement(
    mut camera: Single<(&mut Transform, &mut OrbitCamera), With<Camera3d>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let (mut camera_rotation_x, mut camera_rotation_y, camera_rotation_z) =
        camera.0.rotation.to_euler(EulerRot::YXZ);

    if mouse_buttons.pressed(camera.1.move_button)
        || mouse_buttons.pressed(camera.1.secondary_move_button)
            && keyboard.pressed(camera.1.secondary_button)
    {
        let x = -mouse_motion.delta.x * camera.1.x_speed;
        let y = -mouse_motion.delta.y * camera.1.y_speed;
        camera_rotation_x += x;
        camera_rotation_y += y;
    }
    camera.0.rotation = Quat::from_euler(
        EulerRot::YXZ,
        camera_rotation_x,
        camera_rotation_y,
        camera_rotation_z,
    );
    camera.1.zoom += -mouse_scroll.delta.y * camera.1.zoom_size;
    camera.0.translation = camera.1.target_position - camera.0.forward() * camera.1.zoom;
}

fn set_camera_target(mut camera: Single<&mut OrbitCamera, With<Camera3d>>, data: Res<Data>) {
    
    let Some((_time, position)) = data.data_points.position.iter().filter(|f|*f.0 < data.current_time).min_by_key(|p| p.0.abs_diff(data.current_time)) else {
        return;
    };
    let position = data.get_point_relative_position(position);
    camera.target_position = position;
}
