// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
//
// Provided you haven't disabled it, agb does provide an allocator, so it is possible
// to use both the `core` and the `alloc` built in crates.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![no_main]

use agb::display;
use agb::display::object::{ObjectController, Sprite, Tag};
use agb::display::tiled::VRamManager;
use agb::display::Priority;
use agb::interrupt::VBlank;
use agb::rng::RandomNumberGenerator;

extern crate alloc;
use alloc::vec::Vec;

mod background;
mod battle;
mod customise;

use background::StarBackground;

const DICE_FACES: &agb::display::object::Graphics =
    agb::include_aseprite!("gfx/dice-faces.aseprite");

const FACE_SPRITES: &FaceSprites = &FaceSprites::load_face_sprites();
const SELECT_BOX: &Tag = DICE_FACES.tags().get("selection");

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Face {
    Attack,
    Shield,
    Malfunction,
}

struct FaceSprites {
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

    fn sprite_for_face(&self, face: Face) -> &'static Sprite {
        self.sprites[face as usize]
    }
}

#[derive(Debug, Clone)]
struct Die {
    faces: [Face; 6],
}

impl Die {
    /// roll this die (potentially using the custom probabilities, should we implement that) and return which face index is showing
    fn roll(&self, rng: &mut RandomNumberGenerator) -> Face {
        let n = rng.gen().rem_euclid(6);
        self.faces[n as usize]
    }
}

#[derive(Debug, Clone)]
struct PlayerDice {
    dice: Vec<Die>,
}

struct Agb<'a> {
    obj: ObjectController,
    vblank: VBlank,
    star_background: StarBackground<'a>,
    vram: VRamManager,
}

fn main(mut gba: agb::Gba) -> ! {
    let gfx = gba.display.object.get();
    let vblank = agb::interrupt::VBlank::get();

    let (tiled, mut vram) = gba.display.video.tiled0();
    let mut background0 = tiled.background(
        Priority::P0,
        display::tiled::RegularBackgroundSize::Background64x32,
    );
    let mut background1 = tiled.background(
        Priority::P0,
        display::tiled::RegularBackgroundSize::Background64x32,
    );

    background::load_palettes(&mut vram);
    background0.show();
    background1.show();

    let mut star_background = StarBackground::new(&mut background0, &mut background1, &mut vram);

    star_background.commit(&mut vram);

    let mut agb = Agb {
        obj: gfx,
        vblank,
        star_background,
        vram,
    };

    let mut dice = PlayerDice { dice: Vec::new() };

    loop {
        dice = customise::customise_screen(&mut agb, dice.clone());

        battle::battle_screen(&mut agb, dice.clone());
    }
}

#[agb::entry]
fn entry(mut gba: agb::Gba) -> ! {
    main(gba)
}
