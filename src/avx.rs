/**
 * @file avx.rs
 * @author Mit Bailey (mitbailey@outlook.com)
 * @brief
 * @version See Git tags for version information.
 * @date 2024.04.15
 *
 * @copyright Copyright (c) 2024
 *
 */

 use serialport::{available_ports, SerialPort, SerialPortType};
use std::error::Error;
use std::time::Duration;
use std::{io, str};

mod coordinates;
pub use coordinates::{AzmAlt, RADec};

const REV: i64 = 0x100000000;

// Input slew rate in arcseconds/second
fn slew_rate(rate: u16) -> (u8, u8) {
    (
        ((rate * 4) / 256).try_into().unwrap(),
        ((rate * 4) % 256).try_into().unwrap(),
    )
}

pub enum TrackingMode {
    Off = 0,
    AltAz = 1,
    EQNorth = 2,
    EQSouth = 3,
}

pub enum SlewAxis {
    RAAzm = 0,
    DecAlt = 1,
}

pub enum SlewDir {
    Positive = 0,
    Negative = 1,
}

pub enum SlewRate {
    Stop = 0,
    Rate1 = 1,
    Rate2 = 2,
    Rate3 = 3,
    Rate4 = 4,
    Rate5 = 5,
    Rate6 = 6,
    Rate7 = 7,
    Rate8 = 8,
    Rate9 = 9,
}

pub struct AdvancedVX {
    port: Box<dyn SerialPort>,
}

impl Default for AdvancedVX {
    fn default() -> Self {
        Self::new().expect("Failed to create AdvancedVX object.")
    }
}

impl AdvancedVX {
    pub fn new() -> Result<AdvancedVX, io::Error> {
        println!("Available ports:");

        let ports_info = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                eprintln!("Error listing serial ports: {:?}", e);
                return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
            }
        };

        println!("Found {} ports", ports_info.len());

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

        match &port_name {
            Some(p) => println!("Found device: {}", p),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "AVX device not found.",
                ))
            }
        }

        Ok(AdvancedVX {
            // "Software drivers should be prepared to wait up to 3.5s (worst case scenario) for a hand control response."
            port: serialport::new(port_name.unwrap(), 9600)
                .timeout(Duration::from_millis(3500))
                .stop_bits(serialport::StopBits::One)
                .parity(serialport::Parity::None)
                .open()
                .expect("Failed to open port."),
        })
    }

    // Always use the precise variants!
    pub fn get_position_ra_dec(&mut self) -> Result<RADec, io::Error> {
        match self.port.write_all(b"e") {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to write to port: {:?}", e);
                return Err(e);
            }
        }

        let mut serial_buf: Vec<u8> = vec![0; 32];
        match self.port.read(serial_buf.as_mut_slice()) {
            Err(e) => {
                eprintln!("Failed to read from port: {:?}", e);
                Err(e)
            }
            Ok(0) => {
                eprintln!("No data found.");
                Err(io::Error::new(io::ErrorKind::TimedOut, "No data found."))
            }
            Ok(_n) => {
                println!("Data found: {:?}", serial_buf);
                Ok(RADec::from_msg(&serial_buf))
            }
        }
    }

    pub fn get_position_azm_alt(&mut self) -> Result<AzmAlt, io::Error> {
        match self.port.write_all(b"z") {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to write to port: {:?}", e);
                return Err(e);
            }
        }

        let mut serial_buf: Vec<u8> = vec![0; 32];
        match self.port.read(serial_buf.as_mut_slice()) {
            Err(e) => {
                eprintln!("Failed to read from port: {:?}", e);
                Err(e)
            }
            Ok(0) => {
                eprintln!("No data found.");
                Err(io::Error::new(io::ErrorKind::TimedOut, "No data found."))
            }
            Ok(_n) => {
                println!("Data found: {:?}", serial_buf);
                Ok(AzmAlt::from_msg(&serial_buf))
            }
        }
    }

    // Goto commands:
    // - AzmAlt will be relative to where it was powered on if not aligned.
    // - Ra/Dec will not work at all if not aligned.
    // degrees as f64 ==> position as i64 ==> Hex String
    pub fn goto_ra_dec(&mut self, mut coord: RADec) -> Result<(), io::Error> {
        println!("GOTO: RA: {:?} Dec: {:?}", coord.ra, coord.dec);
        println!(
            "GOTO: RA: {:?} Dec: {:?}",
            coord.absolute_ra(),
            coord.absolute_dec()
        );
        println!(
            "GOTO: RA: {:X} Dec: {:X}",
            coord.absolute_ra(),
            coord.absolute_dec()
        );

        match self
            .port
            .write_all(format!("r{:X},{:X}", coord.absolute_ra(), coord.absolute_dec()).as_bytes())
        {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Failed to write to port: {:?}", e);
                Err(e)
            }
        }
    }

    pub fn goto_azm_alt(&mut self, mut coord: AzmAlt) -> Result<(), io::Error> {
        match self
            .port
            .write_all(format!("r{:X},{:X}", coord.absolute_azm(), coord.absolute_alt()).as_bytes())
        {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Failed to write to port: {:?}", e);
                Err(e)
            }
        }
    }

    // Need further investigation.
    pub fn sync(&mut self, mut coord: RADec) -> Result<(), io::Error> {
        match self
            .port
            .write_all(format!("s{:X},{:X}", coord.absolute_ra(), coord.absolute_dec()).as_bytes())
        {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Failed to write to port: {:?}", e);
                Err(e)
            }
        }
    }

    // 0 = Off
    // 1 = Alt/Az
    // 2 = EQ North
    // 3 = EQ South
    pub fn get_tracking_mode(&mut self) -> Result<TrackingMode, io::Error> {
        match self.port.write_all(b"t") {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to write to port: {:?}", e);
                return Err(e);
            }
        }

        // This is how read is properly error handled.
        let mut serial_buf: Vec<u8> = vec![0; 32];
        match self.port.read(serial_buf.as_mut_slice()) {
            Err(e) => {
                eprintln!("Failed to read from port: {:?}", e);
                Err(e)
            }
            // Required to handle the case where read didn't read anything.
            Ok(0) => {
                eprintln!("No data found.");
                Err(io::Error::new(io::ErrorKind::TimedOut, "No data found."))
            }
            // Also required to read the exact number of bytes that were read in, even if we don't use it nor care.
            Ok(_n) => {
                println!("Data found: {:?}", serial_buf);
                match serial_buf[0] {
                    0 => Ok(TrackingMode::Off),
                    1 => Ok(TrackingMode::AltAz),
                    2 => Ok(TrackingMode::EQNorth),
                    3 => Ok(TrackingMode::EQSouth),
                    _ => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid tracking mode.",
                    )),
                }
            }
        }
    }

    pub fn set_tracking_mode(&mut self, mode: TrackingMode) -> Result<(), io::Error> {
        match self.port.write_all(format!("T{}", mode as u8).as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Failed to write to port: {:?}", e);
                Err(e)
            }
        }
    }

    pub fn slew_variable(
        &mut self,
        axis: SlewAxis,
        dir: SlewDir,
        rate: u16,
    ) -> Result<(), io::Error> {
        let axis_byte = match axis {
            SlewAxis::RAAzm => 16,
            SlewAxis::DecAlt => 17,
        };

        let dir_byte = match dir {
            SlewDir::Positive => 6,
            SlewDir::Negative => 7,
        };

        let rate_bytes = slew_rate(rate);

        match self.port.write_all(
            format!(
                "P{}{}{}{}{}{}{}",
                3, axis_byte, dir_byte, rate_bytes.0, rate_bytes.1, 0, 0
            )
            .as_bytes(),
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Failed to write to port: {:?}", e);
                Err(e)
            }
        }
    }

    pub fn slew_fixed(
        &mut self,
        axis: SlewAxis,
        dir: SlewDir,
        rate: SlewRate,
    ) -> Result<(), io::Error> {
        let axis_byte = match axis {
            SlewAxis::RAAzm => 16,
            SlewAxis::DecAlt => 17,
        };

        let dir_byte = match dir {
            SlewDir::Positive => 36,
            SlewDir::Negative => 37,
        };

        match self.port.write_all(
            format!(
                "P{}{}{}{}{}{}{}",
                2, axis_byte, dir_byte, rate as u8, 0, 0, 0
            )
            .as_bytes(),
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Failed to write to port: {:?}", e);
                Err(e)
            }
        }
    }

    fn get_location() {}

    fn set_location() {}

    fn get_time() {}

    fn set_time() {}

    fn gps_is_linked() {}

    fn gps_get_lat() {}

    fn gps_get_lon() {}

    fn gps_get_date() {}

    fn gps_get_year() {}

    fn gps_get_time() {}

    fn rtc_get_date() {}

    fn rtc_get_year() {}

    fn rtc_get_time() {}

    fn rtc_set_date() {}

    fn rtc_set_year() {}

    fn rtc_set_time() {}

    fn get_version() {}

    fn get_device_version() {}

    fn get_model() {}

    fn echo() {}

    fn is_aligned() {}

    fn goto_in_progress() {}

    fn cancel_goto() {}
}
