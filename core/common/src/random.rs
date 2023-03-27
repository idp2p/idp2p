use rand::{thread_rng, RngCore};

pub fn create_random<const N: usize>() -> [u8; N] {
    let mut key_data = [0u8; N];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    key_data
}
