use hocg_fan_sim::cards::{HoloMemberHashTag::*, *};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-008".into(),
        name: "AZKi".into(),
        colors: vec![Color::Green],
        hp: 70,
        level: HoloMemberLevel::Debut,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "You're Great for Being Able to Do Your Best!".into(),
            cost: vec![Color::Colorless],
            damage: HoloMemberArtDamage::Basic(20),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![],
        rarity: Rarity::Common,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-008.webp"
            .into(),
        artist: "はこに".into(),
    })
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    /// hSD01-008 - AZKi (Debut)
    async fn hsd01_008() {
        // no need for testing: vanilla card
    }
}
