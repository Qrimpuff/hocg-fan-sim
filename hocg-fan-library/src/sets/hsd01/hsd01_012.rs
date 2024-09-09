use hocg_fan_sim::{
    card_effects::*,
    cards::{HoloMemberHashTag::*, *},
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-012".into(),
        name: "Airani Iofifteen".into(),
        colors: vec![Color::Green],
        hp: 70,
        level: HoloMemberLevel::Debut,
        hash_tags: vec![ID, IDGen1, Drawing],
        baton_pass_cost: 1,
        abilities: vec![HoloMemberAbility {
            kind: MemberAbilityKind::CollabEffect,
            name: "Let's Draw Together!".into(),
            text: "Attach one {W} Cheer or {G} Cheer from your Archive to your Center position holomem.".into(),
            condition: (r"
                all from center_stage is_member
            ").parse_effect().expect("hSD01-012"),
            effect: (r"
                let $cheer = select_one from archive is_cheer and (is_color_green or is_color_white)
                let $mem = filter from center_stage is_member
                attach_cards $cheer $mem
            ").parse_effect().expect("hSD01-012"),
        }],
        arts: vec![HoloMemberArt {
            name: "Drawing Is Fun!".into(),
            cost: vec![Color::Green],
            damage: HoloMemberArtDamage::Basic(20),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![],
        rarity: Rarity::Common,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-012.webp".into(),
        artist: "TODO".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-012 - Airani Iofifteen (Debut)
    async fn hsd01_012() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-006".into()),
            back_stage: ["hSD01-012".into()].into(),
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
            // Let's Draw Together!
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
        game.next_step().await.unwrap();

        // to check the changes, and apply them as checks below
        // assert_eq!(state, game.game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.collab = Some("c_0311".into());
        expected_state.player_1.back_stage = [].into();
        expected_state.player_1.archive = [].into();
        expected_state
            .player_1
            .attachments
            .extend([(CardRef::from("c_0511"), "c_0211".into())]);
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
