use hocg_fan_sim::{
    card_effects::*,
    cards::{HoloMemberHashTag::*, *},
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-007".into(),
        name: "IRyS".into(),
        colors: vec![Color::White],
        hp: 50,
        level: HoloMemberLevel::Debut,
        hash_tags: vec![EN, Promise, Song],
        baton_pass_cost: 1,
        abilities: vec![HoloMemberAbility {
            kind: MemberAbilityKind::CollabEffect,
            name: "HOPE".into(),
            text: "Look at your holoPOWER. You may reveal a card from among your holoPOWER and put it into your hand. Then put 1 card from your hand onto your holoPOWER.".into(),
            condition: (r"
                exist from holo_power
            ").parse_effect().expect("hSD01-007"),
            effect: (r"
                let $choice = select_one from holo_power anything
                reveal $choice
                send_to hand $choice
                let $hand = select_one from hand anything
                send_to holo_power $hand
            ").parse_effect().expect("hSD01-007"),
        }],
        arts: vec![HoloMemberArt {
            name: "Avatar of Hope".into(),
            cost: vec![Color::White],
            damage: HoloMemberArtDamage::Basic(20),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![],
        rarity: Rarity::Common,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-007.webp".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-007 - IRyS (Debut)
    async fn hsd01_007() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-006".into()),
            back_stage: ["hSD01-007".into()].into(),
            life: ["hY01-001".into()].into(),
            holo_power: ["hSD01-005".into(), "hSD01-006".into(), "hSD01-007".into()].into(),
            hand: ["hSD01-008".into(), "hSD01-009".into(), "hSD01-010".into()].into(),
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
            // HOPE
            &[2],
            &[1],
            &[2],
            // done
            &[4],
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
        expected_state.player_1.collab = Some("c_0311".into());
        expected_state.player_1.back_stage = [].into();
        expected_state.player_1.holo_power =
            ["c_0a11".into(), "c_0511".into(), "c_0711".into()].into();
        expected_state.player_1.hand = ["c_0811".into(), "c_0911".into(), "c_0611".into()].into();
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
