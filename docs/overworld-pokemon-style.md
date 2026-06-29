# Overworld — Pokemon-Style Visual Upgrade Plan

## Current state

```
ow-tile div   →  background-color (CSS flat)  +  emoji text content
ow-sprite div →  emoji text (🧑 👹 🧓)
```

## Target state

```
ow-tile div   →  background-image (pixel-art tileset spritesheet)
ow-sprite div →  <img> from assets/img/  or  character spritesheet
```

---

## Phase 1 — Tile backgrounds

**Goal:** replace flat CSS colors with pixel-art tiles.  
**Effort:** 1–2 h  |  **Files touched:** `assets/main.css`, `assets/img/`, `overworld.rs` (1 line)

### 1.1 Get a tileset PNG

One image containing all tile types on a grid. Each source tile is typically 16×16 px.

Free sources:
- [OpenGameArt.org](https://opengameart.org) — search "Pokemon tileset" or "RPG overworld tiles"
- [LPC compatible map tiles](https://opengameart.org/content/lpc-compatible-map-tiles)
- [itch.io](https://itch.io) — many free/paid pixel-art packs
- Tools for custom art: Aseprite, Pixilart, GIMP

→ Drop the final file at `assets/img/tileset_overworld.png`

### 1.2 Map each TileKind to a spritesheet position

Each tile is a `div` with `background-image` + `background-position`.  
If the source tiles are 16×16 and you scale 3× to 48×48:

```css
.ow-tile {
    width: 48px; height: 48px;
    background-image: url('/assets/img/tileset_overworld.png');
    background-repeat: no-repeat;
    /* example: 4 cols × 4 rows of 16px tiles scaled 3× */
    background-size: 192px 192px;
}

.ow-floor { background-position:   0px   0px; }  /* col 0, row 0 */
.ow-wall  { background-position: -48px   0px; }  /* col 1, row 0 */
.ow-grass { background-position:   0px -48px; }  /* col 0, row 1 */
.ow-water { background-position: -48px -48px; }  /* col 1, row 1 */
.ow-door  { background-position: -96px   0px; }  /* col 2, row 0 */
```

### 1.3 Remove emoji from tile divs

In `overworld.rs`, the tile loop currently outputs the emoji string.  
Change tile `div` bodies to empty — the tileset background does all the work:

```rust
// before
div { class: "{tile_css(tile_kind)}", "{tile_emoji_at(tile_kind, x, y, &ow.locked_doors)}" }

// after
div { class: "{tile_css(tile_kind)}" }
```

The door lock indicator (🔒) can become a CSS `::after` overlay on `.ow-door.ow-locked`.

---

## Phase 2 — Character sprites

**Goal:** replace emoji hero/NPC with actual pixel-art images.  
**Effort:** 2–3 h  |  **Files touched:** `overworld.rs`, `main.css`, `lib-rpg/overworld_manager.rs` (minor)

### 2.1 Hero sprite

Character PNGs already exist in `assets/img/` (Thalia.png, thrain.png, …).  
The hero `id_name` is available from `server_data`:

```rust
img {
    src: "/assets/img/{hero_id}.png",
    class: "ow-sprite ow-hero ow-sprite-img",
    style: "left:{pos.x * TILE_PX}px; top:{pos.y * TILE_PX}px; width:{TILE_PX}px; height:{TILE_PX}px;",
}
```

### 2.2 NPC sprites

Add an optional `sprite` field to `NpcJson` / `NpcState` in lib-rpg:

```rust
pub struct NpcState {
    pub id: String,
    pub pos: Position,
    pub dialog: Vec<String>,
    pub fight_scenario_id: Option<String>,
    #[serde(default)]
    pub sprite: Option<String>,   // ← new
}
```

Map JSON:

```json
{ "id": "gandalf", "x": 2, "y": 2, "sprite": "gandalf.png", "dialog": ["..."] }
```

Rendering with fallback:

```rust
let src = npc.sprite.as_deref().unwrap_or("default.png");
img { src: "/assets/img/{src}", class: "ow-sprite ow-npc ow-sprite-img" }
```

### 2.3 CSS for sprite images

```css
.ow-sprite-img {
    width: 80%;
    height: 80%;
    object-fit: contain;
    image-rendering: pixelated;
    pointer-events: none;
}
```

---

## Phase 3 — Tile variants / richer maps

**Goal:** wall corners, grass edges, path curves — the detail that makes Pokemon maps rich.

Pokemon uses **auto-tiling**: a single `wall` kind has 16+ visual variants depending on which
adjacent tiles are also walls (Wang/blob tiles).

### Options (easiest → hardest)

| Option | Description | Effort |
|--------|-------------|--------|
| **A — Manual variant** | Add `"variant": N` per tile in JSON; renders as a different `background-position` | Low — no lib-rpg change |
| **B — Computed auto-tiling** | At map-load time, compute a 4-bit neighbor mask per wall/grass tile and pick the right frame | Medium — pure dx-rpg logic |
| **C — Full Wang tile system** | Full 16-variant tileset with corner-aware selection | High |

**Recommendation: Option A** for a quick win — add a `#[serde(default)]` variant number
to `MapData` tile parsing; lib-rpg ignores it, dx-rpg uses it only for rendering.

---

## Phase 4 — Animations (optional polish)

| Feature | Implementation | Effort |
|---------|---------------|--------|
| Animated water | CSS `@keyframes` cycling through 2–3 `background-position` values on `.ow-water` | Very low |
| Hero walking cycle | 4-frame walk spritesheet; CSS animation class triggered on movement | Medium |
| Grass sway | CSS `@keyframes` subtle vertical shift on `.ow-grass` | Low |

The `transition: left 0.1s linear, top 0.1s linear` on `.ow-sprite` is already in place.

---

## Recommended implementation order

```
Phase 1 (tileset PNG + CSS + remove emoji)  →  highest visual impact, ~1–2 h
Phase 2 (character <img> sprites)           →  ~2–3 h, minor lib-rpg NpcState addition
Phase 3-A (variant field in JSON)           →  ~1 h code + time to rework map JSONs
Phase 4 (CSS animations)                   →  optional, pure CSS
```

### Files changed per phase

| Phase | dx-rpg | lib-rpg |
|-------|--------|---------|
| 1 | `assets/main.css`, `assets/img/tileset_overworld.png`, `overworld.rs` (1 line) | — |
| 2 | `overworld.rs`, `assets/main.css`, `offlines/maps/*.json` | `overworld_manager.rs` (add `sprite` field) |
| 3-A | `overworld.rs`, `offlines/maps/*.json` | optionally extend `NpcJson` |
| 4 | `assets/main.css` | — |
