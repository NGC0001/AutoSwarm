pub fn get_socket_name(id: &u32) -> String {
    format!("socket_{:06}", id)
}