use core::f32;

use communication::data::GyroscopeData;
use esp_idf_svc::{hal::{delay::BLOCK, i2c::I2cDriver}, sys::EspError};
pub struct GyroSensor;

impl GyroSensor {
    pub fn init(i2c_driver: &mut I2cDriver) -> Result<Self, EspError> {
        i2c_driver.write(0x6A, &[0x11, 0x80], BLOCK)?;
        //TODO add hardware software reset thing
        Ok(Self)
    }
    pub fn read_gyro(&self, i2c_driver: &mut I2cDriver) -> Result<GyroscopeData, EspError> {
    let mut outx_l_g= [0;1];
    let mut outx_h_g= [0;1];
    i2c_driver.write_read(0x6A, &[0x22], &mut outx_l_g, BLOCK)?;
    i2c_driver.write_read(0x6A, &[0x23], &mut outx_h_g, BLOCK)?;
    let x = u16::from_le_bytes([outx_h_g[0], outx_l_g[0]]);

    let mut outy_l_g= [0;1];
    let mut outy_h_g= [0;1];
    i2c_driver.write_read(0x6A, &[0x24], &mut outy_l_g, BLOCK)?;
    i2c_driver.write_read(0x6A, &[0x25], &mut outy_h_g, BLOCK)?;
    let y = u16::from_le_bytes([outy_h_g[0], outy_l_g[0]]);

    let mut outz_l_g= [0;1];
    let mut outz_h_g= [0;1];
    i2c_driver.write_read(0x6A, &[0x26], &mut outz_l_g, BLOCK)?;
    i2c_driver.write_read(0x6A, &[0x27], &mut outz_h_g, BLOCK)?;
    let z = u16::from_le_bytes([outz_h_g[0], outz_l_g[0]]);
       Ok(GyroscopeData {
            gyro: glam::Vec3 { x: 8.75 * 0.001 * x as f32, y: 8.75 * 0.001 * y as f32, z: 8.75 * 0.001 * z as f32 }
    })
    }

    pub fn read_acceleration(&self, i2c_driver: &mut I2cDriver) -> Result<f32, EspError> {
        Ok(3.)
    }
}
