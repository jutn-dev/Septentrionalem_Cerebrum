use std::sync::{Arc, Mutex};

use esp_idf_svc::{
    hal::{
        delay::BLOCK,
        i2c::I2cDriver,
    },
    sys::EspError,
};
use communication::data::GPSData;

#[derive(Debug, Clone)]
pub struct GPSDriver {
    current_data: Vec<u8>,
    reading_data: bool,
}

impl GPSDriver {
    pub fn init(i2c_driver: &mut I2cDriver) -> Result<Self, EspError> {
            i2c_driver.write(
        0x10,
        b"$PMTK314,1,1,1,1,1,5,0,0,0,0,0,0,0,0,0,0,0,1,0*2D \r\n",
        BLOCK,
    )?;
    i2c_driver.write(0x10, b"$PMTK353,1,1,0,0,0*2B\r\n", BLOCK)?;
    i2c_driver.write(0x10, b"$PMTK220,200*2C\r\n", BLOCK)?;
    //i2c_driver.write_read(0x10, b"$GPGGA*56\r\n", &mut buf, BLOCK)?;
    //i2c_driver.write_read(0x10, b"$GNGGA,165006.000,2241.9107,N,12017.2383,E,1,14,0.79,22.6,M,18.5,M,,*42\r\n", &mut buf, BLOCK)?;
    Ok(GPSDriver {
        current_data: vec![],
        reading_data: false,
    })

    }
    pub fn get_data(&mut self, i2c: &Arc<Mutex<I2cDriver>>) -> Result<Option<Vec<GPSData>>, EspError> {
        //TODO ADD TIMEOUTS
        let mut buf = [0; 64];
        let mut i2c_driver = i2c.lock().unwrap();
        i2c_driver.read(0x10, &mut buf, BLOCK)?;
        drop(i2c_driver);
        let Some(strings) = self.parse(buf.to_vec()) else {
            return Ok(None);
        };
        let mut gps_datas = vec![];
        for s in strings {
            let data: Vec<&str> = s.split(',').collect();
            if data[0] == "GNGGA" {
                if data[2].is_empty() {
                    continue;
                }

                let n_or_s = data[3] == "N";
                let w_or_e = data[5] == "W";

                let lat_degree: f32 = data[2][0..2].parse().unwrap();
                let lat_minutes: f32 = data[2][2..].parse().unwrap();
                let lat = lat_degree + (lat_minutes/60.);

                let lon_degree: f32 = data[4][0..3].parse().unwrap();
                let lon_minutes: f32 = data[4][3..].parse().unwrap();
                let lon = lon_degree + (lon_minutes/60.);

                let gps = GPSData {
                    lat,
                    lon,
                    n_or_s,
                    w_or_e,
                    satellites_used: data[7].parse().unwrap(),
                    altitude: data[9].parse().unwrap(),
                };
                log::info!("gps: {:?}", gps);
                gps_datas.push(gps);
            }
        }
        if gps_datas.is_empty() {
            return Ok(None);
        }
        Ok(Some(gps_datas))
    }

    fn parse(&mut self, input_buffer: Vec<u8>) -> Option<Vec<String>> {
        let mut complete_packets = vec![];
        for byte in input_buffer {
            if byte == u8::from_ne_bytes(*b"$") {
                self.reading_data = true;
                self.current_data.clear();
                continue;
            }

            if byte != u8::from_ne_bytes(*b"\r") {
                if self.reading_data {
                    self.current_data.push(byte);
                }
                continue;
            }

            complete_packets.push(
                String::from_utf8(self.current_data.clone())
                    .unwrap()
                    .replace("\n", ""),
            );
            self.reading_data = false;
            self.current_data.clear();
        }
        if complete_packets.is_empty() {
            return None;
        }
        Some(complete_packets)
    }
}
