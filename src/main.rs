mod avx;
pub use avx::{AdvancedVX, RADec, AzmAlt};

fn main () {
    println!("Main function called.");
}

#[cfg(test)]
mod tests {
    // mod avx;
    pub use crate::avx::AdvancedVX;
    use crate::avx::RADec;

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

}