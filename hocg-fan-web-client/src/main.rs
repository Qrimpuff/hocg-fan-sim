#![allow(non_snake_case)]

use std::iter;

use async_oneshot::oneshot;
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use futures::future;
use gloo_timers::future::TimeoutFuture;
use hocg_fan_sim::{
    cards::Card,
    client::{Client, DefaultEventHandler, EventHandler, IntentRequestHandler},
    events::{Event, IntentRequest, IntentResponse, Shuffle},
    gameplay::{
        CardDisplay, CardRef, Game, GameDirector, MainStepActionDisplay,
        PerformanceStepActionDisplay, Player, Zone,
    },
    library::{load_library, Loadout},
    modifiers::ModifierKind,
    prompters::PreferFirstPrompter,
};
use iter_tools::Itertools;

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
    pub fn relative_to(&self, mat_size: (i32, i32)) -> Mat {
        let rel = |pos: (i32, i32)| {
            (
                (pos.0 as f64 / self.mat_size.0 as f64 * mat_size.0 as f64) as i32,
                (pos.1 as f64 / self.mat_size.1 as f64 * mat_size.1 as f64) as i32,
            )
        };
        Mat {
            mat_size,
            card_size: rel(self.card_size),
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
static GAME: GlobalSignal<Game> = Signal::global(Game::default);
static EVENT: GlobalSignal<Option<Event>> = Signal::global(|| None);
static ANIM_LOCK: GlobalSignal<Option<async_oneshot::Sender<()>>> = Signal::global(|| None);
static ANIM_COUNT: GlobalSignal<u32> = Signal::global(|| 0);
static INTENT_REQUEST: GlobalSignal<Option<IntentRequest>> = Signal::global(|| None);
static INTENT_RESPONSE: GlobalSignal<Option<async_oneshot::Sender<IntentResponse>>> =
    Signal::global(|| None);

#[derive(Default)]
pub struct WebGameEventHandler {}
impl WebGameEventHandler {
    pub fn new() -> Self {
        Self {}
    }
}
impl EventHandler for WebGameEventHandler {
    async fn handle_event(&mut self, game: &Game, event: Event) {
        info!("it's in web");

        // TODO there might be a better way
        // load library on first entry
        if GAME.read().library.is_none() {
            *GAME.write() = Game::new().await;
        }

        GAME.write().state = game.state.clone();
        *EVENT.write() = Some(event.clone());
        *COUNT.write() += 1;

        if matches!(event, Event::Shuffle(_),) {
            let (s, r) = oneshot::<()>();
            *ANIM_LOCK.write() = Some(s);
            *ANIM_COUNT.write() = 0;
            // timeout animation after 5 seconds
            future::select(r, TimeoutFuture::new(5000)).await;
            *ANIM_LOCK.write() = None;
        }

        // yield
        TimeoutFuture::new(1).await;
    }
}

#[derive(Default)]
pub struct WebGameIntentHandler {}
impl WebGameIntentHandler {
    pub fn new() -> Self {
        Self {}
    }
}
impl IntentRequestHandler for WebGameIntentHandler {
    async fn handle_intent_request(&mut self, _game: &Game, req: IntentRequest) -> IntentResponse {
        let (s, r) = oneshot::<IntentResponse>();
        *INTENT_RESPONSE.write() = Some(s);
        *INTENT_REQUEST.write() = Some(req.clone());
        let resp = r.await.expect("should not be closed");
        *INTENT_RESPONSE.write() = None;
        *INTENT_REQUEST.write() = None;

        resp

        // match req {
        //     IntentRequest::Rps { player, select_rps } => {
        //         let rps = self.prompt_choice("choose rock, paper or scissor:", select_rps);
        //         IntentResponse::Rps {
        //             player,
        //             select_rps: rps,
        //         }
        //     }
        //     IntentRequest::Mulligan { player, .. } => {
        //         let mulligan =
        //             self.prompt_choice("do you want to mulligan?", vec!["Yes", "No"]) == "Yes";
        //         IntentResponse::Mulligan {
        //             player,
        //             select_yes_no: mulligan,
        //         }
        //     }
        //     IntentRequest::ActivateEffect { player, .. } => {
        //         let activate = self
        //             .prompt_choice("do you want to activate the effect?", vec!["Yes", "No"])
        //             == "Yes";
        //         IntentResponse::ActivateEffect {
        //             player,
        //             select_yes_no: activate,
        //         }
        //     }
        //     IntentRequest::LookSelectZoneToZone {
        //         player,
        //         select_cards,
        //         min_amount,
        //         max_amount,
        //         ..
        //     } => {
        //         let select_cards = select_cards
        //             .into_iter()
        //             .map(|c| CardDisplay::new(c, game))
        //             .collect_vec();
        //         let selected = self
        //             .prompt_multi_choices("choose cards:", select_cards, min_amount, max_amount)
        //             .into_iter()
        //             .map(|c| c.card)
        //             .collect();
        //         IntentResponse::LookSelectZoneToZone {
        //             player,
        //             select_cards: selected,
        //         }
        //     }
        //     IntentRequest::SelectToAttach {
        //         player,
        //         select_cards,
        //         ..
        //     } => {
        //         let select_cards = select_cards
        //             .into_iter()
        //             .map(|c| CardDisplay::new(c, game))
        //             .collect_vec();
        //         let selected = self.prompt_choice("choose card:", select_cards).card;
        //         IntentResponse::SelectToAttach {
        //             player,
        //             select_card: selected,
        //         }
        //     }
        //     IntentRequest::MainStepAction {
        //         player,
        //         select_actions,
        //     } => {
        //         let select_actions = select_actions
        //             .into_iter()
        //             .map(|a| MainStepActionDisplay::new(a, game))
        //             .collect_vec();
        //         let selected = self
        //             .prompt_choice("main step action:", select_actions)
        //             .action;
        //         IntentResponse::MainStepAction {
        //             player,
        //             select_action: selected,
        //         }
        //     }
        //     IntentRequest::SelectAttachments {
        //         player,
        //         select_attachments,
        //         min_amount,
        //         max_amount,
        //         ..
        //     } => {
        //         let select_attachments = select_attachments
        //             .into_iter()
        //             .map(|c| CardDisplay::new(c, game))
        //             .collect_vec();
        //         let selected = self
        //             .prompt_multi_choices(
        //                 "choose attachments:",
        //                 select_attachments,
        //                 min_amount,
        //                 max_amount,
        //             )
        //             .into_iter()
        //             .map(|c| c.card)
        //             .collect();
        //         IntentResponse::SelectAttachments {
        //             player,
        //             select_attachments: selected,
        //         }
        //     }
        //     IntentRequest::PerformanceStepAction {
        //         player,
        //         select_actions,
        //     } => {
        //         let select_actions = select_actions
        //             .into_iter()
        //             .map(|a| PerformanceStepActionDisplay::new(a, game))
        //             .collect_vec();
        //         let selected = self
        //             .prompt_choice("performance step action:", select_actions)
        //             .action;
        //         IntentResponse::PerformanceStepAction {
        //             player,
        //             select_action: selected,
        //         }
        //     }
        //     IntentRequest::SelectNumber {
        //         player,
        //         select_numbers,
        //     } => {
        //         let number = self.prompt_choice("choose a number:", select_numbers);
        //         IntentResponse::SelectNumber {
        //             player,
        //             select_number: number,
        //         }
        //     }
        // }
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

        // TODO will be replaced by file loading from GitHub
        load_library(&include_bytes!("../../hocg-fan-lib.gz")[..]).await;

        let mut game = GameDirector::setup(
            &player_1,
            &player_2,
            (p1_channel_1.0, p1_channel_2.1),
            (p2_channel_1.0, p2_channel_2.1),
        )
        .await;

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
    let _p1_c: Coroutine<()> = use_coroutine(|_rx| async move {
        let p1_client = Client::new(
            (p1_channel_2.0, p1_channel_1.1),
            WebGameEventHandler::new(),
            WebGameIntentHandler::new(),
        )
        .await;

        p1_client.receive_requests().await;
    });

    // Player 2
    let _p2_c: Coroutine<()> = use_coroutine(|_rx| async move {
        let p2_client = Client::new(
            (p2_channel_2.0, p2_channel_1.1),
            DefaultEventHandler::new(),
            PreferFirstPrompter::new(),
        )
        .await;

        p2_client.receive_requests().await;
    });

    /////////////////////////////////////////

    // mat size and position data
    let mat = use_context_provider(|| {
        Signal::new(Mat {
            mat_size: (2040, 1044),
            card_size: (220, 307), // easier to center the zones
            oshi_pos: (1334 + 236 / 2, 110 + 323 / 2),
            center_pos: (928 + 236 / 2, 110 + 323 / 2),
            collab_pos: (539 + 236 / 2, 110 + 323 / 2),
            back_line: (
                (928 + (236 / 2) - 23 * 3 - 236 * 2 - 30, 630 + 323 / 2),
                (928 + (236 / 2) + 23 * 3 + 236 * 2 - 30, 630 + 323 / 2),
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
        mat.relative_to((700, 358))
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
            class: "",
            onanimationstart: move |_event| {
                *ANIM_COUNT.write() += 1;
            },
            onanimationend: move |_event| {
                *ANIM_COUNT.write() -= 1;
                if *ANIM_COUNT.read() == 0 {
                    if let Some(lock) = ANIM_LOCK.write().as_mut() {
                        lock.send(()).unwrap();
                    }
                }
            },
            div {
                perspective: "1000px",
                width: "{rel_mat_size.0}px",
                class: "mx-auto",
                div {
                    transform: "rotateX(20deg) translateY(-48px)",
                    transform_style: "preserve-3d",
                    Board { mat: rel_mat, player: Player::Two }
                    Board { mat: rel_mat, player: Player::One }
                }
            }
        }
        div {
            Intent { player: Player::One }
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
    let support = game
        .board(player)
        .activate_support
        .iter()
        .copied()
        .map(|c| {
            rsx! {
                Card { key: "{c}", mat, card: c }
            }
        });

    rsx! {
        div {
            transform: if player == Player::Two { "rotateZ(180deg)" },
            transform_style: "preserve-3d",
            div {
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
                {life},
                // activate support
                {support}
            }
            Hand { mat, player }
        }
    }
}

#[component]
fn Card(mat: Signal<Mat>, card: CardRef, num: Option<(u32, u32)>) -> Element {
    let zone = use_memo(move || {
        GAME.read()
            .board_for_card(card)
            .find_card_zone(card)
            .unwrap()
    });
    let mut moving = use_signal(|| false);
    let rested = use_memo(move || GAME.read().has_modifier(card, ModifierKind::Resting));
    let mut flipped = use_signal(|| zone() == Zone::Life);
    let mut flipping = use_signal(|| false);

    let game = GAME.read();
    let card_lookup = game.lookup_card(card);
    // let card_number = card_lookup.card_number().to_owned();
    let illustration_url = card_lookup.illustration_url().to_owned();

    let card_size = mat().card_size;

    let pos = if zone() == Zone::Hand {
        (0, 0, 0)
    } else if zone() == Zone::ActivateSupport {
        let deck_pos = mat().zone_pos(Zone::MainDeck, None);
        (
            deck_pos.0 - card_size.0 * 2,
            deck_pos.1 - card_size.1 / 2,
            20,
        )
    } else {
        let pos = mat().zone_pos(zone(), num);
        (pos.0 - card_size.0 / 2, pos.1 - card_size.1 / 2, 0)
    };

    let z_index = if moving() { "200" } else { "100" };

    let rotate = if zone() == Zone::HoloPower || zone() == Zone::Life {
        "rotateZ(-90deg)"
    } else if rested() {
        "rotateZ(90deg)"
    } else {
        "rotateZ(0)"
    };

    let flipped_class = if flipped() { "card-flipped" } else { "" };
    let flipping_class = if flipping() { "card-flipping" } else { "" };

    let front_img = illustration_url;
    let back_img = match card_lookup {
        Card::OshiHoloMember(_) | Card::Cheer(_) => {
            "https://qrimpuff.github.io/hocg-fan-sim-assets/img/cheer-back.webp"
        }
        Card::HoloMember(_) | Card::Support(_) => {
            "https://qrimpuff.github.io/hocg-fan-sim-assets/img/card-back.webp"
        }
    };

    let attachments_count = game.attachments(card).count();
    let cheer_gap = 8;
    let cheers = game
        .attachments(card)
        .filter_map(|a| game.lookup_cheer(a))
        .enumerate()
        .map(|(i, att)| {
            let img = att.illustration_url.to_string();
            rsx! {
                div {
                    id: "{att.card_number}",
                    transform: "translate3d(0px, {i as i32 * cheer_gap + cheer_gap}px, 0px)",
                    z_index: "{attachments_count - i + 20}",
                    position: "absolute",
                    width: "{card_size.0}px",
                    height: "{card_size.1}px",
                    border_radius: "3.7%",
                    overflow: "hidden",
                    img {
                        width: "{card_size.0}px",
                        height: "{card_size.1}px",
                        border_radius: "3.7%",
                        src: "{img}"
                    }
                }
            }
        });
    let support_gap = -8;
    let supports = game
        .attachments(card)
        .filter_map(|a| game.lookup_support(a))
        .enumerate()
        .map(|(i, att)| {
            let img = att.illustration_url.to_string();
            rsx! {
                div {
                    id: "{att.card_number}",
                    transform: "translate3d(0px, {i as i32 * support_gap + support_gap}px, 0px)",
                    z_index: "{attachments_count - i + 20}",
                    position: "absolute",
                    width: "{card_size.0}px",
                    height: "{card_size.1}px",
                    border_radius: "3.7%",
                    overflow: "hidden",
                    img {
                        width: "{card_size.0}px",
                        height: "{card_size.1}px",
                        border_radius: "3.7%",
                        src: "{img}"
                    }
                }
            }
        });

    let dmg_markers = game.get_damage(card);
    let damage = if card_lookup.is_member() && dmg_markers.0 > 0 {
        let rem_hp = game.remaining_hp(card);
        let dmg_color = match game.remaining_hp(card) {
            ..=50 => ("#D62828", "white"),
            51..=100 => ("#FD9E02", "black"),
            _ => ("#FCBF49", "black"),
        };
        rsx! {
            div {
                transform: "translate3d(3px, 27px, 0px) rotateZ({(rem_hp as i32 / 10 % 6) * 8 - 20}deg)",
                z_index: "300",
                position: "absolute",
                background_color: "{dmg_color.0}",
                color: "{dmg_color.1}",
                border_color: "transparent",
                class: "badge badge-lg",
                "{dmg_markers.to_hp()}"
            }
        }
    } else {
        None
    };

    rsx! {
        div {
            id: "{card}",
            transform_style: "preserve-3d",
            transition: "transform 0.25s ease-in-out",
            ontransitionend: move |_event| moving.set(false),
            transform: "translate3d({pos.0}px, {pos.1}px, {pos.2}px)",
            width: "{card_size.0}px",
            height: "{card_size.1}px",
            z_index: "{z_index}",
            position: "absolute",
            onclick: move |_event| {},
            {cheers},
            {supports},
            div {
                transition: "transform 0.1s ease-in-out",
                transform_style: "preserve-3d",
                position: "absolute",
                width: "100%",
                height: "100%",
                z_index: "{z_index}",
                transform: "{rotate}",
                text_align: "left",
                {damage},
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
                        border_radius: "3.7%",
                        overflow: "hidden",
                        img {
                            width: "{card_size.0}px",
                            height: "{card_size.1}px",
                            border_radius: "3.7%",
                            src: "{front_img}"
                        }
                    }
                    div {
                        width: "100%",
                        height: "100%",
                        position: "absolute",
                        backface_visibility: "hidden",
                        transform: "rotateY(180deg)",
                        border_radius: "3.7%",
                        overflow: "hidden",
                        img {
                            width: "{card_size.0}px",
                            height: "{card_size.1}px",
                            border_radius: "3.7%",
                            src: "{back_img}"
                        }
                    }
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
            if let Some(Event::Shuffle(Shuffle { ref decks })) = *EVENT.read() {
                decks.contains(&(player, zone))
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

    let mut img = match zone {
        Zone::MainDeck | Zone::CenterStage | Zone::Collab | Zone::BackStage | Zone::HoloPower => {
            "https://qrimpuff.github.io/hocg-fan-sim-assets/img/card-back.webp".into()
        }
        Zone::Oshi | Zone::Life | Zone::CheerDeck => {
            "https://qrimpuff.github.io/hocg-fan-sim-assets/img/cheer-back.webp".into()
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
            let card = GAME
                .read()
                .board(player)
                .get_zone(zone)
                .peek_card(size() - 1 - i) // this is more stable with step by
                .unwrap();

            // archive is face-up
            if zone == Zone::Archive {
                img = GAME.read().lookup_card(card).illustration_url().to_string();
            }

            rsx! {
                div {
                    id: "{card}",
                    transform: "translate3d(0px, 0px, {i as f64 * px_per_cards}px)",
                    width: "{card_size.0}px",
                    height: "{card_size.1}px",
                    z_index: "2",
                    position: "absolute",
                    border_radius: "3.7%",
                    overflow: "hidden",
                    filter: "drop-shadow(0 1px 1px rgb(0 0 0 / 0.05))",
                    class: "deck-slice",
                    text_align: "center",
                    img {
                        position: "absolute",
                        width: "{card_size.0}px",
                        height: "{card_size.1}px",
                        border_radius: "3.7%",
                        src: "{img}"
                    }
                    if i + 1 == size() {
                        div { z_index: "10", position: "relative", "{size}" }
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

#[component]
fn Hand(mat: Signal<Mat>, player: Player) -> Element {
    let game = GAME.read();
    let card_size = mat().card_size;

    let cards = game.board(player).hand().map(|c| {
        rsx! {
            div {
                transform_style: "preserve-3d",
                width: "{card_size.0}px",
                height: "{card_size.1}px",
                class: "mx-0.5 mt-2",
                Card { key: "{c}", mat, card: c }
            }
        }
    });

    rsx! {
        div {
            transform_style: "preserve-3d",
            height: "{card_size.1 + card_size.1 / 2}px",
            class: "flex justify-center",
            {cards}
        }
    }
}

#[component]
fn Intent(player: Player) -> Element {
    let game = GAME.read();

    if let Some(ref req) = *INTENT_REQUEST.read() {
        let options = match req {
            IntentRequest::Rps { player, select_rps } => select_rps
                .iter()
                .map(|rps| {
                    let player = *player;
                    let select_rps = *rps;
                    rsx! {
                        button {
                            onclick: move |_event| {
                                INTENT_RESPONSE
                                    .write()
                                    .as_mut()
                                    .unwrap()
                                    .send(IntentResponse::Rps {
                                        player,
                                        select_rps,
                                    })
                                    .expect("should send correctly");
                            },
                            class: "btn btn-neutral",
                            "{select_rps}"
                        }
                    }
                })
                .collect_vec(),
            IntentRequest::Mulligan {
                player,
                select_yes_no,
            } => select_yes_no
                .iter()
                .map(|yes_no| {
                    let player = *player;
                    let select_yes_no = *yes_no;
                    rsx! {
                        button {
                            onclick: move |_event| {
                                INTENT_RESPONSE
                                    .write()
                                    .as_mut()
                                    .unwrap()
                                    .send(IntentResponse::Mulligan {
                                        player,
                                        select_yes_no,
                                    })
                                    .expect("should send correctly");
                            },
                            class: "btn btn-neutral",
                            if select_yes_no {
                                "Yes"
                            } else {
                                "No"
                            }
                        }
                    }
                })
                .collect_vec(),
            IntentRequest::ActivateEffect {
                player,
                select_yes_no,
            } => select_yes_no
                .iter()
                .map(|yes_no| {
                    let player = *player;
                    let select_yes_no = *yes_no;
                    rsx! {
                        button {
                            onclick: move |_event| {
                                INTENT_RESPONSE
                                    .write()
                                    .as_mut()
                                    .unwrap()
                                    .send(IntentResponse::ActivateEffect {
                                        player,
                                        select_yes_no,
                                    })
                                    .expect("should send correctly");
                            },
                            class: "btn btn-neutral",
                            if select_yes_no {
                                "Yes"
                            } else {
                                "No"
                            }
                        }
                    }
                })
                .collect_vec(),
            IntentRequest::LookSelectZoneToZone {
                player,
                from_zone,
                to_zone,
                look_cards,
                select_cards,
                min_amount,
                max_amount,
            } => select_cards
                .iter()
                .map(|card| {
                    let player = *player;
                    let select_card = *card;
                    let card_display = CardDisplay::new(*card, &game);
                    rsx! {
                        button {
                            onclick: move |_event| {
                                INTENT_RESPONSE
                                    .write()
                                    .as_mut()
                                    .unwrap()
                                    .send(IntentResponse::LookSelectZoneToZone {
                                        player,
                                        select_cards: vec![select_card],
                                    })
                                    .expect("should send correctly");
                            },
                            class: "btn btn-neutral",
                            "{card_display}"
                        }
                    }
                })
                .collect_vec(),
            IntentRequest::SelectToAttach {
                player,
                zones,
                select_cards,
            } => select_cards
                .iter()
                .map(|card| {
                    let player = *player;
                    let select_card = *card;
                    let card_display = CardDisplay::new(*card, &game);
                    rsx! {
                        button {
                            onclick: move |_event| {
                                INTENT_RESPONSE
                                    .write()
                                    .as_mut()
                                    .unwrap()
                                    .send(IntentResponse::SelectToAttach {
                                        player,
                                        select_card,
                                    })
                                    .expect("should send correctly");
                            },
                            class: "btn btn-neutral",
                            "{card_display}"
                        }
                    }
                })
                .collect_vec(),
            IntentRequest::MainStepAction {
                player,
                select_actions,
            } => select_actions
                .iter()
                .map(|action| {
                    let player = *player;
                    let select_action = *action;
                    let action_display = MainStepActionDisplay::new(*action, &game);
                    rsx! {
                        button {
                            onclick: move |_event| {
                                INTENT_RESPONSE
                                    .write()
                                    .as_mut()
                                    .unwrap()
                                    .send(IntentResponse::MainStepAction {
                                        player,
                                        select_action,
                                    })
                                    .expect("should send correctly");
                            },
                            class: "btn btn-neutral",
                            "{action_display}"
                        }
                    }
                })
                .collect_vec(),
            IntentRequest::SelectAttachments {
                player,
                card,
                select_attachments,
                min_amount,
                max_amount,
            } => select_attachments
                .iter()
                .map(|attachment| {
                    let player = *player;
                    let select_attachment = *attachment;
                    let attachment_display = CardDisplay::new(*attachment, &game);
                    rsx! {
                        button {
                            onclick: move |_event| {
                                INTENT_RESPONSE
                                    .write()
                                    .as_mut()
                                    .unwrap()
                                    .send(IntentResponse::SelectAttachments {
                                        player,
                                        select_attachments: vec![select_attachment],
                                    })
                                    .expect("should send correctly");
                            },
                            class: "btn btn-neutral",
                            "{attachment_display}"
                        }
                    }
                })
                .collect_vec(),
            IntentRequest::PerformanceStepAction {
                player,
                select_actions,
            } => select_actions
                .iter()
                .map(|action| {
                    let player = *player;
                    let select_action = *action;
                    let action_display = PerformanceStepActionDisplay::new(*action, &game);
                    rsx! {
                        button {
                            onclick: move |_event| {
                                INTENT_RESPONSE
                                    .write()
                                    .as_mut()
                                    .unwrap()
                                    .send(IntentResponse::PerformanceStepAction {
                                        player,
                                        select_action,
                                    })
                                    .expect("should send correctly");
                            },
                            class: "btn btn-neutral",
                            "{action_display}"
                        }
                    }
                })
                .collect_vec(),
            IntentRequest::SelectNumber {
                player,
                select_numbers,
            } => select_numbers
                .iter()
                .map(|number| {
                    let player = *player;
                    let select_number = *number;
                    rsx! {
                        button {
                            onclick: move |_event| {
                                INTENT_RESPONSE
                                    .write()
                                    .as_mut()
                                    .unwrap()
                                    .send(IntentResponse::SelectNumber {
                                        player,
                                        select_number,
                                    })
                                    .expect("should send correctly");
                            },
                            class: "btn btn-neutral",
                            "{select_number}"
                        }
                    }
                })
                .collect_vec(),
        };
        let buttons = options.into_iter().map(|o| {
            rsx! {
                div { {o} }
            }
        });
        rsx! {
            div { class: "flex flex-col", {buttons} }
        }
    } else {
        None
    }
}
