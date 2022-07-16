use agb::display::object::{Object, ObjectController};
use alloc::vec::Vec;

use crate::{
    graphics::{HealthBar, FACE_SPRITES, SHIP_SPRITES},
    Ship,
};

use super::{CurrentBattleState, MALFUNCTION_COOLDOWN_FRAMES};

pub struct BattleScreenDisplay<'a> {
    dice: Vec<Object<'a>>,
    dice_cooldowns: Vec<HealthBar<'a>>,
    player_shield: Vec<Object<'a>>,
    enemy_shield: Vec<Object<'a>>,
}

impl<'a> BattleScreenDisplay<'a> {
    pub fn new(obj: &'a ObjectController, current_battle_state: &CurrentBattleState) -> Self {
        let player_x = 12;
        let player_y = 8;
        let enemy_x = 167;

        let dice: Vec<_> = current_battle_state
            .rolled_dice
            .faces_to_render()
            .enumerate()
            .map(|(i, (face, _))| {
                let mut die_obj = obj.object(obj.sprite(FACE_SPRITES.sprite_for_face(face)));

                die_obj.set_y(120).set_x(i as u16 * 40 + 28).show();

                die_obj
            })
            .collect();

        let dice_cooldowns: Vec<_> = dice
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let mut cooldown_bar =
                    HealthBar::new((i as i32 * 40 + 28, 120 - 8).into(), 24, obj);
                cooldown_bar.hide();
                cooldown_bar
            })
            .collect();

        let shield_sprite = SHIP_SPRITES.sprite_for_ship(Ship::Shield);

        let player_shield: Vec<_> = (0..5)
            .into_iter()
            .map(|i| {
                let mut shield_obj = obj.object(obj.sprite(shield_sprite));
                shield_obj
                    .set_x(player_x + 18 + 11 * i)
                    .set_y(player_y)
                    .hide();

                shield_obj
            })
            .collect();

        let enemy_shield: Vec<_> = (0..5)
            .into_iter()
            .map(|i| {
                let mut shield_obj = obj.object(obj.sprite(shield_sprite));
                shield_obj
                    .set_x(enemy_x - 16 - 11 * i)
                    .set_y(player_y)
                    .set_hflip(true)
                    .hide();

                shield_obj
            })
            .collect();

        Self {
            dice,
            dice_cooldowns,
            player_shield,
            enemy_shield,
        }
    }

    pub fn update(&mut self, obj: &'a ObjectController, current_battle_state: &CurrentBattleState) {
        // update the dice display to display the current values
        for ((die_obj, (current_face, cooldown)), cooldown_healthbar) in self
            .dice
            .iter_mut()
            .zip(current_battle_state.rolled_dice.faces_to_render())
            .zip(self.dice_cooldowns.iter_mut())
        {
            die_obj.set_sprite(obj.sprite(FACE_SPRITES.sprite_for_face(current_face)));

            if let Some(cooldown) = cooldown {
                cooldown_healthbar
                    .set_value((cooldown * 24 / MALFUNCTION_COOLDOWN_FRAMES) as usize, obj);
                cooldown_healthbar.show();
            } else {
                cooldown_healthbar.hide();
            }
        }

        for (i, player_shield) in self.player_shield.iter_mut().enumerate() {
            if i < current_battle_state.player.shield_count as usize {
                player_shield.show();
            } else {
                player_shield.hide();
            }
        }

        for (i, player_shield) in self.enemy_shield.iter_mut().enumerate() {
            if i < current_battle_state.enemy.shield_count as usize {
                player_shield.show();
            } else {
                player_shield.hide();
            }
        }
    }
}
