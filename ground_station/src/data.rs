use std::{fs::File, io::BufReader};

use bevy::prelude::*;
use proj::Proj;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Resource, Reflect)]
///Data from CanSat
pub struct Data {
    pub data_points: Vec<DataPoint>,
    pub current_data: usize,
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

    pub air_pressure: f32,
    //gyroscope: Vec3,
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
    pub air_pressure: f32,
    //gyroscope: Vec3,
    //acceleration: Vec3,
}

impl Data {
    pub fn data_form_json_file(path: &str) -> Result<Self, std::io::Error> {
        match File::open(path) {
            Ok(json) => {
                let reader = BufReader::new(json);
                let in_data: Vec<InDataPoint> = serde_json::from_reader(reader).unwrap();
                let data: Vec<DataPoint> = in_data
                    .iter()
                    .map(|d| DataPoint::from_in_data_point(d))
                    .collect();
                Ok(Data {
                    data_points: data,
                    current_data: 0,
                })
            }
            Err(error) => {
                error!("Loading json file failed: {error}");
                Err(error)
            }
        }
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
