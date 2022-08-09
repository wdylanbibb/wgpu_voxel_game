use crate::chunk;
use cgmath::{ElementWise, Vector2};
use paste::paste;
use std::ops::{Deref, DerefMut, Div, Mul};

macro_rules! init_blocks {
	($vis:vis enum $enum_name:ident {
		$(
			$name:ident = $tex_coords:expr,
		)*
	}) => {
		$(
			#[derive(Debug, Clone, Copy, PartialEq, Eq)]
			$vis struct $name;
            impl BlockData for $name {
                fn texture_coordinates(&self) -> TexCoordConfig {
                    $tex_coords
                }
            }
		)*

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $enum_name {
            $(
                $name($name),
            )*
        }

        paste! {
            impl $enum_name {
                $(
                    $vis fn [<$name:lower>]() -> Self {
                        $enum_name::$name($name)
                    }

                    $vis fn [<as_ $name:lower>](self) -> Option<$name> {
                        match self {
                            $enum_name::$name(block) => Some(block),
                            _ => None,
                        }
                    }
                )*
            }
        }

        impl Deref for $enum_name {
            type Target = dyn BlockData;

            fn deref(&self) -> &Self::Target {
                match self {
                    $(
                        $enum_name::$name(v) => v,
                    )*
                }
            }
        }

        impl DerefMut for $enum_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                match self {
                    $(
                        $enum_name::$name(v) => v,
                    )*
                }
            }
        }
	}
}

pub struct TexCoordConfig {
    pub front: Vector2<f32>,
    pub back: Vector2<f32>,
    pub top: Vector2<f32>,
    pub bottom: Vector2<f32>,
    pub left: Vector2<f32>,
    pub right: Vector2<f32>,
}

impl TexCoordConfig {
    pub fn new(
        top: Vector2<f32>,
        bottom: Vector2<f32>,
        left: Vector2<f32>,
        right: Vector2<f32>,
        front: Vector2<f32>,
        back: Vector2<f32>,
    ) -> Self {
        Self {
            front,
            back,
            top,
            bottom,
            left,
            right,
        }
    }

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

init_blocks! {
    pub enum Block {
        Air = TexCoordConfig::zero(),
        Grass = TexCoordConfig::top_bottom_sides(Vector2::new(0.0, 0.0), Vector2::new(32.0, 0.0), Vector2::new(16.0, 0.0)),
        Dirt = TexCoordConfig::all_same(Vector2::new(32.0, 0.0)),
        Stone = TexCoordConfig::all_same(Vector2::new(48.0, 0.0)),
    }
}
