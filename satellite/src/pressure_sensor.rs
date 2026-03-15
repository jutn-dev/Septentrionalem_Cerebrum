use esp_idf_svc::hal;
use esp_idf_svc::hal::delay::BLOCK;
use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::sys::EspError;

pub struct PressureSensor;

impl PressureSensor {
    pub fn init(i2c_driver: &mut I2cDriver) -> Result<Self, EspError> {
        //TODO add hardware software reset thing
        let baro_address = 0x5C;

        let mut ctrl_reg1 = [0; 1];
        i2c_driver.write_read(baro_address, &[0x10], &mut ctrl_reg1, BLOCK)?;
        //set 75hz
        ctrl_reg1[0] |= 0b0011_0000;
        i2c_driver.write_read(baro_address, &[0x10, ctrl_reg1[0]], &mut ctrl_reg1, BLOCK)?;
        Ok(Self)
    }
    pub fn read_temp(&self, i2c_driver: &mut I2cDriver) -> Result<f32, EspError> {
        let baro_address = 0x5C;
        let mut temp_l = [0; 1];
        let mut temp_h = [0; 1];
        i2c_driver.write_read(baro_address, &[0x2C], &mut temp_l, BLOCK)?;
        i2c_driver.write_read(baro_address, &[0x2B], &mut temp_h, BLOCK)?;

        let temp = ((temp_l[0] as u16) << 8 | temp_h[0] as u16) as f32 / 100.;
        Ok(temp)
    }

    pub fn read_pressure(&self, i2c_driver: &mut I2cDriver) -> Result<f32, EspError> {
        let baro_address = 0x5C;
        let mut pressure_h = [0; 1];
        let mut pressure_l = [0; 1];
        let mut pressure_xl = [0; 1];
        i2c_driver.write_read(baro_address, &[0x2A], &mut pressure_h, BLOCK)?;
        i2c_driver.write_read(baro_address, &[0x29], &mut pressure_l, BLOCK)?;
        i2c_driver.write_read(baro_address, &[0x28], &mut pressure_xl, BLOCK)?;

        let combined_pressure = (((pressure_h[0] as u32) << 8 | pressure_l[0] as u32) << 8
            | pressure_xl[0] as u32) as f32;

        //convert to hPa
        let pressure = combined_pressure / 4096.;
        Ok(pressure)
    }
}
