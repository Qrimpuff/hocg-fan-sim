use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{
        Color::*, HoloMemberArtDamage::*, HoloMemberHashTag::*,
        HoloMemberLevel::*, Rarity::*, *,
    },
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-079".into(),
        name: "Hoshimachi Suisei".into(),
        colors: vec![Blue],
        hp: 120,
        level: First,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![HoloMemberAbility {
            kind: MemberAbilityKind::BloomEffect,
            name: "I'll Leave You Starstruck, So Don't Miss It!".into(),
            text: "Deal 20 special damage to one of your opponent's Back position holomem (if that holomem is Knocked Out this way, your opponent does not lose Life).".into(),
            condition: vec![],
            effect: (r"
                let $back = select_one from opponent_back_stage is_member
                add_mod $back no_life_loss this_effect
                deal_special_damage $back 20
            ")
            .parse_effect()
            .expect("hBP01-079"),
        }],
        arts: vec![HoloMemberArt {
            name: "Sui-chan Is... Cute As Always!!".into(),
            cost: vec![Colorless, Colorless],
            damage: Basic(50),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![],
        rarity: Uncommon,
        illustration_url: "".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::*, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn hbp01_079_bloom() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hBP01-079".into()),
            back_stage: ["hBP01-076".into()].into(),
            holo_power: ["hY02-001".into(), "hY02-001".into(), "hY02-001".into()].into(),
            hand: ["hBP01-079".into()].into(),
            life: ["hY01-001".into()].into(),
            cheer_deck: ["hY01-001".into(), "hY01-001".into()].into(),
            ..Default::default()
        };
        let mut p2 = p1.clone();
        p2.center_stage = Some("hSD01-006".into());

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Cheer)
            .with_player_1(p1)
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // bloom
            &[0],
            &[1],
            &[0],
            // done
            &[1],
        ]);
        let p2_p = BufferedPrompter::new(&[]);

        let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p).await;
        tokio::spawn(p1_client.receive_requests());
        tokio::spawn(p2_client.receive_requests());

        // main step
        let _ = game.next_step().await;

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.hand = [].into();
        expected_state.player_1.back_stage = ["c_0a11".into()].into();
        expected_state.player_1.attachments = [("c_0311".into(), "c_0a11".into())].into();
        expected_state
            .card_modifiers
            .entry("c_0a11".into())
            .or_default()
            .extend([Modifier {
                id: "m_0001".into(),
                kind: ModifierKind::PreventBloom,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_modifiers
            .entry("c_0312".into())
            .or_default();
        expected_state
            .card_damage_markers
            .insert("c_0312".into(), DamageMarkers::from_hp(20));

        assert_eq!(expected_state, game.game.state);
    }
}
