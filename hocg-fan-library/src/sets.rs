use hocg_fan_sim::library::GlobalLibrary;

mod hbp01;
mod hsd01;
mod hy;

pub fn append_sets(lib: &mut GlobalLibrary) {
    let mut sets = vec![];

    sets.extend(hbp01::set());
    sets.extend(hsd01::set());
    sets.extend(hy::set());

    for card in sets {
        let card = card();
        lib.cards.insert(card.card_number().into(), card);
    }
}
