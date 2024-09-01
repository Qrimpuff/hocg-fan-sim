#![allow(non_snake_case)]

use std::{iter, time::Duration};

use async_oneshot::oneshot;
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use gloo_timers::future::TimeoutFuture;
use hocg_fan_sim::{
    cards::Loadout,
    client::{Client, DefaultEventHandler, EventHandler},
    events::{Event, Shuffle},
    gameplay::{CardRef, Game, GameState, Player, Zone},
    modifiers::ModifierKind,
    prompters::RandomPrompter,
};

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

fn App() -> Element {
    rsx! {
        script { src: "https://cdn.tailwindcss.com" }
        Router::<Route> {}
    }
}

#[component]
fn Blog(id: i32) -> Element {
    rsx! {
        Link { to: Route::Home {}, "Go to counter" }
        "Blog post {id}"
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Mat {
    mat_size: (i32, i32),
    /// is separate from the mat dimensions
    card_size: (i32, i32),
    oshi_pos: (i32, i32),
    center_pos: (i32, i32),
    collab_pos: (i32, i32),
    back_line: ((i32, i32), (i32, i32)),
    life_line: ((i32, i32), (i32, i32)),
    cheer_deck_pos: (i32, i32),
    main_deck_pos: (i32, i32),
    archive_pos: (i32, i32),
    holo_power_pos: (i32, i32),
}

impl Mat {
    pub fn relative_to(&self, mat_size: (i32, i32), card_size: (i32, i32)) -> Mat {
        let rel = |pos: (i32, i32)| {
            (
                (pos.0 as f64 / self.mat_size.0 as f64 * mat_size.0 as f64) as i32,
                (pos.1 as f64 / self.mat_size.1 as f64 * mat_size.1 as f64) as i32,
            )
        };
        Mat {
            mat_size,
            card_size,
            oshi_pos: rel(self.oshi_pos),
            center_pos: rel(self.center_pos),
            collab_pos: rel(self.collab_pos),
            back_line: (rel(self.back_line.0), rel(self.back_line.1)),
            life_line: (rel(self.life_line.0), rel(self.life_line.1)),
            cheer_deck_pos: rel(self.cheer_deck_pos),
            main_deck_pos: rel(self.main_deck_pos),
            archive_pos: rel(self.archive_pos),
            holo_power_pos: rel(self.holo_power_pos),
        }
    }

    pub fn pos_on_line(&self, range: ((i32, i32), (i32, i32)), num: u32, total: u32) -> (i32, i32) {
        let pos_on_range = |start, end| {
            if total <= 1 {
                return start as f64;
            }
            start as f64 + (end as f64 - start as f64) / (total as f64 - 1.0) * (num as f64 - 1.0)
        };
        (
            pos_on_range(range.0 .0, range.1 .0) as i32,
            pos_on_range(range.0 .1, range.1 .1) as i32,
        )
    }

    pub fn zone_pos(&self, zone: Zone, num: Option<(u32, u32)>) -> (i32, i32) {
        match zone {
            Zone::MainDeck => self.main_deck_pos,
            Zone::Oshi => self.oshi_pos,
            Zone::CenterStage => self.center_pos,
            Zone::Collab => self.collab_pos,
            Zone::BackStage => {
                if let Some(num) = num {
                    self.pos_on_line(self.back_line, num.0, num.1)
                } else {
                    self.back_line.1
                }
            }
            Zone::Life => {
                if let Some(num) = num {
                    self.pos_on_line(self.life_line, num.0, num.1)
                } else {
                    self.life_line.1
                }
            }
            Zone::CheerDeck => self.cheer_deck_pos,
            Zone::HoloPower => self.holo_power_pos,
            Zone::Archive => self.archive_pos,
            _ => unimplemented!(),
        }
    }
}

static COUNT: GlobalSignal<i32> = Signal::global(|| 0);
static GAME: GlobalSignal<GameState> = Signal::global(GameState::new);
static EVENT: GlobalSignal<Option<Event>> = Signal::global(|| None);
static ANIM_LOCK: GlobalSignal<Option<async_oneshot::Sender<()>>> = Signal::global(|| None);
static ANIM_COUNT: GlobalSignal<u32> = Signal::global(|| 0);

#[derive(Default)]
pub struct WebGameEventHandler {}
impl WebGameEventHandler {
    pub fn new() -> Self {
        Self {}
    }
}
impl EventHandler for WebGameEventHandler {
    async fn handle_event(&mut self, game: &GameState, event: Event) {
        info!("it's in web");
        *GAME.write() = game.clone();
        *EVENT.write() = Some(event.clone());
        *COUNT.write() += 1;

        if matches!(event, Event::Shuffle(_),) {
            let (s, r) = oneshot::<()>();
            *ANIM_LOCK.write() = Some(s);
            r.await.unwrap();
        }

        // yield
        TimeoutFuture::new(1).await;
    }
}

#[component]
fn Home() -> Element {
    ////////////////////////////////

    // channels
    let p1_channel_1 = async_channel::unbounded();
    let p1_channel_2 = async_channel::unbounded();
    let p2_channel_1 = async_channel::unbounded();
    let p2_channel_2 = async_channel::unbounded();

    // Game
    let _game_c: Coroutine<()> = use_coroutine(|_rx| async move {
        let main_deck_hsd01 = Vec::from_iter(
            None.into_iter()
                .chain(iter::repeat("hSD01-003".into()).take(4))
                .chain(iter::repeat("hSD01-004".into()).take(3))
                .chain(iter::repeat("hSD01-005".into()).take(3))
                .chain(iter::repeat("hSD01-006".into()).take(2))
                .chain(iter::repeat("hSD01-007".into()).take(2))
                .chain(iter::repeat("hSD01-008".into()).take(4))
                .chain(iter::repeat("hSD01-009".into()).take(3))
                .chain(iter::repeat("hSD01-010".into()).take(3))
                .chain(iter::repeat("hSD01-011".into()).take(2))
                .chain(iter::repeat("hSD01-012".into()).take(2))
                .chain(iter::repeat("hSD01-013".into()).take(2))
                .chain(iter::repeat("hSD01-014".into()).take(2))
                .chain(iter::repeat("hSD01-015".into()).take(2))
                .chain(iter::repeat("hSD01-016".into()).take(3))
                .chain(iter::repeat("hSD01-017".into()).take(3))
                .chain(iter::repeat("hSD01-018".into()).take(3))
                .chain(iter::repeat("hSD01-019".into()).take(3))
                .chain(iter::repeat("hSD01-020".into()).take(2))
                .chain(iter::repeat("hSD01-021".into()).take(2)),
        );
        let cheer_deck_hsd01 = Vec::from_iter(
            None.into_iter()
                .chain(iter::repeat("hY01-001".into()).take(10))
                .chain(iter::repeat("hY02-001".into()).take(10)),
        );

        let player_1 = Loadout {
            oshi: "hSD01-001".into(), // Tokino Sora
            main_deck: main_deck_hsd01.clone(),
            cheer_deck: cheer_deck_hsd01.clone(),
        };
        let player_2 = Loadout {
            oshi: "hSD01-002".into(), // AZKi
            main_deck: main_deck_hsd01,
            cheer_deck: cheer_deck_hsd01,
        };

        let mut game = Game::setup(
            &player_1,
            &player_2,
            (p1_channel_1.0, p1_channel_2.1),
            (p2_channel_1.0, p2_channel_2.1),
        );

        // wait for the page to load
        TimeoutFuture::new(1000).await;

        info!("{:#?}", &game);
        game.start_game().await.unwrap();
        info!("{:#?}", &game);

        while game.next_step().await.is_ok() {
            TimeoutFuture::new(100).await;
        }
        info!("{:#?}", &game);
    });

    // Player 1
    let p1_client = Client::new(
        (p1_channel_2.0, p1_channel_1.1),
        WebGameEventHandler::new(),
        RandomPrompter::new(),
    );
    let _p1_c: Coroutine<()> = use_coroutine(|_rx| async move {
        p1_client.receive_requests().await;
    });

    // Player 2
    let p2_client = Client::new(
        (p2_channel_2.0, p2_channel_1.1),
        DefaultEventHandler::new(),
        RandomPrompter::new(),
    );
    let _p2_c: Coroutine<()> = use_coroutine(|_rx| async move {
        p2_client.receive_requests().await;
    });

    /////////////////////////////////////////

    // mat size and position data
    let mat = use_context_provider(|| {
        Signal::new(Mat {
            mat_size: (2040, 1044),
            card_size: (236, 323), // easier to center the zones
            oshi_pos: (1334 + 236 / 2, 110 + 323 / 2),
            center_pos: (928 + 236 / 2, 110 + 323 / 2),
            collab_pos: (539 + 236 / 2, 110 + 323 / 2),
            back_line: (
                (928 + (236 / 2) - 20 * 3 - 236 * 2 - 22, 630 + 323 / 2),
                (928 + (236 / 2) + 20 * 3 + 236 * 2 - 22, 630 + 323 / 2),
            ),
            life_line: ((49 + 323 / 2, 40 + 236 / 2), (49 + 323 / 2, 305 + 236 / 2)),
            cheer_deck_pos: (49 + 236 / 2, 677 + 323 / 2),
            main_deck_pos: (1755 + 236 / 2, 325 + 323 / 2),
            archive_pos: (1755 + 236 / 2, 677 + 323 / 2),
            holo_power_pos: (1755 + 236 - 323 / 2, 40 + 236 / 2),
        })
    });
    let rel_mat = use_signal(|| {
        let mat = mat.read();
        mat.relative_to((800, 409), (86, 120))
    });

    // relative size
    let rel_mat_size = rel_mat.read().mat_size;

    rsx! {
        Link { to: Route::Blog { id: COUNT() }, "Go to blog" }
        div {
            h1 { "Event counter: {COUNT}" }
            h1 { "Animation counter: {ANIM_COUNT}" }
        }

        div {
            class: "relative text-center mx-auto",
            onanimationstart: move |_event| {
                *ANIM_COUNT.write() += 1;
            },
            onanimationend: move |_event| {
                *ANIM_COUNT.write() -= 1;
                if *ANIM_COUNT.read() == 0 {
                    ANIM_LOCK.write().as_mut().unwrap().send(()).unwrap();
                }
            },
            div {
                perspective: "1000px",
                width: "{rel_mat_size.0}px",
                class: "h-full",
                div {
                    transform: "rotateX(20deg) translateY(-48px)",
                    transform_style: "preserve-3d",
                    Board { mat: rel_mat, player: Player::Two }
                    Board { mat: rel_mat, player: Player::One }
                }
            }
        }
    }
}

#[component]
fn Board(mat: Signal<Mat>, player: Player) -> Element {
    // relative size
    let rel_mat_size = mat().mat_size;

    // cards on stage
    let game = GAME.read();
    let oshi = game.board(player).oshi().map(|c| {
        rsx! {
            Card { key: "{c}", mat, card: c }
        }
    });
    let center_stage = game.board(player).center_stage().map(|c| {
        rsx! {
            Card { key: "{c}", mat, card: c }
        }
    });
    let collab = game.board(player).collab().map(|c| {
        rsx! {
            Card { key: "{c}", mat, card: c }
        }
    });
    let back_stage = game.board(player).back_stage().enumerate().map(|(i, c)| {
        rsx! {
            Card { key: "{c}", mat, card: c, num: (1 + i as u32, 5) }
        }
    });
    let life = game
        .board(player)
        .life
        .iter()
        .rev()
        .copied()
        .enumerate()
        .map(|(i, c)| {
            rsx! {
                    Card { key: "{c}", mat, card: c, num: (1 + i as u32, 6) }
            }
        });

    rsx! {
        div {
            transform: if player == Player::Two { "rotateZ(180deg)" },
            transform_style: "preserve-3d",
            background_image: "url(https://hololive-official-cardgame.com/wp-content/uploads/2024/07/img_sec4_04.jpg)",
            height: "{rel_mat_size.1}px",
            class: "relative bg-cover bg-center",
            // main stage
            {oshi},
            {center_stage},
            {collab},
            // back stage
            {back_stage},
            // cheer deck
            Deck { mat, player, zone: Zone::CheerDeck }
            // archive
            Deck { mat, player, zone: Zone::Archive }
            // -- main deck --
            Deck { mat, player, zone: Zone::MainDeck }
            // holo power
            Deck { mat, player, zone: Zone::HoloPower }
            // life
            {life}
        }
    }
}

#[component]
fn Card(mat: Signal<Mat>, card: CardRef, num: Option<(u32, u32)>) -> Element {
    let zone = use_memo(move || GAME().board_for_card(card).find_card_zone(card).unwrap());
    let mut moving = use_signal(|| false);
    let rested = use_memo(move || GAME().has_modifier(card, ModifierKind::Resting));
    let mut flipped = use_signal(|| zone() == Zone::Life);
    let mut flipping = use_signal(|| false);

    let game = GAME();
    let card_lookup = game.lookup_card(card);
    // let card_number = card_lookup.card_number().to_owned();
    let illustration_url = card_lookup.illustration_url().to_owned();

    let card_size = mat().card_size;

    let pos = mat().zone_pos(zone(), num);
    let pos = (pos.0 - card_size.0 / 2, pos.1 - card_size.1 / 2);

    let z_index = if moving() { "2" } else { "1" };

    let rotate = if zone() == Zone::HoloPower || zone() == Zone::Life {
        "rotateZ(-90deg)"
    } else if rested() {
        "rotateZ(90deg)"
    } else {
        "rotateZ(0)"
    };

    let flipped_class = if flipped() { "card-flipped" } else { "" };
    let flipping_class = if flipping() { "card-flipping" } else { "" };

    // TODO use our own images
    let front_img = illustration_url;
    let back_img = match zone() {
        Zone::MainDeck | Zone::CenterStage | Zone::Collab | Zone::BackStage | Zone::HoloPower => {
            "https://github.com/GabeJWJ/holoDelta/blob/master/fuda_holoBack.png?raw=true"
        }
        Zone::Oshi | Zone::Life | Zone::CheerDeck => {
            "https://github.com/GabeJWJ/holoDelta/blob/master/cheerBack.png?raw=true"
        }
        Zone::Archive => "https://github.com/GabeJWJ/holoDelta/blob/e2d323fffaede48e0f153fc46a2ab579ef0af0a6/hBP01-041.png?raw=true",
        _ => unimplemented!(),
    };

    rsx! {
        div {
            id: "{card}",
            transform_style: "preserve-3d",
            transition: "transform 0.25s ease-in-out",
            ontransitionend: move |_event| moving.set(false),
            transform: "translate3d({pos.0}px, {pos.1}px, 0px) {rotate}",
            width: "{card_size.0}px",
            height: "{card_size.1}px",
            z_index: "{z_index}",
            position: "absolute",
            onclick: move |_event| {},
            div {
                transform_style: "preserve-3d",
                position: "absolute",
                width: "100%",
                height: "100%",
                class: "{flipped_class} {flipping_class}",
                onanimationend: move |_event| {
                    flipped.set(!flipped());
                    flipping.set(false);
                    moving.set(false);
                },
                div {
                    width: "100%",
                    height: "100%",
                    position: "absolute",
                    backface_visibility: "hidden",
                    class: "bg-cover bg-center",
                    background_image: "url({front_img})",
                    border_radius: "3.7%",
                    "{card}"
                }
                div {
                    width: "100%",
                    height: "100%",
                    position: "absolute",
                    backface_visibility: "hidden",
                    transform: "rotateY(180deg)",
                    class: "bg-cover bg-center",
                    background_image: "url({back_img})",
                    border_radius: "3.7%",
                    "{card}"
                }
            }
        }
    }
}

#[component]
fn Deck(mat: Signal<Mat>, player: Player, zone: Zone) -> Element {
    let size = use_memo(move || GAME.read().board(player).get_zone(zone).count());

    let mut shuffling = use_signal(|| false);
    let shuffling_c = use_memo(move || if shuffling() { "deck-shuffling" } else { "" });
    use_effect(move || {
        shuffling.set(
            if let Some(Event::Shuffle(Shuffle { player: p, zone: z })) = *EVENT.read() {
                p == player && z == zone
            } else {
                false
            },
        )
    });

    let card_size = mat().card_size;

    let pos = mat().zone_pos(zone, None);
    let pos = (pos.0 - card_size.0 / 2, pos.1 - card_size.1 / 2);

    let rotate = if zone == Zone::HoloPower || zone == Zone::Life {
        "rotateZ(-90deg)"
    } else {
        "rotateZ(0)"
    };

    // TODO use our own images
    let mut img = match zone {
        Zone::MainDeck | Zone::CenterStage | Zone::Collab | Zone::BackStage | Zone::HoloPower => {
            "https://github.com/GabeJWJ/holoDelta/blob/master/fuda_holoBack.png?raw=true".into()
        }
        Zone::Oshi | Zone::Life | Zone::CheerDeck => {
            "https://github.com/GabeJWJ/holoDelta/blob/master/cheerBack.png?raw=true".into()
        }
        Zone::Archive => "".into(),
        _ => unimplemented!(),
    };

    // that step by makes the deck look cleaner
    let top_cards = 1;
    let step_cards = 3;
    let px_per_cards = 1.0;
    let cards = (0..size().saturating_sub(top_cards))
        .step_by(step_cards)
        .chain(size().saturating_sub(top_cards)..size())
        .map(|i| {
            // archive is face-up
            if zone == Zone::Archive {
                let card = GAME
                    .read()
                    .board(player)
                    .get_zone(zone)
                    .peek_card(size() - 1 - i) // this is more stable with step by
                    .unwrap();
                img = GAME.read().lookup_card(card).illustration_url().to_string();
            }

            rsx! {
                div {
                    transform: "translate3d(0px, 0px, {i as f64 * px_per_cards}px)",
                    width: "{card_size.0}px",
                    height: "{card_size.1}px",
                    z_index: "2",
                    position: "absolute",
                    border_radius: "5%",
                    filter: "drop-shadow(0 1px 1px rgb(0 0 0 / 0.05))",
                    class: "deck-slice bg-cover bg-center",
                    background_image: "url({img})",
                    if i + 1 == size() {
                        "{size}"
                    }
                }
            }
        });

    rsx! {
        div {
            transform_style: "preserve-3d",
            transform: "translate3d({pos.0}px, {pos.1}px, 0px) {rotate}",
            width: "{card_size.0}px",
            height: "{card_size.1}px",
            position: "absolute",
            class: "{shuffling_c}",
            {cards}
        }
    }
}
