pub struct AstroConf {
    pub id: u32,  // starting from 1, 0 is reserved
    pub uav_radius: f32,
}

impl AstroConf {
    pub fn validate(&self) -> Result<(), ()> {
        if self.id == 0 {
            return Err(());
        }
        if self.uav_radius <= 0.0 {
            return Err(());
        }
        Ok(())
    }
}
