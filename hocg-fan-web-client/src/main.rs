#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};

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

#[component]
fn Home() -> Element {
    let mut count = use_signal(|| 0);

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

        div { perspective: "1000px", class: "overflow-hidden",
            div { transform: "rotateX(20deg) translateY(-3rem)",
                div { class: "rotate-180 relative rounded-xl overflow-auto p-1",
                    div { class: "relative text-center rounded-lg overflow-hidden w-56 lg:w-[50rem] mx-auto",
                        // main stage
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[2.4rem] translate-x-[13.2rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[2.4rem] translate-x-[22.75rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[2.4rem] translate-x-[32.7rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // cheer deck
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[1.2rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // main deck
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[43rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // archive
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[7.65rem] translate-x-[43rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // holo power
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] -translate-y-[0.4rem] translate-x-[41.9rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // life
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] -translate-y-[0.4rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[0.9rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[2.2rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[3.5rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[4.8rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[6.1rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // back stage
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[10.15rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[16.45rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[22.75rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[29.05rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[35.35rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // mat
                        img {
                            src: "https://hololive-official-cardgame.com/wp-content/uploads/2024/07/img_sec4_04.jpg",
                            class: "object-cover lg:h-[25rem] w-full"
                        }
                    }
                }
                div { class: "relative rounded-xl overflow-auto p-1",
                    div { class: "relative text-center rounded-lg overflow-hidden w-56 lg:w-[50rem] mx-auto",
                        // main stage
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[2.4rem] translate-x-[13.2rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[2.4rem] translate-x-[22.75rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[2.4rem] translate-x-[32.7rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // cheer deck
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[1.2rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // main deck
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[43rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // archive
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[7.65rem] translate-x-[43rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // holo power
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] -translate-y-[0.4rem] translate-x-[41.9rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // life
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] -translate-y-[0.4rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[0.9rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[2.2rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[3.5rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[4.8rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[6.1rem] translate-x-[2.3rem] -rotate-90 bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // back stage
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[10.15rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[16.45rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[22.75rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[29.05rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        div { class: "absolute z-10 w-[5.8rem] h-[8rem] translate-y-[16.25rem] translate-x-[35.35rem] bg-gradient-to-tr from-cyan-500 to-blue-500" }
                        // mat
                        img {
                            src: "https://hololive-official-cardgame.com/wp-content/uploads/2024/07/img_sec4_04.jpg",
                            class: "object-cover lg:h-[25rem] w-full"
                        }
                    }
                }
            }
        }
    }
}
