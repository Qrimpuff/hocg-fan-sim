use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{
        Color::*, HoloMemberArtDamage::*, HoloMemberHashTag::*,
        HoloMemberLevel::*, Rarity::*, *,
    },
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-080".into(),
        name: "Hoshimachi Suisei".into(),
        colors: vec![Blue],
        hp: 110,
        level: First,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![HoloMemberAbility {
            kind: MemberAbilityKind::CollabEffect,
            name: r#"Memories of a Snowy Mountain"#.into(),
            text: "Roll a six-sided die: If the result is odd, Knock Out 1 of your opponent's Back position holomem that has 40 or more damage marked on them (if that holomem is Knocked Out this way, your opponent does not lose Life).".into(),
            condition: vec![],
            effect: (r"
                let $roll = roll_dice
                if is_odd $roll (
                    let $back = select_one from opponent_back_stage (is_member and dmg_amount >= 40)
                    add_mod $back no_life_loss this_effect
                    knock_out $back
                )
            ")
            .parse_effect()
            .expect("hBP01-080"),
        }],
        arts: vec![HoloMemberArt {
            name: "Battle Maid".into(),
            cost: vec![Blue, Blue],
            damage: Basic(70),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![],
        rarity: Rare,
        illustration_url: "".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::*, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn hbp01_080_collab() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hBP01-079".into()),
            back_stage: ["hBP01-080".into(), "hBP01-080".into()].into(),
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
            .with_damage_markers(Player::Two, Zone::BackStage, 0, DamageMarkers::from_hp(40))
            .build();

        let p1_p = BufferedPrompter::new(&[
            // collab
            &[1],
            &[0],
            &[0],
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
        expected_state.player_1.collab = Some("c_0311".into());
        expected_state.player_1.back_stage = ["c_0411".into()].into();
        expected_state.player_1.holo_power = [].into();
        expected_state.player_1.archive =
            ["c_0a11".into(), "c_0911".into(), "c_0811".into()].into();
        expected_state.player_2.back_stage = ["c_0412".into()].into();
        expected_state.player_2.archive = ["c_0312".into()].into();
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
        expected_state
            .card_modifiers
            .entry("c_0111".into())
            .or_default()
            .extend([Modifier {
                id: "m_0003".into(),
                kind: ModifierKind::PreventOshiSkill(0),
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state.card_damage_markers.remove(&"c_0312".into());

        assert_eq!(expected_state, game.game.state);
    }
}
