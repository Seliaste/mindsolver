use std::thread::sleep;
use std::time::Duration;

use colored::Colorize;
use ev3dev_lang_rust::motors::{MotorPort, TachoMotor};
use ev3dev_lang_rust::sensors::ColorSensor;
use ev3dev_lang_rust::Ev3Result;
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
        base_motor.set_speed_sp(base_motor.get_max_speed()?)?;
        base_motor.set_ramp_down_sp(0)?;
        base_motor.set_stop_action(TachoMotor::STOP_ACTION_HOLD)?;

        let flipper_motor: TachoMotor = TachoMotor::get(MotorPort::OutD)?;
        flipper_motor.set_speed_sp(base_motor.get_max_speed()? / 3)?;
        flipper_motor.set_ramp_down_sp(0)?;
        flipper_motor.set_stop_action(TachoMotor::STOP_ACTION_HOLD)?;

        let sensor_motor: TachoMotor = TachoMotor::get(MotorPort::OutB)?;
        sensor_motor.reset()?;
        sensor_motor.set_speed_sp((base_motor.get_max_speed()? as f32 / 1.5) as i32)?;
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
        Self::run_for_deg(&self.flipper_motor, 90)?;
        sleep(Duration::from_millis(100));
        Self::run_for_deg(&self.flipper_motor, -90)?;
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
        self.sensor_motor.run_forever()?;
        self.sensor_motor
            .wait_until(TachoMotor::STATE_STALLED, None);
        self.sensor_motor.stop()?;
        Ok(())
    }

    pub fn sensor_scan(&self, data: &mut Cube) -> Ev3Result<()> {
        /// Duration of sleep between each scan
        const SLEEP_DURATION: Duration = Duration::from_millis(20);
        /// Amount of movement between scans
        const MOVEMENT: i32 = 8;
        /// Number of scans for a single facelet
        const ITER: usize = 5;
        let mut scans = [[0.; 3]; ITER];
        for i in 0..ITER {
            let scan = self.color_sensor.get_rgb()?;
            scans[i] = [scan.0 as f64, scan.1 as f64, scan.2 as f64];
            Hardware::run_for_deg(&self.sensor_motor, MOVEMENT)?;
            sleep(SLEEP_DURATION);
        }
        Hardware::run_for_deg(&self.sensor_motor, (-MOVEMENT) * ITER as i32)?;
        let scan_avg = scans
            .iter()
            .fold([0.; 3], |acc, x| {
                [acc[0] + x[0], acc[1] + x[1], acc[2] + x[2]]
            })
            .map(|x| x / ITER as f64);
        let rgb = [
            (scan_avg[0] * 1.7) * (255. / 1020.), // hardcoded correction values
            scan_avg[1] * (255. / 1020.),
            (scan_avg[2] * 2.) * (255. / 1020.),
        ];
        log!(
            "Scanned {}",
            format!("{:?}", rgb.map(|x| { x as u8 })).truecolor(
                rgb[0] as u8,
                rgb[1] as u8,
                rgb[2] as u8
            )
        );
        let idx = data.scan_order[data.curr_idx];
        data.facelet_rgb_values[idx] = Point {
            x: rgb[0],
            y: rgb[1],
            z: rgb[2],
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
            Hardware::run_for_rot(&self.base_motor, 0.875)?;
            Hardware::run_for_rot(&self.base_motor, -0.125)?;
        } else {
            // 180deg
            Hardware::run_for_rot(&self.base_motor, 1.675)?;
            Hardware::run_for_rot(&self.base_motor, -0.175)?;
        }
        return Ok(());
    }

    /// Scans the face facing up and adds the colours to the cube struct
    pub fn scan_face(&mut self, cube: &mut Cube) -> Ev3Result<()> {
        if self.locked {
            self.unlock_cube()?;
        }
        Hardware::run_for_deg(&self.sensor_motor, -680)?;
        self.sensor_scan(cube)?;
        let offsets = [100, -20, 10, 10];
        for i in 0..4 {
            Hardware::run_for_deg(&self.sensor_motor, offsets[i])?;
            self.sensor_scan(cube)?;
            self.rot_base45()?;
            if i == 0 { Hardware::run_for_deg(&self.sensor_motor, 20)?; } else { Hardware::run_for_deg(&self.sensor_motor, 40)?; }
            self.sensor_scan(cube)?;
            self.rot_base45()?;
            Hardware::run_for_deg(&self.sensor_motor, -40)?;
        }
        self.reset_sensor_position()?;

        Ok(())
    }

    pub fn scan_cube(&mut self, cube: &mut Cube) -> Ev3Result<()> {
        for c in ['U', 'F', 'D', 'B'] {
            // U,F,D,B scan
            self.flip_cube()?;
            info!("Starting {} face scan...", c);
            self.scan_face(cube)?;
            success!("{} face scan done!", c);
        }
        self.flip_cube()?;
        self.unlock_cube()?;
        self.rot_base90()?;
        self.flip_cube()?;
        // R scan
        info!("Starting R face scan...");
        self.scan_face(cube)?;
        success!("R face scan done! Moving to the next...");
        self.flip_cube()?;
        sleep(Duration::from_millis(100)); // waiting for the cube to fall before second rotation
        self.flip_cube()?;
        // L scan
        info!("Starting L face scan...");
        self.scan_face(cube)?;
        success!("L face scan done! Cube scan over.");
        Ok(())
    }
}
