pub mod format;

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use fi_proc::unit_expr;

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

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> From<f64> for Quantity<D, T, M, K, I, N> {
    fn from(value: f64) -> Self {
        Quantity::new(value)
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

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> MulAssign<Quantity<0, 0, 0, 0, 0, 0>> for Quantity<D, T, M, K, I, N> {
    fn mul_assign(&mut self, rhs: Quantity<0, 0, 0, 0, 0, 0>) {
        self.value *= rhs.value;
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

impl<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8> DivAssign<Quantity<0, 0, 0, 0, 0, 0>> for Quantity<D, T, M, K, I, N> {
    fn div_assign(&mut self, rhs: Quantity<0, 0, 0, 0, 0, 0>) {
        self.value /= rhs.value;
    }
}

pub type Unitless = Quantity<0, 0, 0, 0, 0, 0>;
pub type Distance = Quantity<1, 0, 0, 0, 0, 0>;
pub type Time = Quantity<0, 1, 0, 0, 0, 0>;
pub type Mass = Quantity<0, 0, 1, 0, 0, 0>;
pub type Temperature = Quantity<0, 0, 0, 1, 0, 0>;
pub type Current = Quantity<0, 0, 0, 0, 1, 0>;
pub type Amount = Quantity<0, 0, 0, 0, 0, 1>;

pub type Area = unit_expr!(Distance * Distance);
pub type Volume = unit_expr!(Area * Distance);
pub type Velocity = unit_expr!(Distance / Time);
pub type Acceleration = unit_expr!(Velocity / Time);
pub type Jerk = unit_expr!(Acceleration / Time);
pub type Wavenumber = unit_expr!(Unitless / Distance);
pub type Frequency = unit_expr!(Unitless / Time);
pub type Momentum = unit_expr!(Mass * Velocity);
pub type Force = unit_expr!(Mass * Acceleration);
pub type Pressure = unit_expr!(Force / Area);
pub type Energy = unit_expr!(Force * Distance);
pub type Power = unit_expr!(Energy / Time);
pub type Torque = unit_expr!(Force * Distance);
pub type Impulse = unit_expr!(Force * Time);
pub type Density = unit_expr!(Mass / Volume);
pub type MolarMass = unit_expr!(Mass / Amount);
pub type SpecificEnergy = unit_expr!(Energy / Mass);
pub type MolarEnergy = unit_expr!(Energy / Amount);
pub type HeatCapacity = unit_expr!(Energy / Temperature);
pub type SpecificHeatCapacity = unit_expr!(Energy / (Mass * Temperature));
pub type MolarHeatCapacity = unit_expr!(Energy / (Amount * Temperature));
pub type ThermalConductivity = unit_expr!(Power / (Distance * Temperature));
pub type Concentration = unit_expr!(Amount / Volume);
pub type Molality = unit_expr!(Amount / Mass);
pub type ReactionRate = unit_expr!(Amount / (Volume * Time));
pub type ChemicalPotential = unit_expr!(Energy / Amount);
pub type Charge = unit_expr!(Current * Time);
pub type Voltage = unit_expr!(Power / Current);
pub type Resistance = unit_expr!(Voltage / Current);
pub type Conductance = unit_expr!(Current / Voltage);
pub type Resistivity = unit_expr!(Resistance * Distance);
pub type Conductivity = unit_expr!(Conductance / Distance);
pub type Capacitance = unit_expr!(Charge / Voltage);

pub type Meters = Distance;
pub type Seconds = Time;
pub type Kilograms = Mass;
pub type Kelvin = Temperature;
pub type Amps = Current;
pub type Moles = Amount;

pub type Newtons = Force;
pub type Pascals = Pressure;
pub type Joules = Energy;
pub type Watts = Power;
pub type Coulombs = Charge;
pub type Volts = Voltage;
pub type Ohms = Resistance;
pub type Siemens = Conductance;
pub type Farads = Capacitance;
