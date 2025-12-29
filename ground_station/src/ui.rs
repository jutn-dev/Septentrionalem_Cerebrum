use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::Color32;
use bevy_inspector_egui::egui::ProgressBar;
use bevy_inspector_egui::egui::Slider;
use egui_plot::HLine;
use egui_plot::Legend;
use egui_plot::Line;
use egui_plot::Plot;
use egui_plot::Points;
use egui_plot::VLine;

use crate::data::Data;

pub struct CanSatUIPlugin;
    
impl Plugin for CanSatUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, (data_ui, time_line_ui, graph_ui));
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



fn graph_ui(
    mut context: EguiContexts,
    mut data: ResMut<Data>,
) {
    egui::Window::new("Graph").show(context.ctx_mut().unwrap(), |ui| {
        let points: Vec<[f64;2]> = data.data_points.iter().enumerate().map(|(i, d)| [i as f64, d.air_pressure]).collect();
        Plot::new("Graph").legend(Legend::default()).show(ui, |plot_ui| {
            plot_ui.points(Points::new("air pressure", points.clone()));
            plot_ui.line(Line::new("air pressure", points.clone()));
            plot_ui.vline(VLine::new("", data.current_data as f64).color(Color32::RED));
        });
        
    });

}
