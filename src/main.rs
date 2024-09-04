#![warn(clippy::all)]

use clap::Parser;
use derive_more::{Display, From};
use derive_new::new;
use log::{debug, info, trace};
use medviz::utils;
use medviz::{MedvizErr, Volume, VolumeMd, Voxel};
use memmap::MmapOptions;
use std::io::{self, BufWriter};
use std::num::TryFromIntError;
use std::path::{Path, PathBuf};
use std::{fmt, io::Write};
use std::{fs::File, io::Read};

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

  /// Errors from the medviz library.
  #[display(fmt = "Library Error: {}", _0)]
  Medviz(MedvizErr),
}

/// When returning an error from main(), this will print its Display
/// impl rather than its Debug impl.
impl fmt::Debug for Err {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    (self as &dyn fmt::Display).fmt(f)
  }
}

/// Extract slices from volumetric data.
#[derive(Debug, clap::Parser)]
#[clap(author, version, about, long_about = None)]
struct Opt {
  /// Verbose output (can be specified multiple times).
  #[clap(short, long, action = clap::ArgAction::Count)]
  verbose: u8,

  /// Produce raw data instead of bmp images.
  #[structopt(short, long)]
  raw: bool,

  /// Input: Metadata file.
  #[clap(short, long, name = "metadata-file")]
  metadata: PathBuf,

  /// Input: Volumetric data file.
  #[clap(short, long, name = "data-file")]
  data: PathBuf,

  /// Output: X frame file (bmp).
  #[clap(short, long, name = "x-frame-file")]
  xfile: PathBuf,

  /// Output: Y frame file (bmp).
  #[clap(short, long, name = "y-frame-file")]
  yfile: PathBuf,

  /// Output: Z frame file (bmp).
  #[clap(short, long, name = "z-frame-file")]
  zfile: PathBuf,
}

fn main() -> Result<(), Err> {
  let opt = Opt::parse();

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

  let volume = Volume::from_slice(metadata, &map)?;

  if opt.raw {
    // Produce the X-frame, made up of voxels on the Y- and Z-axis.
    create_frame_raw("X-frame", &opt.xfile, volume.xframe(metadata.xdim() / 2))?;

    // Produce the Y-frame, made up of voxels on the X- and Z-axis.
    create_frame_raw("Y-frame", &opt.yfile, volume.yframe(metadata.ydim() / 2))?;

    // Produce the Z-frame, made up of voxels on the X- and Y-axis.
    create_frame_raw("Z-frame", &opt.zfile, volume.zframe(metadata.zdim() / 2))?;
  } else {
    // Produce the X-frame, made up of voxels on the Y- and Z-axis.
    create_frame_image(
      "X-frame",
      &opt.xfile,
      metadata.ydim(),
      metadata.zdim(),
      volume.xframe(metadata.xdim() / 2),
    )?;

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
  frame_iter: impl Iterator<Item = (Result<Voxel, MedvizErr>, usize, usize)>,
) -> Result<(), Err> {
  info!("Creating {} (raw) file at {}", frame_name, filename.display());
  let file = File::create(filename)?;
  let mut writer = BufWriter::new(file);

  info!("Writing {} (raw) to {}", frame_name, filename.display());
  for (voxel, _, _) in frame_iter {
    let voxel = voxel?;
    writer.write_all(&voxel.value().to_le_bytes())?;
  }

  Ok(())
}

/// Produce a bmp image file of the selected frame.
fn create_frame_image(
  frame_name: &'static str,
  filename: &Path,
  dim1: usize,
  dim2: usize,
  frame_iter: impl Iterator<Item = (Result<Voxel, MedvizErr>, usize, usize)>,
) -> Result<(), Err> {
  info!("Creating {} (bmp)", frame_name);
  let image = utils::frame_bmp(dim1, dim2, frame_iter)?;

  info!("Saving {} (bmp) to {}", frame_name, filename.display());
  image.save(filename)?;

  Ok(())
}
