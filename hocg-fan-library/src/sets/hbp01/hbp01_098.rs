use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{
        Color::*, HoloMemberArtDamage::*, HoloMemberHashTag::*, HoloMemberLevel::*, Rarity::*, *,
    },
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-098".into(),
        name: "Shirogane Noel".into(),
        colors: vec![Colorless],
        hp: 90,
        level: Spot,
        hash_tags: vec![JP, Gen3, Alcohol],
        baton_pass_cost: 1,
        abilities: vec![HoloMemberAbility {
            kind: MemberAbilityKind::CollabEffect,
            name: r#"That Is "Me""#.into(),
            text: "Attach 1 Cheer card from your Archive to 1 of your holomem.".into(),
            condition: (r"
                any from stage is_member
            ")
            .parse_effect()
            .expect("hBP01-098"),
            effect: (r"
                let $cheer = select_one from archive is_cheer
                let $mem = select_one from stage is_member
                attach_cards $cheer $mem
            ")
            .parse_effect()
            .expect("hBP01-098"),
        }],
        arts: vec![HoloMemberArt {
            name: "Noel ~Towards the Other Side of the Door~".into(),
            cost: vec![Colorless, Colorless],
            damage: Basic(20),
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
    async fn hbp01_098() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-006".into()),
            back_stage: ["hBP01-098".into()].into(),
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
            // That Is "Me"
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
        game.next_step().await.unwrap();

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.collab = Some("c_0311".into());
        expected_state.player_1.back_stage = [].into();
        expected_state.player_1.archive = [].into();
        expected_state
            .player_1
            .attachments
            .extend([(CardRef::from("c_0511"), "c_0311".into())]);
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
