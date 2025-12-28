use std::f32::consts::PI;

use bevy::{gizmos::gizmos, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{camera::OrbitCameraPlugin, data::Data, ui::CanSatUIPlugin};

mod camera;
mod data;
mod ui;
mod environment;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct Gizmos3D;

fn main() {
    
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(OrbitCameraPlugin)
        .add_plugins(CanSatUIPlugin)
        .init_gizmo_group::<Gizmos3D>()
        .add_systems(Update, setup)
        .insert_resource(Data::data_form_json_file("data.json").unwrap())
        .run();
}



fn setup(
    mut gizmos: Gizmos<Gizmos3D>
) {
        gizmos.grid(
        Quat::from_rotation_x(PI / 2.),
        UVec2::splat(100),
        Vec2::new(2., 2.),
        // Light gray
        Srgba::rgba_u8(92, 92, 92, 30));
}
