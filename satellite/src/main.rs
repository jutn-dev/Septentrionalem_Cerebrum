use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use communication::data::{DataTypes, Message};
use communication::Serial;
use esp_idf_svc::hal;
use esp_idf_svc::hal::delay::{FreeRtos, BLOCK};
use esp_idf_svc::hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::{hal::gpio::Pin, sys::EspError};

use crate::gps::GPSDriver;
use crate::pressure_sensor::PressureSensor;
use crate::scd41::SCD41;

mod gps;
mod pressure_sensor;
mod scd41;

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

    let uart_config = hal::uart::config::Config::new().baudrate(hal::units::Hertz(9_600));
    let uart2 = esp_idf_svc::hal::uart::UartDriver::new(
        peripherals.uart2,
        peripherals.pins.gpio17,
        peripherals.pins.gpio16,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &uart_config,
    )?;
    let delay = esp_idf_svc::hal::delay::Delay::new_default();

    let i2c_config = hal::i2c::config::Config::new()
        .baudrate(Hertz(100_000))
        .scl_enable_pullup(true)
        .sda_enable_pullup(true); //.tx_buffer_length(128).rx_buffer_length(128);
    let mut i2c_driver = Arc::new(Mutex::new(hal::i2c::I2cDriver::new(
        i2c,
        sda,
        scl,
        &i2c_config,
    )?));

    let mut time = 0;
    let (tx, rx) = std::sync::mpsc::channel();
    let i2c_driver_clone = Arc::clone(&i2c_driver);
    let tx_clone = tx.clone();
    
    std::thread::spawn(move || {
        pressure_sensor_thread(i2c_driver_clone, tx_clone).unwrap();
    });
    let i2c_driver_clone = Arc::clone(&i2c_driver);
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        co2_thread(i2c_driver_clone, tx_clone).unwrap();
    });
    let i2c_driver_clone = Arc::clone(&i2c_driver);
    std::thread::spawn(move || {
        gps_thread(i2c_driver_clone, tx).unwrap();
    });
    let mut set_pin = PinDriver::output(peripherals.pins.gpio15)?;
    set_pin.set_high()?;
    /*FreeRtos::delay_ms(500);
        set_pin.set_low()?;
        FreeRtos::delay_ms(1000);
        let mut buf = [0;32];
        uart2.write(b"WR 434000 3 9 3 0\r\n")?;
        println!("juups");
        //uart2.write(b"RD\r\n")?;
        println!("juups");
        FreeRtos::delay_ms(500);
        uart2.read(&mut buf, BLOCK)?;
        println!("juups: {:?}", buf);
        return Ok(());
    */
    println!("WHAAAAT");
    let bin_serial = Serial::default();
    loop {
        delay.delay_ms(10);
        //let mut buf = [0; 255];
        //uart2.read(&mut buf, BLOCK);
        time += 1;
        for data_type in rx.try_iter() {
            let message = Message::new(time, data_type);
            let data = bin_serial.to_message(message).unwrap();
            uart2.write(data.as_slice())?;
        }
    }
}

fn pressure_sensor_thread(
    i2c: Arc<Mutex<I2cDriver>>,
    tx: Sender<DataTypes>,
) -> Result<(), EspError> {
    let mut i2c_driver = i2c.lock().unwrap();
    let pressure_sesnor = PressureSensor::init(&mut i2c_driver)?;
    drop(i2c_driver);
    loop {
        FreeRtos::delay_ms(100);
        let mut i2c_driver = i2c.lock().unwrap();

        let temp = pressure_sesnor.read_temp(&mut i2c_driver)?;
        let pressure = pressure_sesnor.read_pressure(&mut i2c_driver)?;
        drop(i2c_driver);
        let data_type =
            DataTypes::PressureSensor(communication::data::PressureSensorData { temp, pressure });
        tx.send(data_type).unwrap();
    }
}

fn co2_thread(i2c: Arc<Mutex<I2cDriver>>, tx: Sender<DataTypes>) -> Result<(), EspError> {
    let mut i2c_driver = i2c.lock().unwrap();
    let c02_sensor = SCD41::init_measurements(&mut i2c_driver)?;
    drop(i2c_driver);
    loop {
        FreeRtos::delay_ms(5000);
        let mut i2c_driver = i2c.lock().unwrap();
        let data = c02_sensor.read(&mut i2c_driver)?;
        let data_type = DataTypes::CO2Sensor(data);
        tx.send(data_type).unwrap();
    }
}

fn gps_thread(i2c: Arc<Mutex<I2cDriver>>, tx: Sender<DataTypes>) -> Result<(), EspError> {
    let mut i2c_driver = i2c.lock().unwrap();

    let mut gps_driver = GPSDriver::init(&mut i2c_driver)?;
    drop(i2c_driver);
    loop {
        FreeRtos::delay_ms(200);
        let Some(data_vec) = gps_driver.get_data(&i2c)? else {
            continue;
        };
        for data in data_vec {
            tx.send(DataTypes::GPS(data)).unwrap();
            
        }
    }
}
