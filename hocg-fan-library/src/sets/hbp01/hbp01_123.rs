use hocg_fan_sim::{card_effects::ParseEffect, card_effects::Trigger, cards::*};

pub fn card() -> Card {
    Card::Support(SupportCard {
        card_number: "hBP01-123".into(),
        name: "Nousagi Alliance".into(),
        kind: SupportKind::Fan,
        limited: false,
        text: r#"When an ability of holomem this fan is attached to caused you to roll a six sided die, you may archive this fan: Reroll the die once.\n\nThis fan may only be attached to "Usada Pekora". You may attach any number of copies of this fan to a single holomem."#.into(),
        effects: vec![SupportEffect {
            triggers: vec![Trigger::Attach],
            condition: (r"
                all attach_target is_named_usada_pekora
            ").parse_effect().expect("hBP01-123"),
            effect: vec![],
        },
        SupportEffect {
            triggers: vec![Trigger::OnAfterRollDice],
            condition: (r"
                any attached_to event_origin is_card this_card
            ").parse_effect().expect("hBP01-123"),
            effect: (r"
                let $option = optional_activate
                if $option (
                    let $roll = roll_dice
                    add_global_mod you next_dice_roll $roll until_removed
                    send_to archive this_card
                )
            ").parse_effect().expect("hBP01-123"),
        }],
        rarity: Rarity::Common,
        illustration_url: "".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn hbp01_123() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hBP01-038".into()),
            back_stage: ["hSD01-006".into()].into(),
            hand: ["hBP01-123".into()].into(),
            holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            life: ["hY01-001".into()].into(),
            ..Default::default()
        };
        let mut p2 = p1.clone();
        p2.center_stage = Some("hSD01-006".into());

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Cheer)
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
            // attach
            &[0],
            &[0],
            // done main
            &[4],
            // art to roll dice (rolled 5)
            &[0],
            // activate reroll (rolled 6)
            &[0],
            // done
            &[0],
        ]);
        let p2_p = BufferedPrompter::new(&[]);

        let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p).await;
        tokio::spawn(p1_client.receive_requests());
        tokio::spawn(p2_client.receive_requests());

        // main step
        game.next_step().await.unwrap();

        // performance step
        game.next_step().await.unwrap();

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Performance;
        expected_state.player_1.hand = [].into();
        expected_state.player_1.archive = ["c_0811".into()].into();
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
            .insert("c_0212".into(), DamageMarkers::from_hp(40));

        assert_eq!(expected_state, game.game.state);
    }
}
