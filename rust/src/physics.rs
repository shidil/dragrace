use std::{cmp::min, usize};

const RPM_STEP: i32 = 500;

#[derive(Clone)]
struct CarConfiguration {
    wheel: Wheel,
    gear_ratios: Vec<f32>,
    // final drive/rear gear ratio/differential gear ratio
    final_drive: f32,
    // time it takes to shift from one gear to another in seconds
    shift_lag_seconds: f32,
    // engine stalls below idle_rpm
    idle_rpm: i32,
    // electronic rev limitter. usually less than max rpm
    rev_limit: i32,
    // beyound which engine is doomed to fail
    max_engine_rpm: i32,
    // torque in Nm for 500 rpm increments
    torque_levels: Vec<f32>,
    // total weight in kilograms
    curb_weight: f32,
    /// https://en.wikipedia.org/wiki/Automobile_drag_coefficient
    drag_coefficient: f32,
}

impl Default for CarConfiguration {
    fn default() -> Self {
        CarConfiguration {
            wheel: Wheel {
                width: 0.0,
                aspect_ratio: 0.0,
                diameter: 0.0,
            },
            gear_ratios: vec![],
            final_drive: 0.0,
            shift_lag_seconds: 0.0,
            idle_rpm: 0,
            rev_limit: 0,
            max_engine_rpm: 0,
            torque_levels: vec![],
            curb_weight: 0.0,
            drag_coefficient: 0.0,
        }
    }
}

enum CarState {
    Idle,
    Moving,
    Stalled,
    Busted,
    Shifting,
}

struct Car {
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
    state: CarState,
}

impl Car {
    fn new(config: CarConfiguration) -> Self {
        Self {
            rpm: config.idle_rpm,
            gear: 1,
            velocity: 0.0,
            throttle: 0.0,
            acceleration: 0.0,
            power: 0.0,
            torque: 0.0,
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
        return engine_rpm.round() as i32;
    }

    fn calc_torque(&self, rpm: i32) -> f32 {
        let step: usize = (rpm / RPM_STEP) as usize;
        // accomodate index overflow
        let index = min(step, self.config.torque_levels.len() - 1);
        return self.config.torque_levels[index];
    }

    pub fn start(&mut self) {
        self.rpm = self.config.idle_rpm;
        self.gear = 1;
        self.throttle = 1.0;
    }

    /// https://www.reddit.com/r/gamedev/comments/1320er/math_for_simulating_a_car_transmission_xpost_rmath/
    pub fn update(&mut self, delta: f64) {
        // In GDScript, this would be:
        // rotation += angular_speed * delta

        //let radians = (self.angular_speed * delta) as f32;
        //self.base_mut().rotate(radians);
        // The 'rotate' method requires a f32,
        // therefore we convert 'self.angular_speed * delta' which is a f64 to a f32
        let rpm = self.calc_rpm(self.velocity);
        let torque = self.calc_torque(rpm);

        self.rpm = rpm;
        self.torque = torque;
    }
}

#[derive(Clone, Copy)]
struct Wheel {
    width: f32,
    aspect_ratio: f32,
    diameter: f32,
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
            },
            gear_ratios: vec![3.30],
            final_drive: 4.20,
            ..Default::default()
        };
        let car = Car::new(config);
        assert_eq!(car.calc_rpm(9.0), 3493);
    }

    #[test]
    fn test_calc_torque() {
        let config = CarConfiguration {
            torque_levels: vec![0.0, 100.0, 200.0, 300.0, 400.0],
            ..Default::default()
        };
        let car = Car::new(config);
        assert_eq!(car.calc_torque(500), 100.0);
        assert_eq!(car.calc_torque(800), 100.0);
        assert_eq!(car.calc_torque(1600), 300.0);
        assert_eq!(car.calc_torque(2500), 400.0);
    }

    #[test]
    fn test_radius() {
        let w = Wheel {
            width: 225.0,
            aspect_ratio: 55.0,
            diameter: 17.0,
        };
        assert_eq!(w.radius(), 339.65);

        let w = Wheel {
            width: 285.0,
            aspect_ratio: 35.0,
            diameter: 19.0,
        };
        assert_eq!(w.radius(), 341.05);
    }
}
