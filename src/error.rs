//! The primary error-type for the library.
//!
//! Usually this would be broken down into one error type per
//! module/structure and then collected here. However, since the
//! number of modules is small a single error type for the whole
//! library is workable.

use derive_more::{Display, From};
use derive_new::new;
use std::num::TryFromIntError;

/// Library error type.
#[derive(new, Display, From, Debug, PartialEq, Eq)]
#[display(fmt = "Medviz Error: {}")]
pub enum Err {
  /// Value is out of range.
  #[display(fmt = "Voxel value {} is out of the 0-4095 range.", _0)]
  VoxelValueOOR(u16),

  /// Found a DimSize key without any values.
  #[from(ignore)]
  #[display(fmt = "Metadata Line {}: Expecting values for `DimSize` key", line_number)]
  MdMissingDimSizeValues {
    /// The line number at which the error was found.
    line_number: usize,
  },

  /// One of the `DimSize` values is invalid.
  #[from(ignore)]
  #[display(fmt = "Metadata Line {}: Invalid value {} for dimension size", line_number, value)]
  MdInvalidDimSizeValue {
    /// The line number at which the error was found.
    line_number: usize,

    /// The invalid value.
    value: String,
  },

  /// A duplicate `DimSize` key was found.
  #[from(ignore)]
  #[display(fmt = "Metadata Line {}: Duplicated `DimSize` key", line_number)]
  MdDuplicateKey {
    /// The line number at which the error was found.
    line_number: usize,
  },

  /// Could not find a `DimSize` key.
  #[from(ignore)]
  #[display(fmt = "Invalid metadata, `DimSize` key not found")]
  MdDimSizeNotFound,

  /// Found too many values for `DimSize`.
  #[from(ignore)]
  #[display(fmt = "Line {}: Too many values for `DimSize` key", line_number)]
  MdTooManyDimSizeValues {
    /// The line number at which the error was found.
    line_number: usize,
  },

  /// Data size does not match metadata information.
  #[from(ignore)]
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
  #[from(ignore)]
  DataSizeUneven {
    /// The size of data in bytes.
    size: usize,
  },

  /// Dimension conversion errors.
  #[display(fmt = "Dimension conversion error: {}", _0)]
  DimConversion(TryFromIntError),
}
