use godot::prelude::*;

use std::{
    cmp::{max, min},
    usize,
};

// Pure, dry air has a density of 1.293 kg m−3 at a temperature of 273 K and a pressure of 101.325 kPa
pub const AIR_DENSITY: f32 = 1.293;
pub const ROLL_RESISTANCE_ASPHALT: f32 = 0.02;
pub const DEFAULT_DRAG_COEFFICIENT: f32 = 0.562;

pub trait Vehicle {
    fn idle_rpm(&self) -> i32;
    fn rev_limit(&self) -> i32;
    // total weight in kilograms
    fn curb_weight(&self) -> f32;
    // https://en.wikipedia.org/wiki/Automobile_drag_coefficient
    // cdA value for the car
    fn aero_drag_cda(&self) -> f32;

    // final drive/rear gear ratio/differential gear ratio
    fn final_drive(&self) -> f32;
    fn gear_ratio(&self, gear: usize) -> f32;
    fn max_gear(&self) -> usize;
    // time it takes to shift from one gear to another in seconds
    fn shift_lag_seconds(&self) -> f64;

    // torque in Nm
    fn peak_torque(&self) -> f32;
    // torque for the given engine rpm. Usually extrapolated from a curve
    fn torque(&self, rpm: i32) -> f32;

    fn tyre(&self) -> Wheel;

    /// return wheel radius in metre
    /// https://x-engineer.org/calculate-wheel-radius/
    fn wheel_radius(&self) -> f32 {
        self.tyre().radius()
    }

    /// wheel rolling resistane in coefficient
    /// https://www.engineeringtoolbox.com/rolling-friction-resistance-d_1303.html
    fn rolling_resistance(&self) -> f32 {
        self.tyre().rolling_resistance
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
    config: Box<dyn Vehicle>,
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
    pub fn new(config: Box<dyn Vehicle>) -> Self {
        Self {
            rpm: config.idle_rpm(),
            config,
            gear: 1,
            velocity: 0.0,
            throttle: 0.0,
            acceleration: 0.0,
            power: 0.0,
            torque: 0.0,
            shift_timer: 0.0,
            state: CarState::Idle,
        }
    }

    // effective gear ratio. gear_ratio * final_drive
    fn effective_gear_ratio(&self) -> f32 {
        let gear = self.config.gear_ratio(self.gear);
        let final_drive = self.config.final_drive();
        return gear * final_drive;
    }

    /// calculate engine rpm from car velocity
    fn calc_rpm(&self, velocity: f32) -> i32 {
        let wheel_radi = self.config.wheel_radius();
        let wheel_speed = velocity / wheel_radi;
        let engine_speed = wheel_speed * self.effective_gear_ratio();
        let engine_rpm = (engine_speed * 30.0) / std::f32::consts::PI;

        return min(max(0, engine_rpm.round() as i32), self.config.rev_limit());
    }

    /// https://en.wikipedia.org/wiki/Automobile_drag_coefficient
    fn calc_drag_force(&self, velocity: f32) -> f32 {
        return 0.5 * AIR_DENSITY * velocity * velocity * self.config.aero_drag_cda();
    }

    // rolling resistance force in N
    // https://www.engineeringtoolbox.com/rolling-friction-resistance-d_1303.html
    fn resistance_force(&self) -> f32 {
        let weight = self.config.curb_weight();
        return self.config.rolling_resistance() * weight * 9.81;
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
        self.rpm = self.config.idle_rpm();
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

        let torque = self.config.torque(rpm) * self.throttle;
        let drive_torque = torque * self.effective_gear_ratio();
        let mut force = drive_torque / (self.config.wheel_radius());

        // simulate clutch disengaged scenario or nueutral gear
        if matches!(self.state, CarState::Shifting | CarState::Idle) {
            force = 0.0;
        }
        // apply air drag resistance, and rolling resistance
        if velocity > 0.0 {
            let drag = self.calc_drag_force(velocity);
            let roll_res = self.resistance_force();
            force -= drag + roll_res;
        }

        let acceleration = force / self.config.curb_weight(); // m/s2
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
        let max_gear = self.config.max_gear();
        match shift {
            GearShift::ShiftUp => gear = min(gear + 1, max_gear),
            GearShift::SiftDown => gear = max(1, gear - 1),
        }

        self.gear = gear;
        self.state = CarState::Shifting;
        self.shift_timer = self.config.shift_lag_seconds();
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
    /// return wheel radius in metre
    /// https://x-engineer.org/calculate-wheel-radius/
    fn radius(&self) -> f32 {
        let height = self.width * (self.aspect_ratio / 100.0);
        let diameter = (self.diameter as f32 * 25.4) + (2.0 * height);
        return (diameter / 2.0) / 1000.0;
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    //#[test]
    //fn test_calc_rpm() {
    //    let config = CarConfiguration {
    //        wheel: Wheel {
    //            width: 285.0,
    //            aspect_ratio: 35.0,
    //            diameter: 19.0,
    //            rolling_resistance: 0.0,
    //        },
    //        gear_ratios: vec![3.30],
    //        final_drive: 4.20,
    //        max_engine_rpm: 7000,
    //        ..Default::default()
    //    };
    //    let car = Car::new(config);
    //    assert_eq!(car.calc_rpm(9.0), 3493);
    //}

    //#[test]
    //fn test_calc_torque() {
    //    //let mut curve = Curve::new_gd();
    //    //curve.add_point(Vector2::new(0.0, 0.0));
    //    //curve.add_point(Vector2::new(0.5, 0.5));
    //    //curve.add_point(Vector2::new(0.75, 1.0));
    //    //curve.add_point(Vector2::new(1.0, 1.0));
    //    let config = CarConfiguration {
    //        torque_curve: None,
    //        peak_torque: 500.0,
    //        max_engine_rpm: 3000,
    //        ..Default::default()
    //    };
    //    let car = Car::new(config);
    //    //assert_eq!(car.calc_torque(500), 100.0);
    //    //assert_eq!(car.calc_torque(800), 150.0);
    //    //assert_eq!(car.calc_torque(1500), 250.0);
    //    assert_eq!(car.calc_torque(2550), 500.0);
    //}

    //#[test]
    //fn test_car_update() {
    //    //let mut flat_curve = Curve::new_gd();
    //    //flat_curve.add_point(Vector2::new(0.0, 1.0));
    //    //flat_curve.add_point(Vector2::new(1.0, 1.0));

    //    let config = CarConfiguration {
    //        wheel: Wheel {
    //            width: 285.0,
    //            aspect_ratio: 35.0,
    //            diameter: 19.0,
    //            rolling_resistance: ROLL_RESISTANCE_ASPHALT,
    //        },
    //        final_drive: 4.20,
    //        gear_ratios: vec![3.30],
    //        curb_weight: 1500.0,
    //        torque_curve: None,
    //        peak_torque: 500.0,
    //        max_engine_rpm: 5000,
    //        drag_coefficient: DEFAULT_DRAG_COEFFICIENT,
    //        ..Default::default()
    //    };
    //    let mut car = Car::new(config);
    //    car.start(9.0); // rolling start with 10m/s speed
    //    car.update(0.1);
    //    assert_eq!(
    //        car.metrics(),
    //        CarMetrics {
    //            rpm: 3493,
    //            torque: 500.0,
    //            power: 182.89305,
    //            gear: 1,
    //            velocity: 10.333058,
    //            throttle: 1.0,
    //            acceleration: 13.33058
    //        }
    //    )
    //}

    #[test]
    fn test_radius() {
        let w = Wheel {
            width: 225.0,
            aspect_ratio: 55.0,
            diameter: 17.0,
            rolling_resistance: 0.0,
        };
        assert_eq!(w.radius(), 0.33965);

        let w = Wheel {
            width: 285.0,
            aspect_ratio: 35.0,
            diameter: 19.0,
            rolling_resistance: 0.0,
        };
        assert_eq!(w.radius(), 0.34105);
    }
}
