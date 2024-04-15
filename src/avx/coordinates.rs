use std::str;

const REV: i64 = 0x100000000;

fn bytes_to_int(bytes: &[u8]) -> i64 {
    let as_str = str::from_utf8(bytes).unwrap();
    println!("String: {:?}", as_str);
    i64::from_str_radix(as_str, 16).unwrap()
}

fn pos_int_to_deg(pos: i64) -> f64 {
    (pos as f64 / REV as f64) * 360.0
}

fn deg_to_pos_int(deg: f64) -> i64 {
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
            pos_int_to_deg(bytes_to_int(&msg[0..8])),
            pos_int_to_deg(bytes_to_int(&msg[9..=16])),
        )
    }

    pub fn absolute_ra(&mut self) -> i64 {
        deg_to_pos_int(self.ra)
    }

    pub fn absolute_dec(&mut self) -> i64 {
        deg_to_pos_int(self.dec)
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
            pos_int_to_deg(bytes_to_int(&msg[0..8])),
            pos_int_to_deg(bytes_to_int(&msg[9..=16])),
        )
    }

    pub fn absolute_azm(&mut self) -> i64 {
        deg_to_pos_int(self.azm)
    }

    pub fn absolute_alt(&mut self) -> i64 {
        deg_to_pos_int(self.alt)
    }
}


