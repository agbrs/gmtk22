use agb::{
    display::{
        object::{Object, ObjectController},
        HEIGHT,
    },
    input::{Button, Tri},
};
use alloc::vec::Vec;

use crate::{
    graphics::{FACE_SPRITES, SELECT_BOX},
    Agb, Die, PlayerDice,
};

enum CustomiseState {
    DiceSelect {
        dice: usize,
    },
    FaceSelect {
        dice: usize,
        face: usize,
    },
    UpgradeSelect {
        dice: usize,
        face: usize,
        upgrade: usize,
    },
}

fn net_position_for_index(idx: usize) -> (u32, u32) {
    if idx == 4 {
        (1, 0)
    } else if idx == 5 {
        (1, 2)
    } else {
        (idx as u32, 1)
    }
}

fn screen_position_for_index(idx: usize) -> (u32, u32) {
    let (x, y) = net_position_for_index(idx);
    (x * 32 + 20, y * 32 + HEIGHT as u32 - 3 * 32)
}

fn move_net_position_lr(idx: usize, direction: Tri) -> usize {
    match direction {
        Tri::Zero => idx,
        Tri::Positive => {
            if idx >= 4 {
                2
            } else {
                (idx + 1) % 4
            }
        }
        Tri::Negative => {
            if idx >= 4 {
                0
            } else {
                idx.checked_sub(1).unwrap_or(3)
            }
        }
    }
}

fn move_net_position_ud(idx: usize, direction: Tri) -> usize {
    match direction {
        Tri::Zero => idx,
        Tri::Negative => {
            if idx < 4 {
                4
            } else if idx == 4 {
                5
            } else if idx == 5 {
                1
            } else {
                unreachable!()
            }
        }
        Tri::Positive => {
            if idx < 4 {
                5
            } else if idx == 4 {
                1
            } else if idx == 5 {
                4
            } else {
                unreachable!()
            }
        }
    }
}

fn create_dice_display<'a>(gfx: &'a ObjectController, dice: &'_ PlayerDice) -> Vec<Object<'a>> {
    let mut objects = Vec::new();
    for (idx, dice) in dice.dice.iter().enumerate() {
        let mut obj = gfx.object(gfx.sprite(FACE_SPRITES.sprite_for_face(dice.faces[1])));
        obj.set_x((idx * 32 - 24 / 2 + 20) as u16);
        obj.set_y(16 - 24 / 2);

        obj.show();

        objects.push(obj);
    }
    objects
}

fn create_net<'a>(gfx: &'a ObjectController, die: &'_ Die) -> Vec<Object<'a>> {
    let mut objects = Vec::new();
    for (idx, &face) in die.faces.iter().enumerate() {
        let mut obj = gfx.object(gfx.sprite(FACE_SPRITES.sprite_for_face(face)));
        let (x, y) = screen_position_for_index(idx);
        obj.set_x((x - 24 / 2) as u16);
        obj.set_y((y - 24 / 2) as u16);

        obj.show();

        objects.push(obj);
    }

    objects
}

pub(crate) fn customise_screen(agb: &mut Agb, mut player_dice: PlayerDice) -> PlayerDice {
    // create the dice

    let mut _net = create_net(&agb.obj, &player_dice.dice[0]);
    let _dice = create_dice_display(&agb.obj, &player_dice);

    let mut input = agb::input::ButtonController::new();

    let mut select_box = agb.obj.object(agb.obj.sprite(SELECT_BOX.sprite(0)));

    select_box.show();

    let mut counter = 0usize;

    let mut state = CustomiseState::DiceSelect { dice: 0 };

    loop {
        counter = counter.wrapping_add(1);
        input.update();
        let ud = (
            input.is_just_pressed(Button::UP),
            input.is_just_pressed(Button::DOWN),
        )
            .into();
        let lr = (
            input.is_just_pressed(Button::LEFT),
            input.is_just_pressed(Button::RIGHT),
        )
            .into();

        match &mut state {
            CustomiseState::DiceSelect { dice } => {
                let new_dice = (*dice as isize + lr as isize)
                    .rem_euclid(player_dice.dice.len() as isize)
                    as usize;
                if new_dice != *dice {
                    *dice = new_dice;
                    _net = create_net(&agb.obj, &player_dice.dice[*dice]);
                }

                select_box.set_x((*dice * 32 - 32 / 2 + 20) as u16);
                select_box.set_y(0);

                if input.is_just_pressed(Button::A) {
                    state = CustomiseState::FaceSelect {
                        dice: *dice,
                        face: 1,
                    }
                }
            }
            CustomiseState::FaceSelect { dice, face } => {
                *face = move_net_position_lr(*face, lr);
                *face = move_net_position_ud(*face, ud);

                let (x, y) = screen_position_for_index(*face);
                select_box.set_x((x - 32 / 2) as u16);
                select_box.set_y((y - 32 / 2) as u16);

                if input.is_just_pressed(Button::B) {
                    state = CustomiseState::DiceSelect { dice: *dice };
                }
            }
            CustomiseState::UpgradeSelect {
                dice,
                face,
                upgrade,
            } => {}
        }

        if input.is_just_pressed(Button::START) {
            return player_dice;
        }

        select_box.set_sprite(agb.obj.sprite(SELECT_BOX.animation_sprite(counter / 10)));

        agb.star_background.update();
        let _ = agb::rng::gen();
        agb.vblank.wait_for_vblank();
        agb.obj.commit();
        agb.star_background.commit(&mut agb.vram);
    }
}
