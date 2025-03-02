pub fn update_character_life(life: i32) -> i32 {
    (life - 20).min(0)
}
