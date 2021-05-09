//! Utilities for working with volumetric data.

use crate::MedvizErr;
use crate::Voxel;
use bmp::{px, Image, Pixel};
use std::convert::TryFrom;

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
  frame_iter: impl Iterator<Item = (Result<Voxel, MedvizErr>, usize, usize)>,
) -> Result<Image, MedvizErr> {
  let dim1 = u32::try_from(dim1)?;
  let dim2 = u32::try_from(dim2)?;

  // This call is another linear run over the target image size to
  // initialize all pixels to a default value. It is avoidable if we
  // can stream the image data directly to file.
  let mut image = Image::new(dim1, dim2);

  for (voxel, x, y) in frame_iter {
    let voxel = voxel?;
    let x = u32::try_from(x)?;
    let y = u32::try_from(y)?;
    let normalized = voxel.value_normalized();
    image.set_pixel(x, y, px!(normalized, normalized, normalized));
  }

  Ok(image)
}
