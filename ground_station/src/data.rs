use std::{fs::File, io::BufReader};

use bevy::prelude::*;
use proj::Proj;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Resource, Reflect)]
///Data from CanSat
pub struct Data {
    //data points are expected to be in chronological order.
    //If not you might get some unexpected results.
    pub data_points: Vec<DataPoint>,
    pub current_time: u64,
}

///one point of data
///
///why is there two `DataPoint` sturcts?
///I want to be able to use rust's type safty to ensure that variables
///which get computed on the groundstation such as `position` get initalized properly.
#[derive(Debug, Clone, Reflect)]
pub struct DataPoint {
    ///time of gps
    pub time: u64,

    //coordinates
    pub lon: f32,
    pub lat: f32,
    // 3d position to be used in the engine
    pub position: Vec3,

    pub air_pressure: f64,
    gyroscope: Vec3,
    //acceleration: Vec3,
}

///incoming data points frpm CanSat
#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
struct InDataPoint {
    ///time of gps
    pub time: u64,

    //coordinates
    pub lon: f32,
    pub lat: f32,

    //sensors
    pub air_pressure: f64,
    //gyroscope: Vec3,
    //acceleration: Vec3,
}

#[allow(unused)]
impl Data {
    pub fn data_form_json_file(path: String) -> Result<Self, std::io::Error> {
        match File::open(path) {
            Ok(json) => {
                let reader = BufReader::new(json);
                let in_data: Vec<InDataPoint> = serde_json::from_reader(reader)?;
                let mut data: Vec<DataPoint> =
                    in_data.iter().map(DataPoint::from_in_data_point).collect();
                data.sort();

                Ok(Data {
                    data_points: data,
                    current_time: 0,
                })
            }
            Err(error) => {
                error!("Loading json file failed: {error}");
                Err(error)
            }
        }
    }
    /// returns relative position of data point. If no data point exists or specific `DataPoint`
    /// doesn't exists returns `None`
    pub fn get_point_relative_position(&self, data_point: &DataPoint) -> Option<Vec3> {
        if self.data_points.is_empty() {
            return None;
        }

        let relative_position = self.data_points[0].position - data_point.position;
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
}

impl DataPoint {
    pub(self) fn from_in_data_point(in_data: &InDataPoint) -> Self {
        Self {
            time: in_data.time,
            lon: in_data.lon,
            lat: in_data.lat,
            position: Self::get_position(in_data),
            air_pressure: in_data.air_pressure,
        }
    }
    //get position from in_data.
    //this function should only be used when creating DataPoint
    fn get_position(in_data: &InDataPoint) -> Vec3 {
        let from = "EPSG:4326";
        let to = "EPSG:3857";
        let wsg84_to_epsg3857 = Proj::new_known_crs(from, to, None).unwrap();

        let coordinate = wsg84_to_epsg3857
            .convert((in_data.lon, in_data.lat))
            .unwrap();
        Vec3::new(coordinate.0, 0., coordinate.1)
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
