extern crate ev3dev_lang_rust;

use std::iter;
use std::ops::Index;
use std::thread::sleep;
use std::time::Duration;

use ev3dev_lang_rust::{motors, Ev3Result};
use ev3dev_lang_rust::motors::{MotorPort, TachoMotor};
use ev3dev_lang_rust::sensors::ColorSensor;
use std::process::Command;

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
    facelet_rgb_values: Vec<(i32,i32,i32)>,
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
            facelet_rgb_values: iter::repeat((0,0,0)).take(54).collect(),
            next_faces: ['R','F','L','B'],
            right_face: 'D',
            left_face: 'U'
        }
    }
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
    run_for_deg(&hw.flipper_motor,210)?;
    run_for_deg(&hw.flipper_motor,-210)?;
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
    let sens = hw.color_sensor.get_rgb()?;
    println!("({},{},{})",sens.0,sens.1,sens.2);
    data.facelet_rgb_values[data.scan_order[data.curr_idx]] = sens;
    data.curr_idx+=1;
    Ok(())
}

fn reset_sensor_position(hw: &Hardware) -> Ev3Result<()> {
    println!("Resetting sensor arm");
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
    // TODO: Make the right face face down
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
    base_motor.set_speed_sp(base_motor.get_max_speed()?/2)?;
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
    println!("Color values: {:?}",data.facelet_rgb_values);
    let solution = solve_cube("FRRUUUUUUFFDRRDRRDLLLFFFFFFDDDDDDBLLUBBULLULLBBRBBRBBR".to_string());
    println!("Seultion is {}",solution);
    for part in solution.split_whitespace(){
        apply_solution_part(part.to_owned(), &hw, &mut data)?;
    }
    Ok(())
}
