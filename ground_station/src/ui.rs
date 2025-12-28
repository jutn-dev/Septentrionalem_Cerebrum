use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
use bevy_inspector_egui::egui;

use crate::data::Data;

pub struct CanSatUIPlugin;
    
impl Plugin for CanSatUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, data_ui);     
    }
}



fn data_ui(
    mut context: EguiContexts,
    data: Res<Data>,
) {
    egui::Window::new("Data").show(context.ctx_mut().unwrap(), |ui| {
        ui.horizontal(|ui|{
            ui.label(format!("lon: {}", data.data_points[0].lon));
            ui.label(format!("lat: {}", data.data_points[0].lat));
        })
    });

}
