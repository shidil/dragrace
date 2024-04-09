use godot::obj::{NewGd, WithBaseField};
use godot::{engine::Node2D, prelude::*};

#[derive(GodotClass)]
#[class(base=Node2D)]
struct Player {
    #[export]
    config: Gd<CarConfiguration>,
    car: Car,
    base: Base<Node2D>,
}

use crate::physics::{Car, GearShift};
use crate::res_car::CarConfiguration;

#[godot_api]
impl Player {
    #[signal]
    fn metric_update(rpm: i32, gear: i32, speed: i32);

    fn reset_player(&mut self) {
        godot_print!("resetting player car"); // Prints to the Godot console
        let binding = self.config.bind();
        self.car = Car::new(Box::new(binding.to_owned()));
        godot_print!("reset player");
        self.car.start(0.0); // pass initial velocity for rolling start
    }
}

#[godot_api]
impl INode2D for Player {
    fn init(base: Base<Node2D>) -> Self {
        let car = Car::new(Box::new(CarConfiguration::default()));
        let config = CarConfiguration::new_gd();

        Self { base, car, config }
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
        let speed = metrics.velocity * 3.6;
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
            speed,
        );
        godot_print!("pos.x: {}", pos);

        let mut base = self.base_mut();
        base.translate(Vector2::new(pos, 0.0));
        base.emit_signal(
            "metric_update".into(),
            &[
                Variant::from(metrics.rpm),
                Variant::from(metrics.gear as i32),
                Variant::from(speed as i32),
            ],
        );
    }
}
