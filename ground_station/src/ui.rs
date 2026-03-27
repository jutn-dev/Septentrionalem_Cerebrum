use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::RangeInclusive;

use bevy::math::f64;
use bevy::prelude::*;
use bevy::reflect::DynamicTypePath;
use bevy::reflect::DynamicTyped;
use bevy::reflect::Map;
use bevy::reflect::ReflectRef;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::bevy_egui::EguiPrimaryContextPass;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::Grid;
use bevy_inspector_egui::egui::epaint::tessellator::path;
use bevy_inspector_egui::egui::Button;
use bevy_inspector_egui::egui::Color32;
use bevy_inspector_egui::egui::ComboBox;
use bevy_inspector_egui::egui::DragValue;
use bevy_inspector_egui::egui::TextEdit;
use bevy_inspector_egui::egui::Ui;
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
            .init_resource::<GraphResource>()
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
struct GraphResource {
    buttons_pressed: HashMap<String, bool>,
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
                if ui.button("Write").clicked() {
                    let result = data.write_json_to_file(load_data.path.clone());
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

                    //plot_ui.points(Points::new("time points", points.clone()).radius(5.));
                    for field in data.data_points.iter_fields() {
                        let s = field.reflect_ref();
                        match s {
                            ReflectRef::Map(map) => {
                                let vec: Vec<(u64, &dyn PartialReflect)> = map
                                    .iter()
                                    .map(|f| (*f.0.try_downcast_ref::<u64>().unwrap(), f.1))
                                    .collect();
                                timeline_from_data(vec, None, plot_ui);
                            }
                            ReflectRef::Struct(_) => (),
                            _ => (),
                        }}
                    plot_ui.vline(VLine::new("", data.current_time as f64).color(Color32::RED));
                });

            if plot.response.dragged_by(egui::PointerButton::Primary)
                || plot.response.clicked_by(egui::PointerButton::Primary)
            {
                let point = plot
                    .transform
                    .value_from_position(plot.response.hover_pos().unwrap());
                data.current_time = point.x.round() as u64;
                println!("time; {:?}", point);
            }
        });}

fn graph_ui(
    mut context: EguiContexts,
    mut data: ResMut<Data>,
    mut graph_res: ResMut<GraphResource>,
) {
    egui::SidePanel::left("Graph")
        .resizable(true)
        .min_width(0.0)
        .show(context.ctx_mut().unwrap(), |ui| {
            ui.take_available_width();
            for field in data.data_points.iter_fields() {
                let s = field.reflect_ref();
                match s {
                    ReflectRef::Map(map) => {
                        let vec: Vec<(u64, &dyn PartialReflect)> = map
                            .iter()
                            .map(|f| (*f.0.try_downcast_ref::<u64>().unwrap(), f.1))
                            .collect();
                        graph_buttons_data(vec, None, None, &mut graph_res.buttons_pressed, data.current_time, ui);
                    }
                    ReflectRef::Struct(_) => (),
                    _ => (),
                }
            }
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
                    for field in data.data_points.iter_fields() {
                        let s = field.reflect_ref();
                        match s {
                            ReflectRef::Map(map) => {
                                let vec: Vec<(u64, &dyn PartialReflect)> = map
                                    .iter()
                                    .map(|f| (*f.0.try_downcast_ref::<u64>().unwrap(), f.1))
                                    .collect();
                                graph_from_data(vec, None, plot_ui, &mut graph_res.buttons_pressed);
                            }
                            ReflectRef::Struct(_) => (),
                            _ => (),
                        }
                    }
                    //plot_ui.vline(VLine::new("", data.current_time as f64).color(Color32::RED));
                });
        });
}

fn graph_buttons_data(
    vec: Vec<(u64, &dyn PartialReflect)>,
    path: Option<String>,
    name: Option<String>,
    buttons: &mut HashMap<String, bool>,
    current_time: u64,
    ui: &mut Ui,
) {
    //println!("ty: {:?}", st.get_represented_struct_info().unwrap().ty());
    //let first_struct_item = st.field_at(0).unwrap().reflect_ref().as_struct().unwrap();
    let Some(vec_first) = vec.first() else {
        return;
    };
    ui.horizontal(|ui| {

    if vec_first.1.reflect_ref().as_opaque().is_ok() {
        let Some(type_name) = name else {
            error!("no name for data");
            return;
        };
        let Some(type_path) = path else {
            error!("no path name for data");
            return;
        };
        let mut button_name = format!("{}",type_name);
        if let Some(data) = vec
            .iter()
            .filter(|f| f.0 < current_time)
            .min_by_key(|p| p.0.abs_diff(current_time))
            {
                button_name = format!("{}: {:?}",type_name, data.1);         
        }
    let button_state = buttons.get(&format!("{}::{}", type_path, type_name));
        let selected = Some(&true) == button_state;
        if ui.add(Button::selectable(selected, button_name)).clicked() {
            if selected {
                buttons.insert(format!("{}::{}", type_path, type_name), false);
            } else {
                buttons.insert(format!("{}::{}", type_path, type_name), true);
            }
        }
    } else {
        for (i, _) in vec_first
            .1
            .reflect_ref()
            .as_struct()
            .unwrap()
            .iter_fields()
            .enumerate()
        {
            let type_path = format!(
                "{:?}",
                vec_first
                    .1
                    .reflect_ref()
                    .as_struct()
                    .unwrap()
                    .get_represented_struct_info()
                    .unwrap()
                    .ty(),
            );
            let type_name = vec_first
                .1
                .reflect_ref()
                .as_struct()
                .unwrap()
                .name_at(i)
                .unwrap()
                .to_string();

            let v: Vec<(u64, &dyn PartialReflect)> = vec
                .iter()
                .map(|f| {
                    (
                        f.0,
                        f.1.reflect_ref().as_struct().unwrap().field_at(i).unwrap(),
                    )
                })
                .collect();


            graph_buttons_data(v, Some(type_path), Some(type_name), buttons,current_time, ui);
            ui.end_row();
        }
    }
    });
}
fn graph_from_data(
    vec: Vec<(u64, &dyn PartialReflect)>,
    name: Option<String>,
    plot_ui: &mut PlotUi,
    buttons: &mut HashMap<String, bool>,
) {
    //println!("ty: {:?}", st.get_represented_struct_info().unwrap().ty());
    //let first_struct_item = st.field_at(0).unwrap().reflect_ref().as_struct().unwrap();
    let Some(vec_first) = vec.first() else {
        return;
    };
    if vec_first.1.reflect_ref().as_opaque().is_ok() {
        let Some(name) = name else {
            error!("no name for data");
            return;
        };
        if Some(&true) != buttons.get(&name) {
            return;
        }
        let data: Vec<[f64; 2]> = vec
            .iter()
            .map(|f| {
                if let Ok(point) = format!("{:?}", f.1).parse::<f64>() {
                    return [f.0 as f64, point];
                }
                [f.0 as f64, 0.]
            })
            .collect();
        //println!("v: {:?}", data);
        plot_ui.line(Line::new(name, data));
    } else {
        for (i, _) in vec_first
            .1
            .reflect_ref()
            .as_struct()
            .unwrap()
            .iter_fields()
            .enumerate()
        {
            let type_name = format!(
                "{:?}::{}",
                vec_first
                    .1
                    .reflect_ref()
                    .as_struct()
                    .unwrap()
                    .get_represented_struct_info()
                    .unwrap()
                    .ty(),
                vec_first
                    .1
                    .reflect_ref()
                    .as_struct()
                    .unwrap()
                    .name_at(i)
                    .unwrap()
            );
            let v: Vec<(u64, &dyn PartialReflect)> = vec
                .iter()
                .map(|f| {
                    (
                        f.0,
                        f.1.reflect_ref().as_struct().unwrap().field_at(i).unwrap(),
                    )
                })
                .collect();

            graph_from_data(v, Some(type_name), plot_ui, buttons);
        }
    }
}

fn timeline_from_data(
    vec: Vec<(u64, &dyn PartialReflect)>,
    name: Option<String>,
    plot_ui: &mut PlotUi,
) {
    //println!("ty: {:?}", st.get_represented_struct_info().unwrap().ty());
    //let first_struct_item = st.field_at(0).unwrap().reflect_ref().as_struct().unwrap();
    let Some(vec_first) = vec.first() else {
        return;
    };
    if vec_first.1.reflect_ref().as_opaque().is_ok() {
        let data: Vec<[f64; 2]> = vec
            .iter()
            .map(|f| {
                if let Ok(point) = format!("{:?}", f.1).parse::<f64>() {
                    return [f.0 as f64, 0.];
                }
                [f.0 as f64, 0.]
            })
            .collect();
        //println!("v: {:?}", data);
        let Some(name) = name else {
            error!("no name for data");
            return;
        };
        plot_ui.points(Points::new(name, data));
    } else {
        for (i, _) in vec_first
            .1
            .reflect_ref()
            .as_struct()
            .unwrap()
            .iter_fields()
            .enumerate()
        {
            let type_name = format!(
                "{:?}::{}",
                vec_first
                    .1
                    .reflect_ref()
                    .as_struct()
                    .unwrap()
                    .get_represented_struct_info()
                    .unwrap()
                    .ty(),
                vec_first
                    .1
                    .reflect_ref()
                    .as_struct()
                    .unwrap()
                    .name_at(i)
                    .unwrap()
            );
            let v: Vec<(u64, &dyn PartialReflect)> = vec
                .iter()
                .map(|f| {
                    (
                        f.0,
                        f.1.reflect_ref().as_struct().unwrap().field_at(i).unwrap(),
                    )
                })
                .collect();

            timeline_from_data(v, Some(type_name), plot_ui);
        }
    }
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
