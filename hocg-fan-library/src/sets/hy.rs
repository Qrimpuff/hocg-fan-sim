use hocg_fan_sim::cards::Card;


automod::dir!("src/sets/hy");

pub fn set() -> Vec<fn() -> Card> {
    vec![
        hy01_001::card,
        hy02_001::card,
        hy03_001::card,
        hy04_001::card,
    ]
}
