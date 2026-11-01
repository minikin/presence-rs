#![no_std]

use presence_rs::Patch;
use serde::{Deserialize, Serialize};

/// A model whose patch is derived, serialized and applied without std or alloc.
#[derive(Patch, Serialize, Deserialize)]
#[patch(derive(Serialize, Deserialize))]
pub struct Sensor {
    /// The latest reading.
    pub reading: u32,
    /// A calibration offset, cleared with `null`.
    pub offset: Option<i16>,
}

/// Applies `patch`, so the generated `apply_to` is compiled too.
pub fn update(sensor: &mut Sensor, patch: SensorPatch) {
    patch.apply_to(sensor);
}
