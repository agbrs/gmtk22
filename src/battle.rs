use crate::{
    graphics::{
        FractionDisplay, HealthBar, NumberDisplay, ENEMY_ATTACK_SPRITES, FACE_SPRITES, SELECT_BOX,
        SHIP_SPRITES,
    },
    level_generation::generate_attack,
    Agb, EnemyAttackType, Face, PlayerDice, Ship,
};
use agb::{hash_map::HashMap, input::Button};
use alloc::vec::Vec;

const MALFUNCTION_COOLDOWN_FRAMES: u32 = 5 * 60;
const ROLL_TIME_FRAMES_ALL: u32 = 2 * 60;
const ROLL_TIME_FRAMES_ONE: u32 = 60 / 8;

/// A face of the rolled die and it's cooldown (should it be a malfunction)
#[derive(Debug)]

struct RolledDie {
    face: Face,
    cooldown: u32,
}

impl RolledDie {
    fn new(face: Face) -> Self {
        let cooldown = if face == Face::Malfunction {
            MALFUNCTION_COOLDOWN_FRAMES
        } else {
            0
        };

        Self { face, cooldown }
    }

    fn update(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    fn can_reroll(&self) -> bool {
        self.face != Face::Malfunction || self.cooldown == 0
    }

    fn cooldown(&self) -> Option<u32> {
        if self.face == Face::Malfunction && self.cooldown > 0 {
            Some(self.cooldown)
        } else {
            None
        }
    }
}

#[derive(Debug)]
enum DieState {
    Rolling(u32, Face),
    Rolled(RolledDie),
}

#[derive(Debug)]
struct RolledDice {
    rolls: Vec<DieState>,
}

impl RolledDice {
    fn update(&mut self, player_dice: &PlayerDice) {
        self.rolls
            .iter_mut()
            .zip(player_dice.dice.iter())
            .for_each(|(die_state, player_die)| match die_state {
                DieState::Rolling(ref mut timeout, ref mut face) => {
                    if *timeout == 0 {
                        *die_state = DieState::Rolled(RolledDie::new(player_die.roll()));
                    } else {
                        if *timeout % 2 == 0 {
                            *face = player_die.roll();
                        }
                        *timeout -= 1;
                    }
                }
                DieState::Rolled(ref mut rolled_die) => rolled_die.update(),
            });
    }

    fn faces_for_accepting(&self) -> impl Iterator<Item = Face> + '_ {
        self.rolls.iter().filter_map(|state| match state {
            DieState::Rolled(rolled_die) if rolled_die.face != Face::Malfunction => {
                Some(rolled_die.face)
            }
            _ => None,
        })
    }

    fn faces_to_render(&self) -> impl Iterator<Item = (Face, Option<u32>)> + '_ {
        self.rolls.iter().map(|rolled_die| match rolled_die {
            DieState::Rolling(_, face) => (*face, None),
            DieState::Rolled(rolled_die) => (rolled_die.face, rolled_die.cooldown()),
        })
    }
}

#[derive(Debug)]
struct PlayerState {
    shield_count: u32,
    health: u32,
    max_health: u32,
}

#[derive(Debug)]
pub enum EnemyAttack {
    Shoot(u32),
    Shield,
    Heal(u32),
}

impl EnemyAttack {
    fn apply_effect(&self, player_state: &mut PlayerState, enemy_state: &mut EnemyState) {
        match self {
            EnemyAttack::Shoot(damage) => {
                if *damage > player_state.shield_count {
                    if player_state.shield_count > 0 {
                        player_state.shield_count -= 1;
                    } else {
                        player_state.health = player_state.health.saturating_sub(*damage);
                    }
                }
            }
            EnemyAttack::Shield => {
                if enemy_state.shield_count < 5 {
                    enemy_state.shield_count += 1;
                }
            }
            EnemyAttack::Heal(amount) => {
                enemy_state.health = enemy_state.max_health.min(enemy_state.health + amount);
            }
        }
    }
}

#[derive(Debug)]
struct EnemyAttackState {
    attack: EnemyAttack,
    cooldown: u32,
    max_cooldown: u32,
}

impl EnemyAttackState {
    fn attack_type(&self) -> EnemyAttackType {
        match self.attack {
            EnemyAttack::Shoot(_) => EnemyAttackType::Attack,
            EnemyAttack::Shield => EnemyAttackType::Shield,
            EnemyAttack::Heal(_) => EnemyAttackType::Heal,
        }
    }

    fn value_to_show(&self) -> Option<u32> {
        match self.attack {
            EnemyAttack::Shoot(i) => Some(i),
            EnemyAttack::Heal(i) => Some(i),
            EnemyAttack::Shield => None,
        }
    }

    #[must_use]
    fn update(&mut self, player_state: &mut PlayerState, enemy_state: &mut EnemyState) -> bool {
        if self.cooldown == 0 {
            self.attack.apply_effect(player_state, enemy_state);
            return true;
        }

        self.cooldown -= 1;

        false
    }
}

#[derive(Debug)]
struct EnemyState {
    shield_count: u32,
    health: u32,
    max_health: u32,
}

#[derive(Debug)]
struct CurrentBattleState {
    player: PlayerState,
    enemy: EnemyState,
    rolled_dice: RolledDice,
    player_dice: PlayerDice,
    attacks: [Option<EnemyAttackState>; 2],
    current_level: u32,
}

impl CurrentBattleState {
    fn accept_rolls(&mut self) {
        let mut face_counts: HashMap<Face, u32> = HashMap::new();
        for face in self.rolled_dice.faces_for_accepting() {
            *face_counts.entry(face).or_default() += 1;
        }

        // shield
        let shield = face_counts.entry(Face::Shield).or_default();
        if self.player.shield_count < *shield {
            self.player.shield_count += 1;
        }

        // shooting
        let shoot = *face_counts.entry(Face::Attack).or_default();
        let shoot_power = shoot * shoot;

        if shoot_power > self.enemy.shield_count {
            if self.enemy.shield_count > 0 {
                self.enemy.shield_count -= 1;
            } else {
                self.enemy.health = self.enemy.health.saturating_sub(shoot_power);
            }
        }

        // reroll everything after accepting
        for i in 0..self.player_dice.dice.len() {
            self.roll_die(i, ROLL_TIME_FRAMES_ALL);
        }
    }

    fn roll_die(&mut self, die_index: usize, time: u32) {
        if let DieState::Rolled(ref selected_rolled_die) = self.rolled_dice.rolls[die_index] {
            if selected_rolled_die.can_reroll() {
                self.rolled_dice.rolls[die_index] =
                    DieState::Rolling(time, self.player_dice.dice[die_index].roll());
            }
        }
    }

    fn update(&mut self) {
        self.rolled_dice.update(&self.player_dice);

        for attack in self.attacks.iter_mut() {
            if let Some(attack_state) = attack {
                if attack_state.update(&mut self.player, &mut self.enemy) {
                    attack.take();
                }
            } else if let Some(generated_attack) = generate_attack(self.current_level) {
                attack.replace(EnemyAttackState {
                    attack: generated_attack.attack,
                    cooldown: generated_attack.cooldown,
                    max_cooldown: generated_attack.cooldown,
                });
            };
        }
    }
}

const HEALTH_BAR_WIDTH: usize = 48;

pub(crate) fn battle_screen(agb: &mut Agb, player_dice: PlayerDice, current_level: u32) {
    let obj = &agb.obj;

    let player_sprite = SHIP_SPRITES.sprite_for_ship(Ship::Player);
    let enemy_sprite = SHIP_SPRITES.sprite_for_ship(Ship::Drone);

    let shield_sprite = SHIP_SPRITES.sprite_for_ship(Ship::Shield);

    let mut player_obj = obj.object(obj.sprite(player_sprite));
    let mut enemy_obj = obj.object(obj.sprite(enemy_sprite));

    let player_x = 12;
    let player_y = 8;
    let enemy_x = 167;

    player_obj.set_x(player_x).set_y(player_y).set_z(1).show();
    enemy_obj.set_x(enemy_x).set_y(player_y).set_z(1).show();

    let mut select_box_obj = agb.obj.object(agb.obj.sprite(SELECT_BOX.sprite(0)));
    select_box_obj.show();

    let num_dice = player_dice.dice.len();

    let mut current_battle_state = CurrentBattleState {
        player: PlayerState {
            shield_count: 0,
            health: 120,
            max_health: 120,
        },
        enemy: EnemyState {
            shield_count: 0,
            health: 50,
            max_health: 50,
        },
        rolled_dice: RolledDice {
            rolls: player_dice
                .dice
                .iter()
                .map(|die| DieState::Rolling(ROLL_TIME_FRAMES_ALL, die.roll()))
                .collect(),
        },
        player_dice: player_dice.clone(),
        attacks: [None, None],
        current_level,
    };

    let mut dice_display: Vec<_> = current_battle_state
        .rolled_dice
        .faces_to_render()
        .enumerate()
        .map(|(i, (face, _))| {
            let mut die_obj = obj.object(obj.sprite(FACE_SPRITES.sprite_for_face(face)));

            die_obj.set_y(120).set_x(i as u16 * 40 + 28).show();

            die_obj
        })
        .collect();

    let mut dice_cooldowns: Vec<_> = dice_display
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let mut cooldown_bar = HealthBar::new((i as i32 * 40 + 28, 120 - 8).into(), 24, obj);
            cooldown_bar.hide();
            cooldown_bar
        })
        .collect();

    let mut player_shield_display: Vec<_> = (0..5)
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

    let mut enemy_shield_display: Vec<_> = (0..5)
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
    let mut player_healthbar = HealthBar::new(
        (player_healthbar_x, player_y - 8).into(),
        HEALTH_BAR_WIDTH,
        obj,
    );
    let mut enemy_healthbar = HealthBar::new(
        (enemy_healthbar_x, player_y - 8).into(),
        HEALTH_BAR_WIDTH,
        obj,
    );

    let mut player_health_display = FractionDisplay::new(
        (
            player_healthbar_x + HEALTH_BAR_WIDTH as u16 / 2 - 16,
            player_y,
        )
            .into(),
        3,
        obj,
    );
    let mut enemy_health_display = FractionDisplay::new(
        (
            enemy_healthbar_x + HEALTH_BAR_WIDTH as u16 / 2 - 16,
            player_y,
        )
            .into(),
        3,
        obj,
    );

    let mut enemy_attack_display: Vec<_> = (0..2)
        .into_iter()
        .map(|i| {
            let mut attack_obj = obj.object(
                obj.sprite(ENEMY_ATTACK_SPRITES.sprite_for_attack(EnemyAttackType::Attack)),
            );

            let attack_obj_position = (120, 56 + 32 * i).into();
            attack_obj.set_position(attack_obj_position).hide();

            let mut attack_cooldown = HealthBar::new(attack_obj_position + (32, 8).into(), 48, obj);
            attack_cooldown.hide();

            let attack_number_display = NumberDisplay::new(attack_obj_position - (8, -10).into());

            (attack_obj, attack_cooldown, attack_number_display)
        })
        .collect();

    let mut selected_die = 0usize;
    let mut input = agb::input::ButtonController::new();
    let mut counter = 0usize;

    loop {
        counter = counter.wrapping_add(1);
        current_battle_state.update();

        input.update();

        if input.is_just_pressed(Button::LEFT) {
            if selected_die == 0 {
                selected_die = num_dice - 1;
            } else {
                selected_die -= 1;
            }
        }

        if input.is_just_pressed(Button::RIGHT) {
            if selected_die == num_dice - 1 {
                selected_die = 0;
            } else {
                selected_die += 1;
            }
        }

        if input.is_just_pressed(Button::A) {
            current_battle_state.roll_die(selected_die, ROLL_TIME_FRAMES_ONE);
        }

        if input.is_just_pressed(Button::START) {
            current_battle_state.accept_rolls();
        }

        // update the dice display to display the current values
        for ((die_obj, (current_face, cooldown)), cooldown_healthbar) in dice_display
            .iter_mut()
            .zip(current_battle_state.rolled_dice.faces_to_render())
            .zip(dice_cooldowns.iter_mut())
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

        for (i, player_shield) in player_shield_display.iter_mut().enumerate() {
            if i < current_battle_state.player.shield_count as usize {
                player_shield.show();
            } else {
                player_shield.hide();
            }
        }

        for (i, player_shield) in enemy_shield_display.iter_mut().enumerate() {
            if i < current_battle_state.enemy.shield_count as usize {
                player_shield.show();
            } else {
                player_shield.hide();
            }
        }

        player_healthbar.set_value(
            ((current_battle_state.player.health * HEALTH_BAR_WIDTH as u32)
                / current_battle_state.player.max_health) as usize,
            obj,
        );

        enemy_healthbar.set_value(
            ((current_battle_state.enemy.health * HEALTH_BAR_WIDTH as u32)
                / current_battle_state.enemy.max_health) as usize,
            obj,
        );

        player_health_display.set_value(
            current_battle_state.player.health as usize,
            current_battle_state.player.max_health as usize,
            obj,
        );
        enemy_health_display.set_value(
            current_battle_state.enemy.health as usize,
            current_battle_state.enemy.max_health as usize,
            obj,
        );

        for (i, attack) in current_battle_state.attacks.iter().enumerate() {
            let attack_display = &mut enemy_attack_display[i];

            if let Some(attack) = attack {
                attack_display.0.show().set_sprite(
                    obj.sprite(ENEMY_ATTACK_SPRITES.sprite_for_attack(attack.attack_type())),
                );
                attack_display
                    .1
                    .set_value((attack.cooldown * 48 / attack.max_cooldown) as usize, obj);
                attack_display.1.show();

                attack_display.2.set_value(attack.value_to_show(), obj);
            } else {
                attack_display.0.hide();
                attack_display.1.hide();
                attack_display.2.set_value(None, obj);
            }
        }

        select_box_obj
            .set_y(120 - 4)
            .set_x(selected_die as u16 * 40 + 28 - 4)
            .set_sprite(agb.obj.sprite(SELECT_BOX.animation_sprite(counter / 10)));

        agb.star_background.update();
        agb.sfx.frame();
        agb.vblank.wait_for_vblank();
        agb.obj.commit();
        agb.star_background.commit(&mut agb.vram);
    }
}
