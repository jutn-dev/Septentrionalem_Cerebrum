use bevy::{input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll}, math::VectorSpace, prelude::*};

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
        app.add_systems(Update, camera_movement); 
        app.add_systems(Update, set_camera_target); 
    }
}


fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(),
        OrbitCamera {
            move_button: MouseButton::Middle,
            x_speed: 0.005,
            y_speed: 0.005,
            zoom_size: 5.,
            zoom: 50.,
            target_position: Vec3::ZERO,
        }
    ));
}


fn camera_movement(mut camera: Single<(&mut Transform, &mut OrbitCamera), With<Camera3d>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>
){
    
    let (mut camera_rotation_x,mut camera_rotation_y, camera_rotation_z) = camera.0.rotation.to_euler(EulerRot::YXZ);

    if mouse_buttons.pressed(camera.1.move_button) {
        let x = -mouse_motion.delta.x * camera.1.x_speed;
        let y = -mouse_motion.delta.y * camera.1.y_speed;
        camera_rotation_x += x; 
        camera_rotation_y += y; 
    }
    camera.0.rotation = Quat::from_euler(EulerRot::YXZ, camera_rotation_x, camera_rotation_y, camera_rotation_z);
    camera.1.zoom += -mouse_scroll.delta.y * camera.1.zoom_size;
    camera.0.translation = camera.1.target_position - camera.0.forward() * camera.1.zoom;


}

fn set_camera_target(
mut camera: Single<&mut OrbitCamera, With<Camera3d>>,
    data: Res<Data>,
) {
    if let Some(position) = data.get_point_relative_position(data.current_data) {
        camera.target_position = position;
    }
}
