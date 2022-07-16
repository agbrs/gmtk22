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
use agb::display::object::ObjectController;
use agb::display::tiled::VRamManager;
use agb::display::Priority;
use agb::interrupt::VBlank;

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

mod background;
mod battle;
mod customise;
mod graphics;

use background::StarBackground;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Face {
    Attack,
    Shield,
    Malfunction,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Ship {
    Player,
    Drone,
    Shield,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum EnemyAttackType {
    Attack,
    Shield,
    Heal,
}

#[derive(Debug, Clone)]
pub struct Die {
    faces: [Face; 6],
}

impl Die {
    /// roll this die (potentially using the custom probabilities, should we implement that) and return which face index is showing
    fn roll(&self) -> Face {
        let n = agb::rng::gen().rem_euclid(6);
        self.faces[n as usize]
    }
}

#[derive(Debug, Clone)]
pub struct PlayerDice {
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

    let basic_die = Die {
        faces: [
            Face::Attack,
            Face::Attack,
            Face::Attack,
            Face::Malfunction,
            Face::Shield,
            Face::Shield,
        ],
    };

    let mut dice = PlayerDice {
        dice: vec![basic_die; 5],
    };

    loop {
        dice = customise::customise_screen(&mut agb, dice.clone());

        battle::battle_screen(&mut agb, dice.clone());
    }
}

#[agb::entry]
fn entry(mut gba: agb::Gba) -> ! {
    main(gba)
}
