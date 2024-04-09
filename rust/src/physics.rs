use godot::{engine::Curve, obj::NewGd, prelude::*};

use std::{
    cmp::{max, min},
    usize,
};

// Pure, dry air has a density of 1.293 kg m−3 at a temperature of 273 K and a pressure of 101.325 kPa
pub const AIR_DENSITY: f32 = 1.293;
pub const ROLL_RESISTANCE_ASPHALT: f32 = 0.02;
pub const DEFAULT_DRAG_COEFFICIENT: f32 = 0.562;

#[derive(Debug)]
pub struct CarConfiguration {
    pub wheel: Wheel,
    pub gear_ratios: Vec<f32>,
    // final drive/rear gear ratio/differential gear ratio
    pub final_drive: f32,
    // time it takes to shift from one gear to another in seconds
    pub shift_lag_seconds: f32,
    // engine stalls below idle_rpm
    pub idle_rpm: i32,
    // electronic rev limitter. usually less than max rpm
    pub rev_limit: i32,
    // beyound which engine is doomed to fail
    pub max_engine_rpm: i32,
    // total weight in kilograms
    pub curb_weight: f32,
    /// https://en.wikipedia.org/wiki/Automobile_drag_coefficient
    /// cdA value for the car
    pub drag_coefficient: f32,
    // torque in Nm
    pub torque_curve: Option<Gd<Curve>>,
    pub peak_torque: f32,
}

impl Default for CarConfiguration {
    fn default() -> Self {
        CarConfiguration {
            wheel: Wheel {
                width: 0.0,
                aspect_ratio: 0.0,
                diameter: 0.0,
                rolling_resistance: ROLL_RESISTANCE_ASPHALT,
            },
            gear_ratios: vec![],
            final_drive: 0.0,
            shift_lag_seconds: 0.0,
            idle_rpm: 0,
            rev_limit: 0,
            max_engine_rpm: 0,
            curb_weight: 0.0,
            drag_coefficient: DEFAULT_DRAG_COEFFICIENT,
            torque_curve: None,
            peak_torque: 0.0,
        }
    }
}

pub enum CarState {
    Idle,
    Moving,
    Shifting,
    //Stalled,
    //Busted,
}

pub enum GearShift {
    ShiftUp,
    SiftDown,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct CarMetrics {
    pub rpm: i32,
    // Nm
    pub torque: f32,
    // KW
    pub power: f32,
    // eg: 1 - 7
    pub gear: usize,
    /// m/s
    pub velocity: f32,
    /// 0 - 1.0
    pub throttle: f32,
    /// m/s^2
    pub acceleration: f32,
}

pub struct Car {
    pub state: CarState,
    config: CarConfiguration,
    // simulation values
    rpm: i32,
    // Nm
    torque: f32,
    // KW
    power: f32,
    // eg: 1 - 7
    gear: usize,
    /// m/s
    velocity: f32,
    /// 0 - 1.0
    throttle: f32,
    /// m/s^2
    acceleration: f32,
    shift_timer: f64,
}

impl Car {
    pub fn new(config: CarConfiguration) -> Self {
        Self {
            rpm: config.idle_rpm,
            gear: 1,
            velocity: 0.0,
            throttle: 0.0,
            acceleration: 0.0,
            power: 0.0,
            torque: 0.0,
            shift_timer: 0.0,
            state: CarState::Idle,
            config,
        }
    }

    // effective gear ratio. gear_ratio * final_drive
    fn gear_ratio(&self) -> f32 {
        let gear = self.config.gear_ratios[self.gear - 1];
        let final_drive = self.config.final_drive;
        return gear * final_drive;
    }

    /// calculate engine rpm from car velocity
    fn calc_rpm(&self, velocity: f32) -> i32 {
        let wheel_radi = self.config.wheel.radius() / 1000.0;
        let wheel_speed = velocity / wheel_radi;
        let engine_speed = wheel_speed * self.gear_ratio();
        let engine_rpm = (engine_speed * 30.0) / std::f32::consts::PI;

        return min(
            max(0, engine_rpm.round() as i32),
            self.config.max_engine_rpm,
        );
    }

    fn calc_torque(&self, rpm: i32) -> f32 {
        if let Some(curve) = &self.config.torque_curve {
            let x: f32 = rpm as f32 / self.config.max_engine_rpm as f32;
            let y: f32 = curve.sample(x);

            return y * self.config.peak_torque;
        }
        return self.config.peak_torque;
    }

    /// https://en.wikipedia.org/wiki/Automobile_drag_coefficient
    fn calc_drag_force(&self, velocity: f32) -> f32 {
        return 0.5 * AIR_DENSITY * velocity * velocity * self.config.drag_coefficient;
    }

    pub fn metrics(&self) -> CarMetrics {
        CarMetrics {
            rpm: self.rpm,
            torque: self.torque,
            power: self.power,
            gear: self.gear,
            velocity: self.velocity,
            throttle: self.throttle,
            acceleration: self.acceleration,
        }
    }

    pub fn start(&mut self, speed: f32) {
        self.rpm = self.config.idle_rpm;
        self.gear = 1;
        self.throttle = 1.0;
        // self.state = CarState::Idle;
        self.state = CarState::Moving;

        if speed > 0.0 {
            self.state = CarState::Moving;
            self.velocity = speed;
            self.rpm = self.calc_rpm(speed);
        }
    }

    /// https://www.reddit.com/r/gamedev/comments/1320er/math_for_simulating_a_car_transmission_xpost_rmath/
    pub fn update(&mut self, delta: f64) -> CarMetrics {
        let velocity = self.velocity;
        let mut rpm = self.rpm;

        match self.state {
            CarState::Idle => {
                godot_print!("idling, todo: increment rpm {} based on throttle", rpm);
            }
            CarState::Moving => {
                rpm = self.calc_rpm(velocity);
            }
            CarState::Shifting => {
                self.shift_timer -= delta;
                if self.shift_timer <= 0.0 {
                    self.state = CarState::Moving;
                }
                rpm = self.calc_rpm(velocity);
            }
        }

        let torque = self.calc_torque(rpm) * self.throttle;
        let drive_torque = torque * self.gear_ratio();
        let mut force = drive_torque / (self.config.wheel.radius() / 1000.0);

        // simulate clutch disengaged scenario or nueutral gear
        if matches!(self.state, CarState::Shifting | CarState::Idle) {
            force = 0.0;
        }
        // apply air drag resistance, and rolling resistance
        if velocity > 0.0 {
            let drag = self.calc_drag_force(velocity);
            let roll_res = self.config.wheel.resistance_force(self.config.curb_weight);
            force -= drag + roll_res;
        }

        let acceleration = force / self.config.curb_weight; // m/s2
        let velocity = velocity + (acceleration * delta as f32);

        // use drive_torque instead ??
        let power = (torque * rpm as f32) / 9549.296585513720;

        self.rpm = rpm;
        self.torque = torque;
        self.power = power;
        self.acceleration = acceleration;
        self.velocity = velocity;

        return self.metrics();
    }

    pub fn shift_gear(&mut self, shift: GearShift) {
        if let CarState::Shifting = self.state {
            return;
        }

        let mut gear = self.gear;
        let max_gear = self.config.gear_ratios.len();
        match shift {
            GearShift::ShiftUp => gear = min(gear + 1, max_gear),
            GearShift::SiftDown => gear = max(1, gear - 1),
        }

        self.gear = gear;
        self.state = CarState::Shifting;
        self.shift_timer = self.config.shift_lag_seconds.into();
    }
}

#[derive(Clone, Copy, Debug, GodotClass)]
#[class(init)]
pub struct Wheel {
    pub width: f32,
    pub aspect_ratio: f32,
    pub diameter: f32,
    pub rolling_resistance: f32,
}

impl Default for Wheel {
    fn default() -> Self {
        Self {
            width: 0.0,
            aspect_ratio: 0.0,
            diameter: 0.0,
            rolling_resistance: ROLL_RESISTANCE_ASPHALT,
        }
    }
}

impl Wheel {
    /// return wheel radius in mm
    /// https://x-engineer.org/calculate-wheel-radius/
    pub fn radius(&self) -> f32 {
        let height = self.width * (self.aspect_ratio / 100.0);
        let diameter = (self.diameter as f32 * 25.4) + (2.0 * height);
        return diameter / 2.0;
    }

    // circumference = 2πr
    pub fn circumference(&self) -> f32 {
        return self.radius() * 2.0 * std::f32::consts::PI;
    }

    // rolling resistance force in N
    // https://www.engineeringtoolbox.com/rolling-friction-resistance-d_1303.html
    pub fn resistance_force(&self, weight: f32) -> f32 {
        return self.rolling_resistance * weight * 9.81;
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_calc_rpm() {
        let config = CarConfiguration {
            wheel: Wheel {
                width: 285.0,
                aspect_ratio: 35.0,
                diameter: 19.0,
                rolling_resistance: 0.0,
            },
            gear_ratios: vec![3.30],
            final_drive: 4.20,
            max_engine_rpm: 7000,
            ..Default::default()
        };
        let car = Car::new(config);
        assert_eq!(car.calc_rpm(9.0), 3493);
    }

    #[test]
    fn test_calc_torque() {
        //let mut curve = Curve::new_gd();
        //curve.add_point(Vector2::new(0.0, 0.0));
        //curve.add_point(Vector2::new(0.5, 0.5));
        //curve.add_point(Vector2::new(0.75, 1.0));
        //curve.add_point(Vector2::new(1.0, 1.0));
        let config = CarConfiguration {
            torque_curve: None,
            peak_torque: 500.0,
            max_engine_rpm: 3000,
            ..Default::default()
        };
        let car = Car::new(config);
        //assert_eq!(car.calc_torque(500), 100.0);
        //assert_eq!(car.calc_torque(800), 150.0);
        //assert_eq!(car.calc_torque(1500), 250.0);
        assert_eq!(car.calc_torque(2550), 500.0);
    }

    #[test]
    fn test_car_update() {
        //let mut flat_curve = Curve::new_gd();
        //flat_curve.add_point(Vector2::new(0.0, 1.0));
        //flat_curve.add_point(Vector2::new(1.0, 1.0));

        let config = CarConfiguration {
            wheel: Wheel {
                width: 285.0,
                aspect_ratio: 35.0,
                diameter: 19.0,
                rolling_resistance: ROLL_RESISTANCE_ASPHALT,
            },
            final_drive: 4.20,
            gear_ratios: vec![3.30],
            curb_weight: 1500.0,
            torque_curve: None,
            peak_torque: 500.0,
            max_engine_rpm: 5000,
            drag_coefficient: DEFAULT_DRAG_COEFFICIENT,
            ..Default::default()
        };
        let mut car = Car::new(config);
        car.start(9.0); // rolling start with 10m/s speed
        car.update(0.1);
        assert_eq!(
            car.metrics(),
            CarMetrics {
                rpm: 3493,
                torque: 500.0,
                power: 182.89305,
                gear: 1,
                velocity: 10.333058,
                throttle: 1.0,
                acceleration: 13.33058
            }
        )
    }

    #[test]
    fn test_radius() {
        let w = Wheel {
            width: 225.0,
            aspect_ratio: 55.0,
            diameter: 17.0,
            rolling_resistance: 0.0,
        };
        assert_eq!(w.radius(), 339.65);

        let w = Wheel {
            width: 285.0,
            aspect_ratio: 35.0,
            diameter: 19.0,
            rolling_resistance: 0.0,
        };
        assert_eq!(w.radius(), 341.05);
    }
}
