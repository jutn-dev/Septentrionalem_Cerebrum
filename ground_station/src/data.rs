use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, Write},
};

use bevy::{math::VectorSpace, prelude::*};
use communication::data::{CO2SensorData, DataTypes, GPSData, PressureSensorData};
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
        let relative_position = self.center_position - position.position;
        relative_position
    }

    /*
    pub fn get_closest_point_in_time(&self, time: u64) -> Option<&DataPoints> {
        self.data_points
            .iter()
            .min_by_key(|d| d.time.abs_diff(time))
    }*/

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
            _=>(),
            /*
            DataTypes::GPS(gps_data) => {
                self.data_points.position.insert(
                    message.time,
                    Position {
                        position: Position::get_position(&gps_data),
                        gps_data,
                    },
                );
            }*/
        }
    }
}

impl Position {
    //get position from Coordinate.
    pub fn get_position(gps_data: &GPSData) -> Vec3 {
        let from = "EPSG:4326";
        let to = "EPSG:3857";
        let wsg84_to_epsg3857 = Proj::new_known_crs(from, to, None).unwrap();

        //warn!("THIS FUNCTION DOES NOT OPERATE HOW IT IS SUPPOST TO");
        let coordinate = wsg84_to_epsg3857
            .convert((gps_data.lon, gps_data.lat))
            .unwrap();
        Vec3::new(coordinate.0, 0., coordinate.1)
        //Vec3::ZERO
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
