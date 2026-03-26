use std::fs::File;

use communication::data::{DataTypes, Message};
use esp_idf_svc::{fs::fatfs::Fatfs, hal::{gpio::AnyIOPin, peripherals::Peripherals, sd::{SdCardConfiguration, SdCardDriver, spi::SdSpiHostDriver}, spi::{Dma, SpiDriver, SpiDriverConfig}}, sys::EspError};

pub fn mount_sd_card(peripherals: Peripherals) -> Result<(), EspError> {
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
    let sd_card_driver = SdCardDriver::new_spi(spi, &SdCardConfiguration::new())?;

    let _mounted_sd = esp_idf_svc::io::vfs::MountedFatfs::mount(
        Fatfs::new_sdcard(0, sd_card_driver)?,
        "/sdcard",
        4,
    )?;
    Ok(())
}

pub fn write_data_types(message: Message, time: u64) {
    match message.data {
        DataTypes::PressureSensor(_) => write_into_file(message, time, "/sdcard/pressure.csv"),
        DataTypes::CO2Sensor(_) => write_into_file(message, time, "/sdcard/co2.csv"),
        DataTypes::GPS(_) => write_into_file(message, time, "/sdcard/gps.csv"),

    }
}

fn write_into_file<T: serde::Serialize>(item: T, time: u64, path: &str) {
    let file = File::options().append(true).create(true).open(path).unwrap();
    let mut csv = csv::Writer::from_writer(file);
    csv.serialize(item).unwrap();
    csv.flush().unwrap();
}
