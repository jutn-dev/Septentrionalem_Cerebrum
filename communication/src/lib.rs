use serde::{Deserialize, Serialize};

///incoming data points frpm CanSat
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataPoint {
    ///time of gps
    pub time: u64,

    //coordinates
    pub lon: f32,
    pub lat: f32,

    //sensors
    pub tempeurature: f32,
    pub air_pressure: f64,
    //gyroscope: Vec3,
    //acceleration: Vec3,
}

impl DataPoint {
    fn to_binary() -> [u8; size_of::<DataPoint>()] {}

    fn from_binary(data: [u8; size_of::<DataPoint>()]) -> Self {
    


    }
}
