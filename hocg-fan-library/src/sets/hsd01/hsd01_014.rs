use hocg_fan_sim::cards::{HoloMemberHashTag::*, *};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-014".into(),
        name: "Amane Kanata".into(),
        colors: vec![Color::Colorless],
        hp: 150,
        level: HoloMemberLevel::Spot,
        hash_tags: vec![JP, Gen4, Song],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "Hey".into(),
            cost: vec![Color::White, Color::Green],
            damage: HoloMemberArtDamage::Basic(30),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![],
        rarity: Rarity::Uncommon,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-014.webp"
            .into(),
        artist: "".into(),
    })
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    /// hSD01-014 - Amane Kanata (Spot)
    async fn hsd01_014() {
        // no need for testing: vanilla card
    }
}
