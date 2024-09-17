use hocg_fan_sim::{card_effects::ParseEffect, card_effects::Trigger, cards::*};

pub fn card() -> Card {
    Card::Support(SupportCard {
        card_number: "hSD01-021".into(),
        name: "First Gravity".into(),
        kind: SupportKind::Event,
        limited: true,
        text: "You can use this card only if you have 6 or fewer cards in hand (not including this card). Look at the top 4 cards of your deck.\n\n You may reveal any number of [Tokino Sora] or [AZKi] holomem from among them and put the revealed cards into your hand. Put the rest on the bottom of your deck in any order.".into(),
        effects: vec![SupportEffect {
            triggers: vec![Trigger::PlayFromHand],
            condition: (r"
                    6 >= count filter from hand is_not_card this_card
                ").parse_effect().expect("hSD01-021"),
            effect: (r"
                    let $top_4 = from_top 4 main_deck
                    let $mems = select_any $top_4 is_named_tokino_sora or is_named_azki
                    reveal $mems
                    send_to hand $mems
                    send_to_bottom main_deck leftovers
                ").parse_effect().expect("hSD01-021"),
        }],
        rarity: Rarity::Common,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-021_C.webp".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-021 - First Gravity (Event)
    async fn hsd01_021() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-013".into()),
            hand: ["hSD01-021".into()].into(),
            life: ["hY01-001".into()].into(),
            main_deck: [
                "hSD01-003".into(),
                "hSD01-004".into(),
                "hSD01-005".into(),
                "hSD01-006".into(),
                "hSD01-007".into(),
                "hSD01-008".into(),
                "hSD01-009".into(),
                "hSD01-10".into(),
                "hSD01-11".into(),
                "hSD01-12".into(),
                "hSD01-13".into(),
                "hSD01-14".into(),
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
            // First Gravity
            &[0],
            &[0, 1],
            // done
            &[2],
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
            "c_0611".into(),
            "c_0711".into(),
            "c_0811".into(),
            "c_0911".into(),
            "c_0a11".into(),
            "c_0b11".into(),
            "c_0c11".into(),
            "c_0d11".into(),
            "c_0411".into(),
            "c_0511".into(),
        ]
        .into();
        expected_state.player_1.hand = ["c_0211".into(), "c_0311".into()].into();
        expected_state.player_1.archive = ["c_1011".into()].into();
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
