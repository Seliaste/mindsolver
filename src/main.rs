#![feature(portable_simd)]

extern crate colored;
extern crate ev3dev_lang_rust;
extern crate paris;

use std::path::Path;
use std::time::Duration;

use clap::Parser;
use ev3dev_lang_rust::Ev3Result;
use kewb::error::Error;
use kewb::fs::write_table;
use paris::{error, info, success};

use crate::cube::Cube;
use crate::hardware::*;

mod classification;
mod constants;
mod cube;
mod hardware;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// File source if using a previous scan file. Will skip scan
    #[clap(short, long)]
    file: Option<String>,

    /// Number of color sensor scans per facelet
    #[clap(short, long, default_value_t = 5)]
    iteration: usize,

    /// Movement between each color sensor scan
    #[clap(short, long, default_value_t = 8)]
    movement: i32,

    /// Sleep duration between each color sensor scan (in ms)
    #[clap(short, long, default_value_t = 20)]
    sleep: u32,

    /// Disables the solution application
    #[clap(short, long)]
    nosolve: bool,
}

fn create_cache() -> Result<(), Error> {
    if !Path::new("./cache_file").exists() {
        info!("Creating cache...");
        write_table("./cache_file")?;
    }
    Ok(())
}

fn main() -> Ev3Result<()> {
    if let Err(e) = create_cache() {
        error!("Could not create cache: {e}\nWill try to continue...")
    };
    let args = Args::parse();
    if args.nosolve && args.file.is_some() {
        // we can skip hardware initialisation
        no_hardware(args);
        return Ok(());
    }
    let mut hw = Hardware::init(
        Duration::from_millis(args.sleep as u64),
        args.movement,
        args.iteration,
    )?;
    let mut cube = Cube::init();
    info!("Resetting sensor arm...");
    hw.reset_sensor_position()?;
    if args.file.is_none() {
        info!("Starting cube scan.");
        hw.scan_cube(&mut cube)?;
        cube.export();
    } else {
        cube.import(args.file.unwrap())
            .expect("Could not load scan file");
    }
    let cube_notation = cube.to_notation();
    let fixed_notation = Cube::fixer(cube_notation);
    success!("Cube string is: {}", fixed_notation);
    let solution = Cube::solve(fixed_notation);
    info!("Solution is {}", solution);
    if !args.nosolve {
        for part in solution.get_all_moves() {
            hw.apply_solution_part(part.to_string(), &mut cube)?;
        }
        if hw.locked {
            hw.unlock_cube()?;
        }
        success!("Cube solved! I hope you enjoyed :D");
    }
    Ok(())
}

fn no_hardware(args: Args) {
    let mut cube = Cube::init();
    cube.import(args.file.unwrap())
        .expect("Could not load scan file");
    let cube_notation = cube.to_notation();
    let fixed_notation = Cube::fixer(cube_notation);
    success!("Cube string is: {}", fixed_notation);
    let solution = Cube::solve(fixed_notation);
    info!("Solution is {}", solution);
}
