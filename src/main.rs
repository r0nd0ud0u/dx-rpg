use async_std::task::sleep;

use dioxus::prelude::*;
use dx_rpg::{
    application::{self, log_debug},
    character_page::{self, AttackList},
    common::{tempo_const::TIMER_FUTURE_1S, APP},
};
use lib_rpg::{
    attack_type::AttackType,
    effect::EffectOutcome,
    game_manager::ResultLaunchAttack,
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
    StartGamePage {},
    #[route("/load-game")]
    LoadGame {},
    #[route("/current-game")]
    JoinOngoingGame {},
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
    let mut write_game_manager = use_signal(|| false);
    let mut reload_app = use_signal(|| false);

    use_future(move || {
        async move {
            loop {
                // always sleep at start of loop
                sleep(std::time::Duration::from_millis(TIMER_FUTURE_1S)).await;
                if APP.write().game_manager.is_round_auto(){
                    sleep(std::time::Duration::from_millis(3000)).await;
                    APP.write().game_manager.game_state.last_result_atk = ResultLaunchAttack::default();
                    APP.write().game_manager.launch_attack("SimpleAtk");
                    log_debug(format!("launcher  {}", APP.write().game_manager.game_state.last_result_atk.launcher_name))
                                .await
                                .unwrap();
                    log_debug(format!("target  {}", APP.write().game_manager.game_state.last_result_atk.outcomes[0].target_name))
                        .await
                        .unwrap();
                    current_atk.set(AttackType::default());
                    write_game_manager.set(true);
                }
            }
        }
    });

    // Timer every second to update the game manager by reading json file
    use_future(move || {
        async move {
            loop {
                // always sleep at start of loop
                sleep(std::time::Duration::from_millis(TIMER_FUTURE_1S)).await;
                if !reload_app() {
                    reload_app.set(true);
                }
                if write_game_manager() {
                    write_game_manager.set(false);
                    // save the game manager state
                    let path = format!(
                        "{}",
                        &APP.write()
                            .game_manager
                            .game_paths
                            .current_game_dir
                            .join("game_manager.json")
                            .to_string_lossy(),
                    );
                    let new_dir = APP.write().game_manager.game_paths.current_game_dir.clone();
                    match application::create_dir(new_dir).await {
                        Ok(()) => println!("Directory created successfully"),
                        Err(e) => println!("Failed to create directory: {}", e),
                    }
                    let gm = APP.write().game_manager.clone();
                    match application::save(
                        path.to_owned(),
                        serde_json::to_string_pretty(&gm).unwrap(),
                    )
                    .await
                    {
                        Ok(()) => println!("save"),
                        Err(e) => println!("{}", e),
                    }
                } else if reload_app() {
                    // write the game manager to the app
                    reload_app.set(false);
                    let cur_game_dir = APP.write().game_manager.game_paths.current_game_dir.clone();
                    match application::get_gamemanager_by_game_dir(cur_game_dir.clone()).await {
                        Ok(gm) => {
                            APP.write().game_manager = gm
                        }
                        Err(e) => {
                            println!("Error fetching game manager: {}", e)
                        }
                    }
                }
            }
        }
    });

    // Check if the game is at the end of the game and set the game status to ReplayGame
    use_effect(move || {
        if APP.read().game_manager.game_state.status == GameStatus::EndOfGame {
            game_status.set(ButtonStatus::ReplayGame);
        }
    });

    // Display the game board with characters and attacks
    rsx! {
        div { class: "grid-board",
            div {
                // Heroes
                for c in APP.read().game_manager.pm.active_heroes.iter() {
                    character_page::CharacterPanel {
                        c: c.clone(),
                        current_player_name: APP.read().game_manager.pm.current_player.name.clone(),
                        selected_atk: current_atk,
                        atk_menu_display,
                        write_game_manager,
                        is_auto_atk: false
                    }
                }
            }
            div {
                "{APP.read().game_manager.game_state.current_turn_nb} "
                if atk_menu_display() {
                    AttackList {
                        name: APP.read().game_manager.pm.current_player.name.clone(),
                        display_atklist_sig: atk_menu_display,
                        selected_atk: current_atk,
                        write_game_manager,
                    }
                } else if !current_atk().name.is_empty() {
                    button {
                        onclick: move |_| async move {
                            APP.write().game_manager.launch_attack(current_atk().name.as_str());
                            current_atk.set(AttackType::default());
                            write_game_manager.set(true);
                        },
                        "launch atk"
                    }
                } else {
                    div {
                            ResultAtkText {
                                ra: APP.read().game_manager.game_state.last_result_atk.clone()
                            }
                        }
                }
            }
            div {
                // Bosses
                for c in APP.read().game_manager.pm.active_bosses.iter() {
                    character_page::CharacterPanel {
                        c: c.clone(),
                        current_player_name: "",
                        selected_atk: current_atk,
                        atk_menu_display,
                        write_game_manager,
                        is_auto_atk: APP.read().game_manager.pm.current_player.name == c.name
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
                target: Route::JoinOngoingGame {}.into(),
                name: "Join game".to_string(),
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
fn StartGamePage() -> Element {
    let mut state = use_signal(|| ButtonStatus::StartGame);
    let mut ready_to_start = use_signal(|| true);
    let _ = use_resource(move || async move {
        if state() == ButtonStatus::StartGame {
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
            h4 { "Load game" }
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
                        navigator.push(Route::LobbyPage {});
                    }
                },
                "Start Game"
            }
        }
    }
}

#[component]
fn LobbyPage() -> Element {
    let mut ready_to_start = use_signal(|| false);
    let _ = use_resource(move || async move {
        match application::try_new().await {
            Ok(app) => {
                // init application and game manager
                *APP.write() = app;
                // start a new game
                APP.write().game_manager.start_new_game();
                let _ = APP.write().game_manager.start_new_turn();
                // update ongoing games status
                let all_games_dir = format!(
                    "{}/ongoing-games.json",
                    APP.read()
                        .game_manager
                        .game_paths
                        .games_dir
                        .to_string_lossy()
                );
                let mut ongoing_games =
                    match application::read_ongoinggames_from_json(all_games_dir.clone()).await {
                        Ok(og) => og,
                        Err(e) => {
                            println!("Error reading ongoing games: {}", e);
                            application::OngoingGames::default()
                        }
                    };
                // at the moment only one game is ongoing
                ongoing_games.all_games.clear();
                // add the current game directory to ongoing games
                ongoing_games
                    .all_games
                    .push(APP.read().game_manager.game_paths.current_game_dir.clone());
                match application::save(
                    all_games_dir,
                    serde_json::to_string_pretty(&ongoing_games).unwrap(),
                )
                .await
                {
                    Ok(_) => println!("Game state saved successfully"),
                    Err(e) => println!("Failed to save game state: {}", e),
                }
                // save the game manager state
                let path = format!(
                    "{}",
                    &APP.read()
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
                    Ok(()) => println!("Directory created successfully"),
                    Err(e) => println!("Failed to create directory: {}", e),
                }
                match application::save(
                    path.to_owned(),
                    serde_json::to_string_pretty(&APP.read().game_manager.clone()).unwrap(),
                )
                .await
                {
                    Ok(()) => println!("save"),
                    Err(e) => println!("{}", e),
                }
                ready_to_start.set(true);
            }
            Err(_) => println!("no app"),
        }
    });
    rsx! {
        div { class: "home-container",
            h4 { "LobbyPage" }
            if ready_to_start() {
                ButtonLink {
                    target: Route::StartGamePage {}.into(),
                    name: "Start Game".to_string(),
                }
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
fn ResultAtkText(ra: ResultLaunchAttack) -> Element {
    rsx! {
        "Last round:"
        if !ra.outcomes.is_empty() {
            if ra.is_crit {
                "Critical Strike !"
            }
            for d in ra.all_dodging {
                if d.is_dodging {
                    "{d.name} is dodging"
                } else if d.is_blocking {
                    "{d.name} is blocking"
                }
            }
            for o in ra.outcomes {
                AmountText { eo: o }
            }
        } else {
            "No effects"
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
                        Ok(()) => println!("Directory created successfully"),
                        Err(e) => println!("Failed to create directory: {}", e),
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
fn JoinOngoingGame() -> Element {
    let _ = use_resource(move || async move {
        match application::try_new().await {
            Ok(app) => {
                *APP.write() = app;
                APP.write().game_manager.start_new_game();
                let _ = APP.write().game_manager.start_new_turn();
            }
            Err(_) => println!("no app"),
        }
    });
    let mut ongoing_games_sig = use_signal(application::OngoingGames::default);
    use_future(move || {
        async move {
            loop {
                // always sleep at start of loop
                sleep(std::time::Duration::from_millis(TIMER_FUTURE_1S)).await;
                // update ongoing games status
                let all_games_dir = format!(
                    "{}/ongoing-games.json",
                    APP.write()
                        .game_manager
                        .game_paths
                        .games_dir
                        .to_string_lossy()
                );
                let ongoing_games =
                    match application::read_ongoinggames_from_json(all_games_dir.clone()).await {
                        Ok(og) => {
                            application::log_debug(format!("ongoing games: {:?}", og.all_games))
                                .await
                                .unwrap();
                            if !og.all_games.is_empty() {
                                match application::get_gamemanager_by_game_dir(
                                    og.all_games[0].clone(),
                                )
                                .await
                                {
                                    Ok(gm) => APP.write().game_manager = gm,
                                    Err(e) => {
                                        println!("Error fetching game manager: {}", e)
                                    }
                                }
                                og
                            } else {
                                println!("No ongoing games found");
                                application::OngoingGames::default()
                            }
                        }
                        Err(e) => {
                            println!("Error reading ongoing games: {}", e);
                            application::log_debug("Error reading ongoing games".to_string())
                                .await
                                .unwrap();
                            application::OngoingGames::default()
                        }
                    };
                ongoing_games_sig.set(ongoing_games.clone());
            }
        }
    });
    if !ongoing_games_sig().all_games.is_empty() {
        rsx! {
            div { class: "ongoing-games-container",
                h4 { "Ongoing Games" }
                div { class: "ongoing-game-item",
                    Link { to: Route::StartGamePage {}, "Join Game" }
                    "{ongoing_games_sig().all_games[0].to_string_lossy()}"
                }
            }
        }
    } else {
        rsx! {
            div { "No loading ongoing games" }
        }
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
