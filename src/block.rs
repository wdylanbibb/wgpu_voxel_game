use std::ops::{Deref, DerefMut, Div, Mul};

use cgmath::{ElementWise, Vector2};

use crate::{chunk, trait_enum};

pub struct TexCoordConfig {
    pub front: Vector2<f32>,
    pub back: Vector2<f32>,
    pub top: Vector2<f32>,
    pub bottom: Vector2<f32>,
    pub left: Vector2<f32>,
    pub right: Vector2<f32>,
}

impl TexCoordConfig {
    pub fn all_same(value: Vector2<f32>) -> Self {
        Self {
            front: value,
            back: value,
            top: value,
            bottom: value,
            left: value,
            right: value,
        }
    }

    pub fn top_bottom_sides(top: Vector2<f32>, bottom: Vector2<f32>, sides: Vector2<f32>) -> Self {
        Self {
            front: sides,
            back: sides,
            top,
            bottom,
            left: sides,
            right: sides,
        }
    }

    pub fn zero() -> Self {
        Self {
            front: Vector2::new(0.0, 0.0),
            back: Vector2::new(0.0, 0.0),
            top: Vector2::new(0.0, 0.0),
            bottom: Vector2::new(0.0, 0.0),
            left: Vector2::new(0.0, 0.0),
            right: Vector2::new(0.0, 0.0),
        }
    }

    pub fn to_vec(&self) -> Vec<Vector2<f32>> {
        fn transform(origin: Vector2<f32>, coord: Vector2<f32>) -> Vector2<f32> {
            origin
                .add_element_wise(coord.mul(chunk::TEXTURE_SIZE as f32))
                .div(chunk::ATLAS_SIZE as f32)
        }

        let faces = [
            self.front,
            self.back,
            self.top,
            self.bottom,
            self.left,
            self.right,
        ];

        faces
            .iter()
            .enumerate()
            .map(|(i, face)| {
                let mut result = [
                    transform(*face, Vector2::new(0.0, 1.0)),
                    transform(*face, Vector2::new(1.0, 1.0)),
                    transform(*face, Vector2::new(1.0, 0.0)),
                    transform(*face, Vector2::new(0.0, 0.0)),
                ];

                if i % 2 == 0 {
                    result.swap(0, 1);
                    result.swap(2, 3);
                }

                result
            })
            .flatten()
            .collect::<Vec<_>>()
    }
}

pub trait BlockData {
    fn texture_coordinates(&self) -> TexCoordConfig;
}

trait_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Block: BlockData {
        Air: {
            fn texture_coordinates(&self) -> TexCoordConfig {
                TexCoordConfig::zero()
            }
        },
        Grass: {
            fn texture_coordinates(&self) -> TexCoordConfig {
                TexCoordConfig::top_bottom_sides(Vector2::new(0.0, 0.0), Vector2::new(32.0, 0.0), Vector2::new(16.0, 0.0))
            }
        },
        #[allow(dead_code)]
        Stone: {
            fn texture_coordinates(&self) -> TexCoordConfig {
                TexCoordConfig::all_same(Vector2::new(48.0, 0.0))
            }
        }
    }
}
