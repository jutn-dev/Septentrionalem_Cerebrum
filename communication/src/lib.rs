use std::vec;

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

///DataTypes that can be sent through `Serial`
pub enum DataTypes {
    DataPoint,
}

pub struct Message {
    data_type: DataTypes
}

/// The Seroal communication spec
///
/// Header 2 bytes
/// 0x06 and 0x01
///
/// packet type id 1 byte
///
/// full packet sizes including header, type, size, data and end byte
///
/// Data section can contain any data.
/// if data contains header (0x06 0x01) or end byte (0xFF) or control byte (\) it has to be marked with control byte \
///
/// end byte
/// 0xFF
#[derive(Default)]
pub struct Serial {
    current_data: Vec<u8>,
}



trait DataType<T> {
    fn get_data() -> Vec<u8>;
    fn to_data() -> T;
}

const HEADER: u8 = 0x06;
const END: u8 = 0xFF;
const CTRL_BYTE: u8 = 0x0A;

#[allow(dead_code)]
impl Serial {

    fn read(&mut self, input_buffer: Vec<u8>) -> Option<(DataTypes, Vec<u8)>> {

    }

    pub fn to_message(
        data_type: DataTypes,
        mut message_data: Vec<u8>,
    ) -> Vec<u8> {
        // adding control bytes if needed
        let mut indexes = vec![];
        for (i, d) in message_data.iter().enumerate() {
            if *d == HEADER || *d == END || *d == CTRL_BYTE {
                indexes.push(i);
            }
        }

        if !indexes.is_empty() {
            for i in indexes.iter().rev() {
                message_data.insert(*i, CTRL_BYTE);
            }
        }

        //constructing data
        let mut data: Vec<u8> = vec![HEADER];
        data.push(data_type as u8);
        data.push(message_data.len() as u8 + 4);
        data.append(&mut message_data);
        data.push(0xFF);

        data
    }
}

impl DataPoint {
    //fn to_binary() -> [u8; size_of::<DataPoint>()] {}

    //fn from_binary(data: [u8; size_of::<DataPoint>()]) -> Self {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_message_test() {

        let message_data = vec![0x04, 0x07, 0x43, 0x5];
        assert_eq!(Serial::to_message(DataTypes::DataPoint, message_data), [HEADER, 0x0, 0x8, 0x04, 0x07, 0x43, 0x5, END]);

        let message_data = vec![0x04, END, 0x43, 0x5];
        assert_eq!(Serial::to_message(DataTypes::DataPoint, message_data), [HEADER, 0x0, 0x9, 0x04, CTRL_BYTE, END, 0x43, 0x5, END]);
    }
}
