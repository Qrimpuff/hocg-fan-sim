#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use hocg_fan_sim::gameplay::Zone;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
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
}

#[component]
fn Home() -> Element {
    let mut count = use_signal(|| 0);

    // mat size and position data
    let mat = use_context_provider(|| {
        Signal::new(Mat {
            mat_size: (2040, 1044),
            card_size: (236, 323),
            oshi_pos: (1334 + 236 / 2, 109 + 323 / 2),
            center_pos: (927 + 236 / 2, 109 + 323 / 2),
            collab_pos: (539 + 236 / 2, 109 + 323 / 2),
            back_line: (
                (927 + (236 / 2) - 20 * 3 - 236 * 2, 675 + 323 / 2),
                (927 + (236 / 2) + 20 * 3 + 236 * 2, 675 + 323 / 2),
            ),
            life_line: ((49 + 323 / 2, 40 + 236 / 2), (49 + 323 / 2, 305 + 236 / 2)),
            cheer_deck_pos: (49 + 236 / 2, 675 + 323 / 2),
            main_deck_pos: (1753 + 236 / 2, 323 + 323 / 2),
            archive_pos: (1753 + 236 / 2, 675 + 323 / 2),
            holo_power_pos: (1753 + 236 - 323 / 2, 40 + 236 / 2),
        })
    });
    let rel_mat = use_signal(|| {
        let mat = mat.read();
        mat.relative_to((800, 409), (95, 130))
    });

    // relative size
    let rel_mat_size = rel_mat.read().mat_size;

    rsx! {
        Link { to: Route::Blog { id: count() }, "Go to blog" }
        div {
            h1 { "High-Five counter: {count}" }
            button { class: "btn", onclick: move |_| count += 1, "Up high!" }
            button { class: "btn", onclick: move |_| count -= 1, "Down low!" }
            button { class: "btn btn-lg", "Large" }
            button { class: "btn", "Normal" }
            button { class: "btn btn-sm", "Small" }
            button { class: "btn btn-xs", "Tiny" }
        }

        div { class: "relative text-center mx-auto",
            div {
                perspective: "1000px",
                width: "{rel_mat_size.0}px",
                class: "h-full",
                div {
                    transform: "rotateX(20deg) translateY(-48px)",
                    transform_style: "preserve-3d",
                    div {
                        // transform: "rotateZ(180deg)",
                        transform_style: "preserve-3d",
                        background_image: "url(https://hololive-official-cardgame.com/wp-content/uploads/2024/07/img_sec4_04.jpg)",
                        height: "{rel_mat_size.1}px",
                        class: "relative bg-cover bg-center",
                        // main stage
                        Card { mat: rel_mat(), zone: Zone::Oshi }
                        Card { mat: rel_mat(), zone: Zone::CenterStage }
                        Card { mat: rel_mat(), zone: Zone::Collab }
                        // cheer deck
                        Deck { mat: rel_mat(), zone: Zone::CheerDeck, size: 20 }
                        // archive
                        Deck { mat: rel_mat(), zone: Zone::Archive, size: 1 }
                        // -- main deck --
                        Deck { mat: rel_mat(), zone: Zone::MainDeck, size: 50 }
                        // holo power
                        Deck { mat: rel_mat(), zone: Zone::HoloPower, size: 5 }
                        // life
                        Card { mat: rel_mat(), zone: Zone::Life, num: (1, 6) }
                        Card { mat: rel_mat(), zone: Zone::Life, num: (2, 6) }
                        Card { mat: rel_mat(), zone: Zone::Life, num: (3, 6) }
                        Card { mat: rel_mat(), zone: Zone::Life, num: (4, 6) }
                        Card { mat: rel_mat(), zone: Zone::Life, num: (5, 6) }
                        Card { mat: rel_mat(), zone: Zone::Life, num: (6, 6) }
                        // back stage
                        Card { mat: rel_mat(), zone: Zone::BackStage, num: (1, 5) }
                        Card { mat: rel_mat(), zone: Zone::BackStage, num: (2, 5) }
                        Card { mat: rel_mat(), zone: Zone::BackStage, num: (3, 5) }
                        Card { mat: rel_mat(), zone: Zone::BackStage, num: (4, 5) }
                        Card { mat: rel_mat(), zone: Zone::BackStage, num: (5, 5) }
                    }
                    div {
                        // transform: "rotateZ(180deg)",
                        transform_style: "preserve-3d",
                        background_image: "url(https://hololive-official-cardgame.com/wp-content/uploads/2024/07/img_sec4_04.jpg)",
                        class: "relative lg:h-[409px] bg-cover bg-center ",
                        // main stage
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[38.4px] translate-x-[211.2px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[38.4px] translate-x-[364px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[38.4px] translate-x-[523.2px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // cheer deck
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[260px] translate-x-[19.2px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // top
                        div {
                            transform: "translate3d(19.2px, 260px, 20px)",
                            class: "absolute z-10 w-[92.8px] h-[128px] bg-gradient-to-tr from-red-500 to-purple-500"
                        }
                        // front
                        div {
                            transform: "translate3d(19.2px, 260px, 0px) rotateX(90deg)",
                            transform_origin: "top",
                            class: "absolute z-10 w-[92.8px] h-[20px] bg-gradient-to-tr from-green-500 to-yellow-500"
                        }
                        // back
                        div {
                            transform: "translate3d(19.2px, 388px, 0px) rotateX(90deg)",
                            transform_origin: "top",
                            class: "absolute z-10 w-[92.8px] h-[20px] bg-gradient-to-tr from-green-500 to-yellow-500"
                        }
                        // right side
                        div {
                            transform: "translate3d(92px, 260px, 0px) rotateY(90deg)",
                            transform_origin: "right",
                            class: "absolute z-10 w-[20px] h-[128px] bg-gradient-to-tr from-green-500 to-yellow-500"
                        }
                        // left side
                        div {
                            transform: "translate3d(19.2px, 260px, 0px) rotateY(-90deg)",
                            transform_origin: "left",
                            class: "absolute z-10 w-[20px] h-[128px] bg-gradient-to-tr from-green-500 to-yellow-500"
                        }
                        // archive
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[260px] translate-x-[688px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // -- main deck --
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[122.4px] translate-x-[688px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // top
                        div {
                            transform: "translate3d(688px, 122.4px, 50px)",
                            class: "absolute z-10 w-[92.8px] h-[128px] bg-gradient-to-tr from-red-500 to-purple-500"
                        }
                        // front
                        div {
                            transform: "translate3d(688px, 122.4px, 0px) rotateX(90deg)",
                            transform_origin: "top",
                            class: "absolute z-10 w-[92.8px] h-[50px] bg-gradient-to-tr from-green-500 to-yellow-500"
                        }
                        // back
                        div {
                            transform: "translate3d(688px, 250.4px, 0px) rotateX(90deg)",
                            transform_origin: "top",
                            class: "absolute z-10 w-[92.8px] h-[50px] bg-gradient-to-tr from-green-500 to-yellow-500"
                        }
                        // right side
                        div {
                            transform: "translate3d(730.8px, 122.4px, 0px) rotateY(90deg)",
                            transform_origin: "right",
                            class: "absolute z-10 w-[50px] h-[128px] bg-gradient-to-tr from-green-500 to-yellow-500"
                        }
                        // left side
                        div {
                            transform: "translate3d(688px, 122.4px, 0px) rotateY(-90deg)",
                            transform_origin: "left",
                            class: "absolute z-10 w-[50px] h-[128px] bg-gradient-to-tr from-green-500 to-yellow-500"
                        }
                        // --end of deck --
                        // holo power
                        div { class: "absolute z-10 w-[92.8px] h-[128px] -translate-y-[6.4px] translate-x-[670.4px] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // life
                        div { class: "absolute z-10 w-[92.8px] h-[128px] -translate-y-[6.4px] translate-x-[36.8px] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[14.4px] translate-x-[36.8px] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[35.2px] translate-x-[36.8px] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[56px] translate-x-[36.8px] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[76.8px] translate-x-[36.8px] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[97.6px] translate-x-[36.8px] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // back stage
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[260px] translate-x-[162.4px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[260px] translate-x-[263.2px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[260px] translate-x-[364px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[260px] translate-x-[464.8px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[92.8px] h-[128px] translate-y-[260px] translate-x-[565.6px] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                    }
                }
            }
        }
    }
}

#[component]
fn Card(mat: Mat, zone: Zone, num: Option<(u32, u32)>) -> Element {
    let card_size = mat.card_size;
    let pos = match zone {
        Zone::MainDeck => mat.main_deck_pos,
        Zone::Oshi => mat.oshi_pos,
        Zone::CenterStage => mat.center_pos,
        Zone::Collab => mat.collab_pos,
        Zone::BackStage => {
            if let Some(num) = num {
                mat.pos_on_line(mat.back_line, num.0, num.1)
            } else {
                mat.back_line.1
            }
        }
        Zone::Life => {
            if let Some(num) = num {
                mat.pos_on_line(mat.life_line, num.0, num.1)
            } else {
                mat.life_line.1
            }
        }
        Zone::CheerDeck => mat.cheer_deck_pos,
        Zone::HoloPower => mat.holo_power_pos,
        Zone::Archive => mat.archive_pos,
        _ => unimplemented!(),
    };
    let pos = (pos.0 - card_size.0 / 2, pos.1 - card_size.1 / 2);

    let rotate = match zone {
        Zone::Life | Zone::HoloPower => " rotateZ(-90deg)",
        _ => "",
    };

    rsx! {
        div {
            transform: "translate3d({pos.0}px, {pos.1}px, 0px) {rotate}",
            width: "{card_size.0}px",
            height: "{card_size.1}px",
            class: "absolute z-10 bg-gradient-to-tr from-cyan-500 to-blue-500",
            "{pos:?}"
        }
    }
}

#[component]
fn Deck(mat: Mat, zone: Zone, size: u32) -> Element {
    let card_size = mat.card_size;
    let pos = match zone {
        Zone::MainDeck => mat.main_deck_pos,
        Zone::Oshi => mat.oshi_pos,
        Zone::CenterStage => mat.center_pos,
        Zone::Collab => mat.collab_pos,
        Zone::BackStage => mat.back_line.0,
        Zone::Life => mat.life_line.0,
        Zone::CheerDeck => mat.cheer_deck_pos,
        Zone::HoloPower => mat.holo_power_pos,
        Zone::Archive => mat.archive_pos,
        _ => unimplemented!(),
    };
    let rotate = matches!(zone, Zone::Life | Zone::HoloPower);

    let pos = (pos.0 - card_size.0 / 2, pos.1 - card_size.1 / 2);
    let rotate = if rotate { " rotateZ(-90deg)" } else { "" };

    let size_px = size as i32;

    rsx! {
        div {
            transform_style: "preserve-3d",
            transform: "translate3d({pos.0}px, {pos.1}px, 0px) {rotate}",
            width: "{card_size.0}px",
            height: "{card_size.1}px",
            class: "absolute",
            // bottom
            div {
                transform: "translate3d(0px, 0px, 0px)",
                width: "{card_size.0}px",
                height: "{card_size.1}px",
                class: "absolute z-10 bg-gradient-to-tr from-cyan-500 to-blue-500",
                "{pos:?}"
            }
            // top
            div {
                transform: "translate3d(0px, 0px, {size_px}px)",
                width: "{card_size.0}px",
                height: "{card_size.1}px",
                class: "absolute z-10 bg-gradient-to-tr from-red-500 to-purple-500",
                "{pos:?}"
            }
            // front
            div {
                transform: "translate3d(0px, 0px, 0px) rotateX(90deg)",
                transform_origin: "top",
                width: "{card_size.0}px",
                height: "{size_px}px",
                class: "absolute z-10 bg-gradient-to-tr from-green-500 to-yellow-500"
            }
            // back
            div {
                transform: "translate3d(0px, {card_size.1}px, 0px) rotateX(90deg)",
                transform_origin: "top",
                width: "{card_size.0}px",
                height: "{size_px}px",
                class: "absolute z-10 bg-gradient-to-tr from-green-500 to-yellow-500"
            }
            // right side
            div {
                transform: "translate3d({card_size.0 - size_px}px, 0px, 0px) rotateY(90deg)",
                transform_origin: "right",
                width: "{size_px}px",
                height: "{card_size.1}px",
                class: "absolute z-10 bg-gradient-to-tr from-green-500 to-yellow-500"
            }
            // left side
            div {
                transform: "translate3d(0px, 0px, 0px) rotateY(-90deg)",
                transform_origin: "left",
                width: "{size_px}px",
                height: "{card_size.1}px",
                class: "absolute z-10 bg-gradient-to-tr from-green-500 to-yellow-500"
            }
        }
    }
}
