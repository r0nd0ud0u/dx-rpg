//! Which sound effect a game event should play.
//!
//! An attack sounds like what it costs to cast: mana, vigour, berserk or nothing each
//! get their own impact ([`Sfx::Arcane`], [`Sfx::Heavy`], [`Sfx::Rage`], [`Sfx::Strike`]),
//! so one attack always sounds the same. A crit layers an accent over its family rather
//! than replacing it. Healing overrides the family.
//!
//! [`lib_rpg::common::sound_cue::classify_result_atk`] owns the dodge/block priority and
//! is still called for it, but it only reads HP deltas — so pure support casts (no HP
//! effect at all) return no cue, and lasting regens return the same blip as an instant
//! cure. Both are re-classified here from the effect parameters. Client-side because it is
//! a presentation decision; can move upstream unchanged if another front-end needs it.

use lib_rpg::{
    character_mod::{
        buffers::BufKinds,
        effect::{EffectParam, is_effet_hot_or_dot},
    },
    common::{
        constants::{all_target_const::TARGET_ENNEMY, stats_const::HP},
        sound_cue::{SoundCue, classify_result_atk},
    },
    server::{game_manager::ResultLaunchAttack, players_manager::GameAtkEffect},
};

/// Every sound the client can play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    /// Damage from an attack that costs nothing — the basic strike ("Charge").
    Strike,
    /// Damage from a mana-cost attack.
    Arcane,
    /// Damage from a vigour-cost attack.
    Heavy,
    /// Damage from a berserk-cost attack.
    Rage,
    /// Layered over a family sound on a crit; carries no low end (see gen_sfx.py).
    CriticalHit,
    Dodge,
    Block,
    /// An instant cure.
    Heal,
    /// A lasting boon on an ally — regen, shield, war cry.
    Buff,
    /// A lasting affliction on an enemy, with no damage of its own.
    Debuff,
    Potion,
    Victory,
    GameOver,
}

/// Sounds to play, most-significant first. Several can come back at once — the caller
/// plays them together, and a crit always comes back with the family it accents.
pub fn classify_attack(ra: &ResultLaunchAttack) -> Vec<Sfx> {
    // lib-rpg owns this rule: a dodged/blocked attack didn't connect.
    let outcome = classify_result_atk(ra);
    if outcome.contains(&SoundCue::Dodge) {
        return vec![Sfx::Dodge];
    }
    if outcome.contains(&SoundCue::Block) {
        return vec![Sfx::Block];
    }

    let effects = &ra.new_game_atk_effects;
    let damaged = effects.iter().any(is_damage);
    let restored = effects.iter().any(is_restore);

    // Nothing damaged and nothing restored, but effects did land: a support cast.
    // Its polarity is decided by who the landed effects were aimed at.
    if !damaged && !restored {
        if effects.is_empty() {
            return Vec::new();
        }
        return if effects.iter().any(is_aimed_at_enemy) {
            vec![Sfx::Debuff]
        } else {
            vec![Sfx::Buff]
        };
    }

    let mut cues = Vec::new();
    if damaged {
        cues.push(damage_family(ra));
    }
    if restored {
        cues.push(restore_cue(effects));
    }
    // Damage only: heals crit too, and an impact accent over a regen chime just sounds
    // like a different spell.
    if damaged && effects.iter().any(|e| e.effect_outcome.is_critical) {
        cues.push(Sfx::CriticalHit);
    }
    cues
}

/// Impact sound for an attack, by the resource it charges. Nothing in the game data
/// charges two at once; if it ever does, the most distinctive family wins.
fn damage_family(ra: &ResultLaunchAttack) -> Sfx {
    // Every effect of one attack carries the same `atk_type`, so the first is enough.
    let Some(atk) = ra.new_game_atk_effects.first().map(|e| &e.atk_type) else {
        return Sfx::Strike;
    };
    if atk.berseck_cost > 0 {
        Sfx::Rage
    } else if atk.mana_cost > 0 {
        Sfx::Arcane
    } else if atk.vigor_cost > 0 {
        Sfx::Heavy
    } else {
        Sfx::Strike
    }
}

/// A restore spread over turns is a regen, not a cure.
fn restore_cue(effects: &[GameAtkEffect]) -> Sfx {
    if effects.iter().filter(|e| is_restore(e)).all(is_lasting) {
        Sfx::Buff
    } else {
        Sfx::Heal
    }
}

fn effect_param(effect: &GameAtkEffect) -> &EffectParam {
    &effect.processed_effect_param.input_effect_param
}

fn is_aimed_at_enemy(effect: &GameAtkEffect) -> bool {
    effect_param(effect).target_kind == TARGET_ENNEMY
}

/// Which way an effect moves current HP: negative damages, positive heals, `None` for
/// anything else.
///
/// Read from the effect's parameters, not from what landed: an HP cap or full armour
/// absorption zeroes `real_amount_tx`, and an attack must still sound like itself.
/// HP-only on purpose — aggro generation and resource refunds (`Fracas des Abysses`
/// grants its caster +20 Vigor) also report positive amounts and are not cures.
fn hp_direction(effect: &GameAtkEffect) -> Option<i64> {
    let buffer = &effect_param(effect).buffer;
    // lib-rpg's list of kinds that move a *current* stat; ChangeMaxStat on HP is a buff.
    if buffer.stats_name != HP || !is_effet_hot_or_dot(&buffer.kind) {
        return None;
    }
    // Some effects carry their amount outside `value`; fall back to what landed.
    Some(if buffer.value != 0 {
        buffer.value
    } else {
        effect.effect_outcome.real_amount_tx
    })
}

fn is_damage(effect: &GameAtkEffect) -> bool {
    hp_direction(effect).is_some_and(|amount| amount < 0)
}

fn is_restore(effect: &GameAtkEffect) -> bool {
    hp_direction(effect).is_some_and(|amount| amount > 0)
}

/// Keeps applying after the turn it was cast on — lib-rpg's regen/DoT shape.
fn is_lasting(effect: &GameAtkEffect) -> bool {
    let param = effect_param(effect);
    param.nb_turns > 1 && param.buffer.kind == BufKinds::ChangeCurrentStat
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_rpg::{
        character_mod::{
            attack_type::AttackType,
            buffers::Buffer,
            effect::{EffectOutcome, EffectParam, ProcessedEffectParam},
        },
        common::constants::{
            all_target_const::{TARGET_ALLY, TARGET_HIMSELF},
            stats_const::HP,
        },
        server::players_manager::DodgeInfo,
    };

    #[derive(Clone, Copy, Default)]
    struct Cost {
        mana: u64,
        vigor: u64,
        berseck: u64,
    }

    const FREE: Cost = Cost {
        mana: 0,
        vigor: 0,
        berseck: 0,
    };

    /// One landed effect of an attack costing `cost`: `stat` moved by
    /// `real_amount_tx` on `target_kind`, lasting `nb_turns` turns.
    fn effect(
        cost: Cost,
        target_kind: &str,
        stat: &str,
        real_amount_tx: i64,
        nb_turns: i64,
    ) -> GameAtkEffect {
        GameAtkEffect {
            atk_type: AttackType {
                mana_cost: cost.mana,
                vigor_cost: cost.vigor,
                berseck_cost: cost.berseck,
                ..Default::default()
            },
            processed_effect_param: ProcessedEffectParam {
                input_effect_param: EffectParam {
                    target_kind: target_kind.to_owned(),
                    nb_turns,
                    buffer: Buffer {
                        stats_name: stat.to_owned(),
                        kind: BufKinds::ChangeCurrentStat,
                        // Mirrors the real data, where the effect's own value and
                        // the amount it lands carry the same sign.
                        value: real_amount_tx,
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

    /// A hit sounds like what the attack charged for it — the whole point of the
    /// family split, so that one attack always sounds like itself.
    #[test]
    fn damage_sounds_like_the_resource_it_cost() {
        let damage = |cost| classify_attack(&result(vec![effect(cost, TARGET_ENNEMY, HP, -30, 1)]));
        assert_eq!(damage(FREE), vec![Sfx::Strike]);
        assert_eq!(damage(Cost { mana: 8, ..FREE }), vec![Sfx::Arcane]);
        assert_eq!(damage(Cost { vigor: 5, ..FREE }), vec![Sfx::Heavy]);
        assert_eq!(
            damage(Cost {
                berseck: 12,
                ..FREE
            }),
            vec![Sfx::Rage]
        );
    }

    #[test]
    fn a_critical_accents_its_family_rather_than_replacing_it() {
        let mut crit = effect(Cost { vigor: 5, ..FREE }, TARGET_ENNEMY, HP, -60, 1);
        crit.effect_outcome.is_critical = true;
        assert_eq!(
            classify_attack(&result(vec![crit])),
            vec![Sfx::Heavy, Sfx::CriticalHit]
        );
    }

    /// Healing outranks the cost family: a cast that gives HP back sounds like a
    /// cure even though it charged mana.
    #[test]
    fn an_instant_cure_sounds_like_a_heal_not_like_its_cost() {
        assert_eq!(
            classify_attack(&result(vec![effect(
                Cost { mana: 10, ..FREE },
                TARGET_ALLY,
                HP,
                30,
                1
            )])),
            vec![Sfx::Heal]
        );
    }

    #[test]
    fn lasting_regen_becomes_a_buff() {
        // Essence Régénératrice: +20 HP a turn for five turns.
        assert_eq!(
            classify_attack(&result(vec![effect(
                Cost { mana: 8, ..FREE },
                TARGET_ALLY,
                HP,
                20,
                5
            )])),
            vec![Sfx::Buff]
        );
    }

    /// Heals crit too, and Thalia's Essence Régénératrice does so often enough to
    /// notice: the accent must not turn one cast of an attack into a different
    /// sound from the next.
    #[test]
    fn a_critical_heal_gets_no_impact_accent() {
        let mut crit = effect(Cost { mana: 8, ..FREE }, TARGET_ALLY, HP, 20, 5);
        crit.effect_outcome.is_critical = true;
        assert_eq!(classify_attack(&result(vec![crit])), vec![Sfx::Buff]);
    }

    /// Only HP counts as healing: a mana refund is a support effect, and gets the
    /// buff chime rather than the cure.
    #[test]
    fn a_mana_restore_is_not_a_heal() {
        assert_eq!(
            classify_attack(&result(vec![effect(FREE, TARGET_ALLY, "Mana", 30, 1)])),
            vec![Sfx::Buff]
        );
    }

    #[test]
    fn stat_only_self_cast_becomes_a_buff() {
        // Barrier / Withdraw / Protection naine: no HP effect at all, so lib-rpg
        // classifies nothing and the cast used to be silent.
        assert_eq!(
            classify_attack(&result(vec![
                effect(FREE, TARGET_HIMSELF, "Physical armor", 0, 3),
                effect(FREE, TARGET_HIMSELF, "Magic armor", 0, 3),
            ])),
            vec![Sfx::Buff]
        );
    }

    #[test]
    fn stat_only_enemy_cast_becomes_a_debuff() {
        assert_eq!(
            classify_attack(&result(vec![effect(FREE, TARGET_ENNEMY, "Dodge", 0, 3)])),
            vec![Sfx::Debuff]
        );
    }

    #[test]
    fn an_attack_that_both_damages_and_regenerates_gets_both_cues() {
        let cues = classify_attack(&result(vec![
            effect(Cost { vigor: 4, ..FREE }, TARGET_ENNEMY, HP, -25, 1),
            effect(Cost { vigor: 4, ..FREE }, TARGET_ALLY, HP, 20, 5),
        ]));
        assert_eq!(cues, vec![Sfx::Heavy, Sfx::Buff]);
    }

    /// A blow's side effects report positive amounts just like a heal does — the
    /// aggro it generates, and the vigour `Fracas des Abysses` refunds to its
    /// caster. Caught against real game data, where it made a sword blow classify
    /// as a cure.
    #[test]
    fn the_side_effects_of_a_blow_do_not_make_it_a_heal() {
        let cost = Cost { vigor: 13, ..FREE };
        let cues = classify_attack(&result(vec![
            effect(cost, TARGET_ENNEMY, HP, -75, 1),
            effect(cost, TARGET_ENNEMY, "Dodge", -5, 3),
            effect(cost, TARGET_HIMSELF, "Vigor", 20, 1),
            effect(cost, TARGET_HIMSELF, "Aggro", 25, 1),
        ]));
        assert_eq!(cues, vec![Sfx::Heavy]);
    }

    /// Healing an ally who is already at full HP lands nothing — the HP cap eats
    /// it — but it is still the same cure and must still sound like one.
    #[test]
    fn a_cure_that_lands_nothing_still_sounds_like_a_cure() {
        let mut capped = effect(Cost { mana: 10, ..FREE }, TARGET_ALLY, HP, 40, 1);
        capped.effect_outcome.real_amount_tx = 0;
        assert_eq!(classify_attack(&result(vec![capped])), vec![Sfx::Heal]);
    }

    #[test]
    fn dodge_is_the_whole_story() {
        let mut ra = result(vec![effect(FREE, TARGET_ENNEMY, HP, -30, 1)]);
        ra.all_dodging = vec![DodgeInfo {
            name: "boss".to_owned(),
            is_dodging: true,
            is_blocking: false,
        }];
        assert_eq!(classify_attack(&ra), vec![Sfx::Dodge]);
    }

    #[test]
    fn an_attack_that_landed_nothing_stays_silent() {
        assert_eq!(classify_attack(&result(vec![])), Vec::new());
    }
}
