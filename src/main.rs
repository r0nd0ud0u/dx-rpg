use dioxus::{
    fullstack::{WebSocketOptions, use_websocket},
    logger::tracing::{self, Level},
    prelude::*,
};
use dioxus_sdk_storage::{LocalStorage, use_synced_storage};
use dx_rpg::{
    common::{DX_COMP_CSS, Route, SERVER_NAME, disconnected_user},
    websocket_handler::{
        event::{ClientEvent, ServerEvent, on_rcv_client_event},
        game_state::{GamePhase, ServerData},
    },
};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    // Init logger
    dioxus::logger::init(Level::INFO).expect("failed to init logger");
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
        use dx_rpg::auth_manager::{auth::AuthLayer, db::get_db};

        let pool = get_db().await;

        // Create an axum router that dioxus will attach the app to
        Ok(dioxus::server::router(App)
            .layer(
                AuthLayer::new(Some(pool.clone()))
                    .with_config(AuthConfig::<i64>::default().with_anonymous_user_id(Some(1))), // TODO default anonymous user id, try -1 by default?
            )
            .layer(SessionLayer::new(
                SessionStore::<SessionSqlitePool>::new(
                    Some(pool.clone().into()),
                    SessionConfig::default().with_table_name("test_table"),
                )
                .await?,
            )))
    });
}

#[component]
fn App() -> Element {
    // Local UI state
    let mut message = use_signal(String::new);
    let mut player_client_id = use_signal(|| 0);
    let mut server_data = use_signal(ServerData::default);
    let mut ongoing_games = use_signal(Vec::new);
    let mut saved_game_list = use_signal(Vec::new);

    let socket = use_websocket(|| on_rcv_client_event(WebSocketOptions::new()));

    // synced storage
    let login_name_session_local_sync =
        use_synced_storage::<LocalStorage, String>("synced_user_sql_name".to_string(), || {
            disconnected_user()
        });
    let login_id_session_local_sync =
        use_synced_storage::<LocalStorage, i64>("synced_user_sql_id".to_string(), || -1); // from db, integer primary key not null and from 1 upwards

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
                        if login_name_session_local_sync != disconnected_user()
                            && login_id_session_local_sync != -1
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
                        }
                    }
                    ServerEvent::AssignPlayerId(id) => {
                        player_client_id.set(id);
                    }
                    ServerEvent::UpdateServerData(server_data_update) => {
                        // update server info
                        server_data.set(*server_data_update.clone());
                        *SERVER_NAME.write() = server_data_update.app.server_name.clone();
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

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: DX_COMP_CSS }

        Router::<Route> {}
    }
}
