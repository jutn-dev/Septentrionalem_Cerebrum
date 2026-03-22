use communication::data::CO2SensorData;
use esp_idf_svc::{hal::{delay::{BLOCK, FreeRtos}, i2c::I2cDriver}, sys::EspError};


const SCD41_I2C_ADDRESS:u8 = 0x62;
const START_PERIODIC_MEASUREMENT: u16 = 0x21b1;
const READ_MEASUREMENT: u16 = 0xec05;
const REINIT:u16 = 0x3646;
const SOTP_MEASUREMENT: u16 = 0x3f86;

pub struct SCD41;

impl SCD41 {
    pub fn init_measurements(i2c_driver: &mut I2cDriver) -> Result<Self, EspError> {
        i2c_driver.write(SCD41_I2C_ADDRESS, &SOTP_MEASUREMENT.to_be_bytes(), BLOCK)?;
        FreeRtos::delay_ms(500);
        i2c_driver.write(SCD41_I2C_ADDRESS, &REINIT.to_be_bytes(), BLOCK)?;
        FreeRtos::delay_ms(20);
        i2c_driver.write(SCD41_I2C_ADDRESS, &START_PERIODIC_MEASUREMENT.to_be_bytes(), BLOCK)?;
        Ok(Self)
    }
    pub fn read(&self, i2c_driver: &mut I2cDriver) -> Result<CO2SensorData, EspError> {
        let mut buf = [0; 9];
        i2c_driver.write_read(SCD41_I2C_ADDRESS, &READ_MEASUREMENT.to_be_bytes(),&mut buf,  BLOCK)?;

        //TODO check the crc
        Ok(CO2SensorData {
            co2: u16::from_be_bytes(buf[0..2].try_into().unwrap()),
            temp: (-45. + (175. *(u16::from_be_bytes(buf[4..6].try_into().unwrap()) as f32))) / f32::powf(2., 16.),
            humidity: 100. * (u16::from_be_bytes(buf[6..8].try_into().unwrap()) as f32) / f32::powf(2., 16.),

        })
    }
}
