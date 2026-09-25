use cgmath::{InnerSpace, Vector2, Vector3};

use crate::world::World;

pub const EYE_HEIGHT: f32 = 1.62;
pub const WALK_SPEED: f32 = 4.5;
pub const FIXED_TIMESTEP: f32 = 1.0 / 120.0;

const HALF_WIDTH: f32 = 0.3;
const HEIGHT: f32 = 1.8;
const GROUND_ACCELERATION: f32 = 35.0;
const AIR_ACCELERATION: f32 = 10.0;
const GRAVITY: f32 = 24.0;
const TERMINAL_VELOCITY: f32 = 50.0;
const JUMP_VELOCITY: f32 = 7.75;
const COLLISION_EPSILON: f32 = 0.001;

#[derive(Debug)]
pub struct Player {
    /// The center of the player's feet in world coordinates.
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub grounded: bool,
}

impl Player {
    pub fn eye_position(&self) -> Vector3<f32> {
        self.position + Vector3::new(0.0, EYE_HEIGHT, 0.0)
    }

    pub fn intersects_block(&self, block: Vector3<i32>) -> bool {
        let (min, max) = self.bounds();
        min.x < block.x as f32 + 1.0
            && max.x > block.x as f32
            && min.y < block.y as f32 + 1.0
            && max.y > block.y as f32
            && min.z < block.z as f32 + 1.0
            && max.z > block.z as f32
    }

    pub fn update(
        &mut self,
        world: &World,
        input: Vector2<f32>,
        forward: Vector3<f32>,
        right: Vector3<f32>,
        jump: bool,
        dt: f32,
    ) {
        let mut direction = forward * input.y + right * input.x;
        direction.y = 0.0;

        if direction.magnitude2() > 1.0 {
            direction = direction.normalize();
        }

        let current_horizontal = Vector2::new(self.velocity.x, self.velocity.z);
        let desired_horizontal = Vector2::new(direction.x, direction.z) * WALK_SPEED;

        let next_horizontal = if self.grounded {
            move_towards(
                current_horizontal,
                desired_horizontal,
                GROUND_ACCELERATION * dt,
            )
        } else if direction.magnitude2() > 0.0 {
            move_towards(
                current_horizontal,
                desired_horizontal,
                AIR_ACCELERATION * dt,
            )
        } else {
            current_horizontal
        };

        self.velocity.x = next_horizontal.x;
        self.velocity.z = next_horizontal.y;

        if jump && self.grounded {
            self.velocity.y = JUMP_VELOCITY;
            self.grounded = false;
        }

        self.velocity.y = (self.velocity.y - GRAVITY * dt).max(-TERMINAL_VELOCITY);
        self.grounded = false;

        self.move_axis(world, Axis::X, self.velocity.x * dt);
        self.move_axis(world, Axis::Z, self.velocity.z * dt);
        self.move_axis(world, Axis::Y, self.velocity.y * dt);
    }

    fn move_axis(&mut self, world: &World, axis: Axis, amount: f32) {
        if amount == 0.0 {
            return;
        }

        match axis {
            Axis::X => self.position.x += amount,
            Axis::Y => self.position.y += amount,
            Axis::Z => self.position.z += amount,
        }

        let (min, max) = self.bounds();
        let min_block = Vector3::new(
            min.x.floor() as i32,
            min.y.floor() as i32,
            min.z.floor() as i32,
        );
        let max_block = Vector3::new(
            (max.x - COLLISION_EPSILON).floor() as i32,
            (max.y - COLLISION_EPSILON).floor() as i32,
            (max.z - COLLISION_EPSILON).floor() as i32,
        );

        let mut collided = false;

        for x in min_block.x..=max_block.x {
            for y in min_block.y..=max_block.y {
                for z in min_block.z..=max_block.z {
                    if !world.is_solid(Vector3::new(x, y, z)) {
                        continue;
                    }

                    collided = true;

                    match (axis, amount.is_sign_positive()) {
                        (Axis::X, true) => {
                            self.position.x = self
                                .position
                                .x
                                .min(x as f32 - HALF_WIDTH - COLLISION_EPSILON);
                        }
                        (Axis::X, false) => {
                            self.position.x = self
                                .position
                                .x
                                .max(x as f32 + 1.0 + HALF_WIDTH + COLLISION_EPSILON);
                        }
                        (Axis::Y, true) => {
                            self.position.y =
                                self.position.y.min(y as f32 - HEIGHT - COLLISION_EPSILON);
                        }
                        (Axis::Y, false) => {
                            self.position.y =
                                self.position.y.max(y as f32 + 1.0 + COLLISION_EPSILON);
                            self.grounded = true;
                        }
                        (Axis::Z, true) => {
                            self.position.z = self
                                .position
                                .z
                                .min(z as f32 - HALF_WIDTH - COLLISION_EPSILON);
                        }
                        (Axis::Z, false) => {
                            self.position.z = self
                                .position
                                .z
                                .max(z as f32 + 1.0 + HALF_WIDTH + COLLISION_EPSILON);
                        }
                    }
                }
            }
        }

        if collided {
            match axis {
                Axis::X => self.velocity.x = 0.0,
                Axis::Y => self.velocity.y = 0.0,
                Axis::Z => self.velocity.z = 0.0,
            }
        }
    }

    fn bounds(&self) -> (Vector3<f32>, Vector3<f32>) {
        (
            Vector3::new(
                self.position.x - HALF_WIDTH,
                self.position.y,
                self.position.z - HALF_WIDTH,
            ),
            Vector3::new(
                self.position.x + HALF_WIDTH,
                self.position.y + HEIGHT,
                self.position.z + HALF_WIDTH,
            ),
        )
    }
}

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}

fn move_towards(current: Vector2<f32>, target: Vector2<f32>, max_delta: f32) -> Vector2<f32> {
    let delta = target - current;
    let distance = delta.magnitude();

    if distance <= max_delta || distance == 0.0 {
        target
    } else {
        current + delta / distance * max_delta
    }
}

#[cfg(test)]
mod tests {
    use cgmath::Vector3;

    use super::Player;

    #[test]
    fn detects_only_positive_block_overlap() {
        let player = Player {
            position: Vector3::new(0.5, 1.0, 0.5),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            grounded: true,
        };

        assert!(player.intersects_block(Vector3::new(0, 1, 0)));
        assert!(player.intersects_block(Vector3::new(0, 2, 0)));
        assert!(!player.intersects_block(Vector3::new(0, 0, 0)));
        assert!(!player.intersects_block(Vector3::new(1, 1, 0)));
    }
}
