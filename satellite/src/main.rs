use std::fs::File;
use std::io::Write;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use communication::data::{DataTypes, Message, MiscData};
use communication::Serial;
use esp_idf_svc::fs::fatfs::Fatfs;
use esp_idf_svc::hal;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::delay::{BLOCK, FreeRtos};
use esp_idf_svc::hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::hal::ledc::config::TimerConfig;
use esp_idf_svc::hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_svc::hal::sd::{SdCardConfiguration, SdCardDriver};
use esp_idf_svc::hal::sd::spi::SdSpiHostDriver;
use esp_idf_svc::hal::spi::{Dma, SpiDriver, SpiDriverConfig};
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::sys::EspError;

use crate::gps::GPSDriver;
use crate::pressure_sensor::PressureSensor;
use crate::scd41::SCD41;
use crate::sd_card::{mount_sd_card, write_data_types};

mod gps;
mod pressure_sensor;
mod scd41;
mod sd_card;

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut peripherals = esp_idf_svc::hal::peripherals::Peripherals::take().unwrap();
    
    //let mut led = hal::gpio::PinDriver::output(ripherals.pins.gpio17)?;
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let i2c = peripherals.i2c0;

    let mut ldr_enable_pin = PinDriver::output(peripherals.pins.gpio13)?;
    ldr_enable_pin.set_high()?;


    let mut uart_set_pin = PinDriver::output(peripherals.pins.gpio15)?;
    uart_set_pin.set_high()?;
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
    let (tx, rx) = std::sync::mpsc::channel::<DataTypes>();
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
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        gps_thread(i2c_driver_clone, tx_clone).unwrap();
    });
    
    //ldr 
    let adc = AdcDriver::new(peripherals.adc1)?;
    let adc_pin = AdcChannelDriver::new(adc, peripherals.pins.gpio34, &AdcChannelConfig::new())?;
    let timer_driver = LedcTimerDriver::new(peripherals.ledc.timer0, &TimerConfig::new().frequency(Hertz(50)))?;
    let servo = LedcDriver::new(peripherals.ledc.channel0, timer_driver, peripherals.pins.gpio27)?;
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        ldr_opening_system_thread(servo, adc_pin, tx_clone).unwrap();
    });
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
    let spi_driver_config = SpiDriverConfig::new().dma(Dma::Auto(4096));
    let spi_driver = SpiDriver::new(
        peripherals.spi3,
        peripherals.pins.gpio18,
        peripherals.pins.gpio23,
        Some(peripherals.pins.gpio19),
        &spi_driver_config,
    )?;
    let spi = SdSpiHostDriver::new(
        spi_driver,
        Some(peripherals.pins.gpio4),
        None::<AnyIOPin>,
        None::<AnyIOPin>,
        None::<AnyIOPin>,
        None,
    )?;
    let mut sd_card_available = false;
    let mut _mounted_sd;
    if let Ok(sd_card_driver) = SdCardDriver::new_spi(spi, &SdCardConfiguration::new()) {

        _mounted_sd = esp_idf_svc::io::vfs::MountedFatfs::mount(
            Fatfs::new_sdcard(0, sd_card_driver)?,
            "/sdcard",
            4,
        ).expect("sd card not partitioned properly");
        sd_card_available = true;
    }
    else {
        log::error!("no sd card found")
    }
    let mut bin_serial = Serial::default();
    loop {
        FreeRtos::delay_ms(10);
        time += 1;
        for data_type in rx.try_iter() {
            let message = Message::new(time, data_type);
            let data = bin_serial.to_message(message.clone()).unwrap();
            uart2.write(data.as_slice())?;
            if sd_card_available {
                write_data_types(message);
            }
        }
        let mut buf = [0; 255];
        /*
        match uart2.read(&mut buf, ) {
            Err(_) => (),
            Ok(num) => {
                if let Some(signals) = bin_serial.read::<communication::data::SatControl>(buf[..num].to_vec())
                {
                    for signal in signals {
                        let Ok(signal) = signal else {
                            continue;
                        };
                        match signal {
                            communication::data::SatControl::CloseMotor => (),
                        }
                        
                    }
                }
        //let mut buf = [0; 255];
        //uart2.read(&mut buf, BLOCK);
            }

        }
        */
    }
}

fn ldr_opening_system_thread<'a>(
    mut servo: LedcDriver,
    mut adc: AdcChannelDriver<'a, hal::gpio::Gpio34, AdcDriver<'a, hal::adc::ADC1>>,
    tx: Sender<DataTypes>,
) -> Result<(), EspError> {
    servo.set_duty(10)?; 
    FreeRtos::delay_ms(5000);
   loop {
       FreeRtos::delay_ms(100);
       let ldr = adc.read()?;
       println!("ldr: {:?}", ldr);
        if ldr < 900 {
        FreeRtos::delay_ms(6000);
        servo.set_duty(15)?;
        tx.send(DataTypes::Misc(MiscData {ldr})).unwrap();
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
    let Ok(c02_sensor) = SCD41::init_measurements(&mut i2c_driver) else {
        log::error!("failed to init co2 sensor");
        return Ok(());
    };
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

    let Ok(mut gps_driver) = GPSDriver::init(&mut i2c_driver) else {
        log::error!("failed to init gps sensor");
        return Ok(());
    };
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
