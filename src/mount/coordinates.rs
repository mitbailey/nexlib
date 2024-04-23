use std::str;

const REV: i64 = 0x100000000;

/// Unpacks the data from a message into a u8 array.
fn from_msg_to_i64(bytes: &[u8]) -> i64 {
    let as_str = str::from_utf8(bytes).unwrap();
    println!("String: {:?}", as_str);
    i64::from_str_radix(as_str, 16).unwrap()
}

/// Converts Celestron integer angle format (from a message) to floating point degrees.
fn from_i64_to_deg(pos: i64) -> f64 {
    (pos as f64 / REV as f64) * 360.0
}

/// Converts floating point degrees to transmittable Celestron integer angle format.
fn from_deg_to_i64(deg: f64) -> i64 {
    ((deg / 360.0) * REV as f64) as i64
}

pub struct RADec {
    pub ra: f64,
    pub dec: f64,
}

impl std::fmt::Display for RADec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.ra, self.dec)
    }
}

impl RADec {
    pub fn new(ra: f64, dec: f64) -> RADec {
        RADec {ra, dec}
    }

    pub fn from_msg(msg: &[u8]) -> RADec {
        RADec::new(
            from_i64_to_deg(from_msg_to_i64(&msg[0..8])),
            from_i64_to_deg(from_msg_to_i64(&msg[9..=16])),
        )
    }

    pub fn ra_as_i64(&mut self) -> i64 {
        from_deg_to_i64(self.ra)
    }

    pub fn dec_as_i64(&mut self) -> i64 {
        from_deg_to_i64(self.dec)
    }
}

pub struct AzEl {
    pub az: f64,
    pub el: f64,
}

impl std::fmt::Display for AzEl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.az, self.el)
    }
}

impl AzEl {
    pub fn new(az: f64, el: f64) -> AzEl {
        AzEl {az, el}
    }

    pub fn from_msg(msg: &[u8]) -> AzEl {
        AzEl::new(
            from_i64_to_deg(from_msg_to_i64(&msg[0..8])),
            from_i64_to_deg(from_msg_to_i64(&msg[9..=16])),
        )
    }

    pub fn az_as_i64(&mut self) -> i64 {
        from_deg_to_i64(self.az)
    }

    pub fn el_as_i64(&mut self) -> i64 {
        from_deg_to_i64(self.el)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic_build_az_el() {
        let az_el = super::AzEl::new(0.0, 0.0);
        assert_eq!(az_el.az, 0.0);
        assert_eq!(az_el.el, 0.0);
    }

    #[test]
    fn basic_build_ra_dec() {
        let ra_dec = super::RADec::new(0.0, 0.0);
        assert_eq!(ra_dec.ra, 0.0);
        assert_eq!(ra_dec.dec, 0.0);
    }
}