#![warn(clippy::all)]

#[cfg(test)]
mod reference {
  use medviz::{utils, Volume, VolumeMd};
  use memmap::{Mmap, MmapOptions};
  use std::fs::File;
  use std::io::{BufWriter, Read, Write};
  use std::mem;

  fn read_file(filename: &str) -> Vec<u8> {
    let mut contents = Vec::new();
    let mut file = File::open(filename).unwrap();
    file.read_to_end(&mut contents).unwrap();
    contents
  }

  fn md_and_map() -> (VolumeMd, Mmap) {
    let metadata = VolumeMd::new(512, 512, 333);
    let file = File::open("tests/data/sinus.raw").unwrap();
    (metadata, unsafe { MmapOptions::new().map(&file).unwrap() })
  }

  #[test]
  fn raw_y() {
    let (metadata, map) = md_and_map();
    let volume = Volume::<u16>::from_slice(metadata, &map).unwrap();

    let expected = read_file("tests/data/y.raw");
    assert_eq!(expected.len(), metadata.xdim() * metadata.zdim() * mem::size_of::<u16>());

    let mut actual = Vec::new();
    {
      let mut writer = BufWriter::new(&mut actual);
      for (voxel, _, _) in volume.yframe(metadata.ydim() / 2) {
        writer.write_all(&voxel.to_le_bytes()).unwrap();
      }
    }

    assert_eq!(actual, expected);
  }

  #[test]
  fn raw_z() {
    let (metadata, map) = md_and_map();
    let volume = Volume::<u16>::from_slice(metadata, &map).unwrap();

    let expected = read_file("tests/data/z.raw");
    assert_eq!(expected.len(), metadata.xdim() * metadata.ydim() * mem::size_of::<u16>());

    let mut actual = Vec::new();
    {
      let mut writer = BufWriter::new(&mut actual);
      for (voxel, _, _) in volume.zframe(metadata.zdim() / 2) {
        writer.write_all(&voxel.to_le_bytes()).unwrap();
      }
    }

    assert_eq!(actual, expected);
  }

  #[test]
  fn bmp_y() {
    let (metadata, map) = md_and_map();
    let volume = Volume::<u16>::from_slice(metadata, &map).unwrap();

    let expected = read_file("tests/data/y.bmp");

    let mut actual = Vec::new();
    {
      let mut writer = BufWriter::new(&mut actual);
      utils::frame_bmp(metadata.xdim(), metadata.zdim(), volume.yframe(metadata.ydim() / 2))
        .unwrap()
        .to_writer(&mut writer)
        .unwrap();
    }

    assert_eq!(actual, expected);
  }

  #[test]
  fn bmp_z() {
    let (metadata, map) = md_and_map();
    let volume = Volume::<u16>::from_slice(metadata, &map).unwrap();

    let expected = read_file("tests/data/z.bmp");

    let mut actual = Vec::new();
    {
      let mut writer = BufWriter::new(&mut actual);
      utils::frame_bmp(metadata.xdim(), metadata.ydim(), volume.zframe(metadata.zdim() / 2))
        .unwrap()
        .to_writer(&mut writer)
        .unwrap();
    }

    assert_eq!(actual, expected);
  }
}
