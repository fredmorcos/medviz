#![warn(clippy::all)]

use derive_more::{Display, From};
use derive_new::new;
use log::{debug, info, trace};
use medviz::{VolumeMd, VolumeMdErr};
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::{fs::File, io::Read};
use structopt::StructOpt;

/// General top-level errors.
#[derive(new, From, Display)]
#[display(fmt = "{}")]
enum Err {
  /// IO Errors.
  #[display(fmt = "IO Error: {}", _0)]
  Io(io::Error),

  /// Errors from metadata loading and handling.
  #[display(fmt = "Metadata Error: {}", _0)]
  VolumeMd(VolumeMdErr),
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

  Ok(())
}
