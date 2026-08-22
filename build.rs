use std::{env, fs, path::Path};

fn main() {
    // `SERVER_URL`/`INSECURE_ACCEPT_INVALID_CERTS` can be baked in at compile time (via
    // `option_env!` in src/main.rs) as a fallback for native builds — desktop clients get
    // a real environment at launch, but an installed Android APK has no shell to read
    // env vars from at runtime, so mobile releases need the value baked in at build time.
    // Without these, cargo wouldn't know to recompile when only the env var (not any
    // source file) changes between builds, silently keeping a stale baked-in value.
    println!("cargo:rerun-if-env-changed=SERVER_URL");
    println!("cargo:rerun-if-env-changed=INSECURE_ACCEPT_INVALID_CERTS");

    // Client builds only (web/desktop/mobile) — the server build reads `offlines/`
    // straight off the real filesystem (see `init_data_manager` in src/main.rs) and
    // doesn't need this at all. Embedding lets every client target call
    // `lib_rpg::utils::set_embedded_files` once at startup and have
    // `DataManager::try_new(OFFLINE_PATH)` work identically to the server's real-fs
    // path — required on web (wasm32 has no real filesystem) and used uniformly on
    // desktop/mobile too, so there's no loose `offlines/` directory to locate next to
    // an installed binary.
    if env::var_os("CARGO_FEATURE_SERVER").is_some() {
        return;
    }
    embed_offline_files();
}

/// Walks `offlines/` and generates `$OUT_DIR/embedded_offline_files.rs`, a
/// `pub static EMBEDDED_OFFLINE_FILES: &[(&str, &str)]` of (relative-path, content) pairs
/// via `include_str!` — one entry per file. Keys are relative to the crate root with
/// forward slashes (e.g. `"offlines/characters/lotr/Thraïn.json"`), matching exactly what
/// `Path::join` produces starting from `common::OFFLINE_PATH` ("offlines") — the same
/// paths `DataManager::try_new`'s internal loaders join and look up.
fn embed_offline_files() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is always set");
    let offlines_root = Path::new(&manifest_dir).join("offlines");
    println!("cargo:rerun-if-changed=offlines");

    let mut entries = Vec::new();
    walk(&offlines_root, &manifest_dir, &mut entries);
    entries.sort();

    let mut out = String::new();
    out.push_str("pub static EMBEDDED_OFFLINE_FILES: &[(&str, &str)] = &[\n");
    for (key, abs_path) in &entries {
        out.push_str(&format!("    ({key:?}, include_str!({abs_path:?})),\n",));
    }
    out.push_str("];\n");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is always set for build scripts");
    let dest = Path::new(&out_dir).join("embedded_offline_files.rs");
    fs::write(&dest, out).unwrap_or_else(|e| panic!("failed to write {}: {e}", dest.display()));
}

/// Recursively collects `(relative_key, absolute_path)` for every file under `dir`.
/// `relative_key` is `dir`'s path relative to `manifest_dir`, forward-slash-normalized.
fn walk(dir: &Path, manifest_dir: &str, out: &mut Vec<(String, String)>) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        // Missing offlines/ at build time is a real problem, but failing the whole build
        // over it would block e.g. `cargo check` in a checkout that hasn't fetched game
        // data yet — same "warn, don't hard-fail" spirit as init_data_manager's own
        // missing-offlines handling in src/main.rs.
        println!(
            "cargo:warning=offlines/ not found at {} — client build will ship with no embedded game data",
            dir.display()
        );
        return;
    };
    for entry in read_dir {
        let entry = entry.unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            walk(&path, manifest_dir, out);
        } else {
            let relative = path
                .strip_prefix(manifest_dir)
                .unwrap_or_else(|_| panic!("{} is not under {manifest_dir}", path.display()));
            let key = relative.to_string_lossy().replace('\\', "/");
            out.push((key, path.to_string_lossy().into_owned()));
        }
    }
}
