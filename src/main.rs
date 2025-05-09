use dioxus::prelude::*;
use dx_rpg::{
    application,
    character_page::{self, AttackList},
    common::APP,
};
use lib_rpg::{attack_type::AttackType, effect::EffectOutcome, game_manager::ResultLaunchAttack};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
fn GameBoard() -> Element {
    let mut current_atk = use_signal(AttackType::default);
    let atk_menu_display = use_signal(|| false);
    let mut resultAttack = use_signal(ResultLaunchAttack::default);
    let mut autoResultAttack = use_signal(ResultLaunchAttack::default);

    rsx! {
        div { class: "grid-board",
            div {
                for c in APP.read().game_manager.pm.active_heroes.iter() {
                    character_page::CharacterPanel {
                        c: c.clone(),
                        current_player_name: APP.read().game_manager.pm.current_player.name.clone(),
                        is_auto_atk: false,
                        selected_atk: current_atk,
                        atk_menu_display,
                        result_auto_atk: resultAttack,
                        output_auto_atk: autoResultAttack,
                    }
                }
            }
            div {
                if atk_menu_display() {
                    AttackList {
                        name: APP.read().game_manager.pm.current_player.name.clone(),
                        display_atklist_sig: atk_menu_display,
                        selected_atk: current_atk,
                    }
                } else if !current_atk().name.is_empty() {
                    button {
                        onclick: move |_| async move {
                            resultAttack
                                .set(APP.write().game_manager.launch_attack(current_atk().name.as_str()));
                            current_atk.set(AttackType::default());
                        },
                        "launch atk"
                    }
                } else {
                    if !resultAttack().outcomes.is_empty() {
                        div { class: "show-then-hide",
                            ResultAtkText { ra: resultAttack }
                        }
                    }
                    if !autoResultAttack().outcomes.is_empty() {
                        div { class: "show-then-hide-auto",
                            ResultAtkText { ra: autoResultAttack }
                        }
                    }
                }
            }
            div {
                for c in APP.read().game_manager.pm.active_bosses.iter() {
                    character_page::CharacterPanel {
                        c: c.clone(),
                        current_player_name: "",
                        is_auto_atk: APP.read().game_manager.pm.current_player.name == c.name,
                        selected_atk: current_atk,
                        atk_menu_display,
                        result_auto_atk: resultAttack,
                        output_auto_atk: autoResultAttack,
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ButtonStatus {
    StartGame = 0,
    ValidateAction,
}

/// Home page
#[component]
fn Home() -> Element {
    let mut state = use_signal(|| ButtonStatus::StartGame);
    rsx! {
        if state() == ButtonStatus::StartGame {
            button {
                onclick: move |_| async move {
                    println!("component found");
                    match application::try_new().await {
                        Ok(app) => *APP.write() = app,
                        Err(_) => println!("no app"),
                    }
                    let _ = APP.write().game_manager.start_new_turn();
                    state.set(ButtonStatus::ValidateAction);
                },
                "Start"
            }
        }
        if state() == ButtonStatus::ValidateAction {
            button {
                onclick: move |_| async move {
                    APP.write().game_manager.launch_attack("SimpleAtk");
                },
                "Simple atk"
            }
        }
        GameBoard {}
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link { to: Route::Home {}, "Home" }
        }

        Outlet::<Route> {}
    }
}

#[component]
fn AmountText(eo: EffectOutcome) -> Element {
    let mut colortext = "green";
    if eo.real_amount_tx < 0 {
        colortext = "red";
    }
    rsx! {
        div { color: {colortext}, "{eo.target_name}: {eo.real_amount_tx}" }
    }
}

#[component]
fn ResultAtkText(ra: Signal<ResultLaunchAttack>) -> Element {
    rsx! {
        if !ra().outcomes.is_empty() {
            if ra().is_crit {
                "Critical Strike !"
            }
            for d in ra().all_dodging {
                if d.is_dodging {
                    "{d.name} is dodging"
                } else if d.is_blocking {
                    "{d.name} is blocking"
                }
            }
            for o in ra().outcomes {
                AmountText { eo: o }
            }
        }
    }
}
