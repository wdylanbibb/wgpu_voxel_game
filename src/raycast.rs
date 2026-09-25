use cgmath::InnerSpace;

use crate::{
    camera::Ray,
    world::{World, block::BlockId},
};

const DIRECTION_EPSILON: f32 = 1.0e-6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    pub block_position: cgmath::Vector3<i32>,
    pub place_position: Option<cgmath::Vector3<i32>>,
    pub face_normal: cgmath::Vector3<i32>,
    pub block: BlockId,
}

fn axis_step(direction: f32) -> i32 {
    if direction > DIRECTION_EPSILON {
        1
    } else if direction < -DIRECTION_EPSILON {
        -1
    } else {
        0
    }
}

fn axis_delta(direction: f32) -> f32 {
    if direction.abs() > DIRECTION_EPSILON {
        1.0 / direction.abs()
    } else {
        f32::INFINITY
    }
}

fn axis_initial_distance(origin: f32, cell: i32, direction: f32, step: i32) -> f32 {
    match step {
        1 => ((cell + 1) as f32 - origin) / direction,
        -1 => (origin - cell as f32) / -direction,
        _ => f32::INFINITY,
    }
}

pub fn raycast(world: &World, ray: Ray, max_distance: f32) -> Option<RayHit> {
    if !max_distance.is_finite() || max_distance < 0.0 {
        return None;
    }

    if !ray.origin.x.is_finite()
        || !ray.origin.y.is_finite()
        || !ray.origin.z.is_finite()
        || !ray.direction.x.is_finite()
        || !ray.direction.y.is_finite()
        || !ray.direction.z.is_finite()
    {
        return None;
    }

    if ray.direction.magnitude2() <= DIRECTION_EPSILON * DIRECTION_EPSILON {
        return None;
    }

    let direction = ray.direction.normalize();

    let mut cell = cgmath::Vector3::new(
        ray.origin.x.floor() as i32,
        ray.origin.y.floor() as i32,
        ray.origin.z.floor() as i32,
    );

    if let Some(block) = world.get_block(cell)
        && block.solid()
    {
        return Some(RayHit {
            block_position: cell,
            place_position: None,
            face_normal: cgmath::Vector3::new(0, 0, 0),
            block,
        });
    }

    let step = cgmath::Vector3::new(
        axis_step(direction.x),
        axis_step(direction.y),
        axis_step(direction.z),
    );

    let delta = cgmath::Vector3::new(
        axis_delta(direction.x),
        axis_delta(direction.y),
        axis_delta(direction.z),
    );

    let mut next_boundary = cgmath::Vector3::new(
        axis_initial_distance(ray.origin.x, cell.x, direction.x, step.x),
        axis_initial_distance(ray.origin.y, cell.y, direction.y, step.y),
        axis_initial_distance(ray.origin.z, cell.z, direction.z, step.z),
    );

    loop {
        let previous_cell = cell;
        let distance;
        let face_normal;

        if next_boundary.x <= next_boundary.y && next_boundary.x <= next_boundary.z {
            distance = next_boundary.x;

            if distance > max_distance {
                return None;
            }

            cell.x += step.x;
            next_boundary.x += delta.x;
            face_normal = cgmath::Vector3::new(-step.x, 0, 0);
        } else if next_boundary.y <= next_boundary.z {
            distance = next_boundary.y;

            if distance > max_distance {
                return None;
            }

            cell.y += step.y;
            next_boundary.y += delta.y;
            face_normal = cgmath::Vector3::new(0, -step.y, 0);
        } else {
            distance = next_boundary.z;

            if distance > max_distance {
                return None;
            }

            cell.z += step.z;
            next_boundary.z += delta.z;
            face_normal = cgmath::Vector3::new(0, 0, -step.z);
        }

        if let Some(block) = world.get_block(cell)
            && block.solid()
        {
            return Some(RayHit {
                block_position: cell,
                place_position: Some(previous_cell),
                face_normal,
                block,
            });
        }
    }
}
