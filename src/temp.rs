use std::sync::{Arc, OnceLock};

use crate::{cards::*, ParseEffect, Trigger};

pub fn test_library() -> &'static Arc<GlobalLibrary> {
    static TEST_LIBRARY: OnceLock<Arc<GlobalLibrary>> = OnceLock::new();
    TEST_LIBRARY.get_or_init(|| {
        // like load from file
        let mut lib = GlobalLibrary {
            cards: [
                // Cheers
                (
                    "White-Cheer".into(),
                    Card::Cheer(CheerCard {
                        card_number: "White-Cheer".into(),
                        name: "White Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::White,
                        text: "White Cheer".into(),
                    }),
                ),
                (
                    "Green-Cheer".into(),
                    Card::Cheer(CheerCard {
                        card_number: "Green-Cheer".into(),
                        name: "Green Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Green,
                        text: "Green Cheer".into(),
                    }),
                ),
                (
                    "Blue-Cheer".into(),
                    Card::Cheer(CheerCard {
                        card_number: "Blue-Cheer".into(),
                        name: "Blue Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Blue,
                        text: "Blue Cheer".into(),
                    }),
                ),
                (
                    "Red-Cheer".into(),
                    Card::Cheer(CheerCard {
                        card_number: "Red-Cheer".into(),
                        name: "Red Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Red,
                        text: "Red Cheer".into(),
                    }),
                ),
                (
                    "Purple-Cheer".into(),
                    Card::Cheer(CheerCard {
                        card_number: "Purple-Cheer".into(),
                        name: "Purple Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Purple,
                        text: "Purple Cheer".into(),
                    }),
                ),
                (
                    "Yellow-Cheer".into(),
                    Card::Cheer(CheerCard {
                        card_number: "Yellow-Cheer".into(),
                        name: "Yellow Cheer".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Yellow,
                        text: "Yellow Cheer".into(),
                    }),
                ),
                // Oshi
                (
                    "Sora-Oshi".into(),
                    Card::OshiHoloMember(OshiHoloMemberCard {
                        card_number: "Sora-Oshi".into(),
                        name: "Sora".into(),
                        rarity: Rarity::OshiSuperRare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::White,
                        life: 5,
                        skills: vec![OshiSkill {
                            kind: OshiSkillKind::Normal,
                            name: "Sora dance".into(),
                            cost: 1,
                            trigger: vec![],
                            condition: vec![],
                            effect: "draw 1".parse_effect().expect("const"),
                            text: "draw 1 card".into(),
                        },
                        OshiSkill {
                            kind: OshiSkillKind::Special,
                            name: "Sora super dance".into(),
                            cost: 1,
                            trigger: vec![],
                            condition: vec![],
                            effect: "draw 3".parse_effect().expect("const"),
                            text: "draw 3 card".into(),
                        }],
                    }),
                ),
                (
                    "AZKi-Oshi".into(),
                    Card::OshiHoloMember(OshiHoloMemberCard {
                        card_number: "AZKi-Oshi".into(),
                        name: "AZKi".into(),
                        rarity: Rarity::OshiSuperRare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Green,
                        life: 5,
                        skills: vec![OshiSkill {
                            kind: OshiSkillKind::Normal,
                            name: "AZKi sing".into(),
                            cost: 1,
                            trigger: vec![],
                            condition: vec![],
                            effect: "draw 1".parse_effect().expect("const"),
                            text: "draw 1 card".into(),
                        },
                        OshiSkill {
                            kind: OshiSkillKind::Special,
                            name: "AZKi super sing".into(),
                            cost: 1,
                            trigger: vec![],
                            condition: vec![],
                            effect: "draw 3".parse_effect().expect("const"),
                            text: "draw 3 card".into(),
                        }],
                    }),
                ),
                (
                    "hSD01-002".into(),
                    Card::OshiHoloMember(OshiHoloMemberCard {
                        card_number: "hSD01-002".into(),
                        name: "AZKi".into(),
                        color: Color::Green,
                        life: 6,
                        skills: vec![OshiSkill {
                            kind: OshiSkillKind::Normal,
                            name: "Map on the left".into(),
                            cost: 3,
                            text: "[Once per turn] Can be used when rolling dice using you Holomem's ability: Declare the number of the dice, and treat the next number that appears as the declared number".into(),
                            trigger: vec![Trigger::OnBeforeDiceRoll],
                            condition: vec!["once_per_turn and is_holo_member".parse_effect().expect("const")],
                            effect: "next_dice_number select_dice_number".parse_effect().expect("const"),
                        },
                        OshiSkill {
                            kind: OshiSkillKind::Special,
                            name: "Microphone on right hand".into(),
                            cost: 3,
                            text: "[Once per game] Send as many cheers from your archive as you like to one of your Green Holomem".into(),
                            trigger: vec![],
                            condition: vec!["once_per_game".parse_effect().expect("const")],
                            effect: "for (select_member members_on_stage with color_green) attach (select_cheers_up_to all cheers_in_archive)".parse_effect().expect("const"),
                        }],
                        rarity: Rarity::OshiSuperRare,
                        illustration: "".into(),
                        artist: "Hachi".into(),
                    }),
                ),
                // Members
                (
                    "Sora-Debut".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "Sora-Debut".into(),
                        name: "Sora".into(),
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::White,
                        hp: 50,
                        level: HoloMemberLevel::Debut,
                        tags: vec![HoloMemberHashTag::Generation0],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        arts: vec![HoloMemberArt {
                            name: "attack 1".into(),
                            cost: vec![Color::ColorLess],
                            condition: vec![],
                            damage_modifier: vec![],
                            effect: vec![],
                            text: "do something".into(),
                            damage: HoloMemberArtDamage::Basic(10),
                        }],
                        extra: None,
                        attributes: vec![],
                    }),
                ),
                (
                    "Sora-1".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "Sora-1".into(),
                        name: "Sora".into(),
                        rarity: Rarity::Rare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::White,
                        hp: 80,
                        level: HoloMemberLevel::First,
                        tags: vec![HoloMemberHashTag::Generation0],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        arts: vec![HoloMemberArt {
                            name: "attack 1-b".into(),
                            cost: vec![Color::White],
                            damage_modifier: vec![],
                            condition: vec![],
                            effect: vec![],
                            text: "do something".into(),
                            damage: HoloMemberArtDamage::Basic(30),
                        }],
                        extra: None,
                        attributes: vec![],
                    }),
                ),
                (
                    "Sora-2".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "Sora-2".into(),
                        name: "Sora".into(),
                        rarity: Rarity::DoubleRare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::White,
                        hp: 120,
                        level: HoloMemberLevel::Second,
                        tags: vec![HoloMemberHashTag::Generation0],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        arts: vec![
                            HoloMemberArt {
                                name: "attack 1-w".into(),
                                cost: vec![Color::White],
                                condition: vec![],
                                damage_modifier: vec![],
                                effect: vec![],
                                text: "do something".into(),
                                damage: HoloMemberArtDamage::Basic(40),
                            },
                            HoloMemberArt {
                                name: "attack 1-x".into(),
                                cost: vec![Color::White, Color::White],
                                condition: vec![],
                                damage_modifier: vec![],
                                effect: vec![],
                                text: "do something".into(),
                                damage: HoloMemberArtDamage::Basic(60),
                            },
                        ],
                        extra: None,
                        attributes: vec![],
                    }),
                ),
                (
                    "AZKi-Debut".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "AZKi-Debut".into(),
                        name: "AZKi".into(),
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Green,
                        hp: 50,
                        level: HoloMemberLevel::Debut,
                        tags: vec![HoloMemberHashTag::Generation0],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        arts: vec![HoloMemberArt {
                            name: "attack 1".into(),
                            cost: vec![Color::ColorLess],
                            damage_modifier: vec![],
                            condition: vec![],
                            effect: vec![],
                            text: "do something".into(),
                            damage: HoloMemberArtDamage::Basic(20),
                        }],
                        extra: None,
                        attributes: vec![],
                    }),
                ),
                (
                    "AZKi-1".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "AZKi-1".into(),
                        name: "AZKi".into(),
                        rarity: Rarity::Rare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Green,
                        hp: 80,
                        level: HoloMemberLevel::First,
                        tags: vec![HoloMemberHashTag::Generation0],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        arts: vec![HoloMemberArt {
                            name: "attack 1-b".into(),
                            cost: vec![Color::Green],
                            damage_modifier: vec![],
                            condition: vec![],
                            effect: vec![],
                            text: "do something".into(),
                            damage: HoloMemberArtDamage::Basic(40),
                        }],
                        extra: None,
                        attributes: vec![],
                    }),
                ),
                (
                    "AZKi-2".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "AZKi-2".into(),
                        name: "AZKi".into(),
                        rarity: Rarity::DoubleRare,
                        illustration: "".into(),
                        artist: "".into(),
                        color: Color::Green,
                        hp: 120,
                        level: HoloMemberLevel::Second,
                        tags: vec![HoloMemberHashTag::Generation0],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        arts: vec![
                            HoloMemberArt {
                                name: "attack 1-w".into(),
                                cost: vec![Color::Green],
                                condition: vec![],
                                damage_modifier: vec![],
                                effect: vec![],
                                text: "do something".into(),
                                damage: HoloMemberArtDamage::Basic(40),
                            },
                            HoloMemberArt {
                                name: "attack 1-x".into(),
                                cost: vec![Color::Green, Color::Green],
                                condition: vec![],
                                damage_modifier: vec![],
                                effect: vec![],
                                text: "do something".into(),
                                damage: HoloMemberArtDamage::Basic(50),
                            },
                        ],
                        extra: None,
                        attributes: vec![],
                    }),
                ),
                // items
                (
                    "Support-1".into(),
                    Card::Support(SupportCard {
                        card_number: "Support-1".into(),
                        name: "Heal".into(),
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "".into(),
                        kind: SupportKind::Item,
                        limited_use: false,
                        condition: vec![],
                        effect: "for center_member heal 10".parse_effect().expect("const"),
                        text: "some support".into(),
                    }),
                ),
                (
                    "Support-2".into(),
                    Card::Support(SupportCard {
                        card_number: "Support-2".into(),
                        name: "Draw".into(),
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "".into(),
                        kind: SupportKind::Item,
                        limited_use: true,
                        condition: vec![],
                        effect: "draw 2".parse_effect().expect("const"),
                        text: "some support".into(),
                    }),
                ),
            ]
            .into(),
        };

        // pre-process library with implicit conditions and effects
        lib.pre_process();

        Arc::new(lib)
    })
}
