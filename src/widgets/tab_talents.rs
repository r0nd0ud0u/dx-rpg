use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    prelude::*,
};
use dioxus_i18n::t;
use dioxus_primitives::ContentSide;
use lib_rpg::{
    character_mod::{
        character::Character,
        talent::{TalentDef, TalentTree},
    },
    common::lang::Lang,
    server::server_manager::ServerData,
};

use crate::{
    common::{CtxAppLang, SERVER_NAME, lang_from_app_lang},
    components::{
        button::{Button, ButtonVariant},
        tooltip::{Tooltip, TooltipContent, TooltipTrigger},
    },
    websocket_handler::event::{ClientEvent, ServerEvent},
};

fn talent_name(talent: &TalentDef, lang: Lang) -> &str {
    match lang {
        Lang::Fr if !talent.name_fr.is_empty() => &talent.name_fr,
        _ => &talent.name_en,
    }
}

fn talent_description(talent: &TalentDef, lang: Lang) -> &str {
    match lang {
        Lang::Fr if !talent.description_fr.is_empty() => &talent.description_fr,
        _ => &talent.description_en,
    }
}

#[component]
pub fn TabTalents(c: Character) -> Element {
    // contexts
    let server_data = use_context::<Signal<ServerData>>();
    let app_lang = use_context::<CtxAppLang>().0;
    let server_name = SERVER_NAME();
    let lang = lang_from_app_lang(&app_lang());

    let tree = server_data()
        .core_game_data
        .game_manager
        .pm
        .talent_trees
        .get(&c.db_full_name)
        .cloned();

    let Some(tree) = tree else {
        return rsx! {
            div { class: "equip-empty", {t!("talents-no-tree")} }
        };
    };

    rsx! {
        div { class: "talent-header",
            span { class: "talent-points-label",
                {t!("talents-points-available", points : c.talents.available() as i64)}
            }
            RespecButton { c: c.clone(), server_name: server_name.clone() }
        }
        div { class: "talent-tree",
            for path in tree.paths.iter() {
                TalentPathColumn {
                    path: path.clone(),
                    tree: tree.clone(),
                    character: c.clone(),
                    lang,
                    server_name: server_name.clone(),
                }
            }
        }
    }
}

#[component]
fn RespecButton(c: Character, server_name: String) -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let character_id_name = c.id_name.clone();
    let has_unlocked = !c.talents.unlocked.is_empty();

    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            disabled: !has_unlocked,
            onclick: move |_| {
                let character_id_name = character_id_name.clone();
                let server_name = server_name.clone();
                async move {
                    let _ = socket
                        .send(ClientEvent::RequestRespecTalents(server_name, character_id_name))
                        .await;
                }
            },
            {t!("talents-respec")}
        }
    }
}

#[component]
fn TalentPathColumn(
    path: lib_rpg::character_mod::talent::TalentPath,
    tree: TalentTree,
    character: Character,
    lang: Lang,
    server_name: String,
) -> Element {
    let path_name = if lang == Lang::Fr && !path.name_fr.is_empty() {
        path.name_fr.clone()
    } else {
        path.name_en.clone()
    };
    let mut talents = path.talents.clone();
    talents.sort_by_key(|t| t.tier);

    rsx! {
        div { class: "talent-path",
            div { class: "talent-path-title", "{path_name}" }
            for talent in talents {
                TalentNode {
                    talent: talent.clone(),
                    tree: tree.clone(),
                    character: character.clone(),
                    lang,
                    server_name: server_name.clone(),
                }
            }
        }
    }
}

#[component]
fn TalentNode(
    talent: TalentDef,
    tree: TalentTree,
    character: Character,
    lang: Lang,
    server_name: String,
) -> Element {
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();

    let is_unlocked = character.talents.unlocked.iter().any(|id| id == &talent.id);
    let missing_requires: Vec<String> = talent
        .requires
        .iter()
        .filter(|r| !character.talents.unlocked.contains(r))
        .filter_map(|r| tree.find_talent(r))
        .map(|t| talent_name(t, lang).to_owned())
        .collect();
    let prereqs_met = missing_requires.is_empty();
    let affordable = character.talents.available() >= talent.cost;
    let blocking_capstone: Option<String> = if talent.is_capstone {
        tree.other_capstones(&talent.path)
            .into_iter()
            .find(|other_id| character.talents.unlocked.iter().any(|u| u == other_id))
            .and_then(|other_id| tree.find_talent(other_id))
            .map(|t| talent_name(t, lang).to_owned())
    } else {
        None
    };
    let is_unlockable = !is_unlocked && prereqs_met && affordable && blocking_capstone.is_none();
    let disabled = is_unlocked || !is_unlockable;

    let variant = if is_unlocked {
        ButtonVariant::Primary
    } else if is_unlockable {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Outline
    };

    let talent_id = talent.id.clone();
    let character_id_name = character.id_name.clone();

    rsx! {
        div {
            class: if talent.is_capstone { "talent-node-row talent-node-row--capstone" } else { "talent-node-row" },
            Tooltip {
                TooltipTrigger {
                    div {
                        Button {
                            variant,
                            width: "100%",
                            disabled,
                            onclick: move |_| {
                                let talent_id = talent_id.clone();
                                let character_id_name = character_id_name.clone();
                                let server_name = server_name.clone();
                                async move {
                                    let _ = socket
                                        .send(
                                            ClientEvent::RequestUnlockTalent(
                                                server_name,
                                                character_id_name,
                                                talent_id,
                                            ),
                                        )
                                        .await;
                                }
                            },
                            span { class: "talent-node-label",
                                if talent.is_capstone {
                                    span { class: "talent-capstone-badge", "★ " }
                                }
                                "{talent_name(&talent, lang)}"
                                if is_unlocked {
                                    span { class: "equip-equipped-check", " ✓" }
                                }
                            }
                        }
                    }
                }
                TooltipContent { side: ContentSide::Left,
                    p { style: "margin:0 0 4px 0; font-weight:600; color:var(--rpg-gold,#c9a227);",
                        "{talent_name(&talent, lang)}"
                    }
                    p { style: "margin:0 0 4px 0;", "{talent_description(&talent, lang)}" }
                    p { style: "margin:0; color: var(--rpg-text-muted,#8a8fa8);",
                        {t!("talents-cost-label", cost : talent.cost as i64)}
                    }
                    if !is_unlocked {
                        if !prereqs_met {
                            p { style: "margin:4px 0 0 0; font-size:0.72rem; color: var(--rpg-danger-light,#f87171);",
                                {t!("talents-locked-requires", name : missing_requires.join(", "))}
                            }
                        } else if let Some(blocker) = &blocking_capstone {
                            p { style: "margin:4px 0 0 0; font-size:0.72rem; color: var(--rpg-danger-light,#f87171);",
                                {t!("talents-locked-capstone", name : blocker.clone())}
                            }
                        } else if !affordable {
                            p { style: "margin:4px 0 0 0; font-size:0.72rem; color: var(--rpg-danger-light,#f87171);",
                                {t!("talents-locked-points")}
                            }
                        }
                    }
                }
            }
        }
    }
}
