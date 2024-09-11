use hocg_fan_sim::{card_effects::*, cards::*};

pub fn card() -> Card {
    Card::Support(SupportCard {
        card_number: "hSD01-019".into(),
        name: "Amazing PC".into(),
        kind: SupportKind::Item,
        limited: true,
        text: "You can use this card only if you Archive 1 Cheer card attached to your holomem.\n\n Search your deck for a non-Buzz 1st or 2nd holomem, reveal it, and put it into your hand. Then shuffle your deck.".into(),
        attachment_condition: vec![],
        triggers: vec![],
        condition: (r"
                any from stage has_cheers
            ").parse_effect().expect("hSD01-019"),
        effect: (r"
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
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-019.webp".into(),
        artist: "JinArt KABAKURA".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-019 - Amazing PC (Item)
    async fn hsd01_019() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-013".into()),
            hand: ["hSD01-019".into()].into(),
            life: ["hY01-001".into()].into(),
            main_deck: [
                "hSD01-010".into(),
                "hSD01-011".into(),
                "hSD01-012".into(),
                "hSD01-013".into(),
                "hSD01-014".into(),
            ]
            .into(),
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
            // Amazing PC
            &[0],
            &[0],
            &[0],
            &[0],
            // done
            &[1],
        ]);
        let p2_p = BufferedPrompter::new(&[]);

        let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p).await;
        tokio::spawn(p1_client.receive_requests());
        tokio::spawn(p2_client.receive_requests());

        // performance step
        game.next_step().await.unwrap();

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.main_deck = [
            "c_0511".into(),
            "c_0311".into(),
            "c_0411".into(),
            "c_0611".into(),
        ]
        .into();
        expected_state.player_1.hand = ["c_0211".into()].into();
        expected_state.player_1.archive = ["c_0911".into(), "c_0a31".into()].into();
        expected_state.player_1.attachments = [].into();
        expected_state
            .zone_modifiers
            .entry(Player::One)
            .or_default()
            .extend([(
                Zone::All,
                Modifier {
                    id: "m_0001".into(),
                    kind: ModifierKind::PreventLimitedSupport,
                    life_time: LifeTime::ThisTurn,
                },
            )]);

        assert_eq!(expected_state, game.game.state);
    }
}
