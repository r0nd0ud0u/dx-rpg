# Iteration Plan

## Completed ✅ — Overworld iteration (2026-06-26)

### Overworld v1 — done
- **route_1.json** map created (Pokémon universe, tall-grass encounters, door back to Pallet Town)
- **lotr_shire.json** map created (LOTR universe, Goblin enemy NPC triggers stage-1 fight)
- **Emoji tiles**: 🧱 wall, 🚪 door, 💧 water; floor and grass stay plain
- **"⚔️ Back to Fight" button** in overworld HUD — sends `ExitOverworld` event
- **`ExitOverworld` client event** — server sets `game_phase = Running` and clears overworld state
- **Door spawn fix**: the `_spawn` from `MoveResult::MapTransition` is now passed through `PostAction::EnterMap(map_id, spawn)` and applied via `enter_overworld_at()`
- **Single-hero spawn**: `enter_overworld` now places only the first active hero to prevent ghost replicas
- **NPC fight trigger**: `InteractResult::Fight(scenario_id)` in lib-rpg; `fight_scenario_id` field on NPC JSON
- **`show_overworld` removed** from UI — `game_phase == Overworld` is the single source of truth; random grass encounters now immediately switch view to fight
- **lib-rpg rev** updated in Cargo.toml (local path while branch not pushed to GitHub)

### Next steps for overworld
- [ ] Push `prepare-overworld` branch to GitHub and restore `rev = "..."` in Cargo.toml
- [ ] Add map for LOTR stage 2 and beyond
- [ ] Player sprite differentiation per hero (different emoji per character)
- [ ] Multiplayer: each player's hero spawns independently when they enter the overworld
- [ ] Smooth CSS transitions on sprite movement
- [ ] Save/restore overworld position when re-entering (persist last known position per map)

---

## P0 — Engine correctness

### 1. MultiValue ×3 multiplier doesn't reach cross-character heals
**File:** `lib-rpg/src/character_mod/rounds_information.rs`
**Where:** `process_one_effect` (handles `MultiValue`)
**Problem:** The multiplier is stored in the launcher's `all_buffers`. `apply_buf_debuf` runs on the target's `character_rounds_info`, so the ×3 is silently ignored for ally heal effects in *Fleur de vie sanguinaire*.
**Fix:** In `process_one_effect`, when the effect kind is `MultiValue`, bake the multiplier directly into `ProcessedEffectParam.input_effect_param.buffer.value` (multiply `value` by the MultiValue factor). The target then receives the already-scaled value; `apply_buf_debuf` is not needed for this path.
**Test:** Add `unit_fleur_de_vie_sanguinaire_triple_heal` — simulate Elara dealing damage on turn N, then applying Fleur on turn N+1, assert ally HP increases by `25 × 3 = 75` per tick.

### 2. `build_hp_effect` uses `TARGET_ENNEMY` for consumables
**File:** `lib-rpg/src/character_mod/effect.rs`
**Where:** `build_hp_effect`
**Problem:** HP potions are created with `target_kind = TARGET_ENNEMY`. They work because `use_consumable` bypasses target resolution, but the semantics are wrong and could break if target filtering changes.
**Fix:** Change `build_hp_effect` to use `TARGET_ALLY` (or `TARGET_HIMSELF` for single-target potions).
**Test:** Existing `unit_all_catalog_consumables_work_during_fight` covers the functional behavior; just verify no test regressions.

---

## P1 — Balance

### 3. Fleur de vie sanguinaire — values to tune
Current: ally +25 HP/turn × 3 turns, ×3 if damage prev turn; enemy −35 HP/turn × 2 turns; 5-turn cooldown.
Review after P0.1 fix (the ×3 was never actually firing, so playtesting with real ×3 is needed).

### 4. Elara overall balance pass
Once P0.1 is fixed, run a simulated fight in tests to verify DPS/HPS balance.

---

## P2 — Quality

### 5. `EffectOutcome.real_amount_tx` — document semantics in code
The fix in `apply_processed_effect_param` uses `full_amount.min(apply_result)` for energy stats. Add a short inline comment explaining the `modify_stat_current` overhead semantics so the next reader doesn't have to re-derive it.

### 6. Log format for zero-delta consumable use
If a stat is already full, `real_amount_tx = 0` and the log says `"uses potion de mana"` without a delta. Consider adding "(already full)" to the message when delta is 0 and the stat is at max.

---

## Backlog

- Party potion from bag: verify behavior when multiple heroes use the same potion type in the same round.
- Resurrection potion: currently `apply_consumable_effects` is used directly (bypassing `use_consumable`), which skips inventory removal. Confirm this is intentional.

---

## i18n rollout

Infrastructure landed (2026-07-03): `dioxus-i18n` wired in `main.rs`, `CtxAppLang` (localStorage-backed) + navbar toggle, `navbar.rs` fully converted to `t!()`, and bilingual `description_en`/`description_fr` fields + resolver added to lib-rpg's `AttackType`/`Character` with Elara la guerisseuse de la Lorien migrated as the worked example.

**UI chrome rollout completed (2026-07-02):** every `board_game_components/*.rs` and `widgets/*.rs` file is now converted to `t!()` — navbar, all pages (home/lobby/login/create-server/join/load/start/character-select/character-page/gameboard/overworld), all admin tabs (users/attacks/characters/equipment/scenarios), and `game_sheets.rs` in full (menu/inventory/stats/logs/scenarios/store/settings sheets). `en-US.ftl`/`fr-FR.ftl` key parity is enforced by `unit_locale_files_have_matching_keys`. Deliberately left untranslated: dynamic backend-generated content (`lib_rpg` combat log text via `log_text()`/`log.message`, shop/equipment/consumable item `name`/`description` — game data, not UI chrome, same category as character bios below), enum-backed `value:` attributes on `<select>` options (kept as literal schema keywords), brand/logo strings, and `tracing::*!` log lines (server-side only, never rendered).

Remaining UI-chrome follow-up:
1. **`auth_manager/*` server functions** — audit which `ServerFnError` messages actually surface as UI toasts (vs. stay server-side/logs only) and translate only those.
2. New components/pages added after this pass should follow the same `t!()` convention from the start.

### Bilingual character data (`description_en`/`description_fr`)

~20+ characters remain on the legacy single-language `description` field (harmless — `description_for`/`effects_description_for` transparently fall back to it). Priority order, per `lib-rpg/README.md`'s "Hero Balance (LOTR roster)" documentation depth:
1. **Thraïn** — passive and attacks already documented in lib-rpg's README.
2. **Thalia**
3. **Azrak Ombresang** — his `OverHealBoostStat` passive is documented.
4. Remaining LOTR roster — enumerate via `find offlines/characters/lotr -name '*.json'` at the start of that session.
5. Pokémon-universe characters — confirm first whether their descriptions have the same language-mixing problem before assuming English.
6. `offlines/equipment/*.json` and scenario `description` fields — would need their own struct-field audit first (not covered by this pass's schema work).

Each future migration session should: confirm the existing text's actual language per file (don't assume — this pass verified Elara's text was English before translating), edit both `lib-rpg/offlines/` and `dx-rpg/offlines/` together (they must stay byte-identical), and no code changes are needed — the resolver/UI hookup is already generic.
