//! Utilities for working with volumetric data.

/// Normalize the value of a `u16` voxel to `u8`.
pub fn normalize(voxel: u16) -> u8 {
  let value = f32::from(voxel);
  let normalized = ((value / 4095.0) * 255.0).round();

  // We've normalized the voxel value to the range of u8 values
  // above, so it is now safe to "cast".
  unsafe { normalized.to_int_unchecked::<u8>() }
}

#[cfg(test)]
mod test_normalize {
  use super::*;

  #[test]
  fn zero() {
    assert_eq!(normalize(0), 0);
  }

  #[test]
  fn max() {
    assert_eq!(normalize(4095), 255);
  }

  #[test]
  fn mid() {
    assert_eq!(normalize(2048), 128);
  }
}

/// Create a `u16` voxel value from bytes.
///
/// # Notes
///
/// This function is small and should always end up being
/// inlined. Furthermore, since voxels are stored in little-endian
/// this should compile down to a no-op on LE machines.
pub fn voxel_from_bytes(byte0: u8, byte1: u8) -> u16 {
  u16::from_le_bytes([byte0, byte1])
}
