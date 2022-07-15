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

use agb::rng::RandomNumberGenerator;
use agb::{display, syscall};

extern crate alloc;
use alloc::vec::Vec;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Face {
    Attack,
    Shield,
    Malfunction,
}

#[derive(Debug)]
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

#[derive(Debug)]
struct PlayerDice {
    dice: Vec<Die>,
}

/// A face of the rolled die and it's cooldown (should it be a malfunction)
#[derive(Debug)]

struct RolledDie {
    face: Face,
    cooldown: u32,
}

impl RolledDie {
    fn update(&mut self) {
        self.cooldown = self.cooldown.wrapping_sub(1)
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
    rolled_dice: RolledDice,
}

fn main(mut gba: agb::Gba) -> ! {
    let gfx = gba.display.object.get();
    let vblank = agb::interrupt::VBlank::get();

    loop {
        vblank.wait_for_vblank();
        gfx.commit();
    }
}

#[agb::entry]
fn entry(mut gba: agb::Gba) -> ! {
    main(gba)
}
