use std::str::FromStr;
use crate::unit::{Unit, UnitPrefix};

pub fn parse_number_and_unit(input: &str) -> Result<(f32, Unit), String> {
    let s = input.trim();

    let (num_str, unit_str) = split_leading_number(s);

    let Some(value) = num_str.and_then(|t| f32::from_str(t).ok()) else {
        return Err(input.to_string())
    };

    let mut unit = unit_str.unwrap_or("").replace(char::is_whitespace, "");
    unit = unit.replace('µ', "u");

    if unit.is_empty() {
        return Ok((value, Unit::None));
    }

    match parse_unit_suffix(&unit) {
        Some(u) => Ok((value, u)),
        None => Err(input.to_string()),
    }
}

fn split_leading_number(s: &str) -> (Option<&str>, Option<&str>) {
    let bytes = s.as_bytes();
    let mut i = 0;

    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }

    let mut saw_digit = false;

    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
        saw_digit = true;
    }

    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
            saw_digit = true;
        }
    }

    if !saw_digit {
        return (None, None);
    }

    let mut j = i;
    if j < bytes.len() && (bytes[j] == b'e' || bytes[j] == b'E') {
        j += 1;
        if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
            j += 1;
        }
        let exp_start = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j == exp_start {
        } else {
            i = j;
        }
    }

    let (num, rest) = s.split_at(i);
    let rest = rest.trim_start();
    (Some(num), Some(rest))
}

fn parse_unit_suffix(s: &str) -> Option<Unit> {
    // when adding -> longest units at the top
    let units: &mut [(&str, Unit)] = &mut [
        ("mol", Unit::Moles(UnitPrefix::None)),
        ("m",   Unit::Meters(UnitPrefix::None)),
        ("s",   Unit::Seconds(UnitPrefix::None)),
        ("g",   Unit::Grams(UnitPrefix::None)),
        ("A",   Unit::Amps(UnitPrefix::None)),
        ("K",   Unit::Kelvin(UnitPrefix::None)),
    ];

    for (unit_sym, base) in units {
        if let Some(prefix_sym) = s.strip_suffix(*unit_sym) {
            if let Some(pref) = parse_prefix_symbol(prefix_sym) {
                base.set_prefix(pref);
                return Some(*base);
            }
        }
    }

    None
}

fn parse_prefix_symbol(s: &str) -> Option<UnitPrefix> {
    if s.is_empty() {
        return Some(UnitPrefix::None);
    }

    match s {
        "f" => Some(UnitPrefix::Femto),
        "p" => Some(UnitPrefix::Pico),
        "n" => Some(UnitPrefix::Nano),
        "u" => Some(UnitPrefix::Micro),
        "m" => Some(UnitPrefix::Milli),
        "c" => Some(UnitPrefix::Centi),
        "d" => Some(UnitPrefix::Deci),
        "k" => Some(UnitPrefix::Kilo),
        "M" => Some(UnitPrefix::Mega),
        "G" => Some(UnitPrefix::Giga),
        "T" => Some(UnitPrefix::Tera),
        _ => None,
    }
}