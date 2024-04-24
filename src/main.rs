extern crate ev3dev_lang_rust;

use core::time;
use std::fmt;
use std::thread::sleep;
use std::time::Duration;

use ev3dev_lang_rust::Ev3Result;
use ev3dev_lang_rust::motors::{MotorPort, TachoMotor};
use ev3dev_lang_rust::sensors::ColorSensor;
use std::process::Command;

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

fn rot_base45(base_motor: &TachoMotor) -> Ev3Result<()> {
    run_for_rot(base_motor, 0.375)?;
    Ok(())
}

fn rot_base90(base_motor: &TachoMotor) -> Ev3Result<()> {
    run_for_rot(&base_motor, 0.75)?;
    Ok(())
}

fn flip_cube(flipper_motor: &TachoMotor) -> Ev3Result<()> {
    run_for_deg(&flipper_motor,200)?;
    run_for_deg(&flipper_motor,-200)?;
    Ok(())
}

fn print_sensor_values(sensor: &ColorSensor) -> Ev3Result<()>{
    let sens = sensor.get_rgb()?;
    println!("({},{},{})",sens.0,sens.1,sens.2);
    Ok(())
}

fn reset_sensor_position(sensor_motor: &TachoMotor) -> Ev3Result<()> {
    println!("Resetting sensor arm");
    sensor_motor.run_forever()?;
    sensor_motor.wait_until(TachoMotor::STATE_STALLED, None);
    sensor_motor.stop()?;
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

fn scan_face(flipper_motor: &TachoMotor,sensor_motor: &TachoMotor,base_motor: &TachoMotor, sensor: &ColorSensor) -> Ev3Result<()> {
    println!("Starting face scan");
    run_for_deg(sensor_motor,-600)?;
    print_sensor_values(sensor)?;
    run_for_deg(sensor_motor,90)?;
    for _ in 0..4 {
        print_sensor_values(sensor)?;
        rot_base45(base_motor)?;
        run_for_deg(sensor_motor,60)?;
        print_sensor_values(sensor)?;
        rot_base45(base_motor)?;
        run_for_deg(sensor_motor,-60)?;
    }
    reset_sensor_position(sensor_motor)?;
    println!("Face scan done");
    Ok(())
}

fn scan_cube(flipper_motor: &TachoMotor,sensor_motor: &TachoMotor,base_motor: &TachoMotor, sensor: &ColorSensor) -> Ev3Result<()> {
    for _ in 0..4{
        flip_cube(flipper_motor)?;
        // F,R,B,L scan
        scan_face(flipper_motor, sensor_motor, base_motor, sensor)?;
    }
    rot_base90(base_motor)?;
    flip_cube(flipper_motor)?;
    // U scan
    scan_face(flipper_motor, sensor_motor, base_motor, sensor)?;
    flip_cube(flipper_motor)?;
    sleep(Duration::from_millis(100)); // waiting for the cube to fall before second rotation
    flip_cube(flipper_motor)?;
    // D scan
    scan_face(flipper_motor, sensor_motor, base_motor, sensor)?;
    Ok(())
}

fn main() -> Ev3Result<()> {
    let base_motor: TachoMotor = TachoMotor::get(MotorPort::OutC)?;
    base_motor.set_speed_sp(base_motor.get_max_speed()?/2)?;
    base_motor.set_ramp_down_sp(1000)?; // This is used to make the motor progressively stop. Else it lacks precision

    let flipper_motor: TachoMotor = TachoMotor::get(MotorPort::OutD)?;
    flipper_motor.set_speed_sp(base_motor.get_max_speed()?/4)?;
    flipper_motor.set_ramp_down_sp(1000)?;
    
    let sensor_motor: TachoMotor = TachoMotor::get(MotorPort::OutB)?;
    sensor_motor.set_speed_sp(base_motor.get_max_speed()?/2)?;
    sensor_motor.set_ramp_down_sp(0)?;
    // reset_sensor_position(&sensor_motor)?;

    let sensor = ColorSensor::find()?;
    sensor.set_mode_rgb_raw()?;
    // scan_cube(&flipper_motor, &sensor_motor, &base_motor, &sensor)?;
    // println!("{}",solve_cube("DRLUUBFBRBLURRLRUBLRDDFDLFUFUFFDBRDUBRUFLLFDDBFLUBLRBD".to_string()));
    Ok(())
}
