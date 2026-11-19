//! Four Pillars, ten gods, hidden stems, luck cycles and annual cycles.
use crate::calendar::{self, LunarDate, SolarDate, JIE};
use crate::shensha::{self, Shensha};
use crate::{BaziError, Result};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub const STEMS: [&str; 10] = ["甲","乙","丙","丁","戊","己","庚","辛","壬","癸"];
pub const BRANCHES: [&str; 12] = ["子","丑","寅","卯","辰","巳","午","未","申","酉","戌","亥"];
const ELEMENTS: [&str; 5] = ["木","火","土","金","水"];
const TEN_SAME: [&str; 5] = ["比肩","食神","偏财","偏官","偏印"];
const TEN_DIFF: [&str; 5] = ["劫财","伤官","正财","正官","正印"];
const HIDDEN: [&[usize]; 12] = [
    &[9], &[5,9,7], &[0,2,4], &[1], &[4,1,9], &[2,6,4],
    &[3,5], &[5,3,1], &[6,8,4], &[7], &[4,7,3], &[8,0],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Sex { Male, Female }

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag="kind", rename_all="snake_case")]
pub enum BirthTime {
    Clock { hour:u8, minute:u8 },
    Branch { name:String },
    Unknown,
}
impl Default for BirthTime {
    fn default() -> Self { Self::Unknown }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaziRequest {
    pub solar: Option<SolarDate>,
    pub lunar: Option<LunarDate>,
    #[serde(default)]
    pub time: BirthTime,
    pub sex: Sex,
    #[serde(default)]
    pub place: Option<String>,
    #[serde(default)]
    pub deceased_year: Option<i32>,
    /// Override host current year; useful for deterministic tests and repeatable analyses.
    #[serde(default)]
    pub as_of_year: Option<i32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HiddenStem {
    pub stem: String,
    pub element: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pillar {
    pub stem: String,
    pub branch: String,
    /// Day stem's ten god is omitted; hour pillar is None if birth hour is unknown.
    pub ten_god: Option<String>,
    pub hidden_stems: Vec<HiddenStem>,
}
impl Pillar {
    pub fn name(&self) -> String { format!("{}{}",self.stem,self.branch) }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pillars {
    pub year: Pillar,
    pub month: Pillar,
    pub day: Pillar,
    pub hour: Option<Pillar>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct StartAge {
    pub years: i32,
    pub months: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LuckCycle {
    pub sequence: u8,
    pub start_age: i32,
    pub end_age: i32,
    pub pillar: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnualCycle {
    pub year: i32,
    pub pillar: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaziChart {
    pub solar: SolarDate,
    pub lunar: LunarDate,
    pub solar_text: String,
    pub lunar_text: String,
    pub time_text: String,
    pub sex: Sex,
    pub place: Option<String>,
    pub zi_time: String,
    pub pillars: Pillars,
    pub forward: bool,
    pub start_age: StartAge,
    pub luck_cycles: Vec<LuckCycle>,
    pub annual_cycles: Vec<AnnualCycle>,
    pub as_of_year: i32,
    pub year_for_pillar: i32,
    pub current_jie: String,
    pub shensha: Vec<Shensha>,
    pub warnings: Vec<String>,
}

pub fn ganzhi(index: i32) -> String {
    let i=index.rem_euclid(60) as usize;
    format!("{}{}",STEMS[i%10], BRANCHES[i%12])
}
pub fn annual_ganzhi(year:i32) -> String { ganzhi(year-4) }
pub fn day_ganzhi(date:SolarDate) -> String { ganzhi(date.jdn()+49) }
pub fn ten_god(day_stem:usize, other:usize) -> &'static str {
    let relation=(other as i32/2-day_stem as i32/2).rem_euclid(5) as usize;
    if (day_stem%2)==(other%2) {TEN_SAME[relation]} else {TEN_DIFF[relation]}
}
fn pillar(stem:usize, branch:usize, day:Option<usize>) -> Pillar {
    Pillar {
        stem:STEMS[stem].into(),
        branch:BRANCHES[branch].into(),
        ten_god:day.map(|d|ten_god(d,stem).into()),
        hidden_stems:HIDDEN[branch].iter().map(|&s|HiddenStem {
            stem:STEMS[s].into(),element:ELEMENTS[s/2].into()
        }).collect(),
    }
}
fn branch_index(name:&str) -> Result<usize> {
    BRANCHES.iter().position(|&x|x==name).ok_or_else(||BaziError(
        format!("invalid birth branch {name}; expected one of {}",BRANCHES.join(""))
    ))
}
fn current_beijing_year() -> Result<i32> {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH)
        .map_err(|e|BaziError(e.to_string()))?.as_secs_f64();
    let local_jd=2440587.5+secs/86400.0+8.0/24.0;
    Ok(calendar::local_date(local_jd).year)
}
fn near_boundary(hour:u8, minute:u8) -> bool {
    let total = i32::from(hour)*60+i32::from(minute);
    (0..12).any(|k| (total-(60+120*k)).abs()<=15)
}
fn start_age(days:f64) -> StartAge {
    let whole=days.abs().floor() as i32;
    let fraction=days.abs()-f64::from(whole);
    let mut years=whole/3;
    let mut months=(whole%3)*4+(fraction*4.0).round() as i32;
    if months>=12 {years+=months/12;months%=12;}
    StartAge {years,months}
}
#[derive(Clone)]
struct JieEvent {
    jd:f64,
    name:&'static str,
    branch:usize,
}
fn jie_around(year:i32) -> Vec<JieEvent> {
    let mut events=Vec::with_capacity(36);
    for y in year-1..=year+1 {
        for &(longitude,name,branch) in &JIE {
            events.push(JieEvent {
                jd:calendar::tt_to_beijing_jd(calendar::solar_term_jd(y,longitude)),
                name,branch
            });
        }
    }
    events.sort_by(|a,b|a.jd.total_cmp(&b.jd));
    events
}
/// Calculate one deterministic, typed chart. All temporal input is UTC+08:00,
/// matching the source skill. This is a cultural calendar, not a scientific forecast.
pub fn calculate(request:&BaziRequest) -> Result<BaziChart> {
    if request.solar.is_none() && request.lunar.is_none() {
        return Err(BaziError("provide solar or lunar date".into()));
    }
    let solar=match (request.solar,request.lunar) {
        (Some(d),_)=>SolarDate::new(d.year,d.month,d.day)?,
        (None,Some(d))=>calendar::lunar_to_solar(d)?,
        (None,None)=>unreachable!(),
    };
    if let Some(l)=request.lunar {
        // Validate even when a solar date is also provided.
        calendar::lunar_to_solar(l)?;
    }
    let lunar=match request.lunar {
        Some(l) if request.solar.is_some()=>l,
        _=>calendar::solar_to_lunar(solar)?,
    };
    let mut warnings=Vec::<String>::new();
    let (hour,minute,known,clock_known,shichen) = match &request.time {
        BirthTime::Clock {hour,minute} => {
            if *hour>23 || *minute>59 {return Err(BaziError("clock must be HH:MM (00:00..23:59)".into()));}
            let h=usize::from(*hour);
            let b=if h>=23 || h<1 {0} else {(h+1)/2};
            (*hour,*minute,true,true,b)
        }
        BirthTime::Branch {name}=>{
            let b=branch_index(name)?;
            if b==0 {warnings.push("未提供具体时刻，子时按早子时（当日日柱）处理".into());}
            ((b*2) as u8,0,true,false,b)
        }
        BirthTime::Unknown=>{
            warnings.push("时辰未知".into());
            (12,0,false,false,6)
        }
    };
    let birth_jd=solar.beijing_jd(hour,minute);
    let lichun=calendar::tt_to_beijing_jd(calendar::solar_term_jd(solar.year,315));
    let year_for_pillar=if birth_jd>=lichun {solar.year} else {solar.year-1};
    let year_idx=(year_for_pillar-4).rem_euclid(60) as usize;
    let (year_stem,year_branch)=(year_idx%10,year_idx%12);

    let events=jie_around(solar.year);
    let current_idx=events.iter().rposition(|e|e.jd<=birth_jd)
        .ok_or_else(||BaziError("cannot locate previous Jie solar term".into()))?;
    let current=&events[current_idx];
    let prev_jd=current.jd;
    let next_jd=events.get(current_idx+1).ok_or_else(||BaziError(
        "cannot locate next Jie solar term".into()
    ))?.jd;
    let month_branch=current.branch;
    let step=(month_branch+12-2)%12;
    let month_stem=((year_stem%5)*2+2+step)%10;
    let day_used=if clock_known && hour>=23 {solar.add_days(1)} else {solar};
    let day_idx=(day_used.jdn()+49).rem_euclid(60) as usize;
    let (day_stem,day_branch)=(day_idx%10,day_idx%12);
    let hour_stem=if known {Some(((day_stem%5)*2+shichen)%10)} else {None};
    let hour_branch=if known {Some(shichen)} else {None};
    let zi_time=if clock_known && hour==23 {"夜子时"}
        else if clock_known && hour==0 {"早子时"} else {"无"};
    let pillars=Pillars {
        year:pillar(year_stem,year_branch,Some(day_stem)),
        month:pillar(month_stem,month_branch,Some(day_stem)),
        day:pillar(day_stem,day_branch,None),
        hour:hour_stem.map(|h|pillar(h,shichen,Some(day_stem))),
    };
    let forward=(year_stem%2==0)==(request.sex==Sex::Male);
    let distance=if forward {next_jd-birth_jd} else {birth_jd-prev_jd};
    let start_age=start_age(distance);
    let mut luck_cycles=Vec::with_capacity(8);
    for n in 1..=8 {
        let delta=if forward {n} else {-n};
        let stem=(month_stem as i32+delta).rem_euclid(10) as usize;
        let branch=(month_branch as i32+delta).rem_euclid(12) as usize;
        let from=start_age.years+(n-1)*10;
        luck_cycles.push(LuckCycle {
            sequence:n as u8,start_age:from,end_age:from+9,
            pillar:format!("{}{}",STEMS[stem],BRANCHES[branch])
        });
    }
    if (next_jd-birth_jd).abs()<=1.0 || (birth_jd-prev_jd).abs()<=1.0 {
        warnings.push("节气交界".into());
    }
    if (birth_jd-lichun).abs()<=1.0 ||
        [solar.year-1,solar.year+1].iter().any(|&y| {
            let jd=calendar::tt_to_beijing_jd(calendar::solar_term_jd(y,315));
            (birth_jd-jd).abs()<=1.0
        }) {
        warnings.push("立春前后".into());
    }
    if request.place.is_some() && clock_known && near_boundary(hour,minute) {
        warnings.push("校正后可能跨时辰".into());
    }
    let as_of_year=match (request.deceased_year, request.as_of_year) {
        (Some(y),_)=>y,
        (None,Some(y))=>y,
        (None,None)=>current_beijing_year()?
    };
    if !(1900..=2100).contains(&as_of_year) {
        return Err(BaziError("as-of/deceased year must be 1900..=2100".into()));
    }
    if request.deceased_year.is_some() && as_of_year<solar.year {
        return Err(BaziError("deceased year cannot precede birth".into()));
    }
    let annual_cycles=(solar.year..=as_of_year).map(|year|AnnualCycle {
        year,pillar:annual_ganzhi(year)
    }).collect();
    let time_text=if clock_known {
        format!("{} / {hour:02}:{minute:02}",BRANCHES[shichen])
    } else if known {BRANCHES[shichen].into()} else {"未知".into()};
    let solar_text=if clock_known {
        format!("{solar} {hour:02}:{minute:02}")
    } else {solar.to_string()};
    let shensha=shensha::calculate(
        [Some(year_stem),Some(month_stem),Some(day_stem),hour_stem],
        [Some(year_branch),Some(month_branch),Some(day_branch),hour_branch],
        request.sex==Sex::Male,
    );
    Ok(BaziChart {
        solar,lunar,solar_text,lunar_text:calendar::lunar_text(lunar),
        time_text,sex:request.sex,place:request.place.clone(),zi_time:zi_time.into(),
        pillars,forward,start_age,luck_cycles,annual_cycles,as_of_year,
        year_for_pillar,current_jie:current.name.into(),shensha,warnings
    })
}
impl BaziChart {
    /// Original Chinese Markdown report layout, useful in terminals and agents.
    pub fn markdown(&self) -> String {
        let p=&self.pillars;
        let h=p.hour.as_ref();
        let unknown=||"未知".to_owned();
        let hidden=|pillar:&Pillar|pillar.hidden_stems.iter().map(|x|format!("{}{}",x.stem,x.element)).collect::<Vec<_>>().join("、");
        let mut lines=vec![
            "## 输入".to_owned(),
            format!("- 阳历：{}",self.solar_text),
            format!("- 农历：{}",self.lunar_text),
            format!("- 时辰：{}",self.time_text),
            format!("- 性别：{}", if self.sex==Sex::Male {"男"} else {"女"}),
        ];
        if let Some(place)=&self.place { lines.push(format!("- 出生地：{place}")); }
        lines.extend([
            format!("- 子时：{}",self.zi_time), String::new(),
            "## 四柱".into(), "| | 年柱 | 月柱 | 日柱 | 时柱 |".into(),
            "|---|---|---|---|---|".into(),
            format!("| 天干 | {} | {} | {} | {} |",p.year.stem,p.month.stem,p.day.stem,h.map_or_else(unknown,|x|x.stem.clone())),
            format!("| 地支 | {} | {} | {} | {} |",p.year.branch,p.month.branch,p.day.branch,h.map_or_else(unknown,|x|x.branch.clone())),
            format!("| 十神 | {} | {} | — | {} |",
                p.year.ten_god.as_deref().unwrap_or("—"),p.month.ten_god.as_deref().unwrap_or("—"),
                h.and_then(|x|x.ten_god.as_ref()).map_or("未知",String::as_str)),
            format!("| 藏干 | {} | {} | {} | {} |",hidden(&p.year),hidden(&p.month),
                hidden(&p.day),h.map_or_else(unknown,hidden)),
            String::new(),"## 大运".into(),
            format!("- 方向：{}",if self.forward {"顺排"}else{"逆排"}),
            format!("- 起运：{}岁{}",
                self.start_age.years,if self.start_age.months>0 {format!("{}个月",self.start_age.months)}else{String::new()}),
            "| 大运序 | 年龄范围 | 干支 |".into(),
            "|---|---|---|".into()
        ]);
        if self.start_age.years>0 || self.start_age.months>0 {
            let pre=if self.start_age.months>0 {
                format!("{}岁{}个月",self.start_age.years,self.start_age.months)
            } else {format!("{}岁",self.start_age.years)};
            lines.push(format!("| 起运前 | 0岁-{pre} | {}（小运） |",p.month.name()));
        }
        for cycle in &self.luck_cycles {
            lines.push(format!("| {} | {}-{}岁 | {} |",cycle.sequence,
                cycle.start_age,cycle.end_age,cycle.pillar));
        }
        lines.push(String::new());
        lines.push("## 流年".into());
        lines.push(format!("- 当前：{}年{}",self.as_of_year,annual_ganzhi(self.as_of_year)));
        for annual in &self.annual_cycles {
            lines.push(format!("- {}年{}",annual.year,annual.pillar));
        }
        lines.push(String::new());
        lines.push("## 神煞".into());
        for item in &self.shensha {
            lines.push(format!("- {} — 见于{} — {}",item.name,item.positions.join("、"),item.method));
        }
        lines.push(String::new());
        lines.push("## 警告".into());
        if self.warnings.is_empty() {
            lines.push("- 无".into());
        } else {
            for warning in &self.warnings {
                lines.push(format!("- {warning}"));
            }
        }
        lines.push(String::new());
        lines.join("\n")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn example(time:BirthTime, sex:Sex) -> BaziChart {
        calculate(&BaziRequest {
            solar:Some(SolarDate::new(1990,5,15).unwrap()),lunar:None,
            time,sex,place:None,deceased_year:None,as_of_year:Some(2026)
        }).unwrap()
    }
    #[test]
    fn golden_pillars_and_shensha() {
        let c=example(BirthTime::Branch{name:"午".into()}, Sex::Male);
        assert_eq!(c.pillars.year.name(),"庚午");
        assert_eq!(c.pillars.month.name(),"辛巳");
        assert_eq!(c.pillars.day.name(),"庚辰");
        assert_eq!(c.pillars.hour.as_ref().unwrap().name(),"壬午");
        assert!(c.forward);
        assert!(c.shensha.iter().any(|x|x.name=="天乙贵人"));
        assert!(c.shensha.iter().any(|x|x.name=="将星"));
        assert!(!c.shensha.iter().any(|x|x.name=="驿马" || x.name.contains("桃花")));
    }
    #[test]
    fn early_and_late_zi() {
        let early=example(BirthTime::Clock{hour:0,minute:30},Sex::Male);
        let late=example(BirthTime::Clock{hour:23,minute:30},Sex::Male);
        assert_eq!(early.pillars.day.name(),"庚辰");
        assert_eq!(late.pillars.day.name(),"辛巳");
        assert_eq!(early.zi_time,"早子时");
        assert_eq!(late.zi_time,"夜子时");
    }
    #[test]
    fn before_lichun_uses_previous_year() {
        let chart=calculate(&BaziRequest{
            solar:Some(SolarDate::new(1990,2,3).unwrap()),lunar:None,
            time:BirthTime::Clock{hour:10,minute:0},sex:Sex::Male,
            place:None,deceased_year:None,as_of_year:Some(2026)
        }).unwrap();
        assert_eq!(chart.pillars.year.name(),"己巳");
    }
    #[test]
    fn reverse_and_unknown() {
        let f=example(BirthTime::Unknown,Sex::Female);
        assert!(!f.forward);
        assert!(f.pillars.hour.is_none());
        assert!(f.warnings.iter().any(|x|x.contains("未知")));
        let male=calculate(&BaziRequest{
            solar:Some(SolarDate::new(1991,6,15).unwrap()),lunar:None,
            time:BirthTime::Unknown,sex:Sex::Male,place:None,
            deceased_year:None,as_of_year:Some(2026)
        }).unwrap();
        assert!(!male.forward);
    }
    #[test]
    fn lunar_and_deceased_end() {
        let c=calculate(&BaziRequest {
            solar:None,lunar:Some(LunarDate{year:1990,month:4,day:21,leap:false}),
            time:BirthTime::Branch{name:"午".into()},sex:Sex::Male,place:None,
            deceased_year:Some(2010),as_of_year:Some(2026),
        }).unwrap();
        assert_eq!(c.solar,SolarDate::new(1990,5,15).unwrap());
        assert_eq!(c.annual_cycles.last().unwrap().year,2010);
    }
}
