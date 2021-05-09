#![warn(clippy::all)]

use derive_more::{Display, From};
use derive_new::new;
use log::{debug, info, trace};
use medviz::{utils, Volume, VolumeErr, VolumeMd, VolumeMdErr};
use memmap::MmapOptions;
use std::io::{self, BufWriter};
use std::num::TryFromIntError;
use std::path::{Path, PathBuf};
use std::{fmt, io::Write};
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

  /// Produce raw data instead of bmp images.
  #[structopt(short, long)]
  raw: bool,

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

  if opt.raw {
    // Produce the Y-frame, made up of voxels on the X- and Z-axis.
    create_frame_raw("Y-frame", &opt.yfile, volume.yframe(metadata.ydim() / 2))?;

    // Produce the Z-frame, made up of voxels on the X- and Y-axis.
    create_frame_raw("Z-frame", &opt.zfile, volume.zframe(metadata.zdim() / 2))?;
  } else {
    // Produce the Y-frame, made up of voxels on the X- and Z-axis.
    create_frame_image(
      "Y-frame",
      &opt.yfile,
      metadata.xdim(),
      metadata.zdim(),
      volume.yframe(metadata.ydim() / 2),
    )?;

    // Produce the Z-frame, made up of voxels on the X- and Y-axis.
    create_frame_image(
      "Z-frame",
      &opt.zfile,
      metadata.xdim(),
      metadata.ydim(),
      volume.zframe(metadata.zdim() / 2),
    )?;
  }

  Ok(())
}

/// Produce a file with raw contents of the selected frame.
fn create_frame_raw(
  frame_name: &'static str,
  filename: &Path,
  frame_iter: impl Iterator<Item = (u16, usize, usize)>,
) -> Result<(), io::Error> {
  info!("Creating {} (raw) file at {}", frame_name, filename.display());
  let file = File::create(filename)?;
  let mut writer = BufWriter::new(file);

  info!("Writing {} (raw) to {}", frame_name, filename.display());
  for (voxel, _, _) in frame_iter {
    writer.write_all(&voxel.to_le_bytes())?;
  }

  Ok(())
}

/// Produce a bmp image file of the selected frame.
fn create_frame_image(
  frame_name: &'static str,
  filename: &Path,
  dim1: usize,
  dim2: usize,
  frame_iter: impl Iterator<Item = (u16, usize, usize)>,
) -> Result<(), Err> {
  info!("Creating {} (bmp)", frame_name);
  let image = utils::frame_bmp(dim1, dim2, frame_iter)?;

  info!("Saving {} (bmp) to {}", frame_name, filename.display());
  image.save(filename)?;

  Ok(())
}
