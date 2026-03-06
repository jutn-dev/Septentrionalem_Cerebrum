use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin},
    quick::WorldInspectorPlugin,
};

use crate::{
    camera::OrbitCameraPlugin, data::Data, environment::CanSatEnvironmentPlugin, serial_data::SerialPortDataPlugin, ui::CanSatUIPlugin
};

mod camera;
mod data;
mod serial_data;
mod environment;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins((CanSatEnvironmentPlugin, OrbitCameraPlugin))
        .add_plugins(SerialPortDataPlugin)
        .add_plugins(CanSatUIPlugin)
        .insert_resource(EguiGlobalSettings {
            enable_absorb_bevy_input_system: true,
            ..default()
        })
        .init_resource::<Data>()
        .run();
}
