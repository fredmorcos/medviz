//! Utilities for working with volumetric data.

use bmp::{px, Image, Pixel};
use std::convert::TryFrom;
use std::num::TryFromIntError;

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

/// Produce a bmp image out of a frame.
///
/// # Arguments
///
/// * `dim1` - The first dimension the frame is composed of.
///
/// * `dim2` - The second dimension the frame is composed of.
///
/// * `frame_iter` - The row-major iterator over frame voxels.
///
/// # Returns
///
/// An error in case conversions from `usize` to `u32` fail
/// (i.e. overflow).
pub fn frame_bmp(
  dim1: usize,
  dim2: usize,
  frame_iter: impl Iterator<Item = (u16, usize, usize)>,
) -> Result<Image, TryFromIntError> {
  let dim1 = u32::try_from(dim1)?;
  let dim2 = u32::try_from(dim2)?;

  // This call is another linear run over the target image size to
  // initialize all pixels to a default value. It is avoidable if we
  // can stream the image data directly to file.
  let mut image = Image::new(dim1, dim2);

  for (voxel, x, y) in frame_iter {
    let x = u32::try_from(x)?;
    let y = u32::try_from(y)?;
    let normalized = normalize(voxel);
    image.set_pixel(x, y, px!(normalized, normalized, normalized));
  }

  Ok(image)
}
