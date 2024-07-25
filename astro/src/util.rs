pub fn get_socket_name(id: u32) -> String {
    format!("socket_{:06}", id)
}

pub fn norm3d(x: f32, y: f32, z: f32) -> f32 {
    (x.powi(2) + y.powi(2) + z.powi(2)).sqrt()
}