use serialport::{available_ports, SerialPort, SerialPortType};
use std::time::Duration;

struct AdvancedVX {
    port: Box<dyn SerialPort>,
}

impl AdvancedVX {
    fn new() -> AdvancedVX {
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
                .stop_bits(stop_bits::Stop1)
                .parity(parity::None)
                .open()
                .expect("Failed to open port"),
        }
    }

    // Always use the precise variants!
    fn get_position_ra_dec() {

    }

    fn get_position_azm_alt() {

    }

    // Goto commands: 
    // - AzmAlt will be relative to where it was powered on if not aligned.
    // - Ra/Dec will not work at all if not aligned.
    fn goto_ra_dec() {

    }
    
    fn goto_azm_alt() {

    }

    // Need further investigation.
    fn sync() {

    }

    fn get_tracking_mode() {

    }

    fn set_tracking_mode() {

    }

    // This will cover pos and neg, and will be Azm or RA depending on the mount.
    fn slew_fixed_horizontal() {
        
    }

    // This will cover pos and neg, and will be Azm or RA depending on the mount.
    fn slew_variable_horizontal() {
        
    }
    
    // This will cover pos and neg, and will be Alt or Dec depending on the mount.
    fn slew_fixed_vertical() {

    }

    // This will cover pos and neg, and will be Alt or Dec depending on the mount.
    fn slew_variable_vertical() {

    }

    fn get_location() {

    }

    fn set_location() {

    }

    fn get_time() {

    }

    fn set_time() {

    }

    fn gps_is_linked() {

    }

    fn gps_get_lat() {

    }

    fn gps_get_lon() {

    }

    fn gps_get_date() {

    }

    fn gps_get_year() {

    }

    fn gps_get_time() {

    }

    fn rtc_get_date() {

    }

    fn rtc_get_year() {

    }

    fn rtc_get_time() {

    }

    fn rtc_set_date() {

    }

    fn rtc_set_year() {

    }

    fn rtc_set_time() {

    }

    fn get_version() {

    }

    fn get_model() {

    }

    fn echo() {

    }

    fn is_aligned() {

    }

    fn goto_in_progress() {

    }

    fn cancel_goto() {

    }

}

fn main() {
    let telescope = AdvancedVX();


}
