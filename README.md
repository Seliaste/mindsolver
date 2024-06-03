# Mindsolver

## About

This is a software alternative for the [Mindcuber](http://mindcuber.com/)-style robots using the [ev3dev](https://www.ev3dev.org/) OS.

It uses statistical classification in order to achieve a better chance of success.

## How to run

> **IMPORTANT**: The program currently has hardcoded values for port positions.  
> Therefore, the required setup is: Sensor arm motor on port B, base motor on port C and flipper motor on port D.  
> Color sensor can be put on any input port.

> This guide assumes you want to cross-compile from a computer with good resources.  
> Compiling directly on the robot is possible but slow and unsupported

You first need the rust toolchain. This project has been tested for the 1.78.0 stable toolkit

To build mindsolver for the ev3dev robot, simply run:
```shell
cargo build --release
```  
The `--release` flag is optionnal but is recommended for slower compile time but faster execution time

Alternatively, download an executable from the [latest release](https://github.com/Seliaste/mindsolver/releases/)

Then copy your file on the robot with scp:
```shell
scp target/armv5te-unknown-linux-musleabi/release/mindsolver robot@YOUR_ROBOT_IP:/home/robot/mindsolver
```
Then, using ssh, simply run the executable using `brickrun`:
```shell
brickrun -r ./mindsolver
```

Note that on the first run, a cache file will be generated to have a faster solving. 
Scans will be saved in the scans directory

### Arguments

This software supports some command-line arguments to fine-tune your experience.
```text
Usage: mindsolver [OPTIONS]

Options:
  -f, --file <FILE>            File source if using a previous scan file. Will skip scan
      --iteration <ITERATION>  Number of color sensor scans per facelet [default: 5]
      --movement <MOVEMENT>    Movement between each color sensor scan [default: 8]
      --sleep <SLEEP>          Sleep duration between each color sensor scan (in ms) [default: 20]
  -n, --nosolve                Disables the solution application
  -s, --save                   Enables saving scan to file
  -h, --help                   Print help
  -V, --version                Print version
```

### Run without hardware

You might want to run the program on your own regular computer without hardware.  
In which case, if you use both `-f` and `-n`,
this program will skip hardware initialization.

For example, `cargo run --target x86_64-unknown-linux-gnu -- --file scan_test_files/solvable.txt --nosolve`
