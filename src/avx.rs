use chrono::{DateTime, TimeZone, Utc};
use chrono::{Datelike, Timelike};
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
use serialport::{SerialPort, SerialPortType};
use std::error::Error;
use std::fmt::Display;
use std::io::Read;
use std::time::Duration;
use std::{fmt, io, str};

mod coordinates;
pub use coordinates::{AzmAlt, RADec};

// const REV: i64 = 0x100000000;

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

#[derive(Debug, Copy, Clone)]
pub enum Device {
    AzmRaMotor = 16,
    AltDecMotor = 17,
    GpsUnit = 176,
    RtcUnit = 178,
}

impl Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Device::AzmRaMotor => write!(f, "Azimuth/RA Motor"),
            Device::AltDecMotor => write!(f, "Altitude/Dec Motor"),
            Device::GpsUnit => write!(f, "GPS Unit"),
            Device::RtcUnit => write!(f, "RTC Unit"),
        }
    }
}

#[derive(Debug)]
pub struct AdvancedVX {
    port: Box<dyn SerialPort>,
    recv: [u8; 32],
}

impl AdvancedVX {
    fn read_port(&mut self) -> Result<usize, io::Error> {
        match self.port.read(&mut self.recv) {
            Ok(n) => {
                if self.recv[n - 1] != b'#' {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid data received.",
                    ));
                }
                Ok(n)
            }
            Err(e) => {
                eprintln!("Failed to read from port: {:?}", e);
                Err(e)
            }
        }
    }

    fn read_device(&mut self, dev: Device, cmd: u8, resp_len: usize) -> Result<&[u8], io::Error> {
        self.port
            .write_all(&[b'P', 1, dev as u8, cmd, 0, 0, 0, resp_len as u8])?;
        let len = self.read_port()?;
        if self.recv[len - 1] != b'#' {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid data received.",
            ));
        }
        if len == resp_len + 1 {
            Ok(&self.recv[..resp_len])
        } else if len == resp_len + 2 {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                format!("Device {dev} is unavailable or command {cmd} is invalid."),
            ))
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid data length {len} on command {cmd:?} from device {dev}: expected {resp_len} bytes ({:?}).", self.recv)))
        }
    }

    fn write_device(&mut self, dev: Device, cmd: u8, args: &[u8]) -> Result<(), io::Error> {
        if args.len() > 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Command arguments must be 3 bytes or less. {:?}", args),
            ));
        }
        let mut cmd = [b'P', args.len() as u8 + 1, dev as u8, cmd, 0, 0, 0, 0];
        for (idx, arg) in args.iter().enumerate() {
            cmd[4 + idx] = *arg;
        }
        self.port.write_all(&cmd)?;
        match self.port.read(&mut cmd) {
            Ok(len) => {
                if cmd[len - 1] != b'#' {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Command {cmd:?} failed with error code {}", cmd[len - 1]),
                    ));
                }
            }
            Err(e) => {
                eprintln!("Failed to read from port: {:?}", e);
                return Err(e);
            }
        }
        Ok(())
    }

    fn read(&mut self, cmd: u8) -> Result<&[u8], io::Error> {
        self.port.write_all(&[cmd])?;
        let len = self.read_port()?;
        if self.recv[len - 1] != b'#' {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid data received.",
            ));
        }
        Ok(&self.recv[..len - 1])
    }

    fn write(&mut self, cmd: u8, args: &[u8]) -> Result<Vec<u8>, io::Error> {
        let mut cmd = vec![cmd];
        cmd.extend_from_slice(args);
        self.port.write_all(&cmd)?;
        cmd.clear();
        match self.port.read(&mut cmd) {
            Ok(len) => {
                if cmd[len - 1] != b'#' {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Command {cmd:?} failed with error code {}", cmd[len - 1]),
                    ));
                }
            }
            Err(e) => {
                eprintln!("Failed to read from port: {:?}", e);
                return Err(e);
            }
        }
        Ok(cmd)
    }
}

impl Default for AdvancedVX {
    fn default() -> Self {
        Self::new().expect("Failed to create AdvancedVX object.")
    }
}

impl AdvancedVX {
    pub fn new() -> Result<AdvancedVX, io::Error> {
        println!("Available ports:");

        let ports_info = serialport::available_ports()?;

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
                _ => {
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
                .open()?,
            recv: [0; 32],
        })
    }

    // Always use the precise variants!
    pub fn get_position_ra_dec(&mut self) -> Result<RADec, io::Error> {
        self.port.write_all(b"e")?;

        let mut serial_buf = [0; 32];
        self.read_port()?;

        Ok(RADec::from_msg(&serial_buf))
    }

    pub fn get_position_azm_alt(&mut self) -> Result<AzmAlt, io::Error> {
        self.port.write_all(b"z")?;

        self.read_port()?;

        Ok(AzmAlt::from_msg(&self.recv))
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
        self.port.write_all(b"t")?;

        self.read_port()?;

        println!("Data found: {:?}", self.recv);
        match self.recv[0] {
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

    pub fn set_tracking_mode(&mut self, mode: TrackingMode) -> Result<(), io::Error> {
        self.port.write_all(format!("T{}", mode as u8).as_bytes())?;
        Ok(())
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

        self.port.write_all(
            format!(
                "P{}{}{}{}{}{}{}",
                3, axis_byte, dir_byte, rate_bytes.0, rate_bytes.1, 0, 0
            )
            .as_bytes(),
        )?;

        Ok(())
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

        self.port.write_all(
            format!(
                "P{}{}{}{}{}{}{}",
                2, axis_byte, dir_byte, rate as u8, 0, 0, 0
            )
            .as_bytes(),
        )?;

        Ok(())
    }

    pub fn get_location() {
        todo!();
    }

    pub fn set_location() {
        todo!();
    }

    pub fn get_time(&mut self) -> Result<DateTime<Utc>, io::Error> {
        let res = self.read(b'h')?;
        if res.len() != 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid time data received: {:?}", res),
            ));
        }
        let hour = res[0];
        let min = res[1];
        let sec = res[2];
        let mon = res[3];
        let day = res[4];
        let year = res[5] as i32 + 2000;
        let ndst = res[7] == 1;
        let ofst = (i8::from_be_bytes([res[6]]) + if ndst { 0 } else { 1 }) as i32 * 100;
        let time = format!("{year}-{mon}-{day} {hour}:{min}:{sec} {ofst:+05}");
        let date = DateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S %z").map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse time {time}: {e:?}"),
            )
        })?;
        Ok(date.with_timezone(&chrono::Utc))
    }

    pub fn set_time() {
        todo!();
    }

    fn _gps_is_linked(&mut self) -> Result<bool, io::Error> {
        use Device::*;
        let res = self.read_device(GpsUnit, 55, 1)?;
        match res[0] {
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    fn _gps_get_location(&mut self) -> Result<(f32, f32), io::Error> {
        use Device::*;
        if !self._gps_is_linked()? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "GPS unit is not linked.",
            ));
        }
        let res = self.read_device(GpsUnit, 1, 3)?;
        let lat = (f32::from_be_bytes([0, res[0], res[1], res[2]]) / (0x1000000 as f32)) * 360.;
        let res = self.read_device(GpsUnit, 2, 3)?;
        let lon = (f32::from_be_bytes([0, res[0], res[1], res[2]]) / (0x1000000 as f32)) * 360.;
        Ok((lat, lon))
    }

    pub fn rtc_get_datetime(&mut self) -> Result<DateTime<chrono::Utc>, io::Error> {
        use Device::*;
        let res = self.read_device(RtcUnit, 3, 2)?;
        let mon = res[0];
        let day = res[1];
        let res = self.read_device(RtcUnit, 4, 2)?;
        let year = u16::from_be_bytes([res[0], res[1]]);
        let res = self.read_device(RtcUnit, 51, 3)?;
        let hour = res[0];
        let min = res[1];
        let sec = res[2];
        let res = chrono::Utc
            .with_ymd_and_hms(
                year as i32,
                mon.into(),
                day.into(),
                hour.into(),
                min.into(),
                sec.into(),
            )
            .unwrap();
        Ok(res)
    }

    pub fn rtc_set_datetime_now(&mut self) -> Result<(), io::Error> {
        let now = chrono::Utc::now();
        use Device::*;
        self.write_device(
            RtcUnit,
            131,
            &[now.month() as u8, now.day() as u8],
        )?;
        self.write_device(RtcUnit, 132, &(now.year() as u16).to_be_bytes())?;
        let now = chrono::Utc::now();
        self.write_device(
            RtcUnit,
            179,
            &[
                now.hour() as u8,
                now.minute() as u8,
                now.second() as u8,
            ],
        )
    }

    pub fn rtc_set_datetime(&mut self, datetime: DateTime<chrono::Utc>) -> Result<(), io::Error> {
        use Device::*;
        self.write_device(
            RtcUnit,
            131,
            &[datetime.month() as u8, datetime.day() as u8],
        )?;
        self.write_device(RtcUnit, 132, &(datetime.year() as u16).to_be_bytes())?;
        self.write_device(
            RtcUnit,
            179,
            &[
                datetime.hour() as u8,
                datetime.minute() as u8,
                datetime.second() as u8,
            ],
        )
    }

    pub fn get_version(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let res = self.read(b'V')?;

        if res.len() != 2 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid data received: {:?}", res),
            )));
        }
        Ok(format!("{}.{}", res[0], res[1]))
    }

    pub fn get_device_version(&mut self, device: Device) -> Result<String, Box<dyn Error>> {
        let res = self.read_device(device, 254, 2)?;

        Ok(format!("{}.{}", res[0], res[1]))
    }

    pub fn get_model(&mut self) -> Result<String, Box<dyn Error>> {
        let res = self.read(b'm')?;
        if res.len() != 1 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid data received.",
            )));
        }

        match res[0] {
            1 => Ok("GPS Series".to_string()),
            3 => Ok("i-Series".to_string()),
            4 => Ok("i-Series SE".to_string()),
            5 => Ok("CGE".to_string()),
            6 => Ok("Advanced GT".to_string()),
            7 => Ok("SLT".to_string()),
            9 => Ok("CPC".to_string()),
            10 => Ok("GT".to_string()),
            11 => Ok("4/5 SE".to_string()),
            12 => Ok("6/8 SE".to_string()),
            14 => Ok("CGEM".to_string()),
            20 => Ok("Advanced VX".to_string()),
            22 => Ok("Evolution".to_string()),
            _ => Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid model identifier.",
            ))),
        }
    }

    pub fn echo() {
        unimplemented!();
    }

    pub fn is_aligned(&mut self) -> Result<bool, io::Error> {
        let res = self.read(b'J')?;
        if res.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid data received.",
            ));
        }
        match res[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid goto status.",
            )),
        }
    }

    pub fn goto_in_progress(&mut self) -> Result<bool, io::Error> {
        let res = self.read(b'L')?;
        if res.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid data received.",
            ));
        }

        match res[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid goto status.",
            )),
        }
    }

    pub fn cancel_goto(&mut self) -> Result<(), io::Error> {
        let res = self.read(b'Q')?;

        if res.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid data received.",
            ));
        }

        match res[0] {
            0 => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid goto status.",
            )),
        }
    }
}
