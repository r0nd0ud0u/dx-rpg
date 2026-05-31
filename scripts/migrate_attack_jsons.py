#!/usr/bin/env python3
"""
Migrate attack JSON files: update Buffer.kind/value/stats-name/is-percent
from legacy Type/Stat/Value fields for all effects where Type is non-empty.
"""
import json
import os
import sys

ATTACK_DIRS = [
    r"C:\Skysoft-ATM\git\perso\dx-rpg\offlines\attack",
    r"C:\Skysoft-ATM\git\perso\lib-rpg\offlines\attack",
]

# is-percent = true for these kinds
PERCENT_KINDS = {
    "ChangeMaxStatByPercentage",
    "ChangeCurrentStatByPercentage",
    "DamageRxPercent",
    "HealTxPercent",
    "HealRxPercent",
    "DamageTxPercent",
    "BoostHotsByPercentage",
    "BoostBufByHotsNumberInPercentage",
    "PercentageIntoDamages",
}

# Map legacy Type strings to new Buffer.kind strings
TYPE_TO_KIND = {
    "ChangeCurrentStatByValue": "ChangeCurrentStatByValue",
    "ChangeMaxStatByPercentage": "ChangeMaxStatByPercentage",
    "ChangeMaxStatByValue": "ChangeMaxStatByValue",
    "ChangeCurrentStatByPercentage": "ChangeCurrentStatByPercentage",
    "CooldownTurnsNumber": "CooldownTurnsNumber",
    "DecreasingRateOnTurn": "DecreasingRateOnTurn",
    "DamageRxPercent": "DamageRxPercent",
    "BlockHealAtk": "BlockHealAtk",
    "ReinitBuf": "ReinitBuf",
    "RemoveOneDebuf": "RemoveOneDebuf",
    "AddAsMuchAsHp": "AddAsMuchAsHp",
    "RepeatAsManyAsPossible": "RepeatAsManyAsPossible",
    "PercentageIntoDamages": "PercentageIntoDamages",
    "BoostHotsByPercentage": "BoostHotsByPercentage",
    "BoostBufByHotsNumberInPercentage": "BoostBufByHotsNumberInPercentage",
    # legacy French names
    "Up/down heal TX en %": "HealTxPercent",
    "Buf multi": "MultiValue",
    "Active effet si Dégâts au tour précédent": "ConditionDamagePrevTurn",
    "Dégâts au tour précédent": "IsDamageTxHealNeedyAlly",
    "Répète l'attaque(en % de chance) après heal tour prec.": "RepeatIfHeal",
}

# For these kinds the stats-name comes from the Stat field
USES_STAT_NAME = {
    "ChangeCurrentStatByValue",
    "ChangeMaxStatByPercentage",
    "ChangeMaxStatByValue",
    "ChangeCurrentStatByPercentage",
    "ReinitBuf",
    "AddAsMuchAsHp",
    "PercentageIntoDamages",
    "DecreasingRateOnTurn",
    "IsDamageTxHealNeedyAlly",
    "RepeatAsManyAsPossible",
}

# For these kinds the value comes from "Value" field
USES_VALUE_FIELD = {
    "ChangeCurrentStatByValue",
    "ChangeMaxStatByPercentage",
    "ChangeMaxStatByValue",
    "ChangeCurrentStatByPercentage",
    "DamageRxPercent",
    "HealTxPercent",
    "HealRxPercent",
    "DamageTxPercent",
    "BoostHotsByPercentage",
    "BoostBufByHotsNumberInPercentage",
    "MultiValue",
    "DecreasingRateOnTurn",
    "RepeatAsManyAsPossible",
    "RepeatIfHeal",
}

# For these kinds the value comes from "Valeur de l'effet" (sub_value_effect)
USES_SUB_VALUE = {
    "RemoveOneDebuf",
    "PercentageIntoDamages",
    "IsDamageTxHealNeedyAlly",
}

# For these kinds the value should be 0 (value not applicable)
ZERO_VALUE_KINDS = {
    "CooldownTurnsNumber",
    "BlockHealAtk",
    "ReinitBuf",
    "AddAsMuchAsHp",
    "ConditionDamagePrevTurn",
}


def migrate_effect(effect: dict) -> dict:
    """
    Given an effect dict, update Buffer if Type is non-empty.
    Returns modified effect dict.
    """
    typ = effect.get("Type", "").strip()

    # Empty Type: keep existing Buffer unchanged (already manually correct)
    if not typ:
        return effect

    # Look up the kind
    kind = TYPE_TO_KIND.get(typ)
    if kind is None:
        print(f"  WARNING: Unknown Type '{typ}' – keeping Buffer unchanged", file=sys.stderr)
        return effect

    stat = effect.get("Stat", "")
    value_field = effect.get("Value", 0)
    sub_value = effect.get("Valeur de l'effet", 0)
    nb_turns = effect.get("Tours actifs", 1)

    # Determine stats-name
    if kind in USES_STAT_NAME:
        stats_name = stat
    else:
        stats_name = ""

    # Determine value
    if kind in ZERO_VALUE_KINDS:
        value = 0
    elif kind in USES_SUB_VALUE:
        value = sub_value
    elif kind in USES_VALUE_FIELD:
        value = value_field
    else:
        value = value_field  # default fallback

    # Determine is-percent
    is_percent = kind in PERCENT_KINDS

    effect["Buffer"] = {
        "kind": kind,
        "value": value,
        "is-percent": is_percent,
        "stats-name": stats_name,
        "passive-enabled": effect.get("Buffer", {}).get("passive-enabled", False),
    }

    return effect


def migrate_file(path: str) -> bool:
    """Migrate one JSON file. Returns True if file was changed."""
    with open(path, encoding="utf-8") as f:
        data = json.load(f)

    if "Effet" not in data:
        return False

    original = json.dumps(data, ensure_ascii=False)
    for i, effect in enumerate(data["Effet"]):
        data["Effet"][i] = migrate_effect(effect)

    updated = json.dumps(data, ensure_ascii=False, indent=4)
    if updated == original:
        return False

    with open(path, "w", encoding="utf-8") as f:
        f.write(updated)
        f.write("\n")
    return True


def process_directory(attack_dir: str) -> None:
    if not os.path.isdir(attack_dir):
        print(f"Directory not found: {attack_dir}", file=sys.stderr)
        return

    changed = 0
    total = 0
    for char_dir in sorted(os.listdir(attack_dir)):
        char_path = os.path.join(attack_dir, char_dir)
        if not os.path.isdir(char_path):
            continue
        for fname in sorted(os.listdir(char_path)):
            if not fname.endswith(".json"):
                continue
            fpath = os.path.join(char_path, fname)
            total += 1
            modified = migrate_file(fpath)
            if modified:
                changed += 1
                print(f"  Updated: {char_dir}/{fname}")

    print(f"\n{attack_dir}: {changed}/{total} files updated.")


if __name__ == "__main__":
    for d in ATTACK_DIRS:
        print(f"\nProcessing: {d}")
        process_directory(d)

print("\nDone.")
