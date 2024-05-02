#![feature(portable_simd)]

extern crate colored;
extern crate ev3dev_lang_rust;

use std::thread::sleep;
use std::time::Duration;

use colored::*;
use ev3dev_lang_rust::Ev3Result;
use crate::classification::Point;

use crate::cube::Cube;
use crate::hardware::*;
use crate::knn::Col;
use crate::utils::hsv_from_rgb;

mod hardware;
mod cube;
mod knn;
mod utils;
mod classification;

// We use https://github.com/muodov/kociemba for solving

fn sensor_scan(hw: &Hardware, data: &mut Cube) -> Ev3Result<()> {
    let sens_i32 = hw.color_sensor.get_rgb()?;
    let rgb = ((sens_i32.0 as f64 * 1.7) * (255. / 1020.)
               , sens_i32.1 as f64 * (255. / 1020.)
               , (sens_i32.2 as f64 * 1.875) * (255. / 1020.));
    println!("{}", format!("({},{},{})", rgb.0, rgb.1, rgb.2).truecolor(rgb.0 as u8, rgb.1 as u8, rgb.2 as u8));
    let idx = data.scan_order[data.curr_idx];
    let hsv = hsv_from_rgb(rgb);
    data.facelet_rgb_values[idx] = Point{x:hsv.0,y:hsv.1,z:hsv.2,index:idx};
    data.curr_idx += 1;
    Ok(())
}


fn apply_solution_part(part: String, hw: &Hardware, cube: &mut Cube) -> Ev3Result<()> {
    println!("Applying part {}", part);
    let face = part.chars().nth(0).unwrap();
    if !cube.next_faces.contains(&face) { // then we have to rotate
        hw.rot_base90()?;
        let tmp = cube.left_face;
        let tmp2 = cube.right_face;
        cube.left_face = cube.next_faces[3];
        cube.right_face = cube.next_faces[1];
        cube.next_faces[1] = tmp;
        cube.next_faces[3] = tmp2;
    }
    while cube.next_faces[0] != face {
        hw.flip_cube()?;
        cube.next_faces.rotate_left(1);
    }
    hw.lock_cube()?;
    if part.len() == 1 { // 90deg clockwise
        // We need to go a little further each time as the base borders are not the same width as the cube
        Hardware::run_for_rot(&hw.base_motor, -0.925)?;
        Hardware::run_for_rot(&hw.base_motor, 0.175)?;
    } else if part.ends_with('\'') { // 90 deg counterclockwise
        Hardware::run_for_rot(&hw.base_motor, 0.925)?;
        Hardware::run_for_rot(&hw.base_motor, -0.175)?;
    } else { // 180deg
        Hardware::run_for_rot(&hw.base_motor, 1.675)?;
        Hardware::run_for_rot(&hw.base_motor, -0.175)?;
    }
    hw.unlock_cube()?;
    return Ok(());
}

fn scan_face(hw: &Hardware, cube: &mut Cube) -> Ev3Result<()> {
    println!("Starting face scan");
    Hardware::run_for_deg(&hw.sensor_motor, -600)?;
    sensor_scan(hw, cube)?;
    Hardware::run_for_deg(&hw.sensor_motor, 90)?;
    for i in 0..4 {
        if i == 1 {
            Hardware::run_for_deg(&hw.sensor_motor, -20)?;
        }
        sensor_scan(hw, cube)?;
        hw.rot_base45()?;
        Hardware::run_for_deg(&hw.sensor_motor, 40)?;
        sensor_scan(hw, cube)?;
        hw.rot_base45()?;
        Hardware::run_for_deg(&hw.sensor_motor, -40)?;
        if i == 2 {
            Hardware::run_for_deg(&hw.sensor_motor, 20)?;
        }
    }
    hw.reset_sensor_position()?;
    println!("Face scan done");
    Ok(())
}

fn scan_cube(hw: &Hardware, cube: &mut Cube) -> Ev3Result<()> {
    for _ in 0..4 {
        // U,F,D,B scan
        hw.flip_cube()?;
        scan_face(hw, cube)?;
    }
    hw.rot_base90()?;
    hw.flip_cube()?;
    // R scan
    scan_face(hw, cube)?;
    hw.flip_cube()?;
    sleep(Duration::from_millis(100)); // waiting for the cube to fall before second rotation
    hw.flip_cube()?;
    // L scan
    scan_face(hw, cube)?;
    Ok(())
}

fn main() -> Ev3Result<()> {
    let hw = Hardware::init()?;
    let mut cube = Cube::init();
    hw.reset_sensor_position()?;
    scan_cube(&hw, &mut cube)?;
    println!("Color values: {:?} (size {})", cube.facelet_rgb_values, cube.facelet_rgb_values.len());
    println!("Cube string is: {}", cube.to_notation());
    let solution = cube.solve_cube();
    if solution.eq("Unsolvable cube!"){
        panic!("Error: {}",solution);
    }
    println!("Solution is {}", solution);
    for part in solution.split_whitespace() {
        apply_solution_part(part.to_owned(), &hw, &mut cube)?;
    }
    Ok(())
}
