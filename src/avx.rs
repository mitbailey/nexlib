use serialport::{available_ports, SerialPort, SerialPortType};
use std::str;
use std::time::Duration;

mod coordinates;
pub use coordinates::{Azm_Alt, RA_Dec};

const REV: i64 = 0x100000000;

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

    // fn bytes_to_int(&mut self, bytes: &[u8]) -> i64 {
    //     let as_str = str::from_utf8(bytes).unwrap();
    //     println!("String: {:?}", as_str);
    //     i64::from_str_radix(as_str, 16).unwrap()
    // }

    // fn pos_int_to_deg(&mut self, pos: i64) -> f64 {
    //     (pos as f64 / REV as f64) * 360.0
    // }

    // fn deg_to_pos_int(&mut self, deg: f64) -> i64 {
    //     ((deg / 360.0) * REV as f64) as i64
    // }

    // Always use the precise variants!
    pub fn get_position_ra_dec(&mut self) -> Option<RA_Dec> {
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

                // let coord = RA_Dec::<i64>::new_i64(
                //     self.bytes_to_int(&serial_buf[0..8]),
                //     self.bytes_to_int(&serial_buf[9..=16]),
                // );

                // Some(RA_Dec::<f64>::new_f64(
                //     self.pos_int_to_deg(coord.ra),
                //     self.pos_int_to_deg(coord.dec),
                // ))

                Some(RA_Dec::from_msg(&serial_buf))

                // println!("RA: {:?} Dec: {:?}", ra_val, dec_val);
                // println!("RA: {:?} Dec: {:?}", ra_deg, dec_deg);

                // Some(RA_Dec::<f64>::new_f64(ra_deg, dec_deg)) // Returns tuple.
            }
        }
    }

    fn get_position_azm_alt(&mut self) -> Option<Azm_Alt> {
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

                // let coord = Azm_Alt::<i64>::new_i64(
                //     self.bytes_to_int(&serial_buf[0..8]),
                //     self.bytes_to_int(&serial_buf[9..=16]),
                // );

                // Some(Azm_Alt::<f64>::new_f64(
                //     self.pos_int_to_deg(coord.azm),
                //     self.pos_int_to_deg(coord.alt),
                // ))

                Some(Azm_Alt::from_msg(&serial_buf))
            }
        }
    }

    // Goto commands:
    // - AzmAlt will be relative to where it was powered on if not aligned.
    // - Ra/Dec will not work at all if not aligned.
    // degrees as f64 ==> position as i64 ==> Hex String
    //
    pub fn goto_ra_dec(&mut self, mut coord: RA_Dec) {
        // println!("RA: {:?} Dec: {:?}", ra, dec);

        // let c = RA_Dec::<i64>::new_i64(
        //     self.deg_to_pos_int(coord.ra),
        //     self.deg_to_pos_int(coord.dec),
        // );

        println!("GOTO: RA: {:?} Dec: {:?}", coord.ra, coord.dec);
        println!("GOTO: RA: {:?} Dec: {:?}", coord.absolute_ra(), coord.absolute_dec());
        println!("GOTO: RA: {:X} Dec: {:X}", coord.absolute_ra(), coord.absolute_dec());

        self.port
            .write_all(format!("r{:X},{:X}", coord.absolute_ra(), coord.absolute_dec()).as_bytes())
            .expect("Failed to write to port");
    }

    fn goto_azm_alt(&mut self, mut coord: Azm_Alt) {
        // let c = Azm_Alt::<i64>::new_i64(
        //     self.deg_to_pos_int(coord.azm),
        //     self.deg_to_pos_int(coord.alt),
        // );

        self.port
            .write_all(format!("r{:X},{:X}", coord.absolute_azm(), coord.absolute_alt()).as_bytes())
            .expect("Failed to write to port");
    }

    // Need further investigation.
    fn sync(&mut self, mut coord: RA_Dec) {
        // let c = RA_Dec::<i64>::new_i64(
        //     self.deg_to_pos_int(coord.ra),
        //     self.deg_to_pos_int(coord.dec),
        // );

        self.port
            .write_all(format!("s{:X},{:X}", coord.absolute_ra(), coord.absolute_dec()).as_bytes())
            .expect("Failed to write to port");
    }

    fn get_tracking_mode() {}

    fn set_tracking_mode() {}

    // This will cover pos and neg, and will be Azm or RA depending on the mount.
    fn slew_fixed_horizontal() {}

    // This will cover pos and neg, and will be Azm or RA depending on the mount.
    fn slew_variable_horizontal() {}

    // This will cover pos and neg, and will be Alt or Dec depending on the mount.
    fn slew_fixed_vertical() {}

    // This will cover pos and neg, and will be Alt or Dec depending on the mount.
    fn slew_variable_vertical() {}

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
