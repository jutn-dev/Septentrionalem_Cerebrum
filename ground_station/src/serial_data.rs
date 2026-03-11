use std::{io::Read, time::Duration};

use crate::data::Data;
use bevy::prelude::*;
use communication::data::DataTypes;
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
    //TODO choose better size for buffer
    let mut buffer  = vec![0;255];

    let buffer_lenght = match serial_port.read(buffer.as_mut_slice()) {
        Ok(len) => len,
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => return,
        Err(error) => {
            error!("Serialport failed to read: {}", error);
            return;
        }
    };

    println!("buffer: {:?}", buffer);
    let Some(incoming_data) = serial_port_data.binary_serial_handler
        .read::<DataTypes>(buffer[..buffer_lenght].to_vec())
    else {return;};
    for data_point in incoming_data {
        match data_point {
        Ok(ok_data) => {
            println!("{:?}", ok_data);
            data.extend(&ok_data)
        }
        Err(error) => {
            // TODO better error
            error!("Packet failed to read {:?}", error);
//            info!("errored packet data: {:?}", serial_port_data);
            return;
        }}
    };
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
