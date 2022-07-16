use crate::{Agb, Face, PlayerDice, ShipSprites, FACE_SPRITES, SELECT_BOX, SHIP_SPRITES};
use agb::{hash_map::HashMap, input::Button};
use alloc::vec::Vec;

const MALFUNCTION_COOLDOWN_FRAMES: u32 = 3 * 60;

/// A face of the rolled die and it's cooldown (should it be a malfunction)
#[derive(Debug)]

struct RolledDie {
    face: Face,
    cooldown: u32,
}

impl RolledDie {
    fn update(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
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

    let obj = &agb.obj;

    let mut player_obj = obj.object(obj.sprite(player_sprite));
    let mut enemy_obj = obj.object(obj.sprite(enemy_sprite));

    player_obj.set_x(27).set_y(16).set_z(1).show();
    enemy_obj.set_x(167).set_y(16).set_z(1).show();

    let mut select_box_obj = agb.obj.object(agb.obj.sprite(SELECT_BOX.sprite(0)));
    select_box_obj.show();

    let mut current_battle_state = CurrentBattleState {
        player: PlayerState {
            shield_count: 0,
            health: 100,
        },
        enemy: EnemyState {},
        rolled_dice: RolledDice {
            rolls: player_dice
                .dice
                .iter()
                .map(|die| RolledDie {
                    face: die.roll(),
                    cooldown: 0,
                })
                .collect(),
        },
    };

    let mut dice_display: Vec<_> = current_battle_state
        .rolled_dice
        .rolls
        .iter()
        .enumerate()
        .map(|(i, die)| {
            let mut die_obj = obj.object(obj.sprite(FACE_SPRITES.sprite_for_face(die.face)));

            die_obj.set_y(120).set_x(i as u16 * 40 + 28).show();

            die_obj
        })
        .collect();

    let mut selected_die = 0usize;
    let mut input = agb::input::ButtonController::new();
    let mut counter = 0usize;

    loop {
        counter = counter.wrapping_add(1);
        current_battle_state.rolled_dice.update();

        input.update();

        if input.is_just_pressed(Button::LEFT) {
            if selected_die == 0 {
                selected_die = player_dice.dice.len() - 1;
            } else {
                selected_die -= 1;
            }
        }

        if input.is_just_pressed(Button::RIGHT) {
            if selected_die == player_dice.dice.len() - 1 {
                selected_die = 0;
            } else {
                selected_die += 1;
            }
        }

        if input.is_just_pressed(Button::A) {
            let selected_rolled_die = &mut current_battle_state.rolled_dice.rolls[selected_die];
            if selected_rolled_die.can_reroll() {
                selected_rolled_die.face = player_dice.dice[selected_die].roll();

                if selected_rolled_die.face == Face::Malfunction {
                    selected_rolled_die.cooldown = MALFUNCTION_COOLDOWN_FRAMES;
                }
            }
        }

        if input.is_just_pressed(Button::START) {
            current_battle_state.accept_rolls();
        }

        // update the dice display to display the current values
        for (die_obj, current_roll) in dice_display
            .iter_mut()
            .zip(current_battle_state.rolled_dice.rolls.iter())
        {
            die_obj.set_sprite(obj.sprite(FACE_SPRITES.sprite_for_face(current_roll.face)));
        }

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
