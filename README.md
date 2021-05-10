# `medviz`: Extract 2D images from raw 3D volumetric data

[Github Repository](https://github.com/fredmorcos/medviz)

## About

`medviz` is a library and a program for extracting 2D images (or
"slices") from raw 3D volumetric data.

## Usage

`medviz` has a simple to use command-line interface. Run `cargo run --
--help` or `medviz --help` after building and installing to get help.

### Examples

Produce BMP image files using very verbose logging and short option
names: `medviz -vvv -m tests/data/sinus.mhd -d tests/data/sinus.raw -z
z.bmp -y y.bmp -x x.bmp`

Produce RAW files using very verbose logging and long option names:
`medviz -vvv --metadata tests/data/sinus.mhd --data
tests/data/sinus.raw --zfile z.raw --yfile y.raw --xfile x.raw --raw`

## Installation

Cargo can be used to install `medviz` into `~/.cargo/bin`: `cargo
install --path .`

## Testing

To test the `medviz` library, you need to have the test files
available which are, by default, compressed using `zpaq` in an
`lrz-tar` archive and provided under `tests/data.tar.lrz`.

The reason for this is that Github only allows checked-in files up to
50MB. I previously used [Git LFS](https://git-lfs.github.com) to store
those files but immediately exhausted the provided large-file quota.

To extract the test files, `cd` into the `tests` directory and run
`lrzuntar data.tar.lrz`, you should end up with a `tests/data`
directory containing `.bmp` and `.raw` files.

Once the test data is available, execute `cargo test` in the top-level
source directory.

The test files are produced by a normal run of `medviz` and were
manually checked. The integration test checks the current execution
against these "model" files. I don't currently know a better way to
system test a tool like `medviz`.
