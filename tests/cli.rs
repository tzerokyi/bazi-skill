//! Real binary smoke tests: JSON contract, exit codes, no Python subprocess.
use serde_json::Value;
use std::process::Command;

fn bazi(args:&[&str])->std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bazi")).args(args).output().unwrap()
}
#[test]
fn chart_json_matches_legacy_golden() {
    let result=bazi(&["chart","--solar","1990-05-15","--shichen","午","--sex","男","--as-of","2026","--json"]);
    assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));
    let json:Value=serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["pillars"]["year"]["stem"],"庚");
    assert_eq!(json["pillars"]["year"]["branch"],"午");
    assert_eq!(json["pillars"]["month"]["stem"],"辛");
    assert_eq!(json["pillars"]["month"]["branch"],"巳");
    assert_eq!(json["pillars"]["day"]["stem"],"庚");
    assert_eq!(json["pillars"]["day"]["branch"],"辰");
    assert_eq!(json["pillars"]["hour"]["stem"],"壬");
    assert_eq!(json["pillars"]["hour"]["branch"],"午");
    assert_eq!(json["forward"],true);
    assert_eq!(json["annual_cycles"].as_array().unwrap().last().unwrap()["year"],2026);
}
#[test]
fn lunar_convert_roundtrip_cli() {
    let res=bazi(&["convert","--lunar","1990-04-21","--json"]);
    assert!(res.status.success(),"{}",String::from_utf8_lossy(&res.stderr));
    let json:Value=serde_json::from_slice(&res.stdout).unwrap();
    assert_eq!(json["solar"]["year"],1990);
    assert_eq!(json["solar"]["month"],5);
    assert_eq!(json["solar"]["day"],15);
}
#[test]
fn report_has_original_sections() {
    let res=bazi(&["chart","--solar","1990-05-15","--sex","男","--as-of","2010"]);
    assert!(res.status.success(),"{}",String::from_utf8_lossy(&res.stderr));
    let text=String::from_utf8(res.stdout).unwrap();
    for section in ["## 输入","## 四柱","## 大运","## 流年","## 神煞","## 警告"] {
        assert!(text.contains(section),"missing {section}");
    }
    assert!(text.contains("时辰未知"));
    assert!(!text.contains("2011年"));
}
#[test]
fn invalid_flags_fail_without_stdout_payload() {
    let res=bazi(&["chart","--solar","1990-05-15","--sex","男","--hour","25:99","--json"]);
    assert_eq!(res.status.code(),Some(2));
    assert!(res.stdout.is_empty());
    assert!(String::from_utf8_lossy(&res.stderr).contains("clock"));
}
#[test]
fn terms_json_exposes_twelve_jie() {
    let res=bazi(&["terms","--year","1990","--json"]);
    assert!(res.status.success(),"{}",String::from_utf8_lossy(&res.stderr));
    let json:Value=serde_json::from_slice(&res.stdout).unwrap();
    assert_eq!(json.as_array().unwrap().len(),12);
}
