use hocg_fan_sim::{cards::Loadout, gameplay::Game, prompters::RandomPrompter, temp::test_library};

mod sets;

pub fn simple_loadout<const D: usize, const C: usize>(
    oshi: &str,
    deck: [(&str, usize); D],
    cheers: [(&str, usize); C],
) -> Loadout {
    let main_deck = Vec::from_iter(deck.into_iter().flat_map(|(c, a)| (0..a).map(|_| c.into())));
    let cheer_deck = Vec::from_iter(
        cheers
            .into_iter()
            .flat_map(|(c, a)| (0..a).map(|_| c.into())),
    );

    Loadout {
        oshi: oshi.into(),
        main_deck,
        cheer_deck,
    }
}

// pub fn setup_test_game(player_1: Loadout, player_2: Loadout) -> Game {
//     Game::setup(
//         test_library().clone(),
//         &player_1,
//         &player_2,
//         RandomPrompter::new(),
//     )
// }
