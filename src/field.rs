use std::{
    fmt::Debug,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use rand::Rng;

/// Base trait for an integer
pub trait ConstInt: PartialEq + Clone + Send + From<u64> + Debug {
    const BYTES: usize;
    fn zero() -> Self;
    fn one() -> Self;
    fn is_zero(&self) -> bool;
    fn num_bytes(&self) -> usize;
    fn to_bytes(&self, b: &mut [u8]) -> usize;
    fn from_bytes(_: &[u8]) -> Self;
    //    fn as_usize(&self) -> usize;
    //    fn pow2(_: usize) -> Self;
}

pub trait ModInt {
    fn mod_add(&self, right: &Self, m: &Self) -> Self;
    fn mod_sub(&self, right: &Self, m: &Self) -> Self;
    fn mod_mul(&self, right: &Self, m: &Self) -> Self;
    fn mod_pow(&self, exp: &Self, m: &Self) -> Self;
    fn mod_inv(&self, m: &Self) -> Option<Self>
    where
        Self: Sized;
}

// It would be nice to have an Add<&Self, Output=Self> for &Self bound as well,
// but I can't seem to figure it out
pub trait Adds: Add<Output = Self> + AddAssign + Sized + Sum
where
    for<'a> Self: Add<&'a Self, Output = Self> + AddAssign<&'a Self> + Sized + Sum<&'a Self>,
{
}

pub trait Subs: Sub<Output = Self> + SubAssign + Sized
where
    for<'a> Self: Sub<&'a Self, Output = Self> + SubAssign<&'a Self> + Sized,
{
}

pub trait Muls: Mul<Output = Self> + MulAssign + Sized + Product
where
    for<'a> Self: Mul<&'a Self, Output = Self> + MulAssign<&'a Self> + Sized + Product<&'a Self>,
{
}

pub trait Divs: Div<Output = Self> + DivAssign + Sized
//where
//    for<'a> Self: Div<&'a Self, Output = Self> + DivAssign<&'a Self> + Sized,
{
}

pub trait Ring: ConstInt + Adds + Subs + Muls {}

pub trait Field: Ring {
    fn gen() -> Self;
    fn inv(&self) -> Option<Self>;
}

/// Allows sampling an element in the set
pub trait RandElement {
    fn rand<R: Rng + ?Sized>(rng: &mut R) -> Self;
}

macro_rules! expr {
    ($x:expr) => {
        $x
    };
}
macro_rules! idx {
    ($t:expr, $idx:tt) => {
        expr!($t.$idx)
    };
}

macro_rules! impl_arith {
    ($type:ident, $trait:ident, $tf:ident, $taf:ident) => {
        impl $trait for $type {
            type Output = Self;
            fn $tf(self, other: $type) -> Self {
                let mut c = self.clone();
                c.$taf(other);
                c
            }
        }

        impl $trait<&$type> for $type {
            type Output = Self;
            fn $tf(self, other: &$type) -> Self {
                let mut c = self.clone();
                c.$taf(*other);
                c
            }
        }

        impl<'a> $trait<$type> for &'a $type {
            type Output = $type;
            fn $tf(self, other: $type) -> $type {
                let mut c = self.clone();
                c.$taf(other);
                c
            }
        }

        impl<'a, 'b> $trait<&'b $type> for &'a $type {
            type Output = $type;
            fn $tf(self, other: &'b $type) -> $type {
                let mut c = self.clone();
                c.$taf(*other);
                c
            }
        }
    };
}

macro_rules! impl_arith_assign {
    ($type:ident, $trait:ident, $tf:ident, $traitassign:ident, $taf:ident) => {
        impl $traitassign for $type {
            fn $taf(&mut self, other: $type) {
                self.$taf(other);
            }
        }

        impl $traitassign<&$type> for $type {
            fn $taf(&mut self, other: &$type) {
                self.$taf(other.clone());
            }
        }

        crate::field::impl_arith!($type, $trait, $tf, $taf);
    };
}

macro_rules! impl_sum_prod {
    ($type:ident) => {
        impl std::iter::Sum for $type {
            fn sum<I: Iterator<Item = $type>>(iter: I) -> $type {
                let mut acc = $type::zero();
                for i in iter {
                    acc += i;
                }
                acc
            }
        }
        impl<'a> std::iter::Sum<&'a $type> for $type {
            fn sum<I: Iterator<Item = &'a $type>>(iter: I) -> $type {
                let mut acc = $type::zero();
                for i in iter {
                    acc += i;
                }
                acc
            }
        }

        impl std::iter::Product for $type {
            fn product<I: Iterator<Item = $type>>(iter: I) -> $type {
                let mut acc = $type::one();
                for i in iter {
                    acc *= i;
                }
                acc
            }
        }

        impl<'a> std::iter::Product<&'a $type> for $type {
            fn product<I: Iterator<Item = &'a $type>>(iter: I) -> $type {
                let mut acc = $type::one();
                for i in iter {
                    acc *= i;
                }
                acc
            }
        }
    };
}

pub(crate) use impl_arith;
pub(crate) use impl_arith_assign;
pub(crate) use impl_sum_prod;

// Can't impl the arithmetics on tuples ourselves because of coherence rules
// so just wrap in a struct for convenience
#[repr(transparent)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct FWrap<T>(pub T);

macro_rules! impl_arith_tuple {
    ($($types:ident),+,; $trait:ident, $tf:ident, $traitassign:ident, $taf:ident) => {
        impl<$($types: $traitassign,)+> $trait for FWrap<($($types,)+)> {
            type Output = Self;
            fn $tf(self, other: Self) -> Self {
                let mut c = self;
                c.$taf(other);
                c
            }
        }

        impl<'a, $($types: $traitassign<&'a $types>,)+> $trait<&'a Self> for FWrap<($($types,)+)> {
            type Output = Self;
            fn $tf(self, other: &'a Self) -> Self {
                let mut c = self;
                c.$taf(other);
                c
            }
        }

        impl<'a, $($types: $traitassign + Clone,)+> $trait<FWrap<($($types,)+)>> for &'a FWrap<($($types,)+)> {
            type Output = FWrap<($($types,)+)>;
            fn $tf(self, other: FWrap<($($types,)+)>) -> Self::Output {
                let mut c = self.clone();
                c.$taf(other);
                c
            }
        }

        impl<'a, 'b, $($types: $traitassign<&'b $types> + Clone,)+> $trait<&'b FWrap<($($types,)+)>> for &'a FWrap<($($types,)+)> {
            type Output = FWrap<($($types,)+)>;
            fn $tf(self, other: &'b FWrap<($($types,)+)>) -> Self::Output {
                let mut c = self.clone();
                c.$taf(other);
                c
            }
        }
    };
}

macro_rules! impl_arith_assign_tuple {
    (,$($is:tt),+; $($types:ident),+,; $trait:ident, $tf:ident, $traitassign:ident, $taf:ident) => {
        impl<$($types: $traitassign,)+> $traitassign for FWrap<($($types,)+)> {
            fn $taf(&mut self, other: Self) {
                $(
                idx!(self.0, $is).$taf(idx!(other.0, $is));
                )+
            }
        }

        impl<'a, $($types: $traitassign<&'a $types>,)+> $traitassign<&'a Self> for FWrap<($($types,)+)> {
            fn $taf(&mut self, other: &'a Self) {
                $(
                idx!(self.0, $is).$taf(&idx!(other.0, $is));
                )+
            }
        }

        impl_arith_tuple!($($types,)+; $trait, $tf, $traitassign, $taf);
    };
}

macro_rules! impl_sum_prod_tuple {
    ($($types:ident),+,) => {
        impl<$($types,)+> std::iter::Sum for FWrap<($($types,)+)>
            where Self: ConstInt + AddAssign
        {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                let mut acc = Self::zero();
                for i in iter {
                    acc += i;
                }
                acc
            }
        }
        impl<'a, $($types: AddAssign<&'a $types>,)+> std::iter::Sum<&'a Self> for FWrap<($($types,)+)>
            where Self: ConstInt + AddAssign
        {
            fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                let mut acc = Self::zero();
                for i in iter {
                    // TODO: cloning to appease the borrow checker about "type T will meet its required lifetime bounds"
                    acc += i.clone();
                }
                acc
            }
        }

        impl<$($types,)+> std::iter::Product for FWrap<($($types,)+)>
            where Self: ConstInt + MulAssign
        {
            fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                let mut acc = Self::one();
                for i in iter {
                    acc *= i;
                }
                acc
            }
        }


        impl<'a, $($types: MulAssign<&'a $types>,)+> std::iter::Product<&'a Self> for FWrap<($($types,)+)>
            where Self: ConstInt + MulAssign
        {
            fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                let mut acc = Self::one();
                for i in iter {
                    // TODO: cloning to appease the borrow checker about "type T will meet its required lifetime bounds"
                    acc *= i.clone();
                }
                acc
            }
        }
    };
}

macro_rules! impl_const_int_tuple {
    (,$($is: tt),+; $($types:ident),+,) => {
        impl<$($types: From<u64>,)+> From<u64> for FWrap<($($types,)+)> {
            fn from(other: u64) -> Self {
                return FWrap(($($types::from(other),)+));
            }
        }

        impl<$($types: ConstInt,)+> ConstInt for FWrap<($($types,)+)> {
            const BYTES: usize = {$($types::BYTES +)+ 0};
            fn zero() -> Self {
                FWrap(($($types::zero(),)+))
            }
            fn one() -> Self {
                FWrap(($($types::one(),)+))
            }
            fn is_zero(&self) -> bool {
                $(idx!(self.0, $is).is_zero() &&)+ true
            }
            fn num_bytes(&self) -> usize {
                Self::BYTES
            }
            fn to_bytes(&self, b: &mut [u8]) -> usize {
                assert!(b.len() >= Self::BYTES);

                let mut start = 0;

                $(
                    idx!(self.0, $is).to_bytes(&mut b[start..]);
                    start += $types::BYTES;
                )+

                start
            }
            fn from_bytes(b: &[u8]) -> Self {
                let mut starts = Vec::new();
                starts.push(0);
                $(
                starts.push(starts.last().unwrap() + $types::BYTES);
                )+

                FWrap(($(
                    $types::from_bytes(&b[starts[$is]..]),
                )+))
            }
        }
    }
}

macro_rules! impl_vectorized_arith {
    ($($is: tt),+; $($types:ident),+) => {
        impl_const_int_tuple!($(,$is)+; $($types,)+);
        impl_arith_assign_tuple!($(,$is)+; $($types,)+; Add, add, AddAssign, add_assign);
        impl_arith_assign_tuple!($(,$is)+; $($types,)+; Sub, sub, SubAssign, sub_assign);
        impl_arith_assign_tuple!($(,$is)+; $($types,)+; Mul, mul, MulAssign, mul_assign);
        impl_sum_prod_tuple!($($types,)+);

        impl<$($types: Adds,)+> Adds for FWrap<($($types,)+)>
            where for<'a> Self: ConstInt + AddAssign + AddAssign<&'a Self> {}
        impl<$($types: Subs,)+> Subs for FWrap<($($types,)+)> {}
        impl<$($types: ConstInt + Muls,)+> Muls for FWrap<($($types,)+)> {}
        impl<$($types: Ring,)+> Ring for FWrap<($($types,)+)> {}

        impl<$($types: RandElement,)+> RandElement for FWrap<($($types,)+)> {
            fn rand<R: Rng + ?Sized>(rng: &mut R) -> Self {
                FWrap(($(
                    $types::rand(rng),
                )+))
            }
        }
    }
}

impl_vectorized_arith!(0; T0);
impl_vectorized_arith!(0, 1; T0, T1);
impl_vectorized_arith!(0, 1, 2; T0, T1, T2);
impl_vectorized_arith!(0, 1, 2, 3; T0, T1, T2, T3);
