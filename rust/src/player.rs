use godot::engine::Node2D;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct Player {
    base: Base<Node2D>,
}

use godot::engine::INode2D;

#[godot_api]
impl INode2D for Player {
    fn init(base: Base<Node2D>) -> Self {
        godot_print!("Hello, world!"); // Prints to the Godot console

        Self { base }
    }

    fn process(&mut self, delta: f64) {
        // In GDScript, this would be:
        // rotation += angular_speed * delta

        //let radians = (self.angular_speed * delta) as f32;
        //self.base_mut().rotate(radians);
        // The 'rotate' method requires a f32,
        // therefore we convert 'self.angular_speed * delta' which is a f64 to a f32
    }
}
