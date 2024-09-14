use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{
        Color::*, HoloMemberArtDamage::*, HoloMemberExtraAttribute::*, HoloMemberHashTag::*,
        HoloMemberLevel::*, Rarity::*, *,
    },
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-076".into(),
        name: "Hoshimachi Suisei".into(),
        colors: vec![Blue],
        hp: 90,
        level: Debut,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![],
        arts: vec![HoloMemberArt {
            name: "Diamond in the Rough".into(),
            cost: vec![Colorless],
            damage: Basic(20),
            special_damage: None,
            text: "Deal 10 special damage to one of your opponent's Back position holomem (if that holomem is Knocked Out this way, your opponent does not lose Life).".into(),
            condition: vec![],
            effect: (r"
                let $back = select_one from opponent_back_stage is_member
                add_mod $back no_life_loss this_effect
                deal_special_damage $back 10
            ")
            .parse_effect()
            .expect("hBP01-076"),
        }],
        attributes: vec![Unlimited],
        rarity: Common,
        illustration_url: "".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::*, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn hbp01_076() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hBP01-076".into()),
            back_stage: ["hSD01-004".into()].into(),
            holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            life: ["hY01-001".into()].into(),
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
                ["hY04-001".into(), "hY02-001".into()].into(),
            )
            .with_player_2(p2)
            .with_damage_markers(Player::Two, Zone::BackStage, 0, DamageMarkers::from_hp(40))
            .build();

        let p1_p = BufferedPrompter::new(&[
            // art
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
        expected_state.player_2.back_stage = [].into();
        expected_state.player_2.archive = ["c_0312".into()].into();
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
            .insert("c_0212".into(), DamageMarkers::from_hp(20));
        expected_state.card_damage_markers.remove(&"c_0312".into());
        expected_state.zone_modifiers.insert(Player::One, [].into());

        assert_eq!(expected_state, game.game.state);
    }
}
