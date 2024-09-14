use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{HoloMemberHashTag::*, *},
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-005".into(),
        name: "Tokino Sora".into(),
        colors: vec![Color::White],
        hp: 150,
        level: HoloMemberLevel::First,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![
            HoloMemberArt {
                name: "Let's Nunnun".into(),
                cost: vec![Color::White],
                damage: HoloMemberArtDamage::Basic(30),
                special_damage: None,
                text: "".into(),
                condition: (r"").parse_effect().expect("hSD01-005"),
                effect: (r"").parse_effect().expect("hSD01-005"),
            },
            HoloMemberArt {
                name: "Your Heart... Will Go from Cloudy to Sunny!".into(),
                cost: vec![Color::White, Color::Colorless],
                damage: HoloMemberArtDamage::Basic(50),
                special_damage: None,
                text: "".into(),
                condition: (r"").parse_effect().expect("hSD01-005"),
                effect: (r"").parse_effect().expect("hSD01-005"),
            },
        ],
        attributes: vec![],
        rarity: Rarity::Uncommon,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-005.webp"
            .into(),
        artist: "おるだん".into(),
    })
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    /// hSD01-005 - Tokino Sora (First)
    async fn hsd01_005() {
        // no need for testing: vanilla card
    }
}
