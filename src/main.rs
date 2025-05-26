use dioxus::prelude::*;
use dx_rpg::{
    application,
    character_page::{self, AttackList},
    common::{PageStatus, APP, CURRENT_PAGE},
};
use lib_rpg::{
    attack_type::AttackType, effect::EffectOutcome, game_manager::ResultLaunchAttack,
    game_state::GameStatus,
};

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
        document::Stylesheet {
            // Urls are relative to your Cargo.toml file
            href: asset!("/assets/tailwind.css"),
        }
        Router::<Route> {}
    }
}

#[component]
fn GameBoard(game_status: Signal<ButtonStatus>) -> Element {
    let mut current_atk = use_signal(AttackType::default);
    let atk_menu_display = use_signal(|| false);
    let mut resultAttack = use_signal(ResultLaunchAttack::default);
    let autoResultAttack = use_signal(ResultLaunchAttack::default);

    use_effect(move || {
        if APP.read().game_manager.game_state.status == GameStatus::EndOfGame {
            game_status.set(ButtonStatus::ReplayGame);
        }
    });
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
    ReplayGame,
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        div { class: "home-container",
            if CURRENT_PAGE.cloned() == PageStatus::HomePage {
                h1 { "Welcome to the RPG game!" }
                button {
                    onclick: move |_| async move {
                        *CURRENT_PAGE.write() = PageStatus::NewGame;
                    },
                    "NEW GAME"
                }
                button {
                    onclick: move |_| async move {
                        *CURRENT_PAGE.write() = PageStatus::LoadGame;
                    },
                    "LOAD GAME"
                }
            }
        }
        if CURRENT_PAGE.cloned() == PageStatus::NewGame {
            NewGame {}
        }
    }
}

/// New game
#[component]
fn NewGame() -> Element {
    let mut state = use_signal(|| ButtonStatus::StartGame);
    let mut ready_to_start = use_signal(|| true);
    let _ = use_resource(move || async move {
        if state() == ButtonStatus::StartGame {
            match application::try_new().await {
                Ok(app) => *APP.write() = app,
                Err(_) => println!("no app"),
            }

            APP.write().game_manager.start_game();
            let path = APP.write().game_manager.game_paths.clone();
            match application::start_game(path).await {
                Ok(_) => println!("start app"),
                Err(_) => println!("no app"),
            }
            let _ = APP.write().game_manager.start_new_turn();
            ready_to_start.set(true);
        }
    });

    rsx! {
        h4 { "{\nAPP.read().game_manager.game_state.current_turn_nb}" }
        if state() == ButtonStatus::ReplayGame {
            button {
                onclick: move |_| async move {
                    state.set(ButtonStatus::StartGame);
                    ready_to_start.set(false);
                },
                "Replay game"
            }
        } else if state() == ButtonStatus::StartGame && ready_to_start() {
            button {
                onclick: move |_| async move {
                    APP.write().game_manager.launch_attack("SimpleAtk");
                },
                "Simple atk"
            }
            SaveButton {}
            GameBoard { game_status: state }
        } else if state() == ButtonStatus::StartGame && !ready_to_start() {
            h4 { "Loading..." }
        }
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
        div { color: colortext, "{eo.target_name}: {eo.real_amount_tx}" }
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

#[component]
fn SaveButton() -> Element {
    rsx! {
        button {
            onclick: move |_| async move {
                for c in &APP.read().game_manager.pm.active_heroes {
                    let path = format!(
                        "{}/{}.json",
                        &APP.read().game_manager.game_paths.characters.to_string_lossy(),
                        &c.name,
                    );
                    match application::save(
                            path.to_owned(),
                            serde_json::to_string_pretty(&c).unwrap(),
                        )
                        .await
                    {
                        Ok(()) => println!("save"),
                        Err(e) => println!("{}", e),
                    }
                }
                for b in &APP.read().game_manager.pm.active_bosses {
                    let path = format!(
                        "{}/{}.json",
                        &APP.read().game_manager.game_paths.characters.to_string_lossy(),
                        &b.name,
                    );
                    match application::save(
                            path.to_owned(),
                            serde_json::to_string_pretty(&b).unwrap(),
                        )
                        .await
                    {
                        Ok(()) => println!("save"),
                        Err(e) => println!("{}", e),
                    }
                }
                let path = format!(
                    "{}/{}.json",
                    &APP.read().game_manager.game_paths.game_state.to_string_lossy(),
                    "gamestate",
                );
                match application::save(
                        path.to_owned(),
                        serde_json::to_string_pretty(&APP.read().game_manager.game_state)
                            .unwrap(),
                    )
                    .await
                {
                    Ok(()) => println!("save"),
                    Err(e) => println!("{}", e),
                }
            },
            "Save"
        }
    }
}
