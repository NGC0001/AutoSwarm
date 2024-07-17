use std::time::Duration;

use astro::gps::Position;

pub const DEFAULT_POSITION_SEND_INTERVAL: Duration = Duration::from_millis(100);
pub const DEFAULT_MSG_OUT_DISTANCE: f32 = 200.0;  // m
pub const DEFAULT_UAV_RADIUS: f32 = 0.5;  // m

pub struct UavConf {
    pub id: u32,
    pub init_p: Position,
    pub p_send_intrvl: Duration,
    pub msg_out_distance: f32,  // how far away this UAV can transmit its messages
    pub radius: f32,
}

impl UavConf {
    pub fn new(id: u32, init_p: Position) -> UavConf {
        UavConf {
            id,
            init_p,
            p_send_intrvl: DEFAULT_POSITION_SEND_INTERVAL,
            msg_out_distance: DEFAULT_MSG_OUT_DISTANCE,
            radius: DEFAULT_UAV_RADIUS,
        }
    }
}