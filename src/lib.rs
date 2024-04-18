mod mount;
pub use mount::{Mount, RADec, AzmAlt, Device};

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};
    use chrono::Utc;
    pub use crate::mount::Mount;
    use crate::{mount::RADec, AzmAlt, Device};

    #[test]
    fn nocon_basic_build() {
        let _mount = Mount::new().expect("Failed to connect to mount. Are you in WSL?");
    }

    #[test]
    fn nocon_get_ra_dec() {
        let mut mount = Mount::new().expect("Failed to connect to mount.");
        let _pos = mount.get_position_ra_dec().expect("Failed to get position.");
    }

    #[test]
    fn nocon_get_azm_alt() {
        let mut mount = Mount::new().expect("Failed to connect to mount.");
        let _pos = mount.get_position_azm_alt().expect("Failed to get position.");
    }

    #[test]
    fn nocon_goto_ra_dec() {
        let mut mount = Mount::new().expect("Failed to connect to mount.");
        let pos = mount.get_position_ra_dec().expect("Failed to get position.");
        mount.goto_ra_dec(RADec::new(pos.ra + 5., pos.dec + 5.)).expect("Failed to goto position.");
        while mount.goto_in_progress().expect("Failed to get goto in progress.") {
            sleep(Duration::from_secs(1));
        }
        // Verify that we are within 1 degree of the target
        let new_pos = mount.get_position_ra_dec().expect("Failed to get position.");
        assert!((new_pos.ra - 5.).abs() < 1.0);
        assert!((new_pos.dec - 5.).abs() < 1.0);
    }

    #[test]
    fn nocon_goto_azm_alt() {
        let mut mount = Mount::new().expect("Failed to connect to mount.");
        let pos = mount.get_position_azm_alt().expect("Failed to get position.");
        mount.goto_azm_alt(AzmAlt::new(pos.azm + 5., pos.alt + 5.)).expect("Failed to goto position.");
        while mount.goto_in_progress().expect("Failed to get goto in progress.") {
            sleep(Duration::from_secs(1));
        }
        // Verify that we are within 1 degree of the target
        let new_pos = mount.get_position_azm_alt().expect("Failed to get position.");
        assert!((new_pos.azm - 5.).abs() < 1.0);
        assert!((new_pos.alt - 5.).abs() < 1.0);
    }

    // Sync

    #[test]
    fn nocon_get_tracking_mode() {
        let mut mount = Mount::new().expect("Failed to connect to mount.");
        
        let mode = mount.get_tracking_mode().unwrap();

        println!("{:?}", mode as u8);
    }

    // Set tracking mode (verify!)

    // Slew variable (and wait til done!)

    // Slew fixed (and wait til done!)

    // Get location...
    // Set location... (verify!)

    // Get time

    // Set time (verify!)

    #[test]
    fn nocon_rtc_get_datetime() {
        let mut mount = Mount::new().expect("Failed to connect to mount.");
        
        let datetime = mount.rtc_get_datetime();

        match datetime {
            Ok(dt) => {
                println!("{}", dt.format("%Y-%m-%d %H:%M:%S"));
            },
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    #[test]
    fn nocon_rtc_set_datetime_now() {
        let mut mount = Mount::new().expect("Failed to connect to mount.");
        let datetime = Utc::now();
        mount.rtc_set_datetime_now().expect("Failed to set datetime.");
        sleep(Duration::from_secs(1));
        println!("Getting... ");
        let ndatetime = mount.rtc_get_datetime().expect("Failed to get datetime.");
        println!("Got: {}", ndatetime.format("%Y-%m-%d %H:%M:%S"));
        assert!(ndatetime - datetime > chrono::Duration::seconds(1));
    }

    #[test] 
    fn nocon_get_version() {
        let mut mount = Mount::new().expect("Failed to connect to mount.");
        
        let version = mount.get_version().unwrap();

        println!("{:?}", version);
    }

    #[test]
    fn nocon_get_device_version() {
        let mut mount = Mount::new().expect("Failed to connect to mount.");
        
        // let version = mount.get_device_version(Device::AzmRaMotor).unwrap();

        println!("AzmRaMotor Version: {:?}", mount.get_device_version(Device::AzmRaMotor).unwrap());
        println!("AltDecMotor Version: {:?}", mount.get_device_version(Device::AltDecMotor).unwrap());
        println!("GpsUnit Version: {:?}", mount.get_device_version(Device::GpsUnit).unwrap());
        println!("RtcUnit Version: {:?}", mount.get_device_version(Device::RtcUnit).unwrap());
    }
    
    #[test]
    fn nocon_get_model() {
        let mut mount = Mount::new().expect("Failed to connect to mount.");
        let _model = mount.get_model().unwrap();
    }

    // echo

    // is aligned

    // goto_in_progress

    // cancel goto (check if its still moving after a cancellation - measure amount of time it takes to stop?)
}