use agb::rng;

use crate::battle::EnemyAttack;

pub struct GeneratedAttack {
    pub attack: EnemyAttack,
    pub cooldown: u32,
}

pub fn generate_attack(current_level: u32) -> Option<GeneratedAttack> {
    if (rng::gen().rem_euclid(1024) as u32) < current_level * 2 {
        Some(GeneratedAttack {
            attack: generate_enemy_attack(current_level),
            cooldown: generate_cooldown(current_level),
        })
    } else {
        None
    }
}

fn generate_enemy_attack(current_level: u32) -> EnemyAttack {
    let attack_id = rng::gen().rem_euclid(10) as u32;

    if attack_id < 4 {
        EnemyAttack::Shoot(rng::gen().rem_euclid(((current_level + 2) / 3) as i32) as u32 + 1)
    } else if attack_id < 9 {
        EnemyAttack::Shield(rng::gen().rem_euclid(((current_level + 4) / 5) as i32) as u32 + 1)
    } else {
        EnemyAttack::Heal(rng::gen().rem_euclid(((current_level + 1) / 2) as i32) as u32)
    }
}

fn generate_cooldown(_current_level: u32) -> u32 {
    rng::gen().rem_euclid(128) as u32 + 2 * 60
}
