pub struct AstroConf {
    pub id: u32,
    pub uav_radius: f32,
    pub msg_range: f32,  // how far each uav can transmit/receive its messages.
                         // assumes same transmission and reception range.
                         // assumes homogeneous swarm.
    pub contact_range: f32,  // uavs within `contact_range` are considered safe to keep contact with.
                             // farther than this threshold risks losing contact.
                             // should be shorter than `msg_range`
    pub max_v: f32,  // how fast can uav fly, assuming isotropic
}

impl AstroConf {
    pub fn validate(&self) -> Result<(), ()> {
        if self.uav_radius <= 0.0 {
            return Err(());
        }
        Ok(())
    }
}
