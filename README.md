## Screenshots

![IBM](/img/ibm-logo.png)
![Maze](/img/maze.png)
![Pong](/img/pong.png)
![Blinky](/img/blinky.png)

## Introduction

EMU-8 is a simple CHIP-8 interpreter written in Rust. This is my first Rust project and my first attempt at writing an emulator. The core functionality is in place however it does not currently feature sound support.

## Usage

### Prerequisites
Make sure you have Rust installed. If you don’t, you can install it via [rustup](https://rustup.rs/):

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Cloning the Repository
Clone this repository with:

```
git clone https://github.com/lukefidalgo/EMU-8
```

### Running a Pogram
To run a CHIP-8 program, use:

```
cargo run -- <file>
```

For example:

```
cargo run -- roms/IBM_Logo.ch8
```

## ROMS

You can find ROMs to play around with here:
[kripod/chip8-roms](https://github.com/kripod/chip8-roms)

## Resources

This emulator was implemented with the help of this fantastic article by [Tobias V. Langhoff](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/). If you ever want to write your own CHIP-8 emulator this is the resource to read.
