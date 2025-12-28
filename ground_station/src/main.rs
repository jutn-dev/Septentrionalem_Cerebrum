use bevy:: prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{camera::OrbitCameraPlugin, data::Data, environment::CanSatEnvironmentPlugin, ui::CanSatUIPlugin};

mod camera;
mod data;
mod ui;
mod environment;


fn main() {
    
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins((CanSatEnvironmentPlugin, OrbitCameraPlugin))
        .add_plugins(CanSatUIPlugin)
        .insert_resource(Data::data_form_json_file("data.json").unwrap())
        .run();
}



