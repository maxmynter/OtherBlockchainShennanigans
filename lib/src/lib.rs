use serde::{Deserialize, Serialize};
use uint::construct_uint;

construct_uint! {
    #[derive(Serialize, Deserialize)]
    pub struct U256(4);
}
pub mod crypto;
pub mod sha256;
pub mod types;
pub mod util;

impl From<[u8; 32]> for U256 {
    fn from(bytes: [u8; 32]) -> Self {
        let mut slice = [0u64; 4];
        for i in 0..4 {
            let mut val = 0u64;
            for j in 0..8 {
                val = (val << 8) | (bytes[i * 8 + j] as u64)
            }
            slice[3 - i] = val;
        }
        U256(slice)
    }
}
