use hocg_fan_sim::cards::Card;

mod hy01_001;
mod hy02_001;
mod hy03_001;
mod hy04_001;

pub fn set() -> Vec<fn() -> Card> {
    vec![
        hy01_001::card,
        hy02_001::card,
        hy03_001::card,
        hy04_001::card,
    ]
}
