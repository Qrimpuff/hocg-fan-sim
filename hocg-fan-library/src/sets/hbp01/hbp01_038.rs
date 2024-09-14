use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{
        Color::*, HoloMemberArtDamage::*, HoloMemberExtraAttribute::*, HoloMemberHashTag::*,
        HoloMemberLevel::*, Rarity::*, *,
    },
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-038".into(),
        name: "Usada Pekora".into(),
        colors: vec![Green],
        hp: 90,
        level: Debut,
        hash_tags: vec![JP, Gen3, AnimalEars],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "Konpeko!".into(),
            cost: vec![Green],
            damage: Plus(20),
            special_damage: None,
            text: "Roll a six-sided die: If the result is even, this Art gains +20 damage.".into(),
            condition: vec![],
            effect: (r"
                let $roll = roll_dice
                if is_even $roll (
                    add_mod this_card deal_more_dmg 20 this_art
                )
            ")
            .parse_effect()
            .expect("hBP01-038"),
        }],
        attributes: vec![Unlimited],
        rarity: Common,
        illustration_url: "".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::*, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn hbp01_038_odd() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hBP01-038".into()),
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
            // Konpeko!
            &[0],
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
                id: "m_0003".into(),
                kind: ModifierKind::PreventAllArts,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0212".into(), DamageMarkers::from_hp(20));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.game.state);
    }

    #[tokio::test]
    async fn hbp01_038_even() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hBP01-038".into()),
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
            // Konpeko!
            &[0],
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
            .insert("c_0212".into(), DamageMarkers::from_hp(40));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.game.state);
    }
}
