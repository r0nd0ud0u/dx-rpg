use dioxus::{
    fullstack::{WebSocketOptions, use_websocket},
    logger::tracing::{self, Level},
    prelude::*,
};
use dioxus_i18n::prelude::*;
use dioxus_sdk_storage::{LocalStorage, use_synced_storage};
use dotenv::dotenv;
use dx_rpg::{
    common::{
        CtxAppLang, CtxAutoSaveScenario, CtxShopEnabled, CtxShowAtkTooltips, CtxShowBossEnergy,
        CtxShowBossHp, CtxShowHeroAggro, CtxToggleAtkAnimation, DISCONNECTED_USER, DX_COMP_CSS,
        Route, SERVER_NAME,
    },
    websocket_handler::{
        NO_CLIENT_ID,
        event::{ClientEvent, ServerEvent, on_rcv_client_event},
    },
};
use lib_rpg::server::server_manager::{GamePhase, ServerData};
use unic_langid::langid;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    // Reads the .env file
    #[cfg(feature = "server")]
    dotenv().ok();

    // Init logger
    let _ = dioxus::logger::init(
        std::env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_owned())
            .parse::<Level>()
            .unwrap_or(Level::INFO),
    );
    tracing::info!("Rendering app!");

    // On the client, we simply launch the app as normal, taking over the main thread
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    // On the server, we can use `dioxus::serve` to create a server that serves our app.
    //
    // The `serve` function takes a closure that returns a `Future` which resolves to an `axum::Router`.
    //
    // We return a `Router` such that dioxus sets up logging, hot-reloading, devtools, and wires up the
    // IP and PORT environment variables to our server.
    #[cfg(feature = "server")]
    dioxus::serve(|| async {
        use axum_session::{SessionConfig, SessionLayer, SessionStore};
        use axum_session_auth::AuthConfig;
        use axum_session_sqlx::SessionSqlitePool;
        use dx_rpg::{
            auth_manager::{auth::AuthLayer, db::get_db, server_fn::update_all_connection_status},
            websocket_handler::STARTING_CLIENT_ID,
        };

        // on server start, set all users to disconnected in db to avoid stale connection status
        update_all_connection_status(false).await.unwrap();
        // create db pool for session store
        let pool = get_db().await;

        // initialize data manager
        init_data_manager().await;

        // Create an axum router that dioxus will attach the app to
        Ok(dioxus::server::router(App)
            .route("/img-srv/{filename}", axum::routing::get(serve_img_handler))
            .layer(AuthLayer::new(Some(pool.clone())).with_config(
                AuthConfig::<i64>::default().with_anonymous_user_id(Some(STARTING_CLIENT_ID)),
            ))
            .layer(SessionLayer::new(
                SessionStore::<SessionSqlitePool>::new(
                    Some(pool.clone().into()),
                    SessionConfig::default().with_table_name("test_table"),
                )
                .await?,
            )))
    });
}

/// Resolves the filesystem path for a static image bundled from `assets/img/`.
///
/// Dioxus content-hashes the folder at bundle time:
///   dev  (dx serve, CWD = project root): `assets/img/<file>`
///   prod (Docker, CWD = bundle output):  `public/assets/img-<hash>/<file>`
///
/// The bundled directory is discovered once and cached for the process lifetime.
#[cfg(feature = "server")]
fn resolve_static_img(filename: &str) -> std::path::PathBuf {
    use std::sync::OnceLock;
    static BUNDLED_IMG_DIR: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();

    // dev: assets/img/ exists relative to the project root (dx serve CWD)
    let dev = std::path::Path::new("assets/img").join(filename);
    if dev.exists() {
        return dev;
    }

    // bundled: Dioxus emits public/assets/img-<hash>/ — discover it once
    let bundled_dir = BUNDLED_IMG_DIR.get_or_init(|| {
        std::fs::read_dir("public/assets")
            .ok()?
            .flatten()
            .find(|e| {
                e.file_name().to_string_lossy().starts_with("img-")
                    && e.file_type().map(|t| t.is_dir()).unwrap_or(false)
            })
            .map(|e| e.path())
    });

    if let Some(dir) = bundled_dir {
        let candidate = dir.join(filename);
        if candidate.exists() {
            return candidate;
        }
    }

    dev // not found — return dev path so the caller gets a clear 404
}

#[cfg(feature = "server")]
pub async fn serve_img_handler(
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> axum::response::Response {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;
    // Security: reject any path traversal attempt
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
            vec![],
        )
            .into_response();
    }
    let photos_dir = std::env::var("PHOTOS_PATH").unwrap_or_else(|_| "photos".to_owned());
    let path = std::path::Path::new(&photos_dir).join(&filename);
    // Prefer user-uploaded photo; fall back to static bundled image
    let path = if path.exists() {
        path
    } else {
        resolve_static_img(&filename)
    };
    match std::fs::read(&path) {
        Ok(bytes) => {
            let mime = match path.extension().and_then(|e| e.to_str()) {
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("webp") => "image/webp",
                Some("gif") => "image/gif",
                _ => "application/octet-stream",
            };
            (StatusCode::OK, [(header::CONTENT_TYPE, mime)], bytes).into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            vec![],
        )
            .into_response(),
    }
}

#[cfg(feature = "server")]
pub async fn init_data_manager() {
    use dx_rpg::common::{DATA_MANAGER, OFFLINE_PATH};
    use lib_rpg::server::data_manager::DataManager;
    let mut dm = DATA_MANAGER.lock().unwrap();
    *dm = DataManager::try_new(OFFLINE_PATH).unwrap();
    tracing::info!(
        "Data manager initialized with {} equipments and {} heroes",
        dm.equipment_table.len(),
        dm.all_heroes.len()
    );
    drop(dm);
}

#[component]
fn App() -> Element {
    // i18n — English default, French available; toggle lives in the Navbar.
    use_init_i18n(|| {
        I18nConfig::new(langid!("en-US"))
            .with_locale((langid!("en-US"), include_str!("./i18n/en-US.ftl")))
            .with_locale((langid!("fr-FR"), include_str!("./i18n/fr-FR.ftl")))
    });

    // Local UI state
    let mut message = use_signal(String::new);
    let mut player_client_id = use_signal(|| 0);
    let mut server_data = use_signal(ServerData::default);
    let mut ongoing_games = use_signal(Vec::new);
    let mut saved_game_list = use_signal(Vec::new);
    let mut all_characters_names = use_signal(Vec::new);
    let mut toggle_atk_animation = use_signal(|| false);
    // Set to Some(map_id) by the lightweight OverworldEntered event.
    let mut overworld_map_id: Signal<Option<String>> = use_signal(|| None);

    let socket = use_websocket(|| on_rcv_client_event(WebSocketOptions::new()));

    // synced storage
    let mut login_name_session_local_sync =
        use_synced_storage::<LocalStorage, String>("synced_user_sql_name".to_owned(), || {
            DISCONNECTED_USER.clone()
        });
    let mut login_id_session_local_sync =
        use_synced_storage::<LocalStorage, i64>("synced_user_sql_id".to_owned(), || NO_CLIENT_ID); // from db, integer primary key not null and from 1 upwards
    let app_lang_local_sync =
        use_synced_storage::<LocalStorage, String>("synced_app_lang".to_owned(), || {
            "en".to_owned()
        });

    // Keep dioxus-i18n's active locale synced to the persisted "en"/"fr" value —
    // covers both the initial load from localStorage and every toggle click.
    use_effect(move || {
        let mut i18n = i18n();
        let target = if app_lang_local_sync() == "fr" {
            langid!("fr-FR")
        } else {
            langid!("en-US")
        };
        i18n.set_language(target);
    });

    // Set the theme to dark on app load
    use_effect(|| {
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Some(html) = document.document_element()
        {
            html.set_attribute("data-theme", "dark").ok();
        }
    });
    // Receive events from the websocket and update local signals.
    use_future(move || {
        let mut socket = socket;
        async move {
            tracing::info!("[client] ws-loop starting");
            while let Ok(event) = socket.recv().await {
                tracing::debug!("[client] ws-loop: received an event");
                match event {
                    ServerEvent::NewClientOnExistingPlayer(msg, client_id) => {
                        message.set(msg);
                        let login_name_session_local_sync = login_name_session_local_sync();
                        let login_id_session_local_sync = login_id_session_local_sync();
                        // re-send SetName to server
                        if login_name_session_local_sync != *DISCONNECTED_USER
                            && login_id_session_local_sync != NO_CLIENT_ID
                        {
                            let _ = socket
                                .clone()
                                .send(ClientEvent::AddPlayer(
                                    login_name_session_local_sync.clone(),
                                ))
                                .await;
                            tracing::info!(
                                "Client {} sent AddPlayer for player {} (id {})",
                                client_id,
                                login_name_session_local_sync,
                                login_id_session_local_sync
                            );
                            let _ = socket
                                .clone()
                                .send(ClientEvent::RequestSavedGameList(
                                    login_name_session_local_sync.clone(),
                                ))
                                .await;
                            let _ = socket
                                .clone()
                                .send(ClientEvent::RequestOnGoingGamesList)
                                .await;
                        }
                    }
                    ServerEvent::InitClient(id, characters_list) => {
                        player_client_id.set(id);
                        // set character list
                        all_characters_names.set(characters_list);
                        tracing::info!(
                            "Client {} received characters list with {} characters",
                            id,
                            all_characters_names().len(),
                        );
                    }
                    ServerEvent::UpdateServerData(server_data_update) => {
                        // update server info
                        server_data.set(*server_data_update.clone());
                        *SERVER_NAME.write() =
                            server_data_update.core_game_data.server_name.clone();
                    }
                    ServerEvent::UpdateOngoingGames(ongoing_games_update) => {
                        ongoing_games.set(ongoing_games_update);
                    }
                    ServerEvent::ReconnectAllSessions(username, sql_id) => {
                        let login_name_session_local_sync = login_name_session_local_sync();
                        let login_id_session_local_sync = login_id_session_local_sync();
                        if login_name_session_local_sync == username
                            && login_id_session_local_sync == sql_id
                        {
                            tracing::info!(
                                "ReconnectAllSessions for player {}",
                                login_name_session_local_sync
                            );
                            let _ = socket
                                .clone()
                                .send(ClientEvent::AddPlayer(
                                    login_name_session_local_sync.clone(),
                                ))
                                .await;
                        } else {
                            tracing::info!(
                                "Skipping ReconnectAllSessions for player {} (username: {}, sql_id: {})",
                                login_name_session_local_sync,
                                username,
                                sql_id
                            );
                        }
                    }
                    ServerEvent::AnswerSavedGameList(games_list) => {
                        tracing::info!("Received saved game list with {} games", games_list.len());
                        saved_game_list.set(games_list);
                    }
                    ServerEvent::ResetClientFromServerData => {
                        tracing::info!("Reset client from server-data {}", SERVER_NAME());
                        server_data.set(ServerData::reset(GamePhase::Ended));
                        SERVER_NAME.write().clear();
                    }
                    ServerEvent::LogOut => {
                        tracing::info!("Received LogOut event, resetting client data");
                        server_data.set(ServerData::default());
                        SERVER_NAME.write().clear();
                        login_name_session_local_sync.set(DISCONNECTED_USER.clone());
                        login_id_session_local_sync.set(NO_CLIENT_ID);
                    }
                    ServerEvent::SetAtkAnimation(is_animated) => {
                        tracing::debug!("Received SetAtkAnimation event");
                        toggle_atk_animation.set(is_animated);
                    }
                    ServerEvent::OverworldEntered(map_id) => {
                        tracing::info!("[client] OverworldEntered: {}", map_id);
                        overworld_map_id.set(Some(map_id));
                    }
                }
            }
            tracing::warn!("[client] ws-loop EXITED — deserialization error or socket closed");
        }
    });

    use_context_provider(|| socket);
    use_context_provider(|| player_client_id);
    use_context_provider(|| login_name_session_local_sync);
    use_context_provider(|| login_id_session_local_sync);
    use_context_provider(|| server_data);
    use_context_provider(|| overworld_map_id);
    use_context_provider(|| ongoing_games);
    use_context_provider(|| saved_game_list);
    use_context_provider(|| all_characters_names);
    // Wrap each bool signal in a distinct newtype so Dioxus context lookup
    // (keyed by TypeId) stores them independently instead of all colliding on Signal<bool>.
    use_context_provider(|| CtxToggleAtkAnimation(toggle_atk_animation));
    // Show attack tooltips — default true, overridden from DB once settings load
    let show_atk_tooltips: Signal<bool> = use_signal(|| true);
    use_context_provider(|| CtxShowAtkTooltips(show_atk_tooltips));
    // Show boss energy bars — default hidden
    let show_boss_energy: Signal<bool> = use_signal(|| true);
    use_context_provider(|| CtxShowBossEnergy(show_boss_energy));
    // Show hero aggro — default hidden
    let show_hero_aggro: Signal<bool> = use_signal(|| true);
    use_context_provider(|| CtxShowHeroAggro(show_hero_aggro));
    // Show boss HP bar — default visible
    let show_boss_hp: Signal<bool> = use_signal(|| true);
    use_context_provider(|| CtxShowBossHp(show_boss_hp));
    // Auto-save at the start of each scenario — default enabled
    let auto_save_scenario: Signal<bool> = use_signal(|| true);
    use_context_provider(|| CtxAutoSaveScenario(auto_save_scenario));
    // Shop access during an active scenario — default disabled
    let shop_enabled: Signal<bool> = use_signal(|| false);
    use_context_provider(|| CtxShopEnabled(shop_enabled));
    // UI language ("en"/"fr") — localStorage-backed so it works pre-login
    use_context_provider(|| CtxAppLang(app_lang_local_sync));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: DX_COMP_CSS }

        Router::<Route> {}
    }
}
