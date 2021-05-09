//! Handles metadata related to 3D volumetric data. The primary
//! structure is the [volume metadata struct](VolumeMd).

use crate::MedvizErr;
use atoi::FromRadix10Checked;
use derive_new::new;
use log::{debug, warn};

/// Volume metadata.
#[derive(new, Debug, PartialEq, Eq, Clone, Copy)]
pub struct VolumeMd {
  /// Number of voxels on the X-axis.
  xdim: usize,

  /// Number of voxels on the Y-axis.
  ydim: usize,

  /// Number of voxels on the Z-axis.
  zdim: usize,
}

impl VolumeMd {
  /// Load [volume metadata](VolumeMd) from a buffered reader.
  ///
  /// Finds the first line in the metadata with the `DimSize` key and
  /// loads the values for it.
  ///
  /// # Notes
  ///
  /// This is not a real parser. Instead, it works by splitting lines
  /// and tries to graciously handle invalid data.
  ///
  /// # Arguments
  ///
  /// * `reader` - A buffered reader of the input data.
  ///
  /// # Returns
  ///
  /// A populated [volume metadata structure](VolumeMd) or [an
  /// error](Err).
  pub fn from_buffer(buffer: &str) -> Result<Self, MedvizErr> {
    // The resulting VolumeMd. None if we haven't found a valid
    // `DimSize` entry and Some(...) if we have.
    let mut res = None;

    for (line_index, line) in buffer.split(|c| c == '\n').enumerate() {
      let line_number = line_index + 1;

      let mut entry = line.split('=');

      let key = match entry.next() {
        Some(key) => key.trim(),
        None => {
          warn!("Line {}: Skipping entry without an `=` sign", line_number);
          continue;
        }
      };

      if key.is_empty() {
        debug!("Line {}: Skipping empty line or line with empty key", line_number);
        continue;
      }

      if key != "DimSize" {
        debug!("Line {}: Skipping key {}", line_number, key);
        continue;
      }

      if res.is_some() {
        // We've already found a valid `DimSize` entry.
        return Err(MedvizErr::new_md_duplicate_key(line_number));
      }

      let value = match entry.next() {
        Some(value) => value.trim(),
        None => return Err(MedvizErr::new_md_missing_dim_size_values(line_number)),
      };

      let mut dims = value.split_whitespace();

      /// Read a dimension value from the dims iterator.
      ///
      /// # Uses
      ///
      /// * `dims` - The iterator over dimension values.
      ///
      /// * `line_number` - The current input line number for errors.
      ///
      /// # Returns
      ///
      /// The value read or exits the function with [an error](Err) in
      /// case there are no more values to be read.
      macro_rules! read_dimension_size {
        () => {
          match dims.next() {
            Some(dim) => dim,
            None => return Err(MedvizErr::new_md_missing_dim_size_values(line_number)),
          }
        };
      }

      let xdim_text = read_dimension_size!();
      let ydim_text = read_dimension_size!();
      let zdim_text = read_dimension_size!();

      if dims.next().is_some() {
        // There were more than 3 values provided.
        return Err(MedvizErr::new_md_too_many_dim_size_values(line_number));
      }

      /// Parse a dimension value from a string.
      ///
      /// # Arguments
      ///
      /// * `$text` - The input text to parse the value from.
      ///
      /// # Uses
      ///
      /// * `line_number` - The current input line number for errors.
      ///
      /// # Returns
      ///
      /// The value parsed or exits the function with [an error](Err)
      /// in case of invalid input or overflow.
      macro_rules! parse_dimension_size {
        ($text:ident) => {{
          let text: &str = $text;
          let (dim, rem) = usize::from_radix_10_checked(text.as_bytes());

          if rem == 0 {
            // The input was not a valid number.
            return Err(MedvizErr::new_md_invalid_dim_size_value(line_number, text.into()));
          }

          match dim {
            Some(dim) => dim,
            None => {
              // The input value would overflow usize.
              return Err(MedvizErr::new_md_invalid_dim_size_value(line_number, text.into()));
            }
          }
        }};
      }

      let xdim = parse_dimension_size!(xdim_text);
      let ydim = parse_dimension_size!(ydim_text);
      let zdim = parse_dimension_size!(zdim_text);

      res = Some(Self { xdim, ydim, zdim });
    }

    match res {
      Some(res) => Ok(res),
      None => Err(MedvizErr::new_md_dim_size_not_found()),
    }
  }

  /// Number of voxels in the X dimension.
  pub fn xdim(&self) -> usize {
    self.xdim
  }

  /// Number of voxels in the Y dimension.
  pub fn ydim(&self) -> usize {
    self.ydim
  }

  /// Number of voxels in the Z dimension.
  pub fn zdim(&self) -> usize {
    self.zdim
  }

  /// Number of voxels in a frame on the X-axis.
  pub fn xframe_len(&self) -> usize {
    self.ydim * self.zdim
  }

  /// Number of voxels in a frame on the Y-axis.
  pub fn yframe_len(&self) -> usize {
    self.xdim * self.zdim
  }

  /// Number of voxels in a frame on the Z-axis.
  pub fn zframe_len(&self) -> usize {
    self.xdim * self.ydim
  }
}

#[cfg(test)]
mod volume_metadata_tests {
  use super::VolumeMd;
  use crate::MedvizErr;

  #[test]
  fn from_reader_success() {
    let input = "\n\
                  NDims = 3\n\
                  DimSize = 512 512 333\n\
                  ElementSpacing = 0.402344 0.402344 0.899994\n";
    let metadata = VolumeMd::from_buffer(&input).unwrap();
    assert_eq!(metadata.xdim, 512);
    assert_eq!(metadata.ydim, 512);
    assert_eq!(metadata.zdim, 333);
  }

  #[test]
  fn from_reader_success_single() {
    let input = "DimSize = 512 512 333";
    let metadata = VolumeMd::from_buffer(&input).unwrap();
    assert_eq!(metadata.xdim, 512);
    assert_eq!(metadata.ydim, 512);
    assert_eq!(metadata.zdim, 333);
  }

  #[test]
  fn from_reader_fail_dimsize_values() {
    let input = "\n\
                  NDims = 3\n\
                  DimSize = \n\
                  ElementSpacing = 0.402344 0.402344 0.899994\n";
    let err = VolumeMd::from_buffer(&input);
    assert_eq!(err, Err(MedvizErr::MdMissingDimSizeValues { line_number: 3 }));
  }

  #[test]
  fn from_reader_fail_invalid_value() {
    let input = "\n\
                  NDims = 3\n\
                  DimSize = 512 512 abc\n\
                  ElementSpacing = 0.402344 0.402344 0.899994\n";
    let err = VolumeMd::from_buffer(&input);
    assert_eq!(
      err,
      Err(MedvizErr::MdInvalidDimSizeValue { line_number: 3, value: String::from("abc") })
    );
  }

  #[test]
  fn from_reader_fail_duplicate_key() {
    let input = "\n\
                  NDims = 3\n\
                  DimSize = 512 512 333\n\
                  ElementSpacing = 0.402344 0.402344 0.899994\n\
                  DimSize = 512 512 333\n";
    let err = VolumeMd::from_buffer(&input);
    assert_eq!(err, Err(MedvizErr::MdDuplicateKey { line_number: 5 }));
  }

  #[test]
  fn from_reader_fail_key_not_found() {
    let input = "\n\
                  NDims = 3\n\
                  ElementSpacing = 0.402344 0.402344 0.899994\n";
    let err = VolumeMd::from_buffer(&input);
    assert_eq!(err, Err(MedvizErr::MdDimSizeNotFound));
  }

  #[test]
  fn from_reader_fail_too_many_values() {
    let input = "\n\
                  NDims = 3\n\
                  DimSize = 512 512 333 333\n\
                  ElementSpacing = 0.402344 0.402344 0.899994\n";
    let err = VolumeMd::from_buffer(&input);
    assert_eq!(err, Err(MedvizErr::MdTooManyDimSizeValues { line_number: 3 }));
  }

  #[test]
  fn from_reader_fail_empty() {
    let input = "";
    let err = VolumeMd::from_buffer(&input);
    assert_eq!(err, Err(MedvizErr::MdDimSizeNotFound));
  }

  #[test]
  fn from_reader_fail_newlines() {
    let input = "    \n\
                  \n\
                  \n";
    let err = VolumeMd::from_buffer(&input);
    assert_eq!(err, Err(MedvizErr::MdDimSizeNotFound));
  }
}
