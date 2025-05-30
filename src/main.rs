use dioxus::prelude::*;
use dx_rpg::{
    application::{self, log_debug},
    character_page::{self, AttackList},
    common::APP,
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
    #[route("/create-server")]
    CreateServer {},
    #[route("/lobby-page")]
    LobbyPage {},
    #[route("/start-game")]
    StartGamePage {is_new_game: bool},
    #[route("/load-game")]
    LoadGame {},
    #[route("/ongoing-games")]
    OngoingGames {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    // Init logger
    println!("starting app");
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
            h1 { "Welcome to the RPG game!" }
            ButtonLink {
                target: Route::CreateServer {}.into(),
                name: "Create Server".to_string(),
            }
            ButtonLink {
                target: Route::OngoingGames {}.into(),
                name: "Servers available".to_string(),
            }
        }
    }
}

/// CreateServer page
#[component]
fn CreateServer() -> Element {
    rsx! {
        div { class: "home-container",
            ButtonLink {
                target: Route::LobbyPage {}.into(),
                name: "New Game".to_string(),
            }
            ButtonLink {
                target: Route::LoadGame {}.into(),
                name: "Load Game".to_string(),
            }
        }
    }
}

/// New game
#[component]
fn StartGamePage(is_new_game: bool) -> Element {
    let mut state = use_signal(|| ButtonStatus::StartGame);
    let mut ready_to_start = use_signal(|| true);
    let mut is_new_game_sig = use_signal(|| is_new_game);
    let _ = use_resource(move || async move {
        if state() == ButtonStatus::StartGame && is_new_game_sig() {
            APP.write().game_manager.start_new_game();
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

#[component]
fn LoadGame() -> Element {
    let mut active_button: Signal<i64> = use_signal(|| -1);
    let navigator = use_navigator();
    // get_game_list
    let games_list = use_resource(move || async move {
        match application::try_new().await {
            Ok(app) => *APP.write() = app,
            Err(_) => println!("no app"),
        }
        let path_dir = APP.write().game_manager.game_paths.clone();
        match application::get_game_list(path_dir.games_dir).await {
            Ok(games) => {
                for game in &games {
                    println!("Game: {}", game.to_string_lossy());
                }
                games
            }
            Err(e) => {
                println!("Error fetching game list: {}", e);
                vec![]
            }
        }
    });
    let games_list = games_list().unwrap_or_default();

    rsx! {
        div { class: "home-container",
            h4 { "LobbyPage" }
            for (index , i) in games_list.iter().enumerate() {
                button {
                    class: "button-lobby-list",
                    disabled: active_button() == index as i64,
                    onclick: move |_| async move { active_button.set(index as i64) },
                    "{i.clone().to_string_lossy()}"
                }
            }
            button {
                onclick: move |_| {
                    let cur_game = games_list.get(active_button() as usize).unwrap().to_owned();
                    async move {
                        let _ = log_debug(
                                format!("loading game: {}", cur_game.clone().to_string_lossy()),
                            )
                            .await;
                        let gm = match application::get_gamemanager_by_game_dir(cur_game.clone())
                            .await
                        {
                            Ok(gm) => gm,
                            Err(e) => {
                                println!("Error fetching game manager: {}", e);
                                return;
                            }
                        };
                        APP.write().game_manager = gm;
                        navigator.push(Route::StartGamePage {is_new_game: false});
                    }
                },
                "Start Game"
            }
        }
    }
}

#[component]
fn LobbyPage() -> Element {
    let _ = use_resource(move || async move {
        match application::try_new().await {
            Ok(app) => *APP.write() = app,
            Err(_) => println!("no app"),
        }
        
    });
    rsx! {
        div { class: "home-container",
            h4 { "LobbyPage" }
            ButtonLink {
                target: Route::StartGamePage {is_new_game: true}.into(),
                name: "Start Game".to_string(),
            }
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
            onclick: move |_| {
                let gm = APP.read().game_manager.clone();
                async move {
                    println!("Saving game state...");
                    let path = format!(
                        "{}",
                        &APP
                            .read()
                            .game_manager
                            .game_paths
                            .current_game_dir
                            .join("game_manager.json")
                            .to_string_lossy(),
                    );
                    match application::create_dir(
                            APP.read().game_manager.game_paths.current_game_dir.clone(),
                        )
                        .await
                    {
                        Ok(_) => println!("create game dir"),
                        Err(_) => println!("create game dir"),
                    }
                    match application::save(
                            path.to_owned(),
                            serde_json::to_string_pretty(&gm).unwrap(),
                        )
                        .await
                    {
                        Ok(()) => println!("save"),
                        Err(e) => println!("{}", e),
                    }
                }
            },
            "Save"
        }
    }
}

#[component]
fn OngoingGames() -> Element {
    rsx! {
        div { "ongoinggames" }
    }
}

#[component]
fn ButtonLink(target: NavigationTarget, name: String) -> Element {
    rsx! {
        div { class: "button-link",
            Link { class: "header-text", to: target, "{name}" }
        }
    }
}
