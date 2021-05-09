//! Handles voxels.

use crate::MedvizErr;
use std::mem;

/// A voxel.
#[derive(Clone, Copy)]
pub struct Voxel(u16);

impl Voxel {
  /// Create a voxel.
  ///
  /// Returns an error if the provided value is out of the 0-4095
  /// (12-bit) range.
  pub fn from(value: u16) -> Result<Self, MedvizErr> {
    // Voxels are actually 12-bits wide. Fail if the value is out of
    // range.
    if value > 4095 {
      return Err(MedvizErr::new_voxel_value_oor(value));
    }

    Ok(Self(value))
  }

  /// Create a voxel from an array of two bytes.
  ///
  /// Returns an error if the provided value is out of the 0-4095
  /// (12-bit) range.
  ///
  /// # Notes
  ///
  /// This function is small and should always end up being
  /// inlined. Furthermore, since voxels are stored in little-endian
  /// this should compile down to a no-op on LE machines.
  pub fn from_array(bytes: [u8; 2]) -> Result<Self, MedvizErr> {
    Self::from(u16::from_le_bytes(bytes))
  }

  /// Create a voxel from bytes.
  pub fn from_bytes(byte0: u8, byte1: u8) -> Result<Self, MedvizErr> {
    Self::from_array([byte0, byte1])
  }

  /// Create a voxel from a byteslice.
  ///
  /// # Notes
  ///
  /// Panics if slice does not contain at least 2 bytes.
  ///
  /// Returns an error if the provided value is out of the 0-4095
  /// (12-bit) range.
  pub fn from_slice(slice: &[u8]) -> Result<Self, MedvizErr> {
    Self::from_array([slice[0], slice[1]])
  }

  /// Return the value.
  pub fn value(&self) -> u16 {
    self.0
  }

  /// Return the normalized value of a voxel to `u8`.
  pub fn value_normalized(&self) -> u8 {
    const VOXEL_MAX: f32 = 4095.0;
    const VOXEL_NORMALIZED_MAX: f32 = 255.0;

    let value = f32::from(self.0);
    let normalized = ((value / VOXEL_MAX) * VOXEL_NORMALIZED_MAX).round();

    // We've normalized the voxel value to the range of u8 values
    // above, so it is now safe to "cast".
    unsafe { normalized.to_int_unchecked::<u8>() }
  }

  /// The size of a voxel.
  pub fn size() -> usize {
    mem::size_of::<u16>()
  }
}

#[cfg(test)]
mod voxel_tests {
  use super::Voxel;

  #[test]
  fn test_voxel_create() {
    Voxel::from(0).unwrap();
    Voxel::from(4095).unwrap();
  }

  #[test]
  #[should_panic]
  fn test_voxel_create_fail() {
    Voxel::from(4096).unwrap();
  }

  #[test]
  fn normalize_zero() {
    assert_eq!(Voxel::from(0).unwrap().value_normalized(), 0);
  }

  #[test]
  fn normalize_max() {
    assert_eq!(Voxel::from(4095).unwrap().value_normalized(), 255);
  }

  #[test]
  fn normalize_mid() {
    assert_eq!(Voxel::from(2048).unwrap().value_normalized(), 128);
  }
}
