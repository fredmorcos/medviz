//! Handles data related to 3D volumetric data. The primary structure
//! is the [volume struct](Volume).

use crate::MedvizErr;
use crate::VolumeMd;
use crate::Voxel;

/// Volume data.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Volume<'d> {
  /// Metadata related to the volume.
  metadata: VolumeMd,

  /// Data related to the volume.
  data: &'d [u8],
}

impl<'d> Volume<'d> {
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
  pub fn from_slice(metadata: VolumeMd, data: &'d [u8]) -> Result<Self, MedvizErr> {
    let expected = metadata.xdim() * metadata.ydim() * metadata.zdim() * Voxel::size();

    if data.len() != expected {
      return Err(MedvizErr::new_data_size_mismatch(data.len(), expected));
    }

    if data.len() % Voxel::size() != 0 {
      return Err(MedvizErr::new_data_size_uneven(data.len()));
    }

    Ok(Self { metadata, data })
  }

  /// Return a slice of bytes of a frame on the Z-axis.
  fn zframe_bytes(&'d self, zframe_index: usize) -> &'d [u8] {
    // Size in bytes of a frame on the Z-axis.
    let zframe_size = self.metadata.zframe_len() * Voxel::size();
    let zframe_byte_index = zframe_size * zframe_index;
    &self.data[zframe_byte_index..zframe_byte_index + zframe_size]
  }

  /// Return an iterator of voxels of a frame on the Z-axis.
  fn zframe_iter(
    &'d self,
    zframe_index: usize,
  ) -> impl Iterator<Item = Result<Voxel, MedvizErr>> + 'd {
    self.zframe_bytes(zframe_index).chunks(Voxel::size()).map(|bytes| Voxel::from_slice(bytes))
  }

  /// Return a slice of bytes of a row on a frame on the Z-axis.
  fn zframe_row_bytes(&'d self, zframe_index: usize, row_index: usize) -> &'d [u8] {
    // Size in bytes of a row on a frame on the Z-axis.
    let row_size = self.metadata.xdim() * Voxel::size();
    let row_byte_index = row_size * row_index;
    let zframe = self.zframe_bytes(zframe_index);
    &zframe[row_byte_index..row_byte_index + row_size]
  }

  /// Return an iterator of voxels of a row on a frame on the Z-axis.
  fn zframe_row_iter(
    &'d self,
    zframe_index: usize,
    row_index: usize,
  ) -> impl Iterator<Item = Result<Voxel, MedvizErr>> + 'd {
    self
      .zframe_row_bytes(zframe_index, row_index)
      .chunks(Voxel::size())
      .map(|bytes| Voxel::from_slice(bytes))
  }

  /// Return a slice of bytes of a voxel on a frame on the Z-axis.
  fn zframe_voxel_bytes(&'d self, zframe_index: usize, x: usize, y: usize) -> &'d [u8] {
    let row = self.zframe_row_bytes(zframe_index, y);
    let voxel_byte_index = x * Voxel::size();
    &row[voxel_byte_index..voxel_byte_index + Voxel::size()]
  }

  /// Return an iterator of voxels of a column on a frame on the
  /// Z-axis.
  ///
  /// Note that columns are not contiguous in memory, which means they
  /// cannot be returned as a slice, making this the only function
  /// available to get the voxels of a column.
  fn zframe_col_iter(
    &'d self,
    frame_index: usize,
    col_index: usize,
  ) -> impl Iterator<Item = Result<Voxel, MedvizErr>> + 'd {
    (0..self.metadata.ydim()).map(move |row_index| {
      let bytes = self.zframe_voxel_bytes(frame_index, col_index, row_index);
      Voxel::from_slice(bytes)
    })
  }

  /// Create an iterator over the voxels in a frame on the X-axis.
  ///
  /// The returned iterator also produces the coordinates for each
  /// voxel value returned.
  ///
  /// # Notes
  ///
  /// Panics if `xframe_index` is outside the range of frames.
  ///
  /// # Arguments
  ///
  /// * `xframe_index` - The index of the frame on the X-axis.
  ///
  /// # Returns
  ///
  /// An iterator over the voxels in the frame and their corresponding
  /// coordinates.
  pub fn xframe(
    &'d self,
    xframe_index: usize,
  ) -> impl Iterator<Item = (Result<Voxel, MedvizErr>, usize, usize)> + 'd {
    // This works by going over every frame on the Z-axis. At each of
    // those frames, creates a "line" (iterator) out of the relevant
    // column.
    //
    // The .rev() call is used to start going over the frames on the
    // Z-axis in reverse, since that seems to produce the expected
    // result.
    //
    // This causes the creation of multiple iterators, one per
    // column. These iterators then need to be chained together by
    // flattening.
    //
    // The enumerate() iterator is useful to produce indexes from
    // which the coordinates will be calculated.
    //
    // The remaining step is to create the coordinates out of the
    // index and pack up each voxel and its coordinates in a
    // triple. Both of these are done in the final mapping.
    (0..self.metadata.zdim())
      .rev()
      .map(move |zframe_index| self.zframe_col_iter(zframe_index, xframe_index))
      .flatten()
      .enumerate()
      .map(move |(index, voxel)| {
        // `index` was produced by the call to .enumerate().
        let ydim = self.metadata.ydim();
        (voxel, index % ydim, index / ydim)
      })
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
  pub fn yframe(
    &'d self,
    yframe_index: usize,
  ) -> impl Iterator<Item = (Result<Voxel, MedvizErr>, usize, usize)> + 'd {
    // This works by going over every frame on the Z-axis. At each of
    // those frames, creates a "line" (iterator) out of the relevant
    // row.
    //
    // The .rev() call is used to start going over the frames on the
    // Z-axis in reverse, since that seems to produce the expected
    // result.
    //
    // This causes the creation of multiple iterators, one per
    // row. These iterators then need to be chained together by
    // flattening.
    //
    // The enumerate() iterator is useful to produce indexes from
    // which the coordinates will be calculated.
    //
    // The remaining step is to create the coordinates out of the
    // index and pack up each voxel and its coordinates in a
    // triple. Both of these are done in the final mapping.
    (0..self.metadata.zdim())
      .rev()
      .map(move |zframe_index| self.zframe_row_iter(zframe_index, yframe_index))
      .flatten()
      .enumerate()
      .map(move |(index, voxel)| {
        // `index` was produced by the call to .enumerate().
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
  pub fn zframe(
    &'d self,
    zframe_index: usize,
  ) -> impl Iterator<Item = (Result<Voxel, MedvizErr>, usize, usize)> + 'd {
    self.zframe_iter(zframe_index).enumerate().map(move |(index, voxel)| {
      // `index` was produced by the call to .enumerate().
      let xdim = self.metadata.xdim();
      (voxel, index % xdim, index / xdim)
    })
  }
}
