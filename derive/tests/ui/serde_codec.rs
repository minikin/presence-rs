use presence_rs::Patch;
use serde::{Deserialize, Serialize};

mod codec {
    pub fn serialize<S: serde::Serializer>(value: &u8, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u8(*value)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u8, D::Error> {
        serde::Deserialize::deserialize(d)
    }
}

#[derive(Serialize, Deserialize)]
struct Extra {
    more: u8,
}

#[derive(Patch, Serialize, Deserialize)]
#[patch(derive(Serialize, Deserialize))]
struct Model {
    #[serde(with = "codec")]
    with: u8,
    #[serde(serialize_with = "codec::serialize")]
    serialize_with: u8,
    #[serde(deserialize_with = "codec::deserialize")]
    deserialize_with: u8,
    #[serde(flatten)]
    flatten: Extra,
    #[patch(skip)]
    #[serde(with = "codec")]
    skipped: u8,
}

fn main() {}
