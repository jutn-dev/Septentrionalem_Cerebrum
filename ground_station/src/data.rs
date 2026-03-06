use std::{fs::File, io::BufReader};

use bevy::prelude::*;
use communication::data::DataTypes;
use proj::Proj;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Position {
    pub lon: f32,
    pub lat: f32,
    // 3d position to be used in the engine
    pub position: Vec3,
}

#[derive(Debug, Clone, Default, Resource)]
///Data from CanSat
pub struct Data {
    //data points are expected to be in chronological order.
    //If not you might get some unexpected results.
    pub data_points: Vec<DataPoint>,
    pub current_time: u64,
    center_position: Vec3,
}

///one point of data
///
///why is there two `DataPoint` sturcts?
///I want to be able to use rust's type safty to ensure that variables
///which get computed on the groundstation such as `position` get initalized properly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub time: u64,
    pub position: Option<Position>,
    pub air_pressure: Option<f32>,
    pub gyroscope: Option<Vec3>,
    pub acceleration: Option<Vec3>,
}

#[allow(unused)]
impl Data {
    pub fn data_form_json_file(path: String) -> Result<Self, std::io::Error> {
        match File::open(path) {
            Ok(json) => {
                let reader = BufReader::new(json);
                let mut data: Vec<DataPoint> = serde_json::from_reader(reader)?;
                data.sort();

                // if coming from previous git commit
                // positions do not get created automatically

                return Ok(Data {
                    center_position: Data::get_center_position(&data),
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
    // vec should be sorted when using this function
    // if no coordinates found returns Vec3::Zero
    fn get_center_position(datapoints: &Vec<DataPoint>) -> Vec3 {
        for datapoint in datapoints {
            if let Some(position) = &datapoint.position {
                return position.position;
            }
        }
        Vec3::ZERO
    }
    /// returns relative position of data point. If no data point exists or specific `DataPoint`
    /// doesn't exists returns `None`
    pub fn get_point_relative_position(&self, data_point: &DataPoint) -> Option<Vec3> {
        let Some(data_point_position) = &data_point.position else {
            return None;
        };
        let relative_position = self.center_position - data_point_position.position;
        Some(relative_position)
    }

    pub fn get_relative_time(&self, data_point: &DataPoint) -> Option<u64> {
        if self.data_points.is_empty() {
            return None;
        }

        let relative_time = self.data_points[0].time - data_point.time;
        Some(relative_time)
    }

    pub fn get_closest_point_in_time(&self, time: u64) -> Option<&DataPoint> {
        self.data_points
            .iter()
            .min_by_key(|d| d.time.abs_diff(time))
    }

    pub fn extend(&mut self, data_types: Vec<DataTypes>) {
        let data_points: Vec<DataPoint> = data_types.iter().map(|d| DataPoint::from_data_types(d)).collect();
        self.data_points.extend(data_points);
    }
}

impl DataPoint {
    //get position from Coordinate.
    //this function should only be used when creating DataPoint
    fn get_position(in_data: &Position) -> Vec3 {
        let from = "EPSG:4326";
        let to = "EPSG:3857";
        let wsg84_to_epsg3857 = Proj::new_known_crs(from, to, None).unwrap();

        let coordinate = wsg84_to_epsg3857
            .convert((in_data.lon, in_data.lat))
            .unwrap();
        Vec3::new(coordinate.0, 0., coordinate.1)
    }
    pub fn from_data_types(data_types: &DataTypes) -> Self {
        match data_types {
            DataTypes::Pressure(pressure_data) => Self {
                time: pressure_data.time,
                position: None,
                air_pressure: Some(pressure_data.pressure),
                gyroscope: None,
                acceleration: None,
            },
        }
    }
}

impl Ord for DataPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}
impl PartialOrd for DataPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for DataPoint {
    /// if self.time is same
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl Eq for DataPoint {}
