use dioxus::{
    fullstack::{WebSocketOptions, use_websocket},
    logger::tracing::{self, Level},
    prelude::*,
};
use dioxus_sdk_storage::{LocalStorage, use_synced_storage};
use dotenv::dotenv;
use dx_rpg::{
    common::{DISCONNECTED_USER, DX_COMP_CSS, Route, SERVER_NAME},
    websocket_handler::{
        NO_CLIENT_ID,
        event::{ClientEvent, ServerEvent, on_rcv_client_event},
    },
};
use lib_rpg::server::server_manager::{GamePhase, ServerData};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    // Reads the .env file
    #[cfg(feature = "server")]
    dotenv().ok();

    // Init logger
    let _ = dioxus::logger::init(
        std::env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string())
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
    // Local UI state
    let mut message = use_signal(String::new);
    let mut player_client_id = use_signal(|| 0);
    let mut server_data = use_signal(ServerData::default);
    let mut ongoing_games = use_signal(Vec::new);
    let mut saved_game_list = use_signal(Vec::new);
    let mut all_characters_names = use_signal(Vec::new);

    let socket = use_websocket(|| on_rcv_client_event(WebSocketOptions::new()));

    // synced storage
    let mut login_name_session_local_sync =
        use_synced_storage::<LocalStorage, String>("synced_user_sql_name".to_string(), || {
            DISCONNECTED_USER.clone()
        });
    let mut login_id_session_local_sync =
        use_synced_storage::<LocalStorage, i64>("synced_user_sql_id".to_string(), || NO_CLIENT_ID); // from db, integer primary key not null and from 1 upwards

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
            while let Ok(event) = socket.recv().await {
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
                            "Client {} received characters list with {} characters: {:?}",
                            id,
                            all_characters_names().len(),
                            all_characters_names()
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
                }
            }
        }
    });

    use_context_provider(|| socket);
    use_context_provider(|| player_client_id);
    use_context_provider(|| login_name_session_local_sync);
    use_context_provider(|| login_id_session_local_sync);
    use_context_provider(|| server_data);
    use_context_provider(|| ongoing_games);
    use_context_provider(|| saved_game_list);
    use_context_provider(|| all_characters_names);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: DX_COMP_CSS }

        Router::<Route> {}
    }
}
