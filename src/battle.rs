use crate::{
    graphics::SELECT_BOX, level_generation::generate_attack, Agb, EnemyAttackType, Face, PlayerDice,
};
use agb::{hash_map::HashMap, input::Button};
use alloc::vec;
use alloc::vec::Vec;

use self::display::{BattleScreenDisplay, DisplayAnimation};

mod display;

pub(super) const MALFUNCTION_COOLDOWN_FRAMES: u32 = 5 * 60;
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

    fn can_reroll_after_accept(&self) -> bool {
        self.face != Face::Malfunction
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
    Shield(u32),
    Heal(u32),
}

impl EnemyAttack {
    fn apply_effect(
        &self,
        player_state: &mut PlayerState,
        enemy_state: &mut EnemyState,
    ) -> Option<DisplayAnimation> {
        match self {
            EnemyAttack::Shoot(damage) => {
                if *damage > player_state.shield_count {
                    if player_state.shield_count > 0 {
                        player_state.shield_count = 0;
                        Some(DisplayAnimation::EnemyBreakShield)
                    } else {
                        player_state.health = player_state.health.saturating_sub(*damage);
                        Some(DisplayAnimation::EnemyShootPlayer)
                    }
                } else {
                    None
                }
            }
            EnemyAttack::Shield(shield) => {
                let should_animate = enemy_state.shield_count < *shield;
                enemy_state.shield_count = enemy_state.shield_count.max(*shield);

                if should_animate {
                    Some(DisplayAnimation::EnemyNewShield)
                } else {
                    None
                }
            }
            EnemyAttack::Heal(amount) => {
                enemy_state.health = enemy_state.max_health.min(enemy_state.health + amount);
                Some(DisplayAnimation::EnemyHeal)
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
            EnemyAttack::Shield(_) => EnemyAttackType::Shield,
            EnemyAttack::Heal(_) => EnemyAttackType::Heal,
        }
    }

    fn value_to_show(&self) -> Option<u32> {
        match self.attack {
            EnemyAttack::Shoot(i) => Some(i),
            EnemyAttack::Heal(i) => Some(i),
            EnemyAttack::Shield(i) => Some(i),
        }
    }

    #[must_use]
    fn update(
        &mut self,
        player_state: &mut PlayerState,
        enemy_state: &mut EnemyState,
    ) -> (Option<DisplayAnimation>, bool) {
        if self.cooldown == 0 {
            return (self.attack.apply_effect(player_state, enemy_state), true);
        }

        self.cooldown -= 1;

        (None, false)
    }
}

#[derive(Debug)]
struct EnemyState {
    shield_count: u32,
    health: u32,
    max_health: u32,
}

#[derive(Debug)]
pub struct CurrentBattleState {
    player: PlayerState,
    enemy: EnemyState,
    rolled_dice: RolledDice,
    player_dice: PlayerDice,
    attacks: [Option<EnemyAttackState>; 2],
    current_level: u32,
}

impl CurrentBattleState {
    fn accept_rolls(&mut self) -> Vec<DisplayAnimation> {
        let mut animations = vec![];

        let mut face_counts: HashMap<Face, u32> = HashMap::new();
        for face in self.rolled_dice.faces_for_accepting() {
            match face {
                Face::DoubleShot => *face_counts.entry(Face::Shoot).or_default() += 2,
                Face::TripleShot => *face_counts.entry(Face::Shoot).or_default() += 3,
                other => *face_counts.entry(other).or_default() += 1,
            }
        }

        // shield
        let shield = face_counts.entry(Face::Shield).or_default();

        if *shield > self.player.shield_count {
            self.player.shield_count = *shield;
            animations.push(DisplayAnimation::PlayerNewShield);
        }

        // shooting
        let shoot = *face_counts.entry(Face::Shoot).or_default();
        let shoot_power = (shoot * (shoot + 1)) / 2;
        let enemy_shield_equiv = self
            .enemy
            .shield_count
            .saturating_sub(*face_counts.entry(Face::Bypass).or_default());

        if shoot_power >= enemy_shield_equiv {
            if enemy_shield_equiv > 0 {
                self.enemy.shield_count = 0;
            } else {
                self.enemy.shield_count = 0;
                self.enemy.health = self.enemy.health.saturating_sub(shoot_power);
            }

            if self.enemy.shield_count > 0 {
                animations.push(DisplayAnimation::PlayerBreakShield);
            } else {
                animations.push(DisplayAnimation::PlayerShootEnemy);
            }
        }

        // disrupt

        let disrupt = *face_counts.entry(Face::Disrupt).or_default();
        let disrupt_power = (disrupt * (disrupt + 1)) / 2;
        for a in self.attacks.iter_mut().flatten() {
            a.cooldown += disrupt_power * 60;
            // TODO: disrupt animation
        }

        let mut malfunction_all = false;

        for roll in self
            .rolled_dice
            .rolls
            .iter_mut()
            .filter_map(|face| match face {
                DieState::Rolled(rolled_die) => Some(rolled_die),
                _ => None,
            })
        {
            if roll.face == Face::DoubleShot {
                roll.cooldown = MALFUNCTION_COOLDOWN_FRAMES;
                roll.face = Face::Malfunction;
            }
            if roll.face == Face::TripleShot {
                malfunction_all = true;
            }
        }

        if malfunction_all {
            for roll in self
                .rolled_dice
                .rolls
                .iter_mut()
                .filter_map(|face| match face {
                    DieState::Rolled(rolled_die) => Some(rolled_die),
                    _ => None,
                })
            {
                roll.cooldown = MALFUNCTION_COOLDOWN_FRAMES;
                roll.face = Face::Malfunction;
            }
        }

        // reroll non-malfunctions after accepting
        for i in 0..self.player_dice.dice.len() {
            self.roll_die(i, ROLL_TIME_FRAMES_ALL, true);
        }

        animations
    }

    fn roll_die(&mut self, die_index: usize, time: u32, is_after_accept: bool) {
        if let DieState::Rolled(ref selected_rolled_die) = self.rolled_dice.rolls[die_index] {
            let can_reroll = if is_after_accept {
                selected_rolled_die.can_reroll_after_accept()
            } else {
                selected_rolled_die.can_reroll()
            };

            if can_reroll {
                self.rolled_dice.rolls[die_index] =
                    DieState::Rolling(time, self.player_dice.dice[die_index].roll());
            }
        }
    }

    fn update(&mut self) -> Vec<DisplayAnimation> {
        self.rolled_dice.update(&self.player_dice);

        let mut animations = vec![];

        for attack in self.attacks.iter_mut() {
            if let Some(attack_state) = attack {
                if let (animation, true) = attack_state.update(&mut self.player, &mut self.enemy) {
                    attack.take();
                    if let Some(animation) = animation {
                        animations.push(animation);
                    }
                }
            } else if let Some(generated_attack) = generate_attack(self.current_level) {
                attack.replace(EnemyAttackState {
                    attack: generated_attack.attack,
                    cooldown: generated_attack.cooldown,
                    max_cooldown: generated_attack.cooldown,
                });
            }
        }

        animations
    }
}

pub(crate) fn battle_screen(agb: &mut Agb, player_dice: PlayerDice, current_level: u32) {
    let obj = &agb.obj;

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

    let mut battle_screen_display = BattleScreenDisplay::new(obj, &current_battle_state);

    let mut selected_die = 0usize;
    let mut input = agb::input::ButtonController::new();
    let mut counter = 0usize;

    loop {
        counter = counter.wrapping_add(1);

        if battle_screen_display.update(obj, &current_battle_state) {
            for animation in current_battle_state.update() {
                battle_screen_display.add_animation(animation);
            }
        }

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
            current_battle_state.roll_die(selected_die, ROLL_TIME_FRAMES_ONE, false);
            agb.sfx.roll();
        }

        if input.is_just_pressed(Button::START) {
            for animation in current_battle_state.accept_rolls() {
                battle_screen_display.add_animation(animation);
            }
            agb.sfx.roll_multi();
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
