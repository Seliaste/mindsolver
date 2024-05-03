#![feature(portable_simd)]

extern crate colored;
extern crate ev3dev_lang_rust;


use ev3dev_lang_rust::Ev3Result;

use crate::cube::Cube;
use crate::hardware::*;

mod hardware;
mod cube;
mod classification;

// We use https://github.com/muodov/kociemba for solving
fn main() -> Ev3Result<()> {
    let hw = Hardware::init()?;
    let mut cube = Cube::init();
    hw.reset_sensor_position()?;
    hw.scan_cube(&mut cube)?;
    println!("Cube string is: {}", cube.to_notation());
    let solution = cube.solve_cube();
    if solution.trim() == "Unsolvable cube!" {
        println!("Error: {}", solution);
        return Ok(());
    }
    println!("Solution is {}", solution);
    for part in solution.split_whitespace() {
        hw.apply_solution_part(part.to_owned(), &mut cube)?;
    }
    Ok(())
}
