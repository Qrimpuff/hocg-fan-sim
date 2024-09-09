use hocg_fan_sim::{
    card_effects::*,
    cards::{HoloMemberHashTag::*, *},
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-015".into(),
        name: "Hakui Koyori".into(),
        colors: vec![Color::Colorless],
        hp: 50,
        level: HoloMemberLevel::Spot,
        hash_tags: vec![JP, SecretSocietyholoX, AnimalEars],
        baton_pass_cost: 1,
        abilities: vec![HoloMemberAbility {
            kind: MemberAbilityKind::CollabEffect,
            name: "SoAzKo".into(),
            text: "⯀ When this card collabs with [Tokino Sora], draw a card.\n⯀ When this card collabs with [AZKi], attach the top card of your Cheer Deck to your Center position holomem.".into(),
            condition: vec![],
            effect: (r"
                let $center_mem = filter from center_stage is_member
                if all $center_mem is_named_tokino_sora (
                    draw 1
                )
                if all $center_mem is_named_azki (
                    let $cheer = from_top 1 cheer_deck
                    reveal $cheer
                    attach_cards $cheer $center_mem
                )
            ").parse_effect().expect("hSD01-015"),
        }],
        arts: vec![HoloMemberArt {
            name: "Pure, Pure, Pure!".into(),
            cost: vec![Color::Colorless],
            damage: HoloMemberArtDamage::Basic(10),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![],
        rarity: Rarity::Uncommon,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-015.webp".into(),
        artist: "".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-015 - Hakui Koyori (Spot)
    async fn hsd01_015_sora() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-006".into()),
            back_stage: ["hSD01-015".into()].into(),
            life: ["hY01-001".into()].into(),
            main_deck: ["hSD01-015".into(), "hSD01-015".into()].into(),
            cheer_deck: ["hY01-001".into()].into(),
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
            // SoAzKo
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
        // assert_eq!(state, game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.collab = Some("c_0511".into());
        expected_state.player_1.back_stage = [].into();
        expected_state.player_1.holo_power = ["c_0211".into()].into();
        expected_state.player_1.main_deck = [].into();
        expected_state.player_1.hand = ["c_0311".into()].into();
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

        assert_eq!(expected_state, game.game.state);
    }

    #[tokio::test]
    /// hSD01-015 - Hakui Koyori (Spot)
    async fn hsd01_015_azki() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-009".into()),
            back_stage: ["hSD01-015".into()].into(),
            life: ["hY01-001".into()].into(),
            main_deck: ["hSD01-015".into(), "hSD01-015".into()].into(),
            cheer_deck: ["hY01-001".into()].into(),
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
            // SoAzKo
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
        // assert_eq!(state, game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.collab = Some("c_0511".into());
        expected_state.player_1.cheer_deck = [].into();
        expected_state.player_1.back_stage = [].into();
        expected_state.player_1.holo_power = ["c_0211".into()].into();
        expected_state.player_1.main_deck = ["c_0311".into()].into();
        expected_state
            .player_1
            .attachments
            .extend([(CardRef::from("c_0711"), "c_0411".into())]);
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

        assert_eq!(expected_state, game.game.state);
    }
    #[tokio::test]
    /// hSD01-015 - Hakui Koyori (Spot)
    async fn hsd01_015_soraz() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-013".into()),
            back_stage: ["hSD01-015".into()].into(),
            life: ["hY01-001".into()].into(),
            main_deck: ["hSD01-015".into(), "hSD01-015".into()].into(),
            cheer_deck: ["hY01-001".into()].into(),
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
            // SoAzKo
            &[0],
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
        // assert_eq!(state, game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.collab = Some("c_0511".into());
        expected_state.player_1.cheer_deck = [].into();
        expected_state.player_1.back_stage = [].into();
        expected_state.player_1.holo_power = ["c_0211".into()].into();
        expected_state.player_1.main_deck = [].into();
        expected_state.player_1.hand = ["c_0311".into()].into();
        expected_state
            .player_1
            .attachments
            .extend([(CardRef::from("c_0711"), "c_0411".into())]);
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

        assert_eq!(expected_state, game.game.state);
    }
}
