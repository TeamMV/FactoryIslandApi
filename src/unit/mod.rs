pub mod parsing;

use mvutils::Savable;

pub const KELVIN_CELSIUS_OFFSET: f32 = 273.15;

#[derive(Savable, PartialEq, Clone, Copy, Debug)]
pub enum Unit {
    None,
    Meters(UnitPrefix),
    Seconds(UnitPrefix),
    Grams(UnitPrefix),
    Amps(UnitPrefix),
    Kelvin(UnitPrefix),
    Moles(UnitPrefix),

    // compound units
    // BeardFornightsPerFortnight(UnitPrefix), // m s^-1
    // BeardFornightsPerFortnightPerFornight(UnitPrefix), // m s^-2
    // GrainBeardFornightsPerFortnightPerFornight(UnitPrefix), // kg m s^-2
    // GrainBeardFornightsBeardFornightsPerFortnightPerFornight(UnitPrefix), // kg m^2 s^-2
    // GrainBeardFornightsBeardFornightsPerFortnightPerFornightPerFornight(UnitPrefix), // kg m^2 s^-3
    // GrainBeardFornightsBeardFornightsPerFortnightPerFornightPerFornightPerAmps(UnitPrefix), // kg m^2 s^-3 A^-1
    // GrainBeardFornightsBeardFornightsPerFortnightPerFornightPerFornightPerAmpsPerAmps(UnitPrefix), // kg m^2 s^-3 A^-2

}

impl Unit {
    pub fn set_prefix(&mut self, p: UnitPrefix) {
        match self {
            Unit::None => {}
            Unit::Meters(old) => *old = p,
            Unit::Seconds(old) => *old = p,
            Unit::Grams(old) => *old = p,
            Unit::Amps(old) => *old = p,
            Unit::Kelvin(old) => *old = p,
            Unit::Moles(old) => *old = p,
        }
    }
}

//μ
#[derive(Savable, PartialEq, Clone, Copy, Debug)]
pub enum UnitPrefix {
    Femto,
    Pico,
    Nano,
    Micro,
    Milli,
    Centi,
    Deci,
    None,
    Kilo,
    Mega,
    Giga,
    Tera,
}

impl UnitPrefix {
    pub fn power_of_ten(&self) -> i8 {
        match self {
            UnitPrefix::Femto => -15,
            UnitPrefix::Pico => -12,
            UnitPrefix::Nano => -9,
            UnitPrefix::Micro => -6,
            UnitPrefix::Milli => -3,
            UnitPrefix::Centi => -2,
            UnitPrefix::Deci => -1,
            UnitPrefix::None => 0,
            UnitPrefix::Kilo => 3,
            UnitPrefix::Mega => 6,
            UnitPrefix::Giga => 9,
            UnitPrefix::Tera => 12,
        }
    }
}