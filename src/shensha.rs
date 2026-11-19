//! Table-driven Shensha: only detected hits are returned.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shensha {
    pub name: String,
    pub positions: Vec<String>,
    pub method: String,
}
const GAN_LABELS: [&str;4] = ["年干","月干","日干","时干"];
const ZHI_LABELS: [&str;4] = ["年支","月支","日支","时支"];
const MDH_LABELS: [&str;3] = ["月支","日支","时支"];

fn positions<const N:usize>(
    values: [Option<usize>;N], labels: [&str;N], needles: &[usize]
) -> Vec<String> {
    values.into_iter().zip(labels)
        .filter_map(|(v, label)| v.filter(|value| needles.contains(value)).map(|_| label.to_owned()))
        .collect()
}
fn insert(out:&mut Vec<Shensha>, name:&str, positions:Vec<String>, method:&str) {
    if !positions.is_empty() {
        out.push(Shensha { name:name.into(), positions, method:method.into() });
    }
}
// Ganzhi indexing: 甲=0..癸=9, 子=0..亥=11.
const TIANYI: [&[usize];10] = [
    &[1,7], &[0,8], &[11,9], &[11,9], &[1,7],
    &[0,8], &[1,7,2,6], &[2,6], &[3,5], &[3,5],
];
// Each TianDe entry is (true => stem, false => branch, index).
const TIANDE: [(bool,usize);12] = [
    (true,5), (true,6), (true,3), (false,8),
    (true,8), (true,7), (false,11), (true,0),
    (true,9), (false,2), (true,2), (true,1),
];
const YUEDE: [usize;12] = [8,6,2,0,8,6,2,0,8,6,2,0];
const WENCHANG: [usize;10] = [5,6,8,9,8,9,11,0,2,3];
const XUETANG: [usize;10] = [11,6,2,9,2,9,5,0,8,3];
const CIGUAN: [usize;10] = [2,3,5,6,5,6,8,9,11,0];
// Group: water, wood, metal, fire.
const SANHE: [usize;12] = [0,2,3,1,0,2,3,1,0,2,3,1];
const JIANGXING: [usize;4] = [0,3,9,6];
const HUAGAI: [usize;4] = [4,7,1,10];
const YIMA: [usize;4] = [2,5,11,8];
const JIESHA: [usize;4] = [5,8,2,11];
const ZAISHA: [usize;4] = [6,9,3,0];
const WANGSHEN: [usize;4] = [11,2,8,5];
const TAOHUA: [usize;4] = [9,0,6,3];
const GUCHEN: [usize;12] = [2,2,5,5,5,8,8,8,11,11,11,2];
const GUASU: [usize;12] = [10,10,1,1,1,4,4,4,7,7,7,10];
const JINYU: [usize;10] = [4,5,7,8,7,8,10,11,1,2];
const YANGBLADE: [usize;10] = [3,2,6,5,6,5,9,0,0,11];
const YUANCHEN_FORWARD: [usize;12] = [7,8,9,10,11,0,1,2,3,4,5,6];
const YUANCHEN_REVERSE: [usize;12] = [5,6,7,8,9,10,11,0,1,2,3,4];
const XUEREN: [usize;12] = [4,5,6,7,8,9,10,11,0,1,2,3];

pub(crate) fn calculate(
    gans:[Option<usize>;4], zhis:[Option<usize>;4], male:bool
) -> Vec<Shensha> {
    let yg=gans[0].expect("year stem");
    let dg=gans[2].expect("day stem");
    let yz=zhis[0].expect("year branch");
    let mz=zhis[1].expect("month branch");
    let dz=zhis[2].expect("day branch");
    let mut out = Vec::new();
    let z = |need:&[usize]| positions(zhis, ZHI_LABELS, need);
    let g = |need:&[usize]| positions(gans, GAN_LABELS, need);
    let mut ti = TIANYI[yg].to_vec();
    ti.extend_from_slice(TIANYI[dg]);
    insert(&mut out,"天乙贵人",z(&ti),"年干/日干查地支");
    let (is_gan, word) = TIANDE[mz];
    insert(&mut out, "天德", if is_gan {g(&[word])} else {z(&[word])},
        if is_gan {"月支查天干"} else {"月支查地支"});
    insert(&mut out,"月德",g(&[YUEDE[mz]]),"月支三合");
    insert(&mut out,"文昌",z(&[WENCHANG[yg],WENCHANG[dg]]),"年干/日干查地支");
    insert(&mut out,"学堂",z(&[XUETANG[dg]]),"日干长生");
    insert(&mut out,"词馆",z(&[CIGUAN[dg]]),"日干临官");
    let sanhe = |table:[usize;4]| [table[SANHE[yz]],table[SANHE[dz]]];
    insert(&mut out,"将星",z(&sanhe(JIANGXING)),"年支/日支三合");
    insert(&mut out,"华盖",z(&sanhe(HUAGAI)),"年支/日支");
    insert(&mut out,"驿马",z(&sanhe(YIMA)),"年支/日支");
    insert(&mut out,"天医",z(&[(mz+11)%12]),"月支前一位");
    insert(&mut out,"禄神",z(&[CIGUAN[dg]]),"日干");
    insert(&mut out,"金舆",z(&[JINYU[dg]]),"日干");
    insert(&mut out,"羊刃",z(&[YANGBLADE[dg]]),"日干");
    insert(&mut out,"劫煞",z(&sanhe(JIESHA)),"年支/日支");
    insert(&mut out,"灾煞",z(&sanhe(ZAISHA)),"年支/日支");
    insert(&mut out,"亡神",z(&sanhe(WANGSHEN)),"年支/日支");
    insert(&mut out,"桃花/咸池",z(&sanhe(TAOHUA)),"年支/日支");
    insert(&mut out,"孤辰",z(&[GUCHEN[yz]]),"年支");
    insert(&mut out,"寡宿",z(&[GUASU[yz]]),"年支");
    let xun_start = (dz+12-dg)%12;
    insert(&mut out,"空亡/旬空",z(&[(xun_start+10)%12,(xun_start+11)%12]),"日柱旬空");
    let yu = if (yg%2==0)==male {YUANCHEN_FORWARD[yz]} else {YUANCHEN_REVERSE[yz]};
    insert(&mut out,"元辰",positions([zhis[1],zhis[2],zhis[3]],MDH_LABELS,&[yu]),"年支阴阳男女");
    insert(&mut out,"血刃",z(&[XUEREN[mz]]),"月支");
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn table_cases() {
        let first=calculate([Some(1),Some(4),Some(0),None],
            [Some(11),Some(1),Some(2),None],true);
        assert!(first.iter().any(|x|x.name=="天乙贵人"));
        let second=calculate([Some(0),Some(1),Some(0),None],
            [Some(10),Some(11),Some(0),None],true);
        assert!(second.iter().any(|x|x.name.contains("空亡")));
    }
}
