navbar-admin-panel = 🛡️ Panneau
navbar-quit-game = Quitter la partie
navbar-sign-in = Se connecter
navbar-sign-out = Se déconnecter
navbar-menu-title = Options
navbar-menu-open = Ouvrir les options
lang-select-label = Langue

common-close = Fermer
common-cancel = Annuler
common-confirm = Confirmer
common-back = Retour

quit-dialog-title = Quitter la partie
quit-dialog-body = Êtes-vous sûr de vouloir quitter la partie ?

navbar-connection-connected = Connecté
navbar-connection-reconnecting = Reconnexion…
navbar-connection-latency = Connecté — { $ms } ms

navbar-debug-console = 🐞 Débogage
debug-console-title = Console de débogage
debug-console-empty = Aucun journal capturé pour le moment.
debug-console-clear = Effacer

navbar-server-settings = 🌐 Serveur
navbar-fullscreen-toggle = Plein écran
server-settings-title = Connexion au serveur
server-settings-current = Actuellement connecté à : { $url }
server-settings-placeholder = https://votre-serveur.exemple.com
server-settings-insecure-label = Accepter les certificats invalides (non sécurisé)
server-settings-insecure-warning = Désactive la validation du certificat TLS pour ce serveur. À n'utiliser qu'avec un serveur et un réseau de confiance.
server-settings-save = Enregistrer
server-settings-saved = Enregistré — redémarrez l'application pour appliquer ce changement.

sound-settings-title = Réglages sonores
sound-settings-muted = Couper tous les sons
sound-settings-background = Continuer la musique en arrière-plan
sound-settings-music-volume = Volume de la musique
sound-settings-sfx-volume = Volume des effets sonores

navbar-change-password = 🔑 Mot de passe
change-password-title = Changer le mot de passe
change-password-current-placeholder = Mot de passe actuel
change-password-new-placeholder = Nouveau mot de passe
change-password-confirm-placeholder = Confirmer le nouveau mot de passe
change-password-save = Enregistrer
change-password-saved = Mot de passe changé avec succès.
change-password-mismatch = Les nouveaux mots de passe ne correspondent pas.
change-password-empty = Le nouveau mot de passe ne peut pas être vide.

## fenêtre « Comment jouer » (board_game_components/tutorial.rs)
help-title = 📖 Comment jouer
help-intro = Première partie ? Suivez les trois étapes. De retour ? Filez aux Astuces.
help-first-run-hint = Vous pouvez rouvrir cette aide à tout moment avec le bouton ❓ de la barre du haut.

help-tab-basics = 🚀 Premiers pas
help-tab-combat = ⚔️ Combat
help-tab-world = 🗺️ Exploration
help-tab-gear = 🎒 Équipement & or
help-tab-progress = 🏆 Progression
help-tab-pro = 💡 Astuces
help-tab-admin = 🛡️ Admin

help-basics-1-title = Lancez une aventure
help-basics-1-body = Connectez-vous, puis Créer un serveur — choisissez un univers (LOTR ou Pokémon), Solo ou Multijoueur, et un emplacement de sauvegarde. Ou Rejoindre une partie pour entrer dans celle d'un autre joueur.
help-basics-2-title = Choisissez votre héros
help-basics-2-body = Dans le salon, cliquez sur une carte pour prendre ce héros. Détails montre sa classe, son niveau, ses PV et toutes ses statistiques avant de vous engager — une carte marquée 🔒 est déjà prise.
help-basics-3-title = Combattez, tour par tour
help-basics-3-body = Quand la carte de votre héros s'anime, appuyez sur ⚔️, choisissez une attaque, choisissez une cible, confirmez. Toute la boucle tient là.
help-basics-host-note = ▶ Démarrer la partie appartient à l'hôte, une fois que tout le monde a choisi. En Solo, vous prenez plusieurs héros et les jouez tous — recliquez sur une carte prise pour la libérer.
help-basics-offline-title = ✈️ Pressé ?
help-basics-offline-body = Jouer hors ligne sur la page de connexion : pas de compte, pas de serveur, une partie solo à un clic.
help-basics-legend-title = Ce que veulent dire les icônes
help-legend-attack = ⚔️ — ouvre votre liste d'attaques (à votre tour)
help-legend-potion = 💊 — boire une potion ; cela consomme votre action
help-legend-aggro = 🎯 — aggro : la menace accumulée sur les 5 derniers tours
help-legend-extra = ⚡×1 — un tour bonus gagné grâce à la Vitesse
help-legend-taken = 🔒 — héros déjà pris par un autre joueur
help-legend-badge = Point rouge — points de talent non dépensés ou nouvel équipement

help-combat-flow-title = Votre tour
help-combat-flow-body = ⚔️ ouvre votre liste d'attaques, chacune avec son coût en énergie. Choisissez-en une, cliquez sur les boutons de cible, puis confirmez avec Lancer l'attaque.
help-combat-potion-title = Potions
help-combat-potion-body = 💊 liste vos potions et celles de l'équipe. Boire consomme votre action du tour — soignez un coup avant d'en avoir besoin, pas un coup trop tard.
help-combat-energy-title = Énergie
help-combat-energy-body = Les attaques dépensent et régénèrent 🔮 Mana, 💪 Vigueur et 😡 Rage. Un tour sur une attaque bon marché, c'est un tour à recharger la plus chère.
help-combat-order-title = Ordre des tours
help-combat-order-body = La Vitesse fixe l'ordre — le plus rapide d'abord, tous les héros puis tous les boss. Dans chaque camp, le plus rapide au-dessus de 100 de Vitesse gagne en plus un tour bonus : le badge ⚡ sur sa carte.
help-combat-crit-title = Critiques, esquives, blocages
help-combat-crit-body = Un coup critique double les dégâts. La plupart des héros esquivent ; les Berserkers bloquent à la place. Les séries de malchance sont brisées volontairement : une mauvaise passe finit toujours.
help-combat-read-title = Lisez le plateau
help-combat-read-body = Journaux rejoue chaque coup, soin et effet. Statistiques suit les dégâts, les soins, les éliminations et la progression du scénario. Paramètres décide si les barres de PV et d'énergie des boss sont visibles.

help-world-intro = Entre deux combats, l'hôte peut ouvrir 🗺 Explorer le monde — une carte en cases que l'on parcourt pour trouver le boss suivant.
help-world-move-label = Se déplacer
help-world-move-keys = Touches fléchées, ou la croix directionnelle à l'écran
help-world-interact-label = Interagir
help-world-interact-keys = Entrée ou Espace, en étant à côté d'un PNJ
help-world-zoom-label = Zoom
help-world-zoom-keys = − / + dans le coin de la carte ; votre réglage est mémorisé
help-world-grass = Les hautes herbes cachent des rencontres aléatoires — environ un pas sur deux déclenche un combat.
help-world-boss = Parlez à un PNJ boss pour lancer son dialogue d'avant-combat, puis confirmez pour engager la bataille.
help-world-door = Battez ce boss et la porte suivante s'ouvre ; le PNJ quitte la carte définitivement.
help-world-back = ⚔️ Retour au combat vous ramène au combat en cours à tout moment.

help-gear-store-title = La boutique
help-gear-store-body = Elle s'ouvre sur l'écran de fin de scénario via 🛒 Boutique — ou à tout moment pendant un scénario si vous activez Boutique pendant le scénario dans les Paramètres.
help-gear-tabs-title = Boutique et Sac
help-gear-tabs-body = La boutique vend de l'équipement (armes, armures, anneaux et plus) et des consommables — potions de PV, Mana, Vigueur, Rage et Résurrection. Le Sac contient ce que vous possédez et le rachète à moitié prix : dépensez à bon escient.
help-gear-equip-title = S'équiper
help-gear-equip-body = L'équipement acheté arrive dans votre sac ; ouvrez Inventaire pour l'équiper et voir les statistiques bouger. Un point sur la barre d'outils signale une nouveauté qui vous attend.
help-gear-gold-title = L'or
help-gear-gold-body = L'or tombe en butin à la fin de chaque scénario, et chaque héros garde sa propre bourse — en plus du butin d'équipe accessible à tous.

help-progress-scenario-title = Fin d'un scénario
help-progress-scenario-body = Le butin et l'expérience sont distribués, les héros montent de niveau et de nouvelles attaques se débloquent — l'écran de résumé liste exactement ce qui a changé.
help-progress-talents-title = Talents
help-progress-talents-body = Chaque niveau donne un point de compétence, plus un point bonus tous les cinq niveaux. Dépensez-les dans l'arbre de Talents ; Réinitialiser vous les rend tous si vous voulez une autre voie.
help-progress-cap-title = Niveau 13
help-progress-cap-body = 13 est le niveau maximum — et une attaque ultime de niveau 13 ne peut jamais être esquivée ni bloquée.
help-progress-stages-title = Dix étapes
help-progress-stages-body = Chaque univers compte dix scénarios. L'hôte charge le suivant ; Scénarios montre ce qui est terminé, en cours et encore verrouillé.
help-progress-save-title = Sauvegarder
help-progress-save-body = Trois emplacements par joueur. Sauvegarder écrit dans l'emplacement courant, Sauvegarde auto le fait pour vous à chaque nouvelle étape, et Charger une partie reprend l'aventure des jours plus tard.

help-pro-title = Ce que font les habitués
help-pro-speed = Une Vitesse au-dessus de 100 achète un tour bonus — un par camp, par tour. Un équipement qui donne de la Vitesse vaut souvent mieux qu'un équipement qui donne des dégâts.
help-pro-panel = ⚙ Configurer, sur le panneau d'attaques, permet de glisser vos attaques dans l'ordre où vous les utilisez vraiment. Enregistré par héros et par serveur.
help-pro-aggro = Activez Aggro des héros dans les Paramètres pour voir d'un coup d'œil quel héros porte le combat depuis cinq tours.
help-pro-ultimate = Gardez l'ultime pour un boss : rien ne l'esquive, rien ne la bloque.
help-pro-berserker = Les Berserkers bloquent au lieu d'esquiver et brisent une série sans critique au bout de trois tours — l'agressivité est leur défense.
help-pro-mystery = Masquez la barre de PV du boss pour un combat sous tension ; affichez plutôt ses barres d'énergie et vous verrez venir la grosse attaque.
help-pro-save = Sauvegardez avant un boss. Si toute l'équipe tombe, la partie est finie.
help-pro-solo = Servez-vous d'une partie solo pour tester une composition d'équipe avant de l'emmener en multijoueur.

help-admin-intro = Connecté sur le compte administrateur, le lien 🛡️ Panneau apparaît dans la barre du haut.
help-admin-users = Utilisateurs — les comptes et qui est connecté en ce moment.
help-admin-characters = Personnages — tous les héros et boss, par univers.
help-admin-scenarios = Scénarios — ajoutez, modifiez ou supprimez des scénarios dans l'éditeur JSON intégré.
help-admin-content = Attaques & Équipement — ajustez les coûts, dégâts, effets et prix.

## conseils de combat du premier scénario (board_game_components/tutorial.rs)
hint-badge = Tutoriel
hint-dismiss = Masquer ces conseils
hint-waiting = ⏳ { $name } joue — héros et boss agissent par ordre de Vitesse. Votre carte s'illumine quand c'est votre tour.
hint-your-turn = ⚔️ À vous. Appuyez sur ⚔️ sur la carte de { $name } pour ouvrir les attaques — ou sur 💊 pour boire une potion.
hint-pick-attack = Choisissez une attaque. Une attaque grisée ne peut pas être lancée ce tour-ci, le plus souvent faute d'énergie.
hint-pick-attack-target = 🎯 Cliquez sur un cercle clignotant d'une carte pour viser, puis confirmez avec « ⚔️ Lancer l'attaque ».
hint-pick-potion-target = Cliquez sur le héros qui la boit, puis appuyez sur « ✅ Utiliser ».

footer-about = À propos
footer-lib-rpg-engine = moteur lib-rpg
footer-built-with-dioxus = Construit avec Dioxus
footer-contact = Contact
footer-report-issue = Signaler un problème
footer-discussions = Discussions

## join-ongoing-game page
join-ongoing-title = 🗺️ Aventures en cours
join-ongoing-empty = Aucune partie en cours. Créez-en une !

## home page
home-title = ⚔️ Aventure RPG
home-welcome = Bienvenue, { $user_name } !
home-create-server = Créer un serveur
home-create-server-desc = Démarrez une nouvelle aventure en tant qu'hôte
home-join-game = Rejoindre une partie
home-join-game-desc = Rejoignez une aventure en cours

## admin page shell
admin-panel-title = 🛡️ Panneau d'administration
admin-panel-disabled = Le panneau d'administration est désactivé.
admin-panel-access-denied = Vous n'avez pas accès au panneau d'administration.
admin-tab-users = 👤 Utilisateurs
admin-tab-scenarios = 📜 Scénarios
admin-tab-characters = 🧙 Personnages
admin-tab-equipment = 🔧 Équipement

## common (reused across many pages)
common-loading = Chargement…

## admin users tab
admin-users-title = 📋 Tous les utilisateurs
admin-users-empty = Aucun utilisateur trouvé.
admin-users-col-username = Nom d'utilisateur
admin-users-col-connected = Connecté
admin-users-col-saves = Sauvegardes
admin-users-delete-title = 🗑️ Supprimer un utilisateur
admin-users-delete-label = Nom d'utilisateur à supprimer
admin-users-delete-placeholder = Entrez un nom d'utilisateur…
admin-users-delete-button = Supprimer l'utilisateur
admin-users-delete-success = ✅ Utilisateur supprimé.
admin-users-delete-error = ❌ Ce nom ne peut pas être supprimé.

## load game page
loadgame-fetch-error = Échec du chargement des sauvegardes : { $error }
loadgame-slot-scenario = 📜 { $scenario } (Niv { $level })
loadgame-mode-solo = 🎮 Solo
loadgame-mode-multi = 👥 Multi ({ $players }j)
loadgame-universe = 🌐 { $universe }

## login page
login-sign-in-title = Connexion
login-empty-username = Veuillez saisir un nom d'utilisateur.
login-empty-fields = Veuillez saisir un nom d'utilisateur et un mot de passe.
login-username-placeholder = Votre nom d'utilisateur
login-password-placeholder = Mot de passe
login-success = { $username } connecté(e)
login-session-expired = Votre session a expiré. Veuillez vous reconnecter.
login-sign-in-button = Se connecter →
login-create-account-title = Créer un compte
login-choose-username-placeholder = Choisissez un nom d'utilisateur
login-choose-password-placeholder = Choisissez un mot de passe
login-invalid-login = Identifiants invalides
login-name-taken = Ce nom est déjà pris.
login-sign-up-button = S'inscrire →
login-offline-title = ✈️ Jouer hors ligne
login-offline-hint = Sans compte, sans serveur — choisissez un univers et lancez une partie solo locale.
login-offline-start-button = Jouer hors ligne
login-offline-choose-universe-label = Choisissez un univers
login-offline-choose-universe-option = -- Choisissez un univers --

## create server page
create-server-title = 🏰 Créer une partie
create-server-step1 = 1️⃣ Mode de jeu
create-server-multiplayer = 👥 Multijoueur
create-server-singleplayer = 🎮 Solo
create-server-singleplayer-hint = Un joueur contrôle tous les héros.
create-server-multiplayer-hint = Chaque joueur connecté choisit un héros.
create-server-step2 = 2️⃣ Choisissez un emplacement de sauvegarde
create-server-empty-slot = Emplacement vide { $index }
create-server-continue-play = ▶ Continuer
create-server-overwrite-play = 🗑 Écraser et jouer
create-server-overwrite-confirm-title = Écraser cette sauvegarde ?
create-server-overwrite-confirm-body = Cette action supprimera définitivement cette aventure sauvegardée pour en démarrer une nouvelle à la place. Cette action est irréversible.

## lobby page
lobby-title = ⚔️ Salon
lobby-loading = ⏳ Chargement…
lobby-server-label = Serveur
lobby-players-label = Joueurs
lobby-universe-label = Univers
lobby-scenarios-label = Scénarios
lobby-start-game = ▶ Démarrer la partie
lobby-universe-saved-label = Univers (sauvegardé)
lobby-universe-locked = 🔒 { $universe }
lobby-choose-universe-label = Choisissez un univers
lobby-select-universe-option = — sélectionnez un univers —
lobby-not-enough-players = Pas assez de joueurs
lobby-game-ended = Plus de partie, retour à l'accueil

## equipment tab widget
equip-count-equipped = { $count } équipé(s)
equip-new-item = Nouvel objet !
equip-section-equipped = ✅ Équipé
equip-section-in-bag = 🎒 Dans le sac
equip-empty-slot = Aucun objet dans cet emplacement.
equip-not-found = Équipement introuvable
equip-new-dot-title = Nouveau !
equip-no-stat-bonuses = Aucun bonus de statistique.
equip-click-to-unequip = Cliquez pour déséquiper
equip-click-to-equip = Cliquez pour équiper

## common (reused across many pages)
common-level = Niv { $level }

## character select
char-select-title-single = 🎮 Solo — Choisissez vos héros
char-select-title-multi = 👥 Choisissez votre personnage
char-select-other-players = Autres joueurs :
char-card-taken-by = 🔒 { $taker }
char-card-remove = × Retirer
char-card-select = + Choisir
char-card-details = ℹ️ Détails

## charts widget
charts-no-attacks = Aucune attaque enregistrée pour l'instant.
charts-attack-frequency = ⚔️ Fréquence des attaques
charts-no-dmg-heal = Aucune donnée de dégâts ou de soin pour l'instant.
charts-damage-dealt = 🗡️ Dégâts infligés
charts-healing-done = 💚 Soins prodigués
charts-suffix-dmg = dégâts
charts-suffix-hp = PV
charts-total-damage = Dégâts totaux
charts-total-heal = Soins totaux
charts-dmg-per-round = Dégâts / Tour
charts-heal-per-round = Soins / Tour
charts-attacks-cast = Attaques lancées
charts-favourite = Favorite
charts-party-tab = 🌐 Groupe

## startgame / running game page
startgame-lvl = Niv { $level }
startgame-defeated = 💀 Vaincu
startgame-quit = 🚪 Quitter
startgame-game-over = 💀 Partie terminée
startgame-remaining-players = Joueurs restants : { $count }
startgame-replay-game = 🔄 Rejouer la partie
startgame-scenario-complete = 🏆 Scénario terminé !
startgame-finishing-blow-dot = ⚔️ Coup fatal (DOT)
startgame-finishing-blow = ⚔️ Coup fatal
startgame-enemy-last-attack = Dernière attaque de l'ennemi : { $name }
startgame-shop = 🛒 Boutique
startgame-explore-overworld = 🗺 Explorer le monde
startgame-loots = 🎁 Butin
startgame-no-loots = Aucun butin cette fois-ci.
startgame-level-upgrades = ⬆️ Montées de niveau
startgame-level-up-change = ⬆️ { $old } → { $new }
startgame-level-unchanged = 🟰 Niv { $level }
startgame-new-attacks = ✨ Nouvelles attaques débloquées :
startgame-turn-round = ⚔️ Tour { $turn } - Manche { $round }
startgame-run-away = 🗺 Fuir

## gameboard
gameboard-spectator-mode = 👁 Mode spectateur — vous n'avez aucun personnage actif dans cette partie
gameboard-use = ✅ Utiliser
gameboard-launch-attack = ⚔️ Lancer l'attaque
gameboard-attacks = ⚔️ { $launcher } attaque !
gameboard-turn-round = 🔄 Tour { $turn } — Manche { $round }
gameboard-last-attack = Dernière attaque : { $name }
gameboard-critical-strike = 💥 Coup critique !
gameboard-is-dodging = { $name } esquive
gameboard-is-blocking = { $name } bloque

## overworld
overworld-entering = 🗺 Entrée dans le monde…
overworld-start-fight-question = Voulez-vous commencer le combat ?
overworld-yes-fight = ⚔️ Oui, combattre !
overworld-no-not-yet = 🚪 Non, pas encore
overworld-controls-hint = Flèches : déplacer  |  Entrée/⚔ : interagir

## common admin error/status
admin-error = ❌ { $error }
admin-deleted = ✅ Supprimé.

## admin equipment tab
admin-equip-browser-title = 🔧 Navigateur d'équipement
admin-equip-select-type = — type —
admin-equip-select-category = — catégorie —
admin-equip-items-title = 📋 Objets — { $category }
admin-equip-cancel-new = ✕ Annuler
admin-equip-new = ➕ Nouveau
admin-equip-new-item-in = ➕ Nouvel objet dans { $category }
admin-equip-filename-label = Nom de fichier de l'objet (sans espaces, sans extension)
admin-equip-filename-placeholder = ex. epic_sword
admin-equip-name-empty = ❌ Le nom ne peut pas être vide.
admin-equip-created = ✅ « { $name } » créé.
admin-equip-create = 💾 Créer
admin-equip-edit-title = ✏️ { $name }
admin-equip-json-mode = ✏️ Mode JSON
admin-equip-form-mode = 📝 Mode formulaire
admin-equip-name-label = Nom
admin-equip-unique-name-label = Nom unique
admin-equip-category-label = Catégorie
admin-equip-stats-title = Statistiques
admin-equip-stat-col = Statistique
admin-equip-value-col = Valeur
admin-equip-save = 💾 Enregistrer
admin-equip-saved = ✅ Enregistré.

## admin scenarios tab
admin-scenarios-select-universe = 🌐 Sélectionner un univers
admin-scenarios-choose-universe = — choisissez un univers —
admin-scenarios-title = 📜 Scénarios — { $universe }
admin-scenarios-empty = Aucun scénario trouvé pour cet univers.
admin-scenarios-col-level = Niv
admin-scenarios-col-bosses = Boss
admin-scenarios-col-description = Description
admin-scenarios-col-file = Fichier
admin-scenarios-col-actions = Actions
admin-scenarios-edit = ✏️ Modifier
admin-scenarios-confirm-delete = ⚠️ Confirmer
admin-scenarios-delete = 🗑️ Supprimer
admin-scenarios-add = ➕ Ajouter un scénario
admin-scenarios-new-title = ➕ Nouveau scénario
admin-scenarios-edit-title = ✏️ Modifier le scénario
admin-scenarios-file-stem-label = Nom de fichier (ex. stage_11)
admin-scenarios-file-stem-placeholder = stage_11
admin-scenarios-name-placeholder = Nom du scénario
admin-scenarios-description-placeholder = Décrivez le scénario…
admin-scenarios-level-label = Niveau
admin-scenarios-bosses-label = Boss (un par ligne — « NomDuBoss » ou « NomDuBoss : 0, 1, 2 »)
admin-scenarios-loots-label = Butin
admin-scenarios-loot-level-placeholder = Niv
admin-scenarios-loot-classes-placeholder = Classes (Standard, Guerrier…)
admin-scenarios-remove-loot = ✕
admin-scenarios-add-loot = ＋ Ajouter du butin
admin-scenarios-file-stem-empty = ❌ Le nom de fichier ne peut pas être vide.

## loot kind / rank labels (shared across admin tabs)
loot-kind-equipment = Équipement
loot-kind-consumable = Consommable
loot-kind-material = Matériau
loot-kind-currency = Monnaie
rank-common = Commun
rank-intermediate = Intermédiaire
rank-advanced = Avancé

## admin attacks tab
admin-atk-upload-error = ❌ Envoi : { $error }
admin-atk-title = ⚔️ Attaques — { $character }
admin-atk-empty = Aucune attaque trouvée.
admin-atk-form-edit-title = 📝 { $name }
admin-atk-level-label = Niveau
admin-atk-target-label = Cible
admin-atk-reach-label = Portée
admin-atk-form-label = Forme
admin-atk-photo-label = Photo
admin-atk-photo-placeholder = ex. Fireball.png
admin-atk-upload-photo-label = Envoyer une photo
admin-atk-cost-mana-label = Coût en mana
admin-atk-cost-rage-label = Coût en rage
admin-atk-cost-vigor-label = Coût en vigueur
admin-atk-duration-label = Durée
admin-atk-aggro-label = Aggro
admin-atk-damage-label = Dégâts
admin-atk-heal-label = Soin
admin-atk-regen-mana-label = Régén. Mana
admin-atk-regen-rage-label = Régén. Rage
admin-atk-regen-vigor-label = Régén. Vigueur
admin-atk-effects-label = Effets (tableau JSON)
admin-atk-target-enemy = Ennemi
admin-atk-target-ally = Allié
admin-atk-target-self = Soi-même
admin-atk-target-zone = Zone
admin-atk-target-all = Tous
admin-atk-reach-individual = Individuel
admin-atk-reach-area = Zone
admin-atk-reach-all = Tous
admin-atk-form-standard = Standard
admin-atk-form-magic = Magique
admin-atk-form-healing = Soin
admin-atk-form-support = Soutien
admin-atk-new-title = ➕ Nouvelle attaque
admin-atk-new-placeholder = Nom de fichier de l'attaque (ex. Fireball)
admin-atk-created = ✅ Créée.
admin-atk-create-button = Créer

## character page (attack tooltip / potion list)
character-page-effects-label = Effets
character-page-no-potions = Aucune potion disponible
character-page-lvl = Niv. { $level }
character-page-extra-round-title = Tour supplémentaire grâce à l'avantage de vitesse
character-page-aggro-title = Aggro
character-page-sort-by-level = ⇅ Niv.
character-page-sort-by-cost = ⇅ Coût
character-page-configure-atk-panel = ⚙ Configurer
character-page-configure-atk-panel-title = Configurer le panneau d'attaques
character-page-configure-atk-panel-desc = Glissez-déposez pour réorganiser vos attaques. Cette disposition est sauvegardée pour ce personnage sur ce serveur.
character-page-cancel = Annuler
character-page-reset-atk-panel = Réinitialiser
character-page-save-atk-panel = Enregistrer

## admin characters tab
admin-chars-filter-universe = 🌐 Filtrer par univers
admin-chars-all-universes = — tous les univers —
admin-chars-create-universe-title = 🌍 Créer un univers
admin-chars-universe-name-placeholder = Nom de l'univers (ex. pokemon)
admin-chars-universe-created = ✅ Univers créé.
admin-chars-heroes = 🧙 Héros
admin-chars-bosses = 👹 Boss
admin-chars-none-found = Aucun { $kind } trouvé.
admin-chars-form-button = 📝 Formulaire
admin-chars-json-button = ✏️ JSON
admin-chars-attacks-button = ⚔️ Attaques
admin-chars-form-title = 📝 Formulaire : { $name }
admin-chars-name-placeholder = Nom du personnage
admin-chars-short-name-label = Nom court
admin-chars-class-label = Classe
admin-chars-rank-label = Rang
admin-chars-type-label = Type
admin-chars-type-hero = Héros
admin-chars-type-boss = Boss
admin-chars-photo-filename-label = Nom de fichier de la photo
admin-chars-photo-placeholder = ex. Thalia.png
admin-chars-color-label = Couleur
admin-chars-color-placeholder = ex. green
admin-chars-max-actions-label = Actions max / tour
admin-chars-description-placeholder = Description du personnage…
admin-chars-energies-label = Énergies
admin-chars-blocking-atk-label = Bloque les attaques
admin-chars-current-col = Actuel
admin-chars-max-col = Max
admin-chars-json-title = ✏️ JSON : { $name }

## character class labels (shared)
class-warrior = Guerrier
class-mage = Mage
class-healer = Guérisseur
class-berserker = Berserker

## energy labels (shared)
energy-mana = Mana
energy-rage = Rage
energy-vigor = Vigueur

## game_sheets.rs
gs-save = Sauvegarder
gs-menu = Menu
gs-inventory = Inventaire
gs-talents = Talents
gs-talents-unspent-points = Points de talent non dépensés !
gs-new-equipment = Nouvel équipement !
gs-new-equipment-for = Nouvel équipement pour { $name } !
gs-logs = Journal
gs-game-stats = Statistiques
gs-scenarios = Scénarios
gs-settings = Paramètres
gs-store-disabled-hint = La boutique est désactivée dans les paramètres
gs-store = Boutique
gs-close = Fermer
gs-inv-title = 🎒 Inventaire — { $name }
gs-inv-desc = Aperçu de l'équipement au niveau { $level }
gs-inv-use-items-title = 💊 Utiliser des objets
gs-inv-no-consumables = Aucune potion ou objet à utiliser.
gs-inv-personal-potions = 💊 Potions personnelles
gs-inv-party-potions = 🎁 Consommables communs
gs-inv-use = Utiliser
gs-talents-title = 🌳 Talents — { $name }
gs-talents-desc = Arbre de talents au niveau { $level }
talents-points-available = Points de compétence disponibles : { $points }
talents-respec = Réinitialiser
talents-no-tree = Aucun talent disponible pour ce héros pour le moment.
talents-cost-label = Coût : { $cost } point(s)
talents-locked-requires = Nécessite : { $name }
talents-locked-capstone = Le talent capstone d'une autre voie ({ $name }) est déjà actif — réinitialisez d'abord
talents-locked-points = Points de compétence insuffisants
gs-stats-title = 📊 Statistiques
gs-stats-desc = Aperçu de la session en cours.
gs-turn-label = Tour
gs-round-label = Manche
gs-kills-label = Éliminations
gs-stats-active-player-label = Joueur actif
gs-scenario-progress-title = Progression du scénario
gs-scenarios-completed = { $completed } / { $total } terminés
gs-heroes-status-title = État des héros
gs-menu-title = Menu
gs-menu-desc = Contrôles de la session de jeu.
gs-toolbar-title = Menu de jeu
gs-toolbar-open = Ouvrir le menu de jeu
gs-menu-server-label = Serveur
gs-menu-active-player-label = Joueur actif
gs-menu-players-label = Joueurs
gs-game-saved = Partie sauvegardée !
gs-logs-title = Journal
gs-logs-desc = Historique de tous les événements de la partie.
gs-logs-all = Tout
gs-logs-combat = ⚔ Combat
gs-logs-healing = 💚 Soins
gs-logs-events = ℹ Événements
gs-logs-empty = Aucun événement pour l'instant.
gs-scenarios-sheet-title = 📜 Scénarios
gs-scenarios-sheet-desc = Progression à travers toutes les étapes disponibles.
gs-scenarios-empty = Aucun scénario chargé.
gs-scenario-completed = ✅ Terminé
gs-scenario-in-progress = ⚔️ En cours
gs-scenario-not-started = 🔒 Non commencé
gs-store-title = 🛒 Boutique — { $name }
gs-gold-amount = 💰 { $amount } or
gs-store-shop = 🏪 Boutique
gs-store-bag = 🎒 Sac
gs-store-equipment = ⚔️ Équipement
gs-store-consumables = 💊 Consommables
gs-store-slot = Emplacement : { $category }
gs-store-in-bag = (×{ $count } dans le sac)
gs-store-buy = Acheter
gs-store-no-gold = Pas d'or
gs-store-bag-empty = Votre sac est vide.
gs-store-party-loot = 🎒 Butin du groupe
gs-store-sell = Vendre
gs-settings-title = ⚙️ Paramètres
gs-settings-desc = Personnalisez votre expérience de jeu.
gs-settings-tooltips-label = Infobulles d'attaque
gs-settings-tooltips-hint = Afficher la description de l'attaque au survol dans la liste des attaques.
gs-settings-boss-energy-label = Barres d'énergie du boss
gs-settings-boss-energy-hint = Afficher les barres de mana/vigueur/rage pour les boss.
gs-settings-hero-aggro-label = Aggro des héros
gs-settings-hero-aggro-hint = Afficher la valeur d'aggro sur l'en-tête du panneau du héros.
gs-settings-boss-hp-label = Barre de vie du boss
gs-settings-boss-hp-hint = Afficher la barre de vie sur les panneaux de boss (masquez-la pour garder le mystère).
gs-settings-autosave-label = Sauvegarde auto au scénario
gs-settings-autosave-hint = Sauvegarder automatiquement au début de chaque nouveau scénario.
gs-settings-shop-label = Boutique pendant le scénario
gs-settings-shop-hint = Autoriser l'ouverture de la boutique pendant un scénario en cours.
gs-settings-saving = Enregistrement…
gs-settings-saved = ✅ Enregistré

## popover_comp.rs (unreferenced demo component)
popover-demo-trigger = Afficher la popover
popover-demo-title = Supprimer l'objet ?
popover-demo-confirmed = Objet supprimé !
