use hocg_fan_sim::cards::Card;

mod hsd01_001;
mod hsd01_002;
mod hsd01_003;
mod hsd01_004;
mod hsd01_005;
mod hsd01_006;
mod hsd01_007;
mod hsd01_008;
mod hsd01_009;
mod hsd01_010;
mod hsd01_011;
mod hsd01_012;
mod hsd01_013;
mod hsd01_014;
mod hsd01_015;
mod hsd01_016;
mod hsd01_017;
mod hsd01_018;
mod hsd01_019;
mod hsd01_020;
mod hsd01_021;

pub fn set() -> Vec<fn() -> Card> {
    vec![
        hsd01_001::card,
        hsd01_002::card,
        hsd01_003::card,
        hsd01_004::card,
        hsd01_005::card,
        hsd01_006::card,
        hsd01_007::card,
        hsd01_008::card,
        hsd01_009::card,
        hsd01_010::card,
        hsd01_011::card,
        hsd01_012::card,
        hsd01_013::card,
        hsd01_014::card,
        hsd01_015::card,
        hsd01_016::card,
        hsd01_017::card,
        hsd01_018::card,
        hsd01_019::card,
        hsd01_020::card,
        hsd01_021::card,
    ]
}
