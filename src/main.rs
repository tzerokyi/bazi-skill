//! Thin CLI adapter over the reusable bazi_skill library.
use bazi_skill::{
    calculate, lunar_to_solar, solar_terms, solar_to_lunar, BaziError, BaziRequest,
    BirthTime, LunarDate, Sex, SolarDate,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

const HELP: &str = "bazi — native Rust Four Pillars calculator (Beijing UTC+08:00)

USAGE:
  bazi chart --solar YYYY-MM-DD --sex 男|女 [OPTIONS]
  bazi chart --lunar YYYY-MM-DD [--leap] --sex 男|女 [OPTIONS]
  bazi convert --solar YYYY-MM-DD [--json]
  bazi convert --lunar YYYY-MM-DD [--leap] [--json]
  bazi terms --year YYYY [--json]
  bazi --help | --version

CHART OPTIONS:
  --hour HH:MM           Birth clock (alias --time), exclusive with --shichen
  --shichen 子|丑|寅|卯|辰|巳|午|未|申|酉|戌|亥
  --place TEXT           Only displayed; emits boundary warning, no longitude correction
  --deceased-year YYYY   End annual list at year of death
  --as-of YYYY           Override current year for deterministic output
  --json                 Structured UTF-8 JSON; diagnostics always on stderr

SOLAR and LUNAR may both be passed to chart: solar controls the pillars.
Without --hour or --shichen the hour pillar remains unknown.
Traditional calendar computation is not a scientific prediction.
";
#[derive(Default)]
struct Args {
    command:String,
    values:BTreeMap<String,String>,
    switches:BTreeSet<String>,
}
impl Args {
    fn value(&self,key:&str)->Option<&str> { self.values.get(key).map(String::as_str) }
    fn has(&self,key:&str)->bool { self.switches.contains(key) }
    fn validate(&self, allowed:&[&str]) -> Result<(),BaziError> {
        for key in self.values.keys().chain(self.switches.iter()) {
            if !allowed.contains(&key.as_str()) {
                return Err(BaziError(format!("unknown option --{key} for {}",self.command)));
            }
        }
        Ok(())
    }
}
fn parse() -> Result<Args,BaziError> {
    let mut argv=std::env::args().skip(1).peekable();
    let first=argv.next().unwrap_or_else(||"--help".into());
    let mut args=Args {
        command:match first.as_str() {
            "chart" => "chart".into(),
            "convert" => "convert".into(),
            "terms" => "terms".into(),
            "--help"|"-h"|"help"=>"help".into(),
            "--version"|"-V"|"version"=>"version".into(),
            x if x.starts_with("--")=>return Err(BaziError(
                "expected a subcommand: chart, convert or terms".into()
            )),
            _=>return Err(BaziError(format!("unknown subcommand {first}"))),
        },..Default::default()
    };
    while let Some(key)=argv.next() {
        if key=="--help" || key=="-h" {
            args.command="help".into();
            continue;
        }
        if !key.starts_with("--") {
            return Err(BaziError(format!("unexpected argument {key}")));
        }
        let canonical=match key.as_str() {
            "--time"=>"hour".to_owned(),
            _=>key[2..].to_owned()
        };
        if canonical=="leap" || canonical=="json" {
            if !args.switches.insert(canonical.clone()) {
                return Err(BaziError(format!("duplicate option --{canonical}")));
            }
            continue;
        }
        let value=argv.next().ok_or_else(||BaziError(format!("missing value for --{canonical}")))?;
        if value.starts_with("--") {
            return Err(BaziError(format!("missing value for --{canonical}")));
        }
        if args.values.insert(canonical.clone(),value).is_some() {
            return Err(BaziError(format!("duplicate option --{canonical}")));
        }
    }
    Ok(args)
}
fn date_option(args:&Args,key:&str) -> Result<Option<SolarDate>,BaziError> {
    args.value(key).map(SolarDate::parse).transpose()
}
fn lunar_option(args:&Args)->Result<Option<LunarDate>,BaziError> {
    let Some(value)=args.value("lunar") else {return Ok(None);};
    let parts:Vec<_>=value.split('-').collect();
    if parts.len()!=3 {return Err(BaziError("--lunar requires YYYY-MM-DD".into()));}
    let year=parts[0].parse::<i32>().map_err(|_|BaziError("invalid lunar year".into()))?;
    let month=parts[1].parse::<u8>().map_err(|_|BaziError("invalid lunar month".into()))?;
    let day=parts[2].parse::<u8>().map_err(|_|BaziError("invalid lunar day".into()))?;
    if !(1900..=2100).contains(&year) || !(1..=12).contains(&month) || !(1..=30).contains(&day) {
        return Err(BaziError("lunar date outside 1900-2100, months 1-12, days 1-30".into()));
    }
    Ok(Some(LunarDate{year,month,day,leap:args.has("leap")}))
}
fn parse_year(args:&Args,key:&str)->Result<Option<i32>,BaziError> {
    args.value(key).map(|s|s.parse::<i32>().map_err(|_|BaziError(
        format!("--{key} must be a year")
    ))).transpose()
}
fn output<T:Serialize>(value:&T, json:bool) -> Result<(),BaziError> {
    if json {
        println!("{}",serde_json::to_string_pretty(value).map_err(|e|BaziError(e.to_string()))?);
    }
    Ok(())
}
fn chart(args:&Args)->Result<(),BaziError> {
    args.validate(&["solar","lunar","leap","sex","hour","shichen","place","deceased-year","as-of","json"])?;
    let sex=match args.value("sex") {
        Some("男"|"male"|"m")=>Sex::Male,
        Some("女"|"female"|"f")=>Sex::Female,
        _=>return Err(BaziError("--sex required: 男|女|male|female".into())),
    };
    if args.has("leap") && args.value("lunar").is_none() {
        return Err(BaziError("--leap requires --lunar".into()));
    }
    if args.value("hour").is_some() && args.value("shichen").is_some() {
        return Err(BaziError("--hour and --shichen are mutually exclusive".into()));
    }
    let time=if let Some(clock)=args.value("hour") {
        let (h,m)=clock.split_once(':').ok_or_else(||BaziError(
            "--hour requires HH:MM".into()
        ))?;
        BirthTime::Clock{
            hour:h.parse().map_err(|_|BaziError("bad hour".into()))?,
            minute:m.parse().map_err(|_|BaziError("bad minute".into()))?,
        }
    } else if let Some(name)=args.value("shichen") {
        BirthTime::Branch {name:name.into()}
    } else { BirthTime::Unknown };
    let request=BaziRequest {
        solar:date_option(args,"solar")?,lunar:lunar_option(args)?,time,sex,
        place:args.value("place").map(str::to_owned),
        deceased_year:parse_year(args,"deceased-year")?,
        as_of_year:parse_year(args,"as-of")?
    };
    let mut c=calculate(&request)?;
    // Match legacy precedence: both specified => provided lunar is presentation-only.
    if request.solar.is_some() && request.lunar.is_some() {
        c.lunar_text=bazi_skill::calendar_lunar_text(c.lunar);
    }
    if args.has("json") {output(&c,true)} else {println!("{}",c.markdown());Ok(())}
}
#[derive(Serialize)]
struct Conversion {solar:SolarDate,lunar:LunarDate}
fn convert(args:&Args)->Result<(),BaziError> {
    args.validate(&["solar","lunar","leap","json"])?;
    let s=date_option(args,"solar")?;
    let l=lunar_option(args)?;
    if s.is_some()==l.is_some() {return Err(BaziError(
        "convert requires exactly one of --solar or --lunar".into()
    ));}
    if args.has("leap") && l.is_none(){return Err(BaziError("--leap requires --lunar".into()));}
    let data=if let Some(s)=s {
        Conversion{solar:s,lunar:solar_to_lunar(s)?}
    } else {
        let l=l.ok_or_else(||BaziError("missing lunar date".into()))?;
        Conversion{solar:lunar_to_solar(l)?,lunar:l}
    };
    if args.has("json") {output(&data,true)} else {
        println!("阳历：{}\n农历：{}",data.solar,bazi_skill::calendar_lunar_text(data.lunar));
        Ok(())
    }
}
fn terms(args:&Args)->Result<(),BaziError> {
    args.validate(&["year","json"])?;
    let year=parse_year(args,"year")?.ok_or_else(||BaziError("--year required".into()))?;
    let data=solar_terms(year)?;
    if args.has("json") {output(&data,true)} else {
        for term in data {
            println!("{} {:04}-{:02}-{:02} {:02}:{:02} UTC+08:00 ({}°)",
                term.name,term.date.year,term.date.month,term.date.day,term.hour,term.minute,term.longitude);
        }
        Ok(())
    }
}
fn run()->Result<(),BaziError> {
    let args=parse()?;
    match args.command.as_str() {
        "help"=>{print!("{HELP}");Ok(())},
        "version"=>{println!("bazi {}",env!("CARGO_PKG_VERSION"));Ok(())},
        "chart"=>chart(&args),
        "convert"=>convert(&args),
        "terms"=>terms(&args),
        _=>unreachable!()
    }
}
fn main() {
    if let Err(e)=run() {
        eprintln!("bazi: {e}\nRun 'bazi --help' for usage.");
        std::process::exit(2);
    }
}
