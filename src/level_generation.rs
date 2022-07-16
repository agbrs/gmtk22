use agb::rng;

use crate::battle::EnemyAttack;

pub struct GeneratedAttack {
    pub attack: EnemyAttack,
    pub cooldown: u32,
}

pub fn generate_attack(current_level: u32) -> Option<GeneratedAttack> {
    if (rng::gen().rem_euclid(128) as u32) < current_level * 2 {
        Some(GeneratedAttack {
            attack: generate_enemy_attack(current_level),
            cooldown: generate_cooldown(current_level),
        })
    } else {
        None
    }
}

fn generate_enemy_attack(_current_level: u32) -> EnemyAttack {
    let attack_id = rng::gen().rem_euclid(8) as u32;

    if attack_id < 3 {
        EnemyAttack::Shoot(rng::gen().rem_euclid(8) as u32 + 2)
    } else if attack_id < 5 {
        EnemyAttack::Shield(rng::gen().rem_euclid(3) as u32 + 1)
    } else {
        EnemyAttack::Heal(rng::gen().rem_euclid(8) as u32)
    }
}

fn generate_cooldown(_current_level: u32) -> u32 {
    rng::gen().rem_euclid(128) as u32 + 2 * 60
}
