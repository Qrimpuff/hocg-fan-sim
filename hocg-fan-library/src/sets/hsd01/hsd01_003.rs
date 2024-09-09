use hocg_fan_sim::cards::{HoloMemberHashTag::*, *};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-003".into(),
        name: "Tokino Sora".into(),
        colors: vec![Color::White],
        hp: 60,
        level: HoloMemberLevel::Debut,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "(๑╹ᆺ╹) Nun nun".into(),
            cost: vec![Color::Colorless],
            damage: HoloMemberArtDamage::Basic(30),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![],
        rarity: Rarity::Common,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-003.webp"
            .into(),
        artist: "はこに".into(),
    })
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    /// hSD01-003 - Tokino Sora (Debut)
    async fn hsd01_003() {
        // no need for testing: vanilla card
    }
}
