use agb::display::object::{Sprite, Tag};

use crate::{Face, Ship};

const DICE_FACES: &agb::display::object::Graphics =
    agb::include_aseprite!("gfx/dice-faces.aseprite");
pub const FACE_SPRITES: &FaceSprites = &FaceSprites::load_face_sprites();
pub const SELECT_BOX: &Tag = DICE_FACES.tags().get("selection");

const SHIPS: &agb::display::object::Graphics = agb::include_aseprite!("gfx/ships.aseprite");
pub const SHIP_SPRITES: &ShipSprites = &ShipSprites::load_ship_sprites();

const SMALL_SPRITES_GFX: &agb::display::object::Graphics =
    agb::include_aseprite!("gfx/small-sprites.aseprite");
pub const SMALL_SPRITES: &SmallSprites = &SmallSprites {};

pub struct FaceSprites {
    sprites: [&'static Sprite; 3],
}

impl FaceSprites {
    const fn load_face_sprites() -> Self {
        const S_SHOOT: &Sprite = DICE_FACES.tags().get("shoot").sprite(0);
        const S_SHIELD: &Sprite = DICE_FACES.tags().get("shield").sprite(0);
        const S_MALFUNCTION: &Sprite = DICE_FACES.tags().get("malfunction").sprite(0);
        Self {
            sprites: [S_SHOOT, S_SHIELD, S_MALFUNCTION],
        }
    }

    pub fn sprite_for_face(&self, face: Face) -> &'static Sprite {
        self.sprites[face as usize]
    }
}

pub struct ShipSprites {
    sprites: [&'static Sprite; 3],
}

impl ShipSprites {
    const fn load_ship_sprites() -> Self {
        const S_PLAYER: &Sprite = SHIPS.tags().get("player").sprite(0);
        const S_DRONE: &Sprite = SHIPS.tags().get("drone").sprite(0);
        const S_SHIELD: &Sprite = SHIPS.tags().get("shield").sprite(0);

        Self {
            sprites: [S_PLAYER, S_DRONE, S_SHIELD],
        }
    }

    pub fn sprite_for_ship(&self, ship: Ship) -> &'static Sprite {
        self.sprites[ship as usize]
    }
}

pub struct SmallSprites;

impl SmallSprites {
    pub const fn number(i: u32) -> &'static Sprite {
        SMALL_SPRITES_GFX.tags().get("numbers").sprite(i as usize)
    }

    pub const fn slash() -> &'static Sprite {
        SMALL_SPRITES_GFX.tags().get("numbers").sprite(10)
    }

    pub const fn red_bar(i: usize) -> &'static Sprite {
        SMALL_SPRITES_GFX.tags().get("red bar").sprite(i)
    }
}
