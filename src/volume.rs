//! Handles data related to 3D volumetric data. The primary structure
//! is the [volume struct](Volume).

use crate::VolumeMd;
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

  /// Create an iterator over the voxels in a frame on the Z-axis.
  ///
  /// The returned iterator also produces the coordinates for each
  /// voxel value returned.
  ///
  /// # Notes
  ///
  /// Panics if `frame_index` is outside the range of frames.
  ///
  /// # Arguments
  ///
  /// * `frame_index` - The index of the frame on the Z-axis.
  ///
  /// # Returns
  ///
  /// An iterator over the voxels in the frame and their corresponding
  /// coordinates.
  pub fn zframe(&'d self, frame_index: usize) -> impl Iterator<Item = (u16, usize, usize)> + 'd {
    let xdim = self.metadata.xdim();

    self.zframe_bytes(frame_index).chunks(2).enumerate().map(move |(index, bytes)| {
      // Voxels are stored in little-endian. This should compile down
      // to a no-op on LE machines.
      let voxel = u16::from_le_bytes([bytes[0], bytes[1]]);
      (voxel, index % xdim, index / xdim)
    })
  }
}
