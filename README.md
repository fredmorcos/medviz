# `medviz`: Extract 2D images from 3D volumetric data

[Github Repository](https://github.com/fredmorcos/medviz)

## About

`medviz` is a library and a program for extracting 2D images (or
"slices") from 3D volumetric data.

## Source code

When cloning this repository, you may also want to retrieve some
rather large test files. [Git LFS](https://git-lfs.github.com) is used
to store said files: `cd` into the source directory and use `git lfs
pull` to retrieve the files.

## Usage

`medviz` has a simple to use command-line interface. Run `cargo run --
--help` or `medviz --help` after building and installing to get help.

## Examples

TODO

## Installation

Cargo can be used to install `medviz` into `~/.cargo/bin`: `cargo
install --path .`

## Testing

To test the `medviz` library, execute `cargo test` in the top-level
source directory.
