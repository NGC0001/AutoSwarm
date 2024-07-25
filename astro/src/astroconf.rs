pub struct AstroConf {
    pub id: u32,
    pub uav_radius: f32,
    pub msg_range: f32,  // how far each UAV can transmit/receive its messages
                         // assumes same transmission and reception range
                         // assumes homogeneous swarm
    pub max_v: f32,  // how fast can UAV fly, assuming isotropic
}

impl AstroConf {
    pub fn validate(&self) -> Result<(), ()> {
        if self.uav_radius <= 0.0 {
            return Err(());
        }
        Ok(())
    }
}
