use std::{io::Read, time::Duration};

use crate::data::Data;
use bevy::prelude::*;
use communication::{ReadError, data::DataTypes};
use serialport::SerialPort;
#[cfg(unix)]
use serialport::TTYPort;

#[cfg(windows)]
use serialport::COMPort;

pub struct SerialPortDataPlugin;

#[derive(Debug, Resource, Default)]
struct SerialPortData {
    #[cfg(unix)]
    serialport: Option<TTYPort>,
    #[cfg(windows)]
    serialport: Option<COMPort>,

    binary_serial_handler: communication::Serial,
}

#[derive(Debug, Clone, Message)]
pub struct InitSerialPortMessage {
    pub path: String,
    pub baudrate: u32,
}

impl Plugin for SerialPortDataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SerialPortData>()
            .add_message::<InitSerialPortMessage>()
            .add_systems(Update, (init_serial_port, read_serial_data));
    }
}

fn read_serial_data(mut serial_port_data: ResMut<SerialPortData>, mut data: ResMut<Data>) {
    let Some(serial_port) = &mut serial_port_data.serialport else {
        return;
    };
    let mut buffer  = vec![0; 1];

    let number = match serial_port.read(buffer.as_mut_slice()) {
        Ok(num) => num,
        Err(error) => {
            error!("Serialport failed to read: {}", error);
            return;
        }
    };
    let in_data = match serial_port_data
        .binary_serial_handler
        .read::<DataTypes>(buffer)
    {
        Ok(in_data) => in_data,
        Err(error) => {
            if error == ReadError::NoCompletePacket {
                return;
            }
            // TODO better error
            error!("Packet failed to read {:?}", error);
            return;
        }
    };
    info!("indata: {:?}", in_data);
    data.extend(in_data);
}

fn init_serial_port(
    mut serialport: ResMut<SerialPortData>,
    mut message: MessageReader<InitSerialPortMessage>,
) {
    for msg in message.read() {
        serialport.binary_serial_handler = communication::Serial::default();
        serialport.serialport = match serialport::new(msg.path.clone(), msg.baudrate)
            .timeout(Duration::from_millis(10))
            .open_native()
        {
            Ok(mut port) => {
                port.write_request_to_send(false).unwrap();
                Some(port)
            }
            Err(error) => {
                error!("Error opening serialport: {}", error);
                None
            }
        };
    }
}
