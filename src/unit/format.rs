use hashbrown::HashMap;
use mvutils::lazy;
use crate::unit::{Acceleration, Amps, Area, Concentration, Coulombs, Density, Dimension, Farads, Frequency, Joules, Kelvin, Kilograms, Meters, Molality, MolarEnergy, MolarMass, Moles, Momentum, Newtons, Ohms, Pascals, Quantity, Resistance, Seconds, Siemens, SpecificEnergy, SpecificHeatCapacity, ThermalConductivity, Torque, Unitless, Velocity, Volts, Volume, Watts};

#[derive(Clone, Copy)]
pub struct FormatOptions {
    pub prefix: bool,
    pub sig_figs: u8,
    pub sci_hi: f64,
    pub sci_lo: f64,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { prefix: true, sig_figs: 3, sci_hi: 1e5, sci_lo: 1e-3 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PrefixBand {
    Yocto=-8, Zepto=-7, Atto=-6, Femto=-5, Pico=-4, Nano=-3,
    Micro=-2, Milli=-1, Base=0, Kilo=1, Mega=2, Giga=3, Tera=4, Peta=5,
}

struct Prefix { band: PrefixBand, sym: &'static str, factor: f64 }
const PREFIXES: &[Prefix] = &[
    Prefix{band:PrefixBand::Yocto, sym:"y", factor:1e-24},
    Prefix{band:PrefixBand::Zepto, sym:"z", factor:1e-21},
    Prefix{band:PrefixBand::Atto,  sym:"a", factor:1e-18},
    Prefix{band:PrefixBand::Femto, sym:"f", factor:1e-15},
    Prefix{band:PrefixBand::Pico,  sym:"p", factor:1e-12},
    Prefix{band:PrefixBand::Nano,  sym:"n", factor:1e-9},
    Prefix{band:PrefixBand::Micro, sym:"µ", factor:1e-6},
    Prefix{band:PrefixBand::Milli, sym:"m", factor:1e-3},
    Prefix{band:PrefixBand::Base,  sym:"",  factor:1.0},
    Prefix{band:PrefixBand::Kilo,  sym:"k", factor:1e3},
    Prefix{band:PrefixBand::Mega,  sym:"M", factor:1e6},
    Prefix{band:PrefixBand::Giga,  sym:"G", factor:1e9},
    Prefix{band:PrefixBand::Tera,  sym:"T", factor:1e12},
    Prefix{band:PrefixBand::Peta,  sym:"P", factor:1e15},
];

#[derive(Clone, Copy)]
pub enum PrefixPolicy {
    Any,
    Small,
    Large,
    None,
}

#[derive(Clone)]
struct Alias {
    sym: &'static str,
    policy: PrefixPolicy,
    power: i8,
    factor: f64,
}

lazy! {
    pub static UNIT_ALIASES: HashMap<Dimension, Alias> = HashMap::from([
        (Joules::dimension(), Alias { sym: "J",    policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Watts::dimension(), Alias { sym: "W",    policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Newtons::dimension(), Alias { sym: "N",    policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Pascals::dimension(), Alias { sym: "Pa",   policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Meters::dimension(), Alias { sym: "m",    policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Seconds::dimension(), Alias { sym: "s",    policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Kilograms::dimension(), Alias { sym: "g",   policy:PrefixPolicy::Any,   power: 1, factor: 1.0e3 }),
        (Kelvin::dimension(), Alias { sym: "K",    policy:PrefixPolicy::Small, power: 1, factor: 1.0 }),
        (Amps::dimension(), Alias { sym: "A",    policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Moles::dimension(), Alias { sym: "mol",  policy:PrefixPolicy::None,   power: 1, factor: 1.0 }),
        (Velocity::dimension(), Alias { sym: "m/s",  policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Acceleration::dimension(), Alias { sym: "m/s^2",  policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Volts::dimension(), Alias { sym: "V", policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Area::dimension(), Alias { sym: "m^2", policy:PrefixPolicy::Any,   power: 2, factor: 1.0 }),
        (Volume::dimension(), Alias { sym: "m^3", policy:PrefixPolicy::Any,   power: 3, factor: 1.0 }),
        (Frequency::dimension(), Alias { sym: "Hz", policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Momentum::dimension(), Alias { sym: "Ns", policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Torque::dimension(), Alias { sym: "Nm", policy:PrefixPolicy::Any,   power: 1, factor: 1.0 }),
        (Density::dimension(), Alias { sym: "g/m^3", policy:PrefixPolicy::Any,   power: 1, factor: 1.0e3 }),
        (MolarMass::dimension(), Alias { sym: "g/mol", policy:PrefixPolicy::Any, power: 1, factor: 1.0e3 }),
        (ThermalConductivity::dimension(), Alias { sym: "W/mK", policy:PrefixPolicy::Any, power: 1, factor: 1.0 }),
        (Molality::dimension(), Alias { sym: "mol/kg", policy:PrefixPolicy::None, power: 1, factor: 1.0 }),
        (Ohms::dimension(), Alias { sym: "Ω", policy:PrefixPolicy::Any, power: 1, factor: 1.0 }),
        (Coulombs::dimension(), Alias { sym: "C", policy:PrefixPolicy::Any, power: 1, factor: 1.0 }),
        (Siemens::dimension(), Alias { sym: "S", policy:PrefixPolicy::Any, power: 1, factor: 1.0 }),
        (Farads::dimension(), Alias { sym: "F", policy:PrefixPolicy::Any, power: 1, factor: 1.0 }),

        (MolarEnergy::dimension(), Alias { sym: "J/mol", policy:PrefixPolicy::Any, power: 1, factor: 1.0 } ),
        (SpecificEnergy::dimension(), Alias { sym: "J/kg", policy:PrefixPolicy::Any, power: 1, factor: 1.0 } ),


        (Concentration::dimension(), Alias { sym: "mol/dm^3", policy:PrefixPolicy::None, power: 1, factor: 1.0e-3 }),
        (SpecificHeatCapacity::dimension(), Alias { sym: "J/kgK" policy:PrefixPolicy::Any, power: 1, factor: 1.0 }),

        (Unitless::dimension(), Alias { sym: "",     policy:PrefixPolicy::None,  power: 1, factor: 1.0 }),
    ]);
}

fn canonical_symbol(dim: Dimension) -> String {
    const NAMES: [&str;6] = ["m","s","kg","K","A","mol"];
    let mut out = Vec::new();
    for (e, n) in dim.into_iter().zip(NAMES) {
        if e == 0 { continue; }
        if e == 1 { out.push(n.to_string()); }
        else { out.push(format!("{n}^{e}")); }
    }
    if out.is_empty() { "1".into() } else { out.join("·") }
}

fn pick_prefix(x_si: f64, policy: PrefixPolicy, allow_prefix: bool, power: i8, scale: f64) -> (f64, &'static str) {
    if !allow_prefix { return (1.0, ""); }
    let ax = x_si.abs();
    if ax == 0.0 { return (1.0, ""); }

    let ax = ax * scale;

    match policy {
        PrefixPolicy::None => (1.0, ""),
        PrefixPolicy::Small => {
            let mut best = (1.0, "");
            for p in PREFIXES {
                if p.factor > 1.0 {
                    continue;
                }
                let y = ax / p.factor.powi(power as i32);
                if y >= 1.0 && y < 1000.0.powi(power as i32) {
                    best = (p.factor, p.sym);
                }
            }
            best
        }
        PrefixPolicy::Large => {
            let mut best = (1.0, "");
            for p in PREFIXES {
                if p.factor < 1.0 {
                    continue;
                }
                let y = ax / p.factor.powi(power as i32);
                if y >= 1.0 && y < 1000.0.powi(power as i32) {
                    best = (p.factor, p.sym);
                }
            }
            best
        }
        PrefixPolicy::Any => {
            let mut best = (1.0, "");
            for p in PREFIXES {
                let y = ax / p.factor.powi(power as i32);
                if y >= 1.0 && y < 1000.0.powi(power as i32) {
                    best = (p.factor, p.sym);
                    break;
                }
            }
            best
        }
    }
}

fn round_sig(x: f64, sig: u8) -> f64 {
    if x == 0.0 { return 0.0; }
    let sig = sig.max(1) as i32;
    let log10 = x.abs().log10().floor();
    let scale = 10f64.powi((sig - 1) - log10 as i32);
    (x * scale).round() / scale
}

fn fmt_number(x: f64, sig: u8, sci_hi: f64, sci_lo: f64) -> String {
    let ax = x.abs();
    if (ax >= sci_hi || (ax != 0.0 && ax <= sci_lo)) {
        format!("{:.*e}", sig.saturating_sub(1) as usize, x)
    } else {
        let y = round_sig(x, sig);
        let s = format!("{}", y);
        s
    }
}

pub enum Style { BestAlias, Canonical }

pub fn format_quantity<const D: i8, const T: i8, const M: i8, const K: i8, const I: i8, const N: i8>(
    q: Quantity<D, T, M, K, I, N>,
    style: Style,
    opts: FormatOptions,
) -> String {
    let dim: Dimension = [D, T, M, K, I, N];

    let (sym, policy, power, scale) = match style {
        Style::Canonical => (canonical_symbol(dim), PrefixPolicy::Any, 1, 1.0),
        Style::BestAlias => {
            if let Some(a) = UNIT_ALIASES.get(&dim) {
                (a.sym.to_string(), a.policy, a.power, a.factor)
            } else {
                (canonical_symbol(dim), PrefixPolicy::Any, 1, 1.0)
            }
        }
    };

    let (factor, pre) = pick_prefix(q.value(), policy, opts.prefix, power, scale);
    let factor = factor.powi(power as i32);

    let val = (q.value() / factor) * scale;
    let num = fmt_number(val, opts.sig_figs, opts.sci_hi, opts.sci_lo);

    let unit_str = if style == Style::Canonical {
        let canon = canonical_symbol(dim);
        if !opts.prefix || pre.is_empty() || canon == "1" {
            canon
        } else {
            canon
        }
    } else {
        if sym.is_empty() { "".into() } else { format!("{pre}{sym}") }
    };

    if unit_str.is_empty() { num } else { format!("{num}{unit_str}") }
}
