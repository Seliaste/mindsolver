#![feature(portable_simd)]

extern crate colored;
extern crate ev3dev_lang_rust;
extern crate paris;

use ev3dev_lang_rust::Ev3Result;
use paris::{error, info, success};
use std::time::Duration;

use crate::cube::Cube;
use crate::hardware::*;

mod classification;
mod cube;
mod hardware;

use clap::Parser;
use kewb::{CubieCube, FaceCube, Solver};
use kewb::fs::read_table;

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

fn main() -> Ev3Result<()> {
    let args = Args::parse();
    if args.nosolve && args.file.is_some() {
        no_hardware(args);
        return Ok(());
    } // we can skip hardware initialisation
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
    success!("Cube string is: {}", cube_notation);
    let table = read_table("./cache_file").unwrap();
    let mut solver = Solver::new(&table, 23, Some(5.));
    let face_cube = FaceCube::try_from(cube_notation.as_str()).expect("Could not convert string to faces");
    let state = match CubieCube::try_from(&face_cube){
        Ok(x) => {if x.is_solvable() { x } else { error!("Cube not solvable: {:?}.", x); return Ok(()) }}
        Err(e) => {error!("Invalid cube: {:?}.", e); return Ok(())}
    };
    face_cube.
    let solution = solver.solve(state).expect("Could not solve cube");
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
    success!("Cube string is: {}", cube_notation);
    let table = read_table("./cache_file").unwrap();
    let mut solver = Solver::new(&table, 23, Some(5.));
    let face_cube = FaceCube::try_from(cube_notation.as_str()).expect("Could not convert string to faces");
    let state = match CubieCube::try_from(&face_cube){
        Ok(x) => {if x.is_solvable() { x } else { error!("Cube not solvable: {:?}.", x); return () }}
        Err(e) => {error!("Invalid cube: {:?}.", e); return ()}
    };
    let solution = solver.solve(state).expect("Could not solve cube");
    info!("Solution is {}", solution);
}
