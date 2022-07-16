use agb::display::object::{Object, ObjectController};
use alloc::vec;
use alloc::vec::Vec;

use crate::graphics::SHIELD;
use crate::{
    graphics::{
        FractionDisplay, HealthBar, NumberDisplay, BULLET_SPRITE, ENEMY_ATTACK_SPRITES,
        FACE_SPRITES, SHIP_SPRITES,
    },
    EnemyAttackType, Ship,
};

use super::{CurrentBattleState, EnemyAttackState, MALFUNCTION_COOLDOWN_FRAMES};

#[derive(Clone, Copy)]
pub enum DisplayAnimation {
    PlayerShootEnemy,
    EnemyShootPlayer,
    PlayerBreakShield,
    EnemyBreakShield,
    PlayerNewShield,
    EnemyNewShield,
    EnemyHeal,
}

struct BattleScreenDisplayObjects<'a> {
    dice: Vec<Object<'a>>,
    dice_cooldowns: Vec<HealthBar<'a>>,
    player_shield: Vec<Object<'a>>,
    enemy_shield: Vec<Object<'a>>,

    player_healthbar: HealthBar<'a>,
    enemy_healthbar: HealthBar<'a>,
    player_health: FractionDisplay<'a>,
    enemy_health: FractionDisplay<'a>,

    enemy_attack_display: Vec<EnemyAttackDisplay<'a>>,
}

pub struct BattleScreenDisplay<'a> {
    objs: BattleScreenDisplayObjects<'a>,
    animations: Vec<AnimationState<'a>>,

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

        let objs = BattleScreenDisplayObjects {
            dice,
            dice_cooldowns,
            player_shield,
            enemy_shield,

            player_healthbar,
            enemy_healthbar,
            player_health: player_health_display,
            enemy_health: enemy_health_display,

            enemy_attack_display,
        };

        Self {
            objs,

            animations: vec![],

            _misc_sprites: misc_sprites,
        }
    }

    pub fn update(
        &mut self,
        obj: &'a ObjectController,
        current_battle_state: &CurrentBattleState,
    ) -> bool {
        // update the dice display to display the current values
        for ((die_obj, (current_face, cooldown)), cooldown_healthbar) in self
            .objs
            .dice
            .iter_mut()
            .zip(current_battle_state.rolled_dice.faces_to_render())
            .zip(self.objs.dice_cooldowns.iter_mut())
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

        let mut animations_to_remove = vec![];
        for (i, animation) in self.animations.iter_mut().enumerate() {
            if animation.update(&mut self.objs, obj, current_battle_state) {
                animations_to_remove.push(i);
            }
        }

        for &animation_to_remove in animations_to_remove.iter().rev() {
            self.animations.swap_remove(animation_to_remove);
        }

        for (i, player_shield) in self.objs.player_shield.iter_mut().enumerate() {
            if i < current_battle_state.player.shield_count as usize {
                player_shield.show();
            } else {
                player_shield.hide();
            }
        }

        for (i, player_shield) in self.objs.enemy_shield.iter_mut().enumerate() {
            if i < current_battle_state.enemy.shield_count as usize {
                player_shield.show();
            } else {
                player_shield.hide();
            }
        }

        self.objs.player_healthbar.set_value(
            ((current_battle_state.player.health * HEALTH_BAR_WIDTH as u32)
                / current_battle_state.player.max_health) as usize,
            obj,
        );

        self.objs.enemy_healthbar.set_value(
            ((current_battle_state.enemy.health * HEALTH_BAR_WIDTH as u32)
                / current_battle_state.enemy.max_health) as usize,
            obj,
        );

        self.objs.player_health.set_value(
            current_battle_state.player.health as usize,
            current_battle_state.player.max_health as usize,
            obj,
        );

        self.objs.enemy_health.set_value(
            current_battle_state.enemy.health as usize,
            current_battle_state.enemy.max_health as usize,
            obj,
        );

        for (i, attack) in current_battle_state.attacks.iter().enumerate() {
            self.objs.enemy_attack_display[i].update(attack, obj);
        }

        true
    }

    pub fn add_animation(&mut self, anim: DisplayAnimation, obj: &'a ObjectController) {
        self.animations
            .push(AnimationState::for_animation(anim, obj))
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

enum AnimationState<'a> {
    PlayerShootEnemy {
        bullet: Object<'a>,
        x_position: i32,
    },
    EnemyShootPlayer {
        bullet: Object<'a>,
        x_position: i32,
    },
    PlayerBreakShield {
        bullet: Object<'a>,
        x_position: i32,
        shield_break_frame: i32,
    },
    EnemyBreakShield {
        bullet: Object<'a>,
        x_position: i32,
        shield_break_frame: i32,
    },
    PlayerNewShield {
        shield_frame: i32,
    },
    EnemyNewShield {
        shield_frame: i32,
    },
    EnemyHeal {
        heal_frame: i32,
    },
}

impl<'a> AnimationState<'a> {
    fn for_animation(a: DisplayAnimation, obj: &'a ObjectController) -> Self {
        match a {
            DisplayAnimation::PlayerShootEnemy => Self::PlayerShootEnemy {
                x_position: 64,
                bullet: obj.object(obj.sprite(BULLET_SPRITE)),
            },
            DisplayAnimation::PlayerBreakShield => Self::PlayerBreakShield {
                bullet: obj.object(obj.sprite(BULLET_SPRITE)),
                x_position: 64,
                shield_break_frame: 0,
            },
            DisplayAnimation::PlayerNewShield => Self::PlayerNewShield { shield_frame: 6 },
            DisplayAnimation::EnemyShootPlayer => Self::EnemyShootPlayer {
                bullet: obj.object(obj.sprite(BULLET_SPRITE)),
                x_position: 176,
            },
            DisplayAnimation::EnemyBreakShield => Self::EnemyBreakShield {
                bullet: obj.object(obj.sprite(BULLET_SPRITE)),
                x_position: 176,
                shield_break_frame: 0,
            },
            DisplayAnimation::EnemyNewShield => Self::EnemyNewShield { shield_frame: 6 },
            DisplayAnimation::EnemyHeal => AnimationState::EnemyHeal { heal_frame: 0 },
        }
    }

    fn update(
        &mut self,
        objs: &mut BattleScreenDisplayObjects<'a>,
        obj: &'a ObjectController,
        current_battle_state: &CurrentBattleState,
    ) -> bool {
        match self {
            Self::PlayerShootEnemy { bullet, x_position } => {
                bullet.set_x(*x_position as u16).set_y(36).show();
                *x_position += 2;

                *x_position > 190
            }
            Self::PlayerBreakShield {
                bullet,
                x_position,
                shield_break_frame,
            } => {
                if *x_position > 190 {
                    if *shield_break_frame >= 12 {
                        for shield_obj in objs.enemy_shield.iter_mut() {
                            shield_obj.set_sprite(obj.sprite(SHIELD.sprite(0)));
                        }
                        true
                    } else {
                        for shield_obj in objs.enemy_shield.iter_mut() {
                            shield_obj.set_sprite(
                                obj.sprite(SHIELD.sprite((*shield_break_frame / 2) as usize)),
                            );
                        }
                        false
                    }
                } else {
                    bullet.set_x(*x_position as u16).set_y(36).show();
                    *x_position += 2;

                    false
                }
            }
            Self::PlayerNewShield { shield_frame } => {
                objs.player_shield[(current_battle_state.player.shield_count - 1) as usize]
                    .show()
                    .set_sprite(obj.sprite(SHIELD.sprite(*shield_frame as usize / 2)));

                *shield_frame -= 1;
                *shield_frame == 0
            }
            Self::EnemyShootPlayer { bullet, x_position } => {
                bullet
                    .set_x(*x_position as u16)
                    .set_y(36)
                    .show()
                    .set_hflip(true);
                *x_position -= 2;

                *x_position < 48
            }
            Self::EnemyBreakShield {
                bullet,
                x_position,
                shield_break_frame,
            } => {
                if *x_position < 48 {
                    if *shield_break_frame >= 12 {
                        for shield_obj in objs.player_shield.iter_mut() {
                            shield_obj.set_sprite(obj.sprite(SHIELD.sprite(0)));
                        }
                        true
                    } else {
                        for shield_obj in objs.player_shield.iter_mut() {
                            shield_obj.set_sprite(
                                obj.sprite(SHIELD.sprite((*shield_break_frame / 2) as usize)),
                            );
                        }
                        *shield_break_frame += 1;
                        false
                    }
                } else {
                    bullet
                        .set_x(*x_position as u16)
                        .set_y(36)
                        .show()
                        .set_hflip(true);
                    *x_position -= 2;

                    false
                }
            }
            Self::EnemyNewShield { shield_frame } => {
                objs.enemy_shield[(current_battle_state.enemy.shield_count - 1) as usize]
                    .show()
                    .set_sprite(obj.sprite(SHIELD.sprite(*shield_frame as usize / 2)));

                *shield_frame -= 1;
                *shield_frame == 0
            }
            _ => true,
        }
    }
}
