use godot::obj::{NewGd, WithBaseField};
use godot::{engine::Node2D, prelude::*};

#[derive(GodotClass)]
#[class(base=Node2D)]
struct AIOpponent {
    #[export]
    config: Gd<CarConfiguration>,
    car: Car,
    base: Base<Node2D>,
}

use crate::physics::{Car, GearShift};
use crate::res_car::CarConfiguration;

#[godot_api]
impl AIOpponent {
    fn reset(&mut self) {
        godot_print!("resetting opponent car"); // Prints to the Godot console
        let binding = self.config.bind();
        self.car = Car::new(Box::new(binding.to_owned()));
        godot_print!("reset player");
        self.car.start(0.0); // pass initial velocity for rolling start
    }
}

#[godot_api]
impl INode2D for AIOpponent {
    fn init(base: Base<Node2D>) -> Self {
        let car = Car::new(Box::new(CarConfiguration::default()));
        let config = CarConfiguration::new_gd();

        Self { base, car, config }
    }

    fn ready(&mut self) {
        self.reset();
    }

    fn process(&mut self, delta: f64) {
        // gear up automatically
        let metrics = self.car.update(delta);
        let pos = metrics.velocity * delta as f32;
        let speed = metrics.velocity * 3.6;

        if metrics.gear < metrics.ideal_gear {
            self.car.shift_gear(GearShift::ShiftUp);
        } else if metrics.gear > metrics.ideal_gear {
            self.car.shift_gear(GearShift::ShiftDown);
        }

        let mut base = self.base_mut();
        base.translate(Vector2::new(pos, 0.0));

        godot_print!(
            "opponnet# rpm: {} | torque: {} Nm | pow: {} KW",
            metrics.rpm,
            metrics.torque,
            metrics.power,
        );
        godot_print!(
            "opponnet# gear: {} | v: {} m/s | speed: {} km/h",
            metrics.gear,
            metrics.velocity,
            speed,
        );
        godot_print!("opponent# pos.x: {}", pos);
    }
}
