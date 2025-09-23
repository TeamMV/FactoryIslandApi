pub mod parsing;
pub mod r#mod;

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

    pub fn base_symbol(self) -> &'static str {
        match self {
            Unit::None        => "",
            Unit::Meters(_)   => "m",
            Unit::Seconds(_)  => "s",
            Unit::Grams(_)    => "g",
            Unit::Amps(_)     => "A",
            Unit::Kelvin(_)   => "K",
            Unit::Moles(_)    => "mol",
        }
    }

    pub fn current_prefix(self) -> UnitPrefix {
        match self {
            Unit::None => UnitPrefix::None,
            Unit::Meters(p)
            | Unit::Seconds(p)
            | Unit::Grams(p)
            | Unit::Amps(p)
            | Unit::Kelvin(p)
            | Unit::Moles(p) => p,
        }
    }

    pub fn preferred_prefixes(self) -> &'static [UnitPrefix] {
        match self {
            Unit::None => &[],
            Unit::Meters(_) => &[
                UnitPrefix::Nano, UnitPrefix::Micro, UnitPrefix::Milli, UnitPrefix::Centi, UnitPrefix::None, UnitPrefix::Kilo,
            ],
            Unit::Seconds(_) => &[
                UnitPrefix::Nano, UnitPrefix::Micro, UnitPrefix::Milli, UnitPrefix::None,
            ],
            Unit::Grams(_) => &[
                UnitPrefix::Nano, UnitPrefix::Micro, UnitPrefix::Milli, UnitPrefix::None, UnitPrefix::Kilo,
            ],
            Unit::Amps(_) => &[
                UnitPrefix::Milli, UnitPrefix::None, UnitPrefix::Kilo,
            ],
            Unit::Kelvin(_) => &[
                UnitPrefix::None,
            ],
            Unit::Moles(_) => &[
                UnitPrefix::Milli, UnitPrefix::None,
            ],
        }
    }

    pub fn pick_display_prefix(self, value_in_base: f64) -> UnitPrefix {
        let abs = value_in_base.abs();
        if abs == 0.0 {
            return match self {
                Unit::Kelvin(_) => UnitPrefix::None,
                _ => UnitPrefix::None,
            };
        }

        let candidates = self.preferred_prefixes();
        for &p in candidates {
            let scaled = abs / p.factor();
            if scaled >= 1.0 && scaled < 1000.0 {
                return p;
            }
        }

        if abs >= candidates.last().unwrap().factor() {
            *candidates.last().unwrap()
        } else {
            *candidates.first().unwrap()
        }
    }

    pub fn format_value(self, value_in_base: f64) -> String {
        let prefix = self.pick_display_prefix(value_in_base);
        let scaled = value_in_base / prefix.factor();

        let txt = pretty_number(scaled);
        let unit_txt = format!("{}{}", prefix.symbol(), self.base_symbol());

        if unit_txt.is_empty() { txt } else { format!("{txt}{unit_txt}") }
    }

    pub fn to_base(self, raw: f64) -> f64 {
        raw * self.current_prefix().factor()
    }
}

fn pretty_number(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }

    let abs = x.abs();
    let exp10 = abs.log10().floor() as i32;

    let use_plain = exp10 >= -3 && exp10 < 6;

    if use_plain {
        let digits_before = if abs >= 1.0 { (abs.log10().floor() as i32 + 1) as usize } else { 0 };
        let sig_figs = 3usize;
        let mut dec = sig_figs.saturating_sub(digits_before);
        if dec > 4 { dec = 4; }
        let s = format!("{:.*}", dec, x);
        return trim_trailing_zeros(&s);
    }

    let mant = x / 10f64.powi(exp10);
    let s = format!("{:.3}", mant);
    let mant_txt = trim_trailing_zeros(&s);

    format!("{mant_txt}×10^{exp10}")
}

fn trim_trailing_zeros(s: &str) -> String {
    if let Some(dot) = s.find('.') {
        let (int, frac) = s.split_at(dot);
        let mut frac = &frac[1..]; // drop the dot
        frac = frac.trim_end_matches('0');
        if frac.is_empty() {
            int.to_string()
        } else {
            format!("{int}.{frac}")
        }
    } else {
        s.to_string()
    }
}

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

    pub fn factor(self) -> f64 {
        match self {
            UnitPrefix::Femto => 1e-15,
            UnitPrefix::Pico  => 1e-12,
            UnitPrefix::Nano  => 1e-9,
            UnitPrefix::Micro => 1e-6,
            UnitPrefix::Milli => 1e-3,
            UnitPrefix::Centi => 1e-2,
            UnitPrefix::Deci  => 1e-1,
            UnitPrefix::None  => 1.0,
            UnitPrefix::Kilo  => 1e3,
            UnitPrefix::Mega  => 1e6,
            UnitPrefix::Giga  => 1e9,
            UnitPrefix::Tera  => 1e12,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            UnitPrefix::Femto => "f",
            UnitPrefix::Pico  => "p",
            UnitPrefix::Nano  => "n",
            UnitPrefix::Micro => "µ",
            UnitPrefix::Milli => "m",
            UnitPrefix::Centi => "c",
            UnitPrefix::Deci  => "d",
            UnitPrefix::None  => "",
            UnitPrefix::Kilo  => "k",
            UnitPrefix::Mega  => "M",
            UnitPrefix::Giga  => "G",
            UnitPrefix::Tera  => "T",
        }
    }
}