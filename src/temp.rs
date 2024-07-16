use std::sync::{Arc, OnceLock};

use crate::{cards::*, Action, ParseEffect};

pub fn test_library() -> &'static Arc<GlobalLibrary> {
    static TEST_LIBRARY: OnceLock<Arc<GlobalLibrary>> = OnceLock::new();
    TEST_LIBRARY.get_or_init(|| {
        Arc::new(GlobalLibrary {
            cards: [
                // Cheers
                (
                    "White-Cheer".into(),
                    Card::Cheer(CheerCard {
                        id: "White-Cheer".into(),
                        name: "White Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::White,
                        trigger: None,
                        condition: None,
                        effect: None,
                        text: "White Cheer".into(),
                    }),
                ),
                (
                    "Green-Cheer".into(),
                    Card::Cheer(CheerCard {
                        id: "Green-Cheer".into(),
                        name: "Green Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Green,
                        trigger: None,
                        condition: None,
                        effect: None,
                        text: "Green Cheer".into(),
                    }),
                ),
                (
                    "Blue-Cheer".into(),
                    Card::Cheer(CheerCard {
                        id: "Blue-Cheer".into(),
                        name: "Blue Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Blue,
                        trigger: None,
                        condition: None,
                        effect: None,
                        text: "Blue Cheer".into(),
                    }),
                ),
                (
                    "Red-Cheer".into(),
                    Card::Cheer(CheerCard {
                        id: "Red-Cheer".into(),
                        name: "Red Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Red,
                        trigger: None,
                        condition: None,
                        effect: None,
                        text: "Red Cheer".into(),
                    }),
                ),
                (
                    "Purple-Cheer".into(),
                    Card::Cheer(CheerCard {
                        id: "Purple-Cheer".into(),
                        name: "Purple Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Purple,
                        trigger: None,
                        condition: None,
                        effect: None,
                        text: "Purple Cheer".into(),
                    }),
                ),
                (
                    "Yellow-Cheer".into(),
                    Card::Cheer(CheerCard {
                        id: "Yellow-Cheer".into(),
                        name: "Yellow Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Yellow,
                        trigger: None,
                        condition: None,
                        effect: None,
                        text: "Yellow Cheer".into(),
                    }),
                ),
                // Oshi
                (
                    "Sora-Oshi".into(),
                    Card::OshiHoloMember(OshiHoloMemberCard {
                        id: "Sora-Oshi".into(),
                        name: "Sora".into(),
                        rarity: Rarity::OshiSuperRare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::White,
                        life: 5,
                        abilities: vec![OshiAbility {
                            kind: OshiAbilityKind::Unknown,
                            name: "Sora dance".into(),
                            cost: 1,
                            trigger: None,
                            condition: None,
                            effect: "draw 1".parse_effect().expect("static"),
                            text: "draw 1 card".into(),
                        }],
                    }),
                ),
                (
                    "AZKi-Oshi".into(),
                    Card::OshiHoloMember(OshiHoloMemberCard {
                        id: "AZKi-Oshi".into(),
                        name: "AZKi".into(),
                        rarity: Rarity::OshiSuperRare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Green,
                        life: 5,
                        abilities: vec![OshiAbility {
                            kind: OshiAbilityKind::Unknown,
                            name: "AZKi sing".into(),
                            cost: 1,
                            trigger: None,
                            condition: None,
                            effect: "draw 1".parse_effect().expect("static"),
                            text: "draw 1 card".into(),
                        }],
                    }),
                ),
                // Members
                (
                    "Sora-Debut".into(),
                    Card::HoloMember(HoloMemberCard {
                        id: "Sora-Debut".into(),
                        name: "Sora".into(),
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::White,
                        hp: 50,
                        rank: HoloMemberRank::Debut,
                        tags: vec!["gen 0".into()],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        attacks: vec![MemberAttack {
                            name: "attack 1".into(),
                            cost: "1 white".into(),
                            trigger: None,
                            condition: None,
                            effect: vec![Action::Noop],
                            text: "do something".into(),
                            damage: 30,
                        }],
                    }),
                ),
                (
                    "Sora-1".into(),
                    Card::HoloMember(HoloMemberCard {
                        id: "Sora-1".into(),
                        name: "Sora".into(),
                        rarity: Rarity::Rare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::White,
                        hp: 80,
                        rank: HoloMemberRank::First,
                        tags: vec!["gen 0".into()],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        attacks: vec![MemberAttack {
                            name: "attack 1b".into(),
                            cost: "1 white".into(),
                            trigger: None,
                            condition: None,
                            effect: vec![Action::Noop],
                            text: "do something".into(),
                            damage: 50,
                        }],
                    }),
                ),
                (
                    "Sora-2".into(),
                    Card::HoloMember(HoloMemberCard {
                        id: "Sora-2".into(),
                        name: "Sora".into(),
                        rarity: Rarity::DoubleRare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::White,
                        hp: 120,
                        rank: HoloMemberRank::Second,
                        tags: vec!["gen 0".into()],
                        baton_pass_cost: 1,
                        abilities: vec![MemberAbility {
                            kind: MemberAbilityKind::Unknown,
                            name: "ability 1".into(),
                            trigger: None,
                            condition: None,
                            effect: vec![Action::Noop],
                            text: "do something".into(),
                        }],
                        attacks: vec![MemberAttack {
                            name: "attack 1x".into(),
                            cost: "1 white".into(),
                            trigger: None,
                            condition: None,
                            effect: vec![Action::Noop],
                            text: "do something".into(),
                            damage: 100,
                        }],
                    }),
                ),
                (
                    "AZKi-Debut".into(),
                    Card::HoloMember(HoloMemberCard {
                        id: "AZKi-Debut".into(),
                        name: "AZKi".into(),
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Green,
                        hp: 50,
                        rank: HoloMemberRank::Debut,
                        tags: vec!["gen 0".into()],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        attacks: vec![MemberAttack {
                            name: "attack 1".into(),
                            cost: "1 green".into(),
                            trigger: None,
                            condition: None,
                            effect: vec![Action::Noop],
                            text: "do something".into(),
                            damage: 30,
                        }],
                    }),
                ),
                (
                    "AZKi-1".into(),
                    Card::HoloMember(HoloMemberCard {
                        id: "AZKi-1".into(),
                        name: "AZKi".into(),
                        rarity: Rarity::Rare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Green,
                        hp: 80,
                        rank: HoloMemberRank::First,
                        tags: vec!["gen 0".into()],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        attacks: vec![MemberAttack {
                            name: "attack 1b".into(),
                            cost: "1 green".into(),
                            trigger: None,
                            condition: None,
                            effect: vec![Action::Noop],
                            text: "do something".into(),
                            damage: 50,
                        }],
                    }),
                ),
                (
                    "AZKi-2".into(),
                    Card::HoloMember(HoloMemberCard {
                        id: "AZKi-2".into(),
                        name: "AZKi".into(),
                        rarity: Rarity::DoubleRare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Green,
                        hp: 120,
                        rank: HoloMemberRank::Second,
                        tags: vec!["gen 0".into()],
                        baton_pass_cost: 1,
                        abilities: vec![MemberAbility {
                            kind: MemberAbilityKind::Unknown,
                            name: "ability 1".into(),
                            trigger: None,
                            condition: None,
                            effect: vec![Action::Noop],
                            text: "do something".into(),
                        }],
                        attacks: vec![MemberAttack {
                            name: "attack 1x".into(),
                            cost: "1 green".into(),
                            trigger: None,
                            condition: None,
                            effect: vec![Action::Noop],
                            text: "do something".into(),
                            damage: 100,
                        }],
                    }),
                ),
                // items
                (
                    "Support-1".into(),
                    Card::Support(SupportCard {
                        id: "Support-1".into(),
                        name: "Support 1".into(),
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "".into(),
                        kind: SupportKind::Unknown,
                        trigger: None,
                        condition: None,
                        effect: vec![Action::Noop],
                        text: "some support".into(),
                    }),
                ),
            ]
            .into(),
        })
    })
}
