use crate::{
    graphics::{HealthBar, NumberDisplay, FACE_SPRITES, SELECT_BOX, SHIP_SPRITES},
    Agb, Face, PlayerDice, Ship,
};
use agb::{hash_map::HashMap, input::Button};
use alloc::vec::Vec;

const MALFUNCTION_COOLDOWN_FRAMES: u32 = 5 * 60;
const ROLL_TIME_FRAMES: u32 = 2 * 60;

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
                        if *timeout % 4 == 0 {
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
            self.roll_die(i);
        }
    }

    fn roll_die(&mut self, die_index: usize) {
        if let DieState::Rolled(ref selected_rolled_die) = self.rolled_dice.rolls[die_index] {
            if selected_rolled_die.can_reroll() {
                self.rolled_dice.rolls[die_index] =
                    DieState::Rolling(ROLL_TIME_FRAMES, self.player_dice.dice[die_index].roll());
            }
        }
    }
}

const HEALTH_BAR_WIDTH: usize = 48;

pub(crate) fn battle_screen(agb: &mut Agb, player_dice: PlayerDice) {
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
            health: 58,
            max_health: 120,
        },
        enemy: EnemyState {
            shield_count: 5,
            health: 38,
            max_health: 50,
        },
        rolled_dice: RolledDice {
            rolls: player_dice
                .dice
                .iter()
                .map(|die| DieState::Rolling(ROLL_TIME_FRAMES, die.roll()))
                .collect(),
        },
        player_dice: player_dice.clone(),
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

    let mut player_health_display = NumberDisplay::new(
        (
            player_healthbar_x + HEALTH_BAR_WIDTH as u16 / 2 - 16,
            player_y,
        )
            .into(),
        3,
        obj,
    );
    let mut enemy_health_display = NumberDisplay::new(
        (
            enemy_healthbar_x + HEALTH_BAR_WIDTH as u16 / 2 - 16,
            player_y,
        )
            .into(),
        3,
        obj,
    );

    let mut selected_die = 0usize;
    let mut input = agb::input::ButtonController::new();
    let mut counter = 0usize;

    loop {
        counter = counter.wrapping_add(1);
        current_battle_state.rolled_dice.update(&player_dice);

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
            current_battle_state.roll_die(selected_die);
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

        select_box_obj
            .set_y(120 - 4)
            .set_x(selected_die as u16 * 40 + 28 - 4);
        select_box_obj.set_sprite(agb.obj.sprite(SELECT_BOX.animation_sprite(counter / 10)));

        agb.star_background.update();
        agb.vblank.wait_for_vblank();
        agb.obj.commit();
        agb.star_background.commit(&mut agb.vram);
    }
}
