use std::f32;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Mul, MulAssign};

/// For some operations, the weight set associated to a wFST must have the structure of a semiring.
/// `(S, +, *, 0, 1)` is a semiring if `(S, +, 0)` is a commutative monoid with identity element 0,
/// `(S, *, 1)` is a monoid with identity element `1`, `*` distributes over `+`,
/// `0` is an annihilator for `*`.
/// Thus, a semiring is a ring that may lack negation.
/// For more information : https://cs.nyu.edu/~mohri/pub/hwa.pdf
pub trait Semiring:
    Clone + PartialEq + PartialOrd + Debug + Default + Add + AddAssign + Mul + MulAssign + Display
{
    type Type: Display;

    fn zero() -> Self;
    fn one() -> Self;

    fn new(value: Self::Type) -> Self;
    fn plus(&self, rhs: &Self) -> Self {
        let mut w = self.clone();
        w.plus_mut(rhs);
        w
    }
    fn plus_mut(&mut self, rhs: &Self);
    fn times(&self, rhs: &Self) -> Self {
        let mut w = self.clone();
        w.times_mut(rhs);
        w
    }
    fn times_mut(&mut self, rhs: &Self);
    fn value(&self) -> Self::Type;
    fn set_value(&mut self, value: Self::Type);
    fn is_one(&self) -> bool {
        *self == Self::one()
    }
    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

/// A semiring is said to be divisible if all non-0 elements admit an inverse,
/// that is if `S-{0}` is a group.
/// `(S, +, *, 0, 1)` is said to be weakly divisible if
/// for any `x` and `y` in `S` such that `x + y != 0`,
/// there exists at least one `z` such that `x = (x+y)*z`.
/// For more information : `https://cs.nyu.edu/~mohri/pub/hwa.pdf`
pub trait WeaklyDivisibleSemiring: Semiring {
    /// Inverse for the * operation
    fn inverse(&self) -> Self {
        let mut w = self.clone();
        w.inverse_mut();
        w
    }
    fn inverse_mut(&mut self);
    // TODO : Not always commutative
    fn divide(&self, rhs: &Self) -> Self;
}

/// A semiring `(S, ⊕, ⊗, 0, 1)` is said to be complete if for any index set `I` and any family
/// `(ai)i ∈ I` of elements of `S`, `⊕(ai)i∈I` is an element of `S` whose definition
/// does not depend on the order of the terms in the ⊕-sum.
/// Note that in a complete semiring all weighted transducers are regulated since all
/// infinite sums are elements of S.
/// For more information : `https://cs.nyu.edu/~mohri/pub/hwa.pdf`
pub trait CompleteSemiring: Semiring {}

/// A complete semiring S is a starsemiring that is a semiring that can be augmented with an
/// internal unary closure operation ∗ defined by `a∗=⊕an (infinite sum) for any a ∈ S`.
/// Furthermore, associativity, commutativity, and distributivity apply to these infinite sums.
/// For more information : `https://cs.nyu.edu/~mohri/pub/hwa.pdf`
pub trait StarSemiring: Semiring {
    fn closure(&self) -> Self;
}

pub trait WeightQuantize: Semiring<Type = f32> {
    fn quantize(&self, delta: f32) -> Self {
        let v = self.value();
        if v == f32::INFINITY || v == f32::NEG_INFINITY {
            return self.clone();
        }
        Self::new(((v / delta) + 0.5).floor() * delta)
    }
}

macro_rules! add_mul_semiring {
    ($semiring:ty) => {
        impl Add for $semiring {
            type Output = $semiring;

            fn add(self, other: $semiring) -> $semiring {
                self.plus(&other)
            }
        }

        impl AddAssign for $semiring {
            fn add_assign(&mut self, other: $semiring) {
                let new_value = self.plus(&other).value();
                self.set_value(new_value);
            }
        }

        impl Mul for $semiring {
            type Output = $semiring;

            fn mul(self, other: $semiring) -> $semiring {
                self.times(&other)
            }
        }

        impl MulAssign for $semiring {
            fn mul_assign(&mut self, other: $semiring) {
                let new_value = self.times(&other).value();
                self.set_value(new_value)
            }
        }
    };
}

macro_rules! display_semiring {
    ($semiring:tt) => {
        use std::fmt;
        impl fmt::Display for $semiring {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.value())?;
                Ok(())
            }
        }
    };
}

macro_rules! partial_eq_f32 {
    ($semiring:tt) => {
        impl PartialEq for $semiring {
            fn eq(&self, other: &Self) -> bool {
                self.quantize(KDELTA).value() == other.quantize(KDELTA).value()
            }
        }
    };
}
