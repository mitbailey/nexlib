use chrono::{DateTime, TimeZone, Utc};
use chrono::{Datelike, Timelike};
use serialport::{SerialPort, SerialPortType};
use std::error::Error;
use std::fmt::Display;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fmt, io};

mod coordinates;
pub use coordinates::{AzEl, RADec};

// const REV: i64 = 0x100000000;

/// Converts a slew rate in arcseconds/second to a mount-readable format.
/// The rate is multiplied by four and separated into a high and low byte.
fn slew_rate(rate: u16) -> (u8, u8) {
    (
        ((rate * 4) / 256).try_into().unwrap(),
        ((rate * 4) % 256).try_into().unwrap(),
    )
}

pub enum TrackingMode {
    Off = 0,
    AzEl = 1,
    EQNorth = 2,
    EQSouth = 3,
}

pub enum SlewAxis {
    RAAz = 0,
    DecEl = 1,
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

pub enum Model {
    GPSSeries = 1,
    ISeries = 3,
    ISeriesSe = 4,
    Cge = 5,
    AdvancedGT = 6,
    Slt = 7,
    Cpc = 9,
    Gt = 10,
    FourFiveSE = 11,
    SixEightSE = 12,
    Cgem = 14,
    AdvancedVX = 20,
    Evolution = 22,
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Model::GPSSeries => write!(f, "GPS Series"),
            Model::ISeries => write!(f, "i-Series"),
            Model::ISeriesSe => write!(f, "i-Series SE"),
            Model::Cge => write!(f, "CGE"),
            Model::AdvancedGT => write!(f, "Advanced GT"),
            Model::Slt => write!(f, "SLT"),
            Model::Cpc => write!(f, "CPC"),
            Model::Gt => write!(f, "GT"),
            Model::FourFiveSE => write!(f, "4/5 SE"),
            Model::SixEightSE => write!(f, "6/8 SE"),
            Model::Cgem => write!(f, "CGEM"),
            Model::AdvancedVX => write!(f, "Advanced VX"),
            Model::Evolution => write!(f, "Evolution"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Device {
    AzRaMotor = 16,
    ElDecMotor = 17,
    GpsUnit = 176,
    RtcUnit = 178,
}

#[derive(Debug, Copy, Clone)]
pub enum NonGpsDevice {
    AzRaMotor = 16,
    ElDecMotor = 17,
    RtcUnit = 178,
}

impl NonGpsDevice {
    fn as_device(&self) -> Device {
        match self {
            NonGpsDevice::AzRaMotor => Device::AzRaMotor,
            NonGpsDevice::ElDecMotor => Device::ElDecMotor,
            NonGpsDevice::RtcUnit => Device::RtcUnit,
        }
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Device::AzRaMotor => write!(f, "Azimuth/RA Motor"),
            Device::ElDecMotor => write!(f, "Elevation/Dec Motor"),
            Device::GpsUnit => write!(f, "GPS Unit"),
            Device::RtcUnit => write!(f, "RTC Unit"),
        }
    }
}

pub trait Mount {
    fn get_position_ra_dec(&mut self) -> Result<RADec, io::Error>;
    fn get_position_az_el(&mut self) -> Result<AzEl, io::Error>;
    fn goto_ra_dec(&mut self, coord: RADec) -> Result<(), io::Error>;
    fn goto_az_el(&mut self, coord: AzEl) -> Result<(), io::Error>;
    fn sync(&mut self, coord: RADec) -> Result<(), io::Error>;
    fn get_tracking_mode(&mut self) -> Result<TrackingMode, io::Error>;
    fn set_tracking_mode(&mut self, mode: TrackingMode) -> Result<(), io::Error>;
    fn slew_variable(&mut self, axis: SlewAxis, dir: SlewDir, rate: u16) -> Result<(), io::Error>;
    fn slew_fixed(&mut self, axis: SlewAxis, dir: SlewDir, rate: SlewRate)
        -> Result<(), io::Error>;
    fn get_location();
    fn set_location();
    fn get_time(&mut self) -> Result<DateTime<Utc>, io::Error>;
    fn set_time();
    fn get_version(&mut self) -> Result<String, Box<dyn std::error::Error>>;
    fn get_device_version(&mut self, device: NonGpsDevice) -> Result<String, Box<dyn Error>>;
    fn get_model(&mut self) -> Result<Model, io::Error>;
    fn echo();
    fn is_aligned(&mut self) -> Result<bool, io::Error>;
    fn goto_in_progress(&mut self) -> Result<bool, io::Error>;
    fn cancel_goto(&mut self) -> Result<(), io::Error>;
    fn stop_slew(&mut self, slew: SlewAxis) -> Result<(), io::Error>;

    /// Get GPS device
    fn get_gps(&mut self) -> Result<CelestronGps, io::Error>;
}

pub trait Gps {
    fn is_linked(&mut self) -> Result<bool, io::Error>;
    fn get_location(&mut self) -> Result<(f32, f32), io::Error>;
    fn get_datetime(&mut self) -> Result<DateTime<chrono::Utc>, io::Error>;
    fn get_device_version(&mut self) -> Result<String, Box<dyn Error>>;
}

pub trait Rtc {
    fn get_datetime(&mut self) -> Result<DateTime<chrono::Utc>, io::Error>;
    fn set_datetime_now(&mut self) -> Result<(), io::Error>;
}

/// The device which we control.
///
/// Orientates a telescope tube.
#[derive(Debug)]
pub struct CelestronMount {
    /// `port` should ONLY be accessed in `read_port` and `write_port`.
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    recv: [u8; 32],
}

pub struct CelestronGps<'a> {
    mount: &'a mut CelestronMount,
}

impl<'a> Gps for CelestronGps<'a> {
    // Not available on AVX.
    fn is_linked(&mut self) -> Result<bool, io::Error> {
        use Device::*;

        let res = self.mount.read_passthrough(GpsUnit, 55, 1)?;

        match res[0] {
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    // Not available on AVX.
    fn get_location(&mut self) -> Result<(f32, f32), io::Error> {
        use Device::*;

        if !self.is_linked()? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "GPS unit is not linked.",
            ));
        }

        let res = self.mount.read_passthrough(GpsUnit, 1, 3)?;
        let lat = (f32::from_be_bytes([0, res[0], res[1], res[2]]) / (0x1000000 as f32)) * 360.;
        let res = self.mount.read_passthrough(GpsUnit, 2, 3)?;
        let lon = (f32::from_be_bytes([0, res[0], res[1], res[2]]) / (0x1000000 as f32)) * 360.;

        Ok((lat, lon))
    }

    fn get_datetime(&mut self) -> Result<DateTime<chrono::Utc>, io::Error> {
        todo!();
    }
    
    /// Gets the version of the mount's firmware.
    fn get_device_version(&mut self) -> Result<String, Box<dyn Error>> {
        let res = self.mount.read_passthrough(Device::GpsUnit, 254, 2)?;
        Ok(format!("{}.{}", res[0], res[1]))
    }
}

/// Private functions for CelestronMount.
impl CelestronMount {
    /// Reads from a USB port and checks for the '#' character at the end of the message.
    ///
    /// The NexStar Communication Protocol requires a '#' at the end of each message sent by the mount.
    fn read_port(&mut self) -> Result<usize, io::Error> {
        let mut port = self.port.lock().unwrap();
        
        match port.read(&mut self.recv) {
            Ok(n) => {
                println!("RECEIVED (Ok): {:?}", &self.recv[..n]);
                if self.recv[n - 1] != b'#' {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("[{}:{}] Invalid data received: {:?}", file!(), line!(), self.recv),
                    ));
                }
                
                Ok(n)
            }
            Err(e) => {
                println!("RECEIVED (Err): {:?}", &self.recv);
                eprintln!(
                    "[{}:{}] Failed to read from port: {:?}",
                    file!(),
                    line!(),
                    e
                );
                Err(e)
            }
        }
    }

    fn write_port(&mut self, buf: &[u8]) -> Result<(), io::Error> {
        println!("TRANSMITTED: {:?}", buf);

        self.port.lock().unwrap().write_all(buf)?;
        
        // Ok, so.
        // This loop is necessary because when we send a command where we do not expect any data back, we do expect to receive a '#' back. Unfortunately, it doesn't seem to be sent immediately. So, we must wait here until we get some sort of response (and we should always get some response) before we can continue. Then, the calling function should always call self.read_port() to clear the buffer whether or not it actually wants to read the data. Typically, its 10 - 100 ms.
        while self.port.lock().unwrap().bytes_to_read()? == 0 {
            println!("Waiting for there to be bytes to read...");
            std::thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }

    /// Communicates through the hand controller to a device internal to the mount.
    ///
    /// Expects a response with data.
    fn read_passthrough(
        &mut self,
        dev: Device,
        cmd: u8,
        resp_len: usize,
    ) -> Result<&[u8], io::Error> {
        self.write_port(&[b'P', 1, dev as u8, cmd, 0, 0, 0, resp_len as u8])?;

        let len = self.read_port()?;
        if self.recv[len - 1] != b'#' {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("[{}:{}] Invalid data received: {:?}", file!(), line!(), self.recv),
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

    /// Communicates through the hand controller to a device internal to the mount.
    ///
    /// Expects a response with no data.
    fn write_passthrough(&mut self, dev: Device, cmd: u8, args: &[u8]) -> Result<(), io::Error> {
        // let mut port = self.port.lock().unwrap();

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
        
        // port.write_all(&cmd)?;
        self.write_port(&cmd)?;
        self.read_port()?; // Necessary to clear the buffer - we expect to get back a #.
        
        Ok(())
    }

    /// Communicates directly with the hand controller.
    ///
    /// Expects a response with data.
    fn read_handcontrol(&mut self, cmd: u8) -> Result<&[u8], io::Error> {
        self.write_port(&[cmd])?;

        let len = self.read_port()?;

        if self.recv[len - 1] != b'#' {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("[{}:{}] Invalid data received: {:?}", file!(), line!(), self.recv),
            ));
        }

        Ok(&self.recv[..len - 1])
    }

    /// Communicates directly with the hand controller.
    ///
    /// Expects a response with no data.
    fn write_handcontrol(&mut self, cmd: u8, args: &[u8]) -> Result<Vec<u8>, io::Error> {
        let mut cmd = vec![cmd];

        cmd.extend_from_slice(args);

        self.write_port(&cmd)?;
        self.read_port()?; // Necessary to clear the buffer - we expect to get back a #.

        cmd.clear();

        Ok(cmd)
    }
}

impl Default for CelestronMount {
    fn default() -> Self {
        Self::new().expect("Failed to create AdvancedVX object.")
    }
}

/// Public functions for Mount.
impl CelestronMount {
    pub fn new() -> Result<CelestronMount, io::Error> {
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

        Ok(CelestronMount {
            // "Software drivers should be prepared to wait up to 3.5s (worst case scenario) for a hand control response."
            port: Arc::new(Mutex::new(
                serialport::new(port_name.unwrap(), 9600)
                    .timeout(Duration::from_millis(3500)) // should be 3500 ms
                    .stop_bits(serialport::StopBits::One)
                    .parity(serialport::Parity::None)
                    .open()?,
            )),
            recv: [0; 32],
        })
    }
}

impl Mount for CelestronMount {
    /// Gets the current pointing position of the mount in right ascension and declination.
    ///
    /// Uses the high precision 24-bit NexStar coordinates.
    fn get_position_ra_dec(&mut self) -> Result<RADec, io::Error> {
        self.read_handcontrol(b'e')?;
        Ok(RADec::from_msg(&self.recv))
    }

    /// Gets the current pointing position of the mount in azimuth and elevation.
    ///
    /// Uses the precise 24-bit NexStar Get Position command.
    fn get_position_az_el(&mut self) -> Result<AzEl, io::Error> {
        self.read_handcontrol(b'z')?;
        Ok(AzEl::from_msg(&self.recv))
    }

    /// Moves the mount to a specified right ascension and declination.
    ///
    /// Uses the high precision 24-bit NexStar coordinates.
    ///
    /// Will not work if the mount is not aligned.
    fn goto_ra_dec(&mut self, mut coord: RADec) -> Result<(), io::Error> {
        self.write_handcontrol(
            b'r',
            format!("{:X},{:X}", coord.ra_as_i64(), coord.dec_as_i64()).as_bytes(),
        )?;
        Ok(())
    }

    /// Moves the mount to a specified azimuth and elevation.
    ///
    /// Uses the high precision 24-bit NexStar coordinates.
    ///
    /// Will be relative to where it was powered on if not aligned.
    fn goto_az_el(&mut self, mut coord: AzEl) -> Result<(), io::Error> {
        self.write_handcontrol(
            b'r',
            format!("{:X},{:X}", coord.az_as_i64(), coord.el_as_i64()).as_bytes(),
        )?;
        Ok(())
    }

    /// Sets the mount's current pointing to the passed coordinates.
    ///
    /// Uses the high precision 24-bit NexStar coordinates.
    ///
    /// Improves the pointing accuracy of future movements by assuming that this position is accurate.
    ///
    /// # Arguments
    ///
    /// * `coord` - The `RADec` coordinates to sync to; should be the expected coordinates of the object currently
    /// pointed at.
    fn sync(&mut self, mut coord: RADec) -> Result<(), io::Error> {
        self.write_handcontrol(
            b's',
            format!("{:X},{:X}", coord.ra_as_i64(), coord.dec_as_i64()).as_bytes(),
        )?;
        Ok(())
    }

    /// Gets the current tracking mode of the mount.
    fn get_tracking_mode(&mut self) -> Result<TrackingMode, io::Error> {
        self.read_handcontrol(b't')?;

        println!("Data found: {:?}", self.recv);
        match self.recv[0] {
            0 => Ok(TrackingMode::Off),
            1 => Ok(TrackingMode::AzEl),
            2 => Ok(TrackingMode::EQNorth),
            3 => Ok(TrackingMode::EQSouth),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid tracking mode.",
            )),
        }
    }

    /// Sets the tracking mode of the mount.
    fn set_tracking_mode(&mut self, mode: TrackingMode) -> Result<(), io::Error> {
        self.write_handcontrol(b'T', &[mode as u8])?;
        Ok(())
    }

    /// Begins a variable (user specified speed) slew movement.
    ///
    ///  # Arguments
    ///
    /// * `axis` - The axis to slew.
    /// * `dir` - The direction to slew.
    /// * `rate` - The rate of movement in arcseconds/second.
    fn slew_variable(&mut self, axis: SlewAxis, dir: SlewDir, rate: u16) -> Result<(), io::Error> {
        let device = match axis {
            SlewAxis::RAAz => Device::AzRaMotor,
            SlewAxis::DecEl => Device::ElDecMotor,
        };

        let dir_byte = match dir {
            SlewDir::Positive => 6,
            SlewDir::Negative => 7,
        };

        let rate_bytes = slew_rate(rate);

        self.write_passthrough(device, dir_byte, &[rate_bytes.0, rate_bytes.1])?;

        Ok(())
    }

    /// Begins a fixed (predefined speed) slew movement.
    ///
    /// # Arguments
    ///
    /// * `axis` - The axis to slew.
    /// * `dir` - The direction to slew.
    /// * `rate` - The rate of movement selected from the NexStar protocol's predefined speeds.
    fn slew_fixed(
        &mut self,
        axis: SlewAxis,
        dir: SlewDir,
        rate: SlewRate,
    ) -> Result<(), io::Error> {
        let device = match axis {
            SlewAxis::RAAz => Device::AzRaMotor,
            SlewAxis::DecEl => Device::ElDecMotor,
        };

        let dir_byte = match dir {
            SlewDir::Positive => 36,
            SlewDir::Negative => 37,
        };

        self.write_passthrough(device, dir_byte, &[rate as u8])?;
        Ok(())
    }

    fn get_location() {
        todo!();
    }

    fn set_location() {
        todo!();
    }

    /// Gets the current time from the mount.
    fn get_time(&mut self) -> Result<DateTime<Utc>, io::Error> {
        let res = self.read_handcontrol(b'h')?;

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

    /// Sets the current time on the mount.
    fn set_time() {
        todo!();
    }

    /// Gets the version of the hand controller's firmware.
    fn get_version(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let res = self.read_handcontrol(b'V')?;

        if res.len() != 2 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("[{}:{}] Invalid data received: {:?}", file!(), line!(), res),
            )));
        }

        Ok(format!("{}.{}", res[0], res[1]))
    }

    /// Gets the version of the mount's firmware.
    fn get_device_version(&mut self, device: NonGpsDevice) -> Result<String, Box<dyn Error>> {
        let res = self.read_passthrough(device.as_device(), 254, 2)?;
        Ok(format!("{}.{}", res[0], res[1]))
    }

    /// Gets the model of the mount.
    fn get_model(&mut self) -> Result<Model, io::Error> {
        let res = self.read_handcontrol(b'm')?;
        if res.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("[{}:{}] Invalid data received: {:?}", file!(), line!(), res),
            ));
        }

        match res[0] {
            1 => Ok(Model::GPSSeries),
            3 => Ok(Model::ISeries),
            4 => Ok(Model::ISeriesSe),
            5 => Ok(Model::Cge),
            6 => Ok(Model::AdvancedGT),
            7 => Ok(Model::Slt),
            9 => Ok(Model::Cpc),
            10 => Ok(Model::Gt),
            11 => Ok(Model::FourFiveSE),
            12 => Ok(Model::SixEightSE),
            14 => Ok(Model::Cgem),
            20 => Ok(Model::AdvancedVX),
            22 => Ok(Model::Evolution),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid model identifier.",
            )),
        }
    }

    /// Repeats back the message that was sent to it.
    fn echo() {
        unimplemented!();
    }

    /// Gets the mount's current alignment status.
    fn is_aligned(&mut self) -> Result<bool, io::Error> {
        let res = self.read_handcontrol(b'J')?;
        if res.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("[{}:{}] Invalid data received: {:?}", file!(), line!(), res),
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

    /// Determines if the mount is currently executing a goto command.
    fn goto_in_progress(&mut self) -> Result<bool, io::Error> {
        let res = self.read_handcontrol(b'L')?;
        if res.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("[{}:{}] Invalid data received: {:?}", file!(), line!(), res),
            ));
        }

        match res[0] {
            48 => Ok(false),
            49 => Ok(true),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid goto status: {} from {:?}.", res[0], res),
            )),
        }
    }

    /// Cancels the current goto in progress.
    fn cancel_goto(&mut self) -> Result<(), io::Error> {
        let res = self.read_handcontrol(b'Q')?;

        if res.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("[{}:{}] Invalid data received: {:?}", file!(), line!(), res),
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

    /// Get GPS device
    fn get_gps(&mut self) -> Result<CelestronGps, io::Error> {
        let model = self.get_model()?;

        match model {
            Model::GPSSeries => (),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No GPS device on model {}.", model),
                ))
            }
        }

        Ok(CelestronGps {
            mount: self,
        })
    }
    
    fn stop_slew(&mut self, axis: SlewAxis) -> Result<(), io::Error> {
        self.slew_variable(axis, SlewDir::Positive, 0)
    }
}

impl Rtc for CelestronMount {
    /// Gets the current date and time from the mount's real-time clock.
    fn get_datetime(&mut self) -> Result<DateTime<chrono::Utc>, io::Error> {
        use Device::*;

        let res = self.read_passthrough(RtcUnit, 3, 2)?;
        let mon = res[0];
        let day = res[1];
        let res = self.read_passthrough(RtcUnit, 4, 2)?;
        let year = u16::from_be_bytes([res[0], res[1]]);
        let res = self.read_passthrough(RtcUnit, 51, 3)?;
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

    /// Sets the current date and time on the mount's real-time clock.
    fn set_datetime_now(&mut self) -> Result<(), io::Error> {
        let now = chrono::Utc::now();

        use Device::*;

        self.write_passthrough(RtcUnit, 131, &[now.month() as u8, now.day() as u8])?;
        self.write_passthrough(RtcUnit, 132, &(now.year() as u16).to_be_bytes())?;

        let now = chrono::Utc::now();

        self.write_passthrough(
            RtcUnit,
            179,
            &[now.hour() as u8, now.minute() as u8, now.second() as u8],
        )
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Allows testing of private functions.
}
