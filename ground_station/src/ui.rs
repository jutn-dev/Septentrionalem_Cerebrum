use std::ops::RangeInclusive;

use bevy::math::f64;
use bevy::prelude::*;
use bevy::render::render_resource::binding_types::uniform_buffer;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::emath::RectTransform;
use bevy_inspector_egui::egui::pos2;
use bevy_inspector_egui::egui::Button;
use bevy_inspector_egui::egui::Color32;
use bevy_inspector_egui::egui::Frame;
use bevy_inspector_egui::egui::Pos2;
use bevy_inspector_egui::egui::Sense;
use bevy_inspector_egui::egui::Shape;
use bevy_inspector_egui::egui::Slider;
use bevy_inspector_egui::egui::Stroke;
use bevy_inspector_egui::egui::Vec2b;
use chrono::DateTime;
use chrono::Timelike;
use egui_plot::log_grid_spacer;
use egui_plot::uniform_grid_spacer;
use egui_plot::AxisHints;
use egui_plot::CoordinatesFormatter;
use egui_plot::GridInput;
use egui_plot::GridMark;
use egui_plot::Legend;
use egui_plot::Line;
use egui_plot::Plot;
use egui_plot::PlotBounds;
use egui_plot::PlotPoint;
use egui_plot::PlotPoints;
use egui_plot::Points;
use egui_plot::VLine;

use crate::data::Data;

pub struct CanSatUIPlugin;

impl Plugin for CanSatUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, (data_ui, time_line_ui, graph_ui));
    }
}

fn data_ui(mut context: EguiContexts, data: Res<Data>) {}

fn time_line_ui(mut context: EguiContexts, mut data: ResMut<Data>) {
    egui::TopBottomPanel::bottom("Timeline")
        .resizable(true)
        .show(context.ctx_mut().unwrap(), |ui| {
            let mut data_button_clicked = false;
            ui.horizontal(|ui| {
                ui.heading("Timeline");
                data_button_clicked = ui.add(Button::new("Data")).clicked();
                ui.label(time_formatter(data.current_time));
            });

            let points: Vec<[f64; 2]> = data
                .data_points
                .iter()
                .map(|d| [d.time as f64, -1.])
                .collect();
            let x_axis = vec![AxisHints::new_x().formatter(x_axis_time_formatter)];
            let plot = Plot::new("TimeLine")
                .custom_x_axes(x_axis)
                .show_y(false)
                .show_x(false)
                .show_grid(Vec2b::new(true, false))
                .show_axes(Vec2b::new(true, false))
                .allow_scroll(Vec2b::new(true, false))
                .allow_drag(Vec2b::new(false, false))
                .allow_zoom(Vec2b::new(true, false))
                //.x_grid_spacer(log_grid_spacer(10))
                .show(ui, |plot_ui| {
                    plot_ui.points(Points::new("time points", points.clone()).radius(5.));
                    plot_ui.vline(VLine::new("", data.current_time as f64).color(Color32::RED));
                    if data_button_clicked {
                        plot_ui.set_plot_bounds_x(
                            points.first().unwrap()[0]..=points.last().unwrap()[0],
                        );
                    }
                });

            if plot.response.dragged_by(egui::PointerButton::Primary)
                || plot.response.clicked_by(egui::PointerButton::Primary)
            {
                let point = plot
                    .transform
                    .value_from_position(plot.response.hover_pos().unwrap());
                data.current_time = point.x.floor() as u64;
            }

            /*
            Frame::canvas(ui.style()).show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::all());
                let to = RectTransform::from_to(
                    egui::Rect::from_min_size(Pos2::ZERO, response.rect.size()),
                    response.rect,
                );

                let s = Shape::LineSegment {
                    points: [to * pos2(1., 1.), to * pos2(2000., 2000.)],
                    stroke: Stroke {
                        width: 20.,
                        color: Color32::WHITE,
                    },
                };

                painter.add(s.clone());
                response
            });

            //ui.add(response);
            */
        });
}

fn graph_ui(mut context: EguiContexts, mut data: ResMut<Data>) {
    let current_data = data.get_closest_point_in_time(data.current_time);
    egui::Window::new("Graph").show(context.ctx_mut().unwrap(), |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("lon: {}", current_data.lon));
            ui.label(format!("lat: {}", current_data.lat));
            ui.label(format!("({})", current_data.position));
        });

        let points: Vec<[f64; 2]> = data
            .data_points
            .iter()
            .map(|d| [d.time as f64, d.air_pressure])
            .collect();
        let spaces = |input: GridInput| {
            let (min, max) = input.bounds;
            let min = min.floor() as i64;
            let max = max.ceil() as i64;
            let range = (min..max).filter(|x| {
                let bool = x % input.base_step_size.ceil() as i64 == 0;
                //println!("bool: {} {} {}", bool, x, input.base_step_size.ceil() as i64);

                bool
            });
            //            marks.reserve(range.clone().count());
            //

            //let mut marks = Vec::with_capacity(range.clone().count());
            let mut marks = vec![];
            for i in range {
                let step_size = if i % 10 == 0 {
                    10.
                } else {
                    continue;
                };
                marks.push(GridMark {
                    value: i as f64,
                    step_size: 10.,
                });
            }
            marks
        };
        let x_axis = vec![AxisHints::new_x().label("Time").formatter(x_axis_time_formatter)];
        Plot::new("Graph")
            .legend(Legend::default())
            .custom_x_axes(x_axis)
            .x_grid_spacer(log_grid_spacer(10))
            .coordinates_formatter(
                egui_plot::Corner::LeftTop,
                CoordinatesFormatter::new(coordinates_formatter),
            )
            .show(ui, |plot_ui| {
                plot_ui.points(Points::new("air pressure", points.clone()));
                plot_ui.line(Line::new("air pressure", points.clone()));
                plot_ui.vline(VLine::new("", data.current_time as f64).color(Color32::RED));
            });
    });
}

fn x_axis_time_formatter(mark: GridMark, _range: &RangeInclusive<f64>) -> String {
    time_formatter(mark.value as u64)
}

fn coordinates_formatter(plot_point: &PlotPoint, _plot_bounds: &PlotBounds) -> String {
    time_formatter(plot_point.x as u64)
}


fn time_formatter(time: u64) -> String {
    let date_time = DateTime::from_timestamp_millis(time as i64).unwrap();
    if date_time.second() != 0 {
        date_time.format("%H:%M.%S").to_string()
    } else {
        date_time.format("%H:%M").to_string()
    }
}
