use esp_idf_svc::hal;
use esp_idf_svc::hal::delay::BLOCK;
use esp_idf_svc::hal::gpio::AnyIOPin;
use esp_idf_svc::{hal::gpio::Pin, sys::EspError};

use crate::pressure_sensor::{read_pressure, read_temp};

mod pressure_sensor;

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = esp_idf_svc::hal::peripherals::Peripherals::take().unwrap();
    //let mut led = hal::gpio::PinDriver::output(peripherals.pins.gpio17)?;
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let i2c = peripherals.i2c0;


    let uart2_config = hal::uart::config::Config::new().baudrate(hal::units::Hertz(9_600));
    let uart2 = esp_idf_svc::hal::uart::UartDriver::new(
        peripherals.uart2,
        peripherals.pins.gpio17,
        peripherals.pins.gpio16,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &uart2_config,
    )?;
    let delay = esp_idf_svc::hal::delay::Delay::new_default();


    let baro_config = hal::i2c::config::Config::new(); //.tx_buffer_length(128).rx_buffer_length(128);
    let mut baro = hal::i2c::I2cDriver::new(i2c, sda, scl, &baro_config)?;
    

    loop {
        delay.delay_ms(100);
        //        uart2.write(&[255])?;
        //let size = uart2.read(&mut buf, BLOCK)?;
        log::info!("Temp: {:?}",read_temp(&mut baro)?);
        log::info!("Pressure: {:?}",read_pressure(&mut baro)?);
    }
}

