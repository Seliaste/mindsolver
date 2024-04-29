#![feature(portable_simd)]
extern crate ev3dev_lang_rust;
extern crate colored;
extern crate nabo;

use std::collections::HashMap;
use std::iter;
use std::thread::sleep;
use std::time::Duration;
use colored::*;

use ev3dev_lang_rust::Ev3Result;
use ev3dev_lang_rust::motors::{MotorPort, TachoMotor};
use ev3dev_lang_rust::sensors::ColorSensor;
use std::process::Command;
use nabo::*;

// We use https://github.com/muodov/kociemba for solving

#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct Col([NotNan<f64>;3],char);

impl Point<f64> for Col {
    const DIM: u32 = 3;
    fn set(&mut self, index: u32, value: NotNan<f64>) {
        self.0[index as usize] = value;
    }
    fn get(&self, index: u32) -> NotNan<f64> {
        self.0[index as usize]
    }
}

struct Hardware {
    base_motor: TachoMotor,
    flipper_motor: TachoMotor,
    sensor_motor: TachoMotor,
    color_sensor: ColorSensor
}

struct Data {
    // The scan order will always be the same, 
    // so insted of complicated code it's better to hardcode it
    scan_order : Vec<usize>,
    // Current facelet number 
    curr_idx : usize,
    // Stores RGB values in the order of the standard notatio
    facelet_rgb_values: Vec<Col>,
    next_faces: [char;4], // Faces that can be accessed by simply flipping. First one is the one currently down
    // right and left from the sensor POV
    right_face: char, 
    left_face: char,
}

impl Data {
    pub fn init() -> Self {
        Self {
            // NOTE: THIS NEEDS TO BE VERIFIED, I PROBABLY MADE A MISTAKE
            scan_order : vec![4,7,8,5,2,1,0,3,6, // U
            22,25,26,23,20,19,18,21,24, // F
            31,34,35,32,29,28,27,30,33, // D
            49,52,53,50,47,46,45,48,51,// B
            13,16,17,14,11,10,9,12,15, // R
            40,37,36,39,42,43,44,41,38],// L
            curr_idx : 0,
        facelet_rgb_values: iter::repeat(Col([NotNan::new(0.).unwrap(),NotNan::new(0.).unwrap(),NotNan::new(0.).unwrap()],' ')).take(54).collect(),
            next_faces: ['R','F','L','B'],
            right_face: 'D',
            left_face: 'U'
        }
    }
}

fn hsv_from_rgb(rgb:(f64,f64,f64)) -> (f64,f64,f64){
        let r = rgb.0 / 255.0;
        let g = rgb.1 / 255.0;
        let b = rgb.2 / 255.0;

        let min = r.min(g.min(b));
        let max = r.max(g.max(b));
        let delta = max - min;

        let v = max;
        let s = match max > 1e-3 {
            true => delta / max,
            false => 0.0,
        };
        let h = match delta == 0.0 {
            true => 0.0,
            false => {
                if r == max {
                    (g - b) / delta
                } else if g == max {
                    2.0 + (b - r) / delta
                } else {
                    4.0 + (r - g) / delta
                }
            }
        };
        let h2 = ((h * 60.0) + 360.0) % 360.0;

        (h2, s, v)
}


fn run_for_deg(motor: &TachoMotor, degree: i32)  -> Ev3Result<()> {
    let count = motor.get_count_per_rot()? as f64/360.*degree as f64;
    motor.run_to_rel_pos(Some(count as i32))?;
    motor.wait_until_not_moving(None);
    sleep(Duration::from_millis(20));
    Ok(())
}

fn run_for_rot(motor: &TachoMotor, rot: f64) -> Ev3Result<()> {
    let count = motor.get_count_per_rot()? as f64*rot as f64;
    motor.run_to_rel_pos(Some(count as i32))?;
    motor.wait_until_not_moving(None);
    Ok(())
}

fn rot_base45(hw: &Hardware) -> Ev3Result<()> {
    run_for_rot(&hw.base_motor, 0.375)?;
    Ok(())
}

fn rot_base90(hw: &Hardware) -> Ev3Result<()> {
    run_for_rot(&hw.base_motor, 0.75)?;
    Ok(())
}

fn flip_cube(hw: &Hardware) -> Ev3Result<()> {
    run_for_deg(&hw.flipper_motor,200)?;
    run_for_deg(&hw.flipper_motor,-200)?;
    Ok(())
}

fn lock_cube(hw: &Hardware) -> Ev3Result<()> {
    run_for_deg(&hw.flipper_motor,100)?;
    Ok(())
}

fn unlock_cube(hw: &Hardware) -> Ev3Result<()> {
    run_for_deg(&hw.flipper_motor,-100)?;
    Ok(())
}

fn sensor_scan(hw: &Hardware,data :&mut Data) -> Ev3Result<()>{
    let sensi32 = hw.color_sensor.get_rgb()?;
    let rgb = ((sensi32.0 as f64*1.7)*(255./1020.)
    ,sensi32.1 as f64*(255./1020.)
    ,(sensi32.2 as f64*1.875)*(255./1020.));
    println!("{}",format!("({},{},{})",rgb.0,rgb.1,rgb.2).truecolor(rgb.0 as u8, rgb.1 as u8, rgb.2 as u8));
    let idx = data.scan_order[data.curr_idx];
    let hsv = hsv_from_rgb(rgb);
    data.facelet_rgb_values[idx] = Col([NotNan::new(hsv.0).unwrap(),NotNan::new(hsv.1).unwrap(),NotNan::new(hsv.2).unwrap()],' ');
    data.curr_idx+=1;
    Ok(())
}

fn reset_sensor_position(hw: &Hardware) -> Ev3Result<()> {
    println!("{}","Resetting sensor arm".blue());
    hw.sensor_motor.run_forever()?;
    hw.sensor_motor.wait_until(TachoMotor::STATE_STALLED, None);
    hw.sensor_motor.stop()?;
    Ok(())
}

fn solve_cube(cube_notation: String) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("./kociemba {}",cube_notation))
        .output()
        .expect("Failed to execute Kociemba executable");
    String::from_utf8(output.stdout).expect("Could not convert Kociemba output to string")
}

fn apply_solution_part(part: String, hw: &Hardware, data :&mut Data) -> Ev3Result<()> {
    println!("Applying part {}",part);
    let face = part.chars().nth(0).unwrap();
    if !data.next_faces.contains(&face) { // then we have to rotate
        rot_base90(hw)?;
        let tmp = data.left_face;
        let tmp2 = data.right_face;
        data.left_face = data.next_faces[3];
        data.right_face = data.next_faces[1];
        data.next_faces[1] = tmp;
        data.next_faces[3] = tmp2;
    }
    while data.next_faces[0] != face {
        flip_cube(hw)?;
        data.next_faces.rotate_left(1);
    }
    lock_cube(hw)?;
    if part.len() == 1 { // 90deg clockwise
        // We need to go a little further each time as the base borders are not the same width as the cube
        run_for_rot(&hw.base_motor, -0.925)?; 
        run_for_rot(&hw.base_motor, 0.175)?;
    }
    else if part.ends_with('\''){ // 90 deg counterclockwise
        run_for_rot(&hw.base_motor, 0.925)?;
        run_for_rot(&hw.base_motor, -0.175)?;
    } else { // 180deg
        run_for_rot(&hw.base_motor, 1.675)?;
        run_for_rot(&hw.base_motor, -0.175)?;
    }
    unlock_cube(hw)?;
    return Ok(());
}

fn scan_face(hw: &Hardware, data :&mut Data) -> Ev3Result<()> {
    println!("Starting face scan");
    run_for_deg(&hw.sensor_motor,-600)?;
    sensor_scan(hw,data)?;
    run_for_deg(&hw.sensor_motor,90)?;
    for i in 0..4 {
        if i == 1 {
            run_for_deg(&hw.sensor_motor,-20)?;
        }
        sensor_scan(hw,data)?;
        rot_base45(hw)?;
        run_for_deg(&hw.sensor_motor,40)?;
        sensor_scan(hw,data)?;
        rot_base45(hw)?;
        run_for_deg(&hw.sensor_motor,-40)?;
        if i == 2 {
            run_for_deg(&hw.sensor_motor,20)?;
        }
    }
    reset_sensor_position(hw)?;
    println!("Face scan done");
    Ok(())
}

fn scan_cube(hw: &Hardware, data :&mut Data) -> Ev3Result<()> {
    for _ in 0..4{
        // U,F,D,B scan
        flip_cube(hw)?;
        scan_face(hw, data)?;
    }
    rot_base90(hw)?;
    flip_cube(hw)?;
    // R scan
    scan_face(hw,data)?;
    flip_cube(hw)?;
    sleep(Duration::from_millis(100)); // waiting for the cube to fall before second rotation
    flip_cube(hw)?;
    // L scan
    scan_face(hw,data)?;
    Ok(())
}

fn main() -> Ev3Result<()> {
    
    let base_motor: TachoMotor = TachoMotor::get(MotorPort::OutC)?;
    base_motor.set_speed_sp((base_motor.get_max_speed()?as f32/1.5) as i32)?;
    base_motor.set_ramp_down_sp(1000)?; // This is used to make the motor progressively stop. Else it lacks precision

    let flipper_motor: TachoMotor = TachoMotor::get(MotorPort::OutD)?;
    flipper_motor.set_speed_sp(base_motor.get_max_speed()?/3)?;
    flipper_motor.set_ramp_down_sp(1000)?;
    
    let sensor_motor: TachoMotor = TachoMotor::get(MotorPort::OutB)?;
    sensor_motor.set_speed_sp(base_motor.get_max_speed()?/2)?;
    sensor_motor.set_ramp_down_sp(0)?;
    

    let sensor = ColorSensor::find()?;
    sensor.set_mode_rgb_raw()?;

    let hw = Hardware{base_motor: base_motor, 
        flipper_motor: flipper_motor, 
        sensor_motor:sensor_motor, 
        color_sensor: sensor};
    let mut data = Data::init();
    reset_sensor_position(&hw)?;
    scan_cube(&hw,&mut data)?;
    println!("Color values: {:?} (size {})",data.facelet_rgb_values, data.facelet_rgb_values.len());
    let tree = KDTree::new(&data.facelet_rgb_values);
    let centre_to_face: HashMap<usize, char> = HashMap::from([(4,'U'),(22,'F'),(31,'D'),(49,'B'),(13,'R'),(40,'L')]);
    for centre in [4, 22, 31, 49, 13, 40]{
        let face = centre_to_face.get(&centre).unwrap();
        data.facelet_rgb_values[centre].1 = face.clone();
        let neighbours = tree.knn(8, &data.facelet_rgb_values[centre]);
        for mut neighbour in neighbours{
            neighbour.point.1 = face.clone();
            data.facelet_rgb_values[neighbour.index as usize] = neighbour.point;
        }
    }
    let cube_string = data.facelet_rgb_values.iter().map(|x|x.1).collect();
    println!("Cube string is: {}",cube_string);
    let solution = solve_cube(cube_string);
    println!("Solution is {}",solution);
    for part in solution.split_whitespace(){
        apply_solution_part(part.to_owned(), &hw, &mut data)?;
    }
    Ok(())
}
