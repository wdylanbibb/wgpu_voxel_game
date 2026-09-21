use std::range::Range;

use crate::{player, world};

pub struct Game {
    pub world: world::World,
    pub player: player::Player,
}

impl Game {
    pub fn new(dimensions: cgmath::Vector3<Range<i32>>) -> Self {
        let world = world::World::new(dimensions);

        let spawn = world
            .spawn_point(0, 0)
            .expect("could not find a valid player spawn point");

        let player = player::Player {
            position: spawn,
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            grounded: false,
        };

        Self { world, player }
    }

    pub fn update(
        &mut self,
        dt: f32,
        movement: cgmath::Vector2<f32>,
        forward: cgmath::Vector3<f32>,
        right: cgmath::Vector3<f32>,
        jump: bool,
    ) {
        self.player
            .update(&self.world, movement, forward, right, jump, dt);
    }
}
