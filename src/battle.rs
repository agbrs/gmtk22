use crate::{Agb, Face, PlayerDice, FACE_SPRITES, SELECT_BOX, SHIP_SPRITES};
use agb::{hash_map::HashMap, input::Button};
use alloc::vec::Vec;

const MALFUNCTION_COOLDOWN_FRAMES: u32 = 3 * 60;
const ROLL_TIME_FRAMES: u32 = 60;

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

    fn faces_to_render(&self) -> impl Iterator<Item = Face> + '_ {
        self.rolls.iter().map(|rolled_die| match rolled_die {
            DieState::Rolling(_, face) => *face,
            DieState::Rolled(RolledDie { face, .. }) => *face,
        })
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
        if self.player.shield_count <= *shield {
            self.player.shield_count += 1;
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

    let num_dice = player_dice.dice.len();

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
                .map(|die| DieState::Rolling(ROLL_TIME_FRAMES, die.roll()))
                .collect(),
        },
        player_dice: player_dice.clone(),
    };

    let mut dice_display: Vec<_> = current_battle_state
        .rolled_dice
        .faces_to_render()
        .enumerate()
        .map(|(i, face)| {
            let mut die_obj = obj.object(obj.sprite(FACE_SPRITES.sprite_for_face(face)));

            die_obj.set_y(120).set_x(i as u16 * 40 + 28).show();

            die_obj
        })
        .collect();

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
        for (die_obj, current_roll) in dice_display
            .iter_mut()
            .zip(current_battle_state.rolled_dice.faces_to_render())
        {
            die_obj.set_sprite(obj.sprite(FACE_SPRITES.sprite_for_face(current_roll)));
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
