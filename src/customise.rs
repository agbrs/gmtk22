use agb::{
    display::{
        object::{Object, ObjectController},
        palette16::Palette16,
        tiled::{RegularMap, TileSet, TileSetting},
        HEIGHT, WIDTH,
    },
    include_gfx,
    input::{Button, Tri},
};
use alloc::vec;
use alloc::vec::Vec;

use crate::{
    graphics::{FACE_SPRITES, SELECT_BOX},
    Agb, Die, Face, PlayerDice,
};

include_gfx!("gfx/descriptions.toml");

pub const DESCRIPTIONS_PALETTE: &Palette16 = &descriptions::descriptions.palettes[0];

enum CustomiseState {
    Dice {
        dice: usize,
    },
    Face {
        dice: usize,
        face: usize,
    },
    Upgrade {
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
        obj.set_x((idx as i32 * 32 - 24 / 2 + 20) as u16);
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

fn upgrade_position(idx: usize) -> (u32, u32) {
    (
        (WIDTH - 80) as u32,
        (idx * 32 + HEIGHT as usize - 3 * 32) as u32,
    )
}

fn create_upgrade_objects<'a>(gfx: &'a ObjectController, upgrades: &[Face]) -> Vec<Object<'a>> {
    let mut objects = Vec::new();
    for (idx, &upgrade) in upgrades.iter().enumerate() {
        let mut obj = gfx.object(gfx.sprite(FACE_SPRITES.sprite_for_face(upgrade)));
        let (x, y) = upgrade_position(idx);
        obj.set_x((x - 24 / 2) as u16);
        obj.set_y((y - 24 / 2) as u16);

        obj.show();

        objects.push(obj);
    }
    objects
}

fn generate_upgrades(difficulty: u32) -> Vec<Face> {
    vec![Face::Attack, Face::Shield, Face::Malfunction]
}

pub(crate) fn customise_screen(
    agb: &mut Agb,
    mut player_dice: PlayerDice,
    descriptions_map: &mut RegularMap,
) -> PlayerDice {
    descriptions_map.set_scroll_pos((u16::MAX - 174, u16::MAX - 52).into());

    descriptions_map.show();

    let descriptions_tileset = TileSet::new(
        descriptions::descriptions.tiles,
        agb::display::tiled::TileFormat::FourBpp,
    );

    // create the dice

    let mut _net = create_net(&agb.obj, &player_dice.dice[0]);
    let mut _dice = create_dice_display(&agb.obj, &player_dice);

    let mut upgrades = generate_upgrades(0);
    let mut _upgrade_objects = create_upgrade_objects(&agb.obj, &upgrades);

    let mut input = agb::input::ButtonController::new();

    let mut select_box = agb.obj.object(agb.obj.sprite(SELECT_BOX.sprite(0)));

    select_box.show();

    let mut counter = 0usize;

    let mut state = CustomiseState::Dice { dice: 0 };

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
            CustomiseState::Dice { dice } => {
                let new_dice = (*dice as isize + lr as isize)
                    .rem_euclid(player_dice.dice.len() as isize)
                    as usize;
                if new_dice != *dice {
                    *dice = new_dice;
                    _net = create_net(&agb.obj, &player_dice.dice[*dice]);
                }

                select_box.set_x((*dice as i32 * 32 - 32 / 2 + 20) as u16);
                select_box.set_y(0);

                if input.is_just_pressed(Button::A) {
                    state = CustomiseState::Face {
                        dice: *dice,
                        face: 1,
                    }
                }
            }
            CustomiseState::Face { dice, face } => {
                *face = move_net_position_lr(*face, lr);
                *face = move_net_position_ud(*face, ud);

                let (x, y) = screen_position_for_index(*face);
                select_box.set_x((x - 32 / 2) as u16);
                select_box.set_y((y - 32 / 2) as u16);

                if input.is_just_pressed(Button::B) {
                    state = CustomiseState::Dice { dice: *dice };
                } else if input.is_just_pressed(Button::A) && !upgrades.is_empty() {
                    state = CustomiseState::Upgrade {
                        dice: *dice,
                        face: *face,
                        upgrade: upgrades.len(),
                    };
                }
            }
            CustomiseState::Upgrade {
                dice,
                face,
                upgrade,
            } => {
                let old_updade = *upgrade;
                *upgrade =
                    (*upgrade as isize + ud as isize).rem_euclid(upgrades.len() as isize) as usize;

                if *upgrade != old_updade {
                    for y in 0..11 {
                        for x in 0..8 {
                            descriptions_map.set_tile(
                                &mut agb.vram,
                                (x, y).into(),
                                &descriptions_tileset,
                                TileSetting::new(
                                    y * 8 + x + 8 * 11 * upgrades[*upgrade] as u16,
                                    false,
                                    false,
                                    1,
                                ),
                            )
                        }
                    }
                }

                let (x, y) = upgrade_position(*upgrade);
                select_box.set_x((x - 32 / 2) as u16);
                select_box.set_y((y - 32 / 2) as u16);

                if input.is_just_pressed(Button::B) {
                    state = CustomiseState::Face {
                        dice: *dice,
                        face: *face,
                    };
                } else if input.is_just_pressed(Button::A)
                    && player_dice.dice[*dice].faces[*face] != upgrades[*upgrade]
                {
                    player_dice.dice[*dice].faces[*face] = upgrades[*upgrade];
                    upgrades.remove(*upgrade);
                    _upgrade_objects = create_upgrade_objects(&agb.obj, &upgrades);

                    _net = create_net(&agb.obj, &player_dice.dice[*dice]);
                    _dice = create_dice_display(&agb.obj, &player_dice);
                    state = CustomiseState::Face {
                        dice: *dice,
                        face: *face,
                    };
                }
            }
        }

        if input.is_just_pressed(Button::START) {
            break;
        }

        select_box.set_sprite(agb.obj.sprite(SELECT_BOX.animation_sprite(counter / 10)));

        agb.star_background.update();
        let _ = agb::rng::gen();
        agb.vblank.wait_for_vblank();
        agb.obj.commit();
        descriptions_map.commit(&mut agb.vram);
        agb.star_background.commit(&mut agb.vram);
    }

    descriptions_map.hide();

    player_dice
}
