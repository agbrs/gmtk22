use crate::{Agb, Face, PlayerDice, ShipSprites, SHIP_SPRITES};
use agb::hash_map::HashMap;
use alloc::vec::Vec;

/// A face of the rolled die and it's cooldown (should it be a malfunction)
#[derive(Debug)]

struct RolledDie {
    face: Face,
    cooldown: u32,
}

impl RolledDie {
    fn update(&mut self) {
        self.cooldown = self.cooldown.wrapping_sub(1)
    }

    fn can_reroll(&self) -> bool {
        self.face != Face::Malfunction || self.cooldown == 0
    }
}

#[derive(Debug)]
struct RolledDice {
    rolls: Vec<RolledDie>,
}

impl RolledDice {
    fn update(&mut self) {
        self.rolls.iter_mut().for_each(RolledDie::update);
    }
}

#[derive(Debug)]
struct PlayerState {
    shield_count: u32,
    health: u32,
}

#[derive(Debug)]
struct EnemyState {}

#[derive(Debug)]
struct CurrentBattleState {
    player: PlayerState,
    enemy: EnemyState,
    rolled_dice: RolledDice,
}

impl CurrentBattleState {
    fn accept_rolls(&mut self) {
        let rolls = &self.rolled_dice.rolls;
        let mut face_counts: HashMap<Face, u32> = HashMap::new();
        for f in rolls {
            *face_counts.entry(f.face).or_default() += 1;
        }

        // shield
        let shield = face_counts.entry(Face::Shield).or_default();
        if self.player.shield_count <= *shield {
            self.player.shield_count += 1;
        }
    }
}

pub(crate) fn battle_screen(agb: &mut Agb, player_dice: PlayerDice) {
    let player_sprite = SHIP_SPRITES.sprites[0];
    let enemy_sprite = SHIP_SPRITES.sprites[1];

    let mut player_obj = agb.obj.object(agb.obj.sprite(player_sprite));
    let mut enemy_obj = agb.obj.object(agb.obj.sprite(enemy_sprite));

    player_obj.set_x(27).set_y(16).set_z(1).show();
    enemy_obj.set_x(167).set_y(16).set_z(1).show();

    loop {
        agb.star_background.update();
        agb.vblank.wait_for_vblank();
        agb.obj.commit();
        agb.star_background.commit(&mut agb.vram);
    }
}
