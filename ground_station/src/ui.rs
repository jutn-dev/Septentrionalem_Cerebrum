use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::ProgressBar;
use bevy_inspector_egui::egui::Slider;

use crate::data::Data;

pub struct CanSatUIPlugin;
    
impl Plugin for CanSatUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, (data_ui, time_line_ui));
    }
}



fn data_ui(
    mut context: EguiContexts,
    data: Res<Data>,
) {
    egui::Window::new("Data").show(context.ctx_mut().unwrap(), |ui| {
        ui.horizontal(|ui|{
            ui.label(format!("lon: {}", data.data_points[data.current_data].lon));
            ui.label(format!("lat: {}", data.data_points[data.current_data].lat));
            ui.label(format!("({})", data.data_points[data.current_data].position));
        })
    });

}

fn time_line_ui(
    mut context: EguiContexts,
    mut data: ResMut<Data>,
) {
    egui::Window::new("Timeline").show(context.ctx_mut().unwrap(), |ui| {
        let data_point_amount = data.data_points.len() - 1;
        ui.add(Slider::new(&mut data.current_data, 0..=data_point_amount))
    });

}
