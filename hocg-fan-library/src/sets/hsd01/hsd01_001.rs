use hocg_fan_sim::{card_effects::ParseEffect, cards::*};

pub fn card() -> Card {
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
                condition: (r"
                    2 <= count from stage
                    any from stage has_cheers
                ").parse_effect().unwrap(),
                effect: (r"
                    let $mem = select_one from stage is_member and has_cheers
                    let $cheer = select_one attached $mem is_cheer
                    let $to_mem = select_one from stage is_member and is_not $mem
                    attach_cards $cheer $to_mem
                ").parse_effect().unwrap(),
            },
            OshiSkill {
                kind: OshiSkillKind::Special,
                name: "So You're the Enemy?".into(),
                cost: 2,
                text: "[Once per game] Switch 1 of your opponent's Back position holomem with their Center position holomem. Until end of turn, your White Center position holomem have +50 to their Arts.".into(),
                triggers: vec![],
                condition: (r"
                    exist from opponent_center_stage
                    exist from opponent_back_stage
                ").parse_effect().unwrap(),
                effect: (r"
                    let $back_mem = select_one from opponent_back_stage is_member
                    let $center_mem = from opponent_center_stage
                    send_to opponent_back_stage $center_mem
                    send_to opponent_center_stage $back_mem
                    add_zone_mod center_stage when is_color_white more_dmg 50 this_turn
                ").parse_effect().unwrap(),
            }],
            rarity: Rarity::OshiSuperRare,
            illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-001.webp".into(),
            artist: "でいりー".into(),
        })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{
        card_effects::Condition, gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*,
    };
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-001 - Tokino Sora (Oshi)
    async fn hsd01_001() {
        // // --------------- setup logs ---------------------
        // env::set_var("RUST_BACKTRACE", "1");
        // env::set_var("RUST_LOG", "DEBUG");

        // let file_appender = tracing_appender::rolling::daily("logs", "hocg-fan-sim.log");
        // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        // tracing_subscriber::fmt()
        //     .with_timer(LocalTime::new(format_description!(
        //         "[year]-[month]-[day] [hour repr:24]:[minute]:[second].[subsecond digits:4]"
        //     )))
        //     .with_writer(non_blocking)
        //     .with_ansi(false)
        //     // enable thread id to be emitted
        //     .with_thread_ids(true)
        //     .with_env_filter(EnvFilter::from_default_env())
        //     .init();
        // // -------------- end setup logs -------------------

        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-003".into()),
            back_stage: ["hSD01-004".into()].into(),
            life: ["hY01-001".into()].into(),
            holo_power: ["hSD01-005".into(), "hSD01-005".into(), "hSD01-005".into()].into(),
            ..Default::default()
        };
        let p2 = p1.clone();

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Cheer)
            .with_player_1(p1)
            .with_attachments(
                Player::One,
                Zone::CenterStage,
                0,
                ["hY01-001".into()].into(),
            )
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // Replacement
            &[2],
            &[0],
            &[0],
            &[0],
            // So You're the Enemy?
            &[1],
            &[0],
            // done
            &[1],
        ]);
        let p2_p = BufferedPrompter::new(&[]);

        let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p);
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
        expected_state.player_1.holo_power = [].into();
        expected_state.player_1.archive =
            ["c_0711".into(), "c_0611".into(), "c_0511".into()].into();
        expected_state
            .player_1
            .attachments
            .insert("c_0831".into(), "c_0311".into());
        // expected_state.player_2
        expected_state.player_2.center_stage = Some("c_0312".into());
        expected_state.player_2.back_stage = ["c_0212".into()].into();
        // expected_state.active_player
        // expected_state.active_step
        expected_state.active_step = Step::Main;
        // expected_state.turn_number
        // expected_state.zone_modifiers
        expected_state
            .zone_modifiers
            .entry(Player::One)
            .or_default()
            .push((
                Zone::CenterStage,
                Modifier {
                    id: "m_0002".into(),
                    kind: ModifierKind::Conditional(
                        Condition::IsColorWhite,
                        Box::new(ModifierKind::MoreDamage(50)),
                    ),
                    life_time: LifeTime::ThisTurn,
                },
            ));
        // expected_state.card_modifiers
        expected_state
            .card_modifiers
            .entry("c_0111".into())
            .or_default()
            .push(Modifier {
                id: "m_0001".into(),
                kind: ModifierKind::PreventOshiSkill(0),
                life_time: LifeTime::ThisTurn,
            });
        expected_state
            .card_modifiers
            .entry("c_0111".into())
            .or_default()
            .push(Modifier {
                id: "m_0003".into(),
                kind: ModifierKind::PreventOshiSkill(1),
                life_time: LifeTime::ThisGame,
            });
        // expected_state.card_damage_markers
        // expected_state.event_span

        assert_eq!(expected_state, game.state);
    }
}
