use hocg_fan_sim::{card_effects::*, cards::*};

pub fn card() -> Card {
    Card::OshiHoloMember(OshiHoloMemberCard {
        card_number: "hSD01-002".into(),
        name: "AZKi".into(),
        color: Color::Green,
        life: 6,
        skills: vec![OshiSkill {
            kind: OshiSkillKind::Normal,
            name: "In My Left Hand, a Map".into(),
            cost: 3,
            text: "[Once per turn] You may use this skill when one of your holomem's abilities instructs you to roll a six-sided die: Declare a number from 1 to 6. You may use the declared number as the result of your die roll.".into(),
            triggers: vec![
                Trigger::OnBeforeRollDice
            ],
            condition: (r"
               all event_origin is_member and yours
            ").parse_effect().expect("hSD01-002"),
            effect: (r"
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
            condition: (r"
                 any from stage is_member and is_color_green
            ").parse_effect().expect("hSD01-002"),
            effect: (r"
                let $cheers = select_any from archive is_cheer
                let $mem = select_one from stage is_member and is_color_green
                attach_cards $cheers $mem
            ").parse_effect().expect("hSD01-002"),
        }],
        rarity: Rarity::OshiSuperRare,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-002.webp".into(),
        artist: "Hachi".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-002 - AZKi (Oshi)
    async fn hsd01_002() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hSD01-003".into()),
            back_stage: ["hSD01-009".into()].into(),
            life: ["hY01-001".into()].into(),
            holo_power: [
                "hSD01-005".into(),
                "hSD01-005".into(),
                "hSD01-005".into(),
                "hSD01-005".into(),
                "hSD01-005".into(),
                "hSD01-005".into(),
            ]
            .into(),
            archive: ["hY01-001".into(), "hY01-001".into(), "hY01-001".into()].into(),
            ..Default::default()
        };
        let p2 = p1.clone();

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Cheer)
            .with_player_1(p1)
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // In My Left Hand, a Map
            &[0],
            &[0],
            &[5],
            // In My Right Hand, a Mic
            &[0],
            &[0, 1, 2],
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

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.state);

        let mut expected_state = state.clone();
        // expected_state.game_outcome
        // expected_state.card_map
        // expected_state.player_1
        expected_state.player_1.collab = Some("c_0311".into());
        expected_state.player_1.back_stage = [].into();
        expected_state.player_1.holo_power = [].into();
        expected_state.player_1.archive = [
            "c_0a11".into(),
            "c_0911".into(),
            "c_0811".into(),
            "c_0711".into(),
            "c_0611".into(),
            "c_0511".into(),
        ]
        .into();
        expected_state.player_1.attachments.extend([
            (CardRef::from("c_0d11"), "c_0311".into()),
            ("c_0b11".into(), "c_0311".into()),
            ("c_0c11".into(), "c_0311".into()),
        ]);
        // expected_state.player_2
        // expected_state.active_player
        // expected_state.active_step
        expected_state.active_step = Step::Main;
        // expected_state.turn_number
        // expected_state.zone_modifiers
        expected_state
            .zone_modifiers
            .entry(Player::One)
            .or_default()
            .extend([(
                Zone::All,
                Modifier {
                    id: "m_0001".into(),
                    kind: ModifierKind::PreventCollab,
                    life_time: LifeTime::ThisTurn,
                },
            )]);
        // expected_state.card_modifiers
        expected_state
            .card_modifiers
            .entry("c_0111".into())
            .or_default()
            .extend([
                Modifier {
                    id: "m_0003".into(),
                    kind: ModifierKind::PreventOshiSkill(0),
                    life_time: LifeTime::ThisTurn,
                },
                Modifier {
                    id: "m_0004".into(),
                    kind: ModifierKind::PreventOshiSkill(1),
                    life_time: LifeTime::ThisGame,
                },
            ]);
        // expected_state.card_damage_markers
        // expected_state.event_span

        assert_eq!(expected_state, game.game.state);
    }

    #[tokio::test]
    /// hSD01-002 - AZKi (Oshi)
    async fn hsd01_002_not_enough_holo_power() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hSD01-003".into()),
            back_stage: ["hSD01-009".into()].into(),
            life: ["hY01-001".into()].into(),
            holo_power: [].into(),
            archive: ["hY01-001".into(), "hY01-001".into(), "hY01-001".into()].into(),
            ..Default::default()
        };
        let p2 = p1.clone();

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Cheer)
            .with_player_1(p1)
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // Can't use, In My Left Hand, a Map
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

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.state);

        let mut expected_state = state.clone();
        // expected_state.game_outcome
        // expected_state.card_map
        // expected_state.player_1
        expected_state.player_1.collab = Some("c_0311".into());
        expected_state.player_1.back_stage = [].into();
        expected_state.player_1.holo_power = [].into();
        // expected_state.player_2
        // expected_state.active_player
        // expected_state.active_step
        expected_state.active_step = Step::Main;
        // expected_state.turn_number
        // expected_state.zone_modifiers
        expected_state
            .zone_modifiers
            .entry(Player::One)
            .or_default()
            .extend([(
                Zone::All,
                Modifier {
                    id: "m_0001".into(),
                    kind: ModifierKind::PreventCollab,
                    life_time: LifeTime::ThisTurn,
                },
            )]);
        // expected_state.card_modifiers
        // expected_state.card_damage_markers
        // expected_state.event_span

        assert_eq!(expected_state, game.game.state);
    }
}
