extern crate rand;

use rand::Rng;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const EPOCH: u64 = 1735707600000; // 2025-01-01 00:00:00
const TIMESTAMP_SHIFT_OFFSET: u32 = 23;
const VERSION_SHIFT_OFFSET: u32 = 21;
const VERSION_MASK: u64 =  0b11 << VERSION_SHIFT_OFFSET;
const V1_VERSION: u64 = 0b01 << VERSION_SHIFT_OFFSET;
const V1_TYPE_SHIFT_OFFSET: u32 = 9;
const V1_TYPE_FLAG: u64 = 0b1 << 20;
const V1_TYPE_SIZE: u64 = 0b1111111;
const V1_TYPE_MASK: u64 = V1_TYPE_SIZE << V1_TYPE_SHIFT_OFFSET;
const V1_RANDOM_SPACE: u64 = 0xfffff;

const NIL: Smolid = Smolid { d: 0 };

pub struct Smolid {
    d: u64,
}

impl Smolid {
    pub fn as_u64(&self) -> u64 {
        self.d
    }

    pub fn timestamp(&self) -> SystemTime {
        let ms: u64 = (self.d >> TIMESTAMP_SHIFT_OFFSET) + EPOCH;
        UNIX_EPOCH + Duration::from_millis(ms)
    }

    pub fn nil() -> Smolid {
        NIL
    }

    pub fn is_nil(&self) -> bool {
        self.d == 0
    }

    pub fn version(&self) -> u16 {
        (self.d & VERSION_MASK >> VERSION_SHIFT_OFFSET) as u16
    }

    pub fn is_valid(&self) -> bool {
        self.version() == 1
    }

    pub fn new() -> Smolid {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        let mut n = (now - EPOCH) << TIMESTAMP_SHIFT_OFFSET;
        n |= V1_VERSION;
        n |= rand::rng().random_range(0..=V1_RANDOM_SPACE);

        Smolid { d: n }
    }

    pub fn new_with_type(typ: u16) -> Result<Smolid, String> {
        if typ > V1_TYPE_SIZE as u16 {
            return Err(format!("Type must be between 0 and {}", V1_TYPE_SIZE));
        }

        let s = Smolid::new();
        let mut n = s.d;

        n &= !V1_TYPE_MASK; // clear random data in type space
        n |= V1_TYPE_FLAG;
        n |= (typ as u64) << V1_TYPE_SHIFT_OFFSET;

        Ok(Smolid { d: n })
    }

    pub fn get_type(&self) -> Option<u16> {
        if self.d & V1_TYPE_FLAG == 0 {
            return None;
        }

        Some(((self.d & V1_TYPE_MASK) >> V1_TYPE_SHIFT_OFFSET) as u16)
    }

    pub fn is_of_type(&self, typ: u16) -> bool {
        self.get_type().unwrap() == typ
    }

    pub fn to_u64(&self) -> u64 {
        self.d
    }
}

impl FromStr for Smolid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_uppercase();
        let bytes = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &s)
            .ok_or_else(|| "Invalid base32 string".to_string())?;

        if bytes.len() != 8 {
            return Err(format!(
                "Invalid smolid length: expected 8 bytes, got {}",
                bytes.len()
            ));
        }

        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes);

        Ok(Smolid {
            d: u64::from_be_bytes(buf),
        })
    }
}

impl Display for Smolid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bytes = self.as_u64().to_be_bytes();
        let encoded = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &bytes);
        write!(f, "{}", encoded.to_lowercase())
    }
}
