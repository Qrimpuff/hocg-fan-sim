use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{HoloMemberHashTag::*, *},
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-004".into(),
        name: "Tokino Sora".into(),
        colors: vec![Color::White],
        hp: 50,
        level: HoloMemberLevel::Debut,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![HoloMemberAbility {
            kind: MemberAbilityKind::CollabEffect,
            name: "Let's Dance!".into(),
            text: "Until end of turn, your Center position holomem gains +20 to their Arts.".into(),
            condition: vec![],
            effect: (r"
                add_zone_mod center_stage deal_more_dmg 20 this_turn
            ")
            .parse_effect()
            .expect("hSD01-004"),
        }],
        arts: vec![HoloMemberArt {
            name: "On Stage!".into(),
            cost: vec![Color::Colorless],
            damage: HoloMemberArtDamage::Basic(20),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![],
        rarity: Rarity::Rare,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-004_R.webp"
            .into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-004 - Tokino Sora (Debut)
    async fn hsd01_004() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-004".into()),
            back_stage: ["hSD01-004".into()].into(),
            life: ["hY01-001".into()].into(),
            ..Default::default()
        };
        let p2 = p1.clone();

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Cheer)
            .with_player_1(p1)
            .with_attachments(
                Player::One,
                Zone::CenterStage,
                0,
                ["hY01-001".into()].into(),
            )
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // Let's Dance!
            &[0],
            // done
            &[0],
            // performance step
            &[0],
            // done
            &[0],
        ]);
        let p2_p = BufferedPrompter::new(&[]);

        let (mut game, p1_client, p2_client) = setup_test_game(state.clone(), p1_p, p2_p).await;
        tokio::spawn(p1_client.receive_requests());
        tokio::spawn(p2_client.receive_requests());

        // main step
        game.next_step().await.unwrap();
        // performance step
        game.next_step().await.unwrap();

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Performance;
        expected_state.player_1.collab = Some("c_0311".into());
        expected_state.player_1.back_stage = [].into();
        expected_state.player_1.holo_power = [].into();
        expected_state
            .zone_modifiers
            .entry(Player::One)
            .or_default()
            .extend([
                (
                    Zone::All,
                    Modifier {
                        id: "m_0001".into(),
                        kind: ModifierKind::PreventCollab,
                        life_time: LifeTime::ThisTurn,
                    },
                ),
                (
                    Zone::CenterStage,
                    Modifier {
                        id: "m_0002".into(),
                        kind: ModifierKind::DealMoreDamage(20),
                        life_time: LifeTime::ThisTurn,
                    },
                ),
            ]);
        expected_state
            .card_modifiers
            .entry("c_0211".into())
            .or_default()
            .extend([Modifier {
                id: "m_0003".into(),
                kind: ModifierKind::PreventAllArts,
                life_time: LifeTime::ThisTurn,
            }]);
        expected_state
            .card_damage_markers
            .insert("c_0212".into(), DamageMarkers::from_hp(40));
        // expected_state.card_damage_markers

        assert_eq!(expected_state, game.game.state);
    }
}
