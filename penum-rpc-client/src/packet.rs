use rand::RngCore;

pub const PACKET_SIZE: usize = 1024;
pub const HEADER_LEN: usize = 32;
pub const AEAD_TAG_LEN: usize = 16;

pub struct Packet;

impl Packet {
    pub fn new_random() -> [u8; PACKET_SIZE] {
        let mut data = [0u8; PACKET_SIZE];
        rand::thread_rng().fill_bytes(&mut data);
        data
    }
}
