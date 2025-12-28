use std::{fs::File, io::{BufReader, Read}};

use bevy::{prelude::*};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Deserialize, Serialize, Reflect)]
pub struct DataPoint {
    pub time: u64,
    pub lon: f32,
    pub lat: f32,
    pub air_pressure: f32,
    //gyroscope: Vec3,
    //acceleration: Vec3,
}


#[derive(Debug, Clone, Resource, Deserialize, Serialize, Reflect)]
///Data from CanSat 
pub struct Data {
    pub data_points: Vec<DataPoint>
}

impl Data {
    pub fn data_form_json_file(path: &str) -> Result<Self, std::io::Error> {
        match File::open(path) {
            Ok(json) => {
                let reader = BufReader::new(json);
                let data: Vec<DataPoint> = serde_json::from_reader(reader).unwrap();
                Ok(Data { data_points: data })
            },
            Err(error) => {
                error!("Loading json file failed: {error}");
                Err(error)
            },
        }
    }

}





