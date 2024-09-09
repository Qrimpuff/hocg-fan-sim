use hocg_fan_sim::cards::{
    Color::*, HoloMemberArtDamage::*, HoloMemberExtraAttribute::*, HoloMemberHashTag::*,
    HoloMemberLevel::*, Rarity::*, *,
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-068".into(),
        name: "Omaru Polka".into(),
        colors: vec![Red],
        hp: 70,
        level: Debut,
        hash_tags: vec![JP, Gen5, AnimalEars],
        baton_pass_cost: 0,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "Polka, On the Dot!".into(),
            cost: vec![Colorless],
            damage: Basic(20),
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
    async fn hbp01_068() {
        // no need for testing: vanilla card
    }
}
