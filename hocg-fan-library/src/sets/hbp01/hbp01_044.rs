use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{
        Color::*, HoloMemberArtDamage::*, HoloMemberExtraAttribute::*, HoloMemberHashTag::*,
        HoloMemberLevel::*, Rarity::*, *,
    },
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-044".into(),
        name: "AZKi".into(),
        colors: vec![Green],
        hp: 100,
        level: Debut,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "Kon-AZKi".into(),
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
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::*, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn hbp01_044() {
        // no need for testing: vanilla card
    }
}
