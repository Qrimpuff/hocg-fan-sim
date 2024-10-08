use hocg_fan_sim::cards::{
    Color::*, HoloMemberArtDamage::*, HoloMemberExtraAttribute::*, HoloMemberHashTag::*,
    HoloMemberLevel::*, Rarity::*, *,
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-082".into(),
        name: "Kobo Kanaeru".into(),
        colors: vec![Blue],
        hp: 100,
        level: Debut,
        hash_tags: vec![ID, IDGen3],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "Bokobokobokobo".into(),
            cost: vec![Blue],
            damage: Basic(30),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![Unlimited],
        rarity: Common,
        illustration_url: "".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn hbp01_082() {
        // no need for testing: vanilla card
    }
}
