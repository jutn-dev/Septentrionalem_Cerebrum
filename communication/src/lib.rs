use serde::{Deserialize, Serialize};


#[allow(unused)]
mod data {
    use glam::Vec3;

    enum DataTypes {
        FullTemp(FullTempData),
        Gyroscope(GyroscopeData)
    }

    struct FullTempData {
        temp_pressure: f32, 
        temp_gyro: f32, 
    }

    struct GyroscopeData {
        gyro: Vec3,
    }
}

/// The Seroal communication spec
///
/// Header byte
/// 0x06
///
/// Data section can contain any data.
/// if data contains header (0x06) or end byte (0xFF) or control byte (0x0A) it has to be marked with control byte (0x0A)
///
/// end byte
/// 0xFF
#[derive(Debug, Default, Clone)]
pub struct Serial {
    current_data: Vec<u8>,
    reading_data: bool,
    ctrl_byte_last: bool,
}

const HEADER: u8 = 0x06;
const END: u8 = 0xFF;
const CTRL_BYTE: u8 = 0x0A;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ReadError {
    NoCompletePacket,
    PostcardError(postcard::Error),
}

impl From<postcard::Error> for ReadError {
    fn from(value: postcard::Error) -> Self {
        ReadError::PostcardError(value)
    }
}

#[allow(dead_code)]
impl Serial {
    /// returns `None` if no comlete packet is read or `Vec<T>` if packets are recieved
    fn read<T: for<'a> Deserialize<'a>>(
        &mut self,
        input_buffer: Vec<u8>,
    ) -> Result<Vec<T>, ReadError> {
        let mut complete_packets = vec![];
        for byte in input_buffer {
            if byte == CTRL_BYTE && !self.ctrl_byte_last {
                self.ctrl_byte_last = true;
                continue;
            }

            if byte == HEADER && !self.ctrl_byte_last {
                self.reading_data = true;
                self.current_data.clear();
                continue;
            }

            if byte != END {
                if self.reading_data {
                    self.current_data.push(byte);
                }
                continue;
            }

            complete_packets.push(postcard::from_bytes(&self.current_data)?);
            self.reading_data = false;
            self.ctrl_byte_last = false;
            self.current_data.clear();
        }
        if complete_packets.is_empty() {
            return Err(ReadError::NoCompletePacket);
        }
        Ok(complete_packets)
    }

    pub fn to_message<T: Serialize>(&self, data: T) -> Result<Vec<u8>, postcard::Error> {
        let mut message_data = postcard::to_allocvec(&data)?;

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
        //data.push(data_type_id as u8);
        //data.push(message_data.len() as u8 + 4);
        data.append(&mut message_data);
        data.push(0xFF);

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct DataPoint {
        val1: i32,
        val2: u8,
        val3: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct DataPoint2 {
        val1: i32,
        val2: u8,
        val3: String,
    }
    ///DataTypes that can be sent through `Serial`
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum DataTypes {
        DataPoint(DataPoint),
        DataPoint2(DataPoint2),
    }

    #[test]
    fn serial_test() {
        let mut serial = Serial::default();
        let data_point = DataTypes::DataPoint(DataPoint {
            val1: 5,
            val2: 4,
            val3: String::from("testing"),
        });

        let deser = postcard::to_allocvec(&data_point).unwrap();
        let ser: DataTypes = postcard::from_bytes(&deser).unwrap();
        assert_eq!(data_point, ser);

        let data_u8 = serial.to_message(data_point.clone()).unwrap();
        println!("{:?} \n{:?}", deser, data_u8);
        let data = serial.read::<DataTypes>(data_u8.clone()).unwrap();
        assert_eq!(data_point, data[0]);

        let data_point2 = DataTypes::DataPoint2(DataPoint2 {
            val1: 5,
            val2: 4,
            val3: String::from("testing"),
        });
        let data2_u8 = serial.to_message(data_point2.clone()).unwrap();
        let data2 = serial.read::<DataTypes>(data2_u8.clone()).unwrap();
        assert_ne!(data_u8, data2_u8);
        assert_ne!(data[0], data2[0]);
    }
}
