use hocg_fan_sim::{
    card_effects::*,
    cards::{HoloMemberHashTag::*, *},
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-006".into(),
        name: "Tokino Sora".into(),
        colors: vec![Color::White],
        hp: 240,
        level: HoloMemberLevel::First,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 2,
        abilities: vec![],
        arts: vec![
            HoloMemberArt {
                name: "Dream Live".into(),
                cost: vec![Color::White, Color::Colorless],
                damage: HoloMemberArtDamage::Basic(50),
                special_damage: None,
                text: "".into(),
                condition: vec![],
                effect: vec![],
            },
            HoloMemberArt {
                name: "SorAZ Sympathy".into(),
                cost: vec![Color::White, Color::Green, Color::Colorless],
                damage: HoloMemberArtDamage::Plus(60),
                special_damage: None,
                text: "If a [AZKi] holomem is on your Stage, this Art deals 50 additional damage."
                    .into(),
                condition: vec![],
                effect: (r"
                if any from stage is_member and is_named_azki (
                    add_mod this_card more_dmg 50 this_art
                )
            ")
                .parse_effect()
                .expect("hSD01-006"),
            },
        ],
        extra: Some("When this holomem is Knocked Out, you lose 2 Life.".into()),
        attributes: vec![HoloMemberExtraAttribute::Buzz],
        rarity: Rarity::DoubleRare,
        illustration_url: "/hocg-fan-sim-assets/img/hSD01/hSD01-006.webp".into(),
        artist: "I☆LA".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-006 - Tokino Sora (First)
    async fn hsd01_006_without_azki() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-006".into()),
            life: ["hY01-001".into()].into(),
            ..Default::default()
        };
        let p2 = p1.clone();

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Main)
            .with_player_1(p1)
            .with_attachments(
                Player::One,
                Zone::CenterStage,
                0,
                ["hY02-001".into(), "hY01-001".into(), "hY01-001".into()].into(),
            )
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // SorAZ Sympathy
            &[1],
            // done
            &[0],
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
        expected_state.active_step = Step::Performance;
        expected_state
            .card_modifiers
            .entry("c_0211".into())
            .or_default()
            .extend([Modifier {
                id: "m_0001".into(),
                kind: ModifierKind::PreventAllArts,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0212".into(), DamageMarkers::from_hp(60));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.state);
    }
    #[tokio::test]
    /// hSD01-006 - Tokino Sora (First)
    async fn hsd01_006_with_azki() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-006".into()),
            back_stage: ["hSD01-011".into()].into(),
            life: ["hY01-001".into()].into(),
            ..Default::default()
        };
        let p2 = p1.clone();

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Main)
            .with_player_1(p1)
            .with_attachments(
                Player::One,
                Zone::CenterStage,
                0,
                ["hY02-001".into(), "hY01-001".into(), "hY01-001".into()].into(),
            )
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // SorAZ Sympathy
            &[1],
            // done
            &[0],
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
        expected_state.active_step = Step::Performance;
        expected_state
            .card_modifiers
            .entry("c_0211".into())
            .or_default()
            .extend([Modifier {
                id: "m_0002".into(),
                kind: ModifierKind::PreventAllArts,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0212".into(), DamageMarkers::from_hp(110));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.state);
    }
}
