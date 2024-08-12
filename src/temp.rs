use std::sync::{Arc, OnceLock};

use crate::{card_effects::{ParseEffect, SerializeEffect, Trigger}, cards::*};
use HoloMemberHashTag::*;
use pretty_assertions::assert_eq;


pub fn test_library() -> &'static Arc<GlobalLibrary> {
    static TEST_LIBRARY: OnceLock<Arc<GlobalLibrary>> = OnceLock::new();
    TEST_LIBRARY.get_or_init(|| {

        let mut effects = vec![];
        let mut test_effect = |s| {effects.push(s); s} ;
        let mut conditions = vec![];
        let mut test_condition = |s| {conditions.push(s); s} ;

        // like load from file
        let mut lib = GlobalLibrary {
            cards: [
                // Cheers
                (
                    "hY01-001".into(),
                    Card::Cheer(CheerCard {
                        card_number: "hY01-001".into(),
                        name: "White Cheer".into(),
                        color: Color::White,
                        text: "⯀ When a holomem leaves the stage, archive all Cheer cards attached to them.\n⯀ When a holomem Baton Passes, archive a number of Cheer cards attached to them equal to the Baton Pass cost.".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "はずき".into(),
                    }),
                ),
                (
                    "hY02-001".into(),
                    Card::Cheer(CheerCard {
                        card_number: "hY02-001".into(),
                        name: "Green Cheer".into(),
                        color: Color::Green,
                        text: "⯀ When a holomem leaves the stage, archive all Cheer cards attached to them.\n⯀ When a holomem Baton Passes, archive a number of Cheer cards attached to them equal to the Baton Pass cost.".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "はずき".into(),
                    }),
                ),
                (
                    "hY03-001".into(),
                    Card::Cheer(CheerCard {
                        card_number: "hY03-001".into(),
                        name: "Red Cheer".into(),
                        color: Color::Red,
                        text: "⯀ When a holomem leaves the stage, archive all Cheer cards attached to them.\n⯀ When a holomem Baton Passes, archive a number of Cheer cards attached to them equal to the Baton Pass cost.".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "はずき".into(),
                    }),
                ),
                (
                    "hY04-001".into(),
                    Card::Cheer(CheerCard {
                        card_number: "hY04-001".into(),
                        name: "Blue Cheer".into(),
                        color: Color::Blue,
                        text: "⯀ When a holomem leaves the stage, archive all Cheer cards attached to them.\n⯀ When a holomem Baton Passes, archive a number of Cheer cards attached to them equal to the Baton Pass cost.".into(),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "はずき".into(),
                    }),
                ),
                // hSD01 - Tokino Sora & AZki Starter Deck
                // Oshi
                (
                    "hSD01-001".into(),
                    Card::OshiHoloMember(OshiHoloMemberCard {
                        card_number: "hSD01-001".into(),
                        name: "Tokino Sora".into(),
                        color: Color::White,
                        life: 5,
                        skills: vec![OshiSkill {
                            kind: OshiSkillKind::Normal,
                            name: "Replacement".into(),
                            cost: 1,
                            text: "[Once per turn] Move one Cheer card attached to one of your holomem to another of your holomem.".into(),
                            triggers: vec![],
                            condition: test_condition(r"
                                2 <= count from stage
                                any from stage has_cheers
                            ").parse_effect().expect("hSD01-001"),
                            effect: test_effect(r"
                                let $mem = select_one from stage is_member and has_cheers
                                let $cheer = select_one attached $mem is_cheer
                                let $to_mem = select_one from stage is_member and is_not $mem
                                attach_cards $cheer $to_mem
                            ").parse_effect().expect("hSD01-001"),
                        },
                        OshiSkill {
                            kind: OshiSkillKind::Special,
                            name: "So You're the Enemy?".into(),
                            cost: 2,
                            text: "[Once per game] Switch 1 of your opponent's Back position holomem with their Center position holomem. Until end of turn, your White Center position holomem have +50 to their Arts.".into(),
                            triggers: vec![],
                            condition: test_condition(r"
                                exist from opponent_center_stage
                                exist from opponent_back_stage
                            ").parse_effect().expect("hSD01-001"),
                            effect: test_effect(r"
                                let $back_mem = select_one from opponent_back_stage is_member
                                let $center_mem = from opponent_center_stage
                                send_to opponent_back_stage $center_mem
                                send_to opponent_center_stage $back_mem
                                add_zone_mod center_stage when is_color_white more_dmg 50 this_turn
                            ").parse_effect().expect("hSD01-001"),
                        }],
                        rarity: Rarity::OshiSuperRare,
                        illustration: "".into(),
                        artist: "でいりー".into(),
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
                            name: "In My Left Hand, a Map".into(),
                            cost: 0,
                            text: "[Once per turn] You may use this skill when one of your holomem's abilities instructs you to roll a six-sided die: Declare a number from 1 to 6. You may use the declared number as the result of your die roll.".into(),
                            triggers: vec![
                                Trigger::OnBeforeRollDice
                            ],
                            condition: test_condition(r"
                               all event_origin is_member and yours
                            ").parse_effect().expect("hSD01-002"),
                            effect: test_effect(r"
                                let $num = select_number_between 1 6
                                add_global_mod you next_dice_roll $num until_removed
                            ").parse_effect().expect("hSD01-002"),
                        },
                        OshiSkill {
                            kind: OshiSkillKind::Special,
                            name: "In My Right Hand, a Mic".into(),
                            cost: 3,
                            text: "[Once per game] Attach any number of Cheer cards from your Archive to one of your Green holomem.".into(),
                            triggers: vec![],
                            condition: test_condition(r"
                                 any from stage is_member and is_color_green
                            ").parse_effect().expect("hSD01-002"),
                            effect: test_effect(r"
                                let $cheers = select_any from archive is_cheer
                                let $mem = select_one from stage is_member and is_color_green
                                attach_cards $cheers $mem
                            ").parse_effect().expect("hSD01-002"),
                        }],
                        rarity: Rarity::OshiSuperRare,
                        illustration: "".into(),
                        artist: "Hachi".into(),
                    }),
                ),
                // Members
                (
                    "hSD01-003".into(),
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
                            cost: vec![Color::ColorLess],
                            damage: HoloMemberArtDamage::Basic(30),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        }],
                        extra: None,
                        attributes: vec![],
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "はこに".into(),
                    }),
                ),
                (
                    "hSD01-004".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "hSD01-004".into(),
                        name: "Tokino Sora".into(),
                        colors: vec![Color::White],
                        hp: 50,
                        level: HoloMemberLevel::Debut,
                        hash_tags: vec![JP, Gen0, Song],
                        baton_pass_cost: 1,
                        abilities: vec![HoloMemberAbility {
                            kind: MemberAbilityKind::CollabEffect,
                            name: "Let's Dance!".into(),
                            text: "Until end of turn, your Center position holomem gains +20 to their Arts.".into(),
                            condition: vec![],
                            effect: test_effect(r"
                                add_zone_mod center_stage more_dmg 20 this_turn
                            ").parse_effect().expect("hSD01-004"),
                        }],
                        arts: vec![HoloMemberArt {
                            name: "On Stage!".into(),
                            cost: vec![Color::ColorLess],
                            damage: HoloMemberArtDamage::Basic(20),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        }],
                        extra: None,
                        attributes: vec![],
                        rarity: Rarity::Rare,
                        illustration: "".into(),
                        artist: "".into(),
                    }),
                ),
                (
                    "hSD01-005".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "hSD01-005".into(),
                        name: "Tokino Sora".into(),
                        colors: vec![Color::White],
                        hp: 150,
                        level: HoloMemberLevel::First,
                        hash_tags: vec![JP, Gen0, Song],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        arts: vec![HoloMemberArt {
                            name: "Let's Nunnun".into(),
                            cost: vec![Color::White],
                            damage: HoloMemberArtDamage::Basic(30),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        },
                        HoloMemberArt {
                            name: "Your Heart... Will Go from Cloudy to Sunny!".into(),
                            cost: vec![Color::White, Color::ColorLess],
                            damage: HoloMemberArtDamage::Basic(50),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        }],
                        extra: None,
                        attributes: vec![],
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "おるだん".into(),
                    }),
                ),
                (
                    "hSD01-006".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "hSD01-006".into(),
                        name: "Tokino Sora".into(),
                        colors: vec![Color::White],
                        hp: 240,
                        level: HoloMemberLevel::First,
                        hash_tags: vec![JP, Gen0, Song],
                        baton_pass_cost: 2,
                        abilities: vec![],
                        arts: vec![HoloMemberArt {
                            name: "Dream Live".into(),
                            cost: vec![Color::White, Color::ColorLess],
                            damage: HoloMemberArtDamage::Basic(50),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        },
                        HoloMemberArt {
                            name: "SorAZ Sympathy".into(),
                            cost: vec![Color::White, Color::Green, Color::ColorLess],
                            damage: HoloMemberArtDamage::Plus(60),
                            special_damage: None,
                            text: "If a [AZKi] holomem is on your Stage, this Art deals 50 additional damage.".into(),
                            condition: vec![],
                            effect: test_effect(r"
                                if any from stage is_member and is_named_azki (
                                    add_mod this_card more_dmg 50 this_art
                                )
                            ").parse_effect().expect("hSD01-006"),
                        }],
                        extra: Some("When this holomem is Knocked Out, you lose 2 Life.".into()),
                        attributes: vec![HoloMemberExtraAttribute::Buzz],
                        rarity: Rarity::DoubleRare,
                        illustration: "".into(),
                        artist: "I☆LA".into(),
                    }),
                ),
                (
                    "hSD01-007".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "hSD01-007".into(),
                        name: "IRyS".into(),
                        colors: vec![Color::White],
                        hp: 50,
                        level: HoloMemberLevel::Debut,
                        hash_tags: vec![EN, Promise, Song],
                        baton_pass_cost: 1,
                        abilities: vec![HoloMemberAbility {
                            kind: MemberAbilityKind::CollabEffect,
                            name: "HOPE".into(),
                            text: "Look at your holoPOWER. You may reveal a card from among your holoPOWER and put it into your hand. Then put 1 card from your hand onto your holoPOWER.".into(),
                            condition: test_condition(r"
                                exist from holo_power
                            ").parse_effect().expect("hSD01-007"),
                            effect: test_effect(r"
                                let $choice = select_one from holo_power anything
                                reveal $choice
                                send_to hand $choice
                                let $hand = select_one from hand anything
                                send_to holo_power $hand
                            ").parse_effect().expect("hSD01-007"),
                        }],
                        arts: vec![HoloMemberArt {
                            name: "Avatar of Hope".into(),
                            cost: vec![Color::White],
                            damage: HoloMemberArtDamage::Basic(20),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        }],
                        extra: None,
                        attributes: vec![],
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                    }),
                ),
                (
                    "hSD01-008".into(),
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
                            cost: vec![Color::ColorLess],
                            damage: HoloMemberArtDamage::Basic(20),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        }],
                        extra: None,
                        attributes: vec![],
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "はこに".into(),
                    }),
                ),
                (
                    "hSD01-009".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "hSD01-009".into(),
                        name: "AZKi".into(),
                        colors: vec![Color::Green],
                        hp: 60,
                        level: HoloMemberLevel::Debut,
                        hash_tags: vec![JP, Gen0, Song],
                        baton_pass_cost: 1,
                        abilities: vec![HoloMemberAbility {
                            kind: MemberAbilityKind::CollabEffect,
                            name: "Expanding Map".into(),
                            text: "Roll a six-sided die: If the result is 4 or less, attach the top card of your Cheer Deck to one of your Back position holomem. If the result is 1, you may also move this holomem to the Backstage.".into(),
                            condition: vec![],
                            effect: test_effect(r"
                                let $roll = roll_dice
                                if (($roll <= 4) and exist from back_stage) (
                                    let $cheer = from_top 1 cheer_deck
                                    reveal $cheer
                                    let $back_mem = select_one from back_stage is_member
                                    attach_cards $cheer $back_mem
                                )
                                if $roll == 1 (
                                    let $option = optional_activate
                                    if $option (
                                        send_to back_stage this_card
                                    )
                                )
                            ").parse_effect().expect("hSD01-009"),
                        }],
                        arts: vec![HoloMemberArt {
                            name: "Where Next, Where Next?".into(),
                            cost: vec![Color::ColorLess],
                            damage: HoloMemberArtDamage::Basic(10),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        }],
                        extra: None,
                        attributes: vec![],
                        rarity: Rarity::Rare,
                        illustration: "".into(),
                        artist: "".into(),
                    }),
                ),
                (
                    "hSD01-010".into(),
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
                            cost: vec![Color::Green, Color::ColorLess],
                            damage: HoloMemberArtDamage::Basic(50),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        }],
                        extra: None,
                        attributes: vec![],
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "".into(),
                    }),
                ),
                (
                    "hSD01-011".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "hSD01-011".into(),
                        name: "AZKi".into(),
                        colors: vec![Color::Green],
                        hp: 190,
                        level: HoloMemberLevel::Second,
                        hash_tags: vec![JP, Gen0, Song],
                        baton_pass_cost: 2,
                        abilities: vec![],
                        arts: vec![HoloMemberArt {
                            name: "SorAZ Gravity".into(),
                            cost: vec![Color::Green],
                            damage: HoloMemberArtDamage::Basic(60),
                            special_damage: Some((Color::Blue, 50)),
                            text: "If there is a [Tokino Sora] holomem on your Stage, attach 1 card from the top of your Cheer Deck to one of your holomem.".into(),
                            condition: vec![],
                            effect: test_effect(r"
                                if any from stage is_member and is_named_tokino_sora (
                                    let $cheer = from_top 1 cheer_deck
                                    reveal $cheer
                                    let $mem = select_one from stage is_member
                                    attach_cards $cheer $mem
                                )
                            ").parse_effect().expect("hSD01-011"),
                        },
                        HoloMemberArt {
                            name: "Destiny Song".into(),
                            cost: vec![Color::Green, Color::Green, Color::ColorLess],
                            damage: HoloMemberArtDamage::Plus(100),
                            special_damage: Some((Color::Blue, 50)),
                            text: "Roll a six-sided die: If the result is odd, this Art gains +50 damage. If the result is 1, this Art gains an additional +50 damage.".into(),
                            condition: vec![],
                            effect: test_effect(r"
                                let $roll = roll_dice
                                if is_odd $roll (
                                    add_mod this_card more_dmg 50 this_art
                                )
                                if $roll == 1 (
                                    add_mod this_card more_dmg 50 this_art
                                )
                            ").parse_effect().expect("hSD01-011"),
                        }],
                        extra: None,
                        attributes: vec![],
                        rarity: Rarity::DoubleRare,
                        illustration: "".into(),
                        artist: "I☆LA".into(),
                    }),
                ),
                (
                    "hSD01-012".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "hSD01-012".into(),
                        name: "Airani Iofifteen".into(),
                        colors: vec![Color::Green],
                        hp: 70,
                        level: HoloMemberLevel::Debut,
                        hash_tags: vec![ID, IDGen1, Drawing],
                        baton_pass_cost: 1,
                        abilities: vec![HoloMemberAbility {
                            kind: MemberAbilityKind::CollabEffect,
                            name: "Let's Draw Together!".into(),
                            text: "Attach one {W} Cheer or {G} Cheer from your Archive to your Center position holomem.".into(),
                            condition: test_condition(r"
                                all from center_stage is_member
                            ").parse_effect().expect("hSD01-012"),
                            effect: test_effect(r"
                                let $cheer = select_one from archive is_cheer and (is_color_green or is_color_white)
                                let $mem = filter from center_stage is_member
                                attach_cards $cheer $mem
                            ").parse_effect().expect("hSD01-012"),
                        }],
                        arts: vec![HoloMemberArt {
                            name: "Drawing Is Fun!".into(),
                            cost: vec![Color::Green],
                            damage: HoloMemberArtDamage::Basic(20),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        }],
                        extra: None,
                        attributes: vec![],
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                    }),
                ),
                (
                    "hSD01-013".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "hSD01-013".into(),
                        name: "SorAZ".into(),
                        colors: vec![Color::White, Color::Green],
                        hp: 130,
                        level: HoloMemberLevel::First,
                        hash_tags: vec![JP, Gen0, Song],
                        baton_pass_cost: 1,
                        abilities: vec![],
                        arts: vec![HoloMemberArt {
                            name: "The Future We Want to Overcome".into(),
                            cost: vec![Color::ColorLess, Color::ColorLess],
                            damage: HoloMemberArtDamage::Basic(50),
                            special_damage: None,
                            text: "Roll a six-sided die: If the result is odd, attach one Cheer card from the top of your Cheer Deck to this holomem. If the result is even, draw a card.".into(),
                            condition: vec![],
                            effect: test_effect(r"
                                let $roll = roll_dice
                                if is_odd $roll (
                                    let $cheer = from_top 1 cheer_deck
                                    reveal $cheer
                                    attach_cards $cheer this_card
                                )
                                if is_even $roll (
                                    draw 1
                                )
                            ").parse_effect().expect("hSD01-013"),
                        }],
                        extra: Some("This card is treated as both [Tokino Sora] and [AZKi].".into()),
                        attributes: vec![
                            HoloMemberExtraAttribute::Name("Tokino Sora".into()),
                            HoloMemberExtraAttribute::Name("AZKi".into())
                        ],
                        rarity: Rarity::Rare,
                        illustration: "".into(),
                        artist: "©2023 Victor Entertainment".into(),
                    }),
                ),
                (
                    "hSD01-014".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "hSD01-014".into(),
                        name: "Amane Kanata".into(),
                        colors: vec![Color::ColorLess],
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
                        extra: Some("This holomem cannot Bloom.".into()),
                        attributes: vec![],
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "".into(),
                    }),
                ),
                (
                    "hSD01-015".into(),
                    Card::HoloMember(HoloMemberCard {
                        card_number: "hSD01-015".into(),
                        name: "Hakui Koyori".into(),
                        colors: vec![Color::ColorLess],
                        hp: 50,
                        level: HoloMemberLevel::Spot,
                        hash_tags: vec![JP, SecretSocietyholoX, AnimalEars],
                        baton_pass_cost: 1,
                        abilities: vec![HoloMemberAbility {
                            kind: MemberAbilityKind::CollabEffect,
                            name: "SoAzKo".into(),
                            text: "⯀ When this card collabs with [Tokino Sora], draw a card.\n⯀ When this card collabs with [AZKi], attach the top card of your Cheer Deck to your Center position holomem.".into(),
                            condition: vec![],
                            effect: test_effect(r"
                                let $center_mem = filter from center_stage is_member
                                if all $center_mem is_named_tokino_sora (
                                    draw 1
                                )
                                if all $center_mem is_named_azki (
                                    let $cheer = from_top 1 cheer_deck
                                    reveal $cheer
                                    attach_cards $cheer $center_mem
                                )
                            ").parse_effect().expect("hSD01-015"),
                        }],
                        arts: vec![HoloMemberArt {
                            name: "Pure, Pure, Pure!".into(),
                            cost: vec![Color::ColorLess],
                            damage: HoloMemberArtDamage::Basic(10),
                            special_damage: None,
                            text: "".into(),
                            condition: vec![],
                            effect: vec![],
                        }],
                        extra: Some("This holomem cannot Bloom.".into()),
                        attributes: vec![],
                        rarity: Rarity::Uncommon,
                        illustration: "".into(),
                        artist: "".into(),
                    }),
                ),
                // items
                (
                    "hSD01-016".into(),
                    Card::Support(SupportCard {
                        card_number: "hSD01-016".into(),
                        name: "Harusaki Nodoka".into(),
                        kind: SupportKind::Staff,
                        limited: true,
                        text: "Draw 3 cards.".into(),
                        attachment_condition: vec![],
                        triggers: vec![],
                        condition: vec![],
                        effect: test_effect(r"
                                draw 3
                            ").parse_effect().expect("hSD01-016"),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "Yoshimo".into(),
                    }),
                ),
                (
                    "hSD01-017".into(),
                    Card::Support(SupportCard {
                        card_number: "hSD01-017".into(),
                        name: "Mane-chan".into(),
                        kind: SupportKind::Staff,
                        limited: true,
                        text: "You can use this card only if you have 1 or more card in hand, not including this card.\n\n Shuffle your hand into your deck, then draw 5 cards.".into(),
                        attachment_condition: vec![],
                        triggers: vec![],
                        condition: test_condition(r"
                                1 <= count filter from hand is_not this_card
                            ").parse_effect().expect("hSD01-017"),
                        effect: test_effect(r"
                                let $hand = from hand
                                send_to main_deck $hand
                                shuffle main_deck
                                draw 5
                            ").parse_effect().expect("hSD01-017"),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "株式会社 HIKE / Trigono".into(),
                    }),
                ),
                (
                    "hSD01-018".into(),
                    Card::Support(SupportCard {
                        card_number: "hSD01-018".into(),
                        name: "Second PC".into(),
                        kind: SupportKind::Item,
                        limited: false,
                        text: "Look at the top 5 cards of your deck. You may reveal a LIMITED Support card from among them and put it into your hand. Put the rest on the bottom of your deck in any order.".into(),
                        attachment_condition: vec![],
                        triggers: vec![],
                        condition: vec![],
                        effect: test_effect(r"
                                let $top_5 = from_top 5 main_deck
                                let $limited = select_up_to 1 $top_5 is_support_limited
                                reveal $limited
                                send_to hand $limited
                                send_to_bottom main_deck $_leftovers
                            ").parse_effect().expect("hSD01-018"),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "JinArt こばやかわやまと".into(),
                    }),
                ),
                (
                    "hSD01-019".into(),
                    Card::Support(SupportCard {
                        card_number: "hSD01-019".into(),
                        name: "Amazing PC".into(),
                        kind: SupportKind::Item,
                        limited: true,
                        text: "You can use this card only if you Archive 1 Cheer card attached to your holomem.\n\n Search your deck for a non-Buzz 1st or 2nd holomem, reveal it, and put it into your hand. Then shuffle your deck.".into(),
                        attachment_condition: vec![],
                        triggers: vec![],
                        condition: test_condition(r"
                                any from stage has_cheers
                            ").parse_effect().expect("hSD01-019"),
                        effect: test_effect(r"
                                let $mem = select_one from stage is_member and has_cheers
                                let $cheer = select_one attached $mem is_cheer
                                send_to archive $cheer
                                let $cond = ((is_level_first or is_level_second) and not is_attribute_buzz) 
                                let $choice = select_one from main_deck $cond
                                reveal $choice
                                send_to hand $choice
                                shuffle main_deck
                            ").parse_effect().expect("hSD01-019"),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "JinArt KABAKURA".into(),
                    }),
                ),
                (
                    "hSD01-020".into(),
                    Card::Support(SupportCard {
                        card_number: "hSD01-020".into(),
                        name: "hololive Fan Circle".into(),
                        kind: SupportKind::Event,
                        limited: false,
                        text: "Roll a six-sided die: If the result is 3 or greater, attach a Cheer card from your Archive to one of your holomem.".into(),
                        attachment_condition: vec![],
                        triggers: vec![],
                        condition: vec![],
                        effect: test_effect(r"
                                let $roll = roll_dice
                                if $roll >= 3 (
                                    let $cheer = select_one from archive is_cheer
                                    if exist $cheer (
                                        let $mem = select_one from stage is_member
                                        attach_cards $cheer $mem
                                    )
                                )
                            ").parse_effect().expect("hSD01-020"),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "JinArt KABAKURA".into(),
                    }),
                ),
                (
                    "hSD01-021".into(),
                    Card::Support(SupportCard {
                        card_number: "hSD01-021".into(),
                        name: "First Gravity".into(),
                        kind: SupportKind::Event,
                        limited: true,
                        text: "You can use this card only if you have 6 or fewer cards in hand (not including this card). Look at the top 4 cards of your deck.\n\n You may reveal any number of [Tokino Sora] or [AZKi] holomem from among them and put the revealed cards into your hand. Put the rest on the bottom of your deck in any order.".into(),
                        attachment_condition: vec![],
                        triggers: vec![],
                        condition: test_condition(r"
                                6 >= count filter from hand is_not this_card
                            ").parse_effect().expect("hSD01-021"),
                        effect: test_effect(r"
                                let $top_4 = from_top 4 main_deck
                                let $mems = select_any $top_4 is_named_tokino_sora or is_named_azki
                                reveal $mems
                                send_to hand $mems
                                send_to_bottom main_deck $_leftovers
                            ").parse_effect().expect("hSD01-021"),
                        rarity: Rarity::Common,
                        illustration: "".into(),
                        artist: "".into(),
                    }),
                ),
            ]
            .into(),
        };

        // pre-process library with implicit conditions and effects
        lib.pre_process();

        
        // verify effect serialization consistency (ser -> de -> ser), check that tokens were used
        for string in effects {
            let effect = string.parse_effect::<CardEffect>().expect("should already be parsed above");
            let de_string = effect.serialize_effect();

            let string = string.replace(['(', ')', ' ', '\n','\r'], "");
            let de_string = de_string.replace(['(', ')', ' ', '\n','\r'], "");
            assert_eq!(string, de_string);
        }
        for string in conditions {
            let condition = string.parse_effect::<CardEffectCondition>().expect("should already be parsed above");
            let de_string = condition.serialize_effect();

            let string = string.replace(['(', ')', ' ', '\n','\r'], "");
            let de_string = de_string.replace(['(', ')', ' ', '\n','\r'], "");
            assert_eq!(string, de_string);
        }

        Arc::new(lib)
    })
}
