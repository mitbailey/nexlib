mod avx;
pub use avx::{AdvancedVX, RADec, AzmAlt, Device};

fn main () {
    println!("Main function called.");
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::{Duration, SystemTime, UNIX_EPOCH}};

    use chrono::{Utc, TimeZone};

    // mod avx;
    pub use crate::avx::AdvancedVX;
    use crate::{avx::RADec, Device};

    #[test]
    fn get_ra_dec() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        let pos = mount.get_position_ra_dec().expect("Failed to get position.");

        println!("{}, {}",pos.ra, pos.dec);
    }

    #[test]
    fn goto_ra_dec() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        
        mount.goto_ra_dec(RADec::new(138.7265968322754, 89.58314180374146)).expect("Failed to goto position.");
    }

    #[test]
    fn get_tracking_mode() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        
        let mode = mount.get_tracking_mode().unwrap();

        println!("{:?}", mode as u8);
    }

    #[test]
    fn get_model() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        
        let model = mount.get_model().unwrap();

        println!("{:?}", model);
    }

    #[test]
    fn get_device_version() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        
        // let version = mount.get_device_version(Device::AzmRaMotor).unwrap();

        println!("AzmRaMotor Version: {:?}", mount.get_device_version(Device::AzmRaMotor).unwrap());
        println!("AltDecMotor Version: {:?}", mount.get_device_version(Device::AltDecMotor).unwrap());
        println!("GpsUnit Version: {:?}", mount.get_device_version(Device::GpsUnit).unwrap());
        println!("RtcUnit Version: {:?}", mount.get_device_version(Device::RtcUnit).unwrap());
    }

    #[test] 
    fn get_version() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        
        let version = mount.get_version().unwrap();

        println!("{:?}", version);
    }

    #[test]
    fn get_gps_location() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        
        let location = mount.gps_get_location();

        match location {
            Ok(loc) => {
                println!("Latitude: {}", loc.0);
                println!("Longitude: {}", loc.1);
            },
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    #[test]
    fn rtc_get_datetime() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        
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
    fn rtc_set_datetime() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        
        let now = SystemTime::now();
        let (sec, nsec) = match now.duration_since(UNIX_EPOCH) {
            Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
            Err(e) => { // unlikely but should be handled
                let dur = e.duration();
                let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
                if nsec == 0 {
                    (-sec, 0)
                } else {
                    (-sec - 1, 1_000_000_000 - nsec)
                }
            },
        };
        let datetime = Utc.timestamp_opt(sec, nsec).unwrap();

        println!("Setting: {}", datetime.format("%Y-%m-%d %H:%M:%S"));

        mount.rtc_set_datetime(datetime).expect("Failed to set datetime.");
        sleep(Duration::from_secs(1));
        println!("Getting... ");
        let ndatetime = mount.rtc_get_datetime().expect("Failed to get datetime.");
        println!("Got: {}", ndatetime.format("%Y-%m-%d %H:%M:%S"));
        assert!((ndatetime - datetime) > chrono::Duration::seconds(1));
    }

    #[test]
    fn rtc_set_datetime_now() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        let datetime = Utc::now();
        mount.rtc_set_datetime_now().expect("Failed to set datetime.");
        sleep(Duration::from_secs(1));
        println!("Getting... ");
        let ndatetime = mount.rtc_get_datetime().expect("Failed to get datetime.");
        println!("Got: {}", ndatetime.format("%Y-%m-%d %H:%M:%S"));
        assert!(ndatetime - datetime > chrono::Duration::seconds(1));
    }

    #[test]
    fn rtc_get_time() {
        let mut mount = AdvancedVX::new().expect("Failed to connect to mount.");
        let time = mount.get_time().expect("Failed to get time.");
        println!("{}", time.format("%Y-%m-%d %H:%M:%S"));
    }
}