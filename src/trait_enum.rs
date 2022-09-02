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

    // Creates an enum containing traits that all implement a given trait.
    (
        $(#[$enum_attr:meta])*
        $vis:vis enum $enum_name:ident {
            $(
                $(#[$struct_attr:meta])*
                $name:ident = $trait:ident {
                    $($impl:item)*
                }
            ),* $(,)?
        }
    ) => {
        trait_enum! (
            @build_struct
            ($(#[$enum_attr])*)
            $(
                ($(#[$struct_attr])*)
                $vis struct $name;
                impl $trait {
                    $($impl)*
                }
            )*
        );

        $(#[$enum_attr])*
        $vis enum $enum_name {
            $(
                $(#[$struct_attr])*
                $name($name),
            )*
        }
        paste::paste! {
            impl $enum_name {
                $(
                    $vis fn [<$name:lower>]() -> Self {
                        $enum_name::$name($name)
                    }

                    // Not needed because of get_inner<T>()
                    // $vis fn [<as_ $name:lower>](&self) -> Option<&$name> {
                    //     match self {
                    //         $enum_name::$name(v) => Some(&v),
                    //         _ => None,
                    //     }
                    // }
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
                $name:ident: {
                    $($impl:item)*
                }
            ),* $(,)?
        }
    )=>{
        pub trait WithAny: $trait {
            fn as_any(&self) -> &dyn std::any::Any;
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
        }

        trait_enum! (
            $(#[$enum_attr])*
            $vis enum $enum_name {
                $(
                    $(#[$struct_attr])*
                    $name = $trait {
                        $($impl)*
                    }
                ),*
            }
        );

        $(
            impl WithAny for $name {
                fn as_any(&self) -> &dyn std::any::Any {
                    self
                }

                fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                    self
                }
            }
        )*

        impl $enum_name {
            $vis fn get_inner<T>(&self) -> Option<&T>
                where T: WithAny + 'static
            {
                self.deref().as_any().downcast_ref::<T>()
            }

            $vis fn get_inner_mut<T>(&mut self) -> Option<&mut T>
                where T: WithAny + 'static
            {
                self.deref_mut().as_any_mut().downcast_mut::<T>()
            }
        }

        impl std::ops::Deref for $enum_name {
            type Target = dyn WithAny;

            fn deref(&self) -> &Self::Target {
                match self {
                    $(
                        $enum_name::$name(v) => v,
                    )*
                }
            }
        }

        impl std::ops::DerefMut for $enum_name {
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
