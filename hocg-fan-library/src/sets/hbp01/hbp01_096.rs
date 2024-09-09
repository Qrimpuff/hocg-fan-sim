use hocg_fan_sim::{
    card_effects::ParseEffect,
    cards::{
        Color::*, HoloMemberArtDamage::*, HoloMemberHashTag::*, HoloMemberLevel::*, Rarity::*, *,
    },
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hBP01-096".into(),
        name: "Usada Pekora".into(),
        colors: vec![Colorless],
        hp: 80,
        level: Spot,
        hash_tags: vec![JP, Gen3, AnimalEars],
        baton_pass_cost: 1,
        abilities: vec![HoloMemberAbility {
            kind: MemberAbilityKind::CollabEffect,
            name: r#"That Is "Adventure""#.into(),
            text: "Roll a six-sided die: If the result is even, you may search your deck for a Buzz holomem, reveal it, and put it into your hand. Then shuffle your deck.".into(),
            condition: vec![],
            effect: (r"
                let $roll = roll_dice
                if is_even $roll (
                    let $choice = select_one from main_deck (is_member and is_attribute_buzz)
                    reveal $choice
                    send_to hand $choice
                    shuffle main_deck
                )
            ")
            .parse_effect()
            .expect("hBP01-096"),
        }],
        arts: vec![HoloMemberArt {
            name: "Pekora ~Towards the Other Side of the Door~".into(),
            cost: vec![Colorless, Colorless],
            damage: Basic(10),
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
    async fn hbp01_096() {
        // let _guard = setup_test_logs();

        let p1 = TestGameBoard {
            oshi: Some("hSD01-001".into()),
            center_stage: Some("hSD01-006".into()),
            back_stage: ["hBP01-096".into()].into(),
            life: ["hY01-001".into()].into(),
            archive: ["hY01-001".into()].into(),
            main_deck: [
                "hSD01-004".into(),
                "hSD01-005".into(),
                "hSD01-006".into(),
                "hSD01-007".into(),
            ]
            .into(),
            ..Default::default()
        };
        let p2 = p1.clone();

        let state = GameStateBuilder::new()
            .with_active_player(Player::One)
            .with_active_step(Step::Cheer)
            .with_player_1(p1)
            .with_zone_modifiers(
                Player::One,
                Zone::All,
                ModifierKind::NextDiceRoll(2),
                LifeTime::UntilRemoved,
            )
            .with_player_2(p2)
            .build();

        let p1_p = BufferedPrompter::new(&[
            // That Is "Adventure"
            &[0],
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
        // assert_eq!(state, game.game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.main_deck = ["c_0311".into(), "c_0511".into()].into();
        expected_state.player_1.collab = Some("c_0711".into());
        expected_state.player_1.back_stage = [].into();
        expected_state.player_1.holo_power = ["c_0211".into()].into();
        expected_state.player_1.hand = ["c_0411".into()].into();
        expected_state.zone_modifiers.insert(
            Player::One,
            [(
                Zone::All,
                Modifier {
                    id: "m_0001".into(),
                    kind: ModifierKind::PreventCollab,
                    life_time: LifeTime::ThisTurn,
                },
            )]
            .into(),
        );

        assert_eq!(expected_state, game.game.state);
    }
}
