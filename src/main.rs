use dioxus::{
    fullstack::{WebSocketOptions, use_websocket},
    logger::tracing::{self, Level},
    prelude::*,
};
use dioxus_sdk_storage::{LocalStorage, use_synced_storage};
use dx_rpg::{
    application::Application,
    common::{DX_COMP_CSS, Route, SERVER_NAME, disconnected_user},
    websocket_handler::{
        event::{ServerEvent, new_event},
        game_state::GameStateWebsocket,
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
                    .with_config(AuthConfig::<i64>::default().with_anonymous_user_id(Some(1))),
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
    let mut player_id = use_signal(|| 0);
    let mut game_state = use_signal(GameStateWebsocket::default);
    let mut app = use_signal(Application::default);

    let socket = use_websocket(|| new_event(WebSocketOptions::new()));

    // synced storage
    let login_session_local =
        use_synced_storage::<LocalStorage, String>("synced".to_string(), || disconnected_user());

    // Receive events from the websocket and update local signals.
    use_future(move || {
        let mut socket = socket;
        async move {
            while let Ok(event) = socket.recv().await {
                match event {
                    ServerEvent::Message(msg) => {
                        message.set(msg);
                    }
                    ServerEvent::AssignPlayerId(id) => {
                        player_id.set(id);
                    }
                    ServerEvent::SnapshotPlayers(gs) => {
                        game_state.set(gs);
                    }
                    ServerEvent::UpdateApplication(app_update) => {
                        if SERVER_NAME() == app_update.server_name {
                            // check auto update
                            // lauunch attack
                            app.set(app_update);
                        }
                    }
                }
            }
        }
    });

    use_context_provider(|| socket);
    use_context_provider(|| player_id);
    use_context_provider(|| game_state);
    use_context_provider(|| login_session_local);
    use_context_provider(|| app);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: DX_COMP_CSS }

        Router::<Route> {}
    }
}
