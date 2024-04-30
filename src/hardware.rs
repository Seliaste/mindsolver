use std::thread::sleep;
use std::time::Duration;

use colored::Colorize;
use ev3dev_lang_rust::Ev3Result;
use ev3dev_lang_rust::motors::{MotorPort, TachoMotor};
use ev3dev_lang_rust::sensors::ColorSensor;

pub struct Hardware {
    pub base_motor: TachoMotor,
    pub flipper_motor: TachoMotor,
    pub sensor_motor: TachoMotor,
    pub color_sensor: ColorSensor,
}

impl Hardware {
    pub fn init() -> Ev3Result<Self> {
        let base_motor: TachoMotor = TachoMotor::get(MotorPort::OutC)?;
        base_motor.set_speed_sp((base_motor.get_max_speed()? as f32 / 1.5) as i32)?;
        base_motor.set_ramp_down_sp(1000)?; // This is used to make the motor progressively stop. Else it lacks precision

        let flipper_motor: TachoMotor = TachoMotor::get(MotorPort::OutD)?;
        flipper_motor.set_speed_sp(base_motor.get_max_speed()? / 3)?;
        flipper_motor.set_ramp_down_sp(1000)?;

        let sensor_motor: TachoMotor = TachoMotor::get(MotorPort::OutB)?;
        sensor_motor.set_speed_sp(base_motor.get_max_speed()? / 2)?;
        sensor_motor.set_ramp_down_sp(0)?;


        let color_sensor = ColorSensor::find()?;
        color_sensor.set_mode_rgb_raw()?;
        return Ok(Hardware { base_motor, flipper_motor, sensor_motor, color_sensor });
    }
    pub fn run_for_deg(motor: &TachoMotor, degree: i32) -> Ev3Result<()> {
        let count = motor.get_count_per_rot()? as f64 / 360. * degree as f64;
        motor.run_to_rel_pos(Some(count as i32))?;
        motor.wait_until_not_moving(None);
        sleep(Duration::from_millis(20));
        Ok(())
    }

    pub fn run_for_rot(motor: &TachoMotor, rot: f64) -> Ev3Result<()> {
        let count = motor.get_count_per_rot()? as f64 * rot;
        motor.run_to_rel_pos(Some(count as i32))?;
        motor.wait_until_not_moving(None);
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

    pub fn flip_cube(&self) -> Ev3Result<()> {
        Self::run_for_deg(&self.flipper_motor, 200)?;
        Self::run_for_deg(&self.flipper_motor, -200)?;
        Ok(())
    }

    pub fn lock_cube(&self) -> Ev3Result<()> {
        Self::run_for_deg(&self.flipper_motor, 100)?;
        Ok(())
    }

    pub fn unlock_cube(&self) -> Ev3Result<()> {
        Self::run_for_deg(&self.flipper_motor, -100)?;
        Ok(())
    }

    pub fn reset_sensor_position(&self) -> Ev3Result<()> {
        println!("{}", "Resetting sensor arm".blue());
        self.sensor_motor.run_forever()?;
        self.sensor_motor.wait_until(TachoMotor::STATE_STALLED, None);
        self.sensor_motor.stop()?;
        Ok(())
    }
}
