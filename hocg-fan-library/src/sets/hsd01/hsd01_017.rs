use hocg_fan_sim::{card_effects::*, cards::*};

pub fn card() -> Card {
    Card::Support(SupportCard {
        card_number: "hSD01-017".into(),
        name: "Mane-chan".into(),
        kind: SupportKind::Staff,
        limited: true,
        text: "You can use this card only if you have 1 or more card in hand, not including this card.\n\n Shuffle your hand into your deck, then draw 5 cards.".into(),
        attachment_condition: vec![],
        triggers: vec![],
        condition: (r"
                1 <= count filter from hand is_not this_card
            ").parse_effect().expect("hSD01-017"),
        effect: (r"
                let $hand = from hand
                send_to main_deck $hand
                shuffle main_deck
                draw 5
            ").parse_effect().expect("hSD01-017"),
        rarity: Rarity::Common,
        illustration_url: "/hocg-fan-sim-assets/img/hSD01/hSD01-017.webp".into(),
        artist: "株式会社 HIKE / Trigono".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-017 - Mane-chan (Staff)
    async fn hsd01_017() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-013".into()),
            hand: [
                "hSD01-017".into(),
                "hSD01-018".into(),
                "hSD01-019".into(),
                "hSD01-020".into(),
            ]
            .into(),
            life: ["hY01-001".into()].into(),
            main_deck: [
                "hSD01-010".into(),
                "hSD01-011".into(),
                "hSD01-012".into(),
                "hSD01-013".into(),
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
            // Mane-chan
            &[0],
            // done
            &[5],
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
        expected_state.player_1.main_deck = ["c_0b11".into(), "c_0a11".into()].into();
        expected_state.player_1.hand = [
            "c_0911".into(),
            "c_0511".into(),
            "c_0311".into(),
            "c_0211".into(),
            "c_0411".into(),
        ]
        .into();
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
