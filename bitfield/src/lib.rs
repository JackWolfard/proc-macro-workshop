// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.
use bitfield_impl::bit_specifier;
pub use bitfield_impl::bitfield;

pub trait Specifier {
    const BITS: usize;
}

pub mod checks {
    use bitfield_impl::multiple_of_8;
    use std::marker::PhantomData;

    pub trait KnownSize {
        type Check;
    }

    pub struct TotalSize<T>(PhantomData<T>);

    multiple_of_8!(0, ZeroMod8);
    multiple_of_8!(1, OneMod8);
    multiple_of_8!(2, TwoMod8);
    multiple_of_8!(3, ThreeMod8);
    multiple_of_8!(4, FourMod8);
    multiple_of_8!(5, FiveMod8);
    multiple_of_8!(6, SixMod8);
    multiple_of_8!(7, SevenMod8);

    pub trait TotalSizeIsMultipleOf8 {}

    impl TotalSizeIsMultipleOf8 for ZeroMod8 {}

    pub trait CheckTotalSizeIsMultipleOf8
    where
        <Self::Size as KnownSize>::Check: TotalSizeIsMultipleOf8,
    {
        type Size: KnownSize;
    }
}

pub enum Zero {}

impl Specifier for Zero {
    const BITS: usize = 0;
}

bit_specifier!(1..64);
