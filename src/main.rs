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

use agb::display::object::{ObjectController, Sprite};
use agb::hash_map::HashMap;
use agb::interrupt::VBlank;
use agb::rng::RandomNumberGenerator;
use agb::{display, syscall};

extern crate alloc;
use alloc::vec::Vec;

const FACE_SPRITES: &FaceSprites = &FaceSprites::load_face_sprites();

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
        const DICE_FACES: &agb::display::object::Graphics =
            agb::include_aseprite!("gfx/dice-faces.aseprite");

        const S_SHOOT: &Sprite = DICE_FACES.tags().get("shoot").sprite(0);
        const S_SHIELD: &Sprite = DICE_FACES.tags().get("shoot").sprite(0);
        const S_MALFUNCTION: &Sprite = DICE_FACES.tags().get("shoot").sprite(0);
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

struct Agb {
    obj: ObjectController,
    vblank: VBlank,
}

fn battle_screen(agb: &mut Agb, player_dice: PlayerDice) {
    loop {
        agb.vblank.wait_for_vblank();
        agb.obj.commit();
    }
}
fn customise_screen(agb: &mut Agb, player_dice: PlayerDice) -> PlayerDice {
    loop {
        agb.vblank.wait_for_vblank();
        agb.obj.commit();
    }
}

fn main(mut gba: agb::Gba) -> ! {
    let gfx = gba.display.object.get();
    let vblank = agb::interrupt::VBlank::get();

    let mut agb = Agb { obj: gfx, vblank };

    let mut dice = PlayerDice { dice: Vec::new() };

    loop {
        battle_screen(&mut agb, dice.clone());
        dice = customise_screen(&mut agb, dice.clone());
    }
}

#[agb::entry]
fn entry(mut gba: agb::Gba) -> ! {
    main(gba)
}
