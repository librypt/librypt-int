pub use bitint_macro::bituint;
use paste::paste;

/// Fixed bit integers

macro_rules! define_multiple_uints {
    () => {};

    ($x:literal) => {
        paste! {
            #[allow(non_camel_case_types)]
            #[bituint($x)]
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
            pub struct [<u $x>];
        }
    };

    ($x:literal, $($xs:literal),*) => {
        paste! {
            #[allow(non_camel_case_types)]
            #[bituint($x)]
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
            pub struct [<u $x>];
        }

        define_multiple_uints!($($xs),*);
    };
}

define_multiple_uints!(24, 48, 80, 256, 512, 1024, 2048, 4096);

#[cfg(test)]
mod tests {
    use crate::u24;
    #[test]
    fn test() {
        assert_eq!(u24::from(5) + u24::from(251), u24::from(256));
        assert_eq!(u24::from(257) - u24::from(251), u24::from(6));
        assert_eq!(u128::from(u24::from(257)), 257);
    }
}
