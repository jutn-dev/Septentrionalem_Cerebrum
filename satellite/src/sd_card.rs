use std::{fs::File, io::Write};

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

pub fn write_data_types(message: Message) {
    match message.data {
        DataTypes::PressureSensor(_) => write_into_file(message, "/sdcard/pressure.json"),
        DataTypes::CO2Sensor(_) => write_into_file(message, "/sdcard/co2.json"),
        DataTypes::GPS(_) => write_into_file(message, "/sdcard/gps.json"),
    }
}

fn write_into_file<T: serde::Serialize>(item: T, path: &str) {
    log::info!("file: {}", path);
    let mut file = File::options().read(true).append(true).create(true).open(path).unwrap();
    let json_string = serde_json::to_string(&item).unwrap();
    write!(file, "{}", json_string);
    /*let mut csv = csv::Writer::from_writer(file);
    csv.serialize(item).unwrap();
    csv.flush().unwrap();
    */
}
