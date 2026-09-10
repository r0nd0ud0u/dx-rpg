use dioxus::{
    fullstack::{WebSocketOptions, use_websocket},
    logger::tracing::{self, Level},
    prelude::*,
};
use dioxus_i18n::prelude::*;
use dioxus_sdk_storage::{LocalStorage, use_synced_storage};
#[cfg(all(not(feature = "server"), not(target_arch = "wasm32")))]
use dioxus_sdk_storage::{StorageBacking, set_dir};
#[cfg(not(target_arch = "wasm32"))]
use dotenv::dotenv;
use dx_rpg::{
    common::{
        ConnectionStatus, CtxAppLang, CtxAtkPanelOrders, CtxAudioSettings, CtxAutoSaveScenario,
        CtxCombatHintsOff, CtxConnectionLatency, CtxConnectionStatus, CtxDeviceToken,
        CtxOverworldZoom, CtxSessionExpired, CtxShopEnabled, CtxShowAtkTooltips, CtxShowBossEnergy,
        CtxShowBossHp, CtxShowHeroAggro, CtxSyncedInsecureCerts, CtxSyncedServerUrl,
        CtxToggleAtkAnimation, CtxTutorialSeen, DISCONNECTED_USER, DX_COMP_CSS, Route, SERVER_NAME,
        SYNCED_AUDIO_MUTED_KEY, SYNCED_BACKGROUND_AUDIO_KEY, SYNCED_COMBAT_HINTS_OFF_KEY,
        SYNCED_DEVICE_TOKEN_KEY, SYNCED_MUSIC_VOLUME_KEY, SYNCED_OVERWORLD_ZOOM_KEY,
        SYNCED_SFX_VOLUME_KEY, SYNCED_TUTORIAL_SEEN_KEY,
    },
    components::{
        alert_dialog, button, drag_and_drop_list, input, label, popover, select, separator, sheet,
        sidebar, tabs, tooltip,
    },
    websocket_handler::{
        NO_CLIENT_ID,
        event::{ClientEvent, ServerEvent, on_rcv_client_event},
    },
};
// These constants are only used in the native (non-web, non-server) build path where
// CtxSyncedServerUrl / CtxSyncedInsecureCerts are backed by use_synced_storage.
#[cfg(all(not(feature = "server"), not(target_arch = "wasm32")))]
use dx_rpg::common::{SYNCED_INSECURE_CERTS_KEY, SYNCED_SERVER_URL_KEY};
use lib_rpg::server::server_manager::{GamePhase, ServerData};
use unic_langid::langid;
// StorageBacking is needed on wasm32 to call LocalStorage::get / LocalStorage::set
// (trait methods) inside the login-restore effect.  On native it is already
// imported above via the #[cfg(all(not(feature="server"),not(target_arch="wasm32")))]
// block; on the server binary the effect block doesn't compile at all.
#[cfg(target_arch = "wasm32")]
use dioxus_sdk_storage::StorageBacking;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

/// The `<head>` both native clients boot from.
///
/// Everything a webview needs for a correct *first* paint has to be in the initial HTML:
/// App()'s `document::Link` stylesheets are injected by an effect that runs after that
/// paint, which is what used to make the launch flash unstyled. The App()-root links stay
/// — web needs them, and here they harmlessly re-apply the same hrefs.
#[cfg(all(not(feature = "server"), any(feature = "desktop", feature = "mobile")))]
fn native_boot_head() -> String {
    // Must be kept in sync by hand with App()'s document::Link list: one is a const
    // here, the other is markup inside the component.
    let stylesheets: &[Asset] = &[
        MAIN_CSS,
        dx_rpg::common::DX_COMP_CSS,
        dx_rpg::components::alert_dialog::STYLE_CSS,
        dx_rpg::components::button::STYLE_CSS,
        dx_rpg::components::drag_and_drop_list::STYLE_CSS,
        dx_rpg::components::input::STYLE_CSS,
        dx_rpg::components::label::STYLE_CSS,
        dx_rpg::components::popover::STYLE_CSS,
        dx_rpg::components::select::STYLE_CSS,
        dx_rpg::components::separator::STYLE_CSS,
        dx_rpg::components::sheet::STYLE_CSS,
        dx_rpg::components::sidebar::STYLE_CSS,
        dx_rpg::components::tabs::STYLE_CSS,
        dx_rpg::components::tooltip::STYLE_CSS,
    ];
    let mut head = format!(r#"<link rel="icon" href="{FAVICON}">"#);
    // Inline, first, and request-free: whatever else is still in flight, the window is
    // already the app's dark ground in the right font instead of a white page.
    head.push_str(&format!(
        "<style>{}{}</style>",
        dx_rpg::common::boot_critical_css(),
        dx_rpg::common::inter_font_face_css()
    ));
    // A native client streams its first DOM only after `window.onload` (see
    // dioxus-desktop's module loader — `dioxus::mobile` is that same crate), so preloading
    // the two upright faces here means text is laid out in Inter the first time it is
    // painted, with no swap. `crossorigin` because font fetches are CORS-mode even
    // same-origin; the asset protocol answers with `Access-Control-Allow-Origin: *`.
    for subset in ["latin", "latin-ext"] {
        head.push_str(&format!(
            r#"<link rel="preload" as="font" type="font/woff2" crossorigin href="{}/inter-{subset}-normal.woff2">"#,
            dx_rpg::common::PATH_FONTS
        ));
    }
    for href in stylesheets {
        head.push_str(&format!(r#"<link rel="stylesheet" href="{href}">"#));
    }
    head
}

fn main() {
    // Reads the .env file. Native builds (server, and native clients below) can have one;
    // the browser (wasm32) has no filesystem so it never reads one.
    #[cfg(not(target_arch = "wasm32"))]
    dotenv().ok();

    // Init logger. Also mirrors every log line into an in-memory buffer the admin-only
    // DebugConsole reads from — see src/debug_console.rs for why: mobile builds have no
    // attached console to read stdout/logcat from.
    dx_rpg::debug_console::init(
        std::env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_owned())
            .parse::<Level>()
            .unwrap_or(Level::INFO),
    );
    tracing::info!("Rendering app!");

    // Native clients (desktop, mobile) connect to a remote multiplayer server over the same
    // websocket/server-fn protocol the web client uses, but — unlike a browser page, which
    // infers the server from same-origin — they have no origin to infer it from, so the
    // server URL must be set explicitly before launch.
    #[cfg(all(not(feature = "server"), not(target_arch = "wasm32")))]
    {
        // LocalStorage uses a filesystem backend on native and panics without this. On
        // Android `set_dir!()` panics too — `directories` has no Android support — so pass
        // the app's data path explicitly.
        #[cfg(target_os = "android")]
        set_dir!("/data/data/com.aogin.rpgadventure/files/rpg-adventure");
        #[cfg(not(target_os = "android"))]
        set_dir!();

        // Order: runtime env var (dev), then the in-app Server settings dialog's saved
        // value (read off disk — main() predates any hook), then the compile-time bake
        // (build.rs — the only option an installed APK has), then a fallback.
        let persisted_server_url =
            LocalStorage::get::<String>(&dx_rpg::common::SYNCED_SERVER_URL_KEY.to_owned())
                .filter(|s: &String| !s.is_empty());
        let server_url = std::env::var("SERVER_URL")
            .ok()
            .or(persisted_server_url)
            .or_else(|| option_env!("SERVER_URL").map(str::to_owned))
            .unwrap_or_else(|| "http://127.0.0.1:8080".to_owned());
        tracing::info!("Native client connecting to server at {server_url}");
        dioxus::fullstack::set_server_url(Box::leak(server_url.into_boxed_str()));

        // Opt-in escape hatch for a self-signed server cert. Disables validation for every
        // request (server fns and the websocket share one reqwest client), so it removes MITM
        // protection — trusted networks only. Same resolution order as SERVER_URL above.
        let persisted_insecure_certs =
            LocalStorage::get::<bool>(&dx_rpg::common::SYNCED_INSECURE_CERTS_KEY.to_owned());
        let insecure_accept_invalid_certs = std::env::var("INSECURE_ACCEPT_INVALID_CERTS")
            .ok()
            .or_else(|| persisted_insecure_certs.map(|b| b.to_string()))
            .or_else(|| option_env!("INSECURE_ACCEPT_INVALID_CERTS").map(str::to_owned));
        if insecure_accept_invalid_certs.as_deref() == Some("true") {
            tracing::warn!(
                "INSECURE_ACCEPT_INVALID_CERTS=true — TLS certificate validation is DISABLED for all requests to {}. Do not use this against an untrusted network or server.",
                dioxus::fullstack::get_server_url()
            );
            let client = dioxus::fullstack::reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .cookie_store(true)
                .build()
                .expect("failed to build insecure reqwest client");
            // Best-effort: if something already initialized the default client (shouldn't
            // happen this early), fall back to the validated default rather than panicking.
            let _ = dioxus::fullstack::GLOBAL_REQUEST_CLIENT.set(client);
        }
    }

    // Registers the build.rs-embedded offlines/ data with lib-rpg so offline mode can build
    // a DataManager. Must run before any hook exists. Cfg'd out of the server build.
    #[cfg(not(feature = "server"))]
    dx_rpg::embedded_data::register();

    #[cfg(all(not(feature = "server"), feature = "desktop"))]
    {
        // `with_icon` below never reaches Wayland — GTK3 implements no per-window icon
        // protocol there. The compositor matches the xdg-shell app_id against an installed
        // .desktop file instead, and GTK3 takes that app_id from `g_get_prgname()`, i.e.
        // basename(argv[0]). `dx serve` runs a hashed copy (`rpg-adventure-<hash>`) that
        // matches no .desktop file, so pin the name. Must precede gtk_init().
        #[cfg(all(
            unix,
            not(target_os = "macos"),
            not(target_os = "android"),
            not(target_os = "ios")
        ))]
        glib::set_prgname(Some("rpg-adventure"));

        // `dx serve` opens the window straight through tao/wry, bypassing Dioxus.toml's
        // `[bundle].icon` (read only by `dx bundle`), so set it here from the same PNG.
        // X11/Windows/macOS only, per the Wayland note above.
        let icon_png = include_bytes!("../assets/icon-512.png");
        let icon = image::load_from_memory(icon_png)
            .expect("assets/icon-512.png must be a valid image")
            .into_rgba8();
        let (icon_width, icon_height) = icon.dimensions();
        let window_icon =
            dioxus_desktop::tao::window::Icon::from_rgba(icon.into_raw(), icon_width, icon_height)
                .expect("assets/icon-512.png must be a valid RGBA icon");

        dioxus::LaunchBuilder::new()
            .with_cfg(
                dioxus_desktop::Config::new()
                    .with_custom_head(native_boot_head())
                    // Painted by the webview before it has a document at all — without it
                    // the window opens white for as long as the first paint takes.
                    .with_background_color(dx_rpg::common::BOOT_BG_RGBA)
                    .with_icon(window_icon),
            )
            .launch(App);
    }

    // Android/iOS. `dioxus::mobile` *is* dioxus-desktop (same webview stack), so the
    // launch gets the same treatment as the desktop client above, minus the window icon
    // and prgname, which are desktop window-manager concerns. The system window behind
    // the webview is themed by the platform, not from here.
    #[cfg(all(not(feature = "server"), feature = "mobile", not(feature = "desktop")))]
    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::mobile::Config::new()
                .with_custom_head(native_boot_head())
                .with_background_color(dx_rpg::common::BOOT_BG_RGBA),
        )
        .launch(App);

    // Web: dx generates index.html and the fullstack server renders App()'s
    // `document::Link`s into it, so the markup arrives styled with no head to patch.
    #[cfg(all(
        not(feature = "server"),
        not(feature = "desktop"),
        not(feature = "mobile")
    ))]
    dioxus::launch(App);

    // `dioxus::serve` takes a closure returning an `axum::Router` and wires up logging,
    // hot-reloading, devtools and the IP/PORT env vars.
    #[cfg(feature = "server")]
    dioxus::serve(|| async {
        use axum_session::{SessionConfig, SessionLayer, SessionStore};
        use axum_session_auth::AuthConfig;
        use axum_session_sqlx::SessionSqlitePool;
        use dx_rpg::auth_manager::{
            auth::AuthLayer,
            db::get_db,
            server_fn::{auth_rate_limit, update_all_connection_status},
        };

        // Id for a request with no valid session cookie. Must never match a real
        // `users.id` — db.rs seeds Admin at 1, so reusing `STARTING_CLIENT_ID` (also 1) made
        // every cookie-less request load Admin and pass `require_admin`. SQLite rowids start
        // at 1, so `0` is safe; `User::load_user` returns an empty-permission anonymous user
        // for an unknown id.
        const ANONYMOUS_SESSION_USER_ID: i64 = 0;

        let bind_ip = std::env::var("IP").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let bind_port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_owned());
        let is_main_server = std::env::var("IS_MAIN_SERVER")
            .unwrap_or_else(|_| "false".to_owned())
            .trim()
            .to_lowercase()
            == "true";
        tracing::info!(
            "dx-rpg server starting on {}:{} (main server: {})",
            bind_ip,
            bind_port,
            is_main_server
        );

        // Only the main instance clears stale is_connected at start; a second process on the
        // same database would wipe the flag for users live on the main one.
        if is_main_server {
            update_all_connection_status(false).await.unwrap();
        }
        // create db pool for session store
        let pool = get_db().await;

        // initialize data manager
        init_data_manager().await;

        // Explicit rather than axum_session's defaults: after SESSION_IDLE_TIMEOUT_HOURS
        // idle (renewed by any authenticated request) or SESSION_MAX_LIFETIME_DAYS outright,
        // `current_user` reverts to anonymous and `auth: Session` endpoints reject.
        // `AdminPage` turns that into a "session expired" prompt.
        let session_idle_timeout = chrono::Duration::hours(
            std::env::var("SESSION_IDLE_TIMEOUT_HOURS")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(24),
        );
        let session_max_lifetime = chrono::Duration::days(
            std::env::var("SESSION_MAX_LIFETIME_DAYS")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(30),
        );

        // Create an axum router that dioxus will attach the app to
        Ok(dioxus::server::router(App)
            .route("/img-srv/{filename}", axum::routing::get(serve_img_handler))
            .layer(axum::middleware::from_fn(auth_rate_limit))
            .layer(
                AuthLayer::new(Some(pool.clone())).with_config(
                    AuthConfig::<i64>::default()
                        .with_anonymous_user_id(Some(ANONYMOUS_SESSION_USER_ID)),
                ),
            )
            .layer(SessionLayer::new(
                SessionStore::<SessionSqlitePool>::new(
                    Some(pool.clone().into()),
                    SessionConfig::default()
                        .with_table_name("test_table")
                        .with_lifetime(session_idle_timeout)
                        .with_max_lifetime(session_max_lifetime)
                        .with_max_age(Some(session_max_lifetime)),
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
fn is_safe_filename(filename: &str) -> bool {
    !filename.contains("..") && !filename.contains('/') && !filename.contains('\\')
}

#[cfg(feature = "server")]
pub async fn serve_img_handler(
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> axum::response::Response {
    use axum::http::{StatusCode, header};
    use axum::response::IntoResponse;
    // Security: reject any path traversal attempt
    if !is_safe_filename(&filename) {
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
    *dm = DataManager::try_new(OFFLINE_PATH).unwrap_or_else(|e| {
        let cwd = std::env::current_dir().unwrap_or_default();
        eprintln!(
            "Failed to load game data from \"{OFFLINE_PATH}\" (resolved to {}): {e}\n\n\
             The server binary must be run from a directory that contains the full \
             \"{OFFLINE_PATH}/\" folder alongside it (this is what the self-hostable \
             web/server bundle ships, e.g. bundle_web_linux.zip / bundle_web_windows.zip — not \
             the desktop/mobile client-only bundles, which don't include game data).",
            cwd.join(OFFLINE_PATH).display(),
        );
        std::process::exit(1);
    });
    tracing::info!(
        "Data manager initialized with {} equipments and {} heroes",
        dm.equipment_table.len(),
        dm.all_heroes.len()
    );
    drop(dm);
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn is_safe_filename_rejects_traversal_and_separators() {
        assert!(!is_safe_filename("../secret.png"));
        assert!(!is_safe_filename("a/b.png"));
        assert!(!is_safe_filename("a\\b.png"));
        assert!(!is_safe_filename(".."));
    }

    #[test]
    fn is_safe_filename_accepts_plain_names() {
        assert!(is_safe_filename("Elara.png"));
        assert!(is_safe_filename("some-file_1.jpg"));
    }

    #[test]
    fn resolve_static_img_finds_dev_asset() {
        // assets/img/Elara.png ships in the repo, so cargo test's CWD (crate root) finds it.
        let path = resolve_static_img("Elara.png");
        assert_eq!(path, std::path::Path::new("assets/img").join("Elara.png"));
        assert!(path.exists());
    }

    #[test]
    fn resolve_static_img_falls_back_to_dev_path_when_missing() {
        // No public/assets/img-*/ directory exists in a source checkout, so a filename
        // that isn't in assets/img/ either should fall back to the (non-existent) dev path.
        let path = resolve_static_img("does-not-exist-anywhere.png");
        assert_eq!(
            path,
            std::path::Path::new("assets/img").join("does-not-exist-anywhere.png")
        );
        assert!(!path.exists());
    }
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
    // Websocket link state, shown as Navbar's status badge. Starts optimistic —
    // `use_websocket` connects synchronously and the ws-loop flips it on failure.
    let mut connection_status = use_signal(|| ConnectionStatus::Connected);
    // See `CtxSessionExpired`'s doc comment — set by `AdminPage` when it discovers the
    // live server session no longer matches the identity persisted locally.
    let session_expired = use_signal(|| false);
    // Last measured round-trip latency to the server, from the ping loop below — `None`
    // before the first measurement or after one times out (see `CtxConnectionLatency`'s
    // doc comment for why that case matters separately from `connection_status`).
    let mut latency_ms: Signal<Option<u64>> = use_signal(|| None);
    // (nonce, sent-at) of the ping awaiting a Pong. The nonce stops a late Pong from a
    // timed-out ping being read as a reply to the next one.
    let mut pending_ping: Signal<Option<(u64, web_time::Instant)>> = use_signal(|| None);

    // Log which server URL this client is about to talk to (server-fn calls + websocket) —
    // same-origin implicit on web/server, explicit remote target on native — to make
    // connectivity issues visible without needing native-only debugging.
    use_effect(|| {
        tracing::info!(
            "[client] connecting to server at {}",
            dioxus::fullstack::get_server_url()
        );
    });

    let socket = use_websocket(|| on_rcv_client_event(WebSocketOptions::new()));

    // GameChannel wraps `socket` plus, on client builds, a LocalChannel that routes into
    // local_engine once `offline_mode` flips. See game_channel.rs for why it's cfg-gated.
    #[cfg(not(feature = "server"))]
    let local_channel_handle = dx_rpg::local_channel::LocalChannel::new();
    #[cfg(not(feature = "server"))]
    let offline_mode = use_signal(|| false);
    #[cfg(not(feature = "server"))]
    let game_channel =
        dx_rpg::game_channel::GameChannel::new(socket, local_channel_handle, offline_mode);
    #[cfg(feature = "server")]
    let game_channel = dx_rpg::game_channel::GameChannel::new(socket);

    // login_name picks the page (LoginPage vs home), so both sides must start at
    // DISCONNECTED_USER or hydration renders different trees — DioxusLabs/dioxus#3583.
    // Native has no hydration, so use_synced_storage is fine there.
    #[cfg(any(target_arch = "wasm32", feature = "server"))]
    let mut login_name_session_local_sync = use_signal(|| DISCONNECTED_USER.clone());
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "server")))]
    let mut login_name_session_local_sync =
        use_synced_storage::<LocalStorage, String>("synced_user_sql_name".to_owned(), || {
            DISCONNECTED_USER.clone()
        });
    let mut login_id_session_local_sync =
        use_synced_storage::<LocalStorage, i64>("synced_user_sql_id".to_owned(), || NO_CLIENT_ID); // from db, integer primary key not null and from 1 upwards
    // Persistent per-device/browser random token — generated once, lazily, on first read.
    // See CtxDeviceToken / SYNCED_DEVICE_TOKEN_KEY doc comments in common.rs.
    let device_token_local_sync =
        use_synced_storage::<LocalStorage, String>(SYNCED_DEVICE_TOKEN_KEY.to_owned(), || {
            format!("{:032x}", rand::random::<u128>())
        });
    let app_lang_local_sync =
        use_synced_storage::<LocalStorage, String>("synced_app_lang".to_owned(), || {
            "en".to_owned()
        });
    // Audio settings — same on every platform (no native/web split needed), so unlike
    // CtxSyncedServerUrl below these are plain use_synced_storage calls.
    let music_volume_local_sync =
        use_synced_storage::<LocalStorage, i32>(SYNCED_MUSIC_VOLUME_KEY.to_owned(), || 60);
    let sfx_volume_local_sync =
        use_synced_storage::<LocalStorage, i32>(SYNCED_SFX_VOLUME_KEY.to_owned(), || 80);
    let audio_muted_local_sync =
        use_synced_storage::<LocalStorage, bool>(SYNCED_AUDIO_MUTED_KEY.to_owned(), || false);
    let background_audio_local_sync =
        use_synced_storage::<LocalStorage, bool>(SYNCED_BACKGROUND_AUDIO_KEY.to_owned(), || false);
    let overworld_zoom_local_sync =
        use_synced_storage::<LocalStorage, f32>(SYNCED_OVERWORLD_ZOOM_KEY.to_owned(), || {
            dx_rpg::board_game_components::overworld::DEFAULT_ZOOM
        });
    // Read once at startup so a fresh install opens the how-to-play dialog by itself;
    // Navbar sets it when the dialog is closed. See CtxTutorialSeen in common.rs.
    let tutorial_seen_local_sync =
        use_synced_storage::<LocalStorage, bool>(SYNCED_TUTORIAL_SEEN_KEY.to_owned(), || false);
    let combat_hints_off_local_sync =
        use_synced_storage::<LocalStorage, bool>(SYNCED_COMBAT_HINTS_OFF_KEY.to_owned(), || false);
    // Native-only server URL/TLS override from Navbar's settings dialog. Declared here
    // because use_synced_storage in Navbar (a #[layout] component) stack-overflows.
    // Declared unconditionally for hook order, inert off native — DioxusLabs/dioxus#3583.
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "server")))]
    let synced_server_url =
        use_synced_storage::<LocalStorage, String>(SYNCED_SERVER_URL_KEY.to_owned(), || {
            dioxus::fullstack::get_server_url().to_owned()
        });
    #[cfg(any(target_arch = "wasm32", feature = "server"))]
    let synced_server_url = use_signal(|| dioxus::fullstack::get_server_url().to_owned());

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "server")))]
    let synced_insecure_certs =
        use_synced_storage::<LocalStorage, bool>(SYNCED_INSECURE_CERTS_KEY.to_owned(), || false);
    #[cfg(any(target_arch = "wasm32", feature = "server"))]
    let synced_insecure_certs = use_signal(|| false);

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

    // Set the theme to dark on app load.
    // `document::eval` (not raw web_sys) so this also works on desktop/mobile clients,
    // which don't compile web_sys (it's a wasm-bindgen crate, native targets don't have it).
    //
    // No longer what decides the first paint: this runs in an effect, i.e. after it, so
    // the launch used to flash the light palette on a light-themed system. Dark is now the
    // CSS default (`html:root` in assets/dx-components-theme.css) and the attribute only
    // states it explicitly — keep both, so switching themes later stays a one-liner.
    use_effect(|| {
        document::eval("document.documentElement.setAttribute('data-theme', 'dark');");
    });

    // Sets up the music/sfx `<audio>` elements once. The bridge starts with background
    // audio off, so the stored setting has to be pushed in right behind it.
    let audio_settings = CtxAudioSettings {
        music_volume: music_volume_local_sync,
        sfx_volume: sfx_volume_local_sync,
        muted: audio_muted_local_sync,
        background: background_audio_local_sync,
    };
    use_effect(move || {
        dx_rpg::audio::init_audio_bridge();
        dx_rpg::audio::set_background_audio(audio_settings);
    });

    // Keeps open tooltips inside the viewport whatever `side` the call site picked.
    use_effect(|| {
        dx_rpg::components::tooltip::init_positioning();
    });

    // Android's WebView ignores <meta viewport> and evaluates every `max-width` query
    // against a fake ~980px layout, so main.css's mobile breakpoints never match there.
    // `window.screen.width` reports the real width, so toggle `html.force-mobile-nav` from
    // it (see the matching rules in main.css). Redundant but harmless elsewhere.
    use_effect(|| {
        document::eval(
            r#"
            function applyForceMobileNav() {
                var w = (window.screen && window.screen.width) ? window.screen.width : window.innerWidth;
                document.documentElement.classList.toggle('force-mobile-nav', w <= 768);
            }
            applyForceMobileNav();
            window.addEventListener('resize', applyForceMobileNav);
            window.addEventListener('orientationchange', applyForceMobileNav);
            "#,
        );
    });

    // Web only: the first run (just after hydration) restores login_name from localStorage;
    // later runs persist changes. The Rc<Cell<bool>> tells the two apart.
    #[cfg(target_arch = "wasm32")]
    {
        use std::{cell::Cell, rc::Rc};
        let restored = use_hook(|| Rc::new(Cell::new(false)));
        use_effect(move || {
            let name = login_name_session_local_sync(); // subscribe so effect re-runs on changes
            if restored.get() {
                // After initial restoration: persist any changes to localStorage
                LocalStorage::set("synced_user_sql_name".to_owned(), &name);
            } else {
                // First call after hydration: restore the saved session (do NOT save yet,
                // to avoid overwriting localStorage with the default "not connected" value)
                restored.set(true);
                if let Some(saved) = LocalStorage::get::<String>(&"synced_user_sql_name".to_owned())
                {
                    if !saved.is_empty() && saved != *DISCONNECTED_USER {
                        login_name_session_local_sync.set(saved);
                    }
                }
            }
        });
    }

    // Receive events from the websocket and update local signals.
    //
    // Wrapped in an outer reconnect loop: `use_websocket` establishes the connection once and
    use_future(move || {
        // `socket` is only for the reconnect logic below (meaningless offline); everything
        // else goes through `game_channel`. Both are Copy, so no clones needed.
        let mut socket = socket;
        let mut game_channel = game_channel;
        async move {
            loop {
                tracing::info!("[client] ws-loop starting");
                while let Ok(event) = game_channel.recv().await {
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
                                let _ = game_channel
                                    .clone()
                                    .send(ClientEvent::AddPlayer(
                                        login_name_session_local_sync.clone(),
                                        device_token_local_sync(),
                                    ))
                                    .await;
                                tracing::info!(
                                    "Client {} sent AddPlayer for player {} (id {})",
                                    client_id,
                                    login_name_session_local_sync,
                                    login_id_session_local_sync
                                );
                                let _ = game_channel
                                    .clone()
                                    .send(ClientEvent::RequestSavedGameList(
                                        login_name_session_local_sync.clone(),
                                    ))
                                    .await;
                                let _ = game_channel
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
                                let _ = game_channel
                                    .clone()
                                    .send(ClientEvent::AddPlayer(
                                        login_name_session_local_sync.clone(),
                                        device_token_local_sync(),
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
                            tracing::info!(
                                "Received saved game list with {} games",
                                games_list.len()
                            );
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
                        ServerEvent::UpdateOverworld(overworld_update) => {
                            server_data.write().core_game_data.overworld = Some(*overworld_update);
                        }
                        ServerEvent::UpdateCombat(combat_update) => {
                            server_data
                                .write()
                                .core_game_data
                                .apply_combat_update(*combat_update);
                        }
                        ServerEvent::Pong(nonce) => {
                            if let Some((pending_nonce, sent_at)) = pending_ping()
                                && pending_nonce == nonce
                            {
                                latency_ms.set(Some(sent_at.elapsed().as_millis() as u64));
                                pending_ping.set(None);
                            }
                        }
                    }
                }
                // Offline, recv() only fails if LocalChannel's sender was dropped, and there
                // is no server to reconnect to — retry locally instead.
                #[cfg(not(feature = "server"))]
                if offline_mode() {
                    tracing::warn!("[client] ws-loop: local channel closed unexpectedly, retrying");
                    continue;
                }

                tracing::warn!(
                    "[client] ws-loop: connection lost (deserialization error or socket closed), reconnecting"
                );
                connection_status.set(ConnectionStatus::Reconnecting);
                latency_ms.set(None);
                pending_ping.set(None);

                // Capped exponential backoff: reconnect quickly on the first attempts (matters
                // for a briefly backgrounded mobile client racing the server's grace period),
                // then back off so a genuinely offline client doesn't hammer the server.
                let mut backoff = std::time::Duration::from_secs(1);
                const MAX_RECONNECT_BACKOFF: std::time::Duration =
                    std::time::Duration::from_secs(10);
                loop {
                    // Without this the loop retries forever (its only exit is a successful
                    // reconnect), never returning to the outer `game_channel.recv()` — so
                    // clicking "Play Offline" while stuck here would leave the local
                    // channel's queued events undrained and the screen blank.
                    #[cfg(not(feature = "server"))]
                    if offline_mode() {
                        tracing::info!(
                            "[client] ws-loop: offline mode activated mid-reconnect, abandoning it"
                        );
                        break;
                    }
                    let reconnect_result = on_rcv_client_event(WebSocketOptions::new()).await;
                    let reconnected = reconnect_result.is_ok();
                    if let Err(ref err) = reconnect_result {
                        tracing::warn!(
                            "[client] ws-loop: reconnect attempt failed ({err:?}), retrying in {backoff:?}"
                        );
                    }
                    socket.set(reconnect_result);
                    if reconnected {
                        tracing::info!("[client] ws-loop: reconnected");
                        connection_status.set(ConnectionStatus::Connected);
                        break;
                    }
                    dioxus_sdk_time::sleep(backoff).await;
                    backoff = (backoff * 2).min(MAX_RECONNECT_BACKOFF);
                }
            }
        }
    });

    // Round-trip probe, separate from the ws-loop: a congested socket stays open and
    // `Connected` while delivering nothing. One ping per `PING_INTERVAL`; no Pong before the
    // next is due counts as lost.
    use_future(move || {
        let game_channel = game_channel;
        async move {
            // Also doubles as the timeout window for the previous ping (see below) — no
            // point pinging more often than that window anyway.
            const PING_INTERVAL: std::time::Duration = std::time::Duration::from_secs(4);
            let mut nonce: u64 = 0;
            loop {
                if game_channel.is_offline() {
                    dioxus_sdk_time::sleep(PING_INTERVAL).await;
                    continue;
                }
                nonce = nonce.wrapping_add(1);
                let sent_at = web_time::Instant::now();
                pending_ping.set(Some((nonce, sent_at)));
                let _ = game_channel.send(ClientEvent::Ping(nonce)).await;
                dioxus_sdk_time::sleep(PING_INTERVAL).await;
                // Still the ping we just sent, unanswered a full interval later: give up on
                // it rather than let a stale Pong keep matching a much later ping's nonce.
                if matches!(pending_ping(), Some((pending_nonce, _)) if pending_nonce == nonce) {
                    latency_ms.set(None);
                    pending_ping.set(None);
                }
            }
        }
    });

    use_context_provider(|| game_channel);
    use_context_provider(|| player_client_id);
    use_context_provider(|| login_name_session_local_sync);
    use_context_provider(|| login_id_session_local_sync);
    use_context_provider(|| CtxDeviceToken(device_token_local_sync));
    use_context_provider(|| CtxAudioSettings {
        music_volume: music_volume_local_sync,
        sfx_volume: sfx_volume_local_sync,
        muted: audio_muted_local_sync,
        background: background_audio_local_sync,
    });
    use_context_provider(|| CtxOverworldZoom(overworld_zoom_local_sync));
    use_context_provider(|| CtxTutorialSeen(tutorial_seen_local_sync));
    use_context_provider(|| CtxCombatHintsOff(combat_hints_off_local_sync));
    use_context_provider(|| server_data);
    use_context_provider(|| overworld_map_id);
    use_context_provider(|| CtxConnectionStatus(connection_status));
    use_context_provider(|| CtxSessionExpired(session_expired));
    use_context_provider(|| CtxConnectionLatency(latency_ms));
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
    // Per-character custom attack panel order — empty until loaded on demand by AttackList
    let atk_panel_orders: Signal<std::collections::HashMap<String, Vec<String>>> =
        use_signal(std::collections::HashMap::new);
    use_context_provider(|| CtxAtkPanelOrders(atk_panel_orders));
    // UI language ("en"/"fr") — localStorage-backed so it works pre-login
    use_context_provider(|| CtxAppLang(app_lang_local_sync));
    // Native clients only: server address / TLS-validation override, editable from the
    // Server settings dialog in Navbar (see the doc comment on CtxSyncedServerUrl above).
    use_context_provider(|| CtxSyncedServerUrl(synced_server_url));
    use_context_provider(|| CtxSyncedInsecureCerts(synced_insecure_certs));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        // Bundled Inter (assets/fonts/). Declared here for web and mobile; the desktop
        // client also inlines the same rules into its index head, early enough to matter
        // at launch (see the `with_custom_head` block in main()).
        document::Style { {dx_rpg::common::inter_font_face_css()} }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: DX_COMP_CSS }
        // Shared dx-components-library stylesheets: loaded here (app root) rather than via
        // a document::Link nested inside each component's own render — dioxus-desktop only
        // injects document::Link stylesheets declared at the App() root into <head>, not
        // ones declared inside a child component (see each component's STYLE_CSS comment).
        document::Link { rel: "stylesheet", href: alert_dialog::STYLE_CSS }
        document::Link { rel: "stylesheet", href: button::STYLE_CSS }
        document::Link { rel: "stylesheet", href: drag_and_drop_list::STYLE_CSS }
        document::Link { rel: "stylesheet", href: input::STYLE_CSS }
        document::Link { rel: "stylesheet", href: label::STYLE_CSS }
        document::Link { rel: "stylesheet", href: popover::STYLE_CSS }
        document::Link { rel: "stylesheet", href: select::STYLE_CSS }
        document::Link { rel: "stylesheet", href: separator::STYLE_CSS }
        document::Link { rel: "stylesheet", href: sheet::STYLE_CSS }
        document::Link { rel: "stylesheet", href: sidebar::STYLE_CSS }
        document::Link { rel: "stylesheet", href: tabs::STYLE_CSS }
        document::Link { rel: "stylesheet", href: tooltip::STYLE_CSS }

        Router::<Route> {}
    }
}
