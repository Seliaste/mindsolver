use std::thread::sleep;
use std::time::Duration;

use colored::Colorize;
use ev3dev_lang_rust::Ev3Result;
use ev3dev_lang_rust::motors::{MotorPort, TachoMotor};
use ev3dev_lang_rust::sensors::ColorSensor;
use paris::{info, log, success};

use crate::classification::Point;
use crate::cube::Cube;

/// A representation of the robot hardware, as in motors and sensor.
pub struct Hardware {
    /// Motor of the platform
    pub base_motor: TachoMotor,
    /// Motor for the flipper arm
    pub flipper_motor: TachoMotor,
    /// Motor for the sensor arm
    pub sensor_motor: TachoMotor,
    pub color_sensor: ColorSensor,
    /// Represents whether the flipper arm is locking the cube
    pub locked: bool,
}

impl Hardware {
    pub fn init() -> Ev3Result<Self> {
        let base_motor: TachoMotor = TachoMotor::get(MotorPort::OutC)?;
        base_motor.set_speed_sp((base_motor.get_max_speed()? as f32 / 1.5) as i32)?;
        base_motor.set_ramp_down_sp(0)?; // This is used to make the motor progressively stop. Else it lacks precision
        base_motor.set_stop_action(TachoMotor::STOP_ACTION_HOLD)?;

        let flipper_motor: TachoMotor = TachoMotor::get(MotorPort::OutD)?;
        flipper_motor.set_speed_sp(base_motor.get_max_speed()? / 3)?;
        flipper_motor.set_ramp_down_sp(0)?;
        flipper_motor.set_stop_action(TachoMotor::STOP_ACTION_HOLD)?;

        let sensor_motor: TachoMotor = TachoMotor::get(MotorPort::OutB)?;
        sensor_motor.reset()?;
        sensor_motor.set_speed_sp(base_motor.get_max_speed()? / 2)?;
        sensor_motor.set_ramp_down_sp(0)?;
        sensor_motor.set_stop_action(TachoMotor::STOP_ACTION_HOLD)?;
        sensor_motor.set_polarity(TachoMotor::POLARITY_NORMAL)?;

        let color_sensor = ColorSensor::find()?;
        color_sensor.set_mode_rgb_raw()?;
        return Ok(Hardware {
            base_motor,
            flipper_motor,
            sensor_motor,
            color_sensor,
            locked: false,
        });
    }

    pub fn run_for_deg(motor: &TachoMotor, degree: i32) -> Ev3Result<()> {
        let count = motor.get_count_per_rot()? as f64 / 360. * degree as f64;
        motor.run_to_rel_pos(Some(count as i32))?;
        motor.wait_until_not_moving(None);
        Ok(())
    }

    pub fn run_for_rot(motor: &TachoMotor, rot: f64) -> Ev3Result<()> {
        Self::run_for_deg(motor, (rot * 360.) as i32)?;
        Ok(())
    }

    pub fn rot_base45(&self) -> Ev3Result<()> {
        Self::run_for_rot(&self.base_motor, 0.375)?;
        Ok(())
    }

    pub fn rot_base90(&self) -> Ev3Result<()> {
        Self::run_for_rot(&self.base_motor, 0.75)?;
        Ok(())
    }

    pub fn flip_cube(&mut self) -> Ev3Result<()> {
        if !self.locked {
            self.lock_cube()?;
        }
        Self::run_for_deg(&self.flipper_motor, 100)?;
        sleep(Duration::from_millis(100));
        Self::run_for_deg(&self.flipper_motor, -100)?;
        sleep(Duration::from_millis(100));
        Ok(())
    }

    pub fn lock_cube(&mut self) -> Ev3Result<()> {
        Self::run_for_deg(&self.flipper_motor, 100)?;
        self.locked = true;
        Ok(())
    }

    pub fn unlock_cube(&mut self) -> Ev3Result<()> {
        Self::run_for_deg(&self.flipper_motor, -100)?;
        self.locked = false;
        Ok(())
    }

    pub fn reset_sensor_position(&self) -> Ev3Result<()> {
        info!("Resetting sensor arm");
        self.sensor_motor.run_forever()?;
        self.sensor_motor
            .wait_until(TachoMotor::STATE_STALLED, None);
        self.sensor_motor.stop()?;
        Ok(())
    }

    pub fn sensor_scan(&self, data: &mut Cube) -> Ev3Result<()> {
        sleep(Duration::from_millis(20));
        let sens_1 = self.color_sensor.get_rgb()?;
        let sens_2 = self.color_sensor.get_rgb()?;
        let sens_3 = self.color_sensor.get_rgb()?;
        let sens_i32 = (
            (sens_1.0 + sens_2.0 + sens_3.0) / 3,
            (sens_1.1 + sens_2.1 + sens_3.1) / 3,
            (sens_1.2 + sens_2.2 + sens_3.2) / 3,
        );
        let rgb = (
            (sens_i32.0 as f64 * 1.7) * (255. / 1020.),
            sens_i32.1 as f64 * (255. / 1020.),
            (sens_i32.2 as f64 * 2.) * (255. / 1020.),
        );
        log!(
            "Scanned {}",
            format!("({},{},{})", rgb.0, rgb.1, rgb.2).truecolor(
                rgb.0 as u8,
                rgb.1 as u8,
                rgb.2 as u8
            )
        );
        let idx = data.scan_order[data.curr_idx];
        data.facelet_rgb_values[idx] = Point {
            x: rgb.0,
            y: rgb.1,
            z: rgb.2,
            index: idx,
        };
        data.curr_idx += 1;
        Ok(())
    }

    /// Will apply a transformation. Examples of transformation notations are `R, U, R', U2`
    pub fn apply_solution_part(&mut self, part: String, cube: &mut Cube) -> Ev3Result<()> {
        info!("Applying part {}", part);
        let face = part.chars().nth(0).unwrap();
        if !cube.next_faces.contains(&face) {
            // then we have to rotate
            if self.locked {
                self.unlock_cube()?;
            }
            self.rot_base90()?;
            let tmp = cube.left_face;
            let tmp2 = cube.right_face;
            cube.left_face = cube.next_faces[3];
            cube.right_face = cube.next_faces[1];
            cube.next_faces[1] = tmp;
            cube.next_faces[3] = tmp2;
        }
        while cube.next_faces[0] != face {
            self.flip_cube()?;
            cube.next_faces.rotate_left(1);
        }
        if !self.locked {
            self.lock_cube()?;
        }
        if part.len() == 1 {
            // 90deg clockwise
            // We need to go a little further each time as the base borders are not the same width as the cube
            Hardware::run_for_rot(&self.base_motor, -0.925)?;
            Hardware::run_for_rot(&self.base_motor, 0.175)?;
        } else if part.ends_with('\'') {
            // 90 deg counterclockwise
            Hardware::run_for_rot(&self.base_motor, 0.900)?;
            Hardware::run_for_rot(&self.base_motor, -0.150)?;
        } else {
            // 180deg
            Hardware::run_for_rot(&self.base_motor, 1.675)?;
            Hardware::run_for_rot(&self.base_motor, -0.175)?;
        }
        return Ok(());
    }

    /// Scans the face facing up and adds the colours to the cube struct
    pub fn scan_face(&mut self, cube: &mut Cube) -> Ev3Result<()> {
        info!("Starting face scan...");
        if self.locked {
            self.unlock_cube()?;
        }
        Hardware::run_for_deg(&self.sensor_motor, -680)?;
        self.sensor_scan(cube)?;
        let offsets = [100, -30, 10, 10];
        for i in 0..4 {
            Hardware::run_for_deg(&self.sensor_motor, offsets[i])?;
            self.sensor_scan(cube)?;
            self.rot_base45()?;
            Hardware::run_for_deg(&self.sensor_motor, 50)?;
            self.sensor_scan(cube)?;
            self.rot_base45()?;
            Hardware::run_for_deg(&self.sensor_motor, -50)?;
        }
        self.reset_sensor_position()?;
        success!("Face scan done!");
        Ok(())
    }

    pub fn scan_cube(&mut self, cube: &mut Cube) -> Ev3Result<()> {
        for _ in 0..4 {
            // U,F,D,B scan
            self.flip_cube()?;
            self.scan_face(cube)?;
        }
        self.flip_cube()?;
        self.unlock_cube()?;
        self.rot_base90()?;
        self.flip_cube()?;
        // R scan
        self.scan_face(cube)?;
        self.flip_cube()?;
        sleep(Duration::from_millis(100)); // waiting for the cube to fall before second rotation
        self.flip_cube()?;
        // L scan
        self.scan_face(cube)?;
        Ok(())
    }
}
