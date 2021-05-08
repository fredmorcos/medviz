//! Handles data related to 3D volumetric data. The primary structure
//! is the [volume struct](Volume).

use crate::{utils, VolumeMd};
use derive_more::Display;
use derive_new::new;
use std::{marker::PhantomData, mem};

/// A voxel.
struct Voxel<T>(T);

impl<T> Voxel<T> {
  /// The size of a voxel.
  fn size() -> usize {
    mem::size_of::<T>()
  }
}

/// Volume-related error type.
#[derive(new, Display, Debug, PartialEq, Eq)]
#[display(fmt = "{}")]
pub enum Err {
  /// Data size does not match metadata information.
  #[display(
    fmt = "Data size of {} bytes does not match metadata: expecting {} bytes",
    actual,
    expected
  )]
  DataSizeMismatch {
    /// The size of data in bytes.
    actual: usize,

    /// The expected size in bytes based on metadata.
    expected: usize,
  },

  /// Data size is uneven.
  #[display(fmt = "Data size of {} bytes is uneven", size)]
  DataSizeUneven {
    /// The size of data in bytes.
    size: usize,
  },
}

/// Volume data.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Volume<'d, T> {
  /// Metadata related to the volume.
  metadata: VolumeMd,

  /// Data related to the volume.
  data: &'d [u8],

  /// Phantom data to use T.
  phantom: PhantomData<T>,
}

impl<'d, T> Volume<'d, T> {
  /// Create a [volume structure](Volume) from metadata and a byte
  /// buffer.
  ///
  /// # Arguments
  ///
  /// * `metadata` - Metadata related to the volume.
  ///
  /// * `data` - The byte buffer containing volumetric data.
  ///
  /// # Returns
  ///
  /// A [volume structure](Volume) or [an error](Err) in case the size
  /// of `data` does not match the expected size provided by
  /// `metadata`.
  pub fn from_slice(metadata: VolumeMd, data: &'d [u8]) -> Result<Self, Err> {
    let expected = metadata.xdim() * metadata.ydim() * metadata.zdim() * Voxel::<T>::size();

    if data.len() != expected {
      return Err(Err::new_data_size_mismatch(data.len(), expected));
    }

    if data.len() % Voxel::<T>::size() != 0 {
      return Err(Err::new_data_size_uneven(data.len()));
    }

    Ok(Self { metadata, data, phantom: PhantomData })
  }

  /// Return the slice of bytes that represents a frame on the Z-axis.
  fn zframe_bytes(&'d self, frame_index: usize) -> &'d [u8] {
    let zframe_bytes = self.metadata.zframe_len() * Voxel::<T>::size();
    let start_voxel_byte_index = zframe_bytes * frame_index;
    &self.data[start_voxel_byte_index..start_voxel_byte_index + zframe_bytes]
  }

  /// Return the slice of bytes that represents a row on a frame on
  /// the Z-axis.
  fn zframe_row_bytes(&'d self, frame_index: usize, row_index: usize) -> &'d [u8] {
    let row_bytes = self.metadata.xdim() * Voxel::<T>::size();
    let start_byte_index = row_bytes * row_index;
    let data = self.zframe_bytes(frame_index);
    &data[start_byte_index..start_byte_index + row_bytes]
  }

  /// Create an iterator over the voxels in a frame on the Y-axis.
  ///
  /// The returned iterator also produces the coordinates for each
  /// voxel value returned.
  ///
  /// # Notes
  ///
  /// Panics if `yframe_index` is outside the range of frames.
  ///
  /// # Arguments
  ///
  /// * `yframe_index` - The index of the frame on the Y-axis.
  ///
  /// # Returns
  ///
  /// An iterator over the voxels in the frame and their corresponding
  /// coordinates.
  pub fn yframe(&'d self, yframe_index: usize) -> impl Iterator<Item = (u16, usize, usize)> + 'd {
    // This works by going over every frame on the Z-axis. At each of
    // those frames, creates a slice out of the relevant row. The
    // chunking is there to later collect the pairs of u8 values into
    // u16 voxel values.
    //
    // The .rev() call is used to start going over the frames on the
    // Z-axis in reverse, since that seems to produce the better
    // (expected) result.
    //
    // This causes the creation of multiple iterators, one per
    // row. These iterators then need to be chained together, this is
    // done by flattening.
    //
    // The enumeration is useful to produce indexes from which the
    // coordinates will be calculated.
    //
    // The remaining step is to create the u16 voxel value and create
    // the coordinates out of the index. Both of these are done in the
    // final mapping.
    (0..self.metadata.zdim())
      .rev()
      .map(move |zframe_index| self.zframe_row_bytes(zframe_index, yframe_index).chunks(2))
      .flatten()
      .enumerate()
      .map(move |(index, bytes)| {
        // `index` was produced by the call to .enumerate(), and
        // `bytes` (a slice of 2 u8 elements) was produced by the call
        // to .chunks(2).
        let voxel = utils::voxel_from_bytes(bytes[0], bytes[1]);
        let xdim = self.metadata.xdim();
        (voxel, index % xdim, index / xdim)
      })
  }

  /// Create an iterator over the voxels in a frame on the Z-axis.
  ///
  /// The returned iterator also produces the coordinates for each
  /// voxel value returned.
  ///
  /// # Notes
  ///
  /// Panics if `zframe_index` is outside the range of frames.
  ///
  /// # Arguments
  ///
  /// * `zframe_index` - The index of the frame on the Z-axis.
  ///
  /// # Returns
  ///
  /// An iterator over the voxels in the frame and their corresponding
  /// coordinates.
  pub fn zframe(&'d self, zframe_index: usize) -> impl Iterator<Item = (u16, usize, usize)> + 'd {
    self.zframe_bytes(zframe_index).chunks(2).enumerate().map(move |(index, bytes)| {
      // Voxels are stored in little-endian. This should compile down
      // to a no-op on LE machines.
      let voxel = utils::voxel_from_bytes(bytes[0], bytes[1]);
      let xdim = self.metadata.xdim();
      (voxel, index % xdim, index / xdim)
    })
  }
}
