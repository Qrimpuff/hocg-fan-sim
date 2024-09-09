use hocg_fan_sim::{
    card_effects::*,
    cards::{HoloMemberHashTag::*, *},
};

pub fn card() -> Card {
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
            effect: (r"
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
            cost: vec![Color::Green, Color::Green, Color::Colorless],
            damage: HoloMemberArtDamage::Plus(100),
            special_damage: Some((Color::Blue, 50)),
            text: "Roll a six-sided die: If the result is odd, this Art gains +50 damage. If the result is 1, this Art gains an additional +50 damage.".into(),
            condition: vec![],
            effect: (r"
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
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-011.webp".into(),
        artist: "I☆LA".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-011 - AZKi (Second)
    async fn hsd01_011_art_1() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-011".into()),
            back_stage: ["hSD01-006".into()].into(),
            life: ["hY01-001".into()].into(),
            cheer_deck: ["hY02-001".into()].into(),
            ..Default::default()
        };
        let p2 = p1.clone();

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Main)
            .with_player_1(p1)
            .with_attachments(
                Player::One,
                Zone::CenterStage,
                0,
                ["hY02-001".into(), "hY01-001".into(), "hY01-001".into()].into(),
            )
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // SorAZ Gravity
            &[0],
            &[1],
            // done
            &[0],
        ]);
        let p2_p = BufferedPrompter::new(&[]);

        let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p).await;
        tokio::spawn(p1_client.receive_requests());
        tokio::spawn(p2_client.receive_requests());

        // performance step
        game.next_step().await.unwrap();

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Performance;
        expected_state.player_1.cheer_deck = [].into();
        expected_state
            .player_1
            .attachments
            .extend([(CardRef::from("c_0511"), "c_0311".into())]);
        expected_state
            .card_modifiers
            .entry("c_0211".into())
            .or_default()
            .extend([Modifier {
                id: "m_0001".into(),
                kind: ModifierKind::PreventAllArts,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0212".into(), DamageMarkers::from_hp(60));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.game.state);
    }

    #[tokio::test]
    /// hSD01-011 - AZKi (Second)
    async fn hsd01_011_art_2() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hSD01-011".into()),
            back_stage: ["hSD01-006".into()].into(),
            holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            life: ["hY01-001".into()].into(),
            ..Default::default()
        };
        let mut p2 = p1.clone();
        p2.center_stage = Some("hSD01-006".into());

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Main)
            .with_player_1(p1)
            .with_attachments(
                Player::One,
                Zone::CenterStage,
                0,
                ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            )
            .with_zone_modifiers([(Player::One, vec![])].into())
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // Destiny Song
            &[1],
            &[0],
            &[0],
            // done
            &[0],
        ]);
        let p2_p = BufferedPrompter::new(&[]);

        let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p).await;
        tokio::spawn(p1_client.receive_requests());
        tokio::spawn(p2_client.receive_requests());

        // performance step
        let _ = game.next_step().await;

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Performance;
        expected_state.player_1.holo_power = [].into();
        expected_state.player_1.archive =
            ["c_0711".into(), "c_0611".into(), "c_0511".into()].into();
        expected_state
            .card_modifiers
            .entry("c_0111".into())
            .or_default()
            .extend([Modifier {
                id: "m_0002".into(),
                kind: ModifierKind::PreventOshiSkill(0),
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_modifiers
            .entry("c_0211".into())
            .or_default()
            .extend([Modifier {
                id: "m_0005".into(),
                kind: ModifierKind::PreventAllArts,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0212".into(), DamageMarkers::from_hp(200));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.game.state);
    }
}
