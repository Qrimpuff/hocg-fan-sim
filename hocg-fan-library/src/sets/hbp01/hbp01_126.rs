use hocg_fan_sim::{card_effects::ParseEffect, card_effects::Trigger, cards::*};

pub fn card() -> Card {
    Card::Support(SupportCard {
        card_number: "hBP01-126".into(),
        name: "Troupe Member".into(),
        kind: SupportKind::Fan,
        limited: false,
        text: r#"When the holomem this Fan is attached to uses Arts, this Fan may be treated as a {R} Cheer.\n\nWhen the holomem this Fan is attached to receives damage, they receive 10 more damage.\n\nYou may only attach this Fan to "Omaru Polka". You may attach any number of copies of this Fan to a single holomem."#.into(),
        effects: vec![SupportEffect {
            triggers: vec![Trigger::Attach],
            condition: (r"
                all attach_target is_named_omaru_polka
            ").parse_effect().expect("hBP01-126"),
            effect: (r"
                add_mod this_card as_art_cost 1 red while_attached this_card
                add_mod attach_target recv_more_dmg 10 while_attached this_card
            ").parse_effect().expect("hBP01-126"),
        },
        SupportEffect {
            triggers: vec![Trigger::OnBeforePerformArt],
            condition: (r"
                any attached_to event_origin is_card this_card
            ").parse_effect().expect("hBP01-126"),
            effect: (r"
                add_mod this_card as_cheer 1 red this_art
            ").parse_effect().expect("hBP01-126"),
        }],
        rarity: Rarity::Common,
        illustration_url: "".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{
        cards::Color, gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*,
    };
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn hbp01_126() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hBP01-068".into()),
            back_stage: ["hSD01-006".into()].into(),
            hand: ["hBP01-126".into()].into(),
            holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            life: ["hY01-001".into()].into(),
            ..Default::default()
        };
        let p2 = p1.clone();

        let state = GameStateBuilder::new()
            .with_active_player(Player::Two)
            .with_active_step(Step::Cheer)
            .with_player_1(p1)
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // attach
            &[0],
            &[0],
            // done main
            &[3],
            // art
            &[0],
            // done
            &[0],
        ]);
        let p2_p = BufferedPrompter::new(&[
            // attach
            &[0],
            &[0],
            // done
            &[3],
        ]);

        let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p).await;
        tokio::spawn(p1_client.receive_requests());
        tokio::spawn(p2_client.receive_requests());

        // attach for p2
        game.next_step().await.unwrap();

        // main step
        game.game.state.active_step = Step::Cheer;
        game.game.state.active_player = Player::One;
        game.next_step().await.unwrap();

        // performance step
        game.next_step().await.unwrap();

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.game.state);

        let mut expected_state = state.clone();
        expected_state.active_player = Player::One;
        expected_state.active_step = Step::Performance;
        expected_state.player_1.hand = [].into();
        expected_state.player_1.attachments = [("c_0811".into(), "c_0211".into())].into();
        expected_state.player_2.hand = [].into();
        expected_state.player_2.attachments = [("c_0812".into(), "c_0212".into())].into();
        // p1 troupe
        expected_state
            .card_modifiers
            .entry("c_0811".into())
            .or_default()
            .extend([Modifier {
                id: "m_0003".into(),
                kind: ModifierKind::AsArtCost(Color::Red, 1),
                life_time: LifeTime::WhileAttached("c_0811".into()),
            }]);
        // p1 polka
        expected_state
            .card_modifiers
            .entry("c_0211".into())
            .or_default()
            .extend([
                Modifier {
                    id: "m_0004".into(),
                    kind: ModifierKind::ReceiveMoreDamage(10),
                    life_time: LifeTime::WhileAttached("c_0811".into()),
                },
                Modifier {
                    id: "m_0006".into(),
                    kind: ModifierKind::PreventAllArts,
                    life_time: LifeTime::ThisTurn,
                },
            ]);
        // p2 troupe
        expected_state
            .card_modifiers
            .entry("c_0812".into())
            .or_default()
            .extend([Modifier {
                id: "m_0001".into(),
                kind: ModifierKind::AsArtCost(Color::Red, 1),
                life_time: LifeTime::WhileAttached("c_0812".into()),
            }]);
        // p2 polka
        expected_state
            .card_modifiers
            .entry("c_0212".into())
            .or_default()
            .extend([Modifier {
                id: "m_0002".into(),
                kind: ModifierKind::ReceiveMoreDamage(10),
                life_time: LifeTime::WhileAttached("c_0812".into()),
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0212".into(), DamageMarkers::from_hp(30));

        assert_eq!(expected_state, game.game.state);
    }
}
