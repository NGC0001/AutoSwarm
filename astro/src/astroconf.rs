pub struct AstroConf {
    pub id: u32,
    pub uav_radius: f32,
}

impl AstroConf {
    pub fn validate(&self) -> Result<(), ()> {
        if self.uav_radius <= 0.0 {
            return Err(());
        }
        Ok(())
    }
}
