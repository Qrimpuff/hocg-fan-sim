use hocg_fan_sim::cards::Card;

mod hbp01_038;
mod hbp01_041;
mod hbp01_042;
mod hbp01_048;
mod hbp01_068;
mod hbp01_096;
mod hbp01_098;

pub fn set() -> Vec<fn() -> Card> {
    vec![
        // TODO sort
        hbp01_038::card,
        hbp01_068::card,
        hbp01_098::card,
        hbp01_096::card,
        hbp01_048::card,
        hbp01_042::card,
        hbp01_041::card,
    ]
}
