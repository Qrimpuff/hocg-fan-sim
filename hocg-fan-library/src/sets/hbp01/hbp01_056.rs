use hocg_fan_sim::cards::{
    Color::*, HoloMemberArtDamage::*, HoloMemberExtraAttribute::*, HoloMemberHashTag::*,
    HoloMemberLevel::*, Rarity::*, *,
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-056".into(),
        name: "Takane Lui".into(),
        colors: vec![Red],
        hp: 100,
        level: Debut,
        hash_tags: vec![JP, SecretSocietyholoX, Bird, Alcohol],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "Did I Luive You Waiting?".into(),
            cost: vec![Colorless],
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
    async fn hbp01_056() {
        // no need for testing: vanilla card
    }
}
