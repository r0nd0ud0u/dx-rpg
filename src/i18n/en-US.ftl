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
common-back = Back

quit-dialog-title = Quit Game
quit-dialog-body = Are you sure you want to quit the game?

navbar-connection-connected = Connected
navbar-connection-reconnecting = Reconnecting…
navbar-connection-latency = Connected — { $ms } ms

navbar-debug-console = 🐞 Debug
debug-console-title = Debug Console
debug-console-empty = No logs captured yet.
debug-console-clear = Clear

navbar-server-settings = 🌐 Server
navbar-fullscreen-toggle = Fullscreen
server-settings-title = Server Connection
server-settings-current = Currently connected to: { $url }
server-settings-placeholder = https://your-server.example.com
server-settings-insecure-label = Accept invalid certificates (insecure)
server-settings-insecure-warning = Disables TLS certificate validation for this server. Only use this against a server and network you trust.
server-settings-save = Save
server-settings-saved = Saved — restart the app for this to take effect.

sound-settings-title = Sound Settings
sound-settings-muted = Mute all sound
sound-settings-background = Keep music playing in the background
sound-settings-music-volume = Music volume
sound-settings-sfx-volume = Sound effects volume

navbar-change-password = 🔑 Password
change-password-title = Change Password
change-password-current-placeholder = Current password
change-password-new-placeholder = New password
change-password-confirm-placeholder = Confirm new password
change-password-save = Save
change-password-saved = Password changed successfully.
change-password-mismatch = New passwords don't match.
change-password-empty = New password can't be empty.

## how-to-play dialog (board_game_components/tutorial.rs)
help-title = 📖 How to play
help-intro = First time here? Do the three steps. Coming back? Jump to Pro tips.
help-first-run-hint = You can reopen this any time with the ❓ button in the top bar.

help-tab-basics = 🚀 First steps
help-tab-combat = ⚔️ Combat
help-tab-world = 🗺️ Overworld
help-tab-gear = 🎒 Gear & gold
help-tab-progress = 🏆 Progression
help-tab-pro = 💡 Pro tips
help-tab-admin = 🛡️ Admin

help-basics-1-title = Start an adventure
help-basics-1-body = Sign in, then Create Server — pick a universe (LOTR or Pokémon), Single Player or Multiplayer, and a save slot. Or Join Game to drop into a run someone else is hosting.
help-basics-2-title = Pick your hero
help-basics-2-body = In the lobby, click a card to claim that hero. Details shows their class, level, HP and full stats before you commit — a card marked 🔒 is already taken.
help-basics-3-title = Fight, turn by turn
help-basics-3-body = When your hero's card wakes up, press ⚔️, choose an attack, choose a target, confirm. That is the whole loop.
help-basics-host-note = ▶ Start Game belongs to the host, once everyone has chosen. In Single Player you take several heroes and play them all — click a claimed card again to release it.
help-basics-offline-title = ✈️ In a hurry?
help-basics-offline-body = Play Offline on the login page: no account, no server, a solo run one click away.
help-basics-legend-title = What the icons mean
help-legend-attack = ⚔️ — open your attack list (on your turn)
help-legend-potion = 💊 — drink a potion; it costs your action
help-legend-aggro = 🎯 — aggro: the threat you built over the last 5 turns
help-legend-extra = ⚡×1 — a bonus round earned with Speed
help-legend-taken = 🔒 — hero already taken by another player
help-legend-badge = Red dot — unspent talent points or new gear waiting

help-combat-flow-title = Your turn
help-combat-flow-body = ⚔️ opens your attack list, each attack with its energy cost. Pick one, click the target buttons, then confirm with Launch Attack.
help-combat-potion-title = Potions
help-combat-potion-body = 💊 lists your own potions and the party's shared ones. Drinking is your action for the turn — heal one hit before you need it, not one hit too late.
help-combat-energy-title = Energy
help-combat-energy-body = Attacks spend and refund 🔮 Mana, 💪 Vigor and 😡 Berserk. A turn on a cheap attack is a turn spent recharging the expensive one.
help-combat-order-title = Turn order
help-combat-order-body = Speed sets the order — fastest first, all heroes then all bosses. On each side the fastest character above 100 Speed also earns a bonus round: the ⚡ badge on their card.
help-combat-crit-title = Crits, dodges, blocks
help-combat-crit-body = A critical hit doubles the damage. Most heroes dodge; Berserkers block instead. Unlucky streaks are broken on purpose, so a dry run always ends.
help-combat-read-title = Read the board
help-combat-read-body = Logs replays every hit, heal and effect. Game Stats tracks damage, healing, kills and scenario progress. Settings decides whether boss HP and energy bars are visible at all.

help-world-intro = Between fights the host can open 🗺 Explore Overworld — a tile map you walk across to find the next boss.
help-world-move-label = Move
help-world-move-keys = Arrow keys, or the on-screen D-pad
help-world-interact-label = Interact
help-world-interact-keys = Enter or Space, standing next to an NPC
help-world-zoom-label = Zoom
help-world-zoom-keys = − / + in the map corner; your setting is remembered
help-world-grass = Tall grass hides random encounters — about one step in two starts a fight.
help-world-boss = Talk to a boss NPC to trigger its pre-fight dialog, then confirm to begin the battle.
help-world-door = Beat that boss and the next door unlocks; the NPC leaves the map for good.
help-world-back = ⚔️ Back to Fight returns to the fight in progress at any time.

help-gear-store-title = The Store
help-gear-store-body = It opens on the end-of-scenario screen via 🛒 Shop — or at any moment during a scenario once you enable Shop During Scenario in Settings.
help-gear-tabs-title = Shop and Bag
help-gear-tabs-body = Shop sells equipment (weapons, armour, rings and more) and consumables — HP, Mana, Vigor, Berserk and Resurrection potions. Bag holds what you own and buys it back at half price, so spend on purpose.
help-gear-equip-title = Equipping
help-gear-equip-body = Bought gear lands in your bag; open Inventory to equip it and watch the stats move. A dot on the toolbar means something new is waiting there.
help-gear-gold-title = Gold
help-gear-gold-body = Gold drops as loot at the end of each scenario, and every hero keeps their own purse — plus the party loot everyone can reach.

help-progress-scenario-title = End of a scenario
help-progress-scenario-body = Loot and experience are shared out, heroes level up and new attacks unlock — the summary screen lists exactly what changed.
help-progress-talents-title = Talents
help-progress-talents-body = Every level grants a skill point, plus a bonus one every fifth level. Spend them in the Talents tree; Respec hands them all back if you want another path.
help-progress-cap-title = Level 13
help-progress-cap-body = 13 is the cap — and a level-13 ultimate attack can never be dodged or blocked.
help-progress-stages-title = Ten stages
help-progress-stages-body = Each universe runs ten scenarios. The host loads the next one; Scenarios shows what is done, in progress and still locked.
help-progress-save-title = Saving
help-progress-save-body = Three save slots per player. Save writes to the current one, Auto-save on Scenario does it for you at each new stage, and Load Game picks a run back up days later.

help-pro-title = Things veterans do
help-pro-speed = Speed above 100 buys a bonus round — one per side, per turn. Gear that adds Speed often beats gear that adds damage.
help-pro-panel = ⚙ Configure on the attack panel lets you drag your attacks into the order you actually use them. Saved per hero, per server.
help-pro-aggro = Turn on Hero Aggro in Settings to see, in one glance, which hero has been pushing the fight for the last five turns.
help-pro-ultimate = Hold the ultimate for a boss: nothing dodges it, nothing blocks it.
help-pro-berserker = Berserkers block instead of dodging and break a crit drought after three turns — aggression is their defence.
help-pro-mystery = Hide the boss HP bar for a tense fight; show boss energy bars instead and you can see the big attack coming.
help-pro-save = Save before a boss. If the whole party falls, the run is over.
help-pro-solo = Use a solo run to test a team composition before taking it into multiplayer.

help-admin-intro = Signed in on the admin account, the 🛡️ Panel link appears in the top bar.
help-admin-users = Users — accounts and who is connected right now.
help-admin-characters = Characters — every hero and boss, by universe.
help-admin-scenarios = Scenarios — add, edit or delete scenarios in the inline JSON editor.
help-admin-content = Attacks & Equipment — tune costs, damage, effects and prices.

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
admin-panel-access-denied = You don't have access to the admin panel.
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
loadgame-fetch-error = Failed to load saves: { $error }
loadgame-slot-scenario = 📜 { $scenario } (Lvl { $level })
loadgame-mode-solo = 🎮 Solo
loadgame-mode-multi = 👥 Multi ({ $players }p)
loadgame-universe = 🌐 { $universe }

## login page
login-sign-in-title = Sign In
login-empty-username = Please enter a username.
login-empty-fields = Please enter a username and password.
login-username-placeholder = Your username
login-password-placeholder = Password
login-success = { $username } logged in
login-session-expired = Your session has expired. Please sign in again.
login-sign-in-button = Sign In →
login-create-account-title = Create Account
login-choose-username-placeholder = Choose a username
login-choose-password-placeholder = Choose a password
login-invalid-login = Invalid login
login-name-taken = This name is already taken.
login-sign-up-button = Sign Up →
login-offline-title = ✈️ Play Offline
login-offline-hint = No account, no server — pick a universe and start a local solo game.
login-offline-start-button = Play Offline
login-offline-choose-universe-label = Choose a universe
login-offline-choose-universe-option = -- Choose a universe --

## create server page
create-server-title = 🏰 Create a Game
create-server-step1 = 1️⃣ Game Mode
create-server-multiplayer = 👥 Multiplayer
create-server-singleplayer = 🎮 Single Player
create-server-singleplayer-hint = One player controls all heroes.
create-server-multiplayer-hint = Each connected player picks one hero.
create-server-step2 = 2️⃣ Choose a Save Slot
create-server-empty-slot = Empty Slot { $index }
create-server-continue-play = ▶ Continue
create-server-overwrite-play = 🗑 Overwrite & Play
create-server-overwrite-confirm-title = Overwrite this save?
create-server-overwrite-confirm-body = This will permanently delete this saved adventure and start a new one in its place. This cannot be undone.

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
char-card-details = ℹ️ Details

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
startgame-explore-overworld = 🗺 Explore Overworld
startgame-loots = 🎁 Loots
startgame-no-loots = No loot this time.
startgame-level-upgrades = ⬆️ Level Upgrades
startgame-level-up-change = ⬆️ { $old } → { $new }
startgame-level-unchanged = 🟰 Lvl { $level }
startgame-new-attacks = ✨ New attacks unlocked:
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
character-page-sort-by-level = ⇅ Lvl
character-page-sort-by-cost = ⇅ Cost
character-page-configure-atk-panel = ⚙ Configure
character-page-configure-atk-panel-title = Configure attack panel
character-page-configure-atk-panel-desc = Drag and drop to reorder your attacks. This layout is saved for this character on this server.
character-page-cancel = Cancel
character-page-reset-atk-panel = Reset to default
character-page-save-atk-panel = Save

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
gs-inv-use-items-title = 💊 Use Items
gs-inv-no-consumables = No potions or items to use.
gs-inv-personal-potions = 💊 Personal potions
gs-inv-party-potions = 🎁 Common consumables
gs-inv-use = Use
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
