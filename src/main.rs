use serialport::{available_ports, SerialPort, SerialPortType};
use std::time::Duration;

fn main() {
    println!("Available ports:");

    let ports_info = serialport::available_ports().expect("No ports found!");

    println!("Found {} ports", ports_info.len());

    // for p in ports {
    // }

    // let mut port: Option<Box<dyn SerialPort>> = None;
    let mut port_name: Option<String> = None; //String::new();

    for p_info in ports_info {
        println!("Port name: {}", p_info.port_name);
        match p_info.port_type {
            SerialPortType::UsbPort(info) => {
                println!("USB device: VID: {:04x} PID: {:04x}", info.vid, info.pid);

                if info.vid == 0x067b && info.pid == 0x23d3 {
                    port_name = Some(p_info.port_name);
                    break;
                } else {
                    println!("Not the device we are looking for.");
                }
            }
            SerialPortType::BluetoothPort => {
                println!("Bluetooth device");
            }
            SerialPortType::PciPort => {
                println!("PCI device");
            }
            SerialPortType::Unknown => {
                println!("Unknown device");
            }
        }
    }

    if port_name.is_none() {
        println!("Device not found.");
        panic!("Device not found.");
    }

    let port = serialport::new(port_name.unwrap(), 9600)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");
}
