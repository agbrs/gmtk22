use agb::display::object::{Object, ObjectController};
use alloc::vec;
use alloc::vec::Vec;

use crate::{
    graphics::{
        FractionDisplay, HealthBar, NumberDisplay, ENEMY_ATTACK_SPRITES, FACE_SPRITES, SHIP_SPRITES,
    },
    EnemyAttackType, Ship,
};

use super::{CurrentBattleState, EnemyAttackState, MALFUNCTION_COOLDOWN_FRAMES};

pub struct BattleScreenDisplay<'a> {
    dice: Vec<Object<'a>>,
    dice_cooldowns: Vec<HealthBar<'a>>,
    player_shield: Vec<Object<'a>>,
    enemy_shield: Vec<Object<'a>>,

    player_healthbar: HealthBar<'a>,
    enemy_healthbar: HealthBar<'a>,
    player_health: FractionDisplay<'a>,
    enemy_health: FractionDisplay<'a>,

    enemy_attack_display: Vec<EnemyAttackDisplay<'a>>,

    _misc_sprites: Vec<Object<'a>>,
}

const HEALTH_BAR_WIDTH: usize = 48;

impl<'a> BattleScreenDisplay<'a> {
    pub fn new(obj: &'a ObjectController, current_battle_state: &CurrentBattleState) -> Self {
        let mut misc_sprites = vec![];
        let player_x = 12;
        let player_y = 8;
        let enemy_x = 167;

        let player_sprite = SHIP_SPRITES.sprite_for_ship(Ship::Player);
        let enemy_sprite = SHIP_SPRITES.sprite_for_ship(Ship::Drone);

        let mut player_obj = obj.object(obj.sprite(player_sprite));
        let mut enemy_obj = obj.object(obj.sprite(enemy_sprite));

        player_obj.set_x(player_x).set_y(player_y).set_z(1).show();
        enemy_obj.set_x(enemy_x).set_y(player_y).set_z(1).show();

        misc_sprites.push(player_obj);
        misc_sprites.push(enemy_obj);

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

        let player_healthbar_x = 18;
        let enemy_healthbar_x = 180;
        let player_healthbar = HealthBar::new(
            (player_healthbar_x, player_y - 8).into(),
            HEALTH_BAR_WIDTH,
            obj,
        );
        let enemy_healthbar = HealthBar::new(
            (enemy_healthbar_x, player_y - 8).into(),
            HEALTH_BAR_WIDTH,
            obj,
        );

        let player_health_display = FractionDisplay::new(
            (
                player_healthbar_x + HEALTH_BAR_WIDTH as u16 / 2 - 16,
                player_y,
            )
                .into(),
            3,
            obj,
        );
        let enemy_health_display = FractionDisplay::new(
            (
                enemy_healthbar_x + HEALTH_BAR_WIDTH as u16 / 2 - 16,
                player_y,
            )
                .into(),
            3,
            obj,
        );

        let enemy_attack_display = (0..2)
            .into_iter()
            .map(|i| {
                let mut attack_obj = obj.object(
                    obj.sprite(ENEMY_ATTACK_SPRITES.sprite_for_attack(EnemyAttackType::Attack)),
                );

                let attack_obj_position = (120, 56 + 32 * i).into();
                attack_obj.set_position(attack_obj_position).hide();

                let mut attack_cooldown =
                    HealthBar::new(attack_obj_position + (32, 8).into(), 48, obj);
                attack_cooldown.hide();

                let attack_number_display =
                    NumberDisplay::new(attack_obj_position - (8, -10).into());

                EnemyAttackDisplay::new(attack_obj, attack_cooldown, attack_number_display)
            })
            .collect();

        Self {
            dice,
            dice_cooldowns,
            player_shield,
            enemy_shield,

            player_healthbar,
            enemy_healthbar,
            player_health: player_health_display,
            enemy_health: enemy_health_display,

            enemy_attack_display,
            _misc_sprites: misc_sprites,
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

        self.player_healthbar.set_value(
            ((current_battle_state.player.health * HEALTH_BAR_WIDTH as u32)
                / current_battle_state.player.max_health) as usize,
            obj,
        );

        self.enemy_healthbar.set_value(
            ((current_battle_state.enemy.health * HEALTH_BAR_WIDTH as u32)
                / current_battle_state.enemy.max_health) as usize,
            obj,
        );

        self.player_health.set_value(
            current_battle_state.player.health as usize,
            current_battle_state.player.max_health as usize,
            obj,
        );

        self.enemy_health.set_value(
            current_battle_state.enemy.health as usize,
            current_battle_state.enemy.max_health as usize,
            obj,
        );

        for (i, attack) in current_battle_state.attacks.iter().enumerate() {
            self.enemy_attack_display[i].update(attack, obj);
        }
    }
}

struct EnemyAttackDisplay<'a> {
    face: Object<'a>,
    cooldown: HealthBar<'a>,
    number: NumberDisplay<'a>,
}

impl<'a> EnemyAttackDisplay<'a> {
    pub fn new(face: Object<'a>, cooldown: HealthBar<'a>, number: NumberDisplay<'a>) -> Self {
        Self {
            face,
            cooldown,
            number,
        }
    }

    pub fn update(&mut self, attack: &Option<EnemyAttackState>, obj: &'a ObjectController) {
        if let Some(attack) = attack {
            self.face.show().set_sprite(
                obj.sprite(ENEMY_ATTACK_SPRITES.sprite_for_attack(attack.attack_type())),
            );
            self.cooldown
                .set_value((attack.cooldown * 48 / attack.max_cooldown) as usize, obj);
            self.cooldown.show();

            self.number.set_value(attack.value_to_show(), obj);
        } else {
            self.face.hide();
            self.cooldown.hide();
            self.number.set_value(None, obj);
        }
    }
}
