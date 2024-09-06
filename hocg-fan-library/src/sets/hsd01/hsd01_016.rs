use hocg_fan_sim::{card_effects::*, cards::*};

pub fn card() -> Card {
    Card::Support(SupportCard {
        card_number: "hSD01-016".into(),
        name: "Harusaki Nodoka".into(),
        kind: SupportKind::Staff,
        limited: true,
        text: "Draw 3 cards.".into(),
        attachment_condition: vec![],
        triggers: vec![],
        condition: vec![],
        effect: (r"
                draw 3
            ")
        .parse_effect()
        .expect("hSD01-016"),
        rarity: Rarity::Common,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-016.webp".into(),
        artist: "Yoshimo".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-016 - Harusaki Nodoka (Staff)
    async fn hsd01_016() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-013".into()),
            hand: ["hSD01-016".into()].into(),
            life: ["hY01-001".into()].into(),
            main_deck: [
                "hSD01-015".into(),
                "hSD01-015".into(),
                "hSD01-015".into(),
                "hSD01-015".into(),
            ]
            .into(),
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
            // Harusaki Nodoka
            &[0],
            // done
            &[3],
        ]);
        let p2_p = BufferedPrompter::new(&[]);

        let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p);
        tokio::spawn(p1_client.receive_requests());
        tokio::spawn(p2_client.receive_requests());

        // performance step
        game.next_step().await.unwrap();

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.main_deck = ["c_0511".into()].into();
        expected_state.player_1.hand = ["c_0211".into(), "c_0311".into(), "c_0411".into()].into();
        expected_state.player_1.archive = ["c_0811".into()].into();
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

        assert_eq!(expected_state, game.state);
    }
}
