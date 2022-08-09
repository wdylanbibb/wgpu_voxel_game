use crate::chunk;
use cgmath::{ElementWise, Vector2};
use paste::paste;
use std::ops::{Deref, DerefMut, Div, Mul};

macro_rules! init_blocks {
    // Creates the struct given a block of enum attributes,
    // a block of struct attributes, and struct name
    (
        @expand_meta
        ($(#[$enum_attr:meta])*)
        ($(#[$meta:meta])*)
        $vis:vis struct $name:ident;
        impl $trait:ident $impl:tt
    ) => {
        $(#[$enum_attr])*
        $(#[$meta])*
        $vis struct $name;
        impl $trait for $name $impl

    };

    // Builds a struct with attributes given to the enum and
    // the struct itself
    // (Passes attributes as tt to avoid nested repetition)
    (
        @build_struct
        $enum_attr:tt
        $(
            $meta:tt
            $vis:vis struct $name:ident;
            impl $trait:ident $impl:tt
        )*
    ) => {
        $(
            init_blocks! (
                @expand_meta
                $enum_attr
                $meta
                $vis struct $name;
                impl $trait $impl
            );
        )*
    };

    // Creates an enum containing structs that all have a certain
    // trait in common.
    (
        $(#[$enum_attr:meta])*
        $vis:vis enum $enum_name:ident: $trait:ident {
            $(
                $(#[$struct_attr:meta])*
                $name:ident: $impl:tt,
            )*
        }
    )=>{
        init_blocks! (
            @build_struct
            ($(#[$enum_attr])*)
            $(
                ($(#[$struct_attr])*)
                $vis struct $name;
                impl $trait $impl
            )*
        );

        $(#[$enum_attr])*
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
                            $enum_name::$name(v) => Some(v),
                            _ => None,
                        }
                    }
                )*
            }
        }

        impl Deref for $enum_name {
            type Target = dyn $trait;

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
    };
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
        Dirt: {
            fn texture_coordinates(&self) -> TexCoordConfig {
                TexCoordConfig::all_same(Vector2::new(32.0, 0.0))
            }
        },
        #[allow(dead_code)]
        Stone: {
            fn texture_coordinates(&self) -> TexCoordConfig {
                TexCoordConfig::all_same(Vector2::new(48.0, 0.0))
            }
        },
    }
}
