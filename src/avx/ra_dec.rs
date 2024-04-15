pub struct RA_Dec<T> {
    pub ra: T,
    pub dec: T,
}

impl<T> RA_Dec<T> {
    pub fn new_f64(ra: f64, dec: f64) -> RA_Dec<f64> {
        RA_Dec {ra, dec}
    }

    pub fn new_i64(ra: i64, dec: i64) -> RA_Dec<i64> {
        RA_Dec {ra, dec}
    }
}