use hocg_fan_sim::cards::Card;


automod::dir!("src/sets/hsd01");

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
