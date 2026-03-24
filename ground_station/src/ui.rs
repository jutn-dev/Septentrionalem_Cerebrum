use std::any::Any;
use std::fmt::Debug;
use std::ops::RangeInclusive;

use bevy::math::f64;
use bevy::prelude::*;
use bevy::reflect::DynamicTyped;
use bevy::reflect::Map;
use bevy::reflect::ReflectRef;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::Button;
use bevy_inspector_egui::egui::Color32;
use bevy_inspector_egui::egui::ComboBox;
use bevy_inspector_egui::egui::DragValue;
use bevy_inspector_egui::egui::TextEdit;
use bevy_inspector_egui::egui::Vec2b;
use chrono::DateTime;
use chrono::Timelike;
use egui_plot::PlotUi;
use egui_plot::{
    log_grid_spacer, AxisHints, CoordinatesFormatter, GridInput, GridMark, Legend, Line, Plot,
    PlotBounds, PlotPoint, Points, VLine,
};

use crate::data::Data;
use crate::data::DataPoints;
use crate::data::Position;
use crate::serial_data::InitSerialPortMessage;

pub struct CanSatUIPlugin;

impl Plugin for CanSatUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadDataFile>()
            .init_resource::<LoadDataSerial>()
            .add_systems(EguiPrimaryContextPass, (data_ui, graph_ui, time_line_ui));
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
enum LoadDataMode {
    #[default]
    File,
    Serial,
    Udp,
}

#[derive(Debug, Clone, Resource, Default)]
struct LoadDataFile {
    path: String,
    mode: LoadDataMode,
}

#[derive(Debug, Clone, Resource)]
struct LoadDataSerial {
    path: String,
    baudrate: u32,
}
impl Default for LoadDataSerial {
    fn default() -> Self {
        Self {
            path: String::default(),
            baudrate: 9600,
        }
    }
}

fn data_ui(
    mut context: EguiContexts,
    mut data: ResMut<Data>,
    mut load_data: ResMut<LoadDataFile>,
    mut load_serial: ResMut<LoadDataSerial>,
    mut init_serial_message: MessageWriter<InitSerialPortMessage>,
) {
    egui::Window::new("Load Data").show(context.ctx_mut().unwrap(), |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut load_data.mode, LoadDataMode::File, "File");
            ui.selectable_value(&mut load_data.mode, LoadDataMode::Serial, "Serial");
            ui.selectable_value(&mut load_data.mode, LoadDataMode::Udp, "UDP");
        });
        match load_data.mode {
            LoadDataMode::File => {
                ui.add(TextEdit::singleline(&mut load_data.path).hint_text("file path"));
                if ui.button("Load").clicked() {
                    let result = Data::data_form_json_file(load_data.path.clone());
                    match result {
                        Ok(result_data) => *data = result_data,
                        Err(e) => error!("{e}"),
                    }
                }
            }
            LoadDataMode::Serial => {
                ComboBox::from_label("serial port")
                    .selected_text(&load_serial.path)
                    .show_ui(ui, |ui| {
                        let ports = serialport::available_ports().unwrap();
                        for port in ports {
                            ui.selectable_value(
                                &mut load_serial.path,
                                port.port_name.clone(),
                                port.port_name,
                            );
                        }
                    });
                ui.add(DragValue::new(&mut load_serial.baudrate));
                if ui.button("Open").clicked() {
                    init_serial_message.write(InitSerialPortMessage {
                        path: load_serial.path.clone(),
                        baudrate: load_serial.baudrate,
                    });
                }
            }
            LoadDataMode::Udp => (),
        }
    });
}

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
            /*
            let points: Vec<[f64; 2]> = data
                .data_points
                .iter()
                .map(|d| [d.time as f64, -1.])
                .collect();*/
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

                    /*
                    plot_ui.points(Points::new("time points", points.clone()).radius(5.));
                    plot_ui.vline(VLine::new("", data.current_time as f64).color(Color32::RED));
                    if data_button_clicked {
                        plot_ui.set_plot_bounds_x(
                            //TODO remove unwrap
                            //points.first().unwrap()[0]..=points.last().unwrap()[0],
                        );
                    }
                    */
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
    egui::SidePanel::left("Graph")
        .resizable(true)
        .min_width(0.0)
        .show(context.ctx_mut().unwrap(), |ui| {
            ui.take_available_width();
            if data.data_points.pressure_data.is_empty() {
            }
            if data.data_points.co2_data.is_empty() {
            }
            /*let Some(data_point) = data.get_closest_point_in_time(data.current_time) else {
                ui.label("no data points found");
                return;
            };
            for (i, _) in data_point.iter_fields().enumerate() {

                ui.add(Button::selectable(
                    true,
                    format!("{:?}, {:?}", data_point.name_at(i), data_point.field_at(i).unwrap())
                ));
            }
            */

            //TODO
            /*
            ui.horizontal(|ui| {
                ui.label(format!("lon: {}", current_data.lon));
                ui.label(format!("lat: {}", current_data.lat));
                ui.label(format!("({})", current_data.position));
            });
            */

            /*
            let points: Vec<[f64; 2]> = data
                .data_points
                .iter()
                .filter_map(|d| {
                    if let Some(pressure) = &d.pressure_data {
                        Some([d.time as f64, pressure.pressure as f64])
                    } else {
                        None
                    }
                })
                //.map(|a| [a.time as f64, a.air_pressure])
                .collect();*/
            let _spaces = |input: GridInput| {
                let (min, max) = input.bounds;
                let min = min.floor() as i64;
                let max = max.ceil() as i64;
                let range = (min..max).filter(|x| x % input.base_step_size.ceil() as i64 == 0);
                //            marks.reserve(range.clone().count());
                //

                //let mut marks = Vec::with_capacity(range.clone().count());
                let mut marks = vec![];
                for i in range {
                    let _step_size = if i % 10 == 0 {
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
            let x_axis = vec![AxisHints::new_x()
                .label("Time")
                .formatter(x_axis_time_formatter)];
            Plot::new("Graph")
                .legend(Legend::default())
                .custom_x_axes(x_axis)
                .x_grid_spacer(log_grid_spacer(10))
                .coordinates_formatter(
                    egui_plot::Corner::LeftTop,
                    CoordinatesFormatter::new(coordinates_formatter),
                )
                .show(ui, |plot_ui| {
                    for f in data.data_points.iter_fields() {
                        let s = f.reflect_ref();
                        //println!("s: {:?}", s.kind());
                        match s {
                            ReflectRef::Map(map) => {
                                let typee = map.get_represented_map_info().unwrap().value_ty();
                                //println!("m: {:?}", typee);

                                let vec: Vec<(u64, &dyn PartialReflect)> =
                                    map.iter().map(|f| (*f.0.try_downcast_ref::<u64>().unwrap(), f.1)).collect();
                                cool( vec, plot_ui);
                                //cool(map, first_struct_item, plot_ui);

                                /*
                                for (i, _) in first_struct_item.iter_fields().enumerate() {
                                    for (_, map_value) in map.iter() {
                                        let data_struct =
                                            map_value.reflect_ref().as_struct().unwrap();
                                        let data = data_struct.field_at(i).unwrap();
                                        println!("data: {:?}", data);
                                        if data.reflect_ref().as_opaque().is_ok() {
                                            println!("true :D");
                                        }
                                    }
                                }
                                */
                            }
                            ReflectRef::Struct(s) => (),
                            _ => (),
                        }
                    }
                    /*plot_ui.points(Points::new("air pressure", points.clone()));
                    plot_ui.line(Line::new("air pressure", points.clone()));
                    plot_ui.vline(VLine::new("", data.current_time as f64).color(Color32::RED));*/
                });
        });
}
fn cool(vec: Vec<(u64, &dyn PartialReflect)>, plot_ui: &mut PlotUi) {
    //println!("ty: {:?}", st.get_represented_struct_info().unwrap().ty());
    //let first_struct_item = st.field_at(0).unwrap().reflect_ref().as_struct().unwrap();
    let Some(vec_first) = vec.first() else {
        return;
    };
    if vec_first.1.reflect_ref().as_opaque().is_ok() {
        let data: Vec<[f64;2]> = vec
            .iter()
            .map(|f| {
                if let Some(point) = f.1.try_downcast_ref::<f32>() {
                    return [f.0 as f64, *point as f64];
                }
                return [f.0 as f64, 0.];
            })
            .collect();
        //println!("v: {:?}", data);
        plot_ui.line(Line::new(vec_first.1.reflect_type_path(), data));
    } else {
        

        for (i, _) in vec_first.1.reflect_ref().as_struct().unwrap().iter_fields().enumerate() {
            //println!("f: {:?}", vec_first.1.reflect_ref().as_struct().unwrap().get_represented_struct_info().unwrap().ty());
            let v: Vec<(u64, &dyn PartialReflect)> = vec
                .iter()
                .map(|f| (f.0, f.1.reflect_ref().as_struct().unwrap().field_at(i).unwrap()))
                .collect();
            cool( v, plot_ui);
        }
    }
    /*
    for (i, st_field) in st.iter().enumerate() {
        let mut points = vec![];
        for (j, (map_key, _map_value)) in map.iter().enumerate() {
            //DIG THE FALUES FROM THE MAP :D
            //collect inbetween values to vec
            //let st_field = st.field_at(j).unwrap();
            println!("st_field: {:?}", st_field);
            if st_field.reflect_ref().as_opaque().is_ok() {
                //                println!("stt_fp: {}",st_field.get_represented_type_info().unwrap().as_struct().unwrap());
                if let Some(point) = st_field.try_downcast_ref::<f32>() {
                    points.push([
                        *map_key.try_downcast_ref::<u64>().unwrap() as f64,
                        *point as f64,
                    ]);
                }
                //println!("true :D");
            } else {
                //println!("ty3: {:?}", st_field.reflect_ref().as_struct().unwrap().get_represented_struct_info().unwrap().ty());
                cool(map, st_field.reflect_ref().as_struct().unwrap(), plot_ui);
            }
        }
        println!("points: {:?}", points);
        plot_ui.line(Line::new(
            format!("{}_{}", st.reflect_type_path(), st.name_at(i).unwrap()),
            points,
        ));
    }
    */
}
/*
fn strrr(s: &dyn PartialReflect) -> Option<&dyn PartialReflect> {
    match s.reflect_ref() {
        ReflectRef::Struct(str) => {
            for (i, _) in str
                .iter_fields()
                .next()
                .unwrap()
                .1
                .reflect_ref()
                .as_struct()
                .unwrap()
                .iter_fields()
                .enumerate()
            {}
            if strrr(s).is_none() {
                return Some(s);
            }
        }
        ReflectRef::Opaque(_) => {
            return None;
        }
        _ => (),
    }
}
*/
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
