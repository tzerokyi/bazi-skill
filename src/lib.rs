//! Native Rust BaZi calendar library.
//!
//! Input is explicitly interpreted as Beijing standard time (UTC+08:00).
//! This library performs deterministic traditional calendar calculations;
//! astrological interpretations are cultural, not scientifically validated.
mod bazi;
mod calendar;
mod shensha;

pub use bazi::{
    annual_ganzhi, calculate, day_ganzhi, ganzhi, ten_god, AnnualCycle, BaziChart,
    BaziRequest, BirthTime, HiddenStem, LuckCycle, Pillar, Pillars, Sex, StartAge,
    BRANCHES, STEMS,
};
pub use calendar::{
    lunar_to_solar, new_moon_jde, solar_term_jd, solar_terms, solar_to_lunar,
    sun_apparent_longitude, LunarDate, SolarDate, SolarTerm,
};
pub use shensha::Shensha;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaziError(pub String);
impl std::fmt::Display for BaziError {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for BaziError {}
pub type Result<T> = std::result::Result<T, BaziError>;
