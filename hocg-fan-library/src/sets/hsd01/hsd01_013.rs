use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{HoloMemberHashTag::*, *},
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-013".into(),
        name: "SorAZ".into(),
        colors: vec![Color::White, Color::Green],
        hp: 130,
        level: HoloMemberLevel::First,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "The Future We Want to Overcome".into(),
            cost: vec![Color::Colorless, Color::Colorless],
            damage: HoloMemberArtDamage::Basic(50),
            special_damage: None,
            text: "Roll a six-sided die: If the result is odd, attach one Cheer card from the top of your Cheer Deck to this holomem. If the result is even, draw a card.".into(),
            condition: vec![],
            effect: (r"
                let $roll = roll_dice
                if is_odd $roll (
                    let $cheer = from_top 1 cheer_deck
                    reveal $cheer
                    attach_cards $cheer this_card
                )
                if is_even $roll (
                    draw 1
                )
            ").parse_effect().expect("hSD01-013"),
        }],
        attributes: vec![
            HoloMemberExtraAttribute::Name("Tokino Sora".into()),
            HoloMemberExtraAttribute::Name("AZKi".into())
        ],
        rarity: Rarity::Rare,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-013_R.webp".into(),
        artist: "©2023 Victor Entertainment".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-013 - SorAZ (First)
    async fn hsd01_013_odd() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hSD01-013".into()),
            back_stage: ["hSD01-006".into()].into(),
            holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            life: ["hY01-001".into()].into(),
            main_deck: ["hSD01-006".into()].into(),
            cheer_deck: ["hY01-001".into()].into(),
            ..Default::default()
        };
        let mut p2 = p1.clone();
        p2.center_stage = Some("hSD01-006".into());

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Main)
            .with_player_1(p1)
            .with_attachments(
                Player::One,
                Zone::CenterStage,
                0,
                ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            )
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // The Future We Want to Overcome
            &[0],
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
        let _ = game.next_step().await;

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Performance;
        expected_state.player_1.cheer_deck = [].into();
        expected_state.player_1.holo_power = [].into();
        expected_state.player_1.archive =
            ["c_0911".into(), "c_0811".into(), "c_0711".into()].into();
        expected_state
            .player_1
            .attachments
            .extend([(CardRef::from("c_0611"), "c_0311".into())]);
        expected_state
            .card_modifiers
            .entry("c_0111".into())
            .or_default()
            .extend([Modifier {
                id: "m_0002".into(),
                kind: ModifierKind::PreventOshiSkill(0),
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_modifiers
            .entry("c_0311".into())
            .or_default()
            .extend([Modifier {
                id: "m_0003".into(),
                kind: ModifierKind::PreventAllArts,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0312".into(), DamageMarkers::from_hp(50));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.game.state);
    }

    #[tokio::test]
    /// hSD01-013 - SorAZ (First)
    async fn hsd01_013_even() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hSD01-013".into()),
            back_stage: ["hSD01-006".into()].into(),
            holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            life: ["hY01-001".into()].into(),
            main_deck: ["hSD01-006".into()].into(),
            cheer_deck: ["hY01-001".into()].into(),
            ..Default::default()
        };
        let mut p2 = p1.clone();
        p2.center_stage = Some("hSD01-006".into());

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Main)
            .with_player_1(p1)
            .with_attachments(
                Player::One,
                Zone::CenterStage,
                0,
                ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            )
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // The Future We Want to Overcome
            &[0],
            &[0],
            &[1],
            // done
            &[0],
        ]);
        let p2_p = BufferedPrompter::new(&[]);

        let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p).await;
        tokio::spawn(p1_client.receive_requests());
        tokio::spawn(p2_client.receive_requests());

        // performance step
        let _ = game.next_step().await;

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Performance;
        expected_state.player_1.main_deck = [].into();
        expected_state.player_1.holo_power = [].into();
        expected_state.player_1.archive =
            ["c_0911".into(), "c_0811".into(), "c_0711".into()].into();
        expected_state.player_1.hand = ["c_0211".into()].into();
        expected_state
            .card_modifiers
            .entry("c_0111".into())
            .or_default()
            .extend([Modifier {
                id: "m_0002".into(),
                kind: ModifierKind::PreventOshiSkill(0),
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_modifiers
            .entry("c_0311".into())
            .or_default()
            .extend([Modifier {
                id: "m_0003".into(),
                kind: ModifierKind::PreventAllArts,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0312".into(), DamageMarkers::from_hp(50));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.game.state);
    }
}
