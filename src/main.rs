#![feature(portable_simd)]

extern crate colored;
extern crate ev3dev_lang_rust;
extern crate paris;

use ev3dev_lang_rust::Ev3Result;
use paris::{error, info, success};

use crate::cube::Cube;
use crate::hardware::*;

mod classification;
mod cube;
mod hardware;

fn main() -> Ev3Result<()> {
    let mut hw = Hardware::init()?;
    let mut cube = Cube::init();
    info!("Resetting sensor arm...");
    hw.reset_sensor_position()?;
    success!("Sensor reset. Starting cube scan.");
    hw.scan_cube(&mut cube)?;
    cube.export();
    let cube_notation = cube.to_notation();
    success!("Cube string is: {}", cube_notation);
    let solution = cube.solve_cube(cube.to_notation());
    if solution.trim() == "Unsolvable cube!" {
        error!("Can't apply a solution: {}", solution);
        return Ok(());
    }
    info!("Solution is {}", solution);
    for part in solution.split_whitespace() {
        hw.apply_solution_part(part.to_owned(), &mut cube)?;
    }
    if hw.locked {
        hw.unlock_cube()?;
    }
    success!("Cube solved! I hope you enjoyed :D");
    Ok(())
}
