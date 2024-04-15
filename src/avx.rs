use serialport::{available_ports, SerialPort, SerialPortType};
use std::str;
use std::time::Duration;

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

pub struct AdvancedVX {
    port: Box<dyn SerialPort>,
}

impl Default for AdvancedVX {
    fn default() -> Self {
        Self::new()
    }
}

impl AdvancedVX {
    pub fn new() -> AdvancedVX {
        println!("Available ports:");

        let ports_info = serialport::available_ports().expect("No ports found!");

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

        if port_name.is_none() {
            println!("Device not found.");
            panic!("Device not found.");
        }

        AdvancedVX {
            // "Software drivers should be prepared to wait up to 3.5s (worst case scenario) for a hand control response."
            port: serialport::new(port_name.unwrap(), 9600)
                .timeout(Duration::from_millis(3500))
                .stop_bits(serialport::StopBits::One)
                .parity(serialport::Parity::None)
                .open()
                .expect("Failed to open port"),
        }
    }

    // Always use the precise variants!
    pub fn get_position_ra_dec(&mut self) -> Option<RADec> {
        self.port.write_all(b"e").expect("Failed to write to port");

        let mut serial_buf: Vec<u8> = vec![0; 32];
        match self
            .port
            .read(serial_buf.as_mut_slice())
            .expect("Found no data!")
        {
            0 => {
                println!("No data found.");

                None // Returns None Option.
            }
            _ => {
                println!("Data found: {:?}", serial_buf);

                Some(RADec::from_msg(&serial_buf))
            }
        }
    }

    fn get_position_azm_alt(&mut self) -> Option<AzmAlt> {
        self.port.write_all(b"z").expect("Failed to write to port");

        let mut serial_buf: Vec<u8> = vec![0; 32];
        match self
            .port
            .read(serial_buf.as_mut_slice())
            .expect("Found no data!")
        {
            0 => {
                println!("No data found.");

                None // Returns None Option.
            }
            _ => {
                println!("Data found: {:?}", serial_buf);

                Some(AzmAlt::from_msg(&serial_buf))
            }
        }
    }

    // Goto commands:
    // - AzmAlt will be relative to where it was powered on if not aligned.
    // - Ra/Dec will not work at all if not aligned.
    // degrees as f64 ==> position as i64 ==> Hex String
    pub fn goto_ra_dec(&mut self, mut coord: RADec) {
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

        self.port
            .write_all(format!("r{:X},{:X}", coord.absolute_ra(), coord.absolute_dec()).as_bytes())
            .expect("Failed to write to port");
    }

    fn goto_azm_alt(&mut self, mut coord: AzmAlt) {
        self.port
            .write_all(format!("r{:X},{:X}", coord.absolute_azm(), coord.absolute_alt()).as_bytes())
            .expect("Failed to write to port");
    }

    // Need further investigation.
    pub fn sync(&mut self, mut coord: RADec) {
        self.port
            .write_all(format!("s{:X},{:X}", coord.absolute_ra(), coord.absolute_dec()).as_bytes())
            .expect("Failed to write to port");
    }

    // 0 = Off
    // 1 = Alt/Az
    // 2 = EQ North
    // 3 = EQ South
    pub fn get_tracking_mode(&mut self) -> Option<TrackingMode> {
        self.port.write_all(b"t").expect("Failed to write to port");

        let mut serial_buf: Vec<u8> = vec![0; 32];
        match self
            .port
            .read(serial_buf.as_mut_slice())
            .expect("Found no data!")
        {
            0 => {
                println!("No data found.");

                None // Returns None Option.
            }
            _ => {
                println!("Data found: {:?}", serial_buf);

                match serial_buf[0] {
                    0 => Some(TrackingMode::Off),
                    1 => Some(TrackingMode::AltAz),
                    2 => Some(TrackingMode::EQNorth),
                    3 => Some(TrackingMode::EQSouth),
                    _ => None,
                }
            }
        }
    }

    pub fn set_tracking_mode(&mut self, mode: TrackingMode) {
        self.port
            .write_all(format!("T{}", mode as u8).as_bytes())
            .expect("Failed to write to port");
    }

    fn slew_variable(&mut self, axis: SlewAxis, dir: SlewDir, rate: u16) {
        let mut axis_byte = 0;
        let mut dir_byte = 0;

        match axis {
            SlewAxis::RAAzm => {
                axis_byte = 16;
            }
            SlewAxis::DecAlt => {
                axis_byte = 17;
            }
        }

        match dir {
            SlewDir::Positive => {
                dir_byte = 6;
            }
            SlewDir::Negative => {
                dir_byte = 7;
            }
        }

        let rate_bytes = slew_rate(rate);

        self.port
            .write_all(
                format!(
                    "P{}{}{}{}{}{}{}",
                    0x3, axis_byte, dir_byte, rate_bytes.0, rate_bytes.1, 0x0, 0x0
                )
                .as_bytes(),
            )
            .expect("Failed to write to port");
    }

    fn slew_fixed(&mut self) {
        
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

    fn get_model() {}

    fn echo() {}

    fn is_aligned() {}

    fn goto_in_progress() {}

    fn cancel_goto() {}
}
