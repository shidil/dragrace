use godot::engine::Curve;
use godot::obj::{NewGd, WithBaseField};
use godot::{engine::Node2D, prelude::*};

#[derive(GodotClass)]
#[class(base=Node2D)]
struct Player {
    car: Car,
    base: Base<Node2D>,
    #[export]
    gear_ratios: Array<f32>,
    #[export]
    final_drive: f32,
    #[export]
    torque_curve: Gd<Curve>,
    #[export]
    peak_torque: f32,
    //#[export]
    //torque_levels: Array<f32>,
    #[export]
    idle_rpm: i32,
    #[export]
    rev_limit: i32,
    #[export]
    max_engine_rpm: i32,
    #[export]
    shift_lag_seconds: f32,
    #[export]
    curb_weight: f32,
    #[export]
    drag_coefficient: f32,
    #[export]
    wheel: Vector3,
    #[export]
    wheel_roll_resistancve: f32,
}

use crate::physics::{
    Car, CarConfiguration, GearShift, Wheel, DEFAULT_DRAG_COEFFICIENT, ROLL_RESISTANCE_ASPHALT,
};

impl Player {
    fn reset_player(&mut self) {
        let mut gear_ratios: Vec<f32> = vec![];
        for v in self.gear_ratios.iter_shared() {
            gear_ratios.push(v);
        }

        self.car = Car::new(CarConfiguration {
            wheel: Wheel {
                width: self.wheel.x,
                aspect_ratio: self.wheel.y,
                diameter: self.wheel.z,
                rolling_resistance: self.wheel_roll_resistancve,
            },
            torque_curve: Some(self.torque_curve.to_owned()),
            peak_torque: self.peak_torque,
            gear_ratios,
            final_drive: self.final_drive,
            shift_lag_seconds: self.shift_lag_seconds,
            idle_rpm: self.idle_rpm,
            rev_limit: self.rev_limit,
            max_engine_rpm: self.max_engine_rpm,
            curb_weight: self.curb_weight,
            drag_coefficient: self.drag_coefficient,
        });
        self.car.start(0.0); // pass initial velocity for rolling start
    }
}

#[godot_api]
impl INode2D for Player {
    fn init(base: Base<Node2D>) -> Self {
        godot_print!("Hello, world! ddd"); // Prints to the Godot console
        let car = Car::new(CarConfiguration::default());

        Self {
            base,
            car,
            wheel: Vector3 {
                x: 235.0,
                y: 55.0,
                z: 17.0,
            },
            wheel_roll_resistancve: ROLL_RESISTANCE_ASPHALT,
            gear_ratios: Array::new(),
            torque_curve: Curve::new_gd(),
            peak_torque: 500.0,
            //torque_levels: Array::new(),
            final_drive: 4.0,
            idle_rpm: 800,
            rev_limit: 7000,
            max_engine_rpm: 7500,
            shift_lag_seconds: 1.0,
            curb_weight: 1800.0,
            drag_coefficient: DEFAULT_DRAG_COEFFICIENT,
        }
    }

    fn ready(&mut self) {
        self.reset_player();
    }

    fn process(&mut self, delta: f64) {
        let input = Input::singleton();
        if input.is_action_pressed("reset_game".into()) {
            self.reset_player();
        }
        if input.is_action_pressed("shift_up".into()) {
            self.car.shift_gear(GearShift::ShiftUp);
        }
        if input.is_action_pressed("shift_down".into()) {
            self.car.shift_gear(GearShift::SiftDown);
        }

        let metrics = self.car.update(delta);
        let pos = metrics.velocity * delta as f32;
        godot_print!(
            "rpm: {} | torque: {} Nm | pow: {} KW",
            metrics.rpm,
            metrics.torque,
            metrics.power,
        );
        godot_print!(
            "gear: {} | v: {} m/s | speed: {} km/h",
            metrics.gear,
            metrics.velocity,
            metrics.velocity * 3.6,
        );
        godot_print!("pos.x: {}", pos);
        self.base_mut().translate(Vector2::new(pos, 0.0))
    }
}
