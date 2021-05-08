#![warn(clippy::all)]
#![warn(missing_docs)]
// #![warn(missing_doc_code_examples)]

//! medviz is a library to help extract 2D images from 3D volumetric
//! data. "Images" can also be referred to as "frames" or more
//! commonly "slices". I will be using "frames" in documentation
//! instead of "slices" to avoid confusion when also discussing Rust's
//! slices.

pub mod metadata;
pub mod utils;
pub mod volume;

pub use metadata::Err as VolumeMdErr;
pub use metadata::VolumeMd;
pub use volume::Err as VolumeErr;
pub use volume::Volume;
