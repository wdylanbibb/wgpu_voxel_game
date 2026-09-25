use std::range::Range;

use crate::{
    camera::Ray,
    player, raycast,
    world::{self, block::BlockId},
};

pub const BLOCK_INTERACTION_REACH: f32 = 6.0;

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

    pub fn break_block(&mut self, ray: Ray) -> bool {
        let Some(hit) = raycast::raycast(&self.world, ray, BLOCK_INTERACTION_REACH) else {
            return false;
        };

        self.world.set_block(hit.block_position, BlockId::Air)
    }

    pub fn place_block(&mut self, ray: Ray, block: BlockId) -> bool {
        let Some(hit) = raycast::raycast(&self.world, ray, BLOCK_INTERACTION_REACH) else {
            return false;
        };
        let Some(position) = hit.place_position else {
            return false;
        };

        let Some(existing) = self.world.get_block(position) else {
            return false;
        };

        if self.player.intersects_block(position) || existing.solid() {
            return false;
        }

        self.world.set_block(position, block)
    }
}
