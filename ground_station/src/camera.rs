use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};
use bevy_inspector_egui::bevy_egui::input::egui_wants_any_input;

use crate::data::Data;

#[derive(Component, Debug)]
pub struct OrbitCamera {
    move_button: MouseButton,
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
) {
    let (mut camera_rotation_x, mut camera_rotation_y, camera_rotation_z) =
        camera.0.rotation.to_euler(EulerRot::YXZ);

    if mouse_buttons.pressed(camera.1.move_button) {
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
    let current_data = data.get_closest_point_in_time(data.current_time);
    let Some(current_data) = current_data else {
        return;
    };
    let Some(position) = data.get_point_relative_position(current_data) else {
        return;
    };
    camera.target_position = position;
}
