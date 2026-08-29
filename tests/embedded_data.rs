//! Verifies the build.rs-generated `offlines/` embedding actually round-trips through
//! `DataManager::try_new` end to end, producing sensible production data.
//!
//! Lives under `tests/` (a separate binary) deliberately: `lib_rpg::utils`'s
//! `EMBEDDED_FILES` is a process-global `OnceLock` — first caller wins, for the whole
//! process — so calling `register()` inside dx-rpg's own `src/`-internal unit test binary
//! would risk contaminating any other test in that same binary that assumes real-fs
//! behavior (same reasoning as lib-rpg's own `tests/embedded_data.rs`).
#![cfg(not(feature = "server"))]

use dx_rpg::{common::OFFLINE_PATH, embedded_data};
use lib_rpg::server::data_manager::DataManager;

#[test]
fn offline_path_loads_via_embedded_data_matching_production_counts() {
    embedded_data::register();

    let dm = DataManager::try_new(OFFLINE_PATH)
        .expect("DataManager::try_new(OFFLINE_PATH) over embedded offline data");

    // Same counts lib-rpg's own unit_try_new asserts against the real filesystem for
    // this exact production offlines/ tree: "lotr: 4 heroes + 8 bosses; pokemon: 3
    // heroes + 9 bosses".
    assert_eq!(dm.all_heroes.len(), 7, "4 lotr heroes + 3 pokemon heroes");
    assert!(dm.all_bosses.len() >= 2, "at least the original 2 bosses");
    assert!(!dm.all_scenarios.is_empty());
    assert!(!dm.equipment_table.is_empty());

    let universes = dm.list_hero_universes();
    assert!(universes.contains(&"lotr".to_owned()));
    assert!(universes.contains(&"pokemon".to_owned()));
}
