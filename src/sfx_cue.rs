//! Which sound effect a game event should play, from the client's point of view.
//!
//! [`lib_rpg::common::sound_cue::classify_result_atk`] is the server-side half of
//! this: it looks at the HP delta of every landed effect and returns
//! [`SoundCue::Hit`] / [`SoundCue::CriticalHit`] / [`SoundCue::Heal`], plus the
//! dodge/block cues. That covers damage and instant cures, but leaves two
//! families of attacks with nothing to play:
//!
//! * **Pure support casts** — attacks whose effects only move non-HP stats
//!   (`Barrier`, `Withdraw`, `Furie du Mordor`, Thraïn's shields and taunts, …).
//!   Every effect reports `real_amount_tx == 0`, so the classifier returns no
//!   cue at all and the cast lands in total silence.
//! * **Lasting regens** — `Essence Régénératrice` and friends. The first tick of
//!   the heal-over-time does report an HP gain, so these play the same short
//!   `heal` blip as a one-shot cure; a five-turn blessing ends up sounding like
//!   a UI confirmation, which is why it reads as "no sound" in play.
//!
//! [`classify_attack`] re-classifies both from the effect parameters the server
//! already sends, so every landed attack makes a sound and support casts sound
//! like support. Kept client-side because it is a presentation decision, in the
//! spirit of `sound_cue`'s own "playback is a client/UI concern" note — if it
//! ever needs to be shared with another front-end, it can move upstream as-is.

use lib_rpg::{
    character_mod::buffers::BufKinds,
    common::{
        constants::all_target_const::TARGET_ENNEMY,
        sound_cue::{SoundCue, classify_result_atk},
    },
    server::{game_manager::ResultLaunchAttack, players_manager::GameAtkEffect},
};

/// Every sound the client can play. A superset of [`SoundCue`]: the server has no
/// notion of the two support cues, which exist purely to give buff/debuff casts a
/// voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    Hit,
    CriticalHit,
    Dodge,
    Block,
    /// An instant cure.
    Heal,
    /// A lasting boon landed on an ally — regen, shield, war cry.
    Buff,
    /// A lasting affliction landed on an enemy with no damage of its own.
    Debuff,
    Potion,
    Victory,
    GameOver,
}

impl From<SoundCue> for Sfx {
    fn from(cue: SoundCue) -> Self {
        match cue {
            SoundCue::Hit => Sfx::Hit,
            SoundCue::CriticalHit => Sfx::CriticalHit,
            SoundCue::Dodge => Sfx::Dodge,
            SoundCue::Block => Sfx::Block,
            SoundCue::Heal => Sfx::Heal,
            SoundCue::Potion => Sfx::Potion,
            SoundCue::Victory => Sfx::Victory,
            SoundCue::GameOver => Sfx::GameOver,
        }
    }
}

/// Classifies an attack result into the sounds to play, most-significant first.
///
/// Starts from the server-side classification and fills its two gaps (see the
/// module docs): a heal that is really a regen becomes [`Sfx::Buff`], and a cast
/// that landed effects but earned no cue at all becomes [`Sfx::Buff`] or
/// [`Sfx::Debuff`] depending on who it landed on.
pub fn classify_attack(ra: &ResultLaunchAttack) -> Vec<Sfx> {
    let cues: Vec<Sfx> = classify_result_atk(ra).into_iter().map(Sfx::from).collect();

    // A dodge/block is the whole story of the attack — nothing else connected.
    if matches!(cues.as_slice(), [Sfx::Dodge] | [Sfx::Block]) {
        return cues;
    }

    // Nothing damaged and nothing cured, but effects did land: a support cast.
    // Its polarity is decided by who the landed effects were aimed at.
    if cues.is_empty() {
        if ra.new_game_atk_effects.is_empty() {
            return Vec::new();
        }
        return if ra.new_game_atk_effects.iter().any(is_aimed_at_enemy) {
            vec![Sfx::Debuff]
        } else {
            vec![Sfx::Buff]
        };
    }

    // A heal whose every restoring effect ticks over several turns is a regen,
    // not a cure — give it the blessing chime instead of the one-shot heal blip.
    // An instant restore, of HP or of any other stat, is left alone.
    if matches!(cues.as_slice(), [Sfx::Heal]) {
        let mut restores = ra
            .new_game_atk_effects
            .iter()
            .filter(|e| is_restore(e))
            .peekable();
        if restores.peek().is_some() && restores.all(is_lasting) {
            return vec![Sfx::Buff];
        }
    }

    cues
}

fn effect_param(effect: &GameAtkEffect) -> &lib_rpg::character_mod::effect::EffectParam {
    &effect.processed_effect_param.input_effect_param
}

fn is_aimed_at_enemy(effect: &GameAtkEffect) -> bool {
    effect_param(effect).target_kind == TARGET_ENNEMY
}

/// An effect that gave something back — HP, mana, vigour. This is exactly what
/// lib-rpg reads as [`SoundCue::Heal`].
fn is_restore(effect: &GameAtkEffect) -> bool {
    effect.effect_outcome.real_amount_tx > 0
}

/// True for an effect that keeps applying after the turn it was cast on — the
/// `ChangeCurrentStat`-over-N-turns shape lib-rpg uses for regens and DoTs.
fn is_lasting(effect: &GameAtkEffect) -> bool {
    let param = effect_param(effect);
    param.nb_turns > 1 && param.buffer.kind == BufKinds::ChangeCurrentStat
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_rpg::{
        character_mod::{
            buffers::Buffer,
            effect::{EffectOutcome, EffectParam, ProcessedEffectParam},
        },
        common::constants::{
            all_target_const::{TARGET_ALLY, TARGET_HIMSELF},
            stats_const::HP,
        },
        server::players_manager::DodgeInfo,
    };

    /// One landed effect: `stat` moved by `real_amount_tx` on `target_kind`,
    /// lasting `nb_turns` turns.
    fn effect(target_kind: &str, stat: &str, real_amount_tx: i64, nb_turns: i64) -> GameAtkEffect {
        GameAtkEffect {
            processed_effect_param: ProcessedEffectParam {
                input_effect_param: EffectParam {
                    target_kind: target_kind.to_owned(),
                    nb_turns,
                    buffer: Buffer {
                        stats_name: stat.to_owned(),
                        kind: BufKinds::ChangeCurrentStat,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            effect_outcome: EffectOutcome {
                real_amount_tx,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn result(effects: Vec<GameAtkEffect>) -> ResultLaunchAttack {
        ResultLaunchAttack {
            new_game_atk_effects: effects,
            ..Default::default()
        }
    }

    #[test]
    fn plain_damage_still_hits() {
        assert_eq!(
            classify_attack(&result(vec![effect(TARGET_ENNEMY, HP, -30, 1)])),
            vec![Sfx::Hit]
        );
    }

    #[test]
    fn instant_cure_stays_a_heal() {
        assert_eq!(
            classify_attack(&result(vec![effect(TARGET_ALLY, HP, 30, 1)])),
            vec![Sfx::Heal]
        );
    }

    #[test]
    fn lasting_regen_becomes_a_buff() {
        // Essence Régénératrice: +20 HP a turn for five turns.
        assert_eq!(
            classify_attack(&result(vec![effect(TARGET_ALLY, HP, 20, 5)])),
            vec![Sfx::Buff]
        );
    }

    #[test]
    fn stat_only_self_cast_becomes_a_buff() {
        // Barrier / Withdraw / Protection naine: no HP effect at all, so lib-rpg
        // classifies nothing and the cast used to be silent.
        assert_eq!(
            classify_attack(&result(vec![
                effect(TARGET_HIMSELF, "Physical armor", 0, 3),
                effect(TARGET_HIMSELF, "Magic armor", 0, 3),
            ])),
            vec![Sfx::Buff]
        );
    }

    #[test]
    fn stat_only_enemy_cast_becomes_a_debuff() {
        assert_eq!(
            classify_attack(&result(vec![effect(TARGET_ENNEMY, "Dodge", 0, 3)])),
            vec![Sfx::Debuff]
        );
    }

    #[test]
    fn a_regen_that_also_damages_keeps_the_impact() {
        let cues = classify_attack(&result(vec![
            effect(TARGET_ENNEMY, HP, -25, 1),
            effect(TARGET_ALLY, HP, 20, 5),
        ]));
        assert!(cues.contains(&Sfx::Hit), "{cues:?} should keep the impact");
        assert!(
            !cues.contains(&Sfx::Buff),
            "{cues:?} should not add a buff chime"
        );
    }

    #[test]
    fn dodge_is_the_whole_story() {
        let mut ra = result(vec![effect(TARGET_ENNEMY, HP, -30, 1)]);
        ra.all_dodging = vec![DodgeInfo {
            name: "boss".to_owned(),
            is_dodging: true,
            is_blocking: false,
        }];
        assert_eq!(classify_attack(&ra), vec![Sfx::Dodge]);
    }

    #[test]
    fn an_instant_mana_restore_stays_a_heal() {
        assert_eq!(
            classify_attack(&result(vec![effect(TARGET_ALLY, "Mana", 30, 1)])),
            vec![Sfx::Heal]
        );
    }

    #[test]
    fn an_attack_that_landed_nothing_stays_silent() {
        assert_eq!(classify_attack(&result(vec![])), Vec::new());
    }
}
