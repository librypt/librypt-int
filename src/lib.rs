use bitint_macro::bituint;
use paste::paste;

macro_rules! define_multiple_uints {
    () => {};

    ($x:literal) => {
        paste! {
            #[allow(non_camel_case_types)]
            #[bituint($x)]
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
            pub struct [<u $x>];
        }
    };

    ($x:literal, $($xs:literal),*) => {
        paste! {
            #[allow(non_camel_case_types)]
            #[bituint($x)]
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
            pub struct [<u $x>];
        }

        define_multiple_uints!($($xs),*);
    };
}

define_multiple_uints!(24, 48, 80, 256, 512, 1024, 2048, 4096);
