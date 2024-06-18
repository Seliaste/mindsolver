extern crate colored;
extern crate ev3dev_lang_rust;
extern crate paris;

use std::path::Path;
use std::thread::sleep;
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
    #[clap(long, default_value_t = 5)]
    iteration: usize,

    /// Movement between each color sensor scan
    #[clap(long, default_value_t = 8)]
    movement: i32,

    /// Sleep duration between each color sensor scan (in ms)
    #[clap(long, default_value_t = 20)]
    sleep: u32,

    /// Disables the solution application
    #[clap(short, long)]
    nosolve: bool,

    /// Enables saving scan to file
    #[clap(short, long)]
    save: bool,
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
    ctrlc::set_handler(move || {
        Hardware::shutdown().expect("Could not shutdown hardware");
        std::process::exit(0);
    }).expect("Could not define ctlr-c handler");
    let mut cube = Cube::init();

    info!("Resetting sensor arm...");
    hw.reset_sensor_position()?;

    if args.file.is_none() {
        info!("Starting cube scan.");
        hw.scan_cube(&mut cube)?;
        if args.save {
            cube.export();
        }
    } else {
        cube.import(args.file.unwrap())
            .expect("Could not load scan file");
    }

    let cube_notation = cube.to_notation();
    info!("Unfixed cube string is: {}", cube_notation);
    Cube::print_graphical(cube_notation.as_str());
    let (fixed_notation,steps) = Cube::bruteforce_fixer(cube_notation);
    success!("Cube string fixed in {steps} steps is: {}", fixed_notation);
    Cube::print_graphical(fixed_notation.as_str());

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
    sleep(Duration::from_secs(1)); // waiting for the flipper to stabilize
    Hardware::shutdown()?;
    Ok(())
}

fn no_hardware(args: Args) {
    let mut cube = Cube::init();
    cube.import(args.file.unwrap())
        .expect("Could not load scan file");
    let cube_notation = cube.to_notation();
    let (fixed_notation,steps) = Cube::bruteforce_fixer(cube_notation);
    success!("Cube string fixed in {steps} steps is: {}", fixed_notation);
    Cube::print_graphical(fixed_notation.as_str());
    // let solution = Cube::solve(fixed_notation);
    // info!("Solution is {}", solution);
}
