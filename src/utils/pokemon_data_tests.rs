/// Validates that all Pokemon universe JSON data files are well-formed
/// and contain the required fields expected by lib_rpg's DataManager.
#[cfg(test)]
mod tests {
    use serde_json::Value;
    use std::fs;

    const OFFLINES: &str = "offlines";

    fn load_json(path: &str) -> Value {
        let content =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("Cannot read {path}: {e}"));
        serde_json::from_str(&content).unwrap_or_else(|e| panic!("Invalid JSON in {path}: {e}"))
    }

    // ── Character files ───────────────────────────────────────────────────────

    fn assert_character(path: &str, expected_type: &str, expected_class: &str) {
        let v = load_json(path);
        assert_eq!(
            v["Type"].as_str().unwrap_or(""),
            expected_type,
            "{path} Type"
        );
        assert_eq!(
            v["Class"].as_str().unwrap_or(""),
            expected_class,
            "{path} Class"
        );
        assert!(
            !v["Name"].as_str().unwrap_or("").is_empty(),
            "{path} Name must be set"
        );
        assert!(
            v["Stats"]["HP"]["Max"].as_f64().unwrap_or(0.0) > 0.0,
            "{path} HP.Max must be > 0"
        );
    }

    #[test]
    fn bulbasaur_hero_json_valid() {
        assert_character(
            &format!("{OFFLINES}/characters/pokemon/Bulbasaur.json"),
            "Hero",
            "Mage",
        );
    }

    #[test]
    fn charmander_hero_json_valid() {
        assert_character(
            &format!("{OFFLINES}/characters/pokemon/Charmander.json"),
            "Hero",
            "Berserker",
        );
    }

    #[test]
    fn squirtle_hero_json_valid() {
        assert_character(
            &format!("{OFFLINES}/characters/pokemon/Squirtle.json"),
            "Hero",
            "Warrior",
        );
    }

    #[test]
    fn boss_characters_json_valid() {
        let bosses = [
            "Rattata",
            "Pidgey",
            "Mankey",
            "Machoke",
            "Gengar",
            "Haunter",
            "Dragonite",
            "Mewtwo",
            "Mewtwo Armure",
        ];
        for boss in &bosses {
            assert_character(
                &format!("{OFFLINES}/characters/pokemon/{boss}.json"),
                "Boss",
                "Warrior",
            );
        }
    }

    // ── Attack files ──────────────────────────────────────────────────────────

    fn assert_attack(path: &str) {
        let v = load_json(path);
        assert!(
            !v["Nom"].as_str().unwrap_or("").is_empty(),
            "{path} Nom must be set"
        );
        assert!(v["Effet"].is_array(), "{path} Effet must be an array");
        assert!(
            !v["Effet"].as_array().unwrap().is_empty(),
            "{path} Effet must not be empty"
        );
    }

    #[test]
    fn bulbasaur_attacks_valid() {
        for atk in &[
            "SimpleAtk",
            "Vine Whip",
            "Leech Seed",
            "Razor Leaf",
            "Synthesis",
            "Solar Beam",
        ] {
            assert_attack(&format!("{OFFLINES}/attack/Bulbasaur/{atk}.json"));
        }
    }

    #[test]
    fn charmander_attacks_valid() {
        for atk in &[
            "Charge",
            "Ember",
            "Fire Spin",
            "Flamethrower",
            "Dragon Rage",
            "Fire Blast",
        ] {
            assert_attack(&format!("{OFFLINES}/attack/Charmander/{atk}.json"));
        }
    }

    #[test]
    fn squirtle_attacks_valid() {
        for atk in &[
            "Charge",
            "Water Gun",
            "Bubble Beam",
            "Withdraw",
            "Surf",
            "Ice Beam",
        ] {
            assert_attack(&format!("{OFFLINES}/attack/Squirtle/{atk}.json"));
        }
    }

    #[test]
    fn boss_attacks_valid() {
        let boss_attacks: &[(&str, &[&str])] = &[
            ("Rattata", &["Charge", "Quick Attack"]),
            ("Pidgey", &["Charge", "Gust"]),
            ("Mankey", &["Charge", "Low Kick", "Cross Chop"]),
            ("Machoke", &["Charge", "Karate Chop", "Submission"]),
            ("Gengar", &["Charge", "Shadow Ball", "Hex"]),
            ("Haunter", &["Charge", "Lick", "Night Shade"]),
            (
                "Dragonite",
                &["Charge", "Wing Attack", "Dragon Rage", "Hyper Beam"],
            ),
            ("Mewtwo", &["Charge", "Psychic", "Aura Sphere", "Psystrike"]),
            (
                "Mewtwo Armure",
                &["Charge", "Psystrike", "Barrier", "Shadow Storm"],
            ),
        ];
        for (boss, atks) in boss_attacks {
            for atk in *atks {
                assert_attack(&format!("{OFFLINES}/attack/{boss}/{atk}.json"));
            }
        }
    }

    // ── Scenario files ────────────────────────────────────────────────────────

    #[test]
    fn pokemon_scenarios_valid() {
        for stage in 1..=10 {
            let path = format!("{OFFLINES}/scenarios/pokemon/stage_{stage}.json");
            let v = load_json(&path);
            assert!(
                !v["name"].as_str().unwrap_or("").is_empty(),
                "{path} name must be set"
            );
            assert!(
                v["level"].as_u64().unwrap_or(0) == stage as u64,
                "{path} level must be {stage}"
            );
            assert!(
                v["boss_patterns"].is_object(),
                "{path} boss_patterns must be an object"
            );
            assert!(
                !v["boss_patterns"].as_object().unwrap().is_empty(),
                "{path} boss_patterns must not be empty"
            );
        }
    }
}
