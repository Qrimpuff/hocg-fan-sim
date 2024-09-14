use hocg_fan_sim::{card_effects::ParseEffect, card_effects::Trigger, cards::*};

pub fn card() -> Card {
    Card::Support(SupportCard {
        card_number: "hSD01-018".into(),
        name: "Second PC".into(),
        kind: SupportKind::Item,
        limited: false,
        text: "Look at the top 5 cards of your deck. You may reveal a LIMITED Support card from among them and put it into your hand. Put the rest on the bottom of your deck in any order.".into(),
        effects: vec![SupportEffect {
            triggers: vec![Trigger::PlayFromHand],
            condition: vec![],
            effect: (r"
                    let $top_5 = from_top 5 main_deck
                    let $limited = select_up_to 1 $top_5 is_support_limited
                    reveal $limited
                    send_to hand $limited
                    send_to_bottom main_deck leftovers
                ").parse_effect().expect("hSD01-018"),
        }],
        rarity: Rarity::Common,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-018.webp".into(),
        artist: "JinArt こばやかわやまと".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-018 - Second PC (Item)
    async fn hsd01_018() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-013".into()),
            hand: ["hSD01-018".into()].into(),
            life: ["hY01-001".into()].into(),
            main_deck: [
                "hSD01-017".into(),
                "hSD01-019".into(),
                "hSD01-020".into(),
                "hSD01-010".into(),
                "hSD01-011".into(),
                "hSD01-017".into(),
                "hSD01-019".into(),
                "hSD01-020".into(),
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
            // Second PC
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
        game.next_step().await.unwrap();

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.main_deck = [
            "c_0711".into(),
            "c_0811".into(),
            "c_0911".into(),
            "c_0a11".into(),
            "c_0b11".into(),
            "c_0c11".into(),
            "c_0d11".into(),
            "c_0311".into(),
            "c_0411".into(),
            "c_0511".into(),
            "c_0611".into(),
        ]
        .into();
        expected_state.player_1.hand = ["c_0211".into()].into();
        expected_state.player_1.archive = ["c_1011".into()].into();

        assert_eq!(expected_state, game.game.state);
    }
}
