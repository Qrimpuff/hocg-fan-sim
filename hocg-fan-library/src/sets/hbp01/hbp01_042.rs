use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{
        Color::*, HoloMemberArtDamage::*, HoloMemberHashTag::*, HoloMemberLevel::*, Rarity::*, *,
    },
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-042".into(),
        name: "Usada Pekora".into(),
        colors: vec![Green],
        hp: 120,
        level: First,
        hash_tags: vec![JP, Gen3, AnimalEars],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![
            HoloMemberArt {
                name: "The White Beach and the Rabbit Girl".into(),
                cost: vec![Green],
                damage: Basic(40),
                special_damage: None,
                text: "".into(),
                condition: vec![],
                effect: vec![],
            },
            HoloMemberArt {
                name: "It'a Hereee".into(),
                cost: vec![Green, Green],
                damage: Plus(50),
                special_damage: None,
                text: "Roll a six-sided die: This Art deals additional damage equal to the number you rolled times 10.".into(),
                condition: vec![],
                effect: (r"
                    let $roll = roll_dice
                    add_mod this_card deal_more_dmg ($roll * 10) this_art
                ").parse_effect().expect("hBP01-042"),
            },
        ],
        attributes: vec![],
        rarity: Rare,
        illustration_url: "".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::*, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn hbp01_042_roll_6() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hBP01-042".into()),
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
                ["hY02-001".into(), "hY02-001".into()].into(),
            )
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // It'a Hereee
            &[1],
            &[0],
            &[5],
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
        // assert_eq!(state, game.game.state);

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
                id: "m_0004".into(),
                kind: ModifierKind::PreventAllArts,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0212".into(), DamageMarkers::from_hp(110));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.game.state);
    }

    #[tokio::test]
    async fn hbp01_042_roll_3() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hBP01-042".into()),
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
                ["hY02-001".into(), "hY02-001".into()].into(),
            )
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // It'a Hereee
            &[1],
            &[0],
            &[2],
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
        // assert_eq!(state, game.game.state);

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
                id: "m_0004".into(),
                kind: ModifierKind::PreventAllArts,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0212".into(), DamageMarkers::from_hp(80));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.game.state);
    }
}
