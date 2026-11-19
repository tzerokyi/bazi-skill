//! Gregorian, solar-term and astronomical new-moon calculations.
//! Chinese calendar boundary uses fixed UTC+08:00, independent of host timezone.
use crate::{BaziError, Result};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::fmt;

const J2000: f64 = 2_451_545.0;
const DAYS_PER_CENTURY: f64 = 36_525.0;
const ZHONGQI: [u16; 12] = [0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330];
pub const JIE: [(u16, &str, usize); 12] = [
    (315, "立春", 2), (345, "惊蛰", 3), (15, "清明", 4),
    (45, "立夏", 5), (75, "芒种", 6), (105, "小暑", 7),
    (135, "立秋", 8), (165, "白露", 9), (195, "寒露", 10),
    (225, "立冬", 11), (255, "大雪", 0), (285, "小寒", 1),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SolarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}
impl SolarDate {
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self> {
        if !(1900..=2100).contains(&year) || !(1..=12).contains(&month) {
            return Err(BaziError("solar date must be in 1900..=2100".into()));
        }
        let max = match month {
            4 | 6 | 9 | 11 => 30,
            2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
            2 => 28,
            _ => 31,
        };
        if day == 0 || day > max {
            return Err(BaziError(format!("invalid Gregorian date {year}-{month:02}-{day:02}")));
        }
        Ok(Self { year, month, day })
    }
    pub fn parse(value: &str) -> Result<Self> {
        let mut s = value.split('-');
        let y = s.next().unwrap_or("").parse().map_err(|_| BaziError("expected YYYY-MM-DD".into()))?;
        let m = s.next().unwrap_or("").parse().map_err(|_| BaziError("expected YYYY-MM-DD".into()))?;
        let d = s.next().unwrap_or("").parse().map_err(|_| BaziError("expected YYYY-MM-DD".into()))?;
        if s.next().is_some() { return Err(BaziError("expected YYYY-MM-DD".into())); }
        Self::new(y, m, d)
    }
    pub fn jdn(self) -> i32 {
        let a = (14 - i32::from(self.month)) / 12;
        let y = self.year + 4800 - a;
        let m = i32::from(self.month) + 12 * a - 3;
        i32::from(self.day) + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
    }
    pub(crate) fn from_jdn(jdn: i32) -> Self {
        let a = jdn + 32044;
        let b = (4 * a + 3) / 146097;
        let c = a - (146097 * b) / 4;
        let d = (4 * c + 3) / 1461;
        let e = c - (1461 * d) / 4;
        let m = (5 * e + 2) / 153;
        Self {
            year: 100 * b + d - 4800 + m / 10,
            month: (m + 3 - 12 * (m / 10)) as u8,
            day: (e - (153 * m + 2) / 5 + 1) as u8,
        }
    }
    pub(crate) fn add_days(self, days: i32) -> Self {
        Self::from_jdn(self.jdn() + days)
    }
    /// Julian day in UTC from Beijing clock time.
    pub(crate) fn beijing_jd(self, hour: u8, minute: u8) -> f64 {
        f64::from(self.jdn()) - 0.5 + f64::from(hour) / 24.0
            + f64::from(minute) / 1440.0 - 8.0 / 24.0
    }
}
impl fmt::Display for SolarDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LunarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub leap: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SolarTerm {
    pub name: String,
    pub longitude: u16,
    pub date: SolarDate,
    pub hour: u8,
    pub minute: u8,
}

pub(crate) fn delta_t_seconds(year: f64) -> f64 {
    let t = year - 2000.0;
    if year < 1920.0 {
        let x = year - 1900.0;
        -2.79 + 1.494119 * x - 0.0598939 * x.powi(2)
            + 0.0061966 * x.powi(3) - 0.000197 * x.powi(4)
    } else if year < 1941.0 {
        let x = year - 1920.0;
        21.20 + 0.84493 * x - 0.076100 * x.powi(2) + 0.0020936 * x.powi(3)
    } else if year < 1961.0 {
        let x = year - 1950.0;
        29.07 + 0.407 * x - x.powi(2) / 233.0 + x.powi(3) / 2547.0
    } else if year < 1986.0 {
        let x = year - 1975.0;
        45.45 + 1.067 * x - x.powi(2) / 260.0 - x.powi(3) / 718.0
    } else if year < 2005.0 {
        let x = year - 2000.0;
        63.86 + 0.3345 * x - 0.060374 * x.powi(2) + 0.0017275 * x.powi(3)
            + 0.000651814 * x.powi(4) + 0.00002373599 * x.powi(5)
    } else if year < 2050.0 {
        62.92 + 0.32217 * t + 0.005589 * t * t
    } else {
        -20.0 + 32.0 * ((year - 1820.0) / 100.0).powi(2) - 0.5628 * (2150.0 - year)
    }
}

fn radians(deg: f64) -> f64 { deg * PI / 180.0 }
fn longitude_delta(actual: f64, target: f64) -> f64 {
    (actual - target + 180.0).rem_euclid(360.0) - 180.0
}
pub fn sun_apparent_longitude(jd_tt: f64) -> f64 {
    let t = (jd_tt - J2000) / DAYS_PER_CENTURY;
    let l0 = 280.46646 + 36000.76983 * t + 0.0003032 * t * t;
    let m = 357.52911 + 35999.05029 * t - 0.0001537 * t * t;
    let mr = radians(m);
    let c = (1.914602 - 0.004817 * t - 0.000014 * t * t) * mr.sin()
        + (0.019993 - 0.000101 * t) * (2.0 * mr).sin()
        + 0.000289 * (3.0 * mr).sin();
    let omega = 125.04 - 1934.136 * t;
    (l0 + c - 0.00569 - 0.00478 * radians(omega).sin()).rem_euclid(360.0)
}
/// Meeus simplified apparent solar longitude root in terrestrial Julian days.
/// Decimal longitude 0..360, with the first occurrence after Jan 1.
pub fn solar_term_jd(year: i32, longitude: u16) -> f64 {
    let longitude = f64::from(longitude % 360);
    let delta = (longitude - 280.46646).rem_euclid(360.0);
    let center = J2000 + f64::from(year - 2000) * 365.242189 + delta / 0.98564736;
    let (mut lo, mut hi) = (center - 8.0, center + 8.0);
    for _ in 0..52 {
        let mid = (lo + hi) / 2.0;
        if longitude_delta(sun_apparent_longitude(mid), longitude) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}
/// TT astronomical Julian day to Beijing civil Julian day.
pub(crate) fn tt_to_beijing_jd(jd_tt: f64) -> f64 {
    let year = 2000.0 + (jd_tt - J2000) / 365.242189;
    jd_tt - delta_t_seconds(year) / 86400.0 + 8.0 / 24.0
}
pub(crate) fn local_date_time(jd: f64) -> (SolarDate, u8, u8) {
    let day = (jd + 0.5).floor();
    let minutes = ((jd + 0.5 - day) * 1440.0).round() as i32;
    let shift = minutes.div_euclid(1440);
    let mins = minutes.rem_euclid(1440);
    (SolarDate::from_jdn(day as i32 + shift), (mins / 60) as u8, (mins % 60) as u8)
}
pub(crate) fn local_date(jd: f64) -> SolarDate {
    SolarDate::from_jdn((jd + 0.5).floor() as i32)
}
pub fn solar_terms(year: i32) -> Result<Vec<SolarTerm>> {
    if !(1900..=2100).contains(&year) { return Err(BaziError("year must be 1900..=2100".into())); }
    let mut out = Vec::with_capacity(12);
    for &(lon, name, _) in &JIE {
        let (date, hour, minute) = local_date_time(tt_to_beijing_jd(solar_term_jd(year, lon)));
        out.push(SolarTerm { name: name.into(), longitude: lon, date, hour, minute });
    }
    out.sort_by_key(|t| (t.date, t.hour, t.minute));
    Ok(out)
}

/// Mean conjunction corrected with Meeus AA 49 periodic terms, TT Julian day.
pub fn new_moon_jde(k: i32) -> f64 {
    let k = f64::from(k);
    let t = k / 1236.85;
    let e = 1.0 - 0.002516 * t - 0.0000074 * t * t;
    let m = radians(2.5534 + 29.10535670 * k - 0.0000014 * t.powi(2) - 0.00000011 * t.powi(3));
    let mp = radians(201.5643 + 385.81693528 * k + 0.0107582 * t.powi(2)
        + 0.00001238 * t.powi(3) - 0.000000058 * t.powi(4));
    let f = radians(160.7108 + 390.67050284 * k - 0.0016118 * t.powi(2)
        - 0.00000227 * t.powi(3) + 0.000000011 * t.powi(4));
    let om = radians(124.7746 - 1.56375588 * k + 0.0020672 * t.powi(2) + 0.00000215 * t.powi(3));
    let args = [
        mp, m, 2.0*mp, 2.0*f, mp-m, mp+m, 2.0*m, mp-2.0*f, mp+2.0*f,
        2.0*mp+m, 3.0*mp, m+2.0*f, m-2.0*f, 2.0*mp-m, om, mp+2.0*m,
        2.0*mp-2.0*f, 3.0*m, mp+m-2.0*f, 2.0*mp+2.0*f,
        mp+m+2.0*f, mp-m+2.0*f, mp-m-2.0*f, 3.0*mp+m, 4.0*mp
    ];
    let coeff = [
        -0.40720, 0.17241*e, 0.01608, 0.01039, 0.00739*e,
        -0.00514*e, 0.00208*e*e, -0.00111, -0.00057, 0.00056*e,
        -0.00042, 0.00042*e, 0.00038*e, -0.00024, -0.00017,
        -0.00007, 0.00004, 0.00004, 0.00003, 0.00003,
        -0.00003, 0.00003, -0.00002, -0.00002, 0.00002
    ];
    let mut jde = 2451550.09766 + 29.530588861*k + 0.00015437*t.powi(2)
        - 0.000000150*t.powi(3) + 0.00000000073*t.powi(4);
    for (c, a) in coeff.iter().zip(args) { jde += c*a.sin(); }
    let angles = [
        299.77 + 0.107408*k - 0.009173*t*t, 251.88+0.016321*k,
        251.83+26.651886*k, 349.42+36.412478*k, 84.66+18.206239*k,
        141.74+53.303771*k, 207.14+2.453732*k, 154.84+7.306860*k,
        34.52+27.261239*k, 207.19+0.121824*k, 291.34+1.844379*k,
        161.72+24.198154*k, 239.56+25.513099*k, 331.55+3.592518*k
    ];
    let micro = [325.,165.,164.,126.,110.,62.,60.,56.,47.,42.,40.,37.,35.,23.];
    for (c,a) in micro.iter().zip(angles) { jde += c*1e-6*radians(a).sin(); }
    jde
}
fn last_new_moon_k(jd: f64) -> i32 {
    let mut k = ((jd - 2451550.09766) / 29.530588861).round() as i32;
    while new_moon_jde(k) > jd { k -= 1; }
    while new_moon_jde(k+1) <= jd { k += 1; }
    k
}
#[derive(Clone, Debug)]
struct LunarMonth {
    year: i32,
    month: u8,
    leap: bool,
    start: SolarDate,
    days: i32,
}
fn lunar_months_between_dongzhi(year: i32) -> Vec<LunarMonth> {
    let dz0 = solar_term_jd(year, 270);
    let dz1 = solar_term_jd(year+1, 270);
    let k0 = last_new_moon_k(dz0);
    let k1 = last_new_moon_k(dz1);
    let moons: Vec<_> = (k0..=k1).map(new_moon_jde).collect();
    let n = moons.len()-1;
    let zhong: Vec<_> = (year-1..=year+1)
        .flat_map(|y| ZHONGQI.iter().map(move |&lon| solar_term_jd(y, lon)))
        .collect();
    let leap_index = if n == 13 {
        (0..n).find(|&i| !zhong.iter().any(|&z| moons[i] <= z && z < moons[i+1]))
    } else { None };
    let mut current_month = 11;
    let mut lunar_year = year;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let leap = leap_index == Some(i);
        if i > 0 && !leap {
            current_month += 1;
            if current_month == 13 { current_month = 1; lunar_year = year+1; }
        }
        let start = local_date(tt_to_beijing_jd(moons[i]));
        let next = local_date(tt_to_beijing_jd(moons[i+1]));
        out.push(LunarMonth { year: lunar_year, month: current_month, leap,
            start, days: next.jdn()-start.jdn() });
    }
    out
}
pub fn solar_to_lunar(solar: SolarDate) -> Result<LunarDate> {
    SolarDate::new(solar.year, solar.month, solar.day)?;
    let mut months = lunar_months_between_dongzhi(solar.year-1);
    months.extend(lunar_months_between_dongzhi(solar.year));
    months.extend(lunar_months_between_dongzhi(solar.year-2));
    for m in months {
        let delta = solar.jdn() - m.start.jdn();
        if (0..m.days).contains(&delta) {
            return Ok(LunarDate { year: m.year, month: m.month, day: (delta+1) as u8, leap: m.leap });
        }
    }
    Err(BaziError(format!("no lunar date for {solar}")))
}
pub fn lunar_to_solar(lunar: LunarDate) -> Result<SolarDate> {
    if !(1900..=2100).contains(&lunar.year) || !(1..=12).contains(&lunar.month) {
        return Err(BaziError("invalid lunar year/month".into()));
    }
    let months = lunar_months_between_dongzhi(lunar.year-1)
        .into_iter().chain(lunar_months_between_dongzhi(lunar.year));
    for m in months {
        if m.year == lunar.year && m.month == lunar.month && m.leap == lunar.leap {
            if lunar.day == 0 || i32::from(lunar.day) > m.days {
                return Err(BaziError(format!("lunar month has {} days", m.days)));
            }
            let day = m.start.add_days(i32::from(lunar.day)-1);
            return SolarDate::new(day.year, day.month, day.day);
        }
    }
    Err(BaziError("lunar month does not exist (check leap flag)".into()))
}
pub fn lunar_text(lunar: LunarDate) -> String {
    const MONTHS: [&str; 12] = [
        "正月", "二月", "三月", "四月", "五月", "六月",
        "七月", "八月", "九月", "十月", "十一月", "十二月"
    ];
    const DAYS: [&str; 30] = [
        "初一","初二","初三","初四","初五","初六","初七","初八","初九","初十",
        "十一","十二","十三","十四","十五","十六","十七","十八","十九","二十",
        "廿一","廿二","廿三","廿四","廿五","廿六","廿七","廿八","廿九","三十"
    ];
    let month = MONTHS.get(lunar.month.saturating_sub(1) as usize).copied().unwrap_or("?");
    let day = DAYS.get(lunar.day.saturating_sub(1) as usize).copied().unwrap_or("?");
    format!("{}年{}{}{}", lunar.year, if lunar.leap { "闰" } else { "" }, month, day)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gregorian_roundtrip_and_leap() {
        for year in [1900, 1904, 1990, 2000, 2026, 2100] {
            for month in 1..=12 {
                let d = SolarDate::new(year, month, 1).unwrap();
                assert_eq!(SolarDate::from_jdn(d.jdn()), d);
            }
        }
        assert!(SolarDate::new(1900, 2, 29).is_err());
        assert!(SolarDate::new(2000, 2, 29).is_ok());
    }
    #[test]
    fn lunar_anchor_and_roundtrip() {
        let solar = SolarDate::new(1990, 5, 15).unwrap();
        assert_eq!(solar_to_lunar(solar).unwrap(),
            LunarDate { year: 1990, month: 4, day: 21, leap: false });
        assert_eq!(lunar_to_solar(LunarDate { year: 1990, month: 4, day: 21, leap: false }).unwrap(), solar);
    }
    #[test]
    fn solar_terms_are_chronological() {
        let ts = solar_terms(1990).unwrap();
        assert_eq!(ts.len(), 12);
        assert!(ts.windows(2).all(|w| w[0].date <= w[1].date));
    }
}
