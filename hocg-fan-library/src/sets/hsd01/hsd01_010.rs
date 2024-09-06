use hocg_fan_sim::cards::{HoloMemberHashTag::*, *};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-010".into(),
        name: "AZKi".into(),
        colors: vec![Color::Green],
        hp: 160,
        level: HoloMemberLevel::First,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "An Aimless Journey with You".into(),
            cost: vec![Color::Green, Color::Colorless],
            damage: HoloMemberArtDamage::Basic(50),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        extra: None,
        attributes: vec![],
        rarity: Rarity::Uncommon,
        illustration_url: "/hocg-fan-sim-assets/img/hSD01/hSD01-010.webp".into(),
        artist: "".into(),
    })
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    /// hSD01-010 - AZKi (First)
    async fn hsd01_010() {
        // no need for testing: vanilla card
    }
}
