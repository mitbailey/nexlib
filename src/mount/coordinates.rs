use std::str;

const REV: i64 = 0x100000000;

// Unpacks the data from a message into a u8 array.
fn from_msg_to_i64(bytes: &[u8]) -> i64 {
    let as_str = str::from_utf8(bytes).unwrap();
    println!("String: {:?}", as_str);
    i64::from_str_radix(as_str, 16).unwrap()
}

// Converts Celestron integer angle format (from a message) to floating point degrees.
fn from_i64_to_deg(pos: i64) -> f64 {
    (pos as f64 / REV as f64) * 360.0
}

// Converts floating point degrees to transmittable Celestron integer angle format.
fn from_deg_to_i64(deg: f64) -> i64 {
    ((deg / 360.0) * REV as f64) as i64
}

pub struct RADec {
    pub ra: f64,
    pub dec: f64,
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

pub struct AzmAlt {
    pub azm: f64,
    pub alt: f64,
}

impl AzmAlt {
    pub fn new(azm: f64, alt: f64) -> AzmAlt {
        AzmAlt {azm, alt}
    }

    pub fn from_msg(msg: &[u8]) -> AzmAlt {
        AzmAlt::new(
            from_i64_to_deg(from_msg_to_i64(&msg[0..8])),
            from_i64_to_deg(from_msg_to_i64(&msg[9..=16])),
        )
    }

    pub fn azm_as_i64(&mut self) -> i64 {
        from_deg_to_i64(self.azm)
    }

    pub fn alt_as_i64(&mut self) -> i64 {
        from_deg_to_i64(self.alt)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic_build_azm_alt() {
        let azm_alt = super::AzmAlt::new(0.0, 0.0);
        assert_eq!(azm_alt.azm, 0.0);
        assert_eq!(azm_alt.alt, 0.0);
    }

    #[test]
    fn basic_build_ra_dec() {
        let ra_dec = super::RADec::new(0.0, 0.0);
        assert_eq!(ra_dec.ra, 0.0);
        assert_eq!(ra_dec.dec, 0.0);
    }
}