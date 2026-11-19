# bazi-skill — Native Rust 四柱八字 core

[MIT](LICENSE) · Rust library + headless CLI · No Python/runtime services/frontend

Port of the original [jinchenma94/bazi-skill](https://github.com/jinchenma94/bazi-skill) deterministic pai-pan engine. Keeps the original traditional-calendar rules and MIT credit. This repository is a **crate-ready source project**; publication to crates.io has **not** happened.

## Features

- Gregorian day arithmetic and sexagenary year/month/day/hour pillars
- Solar-term boundaries computed by Meeus apparent solar longitude (not hard-coded dates)
- Astronomical new moons and lunisolar conversion, including leap lunar months
- Ten gods, hidden stems, 21 traditional Shensha lookup rules
- Start-of-luck age, forward/reverse ten-year luck cycles, annual pillars
- Deterministic JSON responses and Chinese Markdown output for terminal/agents
- Explicit UTC+08:00 Beijing civil-time convention, without machine timezone dependence
- No UI, network, Python, LLM, or background server

The calendar currently accepts **1900–2100** dates. Simplified solar/lunar astronomical series are approximations near midnight and boundary instants; verify borderline cases with authoritative ephemerides before relying on them.

## CLI

```sh
cargo run --bin bazi -- --help
cargo run --bin bazi -- chart --solar 1990-05-15 --shichen 午 --sex 男 --json --as-of 2026
cargo run --bin bazi -- chart --lunar 1990-04-21 --sex 男 --hour 12:00
cargo run --bin bazi -- convert --solar 1990-05-15 --json
cargo run --bin bazi -- convert --lunar 1990-04-21 --json
cargo run --bin bazi -- terms --year 1990 --json
```

For existing workflows, `--hour`, `--shichen`, `--solar`, `--lunar`, `--leap`, `--sex`, `--place`, and `--deceased-year` are supported under `bazi chart`. The `--hour` and `--shichen` flags are exclusive. When the hour is unknown, the hour pillar stays absent. `--as-of` freezes annual output for reproducibility.

Output is UTF-8. Stdout carries JSON or Markdown, stderr carries errors, failure uses exit code 2. Commands never start a GUI.

## Rust API

```rust
use bazi_skill::{calculate, BaziRequest, BirthTime, Sex, SolarDate};

let chart = calculate(&BaziRequest {
    solar: Some(SolarDate::new(1990, 5, 15)?),
    lunar: None,
    time: BirthTime::Branch { name: "午".into() },
    sex: Sex::Male,
    place: None,
    deceased_year: None,
    as_of_year: Some(2026),
})?;
assert_eq!(chart.pillars.year.name(), "庚午");
assert_eq!(chart.pillars.month.name(), "辛巳");
# Ok::<(), bazi_skill::BaziError>(())
```

Public library exports: `calculate`, `solar_to_lunar`, `lunar_to_solar`, `solar_terms`, `solar_term_jd`, `new_moon_jde`, `annual_ganzhi`, `day_ganzhi`, `ten_god`, typed request/result/error structs. Serde structs are suitable for a future small MCP/FFI adapter; no MCP transport is shipped.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --bin bazi
```

Golden source anchors: 1990-05-15 午 男 => 庚午 / 辛巳 / 庚辰 / 壬午; 1990-02-03 10:00 => 己巳 year; 23:30 switches to next day; lunar 1990-04-21 = solar 1990-05-15. Regression tests also cover unknown hour, reverse luck, annual cutoff, Shensha, and CLI JSON/errors. The self-hosted workflow validates and builds; do not commit artifacts/`target`.

## Provenance / limits

The original classical interpretations remain as reference material in [references/](references/). This project computes a **traditional cultural calendar**, not empirically supported life predictions. The original [MIT license](LICENSE) and copyright notice are retained.

The legacy Python source was replaced by native Rust modules; comparison contracts derive from the prior `scripts/test_pai_pan.py` tests. Neither Python nor a web frontend is required at runtime.
