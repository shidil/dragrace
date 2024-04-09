use godot::{engine::Curve, prelude::*};

use crate::physics::{Vehicle, Wheel, DEFAULT_DRAG_COEFFICIENT};

#[derive(GodotClass, Clone, Debug)]
#[class(tool, init, base=Resource)]
pub struct CarConfiguration {
    #[export]
    gear_ratios: Array<f32>,
    #[export]
    final_drive: f32,
    #[export]
    torque_curve: Option<Gd<Curve>>,
    #[export]
    peak_torque: f32,
    #[export]
    idle_rpm: i32,
    #[export]
    rev_limit: i32,
    #[export]
    shift_lag_seconds: f64,
    #[export]
    curb_weight: f32,
    #[export]
    drag_coefficient: f32,
    #[export]
    wheel: Vector3,
    #[export]
    wheel_roll_resistancve: f32,
}

impl Default for CarConfiguration {
    fn default() -> Self {
        CarConfiguration {
            wheel: Vector3::default(),
            gear_ratios: Array::new(),
            final_drive: 0.0,
            shift_lag_seconds: 0.0,
            idle_rpm: 0,
            rev_limit: 0,
            curb_weight: 0.0,
            drag_coefficient: DEFAULT_DRAG_COEFFICIENT,
            torque_curve: None,
            peak_torque: 0.0,
            wheel_roll_resistancve: 0.0,
        }
    }
}

impl Vehicle for CarConfiguration {
    fn idle_rpm(&self) -> i32 {
        self.idle_rpm
    }
    fn rev_limit(&self) -> i32 {
        self.rev_limit
    }
    fn curb_weight(&self) -> f32 {
        self.curb_weight
    }
    fn aero_drag_cda(&self) -> f32 {
        self.drag_coefficient
    }
    fn final_drive(&self) -> f32 {
        self.final_drive
    }
    fn gear_ratio(&self, gear: usize) -> f32 {
        self.gear_ratios.get(gear - 1)
    }
    fn max_gear(&self) -> usize {
        self.gear_ratios.len()
    }
    fn shift_lag_seconds(&self) -> f64 {
        self.shift_lag_seconds
    }
    fn peak_torque(&self) -> f32 {
        self.peak_torque
    }
    fn torque(&self, rpm: i32) -> f32 {
        if let Some(curve) = &self.torque_curve {
            let x: f32 = rpm as f32 / self.rev_limit as f32;
            let y: f32 = curve.sample(x);

            return y * self.peak_torque;
        }
        return self.peak_torque;
    }
    fn tyre(&self) -> Wheel {
        Wheel {
            width: self.wheel.x,
            aspect_ratio: self.wheel.y,
            diameter: self.wheel.z,
            rolling_resistance: self.wheel_roll_resistancve,
        }
    }
}
