use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, Write},
};

use bevy::{math::VectorSpace, prelude::*};
use communication::data::{CO2SensorData, DataTypes, GPSData, MiscData, PressureSensorData};
use egui_plot::PlotPoints;
use proj::Proj;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct Position {
    pub gps_data: GPSData,
    // 3d position to be used in the engine
    pub position: Vec3,
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
///Data from CanSat
pub struct Data {
    //data points are expected to be in chronological order.
    //      TODO REMOVE THIS COMMENT AS THE DATA TYPE SHOULD ENFORCE THIS SATATEMENT
    //If not you might get some unexpected results.
    pub data_points: DataPoints,
    pub current_time: u64,
    center_position: Vec3,
}

///one point of data
///
///why is there two `DataPoint` sturcts?
///I want to be able to use rust's type safty to ensure that variables
///which get computed on the groundstation such as `position` get initalized properly.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
pub struct DataPoints {
    pub position: BTreeMap<u64, Position>,
    pub pressure_data: BTreeMap<u64, PressureSensorData>,
    pub co2_data: BTreeMap<u64, CO2SensorData>,
    pub misc_data: BTreeMap<u64, MiscData>,
}

#[allow(unused)]
impl Data {
    pub fn data_form_json_file(path: String) -> Result<Self, std::io::Error> {
        match File::open(path) {
            Ok(json) => {
                let reader = BufReader::new(json);
                let mut data: DataPoints = serde_json::from_reader(reader)?;
                // if coming from previous git commit
                // positions do not get created automatically

                return Ok(Data {
                    center_position: Vec3::ZERO, //Data::get_center_position(&data),
                    current_time: 0,
                    data_points: data,
                });
            }
            Err(error) => {
                error!("Loading json file failed: {error}");
                Err(error)
            }
        }
    }
    pub fn write_json_to_file(&self, path: String) {
        let json_string = serde_json::to_string(&self.data_points).unwrap();
        let mut file = File::create(path).unwrap();
        write!(file, "{}", json_string);
    }

    /*
    // vec should be sorted when using this function
    // if no coordinates found returns Vec3::Zero
    fn get_center_position(datapoints: &Vec<DataPoints>) -> Vec3 {
        for datapoint in datapoints {
            if let Some(position) = &datapoint.position {
                return position.position;
            }
        }
        Vec3::ZERO
    }
    */
    /// returns relative position of data point. If no data point exists or specific `DataPoint`
    /// doesn't exists returns `None`
    pub fn get_point_relative_position(&self, position: &Position) -> Vec3 {
        self.center_position - position.position
    }

    pub fn extend(&mut self, message: &communication::data::Message) {
        match message.data.clone() {
            DataTypes::PressureSensor(pressure_data) => {
                self.data_points
                    .pressure_data
                    .insert(message.time, pressure_data);
            }
            DataTypes::CO2Sensor(co2_data) => {
                self.data_points.co2_data.insert(message.time, co2_data);
            }
            DataTypes::GPS(gps_data) => {
                self.data_points.position.insert(
                    message.time,
                    Position {
                        position: self.get_position(&gps_data, message.time),
                        gps_data,
                    },
                );
            }
            DataTypes::Misc(m) => {
                self.data_points.misc_data.insert(message.time, m);
            },
            _ => (),
        }
    }
    //get position from Coordinate.
    pub fn get_position(&self, gps_data: &GPSData, time: u64) -> Vec3 {
        let from = "EPSG:4326";
        let to = "EPSG:3857";
        let wsg84_to_epsg3857 = Proj::new_known_crs(from, to, None).unwrap();

        //warn!("THIS FUNCTION DOES NOT OPERATE HOW IT IS SUPPOST TO");
        let coordinate = wsg84_to_epsg3857
            .convert((gps_data.lon, gps_data.lat))
            .unwrap();
        let Some((_time, pressure_data)) = self
            .data_points
            .pressure_data
            .iter()
            .filter(|f| *f.0 < time)
            .min_by_key(|p| p.0.abs_diff(time))
        else {
            return Vec3::new(coordinate.0, 0., coordinate.1);
        };
        Vec3::new(
            coordinate.0,
            Self::calculate_height_from_pressure(pressure_data.pressure),
            coordinate.1,
        )
    }
    fn calculate_height_from_pressure(pressure: f32) -> f32 {
        44330.0 * (1.-f32::powf(pressure/1013.25, 0.190295))
    }
}

/*
impl Ord for DataPoints {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}
impl PartialOrd for DataPoints {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for DataPoints {
    /// if self.time is same
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl Eq for DataPoints {}
*/
