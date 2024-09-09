use hocg_fan_sim::cards::{
    Color::*, HoloMemberArtDamage::*, HoloMemberExtraAttribute::*, HoloMemberHashTag::*,
    HoloMemberLevel::*, Rarity::*, *,
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-048".into(),
        name: "Kazama Iroha".into(),
        colors: vec![Green],
        hp: 120,
        level: Debut,
        hash_tags: vec![JP, SecretSocietyholoX],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "Everyone, Kazama Iroha Here, I Daresay".into(),
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
        artist: "".into(),
    })
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn hbp01_048() {
        // no need for testing: vanilla card
    }
}
