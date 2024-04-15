mod avx;
pub use avx::{AdvancedVX, RADec, AzmAlt};

fn main() {
    let mount = avx::AdvancedVX::new();
    
}

#[cfg(test)]
mod tests {
    // mod avx;
    pub use crate::avx::AdvancedVX;
    use crate::avx::RADec;

    // #[test]
    // fn build() {
    //     let mount = AdvancedVX::new();
    // }

    // Example test
    #[test]
    fn get_ra_dec() {
        let mut mount = AdvancedVX::new();
        let pos = mount.get_position_ra_dec().unwrap();

        println!("{}, {}",pos.ra, pos.dec);
    }

    #[test]
    fn goto_ra_dec() {
        let mut mount = AdvancedVX::new();
        
        mount.goto_ra_dec(RADec::new(138.7265968322754, 89.58314180374146));
    }
    
    #[test]
    fn get_tracking_mode() {
        let mut mount = AdvancedVX::new();
        let mode = mount.get_tracking_mode().unwrap();

        println!("{:?}", mode as u8);
    }

}