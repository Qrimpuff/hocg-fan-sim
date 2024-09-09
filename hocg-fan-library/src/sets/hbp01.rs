use hocg_fan_sim::cards::Card;

mod hbp01_042;
mod hbp01_048;
mod hbp01_068;
mod hbp01_098;

pub fn set() -> Vec<fn() -> Card> {
    vec![
        hbp01_068::card,
        hbp01_098::card,
        hbp01_048::card,
        hbp01_042::card,
    ]
}
