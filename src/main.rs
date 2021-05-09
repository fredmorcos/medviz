#![warn(clippy::all)]

use bmp::{px, Image, Pixel};
use derive_more::{Display, From};
use derive_new::new;
use log::{debug, info, trace};
use medviz::{utils, Volume, VolumeErr, VolumeMd, VolumeMdErr};
use memmap::MmapOptions;
use std::convert::TryFrom;
use std::fmt;
use std::io;
use std::num::TryFromIntError;
use std::path::{Path, PathBuf};
use std::{fs::File, io::Read};
use structopt::StructOpt;

/// General top-level errors.
#[derive(new, From, Display)]
#[display(fmt = "{}")]
enum Err {
  /// IO Errors.
  #[display(fmt = "IO Error: {}", _0)]
  Io(io::Error),

  /// Errors when converting dimensions to different types.
  #[display(fmt = "Dimension Error: {}", _0)]
  Dimension(TryFromIntError),

  /// Errors from metadata loading and handling.
  #[display(fmt = "Metadata Error: {}", _0)]
  VolumeMd(VolumeMdErr),

  /// Errors from volume data handling.
  #[display(fmt = "Volume Data Error: {}", _0)]
  Volume(VolumeErr),
}

/// When returning an error from main(), this will print its Display
/// impl rather than its Debug impl.
impl fmt::Debug for Err {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    (self as &dyn fmt::Display).fmt(f)
  }
}

/// Extract slices from volumetric data.
#[derive(Debug, StructOpt)]
struct Opt {
  /// Verbose output (can be specified multiple times).
  #[structopt(short, long, parse(from_occurrences))]
  verbose: u8,

  /// Input: Metadata file.
  #[structopt(short, long, name = "metadata-file", parse(from_os_str))]
  metadata: PathBuf,

  /// Input: Volumetric data file.
  #[structopt(short, long, name = "data-file", parse(from_os_str))]
  data: PathBuf,

  /// Output: Y frame file (bmp).
  #[structopt(short, long, name = "y-frame-file", parse(from_os_str))]
  yfile: PathBuf,

  /// Output: Z frame file (bmp).
  #[structopt(short, long, name = "z-frame-file", parse(from_os_str))]
  zfile: PathBuf,
}

fn main() -> Result<(), Err> {
  let opt = Opt::from_args();

  let log_level = match opt.verbose {
    0 => log::LevelFilter::Warn,
    1 => log::LevelFilter::Info,
    2 => log::LevelFilter::Debug,
    _ => log::LevelFilter::Trace,
  };

  env_logger::Builder::new().filter_level(log_level).try_init().unwrap_or_else(|e| {
    eprintln!("Error initializing logger: {}", e);
  });

  info!("Informational output enabled.");
  debug!("Debug output enabled.");
  trace!("Tracing output enabled.");

  let mut metadata_contents = String::new();
  File::open(&opt.metadata)?.read_to_string(&mut metadata_contents)?;
  let metadata = VolumeMd::from_buffer(&metadata_contents)?;

  info!("Loaded metadata from {}", opt.metadata.display());
  info!("  X-dim = {}", metadata.xdim());
  info!("  Y-dim = {}", metadata.ydim());
  info!("  Z-dim = {}", metadata.zdim());

  let file = File::open(&opt.data)?;
  let map = unsafe { MmapOptions::new().map(&file)? };

  info!("Mapped {} bytes of data from {}", map.len(), opt.data.display());

  let volume = Volume::<u16>::from_slice(metadata, &map)?;

  // Produce the Y-frame, made up of voxels on the X- and Z-axis.
  produce_image(
    metadata.xdim(),
    metadata.zdim(),
    volume.yframe(metadata.ydim() / 2),
    &opt.yfile,
    "Y-frame",
  )?;

  // Produce the Z-frame, made up of voxels on the X- and Y-axis.
  produce_image(
    metadata.xdim(),
    metadata.ydim(),
    volume.zframe(metadata.zdim() / 2),
    &opt.zfile,
    "Z-frame",
  )?;

  Ok(())
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
/// * `filename` - The output bmp filename.
///
/// * `frame_name` - The frame name (for logging output).
///
/// # Returns
///
/// An error in case saving the bmp image failed.
fn produce_image(
  dim1: usize,
  dim2: usize,
  frame_iter: impl Iterator<Item = (u16, usize, usize)>,
  filename: &Path,
  frame_name: &'static str,
) -> Result<(), Err> {
  let dim1 = u32::try_from(dim1)?;
  let dim2 = u32::try_from(dim2)?;

  // This call is another linear run over the target image size to
  // initialize all pixels to a default value. It is avoidable if we
  // can stream the image data directly to file.
  let mut image = Image::new(dim1, dim2);

  for (voxel, x, y) in frame_iter {
    let x = u32::try_from(x)?;
    let y = u32::try_from(y)?;
    let normalized = utils::normalize(voxel);
    image.set_pixel(x, y, px!(normalized, normalized, normalized));
  }

  info!("Saving {} to {}", frame_name, filename.display());
  image.save(filename)?;

  Ok(())
}
