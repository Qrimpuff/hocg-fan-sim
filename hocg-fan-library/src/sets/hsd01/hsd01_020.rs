use hocg_fan_sim::{card_effects::*, cards::*};

pub fn card() -> Card {
    Card::Support(SupportCard {
        card_number: "hSD01-020".into(),
        name: "hololive Fan Circle".into(),
        kind: SupportKind::Event,
        limited: false,
        text: "Roll a six-sided die: If the result is 3 or greater, attach a Cheer card from your Archive to one of your holomem.".into(),
        attachment_condition: vec![],
        triggers: vec![],
        condition: vec![],
        effect: (r"
                let $roll = roll_dice
                if $roll >= 3 (
                    let $cheer = select_one from archive is_cheer
                    if exist $cheer (
                        let $mem = select_one from stage is_member
                        attach_cards $cheer $mem
                    )
                )
            ").parse_effect().expect("hSD01-020"),
        rarity: Rarity::Common,
        illustration_url: "/hocg-fan-sim-assets/img/hSD01/hSD01-020.webp".into(),
        artist: "JinArt KABAKURA".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-020 - hololive Fan Circle (Event)
    async fn hsd01_020() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hSD01-013".into()),
            hand: ["hSD01-020".into()].into(),
            holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            life: ["hY01-001".into()].into(),
            archive: ["hY01-001".into()].into(),
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
            // hololive Fan Circle
            &[0],
            // rolled a 5
            &[0],
            &[0],
            // done
            &[1],
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
        expected_state.player_1.hand = [].into();
        expected_state.player_1.archive = ["c_0811".into()].into();
        expected_state
            .player_1
            .attachments
            .extend([(CardRef::from("c_0711"), "c_0211".into())]);

        assert_eq!(expected_state, game.state);
    }
}
