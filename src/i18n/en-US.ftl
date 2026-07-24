navbar-admin-panel = 🛡️ Panel
navbar-quit-game = Quit game
navbar-sign-in = Sign in
navbar-sign-out = Sign out
navbar-menu-title = Options
navbar-menu-open = Open options
lang-select-label = Language

common-close = Close
common-cancel = Cancel
common-confirm = Confirm

quit-dialog-title = Quit Game
quit-dialog-body = Are you sure you want to quit the game?

navbar-server-settings = 🌐 Server
server-settings-title = Server Connection
server-settings-current = Currently connected to: { $url }
server-settings-placeholder = https://your-server.example.com
server-settings-insecure-label = Accept invalid certificates (insecure)
server-settings-insecure-warning = Disables TLS certificate validation for this server. Only use this against a server and network you trust.
server-settings-save = Save
server-settings-saved = Saved — restart the app for this to take effect.

navbar-change-password = 🔑 Password
change-password-title = Change Password
change-password-current-placeholder = Current password
change-password-new-placeholder = New password
change-password-confirm-placeholder = Confirm new password
change-password-save = Save
change-password-saved = Password changed successfully.
change-password-mismatch = New passwords don't match.
change-password-empty = New password can't be empty.

help-title = How to play

help-section-getting-started = 🚀 Getting started
help-step-1 = 1. 🔐 Sign in or create an account on the login page.
help-step-2 = 2. 🌍 Choose a universe (LOTR or Pokémon) when creating a server.
help-step-3 = 3. 🎮 From the home page, create a new game or join an ongoing one.

help-section-game-modes = 🕹️ Game modes
help-mode-multiplayer = • Multiplayer — each connected player picks exactly one hero; other cards are locked 🔒 for other players.
help-mode-singleplayer = • Single-player — one player picks multiple heroes and controls them all in battle; click a selected card again to deselect it.

help-section-lobby = 🧙 Lobby & character selection
help-step-4 = 4. Select your character card in the lobby. Wait for all players to be ready.
help-step-5 = 5. ▶️ The host clicks 'Start Game' once everyone has chosen.

help-section-combat = ⚔️ Combat
help-step-6 = 6. On your turn, click ⚔️ on your character card to open the attack list, then pick an attack.
help-step-7 = 7. 🎯 Click target buttons to select your target(s), then confirm with '⚔️ Launch Attack'.
help-step-8 = 8. 💊 Click 💊 on your character card to use a potion (counts as your turn action).

help-section-toolbar = 🛠️ Game toolbar
help-step-9 = 9. 📦 Inventory — view your hero's stats and equipment.
help-step-10 = 10. 📊 Stats — track damage dealt, healing done, kill count, and scenario progress bar.
help-step-11 = 11. 📜 Scenarios — side sheet listing all stages with their completion status (Not Started / In Progress / ✅ Done).
help-step-12 = 12. ⚙️ Settings — toggle 'Attack Tooltips' to show/hide attack descriptions on hover.

help-section-overworld = 🗺️ Overworld exploration
help-step-13 = 13. The host clicks '🗺 Overworld' to enter the tile-map exploration mode.
help-overworld-move = • Arrow keys / D-pad — move your hero.
help-overworld-interact = • Enter or Space — interact with adjacent NPCs.
help-overworld-encounter = • Walking on grass may trigger a random encounter (50 % chance per step).
help-overworld-boss = • Interact with a boss NPC to start its pre-fight dialog, then confirm to begin the fight.
help-overworld-unlock = • Defeating a boss NPC unlocks the next door and removes the NPC from the map.
help-overworld-back = • '⚔️ Back to Fight' returns to the active fight at any time.

help-section-store = 🛒 Store
help-step-14 = 14. The store opens between scenarios (end-of-scenario screen) via the '🛒 Shop' button.
help-store-equipment = • Equipment tab — weapons, armour, rings and more; bought items go to your Bag.
help-store-consumables = • Consumables tab — potions (HP / Mana / Vigor / Berserk / Resurrection).
help-store-bag = • Bag tab — sell unequipped items for 50 % of their price; equip them from the Inventory sheet.
help-store-gold = • Gold is earned as loot at the end of each scenario.

help-section-progression = 🏆 Progression
help-step-15 = 15. At the end of a scenario the host loads the next stage.
help-step-16 = 16. Each universe has 10 progressive stages. Complete them all to win!
help-step-17 = 17. Save slots (up to 3 by default) let you continue a run later from the Load Game page.
help-step-18 = 18. ⚙️ Settings — toggle auto-save, attack tooltips, boss HP / energy bars, and hero aggro display.

help-section-admin = 🛡️ Admin panel
help-step-19 = 19. If you are an admin, access the 🛡️ Panel link in the navbar.
help-admin-users = • Users tab: manage accounts and connection status.
help-admin-characters = • Characters tab: browse all heroes and bosses by universe.
help-admin-scenarios = • Scenarios tab: add, edit or delete scenarios via inline JSON editor.

footer-about = About
footer-lib-rpg-engine = lib-rpg engine
footer-built-with-dioxus = Built with Dioxus
footer-contact = Contact
footer-report-issue = Report an issue
footer-discussions = Discussions

## join-ongoing-game page
join-ongoing-title = 🗺️ Ongoing Adventures
join-ongoing-empty = No games running yet. Create one!

## home page
home-title = ⚔️ RPG Adventure
home-welcome = Welcome, { $user_name }!
home-create-server = Create Server
home-create-server-desc = Start a new adventure as host
home-join-game = Join Game
home-join-game-desc = Join an ongoing adventure

## admin page shell
admin-panel-title = 🛡️ Admin Panel
admin-panel-disabled = The admin panel is disabled.
admin-tab-users = 👤 Users
admin-tab-scenarios = 📜 Scenarios
admin-tab-characters = 🧙 Characters
admin-tab-equipment = 🔧 Equipment

## common (reused across many pages)
common-loading = Loading…

## admin users tab
admin-users-title = 📋 All Users
admin-users-empty = No users found.
admin-users-col-username = Username
admin-users-col-connected = Connected
admin-users-col-saves = Saves
admin-users-delete-title = 🗑️ Delete User
admin-users-delete-label = Username to delete
admin-users-delete-placeholder = Enter username…
admin-users-delete-button = Delete User
admin-users-delete-success = ✅ User deleted.
admin-users-delete-error = ❌ This name cannot be deleted.

## load game page
loadgame-title = 💾 Load Game
loadgame-fetch-error = Failed to load saves: { $error }
loadgame-count-one = { $count } saved adventure
loadgame-count-other = { $count } saved adventures
loadgame-empty = No saved games found.
loadgame-empty-hint = Create a new game first.
loadgame-slot-scenario = 📜 { $scenario } (Lvl { $level })
loadgame-mode-solo = 🎮 Solo
loadgame-mode-multi = 👥 Multi ({ $players }p)
loadgame-universe = 🌐 { $universe }
loadgame-load-button = ▶ Load
loadgame-delete-button = 🗑 Delete

## login page
login-sign-in-title = Sign In
login-empty-username = Please enter a username.
login-empty-fields = Please enter a username and password.
login-username-placeholder = Your username
login-password-placeholder = Password
login-success = { $username } logged in
login-sign-in-button = Sign In →
login-create-account-title = Create Account
login-choose-username-placeholder = Choose a username
login-choose-password-placeholder = Choose a password
login-invalid-login = Invalid login
login-name-taken = This name is already taken.
login-sign-up-button = Sign Up →

## create server page
create-server-title = 🏰 Create a Game
create-server-step1 = 1️⃣ Game Mode
create-server-multiplayer = 👥 Multiplayer
create-server-singleplayer = 🎮 Single Player
create-server-singleplayer-hint = One player controls all heroes.
create-server-multiplayer-hint = Each connected player picks one hero.
create-server-step2 = 2️⃣ Choose a Save Slot
create-server-empty-slot = Empty Slot { $index }
create-server-overwrite-play = ▶ Overwrite & Play
create-server-load-game = Load Game
create-server-load-game-desc = Continue a saved adventure

## lobby page
lobby-title = ⚔️ Lobby
lobby-loading = ⏳ Loading…
lobby-server-label = Server
lobby-players-label = Players
lobby-universe-label = Universe
lobby-scenarios-label = Scenarios
lobby-start-game = ▶ Start Game
lobby-universe-saved-label = Universe (saved)
lobby-universe-locked = 🔒 { $universe }
lobby-choose-universe-label = Choose Universe
lobby-select-universe-option = — select a universe —
lobby-not-enough-players = Not enough players
lobby-game-ended = No more game, back to home

## equipment tab widget
equip-count-equipped = { $count } equipped
equip-new-item = New item!
equip-section-equipped = ✅ Equipped
equip-section-in-bag = 🎒 In bag
equip-empty-slot = No item in this slot.
equip-not-found = Equipment not found
equip-new-dot-title = New!
equip-no-stat-bonuses = No stat bonuses.
equip-click-to-unequip = Click to unequip
equip-click-to-equip = Click to equip

## common (reused across many pages)
common-level = Lv { $level }

## character select
char-select-title-single = 🎮 Single Player — Choose Your Heroes
char-select-title-multi = 👥 Choose Your Character
char-select-other-players = Other players:
char-card-taken-by = 🔒 { $taker }
char-card-remove = × Remove
char-card-select = + Select

## charts widget
charts-no-attacks = No attacks recorded yet.
charts-attack-frequency = ⚔️ Attack Frequency
charts-no-dmg-heal = No damage or heal data yet.
charts-damage-dealt = 🗡️ Damage dealt
charts-healing-done = 💚 Healing done
charts-suffix-dmg = dmg
charts-suffix-hp = hp
charts-total-damage = Total Damage
charts-total-heal = Total Heal
charts-dmg-per-round = Dmg / Round
charts-heal-per-round = Heal / Round
charts-attacks-cast = Attacks cast
charts-favourite = Favourite
charts-party-tab = 🌐 Party

## startgame / running game page
startgame-lvl = Lvl { $level }
startgame-defeated = 💀 Defeated
startgame-quit = 🚪 Quit
startgame-game-over = 💀 Game Over
startgame-remaining-players = Remaining players: { $count }
startgame-replay-game = 🔄 Replay Game
startgame-scenario-complete = 🏆 Scenario Complete!
startgame-finishing-blow-dot = ⚔️ Finishing Blow (DOT)
startgame-finishing-blow = ⚔️ Finishing Blow
startgame-enemy-last-attack = Enemy's last attack: { $name }
startgame-shop = 🛒 Shop
startgame-load-next-scenario = ⚡ Load Next Scenario
startgame-explore-overworld = 🗺 Explore Overworld
startgame-loots = 🎁 Loots
startgame-level-upgrades = ⬆️ Level Upgrades
startgame-turn-round = ⚔️ Turn { $turn } - Round { $round }
startgame-run-away = 🗺 Run away

## gameboard
gameboard-spectator-mode = 👁 Spectator mode — you have no active character in this game
gameboard-use = ✅ Use
gameboard-launch-attack = ⚔️ Launch Attack
gameboard-attacks = ⚔️ { $launcher } attacks!
gameboard-turn-round = 🔄 Turn { $turn } — Round { $round }
gameboard-last-attack = Last attack: { $name }
gameboard-critical-strike = 💥 Critical Strike!
gameboard-is-dodging = { $name } is dodging
gameboard-is-blocking = { $name } is blocking

## overworld
overworld-entering = 🗺 Entering overworld…
overworld-start-fight-question = Do you want to start the fight?
overworld-yes-fight = ⚔️ Yes, fight!
overworld-no-not-yet = 🚪 No, not yet
overworld-controls-hint = Arrows: move  |  Enter/⚔: interact

## common admin error/status
admin-error = ❌ { $error }
admin-deleted = ✅ Deleted.

## admin equipment tab
admin-equip-browser-title = 🔧 Equipment Browser
admin-equip-select-type = — type —
admin-equip-select-category = — category —
admin-equip-items-title = 📋 Items — { $category }
admin-equip-cancel-new = ✕ Cancel
admin-equip-new = ➕ New
admin-equip-new-item-in = ➕ New Item in { $category }
admin-equip-filename-label = Item filename (no spaces, no extension)
admin-equip-filename-placeholder = e.g. epic_sword
admin-equip-name-empty = ❌ Name cannot be empty.
admin-equip-created = ✅ '{ $name }' created.
admin-equip-create = 💾 Create
admin-equip-edit-title = ✏️ { $name }
admin-equip-json-mode = ✏️ JSON mode
admin-equip-form-mode = 📝 Form mode
admin-equip-name-label = Name
admin-equip-unique-name-label = Unique name
admin-equip-category-label = Category
admin-equip-stats-title = Stats
admin-equip-stat-col = Stat
admin-equip-value-col = Value
admin-equip-save = 💾 Save
admin-equip-saved = ✅ Saved.

## admin scenarios tab
admin-scenarios-select-universe = 🌐 Select Universe
admin-scenarios-choose-universe = — choose a universe —
admin-scenarios-title = 📜 Scenarios — { $universe }
admin-scenarios-empty = No scenarios found for this universe.
admin-scenarios-col-level = Lvl
admin-scenarios-col-bosses = Bosses
admin-scenarios-col-description = Description
admin-scenarios-col-file = File
admin-scenarios-col-actions = Actions
admin-scenarios-edit = ✏️ Edit
admin-scenarios-confirm-delete = ⚠️ Confirm
admin-scenarios-delete = 🗑️ Delete
admin-scenarios-add = ➕ Add Scenario
admin-scenarios-new-title = ➕ New Scenario
admin-scenarios-edit-title = ✏️ Edit Scenario
admin-scenarios-file-stem-label = File stem (e.g. stage_11)
admin-scenarios-file-stem-placeholder = stage_11
admin-scenarios-name-placeholder = Scenario name
admin-scenarios-description-placeholder = Describe the scenario…
admin-scenarios-level-label = Level
admin-scenarios-bosses-label = Bosses (one per line — "BossName" or "BossName: 0, 1, 2")
admin-scenarios-loots-label = Loots
admin-scenarios-loot-level-placeholder = Lvl
admin-scenarios-loot-classes-placeholder = Classes (Standard, Warrior…)
admin-scenarios-remove-loot = ✕
admin-scenarios-add-loot = ＋ Add Loot
admin-scenarios-file-stem-empty = ❌ File stem cannot be empty.

## loot kind / rank labels (shared across admin tabs)
loot-kind-equipment = Equipment
loot-kind-consumable = Consumable
loot-kind-material = Material
loot-kind-currency = Currency
rank-common = Common
rank-intermediate = Intermediate
rank-advanced = Advanced

## admin attacks tab
admin-atk-upload-error = ❌ Upload: { $error }
admin-atk-title = ⚔️ Attacks — { $character }
admin-atk-empty = No attacks found.
admin-atk-form-edit-title = 📝 { $name }
admin-atk-level-label = Level
admin-atk-target-label = Target
admin-atk-reach-label = Reach
admin-atk-form-label = Form
admin-atk-photo-label = Photo
admin-atk-photo-placeholder = e.g. Fireball.png
admin-atk-upload-photo-label = Upload Photo
admin-atk-cost-mana-label = Mana Cost
admin-atk-cost-rage-label = Rage Cost
admin-atk-cost-vigor-label = Vigor Cost
admin-atk-duration-label = Duration
admin-atk-aggro-label = Aggro
admin-atk-damage-label = Damage
admin-atk-heal-label = Heal
admin-atk-regen-mana-label = Mana Regen
admin-atk-regen-rage-label = Rage Regen
admin-atk-regen-vigor-label = Vigor Regen
admin-atk-effects-label = Effects (JSON array)
admin-atk-target-enemy = Enemy
admin-atk-target-ally = Ally
admin-atk-target-self = Self
admin-atk-target-zone = Zone
admin-atk-target-all = All
admin-atk-reach-individual = Individual
admin-atk-reach-area = Area
admin-atk-reach-all = All
admin-atk-form-standard = Standard
admin-atk-form-magic = Magic
admin-atk-form-healing = Healing
admin-atk-form-support = Support
admin-atk-new-title = ➕ New Attack
admin-atk-new-placeholder = Attack file name (e.g. Fireball)
admin-atk-created = ✅ Created.
admin-atk-create-button = Create

## character page (attack tooltip / potion list)
character-page-effects-label = Effects
character-page-no-potions = No potions available
character-page-lvl = Lvl { $level }
character-page-extra-round-title = Extra round from speed advantage
character-page-aggro-title = Aggro

## admin characters tab
admin-chars-filter-universe = 🌐 Filter by Universe
admin-chars-all-universes = — all universes —
admin-chars-create-universe-title = 🌍 Create Universe
admin-chars-universe-name-placeholder = Universe name (e.g. pokemon)
admin-chars-universe-created = ✅ Universe created.
admin-chars-heroes = 🧙 Heroes
admin-chars-bosses = 👹 Bosses
admin-chars-none-found = No { $kind } found.
admin-chars-form-button = 📝 Form
admin-chars-json-button = ✏️ JSON
admin-chars-attacks-button = ⚔️ Attacks
admin-chars-form-title = 📝 Form: { $name }
admin-chars-name-placeholder = Character name
admin-chars-short-name-label = Short name
admin-chars-class-label = Class
admin-chars-rank-label = Rank
admin-chars-type-label = Type
admin-chars-type-hero = Hero
admin-chars-type-boss = Boss
admin-chars-photo-filename-label = Photo filename
admin-chars-photo-placeholder = e.g. Thalia.png
admin-chars-color-label = Color
admin-chars-color-placeholder = e.g. green
admin-chars-max-actions-label = Max actions / round
admin-chars-description-placeholder = Character description…
admin-chars-energies-label = Energies
admin-chars-blocking-atk-label = Is blocking attack
admin-chars-current-col = Current
admin-chars-max-col = Max
admin-chars-json-title = ✏️ JSON: { $name }

## character class labels (shared)
class-warrior = Warrior
class-mage = Mage
class-healer = Healer
class-berserker = Berserker

## energy labels (shared)
energy-mana = Mana
energy-rage = Rage
energy-vigor = Vigor

## game_sheets.rs
gs-save = Save
gs-menu = Menu
gs-inventory = Inventory
gs-talents = Talents
gs-talents-unspent-points = Unspent skill points!
gs-new-equipment = New equipment!
gs-new-equipment-for = New equipment for { $name }!
gs-logs = Logs
gs-game-stats = Game Stats
gs-scenarios = Scenarios
gs-settings = Settings
gs-store-disabled-hint = Store is disabled in Settings
gs-store = Store
gs-close = Close
gs-inv-title = 🎒 Inventory — { $name }
gs-inv-desc = Level { $level } equipment overview
gs-talents-title = 🌳 Talents — { $name }
gs-talents-desc = Level { $level } talent tree
talents-points-available = Skill points available: { $points }
talents-respec = Respec
talents-no-tree = No talents available for this hero yet.
talents-cost-label = Cost: { $cost } point(s)
talents-locked-requires = Requires: { $name }
talents-locked-capstone = Another path's capstone ({ $name }) is already active — respec first
talents-locked-points = Not enough skill points
gs-stats-title = 📊 Game Stats
gs-stats-desc = Overview of the current session.
gs-turn-label = Turn
gs-round-label = Round
gs-kills-label = Kills
gs-stats-active-player-label = Active Player
gs-scenario-progress-title = Scenario Progress
gs-scenarios-completed = { $completed } / { $total } completed
gs-heroes-status-title = Heroes Status
gs-menu-title = Menu
gs-menu-desc = Game session controls.
gs-toolbar-title = Game Menu
gs-toolbar-open = Open game menu
gs-menu-server-label = Server
gs-menu-active-player-label = Active Player
gs-menu-players-label = Players
gs-game-saved = Game saved!
gs-logs-title = Logs
gs-logs-desc = History of all game events.
gs-logs-all = All
gs-logs-combat = ⚔ Combat
gs-logs-healing = 💚 Healing
gs-logs-events = ℹ Events
gs-logs-empty = No logs yet.
gs-scenarios-sheet-title = 📜 Scenarios
gs-scenarios-sheet-desc = Progress through all available stages.
gs-scenarios-empty = No scenarios loaded.
gs-scenario-completed = ✅ Completed
gs-scenario-in-progress = ⚔️ In Progress
gs-scenario-not-started = 🔒 Not Started
gs-store-title = 🛒 Store — { $name }
gs-gold-amount = 💰 { $amount } gold
gs-store-shop = 🏪 Shop
gs-store-bag = 🎒 Bag
gs-store-equipment = ⚔️ Equipment
gs-store-consumables = 💊 Consumables
gs-store-slot = Slot: { $category }
gs-store-in-bag = (×{ $count } in bag)
gs-store-buy = Buy
gs-store-no-gold = No gold
gs-store-bag-empty = Your bag is empty.
gs-store-party-loot = 🎒 Party loot
gs-store-sell = Sell
gs-settings-title = ⚙️ Settings
gs-settings-desc = Personalise your game experience.
gs-settings-tooltips-label = Attack Tooltips
gs-settings-tooltips-hint = Show attack description on hover in the attack list.
gs-settings-boss-energy-label = Boss Energy Bars
gs-settings-boss-energy-hint = Show mana/vigor/berserk bars for bosses.
gs-settings-hero-aggro-label = Hero Aggro
gs-settings-hero-aggro-hint = Show aggro value on the hero panel header.
gs-settings-boss-hp-label = Boss HP Bar
gs-settings-boss-hp-hint = Show the HP bar on boss panels (hidden if you want mystery).
gs-settings-autosave-label = Auto-save on Scenario
gs-settings-autosave-hint = Automatically save at the start of each new scenario.
gs-settings-shop-label = Shop During Scenario
gs-settings-shop-hint = Allow opening the Store during an active scenario.
gs-settings-saving = Saving…
gs-settings-saved = ✅ Saved

## popover_comp.rs (unreferenced demo component)
popover-demo-trigger = Show Popover
popover-demo-title = Delete Item?
popover-demo-confirmed = Item deleted!
