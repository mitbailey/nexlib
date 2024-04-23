mod mount;
pub use mount::{AzEl, CelestronMount, NonGpsDevice, RADec};

// TODO: Fix issue where the serial port always waits the 3.5 second timeout before returning the buffer, even when something has been read. Perhaps this has to do with the fact that the buffer hasn't been filled to capacity?

/// Tests prefixed with `nocon` require exclusive communication access to a mount and cannot be run concurrently. These tests should only be run using `cargo test nocon -- --test-threads=1`. If all tests are to be run, then `cargo test -- --test-threads=1` should be used since some will require exclusive access to the same hardware device.
#[cfg(test)]
mod tests {
    pub use crate::mount::CelestronMount;
    use crate::{
        mount::{Gps, Mount, RADec, Rtc, SlewAxis, SlewDir, SlewRate, TrackingMode},
        AzEl, NonGpsDevice,
    };
    use chrono::Utc;
    use std::{thread::sleep, time::Duration};

    const ERR_MSG_1: &str = "Failed to connect to mount. Are you in WSL?";

    #[test]
    fn nocon_basic_build() {
        let _mount = CelestronMount::new().expect(ERR_MSG_1);
    }

    #[test]
    fn nocon_get_gps_expect() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let _gps = mount.get_gps().expect("Failed to get GPS.");
    }

    #[test]
    #[should_panic]
    fn nocon_get_gps_panic() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let _gps = mount.get_gps().expect("Failed to get GPS.");
    }

    #[test]
    #[should_panic]
    fn nocon_gps_is_linked() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let mut gps = mount.get_gps().expect("Failed to get GPS.");
        gps.is_linked().expect("Failed to get GPS link status.");
    }

    #[test]
    #[should_panic]
    fn nocon_gps_get_location() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let mut gps = mount.get_gps().expect("Failed to get GPS.");
        gps.get_location().expect("Failed to get GPS link status.");
    }

    #[test]
    #[should_panic]
    fn nocon_gps_get_datetime() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let mut gps = mount.get_gps().expect("Failed to get GPS.");
        gps.get_datetime().expect("Failed to get GPS link status.");
    }

    #[test]
    fn nocon_get_ra_dec() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        println!("{}", pos);
    }

    #[test]
    fn nocon_get_az_el() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let _pos = mount.get_position_az_el().expect("Failed to get position.");
    }

    #[test]
    fn nocon_goto_ra_dec() {
        const DX: f64 = 1.0;
        const ACC: f64 = 0.5;

        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        mount
            .goto_ra_dec(RADec::new(pos.ra + DX, pos.dec + DX))
            .expect("Failed to goto position.");

        while mount
            .goto_in_progress()
            .expect("Failed to get goto in progress.")
        {
            sleep(Duration::from_secs(1));
        }

        // Verify that we are within 1 degree of the target
        let new_pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        assert!((new_pos.ra - (pos.ra + DX)).abs() < ACC, "RA: {} -> {}", pos.ra, new_pos.ra);
        assert!((new_pos.dec - (pos.dec + DX)).abs() < ACC, "Dec: {} -> {}", pos.dec, new_pos.dec);
    }

    #[test]
    fn nocon_get_goto_in_progress() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let _in_progress = mount
            .goto_in_progress()
            .expect("Failed to get goto in progress.");
    }

    #[test]
    fn nocon_goto_az_el() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let pos = mount.get_position_az_el().expect("Failed to get position.");
        mount
            .goto_az_el(AzEl::new(pos.az + 5., pos.el + 5.))
            .expect("Failed to goto position.");
        while mount
            .goto_in_progress()
            .expect("Failed to get goto in progress.")
        {
            sleep(Duration::from_secs(1));
        }
        // Verify that we are within 1 degree of the target
        let new_pos = mount.get_position_az_el().expect("Failed to get position.");
        assert!((new_pos.az - 5.).abs() < 1.0);
        assert!((new_pos.el - 5.).abs() < 1.0);
    }

    // Sync

    #[test]
    fn nocon_get_tracking_mode() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let mode = mount.get_tracking_mode().unwrap();

        println!("{:?}", mode as u8);
    }

    // Set tracking mode (verify!)

    // Slew variable (and wait til done!)
    #[test]
    fn nocon_slew_variable_decel() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        println!("Current position: {}", pos);

        mount
            .slew_variable(SlewAxis::DecEl, SlewDir::Positive, 1800)
            .expect("Failed to slew.");

        for _ in 0..3 {
            println!("Sleeping...");
            sleep(Duration::from_secs(1));
        }

        mount
            .slew_variable(SlewAxis::DecEl, SlewDir::Negative, 1800)
            .expect("Failed to slew.");

        for _ in 0..3 {
            println!("Sleeping...");
            sleep(Duration::from_secs(1));
        }

        mount.stop_slew(SlewAxis::RAAz).expect("Failed to stop slew.");

        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        println!("Current position: {}", pos);
    }

    #[test]
    fn nocon_slew_variable_raaz() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        println!("Current position: {}", pos);

        mount
            .slew_variable(SlewAxis::RAAz, SlewDir::Negative, 1800)
            .expect("Failed to slew.");

        for _ in 0..3 {
            println!("Sleeping...");
            sleep(Duration::from_secs(1));
        }

        mount
            .slew_variable(SlewAxis::RAAz, SlewDir::Positive, 1800)
            .expect("Failed to slew.");

        for _ in 0..3 {
            println!("Sleeping...");
            sleep(Duration::from_secs(1));
        }

        mount.stop_slew(SlewAxis::RAAz).expect("Failed to stop slew.");

        let pos = mount
            .get_position_ra_dec()
            .expect("Failed to get position.");

        println!("Current position: {}", pos);
    }

    // Slew fixed (and wait til done!)

    // Get location...
    // Set location... (verify!)

    // Get time

    // Set time (verify!)

    #[test]
    fn nocon_rtc_get_datetime() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let datetime = mount.get_datetime();

        match datetime {
            Ok(dt) => {
                println!("{}", dt.format("%Y-%m-%d %H:%M:%S"));
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    #[test]
    fn nocon_rtc_set_datetime_now() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let datetime = Utc::now();
        mount
            .set_datetime_now()
            .expect("Failed to set datetime.");
        sleep(Duration::from_secs(1));
        println!("Getting... ");
        let ndatetime = mount.get_datetime().expect("Failed to get datetime.");
        println!("Got: {}", ndatetime.format("%Y-%m-%d %H:%M:%S"));
        assert!(ndatetime - datetime > chrono::Duration::seconds(1));
    }

    #[test]
    fn nocon_get_version() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        let version = mount.get_version().unwrap();

        println!("{:?}", version);
    }

    #[test]
    fn nocon_get_device_version() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);

        // let version = mount.get_device_version(Device::AzRaMotor).unwrap();

        match mount.get_gps() {
            Ok(mut gps) => {
                println!("GPS: {:?}", gps.get_device_version().unwrap());
            }
            Err(e) => {
                println!("No GPS device found: {:?}", e);
            }
        }

        println!(
            "AzRaMotor Version: {:?}",
            mount.get_device_version(NonGpsDevice::AzRaMotor).unwrap()
        );
        println!(
            "ElDecMotor Version: {:?}",
            mount.get_device_version(NonGpsDevice::ElDecMotor).unwrap()
        );
        println!(
            "RtcUnit Version: {:?}",
            mount.get_device_version(NonGpsDevice::RtcUnit).unwrap()
        );
    }

    #[test]
    fn nocon_get_model() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        let _model = mount.get_model().unwrap();
    }

    #[test]
    fn nocon_set_tracking_mode_eqsouth() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        mount.set_tracking_mode(TrackingMode::EQNorth).unwrap();
    }

    #[test]
    fn nocon_set_tracking_mode_eqnorth() {
        let mut mount = CelestronMount::new().expect(ERR_MSG_1);
        mount.set_tracking_mode(TrackingMode::EQNorth).unwrap();
    }

    // echo

    // is aligned

    // goto_in_progress

    // cancel goto (check if its still moving after a cancellation - measure amount of time it takes to stop?)
}
