use core::fmt;

use strict_num::{FiniteF64, PositiveF64};
use strum::FromRepr;

pub struct FloatColorStatsDisplay<'a> {
    pub values: &'a [FiniteF64],
    pub width: usize,
    pub postfix: FloatDisplayPostfix,
}
impl<'a> fmt::Display for FloatColorStatsDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let limit = if self.width == 1 { 0.05 } else { 0.005 };
        let limit = PositiveF64::new(limit).unwrap();

        for v in self.values {
            // Color start
            let color_start = || {
                if round_half_to_even(*v, self.width, limit) {
                    return zero_int_stat_color();
                }
                if v.get() <= -10.0 {
                    return extreme_percent_color();
                }
                if v.get() <= -5.0 {
                    return warn_percent_color();
                }
                int_stat_color()
            };

            match self.postfix {
                FloatDisplayPostfix::Unit(unit) => write!(
                    f,
                    "{}",
                    ValueUnitDisplay {
                        color: color_start(),
                        width: self.width,
                        value: *v,
                        unit,
                    }
                )?,
                FloatDisplayPostfix::Decimals(decimals) => write!(
                    f,
                    "{start} {value:width$.decimals$}{end}",
                    value = v.get(),
                    width = self.width,
                    start = color_start(),
                    end = normal_color()
                )?,
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy)]
pub enum FloatDisplayPostfix {
    Unit(MemoryUnit),
    Decimals(usize),
}

fn round_half_to_even(v: FiniteF64, width: usize, limit: PositiveF64) -> bool {
    match width {
        0 => {
            if -0.5 <= v.get() && v.get() <= 0.5 {
                return true;
            }
        }
        _ => {
            if -limit.get() < v.get() && v.get() < limit.get() {
                return true;
            }
        }
    }
    false
}

#[derive(Debug, Clone, Copy)]
pub struct PercentageColorStatsDisplay<'a> {
    /// Not in the form of percentage numbers
    pub values: &'a [PositiveF64],
    pub width: usize,
    pub decimals: usize,
    pub limit: PercentageDisplayLimit,
}
impl<'a> fmt::Display for PercentageColorStatsDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let limit = if self.width == 1 { 0.05 } else { 0.005 };
        let limit = PositiveF64::new(limit).unwrap();

        // let width = self.width.saturating_sub(1);
        let width = self.width;

        for v in self.values {
            const EXTREME_HIGH: f64 = 90.0;
            const HIGH: f64 = 75.0;
            const LOW: f64 = 25.0;
            const EXTREME_LOW: f64 = 10.0;

            let v = v.get() * 100.;

            // Color start
            let color_start = || {
                let low = || {
                    if v <= EXTREME_LOW {
                        return Some(extreme_percent_color());
                    }
                    if v <= LOW {
                        return Some(warn_percent_color());
                    }
                    None
                };

                match self.limit {
                    PercentageDisplayLimit::ExtremeHigh => {
                        if EXTREME_HIGH <= v {
                            return extreme_percent_color();
                        }
                        if HIGH <= v {
                            return warn_percent_color();
                        }
                    }
                    PercentageDisplayLimit::ExtremeLow => {
                        if let Some(color) = low() {
                            return color;
                        }
                    }
                    PercentageDisplayLimit::ExtremeLow0 => {
                        if limit.get() <= v {
                            if let Some(color) = low() {
                                return color;
                            }
                        }
                    }
                }
                if round_half_to_even(FiniteF64::new(v).unwrap(), width, limit) {
                    return zero_int_stat_color();
                }
                int_stat_color()
            };

            write!(
                f,
                "{start} {value:width$.decimals$}{end}",
                value = v,
                width = width,
                decimals = self.decimals,
                start = color_start(),
                end = normal_color()
            )?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy)]
pub enum PercentageDisplayLimit {
    ExtremeHigh,
    ExtremeLow,
    ExtremeLow0,
}

pub struct U64ColorStatsDisplay<'a> {
    pub values: &'a [u64],
    pub width: usize,
    pub unit: Option<MemoryUnit>,
}
impl<'a> fmt::Display for U64ColorStatsDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in self.values {
            let color_start = || {
                if *v == 0 {
                    return zero_int_stat_color();
                }
                int_stat_color()
            };
            match self.unit {
                Some(unit) => write!(
                    f,
                    "{}",
                    ValueUnitDisplay {
                        color: color_start(),
                        width: self.width,
                        value: FiniteF64::new((*v) as f64).unwrap(),
                        unit
                    }
                )?,
                None => write!(
                    f,
                    "{start} {v:width$}{end}",
                    start = color_start(),
                    width = self.width,
                    end = normal_color()
                )?,
            }
        }
        Ok(())
    }
}

struct ValueUnitDisplay {
    pub color: &'static str,
    /// Width of overall display including the unit char
    pub width: usize,
    pub value: FiniteF64,
    pub unit: MemoryUnit,
}
impl fmt::Display for ValueUnitDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (value, unit) = match self.unit.upgrade(self.value) {
            Some(x) => x,
            None => (self.value, self.unit),
        };
        let width = self.width.saturating_sub(unit.as_str().len());
        write!(
            f,
            "{start} {value:width$.1}{end}{unit}",
            value = value.get(),
            start = self.color,
            end = normal_color(),
            unit = unit.as_str()
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, FromRepr)]
#[repr(u8)]
pub enum MemoryUnit {
    Bytes = 0,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terabytes,
    Petabytes,
}
impl MemoryUnit {
    pub fn upgrade(&self, mut value: FiniteF64) -> Option<(FiniteF64, Self)> {
        let mut unit = *self as u8;
        while 1024.0 <= value.get().abs() {
            let v = value.get() / 1024.0;
            value = FiniteF64::new(v).unwrap();
            unit += 1;
        }
        let unit = Self::from_repr(unit)?;
        Some((value, unit))
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryUnit::Bytes => "B",
            MemoryUnit::Kilobytes => "k",
            MemoryUnit::Megabytes => "M",
            MemoryUnit::Gigabytes => "G",
            MemoryUnit::Terabytes => "T",
            MemoryUnit::Petabytes => "P",
        }
    }
}

const fn warn_percent_color() -> &'static str {
    BOLD_MAGENTA
}
const fn extreme_percent_color() -> &'static str {
    BOLD_RED
}
pub const fn zero_int_stat_color() -> &'static str {
    LIGHT_BLUE
}
pub const fn int_stat_color() -> &'static str {
    BOLD_BLUE
}
pub const fn item_name_color() -> &'static str {
    LIGHT_GREEN
}
pub const fn normal_color() -> &'static str {
    NORMAL
}

#[allow(dead_code)]
const LIGHT_RED: &str = "\x1b[31;22m";
const BOLD_RED: &str = "\x1b[31;1m";
const LIGHT_GREEN: &str = "\x1b[32;22m";
#[allow(dead_code)]
const BOLD_GREEN: &str = "\x1b[32;1m";
#[allow(dead_code)]
const LIGHT_YELLOW: &str = "\x1b[33;22m";
const BOLD_MAGENTA: &str = "\x1b[35;1m";
const BOLD_BLUE: &str = "\x1b[34;1m";
const LIGHT_BLUE: &str = "\x1b[34;22m";
const NORMAL: &str = "\x1b[0m";
