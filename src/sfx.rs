use agb::sound::mixer::{Mixer, SoundChannel};
use agb::{include_wav, rng};

const DICE_ROLLS: &[&[u8]] = &[
    include_wav!("sfx/SingleRoll_1.wav"),
    include_wav!("sfx/SingleRoll_2.wav"),
    include_wav!("sfx/SingleRoll_3.wav"),
    include_wav!("sfx/SingleRoll_4.wav"),
    include_wav!("sfx/SingleRoll_5.wav"),
];

const MULTI_ROLLS: &[&[u8]] = &[
    include_wav!("sfx/MultiRoll_1.wav"),
    include_wav!("sfx/MultiRoll_2.wav"),
    include_wav!("sfx/MultiRoll_3.wav"),
    include_wav!("sfx/MultiRoll_4.wav"),
    include_wav!("sfx/MultiRoll_5.wav"),
];

pub struct Sfx<'a> {
    mixer: &'a mut Mixer,
}

impl<'a> Sfx<'a> {
    pub fn new(mixer: &'a mut Mixer) -> Self {
        Self { mixer }
    }

    pub fn frame(&mut self) {
        self.mixer.frame();
    }

    pub fn roll(&mut self) {
        let roll_sound_to_use = rng::gen().rem_euclid(DICE_ROLLS.len() as i32);
        let sound_channel = SoundChannel::new(DICE_ROLLS[roll_sound_to_use as usize]);

        self.mixer.play_sound(sound_channel);
    }

    pub fn roll_multi(&mut self) {
        let roll_sound_to_use = rng::gen().rem_euclid(MULTI_ROLLS.len() as i32);
        let sound_channel = SoundChannel::new(MULTI_ROLLS[roll_sound_to_use as usize]);

        self.mixer.play_sound(sound_channel);
    }
}
