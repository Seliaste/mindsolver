# Rust Rubik's Cube solver robot

## About

This is a software alternative for the [Mindcuber](http://mindcuber.com/)-style robots using the [ev3dev](https://www.ev3dev.org/) OS.

It uses statistical classification in order to achieve a better chance of success.

## How to run

> **IMPORTANT**: The program currently has hardcoded values for port positions.  
> Therefore, the required setup is: Sensor arm motor on port B, base motor on port C and flipper motor on port D.  
> Color sensor can be put on any input port.

> This guide assumes you want to cross-compile from a computer with good resources.  
> Compiling directly on the robot is possible but slow and unsupported

To build for the ev3dev robot, simply run:
```shell
cargo build --release
```  
The `--release` flag is optionnal but is recommended for slower compile time but faster execution time

Then copy your file on the robot with scp:
```shell
scp target/armv5te-unknown-linux-musleabi/release/rubiks-cube-robot-rust robot@YOUR_ROBOT_IP:/home/robot/rubiks-cube-robot-rust
```
Then, using ssh, simply run the executable using `brickrun`:
```shell
brickrun -r ./rubiks-cube-robot-rust
```

### Arguments

This software supports some command-line arguments to fine-tune your experience.
```text
    -f, --file <FILE>              File source if using a previous scan file. Will skip scan
    -h, --help                     Print help information
    -i, --iteration <ITERATION>    Number of color sensor scans per facelet [default: 5]
    -m, --movement <MOVEMENT>      Movement between each color sensor scan [default: 8]
    -n, --nosolve                  Disables the solution application
    -s, --sleep <SLEEP>            Sleep duration between each color sensor scan (in ms) [default 20]
    -V, --version                  Print version information
```

### Run without hardware

You might want to run the program on your own regular computer without hardware.  
In which case, if you use both `-f` and `-n`,
this program will skip hardware initialization.

For example, `cargo run --target x86_64-unknown-linux-gnu -- --file scan_test_files/solvable.txt --nosolve`

## What's left to do
- Option to disable saving of scans
- Check edges for duplicates or impossible pieces to try to fix them post-classification