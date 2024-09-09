use hocg_fan_sim::{
    card_effects::*,
    cards::{HoloMemberHashTag::*, *},
};

pub fn card() -> Card {
    Card::HoloMember(HoloMemberCard {
        card_number: "hSD01-009".into(),
        name: "AZKi".into(),
        colors: vec![Color::Green],
        hp: 60,
        level: HoloMemberLevel::Debut,
        hash_tags: vec![JP, Gen0, Song],
        baton_pass_cost: 1,
        abilities: vec![HoloMemberAbility {
            kind: MemberAbilityKind::CollabEffect,
            name: "Expanding Map".into(),
            text: "Roll a six-sided die: If the result is 4 or less, attach the top card of your Cheer Deck to one of your Back position holomem. If the result is 1, you may also move this holomem to the Backstage.".into(),
            condition: vec![],
            effect: (r"
                let $roll = roll_dice
                if (($roll <= 4) and exist from back_stage) (
                    let $cheer = from_top 1 cheer_deck
                    reveal $cheer
                    let $back_mem = select_one from back_stage is_member
                    attach_cards $cheer $back_mem
                )
                if $roll == 1 (
                    let $option = optional_activate
                    if $option (
                        send_to back_stage this_card
                    )
                )
            ").parse_effect().expect("hSD01-009"),
        }],
        arts: vec![HoloMemberArt {
            name: "Where Next, Where Next?".into(),
            cost: vec![Color::Colorless],
            damage: HoloMemberArtDamage::Basic(10),
            special_damage: None,
            text: "".into(),
            condition: vec![],
            effect: vec![],
        }],
        attributes: vec![],
        rarity: Rarity::Rare,
        illustration_url: "https://qrimpuff.github.io/hocg-fan-sim-assets/img/hSD01/hSD01-009.webp".into(),
        artist: "".into(),
    })
}

#[cfg(test)]
mod tests {
    use hocg_fan_sim::{gameplay::*, modifiers::*, prompters::BufferedPrompter, tests::*};
    use pretty_assertions::assert_eq;

    #[tokio::test]
    /// hSD01-009 - AZKi (Debut)
    async fn hsd01_009() {
        let p1 = TestGameBoard {
            oshi: Some("hSD01-002".into()),
            center_stage: Some("hSD01-006".into()),
            back_stage: ["hSD01-009".into()].into(),
            life: ["hY01-001".into()].into(),
            holo_power: ["hSD01-005".into(), "hSD01-006".into(), "hSD01-007".into()].into(),
            cheer_deck: ["hY02-001".into()].into(),
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
            // Expanding Map
            &[0],
            &[0],
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
        // assert_eq!(state, game.state);

        let mut expected_state = state.clone();
        expected_state.active_step = Step::Main;
        expected_state.player_1.holo_power = [].into();
        expected_state.player_1.archive =
            ["c_0811".into(), "c_0711".into(), "c_0611".into()].into();
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
            .push(Modifier {
                id: "m_0003".into(),
                kind: ModifierKind::PreventOshiSkill(0),
                life_time: LifeTime::ThisTurn,
            });

        assert_eq!(expected_state, game.game.state);
    }
}
