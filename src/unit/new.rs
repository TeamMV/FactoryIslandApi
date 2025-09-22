#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

pub trait NewQuantity {
    /// Used in internal function implementations, use `Quantity::new` instead of this
    fn new_quantity(value: f64) -> Self;
}

pub trait SumType<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8, const D2: i8, const T2: i8, const M2: i8, const K2: i8, const I2: i8, const N2: i8, const DR: i8, const TR: i8, const MR: i8, const KR: i8, const IR: i8, const NR: i8> {
    type SumType: NewQuantity;
}

pub trait DiffType<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8, const D2: i8, const T2: i8, const M2: i8, const K2: i8, const I2: i8, const N2: i8, const DR: i8, const TR: i8, const MR: i8, const KR: i8, const IR: i8, const NR: i8> {
    type DiffType: NewQuantity;
}

pub trait True {}
impl True for [(); 1] {}

pub struct Divisible<const A: i8, const B: i8>;
impl<const A: i8, const B: i8> True for Divisible<A, B> where [(); (A % B == 0) as usize ]: True {}

// These will most likely never be needed, but if they ever are they exist :)
pub struct Equal<const A: i8, const B: i8>;
impl<const A: i8, const B: i8> True for Equal<A, B> where [(); (A == B) as usize ]: True {}

pub struct NotEqual<const A: i8, const B: i8>;
impl<const A: i8, const B: i8> True for NotEqual<A, B> where [(); (A != B) as usize ]: True {}

pub struct Greater<const A: i8, const B: i8>;
impl<const A: i8, const B: i8> True for Greater<A, B> where [(); (A > B) as usize ]: True {}

pub struct Less<const A: i8, const B: i8>;
impl<const A: i8, const B: i8> True for Less<A, B> where [(); (A < B) as usize ]: True {}

pub struct GreaterOrEqual<const A: i8, const B: i8>;
impl<const A: i8, const B: i8> True for GreaterOrEqual<A, B> where [(); (A >= B) as usize ]: True {}

pub struct LessOrEqual<const A: i8, const B: i8>;
impl<const A: i8, const B: i8> True for LessOrEqual<A, B> where [(); (A <= B) as usize ]: True {}

pub type Dimension = [i8; 6];

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Quantity<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> {
    pub value: f64,
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> NewQuantity for Quantity<D, T, M, K, I, N> {
    fn new_quantity(value: f64) -> Self {
        Quantity { value }
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8, const D2: i8, const T2: i8, const M2: i8, const K2: i8, const I2: i8, const N2: i8> SumType<D, T, M, K, I, N, D2, T2, M2, K2, I2, N2, { D + D2 }, { T + T2 }, { M + M2 }, { K + K2 }, { I + I2 }, { N + N2 }> for Quantity<D, T, M, K, I, N> {
    type SumType = Quantity<{ D + D2 }, { T + T2 }, { M + M2 }, { K + K2 }, { I + I2 }, { N + N2 }>;
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8, const D2: i8, const T2: i8, const M2: i8, const K2: i8, const I2: i8, const N2: i8> DiffType<D, T, M, K, I, N, D2, T2, M2, K2, I2, N2, { D - D2 }, { T - T2 }, { M - M2 }, { K - K2 }, { I - I2 }, { N - N2 }> for Quantity<D, T, M, K, I, N> {
    type DiffType = Quantity<{ D - D2 }, { T - T2 }, { M - M2 }, { K - K2 }, { I - I2 }, { N - N2 }>;
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8>
    Quantity<D, T, M, K, I, N>
{
    pub const fn dimension() -> Dimension {
        [D, T, M, K, I, N]
    }

    pub fn new(value: f64) -> Quantity<D, T, M, K, I, N> {
        Quantity { value }
    }

    pub fn value(&self) -> f64 { self.value }

    pub fn set_value(&mut self, value: f64) { self.value = value }

    pub fn powi<const P: i8>(self) -> Quantity<{ D * P }, { T * P }, { M * P }, { K * P }, { I * P }, { N * P }> {
        Quantity::new(self.value.powi(P as i32))
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8>
Quantity<D, T, M, K, I, N> where
    Divisible<D, 2>: True,
    Divisible<T, 2>: True,
    Divisible<M, 2>: True,
    Divisible<K, 2>: True,
    Divisible<I, 2>: True,
    Divisible<N, 2>: True
{
    pub fn sqrt(self) -> Quantity<{ D / 2 }, { T / 2 }, { M / 2 }, { K / 2 }, { I / 2 }, { N / 2 }> {
        Quantity::new(self.value.sqrt())
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8>
Quantity<D, T, M, K, I, N> where
    Divisible<D, 3>: True,
    Divisible<T, 3>: True,
    Divisible<M, 3>: True,
    Divisible<K, 3>: True,
    Divisible<I, 3>: True,
    Divisible<N, 3>: True
{
    pub fn cbrt(self) -> Quantity<{ D / 3 }, { T / 3 }, { M / 3 }, { K / 3 }, { I / 3 }, { N / 3 }> {
        Quantity::new(self.value.cbrt())
    }
}


impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> Add for Quantity<D, T, M, K, I, N> {
    type Output = Quantity<D, T, M, K, I, N>;

    fn add(self, rhs: Quantity<D, T, M, K, I, N>) -> Self::Output {
        Quantity::new(self.value + rhs.value)
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> AddAssign for Quantity<D, T, M, K, I, N> {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> Sub for Quantity<D, T, M, K, I, N> {
    type Output = Quantity<D, T, M, K, I, N>;

    fn sub(self, rhs: Quantity<D, T, M, K, I, N>) -> Self::Output {
        Quantity::new(self.value - rhs.value)
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> SubAssign for Quantity<D, T, M, K, I, N> {
    fn sub_assign(&mut self, rhs: Self) {
        self.value -= rhs.value;
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> Neg for Quantity<D, T, M, K, I, N> {
    type Output = Quantity<D, T, M, K, I, N>;

    fn neg(self) -> Self::Output {
        Quantity::new(-self.value)
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8, const D2: i8, const T2: i8, const M2: i8, const K2: i8, const I2: i8, const N2: i8> Mul<Quantity<D2, T2, M2, K2, I2, N2>> for Quantity<D, T, M, K, I, N> where Quantity<D, T, M, K, I, N>: SumType<D, T, M, K, I, N, D2, T2, M2, K2, I2, N2, { D + D2 }, { T + T2 }, { M + M2 }, { K + K2 }, { I + I2 }, { N + N2 }> {
    type Output = <Quantity<D, T, M, K, I, N> as SumType<D, T, M, K, I, N, D2, T2, M2, K2, I2, N2, { D + D2 }, { T + T2 }, { M + M2 }, { K + K2 }, { I + I2 }, { N + N2 }>>::SumType;

    fn mul(self, rhs: Quantity<D2, T2, M2, K2, I2, N2>) -> Self::Output {
        Self::Output::new_quantity(self.value * rhs.value)
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> Mul<f64> for Quantity<D, T, M, K, I, N> {
    type Output = Quantity<D, T, M, K, I, N>;

    fn mul(self, rhs: f64) -> Self::Output {
        Quantity::new(self.value * rhs)
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> Mul<Quantity<D, T, M, K, I, N>> for f64 {
    type Output = Quantity<D, T, M, K, I, N>;

    fn mul(self, rhs: Quantity<D, T, M, K, I, N>) -> Self::Output {
        Quantity::new(self * rhs.value)
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8, const D2: i8, const T2: i8, const M2: i8, const K2: i8, const I2: i8, const N2: i8> Div<Quantity<D2, T2, M2, K2, I2, N2>> for Quantity<D, T, M, K, I, N> where Quantity<D, T, M, K, I, N>: DiffType<D, T, M, K, I, N, D2, T2, M2, K2, I2, N2, { D - D2 }, { T - T2 }, { M - M2 }, { K - K2 }, { I - I2 }, { N - N2 }> {
    type Output = <Quantity<D, T, M, K, I, N> as DiffType<D, T, M, K, I, N, D2, T2, M2, K2, I2, N2, { D - D2 }, { T - T2 }, { M - M2 }, { K - K2 }, { I - I2 }, { N - N2 }>>::DiffType;

    fn div(self, rhs: Quantity<D2, T2, M2, K2, I2, N2>) -> Self::Output {
        Self::Output::new_quantity(self.value / rhs.value)
    }
}

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> Div<f64> for Quantity<D, T, M, K, I, N> {
    type Output = Quantity<D, T, M, K, I, N>;

    fn div(self, rhs: f64) -> Self::Output {
        Quantity::new(self.value / rhs)
    }
}

macro_rules! ty_expr {
    ($t:ty) => {
        $t
    };

    ($lhs:ty * $rhs:ty) => {
        <$lhs as std::ops::Mul<$rhs>>::Output
    };
    ($lhs:ty / $rhs:ty) => {
        <$lhs as std::ops::Div<$rhs>>::Output
    };

    (($($lhs:tt)+) * $rhs:ty) => {
        <ty_expr!($($lhs)*) as std::ops::Mul<$rhs>>::Output
    };
    ($lhs:ty * $($rhs:ty)+) => {
        <$lhs as std::ops::Mul<ty_expr!($($rhs)+)>>::Output
    };
    (($($lhs:tt)+) * $($rhs:ty)+) => {
        <ty_expr!($($lhs)+) as std::ops::Mul<ty_expr!($($rhs)+)>>::Output
    };

    (($($lhs:tt)+) / $rhs:ty) => {
        <ty_expr!($($lhs)*) as std::ops::Div<$rhs>>::Output
    };
    ($lhs:ty / $($rhs:ty)+) => {
        <$lhs as std::ops::Div<ty_expr!($($rhs)+)>>::Output
    };
    (($($lhs:tt)+) / $($rhs:ty)+) => {
        <ty_expr!($($lhs)+) as std::ops::Div<ty_expr!($($rhs)+)>>::Output
    };
}

pub type Distance = Quantity<1, 0, 0, 0, 0, 0>;
pub type Time = Quantity<0, 1, 0, 0, 0, 0>;
pub type Mass = Quantity<0, 0, 1, 0, 0, 0>;
pub type Temperature = Quantity<0, 0, 0, 1, 0, 0>;
pub type Current = Quantity<0, 0, 0, 0, 1, 0>;
pub type Amount = Quantity<0, 0, 0, 0, 0, 1>;

// pub type Velocity = Quantity<1, -1, 0, 0, 0, 0>;
// pub type Acceleration = Quantity<1, -2, 0, 0, 0, 0>;
// pub type Newtons = Quantity<1, -2, 1, 0, 0, 0>;
// pub type Joules = Quantity<2, -2, 1, 0, 0, 0>;
// pub type Watts = Quantity<2, -3, 1, 0, 0, 0>;
// pub type Volts = Quantity<2, -3, 1, 0, -1, 0>;

pub type Velocity = ty_expr!(Distance / Time);
pub type Acceleration = ty_expr!(Velocity / Time);
pub type Force = ty_expr!(Acceleration * Mass);
pub type Energy = ty_expr!(Force * Distance);
pub type Power = ty_expr!(Energy / Time);
pub type Voltage = ty_expr!(Power / Current);

pub type Volts2 = ty_expr!((((((Distance / Time) / Time) * Mass) * Distance) / Time) / Current);

pub struct ExampleMachine {
    _power: Power,
}

fn main() {
    let energy: Energy = Energy::new(50.0);
    let seconds: Time = Time::new(10.0);
    let power: Power = energy / seconds;

    let seconds = Time::new(2.0);
    let seconds_squared = seconds.powi::<2>();

    seconds_squared.sqrt();

    let _machine = ExampleMachine {
        _power: power,
    };
}
