use paste::paste;

#[macro_export]
macro_rules! trait_enum {
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
            trait_enum! (
                @expand_meta
                $enum_attr
                $meta
                $vis struct $name;
                impl $trait $impl
            );
        )*
    };

    (
        $(#[$enum_attr:meta])*
        $vis:vis enum $enum_name:ident {
            $(
                $(#[$struct_attr:meta])*
                $name:ident = $trait:ident $impl:tt
            ),* $(,)?
        }
    ) => {
        trait_enum! (
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
                $(#[$struct_attr])*
                $name($name),
            )*
        }

        paste! {
            impl $enum_name {
                $(
                    $vis fn [<$name:lower>]() -> Self {
                        $enum_name::$name($name)
                    }

                    $vis fn [<as_ $name:lower>](&self) -> Option<&$name> {
                        match self {
                            $enum_name::$name(v) => Some(&v),
                            _ => None,
                        }
                    }
                )*
            }
        }
    };

    // Creates an enum containing structs that all have a certain
    // trait in common.
    (
        $(#[$enum_attr:meta])*
        $vis:vis enum $enum_name:ident: $trait:ident {
            $(
                $(#[$struct_attr:meta])*
                $name:ident: $impl:tt
            ),* $(,)?
        }
    )=>{
        trait_enum! (
            $(#[$enum_attr])*
            $vis enum $enum_name {
                $(
                    $(#[$struct_attr])*
                    $name = $trait $impl
                ),*
            }
        );

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

#[cfg(test)]
mod tests {
    use paste::paste;

    trait Animal {
        fn make_sound(&self) -> &str;
    }

    #[test]
    fn default_behavior() {
        trait_enum! (
            #[derive(Debug)]
            enum Animals {
                Dog = Animal {
                    fn make_sound(&self) -> &str {
                        "woof"
                    }
                },
                Cat = Animal {
                    fn make_sound(&self) -> &str {
                        "meow"
                    }
                },
            }
        );

        let dog: Animals = Animals::dog();

        if let Some(dog) = dog.as_dog() {
            assert_eq!(dog.make_sound(), "woof");
        } else {
            panic!("Expected Dog");
        }
    }

    #[test]
    fn deref() {
        use std::ops::{Deref, DerefMut};

        trait_enum! (
            #[derive(Debug)]
            enum Animals: Animal {
                /// A dog
                Dog: {
                    fn make_sound(&self) -> &str {
                        "woof"
                    }
                },
                /// A cat
                Cat: {
                    fn make_sound(&self) -> &str {
                        "meow"
                    }
                },
            }
        );

        let dog: Animals = Animals::dog();

        let deref: &dyn Animal = dog.deref();

        assert_eq!(deref.make_sound(), "woof");
    }
}
